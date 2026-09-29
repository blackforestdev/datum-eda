//! Shared integer damage predicate and immutable, accounted submission uniforms.
use crate::text_gpu::budget::{Budget, GpuReservation};
use crate::text_gpu::lifetime::{Kind, Owner, SubmissionRef, Tracked};
use std::sync::Arc;
use wgpu::util::DeviceExt;
#[path = "gpu_damage_restore.rs"]
pub(crate) mod restore;
use restore::Restoration;
#[path = "painter_clip.rs"]
pub(crate) mod clip;

#[path = "regional_plan.rs"]
pub(crate) mod regional;

const RECTANGLES: usize = 32;
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniform {
    header: [u32; 4],
    rectangles: [[u32; 4]; RECTANGLES],
}

pub(crate) fn layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("datum-shared-damage-layout"),
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: wgpu::BufferSize::new(std::mem::size_of::<Uniform>() as u64),
            },
            count: None,
        }],
    })
}

/// All suffix painters use this exact predicate. Texture sampling that needs
/// implicit derivatives must happen before a conditional discard at the caller.
pub(crate) fn shader(source: &str, group: u32) -> String {
    format!(
        "{source}\nstruct DatumDamage {{ header: vec4<u32>, rectangles: array<vec4<u32>, 32>, }};\n\
         @group({group}) @binding(0) var<uniform> datum_damage: DatumDamage;\n\
         fn datum_damaged(position: vec4<f32>) -> bool {{\n\
             if datum_damage.header.x == 0u {{ return true; }}\n\
             let pixel = vec2<u32>(position.xy);\n\
             for (var i = 0u; i < datum_damage.header.y; i += 1u) {{\n\
                 let rect = datum_damage.rectangles[i];\n\
                 if all(pixel >= rect.xy) && all(pixel < rect.zw) {{ return true; }}\n\
             }}\n\
             return false;\n\
         }}\n"
    )
}

pub(crate) struct Binding {
    buffer: Tracked<wgpu::Buffer>,
    pub(crate) group: wgpu::BindGroup,
}
impl Binding {
    pub(crate) fn set_consumers(&self, consumers: crate::resource_consumers::Consumers) {
        self.buffer.set_consumers(consumers);
    }
    pub(crate) fn submission_ref(&self) -> SubmissionRef {
        self.buffer.submission_ref()
    }
}

pub(crate) struct Masks {
    pub(crate) restoration: Option<Restoration>,
    pub(crate) unrestricted: Binding,
    pub(crate) layout: wgpu::BindGroupLayout,
    fixed_generations: Arc<Budget>,
}
impl Masks {
    pub(crate) fn new(
        device: &wgpu::Device,
        screen: Arc<Budget>,
        previous: Option<&Self>,
    ) -> anyhow::Result<Self> {
        let layout = layout(device);
        let fixed_generations =
            previous.map_or_else(|| Budget::new(2), |p| p.fixed_generations.clone());
        let unrestricted = Self::binding(device, &layout, &screen, &fixed_generations, None)?;
        Ok(Self {
            restoration: None,
            unrestricted,
            layout,
            fixed_generations,
        })
    }

    fn binding(
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        screen: &Arc<Budget>,
        generations: &Arc<Budget>,
        rectangles: Option<&[[u32; 4]]>,
    ) -> anyhow::Result<Binding> {
        let mut uniform = Uniform {
            header: [0; 4],
            rectangles: [[0; 4]; RECTANGLES],
        };
        if let Some(rectangles) = rectangles {
            anyhow::ensure!(rectangles.len() <= RECTANGLES, "damage rectangle limit");
            uniform.header = [1, rectangles.len() as u32, 0, 0];
            uniform.rectangles[..rectangles.len()].copy_from_slice(rectangles);
        }
        let bytes = std::mem::size_of::<Uniform>() as u64;
        let permits = vec![screen.reserve(bytes)?, generations.reserve(1)?];
        let reservation = if rectangles.is_some() {
            GpuReservation::optional(bytes, permits)?
        } else {
            GpuReservation::new(bytes, permits)?
        };
        let buffer = Owner::new().track_reserved(
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("datum-immutable-damage-uniform"),
                contents: bytemuck::bytes_of(&uniform),
                usage: wgpu::BufferUsages::UNIFORM,
            }),
            1,
            Kind::Uniform,
            reservation,
        );
        let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("datum-shared-damage-binding"),
            layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        });
        Ok(Binding { buffer, group })
    }
}

impl super::Renderer {
    /// Admit optional restoration on this actual adapter/device before allocating
    /// attachments. Replacements must call again with their new adapter.
    pub async fn admit_damage_restoration(
        &mut self,
        device: &wgpu::Device,
        adapter: &wgpu::Adapter,
    ) {
        assert!(self.surface_attachment_snapshots().next().is_none());
        self.damage_masks.restoration =
            Restoration::admit(device, adapter, self.msaa_format, self.msaa_samples).await;
        self.surface_attachments
            .admit_damage_views(self.damage_masks.restoration.is_some());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn shared_suffix_and_restoration_shaders_validate_offline() {
        let raw = |source: &'static str| {
            source
                .split_once("r#\"")
                .unwrap()
                .1
                .split_once("\"#")
                .unwrap()
                .0
        };
        let sources = [
            shader(raw(include_str!("gpu_init.rs")), 1),
            shader(include_str!("../text_gpu/glyph.wgsl"), 1),
            shader(raw(include_str!("terminal_graphics.rs")), 2),
            include_str!("gpu_damage_restore.wgsl").to_owned(),
            include_str!("gpu_damage_restore.wgsl").replace("i32(sample)", "0"),
            include_str!("gpu_damage_restore.wgsl").replace(
                "output.source = vec2(tile % columns, tile / columns) * 32u;",
                "output.source = vec2(tile % columns, tile / columns) * 32u + vec2<u32>(32u, 0u);",
            ),
        ];
        for source in sources {
            let module = wgpu::naga::front::wgsl::parse_str(&source).unwrap();
            wgpu::naga::valid::Validator::new(
                wgpu::naga::valid::ValidationFlags::all(),
                wgpu::naga::valid::Capabilities::all(),
            )
            .validate(&module)
            .unwrap();
        }
        assert_eq!(std::mem::size_of::<Uniform>(), 528);
    }
}
