//! Warm grid LOD memory owned by pane + surface identity.
//!
//! Camera motion deliberately does not mutate this store: each subsequent
//! frame feeds the previous state back through `GridEngine::resolve_lod`, which
//! supplies the governed 20/80 px hysteresis. Retargeting a pane to another
//! surface resets only that pane; duplicate surfaces remain independent.

use datum_gui_protocol::{PaneContent, PaneId};
use datum_gui_viewport::GridLodState;
use std::collections::BTreeMap;

use super::{CameraState, PaneCameras, Runtime};

#[derive(Debug, Default)]
pub(crate) struct PaneGridLod {
    warm: BTreeMap<PaneId, (PaneContent, GridLodState)>,
}

impl Runtime {
    pub(super) fn workspace(&self) -> &datum_gui_protocol::ReviewWorkspaceState {
        self.session.workspace()
    }

    pub(crate) fn apply_prepared_grid_lod(
        &mut self,
        prepared: &mut datum_gui_render::PreparedScene,
    ) {
        let passes = prepared.surface_passes().to_vec();
        for pass in passes {
            let content = match pass.surface {
                datum_gui_render::SceneSurface::Board => PaneContent::Board,
                datum_gui_render::SceneSurface::Schematic => PaneContent::Schematic,
            };
            if let Some(camera) = camera_for_render(
                self.scene_leaf_id(),
                self.camera,
                &self.pane_cameras,
                pass.pane_id,
                content,
            ) {
                prepared.set_surface_camera(pass.pane_id, camera);
            }
        }
        self.pane_grid_lod.apply_to_prepared(prepared);
    }
}

/// Resolve the single authoritative camera for a rendered pane. The active
/// Board camera lives in `Runtime::camera`; only inactive Board panes and all
/// other surfaces read their independent camera from the warm pane store.
fn camera_for_render(
    active_board: Option<PaneId>,
    active_camera: CameraState,
    warm: &PaneCameras,
    pane: PaneId,
    content: PaneContent,
) -> Option<CameraState> {
    if content == PaneContent::Board && active_board == Some(pane) {
        Some(active_camera)
    } else {
        warm.camera(pane, content)
    }
}

impl PaneGridLod {
    /// Feed every prepared pane its own warm state and persist the resolved
    /// result for the next frame. This mutates immediate descriptors only; it
    /// never touches retained authored geometry.
    pub(crate) fn apply_to_prepared(&mut self, prepared: &mut datum_gui_render::PreparedScene) {
        let passes = prepared.surface_passes().to_vec();
        for pass in passes {
            let content = match pass.surface {
                datum_gui_render::SceneSurface::Board => PaneContent::Board,
                datum_gui_render::SceneSurface::Schematic => PaneContent::Schematic,
            };
            let previous = self.previous(pass.pane_id, content);
            let resolved = datum_gui_render::resolve_surface_grid_lod(
                pass.surface,
                pass.scene_viewport,
                &pass.bounds,
                pass.camera,
                previous,
            );
            prepared.set_surface_grid_lod(pass.pane_id, previous, resolved);
            self.update(pass.pane_id, content, resolved);
        }
    }

    pub(crate) fn previous(&mut self, pane: PaneId, content: PaneContent) -> GridLodState {
        match self.warm.get(&pane).copied() {
            Some((stored, state)) if stored == content => state,
            _ => {
                self.warm.insert(pane, (content, GridLodState::default()));
                GridLodState::default()
            }
        }
    }

    pub(crate) fn update(&mut self, pane: PaneId, content: PaneContent, state: GridLodState) {
        self.warm.insert(pane, (content, state));
    }

    /// Reset a retargeted pane immediately, before its next prepared frame.
    pub(crate) fn retarget(&mut self, pane: PaneId, content: PaneContent) {
        if self.warm.get(&pane).is_some_and(|(old, _)| *old != content) {
            self.warm.remove(&pane);
        }
    }

    /// Layout presets allocate fresh pane identities and intentionally discard
    /// all prior view state.
    pub(crate) fn reset(&mut self) {
        self.warm.clear();
    }

