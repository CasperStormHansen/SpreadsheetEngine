use std::collections::HashMap;
use crate::cell::Cell;
use crate::cell_address::CellAddress;
use crate::cell_value::CellValue::Unevaluated;

pub struct Spreadsheet {
    pub(crate) cells: HashMap<CellAddress, Cell>,
}

impl Spreadsheet {
    pub fn new() -> Spreadsheet {
        Self {
            cells: HashMap::new(),
        }
    }

    pub fn input_raw_formula(&mut self, cell_address: CellAddress, raw_formula: &str) {
        let cell_update_type = self.get_cell_update_type(cell_address, raw_formula);
        let evaluation_queue: Vec<CellAddress>;

        match cell_update_type {
            CellUpdateType::Create => {
                let cell = Cell::new(raw_formula);
                self.cells.insert(cell_address, cell);
                self.attach_to_parents(cell_address);
                self.attach_to_children(cell_address);
                self.clear_ancestor_values(cell_address);
                evaluation_queue = self.get_cell_if_no_unevaluated_children(cell_address);
            },
            CellUpdateType::Modify => {
                // todo: can be optimized by using child_region-delta instead of re-computing all children
                self.detach_from_children(cell_address);
                self.cell_mut(cell_address).update_formula(raw_formula);
                self.attach_to_children(cell_address);
                self.clear_ancestor_values(cell_address);
                evaluation_queue = self.get_cell_if_no_unevaluated_children(cell_address);
            },
            CellUpdateType::Delete => {
                self.clear_ancestor_values(cell_address);
                evaluation_queue = self.get_parents_with_no_unevaluated_children(cell_address);
                self.detach_from_parents(cell_address);
                self.detach_from_children(cell_address);
                self.cells.remove(&cell_address);
            },
            CellUpdateType::KeepAbsent => return
        }

        self.evaluate(evaluation_queue);
    }

    fn get_cell_update_type(& self, cell_address: CellAddress, raw_formula: &str) -> CellUpdateType {
        let raw_formula_has_content = !raw_formula.trim().is_empty();
        let cell_already_exists = self.cells.contains_key(&cell_address);

        match (raw_formula_has_content, cell_already_exists) {
            (true, false) => CellUpdateType::Create,
            (true, true) => CellUpdateType::Modify,
            (false, true) => CellUpdateType::Delete,
            (false, false) => CellUpdateType::KeepAbsent
        }
    }

    // TODO: Consider adding an index for cells to avoid looping over all cells (in 'attach_to_parents'
    // and 'attach_to_children'). Options include: BTreeMap<u32, BTreeMap<u32, Cell>>, R-tree, quadtree,
    // range tree, segment tree.
    fn attach_to_parents(&mut self, address: CellAddress) {
        let parent_addresses: Vec<CellAddress> = self.cells.iter()
            .filter(|(_, potential_parent)| {
                potential_parent.child_regions.iter()
                    .any(|child_region| child_region.contains(&address))
            })
            .map(|(&potential_parent_address, _)| potential_parent_address)
            .collect();

        for parent_address in parent_addresses {
            self.cell_mut(address).parents.insert(parent_address);
            self.cell_mut(parent_address).children.insert(address);
        }
    }

    fn detach_from_parents(&mut self, address: CellAddress) {
        let parent_addresses: Vec<CellAddress> = self.cell_mut(address)
            .parents.iter().copied().collect();

        self.cell_mut(address).parents.clear();
        for parent_address in parent_addresses {
            self.cell_mut(parent_address).children.remove(&address);
        }
    }

    fn attach_to_children(&mut self, address: CellAddress) {
        let child_addresses: Vec<CellAddress> = self.cells[&address]
            .child_regions.iter().flat_map(|child_region| {
                self.cells.iter()
                    .filter(|(potential_child_address, _)| {
                        child_region.contains(&potential_child_address)
                    })
                    .map(|(&potential_child_address, _)| potential_child_address)
            })
            .collect();

        for child_address in child_addresses {
            self.cell_mut(address).children.insert(child_address);
            self.cell_mut(child_address).parents.insert(address);
        }
    }

    fn detach_from_children(&mut self, address: CellAddress) {
        let child_addresses: Vec<CellAddress> = self.cell_mut(address)
            .children.iter().copied().collect();

        self.cell_mut(address).children.clear();
        for child_address in child_addresses {
            self.cell_mut(child_address).parents.remove(&address);
        }
    }

    fn clear_ancestor_values(&mut self, address: CellAddress) {
        let mut queue: Vec<_> = self.cells[&address].parents.iter().cloned().collect();
        while let Some(ancestor_address) = queue.pop() {
            let ancestor = self.cell_mut(ancestor_address);
            if ancestor.value == Unevaluated {
                continue;
            }
            ancestor.value = Unevaluated;
            for ancestor_parent_address in &ancestor.parents {
                queue.push(*ancestor_parent_address);
            }
        }
    }

    fn get_parents_with_no_unevaluated_children(& self, address: CellAddress) -> Vec<CellAddress> {
        self.cells[&address].parents.clone().into_iter()
            .filter(|parent_address| self.has_no_unevaluated_children(*parent_address))
            .collect()
    }

    fn get_cell_if_no_unevaluated_children(& self, address: CellAddress) -> Vec<CellAddress> {
        if self.has_no_unevaluated_children(address) {
            vec![address]
        } else {
            Vec::new()
        }
    }

    fn has_no_unevaluated_children(& self, address: CellAddress) -> bool {
        self.cells[&address].children.iter()
            .all(|child_address| {
                self.cells[child_address].value != Unevaluated
            })
    }

    fn evaluate(&mut self, mut evaluation_queue: Vec<CellAddress>) {
        while let Some(address) = evaluation_queue.pop() {
            let value = self.cells[&address].parsed_formula.evaluate(self);
            self.cell_mut(address).value = value;
            evaluation_queue.extend(self.get_parents_with_no_unevaluated_children(address));
        }
    }

    fn cell_mut(&mut self, address: CellAddress) -> &mut Cell {
        self.cells
            .get_mut(&address)
            .unwrap_or_else(|| panic!("cell at address {:?} does not exist", address))
    }
}

enum CellUpdateType {
    Create,
    Modify,
    Delete,
    KeepAbsent,
}
