//! Bounded GPU texture projection for TerminalCore graphics snapshots.

use std::collections::BTreeSet;

use wgpu::util::DeviceExt;

use super::PreparedTerminalGraphic;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct TerminalGraphicVertex {
    position: [f32; 2],
    uv: [f32; 2],
}

impl TerminalGraphicVertex {
    const fn layout<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: 8,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct TerminalGraphicTextureKey {
    id: u64,
    width: u32,
    height: u32,
    pixels_address: usize,
}

struct CachedTerminalGraphicTexture {
    key: TerminalGraphicTextureKey,
    _texture: wgpu::Texture,
    bind_group: wgpu::BindGroup,
}

struct TerminalGraphicDraw {
    texture_key: TerminalGraphicTextureKey,
    vertex_buffer: wgpu::Buffer,
    clip: (u32, u32, u32, u32),
    foreground: bool,
}

pub(super) struct TerminalGraphicsRenderer {
    pipeline: wgpu::RenderPipeline,
    texture_layout: wgpu::BindGroupLayout,
    textures: Vec<CachedTerminalGraphicTexture>,
    draws: Vec<TerminalGraphicDraw>,
}

impl TerminalGraphicsRenderer {
    pub(super) fn new(
        device: &wgpu::Device,
        screen_layout: &wgpu::BindGroupLayout,
        format: wgpu::TextureFormat,
        samples: u32,
    ) -> Self {
        let texture_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("datum-terminal-graphic-texture-layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("datum-terminal-graphic-pipeline-layout"),
            bind_group_layouts: &[screen_layout, &texture_layout],
            immediate_size: 0,
        });
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("datum-terminal-graphic-shader"),
            source: wgpu::ShaderSource::Wgsl(
                r#"
struct ScreenUniform { resolution: vec2<f32>, _pad: vec2<f32> };
@group(0) @binding(0) var<uniform> screen: ScreenUniform;
@group(1) @binding(0) var image: texture_2d<f32>;
@group(1) @binding(1) var image_sampler: sampler;

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,
};
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    output.position = vec4<f32>(
        (input.position.x / screen.resolution.x) * 2.0 - 1.0,
        1.0 - (input.position.y / screen.resolution.y) * 2.0,
        0.0,
        1.0,
    );
    output.uv = input.uv;
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(image, image_sampler, input.uv);
}
"#
                .into(),
            ),
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("datum-terminal-graphic-pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[TerminalGraphicVertex::layout()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: samples,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview_mask: None,
            cache: None,
        });
        Self {
            pipeline,
            texture_layout,
            textures: Vec::new(),
            draws: Vec::new(),
        }
    }

    pub(super) fn sync(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        prepared: &[PreparedTerminalGraphic],
        surface_width: u32,
        surface_height: u32,
    ) {
        self.draws.clear();
        let live_keys = prepared.iter().map(texture_key).collect::<BTreeSet<_>>();
        self.textures.retain(|entry| live_keys.contains(&entry.key));
        for graphic in prepared {
            let placement = graphic.graphic.placement();
            if placement.width() == 0 || placement.height() == 0 || placement.pixels().is_empty() {
                continue;
            }
            let key = texture_key(graphic);
            if !self.textures.iter().any(|entry| entry.key == key) {
                self.textures.push(create_texture(
                    device,
                    queue,
                    &self.texture_layout,
                    graphic,
                    key,
                ));
            }
            let vertices = graphic_vertices(graphic);
            let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("datum-terminal-graphic-vertices"),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });
            let clip_x = graphic.clip.x.max(0.0).floor() as u32;
            let clip_y = graphic.clip.y.max(0.0).floor() as u32;
            let clip_right = (graphic.clip.x + graphic.clip.width)
                .min(surface_width as f32)
                .ceil() as u32;
            let clip_bottom = (graphic.clip.y + graphic.clip.height)
                .min(surface_height as f32)
                .ceil() as u32;
            if clip_right > clip_x && clip_bottom > clip_y {
                self.draws.push(TerminalGraphicDraw {
                    texture_key: key,
                    vertex_buffer,
                    clip: (clip_x, clip_y, clip_right - clip_x, clip_bottom - clip_y),
                    foreground: placement.z_index() >= 0,
                });
            }
        }
    }

    pub(super) fn encode_layer(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        msaa_view: &wgpu::TextureView,
        target: &wgpu::TextureView,
        screen_bind_group: &wgpu::BindGroup,
        foreground: bool,
    ) {
        if !self.draws.iter().any(|draw| draw.foreground == foreground) {
            return;
        }
        let label = if foreground {
            "datum-terminal-foreground-graphics-pass"
        } else {
            "datum-terminal-background-graphics-pass"
        };
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some(label),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: msaa_view,
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
        self.draw(&mut pass, screen_bind_group, foreground);
    }

    fn draw<'pass>(
        &'pass self,
        pass: &mut wgpu::RenderPass<'pass>,
        screen_bind_group: &'pass wgpu::BindGroup,
        foreground: bool,
    ) {
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, screen_bind_group, &[]);
        for draw in self
            .draws
            .iter()
            .filter(|draw| draw.foreground == foreground)
        {
            let Some(texture) = self
                .textures
                .iter()
                .find(|entry| entry.key == draw.texture_key)
            else {
                continue;
            };
            pass.set_scissor_rect(draw.clip.0, draw.clip.1, draw.clip.2, draw.clip.3);
            pass.set_bind_group(1, &texture.bind_group, &[]);
            pass.set_vertex_buffer(0, draw.vertex_buffer.slice(..));
            pass.draw(0..6, 0..1);
        }
    }
}

