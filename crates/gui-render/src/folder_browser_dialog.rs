//! Shared inline directory-list and complete-path preview widgets.
use super::*;
use datum_gui_protocol::{
    FolderBrowserFocus as Focus, FolderBrowserMode as Mode, FolderBrowserState,
};
use datum_gui_viewport::{ScreenRectPx, scroll::ScrollViewport};
#[path = "folder_suggestion_chrome.rs"]
mod suggestions;
pub(super) use suggestions::suggestion_header_lines;

#[allow(clippy::too_many_arguments)]
pub(super) fn paint_location(
    nav: Option<&FolderBrowserState>,
    focus: Option<Focus>,
    bounds: RectPx,
    enabled: bool,
    controls: &mut ControlMeshCache,
    scale: f32,
    quads: &mut Vec<Quad>,
    text: &mut Vec<TextRun>,
    hits: &mut Vec<HitRegion>,
) -> anyhow::Result<()> {
    let mut painter = ControlPainter::new(quads, controls, scale);
    draw_text(
        "Location",
        18.0,
        bounds.y + 10.0,
        design_tokens::typography::CAPTION_SIZE,
        TEXT_SECONDARY,
        TextFace::UiStrong,
        text,
    );
    let width = measured_text_run_width_px(
        "Browse…",
        design_tokens::typography::CAPTION_SIZE,
        TextFace::Ui,
    )? + 28.0;
    let browse = RectPx {
        x: bounds.x + bounds.width - width,
        width,
        ..bounds
    };
    crate::single_line_control::paint(
        nav.map(|n| n.path_input.as_str()).unwrap_or(""),
        "Enter Location",
        nav.map(|n| n.path_editor).unwrap_or_default(),
        focus == Some(Focus::Path),
        enabled,
        RectPx {
            width: (bounds.width - width - 8.0).max(1.0),
            ..bounds
        },
        &mut painter,
        text,
        hits,
        HitTarget::FolderBrowser(Focus::Path),
    )?;
    button(
        "Browse…",
        browse,
        focus == Some(Focus::Browse),
        enabled,
        &mut painter,
        text,
    );
    hits.push(HitRegion {
        target: HitTarget::FolderBrowser(Focus::Browse),
        rect: browse,
    });
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub(super) fn paint_navigation(
    nav: Option<&FolderBrowserState>,
    _mode: Mode,
    focus: Option<Focus>,
    bounds: RectPx,
    list_height: f32,
    scroll: &mut ScrollViewport,
    reveal: bool,
    enabled: bool,
    controls: &mut ControlMeshCache,
    scale: f32,
    quads: &mut Vec<Quad>,
    text: &mut Vec<TextRun>,
    hits: &mut Vec<HitRegion>,
) -> anyhow::Result<()> {
    if !nav.is_some_and(|n| n.expanded) {
        return Ok(());
    }
    let mut painter = ControlPainter::new(quads, controls, scale);
    for (label, control, x, width) in [
        ("↑ Up", Focus::Up, bounds.x, 48.0),
        ("Home", Focus::Home, bounds.x + 54.0, 54.0),
    ] {
        let rect = RectPx {
            x,
            y: bounds.y,
            width,
            height: 30.0,
        };
        button(
            label,
            rect,
            focus == Some(control),
            enabled,
            &mut painter,
            text,
        );
        hits.push(HitRegion {
            target: HitTarget::FolderBrowser(control),
            rect,
        });
    }
    let query_header = suggestion_header_lines(nav, bounds.width)?;
    for (i, line) in query_header.iter().enumerate() {
        draw_text(
            line,
            bounds.x,
            bounds.y + 38.0 + i as f32 * 16.0,
            design_tokens::typography::CAPTION_SIZE,
            TEXT_MUTED,
            TextFace::Mono,
            text,
        );
    }
    let list = RectPx {
        x: bounds.x,
        y: bounds.y
            + 38.0
            + if query_header.is_empty() {
                0.0
            } else {
                query_header.len() as f32 * 16.0 + 8.0
            },
        width: bounds.width,
        height: list_height,
    };
    painter.push(Quad::from_rect(list, design_tokens::chrome::SURFACE_01));
    push_rect_border(
        &mut painter,
        list,
        if focus == Some(Focus::Folders) {
            design_tokens::chrome::STATUS_INFO
        } else {
            design_tokens::chrome::BORDER_STRONG
        },
        1.0,
    );
    hits.push(HitRegion {
        target: HitTarget::FolderBrowser(Focus::Folders),
        rect: list,
    });
    let count = nav.map(|n| n.entries.len()).unwrap_or(0);
    scroll.layout(
        ScreenRectPx {
            x: list.x,
            y: list.y,
            width: list.width,
            height: list.height,
        },
        count as f32 * 24.0,
    );
    if reveal && let Some(index) = nav.and_then(|n| n.selected) {
        scroll.reveal(index as f32 * 24.0, (index + 1) as f32 * 24.0);
    }
    let starts = (painter.len(), text.len(), hits.len());
    if let Some(nav) = nav {
        if nav.loading || nav.error.is_some() || nav.entries.is_empty() {
            let empty = if nav.loading {
                "Reading folder…".into()
            } else if nav.error.is_some() {
                "This folder could not be read".into()
            } else {
                nav.suggestion_empty_message()
                    .unwrap_or_else(|| "No folders here".into())
            };
            for (i, line) in wrap_text(&empty, (list.width - 20.0).max(1.0))?
                .iter()
                .enumerate()
            {
                draw_text(
                    line,
                    list.x + 10.0,
                    list.y + 14.0 + i as f32 * 17.0,
                    design_tokens::typography::CAPTION_SIZE,
                    TEXT_MUTED,
                    TextFace::Ui,
                    text,
                );
            }
        } else {
            let first = (scroll.offset() / 24.0).floor() as usize;
            for (index, entry) in nav
                .entries
                .iter()
                .enumerate()
                .skip(first)
                .take((list.height / 24.0).ceil() as usize + 1)
            {
                let row = RectPx {
                    x: list.x + 1.0,
                    y: list.y + index as f32 * 24.0 - scroll.offset(),
                    width: (list.width - 12.0).max(1.0),
                    height: 24.0,
                };
                if nav.selected == Some(index) {
                    painter.push(Quad::from_rect(row, design_tokens::chrome::SURFACE_02));
                    push_rect_border(&mut painter, row, design_tokens::chrome::STATUS_INFO, 1.0);
                }
                let label = if entry.navigable() {
                    format!("{}/", entry.name)
                } else {
                    entry.name.clone()
                };
                suggestions::label(
                    &tail_text(
                        &label,
                        row.width - if entry.navigable() { 16.0 } else { 58.0 },
                    ),
                    nav.suggestions.as_ref().map(|q| q.fragment.as_str()),
                    row.x + 8.0,
                    row.y + 6.0,
                    if entry.navigable() {
                        TEXT_PRIMARY
                    } else {
                        TEXT_SECONDARY
                    },
                    text,
                )?;
                if !entry.navigable() {
                    draw_text(
                        entry.kind_label(),
                        row.x + row.width - 40.0,
                        row.y + 6.0,
                        design_tokens::typography::MICRO_SIZE,
                        TEXT_MUTED,
                        TextFace::Ui,
                        text,
                    );
                }
                hits.push(HitRegion {
                    target: HitTarget::FolderBrowserRow(index),
                    rect: row,
                });
            }
        }
    } else {
        draw_text(
            "Folder navigation unavailable",
            list.x + 10.0,
            list.y + 14.0,
            design_tokens::typography::CAPTION_SIZE,
            TEXT_MUTED,
            TextFace::Ui,
            text,
        );
    }
    drop(painter);
    crate::hit_clipping::clip_content(quads, text, hits, starts.0, starts.1, starts.2, list);
    crate::global_preferences_primitives::paint_scrollbar(scroll, quads);
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub(super) fn paint_preview(
    nav: Option<&FolderBrowserState>,
    mode: Mode,
    leaf: Option<&str>,
    full_refusal: Option<&str>,
    focus: Option<Focus>,
    bounds: RectPx,
    scroll: &mut ScrollViewport,
    controls: &mut ControlMeshCache,
    scale: f32,
    quads: &mut Vec<Quad>,
    text: &mut Vec<TextRun>,
    hits: &mut Vec<HitRegion>,
) -> anyhow::Result<()> {
    let mut value = nav.and_then(|n| n.preview(leaf)).unwrap_or_else(|| {
        if mode == Mode::New {
            "Type a valid folder name".into()
        } else {
            "none".into()
        }
    });
    if mode == Mode::Open
        && nav.is_some_and(|n| {
            n.expanded
                && n.ready()
                && !n
                    .selected
                    .and_then(|i| n.entries.get(i))
                    .is_some_and(datum_gui_protocol::BrowserEntry::navigable)
        })
    {
        value.push_str(" (this folder)");
    }
    let mut lines = wrap_text(&value, (bounds.width - 24.0).max(1.0))?;
    lines.extend(wrap_text(
        if mode == Mode::New && nav.is_some_and(|n| n.suggestions.is_some()) {
            "Exactly what Location says now — Create checks it when pressed."
        } else if mode == Mode::New {
            "Created only when you press Create."
        } else {
            "Open checks the Project before opening it."
        },
        (bounds.width - 24.0).max(1.0),
    )?);
    if let Some(reason) = full_refusal {
        lines.push(String::new());
        lines.push("Full refusal details".into());
        lines.extend(wrap_text(reason, (bounds.width - 24.0).max(1.0))?);
    }
    scroll.layout(
        ScreenRectPx {
            x: bounds.x,
            y: bounds.y,
            width: bounds.width,
            height: bounds.height,
        },
        lines.len() as f32 * 18.0 + 12.0,
    );
    let mut painter = ControlPainter::new(quads, controls, scale);
    push_rect_border(
        &mut painter,
        bounds,
        if focus == Some(Focus::Preview) {
            design_tokens::chrome::STATUS_INFO
        } else {
            design_tokens::chrome::BORDER_STRONG
        },
        1.0,
    );
    hits.push(HitRegion {
        target: HitTarget::FolderBrowser(Focus::Preview),
        rect: bounds,
    });
    let starts = (painter.len(), text.len(), hits.len());
    let first = (scroll.offset() / 18.0).floor() as usize;
    for (index, line) in lines
        .iter()
        .enumerate()
        .skip(first)
        .take((bounds.height / 18.0).ceil() as usize + 1)
    {
        draw_text(
            line,
            bounds.x + 6.0,
            bounds.y + 6.0 + index as f32 * 18.0 - scroll.offset(),
            design_tokens::typography::CAPTION_SIZE,
            TEXT_PRIMARY,
            TextFace::Ui,
            text,
        );
    }
    drop(painter);
    crate::hit_clipping::clip_content(quads, text, hits, starts.0, starts.1, starts.2, bounds);
    crate::global_preferences_primitives::paint_scrollbar(scroll, quads);
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub(super) use crate::control_text::{tail_text, wrap_text, wrap_text_face};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn folder_browser_full_path_wrap_keeps_every_character_and_both_ends() {
        let path = format!(
            "New Project folder: /home/user/{}/explicit leaf λ",
            "long parent directory/".repeat(25)
        );
        for face in [TextFace::Ui, TextFace::UiStrong] {
            for width in [220.0, 700.0] {
                let lines = wrap_text_face(&path, width, face).unwrap();
                assert_eq!(lines.concat(), path);
                assert!(lines.len() > 1);
                for line in lines {
                    assert!(
                        measured_text_run_width_px(
                            &line,
                            design_tokens::typography::CAPTION_SIZE,
                            face
                        )
                        .unwrap()
                            <= width
                    );
                }
            }
        }
    }
}
