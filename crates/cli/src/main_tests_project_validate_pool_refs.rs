use super::main_tests_project_validate::*;
use super::*;
use eda_engine::ir::serialization::to_json_deterministic;

#[test]
fn project_validate_checks_native_pool_logical_refs() {
    let root = unique_project_root("datum-eda-cli-project-validate-pool");
    create_native_project(&root, Some("Validate Pool".to_string()))
        .expect("native project scaffold should succeed");
    add_project_pool(&root, "pool");

    let unit_id = Uuid::new_v4();
    let pin_id = Uuid::new_v4();
    let symbol_id = Uuid::new_v4();
    let entity_id = Uuid::new_v4();
    let gate_id = Uuid::new_v4();
    let padstack_id = Uuid::new_v4();
    let package_id = Uuid::new_v4();
    let pad_id = Uuid::new_v4();
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
                pad_id: {
                    "uuid": pad_id,
                    "name": "1",
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
                    "name": "1",
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
            "pad_map": {
                pad_id: {
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
                pin_id: footprint_pad_id
            }
        }),
    );

    let (output, exit_code) = execute_with_exit_code(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "validate",
            root.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("project validate should execute");
    let report: serde_json::Value =
        serde_json::from_str(&output).expect("validation JSON should parse");
    assert_eq!(exit_code, 0);
    assert_eq!(report["valid"], true);

    let missing_unit = Uuid::new_v4();
    let bad_symbol_id = Uuid::new_v4();
    let bad_filename_id = Uuid::new_v4();
    write_pool_json(
        &root,
        "pool/symbols",
        bad_filename_id,
        serde_json::json!({
            "uuid": bad_symbol_id,
            "name": "BAD",
            "unit": missing_unit
        }),
    );
    let missing_pad = Uuid::new_v4();
    let bad_part_id = Uuid::new_v4();
    write_pool_json(
        &root,
        "pool/parts",
        bad_part_id,
        serde_json::json!({
            "schema_version": 1,
            "uuid": bad_part_id,
            "entity": Uuid::new_v4(),
            "package": package_id,
            "pad_map": {
                missing_pad: {
                    "gate": Uuid::new_v4(),
                    "pin": Uuid::new_v4()
                }
            }
        }),
    );
    let bad_pin_pad_map_id = Uuid::new_v4();
    let missing_pin_map_key = Uuid::new_v4();
    let missing_pin_map_pad = Uuid::new_v4();
    write_pool_json(
        &root,
        "pool/pin_pad_maps",
        bad_pin_pad_map_id,
        serde_json::json!({
            "schema_version": 1,
            "uuid": bad_pin_pad_map_id,
            "part": part_id,
            "footprint": footprint_id,
            "mappings": {
                missing_pin_map_key: pad_id,
                pin_id: missing_pin_map_pad
            }
        }),
    );
    let bad_footprint_id = Uuid::new_v4();
    let missing_padstack_id = Uuid::new_v4();
    write_pool_json(
        &root,
        "pool/footprints",
        bad_footprint_id,
        serde_json::json!({
            "schema_version": 1,
            "uuid": bad_footprint_id,
            "name": "BAD-FP",
            "package": Uuid::new_v4(),
            "pads": {
                Uuid::new_v4(): {
                    "uuid": Uuid::new_v4(),
                    "name": "BAD",
                    "position": { "x": 0, "y": 0 },
                    "padstack": missing_padstack_id,
                    "layer": 1
                }
            },
            "courtyard": { "points": [] }
        }),
    );

    let (output, exit_code) = execute_with_exit_code(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "validate",
            root.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("project validate should execute");
    let report: serde_json::Value =
        serde_json::from_str(&output).expect("validation JSON should parse");
    let issue_codes = report["issues"]
        .as_array()
        .expect("issues should be an array")
        .iter()
        .map(|issue| issue["code"].as_str().unwrap().to_string())
        .collect::<Vec<_>>();
    assert_eq!(exit_code, 1);
    assert_eq!(report["valid"], false);
    assert!(issue_codes.contains(&"missing_schema_version".to_string()));
    assert!(issue_codes.contains(&"uuid_filename_mismatch".to_string()));
    assert!(issue_codes.contains(&"dangling_reference".to_string()));

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_validate_checks_pool_model_blob_integrity() {
    let (root, _part_id, _blob_path) =
        create_project_with_attached_model("datum-eda-cli-project-validate-pool-model-clean");
    let (report, exit_code) = validate_project_json(&root);
    assert_eq!(exit_code, 0);
    assert_eq!(report["valid"], true);
    let _ = std::fs::remove_dir_all(&root);

    let (root, _part_id, blob_path) =
        create_project_with_attached_model("datum-eda-cli-project-validate-pool-model-tamper");
    std::fs::write(&blob_path, b".subckt TAMPERED IN OUT\n.ends\n")
        .expect("model blob should be tampered");
    let (report, exit_code) = validate_project_json(&root);
    assert_eq!(exit_code, 1);
    assert_eq!(report["valid"], false);
    assert_issue_code(&report, "model_blob_hash_mismatch");
    let _ = std::fs::remove_dir_all(&root);

    let (root, _part_id, blob_path) =
        create_project_with_attached_model("datum-eda-cli-project-validate-pool-model-missing");
    std::fs::remove_file(&blob_path).expect("model blob should be removed");
    let (report, exit_code) = validate_project_json(&root);
    assert_eq!(exit_code, 1);
    assert_eq!(report["valid"], false);
    assert_issue_code(&report, "missing_model_blob");
    let _ = std::fs::remove_dir_all(&root);

    let (root, part_id, _blob_path) =
        create_project_with_attached_model("datum-eda-cli-project-validate-pool-model-bad-uuid");
    let part_path = root.join(format!("pool/parts/{part_id}.json"));
    let mut part: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&part_path).expect("part should read"))
            .expect("part should parse");
    part["behavioural_models"][0]["model_uuid"] =
        serde_json::Value::String(Uuid::new_v4().to_string());
    std::fs::write(
        &part_path,
        format!(
            "{}\n",
            to_json_deterministic(&part).expect("part should serialize")
        ),
    )
    .expect("part should write");
    let (report, exit_code) = validate_project_json(&root);
    assert_eq!(exit_code, 1);
    assert_eq!(report["valid"], false);
    assert_issue_code(&report, "bad_deterministic_model_uuid");
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

fn assert_issue_code(report: &serde_json::Value, code: &str) {
    let issue_codes = report["issues"]
        .as_array()
        .expect("issues should be an array")
        .iter()
        .map(|issue| issue["code"].as_str().unwrap().to_string())
        .collect::<Vec<_>>();
    assert!(
        issue_codes.contains(&code.to_string()),
        "expected issue code {code}, got {issue_codes:?}"
    );
}
