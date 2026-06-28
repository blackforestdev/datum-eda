use super::*;
use eda_engine::substrate::{
    CommitProvenance, CommitSource, Operation, OperationBatch, ProjectResolver,
};

fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}", label, Uuid::new_v4()))
}

fn commit_schematic_sheet(root: &Path) -> (Uuid, String) {
    let schematic: serde_json::Value =
        serde_json::from_slice(&std::fs::read(root.join("schematic/schematic.json")).unwrap())
            .unwrap();
    let schematic_id = Uuid::parse_str(schematic["uuid"].as_str().unwrap()).unwrap();
    let mut model = ProjectResolver::new(root)
        .resolve()
        .expect("project resolves before schematic sheet");
    let sheet_id = Uuid::new_v4();
    let relative_path = format!("sheets/{sheet_id}.json");
    model
        .commit_journaled(
            root,
            OperationBatch {
                batch_id: Uuid::new_v4(),
                expected_model_revision: Some(model.model_revision.clone()),
                provenance: CommitProvenance {
                    actor: "cli-test".to_string(),
                    source: CommitSource::Cli,
                    reason: "record unreadable schematic sheet shard".to_string(),
                },
                operations: vec![Operation::CreateSchematicSheet {
                    schematic_id,
                    sheet_id,
                    relative_path: relative_path.clone(),
                    sheet: serde_json::json!({
                        "schema_version": 1,
                        "uuid": sheet_id,
                        "name": "Unknown Sheet",
                        "symbols": {},
                        "wires": {},
                        "junctions": {},
                        "labels": {},
                        "buses": {},
                        "bus_entries": {},
                        "ports": {},
                        "noconnects": {},
                        "texts": {},
                        "drawings": {}
                    }),
                }],
            },
        )
        .expect("schematic sheet should commit");
    (sheet_id, format!("schematic/{relative_path}"))
}

fn commit_schematic_definition(root: &Path) -> (Uuid, String) {
    let schematic: serde_json::Value =
        serde_json::from_slice(&std::fs::read(root.join("schematic/schematic.json")).unwrap())
            .unwrap();
    let schematic_id = Uuid::parse_str(schematic["uuid"].as_str().unwrap()).unwrap();
    let (root_sheet_id, _) = commit_schematic_sheet(root);
    let mut model = ProjectResolver::new(root)
        .resolve()
        .expect("project resolves before schematic definition");
    let definition_id = Uuid::new_v4();
    let relative_path = format!("definitions/{definition_id}.json");
    model
        .commit_journaled(
            root,
            OperationBatch {
                batch_id: Uuid::new_v4(),
                expected_model_revision: Some(model.model_revision.clone()),
                provenance: CommitProvenance {
                    actor: "cli-test".to_string(),
                    source: CommitSource::Cli,
                    reason: "record unreadable schematic definition shard".to_string(),
                },
                operations: vec![Operation::CreateSchematicDefinition {
                    schematic_id,
                    definition_id,
                    relative_path: relative_path.clone(),
                    definition: serde_json::json!({
                        "schema_version": 1,
                        "uuid": definition_id,
                        "root_sheet": root_sheet_id,
                        "name": "Unknown Definition"
                    }),
                }],
            },
        )
        .expect("schematic definition should commit");
    (definition_id, format!("schematic/{relative_path}"))
}

#[test]
fn project_query_resolve_debug_reports_unknown_schematic_sheet_shard() {
    let root = unique_project_root("datum-eda-cli-project-resolve-debug-unknown-sheet");
    create_native_project(
        &root,
        Some("Resolve Debug Unknown Schematic Sheet Demo".to_string()),
    )
    .expect("initial scaffold should succeed");
    let (sheet_id, relative_path) = commit_schematic_sheet(&root);
    let promoted_path = root.join(&relative_path);
    std::fs::remove_file(&promoted_path).expect("promoted schematic sheet should remove");
    std::fs::create_dir(&promoted_path)
        .expect("directory at promoted schematic sheet path should create");

    let output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "query",
            root.to_str().unwrap(),
            "resolve-debug",
        ])
        .expect("CLI should parse"),
    )
    .expect("project query resolve-debug should succeed");
    let report: serde_json::Value =
        serde_json::from_str(&output).expect("resolve-debug JSON should parse");
    assert!(
        report["source_shards"]
            .as_array()
            .unwrap()
            .iter()
            .any(|shard| {
                shard["path"] == relative_path
                    && shard["kind"] == "SchematicSheet"
                    && shard["authority"] == "AuthoredDesign"
                    && shard["dirty_state"] == "Unknown"
            }),
        "resolve-debug should expose unreadable journal-recovered schematic sheet as Unknown: {sheet_id}"
    );

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_query_resolve_debug_reports_unknown_schematic_definition_shard() {
    let root = unique_project_root("datum-eda-cli-project-resolve-debug-unknown-definition");
    create_native_project(
        &root,
        Some("Resolve Debug Unknown Schematic Definition Demo".to_string()),
    )
    .expect("initial scaffold should succeed");
    let (definition_id, relative_path) = commit_schematic_definition(&root);
    let promoted_path = root.join(&relative_path);
    std::fs::remove_file(&promoted_path).expect("promoted schematic definition should remove");
    std::fs::create_dir(&promoted_path)
        .expect("directory at promoted schematic definition path should create");

    let output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "query",
            root.to_str().unwrap(),
            "resolve-debug",
        ])
        .expect("CLI should parse"),
    )
    .expect("project query resolve-debug should succeed");
    let report: serde_json::Value =
        serde_json::from_str(&output).expect("resolve-debug JSON should parse");
    assert!(
        report["source_shards"]
            .as_array()
            .unwrap()
            .iter()
            .any(|shard| {
                shard["path"] == relative_path
                    && shard["kind"] == "SchematicDefinition"
                    && shard["authority"] == "AuthoredDesign"
                    && shard["dirty_state"] == "Unknown"
            }),
        "resolve-debug should expose unreadable journal-recovered schematic definition as Unknown: {definition_id}"
    );

    let model = ProjectResolver::new(&root)
        .resolve()
        .expect("project should still resolve with unreadable promoted definition");
    let materialized = model
        .materialized_source_shard_value_by_relative_path(&relative_path)
        .expect("journal-recovered definition should materialize despite unreadable promoted path");
    assert_eq!(materialized["uuid"], definition_id.to_string());

    let _ = std::fs::remove_dir_all(&root);
}
