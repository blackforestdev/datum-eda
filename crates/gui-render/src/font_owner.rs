//! Mutable renderer font ownership, separate from shared catalog and returned shapes.
#[path = "font_shape_admission.rs"]
mod admission;
#[path = "font_bidi_scratch.rs"]
mod bidi_scratch;
use crate::cpu_alloc::{Scope, heap::capacity_bytes};
use glyphon::{AttrsList, CacheKey, FontSystem, ShapeLine, Shaping, SwashCache, SwashImage};
use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicU64, Ordering},
};

pub(crate) struct Shape {
    line: Option<ShapeLine>,
    _lease: Option<Lease>,
    pub(super) accounted: AtomicBool,
    construction: std::sync::Mutex<Option<crate::text_buffer_cache::budget::Construction>>,
}
struct Lease {
    bytes: u64,
    outputs: Arc<AtomicU64>,
    lifetimes: Arc<std::sync::Mutex<()>>,
}
impl Drop for Shape {
    fn drop(&mut self) {
        let _lock = self
            ._lease
            .as_ref()
            .map(|lease| lease.lifetimes.lock().unwrap_or_else(|e| e.into_inner()));
        // Retiring returned output is one operation relative to call windows.
        drop(self.line.take());
        if let Some(lease) = &self._lease {
            lease.outputs.fetch_sub(lease.bytes, Ordering::AcqRel);
        }
    }
}
impl std::ops::Deref for Shape {
    type Target = ShapeLine;
    fn deref(&self) -> &ShapeLine {
        self.line.as_ref().expect("live shape owns its line")
    }
}
impl Shape {
    pub fn untracked(line: ShapeLine) -> Self {
        Self {
            line: Some(line),
            _lease: None,
            accounted: AtomicBool::new(false),
            construction: std::sync::Mutex::new(None),
        }
    }
    pub(super) fn published(&self) {
        *self.construction.lock().unwrap_or_else(|e| e.into_inner()) = None;
    }
    pub fn payload_bytes(&self) -> usize {
        payload_bytes(self)
    }
}
fn payload_bytes(line: &ShapeLine) -> usize {
    capacity_bytes::<glyphon::ShapeSpan>(line.spans.capacity())
        + line
            .spans
            .iter()
            .map(|span| {
                capacity_bytes::<glyphon::ShapeWord>(span.words.capacity())
                    + span
                        .words
                        .iter()
                        .map(|word| capacity_bytes::<glyphon::ShapeGlyph>(word.glyphs.capacity()))
                        .sum::<usize>()
            })
            .sum::<usize>()
}

pub(super) fn shape_container_bytes() -> usize {
    if !crate::cpu_alloc::installed() {
        // External consumers without Datum's allocator retain public-size reporting.
        return std::mem::size_of::<Shape>();
    }
    static BYTES: std::sync::OnceLock<usize> = std::sync::OnceLock::new();
    *BYTES.get_or_init(|| {
        crate::cpu_alloc::heap::arc_bytes(Shape::untracked(ShapeLine {
            rtl: false,
            spans: Vec::new(),
            metrics_opt: None,
        }))
    })
}

pub(crate) trait Source {
    fn release_for(&mut self, _bytes: u64) {}
    fn shape(&mut self, text: &str, attrs: &AttrsList) -> anyhow::Result<Shape>;
    fn shape_admitted(
        &mut self,
        text: &str,
        attrs: &AttrsList,
        owner: Option<&crate::text_buffer_cache::budget::Owner>,
    ) -> anyhow::Result<Arc<Shape>> {
        admission::shape(self, text, attrs, owner)
    }
    fn raster(
        &mut self,
        cache: &mut SwashCache,
        scope: &Scope,
        key: CacheKey,
    ) -> anyhow::Result<Option<SwashImage>>;
}
impl Source for FontSystem {
    fn shape(&mut self, text: &str, attrs: &AttrsList) -> anyhow::Result<Shape> {
        Ok(Shape::untracked(ShapeLine::new(
            self,
            text,
            attrs,
            Shaping::Basic,
            8,
        )))
    }
    fn raster(
        &mut self,
        cache: &mut SwashCache,
        scope: &Scope,
        key: CacheKey,
    ) -> anyhow::Result<Option<SwashImage>> {
        if self.get_font(key.font_id, key.font_weight).is_none() {
            return Ok(None);
        }
        Ok(scope.with(|| cache.get_image_uncached(self, key)))
    }
}

