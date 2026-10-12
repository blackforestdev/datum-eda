use super::*;
use crate::{Output, Quad, RectPx, quads_to_vertices};
use std::sync::{
    Mutex,
    atomic::{AtomicBool, Ordering},
};
static OBSERVED: Mutex<Vec<(u64, Option<bool>, bool)>> = Mutex::new(Vec::new());
static FAIL: AtomicBool = AtomicBool::new(false);
fn observe(frame: crate::resource_observation::FrameObservation<'_>) -> anyhow::Result<()> {
    assert_eq!(frame.extent, [960, 720]);
    assert_eq!(
        frame
            .prepared
            .geometry_admission(frame.board, frame.schematic)
            .count(),
        0
    );
    if frame.submitted_frame == Some(true) {
        let draws: Vec<_> = frame
            .renderer
            .encoded_screen_geometry()
            .unwrap()
            .into_iter()
            .flatten()
            .collect();
        assert_eq!(draws.len(), 1);
        assert_eq!(draws[0].group, "menu_overlay");
        assert_eq!(draws[0].scissor, Some([0, 0, 960, 720]));
        assert_eq!(draws[0].commands, 1);
        assert_eq!(draws[0].range, Some([0, draws[0].vertices as u32]));
        assert_eq!(
            draws[0].vertices as usize,
            frame.prepared.screen_geometry_admission()[1].vertices
        );
    }
    OBSERVED.lock().unwrap().push((
        frame.renderer.resource_owner_id(),
        frame.submitted_frame,
        frame.text_observation_attempted,
    ));
    anyhow::ensure!(
        !FAIL.load(Ordering::Acquire),
        "injected observer delivery failure"
    );
    Ok(())
}
#[test]
#[ignore = "requires local GPU; render observation delivery and pixel preservation"]
fn frame_observation_delivers_reused_frames_without_changing_pixels_and_propagates_failure() {
    struct Reset;
    impl Drop for Reset {
        fn drop(&mut self) {
            FAIL.store(false, Ordering::Release);
        }
    }
    let _reset = Reset;
    OBSERVED.lock().unwrap().clear();
    let state = crate::global_preferences_dialog_tests::state_with_preferences_open();
    let prepared =
        PreparedScene::from_native_preferences(&state.ui.global_preferences, 960, 720, 1.0)
            .unwrap();
    let mut gpu = hardware_renderer(960, 720);
    let reference = capture(&mut gpu, &prepared);
    gpu.renderer.text_admission = crate::text_admission_observation::Observer::new(true);
    gpu.renderer.frame_observer = Some(observe);
    assert!(reference == capture(&mut gpu, &prepared));
    assert!(reference == capture(&mut gpu, &prepared));
    {
        let rows = OBSERVED.lock().unwrap();
        assert_eq!(rows.len(), 2);
        assert!(rows.iter().all(
            |(id, outcome, text)| *id == gpu.renderer.resource_owner_id()
                && *outcome == Some(true)
                && *text
        ));
    }
    FAIL.store(true, Ordering::Release);
    let target =
        crate::capture_resource::CaptureTarget::new(&gpu.device, gpu.extent(), OUTPUT_FORMAT)
            .unwrap();
    let result = gpu.renderer.render(
        &gpu.device,
        &gpu.queue,
        target.create_view(&wgpu::TextureViewDescriptor::default()),
        &prepared,
        &RetainedScene::empty(),
        None,
        960,
        720,
    );
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("injected observer delivery failure")
    );
    assert!(
        gpu.renderer.encoded_screen_geometry().is_none(),
        "records retire after callback even on delivery error"
    );
    gpu.renderer.screen_admission.set(Some([None; 8]));
    for _ in 0..2 {
        gpu.renderer.observe_screen_draw(
            crate::immediate_admission::screen_admission::ScreenGroup::Menu,
            6,
            [0, 0, 960, 720],
        );
    }
    let draws = gpu.renderer.encoded_screen_geometry().unwrap();
    let menu = draws[1].unwrap();
    assert_eq!(menu.commands, 2);
    assert_eq!(menu.vertices, 12);
    assert_eq!(menu.scissor, Some([0, 0, 960, 720]));
    assert_eq!(menu.range, None);
    gpu.renderer.observe_screen_range(
        crate::immediate_admission::screen_admission::ScreenGroup::Menu,
        12..18,
        [0, 96, 960, 96],
    );
    let menu = gpu.renderer.encoded_screen_geometry().unwrap()[1].unwrap();
    assert_eq!(menu.vertices, 18);
    assert_eq!(
        menu.scissor, None,
        "multiple scissors are not a coverage rectangle"
    );
    let mut draws = [None; 8];
    draws[1] = Some(crate::resource_observation::EncodedScreenGeometry {
        commands: u64::MAX,
        ..menu
    });
    gpu.renderer.screen_admission.set(Some(draws));
    gpu.renderer.observe_screen_draw(
        crate::immediate_admission::screen_admission::ScreenGroup::Menu,
        6,
        [0, 0, 960, 720],
    );
    assert!(
        gpu.renderer.encoded_screen_geometry().is_none(),
        "checked command overflow invalidates observation"
    );
    let mut draws = [None; 8];
    draws[1] = Some(crate::resource_observation::EncodedScreenGeometry {
        vertices: u64::MAX,
        commands: 1,
        ..menu
    });
    gpu.renderer.screen_admission.set(Some(draws));
    gpu.renderer.observe_screen_draw(
        crate::immediate_admission::screen_admission::ScreenGroup::Menu,
        6,
        [0, 0, 960, 720],
    );
    assert!(
        gpu.renderer.encoded_screen_geometry().is_none(),
        "checked vertex overflow invalidates observation"
    );
}

