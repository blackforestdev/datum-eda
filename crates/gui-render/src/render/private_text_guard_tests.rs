use crate::cpu_alloc::calls::Overrun;

#[test]
#[ignore = "requires local GPU; production overrun must not become atlas retry"]
fn production_overrun_stops_preparation_before_retry_or_publication() {
    let instance = wgpu::Instance::default();
    let adapter = pollster::block_on(instance.request_adapter(&Default::default())).unwrap();
    let (device, queue) = pollster::block_on(adapter.request_device(&Default::default())).unwrap();
    let mut renderer =
        crate::Renderer::new(&device, &queue, wgpu::TextureFormat::Rgba8UnormSrgb, 4).unwrap();
    let state = crate::global_preferences_dialog_tests::state_with_preferences_open();
    let mut prepared =
        crate::PreparedScene::from_native_preferences(&state.ui.global_preferences, 960, 720, 1.0);
    let mut run = prepared.menu_overlay_text_runs[0].clone();
    run.text = "W".into();
    run.rich_spans.clear();
    run.clip_bounds = None;
    run.x = 0.0;
    run.y = 0.0;
    run.size = 512.0;
    prepared.text_runs = vec![run];
    prepared.menu_overlay_text_runs.clear();
    renderer
        .prepare_frame_text(&device, &queue, &prepared, 960, 720, false)
        .unwrap();
    assert!(renderer.atlas.pending_cpu_bytes() > 64 * 1024);
    renderer.atlas.repack();
    renderer.swash_cache.clear();
    let host = renderer.atlas.staging_budget.clone();
    let filler = host.reserve(host.available() - 64 * 1024).unwrap();
    let retries = renderer.text_preparation.atlas_retries;
    let error = renderer
        .prepare_frame_text(&device, &queue, &prepared, 960, 720, false)
        .expect_err("large private raster output must exceed remaining memory");
    assert!(error.is::<Overrun>(), "{error}");
    assert_eq!(
        renderer.text_preparation.atlas_retries, retries,
        "an overrun is not permission to trim and continue the rejected batch"
    );
    assert!(renderer.text_preparation.prepared.is_none());
    assert!(renderer.text_preparation.overlay_prepared.is_none());
    assert_eq!(prepared.text_runs[0].text, "W");
    assert_eq!(renderer.atlas.pending_cpu_bytes(), 0);
    drop(filler);
    renderer
        .prepare_frame_text(&device, &queue, &prepared, 960, 720, false)
        .unwrap();
    assert!(renderer.text_preparation.prepared.is_some());
    assert!(renderer.atlas.pending_cpu_bytes() > 64 * 1024);
}
