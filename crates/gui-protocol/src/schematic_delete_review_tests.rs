//! Complete native selections stay frozen across metadata-only Proposal revisions.
use super::*;
use crate::schematic_authoring::delete_review::DeleteReview;
use crate::{
    AuthoredSelectionClass as Class, AuthoredSelectionIdentity as Id, CompoundSelection, PaneId,
    SelectionSubject, SelectionTarget,
};
use eda_engine::api::native_write::schematic_delete_proposal::{
    DeletedObject, NativeProposalAction as Action, NativeProposalResponse,
};
use eda_engine::substrate::ProposalStatus;

fn workspace(
    model: &eda_engine::substrate::DesignModel,
    root: &std::path::Path,
) -> crate::ReviewWorkspaceState {
    let mut state = crate::load_fixture_workspace_state();
    state.scene.project_uuid = model.project.project_id.to_string();
    state.scene.source_revision = model.model_revision.0.clone();
    state.library =
        crate::library_browser::LibraryBrowser::capture(std::sync::Arc::new(model.clone()));
    state.schematic_authoring = SchematicAuthoring::from_model(model).unwrap();
    state.schematic_documents = crate::native_schematic_scene::project_native_schematic_documents(
        model,
        root,
        &state.ui.layout,
    )
    .unwrap()
    .unwrap();
    state.schematic_scene = state.schematic_documents.scene_for_pane(PaneId(1)).cloned();
    state.ui.layout.focused = PaneId(1);
    let sheet = &state.schematic_authoring.source.as_ref().unwrap().sheets
        [&state.schematic_authoring.sheet.unwrap()];
    let members = sheet
        .symbols
        .keys()
        .take(2)
        .map(|id| Id {
            class: Class::Symbol,
            id: *id,
            instance_path: vec![],
        })
        .collect::<Vec<_>>();
    assert_eq!(members.len(), 2);
    let authority = crate::selection_resolution::NativeSelectionResolution::capture(model).unwrap();
    state.selection = SelectionTarget::Native(Box::new(
        crate::selection_view::build_selection_view(
            model,
            &authority,
            SelectionSubject::Compound(CompoundSelection::new(members, None).unwrap()),
            &state.scene,
            state.schematic_scene.as_ref(),
            |_| true,
        )
        .unwrap(),
    ));
    state
}
fn response(review: &DeleteReview) -> NativeProposalResponse {
    NativeProposalResponse {
        assignment: None,
        disposition: None,
        proposal_id: review.proposal_id,
        status: ProposalStatus::Draft,
        prepared_against: review.basis.revision.clone(),
        current_model_revision: "proposal-metadata-revision".into(),
        transaction_tip: Some(Uuid::new_v4()),
        selected_count: Some(review.targets.len()),
        owned_count: Some(0),
        transaction_id: None,
        deleted: review
            .targets
            .iter()
            .map(|t| DeletedObject {
                id: t.id,
                class: "symbol".into(),
                name: review.source.symbols[&t.id].reference.clone(),
            })
            .collect(),
    }
}
#[test]
fn batch_delete_review_freezes_complete_selection_and_refuses_stale_foreign_or_unsupported_members()
{
    let fixture = isolated_fixture();
    let model = eda_engine::substrate::ProjectResolver::new(&fixture.0)
        .resolve()
        .unwrap();
    let mut state = workspace(&model, &fixture.0);
    let review = DeleteReview::capture(&state).unwrap();
    let Action::CreateDelete {
        targets,
        instance_path,
        ..
    } = review.pending.as_ref().unwrap()
    else {
        panic!("prepare only")
    };
    assert_eq!(targets.len(), 2);
    assert!(instance_path.is_empty());
    assert!(review.response.is_none());
    assert!(!review.ready());
    let selected = state.selection.clone();
    if let SelectionTarget::Native(view) = &mut state.selection {
        view.stale = true;
    }
    assert!(DeleteReview::capture(&state).is_err());
    state.selection = selected.clone();
    if let SelectionTarget::Native(view) = &mut state.selection {
        view.members.insert(Id {
            class: Class::Pin,
            id: Uuid::new_v4(),
            instance_path: vec![],
        });
    }
    assert!(
        DeleteReview::capture(&state)
            .unwrap_err()
            .contains("unsupported")
    );
    state.selection = selected;
    if let SelectionTarget::Native(view) = &mut state.selection {
        view.members.insert(Id {
            class: Class::Wire,
            id: Uuid::new_v4(),
            instance_path: vec![],
        });
    }
    assert!(
        DeleteReview::capture(&state)
            .unwrap_err()
            .contains("source sheets")
    );
    assert_eq!(
        review.targets.len(),
        2,
        "refusal cannot silently shrink a prior request"
    );
}
#[test]
fn batch_delete_review_metadata_refresh_cancel_and_accept_never_reacquire_selection() {
    let fixture = isolated_fixture();
    let model = eda_engine::substrate::ProjectResolver::new(&fixture.0)
        .resolve()
        .unwrap();
    let mut state = workspace(&model, &fixture.0);
    let mut review = DeleteReview::capture(&state).unwrap();
    let frozen = review.targets.clone();
    let source = review.source.clone();
    let mut receipt = response(&review);
    let mut omitted = receipt.clone();
    omitted.deleted.pop();
    assert!(review.observe(omitted).is_err());
    assert_eq!(review.targets, frozen);
    review.observe(receipt.clone()).unwrap();
    state.scene.source_revision = receipt.current_model_revision.clone();
    state.supervision.journal.accepted_transaction_tip =
        receipt.transaction_tip.map(|id| id.to_string());
    state.selection = SelectionTarget::None;
    assert!(
        review.current(&state),
        "metadata revision is rebased without consulting a later selection"
    );
    assert!(review.ready());
    let preview = review.preview_scene();
    assert!(
        preview
            .board_graphics
            .iter()
            .all(|g| !review.deleted_sources().contains(&g.source_object_uuid))
    );
    assert!(review.scene.board_graphics.len() > preview.board_graphics.len());
    assert_eq!(review.source, source);
    review.stage(true).unwrap();
    assert!(
        matches!(review.pending,Some(Action::AcceptApply{proposal_id}) if proposal_id==review.proposal_id)
    );
    assert_eq!(review.targets, frozen);
    assert_eq!(review.source, source);
    review.pending = None;
    let mut accepted = receipt.clone();
    accepted.status = ProposalStatus::Accepted;
    review.observe(accepted).unwrap();
    review.refusal = Some("apply refused after acceptance".into());
    assert!(
        review.accepted() && review.ready(),
        "confirmed Accepted remains explicitly retryable"
    );
    review.stage(true).unwrap();
    assert!(
        matches!(review.pending, Some(Action::AcceptApply { .. })),
        "only another explicit accept stages apply"
    );
    review.pending = None;
    state
        .schematic_authoring
        .source
        .as_mut()
        .unwrap()
        .sheets
        .get_mut(&review.basis.source)
        .unwrap()
        .name
        .push_str(" changed");
    assert!(!review.current(&state));
    assert!(review.closes_locally());
    assert!(
        review.stage(false).is_err(),
        "Accepted cannot submit an illegal Reject"
    );
    review.observe(receipt.clone()).unwrap();
    review.stage(false).unwrap();
    assert!(matches!(review.pending, Some(Action::Reject { .. })));
    receipt.status = ProposalStatus::Rejected;
    review.observe(receipt).unwrap();
    assert!(review.finished());
    assert_eq!(review.source, source);
    assert_eq!(review.targets, frozen);
    assert!(
        review
            .names()
            .contains(&source.symbols[&frozen[0].id].reference)
    );
}

