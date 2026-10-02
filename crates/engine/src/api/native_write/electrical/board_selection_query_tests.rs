//! Native PM054 C04/C05 acquisition, source conservation and refusal oracles.
use super::super::{board_routing::*, commit_prepared, test_support::*};
use super::tests::provenance;
use super::*;
use crate::board::{Net, Stackup, StackupLayer, StackupLayerType, Track, Via, Zone};
use crate::connectivity::PhysicalQueryFailure;
use crate::ir::geometry::{Point, Polygon};
use crate::substrate::{
    ElectricalOccurrence, ElectricalQueryFailure, ElectricalSelectionSnapshot, ModelRevision,
    ProjectResolver, ZoneFill, ZoneFillState,
};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use uuid::Uuid;

fn member(class: &str, source_id: Uuid) -> ElectricalOccurrence {
    ElectricalOccurrence {
        class: class.into(),
        source_id,
        instance_path: vec![],
    }
}
fn fixture(name: &str) -> (PathBuf, DesignModel, Uuid) {
    let root = temp_project_root(name);
    write_minimal_project(&root, Uuid::new_v4(), Uuid::new_v4());
    let path = root.join("board/board.json");
    let mut value: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    // Genesis native source has explicit stackup; no inferred numeric span.
    value["stackup"] = serde_json::to_value(Stackup {
        layers: vec![
            StackupLayer::new(8, "front", StackupLayerType::Copper, 35_000),
            StackupLayer::new(7, "core", StackupLayerType::Dielectric, 1_000_000),
            StackupLayer::new(9, "back", StackupLayerType::Copper, 35_000),
        ],
    })
    .unwrap();
    value["pads"] = serde_json::json!({});
    std::fs::write(path, serde_json::to_vec_pretty(&value).unwrap()).unwrap();
    let mut model = ProjectResolver::new(&root).resolve().unwrap();
    let net = Net::new(Uuid::new_v4(), "actual assignment", Uuid::nil());
    let write = build_place_board_net(&model, provenance(), &net).unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    (root, model, net.uuid)
}
fn line(net: Uuid, from: (i64, i64), to: (i64, i64)) -> Track {
    Track::straight(
        Uuid::new_v4(),
        net,
        Point::new(from.0, from.1),
        Point::new(to.0, to.1),
        2,
        8,
    )
}
fn tracks(model: &mut DesignModel, root: &Path, tracks: &[Track]) {
    let write = BatchComposer::compose(model, provenance())
        .push_ops(tracks.iter().map(|t| Operation::CreateBoardTrack {
            track_id: t.uuid,
            track: serde_json::to_value(t).unwrap(),
        }))
        .finish()
        .unwrap();
    commit_prepared(model, root, write).unwrap();
}
fn query(
    model: &DesignModel,
    origin: &ElectricalOccurrence,
    hit: Option<Point>,
) -> Result<crate::connectivity::BoardRunMembership, ElectricalQueryFailure> {
    let snapshot = ElectricalSelectionSnapshot::capture(model).unwrap();
    snapshot.board_run(snapshot.revision(), origin, hit)
}
fn rectangle(x: i64, width: i64) -> Polygon {
    Polygon::new(vec![
        Point::new(x, 0),
        Point::new(x + width, 0),
        Point::new(x + width, 10),
        Point::new(x, 10),
    ])
}
fn zone(model: &mut DesignModel, root: &Path, net: Uuid) -> Zone {
    let zone = Zone {
        uuid: Uuid::new_v4(),
        net,
        polygon: rectangle(0, 30),
        layer: 8,
        priority: 0,
        thermal_relief: false,
        thermal_gap: 0,
        thermal_spoke_width: 0,
    };
    let write = build_place_board_zone(model, provenance(), &zone).unwrap();
    commit_prepared(model, root, write).unwrap();
    zone
}
fn fill(
    model: &mut DesignModel,
    root: &Path,
    zone: &Zone,
    state: ZoneFillState,
    islands: Vec<Polygon>,
) {
    let fill = ZoneFill {
        schema_version: 2,
        zone_id: zone.uuid,
        state,
        source_zone_revision: model.objects[&zone.uuid].object_revision,
        model_revision: model.model_revision.clone(),
        islands,
        provenance: Some(
            "PM054 native acquisition proof: current successful producer result".into(),
        ),
    };
    let write = build_set_zone_fills(model, provenance(), &[fill]).unwrap();
    commit_prepared(model, root, write).unwrap();
}

