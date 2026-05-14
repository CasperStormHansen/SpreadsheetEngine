use crate::cell_address::CellAddress;

#[derive(PartialEq, Eq, Hash, Debug, Clone)]
pub(crate) struct CellRectangle {
    pub(crate) upper_left: CellAddress, 
    pub(crate) lower_right: CellAddress,
}

impl CellRectangle {
    pub(crate) fn new(upper_left: CellAddress, lower_right: CellAddress) -> Option<Self> {
        if upper_left.column <= lower_right.column && upper_left.row <= lower_right.row {
            return Some(CellRectangle { upper_left, lower_right });
        }
        None
    }
}