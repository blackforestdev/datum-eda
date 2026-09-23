use super::*;

pub(super) fn render_active_inspector(
    state: &ReviewWorkspaceState,
    rect: RectPx,
    panel_quads: &mut Vec<Quad>,
    text_runs: &mut Vec<TextRun>,
    hit_regions: &mut Vec<HitRegion>,
) {
    let starts = (panel_quads.len(), text_runs.len(), hit_regions.len());
    if !revision_workspace::render_evidence_inspector(state, rect, panel_quads, text_runs) {
        render_inspector_panel(state, rect, panel_quads, text_runs, hit_regions);
    }
    crate::hit_clipping::clip_content(
        panel_quads,
        text_runs,
        hit_regions,
        starts.0,
        starts.1,
        starts.2,
        rect,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use datum_gui_protocol::{PaneContent, RevisionPane, RevisionSurface, SplitOrientation};

    #[test]
    fn active_inspector_profiles_keep_content_inside_the_panel() {
        let mut state = datum_gui_protocol::load_fixture_workspace_state();
        let rect = RectPx {
            x: 400.0,
            y: 80.0,
            width: 90.0,
            height: 120.0,
        };
        for evidence in [false, true] {
            if evidence {
                state.ui.layout.open_beside_root(
                    PaneContent::Revision(RevisionPane::Surface(RevisionSurface::Evidence)),
                    SplitOrientation::Vertical,
                    0.5,
                    true,
                );
            }
            let mut quads = Vec::new();
            let mut text = Vec::new();
            let mut hits = Vec::new();
            render_active_inspector(&state, rect, &mut quads, &mut text, &mut hits);
            assert!(!text.is_empty());
            for run in text {
                let clip = run
                    .clip_bounds
                    .expect("every Inspector label inherits its panel clip");
                assert_eq!(clip.intersect(rect), Some(clip));
            }
            for quad in quads {
                assert!(quad.points.iter().all(|&(x, y)| rect.contains(x, y)));
            }
            for hit in hits {
                assert_eq!(hit.rect.intersect(rect), Some(hit.rect));
            }
        }
    }
    #[test]
    #[cfg(feature = "visual")]
    #[ignore = "requires local GPU; inspector width adoption and rendered review"]
    fn selected_inspector_uses_measured_width_and_stays_warm() {
        let mut state = datum_gui_protocol::load_fixture_workspace_state();
        state.selection = SelectionTarget::ReviewAction("action-1".into());
        let rect = RectPx {
            x: 400.0,
            y: 80.0,
            width: 300.0,
            height: 400.0,
        };
        let mut quads = Vec::new();
        let mut text = Vec::new();
        let mut hits = Vec::new();
        render_active_inspector(&state, rect, &mut quads, &mut text, &mut hits);
        let label = text.iter().find(|run| run.text == "SELECTED").unwrap();
        let actual_right =
            label.x + measured_text_run_width_px(&label.text, label.size, label.face);
        assert!((actual_right - (rect.x + rect.width - 12.0 - 7.0)).abs() < 0.001);
        let mut renderer =
            crate::visual::visual_capture::OffscreenRenderer::new(1280, 800).unwrap();
        let cold = renderer.render_workspace(&state, None).unwrap();
        assert_eq!(cold, renderer.render_workspace(&state, None).unwrap());
        if let Ok(path) = std::env::var("PM045_INSPECTOR_CAPTURE") {
            cold.save(path).unwrap();
        }
    }
}
