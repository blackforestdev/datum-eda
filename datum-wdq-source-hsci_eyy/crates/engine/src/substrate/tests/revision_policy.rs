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
fn rev_i03_closed_inventory_is_bidirectionally_exact() {
    let new_families = &AuthorityRecordKind::ALL[43..49];
    assert_eq!(
        new_families
            .iter()
            .map(|kind| kind.wire_tag())
            .collect::<Vec<_>>(),
        [
            "project_revision_policy",
            "revision_scheme",
            "actor_identity",
            "role_assignment",
            "role_delegation",
            "project_seed_receipt",
        ]
    );
    assert_eq!(REVISION_SCHEME_KINDS.len(), 5);
    assert_eq!(ACTOR_KINDS.len(), 4);
    assert_eq!(CAPABILITIES.len(), 8);
    assert_eq!(APPROVAL_INTENTS.len(), 6);
    assert_eq!(ATTESTATION_DISPOSITIONS.len(), 3);
    assert_eq!(
        EFFECTIVITY_NODE_KINDS,
        ["selector", "all_of", "any_of", "not"]
    );
    assert_eq!(EFFECTIVITY_SELECTOR_FAMILIES.len(), 11);
    assert_eq!(REV_I03_POLICY_SECTIONS.len(), 9);
    assert_eq!(REV_I03_MUTATIONS.len(), 15);
    assert_eq!(REV_I03_QUERIES.len(), 8);
    assert_eq!(REVISION_SEED_KEYS.len(), 3);
    for inventory in [
        REV_I03_POLICY_SECTIONS,
        REV_I03_MUTATIONS,
        REV_I03_QUERIES,
        REVISION_SEED_KEYS,
    ] {
        assert_eq!(
            inventory.len(),
            inventory.iter().copied().collect::<BTreeSet<_>>().len()
        );
    }
    assert_eq!(
        RevisionMutationKind::ALL
            .iter()
            .map(|kind| kind.wire_tag())
            .collect::<Vec<_>>(),
        REV_I03_MUTATIONS
    );
    assert_eq!(
        RevisionQueryKind::ALL
            .iter()
            .map(|kind| kind.wire_tag())
            .collect::<Vec<_>>(),
        REV_I03_QUERIES
    );
}

#[test]
fn policy_absence_is_unmanaged_and_factory_profile_is_not_synthesized() {
    assert_eq!(
        resolve_project_revision_policy(None),
        ProjectRevisionPolicyResolution::Unmanaged
    );
    let (_, project_id, store) = establish_store("revision_policy_unmanaged");
    let before = std::fs::read(store.root.join("head.json")).expect("head");
    let resolution = match store.resolve_authority(project_id) {
        AuthorityResolution::Unconfigured => resolve_project_revision_policy(None),
        other => panic!("unexpected authority state: {other:?}"),
    };
    assert_eq!(resolution, ProjectRevisionPolicyResolution::Unmanaged);
    assert_eq!(
        std::fs::read(store.root.join("head.json")).expect("head after pure query"),
        before
    );
}

