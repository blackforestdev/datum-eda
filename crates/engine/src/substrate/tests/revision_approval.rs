use std::collections::{BTreeMap, BTreeSet};

use super::*;
use crate::revision::*;

fn body<I, P>(id: I, project_id: Uuid, semantics: P) -> AuthorityRecordBody<I, P> {
    AuthorityRecordBody {
        schema_version: AUTHORITY_SCHEMA_VERSION,
        id,
        project_id,
        display_name: None,
        physical_locator: None,
        references: Vec::new(),
        semantics,
    }
}

fn role(actor_id: ActorIdentityId, project_id: Uuid, capability: Capability) -> RoleAssignmentData {
    RoleAssignmentData {
        actor_id,
        capabilities: BTreeSet::from([capability]),
        scope: RoleScope::Project(project_id),
        effective: EffectiveInterval {
            from_inclusive: 0,
            until_exclusive: Some(100),
        },
        source: "test".to_string(),
        rationale: "approval proof".to_string(),
        supersedes: None,
        revokes: None,
    }
}

#[allow(clippy::too_many_arguments)]
fn attestation(
    id: ApprovalAttestationId,
    project_id: Uuid,
    target: AuthorityRef,
    target_digest: AlgorithmQualifiedDigest,
    actor_id: ActorIdentityId,
    role_assignment_id: RoleAssignmentId,
    policy_id: ProjectRevisionPolicyId,
    intent: ApprovalIntent,
) -> AuthorityRecordBody<ApprovalAttestationId, ApprovalAttestationData> {
    body(
        id,
        project_id,
        ApprovalAttestationData {
            intent,
            disposition: AttestationDisposition::Active,
            target,
            target_digest,
            actor_id,
            role_assignment_id,
            policy_id,
            policy_version: 1,
            method_class: "synthetic-test".to_string(),
            asserted_at: 20,
            signature: vec![0xa5],
            disposes: None,
        },
    )
}

fn approval_policy(scheme_id: RevisionSchemeId) -> ProjectRevisionPolicyData {
    ProjectRevisionPolicyData {
        policy_version: 1,
        supersedes: None,
        scheme_profile_selection: SchemeProfileSelection {
            profile_name: FACTORY_PROFILE_NAME.to_string(),
            scheme_id,
            scheme_version: 1,
        },
        per_ci_namespace_rules: PerCiNamespaceRules {
            require_one_namespace_per_configuration_item: true,
            allow_cross_item_token_reuse: true,
        },
        earlier_control: EarlierControlMode::NoEarlierControl,
        build_presentation: BuildPresentationMode::Quiet,
        namespace_transition: NamespaceTransitionMode::Continuous,
        approval_policy: ApprovalPolicy {
            requirements: vec![
                ApprovalRequirement {
                    intent: ApprovalIntent::Approve,
                    required_capability: Capability::Reviewer,
                    quorum: 1,
                    order: 1,
                    independent_from_intents: BTreeSet::new(),
                },
                ApprovalRequirement {
                    intent: ApprovalIntent::Verify,
                    required_capability: Capability::Verifier,
                    quorum: 1,
                    order: 2,
                    independent_from_intents: BTreeSet::from([ApprovalIntent::Approve]),
                },
            ],
            separation_of_duty: true,
        },
        effectivity_obligations: EffectivityObligations {
            required_for_intents: BTreeSet::new(),
            exact_population_snapshot_when_enumerable: true,
        },
        controlled_terminology: ControlledTerminology {
            terms: BTreeMap::new(),
        },
        required_method_classes: RequiredMethodClasses {
            by_intent: BTreeMap::from([
                (
                    ApprovalIntent::Approve,
                    BTreeSet::from(["synthetic-test".to_string()]),
                ),
                (
                    ApprovalIntent::Verify,
                    BTreeSet::from(["synthetic-test".to_string()]),
                ),
            ]),
        },
    }
}

struct SyntheticProvider;

