use std::collections::HashSet;
use std::str::FromStr;
use crate::{EvaluatedValue, Spreadsheet};
use crate::cell_lookup_structure::cell_rectangle::CellRectangle;
use crate::formula::{Formula, WellFormedFormula};
use crate::formula::utils::normalized_raw_formula::NormalizedRawFormula;
use crate::value_types::{CompletedEvaluationResult, EvaluationResult};

pub(crate) struct Literal<T> {
    value: T,
}

impl<T: IntoEvaluatedValue + Clone> Formula for Literal<T> {
    fn evaluate(&self, _spreadsheet: &Spreadsheet) -> EvaluationResult {
        Ok(CompletedEvaluationResult(
            self.value.clone().into_value(),
            self.get_initial_child_rectangles(),
        ))
    }

    fn get_initial_child_rectangles(&self) -> HashSet<CellRectangle> {
        HashSet::new()
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
    fn into_value(self) -> EvaluatedValue;
}

pub(crate) mod number_literal;
pub(crate) mod boolean_literal;
pub(crate) mod text_literal;
