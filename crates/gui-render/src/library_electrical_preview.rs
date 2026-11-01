//! Read-only Unit rows and the declared rectangular Symbol body / pin stubs.
use crate::*;
use datum_gui_protocol::{LibraryJsonValue as Value, PointNm, ReviewWorkspaceState, SceneBounds};

pub(super) fn render(
    state: &ReviewWorkspaceState,
    kind: &str,
    raw: &Value,
    rect: RectPx,
    quads: &mut Vec<Quad>,
    text: &mut Vec<TextRun>,
) {
    if kind == "units" {
        let mut y = rect.y;
        for pin in raw
            .get("pins")
            .and_then(Value::as_object)
            .into_iter()
            .flat_map(|p| p.values())
        {
            let name = pin
                .get("name")
                .and_then(Value::as_str)
                .unwrap_or("Unnamed pin");
            let direction = pin
                .get("electrical_type")
                .or_else(|| pin.get("direction"))
                .and_then(Value::as_str)
                .unwrap_or("Unspecified");
            draw_text_clipped(
                &format!("{name} · {direction}"),
                rect.x + 8.,
                y,
                12.,
                TEXT_PRIMARY,
                TextFace::Ui,
                rect,
                text,
            );
            y += 20.;
        }
        return;
    }
    if raw
        .get("drawings")
        .and_then(Value::as_array)
        .is_none_or(|d| d.len() != 1)
        || raw
            .get("fields")
            .and_then(Value::as_array)
            .is_some_and(|f| !f.is_empty())
        || raw
            .get("pin_anchors")
            .and_then(Value::as_array)
            .is_none_or(|anchors| {
                anchors.iter().any(|a| {
                    endpoints(a).is_none()
                        || a.pointer("/style/decoration")
                            .and_then(Value::as_str)
                            .is_some_and(|d| d != "none")
                })
            })
    {
        unavailable(rect, text);
        return;
    }
    let body = raw.pointer("/drawings/0/Rect");
    let coordinate = |path: &str| body.and_then(|r| r.pointer(path)).and_then(Value::as_i64);
    let (Some(min_x), Some(min_y), Some(max_x), Some(max_y), Some(width)) = (
        coordinate("/min/x"),
        coordinate("/min/y"),
        coordinate("/max/x"),
        coordinate("/max/y"),
        coordinate("/width"),
    ) else {
        unavailable(rect, text);
        return;
    };
    if max_x <= min_x || max_y <= min_y || width <= 0 {
        unavailable(rect, text);
        return;
    }
    let body = datum_gui_protocol::RectNm {
        min_x,
        min_y,
        max_x,
        max_y,
    };
    let mut bounds = SceneBounds {
        min_x,
        min_y,
        max_x,
        max_y,
    };
    let unit = raw.get("unit").and_then(Value::as_str);
    let pins = state
        .library
        .entries
        .iter()
        .find(|e| e.kind == "units" && unit == Some(e.id.to_string().as_str()))
        .and_then(|e| e.raw.get("pins"))
        .and_then(Value::as_object);
    let mut anchors = Vec::new();
    for anchor in raw
        .get("pin_anchors")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        let Some((from, to)) = endpoints(anchor) else {
            continue;
        };
        for p in [from, to] {
            bounds.min_x = bounds.min_x.min(p.x);
            bounds.min_y = bounds.min_y.min(p.y);
            bounds.max_x = bounds.max_x.max(p.x);
            bounds.max_y = bounds.max_y.max(p.y);
        }
        let name = anchor
            .get("pin")
            .and_then(Value::as_str)
            .and_then(|id| pins.and_then(|p| p.get(id)))
            .and_then(|pin| pin.get("name"))
            .and_then(Value::as_str)
            .unwrap_or("Unresolved pin");
        anchors.push((from, to, name));
    }
    if bounds.max_x.checked_sub(bounds.min_x).is_none()
        || bounds.max_y.checked_sub(bounds.min_y).is_none()
    {
        unavailable(rect, text);
        return;
    }
    let mut camera = CameraState::fit_to_bounds(&bounds);
    camera.zoom = 0.7;
    let projection = Projection::new(rect, &bounds, camera);
    push_rect_border(
        quads,
        projection.project_rect(body),
        TEXT_PRIMARY,
        projection.world_length_to_px(width).max(1.),
    );
    for (from, to, name) in anchors {
        let (x1, y1) = projection.project_point(from);
        let (x2, y2) = projection.project_point(to);
        quads.push(Quad::from_rect(
            RectPx {
                x: x1.min(x2) - 0.5,
                y: y1.min(y2) - 0.5,
                width: (x2 - x1).abs().max(1.),
                height: (y2 - y1).abs().max(1.),
            },
            TEXT_MUTED,
        ));
        quads.push(Quad::from_rect(
            RectPx {
                x: x1 - 2.,
                y: y1 - 2.,
                width: 4.,
                height: 4.,
            },
            TEXT_ACCENT,
        ));
        draw_text_clipped(
            name,
            x2 + 4.,
            y2 - 6.,
            11.,
            TEXT_PRIMARY,
            TextFace::Ui,
            rect,
            text,
        );
    }
}

fn unavailable(rect: RectPx, text: &mut Vec<TextRun>) {
    draw_text_clipped(
        "Preview unavailable · unsupported Symbol geometry or styling",
        rect.x + 8.,
        rect.y + 6.,
        11.,
        TEXT_MUTED,
        TextFace::Ui,
        rect,
        text,
    );
}

fn endpoints(anchor: &Value) -> Option<(PointNm, PointNm)> {
    let from = PointNm {
        x: anchor.pointer("/position/x")?.as_i64()?,
        y: anchor.pointer("/position/y")?.as_i64()?,
    };
    let length = anchor.pointer("/style/length_nm")?.as_i64()?;
    if length < 0 {
        return None;
    }
    let to = match anchor.pointer("/style/orientation")?.as_str()? {
        "Left" => PointNm {
            x: from.x.checked_sub(length)?,
            y: from.y,
        },
        "Right" => PointNm {
            x: from.x.checked_add(length)?,
            y: from.y,
        },
        "Up" => PointNm {
            x: from.x,
            y: from.y.checked_sub(length)?,
        },
        "Down" => PointNm {
            x: from.x,
            y: from.y.checked_add(length)?,
        },
        _ => return None,
    };
    Some((from, to))
}
