fn push_scene_overlay_and_hits(
    out: &mut Vec<Quad>,
    scene: &BoardReviewSceneV1,
    scene_viewport: RectPx,
    camera: CameraState,
    state: &ReviewWorkspaceState,
    text_runs: &mut Vec<TextRun>,
    hit_regions: &mut Vec<HitRegion>,
) {
    let board_field = inset_rect(scene_viewport, 10.0, 10.0, 10.0, 10.0);
    let projection = Projection::new(board_field, &scene.bounds, camera);
    let active_move_component_uuid: Option<String> = None;
    let preview_affected_ids = proposal_preview_affected_ids(state);
    push_lightweight_selection_overlay(out, scene, state, &projection);
    for component in &scene.components {
        if !authored_visible(state) || !layer_visible(state, &component.placement_layer) {
            continue;
        }
        if active_move_component_uuid.as_deref() == Some(component.component_uuid.as_str()) {
            continue;
        }
        let has_detail_text = component_has_detail_text(scene, &component.component_uuid);
        // Skip synthetic labels when imported silk text exists — silk handles it
        if has_detail_text {
            continue;
        }
        let selected = matches!(state.selection, SelectionTarget::AuthoredObject(ref id) if id == &component.object_id)
            || component_is_selection_active(&component.component_uuid, scene, state);
        let related = component_overlaps_active_action(component, state)
            || component_is_selection_related(&component.component_uuid, scene, state)
            || source_object_matches_preview(
                &preview_affected_ids,
                &component.object_id,
                &component.source_object_uuid,
            );
        let dimmed = dim_unrelated_active(state) && !selected && !related;
        let label_rect = project_rect(component.bounds, &projection);
        let label_text = truncate_text(&component.reference.to_uppercase(), 6);
        let label_size = if selected || related { 11.0 } else { 10.0 };
        let label_color = if selected {
            selected_silk_color(COMPONENT_SILK)
        } else if related {
            PAD_COPPER_RELATED
        } else {
            dim_context_color(COMPONENT_SILK, dimmed)
        };
        // Center label inside component body
        let label_x =
            label_rect.x + (label_rect.width * 0.5) - (label_text.len() as f32 * label_size * 0.32);
        let label_y = label_rect.y + (label_rect.height * 0.5) - (label_size * 0.5);
        draw_text_clipped(
            &label_text,
            label_x.max(label_rect.x + 4.0),
            label_y.max(board_field.y + 6.0),
            label_size,
            label_color,
            TextFace::Mono,
            scene_viewport,
            text_runs,
        );
        let label_hit = RectPx {
            x: label_x.max(label_rect.x + 2.0) - 4.0,
            y: label_y.max(board_field.y + 6.0) - label_size,
            width: (label_text.len() as f32 * label_size * 0.64).max(20.0),
            height: (label_size + 6.0).max(14.0),
        };
        hit_regions.push(HitRegion {
            target: HitTarget::AuthoredObject(component.object_id.clone()),
            rect: label_hit,
        });
        if !has_detail_text {
            let (label_x, label_y) = project_point(component.position, &projection);
            draw_text_clipped(
                &truncate_text(&component.reference.to_uppercase(), 6),
                label_x - 9.0,
                (label_y - 4.0).max(board_field.y + 6.0),
                9.0,
                if selected {
                    selected_silk_color([0.80, 0.82, 0.86])
                } else if related {
                    PAD_COPPER_RELATED
                } else {
                    dim_context_color([0.80, 0.82, 0.86], dimmed)
                },
                TextFace::Mono,
                scene_viewport,
                text_runs,
            );
            hit_regions.push(HitRegion {
                target: HitTarget::AuthoredObject(component.object_id.clone()),
                rect: RectPx {
                    x: label_x - 12.0,
                    y: (label_y - 4.0).max(board_field.y + 6.0) - 10.0,
                    width: 32.0,
                    height: 18.0,
                },
            });
        }
    }
    for text in &scene.component_texts {
        if !authored_visible(state) {
            continue;
        }
        if let Some(lid) = text.layer_id.as_deref()
            && !layer_visible(state, lid) {
                continue;
            }
        if active_move_component_uuid.as_deref() == Some(text.component_uuid.as_str()) {
            continue;
        }
        let related = scene.components.iter().any(|component| {
            component.component_uuid == text.component_uuid
                && component_overlaps_active_action(component, state)
        }) || component_is_selection_related(&text.component_uuid, scene, state)
            || component_matches_preview(&text.component_uuid, scene, &preview_affected_ids);
        let selected = matches!(
            state.selection,
            SelectionTarget::AuthoredObject(ref id)
                if id == &format!("component:{}", text.component_uuid)
        ) || component_is_selection_active(&text.component_uuid, scene, state);
        let dimmed = dim_unrelated_active(state) && !selected && !related;
        push_component_text_world(
            out,
            text_runs,
            text,
            &scene.layers,
            &projection,
            scene_viewport,
            selected,
            related,
            dimmed,
        );
        let (tx, ty) = project_point(text.position, &projection);
        let text_size = footprint_text_size_px(text.height_nm, &projection);
        hit_regions.push(HitRegion {
            target: HitTarget::AuthoredObject(format!("component:{}", text.component_uuid)),
            rect: RectPx {
                x: tx - (text.text.len() as f32 * text_size * 0.36).max(10.0),
                y: ty - text_size,
                width: (text.text.len() as f32 * text_size * 0.72).max(24.0),
                height: (text_size + 6.0).max(14.0),
            },
        });
    }
    // Show net name for selected or hovered pads
    for pad in &scene.pads {
        let selected_pad =
            matches!(&state.selection, SelectionTarget::AuthoredObject(id) if id == &pad.object_id);
        let hovered_pad = is_hovered(state, &pad.object_id);
        if (selected_pad || hovered_pad) && pad.net_uuid.is_some() {
            let net_label = state
                .review
                .proposal_actions
                .iter()
                .find(|a| Some(&a.net_uuid) == pad.net_uuid.as_ref())
                .map(|a| a.net_name.clone())
                .unwrap_or_else(|| "NET".to_string());
            let (px, py) = project_point(pad.center, &projection);
            draw_text_clipped(
                &net_label.to_uppercase(),
                px + 8.0,
                py - 14.0,
                10.0,
                TEXT_ACCENT,
                TextFace::Mono,
                scene_viewport,
                text_runs,
            );
        }
    }
    if let Some(active_component_uuid) = active_move_component_uuid.as_deref()
        && let Some(component) = scene
            .components
            .iter()
            .find(|component| component.component_uuid == active_component_uuid)
    {
        let has_detail_text = component_has_detail_text(scene, &component.component_uuid);
        let selected = true;
        let related = component_overlaps_active_action(component, state);
        let dimmed = false;
        let label_rect = project_rect(component.bounds, &projection);
        draw_text_clipped(
            &truncate_text(&component.reference.to_uppercase(), 6),
            label_rect.x + 6.0,
            (label_rect.y - 11.0).max(board_field.y + 6.0),
            10.0,
            selected_silk_color(COMPONENT_SILK),
            TextFace::Mono,
            scene_viewport,
            text_runs,
        );
        if !has_detail_text {
            let (label_x, label_y) = project_point(component.position, &projection);
            draw_text_clipped(
                &truncate_text(&component.reference.to_uppercase(), 6),
                label_x - 9.0,
                (label_y - 4.0).max(board_field.y + 6.0),
                9.0,
                selected_silk_color([0.80, 0.82, 0.86]),
                TextFace::Mono,
                scene_viewport,
                text_runs,
            );
        }
        for text in scene
            .component_texts
            .iter()
            .filter(|text| text.component_uuid == component.component_uuid)
        {
            push_component_text_world(
                out,
                text_runs,
                text,
                &scene.layers,
                &projection,
                scene_viewport,
                selected,
                related,
                dimmed,
            );
        }
    }
    if proposed_visible(state) {
        for overlay in &scene.proposal_overlay_primitives {
            if !overlay
                .layer_id
                .as_deref()
                .is_none_or(|layer_id| layer_visible(state, layer_id))
            {
                continue;
            }
            let selected = overlay.proposal_action_id == state.active_review_target_id;
            let color = match overlay.render_role.as_str() {
                "proposed_focus" if selected => PROPOSAL_FOCUS,
                "proposed_overlay" if selected => PROPOSAL_FOCUS,
                "proposed_overlay" => PROPOSAL_BASE,
                "authored_related" => AUTHOR_RELATED,
                _ => PROPOSAL_BASE,
            };
            let rects = push_overlay(out, overlay, &projection, color, selected, false);
            for rect in rects {
                hit_regions.push(HitRegion {
                    target: HitTarget::ReviewAction(overlay.proposal_action_id.clone()),
                    rect,
                });
            }
        }
        for overlay in production_proposal_overlay_primitives(state) {
            if !overlay
                .layer_id
                .as_deref()
                .is_none_or(|layer_id| layer_visible(state, layer_id))
            {
                continue;
            }
            let rects = push_overlay(out, &overlay, &projection, PROPOSAL_BASE, false, false);
            for rect in rects {
                hit_regions.push(HitRegion {
                    target: HitTarget::ReviewAction(overlay.proposal_action_id.clone()),
                    rect,
                });
            }
        }
    }
    let active_evidence_key = state
        .selected_review_action()
        .map(|action| format!("segment:{}", action.selected_path_segment_index));
    for review in &scene.review_primitives {
        let active = review.evidence_key.as_ref() == active_evidence_key.as_ref();
        push_dashed_polyline_segments(
            out,
            &review.path,
            &projection,
            DIAGNOSTIC_UNDERLAY,
            if active { 2.1 } else { 1.6 },
            10.0,
            6.0,
        );
        push_dashed_polyline_segments(
            out,
            &review.path,
            &projection,
            if active {
                DIAGNOSTIC_FOCUS
            } else {
                DIAGNOSTIC_BASE
            },
            if active { 1.2 } else { 0.9 },
            10.0,
            6.0,
        );
        // Diagnostic emphasis marks where the evidence span starts and ends.
        // Interior vertices stay unmarked: per-vertex dots read as generic
        // path-editing handles, which M7-REN-003 forbids over proposed copper.
        if let (Some(first), Some(last)) = (review.path.first(), review.path.last()) {
            push_points(
                out,
                &[*first, *last],
                &projection,
                if active {
                    DIAGNOSTIC_FOCUS
                } else {
                    DIAGNOSTIC_BASE
                },
                if active { 4.0 } else { 3.0 },
            );
        }
    }
}

