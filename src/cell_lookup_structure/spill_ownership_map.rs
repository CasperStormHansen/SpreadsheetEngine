use std::collections::HashMap;

use interavl::IntervalTree;

use crate::cell_lookup_structure::cell_address::CellAddress;
use crate::cell_lookup_structure::cell_rectangle::CellRectangle;
use crate::cell_lookup_structure::utils::{col_range, row_range};

/// Tracks all spill claims of dynamic arrays, including conflicting ones.
///
/// Invariant: no two *active* claims overlap. Multiple claims may overlap as long as at most one
/// is active. Callers are responsible for enforcing this before activating a claim.
///
/// Supports two queries:
/// - Forward: given an owner cell, what rectangle and status does it have?
/// - Reverse: given a rectangle, which cells have registered a claim inside it?
pub(crate) struct SpillOwnershipMap {
    by_owner: HashMap<CellAddress, (CellRectangle, ClaimStatus)>,
    // Outer tree indexes column intervals; inner tree indexes row intervals.
    // Each leaf holds all claimants for that exact rectangle (multiple possible when conflicting).
    by_rectangle: IntervalTree<u64, IntervalTree<u64, HashMap<CellAddress, ClaimStatus>>>,
}

impl SpillOwnershipMap {
    pub(crate) fn new() -> Self {
        Self {
            by_owner: HashMap::new(),
            by_rectangle: IntervalTree::default(),
        }
    }

    pub(crate) fn insert(&mut self, owner: CellAddress, rectangle: CellRectangle, status: ClaimStatus) {
        let col_range = col_range(&rectangle);
        let row_range = row_range(&rectangle);

        self.by_rectangle
            .entry(col_range)
            .or_insert_with(IntervalTree::default)
            .entry(row_range)
            .or_insert_with(HashMap::new)
            .insert(owner, status);

        self.by_owner.insert(owner, (rectangle, status));
    }

    pub(crate) fn remove(&mut self, owner: &CellAddress) {
        if let Some((rectangle, _)) = self.by_owner.remove(owner) {
            let col_range = col_range(&rectangle);
            let row_range = row_range(&rectangle);

            if let Some(row_tree) = self.by_rectangle.get_mut(&col_range) {
                if let Some(claimants) = row_tree.get_mut(&row_range) {
                    claimants.remove(owner);

                    if claimants.is_empty() {
                        row_tree.remove(&row_range);
                    }
                }

                if row_tree.iter().next().is_none() {
                    self.by_rectangle.remove(&col_range);
                }
            }
        }
    }

    pub(crate) fn set_status(&mut self, owner: &CellAddress, status: ClaimStatus) {
        if let Some((rectangle, current_status)) = self.by_owner.get_mut(owner) {
            *current_status = status;
            let col_range = col_range(rectangle);
            let row_range = row_range(rectangle);

            if let Some(row_tree) = self.by_rectangle.get_mut(&col_range) {
                if let Some(claimants) = row_tree.get_mut(&row_range) {
                    if let Some(s) = claimants.get_mut(owner) {
                        *s = status;
                    }
                }
            }
        }
    }

    pub(crate) fn get_owned_rectangle(&self, owner: &CellAddress) -> Option<&CellRectangle> {
        self.by_owner.get(owner).map(|(rect, _)| rect)
    }

    pub(crate) fn get_claim_status(&self, owner: &CellAddress) -> Option<ClaimStatus> {
        self.by_owner.get(owner).map(|(_, status)| *status)
    }

    pub(crate) fn blocked_owners(&self) -> impl Iterator<Item = CellAddress> + '_ {
        self.by_owner.iter()
            .filter(|(_, (_, status))| *status == ClaimStatus::Blocked)
            .map(|(&addr, _)| addr)
    }

    pub(crate) fn get_claims_in_rectangle(&self, rectangle: &CellRectangle) -> Vec<(CellAddress, ClaimStatus)> {
        let col_query = col_range(rectangle);
        let row_query = row_range(rectangle);

        let mut claims = Vec::new();

        for (_, row_tree) in self.by_rectangle.iter_overlaps(&col_query) {
            for (_, claimants) in row_tree.iter_overlaps(&row_query) {
                claims.extend(claimants.iter().map(|(&addr, &status)| (addr, status)));
            }
        }

        claims
    }
}

/// Whether a formula cell's claim on a rectangle is currently in effect.
///
/// A claim is `Active` when the formula cell owns the
/// cells and is responsible for their values. A claim is `Blocked` when another `Active` claim
/// covers part of the same area, preventing this claim from taking effect. A `Blocked` claim is
/// retained so that if the conflicting `Active` claim is removed, the previously blocked formula
/// can be re-evaluated and potentially become `Active`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum ClaimStatus {
    Active,
    Blocked,
}

#[cfg(test)]
mod tests {
    use super::*;
    use ClaimStatus::*;

    macro_rules! adr {
        ($column:expr, $row:expr) => {
            CellAddress::new($column, $row)
        };
    }

    macro_rules! rect {
        ($c1:expr, $r1:expr, $c2:expr, $r2:expr) => {
            CellRectangle::new(adr![$c1, $r1], adr![$c2, $r2]).unwrap()
        };
    }