impl TestSignatureProvider for SyntheticProvider {
    fn verify(&self, attestation: &ApprovalAttestationData) -> bool {
        attestation.signature == [0xa5]
    }
}

#[test]
fn approval_keeps_crypto_role_scope_quorum_order_and_separation_distinct() {
    let project_id = Uuid::new_v4();
    let policy_id = ProjectRevisionPolicyId(Uuid::new_v4());
    let policy = approval_policy(RevisionSchemeId(Uuid::new_v4()));
    let reviewer = ActorIdentityId(Uuid::new_v4());
    let verifier = ActorIdentityId(Uuid::new_v4());
    let outsider = ActorIdentityId(Uuid::new_v4());
    let reviewer_role = RoleAssignmentId(Uuid::new_v4());
    let verifier_role = RoleAssignmentId(Uuid::new_v4());
    let target = AuthorityRef::ProjectRevisionPolicy(policy_id);
    let target_digest = AlgorithmQualifiedDigest(format!("sha256:{}", "a".repeat(64)));
    let approve_id = ApprovalAttestationId(Uuid::new_v4());
    let verify_id = ApprovalAttestationId(Uuid::new_v4());
    let outsider_id = ApprovalAttestationId(Uuid::new_v4());
    let snapshot = AuthoritySnapshot {
        schema_version: AUTHORITY_SCHEMA_VERSION,
        project_id,
        records: vec![
            AuthorityRecord::ProjectRevisionPolicy(body(policy_id, project_id, policy.clone())),
            AuthorityRecord::ActorIdentity(body(
                reviewer,
                project_id,
                ActorIdentityData {
                    kind: ActorKind::Person,
                    stable_name: "reviewer".to_string(),
                },
            )),
            AuthorityRecord::ActorIdentity(body(
                verifier,
                project_id,
                ActorIdentityData {
                    kind: ActorKind::Person,
                    stable_name: "verifier".to_string(),
                },
            )),
            AuthorityRecord::ActorIdentity(body(
                outsider,
                project_id,
                ActorIdentityData {
                    kind: ActorKind::Person,
                    stable_name: "outsider".to_string(),
                },
            )),
            AuthorityRecord::RoleAssignment(body(
                reviewer_role,
                project_id,
                role(reviewer, project_id, Capability::Reviewer),
            )),
            AuthorityRecord::RoleAssignment(body(
                verifier_role,
                project_id,
                role(verifier, project_id, Capability::Verifier),
            )),
            AuthorityRecord::ApprovalAttestation(attestation(
                approve_id,
                project_id,
                target,
                target_digest.clone(),
                reviewer,
                reviewer_role,
                policy_id,
                ApprovalIntent::Approve,
            )),
            AuthorityRecord::ApprovalAttestation(attestation(
                verify_id,
                project_id,
                target,
                target_digest.clone(),
                verifier,
                verifier_role,
                policy_id,
                ApprovalIntent::Verify,
            )),
            AuthorityRecord::ApprovalAttestation(attestation(
                outsider_id,
                project_id,
                target,
                target_digest.clone(),
                outsider,
                reviewer_role,
                policy_id,
                ApprovalIntent::Approve,
            )),
        ],
        events: Vec::new(),
        opaque_records: Vec::new(),
        opaque_events: Vec::new(),
    };
    let crypto_valid = verified_attestations_for_test(&snapshot, &SyntheticProvider);
    assert!(crypto_valid.contains(&outsider_id));
    let input = ApprovalEvaluationInput {
        target,
        target_digest: target_digest.clone(),
        policy_id,
        instant: 20,
        cryptographically_valid: crypto_valid,
    };
    let mut expected = vec![approve_id, verify_id];
    expected.sort();
    assert!(matches!(
        evaluate_approval_policy(&snapshot, &policy, &input),
        ApprovalPolicyEvaluation::Satisfied { attestations }
            if attestations == expected
    ));
    let changed = ApprovalEvaluationInput {
        target_digest: AlgorithmQualifiedDigest(format!("sha256:{}", "b".repeat(64))),
        ..input
    };
    assert!(matches!(
        evaluate_approval_policy(&snapshot, &policy, &changed),
        ApprovalPolicyEvaluation::Refused(AuthorityDiagnostic { code, .. })
            if code == "revision_approval_quorum_unsatisfied"
    ));

    let valid_signature_without_authority = ApprovalEvaluationInput {
        target,
        target_digest: target_digest.clone(),
        policy_id,
        instant: 20,
        cryptographically_valid: BTreeSet::from([outsider_id]),
    };
    assert!(matches!(
        evaluate_approval_policy(&snapshot, &policy, &valid_signature_without_authority),
        ApprovalPolicyEvaluation::Refused(AuthorityDiagnostic { code, .. })
            if code == "revision_approval_quorum_unsatisfied"
    ));

    let authority_without_valid_signature = ApprovalEvaluationInput {
        target,
        target_digest,
        policy_id,
        instant: 20,
        cryptographically_valid: BTreeSet::new(),
    };
    assert!(matches!(
        evaluate_approval_policy(&snapshot, &policy, &authority_without_valid_signature),
        ApprovalPolicyEvaluation::Refused(AuthorityDiagnostic { code, .. })
            if code == "revision_approval_quorum_unsatisfied"
    ));
}