fn production_proposal_overlay_primitives(
    state: &ReviewWorkspaceState,
) -> Vec<ProposalOverlayPrimitive> {
    state
        .production
        .proposals
        .iter()
        .filter_map(|proposal| {
            proposal
                .preview
                .as_ref()
                .map(|preview| (proposal.proposal_id.as_str(), preview))
        })
        .flat_map(|(proposal_id, preview)| {
            preview
                .render_deltas
                .iter()
                .enumerate()
                .filter(|(_, delta)| {
                    (delta.primitive_kind == "track_path" && delta.path.len() >= 2)
                        || (delta.primitive_kind == "via" && !delta.path.is_empty())
                })
                .map(move |(index, delta)| ProposalOverlayPrimitive {
                    overlay_id: format!("proposal:{proposal_id}:preview:{index}"),
                    primitive_kind: delta.primitive_kind.clone(),
                    proposal_action_id: proposal_id.to_string(),
                    layer_id: Some(delta.layer_id.clone()),
                    render_role: "proposed_preview".to_string(),
                    width_nm: Some(delta.width_nm),
                    drill_nm: delta.drill_nm,
                    diameter_nm: delta.diameter_nm,
                    path: delta.path.clone(),
                })
        })
        .collect()
}

fn push_lightweight_selection_overlay(
    out: &mut Vec<Quad>,
    scene: &BoardReviewSceneV1,
    state: &ReviewWorkspaceState,
    projection: &Projection,
) {
    let SelectionTarget::AuthoredObject(object_id) = &state.selection else {
        return;
    };
    if let Some(text) = scene
        .board_texts
        .iter()
        .find(|text| &text.object_id == object_id)
    {
        if !authored_visible(state) || !layer_visible(state, &text.layer_id) {
            return;
        }
        let rect = project_rect(board_text_hit_rect(text), projection);
        let halo = RectPx {
            x: rect.x - 4.0,
            y: rect.y - 4.0,
            width: rect.width + 8.0,
            height: rect.height + 8.0,
        };
        push_rect_border(out, halo, selected_silk_color(COMPONENT_SILK), 2.0);
        return;
    }
    if let Some(outline) = scene
        .outline
        .iter()
        .find(|outline| &outline.object_id == object_id)
    {
        if !authored_visible(state) || !layer_visible(state, &outline.layer_id) {
            return;
        }
        push_polyline_segments(
            out,
            &outline.path,
            projection,
            selected_mechanical_color(board_surface_color(BoardSurfaceRole::Edge)),
            3.0,
        );
        return;
    }
    if let Some(graphic) = scene
        .board_graphics
        .iter()
        .find(|graphic| &graphic.object_id == object_id)
    {
        if !authored_visible(state) || !layer_visible(state, &graphic.layer_id) {
            return;
        }
        push_polyline_segments(
            out,
            &graphic.path,
            projection,
            selected_mechanical_color(board_graphic_world_color(
                &graphic.layer_id,
                &scene.layers,
                false,
            )),
            3.0,
        );
    }
}

