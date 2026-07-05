use super::*;

fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}", label, Uuid::new_v4()))
}

#[test]
fn project_pool_authoring_reads_resolver_materialized_pool_refs() {
    let root = unique_project_root("datum-eda-cli-project-pool-resolver");
    create_native_project(&root, Some("Pool Resolver".to_string()))
        .expect("initial scaffold should succeed");
    let project_json = root.join("project.json");
    let stale_project_without_pool =
        std::fs::read_to_string(&project_json).expect("project manifest should read");

    let unit_id = Uuid::new_v4();
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
    std::fs::write(&project_json, stale_project_without_pool)
        .expect("stale project manifest should restore");

    let second_unit_id = Uuid::new_v4();
    execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "create-pool-unit",
            root.to_str().unwrap(),
            "--unit",
            &second_unit_id.to_string(),
            "--name",
            "SecondUnit",
        ])
        .expect("CLI should parse"),
    )
    .expect("second typed pool unit create should use resolver materialized pool refs");

    let journal = std::fs::read_to_string(root.join(".datum/journal/transactions.jsonl"))
        .expect("transaction journal should exist");
    assert_eq!(
        journal.matches("\"kind\":\"add_project_pool_ref\"").count(),
        1
    );

    let pools_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "query",
            root.to_str().unwrap(),
            "pools",
        ])
        .expect("CLI should parse"),
    )
    .expect("pool query should use resolver materialized project manifest");
    let pools: serde_json::Value =
        serde_json::from_str(&pools_output).expect("pools query JSON should parse");
    assert_eq!(pools.as_array().expect("pools should be an array").len(), 1);
    assert_eq!(pools[0]["manifest_path"], "pool");

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_pool_mutation_reads_resolver_materialized_previous_object() {
    let root = unique_project_root("datum-eda-cli-project-pool-object-resolver");
    create_native_project(&root, Some("Pool Object Resolver".to_string()))
        .expect("initial scaffold should succeed");
    let part_id = Uuid::new_v4();
    let payload_path = root.join("part.json");
    std::fs::write(
        &payload_path,
        serde_json::to_string_pretty(&serde_json::json!({
            "schema_version": 1,
            "uuid": part_id,
            "entity": Uuid::new_v4(),
            "package": Uuid::new_v4(),
            "pad_map": {},
            "mpn": "OLD",
            "manufacturer": "Datum",
            "value": "R1",
            "description": "",
            "datasheet": "",
            "parametric": {},
            "orderable_mpns": [],
            "tags": [],
            "lifecycle": "Active",
            "base": null
        }))
        .expect("part payload should serialize"),
    )
    .expect("part payload should write");
    execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "create-pool-library-object",
            root.to_str().unwrap(),
            "--kind",
            "parts",
            "--object",
            &part_id.to_string(),
            "--from-json",
            payload_path.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("pool part create should succeed");

    let promoted_part = root.join(format!("pool/parts/{part_id}.json"));
    std::fs::remove_file(&promoted_part).expect("promoted part should remove");
    execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "set-pool-part-metadata",
            root.to_str().unwrap(),
            "--part",
            &part_id.to_string(),
            "--mpn",
            "NEW",
        ])
        .expect("CLI should parse"),
    )
    .expect("metadata edit should use resolver-materialized previous object");

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
            "parts",
            "--object",
            &part_id.to_string(),
            "--include-payload",
        ])
        .expect("CLI should parse"),
    )
    .expect("pool part query should succeed after resolver-backed edit");
    let report: serde_json::Value =
        serde_json::from_str(&output).expect("query report JSON should parse");
    assert_eq!(report["object_count"], 1);
    assert_eq!(report["objects"][0]["payload"]["mpn"], "NEW");

    let journal = std::fs::read_to_string(root.join(".datum/journal/transactions.jsonl"))
        .expect("transaction journal should exist");
    assert!(journal.contains("\"kind\":\"set_pool_library_object\""));

    let _ = std::fs::remove_dir_all(&root);
}
