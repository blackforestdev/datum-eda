//! Per-host derived rendering state. Editors supply source and view inputs;
//! active payloads and bounded history share one lifetime/accounting owner.
use crate::RetainedScene;
use datum_gui_protocol::ReviewWorkspaceState;
#[path = "retained_scene_history.rs"]
mod history;
use history::RetainedSceneHistory;
pub use history::{RetainedSceneCacheKey, retained_selection_cache_key};

#[derive(Default)]
pub struct RenderSession {
    board: Option<RetainedScene>,
    board_history: RetainedSceneHistory,
    schematic: Option<RetainedScene>,
    schematic_history: RetainedSceneHistory,
}

impl RenderSession {
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
        self.board = None;
        self.board_history.clear();
        self.clear_schematic();
    }

    fn clear_schematic(&mut self) {
        self.schematic = None;
        self.schematic_history.clear();
    }

    pub fn resize_content(&mut self) {
        self.board_history.invalidate_surface_size(&mut self.board);
        self.schematic_history
            .invalidate_surface_size(&mut self.schematic);
    }

    pub fn change_document(
        &mut self,
        previous: RetainedSceneCacheKey,
        next: &RetainedSceneCacheKey,
    ) {
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
