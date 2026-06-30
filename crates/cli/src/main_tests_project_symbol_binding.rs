use super::*;
use eda_engine::ir::serialization::to_json_deterministic;

fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}", label, Uuid::new_v4()))
}

fn seed_native_sheet(root: &Path) -> Uuid {
    let sheet_uuid = Uuid::new_v4();
    let sheet_path = root
        .join("schematic/sheets")
        .join(format!("{sheet_uuid}.json"));
    std::fs::write(&sheet_path, format!("{}\n", to_json_deterministic(&serde_json::json!({"schema_version": 1, "uuid": sheet_uuid, "name": "Main", "frame": null, "symbols": {}, "wires": {}, "junctions": {}, "labels": {}, "buses": {}, "bus_entries": {}, "ports": {}, "noconnects": {}, "texts": {}, "drawings": {}})).expect("sheet JSON should serialize"))).expect("sheet file should write");
    let schematic_json = root.join("schematic/schematic.json");
    let mut schematic_value: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&schematic_json).expect("schematic.json should read"),
    )
    .expect("schematic.json should parse");
    schematic_value["sheets"] =
        serde_json::json!({ sheet_uuid.to_string(): format!("sheets/{sheet_uuid}.json") });
    std::fs::write(
        &schematic_json,
        format!(
            "{}\n",
            to_json_deterministic(&schematic_value)
                .expect("canonical serialization should succeed")
        ),
    )
    .expect("schematic.json should write");
    sheet_uuid
}

fn create_minimal_pool_symbol(root: &Path) -> (Uuid, Uuid, Uuid, Uuid) {
    let unit_id = Uuid::new_v4();
    let pin_id = Uuid::new_v4();
    let symbol_id = Uuid::new_v4();
    let entity_id = Uuid::new_v4();
    let gate_id = Uuid::new_v4();
    execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "create-pool-unit",
            root.to_str().unwrap(),
            "--unit",
            &unit_id.to_string(),
            "--name",
            "Unit",
        ])
        .expect("CLI should parse"),
    )
    .expect("pool unit create should succeed");
    execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "set-pool-unit-pin",
            root.to_str().unwrap(),
            "--unit",
            &unit_id.to_string(),
            "--pin",
            &pin_id.to_string(),
            "--name",
            "A",
            "--electrical-type",
            "Passive",
        ])
        .expect("CLI should parse"),
    )
    .expect("pool unit pin set should succeed");
    execute(
        Cli::try_parse_from([
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
            "Symbol",
        ])
        .expect("CLI should parse"),
    )
    .expect("pool symbol create should succeed");
    execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "set-pool-symbol-pin-anchor",
            root.to_str().unwrap(),
            "--symbol",
            &symbol_id.to_string(),
            "--pin",
            &pin_id.to_string(),
            "--x-nm",
            "0",
            "--y-nm",
            "0",
        ])
        .expect("CLI should parse"),
    )
    .expect("pool symbol pin anchor set should succeed");
    execute(
        Cli::try_parse_from([
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
            "Entity",
            "--prefix",
            "U",
        ])
        .expect("CLI should parse"),
    )
    .expect("pool entity create should succeed");
    (unit_id, symbol_id, entity_id, gate_id)
}

fn create_minimal_unbound_pool_symbol(root: &Path) -> Uuid {
    let unit_id = Uuid::new_v4();
    let pin_id = Uuid::new_v4();
    let symbol_id = Uuid::new_v4();
    execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "create-pool-unit",
            root.to_str().unwrap(),
            "--unit",
            &unit_id.to_string(),
            "--name",
            "UnboundUnit",
        ])
        .expect("CLI should parse"),
    )
    .expect("pool unit create should succeed");
    execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "set-pool-unit-pin",
            root.to_str().unwrap(),
            "--unit",
            &unit_id.to_string(),
            "--pin",
            &pin_id.to_string(),
            "--name",
            "A",
            "--electrical-type",
            "Passive",
        ])
        .expect("CLI should parse"),
    )
    .expect("pool unit pin set should succeed");
    execute(
        Cli::try_parse_from([
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
            "UnboundSymbol",
        ])
        .expect("CLI should parse"),
    )
    .expect("pool symbol create should succeed");
    symbol_id
}

fn create_minimal_pool_package(root: &Path) -> Uuid {
    let package_id = Uuid::new_v4();
    execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "create-pool-package",
            root.to_str().unwrap(),
            "--package",
            &package_id.to_string(),
            "--name",
            "PKG",
        ])
        .expect("CLI should parse"),
    )
    .expect("pool package create should succeed");
    package_id
}

fn create_minimal_pool_part(root: &Path, entity_id: Uuid, package_id: Uuid) -> Uuid {
    let part_id = Uuid::new_v4();
    execute(
        Cli::try_parse_from([
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
            "MPN",
            "--manufacturer",
            "MFR",
            "--value",
            "VALUE",
        ])
        .expect("CLI should parse"),
    )
    .expect("pool part create should succeed");
    part_id
}

