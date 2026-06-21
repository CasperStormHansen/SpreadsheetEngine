use std::collections::{HashMap, HashSet};
use crate::cell_lookup_structure::cell_address::CellAddress;

const MAX_ADDITIONS: u32 = 100;

/// A LIFO queue of [`CellAddress`] values that enforces two constraints:
/// - No address appears in the queue more than once at a time.
/// - No address can be added more than [`MAX_ADDITIONS`] times across the lifetime of the queue,
///   to prevent runaway re-evaluation caused by circular or oscillating dependencies.
pub(crate) struct EvaluationQueue {
    items: Vec<CellAddress>,
    in_queue: HashSet<CellAddress>,
    addition_counts: HashMap<CellAddress, u32>,
}

impl EvaluationQueue {
    pub(crate) fn new() -> Self {
        Self {
            items: Vec::new(),
            in_queue: HashSet::new(),
            addition_counts: HashMap::new(),
        }
    }

    pub(crate) fn push(&mut self, item: CellAddress) {
        if self.in_queue.contains(&item) {
            return;
        }
        let count = self.addition_counts.entry(item).or_insert(0);
        if *count >= MAX_ADDITIONS {
            return;
        }
        *count += 1;
        self.in_queue.insert(item);
        self.items.push(item);
    }

    pub(crate) fn pop(&mut self) -> Option<CellAddress> {
        let item = self.items.pop()?;
        self.in_queue.remove(&item);
        Some(item)
    }

    pub(crate) fn extend(&mut self, iter: impl IntoIterator<Item = CellAddress>) {
        for item in iter {
            self.push(item);
        }
    }
}

impl FromIterator<CellAddress> for EvaluationQueue {
    fn from_iter<I: IntoIterator<Item = CellAddress>>(iter: I) -> Self {
        let mut queue = Self::new();
        queue.extend(iter);
        queue
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn addr(col: u32, row: u32) -> CellAddress {
        CellAddress::new(col, row)
    }

    #[test]
    fn push_and_pop() {
        let mut queue = EvaluationQueue::new();
        queue.push(addr(0, 0));
        queue.push(addr(1, 0));
        assert_eq!(queue.pop(), Some(addr(1, 0)));
        assert_eq!(queue.pop(), Some(addr(0, 0)));
        assert_eq!(queue.pop(), None);
    }

    #[test]
    fn duplicate_push_is_ignored() {
        let mut queue = EvaluationQueue::new();
        queue.push(addr(0, 0));
        queue.push(addr(0, 0));
        assert_eq!(queue.pop(), Some(addr(0, 0)));
        assert_eq!(queue.pop(), None);
    }

    #[test]
    fn duplicate_does_not_increment_count() {
        let mut queue = EvaluationQueue::new();
        for _ in 0..MAX_ADDITIONS {
            queue.push(addr(0, 0));
            queue.pop();
        }
        queue.push(addr(0, 0));
        assert_eq!(queue.pop(), None);

        // A different address is unaffected
        queue.push(addr(1, 0));
        assert_eq!(queue.pop(), Some(addr(1, 0)));
    }

    #[test]
    fn popped_item_can_be_repushed_within_limit() {
        let mut queue = EvaluationQueue::new();
        queue.push(addr(0, 0));
        queue.pop();
        queue.push(addr(0, 0));
        assert_eq!(queue.pop(), Some(addr(0, 0)));
    }

    #[test]
    fn addition_limit_is_enforced() {
        let mut queue = EvaluationQueue::new();
        for _ in 0..MAX_ADDITIONS {
            queue.push(addr(0, 0));
            queue.pop();
        }
        queue.push(addr(0, 0));
        assert_eq!(queue.pop(), None);
    }

    #[test]
    fn extend_deduplicates() {
        let mut queue = EvaluationQueue::new();
        queue.push(addr(0, 0));
        queue.extend([addr(0, 0), addr(1, 0), addr(1, 0), addr(2, 0)]);
        assert_eq!(queue.pop(), Some(addr(2, 0)));
        assert_eq!(queue.pop(), Some(addr(1, 0)));
        assert_eq!(queue.pop(), Some(addr(0, 0)));
        assert_eq!(queue.pop(), None);
    }

    #[test]
    fn from_iterator_deduplicates() {
        let mut queue: EvaluationQueue =
            [addr(0, 0), addr(1, 0), addr(0, 0), addr(2, 0)].into_iter().collect();
        assert_eq!(queue.pop(), Some(addr(2, 0)));
        assert_eq!(queue.pop(), Some(addr(1, 0)));
        assert_eq!(queue.pop(), Some(addr(0, 0)));
        assert_eq!(queue.pop(), None);
    }
}
