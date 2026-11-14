//! Synthetic hostile-history controls supplement the native move/cold workflow.
use super::*;
use crate::substrate::{CommitDiff, CommitProvenance, CommitSource, ModelRevision};
use serde_json::json;

fn append(board: &mut Value, journal: &mut Vec<TransactionRecord>, op: Operation) {
    let mut inverse = Vec::new();
    crate::substrate::board_journal_ops::inverse_board_operation(board, &op, &mut inverse).unwrap();
    let created = matches!(op, Operation::CreateBoardPackage { .. });
    let id = match &op {
        Operation::CreateBoardPackage { package_id, .. }
        | Operation::SetBoardPackagePosition { package_id, .. }
        | Operation::SetBoardPackageRotation { package_id, .. }
        | Operation::SetBoardPackageValue { package_id, .. } => *package_id,
        _ => panic!("test operation"),
    };
    journal.push(TransactionRecord {
        transaction_id: Uuid::new_v4(),
        batch_id: Uuid::new_v4(),
        transaction_kind: TransactionKind::Normal,
        undo_of: None,
        redo_of: None,
        before_model_revision: ModelRevision(format!("test-{}", journal.len())),
        after_model_revision: ModelRevision(format!("test-{}", journal.len() + 1)),
        provenance: CommitProvenance {
            actor: "history-test".into(),
            source: CommitSource::Manual,
            reason: "Synthetic transcript".into(),
        },
        agent_provenance: None,
        diff: CommitDiff {
            created: if created { vec![id] } else { vec![] },
            modified: if created { vec![] } else { vec![id] },
            deleted: vec![],
        },
        operations: vec![op],
        inverse_operations: inverse,
    });
}

fn fixture() -> (Value, Vec<TransactionRecord>, RevisionedRef) {
    let id = Uuid::new_v4();
    let mut board = json!({"packages":{},"pads":{}});
    let mut journal = Vec::new();
    append(
        &mut board,
        &mut journal,
        Operation::CreateBoardPackage {
            package_id: id,
            materialized: json!({}),
            package: json!({"uuid":id,"part":Uuid::new_v4(),"package":Uuid::new_v4(),"reference":"R1","value":"10k","position":{"x":0,"y":0},"rotation":0,"layer":1,"locked":false}),
        },
    );
    append(
        &mut board,
        &mut journal,
        Operation::SetBoardPackagePosition {
            package_id: id,
            x: 2_000_000,
            y: 3_000_000,
        },
    );
    append(
        &mut board,
        &mut journal,
        Operation::SetBoardPackageRotation {
            package_id: id,
            rotation: 90,
        },
    );
    (
        board,
        journal,
        RevisionedRef {
            object_id: id,
            object_revision: ObjectRevision(0),
        },
    )
}

#[test]
fn declared_anchor_requires_exact_pose_preimages_and_real_counters() {
    let (board, journal, reference) = fixture();
    assert!(qualifies(&board, &journal, &reference, ObjectRevision(2)).unwrap());
    let mut forged = journal.clone();
    forged[1].inverse_operations = vec![Operation::SetBoardPackagePosition {
        package_id: reference.object_id,
        x: 1,
        y: 1,
    }];
    assert!(qualifies(&board, &forged, &reference, ObjectRevision(2)).is_err());
    assert!(!qualifies(&board, &journal, &reference, ObjectRevision(3)).unwrap());
    let mut tampered = board.clone();
    tampered["packages"][reference.object_id.to_string()]["part"] = json!(Uuid::new_v4());
    assert!(!qualifies(&tampered, &journal, &reference, ObjectRevision(2)).unwrap());
}

#[test]
fn missing_birth_recreation_and_unlinked_compensation_never_bridge_a_stale_ref() {
    let (board, journal, reference) = fixture();
    assert!(qualifies(&board, &journal[1..], &reference, ObjectRevision(2)).is_err());
    let mut duplicate = journal.clone();
    duplicate.push(journal[0].clone());
    assert!(qualifies(&board, &duplicate, &reference, ObjectRevision(2)).is_err());
    let mut unknown = journal.clone();
    unknown[2].transaction_kind = TransactionKind::Undo;
    unknown[2].undo_of = Some(Uuid::new_v4());
    assert!(qualifies(&board, &unknown, &reference, ObjectRevision(2)).is_err());
    let mut missing_inverse = journal.clone();
    missing_inverse[2].inverse_operations.clear();
    assert!(qualifies(&board, &missing_inverse, &reference, ObjectRevision(2)).is_err());
    let mut bump = journal.clone();
    bump[2].operations = vec![Operation::BumpObjectRevision {
        object_id: reference.object_id,
    }];
    assert!(qualifies(&board, &bump, &reference, ObjectRevision(2)).is_err());
}

#[test]
fn pre_anchor_metadata_requires_its_recorded_review_receipt() {
    let (mut board, mut journal, mut reference) = fixture();
    append(
        &mut board,
        &mut journal,
        Operation::SetBoardPackageValue {
            package_id: reference.object_id,
            value: "20k".into(),
        },
    );
    reference.object_revision = ObjectRevision(3);
    append(
        &mut board,
        &mut journal,
        Operation::SetBoardPackageRotation {
            package_id: reference.object_id,
            rotation: 180,
        },
    );
    assert!(qualifies(&board, &journal, &reference, ObjectRevision(4)).is_err());
}

