//! Count actual engine acquisitions across ordinary selection menu and view consumption.
use crate::native_selection_fixture as fixture;

use super::*;
use crate::{
    PointNm,
    selection_menu::{self, Command, SelectionMenu},
    selection_view::build_selection_view,
};
use eda_engine::substrate::ZoneFillState;
use fixture::{Fixture, id, rectangle};

fn acquisitions(authority: &NativeSelectionResolution) -> usize {
    authority.run_query.acquisitions()
}

fn two_islands(name: &str) -> (Fixture, eda_engine::board::Zone, AuthoredSelectionIdentity) {
    let mut fixture = Fixture::new(name);
    let zone = fixture.zone(id(10));
    fixture.fill(
        &zone,
        ZoneFillState::Filled,
        vec![rectangle(0, 0, 4, 4), rectangle(20, 0, 4, 4)],
    );
    let origin = AuthoredSelectionIdentity {
        class: Class::Zone,
        id: zone.uuid,
        instance_path: vec![],
    };
    (fixture, zone, origin)
}

#[test]
fn menu_and_view_preserve_same_hit_run_acquisition() {
    let (fixture, _, origin) = two_islands("menu-run-reuse");
    let before = fixture.source_bytes();
    let authority = NativeSelectionResolution::capture(&fixture.model).unwrap();
    let workspace = crate::load_fixture_workspace_state();
    let menu = SelectionMenu::capture(
        &authority,
        vec![(origin.clone(), origin)],
        PointNm { x: 1, y: 1 },
        workspace.ui.filters,
    );
    let item = menu
        .items
        .iter()
        .find(|item| matches!(item.command, Command::Run { .. }))
        .unwrap();
    assert!(item.unavailable.is_none(), "{:?}", item.unavailable);
    assert_eq!(acquisitions(&authority), 1);

    for _ in 0..3 {
        // The menu's real command path acquires and resolves; the view resolves again.
        // Output equality alone cannot detect a costly recomputation here.
        let subject = selection_menu::resolve(&authority, &item.command)
            .unwrap()
            .unwrap();
        let view = build_selection_view(
            &fixture.model,
            &authority,
            subject.clone(),
            &workspace.scene,
            None,
            |_| true,
        )
        .unwrap();
        assert_eq!(view.subject, subject);
        assert_eq!(view.members.len(), 1);
        assert_eq!(
            acquisitions(&authority),
            1,
            "menu/view consumption must not trigger another board_run acquisition"
        );
    }
    Fixture::assert_bytes(&before);
}

#[test]
fn different_hits_and_new_authority_require_fresh_acquisition() {
    let (mut fixture, zone, origin) = two_islands("run-reuse-invalidation");
    let authority = NativeSelectionResolution::capture(&fixture.model).unwrap();
    let first = authority
        .acquire_run(origin.clone(), Some(Point::new(1, 1)))
        .unwrap();
    authority.resolve(&first).unwrap();
    let other = authority
        .acquire_run(origin.clone(), Some(Point::new(21, 1)))
        .unwrap();
    authority.resolve(&other).unwrap();
    assert_ne!(first, other, "the second hit must select the other island");
    assert_eq!(acquisitions(&authority), 2);
    assert_eq!(
        authority
            .acquire_run(origin.clone(), Some(Point::new(1, 1)))
            .unwrap(),
        first
    );
    assert_eq!(
        acquisitions(&authority),
        3,
        "the displaced hit is recomputed"
    );
    assert!(authority.acquire_run(origin.clone(), None).is_err());
    assert_eq!(
        acquisitions(&authority),
        4,
        "no inferred region for a hitless query"
    );

    // Authored mutation advances the model revision; a derived fill alone does not.
    fixture.line(id(11), Point::new(200, 0), Point::new(210, 0));
    fixture.fill(
        &zone,
        ZoneFillState::Filled,
        vec![rectangle(0, 0, 6, 6), rectangle(20, 0, 4, 4)],
    );
    let next = NativeSelectionResolution::capture(&fixture.model).unwrap();
    assert_ne!(authority.revision(), next.revision());
    assert_eq!(acquisitions(&next), 0);
    let next_run = next
        .acquire_run(origin.clone(), Some(Point::new(1, 1)))
        .unwrap();
    next.resolve(&next_run).unwrap();
    assert_eq!(
        next.acquire_run(origin, Some(Point::new(1, 1))).unwrap(),
        next_run
    );
    assert_eq!(
        acquisitions(&next),
        1,
        "a new source basis starts its own cache"
    );
    assert_eq!(acquisitions(&authority), 4);
}
