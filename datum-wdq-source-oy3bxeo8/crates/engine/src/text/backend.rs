use crate::export::ExportError;

use super::geometry::{TextAttributes, TextGeometryPrimitive, TextStroke};
use super::outline::vendored_outline_backend;
use super::registry::{family_asset_is_vendored, family_backend_kind};
use super::stroke::newstroke_backend;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GlyphBackendKind {
    Stroke,
    Outline,
}

pub trait GlyphBackend: Send + Sync {
    fn kind(&self) -> GlyphBackendKind;
    fn shape_text_geometry(
        &self,
        text: &str,
        attrs: &TextAttributes,
    ) -> Result<Vec<TextGeometryPrimitive>, ExportError>;
    fn shape_text(
        &self,
        text: &str,
        attrs: &TextAttributes,
    ) -> Result<Vec<TextStroke>, ExportError> {
        Ok(self
            .shape_text_geometry(text, attrs)?
            .into_iter()
            .flat_map(|primitive| primitive.stroke_fallback(attrs.stroke_width_nm))
            .collect())
    }
}

pub fn default_backend_for_attributes(attrs: &TextAttributes) -> &'static dyn GlyphBackend {
    if family_backend_kind(&attrs.family) == GlyphBackendKind::Outline
        && family_asset_is_vendored(&attrs.family)
    {
        return vendored_outline_backend();
    }
    newstroke_backend()
}
