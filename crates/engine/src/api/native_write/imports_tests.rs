use std::collections::{HashMap, HashSet};

use super::super::test_support::{temp_project_root, write_minimal_project};
use super::*;
use crate::board::{Net, PlacedPackage, Stackup, StackupLayer, StackupLayerType, Track};
use crate::ir::geometry::{Point, Polygon};
use crate::pool::Unit;
use crate::substrate::{CommitSource, ImportMapEntryStatus, ObjectRevision, ProjectResolver};

fn test_provenance() -> WriteProvenance {
    WriteProvenance::new("unit-test", CommitSource::Test, "imports facade test")
}

fn resolved_minimal_model(name: &str) -> (std::path::PathBuf, DesignModel, Uuid) {
    let root = temp_project_root(name);
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    write_minimal_project(&root, project_id, board_id);
    let model = ProjectResolver::new(&root)
        .resolve()
        .expect("fixture project should resolve");
    (root, model, board_id)
}

fn test_import_map_entry(import_key: &str, object_id: Uuid) -> ImportMapEntry {
    ImportMapEntry {
        import_key: import_key.to_string(),
        object_id,
        source_shard_id: source_shard_id_for_relative_path("board/board.json"),
        status: ImportMapEntryStatus::Active,
        source_tool: "kicad".to_string(),
        source_path: "source.kicad_pcb".to_string(),
        source_object_ref: import_key.to_string(),
        source_hash: "sha256:test".to_string(),
    }
}

fn test_board(net_id: Uuid, package_id: Uuid, track_id: Uuid) -> Board {
    Board {
        uuid: Uuid::new_v4(),
        name: "Imported".to_string(),
        stackup: Stackup {
            layers: vec![StackupLayer::new(
                0,
                "F.Cu",
                StackupLayerType::Copper,
                35_000,
            )],
        },
        pad_expansion_setup: Default::default(),
        outline: Polygon {
            vertices: vec![
                Point { x: 0, y: 0 },
                Point { x: 10, y: 0 },
                Point { x: 0, y: 0 },
            ],
            closed: true,
        },
        packages: HashMap::from([(
            package_id,
            PlacedPackage {
                uuid: package_id,
                part: Uuid::new_v4(),
                package: Uuid::new_v4(),
                reference: "U1".to_string(),
                value: "IMPORTED".to_string(),
                position: Point { x: 0, y: 0 },
                rotation: 0,
                layer: 0,
                locked: false,
            },
        )]),
        pads: HashMap::new(),
        tracks: HashMap::from([(
            track_id,
            Track::straight(
                track_id,
                net_id,
                Point { x: 0, y: 0 },
                Point { x: 5, y: 5 },
                250_000,
                0,
            ),
        )]),
        vias: HashMap::new(),
        zones: HashMap::new(),
        nets: HashMap::from([(net_id, Net::new(net_id, "SIG", Uuid::nil()))]),
        net_classes: HashMap::new(),
        rules: Vec::new(),
        keepouts: Vec::new(),
        dimensions: Vec::new(),
        texts: Vec::new(),
    }
}

