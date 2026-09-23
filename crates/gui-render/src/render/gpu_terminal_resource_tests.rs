//! Exercise shared terminal graphics ownership through actual frame submission.
use super::*;

#[test]
#[ignore = "requires local GPU; terminal graphics resource and pixel proof"]
fn terminal_graphics_reuse_quads_and_retire_textures_without_changing_pixels() {
    let mut state = datum_gui_protocol::load_fixture_workspace_state();
    state.ui.active_dock_tab = Some(datum_gui_protocol::DockTab::Terminal);
    state.ui.dock_height_px = 220;
    let snapshot = super::super::tests::sixel_snapshot(true);
    let retained = RetainedScene::from_workspace(&state, 960, 720);
    let camera = CameraState::fit_to_bounds(&state.scene.bounds);
    let prepared = PreparedScene::from_workspace_with_terminal_snapshot(
        &state,
        960,
        720,
        1.0,
        camera,
        &retained,
        Some(&snapshot),
    );
    assert!(prepared.terminal_graphics.len() >= 2);
    let mut renderer = hardware_renderer(960, 720);
    renderer
        .renderer
        .sync_terminal_graphics(&renderer.device, &renderer.queue, &prepared, 960, 720)
        .unwrap();
    let cancelled: Vec<_> = Renderer::gpu_process_allocations()
        .into_iter()
        .filter(|r| r.kind == crate::text_gpu::Kind::TerminalTexture)
        .collect();
    assert!(!cancelled.is_empty());
    renderer.renderer.terminal_graphics.cancel_uploads();
    for texture in cancelled {
        assert!(
            !Renderer::gpu_process_allocations()
                .iter()
                .any(|r| r.id == texture.id)
        );
    }
    let cold = capture_retained(&mut renderer, &prepared, &retained);
    let initial = renderer.renderer.terminal_graphics.vertex_state();
    assert!(initial.iter().all(|(_, bytes)| *bytes == 96));
    let textures: Vec<_> = Renderer::gpu_process_allocations()
        .into_iter()
        .filter(|r| r.kind == crate::text_gpu::Kind::TerminalTexture)
        .collect();
    assert_eq!(textures.len(), prepared.terminal_graphics.len());
    assert!(cold == capture_retained(&mut renderer, &prepared, &retained));
    let warm = renderer.renderer.terminal_graphics.vertex_state();
    for ((old, _), (current, bytes)) in initial.iter().zip(&warm) {
        assert_eq!(old, current);
        assert_eq!(*bytes, 0, "unchanged terminal placement must not upload");
    }
    assert_eq!(
        textures,
        Renderer::gpu_process_allocations()
            .into_iter()
            .filter(|r| r.kind == crate::text_gpu::Kind::TerminalTexture)
            .collect::<Vec<_>>()
    );
    let mut moved = prepared.clone();
    for graphic in &mut moved.terminal_graphics {
        graphic.rect.x += 10.0;
    }
    moved.terminal_graphics.reverse();
    let changed = capture_retained(&mut renderer, &moved, &retained);
    assert_ne!(cold, changed);
    assert!(
        renderer
            .renderer
            .terminal_graphics
            .vertex_state()
            .iter()
            .any(|(_, bytes)| *bytes > 0)
    );
    let mut fresh = hardware_renderer(960, 720);
    assert!(changed == capture_retained(&mut fresh, &moved, &retained));
    let mut clipped = moved.clone();
    for graphic in &mut clipped.terminal_graphics {
        graphic.clip.width = 0.0;
    }
    let hidden = capture_retained(&mut renderer, &clipped, &retained);
    assert!(
        renderer
            .renderer
            .terminal_graphics
            .vertex_state()
            .is_empty()
    );
    assert!(hidden == capture_retained(&mut fresh, &clipped, &retained));
    assert!(cold == capture_retained(&mut renderer, &prepared, &retained));
    // Explicitly retain the same submission references across consumer close.
    let held: Vec<_> = renderer
        .renderer
        .terminal_graphics
        .submission_refs()
        .collect();
    let mut closed = prepared.clone();
    closed.terminal_graphics.clear();
    capture_retained(&mut renderer, &closed, &retained);
    assert!(
        renderer
            .renderer
            .terminal_graphics
            .vertex_state()
            .is_empty()
    );
    for texture in &textures {
        assert!(
            Renderer::gpu_process_allocations()
                .iter()
                .any(|r| r.id == texture.id && r.retiring)
        );
    }
    drop(held);
    for texture in &textures {
        assert!(
            !Renderer::gpu_process_allocations()
                .iter()
                .any(|r| r.id == texture.id)
        );
    }
    assert!(cold == capture_retained(&mut renderer, &prepared, &retained));
}

