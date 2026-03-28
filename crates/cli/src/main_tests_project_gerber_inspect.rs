use super::*;
use eda_engine::ir::serialization::to_json_deterministic;

fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}", label, Uuid::new_v4()))
}

#[test]
fn project_inspect_gerber_reports_mixed_subset_geometry() {
    let root = unique_project_root("datum-eda-cli-project-gerber-inspect");
    create_native_project(&root, Some("Gerber Inspect Demo".to_string()))
        .expect("initial scaffold should succeed");

    let net_uuid = Uuid::new_v4();
    let class_uuid = Uuid::new_v4();
    let pad_uuid = Uuid::new_v4();
    let track_uuid = Uuid::new_v4();
    let zone_uuid = Uuid::new_v4();
    let board_json = root.join("board/board.json");
    std::fs::write(
        &board_json,
        format!(
            "{}\n",
            to_json_deterministic(&serde_json::json!({
                "schema_version": 1,
                "uuid": Uuid::new_v4(),
                "name": "Gerber Inspect Demo Board",
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
                "vias": {},
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

    let gerber_path = root.join("inspect-top-copper.gbr");
    let export_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "export-gerber-copper-layer",
        root.to_str().unwrap(),
        "--layer",
        "1",
        "--out",
        gerber_path.to_str().unwrap(),
    ])
    .expect("CLI should parse");
    execute(export_cli).expect("gerber export should succeed");

    let inspect_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "inspect-gerber",
        gerber_path.to_str().unwrap(),
    ])
    .expect("CLI should parse");
    let output = execute(inspect_cli).expect("gerber inspect should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).expect("report JSON");
    assert_eq!(report["action"], "inspect_gerber");
    assert_eq!(report["geometry_count"], 3);
    assert_eq!(report["stroke_count"], 1);
    assert_eq!(report["flash_count"], 1);
    assert_eq!(report["region_count"], 1);

    let entries = report["entries"].as_array().expect("entries array");
    assert_eq!(entries.len(), 3);
    assert!(entries.iter().any(|entry| entry["kind"] == "stroke"));
    assert!(entries.iter().any(|entry| entry["kind"] == "flash"));
    assert!(entries.iter().any(|entry| entry["kind"] == "region"));

    let _ = std::fs::remove_dir_all(&root);
}
