//! Runtime view operations (decomposition of the gui-app monolith, decision
//! 021 / source-size governance): camera fit/pan/zoom, workspace-pane focus/
//! split/close/zoom/preset, divider-drag resize, layout/hit-test helpers, and
//! the dock resize drag. Split out of `main.rs`'s `impl Runtime`; behavior
//! unchanged. A child module of the crate root, so it sees `Runtime`'s private
//! fields/methods via `use super::*` exactly as the inline impl did.

use super::*;
use datum_gui_render::RectPx;

/// A small `(slot, bounds, viewport)` handle for the focused pane's camera routing (S2,
/// UVT-001/004): `slot` is `None` for the active board camera or `Some(leaf)` for a warm
/// `PaneCameras` entry; `bounds`/`viewport` are the scene and screen rect it hit-tests.
type FocusedViewport = (Option<datum_gui_protocol::PaneId>, SceneBounds, RectPx);

impl Runtime {
    /// Resolve the FOCUSED pane to its `(slot, bounds, viewport)` — one path for board,
    /// schematic, and any future pane, collapsing the old per-surface fork. `None` = a
    /// focused schematic with no resolved scene; handlers then bail (pan/zoom) or fit board.
    fn focused_viewport(&self) -> Option<FocusedViewport> {
        if self.focused_camera_is_schematic() {
            return Some((
                Some(self.schematic_leaf_id()?),
                self.schematic_bounds()?,
                self.schematic_scene_viewport()?,
            ));
        }
        Some((None, self.workspace().scene.bounds.clone(), self.scene_viewport()))
    }

    /// A mutable handle to the camera behind `slot` — the active board camera
    /// (`None`) or a warm per-leaf camera created via `init` on first touch.
    fn camera_slot_mut(
        &mut self,
        slot: Option<datum_gui_protocol::PaneId>,
        init: impl FnOnce() -> CameraState,
    ) -> &mut CameraState {
        match slot {
            None => &mut self.camera,
            Some(leaf) => self.pane_cameras.entry_or_insert_with(leaf, init),
        }
    }

    pub(super) fn fit_camera(&mut self) {
        // S2: fit the FOCUSED pane's camera to its bounds via the one resolver; a focused
        // schematic with no resolved scene falls back to the board (pre-collapse behavior).
        let (slot, bounds) = match self.focused_viewport() {
            Some((slot, bounds, _)) => (slot, bounds),
            None => (None, self.workspace().scene.bounds.clone()),
        };
        let fit = CameraState::fit_to_bounds(&bounds);
        *self.camera_slot_mut(slot, || fit) = fit;
        self.invalidate_frame();
    }

    pub(super) fn fit_review_target(&mut self) -> bool {
        let Some(bounds) = self.active_review_bounds() else {
            return false;
        };
        self.camera = CameraState::fit_to_bounds(&bounds);
        self.invalidate_frame();
        true
    }

    pub(super) fn fit_scene_object(&mut self, object_id: &str) -> bool {
        let Some(bounds) = self.scene_object_bounds(object_id) else {
            return false;
        };
        self.camera = CameraState::fit_to_bounds(&bounds);
        self.invalidate_frame();
        true
    }

    pub(super) fn scene_object_bounds(&self, object_id: &str) -> Option<SceneBounds> {
        let scene = &self.workspace().scene;
        if let Some(component) = scene
            .components
            .iter()
            .find(|item| item.object_id == object_id)
        {
            return Some(padded_rect_bounds(component.bounds, 1_500_000));
        }
        if let Some(pad) = scene.pads.iter().find(|item| item.object_id == object_id) {
            return Some(padded_rect_bounds(pad.bounds, 500_000));
        }
        if let Some(track) = scene.tracks.iter().find(|item| item.object_id == object_id) {
            return bounds_from_points(track.path.iter().copied(), 750_000);
        }
        if let Some(via) = scene.vias.iter().find(|item| item.object_id == object_id) {
            let radius = (via.diameter_nm / 2).max(250_000);
            return bounds_from_points([via.position], radius + 500_000);
        }
        if let Some(zone) = scene.zones.iter().find(|item| item.object_id == object_id) {
            return bounds_from_points(zone.polygon.iter().copied(), 750_000);
        }
        if let Some(text) = scene
            .board_texts
            .iter()
            .find(|item| item.object_id == object_id)
        {
            return bounds_from_points([text.position], text.height_nm.max(500_000));
        }
        if let Some(graphic) = scene
            .board_graphics
            .iter()
            .find(|item| item.object_id == object_id)
        {
            return bounds_from_points(graphic.path.iter().copied(), 750_000);
        }
        scene
            .outline
            .iter()
            .find(|item| item.object_id == object_id)
            .and_then(|outline| bounds_from_points(outline.path.iter().copied(), 750_000))
    }

