//! PM054 T01/T02 authored anchors and canonical refusal/persistence.
use super::tests::{resolved_model_with_net_and_zone, test_provenance};
use super::*;
use crate::api::native_write::commit_prepared;
use crate::ir::geometry::Point;
use crate::substrate::ProjectResolver;

fn stored(root: &std::path::Path, id: Uuid) -> serde_json::Value {
    let board: serde_json::Value =
        serde_json::from_slice(&std::fs::read(root.join("board/board.json")).unwrap()).unwrap();
    board["tracks"][id.to_string()].clone()
}
#[test]
fn native_arc_and_straight_conversion_preserve_identity_and_exact_anchors_on_history() {
    let (root, mut model, net, _) = resolved_model_with_net_and_zone("pm054_arc_source");
    let mut track = Track::straight(
        Uuid::new_v4(),
        net.uuid,
        Point::new(0, 0),
        Point::new(3_000_000, 0),
        100_001,
        1,
    );
    let before = serde_json::to_value(&track).unwrap();
    assert!(
        before.get("midpoint").is_none(),
        "legacy straight encoding unchanged"
    );
    let write = build_place_board_track(&model, test_provenance(), &track).unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    track.midpoint = Some(Point::new(1_000_000, 3_000_000));
    let arc = serde_json::to_value(&track).unwrap();
    let write = build_set_board_track(&model, test_provenance(), &track).unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    assert_eq!(stored(&root, track.uuid), arc);
    let board_value: serde_json::Value =
        serde_json::from_slice(&std::fs::read(root.join("board/board.json")).unwrap()).unwrap();
    assert_eq!(board_value["schema_version"], 2);
    model
        .commit_journal_undo(&root, test_provenance().into())
        .unwrap();
    assert_eq!(stored(&root, track.uuid), before);
    model
        .commit_journal_redo(&root, test_provenance().into())
        .unwrap();
    assert_eq!(stored(&root, track.uuid), arc);
    let reopened = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(reopened.model_revision, model.model_revision);
    assert_eq!(reopened.objects[&track.uuid], model.objects[&track.uuid]);
    assert_eq!(
        reopened
            .materialized_source_shard_value(crate::substrate::SourceShardKind::BoardRoot)
            .unwrap()["tracks"][track.uuid.to_string()],
        arc
    );
    std::fs::remove_dir_all(root).unwrap();
}
#[test]
fn malformed_arc_source_refuses_atomically_through_raw_canonical_operations() {
    let (root, mut model, net, _) = resolved_model_with_net_and_zone("pm054_arc_refusal");
    let journal = crate::substrate::transaction_journal_path(&root);
    let board_before = std::fs::read(root.join("board/board.json")).unwrap();
    let journal_before = std::fs::read(&journal).unwrap();
    let before = model.clone();
    for midpoint in [
        Point::new(0, 0),
        Point::new(1_000_000, 0),
        Point::new(i64::MAX, i64::MAX),
    ] {
        let track = Track {
            midpoint: Some(midpoint),
            ..Track::straight(
                Uuid::new_v4(),
                net.uuid,
                Point::new(0, 0),
                Point::new(3_000_000, 0),
                100_001,
                1,
            )
        };
        let write = build_place_board_track(&model, test_provenance(), &track).unwrap();
        assert!(commit_prepared(&mut model, &root, write).is_err());
        assert_eq!(
            std::fs::read(root.join("board/board.json")).unwrap(),
            board_before
        );
        assert_eq!(std::fs::read(&journal).unwrap(), journal_before);
        assert_eq!(model.objects, before.objects);
        assert_eq!(model.model_revision, before.model_revision);
    }
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn native_schema_refuses_undeclared_arc_and_future_source_versions() {
    let (root, mut model, net, _) = resolved_model_with_net_and_zone("pm054_arc_schema");
    let track = Track {
        midpoint: Some(Point::new(5, 5)),
        ..Track::straight(
            Uuid::new_v4(),
            net.uuid,
            Point::new(0, 0),
            Point::new(10, 0),
            1,
            1,
        )
    };
    let write = build_place_board_track(&model, test_provenance(), &track).unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    let path = root.join("board/board.json");
    let mut value: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    for version in [1, 4] {
        value["schema_version"] = serde_json::json!(version);
        std::fs::write(&path, serde_json::to_vec_pretty(&value).unwrap()).unwrap();
        assert!(ProjectResolver::new(&root).resolve().is_err());
    }
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn arc_source_uuid_cannot_replace_operation_identity_atomically() {
    let (root, mut model, net, _) = resolved_model_with_net_and_zone("pm054_arc_identity_refusal");
    let track = Track {
        midpoint: Some(Point::new(5, 5)),
        ..Track::straight(
            Uuid::new_v4(),
            net.uuid,
            Point::new(0, 0),
            Point::new(10, 0),
            1,
            1,
        )
    };
    let before = model.clone();
    let board = std::fs::read(root.join("board/board.json")).unwrap();
    let journal = std::fs::read(crate::substrate::transaction_journal_path(&root)).unwrap();
    let mut write = build_place_board_track(&model, test_provenance(), &track).unwrap();
    for operation in &mut write.batch.operations {
        if let crate::substrate::Operation::CreateBoardTrack { track, .. } = operation {
            track["uuid"] = serde_json::json!(Uuid::new_v4());
        }
    }
    assert!(commit_prepared(&mut model, &root, write).is_err());
    assert_eq!(model.model_revision, before.model_revision);
    assert_eq!(model.objects, before.objects);
    assert_eq!(std::fs::read(root.join("board/board.json")).unwrap(), board);
    assert_eq!(
        std::fs::read(crate::substrate::transaction_journal_path(&root)).unwrap(),
        journal
    );
    std::fs::remove_dir_all(root).unwrap();
}
