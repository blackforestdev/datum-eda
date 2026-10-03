use super::*;
use crate::console_accessibility::AnnouncementPriority;
use crate::terminal_accessibility::TerminalAccessibilityBounds;

fn snapshot(text: &str) -> Arc<TerminalAccessibilitySnapshot> {
    Arc::new(TerminalAccessibilitySnapshot {
        session_id: "s".into(),
        title: "Terminal".into(),
        text: text.into(),
        caret: text.chars().count(),
        selection: None,
        links: Vec::new(),
        focused: true,
        bell_count: 0,
        bounds: TerminalAccessibilityBounds::default(),
    })
}

use crate::terminal_accessibility_platform::selection_fixture as fixture;

#[test]
fn selection_accessibility_publication_preserves_all_siblings_and_latest_basis() {
    use datum_gui_protocol::{
        CompoundSelection, SelectionSubject, selection_resolution::NativeSelectionResolution,
        selection_view::build_selection_view,
    };
    let mut f = fixture::Fixture::new("accessibility-coalescing");
    f.line(
        fixture::id(1),
        eda_engine::ir::geometry::Point::new(0, 0),
        eda_engine::ir::geometry::Point::new(10, 0),
    );
    let authority = NativeSelectionResolution::capture(&f.model).unwrap();
    let members = authority
        .authored_identities()
        .iter()
        .filter(|id| id.class == datum_gui_protocol::AuthoredSelectionClass::Track)
        .cloned()
        .collect::<Vec<_>>();
    let scene = datum_gui_protocol::load_fixture_workspace_state().scene;
    let view = build_selection_view(
        &f.model,
        &authority,
        SelectionSubject::Compound(CompoundSelection::new(members, None).unwrap()),
        &scene,
        None,
        |_| true,
    )
    .unwrap();
    let first = SelectionAccessibilitySnapshot {
        view: Arc::new(view),
        focused: true,
        pane: Some(datum_gui_protocol::PaneId(0)),
    };
    let (wake, _reader) = UnixStream::pair().unwrap();
    let mut bridge = PlatformBridge {
        shared: Arc::new(Mutex::new(Shared {
            pending: None,
            latest_snapshot: snapshot("terminal"),
            terminal_available: true,
            latest_preferences: Vec::new(),
            latest_menus: Vec::new(),
            latest_selection: None,
        })),
        wake,
    };
    bridge.publish_selection(Some(first.clone()));
    let menus = datum_gui_protocol::gui_menu_model::accessibility::menu_accessibility_nodes(
        &datum_gui_protocol::load_default_gui_menu_model().unwrap(),
        &{
            let mut s = datum_gui_protocol::load_fixture_workspace_state();
            s.ui.active_menu = Some("View".into());
            s
        },
    );
    bridge.publish_menus(menus.clone());
    bridge.publish(
        snapshot("updated terminal"),
        vec![TerminalAccessibilityEvent::TextChanged],
    );
    bridge.publish_preferences(Vec::new());
    bridge.publish_announcement(AccessibilityAnnouncement {
        text: "Derived membership change refused".into(),
        priority: AnnouncementPriority::Medium,
    });
    let pending = take_update(&bridge.shared).unwrap();
    assert_eq!(pending.selection, Some(first.clone()));
    assert_eq!(pending.snapshot.text, "updated terminal");
    assert_eq!(pending.menus, menus);
    assert_eq!(pending.announcements.len(), 1);
    // The next sibling update constructs a fresh pending packet and must
    // retain the latest selection even without a new selection publication.
    bridge.publish(snapshot("another terminal basis"), Vec::new());
    assert_eq!(take_update(&bridge.shared).unwrap().selection, Some(first));
    bridge.publish_selection(None);
    bridge.publish_menus(Vec::new());
    assert!(take_update(&bridge.shared).unwrap().selection.is_none());
}

#[test]
fn pending_updates_replace_snapshot_and_coalesce_event_kinds() {
    let shared = Mutex::new(Shared {
        pending: Some(PendingUpdate {
            snapshot: snapshot("a"),
            events: vec![TerminalAccessibilityEvent::TextChanged],
            terminal_available: true,
            announcements: VecDeque::new(),
            preferences: Vec::new(),
            menus: Vec::new(),
            selection: None,
        }),
        latest_snapshot: snapshot("a"),
        terminal_available: true,
        latest_preferences: Vec::new(),
        latest_menus: Vec::new(),
        latest_selection: None,
    });
    {
        let mut state = shared.lock().unwrap();
        let pending = state.pending.as_mut().unwrap();
        pending.snapshot = snapshot("abc");
        if !pending
            .events
            .contains(&TerminalAccessibilityEvent::CaretMoved)
        {
            pending.events.push(TerminalAccessibilityEvent::CaretMoved);
        }
    }
    let pending = take_update(&shared).unwrap();
    assert_eq!(pending.snapshot.text, "abc");
    assert_eq!(pending.events.len(), 2);
    assert!(take_update(&shared).is_none());
}

