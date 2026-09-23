use super::*;

#[test]
fn shelf_rejection_preserves_previous_and_remaining_space() {
    let mut shelves = Shelves::default();
    assert_eq!(shelves.reserve([7, 3], 10), Some([0, 0]));
    assert_eq!(shelves.reserve([4, 8], 10), None);
    assert_eq!(shelves.reserve([3, 2], 10), Some([7, 0]));
    assert_eq!(shelves.reserve([10, 7], 10), Some([0, 3]));
    assert_eq!(shelves.reserve([1, 1], 10), None);
}

#[test]
#[ignore = "requires local GPU; local atlas retirement admission"]
fn local_atlas_limit_counts_retired_pages_before_replacement() {
    let instance = wgpu::Instance::default();
    let adapter = pollster::block_on(instance.request_adapter(&Default::default())).unwrap();
    let (device, queue) = pollster::block_on(adapter.request_device(&Default::default())).unwrap();
    let mut fonts = crate::load_datum_fonts();
    let mut buffer = glyphon::Buffer::new(&mut fonts, glyphon::Metrics::new(18.0, 22.0));
    buffer.set_text(
        &mut fonts,
        "A",
        &crate::text_attrs(crate::TextFace::Ui),
        glyphon::Shaping::Basic,
        None,
    );
    buffer.shape_until_scroll(&mut fonts, false);
    let key = buffer.layout_runs().next().unwrap().glyphs[0]
        .physical((0.0, 0.0), 1.0)
        .cache_key;
    let mut raster = SwashCache::new();
    let mut atlas = Atlas::new(&device);
    atlas.set_test_limit(1024 * 1024);
    let local = atlas.local_budget.clone();
    atlas
        .glyph(&device, &queue, &mut fonts, &mut raster, key)
        .unwrap();
    let held = atlas.submission_refs();
    drop(atlas.reset());
    assert_eq!(atlas.retained_texture_bytes(), 0);
    assert_eq!(local.used(), 1024 * 1024);
    assert!(
        atlas
            .glyph(&device, &queue, &mut fonts, &mut raster, key)
            .is_err()
    );
    assert!(atlas.pages.is_empty());
    drop(held);
    assert_eq!(local.used(), 0);
    atlas
        .glyph(&device, &queue, &mut fonts, &mut raster, key)
        .unwrap();
    assert_eq!(local.used(), 1024 * 1024);
    drop(atlas);
    assert_eq!(local.used(), 0);
}

#[test]
#[ignore = "requires local GPU; run explicitly with the visual feature"]
fn owned_atlas_process_admission_precedes_allocation_and_waits_for_retirement() {
    let instance = wgpu::Instance::default();
    let adapter = pollster::block_on(instance.request_adapter(&Default::default())).unwrap();
    let (device, queue) = pollster::block_on(adapter.request_device(&Default::default())).unwrap();
    let mut fonts = crate::load_datum_fonts();
    let mut buffer = glyphon::Buffer::new(&mut fonts, glyphon::Metrics::new(18.0, 22.0));
    buffer.set_text(
        &mut fonts,
        "A",
        &crate::text_attrs(crate::TextFace::Ui),
        glyphon::Shaping::Basic,
        None,
    );
    buffer.shape_until_scroll(&mut fonts, false);
    let key = buffer.layout_runs().next().unwrap().glyphs[0]
        .physical((0.0, 0.0), 1.0)
        .cache_key;
    let budget = super::super::budget::Budget::new(2 * 1024 * 1024);
    let mut raster = SwashCache::new();
    let mut atlases: Vec<_> = (0..3)
        .map(|_| {
            let mut atlas = Atlas::new(&device);
            atlas.texture_budget = budget.clone();
            atlas
        })
        .collect();
    for atlas in &mut atlases[..2] {
        atlas
            .glyph(&device, &queue, &mut fonts, &mut raster, key)
            .unwrap();
    }
    assert_eq!(budget.used(), 2 * 1024 * 1024);
    assert!(
        atlases[2]
            .glyph(&device, &queue, &mut fonts, &mut raster, key)
            .is_err()
    );
    assert!(
        atlases[2].pages.is_empty(),
        "reject before texture allocation"
    );
    let held = atlases[0].submission_refs();
    let retired = atlases.remove(0);
    drop(retired);
    assert_eq!(budget.used(), 2 * 1024 * 1024);
    assert!(
        atlases[1]
            .glyph(&device, &queue, &mut fonts, &mut raster, key)
            .is_err()
    );
    drop(held);
    assert_eq!(budget.used(), 1024 * 1024);
    atlases[1]
        .glyph(&device, &queue, &mut fonts, &mut raster, key)
        .unwrap();
    assert_eq!(budget.used(), 2 * 1024 * 1024);
    drop(atlases);
    assert_eq!(budget.used(), 0);
}

