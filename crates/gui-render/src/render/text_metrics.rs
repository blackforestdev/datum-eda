//! Font identity and bounded exact text measurements shared by UI layout and rendering.

use super::*;
use glyphon::{Attrs, Color, Family, Weight};

#[path = "text_shape.rs"]
pub(crate) mod text_shape;

#[path = "measurement_owner.rs"]
pub(crate) mod measurement_owner;

#[path = "font_catalog.rs"]
mod font_catalog;
#[cfg(test)]
use font_catalog::DATUM_FONT_BYTES;
#[cfg(test)]
pub(super) use font_catalog::load_datum_fonts;
pub(super) use font_catalog::try_load_datum_fonts;

/// Real shaped width of a single text run, in px, using cosmic-text/glyphon with
/// the per-`TextFace` attributes and line height of size × 1.22 used by the
/// retained `TextLayout` owner and the GPU draw path.
/// Unlike `estimated_text_run_width_px` (a fixed-advance monospace-style estimate
/// with baked padding), this reflects the PROPORTIONAL IBM Plex Sans Condensed UI
/// face, so per-label error is zero and downstream layout gaps stay uniform.
/// Deterministic: same inputs -> same width, so it is golden-stable.
fn measure_uncached(text: &str, size: f32, face: TextFace) -> anyhow::Result<f32> {
    Ok(measurement_owner::measure(text, size, face, None)?.0)
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

pub(super) fn measured_text_run_width_px(
    text: &str,
    size: f32,
    face: TextFace,
) -> anyhow::Result<f32> {
    MEASUREMENTS.with(|cache| {
        cache
            .borrow_mut()
            .try_measure_kind(text, size, face, MeasurementKind::Width, || {
                measure_uncached(text, size, face)
            })
    })
}

/// Wrapped height uses the same immutable font/metric authority as width.
/// Failed measurements never enter the scalar cache.
pub(super) fn measured_text_run_height_px(
    text: &str,
    width: f32,
    size: f32,
    face: TextFace,
) -> anyhow::Result<f32> {
    let width = width.ceil().max(1.0);
    MEASUREMENTS.with(|cache| {
        cache.borrow_mut().try_measure_kind(
            text,
            size,
            face,
            MeasurementKind::WrappedHeight(width.to_bits()),
            || {
                let (_, rows) = measurement_owner::measure(text, size, face, Some(width))?;
                Ok(rows.max(1) as f32 * (size * 1.22))
            },
        )
    })
}

pub(super) fn text_color(color: [f32; 3]) -> Color {
    Color::rgb(
        (color[0].clamp(0.0, 1.0) * 255.0).round() as u8,
        (color[1].clamp(0.0, 1.0) * 255.0).round() as u8,
        (color[2].clamp(0.0, 1.0) * 255.0).round() as u8,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use glyphon::{Buffer, FontSystem, Metrics, Shaping};

    #[test]
    fn wrapped_height_reuses_only_matching_layout_dependencies() {
        MEASUREMENTS.with(|cache| *cache.borrow_mut() = MeasurementCache::default());
        let text = "A wrapped Console history record with several words.";
        let height = measured_text_run_height_px(text, 99.25, 12.0, TextFace::Mono).unwrap();
        let misses = || MEASUREMENTS.with(|cache| cache.borrow().misses);
        assert_eq!(misses(), 1);
        assert_eq!(
            measured_text_run_height_px(text, 100.0, 12.0, TextFace::Mono).unwrap(),
            height
        );
        assert_eq!(
            misses(),
            1,
            "effective width matches; no buffer or shaping work"
        );
        let wide = measured_text_run_height_px(text, 500.0, 12.0, TextFace::Mono).unwrap();
        assert!(wide < height);
        assert_eq!(misses(), 2);
        measured_text_run_height_px(text, 100.0, 13.0, TextFace::Mono).unwrap();
        measured_text_run_height_px(text, 100.0, 12.0, TextFace::Ui).unwrap();
        measured_text_run_width_px(text, 12.0, TextFace::Mono).unwrap();
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
