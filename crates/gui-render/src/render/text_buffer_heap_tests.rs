//! Whole retained text owner conformance against actual allocator lifetimes.
use super::*;

#[test]
fn accounting_and_retirement_allocate_no_temporary_storage_for_shared_extents() {
    let mut fonts = load_datum_fonts();
    let mut cache = TextBufferCache::default();
    let runs: Vec<_> = (0..160)
        .map(|index| TextRun {
            text: "A shared shape with many visible extents".into(),
            layout_size: Some((80.0 + index as f32, 120.0)),
            ..run()
        })
        .collect();
    cache.begin_frame(Profile::Overlay);
    cache.indices(&mut fonts, &runs, 960, 720);
    cache.begin_frame(Profile::Overlay);
    cache.indices(&mut fonts, &runs[159..], 960, 720);
    let scope = crate::cpu_alloc::Scope::new("text-retirement-temporaries");
    let usage = scope.with(|| {
        cache.finish_frame();
        cache.key_usage()
    });
    assert_eq!(
        scope.usage().peak_payload_bytes,
        0,
        "retirement and shared-shape counting must not allocate sort or map scratch"
    );
    assert_eq!(usage.entries, MAX_OVERLAY_BUFFERS);
    assert_eq!(cache.entries[0].key.width_px, 239);
    // Independent inventory, deliberately outside the production-accounting scope.
    let shapes: std::collections::BTreeMap<_, _> = cache
        .entries
        .iter()
        .flat_map(|entry| entry.buffer.shape_allocations())
        .collect();
    let expected = shapes.values().sum::<usize>()
        + cache
            .entries
            .iter()
            .map(|entry| entry.buffer.layout_storage_bytes())
            .sum::<usize>();
    assert_eq!(usage.shaped_payload_bytes, expected);
    std::thread::scope(|threads| {
        for _ in 0..2 {
            let cache = &cache;
            threads.spawn(move || {
                for _ in 0..20 {
                    assert_eq!(cache.key_usage().shaped_payload_bytes, expected);
                }
            });
        }
    });
}

#[test]
fn complete_retained_text_accounting_matches_heap_after_shared_layout_construction() {
    // This measures the complete retained owner rather than individual containers.
    // Font caches are independent of retained layouts; dropping fonts below leaves
    // their allocation lifetime out of the shaped-payload comparison.
    let mut fonts = load_datum_fonts();
    let mut cache = TextBufferCache::default();
    let scope = crate::cpu_alloc::Scope::new("complete-retained-text");
    let mut plain = run();
    plain.text = "Shared paragraphs with wrapped text\nsecond paragraph".into();
    plain.layout_size = Some((180.0, 300.0));
    let mut rich = plain.clone();
    rich.rich_spans = vec![
        TextRunSpan {
            text: "Styled ".into(),
            color: TEXT_PRIMARY,
            bold: true,
            italic: false,
        },
        TextRunSpan {
            text: "paragraph".into(),
            color: TEXT_SECONDARY,
            bold: false,
            italic: true,
        },
    ];
    let mut narrow = plain.clone();
    narrow.layout_size = Some((90.0, 300.0));
    scope.with(|| {
        cache.begin_frame(Profile::Workspace);
        let mut runs = [plain, rich, narrow];
        let (indices, stats) = cache.indices(&mut fonts, &runs, 960, 720);
        assert_eq!(stats.misses, 3);
        assert_eq!(indices.len(), 3);
        assert_eq!(cache.published_bytes, cache.retained_payload_bytes());
        cache.begin_frame(Profile::Workspace);
        runs[0].layout_size = Some((220.0, 300.0));
        cache.indices(&mut fonts, &runs, 960, 720);
    });
    assert_eq!(
        cache.published_bytes,
        cache.retained_payload_bytes(),
        "incremental preparation equals the independent full cache inventory"
    );
    drop(fonts);
    let usage = scope.usage();
    let returned_glyphs: usize = cache
        .entries
        .iter()
        .map(|entry| entry.buffer.layout_glyph_bytes() + entry.buffer.layout_glyph_tracking_bytes())
        .sum();
    assert_eq!(
        cache.retained_payload_bytes(),
        std::mem::size_of::<TextBufferCache>()
            + (usage.payload_bytes + usage.tracking_bytes) as usize
            + returned_glyphs,
        "private layout scratch is separate; shared shapes count once"
    );
    drop(cache);
    assert_eq!(scope.usage().allocations, 0);
}
