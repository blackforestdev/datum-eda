//! Shared single-line value/selection/caret paint and matching pointer placement.
use super::*;
use crate::global_preferences_primitives::{
    ControlPainter, push_dashed_rect_border, push_rounded_rect_with_border,
};
use datum_gui_protocol::single_line_editor::SingleLineEditor;
const FONT_SIZE: f32 = 11.5; // Approved .tin/.ned, start-page target.
fn width(text: &str) -> anyhow::Result<f32> {
    measured_text_run_width_px(text, FONT_SIZE, TextFace::Mono)
}
fn scroll(value: &str, editor: SingleLineEditor, available: f32) -> anyhow::Result<f32> {
    let mut editor = editor;
    editor.position(value, editor.caret, true);
    Ok((width(&value[..editor.caret])? - available.max(0.0) + 1.0).max(0.0))
}
fn boundaries(value: &str) -> Vec<usize> {
    value
        .char_indices()
        .map(|(i, _)| i)
        .chain(std::iter::once(value.len()))
        .collect()
}
fn prefix_at_or_before(value: &str, boundaries: &[usize], target: f32) -> anyhow::Result<usize> {
    let (mut low, mut high) = (0, boundaries.len() - 1);
    while low < high {
        let middle = low + (high - low).div_ceil(2);
        if width(&value[..boundaries[middle]])? <= target {
            low = middle;
        } else {
            high = middle - 1;
        }
    }
    Ok(low)
}
/// Shape only this bounded visible projection. The global shaped-buffer owner
/// caps width to the surface; feeding a full offscreen value would wrap its
/// tail below the one-line clip. Caller text and byte-based editing stay whole.
fn visible_text(value: &str, offset: f32, available: f32) -> anyhow::Result<(&str, f32)> {
    let boundaries = boundaries(value);
    let start = prefix_at_or_before(value, &boundaries, offset)?;
    let end = (prefix_at_or_before(value, &boundaries, offset + available.max(0.0))? + 1)
        .min(boundaries.len() - 1);
    Ok((
        &value[boundaries[start]..boundaries[end]],
        width(&value[..boundaries[start]])?,
    ))
}
/// Coordinates are relative to the field's logical left, using the same font
/// and horizontal reveal as painting. UTF-8 boundary search never splits text.
pub fn single_line_caret_at(
    value: &str,
    editor: SingleLineEditor,
    field_width: f32,
    x: f32,
) -> anyhow::Result<usize> {
    let target = (x - 9.0 + scroll(value, editor, field_width - 18.0)?).max(0.0);
    let boundaries = boundaries(value);
    let low = prefix_at_or_before(value, &boundaries, target)?;
    let left = boundaries[low];
    let right = *boundaries.get(low + 1).unwrap_or(&left);
    Ok(
        if target - width(&value[..left])? <= width(&value[..right])? - target {
            left
        } else {
            right
        },
    )
}
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint(
    value: &str,
    placeholder: &str,
    editor: SingleLineEditor,
    focused: bool,
    editable: bool,
    rect: RectPx,
    painter: &mut ControlPainter<'_>,
    text: &mut Vec<TextRun>,
    hits: &mut Vec<HitRegion>,
    target: HitTarget,
) -> anyhow::Result<()> {
    let border = if focused {
        design_tokens::chrome::STATUS_INFO
    } else {
        design_tokens::chrome::BORDER_STRONG
    };
    push_rounded_rect_with_border(
        painter,
        rect,
        design_tokens::chrome::SURFACE_02,
        border,
        if focused { 2.0 } else { 1.0 },
        5.0,
    );
    if !editable {
        push_dashed_rect_border(painter, rect, border);
    }
    let inner = RectPx {
        x: rect.x + 9.0,
        y: rect.y + 6.0,
        width: (rect.width - 18.0).max(0.0),
        height: (rect.height - 12.0).max(0.0),
    };
    let offset = scroll(value, editor, inner.width)?;
    let x = inner.x - offset;
    let selection = editor.selection(value);
    let selection_rect = RectPx {
        x: x + width(&value[..selection.start])?,
        y: rect.y + 7.0,
        width: (width(&value[..selection.end])? - width(&value[..selection.start])?).max(0.0),
        height: 18.0,
    };
    if focused && !selection.is_empty() {
        if let Some(visible) = selection_rect.intersect(inner) {
            painter.push(Quad::from_rect(visible, design_tokens::chrome::STATUS_INFO));
        }
    }
    let (shown, shown_x) = if value.is_empty() {
        (placeholder, inner.x + if focused { 3.0 } else { 0.0 })
    } else {
        let (shown, prefix_width) = visible_text(value, offset, inner.width)?;
        (shown, x + prefix_width)
    };
    let run_start = text.len();
    draw_text_clipped(
        shown,
        shown_x,
        rect.y + 10.0,
        FONT_SIZE,
        if value.is_empty() || !editable {
            TEXT_MUTED
        } else {
            TEXT_PRIMARY
        },
        TextFace::Mono,
        inner,
        text,
    );
    if let Some(run) = text.get_mut(run_start) {
        run.layout_size = Some((width(shown)?.max(inner.width) + 1.0, 24.0));
    }
    if focused
        && !selection.is_empty()
        && let Some(clip) = selection_rect.intersect(inner)
    {
        draw_text_clipped(
            shown,
            shown_x,
            rect.y + 10.0,
            FONT_SIZE,
            design_tokens::chrome::TEXT_ON_ACCENT,
            TextFace::Mono,
            clip,
            text,
        );
        if let Some(run) = text.last_mut() {
            run.layout_size = Some((width(shown)?.max(inner.width) + 1.0, 24.0));
        }
    }
    if focused && editable {
        let mut editor = editor;
        editor.position(value, editor.caret, true);
        let caret = RectPx {
            x: x + width(&value[..editor.caret])?,
            y: rect.y + 7.0,
            width: 1.0,
            height: 18.0,
        };
        if let Some(visible) = caret.intersect(inner) {
            painter.push(Quad::from_rect(visible, TEXT_PRIMARY));
        }
    }
    hits.push(HitRegion { target, rect });
    Ok(())
}
