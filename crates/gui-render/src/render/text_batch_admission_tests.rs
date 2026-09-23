use super::*;
use crate::text_layout::fonts::{Fonts, Shape, Source};

struct CountingFonts {
    inner: Fonts,
    shapes: usize,
}
impl Source for CountingFonts {
    fn release_for(&mut self, bytes: u64) {
        self.inner.release_for(bytes);
    }
    fn shape(&mut self, text: &str, attrs: &glyphon::AttrsList) -> Shape {
        self.shapes += 1;
        self.inner.shape(text, attrs)
    }
    fn raster(
        &mut self,
        cache: &mut glyphon::SwashCache,
        scope: &crate::cpu_alloc::Scope,
        key: glyphon::CacheKey,
    ) -> Option<glyphon::SwashImage> {
        self.inner.raster(cache, scope, key)
    }
}

#[test]
#[ignore = "requires serial process-wide text admission"]
fn overlay_pressure_stops_at_first_excess_layout_and_invalidates_workspace_indices() {
    let state = crate::global_preferences_dialog_tests::state_with_preferences_open();
    let prepared =
        PreparedScene::from_native_preferences(&state.ui.global_preferences, 960, 720, 1.0);
    let mut run = prepared.menu_overlay_text_runs[0].clone();
    run.text = "current workspace label".into();
    run.rich_spans.clear();
    let host = crate::text_gpu::budget::Budget::new(16 * 1024 * 1024);
    let mut fonts = CountingFonts {
        inner: Fonts::new(host.clone()),
        shapes: 0,
    };
    let mut cache = TextBufferCache::default();
    cache.begin_frame(Profile::Workspace);
    let (mut workspace, _) = cache
        .admitted_indices(
            &mut fonts,
            std::slice::from_ref(&run),
            960,
            720,
            false,
            &host,
            None,
        )
        .unwrap();
    assert_eq!(cache.published_bytes, cache.retained_payload_bytes());
    let limit = cache.published_bytes;
    let calls = fonts.shapes;
    let filler = budget::Owner::new(0);
    let others: usize = crate::Renderer::text_cache_process_usage()
        .iter()
        .filter(|usage| usage.owner_id != cache.owner.id() && usage.owner_id != filler.id())
        .map(|usage| usage.bytes)
        .sum();
    filler
        .publish(32 * 1024 * 1024 - crate::Renderer::text_cache_registry_bytes() - others - limit);
    let overlays: Vec<_> = (0..12)
        .map(|index| {
            let mut label = run.clone();
            label.text = format!("new overlay label {index}");
            label
        })
        .collect();
    assert!(
        cache
            .admitted_indices(
                &mut fonts,
                &overlays,
                960,
                720,
                true,
                &host,
                Some(&mut workspace)
            )
            .is_err()
    );
    assert_eq!(
        fonts.shapes, calls,
        "refuse construction before shaping an over-budget batch"
    );
    assert!(
        workspace.is_empty(),
        "rejection invalidates every earlier group"
    );
    assert!(cache.entries.is_empty());
    assert_eq!(cache.published_bytes, cache.retained_payload_bytes());
    drop(filler);
    let (mut workspace, _) = cache
        .admitted_indices(
            &mut fonts,
            std::slice::from_ref(&run),
            960,
            720,
            false,
            &host,
            None,
        )
        .unwrap();
    let (overlay, _) = cache
        .admitted_indices(
            &mut fonts,
            &overlays,
            960,
            720,
            true,
            &host,
            Some(&mut workspace),
        )
        .unwrap();
    assert_eq!(cache.entries[workspace[0]].key.text, run.text);
    assert_eq!(overlay.len(), overlays.len());
    for (index, input) in overlay.iter().zip(&overlays) {
        assert_eq!(cache.entries[*index].key.text, input.text);
    }
    assert_eq!(cache.published_bytes, cache.retained_payload_bytes());
    let shaped = fonts.shapes;
    cache.begin_frame(Profile::Workspace);
    let (mut workspace, stats) = cache
        .admitted_indices(
            &mut fonts,
            std::slice::from_ref(&run),
            960,
            720,
            false,
            &host,
            None,
        )
        .unwrap();
    assert_eq!(stats.hits, 1);
    let (_, stats) = cache
        .admitted_indices(
            &mut fonts,
            &overlays,
            960,
            720,
            true,
            &host,
            Some(&mut workspace),
        )
        .unwrap();
    assert_eq!(stats.hits, overlays.len());
    assert_eq!(
        fonts.shapes, shaped,
        "warm frame does not re-shape admitted labels"
    );
}

