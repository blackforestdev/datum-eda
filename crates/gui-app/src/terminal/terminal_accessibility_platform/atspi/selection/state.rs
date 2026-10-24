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
        Some((
            !matches!(node, Node::RunErc)
                || snapshot
                    .inspection
                    .as_ref()
                    .is_some_and(|ui| ui.rerun_available),
            workspace::focus(snapshot, node)
                .unwrap_or(matches!(node, Node::Root) && snapshot.focused),
        )),
    );
    if matches!(node, Node::Member(_)) {
        words[(STATE_SELECTED / 32) as usize] |= 1 << (STATE_SELECTED % 32);
    }
    match node {
        Node::Section(section) => {
            words[0] |= (1 << 9) | (1 << if section.collapsed { 5 } else { 10 });
        }
        Node::Document(row)
            if snapshot.inspection.as_ref().is_some_and(|ui| {
                ui.documents
                    .iter()
                    .any(|child| child.parent.as_ref() == Some(&row.key))
            }) =>
        {
            words[0] |= (1 << 9) | (1 << if row.expanded { 10 } else { 5 });
        }
        Node::SectionField(section, _) if section.collapsed => {
            words[0] &= !(1 << 30);
        }
        Node::RunErc
            if snapshot.inspection.as_ref().is_some_and(|ui| {
                ui.sections.iter().any(|section| {
                    section.id == datum_gui_protocol::inspector_context::InspectorSectionId::Checks
                        && section.collapsed
                })
            }) =>
        {
            words[0] &= !(1 << 30);
        }
        _ => {}
    }
    if matches!(node, Node::Document(row) if !row.visible) {
        words[0] &= !(1 << 30);
    }
    super::super::state::words_body(words)
}
