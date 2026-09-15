use super::{impact_analysis::*, *};

use std::collections::{BTreeMap, BTreeSet};

use crate::{
    revision::{
        AUTHORITY_SCHEMA_VERSION, ApprovalAttestationId, AuthorityEvent, AuthorityRecordBody,
        BuildIdentityId, ConfigurationItemId, ControlledDocumentId, DependencySnapshotId,
        EffectivityId, EmptyAuthorityPayload, EngineeringChangeId, ProjectRevisionPolicyId,
        ReleasePackageId, RevisionSchemeId,
    },
    substrate::{
        ComponentInstance, ComponentInstanceAuthority, LibraryBinding, LibraryBindingRole,
        ObjectRevision, Operation,
    },
};
use uuid::Uuid;

fn digest(byte: char) -> AlgorithmQualifiedDigest {
    AlgorithmQualifiedDigest(format!("sha256:{}", byte.to_string().repeat(64)))
}

fn working(name: &str) -> ConfigurationTarget {
    ConfigurationTarget::Working {
        model_revision: format!("model-{name}"),
        accepted_transaction_tip: format!("tip-{name}"),
    }
}

fn graph_refs() -> [AuthorityRef; 10] {
    [
        AuthorityRef::ConfigurationItem(ConfigurationItemId(Uuid::from_u128(1))),
        AuthorityRef::RevisionScheme(RevisionSchemeId(Uuid::from_u128(2))),
        AuthorityRef::EngineeringChange(EngineeringChangeId(Uuid::from_u128(3))),
        AuthorityRef::ApprovalAttestation(ApprovalAttestationId(Uuid::from_u128(4))),
        AuthorityRef::Effectivity(EffectivityId(Uuid::from_u128(5))),
        AuthorityRef::BuildIdentity(BuildIdentityId(Uuid::from_u128(6))),
        AuthorityRef::ControlledDocument(ControlledDocumentId(Uuid::from_u128(7))),
        AuthorityRef::ReleasePackage(ReleasePackageId(Uuid::from_u128(8))),
        AuthorityRef::ProjectRevisionPolicy(ProjectRevisionPolicyId(Uuid::from_u128(9))),
        AuthorityRef::DependencySnapshot(DependencySnapshotId(Uuid::from_u128(10))),
    ]
}

fn complete_graph() -> DependencySnapshotData {
    let refs = graph_refs();
    let nodes = DependencyNodeKind::ALL
        .iter()
        .enumerate()
        .rev()
        .map(|(index, kind)| DependencyNode {
            authority_ref: refs[index],
            exact_technical_revision: format!("r{index}"),
            semantic_digest: digest(char::from(b'a' + index as u8)),
            node_kind: *kind,
        })
        .collect();
    capture_dependency_snapshot(DependencySnapshotData {
        configuration: working("after"),
        nodes,
        edges: vec![
            DependencyEdge {
                edge_id: "z-zone-fill".to_string(),
                source: refs[1],
                target: refs[6],
                edge_kind: DependencyEdgeKind::Generates,
                sensitivity: vec!["geometry".to_string()],
                origin: "ZoneFill input declaration".to_string(),
                evaluator_id: "zone-fill-impact".to_string(),
                evaluator_revision: "1".to_string(),
            },
            DependencyEdge {
                edge_id: "a-library".to_string(),
                source: refs[0],
                target: refs[1],
                edge_kind: DependencyEdgeKind::UsesLibrary,
                sensitivity: vec!["geometry".to_string(), "geometry".to_string()],
                origin: "placed exact library binding".to_string(),
                evaluator_id: "library-impact".to_string(),
                evaluator_revision: "1".to_string(),
            },
            DependencyEdge {
                edge_id: "b-rule-disjoint".to_string(),
                source: refs[0],
                target: refs[3],
                edge_kind: DependencyEdgeKind::Checks,
                sensitivity: vec!["clearance".to_string()],
                origin: "check sensitivity declaration".to_string(),
                evaluator_id: "check-impact".to_string(),
                evaluator_revision: "1".to_string(),
            },
        ],
        evaluator_registry_revision: "registry-1".to_string(),
        graph_complete: true,
        unresolved_inputs: Vec::new(),
    })
}