#[test]
#[ignore = "requires local GPU and serial execution: terminal process admission"]
fn terminal_admission_includes_quads_and_holds_capacity_until_retirement() {
    let mut state = datum_gui_protocol::load_fixture_workspace_state();
    state.ui.active_dock_tab = Some(datum_gui_protocol::DockTab::Terminal);
    state.ui.dock_height_px = 220;
    let snapshot = super::super::tests::sixel_snapshot(true);
    let retained = RetainedScene::from_workspace(&state, 960, 720);
    let camera = CameraState::fit_to_bounds(&state.scene.bounds);
    let mut prepared = PreparedScene::from_workspace_with_terminal_snapshot(
        &state,
        960,
        720,
        1.0,
        camera,
        &retained,
        Some(&snapshot),
    );
    prepared.terminal_graphics.truncate(1);
    let placement = prepared.terminal_graphics[0].graphic.placement();
    let bytes = u64::from(placement.width()) * u64::from(placement.height()) * 4;
    let mut renderer = hardware_renderer(960, 720);
    let host_budget = renderer.renderer.screen_budget.clone();
    let host_baseline = host_budget.used();
    let budget = crate::text_gpu::budget::terminal_process();
    let baseline = budget.used();
    // Leave exactly texture capacity: the placement quad must also be admitted.
    let filler = budget.reserve(64 * 1024 * 1024 - baseline - bytes).unwrap();
    assert!(
        renderer
            .renderer
            .sync_terminal_graphics(&renderer.device, &renderer.queue, &prepared, 960, 720,)
            .is_err()
    );
    assert_eq!(budget.used(), 64 * 1024 * 1024);
    renderer.renderer.terminal_graphics.cancel_uploads();
    assert_eq!(budget.used(), 64 * 1024 * 1024 - bytes);
    drop(filler);
    renderer
        .renderer
        .sync_terminal_graphics(&renderer.device, &renderer.queue, &prepared, 960, 720)
        .unwrap();
    assert_eq!(budget.used(), baseline + bytes + 96);
    assert_eq!(host_budget.used(), host_baseline + 96);
    let held: Vec<_> = renderer
        .renderer
        .terminal_graphics
        .submission_refs()
        .collect();
    drop(renderer);
    assert_eq!(host_budget.used(), 96);
    assert_eq!(budget.used(), baseline + bytes + 96);
    drop(held);
    assert_eq!(host_budget.used(), 0);
    assert_eq!(budget.used(), baseline);
}

