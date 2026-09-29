use super::Renderer;
use crate::text_gpu::budget::{GpuReservation, Permit};
use crate::text_gpu::lifetime::{Kind, Observer, Owner, SubmissionRef, Tracked};
use std::sync::Arc;
#[path = "gpu_prefix_images.rs"]
mod prefix_images;
#[path = "gpu_regional_images.rs"]
mod regional_images;
pub(crate) use prefix_images::{CompositionIdentity, PrefixImages};

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

/// Current and submitted generations may temporarily be disjoint. Alias handles
/// share allocation identity and must not duplicate resource incidence.
fn merge_usage(
    mut current: impl Iterator<Item = SurfaceAttachmentSnapshot>,
    submitted: [Option<SurfaceAttachmentSnapshot>; 5],
) -> [Option<(SurfaceAttachmentSnapshot, bool)>; 10] {
    let mut usage = std::array::from_fn(|_| current.next().map(|image| (image, true)));
    for submitted in submitted.into_iter().flatten() {
        if !usage.iter().flatten().any(|(image, _)| {
            image.owner == submitted.owner && image.allocation == submitted.allocation
        }) {
            *usage
                .iter_mut()
                .find(|slot| slot.is_none())
                .expect("current and retiring coherent image bundles") = Some((submitted, false));
        }
    }
    usage
}

struct AttachmentImage {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    unorm_view: Option<wgpu::TextureView>,
}

#[derive(Clone)]
pub(crate) struct SurfaceAttachment {
    key: AttachmentKey,
    allocation: u64,
    image: Arc<Tracked<AttachmentImage>>,
    generation: Arc<Permit>,
    optional_bytes: Arc<crate::text_gpu::budget::Budget>,
}

/// Keep the resource and its exact reuse identity together. Native admission
/// waits for prior work before replacing an extent. Shared submission references
/// retain the same tracked allocation until the queue reports completion;
/// replacement and close are not themselves completion signals.
pub(crate) struct SurfaceAttachments {
    damage_views: bool,
    #[cfg(all(test, feature = "visual"))]
    sample_readback: bool,
    owner: Owner,
    generations: Arc<crate::text_gpu::budget::Budget>,
    allocations: u64,
    current: Option<SurfaceAttachment>,
    optional: Arc<prefix_images::Optional>,
    optional_registration: Option<crate::text_gpu::optional_residency::Registration>,
    submitted: [Option<SurfaceAttachmentSnapshot>; 5],
    #[cfg(all(test, feature = "visual", target_os = "linux"))]
    force_replacement: bool,
}
impl Default for SurfaceAttachments {
    fn default() -> Self {
        Self::with_generations(crate::text_gpu::budget::Budget::new(2))
    }
}
impl SurfaceAttachments {
    fn with_generations(generations: Arc<crate::text_gpu::budget::Budget>) -> Self {
        Self {
            damage_views: false,
            #[cfg(all(test, feature = "visual"))]
            sample_readback: false,
            owner: Owner::new(),
            generations,
            allocations: 0,
            current: None,
            optional: crate::cpu_alloc::Scope::new("prefix-image-metadata")
                .with(|| Arc::new(prefix_images::Optional::default())),
            optional_registration: None,
            submitted: [None; 5],
            #[cfg(all(test, feature = "visual", target_os = "linux"))]
            force_replacement: false,
        }
    }
}
impl SurfaceAttachments {
    #[cfg(all(test, feature = "visual"))]
    pub(crate) fn enable_sample_readback(&mut self) {
        assert!(self.current.is_none());
        self.sample_readback = true;
    }

    pub(crate) fn admit_damage_views(&mut self, admitted: bool) {
        assert!(self.current.is_none());
        self.damage_views = admitted;
    }

    pub(super) fn replacement(&self) -> Self {
        Self::with_generations(self.generations.clone())
    }

    pub(crate) fn set_consumers(&self, consumers: crate::resource_consumers::Consumers) {
        if let Some(attachment) = &self.current {
            attachment.image.set_consumers(consumers);
        }
        let optional = self.optional.0.lock().unwrap();
        for attachment in optional.images().into_iter().flatten() {
            attachment.image.set_consumers(consumers);
        }
    }

    pub(super) fn submission_refs(&self) -> impl Iterator<Item = SubmissionRef> {
        let optional = self.optional.0.lock().unwrap();
        [
            self.current.as_ref(),
            optional.images()[0],
            optional.images()[1],
            optional.images()[2],
            optional.images()[3],
        ]
        .map(|image| image.map(|image| image.image.submission_ref()))
        .into_iter()
        .flatten()
    }