    #[cfg(test)]
    fn state(&self, pane: PaneId) -> Option<(PaneContent, GridLodState)> {
        self.warm.get(&pane).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use datum_gui_viewport::{GridConfig, GridEngine, GridMark, GridMode, GridTier, WeightClass};

    static TIERS: [GridTier; 3] = [
        GridTier {
            major_pitch_nm: (100, 100),
            minor_pitch_nm: None,
        },
        GridTier {
            major_pitch_nm: (50, 50),
            minor_pitch_nm: Some((20, 20)),
        },
        GridTier {
            major_pitch_nm: (20, 20),
            minor_pitch_nm: Some((5, 5)),
        },
    ];

    fn config() -> GridConfig {
        GridConfig {
            mode: GridMode::Square,
            mark: GridMark::Lines,
            weight: WeightClass::ScreenConstant(1.0),
            minor_color: [0.0; 3],
            major_color: [0.0; 3],
            tiers: &TIERS,
            origin_nm: None,
        }
    }

    #[test]
    fn camera_motion_preserves_warm_state() {
        let pane = PaneId(4);
        let mut lod = PaneGridLod::default();
        lod.update(pane, PaneContent::Board, GridLodState { tier: Some(2) });
        assert_eq!(
            lod.previous(pane, PaneContent::Board),
            GridLodState { tier: Some(2) }
        );
    }

    #[test]
    fn content_replacement_resets_only_mismatched_surface() {
        let pane = PaneId(4);
        let other = PaneId(9);
        let mut lod = PaneGridLod::default();
        lod.update(pane, PaneContent::Board, GridLodState { tier: Some(2) });
        lod.update(other, PaneContent::Board, GridLodState { tier: Some(1) });
        lod.retarget(pane, PaneContent::Schematic);
        assert_eq!(
            lod.previous(pane, PaneContent::Schematic),
            GridLodState::default()
        );
        assert_eq!(lod.state(other).unwrap().1.tier, Some(1));
    }

    #[test]
    fn duplicate_surface_panes_are_independent() {
        let mut lod = PaneGridLod::default();
        lod.update(
            PaneId(1),
            PaneContent::Board,
            GridLodState { tier: Some(0) },
        );
        lod.update(
            PaneId(2),
            PaneContent::Board,
            GridLodState { tier: Some(2) },
        );
        assert_eq!(lod.previous(PaneId(1), PaneContent::Board).tier, Some(0));
        assert_eq!(lod.previous(PaneId(2), PaneContent::Board).tier, Some(2));
    }

    #[test]
    fn active_board_render_uses_live_zoom_instead_of_stale_warm_camera() {
        let pane = PaneId(1);
        let stale = CameraState {
            center_x_nm: 10.0,
            center_y_nm: 20.0,
            zoom: 1.0,
        };
        let zoomed = CameraState { zoom: 2.4, ..stale };
        let warm = PaneCameras::new(pane, PaneContent::Board, stale);

        assert_eq!(
            camera_for_render(Some(pane), zoomed, &warm, pane, PaneContent::Board),
            Some(zoomed)
        );
    }

    #[test]
    fn inactive_duplicate_board_keeps_its_independent_warm_camera() {
        let active = PaneId(1);
        let inactive = PaneId(2);
        let active_camera = CameraState {
            center_x_nm: 10.0,
            center_y_nm: 20.0,
            zoom: 2.4,
        };
        let inactive_camera = CameraState {
            center_x_nm: 30.0,
            center_y_nm: 40.0,
            zoom: 0.75,
        };
        let mut warm = PaneCameras::new(active, PaneContent::Board, CameraState {
            zoom: 1.0,
            ..active_camera
        });
        warm.inherit(inactive, PaneContent::Board, inactive_camera);

        assert_eq!(
            camera_for_render(
                Some(active),
                active_camera,
                &warm,
                inactive,
                PaneContent::Board,
            ),
            Some(inactive_camera)
        );
    }

    #[test]
    fn sequential_frames_apply_twenty_eighty_hysteresis() {
        let pane = PaneId(1);
        let mut warm = PaneGridLod::default();
        warm.update(pane, PaneContent::Board, GridLodState { tier: Some(2) });

        // Fine spacing is 15 px: below the 20 px coarsen knee, so one frame
        // moves to Normal and stores that resolution for the next frame.
        let first =
            GridEngine::resolve_lod(&config(), 3.0, warm.previous(pane, PaneContent::Board));
        assert_eq!(first.tier, Some(1));
        warm.update(pane, PaneContent::Board, first);

        // Normal's finer pitch is now 60 px. It stays Normal because re-fine
        // requires 80 px; a stateless 20 px selector would incorrectly chatter.
        let held = GridEngine::resolve_lod(&config(), 3.0, warm.previous(pane, PaneContent::Board));
        assert_eq!(held.tier, Some(1));

        let refined =
            GridEngine::resolve_lod(&config(), 16.0, warm.previous(pane, PaneContent::Board));
        assert_eq!(refined.tier, Some(2));
    }
}
