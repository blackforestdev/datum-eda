use super::*;
use eda_engine::ir::serialization::to_json_deterministic;

fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}", label, Uuid::new_v4()))
}

fn write_manufacturing_validation_board(root: &Path) {
    let board_json = root.join("board/board.json");
    let component_uuid = Uuid::new_v4();
    let package_uuid = Uuid::new_v4();
    let part_uuid = Uuid::new_v4();
    let via_uuid = Uuid::new_v4();
    let net_uuid = Uuid::new_v4();
    std::fs::write(
        &board_json,
        format!(
            "{}\n",
            to_json_deterministic(&serde_json::json!({
                "schema_version": 1,
                "uuid": Uuid::new_v4(),
                "name": "Manufacturing Validate Demo Board",
                "stackup": {
                    "layers": [
                        { "id": 1, "name": "Top Copper", "layer_type": "Copper", "thickness_nm": 35000 },
                        { "id": 2, "name": "Top Mask", "layer_type": "SolderMask", "thickness_nm": 10000 },
                        { "id": 3, "name": "Top Silk", "layer_type": "Silkscreen", "thickness_nm": 10000 },
                        { "id": 4, "name": "Top Paste", "layer_type": "Paste", "thickness_nm": 10000 },
                        { "id": 31, "name": "Bottom Copper", "layer_type": "Copper", "thickness_nm": 35000 },
                        { "id": 41, "name": "Mechanical 41", "layer_type": "Mechanical", "thickness_nm": 0 }
                    ]
                },
                "outline": {
                    "vertices": [
                        { "x": 0, "y": 0 },
                        { "x": 1000, "y": 0 },
                        { "x": 1000, "y": 500 },
                        { "x": 0, "y": 500 }
                    ],
                    "closed": true
                },
                "packages": {
                    component_uuid.to_string(): {
                        "uuid": component_uuid,
                        "reference": "U1",
                        "value": "MCU",
                        "package": package_uuid,
                        "part": part_uuid,
                        "position": { "x": 1000000, "y": 2000000 },
                        "rotation": 0,
                        "layer": 1,
                        "locked": false
                    }
                },
                "component_pads": {
                    component_uuid.to_string(): [
                        {
                            "uuid": Uuid::new_v4(),
                            "name": "P1",
                            "position": { "x": 1000000, "y": 1500000 },
                            "padstack": Uuid::new_v4(),
                            "layer": 1,
                            "drill_nm": 300000,
                            "shape": "circle",
                            "diameter_nm": 600000,
                            "width_nm": 0,
                            "height_nm": 0
                        }
                    ]
                },
                "component_silkscreen": {},
                "component_silkscreen_texts": {},
                "component_silkscreen_arcs": {},
                "component_silkscreen_circles": {},
                "component_silkscreen_polygons": {},
                "component_silkscreen_polylines": {},
                "component_mechanical_lines": {},
                "component_mechanical_texts": {},
                "component_mechanical_polygons": {},
                "component_mechanical_polylines": {},
                "component_mechanical_circles": {},
                "component_mechanical_arcs": {},
                "component_models_3d": {},
                "pads": {},
                "tracks": {},
                "vias": {
                    via_uuid.to_string(): {
                        "uuid": via_uuid,
                        "net": net_uuid,
                        "position": { "x": 500000, "y": 500000 },
                        "diameter": 700000,
                        "drill": 300000,
                        "from_layer": 1,
                        "to_layer": 31
                    }
                },
                "zones": {},
                "nets": {
                    net_uuid.to_string(): {
                        "uuid": net_uuid,
                        "name": "N$1",
                        "class": null
                    }
                },
                "net_classes": {},
                "keepouts": [],
                "dimensions": [],
                "texts": [{
                    "uuid": Uuid::new_v4(),
                    "text": "TOP",
                    "position": { "x": 1000000, "y": 2000000 },
                    "rotation": 0,
                    "height_nm": 1000000,
                    "stroke_width_nm": 120000,
                    "layer": 3
                }]
            }))
            .expect("canonical serialization should succeed")
        ),
    )
    .expect("board file should write");
}

#[test]
fn project_validate_manufacturing_set_passes_for_clean_export() {
    let root = unique_project_root("datum-eda-cli-project-manufacturing-validate-pass");
    create_native_project(&root, Some("Manufacturing Validate Demo".to_string()))
        .expect("initial scaffold should succeed");
    write_manufacturing_validation_board(&root);

    let output_dir = root.join("manufacturing");
    execute(
        Cli::try_parse_from([
            "eda",
            "project",
            "export-manufacturing-set",
            root.to_str().unwrap(),
            "--output-dir",
            output_dir.to_str().unwrap(),
            "--prefix",
            "Release A",
        ])
        .expect("CLI should parse"),
    )
    .expect("export should succeed");

    let (output, exit_code) = execute_with_exit_code(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "validate-manufacturing-set",
            root.to_str().unwrap(),
            "--output-dir",
            output_dir.to_str().unwrap(),
            "--prefix",
            "Release A",
        ])
        .expect("CLI should parse"),
    )
    .expect("validation should run");
    let report: serde_json::Value = serde_json::from_str(&output).expect("report JSON");
    assert_eq!(exit_code, 0);
    assert_eq!(report["action"], "validate_manufacturing_set");
    assert_eq!(report["expected_count"], 11);
    assert_eq!(report["matched_count"], 11);
    assert_eq!(report["missing_count"], 0);
    assert_eq!(report["mismatched_count"], 0);
    assert_eq!(report["extra_count"], 0);

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_validate_manufacturing_set_reports_missing_mismatched_and_extra_files() {
    let root = unique_project_root("datum-eda-cli-project-manufacturing-validate-drift");
    create_native_project(&root, Some("Manufacturing Validate Demo".to_string()))
        .expect("initial scaffold should succeed");
    write_manufacturing_validation_board(&root);

    let output_dir = root.join("manufacturing");
    execute(
        Cli::try_parse_from([
            "eda",
            "project",
            "export-manufacturing-set",
            root.to_str().unwrap(),
            "--output-dir",
            output_dir.to_str().unwrap(),
            "--prefix",
            "Release A",
        ])
        .expect("CLI should parse"),
    )
    .expect("export should succeed");

    std::fs::remove_file(output_dir.join("release-a-pnp.csv")).expect("pnp should remove");
    std::fs::write(
        output_dir.join("release-a-bom.csv"),
        "reference,value,part_uuid,package_uuid,layer,x_nm,y_nm,rotation_deg,locked\nU1,BAD,part,package,1,1,2,0,false\n",
    )
    .expect("bom should rewrite");
    std::fs::write(output_dir.join("extra.txt"), "extra").expect("extra file should write");

    let (output, exit_code) = execute_with_exit_code(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "validate-manufacturing-set",
            root.to_str().unwrap(),
            "--output-dir",
            output_dir.to_str().unwrap(),
            "--prefix",
            "Release A",
        ])
        .expect("CLI should parse"),
    )
    .expect("validation should run");
    let report: serde_json::Value = serde_json::from_str(&output).expect("report JSON");
    assert_eq!(exit_code, 1);
    assert_eq!(report["missing_count"], 1);
    assert_eq!(report["mismatched_count"], 1);
    assert_eq!(report["extra_count"], 1);

    let _ = std::fs::remove_dir_all(&root);
}