    #[test]
    fn insert_and_forward_lookup() {
        let mut map = SpillOwnershipMap::new();
        map.insert(adr![0, 0], rect![0, 1, 0, 3], Active);

        assert_eq!(map.get_owned_rectangle(&adr![0, 0]), Some(&rect![0, 1, 0, 3]));
        assert_eq!(map.get_claim_status(&adr![0, 0]), Some(Active));
        assert_eq!(map.get_owned_rectangle(&adr![1, 0]), None);
        assert_eq!(map.get_claim_status(&adr![1, 0]), None);
    }

    #[test]
    fn remove_clears_both_lookups() {
        let mut map = SpillOwnershipMap::new();
        map.insert(adr![0, 0], rect![0, 1, 0, 3], Active);
        map.remove(&adr![0, 0]);

        assert_eq!(map.get_owned_rectangle(&adr![0, 0]), None);
        assert!(map.get_claims_in_rectangle(&rect![0, 1, 0, 3]).is_empty());
    }

    #[test]
    fn remove_nonexistent_is_a_no_op() {
        let mut map = SpillOwnershipMap::new();
        map.remove(&adr![5, 5]);
    }

    #[test]
    fn set_status_updates_both_lookups() {
        let mut map = SpillOwnershipMap::new();
        map.insert(adr![0, 0], rect![0, 1, 0, 3], Active);
        map.set_status(&adr![0, 0], Blocked);

        assert_eq!(map.get_claim_status(&adr![0, 0]), Some(Blocked));

        let claims = map.get_claims_in_rectangle(&rect![0, 1, 0, 3]);
        assert_eq!(claims, vec![(adr![0, 0], Blocked)]);
    }

    #[test]
    fn two_claimants_same_rectangle() {
        let mut map = SpillOwnershipMap::new();
        map.insert(adr![0, 0], rect![0, 1, 0, 3], Active);
        map.insert(adr![1, 0], rect![0, 1, 0, 3], Blocked);

        let claims = map.get_claims_in_rectangle(&rect![0, 1, 0, 3]);
        assert_eq!(claims.len(), 2);
        assert!(claims.contains(&(adr![0, 0], Active)));
        assert!(claims.contains(&(adr![1, 0], Blocked)));
    }

    #[test]
    fn two_claimants_overlapping_rectangles() {
        let mut map = SpillOwnershipMap::new();
        map.insert(adr![0, 0], rect![0, 0, 0, 3], Active);
        map.insert(adr![1, 0], rect![0, 2, 0, 5], Blocked);

        let claims = map.get_claims_in_rectangle(&rect![0, 0, 0, 5]);
        assert_eq!(claims.len(), 2);
        assert!(claims.contains(&(adr![0, 0], Active)));
        assert!(claims.contains(&(adr![1, 0], Blocked)));
    }

    #[test]
    fn removing_active_leaves_blocked_findable() {
        let mut map = SpillOwnershipMap::new();
        map.insert(adr![0, 0], rect![0, 1, 0, 3], Active);
        map.insert(adr![1, 0], rect![0, 1, 0, 3], Blocked);
        map.remove(&adr![0, 0]);

        let claims = map.get_claims_in_rectangle(&rect![0, 1, 0, 3]);
        assert_eq!(claims, vec![(adr![1, 0], Blocked)]);
    }

    #[test]
    fn reverse_lookup_returns_overlapping_claims() {
        let mut map = SpillOwnershipMap::new();
        map.insert(adr![0, 0], rect![0, 1, 0, 3], Active);
        map.insert(adr![1, 0], rect![1, 1, 1, 3], Active);

        let claims = map.get_claims_in_rectangle(&rect![0, 1, 1, 3]);
        assert_eq!(claims.len(), 2);
        assert!(claims.contains(&(adr![0, 0], Active)));
        assert!(claims.contains(&(adr![1, 0], Active)));
    }

    #[test]
    fn reverse_lookup_partial_overlap() {
        let mut map = SpillOwnershipMap::new();
        map.insert(adr![0, 0], rect![0, 0, 2, 2], Active);
        map.insert(adr![1, 0], rect![3, 3, 5, 5], Active);

        let claims = map.get_claims_in_rectangle(&rect![0, 0, 2, 2]);
        assert_eq!(claims, vec![(adr![0, 0], Active)]);
    }

    #[test]
    fn reverse_lookup_empty_when_no_overlap() {
        let mut map = SpillOwnershipMap::new();
        map.insert(adr![0, 0], rect![0, 0, 1, 1], Active);

        assert!(map.get_claims_in_rectangle(&rect![5, 5, 6, 6]).is_empty());
    }

    #[test]
    fn multiple_owners_same_column_different_rows() {
        let mut map = SpillOwnershipMap::new();
        map.insert(adr![0, 0], rect![0, 0, 0, 2], Active);
        map.insert(adr![0, 3], rect![0, 3, 0, 5], Active);

        assert_eq!(map.get_claims_in_rectangle(&rect![0, 0, 0, 2]), vec![(adr![0, 0], Active)]);
        assert_eq!(map.get_claims_in_rectangle(&rect![0, 3, 0, 5]), vec![(adr![0, 3], Active)]);

        let all = map.get_claims_in_rectangle(&rect![0, 0, 0, 5]);
        assert_eq!(all.len(), 2);
        assert!(all.contains(&(adr![0, 0], Active)));
        assert!(all.contains(&(adr![0, 3], Active)));
    }
}
