//! Complete native menu inventory and read-only command authority.
#[path = "support/native_selection_fixture.rs"]
mod fixture;
use datum_gui_protocol::selection_resolution::SelectionAuthority;
use datum_gui_protocol::{
    AuthoredSelectionClass as C, AuthoredSelectionIdentity as Id, PointNm, SelectionSubject,
    selection_menu::{Command, SelectionMenu, resolve},
    selection_resolution::NativeSelectionResolution,
};
use eda_engine::{board::Track, ir::geometry::Point, substrate::Operation};
use fixture::{Fixture, id};
#[test]
fn native_select_menu_keeps_complete_inventory_and_refuses_unbound_semantics_without_mutation() {
    let mut f = Fixture::new("native-select-menu");
    let net = f.net;
    f.commit(
        (1..=300)
            .map(|n| {
                let x = n as i64 * 100;
                let track =
                    Track::straight(id(n), net, Point::new(x, 0), Point::new(x + 10, 0), 2, 1);
                Operation::CreateBoardTrack {
                    track_id: track.uuid,
                    track: serde_json::to_value(track).unwrap(),
                }
            })
            .collect::<Vec<_>>(),
    );
    let before = f.source_bytes();
    let authority = NativeSelectionResolution::capture(&f.model).unwrap();
    let origins: Vec<_> = (1..=300)
        .rev()
        .map(|n| {
            let v = Id {
                class: C::Track,
                id: id(n),
                instance_path: vec![],
            };
            (v.clone(), v)
        })
        .collect();
    let filters = datum_gui_protocol::load_fixture_workspace_state()
        .ui
        .filters;
    let menu = SelectionMenu::capture(
        &authority,
        origins.clone(),
        PointNm { x: 30_000, y: 0 },
        filters,
    );
    assert_eq!(menu.items.len(), 303);
    assert_eq!(
        menu.items[299].command,
        Command::Object(origins[299].0.clone())
    );
    assert_eq!(
        resolve(&authority, &menu.items[299].command).unwrap(),
        Some(SelectionSubject::Object(origins[299].0.clone()))
    );
    assert!(matches!(
        resolve(&authority, &menu.items[300].command).unwrap(),
        Some(SelectionSubject::Run(_))
    ));
    assert!(
        menu.items[301]
            .unavailable
            .as_ref()
            .unwrap()
            .contains("UnavailableAssignment")
    );
    assert!(resolve(&authority, &menu.items[301].command).is_err());
    assert_eq!(resolve(&authority, &menu.items[302].command).unwrap(), None);
    let mut workspace = datum_gui_protocol::load_fixture_workspace_state();
    let mut open = menu.clone();
    open.open = true;
    open.focus = 299;
    workspace.ui.focus = datum_gui_protocol::ApplicationFocus::Overlay;
    workspace.ui.marking_menu = Some(datum_gui_protocol::MarkingMenuState {
        menu_key: "pcb.track".into(),
        target_object_id: None,
        anchor_x_px: 0,
        anchor_y_px: 0,
        preview_slot: None,
        gesture_dx_px: 0,
        gesture_dy_px: 0,
        selection: Some(open),
    });
    let nodes = datum_gui_protocol::gui_menu_model::accessibility::menu_accessibility_nodes(
        datum_gui_protocol::gui_menu_model::default_gui_menu_model().unwrap(),
        &workspace,
    );
    assert_eq!(nodes.len(), 304);
    assert!(nodes[300].focused);
    assert!(nodes[300].id.contains(&id(1).to_string()));
    assert!(!nodes[302].available);
    assert!(nodes[302].description.contains("UnavailableAssignment"));
    assert_eq!(
        nodes
            .iter()
            .map(|n| &n.id)
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        304
    );
    let subject = SelectionSubject::Compound(
        datum_gui_protocol::CompoundSelection::new(
            origins.iter().map(|(id, _)| id.clone()),
            Some(origins[0].0.clone()),
        )
        .unwrap(),
    );
    let view = datum_gui_protocol::selection_view::build_selection_view(
        &f.model,
        &authority,
        subject.clone(),
        &workspace.scene,
        None,
        |_| true,
    )
    .unwrap();
    let mut inspected = menu.clone();
    inspected.append_inspection(&view);
    assert_eq!(inspected.inspection_subject, Some(subject));
    assert!(
        inspected
            .items
            .iter()
            .any(|item| item.command == Command::InspectScope(Some(C::Track)))
    );
    assert!(
        inspected
            .items
            .iter()
            .any(|item| item.command == Command::InspectScope(None))
    );
    assert_eq!(
        inspected
            .items
            .iter()
            .filter(|item| matches!(item.command, Command::FilterToggle(_)))
            .count(),
        18
    );
    assert_eq!(view.members.len(), 300);
    Fixture::assert_bytes(&before);
    f.commit([Operation::DeleteBoardTrack {
        track_id: id(1),
        track: serde_json::to_value(Track::straight(
            id(1),
            net,
            Point::new(100, 0),
            Point::new(110, 0),
            2,
            1,
        ))
        .unwrap(),
    }]);
    let next = NativeSelectionResolution::capture(&f.model).unwrap();
    assert!(resolve(&next, &menu.items[299].command).is_err());
    assert_ne!(menu.revision, next.revision().0);
}

#[test]
fn explicit_checks_entries_keep_complete_fingerprints_and_refuse_ambiguous_or_foreign_producers() {
    let f = Fixture::new("checks-menu");
    let authority = NativeSelectionResolution::capture(&f.model).unwrap();
    let state = datum_gui_protocol::load_fixture_workspace_state();
    let mut menu = SelectionMenu::capture(
        &authority,
        vec![],
        datum_gui_protocol::PointNm { x: 0, y: 0 },
        state.ui.filters,
    );
    let mut checks = datum_gui_protocol::CheckRunReviewState {
        project_id: Some(authority.project().to_string()),
        findings: (0..300)
            .map(|i| datum_gui_protocol::CheckFindingSummary {
                fingerprint: format!("sha256:finding-{i}"),
                severity: "warning".into(),
                rule_id: "R".into(),
                ..Default::default()
            })
            .collect(),
        ..Default::default()
    };
    menu.append_checks(&checks);
    assert_eq!(menu.items.len(), 301);
    assert_eq!(
        menu.items[300].command,
        Command::Diagnostic("sha256:finding-299".into())
    );
    assert!(menu.items.iter().all(|item| item.unavailable.is_none()));
    checks.findings.push(checks.findings[0].clone());
    let mut ambiguous = menu.clone();
    ambiguous.items.clear();
    ambiguous.append_checks(&checks);
    assert!(ambiguous.items[0].unavailable.is_some());
    assert!(ambiguous.items[300].unavailable.is_some());
    checks.project_id = Some(uuid::Uuid::new_v4().to_string());
    let mut foreign = menu.clone();
    foreign.items.clear();
    foreign.append_checks(&checks);
    assert!(foreign.items.is_empty());
}
