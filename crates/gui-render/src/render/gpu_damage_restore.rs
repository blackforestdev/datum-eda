//! Exact UNORM sample restoration. No resolve, filtering, blend or gamma conversion.
pub(crate) struct Restoration {
    pipeline: wgpu::RenderPipeline,
    layout: wgpu::BindGroupLayout,
}
impl Restoration {
    pub(crate) async fn admit(
        device: &wgpu::Device,
        adapter: &wgpu::Adapter,
        format: wgpu::TextureFormat,
        samples: u32,
    ) -> Option<Self> {
        let alias = format.remove_srgb_suffix();
        if samples != 8
            || !matches!(
                alias,
                wgpu::TextureFormat::Rgba8Unorm | wgpu::TextureFormat::Bgra8Unorm
            )
            || !device
                .features()
                .contains(wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES)
            || !adapter.get_downlevel_capabilities().flags.contains(
                wgpu::DownlevelFlags::VIEW_FORMATS | wgpu::DownlevelFlags::MULTISAMPLED_SHADING,
            )
        {
            return None;
        }
        for candidate in [format, alias] {
            let features = adapter.get_texture_format_features(candidate);
            if !features
                .flags
                .contains(wgpu::TextureFormatFeatureFlags::MULTISAMPLE_X8)
                || !features.allowed_usages.contains(
                    wgpu::TextureUsages::RENDER_ATTACHMENT
                        | wgpu::TextureUsages::TEXTURE_BINDING
                        | wgpu::TextureUsages::COPY_SRC
                        | wgpu::TextureUsages::COPY_DST,
                )
            {
                return None;
            }
        }
        Self::create(device, alias, include_str!("gpu_damage_restore.wgsl")).await
    }

    async fn create(
        device: &wgpu::Device,
        alias: wgpu::TextureFormat,
        source: &str,
    ) -> Option<Self> {
        let error_scope = device.push_error_scope(wgpu::ErrorFilter::Validation);
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("datum-sample-restoration-layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: false },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: true,
                },
                count: None,
            }],
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("datum-sample-restoration-pipeline-layout"),
            bind_group_layouts: &[&layout],
            immediate_size: 0,
        });
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("datum-exact-sample-restoration"),
            source: wgpu::ShaderSource::Wgsl(source.into()),
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("datum-exact-sample-restoration"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vertex"),
                compilation_options: Default::default(),
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fragment"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: alias,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: Default::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 8,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview_mask: None,
            cache: None,
        });
        // Initialization only: refusal retains the full graph. Never wait or
        // manufacture admission while processing a frame.
        if error_scope.pop().await.is_some() {
            return None;
        }
        Some(Self { pipeline, layout })
    }

    pub(crate) fn encode_regional(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        source: &wgpu::TextureView,
        destination: &wgpu::TextureView,
        plan: &super::regional::Plan,
        timestamps: Option<wgpu::RenderPassTimestampWrites<'_>>,
    ) {
        let binding = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("datum-regional-sample-source"),
            layout: &self.layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(source),
            }],
        });
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("datum-regional-sample-restoration"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: destination,
                resolve_target: None,
                depth_slice: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                    store: wgpu::StoreOp::Store,
                },
            })],
            timestamp_writes: timestamps,
            ..Default::default()
        });
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &binding, &[]);
        for tile in plan.tiles() {
            pass.set_scissor_rect(tile.atlas[0], tile.atlas[1], tile.extent[0], tile.extent[1]);
            // The source ID is captured by the command buffer. No mutable GPU
            // mapping, unmeasured upload, or per-tile buffer allocation is needed.
            pass.draw(0..3, tile.id..tile.id + 1);
        }
    }

    #[cfg(all(test, feature = "visual", target_os = "linux"))]
    pub(crate) fn encode(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        source: &wgpu::TextureView,
        destination: &wgpu::TextureView,
        rectangles: &[[u32; 4]],
        timestamps: Option<wgpu::RenderPassTimestampWrites<'_>>,
    ) {
        let binding = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("datum-sample-restoration-source"),
            layout: &self.layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(source),
            }],
        });
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("datum-damaged-sample-restoration"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: destination,
                resolve_target: None,
                depth_slice: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            timestamp_writes: timestamps,
            ..Default::default()
        });
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &binding, &[]);
        for &[left, top, right, bottom] in rectangles {
            pass.set_scissor_rect(left, top, right - left, bottom - top);
            pass.draw(0..3, 0..1);
        }
    }
}

#[cfg(all(test, feature = "visual", target_os = "linux"))]
#[path = "gpu_damage_sample_tests.rs"]
pub(crate) mod sample_tests;
