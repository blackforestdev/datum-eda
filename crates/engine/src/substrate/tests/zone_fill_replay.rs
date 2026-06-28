use super::*;
use crate::ir::geometry::{Point, Polygon};

fn filled_island() -> Polygon {
    Polygon {
        vertices: vec![
            Point { x: 0, y: 0 },
            Point { x: 1000, y: 0 },
            Point { x: 1000, y: 1000 },
            Point { x: 0, y: 1000 },
        ],
        closed: true,
    }
}

#[test]
fn journal_replay_recovers_missing_zone_fill_generated_evidence() {
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    let zone_id = Uuid::new_v4();
    let net_id = Uuid::new_v4();
    let root = temp_project_root("zone_fill_missing_promoted_shard");
    write_minimal_project(&root, project_id, board_id);
    write_json(
        &root.join("board/board.json"),
        serde_json::json!({
            "schema_version": 1,
            "uuid": board_id,
            "name": "Board",
            "packages": {},
            "tracks": {},
            "vias": {},
            "zones": {
                zone_id.to_string(): {
                    "uuid": zone_id,
                    "net": net_id,
                    "polygon": filled_island(),
                    "layer": 0,
                    "priority": 0,
                    "thermal_relief": false,
                    "thermal_gap": 0,
                    "thermal_spoke_width": 0
                }
            },
            "nets": {},
            "net_classes": {}
        }),
    );

    let mut model = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves before zone fill");
    let fill = ZoneFill {
        schema_version: ZONE_FILL_SCHEMA_VERSION,
        zone_id,
        state: ZoneFillState::Filled,
        source_zone_revision: ObjectRevision(0),
        model_revision: model.model_revision.clone(),
        islands: vec![filled_island()],
        provenance: Some("unit-test-journal-fill".to_string()),
    };
    model
        .commit_journaled(
            &root,
            OperationBatch {
                batch_id: Uuid::new_v5(&project_id, b"recover-zone-fill"),
                expected_model_revision: Some(model.model_revision.clone()),
                provenance: CommitProvenance {
                    actor: "unit-test".to_string(),
                    source: CommitSource::Test,
                    reason: "recover zone fill from journal".to_string(),
                },
                operations: vec![Operation::SetZoneFill {
                    zone_id,
                    previous_zone_fill: None,
                    zone_fill: serde_json::to_value(&fill).expect("fill should serialize"),
                }],
            },
        )
        .expect("zone fill should commit");

    std::fs::remove_file(root.join(format!(".datum/zone_fills/{zone_id}.json")))
        .expect("promoted zone fill should remove");
    let replayed = ProjectResolver::new(&root)
        .resolve()
        .expect("project should recover zone fill from journal");
    assert_eq!(replayed.zone_fills[&zone_id], fill);
    assert!(replayed.source_shards.iter().any(|shard| {
        shard.kind == SourceShardKind::ZoneFill
            && shard.authority == SourceShardAuthority::GeneratedEvidence
            && shard.relative_path == format!(".datum/zone_fills/{zone_id}.json")
            && shard.schema_version == Some(ZONE_FILL_SCHEMA_VERSION)
    }));
}

#[test]
fn journal_replay_deleted_zone_fill_suppresses_stale_promoted_evidence() {
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    let zone_id = Uuid::new_v4();
    let net_id = Uuid::new_v4();
    let root = temp_project_root("zone_fill_deleted_stale_promoted_shard");
    write_minimal_project(&root, project_id, board_id);
    write_json(
        &root.join("board/board.json"),
        serde_json::json!({
            "schema_version": 1,
            "uuid": board_id,
            "name": "Board",
            "packages": {},
            "tracks": {},
            "vias": {},
            "zones": {
                zone_id.to_string(): {
                    "uuid": zone_id,
                    "net": net_id,
                    "polygon": filled_island(),
                    "layer": 0,
                    "priority": 0,
                    "thermal_relief": false,
                    "thermal_gap": 0,
                    "thermal_spoke_width": 0
                }
            },
            "nets": {},
            "net_classes": {}
        }),
    );

    let mut model = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves before zone fill");
    let fill = ZoneFill {
        schema_version: ZONE_FILL_SCHEMA_VERSION,
        zone_id,
        state: ZoneFillState::Filled,
        source_zone_revision: ObjectRevision(0),
        model_revision: model.model_revision.clone(),
        islands: vec![filled_island()],
        provenance: Some("unit-test-journal-fill-delete".to_string()),
    };
    let before_fill_revision = model.model_revision.clone();
    model
        .commit_journaled(
            &root,
            OperationBatch {
                batch_id: Uuid::new_v5(&project_id, b"set-zone-fill-before-delete"),
                expected_model_revision: Some(model.model_revision.clone()),
                provenance: CommitProvenance {
                    actor: "unit-test".to_string(),
                    source: CommitSource::Test,
                    reason: "record zone fill generated evidence before delete".to_string(),
                },
                operations: vec![Operation::SetZoneFill {
                    zone_id,
                    previous_zone_fill: None,
                    zone_fill: serde_json::to_value(&fill).expect("fill should serialize"),
                }],
            },
        )
        .expect("zone fill should commit");
    assert_eq!(
        model.model_revision, before_fill_revision,
        "generated evidence set must not mutate the authored model revision"
    );

    let promoted_path = root.join(format!(".datum/zone_fills/{zone_id}.json"));
    let stale_promoted_bytes =
        std::fs::read(&promoted_path).expect("promoted zone fill should exist before delete");
    let before_delete_revision = model.model_revision.clone();
    model
        .commit_journaled(
            &root,
            OperationBatch {
                batch_id: Uuid::new_v5(&project_id, b"delete-zone-fill"),
                expected_model_revision: Some(model.model_revision.clone()),
                provenance: CommitProvenance {
                    actor: "unit-test".to_string(),
                    source: CommitSource::Test,
                    reason: "delete zone fill generated evidence".to_string(),
                },
                operations: vec![Operation::DeleteZoneFill {
                    zone_id,
                    zone_fill: serde_json::to_value(&fill).expect("fill should serialize"),
                }],
            },
        )
        .expect("zone fill delete should commit");
    assert_eq!(
        model.model_revision, before_delete_revision,
        "generated evidence delete must not mutate the authored model revision"
    );
    assert!(
        !promoted_path.exists(),
        "delete operation should remove promoted zone fill shard"
    );

    std::fs::write(&promoted_path, stale_promoted_bytes)
        .expect("stale promoted zone fill should be restored to prove replay authority");
    let replayed = ProjectResolver::new(&root)
        .resolve()
        .expect("project should resolve with stale promoted zone fill");
    let replayed_fill = replayed
        .zone_fills
        .get(&zone_id)
        .expect("authored zones derive an unfilled zone-fill state after evidence delete");
    assert_eq!(
        replayed_fill.state,
        ZoneFillState::Unfilled,
        "journaled delete must suppress stale promoted filled evidence"
    );
    assert!(
        replayed_fill.islands.is_empty(),
        "journaled delete must not retain stale generated copper islands"
    );
    assert!(!replayed.source_shards.iter().any(|shard| {
        shard.kind == SourceShardKind::ZoneFill
            && shard.relative_path == format!(".datum/zone_fills/{zone_id}.json")
    }));
}
