use super::*;
use eda_engine::ir::serialization::to_json_deterministic;

fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}", label, Uuid::new_v4()))
}

fn run_project_command(args: &[&str]) -> Result<String> {
    execute(Cli::try_parse_from(args).expect("CLI should parse"))
}

fn validate_project_json(root: &Path) -> (serde_json::Value, i32) {
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
    (
        serde_json::from_str(&output).expect("validation JSON should parse"),
        exit_code,
    )
}

fn create_authored_pool_part(root: &Path) -> Uuid {
    let unit_id = Uuid::new_v4();
    let symbol_id = Uuid::new_v4();
    let entity_id = Uuid::new_v4();
    let gate_id = Uuid::new_v4();
    let padstack_id = Uuid::new_v4();
    let package_id = Uuid::new_v4();
    let pad_id = Uuid::new_v4();
    let part_id = Uuid::new_v4();
    run_project_command(&[
        "eda",
        "--format",
        "json",
        "project",
        "create-pool-unit",
        root.to_str().unwrap(),
        "--unit",
        &unit_id.to_string(),
        "--name",
        "ModelUnit",
    ])
    .expect("unit create should succeed");
    run_project_command(&[
        "eda",
        "--format",
        "json",
        "project",
        "create-pool-symbol",
        root.to_str().unwrap(),
        "--symbol",
        &symbol_id.to_string(),
        "--unit",
        &unit_id.to_string(),
        "--name",
        "ModelSymbol",
    ])
    .expect("symbol create should succeed");
    run_project_command(&[
        "eda",
        "--format",
        "json",
        "project",
        "create-pool-entity",
        root.to_str().unwrap(),
        "--entity",
        &entity_id.to_string(),
        "--gate",
        &gate_id.to_string(),
        "--unit",
        &unit_id.to_string(),
        "--symbol",
        &symbol_id.to_string(),
        "--name",
        "ModelEntity",
        "--prefix",
        "U",
    ])
    .expect("entity create should succeed");
    run_project_command(&[
        "eda",
        "--format",
        "json",
        "project",
        "create-pool-padstack",
        root.to_str().unwrap(),
        "--padstack",
        &padstack_id.to_string(),
        "--name",
        "RoundPad",
        "--aperture",
        "circle",
        "--diameter-nm",
        "1200000",
    ])
    .expect("padstack create should succeed");
    run_project_command(&[
        "eda",
        "--format",
        "json",
        "project",
        "create-pool-package",
        root.to_str().unwrap(),
        "--package",
        &package_id.to_string(),
        "--name",
        "SOT23",
        "--pad",
        &pad_id.to_string(),
        "--padstack",
        &padstack_id.to_string(),
    ])
    .expect("package create should succeed");
    run_project_command(&[
        "eda",
        "--format",
        "json",
        "project",
        "create-pool-part",
        root.to_str().unwrap(),
        "--part",
        &part_id.to_string(),
        "--entity",
        &entity_id.to_string(),
        "--package",
        &package_id.to_string(),
        "--mpn",
        "MODEL-PART",
        "--manufacturer",
        "Datum",
        "--value",
        "MODEL",
    ])
    .expect("part create should succeed");
    part_id
}

fn create_project_with_attached_model(label: &str) -> (PathBuf, Uuid, PathBuf) {
    let root = unique_project_root(label);
    create_native_project(&root, Some("Validate Pool Model".to_string()))
        .expect("native project scaffold should succeed");
    let part_id = create_authored_pool_part(&root);
    let source_dir = root.join("vendor");
    std::fs::create_dir_all(&source_dir).expect("vendor dir should be created");
    let source = source_dir.join("model.lib");
    std::fs::write(&source, b".subckt MODEL IN OUT\n.ends\n")
        .expect("model fixture should be written");
    run_project_command(&[
        "eda",
        "--format",
        "json",
        "project",
        "attach-pool-part-model",
        root.to_str().unwrap(),
        "--part",
        &part_id.to_string(),
        "--source",
        source.to_str().unwrap(),
        "--role",
        "Spice",
        "--dialect",
        "Ngspice",
        "--model-name",
        "MODEL",
    ])
    .expect("model attach should succeed");
    let blob_path = std::fs::read_dir(root.join("pool/models/spice"))
        .expect("model dir should exist")
        .next()
        .expect("model blob should exist")
        .expect("model blob entry should read")
        .path();
    (root, part_id, blob_path)
}

