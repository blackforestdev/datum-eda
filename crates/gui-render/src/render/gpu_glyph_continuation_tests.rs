//! Actual renderer proof for oversized atlas uploads in both native render paths.
use super::*;

#[test]
#[ignore = "requires local GPU; glyph upload continuation and current-state rendering"]
fn oversized_atlas_uploads_yield_preserve_preparation_and_render_latest_text() {
    for overlay_only in [true, false] {
        let state = crate::global_preferences_dialog_tests::state_with_preferences_open();
        let mut prepared =
            PreparedScene::from_native_preferences(&state.ui.global_preferences, 960, 720, 1.0);
        let template = prepared.menu_overlay_text_runs[0].clone();
        let runs: Vec<_> = (0..6)
            .map(|i| {
                let mut run = template.clone();
                run.text = "W".into();
                run.rich_spans.clear();
                run.size = 1800.0 + i as f32 * 20.0;
                run.x = i as f32 * 20.0;
                run.y = -900.0;
                run.clip_bounds = None;
                run.layout_size = Some((4000.0, 4000.0));
                run
            })
            .collect();
        prepared.menu_overlay_text_runs = runs;
        if !overlay_only {
            prepared.text_runs = vec![template];
        }
        let retained = RetainedScene::empty();
        let mut renderer = hardware_renderer(960, 720);
        let target = renderer.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("glyph-continuation-proof"),
            size: wgpu::Extent3d {
                width: 960,
                height: 720,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: OUTPUT_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        let view = target.create_view(&Default::default());
        let mut submissions = 0;
        let first = renderer
            .renderer
            .render_with_submission(
                &renderer.device,
                &renderer.queue,
                &view,
                &prepared,
                &retained,
                None,
                960,
                720,
                &mut |_| submissions += 1,
            )
            .unwrap();
        assert!(!first, "oversized cold atlas must yield before drawing");
        assert_eq!(submissions, 1);
        assert!(
            renderer.renderer.atlas.has_pending_uploads(),
            "fixture must exceed one chunk"
        );
        assert!(
            renderer.renderer.upload_staging_reserved_bytes()
                - renderer.renderer.layout_scratch_reserved_bytes()
                - renderer.renderer.pending_glyph_pixel_bytes()
                <= 4 * 1024 * 1024
        );
        let first_rasters = renderer.renderer.atlas.rasterization_count();
        assert!(renderer.renderer.pending_glyph_pixel_bytes() > 0);
        assert!(renderer.renderer.upload_staging_reserved_bytes() <= 16 * 1024 * 1024);
        let generation = renderer.renderer.atlas.generation;
        renderer
            .device
            .poll(wgpu::PollType::wait_indefinitely())
            .unwrap();
        let pending = renderer.renderer.atlas.pending_staging_bytes();
        let filler = renderer
            .renderer
            .atlas
            .staging_budget
            .reserve(renderer.renderer.atlas.staging_budget.available())
            .unwrap();
        assert!(
            renderer
                .renderer
                .render_with_submission(
                    &renderer.device,
                    &renderer.queue,
                    &view,
                    &prepared,
                    &retained,
                    None,
                    960,
                    720,
                    &mut |_| panic!("refused copy submitted")
                )
                .is_err()
        );
        assert_eq!(renderer.renderer.atlas.pending_staging_bytes(), pending);
        drop(filler);
        // A color-only change during upload must affect the eventual current frame,
        // without rasterizing again or invalidating already copied page contents.
        prepared.menu_overlay_text_runs[0].color = [1.0, 0.0, 0.0];
        let mut complete = false;
        for _ in 0..8 {
            complete = renderer
                .renderer
                .render_with_submission(
                    &renderer.device,
                    &renderer.queue,
                    &view,
                    &prepared,
                    &retained,
                    None,
                    960,
                    720,
                    &mut |_| submissions += 1,
                )
                .unwrap();
            renderer
                .device
                .poll(wgpu::PollType::wait_indefinitely())
                .unwrap();
            assert_eq!(
                renderer.renderer.upload_staging_reserved_bytes(),
                renderer.renderer.layout_scratch_reserved_bytes()
                    + renderer.renderer.pending_glyph_pixel_bytes()
            );
            if complete {
                break;
            }
        }
        assert!(
            complete,
            "must finish, not repack/restart after every yielded turn"
        );
        assert!(submissions >= 3);
        assert_eq!(renderer.renderer.atlas.generation, generation);
        assert!(
            renderer.renderer.atlas.rasterization_count() > first_rasters,
            "raster preparation must resume after the bounded pending queue drains"
        );
        assert!((2..=4).contains(&renderer.renderer.text_preparation.overlay_prepares));
        assert_eq!(renderer.renderer.pending_glyph_pixel_bytes(), 0);
        let pixels = capture(&mut renderer, &prepared);
        let mut fresh = hardware_renderer(960, 720);
        assert!(pixels == capture(&mut fresh, &prepared));
        let prepares = renderer.renderer.text_preparation.overlay_prepares;
        assert!(pixels == capture(&mut renderer, &prepared));
        assert_eq!(
            renderer.renderer.text_preparation.overlay_prepares,
            prepares
        );
    }
}

#[test]
#[ignore = "requires local GPU and serial process-wide cache admission"]
fn text_cache_pressure_refuses_before_glyphs_and_retries_current_frame() {
    for overlay_only in [true, false] {
        let state = crate::global_preferences_dialog_tests::state_with_preferences_open();
        let mut prepared =
            PreparedScene::from_native_preferences(&state.ui.global_preferences, 960, 720, 1.0);
        if !overlay_only {
            prepared.text_runs = vec![prepared.menu_overlay_text_runs[0].clone()];
        }
        let mut renderer = hardware_renderer(960, 720);
        let target = renderer.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("text-admission-proof"),
            size: wgpu::Extent3d {
                width: 960,
                height: 720,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: OUTPUT_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        let view = target.create_view(&Default::default());
        let before = capture(&mut renderer, &prepared);
        let overlay_prepares = renderer.renderer.text_preparation.overlay_prepares;
        let workspace_prepares = renderer.renderer.text_preparation.workspace_prepares;
        let atlas_bytes = renderer.renderer.text_atlas_reserved_bytes();
        let filler = crate::text_buffer_cache::budget::Owner::new(32 * 1024 * 1024);
        let error = renderer
            .renderer
            .render_with_submission(
                &renderer.device,
                &renderer.queue,
                &view,
                &prepared,
                &RetainedScene::empty(),
                None,
                960,
                720,
                &mut |_| panic!("over-budget frame submitted"),
            )
            .unwrap_err();
        assert!(
            error
                .to_string()
                .contains("required text layout exceeds CPU cache admission")
        );
        assert_eq!(
            renderer.renderer.text_preparation.overlay_prepares,
            overlay_prepares
        );
        assert_eq!(
            renderer.renderer.text_preparation.workspace_prepares,
            workspace_prepares
        );
        assert_eq!(renderer.renderer.text_atlas_reserved_bytes(), atlas_bytes);
        let usage = renderer.renderer.text_cache_key_usage();
        assert_eq!(usage.entries, 0);
        assert_eq!(usage.key_text_bytes, 0);
        assert_eq!(usage.shaped_payload_bytes, 0);
        assert_eq!(renderer.renderer.layout_scratch_reserved_bytes(), 0);
        assert!(renderer.renderer.text_preparation.is_invalid());
        drop(filler);
        prepared.menu_overlay_text_runs[0].text.push_str(" latest");
        prepared.menu_overlay_text_runs[0].color = [1.0, 0.0, 0.0];
        let pixels = capture(&mut renderer, &prepared);
        assert_ne!(pixels, before);
        let mut fresh = hardware_renderer(960, 720);
        assert!(pixels == capture(&mut fresh, &prepared));
    }
}
