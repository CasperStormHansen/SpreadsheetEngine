mod common_test;

use spreadsheet_engine::CellAddress;
use spreadsheet_engine::value_types::EvaluatedValue::*;
use spreadsheet_engine::Spreadsheet;

macro_rules! assert_integer_in_range {
    ($spreadsheet:expr, $address:expr, $low:expr, $high:expr $(,)?) => {{
        match $spreadsheet.cell_value($address) {
            Some(Some(Number(n))) => {
                assert_eq!(n.fract(), 0.0, "Expected integer, got {n}");
                assert!(n >= $low as f64 && n <= $high as f64,
                    "Expected number in [{}, {}], got {n}", $low, $high);
            }
            other => panic!("Expected Number in [{}, {}], got {other:?}", $low, $high),
        }
    }};
}

#[test]
fn simple_randbetween() {
    let mut spreadsheet = Spreadsheet::new();
    spreadsheet.input_raw_formula(adr![0, 0], "randbetween(1, 10)");

    assert_integer_in_range!(spreadsheet, adr![0, 0], 1, 10);
}

#[test]
fn randbetween_single_value() {
    let mut spreadsheet = Spreadsheet::new();
    spreadsheet.input_raw_formula(adr![0, 0], "randbetween(5, 5)");

    assert_value!(spreadsheet, adr![0, 0], Number(5.0));
}

#[test]
fn randbetween_negative_range() {
    let mut spreadsheet = Spreadsheet::new();
    spreadsheet.input_raw_formula(adr![0, 0], "randbetween(-10, -1)");

    assert_integer_in_range!(spreadsheet, adr![0, 0], -10, -1);
}

#[test]
fn randbetween_lower_greater_than_upper() {
    let mut spreadsheet = Spreadsheet::new();
    spreadsheet.input_raw_formula(adr![0, 0], "randbetween(10, 1)");

    assert_value!(spreadsheet, adr![0, 0], Error("The lower bound is greater than the upper bound".to_string()));
}

#[test]
fn randbetween_non_integer_lower() {
    let mut spreadsheet = Spreadsheet::new();
    spreadsheet.input_raw_formula(adr![0, 0], "randbetween(1.5, 10)");

    assert_value!(spreadsheet, adr![0, 0], Error("The lower bound is not an integer".to_string()));
}

#[test]
fn randbetween_non_integer_upper() {
    let mut spreadsheet = Spreadsheet::new();
    spreadsheet.input_raw_formula(adr![0, 0], "randbetween(1, 10.5)");

    assert_value!(spreadsheet, adr![0, 0], Error("The upper bound is not an integer".to_string()));
}

#[test]
fn randbetween_non_number_lower() {
    let mut spreadsheet = Spreadsheet::new();
    spreadsheet.input_raw_formula(adr![0, 0], "randbetween(true, 10)");

    assert_value!(spreadsheet, adr![0, 0], Error("The lower bound is not an integer".to_string()));
}

#[test]
fn randbetween_non_number_upper() {
    let mut spreadsheet = Spreadsheet::new();
    spreadsheet.input_raw_formula(adr![0, 0], "randbetween(1, true)");

    assert_value!(spreadsheet, adr![0, 0], Error("The upper bound is not an integer".to_string()));
}

#[test]
fn randbetween_with_cell_references() {
    let mut spreadsheet = Spreadsheet::new();
    spreadsheet.input_raw_formula(adr![0, 0], "randbetween((0,1), (0,2))");
    spreadsheet.input_raw_formula(adr![0, 1], "3");
    spreadsheet.input_raw_formula(adr![0, 2], "7");

    assert_integer_in_range!(spreadsheet, adr![0, 0], 3, 7);
}

#[test]
fn randbetween_updates_when_bounds_change() {
    let mut spreadsheet = Spreadsheet::new();
    spreadsheet.input_raw_formula(adr![0, 0], "randbetween((0,1), (0,2))");
    spreadsheet.input_raw_formula(adr![0, 1], "1");
    spreadsheet.input_raw_formula(adr![0, 2], "1");

    assert_value!(spreadsheet, adr![0, 0], Number(1.0));

    spreadsheet.input_raw_formula(adr![0, 1], "5");
    spreadsheet.input_raw_formula(adr![0, 2], "5");

    assert_value!(spreadsheet, adr![0, 0], Number(5.0));
}

#[test]
fn randbetween_inside_conditional() {
    let mut spreadsheet = Spreadsheet::new();
    spreadsheet.input_raw_formula(adr![0, 0], "if(true, randbetween(1, 10), 0)");

    assert_integer_in_range!(spreadsheet, adr![0, 0], 1, 10);
}

#[test]
fn randbetween_with_conditional_bound() {
    let mut spreadsheet = Spreadsheet::new();
    spreadsheet.input_raw_formula(adr![0, 0], "randbetween(if(true, 10, -1000000), 10)");

    assert_value!(spreadsheet, adr![0, 0], Number(10.0));
}

#[test]
fn randbetween_with_area_sum_bound() {
    let mut spreadsheet = Spreadsheet::new();
    spreadsheet.input_raw_formula(adr![0, 0], "randbetween(1, SUM(0,1:0,2))");
    spreadsheet.input_raw_formula(adr![0, 1], "4");
    spreadsheet.input_raw_formula(adr![0, 2], "-3");

    assert_value!(spreadsheet, adr![0, 0], Number(1.0));
}
