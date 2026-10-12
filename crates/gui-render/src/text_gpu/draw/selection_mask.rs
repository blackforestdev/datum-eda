//! Replay actual clipped fallback glyph instances into the shared silhouette mask.
use super::*;
impl Draw {
    pub(crate) fn prepare_selection_mask(
        &self,
        device: &wgpu::Device,
        atlas: &Atlas,
        format: wgpu::TextureFormat,
        samples: u32,
    ) {
        self.mask_pipeline.get_or_init(|| {
            let original = include_str!("../glyph.wgsl");
            let vertex = original.split("@fragment fn fragment").next().expect("glyph vertex owner");
            let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("datum-selection-fallback-glyph-mask"),
                source: wgpu::ShaderSource::Wgsl(format!("{vertex}{MASK_FRAGMENT}").into()),
            });
            let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("datum-selection-fallback-glyph-mask-layout"),
                bind_group_layouts: &[&atlas.layout], immediate_size: 0,
            });
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("datum-selection-fallback-glyph-mask-pipeline"), layout: Some(&layout),
                vertex: wgpu::VertexState { module: &shader, entry_point: Some("vertex"),
                    buffers: &[wgpu::VertexBufferLayout { array_stride: std::mem::size_of::<Instance>() as u64,
                        step_mode: wgpu::VertexStepMode::Instance,
                        attributes: &wgpu::vertex_attr_array![0=>Float32x4,1=>Uint32x4,2=>Uint32,3=>Uint32],
                    }], compilation_options: Default::default(),
                },
                fragment: Some(wgpu::FragmentState { module: &shader, entry_point: Some("mask"),
                    targets: &[
                        Some(wgpu::ColorTargetState { format: format.remove_srgb_suffix(), blend: None, write_mask: wgpu::ColorWrites::ALL }),
                        Some(wgpu::ColorTargetState { format, blend: None, write_mask: wgpu::ColorWrites::ALL }),
                    ], compilation_options: Default::default(),
                }),
                primitive: Default::default(), depth_stencil: None,
                multisample: wgpu::MultisampleState { count: samples, ..Default::default() },
                multiview_mask: None, cache: None,
            })
        });
    }
    pub(crate) fn render_selection_mask(
        &self,
        atlas: &Atlas,
        pass: &mut wgpu::RenderPass<'_>,
        pane: datum_gui_protocol::PaneId,
    ) -> anyhow::Result<()> {
        if self.selected_instances(pane) == 0 {
            return Ok(());
        }
        anyhow::ensure!(
            self.generation == Some(atlas.generation),
            "stale selected fallback glyphs"
        );
        if let Some(instances) = &self.instances {
            pass.set_pipeline(
                self.mask_pipeline
                    .get()
                    .expect("fallback mask pipeline prepared"),
            );
            pass.set_vertex_buffer(0, instances.slice(..));
            for (page, layer, range) in self.batches.iter() {
                if *layer != crate::TextLayer::Selection(pane) {
                    continue;
                }
                pass.set_bind_group(0, &atlas.pages[*page].bind_group, &[]);
                pass.draw(0..6, range.clone());
            }
        }
        Ok(())
    }
    pub(crate) fn render_workspace(
        &self,
        atlas: &Atlas,
        pass: &mut wgpu::RenderPass<'_>,
        damage: &wgpu::BindGroup,
        overflow: impl Fn(datum_gui_protocol::PaneId) -> bool,
    ) -> anyhow::Result<()> {
        self.render_filtered(atlas, pass, damage, |layer| match layer {
            crate::TextLayer::Workspace => true,
            crate::TextLayer::Selection(pane) => !overflow(pane),
            crate::TextLayer::Foreground => false,
        })
    }
}
const MASK_FRAGMENT: &str = r#"
struct MaskOut { @location(0) label:vec4<f32>, @location(1) material:vec4<f32> };
@fragment fn mask(input: Output)->MaskOut {
    let texel=textureLoad(glyph_page,vec2<i32>(input.tex),0);
    let alpha=select(texel.r,texel.a,input.is_color!=0u);
    if (alpha<=0.0) {discard;}
    var output:MaskOut; output.label=vec4<f32>(1.0/255.0,0.0,0.0,0.0);
    output.material=vec4<f32>(0.0); return output;
}
"#;
