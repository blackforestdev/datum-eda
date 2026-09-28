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
    prepared: Option<crate::PreparedScene>,
    prepared_revision: u64,
    prepared_hits_pending: bool,
    publication: Option<(Vec<crate::HitRegion>, Option<crate::ConsoleOverlayLayout>)>,
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
        self.prepared = None;
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
            if self.prepared_revision <= revision
                && self.prepared_hits_pending
                && let Some(prepared) = self.prepared.as_mut()
            {
                self.publication = Some((
                    std::mem::take(&mut prepared.hit_regions),
                    prepared.console_overlay_layout(),
                ));
                self.prepared_hits_pending = false;
            }
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
        self.schematic_history
            .invalidate_surface_size(&mut self.schematic);
    }

    pub fn change_document(
        &mut self,
        previous: RetainedSceneCacheKey,
        next: &RetainedSceneCacheKey,
    ) {
        self.revisions.update(Change::Content);
        self.prepared = None;
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

impl RenderSession {
    pub fn prepared(&self) -> Option<&crate::PreparedScene> {
        self.prepared.as_ref()
    }

    /// Transitional preparation adapter. Scene construction will join typed
    /// input submission; this owner already controls storage and publication.
    pub fn install_prepared(&mut self, prepared: crate::PreparedScene) {
        self.prepared_revision = self.revisions.update(Change::Composition);
        self.prepared = Some(prepared);
        self.prepared_hits_pending = true;
    }

    pub fn refresh_interaction(&mut self, state: &ReviewWorkspaceState) {
        self.interaction_changed();
        if let (Some(prepared), Some(retained)) = (&mut self.prepared, &self.board) {
            prepared.refresh_interaction(state, retained);
            self.prepared_revision = self.revisions.current();
        }
    }

    /// Only complete_frame can create a publication. Native adapters transfer
    /// this read-only snapshot to their input projection after matching success.
    pub fn take_published_frame(
        &mut self,
    ) -> Option<(Vec<crate::HitRegion>, Option<crate::ConsoleOverlayLayout>)> {
        self.publication.take()
    }
}

impl crate::Renderer {
    /// Lend one immutable prepared snapshot while mutating GPU resource owners.
    /// Move its envelope, never clone the geometry or borrow through a mutable
    /// alias. A concurrent semantic update prevents reinserting an old snapshot.
    pub fn with_prepared_scene<R>(
        &mut self,
        encode: impl FnOnce(&mut Self, &crate::PreparedScene) -> anyhow::Result<R>,
    ) -> anyhow::Result<R> {
        let prepared = self
            .render_session
            .prepared
            .take()
            .ok_or_else(|| anyhow::anyhow!("shared prepared scene must exist before encode"))?;
        let revision = self.render_session.revisions.current();
        let result = encode(self, &prepared);
        if self.render_session.revisions.current() == revision {
            self.render_session.prepared = Some(prepared);
        }
        result
    }
}

#[cfg(test)]
#[path = "prepared_session_tests.rs"]
mod prepared_tests;
