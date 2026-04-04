use spreadsheet_engine::CellAddress;
use spreadsheet_engine::CellValue::{Error, Number, Unevaluated};
use spreadsheet_engine::Spreadsheet;

mod common_test;

#[test]
fn area_sum_happy_path() {
    let mut spreadsheet = Spreadsheet::new();
    spreadsheet.input_raw_formula(adr![0, 0], "1");
    spreadsheet.input_raw_formula(adr![0, 1], "2");
    spreadsheet.input_raw_formula(adr![1, 0], "4");
    spreadsheet.input_raw_formula(adr![2, 0], "SUM(0,0:1,1)");

    assert_value!(spreadsheet, adr![2, 0], Number(7.0));

    spreadsheet.input_raw_formula(adr![1, 1], "(3,0)");
    spreadsheet.input_raw_formula(adr![3, 0], "8");

    assert_value!(spreadsheet, adr![2, 0], Number(15.0));

    spreadsheet.input_raw_formula(adr![4, 0], "(2,0)");

    assert_value!(spreadsheet, adr![4, 0], Number(15.0));
}

#[test]
fn area_sum_self_reference() {
    let mut spreadsheet = Spreadsheet::new();
    spreadsheet.input_raw_formula(adr![0, 0], "1");
    spreadsheet.input_raw_formula(adr![0, 1], "2");
    spreadsheet.input_raw_formula(adr![1, 0], "4");
    spreadsheet.input_raw_formula(adr![1, 1], "SUM(0,0:1,1)");

    assert_value!(spreadsheet, adr![1, 1], Unevaluated);
}

#[test]
fn area_sum_area_includes_self_referential_cell() {
    let mut spreadsheet = Spreadsheet::new();
    spreadsheet.input_raw_formula(adr![0, 0], "(0,0)");
    spreadsheet.input_raw_formula(adr![0, 1], "2");
    spreadsheet.input_raw_formula(adr![1, 0], "4");
    spreadsheet.input_raw_formula(adr![2, 0], "SUM(0,0:1,1)");

    assert_value!(spreadsheet, adr![2, 0], Unevaluated);
}

#[test]
fn area_sum_area_includes_error_cell() {
    let mut spreadsheet = Spreadsheet::new();
    spreadsheet.input_raw_formula(adr![0, 0], "InvalidFormula");
    spreadsheet.input_raw_formula(adr![0, 1], "2");
    spreadsheet.input_raw_formula(adr![1, 0], "4");
    spreadsheet.input_raw_formula(adr![2, 0], "SUM(0,0:1,1)");

    assert_value!(spreadsheet, adr![2, 0], Error("Summing over area with error".to_string()));
}