fn erc_finding(state: &mut crate::ReviewWorkspaceState) {
    state.checks = crate::CheckRunReviewState {
        check_run_id: Some(Uuid::new_v4().to_string()),
        project_id: Some(state.scene.project_uuid.clone()),
        model_revision: Some(state.scene.source_revision.clone()),
        profile_id: Some("erc".into()),
        persisted: true,
        finding_count: 1,
        findings: vec![crate::CheckFindingSummary {
            finding_id: Some(Uuid::new_v4().to_string()),
            fingerprint: "erc:qualified-pin".into(),
            domain: "erc".into(),
            source: "erc".into(),
            status: "active".into(),
            rule_id: "unconnected_pin".into(),
            message: "R1 pin 2 is unconnected".into(),
            ..Default::default()
        }],
        ..Default::default()
    };
    state.selection = SelectionTarget::CheckFinding(state.checks.findings[0].fingerprint.clone());
}
#[test]
fn erc_disposition_review_freezes_evidence_and_requires_explicit_accept_or_dismiss() {
    use eda_engine::api::native_write::schematic_delete_proposal::{
        DispositionKind, DispositionSummary,
    };
    let fixture = isolated_fixture();
    let model = eda_engine::substrate::ProjectResolver::new(&fixture.0)
        .resolve()
        .unwrap();
    let mut state = workspace(&model, &fixture.0);
    erc_finding(&mut state);
    let mut review = DeleteReview::capture_disposition(&state, DispositionKind::Waiver).unwrap();
    assert!(
        review.pending.is_none(),
        "typing and canceling create no Proposal"
    );
    assert!(!review.ready());
    assert!(review.stage(true).is_err());
    assert!(review.stage_disposition(&state, "  ").is_err());
    assert!(review.pending.is_none());
    let before = state.schematic_authoring.source.clone();
    // Selection may change while input is open; never retarget the request to that row.
    state.selection = SelectionTarget::CheckFinding("different-finding".into());
    review
        .stage_disposition(&state, " Intentional test point ")
        .unwrap();
    let Action::CreateDisposition {
        check_run_id,
        fingerprint,
        rationale,
        ..
    } = review.pending.as_ref().unwrap()
    else {
        panic!("typed Proposal only")
    };
    assert_eq!(Some(check_run_id.to_string()), state.checks.check_run_id);
    assert_eq!(fingerprint, "erc:qualified-pin");
    assert_eq!(rationale, " Intentional test point ");
    let evidence = review.disposition.as_ref().unwrap();
    let mut receipt = NativeProposalResponse {
        assignment: None,
        proposal_id: review.proposal_id,
        status: ProposalStatus::Draft,
        prepared_against: review.basis.revision.clone(),
        current_model_revision: review.revision.clone(),
        transaction_tip: Some(Uuid::new_v4()),
        selected_count: None,
        owned_count: None,
        deleted: vec![],
        transaction_id: None,
        disposition: Some(DispositionSummary {
            disposition: evidence.disposition,
            disposition_id: Uuid::new_v4(),
            check_run_id: evidence.check_run_id,
            finding_id: evidence.finding_id,
            fingerprint: evidence.fingerprint.clone(),
            rule_id: evidence.rule_id.clone(),
            message: evidence.message.clone(),
            rationale: evidence.rationale.clone(),
        }),
    };
    let mut foreign = receipt.clone();
    foreign.disposition.as_mut().unwrap().fingerprint = "other".into();
    assert!(review.observe(foreign).is_err());
    review.observe(receipt.clone()).unwrap();
    state.supervision.journal.accepted_transaction_tip =
        receipt.transaction_tip.map(|t| t.to_string());
    assert!(review.current(&state));
    assert!(review.ready());
    assert_eq!(
        review.preview_scene(),
        review.scene,
        "disposition never removes circuit geometry"
    );
    review.stage(false).unwrap();
    assert!(matches!(review.pending, Some(Action::Reject { .. })));
    receipt.status = ProposalStatus::Rejected;
    review.observe(receipt.clone()).unwrap();
    assert!(review.finished());
    assert_eq!(state.schematic_authoring.source, before);
    receipt.status = ProposalStatus::Accepted;
    review.observe(receipt).unwrap();
    assert!(review.closes_locally());
    assert!(review.stage(false).is_err());
    review.stage(true).unwrap();
    assert!(matches!(review.pending, Some(Action::AcceptApply { .. })));
}
#[test]
fn erc_disposition_review_refuses_stale_unpersisted_duplicate_or_disposed_evidence() {
    use eda_engine::api::native_write::schematic_delete_proposal::DispositionKind;
    let fixture = isolated_fixture();
    let model = eda_engine::substrate::ProjectResolver::new(&fixture.0)
        .resolve()
        .unwrap();
    let mut state = workspace(&model, &fixture.0);
    erc_finding(&mut state);
    let original = state.checks.clone();
    let mut draft = DeleteReview::capture_disposition(&state, DispositionKind::Deviation).unwrap();
    for variant in 0..6 {
        state.checks = original.clone();
        match variant {
            0 => state.checks.persisted = false,
            1 => state.checks.model_revision = Some("old".into()),
            2 => state.checks.project_id = Some(Uuid::new_v4().to_string()),
            3 => state.checks.findings.push(state.checks.findings[0].clone()),
            4 => state.checks.findings[0].waiver_refs.push("waiver".into()),
            _ => state.checks.findings[0].domain = "drc".into(),
        }
        assert!(DeleteReview::capture_disposition(&state, DispositionKind::Waiver).is_err());
        assert!(draft.stage_disposition(&state, "Intentional").is_err());
        assert!(draft.pending.is_none());
    }
    state.checks = original;
    state.schematic_authoring.disposition_draft = Some(draft);
    state.schematic_authoring.label =
        Some(LabelDraft::new(eda_engine::schematic::LabelKind::Local));
    state.schematic_authoring.picker = true;
    state.schematic_authoring.cancel();
    assert!(state.schematic_authoring.disposition_draft.is_none());
    assert!(state.schematic_authoring.delete_review.is_none());
}

