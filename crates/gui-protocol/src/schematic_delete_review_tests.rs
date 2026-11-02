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