fn pad_matches_active_action(
    pad: &datum_gui_protocol::PadPrimitive,
    state: &ReviewWorkspaceState,
) -> bool {
    let Some(action) = state.selected_review_action() else {
        return false;
    };
    pad.pad_uuid == action.from_anchor_pad_uuid || pad.pad_uuid == action.to_anchor_pad_uuid
}

fn track_matches_active_action(
    track: &datum_gui_protocol::TrackPrimitive,
    state: &ReviewWorkspaceState,
) -> bool {
    let Some(action) = state.selected_review_action() else {
        return false;
    };
    let Some(net_uuid) = &track.net_uuid else {
        return false;
    };
    net_uuid == &action.net_uuid
}

fn via_matches_active_action(
    via: &datum_gui_protocol::ViaPrimitive,
    state: &ReviewWorkspaceState,
) -> bool {
    let Some(action) = state.selected_review_action() else {
        return false;
    };
    let Some(net_uuid) = &via.net_uuid else {
        return false;
    };
    net_uuid == &action.net_uuid
}

fn zone_matches_active_action(
    zone: &datum_gui_protocol::ZonePrimitive,
    state: &ReviewWorkspaceState,
) -> bool {
    let Some(action) = state.selected_review_action() else {
        return false;
    };
    let Some(net_uuid) = &zone.net_uuid else {
        return false;
    };
    net_uuid == &action.net_uuid
}