#[test]
#[ignore = "requires local GPU; terminal pixel continuation and replacement"]
fn terminal_pixels_yield_with_bounded_padded_staging_and_resume_current_content() {
    let mut state = datum_gui_protocol::load_fixture_workspace_state();
    state.ui.active_dock_tab = Some(datum_gui_protocol::DockTab::Terminal);
    state.ui.dock_height_px = 220;
    let snapshot = super::super::tests::sixel_snapshot_sized(false, 1025, 1100);
    let retained = RetainedScene::from_workspace(&state, 960, 720);
    let camera = CameraState::fit_to_bounds(&state.scene.bounds);
    let mut prepared = PreparedScene::from_workspace_with_terminal_snapshot(
        &state,
        960,
        720,
        1.0,
        camera,
        &retained,
        Some(&snapshot),
    );
    assert_eq!(prepared.terminal_graphics.len(), 1);
    let mut renderer = hardware_renderer(960, 720);
    // Warm unrelated resources before introducing the cold terminal image.
    let mut empty = prepared.clone();
    empty.terminal_graphics.clear();
    capture_retained(&mut renderer, &empty, &retained);
    let target = renderer.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("terminal-continuation-proof"),
        size: wgpu::Extent3d {
            width: 960,
            height: 720,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });
    let view = target.create_view(&Default::default());
    let mut submissions = 0;
    for attempt in 0..3 {
        let ready = renderer
            .renderer
            .render_with_submission(
                &renderer.device,
                &renderer.queue,
                &view,
                &prepared,
                &retained,
                None,
                960,
                720,
                &mut |_| submissions += 1,
            )
            .unwrap();
        assert_eq!(ready, attempt == 2);
        assert!(
            renderer.renderer.upload_staging_reserved_bytes()
                - renderer.renderer.layout_scratch_reserved_bytes()
                - renderer.renderer.pending_glyph_pixel_bytes()
                - renderer.renderer.screen_upload_metadata_bytes()
                - renderer.renderer.screen_upload_snapshot_bytes()
                - renderer.renderer.glyph_upload_storage_bytes()
                - renderer.renderer.text_preparation_storage_bytes()
                - renderer.renderer.raster_scratch_reserved_bytes()
                - renderer.renderer.atlas_page_metadata_bytes()
                - renderer.renderer.atlas_lookup_metadata_bytes()
                <= 4 * 1024 * 1024
        );
        renderer
            .device
            .poll(wgpu::PollType::wait_indefinitely())
            .unwrap();
        assert_eq!(
            renderer.renderer.upload_staging_reserved_bytes(),
            renderer.renderer.layout_scratch_reserved_bytes()
                + renderer.renderer.pending_glyph_pixel_bytes()
                + renderer.renderer.screen_upload_metadata_bytes()
                + renderer.renderer.screen_upload_snapshot_bytes()
                + renderer.renderer.glyph_upload_storage_bytes()
                + renderer.renderer.text_preparation_storage_bytes()
                + renderer.renderer.raster_scratch_reserved_bytes()
                + renderer.renderer.atlas_page_metadata_bytes()
                + renderer.renderer.atlas_lookup_metadata_bytes()
        );
        if attempt == 0 {
            let filler = renderer
                .renderer
                .atlas
                .staging_budget
                .reserve(renderer.renderer.atlas.staging_budget.available())
                .unwrap();
            let result = renderer.renderer.render_with_submission(
                &renderer.device,
                &renderer.queue,
                &view,
                &prepared,
                &retained,
                None,
                960,
                720,
                &mut |_| panic!("refused staging submitted"),
            );
            assert!(result.is_err());
            assert_eq!(renderer.renderer.terminal_upload_chunk_count(), 1);
            drop(filler);
        }
    }
    assert_eq!(submissions, 3);
    assert_eq!(renderer.renderer.terminal_upload_chunk_count(), 2);
    assert_eq!(renderer.renderer.terminal_upload_chunk_bytes(), 4352 * 1100);
    let cold = capture_retained(&mut renderer, &prepared, &retained);
    assert_eq!(renderer.renderer.terminal_upload_chunk_count(), 2);
    let mut fresh = hardware_renderer(960, 720);
    assert!(cold == capture_retained(&mut fresh, &prepared, &retained));
    // A distinct pixel owner with identical dimensions must upload from zero.
    let replacement = super::super::tests::sixel_snapshot_sized(false, 1025, 1100);
    prepared.terminal_graphics[0].graphic = replacement.graphics().next().unwrap().clone();
    assert!(
        !renderer
            .renderer
            .render_with_submission(
                &renderer.device,
                &renderer.queue,
                &view,
                &prepared,
                &retained,
                None,
                960,
                720,
                &mut |_| {}
            )
            .unwrap()
    );
    renderer
        .device
        .poll(wgpu::PollType::wait_indefinitely())
        .unwrap();
    let partial: Vec<_> = Renderer::gpu_process_allocations()
        .into_iter()
        .filter(|r| r.kind == crate::text_gpu::Kind::TerminalTexture)
        .collect();
    // Remove the partially uploaded image; no stale pixels or further copy turn.
    assert!(
        capture_retained(&mut renderer, &empty, &retained)
            == capture_retained(&mut fresh, &empty, &retained)
    );
    assert_eq!(renderer.renderer.terminal_upload_chunk_count(), 3);
    for record in partial {
        assert!(
            !Renderer::gpu_process_allocations()
                .iter()
                .any(|r| r.id == record.id)
        );
    }
    assert!(cold == capture_retained(&mut renderer, &prepared, &retained));
    assert_eq!(renderer.renderer.terminal_upload_chunk_count(), 5);
}

