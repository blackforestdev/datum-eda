use std::collections::BTreeSet;

use super::revision_authority_i05_fixtures::rev_i05_fixture_semantics;
use super::revision_authority_i06_fixtures::rev_i06_fixture_semantics;
use super::revision_authority_i07_fixtures::rev_i07_fixture_semantics;
use super::revision_authority_i08_fixtures::rev_i08_fixture_semantics;

use super::*;
use crate::revision::{
    AUTHORITY_SCHEMA_VERSION, AuthorityEvent, AuthorityRecord, AuthorityRecordKind,
    AuthorityResolution, AuthoritySnapshot, ConfigurationItemId, ConfigurationRefId,
    EarlierControlMode, RevisionAuthorityStore, RevisionMutation, apply_revision_mutations,
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
    "project_revision_policy",
    "revision_scheme",
    "actor_identity",
    "role_assignment",
    "role_delegation",
    "project_seed_receipt",
    "waiver_departure",
    "deviation_departure",
    "legacy_revision_fact_mapping",
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
    let mut payload = serde_json::json!({
        "schema_version": AUTHORITY_SCHEMA_VERSION,
        "id": id,
        "project_id": project_id,
        "display_name": kind.wire_tag(),
        "references": []
    });
    let semantics = match kind {
        AuthorityRecordKind::RevisionReservation => serde_json::json!({
            "reservation_key": id,
            "configuration_item": family_id(AuthorityRecordKind::ConfigurationItem),
            "scheme_id": family_id(AuthorityRecordKind::RevisionScheme),
            "scheme_version": 1,
            "proposed_label": "B",
            "governing_change": family_id(AuthorityRecordKind::EngineeringChange),
            "expires_at": 100,
            "standing": "active"
        }),
        AuthorityRecordKind::EngineeringChange => serde_json::json!({
            "change_key": id,
            "events": [{"kind": "created", "sequence": 0}]
        }),
        AuthorityRecordKind::ApprovalAttestation => serde_json::json!({
            "intent": "approve",
            "disposition": "active",
            "target": {"kind": "configuration_item", "id": family_id(AuthorityRecordKind::ConfigurationItem)},
            "target_digest": format!("sha256:{}", "0".repeat(64)),
            "actor_id": family_id(AuthorityRecordKind::ActorIdentity),
            "role_assignment_id": family_id(AuthorityRecordKind::RoleAssignment),
            "policy_id": family_id(AuthorityRecordKind::ProjectRevisionPolicy),
            "policy_version": 1,
            "method_class": "synthetic-test",
            "asserted_at": 1,
            "signature": [1, 2, 3]
        }),
        AuthorityRecordKind::Effectivity => serde_json::json!({
            "expression": {
                "node": "selector",
                "value": {"family": "product_or_assembly", "values": ["all"]}
            }
        }),
        AuthorityRecordKind::ProjectRevisionPolicy => serde_json::json!({
            "policy_version": 1,
            "scheme_profile_selection": {
                "profile_name": "SequentialAlphanumericLegacy",
                "scheme_id": family_id(AuthorityRecordKind::RevisionScheme),
                "scheme_version": 1
            },
            "per_ci_namespace_rules": {
                "require_one_namespace_per_configuration_item": true,
                "allow_cross_item_token_reuse": true
            },
            "earlier_control": "no_earlier_control",
            "build_presentation": "quiet",
            "namespace_transition": "continuous",
            "approval_policy": {"requirements": [], "separation_of_duty": false},
            "effectivity_obligations": {
                "required_for_intents": [],
                "exact_population_snapshot_when_enumerable": true
            },
            "controlled_terminology": {"terms": {}},
            "required_method_classes": {"by_intent": {}}
        }),
        AuthorityRecordKind::RevisionScheme => serde_json::json!({
            "scheme_version": 1,
            "kind": "linear_alphabetic",
            "entries": [{"ordinal": 1, "revision": "A"}],
            "namespaces": {}
        }),
        AuthorityRecordKind::ActorIdentity => serde_json::json!({
            "kind": "person",
            "stable_name": "fixture actor"
        }),
        AuthorityRecordKind::RoleAssignment => serde_json::json!({
            "actor_id": family_id(AuthorityRecordKind::ActorIdentity),
            "capabilities": ["author"],
            "scope": {"scope": "project", "id": project_id},
            "effective": {"from_inclusive": 0, "until_exclusive": 10},
            "source": "fixture",
            "rationale": "closed inventory"
        }),
        AuthorityRecordKind::RoleDelegation => serde_json::json!({
            "delegator_actor_id": family_id(AuthorityRecordKind::ActorIdentity),
            "delegate_actor_id": family_id(AuthorityRecordKind::ActorIdentity),
            "capabilities": ["author"],
            "scope": {"scope": "project", "id": project_id},
            "effective": {"from_inclusive": 0, "until_exclusive": 10},
            "basis_assignment": family_id(AuthorityRecordKind::RoleAssignment),
            "rationale": "closed inventory"
        }),
        AuthorityRecordKind::ProjectSeedReceipt => serde_json::json!({
            "source_profile": "factory",
            "source_generation": "fixture",
            "source_digest": format!("sha256:{}", "1".repeat(64)),
            "copied_policy_id": family_id(AuthorityRecordKind::ProjectRevisionPolicy),
            "items": [
                {"key": "datum.revision.profile_seed", "copied_value": "SequentialAlphanumericLegacy"},
                {"key": "datum.revision.build_presentation_seed", "copied_value": "quiet"},
                {"key": "datum.revision.prototype_transition_seed", "copied_value": "continuous"}
            ]
        }),
        AuthorityRecordKind::WaiverDeparture | AuthorityRecordKind::DeviationDeparture => {
            serde_json::json!({
                "source_fact_id": id,
                "source_fact_digest": format!("sha256:{}", "2".repeat(64)),
                "governing_requirement_or_finding": "REQ-1",
                "exact_scope": {"scope": "project", "id": project_id},
                "authorizing_authority": {"kind": "configuration_item", "id": family_id(AuthorityRecordKind::ConfigurationItem)},
                "validity": {"from_inclusive": 0, "until_exclusive": 100},
                "rationale": "fixture departure",
                "disposition": "accepted"
            })
        }
        AuthorityRecordKind::LegacyRevisionFactMapping => serde_json::json!({
            "source_fact_id": id,
            "source_fact_digest": format!("sha256:{}", "3".repeat(64)),
            "source_kind": "proposal_acceptance",
            "disposition": "retained_as_legacy_evidence",
            "rationale": "not equivalent to approval"
        }),
        _ => rev_i05_fixture_semantics(kind)
            .or_else(|| rev_i06_fixture_semantics(kind))
            .or_else(|| rev_i07_fixture_semantics(kind))
            .or_else(|| rev_i08_fixture_semantics(kind))
            .unwrap_or_else(|| serde_json::json!({})),
    };
    payload
        .as_object_mut()
        .expect("payload object")
        .extend(semantics.as_object().expect("semantic object").clone());
    serde_json::from_value(serde_json::json!({
        "kind": kind.wire_tag(),
        "payload": payload
    }))
    .expect("closed family fixture decodes to its typed variant")
}

