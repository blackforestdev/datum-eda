//! Compare shared capture adoption against the preserved pre-session composition recipe.
use super::*;
use crate::{PreparedScene, RetainedScene};

fn reference(
    capture: &mut OffscreenRenderer,
    state: &ReviewWorkspaceState,
    snapshot: Option<&datum_terminal_core::RenderSnapshot>,
    scale: f32,
) -> RgbaImage {
    let target = crate::capture_resource::CaptureTarget::new(
        &capture.device,
        capture.extent(),
        OUTPUT_FORMAT,
    )
    .unwrap();
    let view = target.create_view(&wgpu::TextureViewDescriptor::default());
    let retained =
        RetainedScene::try_from_workspace_for_surface(state, capture.width, capture.height, scale)
            .unwrap();
    let schematic = RetainedScene::try_from_workspace_schematic_for_surface(
        state,
        capture.width,
        capture.height,
        scale,
    )
    .unwrap();
    let camera = CameraState::fit_to_bounds(&state.scene.bounds);
    let prepared = if state.ui.global_preferences.open {
        PreparedScene::from_workspace_with_terminal_renderer(
            state,
            capture.width,
            capture.height,
            scale,
            camera,
            &retained,
            &[],
            None,
            true,
        )
        .unwrap()
    } else {
        PreparedScene::from_workspace_with_terminal_snapshot(
            state,
            capture.width,
            capture.height,
            scale,
            camera,
            &retained,
            snapshot,
        )
        .unwrap()
    };
    capture
        .renderer
        .render(
            &capture.device,
            &capture.queue,
            &view,
            &prepared,
            &retained,
            schematic.as_ref(),
            capture.width,
            capture.height,
        )
        .unwrap();
    target.hold_submission(&capture.queue);
    capture.read_texture(&target).unwrap()
}

#[test]
#[ignore = "requires GPU readback; exact existing 4x capture compatibility, not native 8x qualification"]
fn shared_offscreen_snapshots_match_legacy_pixels_and_replace_terminal_rows() {
    let mut capture = OffscreenRenderer::new(640, 480).unwrap();
    let mut state = datum_gui_protocol::load_fixture_workspace_state();
    for scale in [1.0, 1.5] {
        for menu in [None, Some("File".to_owned())] {
            state.ui.active_menu = menu;
            let expected = reference(&mut capture, &state, None, scale);
            capture
                .warm_workspace_for_surface_scale(&state, None, scale)
                .unwrap();
            let actual = capture
                .render_workspace_for_surface_scale(&state, None, scale)
                .unwrap();
            assert!(
                actual.as_raw() == expected.as_raw(),
                "scale={scale}, menu={:?}",
                state.ui.active_menu
            );
            assert!(
                capture
                    .renderer
                    .render_session_mut()
                    .take_published_frame()
                    .is_none()
            );
            assert!(capture.renderer.render_session().has_pending_frame());
        }
    }
    state.ui.active_menu = None;
    state.ui.global_preferences.open = true;
    let expected = reference(&mut capture, &state, None, 1.0);
    let actual = capture.render_workspace(&state, None).unwrap();
    assert!(
        actual.as_raw() == expected.as_raw(),
        "preferences capture differs"
    );
    state.ui.global_preferences.open = false;
    state.ui.active_dock_tab = Some(datum_gui_protocol::DockTab::Terminal);
    state.ui.dock_height_px = 220;
    let mut previous = None;
    for bytes in [b"AAAA".as_slice(), b"BBBB".as_slice()] {
        if bytes == b"BBBB" {
            use datum_gui_protocol::{
                TerminalSplitDirection, TerminalSplitNode, TerminalTabLayout,
            };
            state.ui.terminal.active_session_id = Some("terminal".into());
            state.ui.terminal.active_tab_id = Some("capture".into());
            state.ui.terminal.tab_layouts = vec![TerminalTabLayout {
                tab_id: "capture".into(),
                focused_session_id: "terminal".into(),
                root: TerminalSplitNode::Split {
                    direction: TerminalSplitDirection::SideBySide,
                    ratio_millis: 500,
                    first: Box::new(TerminalSplitNode::session("terminal")),
                    second: Box::new(TerminalSplitNode::session("other")),
                },
            }];
        }
        let snapshot = crate::terminal_core_render::test_snapshot(bytes);
        let expected = reference(&mut capture, &state, Some(&snapshot), 1.0);
        let actual = capture
            .render_workspace_with_terminal_snapshot(&state, &snapshot, 1.0)
            .unwrap();
        assert!(
            actual.as_raw() == expected.as_raw(),
            "terminal capture differs"
        );
        if let Some(old) = previous.replace(actual) {
            assert_ne!(
                old.as_raw(),
                previous.as_ref().unwrap().as_raw(),
                "equal-length terminal replacement must not reuse old row pixels"
            );
        }
    }
}

#[path = "prefix_graph_tests.rs"]
mod prefix_graph_tests;