#[test]
fn pending_announcements_preserve_bounded_fifo_payload_order() {
    let mut pending = VecDeque::new();
    for index in 0..ANNOUNCEMENT_CAPACITY + 3 {
        push_bounded_announcement(
            &mut pending,
            AccessibilityAnnouncement {
                text: format!("announcement {index}"),
                priority: AnnouncementPriority::Medium,
            },
        );
    }
    assert_eq!(pending.len(), ANNOUNCEMENT_CAPACITY);
    assert_eq!(pending.front().unwrap().text, "announcement 3");
    assert_eq!(
        pending.back().unwrap().text,
        format!("announcement {}", ANNOUNCEMENT_CAPACITY + 2)
    );
}

#[test]
fn menu_publication_survives_interleaved_terminal_preferences_and_announcements() {
    let (wake, _reader) = UnixStream::pair().unwrap();
    let mut bridge = PlatformBridge {
        shared: Arc::new(Mutex::new(Shared {
            pending: None,
            latest_snapshot: snapshot("terminal"),
            terminal_available: true,
            latest_preferences: Vec::new(),
            latest_menus: Vec::new(),
            latest_selection: None,
        })),
        wake,
    };
    let mut workspace = datum_gui_protocol::load_fixture_workspace_state();
    workspace.ui.active_menu = Some("View".to_owned());
    let menus = datum_gui_protocol::gui_menu_model::accessibility::menu_accessibility_nodes(
        &datum_gui_protocol::load_default_gui_menu_model().unwrap(),
        &workspace,
    );
    let preferences = vec![GlobalPreferencesAccessibleNode {
        id: "global-preferences".into(),
        name: "Global Preferences".into(),
        role: datum_gui_protocol::GlobalPreferencesAccessibleRole::Dialog,
        value: None,
        description: "Retained preferences consumer".into(),
        available: true,
        focused: false,
    }];
    bridge.publish_menus(menus.clone());
    let latest = snapshot("latest terminal");
    let retired = Arc::downgrade(&latest);
    bridge.publish(
        Arc::clone(&latest),
        vec![TerminalAccessibilityEvent::TextChanged],
    );
    bridge.publish_preferences(preferences.clone());
    bridge.publish_announcement(AccessibilityAnnouncement {
        text: "preserved announcement".into(),
        priority: AnnouncementPriority::Medium,
    });
    let pending = take_update(&bridge.shared).unwrap();
    assert_eq!(pending.menus, menus);
    assert_eq!(pending.preferences, preferences);
    assert_eq!(pending.snapshot.text, "latest terminal");
    assert!(Arc::ptr_eq(&pending.snapshot, &latest));
    assert!(Arc::ptr_eq(
        &bridge.shared.lock().unwrap().latest_snapshot,
        &latest
    ));
    let service = ServiceState::new(Arc::clone(&pending.snapshot), true, Vec::new());
    assert!(Arc::ptr_eq(&service.snapshot, &latest));
    drop(latest);
    assert!(pending.terminal_available);
    assert_eq!(pending.announcements.len(), 1);
    // The next update after the pending batch was consumed must also retain
    // the independently cached menu projection.
    bridge.publish(snapshot("next terminal"), Vec::new());
    assert_eq!(take_update(&bridge.shared).unwrap().menus, menus);
    // A consumed update and service may still read the previous generation.
    assert_eq!(service.snapshot.text, "latest terminal");
    drop(pending);
    assert!(retired.upgrade().is_some());
    drop(service);
    assert!(retired.upgrade().is_none());
    bridge.publish_menus(Vec::new());
    let pending = take_update(&bridge.shared).unwrap();
    assert!(pending.menus.is_empty());
    assert_eq!(pending.preferences, preferences);
    assert_eq!(pending.snapshot.text, "next terminal");
}

#[test]
#[ignore = "requires a live Linux accessibility bus"]
fn real_accessibility_bus_accepts_datum_registration() {
    let (connection, service) = connect(
        &snapshot("Datum accessibility probe"),
        true,
        Vec::new(),
        Vec::new(),
    )
    .unwrap();
    assert!(connection.unique_name().starts_with(':'));
    assert_eq!(service.bus_name, connection.unique_name());
    assert!(!service.registry_parent.0.is_empty());
    assert_eq!(service.registry_parent.1, ROOT_PATH);
}
