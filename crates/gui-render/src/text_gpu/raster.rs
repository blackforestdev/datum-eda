//! Datum ownership of private raster scratch and independently retained pixels.
use super::budget::{Budget, Permit, staging_process};
use glyphon::{CacheKey, SwashImage};
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
    lifetimes: Arc<std::sync::Mutex<()>>,
    last_call: Option<crate::cpu_alloc::calls::Report>,
}

pub(crate) struct Pixels {
    data: Option<Vec<u8>>,
    _lease: OutputLease,
    _permits: [Permit; 2],
}
struct OutputLease {
    outputs: Arc<AtomicU64>,
    bytes: u64,
    lifetimes: Arc<std::sync::Mutex<()>>,
}
impl Drop for Pixels {
    fn drop(&mut self) {
        let _lock = self
            ._lease
            .lifetimes
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        drop(self.data.take());
        self._lease
            .outputs
            .fetch_sub(self._lease.bytes, Ordering::AcqRel);
    }
}
impl std::ops::Deref for Pixels {
    type Target = Vec<u8>;
    fn deref(&self) -> &Self::Target {
        self.data.as_ref().expect("live pixels own their data")
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
            lifetimes: Arc::new(std::sync::Mutex::new(())),
            last_call: None,
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
        fonts: &mut impl crate::text_layout::fonts::Source,
        key: CacheKey,
        host: &Arc<Budget>,
    ) -> anyhow::Result<Option<(SwashImage, Pixels)>> {
        if self
            .host
            .as_ref()
            .is_some_and(|old| !Arc::ptr_eq(old, host))
        {
            self.clear();
        }
        let lifetimes = self.lifetimes.clone();
        let _lock = lifetimes.lock().unwrap_or_else(|e| e.into_inner());
        let call = crate::cpu_alloc::calls::Call::begin(
            &self.scope,
            host.clone(),
            staging_process(),
            self.outputs.load(Ordering::Acquire),
            self.bytes,
        );
        let cache = self
            .scope
            .with(|| self.cache.get_or_insert_with(crate::SwashCache::new));
        let image = match fonts.raster(cache, &self.scope, key) {
            Ok(image) => image,
            Err(error) => {
                drop(call);
                self.clear();
                return Err(error);
            }
        };
        // Until transfer, output pixels participate in the private call's live
        // and peak scratch totals. Retain their permits through atlas submission.
        let bytes = self.private_bytes().unwrap_or_else(|| {
            image
                .as_ref()
                .map_or(0, |image| image.data.capacity() as u64)
        });
        let mut permits = match call.finish(bytes, self.permits.take(), None) {
            Ok((permits, report)) => {
                self.last_call = Some(report);
                permits
            }
            Err(error) => {
                self.last_call = error
                    .downcast_ref::<crate::cpu_alloc::calls::Overrun>()
                    .map(|e| e.0);
                drop(image);
                self.clear();
                return Err(error);
            }
        };
        let output = image.map(|mut image| {
            let data = std::mem::take(&mut image.data);
            let bytes = crate::cpu_alloc::heap::capacity_bytes::<u8>(data.capacity()) as u64;
            self.outputs.fetch_add(bytes, Ordering::AcqRel);
            let pixels = Pixels {
                data: Some(data),
                _lease: OutputLease {
                    outputs: self.outputs.clone(),
                    bytes,
                    lifetimes: self.lifetimes.clone(),
                },
                _permits: [permits[0].split(bytes), permits[1].split(bytes)],
            };
            (image, pixels)
        });
        self.bytes = self.private_bytes().unwrap_or(0);
        self.permits = Some(permits);
        self.host = Some(host.clone());
        Ok(output)
    }
}

impl crate::Renderer {
    pub fn private_raster_call_usage(&self) -> Option<crate::cpu_alloc::calls::Report> {
        self.swash_cache.last_call
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
        let (_, first) = raster.image(&mut fonts, key, &host).unwrap().unwrap();
        assert!(!first.is_empty());
        assert!(raster.reserved_bytes() > 0);
        assert_eq!(
            host.used(),
            raster.private_bytes().unwrap()
                + crate::cpu_alloc::heap::capacity_bytes::<u8>(first.capacity()) as u64
        );
        let usage = raster.scope.usage();
        assert_eq!(usage.payload_bytes + usage.tracking_bytes, host.used());
        let blocked = Budget::new(0);
        assert!(raster.image(&mut fonts, key, &blocked).is_err());
        assert_eq!(raster.reserved_bytes(), 0);
        assert_eq!(raster.private_bytes(), Some(0));
        assert_eq!(
            host.used(),
            crate::cpu_alloc::heap::capacity_bytes::<u8>(first.capacity()) as u64
        );
        let (_, second) = raster.image(&mut fonts, key, &host).unwrap().unwrap();
        assert_eq!(&**first, &**second);
        raster.clear();
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
