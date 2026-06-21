use std::cmp::PartialEq;
use std::collections::HashSet;
use crate::cell_lookup_structure::cell_map::CellMap;
use crate::cell::{Cell, IndependentData};
use crate::cell::Kind::{Dependent, Independent};
use crate::cell_lookup_structure::cell_address::CellAddress;
use crate::cell_lookup_structure::cell_parent_map::ParentLookupTree;
use crate::cell_lookup_structure::cell_rectangle::CellRectangle;
use crate::cell_lookup_structure::spill_ownership_map::{ClaimStatus, SpillOwnershipMap};
use crate::value_types::{CompletedEvaluationResult, Value};
use crate::{formula, value_types};
use crate::value_types::EvaluatedValue::{ArrayValue, SingleCellValue};
use crate::value_types::SingleCellValue::Error;

pub struct Spreadsheet {
    pub(crate) cells: CellMap,
    parent_lookup_tree: ParentLookupTree,
    spill_ownership_map: SpillOwnershipMap,
    volatile_cells: HashSet<CellAddress>,
}

impl Spreadsheet {
    pub fn new() -> Spreadsheet {
        Self {
            cells: CellMap::new(),
            parent_lookup_tree: ParentLookupTree::new(),
            spill_ownership_map: SpillOwnershipMap::new(),
            volatile_cells: HashSet::new(),
        }
    }

    pub fn cell_value(&self, cell_address: CellAddress) -> Option<Value> {
        self.cells.get(&cell_address).map(|cell| cell.value.clone())
    }

    pub fn input_raw_formula(&mut self, cell_address: CellAddress, raw_formula: &str) {
        let mut reset_cells = self.update_cell_and_structure_and_reset_values(cell_address, raw_formula);
        let volatile: Vec<CellAddress> = self.volatile_cells.iter().copied().collect(); // todo: move into 'evaluate'?
        for cell in volatile {
            reset_cells.extend(self.reset_value_and_children_for_cell_and_ancestors(cell));
        }
        self.evaluate(reset_cells);
    }

    fn update_cell_and_structure_and_reset_values(&mut self, cell_address: CellAddress, raw_formula: &str) -> HashSet<CellAddress> {
        match self.get_cell_update_type(cell_address, raw_formula) {
            CellUpdateType::CreateIndependentCell => {
                let parsed_formula = formula::parse(raw_formula);
                let is_volatile = parsed_formula.is_volatile();
                let cell = Cell {
                    value: None,
                    parents: HashSet::new(),
                    kind: Independent(IndependentData {
                        raw_formula: raw_formula.to_string(),
                        parsed_formula,
                        child_rectangles: HashSet::new(),
                        children: HashSet::new(),
                    })
                };
                self.cells.insert(cell_address, cell);
                if is_volatile {
                    self.volatile_cells.insert(cell_address);
                }
                self.attach_to_parents(cell_address);
                let reset_cells = self.reset_value_and_children_for_cell_and_ancestors(cell_address); // todo: move out?
                reset_cells
            },
            CellUpdateType::ModifyIndependentCell(independent_data) => {
                independent_data.raw_formula = raw_formula.to_string();
                independent_data.parsed_formula = formula::parse(&raw_formula);
                if independent_data.parsed_formula.is_volatile() {
                    self.volatile_cells.insert(cell_address);
                } else {
                    self.volatile_cells.remove(&cell_address);
                }
                let reset_cells = self.reset_value_and_children_for_cell_and_ancestors(cell_address);
                reset_cells
            }
            CellUpdateType::DeleteIndependentCell => {
                self.volatile_cells.remove(&cell_address);
                let mut reset_cells = self.reset_value_and_children_for_cell_and_ancestors(cell_address);
                self.detach_from_parents(cell_address);
                self.detach_from_children(cell_address);
                self.remove_from_parent_lookup_tree(cell_address);
                self.cells.remove(&cell_address);
                reset_cells.remove(&cell_address);
                reset_cells
            },
            CellUpdateType::KeepAbsent => HashSet::new(),
            CellUpdateType::ReplaceDependentCell => {
                todo!()
            }
        }
    }

    fn get_cell_update_type(&mut self, cell_address: CellAddress, raw_formula: &str) -> CellUpdateType {
        let raw_formula_has_content = !raw_formula.trim().is_empty();
        let cell_already_exists = self.cells.contains(&cell_address);

        match (raw_formula_has_content, cell_already_exists) {
            (true, false) => CellUpdateType::CreateIndependentCell,
            (false, true) => CellUpdateType::DeleteIndependentCell,
            (false, false) => CellUpdateType::KeepAbsent,
            (true, true) => {
                let cell = &mut self.cells[&cell_address];
                if let Independent(ref mut independent_data) = cell.kind {
                    CellUpdateType::ModifyIndependentCell(independent_data)
                } else {
                    CellUpdateType::ReplaceDependentCell
                }
            }
        }
    }

