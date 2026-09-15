use super::*;

fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}", label, Uuid::new_v4()))
}

fn write_symbol_payload(root: &Path, symbol_id: Uuid) -> PathBuf {
    let path = root.join("symbol.json");
    std::fs::write(
        &path,
        serde_json::to_string_pretty(&serde_json::json!({
            "schema_version": 1,
            "uuid": symbol_id,
            "name": "CliNativeSymbol",
            "unit": Uuid::new_v4()
        }))
        .expect("symbol payload should serialize"),
    )
    .expect("symbol payload should write");
    path
}

fn query_pool_object_payload(root: &Path, kind: &str, object_id: Uuid) -> serde_json::Value {
    let object = object_id.to_string();
    let output = execute(
        Cli::try_parse_from([
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
            &object,
            "--include-payload",
        ])
        .expect("CLI should parse"),
    )
    .expect("pool object query should succeed");
    let report: serde_json::Value =
        serde_json::from_str(&output).expect("query report JSON should parse");
    assert_eq!(report["object_count"], 1);
    report["objects"][0]["payload"].clone()
}

fn seed_promoted_pool_unit(root: &Path, unit_id: Uuid) {
    let project_json = root.join("project.json");
    let mut project: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&project_json).expect("project manifest should read"),
    )
    .expect("project manifest JSON should parse");
    project["pools"] = serde_json::json!([{ "path": "pool", "priority": 1 }]);
    std::fs::write(
        &project_json,
        serde_json::to_string_pretty(&project).expect("project manifest should serialize"),
    )
    .expect("project manifest should write");

    let unit_path = root.join(format!("pool/units/{unit_id}.json"));
    std::fs::create_dir_all(unit_path.parent().expect("unit path should have parent"))
        .expect("pool units directory should be created");
    std::fs::write(
        &unit_path,
        serde_json::to_string_pretty(&serde_json::json!({
            "schema_version": 1,
            "uuid": unit_id,
            "name": "LMV358Unit",
            "manufacturer": "",
            "pins": {},
            "tags": []
        }))
        .expect("unit payload should serialize"),
    )
    .expect("unit payload should write");
}

fn seed_promoted_pool_symbol(root: &Path, symbol_id: Uuid, unit_id: Uuid) {
    let symbol_path = root.join(format!("pool/symbols/{symbol_id}.json"));
    std::fs::create_dir_all(
        symbol_path
            .parent()
            .expect("symbol path should have parent"),
    )
    .expect("pool symbols directory should be created");
    std::fs::write(
        &symbol_path,
        serde_json::to_string_pretty(&serde_json::json!({
            "schema_version": 1,
            "uuid": symbol_id,
            "name": "LMV358Symbol",
            "unit": unit_id
        }))
        .expect("symbol payload should serialize"),
    )
    .expect("symbol payload should write");
}

#[test]
fn proposal_create_pool_library_object_defers_until_accept_apply() {
    let root = unique_project_root("datum-eda-cli-project-pool-library-proposal");
    create_native_project(&root, Some("Pool Library Proposal Demo".to_string()))
        .expect("initial scaffold should succeed");
    let symbol_id = Uuid::new_v4();
    let symbol_payload = write_symbol_payload(&root, symbol_id);
    let symbol_path = root.join(format!("pool/symbols/{symbol_id}.json"));
    let proposal_id = Uuid::new_v4();

    let create_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "proposal",
            "create-pool-library-object",
            root.to_str().unwrap(),
            "--kind",
            "symbols",
            "--object",
            &symbol_id.to_string(),
            "--from-json",
            symbol_payload.to_str().unwrap(),
            "--proposal",
            &proposal_id.to_string(),
            "--rationale",
            "review AI-authored library symbol",
        ])
        .expect("CLI should parse"),
    )
    .expect("pool library proposal create should succeed");
    let create_report: serde_json::Value =
        serde_json::from_str(&create_output).expect("create report JSON should parse");
    assert_eq!(create_report["contract"], "proposal_create_v1");
    assert_eq!(
        create_report["action"],
        "create_pool_library_object_proposal"
    );
    assert_eq!(create_report["proposal_id"], proposal_id.to_string());
    assert_eq!(create_report["proposal"]["source"], "tool");
    assert_eq!(create_report["proposal"]["status"], "draft");
    assert_eq!(
        create_report["proposal"]["batch"]["operations"]
            .as_array()
            .expect("proposal operations should be an array")
            .len(),
        2
    );
    assert!(
        !symbol_path.exists(),
        "proposal creation must not write the pool symbol shard"
    );

    execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "proposal",
            "accept-apply",
            root.to_str().unwrap(),
            "--proposal",
            &proposal_id.to_string(),
        ])
        .expect("CLI should parse"),
    )
    .expect("pool library proposal accept-apply should succeed");

    assert!(symbol_path.exists());
    let queried = query_pool_object_payload(&root, "symbols", symbol_id);
    assert_eq!(queried["uuid"], symbol_id.to_string());
}

