use std::collections::HashSet;
use crate::cell_value::CellValue;
use crate::formula::{Formula, WellFormedFormula};
use crate::{CellAddress, Spreadsheet};
use crate::cell_rectangle::CellRectangle;
use crate::formula::utils::common_parsing::parse_cell_address;
use crate::formula::utils::normalized_raw_formula::NormalizedRawFormula;

pub(crate) struct CellReference {
    cell_address: CellAddress
}

impl Formula for CellReference {
    fn evaluate(&self, spreadsheet: &Spreadsheet) -> CellValue {
        match spreadsheet.cells.get(&self.cell_address) {
            Some(cell) => cell.value.clone(),
            None => CellValue::Number(0.0),
        }
    }
    
    fn get_child_rectangles(&self) -> HashSet<CellRectangle> {
        HashSet::from([
            CellRectangle::new(self.cell_address.clone(), self.cell_address.clone()).unwrap()
        ])
    }
}

impl WellFormedFormula for CellReference {
    fn try_parse(raw_formula: &NormalizedRawFormula) -> Option<Self> {
        let inner = raw_formula.strip_prefix('(')?.strip_suffix(')')?;
        let cell_address = parse_cell_address(inner)?;
        Some(Self{cell_address})
    }
}