    pub(crate) fn begin_attempt(&mut self) {
        self.submitted = [None; 5];
    }
    pub(crate) fn mark_submission(&mut self, images: Option<&PrefixImages>) {
        self.submitted = if let Some(images) = images {
            images.snapshots(self.owner.id(), self.allocations)
        } else {
            [self.snapshot(), None, None, None, None]
        };
    }
    fn usage(&self) -> [Option<(SurfaceAttachmentSnapshot, bool)>; 10] {
        merge_usage(self.snapshots(), self.submitted)
    }
    fn snapshot(&self) -> Option<SurfaceAttachmentSnapshot> {
        self.snapshots().next()
    }
    fn snapshots(&self) -> impl Iterator<Item = SurfaceAttachmentSnapshot> {
        let optional = self.optional.0.lock().unwrap();
        [
            self.current.as_ref(),
            optional.images()[0],
            optional.images()[1],
            optional.images()[2],
            optional.images()[3],
        ]
        .map(|image| {
            image.map(|image| SurfaceAttachmentSnapshot {
                owner: self.owner.id(),
                allocation: image.allocation,
                allocations_created: self.allocations,
                extent: image.key.extent,
                samples: image.key.samples,
                format: image.key.format,
                payload_bytes: image.key.payload_bytes(),
            })
        })
        .into_iter()
        .flatten()
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
            let generation = Arc::new(self.generations.reserve(1).map_err(|_| {
                anyhow::anyhow!("attachment generation limit reached: one current and one retiring")
            })?);
            let reservation = GpuReservation::new(bytes, vec![])?;
            let alias_formats = [key.format.remove_srgb_suffix()];
            let usage = wgpu::TextureUsages::RENDER_ATTACHMENT
                | if prefix_images::eligible(key) {
                    wgpu::TextureUsages::COPY_DST
                } else {
                    wgpu::TextureUsages::empty()
                };
            #[cfg(all(test, feature = "visual"))]
            let usage = if self.sample_readback {
                assert!(prefix_images::eligible(key));
                usage | wgpu::TextureUsages::TEXTURE_BINDING
            } else {
                usage
            };
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
                usage,
                view_formats: if self.damage_views && prefix_images::eligible(key) {
                    &alias_formats
                } else {
                    &[]
                },
            });
            self.allocations = self
                .allocations
                .checked_add(1)
                .expect("attachment allocation exhausted");
            anyhow::ensure!(healthy(), "surface attachment allocation failed");
            let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
            let unorm_view = (self.damage_views && prefix_images::eligible(key)).then(|| {
                texture.create_view(&wgpu::TextureViewDescriptor {
                    format: Some(key.format.remove_srgb_suffix()),
                    ..Default::default()
                })
            });
            let replacement = SurfaceAttachment {
                key,
                allocation: self.allocations,
                image: Arc::new(
                    self.owner
                        .track_reserved(
                            AttachmentImage {
                                texture,
                                view,
                                unorm_view,
                            },
                            self.allocations,
                            Kind::Attachment,
                            reservation,
                        )
                        .with_shared_permit(generation.clone()),
                ),
                generation,
                optional_bytes: crate::text_gpu::budget::Budget::new(
                    regional_images::OPTIONAL_BYTES,
                ),
            };
            // Backend error callbacks may report allocation/validation failure
            // during creation. Keep the old reference until this check passes;
            // the caller aborts before uploads, encoding or submission.
            anyhow::ensure!(healthy(), "surface attachment replacement failed");
            if let Some(previous) = &self.current {
                previous
                    .image
                    .retire(crate::text_gpu::lifetime::RetirementReason::Replaced);
            }
            self.release_prefix();
            self.current = Some(replacement);
            self.submitted = [None; 5];
            let mut optional = self.optional.0.lock().unwrap();
            optional.suppressed = false;
            optional.evicted = false;
        }
        Ok(&self
            .current
            .as_ref()
            .expect("MSAA attachment initialized")
            .image
            .view)
    }
}

