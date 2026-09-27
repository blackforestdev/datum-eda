//! Text placement payload and its originating production surface.
use super::{RectPx, TextFace, TextRunSpan};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum TextOrigin {
    #[default]
    Host,
    Viewport(datum_gui_protocol::PaneId),
    /// Index in this prepared frame's immutable terminal session identity list.
    TerminalLeaf(usize),
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct TextRun {
    pub(crate) origin: TextOrigin,
    pub(crate) text: String,
    pub(crate) rich_spans: Vec<TextRunSpan>,
    pub(crate) x: f32,
    pub(crate) y: f32,
    pub(crate) size: f32,
    pub(crate) color: [f32; 3],
    pub(crate) face: TextFace,
    pub(crate) clip_bounds: Option<RectPx>,
    // Stable layout extent; ancestor clipping changes visibility only.
    pub(crate) layout_size: Option<(f32, f32)>,
}

impl TextRun {
    pub(crate) fn annotate(runs: &mut [Self], origin: TextOrigin) {
        for run in runs {
            run.origin = origin;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn production_pane_composition_retains_text_origin_through_scaling() {
        let state = crate::gpu_surface_pass::board_fixture_state();
        for scale in [1.0, 1.5, 2.0] {
            let retained =
                crate::RetainedScene::from_workspace_for_surface(&state, 1280, 800, scale);
            let prepared = crate::PreparedScene::from_workspace_for_surface(
                &state,
                1280,
                800,
                scale,
                crate::CameraState::fit_to_bounds(&state.scene.bounds),
                &retained,
            )
            .unwrap();
            assert!(
                prepared
                    .text_runs
                    .iter()
                    .any(|r| r.origin == TextOrigin::Host)
            );
            let panes = prepared.layout.viewport_panes(&state.ui.layout);
            assert!(panes.panes.len() >= 2);
            for pane in panes.panes {
                assert!(
                    prepared
                        .text_runs
                        .iter()
                        .any(|r| r.origin == TextOrigin::Viewport(pane.id))
                );
            }
        }
    }
}

#[cfg(test)]
#[path = "text_origin_terminal_tests.rs"]
mod terminal_tests;
