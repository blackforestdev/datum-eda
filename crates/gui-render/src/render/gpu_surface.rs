use super::Renderer;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct AttachmentKey {
    extent: (u32, u32),
    format: wgpu::TextureFormat,
    samples: u32,
}
impl AttachmentKey {
    fn new(width: u32, height: u32, format: wgpu::TextureFormat, samples: u32) -> Self {
        Self {
            extent: (width.max(1), height.max(1)),
            format,
            samples,
        }
    }

    fn payload_bytes(self) -> Option<u64> {
        let (block_width, block_height) = self.format.block_dimensions();
        u64::from(self.extent.0.div_ceil(block_width))
            .checked_mul(u64::from(self.extent.1.div_ceil(block_height)))?
            .checked_mul(u64::from(self.format.block_copy_size(None)?))?
            .checked_mul(u64::from(self.samples))
    }
}

/// Current renderer reference, not driver residency or GPU retirement evidence.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SurfaceAttachmentSnapshot {
    pub owner: u64,
    pub allocation: u64,
    pub allocations_created: u64,
    pub extent: (u32, u32),
    pub samples: u32,
    pub format: wgpu::TextureFormat,
    pub payload_bytes: Option<u64>,
}

#[derive(Clone)]
pub(crate) struct SurfaceAttachment {
    key: AttachmentKey,
    allocation: u64,
    view: wgpu::TextureView,
}

/// Keep the resource and its exact reuse identity together. Native admission
/// waits for prior work before replacing an extent; wgpu retains backend GPU
/// references after CPU handles are dropped. This owner does not invent a GPU
/// completion signal from replacement or close.
pub(crate) struct SurfaceAttachments {
    owner: u64,
    allocations: u64,
    current: Option<SurfaceAttachment>,
    #[cfg(all(test, feature = "visual", target_os = "linux"))]
    force_replacement: bool,
}
impl Default for SurfaceAttachments {
    fn default() -> Self {
        static NEXT_OWNER: AtomicU64 = AtomicU64::new(1);
        Self {
            owner: NEXT_OWNER
                .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |v| v.checked_add(1))
                .expect("attachment owner exhausted"),
            allocations: 0,
            current: None,
            #[cfg(all(test, feature = "visual", target_os = "linux"))]
            force_replacement: false,
        }
    }
}
impl SurfaceAttachments {
    fn snapshot(&self) -> Option<SurfaceAttachmentSnapshot> {
        let current = self.current.as_ref()?;
        Some(SurfaceAttachmentSnapshot {
            owner: self.owner,
            allocation: current.allocation,
            allocations_created: self.allocations,
            extent: current.key.extent,
            samples: current.key.samples,
            format: current.key.format,
            payload_bytes: current.key.payload_bytes(),
        })
    }

    fn ensure(&mut self, device: &wgpu::Device, key: AttachmentKey) -> &wgpu::TextureView {
        #[cfg(all(test, feature = "visual", target_os = "linux"))]
        let forced = std::mem::take(&mut self.force_replacement);
        #[cfg(not(all(test, feature = "visual", target_os = "linux")))]
        let forced = false;
        if forced
            || self
                .current
                .as_ref()
                .is_none_or(|current| current.key != key)
        {
            let texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("datum-gui-render-msaa"),
                size: wgpu::Extent3d {
                    width: key.extent.0,
                    height: key.extent.1,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: key.samples,
                dimension: wgpu::TextureDimension::D2,
                format: key.format,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            });
            self.allocations = self
                .allocations
                .checked_add(1)
                .expect("attachment allocation exhausted");
            // Allocate before replacing the old reference, as in the recovered
            // renderer. No empty intermediate attachment or quality reduction.
            self.current = Some(SurfaceAttachment {
                key,
                allocation: self.allocations,
                view: texture.create_view(&wgpu::TextureViewDescriptor::default()),
            });
        }
        &self
            .current
            .as_ref()
            .expect("MSAA attachment initialized")
            .view
    }
}

impl Renderer {
    pub fn surface_attachment_snapshot(&self) -> Option<SurfaceAttachmentSnapshot> {
        self.surface_attachments.snapshot()
    }

    pub(crate) fn ensure_msaa(
        &mut self,
        device: &wgpu::Device,
        width: u32,
        height: u32,
    ) -> &wgpu::TextureView {
        self.surface_attachments.ensure(
            device,
            AttachmentKey::new(width, height, self.msaa_format, self.msaa_samples),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn attachment_identity_uses_physical_extent_format_and_sample_count() {
        let key = AttachmentKey::new(1200, 800, wgpu::TextureFormat::Bgra8UnormSrgb, 8);
        assert_eq!(key.payload_bytes(), Some(1200 * 800 * 4 * 8));
        assert_ne!(key, AttachmentKey::new(800, 1200, key.format, 8));
        assert_ne!(key, AttachmentKey::new(1200, 800, key.format, 4));
        assert_ne!(
            key,
            AttachmentKey::new(1200, 800, wgpu::TextureFormat::Rgba8UnormSrgb, 8)
        );
        assert_eq!(
            AttachmentKey::new(0, 0, key.format, 8),
            AttachmentKey::new(1, 1, key.format, 8)
        );
        assert_ne!(
            SurfaceAttachments::default().owner,
            SurfaceAttachments::default().owner
        );
    }

    #[cfg(all(feature = "visual", target_os = "linux"))]
    #[test]
    fn actual_attachment_reuse_replacement_and_negative_control_have_stable_ids() {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::VULKAN,
            ..Default::default()
        });
        let adapter =
            pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default()))
                .unwrap();
        let (device, _) =
            pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default())).unwrap();
        let mut owner = SurfaceAttachments::default();
        let key = AttachmentKey::new(32, 64, wgpu::TextureFormat::Rgba8Unorm, 4);
        owner.ensure(&device, key);
        let first = owner.snapshot().unwrap();
        for _ in 0..20 {
            owner.ensure(&device, key);
        }
        assert_eq!(owner.snapshot(), Some(first));
        // Equal payload size is not equal extent identity.
        owner.ensure(&device, AttachmentKey::new(64, 32, key.format, 4));
        let second = owner.snapshot().unwrap();
        assert_eq!(second.payload_bytes, first.payload_bytes);
        assert_eq!(second.allocations_created, 2);
        assert_ne!(second.allocation, first.allocation);
        owner.force_replacement = true;
        owner.ensure(&device, AttachmentKey::new(64, 32, key.format, 4));
        let negative = owner.snapshot().unwrap();
        assert_ne!(negative.allocation, second.allocation);
        assert_eq!(negative.allocations_created, 3);
        assert_eq!(negative.extent, second.extent);
    }
}

#[cfg(all(test, feature = "visual", target_os = "linux"))]
#[path = "attachment_cost_probe.rs"]
mod attachment_cost_probe;