#[test]
fn attestation_dispositions_append_standing_without_body_mutation() {
    let project_id = Uuid::new_v4();
    let original_id = ApprovalAttestationId(Uuid::new_v4());
    let superseding_id = ApprovalAttestationId(Uuid::new_v4());
    let revoking_id = ApprovalAttestationId(Uuid::new_v4());
    let target = AuthorityRef::ProjectRevisionPolicy(ProjectRevisionPolicyId(Uuid::new_v4()));
    let digest = AlgorithmQualifiedDigest(format!("sha256:{}", "c".repeat(64)));
    let actor = ActorIdentityId(Uuid::new_v4());
    let role_id = RoleAssignmentId(Uuid::new_v4());
    let policy_id = ProjectRevisionPolicyId(Uuid::new_v4());
    let original = attestation(
        original_id,
        project_id,
        target,
        digest.clone(),
        actor,
        role_id,
        policy_id,
        ApprovalIntent::Approve,
    );
    let original_bytes = serde_json::to_vec(&original).expect("original bytes");
    let mut superseding = attestation(
        superseding_id,
        project_id,
        target,
        digest.clone(),
        actor,
        role_id,
        policy_id,
        ApprovalIntent::Approve,
    );
    superseding.semantics.disposition = AttestationDisposition::Superseded;
    superseding.semantics.disposes = Some(original_id);
    let mut revoking = attestation(
        revoking_id,
        project_id,
        target,
        digest,
        actor,
        role_id,
        policy_id,
        ApprovalIntent::Approve,
    );
    revoking.semantics.disposition = AttestationDisposition::Revoked;
    revoking.semantics.disposes = Some(superseding_id);
    let snapshot = AuthoritySnapshot {
        schema_version: AUTHORITY_SCHEMA_VERSION,
        project_id,
        records: vec![
            AuthorityRecord::ApprovalAttestation(original.clone()),
            AuthorityRecord::ApprovalAttestation(superseding),
            AuthorityRecord::ApprovalAttestation(revoking),
        ],
        events: Vec::new(),
        opaque_records: Vec::new(),
        opaque_events: Vec::new(),
    };
    assert_eq!(
        query_attestation_standing(&snapshot, original_id),
        AttestationStanding::Superseded { by: superseding_id }
    );
    assert_eq!(
        query_attestation_standing(&snapshot, superseding_id),
        AttestationStanding::Revoked { by: revoking_id }
    );
    assert_eq!(
        serde_json::to_vec(&original).expect("original after"),
        original_bytes
    );
}
