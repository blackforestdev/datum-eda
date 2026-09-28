//! Shared source contracts must reject replacement and weak-hint stale geometry.
use super::*;
use crate::{CameraState, PreparedScene};
use datum_gui_protocol::{PaneContent, SelectionTarget};

fn submit(session: &mut RenderSession, state: &ReviewWorkspaceState, epoch: &SourceEpoch) {
    session
        .ensure_sources(state, Some(epoch.revision()), 960, 720, 1.0)
        .unwrap();
}

fn install(session: &mut RenderSession, state: &ReviewWorkspaceState) {
    let prepared = PreparedScene::from_workspace(
        state,
        960,
        720,
        CameraState::fit_to_bounds(&state.scene.bounds),
        session.board().unwrap(),
    )
    .unwrap();
    session.install_prepared(
        prepared,
        Preparation::workspace(state, session.board().unwrap(), [960, 720]),
    );
}

#[test]
fn equal_identity_length_replacement_and_unknown_producer_never_reuse_old_payload() {
    let mut state = datum_gui_protocol::load_fixture_workspace_state();
    let mut epoch = SourceEpoch::default();
    let mut session = RenderSession::default();
    submit(&mut session, &state, &epoch);
    install(&mut session, &state);
    let old = session.board().unwrap().clone();
    let old_pointer = session.pointer_generation();
    state.scene.tracks[0].width_nm += 10_000;
    epoch.replaced();
    submit(&mut session, &state, &epoch);
    assert_ne!(session.board().unwrap(), &old);
    assert!(session.prepared().is_none());
    assert!(!session.update_pointer(render_input::PointerUpdate {
        generation: old_pointer,
        cursor: None,
        hover: None,
        style: datum_gui_protocol::CrosshairStyle::None,
    }));
    assert_eq!(
        session.board().unwrap(),
        &RetainedScene::from_workspace(&state, 960, 720)
    );
    let current = session.board().unwrap().clone();
    session.ensure_sources(&state, None, 960, 720, 1.0).unwrap();
    assert!(!std::sync::Arc::ptr_eq(
        &current.world_hit_index,
        &session.board().unwrap().world_hit_index
    ));
    let foreign = SourceEpoch::default();
    let current = session.board().unwrap().clone();
    submit(&mut session, &state, &foreign);
    assert!(!std::sync::Arc::ptr_eq(
        &current.world_hit_index,
        &session.board().unwrap().world_hit_index
    ));
}

#[test]
fn shared_selection_history_and_filter_validation_ignore_weak_editor_hints() {
    let mut state = datum_gui_protocol::load_fixture_workspace_state();
    let epoch = SourceEpoch::default();
    let mut session = RenderSession::default();
    state.selection = SelectionTarget::None;
    submit(&mut session, &state, &epoch);
    let original = session.board().unwrap().clone();
    state.selection = SelectionTarget::AuthoredObject(state.scene.tracks[0].object_id.clone());
    submit(&mut session, &state, &epoch);
    assert_eq!(
        session.board().unwrap(),
        &RetainedScene::from_workspace(&state, 960, 720)
    );
    state.selection = SelectionTarget::None;
    let before = crate::retained_scene_resolve_count();
    submit(&mut session, &state, &epoch);
    assert_eq!(
        before,
        crate::retained_scene_resolve_count(),
        "shared owner restores history"
    );
    assert!(std::sync::Arc::ptr_eq(
        &original.world_hit_index,
        &session.board().unwrap().world_hit_index
    ));
    install(&mut session, &state);
    state.ui.filters.show_authored = false;
    submit(&mut session, &state, &epoch);
    assert!(session.prepared().is_none());
    assert_eq!(
        session.board().unwrap(),
        &RetainedScene::from_workspace(&state, 960, 720)
    );
}

