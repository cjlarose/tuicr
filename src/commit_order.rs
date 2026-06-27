//! Ordering convention for the inline commit selector.
//!
//! The selector holds a review's commits as an ordered **walk** — the sequence
//! a VCS revision walk (`git rev-list`) or forge API produces. That sequence is
//! a topological order, so two adjacent entries are not "older" and "newer"
//! than one another in any wall-clock sense; the only robust facts are a
//! commit's *position* in the walk and the walk's two endpoints.
//!
//! The endpoints carry the only direction that matters, and it comes from the
//! reviewed range `base..head`, not from time:
//! - the **head** end is the range's head; a selection's head commit is the one
//!   whose tree forms the diff's new side;
//! - the **base** end is the range's base boundary; the commit just past it is
//!   the diff's base (old side).
//!
//! The selector stores the walk **head-end first** (index `0` is the head end).
//! Every order-dependent operation routes through this module, so the storage
//! direction lives in exactly one place: changing it (for example to display
//! the walk base-end first) becomes a change to these functions — plus the
//! renderer and cursor placement — while the head/base *roles* stay fixed, so
//! callers keep working unchanged.
//!
//! A *selection* is an inclusive `(start, end)` index pair into the walk with
//! `start <= end`. With head-end-first storage, `start` is the selection's
//! head-side position and `end` its base-side position.

/// Inclusive `(start, end)` selection over the commit walk, in storage order.
pub type SelectionRange = (usize, usize);

/// Reverse a base-end-first walk — the order a VCS revision walk or forge API
/// yields (base boundary first, head last) — into the selector's storage order,
/// which is head-end first.
pub fn into_storage_order<T>(mut base_to_head: Vec<T>) -> Vec<T> {
    base_to_head.reverse();
    base_to_head
}

/// The indices of `range` yielded base-end first, head-end last — the order a
/// diff consumes a commit list (first entry is the base side).
pub fn base_to_head_indices(range: SelectionRange) -> impl DoubleEndedIterator<Item = usize> {
    (range.0..=range.1).rev()
}

/// Storage index of the selection's head commit — the one whose tree forms the
/// diff's new side.
pub fn head_index(range: SelectionRange) -> usize {
    range.0
}

/// Storage index just past the base end of the selection: the commit adjacent
/// to it on the base side, used as the diff's base (old side). May equal the
/// walk length, meaning the selection reaches the base end and no commit lies
/// beyond it.
pub fn base_boundary_index(range: SelectionRange) -> usize {
    range.1 + 1
}

/// Selection covering the whole walk of `len` commits. Callers ensure
/// `len > 0`; `saturating_sub` keeps an empty walk from underflowing.
pub fn full(len: usize) -> SelectionRange {
    (0, len.saturating_sub(1))
}

/// Whether `range` spans the entire walk of `len` commits.
pub fn is_full(range: SelectionRange, len: usize) -> bool {
    range.0 == 0 && range.1 + 1 == len
}

/// Selection covering the commits on the head side of the commit at `index` —
/// the positions ahead of it in the walk. Used to scope "everything past the
/// last-reviewed commit". `None` when nothing lies on its head side (it is
/// already at the head end).
pub fn head_side_of(index: usize) -> Option<SelectionRange> {
    (index > 0).then(|| (0, index - 1))
}

/// Direction the selector *renders* the walk in. The stored/canonical order is
/// always head-end first (see the module docs); this affects presentation and
/// navigation only — never the persisted selection indices, the diff, or the
/// head/base roles.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DisplayOrder {
    /// Head end at the top (storage index `0` first) — the historical default.
    #[default]
    HeadFirst,
    /// Base end at the top (parent → child, like a GitHub PR's commit list).
    BaseFirst,
}

impl DisplayOrder {
    /// Parse a `commit_order` config value. Returns `None` for unknown values
    /// so the config layer can warn and fall back to the default.
    pub fn parse_name(s: &str) -> Option<Self> {
        match s {
            "head-first" => Some(Self::HeadFirst),
            "base-first" => Some(Self::BaseFirst),
            _ => None,
        }
    }
}

/// Convert between a storage position (head-end first) and its display row
/// under `order`, for a list of `n` rows. The mapping is an involution, so the
/// same call converts storage→row and row→storage.
pub fn display_position(pos: usize, n: usize, order: DisplayOrder) -> usize {
    match order {
        DisplayOrder::HeadFirst => pos,
        DisplayOrder::BaseFirst => n.saturating_sub(1).saturating_sub(pos),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn into_storage_order_reverses_walk() {
        assert_eq!(into_storage_order(vec![1, 2, 3]), vec![3, 2, 1]);
        assert_eq!(into_storage_order(Vec::<u8>::new()), Vec::<u8>::new());
    }

    #[test]
    fn base_to_head_indices_yields_base_end_first() {
        assert_eq!(
            base_to_head_indices((1, 3)).collect::<Vec<_>>(),
            vec![3, 2, 1]
        );
        assert_eq!(base_to_head_indices((2, 2)).collect::<Vec<_>>(), vec![2]);
    }

    #[test]
    fn head_and_base_boundary_indices() {
        assert_eq!(head_index((0, 4)), 0);
        assert_eq!(head_index((2, 4)), 2);
        assert_eq!(base_boundary_index((1, 3)), 4);
        assert_eq!(base_boundary_index((0, 0)), 1);
    }

    #[test]
    fn full_and_is_full() {
        assert_eq!(full(5), (0, 4));
        assert_eq!(full(1), (0, 0));
        assert!(is_full((0, 4), 5));
        assert!(!is_full((1, 4), 5));
        assert!(!is_full((0, 3), 5));
        // An empty walk is never "full" of a real selection.
        assert!(!is_full((0, 0), 0));
    }

    #[test]
    fn head_side_of_excludes_when_at_head_end() {
        assert_eq!(head_side_of(0), None);
        assert_eq!(head_side_of(1), Some((0, 0)));
        assert_eq!(head_side_of(3), Some((0, 2)));
    }

    #[test]
    fn display_position_is_identity_for_head_first() {
        for i in 0..5 {
            assert_eq!(display_position(i, 5, DisplayOrder::HeadFirst), i);
        }
    }

    #[test]
    fn display_position_reverses_for_base_first_and_is_involution() {
        assert_eq!(display_position(0, 5, DisplayOrder::BaseFirst), 4);
        assert_eq!(display_position(4, 5, DisplayOrder::BaseFirst), 0);
        for i in 0..5 {
            let row = display_position(i, 5, DisplayOrder::BaseFirst);
            assert_eq!(display_position(row, 5, DisplayOrder::BaseFirst), i);
        }
    }

    #[test]
    fn display_order_parses_known_values() {
        assert_eq!(
            DisplayOrder::parse_name("head-first"),
            Some(DisplayOrder::HeadFirst)
        );
        assert_eq!(
            DisplayOrder::parse_name("base-first"),
            Some(DisplayOrder::BaseFirst)
        );
        assert_eq!(DisplayOrder::parse_name("nope"), None);
        assert_eq!(DisplayOrder::default(), DisplayOrder::HeadFirst);
    }
}
