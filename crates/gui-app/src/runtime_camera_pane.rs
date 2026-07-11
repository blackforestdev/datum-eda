//! Runtime view operations (decomposition of the gui-app monolith, decision
//! 021 / source-size governance): camera fit/pan/zoom, workspace-pane focus/
//! split/close/zoom/preset, divider-drag resize, layout/hit-test helpers, and
//! the dock resize drag. Split out of `main.rs`'s `impl Runtime`; behavior
//! unchanged. A child module of the crate root, so it sees `Runtime`'s private
//! fields/methods via `use super::*` exactly as the inline impl did.

use super::*;
use datum_gui_render::RectPx;

#[derive(Clone)]
struct CameraViewport {
    pane: datum_gui_protocol::PaneId,
    content: datum_gui_protocol::PaneContent,
    bounds: SceneBounds,
    viewport: RectPx,
    active_board: bool,
}

impl Runtime {
    /// Resolve the FOCUSED pane to its `(slot, bounds, viewport)` — one path for board,
    /// schematic, and any future pane, collapsing the old per-surface fork. `None` = a
    /// focused schematic with no resolved scene; handlers then bail (pan/zoom) or fit board.
    fn camera_viewport_for_pane(&self, pane: datum_gui_protocol::PaneId) -> Option<CameraViewport> {
        let panes = self
            .current_layout()
            .viewport_panes(&self.workspace().ui.layout);
        let leaf = panes.panes.iter().find(|leaf| leaf.id == pane)?;
        let bounds = match leaf.content {
            datum_gui_protocol::PaneContent::Board => self.workspace().scene.bounds.clone(),
            datum_gui_protocol::PaneContent::Schematic => self.schematic_bounds()?,
        };
        Some(CameraViewport {
            pane,
            content: leaf.content,
            bounds,
            viewport: leaf.rect.scene,
            active_board: leaf.content == datum_gui_protocol::PaneContent::Board
                && panes.scene_leaf_id() == Some(pane),
        })
    }

    fn focused_viewport(&self) -> Option<CameraViewport> {
        self.camera_viewport_for_pane(self.workspace().ui.layout.focused)
    }

    fn pointer_viewport(&self, pos: (f32, f32)) -> Option<CameraViewport> {
        self.camera_viewport_for_pane(self.pane_at_screen(pos.0, pos.1)?)
    }

    /// A mutable handle to the camera behind `slot` — the active board camera
    /// (`None`) or a warm per-leaf camera created via `init` on first touch.
    fn camera_slot_mut(
        &mut self,
        route: &CameraViewport,
        init: impl FnOnce() -> CameraState,
    ) -> &mut CameraState {
        if route.active_board {
            &mut self.camera
        } else {
            self.pane_cameras
                .entry_or_insert_with(route.pane, route.content, init)
        }
    }

    pub(super) fn fit_camera(&mut self) {
        // S2: fit the FOCUSED pane's camera to its bounds via the one resolver; a focused
        // schematic with no resolved scene falls back to the board (pre-collapse behavior).
        let Some(route) = self.focused_viewport() else {
            return;
        };
        let fit = CameraState::fit_to_bounds(&route.bounds);
        *self.camera_slot_mut(&route, || fit) = fit;
        self.invalidate_frame();
    }

    pub(super) fn handle_pan_drag(
        &mut self,
        previous: (f32, f32),
        next_cursor_pos: (f32, f32),
    ) -> bool {
        let dx = next_cursor_pos.0 - previous.0;
        let dy = next_cursor_pos.1 - previous.1;
        let Some(route) = self.pointer_viewport(next_cursor_pos) else {
            return false;
        };
        // Crossing a pane boundary during a drag starts a fresh segment instead
        // of applying pixels measured in one pane to another pane's camera.
        if self.pointer_viewport(previous).map(|prior| prior.pane) != Some(route.pane) {
            return false;
        }
        // Deliberate board-specific branch: the artifact-preview drag is a
        // board-scene overlay affordance; if it consumes the drag, pan is skipped.
        if route.content == datum_gui_protocol::PaneContent::Board {
            let prepared = self.prepared_scene().clone();
            if self.handle_artifact_preview_pan_drag(&prepared, previous, next_cursor_pos) {
                return true;
            }
        }
        let fit = CameraState::fit_to_bounds(&route.bounds);
        self.camera_slot_mut(&route, || fit)
            .pan_pixels(route.viewport, &route.bounds, dx, dy);
        self.invalidate_frame();
        true
    }

