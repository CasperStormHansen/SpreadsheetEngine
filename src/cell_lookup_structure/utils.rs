use std::ops::Range;

use crate::cell_lookup_structure::cell_rectangle::CellRectangle;

pub(super) fn col_range(rectangle: &CellRectangle) -> Range<u64> {
    u64::from(rectangle.upper_left.column)..u64::from(rectangle.lower_right.column) + 1
}

pub(super) fn row_range(rectangle: &CellRectangle) -> Range<u64> {
    u64::from(rectangle.upper_left.row)..u64::from(rectangle.lower_right.row) + 1
}
