use std::collections::HashSet;
use crate::cell_value::CellValue;
use crate::formula::{Formula, WellFormedFormula};
use crate::{CellAddress, Spreadsheet};
use crate::cell_region::CellRegion;

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
    
    fn get_child_regions(&self) -> HashSet<CellRegion> {
        HashSet::from([CellRegion::SingleCell(self.cell_address.clone())])
    }
}

impl WellFormedFormula for CellReference {
    fn try_parse(raw_formula: &str) -> Option<Self> {
        let input_without_parenthesis = raw_formula.strip_prefix('(')?.strip_suffix(')')?;
        let mut parts = input_without_parenthesis.split(',');

        let column = parts.next()?.trim().parse::<u32>().ok()?;
        let row = parts.next()?.trim().parse::<u32>().ok()?;

        if parts.next().is_some() {
            return None;
        }

        Some(Self{cell_address: CellAddress::new(column, row)})
    }
}
