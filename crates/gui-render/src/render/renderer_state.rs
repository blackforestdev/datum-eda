//! Renderer-owned resources shared by full-scene and auxiliary rendering.
use super::*;
#[path = "../cpu_alloc.rs"]
pub mod cpu_alloc;
pub use crate::text_buffer_cache::budget::TextCacheOwnerUsage;
use crate::text_gpu::{Atlas as TextAtlas, Draw as TextRenderer};
pub use crate::text_gpu::{
    Kind as TextGpuAllocationKind, Observer as TextGpuAllocationObserver,
    Record as TextGpuAllocation,
};

/// Retained control CPU ownership. Key capacity is a subset of entry storage;
/// do not add it again. Total also includes the inline cache owner itself.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ControlMeshUsage {
    pub entries: usize,
    pub entry_capacity: usize,
    pub key_capacity_bytes: usize,
    pub entry_storage_bytes: usize,
    pub mesh_storage_bytes: usize,
    pub total_bytes: usize,
}

pub struct Renderer {
    pub(super) cold_world: gpu_vertex_upload::ColdWorldUploads,
    pub(super) control_gpu_budget: std::sync::Arc<crate::text_gpu::budget::Budget>,
    pub(super) screen_budget: std::sync::Arc<crate::text_gpu::budget::Budget>,
    pub(super) control_meshes: crate::global_preferences_primitives::ControlMeshCache,
    pub(super) schematic_world_strokes_gpu:
        gpu_data::retained_buffer::RetainedBuffer<WorldStrokeInstance>,
    pub(super) schematic_world_vertices_gpu: gpu_data::retained_buffer::RetainedBuffer<Vertex>,
    pub(super) world_strokes_gpu: gpu_data::retained_buffer::RetainedBuffer<WorldStrokeInstance>,
    pub(super) world_vertices_gpu: gpu_data::retained_buffer::RetainedBuffer<Vertex>,
    pub(super) pipeline: wgpu::RenderPipeline,
    pub(super) world_pipeline: wgpu::RenderPipeline,
    pub(super) world_stroke_pipeline: wgpu::RenderPipeline,
    pub(super) terminal_graphics: terminal_graphics::TerminalGraphicsRenderer,
    pub(super) uniform_bind_group: wgpu::BindGroup,
    pub(super) uniform_buffer: gpu_data::uniform_buffer::UniformBuffer<ScreenUniform>,
    pub(super) scene_bind_group: gpu_data::uniform_buffer::UniformBinding<SceneUniform>,
    pub(super) scene_bind_group_layout: wgpu::BindGroupLayout,
    pub(super) surface_world_bundles: Vec<gpu_surface_pass::CachedSurfaceBundle>,
    pub(super) pane_uniform_generations: crate::text_gpu::slot_generations::SlotGenerations,
    pub(super) surface_scene_uniforms: Vec<gpu_data::uniform_buffer::UniformBinding<SceneUniform>>,
    pub(super) surface_grid_gpu: gpu_data::screen_buffer::ScreenBuffer,
    pub(super) schematic_scene_bind_group: gpu_data::uniform_buffer::UniformBinding<SceneUniform>,
    pub(super) schematic_underlay_gpu: gpu_data::screen_buffer::ScreenBuffer,
    pub(super) schematic_overlay_gpu: gpu_data::screen_buffer::ScreenBuffer,
    pub(super) font_system: crate::text_layout::fonts::Fonts,
    pub(super) text_cpu: crate::cpu_alloc::Scope,
    pub(super) swash_cache: crate::text_gpu::raster::Raster,
    pub(super) text_resolution: [u32; 2],
    pub(super) atlas: TextAtlas,
    pub(super) text_renderer: TextRenderer,
    pub(super) menu_overlay_text_renderer: TextRenderer,
    pub(super) text_buffers: text_buffer_cache::TextBufferCache,
    pub(super) text_preparation: gpu_overlay::gpu_text::GlyphPreparation,
    pub(super) panel_gpu: gpu_data::screen_buffer::ScreenBuffer,
    pub(super) viewport_underlay_gpu: gpu_data::screen_buffer::ScreenBuffer,
    pub(super) viewport_overlay_gpu: gpu_data::screen_buffer::ScreenBuffer,
    pub(super) board_interaction_gpu: gpu_data::screen_buffer::ScreenBuffer,
    pub(super) console_gpu: gpu_console::ConsoleGpuResources,
    pub(super) menu_overlay_gpu: gpu_data::screen_buffer::ScreenBuffer,
    pub(super) surface_attachments: gpu_surface::SurfaceAttachments,
    pub(super) msaa_format: wgpu::TextureFormat,
    pub(super) msaa_samples: u32,
    pub(super) measurements: Option<gpu_measurements::GpuMeasurements>,
}

/// Retained scalar measurement storage for one live cache owner.
/// Font-system/shaping scratch and accounting-registry storage are separate.
#[derive(Clone, Debug)]
pub struct WidthMeasurementCacheUsage {
    pub owner_id: u64,
    pub thread: std::thread::ThreadId,
    pub entries: usize,
    pub key_bytes: usize,
    pub retained_bytes: usize,
}

/// Datum-owned cache keys, unique shape payloads and layout/entry capacities.
/// Includes measured Arc/Datum allocation headers. Font/private scratch,
/// accounting-registry storage and GPU resources remain separate.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TextCacheKeyUsage {
    /// Label memberships, bounded after frame completion. Current preparation
    /// may exceed retention limits; shared workspace storage counts once in totals.
    pub label_entries: usize,
    pub label_key_text_bytes: usize,
    pub owner_id: u64,
    pub entries: usize,
    pub key_text_bytes: usize,
    pub entry_storage_bytes: usize,
    pub shaped_payload_bytes: usize,
}

impl Renderer {
    /// Live text textures/instance buffers, including retiring submission holds.
    /// Excludes staging, CPU shaping, driver residency and other GPU owners.
    pub fn text_gpu_allocation_observer(&self) -> TextGpuAllocationObserver {
        self.atlas.owner.observer()
    }

    pub fn text_gpu_allocations(&self) -> Vec<TextGpuAllocation> {
        self.atlas.owner.records()
    }

    /// Enumerate all live measurement caches, including caches on other threads.
    /// Sum retained_bytes for the process total at this observation point.
    pub fn width_measurement_cache_usage() -> Vec<WidthMeasurementCacheUsage> {
        crate::text_metrics::measurement_cache_usage()
    }

    /// Cumulative control-mesh builds for this owner, saturating at usize::MAX.
    /// Cache hits do not increment it; this is work count, not resource accounting.
    pub fn control_mesh_build_count(&self) -> usize {
        self.control_meshes.builds
    }

    /// Allocated panel/menu capacities admitted to retention, including retiring
    /// submission references. Mixed non-control vertices are counted conservatively.
    /// Uncached frame allocations are counted by the screen/process budgets instead.
    pub fn control_mesh_retained_gpu_bytes(&self) -> u64 {
        self.control_gpu_budget.used()
    }

    pub fn control_mesh_usage(&self) -> ControlMeshUsage {
        self.control_meshes.usage()
    }

    /// Retained CPU cache storage, including entry capacity and mesh payload.
    /// Does not include transient construction, frame output or GPU buffers.
    pub fn control_mesh_retained_cpu_bytes(&self) -> usize {
        self.control_meshes.retained_cpu_bytes()
    }
}
