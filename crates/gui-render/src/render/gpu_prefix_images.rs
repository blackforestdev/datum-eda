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
    regional: [SurfaceAttachment; 3],
    pub identity: PairIdentity,
}
impl PrefixImages {
    fn images(&self) -> [Option<&SurfaceAttachment>; 5] {
        [
            Some(&self.working),
            Some(&self.prefix),
            Some(&self.regional[0]),
            Some(&self.regional[1]),
            Some(&self.regional[2]),
        ]
    }
    pub(crate) fn submission_refs(&self) -> impl Iterator<Item = SubmissionRef> {
        self.images()
            .map(|image| image.map(|image| image.image.submission_ref()))
            .into_iter()
            .flatten()
    }
    pub(super) fn snapshots(
        &self,
        owner: u64,
        allocations_created: u64,
    ) -> [Option<SurfaceAttachmentSnapshot>; 5] {
        self.images().map(|image| {
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

    pub(crate) fn restoration_views(&self) -> Option<(&wgpu::TextureView, &wgpu::TextureView)> {
        Some((
            self.prefix.image.unorm_view.as_ref()?,
            self.working.image.unorm_view.as_ref()?,
        ))
    }
    pub fn prefix_view(&self) -> &wgpu::TextureView {
        &self.prefix.image.view
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
    pub regional: Option<[SurfaceAttachment; 3]>,
    pub suppressed: bool,
    pub evicted: bool,
}
#[derive(Default)]
pub(super) struct Optional(pub std::sync::Mutex<OptionalState>);
impl OptionalState {
    pub(super) fn images(&self) -> [Option<&SurfaceAttachment>; 4] {
        [
            self.prefix.as_ref(),
            self.regional.as_ref().map(|r| &r[0]),
            self.regional.as_ref().map(|r| &r[1]),
            self.regional.as_ref().map(|r| &r[2]),
        ]
    }
    fn release(&mut self) {
        for image in self
            .prefix
            .take()
            .into_iter()
            .chain(self.regional.take().into_iter().flatten())
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

    /// Refusal leaves the required target usable. Only supported exact8x formats
    /// receive copy usages; dialogs and non-session rendering never call this.
    pub(crate) fn prefix_images(&mut self, device: &wgpu::Device) -> Option<PrefixImages> {
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
        let parent = current.clone();
        if optional.prefix.is_none() {
            match self.optional_bundle(device, &parent) {
                Ok([prefix, composition, atlas, resolve]) => {
                    optional.prefix = Some(prefix);
                    optional.regional = Some([composition, atlas, resolve]);
                }
                Err(_) => {
                    optional.release();
                    optional.suppressed = true;
                    return None;
                }
            }
        }
        optional.prefix.as_ref()?;
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
            regional: optional.regional.as_ref()?.clone(),
        })
    }
}
