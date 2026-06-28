use rand::Rng;
use crate::formula::{parse, DataRequestAndEvaluationMethod, EvaluationData, Formula, ResultOrRequest, WellFormedFormula};
use crate::formula::ResultOrRequest::Result;
use crate::formula::utils::common_parsing::SplitOnceOutsideParentheses;
use crate::formula::utils::normalized_raw_formula::NormalizedRawFormula;
use crate::value_types::EvaluatedValue::SingleCellValue;
use crate::value_types::SingleCellValue::{Error, Number};

pub(crate) struct RandBetween {
    lower_bound: Box<dyn Formula>,
    upper_bound: Box<dyn Formula>,
}

impl Formula for RandBetween {
    fn initial_data_request_and_evaluation_method(&self) -> DataRequestAndEvaluationMethod<'_> {
        DataRequestAndEvaluationMethod::with_formulas(
            vec![self.lower_bound.as_ref(), self.upper_bound.as_ref()],
            |data| self.evaluate(data),
        )
    }

    fn is_volatile(&self) -> bool {
        true
    }
}

impl RandBetween {
    fn evaluate(&self, evaluation_data: EvaluationData) -> ResultOrRequest<'_> {
        let lower_bound = match &evaluation_data.formula_to_value_map[&self.lower_bound.as_address()] {
            SingleCellValue(Number(n)) if n.fract() == 0.0 => *n as i64,
            _ => return Result(SingleCellValue(Error("The lower bound is not an integer".to_string()))),
        };

        let upper_bound = match &evaluation_data.formula_to_value_map[&self.upper_bound.as_address()] {
            SingleCellValue(Number(n)) if n.fract() == 0.0 => *n as i64,
            _ => return Result(SingleCellValue(Error("The upper bound is not an integer".to_string()))),
        };

        if lower_bound > upper_bound {
            return Result(SingleCellValue(Error("The lower bound is greater than the upper bound".to_string())));
        }

        Result(SingleCellValue(Number(rand::rng().random_range(lower_bound..=upper_bound) as f64)))
    }
}

impl WellFormedFormula for RandBetween {
    fn try_parse(raw_formula: &NormalizedRawFormula) -> Option<Self> {
        let inner = raw_formula.strip_prefix("randbetween(")?.strip_suffix(')')?;
        let (left, right) = inner.split_once_outside_parentheses(',')?;
        let lower_bound = parse(left);
        let upper_bound = parse(right);
        Some(Self{ lower_bound, upper_bound })
    }
}
