use super::atspi::{PREFERENCES_PATH, ROOT_PATH, ServiceState, TERMINAL_PATH};
use super::body::BodyWriter;
use super::dbus::Message;
use crate::terminal_accessibility::{
    TerminalAccessibilityBounds, TerminalAccessibilityLink, TerminalAccessibilitySnapshot,
};
use datum_gui_protocol::{GlobalPreferencesAccessibleNode, GlobalPreferencesAccessibleRole};

const ACCESSIBLE: &str = "org.a11y.atspi.Accessible";
const COMPONENT: &str = "org.a11y.atspi.Component";
const HYPERLINK: &str = "org.a11y.atspi.Hyperlink";
const HYPERTEXT: &str = "org.a11y.atspi.Hypertext";
const PROPERTIES: &str = "org.freedesktop.DBus.Properties";
const TEXT: &str = "org.a11y.atspi.Text";

fn snapshot() -> TerminalAccessibilitySnapshot {
    TerminalAccessibilitySnapshot {
        session_id: "session-7".into(),
        title: "Agent terminal".into(),
        text: "alpha βeta\nlink".into(),
        caret: 6,
        selection: Some((0, 5)),
        links: vec![TerminalAccessibilityLink {
            start: 11,
            end: 15,
            uri: "https://datum.example".into(),
        }],
        focused: true,
        bell_count: 0,
        bounds: TerminalAccessibilityBounds {
            x: 10,
            y: 20,
            width: 800,
            height: 320,
        },
    }
}

fn call(interface: &str, member: &str, signature: &str, body: Vec<u8>) -> Message {
    let mut call = Message::method_call(
        9,
        "ignored",
        TERMINAL_PATH,
        interface,
        member,
        signature,
        body,
    );
    call.header.sender = Some(":1.55".into());
    call
}

#[test]
fn text_offsets_are_unicode_scalars_not_utf8_bytes() {
    let mut service = ServiceState::new(snapshot(), true, Vec::new());
    let mut body = BodyWriter::new();
    body.i32(6);
    body.i32(10);
    let reply = service.dispatch(1, &call(TEXT, "GetText", "ii", body.finish()));
    assert_eq!(reply.header.signature, "s");
    assert_eq!(reply.body_reader().string().unwrap(), "βeta");
    let property = {
        let mut body = BodyWriter::new();
        body.string(TEXT);
        body.string("CharacterCount");
        body.finish()
    };
    let reply = service.dispatch(2, &call(PROPERTIES, "Get", "ss", property));
    assert_eq!(reply.body_reader().variant_i32().unwrap(), 15);
}

#[test]
fn terminal_semantics_expose_role_selection_and_links() {
    let mut service = ServiceState::new(snapshot(), true, Vec::new());
    let role = service.dispatch(1, &call(ACCESSIBLE, "GetRole", "", Vec::new()));
    assert_eq!(role.body_reader().u32().unwrap(), 60);
    let count = service.dispatch(2, &call(HYPERTEXT, "GetNLinks", "", Vec::new()));
    assert_eq!(count.body_reader().i32().unwrap(), 1);
    let selection = {
        let mut body = BodyWriter::new();
        body.i32(0);
        body.finish()
    };
    let reply = service.dispatch(3, &call(TEXT, "GetSelection", "i", selection));
    let mut reader = reply.body_reader();
    assert_eq!((reader.i32().unwrap(), reader.i32().unwrap()), (0, 5));

    let link = service.dispatch(4, &call(HYPERTEXT, "GetLink", "i", i32_arg(0)));
    let mut reference = link.body_reader();
    assert_eq!(reference.string().unwrap(), "");
    let path = reference.object_path().unwrap();
    let uri = service.dispatch(5, &call_at(&path, HYPERLINK, "GetURI", "i", i32_arg(0)));
    assert_eq!(uri.body_reader().string().unwrap(), "https://datum.example");
}

#[test]
fn properties_focus_and_geometry_match_the_immutable_snapshot() {
    let mut service = ServiceState::new(snapshot(), true, Vec::new());
    let name = service.dispatch(
        1,
        &call(PROPERTIES, "Get", "ss", two_strings(ACCESSIBLE, "Name")),
    );
    assert_eq!(
        name.body_reader().variant_string().unwrap(),
        "Agent terminal"
    );

    let caret = service.dispatch(
        2,
        &call(PROPERTIES, "Get", "ss", two_strings(TEXT, "CaretOffset")),
    );
    assert_eq!(caret.body_reader().variant_i32().unwrap(), 6);

    let state = service.dispatch(3, &call(ACCESSIBLE, "GetState", "", Vec::new()));
    let words = state.body_reader().u32_array().unwrap();
    assert_ne!(words[0] & (1 << 12), 0, "focused state bit");
    assert_ne!(words[1] & (1 << (38 - 32)), 0, "selectable-text state bit");

    let extents = service.dispatch(4, &call(COMPONENT, "GetExtents", "u", u32_arg(0)));
    let mut bounds = extents.body_reader();
    assert_eq!(
        (
            bounds.i32().unwrap(),
            bounds.i32().unwrap(),
            bounds.i32().unwrap(),
            bounds.i32().unwrap(),
        ),
        (10, 20, 800, 320)
    );
}

