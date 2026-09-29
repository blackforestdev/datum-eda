//! Bounded P630 exact-output integration cases; these are not performance trials.
use super::prefix_graph_tests::*;
use super::*;
use crate::render_input::{PointerUpdate, SourceEpoch};
use datum_gui_protocol::{CrosshairStyle, HoverTarget, PaneContent, ScreenPointPx};
#[path = "r3_damage_qualification.rs"]
mod r3;

fn exact(c: &mut OffscreenRenderer, label: &str, reuse: bool) -> RgbaImage {
    let actual = frame(c, true);
    assert_eq!(c.renderer.prefix_copy_work().0, reuse, "{label}: reuse");
    if reuse {
        assert_eq!(
            c.renderer.prefix_copy_work().1,
            0,
            "warm frame copied entire image"
        );
        assert_eq!(c.renderer.world_bundle_execution_count(), 0, "{label}");
    }
    let reference = full_reference(c);
    assert!(actual.as_raw() == reference.as_raw(), "{label}: pixels");
    eprintln!("exact8x P630 case passed: {label}");
    actual
}

#[test]
#[ignore = "bounded P630 exact8x correctness, invalidation and recovery qualification"]
fn p630_exact8_pointer_styles_across_board_and_schematic() {
    let mut c = reference_capture8();
    c.width = 1280;
    c.height = 800;
    let state = crate::gpu_surface_pass::board_fixture_state();
    let source = SourceEpoch::default();
    prepare(&mut c, &state, &source);
    let initial = exact(&mut c, "cold board and schematic", false);
    let uploads = world_uploads(&c);
    for surface in [crate::SceneSurface::Board, crate::SceneSurface::Schematic] {
        let viewport = c
            .renderer
            .render_session()
            .prepared()
            .unwrap()
            .surface_passes()
            .iter()
            .find(|p| p.surface == surface)
            .unwrap()
            .scene_viewport;
        for style in [
            CrosshairStyle::FullViewport,
            CrosshairStyle::Local,
            CrosshairStyle::None,
        ] {
            for cursor in [
                Some(ScreenPointPx {
                    x: viewport.x + viewport.width * 0.371 + 0.25,
                    y: viewport.y + viewport.height * 0.419 + 0.125,
                }),
                Some(ScreenPointPx {
                    x: viewport.x + viewport.width * 0.637 + 0.75,
                    y: viewport.y + viewport.height * 0.583 + 0.625,
                }),
                None,
            ] {
                let session = c.renderer.render_session_mut();
                assert!(session.update_pointer(PointerUpdate {
                    generation: session.pointer_generation(),
                    cursor,
                    hover: None,
                    style,
                }));
                let image = exact(&mut c, &format!("{surface:?}/{style:?}/{cursor:?}"), true);
                assert_eq!(world_uploads(&c), uploads);
                if cursor.is_some() && style != CrosshairStyle::None {
                    assert!(
                        image.as_raw() != initial.as_raw(),
                        "visible crosshair required"
                    );
                }
            }
        }
    }
}

fn doa() -> ReviewWorkspaceState {
    let board = std::path::PathBuf::from(
        std::env::var_os("DATUM_NATIVE_TEST_BOARD").expect("pinned F-DOA board is required"),
    );
    let state = datum_gui_protocol::load_board_editor_workspace_state(
        &datum_gui_protocol::LiveReviewRequest {
            project_root: board.parent().unwrap().to_path_buf(),
            board_file: Some(board),
            artifact_path: None,
            net_uuid: None,
            from_anchor_pad_uuid: None,
            to_anchor_pad_uuid: None,
            profile: None,
            kicad_board_source: None,
        },
    )
    .unwrap();
    assert_eq!(state.scene.pads.len(), 81, "F-DOA pad count");
    state
}