#[test]
fn rev_i05_closed_inventories_are_bidirectionally_exact() {
    assert_eq!(
        REV_I05_RECORD_FAMILIES,
        &[
            AuthorityRecordKind::ConfigurationBaseline,
            AuthorityRecordKind::DependencySnapshot,
            AuthorityRecordKind::SemanticDelta,
            AuthorityRecordKind::ImpactEvaluation,
            AuthorityRecordKind::EvidenceInputContext,
            AuthorityRecordKind::EvidenceFreshness,
            AuthorityRecordKind::LibraryUptakeCandidate,
            AuthorityRecordKind::BaselineComparison,
            AuthorityRecordKind::RegenerationPlan,
        ]
    );
    assert_eq!(
        ImpactResult::ALL
            .iter()
            .map(|value| value.wire_tag())
            .collect::<Vec<_>>(),
        IMPACT_RESULTS
    );
    assert_eq!(
        BaselineComparisonResult::ALL
            .iter()
            .map(|value| value.wire_tag())
            .collect::<Vec<_>>(),
        BASELINE_COMPARISON_RESULTS
    );
    assert_eq!(DependencyNodeKind::ALL.len(), 10);
    assert_eq!(DependencyEdgeKind::ALL.len(), 12);
}

#[test]
fn baseline_append_preserves_historical_bytes_and_refuses_out_of_inventory_family() {
    let project_id = Uuid::from_u128(60);
    let item = ConfigurationItemId(Uuid::from_u128(61));
    let item_record = AuthorityRecord::ConfigurationItem(AuthorityRecordBody {
        schema_version: AUTHORITY_SCHEMA_VERSION,
        id: item,
        project_id,
        display_name: None,
        physical_locator: None,
        references: Vec::new(),
        semantics: EmptyAuthorityPayload::default(),
    });
    let item_event = AuthorityEvent::structural_append(project_id, 0, &item_record, None);
    let mut snapshot = AuthoritySnapshot {
        schema_version: AUTHORITY_SCHEMA_VERSION,
        project_id,
        records: vec![item_record],
        events: vec![item_event],
        opaque_records: Vec::new(),
        opaque_events: Vec::new(),
    };
    let baseline_record =
        |id: ConfigurationBaselineId, predecessor_baselines: Vec<ConfigurationBaselineId>| {
            let mut references = vec![AuthorityRef::ConfigurationItem(item)];
            references.extend(
                predecessor_baselines
                    .iter()
                    .copied()
                    .map(AuthorityRef::ConfigurationBaseline),
            );
            AuthorityRecord::ConfigurationBaseline(AuthorityRecordBody {
                schema_version: AUTHORITY_SCHEMA_VERSION,
                id,
                project_id,
                display_name: None,
                physical_locator: None,
                references,
                semantics: ConfigurationBaselineData {
                    human_number: None,
                    baseline_type: "historical-proof".to_string(),
                    scope: vec![AuthorityRef::ConfigurationItem(item)],
                    members: vec![BaselineMember {
                        stable_authority_ref: AuthorityRef::ConfigurationItem(item),
                        exact_technical_revision: "1".to_string(),
                        semantic_digest: digest('1'),
                        target_ref: None,
                        display_name: None,
                        role: "design".to_string(),
                        inclusion_reason: "immutable history proof".to_string(),
                    }],
                    source_model_revision: "model-1".to_string(),
                    accepted_transaction_tip: "tip-1".to_string(),
                    governing_changes: Vec::new(),
                    departures: Vec::new(),
                    profile_refs: Vec::new(),
                    establishment_attestations: Vec::new(),
                    established_by: AuthorityRef::ConfigurationItem(item),
                    established_at: 1,
                    predecessor_baselines,
                },
            })
        };
    let first_id = ConfigurationBaselineId(Uuid::from_u128(62));
    let first = baseline_record(first_id, Vec::new());
    append_rev_i05_records(&mut snapshot, vec![first]).expect("first baseline");
    let historical = snapshot
        .record(AuthorityRef::ConfigurationBaseline(first_id))
        .expect("historical baseline resolves")
        .canonical_digest()
        .expect("historical digest");
    let second_id = ConfigurationBaselineId(Uuid::from_u128(63));
    append_rev_i05_records(
        &mut snapshot,
        vec![baseline_record(second_id, vec![first_id])],
    )
    .expect("successor baseline");
    assert_eq!(
        snapshot
            .record(AuthorityRef::ConfigurationBaseline(first_id))
            .expect("historical baseline remains")
            .canonical_digest()
            .expect("historical digest remains"),
        historical
    );
    assert!(snapshot.validate().is_empty());

    let outside = AuthorityRecord::BuildIdentity(AuthorityRecordBody {
        schema_version: AUTHORITY_SCHEMA_VERSION,
        id: BuildIdentityId(Uuid::from_u128(64)),
        project_id,
        display_name: None,
        physical_locator: None,
        references: Vec::new(),
        semantics: EmptyAuthorityPayload::default(),
    });
    assert!(
        append_rev_i05_records(&mut snapshot, vec![outside])
            .expect_err("closed inventory refuses later family")
            .to_string()
            .contains("outside the closed REV-I05 inventory")
    );
}

