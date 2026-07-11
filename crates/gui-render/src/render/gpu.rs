pub struct Renderer {
    pipeline: wgpu::RenderPipeline,
    world_pipeline: wgpu::RenderPipeline,
    uniform_bind_group: wgpu::BindGroup,
    uniform_buffer: wgpu::Buffer,
    scene_bind_group: wgpu::BindGroup,
    scene_uniform_buffer: wgpu::Buffer,
    // P2.2a: the static companion schematic world pass needs its OWN uniform +
    // bind group (a distinct buffer from the board's), because both uniform writes
    // land at submit time — sharing one buffer would make both passes read the
    // last-written camera/viewport. The board buffers above are untouched.
    schematic_scene_bind_group: wgpu::BindGroup,
    schematic_scene_uniform_buffer: wgpu::Buffer,
    schematic_world_vertex_buffer: Option<wgpu::Buffer>,
    schematic_world_vertex_capacity: usize,
    // Slice S1b: immediate screen-space schematic grid (drawn with the screen
    // pipeline, scissored to the schematic pane) — never a retained world buffer.
    schematic_underlay_vertex_buffer: Option<wgpu::Buffer>,
    schematic_underlay_vertex_capacity: usize,
    // S4 hover/cursor chrome is a post-world overlay, never part of the grid.
    schematic_overlay_vertex_buffer: Option<wgpu::Buffer>,
    schematic_overlay_vertex_capacity: usize,
    font_system: FontSystem,
    swash_cache: SwashCache,
    viewport: Viewport,
    atlas: TextAtlas,
    text_renderer: TextRenderer,
    // Dedicated renderer for the open menu dropdown's own text, prepared and
    // rendered in a FINAL pass on top of the dropdown card so it is never
    // occluded and never disturbs the main text renderer's prepared state.
    menu_overlay_text_renderer: TextRenderer,
    text_buffer_cache: Vec<CachedTextBuffer>,
    last_text_prepare_signature: Option<TextPrepareSignature>,
    panel_vertex_buffer: Option<wgpu::Buffer>,
    panel_vertex_capacity: usize,
    viewport_underlay_vertex_buffer: Option<wgpu::Buffer>,
    viewport_underlay_vertex_capacity: usize,
    viewport_overlay_vertex_buffer: Option<wgpu::Buffer>,
    viewport_overlay_vertex_capacity: usize,
    board_interaction_vertex_buffer: Option<wgpu::Buffer>,
    board_interaction_vertex_capacity: usize,
    menu_overlay_vertex_buffer: Option<wgpu::Buffer>,
    menu_overlay_vertex_capacity: usize,
    world_vertex_buffer: Option<wgpu::Buffer>,
    world_vertex_capacity: usize,
    world_vertex_source_ptr: usize,
    world_vertex_source_len: usize,
    msaa_view: Option<wgpu::TextureView>,
    msaa_size: (u32, u32),
    msaa_format: wgpu::TextureFormat,
    msaa_samples: u32,
}

