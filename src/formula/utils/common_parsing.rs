use crate::CellAddress;

pub(crate) fn parse_cell_address(text: &str) -> Option<CellAddress> {
    let (column, row) = text.split_once(',')?;
    Some(CellAddress::new(column.parse().ok()?, row.parse().ok()?))
}