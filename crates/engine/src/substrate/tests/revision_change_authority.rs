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

fn event(kind: EngineeringChangeEventKind, sequence: u64) -> EngineeringChangeEvent {
    EngineeringChangeEvent {
        kind,
        sequence,
        transaction_id: None,
        affected_item: None,
        effectivity: None,
        closure: None,
        note: None,
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

fn managed_snapshot(
    project_id: Uuid,
    earlier_control: EarlierControlMode,
) -> (
    AuthoritySnapshot,
    ConfigurationBaselineId,
    ConfigurationItemId,
    RevisionSchemeId,
) {
    let baseline_id = ConfigurationBaselineId(Uuid::new_v4());
    let item_id = ConfigurationItemId(Uuid::new_v4());
    let scheme_id = RevisionSchemeId(Uuid::new_v4());
    let policy_id = ProjectRevisionPolicyId(Uuid::new_v4());
    let records = vec![
        AuthorityRecord::ConfigurationBaseline(body(
            baseline_id,
            project_id,
            Vec::new(),
            EmptyAuthorityPayload::default(),
        )),
        AuthorityRecord::ConfigurationItem(body(
            item_id,
            project_id,
            Vec::new(),
            EmptyAuthorityPayload::default(),
        )),
        AuthorityRecord::RevisionScheme(body(
            scheme_id,
            project_id,
            Vec::new(),
            RevisionSchemeData {
                scheme_version: 1,
                kind: RevisionSchemeKind::LinearAlphabetic,
                entries: vec![RevisionSchemeEntry {
                    ordinal: 1,
                    revision: "A".to_string(),
                    suitability_or_status: None,
                }],
                namespaces: BTreeMap::new(),
                supersedes: None,
            },
        )),
        AuthorityRecord::ProjectRevisionPolicy(body(
            policy_id,
            project_id,
            vec![AuthorityRef::RevisionScheme(scheme_id)],
            policy_data(scheme_id, earlier_control),
        )),
    ];
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
    (
        AuthoritySnapshot {
            schema_version: AUTHORITY_SCHEMA_VERSION,
            project_id,
            records,
            events,
            opaque_records: Vec::new(),
            opaque_events: Vec::new(),
        },
        baseline_id,
        item_id,
        scheme_id,
    )
}

fn establish_model(name: &str) -> (PathBuf, DesignModel) {
    let root = temp_project_root(name);
    let project_id = Uuid::new_v4();
    write_minimal_project(&root, project_id, Uuid::new_v4());
    let mut model = ProjectResolver::new(&root).resolve().expect("resolve");
    let batch = rename_batch(&model, "integrity-established");
    model
        .commit_journaled(&root, batch)
        .expect("initial unmanaged commit");
    (root, model)
}

fn rename_batch(model: &DesignModel, name: &str) -> OperationBatch {
    OperationBatch {
        batch_id: Uuid::new_v4(),
        expected_model_revision: Some(model.model_revision.clone()),
        provenance: CommitProvenance {
            actor: "rev-i04-proof".to_string(),
            source: CommitSource::Test,
            reason: "REV-I04 bounded proof".to_string(),
        },
        operations: vec![Operation::SetProjectName {
            project_id: model.project.project_id,
            name: name.to_string(),
        }],
    }
}

#[test]
fn no_earlier_control_collects_identity_free_and_atomically() {
    let (root, mut model) = establish_model("rev_i04_quiet_collection");
    let (mut snapshot, baseline, item, _) = managed_snapshot(
        model.project.project_id,
        EarlierControlMode::NoEarlierControl,
    );
    let opaque = OpaqueAuthorityEnvelope {
        schema_version: 1,
        kind: "future_change_extension".to_string(),
        bytes: vec![0, 1, 2, 255],
    };
    snapshot.opaque_records.push(opaque.clone());
    let store = RevisionAuthorityStore::new(&root);
    store
        .install_authority_fixture(model.project.project_id, &snapshot)
        .expect("install policy");
    let successor = SuccessorCollectionContext {
        predecessor_baseline: baseline,
        configuration_item: item,
    };
    let first = model
        .commit_journaled_with_revision_input(
            &root,
            rename_batch(&model, "first-divergence"),
            RevisionDesignCommitInput {
                successor: Some(successor),
                ..RevisionDesignCommitInput::default()
            },
        )
        .expect("quiet first divergence");
    let first_snapshot = match store.resolve_authority(model.project.project_id) {
        AuthorityResolution::Resolved { snapshot } => snapshot,
        other => panic!("unexpected authority state: {other:?}"),
    };
    let change = query_open_successor_work(&first_snapshot, baseline, item).expect("quiet Change");
    assert_eq!(change.state, EngineeringChangeState::Draft);
    assert_eq!(
        query_change_transaction_links(&first_snapshot, change.record_id),
        [first.transaction.transaction_id]
    );
    assert!(
        first_snapshot
            .records_of_kind(AuthorityRecordKind::RevisionReservation)
            .is_empty()
    );
    for forbidden in [
        AuthorityRecordKind::EngineeringRevision,
        AuthorityRecordKind::Release,
        AuthorityRecordKind::DocumentIssue,
        AuthorityRecordKind::ApprovalAttestation,
    ] {
        assert!(first_snapshot.records_of_kind(forbidden).is_empty());
    }

    let second = model
        .commit_journaled_with_revision_input(
            &root,
            rename_batch(&model, "second-divergence"),
            RevisionDesignCommitInput {
                successor: Some(successor),
                ..RevisionDesignCommitInput::default()
            },
        )
        .expect("repeat collection");
    let second_snapshot = match store.resolve_authority(model.project.project_id) {
        AuthorityResolution::Resolved { snapshot } => snapshot,
        other => panic!("unexpected authority state: {other:?}"),
    };
    let same_change =
        query_open_successor_work(&second_snapshot, baseline, item).expect("same Change");
    assert_eq!(same_change.data.change_key, change.data.change_key);
    assert_eq!(
        query_change_transaction_links(&second_snapshot, same_change.record_id),
        [
            first.transaction.transaction_id,
            second.transaction.transaction_id
        ]
    );
    assert!(
        second_snapshot
            .records_of_kind(AuthorityRecordKind::RevisionReservation)
            .is_empty()
    );
    assert_eq!(second_snapshot.opaque_records, [opaque]);
}

#[test]
fn failed_quiet_collection_leaves_design_and_authority_unchanged() {
    let (root, mut model) = establish_model("rev_i04_quiet_fault");
    let (snapshot, baseline, item, _) = managed_snapshot(
        model.project.project_id,
        EarlierControlMode::NoEarlierControl,
    );
    let store = RevisionAuthorityStore::new(&root);
    store
        .install_authority_fixture(model.project.project_id, &snapshot)
        .expect("install policy");
    let design_before = model.clone();
    let authority_before = snapshot.canonical_bytes().expect("authority before");
    let error = model
        .commit_journaled_with_revision_input_and_fault(
            &root,
            rename_batch(&model, "interrupted-divergence"),
            RevisionDesignCommitInput {
                successor: Some(SuccessorCollectionContext {
                    predecessor_baseline: baseline,
                    configuration_item: item,
                }),
                ..RevisionDesignCommitInput::default()
            },
            IntegrityCommitFaultPoint::AuthorityStage,
        )
        .expect_err("injected interruption");
    assert!(error.to_string().contains("AuthorityStage"));
    assert_eq!(model, design_before);
    assert!(matches!(
        store.resolve_authority(model.project.project_id),
        AuthorityResolution::Resolved { snapshot: after }
            if after.canonical_bytes().expect("after") == authority_before
    ));
}

#[test]
fn authorized_change_required_accepts_only_exact_authorized_scope() {
    let (root, mut model) = establish_model("rev_i04_authorized_change");
    let project_id = model.project.project_id;
    let (mut snapshot, baseline, item, _) =
        managed_snapshot(project_id, EarlierControlMode::AuthorizedChangeRequired);
    let change_id = EngineeringChangeId(Uuid::new_v4());
    let affected = AffectedItem {
        item: AuthorityRef::ConfigurationItem(item),
        action: AffectedItemAction::Modify,
        before: None,
        proposed_after: None,
        effectivity: None,
    };
    snapshot
        .records
        .push(AuthorityRecord::EngineeringChange(body(
            change_id,
            project_id,
            vec![
                AuthorityRef::ConfigurationItem(item),
                AuthorityRef::ConfigurationBaseline(baseline),
            ],
            EngineeringChangeData {
                change_key: change_id.0,
                supersedes: None,
                predecessor_baseline: Some(baseline),
                configuration_item: Some(item),
                events: vec![
                    event(EngineeringChangeEventKind::Created, 0),
                    EngineeringChangeEvent {
                        affected_item: Some(affected),
                        ..event(EngineeringChangeEventKind::AffectedItemUpserted, 1)
                    },
                    event(EngineeringChangeEventKind::SubmittedForImpactReview, 2),
                    event(EngineeringChangeEventKind::Authorized, 3),
                ],
            },
        )));
    rebuild_structural_events(&mut snapshot);
    let store = RevisionAuthorityStore::new(&root);
    store
        .install_authority_fixture(project_id, &snapshot)
        .expect("fixture");
    let design_before = model.clone();
    let wrong = model
        .commit_journaled_with_revision_input(
            &root,
            rename_batch(&model, "wrong-scope"),
            RevisionDesignCommitInput {
                governing_change: Some(change_id),
                affected_scope: vec![AuthorityRef::ConfigurationBaseline(baseline)],
                ..RevisionDesignCommitInput::default()
            },
        )
        .expect_err("wrong exact scope must refuse");
    assert!(wrong.to_string().contains("change_not_authorized"));
    assert_eq!(model, design_before);
    let accepted = model
        .commit_journaled_with_revision_input(
            &root,
            rename_batch(&model, "authorized-scope"),
            RevisionDesignCommitInput {
                governing_change: Some(change_id),
                affected_scope: vec![AuthorityRef::ConfigurationItem(item)],
                ..RevisionDesignCommitInput::default()
            },
        )
        .expect("exact Authorized Change scope");
    let after = match store.resolve_authority(project_id) {
        AuthorityResolution::Resolved { snapshot } => snapshot,
        other => panic!("unexpected: {other:?}"),
    };
    assert!(
        query_change_transaction_links(&after, change_id)
            .contains(&accepted.transaction.transaction_id)
    );
}

#[test]
fn no_earlier_control_is_prompt_free_for_proposal_undo_and_redo_contexts() {
    let (root, mut model) = establish_model("rev_i04_context_parity");
    let project_id = model.project.project_id;
    let (snapshot, _, _, _) = managed_snapshot(project_id, EarlierControlMode::NoEarlierControl);
    let authority_bytes = snapshot.canonical_bytes().expect("authority before");
    let store = RevisionAuthorityStore::new(&root);
    store
        .install_authority_fixture(project_id, &snapshot)
        .expect("fixture");
    model
        .commit_journaled_accepted_proposal_apply(
            &root,
            rename_batch(&model, "accepted-proposal-context"),
        )
        .expect("accepted proposal is ungated");
    model
        .commit_journal_undo(
            &root,
            CommitProvenance {
                actor: "rev-i04-proof".to_string(),
                source: CommitSource::Test,
                reason: "undo context".to_string(),
            },
        )
        .expect("undo is ungated");
    model
        .commit_journal_redo(
            &root,
            CommitProvenance {
                actor: "rev-i04-proof".to_string(),
                source: CommitSource::Test,
                reason: "redo context".to_string(),
            },
        )
        .expect("redo is ungated");
    assert!(matches!(
        store.resolve_authority(project_id),
        AuthorityResolution::Resolved { snapshot: after }
            if after.canonical_bytes().expect("authority after") == authority_bytes
    ));
}

#[test]
fn reservation_and_legacy_mapping_are_explicit_append_only_actions() {
    let (root, model) = establish_model("rev_i04_reservation_legacy");
    let project_id = model.project.project_id;
    let (mut snapshot, baseline, item, scheme) =
        managed_snapshot(project_id, EarlierControlMode::NoEarlierControl);
    let opaque = OpaqueAuthorityEnvelope {
        schema_version: 1,
        kind: "future_departure_extension".to_string(),
        bytes: vec![9, 8, 7],
    };
    snapshot.opaque_events.push(opaque.clone());
    let change_id = EngineeringChangeId(Uuid::new_v4());
    snapshot
        .records
        .push(AuthorityRecord::EngineeringChange(body(
            change_id,
            project_id,
            vec![
                AuthorityRef::ConfigurationItem(item),
                AuthorityRef::ConfigurationBaseline(baseline),
            ],
            EngineeringChangeData {
                change_key: change_id.0,
                supersedes: None,
                predecessor_baseline: Some(baseline),
                configuration_item: Some(item),
                events: vec![event(EngineeringChangeEventKind::Created, 0)],
            },
        )));
    rebuild_structural_events(&mut snapshot);
    let store = RevisionAuthorityStore::new(&root);
    store
        .install_authority_fixture(project_id, &snapshot)
        .expect("fixture");
    let reservation_id = RevisionReservationId(Uuid::new_v4());
    let reservation = body(
        reservation_id,
        project_id,
        vec![
            AuthorityRef::ConfigurationItem(item),
            AuthorityRef::RevisionScheme(scheme),
            AuthorityRef::EngineeringChange(change_id),
        ],
        RevisionReservationData {
            reservation_key: reservation_id.0,
            supersedes: None,
            configuration_item: item,
            scheme_id: scheme,
            scheme_version: 1,
            proposed_label: "B".to_string(),
            governing_change: change_id,
            expires_at: 100,
            standing: RevisionReservationStanding::Active,
        },
    );
    apply_rev_i04_mutations(
        &store,
        project_id,
        vec![RevI04Mutation::ReserveRevisionLabel(reservation.clone())],
    )
    .expect("explicit reservation commits");
    let released_id = RevisionReservationId(Uuid::new_v4());
    apply_rev_i04_mutations(
        &store,
        project_id,
        vec![RevI04Mutation::ReleaseRevisionReservation(
            ReservationDispositionUpdate {
                current: reservation_id,
                next: released_id,
                standing: RevisionReservationStanding::Released,
            },
        )],
    )
    .expect("explicit release disposition");
    let released_snapshot = match store.resolve_authority(project_id) {
        AuthorityResolution::Resolved { snapshot } => snapshot,
        other => panic!("unexpected: {other:?}"),
    };
    assert_eq!(
        resolve_revision_reservation(&released_snapshot, reservation_id)
            .expect("resolved reservation")
            .data
            .standing,
        RevisionReservationStanding::Released
    );
    let replacement_id = RevisionReservationId(Uuid::new_v4());
    let replacement = body(
        replacement_id,
        project_id,
        vec![
            AuthorityRef::ConfigurationItem(item),
            AuthorityRef::EngineeringChange(change_id),
            AuthorityRef::RevisionScheme(scheme),
        ],
        RevisionReservationData {
            reservation_key: replacement_id.0,
            supersedes: None,
            configuration_item: item,
            scheme_id: scheme,
            scheme_version: 1,
            proposed_label: "B".to_string(),
            governing_change: change_id,
            expires_at: 200,
            standing: RevisionReservationStanding::Active,
        },
    );
    apply_rev_i04_mutations(
        &store,
        project_id,
        vec![RevI04Mutation::ReserveRevisionLabel(replacement.clone())],
    )
    .expect("released label can be explicitly reserved again");
    let expired_id = RevisionReservationId(Uuid::new_v4());
    apply_rev_i04_mutations(
        &store,
        project_id,
        vec![RevI04Mutation::ExpireRevisionReservation(
            ReservationDispositionUpdate {
                current: replacement_id,
                next: expired_id,
                standing: RevisionReservationStanding::Expired,
            },
        )],
    )
    .expect("explicit expiry disposition");
    let expired_snapshot = match store.resolve_authority(project_id) {
        AuthorityResolution::Resolved { snapshot } => snapshot,
        other => panic!("unexpected: {other:?}"),
    };
    assert_eq!(
        resolve_revision_reservation(&expired_snapshot, replacement_id)
            .expect("resolved replacement")
            .data
            .standing,
        RevisionReservationStanding::Expired
    );
    let active_after_expiry_id = RevisionReservationId(Uuid::new_v4());
    let mut active_after_expiry = replacement;
    active_after_expiry.id = active_after_expiry_id;
    active_after_expiry.semantics.reservation_key = active_after_expiry_id.0;
    apply_rev_i04_mutations(
        &store,
        project_id,
        vec![RevI04Mutation::ReserveRevisionLabel(
            active_after_expiry.clone(),
        )],
    )
    .expect("expired label can be explicitly reserved again");
    let before_conflict = match store.resolve_authority(project_id) {
        AuthorityResolution::Resolved { snapshot } => snapshot.canonical_bytes().expect("before"),
        other => panic!("unexpected: {other:?}"),
    };
    let mut conflicting = active_after_expiry;
    conflicting.id = RevisionReservationId(Uuid::new_v4());
    conflicting.semantics.reservation_key = conflicting.id.0;
    let error = apply_rev_i04_mutations(
        &store,
        project_id,
        vec![RevI04Mutation::ReserveRevisionLabel(conflicting)],
    )
    .expect_err("same namespace label conflicts");
    assert!(error.to_string().contains("revision_reservation_conflict"));
    assert!(matches!(
        store.resolve_authority(project_id),
        AuthorityResolution::Resolved { snapshot }
            if snapshot.canonical_bytes().expect("after") == before_conflict
    ));

    let source_fact_id = Uuid::new_v4();
    let source_bytes = br#"{"accepted":true,"kind":"check_waiver"}"#.to_vec();
    let source_digest = AlgorithmQualifiedDigest(format!("sha256:{}", "4".repeat(64)));
    let waiver_id = WaiverDepartureId(Uuid::new_v4());
    let mapping_id = LegacyRevisionFactMappingId(Uuid::new_v4());
    apply_rev_i04_mutations(
        &store,
        project_id,
        vec![
            RevI04Mutation::RecordWaiverDeparture(body(
                waiver_id,
                project_id,
                vec![AuthorityRef::ConfigurationItem(item)],
                WaiverDepartureData {
                    source_fact_id,
                    source_fact_digest: source_digest.clone(),
                    governing_requirement_or_finding: "REQ-1".to_string(),
                    exact_scope: RoleScope::Authority(AuthorityRef::ConfigurationItem(item)),
                    authorizing_authority: AuthorityRef::ConfigurationItem(item),
                    validity: EffectiveInterval {
                        from_inclusive: 0,
                        until_exclusive: Some(100),
                    },
                    effectivity: None,
                    rationale: "bounded exception".to_string(),
                    disposition: "accepted".to_string(),
                },
            )),
            RevI04Mutation::RecordLegacyRevisionFactMapping(body(
                mapping_id,
                project_id,
                vec![AuthorityRef::WaiverDeparture(waiver_id)],
                LegacyRevisionFactMappingData {
                    source_fact_id,
                    source_fact_digest: source_digest,
                    source_kind: LegacyRevisionFactKind::CheckWaiver,
                    disposition: LegacyMappingDisposition::Mapped,
                    mapped_departure: Some(AuthorityRef::WaiverDeparture(waiver_id)),
                    rationale: "complete fields assessed explicitly".to_string(),
                },
            )),
        ],
    )
    .expect("explicit mapping");
    let after = match store.resolve_authority(project_id) {
        AuthorityResolution::Resolved { snapshot } => snapshot,
        other => panic!("unexpected: {other:?}"),
    };
    assert!(query_legacy_revision_fact_mapping(&after, source_fact_id).is_some());
    assert_eq!(after.opaque_events, [opaque]);
    assert_eq!(source_bytes, br#"{"accepted":true,"kind":"check_waiver"}"#);
    assert_eq!(
        assess_legacy_revision_fact(LegacyRevisionFactKind::ProposalAcceptance, true).disposition,
        LegacyMappingDisposition::RetainedAsLegacyEvidence
    );
}

fn rebuild_structural_events(snapshot: &mut AuthoritySnapshot) {
    snapshot.events.clear();
    let mut previous = None;
    for (sequence, record) in snapshot.records.iter().enumerate() {
        let event = AuthorityEvent::structural_append(
            snapshot.project_id,
            sequence as u64,
            record,
            previous.clone(),
        );
        previous = Some(event.event_digest.clone());
        snapshot.events.push(event);
    }
}
