use super::*;
use eda_engine::ir::geometry::Polygon;
use eda_engine::ir::serialization::to_json_deterministic;
use eda_engine::pool::{ModelRef, Package};
use std::collections::{HashMap, HashSet};

fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}", label, Uuid::new_v4()))
}

#[test]
fn project_query_board_component_models_3d_reads_persisted_model_refs() {
    let root = unique_project_root("datum-eda-cli-project-board-component-models-3d");
    create_native_project(&root, Some("Board Component Models3d Demo".to_string()))
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
        silkscreen: Vec::new(),
        models_3d: vec![
            ModelRef {
                path: "models/pkg.step".to_string(),
                transform: Some(serde_json::json!({
                    "translate": [1, 2, 3],
                    "rotate": [0, 90, 180]
                })),
            },
            ModelRef {
                path: "models/pkg.wrl".to_string(),
                transform: Some(serde_json::json!({
                    "translate": [-1, -2, 0],
                    "rotate": [10, 20, 30]
                })),
            },
        ],
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
            "board-component-models-3d",
            "--component",
            &component_uuid,
        ])
        .expect("CLI should parse"),
    )
    .expect("query should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).expect("report JSON");
    assert_eq!(report["component_uuid"], component_uuid);
    assert_eq!(report["model_count"], 2);
    assert_eq!(report["models"][0]["path"], "models/pkg.step");
    assert_eq!(report["models"][1]["path"], "models/pkg.wrl");

    let _ = std::fs::remove_dir_all(&root);
}
