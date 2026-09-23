//! Rich paint changes reuse real shaped layouts while updating GPU output.
use super::*;

#[test]
#[ignore = "requires local GPU; rich paint cache reuse and independent cold pixels"]
fn rich_color_changes_reuse_shapes_and_update_each_area() {
    let state = crate::global_preferences_dialog_tests::state_with_preferences_open();
    let mut renderer = hardware_renderer(960, 720);
    let mut fresh = hardware_renderer(960, 720);
    let mut prepared =
        PreparedScene::from_native_preferences(&state.ui.global_preferences, 960, 720, 1.0);
    let mut label = prepared.menu_overlay_text_runs[0].clone();
    label.x = 30.0;
    label.y = 30.0;
    label.layout_size = Some((300.0, 160.0));
    label.clip_bounds = None;
    label.text = "Rich paint".into();
    label.rich_spans = vec![
        crate::TextRunSpan {
            text: "Bold Ω\n".into(),
            color: [1.0, 0.2, 0.1],
            bold: true,
            italic: false,
        },
        crate::TextRunSpan {
            text: "Italic العربية".into(),
            color: [0.1, 0.8, 1.0],
            bold: false,
            italic: true,
        },
    ];
    let mut other = label.clone();
    other.y += 180.0;
    other.rich_spans[0].color = [0.1, 1.0, 0.2];
    prepared.menu_overlay_text_runs = vec![label, other];
    let original = capture(&mut renderer, &prepared);
    assert_eq!(
        renderer.renderer.text_buffers.entries().len(),
        1,
        "simultaneous colors share one layout"
    );
    let revision = renderer.renderer.text_buffers.revision();
    let prepares = renderer.renderer.text_preparation.overlay_prepares;
    prepared.menu_overlay_text_runs[0].rich_spans[0].color = [0.2, 0.1, 1.0];
    prepared.menu_overlay_text_runs[1].rich_spans[1].color = [1.0, 0.1, 0.2];
    let changed = capture(&mut renderer, &prepared);
    assert!(
        original != changed,
        "paint signature must refresh glyph instances"
    );
    assert!(changed == capture(&mut fresh, &prepared));
    assert_eq!(renderer.renderer.text_buffers.revision(), revision);
    assert_eq!(
        renderer.renderer.text_preparation.overlay_prepares,
        prepares + 1
    );
    assert!(changed == capture(&mut renderer, &prepared));
    assert_eq!(
        renderer.renderer.text_preparation.overlay_prepares,
        prepares + 1
    );
}
