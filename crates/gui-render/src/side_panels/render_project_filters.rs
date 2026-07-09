fn render_project_and_filters_panel(
    state: &ReviewWorkspaceState,
    project_layout: &ProjectPanelLayout,
    project_rect: RectPx,
    filters_rect: RectPx,
    panel_quads: &mut Vec<Quad>,
    text_runs: &mut Vec<TextRun>,
    hit_regions: &mut Vec<HitRegion>,
) {
    draw_text(
        &truncate_text(&state.scene.project_name.to_uppercase(), 22),
        project_layout.project_name.x,
        project_layout.project_name.y,
        16.0,
        TEXT_PRIMARY,
        TextFace::Ui,
        text_runs,
    );
    let scene_label = if state.scene.kind == "schematic_review_scene" {
        "SCHEMATIC"
    } else {
        "BOARD"
    };
    draw_text(
        &format!(
            "{} {}",
            scene_label,
            truncate_text(&state.scene.board_name.to_uppercase(), 18)
        ),
        project_layout.board_name.x,
        project_layout.board_name.y,
        12.5,
        TEXT_SECONDARY,
        TextFace::Ui,
        text_runs,
    );
    if let (Some(action), Some(net_rect)) = (state.selected_review_action(), project_layout.net) {
        draw_text(
            &format!("NET {}", truncate_text(&action.net_name.to_uppercase(), 18)),
            net_rect.x,
            net_rect.y,
            13.0,
            TEXT_ACCENT,
            TextFace::Ui,
            text_runs,
        );
    }
    let shard_attention_count = state.source_shards.attention_count();
    let shard_label = source_shard_health_label(&state.source_shards);
    draw_text(
        &truncate_text(&shard_label, 26),
        project_layout.source_label.x,
        project_layout.source_label.y,
        11.0,
        if shard_attention_count == 0 {
            TEXT_MUTED
        } else {
            TEXT_ACCENT
        },
        TextFace::Mono,
        text_runs,
    );
    let source_rows_end_y = render_shard_rows(
        &state.source_shards,
        project_rect,
        project_layout.source_rows.y,
        text_runs,
    );
    let action_y = project_layout.fit_row.y.max(source_rows_end_y + 4.0);
    let fit_board_rect = RectPx {
        x: project_layout.fit_row.x,
        y: action_y,
        width: 72.0,
        height: 20.0,
    };
    let fit_review_rect = RectPx {
        x: project_layout.fit_row.x + 80.0,
        y: action_y,
        width: 92.0,
        height: 20.0,
    };
    let fit_scene_label = if state.scene.kind == "schematic_review_scene" {
        "FIT SCH"
    } else {
        "FIT BOARD"
    };
    for (rect, label, target) in [
        (fit_board_rect, fit_scene_label, HitTarget::FitBoard),
        (fit_review_rect, "FIT REVIEW", HitTarget::FitReviewTarget),
    ] {
        panel_quads.push(Quad::from_rect(rect, REVIEW_ROW_BADGE));
        push_rect_border(panel_quads, rect, PANEL_CARD_BORDER, 1.0);
        draw_text(
            label,
            rect.x + 7.0,
            rect.y + 5.0,
            10.0,
            TEXT_SECONDARY,
            TextFace::Ui,
            text_runs,
        );
        hit_regions.push(HitRegion { target, rect });
    }
    draw_text(
        "READ-ONLY BOARD VIEW",
        project_layout.tool_label.x,
        project_layout.tool_label.y,
        12.0,
        TEXT_MUTED,
        TextFace::Mono,
        text_runs,
    );
    draw_text(
        "Select objects to inspect. Authoring tools are disabled in Phase 1.",
        project_layout.tool_grid.x,
        project_layout.tool_grid.y,
        10.0,
        TEXT_SECONDARY,
        TextFace::Ui,
        text_runs,
    );
    if let Some(import_notice) = project_layout.import_notice {
        draw_text(
            "IMPORT VIEW: AUTHORING REQUIRES NATIVE PROJECT",
            import_notice.x,
            import_notice.y,
            9.5,
            TEXT_ACCENT,
            TextFace::Mono,
            text_runs,
        );
    }
    if let (Some(status), Some(status_rect)) =
        (&state.last_command_status, project_layout.last_status)
    {
        draw_text(
            &truncate_text(&format!("LAST {}", status.action.to_uppercase()), 24),
            status_rect.x,
            status_rect.y,
            11.0,
            TEXT_MUTED,
            TextFace::Mono,
            text_runs,
        );
    }
    let filters_layout = solve_filters_panel_layout_with_taffy(state, filters_rect);
    let Some(filters_layout) = filters_layout else {
        return;
    };
    for (row, label, enabled, target) in [
        (
            filters_layout.authored,
            "AUTHORED",
            state.ui.filters.show_authored,
            HitTarget::ToggleShowAuthored,
        ),
        (
            filters_layout.proposed,
            "PROPOSED",
            state.ui.filters.show_proposed,
            HitTarget::ToggleShowProposed,
        ),
        (
            filters_layout.unrouted,
            "UNROUTED",
            state.ui.filters.show_unrouted,
            HitTarget::ToggleShowUnrouted,
        ),
        (
            filters_layout.dim_unrelated,
            "DIM UNRELATED",
            state.ui.filters.dim_unrelated,
            HitTarget::ToggleDimUnrelated,
        ),
    ] {
        push_boolean_row(row.x, row.y, label, enabled, text_runs);
        hit_regions.push(HitRegion {
            target,
            rect: filter_hit_rect(row),
        });
    }
    // Show all layers — copper first, then non-copper
    let mut display_layers: Vec<&_> = state.scene.layers.iter().collect();
    display_layers.sort_by_key(|l| {
        (
            !l.visible_by_default,
            scene_layer_stack_priority(&l.layer_id, &state.scene.layers),
            l.render_order,
        )
    });
    for (layer, row) in display_layers
        .iter()
        .take(filters_layout.layer_rows.len())
        .zip(filters_layout.layer_rows.iter())
    {
        let visible = state
            .ui
            .filters
            .layer_visibility
            .get(&layer.layer_id)
            .copied()
            .unwrap_or(layer.visible_by_default);
        let active = state
            .ui
            .filters
            .active_layer_id
            .as_deref()
            .is_some_and(|active| active == layer.layer_id);
        render_layer_row(state, row, layer, visible, active, panel_quads, text_runs);
        hit_regions.push(HitRegion {
            target: HitTarget::ToggleLayer(layer.layer_id.clone()),
            rect: filter_hit_rect(*row),
        });
    }
    if let Some(row) = filters_layout.active_summary {
        let active_layer = state
            .ui
            .filters
            .active_layer_id
            .as_deref()
            .and_then(|active_id| {
                state
                    .scene
                    .layers
                    .iter()
                    .find(|layer| layer.layer_id == active_id)
            })
            .map(|layer| layer.name.as_str())
            .unwrap_or("none");
        draw_text(
            &format!("ACTIVE {}", truncate_text(&active_layer.to_uppercase(), 14)),
            row.x,
            row.y,
            11.0,
            TEXT_MUTED,
            TextFace::Mono,
            text_runs,
        );
    }
    draw_text(
        &format!("LAYERS {}", state.scene.layers.len()),
        filters_layout.layers_summary.x,
        filters_layout.layers_summary.y,
        11.0,
        TEXT_MUTED,
        TextFace::Mono,
        text_runs,
    );
    draw_text(
        &format!(
            "FOCUS {}",
            if has_review_focus(state) { "ON" } else { "OFF" }
        ),
        filters_layout.focus_summary.x,
        filters_layout.focus_summary.y,
        11.0,
        TEXT_MUTED,
        TextFace::Mono,
        text_runs,
    );
    draw_text(
        &format!(
            "OUTPUTS {} / ART {} / {}",
            state.production.output_job_count,
            state.production.artifact_count,
            state
                .production
                .latest_status
                .as_deref()
                .unwrap_or("never_run")
                .to_uppercase()
        ),
        filters_layout.outputs_summary.x,
        filters_layout.outputs_summary.y,
        11.0,
        TEXT_MUTED,
        TextFace::Mono,
        text_runs,
    );
}

