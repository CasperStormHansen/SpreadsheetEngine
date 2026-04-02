use std::collections::HashSet;
use crate::cell_region::CellRegion;
use crate::cell_value::CellValue;
use crate::formula::cell_reference::CellReference;
use crate::formula::number_literal::NumberLiteral;
use crate::formula::parsing_error::ParsingError;
use crate::spreadsheet::Spreadsheet;

pub(crate) trait Formula {
    fn try_parse(input: &str) -> Option<Self>
    where
        Self: Sized;
    fn get_child_regions(&self) -> HashSet<CellRegion>;
    fn evaluate(&self, spreadsheet: &Spreadsheet) -> CellValue;
}

macro_rules! try_parse_in_order {
    ($raw_formula:expr, $($ty:ty),+) => {{
        $(
            if let Some(formula) = <$ty>::try_parse($raw_formula) {
                return Box::new(formula);
            }
        )+
        panic!("The last type of formula must be ParsingError, which always succeeds.")
    }};
}

pub(crate) fn parse(raw_formula: &str) -> Box<dyn Formula> {
    try_parse_in_order!(raw_formula, 
        NumberLiteral, 
        CellReference, 
        ParsingError)
}

mod number_literal;
mod cell_reference;
mod parsing_error;