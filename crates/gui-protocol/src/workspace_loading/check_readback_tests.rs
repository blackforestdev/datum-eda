//! Actual canonical exclusion over the explicit admitted input; no design writes.
use super::*;
use std::sync::mpsc;

#[test]
#[ignore = "requires explicit admitted DATUM_INSPECTOR_TEST_PROJECT and pinned DATUM_INSPECTOR_CLI"]
fn completion_snapshot_serializes_behind_reader_and_refreshes_tip() {
    let root = std::path::PathBuf::from(
        std::env::var_os("DATUM_INSPECTOR_TEST_PROJECT").expect("admitted input"),
    );
    let source = LiveReviewRequest {
        project_root: root.clone(),
        board_file: None,
        artifact_path: None,
        net_uuid: None,
        from_anchor_pad_uuid: None,
        to_anchor_pad_uuid: None,
        profile: None,
        kicad_board_source: None,
    };
    let mut previous = load_board_editor_workspace_with_model(&source).unwrap().0;
    let request = CheckProducerRequest::from_workspace(&previous).unwrap();
    let output = std::process::Command::new(std::env::var("DATUM_INSPECTOR_CLI").unwrap())
        .args(["--format", "json", "check", "show"])
        .arg(&root)
        .args(["--check-run", "4b0604c3-ec98-524e-8ae7-ebcc7a7f2102"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let run =
        check_run_review_state_from_json(std::str::from_utf8(&output.stdout).unwrap()).unwrap();
    assert!(request.admits_run(&run));
    let model = previous.library.basis.as_ref().unwrap();
    assert_eq!(
        model.model_revision.0,
        "e165c7950ddbcb0d5ac1329aae2672382bab32130536be1d3d98c125eaa88fd3"
    );
    let tip = previous
        .supervision
        .journal
        .accepted_transaction_tip
        .clone();
    assert!(tip.is_some());
    let before: Vec<_> = model
        .source_shards
        .iter()
        .map(|shard| (shard.path.clone(), std::fs::read(&shard.path).unwrap()))
        .collect();
    // Represent the failed native publication's stale UI supervision in memory.
    // Canonical journal/model bytes remain untouched.
    previous.supervision.journal.accepted_transaction_tip = None;
    previous.supervision.journal.applied_transaction_count = 0;
    let mut altered = run.clone();
    altered.finding_count += 1;
    assert!(
        load_check_completion_workspace(previous.clone(), &request, &altered).is_err(),
        "unverified counts cannot become current completion"
    );
    let mut changed_source = previous.clone();
    Arc::make_mut(changed_source.library.basis.as_mut().unwrap())
        .source_shards
        .iter_mut()
        .find(|s| s.kind != eda_engine::substrate::SourceShardKind::CheckRun)
        .unwrap()
        .content_hash
        .push_str("-stale");
    assert!(
        load_check_completion_workspace(changed_source, &request, &run).is_err(),
        "retained view must have identical source identities"
    );
    let lease = eda_engine::revision::ProjectWriteLease::acquire(&root).unwrap();
    let refusal = load_board_editor_workspace_with_model(&source)
        .unwrap_err()
        .to_string();
    assert!(
        refusal.contains("live writer or unavailable lock"),
        "old completion loader reproduces the observed refusal"
    );
    assert!(
        eda_engine::revision::ProjectWriteLease::acquire(&root).is_err(),
        "ordinary writer still refuses"
    );
    let (started, attempt) = mpsc::sync_channel(0);
    let (finished, result) = mpsc::sync_channel(1);
    let worker = std::thread::spawn(move || {
        // Explicit handshake while actual exclusion is held, no sleeps or
        // elapsed-time assertions. The new path must not return a busy error.
        assert!(eda_engine::revision::ProjectWriteLease::acquire(&request.project_root).is_err());
        started.send(()).unwrap();
        finished
            .send(crate::CheckProducerCandidate { run }.readback(previous, &request))
            .unwrap();
    });
    attempt.recv().unwrap();
    assert!(matches!(result.try_recv(), Err(mpsc::TryRecvError::Empty)));
    drop(lease);
    let next = result.recv().unwrap().unwrap();
    worker.join().unwrap();
    assert_eq!(next.supervision.journal.accepted_transaction_tip, tip);
    let model = next.library.basis.as_ref().unwrap();
    assert_eq!(
        next.supervision.journal.applied_transaction_count,
        model.journal_cursor.applied_transaction_count
    );
    assert_eq!(next.checks.findings.len(), next.checks.finding_count);
    assert_eq!(next.checks.finding_count, 13);
    for (path, bytes) in before {
        assert_eq!(std::fs::read(path).unwrap(), bytes);
    }
    assert!(
        eda_engine::revision::ProjectWriteLease::acquire(&root).is_ok(),
        "snapshot guard was released before publication"
    );
}