#[test]
fn kicad_board_import_matches_hand_built_operation_order_oracle() {
    let (_root, model, board_id) = resolved_minimal_model("imports_board_oracle");
    // Fixed ids ordered so the sorted-by-id family order is observable.
    let net_id = Uuid::from_u128(1);
    let package_id = Uuid::from_u128(2);
    let track_id = Uuid::from_u128(3);
    let board = test_board(net_id, package_id, track_id);
    let entries = vec![test_import_map_entry("kicad:board-segment:test", track_id)];

    let write = build_kicad_board_import(
        &model,
        test_provenance(),
        board_id,
        &board,
        entries.clone(),
        Path::new("source.kicad_pcb"),
    )
    .expect("board import should build");
    let prepared = write.prepared.expect("board import should have a batch");

    // Hand-built oracle: the historical CLI operation sequence — creates
    // sorted by id within each family (nets, packages, pads, tracks,
    // vias, zones), then outline/stackup rewrites (now preceded by the
    // facade's board revision guard), then the import-map shard.
    let oracle = vec![
        Operation::CreateBoardNet {
            net_id,
            net: serde_json::to_value(&board.nets[&net_id]).unwrap(),
        },
        Operation::CreateBoardPackage {
            package_id,
            package: serde_json::to_value(&board.packages[&package_id]).unwrap(),
            materialized: serde_json::json!({}),
        },
        Operation::CreateBoardTrack {
            track_id,
            track: serde_json::to_value(&board.tracks[&track_id]).unwrap(),
        },
        Operation::GuardObjectRevision {
            object_id: board_id,
            expected_object_revision: ObjectRevision(0),
        },
        Operation::SetBoardOutline {
            board_id,
            outline: serde_json::to_value(&board.outline).unwrap(),
        },
        Operation::SetBoardStackup {
            board_id,
            stackup: serde_json::to_value(&board.stackup).unwrap(),
        },
        Operation::CreateImportMapShard {
            relative_path: kicad_board_import_map_relative_path(Path::new("source.kicad_pcb")),
            shard: serde_json::to_value(ImportMapShard {
                schema_version: 1,
                entries,
            })
            .unwrap(),
        },
    ];
    assert_eq!(prepared.batch.operations, oracle);
    assert_eq!(write.created_object_count, 3);
    assert_eq!(
        prepared.batch.expected_model_revision,
        Some(model.model_revision.clone())
    );
}

#[test]
fn kicad_board_import_skips_resolver_known_objects() {
    let (_root, model, board_id) = resolved_minimal_model("imports_board_noop");
    let mut board = test_board(Uuid::from_u128(1), Uuid::from_u128(2), Uuid::from_u128(3));
    board.nets.clear();
    board.packages.clear();
    board.tracks.clear();

    let write = build_kicad_board_import(
        &model,
        test_provenance(),
        board_id,
        &board,
        Vec::new(),
        Path::new("source.kicad_pcb"),
    )
    .expect("board import should build");

    // No objects to create and no import-map entries: the only remaining
    // operations are the (guarded) outline/stackup rewrites against the
    // fixture board root, never a creation.
    assert_eq!(write.created_object_count, 0);
    let prepared = write.prepared.expect("outline/stackup rewrite batch");
    assert!(prepared.batch.operations.iter().all(|operation| matches!(
        operation,
        Operation::GuardObjectRevision { .. }
            | Operation::SetBoardOutline { .. }
            | Operation::SetBoardStackup { .. }
    )));
}

#[test]
fn eagle_library_import_is_guard_free_and_matches_oracle() {
    let (_root, model, _board_id) = resolved_minimal_model("imports_eagle_oracle");
    let unit_id = Uuid::from_u128(7);
    let mut pool = Pool::default();
    pool.units.insert(
        unit_id,
        Unit {
            uuid: unit_id,
            name: "OPAMP".to_string(),
            manufacturer: "Test".to_string(),
            pins: HashMap::new(),
            tags: HashSet::new(),
        },
    );
    let entries = vec![test_import_map_entry("eagle:lbr:test:units", unit_id)];

    let write = build_eagle_library_import(
        &model,
        test_provenance(),
        "pool",
        &pool,
        entries.clone(),
        Path::new("source.lbr"),
    )
    .expect("eagle import should build");
    let prepared = write.prepared.expect("eagle import should have a batch");

    let mut unit_payload = serde_json::to_value(&pool.units[&unit_id]).unwrap();
    unit_payload
        .as_object_mut()
        .unwrap()
        .insert("schema_version".to_string(), serde_json::json!(1));
    let oracle = vec![
        Operation::AddProjectPoolRef {
            path: "pool".to_string(),
            priority: 1,
        },
        Operation::CreatePoolLibraryObject {
            object_id: unit_id,
            relative_path: format!("pool/units/{unit_id}.json"),
            object_kind: "units".to_string(),
            object: unit_payload,
        },
        Operation::CreateImportMapShard {
            relative_path: eagle_library_import_map_relative_path(Path::new("source.lbr")),
            shard: serde_json::to_value(ImportMapShard {
                schema_version: 1,
                entries,
            })
            .unwrap(),
        },
    ];
    assert_eq!(prepared.batch.operations, oracle);
    assert_eq!(write.created_object_count, 1);
    // Import creations are guard-free: the guard pass is a no-op.
    assert!(
        prepared
            .batch
            .operations
            .iter()
            .all(|operation| !matches!(operation, Operation::GuardObjectRevision { .. }))
    );
}

