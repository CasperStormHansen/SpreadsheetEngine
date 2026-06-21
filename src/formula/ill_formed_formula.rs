use crate::cell_lookup_structure::cell_rectangle::CellRectangle;
use crate::formula::Formula;
use crate::value_types::EvaluatedValue::SingleCellValue;
use crate::value_types::{EvaluationResult, CompletedEvaluationResult};
use crate::Spreadsheet;
use std::collections::HashSet;
use crate::value_types::SingleCellValue::Error;

pub(crate) struct IllFormedFormula {
    error_message: String
}

impl Formula for IllFormedFormula {
    fn evaluate(&self, _spreadsheet: &Spreadsheet) -> EvaluationResult {
        Ok(CompletedEvaluationResult(SingleCellValue(
            Error(self.error_message.clone())),
            self.get_initial_child_rectangles()
        ))
    }

    fn get_initial_child_rectangles(&self) -> HashSet<CellRectangle> {
        HashSet::new()
    }

    fn is_volatile(&self) -> bool {
        false
    }
}

impl IllFormedFormula {
    pub(crate) fn new(error_message: &str) -> Self {
        Self {error_message: error_message.to_string() } // Todo: needs improvement
    }
}