fn family_id(kind: AuthorityRecordKind) -> Uuid {
    let index = AuthorityRecordKind::ALL
        .iter()
        .position(|candidate| *candidate == kind)
        .expect("family is registered");
    Uuid::from_u128(index as u128 + 1)
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
    assert_eq!(actual.len(), EXPECTED_FAMILIES.len());
    assert_eq!(
        actual.iter().copied().collect::<BTreeSet<_>>().len(),
        EXPECTED_FAMILIES.len()
    );

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

    let new_family_digest_goldens: Vec<_> = snapshot.records[43..]
        .iter()
        .map(|record| record.canonical_digest().expect("new-family digest").0)
        .collect();
    assert_eq!(
        new_family_digest_goldens,
        [
            "sha256:dd24d7e13466e397c8a26a91357c055e724445d89548b258e00040373f58bf02",
            "sha256:40933f81d214c3b685a004fd60d8ccbe288bbf37448cff2f5db0564e2f9c5524",
            "sha256:943bab243766fd09da802c84d1d912acb900cac13b76f733945a16e9dd31c187",
            "sha256:81cc0599feae633d09deca7946feecd1db9d27a17490765f937d19db2249ce9a",
            "sha256:64fe36d8e3596a161a12039455047748b2ab48b8e1f0e6821770d87b35b56977",
            "sha256:130968995f17e100248ec0dff5f9a56587f8c39460b5d0cc506296e5ca7706c9",
            "sha256:0f7f7c1e760d96742758747cf4c00a452076c7b756b0c3a51eaeeb4bb7eec8f5",
            "sha256:245f939d4dc75f1a29c5b88ffe866862639300a7cd97acb760c34fbb22cedbf6",
            "sha256:e902fc75b2dc8fc76d3adfebb1de9372b9359a232a11b61478b5e90a36e49ed9",
        ]
    );
    let operationalized_digest_goldens = [4_usize, 6].map(|index| {
        snapshot.records[index]
            .canonical_digest()
            .expect("digest")
            .0
    });
    assert_eq!(
        operationalized_digest_goldens,
        [
            "sha256:982682b21f806c722df61d23f64621abcb3b322442ba693117824828f70c5adf",
            "sha256:0fbde3ffa71fbca25d0a6ed219b8d3ec54c153983e101beff49890079f6c9176",
        ]
    );
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
    let AuthorityRecord::ActorIdentity(actor) = record_fixture(
        AuthorityRecordKind::ActorIdentity,
        project_id,
        Uuid::new_v4(),
    ) else {
        unreachable!("fixture kind is exact")
    };
    assert!(
        apply_revision_mutations(
            &store,
            project_id,
            vec![RevisionMutation::RegisterActorIdentity(actor)],
        )
        .is_err()
    );
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
    let backup = temp_project_root("authority_all_families_backup");
    std::fs::remove_dir(&backup).expect("backup starts absent");
    export_backup(&root, project_id, &backup).expect("backup all families");
    verify_backup(&backup).expect("verify all-family backup");
    let restored = temp_project_root("authority_all_families_restore");
    restore_backup(&restored, &backup).expect("restore all families");
    assert_eq!(
        RevisionAuthorityStore::new(&restored).resolve_authority(project_id),
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
        let unmanaged_root = temp_project_root(&format!("authority_real_{name}_unmanaged"));
        let no_earlier_control_root =
            temp_project_root(&format!("authority_real_{name}_no_earlier_control"));
        copy_fixture(&manifest_dir.join(fixture), &unmanaged_root);
        copy_fixture(&manifest_dir.join(fixture), &no_earlier_control_root);
        let mut unmanaged = ProjectResolver::new(&unmanaged_root)
            .resolve()
            .expect("fixture resolve");
        let mut managed = ProjectResolver::new(&no_earlier_control_root)
            .resolve()
            .expect("paired fixture resolve");
        let setup = ordinary_rename(&unmanaged, "before-authority");
        unmanaged
            .commit_journaled(&unmanaged_root, setup.clone())
            .expect("unconfigured setup authoring");
        managed
            .commit_journaled(&no_earlier_control_root, setup)
            .expect("paired setup authoring");
        let snapshot = snapshot_with_every_family(unmanaged.project.project_id);
        let store = RevisionAuthorityStore::new(&no_earlier_control_root);
        store
            .install_authority_fixture(unmanaged.project.project_id, &snapshot)
            .expect("NoEarlierControl fixture");
        let authority_before = snapshot
            .canonical_bytes()
            .expect("authority before authoring");
        let mut unmanaged = ProjectResolver::new(&unmanaged_root)
            .resolve()
            .expect("unmanaged reopen");
        let mut managed = ProjectResolver::new(&no_earlier_control_root)
            .resolve()
            .expect("NoEarlierControl reopen");
        assert_eq!(unmanaged.model_revision, managed.model_revision);
        let ordinary = ordinary_rename(&unmanaged, "after-authority");
        unmanaged
            .commit_journaled(&unmanaged_root, ordinary.clone())
            .expect("unmanaged authoring");
        managed
            .commit_journaled(&no_earlier_control_root, ordinary)
            .expect("NoEarlierControl cannot gate authoring");
        for model in [&mut unmanaged, &mut managed] {
            for shard in &mut model.source_shards {
                shard.path = PathBuf::from(&shard.relative_path);
            }
        }
        assert_eq!(unmanaged, managed);
        assert_eq!(
            std::fs::read(transaction_journal_path(&unmanaged_root)).expect("unmanaged journal"),
            std::fs::read(transaction_journal_path(&no_earlier_control_root))
                .expect("NoEarlierControl journal")
        );
        assert!(matches!(
            RevisionAuthorityStore::new(&unmanaged_root)
                .resolve_authority(unmanaged.project.project_id),
            AuthorityResolution::Unconfigured
        ));
        assert!(matches!(
            store.resolve_authority(managed.project.project_id),
            AuthorityResolution::Resolved { snapshot: after }
                if after.canonical_bytes().expect("authority after authoring") == authority_before
        ));
    }
}

