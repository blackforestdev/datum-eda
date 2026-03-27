use super::*;
use eda_engine::ir::serialization::to_json_deterministic;

fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}", label, Uuid::new_v4()))
}

#[test]
fn project_validate_gerber_copper_layer_reports_match_and_mismatch() {
    let root = unique_project_root("datum-eda-cli-project-gerber-copper-validate");
    create_native_project(&root, Some("Gerber Copper Validate Demo".to_string()))
        .expect("initial scaffold should succeed");

    let net_uuid = Uuid::new_v4();
    let class_uuid = Uuid::new_v4();
    let pad_uuid = Uuid::new_v4();
    let track_uuid = Uuid::new_v4();
    let zone_uuid = Uuid::new_v4();
    let via_uuid = Uuid::new_v4();
    let board_json = root.join("board/board.json");
    std::fs::write(
        &board_json,
        format!(
            "{}\n",
            to_json_deterministic(&serde_json::json!({
                "schema_version": 1,
                "uuid": Uuid::new_v4(),
                "name": "Gerber Copper Validate Demo Board",
                "stackup": { "layers": [] },
                "outline": { "vertices": [], "closed": true },
                "packages": {},
                "pads": {
                    pad_uuid.to_string(): {
                        "uuid": pad_uuid,
                        "package": Uuid::new_v4(),
                        "name": "1",
                        "net": net_uuid,
                        "position": { "x": 750000, "y": 250000 },
                        "layer": 1,
                        "diameter": 450000
                    }
                },
                "tracks": {
                    track_uuid.to_string(): {
                        "uuid": track_uuid,
                        "net": net_uuid,
                        "from": { "x": 0, "y": 0 },
                        "to": { "x": 1000000, "y": 0 },
                        "width": 200000,
                        "layer": 1
                    }
                },
                "vias": {
                    via_uuid.to_string(): {
                        "uuid": via_uuid,
                        "net": net_uuid,
                        "position": { "x": 250000, "y": 250000 },
                        "drill": 300000,
                        "diameter": 600000,
                        "from_layer": 1,
                        "to_layer": 2
                    }
                },
                "zones": {
                    zone_uuid.to_string(): {
                        "uuid": zone_uuid,
                        "net": net_uuid,
                        "polygon": {
                            "vertices": [
                                { "x": 0, "y": 1000000 },
                                { "x": 1000000, "y": 1000000 },
                                { "x": 1000000, "y": 1500000 }
                            ],
                            "closed": true
                        },
                        "layer": 1,
                        "priority": 1,
                        "thermal_relief": true,
                        "thermal_gap": 250000,
                        "thermal_spoke_width": 200000
                    }
                },
                "nets": {
                    net_uuid.to_string(): {
                        "uuid": net_uuid,
                        "name": "GND",
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

    let gerber_path = root.join("top-copper.gbr");
    let export_cli = Cli::try_parse_from([
        "eda",
        "project",
        "export-gerber-copper-layer",
        root.to_str().unwrap(),
        "--layer",
        "1",
        "--out",
        gerber_path.to_str().unwrap(),
    ])
    .expect("export CLI should parse");
    let _ = execute(export_cli).expect("gerber copper export should succeed");

    let validate_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "validate-gerber-copper-layer",
        root.to_str().unwrap(),
        "--layer",
        "1",
        "--gerber",
        gerber_path.to_str().unwrap(),
    ])
    .expect("validate CLI should parse");
    let (output, exit_code) = execute_with_exit_code(validate_cli).expect("validation should run");
    let report: serde_json::Value = serde_json::from_str(&output).expect("report JSON");
    assert_eq!(exit_code, 0);
    assert_eq!(report["action"], "validate_gerber_copper_layer");
    assert_eq!(report["matches_expected"], true);
    assert_eq!(report["layer"], 1);
    assert_eq!(report["pad_count"], 1);
    assert_eq!(report["track_count"], 1);
    assert_eq!(report["zone_count"], 1);
    assert_eq!(report["via_count"], 1);

    std::fs::write(&gerber_path, "corrupted\n").expect("gerber overwrite should succeed");
    let validate_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "validate-gerber-copper-layer",
        root.to_str().unwrap(),
        "--layer",
        "1",
        "--gerber",
        gerber_path.to_str().unwrap(),
    ])
    .expect("validate CLI should parse");
    let (output, exit_code) = execute_with_exit_code(validate_cli).expect("validation should run");
    let report: serde_json::Value = serde_json::from_str(&output).expect("report JSON");
    assert_eq!(exit_code, 1);
    assert_eq!(report["matches_expected"], false);
    assert_eq!(report["pad_count"], 1);
    assert_eq!(report["zone_count"], 1);
    assert_eq!(report["via_count"], 1);

    let _ = std::fs::remove_dir_all(&root);
}