#[test]
#[ignore = "bounded P630 exact8x F-DOA hover, invalidation and recovery qualification"]
fn p630_exact8_doa_hover_composition_and_recovery() {
    let (mut c, adapter) = reference_capture8_with_adapter();
    c.width = 1280;
    c.height = 800;
    let mut state = doa();
    let source = SourceEpoch::default();
    prepare(&mut c, &state, &source);
    exact(&mut c, "F-DOA cold", false);
    // Effective pad hover alters retained material/text, not merely the ring.
    state.selection = datum_gui_protocol::SelectionTarget::None;
    for hover in [
        Some(HoverTarget {
            object_id: state.scene.pads.first().unwrap().object_id.clone(),
            surface: PaneContent::Board,
        }),
        None,
    ] {
        let session = c.renderer.render_session_mut();
        assert!(!session.update_pointer(PointerUpdate {
            generation: session.pointer_generation(),
            cursor: None,
            hover: hover.as_ref(),
            style: CrosshairStyle::Local,
        }));
        assert!(session.prepared().is_none());
        state.ui.hovered_object = hover;
        prepare(&mut c, &state, &source);
        exact(&mut c, "pad hover strong invalidation", false);
        exact(&mut c, "stable pad hover", true);
    }
    for menu in [Some("File".to_owned()), None] {
        state.ui.active_menu = menu;
        prepare(&mut c, &state, &source);
        exact(&mut c, "menu painter order", false);
        exact(&mut c, "menu exposure", true);
    }
    // Above the optional-image cap the exact same painter must render fully.
    c.width = 1600;
    c.height = 1000;
    prepare(&mut c, &state, &source);
    exact(&mut c, "above optional image cap", false);
    assert_eq!(c.renderer.prefix_copy_work().1, 0);
    c.width = 1280;
    c.height = 800;
    prepare(&mut c, &state, &source);
    let before_recovery = exact(&mut c, "return to admitted extent", false);
    let mut replacement = c
        .renderer
        .recreate_for_device(&c.device, &c.queue, OUTPUT_FORMAT, 8)
        .unwrap();
    pollster::block_on(replacement.admit_damage_restoration(&c.device, &adapter));
    replacement.commit_cpu_recovery_from(&mut c.renderer);
    c.renderer = replacement;
    prepare(&mut c, &state, &source);
    let recovered = exact(
        &mut c,
        "replacement resources and preserved CPU session",
        false,
    );
    assert!(before_recovery.as_raw() == recovered.as_raw());
    exact(&mut c, "recovered warm reuse", true);
}

#[test]
#[ignore = "bounded P630 exact8x terminal image/text and menu composition qualification"]
fn p630_exact8_terminal_layers_and_equal_length_replacement() {
    let mut c = reference_capture8();
    c.width = 1280;
    c.height = 800;
    let mut state = crate::gpu_surface_pass::board_fixture_state();
    state.ui.active_dock_tab = Some(datum_gui_protocol::DockTab::Terminal);
    state.ui.dock_height_px = 220;
    let source = SourceEpoch::default();
    let mut previous = None;
    for (label, snapshot) in [
        (
            "terminal text A",
            crate::terminal_core_render::test_snapshot(b"AAAA"),
        ),
        (
            "terminal text B",
            crate::terminal_core_render::test_snapshot(b"BBBB"),
        ),
        (
            "terminal background and foreground images",
            super::super::tests::sixel_snapshot(true),
        ),
    ] {
        for menu in [None, Some("File".to_owned())] {
            state.ui.active_menu = menu;
            let panes = [crate::TerminalPaneRenderState {
                session_id: "terminal".into(),
                focused: true,
                lane: &state.ui.terminal,
                snapshot: snapshot.clone(),
                damage: vec![datum_terminal_core::Damage::Full],
            }];
            c.renderer
                .prepare_session_workspace(
                    &state,
                    WorkspaceView {
                        source_revision: Some(source.revision()),
                        width: c.width,
                        height: c.height,
                        scale: 1.0,
                        camera: CameraState::fit_to_bounds(&state.scene.bounds),
                        schematic_camera: None,
                        include_preferences_overlay: false,
                        single_terminal_snapshot: true,
                        pane_cameras: &[],
                    },
                    &panes,
                )
                .unwrap();
            if label.contains("images") {
                assert_eq!(
                    c.renderer
                        .render_session()
                        .prepared()
                        .unwrap()
                        .terminal_graphics
                        .len(),
                    2
                );
            }
            let image = exact(&mut c, label, false);
            exact(&mut c, "terminal and menu warm composition", true);
            r3::motion(&mut c, "terminal and menu masked pointer frames");
            if state.ui.active_menu.is_none()
                && let Some(old) = previous.replace(image)
            {
                assert!(
                    old.as_raw() != previous.as_ref().unwrap().as_raw(),
                    "replacement must change pixels"
                );
            }
        }
    }
}