#[test]
fn proposal_create_pool_unit_defers_until_accept_apply() {
    let root = unique_project_root("datum-eda-cli-project-pool-unit-proposal");
    create_native_project(&root, Some("Pool Unit Proposal Demo".to_string()))
        .expect("initial scaffold should succeed");
    let unit_id = Uuid::new_v4();
    let unit_path = root.join(format!("pool/units/{unit_id}.json"));
    let proposal_id = Uuid::new_v4();

    let create_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "proposal",
            "create-pool-unit",
            root.to_str().unwrap(),
            "--unit",
            &unit_id.to_string(),
            "--name",
            "LMV358Unit",
            "--manufacturer",
            "Datum Semi",
            "--proposal",
            &proposal_id.to_string(),
            "--rationale",
            "review AI-authored unit",
        ])
        .expect("CLI should parse"),
    )
    .expect("pool unit proposal create should succeed");
    let create_report: serde_json::Value =
        serde_json::from_str(&create_output).expect("create report JSON should parse");
    assert_eq!(create_report["contract"], "proposal_create_v1");
    assert_eq!(create_report["action"], "create_pool_unit_proposal");
    assert_eq!(create_report["proposal_id"], proposal_id.to_string());
    assert_eq!(create_report["proposal"]["source"], "tool");
    assert!(
        !unit_path.exists(),
        "proposal creation must not write the pool unit shard"
    );

    execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "proposal",
            "accept-apply",
            root.to_str().unwrap(),
            "--proposal",
            &proposal_id.to_string(),
        ])
        .expect("CLI should parse"),
    )
    .expect("pool unit proposal accept-apply should succeed");

    assert!(unit_path.exists());
    let queried = query_pool_object_payload(&root, "units", unit_id);
    assert_eq!(queried["uuid"], unit_id.to_string());
    assert_eq!(queried["name"], "LMV358Unit");
    assert_eq!(queried["manufacturer"], "Datum Semi");
}

#[test]
fn proposal_create_pool_symbol_defers_until_accept_apply() {
    let root = unique_project_root("datum-eda-cli-project-pool-symbol-proposal");
    create_native_project(&root, Some("Pool Symbol Proposal Demo".to_string()))
        .expect("initial scaffold should succeed");
    let unit_id = Uuid::new_v4();
    let symbol_id = Uuid::new_v4();
    let proposal_id = Uuid::new_v4();
    seed_promoted_pool_unit(&root, unit_id);

    let symbol_path = root.join(format!("pool/symbols/{symbol_id}.json"));
    let create_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "proposal",
            "create-pool-symbol",
            root.to_str().unwrap(),
            "--symbol",
            &symbol_id.to_string(),
            "--unit",
            &unit_id.to_string(),
            "--name",
            "LMV358Symbol",
            "--proposal",
            &proposal_id.to_string(),
            "--rationale",
            "review AI-authored symbol",
        ])
        .expect("CLI should parse"),
    )
    .expect("pool symbol proposal create should succeed");
    let create_report: serde_json::Value =
        serde_json::from_str(&create_output).expect("create report JSON should parse");
    assert_eq!(create_report["contract"], "proposal_create_v1");
    assert_eq!(create_report["action"], "create_pool_symbol_proposal");
    assert_eq!(create_report["proposal_id"], proposal_id.to_string());
    assert_eq!(create_report["proposal"]["source"], "tool");
    assert!(
        !symbol_path.exists(),
        "proposal creation must not write the pool symbol shard"
    );

    execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "proposal",
            "accept-apply",
            root.to_str().unwrap(),
            "--proposal",
            &proposal_id.to_string(),
        ])
        .expect("CLI should parse"),
    )
    .expect("pool symbol proposal accept-apply should succeed");

    assert!(symbol_path.exists());
    let queried = query_pool_object_payload(&root, "symbols", symbol_id);
    assert_eq!(queried["uuid"], symbol_id.to_string());
    assert_eq!(queried["name"], "LMV358Symbol");
    assert_eq!(queried["unit"], unit_id.to_string());
}

