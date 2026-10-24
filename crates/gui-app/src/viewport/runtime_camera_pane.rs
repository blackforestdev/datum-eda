//! Shared resolved-pane camera fit/pan/zoom, divider resize and hit coordinates.
//! Pane creation, closure and editor fills belong to runtime_pane_lifecycle.

use super::*;
use datum_gui_render::RectPx;
use datum_gui_viewport::{CameraConfig, CameraEngine};

#[path = "runtime_camera_pane/zoom.rs"]
mod zoom;

#[derive(Clone)]
struct CameraViewport {
    pane: datum_gui_protocol::PaneId,
    content: datum_gui_protocol::PaneContent,
    bounds: SceneBounds,
    viewport: RectPx,
    active_board: bool,
    camera: CameraConfig,
}

impl Runtime {
    /// Resolve a selected pane against the caller's layout — one path for board,
    /// schematic, and future panes. An unresolved scene refuses camera input.
    fn camera_viewport_for_pane(
        &self,
        panes: &datum_gui_render::ViewportPanes,
        pane: datum_gui_protocol::PaneId,
    ) -> Option<CameraViewport> {
        let leaf = panes.panes.iter().find(|leaf| leaf.id == pane)?;
        let bounds = datum_gui_protocol::camera_scene_for_pane(self.workspace(), pane)?
            .bounds
            .clone();
        Some(CameraViewport {
            pane,
            content: leaf.content,
            bounds,
            viewport: leaf.rect.scene,
            active_board: leaf.content == datum_gui_protocol::PaneContent::Board
                && panes.scene_leaf_id() == Some(pane),
            camera: datum_gui_viewport::ViewportProfile::default().camera,
        })
    }

    pub(super) fn pane_camera_snapshot(
        &self,
        pane: datum_gui_protocol::PaneId,
    ) -> Option<CameraState> {
        let panes = self
            .current_layout()
            .viewport_panes(&self.workspace().ui.layout);
        let route = self.camera_viewport_for_pane(&panes, pane)?;
        Some(if route.active_board {
            self.camera
        } else {
            self.pane_cameras
                .camera(pane, route.content)
                .unwrap_or_else(|| CameraState::fit_to_bounds(&route.bounds))
        })
    }

    pub(super) fn initialize_focused_camera(&mut self) {
        if let Some(route) = self.focused_viewport() {
            let fit = CameraState::fit_to_bounds(&route.bounds);
            if !route.active_board {
                let _ = self.camera_slot_mut(&route, || fit);
            }
        }
    }

    fn focused_viewport(&self) -> Option<CameraViewport> {
        let panes = self
            .current_layout()
            .viewport_panes(&self.workspace().ui.layout);
        self.camera_viewport_for_pane(&panes, self.workspace().ui.layout.focused)
    }

    fn pointer_viewport(&self, pos: (f32, f32)) -> Option<CameraViewport> {
        // Locate and resolve against one layout: rebuilding it here duplicates
        // pane/divider allocations on every pointer event, even clamped zoom.
        let panes = self
            .current_layout()
            .viewport_panes(&self.workspace().ui.layout);
        self.camera_viewport_for_pane(&panes, panes.leaf_at(pos.0, pos.1)?)
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
        // Fit only the focused resolved pane. An unavailable pane never falls
        // back to a different Board and must not be narrated as a successful Fit.
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
        if route.content == datum_gui_protocol::PaneContent::Board
            && self.handle_artifact_preview_pan_drag(previous, next_cursor_pos)
        {
            return true;
        }
        let fit = CameraState::fit_to_bounds(&route.bounds);
        CameraEngine::pan_pixels(
            self.camera_slot_mut(&route, || fit),
            route.camera,
            route.viewport.into(),
            &route.bounds,
            dx,
            dy,
        );
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
        if !zoom::apply(
            self.camera_slot_mut(&route, || fit),
            route.camera,
            route.viewport.into(),
            &route.bounds,
            x,
            y,
            delta,
        ) {
            return false;
        }
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
        if !zoom::apply(
            self.camera_slot_mut(&route, || fit),
            route.camera,
            route.viewport.into(),
            &route.bounds,
            x,
            y,
            delta,
        ) {
            return false;
        }
        self.invalidate_frame();
        true
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
        self.shell_layout_for_dock(if self.workspace().ui.active_dock_tab.is_some() {
            Some(self.workspace().ui.effective_dock_height_px())
        } else {
            None
        })
    }