    pub(super) fn handle_zoom(&mut self, delta: f32) -> bool {
        let Some((x, y)) = self.last_cursor_pos else {
            return false;
        };
        // Pointer gestures follow the containing pane; focus is intentionally not
        // consulted here. Menu/keyboard actions use `zoom_focused_view` below.
        let Some(route) = self.pointer_viewport((x, y)) else {
            return false;
        };
        if !route.viewport.contains(x, y) {
            return false;
        }
        let fit = CameraState::fit_to_bounds(&route.bounds);
        self.camera_slot_mut(&route, || fit)
            .zoom_about_screen_point(route.viewport, &route.bounds, x, y, delta);
        self.invalidate_frame();
        true
    }

    pub(super) fn zoom_focused_view(&mut self, delta: f32) -> bool {
        let Some(route) = self.focused_viewport() else {
            return false;
        };
        let x = route.viewport.x + route.viewport.width * 0.5;
        let y = route.viewport.y + route.viewport.height * 0.5;
        let fit = CameraState::fit_to_bounds(&route.bounds);
        self.camera_slot_mut(&route, || fit)
            .zoom_about_screen_point(route.viewport, &route.bounds, x, y, delta);
        self.invalidate_frame();
        true
    }

    // ---------------------------------------------------------------------
    // Workspace pane ops (decision 021, P2.1b). Every one is a PURE view-state
    // mutation of the pane tree: it swaps the active camera to the target leaf's
    // warm camera (never a refit) and calls `invalidate_frame`, which rebuilds
    // only the cheap prepared chrome/scene and KEEPS the resolved world scene —
    // so a pane op costs zero world-scene re-resolve (the "no noticeable lag"
    // gate). Only the focused leaf renders live (single-live-scene); non-focused
    // Board leaves render as today (idle real-content snapshot lands with P2.2
    // multi-scene). These are never journaled — they are workspace state.
    // ---------------------------------------------------------------------

    /// Shared focus-change core: mutate focus via `f`, then keep the active camera
    /// bound to the leaf that renders the live BOARD scene. The active camera
    /// belongs to the SCENE leaf, not whichever pane is focused: moving focus
    /// between the board and a schematic pane keeps the same scene leaf, so the
    /// board's framing (zoom/pan) PERSISTS instead of snapping to the schematic's
    /// fit camera (the "PCB zooms to fit / can be zoomed while another pane is
    /// selected" bug). The camera is only swapped when the scene leaf actually
    /// changes — e.g. focus moving to a *different* board pane in a multi-board
    /// split — at which point the outgoing board's framing is stashed warm and the
    /// incoming board's warm camera (fit only if never seen) is activated.
    pub(super) fn swap_pane_focus(
        &mut self,
        f: impl FnOnce(&mut datum_gui_protocol::WorkspaceLayout),
    ) {
        let outgoing_scene = self.scene_leaf_id();
        f(&mut self.session.workspace_mut().ui.layout);
        let incoming_scene = self.scene_leaf_id();
        if let (Some(outgoing), Some(incoming)) = (outgoing_scene, incoming_scene)
            && outgoing != incoming
        {
            let bounds = self.workspace().scene.bounds.clone();
            self.camera = self.pane_cameras.focus_to(
                outgoing,
                datum_gui_protocol::PaneContent::Board,
                self.camera,
                incoming,
                datum_gui_protocol::PaneContent::Board,
                || CameraState::fit_to_bounds(&bounds),
            );
        }
        self.invalidate_frame();
    }

    pub(super) fn pane_focus_next(&mut self) {
        self.swap_pane_focus(|layout| layout.focus_next());
    }

    pub(super) fn pane_focus_prev(&mut self) {
        self.swap_pane_focus(|layout| layout.focus_prev());
    }

