use crate::formula::literal::{IntoEvaluatedValue, Literal, DefaultParsing};
use crate::value_types::SingleCellValue;
use crate::value_types::SingleCellValue::Number;

pub(crate) type NumberLiteral = Literal<f64>;

impl IntoEvaluatedValue for f64 {
    fn into_value(self) -> SingleCellValue {
        Number(self)
    }
}

impl DefaultParsing for f64 {}