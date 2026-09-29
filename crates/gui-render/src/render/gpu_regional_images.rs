//! Transactional admission for one coherent prefix/composition/atlas bundle.
use super::*;

pub(super) const OPTIONAL_BYTES: u64 = 45 * 1024 * 1024;
const ATLAS: u32 = 512;

struct Layout {
    keys: [AttachmentKey; 4],
    bytes: [u64; 4],
    total: u64,
}

impl Layout {
    fn new(prefix: AttachmentKey) -> Option<Self> {
        if !prefix_images::eligible(prefix) {
            return None;
        }
        let keys = [
            prefix,
            AttachmentKey {
                samples: 1,
                ..prefix
            },
            AttachmentKey::new(ATLAS, ATLAS, prefix.format, 8),
            AttachmentKey::new(ATLAS, ATLAS, prefix.format, 1),
        ];
        let mut bytes = [0; 4];
        let mut total = 0_u64;
        for (slot, key) in keys.iter().enumerate() {
            bytes[slot] = key.payload_bytes()?;
            total = total.checked_add(bytes[slot])?;
        }
        (total <= OPTIONAL_BYTES).then_some(Self { keys, bytes, total })
    }

    fn reserve(
        &self,
        allowance: &Arc<crate::text_gpu::budget::Budget>,
    ) -> anyhow::Result<[GpuReservation; 4]> {
        // Both process and bundle admission precede every API allocation. Split
        // reservations keep the complete bundle charged until its final hold.
        let mut reservation =
            GpuReservation::optional(self.total, vec![allowance.reserve(self.total)?])?;
        Ok(self
            .bytes
            .map(|bytes| reservation.split(bytes).expect("checked bundle partition")))
    }
}

impl SurfaceAttachments {
    pub(super) fn optional_bundle(
        &mut self,
        device: &wgpu::Device,
        parent: &SurfaceAttachment,
    ) -> anyhow::Result<[SurfaceAttachment; 4]> {
        anyhow::ensure!(
            device.limits().max_texture_dimension_2d >= ATLAS,
            "regional atlas extent unsupported"
        );
        let layout = Layout::new(parent.key)
            .ok_or_else(|| anyhow::anyhow!("regional image bundle exceeds optional allowance"))?;
        let reservations = layout.reserve(&parent.optional_bytes)?;
        let labels = [
            "datum-retained-prefix",
            "datum-retained-composition",
            "datum-regional-msaa",
            "datum-regional-resolve",
        ];
        let usages = [
            wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::TEXTURE_BINDING,
            wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::COPY_DST,
            wgpu::TextureUsages::RENDER_ATTACHMENT,
            wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        ];
        let mut slot = 0;
        Ok(reservations.map(|reservation| {
            let index = slot;
            slot += 1;
            let key = layout.keys[index];
            let alias = self.damage_views && key.samples == 8;
            let usage = usages[index];
            #[cfg(all(test, feature = "visual"))]
            let usage = if index == 2 && self.sample_readback {
                usage | wgpu::TextureUsages::TEXTURE_BINDING
            } else {
                usage
            };
            let aliases = [key.format.remove_srgb_suffix()];
            let texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some(labels[index]),
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
                view_formats: if alias { &aliases } else { &[] },
            });
            self.allocations = self
                .allocations
                .checked_add(1)
                .expect("attachment allocation exhausted");
            let view = texture.create_view(&Default::default());
            let unorm_view = alias.then(|| {
                texture.create_view(&wgpu::TextureViewDescriptor {
                    format: Some(key.format.remove_srgb_suffix()),
                    ..Default::default()
                })
            });
            SurfaceAttachment {
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
                        .with_shared_permit(parent.generation.clone()),
                ),
                generation: parent.generation.clone(),
                optional_bytes: parent.optional_bytes.clone(),
            }
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aggregate_admission_accounts_for_all_four_optional_images() {
        let key = AttachmentKey::new(1280, 800, wgpu::TextureFormat::Bgra8UnormSrgb, 8);
        let layout = Layout::new(key).unwrap();
        assert_eq!(layout.bytes, [32_768_000, 4_096_000, 8_388_608, 1_048_576]);
        assert_eq!(layout.total, 46_301_184);
        assert_eq!(
            2 * (key.payload_bytes().unwrap() + layout.total),
            158_138_368
        );
        assert!(Layout::new(AttachmentKey::new(1536, 960, key.format, 8)).is_none());
        assert!(Layout::new(AttachmentKey::new(1280, 800, key.format, 4)).is_none());
        assert!(Layout::new(AttachmentKey::new(u32::MAX, u32::MAX, key.format, 8)).is_none());
    }

    #[test]
    fn failed_admission_is_atomic_and_last_image_hold_releases_bundle() {
        let layout = Layout::new(AttachmentKey::new(
            32,
            32,
            wgpu::TextureFormat::Rgba8Unorm,
            8,
        ))
        .unwrap();
        let refused = crate::text_gpu::budget::Budget::new(layout.total - 1);
        assert!(layout.reserve(&refused).is_err());
        assert_eq!(refused.used(), 0);
        let allowed = crate::text_gpu::budget::Budget::new(OPTIONAL_BYTES);
        let [a, c, t, r] = layout.reserve(&allowed).unwrap();
        assert_eq!(allowed.used(), layout.total);
        assert_eq!([a.bytes(), c.bytes(), t.bytes(), r.bytes()], layout.bytes);
        drop((a, c, t));
        assert_eq!(
            allowed.used(),
            layout.total,
            "retiring last hold retains coherent reservation"
        );
        drop(r);
        assert_eq!(allowed.used(), 0);
    }
}
