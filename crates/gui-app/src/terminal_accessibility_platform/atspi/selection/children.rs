//! Preflight complete child-array replies against the existing D-Bus frame cap.
use super::*;
fn paths<'a>(
    view: &'a datum_gui_protocol::selection_view::SelectionView,
    node: Node<'_>,
) -> Box<dyn Iterator<Item = String> + 'a> {
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
    view: &datum_gui_protocol::selection_view::SelectionView,
    node: Node<'_>,
    call: &Message,
) -> DispatchResult {
    super::super::child_array::reply(state, call, || paths(view, node))
}