    fn attach_to_parents(&mut self, address: CellAddress) {
        let parent_addresses = self.parent_lookup_tree.get_all_parents(address);
        for parent_address in parent_addresses {
            self.cells[&address].parents.insert(parent_address);
            self.cells[&parent_address].independent_data_mut().children.insert(address);
        }
    }

    fn detach_from_parents(&mut self, address: CellAddress) {
        let parent_addresses: Vec<CellAddress> = self.cells[&address]
            .parents.iter().copied().collect();

        self.cells[&address].parents.clear();
        for parent_address in parent_addresses {
            self.cells[&parent_address].independent_data_mut().children.remove(&address);
        }
    }

    fn attach_to_children(&mut self, address: CellAddress) {
        let child_rectangles: Vec<CellRectangle> = self.cells[&address]
            .independent_data()
            .child_rectangles.iter().cloned().collect();
        let child_addresses: Vec<CellAddress> = child_rectangles.iter()
            .flat_map(|r| self.cells.get_all_in_rectangle(r).map(|(addr, _)| addr))
            .collect();

        for child_address in child_addresses {
            self.cells[&address].independent_data_mut().children.insert(child_address);
            self.cells[&child_address].parents.insert(address);
        }
    }

    fn detach_from_children(&mut self, address: CellAddress) {
        let child_addresses: Vec<CellAddress> = self.cells[&address].independent_data()
            .children.iter().copied().collect();

        self.cells[&address].independent_data_mut().children.clear();
        for child_address in child_addresses {
            self.cells[&child_address].parents.remove(&address);
        }
    }
    
    fn add_to_parent_lookup_tree(&mut self, address: CellAddress) {
        let child_rectangles = &self.cells[&address].independent_data().child_rectangles;
        for child_rectangle in child_rectangles {
            self.parent_lookup_tree.insert(address, child_rectangle);
        }
    }

    fn remove_from_parent_lookup_tree(&mut self, address: CellAddress) {
        let child_rectangles = &self.cells[&address].independent_data().child_rectangles;
        for child_rectangle in child_rectangles {
            self.parent_lookup_tree.delete(&address, child_rectangle);
        }
    }

    fn reset_value_and_children_for_cell_and_ancestors(&mut self, self_address: CellAddress) -> HashSet<CellAddress> {
        let mut queue = vec!(self_address);
        let mut reset_cells = HashSet::new();

        while let Some(address) = queue.pop() {
            self.cells[&address].value = None;
            if matches!(self.cells[&address].kind, Independent(_)) {
                self.update_child_data(address, ChildDataUpdateType::Reset);
            }
            reset_cells.insert(address);
            queue.extend(
                self.cells[&address].parents.iter()
                    .filter(|parent| !reset_cells.contains(parent))
                    .copied(),
            );

            if let Some(owned_rectangle) = self.spill_ownership_map.get_owned_rectangle(&address).cloned() {
                let dependent_addresses: Vec<CellAddress> = self.cells.get_all_in_rectangle(&owned_rectangle)
                    .map(|(addr, _)| addr)
                    .filter(|addr| *addr != address && !reset_cells.contains(addr))
                    .collect();
                self.spill_ownership_map.remove(&address);
                for dep_addr in dependent_addresses {
                    let dep_reset_cells = self.delete_dependent_cell(dep_addr);
                    reset_cells.extend(dep_reset_cells);
                }
            }
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
        self.cells[address].independent_data().children.iter()
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
                self.cells[&address].independent_data_mut().child_rectangles = self.cells[&address].independent_data().parsed_formula.get_initial_child_rectangles(),
            ChildDataUpdateType::Set(child_rectangles) =>
                self.cells[&address].independent_data_mut().child_rectangles = child_rectangles,
            ChildDataUpdateType::Extend(extra_child_rectangles) =>
                self.cells[&address].independent_data_mut().child_rectangles.extend(extra_child_rectangles)
        }
        self.add_to_parent_lookup_tree(address);
        self.attach_to_children(address);
    }

