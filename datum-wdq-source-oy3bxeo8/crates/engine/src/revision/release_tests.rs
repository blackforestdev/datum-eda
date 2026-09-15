use std::collections::{BTreeMap, BTreeSet};

use uuid::Uuid;

use super::*;

fn id(value: u128) -> Uuid {
    Uuid::from_u128(value)
}

fn digest(byte: char) -> AlgorithmQualifiedDigest {
    let nibble = char::from_digit(byte as u32 % 16, 16).expect("hex nibble");
    AlgorithmQualifiedDigest(format!("sha256:{}", nibble.to_string().repeat(64)))
}

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

fn empty_record(kind: AuthorityRecordKind, project_id: Uuid, value: u128) -> AuthorityRecord {
    let payload = serde_json::json!({
        "schema_version": AUTHORITY_SCHEMA_VERSION,
        "id": id(value),
        "project_id": project_id,
        "references": []
    });
    match kind {
        AuthorityRecordKind::ConfigurationItem => {
            AuthorityRecord::ConfigurationItem(serde_json::from_value(payload).unwrap())
        }
        AuthorityRecordKind::BuildIdentity => {
            AuthorityRecord::BuildIdentity(serde_json::from_value(payload).unwrap())
        }
        _ => panic!("unsupported empty fixture {kind:?}"),
    }
}

