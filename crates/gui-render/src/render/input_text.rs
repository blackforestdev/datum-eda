//! Shared bounded text/caret painting; field profiles retain their own spacing.
use super::*;

pub(crate) struct InputTextLayout {
    pub bounds: RectPx,
    pub left: f32,
    pub right: f32,
    pub placeholder_gap: f32,
    pub caret_top: f32,
    pub caret_height: f32,
    pub caret_trailing: f32,
}

pub(crate) fn paint_input_text(
    value: &str,
    shown: &str,
    focused: bool,
    layout: InputTextLayout,
    quads: &mut ControlPainter<'_>,
    text: &mut Vec<TextRun>,
) -> anyhow::Result<()> {
    let bounds = layout.bounds;
    if bounds.width <= 0.0 || bounds.height <= 0.0 {
        return Ok(());
    }
    let input_x = bounds.x + layout.left;
    let empty = value.is_empty();
    let line_height = design_tokens::typography::BODY_SIZE * 1.22;
    let text_bounds = RectPx {
        x: input_x.min(bounds.x + bounds.width),
        width: (bounds.width - layout.left - layout.right).max(0.0),
        y: bounds.y + 10.0,
        height: line_height.min((bounds.height - 10.0).max(0.0)),
    };
    draw_text_clipped(
        shown,
        input_x
            + if empty && focused {
                layout.placeholder_gap
            } else {
                0.0
            },
        bounds.y + 10.0,
        design_tokens::typography::BODY_SIZE,
        if empty { TEXT_MUTED } else { TEXT_PRIMARY },
        TextFace::Ui,
        text_bounds,
        text,
    );
    text.last_mut().expect("input label").layout_size = Some((text_bounds.width, line_height));
    if focused {
        let caret_x = if empty {
            input_x
        } else {
            (input_x
                + measured_text_run_width_px(
                    value,
                    design_tokens::typography::BODY_SIZE,
                    TextFace::Ui,
                )?
                + layout.caret_trailing)
                .min(bounds.x + bounds.width - layout.right)
        };
        let caret = RectPx {
            x: caret_x,
            y: bounds.y + layout.caret_top,
            width: 1.5,
            height: layout.caret_height,
        };
        if let Some(visible) = caret.intersect(bounds) {
            quads.push(Quad::from_rect(visible, TEXT_PRIMARY));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn narrow_input_bounds_text_and_caret_without_changing_display_text() {
        for width in [1.0, 12.0, 200.0] {
            for value in ["", "a long input value that exceeds a narrow field"] {
                let bounds = RectPx {
                    x: 10.25,
                    y: 4.5,
                    width,
                    height: 34.0,
                };
                let mut cache = ControlMeshCache::default();
                let mut quads = Vec::new();
                let mut text = Vec::new();
                paint_input_text(
                    value,
                    value,
                    true,
                    InputTextLayout {
                        bounds,
                        left: 10.0,
                        right: 10.0,
                        placeholder_gap: 0.0,
                        caret_top: 7.0,
                        caret_height: 20.0,
                        caret_trailing: 0.0,
                    },
                    &mut ControlPainter::new(&mut quads, &mut cache, 1.0),
                    &mut text,
                )
                .unwrap();
                assert_eq!(text[0].text, value);
                assert!(text[0].clip_bounds.unwrap().width <= bounds.width);
                assert_eq!(
                    text[0].layout_size,
                    Some((
                        (width - 20.0).max(0.0),
                        design_tokens::typography::BODY_SIZE * 1.22
                    ))
                );
                assert!(
                    quads
                        .iter()
                        .all(|q| q.points.iter().all(|&(x, y)| bounds.contains(x, y)))
                );
            }
        }
    }
}
