use std::collections::BTreeSet;

use crate::revision::*;

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

#[test]
fn rev_i04_closed_inventory_is_bidirectionally_exact() {
    assert_eq!(
        &AuthorityRecordKind::ALL[49..]
            .iter()
            .map(|kind| kind.wire_tag())
            .collect::<Vec<_>>(),
        &[
            "waiver_departure",
            "deviation_departure",
            "legacy_revision_fact_mapping",
        ]
    );
    assert_eq!(ENGINEERING_CHANGE_STATES.len(), 9);
    assert_eq!(ENGINEERING_CHANGE_EVENT_KINDS.len(), 14);
    assert_eq!(AFFECTED_ITEM_ACTIONS.len(), 4);
    assert_eq!(CHANGE_CLOSURE_KINDS.len(), 2);
    assert_eq!(REVISION_RESERVATION_STANDINGS.len(), 3);
    assert_eq!(LEGACY_REVISION_FACT_KINDS.len(), 4);
    assert_eq!(LEGACY_MAPPING_DISPOSITIONS.len(), 3);
    assert_eq!(REV_I04_MUTATIONS.len(), 23);
    assert_eq!(REV_I04_QUERIES.len(), 9);
    assert_eq!(REVISION_DESIGN_COMMIT_CONTEXTS.len(), 4);
    assert_eq!(REVISION_DESIGN_COMMIT_OUTCOMES.len(), 3);
    assert_eq!(RevI04RefusalCode::ALL.len(), 4);
    assert_eq!(
        RevI04MutationKind::ALL
            .iter()
            .map(|kind| kind.wire_tag())
            .collect::<Vec<_>>(),
        REV_I04_MUTATIONS
    );
    assert_eq!(
        RevI04QueryKind::ALL
            .iter()
            .map(|kind| kind.wire_tag())
            .collect::<Vec<_>>(),
        REV_I04_QUERIES
    );
    for strings in [
        REV_I04_MUTATIONS,
        REV_I04_QUERIES,
        REVISION_DESIGN_COMMIT_OUTCOMES,
        CHANGE_CLOSURE_KINDS,
    ] {
        assert_eq!(
            strings.len(),
            strings.iter().copied().collect::<BTreeSet<_>>().len()
        );
    }
}

#[test]
fn lifecycle_matrix_is_closed_and_prior_events_remain_immutable() {
    let mut allowed = BTreeSet::new();
    allowed.extend([
        (
            EngineeringChangeState::Draft,
            EngineeringChangeEventKind::SubmittedForImpactReview,
        ),
        (
            EngineeringChangeState::ImpactReview,
            EngineeringChangeEventKind::Authorized,
        ),
        (
            EngineeringChangeState::Authorized,
            EngineeringChangeEventKind::ImplementationBegan,
        ),
        (
            EngineeringChangeState::Implementing,
            EngineeringChangeEventKind::ImplementationVerificationRecorded,
        ),
        (
            EngineeringChangeState::Verification,
            EngineeringChangeEventKind::Closed,
        ),
    ]);
    for state in [
        EngineeringChangeState::Draft,
        EngineeringChangeState::ImpactReview,
        EngineeringChangeState::Authorized,
        EngineeringChangeState::Implementing,
        EngineeringChangeState::Verification,
    ] {
        for side_exit in [
            EngineeringChangeEventKind::Rejected,
            EngineeringChangeEventKind::Deferred,
            EngineeringChangeEventKind::Cancelled,
        ] {
            allowed.insert((state, side_exit));
        }
    }
    for state in [
        EngineeringChangeState::ImpactReview,
        EngineeringChangeState::Authorized,
        EngineeringChangeState::Implementing,
        EngineeringChangeState::Verification,
    ] {
        allowed.insert((state, EngineeringChangeEventKind::ReworkRequested));
    }
    for state in ENGINEERING_CHANGE_STATES {
        for kind in ENGINEERING_CHANGE_EVENT_KINDS {
            assert_eq!(
                evaluate_change_transition(*state, *kind),
                allowed.contains(&(*state, *kind)),
                "unexpected transition {state:?} + {kind:?}"
            );
        }
    }
    let events = vec![
        event(EngineeringChangeEventKind::Created, 0),
        event(EngineeringChangeEventKind::SubmittedForImpactReview, 1),
        event(EngineeringChangeEventKind::Authorized, 2),
        event(EngineeringChangeEventKind::ImplementationBegan, 3),
        event(
            EngineeringChangeEventKind::ImplementationVerificationRecorded,
            4,
        ),
        EngineeringChangeEvent {
            closure: Some(ChangeClosureKind::NoReleaseDisposition(
                "prototype ended without release".to_string(),
            )),
            ..event(EngineeringChangeEventKind::Closed, 5)
        },
    ];
    let prefix = serde_json::to_vec(&events[..5]).expect("prior event bytes");
    assert_eq!(
        engineering_change_state(&events),
        Some(EngineeringChangeState::Closed)
    );
    assert_eq!(
        serde_json::to_vec(&events[..5]).expect("prior events"),
        prefix
    );
}

#[test]
fn sole_revision_gate_is_the_journaled_commit_coordinator() {
    let coordinator = include_str!("../commit.rs");
    assert_eq!(
        coordinator
            .matches("prepare_revision_design_commit(")
            .count(),
        1
    );
    for source in [
        include_str!("../operation_application.rs"),
        include_str!("../operation_application_batch.rs"),
        include_str!("../operation_application_dispatch.rs"),
        include_str!("../project_resolver.rs"),
    ] {
        assert!(!source.contains("prepare_revision_design_commit"));
        assert!(!source.contains("evaluate_design_mutation_authority"));
        assert!(!source.contains("change_not_authorized"));
    }
    assert!(coordinator.contains("commit_journaled_accepted_proposal_apply"));
    assert!(include_str!("../undo_redo.rs").contains("commit_journaled_with_links_and_inverse"));
    assert!(
        !include_str!("../operation_application_batch.rs")
            .contains("commit_journaled_with_links_and_inverse")
    );
    for rev_i05 in [
        include_str!("../../revision/impact.rs"),
        include_str!("../../revision/impact_analysis.rs"),
        include_str!("../../revision/release.rs"),
        include_str!("../../revision/release_projection.rs"),
        include_str!("../../revision/release_transaction.rs"),
    ] {
        assert!(!rev_i05.contains("prepare_revision_design_commit("));
        assert!(!rev_i05.contains("evaluate_design_mutation_authority("));
        assert!(!rev_i05.contains("commit_journaled_with_links_and_inverse("));
    }
    for source in [
        include_str!("../operation_application.rs"),
        include_str!("../operation_application_batch.rs"),
        include_str!("../operation_application_dispatch.rs"),
        include_str!("../project_resolver.rs"),
    ] {
        assert!(!source.contains("DependencySnapshotData"));
        assert!(!source.contains("ImpactEvaluationData"));
        assert!(!source.contains("ConfigurationBaselineData"));
    }
}
