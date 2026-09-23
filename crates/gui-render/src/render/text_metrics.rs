//! Font identity and bounded exact text measurements shared by UI layout and rendering.

use super::*;

/// Shared, lazily-initialized measuring `FontSystem` loaded with the SAME vendored
/// IBM Plex faces the renderer uses (`load_datum_fonts`), so a measured width here
/// matches what gpu.rs actually shapes. Kept separate from the renderer's own
/// `FontSystem` because measurement happens during scene preparation (no GPU) and
/// must stay deterministic across threads (goldens depend on it).
static MEASURE_FS: std::sync::OnceLock<std::sync::Mutex<FontSystem>> = std::sync::OnceLock::new();

pub(super) fn measure_font_system() -> &'static std::sync::Mutex<FontSystem> {
    MEASURE_FS.get_or_init(|| std::sync::Mutex::new(load_datum_fonts()))
}

/// Discover the process font inventory and locale once for measurement and all
/// renderers. Each consumer keeps its own mutable shaping caches; database IDs,
/// family defaults, embedded sources and fallback locale come from one catalog.
/// No live font-reload path exists: a future reload must replace this authority
/// and invalidate both measurement and renderer caches as one operation.
pub(super) fn load_datum_fonts() -> FontSystem {
    static CATALOG: std::sync::OnceLock<(String, glyphon::fontdb::Database)> =
        std::sync::OnceLock::new();
    let (locale, database) = CATALOG.get_or_init(|| {
        let mut fonts = FontSystem::new();
        install_datum_font_sources(&mut fonts);
        fonts.into_locale_and_db()
    });
    FontSystem::new_with_locale_and_db(locale.clone(), database.clone())
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
static DATUM_FONT_BYTES: [&[u8]; 6] = [
    include_bytes!(
        "../../../engine/assets/fonts/ibm_plex_sans_condensed/IBMPlexSansCondensed-Regular.ttf"
    ),
    include_bytes!(
        "../../../engine/assets/fonts/ibm_plex_sans_condensed/IBMPlexSansCondensed-Medium.ttf"
    ),
    include_bytes!(
        "../../../engine/assets/fonts/ibm_plex_sans_condensed/IBMPlexSansCondensed-SemiBold.ttf"
    ),
    include_bytes!("../../../engine/assets/fonts/ibm_plex_mono/IBMPlexMono-Regular.ttf"),
    include_bytes!("../../../engine/assets/fonts/ibm_plex_mono/IBMPlexMono-Medium.ttf"),
    include_bytes!("../../../engine/assets/fonts/jetbrains_mono/JetBrainsMono-Regular.ttf"),
];

fn install_datum_font_sources(font_system: &mut FontSystem) {
    // The executable already owns immutable font bytes. Share six small Arc
    // handles instead of allocating another Vec for every font system/renderer.
    // Keep load order and per-system databases/shaping caches unchanged.
    static SOURCES: std::sync::OnceLock<[glyphon::fontdb::Source; 6]> = std::sync::OnceLock::new();
    let sources = SOURCES.get_or_init(|| {
        DATUM_FONT_BYTES.map(|bytes| glyphon::fontdb::Source::Binary(std::sync::Arc::new(bytes)))
    });
    let db = font_system.db_mut();
    for source in sources {
        db.load_font_source(source.clone());
    }
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

#[path = "measurement_cache.rs"]
mod measurement_cache;
pub(crate) use measurement_cache::usage as measurement_cache_usage;
use measurement_cache::{MeasurementCache, MeasurementKind};

thread_local! {
    // Font inventory/attributes are immutable here; text, exact size and face
    // and measurement kind/wrap width determine a measurement. Hits do not shape.
    static MEASUREMENTS: std::cell::RefCell<MeasurementCache> = std::cell::RefCell::default();
}

pub(super) fn measured_text_run_width_px(text: &str, size: f32, face: TextFace) -> f32 {
    MEASUREMENTS.with(|cache| {
        cache
            .borrow_mut()
            .measure(text, size, face, || measure_uncached(text, size, face))
    })
}

/// Wrapped height uses the same immutable font/metric authority as width.
/// The effective integer wrap width is a layout dependency, not placement.
pub(super) fn measured_text_run_height_px(
    text: &str,
    width: f32,
    size: f32,
    face: TextFace,
) -> f32 {
    let width = width.ceil().max(1.0);
    MEASUREMENTS.with(|cache| {
        cache.borrow_mut().measure_kind(
            text,
            size,
            face,
            MeasurementKind::WrappedHeight(width.to_bits()),
            || {
                let mut fonts = crate::measure_font_system()
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
                let line_height = size * 1.22;
                let mut buffer = Buffer::new(&mut fonts, Metrics::new(size, line_height));
                buffer.set_size(&mut fonts, Some(width), None);
                buffer.set_text(&mut fonts, text, &text_attrs(face), Shaping::Basic, None);
                buffer.shape_until_scroll(&mut fonts, false);
                buffer.layout_runs().count().max(1) as f32 * line_height
            },
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wrapped_height_reuses_only_matching_layout_dependencies() {
        MEASUREMENTS.with(|cache| *cache.borrow_mut() = MeasurementCache::default());
        let text = "A wrapped Console history record with several words.";
        let height = measured_text_run_height_px(text, 99.25, 12.0, TextFace::Mono);
        let misses = || MEASUREMENTS.with(|cache| cache.borrow().misses);
        assert_eq!(misses(), 1);
        assert_eq!(
            measured_text_run_height_px(text, 100.0, 12.0, TextFace::Mono),
            height
        );
        assert_eq!(
            misses(),
            1,
            "effective width matches; no buffer or shaping work"
        );
        let wide = measured_text_run_height_px(text, 500.0, 12.0, TextFace::Mono);
        assert!(wide < height);
        assert_eq!(misses(), 2);
        measured_text_run_height_px(text, 100.0, 13.0, TextFace::Mono);
        measured_text_run_height_px(text, 100.0, 12.0, TextFace::Ui);
        measured_text_run_width_px(text, 12.0, TextFace::Mono);
        assert_eq!(misses(), 5, "size, face and measurement kind are distinct");
    }

    #[test]
    fn embedded_font_bytes_are_shared_between_font_systems() {
        let first = load_datum_fonts();
        let second = load_datum_fonts();
        assert_eq!(first.locale(), second.locale());
        assert_eq!(
            first
                .db()
                .faces()
                .map(|face| (face.id, &face.post_script_name))
                .collect::<Vec<_>>(),
            second
                .db()
                .faces()
                .map(|face| (face.id, &face.post_script_name))
                .collect::<Vec<_>>()
        );
        let sources = |fonts: &FontSystem| {
            let mut sources: Vec<_> = fonts
                .db()
                .faces()
                .filter_map(|face| {
                    if let glyphon::fontdb::Source::Binary(bytes) = &face.source {
                        Some((face.post_script_name.clone(), bytes.clone()))
                    } else {
                        None
                    }
                })
                .collect();
            sources.sort_by(|a, b| a.0.cmp(&b.0));
            sources
        };
        let first = sources(&first);
        let second = sources(&second);
        assert_eq!(first.len(), 6);
        assert_eq!(second.len(), 6);
        let mut bytes = 0;
        for ((left_name, left), (right_name, right)) in first.iter().zip(&second) {
            assert_eq!(left_name, right_name);
            assert!(
                std::sync::Arc::ptr_eq(left, right),
                "duplicated font: {left_name}"
            );
            assert!(
                DATUM_FONT_BYTES
                    .iter()
                    .any(|embedded| embedded.as_ptr() == left.as_ref().as_ref().as_ptr()),
                "font source must borrow executable storage"
            );
            bytes += left.as_ref().as_ref().len();
        }
        eprintln!("embedded font payload shared across font systems: {bytes} bytes");
    }

    #[test]
    fn shared_font_sources_match_legacy_copy_shaping() {
        let mut shared = load_datum_fonts();
        let mut legacy = FontSystem::new();
        for bytes in DATUM_FONT_BYTES {
            legacy.db_mut().load_font_data(bytes.to_vec());
        }
        let layout = |fonts: &mut FontSystem, face| {
            let mut buffer = Buffer::new(fonts, Metrics::new(13.0, 13.0 * 1.22));
            buffer.set_text(
                fonts,
                "Datum 123 · µm Ω العربية 漢字",
                &text_attrs(face),
                Shaping::Basic,
                None,
            );
            buffer.shape_until_scroll(fonts, false);
            buffer
                .layout_runs()
                .map(|run| (run.line_w.to_bits(), format!("{:?}", run.glyphs)))
                .collect::<Vec<_>>()
        };
        for face in [
            TextFace::Ui,
            TextFace::UiMedium,
            TextFace::UiStrong,
            TextFace::Mono,
            TextFace::Terminal,
        ] {
            assert_eq!(layout(&mut shared, face), layout(&mut legacy, face));
        }
    }
}
