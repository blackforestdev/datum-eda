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
        &format!("TOOL {}", workspace_tool_label(state.tool)),
        project_layout.tool_label.x,
        project_layout.tool_label.y,
        12.0,
        TEXT_MUTED,
        TextFace::Mono,
        text_runs,
    );
    let tool_rows = [
        (WorkspaceTool::Select, "S", "SELECT"),
        (WorkspaceTool::DrawBoardTrack, "R", "TRACK"),
        (WorkspaceTool::PlaceBoardVia, "V", "VIA"),
        (WorkspaceTool::PlaceBoardText, "B", "TEXT"),
        (WorkspaceTool::Move, "M", "MOVE"),
        (WorkspaceTool::Delete, "X", "DELETE"),
    ];
    let tool_grid_x = project_layout.tool_grid.x;
    let tool_grid_y = project_layout.tool_grid.y;
    let tool_gap = 6.0;
    let tool_columns = 3;
    let tool_width = ((project_layout.tool_grid.width - tool_gap * (tool_columns as f32 - 1.0))
        / tool_columns as f32)
        .max(44.0);
    for (index, (tool, key, label)) in tool_rows.iter().enumerate() {
        let column = index % tool_columns;
        let row = index / tool_columns;
        let rect = RectPx {
            x: tool_grid_x + column as f32 * (tool_width + tool_gap),
            y: tool_grid_y + row as f32 * (UI_ROW_BUTTON + 4.0),
            width: tool_width,
            height: UI_ROW_BUTTON,
        };
        let active = state.tool == *tool;
        panel_quads.push(Quad::from_rect(
            rect,
            if active {
                REVIEW_ROW_ACTIVE_BG
            } else {
                REVIEW_ROW_BADGE
            },
        ));
        push_rect_border(
            panel_quads,
            rect,
            if active {
                TEXT_ACCENT
            } else {
                PANEL_CARD_BORDER
            },
            1.0,
        );
        draw_text(
            &format!("{key} {label}"),
            rect.x + 5.0,
            rect.y + 5.0,
            9.0,
            if active { TEXT_PRIMARY } else { TEXT_SECONDARY },
            TextFace::Ui,
            text_runs,
        );
        hit_regions.push(HitRegion {
            target: HitTarget::SetWorkspaceTool(*tool),
            rect,
        });
    }
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
        push_boolean_row(
            row.x,
            row.y,
            &truncate_text(&layer.name.to_uppercase(), 18),
            visible,
            text_runs,
        );
        hit_regions.push(HitRegion {
            target: HitTarget::ToggleLayer(layer.layer_id.clone()),
            rect: filter_hit_rect(*row),
        });
    }
    if let (Some(action), Some(row)) = (
        state.selected_review_action(),
        filters_layout.active_summary,
    ) {
        draw_text(
            &format!(
                "ACTIVE {}",
                truncate_text(&suffix_id(&action.action_id).to_uppercase(), 14)
            ),
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
