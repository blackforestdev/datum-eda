//! Shared control text wrapping and tail presentation using the installed font owner.
use super::*;
pub(crate) fn wrap_text(value: &str, width: f32) -> anyhow::Result<Vec<String>> {
    wrap_text_face(value, width, TextFace::Ui)
}

pub(crate) fn wrap_text_face(
    value: &str,
    width: f32,
    face: TextFace,
) -> anyhow::Result<Vec<String>> {
    wrap_text_sized(value, width, design_tokens::typography::CAPTION_SIZE, face)
}

pub(crate) fn wrap_text_sized(
    value: &str,
    width: f32,
    size: f32,
    face: TextFace,
) -> anyhow::Result<Vec<String>> {
    // Find each measured line boundary rather than reshaping every growing
    // character prefix. Keep exact UTF-8 slices, including spaces and Unicode.
    let boundaries = value
        .char_indices()
        .map(|(i, _)| i)
        .chain(std::iter::once(value.len()))
        .collect::<Vec<_>>();
    let mut lines = vec![];
    let mut start = 0;
    while start + 1 < boundaries.len() {
        let mut low = start + 1;
        let mut high = boundaries.len() - 1;
        while low < high {
            let middle = low + (high - low).div_ceil(2);
            let run = &value[boundaries[start]..boundaries[middle]];
            if measured_text_run_width_px(run, size, face)? <= width.max(1.0) {
                low = middle;
            } else {
                high = middle - 1;
            }
        }
        lines.push(value[boundaries[start]..boundaries[low]].to_owned());
        start = low;
    }
    Ok(lines)
}

pub(crate) fn tail_text(value: &str, width: f32) -> String {
    let count = ((width - 16.0) / 7.0).max(1.0) as usize;
    if value.chars().count() <= count {
        value.into()
    } else {
        format!(
            "…{}",
            value
                .chars()
                .rev()
                .take(count.saturating_sub(1))
                .collect::<String>()
                .chars()
                .rev()
                .collect::<String>()
        )
    }
}
