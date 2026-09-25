//! Chrome is laid out in logical pixels, then geometry and hit rectangles share
//! one conversion to physical pixels. Text size is scaled once by frame assembly.
use crate::{HitRegion, Quad, RectPx, ShellLayout, TextRun, design_tokens};

#[derive(Clone, Copy)]
pub(crate) struct Scale(f32);
impl Scale {
    pub fn for_layout(layout: &ShellLayout) -> Self {
        Self(layout.top_menu_bar.height / (design_tokens::spacing::SP_07 + 1.0))
    }
    pub fn logical_layout(self, layout: ShellLayout) -> ShellLayout {
        layout.scale_by(1.0 / self.0)
    }
    fn rect(self, rect: &mut RectPx) {
        rect.x *= self.0;
        rect.y *= self.0;
        rect.width *= self.0;
        rect.height *= self.0;
    }
    pub fn quads(self, quads: &mut [Quad]) {
        for quad in quads {
            for (x, y) in &mut quad.points {
                *x *= self.0;
                *y *= self.0;
            }
        }
    }
    pub fn text_geometry(self, runs: &mut [TextRun]) {
        for run in runs {
            run.x *= self.0;
            run.y *= self.0;
            if let Some(clip) = &mut run.clip_bounds {
                self.rect(clip);
            }
            if let Some((width, height)) = &mut run.layout_size {
                *width *= self.0;
                *height *= self.0;
            }
        }
    }
    pub fn hits(self, hits: &mut [HitRegion]) {
        for hit in hits {
            self.rect(&mut hit.rect);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::*;

    fn scene(scale: f32, width: u32) -> PreparedScene {
        let mut state = datum_gui_protocol::load_fixture_workspace_state();
        state.ui.active_menu = Some("File".into());
        PreparedScene::from_workspace_with_terminal_renderer(
            &state,
            width,
            (800.0 * scale) as u32,
            scale,
            CameraState::fit_to_bounds(&state.scene.bounds),
            &RetainedScene::empty(),
            &[],
            None,
            false,
        )
        .unwrap()
    }

    #[test]
    fn composed_scaled_menus_keep_glyphs_and_clicks_inside_the_same_titles() {
        let baseline = scene(1.0, 1280);
        for scale in [1.0, 1.25, 1.5, 2.0] {
            let prepared = scene(scale, (1280.0 * scale) as u32);
            for hit in &prepared.hit_regions {
                if !matches!(
                    hit.target,
                    HitTarget::MenuTitle(_) | HitTarget::MenuItem { .. }
                ) {
                    continue;
                }
                let original = baseline
                    .hit_regions
                    .iter()
                    .find(|h| h.target == hit.target)
                    .unwrap();
                for (actual, logical) in [
                    (hit.rect.x, original.rect.x),
                    (hit.rect.y, original.rect.y),
                    (hit.rect.width, original.rect.width),
                    (hit.rect.height, original.rect.height),
                ] {
                    assert!((actual / scale - logical).abs() < 0.01, "{:?}", hit.target);
                }
                let center = (
                    hit.rect.x + hit.rect.width * 0.5,
                    hit.rect.y + hit.rect.height * 0.5,
                );
                assert_eq!(prepared.hit_test(center.0, center.1), Some(&hit.target));
                if let HitTarget::MenuTitle(label) = &hit.target {
                    let run = prepared
                        .text_runs
                        .iter()
                        .find(|r| &r.text == label && r.y < prepared.layout.top_menu_bar.height)
                        .unwrap();
                    let width = measured_text_run_width_px(&run.text, run.size, run.face).unwrap();
                    assert!(run.x >= hit.rect.x);
                    assert!(run.x + width <= hit.rect.x + hit.rect.width + 0.01);
                }
            }
        }
    }

    #[test]
    fn composed_panels_and_panes_share_scaled_paint_and_input_bounds() {
        let baseline = scene(1.0, 1280);
        let workspace = datum_gui_protocol::load_fixture_workspace_state().ui.layout;
        let original_panes = baseline.layout.viewport_panes(&workspace);
        for scale in [1.25, 1.5, 2.0] {
            let prepared = scene(scale, (1280.0 * scale) as u32);
            let assert_rect = |actual: RectPx, logical: RectPx| {
                for (a, b) in [
                    (actual.x, logical.x),
                    (actual.y, logical.y),
                    (actual.width, logical.width),
                    (actual.height, logical.height),
                ] {
                    assert!(
                        (a / scale - b).abs() < 0.02,
                        "scale {scale}: {actual:?} / {logical:?}"
                    );
                }
            };
            let panes = prepared.layout.viewport_panes(&workspace);
            for (actual, original) in panes.panes.iter().zip(&original_panes.panes) {
                assert_rect(actual.rect.header, original.rect.header);
                assert_rect(actual.rect.scene, original.rect.scene);
            }
            let mut checked = 0;
            for original in &baseline.hit_regions {
                if !matches!(
                    original.target,
                    HitTarget::ToggleLayer(_) | HitTarget::LayerScrollRegion
                ) {
                    continue;
                }
                let actual = prepared
                    .hit_regions
                    .iter()
                    .find(|h| h.target == original.target)
                    .unwrap();
                assert_rect(actual.rect, original.rect);
                checked += 1;
            }
            assert!(checked > 0, "must exercise production Layers geometry");
            for original in baseline
                .text_runs
                .iter()
                .filter(|run| baseline.layout.right_sidebar.contains(run.x, run.y))
            {
                let actual = prepared
                    .text_runs
                    .iter()
                    .find(|run| {
                        run.text == original.text
                            && (run.x / scale - original.x).abs() < 0.02
                            && (run.y / scale - original.y).abs() < 0.02
                    })
                    .unwrap_or_else(|| panic!("unscaled panel text: {}", original.text));
                assert!((actual.size / scale - original.size).abs() < 0.02);
            }
        }
    }

    #[test]
    fn composed_scaled_status_segments_do_not_overlap_at_supported_width() {
        for scale in [1.0, 1.25, 1.5, 2.0] {
            let prepared = scene(scale, 1280);
            let bounds = prepared.layout.status_bar;
            let mut runs = prepared
                .text_runs
                .iter()
                .filter(|r| r.y >= bounds.y)
                .collect::<Vec<_>>();
            runs.sort_by(|a, b| a.x.total_cmp(&b.x));
            assert!(runs.iter().any(|r| r.text == "focus"));
            for pair in runs.windows(2) {
                let width =
                    measured_text_run_width_px(&pair[0].text, pair[0].size, pair[0].face).unwrap();
                assert!(
                    pair[0].x + width <= pair[1].x + 0.5,
                    "scale {scale}: {} overlaps {}",
                    pair[0].text,
                    pair[1].text
                );
            }
        }
    }
}
