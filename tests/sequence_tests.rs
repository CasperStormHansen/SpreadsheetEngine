mod common_test;

use spreadsheet_engine::CellAddress;
use spreadsheet_engine::value_types::SingleCellValue::*;
use spreadsheet_engine::Spreadsheet;

fn extract_positive_integer(spreadsheet: &Spreadsheet, address: CellAddress) -> u32 {
    match spreadsheet.cell_value(address) {
        Some(Some(Number(n))) if n >= 1.0 && n.fract() == 0.0 => n as u32,
        other => panic!("expected positive integer at {:?}, got {:?}", address, other),
    }
}

#[test]
fn sequence_single_cell() {
    let mut spreadsheet = Spreadsheet::new();
    spreadsheet.input_raw_formula(adr![0, 0], "SEQUENCE(1,1)");

    assert_value!(spreadsheet, adr![0, 0], Number(1.0));
}

#[test]
fn sequence_single_row() {
    let mut spreadsheet = Spreadsheet::new();
    spreadsheet.input_raw_formula(adr![0, 0], "SEQUENCE(1,3)");

    assert_value!(spreadsheet, adr![0, 0], Number(1.0));
    assert_value!(spreadsheet, adr![1, 0], Number(2.0));
    assert_value!(spreadsheet, adr![2, 0], Number(3.0));
    assert_empty!(spreadsheet, adr![3, 0]);
}

#[test]
fn sequence_single_column() {
    let mut spreadsheet = Spreadsheet::new();
    spreadsheet.input_raw_formula(adr![0, 0], "SEQUENCE(3,1)");

    assert_value!(spreadsheet, adr![0, 0], Number(1.0));
    assert_value!(spreadsheet, adr![0, 1], Number(2.0));
    assert_value!(spreadsheet, adr![0, 2], Number(3.0));
    assert_empty!(spreadsheet, adr![0, 3]);
}

#[test]
fn sequence_2d() {
    let mut spreadsheet = Spreadsheet::new();
    spreadsheet.input_raw_formula(adr![0, 0], "SEQUENCE(2,3)");

    assert_value!(spreadsheet, adr![0, 0], Number(1.0));
    assert_value!(spreadsheet, adr![1, 0], Number(2.0));
    assert_value!(spreadsheet, adr![2, 0], Number(3.0));
    assert_value!(spreadsheet, adr![0, 1], Number(4.0));
    assert_value!(spreadsheet, adr![1, 1], Number(5.0));
    assert_value!(spreadsheet, adr![2, 1], Number(6.0));
    assert_empty!(spreadsheet, adr![3, 0]);
    assert_empty!(spreadsheet, adr![0, 2]);
}

#[test]
fn sequence_non_integer_rows() {
    let mut spreadsheet = Spreadsheet::new();
    spreadsheet.input_raw_formula(adr![0, 0], "SEQUENCE(1.5,1)");

    assert_value!(spreadsheet, adr![0, 0], Error("rows must be a positive integer".to_string()));
}

#[test]
fn sequence_non_positive_rows() {
    let mut spreadsheet = Spreadsheet::new();
    spreadsheet.input_raw_formula(adr![0, 0], "SEQUENCE(0,1)");

    assert_value!(spreadsheet, adr![0, 0], Error("rows must be a positive integer".to_string()));
}

#[test]
fn sequence_non_integer_columns() {
    let mut spreadsheet = Spreadsheet::new();
    spreadsheet.input_raw_formula(adr![0, 0], "SEQUENCE(1,2.5)");

    assert_value!(spreadsheet, adr![0, 0], Error("columns must be a positive integer".to_string()));
}

#[test]
fn sequence_spill_blocked() {
    let mut spreadsheet = Spreadsheet::new();
    spreadsheet.input_raw_formula(adr![0, 1], "1");
    spreadsheet.input_raw_formula(adr![0, 0], "SEQUENCE(3,1)");

    assert_value!(spreadsheet, adr![0, 0], Error("The required cells are not free".to_string()));
}

#[test]
fn sequence_updates_with_volatile_arguments() {
    let mut spreadsheet = Spreadsheet::new();
    spreadsheet.input_raw_formula(adr![10, 0], "RANDBETWEEN(1,3)");
    spreadsheet.input_raw_formula(adr![11, 0], "RANDBETWEEN(1,3)");
    spreadsheet.input_raw_formula(adr![0, 0], "SEQUENCE((10,0),(11,0))");

    let check_consistency = |spreadsheet: &Spreadsheet| {
        let rows = extract_positive_integer(spreadsheet, adr![10, 0]);
        let cols = extract_positive_integer(spreadsheet, adr![11, 0]);
        assert_value!(spreadsheet, adr![0, 0], Number(1.0));
        assert_value!(spreadsheet, adr![cols - 1, rows - 1], Number((rows * cols) as f64));
    };

    check_consistency(&spreadsheet);
    spreadsheet.input_raw_formula(adr![20, 0], "1");
    check_consistency(&spreadsheet);
}

#[test]
fn sequence_with_cell_reference_arguments() {
    let mut spreadsheet = Spreadsheet::new();
    spreadsheet.input_raw_formula(adr![5, 0], "3");
    spreadsheet.input_raw_formula(adr![6, 0], "2");
    spreadsheet.input_raw_formula(adr![0, 0], "SEQUENCE((5,0),(6,0))");

    assert_value!(spreadsheet, adr![0, 0], Number(1.0));
    assert_value!(spreadsheet, adr![1, 0], Number(2.0));
    assert_value!(spreadsheet, adr![0, 1], Number(3.0));
    assert_value!(spreadsheet, adr![1, 1], Number(4.0));
    assert_value!(spreadsheet, adr![0, 2], Number(5.0));
    assert_value!(spreadsheet, adr![1, 2], Number(6.0));
}

#[test]
fn sequence_area_gets_freed_up() {
    let mut spreadsheet = Spreadsheet::new();
    spreadsheet.input_raw_formula(adr![0, 0], "5");
    spreadsheet.input_raw_formula(adr![0, 1], "5");
    spreadsheet.input_raw_formula(adr![1, 0], "SEQUENCE((0,0),(0,1))");
    spreadsheet.input_raw_formula(adr![0, 3], "SEQUENCE(2,2)");
    assert_value!(spreadsheet, adr![0, 3], Error("The required cells are not free".to_string()));

    spreadsheet.input_raw_formula(adr![0, 0], "1");
    spreadsheet.input_raw_formula(adr![0, 1], "1");
    assert_value!(spreadsheet, adr![0, 3], Number(1.0));
    assert_value!(spreadsheet, adr![1, 3], Number(2.0));
    assert_value!(spreadsheet, adr![0, 4], Number(3.0));
    assert_value!(spreadsheet, adr![1, 4], Number(4.0));
}