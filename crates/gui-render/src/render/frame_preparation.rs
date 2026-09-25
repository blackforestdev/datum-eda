//! CPU frame composition with explicit renderer-owned control preparation.
use super::*;
use crate::resource_consumers::{Consumer, FrameConsumers, Stream};

impl PreparedScene {
    #[allow(clippy::too_many_arguments)]
    pub fn from_workspace_with_terminal_renderer(
        state: &ReviewWorkspaceState,
        width: u32,
        height: u32,
        scale_factor: f32,
        camera: CameraState,
        retained_scene: &RetainedScene,
        terminal_panes: &[crate::TerminalPaneRenderState],
        terminal_cache: Option<&mut crate::TerminalRenderCache>,
        global_preferences_native_window: bool,
    ) -> anyhow::Result<Self> {
        Ok({
            Self::from_workspace_with_controls(
                state,
                width,
                height,
                scale_factor,
                camera,
                retained_scene,
                terminal_panes,
                terminal_cache,
                global_preferences_native_window,
                &mut crate::global_preferences_primitives::ControlMeshCache::default(),
            )?
        })
    }
}

impl PreparedScene {
    #[allow(clippy::too_many_arguments)]
    fn from_workspace_with_controls(
        state: &ReviewWorkspaceState,
        width: u32,
        height: u32,
        scale_factor: f32,
        camera: CameraState,
        retained_scene: &RetainedScene,
        terminal_panes: &[crate::TerminalPaneRenderState],
        terminal_cache: Option<&mut crate::TerminalRenderCache>,
        global_preferences_native_window: bool,
        controls: &mut crate::global_preferences_primitives::ControlMeshCache,
    ) -> anyhow::Result<Self> {
        Ok({
            let mut consumers = FrameConsumers::default();
            let scale = scale_factor.max(0.01);
            let layout =
                ShellLayout::for_surface(width, height, scale, dock_height_for_state(state));
            let mut panel_quads = Vec::new();
            let mut menu_overlay_quads = Vec::new();
            let mut menu_overlay_text_runs = Vec::new();
            let mut viewport_underlay_quads = Vec::new();
            let mut viewport_overlay_quads = Vec::new();
            let mut board_interaction_quads = Vec::new();
            let mut text_runs = Vec::new();
            let mut terminal_graphics = Vec::new();
            let mut hit_regions = Vec::new();
            let scene_viewport = layout.scene_viewport(&state.ui.layout);
            let (board_pane_id, schematic_pane_id) =
                coordinate_hit::surface_pane_ids(&layout, &state.ui.layout);
            let board_scene_active = layout
                .viewport_panes(&state.ui.layout)
                .scene_leaf()
                .is_some();

            let board_hover_bounds = state.ui.hovered_object.as_ref().and_then(|hover| {
                (hover.surface == datum_gui_protocol::PaneContent::Board)
                    .then(|| {
                        interaction_overlay::board_hover_bounds(retained_scene, &hover.object_id)
                    })
                    .flatten()
            });
            let schematic_hover_bounds = state.ui.hovered_object.as_ref().and_then(|hover| {
                (hover.surface == datum_gui_protocol::PaneContent::Schematic)
                    .then(|| {
                        state.schematic_scene.as_ref().and_then(|scene| {
                            interaction_overlay::schematic_symbol_bounds(scene, &hover.object_id)
                        })
                    })
                    .flatten()
            });
            let crosshair_cursor_screen = state.ui.cursor_pos.map(|p| (p.x, p.y));
            let crosshair_style = state.ui.crosshair_style;

            panel_quads.push(Quad::from_rect(layout.top_menu_bar, APP_BG));
            panel_quads.push(Quad::from_rect(layout.left_sidebar, APP_BG));
            panel_quads.push(Quad::from_rect(layout.right_sidebar, APP_BG));
            panel_quads.push(Quad::from_rect(layout.bottom_strip, APP_BG));
            panel_quads.push(Quad::from_rect(layout.status_bar, PANEL_BG));
            viewport_underlay_quads.push(Quad::from_rect(layout.viewport, VIEWPORT_BG));

            consumers.note(Stream::Panel, Consumer::Main, 0, panel_quads.len());
            consumers.note(
                Stream::Underlay,
                Consumer::Main,
                0,
                viewport_underlay_quads.len(),
            );
            let before_panel_quads = panel_quads.len();
            let before_text_runs = text_runs.len();
            let pane_consumers = render_phase1_shell_chrome(
                state,
                &layout,
                &mut crate::global_preferences_primitives::ControlPainter::new(
                    &mut panel_quads,
                    controls,
                    scale,
                ),
                &mut text_runs,
            )?;
            consumers.note(
                Stream::Panel,
                Consumer::Main,
                before_panel_quads,
                panel_quads.len(),
            );
            consumers.note(
                Stream::Text,
                Consumer::Main,
                before_text_runs,
                text_runs.len(),
            );
            let before_panel_quads = panel_quads.len();
            let before_text_runs = text_runs.len();
            consumers.include(Stream::Panel, pane_consumers);
            consumers.include(Stream::Text, pane_consumers);
            let before_menu_overlay_quads = menu_overlay_quads.len();
            let before_menu_overlay_text_runs = menu_overlay_text_runs.len();
            let menu_hit_start = hit_regions.len();
            menu_chrome::render_menu_bar(
                state,
                &layout,
                &mut panel_quads,
                &mut menu_overlay_quads,
                &mut menu_overlay_text_runs,
                &mut text_runs,
                &mut hit_regions,
            )?;
            let menu_hit_count = hit_regions.len() - menu_hit_start;
            consumers.note(
                Stream::Panel,
                Consumer::Menu,
                before_panel_quads,
                panel_quads.len(),
            );
            consumers.note(
                Stream::Text,
                Consumer::Menu,
                before_text_runs,
                text_runs.len(),
            );
            consumers.note(
                Stream::Overlay,
                Consumer::Menu,
                before_menu_overlay_quads,
                menu_overlay_quads.len(),
            );
            consumers.note(
                Stream::OverlayText,
                Consumer::Menu,
                before_menu_overlay_text_runs,
                menu_overlay_text_runs.len(),
            );
            let before_panel_quads = panel_quads.len();
            let before_text_runs = text_runs.len();
            side_panels::render_side_panels(
                state,
                &layout,
                &mut panel_quads,
                &mut text_runs,
                &mut hit_regions,
            )?;
            consumers.note(
                Stream::Panel,
                Consumer::Navigator,
                before_panel_quads,
                panel_quads.len(),
            );
            consumers.note(
                Stream::Panel,
                Consumer::Layers,
                before_panel_quads,
                panel_quads.len(),
            );
            consumers.note(
                Stream::Panel,
                Consumer::Inspector,
                before_panel_quads,
                panel_quads.len(),
            );
            consumers.note(
                Stream::Text,
                Consumer::Navigator,
                before_text_runs,
                text_runs.len(),
            );
            consumers.note(
                Stream::Text,
                Consumer::Layers,
                before_text_runs,
                text_runs.len(),
            );
            consumers.note(
                Stream::Text,
                Consumer::Inspector,
                before_text_runs,
                text_runs.len(),
            );
            let before_panel_quads = panel_quads.len();
            let before_text_runs = text_runs.len();
            bottom_dock::render_bottom_tabs(
                state,
                (!terminal_panes.is_empty()).then_some(bottom_dock::TerminalRenderInput {
                    panes: terminal_panes,
                    cache: terminal_cache,
                }),
                &layout,
                &mut panel_quads,
                &mut text_runs,
                &mut hit_regions,
            );
            consumers.note(
                Stream::Panel,
                Consumer::Terminal,
                before_panel_quads,
                panel_quads.len(),
            );
            consumers.note(
                Stream::Text,
                Consumer::Terminal,
                before_text_runs,
                text_runs.len(),
            );
            let before_terminal_graphics = terminal_graphics.len();
            terminal_scene::prepare_graphics(
                state,
                &layout,
                terminal_panes,
                &mut terminal_graphics,
            );
            consumers.note(
                Stream::TerminalGraphics,
                Consumer::TerminalOverlay,
                before_terminal_graphics,
                terminal_graphics.len(),
            );
            let before_menu_overlay_quads = menu_overlay_quads.len();
            let before_menu_overlay_text_runs = menu_overlay_text_runs.len();
            terminal_clipboard_menu::render_terminal_clipboard_menu(
                state,
                &layout,
                &mut menu_overlay_quads,
                &mut menu_overlay_text_runs,
                &mut hit_regions,
            );
            consumers.note(
                Stream::Overlay,
                Consumer::TerminalOverlay,
                before_menu_overlay_quads,
                menu_overlay_quads.len(),
            );
            consumers.note(
                Stream::OverlayText,
                Consumer::TerminalOverlay,
                before_menu_overlay_text_runs,
                menu_overlay_text_runs.len(),
            );
            if board_scene_active {
                let before_underlay = viewport_underlay_quads.len();
                let before_overlay = viewport_overlay_quads.len();
                let before_text = text_runs.len();
                render_scene(
                    state,
                    scene_viewport,
                    camera,
                    &mut viewport_underlay_quads,
                    &mut viewport_overlay_quads,
                    &mut text_runs,
                    &mut hit_regions,
                );
                consumers.note(
                    Stream::Underlay,
                    Consumer::Board,
                    before_underlay,
                    viewport_underlay_quads.len(),
                );
                consumers.note(
                    Stream::ViewportOverlay,
                    Consumer::Board,
                    before_overlay,
                    viewport_overlay_quads.len(),
                );
                consumers.note(Stream::Text, Consumer::Board, before_text, text_runs.len());
                let board_field = inset_rect(scene_viewport, 10.0, 10.0, 10.0, 10.0);
                let board_projection = Projection::new(board_field, &state.scene.bounds, camera);
                interaction_overlay::push_pane_interaction(
                    &mut board_interaction_quads,
                    &board_projection,
                    scene_viewport,
                    board_hover_bounds,
                    crosshair_cursor_screen,
                    crosshair_style,
                );
            }
            for pane in layout.viewport_panes(&state.ui.layout).panes {
                if let datum_gui_protocol::PaneContent::Revision(revision_pane) = pane.content {
                    let before_overlay = viewport_overlay_quads.len();
                    let before_text = text_runs.len();
                    revision_workspace::render_pane(
                        state,
                        revision_pane,
                        pane.rect.scene,
                        &mut viewport_overlay_quads,
                        &mut text_runs,
                        &mut hit_regions,
                    );
                    consumers.note(
                        Stream::ViewportOverlay,
                        Consumer::Revision,
                        before_overlay,
                        viewport_overlay_quads.len(),
                    );
                    consumers.note(
                        Stream::Text,
                        Consumer::Revision,
                        before_text,
                        text_runs.len(),
                    );
                }
            }
            let before_console_text = text_runs.len();
            let (console_overlay_vertices, console_overlay_layout) =
                scene_console::prepare(state, &layout, scale, &mut text_runs, &mut hit_regions)?;
            consumers.note(
                Stream::Console,
                Consumer::Console,
                0,
                console_overlay_vertices.len(),
            );
            consumers.note(
                Stream::Text,
                Consumer::Console,
                before_console_text,
                text_runs.len(),
            );
            let before_panel_quads = panel_quads.len();
            let before_text_runs = text_runs.len();
            // Menus paint above ordinary workspace/Console content. Their hit
            // regions must win there too; later modal/marking surfaces stay above.
            hit_regions[menu_hit_start..].rotate_left(menu_hit_count);
            marking_menu::render_marking_menu(
                state,
                &layout,
                &mut panel_quads,
                &mut text_runs,
                &mut hit_regions,
            );
            consumers.note(
                Stream::Panel,
                Consumer::Menu,
                before_panel_quads,
                panel_quads.len(),
            );
            consumers.note(
                Stream::Text,
                Consumer::Menu,
                before_text_runs,
                text_runs.len(),
            );
            // Application-modal preferences must be appended after every workspace
            // hit region so reverse-order hit testing cannot reach the obscured UI.
            let before_menu_overlay_quads = menu_overlay_quads.len();
            let before_menu_overlay_text_runs = menu_overlay_text_runs.len();
            let modal_consumer =
                if state.ui.global_preferences.open && global_preferences_native_window {
                    Consumer::Global
                } else {
                    Consumer::New
                };
            global_preferences_dialog::render_global_preferences_dialog(
                state,
                &layout,
                global_preferences_native_window,
                controls,
                scale,
                &mut menu_overlay_quads,
                &mut menu_overlay_text_runs,
                &mut hit_regions,
            )?;
            consumers.note(
                Stream::Overlay,
                modal_consumer,
                before_menu_overlay_quads,
                menu_overlay_quads.len(),
            );
            consumers.note(
                Stream::OverlayText,
                modal_consumer,
                before_menu_overlay_text_runs,
                menu_overlay_text_runs.len(),
            );
            if (scale - 1.0).abs() > f32::EPSILON {
                scale_text_run_sizes(&mut text_runs, scale);
                scale_text_run_sizes(&mut menu_overlay_text_runs, scale);
            }
            let panel_vertices = quads_to_vertices(&panel_quads);
            let menu_overlay_vertices = quads_to_vertices(&menu_overlay_quads);
            let viewport_underlay_vertices = quads_to_vertices(&viewport_underlay_quads);
            let viewport_overlay_vertices = quads_to_vertices(&viewport_overlay_quads);
            let board_interaction_vertices = quads_to_vertices(&board_interaction_quads);
            // P2.2a: describe the companion schematic pass. It is active only when the
            // layout has a Schematic pane AND the workspace carries a projected
            // schematic scene. The camera seeded here is fit-to-schematic-bounds — the
            // INITIAL framing; P2.2d makes the focused schematic pane interactive by
            // overriding this via `set_schematic_camera` with the pane's warm camera
            // (the gui-app render/capture path). Left as fit, this is byte-identical to
            // the pre-P2.2d static default (goldens/tests take this path unchanged).
            let (schematic_scene_viewport, schematic_bounds, schematic_camera) =
                match state.schematic_scene.as_ref() {
                    Some(schematic_scene) => (
                        layout.schematic_scene_viewport(&state.ui.layout),
                        schematic_scene.bounds.clone(),
                        CameraState::fit_to_bounds(&schematic_scene.bounds),
                    ),
                    None => {
                        // Inert placeholder: with no schematic viewport the second pass
                        // is gated off in gpu.rs, so these values are never consumed.
                        let inert = datum_gui_protocol::SceneBounds {
                            min_x: 0,
                            min_y: 0,
                            max_x: 1,
                            max_y: 1,
                        };
                        let camera = CameraState::fit_to_bounds(&inert);
                        (None, inert, camera)
                    }
                };
            let visible_draw_commands = if board_scene_active {
                retained_scene.visible_draw_commands(state)
            } else {
                Vec::new()
            };

            // S4: the schematic grid + interaction overlays share ONE immediate
            // screen-space underlay buffer (spec §1.2 / S1b), rebuilt against the pane's
            // warm camera in `set_schematic_camera` so grid weight + crosshair track it.
            let schematic_underlay_vertices = interaction_overlay::build_schematic_grid_vertices(
                schematic_scene_viewport,
                &schematic_bounds,
                schematic_camera,
            );
            let schematic_overlay_vertices =
                interaction_overlay::build_schematic_interaction_vertices(
                    schematic_scene_viewport,
                    &schematic_bounds,
                    schematic_camera,
                    schematic_hover_bounds,
                    crosshair_cursor_screen,
                    crosshair_style,
                );
            let surface_passes =
                coordinate_hit::build_surface_passes(&layout, state, camera, schematic_camera);

            consumers.note(
                Stream::BoardInteraction,
                Consumer::Board,
                0,
                board_interaction_vertices.len(),
            );
            consumers.note(
                Stream::BoardWorld,
                Consumer::Board,
                0,
                visible_draw_commands.len(),
            );
            consumers.note(
                Stream::SchematicUnderlay,
                Consumer::Schematic,
                0,
                schematic_underlay_vertices.len(),
            );
            consumers.note(
                Stream::SchematicOverlay,
                Consumer::Schematic,
                0,
                schematic_overlay_vertices.len(),
            );
            consumers.note(
                Stream::SchematicWorld,
                Consumer::Schematic,
                0,
                usize::from(schematic_scene_viewport.is_some()),
            );
            consumers.note(
                Stream::Grid,
                Consumer::Board,
                0,
                usize::from(board_scene_active),
            );
            consumers.note(
                Stream::Grid,
                Consumer::Schematic,
                0,
                usize::from(schematic_scene_viewport.is_some()),
            );
            Self {
                consumers,
                layout,
                hit_regions,
                scene_viewport,
                surface_passes,
                board_pane_id,
                scene_bounds: state.scene.bounds.clone(),
                camera,
                panel_vertices,
                menu_overlay_vertices,
                menu_overlay_text_runs,
                viewport_underlay_vertices,
                viewport_overlay_vertices,
                board_interaction_vertices,
                console_overlay_vertices,
                console_overlay_layout,
                visible_draw_commands,
                text_runs,
                terminal_graphics,
                schematic_scene_viewport,
                schematic_pane_id,
                schematic_bounds,
                schematic_camera,
                schematic_hover_bounds_nm: schematic_hover_bounds,
                crosshair_cursor_screen,
                crosshair_style,
                schematic_underlay_vertices,
                schematic_overlay_vertices,
            }
        })
    }
}

impl Renderer {
    #[allow(clippy::too_many_arguments)]
    pub fn prepare_workspace_with_terminal_renderer(
        &mut self,
        state: &ReviewWorkspaceState,
        width: u32,
        height: u32,
        scale_factor: f32,
        camera: CameraState,
        retained_scene: &RetainedScene,
        terminal_panes: &[crate::TerminalPaneRenderState],
        terminal_cache: Option<&mut crate::TerminalRenderCache>,
        native_dialog: bool,
    ) -> anyhow::Result<PreparedScene> {
        crate::text_metrics::measurement_owner::with_owner(&mut self.measurement_fonts, || {
            Ok({
                PreparedScene::from_workspace_with_controls(
                    state,
                    width,
                    height,
                    scale_factor,
                    camera,
                    retained_scene,
                    terminal_panes,
                    terminal_cache,
                    native_dialog,
                    &mut self.control_meshes,
                )?
            })
        })
    }
}
