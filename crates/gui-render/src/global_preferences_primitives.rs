//! Small screen-space primitives used by the protected Global Preferences grammar.

use super::*;
#[path = "render/control_mesh.rs"]
mod control_mesh;
pub(crate) use control_mesh::{ControlMeshCache, ControlPainter};

const ROUNDED_RECT_CORNER_SEGMENTS: usize = 4;

pub(super) fn push_rounded_rect_with_border(
    quads: &mut ControlPainter<'_>,
    rect: RectPx,
    fill: [f32; 3],
    border: [f32; 3],
    thickness: f32,
    radius: f32,
) {
    quads.rounded_fill(rect, border, radius, thickness);
    let inset = thickness
        .max(0.0)
        .min(rect.width * 0.5)
        .min(rect.height * 0.5);
    if inset <= 0.0 {
        return;
    }
    let inner = RectPx {
        x: rect.x + inset,
        y: rect.y + inset,
        width: rect.width - inset * 2.0,
        height: rect.height - inset * 2.0,
    };
    if inner.width > 0.0 && inner.height > 0.0 {
        quads.rounded_fill(inner, fill, (radius - inset).max(0.0), thickness);
    }
}

pub(super) fn rounded_rect_points(rect: RectPx, radius: f32) -> Vec<(f32, f32)> {
    let radius = radius.max(0.0).min(rect.width * 0.5).min(rect.height * 0.5);
    if radius == 0.0 {
        return vec![
            (rect.x, rect.y),
            (rect.x + rect.width, rect.y),
            (rect.x + rect.width, rect.y + rect.height),
            (rect.x, rect.y + rect.height),
        ];
    }

    let mut points = Vec::with_capacity((ROUNDED_RECT_CORNER_SEGMENTS + 1) * 4);
    let corners = [
        (
            rect.x + rect.width - radius,
            rect.y + radius,
            -std::f32::consts::FRAC_PI_2,
        ),
        (
            rect.x + rect.width - radius,
            rect.y + rect.height - radius,
            0.0,
        ),
        (
            rect.x + radius,
            rect.y + rect.height - radius,
            std::f32::consts::FRAC_PI_2,
        ),
        (rect.x + radius, rect.y + radius, std::f32::consts::PI),
    ];
    static DIRECTIONS: std::sync::OnceLock<[(f32, f32); (ROUNDED_RECT_CORNER_SEGMENTS + 1) * 4]> =
        std::sync::OnceLock::new();
    let directions = DIRECTIONS.get_or_init(|| {
        let starts = [
            -std::f32::consts::FRAC_PI_2,
            0.0,
            std::f32::consts::FRAC_PI_2,
            std::f32::consts::PI,
        ];
        std::array::from_fn(|index| {
            let corner = index / (ROUNDED_RECT_CORNER_SEGMENTS + 1);
            let step = index % (ROUNDED_RECT_CORNER_SEGMENTS + 1);
            let angle = starts[corner]
                + std::f32::consts::FRAC_PI_2 * step as f32 / ROUNDED_RECT_CORNER_SEGMENTS as f32;
            (angle.cos(), angle.sin())
        })
    });
    for ((center_x, center_y, _), arc) in corners
        .into_iter()
        .zip(directions.chunks_exact(ROUNDED_RECT_CORNER_SEGMENTS + 1))
    {
        for &(cos, sin) in arc {
            points.push((center_x + radius * cos, center_y + radius * sin));
        }
    }
    points
}

pub(super) fn push_dashed_rect_border(
    quads: &mut ControlPainter<'_>,
    rect: RectPx,
    color: [f32; 3],
) {
    let dash: f32 = 6.0;
    let gap: f32 = 4.0;
    let mut horizontal = |y: f32| {
        let mut x = rect.x;
        while x < rect.x + rect.width {
            quads.push(Quad::from_rect(
                RectPx {
                    x,
                    y,
                    width: dash.min(rect.x + rect.width - x),
                    height: 1.0,
                },
                color,
            ));
            x += dash + gap;
        }
    };
    horizontal(rect.y);
    horizontal(rect.y + rect.height - 1.0);
    let mut y = rect.y;
    while y < rect.y + rect.height {
        let height = dash.min(rect.y + rect.height - y);
        for x in [rect.x, rect.x + rect.width - 1.0] {
            quads.push(Quad::from_rect(
                RectPx {
                    x,
                    y,
                    width: 1.0,
                    height,
                },
                color,
            ));
        }
        y += dash + gap;
    }
}

