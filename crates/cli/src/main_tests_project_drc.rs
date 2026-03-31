use super::*;
use eda_engine::ir::serialization::to_json_deterministic;
use eda_engine::schematic::{CheckDomain, WaiverTarget};

fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}", label, Uuid::new_v4()))
}

fn seed_board_drc_fixture(root: &Path) -> Uuid {
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
                "name": "Board DRC Demo Board",
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
    net_uuid
}

fn write_native_waivers(root: &Path, waivers: &[serde_json::Value]) {
    let schematic_json = root.join("schematic/schematic.json");
    let mut schematic_value: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&schematic_json).expect("schematic.json should read"),
    )
    .expect("schematic.json should parse");
    schematic_value["waivers"] = serde_json::Value::Array(waivers.to_vec());
    std::fs::write(
        &schematic_json,
        format!(
            "{}\n",
            to_json_deterministic(&schematic_value)
                .expect("canonical serialization should succeed")
        ),
    )
    .expect("schematic.json should write");
}

#[test]
fn project_query_drc_reports_native_board_drc_json() {
    let root = unique_project_root("datum-eda-cli-project-drc-json");
    create_native_project(&root, Some("Board DRC Demo".to_string()))
        .expect("initial scaffold should succeed");
    seed_board_drc_fixture(&root);

    let cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "query",
        root.to_str().unwrap(),
        "drc",
    ])
    .expect("CLI should parse");

    let output = execute(cli).expect("project query drc should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).expect("query JSON should parse");
    assert_eq!(report["passed"], false);
    assert_eq!(report["summary"]["errors"], 2);
    assert_eq!(report["summary"]["waived"], 0);
    assert!(
        report["violations"]
            .as_array()
            .unwrap()
            .iter()
            .any(|entry| entry["code"] == "connectivity_unrouted_net" && entry["waived"] == false)
    );

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_query_drc_reports_native_board_drc_text() {
    let root = unique_project_root("datum-eda-cli-project-drc-text");
    create_native_project(&root, Some("Board DRC Demo".to_string()))
        .expect("initial scaffold should succeed");
    seed_board_drc_fixture(&root);

    let cli = Cli::try_parse_from(["eda", "project", "query", root.to_str().unwrap(), "drc"])
        .expect("CLI should parse");

    let output = execute(cli).expect("project query drc should succeed");
    assert!(output.contains("drc: passed=false errors=2 warnings=0 waived=0"));
    assert!(output.contains("violations:"));
    assert!(output.contains("connectivity_no_copper"));
    assert!(output.contains("connectivity_unrouted_net"));

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_query_drc_honors_native_authored_waiver() {
    let root = unique_project_root("datum-eda-cli-project-drc-waiver");
    create_native_project(&root, Some("Board DRC Demo".to_string()))
        .expect("initial scaffold should succeed");
    let net_uuid = seed_board_drc_fixture(&root);

    write_native_waivers(
        &root,
        &[serde_json::to_value(serde_json::json!({
            "uuid": Uuid::new_v4(),
            "domain": CheckDomain::DRC,
            "target": WaiverTarget::Object(net_uuid),
            "rationale": "Intentional unrouted fixture net",
            "created_by": "cli-test"
        }))
        .expect("waiver should serialize")],
    );

    let cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "query",
        root.to_str().unwrap(),
        "drc",
    ])
    .expect("CLI should parse");

    let output = execute(cli).expect("project query drc should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).expect("query JSON should parse");
    assert_eq!(report["passed"], true);
    assert_eq!(report["summary"]["errors"], 0);
    assert_eq!(report["summary"]["waived"], 2);
    assert!(
        report["violations"]
            .as_array()
            .unwrap()
            .iter()
            .all(|entry| entry["waived"] == true)
    );

    let _ = std::fs::remove_dir_all(&root);
}
