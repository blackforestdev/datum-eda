//! Per-host derived rendering state. Editors supply source and view inputs;
//! active payloads and bounded history share one lifetime/accounting owner.
use crate::{RetainedScene, TerminalPaneRenderState};
#[path = "terminal_damage.rs"]
mod terminal_damage;
use terminal_damage::PendingTerminalDamage;
#[path = "session_grid.rs"]
mod session_grid;
#[path = "session_preparation.rs"]
mod session_preparation;
pub use session_preparation::WorkspaceView;
#[path = "session_dialog.rs"]
mod session_dialog;
pub use session_dialog::{DialogInput, DialogView};
use session_dialog::{Preparation, PreparedProfile};
#[path = "session_pointer.rs"]
pub mod render_input;
#[path = "session_frame.rs"]
mod session_frame;
#[path = "session_hover.rs"]
mod session_hover;
#[path = "session_sources.rs"]
mod session_sources;
pub use session_frame::{FramePlan, SubmittedFrame};
#[path = "frame_revision.rs"]
mod frame_revision;
use datum_gui_protocol::ReviewWorkspaceState;
use frame_revision::{Change, Receipt, Revisions, Target};
#[path = "retained_scene_history.rs"]
mod history;
use history::RetainedSceneCacheKey;
use history::RetainedSceneHistory;

#[derive(Default)]
pub struct RenderSession {
    revisions: Revisions,
    sources: session_sources::Sources,
    prepared: Option<crate::PreparedScene>,
    prepared_revision: u64,
    preparation_generation: u64,
    preparation: Option<Preparation>,
    prepared_hits_pending: bool,
    publication: Option<(Vec<crate::HitRegion>, Option<crate::ConsoleOverlayLayout>)>,
    terminal_damage: PendingTerminalDamage,
    terminal_cache: crate::TerminalRenderCache,
    grid_lod: session_grid::PaneGridLod,
    board: Option<RetainedScene>,
    empty: std::sync::OnceLock<RetainedScene>,
    board_history: RetainedSceneHistory,
    schematic: Option<RetainedScene>,
    schematic_history: RetainedSceneHistory,
}

impl RenderSession {
    pub fn content_revision(&self) -> u64 {
        self.revisions.current()
    }
    pub fn has_pending_frame(&self) -> bool {
        self.revisions.pending()
    }
    pub fn interaction_only_damage(&self) -> bool {
        self.revisions.interaction_only()
    }
    pub fn composition_changed(&mut self) {
        self.revisions.update(Change::Composition);
        self.prepared = None;
    }
    fn begin_frame(&mut self, host: u64, device: u64, configuration: u64, native: bool) -> Receipt {
        self.revisions.begin(
            Target {
                host,
                device,
                configuration,
            },
            native,
        )
    }
    #[cfg(test)]
    fn complete_frame(
        &mut self,
        plan: FramePlan,
        host: u64,
        device: u64,
        configuration: u64,
        presented: bool,
    ) -> bool {
        self.finish_snapshot(
            plan,
            Target {
                host,
                device,
                configuration,
            },
            presented,
        )
    }

    pub fn retain_terminal_damage(&mut self, panes: &[TerminalPaneRenderState<'_>]) {
        let revision = self.revisions.update(Change::Composition);
        self.terminal_damage.retain(revision, panes);
    }

    pub fn restore_terminal_damage(&mut self) -> Vec<(String, Vec<datum_terminal_core::Damage>)> {
        self.terminal_damage.restore()
    }
    pub fn retire_target(&mut self) {
        self.revisions.retire_target();
        // A replacement native host starts with no displayed hit projection.
        // Rebuild even when CPU retained content survives the device transfer.
        self.prepared = None;
        self.prepared_hits_pending = false;
        self.publication = None;
    }

    /// Read-only projection for hit testing. Clones share charged immutable data.
    pub fn board(&self) -> Option<&RetainedScene> {
        self.board.as_ref()
    }

    pub fn schematic(&self) -> Option<&RetainedScene> {
        self.schematic.as_ref()
    }

    pub fn retry_content(&mut self) {
        self.board_history.retry_construction();
        self.schematic_history.retry_construction();
    }

    pub fn check_content_budget(&mut self) -> anyhow::Result<()> {
        self.board_history.check_render_budget()?;
        self.schematic_history.check_render_budget()
    }

    pub fn clear_content(&mut self) {
        self.revisions.update(Change::Content);
        self.prepared = None;
        self.board = None;
        self.sources.active = None;
        self.board_history.clear();
        self.clear_schematic();
    }

    fn clear_schematic(&mut self) {
        self.schematic = None;
        self.schematic_history.clear();
    }

    pub fn resize_content(&mut self) {
        self.retire_target();
        self.board_history.invalidate_surface_size(&mut self.board);
        if self.board.is_none() {
            self.sources.active = None;
        }
        self.schematic_history
            .invalidate_surface_size(&mut self.schematic);
    }

    pub(crate) fn ensure_board(
        &mut self,
        state: &ReviewWorkspaceState,
        width: u32,
        height: u32,
        scale: f32,
    ) -> bool {
        if self.board_history.construction_error.is_some() {
            return false;
        }
        if self.board.is_none() {
            let Some(scene) = self.board_history.construct(&state.scene.scene_id, || {
                RetainedScene::try_from_workspace_for_surface(state, width, height, scale)
            }) else {
                return false;
            };
            self.board_history.limit_for_active(&scene);
            self.board = Some(scene);
        }
        true
    }

    pub(crate) fn ensure_schematic(
        &mut self,
        state: &ReviewWorkspaceState,
        width: u32,
        height: u32,
        scale: f32,
    ) -> bool {
        if self.schematic_history.construction_error.is_some() {
            return false;
        }
        if self.schematic.is_none() {
            let Some(schematic) = &state.schematic_scene else {
                return true;
            };
            let Some(scene) = self.schematic_history.construct(&schematic.scene_id, || {
                RetainedScene::try_from_workspace_schematic_for_surface(state, width, height, scale)
            }) else {
                return false;
            };
            if let Some(scene) = &scene {
                self.schematic_history.limit_for_active(scene);
            }
            self.schematic = scene;
        }
        true
    }
}

impl RenderSession {
    pub fn prepared(&self) -> Option<&crate::PreparedScene> {
        self.prepared.as_ref()
    }

    // Only shared preparation owners install derived scene/profile pairs.
    fn install_prepared(&mut self, prepared: crate::PreparedScene, preparation: Preparation) {
        self.preparation = Some(preparation);
        self.prepared_revision = self.revisions.update(Change::Composition);
        self.preparation_generation = self.prepared_revision;
        self.prepared = Some(prepared);
        self.prepared_hits_pending = true;
    }

    /// Only matching submitted-frame completion can create a publication. Native adapters transfer
    /// this read-only snapshot to their input projection after matching success.
    pub fn take_published_frame(
        &mut self,
    ) -> Option<(Vec<crate::HitRegion>, Option<crate::ConsoleOverlayLayout>)> {
        self.publication.take()
    }
}

#[cfg(test)]
#[path = "prepared_session_tests.rs"]
mod prepared_tests;
