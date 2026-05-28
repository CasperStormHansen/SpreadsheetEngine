mod common_test;

use spreadsheet_engine::CellAddress;
use spreadsheet_engine::CellValue::{Boolean, Number};
use spreadsheet_engine::Spreadsheet;

#[test]
fn single_cell_boolean_literal() {
    let test_cases = [
        ("true", true),
        ("TRUE", true),
        ("   false", false),
        (" False ", false),
    ];

    for (raw_formula, expected_value) in test_cases {
        let mut spreadsheet = Spreadsheet::new();
        spreadsheet.input_raw_formula(adr![0, 0], raw_formula);

        assert_value!(spreadsheet, adr![0, 0], Boolean(expected_value));
    }
}

#[test]
fn one_cell_referencing_boolean_cell() {
    let mut spreadsheet = Spreadsheet::new();
    spreadsheet.input_raw_formula(adr![0, 0], "(0,1)");

    assert_value!(spreadsheet, adr![0, 0], Number(0.0));

    spreadsheet.input_raw_formula(adr![0, 1], "true");

    assert_value!(spreadsheet, adr![0, 0], Boolean(true));
    assert_value!(spreadsheet, adr![0, 1], Boolean(true));
}