use std::collections::BTreeSet;

use super::*;
use crate::revision::{
    AUTHORITY_SCHEMA_VERSION, AuthorityEvent, AuthorityRecordKind, AuthorityResolution,
    AuthoritySnapshot, ConfigurationItemId, ConfigurationRefId, RevisionAuthorityStore,
    export_backup, restore_backup, verify_backup,
};

const EXPECTED_FAMILIES: &[&str] = &[
    "configuration_item",
    "configuration_ref",
    "configuration_baseline",
    "engineering_revision",
    "revision_reservation",
    "build_identity",
    "engineering_change",
    "approval_attestation",
    "effectivity",
    "release_candidate",
    "release",
    "supersession_established",
    "authorization_withdrawn",
    "obsolescence_declared",
    "controlled_document",
    "document_issue",
    "release_package",
    "transmittal",
    "standards_profile",
    "requirement_disposition",
    "audit_evaluation",
    "retention_policy",
    "legal_or_policy_hold",
    "record_disposition",
    "dependency_snapshot",
    "semantic_delta",
    "impact_evaluation",
    "evidence_input_context",
    "evidence_freshness",
    "library_uptake_candidate",
    "baseline_comparison",
    "regeneration_plan",
    "reproduction_manifest",
    "reproduction_attempt",
    "external_mapping_receipt",
    "adapter_divergence_observation",
    "release_mirror_request",
    "release_mirror_result",
    "authority_exchange_envelope",
    "authority_exchange_receipt",
    "external_change_candidate",
    "credential_or_trust_event",
    "trusted_timestamp_evidence",
];

fn snapshot_with_every_family(project_id: Uuid) -> AuthoritySnapshot {
    let records: Vec<_> = AuthorityRecordKind::ALL
        .iter()
        .enumerate()
        .map(|(index, kind)| record_fixture(*kind, project_id, Uuid::from_u128(index as u128 + 1)))
        .collect();
    let mut previous = None;
    let events = records
        .iter()
        .enumerate()
        .map(|(sequence, record)| {
            let event = AuthorityEvent::structural_append(
                project_id,
                sequence as u64,
                record,
                previous.clone(),
            );
            previous = Some(event.event_digest.clone());
            event
        })
        .collect();
    AuthoritySnapshot {
        schema_version: AUTHORITY_SCHEMA_VERSION,
        project_id,
        records,
        events,
        opaque_records: Vec::new(),
        opaque_events: Vec::new(),
    }
}

fn record_fixture(
    kind: AuthorityRecordKind,
    project_id: Uuid,
    id: Uuid,
) -> crate::revision::AuthorityRecord {
    serde_json::from_value(serde_json::json!({
        "kind": kind.wire_tag(),
        "payload": {
            "schema_version": AUTHORITY_SCHEMA_VERSION,
            "id": id,
            "project_id": project_id,
            "display_name": kind.wire_tag(),
            "references": []
        }
    }))
    .expect("closed family fixture decodes to its typed variant")
}

fn ordinary_rename(model: &DesignModel, name: &str) -> OperationBatch {
    OperationBatch {
        batch_id: Uuid::new_v4(),
        expected_model_revision: Some(model.model_revision.clone()),
        provenance: CommitProvenance {
            actor: "ordinary-author".to_string(),
            source: CommitSource::Test,
            reason: "ordinary Design authoring stays first class".to_string(),
        },
        operations: vec![Operation::SetProjectName {
            project_id: model.project.project_id,
            name: name.to_string(),
        }],
    }
}

#[test]
fn closed_family_inventory_is_exact_unique_and_canonical() {
    let actual: Vec<_> = AuthorityRecordKind::ALL
        .iter()
        .map(|kind| kind.wire_tag())
        .collect();
    assert_eq!(actual, EXPECTED_FAMILIES);
    assert_eq!(actual.len(), 43);
    assert_eq!(actual.iter().copied().collect::<BTreeSet<_>>().len(), 43);

    let project_id = Uuid::from_u128(1);
    let snapshot = snapshot_with_every_family(project_id);
    assert!(snapshot.validate().is_empty());
    let bytes = snapshot.canonical_bytes().expect("canonical snapshot");
    let decoded: AuthoritySnapshot = serde_json::from_slice(&bytes).expect("round trip");
    assert_eq!(decoded, snapshot);
    assert_eq!(snapshot.as_of(0).len(), 1);
    assert_eq!(
        snapshot.as_of((EXPECTED_FAMILIES.len() - 1) as u64).len(),
        EXPECTED_FAMILIES.len()
    );

    let mut reordered = snapshot.clone();
    reordered.records.reverse();
    assert_eq!(reordered.canonical_bytes().expect("reordered"), bytes);
}

