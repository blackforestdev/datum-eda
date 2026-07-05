use super::*;

pub(super) struct PinnedMapFixture {
    pub(super) symbol_id: Uuid,
    pub(super) part_id: Uuid,
    pub(super) gate_id: Uuid,
    pub(super) footprint_id: Uuid,
    pub(super) pin_ids: Vec<Uuid>,
    pub(super) pad_ids: Vec<Uuid>,
}

pub(super) fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}", label, Uuid::new_v4()))
}

pub(super) fn run_project_command(args: &[&str]) -> Result<String> {
    execute(Cli::try_parse_from(args).expect("CLI should parse"))
}

pub(super) fn query_pool_object_payload(
    root: &Path,
    kind: &str,
    object_id: Uuid,
) -> serde_json::Value {
    let path = root.join(format!("pool/{kind}/{object_id}.json"));
    serde_json::from_str(&std::fs::read_to_string(path).expect("pool object should read"))
        .expect("pool object should parse")
}

pub(super) fn set_part_default_pin_pad_map_raw(root: &Path, part_id: Uuid, map_id: Uuid) {
    let path = root.join(format!("pool/parts/{part_id}.json"));
    let mut part: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&path).expect("part should read"))
            .expect("part should parse");
    part["default_pin_pad_map"] = serde_json::Value::String(map_id.to_string());
    std::fs::write(
        path,
        format!(
            "{}\n",
            serde_json::to_string_pretty(&part).expect("part should serialize")
        ),
    )
    .expect("part should write");
}

pub(super) fn create_fixture(
    root: &Path,
    pin_names: &[&str],
    pad_names: &[&str],
) -> PinnedMapFixture {
    let pin_specs = pin_names
        .iter()
        .map(|name| (*name, "Input"))
        .collect::<Vec<_>>();
    create_fixture_with_pin_types(root, &pin_specs, pad_names)
}

pub(super) fn create_fixture_with_pin_types(
    root: &Path,
    pin_specs: &[(&str, &str)],
    pad_names: &[&str],
) -> PinnedMapFixture {
    let unit_id = Uuid::new_v4();
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
        "PinnedUnit",
    ])
    .expect("unit create should succeed");

    let pin_ids = pin_specs
        .iter()
        .map(|(name, electrical_type)| {
            let pin_id = Uuid::new_v4();
            run_project_command(&[
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
                name,
                "--electrical-type",
                electrical_type,
            ])
            .expect("unit pin set should succeed");
            pin_id
        })
        .collect::<Vec<_>>();

    let symbol_id = Uuid::new_v4();
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
        "PinnedSymbol",
    ])
    .expect("symbol create should succeed");

    for (index, pin_id) in pin_ids.iter().enumerate() {
        run_project_command(&[
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
            &(index as i64 * 2_540_000).to_string(),
            "--y-nm",
            "0",
        ])
        .expect("symbol pin anchor set should succeed");
    }

    let entity_id = Uuid::new_v4();
    let gate_id = Uuid::new_v4();
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
        "PinnedEntity",
        "--prefix",
        "U",
    ])
    .expect("entity create should succeed");

    let padstack_id = Uuid::new_v4();
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
        "P1",
        "--aperture",
        "circle",
        "--diameter-nm",
        "500000",
    ])
    .expect("padstack create should succeed");

    let package_id = Uuid::new_v4();
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
        "PKG",
    ])
    .expect("package create should succeed");

    let footprint_id = Uuid::new_v4();
    run_project_command(&[
        "eda",
        "--format",
        "json",
        "project",
        "create-pool-footprint",
        root.to_str().unwrap(),
        "--footprint",
        &footprint_id.to_string(),
        "--package",
        &package_id.to_string(),
        "--name",
        "PKG_LandPattern",
    ])
    .expect("footprint create should succeed");

    let mut pad_ids = Vec::new();
    for name in pad_names {
        let pad_id = Uuid::new_v4();
        run_project_command(&[
            "eda",
            "--format",
            "json",
            "project",
            "set-pool-footprint-pad",
            root.to_str().unwrap(),
            "--footprint",
            &footprint_id.to_string(),
            "--pad",
            &pad_id.to_string(),
            "--padstack",
            &padstack_id.to_string(),
            "--pad-name",
            name,
        ])
        .expect("footprint pad set should succeed");
        pad_ids.push(pad_id);
    }

    let part_id = Uuid::new_v4();
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
        "PINNED",
    ])
    .expect("part create should succeed");

    PinnedMapFixture {
        symbol_id,
        part_id,
        gate_id,
        footprint_id,
        pin_ids,
        pad_ids,
    }
}

