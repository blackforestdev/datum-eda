//! Per-host derived rendering state. Editors supply source and view inputs;
//! active payloads and bounded history share one lifetime/accounting owner.
use crate::{RetainedScene, TerminalPaneRenderState};
#[path = "terminal_damage.rs"]
mod terminal_damage;
use terminal_damage::PendingTerminalDamage;
#[path = "frame_revision.rs"]
mod frame_revision;
use datum_gui_protocol::ReviewWorkspaceState;
use frame_revision::{Change, Receipt, Revisions, Target};
#[path = "retained_scene_history.rs"]
mod history;
use history::RetainedSceneHistory;
pub use history::{RetainedSceneCacheKey, retained_selection_cache_key};

#[derive(Default)]
pub struct RenderSession {
    revisions: Revisions,
    terminal_damage: PendingTerminalDamage,
    board: Option<RetainedScene>,
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
    }
    pub fn interaction_changed(&mut self) {
        self.revisions.update(Change::Interaction);
    }
    pub fn begin_frame(
        &mut self,
        host: u64,
        device: u64,
        configuration: u64,
        native: bool,
    ) -> Receipt {
        self.revisions.begin(
            Target {
                host,
                device,
                configuration,
            },
            native,
        )
    }
    pub fn complete_frame(
        &mut self,
        receipt: Receipt,
        host: u64,
        device: u64,
        configuration: u64,
        presented: bool,
    ) -> bool {
        let revision = receipt.revision();
        let accepted = self.revisions.complete(
            receipt,
            Target {
                host,
                device,
                configuration,
            },
            presented,
        );
        if accepted {
            self.terminal_damage.presented(revision);
        }
        accepted
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
        self.board = None;
        self.board_history.clear();
        self.clear_schematic();
    }

    fn clear_schematic(&mut self) {
        self.schematic = None;
        self.schematic_history.clear();
    }

    pub fn resize_content(&mut self) {
        self.revisions.retire_target();
        self.board_history.invalidate_surface_size(&mut self.board);
        self.schematic_history
            .invalidate_surface_size(&mut self.schematic);
    }

    pub fn change_document(
        &mut self,
        previous: RetainedSceneCacheKey,
        next: &RetainedSceneCacheKey,
    ) {
        self.revisions.update(Change::Content);
        if let Some(board) = self.board.take() {
            self.board_history.insert(previous, board);
        }
        self.clear_schematic();
        self.board = self.board_history.take(next);
    }

    pub fn ensure_board(
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

    pub fn ensure_schematic(
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