fn texture_key(graphic: &PreparedTerminalGraphic) -> TerminalGraphicTextureKey {
    let placement = graphic.graphic.placement();
    TerminalGraphicTextureKey {
        id: placement.id().get(),
        width: placement.width(),
        height: placement.height(),
        pixels_address: placement.pixels().as_ptr() as usize,
    }
}

impl super::Renderer {
    pub(super) fn sync_terminal_graphics(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        prepared: &super::PreparedScene,
        width: u32,
        height: u32,
    ) {
        self.terminal_graphics
            .sync(device, queue, prepared.terminal_graphics(), width, height);
    }

    pub(super) fn encode_terminal_graphics(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        msaa_view: &wgpu::TextureView,
        target: &wgpu::TextureView,
        foreground: bool,
    ) {
        self.terminal_graphics.encode_layer(
            encoder,
            msaa_view,
            target,
            &self.uniform_bind_group,
            foreground,
        );
    }
}

fn create_texture(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    layout: &wgpu::BindGroupLayout,
    graphic: &PreparedTerminalGraphic,
    key: TerminalGraphicTextureKey,
) -> CachedTerminalGraphicTexture {
    let placement = graphic.graphic.placement();
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("datum-terminal-graphic-texture"),
        size: wgpu::Extent3d {
            width: key.width,
            height: key.height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    let bytes = placement
        .pixels()
        .iter()
        .flat_map(|pixel| [pixel.red, pixel.green, pixel.blue, pixel.alpha])
        .collect::<Vec<_>>();
    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        &bytes,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(key.width * 4),
            rows_per_image: Some(key.height),
        },
        wgpu::Extent3d {
            width: key.width,
            height: key.height,
            depth_or_array_layers: 1,
        },
    );
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("datum-terminal-graphic-sampler"),
        mag_filter: wgpu::FilterMode::Nearest,
        min_filter: wgpu::FilterMode::Nearest,
        ..Default::default()
    });
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("datum-terminal-graphic-bind-group"),
        layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&sampler),
            },
        ],
    });
    CachedTerminalGraphicTexture {
        key,
        _texture: texture,
        bind_group,
    }
}

fn graphic_vertices(graphic: &PreparedTerminalGraphic) -> [TerminalGraphicVertex; 6] {
    let placement = graphic.graphic.placement();
    let source = placement.source();
    let source_width = if source.width == 0 {
        placement.width().saturating_sub(source.x)
    } else {
        source.width.min(placement.width().saturating_sub(source.x))
    };
    let source_height = if source.height == 0 {
        placement.height().saturating_sub(source.y)
    } else {
        source
            .height
            .min(placement.height().saturating_sub(source.y))
    };
    let u0 = source.x as f32 / placement.width() as f32;
    let v0 = source.y as f32 / placement.height() as f32;
    let u1 = (source.x + source_width) as f32 / placement.width() as f32;
    let v1 = (source.y + source_height) as f32 / placement.height() as f32;
    let x0 = graphic.rect.x;
    let y0 = graphic.rect.y;
    let x1 = x0 + graphic.rect.width;
    let y1 = y0 + graphic.rect.height;
    [
        vertex(x0, y0, u0, v0),
        vertex(x1, y0, u1, v0),
        vertex(x1, y1, u1, v1),
        vertex(x0, y0, u0, v0),
        vertex(x1, y1, u1, v1),
        vertex(x0, y1, u0, v1),
    ]
}

const fn vertex(x: f32, y: f32, u: f32, v: f32) -> TerminalGraphicVertex {
    TerminalGraphicVertex {
        position: [x, y],
        uv: [u, v],
    }
}
