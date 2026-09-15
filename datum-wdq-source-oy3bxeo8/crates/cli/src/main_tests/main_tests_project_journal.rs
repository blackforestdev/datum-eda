use super::*;
use eda_engine::ir::serialization::to_json_deterministic;
use eda_engine::substrate::{
    CommitProvenance, CommitSource, ModelRevision, Operation, OperationBatch,
};

fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}", label, Uuid::new_v4()))
}

fn create_journaled_test_transaction(root: &Path) -> serde_json::Value {
    let board: serde_json::Value = serde_json::from_slice(
        &std::fs::read(root.join("board/board.json")).expect("board should read"),
    )
    .expect("board should parse");
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
    .expect("project query resolve-debug should succeed");
    let resolve_report: serde_json::Value =
        serde_json::from_str(&resolve_output).expect("resolve-debug JSON should parse");
    let batch_path = root.join("journal-test-batch.json");
    let batch = OperationBatch {
        batch_id: Uuid::new_v4(),
        expected_model_revision: Some(ModelRevision(
            resolve_report["model_revision"]
                .as_str()
                .expect("model revision should exist")
                .to_string(),
        )),
        provenance: CommitProvenance {
            actor: "cli-test".to_string(),
            source: CommitSource::Cli,
            reason: "prove CLI journal query surface".to_string(),
        },
        operations: vec![Operation::BumpObjectRevision {
            object_id: Uuid::parse_str(board["uuid"].as_str().expect("board uuid should exist"))
                .expect("board uuid should parse"),
        }],
    };
    std::fs::write(
        &batch_path,
        to_json_deterministic(&batch).expect("batch should serialize"),
    )
    .expect("batch should write");

    let output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "query",
            root.to_str().unwrap(),
            "resolve-debug",
            "--commit-batch",
            batch_path.to_str().unwrap(),
            "--apply",
        ])
        .expect("CLI should parse"),
    )
    .expect("project query resolve-debug --commit-batch --apply should succeed");
    serde_json::from_str(&output).expect("commit apply debug JSON should parse")
}

fn place_journal_test_component(root: &Path) -> String {
    let part_uuid = Uuid::new_v4();
    let package_uuid = Uuid::new_v4();
    let output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "place-board-component",
            root.to_str().unwrap(),
            "--part",
            &part_uuid.to_string(),
            "--package",
            &package_uuid.to_string(),
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
    .expect("place board component should succeed");
    let report: serde_json::Value =
        serde_json::from_str(&output).expect("place output should parse");
    report["component_uuid"]
        .as_str()
        .expect("component uuid should exist")
        .to_string()
}

fn set_journal_test_component_value(root: &Path, component_uuid: &str, value: &str) {
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
    .expect("set board component value should succeed");
}

fn board_component_value(root: &Path, component_uuid: &str) -> String {
    let board: serde_json::Value = serde_json::from_slice(
        &std::fs::read(root.join("board/board.json")).expect("board should read"),
    )
    .expect("board should parse");
    board["packages"][component_uuid]["value"]
        .as_str()
        .expect("component value should exist")
        .to_string()
}

