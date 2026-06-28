use crate::formula::utils::common_parsing::SplitOnceOutsideParentheses;
use crate::formula::utils::normalized_raw_formula::NormalizedRawFormula;
use crate::formula::{parse, DataRequestAndEvaluationMethod, EvaluationData, Formula, ResultOrRequest, WellFormedFormula};
use crate::value_types::EvaluatedValue::SingleCellValue;
use crate::formula::ResultOrRequest::{Request, Result};
use crate::value_types::SingleCellValue::{Boolean, Error};

pub(crate) struct Conditional {
    condition: Box<dyn Formula>,
    true_formula: Box<dyn Formula>,
    false_formula: Box<dyn Formula>,
}

impl Formula for Conditional {
    fn initial_data_request_and_evaluation_method(&self) -> DataRequestAndEvaluationMethod<'_> {
        DataRequestAndEvaluationMethod::with_formulas(
            vec![self.condition.as_ref()],
            |data| self.evaluate_initial(data),
        )
    }

    fn is_volatile(&self) -> bool {
        self.condition.is_volatile() || self.true_formula.is_volatile() || self.false_formula.is_volatile()
    }
}

impl Conditional {
    fn evaluate_initial(&self, evaluation_data: EvaluationData) -> ResultOrRequest<'_> {
        match evaluation_data.formula_to_value_map[&self.condition.as_address()] {
            SingleCellValue(Boolean(true)) =>
                Request(DataRequestAndEvaluationMethod::with_formulas(
                    vec![self.true_formula.as_ref()],
                    |data| self.evaluate_true(data),
                )),
            SingleCellValue(Boolean(false)) =>
                Request(DataRequestAndEvaluationMethod::with_formulas(
                    vec![self.false_formula.as_ref()],
                    |data| self.evaluate_false(data),
                )),
            _ =>
                Result(SingleCellValue(Error("Condition is not boolean".to_string()))),
        }
    }

    fn evaluate_true(&self, evaluation_data: EvaluationData) -> ResultOrRequest<'_> {
        Result(evaluation_data.formula_to_value_map[&self.true_formula.as_address()].clone())
    }

    fn evaluate_false(&self, evaluation_data: EvaluationData) -> ResultOrRequest<'_> {
        Result(evaluation_data.formula_to_value_map[&self.false_formula.as_address()].clone())
    }
}

impl WellFormedFormula for Conditional {
    fn try_parse(raw_formula: &NormalizedRawFormula) -> Option<Self> {
        let inner = raw_formula.strip_prefix("if(")?.strip_suffix(')')?;
        let (left, remainder) = inner.split_once_outside_parentheses(',')?;
        let (center, right) = remainder.split_once_outside_parentheses(',')?;
        let condition = parse(left);
        let true_formula = parse(center);
        let false_formula = parse(right);
        Some(Self{ condition, true_formula, false_formula })
    }
}