    pub(super) fn shell_layout_for_dock(&self, dock_height: Option<u32>) -> ShellLayout {
        ShellLayout::for_surface(
            self.config.width,
            self.config.height,
            self.scale_factor,
            dock_height,
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

    /// Whether the current pointer is inside an editor pane's drawable scene,
    /// excluding pane chrome, split gutters, side panels, and the dock.
    pub(super) fn cursor_in_editor_scene(&self) -> bool {
        let Some(pos) = self.last_cursor_pos else {
            return false;
        };
        self.pointer_viewport(pos)
            .is_some_and(|route| route.viewport.contains(pos.0, pos.1))
    }

    /// S3/UVT-004: resolve a primary click that `world_point_at_screen` reported on
    /// the SCHEMATIC surface. It hit-tests the schematic symbol regions in the
    /// schematic camera's space and acquires its source-qualified identity through
    /// the shared selection owner.
    pub(super) fn resolve_schematic_primary_click(
        &mut self,
        screen: (f32, f32),
        world_point: PointNm,
    ) -> bool {
        let qualified = match self.selection_world_hit(
            world_point,
            SceneSurface::Schematic,
            self.pane_at_screen(screen.0, screen.1),
        ) {
            Ok(hit) => hit,
            Err(()) => return true,
        };
        let hit = qualified.target;
        self.trace_click(format!(
            "primary click ({:.1}, {:.1}) schematic world ({}, {}) hit {hit:?} (native selection acquisition)",
            screen.0, screen.1, world_point.x, world_point.y
        ));
        if let Some(HitTarget::AuthoredObject(id)) = hit
            && let Some(basis) = qualified.basis.as_deref()
        {
            return self
                .acquire_native_key_from_basis(&id, Some(world_point), basis)
                .unwrap_or(false);
        }
        false
    }

    /// S4 / UVT-004 per-surface hover: resolve hover in whichever pane the cursor
    /// is over, in that pane's OWN camera/space (via the S3 `world_point_at_screen`
    /// keystone + the matching per-pane world hit-test), and store the hovered
    /// target together with its surface plus the live cursor. Board hover is
    /// unchanged; a schematic-pane cursor over a symbol now resolves that symbol's
    /// identity (impossible pre-S3, when this wrote a single board-only global) and
    /// the renderer projects the hover pre-highlight with the matching camera.
    pub(super) fn update_hover(&mut self, pos: (f32, f32)) -> bool {
        // Sidebars, pane headers, gutters, and dock chrome have no authored
        // world hover. Reject them through the cheap solved-layout path before
        // touching PreparedScene/retained geometry; Navigator pointer latency
        // must not inherit a cold world-scene build.
        if !self
            .pointer_viewport(pos)
            .is_some_and(|route| route.viewport.contains(pos.0, pos.1))
        {
            return self.clear_interaction_overlay();
        }
        // Shared preparation validates both retained sources before projection.
        if self.prepared_scene().is_none() || !self.ensure_retained_scene() {
            return self.clear_interaction_overlay();
        }
        let resolved = {
            let prepared = self
                .renderer
                .render_session()
                .prepared()
                .expect("prepared scene built above");
            let board_retained = self
                .renderer
                .render_session()
                .board()
                .expect("board retained scene built with prepared");
            datum_gui_render::resolve_pane_hover(
                prepared,
                board_retained,
                self.renderer.render_session().schematic(),
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
}

impl Runtime {
    pub(super) fn pan_selection_pane(
        &mut self,
        pane: datum_gui_protocol::PaneId,
        delta: (f32, f32),
    ) -> bool {
        let panes = self
            .current_layout()
            .viewport_panes(&self.workspace().ui.layout);
        let Some(route) = self.camera_viewport_for_pane(&panes, pane) else {
            return false;
        };
        let fit = CameraState::fit_to_bounds(&route.bounds);
        CameraEngine::pan_pixels(
            self.camera_slot_mut(&route, || fit),
            route.camera,
            route.viewport.into(),
            &route.bounds,
            delta.0,
            delta.1,
        );
        self.invalidate_frame();
        true
    }
}
