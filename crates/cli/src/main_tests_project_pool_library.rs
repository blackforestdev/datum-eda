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
fn write_pool_object_payload(root: &Path, kind: &str, object_id: Uuid) -> PathBuf {
    write_pool_object_payload_named(root, kind, object_id, &format!("Test {kind}"))
}

fn write_pool_object_payload_named(
    root: &Path,
    kind: &str,
    object_id: Uuid,
    name: &str,
) -> PathBuf {
    let path = root.join(format!("{kind}-{object_id}.json"));
    let payload = match kind {
        "units" => {
            serde_json::json!({"schema_version": 1, "uuid": object_id, "name": name, "manufacturer": "", "pins": {}, "tags": []})
        }
        "symbols" => {
            serde_json::json!({"schema_version": 1, "uuid": object_id, "name": name, "unit": Uuid::new_v4()})
        }
        "entities" => {
            serde_json::json!({"schema_version": 1, "uuid": object_id, "name": name, "prefix": "U", "manufacturer": "", "gates": {}, "tags": []})
        }
        "parts" => {
            serde_json::json!({"schema_version": 1, "uuid": object_id, "entity": Uuid::new_v4(), "package": Uuid::new_v4(), "pad_map": {}, "mpn": "", "manufacturer": "", "value": name, "description": "", "datasheet": "", "parametric": {}, "orderable_mpns": [], "tags": [], "lifecycle": "Active", "base": null})
        }
        "packages" => {
            serde_json::json!({"schema_version": 1, "uuid": object_id, "name": name, "pads": {}, "courtyard": {"vertices": [], "closed": true}, "silkscreen": [], "models_3d": [], "tags": []})
        }
        "padstacks" => {
            serde_json::json!({"schema_version": 1, "uuid": object_id, "name": name, "aperture": null, "drill_nm": null})
        }
        "pin_pad_maps" => {
            serde_json::json!({"schema_version": 1, "uuid": object_id, "name": name, "part": Uuid::new_v4(), "mappings": {}})
        }
        _ => serde_json::json!({"schema_version": 1, "uuid": object_id, "name": name}),
    };
    std::fs::write(
        &path,
        serde_json::to_string_pretty(&payload).expect("pool object payload should serialize"),
    )
    .expect("pool object payload should write");
    path
}
fn create_typed_pool_padstack(root: &Path, padstack_id: Uuid) {
    let padstack = padstack_id.to_string();
    execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "create-pool-padstack",
            root.to_str().unwrap(),
            "--padstack",
            &padstack,
            "--name",
            "RoundViaPad",
            "--aperture",
            "circle",
            "--diameter-nm",
            "1200000",
        ])
        .expect("CLI should parse"),
    )
    .expect("typed pool padstack create should succeed");
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