#[test]
fn transitive_affected_disjoint_unaffected_and_missing_graph_unknown_are_durable() {
    let refs = graph_refs();
    let graph = complete_graph();
    assert!(validate_dependency_snapshot(&graph).is_empty());
    assert_eq!(graph.edges[0].edge_id, "a-library");
    assert_eq!(graph.edges[0].sensitivity, vec!["geometry"]);
    let delta = SemanticDeltaData {
        from_configuration: working("before"),
        to_configuration: working("after"),
        subject: refs[0],
        changed_observations: vec!["geometry".to_string()],
        administrative_observations: Vec::new(),
        evaluator_id: "design-delta".to_string(),
        evaluator_revision: "1".to_string(),
    };
    let impacts =
        evaluate_subject_impacts(&graph, std::slice::from_ref(&delta), &[refs[6], refs[3]]);
    assert_eq!(impacts[0].subject, refs[3]);
    assert_eq!(impacts[0].result, ImpactResult::Unaffected);
    assert_eq!(
        impacts[0].witness_paths,
        vec![vec!["b-rule-disjoint".to_string()]]
    );
    assert_eq!(impacts[1].subject, refs[6]);
    assert_eq!(impacts[1].result, ImpactResult::Affected);
    assert_eq!(
        impacts[1].witness_paths,
        vec![vec!["a-library".to_string(), "z-zone-fill".to_string()]]
    );

    let mut incomplete = graph;
    incomplete.graph_complete = false;
    incomplete.unresolved_inputs = vec!["missing Publish edge evaluator".to_string()];
    let unknown = evaluate_subject_impacts(&incomplete, &[delta], &[refs[6]]);
    assert_eq!(unknown[0].result, ImpactResult::ImpactUnknown);
    assert_eq!(unknown[0].reason_code, "graph_incomplete");
    assert!(unknown[0].witness_paths.is_empty());
}

#[test]
fn freshness_distinguishes_current_differing_missing_and_unsupported_inputs() {
    let refs = graph_refs();
    let recorded = vec![
        DigestQualifiedInput {
            input_ref: refs[0],
            digest: digest('a'),
        },
        DigestQualifiedInput {
            input_ref: refs[1],
            digest: digest('b'),
        },
        DigestQualifiedInput {
            input_ref: refs[2],
            digest: digest('c'),
        },
        DigestQualifiedInput {
            input_ref: refs[3],
            digest: digest('d'),
        },
    ];
    let current = BTreeMap::from([
        (refs[0], Some(digest('a'))),
        (refs[1], Some(digest('x'))),
        (refs[3], Some(digest('d'))),
    ]);
    let freshness = evaluate_evidence_freshness(
        EvidenceInputContextId(Uuid::from_u128(20)),
        working("target"),
        &recorded,
        &current,
        &BTreeSet::from([refs[3]]),
    );
    assert!(!freshness.current);
    assert_eq!(freshness.differing_inputs, vec![refs[1]]);
    assert_eq!(freshness.unresolved_inputs, vec![refs[2]]);
    assert_eq!(freshness.unsupported_inputs, vec![refs[3]]);
    assert_eq!(freshness.reasons.len(), 3);
}