fn set_part_legacy_pad_map(root: &Path, part_id: Uuid, pad_id: Uuid, gate_id: Uuid, pin_id: Uuid) {
    let part_path = root.join("pool/parts").join(format!("{part_id}.json"));
    let mut part: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&part_path).expect("part object should read"),
    )
    .expect("part object should parse");
    part["pad_map"] = serde_json::json!({
        pad_id.to_string(): {
            "gate": gate_id,
            "pin": pin_id
        }
    });
    let source_path = root.join(format!("{part_id}-replacement.json"));
    std::fs::write(
        &source_path,
        format!(
            "{}\n",
            to_json_deterministic(&part).expect("part JSON should serialize")
        ),
    )
    .expect("replacement part should write");
    execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "set-pool-library-object",
            root.to_str().unwrap(),
            "--kind",
            "parts",
            "--object",
            &part_id.to_string(),
            "--from-json",
            source_path.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("pool part replacement should succeed");
    let _ = std::fs::remove_file(source_path);
}

fn place_minimal_pool_symbol(root: &Path, sheet_uuid: Uuid, symbol_id: Uuid) -> serde_json::Value {
    let place_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "place-symbol",
            root.to_str().unwrap(),
            "--sheet",
            &sheet_uuid.to_string(),
            "--reference",
            "U1",
            "--value",
            "VALUE",
            "--lib-id",
            &symbol_id.to_string(),
            "--x-nm",
            "10",
            "--y-nm",
            "20",
        ])
        .expect("CLI should parse"),
    )
    .expect("place-symbol should succeed");
    serde_json::from_str(&place_output).expect("place-symbol JSON should parse")
}

