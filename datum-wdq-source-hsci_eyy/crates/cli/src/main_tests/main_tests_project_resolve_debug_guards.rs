use super::*;
use eda_engine::ir::serialization::to_json_deterministic;
use eda_engine::substrate::{CommitProvenance, CommitSource, Operation, OperationBatch};

fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}", label, Uuid::new_v4()))
}

fn read_project_core_files(root: &Path) -> Vec<(PathBuf, Vec<u8>)> {
    [
        "project.json",
        "schematic/schematic.json",
        "board/board.json",
        "rules/rules.json",
    ]
    .into_iter()
    .map(|relative| {
        let path = root.join(relative);
        let bytes = std::fs::read(&path).expect("project core file should read");
        (path, bytes)
    })
    .collect()
}

fn write_batch(root: &Path, batch: &OperationBatch, name: &str) -> PathBuf {
    let batch_path = root.join(name);
    std::fs::write(
        &batch_path,
        to_json_deterministic(batch).expect("batch should serialize"),
    )
    .expect("batch should write");
    batch_path
}

fn resolve_debug_apply(root: &Path, batch_path: &Path) -> serde_json::Value {
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
    .expect("resolve-debug apply should succeed");
    serde_json::from_str(&output).expect("resolve-debug output should parse")
}

#[test]
fn project_query_resolve_debug_commit_batch_apply_rejects_missing_expected_revision() {
    let root = unique_project_root("datum-eda-cli-project-resolve-debug-commit-missing-revision");
    create_native_project(
        &root,
        Some("Resolve Debug Missing Revision Demo".to_string()),
    )
    .expect("initial scaffold should succeed");
    let before = read_project_core_files(&root);
    let board: serde_json::Value = serde_json::from_slice(
        &std::fs::read(root.join("board/board.json")).expect("board should read"),
    )
    .expect("board should parse");
    let batch_path = root.join("commit-batch-missing-revision.json");
    let batch = OperationBatch {
        batch_id: Uuid::new_v4(),
        expected_model_revision: None,
        provenance: CommitProvenance {
            actor: "cli-test".to_string(),
            source: CommitSource::Cli,
            reason: "prove CLI substrate journal guard".to_string(),
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

    let error = execute(
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
    .expect_err("missing expected revision should fail");

    assert!(
        error
            .to_string()
            .contains("journaled commit requires expected_model_revision")
    );
    assert!(!root.join(".datum/journal/transactions.jsonl").exists());
    for (path, bytes) in before {
        assert_eq!(
            std::fs::read(&path).expect("project core file should read after rejected apply"),
            bytes,
            "rejected commit apply must not rewrite {}",
            path.display()
        );
    }

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_query_resolve_debug_write_boundary_uses_operation_vocabulary() {
    let root = unique_project_root("datum-eda-cli-project-resolve-debug-write-boundary");
    create_native_project(&root, Some("Resolve Debug Boundary Demo".to_string()))
        .expect("initial scaffold should succeed");
    let model = eda_engine::substrate::ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves");
    let project_id = model.project.project_id;
    let batch = OperationBatch {
        batch_id: Uuid::new_v4(),
        expected_model_revision: Some(model.model_revision),
        provenance: CommitProvenance {
            actor: "cli-test".to_string(),
            source: CommitSource::Cli,
            reason: "prove operation vocabulary write boundary".to_string(),
        },
        operations: vec![Operation::SetBoardName {
            board_id: project_id,
            name: "Boundary Board".to_string(),
        }],
    };
    let batch_path = write_batch(&root, &batch, "commit-batch-set-board-name.json");
    let report = resolve_debug_apply(&root, &batch_path);
    assert_eq!(
        report["write_boundary"],
        "journal_and_project_shards_written"
    );

    let root = unique_project_root("datum-eda-cli-project-resolve-debug-schematic-definition");
    create_native_project(
        &root,
        Some("Resolve Debug Schematic Definition Demo".to_string()),
    )
    .expect("initial scaffold should succeed");
    let model = eda_engine::substrate::ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves");
    let definition_id = Uuid::new_v4();
    let schematic_root: serde_json::Value =
        serde_json::from_slice(&std::fs::read(root.join("schematic/schematic.json")).unwrap())
            .unwrap();
    let schematic_id =
        Uuid::parse_str(schematic_root["uuid"].as_str().expect("schematic uuid")).unwrap();
    let batch = OperationBatch {
        batch_id: Uuid::new_v4(),
        expected_model_revision: Some(model.model_revision),
        provenance: CommitProvenance {
            actor: "cli-test".to_string(),
            source: CommitSource::Cli,
            reason: "prove schematic definition write boundary".to_string(),
        },
        operations: vec![Operation::CreateSchematicDefinition {
            schematic_id,
            definition_id,
            relative_path: format!("definitions/{definition_id}.json"),
            definition: serde_json::json!({
                "schema_version": 1,
                "uuid": definition_id,
                "name": "Definition A"
            }),
        }],
    };
    let batch_path = write_batch(&root, &batch, "commit-batch-create-definition.json");
    let report = resolve_debug_apply(&root, &batch_path);
    assert_eq!(
        report["write_boundary"],
        "journal_and_project_shards_written"
    );

    let root = unique_project_root("datum-eda-cli-project-resolve-debug-journal-only");
    create_native_project(&root, Some("Resolve Debug Journal Only Demo".to_string()))
        .expect("initial scaffold should succeed");
    let model = eda_engine::substrate::ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves");
    let batch = OperationBatch {
        batch_id: Uuid::new_v4(),
        expected_model_revision: Some(model.model_revision),
        provenance: CommitProvenance {
            actor: "cli-test".to_string(),
            source: CommitSource::Cli,
            reason: "prove journal-only operation boundary".to_string(),
        },
        operations: vec![Operation::BumpObjectRevision {
            object_id: model.project.project_id,
        }],
    };
    let batch_path = write_batch(&root, &batch, "commit-batch-bump-object.json");
    let report = resolve_debug_apply(&root, &batch_path);
    assert_eq!(
        report["write_boundary"],
        "journal_only_no_project_shards_written"
    );
}
