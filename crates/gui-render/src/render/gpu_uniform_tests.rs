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
    let scope = crate::cpu_alloc::Scope::new("frame-upload-count");
    let mut count = crate::text_gpu::upload::UploadCount::default();
    scope.with(|| {
        renderer.renderer.uniform_buffer.append_uploads(&mut count);
        renderer.renderer.panel_gpu.append_uploads(&mut count);
    });
    assert_eq!(count.bytes, expected as u64);
    assert!(count.entries > 0);
    assert_eq!(
        scope.usage().peak_payload_bytes,
        0,
        "producer preflight must not allocate"
    );
    let metadata = crate::text_gpu::staging_vec::StagingVec::<
        crate::text_gpu::upload::BufferUpload<'_>,
    >::capacity_bytes(count.entries)
    .unwrap();
    let filler = budget.reserve(budget.available() - metadata + 1).unwrap();
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
    assert_eq!(
        budget.used(),
        expected as u64
            + crate::text_gpu::upload::retention_metadata_bytes(&[], false).unwrap()
            + renderer.renderer.screen_upload_metadata_bytes()
            + renderer.renderer.screen_upload_snapshot_bytes()
    );
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
    assert_eq!(
        budget.used(),
        renderer.renderer.screen_upload_metadata_bytes()
            + renderer.renderer.screen_upload_snapshot_bytes()
    );
    assert!(
        renderer
            .renderer
            .flush_frame_uploads(&renderer.device, &renderer.queue)
            .unwrap()
            .is_none()
    );
}

#[test]
#[ignore = "requires local GPU and serial resource admission"]
fn control_retention_bypasses_full_cache_and_retires_after_submission() {
    let mut host = hardware_renderer(64, 64);
    let retention = host.renderer.control_gpu_budget.clone();
    let screen = host.renderer.screen_budget.clone();
    let baseline = screen.used();
    let full = vec![0x12345678u32; 1024 * 1024];
    host.renderer
        .panel_gpu
        .sync(&host.device, &host.queue, "retained", &full)
        .unwrap();
    assert_eq!(retention.used(), 4 * 1024 * 1024);
    let retained_hold = host.renderer.panel_gpu.submission_ref().unwrap();
    let replacement = host
        .renderer
        .recreate_for_device(
            &host.device,
            &host.queue,
            wgpu::TextureFormat::Rgba8UnormSrgb,
            1,
        )
        .unwrap();
    assert!(std::sync::Arc::ptr_eq(
        &retention,
        &replacement.control_gpu_budget
    ));
    drop(replacement);
    let small = [0x87654321u32; 16];
    host.renderer
        .menu_overlay_gpu
        .sync(&host.device, &host.queue, "uncached", &small)
        .unwrap();
    let transient_hold = host.renderer.menu_overlay_gpu.submission_ref().unwrap();
    assert_eq!(retention.used(), 4 * 1024 * 1024);
    assert_eq!(screen.used(), baseline + full.len() as u64 * 4 + 64);
    let mut batch = host
        .renderer
        .flush_frame_uploads(&host.device, &host.queue)
        .unwrap()
        .unwrap();
    host.queue.submit([batch.command()]);
    host.renderer.hold_frame_submission(&host.queue);
    batch.hold(&host.queue);
    assert!(host.renderer.panel_gpu.buffer().is_some());
    assert!(host.renderer.menu_overlay_gpu.buffer().is_none());
    host.device
        .poll(wgpu::PollType::wait_indefinitely())
        .unwrap();
    assert_eq!(screen.used(), baseline + full.len() as u64 * 4 + 64);
    drop(transient_hold);
    assert_eq!(screen.used(), baseline + full.len() as u64 * 4);
    // Removing the cache owner cannot forgive the in-flight reservation.
    host.renderer
        .panel_gpu
        .sync::<u32>(&host.device, &host.queue, "empty", &[])
        .unwrap();
    assert_eq!(retention.used(), 4 * 1024 * 1024);
    drop(retained_hold);
    assert_eq!(retention.used(), 0);
    host.renderer
        .menu_overlay_gpu
        .sync(&host.device, &host.queue, "retry", &small)
        .unwrap();
    assert_eq!(retention.used(), 64);
    let mut batch = host
        .renderer
        .flush_frame_uploads(&host.device, &host.queue)
        .unwrap()
        .unwrap();
    host.queue.submit([batch.command()]);
    host.renderer.hold_frame_submission(&host.queue);
    batch.hold(&host.queue);
    host.device
        .poll(wgpu::PollType::wait_indefinitely())
        .unwrap();
    assert!(host.renderer.menu_overlay_gpu.buffer().is_some());
    host.renderer
        .menu_overlay_gpu
        .sync(&host.device, &host.queue, "warm", &small)
        .unwrap();
    assert_eq!(host.renderer.menu_overlay_gpu.last_upload_bytes, 0);
}

