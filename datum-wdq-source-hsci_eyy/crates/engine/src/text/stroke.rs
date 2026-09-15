use crate::export::ExportError;

use super::backend::{GlyphBackend, GlyphBackendKind};
use super::geometry::{TextAttributes, TextGeometryPrimitive, TextStroke};
use super::layout::layout_text_strokes_with_glyph_lookup;
use super::newstroke_data::glyph_definition;

#[derive(Debug, Default)]
pub struct NewstrokeGlyphBackend;

static NEWSTROKE_BACKEND: NewstrokeGlyphBackend = NewstrokeGlyphBackend;

pub fn newstroke_backend() -> &'static NewstrokeGlyphBackend {
    &NEWSTROKE_BACKEND
}

impl GlyphBackend for NewstrokeGlyphBackend {
    fn kind(&self) -> GlyphBackendKind {
        GlyphBackendKind::Stroke
    }

    fn shape_text_geometry(
        &self,
        text: &str,
        attrs: &TextAttributes,
    ) -> Result<Vec<TextGeometryPrimitive>, ExportError> {
        Ok(
            layout_text_strokes_with_glyph_lookup(text, attrs, glyph_definition)?
                .into_iter()
                .map(TextGeometryPrimitive::Stroke)
                .collect(),
        )
    }

    fn shape_text(
        &self,
        text: &str,
        attrs: &TextAttributes,
    ) -> Result<Vec<TextStroke>, ExportError> {
        layout_text_strokes_with_glyph_lookup(text, attrs, glyph_definition)
    }
}
