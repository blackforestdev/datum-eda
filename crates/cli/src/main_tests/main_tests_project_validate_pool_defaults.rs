use super::main_tests_project_validate::*;
use super::*;
use eda_engine::ir::serialization::to_json_deterministic;

#[test]
fn project_validate_prefers_default_pin_pad_map_over_legacy_part_pad_map() {
    let root = unique_project_root("datum-eda-cli-project-validate-pool-default-pin-pad-map");
    create_native_project(&root, Some("Validate Pool Default Map".to_string()))
        .expect("native project scaffold should succeed");
    add_project_pool(&root, "pool");

    let unit_id = Uuid::new_v4();
    let pin_id = Uuid::new_v4();
    let symbol_id = Uuid::new_v4();
    let entity_id = Uuid::new_v4();
    let gate_id = Uuid::new_v4();
    let padstack_id = Uuid::new_v4();
    let package_id = Uuid::new_v4();
    let package_pad_id = Uuid::new_v4();
    let footprint_id = Uuid::new_v4();
    let footprint_pad_id = Uuid::new_v4();
    let part_id = Uuid::new_v4();
    let pin_pad_map_id = Uuid::new_v4();

    write_pool_json(
        &root,
        "pool/units",
        unit_id,
        serde_json::json!({
            "schema_version": 1,
            "uuid": unit_id,
            "name": "U",
            "manufacturer": "Datum",
            "pins": {
                pin_id: {
                    "uuid": pin_id,
                    "name": "A",
                    "direction": "Passive",
                    "swap_group": 0,
                    "alternates": []
                }
            },
            "tags": []
        }),
    );
    write_pool_json(
        &root,
        "pool/symbols",
        symbol_id,
        serde_json::json!({
            "schema_version": 1,
            "uuid": symbol_id,
            "name": "SYM",
            "unit": unit_id
        }),
    );
    write_pool_json(
        &root,
        "pool/entities",
        entity_id,
        serde_json::json!({
            "schema_version": 1,
            "uuid": entity_id,
            "name": "E",
            "prefix": "U",
            "manufacturer": "Datum",
            "gates": {
                gate_id: {
                    "uuid": gate_id,
                    "name": "A",
                    "unit": unit_id,
                    "symbol": symbol_id
                }
            },
            "tags": []
        }),
    );
    write_pool_json(
        &root,
        "pool/padstacks",
        padstack_id,
        serde_json::json!({
            "schema_version": 1,
            "uuid": padstack_id,
            "name": "P",
            "aperture": { "circle": { "diameter_nm": 500000 } },
            "drill_nm": null
        }),
    );
    write_pool_json(
        &root,
        "pool/packages",
        package_id,
        serde_json::json!({
            "schema_version": 1,
            "uuid": package_id,
            "name": "PKG",
            "pads": {
                package_pad_id: {
                    "uuid": package_pad_id,
                    "name": "LEGACY",
                    "position": { "x": 0, "y": 0 },
                    "padstack": padstack_id,
                    "layer": 1
                }
            },
            "courtyard": { "points": [] },
            "silkscreen": [],
            "models_3d": [],
            "tags": []
        }),
    );
    write_pool_json(
        &root,
        "pool/footprints",
        footprint_id,
        serde_json::json!({
            "schema_version": 1,
            "uuid": footprint_id,
            "name": "FP",
            "package": package_id,
            "pads": {
                footprint_pad_id: {
                    "uuid": footprint_pad_id,
                    "name": "FIRSTCLASS",
                    "position": { "x": 0, "y": 0 },
                    "padstack": padstack_id,
                    "layer": 1
                }
            },
            "courtyard": { "points": [] },
            "silkscreen": [],
            "fab": [],
            "assembly": [],
            "mechanical": [],
            "models_3d": [],
            "tags": []
        }),
    );
    write_pool_json(
        &root,
        "pool/parts",
        part_id,
        serde_json::json!({
            "schema_version": 1,
            "uuid": part_id,
            "entity": entity_id,
            "package": package_id,
            "default_footprint": footprint_id,
            "default_pin_pad_map": pin_pad_map_id,
            "pad_map": {
                package_pad_id: {
                    "gate": gate_id,
                    "pin": pin_id
                }
            },
            "mpn": "D-1",
            "manufacturer": "Datum",
            "value": "1k",
            "description": "",
            "datasheet": "",
            "parametric": {},
            "orderable_mpns": [],
            "tags": [],
            "lifecycle": "Active",
            "base": null
        }),
    );
    write_pool_json(
        &root,
        "pool/pin_pad_maps",
        pin_pad_map_id,
        serde_json::json!({
            "schema_version": 1,
            "uuid": pin_pad_map_id,
            "part": part_id,
            "footprint": footprint_id,
            "mappings": {
                footprint_pad_id: {
                    "gate": gate_id,
                    "pin": pin_id
                }
            }
        }),
    );

    let (report, exit_code) = validate_project_json(&root);
    assert_eq!(exit_code, 0);
    assert_eq!(report["valid"], true);
    let _ = std::fs::remove_dir_all(&root);
}

fn add_project_pool(root: &Path, pool_path: &str) {
    let project_json = root.join("project.json");
    let mut project: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&project_json).expect("project.json should read"),
    )
    .expect("project.json should parse");
    project["pools"] = serde_json::json!([{ "path": pool_path, "priority": 1 }]);
    std::fs::write(
        &project_json,
        format!(
            "{}\n",
            to_json_deterministic(&project).expect("project serialization should succeed")
        ),
    )
    .expect("project.json should write");
}

fn write_pool_json(root: &Path, directory: &str, object_id: Uuid, value: serde_json::Value) {
    let dir = root.join(directory);
    std::fs::create_dir_all(&dir).expect("pool directory should create");
    std::fs::write(
        dir.join(format!("{object_id}.json")),
        format!(
            "{}\n",
            to_json_deterministic(&value).expect("pool object should serialize")
        ),
    )
    .expect("pool object should write");
}
