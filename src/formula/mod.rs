use std::collections::HashSet;
use crate::cell_region::CellRegion;
use crate::cell_value::CellValue;
use crate::formula::cell_reference::CellReference;
use crate::formula::number_literal::NumberLiteral;
use crate::formula::ill_formed_formula::IllFormedFormula;
use crate::spreadsheet::Spreadsheet;

pub(crate) trait Formula {
    fn evaluate(&self, spreadsheet: &Spreadsheet) -> CellValue;
    fn get_child_regions(&self) -> HashSet<CellRegion>;
}

trait WellFormedFormula: Formula {
    fn try_parse(raw_formula: &str) -> Option<Self>
    where
        Self: Sized;
}

macro_rules! try_parse_in_order {
    ($raw_formula:expr, $($ty:ty),+ $(,)?) => {{
        $(
            if let Some(formula) = <$ty>::try_parse($raw_formula) {
                return Box::new(formula);
            }
        )+
        Box::new(IllFormedFormula::new($raw_formula))
    }};
}

pub(crate) fn parse(raw_formula: &str) -> Box<dyn Formula> {
    try_parse_in_order!(raw_formula, 
        NumberLiteral, 
        CellReference, 
    )
}

mod number_literal;
mod cell_reference;
mod ill_formed_formula;