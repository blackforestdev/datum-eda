//! Font identity and bounded exact text measurements shared by UI layout and rendering.

use super::*;

/// Shared, lazily-initialized measuring `FontSystem` loaded with the SAME vendored
/// IBM Plex faces the renderer uses (`load_datum_fonts`), so a measured width here
/// matches what gpu.rs actually shapes. Kept separate from the renderer's own
/// `FontSystem` because measurement happens during scene preparation (no GPU) and
/// must stay deterministic across threads (goldens depend on it).
static MEASURE_FS: std::sync::OnceLock<std::sync::Mutex<FontSystem>> = std::sync::OnceLock::new();

pub(super) fn measure_font_system() -> &'static std::sync::Mutex<FontSystem> {
    MEASURE_FS.get_or_init(|| {
        let mut font_system = FontSystem::new();
        load_datum_fonts(&mut font_system);
        std::sync::Mutex::new(font_system)
    })
}

/// Real shaped width of a single text run, in px, using cosmic-text/glyphon with
/// the exact per-`TextFace` `Attrs` and `Metrics` gpu.rs renders with (see
/// `ensure_text_buffer`: `Metrics::new(size, size * 1.22)`, `text_attrs(face)`).
/// Unlike `estimated_text_run_width_px` (a fixed-advance monospace-style estimate
/// with baked padding), this reflects the PROPORTIONAL IBM Plex Sans Condensed UI
/// face, so per-label error is zero and downstream layout gaps stay uniform.
/// Deterministic: same inputs -> same width, so it is golden-stable.
fn measure_uncached(text: &str, size: f32, face: TextFace) -> f32 {
    let mutex = measure_font_system();
    let mut font_system = mutex
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let mut buffer = Buffer::new(&mut font_system, Metrics::new(size, size * 1.22));
    let attrs = text_attrs(face);
    buffer.set_text(&mut font_system, text, &attrs, Shaping::Basic, None);
    buffer.shape_until_scroll(&mut font_system, false);
    buffer
        .layout_runs()
        .map(|run| run.line_w)
        .fold(0.0_f32, f32::max)
}

/// Load the vendored IBM Plex faces into the glyphon font database so chrome and
/// on-canvas UI text render in the Design Book typeface rather than a system
/// fallback (`docs/gui/DATUM_RENDERING_BOOK.md` §5). Embedded at compile time
/// from the engine's vendored assets so the GUI never depends on the CWD.
pub(super) fn load_datum_fonts(font_system: &mut FontSystem) {
    let db = font_system.db_mut();
    db.load_font_data(
        include_bytes!(
            "../../../engine/assets/fonts/ibm_plex_sans_condensed/IBMPlexSansCondensed-Regular.ttf"
        )
        .to_vec(),
    );
    db.load_font_data(
        include_bytes!(
            "../../../engine/assets/fonts/ibm_plex_sans_condensed/IBMPlexSansCondensed-Medium.ttf"
        )
        .to_vec(),
    );
    db.load_font_data(
        include_bytes!(
            "../../../engine/assets/fonts/ibm_plex_sans_condensed/IBMPlexSansCondensed-SemiBold.ttf"
        )
        .to_vec(),
    );
    db.load_font_data(
        include_bytes!("../../../engine/assets/fonts/ibm_plex_mono/IBMPlexMono-Regular.ttf")
            .to_vec(),
    );
    db.load_font_data(
        include_bytes!("../../../engine/assets/fonts/ibm_plex_mono/IBMPlexMono-Medium.ttf")
            .to_vec(),
    );
    db.load_font_data(
        include_bytes!("../../../engine/assets/fonts/jetbrains_mono/JetBrainsMono-Regular.ttf")
            .to_vec(),
    );
}

pub(super) fn text_attrs(face: TextFace) -> Attrs<'static> {
    match face {
        TextFace::Ui => Attrs::new().family(Family::Name("IBM Plex Sans Condensed")),
        TextFace::UiMedium => Attrs::new()
            .family(Family::Name("IBM Plex Sans Condensed"))
            .weight(Weight::MEDIUM),
        TextFace::UiStrong => Attrs::new()
            .family(Family::Name("IBM Plex Sans Condensed"))
            .weight(Weight::SEMIBOLD),
        TextFace::Mono => Attrs::new().family(Family::Name("IBM Plex Mono")),
        TextFace::Terminal => Attrs::new()
            .family(Family::Name("JetBrains Mono"))
            .letter_spacing(bottom_dock::TERMINAL_LETTER_SPACING_EM),
    }
}

