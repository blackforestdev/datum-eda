//! Dimensional envelope only: Package schema has no exact body/terminal outline.
use super::*;

pub(super) fn render(raw: &Value, rect: RectPx, quads: &mut Vec<Quad>, text: &mut Vec<TextRun>) {
    let starts = (quads.len(), text.len());
    let x = raw.pointer("/body_dimensions/x_nm").and_then(Value::as_i64);
    let y = raw.pointer("/body_dimensions/y_nm").and_then(Value::as_i64);
    let dimensions = |v: Option<i64>| v.map_or("absent".into(), |n| format!("{n} nm"));
    let terminals = raw.get("terminals").and_then(Value::as_object);
    let messages = [
        format!("Body nominal X {} · Y {}", dimensions(x), dimensions(y)),
        format!(
            "{} named terminals · positions unavailable",
            terminals.map_or(0, |t| t.len())
        ),
        "Dimensional envelope only · exact outline unavailable".into(),
    ];
    let heights: Vec<_> = messages
        .iter()
        .map(|s| {
            crate::text_metrics::measured_text_run_height_px(
                s,
                rect.width.max(1.),
                10.,
                TextFace::Ui,
            )
            .unwrap_or(14.)
                + 4.
        })
        .collect();
    let geometry = RectPx {
        height: (rect.height - heights.iter().sum::<f32>()).max(0.),
        ..rect
    };
    if let (Some(x), Some(y)) = (x.filter(|n| *n > 0), y.filter(|n| *n > 0)) {
        if geometry.height > 10. {
            let bounds = SceneBounds {
                min_x: 0,
                min_y: 0,
                max_x: x,
                max_y: y,
            };
            let mut camera = CameraState::fit_to_bounds(&bounds);
            camera.zoom = 0.75;
            let projection = Projection::new(geometry, &bounds, camera);
            let body = projection.project_rect(datum_gui_protocol::RectNm {
                min_x: 0,
                min_y: 0,
                max_x: x,
                max_y: y,
            });
            push_rect_border(quads, body, TEXT_ACCENT, 1.);
        }
    }
    let mut at = geometry.y + geometry.height;
    for (message, height) in messages.iter().zip(heights) {
        draw_text_clipped(
            message,
            rect.x,
            at,
            10.,
            TEXT_MUTED,
            TextFace::Ui,
            RectPx {
                y: at,
                height,
                ..rect
            },
            text,
        );
        text.last_mut().unwrap().layout_size = Some((rect.width.max(1.), height - 4.));
        at += height;
    }
    crate::hit_clipping::clip_content(quads, text, &mut Vec::new(), starts.0, starts.1, 0, rect);
}
