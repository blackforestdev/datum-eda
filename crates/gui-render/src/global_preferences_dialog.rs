//! Production Global Preferences dialog chrome for the engine-owned GP-F05 rows.

use super::*;
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
    quads.push(Quad::from_rect(card, PANEL_CARD_BG));
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

    let pad = design_tokens::spacing::SP_05;
    draw_text(
        "Global Preferences — Datum",
        card.x + pad,
        card.y + pad,
        design_tokens::typography::DISPLAY_SIZE,
        TEXT_PRIMARY,
        TextFace::UiStrong,
        text,
    );
    draw_text(
        "Global · this device · Changes save immediately",
        card.x + pad,
        card.y + pad + 24.0,
        design_tokens::typography::CAPTION_SIZE,
        TEXT_SECONDARY,
        TextFace::Mono,
        text,
    );
    let content_top = card.y + 68.0;
    let rail_width = if narrow { 0.0 } else { 184.0 };
    let rail = RectPx {
        x: card.x + pad,
        y: content_top,
        width: if narrow {
            card.width - pad * 2.0
        } else {
            rail_width
        },
        height: if narrow { 42.0 } else { card.height - 88.0 },
    };
    quads.push(Quad::from_rect(rail, REVIEW_ROW_ACTIVE_BG));
    push_rect_border(
        quads,
        rail,
        if focus_is(dialog, &GlobalPreferencesFocus::SectionNavigation) {
            TEXT_ACCENT
        } else {
            PANEL_CARD_BORDER
        },
        1.0,
    );
    draw_text(
        if narrow {
            "Section: Appearance  v"
        } else {
            "Appearance"
        },
        rail.x + 12.0,
        rail.y + 13.0,
        design_tokens::typography::BODY_SIZE,
        TEXT_PRIMARY,
        TextFace::UiStrong,
        text,
    );
    hits.push(HitRegion {
        target: HitTarget::GlobalPreferencesSection,
        rect: rail,
    });

    let content_x = if narrow {
        card.x + pad
    } else {
        rail.x + rail.width + pad
    };
    let content_y = if narrow {
        rail.y + rail.height + 12.0
    } else {
        content_top
    };
    let content_width = card.x + card.width - pad - content_x;
    let search = RectPx {
        x: content_x,
        y: content_y,
        width: content_width,
        height: 38.0,
    };
    quads.push(Quad::from_rect(search, design_tokens::chrome::SURFACE_02));
    push_rect_border(
        quads,
        search,
        if focus_is(dialog, &GlobalPreferencesFocus::Search) {
            TEXT_ACCENT
        } else {
            PANEL_CARD_BORDER
        },
        1.0,
    );
    draw_text(
        if dialog.search_query.is_empty() {
            "Search Appearance settings"
        } else {
            &dialog.search_query
        },
        search.x + 12.0,
        search.y + 11.0,
        design_tokens::typography::BODY_SIZE,
        if dialog.search_query.is_empty() {
            TEXT_MUTED
        } else {
            TEXT_PRIMARY
        },
        TextFace::Ui,
        text,
    );
    hits.push(HitRegion {
        target: HitTarget::GlobalPreferencesSearch,
        rect: search,
    });

    let mut y = search.y + search.height + 12.0;
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
        y += notice_rect.height + 8.0;
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
        let row_height = if explained {
            184.0
        } else if choice_count > 0 {
            66.0 + choice_count as f32 * 28.0
        } else {
            94.0
        };
        let row_rect = RectPx {
            x: content_x,
            y,
            width: content_width,
            height: row_height,
        };
        quads.push(Quad::from_rect(row_rect, design_tokens::chrome::SURFACE_01));
        push_rect_border(quads, row_rect, PANEL_CARD_BORDER, 1.0);

        let name_rect = RectPx {
            x: row_rect.x + 10.0,
            y: row_rect.y + 8.0,
            width: (row_rect.width - 190.0).max(130.0),
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
            push_rect_border(quads, name_rect, TEXT_ACCENT, 1.0);
        }
        hits.push(HitRegion {
            target: HitTarget::GlobalPreferencesSettingName(row.key.clone()),
            rect: name_rect,
        });
        draw_text(
            &truncate_text(&row.description, 86),
            row_rect.x + 10.0,
            row_rect.y + 37.0,
            design_tokens::typography::CAPTION_SIZE,
            TEXT_SECONDARY,
            TextFace::Ui,
            text,
        );
        draw_text(
            &row.provenance,
            row_rect.x + 10.0,
            row_rect.y + 63.0,
            design_tokens::typography::CAPTION_SIZE,
            if row.writable {
                TEXT_MUTED
            } else {
                design_tokens::chrome::STATUS_WARN
            },
            TextFace::Mono,
            text,
        );

        let control = RectPx {
            x: row_rect.x + row_rect.width - 166.0,
            y: row_rect.y + 18.0,
            width: 148.0,
            height: 34.0,
        };
        let value_label = match &row.control {
            GlobalPreferenceControlUi::Boolean {
                value,
                off_label,
                on_label,
            } => {
                format!(
                    "{}  {}",
                    if *value { "[x]" } else { "[ ]" },
                    if *value { on_label } else { off_label }
                )
            }
            GlobalPreferenceControlUi::SingleChoice { value, choices } => {
                format!(
                    "{}  v",
                    choices
                        .iter()
                        .find(|(candidate, _)| candidate == value)
                        .map(|(_, label)| label.as_str())
                        .unwrap_or(value)
                )
            }
        };
        let visible_value_label = if row.writable {
            value_label
        } else {
            format!("{value_label} -- unavailable")
        };
        button(
            &visible_value_label,
            control,
            focus_is(dialog, &GlobalPreferencesFocus::Control(row.key.clone())),
            row.writable,
            quads,
            text,
        );
        if row.writable {
            hits.push(HitRegion {
                target: HitTarget::GlobalPreferencesControl(row.key.clone()),
                rect: control,
            });
        }
        if choice_count > 0
            && let GlobalPreferenceControlUi::SingleChoice { value, choices } = &row.control
        {
            for (index, (choice_value, choice_label)) in choices.iter().enumerate() {
                let choice = RectPx {
                    x: control.x,
                    y: control.y + control.height + index as f32 * 28.0,
                    width: control.width,
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
        if row.changed && choice_count == 0 {
            let reset = RectPx {
                x: control.x + 70.0,
                y: control.y + 40.0,
                width: 78.0,
                height: 26.0,
            };
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
        if explained {
            let explanation = RectPx {
                x: row_rect.x + 10.0,
                y: row_rect.y + 91.0,
                width: row_rect.width - 20.0,
                height: 82.0,
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
        y += row_height + 8.0;
    }
}

fn focus_is(
    dialog: &datum_gui_protocol::GlobalPreferencesDialogState,
    candidate: &GlobalPreferencesFocus,
) -> bool {
    &dialog.focus == candidate
}

fn button(
    label: &str,
    rect: RectPx,
    focused: bool,
    available: bool,
    quads: &mut Vec<Quad>,
    text: &mut Vec<TextRun>,
) {
    quads.push(Quad::from_rect(rect, design_tokens::chrome::SURFACE_03));
    let border = if focused {
        TEXT_ACCENT
    } else {
        PANEL_CARD_BORDER
    };
    if available {
        push_rect_border(quads, rect, border, if focused { 2.0 } else { 1.0 });
    } else {
        push_dashed_rect_border(quads, rect, border);
    }
    draw_text(
        label,
        rect.x + 9.0,
        rect.y + (rect.height - design_tokens::typography::CAPTION_SIZE) * 0.5 - 1.0,
        design_tokens::typography::CAPTION_SIZE,
        if available { TEXT_PRIMARY } else { TEXT_MUTED },
        TextFace::Ui,
        text,
    );
}

fn push_dashed_rect_border(quads: &mut Vec<Quad>, rect: RectPx, color: [f32; 3]) {
    let dash: f32 = 6.0;
    let gap: f32 = 4.0;
    let mut horizontal = |y: f32| {
        let mut x = rect.x;
        while x < rect.x + rect.width {
            quads.push(Quad::from_rect(
                RectPx {
                    x,
                    y,
                    width: dash.min(rect.x + rect.width - x),
                    height: 1.0,
                },
                color,
            ));
            x += dash + gap;
        }
    };
    horizontal(rect.y);
    horizontal(rect.y + rect.height - 1.0);
    let mut y = rect.y;
    while y < rect.y + rect.height {
        let height = dash.min(rect.y + rect.height - y);
        for x in [rect.x, rect.x + rect.width - 1.0] {
            quads.push(Quad::from_rect(
                RectPx {
                    x,
                    y,
                    width: 1.0,
                    height,
                },
                color,
            ));
        }
        y += dash + gap;
    }
}
