//! Real canonical source drives the shared child form/ghost/immutable review.
use super::*;
use crate::{AuthoringViewBasis, PaneId};
use eda_engine::substrate::{ProjectResolver, ProposalStatus};
#[test]
#[ignore = "requires DATUM_HIERARCHY_TEST_PROJECT canonical project"]
fn child_sheet_form_preview_qualified_identity_and_review_refusals() {
    let root = std::path::PathBuf::from(
        std::env::var_os("DATUM_HIERARCHY_TEST_PROJECT").expect("canonical project"),
    );
    let model = ProjectResolver::new(&root).resolve().unwrap();
    let mut state = crate::load_fixture_workspace_state();
    state.project_unopened = false;
    state.scene.project_uuid = model.project.project_id.to_string();
    state.scene.source_revision = model.model_revision.0.clone();
    state.library =
        crate::library_browser::LibraryBrowser::capture(std::sync::Arc::new(model.clone()));
    state.schematic_authoring = super::super::SchematicAuthoring::from_model(&model).unwrap();
    state.schematic_documents = crate::native_schematic_scene::project_native_schematic_documents(
        &model,
        &root,
        &state.ui.layout,
    )
    .unwrap()
    .unwrap();
    let primary = state
        .schematic_documents
        .scene_for_pane(PaneId(1))
        .unwrap()
        .clone();
    let primary_key = primary.schematic_document.clone().unwrap();
    state.ui.layout.focused = PaneId(0);
    state
        .ui
        .layout
        .set_focused_content(crate::PaneContent::Schematic);
    state
        .schematic_documents
        .panes
        .insert(PaneId(0), primary_key.clone());
    state.ui.layout.focused = PaneId(1);
    let key = state
        .schematic_documents
        .rows
        .iter()
        .find(|r| r.name == "Library access" && r.key.instance_path.is_empty())
        .unwrap()
        .key
        .clone();
    state
        .schematic_documents
        .prepare_scene(&key, &root, &model)
        .unwrap();
    state
        .schematic_documents
        .panes
        .insert(PaneId(1), key.clone());
    state.schematic_scene = Some(primary.clone());
    state.supervision.journal.accepted_transaction_tip =
        eda_engine::revision::transaction_tip(&model.journal).map(|id| id.to_string());
    state.backing = Some(crate::WorkspaceBacking {
        request: crate::LiveReviewRequest {
            project_root: root.clone(),
            board_file: None,
            artifact_path: None,
            net_uuid: None,
            from_anchor_pad_uuid: None,
            to_anchor_pad_uuid: None,
            profile: None,
            kicad_board_source: None,
        },
        board_path: root.join("board/board.json"),
        native_write_verbs: Default::default(),
    });
    let before = state.schematic_authoring.source.clone();
    let mut draft = ChildSheetDraft::capture(&state).unwrap();
    assert!(draft.name(&state, " \n".into()).is_err());
    draft.name(&state, "New filter".into()).unwrap();
    assert_eq!(draft.review.basis.document, Some(key.clone()));
    assert!(draft.review.pending.is_none());
    let at = PointNm {
        x: 12_000_000,
        y: 25_000_000,
    };
    let ghost = draft
        .scene_at(&state.schematic_authoring, &draft.review.scene, &root, at)
        .unwrap();
    assert_eq!(ghost.schematic_document, Some(key.clone()));
    assert!(
        ghost
            .board_graphics
            .iter()
            .any(|g| g.source_object_uuid == draft.request.instance.to_string())
    );
    assert_ne!(key, primary_key, "exercise a nonprimary parent");
    draft.scene = Some(ghost.clone());
    state.schematic_authoring.child_draft = Some(draft.clone());
    assert_eq!(
        crate::camera_scene_for_pane(&state, PaneId(1)),
        Some(&ghost)
    );
    assert_eq!(
        crate::camera_scene_for_pane(&state, PaneId(0)),
        Some(&primary)
    );
    assert_eq!(state.schematic_scene.as_ref(), Some(&primary));
    assert_eq!(state.schematic_authoring.source, before);
    let ids = (
        draft.request.sheet,
        draft.request.definition,
        draft.request.instance,
    );
    let mut review = draft.clone().stage(&state, at).unwrap();
    let summary = review.child_sheet.as_ref().unwrap().summary.clone();
    assert_eq!((summary.sheet, summary.definition, summary.instance), ids);
    assert_eq!(summary.position, Point::new(at.x, at.y));
    assert_eq!(review.basis.document, Some(key.clone()));
    assert!(review.summary().contains("New filter"));
    assert!(
        review
            .names()
            .contains("source + definition + placed instance")
    );
    let response =
        eda_engine::api::native_write::schematic_delete_proposal::NativeProposalResponse {
            board_field_revision: None,
            proposal_id: review.proposal_id,
            status: ProposalStatus::Draft,
            prepared_against: review.revision.clone(),
            current_model_revision: review.revision.clone(),
            transaction_tip: review.tip,
            child_sheet: Some(summary),
            interface_binding: None,
            gate_placement: None,
            project_annotation: None,
            annotation: None,
            fields: None,
            disposition: None,
            assignment: None,
            deleted: vec![],
            selected_count: None,
            owned_count: None,
            transaction_id: None,
        };
    let mut wrong = response.clone();
    wrong.child_sheet.as_mut().unwrap().parent = Uuid::new_v4();
    assert!(review.observe(wrong).is_err());
    review.observe(response).unwrap();
    assert!(review.ready());
    state.schematic_authoring.child_draft = Some(draft.clone());
    state.schematic_authoring.cancel();
    assert!(state.schematic_authoring.child_draft.is_none());
    assert_eq!(state.schematic_authoring.source, before);
    state.scene.source_revision.push_str("-changed");
    assert!(draft.stage(&state, at).is_err());
    state.scene.source_revision = model.model_revision.0.clone();
    let repeated = state
        .schematic_documents
        .rows
        .iter()
        .find(|r| r.name == "Child A")
        .unwrap()
        .key
        .clone();
    state
        .schematic_documents
        .prepare_scene(&repeated, &root, &model)
        .unwrap();
    state.schematic_documents.panes.insert(PaneId(1), repeated);
    assert!(ChildSheetDraft::capture(&state).is_err());
    assert!(
        AuthoringViewBasis::capture(&state, PaneId(1)).is_err(),
        "ordinary source writers remain root-limited"
    );
    assert_eq!(state.schematic_authoring.source, before);
}
