//! Shared Inspector wording for native accessibility; canonical attributes remain exact.
use super::*;
fn field_value(view: &datum_gui_protocol::selection_view::SelectionView, field: &str) -> String {
    view.field_display_preview(field, usize::MAX)
}
fn member_label(view: &datum_gui_protocol::selection_view::SelectionView, id: &Id) -> String {
    let ordinal = view
        .members
        .iter()
        .position(|member| member == id)
        .unwrap_or(0)
        + 1;
    view.member_display_label(id, ordinal)
}
pub(super) fn name(
    view: &datum_gui_protocol::selection_view::SelectionView,
    node: Node<'_>,
) -> String {
    match node {
        Node::Root => view.summary(),
        Node::Members => format!("Complete inventory: {} members", view.members.len()),
        Node::Fields => format!(
            "Read-only properties: {} members in {}",
            view.scoped_members().count(),
            view.scope
                .map(
                    |class| datum_gui_protocol::selection_view::inspector_state::class_label(class)
                        .to_owned()
                )
                .unwrap_or_else(|| "All".into())
        ),
        Node::Member(id) => member_label(view, id),
        Node::Field(field) => format!(
            "{}: {}",
            datum_gui_protocol::selection_view::field_label(field),
            field_value(view, field)
        ),
    }
}
pub(super) fn description(
    snapshot: &crate::selection_accessibility::SelectionAccessibilitySnapshot,
    node: Node<'_>,
) -> String {
    let view = &snapshot.view;
    let pane = snapshot
        .pane
        .map(|pane| format!("Editor pane {}", pane.0))
        .unwrap_or_else(|| "Editor inactive".into());
    let evaluation = snapshot
        .evaluating
        .map(|(_, count)| {
            format!("Evaluating selection: {count} candidates examined; Escape cancels.")
        })
        .unwrap_or_default();
    let lod = snapshot
        .visual_lod
        .map(|lod| lod.to_string())
        .unwrap_or_else(|| "Selection appearance awaiting redraw".into());
    match node {
        Node::Root=>format!("Read-only {}. {}. {} {} unprojected; {} dropped. {} Move, rotate, mirror, lock and group unavailable for read-only selection. {} related identities keep normal authored appearance and are not selected. {lod}.",view.summary(),pane,evaluation,view.unprojected.len(),view.dropped.len(),if view.stale { "Stale previous basis; current geometry unavailable." } else { "Current resolved basis." },view.related.len()),
        Node::Member(id) => format!(
            "Selected {}. Sheet occurrence depth {}; hidden {}; locked {}; unprojected {}; selection focus {}; unavailable properties {}; read-only.",
            member_label(view, id), id.instance_path.len(), view.hidden.contains(id), view.locked.contains(id),
            view.unprojected.contains(id), view.focus() == Some(id),
            if view.unavailable_fields(id).is_empty() { "None".into() } else {
                view.unavailable_fields(id).into_iter().map(datum_gui_protocol::selection_view::field_label).collect::<Vec<_>>().join(", ")
            }
        ),
        Node::Field(field)=>field_value(view,field),
        Node::Members=>"Complete authoritative membership, including hidden and cross-sheet occurrences. No transport cap.".into(),
        Node::Fields=>"Common, Mixed and Unavailable compare canonical values in the complete declared scope. No editable fields.".into(),
    }
}