    fn evaluate(&mut self, reset_cells: HashSet<CellAddress>) {
        let mut evaluation_queue = self.filter_for_no_unevaluated_children(&reset_cells);

        // todo: add map from cell address to number of reevals and use

        while let Some(address) = evaluation_queue.pop() {
            let cell = &self.cells[&address];
            if let Independent(independent_data) = &cell.kind
            {
                let evaluation_result = independent_data.parsed_formula.evaluate(self);
                match evaluation_result {
                    Ok(CompletedEvaluationResult(value, child_rectangles)) =>
                    {
                        match value {
                            SingleCellValue(single_cell_value) => {
                                self.cells[&address].value = Some(single_cell_value);
                                self.update_child_data(address, ChildDataUpdateType::Set(child_rectangles));
                                evaluation_queue.extend(self.get_parents_with_no_unevaluated_children(address));
                            },
                            ArrayValue(array_value) => {
                                let right_col = u32::try_from(array_value.values.ncols()).ok()
                                    .and_then(|cols| address.column.checked_add(cols - 1));
                                let bottom_row = u32::try_from(array_value.values.nrows()).ok()
                                    .and_then(|rows| address.row.checked_add(rows - 1));
                                if right_col.is_none() || bottom_row.is_none() {
                                    self.cells[&address].value = Some(Error("The required cells would extend beyond the edges of the spreadsheet".to_string()));
                                } else {
                                let lower_right = CellAddress::new(right_col.unwrap(), bottom_row.unwrap());
                                let area = CellRectangle::new(address, lower_right).unwrap();
                                if self.contains_exactly_this_cell(&area, &address) {
                                    self.spill_ownership_map.insert(address, area, ClaimStatus::Active);
                                    self.cells[&address].value = Some(array_value.values[[0, 0]].clone());
                                    self.update_child_data(address, ChildDataUpdateType::Set(child_rectangles));
                                    evaluation_queue.extend(self.get_parents_with_no_unevaluated_children(address));
                                    for row in 0..array_value.values.nrows() {
                                        for col in 0..array_value.values.ncols() {
                                            if row == 0 && col == 0 { continue; }
                                            let cell_address = CellAddress::new(address.column + col as u32, address.row + row as u32);
                                            self.create_dependent_cell(cell_address, array_value.values[[row, col]].clone());
                                            evaluation_queue.extend(self.get_parents_with_no_unevaluated_children(cell_address));
                                        }
                                    }
                                } else {
                                    self.spill_ownership_map.insert(address, area, ClaimStatus::Blocked);
                                    self.cells[&address].value = Some(Error("The required cells are not free".to_string()));
                                }
                                } // end bounds check else
                            },
                        }
                    }
                    Err(extra_child_rectangles) => {
                        self.update_child_data(address, ChildDataUpdateType::Extend(extra_child_rectangles));
                    }
                }
            }
            else { panic!("Dependent cells should not be evaluated"); }
        }

        // todo: check if area has been freed for dynamic arrays
    }

    fn create_dependent_cell(&mut self, cell_address: CellAddress, value: value_types::SingleCellValue) -> HashSet<CellAddress> {
        self.cells.insert(cell_address, Cell {
            kind: Dependent,
            value: None,
            parents: HashSet::new(),
        });
        self.attach_to_parents(cell_address);
        let reset_cells = self.reset_value_and_children_for_cell_and_ancestors(cell_address);
        self.cells[&cell_address].value = Some(value);
        reset_cells
    }

    fn delete_dependent_cell(&mut self, cell_address: CellAddress) -> HashSet<CellAddress> {
        let mut reset_cells = self.reset_value_and_children_for_cell_and_ancestors(cell_address);
        self.detach_from_parents(cell_address);
        self.cells.remove(&cell_address);
        reset_cells.remove(&cell_address);
        reset_cells
    }

    /// Returns true if the 'cell_rectangle' contains 'cell_address' and nothing else.
    /// Returns false if the 'cell_rectangle' contains 'cell_address' and other addresses.
    /// Panics if the 'cell_rectangle' does not contain 'cell_address'.
    fn contains_exactly_this_cell(&self, cell_rectangle: &CellRectangle, cell_address: &CellAddress) -> bool { // todo: Can now be simplified. That it contains itself is now guranteed.
        let mut found = false;
        let mut only = true;

        for (addr, _) in self.cells.get_all_in_rectangle(cell_rectangle) {
            if &addr == cell_address {
                found = true;
            } else {
                only = false;
            }
        }

        assert!(found, "A dynamic array should always assign a value to its own cell");
        only
    }
}

enum CellUpdateType<'a> {
    CreateIndependentCell,
    ModifyIndependentCell(&'a mut IndependentData),
    DeleteIndependentCell,
    KeepAbsent,
    ReplaceDependentCell,
}

#[derive(PartialEq)]
enum ChildDataUpdateType {
    Reset,
    Set(HashSet<CellRectangle>),
    Extend(HashSet<CellRectangle>),
}