#[test]
#[ignore = "requires local GPU and serial resource admission"]
fn oversized_control_gpu_buffer_uploads_all_content_without_retention() {
    let mut host = hardware_renderer(64, 64);
    let words = vec![0x76543210u32; 1024 * 1024 + 1];
    host.renderer
        .panel_gpu
        .sync(&host.device, &host.queue, "oversized-control", &words)
        .unwrap();
    assert_eq!(host.renderer.control_mesh_retained_gpu_bytes(), 0);
    let source = host.renderer.panel_gpu.buffer().unwrap().clone();
    let held = host.renderer.panel_gpu.submission_ref().unwrap();
    let mut batch = host
        .renderer
        .flush_frame_uploads(&host.device, &host.queue)
        .unwrap()
        .unwrap();
    host.queue.submit([batch.command()]);
    host.renderer.hold_frame_submission(&host.queue);
    batch.hold(&host.queue);
    assert!(host.renderer.panel_gpu.buffer().is_none());
    let target = host.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("oversized-control-readback"),
        size: source.size(),
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let mut encoder = host.device.create_command_encoder(&Default::default());
    encoder.copy_buffer_to_buffer(&source, 0, &target, 0, source.size());
    host.queue.submit([encoder.finish()]);
    let (tx, rx) = std::sync::mpsc::channel();
    target
        .slice(..)
        .map_async(wgpu::MapMode::Read, move |r| tx.send(r).unwrap());
    host.device
        .poll(wgpu::PollType::wait_indefinitely())
        .unwrap();
    rx.recv().unwrap().unwrap();
    assert_eq!(
        &*target.slice(..).get_mapped_range(),
        bytemuck::cast_slice::<u32, u8>(&words)
    );
    target.unmap();
    drop(source);
    drop(held);
}

#[test]
#[ignore = "requires local GPU; failed dialog retention and recovery"]
fn failed_dialog_releases_excess_labels_and_retry_matches_fresh_pixels() {
    let state = crate::global_preferences_dialog_tests::state_with_preferences_open();
    let original =
        PreparedScene::from_native_preferences(&state.ui.global_preferences, 960, 720, 1.0);
    let mut oversized =
        PreparedScene::from_native_preferences(&state.ui.global_preferences, 960, 720, 1.0);
    let template = oversized.menu_overlay_text_runs[0].clone();
    oversized.menu_overlay_text_runs = (0..160)
        .map(|n| {
            let mut run = template.clone();
            run.text = format!("{n:03} {}", "🙂".repeat(55));
            run.rich_spans.clear();
            run
        })
        .collect();
    let mut host = hardware_renderer(960, 720);
    host.renderer.atlas.set_test_limit(0);
    let target =
        crate::capture_resource::CaptureTarget::new(&host.device, host.extent(), OUTPUT_FORMAT)
            .unwrap();
    let error = host
        .renderer
        .render(
            &host.device,
            &host.queue,
            &target.create_view(&Default::default()),
            &oversized,
            &RetainedScene::empty(),
            None,
            960,
            720,
        )
        .unwrap_err();
    assert!(
        format!("{error:#}").contains("GPU resource budget exhausted"),
        "{error:#}"
    );
    let usage = host.renderer.text_cache_key_usage();
    assert!(
        usage.entries > 0,
        "exercise post-layout failure, not wholesale CPU admission refusal"
    );
    assert!(usage.entries <= 128);
    assert!(usage.key_text_bytes <= 32 * 1024);
    assert!(host.renderer.text_preparation.is_invalid());
    host.renderer.atlas.set_test_limit(32 * 1024 * 1024);
    let recovered = capture(&mut host, &original);
    let mut fresh = hardware_renderer(960, 720);
    assert_eq!(recovered, capture(&mut fresh, &original));
}

#[test]
#[ignore = "requires local GPU; composed label retention and pixel parity"]
fn composed_workspace_dialog_limits_labels_without_losing_current_content() {
    let state = crate::global_preferences_dialog_tests::state_with_preferences_open();
    let mut prepared =
        PreparedScene::from_native_preferences(&state.ui.global_preferences, 960, 720, 1.0);
    let template = prepared.menu_overlay_text_runs[0].clone();
    prepared.menu_overlay_text_runs = (0..160)
        .map(|n| {
            let mut label = template.clone();
            label.text = format!("Composed label {n}");
            label.rich_spans.clear();
            label.y = 80.0 + (n % 24) as f32 * 20.0;
            label.x = 20.0 + (n / 24) as f32 * 130.0;
            label
        })
        .collect();
    let mut workspace = template;
    workspace.text = "Workspace-owned text".into();
    workspace.rich_spans.clear();
    prepared.text_runs = vec![workspace];
    assert!(!prepared.is_overlay_only());
    let mut host = hardware_renderer(960, 720);
    let first = capture(&mut host, &prepared);
    let usage = host.renderer.text_cache_key_usage();
    assert_eq!(usage.label_entries, 128);
    assert!(usage.label_key_text_bytes <= 32 * 1024);
    assert_eq!(usage.entries, 129);
    assert_eq!(first, capture(&mut host, &prepared));
    let mut fresh = hardware_renderer(960, 720);
    assert_eq!(first, capture(&mut fresh, &prepared));
}
