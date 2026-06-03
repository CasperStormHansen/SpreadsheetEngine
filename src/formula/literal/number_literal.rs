use crate::EvaluatedValue;
use crate::EvaluatedValue::Number;
use crate::formula::literal::{IntoEvaluatedValue, Literal, DefaultParsing};

pub(crate) type NumberLiteral = Literal<f64>;

impl IntoEvaluatedValue for f64 {
    fn into_value(self) -> EvaluatedValue {
        Number(self)
    }
}

impl DefaultParsing for f64 {}