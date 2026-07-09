/// Generic selection-identity band (Design Book insp-title): identity primary
/// line, muted mono kind subtitle, a right-aligned accent-bordered SELECTED
/// pill, and a BORDER_SUBTLE bottom divider. This is selection chrome that
/// applies to the routing-review ACTION selection too — not the deferred
/// populated-component inspector.
fn push_inspector_title_band(
    rect: RectPx,
    identity: &str,
    kind: &str,
    show_pill: bool,
    panel_quads: &mut Vec<Quad>,
    text_runs: &mut Vec<TextRun>,
) {
    let band_top = rect.y + 34.0;
    draw_text(
        identity,
        rect.x + 12.0,
        band_top,
        16.0,
        TEXT_PRIMARY,
        TextFace::UiStrong,
        text_runs,
    );
    if !kind.is_empty() {
        draw_text(
            kind,
            rect.x + 12.0,
            band_top + 18.0,
            11.5,
            TEXT_MUTED,
            TextFace::Mono,
            text_runs,
        );
    }
    if show_pill {
        let label = "SELECTED";
        let pad_x = 7.0_f32;
        let text_w =
            estimated_text_run_width_px(label, 10.0, TextFace::UiMedium) - 16.0;
        let pill = RectPx {
            x: rect.x + rect.width - 12.0 - (text_w + pad_x * 2.0),
            y: band_top - 1.0,
            width: text_w + pad_x * 2.0,
            height: 16.0,
        };
        push_rect_border(panel_quads, pill, TEXT_ACCENT, 1.0);
        draw_text(
            label,
            pill.x + pad_x,
            pill.y + 3.0,
            10.0,
            TEXT_ACCENT,
            TextFace::UiMedium,
            text_runs,
        );
    }
    push_section_divider(
        panel_quads,
        rect.x,
        rect.y + 66.0,
        rect.width,
        PANEL_CARD_BORDER,
    );
}

/// Uppercase section-header strip (Design Book sect-hd): a SURFACE_01 band with
/// a top BORDER_SUBTLE hairline and an ~11px semibold TEXT_MUTED label. Generic
/// chrome; the named Identity/Placement/Checks component sections stay deferred.
fn push_section_header_strip(
    x: f32,
    y: f32,
    width: f32,
    label: &str,
    panel_quads: &mut Vec<Quad>,
    text_runs: &mut Vec<TextRun>,
) {
    let band = RectPx {
        x,
        y,
        width,
        height: 18.0,
    };
    panel_quads.push(Quad::from_rect(band, design_tokens::chrome::SURFACE_01));
    push_section_divider(panel_quads, x, y, width, PANEL_CARD_BORDER);
    draw_text(
        label,
        x + 12.0,
        y + 4.0,
        11.0,
        TEXT_MUTED,
        TextFace::UiStrong,
        text_runs,
    );
}