fn has_review_focus(state: &ReviewWorkspaceState) -> bool {
    state.selected_review_action().is_some()
}

fn component_overlaps_active_action(
    component: &datum_gui_protocol::ComponentBounds,
    state: &ReviewWorkspaceState,
) -> bool {
    let Some(action) = state.selected_review_action() else {
        return false;
    };
    point_in_rect(action.from, component.bounds) || point_in_rect(action.to, component.bounds)
}

fn point_in_rect(point: PointNm, rect: datum_gui_protocol::RectNm) -> bool {
    point.x >= rect.min_x && point.x <= rect.max_x && point.y >= rect.min_y && point.y <= rect.max_y
}

fn point_in_polygon_world(path: &[PointNm], point: PointNm) -> bool {
    if path.len() < 3 {
        return false;
    }
    let mut inside = false;
    let px = point.x as f64;
    let py = point.y as f64;
    let mut j = path.len() - 1;
    for i in 0..path.len() {
        let xi = path[i].x as f64;
        let yi = path[i].y as f64;
        let xj = path[j].x as f64;
        let yj = path[j].y as f64;
        let crosses_y = (yi > py) != (yj > py);
        if crosses_y {
            let denom = yj - yi;
            if denom.abs() <= f64::EPSILON {
                j = i;
                continue;
            }
            let x_at_y = (xj - xi) * (py - yi) / denom + xi;
            if px < x_at_y {
                inside = !inside;
            }
        }
        j = i;
    }
    inside
}