    pub(super) fn pane_split_focused(&mut self, orientation: datum_gui_protocol::SplitOrientation) {
        // Focus is unchanged by a split; the fresh child inherits the focused
        // sibling's warm framing so it opens looking like the pane it split from.
        let before: std::collections::BTreeSet<_> =
            self.workspace().ui.layout.leaves().into_iter().collect();
        self.session
            .workspace_mut()
            .ui
            .layout
            .split_focused(orientation);
        let inherited = self.camera;
        for id in self.workspace().ui.layout.leaves() {
            if !before.contains(&id) {
                let content = self
                    .current_layout()
                    .viewport_panes(&self.workspace().ui.layout)
                    .panes
                    .iter()
                    .find(|pane| pane.id == id)
                    .map(|pane| pane.content)
                    .unwrap_or(datum_gui_protocol::PaneContent::Board);
                let initial = match content {
                    datum_gui_protocol::PaneContent::Board => inherited,
                    datum_gui_protocol::PaneContent::Schematic => self
                        .schematic_bounds()
                        .as_ref()
                        .map(CameraState::fit_to_bounds)
                        .unwrap_or(inherited),
                };
                self.pane_cameras.inherit(id, content, initial);
            }
        }
        self.invalidate_frame();
    }

    pub(super) fn pane_close_focused(&mut self) {
        let outgoing_scene = self.scene_leaf_id();
        self.session.workspace_mut().ui.layout.close_focused();
        let incoming_scene = self.scene_leaf_id();
        let live = self.workspace().ui.layout.leaves();
        self.pane_cameras.retain_live(&live);
        // Swap only when the board SCENE leaf changed (e.g. the board pane itself was
        // closed and another board took over); closing a schematic pane leaves the
        // board's framing untouched.
        if let (Some(outgoing), Some(incoming)) = (outgoing_scene, incoming_scene)
            && outgoing != incoming
        {
            let bounds = self.workspace().scene.bounds.clone();
            self.camera = self.pane_cameras.focus_to(
                outgoing,
                datum_gui_protocol::PaneContent::Board,
                self.camera,
                incoming,
                datum_gui_protocol::PaneContent::Board,
                || CameraState::fit_to_bounds(&bounds),
            );
        }
        self.invalidate_frame();
    }

    pub(super) fn pane_toggle_zoom(&mut self) {
        // Transient maximize of the focused leaf; focus and cameras are untouched.
        self.session.workspace_mut().ui.layout.toggle_zoom();
        self.invalidate_frame();
    }

    pub(super) fn pane_apply_preset(&mut self, preset: datum_gui_protocol::WorkspacePreset) {
        self.session.workspace_mut().ui.layout.apply_preset(preset);
        // A preset rebuilds the tree with fresh ids; reset the warm store to the
        // new focused leaf and fit it (a preset is a deliberate layout reset).
        let focused = self.workspace().ui.layout.focused;
        let bounds = self.workspace().scene.bounds.clone();
        self.camera = CameraState::fit_to_bounds(&bounds);
        let content = self.workspace().ui.layout.focused_content();
        self.pane_cameras.reset(focused, content, self.camera);
        self.invalidate_frame();
    }

    pub(super) fn pane_set_focused_content(&mut self, content: datum_gui_protocol::PaneContent) {
        let outgoing_board = self.scene_leaf_id();
        let outgoing_camera = self.camera;
        self.session
            .workspace_mut()
            .ui
            .layout
            .set_focused_content(content);
        let incoming_board = self.scene_leaf_id();
        if outgoing_board != incoming_board {
            let bounds = self.workspace().scene.bounds.clone();
            match (outgoing_board, incoming_board) {
                (Some(outgoing), Some(incoming)) => {
                    self.camera = self.pane_cameras.focus_to(
                        outgoing,
                        datum_gui_protocol::PaneContent::Board,
                        outgoing_camera,
                        incoming,
                        datum_gui_protocol::PaneContent::Board,
                        || CameraState::fit_to_bounds(&bounds),
                    );
                }
                (Some(outgoing), None) => {
                    self.pane_cameras.inherit(
                        outgoing,
                        datum_gui_protocol::PaneContent::Board,
                        outgoing_camera,
                    );
                }
                (None, Some(incoming)) => {
                    self.camera = *self.pane_cameras.entry_or_insert_with(
                        incoming,
                        datum_gui_protocol::PaneContent::Board,
                        || CameraState::fit_to_bounds(&bounds),
                    );
                }
                (None, None) => {}
            }
        }
        // Touch the retargeted pane with its new identity. If it previously held a
        // different surface, `PaneCameras` discards that incompatible camera.
        if let Some(route) = self.focused_viewport() {
            let fit = CameraState::fit_to_bounds(&route.bounds);
            if !route.active_board {
                let _ = self.camera_slot_mut(&route, || fit);
            }
        }
        self.invalidate_frame();
    }

