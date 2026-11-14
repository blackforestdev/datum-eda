//! Shared native menu classification and contextual admission.
use super::native_project_edit_available;

/// Journal ops are NOT native-write verbs (the registry does not carry them); their
/// availability is the journal's own truth (an open editable Project), never the
/// write-verb catalog. Kept as an explicit carve-out so the catalog gate below governs
/// native-EDIT verbs only.
pub(super) fn journal_verb(verb: &str) -> bool {
    matches!(verb, "datum.journal.undo" | "datum.journal.redo")
}
/// The menu's native-EDIT verb entries. Their AVAILABILITY is engine-derived from
/// native.describe's verb catalog (see `unavailable_reason`); this set only identifies
/// which menu entries route to that catalog gate.
pub(super) fn native_edit_verb(verb: &str) -> bool {
    matches!(
        verb,
        "datum.pcb.move_component"
            | "datum.pcb.draw_track"
            | "datum.pcb.delete_track"
            | "datum.pcb.place_zone"
            | "datum.pcb.set_outline"
            | "datum.check.fill_zones"
            | "datum.schematic.align_symbols"
            | "datum.schematic.create_sheet"
            | "datum.schematic.place_text"
            | "datum.schematic.delete_text"
            | "datum.schematic.place_symbol"
            | "datum.schematic.draw_wire"
            | "datum.schematic.move"
            | "datum.schematic.place_label"
            | "datum.schematic.place_port"
            | "datum.schematic.create_bus"
            | "datum.schematic.place_bus_entry"
            | "datum.schematic.rename_label"
            | "datum.schematic.place_junction"
            | "datum.schematic.place_noconnect"
            | "datum.schematic.set_symbol_reference"
            | "datum.schematic.set_symbol_value"
            | "datum.schematic.set_symbol_field"
            | "datum.schematic.delete_symbol"
            | "datum.schematic.delete_wire"
            | "datum.schematic.delete_label"
            | "datum.schematic.delete_port"
            | "datum.schematic.delete_bus"
            | "datum.schematic.delete_bus_entry"
            | "datum.schematic.delete_junction"
            | "datum.schematic.delete_noconnect"
    )
}
/// Statically build-supported edit verbs (journal carve-out + native-edit). Availability
/// is refined contextually in `unavailable_reason`.
pub(super) fn first_edit_verb(verb: &str) -> bool {
    journal_verb(verb) || native_edit_verb(verb)
}

/// Shared admission for native menu actions and GUI-local configured variants.
pub(super) fn native_project_verb_unavailable_reason(
    state: &crate::ReviewWorkspaceState,
    verb: &str,
) -> Option<&'static str> {
    if state.project_unopened || !native_project_edit_available(state) {
        return Some("Open a persistent Datum Project with --project-root to edit or save.");
    }
    if state
        .backing
        .as_ref()
        .is_some_and(|b| b.native_write_verbs.is_empty())
    {
        return Some("Project editing capabilities are not available yet.");
    }
    if !state
        .backing
        .as_ref()
        .is_some_and(|b| b.native_write_verbs.contains(verb))
    {
        return Some("This operation is not offered by the open Project's engine.");
    }
    None
}

pub fn native_edit_unavailable_reason(
    state: &crate::ReviewWorkspaceState,
    verb: &str,
) -> Option<&'static str> {
    if matches!(
        verb,
        "datum.schematic.place_text"
            | "datum.schematic.align_symbols"
            | "datum.schematic.place_symbol"
            | "datum.schematic.draw_wire"
            | "datum.schematic.place_label"
            | "datum.schematic.place_port"
            | "datum.schematic.create_bus"
            | "datum.schematic.place_bus_entry"
            | "datum.schematic.place_junction"
            | "datum.schematic.place_noconnect"
    ) && state.schematic_authoring.sheet.is_none()
    {
        return Some("This Project needs a native schematic sheet before authoring.");
    }
    if let Some(reason) = native_project_verb_unavailable_reason(state, verb) {
        return Some(reason);
    }
    if matches!(
        verb,
        "datum.pcb.draw_track"
            | "datum.pcb.place_zone"
            | "datum.pcb.set_outline"
            | "datum.check.fill_zones"
    ) {
        if state.ui.layout.focused_content() != crate::PaneContent::Board {
            return Some("Focus a native Board first.");
        }
        if verb == "datum.check.fill_zones"
            && (state.authoring.board_controls.is_some()
                || !crate::CheckProducerRequest::editor_preview_idle(state))
        {
            return Some("Finish or cancel the active editor operation before filling Zones.");
        }
    }
    if verb.starts_with("datum.schematic.")
        // New document creates a root; it does not edit the focused occurrence.
        && verb != "datum.schematic.create_sheet"
        && state.ui.layout.focused_content() == crate::PaneContent::Schematic
        && state
            .schematic_documents
            .panes
            .get(&state.ui.layout.focused)
            .is_some_and(|document| !document.instance_path.is_empty())
    {
        if !matches!(
            verb,
            "datum.schematic.align_symbols"
                | "datum.schematic.place_symbol"
                | "datum.schematic.place_text"
                | "datum.schematic.draw_wire"
                | "datum.schematic.place_marker"
                | "datum.schematic.place_junction"
                | "datum.schematic.place_noconnect"
                | "datum.schematic.set_symbol_reference"
                | "datum.schematic.set_symbol_value"
                | "datum.schematic.set_symbol_field"
                | "datum.schematic.place_label"
                | "datum.schematic.rename_label"
                | "datum.schematic.place_port"
                | "datum.schematic.create_bus"
                | "datum.schematic.place_bus_entry"
                | "datum.schematic.move_symbol"
                | "datum.schematic.move"
                | "datum.schematic.move_label"
                | "datum.schematic.delete_symbol"
                | "datum.schematic.delete_label"
                | "datum.schematic.delete_text"
                | "datum.schematic.delete_port"
                | "datum.schematic.delete_wire"
                | "datum.schematic.delete_junction"
                | "datum.schematic.delete_noconnect"
                | "datum.schematic.delete_bus"
                | "datum.schematic.delete_bus_entry"
        ) {
            return Some("This tool is not available in a child sheet.");
        }
        if crate::AuthoringViewBasis::capture_qualified_schematic(state, state.ui.layout.focused)
            .is_err()
        {
            return Some("Authoring requires one current, unrepeated sheet occurrence.");
        }
    }
    None
}
