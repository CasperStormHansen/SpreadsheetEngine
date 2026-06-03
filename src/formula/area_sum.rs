use crate::cell_lookup_structure::cell_rectangle::CellRectangle;
use crate::formula::utils::common_parsing::parse_cell_address;
use crate::formula::utils::normalized_raw_formula::NormalizedRawFormula;
use crate::formula::{Formula, WellFormedFormula};
use crate::value_types::EvaluatedValue::{Boolean, Error, Number, Text};
use crate::value_types::{EvaluationResult, CompletedEvaluationResult};
use crate::Spreadsheet;
use std::collections::HashSet;

pub(crate) struct AreaSum {
    area: CellRectangle
}

impl Formula for AreaSum {
    // Todo: This can be optimized. See the todo above attach_to_children in Spreadsheet.
    fn evaluate(&self, spreadsheet: &Spreadsheet) -> EvaluationResult {
        let mut sum = 0.0;
        let values = spreadsheet.cells
            .get_all_in_rectangle(&self.area)
            .map(|(_, cell)| &cell.value);
        let child_rectangles = self.get_initial_child_rectangles();
        for value in values {
            match value {
                Some(Number(number)) =>
                    sum += number,
                Some(Boolean(_)) =>
                    return Ok(CompletedEvaluationResult(
                        Error("Summing over area with boolean".to_string()),
                        child_rectangles)),
                Some(Text(_)) =>
                    return Ok(CompletedEvaluationResult(
                        Error("Summing over area with text".to_string()),
                        child_rectangles)),
                Some(Error(_)) =>
                    return Ok(CompletedEvaluationResult(
                        Error("Summing over area with error".to_string()),
                        child_rectangles)),
                None =>
                    return Err(child_rectangles)
            }
        }
        Ok(CompletedEvaluationResult(Number(sum), child_rectangles))
    }

    fn get_initial_child_rectangles(&self) -> HashSet<CellRectangle> {
        HashSet::from([self.area.clone()])
    }
}

impl WellFormedFormula for AreaSum {
    // TODO: Accept cell addresses in the letter-number format.
    fn try_parse(raw_formula: &NormalizedRawFormula) -> Option<Self> {
        let inner = raw_formula.strip_prefix("sum(")?.strip_suffix(')')?;
        let (left, right) = inner.split_once(':')?;
        let upper_left = parse_cell_address(left)?;
        let lower_right = parse_cell_address(right)?;
        let rectangle = CellRectangle::new(upper_left, lower_right)?;
        Some(Self{area: rectangle})
    }
}