pub(super) fn draw_header_chip(
    label: &str,
    x: f32,
    y: f32,
    quads: &mut ControlPainter<'_>,
    text: &mut Vec<TextRun>,
) -> RectPx {
    let rect = RectPx {
        x,
        y,
        width: measured_text_run_width_px(
            label,
            design_tokens::typography::MICRO_SIZE,
            TextFace::Mono,
        ) + 14.0,
        height: 20.0,
    };
    push_rounded_rect_with_border(
        quads,
        rect,
        design_tokens::chrome::SURFACE_01,
        design_tokens::chrome::BORDER_STRONG,
        1.0,
        design_tokens::radius::SM,
    );
    draw_text(
        label,
        rect.x + 7.0,
        rect.y + 5.0,
        design_tokens::typography::MICRO_SIZE,
        TEXT_MUTED,
        TextFace::Mono,
        text,
    );
    rect
}

pub(super) fn draw_search_icon(x: f32, y: f32, quads: &mut ControlPainter<'_>) {
    quads.ellipse_fill(
        RectPx {
            x,
            y,
            width: 10.0,
            height: 10.0,
        },
        TEXT_MUTED,
        16,
    );
    quads.ellipse_fill(
        RectPx {
            x: x + 2.0,
            y: y + 2.0,
            width: 6.0,
            height: 6.0,
        },
        design_tokens::chrome::SURFACE_02,
        16,
    );
    quads.push(Quad {
        points: [
            (x + 8.0, y + 8.0),
            (x + 9.2, y + 7.0),
            (x + 13.0, y + 11.0),
            (x + 11.8, y + 12.0),
        ],
        color: TEXT_MUTED,
    });
}

pub(super) fn draw_choice_control(
    label: &str,
    rect: RectPx,
    focused: bool,
    available: bool,
    quads: &mut ControlPainter<'_>,
    text: &mut Vec<TextRun>,
) {
    push_rounded_rect_with_border(
        quads,
        rect,
        design_tokens::chrome::SURFACE_02,
        if focused {
            design_tokens::chrome::STATUS_INFO
        } else {
            design_tokens::chrome::BORDER_STRONG
        },
        if focused { 2.0 } else { 1.0 },
        5.0,
    );
    if !available {
        push_dashed_rect_border(quads, rect, design_tokens::chrome::BORDER_STRONG);
    }
    let visible_label = if available {
        label.to_owned()
    } else {
        format!("{label} · unavailable")
    };
    draw_text_clipped(
        &visible_label,
        rect.x + 12.0,
        rect.y + 8.0,
        design_tokens::typography::CAPTION_SIZE,
        if available { TEXT_PRIMARY } else { TEXT_MUTED },
        TextFace::Ui,
        RectPx {
            width: (rect.width - if available { 22.0 } else { 0.0 }).max(0.0),
            ..rect
        },
        text,
    );
    text.last_mut().expect("choice label").layout_size = Some((rect.width, rect.height));
    if available {
        draw_text(
            "v",
            rect.x + rect.width - 17.0,
            rect.y + 8.0,
            design_tokens::typography::MICRO_SIZE,
            TEXT_MUTED,
            TextFace::Ui,
            text,
        );
    }
}

pub(super) fn draw_boolean_control(
    value: bool,
    label: &str,
    rect: RectPx,
    focused: bool,
    available: bool,
    quads: &mut ControlPainter<'_>,
    text: &mut Vec<TextRun>,
) {
    let switch = RectPx {
        x: rect.x,
        y: rect.y + 4.0,
        width: 38.0,
        height: 20.0,
    };
    push_rounded_rect_with_border(
        quads,
        switch,
        if value {
            TEXT_ACCENT
        } else {
            design_tokens::chrome::SURFACE_03
        },
        if value {
            TEXT_ACCENT
        } else {
            design_tokens::chrome::BORDER_STRONG
        },
        1.0,
        10.0,
    );
    if !available {
        push_dashed_rect_border(quads, switch, design_tokens::chrome::BORDER_STRONG);
    }
    quads.ellipse_fill(
        RectPx {
            x: switch.x + if value { 21.0 } else { 3.0 },
            y: switch.y + 3.0,
            width: 14.0,
            height: 14.0,
        },
        if value {
            design_tokens::chrome::BG_BASE
        } else {
            TEXT_MUTED
        },
        16,
    );
    let visible_label = if available {
        label.to_owned()
    } else {
        format!("{label} · unavailable")
    };
    draw_text(
        &visible_label,
        switch.x + 46.0,
        rect.y + 8.0,
        design_tokens::typography::MICRO_SIZE,
        if available {
            TEXT_SECONDARY
        } else {
            TEXT_MUTED
        },
        TextFace::Mono,
        text,
    );
    if focused {
        push_rect_border(quads, rect, design_tokens::chrome::STATUS_INFO, 2.0);
    }
}

