//! Scalar display truncation, fallback estimates and surface text scaling.
use super::{TextFace, TextRun};

pub(super) fn truncate_text(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        return text.to_string();
    }
    if max_chars <= 3 {
        return text.chars().take(max_chars).collect();
    }
    let keep = max_chars - 3;
    let front = keep / 2;
    let back = keep - front;
    let head: String = text.chars().take(front).collect();
    let tail: String = text
        .chars()
        .rev()
        .take(back)
        .collect::<String>()
        .chars()
        .rev()
        .collect();
    format!("{head}...{tail}")
}

pub(super) fn estimated_text_run_width_px(text: &str, size: f32, face: TextFace) -> f32 {
    let advance_factor = match face {
        TextFace::Ui => 0.62,
        TextFace::UiMedium => 0.64,
        TextFace::UiStrong => 0.66,
        TextFace::Mono => 0.72,
        TextFace::Terminal => 0.72,
    };
    let glyphs = text.chars().count().max(1) as f32;
    glyphs * size * advance_factor + 16.0
}

pub(super) fn scale_text_run_sizes(text_runs: &mut [TextRun], scale: f32) {
    for run in text_runs {
        // Terminal cells already live in the device-pixel coordinate space of
        // the scaled ShellLayout. Their glyph advance, cursor, mouse mapping,
        // and PTY rows/columns all share the terminal geometry's resolved cell
        // metric. Scaling only the glyph here for HiDPI would apply a second
        // scale and make prompt advance diverge from the cursor.
        if run.face != TextFace::Terminal {
            run.size *= scale;
        }
    }
}