#[test]
fn scheme_registry_is_versioned_per_ci_and_never_allocates() {
    let project_id = Uuid::new_v4();
    let scheme_id = RevisionSchemeId(Uuid::new_v4());
    let item_a = ConfigurationItemId(Uuid::new_v4());
    let item_b = ConfigurationItemId(Uuid::new_v4());
    let mut data = scheme_data(item_a);
    data.namespaces.insert(
        item_b,
        RevisionNamespaceRule {
            prefix: "B-".to_string(),
            suffix: String::new(),
        },
    );
    let snapshot = AuthoritySnapshot {
        schema_version: AUTHORITY_SCHEMA_VERSION,
        project_id,
        records: vec![AuthorityRecord::RevisionScheme(body(
            scheme_id,
            project_id,
            Vec::new(),
            data.clone(),
        ))],
        events: Vec::new(),
        opaque_records: Vec::new(),
        opaque_events: Vec::new(),
    };
    assert!(matches!(
        resolve_revision_scheme(&snapshot, scheme_id, item_a),
        RevisionSchemeResolution::Resolved(_)
    ));
    assert!(matches!(
        resolve_revision_scheme(&snapshot, scheme_id, item_b),
        RevisionSchemeResolution::Resolved(_)
    ));
    let before = snapshot.canonical_bytes().expect("before");
    let _ = resolve_revision_scheme(&snapshot, scheme_id, item_a);
    assert_eq!(snapshot.canonical_bytes().expect("after"), before);

    for kind in REVISION_SCHEME_KINDS {
        let mut kind_data = scheme_data(item_a);
        kind_data.kind = *kind;
        if *kind == RevisionSchemeKind::Iso19650InformationContainer {
            kind_data.entries[0].suitability_or_status = Some("S0".to_string());
            kind_data.entries[1].suitability_or_status = Some("S1".to_string());
        }
        let bytes = serde_json::to_vec(&kind_data).expect("scheme kind encoding");
        assert_eq!(
            serde_json::from_slice::<RevisionSchemeData>(&bytes).expect("scheme kind round trip"),
            kind_data
        );
        assert!(validate_scheme_data(&kind_data).is_empty());
    }

    let mut exhausted = scheme_data(item_a);
    exhausted.entries.clear();
    assert_eq!(
        validate_scheme_data(&exhausted)[0].code,
        "revision_scheme_exhausted"
    );

    let mut duplicate = scheme_data(item_a);
    duplicate.entries[1].revision = duplicate.entries[0].revision.clone();
    duplicate.entries[1].ordinal = duplicate.entries[0].ordinal;
    assert!(
        validate_scheme_data(&duplicate)
            .iter()
            .any(|diagnostic| diagnostic.code == "revision_scheme_ambiguous_sequence")
    );

    let mut ineligible_status = scheme_data(item_a);
    ineligible_status.entries[0].suitability_or_status = Some("S0".to_string());
    assert!(
        validate_scheme_data(&ineligible_status)
            .iter()
            .any(|diagnostic| diagnostic.code == "revision_scheme_status_axis_ineligible")
    );

    data.entries[1].ordinal = 1;
    let invalid = AuthoritySnapshot {
        records: vec![AuthorityRecord::RevisionScheme(body(
            scheme_id,
            project_id,
            Vec::new(),
            data,
        ))],
        ..snapshot
    };
    assert!(matches!(
        resolve_revision_scheme(&invalid, scheme_id, item_a),
        RevisionSchemeResolution::Refused(AuthorityDiagnostic { code, .. })
            if code == "revision_scheme_ambiguous_sequence"
    ));
}

#[test]
fn role_capability_scope_interval_and_nonexpanding_delegation_are_enforced() {
    let project_id = Uuid::new_v4();
    let actor = ActorIdentityId(Uuid::new_v4());
    let delegate = ActorIdentityId(Uuid::new_v4());
    let assignment_id = RoleAssignmentId(Uuid::new_v4());
    let assignment = assignment_data(
        actor,
        project_id,
        BTreeSet::from([Capability::Reviewer, Capability::Verifier]),
    );
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
        rationale: "bounded delegation".to_string(),
        supersedes: None,
        revokes: None,
    };
    let snapshot = AuthoritySnapshot {
        schema_version: AUTHORITY_SCHEMA_VERSION,
        project_id,
        records: vec![
            AuthorityRecord::RoleAssignment(body(
                assignment_id,
                project_id,
                Vec::new(),
                assignment,
            )),
            AuthorityRecord::RoleDelegation(body(
                RoleDelegationId(Uuid::new_v4()),
                project_id,
                Vec::new(),
                delegation,
            )),
        ],
        events: Vec::new(),
        opaque_records: Vec::new(),
        opaque_events: Vec::new(),
    };
    assert!(snapshot.validate().is_empty());
    assert!(matches!(
        evaluate_capability(
            &snapshot,
            delegate,
            Capability::Reviewer,
            RoleScope::Project(project_id),
            50,
        ),
        CapabilityEvaluation::Authorized(_)
    ));
    assert!(matches!(
        evaluate_capability(
            &snapshot,
            delegate,
            Capability::ReleaseAuthority,
            RoleScope::Project(project_id),
            50,
        ),
        CapabilityEvaluation::Refused(AuthorityDiagnostic { code, .. })
            if code == "revision_capability_not_assigned"
    ));
    for (instant, expected) in [
        (5, "revision_role_not_yet_effective"),
        (100, "revision_role_expired"),
    ] {
        assert!(matches!(
            evaluate_capability(
                &snapshot,
                actor,
                Capability::Reviewer,
                RoleScope::Project(project_id),
                instant,
            ),
            CapabilityEvaluation::Refused(AuthorityDiagnostic { code, .. }) if code == expected
        ));
    }
    assert!(matches!(
        evaluate_capability(
            &snapshot,
            actor,
            Capability::Reviewer,
            RoleScope::Project(Uuid::new_v4()),
            50,
        ),
        CapabilityEvaluation::Refused(AuthorityDiagnostic { code, .. })
            if code == "revision_role_wrong_scope"
    ));
    let mut revoked = snapshot.clone();
    let mut revocation = assignment_data(actor, project_id, BTreeSet::new());
    revocation.revokes = Some(assignment_id);
    revoked.records.push(AuthorityRecord::RoleAssignment(body(
        RoleAssignmentId(Uuid::new_v4()),
        project_id,
        Vec::new(),
        revocation,
    )));
    assert!(matches!(
        evaluate_capability(
            &revoked,
            actor,
            Capability::Reviewer,
            RoleScope::Project(project_id),
            50,
        ),
        CapabilityEvaluation::Refused(AuthorityDiagnostic { code, .. })
            if code == "revision_role_revoked"
    ));
    let mut superseded = snapshot.clone();
    let mut replacement =
        assignment_data(actor, project_id, BTreeSet::from([Capability::Verifier]));
    replacement.supersedes = Some(assignment_id);
    superseded
        .records
        .push(AuthorityRecord::RoleAssignment(body(
            RoleAssignmentId(Uuid::new_v4()),
            project_id,
            Vec::new(),
            replacement,
        )));
    assert!(matches!(
        evaluate_capability(
            &superseded,
            actor,
            Capability::Reviewer,
            RoleScope::Project(project_id),
            50,
        ),
        CapabilityEvaluation::Refused(AuthorityDiagnostic { code, .. })
            if code == "revision_role_superseded"
    ));
    let mut expanded = snapshot.clone();
    if let AuthorityRecord::RoleDelegation(body) = &mut expanded.records[1] {
        body.semantics
            .capabilities
            .insert(Capability::ReleaseAuthority);
    }
    assert!(
        expanded
            .validate()
            .iter()
            .any(|diagnostic| { diagnostic.code == "revision_delegation_expands_authority" })
    );
}