pub(crate) struct Fonts {
    fonts: Option<FontSystem>,
    host: Arc<crate::text_gpu::budget::Budget>,
    permits: Option<[crate::text_gpu::budget::Permit; 2]>,
    baseline: u64,
    charged: u64,
    scope: Scope,
    outputs: Arc<AtomicU64>,
    lifetimes: Arc<std::sync::Mutex<()>>,
    last_call: Option<crate::cpu_alloc::calls::Report>,
}
impl Fonts {
    pub fn new(host: Arc<crate::text_gpu::budget::Budget>) -> Self {
        let scope = Scope::new("renderer-fonts-and-shapes");
        // load_datum_fonts assigns catalog initialization to its own shared scope.
        let fonts = scope.with(crate::load_datum_fonts);
        let usage = scope.usage();
        Self {
            fonts: Some(fonts),
            host,
            permits: None,
            baseline: usage.payload_bytes + usage.tracking_bytes,
            charged: 0,
            scope,
            outputs: Arc::new(AtomicU64::new(0)),
            lifetimes: Arc::new(std::sync::Mutex::new(())),
            last_call: None,
        }
    }
    pub fn reserved_bytes(&self) -> u64 {
        self.charged
    }

    fn clear_caches(&mut self) {
        let lifetimes = self.lifetimes.clone();
        let _lock = lifetimes.lock().unwrap_or_else(|e| e.into_inner());
        let fonts = self.fonts.take().expect("font owner initialized");
        // Preserve the exact immutable database IDs, locale and family defaults.
        // Consume the old caches before constructing replacements; returned shapes
        // own their data independently and stay valid across this operation.
        let (locale, db) = fonts.into_locale_and_db();
        self.permits = None;
        self.charged = 0;
        self.fonts = Some(
            self.scope
                .with(|| FontSystem::new_with_locale_and_db(locale, db)),
        );
        self.baseline = self.usage().private_bytes.unwrap_or(0);
    }

    fn call(&self, temporary: u64) -> crate::cpu_alloc::calls::Call {
        crate::cpu_alloc::calls::Call::begin(
            &self.scope,
            self.host.clone(),
            crate::text_gpu::budget::staging_process(),
            self.baseline + self.outputs.load(Ordering::Acquire),
            self.charged + temporary,
        )
    }

    fn finish_call(
        &mut self,
        call: crate::cpu_alloc::calls::Call,
        temporary: Option<[crate::text_gpu::budget::Permit; 2]>,
    ) -> anyhow::Result<()> {
        let bytes = self
            .usage()
            .private_bytes
            .unwrap_or(0)
            .saturating_sub(self.baseline);
        match call.finish(bytes, self.permits.take(), temporary) {
            Ok((permits, report)) => {
                self.last_call = Some(report);
                self.permits = Some(permits);
                self.charged = bytes;
                Ok(())
            }
            Err(error) => {
                self.last_call = error
                    .downcast_ref::<crate::cpu_alloc::calls::Overrun>()
                    .map(|e| e.0);
                self.charged = 0;
                Err(error)
            }
        }
    }

    pub fn usage(&self) -> Usage {
        let allocation = self.scope.usage();
        let returned_shape_bytes = self.outputs.load(Ordering::Acquire);
        let private_bytes = allocation.allocator_installed.then(|| {
            (allocation.payload_bytes + allocation.tracking_bytes)
                .checked_sub(returned_shape_bytes)
                .expect("returned shapes belong to font scope")
        });
        Usage {
            allocation,
            returned_shape_bytes,
            private_bytes,
            fixed_font_bytes: private_bytes.map(|bytes| bytes.min(self.baseline)),
            cache_reserved_bytes: self.charged,
            last_call: self.last_call,
        }
    }
}
impl Source for Fonts {
    fn release_for(&mut self, bytes: u64) {
        if self.charged > 0
            && (bytes > self.host.available()
                || bytes > crate::text_gpu::budget::staging_process().available())
        {
            self.clear_caches();
        }
    }

    fn shape(&mut self, text: &str, attrs: &AttrsList) -> anyhow::Result<Shape> {
        let scratch_bytes = bidi_scratch::required(text)?;
        self.release_for(scratch_bytes);
        let scratch = bidi_scratch::reserve(scratch_bytes, &self.host)?;
        let lifetimes = self.lifetimes.clone();
        let lock = lifetimes.lock().unwrap_or_else(|e| e.into_inner());
        let call = self.call(scratch_bytes);
        let line = self.scope.with(|| {
            ShapeLine::new(
                self.fonts.as_mut().expect("font owner initialized"),
                text,
                attrs,
                Shaping::Basic,
                8,
            )
        });
        let bytes = payload_bytes(&line) as u64;
        self.outputs.fetch_add(bytes, Ordering::AcqRel);
        let shape = Shape {
            line: Some(line),
            accounted: AtomicBool::new(false),
            construction: std::sync::Mutex::new(None),
            _lease: Some(Lease {
                bytes,
                outputs: self.outputs.clone(),
                lifetimes: self.lifetimes.clone(),
            }),
        };
        let result = self.finish_call(call, Some(scratch));
        drop(lock);
        if let Err(error) = result {
            drop(shape);
            self.clear_caches();
            return Err(error);
        }
        Ok(shape)
    }
    fn raster(
        &mut self,
        cache: &mut SwashCache,
        scope: &Scope,
        key: CacheKey,
    ) -> anyhow::Result<Option<SwashImage>> {
        let lifetimes = self.lifetimes.clone();
        let lock = lifetimes.lock().unwrap_or_else(|e| e.into_inner());
        let call = self.call(0);
        let loaded = self.scope.with(|| {
            self.fonts
                .as_mut()
                .expect("font owner initialized")
                .get_font(key.font_id, key.font_weight)
        });
        let image = loaded.and_then(|_| {
            scope.with(|| {
                cache.get_image_uncached(self.fonts.as_mut().expect("font owner initialized"), key)
            })
        });
        let result = self.finish_call(call, None);
        drop(lock);
        if let Err(error) = result {
            self.clear_caches();
            return Err(error);
        }
        Ok(image)
    }
}

