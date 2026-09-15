use super::*;

fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}", label, Uuid::new_v4()))
}

fn write_temp_schematic(root: &Path, name: &str) -> PathBuf {
    let path = root.join(name);
    std::fs::write(
        root.join("child.kicad_sch"),
        r#"(kicad_sch
  (version 20230121)
  (uuid 12345678-1234-1234-1234-123456789abc)
  (hierarchical_label "IN"
    (at 0 0 0)
    (uuid 87654321-4321-4321-4321-cba987654321)))"#,
    )
    .expect("child schematic should write");
    std::fs::write(
        &path,
        r#"(kicad_sch
  (version 20230121)
  (uuid aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa)
  (symbol
    (lib_id "Device:R")
    (at 10 20 0)
    (unit 1)
    (in_bom yes)
    (on_board yes)
    (uuid bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb)
    (property "Reference" "R1" (at 10 20 0))
    (property "Value" "10k" (at 10 22 0)))
  (wire
    (pts (xy 10 20) (xy 20 20))
    (uuid cccccccc-cccc-cccc-cccc-cccccccccccc))
  (junction
    (at 20 20)
    (uuid dddddddd-dddd-dddd-dddd-dddddddddddd))
  (label "SIG"
    (at 20 20 0)
    (uuid eeeeeeee-eeee-eeee-eeee-eeeeeeeeeeee))
  (bus
    (pts (xy 0 0) (xy 10 0))
    (uuid 11111111-2222-3333-4444-555555555555))
  (text "NOTE"
    (at 40 40 0)
    (uuid 33333333-4444-5555-6666-777777777777))
  (rectangle
    (start 40 45)
    (end 50 55)
    (uuid 44444444-5555-6666-7777-888888888888))
  (sheet
    (at 30 30)
    (size 10 10)
    (uuid 22222222-3333-4444-5555-666666666666)
    (property "Sheetname" "Child" (at 30 30 0))
    (property "Sheetfile" "child.kicad_sch" (at 30 32 0))
    (pin "IN" input (at 30 35 180)))
  (no_connect
    (at 10 20)
    (uuid 99999999-9999-9999-9999-999999999999)))"#,
    )
    .expect("schematic should write");
    path
}

fn overwrite_temp_schematic_without_wire(path: &Path) {
    std::fs::write(
        path,
        r#"(kicad_sch
  (version 20230121)
  (uuid aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa)
  (symbol
    (lib_id "Device:R")
    (at 10 20 0)
    (unit 1)
    (in_bom yes)
    (on_board yes)
    (uuid bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb)
    (property "Reference" "R1" (at 10 20 0))
    (property "Value" "10k" (at 10 22 0)))
  (junction
    (at 20 20)
    (uuid dddddddd-dddd-dddd-dddd-dddddddddddd))
  (label "SIG"
    (at 20 20 0)
    (uuid eeeeeeee-eeee-eeee-eeee-eeeeeeeeeeee))
  (bus
    (pts (xy 0 0) (xy 10 0))
    (uuid 11111111-2222-3333-4444-555555555555))
  (text "NOTE"
    (at 40 40 0)
    (uuid 33333333-4444-5555-6666-777777777777))
  (rectangle
    (start 40 45)
    (end 50 55)
    (uuid 44444444-5555-6666-7777-888888888888))
  (sheet
    (at 30 30)
    (size 10 10)
    (uuid 22222222-3333-4444-5555-666666666666)
    (property "Sheetname" "Child" (at 30 30 0))
    (property "Sheetfile" "child.kicad_sch" (at 30 32 0))
    (pin "IN" input (at 30 35 180)))
  (no_connect
    (at 10 20)
    (uuid 99999999-9999-9999-9999-999999999999)))"#,
    )
    .expect("reduced schematic should write");
}