    /// The resize-cursor orientation for screen point `(x, y)`: during an active
    /// divider drag it is the dragged split's orientation; otherwise the
    /// orientation of the divider gutter under the cursor, if any. `None` means the
    /// default cursor. Backs the hover affordance that signals a gutter is
    /// draggable (a vertical split reads east-west, a horizontal split north-south).
    pub(super) fn divider_resize_cursor(
        &self,
        x: f32,
        y: f32,
    ) -> Option<datum_gui_protocol::SplitOrientation> {
        if let Some(drag) = &self.divider_drag {
            return Some(drag.orientation);
        }
        self.current_layout()
            .viewport_panes(&self.workspace().ui.layout)
            .divider_at(x, y)
            .map(|d| d.orientation)
    }

    /// Divider-drag resize (decision 021): if screen point `(x, y)` grabs a split
    /// divider gutter, begin a drag on that split and return `true` (so the press
    /// is consumed instead of focusing a pane / running board interaction).
    pub(super) fn begin_divider_drag(&mut self, x: f32, y: f32) -> bool {
        let panes = self
            .current_layout()
            .viewport_panes(&self.workspace().ui.layout);
        if let Some(divider) = panes.divider_at(x, y) {
            self.divider_drag = Some(DividerDrag {
                path: divider.path.clone(),
                split_frame: divider.split_frame,
                orientation: divider.orientation,
            });
            true
        } else {
            false
        }
    }

    /// Apply a divider-drag move: translate the cursor into a new ratio for the
    /// dragged split and write it (the model clamps so panes never collapse).
    pub(super) fn handle_divider_drag(&mut self, pos: (f32, f32)) -> bool {
        let Some(drag) = &self.divider_drag else {
            return false;
        };
        let ratio = drag.ratio_at(pos.0, pos.1);
        let path = drag.path.clone();
        self.session
            .workspace_mut()
            .ui
            .layout
            .set_ratio_at_path(&path, ratio);
        self.invalidate_frame();
        true
    }

    pub(super) fn current_layout(&self) -> ShellLayout {
        ShellLayout::for_surface(
            self.config.width,
            self.config.height,
            self.scale_factor,
            if self.workspace().ui.active_dock_tab.is_some() {
                Some(self.workspace().ui.dock_height_px)
            } else {
                None
            },
        )
    }

    /// The leaf that renders the live board scene (the pane the active camera
    /// belongs to). Bound to the BOARD pane, not focus — so the board's framing
    /// persists while another pane is focused.
    pub(super) fn scene_leaf_id(&self) -> Option<datum_gui_protocol::PaneId> {
        self.current_layout()
            .viewport_panes(&self.workspace().ui.layout)
            .scene_leaf_id()
    }

    /// The companion schematic leaf's `PaneId`, if the layout has a Schematic pane.
    /// P2.2d keys the schematic's warm interactive camera by this id.
    pub(super) fn schematic_leaf_id(&self) -> Option<datum_gui_protocol::PaneId> {
        self.current_layout()
            .viewport_panes(&self.workspace().ui.layout)
            .schematic_leaf_id()
    }

    /// The companion schematic scene's world bounds, if a schematic scene exists —
    /// the reference the schematic camera fits to and pans/zooms within (P2.2d).
    pub(super) fn schematic_bounds(&self) -> Option<SceneBounds> {
        self.workspace()
            .schematic_scene
            .as_ref()
            .map(|scene| scene.bounds.clone())
    }

    /// The camera the companion schematic pass renders with (P2.2d). Once the owner
    /// has interacted with the focused schematic pane its WARM camera persists here;
    /// until then (store cold) the INITIAL fit-to-schematic-bounds — byte-identical
    /// to the pre-P2.2d static fit, so the default board-focused capture is
    /// unchanged. `None` when there is no companion schematic scene / Schematic pane.
    pub(super) fn schematic_camera_for_render(&self) -> Option<CameraState> {
        let leaf = self.schematic_leaf_id()?;
        let bounds = self.schematic_bounds()?;
        Some(
            self.pane_cameras
                .camera(leaf, datum_gui_protocol::PaneContent::Schematic)
                .unwrap_or_else(|| CameraState::fit_to_bounds(&bounds)),
        )
    }

