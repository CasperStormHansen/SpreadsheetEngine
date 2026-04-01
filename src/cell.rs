use std::collections::HashSet;
use crate::cell_address::CellAddress;
use crate::cell_region::CellRegion;
use crate::cell_value::CellValue;
use crate::formula::Formula;

#[derive(PartialEq, Debug)]
/// Represents a spreadsheet cell that has a non-empty user-entered formula or literal.
///
/// A cell shown as empty in the UI is not represented in memory as a [`Cell`].
pub(crate) struct Cell {
    /// The raw text entered by the user.
    ///
    /// This can be a formula, a literal number, or plain text.
    /// This module is responsible for setting its value.
    pub(crate) raw_formula: String,
    /// The parsed version of [`Self::raw_formula`].
    ///
    /// This module is responsible for triggering a reparse when [`Self::raw_formula`] changes,
    /// delegating the actual parsing to the [`Formula`] module.
    pub(crate) parsed_formula: Formula,
    /// The regions of the spreadsheet that influence this cell's value.
    ///
    /// For example, if the formula is `sum(A1:A10)`, then this set contains the
    /// corresponding [`CellRegion`] value for `A1:A10`.This does not imply that the referenced
    /// cells actually exist in memory.
    ///
    /// This module is responsible for triggering an update when [`Self::parsed_formula`] changes,
    /// delegating the actual determination of the regions to the [`CellRegion`] module.
    pub(crate) child_regions: HashSet<CellRegion>, // todo: should this be a hasmap from cellregions to hashsets of celladdresses?
    /// The set of cells that directly affect this cell's value. Equivalently: the set of cells that
    /// belong to at least one of the [`Self::child_regions`].
    ///
    /// Unlike [`Self::child_regions`], this depends on which cells actually exist in memory.
    /// Therefore, the [`crate::spreadsheet::Spreadsheet`] module is responsible for keeping it updated.
    pub(crate) children: HashSet<CellAddress>,
    /// The computed value of the cell.
    ///
    /// It is the responsibility of the spreadsheet module to trigger an update when necessary,
    /// but the actual calculation is the responsibility of the [`Formula`] module. However, it can
    /// be set to the special value [`CellValue::Unevaluated`] by both the
    /// [`crate::spreadsheet::Spreadsheet`] module directly and by this module.
    pub(crate) value: CellValue,
    /// The set of cells whose values depend on this cell.
    ///
    /// The [`crate::spreadsheet::Spreadsheet`] module is responsible for keeping it updated.
    pub(crate) parents: HashSet<CellAddress>,
}

impl Cell {
    pub(crate) fn new(raw_formula: &str) -> Cell {
        let parsed_formula = Formula::parse(raw_formula);
        let child_regions = CellRegion::get_cell_regions(&parsed_formula);
        Cell {
            raw_formula: raw_formula.to_string(),
            parsed_formula,
            child_regions,
            children: HashSet::new(),
            value: CellValue::Unevaluated,
            parents: HashSet::new(),
        }
    }

    pub(crate) fn update_formula(&mut self, raw_formula: &str) {
        let parsed_formula = Formula::parse(&raw_formula);
        let child_regions = CellRegion::get_cell_regions(&parsed_formula);
        self.raw_formula = raw_formula.to_string();
        self.parsed_formula = parsed_formula;
        self.child_regions = child_regions;
        self.value = CellValue::Unevaluated;
    }
}
