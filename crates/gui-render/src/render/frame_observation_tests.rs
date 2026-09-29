use super::*;
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
        assert_eq!(draws[0].scissor, [0, 0, 960, 720]);
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
    gpu.renderer.screen_admission.set(Some([None; 7]));
    for _ in 0..2 {
        gpu.renderer.observe_screen_draw(
            crate::immediate_admission::screen_admission::ScreenGroup::Menu,
            6,
            [0, 0, 960, 720],
        );
    }
    assert!(
        gpu.renderer.encoded_screen_geometry().is_none(),
        "duplicate stream records invalidate observation"
    );
}
