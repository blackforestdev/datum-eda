use super::*;
use eda_engine::ir::geometry::{Arc, Point, Polygon};
use eda_engine::ir::serialization::to_json_deterministic;
use eda_engine::pool::{ModelRef, Package, Padstack, PadstackAperture, Primitive};
use std::collections::{HashMap, HashSet};

fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}", label, Uuid::new_v4()))
}

fn write_pool_package(pool_root: &Path, package: &Package) {
    let packages_dir = pool_root.join("packages");
    std::fs::create_dir_all(&packages_dir).expect("packages dir should exist");
    std::fs::write(
        packages_dir.join(format!("{}.json", package.uuid)),
        format!(
            "{}\n",
            to_json_deterministic(package).expect("package serialization should succeed")
        ),
    )
    .expect("package file should write");
}

fn write_pool_padstack(pool_root: &Path, padstack: &Padstack) {
    let padstacks_dir = pool_root.join("padstacks");
    std::fs::create_dir_all(&padstacks_dir).expect("padstacks dir should exist");
    std::fs::write(
        padstacks_dir.join(format!("{}.json", padstack.uuid)),
        format!(
            "{}\n",
            to_json_deterministic(padstack).expect("padstack serialization should succeed")
        ),
    )
    .expect("padstack file should write");
}

fn configure_native_project_for_pool_materialization(
    root: &Path,
    pools: serde_json::Value,
    stackup: serde_json::Value,
) {
    let project_json = root.join("project.json");
    let mut manifest: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&project_json).expect("project manifest should read"),
    )
    .expect("project manifest should parse");
    manifest["pools"] = pools;
    std::fs::write(
        &project_json,
        format!(
            "{}\n",
            to_json_deterministic(&manifest).expect("manifest serialization should succeed")
        ),
    )
    .expect("project manifest should write");

    let board_json = root.join("board/board.json");
    let mut board: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&board_json).expect("board should read"))
            .expect("board should parse");
    board["stackup"] = stackup;
    std::fs::write(
        &board_json,
        format!(
            "{}\n",
            to_json_deterministic(&board).expect("board serialization should succeed")
        ),
    )
    .expect("board should write");
}

fn silkscreen_stackup(top_silk_id: i32) -> serde_json::Value {
    serde_json::json!({
        "layers": [
            { "id": 1, "name": "Top Copper", "layer_type": "Copper", "thickness_nm": 35000 },
            { "id": top_silk_id, "name": "Top Silk", "layer_type": "Silkscreen", "thickness_nm": 0 },
            { "id": 41, "name": "Mechanical 41", "layer_type": "Mechanical", "thickness_nm": 0 }
        ]
    })
}

fn place_component(root: &Path, part_uuid: Uuid, package_uuid: Uuid) -> String {
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
        "2000000",
        "--y-nm",
        "3000000",
        "--layer",
        "1",
    ])
    .expect("CLI should parse");
    let placed_output = execute(place_cli).expect("place should succeed");
    let placed: serde_json::Value =
        serde_json::from_str(&placed_output).expect("place output should parse");
    placed["component_uuid"].as_str().unwrap().to_string()
}

fn export_silkscreen_layer(root: &Path, layer: i32, gerber_path: &Path) -> serde_json::Value {
    let cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "export-gerber-silkscreen-layer",
        root.to_str().unwrap(),
        "--layer",
        &layer.to_string(),
        "--out",
        gerber_path.to_str().unwrap(),
    ])
    .expect("CLI should parse");
    let output = execute(cli).expect("silkscreen export should succeed");
    serde_json::from_str(&output).expect("report JSON")
}

fn validate_silkscreen_layer(
    root: &Path,
    layer: i32,
    gerber_path: &Path,
) -> (serde_json::Value, i32) {
    let cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "validate-gerber-silkscreen-layer",
        root.to_str().unwrap(),
        "--layer",
        &layer.to_string(),
        "--gerber",
        gerber_path.to_str().unwrap(),
    ])
    .expect("CLI should parse");
    let (output, exit_code) = execute_with_exit_code(cli).expect("validation should run");
    (
        serde_json::from_str(&output).expect("report JSON"),
        exit_code,
    )
}

fn compare_silkscreen_layer(root: &Path, layer: i32, gerber_path: &Path) -> serde_json::Value {
    let cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "compare-gerber-silkscreen-layer",
        root.to_str().unwrap(),
        "--layer",
        &layer.to_string(),
        "--gerber",
        gerber_path.to_str().unwrap(),
    ])
    .expect("CLI should parse");
    let output = execute(cli).expect("compare should succeed");
    serde_json::from_str(&output).expect("report JSON")
}