fn render_inspector_panel(
    state: &ReviewWorkspaceState,
    inspector_rect: RectPx,
    panel_quads: &mut Vec<Quad>,
    text_runs: &mut Vec<TextRun>,
    hit_regions: &mut Vec<HitRegion>,
) {
    match &state.selection {
        SelectionTarget::ReviewAction(action_id) => {
            push_inspector_title_band(
                inspector_rect,
                &format!("Action {}", truncate_text(suffix_id(action_id), 14)),
                "Route action",
                true,
                panel_quads,
                text_runs,
            );
        }
        SelectionTarget::CheckFinding(fingerprint) => {
            draw_text(
                "SELECTION",
                inspector_rect.x + 12.0,
                inspector_rect.y + 34.0,
                11.0,
                TEXT_MUTED,
                TextFace::Ui,
                text_runs,
            );
            inspector_check_finding::render_check_finding_inspector(
                state,
                fingerprint,
                inspector_rect,
                text_runs,
            );
        }
        SelectionTarget::AuthoredObject(object_id) => {
            let mut y = inspector_rect.y + 54.0;
            if let Some(comp) = state
                .scene
                .components
                .iter()
                .find(|c| &c.object_id == object_id)
            {
                draw_text(
                    &comp.reference,
                    inspector_rect.x + 12.0,
                    y,
                    15.0,
                    TEXT_PRIMARY,
                    TextFace::Mono,
                    text_runs,
                );
                y += text_row_height_for_size(15.0);
                if let Some(value) = &comp.value {
                    push_key_value(
                        inspector_rect.x + 12.0,
                        y,
                        "Value",
                        value,
                        text_runs,
                        TextFace::Ui,
                    );
                    y += key_value_row_height();
                }
                push_key_value(
                    inspector_rect.x + 12.0,
                    y,
                    "Layer",
                    &comp.placement_layer,
                    text_runs,
                    TextFace::Mono,
                );
                y += key_value_row_height();
                let pos = format!(
                    "{:.2}, {:.2} mm",
                    comp.position.x as f64 / 1_000_000.0,
                    comp.position.y as f64 / 1_000_000.0
                );
                push_key_value(
                    inspector_rect.x + 12.0,
                    y,
                    "POS",
                    &pos,
                    text_runs,
                    TextFace::Mono,
                );
            } else if let Some(pad) = state.scene.pads.iter().find(|p| &p.object_id == object_id) {
                draw_text(
                    &format!("PAD {}", pad.shape_kind.to_uppercase()),
                    inspector_rect.x + 12.0,
                    y,
                    15.0,
                    TEXT_PRIMARY,
                    TextFace::Mono,
                    text_runs,
                );
                y += text_row_height_for_size(15.0);
                push_key_value(
                    inspector_rect.x + 12.0,
                    y,
                    "LAYER",
                    &pad.layer_id.to_uppercase(),
                    text_runs,
                    TextFace::Mono,
                );
                y += key_value_row_height();
                let w = (pad.bounds.max_x - pad.bounds.min_x) as f64 / 1_000_000.0;
                let h = (pad.bounds.max_y - pad.bounds.min_y) as f64 / 1_000_000.0;
                push_key_value(
                    inspector_rect.x + 12.0,
                    y,
                    "SIZE",
                    &format!("{w:.2} x {h:.2} mm"),
                    text_runs,
                    TextFace::Mono,
                );
                y += key_value_row_height();
                if let Some(drill) = pad.drill_nm {
                    push_key_value(
                        inspector_rect.x + 12.0,
                        y,
                        "DRILL",
                        &format!("{:.2} mm", drill as f64 / 1_000_000.0),
                        text_runs,
                        TextFace::Mono,
                    );
                }
            } else if let Some(track) = state
                .scene
                .tracks
                .iter()
                .find(|t| &t.object_id == object_id)
            {
                draw_text(
                    "TRACK",
                    inspector_rect.x + 12.0,
                    y,
                    15.0,
                    TEXT_PRIMARY,
                    TextFace::Mono,
                    text_runs,
                );
                y += text_row_height_for_size(15.0);
                push_key_value(
                    inspector_rect.x + 12.0,
                    y,
                    "LAYER",
                    &track.layer_id.to_uppercase(),
                    text_runs,
                    TextFace::Mono,
                );
                y += key_value_row_height();
                push_key_value(
                    inspector_rect.x + 12.0,
                    y,
                    "WIDTH",
                    &format!("{:.2} mm", track.width_nm as f64 / 1_000_000.0),
                    text_runs,
                    TextFace::Mono,
                );
            } else if let Some(via) = state.scene.vias.iter().find(|v| &v.object_id == object_id) {
                draw_text(
                    "VIA",
                    inspector_rect.x + 12.0,
                    y,
                    15.0,
                    TEXT_PRIMARY,
                    TextFace::Mono,
                    text_runs,
                );
                y += text_row_height_for_size(15.0);
                push_key_value(
                    inspector_rect.x + 12.0,
                    y,
                    "DIA",
                    &format!("{:.2} mm", via.diameter_nm as f64 / 1_000_000.0),
                    text_runs,
                    TextFace::Mono,
                );
                y += key_value_row_height();
                push_key_value(
                    inspector_rect.x + 12.0,
                    y,
                    "DRILL",
                    &format!("{:.2} mm", via.drill_nm as f64 / 1_000_000.0),
                    text_runs,
                    TextFace::Mono,
                );
                y += key_value_row_height();
                push_key_value(
                    inspector_rect.x + 12.0,
                    y,
                    "LAYERS",
                    &format!(
                        "{} → {}",
                        via.start_layer_id.to_uppercase(),
                        via.end_layer_id.to_uppercase()
                    ),
                    text_runs,
                    TextFace::Mono,
                );
            } else if let Some(zone) = state
                .scene
                .zones
                .iter()
                .find(|z| &z.object_id == object_id)
            {
                draw_text(
                    "ZONE",
                    inspector_rect.x + 12.0,
                    y,
                    15.0,
                    TEXT_PRIMARY,
                    TextFace::Mono,
                    text_runs,
                );
                y += text_row_height_for_size(15.0);
                push_key_value(
                    inspector_rect.x + 12.0,
                    y,
                    "LAYER",
                    &zone.layer_id.to_uppercase(),
                    text_runs,
                    TextFace::Mono,
                );
                y += key_value_row_height();
                if let Some(net_uuid) = &zone.net_uuid {
                    push_key_value(
                        inspector_rect.x + 12.0,
                        y,
                        "NET",
                        &truncate_text(net_uuid, 18),
                        text_runs,
                        TextFace::Mono,
                    );
                    y += key_value_row_height();
                }
                push_key_value(
                    inspector_rect.x + 12.0,
                    y,
                    "VERTICES",
                    &zone.polygon.len().to_string(),
                    text_runs,
                    TextFace::Mono,
                );
            } else if let Some(text) = state
                .scene
                .board_texts
                .iter()
                .find(|t| &t.object_id == object_id)
            {
                draw_text(
                    "BOARD TEXT",
                    inspector_rect.x + 12.0,
                    y,
                    15.0,
                    TEXT_PRIMARY,
                    TextFace::Mono,
                    text_runs,
                );
                y += text_row_height_for_size(15.0);
                push_board_text_property_row(
                    inspector_rect.x + 12.0,
                    y,
                    "TEXT",
                    &truncate_text(&text.text.to_uppercase(), 18),
                    text_runs,
                );
                hit_regions.push(HitRegion {
                    target: HitTarget::EditSelectedBoardTextContent,
                    rect: RectPx {
                        x: inspector_rect.x + 8.0,
                        y: y - 6.0,
                        width: inspector_rect.width - 16.0,
                        height: key_value_row_height(),
                    },
                });
                y += key_value_row_height();
                push_board_text_property_row(
                    inspector_rect.x + 12.0,
                    y,
                    "MODE",
                    "READ ONLY",
                    text_runs,
                );
                hit_regions.push(HitRegion {
                    target: HitTarget::EditSelectedBoardTextContent,
                    rect: RectPx {
                        x: inspector_rect.x + 8.0,
                        y: y - 6.0,
                        width: inspector_rect.width - 16.0,
                        height: key_value_row_height(),
                    },
                });
                y += key_value_row_height();
                push_board_text_property_row(
                    inspector_rect.x + 12.0,
                    y,
                    "INTENT",
                    &text.render_intent.to_uppercase(),
                    text_runs,
                );
                let row_x = inspector_rect.x + 8.0;
                let row_w = inspector_rect.width - 16.0;
                hit_regions.push(HitRegion {
                    target: HitTarget::CycleSelectedBoardTextRenderIntent,
                    rect: RectPx {
                        x: row_x,
                        y: y - 6.0,
                        width: row_w * 0.25,
                        height: key_value_row_height(),
                    },
                });
                hit_regions.push(HitRegion {
                    target: HitTarget::EditSelectedBoardTextRenderIntent,
                    rect: RectPx {
                        x: row_x + row_w * 0.25,
                        y: y - 6.0,
                        width: row_w * 0.5,
                        height: key_value_row_height(),
                    },
                });
                hit_regions.push(HitRegion {
                    target: HitTarget::CycleSelectedBoardTextRenderIntent,
                    rect: RectPx {
                        x: row_x + row_w * 0.75,
                        y: y - 6.0,
                        width: row_w * 0.25,
                        height: key_value_row_height(),
                    },
                });
                y += key_value_row_height();
                push_board_text_property_row(
                    inspector_rect.x + 12.0,
                    y,
                    "FONT",
                    &truncate_text(&text.family.to_uppercase(), 16),
                    text_runs,
                );
                let row_x = inspector_rect.x + 8.0;
                let row_w = inspector_rect.width - 16.0;
                hit_regions.push(HitRegion {
                    target: HitTarget::CycleSelectedBoardTextFamily,
                    rect: RectPx {
                        x: row_x,
                        y: y - 6.0,
                        width: row_w * 0.25,
                        height: key_value_row_height(),
                    },
                });
                hit_regions.push(HitRegion {
                    target: HitTarget::EditSelectedBoardTextFamily,
                    rect: RectPx {
                        x: row_x + row_w * 0.25,
                        y: y - 6.0,
                        width: row_w * 0.5,
                        height: key_value_row_height(),
                    },
                });
                hit_regions.push(HitRegion {
                    target: HitTarget::CycleSelectedBoardTextFamily,
                    rect: RectPx {
                        x: row_x + row_w * 0.75,
                        y: y - 6.0,
                        width: row_w * 0.25,
                        height: key_value_row_height(),
                    },
                });
                y += key_value_row_height();
                push_board_text_property_row(
                    inspector_rect.x + 12.0,
                    y,
                    "HEIGHT",
                    &format!("{:.2} mm", text.height_nm as f64 / 1_000_000.0),
                    text_runs,
                );
                let row_x = inspector_rect.x + 8.0;
                let row_w = inspector_rect.width - 16.0;
                hit_regions.push(HitRegion {
                    target: HitTarget::DecreaseSelectedBoardTextHeight,
                    rect: RectPx {
                        x: row_x,
                        y: y - 6.0,
                        width: row_w * 0.25,
                        height: key_value_row_height(),
                    },
                });
                hit_regions.push(HitRegion {
                    target: HitTarget::EditSelectedBoardTextHeight,
                    rect: RectPx {
                        x: row_x + row_w * 0.25,
                        y: y - 6.0,
                        width: row_w * 0.5,
                        height: key_value_row_height(),
                    },
                });
                hit_regions.push(HitRegion {
                    target: HitTarget::IncreaseSelectedBoardTextHeight,
                    rect: RectPx {
                        x: row_x + row_w * 0.75,
                        y: y - 6.0,
                        width: row_w * 0.25,
                        height: key_value_row_height(),
                    },
                });
                y += key_value_row_height();
                push_board_text_property_row(
                    inspector_rect.x + 12.0,
                    y,
                    "ROT",
                    &format!("{}°", text.rotation_degrees.rem_euclid(360)),
                    text_runs,
                );
                let row_x = inspector_rect.x + 8.0;
                let row_w = inspector_rect.width - 16.0;
                hit_regions.push(HitRegion {
                    target: HitTarget::RotateSelectedBoardTextCounterClockwise90,
                    rect: RectPx {
                        x: row_x,
                        y: y - 6.0,
                        width: row_w * 0.25,
                        height: key_value_row_height(),
                    },
                });
                hit_regions.push(HitRegion {
                    target: HitTarget::EditSelectedBoardTextRotation,
                    rect: RectPx {
                        x: row_x + row_w * 0.25,
                        y: y - 6.0,
                        width: row_w * 0.5,
                        height: key_value_row_height(),
                    },
                });
                hit_regions.push(HitRegion {
                    target: HitTarget::RotateSelectedBoardTextClockwise90,
                    rect: RectPx {
                        x: row_x + row_w * 0.75,
                        y: y - 6.0,
                        width: row_w * 0.25,
                        height: key_value_row_height(),
                    },
                });
                y += key_value_row_height();
                push_board_text_property_row(
                    inspector_rect.x + 12.0,
                    y,
                    "ALIGN",
                    &format!(
                        "{} / {}",
                        text.h_align.to_uppercase(),
                        text.v_align.to_uppercase()
                    ),
                    text_runs,
                );
                let row_x = inspector_rect.x + 8.0;
                let row_w = inspector_rect.width - 16.0;
                hit_regions.push(HitRegion {
                    target: HitTarget::CycleSelectedBoardTextHAlign,
                    rect: RectPx {
                        x: row_x,
                        y: y - 6.0,
                        width: row_w * 0.25,
                        height: key_value_row_height(),
                    },
                });
                hit_regions.push(HitRegion {
                    target: HitTarget::EditSelectedBoardTextAlignment,
                    rect: RectPx {
                        x: row_x + row_w * 0.25,
                        y: y - 6.0,
                        width: row_w * 0.5,
                        height: key_value_row_height(),
                    },
                });
                hit_regions.push(HitRegion {
                    target: HitTarget::CycleSelectedBoardTextVAlign,
                    rect: RectPx {
                        x: row_x + row_w * 0.75,
                        y: y - 6.0,
                        width: row_w * 0.25,
                        height: key_value_row_height(),
                    },
                });
                y += key_value_row_height();
                push_board_text_property_row(
                    inspector_rect.x + 12.0,
                    y,
                    "LINE",
                    &format!("{}%", text.line_spacing_ratio_ppm / 10_000),
                    text_runs,
                );
                let row_x = inspector_rect.x + 8.0;
                let row_w = inspector_rect.width - 16.0;
                hit_regions.push(HitRegion {
                    target: HitTarget::DecreaseSelectedBoardTextLineSpacing,
                    rect: RectPx {
                        x: row_x,
                        y: y - 6.0,
                        width: row_w * 0.25,
                        height: key_value_row_height(),
                    },
                });
                hit_regions.push(HitRegion {
                    target: HitTarget::EditSelectedBoardTextLineSpacing,
                    rect: RectPx {
                        x: row_x + row_w * 0.25,
                        y: y - 6.0,
                        width: row_w * 0.5,
                        height: key_value_row_height(),
                    },
                });
                hit_regions.push(HitRegion {
                    target: HitTarget::IncreaseSelectedBoardTextLineSpacing,
                    rect: RectPx {
                        x: row_x + row_w * 0.75,
                        y: y - 6.0,
                        width: row_w * 0.25,
                        height: key_value_row_height(),
                    },
                });
                y += key_value_row_height();
                push_board_text_property_row(
                    inspector_rect.x + 12.0,
                    y,
                    "BOLD",
                    if text.bold { "ON" } else { "OFF" },
                    text_runs,
                );
                hit_regions.push(HitRegion {
                    target: HitTarget::ToggleSelectedBoardTextBold,
                    rect: RectPx {
                        x: inspector_rect.x + 8.0,
                        y: y - 6.0,
                        width: inspector_rect.width - 16.0,
                        height: key_value_row_height(),
                    },
                });
                y += key_value_row_height();
                push_board_text_property_row(
                    inspector_rect.x + 12.0,
                    y,
                    "MIRROR",
                    if text.mirrored { "ON" } else { "OFF" },
                    text_runs,
                );
                hit_regions.push(HitRegion {
                    target: HitTarget::ToggleSelectedBoardTextMirrored,
                    rect: RectPx {
                        x: inspector_rect.x + 8.0,
                        y: y - 6.0,
                        width: inspector_rect.width - 16.0,
                        height: key_value_row_height(),
                    },
                });
                y += key_value_row_height();
                push_board_text_property_row(
                    inspector_rect.x + 12.0,
                    y,
                    "UPRIGHT",
                    if text.keep_upright { "ON" } else { "OFF" },
                    text_runs,
                );
                hit_regions.push(HitRegion {
                    target: HitTarget::ToggleSelectedBoardTextKeepUpright,
                    rect: RectPx {
                        x: inspector_rect.x + 8.0,
                        y: y - 6.0,
                        width: inspector_rect.width - 16.0,
                        height: key_value_row_height(),
                    },
                });
                y += key_value_row_height();
                draw_text(
                    "READ-ONLY INSPECTOR",
                    inspector_rect.x + 12.0,
                    y,
                    10.5,
                    TEXT_MUTED,
                    TextFace::Mono,
                    text_runs,
                );
            } else {
                draw_text(
                    &format!(
                        "OBJECT {}",
                        truncate_text(&suffix_id(object_id).to_uppercase(), 14)
                    ),
                    inspector_rect.x + 12.0,
                    y,
                    15.0,
                    TEXT_PRIMARY,
                    TextFace::Mono,
                    text_runs,
                );
            }
            let _ = y;
        }
        SelectionTarget::None => push_inspector_title_band(
            inspector_rect,
            "No selection",
            "Select an object or review action",
            false,
            panel_quads,
            text_runs,
        ),
    }
    let inspector_details = solve_inspector_detail_layout_with_taffy(state, inspector_rect)
        .unwrap_or(InspectorDetailLayout {
            divider_y: None,
            contract: None,
            net: None,
            segment: None,
            layer: None,
            last_status: None,
        });
    // Replace the bare divider over the detail rows with an uppercase section-
    // header strip (Design Book sect-hd). The routing-review CONTRACT/NET/
    // SEGMENT rows sit under a generic "ROUTE ACTION" section.
    if inspector_details.divider_y.is_some() {
        let label = if state.selected_review_action().is_some() {
            "ROUTE ACTION"
        } else {
            "DETAILS"
        };
        push_section_header_strip(
            inspector_rect.x,
            inspector_rect.y + 72.0,
            inspector_rect.width,
            label,
            panel_quads,
            text_runs,
        );
    }
    if let (Some(action), Some(row)) = (state.selected_review_action(), inspector_details.contract)
    {
        push_key_value(
            row.x,
            row.y,
            "CONTRACT",
            &truncate_text(&action.contract.to_uppercase(), 18),
            text_runs,
            TextFace::Mono,
        );
    }
    if let (Some(action), Some(row)) = (state.selected_review_action(), inspector_details.net) {
        push_key_value(
            row.x,
            row.y,
            "NET",
            &truncate_text(&action.net_name.to_uppercase(), 16),
            text_runs,
            TextFace::Ui,
        );
    }
    if let (Some(action), Some(row)) = (state.selected_review_action(), inspector_details.segment) {
        push_key_value(
            row.x,
            row.y,
            "SEGMENT",
            &format!(
                "{} OF {}",
                action.selected_path_segment_index + 1,
                action.selected_path_segment_count
            ),
            text_runs,
            TextFace::Mono,
        );
    }
    if let (Some(segment), Some(row)) = (state.selected_segment_evidence(), inspector_details.layer)
    {
        push_key_value(
            row.x,
            row.y,
            "LAYER",
            &segment.layer.to_string(),
            text_runs,
            TextFace::Mono,
        );
    }
    if let (Some(status), Some(row)) = (&state.last_command_status, inspector_details.last_status) {
        push_key_value(
            row.x,
            row.y,
            "LAST",
            &truncate_text(&status.detail.to_uppercase(), 20),
            text_runs,
            TextFace::Mono,
        );
    }
}
