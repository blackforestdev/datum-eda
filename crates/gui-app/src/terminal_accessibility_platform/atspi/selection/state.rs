//! Standard selected state comes only from current, membership-qualified nodes.
use super::*;

const STATE_SELECTED: u32 = 23;

pub(super) fn body(
    snapshot: &crate::selection_accessibility::SelectionAccessibilitySnapshot,
    node: Node<'_>,
) -> Vec<u8> {
    let mut words = super::super::state::state_words(
        false,
        false,
        Some((true, matches!(node, Node::Root) && snapshot.focused)),
    );
    if matches!(node, Node::Member(_)) {
        words[(STATE_SELECTED / 32) as usize] |= 1 << (STATE_SELECTED % 32);
    }
    super::super::state::words_body(words)
}