const MAX_MEASUREMENTS: usize = 256;
const MAX_MEASUREMENT_TEXT_BYTES: usize = 64 * 1024;

#[derive(Default)]
struct MeasurementCache {
    entries: std::collections::VecDeque<(String, u32, TextFace, f32)>,
    text_bytes: usize,
}

impl MeasurementCache {
    fn measure(
        &mut self,
        text: &str,
        size: f32,
        face: TextFace,
        miss: impl FnOnce() -> f32,
    ) -> f32 {
        if let Some(index) = self.entries.iter().position(|(label, bits, font, _)| {
            *bits == size.to_bits() && *font == face && label == text
        }) {
            let entry = self.entries.remove(index).expect("matched entry exists");
            let width = entry.3;
            self.entries.push_front(entry);
            return width;
        }
        let width = miss();
        // Arbitrary user text must not turn scalar measurement reuse into an
        // unbounded string history. Oversized strings are measured but not held.
        if text.len() <= MAX_MEASUREMENT_TEXT_BYTES {
            while self.entries.len() >= MAX_MEASUREMENTS
                || self.text_bytes + text.len() > MAX_MEASUREMENT_TEXT_BYTES
            {
                let old = self.entries.pop_back().expect("bounded cache has entries");
                self.text_bytes -= old.0.len();
            }
            self.text_bytes += text.len();
            self.entries
                .push_front((text.to_owned(), size.to_bits(), face, width));
        }
        width
    }
}

thread_local! {
    // Font inventory/attributes are immutable here; text, exact size and face
    // fully determine a measurement. Cache hits allocate and shape nothing.
    static MEASUREMENTS: std::cell::RefCell<MeasurementCache> = std::cell::RefCell::default();
}

pub(super) fn measured_text_run_width_px(text: &str, size: f32, face: TextFace) -> f32 {
    MEASUREMENTS.with(|cache| {
        cache
            .borrow_mut()
            .measure(text, size, face, || measure_uncached(text, size, face))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_measurements_reuse_only_identical_font_size_and_text() {
        let mut cache = MeasurementCache::default();
        for (text, size, face) in [
            ("Project Preferences", 13.0, TextFace::Ui),
            ("Project Preferences", 14.0, TextFace::Ui),
            ("Project Preferences", 13.0, TextFace::UiStrong),
            ("Global Preferences", 13.0, TextFace::Ui),
            ("Units · µm", 13.0, TextFace::Mono),
        ] {
            let actual = measure_uncached(text, size, face);
            let first = cache.measure(text, size, face, || actual);
            assert_eq!(first.to_bits(), actual.to_bits());
            assert_eq!(
                cache
                    .measure(text, size, face, || panic!("cache hit reshaped text"))
                    .to_bits(),
                actual.to_bits()
            );
        }
        assert_eq!(cache.entries.len(), 5);
    }

    #[test]
    fn measurement_cache_bounds_changing_labels_and_keeps_recent_hits() {
        let mut cache = MeasurementCache::default();
        for n in 0..1000 {
            cache.measure(&format!("label-{n}"), 13.0, TextFace::Ui, || n as f32);
            assert!(cache.entries.len() <= MAX_MEASUREMENTS);
            assert!(cache.text_bytes <= MAX_MEASUREMENT_TEXT_BYTES);
        }
        assert_eq!(
            cache.measure("label-999", 13.0, TextFace::Ui, || panic!(
                "recent label evicted"
            )),
            999.0
        );
        let large = "x".repeat(MAX_MEASUREMENT_TEXT_BYTES + 1);
        cache.measure(&large, 13.0, TextFace::Ui, || 1.0);
        assert!(!cache.entries.iter().any(|entry| entry.0 == large));
        for n in 0..20 {
            cache.measure(
                &format!("{n}{}", "x".repeat(8000)),
                13.0,
                TextFace::Ui,
                || 1.0,
            );
            assert!(cache.text_bytes <= MAX_MEASUREMENT_TEXT_BYTES);
        }
    }
}