fn render_layer_row(
    state: &ReviewWorkspaceState,
    row: &RectPx,
    layer: &datum_gui_protocol::SceneLayer,
    visible: bool,
    active: bool,
    panel_quads: &mut Vec<Quad>,
    text_runs: &mut Vec<TextRun>,
) {
    let row_rect = RectPx {
        x: row.x - design_tokens::spacing::SP_02,
        y: row.y - design_tokens::spacing::SP_01,
        width: row.width + design_tokens::spacing::SP_02 * 2.0,
        height: row.height,
    };
    if active {
        panel_quads.push(Quad::from_rect(row_rect, REVIEW_ROW_ACTIVE_BG));
        panel_quads.push(Quad::from_rect(
            RectPx {
                x: row_rect.x,
                y: row_rect.y,
                width: 2.0,
                height: row_rect.height,
            },
            TEXT_ACCENT,
        ));
    }
    let appearance = resolve_layer_appearance_with_scene(Some(&layer.layer_id), &state.scene.layers);
    let swatch = RectPx {
        x: row.x,
        y: row.y + 2.0,
        width: 12.0,
        height: 12.0,
    };
    panel_quads.push(Quad::from_rect(
        swatch,
        if visible {
            appearance.authored_track
        } else {
            dim_context_color(appearance.authored_track, true)
        },
    ));
    push_rect_border(panel_quads, swatch, PANEL_CARD_BORDER, 1.0);
    draw_text(
        &truncate_text(&layer.name.to_uppercase(), 16),
        row.x + 20.0,
        row.y,
        11.0,
        if visible { TEXT_SECONDARY } else { TEXT_MUTED },
        TextFace::Ui,
        text_runs,
    );
    draw_text(
        if active {
            "ACTIVE"
        } else if visible {
            "ON"
        } else {
            "OFF"
        },
        row.x + row.width - 44.0,
        row.y,
        10.5,
        if active { TEXT_ACCENT } else { TEXT_MUTED },
        TextFace::Mono,
        text_runs,
    );
}
