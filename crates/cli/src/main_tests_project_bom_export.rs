use super::*;
use eda_engine::ir::serialization::to_json_deterministic;

fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}", label, Uuid::new_v4()))
}

#[test]
fn project_export_bom_writes_deterministic_csv_from_board_components() {
    let root = unique_project_root("datum-eda-cli-project-bom-export");
    create_native_project(&root, Some("BOM Export Demo".to_string()))
        .expect("initial scaffold should succeed");

    let board_json = root.join("board/board.json");
    let c2_uuid = Uuid::new_v4();
    let c1_uuid = Uuid::new_v4();
    let c1_part_uuid = Uuid::new_v4();
    let c2_part_uuid = Uuid::new_v4();
    let c1_package_uuid = Uuid::new_v4();
    let c2_package_uuid = Uuid::new_v4();
    std::fs::write(
        &board_json,
        format!(
            "{}\n",
            to_json_deterministic(&serde_json::json!({
                "schema_version": 1,
                "uuid": Uuid::new_v4(),
                "name": "BOM Export Demo Board",
                "stackup": { "layers": [] },
                "outline": { "vertices": [], "closed": true },
                "packages": {
                    c2_uuid.to_string(): {
                        "uuid": c2_uuid,
                        "part": c2_part_uuid,
                        "package": c2_package_uuid,
                        "reference": "C2",
                        "value": "10uF",
                        "position": { "x": 2000, "y": 3000 },
                        "rotation": 180,
                        "layer": 31,
                        "locked": true
                    },
                    c1_uuid.to_string(): {
                        "uuid": c1_uuid,
                        "part": c1_part_uuid,
                        "package": c1_package_uuid,
                        "reference": "C1",
                        "value": "1uF, \"X7R\"",
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
    let cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "export-bom",
        root.to_str().unwrap(),
        "--out",
        bom_path.to_str().unwrap(),
    ])
    .expect("CLI should parse");
    let output = execute(cli).expect("BOM export should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).expect("report JSON");
    assert_eq!(report["action"], "export_bom");
    assert_eq!(report["rows"], 2);

    let csv = std::fs::read_to_string(&bom_path).expect("bom should read");
    let lines = csv.lines().collect::<Vec<_>>();
    assert_eq!(
        lines[0],
        "reference,value,part_uuid,package_uuid,layer,x_nm,y_nm,rotation_deg,locked"
    );
    assert_eq!(
        lines[1],
        format!("C1,\"1uF, \"\"X7R\"\"\",{c1_part_uuid},{c1_package_uuid},1,1000,1500,90,false")
    );
    assert_eq!(
        lines[2],
        format!("C2,10uF,{c2_part_uuid},{c2_package_uuid},31,2000,3000,180,true")
    );

    let _ = std::fs::remove_dir_all(&root);
}
