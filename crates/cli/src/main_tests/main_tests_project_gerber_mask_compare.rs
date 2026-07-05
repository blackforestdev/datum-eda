use super::*;
use eda_engine::ir::serialization::to_json_deterministic;

fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}", label, Uuid::new_v4()))
}

#[test]
fn project_compare_gerber_soldermask_layer_is_semantic_and_reports_drift() {
    let root = unique_project_root("datum-eda-cli-project-gerber-mask-compare");
    create_native_project(&root, Some("Gerber Mask Compare Demo".to_string()))
        .expect("initial scaffold should succeed");

    let circle_pad_uuid = Uuid::new_v4();
    let rect_pad_uuid = Uuid::new_v4();
    let component_uuid = Uuid::new_v4();
    let board_json = root.join("board/board.json");
    std::fs::write(
        &board_json,
        format!(
            "{}\n",
            to_json_deterministic(&serde_json::json!({
                "schema_version": 1,
                "uuid": Uuid::new_v4(),
                "name": "Gerber Mask Compare Demo Board",
                "stackup": {
                    "layers": [
                        {"id": 1, "name": "Top Copper", "layer_type": "Copper", "thickness_nm": 35000},
                        {"id": 2, "name": "Top Mask", "layer_type": "SolderMask", "thickness_nm": 10000}
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
                    circle_pad_uuid.to_string(): {
                        "uuid": circle_pad_uuid,
                        "package": Uuid::new_v4(),
                        "name": "1",
                        "net": null,
                        "position": { "x": 750000, "y": 250000 },
                        "layer": 1,
                        "diameter": 450000
                    },
                    rect_pad_uuid.to_string(): {
                        "uuid": rect_pad_uuid,
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

    let gerber_path = root.join("top-mask.gbr");
    std::fs::write(
        &gerber_path,
        concat!(
            "G04 semantic mask compare fixture*\n",
            "%FSLAX46Y46*%\n",
            "%MOMM*%\n",
            "%LPD*%\n",
            "%ADD10C,0.450000*%\n",
            "%ADD11C,0.500000*%\n",
            "%ADD12R,0.800000X0.400000*%\n",
            "D11*\n",
            "X1750000Y250000D03*\n",
            "D12*\n",
            "X1250000Y250000D03*\n",
            "D10*\n",
            "X750000Y250000D03*\n",
            "M02*\n"
        ),
    )
    .expect("gerber file should write");

    let cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "compare-gerber-soldermask-layer",
        root.to_str().unwrap(),
        "--layer",
        "2",
        "--gerber",
        gerber_path.to_str().unwrap(),
    ])
    .expect("CLI should parse");
    let output = execute(cli).expect("mask compare should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).expect("report JSON");
    assert_eq!(report["action"], "compare_gerber_soldermask_layer");
    assert_eq!(report["layer"], 2);
    assert_eq!(report["source_copper_layer"], 1);
    assert_eq!(report["expected_pad_count"], 3);
    assert_eq!(report["actual_pad_count"], 3);
    assert_eq!(report["matched_count"], 3);
    assert_eq!(report["missing_count"], 0);
    assert_eq!(report["extra_count"], 0);

    std::fs::write(
        &gerber_path,
        concat!(
            "G04 semantic mask drift fixture*\n",
            "%FSLAX46Y46*%\n",
            "%MOMM*%\n",
            "%LPD*%\n",
            "%ADD10C,0.450000*%\n",
            "%ADD11C,0.700000*%\n",
            "%ADD12C,0.500000*%\n",
            "D10*\n",
            "X750000Y250000D03*\n",
            "D11*\n",
            "X1250000Y250000D03*\n",
            "D12*\n",
            "X1750000Y250000D03*\n",
            "M02*\n"
        ),
    )
    .expect("drift gerber should write");

    let cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "compare-gerber-soldermask-layer",
        root.to_str().unwrap(),
        "--layer",
        "2",
        "--gerber",
        gerber_path.to_str().unwrap(),
    ])
    .expect("CLI should parse");
    let output = execute(cli).expect("mask compare should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).expect("report JSON");
    assert_eq!(report["actual_pad_count"], 2);
    assert_eq!(report["matched_count"], 2);
    assert_eq!(report["missing_count"], 1);
    assert_eq!(report["extra_count"], 1);

    let _ = std::fs::remove_dir_all(&root);
}
