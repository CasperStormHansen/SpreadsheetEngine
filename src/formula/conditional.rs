use crate::cell_lookup_structure::cell_rectangle::CellRectangle;
use crate::formula::utils::common_parsing::SplitOnceOutsideParentheses;
use crate::formula::utils::normalized_raw_formula::NormalizedRawFormula;
use crate::formula::{parse, EvaluationResult, Formula, WellFormedFormula};
use crate::value_types::EvaluatedValue::SingleCellValue;
use crate::value_types::CompletedEvaluationResult;
use crate::Spreadsheet;
use std::collections::HashSet;
use crate::value_types::SingleCellValue::{Boolean, Error};

pub(crate) struct Conditional {
    condition: Box<dyn Formula>,
    true_formula: Box<dyn Formula>,
    false_formula: Box<dyn Formula>,
}

impl Formula for Conditional {
    fn evaluate(&self, spreadsheet: &Spreadsheet) -> EvaluationResult {
        match self.condition.evaluate(spreadsheet) {
            Ok(CompletedEvaluationResult(SingleCellValue(Boolean(true)), child_rectangles)) =>
                evaluate_branch(self.true_formula.as_ref(), spreadsheet, child_rectangles),
            Ok(CompletedEvaluationResult(SingleCellValue(Boolean(false)), child_rectangles)) =>
                evaluate_branch(self.false_formula.as_ref(), spreadsheet, child_rectangles),
            Ok(CompletedEvaluationResult(_, child_rectangles)) =>
                Ok(CompletedEvaluationResult(SingleCellValue(Error("Condition is not boolean".to_string())), child_rectangles)),
            Err(request_for_more_child_rectangles) =>
                Err(request_for_more_child_rectangles),
        }
    }

    fn get_initial_child_rectangles(&self) -> HashSet<CellRectangle> {
        self.condition.get_initial_child_rectangles()
    }
    
    fn is_volatile(&self) -> bool {
        self.condition.is_volatile() || self.true_formula.is_volatile() || self.false_formula.is_volatile()
    }
}

fn evaluate_branch(
    branch_formula: &dyn Formula,
    spreadsheet: &Spreadsheet,
    mut child_rectangles: HashSet<CellRectangle>
) -> EvaluationResult {
    match branch_formula.evaluate(spreadsheet) {
        Ok(CompletedEvaluationResult(value, branch_child_rectangles)) => {
            child_rectangles.extend(branch_child_rectangles);
            Ok(CompletedEvaluationResult(value, child_rectangles))
        }
        Err(mut request_for_more_child_rectangles) => {
            request_for_more_child_rectangles.extend(child_rectangles);
            Err(request_for_more_child_rectangles)
        }
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
