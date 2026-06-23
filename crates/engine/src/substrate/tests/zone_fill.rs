use super::*;
use crate::board::{PadShape, PlacedPad, Track, Zone};
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

fn self_intersecting_island() -> Polygon {
    Polygon {
        vertices: vec![
            Point { x: 0, y: 0 },
            Point { x: 1000, y: 1000 },
            Point { x: 0, y: 1000 },
            Point { x: 1000, y: 0 },
        ],
        closed: true,
    }
}

fn zone(zone_id: Uuid, net_id: Uuid, offset: i64, priority: u32) -> Zone {
    Zone {
        uuid: zone_id,
        net: net_id,
        polygon: Polygon {
            vertices: vec![
                Point { x: offset, y: 0 },
                Point {
                    x: offset + 1000,
                    y: 0,
                },
                Point {
                    x: offset + 1000,
                    y: 1000,
                },
                Point { x: offset, y: 1000 },
            ],
            closed: true,
        },
        layer: 0,
        priority,
        thermal_relief: false,
        thermal_gap: 0,
        thermal_spoke_width: 0,
    }
}

#[test]
fn bounded_zone_fill_cuts_out_single_foreign_pad_with_netclass_clearance() {
    let zone_id = Uuid::new_v4();
    let zone_net = Uuid::new_v4();
    let foreign_net = Uuid::new_v4();
    let mut context = ZoneFillCopperContext::default();
    context.net_clearance_nm.insert(zone_net, 100);
    context.pads.push(PlacedPad {
        uuid: Uuid::new_v4(),
        package: Uuid::new_v4(),
        name: "1".to_string(),
        net: Some(foreign_net),
        position: Point { x: 500, y: 500 },
        layer: 0,
        copper_layers: Vec::new(),
        shape: PadShape::Rect,
        diameter: 0,
        width: 200,
        height: 200,
        drill: 0,
        rotation: 0,
        roundrect_rratio_ppm: 250_000,
        mask_layers: Vec::new(),
        paste_layers: Vec::new(),
        solder_mask_margin_nm: 0,
        solder_paste_margin_nm: 0,
        solder_paste_margin_ratio_ppm: 0,
    });

    let (state, islands, provenance) =
        compute_bounded_zone_fill(&zone(zone_id, zone_net, 0, 0), &context);

    assert_eq!(state, ZoneFillState::Filled);
    assert_eq!(islands.len(), 4);
    assert!(islands.iter().all(|island| island.closed));
    assert_eq!(
        provenance,
        "datum-eda fill-zones: bounded rectangular obstacle cutout fill v2; one foreign pad/via/orthogonal track inflated by netclass clearance"
    );
}

#[test]
fn bounded_zone_fill_rejects_foreign_pad_without_clearance_basis() {
    let zone_id = Uuid::new_v4();
    let zone_net = Uuid::new_v4();
    let foreign_net = Uuid::new_v4();
    let mut context = ZoneFillCopperContext::default();
    context.pads.push(PlacedPad {
        uuid: Uuid::new_v4(),
        package: Uuid::new_v4(),
        name: "1".to_string(),
        net: Some(foreign_net),
        position: Point { x: 500, y: 500 },
        layer: 0,
        copper_layers: Vec::new(),
        shape: PadShape::Circle,
        diameter: 200,
        width: 0,
        height: 0,
        drill: 0,
        rotation: 0,
        roundrect_rratio_ppm: 250_000,
        mask_layers: Vec::new(),
        paste_layers: Vec::new(),
        solder_mask_margin_nm: 0,
        solder_paste_margin_nm: 0,
        solder_paste_margin_ratio_ppm: 0,
    });

    let (state, islands, provenance) =
        compute_bounded_zone_fill(&zone(zone_id, zone_net, 0, 0), &context);

    assert_eq!(state, ZoneFillState::Unsupported);
    assert!(islands.is_empty());
    assert_eq!(
        provenance,
        "datum-eda fill-zones: unsupported because zone net clearance is unavailable"
    );
}

#[test]
fn bounded_zone_fill_cuts_out_multiple_non_overlapping_foreign_pads() {
    let zone_id = Uuid::new_v4();
    let zone_net = Uuid::new_v4();
    let foreign_net = Uuid::new_v4();
    let mut context = ZoneFillCopperContext::default();
    context.net_clearance_nm.insert(zone_net, 50);
    for position in [Point { x: 250, y: 250 }, Point { x: 750, y: 750 }] {
        context.pads.push(PlacedPad {
            uuid: Uuid::new_v4(),
            package: Uuid::new_v4(),
            name: "1".to_string(),
            net: Some(foreign_net),
            position,
            layer: 0,
            copper_layers: Vec::new(),
            shape: PadShape::Rect,
            diameter: 0,
            width: 100,
            height: 100,
            drill: 0,
            rotation: 0,
            roundrect_rratio_ppm: 250_000,
            mask_layers: Vec::new(),
            paste_layers: Vec::new(),
            solder_mask_margin_nm: 0,
            solder_paste_margin_nm: 0,
            solder_paste_margin_ratio_ppm: 0,
        });
    }

    let (state, islands, provenance) =
        compute_bounded_zone_fill(&zone(zone_id, zone_net, 0, 0), &context);

    assert_eq!(state, ZoneFillState::Filled);
    assert_eq!(islands.len(), 23);
    assert!(islands.iter().all(|island| island.closed));
    assert_eq!(
        provenance,
        "datum-eda fill-zones: bounded rectangular obstacle cutout fill v3; multiple non-overlapping foreign pads/vias/orthogonal tracks inflated by netclass clearance"
    );
}

