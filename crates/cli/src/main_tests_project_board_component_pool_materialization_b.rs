use super::main_tests_project_board_component_pool_materialization::*;
use super::*;
use eda_engine::ir::geometry::{Arc, Point, Polygon};
use eda_engine::pool::{Package, Primitive};
use std::collections::{HashMap, HashSet};

#[test]
fn project_board_component_materialization_prefers_lower_priority_pool_package_match() {
    let root = unique_project_root("datum-eda-cli-project-board-component-pool-priority");
    create_native_project(
        &root,
        Some("Board Component Pool Priority Demo".to_string()),
    )
    .expect("initial scaffold should succeed");
    configure_native_project_for_pool_materialization(
        &root,
        serde_json::json!([
            { "path": "pool-high", "priority": 1 },
            { "path": "pool-low", "priority": 2 }
        ]),
        silkscreen_stackup(21),
    );

    let package_uuid = Uuid::new_v4();
    write_pool_package(
        &root.join("pool-high"),
        &Package {
            uuid: package_uuid,
            name: "PKG-HIGH".to_string(),
            package_family: None,
            package_code: None,
            mounting_type: None,
            body_dimensions: None,
            terminals: HashMap::new(),
            pads: HashMap::new(),
            courtyard: Polygon {
                vertices: vec![],
                closed: true,
            },
            silkscreen: vec![Primitive::Line {
                from: Point { x: 0, y: 0 },
                to: Point { x: 1_000_000, y: 0 },
                width: 100_000,
            }],
            models_3d: Vec::new(),
            body_height_nm: None,
            body_height_mounted_nm: None,
            tags: HashSet::new(),
        },
    );
    write_pool_package(
        &root.join("pool-low"),
        &Package {
            uuid: package_uuid,
            name: "PKG-LOW".to_string(),
            package_family: None,
            package_code: None,
            mounting_type: None,
            body_dimensions: None,
            terminals: HashMap::new(),
            pads: HashMap::new(),
            courtyard: Polygon {
                vertices: vec![],
                closed: true,
            },
            silkscreen: vec![Primitive::Circle {
                center: Point {
                    x: 300_000,
                    y: 400_000,
                },
                radius: 200_000,
                width: 80_000,
            }],
            models_3d: Vec::new(),
            body_height_nm: None,
            body_height_mounted_nm: None,
            tags: HashSet::new(),
        },
    );

    let component_uuid = place_component(&root, Uuid::new_v4(), package_uuid);
    let board_json = root.join("board/board.json");
    let board: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&board_json).expect("board should read"))
            .expect("board should parse");
    assert_eq!(
        board["component_silkscreen"][&component_uuid]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        board["component_silkscreen_circles"][&component_uuid]
            .as_array()
            .unwrap()
            .len(),
        0
    );

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_board_component_materialization_supports_absolute_pool_paths() {
    let root = unique_project_root("datum-eda-cli-project-board-component-pool-abs");
    create_native_project(
        &root,
        Some("Board Component Pool Absolute Demo".to_string()),
    )
    .expect("initial scaffold should succeed");
    let external_pool_root =
        unique_project_root("datum-eda-cli-project-board-component-external-pool");
    configure_native_project_for_pool_materialization(
        &root,
        serde_json::json!([{ "path": external_pool_root.to_str().unwrap(), "priority": 1 }]),
        silkscreen_stackup(21),
    );

    let package_uuid = Uuid::new_v4();
    write_pool_package(
        &external_pool_root,
        &Package {
            uuid: package_uuid,
            name: "PKG-ABS".to_string(),
            package_family: None,
            package_code: None,
            mounting_type: None,
            body_dimensions: None,
            terminals: HashMap::new(),
            pads: HashMap::new(),
            courtyard: Polygon {
                vertices: vec![],
                closed: true,
            },
            silkscreen: vec![Primitive::Polygon {
                polygon: Polygon {
                    vertices: vec![
                        Point { x: 0, y: 0 },
                        Point { x: 300_000, y: 0 },
                        Point {
                            x: 300_000,
                            y: 300_000,
                        },
                    ],
                    closed: false,
                },
                width: 75_000,
            }],
            models_3d: Vec::new(),
            body_height_nm: None,
            body_height_mounted_nm: None,
            tags: HashSet::new(),
        },
    );

    let component_uuid = place_component(&root, Uuid::new_v4(), package_uuid);
    let board_json = root.join("board/board.json");
    let board: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&board_json).expect("board should read"))
            .expect("board should parse");
    assert_eq!(
        board["component_silkscreen_polylines"][&component_uuid]
            .as_array()
            .unwrap()
            .len(),
        1
    );

    let _ = std::fs::remove_dir_all(&root);
    let _ = std::fs::remove_dir_all(&external_pool_root);
}

