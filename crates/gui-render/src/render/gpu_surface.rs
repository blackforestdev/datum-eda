use super::Renderer;
use crate::text_gpu::lifetime::{Kind, Observer, Owner, SubmissionRef, Tracked};
use std::sync::Arc;

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
    view: Arc<Tracked<wgpu::TextureView>>,
}

/// Keep the resource and its exact reuse identity together. Native admission
/// waits for prior work before replacing an extent. Shared submission references
/// retain the same tracked allocation until the queue reports completion;
/// replacement and close are not themselves completion signals.
pub(crate) struct SurfaceAttachments {
    owner: Owner,
    allocations: u64,
    current: Option<SurfaceAttachment>,
    #[cfg(all(test, feature = "visual", target_os = "linux"))]
    force_replacement: bool,
}
impl Default for SurfaceAttachments {
    fn default() -> Self {
        Self {
            owner: Owner::new(),
            allocations: 0,
            current: None,
            #[cfg(all(test, feature = "visual", target_os = "linux"))]
            force_replacement: false,
        }
    }
}
impl SurfaceAttachments {
    pub(super) fn submission_ref(&self) -> Option<SubmissionRef> {
        self.current
            .as_ref()
            .map(|attachment| attachment.view.submission_ref())
    }

    fn snapshot(&self) -> Option<SurfaceAttachmentSnapshot> {
        let current = self.current.as_ref()?;
        Some(SurfaceAttachmentSnapshot {
            owner: self.owner.id(),
            allocation: current.allocation,
            allocations_created: self.allocations,
            extent: current.key.extent,
            samples: current.key.samples,
            format: current.key.format,
            payload_bytes: current.key.payload_bytes(),
        })
    }

    fn ensure(
        &mut self,
        device: &wgpu::Device,
        key: AttachmentKey,
    ) -> anyhow::Result<&wgpu::TextureView> {
        self.ensure_guarded(device, key, || true)
    }

    fn ensure_guarded(
        &mut self,
        device: &wgpu::Device,
        key: AttachmentKey,
        mut healthy: impl FnMut() -> bool,
    ) -> anyhow::Result<&wgpu::TextureView> {
        anyhow::ensure!(healthy(), "surface attachment preparation on failed device");
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
            let bytes = key.payload_bytes().ok_or_else(|| {
                anyhow::anyhow!("surface attachment extent or format cannot be accounted")
            })?;
            let permit = crate::text_gpu::budget::gpu_process().reserve(bytes)?;
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
            anyhow::ensure!(healthy(), "surface attachment allocation failed");
            let replacement = SurfaceAttachment {
                key,
                allocation: self.allocations,
                view: Arc::new(self.owner.track_with_permits(
                    texture.create_view(&wgpu::TextureViewDescriptor::default()),
                    bytes,
                    self.allocations,
                    Kind::Attachment,
                    vec![permit],
                )),
            };
            // Backend error callbacks may report allocation/validation failure
            // during creation. Keep the old reference until this check passes;
            // the caller aborts before uploads, encoding or submission.
            anyhow::ensure!(healthy(), "surface attachment replacement failed");
            self.current = Some(replacement);
        }
        Ok(&self
            .current
            .as_ref()
            .expect("MSAA attachment initialized")
            .view)
    }
}

impl Renderer {
    /// Observe this host's current and submitted-retiring attachment capacities.
    /// Survives renderer close without keeping the GPU allocation alive.
    pub fn surface_attachment_allocation_observer(&self) -> Observer {
        self.surface_attachments.owner.observer()
    }

    /// Prepare native attachments before scene uploads and encoding. The host
    /// supplies its existing device-health signal; this does not install another
    /// backend error handler or claim that deferred errors have already arrived.
    /// Returns whether a new attachment was published for lifetime accounting
    /// before any later scene-preparation or encoding error can abort the frame.
    pub fn prepare_surface_attachment(
        &mut self,
        device: &wgpu::Device,
        width: u32,
        height: u32,
        healthy: impl FnMut() -> bool,
    ) -> anyhow::Result<bool> {
        let previous = self.surface_attachments.allocations;
        self.surface_attachments.ensure_guarded(
            device,
            AttachmentKey::new(width, height, self.msaa_format, self.msaa_samples),
            healthy,
        )?;
        Ok(self.surface_attachments.allocations != previous)
    }

    pub fn surface_attachment_snapshot(&self) -> Option<SurfaceAttachmentSnapshot> {
        self.surface_attachments.snapshot()
    }

