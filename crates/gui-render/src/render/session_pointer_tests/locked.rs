use super::*;

#[test]
fn locked_cursor_cache_matches_full_preparation_and_clears_without_resolve() {
    let mut state = workspace();
    state.selection = datum_gui_protocol::SelectionTarget::None;
    state.scene.components[0].locked = true;
    let target = state.scene.components[0].object_id.clone();
    state.ui.hovered_object = Some(HoverTarget {
        object_id: target,
        surface: PaneContent::Board,
    });
    state.ui.crosshair_style = CrosshairStyle::None;
    let mut session = RenderSession::default();
    install(&mut session, &state);
    let pane = session
        .prepared()
        .unwrap()
        .surface_passes()
        .iter()
        .find(|p| p.surface == crate::SceneSurface::Board)
        .unwrap()
        .scene_viewport;
    state.ui.cursor_pos = Some(ScreenPointPx {
        x: pane.x + pane.width / 2.0,
        y: pane.y + pane.height / 2.0,
    });
    install(&mut session, &state);
    acknowledge(&mut session);
    let initial = session
        .prepared()
        .unwrap()
        .board_interaction_vertices()
        .to_vec();
    assert!(!initial.is_empty());
    let generation = session.pointer_generation();
    let resolves = crate::retained_scene_resolve_count();
    let retained = session.board().unwrap().world_vertices().to_vec();
    let center = state.ui.cursor_pos;
    for style in [
        CrosshairStyle::None,
        CrosshairStyle::Local,
        CrosshairStyle::FullViewport,
    ] {
        for cursor in [
            center,
            Some(ScreenPointPx {
                x: pane.x + 2.0,
                y: pane.y + 2.0,
            }),
            None,
        ] {
            state.ui.cursor_pos = cursor;
            state.ui.crosshair_style = style;
            let mut expected = session.prepared().unwrap().clone();
            expected.refresh_interaction(&state, session.board().unwrap());
            assert!(session.update_pointer(PointerUpdate {
                generation,
                cursor,
                hover: state.ui.hovered_object.as_ref(),
                style
            }));
            assert_eq!(
                session.prepared().unwrap().board_interaction_vertices(),
                expected.board_interaction_vertices()
            );
            assert_eq!(session.pointer_generation(), generation);
        }
    }
    // Window leave and hover clear remove both transient glyphs, including when
    // the ordinary crosshair is disabled; source and retained material stay cached.
    assert!(session.update_pointer(PointerUpdate {
        generation,
        cursor: None,
        hover: None,
        style: CrosshairStyle::None
    }));
    assert!(
        session
            .prepared()
            .unwrap()
            .board_interaction_vertices()
            .is_empty()
    );
    assert_eq!(crate::retained_scene_resolve_count(), resolves);
    assert_eq!(session.board().unwrap().world_vertices(), retained);
    assert!(session.interaction_only_damage());
}