#[test]
fn project_validate_reports_clean_native_project() {
    let root = unique_project_root("datum-eda-cli-project-validate-clean");
    create_native_project(&root, Some("Validate Demo".to_string()))
        .expect("native project scaffold should succeed");

    let cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "validate",
        root.to_str().unwrap(),
    ])
    .expect("CLI should parse");

    let (output, exit_code) = execute_with_exit_code(cli).expect("project validate should succeed");
    let report: serde_json::Value =
        serde_json::from_str(&output).expect("validation JSON should parse");
    assert_eq!(exit_code, 0);
    assert_eq!(report["action"], "validate_project");
    assert_eq!(report["project_root"], root.display().to_string());
    assert_eq!(report["valid"], true);
    assert_eq!(report["schema_compatible"], true);
    assert_eq!(report["required_files_expected"], 4);
    assert_eq!(report["required_files_validated"], 4);
    assert_eq!(report["checked_sheet_files"], 0);
    assert_eq!(report["checked_definition_files"], 0);
    assert_eq!(report["issue_count"], 0);
    assert_eq!(report["issues"], serde_json::json!([]));

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_validate_reports_dangling_duplicate_and_missing_native_references() {
    let root = unique_project_root("datum-eda-cli-project-validate-invalid");
    create_native_project(&root, Some("Validate Invalid".to_string()))
        .expect("native project scaffold should succeed");

    let board_json = root.join("board/board.json");
    let mut board_value: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&board_json).expect("board.json should read"),
    )
    .expect("board.json should parse");

    let package_uuid = Uuid::new_v4();
    let package_part_uuid = Uuid::new_v4();
    let package_pkg_uuid = Uuid::new_v4();
    let missing_package_uuid = Uuid::new_v4();
    let net_uuid = Uuid::new_v4();
    let missing_net_uuid = Uuid::new_v4();
    let missing_net_class_uuid = Uuid::new_v4();
    let pad_key_uuid = Uuid::new_v4();
    let pad_value_uuid = Uuid::new_v4();
    let track_uuid = Uuid::new_v4();
    let second_track_key_uuid = Uuid::new_v4();

    board_value["packages"] = serde_json::json!({
        package_uuid: {
            "uuid": package_uuid,
            "part": package_part_uuid,
            "package": package_pkg_uuid,
            "reference": "U1",
            "value": "MCU",
            "position": { "x": 0, "y": 0 },
            "rotation": 0,
            "layer": 1,
            "locked": false
        }
    });
    board_value["nets"] = serde_json::json!({
        net_uuid: {
            "uuid": net_uuid,
            "name": "SIG",
            "class": missing_net_class_uuid
        }
    });
    board_value["pads"] = serde_json::json!({
        pad_key_uuid: {
            "uuid": pad_value_uuid,
            "package": missing_package_uuid,
            "name": "1",
            "net": missing_net_uuid,
            "position": { "x": 1000, "y": 2000 },
            "layer": 1,
            "shape": "circle",
            "diameter": 500000,
            "width": 0,
            "height": 0
        }
    });
    board_value["tracks"] = serde_json::json!({
        track_uuid: {
            "uuid": track_uuid,
            "net": net_uuid,
            "from": { "x": 0, "y": 0 },
            "to": { "x": 1000, "y": 0 },
            "width": 150000,
            "layer": 1
        },
        second_track_key_uuid: {
            "uuid": track_uuid,
            "net": missing_net_uuid,
            "from": { "x": 0, "y": 1000 },
            "to": { "x": 1000, "y": 1000 },
            "width": 150000,
            "layer": 1
        }
    });
    std::fs::write(
        &board_json,
        format!(
            "{}\n",
            to_json_deterministic(&board_value).expect("board serialization should succeed")
        ),
    )
    .expect("board.json should write");

    let schematic_json = root.join("schematic/schematic.json");
    let mut schematic_value: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&schematic_json).expect("schematic.json should read"),
    )
    .expect("schematic.json should parse");
    let missing_sheet_uuid = Uuid::new_v4();
    schematic_value["sheets"] = serde_json::json!({
        missing_sheet_uuid: "sheets/missing-sheet.json"
    });
    std::fs::write(
        &schematic_json,
        format!(
            "{}\n",
            to_json_deterministic(&schematic_value)
                .expect("schematic serialization should succeed")
        ),
    )
    .expect("schematic.json should write");

    let cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "validate",
        root.to_str().unwrap(),
    ])
    .expect("CLI should parse");
    let (output, exit_code) = execute_with_exit_code(cli).expect("project validate should execute");
    let report: serde_json::Value =
        serde_json::from_str(&output).expect("validation JSON should parse");
    let issues = report["issues"]
        .as_array()
        .expect("issues should be an array");
    let issue_codes = issues
        .iter()
        .map(|issue| issue["code"].as_str().unwrap().to_string())
        .collect::<Vec<_>>();

    assert_eq!(exit_code, 1);
    assert_eq!(report["valid"], false);
    assert!(report["issue_count"].as_u64().unwrap() >= 5);
    assert!(issue_codes.contains(&"missing_file".to_string()));
    assert!(issue_codes.contains(&"dangling_reference".to_string()));
    assert!(issue_codes.contains(&"duplicate_uuid_within_type".to_string()));
    assert!(issue_codes.contains(&"uuid_key_mismatch".to_string()));

    let text_output = execute(
        Cli::try_parse_from(["eda", "project", "validate", root.to_str().unwrap()])
            .expect("CLI should parse"),
    )
    .expect("project validate text should execute");
    assert!(text_output.contains("valid: false"));
    assert!(text_output.contains("issues:"));
    assert!(text_output.contains("missing_file"));
    assert!(text_output.contains("dangling_reference"));
    assert!(text_output.contains("duplicate_uuid_within_type"));

    let _ = std::fs::remove_dir_all(&root);
}

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
            "mappings": {}
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
