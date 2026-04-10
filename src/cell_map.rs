use std::collections::BTreeMap;
use std::ops::Bound::Included;
use std::ops::{Index, IndexMut};
use crate::cell::Cell;
use crate::cell_address::CellAddress;
use crate::cell_rectangle::CellRectangle;

pub(crate) struct CellMap {
    by_column: BTreeMap<u32, BTreeMap<u32, Cell>>,
}

impl CellMap {
    pub fn new() -> Self {
        Self {
            by_column: BTreeMap::new(),
        }
    }

    pub fn contains(&self, address: &CellAddress) -> bool {
        self.by_column
            .get(&address.column)
            .and_then(|rows| rows.get(&address.row))
            .is_some()
    }

    pub fn get(&self, address: &CellAddress) -> Option<&Cell> {
        self.by_column
            .get(&address.column)
            .and_then(|rows| rows.get(&address.row))
    }

    pub fn get_mut(&mut self, address: &CellAddress) -> Option<&mut Cell> {
        self.by_column
            .get_mut(&address.column)
            .and_then(|rows| rows.get_mut(&address.row))
    }

    pub fn insert(&mut self, address: CellAddress, cell: Cell) -> Option<Cell> {
        self.by_column
            .entry(address.column)
            .or_default()
            .insert(address.row, cell)
    }

    pub fn remove(&mut self, address: &CellAddress) -> Option<Cell> {
        let removed = self
            .by_column
            .get_mut(&address.column)
            .and_then(|rows| rows.remove(&address.row));

        if self
            .by_column
            .get(&address.column)
            .is_some_and(|rows| rows.is_empty())
        {
            self.by_column.remove(&address.column);
        }

        removed
    }

    pub fn get_all_in_rectangle( // TODO: Consider return type
                                 &self, cell_rectangle: &CellRectangle,
    ) -> impl Iterator<Item = (CellAddress, &Cell)> {
        self.by_column
            .range((Included(cell_rectangle.upper_left.column), Included(cell_rectangle.lower_right.column)))
            .flat_map(move |(&column, rows)| {
                rows.range((Included(cell_rectangle.upper_left.row), Included(cell_rectangle.lower_right.row)))
                    .map(move |(&row, cell)| (CellAddress { column, row }, cell))
            })
    }

    // TODO: To be deleted
    pub fn iter(&self) -> impl Iterator<Item = (CellAddress, &Cell)> {
        self.by_column.iter().flat_map(|(&column, rows)| {
            rows.iter().map(move |(&row, cell)| (CellAddress { column, row }, cell))
        })
    }
}

impl Index<&CellAddress> for CellMap {
    type Output = Cell;

    fn index(&self, address: &CellAddress) -> &Self::Output {
        self.get(address)
            .unwrap_or_else(|| panic!("cell at address {:?} does not exist", address))
    }
}

impl IndexMut<&CellAddress> for CellMap {
    fn index_mut(&mut self, address: &CellAddress) -> &mut Self::Output {
        self.get_mut(address)
            .unwrap_or_else(|| panic!("cell at address {:?} does not exist", address))
    }
}