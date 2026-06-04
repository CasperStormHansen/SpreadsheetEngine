mod common_test;

use spreadsheet_engine::CellAddress;
use spreadsheet_engine::Spreadsheet;
use spreadsheet_engine::value_types::EvaluatedValue::*;

#[test]
fn indirect_with_text() {
    let mut spreadsheet = Spreadsheet::new();
    spreadsheet.input_raw_formula(adr![0, 0], r#"indirect("0,1")"#);
    spreadsheet.input_raw_formula(adr![0, 1], "1");

    assert_value!(spreadsheet, adr![0, 0], Number(1.0));

    spreadsheet.input_raw_formula(adr![0, 1], "2");

    assert_value!(spreadsheet, adr![0, 0], Number(2.0));
}


#[test]
fn indirect_with_reference_to_text() {
    let mut spreadsheet = Spreadsheet::new();
    spreadsheet.input_raw_formula(adr![0, 0], "indirect((0,1))");
    spreadsheet.input_raw_formula(adr![0, 1], r#""0,2""#);
    spreadsheet.input_raw_formula(adr![0, 2], "2");
    spreadsheet.input_raw_formula(adr![0, 3], "3");
    assert_value!(spreadsheet, adr![0, 0], Number(2.0));

    spreadsheet.input_raw_formula(adr![0, 1], r#""0,3""#);
    assert_value!(spreadsheet, adr![0, 0], Number(3.0));

    spreadsheet.input_raw_formula(adr![0, 3], "4");
    assert_value!(spreadsheet, adr![0, 0], Number(4.0));
}

#[test]
fn indirect_to_absent_cell() {
    let mut spreadsheet = Spreadsheet::new();
    spreadsheet.input_raw_formula(adr![0, 0], r#"indirect("0,1")"#);

    assert_value!(spreadsheet, adr![0, 0], Number(0.0));
}

#[test]
fn indirect_to_unevaluated_cell() {
    let mut spreadsheet = Spreadsheet::new();
    spreadsheet.input_raw_formula(adr![0, 0], "(0,1)");
    spreadsheet.input_raw_formula(adr![0, 1], "(0,0)");
    spreadsheet.input_raw_formula(adr![0, 2], r#"indirect("0,0")"#);

    assert_unevaluated!(spreadsheet, adr![0, 2]);
}

#[test]
fn indirect_invalid_address() {
    let mut spreadsheet = Spreadsheet::new();
    spreadsheet.input_raw_formula(adr![0, 0], r#"indirect("notanaddress")"#);

    assert_value!(spreadsheet, adr![0, 0], Error("Indirect reference is not a valid cell address".to_string()));
}

#[test]
fn indirect_non_text_reference() {
    let mut spreadsheet = Spreadsheet::new();
    spreadsheet.input_raw_formula(adr![0, 0], "indirect(1)");

    assert_value!(spreadsheet, adr![0, 0], Error("Indirect reference is not text".to_string()));
}
