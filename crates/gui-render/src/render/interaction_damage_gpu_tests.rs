use super::*;

#[test]
#[ignore = "requires local GPU; exact partial redraw comparison; run serially"]
fn interaction_damage_matches_full_redraw_and_invalidates_changed_composition() {
    let mut state = crate::gpu_surface_pass::board_fixture_state();
    let retained = RetainedScene::from_workspace_for_surface(&state, 960, 720, 1.0);
    let mut prepared = PreparedScene::from_workspace_for_surface(
        &state,
        960,
        720,
        1.0,
        CameraState::fit_to_bounds(&state.scene.bounds),
        &retained,
    )
    .unwrap();
    let pane = prepared
        .surface_passes()
        .iter()
        .find(|p| p.surface == crate::SceneSurface::Board)
        .unwrap()
        .clone();
    let mut gpu = hardware_renderer(960, 720);
    let initial = capture_retained(&mut gpu, &prepared, &retained);
    let attachment = gpu.renderer.surface_attachment_snapshot().unwrap();
    let hover_id = retained
        .world_hit_index
        .regions()
        .iter()
        .find_map(|region| match &region.target {
            crate::HitTarget::AuthoredObject(id) | crate::HitTarget::ReviewAction(id) => {
                Some(id.clone())
            }
            _ => None,
        })
        .expect("board fixture exposes a hover target");
    let mut previous = initial;
    for (index, style) in [
        datum_gui_protocol::CrosshairStyle::FullViewport,
        datum_gui_protocol::CrosshairStyle::FullViewport,
        datum_gui_protocol::CrosshairStyle::Local,
        datum_gui_protocol::CrosshairStyle::None,
    ]
    .into_iter()
    .enumerate()
    {
        state.ui.crosshair_style = style;
        state.ui.hovered_object = (index < 2).then(|| datum_gui_protocol::HoverTarget {
            object_id: hover_id.clone(),
            surface: datum_gui_protocol::PaneContent::Board,
        });
        state.ui.cursor_pos = Some(datum_gui_protocol::ScreenPointPx {
            x: pane.scene_viewport.x + pane.scene_viewport.width * (0.35 + index as f32 * 0.08),
            y: pane.scene_viewport.y + pane.scene_viewport.height * 0.5,
        });
        prepared.refresh_interaction(&state, &retained);
        let actual = capture_retained(&mut gpu, &prepared, &retained);
        assert!(
            gpu.renderer.damage_regions > 0,
            "partial path must execute at step {index}"
        );
        assert!(
            actual != previous,
            "interaction must change pixels at step {index}"
        );
        gpu.renderer.preserved_interaction = None;
        let expected = capture_retained(&mut gpu, &prepared, &retained);
        assert!(actual == expected, "partial/full mismatch at step {index}");
        assert_eq!(
            gpu.renderer.surface_attachment_snapshot().unwrap(),
            attachment
        );
        previous = expected;
    }
    let mut camera = pane.camera;
    camera.center_x_nm += 500_000.0;
    prepared.set_surface_camera(pane.pane_id, camera);
    capture_retained(&mut gpu, &prepared, &retained);
    assert_eq!(
        gpu.renderer.damage_regions, 0,
        "camera change must repaint full composition"
    );
    let deferred = gpu
        .renderer
        .render_with_acquisition(
            &gpu.device,
            &gpu.queue,
            &prepared,
            &retained,
            None,
            960,
            720,
            &mut (),
            &mut |_| Ok(None),
            &mut |_, _| {},
        )
        .unwrap();
    assert!(!deferred);
    assert!(
        gpu.renderer.preserved_interaction.is_none(),
        "failed acquisition invalidates reuse"
    );
    capture_retained(&mut gpu, &prepared, &retained);
    assert_eq!(gpu.renderer.damage_regions, 0);
}

fn capture_scenes(
    gpu: &mut OffscreenRenderer,
    prepared: &PreparedScene,
    board: &RetainedScene,
    schematic: Option<&RetainedScene>,
) -> RgbaImage {
    let target =
        crate::capture_resource::CaptureTarget::new(&gpu.device, gpu.extent(), OUTPUT_FORMAT)
            .unwrap();
    gpu.renderer
        .render(
            &gpu.device,
            &gpu.queue,
            &target.create_view(&wgpu::TextureViewDescriptor::default()),
            prepared,
            board,
            schematic,
            gpu.width,
            gpu.height,
        )
        .unwrap();
    gpu.read_texture(&target).unwrap()
}

fn receipt_observer(
    frame: crate::resource_observation::FrameObservation<'_>,
) -> anyhow::Result<()> {
    if frame.submitted_frame != Some(true) {
        return Ok(());
    }
    assert!(frame.renderer.encoded_screen_geometry().is_some());
    for pane in frame.renderer.encoded_world_admission() {
        let viewport = frame
            .prepared
            .surface_passes()
            .iter()
            .find(|p| p.pane_id == pane.pane_id)
            .unwrap()
            .scene_viewport;
        let mask = pane.encoded_regions().expect("world receipt valid");
        let clip = [
            viewport.x.max(0.0).floor() as u32,
            viewport.y.max(0.0).floor() as u32,
            viewport.width.max(1.0).ceil() as u32,
            viewport.height.max(1.0).ceil() as u32,
        ];
        assert!(
            frame
                .renderer
                .encoded_region_scissors(mask, clip)
                .all(|r| r.is_some())
        );
    }
    for grid in frame.renderer.grid_geometry_admission().unwrap() {
        let mask = grid.encoded_regions().expect("grid receipt valid");
        assert_eq!(grid.encoded(), mask != 0);
    }
    for graphic in frame.renderer.terminal_geometry_admission() {
        let mask = graphic.encoded_regions.expect("terminal receipt valid");
        assert!(
            frame
                .renderer
                .encoded_region_scissors(mask, graphic.scissor)
                .all(|r| r.is_some())
        );
    }
    Ok(())
}