#[test]
#[ignore = "P630 exact8x oracle controls; intentionally wrong graphs are not performance evidence"]
fn p630_exact8_rejects_stale_missing_copy_reduced_samples_and_premature_resolve() {
    use crate::gpu_surface::prefix_negative_control::{Fault, set};
    let mut c = reference_capture8();
    c.width = 1280;
    c.height = 800;
    let mut state = doa();
    let source = SourceEpoch::default();
    prepare(&mut c, &state, &source);
    exact(&mut c, "negative controls original warm image", false);
    // A real fractional camera change changes retained pixels while keeping extent.
    let mut camera = CameraState::fit_to_bounds(&state.scene.bounds);
    camera.zoom = 1.173;
    camera.center_x_nm += 31_250.0;
    camera.center_y_nm -= 17_875.0;
    let pane = c
        .renderer
        .render_session()
        .prepared()
        .unwrap()
        .surface_passes()
        .iter()
        .find(|p| p.surface == crate::SceneSurface::Board)
        .unwrap();
    let projection = crate::Projection::new(
        crate::inset_rect(pane.scene_viewport, 10.0, 10.0, 10.0, 10.0),
        &pane.bounds,
        camera,
    );
    let (x, y) = projection.project_point(state.scene.pads[0].center);
    state.ui.cursor_pos = Some(ScreenPointPx {
        x: x.floor() + 0.375,
        y: y.floor() + 0.625,
    });
    state.ui.crosshair_style = CrosshairStyle::FullViewport;
    assert!(
        pane.scene_viewport.contains(x, y),
        "selected real pad must be visible"
    );
    let prepare_changed = |c: &mut OffscreenRenderer| {
        c.renderer
            .prepare_session_workspace(
                &state,
                WorkspaceView {
                    source_revision: Some(source.revision()),
                    width: c.width,
                    height: c.height,
                    scale: 1.0,
                    camera,
                    schematic_camera: None,
                    include_preferences_overlay: false,
                    single_terminal_snapshot: true,
                    pane_cameras: &[],
                },
                &[],
            )
            .unwrap();
    };
    prepare_changed(&mut c);
    assert!(
        !c.renderer
            .render_session()
            .prepared()
            .unwrap()
            .board_interaction_vertices()
            .is_empty()
    );
    let expected = full_reference(&mut c);
    for (label, fault) in [
        ("stale preparation key", Fault::StaleKey),
        ("missing prefix copy", Fault::MissingCopy),
        (
            "premature resolve and single-pixel expansion",
            Fault::PrematureResolve,
        ),
    ] {
        set(fault);
        let wrong = frame(&mut c, false);
        let changed = wrong
            .as_raw()
            .iter()
            .zip(expected.as_raw())
            .filter(|(a, b)| a != b)
            .count();
        assert!(changed > 0, "oracle failed to reject {label}");
        eprintln!("negative rejected: {label}; differing channels={changed}");
        let recovered = exact(&mut c, "negative recovery full8x", false);
        assert!(recovered.as_raw() == expected.as_raw());
    }
    let lower = Renderer::new(&c.device, &c.queue, OUTPUT_FORMAT, 4).unwrap();
    c.renderer = lower;
    prepare_changed(&mut c);
    let reduced = full_reference(&mut c);
    let changed = reduced
        .as_raw()
        .iter()
        .zip(expected.as_raw())
        .filter(|(a, b)| a != b)
        .count();
    assert!(changed > 0, "oracle failed to reject reduced samples");
    eprintln!("negative rejected: reduced samples 4x; differing channels={changed}");
}