#[test]
fn library_uptake_is_preview_only_until_an_exact_journal_operation_is_requested() {
    let component_id = Uuid::from_u128(30);
    let binding_id = Uuid::from_u128(31);
    let library_id = Uuid::from_u128(32);
    let component = ComponentInstance {
        id: component_id,
        object_revision: ObjectRevision(4),
        authority: ComponentInstanceAuthority::Authored,
        part_ref: None,
        library_bindings: BTreeMap::from([(
            binding_id,
            LibraryBinding {
                target_object_id: library_id,
                pinned_object_revision: ObjectRevision(7),
                pool_ref: None,
                local_override_refs: Vec::new(),
                binding_role: LibraryBindingRole::Footprint,
                provenance: None,
            },
        )]),
        placed_symbol_refs: Vec::new(),
        placed_package_refs: Vec::new(),
        placed_symbol_roles: BTreeMap::new(),
        placed_package_roles: BTreeMap::new(),
    };
    let mut candidate = LibraryUptakeCandidateData {
        component_instance_id: component_id,
        binding_id,
        pinned_library_ref: LibraryRevisionRef {
            object_id: library_id,
            object_revision: ObjectRevision(7),
        },
        proposed_library_ref: LibraryRevisionRef {
            object_id: library_id,
            object_revision: ObjectRevision(8),
        },
        pinned_source_resolves: false,
        proposed_source_resolves: true,
        semantic_delta: SemanticDeltaId(Uuid::from_u128(33)),
        predicted_impact: ImpactEvaluationId(Uuid::from_u128(34)),
        required_checks: vec!["pin-pad-map".to_string()],
        required_regeneration: vec!["zone-fill".to_string()],
        governing_change: None,
    };
    assert!(
        prepare_library_uptake_operation(&candidate, &component)
            .expect_err("missing retained pin must refuse")
            .to_string()
            .contains("cannot float to latest")
    );
    assert_eq!(
        component.library_bindings[&binding_id].pinned_object_revision,
        ObjectRevision(7)
    );
    candidate.pinned_source_resolves = true;
    let operation = prepare_library_uptake_operation(&candidate, &component)
        .expect("explicit exact uptake operation");
    let Operation::SetComponentInstance {
        previous_component_instance,
        component_instance,
        ..
    } = operation
    else {
        panic!("uptake must use the ordinary journaled component-instance mutation");
    };
    let before: ComponentInstance =
        serde_json::from_value(previous_component_instance).expect("before");
    let after: ComponentInstance = serde_json::from_value(component_instance).expect("after");
    assert_eq!(before, component);
    assert_eq!(after.object_revision, ObjectRevision(5));
    assert_eq!(
        after.library_bindings[&binding_id].pinned_object_revision,
        ObjectRevision(8)
    );
}

fn member(
    reference: AuthorityRef,
    revision: &str,
    name: &str,
    target: Option<AuthorityRef>,
) -> BaselineMember {
    BaselineMember {
        stable_authority_ref: reference,
        exact_technical_revision: revision.to_string(),
        semantic_digest: digest(revision.chars().next().unwrap_or('0')),
        target_ref: target,
        display_name: Some(name.to_string()),
        role: "controlled".to_string(),
        inclusion_reason: "comparison proof".to_string(),
    }
}

