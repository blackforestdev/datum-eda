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
fn required_layout_refusal_releases_derived_storage_and_rebuilds_current_content() {
    let mut fonts = crate::load_datum_fonts();
    let state = crate::global_preferences_dialog_tests::state_with_preferences_open();
    let prepared =
        PreparedScene::from_native_preferences(&state.ui.global_preferences, 960, 720, 1.0);
    let mut cache = TextBufferCache::default();
    cache.begin_frame(Profile::Overlay);
    let (mut current, _) = cache.indices(&mut fonts, &prepared.menu_overlay_text_runs, 960, 720);
    let before: Vec<_> = cache
        .entries
        .iter()
        .map(|entry| {
            entry
                .buffer
                .layout_runs()
                .map(|row| format!("{:?}", row.glyphs))
                .collect::<Vec<_>>()
        })
        .collect();
    let revision = cache.revision;
    let scratch_budget = crate::text_gpu::budget::Budget::new(16 * 1024 * 1024);
    cache.admit_layout_scratch(&scratch_budget);
    let filler = budget::Owner::new(32 * 1024 * 1024);
    assert!(cache.admit_frame(&mut [&mut current]).is_err());
    assert!(current.is_empty());
    assert_eq!(cache.entries.capacity(), 0);
    assert_eq!(cache.lookup.capacity(), 0);
    assert_ne!(cache.revision, revision);
    assert_eq!(scratch_budget.used(), 0);
    assert_eq!(cache.layout_scratch.private_bytes(0, 0), Some(0));
    let usage = crate::Renderer::text_cache_process_usage()
        .into_iter()
        .find(|owner| owner.owner_id == cache.owner.id())
        .unwrap();
    assert!(!usage.preparing && usage.retention_overflow);
    assert_eq!(usage.bytes, std::mem::size_of::<TextBufferCache>());
    drop(filler);
    let (mut current, stats) =
        cache.indices(&mut fonts, &prepared.menu_overlay_text_runs, 960, 720);
    assert!(stats.misses > 0);
    let rebuilt: Vec<_> = cache
        .entries
        .iter()
        .map(|entry| {
            entry
                .buffer
                .layout_runs()
                .map(|row| format!("{:?}", row.glyphs))
                .collect::<Vec<_>>()
        })
        .collect();
    assert_eq!(rebuilt, before);
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

#[test]
#[ignore = "requires serial process-wide cache admission"]
fn local_capacity_refusal_drops_oversized_key_allocation_and_all_index_groups() {
    let mut fonts = crate::load_datum_fonts();
    let state = crate::global_preferences_dialog_tests::state_with_preferences_open();
    let prepared =
        PreparedScene::from_native_preferences(&state.ui.global_preferences, 960, 720, 1.0);
    let mut cache = TextBufferCache::default();
    cache.begin_frame(Profile::Workspace);
    let (mut workspace, _) =
        cache.indices(&mut fonts, &prepared.menu_overlay_text_runs[..1], 960, 720);
    let (mut overlay, _) =
        cache.indices(&mut fonts, &prepared.menu_overlay_text_runs[1..2], 960, 720);
    // Model an oversized owned capacity without shaping megabytes of synthetic text.
    cache.entries[0].key.text.reserve(8 * 1024 * 1024);
    cache.revision = cache.revision.wrapping_add(1);
    assert!(cache.retained_payload_bytes() > 8 * 1024 * 1024);
    let error = cache
        .admit_frame(&mut [&mut workspace, &mut overlay])
        .unwrap_err();
    assert!(error.to_string().contains("required text layout exceeds"));
    assert!(workspace.is_empty() && overlay.is_empty());
    assert_eq!(
        cache.retained_payload_bytes(),
        std::mem::size_of::<TextBufferCache>()
    );
    assert_eq!(cache.entries.capacity(), 0);
    assert_eq!(cache.lookup.capacity(), 0);
}

#[test]
#[ignore = "requires serial process-wide cache admission"]
fn admission_compacts_spare_metadata_without_discarding_required_layouts() {
    let mut fonts = crate::load_datum_fonts();
    let state = crate::global_preferences_dialog_tests::state_with_preferences_open();
    let prepared =
        PreparedScene::from_native_preferences(&state.ui.global_preferences, 960, 720, 1.0);
    let mut cache = TextBufferCache::default();
    cache.begin_frame(Profile::Overlay);
    let (mut indices, _) =
        cache.indices(&mut fonts, &prepared.menu_overlay_text_runs[..3], 960, 720);
    let shapes: Vec<_> = indices
        .iter()
        .map(|i| cache.entries[*i].buffer.shape_storage())
        .collect();
    cache.entries.reserve_exact(3);
    cache.lookup.reserve_exact(3);
    cache.revision = cache.revision.wrapping_add(1);
    let before = cache.retained_payload_bytes();
    let required = before
        - capacity_bytes::<CachedTextBuffer>(cache.entries.capacity())
        - capacity_bytes::<(u64, usize)>(cache.lookup.capacity())
        + capacity_bytes::<CachedTextBuffer>(cache.entries.len())
        + capacity_bytes::<(u64, usize)>(cache.lookup.len());
    assert!(before > required);
    let others: usize = crate::Renderer::text_cache_process_usage()
        .iter()
        .filter(|owner| owner.owner_id != cache.owner.id())
        .map(|owner| owner.bytes)
        .sum();
    let filler = budget::Owner::new(0);
    filler.publish(
        32 * 1024 * 1024 - others - required - crate::Renderer::text_cache_registry_bytes(),
    );
    cache.admit_frame(&mut [&mut indices]).unwrap();
    assert_eq!(indices, [0, 1, 2]);
    assert_eq!(cache.entries.capacity(), cache.entries.len());
    assert_eq!(cache.lookup.capacity(), cache.lookup.len());
    assert_eq!(cache.retained_payload_bytes(), required);
    assert_eq!(
        shapes,
        indices
            .iter()
            .map(|i| cache.entries[*i].buffer.shape_storage())
            .collect::<Vec<_>>()
    );
    drop(filler);
}

#[test]
fn warm_frames_publish_preparing_without_changing_owned_bytes() {
    let mut fonts = crate::load_datum_fonts();
    let state = crate::global_preferences_dialog_tests::state_with_preferences_open();
    let prepared =
        PreparedScene::from_native_preferences(&state.ui.global_preferences, 960, 720, 1.0);
    let mut cache = TextBufferCache::default();
    let owner = cache.owner.id();
    let usage = || {
        crate::Renderer::text_cache_process_usage()
            .into_iter()
            .find(|u| u.owner_id == owner)
            .unwrap()
    };
    cache.begin_frame(Profile::Overlay);
    cache.indices(&mut fonts, &prepared.menu_overlay_text_runs[..1], 960, 720);
    cache.finish_frame();
    let before = usage();
    assert!(!before.preparing);
    cache.begin_frame(Profile::Overlay);
    assert!(usage().preparing);
    assert_eq!(usage().bytes, before.bytes);
    assert_eq!(
        cache
            .indices(&mut fonts, &prepared.menu_overlay_text_runs[..1], 960, 720)
            .1
            .hits,
        1
    );
    assert!(usage().preparing);
    cache.finish_frame();
    assert!(!usage().preparing);
    assert_eq!(usage().bytes, before.bytes);
}

#[test]
fn oversized_plain_and_rich_keys_refuse_before_shaping_and_preserve_warm_cache() {
    let state = crate::global_preferences_dialog_tests::state_with_preferences_open();
    let prepared =
        PreparedScene::from_native_preferences(&state.ui.global_preferences, 960, 720, 1.0);
    let mut run = prepared.menu_overlay_text_runs[0].clone();
    run.text = "preserved warm label".into();
    run.rich_spans.clear();
    let host = crate::text_gpu::budget::Budget::new(16 * 1024 * 1024);
    let mut fonts = crate::text_layout::fonts::Fonts::new(host.clone());
    let mut cache = TextBufferCache::default();
    cache.begin_frame(Profile::Overlay);
    let (indices, _) = cache
        .admitted_indices(
            &mut fonts,
            std::slice::from_ref(&run),
            960,
            720,
            true,
            &host,
            None,
        )
        .unwrap();
    drop(indices);
    let revision = cache.revision();
    let font_usage = fonts.usage().allocation;
    let used = host.used();
    let mut rejected = run.clone();
    rejected.text = "x".repeat(budget::LOCAL_LIMIT);
    for rich in [false, true] {
        if rich {
            rejected.rich_spans.push(crate::TextRunSpan {
                text: std::mem::take(&mut rejected.text),
                color: [1.0; 3],
                bold: true,
                italic: false,
            });
        }
        let error = cache
            .admitted_indices(
                &mut fonts,
                std::slice::from_ref(&rejected),
                960,
                720,
                true,
                &host,
                None,
            )
            .err()
            .expect("oversized input refused");
        assert!(error.to_string().contains("before shaping"));
        assert_eq!(cache.revision(), revision);
        assert_eq!(host.used(), used, "no index/staging allocation on refusal");
        assert_eq!(
            fonts.usage().allocation.peak_payload_bytes,
            font_usage.peak_payload_bytes
        );
        assert_eq!(fonts.usage().allocation.allocations, font_usage.allocations);
    }
    let (indices, stats) = cache
        .admitted_indices(
            &mut fonts,
            std::slice::from_ref(&run),
            960,
            720,
            true,
            &host,
            None,
        )
        .unwrap();
    assert_eq!(indices.len(), 1);
    assert_eq!(stats.hits, 1);
    assert_eq!(stats.misses, 0);
    // Rich spans replace fallback text; unused fallback size cannot reject them.
    run.text = "x".repeat(budget::LOCAL_LIMIT);
    run.rich_spans.push(crate::TextRunSpan {
        text: "visible".into(),
        color: [1.0; 3],
        bold: false,
        italic: false,
    });
    TextBufferCache::admit_input_keys(&[run.clone(), run]).unwrap();
}

#[test]
fn rich_input_refuses_before_shaping_and_warm_layouts_need_no_concat_storage() {
    use crate::text_gpu::{budget::Budget, staging_vec::StagingVec};
    let state = crate::global_preferences_dialog_tests::state_with_preferences_open();
    let prepared =
        PreparedScene::from_native_preferences(&state.ui.global_preferences, 960, 720, 1.0);
    let mut run = prepared.menu_overlay_text_runs[0].clone();
    run.layout_size = Some((400.0, 200.0));
    run.rich_spans = vec![
        crate::TextRunSpan {
            text: "Unicode e\u{301} Ω\n".into(),
            color: [1.0; 3],
            bold: true,
            italic: false,
        },
        crate::TextRunSpan {
            text: "אבג finish".into(),
            color: [0.5; 3],
            bold: false,
            italic: true,
        },
    ];
    let host = Budget::new(16 * 1024 * 1024);
    let mut fonts = crate::text_layout::fonts::Fonts::new(host.clone());
    let mut cache = TextBufferCache::default();
    cache.begin_frame(Profile::Overlay);
    let before = fonts.usage().allocation;
    // Index storage fits exactly; concatenation cannot be allocated.
    let index_bytes = StagingVec::<usize>::capacity_bytes(1).unwrap();
    let filler = host.reserve(host.available() - index_bytes).unwrap();
    assert!(
        cache
            .admitted_indices(
                &mut fonts,
                std::slice::from_ref(&run),
                960,
                720,
                true,
                &host,
                None
            )
            .is_err()
    );
    assert!(cache.entries.is_empty());
    assert_eq!(
        fonts.usage().allocation.peak_payload_bytes,
        before.peak_payload_bytes
    );
    assert_eq!(
        host.available(),
        index_bytes,
        "failed batch releases its indices"
    );
    drop(filler);
    let (indices, stats) = cache
        .admitted_indices(
            &mut fonts,
            std::slice::from_ref(&run),
            960,
            720,
            true,
            &host,
            None,
        )
        .unwrap();
    assert_eq!(stats.misses, 1);
    assert!(!cache.entries[indices[0]].buffer.needs_input());
    drop(indices);
    assert_eq!(
        host.used(),
        fonts.reserved_bytes(),
        "concatenation does not survive preparation"
    );
    let filler = host.reserve(host.available() - index_bytes).unwrap();
    let (indices, stats) = cache
        .admitted_indices(
            &mut fonts,
            std::slice::from_ref(&run),
            960,
            720,
            true,
            &host,
            None,
        )
        .unwrap();
    assert_eq!(stats.hits, 1);
    drop(indices);
    cache.begin_frame(Profile::Overlay);
    run.layout_size = Some((300.0, 200.0));
    let (indices, stats) = cache
        .admitted_indices(
            &mut fonts,
            std::slice::from_ref(&run),
            960,
            720,
            true,
            &host,
            None,
        )
        .unwrap();
    assert_eq!(stats.misses, 1);
    assert_eq!(
        cache.shape_reuses, 1,
        "complete shapes relayout without copying input"
    );
    drop(indices);
    drop(filler);
}