#[test]
#[ignore = "P630 Console occlusion regression and exact8x full/split painter proof"]
fn p630_exact8_console_covers_canvas_text_without_hiding_foreground() {
    use datum_gui_protocol::{ConsoleFeedbackDraft, ConsoleFeedbackSource};
    let mut c = reference_capture8();
    c.width = 960;
    c.height = 720;
    let mut state = doa();
    state.selection = datum_gui_protocol::SelectionTarget::AuthoredObject(
        state
            .scene
            .pads
            .iter()
            .find(|pad| pad.net_uuid.is_some())
            .unwrap()
            .object_id
            .clone(),
    );
    state.ui.console.publish(ConsoleFeedbackDraft::action_echo(
        ConsoleFeedbackSource::Viewport,
        42,
        "Board review ready",
    ));
    state.ui.console.set_history_expanded(true);
    let source = SourceEpoch::default();
    prepare(&mut c, &state, &source);
    // Pan an existing board label underneath history so the occlusion oracle
    // cannot pass vacuously when this fixture's initial camera misses the card.
    let initial = c.renderer.render_session().prepared().unwrap();
    let history = initial
        .console_overlay_layout()
        .unwrap()
        .history_panel
        .unwrap();
    let pane = initial
        .surface_passes()
        .iter()
        .find(|p| p.surface == crate::SceneSurface::Board)
        .unwrap();
    let label = initial
        .text_runs
        .iter()
        .find(|run| {
            run.origin == crate::TextOrigin::Viewport(pane.pane_id)
                && run.layer == crate::TextLayer::Workspace
                && pane.scene_viewport.contains(run.x, run.y)
        })
        .expect("visible real board label");
    let mut camera = pane.camera;
    let projection = crate::Projection::new(
        crate::inset_rect(pane.scene_viewport, 10.0, 10.0, 10.0, 10.0),
        &pane.bounds,
        camera,
    );
    camera.center_x_nm += (label.x - history.x - 20.0) / projection.scale;
    camera.center_y_nm += (label.y - history.y - history.height * 0.6) / projection.scale;
    c.renderer
        .prepare_session_workspace(
            &state,
            WorkspaceView {
                source_revision: Some(source.revision()),
                width: c.width,
                height: c.height,
                scale: 1.0,
                camera,
                schematic_camera: None,
                include_preferences_overlay: false,
                single_terminal_snapshot: true,
                pane_cameras: &[],
            },
            &[],
        )
        .unwrap();
    let correct = exact(&mut c, "Console cold painter", false);
    assert!(exact(&mut c, "Console warm painter", true).as_raw() == correct.as_raw());
    r3::motion(&mut c, "Console and text masked pointer frames");
    let scene = c.renderer.render_session().prepared().unwrap().clone();
    let history = scene
        .console_overlay_layout()
        .unwrap()
        .history_panel
        .unwrap();
    let board = c.renderer.render_session().board().unwrap().clone();
    let schematic = c.renderer.render_session().schematic().cloned();
    let render = |c: &mut OffscreenRenderer, scene: &crate::PreparedScene| {
        let target = target(c);
        c.renderer
            .render(
                &c.device,
                &c.queue,
                &target.create_view(&Default::default()),
                scene,
                &board,
                schematic.as_ref(),
                c.width,
                c.height,
            )
            .unwrap();
        target.hold_submission(&c.queue);
        c.read_texture(&target).unwrap()
    };
    let mut without_canvas_text = scene.clone();
    without_canvas_text.text_runs.retain(|run| {
        !(matches!(run.origin, crate::TextOrigin::Viewport(_))
            && run.layer == crate::TextLayer::Workspace)
    });
    assert!(without_canvas_text.text_runs.len() < scene.text_runs.len());
    let hidden = render(&mut c, &without_canvas_text);
    let mut old_order = scene.clone();
    for run in &mut old_order.text_runs {
        run.layer = crate::TextLayer::Foreground;
    }
    let wrong = render(&mut c, &old_order);
    let mut wrong_pixels = 0;
    for y in (history.y.ceil() as u32 + 2)..((history.y + history.height).floor() as u32 - 2) {
        for x in (history.x.ceil() as u32 + 2)..((history.x + history.width).floor() as u32 - 2) {
            assert_eq!(
                correct.get_pixel(x, y),
                hidden.get_pixel(x, y),
                "canvas text leaked at {x},{y}"
            );
            wrong_pixels += usize::from(correct.get_pixel(x, y) != wrong.get_pixel(x, y));
        }
    }
    assert!(
        wrong_pixels > 0,
        "old painter order negative must expose covered canvas labels"
    );
    eprintln!("Console old-order negative differs at {wrong_pixels} interior pixels");
    if let Some(path) = std::env::var_os("DATUM_CONSOLE_CAPTURE_OUT") {
        correct.save(path).unwrap();
    }
}

#[test]
#[ignore = "r3 single P630 exact-output batch; run only after complete offline measurement controls"]
fn r3_p630_exact_output_batch() {
    {
        let (c, adapter) = reference_capture8_with_adapter();
        crate::renderer_state::damage::restore::sample_tests::prove(&c.device, &c.queue, &adapter);
    }
    r3::prove();
    p630_exact8_pointer_styles_across_board_and_schematic();
    p630_exact8_doa_hover_composition_and_recovery();
    p630_exact8_terminal_layers_and_equal_length_replacement();
    p630_exact8_console_covers_canvas_text_without_hiding_foreground();
    p630_exact8_rejects_stale_missing_copy_reduced_samples_and_premature_resolve();
}
