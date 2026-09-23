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
