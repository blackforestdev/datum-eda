//! Radial marking-menu accessibility publication guard (bead gtww runtime gate).
//!
//! Exercises the real runtime projection the protocol-only test bypassed: a
//! marking menu with no selection list must still publish its radial nodes
//! through the bridge, and closing must clear them.
use super::*;

#[test]
fn radial_marking_menu_without_selection_list_publishes_through_bridge() {
    use datum_gui_protocol::MarkingMenuState;
    use datum_gui_protocol::gui_menu_model::accessibility::MenuAccessibleRole;
    let mut workspace = datum_gui_protocol::load_fixture_workspace_state();
    workspace.ui.active_menu = None;
    // Ordinary radial wheel: open with NO selection list (selection None) —
    // the exact case the pre-radial gate suppressed to an empty publish.
    workspace.ui.marking_menu = Some(MarkingMenuState {
        menu_key: "pcb.multiselect".into(),
        target_object_id: None,
        anchor_x_px: 640,
        anchor_y_px: 400,
        preview_slot: Some("N".into()),
        gesture_dx_px: 0,
        gesture_dy_px: 0,
        selection: None,
        submenu_path: Vec::new(),
    });
    workspace.ui.focus = ApplicationFocus::Overlay;

    let mut bridge = LinuxTerminalAccessibilityBridge::without_platform();
    bridge.update_menus(super::menu_accessibility_projection(&workspace));
    assert!(
        bridge
            .menus
            .iter()
            .any(|n| n.role == MenuAccessibleRole::Menu),
        "the radial wheel (selection None) must publish a root menu node"
    );
    let rotate = bridge
        .menus
        .iter()
        .find(|n| n.name == "Rotate")
        .expect("inert Rotate leaf published to the bridge");
    assert!(!rotate.available, "inert radial leaf stays unavailable");
    assert!(
        bridge
            .menus
            .iter()
            .any(|n| n.name == "Align" && n.available),
        "Align submenu parent navigates (available)"
    );

    // Nothing open -> the empty projection clears the published nodes.
    workspace.ui.marking_menu = None;
    bridge.update_menus(super::menu_accessibility_projection(&workspace));
    assert!(
        bridge.menus.is_empty(),
        "no menubar menu and no marking menu clears the published nodes"
    );
}
