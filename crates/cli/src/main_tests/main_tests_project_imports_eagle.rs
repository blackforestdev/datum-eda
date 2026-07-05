use super::*;

fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}", label, Uuid::new_v4()))
}

fn write_temp_eagle_library(root: &Path, name: &str) -> PathBuf {
    let path = root.join(name);
    std::fs::copy(eagle_fixture_path("simple-opamp.lbr"), &path)
        .expect("Eagle fixture should copy");
    path
}

fn remove_altamp_from_eagle_library(path: &Path) {
    let mut source = std::fs::read_to_string(path).expect("Eagle fixture should read");
    let alt_package = r#"        <package name="ALT-3">
          <wire x1="-0.06" y1="0.05" x2="0.06" y2="0.05" width="0.005" layer="21"/>
          <smd name="A" x="-0.05" y="-0.02" dx="0.02" dy="0.05" layer="1"/>
          <smd name="B" x="0.00" y="0.00" dx="0.02" dy="0.05" layer="1"/>
          <smd name="C" x="0.05" y="0.02" dx="0.02" dy="0.05" layer="1"/>
        </package>
"#;
    let alt_deviceset = r#"        <deviceset name="ALTAMP" prefix="U">
          <gates>
            <gate name="G$1" symbol="OPAMP" x="0" y="0"/>
          </gates>
          <devices>
            <device name="" package="ALT-3">
              <connects>
                <connect gate="G$1" pin="IN+" pad="A"/>
                <connect gate="G$1" pin="IN-" pad="B"/>
                <connect gate="G$1" pin="OUT" pad="C"/>
              </connects>
              <technologies>
                <technology name=""/>
              </technologies>
            </device>
          </devices>
        </deviceset>
"#;
    assert!(source.contains(alt_package));
    assert!(source.contains(alt_deviceset));
    source = source.replace(alt_package, "");
    source = source.replace(alt_deviceset, "");
    std::fs::write(path, source).expect("mutated Eagle fixture should write");
}

