//! Production Global Preferences dialog chrome for the engine-owned GP-F05 rows.

use super::*;
use crate::global_preferences_primitives::{
    button, draw_boolean_control, draw_choice_control, draw_header_chip, draw_search_icon,
    push_rounded_rect_with_border,
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
        "Global Preferences — Datum",
        title_x,
        header.y + 13.0,
        design_tokens::typography::BODY_SIZE,
        TEXT_PRIMARY,
        TextFace::UiStrong,
        text,
    );
    let title_width = measured_text_run_width_px(
        "Global Preferences — Datum",
        design_tokens::typography::BODY_SIZE,
        TextFace::UiStrong,
    );
    let scope = draw_header_chip(
        "Global · this device",
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
    let section = RectPx {
        x: rail.x,
        y: rail.y + if narrow { 8.0 } else { 12.0 },
        width: rail.width - if narrow { 0.0 } else { 1.0 },
        height: 32.0,
    };
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
    if focus_is(dialog, &GlobalPreferencesFocus::SectionNavigation) {
        push_rect_border(quads, section, design_tokens::chrome::STATUS_INFO, 2.0);
    }
    draw_text(
        if narrow {
            "Appearance  v"
        } else {
            "Appearance"
        },
        section.x + 18.0,
        section.y + 8.0,
        design_tokens::typography::BODY_SIZE,
        TEXT_PRIMARY,
        TextFace::Ui,
        text,
    );
    hits.push(HitRegion {
        target: HitTarget::GlobalPreferencesSection,
        rect: section,
    });

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
            "search settings…"
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

    let visible: Vec<_> = dialog.visible_rows().collect();
    if !dialog.search_query.is_empty() {
        draw_text(
            &format!(
                "{} result{}",
                visible.len(),
                if visible.len() == 1 { "" } else { "s" }
            ),
            content_x,
            y,
            design_tokens::typography::CAPTION_SIZE,
            TEXT_SECONDARY,
            TextFace::Ui,
            text,
        );
        y += 22.0;
    }
    for row in visible {
        let explained = dialog.explanation_key.as_deref() == Some(row.key.as_str());
        let choice_count = match &row.control {
            GlobalPreferenceControlUi::SingleChoice { choices, .. }
                if dialog.open_choice_key.as_deref() == Some(row.key.as_str()) =>
            {
                choices.len()
            }
            _ => 0,
        };
        let setting_height = 58.0;
        let provenance_height = 28.0;
        let choice_height = choice_count as f32 * 28.0;
        let explanation_height = if explained { 96.0 } else { 0.0 };
        let row_height = setting_height + provenance_height + choice_height + explanation_height;
        let row_rect = RectPx {
            x: content_x,
            y,
            width: content_width,
            height: row_height,
        };
        let setting_rect = RectPx {
            x: row_rect.x,
            y: row_rect.y,
            width: row_rect.width,
            height: setting_height,
        };
        quads.push(Quad::from_rect(
            setting_rect,
            design_tokens::chrome::BG_BASE,
        ));

        let name_rect = RectPx {
            x: setting_rect.x + 18.0,
            y: setting_rect.y + 7.0,
            width: (setting_rect.width - 250.0).max(130.0),
            height: 24.0,
        };
        draw_text(
            &row.label,
            name_rect.x,
            name_rect.y + 4.0,
            design_tokens::typography::BODY_SIZE,
            TEXT_PRIMARY,
            TextFace::UiStrong,
            text,
        );
        if focus_is(
            dialog,
            &GlobalPreferencesFocus::SettingName(row.key.clone()),
        ) {
            push_rect_border(quads, name_rect, design_tokens::chrome::STATUS_INFO, 2.0);
        }
        hits.push(HitRegion {
            target: HitTarget::GlobalPreferencesSettingName(row.key.clone()),
            rect: name_rect,
        });
        draw_text(
            &truncate_text(&row.description, 86),
            setting_rect.x + 18.0,
            setting_rect.y + 32.0,
            design_tokens::typography::CAPTION_SIZE,
            TEXT_MUTED,
            TextFace::Ui,
            text,
        );

        let reset = row.changed.then_some(RectPx {
            x: setting_rect.x + setting_rect.width - 80.0,
            y: setting_rect.y + 14.0,
            width: 62.0,
            height: 28.0,
        });
        let control_right = reset.map_or(setting_rect.x + setting_rect.width - 18.0, |rect| {
            rect.x - 10.0
        });
        let focused_control = focus_is(dialog, &GlobalPreferencesFocus::Control(row.key.clone()));
        let control = match &row.control {
            GlobalPreferenceControlUi::Boolean {
                value,
                off_label,
                on_label,
            } => {
                let rect = RectPx {
                    x: control_right - if row.writable { 78.0 } else { 138.0 },
                    y: setting_rect.y + 14.0,
                    width: if row.writable { 78.0 } else { 138.0 },
                    height: 28.0,
                };
                draw_boolean_control(
                    *value,
                    if *value { on_label } else { off_label },
                    rect,
                    focused_control,
                    row.writable,
                    quads,
                    text,
                );
                rect
            }
            GlobalPreferenceControlUi::SingleChoice { value, choices } => {
                let label = choices
                    .iter()
                    .find(|(candidate, _)| candidate == value)
                    .map(|(_, label)| label.as_str())
                    .unwrap_or(value);
                let width = if row.writable {
                    (measured_text_run_width_px(
                        label,
                        design_tokens::typography::CAPTION_SIZE,
                        TextFace::Ui,
                    ) + 34.0)
                        .max(58.0)
                } else {
                    138.0
                };
                let rect = RectPx {
                    x: control_right - width,
                    y: setting_rect.y + 14.0,
                    width,
                    height: 30.0,
                };
                draw_choice_control(label, rect, focused_control, row.writable, quads, text);
                rect
            }
        };
        if row.writable {
            hits.push(HitRegion {
                target: HitTarget::GlobalPreferencesControl(row.key.clone()),
                rect: control,
            });
        }
        if let Some(reset) = reset {
            button(
                "Reset",
                reset,
                focus_is(dialog, &GlobalPreferencesFocus::Reset(row.key.clone())),
                true,
                quads,
                text,
            );
            hits.push(HitRegion {
                target: HitTarget::GlobalPreferencesReset(row.key.clone()),
                rect: reset,
            });
        }

        let provenance = RectPx {
            x: row_rect.x,
            y: row_rect.y + setting_height + choice_height,
            width: row_rect.width,
            height: provenance_height,
        };
        if row.changed {
            let center_x = provenance.x + 22.0;
            let center_y = provenance.y + 11.0;
            quads.push(Quad {
                points: [
                    (center_x, center_y - 3.0),
                    (center_x + 3.0, center_y),
                    (center_x, center_y + 3.0),
                    (center_x - 3.0, center_y),
                ],
                color: TEXT_ACCENT,
            });
        }
        draw_text(
            &row.provenance,
            provenance.x + if row.changed { 32.0 } else { 18.0 },
            provenance.y + 5.0,
            design_tokens::typography::CAPTION_SIZE,
            if row.writable {
                TEXT_MUTED
            } else {
                design_tokens::chrome::STATUS_WARN
            },
            TextFace::Mono,
            text,
        );
        quads.push(Quad::from_rect(
            RectPx {
                x: provenance.x,
                y: provenance.y + provenance.height - 1.0,
                width: provenance.width,
                height: 1.0,
            },
            design_tokens::chrome::BORDER_SUBTLE,
        ));

        if choice_count > 0
            && let GlobalPreferenceControlUi::SingleChoice { value, choices } = &row.control
        {
            for (index, (choice_value, choice_label)) in choices.iter().enumerate() {
                let choice = RectPx {
                    x: control_right - 148.0,
                    y: setting_rect.y + setting_height + index as f32 * 28.0,
                    width: 148.0,
                    height: 28.0,
                };
                quads.push(Quad::from_rect(
                    choice,
                    if choice_value == value {
                        REVIEW_ROW_ACTIVE_BG
                    } else {
                        design_tokens::chrome::SURFACE_03
                    },
                ));
                push_rect_border(
                    quads,
                    choice,
                    if choice_value == value {
                        TEXT_ACCENT
                    } else {
                        PANEL_CARD_BORDER
                    },
                    1.0,
                );
                draw_text(
                    &format!(
                        "{} {}",
                        if choice_value == value { "(*)" } else { "( )" },
                        choice_label
                    ),
                    choice.x + 8.0,
                    choice.y + 8.0,
                    design_tokens::typography::CAPTION_SIZE,
                    TEXT_PRIMARY,
                    TextFace::Ui,
                    text,
                );
                hits.push(HitRegion {
                    target: HitTarget::GlobalPreferencesChoice {
                        key: row.key.clone(),
                        value: choice_value.clone(),
                    },
                    rect: choice,
                });
            }
        }
        if explained {
            let explanation = RectPx {
                x: row_rect.x + 18.0,
                y: provenance.y + provenance.height + 8.0,
                width: row_rect.width - 36.0,
                height: explanation_height - 16.0,
            };
            quads.push(Quad::from_rect(
                explanation,
                design_tokens::chrome::SURFACE_02,
            ));
            for (index, line) in row.explanation_lines.iter().take(4).enumerate() {
                draw_text(
                    &truncate_text(line, 94),
                    explanation.x + 10.0,
                    explanation.y + 8.0 + index as f32 * 16.0,
                    design_tokens::typography::CAPTION_SIZE,
                    TEXT_SECONDARY,
                    TextFace::Mono,
                    text,
                );
            }
            let close_explanation = RectPx {
                x: explanation.x + explanation.width - 58.0,
                y: explanation.y + explanation.height - 24.0,
                width: 50.0,
                height: 20.0,
            };
            button(
                "Close",
                close_explanation,
                focus_is(dialog, &GlobalPreferencesFocus::ExplanationClose),
                true,
                quads,
                text,
            );
            hits.push(HitRegion {
                target: HitTarget::GlobalPreferencesExplanationClose,
                rect: close_explanation,
            });
        }
        y += row_height;
    }
}

fn focus_is(
    dialog: &datum_gui_protocol::GlobalPreferencesDialogState,
    candidate: &GlobalPreferencesFocus,
) -> bool {
    &dialog.focus == candidate
}
