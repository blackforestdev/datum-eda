use super::tests::run;
use super::*;

pub(super) fn retain_recent_text_buffers<T>(
    entries: &mut Vec<T>,
    current_frame: u64,
    last_used_frame: impl Fn(&T) -> u64,
) {
    entries.retain(|entry| text_buffer_frame_is_recent(last_used_frame(entry), current_frame));
}

pub(super) fn retain_overlay_buffers<T>(
    entries: &mut Vec<T>,
    age: impl Fn(&T) -> u64,
    size: impl Fn(&T) -> usize,
) -> bool {
    let reordered = entries.windows(2).any(|pair| age(&pair[0]) < age(&pair[1]));
    if reordered {
        entries.sort_by_key(|entry| std::cmp::Reverse(age(entry)));
    }
    let old_len = entries.len();
    let mut bytes = 0;
    let mut count = 0;
    entries.retain(|entry| {
        let next = bytes + size(entry);
        if count >= MAX_OVERLAY_BUFFERS || next > MAX_OVERLAY_TEXT_BYTES {
            return false;
        }
        bytes = next;
        count += 1;
        true
    });
    reordered || entries.len() != old_len
}

#[test]
fn composed_label_retention_preserves_workspace_and_shared_layouts() {
    let mut fonts = load_datum_fonts();
    let mut cache = TextBufferCache::default();
    let mut workspace = run();
    workspace.text = "workspace-only".into();
    let labels: Vec<_> = (0..160)
        .map(|n| {
            let mut label = run();
            label.text = format!("dialog-{n}");
            label
        })
        .collect();
    cache.begin_frame(Profile::Workspace);
    let (work, _) = cache.indices(
        &mut fonts,
        &[workspace.clone(), labels[0].clone()],
        960,
        720,
    );
    let (overlay, stats) = cache.overlay_indices(&mut fonts, &labels, 960, 720);
    assert_eq!(overlay[0], work[1], "shared content owns one layout");
    assert_eq!(stats.hits, 1);
    cache.finish_frame();
    let usage = cache.key_usage();
    assert_eq!(usage.label_entries, 128);
    assert!(usage.label_key_text_bytes <= MAX_OVERLAY_TEXT_BYTES);
    assert_eq!(
        usage.entries, 129,
        "workspace entry does not spend label allowance"
    );
    cache.begin_frame(Profile::Workspace);
    assert_eq!(
        cache
            .indices(&mut fonts, &[workspace, labels[0].clone()], 960, 720)
            .1
            .hits,
        2
    );
    let mut oversized = run();
    oversized.text = "🙂".repeat(MAX_OVERLAY_TEXT_BYTES / 4 + 1);
    let (_, stats) = cache.indices(&mut fonts, &[oversized.clone()], 960, 720);
    assert_eq!(stats.misses, 1);
    assert_eq!(
        cache
            .overlay_indices(&mut fonts, &[oversized.clone()], 960, 720)
            .1
            .hits,
        1
    );
    cache.trim_overlay();
    assert!(cache.key_usage().label_key_text_bytes <= MAX_OVERLAY_TEXT_BYTES);
    assert!(
        cache
            .entries
            .iter()
            .any(|entry| entry.key.text == oversized.text && !entry.overlay_retained),
        "workspace reference survives label retention bypass"
    );
}