fn baseline(
    members: Vec<BaselineMember>,
    changes: Vec<EngineeringChangeId>,
) -> ConfigurationBaselineData {
    ConfigurationBaselineData {
        human_number: None,
        baseline_type: "comparison".to_string(),
        scope: Vec::new(),
        members,
        source_model_revision: "model".to_string(),
        accepted_transaction_tip: "tip".to_string(),
        governing_changes: changes,
        departures: Vec::new(),
        profile_refs: Vec::new(),
        establishment_attestations: Vec::new(),
        established_by: graph_refs()[0],
        established_at: 1,
        predecessor_baselines: Vec::new(),
    }
}

#[test]
fn baseline_comparison_emits_exact_five_results_and_refuses_unaccounted_difference() {
    let refs = graph_refs();
    let left = baseline(
        vec![
            member(refs[0], "0", "same", None),
            member(refs[1], "1", "old name", None),
            member(refs[2], "2", "retarget", Some(refs[3])),
            member(refs[4], "4", "removed", None),
        ],
        Vec::new(),
    );
    let right = baseline(
        vec![
            member(refs[0], "0", "same", None),
            member(refs[1], "1", "new name", None),
            member(refs[2], "2", "retarget", Some(refs[5])),
            member(refs[6], "6", "added", None),
        ],
        Vec::new(),
    );
    let comparison = compare_baselines(
        ConfigurationBaselineId(Uuid::from_u128(40)),
        &left,
        ConfigurationBaselineId(Uuid::from_u128(41)),
        &right,
        &BTreeMap::new(),
    );
    let actual: BTreeSet<_> = comparison
        .aligned_members
        .iter()
        .map(|entry| entry.result)
        .collect();
    assert_eq!(
        actual,
        BaselineComparisonResult::ALL.iter().copied().collect()
    );
    let renamed = comparison
        .aligned_members
        .iter()
        .find(|entry| entry.stable_authority_ref == refs[1])
        .expect("renamed stable identity remains aligned");
    assert_eq!(renamed.result, BaselineComparisonResult::Modified);
    assert!(
        renamed.technical_revision_delta.is_none(),
        "administrative rename remains Modified metadata"
    );
    assert!(require_accounted_baseline_difference(&comparison).is_err());

    let change = EngineeringChangeId(Uuid::from_u128(42));
    let accounted = compare_baselines(
        ConfigurationBaselineId(Uuid::from_u128(40)),
        &left,
        ConfigurationBaselineId(Uuid::from_u128(41)),
        &baseline(right.members, vec![change]),
        &BTreeMap::new(),
    );
    require_accounted_baseline_difference(&accounted)
        .expect("governing Change accounts for every difference");
}

#[test]
fn regeneration_plan_is_deterministic_topological_and_reuses_only_declared_current_evidence() {
    let refs = graph_refs();
    let context = EvidenceInputContextId(Uuid::from_u128(50));
    let request = |name: &str, prerequisites: &[&str]| RegenerationRequest {
        output_contract: name.to_string(),
        predecessor_evidence: None,
        reason_paths: vec![vec![format!("reason-{name}")]],
        prerequisites: prerequisites
            .iter()
            .map(|value| (*value).to_string())
            .collect(),
        producer_ref: refs[5],
        producer_revision: "1".to_string(),
        expected_input_context: context,
    };
    let plan = create_regeneration_plan(
        working("target"),
        ImpactEvaluationId(Uuid::from_u128(51)),
        vec![
            request("package", &["gerber"]),
            request("gerber", &["zone"]),
            request("zone", &[]),
        ],
        vec![refs[6], refs[6]],
    );
    assert_eq!(
        plan.steps
            .iter()
            .map(|step| step.output_contract.as_str())
            .collect::<Vec<_>>(),
        vec!["zone", "gerber", "package"]
    );
    assert_eq!(plan.unchanged_evidence_reused, vec![refs[6]]);
    assert!(plan.blockers.is_empty());
    assert!(validate_regeneration_plan(&plan).is_empty());

    let blocked = create_regeneration_plan(
        working("target"),
        ImpactEvaluationId(Uuid::from_u128(51)),
        vec![request("a", &["b"]), request("b", &["a"])],
        Vec::new(),
    );
    assert!(blocked.steps.is_empty());
    assert_eq!(blocked.blockers.len(), 2);
}
