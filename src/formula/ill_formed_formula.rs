use std::collections::HashSet;
use crate::cell_region::CellRegion;
use crate::cell_value::CellValue;
use crate::formula::Formula;
use crate::Spreadsheet;

pub(crate) struct IllFormedFormula {
    error_message: String
}

impl Formula for IllFormedFormula {
    fn evaluate(&self, _spreadsheet: &Spreadsheet) -> CellValue {
        CellValue::Error(self.error_message.clone())
    }

    fn get_child_regions(&self) -> HashSet<CellRegion> {
        HashSet::new()
    }
}

impl IllFormedFormula {
    pub(crate) fn new(error_message: &str) -> Self {
        Self {error_message: error_message.to_string()} // Todo: needs improvement
    }
}
