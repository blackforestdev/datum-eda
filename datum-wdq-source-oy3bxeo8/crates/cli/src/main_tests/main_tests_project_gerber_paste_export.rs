use super::*;
use eda_engine::ir::serialization::to_json_deterministic;

fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}", label, Uuid::new_v4()))
}

#[test]
fn project_export_gerber_paste_layer_writes_rs274x_pad_openings() {
    let root = unique_project_root("datum-eda-cli-project-gerber-paste-export");
    create_native_project(&root, Some("Gerber Paste Demo".to_string()))
        .expect("initial scaffold should succeed");

    let pad_circle_uuid = Uuid::new_v4();
    let pad_rect_uuid = Uuid::new_v4();
    let component_uuid = Uuid::new_v4();
    let board_json = root.join("board/board.json");
    std::fs::write(
        &board_json,
        format!(
            "{}\n",
            to_json_deterministic(&serde_json::json!({
                "schema_version": 1,
                "uuid": Uuid::new_v4(),
                "name": "Gerber Paste Demo Board",
                "stackup": {
                    "layers": [
                        {"id": 1, "name": "Top Copper", "layer_type": "Copper", "thickness_nm": 35000},
                        {"id": 2, "name": "Top Paste", "layer_type": "Paste", "thickness_nm": 10000},
                        {"id": 31, "name": "Bottom Copper", "layer_type": "Copper", "thickness_nm": 35000},
                        {"id": 32, "name": "Bottom Paste", "layer_type": "Paste", "thickness_nm": 10000}
                    ]
                },
                "outline": { "vertices": [], "closed": true },
                "packages": {},
                "component_pads": {
                    component_uuid.to_string(): [
                        {
                            "uuid": Uuid::new_v4(),
                            "name": "CP1",
                            "position": { "x": 1750000, "y": 250000 },
                            "padstack": Uuid::new_v4(),
                            "layer": 1,
                            "shape": "circle",
                            "diameter_nm": 500000,
                            "width_nm": 0,
                            "height_nm": 0
                        },
                        {
                            "uuid": Uuid::new_v4(),
                            "name": "CP2",
                            "position": { "x": 2250000, "y": 250000 },
                            "padstack": Uuid::new_v4(),
                            "layer": 1,
                            "shape": null,
                            "diameter_nm": 0,
                            "width_nm": 0,
                            "height_nm": 0
                        }
                    ]
                },
                "pads": {
                    pad_circle_uuid.to_string(): {
                        "uuid": pad_circle_uuid,
                        "package": Uuid::new_v4(),
                        "name": "1",
                        "net": null,
                        "position": { "x": 750000, "y": 250000 },
                        "layer": 1,
                        "diameter": 450000
                    },
                    pad_rect_uuid.to_string(): {
                        "uuid": pad_rect_uuid,
                        "package": Uuid::new_v4(),
                        "name": "2",
                        "net": null,
                        "position": { "x": 1250000, "y": 250000 },
                        "layer": 1,
                        "shape": "rect",
                        "diameter": 0,
                        "width": 800000,
                        "height": 400000
                    }
                },
                "tracks": {},
                "vias": {},
                "zones": {},
                "nets": {},
                "net_classes": {},
                "rules": [],
                "keepouts": [],
                "dimensions": [],
                "texts": []
            }))
            .expect("canonical serialization should succeed")
        ),
    )
    .expect("board file should write");

    let gerber_path = root.join("top-paste.gbr");
    let cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "export-gerber-paste-layer",
        root.to_str().unwrap(),
        "--layer",
        "2",
        "--out",
        gerber_path.to_str().unwrap(),
    ])
    .expect("CLI should parse");
    let output = execute(cli).expect("gerber paste export should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).expect("report JSON");
    assert_eq!(report["action"], "export_gerber_paste_layer");
    assert_eq!(report["layer"], 2);
    assert_eq!(report["source_copper_layer"], 1);
    assert_eq!(report["pad_count"], 3);

    let gerber = std::fs::read_to_string(&gerber_path).expect("gerber should read");
    assert!(gerber.contains("G04 datum-eda native paste layer 2*"));
    assert!(gerber.contains("%ADD10C,0.450000*%"));
    assert!(gerber.contains("%ADD11C,0.500000*%"));
    assert!(gerber.contains("%ADD12R,0.800000X0.400000*%"));
    assert!(gerber.contains("D10*"));
    assert!(gerber.contains("X750000Y250000D03*"));
    assert!(gerber.contains("D12*"));
    assert!(gerber.contains("X1250000Y250000D03*"));
    assert!(gerber.contains("D11*"));
    assert!(gerber.contains("X1750000Y250000D03*"));
    assert!(!gerber.contains("X2250000Y250000D03*"));
    assert!(gerber.ends_with("M02*\n"));

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_export_gerber_paste_layer_uses_resolver_materialized_board_state() {
    let root = unique_project_root("datum-eda-cli-project-gerber-paste-resolved-export");
    create_native_project(&root, Some("Gerber Paste Resolved Export Demo".to_string()))
        .expect("initial scaffold should succeed");
    let board_json = root.join("board/board.json");
    let stale_board = std::fs::read_to_string(&board_json).expect("board file should read");
    let package_uuid = Uuid::new_v4();

    let place_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "place-board-pad",
        root.to_str().unwrap(),
        "--package",
        &package_uuid.to_string(),
        "--name",
        "1",
        "--x-nm",
        "750000",
        "--y-nm",
        "250000",
        "--layer",
        "1",
        "--diameter-nm",
        "450000",
    ])
    .expect("CLI should parse");
    let _ = execute(place_cli).expect("place board pad should succeed");
    std::fs::write(&board_json, stale_board).expect("stale board file should restore");

    let gerber_path = root.join("top-paste-resolved.gbr");
    let cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "export-gerber-paste-layer",
        root.to_str().unwrap(),
        "--layer",
        "4",
        "--out",
        gerber_path.to_str().unwrap(),
    ])
    .expect("CLI should parse");
    let output = execute(cli).expect("gerber paste export should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).expect("report JSON");
    assert_eq!(report["source_copper_layer"], 1);
    assert_eq!(report["pad_count"], 1);

    let gerber = std::fs::read_to_string(&gerber_path).expect("gerber should read");
    assert!(gerber.contains("G04 datum-eda native paste layer 4*"));
    assert!(gerber.contains("%ADD10C,0.450000*%"));
    assert!(gerber.contains("X750000Y250000D03*"));

    let _ = std::fs::remove_dir_all(&root);
}
