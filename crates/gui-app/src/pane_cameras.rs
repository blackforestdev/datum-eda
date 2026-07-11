//! Warm per-leaf view cameras for the workspace pane tree (P2.1b, decision 021).
//!
//! The owner's stated priority is speed and feel: "clicking an adjacent viewport
//! to make it live must have no noticeable lag." The mechanism is warmth — every
//! leaf pane keeps its OWN camera, so switching focus to another pane is instant.
//! Entries carry both pane and content identity: replacing Board with Schematic
//! can never reinterpret board coordinates as schematic coordinates. The live
//! board pass retains its historical active field; every other camera rests here.
//!
//! Only the focused leaf renders live (single-live-scene, as today): the warm
//! store holds cameras, not GPU textures. Idle real-content snapshot/blit for
//! non-focused panes lands with P2.2 multi-scene, when idle panes gain real
//! content to freeze; until then a non-focused Board leaf keeps only its warm
//! camera. This is consumer/workspace view state — never journaled.

use datum_gui_protocol::{PaneContent, PaneId};
use datum_gui_render::CameraState;
use std::collections::BTreeMap;

/// The warm per-leaf/content camera store. The live board leaf's camera also has
/// a `Runtime` active copy for the current renderer path; all other leaf cameras
/// live here. A content mismatch is treated as a cold entry and initialized fit.
pub(crate) struct PaneCameras {
    warm: BTreeMap<PaneId, (PaneContent, CameraState)>,
}

impl PaneCameras {
    /// Seed the store with the initially-focused leaf and its camera.
    pub(crate) fn new(focused: PaneId, content: PaneContent, active: CameraState) -> Self {
        let mut warm = BTreeMap::new();
        warm.insert(focused, (content, active));
        Self { warm }
    }

    /// Swap the active camera to the newly-focused leaf. Stash the `outgoing`
    /// leaf's live camera, then hand back the `incoming` leaf's WARM camera —
    /// created via `init` only if that leaf has never been focused. An existing
    /// camera is NEVER reset: that is the whole point of warmth (no refit lag on
    /// focus-switch). Returns the camera the caller should make active.
    pub(crate) fn focus_to(
        &mut self,
        outgoing: PaneId,
        outgoing_content: PaneContent,
        active_camera: CameraState,
        incoming: PaneId,
        incoming_content: PaneContent,
        init: impl FnOnce() -> CameraState,
    ) -> CameraState {
        let mut init = Some(init);
        self.warm
            .insert(outgoing, (outgoing_content, active_camera));
        let entry = self.warm.entry(incoming).or_insert_with(|| {
            (
                incoming_content,
                init.take().expect("initializer used once")(),
            )
        });
        if entry.0 != incoming_content {
            *entry = (
                incoming_content,
                init.take().expect("initializer used once")(),
            );
        }
        entry.1
    }

    /// Register a freshly-split leaf: it inherits the focused sibling's warm
    /// camera so the new pane opens framed exactly like the pane it split from
    /// (focus itself is unchanged by a split).
    pub(crate) fn inherit(&mut self, new_leaf: PaneId, content: PaneContent, from: CameraState) {
        self.warm.insert(new_leaf, (content, from));
    }

    /// The warm camera for `id`, if the leaf has one. Read by the render path to
    /// frame a non-active scene (P2.2d: the companion schematic pane's camera,
    /// which lives here — not in the `Runtime`'s active `camera` field — because
    /// the board scene leaf owns the active camera even while the schematic is
    /// focused). `None` before the leaf is first seen (the render path falls back
    /// to fit-to-bounds, keeping the pre-P2.2d static-fit default byte-identical).
    pub(crate) fn camera(&self, id: PaneId, content: PaneContent) -> Option<CameraState> {
        self.warm
            .get(&id)
            .filter(|(stored, _)| *stored == content)
            .map(|(_, camera)| *camera)
    }

    /// A mutable handle to `id`'s warm camera, creating it via `init` (its INITIAL
    /// framing) the first time it is touched. Backs interactive pan/zoom/fit on a
    /// warm-but-non-active pane (P2.2d: the focused schematic pane), so the gesture
    /// persists warm exactly like the board camera does.
    pub(crate) fn entry_or_insert_with(
        &mut self,
        id: PaneId,
        content: PaneContent,
        init: impl FnOnce() -> CameraState,
    ) -> &mut CameraState {
        let mut init = Some(init);
        let entry = self
            .warm
            .entry(id)
            .or_insert_with(|| (content, init.take().expect("initializer used once")()));
        if entry.0 != content {
            *entry = (content, init.take().expect("initializer used once")());
        }
        &mut entry.1
    }

    /// Drop cameras for leaves that no longer exist (after a close or a preset
    /// that rebuilt the tree), so the warm store never leaks stale ids.
    pub(crate) fn retain_live(&mut self, live: &[PaneId]) {
        self.warm.retain(|id, _| live.contains(id));
    }

    /// Reset the store to a single focused leaf with the given active camera —
    /// used when a preset replaces the whole tree with fresh ids.
    pub(crate) fn reset(&mut self, focused: PaneId, content: PaneContent, active: CameraState) {
        self.warm.clear();
        self.warm.insert(focused, (content, active));
    }

