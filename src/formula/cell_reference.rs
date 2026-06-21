use crate::cell_lookup_structure::cell_rectangle::CellRectangle;
use crate::formula::utils::common_parsing::parse_cell_address;
use crate::formula::utils::normalized_raw_formula::NormalizedRawFormula;
use crate::formula::{Formula, WellFormedFormula};
use crate::value_types::{EvaluationResult, CompletedEvaluationResult};
use crate::{CellAddress, Spreadsheet};
use std::collections::HashSet;
use crate::value_types::EvaluatedValue::SingleCellValue;
use crate::value_types::SingleCellValue::Number;

pub(crate) struct CellReference {
    cell_address: CellAddress
}

impl Formula for CellReference {
    fn evaluate(&self, spreadsheet: &Spreadsheet) -> EvaluationResult {
        let child_rectangles = self.get_initial_child_rectangles();
        match spreadsheet.cells.get(&self.cell_address) {
            Some(cell) => {
                match &cell.value {
                    Some(proper_value) =>
                        Ok(CompletedEvaluationResult(SingleCellValue(proper_value.clone()), child_rectangles)),
                    None =>
                        Err(child_rectangles)
                }
            }
            None => Ok(CompletedEvaluationResult(SingleCellValue(Number(0.0)), child_rectangles)),
        }
    }
    
    fn get_initial_child_rectangles(&self) -> HashSet<CellRectangle> {
        HashSet::from([
            CellRectangle::new(self.cell_address.clone(), self.cell_address.clone()).unwrap()
        ])
    }

    fn is_volatile(&self) -> bool {
        false
    }
}

impl WellFormedFormula for CellReference {
    fn try_parse(raw_formula: &NormalizedRawFormula) -> Option<Self> {
        let inner = raw_formula.strip_prefix('(')?.strip_suffix(')')?;
        let cell_address = parse_cell_address(inner)?;
        Some(Self{cell_address})
    }
}