#[test]
fn project_pool_library_object_create_delete_are_journaled_and_undoable() {
    let root = unique_project_root("datum-eda-cli-project-pool-library");
    create_native_project(&root, Some("Pool Library Demo".to_string()))
        .expect("initial scaffold should succeed");
    let symbol_id = Uuid::new_v4();
    let symbol_payload = write_symbol_payload(&root, symbol_id);
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
    assert!(
        resolve_report["source_shards"]
            .as_array()
            .expect("source shards should be an array")
            .iter()
            .any(|shard| shard["path"] == symbol_relative_path)
    );

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
fn project_pool_library_object_create_rejects_mismatched_payload_uuid() {
    let root = unique_project_root("datum-eda-cli-project-pool-library-mismatch");
    create_native_project(&root, Some("Pool Library Mismatch".to_string()))
        .expect("initial scaffold should succeed");
    let symbol_id = Uuid::new_v4();
    let symbol_payload = write_symbol_payload(&root, Uuid::new_v4());
    let error = execute(
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
    .expect_err("mismatched payload uuid should fail");
    assert!(format!("{error:#}").contains("does not match --object"));
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_pool_library_object_create_rejects_missing_schema_version() {
    let root = unique_project_root("datum-eda-cli-project-pool-library-schema");
    create_native_project(&root, Some("Pool Library Schema".to_string()))
        .expect("initial scaffold should succeed");
    let symbol_id = Uuid::new_v4();
    let path = root.join("missing-schema-symbol.json");
    std::fs::write(
        &path,
        serde_json::to_string_pretty(&serde_json::json!({
            "uuid": symbol_id,
            "name": "MissingSchema",
            "unit": Uuid::new_v4()
        }))
        .expect("symbol payload should serialize"),
    )
    .expect("symbol payload should write");
    let error = execute(
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
            path.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect_err("missing schema_version should fail");
    assert!(format!("{error:#}").contains("missing schema_version"));
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_query_pool_library_objects_reports_resolver_backed_objects() {
    let root = unique_project_root("datum-eda-cli-project-pool-library-query");
    create_native_project(&root, Some("Pool Library Query".to_string()))
        .expect("initial scaffold should succeed");
    let kinds = [
        "units",
        "symbols",
        "entities",
        "parts",
        "packages",
        "footprints",
        "padstacks",
        "pin_pad_maps",
    ];
    let mut created = Vec::new();
    for kind in kinds {
        let object_id = Uuid::new_v4();
        let payload = write_pool_object_payload(&root, kind, object_id);
        execute(
            Cli::try_parse_from([
                "eda",
                "--format",
                "json",
                "project",
                "create-pool-library-object",
                root.to_str().unwrap(),
                "--kind",
                kind,
                "--object",
                &object_id.to_string(),
                "--from-json",
                payload.to_str().unwrap(),
            ])
            .expect("CLI should parse"),
        )
        .expect("pool library object create should succeed");
        created.push((kind.to_string(), object_id));
    }

    let query_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "query",
            root.to_str().unwrap(),
            "pool-library-objects",
        ])
        .expect("CLI should parse"),
    )
    .expect("pool library objects query should succeed");
    let query_report: serde_json::Value =
        serde_json::from_str(&query_output).expect("query JSON should parse");
    assert_eq!(
        query_report["contract"],
        "native_project_library_objects_query_v1"
    );
    assert_eq!(query_report["object_count"], 8);
    for (kind, object_id) in &created {
        assert!(
            query_report["objects"]
                .as_array()
                .expect("objects should be an array")
                .iter()
                .any(|object| object["object_kind"] == *kind
                    && object["object_uuid"] == object_id.to_string()
                    && object["relative_path"] == format!("pool/{kind}/{object_id}.json"))
        );
    }

    let (kind, object_id) = created
        .iter()
        .find(|(kind, _)| kind == "symbols")
        .expect("symbol object should exist");
    let filtered_output = execute(
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
            &object_id.to_string(),
            "--include-payload",
        ])
        .expect("CLI should parse"),
    )
    .expect("filtered pool library objects query should succeed");
    let filtered_report: serde_json::Value =
        serde_json::from_str(&filtered_output).expect("filtered query JSON should parse");
    assert_eq!(filtered_report["object_count"], 1);
    assert_eq!(
        filtered_report["objects"][0]["payload"]["uuid"],
        object_id.to_string()
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_create_pool_unit_authors_typed_unit_through_journal() {
    let root = unique_project_root("datum-eda-cli-project-pool-unit");
    create_native_project(&root, Some("Pool Unit".to_string()))
        .expect("initial scaffold should succeed");
    let unit_id = Uuid::new_v4();
    let output = execute(
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
            "--manufacturer",
            "Datum",
        ])
        .expect("CLI should parse"),
    )
    .expect("typed pool unit create should succeed");
    let report: serde_json::Value =
        serde_json::from_str(&output).expect("create-unit report JSON should parse");
    assert_eq!(report["action"], "create_unit");
    assert_eq!(report["object_kind"], "units");
    assert_eq!(
        report["relative_path"],
        format!("pool/units/{unit_id}.json")
    );
    let journal = std::fs::read_to_string(root.join(".datum/journal/transactions.jsonl"))
        .expect("transaction journal should exist");
    assert!(journal.contains("\"kind\":\"create_pool_library_object\""));

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
            "units",
            "--object",
            &unit_id.to_string(),
            "--include-payload",
        ])
        .expect("CLI should parse"),
    )
    .expect("unit query should succeed");
    let query_report: serde_json::Value =
        serde_json::from_str(&query_output).expect("query report JSON should parse");
    assert_eq!(query_report["object_count"], 1);
    assert_eq!(query_report["objects"][0]["payload"]["name"], "OpAmpUnit");
    assert_eq!(
        query_report["objects"][0]["payload"]["manufacturer"],
        "Datum"
    );
    assert!(query_report["objects"][0]["payload"]["pins"].is_object());
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_create_pool_padstack_authors_typed_circle_padstack() {
    let root = unique_project_root("datum-eda-cli-project-pool-padstack");
    create_native_project(&root, Some("Pool Padstack".to_string()))
        .expect("initial scaffold should succeed");
    let padstack_id = Uuid::new_v4();
    let padstack = padstack_id.to_string();
    let output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "create-pool-padstack",
            root.to_str().unwrap(),
            "--padstack",
            &padstack,
            "--name",
            "RoundViaPad",
            "--aperture",
            "circle",
            "--diameter-nm",
            "1200000",
            "--drill-nm",
            "600000",
        ])
        .expect("CLI should parse"),
    )
    .expect("typed pool padstack create should succeed");
    let report: serde_json::Value =
        serde_json::from_str(&output).expect("create-padstack report JSON should parse");
    assert_eq!(report["action"], "create_padstack");
    assert_eq!(report["object_kind"], "padstacks");
    assert_eq!(
        report["relative_path"],
        format!("pool/padstacks/{padstack_id}.json")
    );
    let payload = query_pool_object_payload(&root, "padstacks", padstack_id);
    assert_eq!(payload["aperture"]["circle"]["diameter_nm"], 1200000);
    assert_eq!(payload["drill_nm"], 600000);
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_create_pool_padstack_rejects_invalid_aperture_arguments() {
    let root = unique_project_root("datum-eda-cli-project-pool-padstack-invalid");
    create_native_project(&root, Some("Pool Padstack Invalid".to_string()))
        .expect("initial scaffold should succeed");
    let padstack_id = Uuid::new_v4();
    let padstack = padstack_id.to_string();
    let error = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "create-pool-padstack",
            root.to_str().unwrap(),
            "--padstack",
            &padstack,
            "--name",
            "BadRect",
            "--aperture",
            "rect",
            "--diameter-nm",
            "1000000",
        ])
        .expect("CLI should parse"),
    )
    .expect_err("invalid padstack aperture should fail");
    assert!(format!("{error:#}").contains("rect padstack aperture does not accept diameter-nm"));
    assert!(
        !root
            .join(format!("pool/padstacks/{padstack_id}.json"))
            .exists()
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_create_pool_package_authors_typed_package_with_pad() {
    let root = unique_project_root("datum-eda-cli-project-pool-package");
    create_native_project(&root, Some("Pool Package".to_string()))
        .expect("initial scaffold should succeed");
    let padstack_id = Uuid::new_v4();
    create_typed_pool_padstack(&root, padstack_id);
    let package_id = Uuid::new_v4();
    let pad_id = Uuid::new_v4();
    let (package, pad, padstack) = (
        package_id.to_string(),
        pad_id.to_string(),
        padstack_id.to_string(),
    );
    let output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "create-pool-package",
            root.to_str().unwrap(),
            "--package",
            &package,
            "--name",
            "SOT23",
            "--pad",
            &pad,
            "--padstack",
            &padstack,
            "--pad-name",
            "1",
            "--x-nm",
            "1000",
            "--y-nm",
            "2000",
            "--layer",
            "1",
        ])
        .expect("CLI should parse"),
    )
    .expect("typed pool package create should succeed");
    let report: serde_json::Value =
        serde_json::from_str(&output).expect("create-package report JSON should parse");
    assert_eq!(report["action"], "create_package");
    assert_eq!(
        report["relative_path"],
        format!("pool/packages/{package_id}.json")
    );
    let payload = query_pool_object_payload(&root, "packages", package_id);
    assert_eq!(
        payload["pads"][pad_id.to_string()]["padstack"],
        padstack_id.to_string()
    );
    assert_eq!(payload["pads"][pad_id.to_string()]["position"]["x"], 1000);
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_create_pool_package_rejects_missing_padstack() {
    let root = unique_project_root("datum-eda-cli-project-pool-package-missing-padstack");
    create_native_project(&root, Some("Pool Package Missing Padstack".to_string()))
        .expect("initial scaffold should succeed");
    let (package_id, pad_id, padstack_id) = (Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4());
    let (package, pad, padstack) = (
        package_id.to_string(),
        pad_id.to_string(),
        padstack_id.to_string(),
    );
    let error = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "create-pool-package",
            root.to_str().unwrap(),
            "--package",
            &package,
            "--name",
            "BadPackage",
            "--pad",
            &pad,
            "--padstack",
            &padstack,
        ])
        .expect("CLI should parse"),
    )
    .expect_err("package with missing padstack should fail");
    assert!(format!("{error:#}").contains("missing pool padstack"));
    assert!(
        !root
            .join(format!("pool/packages/{package_id}.json"))
            .exists()
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_pool_library_object_set_is_journaled_query_visible_and_undoable() {
    let root = unique_project_root("datum-eda-cli-project-pool-library-set");
    create_native_project(&root, Some("Pool Library Set".to_string()))
        .expect("initial scaffold should succeed");
    let symbol_id = Uuid::new_v4();
    let initial_payload =
        write_pool_object_payload_named(&root, "symbols", symbol_id, "Initial Symbol");
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

    let replacement_payload =
        write_pool_object_payload_named(&root, "symbols", symbol_id, "Edited Symbol");
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
