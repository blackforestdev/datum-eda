use super::*;

#[test]
fn mixed_family_transaction_undoes_and_redoes_after_reopen() {
    let root = temp_project_root("mixed_family_undo_redo_reopen");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    let sheet_id = Uuid::new_v5(&project_id, b"sheet");
    let wire_id = Uuid::new_v4();
    let track_id = Uuid::new_v4();
    let job_id = Uuid::new_v4();
    write_minimal_project(&root, project_id, board_id);

    let wire = serde_json::json!({
        "uuid": wire_id,
        "from": { "x": 100, "y": 200 },
        "to": { "x": 300, "y": 400 }
    });
    let track = serde_json::json!({
        "uuid": track_id,
        "net": null,
        "layer": 1,
        "width": 125000,
        "start": { "x": 0, "y": 0 },
        "end": { "x": 1000000, "y": 0 }
    });
    let output_job = OutputJob {
        schema_version: PRODUCTION_RECORD_SCHEMA_VERSION,
        id: job_id,
        name: "Mixed family gerbers".to_string(),
        include: vec![ArtifactKind::GerberSet],
        prefix: "mixed".to_string(),
        output_dir: None,
        board_or_panel: board_id,
        variant: None,
        manufacturing_plan: None,
        object_revision: ObjectRevision(0),
    };

    let mut model = ProjectResolver::new(&root)
        .resolve()
        .expect("initial resolve should succeed");
    model
        .commit_journaled(
            &root,
            OperationBatch {
                batch_id: Uuid::new_v5(&project_id, b"mixed-family-create"),
                expected_model_revision: Some(model.model_revision.clone()),
                provenance: CommitProvenance {
                    actor: "unit-test".to_string(),
                    source: CommitSource::Test,
                    reason: "create schematic, pcb, and production objects".to_string(),
                },
                operations: vec![
                    Operation::CreateSchematicWire {
                        sheet_id,
                        wire_id,
                        wire: wire.clone(),
                    },
                    Operation::CreateBoardTrack {
                        track_id,
                        track: track.clone(),
                    },
                    Operation::CreateOutputJob {
                        output_job_id: job_id,
                        output_job: serde_json::to_value(&output_job)
                            .expect("output job should serialize"),
                    },
                ],
            },
        )
        .expect("mixed-family create should commit");

    let sheet_path = root.join("schematic/sheets/main.json");
    let board_path = root.join("board/board.json");
    let job_path = root.join(format!(".datum/output_jobs/{job_id}.json"));
    assert_eq!(
        read_json_value(&sheet_path).expect("sheet should read")["wires"][wire_id.to_string()]["to"]
            ["y"],
        400
    );
    assert_eq!(
        read_json_value(&board_path).expect("board should read")["tracks"][track_id.to_string()]["width"],
        125000
    );
    assert!(job_path.exists());

    let mut reopened = ProjectResolver::new(&root)
        .resolve()
        .expect("reopen after mixed create should replay");
    reopened
        .commit_journal_undo(
            &root,
            CommitProvenance {
                actor: "unit-test".to_string(),
                source: CommitSource::Test,
                reason: "undo mixed-family create after reopen".to_string(),
            },
        )
        .expect("mixed-family undo should commit after reopen");
    assert!(
        read_json_value(&sheet_path).expect("sheet should read after undo")["wires"]
            .as_object()
            .expect("wires should be an object")
            .is_empty()
    );
    assert!(
        read_json_value(&board_path).expect("board should read after undo")["tracks"]
            .as_object()
            .expect("tracks should be an object")
            .is_empty()
    );
    assert!(!job_path.exists());

    let mut reopened_after_undo = ProjectResolver::new(&root)
        .resolve()
        .expect("reopen after mixed undo should replay");
    assert_eq!(
        reopened_after_undo.journal.len(),
        2,
        "reopen after undo must preserve the undo transaction for redo"
    );
    assert_eq!(
        reopened_after_undo.journal[1].transaction_kind,
        TransactionKind::Undo
    );
    reopened_after_undo
        .commit_journal_redo(
            &root,
            CommitProvenance {
                actor: "unit-test".to_string(),
                source: CommitSource::Test,
                reason: "redo mixed-family create after reopen".to_string(),
            },
        )
        .expect("mixed-family redo should commit after reopen");
    assert_eq!(
        read_json_value(&sheet_path).expect("sheet should read after redo")["wires"]
            [wire_id.to_string()]["from"]["x"],
        100
    );
    assert_eq!(
        read_json_value(&board_path).expect("board should read after redo")["tracks"]
            [track_id.to_string()]["end"]["x"],
        1000000
    );
    assert_eq!(
        read_json_value(&job_path).expect("output job should read after redo")["name"],
        "Mixed family gerbers"
    );
}
