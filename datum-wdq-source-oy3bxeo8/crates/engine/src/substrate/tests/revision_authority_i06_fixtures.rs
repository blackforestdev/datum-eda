use uuid::Uuid;

use crate::revision::{AuthorityRecordKind, ReleaseCandidateData, candidate_evaluation_digest};

pub(super) fn rev_i06_fixture_semantics(kind: AuthorityRecordKind) -> Option<serde_json::Value> {
    let digest = |digit: char| format!("sha256:{}", digit.to_string().repeat(64));
    let authority_ref = |kind: AuthorityRecordKind| {
        serde_json::json!({
            "kind": kind.wire_tag(),
            "id": family_id(kind)
        })
    };
    Some(match kind {
        AuthorityRecordKind::EngineeringRevision => serde_json::json!({
            "configuration_item": family_id(AuthorityRecordKind::ConfigurationItem),
            "revision_namespace": "fixture-ci",
            "revision_scheme": family_id(AuthorityRecordKind::RevisionScheme),
            "scheme_version": 1,
            "revision_label": "A",
            "baseline": family_id(AuthorityRecordKind::ConfigurationBaseline),
            "governing_changes": [],
            "allocated_by_policy": family_id(AuthorityRecordKind::ProjectRevisionPolicy),
            "issued_by_release": family_id(AuthorityRecordKind::Release)
        }),
        AuthorityRecordKind::ReleaseCandidate => {
            let mut value = serde_json::json!({
                "candidate_key": Uuid::from_u128(600),
                "candidate_revision": 1,
                "proposed_baseline": baseline_semantics(),
                "proposed_revision_allocations": [{
                    "configuration_item": family_id(AuthorityRecordKind::ConfigurationItem),
                    "engineering_revision_id": family_id(AuthorityRecordKind::EngineeringRevision),
                    "revision_namespace": "fixture-ci",
                    "revision_scheme": family_id(AuthorityRecordKind::RevisionScheme),
                    "scheme_version": 1,
                    "revision_label": "A",
                    "governing_changes": [],
                    "changed": true
                }],
                "governing_changes": [],
                "departures": [],
                "effectivity": [],
                "evidence_refs": [],
                "proposed_document_issues": [],
                "proposed_packages": [],
                "approval_policy": family_id(AuthorityRecordKind::ProjectRevisionPolicy),
                "attestations": [family_id(AuthorityRecordKind::ApprovalAttestation)],
                "standards_evaluations": [],
                "readiness_findings": [],
                "last_evaluated_context": {
                    "model_revision": "fixture-model",
                    "accepted_transaction_tip": "fixture-tip",
                    "policy_digest": digest('a'),
                    "source_revisions": [{
                        "authority_ref": authority_ref(AuthorityRecordKind::ConfigurationItem),
                        "exact_technical_revision": "1",
                        "digest": digest('b')
                    }],
                    "evidence_digests": []
                },
                "evaluation_digest": digest('0')
            });
            let data: ReleaseCandidateData = serde_json::from_value(value.clone()).unwrap();
            value["evaluation_digest"] =
                serde_json::to_value(candidate_evaluation_digest(&data).unwrap()).unwrap();
            value
        }
        AuthorityRecordKind::Release => serde_json::json!({
            "released_at": 1,
            "release_authority_attestation": family_id(AuthorityRecordKind::ApprovalAttestation),
            "source_candidate": family_id(AuthorityRecordKind::ReleaseCandidate),
            "baseline": family_id(AuthorityRecordKind::ConfigurationBaseline),
            "engineering_revisions": [family_id(AuthorityRecordKind::EngineeringRevision)],
            "document_issues": [family_id(AuthorityRecordKind::DocumentIssue)],
            "release_packages": [family_id(AuthorityRecordKind::ReleasePackage)],
            "governing_changes": [], "departures": [], "effectivity": [],
            "qualifying_evidence": [], "audit_evaluations": [],
            "policy_evaluation_digest": digest('c'),
            "engine_version": "fixture-engine"
        }),
        AuthorityRecordKind::SupersessionEstablished => serde_json::json!({
            "prior_revision": family_id(AuthorityRecordKind::EngineeringRevision),
            "successor_revision": Uuid::from_u128(601),
            "authority_attestation": family_id(AuthorityRecordKind::ApprovalAttestation),
            "rationale": "fixture supersession", "effectivity": [], "established_at": 2
        }),
        AuthorityRecordKind::AuthorizationWithdrawn => serde_json::json!({
            "subject": authority_ref(AuthorityRecordKind::Release),
            "authority_attestation": family_id(AuthorityRecordKind::ApprovalAttestation),
            "rationale": "fixture withdrawal", "effectivity": [], "withdrawn_at": 3
        }),
        AuthorityRecordKind::ObsolescenceDeclared => serde_json::json!({
            "subject": authority_ref(AuthorityRecordKind::ControlledDocument),
            "authority_attestation": family_id(AuthorityRecordKind::ApprovalAttestation),
            "rationale": "fixture obsolescence", "effectivity": [], "declared_at": 4
        }),
        AuthorityRecordKind::ControlledDocument => serde_json::json!({
            "document_number": "DOC-1", "name": "Fixture drawing", "document_kind": "drawing",
            "source_ref": authority_ref(AuthorityRecordKind::ConfigurationItem),
            "revision_scheme": family_id(AuthorityRecordKind::RevisionScheme),
            "owning_configuration_item": family_id(AuthorityRecordKind::ConfigurationItem)
        }),
        AuthorityRecordKind::DocumentIssue => serde_json::json!({
            "controlled_document": family_id(AuthorityRecordKind::ControlledDocument),
            "engineering_revision": family_id(AuthorityRecordKind::EngineeringRevision),
            "baseline": family_id(AuthorityRecordKind::ConfigurationBaseline),
            "source_revisions": [{
                "authority_ref": authority_ref(AuthorityRecordKind::ConfigurationItem),
                "exact_technical_revision": "1", "digest": digest('d')
            }],
            "render_context_digest": digest('e'),
            "outputs": [{"artifact_ref": authority_ref(AuthorityRecordKind::ConfigurationItem), "byte_digest": digest('f')}],
            "approvals": [], "issued_by_release": family_id(AuthorityRecordKind::Release)
        }),
        AuthorityRecordKind::ReleasePackage => serde_json::json!({
            "name": "Fixture package", "purpose": "closed inventory",
            "release": family_id(AuthorityRecordKind::Release),
            "ordered_member_refs": [authority_ref(AuthorityRecordKind::DocumentIssue)],
            "exact_artifacts": [{"artifact_ref": authority_ref(AuthorityRecordKind::ConfigurationItem), "byte_digest": digest('1')}],
            "package_metadata_digest": digest('2'), "manifest_digest": digest('3')
        }),
        AuthorityRecordKind::Transmittal => serde_json::json!({
            "release_package": family_id(AuthorityRecordKind::ReleasePackage),
            "exact_package_digest": digest('3'), "sender": "fixture sender",
            "recipients": ["fixture recipient"],
            "delivery_policy_ref": authority_ref(AuthorityRecordKind::ProjectRevisionPolicy),
            "sent_at": 5, "delivery_evidence": [], "receipt_evidence": []
        }),
        _ => return None,
    })
}

fn baseline_semantics() -> serde_json::Value {
    serde_json::json!({
        "baseline_type": "release",
        "scope": [{"kind": "configuration_item", "id": family_id(AuthorityRecordKind::ConfigurationItem)}],
        "members": [{
            "stable_authority_ref": {"kind": "configuration_item", "id": family_id(AuthorityRecordKind::ConfigurationItem)},
            "exact_technical_revision": "1",
            "semantic_digest": format!("sha256:{}", "4".repeat(64)),
            "role": "design", "inclusion_reason": "release fixture"
        }],
        "source_model_revision": "fixture-model", "accepted_transaction_tip": "fixture-tip",
        "governing_changes": [], "departures": [], "profile_refs": [],
        "establishment_attestations": [],
        "established_by": {"kind": "release_candidate", "id": family_id(AuthorityRecordKind::ReleaseCandidate)},
        "established_at": 1, "predecessor_baselines": []
    })
}

fn family_id(kind: AuthorityRecordKind) -> Uuid {
    let index = AuthorityRecordKind::ALL
        .iter()
        .position(|candidate| *candidate == kind)
        .expect("family is registered");
    Uuid::from_u128(index as u128 + 1)
}