#[test]
#[ignore = "requires local GPU; terminal quad close and recovery generations"]
fn terminal_quad_slots_preserve_submission_limits_after_close_and_recovery() {
    let mut state = datum_gui_protocol::load_fixture_workspace_state();
    state.ui.active_dock_tab = Some(datum_gui_protocol::DockTab::Terminal);
    state.ui.dock_height_px = 220;
    let snapshot = super::super::tests::sixel_snapshot(true);
    let retained = RetainedScene::from_workspace(&state, 960, 720);
    let prepared = PreparedScene::from_workspace_with_terminal_snapshot(
        &state,
        960,
        720,
        1.0,
        CameraState::fit_to_bounds(&state.scene.bounds),
        &retained,
        Some(&snapshot),
    );
    assert!(!prepared.terminal_graphics.is_empty());
    let mut clipped = prepared.terminal_graphics.clone();
    for graphic in &mut clipped {
        graphic.clip.width = 0.0;
    }
    let mut host = hardware_renderer(960, 720);
    macro_rules! sync {
        ($graphics:expr) => {
            host.renderer
                .terminal_graphics
                .sync(&host.device, &host.queue, $graphics, 960, 720)
        };
    }
    sync!(&prepared.terminal_graphics).unwrap();
    let first: Vec<_> = host.renderer.terminal_graphics.submission_refs().collect();
    sync!(&clipped).unwrap();
    sync!(&prepared.terminal_graphics).unwrap();
    let second: Vec<_> = host.renderer.terminal_graphics.submission_refs().collect();
    sync!(&clipped).unwrap();
    let before = host.renderer.screen_gpu_reserved_bytes();
    assert!(
        sync!(&prepared.terminal_graphics)
            .unwrap_err()
            .to_string()
            .contains("two live GPU allocations")
    );
    assert_eq!(host.renderer.screen_gpu_reserved_bytes(), before);
    host.renderer = host
        .renderer
        .recreate_for_device(
            &host.device,
            &host.queue,
            OUTPUT_FORMAT,
            DEFAULT_MSAA_SAMPLES,
        )
        .unwrap();
    assert!(sync!(&prepared.terminal_graphics).is_err());
    assert_eq!(host.renderer.screen_gpu_reserved_bytes(), before);
    drop(first);
    sync!(&prepared.terminal_graphics).unwrap();
    assert!(
        host.renderer
            .terminal_graphics
            .vertex_state()
            .iter()
            .all(|(_, bytes)| *bytes > 0)
    );
    drop(second);
    let screen = host.renderer.screen_budget.clone();
    drop(host);
    assert_eq!(screen.used(), 0);
}

#[test]
#[ignore = "requires local GPU; immutable terminal texture recovery admission"]
fn terminal_texture_generations_survive_close_reopen_and_recovery() {
    let mut state = datum_gui_protocol::load_fixture_workspace_state();
    state.ui.active_dock_tab = Some(datum_gui_protocol::DockTab::Terminal);
    state.ui.dock_height_px = 220;
    let snapshot = super::super::tests::sixel_snapshot(true);
    let retained = RetainedScene::from_workspace(&state, 960, 720);
    let prepared = PreparedScene::from_workspace_with_terminal_snapshot(
        &state,
        960,
        720,
        1.0,
        CameraState::fit_to_bounds(&state.scene.bounds),
        &retained,
        Some(&snapshot),
    );
    assert!(!prepared.terminal_graphics.is_empty());
    let mut host = hardware_renderer(960, 720);
    let baseline = Renderer::terminal_graphics_gpu_reserved_bytes();
    macro_rules! sync {
        ($graphics:expr) => {
            host.renderer
                .terminal_graphics
                .sync(&host.device, &host.queue, $graphics, 960, 720)
        };
    }
    sync!(&prepared.terminal_graphics).unwrap();
    let first: Vec<_> = host.renderer.terminal_graphics.submission_refs().collect();
    sync!(&[]).unwrap();
    sync!(&prepared.terminal_graphics).unwrap();
    let second: Vec<_> = host.renderer.terminal_graphics.submission_refs().collect();
    sync!(&[]).unwrap();
    let before = Renderer::terminal_graphics_gpu_reserved_bytes();
    let error = sync!(&prepared.terminal_graphics).unwrap_err();
    assert!(
        error
            .to_string()
            .contains("terminal texture has two live GPU allocations")
    );
    assert_eq!(Renderer::terminal_graphics_gpu_reserved_bytes(), before);
    host.renderer = host
        .renderer
        .recreate_for_device(
            &host.device,
            &host.queue,
            OUTPUT_FORMAT,
            DEFAULT_MSAA_SAMPLES,
        )
        .unwrap();
    assert!(
        sync!(&prepared.terminal_graphics)
            .unwrap_err()
            .to_string()
            .contains("terminal texture has two live GPU allocations")
    );
    assert_eq!(Renderer::terminal_graphics_gpu_reserved_bytes(), before);
    drop(first);
    sync!(&prepared.terminal_graphics).unwrap();
    drop(second);
    drop(host);
    assert_eq!(Renderer::terminal_graphics_gpu_reserved_bytes(), baseline);
    assert!(
        snapshot.graphics().next().is_some(),
        "renderer refusal preserves terminal authority"
    );
}
