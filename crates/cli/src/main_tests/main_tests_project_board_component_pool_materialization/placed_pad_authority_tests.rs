//! Existing selected-Footprint recipe now verifies journaled per-placement sources.
use super::*;

#[test]
fn project_board_component_materialization_uses_part_default_footprint() {
    let root = unique_project_root("datum-eda-cli-project-board-component-part-default-footprint");
    create_native_project(
        &root,
        Some("Board Component Part Default Footprint".to_string()),
    )
    .expect("initial scaffold should succeed");
    configure_native_project_for_pool_materialization(
        &root,
        serde_json::json!([{ "path": "pool", "priority": 1 }]),
        silkscreen_stackup(21),
    );

    let pool_root = root.join("pool");
    let package_uuid = Uuid::new_v4();
    let part_uuid = Uuid::new_v4();
    let fallback_footprint_uuid = Uuid::from_u128(1);
    let default_footprint_uuid = Uuid::from_u128(u128::MAX);
    let fallback_pad_uuid = Uuid::new_v4();
    let default_pad_uuid = Uuid::new_v4();
    let fallback_padstack_uuid = Uuid::new_v4();
    let default_padstack_uuid = Uuid::new_v4();

    write_pool_padstack(
        &pool_root,
        &Padstack {
            uuid: fallback_padstack_uuid,
            name: "fallback-padstack".to_string(),
            aperture: Some(PadstackAperture::Circle {
                diameter_nm: 111_000,
            }),
            drill_nm: None,
            plated: None,
            layer_span: Default::default(),
            mask_policy: Default::default(),
            paste_policy: Default::default(),
            annular_ring_nm: None,
            thermal: None,
            antipad: None,
        },
    );
    write_pool_padstack(
        &pool_root,
        &Padstack {
            uuid: default_padstack_uuid,
            name: "default-padstack".to_string(),
            aperture: Some(PadstackAperture::Rect {
                width_nm: 444_000,
                height_nm: 555_000,
            }),
            drill_nm: None,
            plated: None,
            layer_span: Default::default(),
            mask_policy: Default::default(),
            paste_policy: Default::default(),
            annular_ring_nm: None,
            thermal: None,
            antipad: None,
        },
    );
    write_pool_package(
        &pool_root,
        &Package {
            uuid: package_uuid,
            name: "PKG-BODY".to_string(),
            package_family: Some("SOIC".to_string()),
            package_code: Some("SOIC-8".to_string()),
            mounting_type: Some("smd".to_string()),
            body_dimensions: None,
            terminals: HashMap::new(),
            pads: HashMap::new(),
            courtyard: Polygon {
                vertices: Vec::new(),
                closed: false,
            },
            silkscreen: Vec::new(),
            models_3d: Vec::new(),
            body_height_nm: None,
            body_height_mounted_nm: None,
            tags: HashSet::new(),
        },
    );
    write_pool_footprint(
        &pool_root,
        &Footprint {
            uuid: fallback_footprint_uuid,
            name: "00-fallback".to_string(),
            package: package_uuid,
            pads: HashMap::from([(
                fallback_pad_uuid,
                eda_engine::pool::Pad {
                    uuid: fallback_pad_uuid,
                    name: "FALLBACK".to_string(),
                    position: Point { x: 0, y: 0 },
                    padstack: fallback_padstack_uuid,
                    layer: 1,
                },
            )]),
            courtyard: Polygon {
                vertices: Vec::new(),
                closed: false,
            },
            silkscreen: Vec::new(),
            fab: Vec::new(),
            assembly: Vec::new(),
            mechanical: Vec::new(),
            models_3d: Vec::new(),
            standards_basis: None,
            ipc_basis: None,
            process_aperture_policy: Some("explicit".to_string()),
            tags: HashSet::new(),
        },
    );
    write_pool_footprint(
        &pool_root,
        &Footprint {
            uuid: default_footprint_uuid,
            name: "zz-default".to_string(),
            package: package_uuid,
            pads: HashMap::from([(
                default_pad_uuid,
                eda_engine::pool::Pad {
                    uuid: default_pad_uuid,
                    name: "DEFAULT".to_string(),
                    position: Point {
                        x: 100_000,
                        y: 200_000,
                    },
                    padstack: default_padstack_uuid,
                    layer: 1,
                },
            )]),
            courtyard: Polygon {
                vertices: Vec::new(),
                closed: false,
            },
            silkscreen: Vec::new(),
            fab: Vec::new(),
            assembly: Vec::new(),
            mechanical: Vec::new(),
            models_3d: Vec::new(),
            standards_basis: None,
            ipc_basis: None,
            process_aperture_policy: Some("explicit".to_string()),
            tags: HashSet::new(),
        },
    );
    write_pool_part(
        &pool_root,
        &Part {
            uuid: part_uuid,
            entity: Uuid::new_v4(),
            package: package_uuid,
            default_footprint: Some(default_footprint_uuid),
            default_pin_pad_map: None,
            pad_map: HashMap::new(),
            mpn: "DUT".to_string(),
            manufacturer: "Datum".to_string(),
            manufacturer_jep106: None,
            value: "DUT".to_string(),
            description: String::new(),
            datasheet: String::new(),
            parametric: HashMap::new(),
            orderable_mpns: Vec::new(),
            packaging_options: Vec::new(),
            tags: HashSet::new(),
            lifecycle: Lifecycle::Unknown,
            base: None,
            behavioural_models: Vec::new(),
            thermal: None,
            supply_chain_offers: None,
            last_supply_chain_check: None,
        },
    );

    let place_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "place-board-component",
        root.to_str().unwrap(),
        "--part",
        &part_uuid.to_string(),
        "--package",
        &package_uuid.to_string(),
        "--reference",
        "U1",
        "--value",
        "Driver",
        "--x-nm",
        "0",
        "--y-nm",
        "0",
        "--layer",
        "1",
    ])
    .expect("CLI should parse");
    let placed_output = execute(place_cli).expect("place should succeed");
    let placed: serde_json::Value =
        serde_json::from_str(&placed_output).expect("place output should parse");
    let component_uuid = placed["component_uuid"].as_str().unwrap().to_string();

    let board_json = root.join("board/board.json");
    let board_value: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&board_json).expect("board should read"))
            .expect("board should parse");
    let persisted_pads = board_value["component_pads"][&component_uuid]
        .as_array()
        .expect("persisted component pads should exist");
    assert_eq!(persisted_pads.len(), 1);
    assert_eq!(persisted_pads[0]["name"], "DEFAULT");
    assert_eq!(persisted_pads[0]["shape"], "rect");
    assert_eq!(persisted_pads[0]["width_nm"], 444_000);

    assert_eq!(placed["pad_materialization_status"], "current");
    assert!(
        placed["source_profile_adoption_transaction"]
            .as_str()
            .is_some()
    );
    assert_eq!(board_value["schema_version"], 4);
    let first_id = persisted_pads[0]["uuid"].as_str().unwrap().to_string();
    assert_ne!(first_id, default_pad_uuid.to_string());
    assert_eq!(
        persisted_pads[0]["source_binding"]["library_pad"]["object_id"],
        default_pad_uuid.to_string()
    );
    assert_eq!(
        persisted_pads[0]["source_binding"]["land_pattern"]["object_id"],
        default_footprint_uuid.to_string()
    );
    let second = place_component(&root, part_uuid, package_uuid);
    let model = eda_engine::substrate::ProjectResolver::new(&root)
        .resolve()
        .unwrap();
    let source = model
        .materialized_source_shard_value(eda_engine::substrate::SourceShardKind::BoardRoot)
        .unwrap();
    assert_ne!(source["component_pads"][&second][0]["uuid"], first_id);
    let before = source["component_pads"][&component_uuid].clone();
    let changed = set_native_project_board_component_part(
        &root,
        component_uuid.parse().unwrap(),
        Uuid::new_v4(),
    )
    .unwrap();
    assert_eq!(changed.pad_materialization_status, "stale");
    let model = eda_engine::substrate::ProjectResolver::new(&root)
        .resolve()
        .unwrap();
    let source = model
        .materialized_source_shard_value(eda_engine::substrate::SourceShardKind::BoardRoot)
        .unwrap();
    assert_eq!(source["component_pads"][&component_uuid], before);
    let projection =
        eda_engine::substrate::component_pad_source::component_pad_projection(&model, &source)
            .unwrap();
    assert_eq!(projection.pads.len(), 2);
    assert_eq!(
        projection.pads[&first_id.parse().unwrap()].position,
        Point::new(100000, 200000)
    );
    assert_eq!(projection.pads[&first_id.parse().unwrap()].net, None);
    execute(
        Cli::try_parse_from([
            "eda",
            "project",
            "rotate-board-component",
            root.to_str().unwrap(),
            "--component",
            &component_uuid,
            "--rotation-deg",
            "90",
        ])
        .unwrap(),
    )
    .unwrap();
    let model = eda_engine::substrate::ProjectResolver::new(&root)
        .resolve()
        .unwrap();
    let source = model
        .materialized_source_shard_value(eda_engine::substrate::SourceShardKind::BoardRoot)
        .unwrap();
    let projection =
        eda_engine::substrate::component_pad_source::component_pad_projection(&model, &source)
            .unwrap();
    assert_eq!(
        projection.pads[&first_id.parse().unwrap()].position,
        Point::new(-200000, 100000)
    );
    let before = std::fs::read(&board_json).unwrap();
    let journal = std::fs::read(eda_engine::substrate::transaction_journal_path(&root)).unwrap();
    let loaded = load_native_project_with_resolved_board(&root).unwrap();
    let physical = build_native_project_board(&loaded).unwrap();
    assert_eq!(physical.pads.len(), 2);
    assert_eq!(
        physical.pads[&first_id.parse().unwrap()].position,
        Point::new(-200000, 100000)
    );
    let artifact = root.join("placed-pads.gbr");
    let export = export_native_project_gerber_copper_layer(&root, 1, &artifact).unwrap();
    assert_eq!(export.pad_count, 2);
    let cam = std::fs::read_to_string(&artifact).unwrap();
    assert!(cam.contains("X-200000Y100000D03*"));
    assert!(cam.contains("R,0.555000X0.444000*%"));
    assert_eq!(std::fs::read(&board_json).unwrap(), before);
    assert_eq!(
        std::fs::read(eda_engine::substrate::transaction_journal_path(&root)).unwrap(),
        journal
    );
    execute(
        Cli::try_parse_from([
            "eda",
            "project",
            "rotate-board-component",
            root.to_str().unwrap(),
            "--component",
            &component_uuid,
            "--rotation-deg",
            "45",
        ])
        .unwrap(),
    )
    .unwrap();
    assert!(export_native_project_gerber_copper_layer(&root, 1, &artifact).is_err());
    assert_eq!(std::fs::read_to_string(&artifact).unwrap(), cam);
    let _ = std::fs::remove_dir_all(&root);
}
