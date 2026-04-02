use std::collections::HashSet;
use std::str::FromStr;
use crate::cell_region::CellRegion;
use crate::cell_value::CellValue;
use crate::formula::{Formula, ProperFormula};
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

impl ProperFormula for NumberLiteral {
    fn try_parse(input: &str) -> Option<Self> {
        f64::from_str(input).ok().map(|number| Self { number })
    }
}