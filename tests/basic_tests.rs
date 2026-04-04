mod common_test;

use spreadsheet_engine::Spreadsheet;
use spreadsheet_engine::CellAddress;
use spreadsheet_engine::CellValue::{Number, Error, Unevaluated};

#[test]
fn single_cell_number_literal() {
    let mut spreadsheet = Spreadsheet::new();
    spreadsheet.input_raw_formula(adr![0, 0], "1");

    assert_value!(spreadsheet, adr![0, 0], Number(1.0));
}

#[test]
fn single_self_referencing_cell() {
    let mut spreadsheet = Spreadsheet::new();
    spreadsheet.input_raw_formula(adr![0, 0], "(0,0)");

    assert_value!(spreadsheet, adr![0, 0], Unevaluated);
}

#[test]
fn single_cell_with_invalid_formula() {
    let mut spreadsheet = Spreadsheet::new();
    spreadsheet.input_raw_formula(adr![0, 0], "InvalidFormula");

    assert_value!(spreadsheet, adr![0, 0], Error("InvalidFormula".to_string()));
}

#[test]
fn one_cell_referencing_another() {
    let mut spreadsheet = Spreadsheet::new();
    spreadsheet.input_raw_formula(adr![0, 0], "(0,1)");

    assert_value!(spreadsheet, adr![0, 0], Number(0.0));
    
    spreadsheet.input_raw_formula(adr![0, 1], "1");

    assert_value!(spreadsheet, adr![0, 0], Number(1.0));
    assert_value!(spreadsheet, adr![0, 1], Number(1.0));
}

#[test]
fn two_cells_referencing_each_other() {
    let mut spreadsheet = Spreadsheet::new();
    spreadsheet.input_raw_formula(adr![0, 0], "(0,1)");

    assert_value!(spreadsheet, adr![0, 0], Number(0.0));

    spreadsheet.input_raw_formula(adr![0, 1], "(0,0)");

    assert_value!(spreadsheet, adr![0, 0], Unevaluated);
    assert_value!(spreadsheet, adr![0, 1], Unevaluated);
}

#[test]
fn one_cell_referencing_self_referencing_cell() {
    let mut spreadsheet = Spreadsheet::new();
    spreadsheet.input_raw_formula(adr![0, 0], "(0,0)");
    spreadsheet.input_raw_formula(adr![0, 1], "(0,0)");

    assert_value!(spreadsheet, adr![0, 0], Unevaluated);
    assert_value!(spreadsheet, adr![0, 1], Unevaluated);
}

#[test]
fn chain_of_three_cells() {
    let mut spreadsheet = Spreadsheet::new();
    spreadsheet.input_raw_formula(adr![0, 0], "(0,1)");
    spreadsheet.input_raw_formula(adr![0, 1], "(0,2)");
    spreadsheet.input_raw_formula(adr![0, 2], "1.1");

    assert_value!(spreadsheet, adr![0, 0], Number(1.1));
    assert_value!(spreadsheet, adr![0, 1], Number(1.1));
    assert_value!(spreadsheet, adr![0, 2], Number(1.1));
}

#[test]
fn modification() {
    let mut spreadsheet = Spreadsheet::new();
    spreadsheet.input_raw_formula(adr![0, 0], "(0,1)");
    spreadsheet.input_raw_formula(adr![0, 1], "(0,2)");
    spreadsheet.input_raw_formula(adr![0, 2], "1.1");
    spreadsheet.input_raw_formula(adr![1, 2], "2.2");
    spreadsheet.input_raw_formula(adr![0, 1], "(1,2)");

    assert_value!(spreadsheet, adr![0, 0], Number(2.2));
    assert_value!(spreadsheet, adr![0, 1], Number(2.2));
    assert_value!(spreadsheet, adr![0, 2], Number(1.1));
    assert_value!(spreadsheet, adr![1, 2], Number(2.2));
}

#[test]
fn deletion() {
    let mut spreadsheet = Spreadsheet::new();
    spreadsheet.input_raw_formula(adr![0, 0], "(0,1)");
    spreadsheet.input_raw_formula(adr![0, 1], "(0,2)");
    spreadsheet.input_raw_formula(adr![0, 2], "1.1");
    spreadsheet.input_raw_formula(adr![0, 1], "");

    assert_value!(spreadsheet, adr![0, 0], Number(0.0));
    assert_empty!(spreadsheet, adr![0, 1]);
    assert_value!(spreadsheet, adr![0, 2], Number(1.1));
}

#[test]
fn keep_absent() {
    let mut spreadsheet = Spreadsheet::new();
    spreadsheet.input_raw_formula(adr![0, 0], "(0,1)");
    spreadsheet.input_raw_formula(adr![0, 1], "");

    assert_value!(spreadsheet, adr![0, 0], Number(0.0));
    assert_empty!(spreadsheet, adr![0, 1]);
}