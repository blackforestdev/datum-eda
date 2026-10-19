//! Project and discrete Layers composition, clipped to their owning panels.
use super::*;

pub(super) fn render_project_and_filters_panel(
    state: &ReviewWorkspaceState,
    project_layout: &ProjectPanelLayout,
    project_rect: RectPx,
    filters_rect: RectPx,
    panel_quads: &mut Vec<Quad>,
    text_runs: &mut Vec<TextRun>,
    hit_regions: &mut Vec<HitRegion>,
) {
    let starts = (panel_quads.len(), text_runs.len(), hit_regions.len());
    // Project display slug: data strings render verbatim (mixed case); only
    // panel/section TITLES stay uppercase.
    draw_text(
        &truncate_text(&state.scene.project_name, 24),
        project_layout.project_name.x,
        project_layout.project_name.y,
        15.0,
        TEXT_PRIMARY,
        TextFace::UiStrong,
        text_runs,
    );
    let scene_label = if state.scene.kind == "schematic_review_scene" {
        "Schematic"
    } else {
        "Board"
    };
    // Suppress the scene-kind prefix when the board name would merely duplicate
    // it (e.g. "Board · board"): show one clean label.
    let board_name = state.scene.board_name.trim();
    let board_line = if board_name.is_empty() || board_name.eq_ignore_ascii_case(scene_label) {
        scene_label.to_string()
    } else {
        format!("{} · {}", scene_label, truncate_text(board_name, 18))
    };
    draw_text(
        &board_line,
        project_layout.board_name.x,
        project_layout.board_name.y,
        12.5,
        TEXT_SECONDARY,
        TextFace::Ui,
        text_runs,
    );
    // (The selected net belongs to the Inspector, not the Project panel — the
    // "Net <name>" line is removed here to keep the panel a clean project slug.)
    let _ = project_layout.net;
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
        // REV-I10 gives this permanent Project-panel region to the expanded
        // Revision Navigator. Fit remains available from View and keyboard
        // commands; retaining a second button row here would overlap the
        // owner-approved taxonomy.
        let _ = (rect, label, target);
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
    crate::hit_clipping::clip_content(
        panel_quads,
        text_runs,
        hit_regions,
        starts.0,
        starts.1,
        starts.2,
        project_rect,
    );
    let starts = (panel_quads.len(), text_runs.len(), hit_regions.len());
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
    // Divider between the review-filter toggles (a routing-review-scene artifact,
    // not the prototype's physical-layer list) and the physical-layer rows below,
    // so the two groups read as distinct rather than one flat list. A curated
    // board+component demo scene omits these toggles entirely (see M8).
    push_section_divider(
        panel_quads,
        filters_rect.x,
        filters_layout.dim_unrelated.y + filters_layout.dim_unrelated.height + 4.0,
        filters_rect.width,
        PANEL_CARD_BORDER,
    );
    // Layers follow the focused document, using the shared resolved pane source.
    let layers = datum_gui_protocol::camera_scene_for_pane(state, state.ui.layout.focused)
        .map(|scene| scene.layers.as_slice())
        .unwrap_or_default();
    let mut display_layers: Vec<&_> = layers.iter().collect();
    display_layers.sort_by_key(|l| {
        (
            !l.visible_by_default,
            scene_layer_stack_priority(&l.layer_id, layers),
            l.render_order,
        )
    });
    let visible_capacity = filters_layout.layer_rows.len();
    let start = layer_scroll::render_layer_scroll_affordance(
        state,
        filters_rect,
        &filters_layout.layer_rows,
        display_layers.len(),
        panel_quads,
        text_runs,
        hit_regions,
    );
    for (layer, row) in display_layers
        .iter()
        .skip(start)
        .take(visible_capacity)
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
        render_layer_row(layers, row, layer, visible, active, panel_quads, text_runs);
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
    crate::hit_clipping::clip_content(
        panel_quads,
        text_runs,
        hit_regions,
        starts.0,
        starts.1,
        starts.2,
        filters_rect,
    );
}

