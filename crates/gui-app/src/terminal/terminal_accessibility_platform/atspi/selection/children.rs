//! Preflight complete child-array replies against the existing D-Bus frame cap.
use super::*;
pub(super) fn paths<'a>(
    snapshot: &'a crate::selection_accessibility::SelectionAccessibilitySnapshot,
    node: Node<'a>,
) -> Box<dyn Iterator<Item = String> + 'a> {
    if let Some(ui) = snapshot.inspection.as_ref() {
        if let Some(paths) = workspace::paths(ui, node) {
            if matches!(node, Node::Root) {
                return Box::new(
                    snapshot
                        .view
                        .as_ref()
                        .map(|_| MEMBERS.to_owned())
                        .into_iter()
                        .chain(paths),
                );
            }
            return paths;
        }
    }
    let Some(view) = snapshot.view.as_ref() else {
        return Box::new(std::iter::empty());
    };
    match node {
        Node::Root => Box::new([MEMBERS, FIELDS].into_iter().map(str::to_owned)),
        Node::Members => Box::new(
            view.members
                .iter()
                .map(move |id| member_path(view.project, id)),
        ),
        Node::Fields => Box::new(view.fields().into_iter().map(field_path)),
        _ => Box::new(std::iter::empty()),
    }
}
pub(super) fn reply(
    state: &ServiceState,
    snapshot: &crate::selection_accessibility::SelectionAccessibilitySnapshot,
    node: Node<'_>,
    call: &Message,
) -> DispatchResult {
    super::super::child_array::reply(state, call, || paths(snapshot, node))
}