    #[cfg(test)]
    pub(crate) fn warm_camera(&self, id: PaneId, content: PaneContent) -> Option<CameraState> {
        self.camera(id, content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use datum_gui_protocol::{SceneBounds, SplitOrientation, WorkspaceLayout, WorkspacePreset};

    fn cam(zoom: f32) -> CameraState {
        CameraState {
            center_x_nm: 0.0,
            center_y_nm: 0.0,
            zoom,
        }
    }

    fn fit() -> CameraState {
        CameraState::fit_to_bounds(&SceneBounds {
            min_x: 0,
            min_y: 0,
            max_x: 1_000_000,
            max_y: 1_000_000,
        })
    }

    /// Drive the exact focus-swap the Runtime does and prove a leaf's camera is
    /// WARM across a focus switch away and back: pan/zoom the focused pane, switch
    /// focus away, switch back, and the original camera returns untouched (not
    /// refit). This is the "no noticeable lag" guarantee at the camera layer.
    #[test]
    fn focused_camera_is_warm_across_focus_switch() {
        let mut layout = WorkspaceLayout::board_schematic();
        let board = layout.focused; // PaneId(0)
        let mut cameras = PaneCameras::new(board, PaneContent::Board, fit());

        // The user pans/zooms the Board pane to a specific framing.
        let mut active = cam(3.5);

        // Focus the Schematic leaf: stash Board's camera, activate Schematic's
        // (fresh -> fit).
        let outgoing = layout.focused;
        layout.focus_next();
        let incoming = layout.focused;
        active = cameras.focus_to(
            outgoing,
            PaneContent::Board,
            active,
            incoming,
            PaneContent::Schematic,
            fit,
        );
        assert_eq!(active, fit(), "a never-focused leaf initializes to fit");
        assert_ne!(incoming, board);

        // Frame the Schematic pane differently.
        active = cam(0.6);

        // Focus back to the Board leaf: its camera must return exactly as left
        // (zoom 3.5), never reset/refit.
        let outgoing = layout.focused;
        layout.focus_prev();
        let incoming = layout.focused;
        assert_eq!(incoming, board);
        active = cameras.focus_to(
            outgoing,
            PaneContent::Schematic,
            active,
            incoming,
            PaneContent::Board,
            fit,
        );
        assert_eq!(
            active,
            cam(3.5),
            "the Board leaf's camera stayed warm across a focus switch away and back"
        );
        // And the Schematic leaf's warm camera is preserved too.
        assert_eq!(
            cameras.warm_camera(PaneId(1), PaneContent::Schematic),
            Some(cam(0.6))
        );
    }

    /// A split child inherits the focused sibling's warm framing (not a refit).
    #[test]
    fn split_child_inherits_sibling_camera() {
        let mut layout = WorkspaceLayout::single();
        let board = layout.focused;
        let mut cameras = PaneCameras::new(board, PaneContent::Board, cam(2.0));
        let active = cam(2.0);

        let before: std::collections::BTreeSet<_> = layout.leaves().into_iter().collect();
        layout.split_focused(SplitOrientation::Vertical);
        for id in layout.leaves() {
            if !before.contains(&id) {
                cameras.inherit(id, PaneContent::Schematic, active);
                assert_eq!(
                    cameras.warm_camera(id, PaneContent::Schematic),
                    Some(cam(2.0))
                );
            }
        }
        // Focus is unchanged by a split; the original camera is still active.
        assert_eq!(layout.focused, board);
    }

    /// A preset rebuilds the tree with fresh ids; the store resets and drops the
    /// stale cameras rather than leaking them.
    #[test]
    fn preset_resets_and_prunes_stale_cameras() {
        let mut layout = WorkspaceLayout::single();
        let mut cameras = PaneCameras::new(layout.focused, PaneContent::Board, cam(4.0));
        layout.split_focused(SplitOrientation::Vertical);
        cameras.inherit(PaneId(1), PaneContent::Schematic, cam(4.0));

        layout.apply_preset(WorkspacePreset::BoardSchematic);
        cameras.reset(layout.focused, PaneContent::Board, fit());
        assert_eq!(
            cameras.warm_camera(layout.focused, PaneContent::Board),
            Some(fit())
        );

        // Prune anything not in the live tree (belt-and-suspenders after reset).
        cameras.retain_live(&layout.leaves());
        for id in layout.leaves() {
            // Only the focused leaf was seeded; others are cold until first focus.
            let _ = cameras.warm_camera(id, PaneContent::Board);
        }
    }

    #[test]
    fn content_replacement_never_reuses_the_previous_surfaces_camera() {
        let pane = PaneId(7);
        let mut cameras = PaneCameras::new(pane, PaneContent::Board, cam(9.0));
        let schematic = *cameras.entry_or_insert_with(pane, PaneContent::Schematic, || cam(0.5));
        assert_eq!(schematic, cam(0.5));
        assert_eq!(cameras.camera(pane, PaneContent::Board), None);
    }

    #[test]
    fn duplicate_surface_panes_keep_independent_warm_cameras() {
        let first = PaneId(3);
        let second = PaneId(4);
        let mut cameras = PaneCameras::new(first, PaneContent::Board, cam(2.0));
        *cameras.entry_or_insert_with(second, PaneContent::Board, || cam(0.75)) = cam(1.25);

        assert_eq!(cameras.camera(first, PaneContent::Board), Some(cam(2.0)));
        assert_eq!(
            cameras.camera(second, PaneContent::Board),
            Some(cam(1.25))
        );
    }
}
