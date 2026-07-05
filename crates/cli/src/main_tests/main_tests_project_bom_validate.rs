use super::*;
use eda_engine::ir::serialization::to_json_deterministic;

fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}", label, Uuid::new_v4()))
}

#[test]
fn project_validate_bom_reports_byte_match_and_drift() {
    let root = unique_project_root("datum-eda-cli-project-bom-validate");
    create_native_project(&root, Some("BOM Validate Demo".to_string()))
        .expect("initial scaffold should succeed");

    let board_json = root.join("board/board.json");
    let c1_uuid = Uuid::new_v4();
    let c1_part_uuid = Uuid::new_v4();
    let c1_package_uuid = Uuid::new_v4();
    std::fs::write(
        &board_json,
        format!(
            "{}\n",
            to_json_deterministic(&serde_json::json!({
                "schema_version": 1,
                "uuid": Uuid::new_v4(),
                "name": "BOM Validate Demo Board",
                "stackup": { "layers": [] },
                "outline": { "vertices": [], "closed": true },
                "packages": {
                    c1_uuid.to_string(): {
                        "uuid": c1_uuid,
                        "part": c1_part_uuid,
                        "package": c1_package_uuid,
                        "reference": "C1",
                        "value": "1uF",
                        "position": { "x": 1000, "y": 1500 },
                        "rotation": 90,
                        "layer": 1,
                        "locked": false
                    }
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
                "texts": []
            }))
            .expect("canonical serialization should succeed")
        ),
    )
    .expect("board file should write");

    let bom_path = root.join("bom.csv");
    execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "export-bom",
            root.to_str().unwrap(),
            "--out",
            bom_path.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("BOM export should succeed");

    let ok = execute_with_exit_code(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "validate-bom",
            root.to_str().unwrap(),
            "--bom",
            bom_path.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("validate should succeed");
    let ok_report: serde_json::Value = serde_json::from_str(&ok.0).expect("report JSON");
    assert_eq!(ok.1, 0);
    assert_eq!(ok_report["action"], "validate_bom");
    assert_eq!(ok_report["matches_expected"], true);

    std::fs::write(
        &bom_path,
        format!(
            "reference,value,part_uuid,package_uuid,layer,x_nm,y_nm,rotation_deg,locked\n\
             C1,10uF,{c1_part_uuid},{c1_package_uuid},1,1000,1500,90,false\n"
        ),
    )
    .expect("drifted bom should write");

    let drift = execute_with_exit_code(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "validate-bom",
            root.to_str().unwrap(),
            "--bom",
            bom_path.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("validate should succeed");
    let drift_report: serde_json::Value = serde_json::from_str(&drift.0).expect("report JSON");
    assert_eq!(drift.1, 1);
    assert_eq!(drift_report["matches_expected"], false);

    let _ = std::fs::remove_dir_all(&root);
}
