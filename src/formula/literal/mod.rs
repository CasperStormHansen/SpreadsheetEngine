use std::str::FromStr;
use crate::formula::{DataRequestAndEvaluationMethod, Formula, WellFormedFormula};
use crate::formula::ResultOrRequest::Result;
use crate::formula::utils::normalized_raw_formula::NormalizedRawFormula;
use crate::value_types::{EvaluatedValue, SingleCellValue};

pub(crate) struct Literal<T> {
    value: T,
}

impl<T: IntoEvaluatedValue + Clone> Formula for Literal<T> {
    fn initial_data_request_and_evaluation_method(&self) -> DataRequestAndEvaluationMethod<'_> {
        DataRequestAndEvaluationMethod {
            cell_rectangles: vec!(),
            formulas: vec!(),
            evaluation_method: Box::new(|_| Result(EvaluatedValue::SingleCellValue(self.value.clone().into_value()))),
        }
    }

    fn is_volatile(&self) -> bool {
        false
    }
}

pub(super) trait DefaultParsing: FromStr + Clone {}

impl<T: IntoEvaluatedValue + DefaultParsing> WellFormedFormula for Literal<T> {
    fn try_parse(raw_formula: &NormalizedRawFormula) -> Option<Self> {
        T::from_str(raw_formula).ok().map(|value| Self { value })
    }
}

trait IntoEvaluatedValue {
    fn into_value(self) -> SingleCellValue;
}

pub(crate) mod number_literal;
pub(crate) mod boolean_literal;
pub(crate) mod text_literal;
