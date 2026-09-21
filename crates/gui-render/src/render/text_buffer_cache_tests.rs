//! Shaped-cache identity, layout reuse and bounded retention proofs.
use super::*;

fn run() -> TextRun {
    TextRun {
        text: "Cache label".into(),
        rich_spans: vec![],
        x: 0.0,
        y: 0.0,
        size: 12.0,
        color: TEXT_PRIMARY,
        face: TextFace::Ui,
        clip_bounds: None,
    }
}

#[test]
fn borrowed_lookup_preserves_the_complete_existing_key() {
    let mut original = run();
    original.rich_spans.push(TextRunSpan {
        text: "span".into(),
        color: TEXT_PRIMARY,
        bold: false,
        italic: false,
    });
    let key = text_buffer_key(&original, 1280, 800);
    let mut variants = vec![original.clone()];
    for change in 0..10 {
        let mut changed = original.clone();
        match change {
            0 => {
                changed.x += 20.0;
                changed.y += 30.0;
                changed.color = TEXT_SECONDARY;
            }
            1 => changed.text.push('!'),
            2 => changed.size += 1.0,
            3 => changed.face = TextFace::Terminal,
            4 => changed.rich_spans[0].text.push('!'),
            5 => changed.rich_spans[0].bold = true,
            6 => changed.rich_spans[0].italic = true,
            7 => changed.rich_spans[0].color = TEXT_SECONDARY,
            8 => changed.rich_spans.clear(),
            _ => {
                changed.clip_bounds = Some(RectPx {
                    x: 0.0,
                    y: 0.0,
                    width: 40.0,
                    height: 20.0,
                })
            }
        }
        variants.push(changed);
    }
    for candidate in variants {
        for (width, height) in [(1280, 800), (25, 10)] {
            assert_eq!(
                matches_run(
                    &key,
                    &candidate,
                    text_buffer_extent(&candidate, width, height)
                ),
                key == text_buffer_key(&candidate, width, height)
            );
        }
    }
}

#[test]
fn real_shaped_cache_reuses_placement_changes_and_retires_old_workspace_rows() {
    let mut fonts = FontSystem::new();
    load_datum_fonts(&mut fonts);
    let mut cache = TextBufferCache::default();
    let original = run();
    cache.begin_frame(Profile::Workspace);
    let (indices, cold) = cache.indices(&mut fonts, std::slice::from_ref(&original), 1280, 800);
    assert_eq!(cold.misses, 1);
    assert!(
        cache.entries()[indices[0]]
            .buffer
            .layout_runs()
            .next()
            .is_some()
    );
    let key_storage = cache.entries()[indices[0]].key.text.as_ptr();
    let mut moved = original.clone();
    moved.x += 20.0;
    moved.color = TEXT_SECONDARY;
    cache.begin_frame(Profile::Workspace);
    let (warm_indices, warm) = cache.indices(&mut fonts, &[moved], 1280, 800);
    assert_eq!(warm.hits, 1);
    assert_eq!(warm.misses, 0);
    assert_eq!(indices, warm_indices);
    assert_eq!(key_storage, cache.entries()[indices[0]].key.text.as_ptr());
    let mut changed = original.clone();
    changed.text.push('!');
    cache.begin_frame(Profile::Workspace);
    assert_eq!(cache.indices(&mut fonts, &[changed], 1280, 800).1.misses, 1);
    assert_eq!(cache.entries().len(), 2);
    cache.begin_frame(Profile::Workspace);
    assert_eq!(cache.entries().len(), 1);
    assert_ne!(cache.entries()[0].key.text, original.text);
    cache.begin_frame(Profile::Overlay);
    cache.indices(&mut fonts, std::slice::from_ref(&original), 1280, 800);
    cache.trim_overlay();
    cache.begin_frame(Profile::Overlay);
    assert_eq!(
        cache.indices(&mut fonts, &[original], 1280, 800).1.misses,
        0
    );
}

#[derive(Debug)]
struct SimulatedBuffer {
    key: String,
    last_used_frame: u64,
}

#[test]
fn dialog_cache_retains_scroll_history_with_count_and_text_budgets() {
    let mut entries = Vec::new();
    for frame in 0..1000 {
        entries.push(SimulatedBuffer {
            key: format!("row-{frame}"),
            last_used_frame: frame,
        });
        retain_overlay_buffers(
            &mut entries,
            |entry| entry.last_used_frame,
            |entry| entry.key.len(),
        );
        assert!(entries.len() <= MAX_OVERLAY_BUFFERS);
    }
    assert!(entries.iter().any(|entry| entry.key == "row-990"));
    assert!(entries.iter().any(|entry| entry.key == "row-999"));
    entries.extend((0..100).map(|n| SimulatedBuffer {
        key: "x".repeat(1024),
        last_used_frame: 1000 + n,
    }));
    retain_overlay_buffers(
        &mut entries,
        |entry| entry.last_used_frame,
        |entry| entry.key.len(),
    );
    assert!(entries.iter().map(|entry| entry.key.len()).sum::<usize>() <= MAX_OVERLAY_TEXT_BYTES);
    entries.push(SimulatedBuffer {
        key: "x".repeat(MAX_OVERLAY_TEXT_BYTES + 1),
        last_used_frame: 2000,
    });
    retain_overlay_buffers(
        &mut entries,
        |entry| entry.last_used_frame,
        |entry| entry.key.len(),
    );
    assert!(
        entries
            .iter()
            .all(|entry| entry.key.len() <= MAX_OVERLAY_TEXT_BYTES)
    );
}