#[test]
fn effectivity_covers_all_nodes_and_selector_families_without_universal_fallback() {
    let population = BTreeSet::from(["unit-1".to_string(), "unit-2".to_string()]);
    let mut attributes = BTreeMap::new();
    for family in EFFECTIVITY_SELECTOR_FAMILIES {
        attributes.insert(
            *family,
            BTreeMap::from([("match".to_string(), BTreeSet::from(["unit-1".to_string()]))]),
        );
    }
    let context = EffectivityResolutionContext {
        population: population.clone(),
        attributes,
        supported_families: EFFECTIVITY_SELECTOR_FAMILIES.iter().copied().collect(),
    };
    let selectors: Vec<_> = EFFECTIVITY_SELECTOR_FAMILIES
        .iter()
        .map(|family| {
            EffectivityExpression::Selector(EffectivitySelector {
                family: *family,
                values: BTreeSet::from(["match".to_string()]),
            })
        })
        .collect();
    let expression = EffectivityData {
        expression: EffectivityExpression::AnyOf(vec![
            EffectivityExpression::AllOf(selectors),
            EffectivityExpression::Not(Box::new(EffectivityExpression::Selector(
                EffectivitySelector {
                    family: EffectivitySelectorFamily::ProductOrAssembly,
                    values: BTreeSet::from(["match".to_string()]),
                },
            ))),
        ]),
        supersedes: None,
        resolved_population_snapshot: Some(population.clone()),
    };
    assert!(matches!(
        resolve_effectivity(Some(&expression), &context),
        EffectivityResolution::Resolved { members, .. } if members == population
    ));
    assert_eq!(
        resolve_effectivity(None, &context),
        EffectivityResolution::Universal
    );
    let unsupported = EffectivityResolutionContext {
        supported_families: BTreeSet::new(),
        ..context.clone()
    };
    assert!(matches!(
        resolve_effectivity(Some(&effectivity_data()), &unsupported),
        EffectivityResolution::Refused(AuthorityDiagnostic { code, .. })
            if code == "revision_effectivity_unsupported_selector"
    ));

    let ambiguous = EffectivityData {
        expression: EffectivityExpression::AllOf(Vec::new()),
        supersedes: None,
        resolved_population_snapshot: None,
    };
    assert!(matches!(
        resolve_effectivity(Some(&ambiguous), &context),
        EffectivityResolution::Refused(AuthorityDiagnostic { code, .. })
            if code == "revision_effectivity_ambiguous"
    ));
    let unknown = EffectivityData {
        expression: EffectivityExpression::Selector(EffectivitySelector {
            family: EffectivitySelectorFamily::ProductOrAssembly,
            values: BTreeSet::from(["missing".to_string()]),
        }),
        supersedes: None,
        resolved_population_snapshot: None,
    };
    assert!(matches!(
        resolve_effectivity(Some(&unknown), &context),
        EffectivityResolution::Refused(AuthorityDiagnostic { code, .. })
            if code == "revision_effectivity_unknown_selector"
    ));
    let unresolvable = EffectivityResolutionContext {
        attributes: BTreeMap::new(),
        ..context.clone()
    };
    assert!(matches!(
        resolve_effectivity(Some(&effectivity_data()), &unresolvable),
        EffectivityResolution::Refused(AuthorityDiagnostic { code, .. })
            if code == "revision_effectivity_unresolvable_selector"
    ));
}