#[test]
fn view_changes_preserve_independent_geometry_and_rebuild_dependent_geometry() {
    for independent in [false, true] {
        let mut state = datum_gui_protocol::load_fixture_workspace_state();
        if independent {
            state.scene.unrouted_primitives.clear();
            state
                .scene
                .component_graphics
                .retain(|g| !(g.closed && g.render_role == "component_mechanical"));
        }
        if !independent {
            // Existing retained-resize negative: mechanical dash/gap geometry
            // depends on the reference projection even when the path is empty.
            state
                .scene
                .component_graphics
                .push(datum_gui_protocol::ComponentGraphicPrimitive {
                    graphic_id: "resize-dependent".into(),
                    component_uuid: "resize-component".into(),
                    layer_id: None,
                    primitive_kind: "polyline".into(),
                    render_role: "component_mechanical".into(),
                    width_nm: Some(100_000),
                    closed: true,
                    path: Vec::new(),
                    holes: Vec::new(),
                });
        }
        let epoch = SourceEpoch::default();
        let mut session = RenderSession::default();
        submit(&mut session, &state, &epoch);
        let original = session.board().unwrap().clone();
        assert_eq!(original.can_reuse_for_surface_resize(), independent);
        session.resize_content();
        session
            .ensure_sources(&state, Some(epoch.revision()), 1280, 800, 1.25)
            .unwrap();
        assert_eq!(
            std::sync::Arc::ptr_eq(
                &original.world_hit_index,
                &session.board().unwrap().world_hit_index
            ),
            independent
        );
        assert_eq!(
            session.board().unwrap(),
            &RetainedScene::from_workspace_for_surface(&state, 1280, 800, 1.25)
        );
    }
}

#[test]
fn schematic_pane_admission_and_same_revision_replacement_are_owned_dependencies() {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../engine/testdata/import/kicad/simple-demo.kicad_sch");
    let schematic = datum_gui_protocol::load_kicad_schematic_workspace_state(&path).unwrap();
    let mut state = datum_gui_protocol::load_fixture_workspace_state();
    state.schematic_scene = Some(schematic.scene);
    state.ui.layout.set_focused_content(PaneContent::Schematic);
    let mut epoch = SourceEpoch::default();
    let mut session = RenderSession::default();
    submit(&mut session, &state, &epoch);
    let original = session.schematic().unwrap().clone();
    epoch.replaced(); // Import source_revision can encode identity rather than content.
    submit(&mut session, &state, &epoch);
    assert!(!std::sync::Arc::ptr_eq(
        &original.world_hit_index,
        &session.schematic().unwrap().world_hit_index
    ));
    session.schematic = None;
    session.schematic_history.construction_error = Some("schematic admission failure".into());
    for _ in 0..2 {
        assert!(
            session
                .ensure_sources(&state, Some(epoch.revision()), 960, 720, 1.0)
                .is_err()
        );
        assert!(
            session.schematic_history.construction_error.is_some(),
            "unchanged failed input requires explicit retry"
        );
    }
    session.retry_content();
    submit(&mut session, &state, &epoch);
    assert!(session.schematic().is_some());
    state.schematic_scene = None;
    submit(&mut session, &state, &epoch);
    assert!(session.schematic().is_none());
    assert!(session.prepared().is_none());
}

#[test]
fn admitted_active_key_and_overlap_refusal_preserve_damage_and_allow_retry() {
    let state = datum_gui_protocol::load_fixture_workspace_state();
    let epoch = SourceEpoch::default();
    let mut session = RenderSession::default();
    submit(&mut session, &state, &epoch);
    install(&mut session, &state);
    let observer = session.board().unwrap().geometry_observer();
    let bytes = observer.document_cpu_payload_bytes();
    let pressure = observer
        .try_charge_document_storage(64 * 1024 * 1024 - bytes)
        .unwrap();
    let panes = [TerminalPaneRenderState {
        session_id: "producer".into(),
        focused: true,
        lane: &state.ui.terminal,
        snapshot: crate::terminal_core_render::test_snapshot(b"damage before refusal"),
        damage: vec![datum_terminal_core::Damage::Full],
    }];
    let view = WorkspaceView {
        source_revision: Some(epoch.revision()),
        width: 960,
        height: 720,
        scale: 1.0,
        camera: CameraState::fit_to_bounds(&state.scene.bounds),
        schematic_camera: None,
        include_preferences_overlay: false,
        single_terminal_snapshot: false,
        pane_cameras: &[],
    };
    assert!(
        session
            .begin_workspace_preparation(&state, &view, &panes)
            .is_err()
    );
    assert!(session.prepared().is_none());
    assert!(session.has_pending_frame());
    assert_eq!(
        session.restore_terminal_damage(),
        vec![("producer".into(), vec![datum_terminal_core::Damage::Full])]
    );
    drop(pressure);
    submit(&mut session, &state, &epoch);
    session.check_content_budget().unwrap();
}
