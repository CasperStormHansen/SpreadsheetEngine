use std::collections::HashSet;
use crate::formula::{parse, Formula, WellFormedFormula};
use crate::{CellValue, Spreadsheet};
use crate::cell_rectangle::CellRectangle;
use crate::formula::utils::common_parsing::SplitOnceOutsideParentheses;
use crate::formula::utils::normalized_raw_formula::NormalizedRawFormula;

pub(crate) struct Conditional {
    condition: Box<dyn Formula>,
    true_formula: Box<dyn Formula>,
    false_formula: Box<dyn Formula>,
}

impl Formula for Conditional {
    fn evaluate(&self, spreadsheet: &Spreadsheet) -> CellValue {
        let conditional_value = self.condition.evaluate(spreadsheet);
        match conditional_value {
            CellValue::Boolean(true) => self.true_formula.evaluate(spreadsheet),
            CellValue::Boolean(false) => self.false_formula.evaluate(spreadsheet),
            _ => CellValue::Error("Condition is not boolean".to_string()),
        }
    }

    fn get_child_rectangles(&self) -> HashSet<CellRectangle> {
        // todo: this should be dynamic and depend on the value of the conditional
        let mut union = self.condition.get_child_rectangles();
        union.extend(self.true_formula.get_child_rectangles());
        union.extend(self.false_formula.get_child_rectangles());
        union
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
