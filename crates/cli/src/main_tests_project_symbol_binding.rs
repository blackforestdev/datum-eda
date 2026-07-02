use super::main_tests_project_pool_pin_pad_map::{
    create_default_pin_pad_map, create_fixture_with_pin_types,
};
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

#[test]
fn project_library_pin_pad_map_workflow_places_bound_symbol_and_runs_checks() {
    let root = unique_project_root("datum-eda-cli-project-library-pin-pad-map-e2e");
    create_native_project(&root, Some("Library PinPadMap E2E".to_string()))
        .expect("initial scaffold should succeed");
    let sheet_uuid = seed_native_sheet(&root);
    let fixture = create_fixture_with_pin_types(
        &root,
        &[
            ("OC", "OpenCollector"),
            ("OE", "OpenEmitter"),
            ("TS", "TriState"),
            ("NC", "NoConnect"),
        ],
        &["1", "2", "3", "4"],
    );
    let map_id = create_default_pin_pad_map(
        &root,
        &fixture,
        &[
            (fixture.pin_ids[0], fixture.pad_ids[0]),
            (fixture.pin_ids[1], fixture.pad_ids[1]),
            (fixture.pin_ids[2], fixture.pad_ids[2]),
            (fixture.pin_ids[3], fixture.pad_ids[3]),
        ],
    );

    let placed = place_minimal_pool_symbol(&root, sheet_uuid, fixture.symbol_id);
    assert_eq!(placed["binding_status"], "bound_with_part");
    assert_eq!(placed["part_uuid"], fixture.part_id.to_string());
    assert!(placed["component_instance_uuid"].as_str().is_some());
    assert_eq!(
        placed["binding_evidence"]["part_ref"]["object_id"],
        fixture.part_id.to_string()
    );

    let part_payload = serde_json::from_str::<serde_json::Value>(
        &std::fs::read_to_string(root.join(format!("pool/parts/{}.json", fixture.part_id)))
            .expect("part should read"),
    )
    .expect("part should parse");
    assert_eq!(part_payload["default_pin_pad_map"], map_id.to_string());

    let sheet_payload = serde_json::from_str::<serde_json::Value>(
        &std::fs::read_to_string(root.join(format!("schematic/sheets/{sheet_uuid}.json")))
            .expect("sheet should read"),
    )
    .expect("sheet should parse");
    let placed_symbol_uuid = placed["symbol_uuid"]
        .as_str()
        .expect("placed symbol UUID should be present");
    let placed_symbol = sheet_payload["symbols"][placed_symbol_uuid].clone();
    assert_eq!(placed_symbol["part"], fixture.part_id.to_string());
    let pins = placed_symbol["pins"]
        .as_array()
        .expect("placed symbol pins should be an array");
    for (pin_id, electrical_type) in [
        (fixture.pin_ids[0], "OpenCollector"),
        (fixture.pin_ids[1], "OpenEmitter"),
        (fixture.pin_ids[2], "TriState"),
        (fixture.pin_ids[3], "NoConnect"),
    ] {
        assert!(
            pins.iter().any(|pin| {
                pin["uuid"] == pin_id.to_string() && pin["electrical_type"] == electrical_type
            }),
            "placed symbol should preserve {electrical_type} for pin {pin_id}"
        );
    }

    let (validate_output, validate_exit_code) = execute_with_exit_code(
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
    let validate_report: serde_json::Value =
        serde_json::from_str(&validate_output).expect("validation JSON should parse");
    assert_eq!(validate_exit_code, 0);
    assert_eq!(validate_report["valid"], true);

    let check_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "query",
            root.to_str().unwrap(),
            "check",
        ])
        .expect("CLI should parse"),
    )
    .expect("project query check should succeed");
    let check_report: serde_json::Value =
        serde_json::from_str(&check_output).expect("check JSON should parse");
    assert_eq!(check_report["domain"], "combined");
    assert_eq!(check_report["drc"], serde_json::json!([]));
    assert!(
        check_report["erc"]
            .as_array()
            .unwrap()
            .iter()
            .any(|finding| {
                finding["code"] == "unconnected_component_pin"
                    && finding["object_uuids"]
                        == serde_json::json!([fixture.pin_ids[0].to_string()])
            })
    );
    assert!(
        !check_report["erc"]
            .as_array()
            .unwrap()
            .iter()
            .any(|finding| {
                finding["code"] == "unconnected_component_pin"
                    && finding["object_uuids"]
                        == serde_json::json!([fixture.pin_ids[3].to_string()])
            }),
        "NoConnect pins should not be reported as unconnected component pins"
    );

    let erc_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "query",
            root.to_str().unwrap(),
            "erc",
        ])
        .expect("CLI should parse"),
    )
    .expect("project query erc should succeed");
    let erc_report: serde_json::Value =
        serde_json::from_str(&erc_output).expect("ERC JSON should parse");
    assert_eq!(erc_report["profile_id"], "erc");
    let erc_findings = erc_report["raw_report"]["erc"]
        .as_array()
        .expect("raw ERC findings should be preserved");
    for pin_id in &fixture.pin_ids[..3] {
        assert!(erc_findings.iter().any(|finding| {
            finding["code"] == "unconnected_component_pin"
                && finding["object_uuids"] == serde_json::json!([pin_id.to_string()])
        }));
    }
    assert!(!erc_findings.iter().any(|finding| {
        finding["code"] == "unconnected_component_pin"
            && finding["object_uuids"] == serde_json::json!([fixture.pin_ids[3].to_string()])
    }));

    let _ = std::fs::remove_dir_all(&root);
}
