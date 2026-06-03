use crate::cell_rectangle::CellRectangle;
use crate::formula::Formula;
use crate::value_types::EvaluatedValue::Error;
use crate::value_types::{EvaluationResult, CompletedEvaluationResult};
use crate::Spreadsheet;
use std::collections::HashSet;

pub(crate) struct IllFormedFormula {
    error_message: String
}

impl Formula for IllFormedFormula {
    fn evaluate(&self, _spreadsheet: &Spreadsheet) -> EvaluationResult {
        Ok(CompletedEvaluationResult(
            Error(self.error_message.clone()),
            self.get_initial_child_rectangles()
        ))
    }

    fn get_initial_child_rectangles(&self) -> HashSet<CellRectangle> {
        HashSet::new()
    }
}

impl IllFormedFormula {
    pub(crate) fn new(error_message: &str) -> Self {
        Self {error_message: error_message.to_string() } // Todo: needs improvement
    }
}
