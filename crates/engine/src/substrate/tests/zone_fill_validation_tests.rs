use super::*;

#[test]
fn resolver_rejects_invalid_filled_zone_fill_generated_evidence() {
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    let zone_id = Uuid::new_v4();
    let net_id = Uuid::new_v4();
    let root = temp_project_root("zone_fill_invalid_filled");
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
    let initial = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves");
    let invalid = ZoneFill {
        schema_version: 1,
        zone_id,
        state: ZoneFillState::Filled,
        source_zone_revision: ObjectRevision(0),
        model_revision: initial.model_revision.clone(),
        islands: Vec::new(),
        provenance: Some("unit-test-fill".to_string()),
    };
    persist_zone_fill(&root, &invalid).expect("invalid fill writes for resolver validation");

    let resolved = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves with invalid fill diagnostic");
    let fill = resolved
        .zone_fills
        .get(&zone_id)
        .expect("fallback fill exists");
    assert_eq!(fill.state, ZoneFillState::Unfilled);
    assert!(fill.islands.is_empty());
    assert!(resolved.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == "invalid_zone_fill"
            && diagnostic
                .message
                .contains("filled zone fill must contain at least one island")
    }));
}

#[test]
fn resolver_rejects_self_intersecting_filled_zone_fill_island() {
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    let zone_id = Uuid::new_v4();
    let net_id = Uuid::new_v4();
    let root = temp_project_root("zone_fill_self_intersecting");
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
    let initial = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves");
    let invalid = ZoneFill {
        schema_version: ZONE_FILL_SCHEMA_VERSION,
        zone_id,
        state: ZoneFillState::Filled,
        source_zone_revision: ObjectRevision(0),
        model_revision: initial.model_revision.clone(),
        islands: vec![self_intersecting_island()],
        provenance: Some("unit-test-fill".to_string()),
    };
    persist_zone_fill(&root, &invalid).expect("invalid fill writes for resolver validation");

    let resolved = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves with invalid fill diagnostic");
    let fill = resolved
        .zone_fills
        .get(&zone_id)
        .expect("fallback fill exists");
    assert_eq!(fill.state, ZoneFillState::Unfilled);
    assert!(resolved.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == "invalid_zone_fill"
            && diagnostic
                .message
                .contains("filled zone island 0 must not self-intersect")
    }));
}

#[test]
fn resolver_rejects_nonfilled_zone_fill_with_renderable_islands() {
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    let zone_id = Uuid::new_v4();
    let net_id = Uuid::new_v4();
    let root = temp_project_root("zone_fill_invalid_unsupported");
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
    let initial = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves");
    let invalid = ZoneFill {
        schema_version: ZONE_FILL_SCHEMA_VERSION,
        zone_id,
        state: ZoneFillState::Unsupported,
        source_zone_revision: ObjectRevision(0),
        model_revision: initial.model_revision.clone(),
        islands: vec![filled_island()],
        provenance: Some("unit-test-fill".to_string()),
    };
    persist_zone_fill(&root, &invalid).expect("invalid fill writes for resolver validation");

    let resolved = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves with invalid fill diagnostic");
    let fill = resolved
        .zone_fills
        .get(&zone_id)
        .expect("fallback fill exists");
    assert_eq!(fill.state, ZoneFillState::Unfilled);
    assert!(fill.islands.is_empty());
    assert!(resolved.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == "invalid_zone_fill"
            && diagnostic
                .message
                .contains("Unsupported zone fill must not contain renderable islands")
    }));
}

#[test]
fn resolver_accepts_valid_unsupported_zone_fill_generated_evidence() {
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    let zone_id = Uuid::new_v4();
    let net_id = Uuid::new_v4();
    let root = temp_project_root("zone_fill_valid_unsupported");
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
    let initial = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves");
    let unsupported = ZoneFill {
        schema_version: ZONE_FILL_SCHEMA_VERSION,
        zone_id,
        state: ZoneFillState::Unsupported,
        source_zone_revision: ObjectRevision(0),
        model_revision: initial.model_revision.clone(),
        islands: Vec::new(),
        provenance: Some("unsupported by test solver".to_string()),
    };
    persist_zone_fill(&root, &unsupported).expect("unsupported fill should persist");

    let resolved = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves with unsupported fill");
    assert_eq!(resolved.zone_fills[&zone_id], unsupported);
    assert!(
        !resolved
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "invalid_zone_fill")
    );
}
