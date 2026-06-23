use std::collections::HashSet;
use crate::cell_lookup_structure::cell_address::CellAddress;
use crate::cell_lookup_structure::cell_rectangle::CellRectangle;
use crate::value_types::Value;
use crate::formula::Formula;

/// Represents a spreadsheet cell that has a non-empty user-entered formula.
///
/// An empty cell is not represented in memory as a [`Cell`].
pub(crate) struct Cell {
    /// Indicates whether the cell has a formula of its own or is spill-over from a dynamic array.
    kind: Kind,

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
    parents: HashSet<CellAddress>,

    // Todo: Consider using raw pointers for children and parents.
    // Advantages: More readable code, slightly faster.
    // Handling memory safety can be done centrally.
}

enum Kind {
    /// A cell that is not a spill-over from a dynamic array.
    Independent(IndependentData),
    /// A cell that is a spill-over from a dynamic array.
    Dependent,
}

struct IndependentData {
    /// The raw text entered by the user.
    ///
    /// This can be a formula, a literal number, or plain text.
    /// This module is responsible for setting its value.
    raw_formula: String,

    /// The parsed version of [`Self::raw_formula`].
    ///
    /// The [`crate::spreadsheet::Spreadsheet`] module is responsible for triggering a reparse when
    /// [`Self::raw_formula`] changes, delegating the actual parsing to the [`Formula`] module.
    parsed_formula: Box<dyn Formula>,

    /// The regions of the spreadsheet that directly influence this cell's value.
    ///
    /// For example, if the formula is `sum(A1:A10)`, then this set contains the
    /// corresponding [`CellRectangle`] value for `A1:A10`.This does not imply that the referenced
    /// cells actually exist as [`Cell`] objects.
    ///
    /// The [`crate::spreadsheet::Spreadsheet`] is responsible for triggering an update when
    /// [`Self::parsed_formula`] changes, delegating the actual determination of the rectangles to
    /// the [`Formula`] module.
    child_rectangles: HashSet<CellRectangle>,

    /// The set of cells that directly influence this cell's value. Equivalently: the set of cells that
    /// belong to at least one of the [`Self::child_rectangles`].
    ///
    /// Unlike [`Self::child_rectangles`], this depends on which cells actually exist as a [`Cell`] object.
    /// The [`crate::spreadsheet::Spreadsheet`] module is responsible for keeping it updated.
    children: HashSet<CellAddress>,
}

impl Cell {
    pub(crate) fn new_independent(raw_formula: &str, parsed_formula: Box<dyn Formula>) -> Cell {
        Cell {
            kind: Kind::Independent(IndependentData {
                raw_formula: raw_formula.to_string(),
                parsed_formula,
                child_rectangles: HashSet::new(),
                children: HashSet::new(),
            }),
            value: None,
            parents: HashSet::new(),
        }
    }

    pub(crate) fn new_dependent() -> Cell {
        Cell {
            kind: Kind::Dependent,
            value: None,
            parents: HashSet::new(),
        }
    }

    pub(crate) fn is_independent(&self) -> bool {
        matches!(self.kind, Kind::Independent(_))
    }

    pub(crate) fn parents(&self) -> &HashSet<CellAddress> {
        &self.parents
    }

    pub(crate) fn add_parent(&mut self, address: CellAddress) {
        self.parents.insert(address);
    }

    pub(crate) fn remove_parent(&mut self, address: CellAddress) {
        self.parents.remove(&address);
    }

    pub(crate) fn clear_parents(&mut self) {
        self.parents.clear();
    }

    pub(crate) fn child_rectangles(&self) -> &HashSet<CellRectangle> {
        &self.independent_data().child_rectangles
    }

    pub(crate) fn children(&self) -> &HashSet<CellAddress> {
        &self.independent_data().children
    }

    pub(crate) fn parsed_formula(&self) -> &dyn Formula {
        &*self.independent_data().parsed_formula
    }

    pub(crate) fn set_formula(&mut self, raw_formula: &str, parsed_formula: Box<dyn Formula>) {
        let data = self.independent_data_mut();
        data.raw_formula = raw_formula.to_string();
        data.parsed_formula = parsed_formula;
    }

    pub(crate) fn add_child(&mut self, address: CellAddress) {
        self.independent_data_mut().children.insert(address);
    }

    pub(crate) fn remove_child(&mut self, address: CellAddress) {
        self.independent_data_mut().children.remove(&address);
    }

    pub(crate) fn clear_children(&mut self) {
        self.independent_data_mut().children.clear();
    }

    pub(crate) fn set_child_rectangles(&mut self, child_rectangles: HashSet<CellRectangle>) {
        self.independent_data_mut().child_rectangles = child_rectangles;
    }

    pub(crate) fn extend_child_rectangles(&mut self, extra: HashSet<CellRectangle>) {
        self.independent_data_mut().child_rectangles.extend(extra);
    }

    pub(crate) fn reset_child_rectangles_to_initial(&mut self) {
        let initial = self.independent_data().parsed_formula.get_initial_child_rectangles();
        self.independent_data_mut().child_rectangles = initial;
    }

    fn independent_data(&self) -> &IndependentData {
        if let Kind::Independent(ref data) = self.kind {
            data
        } else {
            panic!("expected independent cell")
        }
    }

    fn independent_data_mut(&mut self) -> &mut IndependentData {
        if let Kind::Independent(ref mut data) = self.kind {
            data
        } else {
            panic!("expected independent cell")
        }
    }
}