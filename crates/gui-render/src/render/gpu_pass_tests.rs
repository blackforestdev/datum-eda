//! Production pass contributions and resolve pixel equivalence.
use super::*;

#[test]
#[ignore = "requires timestamp-capable hardware; production pass contribution proof"]
fn empty_workspace_text_omits_preparation_and_pass_without_pixel_change() {
    let state = crate::global_preferences_dialog_tests::state_with_preferences_open();
    let mut renderer = hardware_renderer_with_features(960, 720, wgpu::Features::TIMESTAMP_QUERY);
    let mut prepared =
        PreparedScene::from_native_preferences(&state.ui.global_preferences, 960, 720, 1.0)
            .unwrap();
    let mut invisible = prepared.menu_overlay_text_runs[0].clone();
    invisible.x = -1000.0;
    invisible.y = -1000.0;
    invisible.clip_bounds = Some(crate::RectPx {
        x: -1000.0,
        y: -1000.0,
        width: 100.0,
        height: 50.0,
    });
    prepared.menu_overlay_text_runs.clear();
    prepared.menu_overlay_vertices.clear();
    prepared.panel_vertices = crate::gpu_data::quads_to_vertices(&[crate::Quad::from_rect(
        crate::RectPx {
            x: 0.25,
            y: 0.5,
            width: 220.0,
            height: 100.0,
        },
        [0.3, 0.5, 0.7],
    )]);
    renderer
        .renderer
        .enable_gpu_measurements(
            &renderer.device,
            &renderer.queue,
            102,
            1,
            Box::new(|r| panic!("unexpected cancellation: {r:?}")),
        )
        .unwrap();
    let without_text = capture(&mut renderer, &prepared);
    let samples = renderer
        .renderer
        .poll_gpu_measurements(&renderer.device)
        .unwrap();
    assert_eq!(
        samples[0].passes_ns.iter().map(|p| p.0).collect::<Vec<_>>(),
        ["scene"]
    );
    assert_eq!(renderer.renderer.text_preparation.workspace_prepares, 0);
    let mut blank = invisible.clone();
    blank.text.clear();
    blank.rich_spans.clear();
    let mut blank_frame = prepared.clone();
    blank_frame.text_runs = vec![blank.clone()];
    for rich in [false, true] {
        if rich {
            blank_frame.text_runs[0].text = "unused plain-text fallback".into();
            blank_frame.text_runs[0].rich_spans = vec![crate::TextRunSpan {
                text: String::new(),
                color: blank.color,
                bold: true,
                italic: false,
            }];
        }
        assert!(without_text == capture(&mut renderer, &blank_frame));
        let samples = renderer
            .renderer
            .poll_gpu_measurements(&renderer.device)
            .unwrap();
        assert_eq!(
            samples[0].passes_ns.iter().map(|p| p.0).collect::<Vec<_>>(),
            ["scene"]
        );
        assert_eq!(renderer.renderer.text_preparation.workspace_prepares, 0);
    }
    let mut extra_pass = prepared.clone();
    extra_pass.text_runs = vec![invisible];
    assert!(without_text == capture(&mut renderer, &extra_pass));
    let samples = renderer
        .renderer
        .poll_gpu_measurements(&renderer.device)
        .unwrap();
    assert_eq!(
        samples[0].passes_ns.iter().map(|p| p.0).collect::<Vec<_>>(),
        ["scene", "text"]
    );
    assert_eq!(renderer.renderer.text_preparation.workspace_prepares, 1);
    assert!(
        without_text == capture(&mut renderer, &prepared),
        "previous glyphs must not leak into empty text frame"
    );
    let samples = renderer
        .renderer
        .poll_gpu_measurements(&renderer.device)
        .unwrap();
    assert_eq!(
        samples[0].passes_ns.iter().map(|p| p.0).collect::<Vec<_>>(),
        ["scene"]
    );
    assert_eq!(renderer.renderer.text_preparation.workspace_prepares, 1);
    let mut menu = prepared.clone();
    menu.menu_overlay_vertices = prepared.panel_vertices.clone();
    menu.menu_overlay_text_runs = vec![blank];
    let card_only = capture(&mut renderer, &menu);
    let samples = renderer
        .renderer
        .poll_gpu_measurements(&renderer.device)
        .unwrap();
    assert_eq!(
        samples[0].passes_ns.iter().map(|p| p.0).collect::<Vec<_>>(),
        ["scene", "menu-background"]
    );
    assert_eq!(renderer.renderer.text_preparation.overlay_prepares, 0);
    menu.menu_overlay_text_runs = extra_pass.text_runs.clone();
    assert!(card_only == capture(&mut renderer, &menu));
    let samples = renderer
        .renderer
        .poll_gpu_measurements(&renderer.device)
        .unwrap();
    assert_eq!(
        samples[0].passes_ns.iter().map(|p| p.0).collect::<Vec<_>>(),
        ["scene", "menu-background", "menu-text"]
    );
    assert_eq!(renderer.renderer.text_preparation.overlay_prepares, 1);
}

#[test]
#[ignore = "requires local GPU; terminal final resolve and invisible later pass parity"]
fn terminal_last_layer_matches_invisible_later_passes() {
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
    )
    .unwrap();
    let mut invisible = prepared.text_runs[0].clone();
    invisible.x = -1000.0;
    invisible.y = -1000.0;
    invisible.clip_bounds = Some(crate::RectPx {
        x: -1000.0,
        y: -1000.0,
        width: 10.0,
        height: 10.0,
    });
    let mut renderer = hardware_renderer(960, 720);
    for foreground in [false, true] {
        let mut terminal = prepared.clone();
        terminal.text_runs.clear();
        terminal.menu_overlay_text_runs.clear();
        terminal.menu_overlay_vertices.clear();
        terminal
            .terminal_graphics
            .retain(|g| (g.graphic.placement().z_index() >= 0) == foreground);
        assert!(!terminal.terminal_graphics.is_empty());
        let expected = capture_retained(&mut renderer, &terminal, &retained);
        let mut absent = terminal.clone();
        absent.terminal_graphics.clear();
        assert_ne!(
            expected,
            capture_retained(&mut renderer, &absent, &retained),
            "the terminal layer must contribute visible pixels"
        );
        terminal.text_runs = vec![invisible.clone()];
        assert_eq!(
            expected,
            capture_retained(&mut renderer, &terminal, &retained),
            "adding invisible text must preserve terminal samples"
        );
        terminal.menu_overlay_vertices =
            crate::gpu_data::quads_to_vertices(&[crate::Quad::from_rect(
                crate::RectPx {
                    x: -100.0,
                    y: -100.0,
                    width: 10.0,
                    height: 10.0,
                },
                [1.0, 0.0, 1.0],
            )]);
        for text in [false, true] {
            if text {
                terminal.menu_overlay_text_runs = vec![invisible.clone()];
            }
            assert_eq!(
                expected,
                capture_retained(&mut renderer, &terminal, &retained),
                "invisible later menu passes must preserve terminal samples"
            );
        }
    }
}
