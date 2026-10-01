use super::*;

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
                pin["library_pin"] == pin_id.to_string()
                    && pin["electrical_type"] == electrical_type
            }),
            "placed symbol should preserve {electrical_type} for pin {pin_id}"
        );
    }

    let placed_pin_ids: Vec<String> = fixture
        .pin_ids
        .iter()
        .map(|id| {
            pins.iter()
                .find(|pin| pin["library_pin"] == id.to_string())
                .unwrap()["uuid"]
                .as_str()
                .unwrap()
                .to_string()
        })
        .collect();
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
                    && finding["object_uuids"] == serde_json::json!([placed_pin_ids[0]])
            })
    );
    assert!(
        !check_report["erc"]
            .as_array()
            .unwrap()
            .iter()
            .any(|finding| {
                finding["code"] == "unconnected_component_pin"
                    && finding["object_uuids"] == serde_json::json!([placed_pin_ids[3]])
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
    for pin_id in &placed_pin_ids[..3] {
        assert!(erc_findings.iter().any(|finding| {
            finding["code"] == "unconnected_component_pin"
                && finding["object_uuids"] == serde_json::json!([pin_id.to_string()])
        }));
    }
    assert!(!erc_findings.iter().any(|finding| {
        finding["code"] == "unconnected_component_pin"
            && finding["object_uuids"] == serde_json::json!([placed_pin_ids[3]])
    }));

    let _ = std::fs::remove_dir_all(&root);
}