#[test]
fn animated_agent_text_cache_retains_only_two_visible_generations() {
    const VISIBLE_RUNS: usize = 64;
    let mut cache: Vec<SimulatedBuffer> = Vec::new();
    let mut maximum_resident = 0usize;
    for unique in 0..100_000_usize {
        let frame = (unique / VISIBLE_RUNS + 1) as u64;
        if unique % VISIBLE_RUNS == 0 {
            retain_recent_text_buffers(&mut cache, frame, |entry| entry.last_used_frame);
        }
        let key = format!("agent-frame-{frame}-run-{}", unique % VISIBLE_RUNS);
        cache.push(SimulatedBuffer {
            key,
            last_used_frame: frame,
        });
        assert!(
            cache
                .iter()
                .any(|entry| entry.key == cache.last().unwrap().key)
        );
        maximum_resident = maximum_resident.max(cache.len());
    }
    assert_eq!(maximum_resident, VISIBLE_RUNS * 2);
    assert!(cache.len() <= VISIBLE_RUNS * 2);
    let last_frame = 100_000_usize.div_ceil(VISIBLE_RUNS) as u64;
    assert!(
        cache
            .iter()
            .all(|entry| entry.last_used_frame >= last_frame - 1)
    );
}

#[test]
fn extent_changes_relayout_cached_shaping_with_fresh_buffer_parity() {
    let mut fonts = FontSystem::new();
    load_datum_fonts(&mut fonts);
    let mut cache = TextBufferCache::default();
    let mut label = run();
    label.text = "Text across several wrap widths: µm and Ω".into();
    let signature = |buffer: &Buffer| {
        buffer
            .layout_runs()
            .map(|line| {
                (
                    line.line_y.to_bits(),
                    line.line_w.to_bits(),
                    format!("{:?}", line.glyphs),
                )
            })
            .collect::<Vec<_>>()
    };
    cache.begin_frame(Profile::Overlay);
    for (step, width) in [220.0, 90.0, 140.0, 90.0].into_iter().enumerate() {
        label.clip_bounds = Some(RectPx {
            x: 0.0,
            y: 0.0,
            width,
            height: 150.0,
        });
        let (indices, _) = cache.indices(&mut fonts, std::slice::from_ref(&label), 400, 300);
        let reused = &cache.entries()[indices[0]].buffer;
        assert!(reused.lines[0].shape_opt().is_some());
        let mut fresh = TextBufferCache::default();
        let (fresh_indices, _) = fresh.indices(&mut fonts, std::slice::from_ref(&label), 400, 300);
        assert_eq!(
            signature(reused),
            signature(&fresh.entries()[fresh_indices[0]].buffer)
        );
        assert_eq!(cache.shape_reuses, step.min(2));
    }
    let mut moved = label.clone();
    moved.x += 10.0;
    moved.clip_bounds.as_mut().unwrap().x += 10.0;
    assert_eq!(cache.indices(&mut fonts, &[moved], 400, 300).1.misses, 0);
    let reuse_count = cache.shape_reuses;
    label.size += 1.0;
    cache.indices(&mut fonts, std::slice::from_ref(&label), 400, 300);
    assert_eq!(
        cache.shape_reuses, reuse_count,
        "font-size change must shape afresh"
    );
    label.rich_spans.push(TextRunSpan {
        text: label.text.clone(),
        color: TEXT_PRIMARY,
        bold: true,
        italic: false,
    });
    cache.indices(&mut fonts, &[label], 400, 300);
    assert_eq!(
        cache.shape_reuses, reuse_count,
        "rich-style change must shape afresh"
    );
}

