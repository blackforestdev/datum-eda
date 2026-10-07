use super::*;
use crate::api::native_write::{commit_prepared, test_support};
use crate::substrate::{CommitProvenance, CommitSource, ProjectResolver};

fn provenance() -> WriteProvenance {
    WriteProvenance::new(
        "profile-test",
        CommitSource::Test,
        "explicit profile-4 adoption",
    )
}

#[test]
fn adoption_records_exact_original_bytes_reopens_and_cannot_downgrade() {
    let (root, mut model, board_id, _) =
        test_support::resolved_model_with_board_package("profile_adoption");
    let path = root.join("board/board.json");
    let original = std::fs::read(&path).unwrap();
    let prepared = build_adopt_board_source_profile(&model, provenance(), board_id).unwrap();
    assert_eq!(
        std::fs::read(&path).unwrap(),
        original,
        "building must not write"
    );
    let report = commit_prepared(&mut model, &root, prepared).unwrap();
    assert!(report.transaction.inverse_operations.is_empty());
    let Operation::AdoptBoardSourceProfile { preimage, .. } = &report.transaction.operations[0]
    else {
        panic!("adoption must be journaled");
    };
    assert_eq!(preimage.as_bytes(), original);
    let adopted = std::fs::read(&path).unwrap();
    let mut expected: serde_json::Value = serde_json::from_slice(&original).unwrap();
    expected["schema_version"] = 4.into();
    assert_eq!(
        adopted,
        format!(
            "{}\n",
            crate::ir::serialization::to_json_deterministic(&expected).unwrap()
        )
        .into_bytes()
    );
    let journal_path = root.join(crate::substrate::JOURNAL_RELATIVE_PATH);
    let journal = std::fs::read(&journal_path).unwrap();
    let mut reopened = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(reopened.model_revision, model.model_revision);
    assert_eq!(
        reopened.journal.last().unwrap().operations,
        report.transaction.operations
    );
    assert!(
        reopened
            .commit_journal_undo(
                &root,
                CommitProvenance {
                    actor: "profile-test".into(),
                    source: CommitSource::Test,
                    reason: "forward-only check".into(),
                }
            )
            .is_err()
    );
    assert!(
        build_adopt_board_source_profile(&reopened, provenance(), board_id)
            .and_then(|p| commit_prepared(&mut reopened, &root, p))
            .is_err()
    );
    assert_eq!(std::fs::read(&path).unwrap(), adopted);
    assert_eq!(std::fs::read(&journal_path).unwrap(), journal);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn adoption_refuses_mixed_batches_unjournaled_use_foreign_identity_and_stale_bytes() {
    let (root, mut model, board_id, package_id) =
        test_support::resolved_model_with_board_package("profile_refusal");
    let path = root.join("board/board.json");
    let bytes = std::fs::read(&path).unwrap();
    let prepared = build_adopt_board_source_profile(&model, provenance(), board_id).unwrap();
    assert!(model.commit(prepared.batch.clone()).is_err());
    let mut mixed = prepared.clone();
    mixed.batch.operations.push(Operation::SetBoardName {
        board_id,
        name: "forbidden mixed edit".into(),
    });
    assert!(commit_prepared(&mut model, &root, mixed).is_err());
    let foreign = build_adopt_board_source_profile(&model, provenance(), package_id).unwrap();
    assert!(commit_prepared(&mut model, &root, foreign).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), bytes);
    let mut external = bytes.clone();
    external.push(b'\n');
    std::fs::write(&path, &external).unwrap();
    assert!(commit_prepared(&mut model, &root, prepared).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), external);
    assert!(model.journal.is_empty());
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn declared_profile_gate_does_not_ignore_graphics_or_future_source() {
    let (root, _, _, _) = test_support::resolved_model_with_board_package("profile_gate");
    let path = root.join("board/board.json");
    let mut value: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    for profile in [1, 2, 3] {
        value["schema_version"] = profile.into();
        value["graphics"] = serde_json::json!({});
        std::fs::write(&path, serde_json::to_vec(&value).unwrap()).unwrap();
        let unavailable = ProjectResolver::new(&root).resolve().unwrap();
        assert!(
            !unavailable
                .source_shards
                .iter()
                .any(|s| s.kind == SourceShardKind::BoardRoot)
        );
        assert!(
            unavailable
                .diagnostics
                .iter()
                .any(|d| d.message.contains("graphics"))
        );
    }
    value["schema_version"] = 4.into();
    let bytes = serde_json::to_vec(&value).unwrap();
    std::fs::write(&path, &bytes).unwrap();
    ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(std::fs::read(&path).unwrap(), bytes);
    value["graphics"] = serde_json::json!({"unknown": {"kind": "unimplemented"}});
    std::fs::write(&path, serde_json::to_vec(&value).unwrap()).unwrap();
    let unavailable = ProjectResolver::new(&root).resolve().unwrap();
    assert!(
        !unavailable
            .source_shards
            .iter()
            .any(|s| s.kind == SourceShardKind::BoardRoot)
    );
    assert!(
        unavailable
            .diagnostics
            .iter()
            .any(|d| d.message.contains("graphics"))
    );
    value["schema_version"] = 5.into();
    value.as_object_mut().unwrap().remove("graphics");
    std::fs::write(&path, serde_json::to_vec(&value).unwrap()).unwrap();
    assert!(ProjectResolver::new(&root).resolve().is_err());
    std::fs::remove_dir_all(root).unwrap();
}
