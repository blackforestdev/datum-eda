use std::collections::{BTreeMap, BTreeSet};

use super::*;
use crate::revision::*;

fn body<I, P>(
    id: I,
    project_id: Uuid,
    references: Vec<AuthorityRef>,
    semantics: P,
) -> AuthorityRecordBody<I, P> {
    AuthorityRecordBody {
        schema_version: AUTHORITY_SCHEMA_VERSION,
        id,
        project_id,
        display_name: None,
        physical_locator: None,
        references,
        semantics,
    }
}

fn scheme_data(configuration_item_id: ConfigurationItemId) -> RevisionSchemeData {
    RevisionSchemeData {
        scheme_version: 1,
        kind: RevisionSchemeKind::LinearAlphabetic,
        entries: vec![
            RevisionSchemeEntry {
                ordinal: 1,
                revision: "A".to_string(),
                suitability_or_status: None,
            },
            RevisionSchemeEntry {
                ordinal: 2,
                revision: "B".to_string(),
                suitability_or_status: None,
            },
        ],
        namespaces: BTreeMap::from([(
            configuration_item_id,
            RevisionNamespaceRule {
                prefix: String::new(),
                suffix: String::new(),
            },
        )]),
        supersedes: None,
    }
}

fn policy_data(
    scheme_id: RevisionSchemeId,
    earlier_control: EarlierControlMode,
) -> ProjectRevisionPolicyData {
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
        earlier_control,
        build_presentation: BuildPresentationMode::Quiet,
        namespace_transition: NamespaceTransitionMode::Continuous,
        approval_policy: ApprovalPolicy {
            requirements: Vec::new(),
            separation_of_duty: false,
        },
        effectivity_obligations: EffectivityObligations {
            required_for_intents: BTreeSet::new(),
            exact_population_snapshot_when_enumerable: true,
        },
        controlled_terminology: ControlledTerminology {
            terms: BTreeMap::new(),
        },
        required_method_classes: RequiredMethodClasses {
            by_intent: BTreeMap::new(),
        },
    }
}

fn assignment_data(
    actor_id: ActorIdentityId,
    project_id: Uuid,
    capabilities: BTreeSet<Capability>,
) -> RoleAssignmentData {
    RoleAssignmentData {
        actor_id,
        capabilities,
        scope: RoleScope::Project(project_id),
        effective: EffectiveInterval {
            from_inclusive: 10,
            until_exclusive: Some(100),
        },
        source: "test".to_string(),
        rationale: "REV-I03 proof".to_string(),
        supersedes: None,
        revokes: None,
    }
}

fn effectivity_data() -> EffectivityData {
    EffectivityData {
        expression: EffectivityExpression::Selector(EffectivitySelector {
            family: EffectivitySelectorFamily::ProductOrAssembly,
            values: BTreeSet::from(["product-a".to_string()]),
        }),
        supersedes: None,
        resolved_population_snapshot: Some(BTreeSet::from(["unit-1".to_string()])),
    }
}

fn establish_store(name: &str) -> (std::path::PathBuf, Uuid, RevisionAuthorityStore) {
    let root = temp_project_root(name);
    let project_id = Uuid::new_v4();
    write_minimal_project(&root, project_id, Uuid::new_v4());
    let mut model = ProjectResolver::new(&root).resolve().expect("resolve");
    model
        .commit_journaled(
            &root,
            OperationBatch {
                batch_id: Uuid::new_v4(),
                expected_model_revision: Some(model.model_revision.clone()),
                provenance: CommitProvenance {
                    actor: "revision-policy-test".to_string(),
                    source: CommitSource::Test,
                    reason: "establish accepted transaction".to_string(),
                },
                operations: vec![Operation::SetProjectName {
                    project_id,
                    name: "revision-policy-host".to_string(),
                }],
            },
        )
        .expect("accepted transaction");
    let store = RevisionAuthorityStore::new(&root);
    (root, project_id, store)
}

