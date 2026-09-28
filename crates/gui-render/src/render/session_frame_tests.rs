//! Exact snapshot identity, rollback and stale-completion controls without GPU work.
use super::*;
use crate::CameraState;

fn prepare(session: &mut RenderSession, menu: bool) -> *const crate::HitRegion {
    let mut state = datum_gui_protocol::load_fixture_workspace_state();
    state.ui.active_menu = menu.then(|| "File".to_owned());
    let retained = RetainedScene::from_workspace(&state, 1280, 800);
    let scene = PreparedScene::from_workspace(
        &state,
        1280,
        800,
        CameraState::fit_to_bounds(&state.scene.bounds),
        &retained,
    )
    .unwrap();
    let hits = scene.hit_regions.as_ptr();
    let preparation = Preparation::workspace(&state, &retained, [1280, 800]);
    session.board = Some(retained);
    session.install_prepared(scene, preparation);
    hits
}
fn plan(session: &mut RenderSession, native: bool) -> FramePlan {
    session.prepare_frame(1, 2, 3, native, 1280, 800).unwrap()
}
fn target() -> Target {
    Target {
        host: 1,
        device: 2,
        configuration: 3,
    }
}

#[test]
fn completion_publishes_the_rendered_snapshot_while_newer_input_stays_pending() {
    let mut session = RenderSession::default();
    let old_hits = prepare(&mut session, true);
    let old = plan(&mut session, true);
    assert!(session.prepared().is_none());
    assert_eq!(old.scene.hit_regions.as_ptr(), old_hits);
    let new_hits = prepare(&mut session, false);
    assert!(session.complete_submitted(SubmittedFrame(old), 1, 2, 3, true));
    assert_eq!(session.take_published_frame().unwrap().0.as_ptr(), old_hits);
    assert_eq!(session.prepared().unwrap().hit_regions.as_ptr(), new_hits);
    assert!(session.has_pending_frame());
    let current = plan(&mut session, true);
    assert!(session.complete_submitted(SubmittedFrame(current), 1, 2, 3, true));
    assert_eq!(session.take_published_frame().unwrap().0.as_ptr(), new_hits);
    assert!(!session.has_pending_frame());
}

#[test]
fn failed_or_capture_snapshot_restores_storage_without_native_publication() {
    let mut session = RenderSession::default();
    let hits = prepare(&mut session, true);
    for (native, presented) in [(true, false), (false, true)] {
        let attempt = plan(&mut session, native);
        assert!(!session.finish_snapshot(attempt, target(), presented));
        assert_eq!(session.prepared().unwrap().hit_regions.as_ptr(), hits);
        assert!(session.has_pending_frame());
        assert!(session.take_published_frame().is_none());
    }
    let attempt = plan(&mut session, true);
    assert!(session.complete_submitted(SubmittedFrame(attempt), 1, 2, 3, true));
    assert_eq!(session.take_published_frame().unwrap().0.as_ptr(), hits);
    let exposure = plan(&mut session, true);
    assert!(session.complete_submitted(SubmittedFrame(exposure), 1, 2, 3, true));
    assert!(session.take_published_frame().is_none());
}

#[test]
fn dropped_and_retired_plans_force_repreparation_and_cannot_overwrite_new_owner() {
    let mut session = RenderSession::default();
    prepare(&mut session, true);
    drop(plan(&mut session, true));
    assert!(session.has_pending_frame());
    assert!(session.prepared().is_none());
    prepare(&mut session, false);
    let old = plan(&mut session, true);
    session.retire_target();
    let current = prepare(&mut session, true);
    assert!(!session.complete_submitted(SubmittedFrame(old), 1, 2, 3, true));
    assert_eq!(session.prepared().unwrap().hit_regions.as_ptr(), current);
    assert!(session.take_published_frame().is_none());
    let foreign = plan(&mut session, true);
    let mut other = RenderSession::default();
    let own = prepare(&mut other, false);
    assert!(!other.complete_submitted(SubmittedFrame(foreign), 1, 2, 3, true));
    assert_eq!(other.prepared().unwrap().hit_regions.as_ptr(), own);
}

#[test]
fn failure_capture_and_old_presentation_preserve_terminal_damage_until_matching_frame() {
    let state = datum_gui_protocol::load_fixture_workspace_state();
    let panes = [TerminalPaneRenderState {
        session_id: "producer".into(),
        focused: true,
        lane: &state.ui.terminal,
        snapshot: crate::terminal_core_render::test_snapshot(b"leased output"),
        damage: vec![datum_terminal_core::Damage::Full],
    }];
    let mut session = RenderSession::default();
    prepare(&mut session, true);
    session.retain_terminal_damage(&panes);
    for (native, success) in [(true, false), (false, true)] {
        let attempt = plan(&mut session, native);
        assert!(!session.finish_snapshot(attempt, target(), success));
        assert!(!session.restore_terminal_damage().is_empty());
        session.retain_terminal_damage(&panes);
    }
    let old = plan(&mut session, true);
    assert!(!session.restore_terminal_damage().is_empty());
    session.retain_terminal_damage(&panes);
    assert!(session.complete_submitted(SubmittedFrame(old), 1, 2, 3, true));
    assert!(!session.restore_terminal_damage().is_empty());
    session.retain_terminal_damage(&panes);
    prepare(&mut session, false);
    let current = plan(&mut session, true);
    assert!(session.complete_submitted(SubmittedFrame(current), 1, 2, 3, true));
    assert!(session.restore_terminal_damage().is_empty());
}

#[test]
fn missing_world_source_refuses_without_consuming_pending_projection() {
    let mut session = RenderSession::default();
    let hits = prepare(&mut session, false);
    let board = session.board.take().unwrap();
    assert!(session.prepare_frame(1, 2, 3, true, 1280, 800).is_err());
    assert_eq!(session.prepared().unwrap().hit_regions.as_ptr(), hits);
    assert!(session.has_pending_frame());
    assert!(session.take_published_frame().is_none());
    session.board = Some(board);
    let attempt = plan(&mut session, true);
    assert!(session.complete_submitted(SubmittedFrame(attempt), 1, 2, 3, true));
}

#[test]
fn equal_area_extent_change_and_missing_schematic_fail_closed() {
    let mut session = RenderSession::default();
    prepare(&mut session, false);
    assert!(session.prepare_frame(1, 2, 3, true, 800, 1280).is_err());
    assert!(session.prepared().is_some());
    let scene = session.prepared.as_mut().unwrap();
    assert!(!scene.surface_passes.is_empty());
    scene.surface_passes[0].surface = crate::SceneSurface::Schematic;
    assert!(session.prepare_frame(1, 2, 3, true, 1280, 800).is_err());
    assert!(session.prepared().is_some());
    assert!(session.has_pending_frame());
}
