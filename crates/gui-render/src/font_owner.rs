//! Mutable renderer font ownership, separate from shared catalog and returned shapes.
use crate::cpu_alloc::{Scope, heap::capacity_bytes};
use glyphon::{AttrsList, CacheKey, FontSystem, ShapeLine, Shaping, SwashCache, SwashImage};
use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};

pub(crate) struct Shape {
    line: ShapeLine,
    _lease: Option<Lease>,
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
        Self { line, _lease: None }
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

pub(crate) trait Source {
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
    fonts: FontSystem,
    scope: Scope,
    outputs: Arc<AtomicU64>,
}
impl Fonts {
    pub fn new() -> Self {
        let scope = Scope::new("renderer-fonts-and-shapes");
        // load_datum_fonts assigns catalog initialization to its own shared scope.
        let fonts = scope.with(crate::load_datum_fonts);
        Self {
            fonts,
            scope,
            outputs: Arc::new(AtomicU64::new(0)),
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
        }
    }
}
impl Source for Fonts {
    fn shape(&mut self, text: &str, attrs: &AttrsList) -> Shape {
        let line = self
            .scope
            .with(|| ShapeLine::new(&mut self.fonts, text, attrs, Shaping::Basic, 8));
        let bytes = payload_bytes(&line) as u64;
        self.outputs.fetch_add(bytes, Ordering::AcqRel);
        Shape {
            line,
            _lease: Some(Lease {
                bytes,
                outputs: self.outputs.clone(),
            }),
        }
    }
    fn raster(
        &mut self,
        cache: &mut SwashCache,
        scope: &Scope,
        key: CacheKey,
    ) -> Option<SwashImage> {
        self.scope
            .with(|| self.fonts.get_font(key.font_id, key.font_weight))?;
        scope.with(|| cache.get_image_uncached(&mut self.fonts, key))
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
    fn font_private_storage_and_shared_shape_lifetimes_are_disjoint() {
        let mut fonts = Fonts::new();
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
