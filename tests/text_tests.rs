mod common_test;

use spreadsheet_engine::Spreadsheet;
use spreadsheet_engine::CellAddress;
use spreadsheet_engine::value_types::SingleCellValue::*;

#[test]
fn simple_text_literal() {
    let mut spreadsheet = Spreadsheet::new();
    spreadsheet.input_raw_formula(adr![0, 0], r#""text""#);

    assert_value!(spreadsheet, adr![0, 0], Text("text".to_string()));
}

#[test]
fn text_literal_with_spaces_and_capitalization() {
    let mut spreadsheet = Spreadsheet::new();
    spreadsheet.input_raw_formula(adr![0, 0], r#""Test Text""#);

    assert_value!(spreadsheet, adr![0, 0], Text("Test Text".to_string()));
}

#[test]
fn text_literal_error() {
    let mut spreadsheet = Spreadsheet::new();
    spreadsheet.input_raw_formula(adr![0, 0], r#""Test Text"#);

    assert_value!(spreadsheet, adr![0, 0], Error(r#""Test Text"#.to_string()));
}