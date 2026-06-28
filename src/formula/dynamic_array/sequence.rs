use ndarray::Array2;
use crate::formula::utils::common_parsing::SplitOnceOutsideParentheses;
use crate::formula::utils::normalized_raw_formula::NormalizedRawFormula;
use crate::formula::{parse, DataRequestAndEvaluationMethod, EvaluationData, Formula, ResultOrRequest, WellFormedFormula};
use crate::formula::ResultOrRequest::Result;
use crate::value_types::SingleCellValue::{Error, Number};
use crate::value_types::{ArrayValue, EvaluatedValue};

pub(crate) struct Sequence {
    rows: Box<dyn Formula>,
    columns: Box<dyn Formula>,
}

impl Formula for Sequence {
    fn initial_data_request_and_evaluation_method(&self) -> DataRequestAndEvaluationMethod<'_> {
        DataRequestAndEvaluationMethod {
            cell_rectangles: vec!(),
            formulas: vec!(self.rows.as_ref(), self.columns.as_ref()),
            evaluation_method: Box::new(|data| self.evaluate(data)),
        }
    }

    fn is_volatile(&self) -> bool {
        false
    }
}

impl Sequence {
    fn evaluate(&self, evaluation_data: EvaluationData) -> ResultOrRequest<'_> {
        let rows = match &evaluation_data.formula_to_value_map[&self.rows.as_address()] {
            EvaluatedValue::SingleCellValue(Number(n)) if n.fract() == 0.0 && *n >= 1.0 => *n as usize,
            _ => return Result(EvaluatedValue::SingleCellValue(Error("rows must be a positive integer".to_string()))),
        };

        let columns = match &evaluation_data.formula_to_value_map[&self.columns.as_address()] {
            EvaluatedValue::SingleCellValue(Number(n)) if n.fract() == 0.0 && *n >= 1.0 => *n as usize,
            _ => return Result(EvaluatedValue::SingleCellValue(Error("columns must be a positive integer".to_string()))),
        };

        let values = Array2::from_shape_fn((rows, columns), |(row, column)| {
            Number((row * columns + column + 1) as f64)
        });

        Result(EvaluatedValue::ArrayValue(ArrayValue { values }))
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
