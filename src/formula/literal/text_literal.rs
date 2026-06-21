use crate::formula::literal::{IntoEvaluatedValue, Literal};
use crate::formula::utils::normalized_raw_formula::NormalizedRawFormula;
use crate::formula::WellFormedFormula;
use crate::value_types::SingleCellValue;
use crate::value_types::SingleCellValue::Text;

pub(crate) type TextLiteral = Literal<String>;

impl IntoEvaluatedValue for String {
    fn into_value(self) -> SingleCellValue {
        Text(self)
    }
}

impl WellFormedFormula for Literal<String> {
    fn try_parse(raw_formula: &NormalizedRawFormula) -> Option<Self> {
        let inner = raw_formula.strip_prefix("\"")?.strip_suffix("\"")?;
        Some(Self{value: inner.to_string()})
    }
}