#[test]
fn exact_typed_reference_rejects_uuid_compatible_kind_substitution() {
    let project_id = Uuid::new_v4();
    let shared = Uuid::new_v4();
    let mut snapshot = AuthoritySnapshot {
        schema_version: AUTHORITY_SCHEMA_VERSION,
        project_id,
        records: vec![record_fixture(
            AuthorityRecordKind::ConfigurationItem,
            project_id,
            shared,
        )],
        events: Vec::new(),
        opaque_records: Vec::new(),
        opaque_events: Vec::new(),
    };
    let wrong = crate::revision::AuthorityRef::ConfigurationRef(ConfigurationRefId(shared));
    let right = crate::revision::AuthorityRef::ConfigurationItem(ConfigurationItemId(shared));
    assert!(snapshot.record(right).is_some());
    assert!(snapshot.record(wrong).is_none());
    if let crate::revision::AuthorityRecord::ConfigurationItem(body) = &mut snapshot.records[0] {
        body.display_name = Some("renamed without changing identity".to_string());
        body.physical_locator = Some("moved/location".to_string());
        body.references.push(right);
    }
    assert!(snapshot.record(right).is_some());
    assert!(
        snapshot
            .validate()
            .iter()
            .any(|diagnostic| diagnostic.code == "authority_reference_cycle")
    );

    snapshot.records.push(record_fixture(
        AuthorityRecordKind::EngineeringChange,
        project_id,
        Uuid::new_v4(),
    ));
    if let crate::revision::AuthorityRecord::EngineeringChange(body) = &mut snapshot.records[1] {
        body.references.push(wrong);
    }
    assert!(
        snapshot
            .validate()
            .iter()
            .any(|diagnostic| { diagnostic.code == "authority_reference_missing_or_wrong_kind" })
    );
}

#[test]
fn absent_authority_is_explicitly_unconfigured() {
    let root = temp_project_root("authority_unconfigured");
    assert_eq!(
        RevisionAuthorityStore::new(&root).resolve_authority(Uuid::new_v4()),
        AuthorityResolution::Unconfigured
    );
}

#[test]
fn unknown_authority_schema_survives_backup_and_restore_byte_exact() {
    let root = temp_project_root("authority_unknown_schema");
    let project_id = Uuid::new_v4();
    write_minimal_project(&root, project_id, Uuid::new_v4());
    let mut model = ProjectResolver::new(&root).resolve().expect("resolve");
    model
        .commit_journaled(&root, ordinary_rename(&model, "authority-host"))
        .expect("establish integrity generation");
    let unknown = br#"{"schema_version":99,"future":"retained"}"#;
    let store = RevisionAuthorityStore::new(&root);
    store
        .install_authority_bytes_for_test(project_id, unknown)
        .expect("install unknown fixture");
    assert!(matches!(
        store.resolve_authority(project_id),
        AuthorityResolution::ReadOnlyDiagnostic { preserved_snapshot_bytes, .. }
            if preserved_snapshot_bytes == unknown
    ));

    let backup = temp_project_root("authority_unknown_backup");
    std::fs::remove_dir(&backup).expect("backup starts absent");
    export_backup(&root, project_id, &backup).expect("backup unknown bytes");
    verify_backup(&backup).expect("independent backup verification");
    let restored = temp_project_root("authority_unknown_restore");
    restore_backup(&restored, &backup).expect("restore unknown bytes");
    assert!(matches!(
        RevisionAuthorityStore::new(&restored).resolve_authority(project_id),
        AuthorityResolution::ReadOnlyDiagnostic { preserved_snapshot_bytes, .. }
            if preserved_snapshot_bytes == unknown
    ));
}

