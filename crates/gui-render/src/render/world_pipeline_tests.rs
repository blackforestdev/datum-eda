//! Lazy world construction must preserve eager output and renderer replacement.
use super::*;

#[test]
#[ignore = "requires local GPU; run explicitly with the visual feature"]
fn world_pipelines_are_lazy_reused_and_recreated_without_pixel_changes() {
    for samples in [4, 8] {
        let mut lazy = hardware_renderer_with_features(
            960,
            720,
            wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES,
        );
        lazy.renderer = Renderer::new(&lazy.device, &lazy.queue, OUTPUT_FORMAT, samples).unwrap();
        let mut eager = OffscreenRenderer {
            device: lazy.device.clone(),
            queue: lazy.queue.clone(),
            renderer: Renderer::new(&lazy.device, &lazy.queue, OUTPUT_FORMAT, samples).unwrap(),
            width: lazy.width,
            height: lazy.height,
        };
        eager.renderer.prepare_world_pipelines(&eager.device);
        assert!(lazy.renderer.world_pipelines.get().is_none());

        let state = crate::global_preferences_dialog_tests::state_with_preferences_open();
        let dialog = PreparedScene::from_native_preferences(
            &state.ui.global_preferences,
            lazy.width,
            lazy.height,
            1.0,
        )
        .unwrap();
        assert!(dialog.is_overlay_only());
        let dialog_pixels = capture(&mut lazy, &dialog);
        assert!(
            lazy.renderer.world_pipelines.get().is_none(),
            "overlay constructed unused world pipelines"
        );
        assert!(
            dialog_pixels == capture(&mut eager, &dialog),
            "lazy dialog differs from eager initialization at {samples}xAA"
        );

        let state = crate::gpu_surface_pass::board_fixture_state();
        let retained =
            RetainedScene::from_workspace_for_surface(&state, lazy.width, lazy.height, 1.0);
        let scene = PreparedScene::from_workspace_with_terminal_renderer(
            &state,
            lazy.width,
            lazy.height,
            1.0,
            CameraState::fit_to_bounds(&state.scene.bounds),
            &retained,
            &[],
            None,
            true,
        )
        .unwrap();
        assert!(!scene.is_overlay_only());
        assert!(
            !retained.all_draw_commands().is_empty(),
            "fixture must exercise world drawing"
        );
        let actual = capture_retained(&mut lazy, &scene, &retained);
        assert!(
            actual == capture_retained(&mut eager, &scene, &retained),
            "cold world pixels differ at {samples}xAA"
        );
        let world = lazy
            .renderer
            .world_pipelines
            .get()
            .expect("world pipelines initialized");
        let quads = world.quads.clone();
        let strokes = world.strokes.clone();
        assert!(actual == capture_retained(&mut lazy, &scene, &retained));
        let world = lazy.renderer.world_pipelines.get().unwrap();
        assert_eq!(quads, world.quads, "warm frame rebuilt quad pipeline");
        assert_eq!(strokes, world.strokes, "warm frame rebuilt stroke pipeline");

        lazy.renderer = lazy
            .renderer
            .recreate_for_device(&lazy.device, &lazy.queue, OUTPUT_FORMAT, samples)
            .unwrap();
        assert!(
            lazy.renderer.world_pipelines.get().is_none(),
            "replacement retained old pipelines"
        );
        assert!(dialog_pixels == capture(&mut lazy, &dialog));
        assert!(lazy.renderer.world_pipelines.get().is_none());
        assert!(actual == capture_retained(&mut lazy, &scene, &retained));
        let replacement = lazy.renderer.world_pipelines.get().unwrap();
        assert_ne!(quads, replacement.quads);
        assert_ne!(strokes, replacement.strokes);
    }
}

#[test]
#[ignore = "requires local GPU; retained triangle expansion pixel parity"]
fn compact_retained_triangles_match_degenerate_expansion_pixels() {
    let state = crate::gpu_surface_pass::board_fixture_state();
    let mut renderer = hardware_renderer_with_features(
        960,
        720,
        wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES,
    );
    renderer.renderer = Renderer::new(&renderer.device, &renderer.queue, OUTPUT_FORMAT, 8).unwrap();
    for scale in [1.0, 1.5] {
        let compact = RetainedScene::from_workspace_for_surface(&state, 960, 720, scale);
        assert!(!compact.world_vertices.is_empty());
        let mut expanded = compact.clone();
        // Reintroduce the old triangle-as-quad encoding without changing any
        // nondegenerate triangle, color, winding, layer or stroke command.
        expanded.world_vertices = compact
            .world_vertices
            .chunks_exact(3)
            .flat_map(|t| [t[0], t[1], t[2], t[0], t[2], t[2]])
            .collect::<Vec<_>>()
            .into();
        let mut commands = compact.all_draw_commands().to_vec();
        for command in &mut commands {
            if let crate::RetainedDrawCommand::Quads { range, .. } = command {
                *range = range.start * 2..range.end * 2;
            }
        }
        expanded.draw_commands = commands.into();
        let prepare = |retained: &RetainedScene| {
            PreparedScene::from_workspace_with_terminal_renderer(
                &state,
                960,
                720,
                scale,
                CameraState::fit_to_bounds(&state.scene.bounds),
                retained,
                &[],
                None,
                true,
            )
            .unwrap()
        };
        let expected = capture_retained(&mut renderer, &prepare(&expanded), &expanded);
        let actual = capture_retained(&mut renderer, &prepare(&compact), &compact);
        assert!(
            expected == actual,
            "triangle expansion changed pixels at scale {scale}"
        );
        assert!(actual == capture_retained(&mut renderer, &prepare(&compact), &compact));
        let mut absent = compact.clone();
        absent.world_vertices = Vec::new().into();
        absent.draw_commands = compact
            .all_draw_commands()
            .iter()
            .filter(|c| !matches!(c, crate::RetainedDrawCommand::Quads { .. }))
            .cloned()
            .collect::<Vec<_>>()
            .into();
        assert!(
            actual != capture_retained(&mut renderer, &prepare(&absent), &absent),
            "fixture must expose missing world triangles"
        );
    }
}
