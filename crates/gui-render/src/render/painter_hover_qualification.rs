//! Regional proof preparation. Do not execute until all r4 group prerequisites are complete.
use super::*;
use crate::gpu_surface::prefix_negative_control::{Fault, set};

fn native_doa() -> ReviewWorkspaceState {
    let project = std::path::PathBuf::from(std::env::var_os("DATUM_NATIVE_TEST_PROJECT").unwrap());
    let state = datum_gui_protocol::load_board_editor_workspace_state(
        &datum_gui_protocol::LiveReviewRequest {
            project_root: project,
            board_file: None,
            artifact_path: None,
            net_uuid: None,
            from_anchor_pad_uuid: None,
            to_anchor_pad_uuid: None,
            profile: None,
            kicad_board_source: None,
        },
    )
    .unwrap();
    assert_eq!(state.scene.pads.len(), 81);
    state
}

fn exact_samples(c: &mut OffscreenRenderer, previous: &PreparedScene, label: &str) {
    let plan = c
        .renderer
        .render_session()
        .regional_plan_for_proof(previous, [c.width, c.height]);
    exact(c, label, true);
    c.renderer
        .assert_regional_samples(&c.device, &c.queue, [c.width, c.height], &plan, label);
}

fn reject(c: &mut OffscreenRenderer, fault: Fault) {
    r3::pointer(c, Some((0.23, 0.31)));
    exact(c, "negative initial support", true);
    // Cross an actual glyph row, including the support guard outside the
    // opaque crosshair, so duplicate alpha painting cannot pass vacuously.
    let scene = c.renderer.render_session().prepared().unwrap();
    let pane = scene
        .surface_passes()
        .iter()
        .find(|p| p.surface == crate::SceneSurface::Board)
        .unwrap();
    let run = scene
        .text_runs
        .iter()
        .find(|r| {
            r.origin == crate::TextOrigin::Viewport(pane.pane_id)
                && r.layer == crate::TextLayer::Workspace
                && pane.scene_viewport.contains(r.x, r.y)
        })
        .expect("visible actual board glyphs");
    let cursor = ScreenPointPx {
        x: pane.scene_viewport.x + pane.scene_viewport.width * 0.67,
        y: run.y + run.size * 0.5 + 0.375,
    };
    let session = c.renderer.render_session_mut();
    assert!(session.update_pointer(PointerUpdate {
        generation: session.pointer_generation(),
        cursor: Some(cursor),
        hover: None,
        style: CrosshairStyle::FullViewport,
    }));
    let reference = full_reference(c);
    set(fault);
    let wrong = frame(c, false);
    assert!(
        wrong.as_raw() != reference.as_raw(),
        "negative must change pixels"
    );
    exact(c, "failed presentation reconstruction", false);
}

fn crossings() {
    let mut c = reference_capture8();
    c.renderer.surface_attachments.enable_sample_readback();
    c.width = 1280;
    c.height = 800;
    let mut state = native_doa();
    state.selection = datum_gui_protocol::SelectionTarget::None;
    let source = SourceEpoch::default();
    prepare(&mut c, &state, &source);
    exact(&mut c, "crossing baseline", false);
    let previous = c.renderer.render_session().prepared().unwrap().clone();
    r3::pointer(&mut c, Some((0.3, 0.4)));
    exact_samples(&mut c, &previous, "fractional crosshair enter");
    let previous = c.renderer.render_session().prepared().unwrap().clone();
    r3::pointer(&mut c, Some((0.7, 0.6)));
    exact_samples(&mut c, &previous, "old/new crossing alpha");
    reject(&mut c, Fault::OverlappingSuffix);
    reject(&mut c, Fault::MissingSuffixMask);
    let plan = crate::renderer_state::damage::regional::Plan::new(
        [c.width, c.height],
        &[[96, 64, 608, 576]],
    )
    .unwrap();
    assert_eq!(plan.tiles().count(), 256);
    c.renderer
        .regional_repaint_for_proof(&c.device, &c.queue, [c.width, c.height], &plan);
    c.renderer.assert_regional_samples(
        &c.device,
        &c.queue,
        [c.width, c.height],
        &plan,
        "all256cells including first/last",
    );
    exact(&mut c, "full-capacity composed and presented bytes", true);
    c.width = 1277;
    c.height = 797;
    prepare(&mut c, &state, &source);
    exact(&mut c, "partial edge extent cold reconstruction", false);
    let plan = crate::renderer_state::damage::regional::Plan::new(
        [c.width, c.height],
        &[[1270, 790, 1277, 797]],
    )
    .unwrap();
    c.renderer
        .regional_repaint_for_proof(&c.device, &c.queue, [c.width, c.height], &plan);
    c.renderer.assert_regional_samples(
        &c.device,
        &c.queue,
        [c.width, c.height],
        &plan,
        "edge padding initialized",
    );
    exact(&mut c, "partial-edge composed and presented bytes", true);
}