#[test]
fn malformed_object_calls_fail_closed_without_mutating_service_state() {
    let mut service = ServiceState::new(snapshot(), true, Vec::new());
    let invalid = service.dispatch(1, &call(HYPERTEXT, "GetLink", "i", i32_arg(9)));
    assert_eq!(invalid.kind, super::dbus::MessageType::Error);
    assert_eq!(service.application_id, 0);

    let mut set = BodyWriter::new();
    set.string("org.a11y.atspi.Application");
    set.string("Id");
    set.variant("s", |body| body.string("not an integer"));
    let invalid = service.dispatch(
        2,
        &call_at(
            super::atspi::ROOT_PATH,
            PROPERTIES,
            "Set",
            "ssv",
            set.finish(),
        ),
    );
    assert_eq!(invalid.kind, super::dbus::MessageType::Error);
    assert_eq!(service.application_id, 0);
}

#[test]
fn application_service_has_no_terminal_child_when_started_by_console() {
    let mut service = ServiceState::new(snapshot(), false, Vec::new());
    let children = service.dispatch(
        1,
        &call_at(ROOT_PATH, ACCESSIBLE, "GetChildren", "", Vec::new()),
    );
    assert_eq!(children.header.signature, "a(so)");
    let mut children_body = children.body_reader();
    assert_eq!(children_body.u32().unwrap(), 0);
    let terminal = service.dispatch(2, &call(ACCESSIBLE, "GetRole", "", Vec::new()));
    assert_eq!(terminal.kind, super::dbus::MessageType::Error);
}

#[test]
fn global_preferences_nodes_are_live_atspi_children_with_value_and_availability() {
    let nodes = vec![
        GlobalPreferencesAccessibleNode {
            id: "global-preferences".into(),
            name: "Global Preferences".into(),
            role: GlobalPreferencesAccessibleRole::Dialog,
            value: Some("Global · this device".into()),
            description: "Application preferences for this device.".into(),
            available: true,
            focused: false,
        },
        GlobalPreferencesAccessibleNode {
            id: "datum.accessibility.reduced_motion-control".into(),
            name: "Reduced motion".into(),
            role: GlobalPreferencesAccessibleRole::Switch,
            value: Some("Off".into()),
            description: "Global · this device. Factory default.".into(),
            available: false,
            focused: true,
        },
    ];
    let mut service = ServiceState::new(snapshot(), false, nodes);
    let count = service.dispatch(
        1,
        &call_at(
            ROOT_PATH,
            PROPERTIES,
            "Get",
            "ss",
            two_strings(ACCESSIBLE, "ChildCount"),
        ),
    );
    assert_eq!(count.body_reader().variant_i32().unwrap(), 1);

    let control_path = format!("{PREFERENCES_PATH}/1");
    let name = service.dispatch(
        2,
        &call_at(
            &control_path,
            PROPERTIES,
            "Get",
            "ss",
            two_strings(ACCESSIBLE, "Name"),
        ),
    );
    assert_eq!(
        name.body_reader().variant_string().unwrap(),
        "Reduced motion"
    );
    let description = service.dispatch(
        3,
        &call_at(
            &control_path,
            PROPERTIES,
            "Get",
            "ss",
            two_strings(ACCESSIBLE, "Description"),
        ),
    );
    assert!(
        description
            .body_reader()
            .variant_string()
            .unwrap()
            .contains("Current value: Off")
    );
    let role = service.dispatch(
        4,
        &call_at(&control_path, ACCESSIBLE, "GetRoleName", "", Vec::new()),
    );
    assert_eq!(role.body_reader().string().unwrap(), "toggle button");
    let state = service.dispatch(
        5,
        &call_at(&control_path, ACCESSIBLE, "GetState", "", Vec::new()),
    );
    let words = state.body_reader().u32_array().unwrap();
    assert_eq!(words[0] & (1 << 8), 0, "unavailable is not enabled");
    assert_ne!(words[0] & (1 << 12), 0, "focused state is exposed");
}

fn call_at(path: &str, interface: &str, member: &str, signature: &str, body: Vec<u8>) -> Message {
    let mut call = Message::method_call(9, "ignored", path, interface, member, signature, body);
    call.header.sender = Some(":1.55".into());
    call
}

fn i32_arg(value: i32) -> Vec<u8> {
    let mut body = BodyWriter::new();
    body.i32(value);
    body.finish()
}

fn u32_arg(value: u32) -> Vec<u8> {
    let mut body = BodyWriter::new();
    body.u32(value);
    body.finish()
}

fn two_strings(first: &str, second: &str) -> Vec<u8> {
    let mut body = BodyWriter::new();
    body.string(first);
    body.string(second);
    body.finish()
}