#[test]
fn schematic_sheet_imports_compose_one_guard_free_batch() {
    let (_root, model, _board_id) = resolved_minimal_model("imports_schematic_sheets");
    let schematic_root = model
        .materialized_source_shard_value(SourceShardKind::SchematicRoot)
        .expect("schematic root should materialize");
    let schematic_id: Uuid = schematic_root["uuid"]
        .as_str()
        .unwrap()
        .parse()
        .expect("schematic uuid");
    let first = Uuid::from_u128(11);
    let second = Uuid::from_u128(12);
    let specs = vec![
        KiCadSchematicSheetCreateSpec {
            sheet_id: first,
            relative_path: format!("sheets/{first}.json"),
            sheet: serde_json::json!({ "schema_version": 1, "uuid": first }),
        },
        KiCadSchematicSheetCreateSpec {
            sheet_id: second,
            relative_path: format!("sheets/{second}.json"),
            sheet: serde_json::json!({ "schema_version": 1, "uuid": second }),
        },
    ];

    let prepared =
        build_kicad_schematic_sheet_imports(&model, test_provenance(), schematic_id, specs)
            .expect("sheet imports should build");

    assert_eq!(
        prepared.batch.operations,
        vec![
            Operation::CreateSchematicSheet {
                schematic_id,
                sheet_id: first,
                relative_path: format!("sheets/{first}.json"),
                sheet: serde_json::json!({ "schema_version": 1, "uuid": first }),
            },
            Operation::CreateSchematicSheet {
                schematic_id,
                sheet_id: second,
                relative_path: format!("sheets/{second}.json"),
                sheet: serde_json::json!({ "schema_version": 1, "uuid": second }),
            },
        ]
    );
}