fn export_mechanical_layer(root: &Path, layer: i32, gerber_path: &Path) -> serde_json::Value {
    let cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "export-gerber-mechanical-layer",
        root.to_str().unwrap(),
        "--layer",
        &layer.to_string(),
        "--out",
        gerber_path.to_str().unwrap(),
    ])
    .expect("CLI should parse");
    let output = execute(cli).expect("mechanical export should succeed");
    serde_json::from_str(&output).expect("report JSON")
}

fn validate_mechanical_layer(
    root: &Path,
    layer: i32,
    gerber_path: &Path,
) -> (serde_json::Value, i32) {
    let cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "validate-gerber-mechanical-layer",
        root.to_str().unwrap(),
        "--layer",
        &layer.to_string(),
        "--gerber",
        gerber_path.to_str().unwrap(),
    ])
    .expect("CLI should parse");
    let (output, exit_code) = execute_with_exit_code(cli).expect("validation should run");
    (
        serde_json::from_str(&output).expect("report JSON"),
        exit_code,
    )
}

fn compare_mechanical_layer(root: &Path, layer: i32, gerber_path: &Path) -> serde_json::Value {
    let cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "compare-gerber-mechanical-layer",
        root.to_str().unwrap(),
        "--layer",
        &layer.to_string(),
        "--gerber",
        gerber_path.to_str().unwrap(),
    ])
    .expect("CLI should parse");
    let output = execute(cli).expect("compare should succeed");
    serde_json::from_str(&output).expect("report JSON")
}