fn board_text_hit_rect(text: &BoardTextPrimitive) -> datum_gui_protocol::RectNm {
    let lines: Vec<&str> = text.text.lines().collect();
    let line_count = lines.len().max(1) as f64;
    let max_chars = lines
        .iter()
        .map(|line| line.chars().count())
        .max()
        .unwrap_or_else(|| text.text.chars().count())
        .max(1) as f64;
    let height = text.height_nm.max(1) as f64;
    let line_spacing = (text.line_spacing_ratio_ppm.max(1) as f64) / 1_000_000.0;
    let width_nm = (max_chars * height * 0.72).max(height * 0.5);
    let height_nm = (height + (line_count - 1.0) * height * line_spacing).max(height);
    let x0 = match text.h_align.as_str() {
        "center" => -width_nm * 0.5,
        "right" => -width_nm,
        _ => 0.0,
    };
    let y0 = match text.v_align.as_str() {
        "center" => -height_nm * 0.5,
        "top" => 0.0,
        _ => -height_nm,
    };
    let x1 = x0 + width_nm;
    let y1 = y0 + height_nm;
    let theta = (text.rotation_degrees as f64).to_radians();
    let (sin_t, cos_t) = theta.sin_cos();
    let mut min_x = f64::INFINITY;
    let mut min_y = f64::INFINITY;
    let mut max_x = f64::NEG_INFINITY;
    let mut max_y = f64::NEG_INFINITY;
    for (x, y) in [(x0, y0), (x1, y0), (x1, y1), (x0, y1)] {
        let rx = text.position.x as f64 + x * cos_t - y * sin_t;
        let ry = text.position.y as f64 + x * sin_t + y * cos_t;
        min_x = min_x.min(rx);
        min_y = min_y.min(ry);
        max_x = max_x.max(rx);
        max_y = max_y.max(ry);
    }
    let padding = (height * 0.25).max(250_000.0);
    datum_gui_protocol::RectNm {
        min_x: (min_x - padding).floor() as i64,
        min_y: (min_y - padding).floor() as i64,
        max_x: (max_x + padding).ceil() as i64,
        max_y: (max_y + padding).ceil() as i64,
    }
}

fn polyline_contains_world_point(path: &[PointNm], point: PointNm, half_width_nm: f32) -> bool {
    let px = point.x as f32;
    let py = point.y as f32;
    let threshold_sq = half_width_nm * half_width_nm;
    path.windows(2).any(|segment| {
        let ax = segment[0].x as f32;
        let ay = segment[0].y as f32;
        let bx = segment[1].x as f32;
        let by = segment[1].y as f32;
        let dx = bx - ax;
        let dy = by - ay;
        let len_sq = dx * dx + dy * dy;
        if len_sq <= 1.0 {
            let ddx = px - ax;
            let ddy = py - ay;
            return ddx * ddx + ddy * ddy <= threshold_sq;
        }
        let t = (((px - ax) * dx + (py - ay) * dy) / len_sq).clamp(0.0, 1.0);
        let cx = ax + dx * t;
        let cy = ay + dy * t;
        let ddx = px - cx;
        let ddy = py - cy;
        ddx * ddx + ddy * ddy <= threshold_sq
    })
}

fn component_graphic_matches_active_action(
    graphic: &ComponentGraphicPrimitive,
    scene: &BoardReviewSceneV1,
    state: &ReviewWorkspaceState,
) -> bool {
    scene.components.iter().any(|component| {
        component.component_uuid == graphic.component_uuid
            && component_overlaps_active_action(component, state)
    })
}

