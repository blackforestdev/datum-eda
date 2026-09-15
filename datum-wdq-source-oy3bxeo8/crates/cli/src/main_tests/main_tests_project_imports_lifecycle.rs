use super::*;

fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}", label, Uuid::new_v4()))
}

fn write_board_with_segment(path: &Path) {
    std::fs::write(
        path,
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
}

fn overwrite_board_without_segment(path: &Path) {
    std::fs::write(
        path,
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
    .expect("reduced board should write");
}

#[test]
fn project_import_kicad_board_marks_absent_source_entries_missing() {
    let root = unique_project_root("datum-eda-cli-project-import-kicad-board-missing-source");
    create_native_project(&root, Some("Import Board Missing Source Demo".to_string()))
        .expect("initial scaffold should succeed");
    let board = root.join("native-import-missing.kicad_pcb");
    write_board_with_segment(&board);

    execute(
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
    overwrite_board_without_segment(&board);

    let reduced_output = execute(
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
    .expect("reduced board import should succeed");
    let reduced_report: serde_json::Value =
        serde_json::from_str(&reduced_output).expect("reduced board report JSON should parse");
    assert_eq!(reduced_report["imported_track_count"], 0);
    assert_eq!(reduced_report["created_object_count"], 0);
    assert_eq!(reduced_report["reused_existing_identity_count"], 4);
    assert_eq!(reduced_report["import_map_entry_count"], 5);

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
    assert_eq!(entries.len(), 5);
    let segment_entry = entries
        .iter()
        .find(|(key, _)| key.contains("board-segment"))
        .map(|(_, entry)| entry)
        .expect("old segment import-map entry should remain");
    assert_eq!(segment_entry["status"], "missing_in_source");
    assert!(
        entries
            .iter()
            .filter(|(key, _)| !key.contains("board-segment"))
            .all(|(_, entry)| entry["status"] == "active")
    );

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
