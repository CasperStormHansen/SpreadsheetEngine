use std::collections::HashSet;
use crate::cell_region::CellRegion;
use crate::cell_value::CellValue;
use crate::formula::Formula;
use crate::Spreadsheet;

pub(crate) struct ParsingError {
    error_message: String
}

impl Formula for ParsingError {
    fn try_parse(input: &str) -> Option<Self> {
        Some(Self {error_message: input.to_string()}) // Todo: needs improvement
    }

    fn get_child_regions(&self) -> HashSet<CellRegion> {
        HashSet::new()
    }

    fn evaluate(&self, _spreadsheet: &Spreadsheet) -> CellValue {
        CellValue::Error(self.error_message.clone())
    }
}