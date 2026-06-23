use super::*;

fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}", label, Uuid::new_v4()))
}

fn write_temp_footprint(root: &Path, name: &str) -> PathBuf {
    let path = root.join(name);
    std::fs::write(
        &path,
        r#"(footprint "NativeImportFootprint"
  (layer "F.Cu")
  (fp_line (start -1 -0.8) (end 1 -0.8) (layer "F.SilkS") (width 0.12))
  (pad "1" smd rect (at 0 0) (size 1 1) (layers "F.Cu" "F.Paste" "F.Mask"))
)"#,
    )
    .expect("footprint should write");
    path
}

#[test]
fn project_import_kicad_footprint_persists_pool_and_import_map() {
    let root = unique_project_root("datum-eda-cli-project-import-kicad-footprint");
    create_native_project(&root, Some("Import Footprint Demo".to_string()))
        .expect("initial scaffold should succeed");
    let footprint = write_temp_footprint(&root, "native-import.kicad_mod");

    let output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "import-kicad-footprint",
            root.to_str().unwrap(),
            "--source",
            footprint.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("footprint import should succeed");
    let report: serde_json::Value =
        serde_json::from_str(&output).expect("import report JSON should parse");
    let package_uuid = report["package_uuid"]
        .as_str()
        .expect("package uuid should be present");

    assert_eq!(
        report["contract"],
        "native_project_kicad_footprint_import_v1"
    );
    assert_eq!(report["padstack_count"], 1);
    assert_eq!(report["reused_existing_identity"], false);
    assert!(
        root.join("pool/packages")
            .join(format!("{package_uuid}.json"))
            .exists()
    );
    assert!(
        root.join("pool/padstacks")
            .read_dir()
            .unwrap()
            .next()
            .is_some()
    );
    assert!(
        root.join(".datum/import_map")
            .read_dir()
            .unwrap()
            .next()
            .is_some()
    );
    let journal = std::fs::read_to_string(root.join(".datum/journal/transactions.jsonl"))
        .expect("transaction journal should exist");
    assert!(journal.contains("\"kind\":\"add_project_pool_ref\""));
    assert!(journal.contains("\"kind\":\"create_pool_package\""));
    assert!(journal.contains("\"kind\":\"create_pool_padstack\""));
    assert!(journal.contains("\"kind\":\"create_import_map_shard\""));

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
    assert_eq!(import_map["import_map_count"], 1);
    let import_entry = import_map["entries"][report["import_key"].as_str().unwrap()]
        .as_object()
        .expect("import map entry should be present");
    assert_eq!(import_entry["source_tool"], "kicad");
    assert_eq!(import_entry["source_path"], footprint.display().to_string());
    assert_eq!(import_entry["source_object_ref"], report["import_key"]);
    assert!(
        import_entry["source_hash"]
            .as_str()
            .is_some_and(|value| value.starts_with("sha256:"))
    );

    let second = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "import-kicad-footprint",
            root.to_str().unwrap(),
            "--source",
            footprint.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("second footprint import should succeed");
    let second_report: serde_json::Value =
        serde_json::from_str(&second).expect("second import report JSON should parse");
    assert_eq!(second_report["package_uuid"], package_uuid);
    assert_eq!(second_report["reused_existing_identity"], true);

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
    assert!(
        !root
            .join("pool/packages")
            .join(format!("{package_uuid}.json"))
            .exists()
    );
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

fn import_map_dir_is_empty(root: &Path) -> bool {
    let import_map_dir = root.join(".datum/import_map");
    match import_map_dir.read_dir() {
        Ok(mut entries) => entries.next().is_none(),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => true,
        Err(error) => panic!("import map directory should be readable: {error}"),
    }
}