#[test]
fn bounded_zone_fill_rejects_overlapping_foreign_pad_cutouts() {
    let zone_id = Uuid::new_v4();
    let zone_net = Uuid::new_v4();
    let foreign_net = Uuid::new_v4();
    let mut context = ZoneFillCopperContext::default();
    context.net_clearance_nm.insert(zone_net, 100);
    for position in [Point { x: 450, y: 500 }, Point { x: 550, y: 500 }] {
        context.pads.push(PlacedPad {
            uuid: Uuid::new_v4(),
            package: Uuid::new_v4(),
            name: "1".to_string(),
            net: Some(foreign_net),
            position,
            layer: 0,
            copper_layers: Vec::new(),
            shape: PadShape::Rect,
            diameter: 0,
            width: 100,
            height: 100,
            drill: 0,
            rotation: 0,
            roundrect_rratio_ppm: 250_000,
            mask_layers: Vec::new(),
            paste_layers: Vec::new(),
            solder_mask_margin_nm: 0,
            solder_paste_margin_nm: 0,
            solder_paste_margin_ratio_ppm: 0,
        });
    }

    let (state, islands, provenance) =
        compute_bounded_zone_fill(&zone(zone_id, zone_net, 0, 0), &context);

    assert_eq!(state, ZoneFillState::Unsupported);
    assert!(islands.is_empty());
    assert_eq!(
        provenance,
        "datum-eda fill-zones: unsupported because obstacle cutouts are outside the bounded non-overlapping rectangular solver envelope"
    );
}

#[test]
fn bounded_zone_fill_cuts_out_single_foreign_orthogonal_track_with_netclass_clearance() {
    let zone_id = Uuid::new_v4();
    let zone_net = Uuid::new_v4();
    let foreign_net = Uuid::new_v4();
    let mut context = ZoneFillCopperContext::default();
    context.net_clearance_nm.insert(zone_net, 50);
    context.tracks.push(Track {
        uuid: Uuid::new_v4(),
        net: foreign_net,
        from: Point { x: 300, y: 500 },
        to: Point { x: 700, y: 500 },
        width: 100,
        layer: 0,
    });

    let (state, islands, provenance) =
        compute_bounded_zone_fill(&zone(zone_id, zone_net, 0, 0), &context);

    assert_eq!(state, ZoneFillState::Filled);
    assert_eq!(islands.len(), 4);
    assert!(islands.iter().all(|island| island.closed));
    assert_eq!(
        provenance,
        "datum-eda fill-zones: bounded rectangular obstacle cutout fill v2; one foreign pad/via/orthogonal track inflated by netclass clearance"
    );
}

#[test]
fn bounded_zone_fill_rejects_non_orthogonal_foreign_track() {
    let zone_id = Uuid::new_v4();
    let zone_net = Uuid::new_v4();
    let foreign_net = Uuid::new_v4();
    let mut context = ZoneFillCopperContext::default();
    context.net_clearance_nm.insert(zone_net, 50);
    context.tracks.push(Track {
        uuid: Uuid::new_v4(),
        net: foreign_net,
        from: Point { x: 300, y: 300 },
        to: Point { x: 700, y: 700 },
        width: 100,
        layer: 0,
    });

    let (state, islands, provenance) =
        compute_bounded_zone_fill(&zone(zone_id, zone_net, 0, 0), &context);

    assert_eq!(state, ZoneFillState::Unsupported);
    assert!(islands.is_empty());
    assert_eq!(
        provenance,
        "datum-eda fill-zones: unsupported because a non-orthogonal different-net track intersects the zone"
    );
}