#[test]
fn project_place_symbol_reports_bound_entity_gate_without_part() {
    let root = unique_project_root("datum-eda-cli-project-symbol-no-part-binding");
    create_native_project(&root, Some("Symbol No Part Binding".to_string()))
        .expect("initial scaffold should succeed");
    let sheet_uuid = seed_native_sheet(&root);
    let (_unit_id, symbol_id, entity_id, gate_id) = create_minimal_pool_symbol(&root);

    let placed = place_minimal_pool_symbol(&root, sheet_uuid, symbol_id);
    assert_eq!(placed["binding_status"], "bound_without_part");
    assert_eq!(placed["entity_uuid"], entity_id.to_string());
    assert_eq!(placed["gate_uuid"], gate_id.to_string());
    assert!(placed["part_uuid"].is_null());
    assert!(placed["component_instance_uuid"].is_null());
    assert_eq!(
        placed["binding_evidence"]["entity_ref"]["object_id"],
        entity_id.to_string()
    );
    assert_eq!(placed["binding_evidence"]["gate_uuid"], gate_id.to_string());
    assert!(placed["binding_evidence"]["part_ref"].is_null());
    assert!(placed["binding_evidence"]["component_instance_ref"].is_null());
    assert!(
        placed["binding_diagnostics"][0]
            .as_str()
            .expect("diagnostic should be a string")
            .contains("has no pool part")
    );

    let component_instances_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "query",
            root.to_str().unwrap(),
            "component-instances",
        ])
        .expect("CLI should parse"),
    )
    .expect("component-instances query should succeed");
    let component_instances: serde_json::Value = serde_json::from_str(&component_instances_output)
        .expect("component-instances JSON should parse");
    assert_eq!(component_instances["component_instance_count"], 0);
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_place_symbol_reports_ambiguous_compatible_pool_parts() {
    let root = unique_project_root("datum-eda-cli-project-symbol-ambiguous-part-binding");
    create_native_project(&root, Some("Symbol Ambiguous Part Binding".to_string()))
        .expect("initial scaffold should succeed");
    let sheet_uuid = seed_native_sheet(&root);
    let (_unit_id, symbol_id, entity_id, gate_id) = create_minimal_pool_symbol(&root);
    let package_id = create_minimal_pool_package(&root);
    let part_a = create_minimal_pool_part(&root, entity_id, package_id);
    let part_b = create_minimal_pool_part(&root, entity_id, package_id);

    let placed = place_minimal_pool_symbol(&root, sheet_uuid, symbol_id);
    assert_eq!(placed["binding_status"], "ambiguous_part");
    assert_eq!(placed["entity_uuid"], entity_id.to_string());
    assert_eq!(placed["gate_uuid"], gate_id.to_string());
    assert!(placed["part_uuid"].is_null());
    assert!(placed["component_instance_uuid"].is_null());
    assert_eq!(
        placed["binding_evidence"]["entity_ref"]["object_id"],
        entity_id.to_string()
    );
    assert_eq!(placed["binding_evidence"]["gate_uuid"], gate_id.to_string());
    assert!(placed["binding_evidence"]["part_ref"].is_null());
    assert!(placed["binding_evidence"]["component_instance_ref"].is_null());
    let diagnostic = placed["binding_diagnostics"][0]
        .as_str()
        .expect("diagnostic should be a string");
    assert!(diagnostic.contains("multiple compatible pool parts"));
    assert!(diagnostic.contains(&part_a.to_string()));
    assert!(diagnostic.contains(&part_b.to_string()));

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_place_symbol_binds_single_compatible_part_among_incompatible_parts() {
    let root = unique_project_root("datum-eda-cli-project-symbol-single-compatible-part-binding");
    create_native_project(
        &root,
        Some("Symbol Single Compatible Part Binding".to_string()),
    )
    .expect("initial scaffold should succeed");
    let sheet_uuid = seed_native_sheet(&root);
    let (_unit_id, symbol_id, entity_id, gate_id) = create_minimal_pool_symbol(&root);
    let package_id = create_minimal_pool_package(&root);
    let compatible_part_id = create_minimal_pool_part(&root, entity_id, package_id);
    let incompatible_part_id = create_minimal_pool_part(&root, entity_id, package_id);
    set_part_legacy_pad_map(
        &root,
        incompatible_part_id,
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
    );

    let placed = place_minimal_pool_symbol(&root, sheet_uuid, symbol_id);
    assert_eq!(placed["binding_status"], "bound_with_part");
    assert_eq!(placed["entity_uuid"], entity_id.to_string());
    assert_eq!(placed["gate_uuid"], gate_id.to_string());
    assert_eq!(placed["part_uuid"], compatible_part_id.to_string());
    assert!(placed["component_instance_uuid"].as_str().is_some());
    assert_eq!(
        placed["binding_evidence"]["part_ref"]["object_id"],
        compatible_part_id.to_string()
    );
    assert_eq!(
        placed["binding_evidence"]["component_instance_ref"]["object_id"],
        placed["component_instance_uuid"]
    );

    let component_instances_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "query",
            root.to_str().unwrap(),
            "component-instances",
        ])
        .expect("CLI should parse"),
    )
    .expect("component-instances query should succeed");
    let component_instances: serde_json::Value = serde_json::from_str(&component_instances_output)
        .expect("component-instances JSON should parse");
    assert_eq!(component_instances["component_instance_count"], 1);

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_place_symbol_reports_unresolved_pool_symbol_entity_gate_binding() {
    let root = unique_project_root("datum-eda-cli-project-symbol-unresolved-binding");
    create_native_project(&root, Some("Symbol Unresolved Binding".to_string()))
        .expect("initial scaffold should succeed");
    let sheet_uuid = seed_native_sheet(&root);
    let symbol_id = create_minimal_unbound_pool_symbol(&root);

    let placed = place_minimal_pool_symbol(&root, sheet_uuid, symbol_id);
    assert_eq!(placed["binding_status"], "unresolved_entity_gate");
    assert!(placed["entity_uuid"].is_null());
    assert!(placed["gate_uuid"].is_null());
    assert!(placed["part_uuid"].is_null());
    assert!(placed["component_instance_uuid"].is_null());
    assert!(placed["binding_evidence"].is_null());
    assert!(
        placed["binding_diagnostics"][0]
            .as_str()
            .expect("diagnostic should be a string")
            .contains("did not resolve to any entity gate")
    );

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_place_symbol_reports_incompatible_pool_part_without_component_instance() {
    let root = unique_project_root("datum-eda-cli-project-symbol-incompatible-part-binding");
    create_native_project(&root, Some("Symbol Incompatible Part Binding".to_string()))
        .expect("initial scaffold should succeed");
    let sheet_uuid = seed_native_sheet(&root);
    let (_unit_id, symbol_id, entity_id, gate_id) = create_minimal_pool_symbol(&root);
    let package_id = create_minimal_pool_package(&root);
    let part_id = create_minimal_pool_part(&root, entity_id, package_id);
    set_part_legacy_pad_map(
        &root,
        part_id,
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
    );

    let placed = place_minimal_pool_symbol(&root, sheet_uuid, symbol_id);
    assert_eq!(placed["binding_status"], "incompatible_part");
    assert_eq!(placed["entity_uuid"], entity_id.to_string());
    assert_eq!(placed["gate_uuid"], gate_id.to_string());
    assert!(placed["part_uuid"].is_null());
    assert!(placed["component_instance_uuid"].is_null());
    assert!(placed["binding_evidence"]["part_ref"].is_null());
    assert!(placed["binding_evidence"]["component_instance_ref"].is_null());
    let diagnostic = placed["binding_diagnostics"][0]
        .as_str()
        .expect("diagnostic should be a string");
    assert!(diagnostic.contains("none are compatible"));
    assert!(diagnostic.contains(&gate_id.to_string()));
    assert!(diagnostic.contains(&part_id.to_string()));

    let component_instances_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "query",
            root.to_str().unwrap(),
            "component-instances",
        ])
        .expect("CLI should parse"),
    )
    .expect("component-instances query should succeed");
    let component_instances: serde_json::Value = serde_json::from_str(&component_instances_output)
        .expect("component-instances JSON should parse");
    assert_eq!(component_instances["component_instance_count"], 0);

    let _ = std::fs::remove_dir_all(&root);
}
