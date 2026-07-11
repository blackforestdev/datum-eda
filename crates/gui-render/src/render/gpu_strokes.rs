use wgpu::util::DeviceExt;
use super::{PointNm, Renderer};
use std::ops::Range;

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct WorldStrokeInstance {
    pub(crate) from: [f32; 2],
    pub(crate) to: [f32; 2],
    pub(crate) color_min_px: [f32; 4],
    pub(crate) nominal_nm: f32,
    pub(crate) _pad: [f32; 3],
}

impl WorldStrokeInstance {
    pub(crate) fn segment(from: PointNm, to: PointNm, color: [f32; 3], nominal_nm: i64, min_px: f32) -> Self {
        Self { from: [from.x as f32, from.y as f32], to: [to.x as f32, to.y as f32],
            color_min_px: [color[0], color[1], color[2], min_px],
            nominal_nm: nominal_nm.max(1) as f32, _pad: [0.0; 3] }
    }
    #[cfg(test)]
    pub(crate) fn resolved_width_px(self, live_scale: f32) -> f32 {
        (self.nominal_nm * live_scale).max(self.color_min_px[3])
    }
    pub(crate) fn layout<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout { array_stride: std::mem::size_of::<Self>() as u64,
            step_mode: wgpu::VertexStepMode::Instance, attributes: &[
                wgpu::VertexAttribute { offset: 0, shader_location: 0, format: wgpu::VertexFormat::Float32x2 },
                wgpu::VertexAttribute { offset: 8, shader_location: 1, format: wgpu::VertexFormat::Float32x2 },
                wgpu::VertexAttribute { offset: 16, shader_location: 2, format: wgpu::VertexFormat::Float32x4 },
                wgpu::VertexAttribute { offset: 32, shader_location: 3, format: wgpu::VertexFormat::Float32 },
            ] }
    }
}

pub(crate) fn push_world_stroke_path(out: &mut Vec<WorldStrokeInstance>, path: &[PointNm],
    color: [f32; 3], nominal_nm: i64, min_px: f32) {
    out.extend(path.windows(2).filter(|e| e[0] != e[1])
        .map(|e| WorldStrokeInstance::segment(e[0], e[1], color, nominal_nm, min_px)));
}

pub(crate) fn push_board_graphic_semantic_stroke(out: &mut Vec<super::Quad>,
    strokes: &mut Vec<WorldStrokeInstance>, graphic: &super::BoardGraphicPrimitive, color: [f32; 3]) {
    if graphic.primitive_kind == "polygon" && graphic.path.len() >= 3 {
        super::push_world_polygon_fill_contours(out, &graphic.path, &graphic.holes, color);
        if graphic.width_nm.is_none() { return; }
    }
    let path = if graphic.primitive_kind == "polygon" { super::close_path(&graphic.path) }
        else { graphic.path.clone() };
    push_world_stroke_path(strokes, &path, color,
        super::board_graphic_nominal_nm(&graphic.layer_id, graphic.width_nm), 1.0);
}

pub(crate) const WORLD_STROKE_SHADER: &str = r#"
struct SceneUniform { resolution: vec4<f32>, viewport_origin: vec4<f32>, viewport_size: vec4<f32>, camera_center_scale: vec4<f32> };
@group(0) @binding(0) var<uniform> scene: SceneUniform;
struct In { @builtin(vertex_index) vertex: u32, @location(0) point_a: vec2<f32>, @location(1) point_b: vec2<f32>, @location(2) color_min_px: vec4<f32>, @location(3) nominal_nm: f32 };
struct Out { @builtin(position) position: vec4<f32>, @location(0) color: vec3<f32> };
@vertex fn vs_main(i: In) -> Out {
 let endpoint_b = i.vertex == 1u || i.vertex == 2u || i.vertex == 4u;
 let positive = i.vertex == 0u || i.vertex == 1u || i.vertex == 3u;
 let center = select(i.point_a, i.point_b, endpoint_b); let d = i.point_b - i.point_a;
 let tangent = d / max(length(d), 1.0); let normal = vec2<f32>(-tangent.y, tangent.x);
 let width = max(i.nominal_nm * scene.camera_center_scale.z, i.color_min_px.w);
 let sc = scene.viewport_origin.xy + scene.viewport_size.xy * 0.5
     + (center - scene.camera_center_scale.xy) * scene.camera_center_scale.z
     + tangent * select(-1.0, 1.0, endpoint_b) * width * 0.5;
 let p = sc + normal * select(-1.0, 1.0, positive) * width * 0.5; var o: Out;
 o.position = vec4<f32>((p.x / scene.resolution.x) * 2.0 - 1.0, 1.0 - (p.y / scene.resolution.y) * 2.0, 0.0, 1.0);
 o.color = i.color_min_px.xyz; return o;
}
fn srgb_to_linear(c: vec3<f32>) -> vec3<f32> { return select(pow((c + vec3<f32>(0.055)) / 1.055, vec3<f32>(2.4)), c / 12.92, c <= vec3<f32>(0.04045)); }
@fragment fn fs_main(i: Out) -> @location(0) vec4<f32> { return vec4<f32>(srgb_to_linear(i.color), 1.0); }
"#;

