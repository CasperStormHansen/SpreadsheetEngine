use crate::EvaluatedValue;
use crate::EvaluatedValue::Boolean;
use crate::formula::literal::{IntoEvaluatedValue, Literal};

pub(crate) type BooleanLiteral = Literal<bool>;

impl IntoEvaluatedValue for bool {
    fn into_value(self) -> EvaluatedValue {
        Boolean(self)
    }
}