#[test]
fn part_assignment_review_keeps_named_choices_source_and_component_identity_through_acceptance() {
    use crate::schematic_authoring::assignment::AssignmentEvidence;
    use eda_engine::api::native_write::{
        schematic_assignment::{AssignmentChoice, AssignmentOptions},
        schematic_delete_proposal::AssignmentSummary,
    };
    let fixture = isolated_fixture();
    let model = eda_engine::substrate::ProjectResolver::new(&fixture.0)
        .resolve()
        .unwrap();
    let mut state = workspace(&model, &fixture.0);
    let mut review = DeleteReview::capture(&state).unwrap();
    let source = review.source.clone();
    let original = source.symbols.values().next().unwrap().clone();
    let first = AssignmentChoice {
        part: Uuid::new_v4(),
        part_name: "Part A".into(),
        entity: Uuid::new_v4(),
        entity_name: "Entity A".into(),
        gate: Uuid::new_v4(),
        gate_name: "A".into(),
    };
    let second_gate = AssignmentChoice {
        gate: Uuid::new_v4(),
        gate_name: "B".into(),
        ..first.clone()
    };
    let other = AssignmentChoice {
        part: Uuid::new_v4(),
        part_name: "Part B".into(),
        ..first.clone()
    };
    let mut evidence = AssignmentEvidence {
        target: Id {
            id: original.uuid,
            class: Class::Symbol,
            instance_path: vec![],
        },
        options: AssignmentOptions {
            sheet: source.uuid,
            original,
            component_instance: None,
            choices: vec![first.clone(), second_gate.clone(), other.clone()],
        },
        selected: 0,
    };
    evidence.cycle(true);
    assert_eq!(evidence.choice(), Some(&second_gate));
    evidence.cycle(false);
    assert_eq!(evidence.choice(), Some(&other));
    evidence.cycle(false);
    assert_eq!(evidence.choice(), Some(&first));
    assert!(evidence.part_label().contains("Part A"));
    assert!(!evidence.part_label().contains(&first.part.to_string()));
    review.assignment = Some(evidence);
    review.targets.clear();
    review.pending = None;
    assert!(
        review.stage(true).is_err(),
        "choice input cannot accept before Proposal preparation"
    );
    state.selection = SelectionTarget::None;
    review.stage_assignment(&state).unwrap();
    let Some(Action::CreateAssignment {
        symbol, part, gate, ..
    }) = &review.pending
    else {
        panic!("Proposal action only")
    };
    assert_eq!(*symbol, review.assignment.as_ref().unwrap().target.id);
    assert_eq!((*part, *gate), (first.part, first.gate));
    let evidence = review.assignment.as_ref().unwrap();
    let mut receipt = NativeProposalResponse {
        proposal_id: review.proposal_id,
        status: ProposalStatus::Draft,
        prepared_against: review.basis.revision.clone(),
        current_model_revision: review.revision.clone(),
        transaction_tip: review.tip,
        selected_count: None,
        owned_count: None,
        transaction_id: None,
        deleted: vec![],
        disposition: None,
        assignment: Some(AssignmentSummary {
            symbol: evidence.target.id,
            sheet: source.uuid,
            instance_path: vec![],
            component_instance: Uuid::new_v4(),
            reference: evidence.options.original.reference.clone(),
            previous_part: evidence.options.original.part,
            choice: first.clone(),
        }),
    };
    review.observe(receipt.clone()).unwrap();
    assert!(review.ready());
    assert_eq!(
        review.preview_scene(),
        review.scene,
        "assignment does not move or delete geometry"
    );
    let mut wrong = receipt.clone();
    wrong.assignment.as_mut().unwrap().component_instance = Uuid::new_v4();
    assert!(
        review.observe(wrong).is_err(),
        "new component identity stays frozen after Draft"
    );
    receipt.assignment.as_mut().unwrap().choice.part_name = "Renamed library metadata".into();
    review.observe(receipt.clone()).unwrap();
    assert!(
        review.names().contains("Part A"),
        "recovery keeps reviewed labels but matches canonical IDs"
    );
    review.stage(false).unwrap();
    assert!(matches!(review.pending, Some(Action::Reject { .. })));
    let frozen = review.assignment.clone();
    receipt.assignment = None;
    for status in [
        ProposalStatus::Draft,
        ProposalStatus::Accepted,
        ProposalStatus::Applied,
    ] {
        receipt.status = status;
        assert!(
            review.observe(receipt.clone()).is_err(),
            "absent assignment cannot grant edit authority"
        );
    }
    receipt.status = ProposalStatus::Rejected;
    review.observe(receipt).unwrap();
    assert_eq!(
        review.assignment, frozen,
        "terminal cancellation retains reviewed evidence even after source disappears"
    );
    assert!(review.finished());
    assert_eq!(review.source, source);
}
#[test]
fn part_assignment_noop_requires_existing_component_and_stale_preview_never_stages() {
    use crate::schematic_authoring::assignment::AssignmentEvidence;
    use eda_engine::api::native_write::schematic_assignment::{
        AssignmentChoice, AssignmentOptions,
    };
    let fixture = isolated_fixture();
    let model = eda_engine::substrate::ProjectResolver::new(&fixture.0)
        .resolve()
        .unwrap();
    let mut state = workspace(&model, &fixture.0);
    let mut review = DeleteReview::capture(&state).unwrap();
    let mut original = review.source.symbols.values().next().unwrap().clone();
    let choice = AssignmentChoice {
        part: Uuid::new_v4(),
        part_name: "Part".into(),
        entity: Uuid::new_v4(),
        entity_name: "Entity".into(),
        gate: Uuid::new_v4(),
        gate_name: "Main".into(),
    };
    original.part = Some(choice.part);
    original.entity = Some(choice.entity);
    original.gate = Some(choice.gate);
    let mut evidence = AssignmentEvidence {
        target: Id {
            id: original.uuid,
            class: Class::Symbol,
            instance_path: vec![],
        },
        options: AssignmentOptions {
            sheet: review.source.uuid,
            original,
            component_instance: None,
            choices: vec![choice],
        },
        selected: 0,
    };
    assert!(
        evidence.action(review.proposal_id).is_ok(),
        "legacy scalar IDs alone are not a bound ComponentInstance"
    );
    evidence.options.component_instance = Some(Uuid::new_v4());
    assert!(evidence.action(review.proposal_id).is_err());
    evidence.options.component_instance = None;
    review.assignment = Some(evidence);
    review.targets.clear();
    review.pending = None;
    state.scene.source_revision.push_str("-changed");
    assert!(review.stage_assignment(&state).is_err());
    assert!(review.pending.is_none());
    state.schematic_authoring.assignment_draft = Some(review);
    state.schematic_authoring.picker = true;
    state.schematic_authoring.cancel();
    assert!(state.schematic_authoring.assignment_draft.is_none());
    assert!(state.schematic_authoring.delete_review.is_none());
}