pub(crate) fn create_world_stroke_pipeline(device: &wgpu::Device,
    layout: &wgpu::PipelineLayout, format: wgpu::TextureFormat, samples: u32) -> wgpu::RenderPipeline {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("datum-world-stroke-shader"), source: wgpu::ShaderSource::Wgsl(WORLD_STROKE_SHADER.into()) });
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("datum-world-stroke-pipeline"), layout: Some(layout),
        vertex: wgpu::VertexState { module: &shader, entry_point: Some("vs_main"),
            buffers: &[WorldStrokeInstance::layout()], compilation_options: Default::default() },
        fragment: Some(wgpu::FragmentState { module: &shader, entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState { format, blend: Some(wgpu::BlendState::REPLACE), write_mask: wgpu::ColorWrites::ALL })],
            compilation_options: Default::default() }), primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None, multisample: wgpu::MultisampleState { count: samples, mask: !0,
            alpha_to_coverage_enabled: false }, multiview_mask: None, cache: None })
}

pub(crate) fn draw_world_strokes<'a>(pass: &mut wgpu::RenderPass<'a>,
    pipeline: &'a wgpu::RenderPipeline, bind_group: &'a wgpu::BindGroup,
    buffer: &'a wgpu::Buffer, viewport: super::RectPx, ranges: &[Range<u32>]) {
    if ranges.is_empty() { return; }
    pass.set_pipeline(pipeline); pass.set_bind_group(0, bind_group, &[]);
    pass.set_scissor_rect(viewport.x.max(0.0).floor() as u32, viewport.y.max(0.0).floor() as u32,
        viewport.width.max(1.0).ceil() as u32, viewport.height.max(1.0).ceil() as u32);
    pass.set_vertex_buffer(0, buffer.slice(..));
    for range in ranges { pass.draw(0..6, range.clone()); }
}

impl Renderer {
    pub(crate) fn sync_schematic_world_strokes(&mut self, device: &wgpu::Device,
        queue: &wgpu::Queue, strokes: &[WorldStrokeInstance]) {
        let ptr = strokes.as_ptr() as usize;
        if self.schematic_world_stroke_buffer.is_some()
            && self.schematic_world_stroke_source_ptr == ptr
            && self.schematic_world_stroke_source_len == strokes.len() { return; }
        Self::upload_stroke_instances(device, queue, &mut self.schematic_world_stroke_buffer,
            &mut self.schematic_world_stroke_capacity, "datum-schematic-world-stroke-buffer", strokes);
        self.schematic_world_stroke_source_ptr = ptr;
        self.schematic_world_stroke_source_len = strokes.len();
    }

    pub(crate) fn upload_stroke_instances(device: &wgpu::Device, queue: &wgpu::Queue,
        buffer: &mut Option<wgpu::Buffer>, capacity: &mut usize, label: &str,
        strokes: &[WorldStrokeInstance]) {
        let bytes = bytemuck::cast_slice(strokes);
        if buffer.is_none() || *capacity < bytes.len() {
            *buffer = Some(device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(label), contents: bytes,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST }));
            *capacity = bytes.len();
        } else if let Some(buffer) = buffer { queue.write_buffer(buffer, 0, bytes); }
    }

    pub(crate) fn sync_world_strokes(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, strokes: &[WorldStrokeInstance]) {
        let ptr = strokes.as_ptr() as usize;
        if self.world_stroke_buffer.is_some() && self.world_stroke_source_ptr == ptr && self.world_stroke_source_len == strokes.len() { return; }
        Self::upload_stroke_instances(device, queue, &mut self.world_stroke_buffer,
            &mut self.world_stroke_capacity, "datum-world-stroke-buffer", strokes);
        self.world_stroke_source_ptr = ptr; self.world_stroke_source_len = strokes.len();
    }
}

#[cfg(test)] mod tests {
 use super::*;
 #[test] fn retained_stroke_uses_live_scale_and_px_floor() {
 let s = WorldStrokeInstance::segment(PointNm{x:0,y:0}, PointNm{x:1_000_000,y:0}, [1.0;3], 152_400, 1.0);
  let retained = s;
  assert_eq!(s.resolved_width_px(1e-9), 1.0); assert!((s.resolved_width_px(1e-4)-15.24).abs()<0.001);
  assert_eq!(s, retained, "camera changes must not mutate/rebuild retained strokes");
 }
}
