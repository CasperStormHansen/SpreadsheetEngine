#[macro_export]
macro_rules! adr {
    ($column:expr, $row:expr) => {
        CellAddress::new($column, $row)
    };
}

#[macro_export]
macro_rules! assert_value {
    ($spreadsheet:expr, $address:expr, $expected:expr $(,)?) => {{
        assert_eq!(
            $spreadsheet.cell_value($address),
            Some($expected),
        );
    }};
}

#[macro_export]
macro_rules! assert_empty {
    ($spreadsheet:expr, $address:expr) => {{
        assert_eq!(
            $spreadsheet.cell_value($address),
            None,
        );
    }};
}