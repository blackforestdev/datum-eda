use super::*;
use eda_engine::ir::serialization::to_json_deterministic;

fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}", label, Uuid::new_v4()))
}

fn board_diagnostics_query_cli(root: &Path) -> Cli {
    Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "query",
        root.to_str().unwrap(),
        "board-diagnostics",
    ])
    .expect("CLI should parse")
}

fn board_unrouted_query_cli(root: &Path) -> Cli {
    Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "query",
        root.to_str().unwrap(),
        "board-unrouted",
    ])
    .expect("CLI should parse")
}

#[test]
fn project_query_board_connectivity_reports_diagnostics_and_airwires() {
    let root = unique_project_root("datum-eda-cli-project-board-connectivity");
    create_native_project(&root, Some("Board Connectivity Demo".to_string()))
        .expect("initial scaffold should succeed");

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
                "name": "Board Connectivity Demo Board",
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

    let diagnostics_output =
        execute(board_diagnostics_query_cli(&root)).expect("board diagnostics query should succeed");
    let diagnostics: serde_json::Value =
        serde_json::from_str(&diagnostics_output).expect("diagnostics output should parse");
    assert_eq!(diagnostics["domain"], "board");
    let entries = diagnostics["diagnostics"]
        .as_array()
        .expect("diagnostics should be an array");
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0]["kind"], "net_without_copper");
    assert!(entries[0]["message"]
        .as_str()
        .unwrap()
        .contains("SIG"));

    let unrouted_output =
        execute(board_unrouted_query_cli(&root)).expect("board unrouted query should succeed");
    let unrouted: serde_json::Value =
        serde_json::from_str(&unrouted_output).expect("unrouted output should parse");
    assert_eq!(unrouted["domain"], "board");
    let airwires = unrouted["airwires"]
        .as_array()
        .expect("airwires should be an array");
    assert_eq!(airwires.len(), 1);
    assert_eq!(airwires[0]["net_name"], "SIG");
    assert_eq!(airwires[0]["from"]["component"], "R1");
    assert_eq!(airwires[0]["to"]["component"], "R2");
    assert_eq!(airwires[0]["from"]["pin"], "1");
    assert_eq!(airwires[0]["to"]["pin"], "1");

    let _ = std::fs::remove_dir_all(&root);
}