#[test]
fn project_board_component_materialization_keeps_gerber_validate_and_compare_pool_independent() {
    let root = unique_project_root("datum-eda-cli-project-board-component-pool-proof");
    create_native_project(&root, Some("Board Component Pool Proof Demo".to_string()))
        .expect("initial scaffold should succeed");
    configure_native_project_for_pool_materialization(
        &root,
        serde_json::json!([{ "path": "pool", "priority": 1 }]),
        silkscreen_stackup(21),
    );

    let package_uuid = Uuid::new_v4();
    let pool_root = root.join("pool");
    write_pool_package(
        &pool_root,
        &Package {
            uuid: package_uuid,
            name: "PKG-PROOF".to_string(),
            package_family: None,
            package_code: None,
            mounting_type: None,
            body_dimensions: None,
            terminals: HashMap::new(),
            pads: HashMap::new(),
            courtyard: Polygon {
                vertices: vec![
                    Point { x: 0, y: 0 },
                    Point { x: 1_000_000, y: 0 },
                    Point {
                        x: 1_000_000,
                        y: 800_000,
                    },
                    Point { x: 0, y: 800_000 },
                ],
                closed: true,
            },
            silkscreen: vec![
                Primitive::Line {
                    from: Point { x: 0, y: 0 },
                    to: Point { x: 800_000, y: 0 },
                    width: 100_000,
                },
                Primitive::Arc {
                    arc: Arc {
                        center: Point {
                            x: 1_200_000,
                            y: 400_000,
                        },
                        radius: 200_000,
                        start_angle: 0,
                        end_angle: 900,
                    },
                    width: 80_000,
                },
                Primitive::Circle {
                    center: Point {
                        x: 900_000,
                        y: 900_000,
                    },
                    radius: 150_000,
                    width: 70_000,
                },
                Primitive::Polygon {
                    polygon: Polygon {
                        vertices: vec![
                            Point { x: 1_500_000, y: 0 },
                            Point { x: 1_900_000, y: 0 },
                            Point {
                                x: 1_700_000,
                                y: 300_000,
                            },
                        ],
                        closed: true,
                    },
                    width: 60_000,
                },
                Primitive::Polygon {
                    polygon: Polygon {
                        vertices: vec![
                            Point { x: 2_100_000, y: 0 },
                            Point {
                                x: 2_400_000,
                                y: 200_000,
                            },
                            Point { x: 2_700_000, y: 0 },
                        ],
                        closed: false,
                    },
                    width: 50_000,
                },
            ],
            models_3d: Vec::new(),
            body_height_nm: None,
            body_height_mounted_nm: None,
            tags: HashSet::new(),
        },
    );

    let _component_uuid = place_component(&root, Uuid::new_v4(), package_uuid);
    let gerber_path = root.join("top-silk-proof.gbr");
    let export_report = export_silkscreen_layer(&root, 21, &gerber_path);
    assert_eq!(export_report["component_stroke_count"], 1);
    assert_eq!(export_report["component_arc_count"], 1);
    assert_eq!(export_report["component_circle_count"], 1);
    assert_eq!(export_report["component_polygon_count"], 1);
    assert_eq!(export_report["component_polyline_count"], 1);
    let mechanical_gerber_path = root.join("mech-proof.gbr");
    let mechanical_export_report = export_mechanical_layer(&root, 41, &mechanical_gerber_path);
    assert_eq!(mechanical_export_report["component_polygon_count"], 1);

    std::fs::remove_file(
        pool_root
            .join("packages")
            .join(format!("{}.json", package_uuid)),
    )
    .expect("package file should delete");

    let (validate_report, validate_exit) = validate_silkscreen_layer(&root, 21, &gerber_path);
    assert_eq!(validate_exit, 0);
    assert_eq!(validate_report["matches_expected"], true);
    assert_eq!(validate_report["component_stroke_count"], 1);
    assert_eq!(validate_report["component_arc_count"], 1);
    assert_eq!(validate_report["component_circle_count"], 1);
    assert_eq!(validate_report["component_polygon_count"], 1);
    assert_eq!(validate_report["component_polyline_count"], 1);

    let compare_report = compare_silkscreen_layer(&root, 21, &gerber_path);
    assert_eq!(compare_report["missing_count"], 0);
    assert_eq!(compare_report["extra_count"], 0);
    assert_eq!(compare_report["expected_component_stroke_count"], 1);
    assert_eq!(compare_report["expected_component_arc_count"], 1);
    assert_eq!(compare_report["expected_component_circle_count"], 1);
    assert_eq!(compare_report["expected_component_polygon_count"], 1);
    assert_eq!(compare_report["expected_component_polyline_count"], 1);

    let (mechanical_validate_report, mechanical_validate_exit) =
        validate_mechanical_layer(&root, 41, &mechanical_gerber_path);
    assert_eq!(mechanical_validate_exit, 0);
    assert_eq!(mechanical_validate_report["matches_expected"], true);
    assert_eq!(mechanical_validate_report["component_polygon_count"], 1);

    let mechanical_compare_report = compare_mechanical_layer(&root, 41, &mechanical_gerber_path);
    assert_eq!(mechanical_compare_report["missing_count"], 0);
    assert_eq!(mechanical_compare_report["extra_count"], 0);
    assert_eq!(
        mechanical_compare_report["expected_component_polygon_count"],
        1
    );

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_board_component_materialization_replays_pool_package_and_padstack() {
    let root = unique_project_root("datum-eda-cli-project-board-component-pool-replay");
    create_native_project(&root, Some("Board Component Pool Replay Demo".to_string()))
        .expect("initial scaffold should succeed");
    configure_native_project_for_pool_materialization(
        &root,
        serde_json::json!([{ "path": "pool", "priority": 1 }]),
        silkscreen_stackup(21),
    );

    let padstack_uuid = Uuid::new_v4();
    let package_uuid = Uuid::new_v4();
    let pad_uuid = Uuid::new_v4();
    let padstack = padstack_uuid.to_string();
    let package = package_uuid.to_string();
    let pad = pad_uuid.to_string();
    execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "create-pool-padstack",
            root.to_str().unwrap(),
            "--padstack",
            &padstack,
            "--name",
            "JournalPad",
            "--aperture",
            "circle",
            "--diameter-nm",
            "1200000",
            "--drill-nm",
            "600000",
        ])
        .expect("CLI should parse"),
    )
    .expect("pool padstack create should succeed");
    execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "create-pool-package",
            root.to_str().unwrap(),
            "--package",
            &package,
            "--name",
            "JournalPackage",
            "--pad",
            &pad,
            "--padstack",
            &padstack,
            "--pad-name",
            "1",
            "--x-nm",
            "1000",
            "--y-nm",
            "2000",
            "--layer",
            "1",
        ])
        .expect("CLI should parse"),
    )
    .expect("pool package create should succeed");

    std::fs::remove_file(root.join(format!("pool/packages/{package_uuid}.json")))
        .expect("promoted package file should delete");
    std::fs::remove_file(root.join(format!("pool/padstacks/{padstack_uuid}.json")))
        .expect("promoted padstack file should delete");

    let component_uuid = place_component(&root, Uuid::new_v4(), package_uuid);
    let board_json = root.join("board/board.json");
    let board: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&board_json).expect("board should read"))
            .expect("board should parse");
    let persisted_pads = board["component_pads"][&component_uuid]
        .as_array()
        .expect("resolver-replayed component pads should exist");
    assert_eq!(persisted_pads.len(), 1);
    assert_eq!(persisted_pads[0]["uuid"], pad_uuid.to_string());
    assert_eq!(persisted_pads[0]["name"], "1");
    assert_eq!(persisted_pads[0]["padstack"], padstack_uuid.to_string());
    assert_eq!(persisted_pads[0]["position"]["x"], 1000);
    assert_eq!(persisted_pads[0]["position"]["y"], 2000);
    assert_eq!(persisted_pads[0]["shape"], "circle");
    assert_eq!(persisted_pads[0]["diameter_nm"], 1200000);
    assert_eq!(persisted_pads[0]["drill_nm"], 600000);

    let _ = std::fs::remove_dir_all(&root);
}
