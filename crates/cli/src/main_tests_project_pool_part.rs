use super::*;

fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}", label, Uuid::new_v4()))
}

fn run_project_command(args: &[&str]) -> Result<String> {
    execute(Cli::try_parse_from(args).expect("CLI should parse"))
}

fn transaction_operation_kinds(root: &Path) -> Vec<String> {
    let journal = std::fs::read_to_string(root.join(".datum/journal/transactions.jsonl"))
        .expect("transaction journal should read");
    journal
        .lines()
        .flat_map(|line| {
            let transaction: serde_json::Value =
                serde_json::from_str(line).expect("transaction line should parse");
            transaction["operations"]
                .as_array()
                .expect("transaction operations should be array")
                .iter()
                .filter_map(|operation| operation["kind"].as_str().map(str::to_string))
                .collect::<Vec<_>>()
        })
        .collect()
}

fn create_minimal_entity_and_package(root: &Path) -> (Uuid, Uuid) {
    let unit_id = Uuid::new_v4();
    let symbol_id = Uuid::new_v4();
    let entity_id = Uuid::new_v4();
    let gate_id = Uuid::new_v4();
    let padstack_id = Uuid::new_v4();
    let package_id = Uuid::new_v4();
    let pad_id = Uuid::new_v4();
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
        "OpAmpUnit",
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
        "OpAmpSymbol",
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
        "OpAmp",
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
    (entity_id, package_id)
}

fn query_pool_object_payload(root: &Path, kind: &str, object_id: Uuid) -> serde_json::Value {
    let output = run_project_command(&[
        "eda",
        "--format",
        "json",
        "project",
        "query",
        root.to_str().unwrap(),
        "pool-library-objects",
        "--kind",
        kind,
        "--object",
        &object_id.to_string(),
        "--include-payload",
    ])
    .expect("pool query should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).expect("query JSON should parse");
    assert_eq!(report["object_count"], 1);
    report["objects"][0]["payload"].clone()
}

fn query_pool_models(root: &Path, extra: &[&str]) -> serde_json::Value {
    let mut args = vec![
        "eda",
        "--format",
        "json",
        "project",
        "query",
        root.to_str().unwrap(),
        "pool-models",
    ];
    args.extend_from_slice(extra);
    let output = run_project_command(&args).expect("pool model query should succeed");
    serde_json::from_str(&output).expect("pool model query JSON should parse")
}

fn create_minimal_part(root: &Path) -> Uuid {
    let (entity_id, package_id) = create_minimal_entity_and_package(root);
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
        "BASE-MPN",
        "--manufacturer",
        "Base Manufacturer",
        "--value",
        "BASE",
    ])
    .expect("part create should succeed");
    part_id
}

fn create_minimal_package(root: &Path, pad_name: &str) -> (Uuid, Uuid, Uuid) {
    let padstack_id = Uuid::new_v4();
    let package_id = Uuid::new_v4();
    let pad_id = Uuid::new_v4();
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
        "--pad-name",
        pad_name,
    ])
    .expect("package create should succeed");
    (padstack_id, package_id, pad_id)
}

struct PinnedPartFixture {
    part_id: Uuid,
    gate_id: Uuid,
    pin_ids: Vec<Uuid>,
    pad_ids: Vec<Uuid>,
}