#[test]
fn erc_disposition_native_diagnostic_entry_uses_exact_current_producing_checkrun() {
    use crate::schematic_authoring::disposition::DispositionEvidence;
    use eda_engine::api::native_write::schematic_delete_proposal::DispositionKind;
    let fixture = isolated_fixture();
    let model = eda_engine::substrate::ProjectResolver::new(&fixture.0)
        .resolve()
        .unwrap();
    let mut state = workspace(&model, &fixture.0);
    erc_finding(&mut state);
    state.checks.findings[0].index = 999; // Display order cannot become write identity.
    let fingerprint = state.checks.findings[0].fingerprint.clone();
    let authority =
        crate::selection_resolution::NativeSelectionResolution::capture(&model).unwrap();
    let view = crate::selection_artifacts::build_workspace_selection_view(
        &model,
        &authority,
        SelectionSubject::Diagnostic(fingerprint.clone()),
        &state,
    )
    .unwrap();
    assert!(view.members.is_empty(), "Diagnostic owns no authored UUIDs");
    state.selection = SelectionTarget::Native(Box::new(view.clone()));
    for kind in [DispositionKind::Waiver, DispositionKind::Deviation] {
        let evidence = DispositionEvidence::capture(&state, kind).unwrap();
        assert_eq!(evidence.fingerprint, fingerprint);
        assert_eq!(
            Some(evidence.check_run_id.to_string()),
            state.checks.check_run_id
        );
        let mut review = DeleteReview::capture_disposition(&state, kind).unwrap();
        review.stage_disposition(&state, "Intentional").unwrap();
        assert!(
            matches!(&review.pending, Some(Action::CreateDisposition { fingerprint: captured, .. }) if captured == &fingerprint)
        );
    }
    let checks = state.checks.clone();
    for variant in 0..7 {
        state.checks = checks.clone();
        let mut selected = view.clone();
        match variant {
            0 => selected.stale = true,
            1 => selected.project = Uuid::new_v4(),
            2 => selected.revision.0.push_str("-old"),
            3 => state.checks.check_run_id = Some(Uuid::new_v4().to_string()),
            4 => state.checks.model_revision = Some("old".into()),
            5 => selected.subject = SelectionSubject::None,
            _ => state.checks.findings.push(state.checks.findings[0].clone()),
        }
        state.selection = SelectionTarget::Native(Box::new(selected));
        assert!(DispositionEvidence::capture(&state, DispositionKind::Waiver).is_err());
    }
    state.checks = checks;
    state.selection = SelectionTarget::CheckFinding(fingerprint);
    assert!(
        DispositionEvidence::capture(&state, DispositionKind::Waiver).is_ok(),
        "legacy compatible entry stays available"
    );
}

#[path = "schematic_bound_delete_tests.rs"]
mod bound_delete;
