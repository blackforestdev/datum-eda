use super::*;

struct PinnedMapFixture {
    part_id: Uuid,
    gate_id: Uuid,
    footprint_id: Uuid,
    pin_ids: Vec<Uuid>,
    pad_ids: Vec<Uuid>,
}

fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}", label, Uuid::new_v4()))
}

fn run_project_command(args: &[&str]) -> Result<String> {
    execute(Cli::try_parse_from(args).expect("CLI should parse"))
}

fn query_pool_object_payload(root: &Path, kind: &str, object_id: Uuid) -> serde_json::Value {
    let path = root.join(format!("pool/{kind}/{object_id}.json"));
    serde_json::from_str(&std::fs::read_to_string(path).expect("pool object should read"))
        .expect("pool object should parse")
}

fn set_part_default_pin_pad_map_raw(root: &Path, part_id: Uuid, map_id: Uuid) {
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

fn create_fixture(root: &Path, pin_names: &[&str], pad_names: &[&str]) -> PinnedMapFixture {
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

    let pin_ids = pin_names
        .iter()
        .map(|name| {
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
                "--direction",
                "Input",
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
        part_id,
        gate_id,
        footprint_id,
        pin_ids,
        pad_ids,
    }
}

fn create_default_pin_pad_map(
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