    pub(crate) fn ensure_msaa(
        &mut self,
        device: &wgpu::Device,
        width: u32,
        height: u32,
    ) -> anyhow::Result<&wgpu::TextureView> {
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
            SurfaceAttachments::default().owner.id(),
            SurfaceAttachments::default().owner.id()
        );
    }

    #[cfg(all(feature = "visual", target_os = "linux"))]
    #[test]
    fn reported_replacement_failure_keeps_old_attachment_and_allows_fresh_retry() {
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
        let old_key = AttachmentKey::new(32, 64, wgpu::TextureFormat::Rgba8Unorm, 4);
        let new_key = AttachmentKey::new(64, 32, old_key.format, old_key.samples);
        owner.ensure(&device, old_key).unwrap();
        let first = owner.snapshot().unwrap();
        assert!(owner.ensure_guarded(&device, new_key, || false).is_err());
        assert_eq!(owner.snapshot(), Some(first));
        // Report failure at texture creation, then at view creation. Neither
        // failed replacement can publish its identity or drop the prior view.
        for fail_at in [2_u32, 3] {
            let mut calls = 0;
            assert!(
                owner
                    .ensure_guarded(&device, new_key, || {
                        calls += 1;
                        calls != fail_at
                    })
                    .is_err()
            );
            let retained = owner.snapshot().unwrap();
            assert_eq!(retained.allocation, first.allocation);
            assert_eq!(retained.extent, first.extent);
            assert_eq!(retained.owner, first.owner);
            assert_eq!(retained.allocations_created, u64::from(fail_at));
            owner.ensure(&device, old_key).unwrap();
            assert_eq!(owner.snapshot(), Some(retained));
        }
        owner.ensure_guarded(&device, new_key, || true).unwrap();
        let recovered = owner.snapshot().unwrap();
        assert_eq!(recovered.extent, new_key.extent);
        assert_eq!(recovered.allocations_created, 4);
        assert_ne!(recovered.allocation, first.allocation);
        assert_eq!(recovered.owner, first.owner);
    }

    #[cfg(all(feature = "visual", target_os = "linux"))]
    #[test]
    fn attachment_retirement_reconciles_submission_completion_and_close() {
        let instance = wgpu::Instance::default();
        let adapter = pollster::block_on(instance.request_adapter(&Default::default())).unwrap();
        let (device, queue) =
            pollster::block_on(adapter.request_device(&Default::default())).unwrap();
        let mut owner = SurfaceAttachments::default();
        let observer = owner.owner.observer();
        let key = AttachmentKey::new(32, 64, wgpu::TextureFormat::Rgba8Unorm, 4);
        let view = owner.ensure(&device, key).unwrap();
        let mut encoder = device.create_command_encoder(&Default::default());
        {
            let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                ..Default::default()
            });
        }
        let first = observer.allocations()[0];
        assert_eq!(first.owner, owner.snapshot().unwrap().owner);
        assert_eq!(first.generation, owner.snapshot().unwrap().allocation);
        assert_eq!(first.bytes, 32 * 64 * 4 * 4);
        let extra = owner.submission_ref().unwrap();
        let submitted = owner.submission_ref().unwrap();
        queue.submit([encoder.finish()]);
        crate::text_gpu::hold_until_done(&queue, vec![submitted]);
        owner
            .ensure(&device, AttachmentKey::new(64, 32, key.format, 4))
            .unwrap();
        let records = observer.allocations();
        assert_eq!(records.len(), 2);
        assert_eq!(
            records.iter().map(|r| r.bytes).sum::<u64>(),
            first.bytes * 2
        );
        assert!(records.iter().any(|r| r.id == first.id && r.retiring));
        device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
        assert_eq!(
            observer.allocations().len(),
            2,
            "explicit shared reference remains"
        );
        drop(extra);
        assert_eq!(
            observer.allocations().len(),
            1,
            "completed submission released its hold"
        );
        drop(owner);
        assert!(
            observer.allocations().is_empty(),
            "observer cannot pin a closed attachment"
        );
        assert!(
            !Renderer::gpu_process_allocations()
                .iter()
                .any(|r| r.id == first.id)
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
        owner.ensure(&device, key).unwrap();
        let first = owner.snapshot().unwrap();
        for _ in 0..20 {
            owner.ensure(&device, key).unwrap();
        }
        assert_eq!(owner.snapshot(), Some(first));
        // Equal payload size is not equal extent identity.
        owner
            .ensure(&device, AttachmentKey::new(64, 32, key.format, 4))
            .unwrap();
        let second = owner.snapshot().unwrap();
        assert_eq!(second.payload_bytes, first.payload_bytes);
        assert_eq!(second.allocations_created, 2);
        assert_ne!(second.allocation, first.allocation);
        owner.force_replacement = true;
        owner
            .ensure(&device, AttachmentKey::new(64, 32, key.format, 4))
            .unwrap();
        let negative = owner.snapshot().unwrap();
        assert_ne!(negative.allocation, second.allocation);
        assert_eq!(negative.allocations_created, 3);
        assert_eq!(negative.extent, second.extent);
    }
}

#[cfg(all(test, feature = "visual", target_os = "linux"))]
#[path = "attachment_cost_probe.rs"]
mod attachment_cost_probe;
