//! Production Global Preferences dialog chrome for the engine-owned GP-F05 rows.

use super::*;

#[path = "render/preferences_rows.rs"]
mod preferences_rows;
#[path = "render/preferences_scene.rs"]
mod preferences_scene;
use crate::global_preferences_primitives::{
    ControlMeshCache, ControlPainter, button, draw_header_chip, draw_preference_control,
    draw_search_icon, push_rounded_rect_with_border,
};
use datum_gui_protocol::{
    GlobalPreferenceControlUi, GlobalPreferencesFocus, GlobalPreferencesNoticeUi,
};

pub(super) fn render_global_preferences_dialog(
    state: &ReviewWorkspaceState,
    layout: &ShellLayout,
    native_window: bool,
    quads: &mut Vec<Quad>,
    text: &mut Vec<TextRun>,
    hits: &mut Vec<HitRegion>,
) {
    let dialog = &state.ui.global_preferences;
    if !dialog.open || !native_window {
        new_project_dialog::render_new_project_dialog(
            state,
            layout,
            native_window,
            quads,
            text,
            hits,
        );
        return;
    }

    render_preferences_dialog(dialog, layout, quads, text, hits);
}

/// Render the shared Project/Global dialog directly, without a workspace scene.
pub(super) fn render_preferences_dialog(
    dialog: &datum_gui_protocol::GlobalPreferencesDialogState,
    layout: &ShellLayout,
    quads: &mut Vec<Quad>,
    text: &mut Vec<TextRun>,
    hits: &mut Vec<HitRegion>,
) {
    let mut scroll = datum_gui_viewport::scroll::ScrollViewport::default();
    let mut controls = ControlMeshCache::default();
    let mut painter = ControlPainter::new(quads, &mut controls, 1.0);
    render_preferences_dialog_scrolled(
        dialog,
        layout,
        &mut painter,
        text,
        hits,
        &mut scroll,
        Some(dialog.scroll_row),
    );
}