#[test]
fn proposal_create_pool_symbol_rejects_missing_unit() {
    let root = unique_project_root("datum-eda-cli-project-pool-symbol-proposal-missing-unit");
    create_native_project(&root, Some("Pool Symbol Proposal Missing Unit".to_string()))
        .expect("initial scaffold should succeed");
    let symbol_id = Uuid::new_v4();
    let unit_id = Uuid::new_v4();

    let error = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "proposal",
            "create-pool-symbol",
            root.to_str().unwrap(),
            "--symbol",
            &symbol_id.to_string(),
            "--unit",
            &unit_id.to_string(),
            "--name",
            "MissingUnitSymbol",
        ])
        .expect("CLI should parse"),
    )
    .expect_err("missing unit should fail proposal creation");
    assert!(format!("{error:#}").contains("missing pool unit"));
    assert!(!root.join(format!("pool/symbols/{symbol_id}.json")).exists());
}

#[test]
fn proposal_create_pool_entity_defers_until_accept_apply() {
    let root = unique_project_root("datum-eda-cli-project-pool-entity-proposal");
    create_native_project(&root, Some("Pool Entity Proposal Demo".to_string()))
        .expect("initial scaffold should succeed");
    let unit_id = Uuid::new_v4();
    let symbol_id = Uuid::new_v4();
    let entity_id = Uuid::new_v4();
    let gate_id = Uuid::new_v4();
    let proposal_id = Uuid::new_v4();
    seed_promoted_pool_unit(&root, unit_id);
    seed_promoted_pool_symbol(&root, symbol_id, unit_id);

    let entity_path = root.join(format!("pool/entities/{entity_id}.json"));
    let create_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "proposal",
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
            "LMV358",
            "--prefix",
            "U",
            "--manufacturer",
            "Datum Semi",
            "--gate-name",
            "A",
            "--proposal",
            &proposal_id.to_string(),
            "--rationale",
            "review AI-authored entity",
        ])
        .expect("CLI should parse"),
    )
    .expect("pool entity proposal create should succeed");
    let create_report: serde_json::Value =
        serde_json::from_str(&create_output).expect("create report JSON should parse");
    assert_eq!(create_report["contract"], "proposal_create_v1");
    assert_eq!(create_report["action"], "create_pool_entity_proposal");
    assert_eq!(create_report["proposal_id"], proposal_id.to_string());
    assert_eq!(create_report["proposal"]["source"], "tool");
    assert!(
        !entity_path.exists(),
        "proposal creation must not write the pool entity shard"
    );

    execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "proposal",
            "accept-apply",
            root.to_str().unwrap(),
            "--proposal",
            &proposal_id.to_string(),
        ])
        .expect("CLI should parse"),
    )
    .expect("pool entity proposal accept-apply should succeed");

    assert!(entity_path.exists());
    let queried = query_pool_object_payload(&root, "entities", entity_id);
    assert_eq!(queried["uuid"], entity_id.to_string());
    assert_eq!(queried["name"], "LMV358");
    assert_eq!(queried["prefix"], "U");
    assert_eq!(queried["manufacturer"], "Datum Semi");
    assert_eq!(
        queried["gates"][gate_id.to_string()]["unit"],
        unit_id.to_string()
    );
    assert_eq!(
        queried["gates"][gate_id.to_string()]["symbol"],
        symbol_id.to_string()
    );
}