    /// The workspace leaf pane whose frame contains screen point `(x, y)`, tiling
    /// the current shell exactly as the renderer does. Backs click-to-focus
    /// (decision 021): a press outside every pane (sidebars/dock/menu) returns
    /// `None` and the click falls through to normal board behavior.
    pub(super) fn pane_at_screen(&self, x: f32, y: f32) -> Option<datum_gui_protocol::PaneId> {
        self.current_layout()
            .viewport_panes(&self.workspace().ui.layout)
            .leaf_at(x, y)
    }

    /// Open the radial marking menu at the cursor (right-click). A board-scene
    /// affordance today; the schematic context menu is S7, so a schematic-pane or
    /// out-of-pane point returns false (S3 only makes the schematic point
    /// resolvable, via `world_point_at_screen` / `schematic_world_hit`).
    pub(super) fn open_marking_menu_at_cursor(&mut self) -> bool {
        let Some((x, y)) = self.last_cursor_pos else {
            return false;
        };
        if self.cursor_in_dock() {
            return false;
        }
        let world_point = {
            let prepared = self.prepared_scene();
            prepared.world_point_at_screen(x, y)
        };
        let Some((world_point, SceneSurface::Board)) = world_point else {
            return false;
        };
        let retained_target = {
            let retained = self.retained_scene.get_or_insert_with(|| {
                RetainedScene::from_workspace_for_surface(
                    self.session.workspace(),
                    self.config.width,
                    self.config.height,
                    self.scale_factor,
                )
            });
            retained
                .hit_test_authored_world(world_point, self.session.workspace())
                .cloned()
        };
        let target_object_id = match retained_target {
            Some(HitTarget::AuthoredObject(id)) | Some(HitTarget::ReviewAction(id)) => Some(id),
            _ => None,
        };
        let menu_key = marking_menu_key_for_target(target_object_id.as_deref());
        let ui = &mut self.session.workspace_mut().ui;
        ui.active_menu = None;
        ui.marking_menu = Some(MarkingMenuState {
            menu_key,
            target_object_id,
            anchor_x_px: x.round() as i32,
            anchor_y_px: y.round() as i32,
            preview_slot: None,
            gesture_dx_px: 0,
            gesture_dy_px: 0,
        });
        self.invalidate_frame();
        true
    }

    /// S3/UVT-004: resolve a primary click that `world_point_at_screen` reported on
    /// the SCHEMATIC surface. It hit-tests the schematic symbol regions in the
    /// schematic camera's space and traces the resolved identity — the coordinate +
    /// hit plumbing S5 (selection) and S7 (context menu) build on. Selection is
    /// deliberately NOT fired here, so it returns `false` (unhandled) and leaves the
    /// board path untouched.
    pub(super) fn resolve_schematic_primary_click(
        &mut self,
        screen: (f32, f32),
        world_point: PointNm,
    ) -> bool {
        let hit = self.schematic_world_hit(world_point);
        self.trace_click(format!(
            "primary click ({:.1}, {:.1}) schematic world ({}, {}) hit {hit:?} (selection deferred to S5)",
            screen.0, screen.1, world_point.x, world_point.y
        ));
        false
    }

    /// Hit-test a schematic-pane world point against the companion schematic scene's
    /// symbol hit regions (built lazily like the board's, and reused when the render
    /// path already resolved it). `None` when there is no schematic scene/pane.
    pub(super) fn schematic_world_hit(&mut self, world_point: PointNm) -> Option<HitTarget> {
        if self.schematic_retained_scene.is_none() {
            self.schematic_retained_scene = RetainedScene::from_workspace_schematic_for_surface(
                self.session.workspace(),
                self.config.width,
                self.config.height,
                self.scale_factor,
            );
        }
        self.schematic_retained_scene
            .as_ref()?
            .hit_test_world(world_point)
            .cloned()
    }