#[allow(clippy::too_many_arguments)]
fn render_preferences_dialog_scrolled(
    dialog: &datum_gui_protocol::GlobalPreferencesDialogState,
    layout: &ShellLayout,
    quads: &mut ControlPainter<'_>,
    text: &mut Vec<TextRun>,
    hits: &mut Vec<HitRegion>,
    scroll: &mut datum_gui_viewport::scroll::ScrollViewport,
    reveal_row: Option<usize>,
) {
    if !dialog.open {
        return;
    }

    let window = RectPx {
        x: 0.0,
        y: 0.0,
        width: layout.top_menu_bar.width,
        height: layout.status_bar.y + layout.status_bar.height,
    };
    quads.push(Quad::from_rect(window, design_tokens::chrome::BG_BASE));
    hits.push(HitRegion {
        target: HitTarget::GlobalPreferencesModal,
        rect: window,
    });

    let narrow = window.width < 800.0;
    let card = window;
    push_rect_border(
        quads,
        card,
        if dialog.high_contrast_noncolor {
            TEXT_PRIMARY
        } else {
            PANEL_CARD_BORDER
        },
        if dialog.high_contrast_noncolor {
            2.0
        } else {
            1.0
        },
    );

    let header = RectPx {
        x: card.x,
        y: card.y,
        width: card.width,
        height: 42.0,
    };
    quads.push(Quad::from_rect(header, design_tokens::chrome::SURFACE_01));
    quads.push(Quad::from_rect(
        RectPx {
            x: header.x,
            y: header.y + header.height - 1.0,
            width: header.width,
            height: 1.0,
        },
        design_tokens::chrome::BORDER_SUBTLE,
    ));
    let title_x = header.x + 14.0;
    draw_text(
        &dialog.title,
        title_x,
        header.y + 13.0,
        design_tokens::typography::BODY_SIZE,
        TEXT_PRIMARY,
        TextFace::UiStrong,
        text,
    );
    let title_width = measured_text_run_width_px(
        &dialog.title,
        design_tokens::typography::BODY_SIZE,
        TextFace::UiStrong,
    );
    let scope = draw_header_chip(
        &dialog.scope,
        title_x + title_width + 18.0,
        header.y + 11.0,
        quads,
        text,
    );
    draw_header_chip(
        "saves immediately",
        scope.x + scope.width + 8.0,
        header.y + 11.0,
        quads,
        text,
    );
    let modality = "input-modal";
    let modality_width = measured_text_run_width_px(
        modality,
        design_tokens::typography::MICRO_SIZE,
        TextFace::Mono,
    );
    draw_text(
        modality,
        header.x + header.width - modality_width - 14.0,
        header.y + 15.0,
        design_tokens::typography::MICRO_SIZE,
        TEXT_MUTED,
        TextFace::Mono,
        text,
    );

    let content_top = header.y + header.height;
    let rail_width = if narrow { card.width } else { 210.0 };
    let rail = RectPx {
        x: card.x,
        y: content_top,
        width: rail_width,
        height: if narrow {
            48.0
        } else {
            card.height - header.height
        },
    };
    quads.push(Quad::from_rect(rail, design_tokens::chrome::SURFACE_01));
    let separator = if narrow {
        RectPx {
            x: rail.x,
            y: rail.y + rail.height - 1.0,
            width: rail.width,
            height: 1.0,
        }
    } else {
        RectPx {
            x: rail.x + rail.width - 1.0,
            y: rail.y,
            width: 1.0,
            height: rail.height,
        }
    };
    quads.push(Quad::from_rect(
        separator,
        design_tokens::chrome::BORDER_SUBTLE,
    ));
    for (index, (section_id, section_label)) in dialog.sections.iter().enumerate() {
        let active = section_id == &dialog.section_id;
        let section = RectPx {
            x: rail.x + if narrow { index as f32 * 150.0 } else { 0.0 },
            y: rail.y
                + if narrow {
                    8.0
                } else {
                    12.0 + index as f32 * 36.0
                },
            width: if narrow { 150.0 } else { rail.width - 1.0 },
            height: 32.0,
        };
        if active {
            quads.push(Quad::from_rect(section, design_tokens::chrome::SURFACE_02));
            quads.push(Quad::from_rect(
                RectPx {
                    x: section.x,
                    y: section.y,
                    width: 2.0,
                    height: section.height,
                },
                TEXT_ACCENT,
            ));
        }
        if active && focus_is(dialog, &GlobalPreferencesFocus::SectionNavigation) {
            push_rect_border(quads, section, design_tokens::chrome::STATUS_INFO, 2.0);
        }
        draw_text(
            section_label,
            section.x + 18.0,
            section.y + 8.0,
            design_tokens::typography::BODY_SIZE,
            if active { TEXT_PRIMARY } else { TEXT_SECONDARY },
            TextFace::Ui,
            text,
        );
        hits.push(HitRegion {
            target: HitTarget::GlobalPreferencesSection(section_id.clone()),
            rect: section,
        });
    }

    let content_x = if narrow { card.x } else { rail.x + rail.width };
    let content_y = if narrow {
        rail.y + rail.height
    } else {
        content_top
    };
    let content_width = card.x + card.width - content_x;
    let search_band = RectPx {
        x: content_x,
        y: content_y,
        width: content_width,
        height: 58.0,
    };
    quads.push(Quad::from_rect(search_band, design_tokens::chrome::BG_BASE));
    quads.push(Quad::from_rect(
        RectPx {
            x: search_band.x,
            y: search_band.y + search_band.height - 1.0,
            width: search_band.width,
            height: 1.0,
        },
        design_tokens::chrome::BORDER_SUBTLE,
    ));
    let search = RectPx {
        x: search_band.x + 18.0,
        y: search_band.y + 12.0,
        width: search_band.width - 36.0,
        height: 35.0,
    };
    let search_input_x = search.x + 30.0;
    let search_text_x = search_input_x
        + if dialog.search_query.is_empty() && focus_is(dialog, &GlobalPreferencesFocus::Search) {
            6.0
        } else {
            0.0
        };
    push_rounded_rect_with_border(
        quads,
        search,
        design_tokens::chrome::SURFACE_02,
        if focus_is(dialog, &GlobalPreferencesFocus::Search) {
            design_tokens::chrome::STATUS_INFO
        } else {
            design_tokens::chrome::BORDER_STRONG
        },
        if focus_is(dialog, &GlobalPreferencesFocus::Search) {
            2.0
        } else {
            1.0
        },
        design_tokens::radius::MD,
    );
    draw_search_icon(search.x + 12.0, search.y + 12.0, quads);
    draw_text(
        if dialog.search_query.is_empty() {
            if dialog.section_id == "units" {
                "search Units settings…"
            } else {
                "search Appearance settings…"
            }
        } else {
            &dialog.search_query
        },
        search_text_x,
        search.y + 10.0,
        design_tokens::typography::BODY_SIZE,
        if dialog.search_query.is_empty() {
            TEXT_MUTED
        } else {
            TEXT_PRIMARY
        },
        TextFace::Ui,
        text,
    );
    if focus_is(dialog, &GlobalPreferencesFocus::Search) {
        let caret_x = if dialog.search_query.is_empty() {
            search_input_x
        } else {
            (search_input_x
                + measured_text_run_width_px(
                    &dialog.search_query,
                    design_tokens::typography::BODY_SIZE,
                    TextFace::Ui,
                )
                + 1.0)
                .min(search.x + search.width - 13.0)
        };
        quads.push(Quad::from_rect(
            RectPx {
                x: caret_x,
                y: search.y + 8.0,
                width: 1.5,
                height: 19.0,
            },
            TEXT_PRIMARY,
        ));
    }
    hits.push(HitRegion {
        target: HitTarget::GlobalPreferencesSearch,
        rect: search,
    });

    let mut y = search_band.y + search_band.height;
    if let Some(notice) = &dialog.notice {
        let (lines, color) = match notice {
            GlobalPreferencesNoticeUi::Polite(message) => {
                (vec![truncate_text(message, 108)], TEXT_SECONDARY)
            }
            GlobalPreferencesNoticeUi::Assertive(message) => (
                vec![truncate_text(message, 108)],
                design_tokens::chrome::STATUS_ERROR,
            ),
            GlobalPreferencesNoticeUi::PreservedUnreadable(_) => (
                vec![
                    "Preferences could not be read. Factory defaults are active for this session."
                        .to_owned(),
                    "Damaged data remains preserved; controls are unavailable.".to_owned(),
                ],
                design_tokens::chrome::STATUS_WARN,
            ),
        };
        let notice_rect = RectPx {
            x: content_x,
            y,
            width: content_width,
            height: if lines.len() > 1 { 58.0 } else { 48.0 },
        };
        quads.push(Quad::from_rect(
            notice_rect,
            design_tokens::chrome::SURFACE_02,
        ));
        push_rect_border(quads, notice_rect, color, 1.0);
        for (index, line) in lines.iter().enumerate() {
            draw_text(
                line,
                notice_rect.x + 10.0,
                notice_rect.y + 12.0 + index as f32 * 18.0,
                design_tokens::typography::CAPTION_SIZE,
                color,
                TextFace::Ui,
                text,
            );
        }
        y += notice_rect.height;
    }

    preferences_rows::render_rows(
        dialog,
        content_x,
        content_width,
        y,
        card,
        quads,
        text,
        hits,
        scroll,
        reveal_row,
    );
}

fn focus_is(
    dialog: &datum_gui_protocol::GlobalPreferencesDialogState,
    candidate: &GlobalPreferencesFocus,
) -> bool {
    &dialog.focus == candidate
}
