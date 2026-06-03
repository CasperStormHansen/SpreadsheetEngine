use std::cmp::PartialEq;
use std::collections::HashSet;
use crate::cell_lookup_structure::cell_map::CellMap;
use crate::cell::Cell;
use crate::cell_lookup_structure::cell_address::CellAddress;
use crate::cell_lookup_structure::cell_parent_map::ParentLookupTree;
use crate::cell_lookup_structure::cell_rectangle::CellRectangle;
use crate::value_types::{CompletedEvaluationResult, Value};
use crate::formula;

pub struct Spreadsheet {
    pub(crate) cells: CellMap,
    parent_lookup_tree: ParentLookupTree,
}

impl Spreadsheet {
    pub fn new() -> Spreadsheet {
        Self {
            cells: CellMap::new(),
            parent_lookup_tree: ParentLookupTree::new(),
        }
    }

    pub fn cell_value(&self, cell_address: CellAddress) -> Option<Value> {
        self.cells.get(&cell_address).map(|cell| cell.value.clone())
    }

    pub fn input_raw_formula(&mut self, cell_address: CellAddress, raw_formula: &str) {
        match self.get_cell_update_type(cell_address, raw_formula) {
            CellUpdateType::Create => {
                let parsed_formula = formula::parse(raw_formula);
                let cell = Cell {
                    raw_formula: raw_formula.to_string(),
                    parsed_formula,
                    child_rectangles: HashSet::new(),
                    children: HashSet::new(),
                    value: None,
                    parents: HashSet::new(),
                };
                self.cells.insert(cell_address, cell);
                self.attach_to_parents(cell_address);
                let reset_cells = self.reset_value_and_children_for_cell_and_ancestors(cell_address);
                self.evaluate(reset_cells);
            },
            CellUpdateType::Modify => {
                let cell = &mut self.cells[&cell_address];
                cell.raw_formula = raw_formula.to_string();
                cell.parsed_formula = formula::parse(&raw_formula);
                self.add_to_parent_lookup_tree(cell_address);
                self.attach_to_children(cell_address);
                let reset_cells = self.reset_value_and_children_for_cell_and_ancestors(cell_address);
                self.evaluate(reset_cells);
            },
            CellUpdateType::Delete => {
                let mut reset_cells = self.reset_value_and_children_for_cell_and_ancestors(cell_address);
                self.detach_from_parents(cell_address);
                self.detach_from_children(cell_address);
                self.remove_from_parent_lookup_tree(cell_address);
                self.cells.remove(&cell_address);
                reset_cells.remove(&cell_address);
                self.evaluate(reset_cells);
            },
            CellUpdateType::KeepAbsent => return
        }
    }

    fn get_cell_update_type(& self, cell_address: CellAddress, raw_formula: &str) -> CellUpdateType {
        let raw_formula_has_content = !raw_formula.trim().is_empty();
        let cell_already_exists = self.cells.contains(&cell_address);

        match (raw_formula_has_content, cell_already_exists) {
            (true, false) => CellUpdateType::Create,
            (true, true) => CellUpdateType::Modify,
            (false, true) => CellUpdateType::Delete,
            (false, false) => CellUpdateType::KeepAbsent
        }
    }

    fn attach_to_parents(&mut self, address: CellAddress) {
        let parent_addresses = self.parent_lookup_tree.get_all_parents(address);
        for parent_address in parent_addresses {
            self.cells[&address].parents.insert(parent_address);
            self.cells[&parent_address].children.insert(address);
        }
    }

    fn detach_from_parents(&mut self, address: CellAddress) {
        let parent_addresses: Vec<CellAddress> = self.cells[&address]
            .parents.iter().copied().collect();

        self.cells[&address].parents.clear();
        for parent_address in parent_addresses {
            self.cells[&parent_address].children.remove(&address);
        }
    }

    fn attach_to_children(&mut self, address: CellAddress) {
        let child_addresses: Vec<CellAddress> = self.cells[&address]
            .child_rectangles.iter().flat_map(|child_rectangles| {
                self.cells.get_all_in_rectangle(child_rectangles)
                    .map(|(child_address, _)| child_address)
            })
            .collect();

        for child_address in child_addresses {
            self.cells[&address].children.insert(child_address);
            self.cells[&child_address].parents.insert(address);
        }
    }

