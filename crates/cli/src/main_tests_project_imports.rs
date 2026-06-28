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

fn write_temp_board(root: &Path, name: &str) -> PathBuf {
    let path = root.join(name);
    std::fs::write(
        &path,
        r#"(kicad_pcb
  (version 20221018)
  (layers
    (0 "F.Cu" signal)
    (31 "B.Cu" signal))
  (net 0 "")
  (net 1 "SIG")
  (footprint "Demo:Mapped"
    (layer "F.Cu")
    (uuid 11111111-1111-1111-1111-111111111111)
    (at 0 0)
    (property "Reference" "U1" (at 0 0 0))
    (property "Value" "Mapped" (at 0 0 0))
    (pad "1" smd rect (at 0 0) (size 1 1) (layers "F.Cu" "F.Paste" "F.Mask") (net 1 "SIG") (uuid 22222222-2222-2222-2222-222222222222)))
  (segment
    (start 1 1)
    (end 5 1)
    (width 0.25)
    (layer "F.Cu")
    (net 1)
    (uuid 33333333-3333-3333-3333-333333333333))
  (via
    (at 2 2)
    (size 0.8)
    (drill 0.4)
    (layers "F.Cu" "B.Cu")
    (net 1)
    (uuid 44444444-4444-4444-4444-444444444444))
  (zone
    (net 1)
    (net_name "SIG")
    (layer "F.Cu")
    (uuid 55555555-5555-5555-5555-555555555555)
    (polygon (pts (xy 0 0) (xy 6 0) (xy 6 6) (xy 0 6)))
    (filled_polygon
      (layer "F.Cu")
      (pts (xy 0 0) (xy 6 0) (xy 6 6) (xy 0 6)))))"#,
    )
    .expect("board should write");
    path
}

