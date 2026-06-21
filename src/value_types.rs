use std::collections::HashSet;
use ndarray::Array2;
use crate::cell_lookup_structure::cell_address::CellAddress;
use crate::cell_lookup_structure::cell_rectangle::CellRectangle;

/// Represents the value of a [`Cell`]. A [`Cell`] has an [`SingleCellValue`] if evaluation is not
/// prevented by circularity.
pub type Value = Option<SingleCellValue>;

#[derive(PartialEq, Debug, Clone)]
pub(crate) enum EvaluatedValue {
    SingleCellValue(SingleCellValue),
    ArrayValue(ArrayValue),
}

#[derive(PartialEq, Debug, Clone)]
pub enum SingleCellValue {
    Number(f64), // todo: To be improved. This choice means that 0.1 + 0.2 != 0.3 due to floating point precision issues.
    Boolean(bool),
    Text(String),
    Error(String),
}

#[derive(PartialEq, Debug, Clone)]
pub(crate) struct ArrayValue {
    pub(crate) values: Array2<SingleCellValue>,
}

impl ArrayValue {
    pub(crate) fn spill_rectangle(&self, anchor: CellAddress) -> Option<CellRectangle> {
        let right_col = u32::try_from(self.values.ncols()).ok()
            .and_then(|cols| cols.checked_sub(1))
            .and_then(|offset| anchor.column.checked_add(offset))?;
        let bottom_row = u32::try_from(self.values.nrows()).ok()
            .and_then(|rows| rows.checked_sub(1))
            .and_then(|offset| anchor.row.checked_add(offset))?;
        Some(CellRectangle::new(anchor, CellAddress::new(right_col, bottom_row)).unwrap())
    }
}

/// Represents the result of attempting to evaluate a formula. If the evaluation is successful, the
/// result is a [`CompletedEvaluationResult`], which contains the resulting value plus the child
/// rectangles on which it depends (so that reevaluation can be triggered on any change in them).
/// Evaluation may be attempted without the engine yet knowing all relevant child rectangles. Example:
/// if the formula is of the form `IF(a,b,c)`, the initial child rectangles are those of `a`, and
/// evaluation will be attempted when all cells in those have an [`EvaluatedValue`]. Then it may be
/// turn out that `a` evaluates to `true`, so the child rectangles of `b` are also relevant. If some
/// cell in one of those has no value (yet), the evaluation attempt is aborted and a
/// [`RequestForMoreChildRectangles`] is returned instead.
pub(crate) type EvaluationResult = Result<CompletedEvaluationResult, RequestForMoreChildRectangles>;

#[derive(PartialEq, Debug, Clone)]
pub struct CompletedEvaluationResult(
    pub(crate) EvaluatedValue,
    pub(crate) UsedChildRectangles
);

pub(crate) type RequestForMoreChildRectangles = HashSet<CellRectangle>;

pub(crate) type UsedChildRectangles = HashSet<CellRectangle>;
