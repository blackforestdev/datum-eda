//! Renderer-owned resources shared by full-scene and auxiliary rendering.
use super::*;

pub struct Renderer {
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
    pub(super) scene_bind_group: wgpu::BindGroup,
    pub(super) scene_bind_group_layout: wgpu::BindGroupLayout,
    pub(super) surface_world_bundles: Vec<gpu_surface_pass::CachedSurfaceBundle>,
    pub(super) surface_scene_uniforms: Vec<(
        gpu_data::uniform_buffer::UniformBuffer<SceneUniform>,
        wgpu::BindGroup,
    )>,
    pub(super) surface_grid_gpu: gpu_data::screen_buffer::ScreenBuffer,
    pub(super) schematic_scene_bind_group: wgpu::BindGroup,
    pub(super) schematic_underlay_gpu: gpu_data::screen_buffer::ScreenBuffer,
    pub(super) schematic_overlay_gpu: gpu_data::screen_buffer::ScreenBuffer,
    pub(super) font_system: FontSystem,
    pub(super) swash_cache: SwashCache,
    pub(super) viewport: Viewport,
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

impl Renderer {
    /// Cumulative control-mesh builds for this owner, saturating at usize::MAX.
    /// Cache hits do not increment it; this is work count, not resource accounting.
    pub fn control_mesh_build_count(&self) -> usize {
        self.control_meshes.builds
    }

    /// Retained CPU cache storage, including entry capacity and mesh payload.
    /// Does not include transient construction, frame output or GPU buffers.
    pub fn control_mesh_retained_cpu_bytes(&self) -> usize {
        self.control_meshes.retained_cpu_bytes()
    }
}