    fn detach_from_children(&mut self, address: CellAddress) {
        let child_addresses: Vec<CellAddress> = self.cells[&address]
            .children.iter().copied().collect();

        self.cells[&address].children.clear();
        for child_address in child_addresses {
            self.cells[&child_address].parents.remove(&address);
        }
    }
    
    fn add_to_parent_lookup_tree(&mut self, address: CellAddress) {
        let child_rectangles = &self.cells[&address].child_rectangles;
        for child_rectangle in child_rectangles {
            self.parent_lookup_tree.insert(address, child_rectangle);
        }
    }

    fn remove_from_parent_lookup_tree(&mut self, address: CellAddress) {
        let child_rectangles = &self.cells[&address].child_rectangles;
        for child_rectangle in child_rectangles {
            self.parent_lookup_tree.delete(&address, child_rectangle);
        }
    }

    fn reset_value_and_children_for_cell_and_ancestors(&mut self, self_address: CellAddress) -> HashSet<CellAddress> {
        let mut queue = vec![self_address];
        let mut reset_cells = HashSet::new();

        while let Some(address) = queue.pop() {
            self.cells[&address].value = None;
            self.update_child_data(address, ChildDataUpdateType::Reset);
            reset_cells.insert(address);
            queue.extend(
                self.cells[&address].parents.iter()
                    .filter(|parent| !reset_cells.contains(parent))
                    .copied(),
            );
        }

        reset_cells
    }

    fn filter_for_no_unevaluated_children(&self, addresses: &HashSet<CellAddress>) -> Vec<CellAddress> {
        addresses.iter()
            .filter(|address| self.has_no_unevaluated_children(*address))
            .copied()
            .collect()
    }

    fn get_parents_with_no_unevaluated_children(& self, address: CellAddress) -> Vec<CellAddress> {
        self.cells[&address].parents.clone().into_iter()
            .filter(|parent_address| self.has_no_unevaluated_children(parent_address))
            .collect()
    }

    fn has_no_unevaluated_children(& self, address: &CellAddress) -> bool {
        self.cells[address].children.iter()
            .all(|child_address| {
                self.cells[child_address].value != None
            })
    }

    fn update_child_data(&mut self, address: CellAddress, child_update_type: ChildDataUpdateType) {
        // todo: optimize, check if child_rectangles have changed
        self.detach_from_children(address);
        self.remove_from_parent_lookup_tree(address);
        match child_update_type {
            ChildDataUpdateType::Reset =>
                self.cells[&address].child_rectangles = self.cells[&address].parsed_formula.get_initial_child_rectangles(),
            ChildDataUpdateType::Set(child_rectangles) =>
                self.cells[&address].child_rectangles = child_rectangles,
            ChildDataUpdateType::Extend(extra_child_rectangles) =>
                self.cells[&address].child_rectangles.extend(extra_child_rectangles)
        }
        self.add_to_parent_lookup_tree(address);
        self.attach_to_children(address);
    }

    fn evaluate(&mut self, reset_cells: HashSet<CellAddress>) {
        let mut evaluation_queue = self.filter_for_no_unevaluated_children(&reset_cells);

        while let Some(address) = evaluation_queue.pop() {
            let evaluation_result = self.cells[&address].parsed_formula.evaluate(self);
            match evaluation_result {
                Ok(CompletedEvaluationResult(value, child_rectangles)) =>
                {
                    self.cells[&address].value = Some(value);
                    self.update_child_data(address, ChildDataUpdateType::Set(child_rectangles));
                    evaluation_queue.extend(self.get_parents_with_no_unevaluated_children(address));
                }
                Err(extra_child_rectangles) => {
                    self.update_child_data(address, ChildDataUpdateType::Extend(extra_child_rectangles));
                }
            }
        }
    }
}

enum CellUpdateType {
    Create,
    Modify,
    Delete,
    KeepAbsent,
}

#[derive(PartialEq)]
enum ChildDataUpdateType {
    Reset,
    Set(HashSet<CellRectangle>),
    Extend(HashSet<CellRectangle>),
}