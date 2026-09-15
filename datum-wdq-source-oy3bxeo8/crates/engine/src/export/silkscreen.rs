use crate::board::BoardText;
use crate::ir::geometry::Point;
use crate::text::layout_text_strokes_from_board_text;

use super::ExportError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SilkscreenStroke {
    pub from: Point,
    pub to: Point,
    pub width_nm: i64,
}

pub fn render_silkscreen_text_strokes(
    text: &BoardText,
) -> Result<Vec<SilkscreenStroke>, ExportError> {
    Ok(layout_text_strokes_from_board_text(text)?
        .into_iter()
        .map(|stroke| SilkscreenStroke {
            from: stroke.from,
            to: stroke.to,
            width_nm: stroke.width_nm,
        })
        .collect())
}