fn hover_dependencies() {
    let mut c = reference_capture8();
    c.width = 1280;
    c.height = 800;
    let mut state = native_doa();
    state.selection = datum_gui_protocol::SelectionTarget::None;
    let source = SourceEpoch::default();
    prepare(&mut c, &state, &source);
    exact(&mut c, "hover baseline", false);
    let uploads = world_uploads(&c);
    let archive = include_str!(
        "../../../../docs/reviews/gui-performance/gpu-redraw-proposal/timed-native-result/cold-demands.json"
    );
    let ids: Vec<_> = archive
        .lines()
        .filter_map(|line| {
            line.trim()
                .strip_prefix("\"after\": \"")
                .and_then(|v| v.strip_suffix("\","))
        })
        .collect();
    assert_eq!(ids.len(), 14);
    for (ordinal, id) in ids.into_iter().enumerate() {
        state.ui.hovered_object = (!id.is_empty()).then(|| HoverTarget {
            object_id: id.into(),
            surface: PaneContent::Board,
        });
        let session = c.renderer.render_session_mut();
        assert!(session.update_pointer(PointerUpdate {
            generation: session.pointer_generation(),
            cursor: None,
            hover: state.ui.hovered_object.as_ref(),
            style: CrosshairStyle::FullViewport,
        }));
        if ordinal == 0 {
            exact(&mut c, "hover ring alpha", true);
        } else {
            exact(&mut c, "archived non-pad hover", true);
        }
        assert_eq!(world_uploads(&c), uploads);
    }
    for pad in [Some(0), Some(1), None] {
        state.ui.hovered_object = pad.map(|i| HoverTarget {
            object_id: state.scene.pads[i].object_id.clone(),
            surface: PaneContent::Board,
        });
        let session = c.renderer.render_session_mut();
        assert!(!session.update_pointer(PointerUpdate {
            generation: session.pointer_generation(),
            cursor: None,
            hover: state.ui.hovered_object.as_ref(),
            style: CrosshairStyle::Local,
        }));
        prepare(&mut c, &state, &source);
        exact(&mut c, "pad material/text change stays cold", false);
        exact(&mut c, "stable material dependency", true);
    }
}

