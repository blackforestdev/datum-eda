use super::*;
use crate::{CameraState, PreparedScene};

fn workspace() -> ReviewWorkspaceState {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../engine/testdata/import/kicad/simple-demo.kicad_sch");
    let schematic = datum_gui_protocol::load_kicad_schematic_workspace_state(&path).unwrap();
    let mut state = datum_gui_protocol::load_fixture_workspace_state();
    state.schematic_scene = Some(schematic.scene);
    state.ui.layout.set_focused_content(PaneContent::Schematic);
    state.ui.layout.focus_next();
    state.ui.layout.set_focused_content(PaneContent::Board);
    state
}

fn install(session: &mut RenderSession, state: &ReviewWorkspaceState) {
    assert!(session.ensure_board(state, 1600, 1000, 1.0));
    assert!(session.ensure_schematic(state, 1600, 1000, 1.0));
    let scene = PreparedScene::from_workspace_for_surface(
        state,
        1600,
        1000,
        1.0,
        CameraState::fit_to_bounds(&state.scene.bounds),
        session.board().unwrap(),
    )
    .unwrap();
    session.install_prepared(
        scene,
        Preparation::workspace(state, session.board().unwrap(), [1600, 1000]),
    );
}

fn acknowledge(session: &mut RenderSession) {
    let plan = session.prepare_frame(1, 1, 1, true, 1600, 1000).unwrap();
    assert!(session.complete_frame(plan, 1, 1, 1, true));
}

#[test]
fn pointer_capability_rejects_stale_foreign_and_replaced_content() {
    let state = workspace();
    let mut session = RenderSession::default();
    install(&mut session, &state);
    let old = session.pointer_generation().unwrap();
    let input = |generation| PointerUpdate {
        generation: Some(generation),
        cursor: None,
        hover: None,
        style: CrosshairStyle::None,
    };
    acknowledge(&mut session);
    assert!(session.update_pointer(input(old)));
    assert!(session.interaction_only_damage());
    session.composition_changed();
    assert!(!session.update_pointer(input(old)));
    assert!(!session.interaction_only_damage());
    install(&mut session, &state);
    let current = session.pointer_generation().unwrap();
    assert_ne!(old, current);
    let revision = session.content_revision();
    assert!(!session.update_pointer(input(old)));
    assert_eq!(session.content_revision(), revision);
    let mut other = RenderSession::default();
    install(&mut other, &state);
    assert!(!session.update_pointer(input(other.pointer_generation().unwrap())));
    assert!(session.update_pointer(input(current)));
    assert!(
        !session.interaction_only_damage(),
        "full input dominates pointer"
    );
    session.clear_content();
    assert!(!session.update_pointer(input(current)));
    assert!(session.prepared().is_none());
}

#[test]
fn shared_pointer_projection_matches_previous_board_and_schematic_geometry() {
    let mut state = workspace();
    state.selection = datum_gui_protocol::SelectionTarget::AuthoredObject("selection".into());
    let mut session = RenderSession::default();
    install(&mut session, &state);
    acknowledge(&mut session);
    let generation = session.pointer_generation().unwrap();
    let base = session.prepared().unwrap().clone();
    let resolves = crate::retained_scene_resolve_count();
    for surface in [PaneContent::Board, PaneContent::Schematic] {
        let retained = match surface {
            PaneContent::Board => session.board().unwrap(),
            _ => session.schematic().unwrap(),
        };
        let id = retained
            .world_hit_index
            .regions()
            .iter()
            .find_map(|r| match &r.target {
                crate::HitTarget::AuthoredObject(id) => Some(id.clone()),
                _ => None,
            })
            .unwrap();
        let view = base
            .surface_passes()
            .iter()
            .find(|p| match surface {
                PaneContent::Board => p.surface == crate::SceneSurface::Board,
                _ => p.surface == crate::SceneSurface::Schematic,
            })
            .unwrap()
            .scene_viewport;
        for style in [
            CrosshairStyle::Local,
            CrosshairStyle::FullViewport,
            CrosshairStyle::None,
        ] {
            for cursor in [
                Some(ScreenPointPx {
                    x: view.x + view.width / 2.0,
                    y: view.y + view.height / 2.0,
                }),
                None,
            ] {
                state.ui.cursor_pos = cursor;
                state.ui.crosshair_style = style;
                state.ui.hovered_object = Some(HoverTarget {
                    object_id: id.clone(),
                    surface,
                });
                let mut expected = base.clone();
                expected.refresh_interaction(&state, session.board().unwrap());
                assert!(session.update_pointer(PointerUpdate {
                    generation: Some(generation),
                    cursor,
                    style,
                    hover: state.ui.hovered_object.as_ref(),
                }));
                assert_eq!(session.prepared().unwrap(), &expected);
                assert_eq!(session.pointer_generation(), Some(generation));
            }
        }
    }
    assert_eq!(crate::retained_scene_resolve_count(), resolves);
    assert!(session.interaction_only_damage());
}

