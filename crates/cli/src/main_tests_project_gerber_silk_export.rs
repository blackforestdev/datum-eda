use super::*;
use eda_engine::ir::serialization::to_json_deterministic;

fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}", label, Uuid::new_v4()))
}

#[test]
fn project_export_gerber_silkscreen_layer_writes_rs274x_text_strokes() {
    let root = unique_project_root("datum-eda-cli-project-gerber-silk-export");
    create_native_project(&root, Some("Gerber Silk Demo".to_string()))
        .expect("initial scaffold should succeed");

    let text_uuid = Uuid::new_v4();
    let component_uuid = Uuid::new_v4();
    let board_json = root.join("board/board.json");
    std::fs::write(
        &board_json,
        format!(
            "{}\n",
            to_json_deterministic(&serde_json::json!({
                "schema_version": 1,
                "uuid": Uuid::new_v4(),
                "name": "Gerber Silk Demo Board",
                "stackup": {
                    "layers": [
                        {"id": 1, "name": "Top Copper", "layer_type": "Copper", "thickness_nm": 35000},
                        {"id": 3, "name": "Top Silk", "layer_type": "Silkscreen", "thickness_nm": 10000}
                    ]
                },
                "outline": { "vertices": [], "closed": true },
                "packages": {
                    component_uuid.to_string(): {
                        "uuid": component_uuid,
                        "part": Uuid::new_v4(),
                        "package": Uuid::new_v4(),
                        "reference": "U1",
                        "value": "DRV",
                        "position": { "x": 3000000, "y": 4000000 },
                        "rotation": 90,
                        "layer": 3,
                        "locked": false
                    }
                },
                "component_silkscreen": {
                    component_uuid.to_string(): [{
                        "from": { "x": 0, "y": 0 },
                        "to": { "x": 1000000, "y": 0 },
                        "width_nm": 150000,
                        "layer": 3
                    }]
                },
                "component_silkscreen_texts": {
                    component_uuid.to_string(): [{
                        "text": "U",
                        "position": { "x": 500000, "y": 0 },
                        "rotation": 0,
                        "height_nm": 1000000,
                        "stroke_width_nm": 150000,
                        "layer": 3
                    }]
                },
                "pads": {},
                "tracks": {},
                "vias": {},
                "zones": {},
                "nets": {},
                "net_classes": {},
                "rules": [],
                "keepouts": [],
                "dimensions": [],
                "texts": [{
                    "uuid": text_uuid,
                    "text": "TOP",
                    "position": { "x": 1000000, "y": 2000000 },
                    "rotation": 0,
                    "height_nm": 1000000,
                    "stroke_width_nm": 120000,
                    "layer": 3
                }]
            }))
            .expect("canonical serialization should succeed")
        ),
    )
    .expect("board file should write");

    let gerber_path = root.join("top-silk.gbr");
    let cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "export-gerber-silkscreen-layer",
        root.to_str().unwrap(),
        "--layer",
        "3",
        "--out",
        gerber_path.to_str().unwrap(),
    ])
    .expect("CLI should parse");
    let output = execute(cli).expect("gerber silkscreen export should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).expect("report JSON");
    assert_eq!(report["action"], "export_gerber_silkscreen_layer");
    assert_eq!(report["layer"], 3);
    assert_eq!(report["text_count"], 1);
    assert_eq!(report["component_text_count"], 1);
    assert_eq!(report["component_stroke_count"], 1);

    let gerber = std::fs::read_to_string(&gerber_path).expect("gerber should read");
    assert!(gerber.contains("G04 datum-eda native silkscreen layer 3*"));
    assert!(gerber.contains("%ADD10C,0.120000*%"));
    assert!(gerber.contains("%ADD11C,0.150000*%"));
    assert!(gerber.contains("D10*"));
    assert!(gerber.contains("D11*"));
    assert!(gerber.contains("D02*"));
    assert!(gerber.contains("D01*"));
    assert!(gerber.contains("X3000000Y4000000D02*"));
    assert!(gerber.contains("X3000000Y5000000D01*"));
    assert!(gerber.contains("X2200000Y4500000D02*"));
    assert!(gerber.contains("X3000000Y4500000D01*"));
    assert!(gerber.ends_with("M02*\n"));

    let _ = std::fs::remove_dir_all(&root);
}
