//! Shared-verb Part creation and exact optional links/history/no-write refusals.
use super::*;
use crate::api::native_write::{
    commit_prepared,
    genesis::{GenesisSpec, bootstrap_native_project},
    registry::find_native_write_verb,
};
use crate::substrate::{CommitSource, DesignModel, ProjectResolver};
use std::path::Path;
pub(super) fn provenance() -> WriteProvenance {
    WriteProvenance::new("test", CommitSource::Manual, "Author Part")
}
pub(super) fn prepare(
    model: &DesignModel,
    root: &Path,
    id: Uuid,
    fields: Value,
) -> Result<PreparedWrite, EngineError> {
    (find_native_write_verb("datum.library.create")
        .unwrap()
        .build)(
        &NativeWriteContext {
            model,
            project_root: root,
        },
        provenance(),
        json!({"kind":"parts","object_id":id,"pool_path":"pool","fields":fields}),
    )
}
pub(super) fn fixture() -> (std::path::PathBuf, DesignModel, Uuid, Uuid, Uuid, Uuid) {
    let root = super::super::super::test_support::temp_project_root("part_form");
    bootstrap_native_project(
        &root,
        GenesisSpec {
            project_name: "Part authoring".into(),
            existing_ids: None,
        },
    )
    .unwrap();
    let mut model = ProjectResolver::new(&root).resolve().unwrap();
    let (entity, unit, symbol, gate, package, other, footprint) = (
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
    );
    for (kind, id, object) in [
        ("units", unit, library::pool_unit_payload(unit, "Logic", "")),
        (
            "symbols",
            symbol,
            library::pool_symbol_payload(symbol, unit, "Symbol"),
        ),
        (
            "entities",
            entity,
            library::pool_entity_payload(entity, gate, unit, symbol, "Logic", "U", "", "A"),
        ),
        (
            "packages",
            package,
            library::pool_package_payload(package, "Body", json!({})),
        ),
        (
            "packages",
            other,
            library::pool_package_payload(other, "Other", json!({})),
        ),
        (
            "footprints",
            footprint,
            super::super::super::library_footprint::pool_footprint_payload(
                footprint, package, "Land",
            ),
        ),
    ] {
        let p = library::build_pool_library_write(
            &model,
            provenance(),
            Some("pool"),
            vec![PoolLibraryOperationSpec::Create {
                target: PoolLibraryObjectTarget::new("pool", kind, id),
                object,
            }],
        )
        .unwrap();
        commit_prepared(&mut model, &root, p).unwrap();
    }
    (root, model, entity, package, other, footprint)
}
pub(super) fn fields(entity: Uuid, package: Uuid, footprint: Option<Uuid>) -> Value {
    json!({"manufacturer":"Designer","mpn":"PN-1","value":"Value as entered","description":"Description","datasheet":"reference.pdf","tags":["logic"],"entity":entity,"package":package,"default_footprint":footprint})
}
#[test]
fn part_creation_exact_metadata_optional_footprint_history_and_cold() {
    let (root, mut model, entity, package, _, footprint) = fixture();
    for footprint in [None, Some(footprint)] {
        let id = Uuid::new_v4();
        let intent = fields(entity, package, footprint);
        let before = std::fs::read(crate::substrate::transaction_journal_path(&root)).unwrap();
        let p = prepare(&model, &root, id, intent.clone()).unwrap();
        assert_eq!(
            std::fs::read(crate::substrate::transaction_journal_path(&root)).unwrap(),
            before
        );
        assert!(!root.join(format!("pool/parts/{id}.json")).exists());
        let stale = p.clone();
        commit_prepared(&mut model, &root, p).unwrap();
        let path = format!("pool/parts/{id}.json");
        let raw = library::pool_library_object_payload(&model, &path, id).unwrap();
        for (key, value) in intent.as_object().unwrap() {
            assert_eq!(&raw[key], value);
        }
        assert_eq!(raw["lifecycle"], json!("Unknown"));
        assert!(raw.get("name").is_none());
        assert!(raw["default_pin_pad_map"].is_null());
        assert_eq!(raw["pad_map"], json!({}));
        assert_eq!(raw["parametric"], json!({}));
        assert_eq!(raw["behavioural_models"], json!([]));
        assert!(raw.get("approval_status").is_none());
        let source = std::fs::read(root.join(&path)).unwrap();
        let journal = std::fs::read(crate::substrate::transaction_journal_path(&root)).unwrap();
        assert!(commit_prepared(&mut model, &root, stale).is_err());
        assert_eq!(std::fs::read(root.join(&path)).unwrap(), source);
        assert_eq!(
            std::fs::read(crate::substrate::transaction_journal_path(&root)).unwrap(),
            journal
        );
        model
            .commit_journal_undo(&root, provenance().into())
            .unwrap();
        assert!(!root.join(&path).exists());
        model
            .commit_journal_redo(&root, provenance().into())
            .unwrap();
        let cold = ProjectResolver::new(&root).resolve().unwrap();
        assert_eq!(
            library::pool_library_object_payload(&cold, &path, id).unwrap(),
            raw
        );
    }
    std::fs::remove_dir_all(root).unwrap();
}
#[test]
fn part_creation_invalid_missing_kind_incompatible_and_status_refuse_without_writes() {
    let (root, model, entity, package, other, footprint) = fixture();
    let id = Uuid::new_v4();
    let valid = fields(entity, package, Some(footprint));
    let before = std::fs::read(crate::substrate::transaction_journal_path(&root)).unwrap();
    for (key, value) in [
        ("manufacturer", json!(" ")),
        ("mpn", json!("")),
        ("value", json!(9)),
        ("entity", json!(Uuid::new_v4())),
        ("entity", json!(package)),
        ("package", json!(entity)),
        ("package", json!(Uuid::new_v4())),
        ("package", json!(other)),
        ("default_footprint", json!(Uuid::new_v4())),
        ("default_footprint", json!(entity)),
        ("lifecycle", json!("Active")),
        ("approval_status", json!("approved")),
        ("default_pin_pad_map", json!(Uuid::new_v4())),
        ("pad_map", json!({})),
        ("name", json!("Unsupported")),
    ] {
        let mut bad = valid.clone();
        bad[key] = value;
        assert!(prepare(&model, &root, id, bad).is_err(), "{key}");
    }
    let mut missing = valid.clone();
    missing.as_object_mut().unwrap().remove("entity");
    assert!(prepare(&model, &root, id, missing).is_err());
    let request = LibraryDefinitionRequest {
        kind: "parts".into(),
        object_id: id,
        pool_path: "pool".into(),
        fields: valid.as_object().unwrap().clone(),
    };
    assert!(
        super::super::build_definition(
            &NativeWriteContext {
                model: &model,
                project_root: &root
            },
            provenance(),
            request,
            false
        )
        .is_err()
    );
    assert_eq!(
        std::fs::read(crate::substrate::transaction_journal_path(&root)).unwrap(),
        before
    );
    assert!(!root.join("pool/parts").exists());
    std::fs::remove_dir_all(root).unwrap();
}
#[test]
fn part_preview_reference_guards_reject_changed_canonical_before_model() {
    let (root, model, entity, package, _, footprint) = fixture();
    let p = prepare(
        &model,
        &root,
        Uuid::new_v4(),
        fields(entity, package, Some(footprint)),
    )
    .unwrap();
    let guards: std::collections::HashSet<_> = p
        .batch
        .operations
        .iter()
        .filter_map(|op| {
            if let Operation::GuardObjectRevision { object_id, .. } = op {
                Some(*object_id)
            } else {
                None
            }
        })
        .collect();
    assert_eq!(
        guards,
        std::collections::HashSet::from([entity, package, footprint])
    );
    let before = std::fs::read(crate::substrate::transaction_journal_path(&root)).unwrap();
    let mut current = model.clone();
    current.commit(p.batch.clone()).unwrap();
    for id in [entity, package, footprint] {
        let mut changed = model.clone();
        changed.objects.get_mut(&id).unwrap().object_revision.0 += 1;
        let error = changed.commit(p.batch.clone()).unwrap_err();
        assert!(error.to_string().contains("object revision mismatch"));
    }
    assert_eq!(
        std::fs::read(crate::substrate::transaction_journal_path(&root)).unwrap(),
        before
    );
    std::fs::remove_dir_all(root).unwrap();
}
