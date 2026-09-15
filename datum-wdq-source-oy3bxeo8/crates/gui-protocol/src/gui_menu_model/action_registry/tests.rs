use super::*;
use crate::{PaneId, WorkspaceLayout, load_default_gui_menu_model, load_fixture_workspace_state};

#[test]
fn production_registry_agrees_with_reviewed_pilot_consumers() {
    let contract: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../../specs/workflow_delivery/pilot.contract.json"
    ))
    .unwrap();
    let expected = contract["consumers"].as_array().unwrap();
    assert_eq!(expected.len(), ACTION_CONSUMERS.len());
    for record in expected {
        let entry = consumer(record["dispatch_key"].as_str().unwrap()).unwrap();
        assert_eq!(record["unavailable_reason"], entry.unavailable_reason);
        let surfaces = entry.entry_surfaces.to_vec();
        assert_eq!(record["entry_surfaces"], serde_json::json!(surfaces));
        match entry.handler_ref() {
            Some((path, symbol)) => {
                assert_eq!(record["handler_ref"]["path"], path);
                assert_eq!(record["handler_ref"]["symbol"], symbol);
            }
            None => assert!(record["handler_ref"].is_null()),
        }
    }
}

#[test]
fn fit_admission_tracks_real_focused_scene_without_board_fallback() {
    let mut state = load_fixture_workspace_state();
    state.ui.layout = WorkspaceLayout::board_schematic();
    state.schematic_scene = None;
    assert_eq!(admit("view.fit", &state), Ok(ActionHandler::FitCamera));
    state.ui.layout.focused = PaneId(1);
    assert!(admit("view.fit", &state).is_err());
    assert!(camera_scene_for_pane(&state, PaneId(1)).is_none());
    state.schematic_scene = Some(state.scene.clone());
    assert_eq!(admit("view.fit", &state), Ok(ActionHandler::FitCamera));
    state.ui.layout.zoomed = Some(PaneId(0));
    assert!(admit("view.fit", &state).is_err());
    state.ui.layout.focused = PaneId(0);
    assert_eq!(admit("view.fit", &state), Ok(ActionHandler::FitCamera));
    state.ui.layout.focused = PaneId(999);
    assert!(admit("view.fit", &state).is_err());
}

#[test]
fn unavailable_consumers_and_unknown_keys_never_admit() {
    let mut state = load_fixture_workspace_state();
    for pane in [PaneId(0), PaneId(1), PaneId(999)] {
        state.ui.layout.focused = pane;
        for key in ["help.about", "window.documents", "view.layers"] {
            let entry = consumer(key).unwrap();
            assert_eq!(admit(key, &state), Err(entry.unavailable_reason));
            assert!(entry.handler_ref().is_none());
        }
        assert!(admit("unknown.pilot.key", &state).is_err());
    }
}

#[test]
fn real_menu_rows_consume_registry_and_other_families_retain_ownership() {
    let model = load_default_gui_menu_model().unwrap();
    let mut state = load_fixture_workspace_state();
    state.ui.layout = WorkspaceLayout::board_schematic();
    state.schematic_scene = None;
    let items: Vec<_> = model
        .menubar
        .iter()
        .flat_map(|group| &group.items)
        .collect();
    for entry in ACTION_CONSUMERS {
        let item = items
            .iter()
            .find(|item| item.gui_local.as_deref() == Some(entry.key))
            .unwrap();
        assert_eq!(item.unavailable_reason(&state), entry.admit(&state).err());
        assert_eq!(item.is_phase_one_enabled(), entry.handler.is_some());
        state.ui.layout.focused = PaneId(1);
        assert_eq!(item.unavailable_reason(&state), entry.admit(&state).err());
        state.ui.layout.focused = PaneId(0);
    }
    let edit = model
        .menubar
        .iter()
        .find(|group| group.menu == "Edit")
        .unwrap();
    let global = &edit.submenus["edit.preferences"][0];
    assert!(consumer(global.gui_local.as_deref().unwrap()).is_none());
    assert!(global.unavailable_reason(&state).is_none());
    let focus_next = items
        .iter()
        .find(|item| item.gui_local.as_deref() == Some("view.focus_next"))
        .unwrap();
    assert!(focus_next.unavailable_reason(&state).is_none());
}
