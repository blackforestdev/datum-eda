//! Retained control geometry parity across live placement and clipping changes.
use super::*;
use crate::global_preferences_primitives::{ControlMeshCache, ControlPainter};

#[test]
#[ignore = "requires local GPU; run explicitly with the visual feature"]
fn rounded_control_fans_match_scanline_pixels() {
    let state = crate::global_preferences_dialog_tests::state_with_preferences_open();
    let mut renderer = hardware_renderer(960, 720);
    let mut prepared =
        PreparedScene::from_native_preferences(&state.ui.global_preferences, 960, 720, 1.0);
    prepared.menu_overlay_text_runs.clear();
    let mut cache = ControlMeshCache::default();
    for dpi in [1.0, 1.5, 2.0] {
        for (width, height, radius) in [
            (140.0, 28.0, 4.0),
            (24.0, 24.0, 12.0),
            (37.5, 19.5, 3.0),
            (200.0, 50.0, 0.0),
        ] {
            let before = cache.builds;
            for offset in [0.0, 0.25, 0.5] {
                let rect = crate::RectPx {
                    x: 40.0 + offset,
                    y: 40.0 + offset,
                    width,
                    height,
                };
                for clipped in [false, true] {
                    let mut expected_quads = Vec::new();
                    crate::push_projected_polygon_fill(
                        &mut expected_quads,
                        &crate::global_preferences_primitives::rounded_rect_points(rect, radius),
                        [0.3, 0.5, 0.7],
                    );
                    let mut actual_quads = Vec::new();
                    ControlPainter::new(&mut actual_quads, &mut cache, dpi).rounded_fill(
                        rect,
                        [0.3, 0.5, 0.7],
                        radius,
                        0.0,
                    );
                    assert_eq!(cache.builds, before + 1, "placement/clip must reuse mesh");
                    if clipped {
                        let clip = crate::RectPx {
                            x: rect.x + 2.25,
                            y: rect.y + 1.5,
                            width: rect.width - 4.75,
                            height: rect.height - 3.25,
                        };
                        for quads in [&mut expected_quads, &mut actual_quads] {
                            crate::hit_clipping::clip_content(
                                quads,
                                &mut Vec::new(),
                                &mut Vec::new(),
                                0,
                                0,
                                0,
                                clip,
                            );
                        }
                    }
                    prepared.menu_overlay_vertices =
                        crate::gpu_data::quads_to_vertices(&expected_quads);
                    let expected = capture(&mut renderer, &prepared);
                    prepared.menu_overlay_vertices =
                        crate::gpu_data::quads_to_vertices(&actual_quads);
                    let actual = capture(&mut renderer, &prepared);
                    assert!(
                        actual == expected,
                        "retained control changed: {rect:?}, radius={radius}, dpi={dpi}, clipped={clipped}"
                    );
                }
            }
        }
    }
    assert_eq!(
        cache.builds, 12,
        "each geometry/DPI combination builds once"
    );
}
