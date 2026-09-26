//! World-only pipelines are constructed on the first general-renderer use.
//! Each renderer retains its own device-bound pair; overlay-only windows never
//! construct it. Device replacement starts with an empty owner.
use super::*;

pub(super) struct WorldPipelines {
    pub(super) quads: wgpu::RenderPipeline,
    pub(super) strokes: wgpu::RenderPipeline,
}

impl Renderer {
    pub(crate) fn prepare_world_pipelines(&self, device: &wgpu::Device) {
        self.world_pipelines.get_or_init(|| {
            WorldPipelines::new(
                device,
                &self.scene_bind_group_layout,
                self.msaa_format,
                self.msaa_samples,
            )
        });
    }
}

impl WorldPipelines {
    fn new(
        device: &wgpu::Device,
        scene_bind_group_layout: &wgpu::BindGroupLayout,
        format: wgpu::TextureFormat,
        msaa_samples: u32,
    ) -> Self {
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
        let world_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("datum-gui-render-world-pipeline-layout"),
                bind_group_layouts: &[scene_bind_group_layout],
                immediate_size: 0,
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
        Self {
            quads: world_pipeline,
            strokes: world_stroke_pipeline,
        }
    }
}
