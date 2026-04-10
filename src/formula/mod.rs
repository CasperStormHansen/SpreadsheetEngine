use std::collections::HashSet;
use crate::cell_rectangle::CellRectangle;
use crate::cell_value::CellValue;
use crate::formula::area_sum::AreaSum;
use crate::formula::cell_reference::CellReference;
use crate::formula::number_literal::NumberLiteral;
use crate::formula::ill_formed_formula::IllFormedFormula;
use crate::formula::utils::normalized_raw_formula::NormalizedRawFormula;
use crate::spreadsheet::Spreadsheet;

pub(crate) trait Formula {
    fn evaluate(&self, spreadsheet: &Spreadsheet) -> CellValue;
    fn get_child_rectangles(&self) -> HashSet<CellRectangle>;
}

trait WellFormedFormula: Formula {
    fn try_parse(raw_formula: &NormalizedRawFormula) -> Option<Self>
    where
        Self: Sized;
}

macro_rules! try_parse_in_order {
    ($raw_formula:expr, $($ty:ty),+ $(,)?) => {{
        let raw_formula: &str = $raw_formula;
        let raw_formula_without_whitespace = NormalizedRawFormula::from(raw_formula);
        $(
            if let Some(formula) = <$ty>::try_parse(&raw_formula_without_whitespace) {
                return Box::new(formula);
            }
        )+
        Box::new(IllFormedFormula::new(raw_formula))
    }};
}

pub(crate) fn parse(raw_formula: &str) -> Box<dyn Formula> {
    try_parse_in_order!(raw_formula,
        NumberLiteral, 
        CellReference,
        AreaSum
    )
}

mod number_literal;
mod cell_reference;
mod ill_formed_formula;
mod area_sum;
mod utils;