    /// S4 / UVT-004 per-surface hover: resolve hover in whichever pane the cursor
    /// is over, in that pane's OWN camera/space (via the S3 `world_point_at_screen`
    /// keystone + the matching per-pane world hit-test), and store the hovered
    /// target together with its surface plus the live cursor. Board hover is
    /// unchanged; a schematic-pane cursor over a symbol now resolves that symbol's
    /// identity (impossible pre-S3, when this wrote a single board-only global) and
    /// the renderer projects the hover pre-highlight with the matching camera.
    pub(super) fn update_hover(&mut self, pos: (f32, f32)) -> bool {
        // Ensure the caches exist: the prepared build also builds the board
        // retained scene; the companion schematic retained scene is built here.
        let _ = self.prepared_scene();
        if self.schematic_retained_scene.is_none() {
            self.schematic_retained_scene = RetainedScene::from_workspace_schematic_for_surface(
                self.session.workspace(),
                self.config.width,
                self.config.height,
                self.scale_factor,
            );
        }
        let resolved = {
            let prepared = self
                .prepared_scene
                .as_ref()
                .expect("prepared scene built above");
            let board_retained = self
                .retained_scene
                .as_ref()
                .expect("board retained scene built with prepared");
            datum_gui_render::resolve_pane_hover(
                prepared,
                board_retained,
                self.schematic_retained_scene.as_ref(),
                self.session.workspace(),
                pos.0,
                pos.1,
            )
        };
        // Preserve the resolver's typed pane ownership beside the opaque object id;
        // rendering must never infer a surface from identifier spelling. S4 cursor
        // crosshair (decision 023 UVT-005): record the live cursor in
        // device-pixel SCREEN space so the renderer can draw the crosshair through
        // it. These are transient overlay changes: authored board and schematic
        // retained geometry must remain warm while the pointer moves.
        let new_cursor = resolved.cursor;
        let new_hover = resolved.hover;
        let cursor_moved = self.session.workspace().ui.cursor_pos != new_cursor;
        let hover_changed = self.session.workspace().ui.hovered_object != new_hover;
        if cursor_moved {
            self.session.workspace_mut().ui.cursor_pos = new_cursor;
        }
        if hover_changed {
            self.session.workspace_mut().ui.hovered_object = new_hover;
        }
        let crosshair_live =
            self.session.workspace().ui.crosshair_style != datum_gui_protocol::CrosshairStyle::None;
        if hover_changed || (cursor_moved && crosshair_live) {
            self.refresh_interaction_overlay();
            return true;
        }
        false
    }

    /// Clear pointer-driven chrome when the cursor leaves an authoring surface or
    /// a modal/terminal interaction captures it. Keeping stale hover/crosshair
    /// state visible is misleading; clearing it still preserves retained geometry.
    pub(super) fn clear_interaction_overlay(&mut self) -> bool {
        let ui = &mut self.session.workspace_mut().ui;
        // Do not combine these `take`s with short-circuiting `||`: both fields must
        // be cleared even when the cursor field was populated.
        let had_cursor = ui.cursor_pos.take().is_some();
        let had_hover = ui.hovered_object.take().is_some();
        let changed = had_cursor || had_hover;
        if changed {
            self.refresh_interaction_overlay();
        }
        changed
    }

    pub(super) fn cursor_in_dock(&self) -> bool {
        let Some((_, y)) = self.last_cursor_pos else {
            return false;
        };
        let layout = self.current_layout();
        y >= layout.bottom_strip.y
    }

    pub(super) fn handle_dock_resize_drag(&mut self, next_cursor_pos: (f32, f32)) -> bool {
        let window_height = self.config.height as f32;
        let new_height_physical =
            (window_height - next_cursor_pos.1).clamp(32.0, window_height * 0.6);
        let new_height_logical = new_height_physical / self.scale_factor.max(0.01);
        let new_height_logical = new_height_logical as u32;
        if self.workspace().ui.dock_height_px == new_height_logical {
            return false;
        }
        self.session.workspace_mut().ui.dock_height_px = new_height_logical;
        self.resize_terminal_to_dock();
        self.invalidate_scene();
        true
    }

    pub(super) fn resize_terminal_to_dock(&mut self) {
        let bottom_height = self.current_layout().bottom_strip.height;
        let cols = ((self.config.width as f32 - 24.0) / 7.5).floor().max(20.0) as u16;
        let rows = ((bottom_height - 76.0) / 16.0).floor().max(4.0) as u16;
        append_gui_verbose_diagnostic_line(format!("terminal resize begin {cols}x{rows}"));
        match self.terminal_sessions.resize_active(cols, rows) {
            Ok(()) => {
                let terminal = &mut self.session.workspace_mut().ui.terminal;
                terminal.columns = cols;
                terminal.rows = rows;
                append_gui_verbose_diagnostic_line("terminal resize end");
            }
            Err(err) => {
                append_gui_diagnostic_line(format!("terminal resize failed: {err}"));
                self.push_terminal_line(format!("terminal resize failed: {err}"));
            }
        }
    }
}
