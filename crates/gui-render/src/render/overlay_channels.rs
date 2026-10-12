//! Original proposal/preview and review geometry, with independent base/emphasis channels.
use super::*;

pub(super) fn width_to_px(width_nm: i64) -> f32 {
    ((width_nm as f32) / 120_000.0).clamp(0.9, 3.6)
}

pub(super) fn overlay_route_width_px(
    width_nm: Option<i64>,
    selected: bool,
    projection: Option<&Projection>,
) -> f32 {
    // If we have real width and a projection, use camera-aware sizing.
    // Preserve true proportional width down to a sub-pixel legibility floor so
    // distinct physical widths remain visually distinct at wide zoom.
    if let (Some(w), Some(proj)) = (width_nm, projection) {
        let projected = proj.world_length_to_px(w);
        let floor = if selected { 2.5 } else { 2.0 };
        return projected.max(floor).clamp(2.0, 32.0);
    }
    let base = width_nm.map(width_to_px).unwrap_or(2.4);
    let scaled = if selected { base * 3.2 } else { base * 2.0 };
    scaled.clamp(
        if selected { 4.5 } else { 3.2 },
        if selected { 10.0 } else { 7.0 },
    )
}

/// One emitter owns both baseline paint and selected geometry/hit footprints.
pub(super) fn push_overlay_channels(
    out: &mut Vec<Quad>,
    plan: &mut native_selection_overlay::cue_plan::PanePlan,
    overlay: &ProposalOverlayPrimitive,
    projection: &Projection,
    color: [f32; 3],
    selected: bool,
) -> Vec<RectPx> {
    let base_hits = {
        let mut output = plan.base_output(out, selected);
        push_overlay(&mut output, overlay, projection, color, false, false)
    };
    if selected {
        let start = out.len();
        let hits = push_overlay(out, overlay, projection, color, true, false);
        plan.finish_detail(out, start);
        hits
    } else {
        base_hits
    }
}

pub(super) fn push_review_channels(
    out: &mut Vec<Quad>,
    plan: &mut native_selection_overlay::cue_plan::PanePlan,
    review: &datum_gui_protocol::ReviewPrimitive,
    projection: &Projection,
    selected: bool,
) {
    {
        let mut output = plan.base_output(out, selected);
        push_review_geometry(&mut output, review, projection, false);
    }
    if selected {
        let start = out.len();
        push_review_geometry(out, review, projection, true);
        plan.finish_detail(out, start);
    }
}

fn push_review_geometry(
    out: &mut impl Output<Quad>,
    review: &datum_gui_protocol::ReviewPrimitive,
    projection: &Projection,
    active: bool,
) {
    push_dashed_polyline_segments(
        out,
        &review.path,
        projection,
        DIAGNOSTIC_UNDERLAY,
        if active { 2.1 } else { 1.6 },
        10.0,
        6.0,
    );
    push_dashed_polyline_segments(
        out,
        &review.path,
        projection,
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
            projection,
            if active {
                DIAGNOSTIC_FOCUS
            } else {
                DIAGNOSTIC_BASE
            },
            if active { 4.0 } else { 3.0 },
        );
    }
}