fn create_pinned_part_fixture(
    root: &Path,
    pin_names: &[&str],
    pad_names: &[&str],
) -> PinnedPartFixture {
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
    let (padstack_id, package_id, first_pad_id) = create_minimal_package(root, pad_names[0]);
    let mut pad_ids = vec![first_pad_id];
    for name in &pad_names[1..] {
        let pad_id = Uuid::new_v4();
        run_project_command(&[
            "eda",
            "--format",
            "json",
            "project",
            "set-pool-package-pad",
            root.to_str().unwrap(),
            "--package",
            &package_id.to_string(),
            "--pad",
            &pad_id.to_string(),
            "--padstack",
            &padstack_id.to_string(),
            "--pad-name",
            name,
        ])
        .expect("package pad set should succeed");
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
    PinnedPartFixture {
        part_id,
        gate_id,
        pin_ids,
        pad_ids,
    }
}

#[test]
fn project_create_pool_part_authors_typed_part_for_entity_package_pair() {
    let root = unique_project_root("datum-eda-cli-project-pool-part");
    create_native_project(&root, Some("Pool Part".to_string()))
        .expect("initial scaffold should succeed");
    let (entity_id, package_id) = create_minimal_entity_and_package(&root);
    let part_id = Uuid::new_v4();
    let output = run_project_command(&[
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
    .expect("part create should succeed");
    let report: serde_json::Value =
        serde_json::from_str(&output).expect("part report JSON should parse");
    assert_eq!(report["action"], "create_part");
    assert_eq!(
        report["relative_path"],
        format!("pool/parts/{part_id}.json")
    );
    let payload = query_pool_object_payload(&root, "parts", part_id);
    assert_eq!(payload["entity"], entity_id.to_string());
    assert_eq!(payload["package"], package_id.to_string());
    assert_eq!(payload["mpn"], "OPA1656ID");
    assert_eq!(payload["lifecycle"], "Active");
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_create_pool_part_rejects_missing_package() {
    let root = unique_project_root("datum-eda-cli-project-pool-part-missing-package");
    create_native_project(&root, Some("Pool Part Missing Package".to_string()))
        .expect("initial scaffold should succeed");
    let (entity_id, _) = create_minimal_entity_and_package(&root);
    let part_id = Uuid::new_v4();
    let missing_package = Uuid::new_v4();
    let error = run_project_command(&[
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
        &missing_package.to_string(),
        "--mpn",
        "BAD",
    ])
    .expect_err("part with missing package should fail");
    assert!(format!("{error:#}").contains("missing pool package"));
    assert!(!root.join(format!("pool/parts/{part_id}.json")).exists());
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_set_pool_part_metadata_updates_supplied_fields_and_undoes() {
    let root = unique_project_root("datum-eda-cli-project-pool-part-metadata");
    create_native_project(&root, Some("Pool Part Metadata".to_string()))
        .expect("initial scaffold should succeed");
    let part_id = create_minimal_part(&root);
    let output = run_project_command(&[
        "eda",
        "--format",
        "json",
        "project",
        "set-pool-part-metadata",
        root.to_str().unwrap(),
        "--part",
        &part_id.to_string(),
        "--mpn",
        "UPDATED-MPN",
        "--description",
        "Updated description",
        "--manufacturer-jep106",
        "41",
        "--lifecycle",
        "Nrnd",
    ])
    .expect("metadata update should succeed");
    let report: serde_json::Value =
        serde_json::from_str(&output).expect("metadata report JSON should parse");
    assert_eq!(report["action"], "set_part_metadata");
    assert_eq!(report["object_kind"], "parts");
    let payload = query_pool_object_payload(&root, "parts", part_id);
    assert_eq!(payload["mpn"], "UPDATED-MPN");
    assert_eq!(payload["manufacturer"], "Base Manufacturer");
    assert_eq!(payload["manufacturer_jep106"], 41);
    assert_eq!(payload["description"], "Updated description");
    assert_eq!(payload["lifecycle"], "Nrnd");
    run_project_command(&[
        "eda",
        "--format",
        "json",
        "project",
        "undo",
        root.to_str().unwrap(),
    ])
    .expect("metadata update undo should succeed");
    let payload = query_pool_object_payload(&root, "parts", part_id);
    assert_eq!(payload["mpn"], "BASE-MPN");
    assert!(payload["manufacturer_jep106"].is_null());
    assert_eq!(payload["description"], "");
    assert_eq!(payload["lifecycle"], "Active");
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_set_pool_part_metadata_rejects_invalid_lifecycle() {
    let root = unique_project_root("datum-eda-cli-project-pool-part-metadata-lifecycle");
    create_native_project(&root, Some("Pool Part Metadata Lifecycle".to_string()))
        .expect("initial scaffold should succeed");
    let part_id = create_minimal_part(&root);
    let error = run_project_command(&[
        "eda",
        "--format",
        "json",
        "project",
        "set-pool-part-metadata",
        root.to_str().unwrap(),
        "--part",
        &part_id.to_string(),
        "--lifecycle",
        "Dormant",
    ])
    .expect_err("invalid lifecycle should fail");
    assert!(format!("{error:#}").contains("unsupported part lifecycle Dormant"));
    assert_eq!(
        query_pool_object_payload(&root, "parts", part_id)["lifecycle"],
        "Active"
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_set_pool_part_metadata_rejects_invalid_jep106_code() {
    let root = unique_project_root("datum-eda-cli-project-pool-part-metadata-jep106");
    create_native_project(&root, Some("Pool Part Metadata Jep106".to_string()))
        .expect("initial scaffold should succeed");
    let part_id = create_minimal_part(&root);
    let error = run_project_command(&[
        "eda",
        "--format",
        "json",
        "project",
        "set-pool-part-metadata",
        root.to_str().unwrap(),
        "--part",
        &part_id.to_string(),
        "--manufacturer-jep106",
        "2048",
    ])
    .expect_err("invalid JEP106 code should fail");
    assert!(
        format!("{error:#}").contains("manufacturer-jep106 must be a valid 11-bit JEP106 code")
    );
    assert!(query_pool_object_payload(&root, "parts", part_id)["manufacturer_jep106"].is_null());
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_set_pool_part_metadata_rejects_no_fields() {
    let root = unique_project_root("datum-eda-cli-project-pool-part-metadata-empty");
    create_native_project(&root, Some("Pool Part Metadata Empty".to_string()))
        .expect("initial scaffold should succeed");
    let part_id = create_minimal_part(&root);
    let error = run_project_command(&[
        "eda",
        "--format",
        "json",
        "project",
        "set-pool-part-metadata",
        root.to_str().unwrap(),
        "--part",
        &part_id.to_string(),
    ])
    .expect_err("metadata update without fields should fail");
    assert!(format!("{error:#}").contains("requires at least one metadata field"));
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_set_pool_part_metadata_rejects_missing_part() {
    let root = unique_project_root("datum-eda-cli-project-pool-part-metadata-missing");
    create_native_project(&root, Some("Pool Part Metadata Missing".to_string()))
        .expect("initial scaffold should succeed");
    let missing_part = Uuid::new_v4();
    let error = run_project_command(&[
        "eda",
        "--format",
        "json",
        "project",
        "set-pool-part-metadata",
        root.to_str().unwrap(),
        "--part",
        &missing_part.to_string(),
        "--mpn",
        "MISSING",
    ])
    .expect_err("missing part metadata update should fail");
    assert!(format!("{error:#}").contains("failed to read pool library object"));
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_set_pool_part_parametric_merges_replaces_and_undoes() {
    let root = unique_project_root("datum-eda-cli-project-pool-part-parametric");
    create_native_project(&root, Some("Pool Part Parametric".to_string()))
        .expect("initial scaffold should succeed");
    let part_id = create_minimal_part(&root);
    let output = run_project_command(&[
        "eda",
        "--format",
        "json",
        "project",
        "set-pool-part-parametric",
        root.to_str().unwrap(),
        "--part",
        &part_id.to_string(),
        "--param",
        "voltage=36V",
        "--param",
        "tolerance=1%",
    ])
    .expect("parametric merge should succeed");
    let report: serde_json::Value =
        serde_json::from_str(&output).expect("parametric report JSON should parse");
    assert_eq!(report["action"], "set_part_parametric");
    let payload = query_pool_object_payload(&root, "parts", part_id);
    assert_eq!(payload["parametric"]["voltage"], "36V");
    assert_eq!(payload["parametric"]["tolerance"], "1%");
    run_project_command(&[
        "eda",
        "--format",
        "json",
        "project",
        "set-pool-part-parametric",
        root.to_str().unwrap(),
        "--part",
        &part_id.to_string(),
        "--mode",
        "replace",
        "--param",
        "power=250mW",
    ])
    .expect("parametric replace should succeed");
    let payload = query_pool_object_payload(&root, "parts", part_id);
    assert!(payload["parametric"]["voltage"].is_null());
    assert_eq!(payload["parametric"]["power"], "250mW");
    run_project_command(&[
        "eda",
        "--format",
        "json",
        "project",
        "undo",
        root.to_str().unwrap(),
    ])
    .expect("parametric replace undo should succeed");
    let payload = query_pool_object_payload(&root, "parts", part_id);
    assert_eq!(payload["parametric"]["voltage"], "36V");
    assert_eq!(payload["parametric"]["tolerance"], "1%");
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_set_pool_part_parametric_rejects_invalid_requests() {
    let root = unique_project_root("datum-eda-cli-project-pool-part-parametric-invalid");
    create_native_project(&root, Some("Pool Part Parametric Invalid".to_string()))
        .expect("initial scaffold should succeed");
    let part_id = create_minimal_part(&root);
    let duplicate_error = run_project_command(&[
        "eda",
        "--format",
        "json",
        "project",
        "set-pool-part-parametric",
        root.to_str().unwrap(),
        "--part",
        &part_id.to_string(),
        "--param",
        "voltage=36V",
        "--param",
        "voltage=48V",
    ])
    .expect_err("duplicate parametric key should fail");
    assert!(format!("{duplicate_error:#}").contains("duplicate part parametric key voltage"));
    let malformed_error = run_project_command(&[
        "eda",
        "--format",
        "json",
        "project",
        "set-pool-part-parametric",
        root.to_str().unwrap(),
        "--part",
        &part_id.to_string(),
        "--param",
        "voltage",
    ])
    .expect_err("malformed parametric entry should fail");
    assert!(format!("{malformed_error:#}").contains("must be key=value"));
    let mode_error = run_project_command(&[
        "eda",
        "--format",
        "json",
        "project",
        "set-pool-part-parametric",
        root.to_str().unwrap(),
        "--part",
        &part_id.to_string(),
        "--mode",
        "append",
        "--param",
        "voltage=36V",
    ])
    .expect_err("unsupported mode should fail");
    assert!(format!("{mode_error:#}").contains("expected merge or replace"));
    assert_eq!(
        query_pool_object_payload(&root, "parts", part_id)["parametric"],
        serde_json::json!({})
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_set_pool_part_orderable_mpns_merges_replaces_and_undoes() {
    let root = unique_project_root("datum-eda-cli-project-pool-part-orderable-mpns");
    create_native_project(&root, Some("Pool Part Orderable MPNs".to_string()))
        .expect("initial scaffold should succeed");
    let part_id = create_minimal_part(&root);
    let output = run_project_command(&[
        "eda",
        "--format",
        "json",
        "project",
        "set-pool-part-orderable-mpns",
        root.to_str().unwrap(),
        "--part",
        &part_id.to_string(),
        "--mpn",
        "OPA1656ID",
        "--mpn",
        "OPA1656IDR",
    ])
    .expect("orderable MPN merge should succeed");
    let report: serde_json::Value =
        serde_json::from_str(&output).expect("orderable MPN report JSON should parse");
    assert_eq!(report["action"], "set_part_orderable_mpns");
    let payload = query_pool_object_payload(&root, "parts", part_id);
    assert_eq!(
        payload["orderable_mpns"],
        serde_json::json!(["OPA1656ID", "OPA1656IDR"])
    );
    run_project_command(&[
        "eda",
        "--format",
        "json",
        "project",
        "set-pool-part-orderable-mpns",
        root.to_str().unwrap(),
        "--part",
        &part_id.to_string(),
        "--mode",
        "replace",
        "--mpn",
        "OPA1656IDRGTR",
    ])
    .expect("orderable MPN replace should succeed");
    let payload = query_pool_object_payload(&root, "parts", part_id);
    assert_eq!(
        payload["orderable_mpns"],
        serde_json::json!(["OPA1656IDRGTR"])
    );
    run_project_command(&[
        "eda",
        "--format",
        "json",
        "project",
        "undo",
        root.to_str().unwrap(),
    ])
    .expect("orderable MPN replace undo should succeed");
    let payload = query_pool_object_payload(&root, "parts", part_id);
    assert_eq!(
        payload["orderable_mpns"],
        serde_json::json!(["OPA1656ID", "OPA1656IDR"])
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_set_pool_part_orderable_mpns_rejects_invalid_requests() {
    let root = unique_project_root("datum-eda-cli-project-pool-part-orderable-mpns-invalid");
    create_native_project(&root, Some("Pool Part Orderable MPNs Invalid".to_string()))
        .expect("initial scaffold should succeed");
    let part_id = create_minimal_part(&root);
    let duplicate_error = run_project_command(&[
        "eda",
        "--format",
        "json",
        "project",
        "set-pool-part-orderable-mpns",
        root.to_str().unwrap(),
        "--part",
        &part_id.to_string(),
        "--mpn",
        "OPA1656ID",
        "--mpn",
        "OPA1656ID",
    ])
    .expect_err("duplicate orderable MPN should fail");
    assert!(format!("{duplicate_error:#}").contains("duplicate part orderable MPN OPA1656ID"));
    let blank_error = run_project_command(&[
        "eda",
        "--format",
        "json",
        "project",
        "set-pool-part-orderable-mpns",
        root.to_str().unwrap(),
        "--part",
        &part_id.to_string(),
        "--mpn",
        " ",
    ])
    .expect_err("blank orderable MPN should fail");
    assert!(format!("{blank_error:#}").contains("part orderable MPN must be non-empty"));
    let mode_error = run_project_command(&[
        "eda",
        "--format",
        "json",
        "project",
        "set-pool-part-orderable-mpns",
        root.to_str().unwrap(),
        "--part",
        &part_id.to_string(),
        "--mode",
        "append",
        "--mpn",
        "OPA1656ID",
    ])
    .expect_err("unsupported mode should fail");
    assert!(format!("{mode_error:#}").contains("expected merge or replace"));
    assert_eq!(
        query_pool_object_payload(&root, "parts", part_id)["orderable_mpns"],
        serde_json::json!([])
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_set_pool_part_packaging_options_merges_replaces_and_undoes() {
    let root = unique_project_root("datum-eda-cli-project-pool-part-packaging-options");
    create_native_project(&root, Some("Pool Part Packaging Options".to_string()))
        .expect("initial scaffold should succeed");
    let part_id = create_minimal_part(&root);
    let reel = r#"{"kind":"reel","tape_width_mm":8,"reel_diameter_inch":7,"qty_per_reel":3000,"mpn_suffix":"R"}"#;
    let tray = r#"{"kind":"tray","qty_per_tray":90}"#;
    let cut = r#"{"kind":"cut","qty":10,"mpn_suffix":"CT"}"#;
    let output = run_project_command(&[
        "eda",
        "--format",
        "json",
        "project",
        "set-pool-part-packaging-options",
        root.to_str().unwrap(),
        "--part",
        &part_id.to_string(),
        "--option",
        reel,
        "--option",
        tray,
    ])
    .expect("packaging option merge should succeed");
    let report: serde_json::Value =
        serde_json::from_str(&output).expect("packaging option report JSON should parse");
    assert_eq!(report["action"], "set_part_packaging_options");
    let payload = query_pool_object_payload(&root, "parts", part_id);
    assert_eq!(payload["packaging_options"].as_array().unwrap().len(), 2);
    assert_eq!(payload["packaging_options"][0]["kind"], "reel");
    assert_eq!(payload["packaging_options"][0]["tape_width_mm"], 8);
    assert_eq!(payload["packaging_options"][0]["mpn_suffix"], "R");
    assert_eq!(payload["packaging_options"][1]["kind"], "tray");
    run_project_command(&[
        "eda",
        "--format",
        "json",
        "project",
        "set-pool-part-packaging-options",
        root.to_str().unwrap(),
        "--part",
        &part_id.to_string(),
        "--mode",
        "replace",
        "--option",
        cut,
    ])
    .expect("packaging option replace should succeed");
    let payload = query_pool_object_payload(&root, "parts", part_id);
    assert_eq!(payload["packaging_options"].as_array().unwrap().len(), 1);
    assert_eq!(payload["packaging_options"][0]["kind"], "cut");
    assert_eq!(payload["packaging_options"][0]["qty"], 10);
    run_project_command(&[
        "eda",
        "--format",
        "json",
        "project",
        "undo",
        root.to_str().unwrap(),
    ])
    .expect("packaging option replace undo should succeed");
    let payload = query_pool_object_payload(&root, "parts", part_id);
    assert_eq!(payload["packaging_options"].as_array().unwrap().len(), 2);
    assert_eq!(payload["packaging_options"][0]["kind"], "reel");
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_set_pool_part_packaging_options_rejects_invalid_requests() {
    let root = unique_project_root("datum-eda-cli-project-pool-part-packaging-options-invalid");
    create_native_project(
        &root,
        Some("Pool Part Packaging Options Invalid".to_string()),
    )
    .expect("initial scaffold should succeed");
    let part_id = create_minimal_part(&root);
    let reel = r#"{"kind":"reel","tape_width_mm":8,"reel_diameter_inch":7,"qty_per_reel":3000}"#;
    let duplicate_error = run_project_command(&[
        "eda",
        "--format",
        "json",
        "project",
        "set-pool-part-packaging-options",
        root.to_str().unwrap(),
        "--part",
        &part_id.to_string(),
        "--option",
        reel,
        "--option",
        reel,
    ])
    .expect_err("duplicate packaging option should fail");
    assert!(format!("{duplicate_error:#}").contains("duplicate part packaging option"));
    let malformed_error = run_project_command(&[
        "eda",
        "--format",
        "json",
        "project",
        "set-pool-part-packaging-options",
        root.to_str().unwrap(),
        "--part",
        &part_id.to_string(),
        "--option",
        "reel",
    ])
    .expect_err("malformed packaging option should fail");
    assert!(format!("{malformed_error:#}").contains("part packaging option must be a JSON object"));
    let schema_error = run_project_command(&[
        "eda",
        "--format",
        "json",
        "project",
        "set-pool-part-packaging-options",
        root.to_str().unwrap(),
        "--part",
        &part_id.to_string(),
        "--option",
        r#"{"kind":"reel","tape_width_mm":8}"#,
    ])
    .expect_err("schema-invalid packaging option should fail");
    assert!(
        format!("{schema_error:#}")
            .contains("part packaging option must match a supported packaging schema")
    );
    let mode_error = run_project_command(&[
        "eda",
        "--format",
        "json",
        "project",
        "set-pool-part-packaging-options",
        root.to_str().unwrap(),
        "--part",
        &part_id.to_string(),
        "--mode",
        "append",
        "--option",
        reel,
    ])
    .expect_err("invalid packaging option mode should fail");
    assert!(format!("{mode_error:#}").contains("unsupported part packaging option mode append"));
    assert!(
        query_pool_object_payload(&root, "parts", part_id)["packaging_options"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_set_pool_part_behavioural_models_merges_replaces_and_undoes() {
    let root = unique_project_root("datum-eda-cli-project-pool-part-behavioural-models");
    create_native_project(&root, Some("Pool Part Behavioural Models".to_string()))
        .expect("initial scaffold should succeed");
    let part_id = create_minimal_part(&root);
    let spice_attachment = r#"{"uuid":"11111111-1111-1111-1111-111111111111","model_uuid":"22222222-2222-2222-2222-222222222222","role":"Spice","dialect":"Ngspice","model_names":["OPA1656"],"encrypted":false,"encryption_scheme":null,"provenance":{"source":"vendor/opa1656.lib","vendor":"Texas Instruments","fetched_at":null,"sha256":"0123456789abcdef"},"format_metadata":{"kind":"spice","ngspice_validates":true}}"#;
    let ibis_attachment = r#"{"uuid":"33333333-3333-3333-3333-333333333333","model_uuid":"44444444-4444-4444-4444-444444444444","role":"Ibis","dialect":null,"model_names":["OPA1656_IBIS"],"encrypted":false,"encryption_scheme":null,"provenance":null,"format_metadata":{"kind":"ibis","ibis_version":"7.2","has_ami":false}}"#;
    let thermal_attachment = r#"{"uuid":"55555555-5555-5555-5555-555555555555","model_uuid":"66666666-6666-6666-6666-666666666666","role":"CompactThermal","dialect":null,"model_names":[],"encrypted":false,"encryption_scheme":null,"provenance":null,"format_metadata":{"kind":"none"}}"#;
    let output = run_project_command(&[
        "eda",
        "--format",
        "json",
        "project",
        "set-pool-part-behavioural-models",
        root.to_str().unwrap(),
        "--part",
        &part_id.to_string(),
        "--model",
        spice_attachment,
        "--model",
        ibis_attachment,
    ])
    .expect("behavioural model merge should succeed");
    let report: serde_json::Value =
        serde_json::from_str(&output).expect("behavioural model report JSON should parse");
    assert_eq!(report["action"], "set_part_behavioural_models");
    let payload = query_pool_object_payload(&root, "parts", part_id);
    assert_eq!(payload["behavioural_models"].as_array().unwrap().len(), 2);
    assert_eq!(payload["behavioural_models"][0]["role"], "Spice");
    assert_eq!(
        payload["behavioural_models"][0]["format_metadata"]["kind"],
        "spice"
    );
    assert_eq!(payload["behavioural_models"][1]["role"], "Ibis");
    run_project_command(&[
        "eda",
        "--format",
        "json",
        "project",
        "set-pool-part-behavioural-models",
        root.to_str().unwrap(),
        "--part",
        &part_id.to_string(),
        "--mode",
        "replace",
        "--model",
        thermal_attachment,
    ])
    .expect("behavioural model replace should succeed");
    let payload = query_pool_object_payload(&root, "parts", part_id);
    assert_eq!(payload["behavioural_models"].as_array().unwrap().len(), 1);
    assert_eq!(payload["behavioural_models"][0]["role"], "CompactThermal");
    run_project_command(&[
        "eda",
        "--format",
        "json",
        "project",
        "undo",
        root.to_str().unwrap(),
    ])
    .expect("behavioural model replace undo should succeed");
    let payload = query_pool_object_payload(&root, "parts", part_id);
    assert_eq!(payload["behavioural_models"].as_array().unwrap().len(), 2);
    assert_eq!(payload["behavioural_models"][0]["role"], "Spice");
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_set_pool_part_behavioural_models_rejects_invalid_requests() {
    let root = unique_project_root("datum-eda-cli-project-pool-part-behavioural-models-invalid");
    create_native_project(
        &root,
        Some("Pool Part Behavioural Models Invalid".to_string()),
    )
    .expect("initial scaffold should succeed");
    let part_id = create_minimal_part(&root);
    let valid_attachment = r#"{"uuid":"11111111-1111-1111-1111-111111111111","model_uuid":"22222222-2222-2222-2222-222222222222","role":"Spice","dialect":"Ngspice","model_names":["OPA1656"],"encrypted":false,"encryption_scheme":null,"provenance":null,"format_metadata":{"kind":"spice","ngspice_validates":true}}"#;
    let duplicate_error = run_project_command(&[
        "eda",
        "--format",
        "json",
        "project",
        "set-pool-part-behavioural-models",
        root.to_str().unwrap(),
        "--part",
        &part_id.to_string(),
        "--model",
        valid_attachment,
        "--model",
        valid_attachment,
    ])
    .expect_err("duplicate behavioural model should fail");
    assert!(format!("{duplicate_error:#}").contains("duplicate part behavioural model attachment"));
    let malformed_error = run_project_command(&[
        "eda",
        "--format",
        "json",
        "project",
        "set-pool-part-behavioural-models",
        root.to_str().unwrap(),
        "--part",
        &part_id.to_string(),
        "--model",
        "spice",
    ])
    .expect_err("malformed behavioural model should fail");
    assert!(
        format!("{malformed_error:#}")
            .contains("part behavioural model attachment must be a JSON object")
    );
    let schema_error = run_project_command(&[
        "eda",
        "--format",
        "json",
        "project",
        "set-pool-part-behavioural-models",
        root.to_str().unwrap(),
        "--part",
        &part_id.to_string(),
        "--model",
        r#"{"role":"Spice"}"#,
    ])
    .expect_err("schema-invalid behavioural model should fail");
    assert!(
        format!("{schema_error:#}")
            .contains("part behavioural model attachment must match ModelAttachment schema")
    );
    let encrypted_error = run_project_command(&[
        "eda",
        "--format",
        "json",
        "project",
        "set-pool-part-behavioural-models",
        root.to_str().unwrap(),
        "--part",
        &part_id.to_string(),
        "--model",
        r#"{"uuid":"11111111-1111-1111-1111-111111111111","model_uuid":"22222222-2222-2222-2222-222222222222","role":"Spice","dialect":"Ngspice","model_names":["OPA1656"],"encrypted":false,"encryption_scheme":"PSpiceEncryptIt","provenance":null,"format_metadata":{"kind":"spice","ngspice_validates":true}}"#,
    ])
    .expect_err("encryption-scheme without encrypted flag should fail");
    assert!(format!("{encrypted_error:#}").contains("encryption scheme requires encrypted=true"));
    let mode_error = run_project_command(&[
        "eda",
        "--format",
        "json",
        "project",
        "set-pool-part-behavioural-models",
        root.to_str().unwrap(),
        "--part",
        &part_id.to_string(),
        "--mode",
        "append",
        "--model",
        valid_attachment,
    ])
    .expect_err("invalid behavioural model mode should fail");
    assert!(format!("{mode_error:#}").contains("unsupported part behavioural model mode append"));
    assert!(
        query_pool_object_payload(&root, "parts", part_id)["behavioural_models"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_attach_pool_part_model_promotes_file_and_attaches_model() {
    let root = unique_project_root("datum-eda-cli-project-pool-part-attach-model");
    create_native_project(&root, Some("Pool Part Attach Model".to_string()))
        .expect("initial scaffold should succeed");
    let part_id = create_minimal_part(&root);
    let source_dir = root.join("vendor");
    std::fs::create_dir_all(&source_dir).expect("vendor dir should be created");
    let source = source_dir.join("opa1656.lib");
    std::fs::write(&source, b".subckt OPA1656 IN OUT VCC VEE\n.ends\n")
        .expect("model fixture should be written");
    let output = run_project_command(&[
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
        "OPA1656",
        "--vendor",
        "Texas Instruments",
        "--format-metadata-json",
        r#"{"kind":"spice","ngspice_validates":null}"#,
    ])
    .expect("model attach should succeed");
    let report: serde_json::Value =
        serde_json::from_str(&output).expect("attach model report JSON should parse");
    assert_eq!(report["action"], "attach_part_model");
    let promoted_files: Vec<_> = std::fs::read_dir(root.join("pool/models/spice"))
        .expect("spice model directory should exist")
        .map(|entry| entry.expect("model dir entry should read").path())
        .collect();
    assert_eq!(promoted_files.len(), 1);
    assert_eq!(
        std::fs::read(&promoted_files[0]).expect("promoted model should read"),
        b".subckt OPA1656 IN OUT VCC VEE\n.ends\n"
    );
    assert_eq!(
        promoted_files[0].extension().and_then(|ext| ext.to_str()),
        Some("lib")
    );
    let payload = query_pool_object_payload(&root, "parts", part_id);
    assert_eq!(payload["behavioural_models"].as_array().unwrap().len(), 1);
    assert!(
        transaction_operation_kinds(&root)
            .iter()
            .any(|kind| kind == "attach_pool_part_model")
    );
    assert_eq!(payload["behavioural_models"][0]["role"], "Spice");
    assert_eq!(payload["behavioural_models"][0]["dialect"], "Ngspice");
    assert_eq!(
        payload["behavioural_models"][0]["model_names"][0],
        "OPA1656"
    );
    assert_eq!(
        payload["behavioural_models"][0]["provenance"]["vendor"],
        "Texas Instruments"
    );
    assert_eq!(
        payload["behavioural_models"][0]["format_metadata"]["kind"],
        "spice"
    );
    let attachment_id = payload["behavioural_models"][0]["uuid"]
        .as_str()
        .expect("attachment uuid should be string")
        .to_string();
    let model_id = payload["behavioural_models"][0]["model_uuid"]
        .as_str()
        .expect("model uuid should be string")
        .to_string();
    let sha256 = payload["behavioural_models"][0]["provenance"]["sha256"]
        .as_str()
        .expect("model sha256 should be string")
        .to_string();
    let models = query_pool_models(&root, &[]);
    assert_eq!(models["contract"], "native_project_pool_models_query_v1");
    assert_eq!(models["model_count"], 1);
    assert_eq!(models["models"][0]["role"], "spice");
    assert_eq!(models["models"][0]["sha256"], sha256);
    assert_eq!(models["models"][0]["computed_sha256"], sha256);
    assert_eq!(models["models"][0]["hash_matches"], true);
    assert_eq!(models["models"][0]["model_uuid"], model_id);
    assert_eq!(models["models"][0]["referenced"], true);
    assert_eq!(models["models"][0]["orphaned"], false);
    assert_eq!(models["models"][0]["attachment_count"], 1);
    assert_eq!(
        models["models"][0]["attachments"][0]["part_uuid"],
        part_id.to_string()
    );
    assert_eq!(
        models["models"][0]["attachments"][0]["attachment_uuid"],
        attachment_id
    );
    assert!(
        models["models"][0]["relative_path"]
            .as_str()
            .expect("relative path should be string")
            .starts_with("pool/models/spice/")
    );
    let filtered = query_pool_models(&root, &["--role", "spice", "--sha256", &sha256]);
    assert_eq!(filtered["model_count"], 1);
    let output = run_project_command(&[
        "eda",
        "--format",
        "json",
        "project",
        "detach-pool-part-model",
        root.to_str().unwrap(),
        "--part",
        &part_id.to_string(),
        "--attachment",
        &attachment_id,
    ])
    .expect("model detach by attachment should succeed");
    let report: serde_json::Value =
        serde_json::from_str(&output).expect("detach model report JSON should parse");
    assert_eq!(report["action"], "detach_part_model");
    let payload = query_pool_object_payload(&root, "parts", part_id);
    assert!(payload["behavioural_models"].as_array().unwrap().is_empty());
    assert!(
        transaction_operation_kinds(&root)
            .iter()
            .any(|kind| kind == "detach_pool_part_model")
    );
    assert!(promoted_files[0].exists());
    let models = query_pool_models(&root, &["--role", "spice", "--sha256", &sha256]);
    assert_eq!(models["model_count"], 1);
    assert_eq!(models["models"][0]["referenced"], false);
    assert_eq!(models["models"][0]["orphaned"], true);
    assert_eq!(models["models"][0]["attachment_count"], 0);
    run_project_command(&[
        "eda",
        "--format",
        "json",
        "project",
        "undo",
        root.to_str().unwrap(),
    ])
    .expect("detach model undo should succeed");
    let payload = query_pool_object_payload(&root, "parts", part_id);
    assert_eq!(payload["behavioural_models"].as_array().unwrap().len(), 1);
    let models = query_pool_models(&root, &["--role", "spice", "--sha256", &sha256]);
    assert_eq!(models["models"][0]["referenced"], true);
    assert_eq!(models["models"][0]["orphaned"], false);
    assert_eq!(models["models"][0]["attachment_count"], 1);
    run_project_command(&[
        "eda",
        "--format",
        "json",
        "project",
        "detach-pool-part-model",
        root.to_str().unwrap(),
        "--part",
        &part_id.to_string(),
        "--model",
        &model_id,
    ])
    .expect("model detach by model uuid should succeed");
    let payload = query_pool_object_payload(&root, "parts", part_id);
    assert!(payload["behavioural_models"].as_array().unwrap().is_empty());
    assert!(promoted_files[0].exists());
    run_project_command(&[
        "eda",
        "--format",
        "json",
        "project",
        "undo",
        root.to_str().unwrap(),
    ])
    .expect("detach by model undo should succeed");
    let payload = query_pool_object_payload(&root, "parts", part_id);
    assert_eq!(payload["behavioural_models"].as_array().unwrap().len(), 1);
    assert!(promoted_files[0].exists());
    run_project_command(&[
        "eda",
        "--format",
        "json",
        "project",
        "detach-pool-part-model",
        root.to_str().unwrap(),
        "--part",
        &part_id.to_string(),
        "--attachment",
        &attachment_id,
    ])
    .expect("model detach before GC should succeed");
    let dry_run = run_project_command(&[
        "eda",
        "--format",
        "json",
        "project",
        "gc-pool-models",
        root.to_str().unwrap(),
        "--role",
        "spice",
        "--sha256",
        &sha256,
    ])
    .expect("model GC dry-run should succeed");
    let dry_run: serde_json::Value =
        serde_json::from_str(&dry_run).expect("GC dry-run report should parse");
    assert_eq!(dry_run["contract"], "native_project_pool_model_gc_v1");
    assert_eq!(dry_run["applied"], false);
    assert_eq!(dry_run["planned_count"], 1);
    assert_eq!(dry_run["deleted_count"], 0);
    assert!(promoted_files[0].exists());
    let applied = run_project_command(&[
        "eda",
        "--format",
        "json",
        "project",
        "gc-pool-models",
        root.to_str().unwrap(),
        "--role",
        "spice",
        "--sha256",
        &sha256,
        "--apply",
    ])
    .expect("model GC apply should succeed");
    let applied: serde_json::Value =
        serde_json::from_str(&applied).expect("GC apply report should parse");
    assert_eq!(applied["applied"], true);
    assert_eq!(applied["planned_count"], 1);
    assert_eq!(applied["deleted_count"], 1);
    assert_eq!(applied["entries"][0]["deleted"], true);
    assert!(!promoted_files[0].exists());
    let models = query_pool_models(&root, &["--role", "spice", "--sha256", &sha256]);
    assert_eq!(models["model_count"], 0);
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_attach_pool_part_model_rejects_invalid_requests() {
    let root = unique_project_root("datum-eda-cli-project-pool-part-attach-model-invalid");
    create_native_project(&root, Some("Pool Part Attach Model Invalid".to_string()))
        .expect("initial scaffold should succeed");
    let part_id = create_minimal_part(&root);
    let source = root.join("vendor").join("bad.lib");
    let missing_source_error = run_project_command(&[
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
    ])
    .expect_err("missing source should fail");
    assert!(format!("{missing_source_error:#}").contains("model source file does not exist"));
    std::fs::create_dir_all(source.parent().unwrap()).expect("vendor dir should be created");
    std::fs::write(&source, b".subckt BAD\n.ends\n").expect("model fixture should be written");
    let role_error = run_project_command(&[
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
        "UnknownRole",
    ])
    .expect_err("bad role should fail");
    assert!(format!("{role_error:#}").contains("role is not a supported enum value"));
    let duplicate_name_error = run_project_command(&[
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
        "--model-name",
        "BAD",
        "--model-name",
        "BAD",
    ])
    .expect_err("duplicate model names should fail");
    assert!(format!("{duplicate_name_error:#}").contains("duplicate model-name BAD"));
    let no_selector_error = run_project_command(&[
        "eda",
        "--format",
        "json",
        "project",
        "detach-pool-part-model",
        root.to_str().unwrap(),
        "--part",
        &part_id.to_string(),
    ])
    .expect_err("detach without selector should fail");
    assert!(format!("{no_selector_error:#}").contains("requires --attachment or --model"));
    let both_selector_error = run_project_command(&[
        "eda",
        "--format",
        "json",
        "project",
        "detach-pool-part-model",
        root.to_str().unwrap(),
        "--part",
        &part_id.to_string(),
        "--attachment",
        &Uuid::new_v4().to_string(),
        "--model",
        &Uuid::new_v4().to_string(),
    ])
    .expect_err("detach with both selectors should fail");
    assert!(format!("{both_selector_error:#}").contains("accepts --attachment or --model"));
    let missing_match_error = run_project_command(&[
        "eda",
        "--format",
        "json",
        "project",
        "detach-pool-part-model",
        root.to_str().unwrap(),
        "--part",
        &part_id.to_string(),
        "--attachment",
        &Uuid::new_v4().to_string(),
    ])
    .expect_err("detach without matching attachment should fail");
    assert!(format!("{missing_match_error:#}").contains("has no matching behavioural model"));
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_set_pool_part_thermal_updates_clears_and_undoes() {
    let root = unique_project_root("datum-eda-cli-project-pool-part-thermal");
    create_native_project(&root, Some("Pool Part Thermal".to_string()))
        .expect("initial scaffold should succeed");
    let part_id = create_minimal_part(&root);
    let output = run_project_command(&[
        "eda",
        "--format",
        "json",
        "project",
        "set-pool-part-thermal",
        root.to_str().unwrap(),
        "--part",
        &part_id.to_string(),
        "--theta-ja-c-per-w",
        "62.5",
        "--theta-jc-top-c-per-w",
        "12",
        "--max-junction-c",
        "150",
        "--thermal-reference",
        "JESD51-2 still-air 1S board",
    ])
    .expect("thermal update should succeed");
    let report: serde_json::Value =
        serde_json::from_str(&output).expect("thermal report JSON should parse");
    assert_eq!(report["action"], "set_part_thermal");
    let payload = query_pool_object_payload(&root, "parts", part_id);
    assert_eq!(payload["thermal"]["theta_ja_c_per_w"], 62.5);
    assert_eq!(payload["thermal"]["theta_jc_top_c_per_w"], 12);
    assert_eq!(payload["thermal"]["max_junction_c"], 150);
    assert_eq!(
        payload["thermal"]["thermal_reference"],
        "JESD51-2 still-air 1S board"
    );
    run_project_command(&[
        "eda",
        "--format",
        "json",
        "project",
        "set-pool-part-thermal",
        root.to_str().unwrap(),
        "--part",
        &part_id.to_string(),
        "--theta-jb-c-per-w",
        "18.25",
    ])
    .expect("thermal partial update should succeed");
    let payload = query_pool_object_payload(&root, "parts", part_id);
    assert_eq!(payload["thermal"]["theta_ja_c_per_w"], 62.5);
    assert_eq!(payload["thermal"]["theta_jb_c_per_w"], 18.25);
    run_project_command(&[
        "eda",
        "--format",
        "json",
        "project",
        "set-pool-part-thermal",
        root.to_str().unwrap(),
        "--part",
        &part_id.to_string(),
        "--clear",
    ])
    .expect("thermal clear should succeed");
    assert!(query_pool_object_payload(&root, "parts", part_id)["thermal"].is_null());
    run_project_command(&[
        "eda",
        "--format",
        "json",
        "project",
        "undo",
        root.to_str().unwrap(),
    ])
    .expect("thermal clear undo should succeed");
    let payload = query_pool_object_payload(&root, "parts", part_id);
    assert_eq!(payload["thermal"]["theta_ja_c_per_w"], 62.5);
    assert_eq!(payload["thermal"]["theta_jb_c_per_w"], 18.25);
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_set_pool_part_thermal_rejects_invalid_requests() {
    let root = unique_project_root("datum-eda-cli-project-pool-part-thermal-invalid");
    create_native_project(&root, Some("Pool Part Thermal Invalid".to_string()))
        .expect("initial scaffold should succeed");
    let part_id = create_minimal_part(&root);
    let no_field_error = run_project_command(&[
        "eda",
        "--format",
        "json",
        "project",
        "set-pool-part-thermal",
        root.to_str().unwrap(),
        "--part",
        &part_id.to_string(),
    ])
    .expect_err("thermal update without fields should fail");
    assert!(
        format!("{no_field_error:#}").contains("requires --clear or at least one thermal field")
    );
    let negative_error = run_project_command(&[
        "eda",
        "--format",
        "json",
        "project",
        "set-pool-part-thermal",
        root.to_str().unwrap(),
        "--part",
        &part_id.to_string(),
        "--theta-ja-c-per-w",
        "-1",
    ])
    .expect_err("negative thermal value should fail");
    assert!(
        format!("{negative_error:#}")
            .contains("theta-ja-c-per-w must be a non-negative JSON number")
    );
    let malformed_error = run_project_command(&[
        "eda",
        "--format",
        "json",
        "project",
        "set-pool-part-thermal",
        root.to_str().unwrap(),
        "--part",
        &part_id.to_string(),
        "--theta-ja-c-per-w",
        "nan",
    ])
    .expect_err("malformed thermal value should fail");
    assert!(
        format!("{malformed_error:#}")
            .contains("theta-ja-c-per-w must be a non-negative JSON number")
    );
    let blank_reference_error = run_project_command(&[
        "eda",
        "--format",
        "json",
        "project",
        "set-pool-part-thermal",
        root.to_str().unwrap(),
        "--part",
        &part_id.to_string(),
        "--thermal-reference",
        "   ",
    ])
    .expect_err("blank thermal reference should fail");
    assert!(format!("{blank_reference_error:#}").contains("thermal-reference must be non-empty"));
    assert!(query_pool_object_payload(&root, "parts", part_id)["thermal"].is_null());
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_set_pool_part_supply_chain_updates_clears_and_undoes() {
    let root = unique_project_root("datum-eda-cli-project-pool-part-supply-chain");
    create_native_project(&root, Some("Pool Part Supply Chain".to_string()))
        .expect("initial scaffold should succeed");
    let part_id = create_minimal_part(&root);
    let digikey_offer = r#"{"distributor":"Digi-Key","price_breaks":[{"qty":1,"price":1.23,"currency":"USD"},{"qty":100,"price":0.98,"currency":"USD"}],"stock":1234,"lead_time_weeks":null,"link":"https://example.invalid/dk"}"#;
    let mouser_offer = r#"{"distributor":"Mouser","price_breaks":[{"qty":10,"price":1.11,"currency":"USD"}],"stock":500,"lead_time_weeks":2,"link":"https://example.invalid/mouser"}"#;
    let output = run_project_command(&[
        "eda",
        "--format",
        "json",
        "project",
        "set-pool-part-supply-chain",
        root.to_str().unwrap(),
        "--part",
        &part_id.to_string(),
        "--checked-at",
        "2026-06-22T12:00:00Z",
        "--offer",
        digikey_offer,
        "--offer",
        mouser_offer,
    ])
    .expect("supply-chain update should succeed");
    let report: serde_json::Value =
        serde_json::from_str(&output).expect("supply-chain report JSON should parse");
    assert_eq!(report["action"], "set_part_supply_chain");
    let payload = query_pool_object_payload(&root, "parts", part_id);
    assert_eq!(payload["supply_chain_offers"].as_array().unwrap().len(), 2);
    assert_eq!(payload["supply_chain_offers"][0]["distributor"], "Digi-Key");
    assert_eq!(payload["last_supply_chain_check"], "2026-06-22T12:00:00Z");
    run_project_command(&[
        "eda",
        "--format",
        "json",
        "project",
        "set-pool-part-supply-chain",
        root.to_str().unwrap(),
        "--part",
        &part_id.to_string(),
        "--clear",
    ])
    .expect("supply-chain clear should succeed");
    let payload = query_pool_object_payload(&root, "parts", part_id);
    assert!(payload["supply_chain_offers"].is_null());
    assert!(payload["last_supply_chain_check"].is_null());
    run_project_command(&[
        "eda",
        "--format",
        "json",
        "project",
        "undo",
        root.to_str().unwrap(),
    ])
    .expect("supply-chain clear undo should succeed");
    let payload = query_pool_object_payload(&root, "parts", part_id);
    assert_eq!(payload["supply_chain_offers"].as_array().unwrap().len(), 2);
    assert_eq!(payload["last_supply_chain_check"], "2026-06-22T12:00:00Z");
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_set_pool_part_supply_chain_rejects_invalid_requests() {
    let root = unique_project_root("datum-eda-cli-project-pool-part-supply-chain-invalid");
    create_native_project(&root, Some("Pool Part Supply Chain Invalid".to_string()))
        .expect("initial scaffold should succeed");
    let part_id = create_minimal_part(&root);
    let no_field_error = run_project_command(&[
        "eda",
        "--format",
        "json",
        "project",
        "set-pool-part-supply-chain",
        root.to_str().unwrap(),
        "--part",
        &part_id.to_string(),
    ])
    .expect_err("supply-chain update without fields should fail");
    assert!(format!("{no_field_error:#}").contains("requires --clear"));
    let malformed_error = run_project_command(&[
        "eda",
        "--format",
        "json",
        "project",
        "set-pool-part-supply-chain",
        root.to_str().unwrap(),
        "--part",
        &part_id.to_string(),
        "--offer",
        "digikey",
    ])
    .expect_err("malformed supply-chain offer should fail");
    assert!(format!("{malformed_error:#}").contains("supply-chain offer must be a JSON object"));
    let schema_error = run_project_command(&[
        "eda",
        "--format",
        "json",
        "project",
        "set-pool-part-supply-chain",
        root.to_str().unwrap(),
        "--part",
        &part_id.to_string(),
        "--offer",
        r#"{"distributor":"Digi-Key"}"#,
    ])
    .expect_err("schema-invalid supply-chain offer should fail");
    assert!(format!("{schema_error:#}").contains("SupplyOffer schema"));
    let quality_error = run_project_command(&[
        "eda",
        "--format",
        "json",
        "project",
        "set-pool-part-supply-chain",
        root.to_str().unwrap(),
        "--part",
        &part_id.to_string(),
        "--offer",
        r#"{"distributor":"","price_breaks":[{"qty":0,"price":-1,"currency":""}],"stock":null,"lead_time_weeks":null,"link":""}"#,
    ])
    .expect_err("quality-invalid supply-chain offer should fail");
    assert!(format!("{quality_error:#}").contains("distributor must be non-empty"));
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_set_pool_part_tags_merges_replaces_and_undoes() {
    let root = unique_project_root("datum-eda-cli-project-pool-part-tags");
    create_native_project(&root, Some("Pool Part Tags".to_string()))
        .expect("initial scaffold should succeed");
    let part_id = create_minimal_part(&root);
    let output = run_project_command(&[
        "eda",
        "--format",
        "json",
        "project",
        "set-pool-part-tags",
        root.to_str().unwrap(),
        "--part",
        &part_id.to_string(),
        "--tag",
        "audio",
        "--tag",
        "low-noise",
    ])
    .expect("part tags merge should succeed");
    let report: serde_json::Value =
        serde_json::from_str(&output).expect("part tags report JSON should parse");
    assert_eq!(report["action"], "set_part_tags");
    let payload = query_pool_object_payload(&root, "parts", part_id);
    assert_eq!(payload["tags"], serde_json::json!(["audio", "low-noise"]));
    run_project_command(&[
        "eda",
        "--format",
        "json",
        "project",
        "set-pool-part-tags",
        root.to_str().unwrap(),
        "--part",
        &part_id.to_string(),
        "--mode",
        "replace",
        "--tag",
        "precision",
    ])
    .expect("part tags replace should succeed");
    let payload = query_pool_object_payload(&root, "parts", part_id);
    assert_eq!(payload["tags"], serde_json::json!(["precision"]));
    run_project_command(&[
        "eda",
        "--format",
        "json",
        "project",
        "undo",
        root.to_str().unwrap(),
    ])
    .expect("part tags replace undo should succeed");
    let payload = query_pool_object_payload(&root, "parts", part_id);
    assert_eq!(payload["tags"], serde_json::json!(["audio", "low-noise"]));
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_set_pool_part_tags_rejects_invalid_requests() {
    let root = unique_project_root("datum-eda-cli-project-pool-part-tags-invalid");
    create_native_project(&root, Some("Pool Part Tags Invalid".to_string()))
        .expect("initial scaffold should succeed");
    let part_id = create_minimal_part(&root);
    let duplicate_error = run_project_command(&[
        "eda",
        "--format",
        "json",
        "project",
        "set-pool-part-tags",
        root.to_str().unwrap(),
        "--part",
        &part_id.to_string(),
        "--tag",
        "audio",
        "--tag",
        "audio",
    ])
    .expect_err("duplicate tag should fail");
    assert!(format!("{duplicate_error:#}").contains("duplicate part tag audio"));
    let blank_error = run_project_command(&[
        "eda",
        "--format",
        "json",
        "project",
        "set-pool-part-tags",
        root.to_str().unwrap(),
        "--part",
        &part_id.to_string(),
        "--tag",
        " ",
    ])
    .expect_err("blank tag should fail");
    assert!(format!("{blank_error:#}").contains("part tag must be non-empty"));
    let mode_error = run_project_command(&[
        "eda",
        "--format",
        "json",
        "project",
        "set-pool-part-tags",
        root.to_str().unwrap(),
        "--part",
        &part_id.to_string(),
        "--mode",
        "append",
        "--tag",
        "audio",
    ])
    .expect_err("unsupported mode should fail");
    assert!(format!("{mode_error:#}").contains("expected merge or replace"));
    assert_eq!(
        query_pool_object_payload(&root, "parts", part_id)["tags"],
        serde_json::json!([])
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_set_pool_part_pad_map_entry_maps_package_pad_to_entity_gate_pin() {
    let root = unique_project_root("datum-eda-cli-project-pool-part-pad-map");
    create_native_project(&root, Some("Pool Part Pad Map".to_string()))
        .expect("initial scaffold should succeed");
    let fixture = create_pinned_part_fixture(&root, &["IN+"], &["1"]);
    let (part_id, pad_id, gate_id, pin_id) = (
        fixture.part_id,
        fixture.pad_ids[0],
        fixture.gate_id,
        fixture.pin_ids[0],
    );
    let output = run_project_command(&[
        "eda",
        "--format",
        "json",
        "project",
        "set-pool-part-pad-map-entry",
        root.to_str().unwrap(),
        "--part",
        &part_id.to_string(),
        "--pad",
        &pad_id.to_string(),
        "--gate",
        &gate_id.to_string(),
        "--pin",
        &pin_id.to_string(),
    ])
    .expect("pad map set should succeed");
    let report: serde_json::Value =
        serde_json::from_str(&output).expect("pad map report JSON should parse");
    assert_eq!(report["action"], "set_part_pad_map_entry");
    let payload = query_pool_object_payload(&root, "parts", part_id);
    assert_eq!(
        payload["pad_map"][pad_id.to_string()]["gate"],
        gate_id.to_string()
    );
    assert_eq!(
        payload["pad_map"][pad_id.to_string()]["pin"],
        pin_id.to_string()
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_set_pool_part_pad_map_entry_rejects_missing_pin() {
    let root = unique_project_root("datum-eda-cli-project-pool-part-pad-map-missing-pin");
    create_native_project(&root, Some("Pool Part Pad Map Missing Pin".to_string()))
        .expect("initial scaffold should succeed");
    let fixture = create_pinned_part_fixture(&root, &["IN+"], &["1"]);
    let (part_id, pad_id, gate_id) = (fixture.part_id, fixture.pad_ids[0], fixture.gate_id);
    let missing_pin = Uuid::new_v4();
    let error = run_project_command(&[
        "eda",
        "--format",
        "json",
        "project",
        "set-pool-part-pad-map-entry",
        root.to_str().unwrap(),
        "--part",
        &part_id.to_string(),
        "--pad",
        &pad_id.to_string(),
        "--gate",
        &gate_id.to_string(),
        "--pin",
        &missing_pin.to_string(),
    ])
    .expect_err("missing pin should fail");
    assert!(format!("{error:#}").contains("has no pin"));
    let payload = query_pool_object_payload(&root, "parts", part_id);
    assert_eq!(
        payload["pad_map"]
            .as_object()
            .expect("pad_map should be object")
            .len(),
        0
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_set_pool_part_pad_map_replace_maps_multiple_entries() {
    let root = unique_project_root("datum-eda-cli-project-pool-part-pad-map-replace");
    create_native_project(&root, Some("Pool Part Pad Map Replace".to_string()))
        .expect("initial scaffold should succeed");
    let fixture = create_pinned_part_fixture(&root, &["IN+", "OUT"], &["1", "2"]);
    let (part_id, gate_id, pin_a_id, pin_b_id, pad_a_id, pad_b_id) = (
        fixture.part_id,
        fixture.gate_id,
        fixture.pin_ids[0],
        fixture.pin_ids[1],
        fixture.pad_ids[0],
        fixture.pad_ids[1],
    );
    run_project_command(&[
        "eda",
        "--format",
        "json",
        "project",
        "set-pool-part-pad-map-entry",
        root.to_str().unwrap(),
        "--part",
        &part_id.to_string(),
        "--pad",
        &pad_a_id.to_string(),
        "--gate",
        &gate_id.to_string(),
        "--pin",
        &pin_a_id.to_string(),
    ])
    .expect("initial pad map set should succeed");
    let output = run_project_command(&[
        "eda",
        "--format",
        "json",
        "project",
        "set-pool-part-pad-map",
        root.to_str().unwrap(),
        "--part",
        &part_id.to_string(),
        "--mode",
        "replace",
        "--entry",
        &format!("{pad_b_id}:{gate_id}:{pin_b_id}"),
    ])
    .expect("bulk pad map replace should succeed");
    let report: serde_json::Value =
        serde_json::from_str(&output).expect("pad map report JSON should parse");
    assert_eq!(report["action"], "set_part_pad_map");
    let payload = query_pool_object_payload(&root, "parts", part_id);
    let pad_map = payload["pad_map"]
        .as_object()
        .expect("pad_map should be object");
    assert_eq!(pad_map.len(), 1);
    assert!(pad_map.get(&pad_a_id.to_string()).is_none());
    assert_eq!(
        payload["pad_map"][pad_b_id.to_string()]["gate"],
        gate_id.to_string()
    );
    assert_eq!(
        payload["pad_map"][pad_b_id.to_string()]["pin"],
        pin_b_id.to_string()
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_set_pool_part_pad_map_rejects_duplicate_entries() {
    let root = unique_project_root("datum-eda-cli-project-pool-part-pad-map-duplicate");
    create_native_project(&root, Some("Pool Part Pad Map Duplicate".to_string()))
        .expect("initial scaffold should succeed");
    let fixture = create_pinned_part_fixture(&root, &["IN+"], &["1"]);
    let (part_id, pad_id, gate_id, pin_id) = (
        fixture.part_id,
        fixture.pad_ids[0],
        fixture.gate_id,
        fixture.pin_ids[0],
    );
    let entry = format!("{pad_id}:{gate_id}:{pin_id}");
    let error = run_project_command(&[
        "eda",
        "--format",
        "json",
        "project",
        "set-pool-part-pad-map",
        root.to_str().unwrap(),
        "--part",
        &part_id.to_string(),
        "--entry",
        &entry,
        "--entry",
        &entry,
    ])
    .expect_err("duplicate entries should fail");
    assert!(format!("{error:#}").contains("duplicate pad-map entry"));
    let payload = query_pool_object_payload(&root, "parts", part_id);
    assert_eq!(
        payload["pad_map"]
            .as_object()
            .expect("pad_map should be object")
            .len(),
        0
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_set_pool_unit_pin_authors_typed_pin() {
    let root = unique_project_root("datum-eda-cli-project-pool-unit-pin");
    create_native_project(&root, Some("Pool Unit Pin".to_string()))
        .expect("initial scaffold should succeed");
    let unit_id = Uuid::new_v4();
    let pin_id = Uuid::new_v4();
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
        "OpAmpUnit",
    ])
    .expect("unit create should succeed");
    let output = run_project_command(&[
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
        "OUT",
        "--direction",
        "Output",
        "--swap-group",
        "1",
    ])
    .expect("unit pin set should succeed");
    let report: serde_json::Value =
        serde_json::from_str(&output).expect("pin report JSON should parse");
    assert_eq!(report["action"], "set_unit_pin");
    assert_eq!(report["object_uuid"], unit_id.to_string());
    let payload = query_pool_object_payload(&root, "units", unit_id);
    assert_eq!(
        payload["pins"][pin_id.to_string()]["uuid"],
        pin_id.to_string()
    );
    assert_eq!(payload["pins"][pin_id.to_string()]["name"], "OUT");
    assert_eq!(payload["pins"][pin_id.to_string()]["direction"], "Output");
    assert_eq!(payload["pins"][pin_id.to_string()]["swap_group"], 1);
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_set_pool_unit_pin_rejects_duplicate_pin() {
    let root = unique_project_root("datum-eda-cli-project-pool-unit-pin-duplicate");
    create_native_project(&root, Some("Pool Unit Pin Duplicate".to_string()))
        .expect("initial scaffold should succeed");
    let unit_id = Uuid::new_v4();
    let pin_id = Uuid::new_v4();
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
        "OpAmpUnit",
    ])
    .expect("unit create should succeed");
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
        "OUT",
    ])
    .expect("first unit pin set should succeed");
    let error = run_project_command(&[
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
        "OUT2",
    ])
    .expect_err("duplicate pin should fail");
    assert!(format!("{error:#}").contains("already has pin"));
    let payload = query_pool_object_payload(&root, "units", unit_id);
    assert_eq!(
        payload["pins"]
            .as_object()
            .expect("pins should be object")
            .len(),
        1
    );
    assert_eq!(payload["pins"][pin_id.to_string()]["name"], "OUT");
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_set_pool_unit_pin_rejects_invalid_direction() {
    let root = unique_project_root("datum-eda-cli-project-pool-unit-pin-invalid-direction");
    create_native_project(&root, Some("Pool Unit Pin Invalid Direction".to_string()))
        .expect("initial scaffold should succeed");
    let unit_id = Uuid::new_v4();
    let pin_id = Uuid::new_v4();
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
        "OpAmpUnit",
    ])
    .expect("unit create should succeed");
    let error = run_project_command(&[
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
        "OUT",
        "--direction",
        "Driver",
    ])
    .expect_err("invalid direction should fail");
    assert!(format!("{error:#}").contains("unsupported pin direction"));
    let payload = query_pool_object_payload(&root, "units", unit_id);
    assert_eq!(
        payload["pins"]
            .as_object()
            .expect("pins should be object")
            .len(),
        0
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_set_pool_package_pad_authors_additional_pad() {
    let root = unique_project_root("datum-eda-cli-project-pool-package-pad");
    create_native_project(&root, Some("Pool Package Pad".to_string()))
        .expect("initial scaffold should succeed");
    let (padstack_id, package_id, _) = create_minimal_package(&root, "1");
    let second_pad_id = Uuid::new_v4();
    let output = run_project_command(&[
        "eda",
        "--format",
        "json",
        "project",
        "set-pool-package-pad",
        root.to_str().unwrap(),
        "--package",
        &package_id.to_string(),
        "--pad",
        &second_pad_id.to_string(),
        "--padstack",
        &padstack_id.to_string(),
        "--pad-name",
        "2",
        "--x-nm",
        "1000",
        "--y-nm",
        "2000",
        "--layer",
        "1",
    ])
    .expect("package pad set should succeed");
    let report: serde_json::Value =
        serde_json::from_str(&output).expect("package pad report JSON should parse");
    assert_eq!(report["action"], "set_package_pad");
    assert_eq!(report["object_uuid"], package_id.to_string());
    let payload = query_pool_object_payload(&root, "packages", package_id);
    assert_eq!(
        payload["pads"]
            .as_object()
            .expect("pads should be object")
            .len(),
        2
    );
    assert_eq!(payload["pads"][second_pad_id.to_string()]["name"], "2");
    assert_eq!(
        payload["pads"][second_pad_id.to_string()]["position"]["x"],
        1000
    );
    assert_eq!(
        payload["pads"][second_pad_id.to_string()]["position"]["y"],
        2000
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_set_pool_package_pad_rejects_duplicate_pad() {
    let root = unique_project_root("datum-eda-cli-project-pool-package-pad-duplicate");
    create_native_project(&root, Some("Pool Package Pad Duplicate".to_string()))
        .expect("initial scaffold should succeed");
    let (padstack_id, package_id, pad_id) = create_minimal_package(&root, "1");
    let error = run_project_command(&[
        "eda",
        "--format",
        "json",
        "project",
        "set-pool-package-pad",
        root.to_str().unwrap(),
        "--package",
        &package_id.to_string(),
        "--pad",
        &pad_id.to_string(),
        "--padstack",
        &padstack_id.to_string(),
        "--pad-name",
        "1B",
    ])
    .expect_err("duplicate package pad should fail");
    assert!(format!("{error:#}").contains("already has pad"));
    let payload = query_pool_object_payload(&root, "packages", package_id);
    assert_eq!(
        payload["pads"]
            .as_object()
            .expect("pads should be object")
            .len(),
        1
    );
    assert_eq!(payload["pads"][pad_id.to_string()]["name"], "1");
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_set_pool_package_pad_rejects_invalid_layer() {
    let root = unique_project_root("datum-eda-cli-project-pool-package-pad-invalid-layer");
    create_native_project(&root, Some("Pool Package Pad Invalid Layer".to_string()))
        .expect("initial scaffold should succeed");
    let (padstack_id, package_id, _) = create_minimal_package(&root, "1");
    let second_pad_id = Uuid::new_v4();
    let error = run_project_command(&[
        "eda",
        "--format",
        "json",
        "project",
        "set-pool-package-pad",
        root.to_str().unwrap(),
        "--package",
        &package_id.to_string(),
        "--pad",
        &second_pad_id.to_string(),
        "--padstack",
        &padstack_id.to_string(),
        "--pad-name",
        "2",
        "--layer",
        "0",
    ])
    .expect_err("invalid package pad layer should fail");
    assert!(format!("{error:#}").contains("layer must be positive"));
    let payload = query_pool_object_payload(&root, "packages", package_id);
    assert_eq!(
        payload["pads"]
            .as_object()
            .expect("pads should be object")
            .len(),
        1
    );
    let _ = std::fs::remove_dir_all(&root);
}
