use datum_gui_protocol::{
    ArtifactPreviewViewportState, ProductionArtifactFilePreviewSummary,
    ProductionArtifactPreviewPoint, ProductionArtifactPreviewPrimitive,
};

use super::{PANEL_CARD_BORDER, Quad, RectPx, TEXT_ACCENT};
use super::{push_projected_ellipse, push_rect_border};

pub(super) fn render_artifact_preview_viewport(
    preview: &ProductionArtifactFilePreviewSummary,
    viewport: &ArtifactPreviewViewportState,
    rect: RectPx,
    panel_quads: &mut Vec<Quad>,
) {
    panel_quads.push(Quad::from_rect(rect, [0.07, 0.09, 0.10]));
    push_rect_border(panel_quads, rect, PANEL_CARD_BORDER, 1.0);
    let plot_rect = inset_preview_rect(rect, 8.0, 24.0, 8.0, 8.0);
    push_preview_grid(panel_quads, plot_rect);
    let Some(bounds) = preview_bounds(&preview.primitives) else {
        return;
    };
    for primitive in &preview.primitives {
        if !primitive_visible(primitive, viewport) {
            continue;
        }
        render_preview_primitive(panel_quads, primitive, plot_rect, bounds, viewport);
    }
}

fn primitive_visible(
    primitive: &ProductionArtifactPreviewPrimitive,
    viewport: &ArtifactPreviewViewportState,
) -> bool {
    match primitive.kind.as_str() {
        "drill_hit" => viewport.show_drills,
        "stroke" | "region" | "flash" => viewport.show_geometry,
        _ => true,
    }
}

fn inset_preview_rect(rect: RectPx, left: f32, top: f32, right: f32, bottom: f32) -> RectPx {
    RectPx {
        x: rect.x + left,
        y: rect.y + top,
        width: (rect.width - left - right).max(1.0),
        height: (rect.height - top - bottom).max(1.0),
    }
}

fn push_preview_grid(panel_quads: &mut Vec<Quad>, rect: RectPx) {
    panel_quads.push(Quad::from_rect(rect, [0.09, 0.12, 0.13]));
    let color = [0.13, 0.18, 0.18];
    for index in 1..4 {
        let x = rect.x + rect.width * index as f32 / 4.0;
        panel_quads.push(Quad::from_rect(
            RectPx {
                x,
                y: rect.y,
                width: 1.0,
                height: rect.height,
            },
            color,
        ));
        let y = rect.y + rect.height * index as f32 / 4.0;
        panel_quads.push(Quad::from_rect(
            RectPx {
                x: rect.x,
                y,
                width: rect.width,
                height: 1.0,
            },
            color,
        ));
    }
}

#[derive(Debug, Clone, Copy)]
struct PreviewBounds {
    min_x: i64,
    min_y: i64,
    max_x: i64,
    max_y: i64,
}

fn preview_bounds(primitives: &[ProductionArtifactPreviewPrimitive]) -> Option<PreviewBounds> {
    let mut bounds = None::<PreviewBounds>;
    for point in primitives
        .iter()
        .flat_map(|primitive| primitive.points.iter())
    {
        bounds = Some(match bounds {
            Some(current) => PreviewBounds {
                min_x: current.min_x.min(point.x_nm),
                min_y: current.min_y.min(point.y_nm),
                max_x: current.max_x.max(point.x_nm),
                max_y: current.max_y.max(point.y_nm),
            },
            None => PreviewBounds {
                min_x: point.x_nm,
                min_y: point.y_nm,
                max_x: point.x_nm,
                max_y: point.y_nm,
            },
        });
    }
    bounds
}

