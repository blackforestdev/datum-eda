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

#[test]
fn project_place_symbol_materializes_pins_from_pool_symbol_uuid_lib_id() {
    let root = unique_project_root("datum-eda-cli-project-symbol-library-materialization");
    create_native_project(&root, Some("Symbol Library Materialization".to_string()))
        .expect("initial scaffold should succeed");
    let sheet_uuid = seed_native_sheet(&root);
    let unit_id = Uuid::new_v4();
    let output_pin_id = Uuid::new_v4();
    let power_pin_id = Uuid::new_v4();
    let open_collector_pin_id = Uuid::new_v4();
    let symbol_id = Uuid::new_v4();
    let entity_id = Uuid::new_v4();
    let gate_id = Uuid::new_v4();
    let padstack_id = Uuid::new_v4();
    let package_id = Uuid::new_v4();
    let package_pad_id = Uuid::new_v4();
    let part_id = Uuid::new_v4();
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
            "OpAmpUnit",
        ])
        .expect("CLI should parse"),
    )
    .expect("typed pool unit create should succeed");
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
            &output_pin_id.to_string(),
            "--name",
            "OUT",
            "--electrical-type",
            "Output",
        ])
        .expect("CLI should parse"),
    )
    .expect("output pool unit pin set should succeed");
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
            &power_pin_id.to_string(),
            "--name",
            "VCC",
            "--electrical-type",
            "PowerIn",
        ])
        .expect("CLI should parse"),
    )
    .expect("power pool unit pin set should succeed");
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
            &open_collector_pin_id.to_string(),
            "--name",
            "FAULT",
            "--electrical-type",
            "OpenCollector",
        ])
        .expect("CLI should parse"),
    )
    .expect("open-collector pool unit pin set should succeed");
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
            "OpAmpSymbol",
        ])
        .expect("CLI should parse"),
    )
    .expect("typed pool symbol create should succeed");
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
            &output_pin_id.to_string(),
            "--x-nm",
            "100",
            "--y-nm",
            "200",
            "--orientation",
            "Left",
            "--length-nm",
            "2540000",
            "--decoration",
            "inverted",
        ])
        .expect("CLI should parse"),
    )
    .expect("output symbol pin anchor set should succeed");
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
            &power_pin_id.to_string(),
            "--x-nm",
            "300",
            "--y-nm",
            "400",
        ])
        .expect("CLI should parse"),
    )
    .expect("power symbol pin anchor set should succeed");
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
            &open_collector_pin_id.to_string(),
            "--x-nm",
            "500",
            "--y-nm",
            "600",
        ])
        .expect("CLI should parse"),
    )
    .expect("open-collector symbol pin anchor set should succeed");
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
            "OpAmp",
            "--prefix",
            "U",
        ])
        .expect("CLI should parse"),
    )
    .expect("typed pool entity create should succeed");
    execute(
        Cli::try_parse_from([
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
        .expect("CLI should parse"),
    )
    .expect("typed pool padstack create should succeed");
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
            "SOT23",
            "--pad",
            &package_pad_id.to_string(),
            "--padstack",
            &padstack_id.to_string(),
        ])
        .expect("CLI should parse"),
    )
    .expect("typed pool package create should succeed");
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
            "OPA1656ID",
            "--manufacturer",
            "Texas Instruments",
            "--value",
            "OPA1656",
        ])
        .expect("CLI should parse"),
    )
    .expect("typed pool part create should succeed");
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
            "AMP",
            "--lib-id",
            &symbol_id.to_string(),
            "--x-nm",
            "1000",
            "--y-nm",
            "2000",
        ])
        .expect("CLI should parse"),
    )
    .expect("place-symbol should succeed");
    let placed: serde_json::Value =
        serde_json::from_str(&place_output).expect("place-symbol JSON should parse");
    let placed_symbol =
        Uuid::parse_str(placed["symbol_uuid"].as_str().unwrap()).expect("symbol uuid should parse");
    assert_eq!(placed["binding_status"], "bound_with_part");
    assert_eq!(placed["entity_uuid"], entity_id.to_string());
    assert_eq!(placed["gate_uuid"], gate_id.to_string());
    assert_eq!(placed["part_uuid"], part_id.to_string());
    assert!(placed["component_instance_uuid"].as_str().is_some());
    assert_eq!(placed["binding_diagnostics"], serde_json::json!([]));
    assert_eq!(
        placed["binding_evidence"]["pool_symbol_ref"],
        serde_json::json!({"object_id": symbol_id.to_string(), "object_revision": 3})
    );
    assert_eq!(
        placed["binding_evidence"]["pool_unit_ref"],
        serde_json::json!({"object_id": unit_id.to_string(), "object_revision": 3})
    );
    assert_eq!(
        placed["binding_evidence"]["entity_ref"],
        serde_json::json!({"object_id": entity_id.to_string(), "object_revision": 0})
    );
    assert_eq!(
        placed["binding_evidence"]["part_ref"],
        serde_json::json!({"object_id": part_id.to_string(), "object_revision": 0})
    );
    assert_eq!(
        placed["binding_evidence"]["placed_symbol_ref"],
        serde_json::json!({"object_id": placed_symbol.to_string(), "object_revision": 0})
    );
    assert_eq!(
        placed["binding_evidence"]["component_instance_ref"]["object_id"],
        placed["component_instance_uuid"]
    );
    assert_eq!(
        placed["binding_evidence"]["component_instance_ref"]["object_revision"],
        0
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
    assert_eq!(
        component_instances["contract"],
        "component_instances_query_v1"
    );
    assert_eq!(component_instances["component_instance_count"], 1);
    let instances = component_instances["component_instances"]
        .as_object()
        .expect("component_instances should be object");
    let instance = instances
        .values()
        .next()
        .expect("one component instance should exist");
    assert_eq!(instance["authority"], "authored");
    assert_eq!(instance["part_ref"], part_id.to_string());
    assert_eq!(
        instance["placed_symbol_refs"],
        serde_json::json!([placed_symbol.to_string()])
    );
    assert_eq!(
        instance["placed_package_refs"]
            .as_array()
            .expect("package refs should be array")
            .len(),
        0
    );

    let pins_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "query",
            root.to_str().unwrap(),
            "symbol-pins",
            "--symbol",
            &placed_symbol.to_string(),
        ])
        .expect("CLI should parse"),
    )
    .expect("symbol-pins query should succeed");
    let pins: serde_json::Value =
        serde_json::from_str(&pins_output).expect("symbol pins JSON should parse");
    let pin_entries = pins.as_array().expect("pins should be array");
    assert_eq!(pin_entries.len(), 3);
    let output_pin_uuid = output_pin_id.to_string();
    let output_pin = pin_entries
        .iter()
        .find(|pin| pin["pin_uuid"].as_str() == Some(output_pin_uuid.as_str()))
        .expect("output pin should be present");
    assert_eq!(output_pin["number"], "OUT");
    assert_eq!(output_pin["electrical_type"], "Output");
    assert_eq!(output_pin["x_nm"], 100);
    assert_eq!(output_pin["y_nm"], 200);
    assert_eq!(output_pin["anchor_orientation"], "Left");
    assert_eq!(output_pin["anchor_length_nm"], 2540000);
    assert_eq!(output_pin["anchor_decoration"], "inverted");
    let power_pin_uuid = power_pin_id.to_string();
    let power_pin = pin_entries
        .iter()
        .find(|pin| pin["pin_uuid"].as_str() == Some(power_pin_uuid.as_str()))
        .expect("power pin should be present");
    assert_eq!(power_pin["number"], "VCC");
    assert_eq!(power_pin["electrical_type"], "PowerIn");
    assert_eq!(power_pin["x_nm"], 300);
    assert_eq!(power_pin["y_nm"], 400);
    assert_eq!(power_pin["anchor_orientation"], "Right");
    assert!(power_pin["anchor_length_nm"].is_null());
    assert_eq!(power_pin["anchor_decoration"], "none");
    let open_collector_pin_uuid = open_collector_pin_id.to_string();
    let open_collector_pin = pin_entries
        .iter()
        .find(|pin| pin["pin_uuid"].as_str() == Some(open_collector_pin_uuid.as_str()))
        .expect("open-collector pin should be present");
    assert_eq!(open_collector_pin["number"], "FAULT");
    assert_eq!(open_collector_pin["electrical_type"], "OpenCollector");

    let nets_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "query",
            root.to_str().unwrap(),
            "nets",
        ])
        .expect("CLI should parse"),
    )
    .expect("nets query should succeed");
    let nets: serde_json::Value =
        serde_json::from_str(&nets_output).expect("nets JSON should parse");
    assert!(nets.as_array().unwrap().iter().any(|net| {
        net["pins"].as_array().unwrap().iter().any(|pin| {
            pin["uuid"] == power_pin_id.to_string()
                && pin["component"] == "U1"
                && pin["pin"] == "VCC"
                && pin["electrical_type"] == "PowerIn"
        })
    }));
    assert!(nets.as_array().unwrap().iter().any(|net| {
        net["pins"].as_array().unwrap().iter().any(|pin| {
            pin["uuid"] == open_collector_pin_id.to_string()
                && pin["component"] == "U1"
                && pin["pin"] == "FAULT"
                && pin["electrical_type"] == "OpenCollector"
        })
    }));
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
    .expect("erc query should succeed");
    let erc: serde_json::Value = serde_json::from_str(&erc_output).expect("erc JSON should parse");
    assert!(
        erc["raw_report"]["erc"]
            .as_array()
            .unwrap()
            .iter()
            .any(|entry| {
                entry["code"] == "power_in_without_source"
                    && entry["object_uuids"] == serde_json::json!([power_pin_id.to_string()])
            })
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_place_symbol_reports_ambiguous_pool_symbol_entity_gate_binding() {
    let root = unique_project_root("datum-eda-cli-project-symbol-ambiguous-binding");
    create_native_project(&root, Some("Symbol Ambiguous Binding".to_string()))
        .expect("initial scaffold should succeed");
    let sheet_uuid = seed_native_sheet(&root);
    let unit_id = Uuid::new_v4();
    let pin_id = Uuid::new_v4();
    let symbol_id = Uuid::new_v4();
    let entity_a_id = Uuid::new_v4();
    let gate_a_id = Uuid::new_v4();
    let entity_b_id = Uuid::new_v4();
    let gate_b_id = Uuid::new_v4();

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
            "AmbiguousUnit",
        ])
        .expect("CLI should parse"),
    )
    .expect("typed pool unit create should succeed");
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
            "AmbiguousSymbol",
        ])
        .expect("CLI should parse"),
    )
    .expect("typed pool symbol create should succeed");
    for (entity_id, gate_id, name) in [
        (entity_a_id, gate_a_id, "EntityA"),
        (entity_b_id, gate_b_id, "EntityB"),
    ] {
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
                name,
                "--prefix",
                "U",
            ])
            .expect("CLI should parse"),
        )
        .expect("typed pool entity create should succeed");
    }

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
            "U2",
            "--value",
            "AMB",
            "--lib-id",
            &symbol_id.to_string(),
            "--x-nm",
            "10",
            "--y-nm",
            "20",
        ])
        .expect("CLI should parse"),
    )
    .expect("ambiguous place-symbol should still place unresolved symbol");
    let placed: serde_json::Value =
        serde_json::from_str(&place_output).expect("place-symbol JSON should parse");
    assert_eq!(placed["binding_status"], "ambiguous_entity_gate");
    assert!(placed["entity_uuid"].is_null());
    assert!(placed["gate_uuid"].is_null());
    assert!(placed["part_uuid"].is_null());
    assert!(placed["component_instance_uuid"].is_null());
    assert!(placed["binding_evidence"].is_null());
    let diagnostic = placed["binding_diagnostics"][0]
        .as_str()
        .expect("diagnostic should be a string");
    assert!(diagnostic.contains("resolves to multiple entity gates"));
    assert!(diagnostic.contains(&entity_a_id.to_string()));
    assert!(diagnostic.contains(&gate_a_id.to_string()));
    assert!(diagnostic.contains(&entity_b_id.to_string()));
    assert!(diagnostic.contains(&gate_b_id.to_string()));

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
