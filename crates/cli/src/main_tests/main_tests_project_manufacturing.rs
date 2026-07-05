use super::*;
use eda_engine::ir::serialization::to_json_deterministic;

fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}", label, Uuid::new_v4()))
}

#[test]
fn project_report_manufacturing_summarizes_supported_outputs_from_persisted_state() {
    let root = unique_project_root("datum-eda-cli-project-manufacturing-report");
    create_native_project(&root, Some("Manufacturing Report Demo".to_string()))
        .expect("initial scaffold should succeed");

    let board_json = root.join("board/board.json");
    let component_uuid = Uuid::new_v4();
    std::fs::write(
        &board_json,
        format!(
            "{}\n",
            to_json_deterministic(&serde_json::json!({
                "schema_version": 1,
                "uuid": Uuid::new_v4(),
                "name": "Manufacturing Report Demo Board",
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
                        "package": Uuid::new_v4(),
                        "part": Uuid::new_v4(),
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
                        },
                        {
                            "uuid": Uuid::new_v4(),
                            "name": "P2",
                            "position": { "x": 2000000, "y": 2500000 },
                            "padstack": Uuid::new_v4(),
                            "layer": 1,
                            "drill_nm": null,
                            "shape": "rect",
                            "diameter_nm": 0,
                            "width_nm": 800000,
                            "height_nm": 500000
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
                    Uuid::new_v4().to_string(): {
                        "uuid": Uuid::new_v4(),
                        "net": Uuid::new_v4(),
                        "position": { "x": 500000, "y": 500000 },
                        "diameter": 700000,
                        "drill": 300000,
                        "from_layer": 1,
                        "to_layer": 31
                    }
                },
                "zones": {},
                "nets": {},
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

    let cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "report-manufacturing",
        root.to_str().unwrap(),
        "--prefix",
        "Release A",
    ])
    .expect("CLI should parse");
    let output = execute(cli).expect("manufacturing report should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).expect("report JSON");
    assert_eq!(report["action"], "report_manufacturing");
    assert_eq!(report["prefix"], "release-a");
    assert_eq!(report["bom_component_count"], 1);
    assert_eq!(report["pnp_component_count"], 1);
    assert_eq!(report["drill_csv_row_count"], 1);
    assert_eq!(report["excellon_via_count"], 1);
    assert_eq!(report["excellon_component_pad_count"], 1);
    assert_eq!(report["excellon_hit_count"], 2);
    assert_eq!(report["drill_hole_class_count"], 1);
    assert_eq!(report["gerber_artifact_count"], 7);
    let gerber_artifacts = report["gerber_artifacts"]
        .as_array()
        .expect("gerber artifact array");
    assert!(
        gerber_artifacts
            .iter()
            .any(|artifact| artifact["filename"] == "release-a-outline.gbr")
    );
    assert!(
        gerber_artifacts
            .iter()
            .any(|artifact| artifact["filename"] == "release-a-l1-top-copper-copper.gbr")
    );
    assert!(
        gerber_artifacts
            .iter()
            .any(|artifact| artifact["filename"] == "release-a-l2-top-mask-mask.gbr")
    );
    assert!(
        gerber_artifacts
            .iter()
            .any(|artifact| artifact["filename"] == "release-a-l3-top-silk-silk.gbr")
    );
    assert!(
        gerber_artifacts
            .iter()
            .any(|artifact| artifact["filename"] == "release-a-l4-top-paste-paste.gbr")
    );
    assert!(
        gerber_artifacts
            .iter()
            .any(|artifact| artifact["filename"] == "release-a-l31-bottom-copper-copper.gbr")
    );
    assert!(
        gerber_artifacts
            .iter()
            .any(|artifact| artifact["filename"] == "release-a-l41-mechanical-41-mech.gbr")
    );

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_bom_and_pnp_are_keyed_by_component_instance_identity() {
    let root = unique_project_root("datum-eda-cli-project-manufacturing-component-instance");
    create_native_project(
        &root,
        Some("Component Instance Manufacturing Demo".to_string()),
    )
    .expect("initial scaffold should succeed");
    let project: serde_json::Value =
        serde_json::from_slice(&std::fs::read(root.join("project.json")).unwrap()).unwrap();
    let project_id = Uuid::parse_str(project["uuid"].as_str().unwrap()).unwrap();
    let part_id = Uuid::new_v4();
    let symbol_id = Uuid::new_v4();
    let package_id = Uuid::new_v4();
    let package_footprint_id = Uuid::new_v4();
    let component_instance_id = Uuid::new_v4();

    let mut schematic: serde_json::Value =
        serde_json::from_slice(&std::fs::read(root.join("schematic/schematic.json")).unwrap())
            .unwrap();
    let sheet_id = Uuid::new_v4();
    let sheet_path = format!("sheets/{sheet_id}.json");
    schematic["sheets"][sheet_id.to_string()] = serde_json::Value::String(sheet_path.clone());
    std::fs::write(
        root.join("schematic/schematic.json"),
        format!("{}\n", to_json_deterministic(&schematic).unwrap()),
    )
    .unwrap();
    let sheet_file = root.join("schematic").join(&sheet_path);
    std::fs::create_dir_all(sheet_file.parent().unwrap()).unwrap();
    std::fs::write(
        &sheet_file,
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
                        "entity": Uuid::new_v5(&project_id, b"entity"),
                        "gate": Uuid::new_v5(&project_id, b"gate"),
                        "lib_id": "test:U",
                        "reference": "U1",
                        "value": "MCU",
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
            .unwrap()
        ),
    )
    .unwrap();

    let mut board: serde_json::Value =
        serde_json::from_slice(&std::fs::read(root.join("board/board.json")).unwrap()).unwrap();
    board["packages"][package_id.to_string()] = serde_json::json!({
        "uuid": package_id,
        "reference": "U1",
        "value": "MCU",
        "package": package_footprint_id,
        "part": part_id,
        "position": { "x": 1000000, "y": 2000000 },
        "rotation": 90,
        "layer": 1,
        "locked": false
    });
    std::fs::write(
        root.join("board/board.json"),
        format!("{}\n", to_json_deterministic(&board).unwrap()),
    )
    .unwrap();
    let component_instance_dir = root.join(".datum/component_instances");
    std::fs::create_dir_all(&component_instance_dir).unwrap();
    std::fs::write(
        component_instance_dir.join(format!("{component_instance_id}.json")),
        format!(
            "{}\n",
            to_json_deterministic(&serde_json::json!({
                "schema_version": 1,
                "component_instance": {
                    "uuid": component_instance_id,
                    "object_revision": 0,
                    "placed_symbol_refs": [{ "object_id": symbol_id, "object_revision": 0 }],
                    "placed_package_refs": [{ "object_id": package_id, "object_revision": 0 }],
                    "placed_package_roles": {
                        package_id.to_string(): { "role": "physical_package", "label": "main" }
                    }
                }
            }))
            .unwrap()
        ),
    )
    .unwrap();

    let bom_path = root.join("component-instance-bom.csv");
    let pnp_path = root.join("component-instance-pnp.csv");
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

    let bom_csv = std::fs::read_to_string(&bom_path).unwrap();
    let pnp_csv = std::fs::read_to_string(&pnp_path).unwrap();
    let role_header =
        "component_instance_uuid,component_instance_role,component_instance_label,reference,";
    assert!(bom_csv.starts_with(role_header));
    assert!(pnp_csv.starts_with(role_header));
    let role_row = format!("{component_instance_id},physical_package,main,U1,");
    assert!(bom_csv.contains(&role_row));
    assert!(pnp_csv.contains(&role_row));

    std::fs::write(
        &bom_path,
        bom_csv.replace(",physical_package,main,", ",alternate,backup,"),
    )
    .unwrap();
    std::fs::write(
        &pnp_path,
        pnp_csv.replace(",physical_package,main,", ",alternate,backup,"),
    )
    .unwrap();

    let bom_compare = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "compare-bom",
            root.to_str().unwrap(),
            "--bom",
            bom_path.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("BOM compare should succeed");
    let bom_report: serde_json::Value = serde_json::from_str(&bom_compare).unwrap();
    assert_eq!(bom_report["missing_count"], 0);
    assert_eq!(bom_report["extra_count"], 0);
    assert_eq!(bom_report["drift_count"], 1);
    assert_eq!(
        bom_report["drift"][0]["identity"],
        component_instance_id.to_string()
    );
    assert_eq!(
        bom_report["drift"][0]["component_instance_uuid"],
        component_instance_id.to_string()
    );
    let bom_fields = bom_report["drift"][0]["fields"].as_array().unwrap();
    assert!(
        bom_fields
            .iter()
            .any(|field| field == "component_instance_role")
    );
    assert!(
        bom_fields
            .iter()
            .any(|field| field == "component_instance_label")
    );
    let pnp_compare = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "compare-pnp",
            root.to_str().unwrap(),
            "--pnp",
            pnp_path.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("PnP compare should succeed");
    let pnp_report: serde_json::Value = serde_json::from_str(&pnp_compare).unwrap();
    assert_eq!(pnp_report["missing_count"], 0);
    assert_eq!(pnp_report["extra_count"], 0);
    assert_eq!(pnp_report["drift_count"], 1);
    assert_eq!(
        pnp_report["drift"][0]["identity"],
        component_instance_id.to_string()
    );
    assert_eq!(
        pnp_report["drift"][0]["component_instance_uuid"],
        component_instance_id.to_string()
    );
    let pnp_fields = pnp_report["drift"][0]["fields"].as_array().unwrap();
    assert!(
        pnp_fields
            .iter()
            .any(|field| field == "component_instance_role")
    );
    assert!(
        pnp_fields
            .iter()
            .any(|field| field == "component_instance_label")
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_bom_and_pnp_variant_filter_uses_component_instance_population() {
    let root = unique_project_root("datum-eda-cli-project-manufacturing-variant-filter");
    create_native_project(&root, Some("Variant Manufacturing Demo".to_string()))
        .expect("initial scaffold should succeed");
    let project: serde_json::Value =
        serde_json::from_slice(&std::fs::read(root.join("project.json")).unwrap()).unwrap();
    let project_id = Uuid::parse_str(project["uuid"].as_str().unwrap()).unwrap();

    let u1_part_id = Uuid::new_v4();
    let u2_part_id = Uuid::new_v4();
    let u1_symbol_id = Uuid::new_v4();
    let u2_symbol_id = Uuid::new_v4();
    let u1_package_id = Uuid::new_v4();
    let u2_package_id = Uuid::new_v4();
    let u1_component_instance_id = Uuid::new_v4();
    let u2_component_instance_id = Uuid::new_v4();

    let mut schematic: serde_json::Value =
        serde_json::from_slice(&std::fs::read(root.join("schematic/schematic.json")).unwrap())
            .unwrap();
    let sheet_id = Uuid::new_v4();
    let sheet_path = format!("sheets/{sheet_id}.json");
    schematic["sheets"][sheet_id.to_string()] = serde_json::Value::String(sheet_path.clone());
    std::fs::write(
        root.join("schematic/schematic.json"),
        format!("{}\n", to_json_deterministic(&schematic).unwrap()),
    )
    .unwrap();
    let sheet_file = root.join("schematic").join(&sheet_path);
    std::fs::create_dir_all(sheet_file.parent().unwrap()).unwrap();
    std::fs::write(
        &sheet_file,
        format!(
            "{}\n",
            to_json_deterministic(&serde_json::json!({
                "schema_version": 1,
                "uuid": sheet_id,
                "name": "Main",
                "symbols": {
                    u1_symbol_id.to_string(): {
                        "uuid": u1_symbol_id,
                        "part": u1_part_id,
                        "entity": Uuid::new_v5(&project_id, b"u1-entity"),
                        "gate": Uuid::new_v5(&project_id, b"u1-gate"),
                        "lib_id": "test:U",
                        "reference": "U1",
                        "value": "FITTED",
                        "fields": [],
                        "pins": [],
                        "position": { "x": 0, "y": 0 },
                        "rotation": 0,
                        "mirrored": false,
                        "unit_selection": null,
                        "display_mode": "LibraryDefault",
                        "pin_overrides": [],
                        "hidden_power_behavior": "SourceDefinedImplicit"
                    },
                    u2_symbol_id.to_string(): {
                        "uuid": u2_symbol_id,
                        "part": u2_part_id,
                        "entity": Uuid::new_v5(&project_id, b"u2-entity"),
                        "gate": Uuid::new_v5(&project_id, b"u2-gate"),
                        "lib_id": "test:U",
                        "reference": "U2",
                        "value": "UNFITTED",
                        "fields": [],
                        "pins": [],
                        "position": { "x": 1000000, "y": 0 },
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
            .unwrap()
        ),
    )
    .unwrap();

    let mut board: serde_json::Value =
        serde_json::from_slice(&std::fs::read(root.join("board/board.json")).unwrap()).unwrap();
    board["packages"][u1_package_id.to_string()] = serde_json::json!({
        "uuid": u1_package_id,
        "reference": "U1",
        "value": "FITTED",
        "package": Uuid::new_v4(),
        "part": u1_part_id,
        "position": { "x": 1000000, "y": 2000000 },
        "rotation": 0,
        "layer": 1,
        "locked": false
    });
    board["packages"][u2_package_id.to_string()] = serde_json::json!({
        "uuid": u2_package_id,
        "reference": "U2",
        "value": "UNFITTED",
        "package": Uuid::new_v4(),
        "part": u2_part_id,
        "position": { "x": 3000000, "y": 4000000 },
        "rotation": 180,
        "layer": 1,
        "locked": false
    });
    board["outline"] = serde_json::json!({
        "vertices": [
            { "x": 0, "y": 0 },
            { "x": 5000000, "y": 0 },
            { "x": 5000000, "y": 3000000 },
            { "x": 0, "y": 3000000 }
        ],
        "closed": true
    });
    std::fs::write(
        root.join("board/board.json"),
        format!("{}\n", to_json_deterministic(&board).unwrap()),
    )
    .unwrap();

    let component_instance_dir = root.join(".datum/component_instances");
    std::fs::create_dir_all(&component_instance_dir).unwrap();
    for (component_instance_id, symbol_id, package_id) in [
        (u1_component_instance_id, u1_symbol_id, u1_package_id),
        (u2_component_instance_id, u2_symbol_id, u2_package_id),
    ] {
        std::fs::write(
            component_instance_dir.join(format!("{component_instance_id}.json")),
            format!(
                "{}\n",
                to_json_deterministic(&serde_json::json!({
                    "schema_version": 1,
                    "component_instance": {
                        "uuid": component_instance_id,
                        "object_revision": 0,
                        "placed_symbol_refs": [{ "object_id": symbol_id, "object_revision": 0 }],
                        "placed_package_refs": [{ "object_id": package_id, "object_revision": 0 }]
                    }
                }))
                .unwrap()
            ),
        )
        .unwrap();
    }

    let before_variant = eda_engine::substrate::ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves before variant");
    let variant_id = Uuid::new_v4();
    let variant_path = root.join(".datum/variants/no-u2.json");
    std::fs::create_dir_all(variant_path.parent().unwrap()).unwrap();
    std::fs::write(
        &variant_path,
        format!(
            "{}\n",
            to_json_deterministic(&serde_json::json!({
                "schema_version": 1,
                "variants": [{
                    "id": variant_id,
                    "name": "No U2",
                    "base_model_revision": before_variant.model_revision,
                    "variant_revision": 0,
                    "fitted": {
                        u1_component_instance_id.to_string(): "fitted",
                        u2_component_instance_id.to_string(): "unfitted"
                    },
                    "relationship_overrides": {},
                    "property_overrides": {}
                }]
            }))
            .unwrap()
        ),
    )
    .unwrap();

    let manufacturing_dir = root.join("variant-manufacturing");
    execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "export-manufacturing-set",
            root.to_str().unwrap(),
            "--output-dir",
            manufacturing_dir.to_str().unwrap(),
            "--prefix",
            "Variant Release",
            "--variant",
            &variant_id.to_string(),
        ])
        .expect("CLI should parse"),
    )
    .expect("variant manufacturing export should succeed");
    let manufacturing_bom =
        std::fs::read_to_string(manufacturing_dir.join("variant-release-bom.csv")).unwrap();
    let manufacturing_pnp =
        std::fs::read_to_string(manufacturing_dir.join("variant-release-pnp.csv")).unwrap();
    assert!(manufacturing_bom.contains(&format!("{u1_component_instance_id},,,U1,")));
    assert!(!manufacturing_bom.contains(&format!("{u2_component_instance_id},,,U2,")));
    assert!(manufacturing_pnp.contains(&format!("{u1_component_instance_id},,,U1,")));
    assert!(!manufacturing_pnp.contains(&format!("{u2_component_instance_id},,,U2,")));

    let validation = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "validate-manufacturing-set",
            root.to_str().unwrap(),
            "--output-dir",
            manufacturing_dir.to_str().unwrap(),
            "--prefix",
            "Variant Release",
            "--variant",
            &variant_id.to_string(),
        ])
        .expect("CLI should parse"),
    )
    .expect("variant manufacturing validation should succeed");
    let validation_report: serde_json::Value = serde_json::from_str(&validation).unwrap();
    assert_eq!(validation_report["missing_count"], 0);
    assert_eq!(validation_report["mismatched_count"], 0);
    assert_eq!(validation_report["extra_count"], 0);

    let comparison = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "compare-manufacturing-set",
            root.to_str().unwrap(),
            "--output-dir",
            manufacturing_dir.to_str().unwrap(),
            "--prefix",
            "Variant Release",
            "--variant",
            &variant_id.to_string(),
        ])
        .expect("CLI should parse"),
    )
    .expect("variant manufacturing comparison should succeed");
    let comparison_report: serde_json::Value = serde_json::from_str(&comparison).unwrap();
    assert_eq!(comparison_report["missing_count"], 0);
    assert_eq!(comparison_report["mismatched_count"], 0);
    assert_eq!(comparison_report["extra_count"], 0);

    let _ = std::fs::remove_dir_all(&root);
}
