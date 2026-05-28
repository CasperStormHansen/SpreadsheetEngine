use std::collections::HashSet;
use std::str::FromStr;
use crate::formula::{Formula, WellFormedFormula};
use crate::{CellValue, Spreadsheet};
use crate::cell_rectangle::CellRectangle;
use crate::formula::utils::normalized_raw_formula::NormalizedRawFormula;

pub(crate) struct BooleanLiteral {
    boolean: bool
}

impl Formula for BooleanLiteral {
    fn evaluate(&self, _spreadsheet: &Spreadsheet) -> CellValue {
        CellValue::Boolean(self.boolean)
    }

    fn get_child_rectangles(&self) -> HashSet<CellRectangle> {
        HashSet::new()
    }
}

impl WellFormedFormula for BooleanLiteral {
    fn try_parse(raw_formula: &NormalizedRawFormula) -> Option<Self> {
        bool::from_str(raw_formula).ok().map(|boolean| Self { boolean })
    }
}