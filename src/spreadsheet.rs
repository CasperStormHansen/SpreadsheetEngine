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
use crate::cell_lookup_structure::evaluation_queue::EvaluationQueue;
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
        let reset_cells = self.update_cell_and_structure_and_reset_values(cell_address, raw_formula);
        self.evaluate(reset_cells);
    }

    fn update_cell_and_structure_and_reset_values(&mut self, cell_address: CellAddress, raw_formula: &str) -> HashSet<CellAddress> {
        match self.get_cell_update_type(cell_address, raw_formula) {
            CellUpdateType::CreateIndependentCell  => self.create_independent_cell(cell_address, raw_formula),
            CellUpdateType::ModifyIndependentCell  => self.modify_independent_cell(cell_address, raw_formula),
            CellUpdateType::DeleteIndependentCell  => self.delete_independent_cell(cell_address),
            CellUpdateType::KeepAbsent             => HashSet::new(),
            CellUpdateType::ReplaceDependentCell   => self.replace_dependent_cell(cell_address, raw_formula),
        }
    }

    fn get_cell_update_type(&self, cell_address: CellAddress, raw_formula: &str) -> CellUpdateType {
        let raw_formula_has_content = !raw_formula.trim().is_empty();
        let cell_already_exists = self.cells.contains(&cell_address);

        match (raw_formula_has_content, cell_already_exists) {
            (true,  false) => CellUpdateType::CreateIndependentCell,
            (false, true)  => CellUpdateType::DeleteIndependentCell,
            (false, false) => CellUpdateType::KeepAbsent,
            (true,  true)  => match self.cells[&cell_address].kind {
                Independent(_) => CellUpdateType::ModifyIndependentCell,
                _              => CellUpdateType::ReplaceDependentCell,
            }
        }
    }

    fn create_independent_cell(&mut self, cell_address: CellAddress, raw_formula: &str) -> HashSet<CellAddress> {
        let parsed_formula = formula::parse(raw_formula);
        let is_volatile = parsed_formula.is_volatile();
        self.cells.insert(cell_address, Cell {
            value: None,
            parents: HashSet::new(),
            kind: Independent(IndependentData {
                raw_formula: raw_formula.to_string(),
                parsed_formula,
                child_rectangles: HashSet::new(),
                children: HashSet::new(),
            })
        });
        if is_volatile {
            self.volatile_cells.insert(cell_address);
        }
        self.attach_to_parents(cell_address);
        self.reset_cell_and_ancestors(cell_address)
    }

    fn modify_independent_cell(&mut self, cell_address: CellAddress, raw_formula: &str) -> HashSet<CellAddress> {
        let parsed_formula = formula::parse(raw_formula);
        let is_volatile = parsed_formula.is_volatile();
        self.cells[&cell_address].set_formula(raw_formula, parsed_formula);
        if is_volatile {
            self.volatile_cells.insert(cell_address);
        } else {
            self.volatile_cells.remove(&cell_address);
        }
        self.reset_cell_and_ancestors(cell_address)
    }

    fn delete_independent_cell(&mut self, cell_address: CellAddress) -> HashSet<CellAddress> {
        self.volatile_cells.remove(&cell_address);
        let mut reset_cells = self.reset_cell_and_ancestors(cell_address);
        self.detach_from_parents(cell_address);
        self.detach_from_children(cell_address);
        self.remove_from_parent_lookup_tree(cell_address);
        self.cells.remove(&cell_address);
        reset_cells.remove(&cell_address);
        reset_cells
    }

    fn replace_dependent_cell(&mut self, cell_address: CellAddress, raw_formula: &str) -> HashSet<CellAddress> {
        let owner = self.spill_ownership_map.get_active_owner_for_cell(cell_address).unwrap();
        let mut reset_cells = self.reset_cell_and_ancestors(owner);
        let more_reset_cells = self.create_independent_cell(cell_address, raw_formula);
        reset_cells.extend(more_reset_cells);
        reset_cells
    }

    fn attach_to_parents(&mut self, address: CellAddress) {
        let parent_addresses = self.parent_lookup_tree.get_all_parents(address);
        for parent_address in parent_addresses {
            self.cells[&address].parents.insert(parent_address);
            self.cells[&parent_address].add_child(address);
        }
    }

    fn detach_from_parents(&mut self, address: CellAddress) {
        let parent_addresses: Vec<CellAddress> = self.cells[&address]
            .parents.iter().copied().collect();

        self.cells[&address].parents.clear();
        for parent_address in parent_addresses {
            self.cells[&parent_address].remove_child(address);
        }
    }

    fn attach_to_children(&mut self, address: CellAddress) {
        let child_rectangles: Vec<CellRectangle> = self.cells[&address]
            .child_rectangles().iter().cloned().collect();
        let child_addresses: Vec<CellAddress> = child_rectangles.iter()
            .flat_map(|r| self.cells.get_all_in_rectangle(r).map(|(addr, _)| addr))
            .collect();

        for child_address in child_addresses {
            self.cells[&address].add_child(child_address);
            self.cells[&child_address].parents.insert(address);
        }
    }

    fn detach_from_children(&mut self, address: CellAddress) {
        let child_addresses: Vec<CellAddress> = self.cells[&address].children()
            .iter().copied().collect();

        self.cells[&address].clear_children();
        for child_address in child_addresses {
            self.cells[&child_address].parents.remove(&address);
        }
    }
    
    fn add_to_parent_lookup_tree(&mut self, address: CellAddress) {
        let child_rectangles = self.cells[&address].child_rectangles();
        for child_rectangle in child_rectangles {
            self.parent_lookup_tree.insert(address, child_rectangle);
        }
    }

    fn remove_from_parent_lookup_tree(&mut self, address: CellAddress) {
        let child_rectangles = self.cells[&address].child_rectangles();
        for child_rectangle in child_rectangles {
            self.parent_lookup_tree.delete(&address, child_rectangle);
        }
    }

    // Marks a cell and all cells that (transitively) depend on it as needing re-evaluation by
    // setting their values to None and resetting their child_rectangles to the formula's initial
    // set. If any visited cell is an active spill owner, its dependent cells are deleted and its
    // ownership entry is removed. Returns the set of all affected addresses.
    fn reset_cell_and_ancestors(&mut self, self_address: CellAddress) -> HashSet<CellAddress> {
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

    fn has_no_unevaluated_children(&self, address: &CellAddress) -> bool {
        self.cells[address].children().iter()
            .all(|child_address| self.cells[child_address].value != None)
    }

    fn update_child_data(&mut self, address: CellAddress, child_update_type: ChildDataUpdateType) {
        // todo: optimize, check if child_rectangles have changed
        self.detach_from_children(address);
        self.remove_from_parent_lookup_tree(address);
        match child_update_type {
            ChildDataUpdateType::Reset          => self.cells[&address].reset_child_rectangles_to_initial(),
            ChildDataUpdateType::Set(rects)     => self.cells[&address].set_child_rectangles(rects),
            ChildDataUpdateType::Extend(rects)  => self.cells[&address].extend_child_rectangles(rects),
        }
        self.add_to_parent_lookup_tree(address);
        self.attach_to_children(address);
    }

    fn evaluate(&mut self, mut reset_cells: HashSet<CellAddress>) {
        let volatile: Vec<CellAddress> = self.volatile_cells.iter().copied().collect();
        for cell in volatile {
            reset_cells.extend(self.reset_cell_and_ancestors(cell));
        }

        let mut evaluation_queue: EvaluationQueue =
            self.filter_for_no_unevaluated_children(&reset_cells).into_iter().collect();

        while let Some(address) = self.get_next_evaluation_address(&mut evaluation_queue) {
            let evaluation_result = self.cells[&address].parsed_formula().evaluate(self);
            match evaluation_result {
                Err(extra_child_rectangles) => { // The evaluation could not be completed now because new unevaluated dependencies (children) were discovered. The cell may enter the queue again later.
                    self.update_child_data(address, ChildDataUpdateType::Extend(extra_child_rectangles));
                }
                Ok(CompletedEvaluationResult(value, child_rectangles)) =>
                {
                    match value {
                        SingleCellValue(single_cell_value) => {
                            self.cells[&address].value = Some(single_cell_value);
                            self.update_child_data(address, ChildDataUpdateType::Set(child_rectangles));
                            evaluation_queue.extend(self.get_parents_with_no_unevaluated_children(address));
                        },
                        ArrayValue(array_value) => {
                            match array_value.spill_rectangle(address) {
                                Some(area) => {
                                    if self.spill_area_is_free(&area, &address) {
                                        self.spill_ownership_map.insert(address, area, ClaimStatus::Active);
                                        self.update_child_data(address, ChildDataUpdateType::Set(child_rectangles));
                                        for ((row, col), value) in array_value.values.indexed_iter() {
                                            let cell_address = CellAddress::new(address.column + col as u32, address.row + row as u32);
                                            if cell_address == address {
                                                self.cells[&address].value = Some(value.clone());
                                                evaluation_queue.extend(self.get_parents_with_no_unevaluated_children(address));
                                            } else {
                                                let mut spill_reset_cells = self.create_dependent_cell(cell_address, value.clone());
                                                spill_reset_cells.remove(&cell_address);
                                                evaluation_queue.extend(self.filter_for_no_unevaluated_children(&spill_reset_cells));
                                            }
                                        }
                                    } else {
                                        self.spill_ownership_map.insert(address, area, ClaimStatus::Blocked);
                                        self.cells[&address].value = Some(Error("The required cells are not free".to_string()));
                                    }
                                }
                                None => {
                                    self.cells[&address].value = Some(Error("The required cells would extend beyond the edges of the spreadsheet".to_string()));
                                }
                            }
                        },
                    }
                }
            }
        }
    }

    fn get_next_evaluation_address(&mut self, evaluation_queue: &mut EvaluationQueue) -> Option<CellAddress> {
        if let Some(address) = evaluation_queue.pop() {
            return Some(address);
        }

        let blocked_owners: Vec<CellAddress> = self.spill_ownership_map.blocked_owners().collect();
        for owner in blocked_owners {
            let Some(rectangle) = self.spill_ownership_map.get_owned_rectangle(&owner).cloned() else { continue; };
            if self.spill_area_is_free(&rectangle, &owner) {
                let reset_cells = self.reset_cell_and_ancestors(owner);
                evaluation_queue.extend(self.filter_for_no_unevaluated_children(&reset_cells));
                if let Some(address) = evaluation_queue.pop() {
                    return Some(address);
                }
            }
        }

        None
    }

    fn create_dependent_cell(&mut self, cell_address: CellAddress, value: value_types::SingleCellValue) -> HashSet<CellAddress> {
        self.cells.insert(cell_address, Cell {
            kind: Dependent,
            value: None,
            parents: HashSet::new(),
        });
        self.attach_to_parents(cell_address);
        let reset_cells = self.reset_cell_and_ancestors(cell_address);
        self.cells[&cell_address].value = Some(value);
        reset_cells
    }

    fn delete_dependent_cell(&mut self, cell_address: CellAddress) -> HashSet<CellAddress> {
        let mut reset_cells = self.reset_cell_and_ancestors(cell_address);
        self.detach_from_parents(cell_address);
        self.cells.remove(&cell_address);
        reset_cells.remove(&cell_address);
        reset_cells
    }

    fn spill_area_is_free(&self, cell_rectangle: &CellRectangle, cell_address: &CellAddress) -> bool {
        self.cells.get_all_in_rectangle(cell_rectangle)
            .all(|(addr, _)| &addr == cell_address)
    }
}

enum CellUpdateType {
    CreateIndependentCell,
    ModifyIndependentCell,
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