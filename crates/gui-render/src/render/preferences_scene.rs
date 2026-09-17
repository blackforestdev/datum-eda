//! Owned Preferences windows contain only dialog geometry and hit targets.
//! They never resolve or upload the Design workspace hidden behind that dialog.

use super::*;

impl RetainedScene {
    /// An owned dialog has no world-space geometry or spatial hit index.
    pub fn empty() -> Self {
        Self {
            world_vertices: Vec::new().into(),
            world_strokes: Vec::new(),
            draw_commands: Vec::new(),
            world_hit_index: datum_gui_viewport::SpatialHitIndex::new(Vec::new()),
        }
    }
}

impl PreparedScene {
    pub fn from_native_preferences(
        dialog: &datum_gui_protocol::GlobalPreferencesDialogState,
        width: u32,
        height: u32,
        scale_factor: f32,
    ) -> Self {
        let scale = scale_factor.max(0.01);
        let layout = ShellLayout::for_surface(width, height, scale, None);
        let mut quads = Vec::new();
        let mut text = Vec::new();
        let mut hits = Vec::new();
        super::render_preferences_dialog(dialog, &layout, &mut quads, &mut text, &mut hits);
        if (scale - 1.0).abs() > f32::EPSILON {
            scale_text_run_sizes(&mut text, scale);
        }
        let bounds = datum_gui_protocol::SceneBounds {
            min_x: 0,
            min_y: 0,
            max_x: 1,
            max_y: 1,
        };
        let camera = CameraState::fit_to_bounds(&bounds);
        Self {
            scene_viewport: layout.viewport,
            layout,
            hit_regions: hits,
            surface_passes: Vec::new(),
            board_pane_id: datum_gui_protocol::PaneId(0),
            scene_bounds: bounds.clone(),
            camera,
            panel_vertices: Vec::new(),
            menu_overlay_vertices: quads_to_vertices(&quads),
            menu_overlay_text_runs: text,
            viewport_underlay_vertices: Vec::new(),
            viewport_overlay_vertices: Vec::new(),
            board_interaction_vertices: Vec::new(),
            console_overlay_vertices: Vec::new(),
            console_overlay_layout: None,
            visible_draw_commands: Vec::new(),
            text_runs: Vec::new(),
            terminal_graphics: Vec::new(),
            schematic_scene_viewport: None,
            schematic_pane_id: None,
            schematic_bounds: bounds,
            schematic_camera: camera,
            schematic_hover_bounds_nm: None,
            crosshair_cursor_screen: None,
            crosshair_style: Default::default(),
            schematic_underlay_vertices: Vec::new(),
            schematic_overlay_vertices: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_preferences_match_legacy_dialog_without_preparing_workspace() {
        let mut state = crate::global_preferences_dialog_tests::state_with_preferences_open();
        for project in [false, true] {
            if project {
                state.ui.global_preferences.title = "Project Preferences".into();
            }
            for (width, height, scale) in [(960, 720, 1.0), (1050, 810, 1.5)] {
                let retained =
                    RetainedScene::from_workspace_for_surface(&state, width, height, scale);
                for row in [0, 1, 2] {
                    state.ui.global_preferences.scroll_row = row;
                    let legacy = PreparedScene::from_workspace_with_terminal_renderer(
                        &state,
                        width,
                        height,
                        scale,
                        CameraState::fit_to_bounds(&state.scene.bounds),
                        &retained,
                        &[],
                        None,
                        true,
                    );
                    let before = retained_scene_resolve_count();
                    let dialog = PreparedScene::from_native_preferences(
                        &state.ui.global_preferences,
                        width,
                        height,
                        scale,
                    );
                    assert_eq!(retained_scene_resolve_count(), before);
                    assert_eq!(dialog.menu_overlay_vertices, legacy.menu_overlay_vertices);
                    assert_eq!(dialog.menu_overlay_text_runs, legacy.menu_overlay_text_runs);
                    assert!(legacy.hit_regions.ends_with(&dialog.hit_regions));
                    assert!(!dialog.hit_regions.is_empty());
                    assert!(dialog.surface_passes.is_empty());
                    assert!(dialog.visible_draw_commands.is_empty());
                    assert!(dialog.panel_vertices.is_empty());
                    assert!(dialog.text_runs.is_empty());
                    assert!(dialog.terminal_graphics.is_empty());
                    assert!(dialog.console_overlay_vertices.is_empty());
                }
            }
        }
    }

    #[test]
    fn scrolling_changes_dialog_content_and_hit_targets_without_world_geometry() {
        let mut state = crate::global_preferences_dialog_tests::state_with_preferences_open();
        let before =
            PreparedScene::from_native_preferences(&state.ui.global_preferences, 960, 720, 1.0);
        assert!(state.ui.global_preferences.scroll_rows(-1));
        let after =
            PreparedScene::from_native_preferences(&state.ui.global_preferences, 960, 720, 1.0);
        assert_ne!(before.menu_overlay_text_runs, after.menu_overlay_text_runs);
        assert_ne!(before.hit_regions, after.hit_regions);
        let retained = RetainedScene::empty();
        assert!(retained.world_vertices.is_empty());
        assert!(retained.world_strokes.is_empty());
        assert!(retained.draw_commands.is_empty());
    }
}
