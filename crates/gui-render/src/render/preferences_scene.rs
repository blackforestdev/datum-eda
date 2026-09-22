//! Owned Preferences windows contain only dialog geometry and hit targets.
//! They never resolve or upload the Design workspace hidden behind that dialog.

use super::*;

impl RetainedScene {
    /// An owned dialog has no world-space geometry or spatial hit index.
    pub fn empty() -> Self {
        Self {
            surface_size_independent: true,
            world_vertices: Vec::new().into(),
            world_strokes: std::sync::Arc::from([]),
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
        let mut scroll = datum_gui_viewport::scroll::ScrollViewport::default();
        Self::from_native_preferences_scrolled(
            dialog,
            width,
            height,
            scale_factor,
            &mut scroll,
            Some(dialog.scroll_row),
        )
    }

    pub fn from_native_preferences_scrolled(
        dialog: &datum_gui_protocol::GlobalPreferencesDialogState,
        width: u32,
        height: u32,
        scale_factor: f32,
        scroll: &mut datum_gui_viewport::scroll::ScrollViewport,
        reveal_row: Option<usize>,
    ) -> Self {
        Self::from_native_preferences_cached(
            dialog,
            width,
            height,
            scale_factor,
            scroll,
            reveal_row,
            &mut ControlMeshCache::default(),
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn from_native_preferences_cached(
        dialog: &datum_gui_protocol::GlobalPreferencesDialogState,
        width: u32,
        height: u32,
        scale_factor: f32,
        scroll: &mut datum_gui_viewport::scroll::ScrollViewport,
        reveal_row: Option<usize>,
        controls: &mut ControlMeshCache,
    ) -> Self {
        let scale = scale_factor.max(0.01);
        let layout = ShellLayout::for_surface(width, height, scale, None);
        let mut quads = Vec::new();
        let mut text = Vec::new();
        let mut hits = Vec::new();
        super::render_preferences_dialog_scrolled(
            dialog,
            &layout,
            &mut ControlPainter::new(&mut quads, controls, scale),
            &mut text,
            &mut hits,
            scroll,
            reveal_row,
        );
        Self::from_dialog_parts(layout, quads, text, hits, scale, (width, height))
    }

    /// Shared native-dialog envelope: no hidden workspace preparation or hits.
    pub(crate) fn from_dialog_parts(
        layout: ShellLayout,
        mut quads: Vec<Quad>,
        mut text: Vec<TextRun>,
        mut hits: Vec<HitRegion>,
        scale: f32,
        surface_size: (u32, u32),
    ) -> Self {
        // Layout rounds logical extents before scaling. The native surface is
        // the final ancestor, including the fractional edge that can differ
        // from that rounded layout. Paint and pointer targets share its bounds.
        crate::hit_clipping::clip_content(
            &mut quads,
            &mut text,
            &mut hits,
            0,
            0,
            0,
            RectPx {
                x: 0.0,
                y: 0.0,
                width: surface_size.0 as f32,
                height: surface_size.1 as f32,
            },
        );
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

impl Renderer {
    pub fn prepare_native_preferences_scrolled(
        &mut self,
        dialog: &datum_gui_protocol::GlobalPreferencesDialogState,
        width: u32,
        height: u32,
        scale_factor: f32,
        scroll: &mut datum_gui_viewport::scroll::ScrollViewport,
        reveal_row: Option<usize>,
    ) -> PreparedScene {
        PreparedScene::from_native_preferences_cached(
            dialog,
            width,
            height,
            scale_factor,
            scroll,
            reveal_row,
            &mut self.control_meshes,
        )
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
                    let surface = RectPx {
                        x: 0.0,
                        y: 0.0,
                        width: width as f32,
                        height: height as f32,
                    };
                    let mut legacy_text = legacy.menu_overlay_text_runs.clone();
                    for run in &mut legacy_text {
                        // The old GPU path implicitly clipped at the surface.
                        run.clip_bounds = run.clip_bounds.unwrap_or(surface).intersect(surface);
                    }
                    assert_eq!(dialog.menu_overlay_text_runs, legacy_text);
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
    fn native_dialog_profiles_share_exact_physical_surface_bounds() {
        let mut state = crate::global_preferences_dialog_tests::state_with_preferences_open();
        state.ui.new_project.open = true;
        for (width, height, scale) in [(257, 179, 1.0), (1001, 751, 1.5), (961, 721, 2.0)] {
            let surface = RectPx {
                x: 0.0,
                y: 0.0,
                width: width as f32,
                height: height as f32,
            };
            for profile in ["Global Preferences", "Project Preferences", "New Project"] {
                let scene = if profile == "New Project" {
                    let layout = ShellLayout::for_surface(width, height, scale, None);
                    let (mut quads, mut text, mut hits) = (Vec::new(), Vec::new(), Vec::new());
                    crate::new_project_dialog::render_new_project_dialog(
                        &state.ui.new_project,
                        &layout,
                        true,
                        &mut ControlMeshCache::default(),
                        scale,
                        &mut quads,
                        &mut text,
                        &mut hits,
                    );
                    PreparedScene::from_dialog_parts(
                        layout,
                        quads,
                        text,
                        hits,
                        scale,
                        (width, height),
                    )
                } else {
                    state.ui.global_preferences.title = profile.into();
                    PreparedScene::from_native_preferences(
                        &state.ui.global_preferences,
                        width,
                        height,
                        scale,
                    )
                };
                assert!(!scene.menu_overlay_vertices.is_empty());
                assert!(!scene.hit_regions.is_empty());
                for vertex in &scene.menu_overlay_vertices {
                    assert!(
                        surface.contains(vertex.pos[0], vertex.pos[1]),
                        "{profile}: {vertex:?}"
                    );
                }
                for run in &scene.menu_overlay_text_runs {
                    let clip = run
                        .clip_bounds
                        .expect("native surface is the outer text clip");
                    assert!(surface.contains(clip.x, clip.y));
                    assert!(surface.contains(clip.x + clip.width, clip.y + clip.height));
                }
                for hit in &scene.hit_regions {
                    assert!(surface.contains(hit.rect.x, hit.rect.y));
                    assert!(
                        surface.contains(hit.rect.x + hit.rect.width, hit.rect.y + hit.rect.height)
                    );
                }
            }
        }
    }

    #[test]
    fn fitting_content_does_not_scroll_past_its_end() {
        let mut state = crate::global_preferences_dialog_tests::state_with_preferences_open();
        let before =
            PreparedScene::from_native_preferences(&state.ui.global_preferences, 960, 720, 1.0);
        assert!(state.ui.global_preferences.scroll_rows(-1));
        let after =
            PreparedScene::from_native_preferences(&state.ui.global_preferences, 960, 720, 1.0);
        assert_eq!(before.menu_overlay_text_runs, after.menu_overlay_text_runs);
        assert_eq!(before.hit_regions, after.hit_regions);
        let retained = RetainedScene::empty();
        assert!(retained.world_vertices.is_empty());
        assert!(retained.world_strokes.is_empty());
        assert!(retained.draw_commands.is_empty());
    }
    #[test]
    fn continuous_scroll_clips_rows_and_hits_with_stable_content_bounds() {
        let state = crate::global_preferences_dialog_tests::state_with_preferences_open();
        let dialog = &state.ui.global_preferences;
        for scale in [1.0, 1.5] {
            let mut scroll = datum_gui_viewport::scroll::ScrollViewport::default();
            let before = PreparedScene::from_native_preferences_scrolled(
                dialog,
                960,
                240,
                scale,
                &mut scroll,
                None,
            );
            assert!(scroll.maximum() > 0.0);
            let total = scroll.content_height;
            let row_y = before
                .menu_overlay_text_runs
                .iter()
                .find(|run| run.text == "Console feedback duration")
                .unwrap()
                .y;
            assert!(scroll.wheel(-0.25));
            let after = PreparedScene::from_native_preferences_scrolled(
                dialog,
                960,
                240,
                scale,
                &mut scroll,
                None,
            );
            let moved_y = after
                .menu_overlay_text_runs
                .iter()
                .find(|run| run.text == "Console feedback duration")
                .unwrap()
                .y;
            assert_eq!(row_y - moved_y, 0.25);
            assert_eq!(scroll.content_height, total);
            scroll.set_offset(scroll.maximum());
            let bottom = PreparedScene::from_native_preferences_scrolled(
                dialog,
                960,
                240,
                scale,
                &mut scroll,
                None,
            );
            assert_eq!(scroll.content_height, total);
            for hit in &bottom.hit_regions {
                if matches!(
                    hit.target,
                    HitTarget::GlobalPreferencesSettingName(_)
                        | HitTarget::GlobalPreferencesControl(_)
                        | HitTarget::GlobalPreferencesReset(_)
                ) {
                    assert!(hit.rect.y >= scroll.viewport.y);
                    assert!(
                        hit.rect.y + hit.rect.height <= scroll.viewport.y + scroll.viewport.height
                    );
                }
            }
            assert!(bottom.visible_draw_commands.is_empty());
            assert!(bottom.panel_vertices.is_empty());
            assert_eq!(
                scroll.thumb().unwrap().y + scroll.thumb().unwrap().height,
                scroll.viewport.y + scroll.viewport.height
            );
        }
    }
}