#[test]
fn complete_board_run_keeps_foreign_contacts_separate_and_disconnected_same_net_out() {
    let (root, mut model, net) = fixture("board_run_net_constraint");
    let foreign = Net::new(Uuid::new_v4(), "actual assignment", Uuid::nil());
    let write = build_place_board_net(&model, provenance(), &foreign).unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    let a = line(net, (0, 0), (10, 0));
    let b = line(net, (10, 0), (20, 0));
    let branch = line(net, (10, 0), (10, 10));
    let disconnected = line(net, (100, 0), (110, 0));
    let cross = line(foreign.uuid, (5, -5), (5, 5));
    tracks(
        &mut model,
        &root,
        &[
            a.clone(),
            b.clone(),
            branch.clone(),
            disconnected.clone(),
            cross.clone(),
        ],
    );
    let before = model.clone();
    let files: Vec<_> = model
        .source_shards
        .iter()
        .map(|s| (s.path.clone(), std::fs::read(&s.path).unwrap()))
        .collect();
    let journal = std::fs::read(crate::substrate::transaction_journal_path(&root)).unwrap();
    let result = query(&model, &member("tracks", a.uuid), None).unwrap();
    assert_eq!(
        result.members,
        BTreeSet::from([
            member("tracks", a.uuid),
            member("tracks", b.uuid),
            member("tracks", branch.uuid)
        ])
    );
    let faults = result.contact_evidence.unwrap();
    assert_eq!(faults.len(), 1);
    assert_eq!(
        (faults[0].left.source_id, faults[0].right.source_id),
        (a.uuid, cross.uuid)
    );
    assert_eq!(
        (faults[0].left_net, faults[0].right_net),
        (net, foreign.uuid)
    );
    assert_eq!(faults[0].layers, BTreeSet::from([8]));
    assert_eq!(
        query(&model, &member("tracks", disconnected.uuid), None)
            .unwrap()
            .members,
        BTreeSet::from([member("tracks", disconnected.uuid)])
    );
    assert_eq!(model, before);
    for (path, bytes) in files {
        assert_eq!(std::fs::read(path).unwrap(), bytes);
    }
    assert_eq!(
        std::fs::read(crate::substrate::transaction_journal_path(&root)).unwrap(),
        journal
    );
    let reopened = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(
        query(&reopened, &member("tracks", a.uuid), None)
            .unwrap()
            .members,
        result.members
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn board_run_uses_arc_locus_and_actual_via_span_not_chords_or_drill_centers() {
    let (root, mut model, net) = fixture("board_run_arc_span");
    let mut arc = line(net, (-10, 0), (10, 0));
    arc.midpoint = Some(Point::new(0, 10));
    let bulge = line(net, (0, 10), (0, 20));
    let chord = line(net, (0, 0), (0, -2));
    let mut back = line(net, (2, 20), (10, 20));
    back.layer = 9;
    tracks(
        &mut model,
        &root,
        &[arc.clone(), bulge.clone(), chord.clone(), back.clone()],
    );
    let via = Via {
        uuid: Uuid::new_v4(),
        net,
        position: Point::new(0, 20),
        drill: 2,
        diameter: 6,
        from_layer: 8,
        to_layer: 9,
    };
    let write = build_place_board_via(&model, provenance(), &via).unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    let result = query(&model, &member("tracks", arc.uuid), None).unwrap();
    assert_eq!(
        result.members,
        BTreeSet::from([
            member("tracks", arc.uuid),
            member("tracks", bulge.uuid),
            member("tracks", back.uuid),
            member("vias", via.uuid)
        ])
    );
    assert_eq!(
        query(&model, &member("tracks", chord.uuid), None)
            .unwrap()
            .members,
        BTreeSet::from([member("tracks", chord.uuid)])
    );
    assert!(matches!(
        query(&model, &member("vias", via.uuid), Some(via.position)),
        Err(ElectricalQueryFailure::Physical(
            PhysicalQueryFailure::OriginOutsideCopper { .. }
        ))
    ));
    // Removing the source-defined through span disconnects the back layer.
    let mut single = via.clone();
    single.to_layer = 8;
    let write = build_set_board_via(&model, provenance(), &single).unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    assert!(
        !query(&model, &member("tracks", arc.uuid), None)
            .unwrap()
            .members
            .contains(&member("tracks", back.uuid))
    );
    model
        .commit_journal_undo(&root, provenance().into())
        .unwrap();
    let reopened = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(
        query(&reopened, &member("tracks", arc.uuid), None)
            .unwrap()
            .members,
        result.members
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn zone_acquisition_requires_current_occupied_component_and_counts_source_once() {
    let (root, mut model, net) = fixture("board_run_zone_region");
    let zone = zone(&mut model, &root, net);
    let origin = member("zones", zone.uuid);
    assert!(matches!(
        query(&model, &origin, Some(Point::new(5, 5))),
        Err(ElectricalQueryFailure::Physical(
            PhysicalQueryFailure::UnavailableFill { .. }
        ))
    ));
    fill(
        &mut model,
        &root,
        &zone,
        ZoneFillState::Filled,
        vec![rectangle(0, 10), rectangle(20, 10)],
    );
    let first = query(&model, &origin, Some(Point::new(5, 5))).unwrap();
    assert_eq!(first.members, BTreeSet::from([origin.clone()]));
    assert_eq!(first.zone_copper[0].polygons, vec![rectangle(0, 10)]);
    assert!(matches!(
        query(&model, &origin, None),
        Err(ElectricalQueryFailure::Physical(
            PhysicalQueryFailure::AmbiguousOrigin { .. }
        ))
    ));
    assert!(matches!(
        query(&model, &origin, Some(Point::new(15, 5))),
        Err(ElectricalQueryFailure::Physical(
            PhysicalQueryFailure::OriginOutsideCopper { .. }
        ))
    ));
    let bridge = line(net, (10, 5), (20, 5));
    tracks(&mut model, &root, std::slice::from_ref(&bridge));
    assert!(matches!(
        query(&model, &origin, None),
        Err(ElectricalQueryFailure::Physical(
            PhysicalQueryFailure::UnavailableFill { .. }
        ))
    ));
    fill(
        &mut model,
        &root,
        &zone,
        ZoneFillState::Filled,
        vec![rectangle(20, 10), rectangle(0, 10)],
    );
    let joined = query(&model, &origin, None).unwrap();
    assert_eq!(
        joined.members,
        BTreeSet::from([origin.clone(), member("tracks", bridge.uuid)])
    );
    assert_eq!(joined.zone_copper.len(), 1);
    assert_eq!(joined.zone_copper[0].zone_id, zone.uuid);
    assert_eq!(joined.zone_copper[0].polygons.len(), 2);
    let reopened = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(query(&reopened, &origin, None).unwrap(), joined);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn successful_empty_fill_is_zero_copper_while_unknown_states_refuse_membership() {
    let (root, mut model, net) = fixture("board_run_empty");
    let zone = zone(&mut model, &root, net);
    let track = line(net, (0, 5), (30, 5));
    tracks(&mut model, &root, std::slice::from_ref(&track));
    for state in [
        ZoneFillState::Unsupported,
        ZoneFillState::Unfilled,
        ZoneFillState::Stale,
    ] {
        fill(&mut model, &root, &zone, state, vec![]);
        assert!(matches!(
            query(&model, &member("tracks", track.uuid), None),
            Err(ElectricalQueryFailure::Physical(
                PhysicalQueryFailure::UnavailableFill { .. }
            ))
        ));
    }
    fill(&mut model, &root, &zone, ZoneFillState::Filled, vec![]);
    let result = query(&model, &member("tracks", track.uuid), None).unwrap();
    assert_eq!(
        result.members,
        BTreeSet::from([member("tracks", track.uuid)])
    );
    assert!(result.zone_copper.is_empty());
    assert!(matches!(
        query(&model, &member("zones", zone.uuid), None),
        Err(ElectricalQueryFailure::Physical(
            PhysicalQueryFailure::OriginOutsideCopper { .. }
        ))
    ));
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn board_run_is_uncapped_and_unknown_contact_assignment_is_not_a_short_free_pass() {
    let (root, mut model, net) = fixture("board_run_complete");
    let chain: Vec<_> = (0..257)
        .map(|i| line(net, (i * 10, 0), (i * 10 + 10, 0)))
        .collect();
    tracks(&mut model, &root, &chain);
    let expected: BTreeSet<_> = chain.iter().map(|t| member("tracks", t.uuid)).collect();
    assert_eq!(
        query(&model, &member("tracks", chain[0].uuid), None)
            .unwrap()
            .members,
        expected
    );
    let unknown = line(Uuid::new_v4(), (5, -5), (5, 5));
    tracks(&mut model, &root, std::slice::from_ref(&unknown));
    assert_eq!(
        query(&model, &member("tracks", chain[0].uuid), None),
        Err(ElectricalQueryFailure::Physical(
            PhysicalQueryFailure::UnavailableAssignment {
                source_id: unknown.uuid
            }
        ))
    );
    let snapshot = ElectricalSelectionSnapshot::capture(&model).unwrap();
    assert_eq!(
        snapshot.board_run(
            &ModelRevision("wrong".into()),
            &member("tracks", chain[0].uuid),
            None
        ),
        Err(ElectricalQueryFailure::StaleRevision)
    );
    let mut wrong_path = member("tracks", chain[0].uuid);
    wrong_path.instance_path.push(Uuid::new_v4());
    assert!(matches!(
        snapshot.board_run(snapshot.revision(), &wrong_path, None),
        Err(ElectricalQueryFailure::InvalidOccurrence { .. })
    ));
    std::fs::remove_dir_all(root).unwrap();
}
