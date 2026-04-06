use std::collections::HashSet;
use crate::formula::{Formula, WellFormedFormula};
use crate::{CellValue, Spreadsheet};
use crate::cell_region::CellRegion;
use crate::formula::utils::common_parsing::parse_cell_address;
use crate::formula::utils::string_without_whitespace::StringWithoutWhitespace;

pub(crate) struct AreaSum {
    area: CellRegion
}

impl Formula for AreaSum {
    // Todo: This can be optimized. See the todo above attach_to_parents in Spreadsheet.
    fn evaluate(&self, spreadsheet: &Spreadsheet) -> CellValue {
        let mut sum = 0.0;
        let values = spreadsheet.cells.iter()
            .filter(|(cell_address, _)| self.area.contains(cell_address))
            .map(|(_, cell_value)| &cell_value.value);
        for value in values {
            match value {
                CellValue::Number(number) =>
                    sum += number,
                CellValue::Error(_) =>
                    return CellValue::Error("Summing over area with error".to_string()),
                CellValue::Unevaluated =>
                    panic!("Evaluation of cell with unevaluated children has been triggered") // TODO: This should not have to be handled in each formula module.
            }
        }
        CellValue::Number(sum)
    }

    fn get_child_regions(&self) -> HashSet<CellRegion> {
        HashSet::from([self.area.clone()])
    }
}

impl WellFormedFormula for AreaSum {
    // TODO: Accept cell addresses in the letter-number format.
    fn try_parse(raw_formula: &StringWithoutWhitespace) -> Option<Self> {
        let inner = raw_formula.strip_prefix("SUM(")?.strip_suffix(')')?;
        let (left, right) = inner.split_once(':')?;
        let upper_left = parse_cell_address(left)?;
        let lower_right = parse_cell_address(right)?;
        let rectangle = CellRegion::new_rectangle(upper_left, lower_right)?;
        Some(Self{area: rectangle})
    }
}
