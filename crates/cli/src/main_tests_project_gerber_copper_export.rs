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
    let track_a_uuid = Uuid::new_v4();
    let track_b_uuid = Uuid::new_v4();
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
                "pads": {},
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
                "vias": {},
                "zones": {},
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
        "eda", "--format", "json", "project", "export-gerber-copper-layer",
        root.to_str().unwrap(), "--layer", "1", "--out", gerber_path.to_str().unwrap(),
    ])
    .expect("CLI should parse");
    let output = execute(cli).expect("gerber copper export should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).expect("report JSON");
    assert_eq!(report["action"], "export_gerber_copper_layer");
    assert_eq!(report["layer"], 1);
    assert_eq!(report["track_count"], 2);

    let gerber = std::fs::read_to_string(&gerber_path).expect("gerber should read");
    assert!(gerber.contains("%ADD10C,0.200000*%"));
    assert!(gerber.contains("%ADD11C,0.300000*%"));
    assert!(gerber.contains("D10*"));
    assert!(gerber.contains("D11*"));
    assert!(gerber.contains("X0Y0D02*"));
    assert!(gerber.contains("X1000000Y0D01*"));
    assert!(gerber.contains("X0Y500000D02*"));
    assert!(gerber.contains("X1000000Y500000D01*"));
    assert!(gerber.ends_with("M02*\n"));

    let _ = std::fs::remove_dir_all(&root);
}
