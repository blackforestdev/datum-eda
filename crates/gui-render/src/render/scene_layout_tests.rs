//! Shell surface scale and prepared-scene layout contracts.
use super::*;

#[test]
fn shell_layout_is_solved_by_taffy_grid_contract() {
    // Assert the derivable, token-driven grid CONTRACT rather than magic
    // pixels that drift: menu bar on top; left sidebar then viewport then
    // right sidebar below it; dock then status bar
    // pinned full-width to the bottom. Values come from design tokens so
    // this cannot silently fall out of sync with the layout again.
    let (w, h, dock) = (1280.0_f32, 800.0_f32, 260.0_f32);
    let layout = ShellLayout::for_window(w as u32, h as u32, Some(dock as u32));

    let menu = design_tokens::spacing::SP_07 + 1.0;
    let status = design_tokens::spacing::SP_06 + design_tokens::spacing::SP_01;
    let content_h = h - menu - dock - status;

    assert_eq!(
        layout.top_menu_bar,
        RectPx {
            x: 0.0,
            y: 0.0,
            width: w,
            height: menu
        }
    );
    assert_eq!(layout.left_sidebar.x, 0.0);
    assert_eq!(layout.left_sidebar.y, menu);
    assert_eq!(layout.left_sidebar.height, content_h);
    assert_eq!(layout.viewport.x, layout.left_sidebar.width);
    assert_eq!(layout.viewport.y, menu);
    assert_eq!(layout.viewport.height, content_h);
    assert_eq!(
        layout.viewport.width,
        w - layout.left_sidebar.width - layout.right_sidebar.width
    );
    assert_eq!(layout.right_sidebar.x, w - layout.right_sidebar.width);
    assert_eq!(layout.right_sidebar.y, menu);
    assert_eq!(layout.right_sidebar.height, content_h);
    assert_eq!(
        layout.bottom_strip,
        RectPx {
            x: 0.0,
            y: menu + content_h,
            width: w,
            height: dock
        }
    );
    assert_eq!(
        layout.status_bar,
        RectPx {
            x: 0.0,
            y: h - status,
            width: w,
            height: status
        }
    );
}

#[test]
fn shell_layout_solves_logical_pixels_then_scales_to_surface_pixels() {
    // Contract: for_surface takes PHYSICAL pixels, solves the layout at
    // logical pixels (physical / scale), then scales every region back up
    // to surface pixels. Assert that relationship (plus token-derived
    // anchors) instead of drifting magic pixels.
    let scale = 1.25_f32;
    let (phys_w, phys_h) = (1600u32, 1000u32);
    let logical = ShellLayout::for_window(
        (phys_w as f32 / scale).round() as u32,
        (phys_h as f32 / scale).round() as u32,
        Some(260),
    );
    let surface = ShellLayout::for_surface(phys_w, phys_h, scale, Some(260));

    assert_eq!(
        surface.top_menu_bar.height,
        logical.top_menu_bar.height * scale
    );
    assert_eq!(
        surface.left_sidebar.width,
        logical.left_sidebar.width * scale
    );
    assert_eq!(
        surface.right_sidebar.width,
        logical.right_sidebar.width * scale
    );
    assert_eq!(surface.status_bar.height, logical.status_bar.height * scale);
    assert_eq!(surface.viewport.x, logical.viewport.x * scale);
    assert_eq!(surface.viewport.width, logical.viewport.width * scale);

    let menu = design_tokens::spacing::SP_07 + 1.0;
    let status = design_tokens::spacing::SP_06 + design_tokens::spacing::SP_01;
    assert_eq!(surface.top_menu_bar.height, menu * scale);
    assert_eq!(surface.status_bar.height, status * scale);
}

#[test]
fn prepared_scene_uses_surface_scale_for_layout_and_text() {
    let state = datum_gui_protocol::load_fixture_workspace_state();
    let retained = RetainedScene::from_workspace_for_surface(&state, 1600, 1000, 1.25);
    let prepared = PreparedScene::from_workspace_for_surface(
        &state,
        1600,
        1000,
        1.25,
        CameraState::fit_to_bounds(&state.scene.bounds),
        &retained,
    )
    .unwrap();

    assert_eq!(
        prepared.layout,
        ShellLayout::for_surface(1600, 1000, 1.25, dock_height_for_state(&state))
    );
    let project_title = prepared
        .text_runs
        .iter()
        .find(|run| run.text == "PROJECT")
        .expect("project title should render");
    assert_eq!(project_title.size, 15.0);
    assert!(
        prepared
            .hit_test(
                prepared.layout.left_sidebar.x + 20.0,
                prepared.layout.left_sidebar.y + 20.0,
            )
            .is_none()
    );
}
