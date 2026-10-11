//! Read-only configured geometry preview using shared projection and primitives.
use crate::*;
use datum_gui_protocol::LibraryJsonValue as Value;
use datum_gui_protocol::{ReviewWorkspaceState, SceneBounds};

pub(crate) fn render(
    state: &ReviewWorkspaceState,
    kind: &str,
    raw: &Value,
    rect: RectPx,
    quads: &mut Vec<Quad>,
    text: &mut Vec<TextRun>,
) {
    if rect.height < 25. || rect.width < 10. {
        return;
    }
    if !matches!(kind, "footprints" | "padstacks" | "packages") {
        draw_text_clipped(
            "Visual geometry unavailable · inspect stored metadata / attachments",
            rect.x,
            rect.y,
            11.,
            TEXT_MUTED,
            TextFace::Ui,
            rect,
            text,
        );
        return;
    }
    let starts = (quads.len(), text.len());
    let mut pads = Vec::new();
    if kind == "padstacks" {
        pads.push((0, 0, raw.clone()));
    } else if matches!(kind, "footprints" | "packages") {
        for pad in raw
            .get("pads")
            .and_then(Value::as_object)
            .into_iter()
            .flat_map(|p| p.values())
        {
            let id = pad
                .get("padstack")
                .and_then(Value::as_str)
                .and_then(|s| uuid::Uuid::parse_str(s).ok());
            let choices = state
                .library
                .entries
                .iter()
                .filter(|e| Some(e.id) == id && e.kind == "padstacks")
                .collect::<Vec<_>>();
            if choices.len() == 1 && choices[0].warning.is_none() {
                if let (Some(x), Some(y)) = (
                    pad.pointer("/position/x").and_then(Value::as_i64),
                    pad.pointer("/position/y").and_then(Value::as_i64),
                ) {
                    pads.push((x, y, choices[0].raw.clone()));
                }
            }
        }
    }
    if pads.is_empty() {
        draw_text_clipped(
            if kind == "packages" {
                "Package geometry unavailable · inspect stored body metadata"
            } else {
                "Geometry unavailable · no resolved copper"
            },
            rect.x,
            rect.y + 4.,
            11.,
            TEXT_MUTED,
            TextFace::Ui,
            rect,
            text,
        );
    }
    let mut shapes = Vec::new();
    let mut messages = Vec::new();
    for (x, y, stack) in pads {
        let aperture = stack.get("aperture").unwrap_or(&Value::Null);
        let shape = if let Some(d) = aperture
            .pointer("/circle/diameter_nm")
            .and_then(Value::as_i64)
        {
            Some((d, d, true))
        } else {
            aperture
                .pointer("/rect/width_nm")
                .and_then(Value::as_i64)
                .zip(aperture.pointer("/rect/height_nm").and_then(Value::as_i64))
                .map(|(w, h)| (w, h, false))
        };
        if let Some((w, h, circle)) = shape.filter(|(w, h, _)| *w > 0 && *h > 0) {
            if let (Some(min_x), Some(min_y), Some(max_x), Some(max_y)) = (
                x.checked_sub(w / 2),
                y.checked_sub(h / 2),
                x.checked_add(w / 2),
                y.checked_add(h / 2),
            ) {
                shapes.push((
                    datum_gui_protocol::RectNm {
                        min_x,
                        min_y,
                        max_x,
                        max_y,
                    },
                    circle,
                    stack.get("drill_nm").and_then(Value::as_i64),
                ));
            }
        }
    }
    let pad = raw
        .get("pads")
        .and_then(Value::as_object)
        .and_then(|p| p.values().nth(state.library.preview_pad));
    let name = pad
        .and_then(|p| p.get("name"))
        .or_else(|| raw.get("name"))
        .and_then(Value::as_str)
        .unwrap_or("Unavailable");
    if kind == "packages" {
        messages
            .push("Package geometry · process inspection belongs to Footprint / Padstack".into());
    } else {
        match state
            .library
            .process_inspection(kind, raw, state.library.preview_pad)
        {
            Ok(e) => {
                messages.push(format!("{name} · Mask {}", aperture_label(&e.mask)));
                messages.push(format!("Paste {}", aperture_label(&e.paste)));
                messages.push(format!(
                    "Basis: {}",
                    e.resolved_basis.as_ref().map_or(
                        "Explicit authored policies".into(),
                        |b| format!(
                            "{} · paste ratio {} ppm",
                            b.declared_basis, b.paste_ratio_ppm
                        )
                    )
                ));
                messages.push(format!(
                    "Standards basis: {}",
                    e.standards_basis.as_deref().unwrap_or("Unavailable")
                ));
            }
            Err(reason) => {
                messages.push(format!("{name} · Mask/paste unresolved"));
                messages.push(reason);
                messages.push("Complete reason: Inspect reason → Console".into());
            }
        }
    }
    let heights: Vec<f32> = messages
        .iter()
        .map(|m| {
            crate::text_metrics::measured_text_run_height_px(
                m,
                rect.width.max(1.),
                10.,
                TextFace::Ui,
            )
            .unwrap_or(14.)
                + 4.
        })
        .collect();
    let geometry = RectPx {
        height: (rect.height - heights.iter().sum::<f32>() - 4.).max(0.),
        ..rect
    };
    if !shapes.is_empty() && geometry.height > 10. {
        let bounds = SceneBounds {
            min_x: shapes.iter().map(|s| s.0.min_x).min().unwrap(),
            min_y: shapes.iter().map(|s| s.0.min_y).min().unwrap(),
            max_x: shapes.iter().map(|s| s.0.max_x).max().unwrap(),
            max_y: shapes.iter().map(|s| s.0.max_y).max().unwrap(),
        };
        let mut camera = CameraState::fit_to_bounds(&bounds);
        camera.zoom = 0.75;
        let projection = Projection::new(geometry, &bounds, camera);
        for (bounds, circle, drill) in shapes {
            let r = projection.project_rect(bounds);
            if circle {
                push_projected_ellipse(quads, r, design_tokens::content::COPPER_FRONT, 48);
            } else {
                quads.push(Quad::from_rect(r, design_tokens::content::COPPER_FRONT));
            }
            if let Some(d) = drill.filter(|d| *d > 0) {
                let diameter = projection.world_length_to_px(d);
                push_projected_ellipse(
                    quads,
                    RectPx {
                        x: r.x + r.width / 2. - diameter / 2.,
                        y: r.y + r.height / 2. - diameter / 2.,
                        width: diameter,
                        height: diameter,
                    },
                    design_tokens::chrome::SURFACE_01,
                    48,
                );
            }
        }
    }
    let mut y = geometry.y + geometry.height + 4.;
    for (message, height) in messages.iter().zip(heights) {
        draw_text_clipped(
            message,
            rect.x,
            y,
            10.,
            TEXT_MUTED,
            TextFace::Ui,
            RectPx { y, height, ..rect },
            text,
        );
        text.last_mut().unwrap().layout_size = Some((rect.width.max(1.), height - 4.));
        y += height;
    }
    let mut hits = Vec::new();
    crate::hit_clipping::clip_content(quads, text, &mut hits, starts.0, starts.1, 0, rect);
}

fn aperture_label(aperture: &eda_engine::pool::process::EffectiveProcessAperture) -> String {
    use eda_engine::pool::process::EffectiveProcessAperture as A;
    fn exact(q: eda_engine::board::quadratic3::Quadratic3) -> String {
        if let Ok(nm) = q.integer_nm() {
            format!("{nm} nm")
        } else {
            format!(
                "({}/{} + {}/{}√3) nm",
                q.a.numerator, q.a.denominator, q.b.numerator, q.b.denominator
            )
        }
    }
    match aperture {
        A::Suppressed => "suppressed".into(),
        A::Circle { diameter_nm } => format!("Circle Ø {}", exact(*diameter_nm)),
        A::Rect {
            width_nm,
            height_nm,
        } => format!("Rect {} × {}", exact(*width_nm), exact(*height_nm)),
    }
}