fn render_preview_primitive(
    panel_quads: &mut Vec<Quad>,
    primitive: &ProductionArtifactPreviewPrimitive,
    rect: RectPx,
    bounds: PreviewBounds,
    viewport: &ArtifactPreviewViewportState,
) {
    match primitive.kind.as_str() {
        "stroke" => {
            let width = primitive
                .aperture_diameter_nm
                .map(|diameter| preview_length_to_px(diameter, rect, bounds).max(1.5))
                .unwrap_or(2.0);
            for segment in primitive.points.windows(2) {
                push_preview_segment(
                    panel_quads,
                    preview_project(segment[0], rect, bounds, viewport),
                    preview_project(segment[1], rect, bounds, viewport),
                    width,
                    [0.92, 0.58, 0.26],
                );
            }
        }
        "region" => {
            for segment in primitive.points.windows(2) {
                push_preview_segment(
                    panel_quads,
                    preview_project(segment[0], rect, bounds, viewport),
                    preview_project(segment[1], rect, bounds, viewport),
                    1.5,
                    TEXT_ACCENT,
                );
            }
        }
        "flash" => {
            if let Some(point) = primitive.points.first().copied() {
                let center = preview_project(point, rect, bounds, viewport);
                let diameter = primitive
                    .aperture_diameter_nm
                    .map(|value| preview_length_to_px(value, rect, bounds))
                    .unwrap_or_else(|| {
                        primitive
                            .aperture_width_nm
                            .map(|value| preview_length_to_px(value, rect, bounds))
                            .unwrap_or(6.0)
                    })
                    .clamp(4.0, 18.0);
                push_projected_ellipse(
                    panel_quads,
                    RectPx {
                        x: center.0 - diameter * 0.5,
                        y: center.1 - diameter * 0.5,
                        width: diameter,
                        height: diameter,
                    },
                    [0.91, 0.80, 0.58],
                    24,
                );
            }
        }
        "drill_hit" => {
            if let Some(point) = primitive.points.first().copied() {
                let center = preview_project(point, rect, bounds, viewport);
                let diameter = primitive
                    .diameter_mm
                    .as_deref()
                    .and_then(|value| value.parse::<f32>().ok())
                    .map(|mm| (mm * 1_000_000.0) as i64)
                    .map(|value| preview_length_to_px(value, rect, bounds))
                    .unwrap_or(6.0)
                    .clamp(4.0, 14.0);
                push_projected_ellipse(
                    panel_quads,
                    RectPx {
                        x: center.0 - diameter * 0.5,
                        y: center.1 - diameter * 0.5,
                        width: diameter,
                        height: diameter,
                    },
                    [0.45, 0.76, 0.86],
                    20,
                );
            }
        }
        _ => {}
    }
}

fn preview_project(
    point: ProductionArtifactPreviewPoint,
    rect: RectPx,
    bounds: PreviewBounds,
    viewport: &ArtifactPreviewViewportState,
) -> (f32, f32) {
    let width = (bounds.max_x - bounds.min_x).max(1) as f32;
    let height = (bounds.max_y - bounds.min_y).max(1) as f32;
    let fit_scale = (rect.width / width)
        .min(rect.height / height)
        .max(0.000_001);
    let scale = fit_scale * (viewport.zoom_ppm.max(1) as f32 / 1_000_000.0);
    let used_width = width * scale;
    let used_height = height * scale;
    let pan_x = rect.width * viewport.pan_x_ppm as f32 / 1_000_000.0;
    let pan_y = rect.height * viewport.pan_y_ppm as f32 / 1_000_000.0;
    let x = rect.x
        + (rect.width - used_width) * 0.5
        + (point.x_nm - bounds.min_x) as f32 * scale
        + pan_x;
    let y = rect.y
        + (rect.height - used_height) * 0.5
        + (bounds.max_y - point.y_nm) as f32 * scale
        + pan_y;
    (x, y)
}

fn preview_length_to_px(length_nm: i64, rect: RectPx, bounds: PreviewBounds) -> f32 {
    let width = (bounds.max_x - bounds.min_x).max(1) as f32;
    let height = (bounds.max_y - bounds.min_y).max(1) as f32;
    length_nm as f32
        * (rect.width / width)
            .min(rect.height / height)
            .max(0.000_001)
}

fn push_preview_segment(
    panel_quads: &mut Vec<Quad>,
    from: (f32, f32),
    to: (f32, f32),
    width: f32,
    color: [f32; 3],
) {
    let dx = to.0 - from.0;
    let dy = to.1 - from.1;
    let len = (dx * dx + dy * dy).sqrt();
    if len <= 0.001 {
        return;
    }
    let nx = -dy / len * width * 0.5;
    let ny = dx / len * width * 0.5;
    panel_quads.push(Quad {
        points: [
            (from.0 + nx, from.1 + ny),
            (to.0 + nx, to.1 + ny),
            (to.0 - nx, to.1 - ny),
            (from.0 - nx, from.1 - ny),
        ],
        color,
    });
}