#[test]
#[ignore = "requires local GPU; mixed-layer damage, receipts and recovery; run serially"]
fn interaction_damage_preserves_mixed_painters_pane_switches_and_resource_replacement() {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../engine/testdata/import/kicad/simple-demo.kicad_sch");
    let mut state = datum_gui_protocol::load_fixture_workspace_state();
    state.schematic_scene = Some(
        datum_gui_protocol::load_kicad_schematic_workspace_state(&path)
            .unwrap()
            .scene,
    );
    state.ui.active_dock_tab = Some(datum_gui_protocol::DockTab::Terminal);
    state.ui.dock_height_px = 220;
    let snapshot = super::super::tests::sixel_snapshot(true);
    let board = RetainedScene::from_workspace_for_surface(&state, 960, 720, 1.0);
    let schematic =
        RetainedScene::from_workspace_schematic_for_surface(&state, 960, 720, 1.0).unwrap();
    let mut prepared = PreparedScene::from_workspace_with_terminal_snapshot(
        &state,
        960,
        720,
        1.0,
        CameraState::fit_to_bounds(&state.scene.bounds),
        &board,
        Some(&snapshot),
    )
    .unwrap();
    let panes = prepared.surface_passes().to_vec();
    assert_eq!(panes.len(), 2);
    let viewport = panes[0].scene_viewport;
    let patch = crate::RectPx {
        x: viewport.x + viewport.width * 0.4,
        y: viewport.y + viewport.height * 0.35,
        width: 60.0,
        height: 45.0,
    };
    // Stress overlap deliberately: real terminal textures, main glyphs, Console
    // and menu chrome share the old/new pointer footprint in this fixture.
    assert!(prepared.terminal_graphics.len() >= 2);
    for graphic in &mut prepared.terminal_graphics {
        graphic.rect = patch;
        graphic.clip = viewport;
    }
    let mut label = prepared.text_runs.first().unwrap().clone();
    label.text = "Damage Åg".into();
    label.rich_spans.clear();
    label.x = patch.x;
    label.y = patch.y;
    label.clip_bounds = Some(viewport);
    label.layout_size = None;
    prepared.text_runs.push(label.clone());
    prepared.menu_overlay_text_runs.push(label);
    prepared.menu_overlay_vertices =
        crate::gpu_data::quads_to_vertices(&[crate::Quad::from_rect(patch, [0.1, 0.2, 0.3])]);
    prepared.console_overlay_vertices = prepared.menu_overlay_vertices.clone();
    prepared.console_overlay_layout = Some(crate::ConsoleOverlayLayout {
        pane_id: panes[0].pane_id,
        pane_body: viewport,
        strip: patch,
        text_clip: viewport,
        history_panel: None,
    });
    prepared.composition_revision.invalidate();
    let mut gpu = hardware_renderer(960, 720);
    gpu.renderer.frame_observer = Some(receipt_observer);
    capture_scenes(&mut gpu, &prepared, &board, Some(&schematic));
    let attachment = gpu.renderer.surface_attachment_snapshot().unwrap();
    for pane_index in [0, 0, 1, 0] {
        let pane = &panes[pane_index];
        state.ui.cursor_pos = Some(datum_gui_protocol::ScreenPointPx {
            x: pane.scene_viewport.x + pane.scene_viewport.width * 0.5,
            y: pane.scene_viewport.y + pane.scene_viewport.height * 0.5,
        });
        prepared.refresh_interaction(&state, &board);
        let actual = capture_scenes(&mut gpu, &prepared, &board, Some(&schematic));
        assert!(gpu.renderer.damage_regions > 0);
        gpu.renderer.invalidate_preserved_frame();
        let expected = capture_scenes(&mut gpu, &prepared, &board, Some(&schematic));
        assert!(
            actual == expected,
            "mixed painter mismatch switching to pane {pane_index}"
        );
        assert_eq!(
            gpu.renderer.surface_attachment_snapshot().unwrap(),
            attachment
        );
    }
    // Changed static text, resolved LOD, attachment extent and device owner each
    // force full redraw; none can inherit old interaction-only damage.
    prepared
        .text_runs
        .last_mut()
        .unwrap()
        .text
        .push_str(" changed");
    prepared.composition_revision.invalidate();
    capture_scenes(&mut gpu, &prepared, &board, Some(&schematic));
    assert_eq!(gpu.renderer.damage_regions, 0);
    let lod = datum_gui_viewport::GridLodState { tier: Some(0) };
    prepared.set_surface_grid_lod(panes[0].pane_id, Default::default(), lod);
    capture_scenes(&mut gpu, &prepared, &board, Some(&schematic));
    assert_eq!(gpu.renderer.damage_regions, 0);
    gpu.width = 1000;
    capture_scenes(&mut gpu, &prepared, &board, Some(&schematic));
    assert_eq!(gpu.renderer.damage_regions, 0);
    let replacement = gpu
        .renderer
        .recreate_for_device(&gpu.device, &gpu.queue, OUTPUT_FORMAT, DEFAULT_MSAA_SAMPLES)
        .unwrap();
    assert!(replacement.preserved_interaction.is_none());
    assert!(replacement.damage_reset.get().is_none());
    gpu.renderer = replacement;
    capture_scenes(&mut gpu, &prepared, &board, Some(&schematic));
    assert_eq!(gpu.renderer.damage_regions, 0);
}
