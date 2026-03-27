use super::*;
use eda_engine::ir::serialization::to_json_deterministic;

fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}", label, Uuid::new_v4()))
}

#[test]
fn project_compare_pnp_reports_matched_missing_extra_and_drift() {
    let root = unique_project_root("datum-eda-cli-project-pnp-compare");
    create_native_project(&root, Some("PnP Compare Demo".to_string()))
        .expect("initial scaffold should succeed");

    let board_json = root.join("board/board.json");
    let u2_uuid = Uuid::new_v4();
    let u1_uuid = Uuid::new_v4();
    let u1_part_uuid = Uuid::new_v4();
    let u2_part_uuid = Uuid::new_v4();
    let u1_package_uuid = Uuid::new_v4();
    let u2_package_uuid = Uuid::new_v4();
    std::fs::write(
        &board_json,
        format!(
            "{}\n",
            to_json_deterministic(&serde_json::json!({
                "schema_version": 1,
                "uuid": Uuid::new_v4(),
                "name": "PnP Compare Demo Board",
                "stackup": { "layers": [] },
                "outline": { "vertices": [], "closed": true },
                "packages": {
                    u2_uuid.to_string(): {
                        "uuid": u2_uuid,
                        "part": u2_part_uuid,
                        "package": u2_package_uuid,
                        "reference": "U2",
                        "value": "MCU",
                        "position": { "x": 2000, "y": 3000 },
                        "rotation": 180,
                        "layer": 31,
                        "locked": true
                    },
                    u1_uuid.to_string(): {
                        "uuid": u1_uuid,
                        "part": u1_part_uuid,
                        "package": u1_package_uuid,
                        "reference": "U1",
                        "value": "ADC",
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
    std::fs::write(
        &pnp_path,
        format!(
            "reference,x_nm,y_nm,rotation_deg,layer,side,package_uuid,part_uuid,value,locked\nU1,1000,1500,90,1,top,{u1_package_uuid},{u1_part_uuid},ADC,false\nU2,2500,3000,180,31,bottom,{u2_package_uuid},{u2_part_uuid},MCU,true\nU3,4000,5000,0,1,top,{u1_package_uuid},{u1_part_uuid},SPARE,false\n"
        ),
    )
    .expect("pnp file should write");

    let cli = Cli::try_parse_from([
        "eda", "--format", "json", "project", "compare-pnp",
        root.to_str().unwrap(), "--pnp", pnp_path.to_str().unwrap(),
    ])
    .expect("CLI should parse");
    let output = execute(cli).expect("PnP compare should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).expect("report JSON");
    assert_eq!(report["action"], "compare_pnp");
    assert_eq!(report["expected_count"], 2);
    assert_eq!(report["actual_count"], 3);
    assert_eq!(report["matched_count"], 1);
    assert_eq!(report["missing_count"], 0);
    assert_eq!(report["extra_count"], 1);
    assert_eq!(report["drift_count"], 1);
    assert_eq!(report["matched"][0], "U1");
    assert_eq!(report["extra"][0], "U3");
    assert_eq!(report["drift"][0]["reference"], "U2");
    assert_eq!(report["drift"][0]["fields"][0], "position");

    let _ = std::fs::remove_dir_all(&root);
}
