use super::*;
use eda_engine::ir::serialization::to_json_deterministic;

fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}", label, Uuid::new_v4()))
}

#[test]
fn project_compare_bom_reports_matches_extra_and_drift() {
    let root = unique_project_root("datum-eda-cli-project-bom-compare");
    create_native_project(&root, Some("BOM Compare Demo".to_string()))
        .expect("initial scaffold should succeed");

    let board_json = root.join("board/board.json");
    let u1_uuid = Uuid::new_v4();
    let u2_uuid = Uuid::new_v4();
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
                "name": "BOM Compare Demo Board",
                "stackup": { "layers": [] },
                "outline": { "vertices": [], "closed": true },
                "packages": {
                    u1_uuid.to_string(): {
                        "uuid": u1_uuid,
                        "part": u1_part_uuid,
                        "package": u1_package_uuid,
                        "reference": "U1",
                        "value": "OPA1642",
                        "position": { "x": 1000, "y": 2000 },
                        "rotation": 90,
                        "layer": 1,
                        "locked": false
                    },
                    u2_uuid.to_string(): {
                        "uuid": u2_uuid,
                        "part": u2_part_uuid,
                        "package": u2_package_uuid,
                        "reference": "U2",
                        "value": "TL072",
                        "position": { "x": 3000, "y": 4000 },
                        "rotation": 180,
                        "layer": 31,
                        "locked": true
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
    std::fs::write(
        &bom_path,
        format!(
            "reference,value,part_uuid,package_uuid,layer,x_nm,y_nm,rotation_deg,locked\n\
             U1,OPA1642,{u1_part_uuid},{u1_package_uuid},1,1000,2000,90,false\n\
             U2,TL074,{u2_part_uuid},{u2_package_uuid},31,3000,4000,180,true\n\
             U3,NE5532,{u2_part_uuid},{u2_package_uuid},1,5000,6000,0,false\n"
        ),
    )
    .expect("bom should write");

    let cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "compare-bom",
        root.to_str().unwrap(),
        "--bom",
        bom_path.to_str().unwrap(),
    ])
    .expect("CLI should parse");
    let output = execute(cli).expect("BOM compare should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).expect("report JSON");
    assert_eq!(report["action"], "compare_bom");
    assert_eq!(report["expected_count"], 2);
    assert_eq!(report["actual_count"], 3);
    assert_eq!(report["matched_count"], 1);
    assert_eq!(report["missing_count"], 0);
    assert_eq!(report["extra_count"], 1);
    assert_eq!(report["drift_count"], 1);
    assert_eq!(report["matched"][0], "U1");
    assert_eq!(report["extra"][0], "U3");
    assert_eq!(report["drift"][0]["reference"], "U2");
    assert_eq!(report["drift"][0]["fields"][0], "value");

    let _ = std::fs::remove_dir_all(&root);
}