#[test]
#[ignore = "requires serial process-wide text admission"]
fn overlay_admission_evicts_history_and_remaps_the_earlier_workspace_group() {
    let state = crate::global_preferences_dialog_tests::state_with_preferences_open();
    let prepared =
        PreparedScene::from_native_preferences(&state.ui.global_preferences, 960, 720, 1.0);
    let mut current = prepared.menu_overlay_text_runs[0].clone();
    current.text = "workspace survives".into();
    current.rich_spans.clear();
    let mut history = current.clone();
    history.text = "obsolete history ".repeat(1000);
    let mut overlay = current.clone();
    overlay.text = "new overlay".into();
    let host = crate::text_gpu::budget::Budget::new(16 * 1024 * 1024);
    let mut fonts = Fonts::new(host.clone());
    let mut cache = TextBufferCache::default();
    cache.begin_frame(Profile::Overlay);
    cache
        .admitted_indices(
            &mut fonts,
            &[history, current.clone()],
            960,
            720,
            true,
            &host,
            None,
        )
        .unwrap();
    let limit = cache.published_bytes / 2;
    cache.begin_frame(Profile::Workspace);
    let (mut workspace, stats) = cache
        .admitted_indices(
            &mut fonts,
            std::slice::from_ref(&current),
            960,
            720,
            false,
            &host,
            None,
        )
        .unwrap();
    assert_eq!(stats.hits, 1);
    assert_eq!(&*workspace, &[1]);
    let filler = budget::Owner::new(0);
    let others: usize = crate::Renderer::text_cache_process_usage()
        .iter()
        .filter(|usage| usage.owner_id != cache.owner.id() && usage.owner_id != filler.id())
        .map(|usage| usage.bytes)
        .sum();
    filler
        .publish(32 * 1024 * 1024 - crate::Renderer::text_cache_registry_bytes() - others - limit);
    let (indices, _) = cache
        .admitted_indices(
            &mut fonts,
            std::slice::from_ref(&overlay),
            960,
            720,
            true,
            &host,
            Some(&mut workspace),
        )
        .unwrap();
    assert_eq!(&*workspace, &[0]);
    assert_eq!(&*indices, &[1]);
    assert_eq!(cache.entries.len(), 2);
    assert_eq!(cache.entries[workspace[0]].key.text, current.text);
    assert_eq!(cache.entries[indices[0]].key.text, overlay.text);
    assert_eq!(cache.published_bytes, cache.retained_payload_bytes());
    assert!(cache.published_bytes <= limit);
}

#[test]
#[ignore = "requires serial process-wide text admission"]
fn interrupted_relayout_cannot_be_reused_as_a_complete_layout() {
    let state = crate::global_preferences_dialog_tests::state_with_preferences_open();
    let prepared =
        PreparedScene::from_native_preferences(&state.ui.global_preferences, 960, 720, 1.0);
    let mut run = prepared.menu_overlay_text_runs[0].clone();
    run.text = "Required text survives an interrupted extent change".into();
    run.rich_spans.clear();
    run.layout_size = Some((300.0, 200.0));
    let host = crate::text_gpu::budget::Budget::new(16 * 1024 * 1024);
    let mut fonts = Fonts::new(host.clone());
    let mut cache = TextBufferCache::default();
    cache.begin_frame(Profile::Workspace);
    let (indices, _) = cache
        .admitted_indices(
            &mut fonts,
            std::slice::from_ref(&run),
            960,
            720,
            false,
            &host,
            None,
        )
        .unwrap();
    drop(indices);
    cache.finish_frame();
    cache.begin_frame(Profile::Workspace);
    run.layout_size = Some((150.0, 200.0));
    let index_bytes = crate::text_gpu::staging_vec::StagingVec::<usize>::capacity_bytes(1).unwrap();
    let filler = host.reserve(host.available() - index_bytes).unwrap();
    assert!(
        cache
            .admitted_indices(
                &mut fonts,
                std::slice::from_ref(&run),
                960,
                720,
                false,
                &host,
                None
            )
            .is_err()
    );
    assert!(
        cache.entries.is_empty(),
        "partial relayout cannot become an exact hit"
    );
    let usage = crate::Renderer::text_cache_process_usage()
        .into_iter()
        .find(|entry| entry.owner_id == cache.owner.id())
        .unwrap();
    assert_eq!(usage.constructing_bytes, 0);
    assert_eq!(usage.bytes, cache.retained_payload_bytes());
    drop(filler);
    let (indices, stats) = cache
        .admitted_indices(
            &mut fonts,
            std::slice::from_ref(&run),
            960,
            720,
            false,
            &host,
            None,
        )
        .unwrap();
    assert_eq!(stats.misses, 1);
    let snapshot = |cache: &TextBufferCache, index: usize| {
        cache.entries[index]
            .buffer
            .layout_runs()
            .map(|row| format!("{:?}", row.glyphs))
            .collect::<Vec<_>>()
    };
    let recovered = snapshot(&cache, indices[0]);
    let mut fresh = TextBufferCache::default();
    fresh.begin_frame(Profile::Workspace);
    let (expected, _) = fresh
        .admitted_indices(
            &mut fonts,
            std::slice::from_ref(&run),
            960,
            720,
            false,
            &host,
            None,
        )
        .unwrap();
    assert_eq!(recovered, snapshot(&fresh, expected[0]));
    assert_eq!(cache.published_bytes, cache.retained_payload_bytes());
}