// Real child modules keep cache and per-frame upload inventories independently governed.
#[path = "text_buffer_cache.rs"]
mod text_buffer_cache;
#[path = "gpu_vertex_upload.rs"]
mod gpu_vertex_upload;

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
            uniform_bind_group,
            uniform_buffer,
            scene_bind_group,
            scene_uniform_buffer,
            schematic_scene_bind_group,
            schematic_scene_uniform_buffer,
            schematic_world_vertex_buffer: None,
            schematic_world_vertex_capacity: 0,
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
            last_text_prepare_signature: None,
            panel_vertex_buffer: None,
            panel_vertex_capacity: 0,
            viewport_underlay_vertex_buffer: None,
            viewport_underlay_vertex_capacity: 0,
            viewport_overlay_vertex_buffer: None,
            viewport_overlay_vertex_capacity: 0,
            board_interaction_vertex_buffer: None,
            board_interaction_vertex_capacity: 0,
            menu_overlay_vertex_buffer: None,
            menu_overlay_vertex_capacity: 0,
            world_vertex_buffer: None,
            world_vertex_capacity: 0,
            world_vertex_source_ptr: 0,
            world_vertex_source_len: 0,
            msaa_view: None,
            msaa_size: (0, 0),
            msaa_format: format,
            msaa_samples,
        }
    }

    fn ensure_msaa(
        &mut self,
        device: &wgpu::Device,
        width: u32,
        height: u32,
    ) -> &wgpu::TextureView {
        if self.msaa_size != (width, height) || self.msaa_view.is_none() {
            let texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("datum-gui-render-msaa"),
                size: wgpu::Extent3d {
                    width: width.max(1),
                    height: height.max(1),
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: self.msaa_samples,
                dimension: wgpu::TextureDimension::D2,
                format: self.msaa_format,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            });
            self.msaa_view = Some(texture.create_view(&wgpu::TextureViewDescriptor::default()));
            self.msaa_size = (width, height);
        }
        self.msaa_view.as_ref().unwrap()
    }

    // Render helper threads many quad/text-run/hit-region sinks.
    #[allow(clippy::too_many_arguments)]
    pub fn render(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        target: &wgpu::TextureView,
        prepared: &PreparedScene,
        retained: &RetainedScene,
        schematic_retained: Option<&RetainedScene>,
        width: u32,
        height: u32,
    ) -> anyhow::Result<()> {
        let render_started = std::time::Instant::now();
        let panel_vertices = prepared.panel_vertices();
        let viewport_underlay_vertices = prepared.viewport_underlay_vertices();
        let viewport_overlay_vertices = prepared.viewport_overlay_vertices();
        let board_interaction_vertices = prepared.board_interaction_vertices();
        let menu_overlay_vertices = prepared.menu_overlay_vertices();
        let world_vertices = retained.world_vertices();
        let visible_world_ranges = prepared.visible_world_ranges();
        let board_field = inset_rect(prepared.scene_viewport, 10.0, 10.0, 10.0, 10.0);
        let projection = Projection::new(board_field, &prepared.scene_bounds, prepared.camera);
        // P2.2a: resolve the additive companion schematic pass. Active only when
        // the layout carries a Schematic pane (`schematic_scene_viewport`) AND a
        // projected schematic RetainedScene was threaded in with geometry to draw.
        let schematic_pass = match (prepared.schematic_scene_viewport, schematic_retained) {
            (Some(scene_viewport), Some(sr)) if !sr.world_vertices().is_empty() => {
                let field = inset_rect(scene_viewport, 10.0, 10.0, 10.0, 10.0);
                let proj =
                    Projection::new(field, &prepared.schematic_bounds, prepared.schematic_camera);
                Some((scene_viewport, field, proj, sr))
            }
            _ => None,
        };
        // S4 grid and interaction overlays remain immediate screen-space geometry;
        // offscreen captures supply neither cursor nor hover quads.
        let schematic_underlay_vertices = prepared.schematic_underlay_vertices();
        let schematic_overlay_vertices = prepared.schematic_overlay_vertices();
        queue.write_buffer(
            &self.uniform_buffer,
            0,
            bytemuck::bytes_of(&ScreenUniform {
                resolution: [width as f32, height as f32],
                _pad: [0.0, 0.0],
            }),
        );
        queue.write_buffer(
            &self.scene_uniform_buffer,
            0,
            bytemuck::bytes_of(&SceneUniform {
                resolution: [width as f32, height as f32, 0.0, 0.0],
                viewport_origin: [board_field.x, board_field.y, 0.0, 0.0],
                viewport_size: [board_field.width, board_field.height, 0.0, 0.0],
                camera_center_scale: [
                    prepared.camera.center_x_nm,
                    prepared.camera.center_y_nm,
                    projection.scale,
                    0.0,
                ],
            }),
        );
        // Companion schematic uniform (distinct backing buffer — see field docs).
        if let Some((_, field, proj, _)) = schematic_pass.as_ref() {
            queue.write_buffer(
                &self.schematic_scene_uniform_buffer,
                0,
                bytemuck::bytes_of(&SceneUniform {
                    resolution: [width as f32, height as f32, 0.0, 0.0],
                    viewport_origin: [field.x, field.y, 0.0, 0.0],
                    viewport_size: [field.width, field.height, 0.0, 0.0],
                    camera_center_scale: [
                        prepared.schematic_camera.center_x_nm,
                        prepared.schematic_camera.center_y_nm,
                        proj.scale,
                        0.0,
                    ],
                }),
            );
        }
        let upload_started = std::time::Instant::now();
        self.upload_frame_vertices(
            device,
            queue,
            panel_vertices,
            viewport_underlay_vertices,
            viewport_overlay_vertices,
            board_interaction_vertices,
            menu_overlay_vertices,
            world_vertices,
            schematic_pass.as_ref().map(|(_, _, _, scene)| *scene),
            schematic_underlay_vertices,
            schematic_overlay_vertices,
        );
        let upload_elapsed = upload_started.elapsed();
        let encode_started = std::time::Instant::now();
        let msaa_view = self.ensure_msaa(device, width, height).clone();
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("datum-gui-render-encoder"),
        });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("datum-gui-render-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &msaa_view,
                    resolve_target: Some(target),
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: APP_BG[0] as f64,
                            g: APP_BG[1] as f64,
                            b: APP_BG[2] as f64,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
                multiview_mask: None,
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &self.uniform_bind_group, &[]);
            if !panel_vertices.is_empty() {
                pass.set_vertex_buffer(
                    0,
                    self.panel_vertex_buffer
                        .as_ref()
                        .expect("panel vertex buffer should exist")
                        .slice(..),
                );
                pass.draw(0..panel_vertices.len() as u32, 0..1);
            }
            if !viewport_underlay_vertices.is_empty() {
                pass.set_scissor_rect(
                    prepared.scene_viewport.x.max(0.0).floor() as u32,
                    prepared.scene_viewport.y.max(0.0).floor() as u32,
                    prepared.scene_viewport.width.max(1.0).ceil() as u32,
                    prepared.scene_viewport.height.max(1.0).ceil() as u32,
                );
                pass.set_vertex_buffer(
                    0,
                    self.viewport_underlay_vertex_buffer
                        .as_ref()
                        .expect("viewport underlay vertex buffer should exist")
                        .slice(..),
                );
                pass.draw(0..viewport_underlay_vertices.len() as u32, 0..1);
            }
            if !world_vertices.is_empty() && !visible_world_ranges.is_empty() {
                pass.set_pipeline(&self.world_pipeline);
                pass.set_bind_group(0, &self.scene_bind_group, &[]);
                pass.set_scissor_rect(
                    prepared.scene_viewport.x.max(0.0).floor() as u32,
                    prepared.scene_viewport.y.max(0.0).floor() as u32,
                    prepared.scene_viewport.width.max(1.0).ceil() as u32,
                    prepared.scene_viewport.height.max(1.0).ceil() as u32,
                );
                pass.set_vertex_buffer(
                    0,
                    self.world_vertex_buffer
                        .as_ref()
                        .expect("world vertex buffer should exist")
                        .slice(..),
                );
                for range in visible_world_ranges {
                    pass.draw(range.clone(), 0..1);
                }
                pass.set_pipeline(&self.pipeline);
                pass.set_bind_group(0, &self.uniform_bind_group, &[]);
            }
            // Slice S1b: immediate schematic grid. Drawn with the SCREEN pipeline
            // (active here — restored after the board world pass, or the default) so
            // its pixel-rect lines keep a fixed device-pixel weight at any schematic
            // zoom, scissored to the schematic pane's scene rect. Painted BEFORE the
            // schematic world pass below so wires/symbols sit on top of the grid,
            // preserving the old draw order the retained bake had.
            if !schematic_underlay_vertices.is_empty()
                && let Some((scene_viewport, _, _, _)) = schematic_pass.as_ref()
                && let Some(buffer) = self.schematic_underlay_vertex_buffer.as_ref()
            {
                pass.set_scissor_rect(
                    scene_viewport.x.max(0.0).floor() as u32,
                    scene_viewport.y.max(0.0).floor() as u32,
                    scene_viewport.width.max(1.0).ceil() as u32,
                    scene_viewport.height.max(1.0).ceil() as u32,
                );
                pass.set_vertex_buffer(0, buffer.slice(..));
                pass.draw(0..schematic_underlay_vertices.len() as u32, 0..1);
            }
            // P2.2a: additive companion schematic world pass. Reuses the same world
            // pipeline, but with the schematic's own bind group (schematic camera +
            // viewport uniform) and vertex buffer, scissored to the schematic pane's
            // scene rect so it can never touch pane A. The board pass above is
            // completely unchanged. Restores the screen pipeline afterward for the
            // overlay/menu passes.
            if let Some((scene_viewport, _, _, sr)) = schematic_pass.as_ref() {
                let ranges = sr.all_world_ranges();
                if !ranges.is_empty()
                    && let Some(buffer) = self.schematic_world_vertex_buffer.as_ref()
                {
                    pass.set_pipeline(&self.world_pipeline);
                    pass.set_bind_group(0, &self.schematic_scene_bind_group, &[]);
                    pass.set_scissor_rect(
                        scene_viewport.x.max(0.0).floor() as u32,
                        scene_viewport.y.max(0.0).floor() as u32,
                        scene_viewport.width.max(1.0).ceil() as u32,
                        scene_viewport.height.max(1.0).ceil() as u32,
                    );
                    pass.set_vertex_buffer(0, buffer.slice(..));
                    for range in ranges {
                        pass.draw(range, 0..1);
                    }
                    pass.set_pipeline(&self.pipeline);
                    pass.set_bind_group(0, &self.uniform_bind_group, &[]);
                }
            }
            // Interaction chrome is intentionally composited after schematic world
            // geometry, matching the board overlay and preventing filled symbols
            // from obscuring hover/cursor feedback.
            if !schematic_overlay_vertices.is_empty()
                && let Some((scene_viewport, _, _, _)) = schematic_pass.as_ref()
                && let Some(buffer) = self.schematic_overlay_vertex_buffer.as_ref()
            {
                pass.set_scissor_rect(
                    scene_viewport.x.max(0.0).floor() as u32,
                    scene_viewport.y.max(0.0).floor() as u32,
                    scene_viewport.width.max(1.0).ceil() as u32,
                    scene_viewport.height.max(1.0).ceil() as u32,
                );
                pass.set_vertex_buffer(0, buffer.slice(..));
                pass.draw(0..schematic_overlay_vertices.len() as u32, 0..1);
            }
            if !viewport_overlay_vertices.is_empty() {
                pass.set_scissor_rect(
                    prepared.scene_viewport.x.max(0.0).floor() as u32,
                    prepared.scene_viewport.y.max(0.0).floor() as u32,
                    prepared.scene_viewport.width.max(1.0).ceil() as u32,
                    prepared.scene_viewport.height.max(1.0).ceil() as u32,
                );
                pass.set_vertex_buffer(
                    0,
                    self.viewport_overlay_vertex_buffer
                        .as_ref()
                        .expect("viewport overlay vertex buffer should exist")
                        .slice(..),
                );
                pass.draw(0..viewport_overlay_vertices.len() as u32, 0..1);
            }
            if !board_interaction_vertices.is_empty() {
                pass.set_scissor_rect(
                    prepared.scene_viewport.x.max(0.0).floor() as u32,
                    prepared.scene_viewport.y.max(0.0).floor() as u32,
                    prepared.scene_viewport.width.max(1.0).ceil() as u32,
                    prepared.scene_viewport.height.max(1.0).ceil() as u32,
                );
                pass.set_vertex_buffer(
                    0,
                    self.board_interaction_vertex_buffer
                        .as_ref()
                        .expect("board interaction vertex buffer should exist")
                        .slice(..),
                );
                pass.draw(0..board_interaction_vertices.len() as u32, 0..1);
            }
            // NOTE: the menu dropdown card is intentionally NOT drawn here. It is
            // composited AFTER the main text pass (below) so it occludes not only
            // the work-pane quads but every underlying text_run too; its own text
            // then draws in a final pass on top of the card.
        }
        let encode_elapsed = encode_started.elapsed();
        self.viewport.update(queue, Resolution { width, height });
        let text_prepare_started = std::time::Instant::now();
        let (text_buffer_indices, text_cache_stats) =
            self.cached_text_buffer_indices(&prepared.text_runs, width, height);
        let text_signature =
            text_prepare_signature(&text_buffer_indices, &prepared.text_runs, width, height);
        let skipped_text_prepare = self
            .last_text_prepare_signature
            .as_ref()
            .is_some_and(|previous| previous == &text_signature)
            && text_cache_stats.misses == 0;
        if !skipped_text_prepare {
            let prepare_result = self.text_renderer.prepare(
                device,
                queue,
                &mut self.font_system,
                &mut self.atlas,
                &self.viewport,
                build_text_areas(
                    &self.text_buffer_cache,
                    &text_buffer_indices,
                    &prepared.text_runs,
                ),
                &mut self.swash_cache,
            );
            if let Err(initial_error) = prepare_result {
                // Keep the glyph atlas warm during normal interaction. Trim only
                // when prepare reports pressure, then retry with the same semantic
                // text areas. This preserves the DOA2526 atlas-safety behavior
                // without forcing avoidable re-rasterization on every selection.
                self.atlas.trim();
                self.text_renderer
                    .prepare(
                        device,
                        queue,
                        &mut self.font_system,
                        &mut self.atlas,
                        &self.viewport,
                        build_text_areas(
                            &self.text_buffer_cache,
                            &text_buffer_indices,
                            &prepared.text_runs,
                        ),
                        &mut self.swash_cache,
                    )
                    .map_err(|retry_error| {
                        anyhow::anyhow!(
                            "prepare GUI text after atlas trim: {retry_error}; initial: {initial_error}"
                        )
                    })?;
            }
            self.last_text_prepare_signature = Some(text_signature);
        }
        let text_prepare_elapsed = text_prepare_started.elapsed();
        let text_encode_started = std::time::Instant::now();
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("datum-gui-text-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &msaa_view,
                    resolve_target: Some(target),
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
                multiview_mask: None,
            });
            self.text_renderer
                .render(&self.atlas, &self.viewport, &mut pass)
                .map_err(|error| anyhow::anyhow!("render GUI text: {error}"))?;
        }
        let text_encode_elapsed = text_encode_started.elapsed();

        // === Menu dropdown, composited LAST (after the main text pass) ===
        // The card is drawn here — not with the base quads — so it occludes the
        // work-pane content AND every underlying text_run below it. The dropdown's
        // OWN text then renders in a final pass on top of the card, so it stays
        // crisp while the bleed text is hidden. Only present when a menu is open.
        if !menu_overlay_vertices.is_empty() {
            // Pass C: the dropdown card background/rows. Base pipeline + screen
            // uniform; full-window scissor so the drop below the menu bar is not
            // re-clipped to the scene viewport.
            {
                let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("datum-gui-menu-overlay-pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &msaa_view,
                        resolve_target: Some(target),
                        depth_slice: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    occlusion_query_set: None,
                    timestamp_writes: None,
                    multiview_mask: None,
                });
                pass.set_pipeline(&self.pipeline);
                pass.set_bind_group(0, &self.uniform_bind_group, &[]);
                pass.set_scissor_rect(0, 0, width, height);
                pass.set_vertex_buffer(
                    0,
                    self.menu_overlay_vertex_buffer
                        .as_ref()
                        .expect("menu overlay vertex buffer should exist")
                        .slice(..),
                );
                pass.draw(0..menu_overlay_vertices.len() as u32, 0..1);
            }

            // Pass D: the dropdown's own text, on top of the card. Uses the
            // dedicated overlay text renderer so the main renderer's prepared
            // state/caching is untouched. The content-keyed text_buffer_cache is
            // shared, so overlay glyph buffers reuse the same atlas.
            let menu_overlay_text_runs = prepared.menu_overlay_text_runs();
            if !menu_overlay_text_runs.is_empty() {
                let (overlay_indices, _) =
                    self.cached_text_buffer_indices(menu_overlay_text_runs, width, height);
                let overlay_prepare = self.menu_overlay_text_renderer.prepare(
                    device,
                    queue,
                    &mut self.font_system,
                    &mut self.atlas,
                    &self.viewport,
                    build_text_areas(
                        &self.text_buffer_cache,
                        &overlay_indices,
                        menu_overlay_text_runs,
                    ),
                    &mut self.swash_cache,
                );
                if let Err(initial_error) = overlay_prepare {
                    self.atlas.trim();
                    self.menu_overlay_text_renderer
                        .prepare(
                            device,
                            queue,
                            &mut self.font_system,
                            &mut self.atlas,
                            &self.viewport,
                            build_text_areas(
                                &self.text_buffer_cache,
                                &overlay_indices,
                                menu_overlay_text_runs,
                            ),
                            &mut self.swash_cache,
                        )
                        .map_err(|retry_error| {
                            anyhow::anyhow!(
                                "prepare menu overlay text after atlas trim: {retry_error}; initial: {initial_error}"
                            )
                        })?;
                }
                {
                    let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("datum-gui-menu-overlay-text-pass"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &msaa_view,
                            resolve_target: Some(target),
                            depth_slice: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Load,
                                store: wgpu::StoreOp::Store,
                            },
                        })],
                        depth_stencil_attachment: None,
                        occlusion_query_set: None,
                        timestamp_writes: None,
                        multiview_mask: None,
                    });
                    self.menu_overlay_text_renderer
                        .render(&self.atlas, &self.viewport, &mut pass)
                        .map_err(|error| anyhow::anyhow!("render menu overlay text: {error}"))?;
                }
            }
        }

        let submit_started = std::time::Instant::now();
        queue.submit([encoder.finish()]);
        let submit_elapsed = submit_started.elapsed();
        trace_render_timing(format!(
            "renderer total={}ms upload={}ms encode={}ms text_prepare={}ms text_encode={}ms submit={}ms vertices panel={} underlay={} world={} overlay={} text_runs={} text_cache={}/{} text_prepare_skipped={}",
            render_started.elapsed().as_millis(),
            upload_elapsed.as_millis(),
            encode_elapsed.as_millis(),
            text_prepare_elapsed.as_millis(),
            text_encode_elapsed.as_millis(),
            submit_elapsed.as_millis(),
            panel_vertices.len(),
            viewport_underlay_vertices.len(),
            world_vertices.len(),
            viewport_overlay_vertices.len(),
            prepared.text_runs.len(),
            text_cache_stats.hits,
            text_cache_stats.misses,
            skipped_text_prepare,
        ));
        Ok(())
    }
}