pub(super) fn create_default_pin_pad_map(
    root: &Path,
    fixture: &PinnedMapFixture,
    entries: &[(Uuid, Uuid)],
) -> Uuid {
    let map_id = Uuid::new_v4();
    let mut args = vec![
        "eda".to_string(),
        "--format".to_string(),
        "json".to_string(),
        "project".to_string(),
        "create-pool-pin-pad-map".to_string(),
        root.to_str().unwrap().to_string(),
        "--map".to_string(),
        map_id.to_string(),
        "--part".to_string(),
        fixture.part_id.to_string(),
        "--footprint".to_string(),
        fixture.footprint_id.to_string(),
        "--set-default".to_string(),
    ];
    for (pin_id, pad_id) in entries {
        args.push("--entry".to_string());
        args.push(format!("{pin_id}:{pad_id}"));
    }
    let refs = args.iter().map(String::as_str).collect::<Vec<_>>();
    run_project_command(&refs).expect("default PinPadMap create should succeed");
    map_id
}

#[test]
fn project_create_pool_pin_pad_map_authors_first_class_map_and_default_binding() {
    let root = unique_project_root("datum-eda-cli-project-pool-pin-pad-map");
    create_native_project(&root, Some("Pool PinPadMap".to_string()))
        .expect("initial scaffold should succeed");
    let fixture = create_fixture(&root, &["IN+", "OUT"], &["1", "2"]);
    let map_id = Uuid::new_v4();

    let output = run_project_command(&[
        "eda",
        "--format",
        "json",
        "project",
        "create-pool-pin-pad-map",
        root.to_str().unwrap(),
        "--map",
        &map_id.to_string(),
        "--part",
        &fixture.part_id.to_string(),
        "--footprint",
        &fixture.footprint_id.to_string(),
        "--entry",
        &format!("{}:{}", fixture.pin_ids[0], fixture.pad_ids[0]),
        "--entry",
        &format!("{}:{}", fixture.pin_ids[1], fixture.pad_ids[1]),
        "--set-default",
    ])
    .expect("PinPadMap create should succeed");
    let report: serde_json::Value =
        serde_json::from_str(&output).expect("PinPadMap report JSON should parse");
    assert_eq!(report["action"], "create_pin_pad_map");
    assert_eq!(report["object_kind"], "pin_pad_maps");

    let map_payload = query_pool_object_payload(&root, "pin_pad_maps", map_id);
    assert_eq!(map_payload["part"], fixture.part_id.to_string());
    assert_eq!(
        map_payload["mappings"][fixture.pad_ids[0].to_string()]["gate"],
        fixture.gate_id.to_string()
    );
    assert_eq!(
        map_payload["mappings"][fixture.pad_ids[0].to_string()]["pin"],
        fixture.pin_ids[0].to_string()
    );
    assert_eq!(
        map_payload["mappings"][fixture.pad_ids[1].to_string()]["gate"],
        fixture.gate_id.to_string()
    );
    assert_eq!(
        map_payload["mappings"][fixture.pad_ids[1].to_string()]["pin"],
        fixture.pin_ids[1].to_string()
    );
    let part_payload = query_pool_object_payload(&root, "parts", fixture.part_id);
    assert_eq!(part_payload["default_pin_pad_map"], map_id.to_string());
    assert_eq!(
        part_payload["pad_map"]
            .as_object()
            .expect("legacy pad_map should remain object")
            .len(),
        0
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_library_pin_pad_map_symbol_component_instance_workflow_is_end_to_end() {
    let root = unique_project_root("datum-eda-cli-project-library-pin-pad-map-e2e");
    create_native_project(&root, Some("Library PinPadMap E2E".to_string()))
        .expect("initial scaffold should succeed");
    let sheet_uuid = super::main_tests_project_schematic_proposals::seed_native_sheet(&root);
    let fixture = create_fixture(&root, &["IN", "OUT"], &["1", "2"]);
    let map_id = create_default_pin_pad_map(
        &root,
        &fixture,
        &[
            (fixture.pin_ids[0], fixture.pad_ids[0]),
            (fixture.pin_ids[1], fixture.pad_ids[1]),
        ],
    );

    let map_payload = query_pool_object_payload(&root, "pin_pad_maps", map_id);
    assert_eq!(map_payload["part"], fixture.part_id.to_string());
    assert_eq!(map_payload["footprint"], fixture.footprint_id.to_string());
    assert_eq!(
        map_payload["mappings"][fixture.pad_ids[0].to_string()]["pin"],
        fixture.pin_ids[0].to_string()
    );

    let place_output = run_project_command(&[
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
        "PINNED",
        "--lib-id",
        &fixture.symbol_id.to_string(),
        "--x-nm",
        "1000",
        "--y-nm",
        "2000",
    ])
    .expect("pool symbol placement should succeed");
    let placed: serde_json::Value =
        serde_json::from_str(&place_output).expect("place-symbol JSON should parse");
    assert_eq!(placed["binding_status"], "bound_with_part");
    assert_eq!(placed["gate_uuid"], fixture.gate_id.to_string());
    assert_eq!(placed["part_uuid"], fixture.part_id.to_string());
    let placed_symbol_id = placed["symbol_uuid"]
        .as_str()
        .expect("placed symbol id should be string")
        .to_string();
    let component_instance_id = placed["component_instance_uuid"]
        .as_str()
        .expect("component instance id should be string")
        .to_string();

    let component_instances =
        super::main_tests_project_schematic_proposals::query_component_instances(&root);
    assert_eq!(
        component_instances["component_instances"][&component_instance_id]["part_ref"],
        fixture.part_id.to_string()
    );
    assert_eq!(
        component_instances["component_instances"][&component_instance_id]["placed_symbol_refs"],
        serde_json::json!([placed_symbol_id])
    );

    let validate_output = run_project_command(&[
        "eda",
        "--format",
        "json",
        "project",
        "validate",
        root.to_str().unwrap(),
    ])
    .expect("project validate should succeed");
    let validate_report: serde_json::Value =
        serde_json::from_str(&validate_output).expect("validate JSON should parse");
    assert_eq!(validate_report["valid"], true);
    assert!(
        validate_report["issues"]
            .as_array()
            .expect("issues should be array")
            .iter()
            .all(|issue| issue["severity"] != "error")
    );

    let erc_output = run_project_command(&[
        "eda",
        "--format",
        "json",
        "project",
        "query",
        root.to_str().unwrap(),
        "erc",
    ])
    .expect("project ERC query should succeed");
    let erc_report: serde_json::Value =
        serde_json::from_str(&erc_output).expect("ERC JSON should parse");
    assert_eq!(erc_report["profile_id"], "erc");
    assert!(erc_report["raw_report"]["erc"].is_array());

    let check_output = run_project_command(&[
        "eda",
        "--format",
        "json",
        "check",
        "run",
        root.to_str().unwrap(),
        "--profile",
        "native-combined",
    ])
    .expect("combined check run should succeed");
    let check_report: serde_json::Value =
        serde_json::from_str(&check_output).expect("check-run JSON should parse");
    assert_eq!(check_report["contract"], "check_run_v1");
    assert_eq!(check_report["profile_id"], "native-combined");
    assert!(
        check_report["coverage"]
            .as_array()
            .unwrap()
            .iter()
            .any(|entry| {
                entry["domain"] == "erc"
                    && entry["rule_id"] == "schematic_connectivity"
                    && entry["status"] == "evaluated"
            })
    );

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_set_pool_pin_pad_map_replaces_mappings_without_touching_part_pad_map() {
    let root = unique_project_root("datum-eda-cli-project-pool-pin-pad-map-replace");
    create_native_project(&root, Some("Pool PinPadMap Replace".to_string()))
        .expect("initial scaffold should succeed");
    let fixture = create_fixture(&root, &["IN+", "OUT"], &["1", "2"]);
    let map_id = Uuid::new_v4();
    run_project_command(&[
        "eda",
        "--format",
        "json",
        "project",
        "create-pool-pin-pad-map",
        root.to_str().unwrap(),
        "--map",
        &map_id.to_string(),
        "--part",
        &fixture.part_id.to_string(),
        "--footprint",
        &fixture.footprint_id.to_string(),
        "--entry",
        &format!("{}:{}", fixture.pin_ids[0], fixture.pad_ids[0]),
    ])
    .expect("PinPadMap create should succeed");

    let output = run_project_command(&[
        "eda",
        "--format",
        "json",
        "project",
        "set-pool-pin-pad-map",
        root.to_str().unwrap(),
        "--map",
        &map_id.to_string(),
        "--mode",
        "replace",
        "--entry",
        &format!("{}:{}", fixture.pin_ids[1], fixture.pad_ids[1]),
    ])
    .expect("PinPadMap replace should succeed");
    let report: serde_json::Value =
        serde_json::from_str(&output).expect("PinPadMap report JSON should parse");
    assert_eq!(report["action"], "set_pin_pad_map");

    let map_payload = query_pool_object_payload(&root, "pin_pad_maps", map_id);
    let mappings = map_payload["mappings"]
        .as_object()
        .expect("mappings should be object");
    assert_eq!(mappings.len(), 1);
    assert!(mappings.get(&fixture.pad_ids[0].to_string()).is_none());
    assert_eq!(
        map_payload["mappings"][fixture.pad_ids[1].to_string()]["pin"],
        fixture.pin_ids[1].to_string()
    );
    let part_payload = query_pool_object_payload(&root, "parts", fixture.part_id);
    assert_eq!(
        part_payload["pad_map"]
            .as_object()
            .expect("legacy pad_map should remain object")
            .len(),
        0
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_create_pool_pin_pad_map_rejects_missing_pad_without_writing_map() {
    let root = unique_project_root("datum-eda-cli-project-pool-pin-pad-map-missing-pad");
    create_native_project(&root, Some("Pool PinPadMap Missing Pad".to_string()))
        .expect("initial scaffold should succeed");
    let fixture = create_fixture(&root, &["IN+"], &["1"]);
    let map_id = Uuid::new_v4();
    let missing_pad = Uuid::new_v4();

    let error = run_project_command(&[
        "eda",
        "--format",
        "json",
        "project",
        "create-pool-pin-pad-map",
        root.to_str().unwrap(),
        "--map",
        &map_id.to_string(),
        "--part",
        &fixture.part_id.to_string(),
        "--entry",
        &format!("{}:{missing_pad}", fixture.pin_ids[0]),
    ])
    .expect_err("missing pad should fail");
    assert!(format!("{error:#}").contains("has no pad"));
    assert!(
        !root
            .join(format!("pool/pin_pad_maps/{map_id}.json"))
            .exists()
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn proposal_create_pool_pin_pad_map_defers_until_accept_apply() {
    let root = unique_project_root("datum-eda-cli-proposal-pool-pin-pad-map");
    create_native_project(&root, Some("Pool PinPadMap Proposal".to_string()))
        .expect("initial scaffold should succeed");
    let fixture = create_fixture(&root, &["IN+"], &["1"]);
    let map_id = Uuid::new_v4();
    let proposal_id = Uuid::new_v4();
    let map_path = root.join(format!("pool/pin_pad_maps/{map_id}.json"));

    let output = run_project_command(&[
        "eda",
        "--format",
        "json",
        "proposal",
        "create-pool-pin-pad-map",
        root.to_str().unwrap(),
        "--map",
        &map_id.to_string(),
        "--part",
        &fixture.part_id.to_string(),
        "--footprint",
        &fixture.footprint_id.to_string(),
        "--entry",
        &format!("{}:{}", fixture.pin_ids[0], fixture.pad_ids[0]),
        "--set-default",
        "--proposal",
        &proposal_id.to_string(),
        "--rationale",
        "review pin pad map",
    ])
    .expect("PinPadMap proposal create should succeed");
    let report: serde_json::Value =
        serde_json::from_str(&output).expect("proposal report JSON should parse");
    assert_eq!(report["contract"], "proposal_create_v1");
    assert_eq!(report["action"], "create_pool_pin_pad_map_proposal");
    assert_eq!(report["proposal_id"], proposal_id.to_string());
    assert!(
        !map_path.exists(),
        "proposal creation must not write the PinPadMap shard"
    );
    let part_payload = query_pool_object_payload(&root, "parts", fixture.part_id);
    assert!(
        part_payload
            .get("default_pin_pad_map")
            .is_none_or(serde_json::Value::is_null)
    );

    run_project_command(&[
        "eda",
        "--format",
        "json",
        "proposal",
        "accept-apply",
        root.to_str().unwrap(),
        "--proposal",
        &proposal_id.to_string(),
    ])
    .expect("PinPadMap proposal accept-apply should succeed");

    assert!(map_path.exists());
    let map_payload = query_pool_object_payload(&root, "pin_pad_maps", map_id);
    assert_eq!(map_payload["part"], fixture.part_id.to_string());
    assert_eq!(map_payload["footprint"], fixture.footprint_id.to_string());
    assert_eq!(
        map_payload["mappings"][fixture.pad_ids[0].to_string()]["pin"],
        fixture.pin_ids[0].to_string()
    );
    let part_payload = query_pool_object_payload(&root, "parts", fixture.part_id);
    assert_eq!(part_payload["default_pin_pad_map"], map_id.to_string());
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn proposal_set_pool_pin_pad_map_defers_until_accept_apply() {
    let root = unique_project_root("datum-eda-cli-proposal-set-pool-pin-pad-map");
    create_native_project(&root, Some("Set Pool PinPadMap Proposal".to_string()))
        .expect("initial scaffold should succeed");
    let fixture = create_fixture(&root, &["A", "B"], &["1", "2"]);
    let map_id =
        create_default_pin_pad_map(&root, &fixture, &[(fixture.pin_ids[0], fixture.pad_ids[0])]);
    let proposal_id = Uuid::new_v4();

    let output = run_project_command(&[
        "eda",
        "--format",
        "json",
        "proposal",
        "set-pool-pin-pad-map",
        root.to_str().unwrap(),
        "--map",
        &map_id.to_string(),
        "--mode",
        "replace",
        "--entry",
        &format!("{}:{}", fixture.pin_ids[1], fixture.pad_ids[1]),
        "--proposal",
        &proposal_id.to_string(),
        "--rationale",
        "review pin pad map update",
    ])
    .expect("PinPadMap set proposal should succeed");
    let report: serde_json::Value =
        serde_json::from_str(&output).expect("proposal report JSON should parse");
    assert_eq!(report["action"], "set_pool_pin_pad_map_proposal");

    let map_payload = query_pool_object_payload(&root, "pin_pad_maps", map_id);
    assert_eq!(map_payload["mappings"].as_object().unwrap().len(), 1);
    assert!(
        map_payload["mappings"]
            .as_object()
            .unwrap()
            .contains_key(&fixture.pad_ids[0].to_string()),
        "proposal creation must not mutate mappings"
    );

    run_project_command(&[
        "eda",
        "--format",
        "json",
        "proposal",
        "accept-apply",
        root.to_str().unwrap(),
        "--proposal",
        &proposal_id.to_string(),
    ])
    .expect("PinPadMap set proposal accept-apply should succeed");

    let map_payload = query_pool_object_payload(&root, "pin_pad_maps", map_id);
    assert_eq!(map_payload["mappings"].as_object().unwrap().len(), 1);
    assert!(
        map_payload["mappings"]
            .as_object()
            .unwrap()
            .contains_key(&fixture.pad_ids[1].to_string())
    );
    assert_eq!(
        map_payload["mappings"][fixture.pad_ids[1].to_string()]["pin"],
        fixture.pin_ids[1].to_string()
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_legacy_part_pad_map_entry_bridges_to_default_pin_pad_map() {
    let root = unique_project_root("datum-eda-cli-project-pool-part-pad-map");
    create_native_project(&root, Some("Pool Part Pad Map".to_string()))
        .expect("initial scaffold should succeed");
    let fixture = create_fixture(&root, &["IN+"], &["1"]);
    let map_id =
        create_default_pin_pad_map(&root, &fixture, &[(fixture.pin_ids[0], fixture.pad_ids[0])]);
    let output = run_project_command(&[
        "eda",
        "--format",
        "json",
        "project",
        "set-pool-part-pad-map-entry",
        root.to_str().unwrap(),
        "--part",
        &fixture.part_id.to_string(),
        "--pad",
        &fixture.pad_ids[0].to_string(),
        "--gate",
        &fixture.gate_id.to_string(),
        "--pin",
        &fixture.pin_ids[0].to_string(),
    ])
    .expect("legacy pad map bridge should succeed");
    let report: serde_json::Value =
        serde_json::from_str(&output).expect("pad map report JSON should parse");
    assert_eq!(report["action"], "set_part_pad_map_entry");
    assert_eq!(report["object_kind"], "pin_pad_maps");
    assert_eq!(report["object_uuid"], map_id.to_string());
    let map_payload = query_pool_object_payload(&root, "pin_pad_maps", map_id);
    assert_eq!(
        map_payload["mappings"][fixture.pad_ids[0].to_string()]["gate"],
        fixture.gate_id.to_string()
    );
    assert_eq!(
        map_payload["mappings"][fixture.pad_ids[0].to_string()]["pin"],
        fixture.pin_ids[0].to_string()
    );
    let part_payload = query_pool_object_payload(&root, "parts", fixture.part_id);
    assert_eq!(part_payload["pad_map"].as_object().unwrap().len(), 0);
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_legacy_part_pad_map_requires_default_pin_pad_map() {
    let root = unique_project_root("datum-eda-cli-project-pool-part-pad-map-no-default");
    create_native_project(&root, Some("Pool Part Pad Map No Default".to_string()))
        .expect("initial scaffold should succeed");
    let fixture = create_fixture(&root, &["IN+"], &["1"]);
    let error = run_project_command(&[
        "eda",
        "--format",
        "json",
        "project",
        "set-pool-part-pad-map-entry",
        root.to_str().unwrap(),
        "--part",
        &fixture.part_id.to_string(),
        "--pad",
        &fixture.pad_ids[0].to_string(),
        "--gate",
        &fixture.gate_id.to_string(),
        "--pin",
        &fixture.pin_ids[0].to_string(),
    ])
    .expect_err("legacy pad map command should require default PinPadMap");
    assert!(format!("{error:#}").contains("requires part default_pin_pad_map"));
    let part_payload = query_pool_object_payload(&root, "parts", fixture.part_id);
    assert_eq!(part_payload["pad_map"].as_object().unwrap().len(), 0);
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_legacy_part_pad_map_requires_existing_default_pin_pad_map() {
    let root = unique_project_root("datum-eda-cli-project-pool-part-pad-map-missing-default");
    create_native_project(&root, Some("Pool Part Pad Map Missing Default".to_string()))
        .expect("initial scaffold should succeed");
    let fixture = create_fixture(&root, &["IN+"], &["1"]);
    let missing_map_id = Uuid::new_v4();
    set_part_default_pin_pad_map_raw(&root, fixture.part_id, missing_map_id);

    let error = run_project_command(&[
        "eda",
        "--format",
        "json",
        "project",
        "set-pool-part-pad-map-entry",
        root.to_str().unwrap(),
        "--part",
        &fixture.part_id.to_string(),
        "--pad",
        &fixture.pad_ids[0].to_string(),
        "--gate",
        &fixture.gate_id.to_string(),
        "--pin",
        &fixture.pin_ids[0].to_string(),
    ])
    .expect_err("legacy pad map command should require an existing default PinPadMap");
    let error = format!("{error:#}");
    assert!(
        error.contains("default_pin_pad_map") || error.contains(&missing_map_id.to_string()),
        "unexpected error: {error}"
    );
    let part_payload = query_pool_object_payload(&root, "parts", fixture.part_id);
    assert_eq!(
        part_payload["default_pin_pad_map"],
        missing_map_id.to_string()
    );
    assert_eq!(part_payload["pad_map"].as_object().unwrap().len(), 0);
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_legacy_part_pad_map_replace_and_duplicate_checks_target_default_map() {
    let root = unique_project_root("datum-eda-cli-project-pool-part-pad-map-replace");
    create_native_project(&root, Some("Pool Part Pad Map Replace".to_string()))
        .expect("initial scaffold should succeed");
    let fixture = create_fixture(&root, &["IN+", "OUT"], &["1", "2"]);
    let map_id =
        create_default_pin_pad_map(&root, &fixture, &[(fixture.pin_ids[0], fixture.pad_ids[0])]);
    let output = run_project_command(&[
        "eda",
        "--format",
        "json",
        "project",
        "set-pool-part-pad-map",
        root.to_str().unwrap(),
        "--part",
        &fixture.part_id.to_string(),
        "--mode",
        "replace",
        "--entry",
        &format!(
            "{}:{}:{}",
            fixture.pad_ids[1], fixture.gate_id, fixture.pin_ids[1]
        ),
    ])
    .expect("bulk pad map replace should succeed");
    let report: serde_json::Value =
        serde_json::from_str(&output).expect("pad map report JSON should parse");
    assert_eq!(report["object_kind"], "pin_pad_maps");
    let map_payload = query_pool_object_payload(&root, "pin_pad_maps", map_id);
    assert_eq!(map_payload["mappings"].as_object().unwrap().len(), 1);
    assert_eq!(
        map_payload["mappings"][fixture.pad_ids[1].to_string()]["gate"],
        fixture.gate_id.to_string()
    );
    assert_eq!(
        map_payload["mappings"][fixture.pad_ids[1].to_string()]["pin"],
        fixture.pin_ids[1].to_string()
    );
    let entry = format!(
        "{}:{}:{}",
        fixture.pad_ids[1], fixture.gate_id, fixture.pin_ids[1]
    );
    let error = run_project_command(&[
        "eda",
        "--format",
        "json",
        "project",
        "set-pool-part-pad-map",
        root.to_str().unwrap(),
        "--part",
        &fixture.part_id.to_string(),
        "--entry",
        &entry,
        "--entry",
        &entry,
    ])
    .expect_err("duplicate entries should fail");
    assert!(format!("{error:#}").contains("duplicate pad-map entry for gate"));
    let part_payload = query_pool_object_payload(&root, "parts", fixture.part_id);
    assert_eq!(part_payload["pad_map"].as_object().unwrap().len(), 0);
    let _ = std::fs::remove_dir_all(&root);
}