#[test]
fn kicad_footprint_import_matches_oracle_and_reused_identity_skips_creates() {
    let (_root, model, _board_id) = resolved_minimal_model("imports_footprint_oracle");
    let footprint_source =
        temp_project_root("imports_footprint_fixture").join("native-import.kicad_mod");
    std::fs::write(
        &footprint_source,
        r#"(footprint "NativeImportFootprint"
  (layer "F.Cu")
  (fp_line (start -1 -0.8) (end 1 -0.8) (layer "F.SilkS") (width 0.12))
  (pad "1" smd rect (at 0 0) (size 1 1) (layers "F.Cu" "F.Paste" "F.Mask"))
)"#,
    )
    .expect("footprint fixture should write");
    let (imported, _report) = crate::import::kicad::import_footprint_document(&footprint_source)
        .expect("footprint should import");
    let package_id = imported.package.uuid;
    let entry = test_import_map_entry("kicad:footprint-package:test", package_id);

    let write = build_kicad_footprint_import(
        &model,
        test_provenance(),
        "pool",
        &imported,
        false,
        vec![entry.clone()],
    )
    .expect("footprint import should build");
    let prepared = write
        .prepared
        .expect("footprint import should have a batch");

    let mut oracle = vec![Operation::AddProjectPoolRef {
        path: "pool".to_string(),
        priority: 1,
    }];
    for padstack in &imported.padstacks {
        oracle.push(Operation::CreatePoolPadstack {
            padstack_id: padstack.uuid,
            relative_path: format!("pool/padstacks/{}.json", padstack.uuid),
            padstack: serde_json::to_value(padstack).unwrap(),
        });
    }
    oracle.push(Operation::CreatePoolPackage {
        package_id,
        relative_path: format!("pool/packages/{package_id}.json"),
        package: serde_json::to_value(&imported.package).unwrap(),
    });
    let mut footprint_value = serde_json::to_value(&imported.footprint).unwrap();
    footprint_value
        .as_object_mut()
        .unwrap()
        .insert("schema_version".to_string(), serde_json::json!(1));
    oracle.push(Operation::CreatePoolLibraryObject {
        object_id: imported.footprint.uuid,
        relative_path: format!("pool/footprints/{}.json", imported.footprint.uuid),
        object_kind: "footprints".to_string(),
        object: footprint_value,
    });
    oracle.push(Operation::CreateImportMapShard {
        relative_path: kicad_footprint_import_map_relative_path(package_id),
        shard: serde_json::to_value(ImportMapShard {
            schema_version: 1,
            entries: vec![entry],
        })
        .unwrap(),
    });
    assert_eq!(prepared.batch.operations, oracle);
    assert_eq!(
        write.created_object_count,
        imported.padstacks.len() + 2,
        "padstacks + package + footprint"
    );

    // Reused identity with the pool already referenced is a no-op write.
    let reused = build_kicad_footprint_import(
        &model,
        test_provenance(),
        "pool",
        &imported,
        true,
        Vec::new(),
    )
    .expect("reused footprint import should build");
    let reused_prepared = reused
        .prepared
        .expect("pool ref is still missing, so one op remains");
    assert_eq!(
        reused_prepared.batch.operations,
        vec![Operation::AddProjectPoolRef {
            path: "pool".to_string(),
            priority: 1,
        }]
    );
    assert_eq!(reused.created_object_count, 0);
}

#[test]
fn import_identity_derivations_match_historical_cli_formulas() {
    let source = Path::new("/tmp/demo.kicad_pcb");
    // Byte-exact historical CLI v5 derivations (pre-migration
    // command_project_imports*.rs).
    assert_eq!(
        source_shard_id_for_relative_path("board/board.json"),
        Uuid::new_v5(
            &Uuid::NAMESPACE_URL,
            "datum-eda:source-shard:board/board.json".as_bytes(),
        )
    );
    let board_id = Uuid::new_v5(
        &Uuid::NAMESPACE_URL,
        format!("datum-eda:kicad-board-import-map:{}", source.display()).as_bytes(),
    );
    assert_eq!(
        kicad_board_import_map_relative_path(source),
        format!(".datum/import_map/kicad-board-{board_id}.json")
    );
    let schematic_id = Uuid::new_v5(
        &Uuid::NAMESPACE_URL,
        format!("datum-eda:kicad-schematic-import-map:{}", source.display()).as_bytes(),
    );
    assert_eq!(
        kicad_schematic_import_map_relative_path(source),
        format!(".datum/import_map/kicad-schematic-{schematic_id}.json")
    );
    let eagle_id = Uuid::new_v5(
        &Uuid::NAMESPACE_URL,
        format!("datum-eda:eagle-library-import-map:{}", source.display()).as_bytes(),
    );
    assert_eq!(
        eagle_library_import_map_relative_path(source),
        format!(".datum/import_map/eagle-library-{eagle_id}.json")
    );
    let object_id = Uuid::from_u128(42);
    assert_eq!(
        eagle_pool_import_key(source, "units", object_id),
        format!("eagle:lbr:{}:units:{object_id}", source.display())
    );
    assert_eq!(
        eagle_pool_relative_path("pool", "units", object_id),
        format!("pool/units/{object_id}.json")
    );
    assert_eq!(
        kicad_footprint_import_map_relative_path(object_id),
        format!(".datum/import_map/kicad-footprint-{object_id}.json")
    );
}
