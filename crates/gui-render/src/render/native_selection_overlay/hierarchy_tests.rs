//! Interface aliases must remain visible when retained GPU masks own ordinary ink.
use super::*;

#[test]
#[ignore = "requires DATUM_HIERARCHY_TEST_PROJECT canonical repeated-instance project"]
fn hierarchy_interface_ink_survives_retained_mask_selection() {
    use datum_gui_protocol::schematic_documents::SchematicDocuments;
    use datum_gui_protocol::selection_resolution::NativeSelectionResolution;
    use datum_gui_protocol::{AuthoredSelectionClass as Class, SelectionSubject};
    let root = std::path::PathBuf::from(
        std::env::var_os("DATUM_HIERARCHY_TEST_PROJECT")
            .expect("supply canonical hierarchy project"),
    );
    let model = eda_engine::substrate::ProjectResolver::new(&root)
        .resolve()
        .unwrap();
    let authority = NativeSelectionResolution::capture(&model).unwrap();
    let target = authority
        .hierarchy_targets()
        .values()
        .find(|target| target.identity.class == Class::SheetInterface)
        .unwrap();
    let mut state = datum_gui_protocol::load_fixture_workspace_state();
    let mut documents = SchematicDocuments::from_source(
        model.project.project_id,
        model.model_revision.clone(),
        "Hierarchy".into(),
        authority.snapshot().schematic_source().unwrap().clone(),
    )
    .unwrap();
    documents
        .prepare_scene(&target.parent, &root, &model)
        .unwrap();
    let pane = documents
        .open_beside(&mut state.ui.layout, &target.parent)
        .unwrap();
    let scene = documents.scene_for_pane(pane).unwrap().clone();
    state.scene.project_uuid = model.project.project_id.to_string();
    state.scene.source_revision = model.model_revision.0.clone();
    let view = datum_gui_protocol::selection_view::build_selection_view_for_documents(
        &model,
        &authority,
        SelectionSubject::Object(target.identity.clone()),
        &state.scene,
        &[&scene],
        |_| true,
    )
    .unwrap();
    state.selection = SelectionTarget::Native(Box::new(view));
    state.schematic_documents = documents;
    state.schematic_scene = Some(scene.clone());
    let viewport = RectPx {
        x: 0.0,
        y: 0.0,
        width: 1200.0,
        height: 900.0,
    };
    let camera = CameraState::fit_to_bounds(&scene.bounds);
    let paint = |scene: &BoardReviewSceneV1, mask| {
        let (mut strokes, mut quads, mut text) = (vec![], vec![], vec![]);
        push_native_impl(
            &mut strokes,
            &mut quads,
            &mut text,
            scene,
            &state,
            viewport,
            camera,
            datum_gui_protocol::PaneContent::Schematic,
            mask,
        );
        quads
    };
    let masked = paint(&scene, true);
    assert!(
        !masked.is_empty(),
        "a selected interface needs visible ink in the production mask path"
    );
    assert!(
        masked
            .iter()
            .all(|quad| quad.color == design_tokens::content::SELECTION),
        "interface ink uses the shared selected accent, not the legacy pale mechanical tint"
    );
    assert_eq!(
        masked,
        paint(&scene, false),
        "aliases have no retained mask substitute"
    );
    let mut outline = scene.clone();
    outline.board_text_geometries.clear();
    assert!(
        !paint(&outline, true).is_empty(),
        "actual port outline remains highlighted"
    );
    let mut glyphs = scene.clone();
    glyphs.board_graphics.clear();
    assert!(
        !paint(&glyphs, true).is_empty(),
        "actual name glyphs remain highlighted"
    );
}
