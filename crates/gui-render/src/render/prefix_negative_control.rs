//! Deliberately wrong graphs for exact-output oracle conformance, test binaries only.
//! Their scratch resources are not resource/performance qualification evidence.
use super::PrefixImages;
#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub(crate) enum Fault {
    #[default]
    None,
    StaleKey,
    MissingCopy,
    PrematureResolve,
}
thread_local! {
    static NEXT: std::cell::Cell<Fault> = const { std::cell::Cell::new(Fault::None) };
}
pub(crate) fn set(fault: Fault) {
    NEXT.set(fault);
}
pub(crate) fn take() -> Fault {
    NEXT.replace(Fault::None)
}
impl Fault {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn copy(
        self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        images: &PrefixImages,
        working: &wgpu::TextureView,
        width: u32,
        height: u32,
        format: wgpu::TextureFormat,
    ) {
        match self {
            Self::None | Self::StaleKey => images.copy(encoder),
            Self::MissingCopy => {
                // Deterministically initialize B; never rely on discarded GPU data.
                let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("negative-missing-copy"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: working,
                        resolve_target: None,
                        depth_slice: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::RED),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    ..Default::default()
                });
            }
            Self::PrematureResolve => {
                premature(device, encoder, images, working, width, height, format)
            }
        }
    }
}

fn premature(
    device: &wgpu::Device,
    encoder: &mut wgpu::CommandEncoder,
    images: &PrefixImages,
    working: &wgpu::TextureView,
    width: u32,
    height: u32,
    format: wgpu::TextureFormat,
) {
    let resolved = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("negative-premature-resolve"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    let view = resolved.create_view(&Default::default());
    {
        let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("negative-resolve-prefix-before-suffix"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: images.prefix_view(),
                resolve_target: Some(&view),
                depth_slice: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            ..Default::default()
        });
    }
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("negative-expand-resolved-pixel-to-all-samples"),
        source: wgpu::ShaderSource::Wgsl(
            r#"
            @group(0) @binding(0) var image: texture_2d<f32>;
            @vertex fn vs(@builtin(vertex_index) index: u32) -> @builtin(position) vec4<f32> {
                let positions = array<vec2<f32>, 3>(vec2(-1., -1.), vec2(3., -1.), vec2(-1., 3.));
                return vec4(positions[index], 0., 1.);
            }
            @fragment fn fs(@builtin(position) p: vec4<f32>) -> @location(0) vec4<f32> {
                return textureLoad(image, vec2<i32>(p.xy), 0);
            }
        "#
            .into(),
        ),
    });
    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("negative-premature-resolve-expansion"),
        layout: None,
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs"),
            compilation_options: Default::default(),
            buffers: &[],
        },
        primitive: Default::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState {
            count: 8,
            ..Default::default()
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs"),
            compilation_options: Default::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        multiview_mask: None,
        cache: None,
    });
    let binding = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("negative-resolved-prefix-input"),
        layout: &pipeline.get_bind_group_layout(0),
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: wgpu::BindingResource::TextureView(&view),
        }],
    });
    let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("negative-lose-per-sample-coverage"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: working,
            resolve_target: None,
            depth_slice: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(wgpu::Color::RED),
                store: wgpu::StoreOp::Store,
            },
        })],
        ..Default::default()
    });
    pass.set_pipeline(&pipeline);
    pass.set_bind_group(0, &binding, &[]);
    pass.draw(0..3, 0..1);
}
