//! Device-bound mask pipelines; ordinary worlds retain their existing pipelines.
use super::*;
pub(super) struct Pipelines {
    pub table_layout: wgpu::BindGroupLayout,
    pub image_layout: wgpu::BindGroupLayout,
    pub mask_quad: wgpu::RenderPipeline,
    pub mask_stroke: wgpu::RenderPipeline,
    pub lift: wgpu::RenderPipeline,
    pub cue: wgpu::RenderPipeline,
}
impl Pipelines {
    pub(super) fn new(
        device: &wgpu::Device,
        scene: &wgpu::BindGroupLayout,
        screen: &wgpu::BindGroupLayout,
        damage: &wgpu::BindGroupLayout,
        format: wgpu::TextureFormat,
        samples: u32,
    ) -> Self {
        let storage = |binding, visibility| wgpu::BindGroupLayoutEntry {
            binding,
            visibility,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: wgpu::BufferSize::new(32),
            },
            count: None,
        };
        let table_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("datum-selection-mask-table-layout"),
            entries: &[storage(0, wgpu::ShaderStages::VERTEX_FRAGMENT)],
        });
        let image_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("datum-selection-mask-image-layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: true,
                    },
                    count: None,
                },
                storage(1, wgpu::ShaderStages::FRAGMENT),
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: true,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(16),
                    },
                    count: None,
                },
            ],
        });
        let world = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("datum-selection-mask-world"),
            source: wgpu::ShaderSource::Wgsl(
                crate::renderer_state::damage::shader(include_str!("selection_mask.wgsl"), 3)
                    .into(),
            ),
        });
        let cue = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("datum-selection-mask-cue"),
            source: wgpu::ShaderSource::Wgsl(
                crate::renderer_state::damage::shader(include_str!("selection_mask_cue.wgsl"), 1)
                    .into(),
            ),
        });
        let mask_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("datum-selection-mask-world-layout"),
            bind_group_layouts: &[scene, &table_layout],
            immediate_size: 0,
        });
        let cue_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("datum-selection-mask-cue-layout"),
            bind_group_layouts: &[screen, damage, &image_layout],
            immediate_size: 0,
        });
        let build = |layout: &wgpu::PipelineLayout,
                     shader: &wgpu::ShaderModule,
                     vertex: &str,
                     fragment: &str,
                     buffers: &[wgpu::VertexBufferLayout<'_>],
                     target,
                     blend| {
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("datum-selection-mask-pipeline"),
                layout: Some(layout),
                vertex: wgpu::VertexState {
                    module: shader,
                    entry_point: Some(vertex),
                    buffers,
                    compilation_options: Default::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: shader,
                    entry_point: Some(fragment),
                    targets: &if fragment == "fs_mask" {
                        vec![
                            Some(wgpu::ColorTargetState {
                                format: target,
                                blend: None,
                                write_mask: wgpu::ColorWrites::ALL,
                            }),
                            Some(wgpu::ColorTargetState {
                                format,
                                blend: None,
                                write_mask: wgpu::ColorWrites::ALL,
                            }),
                        ]
                    } else {
                        vec![Some(wgpu::ColorTargetState {
                            format: target,
                            blend,
                            write_mask: wgpu::ColorWrites::ALL,
                        })]
                    },
                    compilation_options: Default::default(),
                }),
                primitive: Default::default(),
                depth_stencil: None,
                multisample: wgpu::MultisampleState {
                    count: samples,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                multiview_mask: None,
                cache: None,
            })
        };
        Self {
            mask_quad: build(
                &mask_layout,
                &world,
                "vs_quad",
                "fs_mask",
                &[Vertex::layout()],
                format.remove_srgb_suffix(),
                None,
            ),
            mask_stroke: build(
                &mask_layout,
                &world,
                "vs_stroke",
                "fs_mask",
                &[WorldStrokeInstance::layout()],
                format.remove_srgb_suffix(),
                None,
            ),
            lift: build(
                &cue_layout,
                &cue,
                "vs_cue",
                "fs_lift",
                &[],
                format,
                Some(wgpu::BlendState::REPLACE),
            ),
            cue: build(
                &cue_layout,
                &cue,
                "vs_cue",
                "fs_cue",
                &[],
                format,
                Some(wgpu::BlendState::ALPHA_BLENDING),
            ),
            table_layout,
            image_layout,
        }
    }
}
