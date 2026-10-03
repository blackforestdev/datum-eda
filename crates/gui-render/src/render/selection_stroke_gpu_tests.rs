//! Construction/pixel proof, distinct from native S5A acceptance.
use super::*;
use crate::RectPx;
#[test]
#[ignore = "requires local hardware GPU: physical capsule footprint and upload lifetime"]
fn selection_capsule_pixels_preserve_material_clip_and_two_pixel_boundary() {
    let state = crate::global_preferences_dialog_tests::state_with_preferences_open();
    let mut prepared =
        PreparedScene::from_native_preferences(&state.ui.global_preferences, 960, 720, 1.0)
            .unwrap();
    prepared.menu_overlay_vertices.clear();
    prepared.menu_overlay_text_runs.clear();
    // Exercise the production suffix through an ordinary frame descriptor.
    prepared.scene_viewport = RectPx {
        x: 0.0,
        y: 0.0,
        width: 960.0,
        height: 720.0,
    };
    prepared.selection_strokes = vec![crate::selection_stroke::Stroke {
        from: [100.0, 100.0],
        to: [300.0, 100.0],
        material_width: [0.4, 0.6, 0.2, 20.0],
        clip: [110.0, 80.0, 290.0, 120.0],
        high_contrast: [1.0, 0.0, 0.0, 0.0],
    }];
    let mut host = hardware_renderer(960, 720);
    let image = capture(&mut host, &prepared);
    assert_eq!(image.get_pixel(200, 100).0[..3], [110, 165, 55]);
    for y in [88, 89, 110, 111] {
        assert_eq!(image.get_pixel(200, y).0[..3], [206, 90, 146]);
    }
    assert_ne!(image.get_pixel(200, 87).0[..3], [206, 90, 146]);
    assert_ne!(image.get_pixel(200, 112).0[..3], [206, 90, 146]);
    assert_ne!(image.get_pixel(109, 100).0[..3], [110, 165, 55]);
    assert!(host.renderer.selection_stroke_gpu.last_upload_bytes > 0);
    let reserved = host.renderer.screen_budget.used();
    assert_eq!(image, capture(&mut host, &prepared));
    assert_eq!(host.renderer.selection_stroke_gpu.last_upload_bytes, 0);
    assert_eq!(host.renderer.screen_budget.used(), reserved);
    // Reused stream is unchanged; no selection bytes enter authored world buffers.
    let snapshot = host.renderer.screen_upload_snapshot_bytes();
    assert_eq!(image, capture(&mut host, &prepared));
    assert_eq!(snapshot, host.renderer.screen_upload_snapshot_bytes());
    prepared.selection_strokes[0].high_contrast[0] = 0.0;
    let ordinary = capture(&mut host, &prepared);
    assert_ne!(ordinary.get_pixel(200, 90), image.get_pixel(200, 90));
    assert_eq!(ordinary.get_pixel(200, 100), image.get_pixel(200, 100));
    // Minimum ring/X modes use the same production stream, never the lift
    // pass. Compare their authored center against the actual baseline image.
    prepared.selection_strokes.clear();
    let baseline = capture(&mut host, &prepared);
    prepared
        .selection_strokes
        .push(crate::selection_stroke::Stroke::minimum(
            [200.0, 100.0],
            [200.0, 100.0],
            prepared.scene_viewport,
            1.0,
        ));
    let ring = capture(&mut host, &prepared);
    assert_eq!(ring.get_pixel(200, 100), baseline.get_pixel(200, 100));
    assert_eq!(ring.get_pixel(200, 97).0[..3], [206, 90, 146]);
    assert_eq!(ring.get_pixel(200, 102).0[..3], [206, 90, 146]);
    assert_eq!(ring.get_pixel(200, 95), baseline.get_pixel(200, 95));
    prepared.selection_strokes = vec![
        crate::selection_stroke::Stroke::minimum(
            [197.0, 97.0],
            [203.0, 103.0],
            prepared.scene_viewport,
            2.0,
        ),
        crate::selection_stroke::Stroke::minimum(
            [197.0, 103.0],
            [203.0, 97.0],
            prepared.scene_viewport,
            2.0,
        ),
    ];
    let cross = capture(&mut host, &prepared);
    assert_eq!(cross.get_pixel(200, 100).0[..3], [206, 90, 146]);
    assert_eq!(cross.get_pixel(200, 96), baseline.get_pixel(200, 96));
    prepared.selection_strokes.clear();
    capture(&mut host, &prepared);
    assert!(host.renderer.selection_stroke_gpu.buffer().is_none());
}
