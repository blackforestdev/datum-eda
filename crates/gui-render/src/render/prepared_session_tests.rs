//! Publication belongs to the immutable projection acknowledged by a native receipt.
use super::*;
use crate::{CameraState, PreparedScene};

fn scene(menu: bool) -> PreparedScene {
    let mut state = datum_gui_protocol::load_fixture_workspace_state();
    state.ui.active_menu = menu.then(|| "File".to_owned());
    let retained = RetainedScene::from_workspace(&state, 1280, 800);
    PreparedScene::from_workspace(
        &state,
        1280,
        800,
        CameraState::fit_to_bounds(&state.scene.bounds),
        &retained,
    )
    .unwrap()
}

fn install(session: &mut RenderSession, scene: PreparedScene) {
    let state = datum_gui_protocol::load_fixture_workspace_state();
    session.ensure_board(&state, 1280, 800, 1.0);
    session.install_prepared(
        scene,
        Preparation::workspace(&state, session.board().unwrap(), [1280, 800]),
    );
}

#[test]
fn capture_and_failure_cannot_publish_native_hits_or_consume_their_storage() {
    let mut session = RenderSession::default();
    let prepared = scene(true);
    let allocation = prepared.hit_regions.as_ptr();
    assert!(!prepared.hit_regions.is_empty());
    install(&mut session, prepared);
    for (native, success) in [(false, true), (true, false)] {
        let receipt = session.prepare_frame(1, 1, 1, native, 1280, 800).unwrap();
        assert!(!session.complete_frame(receipt, 1, 1, 1, success));
        assert!(session.take_published_frame().is_none());
        assert_eq!(session.prepared().unwrap().hit_regions.as_ptr(), allocation);
    }
    let receipt = session.prepare_frame(1, 1, 1, true, 1280, 800).unwrap();
    assert!(session.complete_frame(receipt, 1, 1, 1, true));
    let (hits, _) = session.take_published_frame().unwrap();
    assert_eq!(
        hits.as_ptr(),
        allocation,
        "publish moves the existing hit allocation"
    );
    assert!(session.prepared().unwrap().hit_regions.is_empty());
    let exposure = session.prepare_frame(1, 1, 1, true, 1280, 800).unwrap();
    assert!(session.complete_frame(exposure, 1, 1, 1, true));
    assert!(
        session.take_published_frame().is_none(),
        "cached exposure must not replace visible hits with an empty map"
    );
}

#[test]
fn old_completion_cannot_publish_a_newer_prepared_projection() {
    let mut session = RenderSession::default();
    let original = scene(true);
    let original_hits = original.hit_regions.as_ptr();
    install(&mut session, original);
    let old = session.prepare_frame(1, 1, 1, true, 1280, 800).unwrap();
    let replacement = scene(false);
    let expected = replacement.hit_regions.as_ptr();
    install(&mut session, replacement);
    assert!(session.complete_frame(old, 1, 1, 1, true));
    assert!(session.has_pending_frame());
    assert_eq!(
        session.take_published_frame().unwrap().0.as_ptr(),
        original_hits
    );
    let current = session.prepare_frame(1, 1, 1, true, 1280, 800).unwrap();
    assert!(session.complete_frame(current, 1, 1, 1, true));
    assert_eq!(session.take_published_frame().unwrap().0.as_ptr(), expected);
}

#[test]
fn stronger_changes_evict_preparation_and_pointer_cannot_recreate_it() {
    let state = datum_gui_protocol::load_fixture_workspace_state();
    let mut session = RenderSession::default();
    install(&mut session, scene(false));
    let receipt = session.prepare_frame(1, 1, 1, true, 1280, 800).unwrap();
    assert!(session.complete_frame(receipt, 1, 1, 1, true));
    let generation = session.pointer_generation().unwrap();
    session.composition_changed();
    assert!(!session.update_pointer(render_input::PointerUpdate {
        generation: Some(generation),
        cursor: state.ui.cursor_pos,
        hover: state.ui.hovered_object.as_ref(),
        style: state.ui.crosshair_style,
    }));
    assert!(session.prepared().is_none());
    assert!(!session.interaction_only_damage());
    install(&mut session, scene(false));
    session.resize_content();
    assert!(session.prepared().is_none());
    install(&mut session, scene(false));
    session.clear_content();
    assert!(session.prepared().is_none());
}

#[test]
fn replacement_target_requires_new_hit_projection_after_cpu_owner_transfer() {
    let mut session = RenderSession::default();
    install(&mut session, scene(true));
    let receipt = session.prepare_frame(1, 1, 1, true, 1280, 800).unwrap();
    assert!(session.complete_frame(receipt, 1, 1, 1, true));
    assert!(session.prepared().unwrap().hit_regions.is_empty());
    session.retire_target();
    assert!(session.prepared().is_none());
    assert!(session.take_published_frame().is_none());
    assert!(session.has_pending_frame());
    install(&mut session, scene(true));
    let receipt = session.prepare_frame(1, 2, 2, true, 1280, 800).unwrap();
    assert!(session.complete_frame(receipt, 1, 2, 2, true));
    assert!(!session.take_published_frame().unwrap().0.is_empty());
}
