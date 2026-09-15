use super::*;
use datum_gui_protocol::gui_menu_model::accessibility::menu_accessibility_nodes;
use datum_gui_protocol::{
    ApplicationFocus, PaneId, load_default_gui_menu_model, load_fixture_workspace_state,
};

fn state() -> (ServiceState, datum_gui_protocol::ReviewWorkspaceState) {
    let mut workspace = load_fixture_workspace_state();
    workspace.ui.active_menu = Some("View".to_owned());
    workspace.ui.focus = ApplicationFocus::Overlay;
    workspace.schematic_scene = None;
    let snapshot = TerminalAccessibilitySnapshot {
        session_id: "terminal-a".into(),
        title: "Terminal".into(),
        text: "terminal preserved".into(),
        caret: 0,
        selection: None,
        links: Vec::new(),
        focused: false,
        bell_count: 0,
        bounds: Default::default(),
    };
    let mut service = ServiceState::new(
        snapshot,
        true,
        vec![GlobalPreferencesAccessibleNode {
            id: "preferences".into(),
            name: "Preferences".into(),
            role: GlobalPreferencesAccessibleRole::Dialog,
            value: None,
            description: "Retained consumer".into(),
            available: true,
            focused: false,
        }],
    );
    service.menus = menu_accessibility_nodes(&load_default_gui_menu_model().unwrap(), &workspace);
    (service, workspace)
}

fn call(path: &str, interface: &str, member: &str, signature: &str, body: Vec<u8>) -> Message {
    Message::method_call(7, "ignored", path, interface, member, signature, body)
}

#[test]
fn live_menu_nodes_preserve_sibling_consumers_and_refuse_phantoms() {
    let (mut service, _) = state();
    assert_eq!(
        service.root_child_paths(),
        vec![
            TERMINAL_PATH.to_owned(),
            preference_path(0),
            path("menu:View")
        ]
    );
    assert_eq!(service.snapshot.text, "terminal preserved");
    let row = path("menu:View/view.layers");
    let state_reply = service.dispatch(1, &call(&row, ACCESSIBLE, "GetState", "", Vec::new()));
    let words = state_reply.body_reader().u32_array().unwrap();
    assert_eq!(words[0] & (1 << STATE_ENABLED), 0);
    assert_eq!(words[0] & (1 << STATE_SENSITIVE), 0);
    assert_ne!(words[0] & (1 << STATE_FOCUSABLE), 0);
    let mut body = BodyWriter::new();
    body.string(ACCESSIBLE);
    body.string("Description");
    let description = service.dispatch(2, &call(&row, PROPERTIES, "Get", "ss", body.finish()));
    assert!(
        description
            .body_reader()
            .variant_string()
            .unwrap()
            .contains("unavailable in this build")
    );
    let role = service.dispatch(3, &call(&row, ACCESSIBLE, "GetRole", "", Vec::new()));
    assert_eq!(role.body_reader().u32().unwrap(), 35);
    let refusal = service.dispatch(
        4,
        &call(&row, "org.a11y.atspi.Action", "DoAction", "i", i32_body(0)),
    );
    assert_eq!(refusal.kind, MessageType::Error);
    service.menus.clear();
    let phantom = service.dispatch(5, &call(&row, ACCESSIBLE, "GetState", "", Vec::new()));
    assert_eq!(phantom.kind, MessageType::Error);
    assert_eq!(service.root_child_paths().len(), 2);
    assert_eq!(service.preferences[0].name, "Preferences");
}

#[test]
fn focus_context_and_dismissal_emit_real_menu_events() {
    let (service, mut workspace) = state();
    workspace.ui.layout.focused = PaneId(1);
    let next = menu_accessibility_nodes(&load_default_gui_menu_model().unwrap(), &workspace);
    let mut serial = 0;
    let changes = messages(
        || {
            serial += 1;
            serial
        },
        ":1.9",
        &service.menus,
        &next,
        2,
        2,
    );
    let fit = path("menu:View/view.fit");
    assert!(
        changes
            .iter()
            .any(|event| event.header.path.as_deref() == Some(&fit)
                && event.header.member.as_deref() == Some("StateChanged")
                && event.body_reader().string().unwrap() == "enabled")
    );
    let removed = messages(
        || {
            serial += 1;
            serial
        },
        ":1.9",
        &next,
        &[],
        2,
        2,
    );
    assert_eq!(removed.len(), next.len());
    assert!(
        removed
            .iter()
            .all(|event| event.header.member.as_deref() == Some("ChildrenChanged"))
    );
    assert!(
        removed
            .iter()
            .all(|event| event.body_reader().string().unwrap() == "remove")
    );
}

#[test]
fn coalesced_sibling_changes_keep_previous_removal_and_next_addition_indices() {
    let (service, _) = state();
    let removed = messages(|| 1, ":1.9", &service.menus, &[], 1, 2);
    let root_removal = removed
        .iter()
        .find(|event| event.header.path.as_deref() == Some(ROOT_PATH))
        .unwrap();
    let mut body = root_removal.body_reader();
    assert_eq!(body.string().unwrap(), "remove");
    assert_eq!(body.i32().unwrap(), 1);

    let added = messages(|| 1, ":1.9", &[], &service.menus, 2, 1);
    let root_addition = added
        .iter()
        .find(|event| event.header.path.as_deref() == Some(ROOT_PATH))
        .unwrap();
    let mut body = root_addition.body_reader();
    assert_eq!(body.string().unwrap(), "add");
    assert_eq!(body.i32().unwrap(), 1);
}

#[test]
fn nested_preferences_menu_is_a_menu_projection_not_a_preferences_replacement() {
    let (mut service, mut workspace) = state();
    workspace.ui.active_menu = Some("Edit".into());
    workspace.ui.active_submenu = Some("edit.preferences".into());
    service.menus = menu_accessibility_nodes(&load_default_gui_menu_model().unwrap(), &workspace);
    let global = service
        .menus
        .iter()
        .find(|node| node.key.as_deref() == Some("preferences.global.open"))
        .unwrap();
    assert!(global.focused);
    assert_eq!(global.parent.as_deref(), Some("menu:edit.preferences"));
    assert_eq!(service.preferences[0].name, "Preferences");
    workspace.ui.active_menu = None;
    assert!(
        menu_accessibility_nodes(&load_default_gui_menu_model().unwrap(), &workspace).is_empty()
    );
}
