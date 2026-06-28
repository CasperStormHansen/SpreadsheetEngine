mod common_test;

use spreadsheet_engine::CellAddress;
use spreadsheet_engine::value_types::SingleCellValue::*;
use spreadsheet_engine::Spreadsheet;

// INDIRECT resolves in two Request stages (evaluate sub-formula → fetch resolved cell).
// Using it as IF's condition means both IF's lazy-eval Request cycle and INDIRECT's
// two-stage Request cycle must interleave correctly.
#[test]
fn if_condition_is_indirect() {
    let mut spreadsheet = Spreadsheet::new();

    spreadsheet.input_raw_formula(adr![0, 0], r#"if(indirect("1,0"), 10, 20)"#);
    spreadsheet.input_raw_formula(adr![1, 0], "true");
    assert_value!(spreadsheet, adr![0, 0], Number(10.0));

    spreadsheet.input_raw_formula(adr![1, 0], "false");
    assert_value!(spreadsheet, adr![0, 0], Number(20.0));
}

// IF is a sub-formula of INDIRECT. IF resolves first (selecting the address string),
// then INDIRECT makes its Request for the resolved cell.
#[test]
fn indirect_address_from_if() {
    let mut spreadsheet = Spreadsheet::new();

    spreadsheet.input_raw_formula(adr![0, 0], r#"indirect(if(true, "1,0", "1,1"))"#);
    spreadsheet.input_raw_formula(adr![1, 0], "42");
    spreadsheet.input_raw_formula(adr![1, 1], "99");
    assert_value!(spreadsheet, adr![0, 0], Number(42.0));

    spreadsheet.input_raw_formula(adr![0, 0], r#"indirect(if(false, "1,0", "1,1"))"#);
    assert_value!(spreadsheet, adr![0, 0], Number(99.0));
}

// IF's lazy evaluation must gate not just simple cell references but also complex
// sub-formulas like INDIRECT. The else-branch INDIRECT must not run (and must not
// attempt to fetch the self-referencing cell that would block evaluation).
#[test]
fn if_lazy_eval_with_indirect_branches() {
    let mut spreadsheet = Spreadsheet::new();

    spreadsheet.input_raw_formula(adr![0, 0], r#"if((0,1), indirect("1,0"), indirect("2,0"))"#);
    spreadsheet.input_raw_formula(adr![0, 1], "true");
    spreadsheet.input_raw_formula(adr![1, 0], "42");
    spreadsheet.input_raw_formula(adr![2, 0], "(2,0)");
    assert_value!(spreadsheet, adr![0, 0], Number(42.0));
    assert_unevaluated!(spreadsheet, adr![2, 0]);
}
