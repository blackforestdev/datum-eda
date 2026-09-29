//! Optional full-sample prefix storage shares required target generation admission.
use super::*;

const PREFIX_BYTES: u64 = 45 * 1024 * 1024;

pub(super) fn eligible(key: AttachmentKey) -> bool {
    key.samples == 8
        && matches!(
            key.format,
            wgpu::TextureFormat::Rgba8Unorm
                | wgpu::TextureFormat::Rgba8UnormSrgb
                | wgpu::TextureFormat::Bgra8Unorm
                | wgpu::TextureFormat::Bgra8UnormSrgb
        )
        && key
            .payload_bytes()
            .is_some_and(|bytes| bytes <= PREFIX_BYTES)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct PairIdentity {
    owner: u64,
    working: u64,
    prefix: u64,
}

#[cfg(test)]
impl PairIdentity {
    pub(crate) fn test_identity(owner: u64) -> Self {
        Self {
            owner,
            working: 1,
            prefix: 2,
        }
    }
}

/// Handles and reservation ownership travel together, including encode failures.
/// Texture and view aliases are one tracked allocation per image.
pub(crate) struct PrefixImages {
    prefix: SurfaceAttachment,
    working: SurfaceAttachment,
    marker: Option<SurfaceAttachment>,
    pub identity: PairIdentity,
}
impl PrefixImages {
    pub(crate) fn submission_refs(&self) -> impl Iterator<Item = SubmissionRef> {
        [
            Some(&self.prefix),
            Some(&self.working),
            self.marker.as_ref(),
        ]
        .map(|image| image.map(|image| image.image.submission_ref()))
        .into_iter()
        .flatten()
    }
    pub(super) fn snapshots(
        &self,
        owner: u64,
        allocations_created: u64,
    ) -> [Option<SurfaceAttachmentSnapshot>; 3] {
        [
            Some(&self.working),
            Some(&self.prefix),
            self.marker.as_ref(),
        ]
        .map(|image| {
            image.map(|image| SurfaceAttachmentSnapshot {
                owner,
                allocations_created,
                allocation: image.allocation,
                extent: image.key.extent,
                samples: image.key.samples,
                format: image.key.format,
                payload_bytes: image.key.payload_bytes(),
            })
        })
    }

    pub fn prefix_view(&self) -> &wgpu::TextureView {
        &self.prefix.image.view
    }
    pub fn marker_view(&self) -> Option<&wgpu::TextureView> {
        self.marker.as_ref().map(|image| &image.image.view)
    }
    pub fn copy(&self, encoder: &mut wgpu::CommandEncoder) {
        encoder.copy_texture_to_texture(
            self.prefix.image.texture.as_image_copy(),
            self.working.image.texture.as_image_copy(),
            wgpu::Extent3d {
                width: self.working.key.extent.0,
                height: self.working.key.extent.1,
                depth_or_array_layers: 1,
            },
        );
    }
    pub fn logical_copy_bytes(&self) -> u64 {
        self.working
            .key
            .payload_bytes()
            .expect("admitted image payload")
            * 2
    }
}

#[derive(Default)]
pub(super) struct OptionalState {
    pub prefix: Option<SurfaceAttachment>,
    pub marker: Option<SurfaceAttachment>,
    pub suppressed: bool,
    pub evicted: bool,
}
#[derive(Default)]
pub(super) struct Optional(pub std::sync::Mutex<OptionalState>);
impl OptionalState {
    fn release(&mut self) {
        for image in [self.prefix.take(), self.marker.take()]
            .into_iter()
            .flatten()
        {
            image
                .image
                .retire(crate::text_gpu::lifetime::RetirementReason::Replaced);
        }
    }
}
impl crate::text_gpu::optional_residency::Evict for Optional {
    fn evict(&self) {
        let mut state = self.0.lock().unwrap();
        if state.prefix.is_some() {
            state.release();
            state.suppressed = true;
            state.evicted = true;
        }
    }
}
impl SurfaceAttachments {
    /// One eviction/defer per required-target generation, never an allocation loop.
    pub(crate) fn evict_optional_for_required(&mut self) -> bool {
        use crate::text_gpu::optional_residency::Evict;
        self.optional.evict();
        std::mem::take(&mut self.optional.0.lock().unwrap().evicted)
    }
    pub(crate) fn release_prefix(&mut self) {
        self.optional.0.lock().unwrap().release();
    }

    fn optional_image(
        &mut self,
        device: &wgpu::Device,
        key: AttachmentKey,
        generation: Arc<Permit>,
        copy_source: bool,
        optional_bytes: Arc<crate::text_gpu::budget::Budget>,
    ) -> anyhow::Result<SurfaceAttachment> {
        let bytes = key.payload_bytes().unwrap();
        let reservation = GpuReservation::optional(bytes, vec![optional_bytes.reserve(bytes)?])?;
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(if copy_source {
                "datum-retained-prefix"
            } else {
                "datum-copy-timestamp-marker"
            }),
            size: wgpu::Extent3d {
                width: key.extent.0,
                height: key.extent.1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: key.samples,
            dimension: wgpu::TextureDimension::D2,
            format: key.format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | if copy_source {
                    wgpu::TextureUsages::COPY_SRC
                } else {
                    wgpu::TextureUsages::empty()
                },
            view_formats: &[],
        });
        self.allocations = self
            .allocations
            .checked_add(1)
            .expect("attachment allocation exhausted");
        let view = texture.create_view(&Default::default());
        Ok(SurfaceAttachment {
            key,
            allocation: self.allocations,
            image: Arc::new(
                self.owner
                    .track_reserved(
                        AttachmentImage { texture, view },
                        self.allocations,
                        Kind::Attachment,
                        reservation,
                    )
                    .with_shared_permit(generation.clone()),
            ),
            generation,
            optional_bytes,
        })
    }

    /// Refusal leaves the required target usable. Only supported exact8x formats
    /// receive copy usages; dialogs and non-session rendering never call this.
    pub(crate) fn prefix_images(
        &mut self,
        device: &wgpu::Device,
        measured: bool,
    ) -> Option<PrefixImages> {
        if !eligible(self.current.as_ref()?.key) {
            self.release_prefix();
            return None;
        }
        if self.optional_registration.is_none() {
            self.optional_registration =
                crate::text_gpu::optional_residency::register(self.optional.clone());
            self.optional_registration.as_ref()?;
        }
        let slot = self.optional.clone();
        let mut optional = slot.0.lock().unwrap();
        if optional.suppressed {
            return None;
        }
        let current = self.current.as_ref()?;
        if !eligible(current.key) {
            optional.release();
            return None;
        }
        let key = current.key;
        let generation = current.generation.clone();
        let optional_bytes = current.optional_bytes.clone();
        if optional.prefix.is_none() {
            optional.prefix = self
                .optional_image(
                    device,
                    key,
                    generation.clone(),
                    true,
                    optional_bytes.clone(),
                )
                .ok();
        }
        optional.prefix.as_ref()?;
        if measured && optional.marker.is_none() {
            optional.marker = self
                .optional_image(
                    device,
                    AttachmentKey::new(1, 1, wgpu::TextureFormat::Rgba8Unorm, 1),
                    generation,
                    false,
                    optional_bytes,
                )
                .ok();
            if optional.marker.is_none() {
                optional.release();
                return None;
            }
        }
        let working = self.current.as_ref()?.clone();
        let prefix = optional.prefix.as_ref()?.clone();
        Some(PrefixImages {
            identity: PairIdentity {
                owner: self.owner.id(),
                working: working.allocation,
                prefix: prefix.allocation,
            },
            working,
            prefix,
            marker: optional.marker.clone(),
        })
    }
}
