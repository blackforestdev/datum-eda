pub struct Renderer {
    pipeline: wgpu::RenderPipeline,
    world_pipeline: wgpu::RenderPipeline,
    world_stroke_pipeline: wgpu::RenderPipeline,
    terminal_graphics: terminal_graphics::TerminalGraphicsRenderer,
    uniform_bind_group: wgpu::BindGroup,
    uniform_buffer: wgpu::Buffer,
    scene_bind_group: wgpu::BindGroup,
    scene_bind_group_layout: wgpu::BindGroupLayout,
    surface_world_bundles: Vec<gpu_surface_pass::CachedSurfaceBundle>,
    surface_scene_uniforms: Vec<(wgpu::Buffer, wgpu::BindGroup)>,
    surface_grid_vertex_buffer: Option<wgpu::Buffer>,
    surface_grid_vertex_capacity: usize,
    schematic_scene_bind_group: wgpu::BindGroup,
    schematic_world_vertex_buffer: Option<wgpu::Buffer>,
    schematic_world_vertex_capacity: usize,
    schematic_world_vertex_source: Option<std::sync::Arc<[Vertex]>>,
    schematic_world_stroke_buffer: Option<wgpu::Buffer>,
    schematic_world_stroke_capacity: usize,
    schematic_world_stroke_source_ptr: usize,
    schematic_world_stroke_source_len: usize,
    schematic_underlay_vertex_buffer: Option<wgpu::Buffer>,
    schematic_underlay_vertex_capacity: usize,
    schematic_overlay_vertex_buffer: Option<wgpu::Buffer>,
    schematic_overlay_vertex_capacity: usize,
    font_system: FontSystem,
    swash_cache: SwashCache,
    viewport: Viewport,
    atlas: TextAtlas,
    text_renderer: TextRenderer,
    menu_overlay_text_renderer: TextRenderer,
    text_buffer_cache: Vec<CachedTextBuffer>,
    text_buffer_frame: u64,
    last_text_prepare_signature: Option<TextPrepareSignature>,
    panel_vertex_buffer: Option<wgpu::Buffer>,
    panel_vertex_capacity: usize,
    viewport_underlay_vertex_buffer: Option<wgpu::Buffer>,
    viewport_underlay_vertex_capacity: usize,
    viewport_overlay_vertex_buffer: Option<wgpu::Buffer>,
    viewport_overlay_vertex_capacity: usize,
    board_interaction_vertex_buffer: Option<wgpu::Buffer>,
    board_interaction_vertex_capacity: usize,
    console_gpu: gpu_console::ConsoleGpuResources,
    menu_overlay_vertex_buffer: Option<wgpu::Buffer>,
    menu_overlay_vertex_capacity: usize,
    world_vertex_buffer: Option<wgpu::Buffer>,
    world_vertex_capacity: usize,
    world_vertex_source: Option<std::sync::Arc<[Vertex]>>,
    world_stroke_buffer: Option<wgpu::Buffer>,
    world_stroke_capacity: usize,
    world_stroke_source_ptr: usize,
    world_stroke_source_len: usize,
    msaa_view: Option<wgpu::TextureView>,
    msaa_size: (u32, u32),
    msaa_format: wgpu::TextureFormat,
    msaa_samples: u32,
}

#[path = "gpu_console.rs"]
mod gpu_console;
#[path = "gpu_vertex_upload.rs"]
mod gpu_vertex_upload;
#[path = "terminal_graphics.rs"]
mod terminal_graphics;
#[path = "text_buffer_cache.rs"]
mod text_buffer_cache;

#[path = "gpu_init.rs"]
mod gpu_init;
#[path = "gpu_overlay.rs"]
mod gpu_overlay;

#[path = "gpu_frame.rs"]
mod gpu_frame;
