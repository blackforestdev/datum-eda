//! Selected-session context fits inside existing strip chrome, never PTY cells.
use super::*;
use datum_gui_protocol::TerminalSessionContext;

const SIZE: f32 = 11.0;
const GAP: f32 = 10.0;

pub(super) fn width(text: &str) -> f32 {
    measure(text, SIZE, TextFace::Mono)
}

fn measure(text: &str, size: f32, face: TextFace) -> f32 {
    // Whole-pixel allocations survive subtraction without turning a single
    // glyph's exact fractional advance into a slightly-too-small empty slot.
    crate::measured_text_run_width_px(text, size, face)
        .unwrap_or_else(|_| (estimated_text_run_width_px(text, size, face) - 16.0).max(0.0))
        .ceil()
}

pub(super) fn tab_width(strip: RectPx, count: usize, actions: f32) -> f32 {
    let desired = if strip.width < 800.0 {
        104.0
    } else {
        MAX_TAB_WIDTH_PX
    };
    // The target reserves 12ch for binding and 11ch for directory at minimum.
    let minimum_context = width("MMMMMMMMMMMMMMMMMMMMMMM") + GAP;
    ((strip.width
        - 24.0
        - PLUS_WIDTH_PX
        - TAB_GAP_PX * (count + 1) as f32
        - actions
        - minimum_context)
        / count.max(1) as f32)
        .clamp(MIN_TAB_WIDTH_PX, desired)
}

pub(super) fn end_ellipsis(text: &str, available: f32) -> String {
    fit_end(text, available, SIZE, TextFace::Mono)
}

fn fit_end(text: &str, available: f32, size: f32, face: TextFace) -> String {
    if measure(text, size, face) <= available {
        return text.to_owned();
    }
    let ellipsis = measure("…", size, face);
    if available < ellipsis {
        return String::new();
    }
    let indices: Vec<_> = text
        .char_indices()
        .map(|(i, _)| i)
        .chain(std::iter::once(text.len()))
        .collect();
    let (mut lo, mut hi) = (0, indices.len() - 1);
    while lo < hi {
        let mid = lo + (hi - lo).div_ceil(2);
        if measure(&text[..indices[mid]], size, face) + ellipsis <= available {
            lo = mid;
        } else {
            hi = mid - 1;
        }
    }
    format!("{}…", &text[..indices[lo]])
}

pub(super) fn tab_label(tab: &TerminalTabState, tab_width: f32) -> String {
    let attention = if tab.unread_bell_count > 0 {
        " !"
    } else if tab.unread_output {
        " •"
    } else {
        ""
    };
    let available = tab_width - CLOSE_WIDTH_PX - 24.0 - measure(attention, 12.5, TextFace::Ui);
    format!(
        "{}{attention}",
        fit_end(&tab.label, available, 12.5, TextFace::Ui)
    )
}

fn display_directory(context: &TerminalSessionContext) -> String {
    let path = std::env::var_os("HOME")
        .and_then(|home| {
            let home = std::path::PathBuf::from(home);
            std::path::Path::new(&context.directory)
                .strip_prefix(home)
                .ok()
                .map(|suffix| {
                    if suffix.as_os_str().is_empty() {
                        "~".to_owned()
                    } else {
                        format!("~/{}", suffix.display())
                    }
                })
        })
        .unwrap_or_else(|| context.directory.clone());
    if context.directory_reported {
        path
    } else {
        format!("Started in {path}")
    }
}

pub(super) fn render(
    context: &TerminalSessionContext,
    strip: RectPx,
    left: f32,
    right: f32,
    text: &mut Vec<TextRun>,
) {
    let binding = context.binding_label();
    let directory = display_directory(context);
    let available = (right - left).max(0.0);
    if available <= GAP {
        return;
    }
    let natural = width(&binding) + GAP + width(&directory);
    let total = natural.min(available);
    let directory_min = width(&directory).min(width("MMMMMMMMMMM"));
    let binding_width = width(&binding).min((total - GAP - directory_min).max(0.0));
    let directory_width = (total - GAP - binding_width).max(0.0);
    let x = right - total;
    let y = centered_text_top(lifecycle_chrome_rect(strip, x, total), SIZE);
    for (value, x, width, color) in [
        (binding.as_str(), x, binding_width, TEXT_PRIMARY),
        (
            directory.as_str(),
            x + binding_width + GAP,
            directory_width,
            TEXT_MUTED,
        ),
    ] {
        let clip = RectPx {
            x,
            y: strip.y,
            width,
            height: TAB_STRIP_ROW_HEIGHT_PX.min(strip.height),
        };
        crate::draw_text_clipped(
            &end_ellipsis(value, width),
            x,
            y,
            SIZE,
            color,
            TextFace::Mono,
            clip,
            text,
        );
    }
}

#[cfg(test)]
#[path = "terminal_context_chrome_tests.rs"]
mod tests;
