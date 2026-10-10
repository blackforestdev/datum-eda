//! Ordinary generic Symbol authoring retains canonical admission and history.
use super::*;
use eda_engine::substrate::{Operation, ProjectResolver};

fn command(root: &Path, verb: &str, args: &[&str]) -> Result<String> {
    let mut argv = vec![
        "datum-eda",
        "--format",
        "json",
        "project",
        verb,
        root.to_str().unwrap(),
    ];
    argv.extend_from_slice(args);
    execute(Cli::try_parse_from(argv).unwrap())
}
fn project(label: &str) -> PathBuf {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../.datum/footprint-reference-audit")
        .join(format!("{label}-{}", Uuid::new_v4()));
    create_native_project(&root, Some(label.to_string())).unwrap();
    root
}
fn journal(root: &Path) -> Vec<u8> {
    std::fs::read(root.join(".datum/journal/transactions.jsonl")).unwrap_or_default()
}
fn payload(root: &Path, id: Uuid) -> serde_json::Value {
    super::query_pool_object_payload(root, "symbols", id)
}
fn write(root: &Path, verb: &str, id: Uuid, value: &serde_json::Value) -> Result<String> {
    let file = root.join("symbol-input.json");
    std::fs::write(&file, serde_json::to_vec(value).unwrap()).unwrap();
    command(
        root,
        verb,
        &[
            "--kind",
            "symbols",
            "--object",
            &id.to_string(),
            "--from-json",
            file.to_str().unwrap(),
        ],
    )
}
struct Fixture {
    root: PathBuf,
    unit: Uuid,
    alternate: Uuid,
    package: Uuid,
    symbol: Uuid,
}
impl Fixture {
    fn new(label: &str) -> Self {
        let root = project(label);
        let unit = Uuid::new_v4();
        let alternate = Uuid::new_v4();
        let package = Uuid::new_v4();
        let symbol = Uuid::new_v4();
        for (id, name) in [(unit, "Manual unit"), (alternate, "Alternate native unit")] {
            command(
                &root,
                "create-pool-unit",
                &["--unit", &id.to_string(), "--name", name],
            )
            .unwrap();
        }
        command(
            &root,
            "create-pool-package",
            &[
                "--package",
                &package.to_string(),
                "--name",
                "Manual package",
            ],
        )
        .unwrap();
        command(
            &root,
            "create-pool-symbol",
            &[
                "--symbol",
                &symbol.to_string(),
                "--unit",
                &unit.to_string(),
                "--name",
                "Manual symbol",
            ],
        )
        .unwrap();
        Self {
            root,
            unit,
            alternate,
            package,
            symbol,
        }
    }
}

