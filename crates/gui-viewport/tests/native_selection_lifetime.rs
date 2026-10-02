//! Engine-issued Zone qualifications consumed by the one shared lifetime owner.
#[path = "../../gui-protocol/tests/support/native_selection_fixture.rs"]
mod fixture;
use datum_gui_protocol::selection_resolution::{
    NativeSelectionResolution as Native, SelectionAuthority, ZoneClearReason,
};
use datum_gui_protocol::{
    AuthoredSelectionClass as Class, AuthoredSelectionIdentity as Identity,
    SelectionSubject as Subject,
};
use datum_gui_viewport::selection_lifetime::{
    SelectionReconciliationState as State, reconcile_native_selection,
};
use eda_engine::{
    ir::geometry::Point,
    substrate::{Operation, ZoneFillState},
};
use fixture::*;
fn origin() -> Identity {
    Identity {
        class: Class::Zone,
        id: id(1),
        instance_path: vec![],
    }
}

#[test]
fn native_zone_unique_region_rederives_and_qualified_identity_round_trips() {
    let mut f = Fixture::new("native-zone-lifetime-unique");
    let zone = f.zone(id(1));
    let fill = rectangle(0, 0, 10, 10);
    f.fill(&zone, ZoneFillState::Filled, vec![fill]);
    let previous = Native::capture(&f.model).unwrap();
    let selected = previous
        .acquire_run(origin(), Some(Point::new(5, 5)))
        .unwrap();
    let encoded = serde_json::to_vec(&selected).unwrap();
    assert_eq!(
        serde_json::from_slice::<Subject>(&encoded).unwrap(),
        selected
    );
    f.line(id(2), Point::new(10, 5), Point::new(20, 5));
    f.fill(
        &zone,
        ZoneFillState::Filled,
        vec![rectangle(0, 0, 5, 10), rectangle(5, 0, 5, 10)],
    );
    let current = Native::capture(&f.model).unwrap();
    let files = f.source_bytes();
    let result =
        reconcile_native_selection(&selected, current.project(), &previous, &current).unwrap();
    assert!(!result.dissolved);
    assert_eq!(result.state, State::Current);
    assert_eq!(result.members.len(), 2);
    let Subject::Run(run) = &result.subject else {
        panic!()
    };
    assert_eq!(run.origin, origin());
    assert_eq!(
        run.zone_region.as_ref().unwrap().revision(),
        current.revision()
    );
    assert!(
        current
            .resolve(&result.subject)
            .unwrap()
            .board_run
            .unwrap()
            .zone_copper
            .len()
            == 1
    );
    Fixture::assert_bytes(&files);
    let reopened = Native::capture(&f.reopen()).unwrap();
    assert_eq!(reopened.members(&result.subject).unwrap(), result.members);
}

#[test]
fn native_zone_suspend_then_current_empty_clears_without_undo_resurrection() {
    let mut f = Fixture::new("native-zone-lifetime-empty");
    let zone = f.zone(id(1));
    f.fill(&zone, ZoneFillState::Filled, vec![rectangle(0, 0, 10, 10)]);
    let certified = Native::capture(&f.model).unwrap();
    let selected = certified
        .acquire_run(origin(), Some(Point::new(5, 5)))
        .unwrap();
    f.fill(&zone, ZoneFillState::Unsupported, vec![]);
    let unavailable = Native::capture(&f.model).unwrap();
    let suspended =
        reconcile_native_selection(&selected, unavailable.project(), &certified, &unavailable)
            .unwrap();
    assert_eq!(suspended.subject, selected);
    assert!(!suspended.dissolved);
    assert!(matches!(suspended.state, State::Suspended { .. }));
    assert!(suspended.state.explanation().unwrap().contains("stale"));
    f.fill(&zone, ZoneFillState::Filled, vec![]);
    let empty = Native::capture(&f.model).unwrap();
    // Retain the last certified basis while suspended, never pretend the stale
    // intermediate epoch validated the engine's older qualifier.
    let cleared =
        reconcile_native_selection(&suspended.subject, empty.project(), &certified, &empty)
            .unwrap();
    assert_eq!(cleared.subject, Subject::None);
    assert!(cleared.dissolved);
    assert_eq!(
        cleared.state,
        State::RegionCleared(ZoneClearReason::CurrentEmpty)
    );
    assert!(cleared.state.explanation().unwrap().contains("no copper"));
    f.undo();
    let undone = Native::capture(&f.reopen()).unwrap();
    let result =
        reconcile_native_selection(&cleared.subject, undone.project(), &empty, &undone).unwrap();
    assert_eq!(result.subject, Subject::None);
    assert!(result.members.is_empty());
}

