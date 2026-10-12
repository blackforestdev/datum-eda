//! Lossless selection GPU input: shared exact clips and admitted instance bytes.
use super::Stroke;
use crate::gpu_data::screen_buffer::ScreenBuffer;
use crate::text_gpu::{budget::Budget, staging_vec::StagingVec};
use std::sync::Arc;

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub(super) struct Instance {
    from: [f32; 2],
    to: [f32; 2],
    material_width: [f32; 4],
    clip: u32,
    flags: u32,
}
impl Instance {
    pub(super) fn layout() -> wgpu::VertexBufferLayout<'static> {
        const ATTRS: [wgpu::VertexAttribute; 5] =
            wgpu::vertex_attr_array![0=>Float32x2,1=>Float32x2,2=>Float32x4,3=>Uint32,4=>Uint32];
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as u64,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &ATTRS,
        }
    }
}

pub(crate) struct Owner {
    pub(crate) gpu: ScreenBuffer,
    pub(crate) layout: wgpu::BindGroupLayout,
    pub(crate) binding: Option<wgpu::BindGroup>,
    staging: Arc<Budget>,
}
impl Owner {
    pub(crate) fn new(device: &wgpu::Device, screen: Arc<Budget>, staging: Arc<Budget>) -> Self {
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("datum-selection-exact-clips"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
        Self {
            gpu: ScreenBuffer::with_budget(screen).with_staging_budget(staging.clone()),
            layout,
            binding: None,
            staging,
        }
    }
    pub(crate) fn preserve_allowances(&mut self, old: &Self) {
        self.gpu = old.gpu.replacement();
    }

    pub(crate) fn prepare(
        &mut self,
        instances: &mut ScreenBuffer,
        device: &wgpu::Device,
        strokes: &[Stroke],
    ) -> anyhow::Result<()> {
        let (bytes, clips) = encode(strokes, &self.staging)?;
        let clips = self
            .gpu
            .prepare_owned(device, "datum-selection-clip-table", clips, 16)?;
        let bytes = instances.prepare_owned(
            device,
            "datum-selection-stroke-instances",
            bytes,
            std::mem::size_of::<Instance>(),
        )?;
        // Only publish after BOTH byte/range/allocation admissions succeed.
        // Any earlier refusal drops private plans, preserving the old pair.
        self.gpu.commit_owned(clips);
        instances.commit_owned(bytes);
        self.binding = self.gpu.buffer().map(|buffer| {
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("datum-selection-exact-clips"),
                layout: &self.layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffer.as_entire_binding(),
                }],
            })
        });
        Ok(())
    }
}

fn encode(
    strokes: &[Stroke],
    staging: &Arc<Budget>,
) -> anyhow::Result<(StagingVec<u8>, StagingVec<u8>)> {
    let mut bytes = StagingVec::new(
        strokes
            .len()
            .checked_mul(std::mem::size_of::<Instance>())
            .ok_or_else(|| anyhow::anyhow!("selection instance bytes overflow"))?,
        staging,
    )?;
    let mut clips = StagingVec::default();
    for stroke in strokes {
        let clip_bytes = bytemuck::bytes_of(&stroke.clip);
        let clip = match clips.chunks_exact(16).position(|entry| entry == clip_bytes) {
            Some(index) => index,
            None => {
                let index = clips.len() / 16;
                clips.ensure_capacity(
                    clips
                        .len()
                        .checked_add(16)
                        .ok_or_else(|| anyhow::anyhow!("selection clips overflow"))?,
                    staging,
                )?;
                clips.extend_from_slice(clip_bytes);
                index
            }
        };
        // These are precisely the comparisons used by the existing shader.
        // Colors, coordinates, widths and clip floats retain their original bits.
        let mode = if stroke.high_contrast[1] > 1.5 {
            2
        } else if stroke.high_contrast[1] > 0.5 {
            1
        } else {
            0
        };
        let instance = Instance {
            from: stroke.from,
            to: stroke.to,
            material_width: stroke.material_width,
            clip: u32::try_from(clip)?,
            flags: u32::from(stroke.high_contrast[0] > 0.5) | (mode << 1),
        };
        bytes.extend_from_slice(bytemuck::bytes_of(&instance));
    }
    Ok((bytes, clips))
}

#[cfg(test)]
#[path = "gpu/tests.rs"]
mod tests;
