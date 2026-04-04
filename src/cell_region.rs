use crate::cell_address::CellAddress;

#[derive(PartialEq, Eq, Hash, Debug, Clone)]
pub(crate) enum CellRegion {
    SingleCell{single_cell_address: CellAddress},
    Rectangle{upper_left: CellAddress, lower_right: CellAddress},
}

impl CellRegion {
    pub(crate) fn new_single_cell(cell_address: CellAddress) -> Self {
        CellRegion::SingleCell { single_cell_address: cell_address }
    }

    pub(crate) fn new_rectangle(upper_left: CellAddress, lower_right: CellAddress) -> Option<Self> {
        if upper_left.column <= lower_right.column && upper_left.row <= lower_right.row {
            return Some(CellRegion::Rectangle { upper_left, lower_right });
        }
        None
    }
    
    pub(crate) fn contains(& self, cell_address: &CellAddress) -> bool {
        match self {
            CellRegion::SingleCell{ single_cell_address} =>
                single_cell_address == cell_address,
            CellRegion::Rectangle{ upper_left, lower_right } => {
                upper_left.row <= cell_address.row && cell_address.row <= lower_right.row &&
                upper_left.column <= cell_address.column && cell_address.column <= lower_right.column
            }
        }
    }
}