static BANDED: Mutex<Vec<[Option<crate::resource_observation::EncodedScreenGeometry>; 8]>> =
    Mutex::new(Vec::new());
fn observe_banded(frame: crate::resource_observation::FrameObservation<'_>) -> anyhow::Result<()> {
    if frame.submitted_frame == Some(true) {
        BANDED.lock().unwrap().push(
            frame
                .renderer
                .encoded_screen_geometry()
                .expect("banded actual draws remain valid"),
        );
    }
    Ok(())
}
#[test]
#[ignore = "requires local GPU; actual mask-band commands, suppression, pixel and retirement parity"]
fn frame_observation_preserves_banded_selection_and_excludes_suppressed_overlay_draws() {
    let (_fixture, state) = crate::native_selection_overlay::tests::selected_track_state(2_000_000);
    let retained = RetainedScene::from_workspace(&state, 960, 720);
    let camera = crate::CameraState::fit_to_bounds(&state.scene.bounds);
    let mut prepared = PreparedScene::from_workspace(&state, 960, 720, camera, &retained).unwrap();
    prepared.append_native_selection(&state).unwrap();
    assert_eq!(prepared.selection_masks.len(), 1);
    let pane = prepared.selection_masks[0].0;
    let viewport = prepared
        .surface_passes()
        .iter()
        .find(|p| p.pane_id == pane)
        .unwrap()
        .scene_viewport;
    prepared
        .selection_strokes
        .push(crate::selection_stroke::Stroke::minimum(
            [viewport.x + 40.0, viewport.y + 120.0],
            [viewport.x + 50.0, viewport.y + 120.0],
            viewport,
            0.0,
        ));
    let mut plan =
        crate::native_selection_overlay::cue_plan::PanePlan::new(&state, viewport, camera).unwrap();
    let mut quads = Vec::new();
    let shape = RectPx {
        x: viewport.x + 20.0,
        y: viewport.y + 20.0,
        width: 12.0,
        height: 12.0,
    };
    plan.base_output(&mut quads, true)
        .push(Quad::from_rect(shape, crate::PROPOSAL_BASE));
    let start = quads.len();
    quads.extend(std::iter::repeat_n(
        Quad::from_rect(shape, crate::TEXT_ACCENT),
        65_537,
    ));
    plan.finish_detail(&mut quads, start);
    assert_eq!(plan.detail_primitives, 65_537);
    let offset = prepared.viewport_overlay_vertices.len() as u32;
    for span in plan.screen.iter_mut() {
        span.range.start += offset;
        span.range.end += offset;
    }
    for range in plan.detail.iter_mut() {
        range.start += offset;
        range.end += offset;
    }
    prepared
        .pane_cues
        .push((pane, std::sync::Arc::new(plan.freeze().unwrap())));
    prepared.selection_masks[0]
        .2
        .set_screen(
            prepared
                .pane_cues
                .iter()
                .filter(|(id, _)| *id == pane)
                .map(|(_, plan)| plan.clone()),
        )
        .unwrap();
    prepared
        .viewport_overlay_vertices
        .extend(quads_to_vertices(&quads));
    let mut gpu = hardware_renderer(960, 720);
    let expected = capture_retained(&mut gpu, &prepared, &retained);
    assert!(gpu.renderer.selection_mask.detailed_overflow(pane));
    BANDED.lock().unwrap().clear();
    gpu.renderer.frame_observer = Some(observe_banded);
    assert_eq!(capture_retained(&mut gpu, &prepared, &retained), expected);
    let rows = BANDED.lock().unwrap();
    assert_eq!(rows.len(), 1);
    let selection = rows[0][7].unwrap();
    assert!(
        selection.commands > 2,
        "lift and cue execute in multiple bands"
    );
    assert_eq!(selection.commands % 2, 0);
    assert_eq!(selection.vertices, selection.commands * 6);
    assert_eq!(selection.range, None);
    assert_eq!(selection.scissor, None);
    let overlay = rows[0][3].unwrap();
    assert!(overlay.commands > 1);
    // Preserve the observed warm image and valid repeated-draw summary. Its
    // regional tile execution is not inferred from a whole-payload multiplier.
    println!(
        "warm actual overlay commands={} vertices={}",
        overlay.commands, overlay.vertices
    );
    assert_eq!(overlay.range, None);
    assert_eq!(overlay.scissor, None);
    drop(rows);
    // A separate cold full frame has a source-derived exact band oracle, with
    // the identical producer, full base and all 65,537 suppressed detail quads.
    assert_eq!(prepared.selection_overlay_split as u32, offset);
    let detail: Vec<_> = prepared
        .pane_cues
        .iter()
        .filter(|(id, _)| *id == pane)
        .flat_map(|(_, plan)| plan.detail.iter().cloned())
        .collect();
    assert_eq!(
        detail,
        vec![offset + 6..prepared.viewport_overlay_vertices.len() as u32]
    );
    let band = crate::selection_stroke::mask::BAND;
    let bands = 720_u32.div_ceil(band);
    let top = prepared.scene_viewport.y.max(0.0).floor() as u32;
    let bottom = top + prepared.scene_viewport.height.max(1.0).ceil() as u32;
    let prefix_bands = (0..bands)
        .filter(|b| b * band < bottom && (b + 1) * band > top)
        .count() as u64;
    let mut cold = hardware_renderer(960, 720);
    cold.renderer.frame_observer = Some(observe_banded);
    assert_eq!(capture_retained(&mut cold, &prepared, &retained), expected);
    let rows = BANDED.lock().unwrap();
    assert_eq!(rows.len(), 2);
    let overlay = rows[1][3].unwrap();
    assert_eq!(overlay.commands, prefix_bands + u64::from(bands));
    assert_eq!(
        overlay.vertices,
        prefix_bands * u64::from(offset) + u64::from(bands) * 6,
        "prefix clips and base suffix only: all detail vertices remain suppressed"
    );
    let selection = rows[1][7].unwrap();
    assert_eq!(selection.commands, u64::from(bands) * 2);
    assert_eq!(
        selection.vertices,
        selection.commands * prepared.selection_strokes.len() as u64 * 6
    );
    drop(rows);
    let cold_observer = cold.renderer.local_reservation_observer();
    cold.device
        .poll(wgpu::PollType::wait_indefinitely())
        .unwrap();
    drop(cold.renderer);
    cold.device
        .poll(wgpu::PollType::wait_indefinitely())
        .unwrap();
    assert!(cold_observer.usage().released());
    let observer = gpu.renderer.local_reservation_observer();
    gpu.device
        .poll(wgpu::PollType::wait_indefinitely())
        .unwrap();
    drop(gpu.renderer);
    gpu.device
        .poll(wgpu::PollType::wait_indefinitely())
        .unwrap();
    assert!(observer.usage().released());
}
