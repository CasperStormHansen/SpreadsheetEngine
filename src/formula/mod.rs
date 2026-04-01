use std::str::FromStr;
use crate::cell_address::CellAddress;
use crate::cell_value::CellValue;
use crate::spreadsheet::Spreadsheet;
// trait FormulaKind {
//     fn parse(input: &str) -> Option<Self>
//     where
//         Self: Sized;
//
//     fn evaluate(&self) -> CellValue;
//     fn pretty(&self) -> String;
// }

#[derive(PartialEq, Debug)]
pub(crate) enum Formula {
    NumberLiteral(f64),
    CellReference(CellAddress),
    //BinarySum(Box<Formula>, Box<Formula>),
    //RectangleSum(CellRegion::Rectangle),
    ParsingError(String),
}

impl Formula {
    pub(crate) fn parse(raw_formula: &str) -> Formula {
        let trimmed_raw_formula = raw_formula.trim();

        if let Ok(number) = f64::from_str(trimmed_raw_formula) {
            return Formula::NumberLiteral(number);
        }

        if let Some(address) = parse_cell_address(trimmed_raw_formula) {
            return Formula::CellReference(address);
        }

        Formula::ParsingError(raw_formula.to_string())
    }

    pub(crate) fn evaluate(&self, spreadsheet: &Spreadsheet) -> CellValue {
        match self {
            Formula::NumberLiteral(n) => CellValue::Number(*n),
            Formula::CellReference(address)
                => match spreadsheet.cells.get(&address) {
                    Some(cell) => cell.value.clone(),
                    None => CellValue::Number(0.0),
                },
            Formula::ParsingError(_) => CellValue::Error("Parsing error".to_string()), // todo: needs improvement
        }
    }
}

fn parse_cell_address(string: &str) -> Option<CellAddress> {
    let string_without_parenthesis = string.strip_prefix('(')?.strip_suffix(')')?;
    let mut parts = string_without_parenthesis.split(',');

    let column = parts.next()?.trim().parse::<u32>().ok()?;
    let row = parts.next()?.trim().parse::<u32>().ok()?;

    if parts.next().is_some() {
        return None;
    }

    Some(CellAddress::new(column, row))
}