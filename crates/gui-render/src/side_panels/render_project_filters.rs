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
    // (Removed the source-shard health line + shard rows from the Project panel
    // — provenance diagnostics that read as a debug HUD, not the designed tree.)
    let _ = (project_layout.source_label, project_layout.source_rows);
    let action_y = project_layout.fit_row.y;
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
    // (Removed the READ-ONLY BOARD VIEW / "Select objects…" / IMPORT VIEW notices
    // from the Project panel — the read-only state is already conveyed by the empty
    // inspector and disabled menu items; these lines read as HUD clutter and are not
    // part of the designed panel.)
    let _ = (
        project_layout.tool_label,
        project_layout.tool_grid,
        project_layout.import_notice,
    );
    // (Removed the "LAST <action>" command-status line — terminal/debug noise.)
    let _ = project_layout.last_status;
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
        push_boolean_row(row.x, row.y, row.width, label, enabled, text_runs);
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
    // (Removed the Layers-panel diagnostic tail — ACTIVE <layer> / LAYERS <n> /
    // FOLLOWS PANE A / OUTPUTS/ART/status. That state dump read as a debug HUD;
    // the active layer already shows its ACTIVE badge inline, and the rest is
    // not part of the designed Layers panel.)
    let _ = (
        filters_layout.active_summary,
        filters_layout.layers_summary,
        filters_layout.focus_summary,
        filters_layout.outputs_summary,
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
    let swatch_color = layer_swatch_color_with_scene(Some(&layer.layer_id), &state.scene.layers);
    let swatch = RectPx {
        x: row.x,
        y: row.y + 2.0,
        width: 12.0,
        height: 12.0,
    };
    panel_quads.push(Quad::from_rect(
        swatch,
        if visible {
            swatch_color
        } else {
            dim_context_color(swatch_color, true)
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