fn base_snapshot(project_id: Uuid) -> AuthoritySnapshot {
    let ci_a = ConfigurationItemId(id(10));
    let ci_b = ConfigurationItemId(id(11));
    let scheme = RevisionSchemeId(id(12));
    let policy = ProjectRevisionPolicyId(id(13));
    let source = BuildIdentityId(id(14));
    let document = ControlledDocumentId(id(16));
    let records = vec![
        empty_record(AuthorityRecordKind::ConfigurationItem, project_id, 10),
        empty_record(AuthorityRecordKind::ConfigurationItem, project_id, 11),
        AuthorityRecord::RevisionScheme(body(
            scheme,
            project_id,
            Vec::new(),
            RevisionSchemeData {
                scheme_version: 1,
                kind: RevisionSchemeKind::LinearAlphabetic,
                entries: vec![
                    RevisionSchemeEntry {
                        ordinal: 1,
                        revision: "A".into(),
                        suitability_or_status: None,
                    },
                    RevisionSchemeEntry {
                        ordinal: 2,
                        revision: "B".into(),
                        suitability_or_status: None,
                    },
                ],
                namespaces: BTreeMap::from([
                    (
                        ci_a,
                        RevisionNamespaceRule {
                            prefix: String::new(),
                            suffix: String::new(),
                        },
                    ),
                    (
                        ci_b,
                        RevisionNamespaceRule {
                            prefix: String::new(),
                            suffix: String::new(),
                        },
                    ),
                ]),
                supersedes: None,
            },
        )),
        AuthorityRecord::ProjectRevisionPolicy(body(
            policy,
            project_id,
            vec![AuthorityRef::RevisionScheme(scheme)],
            ProjectRevisionPolicyData {
                policy_version: 1,
                supersedes: None,
                scheme_profile_selection: SchemeProfileSelection {
                    profile_name: FACTORY_PROFILE_NAME.into(),
                    scheme_id: scheme,
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
            },
        )),
        empty_record(AuthorityRecordKind::BuildIdentity, project_id, 14),
        empty_record(AuthorityRecordKind::BuildIdentity, project_id, 15),
        AuthorityRecord::ControlledDocument(body(
            document,
            project_id,
            vec![
                AuthorityRef::BuildIdentity(source),
                AuthorityRef::RevisionScheme(scheme),
                AuthorityRef::ConfigurationItem(ci_a),
            ],
            ControlledDocumentData {
                document_number: "DOC-001".into(),
                name: "Assembly drawing".into(),
                document_kind: "assembly_drawing".into(),
                source_ref: AuthorityRef::BuildIdentity(source),
                revision_scheme: scheme,
                owning_configuration_item: ci_a,
                title_block_definition_ref: None,
            },
        )),
    ];
    snapshot(project_id, records)
}

fn snapshot(project_id: Uuid, records: Vec<AuthorityRecord>) -> AuthoritySnapshot {
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

fn baseline(ci_ids: &[ConfigurationItemId], actor: AuthorityRef) -> ConfigurationBaselineData {
    ConfigurationBaselineData {
        human_number: Some("BL-001".into()),
        baseline_type: "release".into(),
        scope: ci_ids
            .iter()
            .copied()
            .map(AuthorityRef::ConfigurationItem)
            .collect(),
        members: ci_ids
            .iter()
            .enumerate()
            .map(|(index, ci)| BaselineMember {
                stable_authority_ref: AuthorityRef::ConfigurationItem(*ci),
                exact_technical_revision: format!("model-{index}"),
                semantic_digest: digest(char::from(b'a' + index as u8)),
                target_ref: None,
                display_name: None,
                role: "design".into(),
                inclusion_reason: "release scope".into(),
            })
            .collect(),
        source_model_revision: "model-release".into(),
        accepted_transaction_tip: "tip-release".into(),
        governing_changes: Vec::new(),
        departures: Vec::new(),
        profile_refs: Vec::new(),
        establishment_attestations: Vec::new(),
        established_by: actor,
        established_at: 100,
        predecessor_baselines: Vec::new(),
    }
}

fn candidate_fixture(
    mut snapshot: AuthoritySnapshot,
    allocations: Vec<ProposedRevisionAllocation>,
) -> (AuthoritySnapshot, ReleaseConfigurationRequest) {
    let project_id = snapshot.project_id;
    let candidate_id = ReleaseCandidateId(id(30));
    let attestation_id = ApprovalAttestationId(id(31));
    let baseline_id = ConfigurationBaselineId(id(32));
    let release_id = ReleaseId(id(33));
    let issue_id = DocumentIssueId(id(34));
    let package_id = ReleasePackageId(id(35));
    let ci_ids = allocations
        .iter()
        .map(|item| item.configuration_item)
        .collect::<Vec<_>>();
    let new_revision = allocations
        .iter()
        .find(|item| item.changed)
        .expect("changed allocation")
        .engineering_revision_id;
    let context = CandidateEvaluationContext {
        model_revision: "model-release".into(),
        accepted_transaction_tip: "tip-release".into(),
        policy_digest: digest('p'),
        source_revisions: vec![ExactAuthorityRevision {
            authority_ref: AuthorityRef::BuildIdentity(BuildIdentityId(id(14))),
            exact_technical_revision: "source-1".into(),
            digest: digest('s'),
        }],
        evidence_digests: vec![digest('e')],
    };
    let mut candidate = ReleaseCandidateData {
        candidate_key: id(300),
        candidate_revision: 1,
        supersedes: None,
        proposed_baseline: baseline(&ci_ids, AuthorityRef::ReleaseCandidate(candidate_id)),
        proposed_revision_allocations: allocations,
        governing_changes: Vec::new(),
        departures: Vec::new(),
        effectivity: Vec::new(),
        evidence_refs: vec![AuthorityRef::BuildIdentity(BuildIdentityId(id(15)))],
        proposed_document_issues: vec![PreparedDocumentIssue {
            document_issue_id: issue_id,
            controlled_document: ControlledDocumentId(id(16)),
            engineering_revision: new_revision,
            source_revisions: context.source_revisions.clone(),
            render_context_digest: digest('r'),
            outputs: vec![ExactOutput {
                artifact_ref: AuthorityRef::BuildIdentity(BuildIdentityId(id(15))),
                byte_digest: digest('o'),
            }],
            approvals: vec![attestation_id],
        }],
        proposed_packages: vec![PreparedReleasePackage {
            release_package_id: package_id,
            package_number: Some("PKG-001".into()),
            name: "Fabrication package".into(),
            purpose: "release proof".into(),
            ordered_member_refs: vec![AuthorityRef::DocumentIssue(issue_id)],
            exact_artifacts: vec![ExactOutput {
                artifact_ref: AuthorityRef::BuildIdentity(BuildIdentityId(id(15))),
                byte_digest: digest('o'),
            }],
            package_metadata_digest: digest('m'),
        }],
        approval_policy: ProjectRevisionPolicyId(id(13)),
        attestations: vec![attestation_id],
        standards_evaluations: Vec::new(),
        readiness_findings: Vec::new(),
        last_evaluated_context: context.clone(),
        evaluation_digest: digest('0'),
    };
    candidate.evaluation_digest = candidate_evaluation_digest(&candidate).unwrap();
    let candidate_record = AuthorityRecord::ReleaseCandidate(body(
        candidate_id,
        project_id,
        vec![
            AuthorityRef::ProjectRevisionPolicy(ProjectRevisionPolicyId(id(13))),
            AuthorityRef::BuildIdentity(BuildIdentityId(id(14))),
            AuthorityRef::BuildIdentity(BuildIdentityId(id(15))),
            AuthorityRef::ControlledDocument(ControlledDocumentId(id(16))),
        ],
        candidate.clone(),
    ));
    let attestation = AuthorityRecord::ApprovalAttestation(body(
        attestation_id,
        project_id,
        vec![
            AuthorityRef::ReleaseCandidate(candidate_id),
            AuthorityRef::ProjectRevisionPolicy(ProjectRevisionPolicyId(id(13))),
        ],
        ApprovalAttestationData {
            intent: ApprovalIntent::Release,
            disposition: AttestationDisposition::Active,
            target: AuthorityRef::ReleaseCandidate(candidate_id),
            target_digest: candidate.evaluation_digest.clone(),
            actor_id: ActorIdentityId(id(90)),
            role_assignment_id: RoleAssignmentId(id(91)),
            policy_id: ProjectRevisionPolicyId(id(13)),
            policy_version: 1,
            method_class: "test-only".into(),
            asserted_at: 99,
            signature: vec![1],
            disposes: None,
        },
    ));
    transaction::append(&mut snapshot, candidate_record).unwrap();
    transaction::append(&mut snapshot, attestation).unwrap();
    assert!(snapshot.validate().is_empty(), "{:?}", snapshot.validate());
    let request = ReleaseConfigurationRequest {
        candidate_id,
        expected_candidate_revision: 1,
        expected_evaluation_digest: candidate.evaluation_digest,
        current_context: context,
        baseline_id,
        release_id,
        release_number: Some("RLS-001".into()),
        released_at: 100,
        release_authority_attestation: attestation_id,
        policy_evaluation_digest: digest('q'),
        engine_version: "test-engine".into(),
    };
    (snapshot, request)
}

fn allocation(ci: u128, revision: u128, label: &str, changed: bool) -> ProposedRevisionAllocation {
    ProposedRevisionAllocation {
        configuration_item: ConfigurationItemId(id(ci)),
        engineering_revision_id: EngineeringRevisionId(id(revision)),
        revision_namespace: format!("ci-{ci}"),
        revision_scheme: RevisionSchemeId(id(12)),
        scheme_version: 1,
        revision_label: label.into(),
        suitability_or_status: None,
        predecessor_revision: None,
        governing_changes: Vec::new(),
        changed,
        reuse_existing_revision: (!changed).then(|| EngineeringRevisionId(id(revision))),
    }
}

#[test]
fn rev_i06_closed_record_inventory_is_bidirectionally_exact() {
    assert_eq!(
        REV_I06_RECORD_FAMILIES,
        &[
            AuthorityRecordKind::EngineeringRevision,
            AuthorityRecordKind::ReleaseCandidate,
            AuthorityRecordKind::Release,
            AuthorityRecordKind::SupersessionEstablished,
            AuthorityRecordKind::AuthorizationWithdrawn,
            AuthorityRecordKind::ObsolescenceDeclared,
            AuthorityRecordKind::ControlledDocument,
            AuthorityRecordKind::DocumentIssue,
            AuthorityRecordKind::ReleasePackage,
            AuthorityRecordKind::Transmittal,
        ]
    );
    assert_eq!(ReleaseCommitFaultPoint::ALL.len(), 6);
}

#[test]
fn single_and_multi_ci_release_are_atomic_and_reuse_unchanged_revision() {
    let project_id = id(1);
    let single = candidate_fixture(
        base_snapshot(project_id),
        vec![allocation(10, 40, "A", true)],
    );
    let issued =
        prepare_release_configuration(&single.0, &single.1, None).expect("single CI release");
    assert_eq!(issued.issued_revisions, [EngineeringRevisionId(id(40))]);
    assert!(issued.reused_revisions.is_empty());

    let mut base = base_snapshot(project_id);
    transaction::append(
        &mut base,
        AuthorityRecord::EngineeringRevision(body(
            EngineeringRevisionId(id(41)),
            project_id,
            Vec::new(),
            EngineeringRevisionData {
                configuration_item: ConfigurationItemId(id(11)),
                revision_namespace: "ci-11".into(),
                revision_scheme: RevisionSchemeId(id(12)),
                scheme_version: 1,
                revision_label: "A".into(),
                suitability_or_status: None,
                baseline: ConfigurationBaselineId(id(900)),
                predecessor_revision: None,
                governing_changes: Vec::new(),
                allocated_by_policy: ProjectRevisionPolicyId(id(13)),
                issued_by_release: ReleaseId(id(901)),
            },
        )),
    )
    .unwrap();
    let multi = candidate_fixture(
        base,
        vec![
            allocation(10, 42, "B", true),
            allocation(11, 41, "A", false),
        ],
    );
    let issued = prepare_release_configuration(&multi.0, &multi.1, None).expect("multi CI release");
    assert_eq!(issued.issued_revisions, [EngineeringRevisionId(id(42))]);
    assert_eq!(issued.reused_revisions, [EngineeringRevisionId(id(41))]);
    assert_eq!(
        issued
            .snapshot
            .records_of_kind(AuthorityRecordKind::Release)
            .len(),
        1
    );
    assert_eq!(
        issued
            .snapshot
            .records_of_kind(AuthorityRecordKind::DocumentIssue)
            .len(),
        1
    );
    assert_eq!(
        issued
            .snapshot
            .records_of_kind(AuthorityRecordKind::ReleasePackage)
            .len(),
        1
    );
}

#[test]
fn every_release_commit_phase_fails_before_any_record_is_visible() {
    let fixture = candidate_fixture(base_snapshot(id(2)), vec![allocation(10, 50, "A", true)]);
    let before = fixture.0.canonical_bytes().unwrap();
    for fault in ReleaseCommitFaultPoint::ALL {
        let error = prepare_release_configuration(&fixture.0, &fixture.1, Some(*fault))
            .expect_err("fault must interrupt atomic preparation");
        assert!(
            error
                .to_string()
                .contains("injected atomic release interruption")
        );
        assert_eq!(fixture.0.canonical_bytes().unwrap(), before);
        assert!(
            fixture
                .0
                .records_of_kind(AuthorityRecordKind::Release)
                .is_empty()
        );
    }
}

#[test]
fn stale_candidate_approval_duplicate_label_and_floating_input_refuse() {
    let fixture = candidate_fixture(base_snapshot(id(3)), vec![allocation(10, 60, "A", true)]);
    let mut stale = fixture.1.clone();
    stale.current_context.model_revision = "changed".into();
    assert!(
        prepare_release_configuration(&fixture.0, &stale, None)
            .unwrap_err()
            .to_string()
            .contains("stale_release_candidate")
    );

    let mut stale_approval = fixture.0.clone();
    let AuthorityRecord::ApprovalAttestation(attestation_body) =
        stale_approval.records.last_mut().unwrap()
    else {
        unreachable!()
    };
    attestation_body.semantics.target_digest = digest('x');
    let previous = stale_approval.events[..stale_approval.events.len() - 1]
        .last()
        .map(|event| event.event_digest.clone());
    let sequence = stale_approval.events.len() as u64 - 1;
    *stale_approval.events.last_mut().unwrap() = AuthorityEvent::structural_append(
        stale_approval.project_id,
        sequence,
        stale_approval.records.last().unwrap(),
        previous,
    );
    assert!(
        prepare_release_configuration(&stale_approval, &fixture.1, None)
            .unwrap_err()
            .to_string()
            .contains("changed_approval_target")
    );

    let mut duplicate_base = base_snapshot(id(30));
    transaction::append(
        &mut duplicate_base,
        AuthorityRecord::EngineeringRevision(body(
            EngineeringRevisionId(id(62)),
            id(30),
            Vec::new(),
            EngineeringRevisionData {
                configuration_item: ConfigurationItemId(id(10)),
                revision_namespace: "ci-10".into(),
                revision_scheme: RevisionSchemeId(id(12)),
                scheme_version: 1,
                revision_label: "A".into(),
                suitability_or_status: None,
                baseline: ConfigurationBaselineId(id(902)),
                predecessor_revision: None,
                governing_changes: Vec::new(),
                allocated_by_policy: ProjectRevisionPolicyId(id(13)),
                issued_by_release: ReleaseId(id(903)),
            },
        )),
    )
    .unwrap();
    let duplicate = candidate_fixture(duplicate_base, vec![allocation(10, 61, "A", true)]);
    assert!(
        prepare_release_configuration(&duplicate.0, &duplicate.1, None)
            .unwrap_err()
            .to_string()
            .contains("duplicate_revision_label")
    );

    let mut floating = fixture.0.clone();
    let AuthorityRecord::ReleaseCandidate(body) = floating
        .records
        .iter_mut()
        .find(|record| matches!(record, AuthorityRecord::ReleaseCandidate(_)))
        .unwrap()
    else {
        unreachable!()
    };
    body.semantics.last_evaluated_context.source_revisions[0]
        .exact_technical_revision
        .clear();
    body.semantics.evaluation_digest = candidate_evaluation_digest(&body.semantics).unwrap();
    let mut floating_request = fixture.1.clone();
    floating_request.expected_evaluation_digest = body.semantics.evaluation_digest.clone();
    floating_request.current_context = body.semantics.last_evaluated_context.clone();
    let candidate_digest = body.semantics.evaluation_digest.clone();
    let AuthorityRecord::ApprovalAttestation(attestation_body) =
        floating.records.last_mut().unwrap()
    else {
        unreachable!()
    };
    attestation_body.semantics.target_digest = candidate_digest;
    let previous = floating.events[..floating.events.len() - 1]
        .last()
        .map(|event| event.event_digest.clone());
    let sequence = floating.events.len() as u64 - 1;
    *floating.events.last_mut().unwrap() = AuthorityEvent::structural_append(
        floating.project_id,
        sequence,
        floating.records.last().unwrap(),
        previous,
    );
    assert!(
        prepare_release_configuration(&floating, &floating_request, None)
            .unwrap_err()
            .to_string()
            .contains("floating_release_input")
    );
}

#[test]
fn standing_is_append_only_and_transmittal_never_remints() {
    let fixture = candidate_fixture(base_snapshot(id(4)), vec![allocation(10, 70, "A", true)]);
    let issued = prepare_release_configuration(&fixture.0, &fixture.1, None).unwrap();
    let prior_revision = issued.issued_revisions[0];
    let release_before = issued
        .snapshot
        .record(AuthorityRef::Release(fixture.1.release_id))
        .unwrap()
        .canonical_digest()
        .unwrap();
    let package_id = issued.release_packages[0];
    let package_digest = issued
        .snapshot
        .records
        .iter()
        .find_map(|record| match record {
            AuthorityRecord::ReleasePackage(body) if body.id == package_id => Some(body),
            _ => None,
        })
        .unwrap()
        .semantics
        .manifest_digest
        .clone();
    let standing = super::release_test_support::append_standing_fixtures(
        issued.snapshot.clone(),
        prior_revision,
        fixture.1.baseline_id,
        fixture.1.release_id,
    );
    assert!(standing.validate().is_empty());
    let transmission = TransmittalData {
        transmittal_number: Some("TX-001".into()),
        release_package: package_id,
        exact_package_digest: package_digest,
        sender: "Datum".into(),
        recipients: vec!["fab-a".into()],
        delivery_policy_ref: AuthorityRef::ProjectRevisionPolicy(ProjectRevisionPolicyId(id(13))),
        classification_egress_context: None,
        sent_at: 200,
        delivery_evidence: Vec::new(),
        receipt_evidence: Vec::new(),
    };
    let once = prepare_transmittal(&standing, TransmittalId(id(71)), transmission.clone()).unwrap();
    let twice = prepare_transmittal(
        &once,
        TransmittalId(id(72)),
        TransmittalData {
            sent_at: 201,
            ..transmission
        },
    )
    .unwrap();
    assert_eq!(
        twice
            .records_of_kind(AuthorityRecordKind::Transmittal)
            .len(),
        2
    );
    assert_eq!(
        twice
            .records_of_kind(AuthorityRecordKind::EngineeringRevision)
            .len(),
        2
    );
    for kind in [
        AuthorityRecordKind::SupersessionEstablished,
        AuthorityRecordKind::AuthorizationWithdrawn,
        AuthorityRecordKind::ObsolescenceDeclared,
    ] {
        assert_eq!(twice.records_of_kind(kind).len(), 1);
    }
    assert_eq!(
        twice
            .record(AuthorityRef::Release(fixture.1.release_id))
            .unwrap()
            .canonical_digest()
            .unwrap(),
        release_before
    );
}

#[test]
fn title_block_uses_engine_authority_and_bound_publish_source_refuses_delete() {
    let fixture = candidate_fixture(base_snapshot(id(5)), vec![allocation(10, 80, "A", true)]);
    let issued = prepare_release_configuration(&fixture.0, &fixture.1, None).unwrap();
    let projection =
        project_title_block_revision(&issued.snapshot, issued.document_issues[0]).unwrap();
    assert_eq!(projection.document_number, "DOC-001");
    assert_eq!(projection.configuration_item, ConfigurationItemId(id(10)));
    assert_eq!(projection.revision_label, "A");
    assert_eq!(projection.release, fixture.1.release_id);
    assert!(
        require_publish_source_unbound(
            &issued.snapshot,
            AuthorityRef::BuildIdentity(BuildIdentityId(id(14)))
        )
        .unwrap_err()
        .to_string()
        .contains("bound_publish_source")
    );
}