pub(super) fn push_overlay(
    out: &mut impl Output<Quad>,
    overlay: &ProposalOverlayPrimitive,
    projection: &Projection,
    color: [f32; 3],
    selected: bool,
    editor_move_preview: bool,
) -> Vec<RectPx> {
    if editor_move_preview {
        return push_overlay_move_preview(out, overlay, projection, color, selected);
    }
    let layer_color = proposal_layer_color(overlay.layer_id.as_deref());
    let outer_color = if selected {
        PROPOSAL_OUTER
    } else {
        mix_color(color, layer_color, 0.45)
    };
    let underlay_color = if selected {
        PROPOSAL_UNDERLAY
    } else {
        mix_color(PROPOSAL_UNDERLAY, layer_color, 0.18)
    };
    let fill_color = if selected { PROPOSAL_FOCUS } else { color };
    match overlay.primitive_kind.as_str() {
        "anchor_marker" => {
            let outer_size = if selected { 17.0 } else { 12.0 };
            let ring_size = if selected { 10.0 } else { 7.0 };
            let core_size = if selected { 4.2 } else { 3.0 };
            let mut rects = push_points(
                out,
                &overlay.path,
                projection,
                if selected {
                    PROPOSAL_UNDERLAY
                } else {
                    [0.30, 0.22, 0.12]
                },
                outer_size,
            );
            rects.extend(push_points(
                out,
                &overlay.path,
                projection,
                if selected {
                    PROPOSAL_FOCUS
                } else {
                    PROPOSAL_ANCHOR_RING
                },
                ring_size,
            ));
            rects.extend(push_points(
                out,
                &overlay.path,
                projection,
                PROPOSAL_ANCHOR_CORE,
                core_size,
            ));
            rects
        }
        "via" => {
            let Some(center) = overlay.path.first().copied() else {
                return Vec::new();
            };
            let diameter_nm = overlay
                .diameter_nm
                .or(overlay.width_nm)
                .unwrap_or(600_000)
                .max(1);
            let drill_nm = overlay.drill_nm.unwrap_or(diameter_nm / 2).max(1);
            let radius = (diameter_nm as f32 * 0.5).round() as i64;
            let drill_radius = (drill_nm as f32 * 0.5).round() as i64;
            let outer_rect = datum_gui_protocol::RectNm {
                min_x: center.x - radius,
                min_y: center.y - radius,
                max_x: center.x + radius,
                max_y: center.y + radius,
            };
            push_world_ellipse_nm(out, outer_rect, outer_color, 96);
            let ring_inset = (diameter_nm as f32 * 0.14).round().max(1.0);
            push_world_ellipse_nm(
                out,
                world_inset_rect(outer_rect, ring_inset),
                fill_color,
                96,
            );
            out.role(crate::presentation_ranges::Role::Void);
            push_world_ellipse_nm(
                out,
                datum_gui_protocol::RectNm {
                    min_x: center.x - drill_radius,
                    min_y: center.y - drill_radius,
                    max_x: center.x + drill_radius,
                    max_y: center.y + drill_radius,
                },
                underlay_color,
                96,
            );
            out.role(crate::presentation_ranges::Role::Material);
            vec![project_rect(outer_rect, projection)]
        }
        _ => {
            let route_width = overlay_route_width_px(overlay.width_nm, selected, Some(projection));
            let underlay_width = if selected {
                route_width + 5.2
            } else {
                route_width + 1.8
            };
            let outer_width = if selected {
                route_width + 2.2
            } else {
                route_width + 0.55
            };
            let inner_width = if selected {
                route_width + 0.45
            } else {
                route_width.max(1.5)
            };
            let mut hit_rects = push_polyline_segments(
                out,
                &overlay.path,
                projection,
                underlay_color,
                underlay_width,
            );
            if selected || overlay.path.len() == 2 {
                hit_rects.extend(push_polyline_endcaps(
                    out,
                    &overlay.path,
                    projection,
                    outer_color,
                    outer_width,
                    (route_width * 2.7).clamp(10.0, 18.0),
                ));
            }
            if let (Some(first), Some(last)) = (overlay.path.first(), overlay.path.last()) {
                let endpoint_radius = if selected {
                    (route_width + 1.8).clamp(4.8, 8.0)
                } else {
                    (route_width + 1.0).clamp(3.8, 5.8)
                };
                hit_rects.extend(push_points(
                    out,
                    &[*first, *last],
                    projection,
                    outer_color,
                    endpoint_radius,
                ));
                hit_rects.extend(push_points(
                    out,
                    &[*first, *last],
                    projection,
                    fill_color,
                    (endpoint_radius * 0.42).clamp(1.8, 3.2),
                ));
            }
            hit_rects.extend(push_polyline_segments(
                out,
                &overlay.path,
                projection,
                outer_color,
                outer_width,
            ));
            hit_rects.extend(push_polyline_segments(
                out,
                &overlay.path,
                projection,
                if selected { PROPOSAL_FOCUS } else { color },
                inner_width,
            ));
            if selected {
                hit_rects.extend(push_polyline_endcaps(
                    out,
                    &overlay.path,
                    projection,
                    PROPOSAL_FOCUS,
                    route_width + 0.8,
                    (route_width * 2.0).clamp(8.0, 14.0),
                ));
            }
            let corner_fill = if selected {
                (inner_width - 0.25).max(1.2)
            } else {
                (inner_width - 0.35).max(1.0)
            };
            if overlay.path.len() > 2 {
                hit_rects.extend(push_points(
                    out,
                    &overlay.path[1..overlay.path.len() - 1],
                    projection,
                    fill_color,
                    corner_fill,
                ));
            }
            hit_rects
        }
    }
}

fn push_overlay_move_preview(
    out: &mut impl Output<Quad>,
    overlay: &ProposalOverlayPrimitive,
    projection: &Projection,
    color: [f32; 3],
    selected: bool,
) -> Vec<RectPx> {
    match overlay.primitive_kind.as_str() {
        "anchor_marker" => push_points(
            out,
            &overlay.path,
            projection,
            if selected {
                PROPOSAL_FOCUS
            } else {
                AUTHOR_RELATED
            },
            if selected { 5.0 } else { 4.0 },
        ),
        _ => {
            let guide_color = if selected {
                mix_color(PROPOSAL_FOCUS, PROPOSAL_CENTERLINE, 0.35)
            } else {
                mix_color(color, PROPOSAL_CENTERLINE, 0.55)
            };
            let underlay = if selected {
                mix_color(PROPOSAL_UNDERLAY, guide_color, 0.25)
            } else {
                mix_color(DIAGNOSTIC_UNDERLAY, guide_color, 0.18)
            };
            let base_width = if selected { 1.4 } else { 1.1 };
            let mut rects = push_dashed_polyline_segments(
                out,
                &overlay.path,
                projection,
                underlay,
                base_width + 0.9,
                10.0,
                6.0,
            );
            rects.extend(push_dashed_polyline_segments(
                out,
                &overlay.path,
                projection,
                guide_color,
                base_width,
                10.0,
                6.0,
            ));
            rects.extend(push_points(
                out,
                &overlay.path,
                projection,
                guide_color,
                if selected { 3.4 } else { 3.0 },
            ));
            rects
        }
    }
}
