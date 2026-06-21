use std::collections::HashSet;
use rand::Rng;
use crate::cell_lookup_structure::cell_rectangle::CellRectangle;
use crate::formula::{parse, Formula, WellFormedFormula};
use crate::formula::utils::common_parsing::SplitOnceOutsideParentheses;
use crate::formula::utils::normalized_raw_formula::NormalizedRawFormula;
use crate::Spreadsheet;
use crate::value_types::{CompletedEvaluationResult, EvaluationResult};
use crate::value_types::EvaluatedValue::SingleCellValue;
use crate::value_types::SingleCellValue::{Error, Number};

pub(crate) struct RandBetween {
    lower_bound: Box<dyn Formula>,
    upper_bound: Box<dyn Formula>,
}

impl Formula for RandBetween {
    fn evaluate(&self, spreadsheet: &Spreadsheet) -> EvaluationResult {
        let (lower_bound, lower_bound_child_rectangles) = match self.lower_bound.evaluate(spreadsheet) {
            Ok(CompletedEvaluationResult(evaluation_result, lower_bound_child_rectangles)) =>
                if let SingleCellValue(Number(number)) = evaluation_result && let Some(int) = (number.fract() == 0.0).then(|| number as i64) {
                    (int, lower_bound_child_rectangles)
                } else {
                    return Ok(CompletedEvaluationResult(SingleCellValue(Error("The lower bound is not an integer".to_string())), lower_bound_child_rectangles))
                },
            Err(request_for_more_child_rectangles) =>
                return Err(request_for_more_child_rectangles),
        };

        let (upper_bound, upper_bound_child_rectangles) = match self.upper_bound.evaluate(spreadsheet) {
            Ok(CompletedEvaluationResult(evaluation_result, upper_bound_child_rectangles)) =>
                if let SingleCellValue(Number(number)) = evaluation_result && let Some(int) = (number.fract() == 0.0).then(|| number as i64) {
                    (int, upper_bound_child_rectangles)
                } else {
                    let mut child_rectangles = lower_bound_child_rectangles.clone();
                    child_rectangles.extend(upper_bound_child_rectangles);
                    return Ok(CompletedEvaluationResult(SingleCellValue(Error("The upper bound is not an integer".to_string())), child_rectangles))
                },
            Err(mut request_for_more_child_rectangles) => {
                request_for_more_child_rectangles.extend(lower_bound_child_rectangles);
                return Err(request_for_more_child_rectangles);
            }
        };

        let mut child_rectangles = lower_bound_child_rectangles;
        child_rectangles.extend(upper_bound_child_rectangles);

        if lower_bound > upper_bound {
            return Ok(CompletedEvaluationResult(SingleCellValue(Error("The lower bound is greater than the upper bound".to_string())), child_rectangles))
        }

        let result = rand::rng().random_range(lower_bound..=upper_bound);
        Ok(CompletedEvaluationResult(SingleCellValue(Number(result as f64)), child_rectangles))
    }

    fn get_initial_child_rectangles(&self) -> HashSet<CellRectangle> {
        let mut initial_child_rectangles = self.lower_bound.get_initial_child_rectangles();
        initial_child_rectangles.extend(self.upper_bound.get_initial_child_rectangles());
        initial_child_rectangles
    }
    
    fn is_volatile(&self) -> bool {
        true
    }
}

impl WellFormedFormula for RandBetween {
    fn try_parse(raw_formula: &NormalizedRawFormula) -> Option<Self>
    {
        let inner = raw_formula.strip_prefix("randbetween(")?.strip_suffix(')')?;
        let (left, right) = inner.split_once_outside_parentheses(',')?;
        let lower_bound = parse(left);
        let upper_bound = parse(right);
        Some(Self{ lower_bound, upper_bound })
    }
}