#[test]
fn proposal_create_pool_entity_rejects_symbol_unit_mismatch() {
    let root = unique_project_root("datum-eda-cli-project-pool-entity-proposal-mismatch");
    create_native_project(&root, Some("Pool Entity Proposal Mismatch".to_string()))
        .expect("initial scaffold should succeed");
    let requested_unit_id = Uuid::new_v4();
    let symbol_unit_id = Uuid::new_v4();
    let symbol_id = Uuid::new_v4();
    let entity_id = Uuid::new_v4();
    let gate_id = Uuid::new_v4();
    seed_promoted_pool_unit(&root, requested_unit_id);
    seed_promoted_pool_unit(&root, symbol_unit_id);
    seed_promoted_pool_symbol(&root, symbol_id, symbol_unit_id);

    let error = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "proposal",
            "create-pool-entity",
            root.to_str().unwrap(),
            "--entity",
            &entity_id.to_string(),
            "--gate",
            &gate_id.to_string(),
            "--unit",
            &requested_unit_id.to_string(),
            "--symbol",
            &symbol_id.to_string(),
            "--name",
            "Mismatch",
            "--prefix",
            "U",
        ])
        .expect("CLI should parse"),
    )
    .expect_err("symbol/unit mismatch should fail proposal creation");
    assert!(format!("{error:#}").contains("does not reference unit"));
    assert!(
        !root
            .join(format!("pool/entities/{entity_id}.json"))
            .exists()
    );
}

#[test]
fn proposal_create_pool_padstack_defers_until_accept_apply() {
    let root = unique_project_root("datum-eda-cli-project-pool-padstack-proposal");
    create_native_project(&root, Some("Pool Padstack Proposal Demo".to_string()))
        .expect("initial scaffold should succeed");
    let padstack_id = Uuid::new_v4();
    let proposal_id = Uuid::new_v4();
    let padstack_path = root.join(format!("pool/padstacks/{padstack_id}.json"));

    let create_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "proposal",
            "create-pool-padstack",
            root.to_str().unwrap(),
            "--padstack",
            &padstack_id.to_string(),
            "--name",
            "RoundViaPad",
            "--aperture",
            "circle",
            "--diameter-nm",
            "1200000",
            "--drill-nm",
            "600000",
            "--proposal",
            &proposal_id.to_string(),
            "--rationale",
            "review AI-authored padstack",
        ])
        .expect("CLI should parse"),
    )
    .expect("pool padstack proposal create should succeed");
    let create_report: serde_json::Value =
        serde_json::from_str(&create_output).expect("create report JSON should parse");
    assert_eq!(create_report["contract"], "proposal_create_v1");
    assert_eq!(create_report["action"], "create_pool_padstack_proposal");
    assert_eq!(create_report["proposal_id"], proposal_id.to_string());
    assert_eq!(create_report["proposal"]["source"], "tool");
    assert!(
        !padstack_path.exists(),
        "proposal creation must not write the pool padstack shard"
    );

    execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "proposal",
            "accept-apply",
            root.to_str().unwrap(),
            "--proposal",
            &proposal_id.to_string(),
        ])
        .expect("CLI should parse"),
    )
    .expect("pool padstack proposal accept-apply should succeed");

    assert!(padstack_path.exists());
    let queried = query_pool_object_payload(&root, "padstacks", padstack_id);
    assert_eq!(queried["uuid"], padstack_id.to_string());
    assert_eq!(queried["name"], "RoundViaPad");
    assert_eq!(queried["aperture"]["circle"]["diameter_nm"], 1200000);
    assert_eq!(queried["drill_nm"], 600000);
}

#[test]
fn proposal_create_pool_padstack_rejects_invalid_aperture_arguments() {
    let root = unique_project_root("datum-eda-cli-project-pool-padstack-proposal-invalid");
    create_native_project(&root, Some("Pool Padstack Proposal Invalid".to_string()))
        .expect("initial scaffold should succeed");
    let padstack_id = Uuid::new_v4();

    let error = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "proposal",
            "create-pool-padstack",
            root.to_str().unwrap(),
            "--padstack",
            &padstack_id.to_string(),
            "--name",
            "BadRect",
            "--aperture",
            "rect",
            "--diameter-nm",
            "1000000",
        ])
        .expect("CLI should parse"),
    )
    .expect_err("invalid padstack aperture should fail proposal creation");
    assert!(format!("{error:#}").contains("rect padstack aperture does not accept diameter-nm"));
    assert!(
        !root
            .join(format!("pool/padstacks/{padstack_id}.json"))
            .exists()
    );
}
