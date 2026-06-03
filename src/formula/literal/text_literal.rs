use crate::EvaluatedValue;
use crate::EvaluatedValue::Text;
use crate::formula::literal::{IntoEvaluatedValue, Literal};
use crate::formula::utils::normalized_raw_formula::NormalizedRawFormula;
use crate::formula::WellFormedFormula;

pub(crate) type TextLiteral = Literal<String>;

impl IntoEvaluatedValue for String {
    fn into_value(self) -> EvaluatedValue {
        Text(self)
    }
}

impl WellFormedFormula for Literal<String> {
    fn try_parse(raw_formula: &NormalizedRawFormula) -> Option<Self> {
        let inner = raw_formula.strip_prefix("\"")?.strip_suffix("\"")?;
        Some(Self{value: inner.to_string()})
    }
}
