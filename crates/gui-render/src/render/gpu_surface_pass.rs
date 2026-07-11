use super::*;

#[allow(clippy::too_many_arguments)]
impl Renderer {
pub(crate) fn draw_surface_grids<'a>(&'a self, pass: &mut wgpu::RenderPass<'a>,
    batches: &[surface_grid_pass::SurfaceGridBatch]) {
    let Some(buffer) = self.surface_grid_vertex_buffer.as_ref() else { return };
    pass.set_pipeline(&self.pipeline); pass.set_bind_group(0, &self.uniform_bind_group, &[]);
    pass.set_vertex_buffer(0, buffer.slice(..));
    for batch in batches {
        set_scissor(pass, batch.viewport);
        pass.draw(batch.vertices.clone(), 0..1);
    }
}

pub(crate) fn prepare_surface_uniforms(&mut self, device: &wgpu::Device, queue: &wgpu::Queue,
    prepared: &PreparedScene, width: u32, height: u32) {
    while self.surface_scene_uniforms.len() < prepared.surface_passes().len() {
        let buffer = device.create_buffer(&wgpu::BufferDescriptor { label: Some("datum-surface-scene-uniform-buffer"),
            size: std::mem::size_of::<SceneUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("datum-surface-scene-bind-group"), layout: &self.scene_bind_group_layout,
            entries: &[wgpu::BindGroupEntry { binding: 0, resource: buffer.as_entire_binding() }] });
        self.surface_scene_uniforms.push((buffer, bind_group));
    }
    for (surface, (buffer, _)) in prepared.surface_passes().iter().zip(&self.surface_scene_uniforms) {
        let field = inset_rect(surface.scene_viewport, 10.0, 10.0, 10.0, 10.0);
        let projection = Projection::new(field, &surface.bounds, surface.camera);
        queue.write_buffer(buffer, 0, bytemuck::bytes_of(&SceneUniform {
            resolution: [width as f32, height as f32, 0.0, 0.0],
            viewport_origin: [field.x, field.y, 0.0, 0.0],
            viewport_size: [field.width, field.height, 0.0, 0.0],
            camera_center_scale: [surface.camera.center_x_nm, surface.camera.center_y_nm,
                projection.scale, 0.0] }));
    }
}

pub(crate) fn draw_surface_world_passes<'a>(
    &'a self, pass: &mut wgpu::RenderPass<'a>,
    prepared: &PreparedScene,
    board: &RetainedScene,
    schematic: Option<&RetainedScene>,
) {
    for (surface, (_, bind_group)) in prepared.surface_passes().iter().zip(&self.surface_scene_uniforms) {
        let (retained, vertex_buffer, stroke_buffer, ranges, stroke_ranges) =
            match surface.surface {
                SceneSurface::Board => (
                    board,
                    self.world_vertex_buffer.as_ref(),
                    self.world_stroke_buffer.as_ref(),
                    prepared.visible_world_ranges().to_vec(),
                    prepared.visible_world_stroke_ranges().to_vec(),
                ),
                SceneSurface::Schematic => {
                    let Some(retained) = schematic else { continue };
                    (
                        retained,
                        self.schematic_world_vertex_buffer.as_ref(),
                        self.schematic_world_stroke_buffer.as_ref(),
                        retained.all_world_ranges(),
                        retained.all_world_stroke_ranges(),
                    )
                }
            };
        if !retained.world_vertices().is_empty()
            && !ranges.is_empty()
            && let Some(buffer) = vertex_buffer
        {
            pass.set_pipeline(&self.world_pipeline);
            pass.set_bind_group(0, bind_group, &[]);
            set_scissor(pass, surface.scene_viewport);
            pass.set_vertex_buffer(0, buffer.slice(..));
            for range in ranges {
                pass.draw(range, 0..1);
            }
        }
        if !retained.world_strokes().is_empty()
            && let Some(buffer) = stroke_buffer
        {
            draw_world_strokes(
                pass,
                &self.world_stroke_pipeline,
                bind_group,
                buffer,
                surface.scene_viewport,
                &stroke_ranges,
            );
        }
    }
}
}

fn set_scissor(pass: &mut wgpu::RenderPass<'_>, viewport: RectPx) {
    pass.set_scissor_rect(
        viewport.x.max(0.0).floor() as u32,
        viewport.y.max(0.0).floor() as u32,
        viewport.width.max(1.0).ceil() as u32,
        viewport.height.max(1.0).ceil() as u32,
    );
}
