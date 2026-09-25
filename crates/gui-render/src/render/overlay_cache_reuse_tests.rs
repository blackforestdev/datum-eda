//! Overlay cache refusal and layout-reuse pixel parity.
use super::*;

#[test]
#[ignore = "requires local GPU; run explicitly with the visual feature"]
fn cached_shape_relayout_matches_fresh_dialog_pixels() {
    let state = crate::global_preferences_dialog_tests::state_with_preferences_open();
    let mut renderer = hardware_renderer(960, 720);
    let mut fresh = hardware_renderer(960, 720);
    let mut prepared =
        PreparedScene::from_native_preferences(&state.ui.global_preferences, 960, 720, 1.0)
            .unwrap();
    let mut label = prepared.menu_overlay_text_runs[0].clone();
    label.text = "Wrap widths preserve shaped text: µm and Ω".into();
    label.rich_spans.clear();
    label.x = 30.0;
    label.y = 30.0;
    prepared.menu_overlay_text_runs = vec![label];
    for (step, width) in [240.0, 100.0, 180.0, 100.0].into_iter().enumerate() {
        prepared.menu_overlay_text_runs[0].clip_bounds = Some(crate::RectPx {
            x: 30.0,
            y: 30.0,
            width,
            height: 150.0,
        });
        let actual = capture(&mut renderer, &prepared);
        fresh.renderer.text_buffers = Default::default();
        assert!(
            actual == capture(&mut fresh, &prepared),
            "cached shape differs at width {width}"
        );
        assert_eq!(renderer.renderer.text_buffers.shape_reuses, step);
        assert_eq!(renderer.renderer.text_buffers.entries().len(), 1);
    }
}

#[test]
#[ignore = "requires local GPU; bounded overlay signature proof"]
fn oversized_overlay_signature_bypasses_reuse_without_omitting_text() {
    let state = crate::global_preferences_dialog_tests::state_with_preferences_open();
    let mut renderer = hardware_renderer(960, 720);
    let mut prepared =
        PreparedScene::from_native_preferences(&state.ui.global_preferences, 960, 720, 1.0)
            .unwrap();
    prepared.menu_overlay_text_runs = vec![prepared.menu_overlay_text_runs[0].clone(); 129];
    let cold = capture(&mut renderer, &prepared);
    let prepares = renderer.renderer.text_preparation.overlay_prepares;
    assert!(cold == capture(&mut renderer, &prepared));
    assert_eq!(
        renderer.renderer.text_preparation.overlay_prepares,
        prepares + 1
    );
    let mut fresh = hardware_renderer(960, 720);
    assert!(cold == capture(&mut fresh, &prepared));
}
