//! Uniform upload identity and live-pane ownership through real render paths.
use super::*;
use crate::Vertex;

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
        )
        .unwrap();
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
        )
        .unwrap();
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
    )
    .unwrap();
    assert!(changed == capture(&mut fresh, &resized));
    assert!(changed == capture(&mut renderer, &resized));
    assert_eq!(renderer.renderer.uniform_buffer.last_upload_bytes, 0);
}

#[test]
#[ignore = "requires local GPU; shared host-buffer admission and isolation"]
fn screen_streams_and_uniforms_share_one_host_limit() {
    let mut renderer = hardware_renderer(960, 720);
    let mut independent = hardware_renderer(960, 720);
    let budget = renderer.renderer.screen_budget.clone();
    assert!(!std::sync::Arc::ptr_eq(
        &budget,
        &independent.renderer.screen_budget
    ));
    let baseline = budget.used();
    assert_eq!(baseline, 16 + 2 * 64);
    let filler = budget.reserve(16 * 1024 * 1024 - baseline - 60).unwrap();
    let vertices = [Vertex {
        pos: [0.0; 2],
        color: [1.0; 3],
    }; 4];
    let r = &mut renderer.renderer;
    for stream in [
        &mut r.panel_gpu,
        &mut r.viewport_underlay_gpu,
        &mut r.viewport_overlay_gpu,
        &mut r.board_interaction_gpu,
        &mut r.console_gpu.vertices,
        &mut r.menu_overlay_gpu,
        &mut r.schematic_underlay_gpu,
        &mut r.schematic_overlay_gpu,
        &mut r.surface_grid_gpu,
    ] {
        stream
            .sync(
                &renderer.device,
                &renderer.queue,
                "host-boundary",
                &vertices[..3],
            )
            .unwrap();
        assert_eq!(budget.used(), 16 * 1024 * 1024);
        let held = stream.submission_ref().unwrap();
        assert!(
            stream
                .sync(&renderer.device, &renderer.queue, "oversize", &vertices)
                .is_err()
        );
        assert_eq!(stream.buffer().unwrap().size(), 60);
        stream
            .sync::<Vertex>(&renderer.device, &renderer.queue, "empty", &[])
            .unwrap();
        assert_eq!(budget.used(), 16 * 1024 * 1024);
        drop(held);
        assert_eq!(budget.used(), 16 * 1024 * 1024 - 60);
    }
    assert!(
        crate::gpu_data::uniform_buffer::UniformBuffer::new(
            &renderer.device,
            "pane-refusal",
            [0_u32; 16],
            &budget,
        )
        .is_err()
    );
    // Another native renderer still has its own allowance and renders normally.
    let state = crate::gpu_surface_pass::board_fixture_state();
    let retained = RetainedScene::from_workspace(&state, 960, 720);
    let prepared = PreparedScene::from_workspace_for_surface(
        &state,
        960,
        720,
        1.0,
        CameraState::fit_to_bounds(&state.scene.bounds),
        &retained,
    );
    let reference = capture_retained(&mut independent, &prepared, &retained);
    // A device replacement for this SAME host cannot obtain a fresh allowance.
    assert!(
        renderer
            .renderer
            .recreate_for_device(
                &renderer.device,
                &renderer.queue,
                OUTPUT_FORMAT,
                DEFAULT_MSAA_SAMPLES,
            )
            .is_err()
    );
    assert_eq!(
        budget.used(),
        16 * 1024 * 1024 - 60,
        "failed initialization rolls back its uniform"
    );
    drop(filler);
    assert_eq!(budget.used(), baseline);
    let replacement = renderer
        .renderer
        .recreate_for_device(
            &renderer.device,
            &renderer.queue,
            OUTPUT_FORMAT,
            DEFAULT_MSAA_SAMPLES,
        )
        .unwrap();
    assert!(std::sync::Arc::ptr_eq(&budget, &replacement.screen_budget));
    assert_eq!(budget.used(), 2 * baseline);
    renderer.renderer = replacement;
    assert_eq!(budget.used(), baseline);
    assert!(reference == capture_retained(&mut renderer, &prepared, &retained));
    drop(renderer);
    assert_eq!(budget.used(), 0);
}

#[test]
#[ignore = "requires local GPU; staging refusal must preserve producer updates"]
fn frame_staging_refusal_preserves_uniform_and_screen_plans_until_retry() {
    let mut renderer = hardware_renderer(960, 720);
    let budget = renderer.renderer.atlas.staging_budget.clone();
    renderer.renderer.uniform_buffer.sync(
        &renderer.queue,
        crate::gpu_data::ScreenUniform {
            resolution: [960.0, 720.0],
            _pad: [0.0, 0.0],
        },
    );
    renderer
        .renderer
        .panel_gpu
        .sync(
            &renderer.device,
            &renderer.queue,
            "staged-panel",
            &[Vertex {
                pos: [1.0, 2.0],
                color: [0.1, 0.2, 0.3],
            }; 6],
        )
        .unwrap();
    let expected = {
        let mut pending = Vec::new();
        renderer
            .renderer
            .uniform_buffer
            .append_uploads(&mut pending);
        renderer.renderer.panel_gpu.append_uploads(&mut pending);
        pending
            .iter()
            .map(|upload| upload.bytes.len())
            .sum::<usize>()
    };
    assert!(expected > 0);
    let filler = budget.reserve(16 * 1024 * 1024).unwrap();
    assert!(
        renderer
            .renderer
            .flush_frame_uploads(&renderer.device, &renderer.queue)
            .is_err()
    );
    {
        let mut pending = Vec::new();
        renderer
            .renderer
            .uniform_buffer
            .append_uploads(&mut pending);
        renderer.renderer.panel_gpu.append_uploads(&mut pending);
        assert_eq!(
            pending
                .iter()
                .map(|upload| upload.bytes.len())
                .sum::<usize>(),
            expected
        );
    }
    drop(filler);
    let mut batch = renderer
        .renderer
        .flush_frame_uploads(&renderer.device, &renderer.queue)
        .unwrap()
        .unwrap();
    assert_eq!(budget.used(), expected as u64);
    let mut pending = Vec::new();
    renderer
        .renderer
        .uniform_buffer
        .append_uploads(&mut pending);
    renderer.renderer.panel_gpu.append_uploads(&mut pending);
    assert!(pending.is_empty());
    renderer.queue.submit([batch.command()]);
    batch.hold(&renderer.queue);
    renderer
        .device
        .poll(wgpu::PollType::wait_indefinitely())
        .unwrap();
    assert_eq!(budget.used(), 0);
    assert!(
        renderer
            .renderer
            .flush_frame_uploads(&renderer.device, &renderer.queue)
            .unwrap()
            .is_none()
    );
}
