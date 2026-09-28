//! Source replacement and retained dependency validation belong to the shared session.
use super::*;
use std::sync::atomic::{AtomicU64, Ordering};

/// Producer-owned epoch for immutable board/schematic/review/check/proposal payloads.
/// Advance whenever any such payload changes, even with identical IDs, revisions,
/// addresses or lengths. UI/camera changes are compared by the shared owner.
/// Producers without that guarantee must submit `None` (replacement every time).
pub struct SourceEpoch(SourceRevision);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SourceRevision {
    owner: u64,
    generation: u64,
}

impl Default for SourceEpoch {
    fn default() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(1);
        let owner = NEXT
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |value| {
                value.checked_add(1)
            })
            .expect("source producer identity exhausted");
        Self(SourceRevision {
            owner,
            generation: 1,
        })
    }
}

impl SourceEpoch {
    pub fn revision(&self) -> SourceRevision {
        self.0
    }
    pub fn replaced(&mut self) {
        self.0.generation = self
            .0
            .generation
            .checked_add(1)
            .expect("source producer generation exhausted");
    }
}

#[derive(Default)]
pub(super) struct Sources {
    revision: Option<SourceRevision>,
    pub(super) active: Option<Active>,
}

pub(super) struct Active {
    key: RetainedSceneCacheKey,
    _charge: crate::DocumentCpuCharge,
}

impl RenderSession {
    /// A full input is authoritative even when an editor supplied a weak damage hint.
    /// Source epochs are independent of device recovery and of source identity strings.
    pub fn ensure_sources(
        &mut self,
        state: &ReviewWorkspaceState,
        revision: Option<SourceRevision>,
        width: u32,
        height: u32,
        scale: f32,
    ) -> anyhow::Result<()> {
        let result = self.validate_sources(state, revision, width, height, scale);
        if result.is_err() {
            self.composition_changed();
            self.preparation = None;
        }
        result
    }

    fn validate_sources(
        &mut self,
        state: &ReviewWorkspaceState,
        revision: Option<SourceRevision>,
        width: u32,
        height: u32,
        scale: f32,
    ) -> anyhow::Result<()> {
        if revision.is_none() || self.sources.revision != revision {
            self.clear_content();
            self.preparation = None;
            self.sources.revision = revision;
        }
        if self
            .sources
            .active
            .as_ref()
            .is_some_and(|active| active.key.scene_id != state.scene.scene_id)
        {
            self.clear_content();
            self.preparation = None;
        }
        if self.sources.active.is_none() && (self.board.is_some() || self.schematic.is_some()) {
            self.clear_content();
            self.preparation = None;
        }
        // Admit key construction before allocating owned strings. The bound covers
        // String formatting spare capacity; actual capacity is checked below.
        anyhow::ensure!(
            self.ensure_board(state, width, height, scale),
            "shared retained board preparation refused"
        );
        let reserved = RetainedSceneCacheKey::allocation_bound(state)
            .ok_or_else(|| anyhow::anyhow!("source key overflow"))?;
        let charge = self
            .board
            .as_ref()
            .expect("constructed board")
            .geometry_observer()
            .try_charge_document_storage(reserved)
            .ok_or_else(|| anyhow::anyhow!("active source key exceeds document CPU budget"))?;
        let next = RetainedSceneCacheKey::for_workspace(state, width, height, scale);
        if let Some(active) = self.sources.active.as_ref() {
            let same_content = active.key.same_content(&next);
            let keep_board = same_content
                && self.board.as_ref().is_some_and(|scene| {
                    scene.can_reuse_for_surface_resize() || active.key.same_projection(&next, false)
                });
            let keep_schematic = same_content
                && next.schematic_present()
                && self.schematic.as_ref().is_some_and(|scene| {
                    scene.can_reuse_for_surface_resize() || active.key.same_projection(&next, true)
                });
            if active.key == next && self.board.is_some() {
                drop(next);
                drop(charge);
                anyhow::ensure!(
                    self.ensure_schematic(state, width, height, scale),
                    "shared retained schematic preparation refused"
                );
                return self.check_content_budget();
            }
            // A new retained hit array cannot inherit a preparation's region indices.
            self.composition_changed();
            self.preparation = None;
            if !keep_board {
                self.revisions.update(Change::Content);
                let active = self.sources.active.take().expect("active source key");
                drop(active._charge);
                if let Some(board) = self.board.take() {
                    self.board_history.insert(active.key, board);
                }
                self.board = self.board_history.take(&next);
            }
            if !keep_schematic {
                self.clear_schematic();
            }
        }
        anyhow::ensure!(
            self.ensure_board(state, width, height, scale),
            "shared retained board preparation refused"
        );
        anyhow::ensure!(
            next.heap_bytes().is_some_and(|bytes| bytes <= reserved),
            "source key allocation exceeded admitted capacity"
        );
        self.sources.active = Some(Active {
            key: next,
            _charge: charge,
        });
        anyhow::ensure!(
            self.ensure_schematic(state, width, height, scale),
            "shared retained schematic preparation refused"
        );
        self.check_content_budget()
    }
}

#[cfg(test)]
#[path = "session_sources_tests.rs"]
mod tests;
