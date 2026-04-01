use std::collections::HashSet;
use crate::cell_address::CellAddress;
use crate::formula::Formula;

#[derive(PartialEq, Eq, Hash, Debug)]
pub(crate) enum CellRegion {
    SingleCell(CellAddress),
    //Rectangle(CellAddress, CellAddress), // todo: add
}

impl CellRegion {
    pub(crate) fn get_cell_regions(parsed_formula: &Formula) -> HashSet<CellRegion> {
        match parsed_formula {
            Formula::NumberLiteral(_) => HashSet::new(),
            Formula::CellReference(cell_reference) 
                => HashSet::from([CellRegion::SingleCell(*cell_reference)]),
            Formula::ParsingError(_) => HashSet::new(),
        }
    }

    pub(crate) fn contains(& self, cell_address: &CellAddress) -> bool {
        match self {
            CellRegion::SingleCell(region_address) => region_address == cell_address,
            // CellRegion::Rectangle(upper_left, lower_right) => {
            //     upper_left.row() <= cell_address.row() && cell_address.row() <= lower_right.row() &&
            //     upper_left.column() <= cell_address.column() && cell_address.column() <= lower_right.column()
            // }
        }
    }
}