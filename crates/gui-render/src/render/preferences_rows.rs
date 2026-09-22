//! Preferences row composition and continuous viewport clipping.
use super::*;

#[allow(clippy::too_many_arguments)]
pub(super) fn render_rows(
    dialog: &datum_gui_protocol::GlobalPreferencesDialogState,
    content_x: f32,
    content_width: f32,
    mut y: f32,
    card: RectPx,
    quads: &mut ControlPainter<'_>,
    text: &mut Vec<TextRun>,
    hits: &mut Vec<HitRegion>,
    scroll: &mut datum_gui_viewport::scroll::ScrollViewport,
    reveal_row: Option<usize>,
) {
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
    let viewport = datum_gui_viewport::ScreenRectPx {
        x: content_x,
        y,
        width: content_width,
        height: (card.y + card.height - y).max(0.0),
    };
    let mut starts = Vec::with_capacity(visible.len());
    let mut total = 0.0;
    let mut section = None;
    for row in &visible {
        let group = if !dialog.search_query.is_empty() && section != Some(row.section_id.as_str()) {
            22.0
        } else {
            0.0
        };
        section = Some(row.section_id.as_str());
        let choices = match &row.control {
            GlobalPreferenceControlUi::SingleChoice { choices, .. }
                if dialog.open_choice_key.as_deref() == Some(row.key.as_str()) =>
            {
                choices.len()
            }
            _ => 0,
        };
        let explanation = if dialog.explanation_key.as_deref() == Some(row.key.as_str()) {
            96.0
        } else {
            0.0
        };
        let height = 86.0 + choices as f32 * 28.0 + explanation;
        starts.push((total, group, height, choices, explanation));
        total += group + height;
    }
    scroll.layout(viewport, total);
    if let Some(index) = reveal_row {
        if index == 0 {
            scroll.set_offset(0.0);
        } else if let Some((top, group, height, _, _)) = starts.get(index) {
            scroll.reveal(*top, top + group + height);
        }
    }
    let body_bottom = viewport.y + viewport.height;
    let q_start = quads.len();
    let t_start = text.len();
    let h_start = hits.len();
    let content_width = content_width - if scroll.track().is_some() { 11.0 } else { 0.0 };
    for (row, (top, search_group_height, row_height, choice_count, explanation_height)) in
        visible.into_iter().zip(starts)
    {
        y = viewport.y + top - scroll.offset();
        if y + search_group_height + row_height <= viewport.y {
            continue;
        }
        if y >= body_bottom {
            break;
        }
        let explained = explanation_height > 0.0;
        let setting_height = 58.0;
        let provenance_height = 28.0;
        let choice_height = choice_count as f32 * 28.0;
        if search_group_height > 0.0 {
            let section_label = dialog
                .sections
                .iter()
                .find(|(id, _)| id == &row.section_id)
                .map(|(_, label)| label.as_str())
                .unwrap_or(row.section_id.as_str());
            draw_text(
                section_label,
                content_x + 4.0,
                y + 4.0,
                design_tokens::typography::CAPTION_SIZE,
                TEXT_ACCENT,
                TextFace::UiStrong,
                text,
            );
            y += search_group_height;
        }
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

        let reset = (row.changed && row.writable).then_some(RectPx {
            x: setting_rect.x + setting_rect.width - 80.0,
            y: setting_rect.y + 14.0,
            width: 62.0,
            height: 28.0,
        });
        let control_right = reset.map_or(setting_rect.x + setting_rect.width - 18.0, |rect| {
            rect.x - 10.0
        });
        let focused_control = focus_is(dialog, &GlobalPreferencesFocus::Control(row.key.clone()));
        let control = draw_preference_control(
            &row.control,
            control_right,
            setting_rect.y + 14.0,
            focused_control,
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
    }
    crate::hit_clipping::clip_content(
        quads,
        text,
        hits,
        q_start,
        t_start,
        h_start,
        RectPx {
            x: viewport.x,
            y: viewport.y,
            width: content_width,
            height: viewport.height,
        },
    );
    crate::global_preferences_primitives::paint_scrollbar(scroll, quads);
}
