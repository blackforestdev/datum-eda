use super::*;
use eda_engine::ir::geometry::{Point, Polygon};
use eda_engine::ir::serialization::to_json_deterministic;
use eda_engine::pool::{Package, Pad, Padstack, PadstackAperture};
use std::collections::{HashMap, HashSet};

fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}", label, Uuid::new_v4()))
}

#[test]
fn project_query_board_component_pads_reads_persisted_component_pads() {
    let root = unique_project_root("datum-eda-cli-project-board-component-pads");
    create_native_project(&root, Some("Board Component Pads Demo".to_string()))
        .expect("initial scaffold should succeed");

    let pool_root = root.join("pool");
    std::fs::create_dir_all(pool_root.join("packages")).expect("packages dir should exist");
    std::fs::create_dir_all(pool_root.join("padstacks")).expect("padstacks dir should exist");

    let padstack_uuid = Uuid::new_v4();
    let package_uuid = Uuid::new_v4();
    let pad_uuid = Uuid::new_v4();
    let package = Package {
        uuid: package_uuid,
        name: "PKG".to_string(),
        pads: HashMap::from([(
            pad_uuid,
            Pad {
                uuid: pad_uuid,
                name: "P1".to_string(),
                position: Point {
                    x: 123_000,
                    y: 456_000,
                },
                padstack: padstack_uuid,
                layer: 1,
            },
        )]),
        courtyard: Polygon {
            vertices: Vec::new(),
            closed: true,
        },
        silkscreen: Vec::new(),
        models_3d: Vec::new(),
        tags: HashSet::new(),
    };
    let padstack = Padstack {
        uuid: padstack_uuid,
        name: "ROUND".to_string(),
        aperture: Some(PadstackAperture::Circle {
            diameter_nm: 600_000,
        }),
        drill_nm: Some(300_000),
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
    std::fs::write(
        pool_root
            .join("padstacks")
            .join(format!("{padstack_uuid}.json")),
        format!(
            "{}\n",
            to_json_deterministic(&padstack).expect("padstack should serialize")
        ),
    )
    .expect("padstack should write");

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
            "board-component-pads",
            "--component",
            &component_uuid,
        ])
        .expect("CLI should parse"),
    )
    .expect("query should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).expect("report JSON");
    assert_eq!(report["component_uuid"], component_uuid);
    assert_eq!(report["pad_count"], 1);
    assert_eq!(report["pads"][0]["name"], "P1");
    assert_eq!(report["pads"][0]["position"]["x"], 123_000);
    assert_eq!(report["pads"][0]["position"]["y"], 456_000);
    assert_eq!(report["pads"][0]["layer"], 1);
    assert_eq!(report["pads"][0]["drill_nm"], 300_000);
    assert_eq!(report["pads"][0]["shape"], "circle");
    assert_eq!(report["pads"][0]["diameter_nm"], 600_000);

    let _ = std::fs::remove_dir_all(&root);
}
