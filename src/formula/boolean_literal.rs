use crate::cell_rectangle::CellRectangle;
use crate::formula::utils::normalized_raw_formula::NormalizedRawFormula;
use crate::formula::{Formula, WellFormedFormula};
use crate::value_types::EvaluatedValue::Boolean;
use crate::value_types::{EvaluationResult, CompletedEvaluationResult};
use crate::Spreadsheet;
use std::collections::HashSet;
use std::str::FromStr;

pub(crate) struct BooleanLiteral {
    boolean: bool
}

impl Formula for BooleanLiteral {
    fn evaluate(&self, _spreadsheet: &Spreadsheet) -> EvaluationResult {
        Ok(CompletedEvaluationResult(
            Boolean(self.boolean),
            self.get_initial_child_rectangles(),
        ))
    }

    fn get_initial_child_rectangles(&self) -> HashSet<CellRectangle> {
        HashSet::new()
    }
}

impl WellFormedFormula for BooleanLiteral {
    fn try_parse(raw_formula: &NormalizedRawFormula) -> Option<Self> {
        bool::from_str(raw_formula).ok().map(|boolean| Self { boolean })
    }
}