mod common_test;

use spreadsheet_engine::CellAddress;
use spreadsheet_engine::value_types::EvaluatedValue::*;
use spreadsheet_engine::Spreadsheet;

#[test]
fn single_cell_conditional() {
    let test_cases = [
        ("if(true, 1, 2)", Number(1.0)),
        ("if(false, 1, 2)", Number(2.0)),
        ("if(false, 1, true)", Boolean(true)),
        ("if(   false  , 1,    invalid-formula  )", Error(String::from("invalid-formula"))),
        ("if(   false  , 1 )", Error(String::from("if(   false  , 1 )"))),
        ("if(   false  , 1,    invalid-formula  ))", Error(String::from("invalid-formula)"))),
    ];

    for (raw_formula, expected_value) in test_cases {
        let mut spreadsheet = Spreadsheet::new();
        spreadsheet.input_raw_formula(adr![0, 0], raw_formula);

        assert_value!(spreadsheet, adr![0, 0], expected_value);
    }
}

#[test]
fn conditional_with_references() {
    let mut spreadsheet = Spreadsheet::new();

    spreadsheet.input_raw_formula(adr![0, 0], "if((0,1), (0,2), (0,3))");
    assert_value!(spreadsheet, adr![0, 0], Error("Condition is not boolean".to_string()));

    spreadsheet.input_raw_formula(adr![0, 1], "true");
    assert_value!(spreadsheet, adr![0, 0], Number(0.0));

    spreadsheet.input_raw_formula(adr![0, 2], "1");
    assert_value!(spreadsheet, adr![0, 0], Number(1.0));

    spreadsheet.input_raw_formula(adr![0, 3], "2");
    assert_value!(spreadsheet, adr![0, 0], Number(1.0));

    spreadsheet.input_raw_formula(adr![0, 1], "false");
    assert_value!(spreadsheet, adr![0, 0], Number(2.0));
}

#[test]
fn conditional_with_area_sum() {
    let mut spreadsheet = Spreadsheet::new();

    spreadsheet.input_raw_formula(adr![0, 0], "if((0,1), (0,2), SUM(1,0:1,1))");
    spreadsheet.input_raw_formula(adr![0, 1], "false");
    spreadsheet.input_raw_formula(adr![1, 0], "2");
    spreadsheet.input_raw_formula(adr![1, 1], "3");
    assert_value!(spreadsheet, adr![0, 0], Number(5.0));
}

#[test]
fn iterated_conditional() {
    let mut spreadsheet = Spreadsheet::new();
    spreadsheet.input_raw_formula(adr![0, 0],
        "if( if(true, false, 2), 3, if(false, 4, 5) )");
    assert_value!(spreadsheet, adr![0, 0], Number(5.0));
}

#[test]
fn conditional_lazy_eval_true() {
    let mut spreadsheet = Spreadsheet::new();

    spreadsheet.input_raw_formula(adr![0, 0], "if(true, (0,1), (0,2))");
    spreadsheet.input_raw_formula(adr![0, 1], "2");
    spreadsheet.input_raw_formula(adr![0, 2], "(0,2)");
    assert_value!(spreadsheet, adr![0, 0], Number(2.0));
    assert_unevaluated!(spreadsheet, adr![0, 2]);
}

#[test]
fn conditional_lazy_eval_false() {
    let mut spreadsheet = Spreadsheet::new();

    spreadsheet.input_raw_formula(adr![0, 0], "if(false, (0,1), (0,2))");
    spreadsheet.input_raw_formula(adr![0, 1], "(0,1)");
    spreadsheet.input_raw_formula(adr![0, 2], "2");
    assert_value!(spreadsheet, adr![0, 0], Number(2.0));
    assert_unevaluated!(spreadsheet, adr![0, 1]);
}

#[test]
fn conditional_complex() {
    let mut spreadsheet = Spreadsheet::new();

    spreadsheet.input_raw_formula(adr![0, 0], "if( if(true, false, 2), 3, if((0,1), 4, SUM(1,0:1,1)) )");
    spreadsheet.input_raw_formula(adr![0, 1], "false");
    spreadsheet.input_raw_formula(adr![1, 0], "2");
    spreadsheet.input_raw_formula(adr![1, 1], "3");
    assert_value!(spreadsheet, adr![0, 0], Number(5.0));

    spreadsheet.input_raw_formula(adr![1, 1], "6");
    assert_value!(spreadsheet, adr![0, 0], Number(8.0));
}

#[test]
fn conditional_resetting_children() {
    let mut spreadsheet = Spreadsheet::new();

    spreadsheet.input_raw_formula(adr![0, 0], "if((1,0), (1,1), (1,2))");
    spreadsheet.input_raw_formula(adr![1, 0], "true");
    spreadsheet.input_raw_formula(adr![1, 1], "(1,1)");
    spreadsheet.input_raw_formula(adr![1, 2], "3");
    assert_unevaluated!(spreadsheet, adr![0, 0]);

    spreadsheet.input_raw_formula(adr![1, 0], "false");
    assert_value!(spreadsheet, adr![0, 0], Number(3.0));
}
