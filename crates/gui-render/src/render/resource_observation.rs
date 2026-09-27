//! Read-only resource views. Cache capacities overlap scoped heap ownership;
//! reservation peaks include pending construction and are not driver residency.
pub use crate::text_gpu::GlyphPreparationCounts;
use crate::text_gpu::budget::Budget;
pub use crate::text_layout::fonts::Usage as FontCpuUsage;
use std::sync::Arc;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReservationUsage {
    pub budget_id: u64,
    pub reserved_bytes: u64,
    pub lifetime_peak_reserved_bytes: u64,
    pub limit_bytes: u64,
}
impl ReservationUsage {
    pub(crate) fn of(budget: &Budget) -> Self {
        Self {
            budget_id: budget.id(),
            reserved_bytes: budget.used(),
            lifetime_peak_reserved_bytes: budget.peak(),
            limit_bytes: budget.limit(),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct LocalReservationUsage {
    pub renderer_id: u64,
    pub screen: ReservationUsage,
    pub control_mesh: ReservationUsage,
    pub atlas: ReservationUsage,
    pub staging: ReservationUsage,
}
impl LocalReservationUsage {
    pub fn released(&self) -> bool {
        [self.screen, self.control_mesh, self.atlas, self.staging]
            .iter()
            .all(|budget| budget.reserved_bytes == 0)
    }
}

/// Holds only four existing budget records, never GPU resources or payloads.
/// Device replacements can share budget IDs: incidence must not multiply bytes.
pub struct LocalReservationObserver {
    renderer_id: u64,
    screen: Arc<Budget>,
    control_mesh: Arc<Budget>,
    atlas: Arc<Budget>,
    staging: Arc<Budget>,
}
impl LocalReservationObserver {
    pub fn renderer_id(&self) -> u64 {
        self.renderer_id
    }
    /// Independent sampled counters, not an atomic transaction across owners.
    pub fn usage(&self) -> LocalReservationUsage {
        LocalReservationUsage {
            renderer_id: self.renderer_id,
            screen: ReservationUsage::of(&self.screen),
            control_mesh: ReservationUsage::of(&self.control_mesh),
            atlas: ReservationUsage::of(&self.atlas),
            staging: ReservationUsage::of(&self.staging),
        }
    }
    /// Per distinct budget ID; several observers can retain the same allocation.
    pub fn budget_metadata_bytes_each() -> usize {
        Budget::cpu_allocation_bytes()
    }
}
impl crate::Renderer {
    /// Workspace and overlay counts remain valid across paint reuse, but become
    /// absent on failed/cancelled preparation and on renderer replacement.
    /// These are preparation groups, not per-pane or complete ADM-01 admission.
    pub fn glyph_preparation_counts(&self) -> [Option<GlyphPreparationCounts>; 2] {
        [
            self.text_renderer.preparation_counts,
            self.menu_overlay_text_renderer.preparation_counts,
        ]
    }

    pub fn local_reservation_observer(&self) -> LocalReservationObserver {
        LocalReservationObserver {
            renderer_id: self.resource_owner_id(),
            screen: self.screen_budget.clone(),
            control_mesh: self.control_gpu_budget.clone(),
            atlas: self.atlas.texture_reservation_budget(),
            staging: self.atlas.staging_budget.clone(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct DocumentCpuUsage {
    pub scene_id: String,
    pub budget_id: u64,
    /// Deduplicated retained geometry, registered history and metadata. Constructor
    /// scratch and scoped allocator views are separate; do not sum overlapping views.
    pub retained_bytes: usize,
    pub history_entries: usize,
    pub limit_bytes: usize,
}
pub fn document_cpu_usage() -> Vec<DocumentCpuUsage> {
    crate::retained_scene_owner::document_cpu::cpu_usage()
}

#[cfg(test)]
#[path = "resource_observation_tests.rs"]
mod tests;

pub use crate::text_admission_observation::{TextAdmissionGroup, TextAdmissionObservation};

pub use crate::geometry_admission::{
    GeometryAdmissionCounts, GeometryAdmissionRange, PreparedGeometryAdmission,
    PreparedSourceIdentities, PreparedSourceIdentity,
};

/// Borrowed state at the end of one render attempt. `Some(true)` means the
/// frame was submitted, not presented; `Some(false)` includes upload-only or
/// deferred acquisition. Errors and missing text observations remain explicit.
pub struct FrameObservation<'a> {
    pub renderer: &'a crate::Renderer,
    pub prepared: &'a crate::PreparedScene,
    pub board: &'a crate::RetainedScene,
    pub schematic: Option<&'a crate::RetainedScene>,
    pub extent: [u32; 2],
    pub submitted_frame: Option<bool>,
    pub error: Option<&'a anyhow::Error>,
    pub text_observation_attempted: bool,
}
pub type FrameObserver = for<'a> fn(FrameObservation<'a>) -> anyhow::Result<()>;
static FRAME_OBSERVER: std::sync::OnceLock<FrameObserver> = std::sync::OnceLock::new();

/// Install before constructing native renderers. Registration retains only a
/// function pointer. A different observer cannot silently replace the writer.
pub fn register_frame_observer(observer: FrameObserver) -> anyhow::Result<()> {
    if let Some(current) = FRAME_OBSERVER.get() {
        anyhow::ensure!(
            std::ptr::fn_addr_eq(*current, observer),
            "frame observer already registered"
        );
        return Ok(());
    }
    FRAME_OBSERVER
        .set(observer)
        .map_err(|_| anyhow::anyhow!("frame observer registration raced"))
}
pub(crate) fn frame_observer() -> Option<FrameObserver> {
    FRAME_OBSERVER.get().copied()
}

pub use crate::gpu_surface_pass::EncodedWorldAdmission;

pub use crate::immediate_admission::{
    PreparedGridAdmission, PreparedScreenGeometry, TerminalGeometryAdmission,
};
