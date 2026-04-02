#[derive(Clone, Copy, Eq, Hash, PartialEq, Debug)]
pub struct CellAddress {
    column: u32,
    row: u32,
}

impl CellAddress {
    pub fn new(column: u32, row: u32) -> Self {
        Self { column, row }
    }
}