#[test]
fn ordinary_design_application_has_no_product_authority_resolver_dependency() {
    for source in [
        include_str!("../project_resolver.rs"),
        include_str!("../operation_application.rs"),
        include_str!("../operation_application_objects.rs"),
        include_str!("../operation_application_batch.rs"),
        include_str!("../operation_application_dispatch.rs"),
        include_str!("../operation_application_board_payloads.rs"),
        include_str!("../operation_application_component_instance.rs"),
        include_str!("../operation_application_object_revision.rs"),
        include_str!("../operation_application_production.rs"),
        include_str!("../operation_application_relationship.rs"),
        include_str!("../operation_application_schematic.rs"),
        include_str!("../operation_application_schematic_definition.rs"),
        include_str!("../operation_application_schematic_instance.rs"),
        include_str!("../operation_application_schematic_waiver.rs"),
        include_str!("../commit.rs"),
    ] {
        assert!(!source.contains("resolve_project_revision_policy"));
        assert!(!source.contains("evaluate_capability"));
        assert!(!source.contains("evaluate_approval_policy"));
        assert!(!source.contains("resolve_authority"));
        assert!(!source.contains("AuthorityEvent"));
        assert!(!source.contains("AuthorityRecord"));
    }
}

#[test]
fn authorized_change_required_refuses_an_unlinked_design_commit_in_rev_i04() {
    let root = temp_project_root("authorized_change_required_inert");
    let project_id = Uuid::new_v4();
    write_minimal_project(&root, project_id, Uuid::new_v4());
    let mut model = ProjectResolver::new(&root).resolve().expect("resolve");
    model
        .commit_journaled(&root, ordinary_rename(&model, "before-policy"))
        .expect("establish integrity");
    let mut snapshot = snapshot_with_every_family(project_id);
    let policy = snapshot
        .records
        .iter_mut()
        .find_map(|record| match record {
            AuthorityRecord::ProjectRevisionPolicy(body) => Some(body),
            _ => None,
        })
        .expect("policy family");
    policy.semantics.earlier_control = EarlierControlMode::AuthorizedChangeRequired;
    snapshot.events.clear();
    let mut previous = None;
    for (sequence, record) in snapshot.records.iter().enumerate() {
        let event = AuthorityEvent::structural_append(
            project_id,
            sequence as u64,
            record,
            previous.clone(),
        );
        previous = Some(event.event_digest.clone());
        snapshot.events.push(event);
    }
    let store = RevisionAuthorityStore::new(&root);
    store
        .install_authority_fixture(project_id, &snapshot)
        .expect("install representable policy");
    let authority_before = snapshot.canonical_bytes().expect("authority bytes");
    let mut reopened = ProjectResolver::new(&root).resolve().expect("Project open");
    let before_model = reopened.clone();
    let error = reopened
        .commit_journaled(&root, ordinary_rename(&reopened, "after-policy"))
        .expect_err("explicit earlier control requires an exact governing Change");
    assert!(error.to_string().contains("change_not_authorized"));
    assert_eq!(reopened, before_model);
    assert!(matches!(
        store.resolve_authority(project_id),
        AuthorityResolution::Resolved { snapshot: after }
            if after.canonical_bytes().expect("after") == authority_before
    ));
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
