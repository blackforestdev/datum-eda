//! CPU cache transfer and new-device glyph preparation share one recovery proof.
use super::*;

#[test]
#[ignore = "requires local GPU; run explicitly with the visual feature"]
fn shared_atlas_retry_and_cache_reindex_refresh_workspace_glyphs() {
    let state = crate::global_preferences_dialog_tests::state_with_preferences_open();
    let mut renderer = hardware_renderer(960, 720);
    let mut prepared = renderer
        .renderer
        .prepare_native_preferences_scrolled(
            &state.ui.global_preferences,
            960,
            720,
            1.0,
            &mut datum_gui_viewport::scroll::ScrollViewport::default(),
            None,
        )
        .unwrap();
    let mut workspace = prepared.menu_overlay_text_runs[0].clone();
    workspace.text = "Workspace glyph residency".into();
    workspace.rich_spans.clear();
    workspace.x = 20.0;
    workspace.y = 680.0;
    workspace.clip_bounds = None;
    let mut overlay = workspace.clone();
    overlay.text = "Overlay glyph residency".into();
    overlay.x = 600.0;
    overlay.y = 20.0;
    prepared.text_runs = vec![workspace.clone()];
    prepared.menu_overlay_text_runs = vec![overlay.clone()];
    prepared.menu_overlay_vertices = crate::gpu_data::quads_to_vertices(&[crate::Quad::from_rect(
        crate::RectPx {
            x: 500.0,
            y: 0.0,
            width: 460.0,
            height: 720.0,
        },
        [0.05; 3],
    )]);
    let cold = capture(&mut renderer, &prepared);
    // Compare the shared immutable pipeline against the former independent
    // pipeline construction while both workspace and overlay batches are live.
    let mut separate = hardware_renderer(960, 720);
    separate.renderer.menu_overlay_text_renderer = crate::text_gpu::Draw::new(
        &separate.device,
        &separate.renderer.atlas,
        OUTPUT_FORMAT,
        separate.renderer.msaa_samples,
        separate.renderer.screen_budget.clone(),
    );
    assert!(cold == capture(&mut separate, &prepared));
    let identities = |r: &Renderer| {
        (
            r.font_cpu_usage().allocation.owner_id,
            r.text_cpu_usage().owner_id,
            r.text_cache_key_usage().owner_id,
            r.measurement_cpu_usage().unwrap().allocation.owner_id,
        )
    };
    let old_ids = identities(&renderer.renderer);
    let old_keys = renderer.renderer.text_cache_key_usage();
    assert!(
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            separate
                .renderer
                .commit_cpu_recovery_from(&mut renderer.renderer);
        }))
        .is_err(),
        "unrelated host cannot adopt another host's CPU budget owners"
    );
    assert_eq!(identities(&renderer.renderer), old_ids);
    assert_eq!(renderer.renderer.text_cache_key_usage(), old_keys);
    let old_meshes = renderer.renderer.control_mesh_usage();
    assert!(old_keys.entries > 0 && old_meshes.entries > 0);
    // Staging and abandoning a replacement must not consume the live caches.
    let abandoned = renderer
        .renderer
        .recreate_for_device(
            &separate.device,
            &separate.queue,
            OUTPUT_FORMAT,
            renderer.renderer.msaa_samples,
        )
        .unwrap();
    drop(abandoned);
    assert_eq!(identities(&renderer.renderer), old_ids);
    assert_eq!(renderer.renderer.text_cache_key_usage(), old_keys);
    assert_eq!(renderer.renderer.control_mesh_usage(), old_meshes);

    let mut replacement = renderer
        .renderer
        .recreate_for_device(
            &separate.device,
            &separate.queue,
            OUTPUT_FORMAT,
            renderer.renderer.msaa_samples,
        )
        .unwrap();
    let gpu_owner = replacement.resource_owner_id();
    replacement.commit_cpu_recovery_from(&mut renderer.renderer);
    assert_eq!(identities(&replacement), old_ids);
    assert_eq!(replacement.text_cache_key_usage(), old_keys);
    assert_eq!(replacement.control_mesh_usage(), old_meshes);
    assert_eq!(replacement.resource_owner_id(), gpu_owner);
    assert_eq!(replacement.text_preparation.workspace_prepares, 0);
    assert_eq!(replacement.text_preparation.overlay_prepares, 0);
    // Reuse the independently created device and its capture target. No old
    // GPU target, atlas or preparation can supply the replacement pixels.
    separate.renderer = replacement;
    renderer = separate;
    let builds = renderer.renderer.control_mesh_build_count();
    let _ = renderer
        .renderer
        .prepare_native_preferences_scrolled(
            &state.ui.global_preferences,
            960,
            720,
            1.0,
            &mut datum_gui_viewport::scroll::ScrollViewport::default(),
            None,
        )
        .unwrap();
    assert_eq!(renderer.renderer.control_mesh_build_count(), builds);
    let (_, cached) = renderer.renderer.text_buffers.indices(
        &mut renderer.renderer.font_system,
        &prepared.text_runs,
        960,
        720,
    );
    assert_eq!(cached.misses, 0, "new device reuses retained CPU shaping");
    assert!(cold == capture(&mut renderer, &prepared));
    assert_eq!(
        renderer.renderer.text_cache_key_usage().owner_id,
        old_keys.owner_id
    );
    assert_eq!(
        renderer.renderer.text_cache_key_usage().entries,
        old_keys.entries
    );
    assert_eq!(renderer.renderer.text_preparation.workspace_prepares, 1);

    let count = renderer.renderer.text_preparation.workspace_prepares;
    assert!(cold == capture(&mut renderer, &prepared));
    assert_eq!(
        renderer.renderer.text_preparation.workspace_prepares, count,
        "warm workspace preparation stays skipped"
    );
    renderer.renderer.text_preparation.force_overlay_errors(1);
    assert!(cold == capture(&mut renderer, &prepared));
    assert_eq!(
        renderer.renderer.text_preparation.workspace_prepares,
        count + 1,
        "retry must re-protect workspace glyphs before overlay allocation"
    );

    renderer.renderer.text_preparation.force_overlay_errors(2);
    let failed = renderer.renderer.prepare_frame_text(
        &renderer.device,
        &renderer.queue,
        &prepared,
        960,
        720,
        false,
    );
    assert!(failed.is_err());
    assert!(renderer.renderer.text_preparation.is_invalid());
    let count = renderer.renderer.text_preparation.workspace_prepares;
    assert!(cold == capture(&mut renderer, &prepared));
    assert_eq!(
        renderer.renderer.text_preparation.workspace_prepares,
        count + 1
    );

    // Evict entry zero but retain entry one, which moves into the same index.
    // The new workspace hits that existing buffer with the old placement key.
    renderer
        .renderer
        .text_buffers
        .begin_frame(crate::text_buffer_cache::Profile::Workspace);
    renderer.renderer.text_buffers.indices(
        &mut renderer.renderer.font_system,
        &[overlay],
        960,
        720,
    );
    renderer
        .renderer
        .text_buffers
        .begin_frame(crate::text_buffer_cache::Profile::Workspace);
    prepared.text_runs[0].text = "Overlay glyph residency".into();
    let (indices, stats) = renderer.renderer.text_buffers.indices(
        &mut renderer.renderer.font_system,
        &prepared.text_runs,
        960,
        720,
    );
    assert_eq!(indices, vec![0]);
    assert_eq!(
        stats.misses, 0,
        "collision is a retained cache hit, not new shaping"
    );
    let count = renderer.renderer.text_preparation.workspace_prepares;
    let reindexed = capture(&mut renderer, &prepared);
    assert_eq!(
        renderer.renderer.text_preparation.workspace_prepares,
        count + 1
    );
    assert!(
        reindexed != cold,
        "workspace changed to the retained overlay label"
    );
    renderer.renderer.text_preparation = Default::default();
    assert!(
        reindexed == capture(&mut renderer, &prepared),
        "reindex reuse matches forced fresh glyph preparation"
    );
}
