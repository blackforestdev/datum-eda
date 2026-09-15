use super::*;
use eda_engine::ir::serialization::to_json_deterministic;

fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{label}-{}", Uuid::new_v4()))
}

fn write_variant_manufacturing_fixture(root: &Path) -> (Uuid, Uuid, Uuid) {
    let project: serde_json::Value =
        serde_json::from_slice(&std::fs::read(root.join("project.json")).unwrap()).unwrap();
    let project_id = Uuid::parse_str(project["uuid"].as_str().unwrap()).unwrap();
    let u1_part = Uuid::new_v4();
    let u2_part = Uuid::new_v4();
    let u1_symbol = Uuid::new_v4();
    let u2_symbol = Uuid::new_v4();
    let u1_package = Uuid::new_v4();
    let u2_package = Uuid::new_v4();
    let u1_instance = Uuid::new_v4();
    let u2_instance = Uuid::new_v4();
    let sheet_id = Uuid::new_v4();

    let mut schematic: serde_json::Value =
        serde_json::from_slice(&std::fs::read(root.join("schematic/schematic.json")).unwrap())
            .unwrap();
    schematic["sheets"][sheet_id.to_string()] =
        serde_json::Value::String(format!("sheets/{sheet_id}.json"));
    std::fs::write(
        root.join("schematic/schematic.json"),
        format!("{}\n", to_json_deterministic(&schematic).unwrap()),
    )
    .unwrap();
    let sheet_file = root.join(format!("schematic/sheets/{sheet_id}.json"));
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
                    u1_symbol.to_string(): symbol_json(project_id, u1_symbol, u1_part, "U1", "FITTED", b"u1"),
                    u2_symbol.to_string(): symbol_json(project_id, u2_symbol, u2_part, "U2", "UNFITTED", b"u2")
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

    std::fs::write(
        root.join("board/board.json"),
        format!(
            "{}\n",
            to_json_deterministic(&serde_json::json!({
                "schema_version": 1,
                "uuid": Uuid::new_v4(),
                "name": "Variant Output Job Board",
                "stackup": {
                    "layers": [
                        {"id": 1, "name": "Top Copper", "layer_type": "Copper", "thickness_nm": 35000},
                        {"id": 2, "name": "Top Mask", "layer_type": "SolderMask", "thickness_nm": 10000},
                        {"id": 3, "name": "Top Silk", "layer_type": "Silkscreen", "thickness_nm": 10000},
                        {"id": 4, "name": "Top Paste", "layer_type": "Paste", "thickness_nm": 10000},
                        {"id": 41, "name": "Mechanical 41", "layer_type": "Mechanical", "thickness_nm": 0}
                    ]
                },
                "outline": {
                    "vertices": [
                        { "x": 0, "y": 0 },
                        { "x": 5000000, "y": 0 },
                        { "x": 5000000, "y": 3000000 },
                        { "x": 0, "y": 3000000 }
                    ],
                    "closed": true
                },
                "packages": {
                    u1_package.to_string(): package_json(u1_package, u1_part, "U1", "FITTED", 1000000, 2000000),
                    u2_package.to_string(): package_json(u2_package, u2_part, "U2", "UNFITTED", 3000000, 2000000)
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
                "pads": {},
                "tracks": {},
                "vias": {},
                "zones": {},
                "nets": {},
                "net_classes": {},
                "keepouts": [],
                "dimensions": [],
                "texts": []
            }))
            .unwrap()
        ),
    )
    .unwrap();

    let component_instance_dir = root.join(".datum/component_instances");
    std::fs::create_dir_all(&component_instance_dir).unwrap();
    for (instance, symbol, package) in [
        (u1_instance, u1_symbol, u1_package),
        (u2_instance, u2_symbol, u2_package),
    ] {
        std::fs::write(
            component_instance_dir.join(format!("{instance}.json")),
            format!(
                "{}\n",
                to_json_deterministic(&serde_json::json!({
                    "schema_version": 1,
                    "component_instance": {
                        "uuid": instance,
                        "object_revision": 0,
                        "placed_symbol_refs": [{ "object_id": symbol, "object_revision": 0 }],
                        "placed_package_refs": [{ "object_id": package, "object_revision": 0 }],
                        "placed_package_roles": {
                            package.to_string(): {
                                "role": "physical_package",
                                "label": if instance == u1_instance { "main" } else { "spare" }
                            }
                        }
                    }
                }))
                .unwrap()
            ),
        )
        .unwrap();
    }

    let before_variant = eda_engine::substrate::ProjectResolver::new(root)
        .resolve()
        .unwrap();
    let variant = Uuid::new_v4();
    let variant_path = root.join(".datum/variants/no-u2.json");
    std::fs::create_dir_all(variant_path.parent().unwrap()).unwrap();
    std::fs::write(
        variant_path,
        format!(
            "{}\n",
            to_json_deterministic(&serde_json::json!({
                "schema_version": 1,
                "variants": [{
                    "id": variant,
                    "name": "No U2",
                    "base_model_revision": before_variant.model_revision,
                    "variant_revision": 0,
                    "fitted": {
                        u2_package.to_string(): "unfitted"
                    },
                    "relationship_overrides": {},
                    "property_overrides": {}
                }]
            }))
            .unwrap()
        ),
    )
    .unwrap();
    (variant, u1_instance, u2_instance)
}

fn symbol_json(
    project_id: Uuid,
    uuid: Uuid,
    part: Uuid,
    reference: &str,
    value: &str,
    suffix: &[u8],
) -> serde_json::Value {
    serde_json::json!({
        "uuid": uuid,
        "part": part,
        "entity": Uuid::new_v5(&project_id, &[b"entity-", suffix].concat()),
        "gate": Uuid::new_v5(&project_id, &[b"gate-", suffix].concat()),
        "lib_id": "test:U",
        "reference": reference,
        "value": value,
        "fields": [],
        "pins": [],
        "position": { "x": 0, "y": 0 },
        "rotation": 0,
        "mirrored": false,
        "unit_selection": null,
        "display_mode": "LibraryDefault",
        "pin_overrides": [],
        "hidden_power_behavior": "SourceDefinedImplicit"
    })
}

fn package_json(
    uuid: Uuid,
    part: Uuid,
    reference: &str,
    value: &str,
    x: i64,
    y: i64,
) -> serde_json::Value {
    serde_json::json!({
        "uuid": uuid,
        "reference": reference,
        "value": value,
        "package": Uuid::new_v4(),
        "part": part,
        "position": { "x": x, "y": y },
        "rotation": 0,
        "layer": 1,
        "locked": false
    })
}

#[test]
fn project_run_output_job_uses_stored_variant_for_manufacturing_rows() {
    let root = unique_project_root("datum-eda-cli-project-run-output-job-variant");
    create_native_project(&root, Some("Variant Output Job Demo".to_string())).unwrap();
    let (variant, u1_instance, u2_instance) = write_variant_manufacturing_fixture(&root);
    let output_dir = root.join("variant-job-out");

    let create_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "create-output-job",
            root.to_str().unwrap(),
            "--prefix",
            "Variant Job",
            "--include",
            "manufacturing-set",
            "--output-dir",
            output_dir.to_str().unwrap(),
            "--variant",
            &variant.to_string(),
        ])
        .unwrap(),
    )
    .unwrap();
    let create_report: serde_json::Value = serde_json::from_str(&create_output).unwrap();
    let output_job = create_report["output_job"]["id"].as_str().unwrap();
    assert_eq!(create_report["output_job"]["variant"], variant.to_string());

    let run_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "run-output-job",
            root.to_str().unwrap(),
            "--output-job",
            output_job,
        ])
        .unwrap(),
    )
    .unwrap();
    let run_report: serde_json::Value = serde_json::from_str(&run_output).unwrap();
    assert_eq!(run_report["status"], "succeeded");
    assert_eq!(
        run_report["artifact_report"]["generated"][0]["report"]["artifact_metadata"]["variant"],
        variant.to_string()
    );
    let bom = std::fs::read_to_string(output_dir.join("variant-job-bom.csv")).unwrap();
    let pnp = std::fs::read_to_string(output_dir.join("variant-job-pnp.csv")).unwrap();
    assert!(bom.contains(&format!("{u1_instance},physical_package,main,U1,")));
    assert!(!bom.contains(&format!("{u2_instance},U2,")));
    assert!(pnp.contains(&format!("{u1_instance},physical_package,main,U1,")));
    assert!(!pnp.contains(&format!("{u2_instance},U2,")));
}
