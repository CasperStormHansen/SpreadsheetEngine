use std::collections::HashSet;
use std::str::FromStr;
use crate::cell_region::CellRegion;
use crate::cell_value::CellValue;
use crate::formula::{Formula, WellFormedFormula};
use crate::formula::utils::string_without_whitespace::StringWithoutWhitespace;
use crate::Spreadsheet;

pub(crate) struct NumberLiteral {
    number: f64
}

impl Formula for NumberLiteral {
    fn evaluate(&self, _spreadsheet: &Spreadsheet) -> CellValue {
        CellValue::Number(self.number)
    }

    fn get_child_regions(&self) -> HashSet<CellRegion> {
        HashSet::new()
    }
}

impl WellFormedFormula for NumberLiteral {
    fn try_parse(raw_formula: &StringWithoutWhitespace) -> Option<Self> {
        f64::from_str(raw_formula).ok().map(|number| Self { number })
    }
}