#[test]
fn seed_consumption_is_atomic_copy_once_and_receipted() {
    let (root, project_id, store) = establish_store("revision_seed_copy_once");
    let scheme_id = RevisionSchemeId(Uuid::new_v4());
    let ci = ConfigurationItemId(Uuid::new_v4());
    apply_revision_mutations(
        &store,
        project_id,
        vec![RevisionMutation::RegisterRevisionScheme(body(
            scheme_id,
            project_id,
            Vec::new(),
            scheme_data(ci),
        ))],
    )
    .expect("register scheme");
    let frozen = FrozenRevisionSeedSnapshot {
        source_profile: "factory".to_string(),
        source_generation: "factory-v1".to_string(),
        values: BTreeMap::from([
            (
                "datum.revision.profile_seed".to_string(),
                FACTORY_PROFILE_NAME.to_string(),
            ),
            (
                "datum.revision.build_presentation_seed".to_string(),
                "quiet".to_string(),
            ),
            (
                "datum.revision.prototype_transition_seed".to_string(),
                "continuous".to_string(),
            ),
        ]),
    };
    let mutation = consume_seed_mutation(
        project_id,
        ProjectRevisionPolicyId(Uuid::new_v4()),
        ProjectSeedReceiptId(Uuid::new_v4()),
        scheme_id,
        policy_data(scheme_id, EarlierControlMode::NoEarlierControl),
        frozen.clone(),
    )
    .expect("typed seed mutation");
    apply_revision_mutations(&store, project_id, vec![mutation]).expect("consume seed");
    let snapshot = match store.resolve_authority(project_id) {
        AuthorityResolution::Resolved { snapshot } => snapshot,
        other => panic!("expected resolved authority: {other:?}"),
    };
    assert!(matches!(
        resolve_project_revision_policy(Some(&snapshot)),
        ProjectRevisionPolicyResolution::Managed { policy, .. }
            if policy.earlier_control == EarlierControlMode::NoEarlierControl
    ));
    let receipt = query_project_seed_receipt(&snapshot).expect("durable receipt");
    assert_eq!(receipt.items.len(), 3);
    let before = snapshot.canonical_bytes().expect("copied bytes");
    let mut changed = frozen;
    changed.source_generation = "factory-v2".to_string();
    changed.values.insert(
        "datum.revision.build_presentation_seed".to_string(),
        "phase_build".to_string(),
    );
    let second = consume_seed_mutation(
        project_id,
        ProjectRevisionPolicyId(Uuid::new_v4()),
        ProjectSeedReceiptId(Uuid::new_v4()),
        scheme_id,
        policy_data(scheme_id, EarlierControlMode::NoEarlierControl),
        changed,
    )
    .expect("second typed seed mutation");
    assert!(apply_revision_mutations(&store, project_id, vec![second]).is_err());
    let after = match store.resolve_authority(project_id) {
        AuthorityResolution::Resolved { snapshot } => snapshot
            .canonical_bytes()
            .expect("stable existing Project bytes"),
        other => panic!("expected resolved authority: {other:?}"),
    };
    assert_eq!(after, before);

    let backup = temp_project_root("revision_seed_backup");
    std::fs::remove_dir(&backup).expect("backup target absent");
    export_backup(&root, project_id, &backup).expect("backup");
    verify_backup(&backup).expect("verify backup");
    let restored = temp_project_root("revision_seed_restore");
    restore_backup(&restored, &backup).expect("restore");
    assert!(matches!(
        RevisionAuthorityStore::new(&restored).resolve_authority(project_id),
        AuthorityResolution::Resolved { snapshot: restored_snapshot }
            if restored_snapshot.canonical_bytes().expect("restored bytes") == before
    ));
}