#[test]
fn retained_schematic_hit_bounds_equal_source_path_bounds_for_every_eligible_graphic() {
    let state = workspace();
    let mut session = RenderSession::default();
    install(&mut session, &state);
    let retained = session.schematic().unwrap();
    let source = state.schematic_scene.as_ref().unwrap();
    let mut count = 0;
    for graphic in &source.board_graphics {
        if graphic.schematic_hit_kind().is_none() || graphic.path.is_empty() {
            continue;
        }
        assert_eq!(
            crate::interaction_overlay::board_hover_bounds(retained, &graphic.object_id),
            crate::interaction_overlay::schematic_symbol_bounds(source, &graphic.object_id)
        );
        count += 1;
    }
    assert!(count > 0);
}

#[test]
fn pointer_arriving_during_an_attempt_prevents_restoring_its_old_projection() {
    let mut state = workspace();
    let mut session = RenderSession::default();
    install(&mut session, &state);
    let old = session.prepare_frame(1, 1, 1, true, 1600, 1000).unwrap();
    assert!(session.pointer_generation().is_none());
    state.ui.cursor_pos = Some(ScreenPointPx { x: 300.0, y: 250.0 });
    assert!(!session.update_pointer(PointerUpdate {
        generation: None,
        cursor: state.ui.cursor_pos,
        hover: None,
        style: CrosshairStyle::Local,
    }));
    assert!(session.complete_frame(old, 1, 1, 1, true));
    assert!(session.prepared().is_none());
    assert!(session.has_pending_frame());
    assert!(!session.interaction_only_damage());
    install(&mut session, &state);
    assert_eq!(
        session.prepared().unwrap().crosshair_cursor_screen,
        Some((300.0, 250.0))
    );
}

#[test]
fn effective_board_hover_transitions_require_full_content_and_text_preparation() {
    let mut state = workspace();
    state.selection = datum_gui_protocol::SelectionTarget::None;
    state.ui.hovered_object = None;
    let mut session = RenderSession::default();
    install(&mut session, &state);
    acknowledge(&mut session);
    let generation = session.pointer_generation();
    let pad_id = state.scene.pads.first().unwrap().object_id.clone();
    state.ui.hovered_object = Some(HoverTarget {
        object_id: pad_id,
        surface: PaneContent::Board,
    });
    assert!(!session.update_pointer(PointerUpdate {
        generation,
        cursor: None,
        hover: state.ui.hovered_object.as_ref(),
        style: CrosshairStyle::Local,
    }));
    assert!(session.board().is_none());
    assert!(session.prepared().is_none());
    assert!(!session.interaction_only_damage());
    install(&mut session, &state);
    acknowledge(&mut session);
    let generation = session.pointer_generation();
    assert!(session.update_pointer(PointerUpdate {
        generation,
        cursor: Some(ScreenPointPx { x: 350.0, y: 250.0 }),
        hover: state.ui.hovered_object.as_ref(),
        style: CrosshairStyle::FullViewport,
    }));
    assert!(session.interaction_only_damage());
    state.ui.hovered_object = None;
    assert!(!session.update_pointer(PointerUpdate {
        generation,
        cursor: None,
        hover: None,
        style: CrosshairStyle::Local,
    }));
    assert!(session.board().is_none());
    assert!(session.prepared().is_none());
    install(&mut session, &state);
    // Full workspace submission also catches a changed hover, independent of hints.
    state.ui.hovered_object = Some(HoverTarget {
        object_id: state.scene.pads.first().unwrap().object_id.clone(),
        surface: PaneContent::Board,
    });
    session
        .ensure_sources(&state, None, 1600, 1000, 1.0)
        .unwrap();
    assert_eq!(
        session.board().unwrap(),
        &RetainedScene::from_workspace_for_surface(&state, 1600, 1000, 1.0)
    );
    assert!(session.prepared().is_none());
}

#[test]
fn retained_history_keys_distinguish_equal_length_effective_board_hover() {
    let mut state = workspace();
    state.selection = datum_gui_protocol::SelectionTarget::None;
    state.ui.hovered_object = Some(HoverTarget {
        object_id: "pad:a".into(),
        surface: PaneContent::Board,
    });
    let before = RetainedSceneCacheKey::for_workspace(&state, 1600, 1000, 1.0);
    state.ui.hovered_object.as_mut().unwrap().object_id = "pad:b".into();
    let after = RetainedSceneCacheKey::for_workspace(&state, 1600, 1000, 1.0);
    assert_ne!(before, after);
    state.selection = datum_gui_protocol::SelectionTarget::AuthoredObject("selected".into());
    let disabled = RetainedSceneCacheKey::for_workspace(&state, 1600, 1000, 1.0);
    state.ui.hovered_object = None;
    assert_eq!(
        disabled,
        RetainedSceneCacheKey::for_workspace(&state, 1600, 1000, 1.0)
    );
}
