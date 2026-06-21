use std::collections::HashSet;
use ndarray::Array2;
use crate::cell_lookup_structure::cell_rectangle::CellRectangle;
use crate::formula::utils::common_parsing::SplitOnceOutsideParentheses;
use crate::formula::utils::normalized_raw_formula::NormalizedRawFormula;
use crate::formula::{parse, Formula, WellFormedFormula};
use crate::value_types::SingleCellValue::{Error, Number};
use crate::value_types::{ArrayValue, CompletedEvaluationResult, EvaluatedValue, EvaluationResult};
use crate::Spreadsheet;

pub(crate) struct Sequence {
    rows: Box<dyn Formula>,
    columns: Box<dyn Formula>,
}

impl Formula for Sequence { // todo: avoid code duplication
    fn evaluate(&self, spreadsheet: &Spreadsheet) -> EvaluationResult {
        let (rows, rows_child_rectangles) = match self.rows.evaluate(spreadsheet) {
            Ok(CompletedEvaluationResult(evaluation_result, rows_child_rectangles)) =>
                if let EvaluatedValue::SingleCellValue(Number(number)) = evaluation_result
                    && let Some(rows) = (number.fract() == 0.0 && number >= 1.0).then(|| number as usize) {
                    (rows, rows_child_rectangles)
                } else {
                    return Ok(CompletedEvaluationResult(EvaluatedValue::SingleCellValue(
                        Error("rows must be a positive integer".to_string())), rows_child_rectangles))
                },
            Err(request_for_more_child_rectangles) =>
                return Err(request_for_more_child_rectangles),
        };

        let (columns, columns_child_rectangles) = match self.columns.evaluate(spreadsheet) {
            Ok(CompletedEvaluationResult(evaluation_result, columns_child_rectangles)) =>
                if let EvaluatedValue::SingleCellValue(Number(number)) = evaluation_result
                    && let Some(columns) = (number.fract() == 0.0 && number >= 1.0).then(|| number as usize) {
                    (columns, columns_child_rectangles)
                } else {
                    let mut child_rectangles = rows_child_rectangles;
                    child_rectangles.extend(columns_child_rectangles);
                    return Ok(CompletedEvaluationResult(EvaluatedValue::SingleCellValue(
                        Error("columns must be a positive integer".to_string())), child_rectangles))
                },
            Err(mut request_for_more_child_rectangles) => {
                request_for_more_child_rectangles.extend(rows_child_rectangles);
                return Err(request_for_more_child_rectangles);
            }
        };

        let mut child_rectangles = rows_child_rectangles;
        child_rectangles.extend(columns_child_rectangles);

        let values = Array2::from_shape_fn((rows, columns), |(row, column)| {
            Number((row * columns + column + 1) as f64)
        });

        Ok(CompletedEvaluationResult(
            EvaluatedValue::ArrayValue(ArrayValue { values }),
            child_rectangles,
        ))
    }

    fn get_initial_child_rectangles(&self) -> HashSet<CellRectangle> {
        let mut child_rectangles = self.rows.get_initial_child_rectangles();
        child_rectangles.extend(self.columns.get_initial_child_rectangles());
        child_rectangles
    }

    fn is_volatile(&self) -> bool {
        false
    }
}

impl WellFormedFormula for Sequence {
    fn try_parse(raw_formula: &NormalizedRawFormula) -> Option<Self> {
        let inner = raw_formula.strip_prefix("sequence(")?.strip_suffix(')')?;
        let (rows_str, columns_str) = inner.split_once_outside_parentheses(',')?;
        Some(Self {
            rows: parse(rows_str),
            columns: parse(columns_str),
        })
    }
}
