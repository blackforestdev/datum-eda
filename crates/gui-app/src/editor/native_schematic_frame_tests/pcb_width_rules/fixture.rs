//! Canonical saved design preparation/correction, outside the GUI editing claim.
use super::*;
use eda_engine::api::native_write::{self as native, WriteProvenance, commit_prepared};
use eda_engine::board::{Net, NetClass, Track};
use eda_engine::ir::geometry::{Point, Polygon};
use eda_engine::substrate::CommitSource;
use serde_json::json;

fn provenance() -> WriteProvenance {
    WriteProvenance::new(
        "width-rule-fixture",
        CommitSource::Test,
        "Explicit canonical width-rule fixture operation",
    )
}

pub(super) fn prepare(root: &std::path::Path) -> (Track, Track) {
    super::super::board_transfer::setup(root);
    let mut model = ProjectResolver::new(root).resolve().unwrap();
    if let eda_engine::ir::units::PreFeatureProjectUnitsMigration::Create { profile, receipt } =
        eda_engine::ir::units::migrate_pre_feature_project_units(None).unwrap()
    {
        let prepared = native::project::build_initialize_project_display_units(
            &model,
            provenance(),
            profile,
            &receipt,
        )
        .unwrap();
        commit_prepared(&mut model, root, prepared).unwrap();
    }
    let class = NetClass {
        uuid: Uuid::new_v4(),
        name: "100000nm native minimum".into(),
        clearance: 100000,
        track_width: 100000,
        via_drill: 300000,
        via_diameter: 600000,
        diffpair_width: 0,
        diffpair_gap: 0,
    };
    let net = Net::new(Uuid::new_v4(), "WIDTH_RULE", class.uuid);
    let peer_net = Net::new(Uuid::new_v4(), "CLEARANCE_RULE", class.uuid);
    let ids = [Uuid::new_v4(), Uuid::new_v4()];
    assert_ne!(ids[0], ids[1]);
    let track_id = *ids.iter().max().unwrap();
    let peer_id = *ids.iter().min().unwrap();
    assert!(!model.objects.contains_key(&track_id));
    assert!(!model.objects.contains_key(&peer_id));
    let peer = Track::straight(
        peer_id,
        peer_net.uuid,
        Point::new(7000000, 3500000),
        Point::new(9000000, 3500000),
        500000,
        1,
    );
    let track = Track::straight(
        track_id,
        net.uuid,
        Point::new(2000000, 3000000),
        Point::new(8000000, 3000000),
        200000,
        1,
    );
    let board = model
        .materialized_source_shard_value(SourceShardKind::BoardRoot)
        .unwrap();
    let outline = Polygon::new(vec![
        Point::new(0, 0),
        Point::new(10000000, 0),
        Point::new(10000000, 6000000),
        Point::new(0, 6000000),
    ]);
    let prepared = native::board_layout::build_set_board_outline(
        &model,
        provenance(),
        serde_json::from_value(board["uuid"].clone()).unwrap(),
        &outline,
    )
    .unwrap();
    commit_prepared(&mut model, root, prepared).unwrap();
    let prepared =
        native::board_routing::build_place_board_net_class(&model, provenance(), &class).unwrap();
    commit_prepared(&mut model, root, prepared).unwrap();
    let prepared =
        native::board_routing::build_place_board_net(&model, provenance(), &net).unwrap();
    commit_prepared(&mut model, root, prepared).unwrap();
    let prepared =
        native::board_routing::build_place_board_track(&model, provenance(), &track).unwrap();
    commit_prepared(&mut model, root, prepared).unwrap();
    let prepared =
        native::board_routing::build_place_board_net(&model, provenance(), &peer_net).unwrap();
    commit_prepared(&mut model, root, prepared).unwrap();
    let prepared =
        native::board_routing::build_place_board_track(&model, provenance(), &peer).unwrap();
    commit_prepared(&mut model, root, prepared).unwrap();
    let verb = native::registry::find_native_write_verb("datum.project.create_rule").unwrap();
    let prepared = (verb.build)(&native::registry::NativeWriteContext { model: &model, project_root: root }, provenance(),
        json!({"rule":{"uuid":Uuid::new_v4(),"name":"Authored All width 500000nm","scope":"All","priority":1,"enabled":true,"rule_type":"track_width", "params":{"min_nm":500000,"preferred_nm":500000,"max_nm":800000}}})).unwrap();
    commit_prepared(&mut model, root, prepared).unwrap();
    // Each public write request starts from the current resolved revision.
    model = ProjectResolver::new(root).resolve().unwrap();
    let prepared = (verb.build)(&native::registry::NativeWriteContext { model: &model, project_root: root }, provenance(),
        json!({"rule":{"uuid":Uuid::new_v4(),"name":"Authored All clearance 200000nm","scope":"All","priority":1,"enabled":true,"rule_type":"clearance_copper", "params":{"min_nm":200000}}})).unwrap();
    commit_prepared(&mut model, root, prepared).unwrap();
    (track, peer)
}

pub(super) fn correct(root: &std::path::Path, track: &mut Track, peer: &mut Track) {
    let mut model = ProjectResolver::new(root).resolve().unwrap();
    let before = model.journal.len();
    peer.from.y = 4000000;
    peer.to.y = 4000000;
    let prepared =
        native::board_routing::build_set_board_track(&model, provenance(), peer).unwrap();
    commit_prepared(&mut model, root, prepared).unwrap();
    track.width = 500000;
    let prepared =
        native::board_routing::build_set_board_track(&model, provenance(), track).unwrap();
    commit_prepared(&mut model, root, prepared).unwrap();
    assert_eq!(model.journal.len(), before + 2);
    assert!(
        model.journal[before..]
            .iter()
            .flat_map(|tx| &tx.operations)
            .all(|op| matches!(
                op,
                Operation::SetBoardTrack { .. } | Operation::GuardObjectRevision { .. }
            ))
    );
}
