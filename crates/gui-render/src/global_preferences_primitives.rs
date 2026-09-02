//! Small screen-space primitives used by the protected Global Preferences grammar.

use super::*;

const ROUNDED_RECT_CORNER_SEGMENTS: usize = 4;

pub(super) fn push_rounded_rect_with_border(
    quads: &mut Vec<Quad>,
    rect: RectPx,
    fill: [f32; 3],
    border: [f32; 3],
    thickness: f32,
    radius: f32,
) {
    push_rounded_rect_fill(quads, rect, border, radius);
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
        push_rounded_rect_fill(quads, inner, fill, (radius - inset).max(0.0));
    }
}

fn push_rounded_rect_fill(quads: &mut Vec<Quad>, rect: RectPx, color: [f32; 3], radius: f32) {
    push_projected_polygon_fill(quads, &rounded_rect_points(rect, radius), color);
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
    for (center_x, center_y, start_angle) in corners {
        for step in 0..=ROUNDED_RECT_CORNER_SEGMENTS {
            let angle = start_angle
                + std::f32::consts::FRAC_PI_2 * step as f32 / ROUNDED_RECT_CORNER_SEGMENTS as f32;
            points.push((
                center_x + radius * angle.cos(),
                center_y + radius * angle.sin(),
            ));
        }
    }
    points
}

pub(super) fn push_dashed_rect_border(quads: &mut Vec<Quad>, rect: RectPx, color: [f32; 3]) {
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
    quads: &mut Vec<Quad>,
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

pub(super) fn draw_search_icon(x: f32, y: f32, quads: &mut Vec<Quad>) {
    push_projected_ellipse(
        quads,
        RectPx {
            x,
            y,
            width: 10.0,
            height: 10.0,
        },
        TEXT_MUTED,
        16,
    );
    push_projected_ellipse(
        quads,
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
    quads: &mut Vec<Quad>,
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
    draw_text(
        &visible_label,
        rect.x + 12.0,
        rect.y + 8.0,
        design_tokens::typography::CAPTION_SIZE,
        if available { TEXT_PRIMARY } else { TEXT_MUTED },
        TextFace::Ui,
        text,
    );
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
    quads: &mut Vec<Quad>,
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
    push_projected_ellipse(
        quads,
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
    quads: &mut Vec<Quad>,
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
    draw_text(
        label,
        rect.x + 9.0,
        rect.y + (rect.height - design_tokens::typography::CAPTION_SIZE) * 0.5 - 1.0,
        design_tokens::typography::CAPTION_SIZE,
        if available { TEXT_PRIMARY } else { TEXT_MUTED },
        TextFace::Ui,
        text,
    );
}
