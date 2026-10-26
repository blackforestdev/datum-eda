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

/// Measured width of a single text run in the installed font owner — the same
/// metric authority `wrap_text_sized` uses, exposed so aligned column layouts
/// (the Inspector key/value grid) can size a key column to its widest key.
pub(crate) fn measure_text_width(value: &str, size: f32, face: TextFace) -> anyhow::Result<f32> {
    measured_text_run_width_px(value, size, face)
}

/// Word-aware wrap for the Inspector value column: pack whole whitespace-
/// delimited words up to `width`, breaking a line BEFORE a word that would
/// overflow. A single token wider than the column (a UUID, pool path or 64-char
/// hash) falls back to character-level breaking via `wrap_text_sized`, so prose
/// never splits mid-word while long opaque tokens still fit.
pub(crate) fn wrap_text_words_sized(
    value: &str,
    width: f32,
    size: f32,
    face: TextFace,
) -> anyhow::Result<Vec<String>> {
    let width = width.max(1.0);
    let mut lines: Vec<String> = Vec::new();
    let mut current = String::new();
    for word in value.split_whitespace() {
        let candidate = if current.is_empty() {
            word.to_string()
        } else {
            format!("{current} {word}")
        };
        if measured_text_run_width_px(&candidate, size, face)? <= width {
            current = candidate;
            continue;
        }
        if !current.is_empty() {
            lines.push(std::mem::take(&mut current));
        }
        if measured_text_run_width_px(word, size, face)? <= width {
            current = word.to_string();
        } else {
            // Token wider than the column: char-break it. Every fragment but the
            // last is a complete line; the last can still accept later words.
            let pieces = wrap_text_sized(word, width, size, face)?;
            let count = pieces.len();
            for (index, piece) in pieces.into_iter().enumerate() {
                if index + 1 < count {
                    lines.push(piece);
                } else {
                    current = piece;
                }
            }
        }
    }
    if !current.is_empty() {
        lines.push(current);
    }
    if lines.is_empty() {
        lines.push(String::new());
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
