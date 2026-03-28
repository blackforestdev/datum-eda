use super::*;
use eda_engine::ir::geometry::Arc;
use eda_engine::ir::geometry::{Point, Polygon};
use eda_engine::ir::serialization::to_json_deterministic;
use eda_engine::pool::{Package, Primitive};
use std::collections::{HashMap, HashSet};

fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}", label, Uuid::new_v4()))
}

#[test]
fn project_query_board_component_silkscreen_reads_persisted_component_silkscreen() {
    let root = unique_project_root("datum-eda-cli-project-board-component-silkscreen");
    create_native_project(&root, Some("Board Component Silkscreen Demo".to_string()))
        .expect("initial scaffold should succeed");

    let pool_root = root.join("pool");
    std::fs::create_dir_all(pool_root.join("packages")).expect("packages dir should exist");

    let package_uuid = Uuid::new_v4();
    let package = Package {
        uuid: package_uuid,
        name: "PKG".to_string(),
        pads: HashMap::new(),
        courtyard: Polygon {
            vertices: Vec::new(),
            closed: true,
        },
        silkscreen: vec![
            Primitive::Line {
                from: Point {
                    x: 10_000,
                    y: 20_000,
                },
                to: Point {
                    x: 30_000,
                    y: 40_000,
                },
                width: 5_000,
            },
            Primitive::Arc {
                arc: Arc {
                    center: Point {
                        x: 50_000,
                        y: 60_000,
                    },
                    radius: 25_000,
                    start_angle: 0,
                    end_angle: 900,
                },
                width: 4_000,
            },
            Primitive::Circle {
                center: Point {
                    x: 70_000,
                    y: 80_000,
                },
                radius: 15_000,
                width: 3_000,
            },
            Primitive::Polygon {
                polygon: Polygon {
                    vertices: vec![
                        Point { x: 0, y: 0 },
                        Point { x: 10_000, y: 0 },
                        Point {
                            x: 10_000,
                            y: 10_000,
                        },
                    ],
                    closed: true,
                },
                width: 2_000,
            },
            Primitive::Polygon {
                polygon: Polygon {
                    vertices: vec![
                        Point {
                            x: 20_000,
                            y: 20_000,
                        },
                        Point {
                            x: 25_000,
                            y: 30_000,
                        },
                        Point {
                            x: 35_000,
                            y: 30_000,
                        },
                    ],
                    closed: false,
                },
                width: 2_500,
            },
        ],
        models_3d: Vec::new(),
        tags: HashSet::new(),
    };

    std::fs::write(
        pool_root
            .join("packages")
            .join(format!("{package_uuid}.json")),
        format!(
            "{}\n",
            to_json_deterministic(&package).expect("package should serialize")
        ),
    )
    .expect("package should write");

    let project_json_path = root.join("project.json");
    let mut project_json: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&project_json_path).expect("project should read"),
    )
    .expect("project json should parse");
    project_json["pools"] =
        serde_json::json!([{ "path": pool_root.to_string_lossy().to_string(), "priority": 1 }]);
    std::fs::write(
        &project_json_path,
        format!(
            "{}\n",
            to_json_deterministic(&project_json).expect("project json should serialize")
        ),
    )
    .expect("project json should write");

    let part_uuid = Uuid::new_v4();
    let placed_output = execute(
        Cli::try_parse_from([
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
            "MCU",
            "--x-nm",
            "1000",
            "--y-nm",
            "2000",
            "--layer",
            "1",
        ])
        .expect("CLI should parse"),
    )
    .expect("placement should succeed");
    let placed: serde_json::Value =
        serde_json::from_str(&placed_output).expect("placement output should parse");
    let component_uuid = placed["component_uuid"].as_str().unwrap().to_string();

    let output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "query",
            root.to_str().unwrap(),
            "board-component-silkscreen",
            "--component",
            &component_uuid,
        ])
        .expect("CLI should parse"),
    )
    .expect("query should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).expect("report JSON");
    assert_eq!(report["component_uuid"], component_uuid);
    assert_eq!(report["text_count"], 0);
    assert_eq!(report["line_count"], 1);
    assert_eq!(report["arc_count"], 1);
    assert_eq!(report["circle_count"], 1);
    assert_eq!(report["polygon_count"], 1);
    assert_eq!(report["polyline_count"], 1);
    assert_eq!(report["lines"][0]["from"]["x"], 10_000);
    assert_eq!(report["arcs"][0]["radius_nm"], 25_000);
    assert_eq!(report["circles"][0]["radius_nm"], 15_000);
    assert_eq!(
        report["polygons"][0]["vertices"].as_array().unwrap().len(),
        3
    );
    assert_eq!(
        report["polylines"][0]["vertices"].as_array().unwrap().len(),
        3
    );

    let _ = std::fs::remove_dir_all(&root);
}
