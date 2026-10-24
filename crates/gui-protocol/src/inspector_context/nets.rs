//! Net totals describe the complete typed selection; terminals remain separate.
use super::*;
use crate::AuthoredSelectionClass as C;

pub(super) fn append<'a>(context: &mut InspectorContext<'a>, _state: &'a ReviewWorkspaceState) {
    let Some(view) = context.view else {
        return;
    };
    // Keep the existing complete collection/omission field. Named scalar rows
    // let a designer read its source and constraints without decoding an array
    // through a terminal or accessibility client. Borrow the retained values.
    if let Some(crate::selection_view::InspectionValue::Common(value)) =
        view.field_ref("board_net_assignments")
    {
        for record in value.as_array().into_iter().flatten() {
            append_board_class(context, record);
        }
    }
    if matches!(view.subject, crate::SelectionSubject::Bus(_)) {
        if let Some(crate::selection_view::InspectionValue::Common(value)) =
            view.field_ref("declared_scalar_nets")
        {
            for net in value
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(|v| v.as_str())
            {
                context.push(
                    InspectorSectionId::Connectivity,
                    InspectorField::text("Declared scalar Net", net),
                );
            }
        }
    }
    if !matches!(view.subject, crate::SelectionSubject::GlobalNet(_)) {
        return;
    }
    for (field, label, reason) in [
        (
            "board_net_assignments",
            "Class source",
            "Unavailable — no explicit BoardNet assignment in this complete selection",
        ),
        (
            "declared_bus_references",
            "Bus source",
            "Unavailable — no explicit Bus declaration contains this logical Net",
        ),
    ] {
        if matches!(view.field_ref(field), Some(crate::selection_view::InspectionValue::Common(value)) if value.as_array().is_some_and(Vec::is_empty))
        {
            context.push(
                InspectorSectionId::Connectivity,
                InspectorField::text(label, reason),
            );
        }
    }
    context.push(
        InspectorSectionId::Connectivity,
        InspectorField::text("Complete members", view.members.len().to_string()),
    );
    context.push(
        InspectorSectionId::Connectivity,
        InspectorField::text(
            "Terminals",
            view.members
                .iter()
                .filter(|id| matches!(id.class, C::Pin | C::Pad))
                .count()
                .to_string(),
        ),
    );
}

fn append_board_class<'a>(context: &mut InspectorContext<'a>, record: &'a serde_json::Value) {
    use InspectorSectionId::Connectivity;
    for (key, label) in [
        ("name", "Board Net name"),
        ("board_net_id", "Board Net UUID"),
        ("state", "Board Net source"),
        ("class_name", "Class name"),
        ("class_id", "Class UUID"),
        ("class_state", "Class source"),
    ] {
        if let Some(value) = record.get(key).and_then(|value| value.as_str()) {
            context.push(Connectivity, InspectorField::text(label, value));
        }
    }
    for (key, label) in [
        ("clearance", "Class clearance"),
        ("track_width", "Class track width"),
        ("via_drill", "Class via drill"),
        ("via_diameter", "Class via diameter"),
        ("diffpair_width", "Class differential width"),
        ("diffpair_gap", "Class differential gap"),
    ] {
        let Some(value) = record
            .get("class_constraints")
            .and_then(|values| values.get(key))
        else {
            continue;
        };
        if let Some(display) = value.get("display").and_then(|value| value.as_str()) {
            context.push(Connectivity, InspectorField::text(label, display));
        } else if let Some(nm) = value.get("canonical_nm").and_then(|value| value.as_i64()) {
            context.push(
                Connectivity,
                InspectorField::text(
                    label,
                    format!("{nm} nm — Project display units unavailable"),
                ),
            );
        }
    }
}
