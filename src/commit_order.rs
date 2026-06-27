//! Ordering convention for the inline commit selector.
//!
//! The selector stores commits **newest-first**: index `0` is the newest
//! (child-most) commit and the final index is the oldest (parent-most). Every
//! order-dependent operation routes through this module so the convention
//! lives in exactly one place — changing the stored order (for example to
//! oldest-first) becomes a change to these functions (plus the renderer and
//! cursor placement) rather than a hunt through scattered index arithmetic.
//!
//! A *selection* is an inclusive `(start, end)` index pair into the commit
//! list with `start <= end`, expressed in storage order. Because storage is
//! newest-first, `start` is the newest selected commit and `end` the oldest.

/// Inclusive `(start, end)` selection over the commit list, in storage order.
pub type SelectionRange = (usize, usize);

/// Reverse a chronological (oldest→newest) list into the selector's storage
/// order. Use at construction time wherever a VCS or forge hands us commits
/// oldest-first.
pub fn into_storage_order<T>(mut chronological: Vec<T>) -> Vec<T> {
    chronological.reverse();
    chronological
}

/// The indices covered by `range`, yielded in chronological (oldest→newest)
/// order — the order a diff wants its commits applied in.
pub fn chronological_indices(range: SelectionRange) -> impl DoubleEndedIterator<Item = usize> {
    (range.0..=range.1).rev()
}

/// Storage index of the newest (head / child-most) commit in `range`.
pub fn head_index(range: SelectionRange) -> usize {
    range.0
}

/// Storage index just past the oldest selected commit — the parent of the
/// selection. May equal the list length, meaning the selection reaches the
/// oldest commit overall and has no parent within the list.
pub fn parent_index(range: SelectionRange) -> usize {
    range.1 + 1
}

/// Selection covering the whole list of `len` commits. Callers ensure
/// `len > 0`; `saturating_sub` keeps an empty list from underflowing.
pub fn full(len: usize) -> SelectionRange {
    (0, len.saturating_sub(1))
}

/// Whether `range` spans the entire list of `len` commits.
pub fn is_full(range: SelectionRange, len: usize) -> bool {
    range.0 == 0 && range.1 + 1 == len
}

/// Selection covering every commit newer (child-ward) than the commit at
/// `reviewed_index` in storage order. `None` when nothing is newer (the
/// reviewed commit is already the newest).
pub fn newer_than(reviewed_index: usize) -> Option<SelectionRange> {
    (reviewed_index > 0).then(|| (0, reviewed_index - 1))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn into_storage_order_reverses_chronological() {
        assert_eq!(into_storage_order(vec![1, 2, 3]), vec![3, 2, 1]);
        assert_eq!(into_storage_order(Vec::<u8>::new()), Vec::<u8>::new());
    }

    #[test]
    fn chronological_indices_yields_oldest_first() {
        assert_eq!(
            chronological_indices((1, 3)).collect::<Vec<_>>(),
            vec![3, 2, 1]
        );
        assert_eq!(chronological_indices((2, 2)).collect::<Vec<_>>(), vec![2]);
    }

    #[test]
    fn head_and_parent_indices() {
        assert_eq!(head_index((0, 4)), 0);
        assert_eq!(head_index((2, 4)), 2);
        assert_eq!(parent_index((1, 3)), 4);
        assert_eq!(parent_index((0, 0)), 1);
    }

    #[test]
    fn full_and_is_full() {
        assert_eq!(full(5), (0, 4));
        assert_eq!(full(1), (0, 0));
        assert!(is_full((0, 4), 5));
        assert!(!is_full((1, 4), 5));
        assert!(!is_full((0, 3), 5));
        // An empty list is never "full" of a real selection.
        assert!(!is_full((0, 0), 0));
    }

    #[test]
    fn newer_than_excludes_when_already_newest() {
        assert_eq!(newer_than(0), None);
        assert_eq!(newer_than(1), Some((0, 0)));
        assert_eq!(newer_than(3), Some((0, 2)));
    }
}
