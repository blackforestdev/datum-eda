//! Existing singleton selection configures the same canonical deletion review.
use super::*;
use eda_engine::api::native_write::{
    commit_prepared,
    context::WriteProvenance,
    registry::{NativeWriteContext, find_native_write_verb},
    schematic_assignment,
    schematic_delete_proposal::{self as proposal, NativeProposalRequest},
    schematic_symbols::build_place_schematic_symbol,
};
use eda_engine::substrate::{CommitSource, DesignModel, ProjectResolver};
use std::path::Path;

fn action(model: &mut DesignModel, root: &Path, action: Action) -> NativeProposalResponse {
    let request = NativeProposalRequest {
        project_root: root.into(),
        expected_model_revision: model.model_revision.0.clone(),
        expected_transaction_tip: eda_engine::revision::transaction_tip(&model.journal),
        action,
    };
    proposal::execute(model, &request).unwrap()
}
fn singleton(model: &DesignModel, root: &Path, id: Uuid) -> crate::ReviewWorkspaceState {
    let mut state = workspace(model, root);
    let authority = crate::selection_resolution::NativeSelectionResolution::capture(model).unwrap();
    state.selection = SelectionTarget::Native(Box::new(
        crate::selection_view::build_selection_view(
            model,
            &authority,
            SelectionSubject::Object(Id {
                class: Class::Symbol,
                id,
                instance_path: vec![],
            }),
            &state.scene,
            state.schematic_scene.as_ref(),
            |_| true,
        )
        .unwrap(),
    ));
    state.supervision.journal.accepted_transaction_tip =
        eda_engine::revision::transaction_tip(&model.journal).map(|id| id.to_string());
    state
}
#[test]
fn native_bound_delete_review_routes_only_assigned_singleton_and_preserves_explicit_decision() {
    let fixture = isolated_fixture();
    let root = &fixture.0;
    let mut model = ProjectResolver::new(root).resolve().unwrap();
    let authoring = SchematicAuthoring::from_model(&model).unwrap();
    let choice = authoring
        .choices
        .iter()
        .find(|c| c.name == "Datum single op amp symbol")
        .unwrap();
    let write=(find_native_write_verb("datum.schematic.place_symbol").unwrap().build)(
        &NativeWriteContext{model:&model,project_root:root},
        WriteProvenance::new("test",CommitSource::Test,"bound singleton Delete review"),
        serde_json::json!({"sheet":authoring.sheet.unwrap(),"library_symbol":choice.id,"reference":"U_BOUND_DELETE","value":"review","x_nm":90_000_000,"y_nm":80_000_000})
    ).unwrap();
    let id = write.primary_object_id.unwrap();
    commit_prepared(&mut model, root, write).unwrap();
    model = ProjectResolver::new(root).resolve().unwrap();
    let target = Id {
        class: Class::Symbol,
        id,
        instance_path: vec![],
    };
    let options = schematic_assignment::options(&model, id, &[]).unwrap();
    let ci = options.component_instance.expect(
        "native placement in this fixture resolves the compatible Part and creates its binding",
    );
    assert!(
        options
            .choices
            .iter()
            .any(|choice| Some(choice.part) == options.original.part)
    );
    // The pinned U1/U2 sources are physically bound too. Author a separate
    // unbound source through the canonical builder's explicit optional binding.
    let mut unbound_symbol = options.original.clone();
    unbound_symbol.uuid = Uuid::new_v4();
    unbound_symbol.reference = "U_UNBOUND_DELETE".into();
    unbound_symbol.part = None;
    unbound_symbol.position.x += 10_000_000;
    for pin in &mut unbound_symbol.pins {
        pin.uuid = Uuid::new_v4();
        pin.position.x += 10_000_000;
    }
    for field in &mut unbound_symbol.fields {
        field.uuid = Uuid::new_v4();
        if let Some(position) = &mut field.position {
            position.x += 10_000_000;
        }
    }
    let write = build_place_schematic_symbol(
        &model,
        WriteProvenance::new("test", CommitSource::Test, "unbound singleton fixture"),
        options.sheet,
        &unbound_symbol,
        None,
    )
    .unwrap();
    commit_prepared(&mut model, root, write).unwrap();
    model = ProjectResolver::new(root).resolve().unwrap();
    let unbound_target = Id {
        id: unbound_symbol.uuid,
        ..target.clone()
    };
    let unbound = singleton(&model, root, unbound_symbol.uuid);
    assert!(
        !DeleteReview::singleton_requires_review(&unbound, &unbound_target).unwrap(),
        "unbound singleton retains ordinary direct Delete"
    );
    assert!(DeleteReview::capture(&unbound).is_err());
    let mut state = singleton(&model, root, id);
    assert!(DeleteReview::singleton_requires_review(&state, &target).unwrap());
    let original = state.schematic_authoring.source.clone();
    let original_ci = model.component_instances.clone();
    let mut review = DeleteReview::capture(&state).unwrap();
    assert_eq!(review.targets.len(), 1);
    assert_eq!(review.targets[0].id, id);
    assert!(review.stage(true).is_err(), "must prepare before accepting");
    let draft = action(&mut model, root, review.pending.clone().unwrap());
    assert!(
        draft
            .deleted
            .iter()
            .any(|d| d.id == ci && d.class == "component_instance")
    );
    review.observe(draft).unwrap();
    assert!(review.summary().contains("Component bindings"));
    assert_eq!(review.names(), "U_BOUND_DELETE");
    assert!(!review.summary().contains(&ci.to_string()));
    assert_eq!(
        SchematicAuthoring::from_model(&model).unwrap().source,
        original
    );
    assert_eq!(model.component_instances, original_ci);
    review.stage(false).unwrap();
    let rejected = action(&mut model, root, review.pending.clone().unwrap());
    review.observe(rejected).unwrap();
    assert!(review.finished());
    assert_eq!(
        SchematicAuthoring::from_model(&model).unwrap().source,
        original,
        "dismissal has no design edit"
    );
    assert_eq!(model.component_instances, original_ci);
    state = singleton(&model, root, id);
    let mut review = DeleteReview::capture(&state).unwrap();
    let prepared = action(&mut model, root, review.pending.clone().unwrap());
    review.observe(prepared).unwrap();
    state.scene.source_revision = review.revision.clone();
    state.supervision.journal.accepted_transaction_tip = review.tip.map(|id| id.to_string());
    state.selection = SelectionTarget::None;
    assert!(
        review.current(&state),
        "review retains its frozen subject despite later selection"
    );
    review.stage(true).unwrap();
    assert_eq!(
        model.component_instances, original_ci,
        "staging is not acceptance execution"
    );
    assert!(matches!(review.pending, Some(Action::AcceptApply { .. })));
    state.scene.source_revision.push_str("-changed");
    assert!(!review.current(&state));
    assert!(DeleteReview::singleton_requires_review(&state, &target).is_err());
    let state = singleton(&model, root, id);
    let foreign = Id {
        instance_path: vec![Uuid::new_v4()],
        ..target
    };
    assert!(DeleteReview::singleton_requires_review(&state, &foreign).is_err());
}
