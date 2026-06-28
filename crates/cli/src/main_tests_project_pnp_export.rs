use super::*;
use eda_engine::ir::serialization::to_json_deterministic;

fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}", label, Uuid::new_v4()))
}

#[test]
fn project_export_pnp_writes_deterministic_csv_from_board_components() {
    let root = unique_project_root("datum-eda-cli-project-pnp-export");
    create_native_project(&root, Some("PnP Export Demo".to_string()))
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
                "name": "PnP Export Demo Board",
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
                        "value": "SOIC-8, \"Analog\"",
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
    let cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "export-pnp",
        root.to_str().unwrap(),
        "--out",
        pnp_path.to_str().unwrap(),
    ])
    .expect("CLI should parse");
    let output = execute(cli).expect("PnP export should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).expect("report JSON");
    assert_eq!(report["action"], "export_pnp");
    assert_eq!(report["production_classification"], "manual_debug_export");
    assert_eq!(report["rows"], 2);

    let csv = std::fs::read_to_string(&pnp_path).expect("pnp should read");
    let lines = csv.lines().collect::<Vec<_>>();
    assert_eq!(
        lines[0],
        "component_instance_uuid,component_instance_role,component_instance_label,reference,x_nm,y_nm,rotation_deg,layer,side,package_uuid,part_uuid,value,locked"
    );
    assert_eq!(
        lines[1],
        format!(
            ",,,U1,1000,1500,90,1,top,{u1_package_uuid},{u1_part_uuid},\"SOIC-8, \"\"Analog\"\"\",false"
        )
    );
    assert_eq!(
        lines[2],
        format!(",,,U2,2000,3000,180,31,bottom,{u2_package_uuid},{u2_part_uuid},MCU,true")
    );

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_export_pnp_reads_resolver_materialized_board_state() {
    let root = unique_project_root("datum-eda-cli-project-pnp-export-resolver");
    create_native_project(&root, Some("PnP Export Resolver Demo".to_string()))
        .expect("initial scaffold should succeed");
    let board_json = root.join("board/board.json");
    let stale_board = std::fs::read_to_string(&board_json).expect("board file should read");
    let part_uuid = Uuid::new_v4();
    let package_uuid = Uuid::new_v4();

    execute(
        Cli::try_parse_from([
            "eda",
            "project",
            "place-board-component",
            root.to_str().unwrap(),
            "--part",
            &part_uuid.to_string(),
            "--package",
            &package_uuid.to_string(),
            "--reference",
            "U1",
            "--value",
            "MCU",
            "--x-nm",
            "1000",
            "--y-nm",
            "2000",
            "--layer",
            "1",
        ])
        .expect("CLI should parse"),
    )
    .expect("place board component should succeed");
    std::fs::write(&board_json, stale_board).expect("stale board file should restore");

    let pnp_path = root.join("pnp-resolved.csv");
    let output = execute(
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
    .expect("PnP export should read resolver state");
    let report: serde_json::Value = serde_json::from_str(&output).expect("report JSON");
    assert_eq!(report["rows"], 1);

    let csv = std::fs::read_to_string(&pnp_path).expect("pnp should read");
    assert!(csv.contains(&format!(
        ",,,U1,1000,2000,0,1,top,{package_uuid},{part_uuid},MCU,false"
    )));

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_pnp_inspect_accepts_role_columns_and_legacy_header() {
    let root = unique_project_root("datum-eda-cli-project-pnp-role-columns");
    create_native_project(&root, Some("PnP Role Column Demo".to_string()))
        .expect("initial scaffold should succeed");
    let pnp_path = root.join("pnp-role.csv");
    std::fs::write(
        &pnp_path,
        "component_instance_uuid,component_instance_role,component_instance_label,reference,x_nm,y_nm,rotation_deg,layer,side,package_uuid,part_uuid,value,locked\nci,alternate,backup,U1,0,0,0,1,top,pkg,part,MCU,false\n",
    )
    .unwrap();
    let inspect = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "inspect-pnp",
            pnp_path.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("inspect should parse role columns");
    let report: serde_json::Value = serde_json::from_str(&inspect).unwrap();
    assert_eq!(report["rows"][0]["component_instance_role"], "alternate");
    assert_eq!(report["rows"][0]["component_instance_label"], "backup");

    std::fs::write(
        &pnp_path,
        "component_instance_uuid,reference,x_nm,y_nm,rotation_deg,layer,side,package_uuid,part_uuid,value,locked\nci,U1,0,0,0,1,top,pkg,part,MCU,false\n",
    )
    .unwrap();
    let legacy = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "inspect-pnp",
            pnp_path.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("inspect should parse legacy component-instance header");
    let legacy_report: serde_json::Value = serde_json::from_str(&legacy).unwrap();
    assert!(legacy_report["rows"][0]["component_instance_role"].is_null());

    let _ = std::fs::remove_dir_all(&root);
}
