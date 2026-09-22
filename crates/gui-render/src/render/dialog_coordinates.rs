//! Native dialog layout and scroll state use logical pixels; paint/hits use physical pixels.
use super::*;

/// Transform only this dialog's appended output. Font sizes are scaled once by
/// the shared frame finalizer, alongside the other text in its frame.
pub(crate) fn scale_output(
    quads: &mut [Quad],
    text: &mut [TextRun],
    hits: &mut [HitRegion],
    starts: (usize, usize, usize),
    scale: f32,
) {
    if scale == 1.0 {
        return;
    }
    let rect = |r: RectPx| RectPx {
        x: r.x * scale,
        y: r.y * scale,
        width: r.width * scale,
        height: r.height * scale,
    };
    for quad in &mut quads[starts.0..] {
        for point in &mut quad.points {
            point.0 *= scale;
            point.1 *= scale;
        }
    }
    for run in &mut text[starts.1..] {
        run.x *= scale;
        run.y *= scale;
        run.clip_bounds = run.clip_bounds.map(rect);
        run.layout_size = run.layout_size.map(|(w, h)| (w * scale, h * scale));
    }
    for hit in &mut hits[starts.2..] {
        hit.rect = rect(hit.rect);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dialog_controls_text_and_hits_share_dpi_transform() {
        let mut state = crate::global_preferences_dialog_tests::state_with_preferences_open();
        state.ui.new_project.open = true;
        for new_project in [false, true] {
            let compose = |scale: f32| {
                let layout = ShellLayout::for_surface(
                    (960.0 * scale) as u32,
                    (720.0 * scale) as u32,
                    scale,
                    None,
                );
                let (mut quads, mut text, mut hits) = (Vec::new(), Vec::new(), Vec::new());
                let mut cache = ControlMeshCache::default();
                if new_project {
                    crate::new_project_dialog::render_new_project_dialog(
                        &state.ui.new_project,
                        &layout,
                        true,
                        &mut cache,
                        scale,
                        &mut quads,
                        &mut text,
                        &mut hits,
                    );
                } else {
                    super::super::render_preferences_dialog(
                        &state.ui.global_preferences,
                        &layout,
                        &mut cache,
                        scale,
                        &mut quads,
                        &mut text,
                        &mut hits,
                    );
                }
                (quads, text, hits)
            };
            let baseline = compose(1.0);
            for scale in [1.5, 2.0] {
                let actual = compose(scale);
                assert_eq!(baseline.0.len(), actual.0.len());
                assert_eq!(baseline.1.len(), actual.1.len());
                assert_eq!(baseline.2.len(), actual.2.len());
                for (old, new) in baseline.0.iter().zip(&actual.0) {
                    for (a, b) in old.points.iter().zip(new.points) {
                        assert!((a.0 * scale - b.0).abs() < 0.001);
                        assert!((a.1 * scale - b.1).abs() < 0.001);
                    }
                }
                for (old, new) in baseline.1.iter().zip(&actual.1) {
                    assert_eq!(old.text, new.text);
                    assert!((old.x * scale - new.x).abs() < 0.001);
                    assert!((old.y * scale - new.y).abs() < 0.001);
                    assert_eq!(
                        old.layout_size.map(|(w, h)| (w * scale, h * scale)),
                        new.layout_size
                    );
                }
                for (old, new) in baseline.2.iter().zip(&actual.2) {
                    assert_eq!(old.target, new.target);
                    assert!((old.rect.x * scale - new.rect.x).abs() < 0.001);
                    assert!((old.rect.y * scale - new.rect.y).abs() < 0.001);
                    assert!((old.rect.width * scale - new.rect.width).abs() < 0.001);
                    assert!((old.rect.height * scale - new.rect.height).abs() < 0.001);
                }
            }
        }
    }
}
