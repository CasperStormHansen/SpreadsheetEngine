use std::collections::HashSet;
use crate::cell::Cell;
use crate::cell_address::CellAddress;
use crate::cell_region::CellRegion::SingleCell;
use crate::cell_value::CellValue::{Number, Error, Unevaluated};
use crate::formula::Formula::{CellReference, NumberLiteral, ParsingError};
use super::*;

macro_rules! set {
    () => {
        HashSet::new()
    };
    ($($item:expr),+ $(,)?) => {
        HashSet::from([$($item),+])
    };
}

macro_rules! adr {
    ($column:expr, $row:expr) => {
        CellAddress::new($column, $row)
    };
}

#[test]
fn single_cell_number_literal() {
    let mut spreadsheet = Spreadsheet::new();
    spreadsheet.input_raw_formula(adr![0, 0], "1");
    
    assert_eq!(spreadsheet.cells.len(), 1);
    assert_eq!(spreadsheet.cells[& adr![0, 0]], Cell {
        raw_formula: "1".to_string(),
        parsed_formula: NumberLiteral(1.0),
        child_regions: set![],
        children: set![],
        value: Number(1.0),
        parents: set![],
    });
}

#[test]
fn single_self_referencing_cell() {
    let mut spreadsheet = Spreadsheet::new();
    spreadsheet.input_raw_formula(adr![0, 0], "(0,0)");

    assert_eq!(spreadsheet.cells.len(), 1);
    assert_eq!(spreadsheet.cells[& adr![0, 0]], Cell {
        raw_formula: "(0,0)".to_string(),
        parsed_formula: CellReference(adr![0, 0]),
        child_regions: set![SingleCell(adr![0, 0])],
        children: set![adr![0, 0]],
        value: Unevaluated,
        parents: set![adr![0, 0]],
    });
}

#[test]
fn single_cell_with_invalid_formula() {
    let mut spreadsheet = Spreadsheet::new();
    spreadsheet.input_raw_formula(adr![0, 0], "invalid formula");

    assert_eq!(spreadsheet.cells.len(), 1);
    assert_eq!(spreadsheet.cells[& adr![0, 0]], Cell {
        raw_formula: "invalid formula".to_string(),
        parsed_formula: ParsingError("invalid formula".to_string()),
        child_regions: set![],
        children: set![],
        value: Error("Parsing error".to_string()),
        parents: set![],
    });
}

#[test]
fn one_cell_referencing_another() {
    let mut spreadsheet = Spreadsheet::new();
    spreadsheet.input_raw_formula(adr![0, 0], "(0,1)");

    assert_eq!(spreadsheet.cells.len(), 1);
    assert_eq!(spreadsheet.cells[& adr![0, 0]], Cell {
        raw_formula: "(0,1)".to_string(),
        parsed_formula: CellReference(adr![0, 1]),
        child_regions: set![SingleCell(adr![0, 1])],
        children: set![],
        value: Number(0.0),
        parents: set![],
    });

    spreadsheet.input_raw_formula(adr![0, 1], "1");

    assert_eq!(spreadsheet.cells.len(), 2);
    assert_eq!(spreadsheet.cells[& adr![0, 0]], Cell {
        raw_formula: "(0,1)".to_string(),
        parsed_formula: CellReference(adr![0, 1]),
        child_regions: set![SingleCell(adr![0, 1])],
        children: set![adr![0, 1]],
        value: Number(1.0),
        parents: set![],
    });
    assert_eq!(spreadsheet.cells[& adr![0, 1]], Cell {
        raw_formula: "1".to_string(),
        parsed_formula: NumberLiteral(1.0),
        child_regions: set![],
        children: set![],
        value: Number(1.0),
        parents: set![adr![0, 0]],
    });
}

#[test]
fn two_cells_referencing_each_other() {
    let mut spreadsheet = Spreadsheet::new();
    spreadsheet.input_raw_formula(adr![0, 0], "(0,1)");

    assert_eq!(spreadsheet.cells.len(), 1);
    assert_eq!(spreadsheet.cells[& adr![0, 0]], Cell {
        raw_formula: "(0,1)".to_string(),
        parsed_formula: CellReference(adr![0, 1]),
        child_regions: set![SingleCell(adr![0, 1])],
        children: set![],
        value: Number(0.0),
        parents: set![],
    });

    spreadsheet.input_raw_formula(adr![0, 1], "(0,0)");

    assert_eq!(spreadsheet.cells.len(), 2);
    assert_eq!(spreadsheet.cells[& adr![0, 0]], Cell {
        raw_formula: "(0,1)".to_string(),
        parsed_formula: CellReference(adr![0, 1]),
        child_regions: set![SingleCell(adr![0, 1])],
        children: set![adr![0, 1]],
        value: Unevaluated,
        parents: set![adr![0, 1]],
    });
    assert_eq!(spreadsheet.cells[& adr![0, 1]], Cell {
        raw_formula: "(0,0)".to_string(),
        parsed_formula: CellReference(adr![0, 0]),
        child_regions: set![SingleCell(adr![0, 0])],
        children: set![adr![0, 0]],
        value: Unevaluated,
        parents: set![adr![0, 0]],
    });
}

#[test]
fn one_cell_referencing_self_referencing_cell() {
    let mut spreadsheet = Spreadsheet::new();
    spreadsheet.input_raw_formula(adr![0, 0], "(0,0)");
    spreadsheet.input_raw_formula(adr![0, 1], "(0,0)");

    assert_eq!(spreadsheet.cells.len(), 2);
    assert_eq!(spreadsheet.cells[& adr![0, 0]], Cell {
        raw_formula: "(0,0)".to_string(),
        parsed_formula: CellReference(adr![0, 0]),
        child_regions: set![SingleCell(adr![0, 0])],
        children: set![adr![0, 0]],
        value: Unevaluated,
        parents: set![adr![0, 0], adr![0, 1]],
    });
    assert_eq!(spreadsheet.cells[& adr![0, 1]], Cell {
        raw_formula: "(0,0)".to_string(),
        parsed_formula: CellReference(adr![0, 0]),
        child_regions: set![SingleCell(adr![0, 0])],
        children: set![adr![0, 0]],
        value: Unevaluated,
        parents: set![],
    });
}

#[test]
fn chain_of_three_cells() {
    let mut spreadsheet = Spreadsheet::new();
    spreadsheet.input_raw_formula(adr![0, 0], "(0,1)");
    spreadsheet.input_raw_formula(adr![0, 1], "(0,2)");
    spreadsheet.input_raw_formula(adr![0, 2], "1.1");

    assert_eq!(spreadsheet.cells.len(), 3);
    assert_eq!(spreadsheet.cells[& adr![0, 0]], Cell {
        raw_formula: "(0,1)".to_string(),
        parsed_formula: CellReference(adr![0, 1]),
        child_regions: set![SingleCell(adr![0, 1])],
        children: set![adr![0, 1]],
        value: Number(1.1),
        parents: set![],
    });
    assert_eq!(spreadsheet.cells[& adr![0, 1]], Cell {
        raw_formula: "(0,2)".to_string(),
        parsed_formula: CellReference(adr![0, 2]),
        child_regions: set![SingleCell(adr![0, 2])],
        children: set![adr![0, 2]],
        value: Number(1.1),
        parents: set![adr![0, 0]],
    });
    assert_eq!(spreadsheet.cells[& adr![0, 2]], Cell {
        raw_formula: "1.1".to_string(),
        parsed_formula: NumberLiteral(1.1),
        child_regions: set![],
        children: set![],
        value: Number(1.1),
        parents: set![adr![0, 1]],
    });
}