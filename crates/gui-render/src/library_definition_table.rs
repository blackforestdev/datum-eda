//! Shared named definition table rows, measured within the Library pane.
use super::*;

pub(super) fn row(p: &mut Painter<'_>, labels: [&str; 4], control: Option<C>, active: bool) {
    let width = (p.rect.width - 16.).max(1.);
    let cell_width = width / 4.;
    let height = labels
        .iter()
        .map(|label| {
            crate::text_metrics::measured_text_run_height_px(
                label,
                (cell_width - 10.).max(1.),
                12.,
                TextFace::Ui,
            )
            .unwrap_or(16.)
                + 12.
        })
        .fold(28., f32::max);
    let rect = RectPx {
        x: p.rect.x + 8.,
        y: p.y,
        width,
        height,
    };
    if control.is_some() {
        p.quads
            .push(Quad::from_rect(rect, design_tokens::chrome::SURFACE_02));
    }
    if active
        || control
            .as_ref()
            .is_some_and(|c| p.state.library.focus.as_ref() == Some(c))
    {
        push_rect_border(p.quads, rect, TEXT_ACCENT, 1.);
    }
    for (column, label) in labels.into_iter().enumerate() {
        let cell = RectPx {
            x: rect.x + column as f32 * cell_width,
            width: cell_width,
            ..rect
        };
        draw_text_clipped(
            label,
            cell.x + 5.,
            cell.y + 6.,
            12.,
            TEXT_PRIMARY,
            if control.is_none() {
                TextFace::UiStrong
            } else {
                TextFace::Ui
            },
            cell,
            p.text,
        );
        p.text.last_mut().unwrap().layout_size = Some(((cell_width - 10.).max(1.), height - 12.));
    }
    if let Some(control) = control {
        p.hits.push(HitRegion {
            target: HitTarget::Library(control),
            rect,
        });
    }
    p.y += height + 4.;
}