#[test]
fn authority_snapshot_is_in_generation_and_design_commit_carries_it_unchanged() {
    let root = temp_project_root("authority_persisted");
    let project_id = Uuid::new_v4();
    write_minimal_project(&root, project_id, Uuid::new_v4());
    let mut model = ProjectResolver::new(&root).resolve().expect("resolve");
    let first_batch = OperationBatch {
        batch_id: Uuid::new_v4(),
        expected_model_revision: Some(model.model_revision.clone()),
        provenance: CommitProvenance {
            actor: "authority-fixture".to_string(),
            source: CommitSource::Test,
            reason: "install inert authority fixture".to_string(),
        },
        operations: vec![Operation::SetProjectName {
            project_id,
            name: "authority-fixture".to_string(),
        }],
    };
    model
        .commit_journaled(&root, first_batch)
        .expect("first ordinary transaction");
    let snapshot = snapshot_with_every_family(project_id);
    let store = RevisionAuthorityStore::new(&root);
    store
        .install_authority_fixture(project_id, &snapshot)
        .expect("install authority fixture");

    assert_eq!(
        store.resolve_authority(project_id),
        AuthorityResolution::Resolved {
            snapshot: snapshot.clone()
        }
    );
    let before = std::fs::read(root.join(".datum/revision/v1/head.json")).expect("authority head");

    let mut authored = ProjectResolver::new(&root)
        .resolve()
        .expect("reopen unconfigured policy");
    let ordinary = ordinary_rename(&authored, "ordinary-design");
    authored
        .commit_journaled(&root, ordinary)
        .expect("ordinary authoring never gated");
    assert_eq!(
        store.resolve_authority(project_id),
        AuthorityResolution::Resolved { snapshot }
    );
    assert_ne!(
        std::fs::read(root.join(".datum/revision/v1/head.json")).expect("new head"),
        before
    );
}

#[test]
fn inert_authority_never_blocks_three_real_project_authoring_paths() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    for (name, fixture) in [
        (
            "native",
            "../test-harness/testdata/library/native_authored_baseline_v1",
        ),
        (
            "profile",
            "../test-harness/testdata/quality/route_strategy_curated_baseline_v1/profile-divergence-authored-copper",
        ),
        (
            "via",
            "../test-harness/testdata/quality/route_strategy_curated_baseline_v1/via-available",
        ),
    ] {
        let root = temp_project_root(&format!("authority_real_{name}"));
        copy_fixture(&manifest_dir.join(fixture), &root);
        let mut model = ProjectResolver::new(&root)
            .resolve()
            .expect("fixture resolve");
        model
            .commit_journaled(&root, ordinary_rename(&model, "before-authority"))
            .expect("unconfigured authoring");
        let snapshot = snapshot_with_every_family(model.project.project_id);
        let store = RevisionAuthorityStore::new(&root);
        store
            .install_authority_fixture(model.project.project_id, &snapshot)
            .expect("inert fixture");
        let mut reopened = ProjectResolver::new(&root)
            .resolve()
            .expect("fixture reopen");
        reopened
            .commit_journaled(&root, ordinary_rename(&reopened, "after-authority"))
            .expect("inert authority cannot gate authoring");
        assert_eq!(
            store.resolve_authority(model.project.project_id),
            AuthorityResolution::Resolved { snapshot }
        );
        assert_eq!(reopened.project.name, "after-authority");
    }
}

#[test]
fn ordinary_design_application_has_no_product_authority_resolver_dependency() {
    for source in [
        include_str!("../operation_application.rs"),
        include_str!("../operation_application_objects.rs"),
        include_str!("../commit.rs"),
    ] {
        assert!(!source.contains("resolve_authority"));
        assert!(!source.contains("AuthorityEvent"));
        assert!(!source.contains("AuthorityRecord"));
    }
}

fn copy_fixture(source: &Path, destination: &Path) {
    std::fs::create_dir_all(destination).expect("fixture destination");
    for entry in std::fs::read_dir(source).expect("fixture source") {
        let entry = entry.expect("fixture entry");
        let target = destination.join(entry.file_name());
        if entry.file_type().expect("fixture type").is_dir() {
            copy_fixture(&entry.path(), &target);
        } else {
            std::fs::copy(entry.path(), target).expect("fixture copy");
        }
    }
}