/// These fields partition one scope. Do not add returned shapes to cache totals again.
#[derive(Debug)]
pub struct Usage {
    pub allocation: crate::cpu_alloc::Usage,
    pub returned_shape_bytes: u64,
    /// Per-renderer font database/cache and private shaping state combined.
    /// Shared catalog, output shapes and independent layout/raster scratch excluded.
    pub private_bytes: Option<u64>,
    /// Constructor-owned locale/database/fallback infrastructure; separate from
    /// evictable cache growth, and still included in private_bytes/RSS reporting.
    pub fixed_font_bytes: Option<u64>,
    pub cache_reserved_bytes: u64,
    pub last_call: Option<crate::cpu_alloc::calls::Report>,
}
impl crate::Renderer {
    pub fn font_cpu_usage(&self) -> Usage {
        self.font_system.usage()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cache_pressure_preserves_font_identity_and_returned_shapes() {
        let host = crate::text_gpu::budget::Budget::new(16 * 1024 * 1024);
        let mut fonts = Fonts::new(host.clone());
        let ids = || {
            crate::load_datum_fonts()
                .db()
                .faces()
                .map(|f| f.id)
                .collect::<Vec<_>>()
        };
        let original_ids = ids();
        let attrs = AttrsList::new(&crate::text_attrs(crate::TextFace::Ui));
        let first = fonts
            .shape("Cache pressure preserves these glyphs", &attrs)
            .unwrap();
        let glyphs = format!("{:?}", *first);
        assert!(fonts.reserved_bytes() > 0);
        assert_eq!(host.used(), fonts.reserved_bytes());
        let filler = host.reserve(host.available()).unwrap();
        fonts.release_for(1);
        assert_eq!(fonts.reserved_bytes(), 0);
        assert_eq!(format!("{:?}", *first), glyphs);
        assert_eq!(
            fonts
                .fonts
                .as_ref()
                .unwrap()
                .db()
                .faces()
                .map(|f| f.id)
                .collect::<Vec<_>>(),
            original_ids
        );
        drop(filler);
        let second = fonts
            .shape("Cache pressure preserves these glyphs", &attrs)
            .unwrap();
        assert_eq!(format!("{:?}", *second), glyphs);
        assert!(fonts.reserved_bytes() > 0);
        drop(fonts);
        assert_eq!(host.used(), 0);
        assert_eq!(format!("{:?}", *first), glyphs);
        let mut refused = Fonts::new(crate::text_gpu::budget::Budget::new(0));
        assert!(
            refused
                .shape("Cache pressure preserves these glyphs", &attrs)
                .is_err()
        );
        assert_eq!(refused.reserved_bytes(), 0);
        assert_eq!(refused.usage().returned_shape_bytes, 0);
    }

    #[test]
    fn font_private_storage_and_shared_shape_lifetimes_are_disjoint() {
        let mut fonts = Fonts::new(crate::text_gpu::budget::Budget::new(16 * 1024 * 1024));
        let attrs = AttrsList::new(&crate::text_attrs(crate::TextFace::Ui));
        let first = Arc::new(fonts.shape("Shared shaped text", &attrs).unwrap());
        let second = Arc::new(fonts.shape("Another paragraph", &attrs).unwrap());
        let shared = first.clone();
        let expected = first.payload_bytes() + second.payload_bytes();
        let usage = fonts.usage();
        assert_eq!(usage.returned_shape_bytes, expected as u64);
        assert!(usage.private_bytes.unwrap() > 0);
        assert_eq!(
            usage.private_bytes.unwrap() + usage.returned_shape_bytes,
            usage.allocation.payload_bytes + usage.allocation.tracking_bytes
        );
        let scope = fonts.scope.clone();
        drop(fonts);
        let live = || {
            let usage = scope.usage();
            usage.payload_bytes + usage.tracking_bytes
        };
        assert_eq!(
            live(),
            expected as u64,
            "font owner closes while returned shapes survive"
        );
        drop(first);
        assert_eq!(
            live(),
            expected as u64,
            "shared shape is counted once and remains live"
        );
        drop(shared);
        assert_eq!(live(), second.payload_bytes() as u64);
        drop(second);
        assert_eq!(live(), 0);
        assert_eq!(scope.usage().allocations, 0);
    }
}
