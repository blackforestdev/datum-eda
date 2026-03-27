use super::*;
use eda_engine::ir::serialization::to_json_deterministic;

fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}", label, Uuid::new_v4()))
}

fn seed_board_connectivity_fixture(root: &Path) {
    let net_uuid = Uuid::new_v4();
    let class_uuid = Uuid::new_v4();
    let package_a_uuid = Uuid::new_v4();
    let package_b_uuid = Uuid::new_v4();
    let pad_a_uuid = Uuid::new_v4();
    let pad_b_uuid = Uuid::new_v4();
    let board_json = root.join("board/board.json");
    std::fs::write(
        &board_json,
        format!(
            "{}\n",
            to_json_deterministic(&serde_json::json!({
                "schema_version": 1,
                "uuid": Uuid::new_v4(),
                "name": "Board Check Demo Board",
                "stackup": { "layers": [] },
                "outline": { "vertices": [], "closed": true },
                "packages": {
                    package_a_uuid.to_string(): {
                        "uuid": package_a_uuid,
                        "part": Uuid::new_v4(),
                        "package": Uuid::new_v4(),
                        "reference": "R1",
                        "value": "10k",
                        "position": { "x": 0, "y": 0 },
                        "rotation": 0,
                        "layer": 1,
                        "locked": false
                    },
                    package_b_uuid.to_string(): {
                        "uuid": package_b_uuid,
                        "part": Uuid::new_v4(),
                        "package": Uuid::new_v4(),
                        "reference": "R2",
                        "value": "10k",
                        "position": { "x": 5000000, "y": 0 },
                        "rotation": 0,
                        "layer": 1,
                        "locked": false
                    }
                },
                "pads": {
                    pad_a_uuid.to_string(): {
                        "uuid": pad_a_uuid,
                        "package": package_a_uuid,
                        "name": "1",
                        "net": net_uuid,
                        "position": { "x": 0, "y": 0 },
                        "layer": 1
                    },
                    pad_b_uuid.to_string(): {
                        "uuid": pad_b_uuid,
                        "package": package_b_uuid,
                        "name": "1",
                        "net": net_uuid,
                        "position": { "x": 5000000, "y": 0 },
                        "layer": 1
                    }
                },
                "tracks": {},
                "vias": {},
                "zones": {},
                "nets": {
                    net_uuid.to_string(): {
                        "uuid": net_uuid,
                        "name": "SIG",
                        "class": class_uuid
                    }
                },
                "net_classes": {
                    class_uuid.to_string(): {
                        "uuid": class_uuid,
                        "name": "Default",
                        "clearance": 150000,
                        "track_width": 200000,
                        "via_drill": 300000,
                        "via_diameter": 600000,
                        "diffpair_width": 0,
                        "diffpair_gap": 0
                    }
                },
                "rules": [],
                "keepouts": [],
                "dimensions": [],
                "texts": []
            }))
            .expect("canonical serialization should succeed")
        ),
    )
    .expect("board file should write");
}

#[test]
fn project_query_board_check_reports_native_board_check_json() {
    let root = unique_project_root("datum-eda-cli-project-board-check-json");
    create_native_project(&root, Some("Board Check Demo".to_string()))
        .expect("initial scaffold should succeed");
    seed_board_connectivity_fixture(&root);

    let cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "query",
        root.to_str().unwrap(),
        "board-check",
    ])
    .expect("CLI should parse");

    let output = execute(cli).expect("project query board-check should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).expect("query JSON should parse");
    assert_eq!(report["domain"], "board");
    assert_eq!(report["summary"]["status"], "info");
    assert!(report["summary"]["by_code"]
        .as_array()
        .unwrap()
        .iter()
        .any(|entry| entry["code"] == "net_without_copper" && entry["count"] == 1));
    assert!(report["diagnostics"].as_array().unwrap().iter().any(|entry| {
        entry["kind"] == "net_without_copper" && entry["message"].as_str().unwrap().contains("SIG")
    }));

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_query_board_check_reports_native_board_check_text() {
    let root = unique_project_root("datum-eda-cli-project-board-check-text");
    create_native_project(&root, Some("Board Check Demo".to_string()))
        .expect("initial scaffold should succeed");
    seed_board_connectivity_fixture(&root);

    let cli = Cli::try_parse_from([
        "eda",
        "project",
        "query",
        root.to_str().unwrap(),
        "board-check",
    ])
    .expect("CLI should parse");

    let output = execute(cli).expect("project query board-check should succeed");
    assert!(output.contains("board check: status=info"));
    assert!(output.contains("counts:"));
    assert!(output.contains("net_without_copper x1"));
    assert!(output.contains("diagnostics:"));
    assert!(output.contains("board net SIG has no imported copper geometry"));

    let _ = std::fs::remove_dir_all(&root);
}
