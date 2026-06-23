use super::*;

fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}", label, Uuid::new_v4()))
}

fn place_component(root: &Path) -> String {
    let output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "place-board-component",
            root.to_str().unwrap(),
            "--part",
            &Uuid::new_v4().to_string(),
            "--package",
            &Uuid::new_v4().to_string(),
            "--reference",
            "U1",
            "--value",
            "MCU",
            "--x-nm",
            "1000",
            "--y-nm",
            "2000",
            "--layer",
            "1",
        ])
        .expect("CLI should parse"),
    )
    .expect("place component should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).expect("place JSON");
    report["component_uuid"].as_str().unwrap().to_string()
}

fn set_value(root: &Path, component_uuid: &str, value: &str) {
    execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "set-board-component-value",
            root.to_str().unwrap(),
            "--component",
            component_uuid,
            "--value",
            value,
        ])
        .expect("CLI should parse"),
    )
    .expect("set component value should succeed");
}

fn component_value(root: &Path, component_uuid: &str) -> String {
    let board: serde_json::Value = serde_json::from_slice(
        &std::fs::read(root.join("board/board.json")).expect("board should read"),
    )
    .expect("board should parse");
    board["packages"][component_uuid]["value"]
        .as_str()
        .unwrap()
        .to_string()
}

fn journal_list(root: &Path) -> serde_json::Value {
    let output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "query",
            root.to_str().unwrap(),
            "journal-list",
        ])
        .expect("CLI should parse"),
    )
    .expect("journal-list should succeed");
    serde_json::from_str(&output).expect("journal-list JSON")
}

#[test]
fn project_journal_undo_rejects_stale_expected_tip_without_mutation() {
    let root = unique_project_root("datum-eda-cli-project-journal-stale-tip");
    create_native_project(&root, Some("Journal Stale Tip Demo".to_string()))
        .expect("initial scaffold should succeed");
    let component_uuid = place_component(&root);
    set_value(&root, &component_uuid, "MCU-REV2");
    let stale_tip = journal_list(&root)["transactions"][1]["transaction_id"]
        .as_str()
        .unwrap()
        .to_string();
    set_value(&root, &component_uuid, "MCU-REV3");

    let error = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "undo",
            root.to_str().unwrap(),
            "--expected-tip-transaction",
            &stale_tip,
        ])
        .expect("CLI should parse"),
    )
    .expect_err("stale expected tip should fail");

    assert!(error.to_string().contains("expected tip transaction"));
    assert_eq!(component_value(&root, &component_uuid), "MCU-REV3");
    assert_eq!(journal_list(&root)["count"], 3);

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_journal_undo_reports_guard_basis_when_expected_values_match() {
    let root = unique_project_root("datum-eda-cli-project-journal-guard-report");
    create_native_project(&root, Some("Journal Guard Report Demo".to_string()))
        .expect("initial scaffold should succeed");
    let component_uuid = place_component(&root);
    set_value(&root, &component_uuid, "MCU-REV2");
    let journal = journal_list(&root);
    let expected_tip = journal["transactions"][1]["transaction_id"]
        .as_str()
        .unwrap()
        .to_string();
    let expected_revision = journal["model_revision"].as_str().unwrap().to_string();

    let output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "undo",
            root.to_str().unwrap(),
            "--expected-model-revision",
            &expected_revision,
            "--expected-tip-transaction",
            &expected_tip,
        ])
        .expect("CLI should parse"),
    )
    .expect("guarded undo should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).expect("undo JSON");
    assert_eq!(report["guard"]["checked"], true);
    assert_eq!(
        report["guard"]["expected_model_revision"],
        expected_revision
    );
    assert_eq!(report["guard"]["current_model_revision"], expected_revision);
    assert_eq!(report["guard"]["expected_tip_transaction"], expected_tip);
    assert_eq!(report["guard"]["current_tip_transaction"], expected_tip);

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_journal_undo_rejects_unhealthy_cursor_without_mutation() {
    let root = unique_project_root("datum-eda-cli-project-journal-cursor-guard");
    create_native_project(&root, Some("Journal Cursor Guard Demo".to_string()))
        .expect("initial scaffold should succeed");
    let component_uuid = place_component(&root);
    set_value(&root, &component_uuid, "MCU-REV2");
    let journal_before = journal_list(&root)["count"].clone();
    let cursor_path = root.join(".datum/journal/cursor.json");
    std::fs::write(&cursor_path, "{\"applied_transaction_count\":0}\n")
        .expect("stale cursor should write");

    let error = execute(
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
    .expect_err("unhealthy cursor should fail");

    assert!(error.to_string().contains("unhealthy journal cursor"));
    assert!(error.to_string().contains("journal_cursor_behind"));
    assert_eq!(component_value(&root, &component_uuid), "MCU-REV2");
    assert_eq!(journal_list(&root)["count"], journal_before);

    let _ = std::fs::remove_dir_all(&root);
}
