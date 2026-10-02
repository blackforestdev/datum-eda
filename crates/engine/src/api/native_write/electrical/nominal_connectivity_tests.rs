//! Native nominal graph/DRC integration with canonical edits and persistence.
use super::current_fill_drc_tests::native_board;
use super::pad_connection_tests::{pad, write_pad};
use super::*;
use crate::board::PadLayerConnection;
use crate::drc::{DrcReport, run_with_current_zone_fills_and_waivers};
use crate::rules::ast::RuleType;
fn report(model: &DesignModel) -> DrcReport {
    run_with_current_zone_fills_and_waivers(
        &native_board(model),
        &[RuleType::Connectivity],
        model,
        &[],
    )
}
fn codes(report: &DrcReport) -> BTreeSet<&str> {
    report.violations.iter().map(|v| v.code.as_str()).collect()
}
fn arc_fixture(name: &str) -> (PathBuf, DesignModel, Uuid, Track) {
    let (root, mut model, net) = fixture(name);
    let mut first = pad(net, PadLayerConnection::Separate);
    first.position = Point::new(-10, 0);
    first.diameter = 2;
    first.drill = 0;
    first.copper_layers = vec![8];
    let mut second = first.clone();
    second.uuid = Uuid::new_v4();
    second.position = Point::new(0, 10);
    write_pad(&mut model, &root, &first, true).unwrap();
    write_pad(&mut model, &root, &second, true).unwrap();
    let arc = Track {
        midpoint: Some(Point::new(0, 10)),
        ..line(net, (-10, 0), (10, 0))
    };
    tracks(&mut model, &root, std::slice::from_ref(&arc));
    (root, model, net, arc)
}
#[test]
fn native_connectivity_uses_arc_bulge_not_chord_and_preserves_fingerprints_on_undo_reopen() {
    let (root, mut model, net, mut arc) = arc_fixture("native_nominal_connectivity_arc");
    let before = model.clone();
    let board_bytes = std::fs::read(root.join("board/board.json")).unwrap();
    let journal_bytes = std::fs::read(crate::substrate::transaction_journal_path(&root)).unwrap();
    assert!(report(&model).passed);
    assert_eq!(model.objects, before.objects);
    assert_eq!(model.model_revision, before.model_revision);
    assert_eq!(
        std::fs::read(root.join("board/board.json")).unwrap(),
        board_bytes
    );
    assert_eq!(
        std::fs::read(crate::substrate::transaction_journal_path(&root)).unwrap(),
        journal_bytes
    );
    arc.midpoint = None;
    let write = build_set_board_track(&model, provenance(), &arc).unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    let disconnected = report(&model);
    assert_eq!(
        codes(&disconnected),
        BTreeSet::from(["connectivity_unrouted_net"])
    );
    assert_eq!(disconnected.violations[0].objects, vec![net]);
    assert!(disconnected.violations[0].fingerprint.is_some());
    let waiver:crate::schematic::CheckWaiver=serde_json::from_value(serde_json::json!({
        "uuid":Uuid::new_v4(),"domain":"DRC","target":{"Fingerprint":disconnected.violations[0].fingerprint.clone().unwrap()},"rationale":"native connectivity proof waiver"
    })).unwrap();
    let waived = run_with_current_zone_fills_and_waivers(
        &native_board(&model),
        &[RuleType::Connectivity],
        &model,
        &[waiver],
    );
    assert!(waived.passed);
    assert_eq!(waived.summary.waived, 1);
    model
        .commit_journal_undo(&root, provenance().into())
        .unwrap();
    assert!(report(&model).passed);
    model
        .commit_journal_redo(&root, provenance().into())
        .unwrap();
    let reopened = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(report(&reopened), disconnected);
    let mut wrong = native_board(&reopened);
    wrong.tracks.get_mut(&arc.uuid).unwrap().width += 1;
    let result =
        run_with_current_zone_fills_and_waivers(&wrong, &[RuleType::Connectivity], &reopened, &[]);
    assert_eq!(
        codes(&result),
        BTreeSet::from(["nominal_connectivity_unavailable"])
    );
    std::fs::remove_dir_all(root).unwrap();
}
#[test]
fn native_connectivity_uses_current_occupied_fill_holes_empty_and_stale_states() {
    let (root, mut model, net) = fixture("native_nominal_connectivity_fill");
    let mut left = pad(net, PadLayerConnection::Separate);
    left.position = Point::new(2, 5);
    left.diameter = 2;
    left.drill = 0;
    left.copper_layers = vec![8];
    let mut right = left.clone();
    right.uuid = Uuid::new_v4();
    right.position = Point::new(28, 5);
    write_pad(&mut model, &root, &left, true).unwrap();
    write_pad(&mut model, &root, &right, true).unwrap();
    let z = zone(&mut model, &root, net);
    let missing = report(&model);
    assert_eq!(
        codes(&missing),
        BTreeSet::from(["nominal_connectivity_unavailable"])
    );
    fill(
        &mut model,
        &root,
        &z,
        ZoneFillState::Filled,
        vec![rectangle(0, 30)],
    );
    assert!(report(&model).passed);
    fill(
        &mut model,
        &root,
        &z,
        ZoneFillState::Filled,
        vec![rectangle(0, 10), rectangle(20, 10)],
    );
    assert_eq!(
        codes(&report(&model)),
        BTreeSet::from(["connectivity_unrouted_net"])
    );
    fill(&mut model, &root, &z, ZoneFillState::Filled, vec![]);
    assert_eq!(
        codes(&report(&model)),
        BTreeSet::from(["connectivity_no_copper", "connectivity_unrouted_net"])
    );
    fill(
        &mut model,
        &root,
        &z,
        ZoneFillState::Stale,
        vec![rectangle(0, 30)],
    );
    let stale = report(&model);
    assert_eq!(
        codes(&stale),
        BTreeSet::from(["nominal_connectivity_unavailable"])
    );
    assert!(stale.violations[0].objects.contains(&z.uuid));
    std::fs::remove_dir_all(root).unwrap();
}
#[test]
fn native_connectivity_detects_foreign_nominal_contacts_without_merging_net_identity() {
    let (root, mut model, net, arc) = arc_fixture("native_nominal_connectivity_foreign");
    let other = Net::new(Uuid::new_v4(), "actual assignment", Uuid::nil());
    let write = build_place_board_net(&model, provenance(), &other).unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    let crossing = line(other.uuid, (0, 8), (0, 20));
    tracks(&mut model, &root, std::slice::from_ref(&crossing));
    let run = query(&model, &member("tracks", arc.uuid), None).unwrap();
    assert_eq!(run.board_net, net);
    assert!(!run.members.contains(&member("tracks", crossing.uuid)));
    let result = report(&model);
    assert_eq!(
        codes(&result),
        BTreeSet::from(["connectivity_cross_net_contact"])
    );
    assert_eq!(
        result.violations.len(),
        2,
        "two distinct source pairs, each emitted once across both Net owners"
    );
    let terminal = native_board(&model)
        .pads
        .values()
        .find(|p| p.position == Point::new(0, 10))
        .unwrap()
        .uuid;
    let mut arc_pair = vec![arc.uuid, crossing.uuid];
    arc_pair.sort();
    let mut pad_pair = vec![terminal, crossing.uuid];
    pad_pair.sort();
    assert_eq!(
        result
            .violations
            .iter()
            .map(|v| v.objects.clone())
            .collect::<BTreeSet<_>>(),
        BTreeSet::from([arc_pair, pad_pair])
    );
    let before = model.clone();
    let mut invalid = crossing.clone();
    invalid.net = Uuid::new_v4();
    let write = build_set_board_track(&model, provenance(), &invalid).unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    assert!(codes(&report(&model)).contains("nominal_connectivity_unavailable"));
    assert_eq!(before.electrical_identities, model.electrical_identities);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn native_connectivity_requires_actual_plating_and_does_not_join_separate_apertures_by_uuid() {
    let (root, mut model, net) = fixture("native_connectivity_plating");
    let mut center = pad(net, PadLayerConnection::Separate);
    write_pad(&mut model, &root, &center, true).unwrap();
    let mut front = center.clone();
    front.uuid = Uuid::new_v4();
    front.copper_layers = vec![8];
    front.position = Point::new(20, 0);
    front.diameter = 2;
    front.drill = 0;
    let mut back = front.clone();
    back.uuid = Uuid::new_v4();
    back.copper_layers = vec![9];
    back.layer = 9;
    back.position = Point::new(0, 20);
    write_pad(&mut model, &root, &front, true).unwrap();
    write_pad(&mut model, &root, &back, true).unwrap();
    tracks(
        &mut model,
        &root,
        &[
            line(net, (3, 0), (20, 0)),
            Track {
                layer: 9,
                ..line(net, (3, 0), (0, 20))
            },
        ],
    );
    assert_eq!(
        codes(&report(&model)),
        BTreeSet::from(["connectivity_unrouted_net"])
    );
    center.layer_connection = PadLayerConnection::PlatedThrough;
    write_pad(&mut model, &root, &center, false).unwrap();
    assert!(report(&model).passed);
    model
        .commit_journal_undo(&root, provenance().into())
        .unwrap();
    assert_eq!(
        codes(&report(&model)),
        BTreeSet::from(["connectivity_unrouted_net"])
    );
    model
        .commit_journal_redo(&root, provenance().into())
        .unwrap();
    assert!(report(&ProjectResolver::new(&root).resolve().unwrap()).passed);
    center.layer_connection = PadLayerConnection::Unknown;
    write_pad(&mut model, &root, &center, false).unwrap();
    assert!(codes(&report(&model)).contains("nominal_connectivity_unavailable"));
    std::fs::remove_dir_all(root).unwrap();
}
