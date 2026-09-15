use super::*;
use crate::ir::geometry::{Point, Polygon};
use crate::substrate::zone_fill::persist_zone_fill;

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

fn write_project_with_zone(root: &Path, project_id: Uuid, board_id: Uuid, zone_id: Uuid) {
    let net_id = Uuid::new_v4();
    write_minimal_project(root, project_id, board_id);
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
}

fn filled_zone_fill(zone_id: Uuid, model_revision: ModelRevision) -> ZoneFill {
    ZoneFill {
        schema_version: ZONE_FILL_SCHEMA_VERSION,
        zone_id,
        state: ZoneFillState::Filled,
        source_zone_revision: ObjectRevision(0),
        model_revision,
        islands: vec![filled_island()],
        provenance: Some("unit-test-fill".to_string()),
    }
}

#[test]
fn resolver_discovers_versioned_zone_fill_generated_evidence() {
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    let zone_id = Uuid::new_v4();
    let root = temp_project_root("zone_fill_schema_versioned");
    write_project_with_zone(&root, project_id, board_id, zone_id);
    let initial = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves");
    let persisted = filled_zone_fill(zone_id, initial.model_revision.clone());

    let path = persist_zone_fill(&root, &persisted).expect("zone fill should persist");
    assert_eq!(path, root.join(format!(".datum/zone_fills/{zone_id}.json")));

    let resolved = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves with persisted fill");
    assert_eq!(resolved.zone_fills[&zone_id], persisted);
    assert_eq!(
        resolved.zone_fills[&zone_id].schema_version,
        ZONE_FILL_SCHEMA_VERSION
    );
    assert!(resolved.source_shards.iter().any(|shard| {
        shard.kind == SourceShardKind::ZoneFill
            && shard.taxon == Some(SourceShardTaxon::ZoneFill)
            && shard.authority == SourceShardAuthority::GeneratedEvidence
            && shard.schema_version == Some(ZONE_FILL_SCHEMA_VERSION)
    }));
}

#[test]
fn resolver_accepts_legacy_zone_fill_without_schema_version() {
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    let zone_id = Uuid::new_v4();
    let root = temp_project_root("zone_fill_schema_legacy");
    write_project_with_zone(&root, project_id, board_id, zone_id);
    let initial = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves");
    let mut legacy =
        serde_json::to_value(filled_zone_fill(zone_id, initial.model_revision.clone()))
            .expect("zone fill should serialize");
    legacy
        .as_object_mut()
        .expect("zone fill should be object")
        .remove("schema_version");
    write_json(
        &root.join(format!(".datum/zone_fills/{zone_id}.json")),
        legacy,
    );

    let resolved = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves with legacy persisted fill");
    assert_eq!(
        resolved.zone_fills[&zone_id].schema_version,
        ZONE_FILL_SCHEMA_VERSION
    );
    assert!(resolved.source_shards.iter().any(|shard| {
        shard.kind == SourceShardKind::ZoneFill
            && shard.authority == SourceShardAuthority::GeneratedEvidence
            && shard.schema_version.is_none()
    }));
}

#[test]
fn resolver_rejects_unsupported_zone_fill_schema_version() {
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    let zone_id = Uuid::new_v4();
    let root = temp_project_root("zone_fill_schema_unsupported");
    write_project_with_zone(&root, project_id, board_id, zone_id);
    let initial = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves");
    let mut unsupported =
        serde_json::to_value(filled_zone_fill(zone_id, initial.model_revision.clone()))
            .expect("zone fill should serialize");
    unsupported["schema_version"] = serde_json::json!(ZONE_FILL_SCHEMA_VERSION + 1);
    write_json(
        &root.join(format!(".datum/zone_fills/{zone_id}.json")),
        unsupported,
    );

    let resolved = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves with invalid fill diagnostic");
    assert_eq!(resolved.zone_fills[&zone_id].state, ZoneFillState::Unfilled);
    assert!(resolved.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == "invalid_zone_fill"
            && diagnostic
                .message
                .contains("unsupported ZoneFill schema_version")
    }));
}
