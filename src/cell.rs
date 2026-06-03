use std::collections::HashSet;
use crate::cell_address::CellAddress;
use crate::cell_rectangle::CellRectangle;
use crate::value_types::Value;
use crate::formula::Formula;

/// Represents a spreadsheet cell that has a non-empty user-entered formula.
///
/// An empty cell is not represented in memory as a [`Cell`].
pub(crate) struct Cell {

    /// The raw text entered by the user.
    ///
    /// This can be a formula, a literal number, or plain text.
    /// This module is responsible for setting its value.
    pub(crate) raw_formula: String,

    /// The parsed version of [`Self::raw_formula`].
    ///
    /// The [`crate::spreadsheet::Spreadsheet`] module is responsible for triggering a reparse when
    /// [`Self::raw_formula`] changes, delegating the actual parsing to the [`Formula`] module.
    pub(crate) parsed_formula: Box<dyn Formula>,

    /// The regions of the spreadsheet that directly influence this cell's value.
    ///
    /// For example, if the formula is `sum(A1:A10)`, then this set contains the
    /// corresponding [`CellRectangle`] value for `A1:A10`.This does not imply that the referenced
    /// cells actually exist as [`Cell`] objects.
    ///
    /// The [`crate::spreadsheet::Spreadsheet`] is responsible for triggering an update when
    /// [`Self::parsed_formula`] changes, delegating the actual determination of the rectangles to
    /// the [`Formula`] module.
    pub(crate) child_rectangles: HashSet<CellRectangle>,

    /// The set of cells that directly influence this cell's value. Equivalently: the set of cells that
    /// belong to at least one of the [`Self::child_rectangles`].
    ///
    /// Unlike [`Self::child_rectangles`], this depends on which cells actually exist as a [`Cell`] object.
    /// The [`crate::spreadsheet::Spreadsheet`] module is responsible for keeping it updated.
    pub(crate) children: HashSet<CellAddress>,

    /// The computed value of the cell.
    ///
    /// It is the responsibility of the `crate::spreadsheet::Spreadsheet`] module to trigger an
    /// update when necessary, but the actual calculation is the responsibility of the [`Formula`]
    /// module. However, it can be set to `None` by the [`crate::spreadsheet::Spreadsheet`] module
    /// directly.
    pub(crate) value: Value,

    /// The set of cells whose values depend directly on this cell.
    ///
    /// The [`crate::spreadsheet::Spreadsheet`] module is responsible for keeping it updated.
    pub(crate) parents: HashSet<CellAddress>,

    // Todo: Consider using raw pointers for children and parents.
    // Advantages: More readable code, slightly faster.
    // Handling memory safety can be done centrally.
}