#[test]
#[ignore = "requires local GPU; run explicitly with the visual feature"]
fn owned_atlas_upload_reuse_reset_and_retirement_handoff() {
    let instance = wgpu::Instance::default();
    let adapter = pollster::block_on(instance.request_adapter(&Default::default())).unwrap();
    let (device, queue) = pollster::block_on(adapter.request_device(&Default::default())).unwrap();
    let mut fonts = crate::load_datum_fonts();
    let mut buffer = glyphon::Buffer::new(&mut fonts, glyphon::Metrics::new(18.0, 22.0));
    buffer.set_text(
        &mut fonts,
        "A",
        &crate::text_attrs(crate::TextFace::Ui),
        glyphon::Shaping::Basic,
        None,
    );
    buffer.shape_until_scroll(&mut fonts, false);
    let key = buffer.layout_runs().next().unwrap().glyphs[0]
        .physical((0.0, 0.0), 1.0)
        .cache_key;
    let mut raster = SwashCache::new();
    let reference = raster.get_image_uncached(&mut fonts, key).unwrap();
    let mut atlas = Atlas::new(&device);
    let budget = super::super::budget::Budget::new(2 * 1024 * 1024);
    atlas.texture_budget = budget.clone();
    let first = atlas
        .glyph(&device, &queue, &mut fonts, &mut raster, key)
        .unwrap()
        .unwrap();
    assert_eq!(
        first.size,
        [reference.placement.width, reference.placement.height]
    );
    assert_eq!(
        first.bearing,
        [reference.placement.left, reference.placement.top]
    );
    atlas.flush_for_test(&device, &queue);
    assert_eq!(atlas.uploads.bytes, reference.data.len() as u64);
    assert_eq!(atlas.uploads.writes, 1);
    let uploads = atlas.uploads;
    let allocation = atlas.pages[first.page].texture.id();
    let retained = atlas.retained_texture_bytes();
    let warm = atlas
        .glyph(&device, &queue, &mut fonts, &mut raster, key)
        .unwrap()
        .unwrap();
    assert_eq!(warm.origin, first.origin);
    assert_eq!(
        atlas.uploads, uploads,
        "warm glyph must not rasterize or upload"
    );
    let epoch = atlas.generation;
    let retiring = atlas.reset();
    assert_eq!(atlas.generation, epoch + 1);
    assert_eq!(atlas.retained_texture_bytes(), 0);
    assert_eq!(
        retiring
            .iter()
            .map(|page| page.texture.width() as u64
                * page.texture.height() as u64
                * page.texture.format().block_copy_size(None).unwrap() as u64)
            .sum::<u64>(),
        retained
    );
    let replacement = atlas
        .glyph(&device, &queue, &mut fonts, &mut raster, key)
        .unwrap()
        .unwrap();
    assert_ne!(atlas.pages[replacement.page].texture.id(), allocation);
    atlas.flush_for_test(&device, &queue);
    assert_eq!(atlas.uploads.writes, 2);
    assert_eq!(atlas.uploads.bytes, 2 * reference.data.len() as u64);
    assert_eq!(atlas.retained_texture_bytes(), retained);
    // Both sets remain alive until all queued writes complete. Reset does
    // not silently turn a retired allocation into released accounting.
    let submitted = retiring
        .iter()
        .map(|page| page.texture.submission_ref())
        .collect();
    drop(retiring);
    let records = atlas.owner.records();
    assert!(
        records
            .iter()
            .find(|record| record.id == allocation)
            .unwrap()
            .retiring
    );
    assert_eq!(
        records
            .iter()
            .filter(|record| record.kind == Kind::Texture)
            .map(|record| record.bytes)
            .sum::<u64>(),
        2 * retained
    );
    assert_eq!(budget.used(), 2 * retained);
    let submission = queue.submit([]);
    super::super::lifetime::hold_until_done(&queue, submitted);
    device
        .poll(wgpu::PollType::Wait {
            submission_index: Some(submission),
            timeout: None,
        })
        .unwrap();
    let records = atlas.owner.records();
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].bytes, retained);
    assert_ne!(records[0].id, allocation);
    assert_eq!(budget.used(), retained);
    drop(atlas);
    assert_eq!(budget.used(), 0);
}