#[test]
fn project_import_kicad_board_persists_board_objects_and_import_map() {
    let root = unique_project_root("datum-eda-cli-project-import-kicad-board");
    create_native_project(&root, Some("Import Board Demo".to_string()))
        .expect("initial scaffold should succeed");
    let board = write_temp_board(&root, "native-import.kicad_pcb");

    let output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "import-kicad-board",
            root.to_str().unwrap(),
            "--source",
            board.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("board import should succeed");
    let report: serde_json::Value =
        serde_json::from_str(&output).expect("board import report JSON should parse");
    assert_eq!(report["contract"], "native_project_kicad_board_import_v1");
    assert_eq!(report["imported_package_count"], 1);
    assert_eq!(report["imported_pad_count"], 1);
    assert_eq!(report["imported_track_count"], 1);
    assert_eq!(report["imported_via_count"], 1);
    assert_eq!(report["imported_zone_count"], 1);
    assert_eq!(report["import_map_entry_count"], 5);
    assert!(report["created_object_count"].as_u64().unwrap() >= 5);

    let journal = std::fs::read_to_string(root.join(".datum/journal/transactions.jsonl"))
        .expect("transaction journal should exist");
    assert!(journal.contains("\"kind\":\"create_board_package\""));
    assert!(journal.contains("\"kind\":\"create_board_pad\""));
    assert!(journal.contains("\"kind\":\"create_board_track\""));
    assert!(journal.contains("\"kind\":\"create_board_via\""));
    assert!(journal.contains("\"kind\":\"create_board_zone\""));
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
    assert_eq!(import_map["import_map_count"], 5);
    let entries = import_map["entries"]
        .as_object()
        .expect("import-map entries should be an object");
    assert!(entries.keys().any(|key| key.contains("board-footprint")));
    assert!(entries.keys().any(|key| key.contains("board-pad")));
    assert!(entries.keys().any(|key| key.contains("board-segment")));
    assert!(entries.keys().any(|key| key.contains("board-via")));
    assert!(entries.keys().any(|key| key.contains("board-zone")));
    assert!(entries.values().all(|entry| entry["status"] == "active"));
    assert_kicad_board_source_refs_are_source_facing(entries);

    let second = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "import-kicad-board",
            root.to_str().unwrap(),
            "--source",
            board.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("second board import should succeed");
    let second_report: serde_json::Value =
        serde_json::from_str(&second).expect("second board import report JSON should parse");
    assert_eq!(second_report["import_map_entry_count"], 0);
    assert_eq!(second_report["created_object_count"], 0);
    assert_eq!(second_report["reused_existing_identity_count"], 5);
    let second_journal = std::fs::read_to_string(root.join(".datum/journal/transactions.jsonl"))
        .expect("transaction journal should exist");
    assert_eq!(
        second_journal
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

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_import_kicad_board_reuses_journal_recovered_import_map() {
    let root = unique_project_root("datum-eda-cli-project-import-kicad-board-replay-map");
    create_native_project(&root, Some("Import Board Replay Map Demo".to_string()))
        .expect("initial scaffold should succeed");
    let board = write_temp_board(&root, "native-import-replay.kicad_pcb");

    let first = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "import-kicad-board",
            root.to_str().unwrap(),
            "--source",
            board.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("first board import should succeed");
    let first_report: serde_json::Value =
        serde_json::from_str(&first).expect("first import report JSON should parse");
    let import_map_path = PathBuf::from(
        first_report["import_map_path"]
            .as_str()
            .expect("import map path should be present"),
    );
    std::fs::remove_file(&import_map_path).expect("promoted import-map sidecar should remove");

    let second = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "import-kicad-board",
            root.to_str().unwrap(),
            "--source",
            board.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("second board import should recover import map from journal");
    let second_report: serde_json::Value =
        serde_json::from_str(&second).expect("second import report JSON should parse");
    assert_eq!(second_report["import_map_entry_count"], 0);
    assert_eq!(second_report["created_object_count"], 0);
    assert_eq!(second_report["reused_existing_identity_count"], 5);

    let journal = std::fs::read_to_string(root.join(".datum/journal/transactions.jsonl"))
        .expect("transaction journal should exist");
    assert_eq!(
        journal.matches("\"kind\":\"create_board_package\"").count(),
        1
    );
    assert_eq!(
        journal
            .matches("\"kind\":\"create_import_map_shard\"")
            .count(),
        1
    );

    let _ = std::fs::remove_dir_all(&root);
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
    assert_eq!(import_entry["status"], "active");
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

#[test]
fn project_import_kicad_footprint_reuses_journal_recovered_import_map() {
    let root = unique_project_root("datum-eda-cli-project-import-kicad-footprint-replay-map");
    create_native_project(&root, Some("Import Footprint Replay Map Demo".to_string()))
        .expect("initial scaffold should succeed");
    let footprint = write_temp_footprint(&root, "native-import-replay.kicad_mod");

    let first = execute(
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
    .expect("first footprint import should succeed");
    let first_report: serde_json::Value =
        serde_json::from_str(&first).expect("first import report JSON should parse");
    let package_uuid = first_report["package_uuid"]
        .as_str()
        .expect("package uuid should be present");
    let import_map_path = PathBuf::from(
        first_report["import_map_path"]
            .as_str()
            .expect("import map path should be present"),
    );
    std::fs::remove_file(&import_map_path).expect("promoted import-map sidecar should remove");

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
    .expect("second footprint import should recover import map from journal");
    let second_report: serde_json::Value =
        serde_json::from_str(&second).expect("second import report JSON should parse");
    assert_eq!(second_report["package_uuid"], package_uuid);
    assert_eq!(second_report["reused_existing_identity"], true);
    let journal = std::fs::read_to_string(root.join(".datum/journal/transactions.jsonl"))
        .expect("transaction journal should exist");
    assert_eq!(
        journal.matches("\"kind\":\"create_pool_package\"").count(),
        1
    );

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_import_kicad_footprint_uses_resolver_materialized_pool_refs() {
    let root = unique_project_root("datum-eda-cli-project-import-kicad-footprint-resolver");
    create_native_project(&root, Some("Import Footprint Resolver Demo".to_string()))
        .expect("initial scaffold should succeed");
    let project_json = root.join("project.json");
    let stale_project = std::fs::read_to_string(&project_json).expect("project file should read");
    let first_footprint = write_temp_footprint(&root, "native-import-a.kicad_mod");
    let second_footprint = write_temp_footprint(&root, "native-import-b.kicad_mod");

    execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "import-kicad-footprint",
            root.to_str().unwrap(),
            "--source",
            first_footprint.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("first footprint import should succeed");
    std::fs::write(&project_json, stale_project).expect("stale project file should restore");
    let first_journal = std::fs::read_to_string(root.join(".datum/journal/transactions.jsonl"))
        .expect("transaction journal should exist");
    assert_eq!(
        first_journal
            .matches("\"kind\":\"add_project_pool_ref\"")
            .count(),
        1
    );

    execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "import-kicad-footprint",
            root.to_str().unwrap(),
            "--source",
            second_footprint.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("second footprint import should use resolver-materialized pool refs");
    let second_journal = std::fs::read_to_string(root.join(".datum/journal/transactions.jsonl"))
        .expect("transaction journal should exist");
    assert_eq!(
        second_journal
            .matches("\"kind\":\"add_project_pool_ref\"")
            .count(),
        1
    );
    assert_eq!(
        second_journal
            .matches("\"kind\":\"create_pool_package\"")
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

fn assert_kicad_board_source_refs_are_source_facing(
    entries: &serde_json::Map<String, serde_json::Value>,
) {
    for entry in entries.values() {
        let source_ref = entry["source_object_ref"]
            .as_str()
            .expect("source object ref should be a string");
        assert!(
            source_ref.starts_with("board-"),
            "board import-map source ref should be source-facing: {source_ref}"
        );
        assert!(
            Uuid::parse_str(source_ref).is_err(),
            "board import-map source ref should not be a raw UUID: {source_ref}"
        );
        assert!(
            entry["source_hash"]
                .as_str()
                .is_some_and(|hash| hash.starts_with("sha256:")),
            "board import-map source hash should be present"
        );
        assert!(
            entry["source_path"]
                .as_str()
                .is_some_and(|path| !path.is_empty()),
            "board import-map source path should be present"
        );
    }
}