#[test]
fn journaled_zone_fill_rejects_mismatched_operation_and_payload_ids() {
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    let operation_zone_id = Uuid::new_v4();
    let payload_zone_id = Uuid::new_v4();
    let net_id = Uuid::new_v4();
    let root = temp_project_root("zone_fill_mismatched_payload");
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
                operation_zone_id.to_string(): {
                    "uuid": operation_zone_id,
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
        .expect("project resolves");
    let bad_fill = ZoneFill {
        zone_id: payload_zone_id,
        state: ZoneFillState::Filled,
        source_zone_revision: ObjectRevision(0),
        model_revision: model.model_revision.clone(),
        islands: vec![filled_island()],
        provenance: Some("bad-test-fill".to_string()),
    };

    let error = model
        .commit_journaled(
            &root,
            OperationBatch {
                batch_id: Uuid::new_v4(),
                expected_model_revision: Some(model.model_revision.clone()),
                provenance: CommitProvenance {
                    actor: "unit-test".to_string(),
                    source: CommitSource::Test,
                    reason: "reject mismatched zone fill".to_string(),
                },
                operations: vec![Operation::SetZoneFill {
                    zone_id: operation_zone_id,
                    previous_zone_fill: None,
                    zone_fill: serde_json::to_value(&bad_fill).expect("fill should serialize"),
                }],
            },
        )
        .expect_err("mismatched zone fill payload should fail");

    assert!(
        error
            .to_string()
            .contains("does not match operation zone_id")
    );
    assert!(!root.join(".datum/zone_fills").exists());
}

#[test]
fn zone_fill_projection_renders_only_filled_evidence() {
    let net_id = Uuid::new_v4();
    let filled_zone_id = Uuid::new_v4();
    let unfilled_zone_id = Uuid::new_v4();
    let authored_zones = vec![
        zone(unfilled_zone_id, net_id, 2000, 0),
        zone(filled_zone_id, net_id, 0, 1),
    ];
    let mut zone_fills = BTreeMap::new();
    zone_fills.insert(
        filled_zone_id,
        ZoneFill {
            zone_id: filled_zone_id,
            state: ZoneFillState::Filled,
            source_zone_revision: ObjectRevision(1),
            model_revision: ModelRevision("rev-a".to_string()),
            islands: vec![filled_island()],
            provenance: Some("test-fill".to_string()),
        },
    );
    zone_fills.insert(
        unfilled_zone_id,
        ZoneFill {
            zone_id: unfilled_zone_id,
            state: ZoneFillState::Unfilled,
            source_zone_revision: ObjectRevision(1),
            model_revision: ModelRevision("rev-a".to_string()),
            islands: Vec::new(),
            provenance: None,
        },
    );

    let (rendered, blocked) = zone_fill_copper_projection_zones(&authored_zones, &zone_fills);

    assert_eq!(rendered.len(), 1);
    assert_ne!(rendered[0].uuid, filled_zone_id);
    assert_eq!(rendered[0].net, net_id);
    assert_eq!(rendered[0].polygon, filled_island());
    assert_eq!(blocked, vec![unfilled_zone_id.to_string()]);
}

#[test]
fn resolver_derives_native_zones_as_unfilled_zone_fills() {
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    let zone_id = Uuid::new_v4();
    let net_id = Uuid::new_v4();
    let root = temp_project_root("zone_fill_unfilled");
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
                    "polygon": {
                        "vertices": [
                            { "x": 0, "y": 0 },
                            { "x": 1000, "y": 0 },
                            { "x": 1000, "y": 1000 },
                            { "x": 0, "y": 1000 }
                        ],
                        "closed": true
                    },
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

    let model = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves");
    let fill = model
        .zone_fills
        .get(&zone_id)
        .expect("zone fill should be derived");
    assert_eq!(fill.state, ZoneFillState::Unfilled);
    assert_eq!(fill.zone_id, zone_id);
    assert_eq!(fill.source_zone_revision, ObjectRevision(0));
    assert_eq!(fill.model_revision, model.model_revision);
    assert!(fill.islands.is_empty());
    assert_eq!(fill.provenance, None);
}

#[test]
fn resolver_discovers_persisted_zone_fill_generated_evidence() {
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    let zone_id = Uuid::new_v4();
    let net_id = Uuid::new_v4();
    let root = temp_project_root("zone_fill_persisted");
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
    let persisted = ZoneFill {
        zone_id,
        state: ZoneFillState::Filled,
        source_zone_revision: ObjectRevision(0),
        model_revision: initial.model_revision.clone(),
        islands: vec![filled_island()],
        provenance: Some("unit-test-fill".to_string()),
    };
    let path = persist_zone_fill(&root, &persisted).expect("zone fill should persist");
    assert_eq!(path, root.join(format!(".datum/zone_fills/{zone_id}.json")));

    let resolved = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves with persisted fill");
    assert_eq!(resolved.zone_fills[&zone_id], persisted);
    assert!(resolved.source_shards.iter().any(|shard| {
        shard.kind == SourceShardKind::ZoneFill
            && shard.authority == SourceShardAuthority::GeneratedEvidence
    }));
}

#[test]
fn resolver_marks_persisted_zone_fill_stale_when_model_revision_changes() {
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    let zone_id = Uuid::new_v4();
    let net_id = Uuid::new_v4();
    let root = temp_project_root("zone_fill_stale");
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
    let stale = ZoneFill {
        zone_id,
        state: ZoneFillState::Filled,
        source_zone_revision: ObjectRevision(0),
        model_revision: ModelRevision("stale".to_string()),
        islands: vec![filled_island()],
        provenance: Some("unit-test-fill".to_string()),
    };
    persist_zone_fill(&root, &stale).expect("zone fill should persist");

    let resolved = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves with stale persisted fill");
    let fill = resolved.zone_fills.get(&zone_id).expect("zone fill exists");
    assert_eq!(fill.state, ZoneFillState::Stale);
    assert_eq!(fill.islands, stale.islands);
    assert_eq!(fill.provenance, stale.provenance);
    assert_eq!(fill.model_revision, stale.model_revision);
}

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
