use super::*;

#[test]
#[ignore = "requires local GPU; shared CPU/GPU staging ownership"]
fn pixels_are_charged_until_copied_and_pressure_preserves_pending_content() {
    let instance = wgpu::Instance::default();
    let adapter = pollster::block_on(instance.request_adapter(&Default::default())).unwrap();
    let (device, queue) = pollster::block_on(adapter.request_device(&Default::default())).unwrap();
    let mut fonts = crate::load_datum_fonts();
    let mut buffer = glyphon::Buffer::new(&mut fonts, glyphon::Metrics::new(18.0, 22.0));
    buffer.set_text(
        &mut fonts,
        "ABCDEF",
        &crate::text_attrs(crate::TextFace::Ui),
        glyphon::Shaping::Basic,
        None,
    );
    buffer.shape_until_scroll(&mut fonts, false);
    let keys: Vec<_> = buffer
        .layout_runs()
        .next()
        .unwrap()
        .glyphs
        .iter()
        .map(|g| g.physical((0.0, 0.0), 1.0).cache_key)
        .collect();
    let mut raster = SwashCache::new();
    let host = crate::text_gpu::budget::Budget::new(16 * 1024 * 1024);
    let process = crate::text_gpu::budget::staging_process();
    let baseline = process.used();
    let mut atlas = Atlas::with_staging_budget(&device, host.clone());
    atlas
        .glyph(&device, &queue, &mut fonts, &mut raster, keys[0])
        .unwrap();
    let cpu = atlas.pending_cpu_bytes();
    assert!(cpu > 0);
    let metadata = atlas.pending_metadata_bytes();
    let pages = atlas.page_metadata_bytes() + atlas.lookup_metadata_bytes();
    assert!(pages > 0);
    assert!(metadata > 0);
    assert_eq!(
        host.used(),
        cpu + metadata + pages + raster.reserved_bytes()
    );
    assert_eq!(
        process.used(),
        baseline + cpu + metadata + pages + raster.reserved_bytes()
    );
    let padded = atlas.pending_staging_bytes();
    // Exhaust required storage, after retiring the newly evictable scratch.
    raster.clear();
    let filler = host.reserve(host.available()).unwrap();
    let error = atlas
        .glyph(&device, &queue, &mut fonts, &mut raster, keys[1])
        .unwrap_err();
    assert!(error.is::<UploadRequired>());
    assert_eq!(atlas.pending_cpu_bytes(), cpu);
    assert_eq!(atlas.pending_staging_bytes(), padded);
    assert_eq!(
        process.used(),
        baseline + cpu + metadata + pages + raster.reserved_bytes(),
        "failed admission rolls back"
    );
    let rasterizations = atlas.uploads.rasterizations;
    assert!(
        atlas
            .glyph(&device, &queue, &mut fonts, &mut raster, keys[0])
            .unwrap()
            .is_some()
    );
    assert_eq!(
        atlas.uploads.rasterizations, rasterizations,
        "known glyph survives pressure"
    );
    drop(filler);
    let mut batch = atlas.flush_uploads(&device, &[]).unwrap().unwrap();
    assert_eq!(atlas.pending_cpu_bytes(), 0);
    assert_eq!(atlas.pending_metadata_bytes(), 0);
    assert_eq!(
        host.used(),
        padded
            + pages
            + raster.reserved_bytes()
            + crate::text_gpu::staging_vec::StagingVec::<Tracked<wgpu::Buffer>>::capacity_bytes(1)
                .unwrap(),
        "CPU pixels retire after staging copy is built"
    );
    queue.submit([batch.command()]);
    batch.hold(&queue);
    device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
    assert_eq!(host.used(), pages + raster.reserved_bytes());
    atlas
        .glyph(&device, &queue, &mut fonts, &mut raster, keys[1])
        .unwrap();
    assert!(host.used() > 0);
    for key in &keys[2..5] {
        atlas
            .glyph(&device, &queue, &mut fonts, &mut raster, *key)
            .unwrap();
    }
    assert_eq!(
        atlas.pending_uploads.len(),
        atlas.pending_uploads.capacity()
    );
    let old_metadata = atlas.pending_metadata_bytes();
    let old_pixels = atlas.pending_cpu_bytes();
    let old_copies = atlas.pending_staging_bytes();
    // Exhaust required storage, after retiring the newly evictable scratch.
    raster.clear();
    let filler = host.reserve(host.available()).unwrap();
    let error = atlas
        .glyph(&device, &queue, &mut fonts, &mut raster, keys[5])
        .unwrap_err();
    assert!(error.is::<UploadRequired>());
    assert_eq!(atlas.pending_metadata_bytes(), old_metadata);
    assert_eq!(atlas.pending_cpu_bytes(), old_pixels);
    assert_eq!(atlas.pending_staging_bytes(), old_copies);
    drop(filler);
    atlas
        .glyph(&device, &queue, &mut fonts, &mut raster, keys[5])
        .unwrap();
    assert!(atlas.pending_metadata_bytes() > old_metadata);
    assert_eq!(
        host.used(),
        atlas.pending_cpu_bytes()
            + atlas.pending_metadata_bytes()
            + atlas.page_metadata_bytes()
            + atlas.lookup_metadata_bytes()
            + raster.reserved_bytes()
    );
    assert_eq!(process.used(), baseline + host.used());
    atlas.repack();
    assert_eq!(atlas.pending_metadata_bytes(), 0);
    assert_eq!(host.used(), pages + raster.reserved_bytes());
    drop(atlas);
    assert_eq!(host.used(), raster.reserved_bytes());
    drop(raster);
    assert_eq!(host.used(), 0);
    assert_eq!(process.used(), baseline);
}
