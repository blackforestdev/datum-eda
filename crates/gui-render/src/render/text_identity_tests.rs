use super::*;
use crate::text_buffer_cache::text_buffer_key;
#[test]
fn text_buffer_key_ignores_position_and_color_but_tracks_content() {
    let base = TextRun {
        text: "PROJECT".to_string(),
        rich_spans: Vec::new(),
        x: 12.0,
        y: 24.0,
        size: 12.0,
        color: TEXT_PRIMARY,
        face: TextFace::Ui,
        clip_bounds: None,
        layout_size: None,
    };
    let mut moved = base.clone();
    moved.x += 100.0;
    moved.y += 50.0;
    moved.color = TEXT_SECONDARY;

    assert_eq!(
        text_buffer_key(&base, 1280, 768),
        text_buffer_key(&moved, 1280, 768)
    );

    let mut changed_text = base.clone();
    changed_text.text.push('!');
    assert_ne!(
        text_buffer_key(&base, 1280, 768),
        text_buffer_key(&changed_text, 1280, 768)
    );

    let mut changed_size = base.clone();
    changed_size.size = 13.0;
    assert_ne!(
        text_buffer_key(&base, 1280, 768),
        text_buffer_key(&changed_size, 1280, 768)
    );

    let mut clipped = base.clone();
    clipped.clip_bounds = Some(RectPx {
        x: 0.0,
        y: 0.0,
        width: 44.0,
        height: 18.0,
    });
    assert_ne!(
        text_buffer_key(&base, 1280, 768),
        text_buffer_key(&clipped, 1280, 768)
    );
}

#[test]
fn conformance_medium_type_tiers_resolve_to_medium_weight() {
    assert_eq!(design_tokens::typography::STRONG_WEIGHT, 500);
    assert_eq!(design_tokens::typography::MICRO_WEIGHT, 500);
    assert_eq!(text_attrs(TextFace::UiMedium).weight, Weight::MEDIUM);
    assert_eq!(text_attrs(TextFace::UiStrong).weight, Weight::SEMIBOLD);
    assert_eq!(text_attrs(TextFace::Ui).weight, Weight::NORMAL);
}

#[test]
fn text_prepare_signature_tracks_render_relevant_inputs() {
    let run = TextRun {
        text: "TERMINAL".to_string(),
        rich_spans: Vec::new(),
        x: 12.0,
        y: 24.0,
        size: 12.0,
        color: TEXT_PRIMARY,
        face: TextFace::Ui,
        clip_bounds: None,
        layout_size: None,
    };
    let base = text_prepare_signature(&[4], std::slice::from_ref(&run), 1280, 768);
    let mut moved = run.clone();
    moved.x += 1.0;
    assert_ne!(
        base,
        text_prepare_signature(&[4], std::slice::from_ref(&moved), 1280, 768)
    );
    let mut recolored = run.clone();
    recolored.color = TEXT_SECONDARY;
    assert_ne!(
        base,
        text_prepare_signature(&[4], std::slice::from_ref(&recolored), 1280, 768)
    );
    assert_ne!(
        base,
        text_prepare_signature(&[5], std::slice::from_ref(&run), 1280, 768)
    );
    assert_ne!(
        base,
        text_prepare_signature(&[4], std::slice::from_ref(&run), 1281, 768)
    );
}
