//! Wrapped query status and matched-name emphasis in the existing folder list.
use super::*;
pub(crate) fn suggestion_header_lines(
    nav: Option<&FolderBrowserState>,
    width: f32,
) -> anyhow::Result<Vec<String>> {
    nav.and_then(FolderBrowserState::suggestion_header)
        .map(|s| wrap_text_face(&s, width, TextFace::Mono))
        .transpose()
        .map(Option::unwrap_or_default)
}
#[allow(clippy::too_many_arguments)]
pub(super) fn label(
    label: &str,
    fragment: Option<&str>,
    x: f32,
    y: f32,
    color: [f32; 3],
    text: &mut Vec<TextRun>,
) -> anyhow::Result<()> {
    let size = design_tokens::typography::CAPTION_SIZE;
    let Some(span) = fragment.and_then(|q| datum_gui_protocol::folder_match_span(label, q)) else {
        draw_text(label, x, y, size, color, TextFace::Ui, text);
        return Ok(());
    };
    let mut left = x;
    for (part, face) in [
        (&label[..span.start], TextFace::Ui),
        (&label[span.clone()], TextFace::UiStrong),
        (&label[span.end..], TextFace::Ui),
    ] {
        draw_text(part, left, y, size, color, face, text);
        left += measured_text_run_width_px(part, size, face)?;
    }
    Ok(())
}