#[test]
fn every_non_seed_mutation_uses_the_canonical_revision_transaction_path() {
    let (_, project_id, store) = establish_store("revision_policy_mutation_inventory");
    let ci = ConfigurationItemId(Uuid::new_v4());
    let scheme_1 = RevisionSchemeId(Uuid::new_v4());
    let scheme_2 = RevisionSchemeId(Uuid::new_v4());
    let mut successor_scheme = scheme_data(ci);
    successor_scheme.scheme_version = 2;
    successor_scheme.supersedes = Some(scheme_1);

    let policy_1 = ProjectRevisionPolicyId(Uuid::new_v4());
    let policy_2 = ProjectRevisionPolicyId(Uuid::new_v4());
    let first_policy = policy_data(scheme_1, EarlierControlMode::NoEarlierControl);
    let mut successor_policy = policy_data(scheme_2, EarlierControlMode::NoEarlierControl);
    successor_policy.policy_version = 2;
    successor_policy.supersedes = Some(policy_1);
    successor_policy.scheme_profile_selection.scheme_version = 2;

    let actor = ActorIdentityId(Uuid::new_v4());
    let delegate = ActorIdentityId(Uuid::new_v4());
    let assignment_id = RoleAssignmentId(Uuid::new_v4());
    let revocation_id = RoleAssignmentId(Uuid::new_v4());
    let delegation_id = RoleDelegationId(Uuid::new_v4());
    let delegation_revocation_id = RoleDelegationId(Uuid::new_v4());
    let mut role_revocation = assignment_data(actor, project_id, BTreeSet::new());
    role_revocation.revokes = Some(assignment_id);
    let delegation = RoleDelegationData {
        delegator_actor_id: actor,
        delegate_actor_id: delegate,
        capabilities: BTreeSet::from([Capability::Reviewer]),
        scope: RoleScope::Project(project_id),
        effective: EffectiveInterval {
            from_inclusive: 20,
            until_exclusive: Some(90),
        },
        basis_assignment: assignment_id,
        rationale: "bounded transaction proof".to_string(),
        supersedes: None,
        revokes: None,
    };
    let mut delegation_revocation = delegation.clone();
    delegation_revocation.capabilities.clear();
    delegation_revocation.revokes = Some(delegation_id);

    let target = AuthorityRef::ProjectRevisionPolicy(policy_2);
    let target_digest = AlgorithmQualifiedDigest(format!("sha256:{}", "d".repeat(64)));
    let attestation_id = ApprovalAttestationId(Uuid::new_v4());
    let superseding_attestation_id = ApprovalAttestationId(Uuid::new_v4());
    let revoking_attestation_id = ApprovalAttestationId(Uuid::new_v4());
    let active_attestation = ApprovalAttestationData {
        intent: ApprovalIntent::Approve,
        disposition: AttestationDisposition::Active,
        target,
        target_digest: target_digest.clone(),
        actor_id: actor,
        role_assignment_id: assignment_id,
        policy_id: policy_2,
        policy_version: 2,
        method_class: "synthetic-test".to_string(),
        asserted_at: 50,
        signature: vec![0xa5],
        disposes: None,
    };
    let mut superseding_attestation = active_attestation.clone();
    superseding_attestation.disposition = AttestationDisposition::Superseded;
    superseding_attestation.disposes = Some(attestation_id);
    let mut revoking_attestation = active_attestation.clone();
    revoking_attestation.disposition = AttestationDisposition::Revoked;
    revoking_attestation.disposes = Some(superseding_attestation_id);

    let effectivity_id = EffectivityId(Uuid::new_v4());
    let successor_effectivity_id = EffectivityId(Uuid::new_v4());
    let mut successor_effectivity = effectivity_data();
    successor_effectivity.supersedes = Some(effectivity_id);

    let mutations = vec![
        RevisionMutation::RegisterRevisionScheme(body(
            scheme_1,
            project_id,
            Vec::new(),
            scheme_data(ci),
        )),
        RevisionMutation::SupersedeRevisionScheme(body(
            scheme_2,
            project_id,
            Vec::new(),
            successor_scheme,
        )),
        RevisionMutation::AdoptProjectRevisionPolicy(body(
            policy_1,
            project_id,
            Vec::new(),
            first_policy,
        )),
        RevisionMutation::SupersedeProjectRevisionPolicy(body(
            policy_2,
            project_id,
            Vec::new(),
            successor_policy,
        )),
        RevisionMutation::RegisterActorIdentity(body(
            actor,
            project_id,
            Vec::new(),
            ActorIdentityData {
                kind: ActorKind::Person,
                stable_name: "actor".to_string(),
            },
        )),
        RevisionMutation::RegisterActorIdentity(body(
            delegate,
            project_id,
            Vec::new(),
            ActorIdentityData {
                kind: ActorKind::ControlledAutomation,
                stable_name: "delegate".to_string(),
            },
        )),
        RevisionMutation::AssignScopedRole(body(
            assignment_id,
            project_id,
            Vec::new(),
            assignment_data(actor, project_id, BTreeSet::from([Capability::Reviewer])),
        )),
        RevisionMutation::RevokeScopedRole(body(
            revocation_id,
            project_id,
            Vec::new(),
            role_revocation,
        )),
        RevisionMutation::DelegateScopedRole(body(
            delegation_id,
            project_id,
            Vec::new(),
            delegation,
        )),
        RevisionMutation::RevokeRoleDelegation(body(
            delegation_revocation_id,
            project_id,
            Vec::new(),
            delegation_revocation,
        )),
        RevisionMutation::RecordApprovalAttestation(body(
            attestation_id,
            project_id,
            Vec::new(),
            active_attestation,
        )),
        RevisionMutation::SupersedeApprovalAttestation(body(
            superseding_attestation_id,
            project_id,
            Vec::new(),
            superseding_attestation,
        )),
        RevisionMutation::RevokeApprovalAttestation(body(
            revoking_attestation_id,
            project_id,
            Vec::new(),
            revoking_attestation,
        )),
        RevisionMutation::DefineEffectivity(body(
            effectivity_id,
            project_id,
            Vec::new(),
            effectivity_data(),
        )),
        RevisionMutation::SupersedeEffectivity(body(
            successor_effectivity_id,
            project_id,
            Vec::new(),
            successor_effectivity,
        )),
    ];
    assert_eq!(
        mutations
            .iter()
            .map(RevisionMutation::wire_tag)
            .collect::<BTreeSet<_>>(),
        REV_I03_MUTATIONS[..14].iter().copied().collect()
    );
    apply_revision_mutations(&store, project_id, mutations).expect("closed mutation batch");
    assert!(matches!(
        store.resolve_authority(project_id),
        AuthorityResolution::Resolved { snapshot }
            if snapshot.records.len() == 15 && snapshot.events.len() == 15
    ));
}
