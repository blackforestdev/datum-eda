//! Datum ownership of private raster scratch and independently retained pixels.
use super::budget::{Budget, Permit, staging_process};
use glyphon::{CacheKey, FontSystem, SwashImage};
use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};

pub(crate) struct Raster {
    cache: Option<crate::SwashCache>,
    permits: Option<[Permit; 2]>,
    bytes: u64,
    host: Option<Arc<Budget>>,
    outputs: Arc<AtomicU64>,
    scope: crate::cpu_alloc::Scope,
}

pub(crate) struct Pixels {
    data: Vec<u8>,
    _lease: OutputLease,
}
struct OutputLease {
    outputs: Arc<AtomicU64>,
    bytes: u64,
}
impl Drop for OutputLease {
    fn drop(&mut self) {
        self.outputs.fetch_sub(self.bytes, Ordering::AcqRel);
    }
}
impl std::ops::Deref for Pixels {
    type Target = Vec<u8>;
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl Raster {
    pub fn new() -> Self {
        Self {
            cache: None,
            permits: None,
            bytes: 0,
            host: None,
            outputs: Arc::new(AtomicU64::new(0)),
            scope: crate::cpu_alloc::Scope::new("raster-scratch-and-pixels"),
        }
    }

    pub fn reserved_bytes(&self) -> u64 {
        self.bytes
    }

    fn private_bytes(&self) -> Option<u64> {
        let usage = self.scope.usage();
        usage.allocator_installed.then(|| {
            (usage.payload_bytes + usage.tracking_bytes)
                .checked_sub(self.outputs.load(Ordering::Acquire))
                .expect("raster pixels belong to raster scope")
        })
    }

    pub fn clear(&mut self) {
        self.cache = None;
        self.permits = None;
        self.bytes = 0;
        self.host = None;
    }

    pub fn release_for(&mut self, bytes: u64, host: &Arc<Budget>) {
        if bytes > host.available() || bytes > staging_process().available() {
            self.clear();
        }
    }

    pub fn image(
        &mut self,
        fonts: &mut FontSystem,
        key: CacheKey,
        host: &Arc<Budget>,
    ) -> Option<(SwashImage, Pixels)> {
        // Load font ownership outside raster scratch. The installed API uses
        // this same key when constructing its scaler; no private fields inspected.
        fonts.get_font(key.font_id, key.font_weight)?;
        let image = self.scope.with(|| {
            self.cache
                .get_or_insert_with(crate::SwashCache::new)
                .get_image_uncached(fonts, key)
        });
        let output = image.map(|mut image| {
            let data = std::mem::take(&mut image.data);
            let bytes = crate::cpu_alloc::heap::capacity_bytes::<u8>(data.capacity()) as u64;
            self.outputs.fetch_add(bytes, Ordering::AcqRel);
            let pixels = Pixels {
                data,
                _lease: OutputLease {
                    outputs: self.outputs.clone(),
                    bytes,
                },
            };
            (image, pixels)
        });
        if let Some(bytes) = self.private_bytes()
            && (bytes != self.bytes
                || self.permits.is_none()
                || !self.host.as_ref().is_some_and(|old| Arc::ptr_eq(old, host)))
        {
            self.permits = None;
            let reserve = || -> anyhow::Result<[Permit; 2]> {
                Ok([host.reserve(bytes)?, staging_process().reserve(bytes)?])
            };
            match reserve() {
                Ok(permits) => {
                    self.permits = Some(permits);
                    self.bytes = bytes;
                    self.host = Some(host.clone());
                }
                // Derived scratch is optional; already produced pixels survive.
                Err(_) => self.clear(),
            }
        }
        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn private_raster_scratch_releases_under_pressure_without_losing_live_pixels() {
        let mut fonts = crate::load_datum_fonts();
        let mut buffer = glyphon::Buffer::new(&mut fonts, glyphon::Metrics::new(18.0, 22.0));
        buffer.set_text(
            &mut fonts,
            "A",
            &crate::text_attrs(crate::TextFace::Ui),
            glyphon::Shaping::Basic,
            None,
        );
        buffer.shape_until_scroll(&mut fonts, false);
        let key = buffer.layout_runs().next().unwrap().glyphs[0]
            .physical((0.0, 0.0), 1.0)
            .cache_key;
        let host = Budget::new(16 * 1024 * 1024);
        let mut raster = Raster::new();
        let (_, first) = raster.image(&mut fonts, key, &host).unwrap();
        assert!(!first.is_empty());
        assert!(raster.reserved_bytes() > 0);
        assert_eq!(host.used(), raster.private_bytes().unwrap());
        let usage = raster.scope.usage();
        assert_eq!(
            usage.payload_bytes + usage.tracking_bytes,
            host.used() + crate::cpu_alloc::heap::capacity_bytes::<u8>(first.capacity()) as u64
        );
        let blocked = Budget::new(0);
        // A different host must re-admit even unchanged retained scratch.
        let (_, second) = raster.image(&mut fonts, key, &blocked).unwrap();
        assert_eq!(raster.reserved_bytes(), 0);
        assert_eq!(raster.private_bytes(), Some(0));
        assert_eq!(host.used(), 0);
        assert_eq!(&**first, &**second);
        drop(first);
        assert_eq!(
            raster.outputs.load(Ordering::Acquire),
            crate::cpu_alloc::heap::capacity_bytes::<u8>(second.capacity()) as u64
        );
        drop(second);
        assert_eq!(raster.scope.usage().allocations, 0);
        assert_eq!(raster.outputs.load(Ordering::Acquire), 0);
    }
}
