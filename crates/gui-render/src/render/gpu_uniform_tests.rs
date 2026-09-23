//! Uniform upload identity and live-pane ownership through real render paths.
use super::*;

#[test]
#[ignore = "requires local GPU; shared uniform ownership proof"]
fn uniform_uploads_stay_warm_and_retire_closed_surface_slots() {
    let state = crate::gpu_surface_pass::board_fixture_state();
    let retained = RetainedScene::from_workspace(&state, 960, 720);
    let mut camera = CameraState::fit_to_bounds(&state.scene.bounds);
    let initial_zoom = camera.zoom;
    let mut renderer = hardware_renderer(960, 720);
    let mut fresh = hardware_renderer(960, 720);
    for (step, (width, height, zoom)) in [(960, 720, 1.0), (960, 720, 1.5), (1050, 810, 1.5)]
        .into_iter()
        .enumerate()
    {
        camera.zoom = initial_zoom * zoom;
        renderer.width = width;
        renderer.height = height;
        fresh.width = width;
        fresh.height = height;
        let prepared = PreparedScene::from_workspace_for_surface(
            &state, width, height, 1.0, camera, &retained,
        );
        let changed = capture_retained(&mut renderer, &prepared, &retained);
        assert_eq!(
            renderer.renderer.uniform_buffer.last_upload_bytes,
            if step == 1 { 0 } else { 8 }
        );
        assert!(
            renderer
                .renderer
                .surface_scene_uniforms
                .iter()
                .any(|binding| if step == 0 {
                    binding.buffer.last_upload_bytes == 64
                } else {
                    binding.buffer.last_upload_bytes > 0 && binding.buffer.last_upload_bytes < 64
                })
        );
        assert!(changed == capture_retained(&mut renderer, &prepared, &retained));
        assert_eq!(renderer.renderer.uniform_buffer.last_upload_bytes, 0);
        assert!(
            renderer
                .renderer
                .surface_scene_uniforms
                .iter()
                .all(|binding| binding.buffer.last_upload_bytes == 0)
        );
        fresh.renderer = Renderer::new(
            &fresh.device,
            &fresh.queue,
            OUTPUT_FORMAT,
            DEFAULT_MSAA_SAMPLES,
        );
        assert!(changed == capture_retained(&mut fresh, &prepared, &retained));

        let mut moved_camera = camera;
        moved_camera.zoom *= 1.1;
        let moved = PreparedScene::from_workspace_for_surface(
            &state,
            width,
            height,
            1.0,
            moved_camera,
            &retained,
        );
        let moved_pixels = capture_retained(&mut renderer, &moved, &retained);
        assert!(
            renderer
                .renderer
                .surface_scene_uniforms
                .iter()
                .any(|binding| binding.buffer.last_upload_bytes > 0
                    && binding.buffer.last_upload_bytes < 64)
        );
        fresh.renderer = Renderer::new(
            &fresh.device,
            &fresh.queue,
            OUTPUT_FORMAT,
            DEFAULT_MSAA_SAMPLES,
        );
        assert!(moved_pixels == capture_retained(&mut fresh, &moved, &retained));

        let mut closed = prepared.clone();
        closed.surface_passes.clear();
        capture_retained(&mut renderer, &closed, &retained);
        assert!(renderer.renderer.surface_scene_uniforms.is_empty());
        assert!(renderer.renderer.surface_world_bundles.is_empty());
        assert!(changed == capture_retained(&mut renderer, &prepared, &retained));
        assert!(
            renderer
                .renderer
                .surface_scene_uniforms
                .iter()
                .all(|binding| binding.buffer.last_upload_bytes == 64)
        );
    }
    let dialog_state = crate::global_preferences_dialog_tests::state_with_preferences_open();
    let dialog = PreparedScene::from_native_preferences(
        &dialog_state.ui.global_preferences,
        renderer.width,
        renderer.height,
        1.0,
    );
    let first = capture(&mut renderer, &dialog);
    assert_eq!(renderer.renderer.uniform_buffer.last_upload_bytes, 0);
    assert!(renderer.renderer.surface_scene_uniforms.is_empty());
    assert!(renderer.renderer.surface_world_bundles.is_empty());
    assert!(first == capture(&mut renderer, &dialog));
    assert_eq!(renderer.renderer.uniform_buffer.last_upload_bytes, 0);
    renderer.width += 80;
    renderer.height += 20;
    let resized = PreparedScene::from_native_preferences(
        &dialog_state.ui.global_preferences,
        renderer.width,
        renderer.height,
        1.0,
    );
    let changed = capture(&mut renderer, &resized);
    assert_eq!(renderer.renderer.uniform_buffer.last_upload_bytes, 8);
    fresh.width = renderer.width;
    fresh.height = renderer.height;
    fresh.renderer = Renderer::new(
        &fresh.device,
        &fresh.queue,
        OUTPUT_FORMAT,
        DEFAULT_MSAA_SAMPLES,
    );
    assert!(changed == capture(&mut fresh, &resized));
    assert!(changed == capture(&mut renderer, &resized));
    assert_eq!(renderer.renderer.uniform_buffer.last_upload_bytes, 0);
}
