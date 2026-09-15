use super::*;
use eda_engine::ir::serialization::to_json_deterministic;

fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}", label, Uuid::new_v4()))
}

#[test]
fn project_validate_pnp_reports_byte_match_and_drift() {
    let root = unique_project_root("datum-eda-cli-project-pnp-validate");
    create_native_project(&root, Some("PnP Validate Demo".to_string()))
        .expect("initial scaffold should succeed");

    let board_json = root.join("board/board.json");
    let u1_uuid = Uuid::new_v4();
    let u1_part_uuid = Uuid::new_v4();
    let u1_package_uuid = Uuid::new_v4();
    std::fs::write(
        &board_json,
        format!(
            "{}\n",
            to_json_deterministic(&serde_json::json!({
                "schema_version": 1,
                "uuid": Uuid::new_v4(),
                "name": "PnP Validate Demo Board",
                "stackup": { "layers": [] },
                "outline": { "vertices": [], "closed": true },
                "packages": {
                    u1_uuid.to_string(): {
                        "uuid": u1_uuid,
                        "part": u1_part_uuid,
                        "package": u1_package_uuid,
                        "reference": "U1",
                        "value": "MCU",
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

    let pnp_path = root.join("pnp.csv");
    execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "export-pnp",
            root.to_str().unwrap(),
            "--out",
            pnp_path.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("PnP export should succeed");

    let ok = execute_with_exit_code(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "validate-pnp",
            root.to_str().unwrap(),
            "--pnp",
            pnp_path.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("validate should succeed");
    let ok_report: serde_json::Value = serde_json::from_str(&ok.0).expect("report JSON");
    assert_eq!(ok.1, 0);
    assert_eq!(ok_report["action"], "validate_pnp");
    assert_eq!(ok_report["matches_expected"], true);

    std::fs::write(
        &pnp_path,
        format!(
            "reference,x_nm,y_nm,rotation_deg,layer,side,package_uuid,part_uuid,value,locked\n\
             U1,1000,1500,90,31,bottom,{u1_package_uuid},{u1_part_uuid},MCU,false\n"
        ),
    )
    .expect("drifted pnp should write");

    let drift = execute_with_exit_code(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "validate-pnp",
            root.to_str().unwrap(),
            "--pnp",
            pnp_path.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("validate should succeed");
    let drift_report: serde_json::Value = serde_json::from_str(&drift.0).expect("report JSON");
    assert_eq!(drift.1, 1);
    assert_eq!(drift_report["matches_expected"], false);

    let _ = std::fs::remove_dir_all(&root);
}