#[test]
fn journal_list_and_show_commands_use_resolver_backed_journal() {
    let root = unique_project_root("datum-eda-cli-journal-list-show");
    create_native_project(&root, Some("Canonical Journal Demo".to_string()))
        .expect("initial scaffold should succeed");
    let commit_report = create_journaled_test_transaction(&root);
    let transaction_id = commit_report["transaction"]["transaction_id"]
        .as_str()
        .expect("transaction id should exist");

    let list_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "journal",
            "list",
            root.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("journal list should succeed");
    let list: serde_json::Value =
        serde_json::from_str(&list_output).expect("journal-list JSON should parse");
    assert_eq!(list["contract"], "project_transaction_journal_list_v1");
    assert_eq!(list["count"], 1);
    assert_eq!(list["transactions"][0]["transaction_id"], transaction_id);

    let show_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "journal",
            "show",
            root.to_str().unwrap(),
            "--transaction",
            transaction_id,
        ])
        .expect("CLI should parse"),
    )
    .expect("journal show should succeed");
    let show: serde_json::Value =
        serde_json::from_str(&show_output).expect("journal-show JSON should parse");
    assert_eq!(show["contract"], "project_transaction_journal_record_v1");
    assert_eq!(show["transaction"], commit_report["transaction"]);

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn journal_undo_and_redo_commands_append_compensating_transactions() {
    let root = unique_project_root("datum-eda-cli-journal-undo-redo");
    create_native_project(&root, Some("Canonical Journal Undo Redo Demo".to_string()))
        .expect("initial scaffold should succeed");
    let component_uuid = place_journal_test_component(&root);
    set_journal_test_component_value(&root, &component_uuid, "MCU-REV2");

    let undo_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "journal",
            "undo",
            root.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("journal undo should succeed");
    let undo: serde_json::Value =
        serde_json::from_str(&undo_output).expect("undo JSON should parse");
    assert_eq!(undo["contract"], "project_transaction_journal_mutation_v1");
    assert_eq!(undo["action"], "undo");
    assert_eq!(undo["status"], "applied");
    assert_eq!(undo["guard"]["checked"], false);
    assert_eq!(
        undo["guard"]["current_model_revision"],
        undo["before_model_revision"]
    );
    assert_eq!(board_component_value(&root, &component_uuid), "MCU");

    let redo_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "journal",
            "redo",
            root.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("journal redo should succeed");
    let redo: serde_json::Value =
        serde_json::from_str(&redo_output).expect("redo JSON should parse");
    assert_eq!(redo["contract"], "project_transaction_journal_mutation_v1");
    assert_eq!(redo["action"], "redo");
    assert_eq!(redo["status"], "applied");
    assert_eq!(board_component_value(&root, &component_uuid), "MCU-REV2");

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_query_journal_list_reports_empty_and_journaled_transactions() {
    let root = unique_project_root("datum-eda-cli-project-journal-list");
    create_native_project(&root, Some("Journal List Demo".to_string()))
        .expect("initial scaffold should succeed");

    let empty_output = execute(
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
    .expect("project query journal-list should succeed");
    let empty_report: serde_json::Value =
        serde_json::from_str(&empty_output).expect("journal-list JSON should parse");
    assert_eq!(
        empty_report["contract"],
        "project_transaction_journal_list_v1"
    );
    assert_eq!(
        empty_report["journal_path"],
        ".datum/journal/transactions.jsonl"
    );
    assert_eq!(empty_report["cursor_path"], ".datum/journal/cursor.json");
    assert_eq!(empty_report["count"], 0);
    assert_eq!(empty_report["cursor_index"], 0);
    assert_eq!(empty_report["can_undo"], false);
    assert_eq!(empty_report["can_redo"], false);
    assert_eq!(empty_report["transactions"].as_array().unwrap().len(), 0);

    let commit_report = create_journaled_test_transaction(&root);
    let transaction_id = commit_report["transaction"]["transaction_id"]
        .as_str()
        .expect("transaction id should exist");
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
    .expect("project query journal-list should succeed");
    let report: serde_json::Value =
        serde_json::from_str(&output).expect("journal-list JSON should parse");

    assert_eq!(report["count"], 1);
    assert_eq!(report["cursor_index"], 1);
    assert_eq!(report["can_undo"], false);
    assert_eq!(report["can_redo"], false);
    assert_eq!(report["transactions"][0]["transaction_id"], transaction_id);
    assert_eq!(report["transactions"][0]["actor"], "cli-test");
    assert_eq!(report["transactions"][0]["source"], "cli");
    assert_eq!(
        report["transactions"][0]["reason"],
        "prove CLI journal query surface"
    );
    assert_eq!(report["transactions"][0]["modified"], 1);
    assert_eq!(report["transactions"][0]["operations"], 1);

    let text_output = execute(
        Cli::try_parse_from([
            "eda",
            "project",
            "query",
            root.to_str().unwrap(),
            "journal-list",
        ])
        .expect("CLI should parse"),
    )
    .expect("project query journal-list text should succeed");
    assert!(text_output.contains("project_transaction_journal_list_v1"));
    assert!(text_output.contains(transaction_id));
    assert!(root.join(".datum/journal/cursor.json").exists());

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_journal_undo_and_redo_append_compensating_transactions() {
    let root = unique_project_root("datum-eda-cli-project-journal-undo-redo");
    create_native_project(&root, Some("Journal Undo Redo Demo".to_string()))
        .expect("initial scaffold should succeed");
    let component_uuid = place_journal_test_component(&root);
    set_journal_test_component_value(&root, &component_uuid, "MCU-REV2");
    assert_eq!(
        board_component_value(&root, &component_uuid),
        "MCU-REV2".to_string()
    );
    let list_output = execute(
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
    let list: serde_json::Value =
        serde_json::from_str(&list_output).expect("journal-list JSON should parse");
    assert_eq!(list["count"], 2);
    let original_transaction_id = list["transactions"][1]["transaction_id"]
        .as_str()
        .expect("original transaction id should exist")
        .to_string();

    let undo_output = execute(
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
    let undo: serde_json::Value =
        serde_json::from_str(&undo_output).expect("undo JSON should parse");
    assert_eq!(undo["contract"], "project_transaction_journal_mutation_v1");
    assert_eq!(undo["action"], "undo");
    assert_eq!(undo["status"], "applied");
    assert_eq!(undo["journal_len"], 3);
    assert_eq!(undo["cursor_before"], 2);
    assert_eq!(undo["cursor_after"], 3);
    assert_eq!(undo["can_undo"], false);
    assert_eq!(undo["can_redo"], true);
    assert_eq!(undo["transaction"]["transaction_kind"], "undo");
    assert_eq!(undo["transaction"]["undo_of"], original_transaction_id);
    assert!(undo["transaction"].get("redo_of").is_none());
    assert_eq!(
        board_component_value(&root, &component_uuid),
        "MCU".to_string()
    );

    let redo_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "redo",
            root.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("project redo should succeed");
    let redo: serde_json::Value =
        serde_json::from_str(&redo_output).expect("redo JSON should parse");
    assert_eq!(redo["contract"], "project_transaction_journal_mutation_v1");
    assert_eq!(redo["action"], "redo");
    assert_eq!(redo["status"], "applied");
    assert_eq!(redo["journal_len"], 4);
    assert_eq!(redo["cursor_before"], 3);
    assert_eq!(redo["cursor_after"], 4);
    assert_eq!(redo["can_undo"], true);
    assert_eq!(redo["can_redo"], false);
    assert_eq!(redo["transaction"]["transaction_kind"], "redo");
    assert_eq!(
        redo["transaction"]["redo_of"],
        undo["transaction"]["transaction_id"]
    );
    assert!(redo["transaction"].get("undo_of").is_none());
    assert_eq!(
        board_component_value(&root, &component_uuid),
        "MCU-REV2".to_string()
    );

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_journal_normal_commit_after_undo_clears_redo() {
    let root = unique_project_root("datum-eda-cli-project-journal-branch-clears-redo");
    create_native_project(&root, Some("Journal Branch Clears Redo Demo".to_string()))
        .expect("initial scaffold should succeed");
    let component_uuid = place_journal_test_component(&root);
    set_journal_test_component_value(&root, &component_uuid, "MCU-REV2");
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
    set_journal_test_component_value(&root, &component_uuid, "MCU-BRANCH");

    let journal_output = execute(
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
    let journal: serde_json::Value =
        serde_json::from_str(&journal_output).expect("journal-list JSON should parse");
    assert_eq!(journal["count"], 4);
    assert_eq!(journal["can_undo"], true);
    assert_eq!(journal["can_redo"], false);
    assert_eq!(
        board_component_value(&root, &component_uuid),
        "MCU-BRANCH".to_string()
    );

    let error = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "redo",
            root.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect_err("project redo should fail after branch commit");
    assert!(
        error
            .to_string()
            .contains("redo stack was cleared by a newer normal transaction")
    );
    assert_eq!(
        board_component_value(&root, &component_uuid),
        "MCU-BRANCH".to_string()
    );

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_query_resolve_debug_reports_invalid_journal_cursor() {
    let root = unique_project_root("datum-eda-cli-project-journal-cursor-invalid");
    create_native_project(&root, Some("Journal Cursor Invalid Demo".to_string()))
        .expect("initial scaffold should succeed");
    create_journaled_test_transaction(&root);
    std::fs::write(
        root.join(".datum/journal/cursor.json"),
        "{\"applied_transaction_count\":99}\n",
    )
    .expect("cursor should write");

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
        report["diagnostics"]
            .as_array()
            .unwrap()
            .iter()
            .any(|entry| { entry["code"] == "journal_cursor_out_of_range" })
    );

    let journal_output = execute(
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
    .expect("project query journal-list should succeed");
    let journal: serde_json::Value =
        serde_json::from_str(&journal_output).expect("journal-list JSON should parse");
    assert_eq!(journal["count"], 1);
    assert_eq!(journal["cursor_index"], 1);
    assert_eq!(journal["can_undo"], false);
    assert_eq!(journal["can_redo"], false);

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_query_journal_list_normalizes_stale_cursor_behind_tip() {
    let root = unique_project_root("datum-eda-cli-project-journal-cursor-behind");
    create_native_project(&root, Some("Journal Cursor Behind Demo".to_string()))
        .expect("initial scaffold should succeed");
    let component_uuid = place_journal_test_component(&root);
    set_journal_test_component_value(&root, &component_uuid, "MCU-REV2");
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
    std::fs::write(
        root.join(".datum/journal/cursor.json"),
        "{\"applied_transaction_count\":1}\n",
    )
    .expect("cursor should write");

    let debug_output = execute(
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
    let debug: serde_json::Value =
        serde_json::from_str(&debug_output).expect("resolve-debug JSON should parse");
    assert!(
        debug["diagnostics"]
            .as_array()
            .unwrap()
            .iter()
            .any(|entry| { entry["code"] == "journal_cursor_behind" })
    );

    let journal_output = execute(
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
    .expect("project query journal-list should succeed");
    let journal: serde_json::Value =
        serde_json::from_str(&journal_output).expect("journal-list JSON should parse");
    assert_eq!(journal["count"], 3);
    assert_eq!(journal["cursor_index"], 3);
    assert_eq!(journal["can_undo"], false);
    assert_eq!(journal["can_redo"], true);

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_query_journal_show_reports_full_transaction() {
    let root = unique_project_root("datum-eda-cli-project-journal-show");
    create_native_project(&root, Some("Journal Show Demo".to_string()))
        .expect("initial scaffold should succeed");
    let commit_report = create_journaled_test_transaction(&root);
    let transaction_id = commit_report["transaction"]["transaction_id"]
        .as_str()
        .expect("transaction id should exist");

    let output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "query",
            root.to_str().unwrap(),
            "journal-show",
            "--transaction",
            transaction_id,
        ])
        .expect("CLI should parse"),
    )
    .expect("project query journal-show should succeed");
    let report: serde_json::Value =
        serde_json::from_str(&output).expect("journal-show JSON should parse");

    assert_eq!(report["contract"], "project_transaction_journal_record_v1");
    assert_eq!(report["journal_path"], ".datum/journal/transactions.jsonl");
    assert_eq!(report["index"], 0);
    assert_eq!(report["transaction"], commit_report["transaction"]);

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_query_journal_show_reports_missing_transaction() {
    let root = unique_project_root("datum-eda-cli-project-journal-show-missing");
    create_native_project(&root, Some("Journal Show Missing Demo".to_string()))
        .expect("initial scaffold should succeed");
    let missing = Uuid::new_v4().to_string();

    let error = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "query",
            root.to_str().unwrap(),
            "journal-show",
            "--transaction",
            &missing,
        ])
        .expect("CLI should parse"),
    )
    .expect_err("missing journal transaction should fail");
    assert!(error.to_string().contains(&format!(
        "transaction {missing} not found in project journal"
    )));

    let _ = std::fs::remove_dir_all(&root);
}
