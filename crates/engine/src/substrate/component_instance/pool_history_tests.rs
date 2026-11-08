//! Canonical in-memory transaction and promoted reconstruction unit evidence.
//! This does not assert successful durable multi-Set promotion (dat-17ld).
use super::*;
use crate::api::native_write::{self as native, WriteProvenance, library};
use serde_json::json;

#[test]
fn canonical_in_memory_and_promoted_pool_multi_set_revision_payload_parity() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../.datum/scratch/codex-delivery")
        .join(format!("pool-history-unit-{}", Uuid::new_v4()));
    native::genesis::bootstrap_native_project(
        &root,
        native::genesis::GenesisSpec {
            project_name: "Pool revision unit fixture".into(),
            existing_ids: None,
        },
    )
    .unwrap();
    let mut model = super::super::super::ProjectResolver::new(&root)
        .resolve()
        .unwrap();
    let (unit, symbol) = (Uuid::new_v4(), Uuid::new_v4());
    let provenance = || {
        WriteProvenance::new(
            "history-unit-test",
            super::super::super::CommitSource::Manual,
            "Edit Symbol source",
        )
    };
    let original = library::pool_symbol_payload(symbol, unit, "Original");
    let prepared = library::build_pool_library_write(
        &model,
        provenance(),
        Some("pool"),
        vec![
            library::PoolLibraryOperationSpec::Create {
                target: library::PoolLibraryObjectTarget::new("pool", "units", unit),
                object: library::pool_unit_payload(unit, "Unit", ""),
            },
            library::PoolLibraryOperationSpec::Create {
                target: library::PoolLibraryObjectTarget::new("pool", "symbols", symbol),
                object: original.clone(),
            },
        ],
    )
    .unwrap();
    native::commit_prepared(&mut model, &root, prepared).unwrap();
    let before = model.clone();
    let revision = model.objects[&symbol].object_revision.0;
    let mut middle = original.clone();
    middle["name"] = json!("Intermediate");
    let mut final_value = middle.clone();
    final_value["name"] = json!("Final");
    let path = format!("pool/symbols/{symbol}.json");
    let batch = native::build_batch(
        &model,
        provenance(),
        vec![
            Operation::SetPoolLibraryObject {
                object_id: symbol,
                relative_path: path.clone(),
                object_kind: "symbols".into(),
                previous_object: original.clone(),
                object: middle.clone(),
            },
            Operation::BumpObjectRevision { object_id: symbol },
            Operation::SetPoolLibraryObject {
                object_id: symbol,
                relative_path: path,
                object_kind: "symbols".into(),
                previous_object: middle.clone(),
                object: final_value.clone(),
            },
        ],
    )
    .unwrap();
    model.commit(batch).unwrap();
    assert!(
        !model
            .journal
            .last()
            .unwrap()
            .operations
            .iter()
            .any(|op| matches!(op, Operation::GuardObjectRevision { .. })),
        "test the real stripped record, never manufactured guards"
    );
    let mut reconstructed = before.objects.clone();
    super::super::super::replay_objects::apply_promoted_journal_object_diffs(
        &mut reconstructed,
        &model.journal[before.journal.len()..],
    );
    assert_eq!(model.objects[&symbol].object_revision.0, revision + 3);
    assert_eq!(
        reconstructed[&symbol].object_revision,
        model.objects[&symbol].object_revision
    );
    let history = PoolHistory {
        shards: &model.source_shards,
        journal: &model.journal,
    };
    for objects in [&model.objects, &reconstructed] {
        for (r, expected) in [
            (revision, &original),
            (revision + 1, &middle),
            (revision + 2, &middle),
            (revision + 3, &final_value),
        ] {
            assert_eq!(
                history
                    .payload(&objects[&symbol], ObjectRevision(r))
                    .unwrap()
                    .as_ref(),
                Some(expected)
            );
        }
        assert!(
            history
                .payload(&objects[&symbol], ObjectRevision(revision + 4))
                .unwrap()
                .is_none()
        );
    }
    std::fs::remove_dir_all(root).unwrap();
}
