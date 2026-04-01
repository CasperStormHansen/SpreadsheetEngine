#[derive(Clone, Copy, Eq, Hash, PartialEq, Debug)] // TODO: Consider!
pub struct CellAddress {
    column: u32,
    row: u32, // TODO: Right size?
}

impl CellAddress {
    pub fn new(column: u32, row: u32) -> Self {
        Self { column, row }
    }

    // pub(crate) fn row(&self) -> u32 {
    //     self.row
    // }
    // 
    // pub(crate) fn column(&self) -> u32 {
    //     self.column
    // }
}
