fn render_review_panel(
    state: &ReviewWorkspaceState,
    review_rect: RectPx,
    panel_quads: &mut Vec<Quad>,
    text_runs: &mut Vec<TextRun>,
    hit_regions: &mut Vec<HitRegion>,
) {
    draw_text(
        &format!(
            "SOURCE {}",
            truncate_text(&state.review.review_source.to_uppercase(), 20)
        ),
        review_rect.x + 12.0,
        review_rect.y + 34.0,
        12.0,
        TEXT_SECONDARY,
        TextFace::Mono,
        text_runs,
    );
    let prev_rect = RectPx {
        x: review_rect.x + review_rect.width - 98.0,
        y: review_rect.y + 30.0,
        width: 36.0,
        height: 20.0,
    };
    let next_rect = RectPx {
        x: review_rect.x + review_rect.width - 54.0,
        y: review_rect.y + 30.0,
        width: 36.0,
        height: 20.0,
    };
    for (rect, label, target) in [
        (prev_rect, "PREV", HitTarget::ReviewPrev),
        (next_rect, "NEXT", HitTarget::ReviewNext),
    ] {
        panel_quads.push(Quad::from_rect(rect, REVIEW_ROW_BADGE));
        push_rect_border(panel_quads, rect, PANEL_CARD_BORDER, 1.0);
        draw_text(
            label,
            rect.x + 7.0,
            rect.y + 5.0,
            10.5,
            TEXT_SECONDARY,
            TextFace::Ui,
            text_runs,
        );
        hit_regions.push(HitRegion { target, rect });
    }
    draw_text(
        &format!("{} ACTIONS", state.review.proposal_actions.len()),
        review_rect.x + 12.0,
        review_rect.y + 54.0,
        15.0,
        TEXT_PRIMARY,
        TextFace::Ui,
        text_runs,
    );
    push_section_divider(
        panel_quads,
        review_rect.x + 12.0,
        review_rect.y + 72.0,
        review_rect.width - 24.0,
        [0.23, 0.25, 0.29],
    );

    let rows: Vec<ReviewActionRow> = state.review_rows();
    let mut row_y = review_rect.y + 82.0;
    for (index, row) in rows.into_iter().enumerate() {
        let selected = row.action_id == state.active_review_target_id;
        let row_rect = RectPx {
            x: review_rect.x + 8.0,
            y: row_y,
            width: review_rect.width - 16.0,
            height: 52.0,
        };
        let badge_rect = RectPx {
            x: row_rect.x + 10.0,
            y: row_rect.y + 10.0,
            width: 30.0,
            height: 30.0,
        };
        let accent_rect = RectPx {
            x: row_rect.x,
            y: row_rect.y,
            width: 4.0,
            height: row_rect.height,
        };
        panel_quads.push(Quad::from_rect(
            row_rect,
            if selected {
                REVIEW_ROW_ACTIVE_BG
            } else {
                REVIEW_ROW_BG
            },
        ));
        panel_quads.push(Quad::from_rect(
            accent_rect,
            if selected {
                PROPOSAL_BASE
            } else {
                PANEL_CARD_BORDER
            },
        ));
        panel_quads.push(Quad::from_rect(
            badge_rect,
            if selected {
                REVIEW_ROW_BADGE_ACTIVE
            } else {
                REVIEW_ROW_BADGE
            },
        ));
        push_rect_border(
            panel_quads,
            row_rect,
            if selected {
                PROPOSAL_BASE
            } else {
                PANEL_CARD_BORDER
            },
            1.0,
        );
        draw_text(
            &(index + 1).to_string(),
            badge_rect.x + 11.0,
            badge_rect.y + 7.0,
            14.0,
            if selected {
                TEXT_PRIMARY
            } else {
                TEXT_SECONDARY
            },
            TextFace::Mono,
            text_runs,
        );
        draw_text(
            &truncate_text(&row.title, 22),
            row_rect.x + 52.0,
            row_rect.y + 10.0,
            14.0,
            if selected {
                TEXT_ACCENT
            } else {
                TEXT_PANEL_VALUE
            },
            TextFace::Ui,
            text_runs,
        );
        draw_text(
            &truncate_text(&row.subtitle, 28),
            row_rect.x + 52.0,
            row_rect.y + 28.0,
            11.0,
            TEXT_SECONDARY,
            TextFace::Mono,
            text_runs,
        );
        if selected {
            draw_text(
                "ACTIVE",
                row_rect.x + row_rect.width - 48.0,
                row_rect.y + 11.0,
                10.5,
                TEXT_ACCENT,
                TextFace::Ui,
                text_runs,
            );
        }
        hit_regions.push(HitRegion {
            target: HitTarget::ReviewAction(row.action_id),
            rect: row_rect,
        });
        row_y += 54.0;
    }
}