impl Renderer {
    /// This host's creating, current and submitted-retiring attachment generations.
    /// Same-host device replacement preserves this allowance.
    pub fn surface_attachment_reserved_generations(&self) -> u64 {
        self.surface_attachments.generations.used()
    }

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
        let _resource_scope = self.resource_host.enter();
        let previous = self.surface_attachments.allocations;
        self.surface_attachments.ensure_guarded(
            device,
            AttachmentKey::new(width, height, self.msaa_format, self.msaa_samples),
            healthy,
        )?;
        Ok(self.surface_attachments.allocations != previous)
    }

    /// Current references plus exact images used by this attempt's submission.
    /// The boolean distinguishes current from already-evicted submitted images.
    pub fn surface_attachment_usage(
        &self,
    ) -> impl Iterator<Item = (SurfaceAttachmentSnapshot, bool)> {
        self.surface_attachments.usage().into_iter().flatten()
    }
    /// Coherent currently referenced working and optional prefix images.
    /// Submitted-retiring storage remains in the allocation observer separately.
    pub fn surface_attachment_snapshots(&self) -> impl Iterator<Item = SurfaceAttachmentSnapshot> {
        self.surface_attachments.snapshots()
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
    #[cfg(all(feature = "visual", target_os = "linux"))]
    impl SurfaceAttachments {
        fn submission_ref(&self) -> Option<SubmissionRef> {
            self.current
                .as_ref()
                .map(|attachment| attachment.image.submission_ref())
        }
    }

    #[test]
    fn five_image_usage_covers_disjoint_generations_and_deduplicates_aliases() {
        let images = |first| {
            std::array::from_fn::<_, 5, _>(|i| SurfaceAttachmentSnapshot {
                owner: 1,
                allocation: first + i as u64,
                allocations_created: 10,
                extent: (1280, 800),
                samples: 8,
                format: wgpu::TextureFormat::Bgra8UnormSrgb,
                payload_bytes: Some(32_768_000),
            })
        };
        let current = images(1);
        let retiring = images(6).map(Some);
        let usage = merge_usage(current.into_iter(), retiring);
        assert_eq!(usage.iter().flatten().count(), 10);
        assert_eq!(usage.iter().flatten().filter(|(_, live)| *live).count(), 5);
        let aliases = merge_usage(current.into_iter(), current.map(Some));
        assert_eq!(aliases.iter().flatten().count(), 5);
        assert!(aliases.iter().flatten().all(|(_, live)| *live));
    }

    #[test]
    fn attachment_identity_uses_physical_extent_format_and_sample_count() {
        let key = AttachmentKey::new(1200, 800, wgpu::TextureFormat::Bgra8UnormSrgb, 8);
        assert_eq!(key.payload_bytes(), Some(1200 * 800 * 4 * 8));
        assert!(prefix_images::eligible(AttachmentKey::new(
            1536, 960, key.format, 8
        )));
        assert!(!prefix_images::eligible(AttachmentKey::new(
            1537, 960, key.format, 8
        )));
        assert!(!prefix_images::eligible(AttachmentKey::new(
            1200, 800, key.format, 4
        )));
        assert!(!prefix_images::eligible(AttachmentKey::new(
            32,
            32,
            wgpu::TextureFormat::Rgba16Float,
            8
        )));
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
            assert_eq!(
                owner.generations.used(),
                1,
                "failed replacement releases generation admission"
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
    fn third_attachment_generation_waits_for_retirement_even_after_recovery() {
        let instance = wgpu::Instance::default();
        let adapter = pollster::block_on(instance.request_adapter(&Default::default())).unwrap();
        let (device, _) = pollster::block_on(adapter.request_device(&Default::default())).unwrap();
        let mut owner = SurfaceAttachments::default();
        let generations = owner.generations.clone();
        let format = wgpu::TextureFormat::Rgba8Unorm;
        owner
            .ensure(&device, AttachmentKey::new(32, 32, format, 4))
            .unwrap();
        let first = owner.submission_ref().unwrap();
        owner
            .ensure(&device, AttachmentKey::new(64, 32, format, 4))
            .unwrap();
        let second = owner.submission_ref().unwrap();
        let current = owner.snapshot().unwrap();
        let third_key = AttachmentKey::new(64, 64, format, 4);
        assert_eq!(generations.used(), 2);
        assert!(owner.ensure(&device, third_key).is_err());
        assert_eq!(
            owner.snapshot(),
            Some(current),
            "refuse before API allocation/publication"
        );
        let mut replacement = owner.replacement();
        assert!(replacement.ensure(&device, third_key).is_err());
        drop(owner);
        assert_eq!(generations.used(), 2, "close is not submission completion");
        drop(first);
        replacement.ensure(&device, third_key).unwrap();
        assert_eq!(generations.used(), 2);
        let stable = replacement.snapshot().unwrap();
        replacement.ensure(&device, third_key).unwrap();
        assert_eq!(replacement.snapshot(), Some(stable));
        drop(second);
        assert_eq!(generations.used(), 1);
        drop(replacement);
        assert_eq!(generations.used(), 0);
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

#[cfg(all(test, feature = "visual"))]
#[path = "prefix_negative_control.rs"]
pub(crate) mod prefix_negative_control;