#[test]
fn project_board_component_materializes_supported_pool_graphics_into_persisted_board_state() {
    let root = unique_project_root("datum-eda-cli-project-board-component-pool");
    create_native_project(&root, Some("Board Component Pool Demo".to_string()))
        .expect("initial scaffold should succeed");
    configure_native_project_for_pool_materialization(
        &root,
        serde_json::json!([{ "path": "pool", "priority": 1 }]),
        silkscreen_stackup(21),
    );
    let board_json = root.join("board/board.json");

    let package_uuid = Uuid::new_v4();
    let replacement_package_uuid = Uuid::new_v4();
    let padstack_circle_uuid = Uuid::new_v4();
    let padstack_rect_uuid = Uuid::new_v4();
    let padstack_unknown_uuid = Uuid::new_v4();
    let replacement_padstack_uuid = Uuid::new_v4();
    let pool_root = root.join("pool");
    write_pool_padstack(
        &pool_root,
        &Padstack {
            uuid: padstack_circle_uuid,
            name: "round-0.5mm".to_string(),
            aperture: Some(PadstackAperture::Circle { diameter_nm: 500_000 }),
            drill_nm: Some(300_000),
        },
    );
    write_pool_padstack(
        &pool_root,
        &Padstack {
            uuid: padstack_rect_uuid,
            name: "rect-0.8x0.4mm".to_string(),
            aperture: Some(PadstackAperture::Rect {
                width_nm: 800_000,
                height_nm: 400_000,
            }),
            drill_nm: Some(350_000),
        },
    );
    write_pool_padstack(
        &pool_root,
        &Padstack {
            uuid: replacement_padstack_uuid,
            name: "round-0.6mm".to_string(),
            aperture: Some(PadstackAperture::Circle { diameter_nm: 600_000 }),
            drill_nm: Some(325_000),
        },
    );
    write_pool_package(
        &pool_root,
        &Package {
            uuid: package_uuid,
            name: "PKG-A".to_string(),
            pads: HashMap::from([
                (
                    Uuid::new_v4(),
                    eda_engine::pool::Pad {
                        uuid: Uuid::new_v4(),
                        name: "1".to_string(),
                        position: Point { x: 100_000, y: 200_000 },
                        padstack: padstack_circle_uuid,
                        layer: 1,
                    },
                ),
                (
                    Uuid::new_v4(),
                    eda_engine::pool::Pad {
                        uuid: Uuid::new_v4(),
                        name: "2".to_string(),
                        position: Point { x: 300_000, y: 400_000 },
                        padstack: padstack_rect_uuid,
                        layer: 1,
                    },
                ),
                (
                    Uuid::new_v4(),
                    eda_engine::pool::Pad {
                        uuid: Uuid::new_v4(),
                        name: "3".to_string(),
                        position: Point { x: 500_000, y: 700_000 },
                        padstack: padstack_unknown_uuid,
                        layer: 1,
                    },
                ),
            ]),
            courtyard: Polygon {
                vertices: vec![
                    Point { x: 0, y: 0 },
                    Point { x: 2_000_000, y: 0 },
                    Point {
                        x: 2_000_000,
                        y: 1_200_000,
                    },
                    Point { x: 0, y: 1_200_000 },
                ],
                closed: true,
            },
            silkscreen: vec![
                Primitive::Line {
                    from: Point { x: 0, y: 0 },
                    to: Point { x: 1_000_000, y: 0 },
                    width: 120_000,
                },
                Primitive::Rect {
                    min: Point { x: 0, y: 0 },
                    max: Point {
                        x: 500_000,
                        y: 300_000,
                    },
                    width: 100_000,
                },
                Primitive::Circle {
                    center: Point {
                        x: 300_000,
                        y: 600_000,
                    },
                    radius: 200_000,
                    width: 90_000,
                },
                Primitive::Polygon {
                    polygon: Polygon {
                        vertices: vec![
                            Point { x: 700_000, y: 0 },
                            Point {
                                x: 1_000_000,
                                y: 200_000,
                            },
                            Point {
                                x: 900_000,
                                y: 500_000,
                            },
                        ],
                        closed: true,
                    },
                    width: 80_000,
                },
                Primitive::Polygon {
                    polygon: Polygon {
                        vertices: vec![
                            Point { x: 0, y: 800_000 },
                            Point {
                                x: 400_000,
                                y: 900_000,
                            },
                            Point {
                                x: 700_000,
                                y: 800_000,
                            },
                        ],
                        closed: false,
                    },
                    width: 70_000,
                },
                Primitive::Arc {
                    arc: Arc {
                        center: Point {
                            x: 1_200_000,
                            y: 600_000,
                        },
                        radius: 300_000,
                        start_angle: 0,
                        end_angle: 900,
                    },
                    width: 60_000,
                },
                Primitive::Text {
                    text: "NOCLAIM".to_string(),
                    position: Point { x: 0, y: 1_200_000 },
                    rotation: 0,
                },
            ],
            models_3d: vec![
                ModelRef {
                    path: "models/pkg-a.step".to_string(),
                    transform: None,
                },
                ModelRef {
                    path: "models/pkg-a-alt.wrl".to_string(),
                    transform: Some(serde_json::json!({
                        "translate": [1, 2, 3]
                    })),
                },
            ],
            tags: HashSet::new(),
        },
    );
    write_pool_package(
        &pool_root,
        &Package {
            uuid: replacement_package_uuid,
            name: "PKG-B".to_string(),
            pads: HashMap::from([(
                Uuid::new_v4(),
                eda_engine::pool::Pad {
                    uuid: Uuid::new_v4(),
                    name: "A".to_string(),
                    position: Point { x: 500_000, y: 600_000 },
                    padstack: replacement_padstack_uuid,
                    layer: 1,
                },
            )]),
            courtyard: Polygon {
                vertices: vec![
                    Point { x: 0, y: 0 },
                    Point { x: 1_500_000, y: 0 },
                    Point {
                        x: 1_500_000,
                        y: 900_000,
                    },
                    Point { x: 0, y: 900_000 },
                ],
                closed: true,
            },
            silkscreen: vec![Primitive::Line {
                from: Point { x: 0, y: 0 },
                to: Point { x: 2_000_000, y: 0 },
                width: 150_000,
            }],
            models_3d: vec![ModelRef {
                path: "models/pkg-b.step".to_string(),
                transform: None,
            }],
            tags: HashSet::new(),
        },
    );

    let part_uuid = Uuid::new_v4();
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
        "2000000",
        "--y-nm",
        "3000000",
        "--layer",
        "1",
    ])
    .expect("CLI should parse");
    let placed_output = execute(place_cli).expect("place should succeed");
    let placed: serde_json::Value =
        serde_json::from_str(&placed_output).expect("place output should parse");
    let component_uuid = placed["component_uuid"].as_str().unwrap().to_string();
    assert_eq!(placed["has_persisted_component_silkscreen"], true);
    assert_eq!(placed["has_persisted_component_mechanical"], true);
    assert_eq!(placed["persisted_component_silkscreen_text_count"], 0);
    assert_eq!(placed["persisted_component_silkscreen_line_count"], 1);
    assert_eq!(placed["persisted_component_silkscreen_arc_count"], 1);
    assert_eq!(placed["persisted_component_silkscreen_circle_count"], 1);
    assert_eq!(placed["persisted_component_silkscreen_polygon_count"], 2);
    assert_eq!(placed["persisted_component_silkscreen_polyline_count"], 1);
    assert_eq!(placed["persisted_component_mechanical_polygon_count"], 1);
    assert_eq!(placed["has_persisted_component_pads"], true);
    assert_eq!(placed["persisted_component_pad_count"], 3);
    assert_eq!(placed["has_persisted_component_models_3d"], true);
    assert_eq!(placed["persisted_component_model_3d_count"], 2);

    let board_value: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&board_json).expect("board should read"))
            .expect("board should parse");
    let persisted_pads = board_value["component_pads"][&component_uuid]
        .as_array()
        .expect("persisted component pads should exist");
    assert_eq!(persisted_pads.len(), 3);
    assert_eq!(persisted_pads[0]["name"], "1");
    assert_eq!(persisted_pads[0]["shape"], "circle");
    assert_eq!(persisted_pads[0]["drill_nm"], 300000);
    assert_eq!(persisted_pads[0]["diameter_nm"], 500000);
    assert_eq!(persisted_pads[1]["name"], "2");
    assert_eq!(persisted_pads[1]["shape"], "rect");
    assert_eq!(persisted_pads[1]["drill_nm"], 350000);
    assert_eq!(persisted_pads[1]["width_nm"], 800000);
    assert_eq!(persisted_pads[1]["height_nm"], 400000);
    assert!(persisted_pads[2]["drill_nm"].is_null());
    assert!(persisted_pads[2]["shape"].is_null());
    assert_eq!(persisted_pads[2]["diameter_nm"], 0);

    let board_components_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "query",
            root.to_str().unwrap(),
            "board-components",
        ])
        .expect("CLI should parse"),
    )
    .expect("board components query should succeed");
    let board_components: Vec<serde_json::Value> =
        serde_json::from_str(&board_components_output).expect("query output should parse");
    assert_eq!(board_components.len(), 1);
    assert_eq!(board_components[0]["uuid"], component_uuid);
    assert_eq!(
        board_components[0]["has_persisted_component_silkscreen"],
        true
    );
    assert_eq!(
        board_components[0]["has_persisted_component_mechanical"],
        true
    );
    assert_eq!(
        board_components[0]["persisted_component_mechanical_polygon_count"],
        1
    );
    assert_eq!(board_components[0]["has_persisted_component_pads"], true);
    assert_eq!(board_components[0]["persisted_component_pad_count"], 3);
    assert_eq!(board_components[0]["has_persisted_component_models_3d"], true);
    assert_eq!(board_components[0]["persisted_component_model_3d_count"], 2);

    let summary_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "query",
            root.to_str().unwrap(),
            "summary",
        ])
        .expect("CLI should parse"),
    )
    .expect("summary query should succeed");
    let summary: serde_json::Value =
        serde_json::from_str(&summary_output).expect("summary output should parse");
    assert_eq!(summary["board"]["components_with_persisted_silkscreen"], 1);
    assert_eq!(summary["board"]["components_with_persisted_mechanical"], 1);
    assert_eq!(summary["board"]["components_with_persisted_pads"], 1);
    assert_eq!(summary["board"]["components_with_persisted_models_3d"], 1);
    assert_eq!(
        summary["board"]["persisted_component_mechanical_polygons"],
        1
    );
    assert_eq!(summary["board"]["persisted_component_pads"], 3);
    assert_eq!(summary["board"]["persisted_component_models_3d"], 2);

    let inspect_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "inspect",
            root.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("inspect should succeed");
    let inspect: serde_json::Value =
        serde_json::from_str(&inspect_output).expect("inspect output should parse");
    assert_eq!(inspect["board_components_with_persisted_silkscreen"], 1);
    assert_eq!(inspect["board_components_with_persisted_mechanical"], 1);
    assert_eq!(inspect["board_components_with_persisted_pads"], 1);
    assert_eq!(inspect["board_components_with_persisted_models_3d"], 1);
    assert_eq!(inspect["persisted_component_mechanical_polygons"], 1);
    assert_eq!(inspect["persisted_component_pads"], 3);
    assert_eq!(inspect["persisted_component_models_3d"], 2);

    let replacement_part_uuid = Uuid::new_v4();
    let set_part_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "set-board-component-part",
            root.to_str().unwrap(),
            "--component",
            &component_uuid,
            "--part",
            &replacement_part_uuid.to_string(),
        ])
        .expect("CLI should parse"),
    )
    .expect("set part should succeed");
    let set_part_report: serde_json::Value =
        serde_json::from_str(&set_part_output).expect("set part output should parse");
    assert_eq!(set_part_report["has_persisted_component_silkscreen"], true);
    assert_eq!(set_part_report["has_persisted_component_mechanical"], true);
    assert_eq!(set_part_report["has_persisted_component_pads"], true);
    assert_eq!(set_part_report["has_persisted_component_models_3d"], true);
    assert_eq!(
        set_part_report["persisted_component_silkscreen_line_count"],
        1
    );
    assert_eq!(
        set_part_report["persisted_component_silkscreen_arc_count"],
        1
    );
    assert_eq!(
        set_part_report["persisted_component_silkscreen_circle_count"],
        1
    );
    assert_eq!(
        set_part_report["persisted_component_silkscreen_polygon_count"],
        2
    );
    assert_eq!(
        set_part_report["persisted_component_silkscreen_polyline_count"],
        1
    );
    assert_eq!(
        set_part_report["persisted_component_mechanical_polygon_count"],
        1
    );
    assert_eq!(set_part_report["persisted_component_pad_count"], 3);
    assert_eq!(set_part_report["persisted_component_model_3d_count"], 2);

    let move_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "move-board-component",
            root.to_str().unwrap(),
            "--component",
            &component_uuid,
            "--x-nm",
            "2500000",
            "--y-nm",
            "3500000",
        ])
        .expect("CLI should parse"),
    )
    .expect("move should succeed");
    let move_report: serde_json::Value =
        serde_json::from_str(&move_output).expect("move output should parse");
    assert_eq!(move_report["has_persisted_component_silkscreen"], true);
    assert_eq!(move_report["has_persisted_component_mechanical"], true);
    assert_eq!(move_report["has_persisted_component_pads"], true);
    assert_eq!(move_report["has_persisted_component_models_3d"], true);
    assert_eq!(move_report["persisted_component_silkscreen_line_count"], 1);
    assert_eq!(move_report["persisted_component_silkscreen_arc_count"], 1);
    assert_eq!(
        move_report["persisted_component_silkscreen_circle_count"],
        1
    );
    assert_eq!(
        move_report["persisted_component_silkscreen_polygon_count"],
        2
    );
    assert_eq!(
        move_report["persisted_component_silkscreen_polyline_count"],
        1
    );
    assert_eq!(
        move_report["persisted_component_mechanical_polygon_count"],
        1
    );
    assert_eq!(move_report["persisted_component_pad_count"], 3);
    assert_eq!(move_report["persisted_component_model_3d_count"], 2);

    let rotate_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "rotate-board-component",
            root.to_str().unwrap(),
            "--component",
            &component_uuid,
            "--rotation-deg",
            "180",
        ])
        .expect("CLI should parse"),
    )
    .expect("rotate should succeed");
    let rotate_report: serde_json::Value =
        serde_json::from_str(&rotate_output).expect("rotate output should parse");
    assert_eq!(rotate_report["has_persisted_component_silkscreen"], true);
    assert_eq!(rotate_report["has_persisted_component_mechanical"], true);
    assert_eq!(
        rotate_report["persisted_component_silkscreen_line_count"],
        1
    );
    assert_eq!(rotate_report["persisted_component_silkscreen_arc_count"], 1);
    assert_eq!(
        rotate_report["persisted_component_silkscreen_circle_count"],
        1
    );
    assert_eq!(
        rotate_report["persisted_component_silkscreen_polygon_count"],
        2
    );
    assert_eq!(
        rotate_report["persisted_component_silkscreen_polyline_count"],
        1
    );
    assert_eq!(
        rotate_report["persisted_component_mechanical_polygon_count"],
        1
    );

    let lock_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "set-board-component-locked",
            root.to_str().unwrap(),
            "--component",
            &component_uuid,
        ])
        .expect("CLI should parse"),
    )
    .expect("lock should succeed");
    let lock_report: serde_json::Value =
        serde_json::from_str(&lock_output).expect("lock output should parse");
    assert_eq!(lock_report["has_persisted_component_silkscreen"], true);
    assert_eq!(lock_report["has_persisted_component_mechanical"], true);
    assert_eq!(lock_report["persisted_component_silkscreen_line_count"], 1);
    assert_eq!(lock_report["persisted_component_silkscreen_arc_count"], 1);
    assert_eq!(
        lock_report["persisted_component_silkscreen_circle_count"],
        1
    );
    assert_eq!(
        lock_report["persisted_component_silkscreen_polygon_count"],
        2
    );
    assert_eq!(
        lock_report["persisted_component_silkscreen_polyline_count"],
        1
    );
    assert_eq!(
        lock_report["persisted_component_mechanical_polygon_count"],
        1
    );

    let unlock_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "clear-board-component-locked",
            root.to_str().unwrap(),
            "--component",
            &component_uuid,
        ])
        .expect("CLI should parse"),
    )
    .expect("unlock should succeed");
    let unlock_report: serde_json::Value =
        serde_json::from_str(&unlock_output).expect("unlock output should parse");
    assert_eq!(unlock_report["has_persisted_component_silkscreen"], true);
    assert_eq!(unlock_report["has_persisted_component_mechanical"], true);
    assert_eq!(
        unlock_report["persisted_component_silkscreen_line_count"],
        1
    );
    assert_eq!(unlock_report["persisted_component_silkscreen_arc_count"], 1);
    assert_eq!(
        unlock_report["persisted_component_silkscreen_circle_count"],
        1
    );
    assert_eq!(
        unlock_report["persisted_component_silkscreen_polygon_count"],
        2
    );
    assert_eq!(
        unlock_report["persisted_component_silkscreen_polyline_count"],
        1
    );
    assert_eq!(
        unlock_report["persisted_component_mechanical_polygon_count"],
        1
    );

    let board_components_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "query",
            root.to_str().unwrap(),
            "board-components",
        ])
        .expect("CLI should parse"),
    )
    .expect("board components query should succeed");
    let board_components: Vec<serde_json::Value> =
        serde_json::from_str(&board_components_output).expect("query output should parse");
    assert_eq!(board_components.len(), 1);
    assert_eq!(board_components[0]["uuid"], component_uuid);
    assert_eq!(
        board_components[0]["has_persisted_component_silkscreen"],
        true
    );
    assert_eq!(
        board_components[0]["has_persisted_component_mechanical"],
        true
    );
    assert_eq!(
        board_components[0]["persisted_component_silkscreen_line_count"],
        1
    );
    assert_eq!(
        board_components[0]["persisted_component_silkscreen_arc_count"],
        1
    );
    assert_eq!(
        board_components[0]["persisted_component_silkscreen_circle_count"],
        1
    );
    assert_eq!(
        board_components[0]["persisted_component_silkscreen_polygon_count"],
        2
    );
    assert_eq!(
        board_components[0]["persisted_component_silkscreen_polyline_count"],
        1
    );
    assert_eq!(
        board_components[0]["persisted_component_mechanical_polygon_count"],
        1
    );
    let summary_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "query",
            root.to_str().unwrap(),
            "summary",
        ])
        .expect("CLI should parse"),
    )
    .expect("summary query should succeed");
    let summary: serde_json::Value =
        serde_json::from_str(&summary_output).expect("summary output should parse");
    assert_eq!(summary["board"]["components_with_persisted_silkscreen"], 1);
    assert_eq!(summary["board"]["components_with_persisted_mechanical"], 1);
    assert_eq!(summary["board"]["persisted_component_silkscreen_lines"], 1);
    assert_eq!(summary["board"]["persisted_component_silkscreen_arcs"], 1);
    assert_eq!(
        summary["board"]["persisted_component_silkscreen_circles"],
        1
    );
    assert_eq!(
        summary["board"]["persisted_component_silkscreen_polygons"],
        2
    );
    assert_eq!(
        summary["board"]["persisted_component_silkscreen_polylines"],
        1
    );
    assert_eq!(
        summary["board"]["persisted_component_mechanical_polygons"],
        1
    );

    let inspect_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "inspect",
            root.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("inspect should succeed");
    let inspect: serde_json::Value =
        serde_json::from_str(&inspect_output).expect("inspect output should parse");
    assert_eq!(inspect["board_components_with_persisted_silkscreen"], 1);
    assert_eq!(inspect["board_components_with_persisted_mechanical"], 1);
    assert_eq!(inspect["persisted_component_silkscreen_lines"], 1);
    assert_eq!(inspect["persisted_component_silkscreen_arcs"], 1);
    assert_eq!(inspect["persisted_component_silkscreen_circles"], 1);
    assert_eq!(inspect["persisted_component_silkscreen_polygons"], 2);
    assert_eq!(inspect["persisted_component_silkscreen_polylines"], 1);
    assert_eq!(inspect["persisted_component_mechanical_polygons"], 1);

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
        board["component_silkscreen_arcs"][&component_uuid]
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
        1
    );
    assert_eq!(
        board["component_silkscreen_polygons"][&component_uuid]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    assert_eq!(
        board["component_silkscreen_polylines"][&component_uuid]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        board["component_silkscreen_texts"][&component_uuid]
            .as_array()
            .unwrap()
            .len(),
        0
    );
    assert_eq!(
        board["component_mechanical_polygons"][&component_uuid]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        board["component_mechanical_polygons"][&component_uuid][0]["layer"].as_i64(),
        Some(41)
    );

    std::fs::write(
        pool_root
            .join("packages")
            .join(format!("{}.json", package_uuid)),
        "{}\n",
    )
    .expect("mutated package should write");

    let gerber_path = root.join("top-silk.gbr");
    let export_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "export-gerber-silkscreen-layer",
        root.to_str().unwrap(),
        "--layer",
        "21",
        "--out",
        gerber_path.to_str().unwrap(),
    ])
    .expect("CLI should parse");
    let output = execute(export_cli).expect("export should succeed from persisted board state");
    let report: serde_json::Value = serde_json::from_str(&output).expect("report JSON");
    assert_eq!(report["component_text_count"], 0);
    assert_eq!(report["component_stroke_count"], 1);
    assert_eq!(report["component_arc_count"], 1);
    assert_eq!(report["component_circle_count"], 1);
    assert_eq!(report["component_polygon_count"], 2);
    assert_eq!(report["component_polyline_count"], 1);

    let set_package_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "set-board-component-package",
        root.to_str().unwrap(),
        "--component",
        &component_uuid,
        "--package",
        &replacement_package_uuid.to_string(),
    ])
    .expect("CLI should parse");
    let set_package_output = execute(set_package_cli).expect("package reassignment should succeed");
    let set_package_report: serde_json::Value =
        serde_json::from_str(&set_package_output).expect("set package output should parse");
    assert_eq!(
        set_package_report["has_persisted_component_silkscreen"],
        true
    );
    assert_eq!(
        set_package_report["has_persisted_component_mechanical"],
        true
    );
    assert_eq!(set_package_report["has_persisted_component_pads"], true);
    assert_eq!(set_package_report["persisted_component_pad_count"], 1);
    assert_eq!(set_package_report["has_persisted_component_models_3d"], true);
    assert_eq!(set_package_report["persisted_component_model_3d_count"], 1);
    assert_eq!(
        set_package_report["persisted_component_silkscreen_text_count"],
        0
    );
    assert_eq!(
        set_package_report["persisted_component_silkscreen_line_count"],
        1
    );
    assert_eq!(
        set_package_report["persisted_component_silkscreen_arc_count"],
        0
    );
    assert_eq!(
        set_package_report["persisted_component_silkscreen_circle_count"],
        0
    );
    assert_eq!(
        set_package_report["persisted_component_silkscreen_polygon_count"],
        0
    );
    assert_eq!(
        set_package_report["persisted_component_silkscreen_polyline_count"],
        0
    );
    assert_eq!(
        set_package_report["persisted_component_mechanical_polygon_count"],
        1
    );

    let board_components_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "query",
            root.to_str().unwrap(),
            "board-components",
        ])
        .expect("CLI should parse"),
    )
    .expect("board components query should succeed");
    let board_components: Vec<serde_json::Value> =
        serde_json::from_str(&board_components_output).expect("query output should parse");
    assert_eq!(board_components[0]["persisted_component_pad_count"], 1);
    assert_eq!(board_components.len(), 1);
    assert_eq!(board_components[0]["uuid"], component_uuid);
    assert_eq!(
        board_components[0]["has_persisted_component_silkscreen"],
        true
    );
    assert_eq!(
        board_components[0]["has_persisted_component_mechanical"],
        true
    );
    assert_eq!(
        board_components[0]["persisted_component_silkscreen_line_count"],
        1
    );
    assert_eq!(
        board_components[0]["persisted_component_silkscreen_arc_count"],
        0
    );
    assert_eq!(
        board_components[0]["persisted_component_silkscreen_circle_count"],
        0
    );
    assert_eq!(
        board_components[0]["persisted_component_silkscreen_polygon_count"],
        0
    );
    assert_eq!(
        board_components[0]["persisted_component_silkscreen_polyline_count"],
        0
    );
    assert_eq!(
        board_components[0]["persisted_component_mechanical_polygon_count"],
        1
    );
    let replacement_board_value: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&board_json).expect("board should read"))
            .expect("board should parse");
    let replacement_pads = replacement_board_value["component_pads"][&component_uuid]
        .as_array()
        .expect("replacement component pads should exist");
    assert_eq!(replacement_pads.len(), 1);
    assert_eq!(replacement_pads[0]["shape"], "circle");
    assert_eq!(replacement_pads[0]["drill_nm"], 325000);
    assert_eq!(replacement_pads[0]["diameter_nm"], 600000);

    let summary_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "query",
            root.to_str().unwrap(),
            "summary",
        ])
        .expect("CLI should parse"),
    )
    .expect("summary query should succeed");
    let summary: serde_json::Value =
        serde_json::from_str(&summary_output).expect("summary output should parse");
    assert_eq!(summary["board"]["components_with_persisted_silkscreen"], 1);
    assert_eq!(summary["board"]["components_with_persisted_mechanical"], 1);
    assert_eq!(summary["board"]["persisted_component_silkscreen_lines"], 1);
    assert_eq!(summary["board"]["persisted_component_silkscreen_arcs"], 0);
    assert_eq!(
        summary["board"]["persisted_component_silkscreen_circles"],
        0
    );
    assert_eq!(
        summary["board"]["persisted_component_silkscreen_polygons"],
        0
    );
    assert_eq!(
        summary["board"]["persisted_component_silkscreen_polylines"],
        0
    );
    assert_eq!(
        summary["board"]["persisted_component_mechanical_polygons"],
        1
    );

    let inspect_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "inspect",
            root.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("inspect should succeed");
    let inspect: serde_json::Value =
        serde_json::from_str(&inspect_output).expect("inspect output should parse");
    assert_eq!(inspect["board_components_with_persisted_silkscreen"], 1);
    assert_eq!(inspect["board_components_with_persisted_mechanical"], 1);
    assert_eq!(inspect["persisted_component_silkscreen_lines"], 1);
    assert_eq!(inspect["persisted_component_silkscreen_arcs"], 0);
    assert_eq!(inspect["persisted_component_silkscreen_circles"], 0);
    assert_eq!(inspect["persisted_component_silkscreen_polygons"], 0);
    assert_eq!(inspect["persisted_component_silkscreen_polylines"], 0);
    assert_eq!(inspect["persisted_component_mechanical_polygons"], 1);

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
        board["component_silkscreen_arcs"][&component_uuid]
            .as_array()
            .unwrap()
            .len(),
        0
    );
    assert_eq!(
        board["component_silkscreen_circles"][&component_uuid]
            .as_array()
            .unwrap()
            .len(),
        0
    );
    assert_eq!(
        board["component_silkscreen_polygons"][&component_uuid]
            .as_array()
            .unwrap()
            .len(),
        0
    );
    assert_eq!(
        board["component_silkscreen_polylines"][&component_uuid]
            .as_array()
            .unwrap()
            .len(),
        0
    );
    assert_eq!(
        board["component_mechanical_polygons"][&component_uuid]
            .as_array()
            .unwrap()
            .len(),
        1
    );

    let output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "export-gerber-silkscreen-layer",
            root.to_str().unwrap(),
            "--layer",
            "21",
            "--out",
            gerber_path.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("export should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).expect("report JSON");
    assert_eq!(report["component_stroke_count"], 1);
    assert_eq!(report["component_arc_count"], 0);
    assert_eq!(report["component_circle_count"], 0);
    assert_eq!(report["component_polygon_count"], 0);
    assert_eq!(report["component_polyline_count"], 0);

    let delete_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "delete-board-component",
            root.to_str().unwrap(),
            "--component",
            &component_uuid,
        ])
        .expect("CLI should parse"),
    )
    .expect("delete board component should succeed");
    let delete_report: serde_json::Value =
        serde_json::from_str(&delete_output).expect("delete output should parse");
    assert_eq!(delete_report["action"], "delete_board_component");
    assert_eq!(delete_report["has_persisted_component_silkscreen"], true);
    assert_eq!(delete_report["has_persisted_component_mechanical"], true);
    assert_eq!(delete_report["has_persisted_component_pads"], true);
    assert_eq!(delete_report["persisted_component_pad_count"], 1);
    assert_eq!(delete_report["has_persisted_component_models_3d"], true);
    assert_eq!(delete_report["persisted_component_model_3d_count"], 1);
    assert_eq!(
        delete_report["persisted_component_silkscreen_line_count"],
        1
    );
    assert_eq!(delete_report["persisted_component_silkscreen_arc_count"], 0);
    assert_eq!(
        delete_report["persisted_component_silkscreen_circle_count"],
        0
    );
    assert_eq!(
        delete_report["persisted_component_silkscreen_polygon_count"],
        0
    );
    assert_eq!(
        delete_report["persisted_component_silkscreen_polyline_count"],
        0
    );
    assert_eq!(
        delete_report["persisted_component_mechanical_polygon_count"],
        1
    );

    let board_components_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "query",
            root.to_str().unwrap(),
            "board-components",
        ])
        .expect("CLI should parse"),
    )
    .expect("board components query should succeed");
    let board_components: Vec<serde_json::Value> =
        serde_json::from_str(&board_components_output).expect("query output should parse");
    assert!(board_components.is_empty());

    let summary_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "query",
            root.to_str().unwrap(),
            "summary",
        ])
        .expect("CLI should parse"),
    )
    .expect("summary query should succeed");
    let summary: serde_json::Value =
        serde_json::from_str(&summary_output).expect("summary output should parse");
    assert_eq!(summary["board"]["components_with_persisted_silkscreen"], 0);
    assert_eq!(summary["board"]["components_with_persisted_mechanical"], 0);
    assert_eq!(summary["board"]["persisted_component_silkscreen_lines"], 0);
    assert_eq!(
        summary["board"]["persisted_component_mechanical_polygons"],
        0
    );
    assert_eq!(summary["board"]["persisted_component_mechanical_lines"], 0);

    let inspect_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "inspect",
            root.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("inspect should succeed");
    let inspect: serde_json::Value =
        serde_json::from_str(&inspect_output).expect("inspect output should parse");
    assert_eq!(inspect["board_components_with_persisted_silkscreen"], 0);
    assert_eq!(inspect["board_components_with_persisted_mechanical"], 0);
    assert_eq!(inspect["persisted_component_silkscreen_lines"], 0);
    assert_eq!(inspect["persisted_component_mechanical_polygons"], 0);
    assert_eq!(inspect["persisted_component_mechanical_lines"], 0);

    let _ = std::fs::remove_dir_all(&root);
}

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
            tags: HashSet::new(),
        },
    );
    write_pool_package(
        &root.join("pool-low"),
        &Package {
            uuid: package_uuid,
            name: "PKG-LOW".to_string(),
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
