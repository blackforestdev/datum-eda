//! Mutable renderer font ownership, separate from shared catalog and returned shapes.
use crate::cpu_alloc::{Scope, heap::capacity_bytes};
use glyphon::{AttrsList, CacheKey, FontSystem, ShapeLine, Shaping, SwashCache, SwashImage};
use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicU64, Ordering},
};

pub(crate) struct Shape {
    line: ShapeLine,
    _lease: Option<Lease>,
    pub(super) accounted: AtomicBool,
}
struct Lease {
    bytes: u64,
    outputs: Arc<AtomicU64>,
}
impl Drop for Lease {
    fn drop(&mut self) {
        self.outputs.fetch_sub(self.bytes, Ordering::AcqRel);
    }
}
impl std::ops::Deref for Shape {
    type Target = ShapeLine;
    fn deref(&self) -> &ShapeLine {
        &self.line
    }
}
impl Shape {
    pub fn untracked(line: ShapeLine) -> Self {
        Self {
            line,
            _lease: None,
            accounted: AtomicBool::new(false),
        }
    }
    pub fn payload_bytes(&self) -> usize {
        payload_bytes(&self.line)
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
    fn shape(&mut self, text: &str, attrs: &AttrsList) -> Shape;
    fn raster(
        &mut self,
        cache: &mut SwashCache,
        scope: &Scope,
        key: CacheKey,
    ) -> Option<SwashImage>;
}
impl Source for FontSystem {
    fn shape(&mut self, text: &str, attrs: &AttrsList) -> Shape {
        Shape::untracked(ShapeLine::new(self, text, attrs, Shaping::Basic, 8))
    }
    fn raster(
        &mut self,
        cache: &mut SwashCache,
        scope: &Scope,
        key: CacheKey,
    ) -> Option<SwashImage> {
        self.get_font(key.font_id, key.font_weight)?;
        scope.with(|| cache.get_image_uncached(self, key))
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
        }
    }
    pub fn reserved_bytes(&self) -> u64 {
        self.charged
    }

    fn clear_caches(&mut self) {
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

    fn admit_caches(&mut self) {
        let Some(private) = self.usage().private_bytes else {
            return;
        };
        let bytes = private.saturating_sub(self.baseline);
        if bytes == self.charged {
            return;
        }
        self.permits = None;
        let reserve = || -> anyhow::Result<_> {
            Ok([
                self.host.reserve(bytes)?,
                crate::text_gpu::budget::staging_process().reserve(bytes)?,
            ])
        };
        match reserve() {
            Ok(permits) => {
                self.permits = Some(permits);
                self.charged = bytes;
            }
            Err(_) => self.clear_caches(),
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

    fn shape(&mut self, text: &str, attrs: &AttrsList) -> Shape {
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
            line,
            accounted: AtomicBool::new(false),
            _lease: Some(Lease {
                bytes,
                outputs: self.outputs.clone(),
            }),
        };
        self.admit_caches();
        shape
    }
    fn raster(
        &mut self,
        cache: &mut SwashCache,
        scope: &Scope,
        key: CacheKey,
    ) -> Option<SwashImage> {
        self.scope.with(|| {
            self.fonts
                .as_mut()
                .expect("font owner initialized")
                .get_font(key.font_id, key.font_weight)
        })?;
        let image = scope.with(|| {
            cache.get_image_uncached(self.fonts.as_mut().expect("font owner initialized"), key)
        });
        self.admit_caches();
        image
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
        let first = fonts.shape("Cache pressure preserves these glyphs", &attrs);
        let glyphs = format!("{:?}", &*first);
        assert!(fonts.reserved_bytes() > 0);
        assert_eq!(host.used(), fonts.reserved_bytes());
        let filler = host.reserve(host.available()).unwrap();
        fonts.release_for(1);
        assert_eq!(fonts.reserved_bytes(), 0);
        assert_eq!(format!("{:?}", &*first), glyphs);
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
        let second = fonts.shape("Cache pressure preserves these glyphs", &attrs);
        assert_eq!(format!("{:?}", &*second), glyphs);
        assert!(fonts.reserved_bytes() > 0);
        drop(fonts);
        assert_eq!(host.used(), 0);
        assert_eq!(format!("{:?}", &*first), glyphs);
        let mut refused = Fonts::new(crate::text_gpu::budget::Budget::new(0));
        let required = refused.shape("Cache pressure preserves these glyphs", &attrs);
        assert_eq!(format!("{:?}", &*required), glyphs);
        assert_eq!(
            refused.reserved_bytes(),
            0,
            "retain output while refusing optional caches"
        );
    }

    #[test]
    fn font_private_storage_and_shared_shape_lifetimes_are_disjoint() {
        let mut fonts = Fonts::new(crate::text_gpu::budget::Budget::new(16 * 1024 * 1024));
        let attrs = AttrsList::new(&crate::text_attrs(crate::TextFace::Ui));
        let first = Arc::new(fonts.shape("Shared shaped text", &attrs));
        let second = Arc::new(fonts.shape("Another paragraph", &attrs));
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
