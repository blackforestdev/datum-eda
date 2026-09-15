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
    assert_eq!(report["production_classification"], "manual_debug_export");
    assert_eq!(report["rows"], 2);

    let csv = std::fs::read_to_string(&bom_path).expect("bom should read");
    let lines = csv.lines().collect::<Vec<_>>();
    assert_eq!(
        lines[0],
        "component_instance_uuid,component_instance_role,component_instance_label,reference,value,part_uuid,package_uuid,layer,x_nm,y_nm,rotation_deg,locked"
    );
    assert_eq!(
        lines[1],
        format!(",,,C1,\"1uF, \"\"X7R\"\"\",{c1_part_uuid},{c1_package_uuid},1,1000,1500,90,false")
    );
    assert_eq!(
        lines[2],
        format!(",,,C2,10uF,{c2_part_uuid},{c2_package_uuid},31,2000,3000,180,true")
    );

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_export_bom_reads_resolver_materialized_board_state() {
    let root = unique_project_root("datum-eda-cli-project-bom-export-resolver");
    create_native_project(&root, Some("BOM Export Resolver Demo".to_string()))
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

    let bom_path = root.join("bom-resolved.csv");
    let output = execute(
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
    .expect("BOM export should read resolver state");
    let report: serde_json::Value = serde_json::from_str(&output).expect("report JSON");
    assert_eq!(report["rows"], 1);

    let csv = std::fs::read_to_string(&bom_path).expect("bom should read");
    assert!(csv.contains(&format!(
        ",,,U1,MCU,{part_uuid},{package_uuid},1,1000,2000,0,false"
    )));

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_export_bom_does_not_use_derived_component_instance_identity() {
    let root = unique_project_root("datum-eda-cli-project-bom-derived-ci");
    create_native_project(&root, Some("BOM Derived CI Demo".to_string()))
        .expect("initial scaffold should succeed");
    let project_id = Uuid::new_v4();
    let sheet_id = Uuid::new_v5(&project_id, b"sheet");
    let symbol_id = Uuid::new_v4();
    let package_id = Uuid::new_v4();
    let part_id = Uuid::new_v4();
    let package_type_id = Uuid::new_v4();
    std::fs::write(
        root.join("schematic/schematic.json"),
        format!(
            "{}\n",
            to_json_deterministic(&serde_json::json!({
                "schema_version": 1,
                "uuid": Uuid::new_v5(&project_id, b"schematic"),
                "sheets": { sheet_id.to_string(): "sheets/main.json" },
                "definitions": {},
                "instances": [],
                "variants": {},
                "waivers": []
            }))
            .expect("schematic root should serialize")
        ),
    )
    .expect("schematic root should write");
    std::fs::write(
        root.join("schematic/sheets/main.json"),
        format!(
            "{}\n",
            to_json_deterministic(&serde_json::json!({
                "schema_version": 1,
                "uuid": sheet_id,
                "name": "Main",
                "symbols": {
                    symbol_id.to_string(): {
                        "uuid": symbol_id,
                        "part": part_id,
                        "entity": null,
                        "gate": null,
                        "lib_id": "test:R",
                        "reference": "R1",
                        "value": "10k",
                        "fields": [],
                        "pins": [],
                        "position": { "x": 0, "y": 0 },
                        "rotation": 0,
                        "mirrored": false,
                        "unit_selection": null,
                        "display_mode": "LibraryDefault",
                        "pin_overrides": [],
                        "hidden_power_behavior": "SourceDefinedImplicit"
                    }
                },
                "wires": {},
                "junctions": {},
                "labels": {},
                "buses": {},
                "bus_entries": {},
                "ports": {},
                "noconnects": {},
                "texts": {},
                "drawings": {}
            }))
            .expect("schematic sheet should serialize")
        ),
    )
    .expect("schematic sheet should write");
    std::fs::write(
        root.join("board/board.json"),
        format!(
            "{}\n",
            to_json_deterministic(&serde_json::json!({
                "schema_version": 1,
                "uuid": Uuid::new_v4(),
                "name": "BOM Derived CI Demo Board",
                "stackup": { "layers": [] },
                "outline": { "vertices": [], "closed": true },
                "packages": {
                    package_id.to_string(): {
                        "uuid": package_id,
                        "part": part_id,
                        "package": package_type_id,
                        "reference": "R1",
                        "value": "10k",
                        "position": { "x": 1000, "y": 2000 },
                        "rotation": 0,
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
            .expect("board should serialize")
        ),
    )
    .expect("board should write");

    let bom_path = root.join("bom-derived.csv");
    execute(
        Cli::try_parse_from([
            "eda",
            "project",
            "export-bom",
            root.to_str().unwrap(),
            "--out",
            bom_path.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("BOM export should succeed");
    let csv = std::fs::read_to_string(&bom_path).expect("bom should read");
    assert!(csv.contains(&format!(
        ",,,R1,10k,{part_id},{package_type_id},1,1000,2000,0,false"
    )));

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_bom_compare_and_inspect_accept_role_columns_and_legacy_header() {
    let root = unique_project_root("datum-eda-cli-project-bom-role-columns");
    create_native_project(&root, Some("BOM Role Column Demo".to_string()))
        .expect("initial scaffold should succeed");
    let bom_path = root.join("bom-role.csv");
    std::fs::write(
        &bom_path,
        "component_instance_uuid,component_instance_role,component_instance_label,reference,value,part_uuid,package_uuid,layer,x_nm,y_nm,rotation_deg,locked\nci,alternate,backup,U1,MCU,part,pkg,1,0,0,0,false\n",
    )
    .unwrap();
    let inspect = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "inspect-bom",
            bom_path.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("inspect should parse role columns");
    let report: serde_json::Value = serde_json::from_str(&inspect).unwrap();
    assert_eq!(report["rows"][0]["component_instance_role"], "alternate");
    assert_eq!(report["rows"][0]["component_instance_label"], "backup");

    std::fs::write(
        &bom_path,
        "component_instance_uuid,reference,value,part_uuid,package_uuid,layer,x_nm,y_nm,rotation_deg,locked\nci,U1,MCU,part,pkg,1,0,0,0,false\n",
    )
    .unwrap();
    let legacy = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "inspect-bom",
            bom_path.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("inspect should parse legacy component-instance header");
    let legacy_report: serde_json::Value = serde_json::from_str(&legacy).unwrap();
    assert!(legacy_report["rows"][0]["component_instance_role"].is_null());

    let _ = std::fs::remove_dir_all(&root);
}
