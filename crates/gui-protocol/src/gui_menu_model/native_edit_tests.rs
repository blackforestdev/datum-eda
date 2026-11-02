//! Native menu availability follows the discovered writer catalog.
use super::*;
fn native_edit_item(verb: &str) -> GuiMenuItem {
    GuiMenuItem {
        label: verb.to_string(),
        icon: None,
        shortcut: None,
        destructive: false,
        verb: Some(verb.to_string()),
        gui_local: None,
        not_built: None,
        submenu: None,
        requires_project: true,
    }
}

fn native_editable_state(catalog: &[&str]) -> crate::ReviewWorkspaceState {
    let mut state = crate::load_fixture_workspace_state();
    state.backing = Some(crate::WorkspaceBacking {
        request: crate::LiveReviewRequest {
            project_root: std::path::PathBuf::from("/tmp/datum menu capability demo"),
            board_file: None,
            artifact_path: None,
            net_uuid: None,
            from_anchor_pad_uuid: None,
            to_anchor_pad_uuid: None,
            profile: None,
            kicad_board_source: None,
        },
        board_path: std::path::PathBuf::from("/tmp/datum menu capability demo/board/board.json"),
        native_write_verbs: catalog.iter().map(|verb| verb.to_string()).collect(),
    });
    state
}

#[test]
fn native_edit_entry_enabled_when_the_engine_reports_the_verb() {
    let item = native_edit_item("datum.pcb.move_component");
    let state = native_editable_state(&["datum.pcb.move_component"]);
    assert_eq!(item.unavailable_reason(&state), None);
}

#[test]
fn native_edit_entry_disabled_when_the_verb_is_absent_from_the_catalog() {
    let item = native_edit_item("datum.pcb.move_component");
    // A verb the engine does not enumerate must leave the entry disabled rather than
    // hardcoded-on; the truthful-default empty catalog is likewise disabled.
    let absent = native_editable_state(&["datum.schematic.place_symbol"]);
    assert_eq!(
        item.unavailable_reason(&absent),
        Some("This operation is not offered by the open Project's engine.")
    );
    let empty = native_editable_state(&[]);
    assert_eq!(
        item.unavailable_reason(&empty),
        Some("Project editing capabilities are not available yet.")
    );
}

#[test]
fn native_edit_entry_disabled_without_an_open_editable_project() {
    let item = native_edit_item("datum.pcb.move_component");
    // No backing (no writable Project) is an absent catalog by construction.
    let state = crate::load_fixture_workspace_state();
    assert!(item.unavailable_reason(&state).is_some());
}

#[test]
fn new_schematic_uses_engine_catalog_without_requiring_an_existing_sheet() {
    let menu = default_gui_menu_model().unwrap();
    let item = menu
        .menubar
        .iter()
        .find(|m| m.menu == "File")
        .unwrap()
        .items
        .iter()
        .find(|i| i.verb.as_deref() == Some("datum.schematic.create_sheet"))
        .unwrap();
    let mut state = native_editable_state(&["datum.schematic.create_sheet"]);
    state.schematic_authoring.sheet = None;
    assert_eq!(item.unavailable_reason(&state), None);
    state.backing.as_mut().unwrap().native_write_verbs.clear();
    assert!(item.unavailable_reason(&state).is_some());
    state
        .backing
        .as_mut()
        .unwrap()
        .native_write_verbs
        .insert("datum.schematic.create_sheet".into());
    state.project_unopened = true;
    assert_eq!(
        item.unavailable_reason(&state),
        Some("This action requires an open Project.")
    );
}

#[test]
fn journal_verbs_are_not_governed_by_the_write_verb_catalog() {
    // Regression trap: undo/redo are not registered native-write verbs, so an empty
    // catalog must NOT disable them; their only gate is an open editable Project.
    let undo = native_edit_item("datum.journal.undo");
    let empty_catalog = native_editable_state(&[]);
    assert_eq!(undo.unavailable_reason(&empty_catalog), None);
    let no_project = crate::load_fixture_workspace_state();
    assert!(undo.unavailable_reason(&no_project).is_some());
}

