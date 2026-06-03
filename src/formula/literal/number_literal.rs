use crate::EvaluatedValue::Number;
use crate::EvaluatedValue;
use crate::formula::literal::{IntoEvaluatedValue, Literal};

pub(crate) type NumberLiteral = Literal<f64>;

impl IntoEvaluatedValue for f64 {
    fn into_value(self) -> EvaluatedValue {
        Number(self)
    }
}