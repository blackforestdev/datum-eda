//! Shared admission and completion ownership for visual capture allocations.
use crate::text_gpu::{
    budget,
    lifetime::{Kind, Owner, Tracked},
};

/// A capture target remains charged through submitted rendering and readback.
pub struct CaptureTarget(Tracked<wgpu::Texture>);
impl CaptureTarget {
    pub fn new(
        device: &wgpu::Device,
        size: wgpu::Extent3d,
        format: wgpu::TextureFormat,
    ) -> anyhow::Result<Self> {
        anyhow::ensure!(
            matches!(
                format,
                wgpu::TextureFormat::Rgba8Unorm
                    | wgpu::TextureFormat::Rgba8UnormSrgb
                    | wgpu::TextureFormat::Bgra8Unorm
                    | wgpu::TextureFormat::Bgra8UnormSrgb
            ),
            "unsupported capture format"
        );
        let bytes = u64::from(size.width)
            .checked_mul(u64::from(size.height))
            .and_then(|n| n.checked_mul(u64::from(size.depth_or_array_layers)))
            .and_then(|n| n.checked_mul(4))
            .ok_or_else(|| anyhow::anyhow!("capture extent overflow"))?;
        let permit = budget::gpu_process().reserve(bytes)?;
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("datum-capture-target"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        Ok(Self(Owner::new().track_with_permits(
            texture,
            bytes,
            1,
            Kind::Attachment,
            vec![permit],
        )))
    }
    pub fn hold_submission(&self, queue: &wgpu::Queue) {
        crate::text_gpu::hold_until_done(queue, vec![self.0.submission_ref()]);
    }
}
impl std::ops::Deref for CaptureTarget {
    type Target = wgpu::Texture;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Readback capacity counts in the process GPU total until the final GPU hold.
pub struct CaptureReadback(Tracked<wgpu::Buffer>);
impl CaptureReadback {
    pub fn new(device: &wgpu::Device, bytes: u64) -> anyhow::Result<Self> {
        let permit = budget::gpu_process().reserve(bytes)?;
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("datum-capture-readback"),
            size: bytes,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        Ok(Self(Owner::new().track_with_permits(
            buffer,
            bytes,
            1,
            Kind::Readback,
            vec![permit],
        )))
    }
    pub fn hold_submission(&self, queue: &wgpu::Queue) {
        crate::text_gpu::hold_until_done(queue, vec![self.0.submission_ref()]);
    }
}
impl std::ops::Deref for CaptureReadback {
    type Target = wgpu::Buffer;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "requires local GPU and serial process admission"]
    fn capture_allocations_refuse_before_creation_and_remain_charged_until_release() {
        let instance = wgpu::Instance::default();
        let adapter = pollster::block_on(instance.request_adapter(&Default::default())).unwrap();
        let (device, queue) =
            pollster::block_on(adapter.request_device(&Default::default())).unwrap();
        let budget = budget::gpu_process();
        let baseline = budget.used();
        let size = wgpu::Extent3d {
            width: 64,
            height: 32,
            depth_or_array_layers: 1,
        };
        let target =
            CaptureTarget::new(&device, size, wgpu::TextureFormat::Rgba8UnormSrgb).unwrap();
        let readback = CaptureReadback::new(&device, 8192).unwrap();
        assert_eq!(budget.used(), baseline + 16384);
        let held_target = target.0.submission_ref();
        let held_readback = readback.0.submission_ref();
        let filler = budget.reserve(budget.available()).unwrap();
        assert!(CaptureTarget::new(&device, size, wgpu::TextureFormat::Rgba8UnormSrgb).is_err());
        assert!(CaptureReadback::new(&device, 256).is_err());
        drop(filler);
        let mut encoder = device.create_command_encoder(&Default::default());
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: &target,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &readback,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(256),
                    rows_per_image: Some(32),
                },
            },
            size,
        );
        queue.submit([encoder.finish()]);
        target.hold_submission(&queue);
        readback.hold_submission(&queue);
        drop(target);
        drop(readback);
        device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
        assert_eq!(budget.used(), baseline + 16384);
        drop(held_target);
        assert_eq!(budget.used(), baseline + 8192);
        drop(held_readback);
        assert_eq!(budget.used(), baseline);
    }
}
