use super::*;
use datum_gui_protocol::{
    ReviewWorkspaceState, WorkspaceLayout, schematic_documents::SchematicDocumentKey,
};

#[test]
fn library_reuse_requires_qualified_native_vacancy_and_preserves_unavailable_bindings() {
    let mut state = ReviewWorkspaceState::unopened();
    state.ui.layout = WorkspaceLayout::board_schematic();
    // Compatibility mode has no bindings even when it has a usable scene.
    assert!(unbound_schematic_pane(&state).is_none());
    let project = eda_engine::substrate::ObjectId::new_v4();
    state.schematic_documents.project = Some(project);
    let pane = unbound_schematic_pane(&state).unwrap();
    assert_eq!(
        state.ui.layout.content_for(pane),
        Some(PaneContent::Schematic)
    );
    // A bound document must survive even when its scene is unavailable.
    state.schematic_documents.panes.insert(
        pane,
        SchematicDocumentKey {
            project,
            schematic: eda_engine::substrate::ObjectId::new_v4(),
            source_sheet: eda_engine::substrate::ObjectId::new_v4(),
            instance_path: Vec::new(),
        },
    );
    assert!(state.schematic_documents.scene_for_pane(pane).is_none());
    assert!(unbound_schematic_pane(&state).is_none());
}