#[test]
fn project_import_eagle_library_persists_pool_objects_and_import_map() {
    let root = unique_project_root("datum-eda-cli-project-import-eagle-library");
    create_native_project(&root, Some("Import Eagle Library Demo".to_string()))
        .expect("initial scaffold should succeed");
    let source = eagle_fixture_path("simple-opamp.lbr");

    let output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "import-eagle-library",
            root.to_str().unwrap(),
            "--source",
            source.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("Eagle library import should succeed");
    let report: serde_json::Value =
        serde_json::from_str(&output).expect("Eagle import report JSON should parse");
    assert_eq!(report["contract"], "native_project_eagle_library_import_v1");
    assert_eq!(report["imported_unit_count"], 1);
    assert_eq!(report["imported_symbol_count"], 1);
    assert_eq!(report["imported_entity_count"], 2);
    assert_eq!(report["imported_package_count"], 2);
    assert_eq!(report["imported_part_count"], 2);
    assert_eq!(report["imported_padstack_count"], 6);
    assert_eq!(report["created_object_count"], 14);
    assert_eq!(report["import_map_entry_count"], 14);
    assert_eq!(report["reused_existing_identity_count"], 0);
    assert!(root.join("pool/units").read_dir().unwrap().next().is_some());
    assert!(
        root.join("pool/symbols")
            .read_dir()
            .unwrap()
            .next()
            .is_some()
    );
    assert!(
        root.join("pool/entities")
            .read_dir()
            .unwrap()
            .next()
            .is_some()
    );
    assert!(
        root.join("pool/packages")
            .read_dir()
            .unwrap()
            .next()
            .is_some()
    );
    assert!(root.join("pool/parts").read_dir().unwrap().next().is_some());
    assert!(
        root.join("pool/padstacks")
            .read_dir()
            .unwrap()
            .next()
            .is_some()
    );

    let import_map_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "query",
            root.to_str().unwrap(),
            "import-map",
        ])
        .expect("CLI should parse"),
    )
    .expect("import-map query should succeed");
    let import_map: serde_json::Value =
        serde_json::from_str(&import_map_output).expect("import-map JSON should parse");
    assert_eq!(import_map["import_map_count"], 14);
    let entries = import_map["entries"]
        .as_object()
        .expect("import-map entries should be an object");
    assert!(entries.keys().any(|key| key.contains("eagle:lbr:")));
    assert!(entries.keys().any(|key| key.contains(":units:")));
    assert!(entries.keys().any(|key| key.contains(":symbols:")));
    assert!(entries.keys().any(|key| key.contains(":entities:")));
    assert!(entries.keys().any(|key| key.contains(":packages:")));
    assert!(entries.keys().any(|key| key.contains(":parts:")));
    assert!(entries.keys().any(|key| key.contains(":padstacks:")));
    assert!(
        entries
            .values()
            .all(|entry| entry["source_tool"] == "eagle")
    );
    assert!(entries.values().all(|entry| entry["status"] == "active"));
    assert_eagle_source_refs_are_source_facing(entries);
    assert!(
        entries
            .values()
            .any(|entry| entry["source_object_ref"] == "package:SOT23-5")
    );
    assert!(
        entries
            .values()
            .any(|entry| entry["source_object_ref"] == "deviceset:LMV321")
    );
    assert!(
        entries
            .values()
            .any(|entry| entry["source_object_ref"] == "device:LMV321:package:SOT23-5")
    );

    let second = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "import-eagle-library",
            root.to_str().unwrap(),
            "--source",
            source.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("second Eagle library import should succeed");
    let second_report: serde_json::Value =
        serde_json::from_str(&second).expect("second Eagle import report JSON should parse");
    assert_eq!(second_report["created_object_count"], 0);
    assert_eq!(second_report["import_map_entry_count"], 0);
    assert_eq!(second_report["reused_existing_identity_count"], 14);
    let journal = std::fs::read_to_string(root.join(".datum/journal/transactions.jsonl"))
        .expect("transaction journal should exist");
    assert_eq!(
        journal
            .matches("\"kind\":\"create_pool_library_object\"")
            .count(),
        14
    );
    assert_eq!(
        journal
            .matches("\"kind\":\"create_import_map_shard\"")
            .count(),
        1
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
    .expect("project undo should succeed");
    assert!(import_map_dir_is_empty(&root));
    let after_undo_import_map_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "query",
            root.to_str().unwrap(),
            "import-map",
        ])
        .expect("CLI should parse"),
    )
    .expect("import-map query after undo should succeed");
    let after_undo_import_map: serde_json::Value =
        serde_json::from_str(&after_undo_import_map_output)
            .expect("post-undo import-map JSON should parse");
    assert_eq!(after_undo_import_map["import_map_count"], 0);

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_import_eagle_library_marks_absent_source_entries_missing() {
    let root = unique_project_root("datum-eda-cli-project-import-eagle-library-missing");
    create_native_project(&root, Some("Import Eagle Library Missing Demo".to_string()))
        .expect("initial scaffold should succeed");
    let source = write_temp_eagle_library(&root, "simple-opamp-edit.lbr");

    execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "import-eagle-library",
            root.to_str().unwrap(),
            "--source",
            source.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("initial Eagle library import should succeed");

    remove_altamp_from_eagle_library(&source);
    let output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "import-eagle-library",
            root.to_str().unwrap(),
            "--source",
            source.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("shrunk Eagle library import should succeed");
    let report: serde_json::Value =
        serde_json::from_str(&output).expect("shrunk Eagle import report JSON should parse");
    assert_eq!(report["imported_unit_count"], 1);
    assert_eq!(report["imported_symbol_count"], 1);
    assert_eq!(report["imported_entity_count"], 1);
    assert_eq!(report["imported_package_count"], 1);
    assert_eq!(report["imported_part_count"], 1);
    assert_eq!(report["imported_padstack_count"], 3);
    assert_eq!(report["created_object_count"], 0);
    assert_eq!(report["reused_existing_identity_count"], 8);
    assert_eq!(report["import_map_entry_count"], 14);

    let import_map_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "query",
            root.to_str().unwrap(),
            "import-map",
        ])
        .expect("CLI should parse"),
    )
    .expect("import-map query should succeed");
    let import_map: serde_json::Value =
        serde_json::from_str(&import_map_output).expect("import-map JSON should parse");
    assert_eq!(import_map["import_map_count"], 14);
    let entries = import_map["entries"]
        .as_object()
        .expect("import-map entries should be an object");
    let active_count = entries
        .values()
        .filter(|entry| entry["status"] == "active")
        .count();
    let missing_count = entries
        .values()
        .filter(|entry| entry["status"] == "missing_in_source")
        .count();
    assert_eq!(active_count, 8);
    assert_eq!(missing_count, 6);
    assert!(
        entries
            .values()
            .all(|entry| entry["source_tool"] == "eagle")
    );
    assert_eagle_source_refs_are_source_facing(entries);
    assert!(entries.values().any(|entry| {
        entry["status"] == "missing_in_source" && entry["source_object_ref"] == "package:ALT-3"
    }));
    assert!(entries.values().any(|entry| {
        entry["status"] == "missing_in_source"
            && entry["source_object_ref"] == "device:ALTAMP:package:ALT-3"
    }));

    let journal = std::fs::read_to_string(root.join(".datum/journal/transactions.jsonl"))
        .expect("transaction journal should exist");
    assert_eq!(
        journal
            .matches("\"kind\":\"create_pool_library_object\"")
            .count(),
        14
    );
    assert_eq!(
        journal
            .matches("\"kind\":\"create_import_map_shard\"")
            .count(),
        2
    );

    let _ = std::fs::remove_dir_all(&root);
}

fn import_map_dir_is_empty(root: &Path) -> bool {
    let import_map_dir = root.join(".datum/import_map");
    match import_map_dir.read_dir() {
        Ok(mut entries) => entries.next().is_none(),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => true,
        Err(error) => panic!("import map directory should be readable: {error}"),
    }
}

fn assert_eagle_source_refs_are_source_facing(
    entries: &serde_json::Map<String, serde_json::Value>,
) {
    for entry in entries.values() {
        let source_ref = entry["source_object_ref"]
            .as_str()
            .expect("source_object_ref should be a string");
        assert!(
            source_ref.contains(':') && Uuid::parse_str(source_ref).is_err(),
            "Eagle source_object_ref should be source-facing, got {source_ref}"
        );
        assert!(entry["source_path"].as_str().is_some());
        assert!(
            entry["source_hash"]
                .as_str()
                .is_some_and(|hash| hash.starts_with("sha256:"))
        );
    }
}