fn render_layer_row(
    scene_layers: &[datum_gui_protocol::SceneLayer],
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
    let swatch_color = schematic_layer_world_color(&layer.name)
        .unwrap_or_else(|| layer_swatch_color_with_scene(Some(&layer.layer_id), scene_layers));
    // Off rows dim the ENTIRE row (swatch + name) uniformly toward the panel
    // surface, matching the prototype's .layer-row.off{opacity:.4}. There is no
    // ON/OFF/ACTIVE text badge — visibility reads through the dim alone.
    let dim = |color: [f32; 3]| -> [f32; 3] {
        if visible {
            color
        } else {
            mix_color(color, PANEL_CARD_BG, 0.6)
        }
    };
    let swatch = RectPx {
        x: row.x,
        y: row.y + 3.0,
        width: 13.0,
        height: 13.0,
    };
    panel_quads.push(Quad::from_rect(swatch, dim(swatch_color)));
    // Subtle ~12% white inset highlight on the swatch edge (not a dark box).
    push_rect_border(
        panel_quads,
        swatch,
        dim(mix_color(swatch_color, [1.0, 1.0, 1.0], 0.12)),
        1.0,
    );
    draw_text(
        &truncate_text(&layer.name, 18),
        row.x + 22.0,
        row.y + 1.0,
        13.5,
        dim(if visible { TEXT_PRIMARY } else { TEXT_MUTED }),
        TextFace::Ui,
        text_runs,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn focused_layers_swatches_use_canvas_roles_and_preserve_board() {
        let (fixture, state) =
            crate::native_selection_overlay::tests::selected_schematic_text_state();
        let source = fixture.source_bytes();
        let schematic = state.schematic_scene.as_ref().unwrap();
        let row = RectPx {
            x: 12.0,
            y: 500.0,
            width: 190.0,
            height: 25.0,
        };
        let check = |layers: &[datum_gui_protocol::SceneLayer],
                     layer: &datum_gui_protocol::SceneLayer,
                     color| {
            for visible in [true, false] {
                let mut quads = Vec::new();
                let mut text = Vec::new();
                render_layer_row(layers, &row, layer, visible, false, &mut quads, &mut text);
                let swatch = &quads[0];
                assert_eq!(
                    swatch.points,
                    Quad::from_rect(
                        RectPx {
                            x: row.x,
                            y: row.y + 3.0,
                            width: 13.0,
                            height: 13.0,
                        },
                        color
                    )
                    .points
                );
                let expected = if visible {
                    color
                } else {
                    mix_color(color, PANEL_CARD_BG, 0.6)
                };
                assert_eq!(swatch.color, expected, "actual {} row swatch", layer.name);
            }
        };
        for (name, color) in [
            ("Schematic.Wire", design_tokens::schematic::WIRE),
            ("Schematic.Junction", design_tokens::schematic::WIRE),
            ("Schematic.Bus", design_tokens::schematic::BUS),
        ] {
            let layer = schematic
                .layers
                .iter()
                .find(|layer| layer.name == name)
                .unwrap();
            check(&schematic.layers, layer, color);
        }
        let copper = state
            .scene
            .layers
            .iter()
            .find(|layer| layer.name == "F.Cu")
            .unwrap();
        check(
            &state.scene.layers,
            copper,
            design_tokens::content::COPPER_FRONT,
        );
        assert_eq!(fixture.source_bytes(), source);
    }

    #[test]
    fn focused_layers_rows_and_clipped_scroll_follow_document() {
        use datum_gui_protocol::PaneContent;
        let (fixture, mut state) =
            crate::native_selection_overlay::tests::selected_schematic_text_state();
        let source = fixture.source_bytes();
        let selection = state.selection.clone();
        let board: Vec<_> = state
            .scene
            .layers
            .iter()
            .map(|l| l.layer_id.clone())
            .collect();
        let schematic: Vec<_> = state
            .schematic_scene
            .as_ref()
            .unwrap()
            .layers
            .iter()
            .map(|l| l.layer_id.clone())
            .collect();
        assert!(schematic.len() > board.len() && schematic.contains(&"L202".to_owned()));
        let paint = |state: &ReviewWorkspaceState| {
            let sidebar = RectPx {
                x: 0.0,
                y: 30.0,
                width: 224.0,
                height: 700.0,
            };
            let layout = solve_project_panel_layout_with_taffy(state, sidebar).unwrap();
            let mut quads = Vec::new();
            let mut text = Vec::new();
            let mut hits = Vec::new();
            render_project_and_filters_panel(
                state,
                &layout,
                layout.project_rect,
                layout.filters_rect,
                &mut quads,
                &mut text,
                &mut hits,
            );
            hits.into_iter()
                .filter_map(|hit| match hit.target {
                    HitTarget::ToggleLayer(id) => Some(id),
                    _ => None,
                })
                .collect::<Vec<_>>()
        };
        for content in [
            PaneContent::Board,
            PaneContent::Schematic,
            PaneContent::Board,
            PaneContent::Library,
        ] {
            state.ui.layout.set_focused_content(content);
            state.ui.filters.layer_scroll_offset = 0;
            let first = paint(&state);
            match content {
                PaneContent::Board => assert_eq!(first, board),
                PaneContent::Schematic => {
                    assert!(first.len() > board.len() && first.len() < schematic.len());
                    assert!(first.iter().all(|id| schematic.contains(id)));
                    state.ui.filters.layer_scroll_offset = usize::MAX;
                    let last = paint(&state);
                    assert_eq!(first.len(), last.len());
                    assert_ne!(
                        first, last,
                        "clipped Layers must reach later schematic roles"
                    );
                    assert!(last.contains(schematic.last().unwrap()));
                    assert!(last.iter().all(|id| schematic.contains(id)));
                }
                _ => assert!(first.is_empty(), "non-editor must not borrow Board layers"),
            }
        }
        state.ui.layout.set_focused_content(PaneContent::Schematic);
        state.schematic_scene = None;
        assert!(
            paint(&state).is_empty(),
            "unresolved Schematic must not borrow Board layers"
        );
        assert_eq!(state.selection, selection);
        assert_eq!(fixture.source_bytes(), source);
    }

    #[test]
    fn project_and_discrete_layers_inherit_separate_panel_bounds() {
        let mut state = datum_gui_protocol::load_fixture_workspace_state();
        let sidebar = RectPx {
            x: 0.0,
            y: 30.0,
            width: 130.0,
            height: 420.0,
        };
        let layout = solve_project_panel_layout_with_taffy(&state, sidebar)
            .unwrap_or_else(|| fallback_project_panel_layout(&state, sidebar));
        for offset in [0, usize::MAX] {
            state.ui.filters.layer_scroll_offset = offset;
            let mut quads = Vec::new();
            let mut text = Vec::new();
            let mut hits = Vec::new();
            render_project_and_filters_panel(
                &state,
                &layout,
                layout.project_rect,
                layout.filters_rect,
                &mut quads,
                &mut text,
                &mut hits,
            );
            assert!(!text.is_empty());
            for run in text {
                let clip = run.clip_bounds.expect("panel labels must inherit a clip");
                assert!(
                    clip.intersect(layout.project_rect) == Some(clip)
                        || clip.intersect(layout.filters_rect) == Some(clip)
                );
            }
            for hit in hits {
                assert!(
                    hit.rect.intersect(layout.project_rect) == Some(hit.rect)
                        || hit.rect.intersect(layout.filters_rect) == Some(hit.rect)
                );
            }
            for quad in quads {
                assert!(quad.points.iter().all(|&(x, y)| sidebar.contains(x, y)));
            }
            assert_eq!(
                state.ui.filters.layer_scroll_offset, offset,
                "painting must not mutate discrete row authority"
            );
        }
    }
}
