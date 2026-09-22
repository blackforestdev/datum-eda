//! Native New Project retains controls without preparing an invisible workspace.
use super::*;
use crate::HitTarget;

#[test]
#[ignore = "requires local GPU; New Project renderer ownership proof"]
fn new_project_controls_retain_meshes_and_match_cold_composition() {
    let mut state = datum_gui_protocol::load_fixture_workspace_state();
    state.ui.global_preferences.open = false;
    state.ui.new_project.open = true;
    state.ui.new_project.project_name = "sensor-node".into();
    state.ui.new_project.destination = "/tmp/sensor-node".into();
    let retained = RetainedScene::from_workspace(&state, 960, 720);
    let camera = CameraState::fit_to_bounds(&state.scene.bounds);
    let mut renderer = hardware_renderer(960, 720);
    for scale in [1.0, 1.5] {
        let cold =
            renderer
                .renderer
                .prepare_native_new_project(&state.ui.new_project, 960, 720, scale);
        assert!(
            cold.is_overlay_only(),
            "native New Project must omit every hidden workspace contribution"
        );
        let builds = renderer.renderer.control_meshes.builds;
        assert!(builds > 0, "the production renderer owns generated meshes");
        let baseline = capture_retained(&mut renderer, &cold, &retained);
        let warm =
            renderer
                .renderer
                .prepare_native_new_project(&state.ui.new_project, 960, 720, scale);
        assert_eq!(renderer.renderer.control_meshes.builds, builds);
        assert!(baseline == capture_retained(&mut renderer, &warm, &retained));
        // Live form content must not be retained in position-only meshes.
        state.ui.new_project.project_name.push_str("-edited");
        let edited =
            renderer
                .renderer
                .prepare_native_new_project(&state.ui.new_project, 960, 720, scale);
        assert_eq!(renderer.renderer.control_meshes.builds, builds);
        let mut fresh = PreparedScene::from_workspace_with_terminal_renderer(
            &state,
            960,
            720,
            scale,
            camera,
            &retained,
            &[],
            None,
            true,
        );
        let legacy_pixels = capture_retained(&mut renderer, &fresh, &retained);
        let surface = crate::RectPx {
            x: 0.0,
            y: 0.0,
            width: 960.0,
            height: 720.0,
        };
        for run in &mut fresh.menu_overlay_text_runs {
            run.clip_bounds = run.clip_bounds.unwrap_or(surface).intersect(surface);
        }
        assert_eq!(edited.menu_overlay_vertices, fresh.menu_overlay_vertices);
        assert_eq!(edited.menu_overlay_text_runs, fresh.menu_overlay_text_runs);
        let modal_start = fresh
            .hit_regions
            .iter()
            .position(|hit| hit.target == HitTarget::NewProjectModal)
            .expect("legacy form modal");
        assert_eq!(edited.hit_regions, fresh.hit_regions[modal_start..]);
        let actual = capture_retained(&mut renderer, &edited, &RetainedScene::empty());
        assert!(
            actual != baseline,
            "editing the field changes visible pixels"
        );
        assert!(actual == legacy_pixels);
        renderer.renderer = Renderer::new(
            &renderer.device,
            &renderer.queue,
            OUTPUT_FORMAT,
            DEFAULT_MSAA_SAMPLES,
        );
        assert_eq!(renderer.renderer.control_meshes.builds, 0);
        let reset =
            renderer
                .renderer
                .prepare_native_new_project(&state.ui.new_project, 960, 720, scale);
        assert!(renderer.renderer.control_meshes.builds > 0);
        assert!(actual == capture_retained(&mut renderer, &reset, &retained));
    }
}

#[test]
#[ignore = "requires local GPU; bounded production input clipping proof"]
fn new_project_long_name_paints_only_inside_its_field() {
    let mut renderer = hardware_renderer(760, 750);
    let mut dialog = datum_gui_protocol::NewProjectDialogState {
        open: true,
        ..Default::default()
    };
    let empty = renderer
        .renderer
        .prepare_native_new_project(&dialog, 760, 750, 1.0);
    let before = capture(&mut renderer, &empty);
    dialog.project_name = "W".repeat(90);
    let filled = renderer
        .renderer
        .prepare_native_new_project(&dialog, 760, 750, 1.0);
    let field = filled
        .hit_regions
        .iter()
        .find(|hit| hit.target == crate::HitTarget::NewProjectName)
        .expect("name field")
        .rect;
    let after = capture(&mut renderer, &filled);
    assert_ne!(before, after);
    for y in 0..750 {
        for x in 0..760 {
            if !field.contains(x as f32, y as f32) || y as f32 >= field.y + 28.0 {
                assert_eq!(
                    before.get_pixel(x, y),
                    after.get_pixel(x, y),
                    "field overflow at {x},{y}"
                );
            }
        }
    }
    if let Ok(path) = std::env::var("DATUM_INPUT_TEXT_CAPTURE") {
        after.save(path).unwrap();
    }
}