pub(super) fn button(
    label: &str,
    rect: RectPx,
    focused: bool,
    available: bool,
    quads: &mut ControlPainter<'_>,
    text: &mut Vec<TextRun>,
) {
    let border = if focused {
        design_tokens::chrome::STATUS_INFO
    } else {
        design_tokens::chrome::BORDER_STRONG
    };
    if available {
        push_rounded_rect_with_border(
            quads,
            rect,
            design_tokens::chrome::SURFACE_02,
            border,
            if focused { 2.0 } else { 1.0 },
            5.0,
        );
    } else {
        quads.push(Quad::from_rect(rect, design_tokens::chrome::SURFACE_02));
        push_dashed_rect_border(quads, rect, border);
    }
    draw_text_clipped(
        label,
        rect.x + 9.0,
        rect.y + (rect.height - design_tokens::typography::CAPTION_SIZE) * 0.5 - 1.0,
        design_tokens::typography::CAPTION_SIZE,
        if available { TEXT_PRIMARY } else { TEXT_MUTED },
        TextFace::Ui,
        rect,
        text,
    );
    text.last_mut().expect("button label").layout_size = Some((rect.width, rect.height));
}

pub(super) fn draw_preference_control(
    control: &datum_gui_protocol::GlobalPreferenceControlUi,
    right: f32,
    y: f32,
    focused: bool,
    available: bool,
    quads: &mut ControlPainter<'_>,
    text: &mut Vec<TextRun>,
) -> RectPx {
    use datum_gui_protocol::GlobalPreferenceControlUi;

    match control {
        GlobalPreferenceControlUi::Boolean {
            value,
            off_label,
            on_label,
        } => {
            let width = if available { 78.0 } else { 138.0 };
            let rect = RectPx {
                x: right - width,
                y,
                width,
                height: 28.0,
            };
            draw_boolean_control(
                *value,
                if *value { on_label } else { off_label },
                rect,
                focused,
                available,
                quads,
                text,
            );
            rect
        }
        GlobalPreferenceControlUi::SingleChoice { value, choices } => {
            let label = choices
                .iter()
                .find(|(candidate, _)| candidate == value)
                .map(|(_, label)| label.as_str())
                .unwrap_or(value);
            let unavailable = format!("{label} · unavailable");
            let width = (measured_text_run_width_px(
                if available { label } else { &unavailable },
                design_tokens::typography::CAPTION_SIZE,
                TextFace::Ui,
            ) + 34.0)
                .max(58.0);
            let rect = RectPx {
                x: right - width,
                y,
                width,
                height: 30.0,
            };
            draw_choice_control(label, rect, focused, available, quads, text);
            rect
        }
        GlobalPreferenceControlUi::Integer { value, suffix, .. } => {
            let rect = control_rect(right, y, 112.0);
            draw_choice_control(
                &format!("{value}{suffix}"),
                rect,
                focused,
                available,
                quads,
                text,
            );
            rect
        }
        GlobalPreferenceControlUi::Identity { value, placeholder } => {
            let rect = control_rect(right, y, 178.0);
            draw_choice_control(
                value.as_deref().unwrap_or(placeholder),
                rect,
                focused,
                available,
                quads,
                text,
            );
            rect
        }
        GlobalPreferenceControlUi::Structured {
            value_summary,
            action_label,
        } => {
            let rect = control_rect(right, y, 178.0);
            button(
                &format!("{value_summary} · {action_label}"),
                rect,
                focused,
                available,
                quads,
                text,
            );
            rect
        }
    }
}

fn control_rect(right: f32, y: f32, width: f32) -> RectPx {
    RectPx {
        x: right - width,
        y,
        width,
        height: 30.0,
    }
}

#[cfg(all(test, feature = "visual"))]
pub(super) fn push_rounded_rect_fill(
    quads: &mut Vec<Quad>,
    rect: RectPx,
    color: [f32; 3],
    radius: f32,
) {
    let mut cache = ControlMeshCache::default();
    ControlPainter::new(quads, &mut cache, 1.0).rounded_fill(rect, color, radius, 0.0);
}

#[cfg(test)]
mod label_bounds_tests {
    use super::*;

    #[test]
    fn long_control_labels_preserve_choice_affordance_and_button_bounds() {
        let mut cache = ControlMeshCache::default();
        let mut quads = Vec::new();
        let mut text = Vec::new();
        let rect = RectPx {
            x: 25.5,
            y: 10.25,
            width: 100.0,
            height: 30.0,
        };
        for available in [true, false] {
            text.clear();
            draw_choice_control(
                &"long value ".repeat(30),
                rect,
                true,
                available,
                &mut ControlPainter::new(&mut quads, &mut cache, 1.5),
                &mut text,
            );
            let bounds = text[0].clip_bounds.expect("choice value is bounded");
            assert_eq!(bounds.x, rect.x);
            assert_eq!(bounds.width, if available { 78.0 } else { 100.0 });
            assert_eq!(
                text.iter().filter(|run| run.text == "v").count(),
                usize::from(available)
            );
            text.clear();
            button(
                &"long action ".repeat(30),
                rect,
                true,
                available,
                &mut ControlPainter::new(&mut quads, &mut cache, 1.5),
                &mut text,
            );
            assert_eq!(text[0].clip_bounds, Some(rect));
        }
    }
}
