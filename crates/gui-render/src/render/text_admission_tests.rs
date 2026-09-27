//! Native-shaped text observation must preserve output and distinguish duplicates.
use super::*;

#[test]
#[ignore = "requires local GPU; text admission observer parity and refusal"]
fn text_admission_observes_reuse_deduplication_and_invalid_input_without_pixel_changes() {
    let state = crate::global_preferences_dialog_tests::state_with_preferences_open();
    let mut prepared =
        PreparedScene::from_native_preferences(&state.ui.global_preferences, 960, 720, 1.0)
            .unwrap();
    let mut gpu = hardware_renderer(960, 720);
    gpu.renderer.text_admission = crate::text_admission_observation::Observer::new(false);
    let reference = capture(&mut gpu, &prepared);
    assert!(gpu.renderer.text_admission_observation().is_none());
    gpu.renderer.text_admission = crate::text_admission_observation::Observer::new(true);
    assert!(reference == capture(&mut gpu, &prepared));
    let first = gpu.renderer.text_admission_observation().unwrap();
    assert_eq!(first.workspace.runs, 0);
    assert_eq!(first.overlay.runs, prepared.menu_overlay_text_runs.len());
    assert!(first.overlay.unique_raster_keys > 0);
    assert!(first.overlay.unique_raster_keys <= first.overlay.shaped_instances);
    assert_eq!(
        first.union_unique_raster_keys,
        first.overlay.unique_raster_keys
    );
    assert!(first.scratch_bytes > 0);
    let usage = gpu.renderer.text_admission_observer_usage().unwrap();
    assert!(usage.allocator_installed);
    assert_eq!(usage.payload_bytes, 0, "observer retains no key collection");
    assert!(usage.peak_payload_bytes > 0);
    assert!(reference == capture(&mut gpu, &prepared));
    let reused = gpu.renderer.text_admission_observation().unwrap();
    assert!(reused.preparation_serial > first.preparation_serial);
    assert_eq!(reused.cache_revision, first.cache_revision);
    assert_eq!(reused.overlay, first.overlay);

    let original = prepared.menu_overlay_text_runs.clone();
    prepared.menu_overlay_text_runs.extend(original.clone());
    capture(&mut gpu, &prepared);
    let doubled = gpu.renderer.text_admission_observation().unwrap();
    assert_eq!(
        doubled.overlay.shaped_instances,
        first.overlay.shaped_instances * 2
    );
    assert_eq!(
        doubled.overlay.unique_raster_keys,
        first.overlay.unique_raster_keys
    );
    prepared.menu_overlay_text_runs = original;
    gpu.renderer
        .observe_text_admission(&[], &[], &[], &prepared.menu_overlay_text_runs);
    assert!(gpu.renderer.text_admission_observation_failed());
    assert!(
        gpu.renderer.text_admission_observation().is_none(),
        "invalid indices cannot count as zero"
    );
    assert!(reference == capture(&mut gpu, &prepared));
    assert!(!gpu.renderer.text_admission_observation_failed());
    assert_eq!(
        gpu.renderer.text_admission_observation().unwrap().overlay,
        first.overlay
    );
}