#[test]
fn label_and_power_port_share_sheet_and_native_capability_admission() {
    let menu = default_gui_menu_model().unwrap();
    let items = &menu
        .menubar
        .iter()
        .find(|m| m.menu == "Place" && m.active_editor.as_deref() == Some("schematic"))
        .unwrap()
        .items;
    let label = items.iter().find(|i| i.label == "Label").unwrap();
    let power = items.iter().find(|i| i.label == "Power Port").unwrap();
    assert_eq!(
        power.gui_local.as_deref(),
        Some("schematic.power_port.open")
    );
    let mut state = native_editable_state(&["datum.schematic.place_label"]);
    state.schematic_authoring.sheet = Some(uuid::Uuid::new_v4());
    for item in [label, power] {
        assert_eq!(item.unavailable_reason(&state), None);
    }
    state.backing.as_mut().unwrap().native_write_verbs.clear();
    for item in [label, power] {
        assert_eq!(
            item.unavailable_reason(&state),
            Some("Project editing capabilities are not available yet.")
        );
    }
    state
        .backing
        .as_mut()
        .unwrap()
        .native_write_verbs
        .insert("datum.schematic.place_symbol".into());
    for item in [label, power] {
        assert_eq!(
            item.unavailable_reason(&state),
            Some("This operation is not offered by the open Project's engine.")
        );
    }
    state
        .backing
        .as_mut()
        .unwrap()
        .native_write_verbs
        .insert("datum.schematic.place_label".into());
    state.schematic_authoring.sheet = None;
    for item in [label, power] {
        assert_eq!(
            item.unavailable_reason(&state),
            Some("This Project needs a native schematic sheet before authoring.")
        );
    }
}

#[test]
fn schematic_place_entries_require_sheet_and_each_engine_capability() {
    let model = default_gui_menu_model().unwrap();
    let place = model
        .menubar
        .iter()
        .find(|menu| menu.menu == "Place")
        .unwrap();
    for (label, verb) in [
        ("Junction", "datum.schematic.place_junction"),
        ("No-Connect", "datum.schematic.place_noconnect"),
        ("Port", "datum.schematic.place_port"),
        ("Bus", "datum.schematic.create_bus"),
        ("Bus Entry", "datum.schematic.place_bus_entry"),
    ] {
        let item = place.items.iter().find(|item| item.label == label).unwrap();
        assert_eq!(item.verb.as_deref(), Some(verb));
        assert!(item.is_phase_one_enabled());
        let mut state = native_editable_state(&[verb]);
        state.schematic_authoring.sheet = Some(uuid::Uuid::new_v4());
        assert_eq!(item.unavailable_reason(&state), None);
        state
            .backing
            .as_mut()
            .unwrap()
            .native_write_verbs
            .remove(verb);
        state
            .backing
            .as_mut()
            .unwrap()
            .native_write_verbs
            .insert("datum.schematic.place_symbol".into());
        assert_eq!(
            item.unavailable_reason(&state),
            Some("This operation is not offered by the open Project's engine.")
        );
        state
            .backing
            .as_mut()
            .unwrap()
            .native_write_verbs
            .insert(verb.into());
        state.schematic_authoring.sheet = None;
        assert_eq!(
            item.unavailable_reason(&state),
            Some("This Project needs a native schematic sheet before authoring.")
        );
    }
}

#[test]
fn marker_delete_entries_use_native_catalog_without_placement_prerequisites() {
    for verb in [
        "datum.schematic.delete_junction",
        "datum.schematic.delete_noconnect",
    ] {
        let item = native_edit_item(verb);
        let mut state = native_editable_state(&[verb]);
        assert!(item.is_phase_one_enabled());
        assert_eq!(item.unavailable_reason(&state), None);
        state.backing.as_mut().unwrap().native_write_verbs.clear();
        assert!(item.unavailable_reason(&state).is_some());
    }
}
