use super::*;
use eda_engine::ir::serialization::to_json_deterministic;

fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}", label, Uuid::new_v4()))
}

#[test]
fn project_export_gerber_mechanical_layer_writes_keepout_regions() {
    let root = unique_project_root("datum-eda-cli-project-gerber-mech-export");
    create_native_project(&root, Some("Gerber Mech Demo".to_string()))
        .expect("initial scaffold should succeed");

    let keepout_uuid = Uuid::new_v4();
    let board_json = root.join("board/board.json");
    std::fs::write(
        &board_json,
        format!(
            "{}\n",
            to_json_deterministic(&serde_json::json!({
                "schema_version": 1,
                "uuid": Uuid::new_v4(),
                "name": "Gerber Mech Demo Board",
                "stackup": {
                    "layers": [
                        {"id": 41, "name": "Mechanical 1", "layer_type": "Mechanical", "thickness_nm": 0}
                    ]
                },
                "outline": { "vertices": [], "closed": true },
                "packages": {},
                "pads": {},
                "tracks": {},
                "vias": {},
                "zones": {},
                "nets": {},
                "net_classes": {},
                "rules": [],
                "keepouts": [{
                    "uuid": keepout_uuid,
                    "polygon": {
                        "vertices": [
                            { "x": 0, "y": 0 },
                            { "x": 1000000, "y": 0 },
                            { "x": 1000000, "y": 500000 }
                        ],
                        "closed": true
                    },
                    "layers": [41],
                    "kind": "mechanical"
                }],
                "dimensions": [],
                "texts": []
            }))
            .expect("canonical serialization should succeed")
        ),
    )
    .expect("board file should write");

    let gerber_path = root.join("mech1.gbr");
    let cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "export-gerber-mechanical-layer",
        root.to_str().unwrap(),
        "--layer",
        "41",
        "--out",
        gerber_path.to_str().unwrap(),
    ])
    .expect("CLI should parse");
    let output = execute(cli).expect("gerber mechanical export should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).expect("report JSON");
    assert_eq!(report["action"], "export_gerber_mechanical_layer");
    assert_eq!(report["layer"], 41);
    assert_eq!(report["keepout_count"], 1);

    let gerber = std::fs::read_to_string(&gerber_path).expect("gerber should read");
    assert!(gerber.contains("G04 datum-eda native mechanical layer 41*"));
    assert!(gerber.contains("%ADD10C,0.100000*%"));
    assert!(gerber.contains("G36*"));
    assert!(gerber.contains("G37*"));
    assert!(gerber.contains("X0Y0D02*"));
    assert!(gerber.contains("X1000000Y0D01*"));
    assert!(gerber.contains("X1000000Y500000D01*"));

    let _ = std::fs::remove_dir_all(&root);
}