#[test]
fn native_zone_split_unknown_and_source_deletion_disclose_distinct_clear_reasons() {
    for case in ["split", "unknown", "deleted"] {
        let mut f = Fixture::new(case);
        let zone = f.zone(id(1));
        f.line(id(2), Point::new(5, 5), Point::new(25, 5));
        f.fill(
            &zone,
            ZoneFillState::Filled,
            vec![rectangle(0, 0, 10, 10), rectangle(20, 0, 10, 10)],
        );
        let previous = Native::capture(&f.model).unwrap();
        let selected = previous
            .acquire_run(origin(), Some(Point::new(5, 5)))
            .unwrap();
        match case {
            "split" => {
                f.commit([Operation::DeleteBoardTrack {
                    track_id: id(2),
                    track: serde_json::to_value(eda_engine::board::Track::straight(
                        id(2),
                        f.net,
                        Point::new(5, 5),
                        Point::new(25, 5),
                        2,
                        1,
                    ))
                    .unwrap(),
                }]);
                f.fill(
                    &zone,
                    ZoneFillState::Filled,
                    vec![rectangle(0, 0, 10, 10), rectangle(20, 0, 10, 10)],
                );
            }
            "unknown" => f.fill(&zone, ZoneFillState::Filled, vec![rectangle(40, 0, 10, 10)]),
            _ => {
                let write = eda_engine::api::native_write::board_routing::build_delete_board_zone(
                    &f.model,
                    provenance(),
                    zone.uuid,
                    serde_json::to_value(&zone).unwrap(),
                )
                .unwrap();
                eda_engine::api::native_write::commit_prepared(&mut f.model, &f.root, write)
                    .unwrap();
            }
        }
        let current = Native::capture(&f.model).unwrap();
        let files = f.source_bytes();
        let result =
            reconcile_native_selection(&selected, current.project(), &previous, &current).unwrap();
        assert!(result.dissolved);
        assert_eq!(result.subject, Subject::None);
        assert!(result.state.explanation().is_some());
        match case {
            "split" => assert!(
                matches!(result.state,State::RegionSplit {ref successors} if successors.len()==2)
            ),
            "unknown" => assert_eq!(result.state, State::RegionUnknown),
            _ => assert_eq!(result.state, State::RegionCleared(ZoneClearReason::Deleted)),
        };
        Fixture::assert_bytes(&files);
    }
}

#[test]
fn native_nonpoint_ambiguous_zone_acquisition_refuses_and_project_replacement_clears() {
    let mut f = Fixture::new("native-zone-lifetime-refusal");
    let zone = f.zone(id(1));
    f.fill(
        &zone,
        ZoneFillState::Filled,
        vec![rectangle(0, 0, 10, 10), rectangle(20, 0, 10, 10)],
    );
    let native = Native::capture(&f.model).unwrap();
    assert!(native.acquire_run(origin(), None).is_err());
    let selected = native
        .acquire_run(origin(), Some(Point::new(5, 5)))
        .unwrap();
    let cleared = reconcile_native_selection(&selected, id(999), &native, &native).unwrap();
    assert_eq!(cleared.subject, Subject::None);
    assert!(cleared.dissolved);
}
