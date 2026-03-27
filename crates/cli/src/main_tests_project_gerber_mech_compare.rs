use super::*;
use eda_engine::ir::serialization::to_json_deterministic;

fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}", label, Uuid::new_v4()))
}

#[test]
fn project_compare_gerber_mechanical_layer_is_semantic_and_reports_drift() {
    let root = unique_project_root("datum-eda-cli-project-gerber-mech-compare");
    create_native_project(&root, Some("Gerber Mech Compare Demo".to_string()))
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
                "name": "Gerber Mech Compare Demo Board",
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
    std::fs::write(
        &gerber_path,
        concat!(
            "G04 semantic mech compare fixture*\n",
            "%FSLAX46Y46*%\n",
            "%MOMM*%\n",
            "%LPD*%\n",
            "%ADD10C,0.100000*%\n",
            "D10*\n",
            "G36*\n",
            "X0Y0D02*\n",
            "X1000000Y0D01*\n",
            "X1000000Y500000D01*\n",
            "X0Y0D01*\n",
            "G37*\n",
            "M02*\n"
        ),
    )
    .expect("gerber file should write");

    let cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "compare-gerber-mechanical-layer",
        root.to_str().unwrap(),
        "--layer",
        "41",
        "--gerber",
        gerber_path.to_str().unwrap(),
    ])
    .expect("CLI should parse");
    let output = execute(cli).expect("mechanical compare should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).expect("report JSON");
    assert_eq!(report["action"], "compare_gerber_mechanical_layer");
    assert_eq!(report["layer"], 41);
    assert_eq!(report["expected_keepout_count"], 1);
    assert_eq!(report["matched_count"], 1);
    assert_eq!(report["missing_count"], 0);
    assert_eq!(report["extra_count"], 0);

    std::fs::write(
        &gerber_path,
        concat!(
            "G04 semantic mech drift fixture*\n",
            "%FSLAX46Y46*%\n",
            "%MOMM*%\n",
            "%LPD*%\n",
            "%ADD10C,0.100000*%\n",
            "D10*\n",
            "G36*\n",
            "X0Y0D02*\n",
            "X900000Y0D01*\n",
            "X900000Y500000D01*\n",
            "X0Y0D01*\n",
            "G37*\n",
            "M02*\n"
        ),
    )
    .expect("drift gerber should write");

    let cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "compare-gerber-mechanical-layer",
        root.to_str().unwrap(),
        "--layer",
        "41",
        "--gerber",
        gerber_path.to_str().unwrap(),
    ])
    .expect("CLI should parse");
    let output = execute(cli).expect("mechanical compare should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).expect("report JSON");
    assert_eq!(report["matched_count"], 0);
    assert_eq!(report["missing_count"], 1);
    assert_eq!(report["extra_count"], 1);

    let _ = std::fs::remove_dir_all(&root);
}
