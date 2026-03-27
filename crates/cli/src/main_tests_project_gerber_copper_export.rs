use super::*;
use eda_engine::ir::serialization::to_json_deterministic;

fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}", label, Uuid::new_v4()))
}

#[test]
fn project_export_gerber_copper_layer_writes_rs274x_track_file() {
    let root = unique_project_root("datum-eda-cli-project-gerber-copper-export");
    create_native_project(&root, Some("Gerber Copper Demo".to_string()))
        .expect("initial scaffold should succeed");

    let net_uuid = Uuid::new_v4();
    let class_uuid = Uuid::new_v4();
    let pad_uuid = Uuid::new_v4();
    let track_a_uuid = Uuid::new_v4();
    let track_b_uuid = Uuid::new_v4();
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
                "name": "Gerber Copper Demo Board",
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
                    track_b_uuid.to_string(): {
                        "uuid": track_b_uuid,
                        "net": net_uuid,
                        "from": { "x": 0, "y": 500000 },
                        "to": { "x": 1000000, "y": 500000 },
                        "width": 300000,
                        "layer": 1
                    },
                    track_a_uuid.to_string(): {
                        "uuid": track_a_uuid,
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
    let cli = Cli::try_parse_from([
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
    let output = execute(cli).expect("gerber copper export should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).expect("report JSON");
    assert_eq!(report["action"], "export_gerber_copper_layer");
    assert_eq!(report["layer"], 1);
    assert_eq!(report["pad_count"], 1);
    assert_eq!(report["track_count"], 2);
    assert_eq!(report["zone_count"], 1);
    assert_eq!(report["via_count"], 1);

    let gerber = std::fs::read_to_string(&gerber_path).expect("gerber should read");
    assert!(gerber.contains("%ADD10C,0.200000*%"));
    assert!(gerber.contains("%ADD11C,0.300000*%"));
    assert!(gerber.contains("%ADD12C,0.450000*%"));
    assert!(gerber.contains("%ADD13C,0.600000*%"));
    assert!(gerber.contains("D10*"));
    assert!(gerber.contains("D11*"));
    assert!(gerber.contains("X0Y0D02*"));
    assert!(gerber.contains("X1000000Y0D01*"));
    assert!(gerber.contains("X0Y500000D02*"));
    assert!(gerber.contains("X1000000Y500000D01*"));
    assert!(gerber.contains("G36*"));
    assert!(gerber.contains("G37*"));
    assert!(gerber.contains("D12*"));
    assert!(gerber.contains("X750000Y250000D03*"));
    assert!(gerber.contains("D13*"));
    assert!(gerber.contains("X250000Y250000D03*"));
    assert!(gerber.contains("X0Y1000000D02*"));
    assert!(gerber.contains("X1000000Y1000000D01*"));
    assert!(gerber.contains("X1000000Y1500000D01*"));
    assert!(gerber.ends_with("M02*\n"));

    let _ = std::fs::remove_dir_all(&root);
}