    pub(super) fn active_review_bounds(&self) -> Option<SceneBounds> {
        let action_id = &self.workspace().active_review_target_id;
        let mut points = Vec::<PointNm>::new();

        for overlay in self
            .workspace()
            .scene
            .proposal_overlay_primitives
            .iter()
            .filter(|overlay| &overlay.proposal_action_id == action_id)
        {
            points.extend(overlay.path.iter().copied());
        }

        if let Some(evidence_key) = self
            .workspace()
            .selected_review_action()
            .map(|action| format!("segment:{}", action.selected_path_segment_index))
        {
            for review in self
                .workspace()
                .scene
                .review_primitives
                .iter()
                .filter(|review| review.evidence_key.as_deref() == Some(evidence_key.as_str()))
            {
                points.extend(review.path.iter().copied());
            }
        }

        let action = self.workspace().selected_review_action()?;
        for pad in self.workspace().scene.pads.iter().filter(|pad| {
            pad.pad_uuid == action.from_anchor_pad_uuid || pad.pad_uuid == action.to_anchor_pad_uuid
        }) {
            points.push(pad.center);
        }

        if points.is_empty() {
            return None;
        }

        let (min_x, max_x) = points
            .iter()
            .map(|point| point.x)
            .fold((i64::MAX, i64::MIN), |(min_x, max_x), x| {
                (min_x.min(x), max_x.max(x))
            });
        let (min_y, max_y) = points
            .iter()
            .map(|point| point.y)
            .fold((i64::MAX, i64::MIN), |(min_y, max_y), y| {
                (min_y.min(y), max_y.max(y))
            });
        let padding_nm = 1_500_000_i64;
        Some(SceneBounds {
            min_x: min_x.saturating_sub(padding_nm),
            min_y: min_y.saturating_sub(padding_nm),
            max_x: max_x.saturating_add(padding_nm),
            max_y: max_y.saturating_add(padding_nm),
        })
    }

    pub(super) fn handle_pan_drag(&mut self, next_cursor_pos: (f32, f32)) -> bool {
        let Some(previous) = self.last_cursor_pos else {
            return false;
        };
        let dx = next_cursor_pos.0 - previous.0;
        let dy = next_cursor_pos.1 - previous.1;
        // S2: pan the FOCUSED pane's camera via the resolver; no viewport -> bail (as before).
        let Some((slot, bounds, viewport)) = self.focused_viewport() else {
            return false;
        };
        // Deliberate board-specific branch: the artifact-preview drag is a
        // board-scene overlay affordance; if it consumes the drag, pan is skipped.
        if slot.is_none() {
            let prepared = self.prepared_scene().clone();
            if self.handle_artifact_preview_pan_drag(&prepared, previous, next_cursor_pos) {
                return true;
            }
        }
        self.camera_slot_mut(slot, || CameraState::fit_to_bounds(&bounds))
            .pan_pixels(viewport, &bounds, dx, dy);
        self.invalidate_frame();
        true
    }