#[test]
fn library_definition_symbol_refusals_preserve_canonical_state_and_errors() {
    let f = Fixture::new("symbol-refusals");
    let root = &f.root;
    let original = payload(root, f.symbol);
    let missing = Uuid::new_v4();
    let created = Uuid::new_v4();
    let before = ProjectResolver::new(root).resolve().unwrap();
    let before_journal = journal(root);
    let manifest = std::fs::read(root.join("project.json")).unwrap();
    let path = root.join(format!("pool/symbols/{}.json", f.symbol));
    let source = std::fs::read(&path).unwrap();
    for verb in ["create-pool-library-object", "set-pool-library-object"] {
        let target = if verb.starts_with("create") {
            created
        } else {
            f.symbol
        };
        for unit in [missing, f.package] {
            let mut value = original.clone();
            value["uuid"] = serde_json::json!(target);
            value["unit"] = serde_json::json!(unit);
            assert_eq!(
                format!("{:#}", write(root, verb, target, &value).unwrap_err()),
                format!("missing pool unit {unit} for symbol {target}")
            );
            assert_eq!(journal(root), before_journal);
            assert_eq!(std::fs::read(root.join("project.json")).unwrap(), manifest);
            assert_eq!(std::fs::read(&path).unwrap(), source);
            assert!(!root.join(format!("pool/symbols/{created}.json")).exists());
            let after = ProjectResolver::new(root).resolve().unwrap();
            assert_eq!(after.model_revision, before.model_revision);
            assert_eq!(after.objects.len(), before.objects.len());
            for (id, object) in &before.objects {
                assert_eq!(after.objects[id].object_revision, object.object_revision);
            }
        }
        for fault in ["uuid", "schema", "shape", "foreign"] {
            let mut value = original.clone();
            value["uuid"] = serde_json::json!(target);
            value["unit"] = serde_json::json!(missing);
            let expected = match fault {
                "uuid" => {
                    value["uuid"] = serde_json::json!(missing);
                    "does not match --object"
                }
                "schema" => {
                    value["schema_version"] = serde_json::json!(2);
                    "unsupported pool library object schema_version 2; expected 1"
                }
                "shape" => {
                    value["pin_anchors"] = serde_json::json!(1);
                    "invalid pool library symbols object"
                }
                "foreign" => {
                    value["extension"] = serde_json::json!({"uuid": f.package});
                    if verb.starts_with("create") {
                        "source UUID already exists"
                    } else {
                        "foreign source UUID collision"
                    }
                }
                _ => unreachable!(),
            };
            assert!(
                format!("{:#}", write(root, verb, target, &value).unwrap_err()).contains(expected)
            );
            assert_eq!(journal(root), before_journal);
            assert_eq!(std::fs::read(&path).unwrap(), source);
        }
    }
    let mut collision = original.clone();
    collision["unit"] = serde_json::json!(missing);
    assert!(
        format!(
            "{:#}",
            write(root, "create-pool-library-object", f.symbol, &collision).unwrap_err()
        )
        .contains("source UUID already exists")
    );
    assert_eq!(journal(root), before_journal);
    let empty = project("symbol-no-pool");
    let empty_manifest = std::fs::read(empty.join("project.json")).unwrap();
    let mut value = original.clone();
    value["uuid"] = serde_json::json!(created);
    assert_eq!(
        format!(
            "{:#}",
            write(&empty, "create-pool-library-object", created, &value).unwrap_err()
        ),
        format!("missing pool unit {} for symbol {created}", f.unit)
    );
    assert_eq!(
        std::fs::read(empty.join("project.json")).unwrap(),
        empty_manifest
    );
    assert!(journal(&empty).is_empty());
    assert!(!empty.join(format!("pool/symbols/{created}.json")).exists());
    std::fs::remove_dir_all(empty).unwrap();
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn library_definition_symbol_valid_create_relink_reopen_undo_redo_preserves_payload() {
    let f = Fixture::new("symbol-history");
    let root = &f.root;
    let mut created = payload(root, f.symbol);
    let id = Uuid::new_v4();
    let leaf = Uuid::new_v4();
    created["uuid"] = serde_json::json!(id);
    created["name"] = serde_json::json!("");
    created["extension"] = serde_json::json!({"uuid": leaf, "retained": [1,2]});
    let before = ProjectResolver::new(root).resolve().unwrap();
    write(root, "create-pool-library-object", id, &created).unwrap();
    let after = ProjectResolver::new(root).resolve().unwrap();
    assert_eq!(after.journal.len(), before.journal.len() + 1);
    assert_eq!(after.objects[&id].object_revision.0, 0);
    assert!(after.objects.contains_key(&leaf));
    assert!(matches!(
        after.journal.last().unwrap().operations.as_slice(),
        [Operation::CreatePoolLibraryObject { .. }]
    ));
    assert_eq!(payload(root, id), created);
    command(root, "undo", &[]).unwrap();
    let undone = ProjectResolver::new(root).resolve().unwrap();
    assert!(!undone.objects.contains_key(&id) && !undone.objects.contains_key(&leaf));
    assert!(!root.join(format!("pool/symbols/{id}.json")).exists());
    command(root, "redo", &[]).unwrap();
    assert_eq!(payload(root, id), created);
    let before_set = ProjectResolver::new(root).resolve().unwrap();
    let mut relinked = created.clone();
    relinked["unit"] = serde_json::json!(f.alternate);
    // Null optional metadata and omitted vectors keep their serde admission behavior.
    relinked["default_refdes_prefix"] = serde_json::Value::Null;
    relinked["check_state"] = serde_json::Value::Null;
    write(root, "set-pool-library-object", id, &relinked).unwrap();
    let after_set = ProjectResolver::new(root).resolve().unwrap();
    assert_eq!(after_set.journal.len(), before_set.journal.len() + 1);
    assert_eq!(
        after_set.objects[&id].object_revision.0,
        before_set.objects[&id].object_revision.0 + 1
    );
    assert!(matches!(
        after_set.journal.last().unwrap().operations.as_slice(),
        [Operation::SetPoolLibraryObject { .. }]
    ));
    assert_eq!(
        after_set.objects[&leaf].source_shard_id,
        before_set.objects[&leaf].source_shard_id
    );
    for source in [f.unit, f.alternate, f.package, f.symbol] {
        assert_eq!(
            after_set.objects[&source].object_revision,
            before_set.objects[&source].object_revision
        );
    }
    assert_eq!(payload(root, id), relinked);
    command(root, "undo", &[]).unwrap();
    assert_eq!(payload(root, id), created);
    command(root, "redo", &[]).unwrap();
    assert_eq!(payload(root, id), relinked);
    let validation: serde_json::Value =
        serde_json::from_str(&command(root, "validate", &[]).unwrap()).unwrap();
    assert_eq!(validation["valid"], true);
    std::fs::remove_dir_all(root).unwrap();
}

// The existing generic lifecycle proofs belong beside Symbol admission/history.
fn symbol_source(root: &Path, id: Uuid, unit: Uuid, name: &str) -> PathBuf {
    let path = root.join(format!("symbol-{id}-input.json"));
    std::fs::write(
        &path,
        serde_json::to_vec(&serde_json::json!({
            "schema_version": 1, "uuid": id, "unit": unit, "name": name,
        }))
        .unwrap(),
    )
    .unwrap();
    path
}
fn native_unit(root: &Path) -> Uuid {
    let unit = Uuid::new_v4();
    command(
        root,
        "create-pool-unit",
        &["--unit", &unit.to_string(), "--name", "Native fixture unit"],
    )
    .unwrap();
    unit
}

#[test]
fn project_pool_library_object_create_delete_are_journaled_and_undoable() {
    let root = project("datum-eda-cli-project-pool-library");
    let unit = native_unit(&root);
    let symbol_id = Uuid::new_v4();
    let symbol_payload = symbol_source(&root, symbol_id, unit, "CliNativeSymbol");
    let symbol_relative_path = format!("pool/symbols/{symbol_id}.json");
    let symbol_path = root.join(&symbol_relative_path);

    let create_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "create-pool-library-object",
            root.to_str().unwrap(),
            "--kind",
            "symbols",
            "--object",
            &symbol_id.to_string(),
            "--from-json",
            symbol_payload.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("pool library object create should succeed");
    let create_report: serde_json::Value =
        serde_json::from_str(&create_output).expect("create report JSON should parse");
    assert_eq!(
        create_report["contract"],
        "native_project_pool_library_object_mutation_v1"
    );
    assert_eq!(create_report["action"], "create");
    assert_eq!(create_report["object_uuid"], symbol_id.to_string());
    assert_eq!(create_report["relative_path"], symbol_relative_path);
    assert!(symbol_path.exists());

    let journal = std::fs::read_to_string(root.join(".datum/journal/transactions.jsonl"))
        .expect("transaction journal should exist");
    assert!(journal.contains("\"kind\":\"add_project_pool_ref\""));
    assert!(journal.contains("\"kind\":\"create_pool_library_object\""));

    let resolve_output = execute(
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
    .expect("resolve-debug should succeed");
    let resolve_report: serde_json::Value =
        serde_json::from_str(&resolve_output).expect("resolve-debug JSON should parse");
    let symbol_shard = resolve_report["source_shards"]
        .as_array()
        .expect("source shards should be an array")
        .iter()
        .find(|shard| shard["path"] == symbol_relative_path)
        .expect("pool symbol shard should be reported");
    #[rustfmt::skip]
    assert_eq!((symbol_shard["kind"].as_str(), symbol_shard["taxon"].as_str()), (Some("Pool"), Some("PoolSymbol")));

    let delete_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "delete-pool-library-object",
            root.to_str().unwrap(),
            "--kind",
            "symbols",
            "--object",
            &symbol_id.to_string(),
        ])
        .expect("CLI should parse"),
    )
    .expect("pool library object delete should succeed");
    let delete_report: serde_json::Value =
        serde_json::from_str(&delete_output).expect("delete report JSON should parse");
    assert_eq!(delete_report["action"], "delete");
    assert!(!symbol_path.exists());
    let journal = std::fs::read_to_string(root.join(".datum/journal/transactions.jsonl"))
        .expect("transaction journal should exist");
    assert!(journal.contains("\"kind\":\"delete_pool_library_object\""));

    execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "undo",
            root.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("delete undo should succeed");
    assert!(symbol_path.exists());

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_pool_library_object_set_is_journaled_query_visible_and_undoable() {
    let root = project("datum-eda-cli-project-pool-library-set");
    let unit = native_unit(&root);
    let symbol_id = Uuid::new_v4();
    let initial_payload = symbol_source(&root, symbol_id, unit, "Initial Symbol");
    execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "create-pool-library-object",
            root.to_str().unwrap(),
            "--kind",
            "symbols",
            "--object",
            &symbol_id.to_string(),
            "--from-json",
            initial_payload.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("pool library object create should succeed");

    let replacement_payload = symbol_source(&root, symbol_id, unit, "Edited Symbol");
    let set_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "set-pool-library-object",
            root.to_str().unwrap(),
            "--kind",
            "symbols",
            "--object",
            &symbol_id.to_string(),
            "--from-json",
            replacement_payload.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("pool library object set should succeed");
    let set_report: serde_json::Value =
        serde_json::from_str(&set_output).expect("set report JSON should parse");
    assert_eq!(set_report["action"], "set");
    let journal = std::fs::read_to_string(root.join(".datum/journal/transactions.jsonl"))
        .expect("transaction journal should exist");
    assert!(journal.contains("\"kind\":\"set_pool_library_object\""));

    let query_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "query",
            root.to_str().unwrap(),
            "pool-library-objects",
            "--kind",
            "symbols",
            "--object",
            &symbol_id.to_string(),
            "--include-payload",
        ])
        .expect("CLI should parse"),
    )
    .expect("pool library objects query should succeed");
    let query_report: serde_json::Value =
        serde_json::from_str(&query_output).expect("query JSON should parse");
    assert_eq!(query_report["objects"][0]["object_revision"], 1);
    assert_eq!(
        query_report["objects"][0]["payload"]["name"],
        "Edited Symbol"
    );

    execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "undo",
            root.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("set undo should succeed");
    let restored: serde_json::Value = serde_json::from_slice(
        &std::fs::read(root.join(format!("pool/symbols/{symbol_id}.json")))
            .expect("symbol payload should exist after undo"),
    )
    .expect("restored payload should parse");
    assert_eq!(restored["name"], "Initial Symbol");
    let _ = std::fs::remove_dir_all(&root);
}