#[test]
fn real_object_collector_map_kinds_use_source_membership_for_pose_authority() {
    use crate::substrate::{SourceShardAuthority, SourceShardDirtyState, SourceShardRef};
    let (board, journal, reference) = fixture();
    let path = std::env::temp_dir().join(format!("datum-pose-source-{}.json", Uuid::new_v4()));
    let bytes = serde_json::to_vec(&board).unwrap();
    std::fs::write(&path, &bytes).unwrap();
    let shard = SourceShardRef {
        shard_id: Uuid::new_v4(),
        kind: SourceShardKind::BoardRoot,
        taxon: None,
        path: path.clone(),
        relative_path: "board.json".into(),
        authority: SourceShardAuthority::AuthoredDesign,
        dirty_state: SourceShardDirtyState::Clean,
        schema_version: None,
        content_hash: crate::substrate::sha256_hex(&bytes),
    };
    let mut objects = Default::default();
    crate::substrate::collect_uuid_objects(
        &board,
        &shard,
        "board",
        &mut objects,
        &mut Default::default(),
    );
    let object = objects.get_mut(&reference.object_id).unwrap();
    assert_eq!(object.kind, reference.object_id.to_string());
    object.object_revision = ObjectRevision(2);
    let history = PoolHistory {
        shards: std::slice::from_ref(&shard),
        journal: &journal,
    };
    assert!(matches(&history, object, &reference).unwrap());
    let mut foreign = object.clone();
    foreign.domain = "schematic".into();
    assert!(!matches(&history, &foreign, &reference).unwrap());
    foreign = object.clone();
    foreign.source_shard_id = Uuid::new_v4();
    assert!(matches(&history, &foreign, &reference).is_err());
    let mut unrelated = board;
    unrelated["packages"].as_object_mut().unwrap().clear();
    let bytes = serde_json::to_vec(&unrelated).unwrap();
    std::fs::write(&path, &bytes).unwrap();
    let mut changed = shard;
    changed.content_hash = crate::substrate::sha256_hex(&bytes);
    let history = PoolHistory {
        shards: std::slice::from_ref(&changed),
        journal: &journal,
    };
    assert!(matches(&history, object, &reference).is_err());
    std::fs::remove_file(path).unwrap();
}

#[test]
fn canonical_package_field_advances_survive_deduplicated_cold_diff() {
    use crate::substrate::{SourceShardAuthority, SourceShardDirtyState, SourceShardRef};
    let (board, original_journal, reference) = fixture();
    let shard = SourceShardRef {
        shard_id: Uuid::new_v4(),
        kind: SourceShardKind::BoardRoot,
        taxon: None,
        path: "board.json".into(),
        relative_path: "board.json".into(),
        authority: SourceShardAuthority::AuthoredDesign,
        dirty_state: SourceShardDirtyState::Clean,
        schema_version: None,
        content_hash: String::new(),
    };
    let mut source = std::collections::BTreeMap::new();
    crate::substrate::collect_uuid_objects(
        &board,
        &shard,
        "board",
        &mut source,
        &mut Default::default(),
    );
    for extra in [
        None,
        Some(Operation::SetBoardPackageRotation {
            package_id: reference.object_id,
            rotation: 180,
        }),
        Some(Operation::BumpObjectRevision {
            object_id: reference.object_id,
        }),
    ] {
        let mut warm = source.clone();
        crate::substrate::replay_objects::apply_promoted_journal_object_diffs(
            &mut warm,
            &original_journal,
        );
        assert_eq!(
            warm[&reference.object_id].object_revision,
            ObjectRevision(2)
        );
        let mut operations = vec![
            Operation::SetBoardPackageReference {
                package_id: reference.object_id,
                reference: "AMP1".into(),
            },
            Operation::SetBoardPackageValue {
                package_id: reference.object_id,
                value: "20k".into(),
            },
        ];
        operations.extend(extra);
        let mut diff = CommitDiff {
            created: vec![],
            modified: vec![],
            deleted: vec![],
        };
        for operation in &operations {
            crate::substrate::operation_application_objects::apply_operation_to_objects(
                &mut warm,
                operation,
                Some(&mut diff),
            )
            .unwrap();
        }
        diff.modified.sort();
        diff.modified.dedup();
        assert_eq!(diff.modified, [reference.object_id]);
        let mut journal = original_journal.clone();
        let mut transaction = journal.last().unwrap().clone();
        transaction.transaction_id = Uuid::new_v4();
        transaction.batch_id = Uuid::new_v4();
        transaction.operations = operations;
        transaction.inverse_operations.clear();
        transaction.diff = diff;
        journal.push(transaction);
        let mut cold = source.clone();
        crate::substrate::replay_objects::apply_promoted_journal_object_diffs(&mut cold, &journal);
        assert_eq!(
            cold[&reference.object_id].object_revision, warm[&reference.object_id].object_revision,
            "actual per-operation kernel advances must not collapse to one diff UUID"
        );
    }
}
