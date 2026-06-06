use crate::cell_lookup_structure::cell_rectangle::CellRectangle;
use crate::formula::utils::common_parsing::parse_cell_address;
use crate::formula::utils::normalized_raw_formula::NormalizedRawFormula;
use crate::formula::{parse, EvaluationResult, Formula, WellFormedFormula};
use crate::value_types::{CompletedEvaluationResult, UsedChildRectangles};
use crate::value_types::EvaluatedValue::Error;
use crate::EvaluatedValue::{Number, Text};
use crate::{CellAddress, Spreadsheet};
use std::collections::HashSet;

pub(crate) struct Indirect {
    reference: Box<dyn Formula>,
}

impl Formula for Indirect {
    fn evaluate(&self, spreadsheet: &Spreadsheet) -> EvaluationResult {
        match self.reference.evaluate(spreadsheet) {
            Ok(CompletedEvaluationResult(Text(text), child_rectangles)) =>
                continue_evaluation_based_on_evaluated_text(spreadsheet, text, child_rectangles),
            Ok(CompletedEvaluationResult(_, child_rectangles)) =>
                Ok(CompletedEvaluationResult(Error("Indirect reference is not text".to_string()), child_rectangles)),
            Err(request_for_more_child_rectangles) =>
                Err(request_for_more_child_rectangles),
        }
    }

    fn get_initial_child_rectangles(&self) -> HashSet<CellRectangle> {
        HashSet::new()
    }

    fn is_volatile(&self) -> bool {
        self.reference.is_volatile()
    }
}

fn continue_evaluation_based_on_evaluated_text(spreadsheet: &Spreadsheet, text: String, child_rectangles: UsedChildRectangles) -> EvaluationResult {
    if let Some(cell_address) = parse_cell_address(&text) {
        match spreadsheet.cells.get(&cell_address) {
            Some(cell) => {
                match &cell.value {
                    Some(proper_value) =>
                        Ok(CompletedEvaluationResult(proper_value.clone(), combine(child_rectangles, cell_address))),
                    None =>
                        Err(combine(child_rectangles, cell_address))
                }
            }
            None =>
                Ok(CompletedEvaluationResult(Number(0.0), combine(child_rectangles, cell_address))),
        }
    } else {
        Ok(CompletedEvaluationResult(Error("Indirect reference is not a valid cell address".to_string()), child_rectangles))
    }
}

fn combine(mut child_rectangles: HashSet<CellRectangle>, cell_address: CellAddress) -> HashSet<CellRectangle> {
    child_rectangles.insert(
        CellRectangle::new(cell_address.clone(), cell_address.clone()).unwrap()
    );
    child_rectangles
}

impl WellFormedFormula for Indirect {
    fn try_parse(raw_formula: &NormalizedRawFormula) -> Option<Self> {
        let inner = raw_formula.strip_prefix("indirect(")?.strip_suffix(')')?;
        let reference = parse(inner);
        Some(Self{ reference })
    }
}
