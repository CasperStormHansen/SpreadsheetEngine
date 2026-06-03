use crate::cell_lookup_structure::cell_rectangle::CellRectangle;
use crate::formula::utils::normalized_raw_formula::NormalizedRawFormula;
use crate::formula::{EvaluationResult, Formula, WellFormedFormula};
use crate::value_types::EvaluatedValue::Number;
use crate::value_types::CompletedEvaluationResult;
use crate::Spreadsheet;
use std::collections::HashSet;
use std::str::FromStr;

pub(crate) struct NumberLiteral {
    number: f64
}

impl Formula for NumberLiteral {
    fn evaluate(&self, _spreadsheet: &Spreadsheet) -> EvaluationResult {
        Ok(CompletedEvaluationResult(
            Number(self.number),
            self.get_initial_child_rectangles(),
        ))
    }

    fn get_initial_child_rectangles(&self) -> HashSet<CellRectangle> {
        HashSet::new()
    }
}

impl WellFormedFormula for NumberLiteral {
    fn try_parse(raw_formula: &NormalizedRawFormula) -> Option<Self> {
        f64::from_str(raw_formula).ok().map(|number| Self { number })
    }
}