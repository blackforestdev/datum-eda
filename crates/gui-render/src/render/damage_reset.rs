//! Scissored equivalent of the full scene's opaque attachment clear.
//! The constant is linear, exactly like LoadOp::Clear; screen-vertex sRGB
//! conversion would reset to a different color and is deliberately not used.
use super::APP_BG;

pub(super) fn pipeline(
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
    samples: u32,
) -> wgpu::RenderPipeline {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("datum-damage-reset-shader"),
        source: wgpu::ShaderSource::Wgsl(format!(r#"
@vertex fn vs_main(@builtin(vertex_index) index: u32) -> @builtin(position) vec4<f32> {{
    let points = array<vec2<f32>, 3>(vec2<f32>(-1.0, -1.0), vec2<f32>(3.0, -1.0), vec2<f32>(-1.0, 3.0));
    return vec4<f32>(points[index], 0.0, 1.0);
}}
@fragment fn fs_main() -> @location(0) vec4<f32> {{
    return vec4<f32>({r}, {g}, {b}, 1.0);
}}
"#, r = APP_BG[0], g = APP_BG[1], b = APP_BG[2]).into()),
    });
    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("datum-damage-reset-layout"),
        bind_group_layouts: &[],
        immediate_size: 0,
    });
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("datum-damage-reset-pipeline"),
        layout: Some(&layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            buffers: &[],
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
            count: samples,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview_mask: None,
        cache: None,
    })
}
