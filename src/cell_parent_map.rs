use std::collections::HashSet;
use std::ops::Range;

use interavl::IntervalTree;

use crate::cell_address::CellAddress;
use crate::cell_rectangle::CellRectangle;

/// 2D lookup structure:
/// - the outer tree indexes column intervals
/// - the inner tree indexes row intervals
/// - each row interval stores a set of parent addresses
pub(crate) struct ParentLookupTree {
    by_column: IntervalTree<u32, IntervalTree<u32, HashSet<CellAddress>>>,
}

impl ParentLookupTree {
    pub(crate) fn new() -> Self {
        Self {
            by_column: IntervalTree::default(),
        }
    }

    pub(crate) fn insert(&mut self, address: CellAddress, child_rectangle: &CellRectangle) {
        let col_range = col_range(child_rectangle);
        let row_range = row_range(child_rectangle);

        let row_tree = self
            .by_column
            .entry(col_range)
            .or_insert_with(IntervalTree::default);

        row_tree
            .entry(row_range)
            .or_insert_with(HashSet::new)
            .insert(address);
    }

    pub(crate) fn delete(&mut self, address: &CellAddress, child_rectangle: &CellRectangle) {
        let col_range = col_range(child_rectangle);
        let row_range = row_range(child_rectangle);

        if let Some(row_tree) = self.by_column.get_mut(&col_range) {
            if let Some(parents) = row_tree.get_mut(&row_range) {
                parents.remove(address);

                if parents.is_empty() {
                    row_tree.remove(&row_range);
                }
            }

            if row_tree.iter().next().is_none() {
                self.by_column.remove(&col_range);
            }
        }
    }

    pub(crate) fn get_all_parents(&self, address: CellAddress) -> HashSet<CellAddress> {
        let col_query = address.column..address.column.saturating_add(1); // TODO: BAD!
        let row_query = address.row..address.row.saturating_add(1); // TODO: BAD!

        let mut parents = HashSet::new();

        for (_, row_tree) in self.by_column.iter_overlaps(&col_query) {
            for (_, row_parents) in row_tree.iter_overlaps(&row_query) {
                parents.extend(row_parents.iter().copied());
            }
        }

        parents
    }
}

fn col_range(rectangle: &CellRectangle) -> Range<u32> {
    rectangle.upper_left.column..rectangle.lower_right.column.saturating_add(1) // TODO: BAD!
}

fn row_range(rectangle: &CellRectangle) -> Range<u32> {
    rectangle.upper_left.row..rectangle.lower_right.row.saturating_add(1) // TODO: BAD!
}