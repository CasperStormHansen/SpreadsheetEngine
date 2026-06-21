use crate::formula::literal::{IntoEvaluatedValue, Literal, DefaultParsing};
use crate::value_types::SingleCellValue;
use crate::value_types::SingleCellValue::Boolean;

pub(crate) type BooleanLiteral = Literal<bool>;

impl IntoEvaluatedValue for bool {
    fn into_value(self) -> SingleCellValue {
        Boolean(self)
    }
}

impl DefaultParsing for bool {}