use super::*;

#[test]
#[ignore = "requires serial process-wide cache admission"]
fn current_layout_admission_evicts_only_history_and_remaps_current_indices() {
    let mut fonts = crate::load_datum_fonts();
    let state = crate::global_preferences_dialog_tests::state_with_preferences_open();
    let prepared =
        PreparedScene::from_native_preferences(&state.ui.global_preferences, 960, 720, 1.0);
    let mut cache = TextBufferCache::default();
    let mut runs = prepared.menu_overlay_text_runs[..2].to_vec();
    runs[0].text = "unused old layout".into();
    runs[1].text = "visible current layout".into();
    cache.begin_frame(Profile::Overlay);
    cache.indices(&mut fonts, &runs, 960, 720);
    cache.finish_frame();
    cache.begin_frame(Profile::Overlay);
    let (mut current, _) = cache.indices(&mut fonts, &runs[1..], 960, 720);
    assert_eq!(current, [1]);
    // Other live owners leave enough for the required row but not its history.
    let required = cache.retained_payload_bytes();
    let filler = budget::Owner::new(32 * 1024 * 1024 - required + 1);
    cache.admit_frame(&mut [&mut current]).unwrap();
    assert_eq!(current, [0]);
    assert_eq!(cache.entries.len(), 1);
    assert_eq!(cache.entries[0].key.text, "visible current layout");
    assert!(cache.retained_payload_bytes() < required);
    let usage = cache.key_usage();
    assert_eq!(
        cache.retained_payload_bytes(),
        usage.key_text_bytes + usage.entry_storage_bytes + usage.shaped_payload_bytes
    );
    cache.finish_frame();
    assert!(
        !crate::Renderer::text_cache_process_usage()
            .iter()
            .find(|owner| owner.owner_id == cache.owner.id())
            .unwrap()
            .preparing
    );
    drop(filler);
}

#[test]
#[ignore = "requires serial process-wide cache admission"]
fn required_layout_refusal_preserves_current_content_for_retry() {
    let mut fonts = crate::load_datum_fonts();
    let state = crate::global_preferences_dialog_tests::state_with_preferences_open();
    let prepared =
        PreparedScene::from_native_preferences(&state.ui.global_preferences, 960, 720, 1.0);
    let mut cache = TextBufferCache::default();
    cache.begin_frame(Profile::Overlay);
    let (mut current, _) = cache.indices(&mut fonts, &prepared.menu_overlay_text_runs, 960, 720);
    let before = current.clone();
    let count = cache.entries.len();
    let filler = budget::Owner::new(32 * 1024 * 1024);
    assert!(cache.admit_frame(&mut [&mut current]).is_err());
    assert_eq!(current, before);
    assert_eq!(cache.entries.len(), count);
    drop(filler);
    cache.admit_frame(&mut [&mut current]).unwrap();
    cache.finish_frame();
    // A warm admitted frame must leave the public preparing state after submit.
    cache.begin_frame(Profile::Overlay);
    let (mut current, _) = cache.indices(&mut fonts, &prepared.menu_overlay_text_runs, 960, 720);
    cache.admit_frame(&mut [&mut current]).unwrap();
    cache.finish_frame();
    assert!(
        !crate::Renderer::text_cache_process_usage()
            .iter()
            .find(|owner| owner.owner_id == cache.owner.id())
            .unwrap()
            .preparing
    );
}