#[test]
fn project_import_kicad_schematic_persists_symbols_and_import_map() {
    let root = unique_project_root("datum-eda-cli-project-import-kicad-schematic");
    create_native_project(&root, Some("Import Schematic Demo".to_string()))
        .expect("initial scaffold should succeed");
    let schematic = write_temp_schematic(&root, "native-import.kicad_sch");

    let output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "import-kicad-schematic",
            root.to_str().unwrap(),
            "--source",
            schematic.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("schematic import should succeed");
    let report: serde_json::Value =
        serde_json::from_str(&output).expect("schematic import report JSON should parse");
    assert_eq!(
        report["contract"],
        "native_project_kicad_schematic_import_v1"
    );
    assert_eq!(report["sheet_created"], true);
    assert_eq!(report["imported_symbol_count"], 1);
    assert_eq!(report["imported_wire_count"], 1);
    assert_eq!(report["imported_junction_count"], 1);
    assert_eq!(report["imported_label_count"], 2);
    assert_eq!(report["imported_bus_count"], 1);
    assert_eq!(report["imported_bus_entry_count"], 0);
    assert_eq!(report["imported_noconnect_count"], 1);
    assert_eq!(report["imported_sheet_count"], 2);
    assert_eq!(report["imported_definition_count"], 1);
    assert_eq!(report["imported_instance_count"], 1);
    assert_eq!(report["imported_port_count"], 1);
    assert_eq!(report["imported_text_count"], 1);
    assert_eq!(report["imported_drawing_count"], 1);
    assert_eq!(report["import_map_entry_count"], 12);
    assert_eq!(report["reused_existing_identity_count"], 0);
    assert!(report["created_object_count"].as_u64().unwrap() >= 2);

    let journal = std::fs::read_to_string(root.join(".datum/journal/transactions.jsonl"))
        .expect("transaction journal should exist");
    assert!(journal.contains("\"kind\":\"create_schematic_sheet\""));
    assert!(journal.contains("\"kind\":\"create_schematic_definition\""));
    assert!(journal.contains("\"kind\":\"create_schematic_sheet_instance\""));
    assert!(journal.contains("\"kind\":\"create_schematic_symbol\""));
    assert!(journal.contains("\"kind\":\"create_schematic_wire\""));
    assert!(journal.contains("\"kind\":\"create_schematic_junction\""));
    assert!(journal.contains("\"kind\":\"create_schematic_label\""));
    assert!(journal.contains("\"kind\":\"create_schematic_port\""));
    assert!(journal.contains("\"kind\":\"create_schematic_bus\""));
    assert!(journal.contains("\"kind\":\"create_schematic_no_connect\""));
    assert!(journal.contains("\"kind\":\"create_schematic_text\""));
    assert!(journal.contains("\"kind\":\"create_schematic_drawing\""));
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
    assert_eq!(import_map["import_map_count"], 12);
    let entries = import_map["entries"]
        .as_object()
        .expect("import-map entries should be an object");
    assert_eq!(entries.len(), 12);
    assert!(entries.values().all(|entry| entry["status"] == "active"));
    assert!(entries.keys().any(|key| key.contains("schematic-symbol")));
    assert!(entries.keys().any(|key| key.contains("schematic-wire")));
    assert!(entries.keys().any(|key| key.contains("schematic-junction")));
    assert!(
        entries
            .keys()
            .any(|key| key.contains("schematic-sheet-definition"))
    );
    assert!(
        entries
            .keys()
            .any(|key| key.contains("schematic-sheet-instance"))
    );
    assert!(
        entries
            .keys()
            .any(|key| key.contains("schematic-sheet-port"))
    );
    assert_eq!(
        entries
            .keys()
            .filter(|key| key.contains("schematic-label"))
            .count(),
        2
    );
    assert!(entries.keys().any(|key| key.contains("schematic-bus")));
    assert!(
        !entries
            .keys()
            .any(|key| key.contains("schematic-bus-entry"))
    );
    assert!(
        entries
            .keys()
            .any(|key| key.contains("schematic-no-connect"))
    );
    assert!(entries.keys().any(|key| key.contains("schematic-text")));
    assert!(entries.keys().any(|key| key.contains("schematic-drawing")));
    assert_kicad_schematic_source_refs_are_source_facing(entries);

    let second = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "import-kicad-schematic",
            root.to_str().unwrap(),
            "--source",
            schematic.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("second schematic import should succeed");
    let second_report: serde_json::Value =
        serde_json::from_str(&second).expect("second schematic import report JSON should parse");
    assert_eq!(second_report["import_map_entry_count"], 0);
    assert_eq!(second_report["created_object_count"], 0);
    assert_eq!(second_report["reused_existing_identity_count"], 12);
    let second_journal = std::fs::read_to_string(root.join(".datum/journal/transactions.jsonl"))
        .expect("transaction journal should exist");
    assert_eq!(
        second_journal
            .matches("\"kind\":\"create_schematic_symbol\"")
            .count(),
        1
    );
    assert_eq!(
        second_journal
            .matches("\"kind\":\"create_schematic_wire\"")
            .count(),
        1
    );
    assert_eq!(
        second_journal
            .matches("\"kind\":\"create_schematic_label\"")
            .count(),
        2
    );
    assert_eq!(
        second_journal
            .matches("\"kind\":\"create_schematic_definition\"")
            .count(),
        1
    );
    assert_eq!(
        second_journal
            .matches("\"kind\":\"create_schematic_sheet_instance\"")
            .count(),
        1
    );
    assert_eq!(
        second_journal
            .matches("\"kind\":\"create_schematic_text\"")
            .count(),
        1
    );
    assert_eq!(
        second_journal
            .matches("\"kind\":\"create_schematic_drawing\"")
            .count(),
        1
    );
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
fn project_import_kicad_schematic_reuses_journal_recovered_import_map() {
    let root = unique_project_root("datum-eda-cli-project-import-kicad-schematic-replay-map");
    create_native_project(&root, Some("Import Schematic Replay Map Demo".to_string()))
        .expect("initial scaffold should succeed");
    let schematic = write_temp_schematic(&root, "native-import-replay.kicad_sch");

    let first = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "import-kicad-schematic",
            root.to_str().unwrap(),
            "--source",
            schematic.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("first schematic import should succeed");
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
            "import-kicad-schematic",
            root.to_str().unwrap(),
            "--source",
            schematic.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("second schematic import should recover import map from journal");
    let second_report: serde_json::Value =
        serde_json::from_str(&second).expect("second import report JSON should parse");
    assert_eq!(second_report["import_map_entry_count"], 0);
    assert_eq!(second_report["created_object_count"], 0);
    assert_eq!(second_report["reused_existing_identity_count"], 12);

    let journal = std::fs::read_to_string(root.join(".datum/journal/transactions.jsonl"))
        .expect("transaction journal should exist");
    assert_eq!(
        journal
            .matches("\"kind\":\"create_schematic_symbol\"")
            .count(),
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
fn project_import_kicad_schematic_marks_absent_source_entries_missing() {
    let root = unique_project_root("datum-eda-cli-project-import-kicad-schematic-missing-source");
    create_native_project(
        &root,
        Some("Import Schematic Missing Source Demo".to_string()),
    )
    .expect("initial scaffold should succeed");
    let schematic = write_temp_schematic(&root, "native-import-missing.kicad_sch");

    execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "import-kicad-schematic",
            root.to_str().unwrap(),
            "--source",
            schematic.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("first schematic import should succeed");
    overwrite_temp_schematic_without_wire(&schematic);

    let reduced_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "import-kicad-schematic",
            root.to_str().unwrap(),
            "--source",
            schematic.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("reduced schematic import should succeed");
    let reduced_report: serde_json::Value =
        serde_json::from_str(&reduced_output).expect("reduced schematic report JSON should parse");
    assert_eq!(reduced_report["imported_wire_count"], 0);
    assert_eq!(reduced_report["created_object_count"], 0);
    assert_eq!(reduced_report["reused_existing_identity_count"], 11);
    assert_eq!(reduced_report["import_map_entry_count"], 12);

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
    let entries = import_map["entries"]
        .as_object()
        .expect("import-map entries should be an object");
    assert_eq!(entries.len(), 12);
    let wire_entry = entries
        .iter()
        .find(|(key, _)| key.contains("schematic-wire"))
        .map(|(_, entry)| entry)
        .expect("old wire import-map entry should remain");
    assert_eq!(wire_entry["status"], "missing_in_source");
    assert!(
        entries
            .iter()
            .filter(|(key, _)| !key.contains("schematic-wire"))
            .all(|(_, entry)| entry["status"] == "active")
    );
    assert_kicad_schematic_source_refs_are_source_facing(entries);

    let journal = std::fs::read_to_string(root.join(".datum/journal/transactions.jsonl"))
        .expect("transaction journal should exist");
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

fn assert_kicad_schematic_source_refs_are_source_facing(
    entries: &serde_json::Map<String, serde_json::Value>,
) {
    for entry in entries.values() {
        let source_ref = entry["source_object_ref"]
            .as_str()
            .expect("source object ref should be a string");
        assert!(
            source_ref.starts_with("schematic-"),
            "schematic import-map source ref should be source-facing: {source_ref}"
        );
        assert!(
            Uuid::parse_str(source_ref).is_err(),
            "schematic import-map source ref should not be a raw UUID: {source_ref}"
        );
        assert!(
            entry["source_hash"]
                .as_str()
                .is_some_and(|hash| hash.starts_with("sha256:")),
            "schematic import-map source hash should be present"
        );
        assert!(
            entry["source_path"]
                .as_str()
                .is_some_and(|path| !path.is_empty()),
            "schematic import-map source path should be present"
        );
    }
}
