//! Renderer pipeline, text, and buffer-resource initialization.

use super::*;

impl Renderer {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        msaa_samples: u32,
    ) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("datum-gui-render-shader"),
            source: wgpu::ShaderSource::Wgsl(
                r#"
struct ScreenUniform {
    resolution: vec2<f32>,
    _pad: vec2<f32>,
};

@group(0) @binding(0)
var<uniform> screen: ScreenUniform;

struct VsIn {
    @location(0) pos: vec2<f32>,
    @location(1) color: vec3<f32>,
};

struct VsOut {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec3<f32>,
};

@vertex
fn vs_main(in: VsIn) -> VsOut {
    var out: VsOut;
    let clip = vec2<f32>(
        (in.pos.x / screen.resolution.x) * 2.0 - 1.0,
        1.0 - (in.pos.y / screen.resolution.y) * 2.0
    );
    out.position = vec4<f32>(clip, 0.0, 1.0);
    out.color = in.color;
    return out;
}

// Tokens arrive as sRGB display values; convert to linear so the sRGB surface's
// encode round-trips to the authored color (near-black stays near-black).
fn srgb_to_linear(c: vec3<f32>) -> vec3<f32> {
    let low = c / 12.92;
    let high = pow((c + vec3<f32>(0.055)) / 1.055, vec3<f32>(2.4));
    return select(high, low, c <= vec3<f32>(0.04045));
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    return vec4<f32>(srgb_to_linear(in.color), 1.0);
}
"#
                .into(),
            ),
        });
        let world_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("datum-gui-render-world-shader"),
            source: wgpu::ShaderSource::Wgsl(
                r#"
struct SceneUniform {
    resolution: vec4<f32>,
    viewport_origin: vec4<f32>,
    viewport_size: vec4<f32>,
    camera_center_scale: vec4<f32>,
};

@group(0) @binding(0)
var<uniform> scene: SceneUniform;

struct VsIn {
    @location(0) pos: vec2<f32>,
    @location(1) color: vec3<f32>,
};

struct VsOut {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec3<f32>,
};

@vertex
fn vs_main(in: VsIn) -> VsOut {
    var out: VsOut;
    let screen = vec2<f32>(
        scene.viewport_origin.x + scene.viewport_size.x * 0.5 + (in.pos.x - scene.camera_center_scale.x) * scene.camera_center_scale.z,
        scene.viewport_origin.y + scene.viewport_size.y * 0.5 + (in.pos.y - scene.camera_center_scale.y) * scene.camera_center_scale.z
    );
    let clip = vec2<f32>(
        (screen.x / scene.resolution.x) * 2.0 - 1.0,
        1.0 - (screen.y / scene.resolution.y) * 2.0
    );
    out.position = vec4<f32>(clip, 0.0, 1.0);
    out.color = in.color;
    return out;
}

fn srgb_to_linear(c: vec3<f32>) -> vec3<f32> {
    let low = c / 12.92;
    let high = pow((c + vec3<f32>(0.055)) / 1.055, vec3<f32>(2.4));
    return select(high, low, c <= vec3<f32>(0.04045));
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    return vec4<f32>(srgb_to_linear(in.color), 1.0);
}
"#
                .into(),
            ),
        });
        let uniform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("datum-gui-render-uniform-bgl"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });
        let scene_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("datum-gui-render-scene-bgl"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });
        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("datum-gui-render-uniform-buffer"),
            contents: bytemuck::bytes_of(&ScreenUniform {
                resolution: [1.0, 1.0],
                _pad: [0.0, 0.0],
            }),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("datum-gui-render-uniform-bg"),
            layout: &uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });
        let scene_uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("datum-gui-render-scene-uniform-buffer"),
            contents: bytemuck::bytes_of(&SceneUniform {
                resolution: [1.0, 1.0, 0.0, 0.0],
                viewport_origin: [0.0, 0.0, 0.0, 0.0],
                viewport_size: [1.0, 1.0, 0.0, 0.0],
                camera_center_scale: [0.0, 0.0, 1.0, 0.0],
            }),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let scene_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("datum-gui-render-scene-bg"),
            layout: &scene_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: scene_uniform_buffer.as_entire_binding(),
            }],
        });
        // P2.2a: independent uniform + bind group for the companion schematic pass
        // (same layout/pipeline as the board world pass, distinct backing buffer).
        let schematic_scene_uniform_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("datum-gui-render-schematic-scene-uniform-buffer"),
                contents: bytemuck::bytes_of(&SceneUniform {
                    resolution: [1.0, 1.0, 0.0, 0.0],
                    viewport_origin: [0.0, 0.0, 0.0, 0.0],
                    viewport_size: [1.0, 1.0, 0.0, 0.0],
                    camera_center_scale: [0.0, 0.0, 1.0, 0.0],
                }),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });
        let schematic_scene_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("datum-gui-render-schematic-scene-bg"),
            layout: &scene_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: schematic_scene_uniform_buffer.as_entire_binding(),
            }],
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("datum-gui-render-pipeline-layout"),
            bind_group_layouts: &[&uniform_bind_group_layout],
            immediate_size: 0,
        });
        let world_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("datum-gui-render-world-pipeline-layout"),
                bind_group_layouts: &[&scene_bind_group_layout],
                immediate_size: 0,
            });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("datum-gui-render-pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::layout()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: msaa_samples,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview_mask: None,
            cache: None,
        });
        let world_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("datum-gui-render-world-pipeline"),
            layout: Some(&world_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &world_shader,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::layout()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &world_shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: msaa_samples,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview_mask: None,
            cache: None,
        });
        let world_stroke_pipeline =
            create_world_stroke_pipeline(device, &world_pipeline_layout, format, msaa_samples);
        let terminal_graphics = terminal_graphics::TerminalGraphicsRenderer::new(
            device,
            &uniform_bind_group_layout,
            format,
            msaa_samples,
        );
        let mut font_system = FontSystem::new();
        load_datum_fonts(&mut font_system);
        let swash_cache = SwashCache::new();
        let cache = Cache::new(device);
        let viewport = Viewport::new(device, &cache);
        let mut atlas = TextAtlas::new(device, queue, &cache, format);
        let text_renderer = TextRenderer::new(
            &mut atlas,
            device,
            wgpu::MultisampleState {
                count: msaa_samples,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            None,
        );
        let menu_overlay_text_renderer = TextRenderer::new(
            &mut atlas,
            device,
            wgpu::MultisampleState {
                count: msaa_samples,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            None,
        );
        Self {
            pipeline,
            world_pipeline,
            world_stroke_pipeline,
            terminal_graphics,
            uniform_bind_group,
            uniform_buffer,
            scene_bind_group,
            scene_bind_group_layout,
            surface_scene_uniforms: Vec::new(),
            surface_world_bundles: Vec::new(),
            surface_grid_vertex_buffer: None,
            surface_grid_vertex_capacity: 0,
            schematic_scene_bind_group,
            schematic_world_vertex_buffer: None,
            schematic_world_vertex_capacity: 0,
            schematic_world_vertex_source: None,
            schematic_world_stroke_buffer: None,
            schematic_world_stroke_capacity: 0,
            schematic_world_stroke_source_ptr: 0,
            schematic_world_stroke_source_len: 0,
            schematic_underlay_vertex_buffer: None,
            schematic_underlay_vertex_capacity: 0,
            schematic_overlay_vertex_buffer: None,
            schematic_overlay_vertex_capacity: 0,
            font_system,
            swash_cache,
            viewport,
            atlas,
            text_renderer,
            menu_overlay_text_renderer,
            text_buffer_cache: Vec::new(),
            text_buffer_frame: 0,
            last_text_prepare_signature: None,
            panel_vertex_buffer: None,
            panel_vertex_capacity: 0,
            viewport_underlay_vertex_buffer: None,
            viewport_underlay_vertex_capacity: 0,
            viewport_overlay_vertex_buffer: None,
            viewport_overlay_vertex_capacity: 0,
            board_interaction_vertex_buffer: None,
            board_interaction_vertex_capacity: 0,
            console_gpu: gpu_console::ConsoleGpuResources::default(),
            menu_overlay_vertex_buffer: None,
            menu_overlay_vertex_capacity: 0,
            world_vertex_buffer: None,
            world_vertex_capacity: 0,
            world_vertex_source: None,
            world_stroke_buffer: None,
            world_stroke_capacity: 0,
            world_stroke_source_ptr: 0,
            world_stroke_source_len: 0,
            msaa_view: None,
            msaa_size: (0, 0),
            msaa_format: format,
            msaa_samples,
        }
    }
}