#[test]
fn indexed_lookup_checks_collisions_and_tracks_retirement() {
    let mut fonts = FontSystem::new();
    load_datum_fonts(&mut fonts);
    let mut cache = TextBufferCache::default();
    let runs: Vec<_> = (0..640)
        .map(|index| TextRun {
            text: format!("indexed label {index}"),
            ..run()
        })
        .collect();
    cache.begin_frame(Profile::Overlay);
    let (cold, stats) = cache.indices(&mut fonts, &runs, 1280, 800);
    assert_eq!(stats.misses, runs.len());
    cache.key_comparisons = 0;
    let (warm, stats) = cache.indices(&mut fonts, &runs, 1280, 800);
    assert_eq!(warm, cold);
    assert_eq!(stats.hits, runs.len());
    assert_eq!(cache.key_comparisons, runs.len());

    // Force an unrelated entry into the same fingerprint bucket. The digest
    // narrows candidates only; accepting it as identity would return row zero.
    let fingerprint = run_fingerprint(&runs[1]);
    cache.lookup = vec![(fingerprint, 0), (fingerprint, 1)];
    cache.key_comparisons = 0;
    let (matched, _) = cache.indices(&mut fonts, &runs[1..2], 1280, 800);
    assert_eq!(matched, [1]);
    assert_eq!(cache.key_comparisons, 2);
    cache.rebuild_lookup();

    cache.begin_frame(Profile::Overlay);
    cache.indices(&mut fonts, &runs[639..], 1280, 800);
    cache.trim_overlay();
    assert_eq!(cache.entries[0].key.text, runs[639].text);
    assert_eq!(cache.entries.len(), MAX_OVERLAY_BUFFERS);
    assert_eq!(cache.lookup.len(), cache.entries.len());
    assert!(cache.lookup.capacity() <= 4 * MAX_OVERLAY_BUFFERS);
    assert!(cache.entries.capacity() <= 4 * MAX_OVERLAY_BUFFERS);
    eprintln!(
        "text index retained metadata capacity bytes={}",
        cache.lookup.capacity() * std::mem::size_of::<(u64, usize)>()
    );
    let retained: Vec<_> = cache
        .entries
        .iter()
        .map(|entry| entry.key.text.clone())
        .collect();
    for text in retained {
        let input = TextRun { text, ..run() };
        let (indices, stats) = cache.indices(&mut fonts, std::slice::from_ref(&input), 1280, 800);
        assert_eq!(stats.hits, 1);
        assert_eq!(cache.entries[indices[0]].key.text, input.text);
    }
    for _ in 0..2 {
        cache.begin_frame(Profile::Workspace);
        let (indices, stats) = cache.indices(&mut fonts, &runs[1..2], 1280, 800);
        assert_eq!(stats.hits, 1);
        assert_eq!(cache.entries[indices[0]].key.text, runs[1].text);
    }
    assert_eq!(cache.entries.len(), 1);
    assert_eq!(cache.lookup.len(), 1);
    assert!(cache.lookup.capacity() <= 4);
    assert!(cache.entries.capacity() <= 4);
    cache.begin_frame(Profile::Workspace);
    cache.begin_frame(Profile::Workspace);
    assert!(cache.lookup.is_empty());
    assert_eq!(cache.lookup.capacity(), 0);
    assert_eq!(cache.entries.capacity(), 0);
}

#[test]
fn relayout_moves_unused_entries_but_preserves_current_frame_layouts() {
    let mut fonts = FontSystem::new();
    load_datum_fonts(&mut fonts);
    let signature = |buffer: &Buffer| {
        buffer
            .layout_runs()
            .map(|line| (line.line_w.to_bits(), format!("{:?}", line.glyphs)))
            .collect::<Vec<_>>()
    };
    for profile in [Profile::Workspace, Profile::Overlay] {
        let mut cache = TextBufferCache::default();
        let mut label = run();
        label.text = "Retain shaped paragraphs across wrap changes: µm Ω".into();
        let mut shape_storage = None;
        for width in [220, 90, 140, 90] {
            cache.begin_frame(profile);
            let revision = cache.revision();
            let (indices, stats) =
                cache.indices(&mut fonts, std::slice::from_ref(&label), width, 300);
            assert_eq!(indices, [0]);
            assert_eq!(stats.misses, 1, "changed layout remains a layout miss");
            assert_eq!(cache.entries.len(), 1, "obsolete layout must not be cloned");
            assert_ne!(cache.revision(), revision);
            let buffer = &cache.entries[0].buffer;
            let storage = buffer.lines[0].shape_opt().unwrap().spans.as_ptr();
            assert_eq!(*shape_storage.get_or_insert(storage), storage);
            let mut fresh = TextBufferCache::default();
            fresh.indices(&mut fonts, std::slice::from_ref(&label), width, 300);
            assert_eq!(signature(buffer), signature(&fresh.entries[0].buffer));
            assert_eq!(
                cache
                    .indices(&mut fonts, std::slice::from_ref(&label), width, 300)
                    .1
                    .hits,
                1
            );
            if matches!(profile, Profile::Overlay) {
                cache.trim_overlay();
            }
        }
        // Two consumers may need different extents within one preparation. The
        // first index must keep its layout while the second gets a separate one.
        cache.begin_frame(profile);
        let (first, _) = cache.indices(&mut fonts, std::slice::from_ref(&label), 90, 300);
        let before = signature(&cache.entries[first[0]].buffer);
        let (second, _) = cache.indices(&mut fonts, std::slice::from_ref(&label), 210, 300);
        assert_ne!(first, second);
        assert_eq!(signature(&cache.entries[first[0]].buffer), before);
        assert_eq!(cache.entries[first[0]].key.width_px, 90);
        assert_eq!(cache.entries[second[0]].key.width_px, 210);
    }
}