fn timing_paths() {
    let mut c = reference_capture8();
    c.width = 1280;
    c.height = 800;
    let state = crate::gpu_surface_pass::board_fixture_state();
    prepare(&mut c, &state, &SourceEpoch::default());
    frame(&mut c, true);
    c.renderer
        .enable_gpu_measurements(
            &c.device,
            &c.queue,
            1,
            1,
            Box::new(|r| panic!("unexpected regional timer cancellation: {r:?}")),
        )
        .unwrap();
    // Exercise the actual bounded continuation marker encoders five times.
    // These draw-free submissions test query capacity, not transfer performance.
    for _ in 0..5 {
        let (leading, trailing) = c
            .renderer
            .begin_upload_measurement(&c.device, "world")
            .unwrap()
            .unwrap();
        c.queue.submit([leading, trailing]);
        c.renderer.finish_upload_measurement(&c.queue).unwrap();
        c.device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
    }
    c.renderer.render_session.prefix = Default::default();
    frame(&mut c, true);
    let samples = c.renderer.poll_gpu_measurements(&c.device).unwrap();
    assert_eq!(samples.len(), 1);
    let sample = &samples[0];
    assert_eq!(sample.submission_manifest.len(), 6);
    assert!(sample.scene_marker_ticks.is_some());
    assert_eq!(
        sample.raw_ticks.len() + 3,
        31,
        "actual cold encoder query use"
    );
    assert_eq!(sample.passes_ns.last().unwrap().0, "frame-trailing");
    assert!(sample.frame_span_ns >= sample.own_pass_sum_ns);
    assert_eq!(
        sample.submission_manifest.last().unwrap().last_tick,
        *sample.raw_ticks.last().unwrap()
    );
    for (label, expected) in [
        ("empty", vec!["upload-leading", "frame-trailing"]),
        (
            "warm",
            vec!["upload-leading", "restore", "suffix", "frame-trailing"],
        ),
        (
            "fallback",
            vec!["upload-leading", "frame", "frame-trailing"],
        ),
    ] {
        if label == "warm" {
            r3::pointer(&mut c, Some((0.4, 0.5)));
        }
        let restoration = if label == "fallback" {
            c.renderer.damage_masks.restoration.take()
        } else {
            None
        };
        frame(&mut c, true);
        if let Some(restoration) = restoration {
            c.renderer.damage_masks.restoration = Some(restoration);
        }
        let samples = c.renderer.poll_gpu_measurements(&c.device).unwrap();
        assert_eq!(samples.len(), 1);
        let sample = &samples[0];
        assert_eq!(
            sample.passes_ns.iter().map(|p| p.0).collect::<Vec<_>>(),
            expected,
            "{label}"
        );
        assert!(sample.frame_span_ns >= sample.own_pass_sum_ns);
        assert_eq!(
            sample.submission_manifest.last().unwrap().last_tick,
            *sample.raw_ticks.last().unwrap()
        );
        eprintln!("r4 actual timing graph passed: {label}");
    }
    eprintln!("r4 maximum cold query use31/32; normal production usages and final markers passed");
}

#[test]
#[ignore = "one approved four-group GPU batch, first failure stops; no native timing trial"]
fn regional_p630_four_group_batch() {
    {
        let (c, _) = reference_capture8_with_adapter();
        crate::renderer_state::damage::restore::sample_tests::prove_regional(&c.device, &c.queue);
    }
    eprintln!("GROUP 1: crossing alpha, exact samples and overlapping/missing restrictions");
    crossings();
    eprintln!("GROUP 2: Console, text, menu and terminal painter clips");
    p630_exact8_terminal_layers_and_equal_length_replacement();
    p630_exact8_console_covers_canvas_text_without_hiding_foreground();
    eprintln!("GROUP 3: archived non-pad and actual pad material/text dependencies");
    hover_dependencies();
    eprintln!("GROUP 4: stale/failed support, admission refusal and submitted retirement");
    r3::prove();
    pair_generation_allowance_survives_replacement_and_device_recovery();
    required_refusal_defers_once_then_propagates_without_recreating_prefix();
    timing_paths();
    let mut c = reference_capture8();
    let state = crate::gpu_surface_pass::board_fixture_state();
    prepare(&mut c, &state, &SourceEpoch::default());
    exact(&mut c, "overflow baseline", false);
    r3::pointer(&mut c, Some((0.4, 0.5)));
    set(Fault::DamageOverflow);
    exact(&mut c, "partition overflow full fallback", false);
    set(Fault::TileOverflow);
    exact(&mut c, "tile overflow cold reconstruction", false);
    eprintln!("FOUR GROUPS PASSED; pointer performance and final qualification remain separate");
}