    pub(super) fn handle_zoom(&mut self, delta: f32) -> bool {
        let Some((x, y)) = self.last_cursor_pos else {
            return false;
        };
        // S2: hit-test the zoom against the FOCUSED pane's scene rect and drive its
        // camera via the one resolver; outside the rect (or no viewport) is a no-op.
        let Some((slot, bounds, viewport)) = self.focused_viewport() else {
            return false;
        };
        if !viewport.contains(x, y) {
            return false;
        }
        self.camera_slot_mut(slot, || CameraState::fit_to_bounds(&bounds))
            .zoom_about_screen_point(viewport, &bounds, x, y, delta);
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
    pub(super) fn swap_pane_focus(&mut self, f: impl FnOnce(&mut datum_gui_protocol::WorkspaceLayout)) {
        let outgoing_scene = self.scene_leaf_id();
        f(&mut self.session.workspace_mut().ui.layout);
        let incoming_scene = self.scene_leaf_id();
        if let (Some(outgoing), Some(incoming)) = (outgoing_scene, incoming_scene)
            && outgoing != incoming
        {
            let bounds = self.workspace().scene.bounds.clone();
            self.camera = self.pane_cameras.focus_to(outgoing, self.camera, incoming, || {
                CameraState::fit_to_bounds(&bounds)
            });
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
                self.pane_cameras.inherit(id, inherited);
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
            self.camera = self.pane_cameras.focus_to(outgoing, self.camera, incoming, || {
                CameraState::fit_to_bounds(&bounds)
            });
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
        self.pane_cameras.reset(focused, self.camera);
        self.invalidate_frame();
    }

    pub(super) fn pane_set_focused_content(&mut self, content: datum_gui_protocol::PaneContent) {
        self.session
            .workspace_mut()
            .ui
            .layout
            .set_focused_content(content);
        self.invalidate_frame();
    }

    /// The resize-cursor orientation for screen point `(x, y)`: during an active
    /// divider drag it is the dragged split's orientation; otherwise the
    /// orientation of the divider gutter under the cursor, if any. `None` means the
    /// default cursor. Backs the hover affordance that signals a gutter is
    /// draggable (a vertical split reads east-west, a horizontal split north-south).
    pub(super) fn divider_resize_cursor(&self, x: f32, y: f32) -> Option<datum_gui_protocol::SplitOrientation> {
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

    pub(super) fn scene_viewport(&self) -> datum_gui_render::RectPx {
        self.current_layout()
            .scene_viewport(&self.workspace().ui.layout)
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

    /// The schematic leaf's scene canvas rect, if a Schematic pane exists — the
    /// viewport the schematic camera projects into and the rect the schematic
    /// zoom/pan hit-tests against (P2.2d).
    pub(super) fn schematic_scene_viewport(&self) -> Option<datum_gui_render::RectPx> {
        self.current_layout()
            .schematic_scene_viewport(&self.workspace().ui.layout)
    }

    /// True iff the focused pane is the companion schematic leaf — the P2.2d input
    /// routing decision: pan/zoom/fit drive the schematic's warm camera when true,
    /// otherwise the board scene camera. Focus (not scene-leaf binding) selects the
    /// interactive camera, so the owner can zoom whichever pane they focus.
    pub(super) fn focused_camera_is_schematic(&self) -> bool {
        focused_is_schematic(self.workspace().ui.layout.focused, self.schematic_leaf_id())
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
                .camera(leaf)
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

    pub(super) fn update_hover(&mut self, pos: (f32, f32)) -> bool {
        let prepared = self.prepared_scene();
        let new_hover = match prepared.hit_test(pos.0, pos.1) {
            Some(HitTarget::AuthoredObject(id)) => Some(id.clone()),
            Some(HitTarget::ReviewAction(id)) => Some(id.clone()),
            _ => None,
        };
        let current = &self.session.workspace().ui.hovered_object_id;
        if &new_hover != current {
            self.session.workspace_mut().ui.hovered_object_id = new_hover;
            self.invalidate_scene();
            return true;
        }
        false
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

/// P2.2d input-routing decision, factored pure so the routing is unit-tested
/// without a live `Runtime` (which owns the GPU/PTY and can't be built headless):
/// pan/zoom/fit drive the schematic pane's warm camera iff the focused leaf IS the
/// schematic leaf, otherwise the board scene camera. `Runtime::focused_camera_is_schematic`
/// is the sole caller; the input handlers branch on it.
fn focused_is_schematic(
    focused: datum_gui_protocol::PaneId,
    schematic_leaf: Option<datum_gui_protocol::PaneId>,
) -> bool {
    schematic_leaf == Some(focused)
}

#[cfg(test)]
mod focused_camera_routing_tests {
    use super::focused_is_schematic;
    use crate::pane_cameras::PaneCameras;
    use datum_gui_protocol::{PaneId, SceneBounds};
    use datum_gui_render::{CameraState, RectPx};

    fn bounds() -> SceneBounds {
        SceneBounds {
            min_x: 0,
            min_y: 0,
            max_x: 2_000_000,
            max_y: 1_000_000,
        }
    }

    fn viewport() -> RectPx {
        RectPx {
            x: 0.0,
            y: 0.0,
            width: 800.0,
            height: 600.0,
        }
    }

    #[test]
    fn routing_decision_follows_focus() {
        // The focused schematic leaf routes input to the schematic camera; any
        // other focus (a board leaf, or no schematic pane at all) routes to board.
        assert!(focused_is_schematic(PaneId(1), Some(PaneId(1))));
        assert!(!focused_is_schematic(PaneId(0), Some(PaneId(1))));
        assert!(!focused_is_schematic(PaneId(0), None));
    }

    /// The crux of P2.2d: a runtime pan/zoom can't be screenshotted, so lock the
    /// routing here. This mirrors the two `handle_*` branches EXACTLY — the board
    /// camera is the `Runtime`'s active `camera`; the schematic camera lives warm in
    /// `PaneCameras` keyed by the schematic leaf. With the schematic focused the
    /// gesture must mutate ONLY the schematic warm camera; with a board leaf focused
    /// ONLY the board camera. `focused_is_schematic` (tested above) selects the branch.
    #[test]
    fn zoom_updates_only_the_focused_panes_camera() {
        let schematic_leaf = PaneId(1);
        let mut board = CameraState::fit_to_bounds(&bounds());
        // Schematic camera seeded warm at its own initial fit, exactly as the render
        // path and the first interaction do.
        let mut cameras = PaneCameras::new(PaneId(0), board);
        cameras.entry_or_insert_with(schematic_leaf, || CameraState::fit_to_bounds(&bounds()));
        let board_before = board;
        let schematic_before = cameras.camera(schematic_leaf).unwrap();

        // --- Schematic focused: zoom drives the schematic warm camera only. ---
        assert!(focused_is_schematic(schematic_leaf, Some(schematic_leaf)));
        cameras
            .entry_or_insert_with(schematic_leaf, || CameraState::fit_to_bounds(&bounds()))
            .zoom_about_screen_point(viewport(), &bounds(), 400.0, 300.0, 1.5);
        let schematic_after = cameras.camera(schematic_leaf).unwrap();
        assert_ne!(
            schematic_after.zoom, schematic_before.zoom,
            "schematic zoom must change when the schematic pane is focused"
        );
        assert_eq!(
            board, board_before,
            "board camera must NOT move while the schematic pane is focused"
        );

        // --- Board focused: zoom drives the board camera only. ---
        assert!(!focused_is_schematic(PaneId(0), Some(schematic_leaf)));
        let schematic_held = cameras.camera(schematic_leaf).unwrap();
        board.zoom_about_screen_point(viewport(), &bounds(), 400.0, 300.0, 1.5);
        assert_ne!(
            board.zoom, board_before.zoom,
            "board zoom must change when a board leaf is focused"
        );
        assert_eq!(
            cameras.camera(schematic_leaf).unwrap(),
            schematic_held,
            "schematic camera must NOT move while a board leaf is focused"
        );
    }

    /// Pan mirror of the crux: a focused-schematic pan moves the schematic warm
    /// camera's center and leaves the board camera untouched, and vice versa.
    #[test]
    fn pan_updates_only_the_focused_panes_camera() {
        let schematic_leaf = PaneId(1);
        let mut board = CameraState::fit_to_bounds(&bounds());
        let mut cameras = PaneCameras::new(PaneId(0), board);
        cameras.entry_or_insert_with(schematic_leaf, || CameraState::fit_to_bounds(&bounds()));
        let board_before = board;
        let schematic_before = cameras.camera(schematic_leaf).unwrap();

        cameras
            .entry_or_insert_with(schematic_leaf, || CameraState::fit_to_bounds(&bounds()))
            .pan_pixels(viewport(), &bounds(), 40.0, 25.0);
        let schematic_after = cameras.camera(schematic_leaf).unwrap();
        assert_ne!(
            (schematic_after.center_x_nm, schematic_after.center_y_nm),
            (schematic_before.center_x_nm, schematic_before.center_y_nm),
            "schematic center must move on a focused-schematic pan"
        );
        assert_eq!(
            board, board_before,
            "board camera must NOT move on a focused-schematic pan"
        );

        let schematic_held = cameras.camera(schematic_leaf).unwrap();
        board.pan_pixels(viewport(), &bounds(), 40.0, 25.0);
        assert_ne!(
            (board.center_x_nm, board.center_y_nm),
            (board_before.center_x_nm, board_before.center_y_nm),
            "board center must move on a focused-board pan"
        );
        assert_eq!(
            cameras.camera(schematic_leaf).unwrap(),
            schematic_held,
            "schematic camera must NOT move on a focused-board pan"
        );
    }
}
