//! The single owner-approved painter/hover batch. A panic stops later groups.
use super::*;
use crate::gpu_surface::prefix_negative_control::{Fault, set};

fn native_doa() -> ReviewWorkspaceState {
    let project = std::path::PathBuf::from(std::env::var_os("DATUM_NATIVE_TEST_PROJECT").unwrap());
    let state = datum_gui_protocol::load_board_editor_workspace_state(
        &datum_gui_protocol::LiveReviewRequest {
            project_root: project,
            board_file: None,
            artifact_path: None,
            net_uuid: None,
            from_anchor_pad_uuid: None,
            to_anchor_pad_uuid: None,
            profile: None,
            kicad_board_source: None,
        },
    )
    .unwrap();
    assert_eq!(state.scene.pads.len(), 81);
    state
}

fn samples(c: &mut OffscreenRenderer) -> Vec<u32> {
    let images = c
        .renderer
        .surface_attachments
        .prefix_images(&c.device)
        .unwrap();
    let (_, working) = images.restoration_views().unwrap();
    crate::renderer_state::damage::restore::sample_tests::read_samples(
        &c.device, &c.queue, working, c.width, c.height,
    )
}

fn exact_samples(c: &mut OffscreenRenderer, label: &str, reuse: bool) {
    let pixels = exact(c, label, reuse);
    let actual = samples(c);
    // A cold prefix/copy/full-suffix graph supplies the per-sample reference;
    // exact() separately compares against the original full painter graph.
    let prefix = std::mem::take(&mut c.renderer.render_session.prefix);
    let cold = frame(c, true);
    assert!(!c.renderer.prefix_copy_work().0);
    let reference = samples(c);
    c.renderer.render_session.prefix = prefix;
    assert!(pixels.as_raw() == cold.as_raw(), "{label}: cold pixels");
    assert!(actual == reference, "{label}: all eight samples");
}

fn reject(c: &mut OffscreenRenderer, fault: Fault) {
    r3::pointer(c, Some((0.23, 0.31)));
    exact(c, "negative initial support", true);
    // Cross an actual glyph row, including the support guard outside the
    // opaque crosshair, so duplicate alpha painting cannot pass vacuously.
    let scene = c.renderer.render_session().prepared().unwrap();
    let pane = scene
        .surface_passes()
        .iter()
        .find(|p| p.surface == crate::SceneSurface::Board)
        .unwrap();
    let run = scene
        .text_runs
        .iter()
        .find(|r| {
            r.origin == crate::TextOrigin::Viewport(pane.pane_id)
                && r.layer == crate::TextLayer::Workspace
                && pane.scene_viewport.contains(r.x, r.y)
        })
        .expect("visible actual board glyphs");
    let cursor = ScreenPointPx {
        x: pane.scene_viewport.x + pane.scene_viewport.width * 0.67,
        y: run.y + run.size * 0.5 + 0.375,
    };
    let session = c.renderer.render_session_mut();
    assert!(session.update_pointer(PointerUpdate {
        generation: session.pointer_generation(),
        cursor: Some(cursor),
        hover: None,
        style: CrosshairStyle::FullViewport,
    }));
    let reference = full_reference(c);
    set(fault);
    let wrong = frame(c, false);
    assert!(
        wrong.as_raw() != reference.as_raw(),
        "negative must change pixels"
    );
    exact(c, "failed presentation reconstruction", false);
}

fn crossings() {
    let mut c = reference_capture8();
    c.renderer.surface_attachments.enable_sample_readback();
    c.width = 1280;
    c.height = 800;
    let mut state = native_doa();
    state.selection = datum_gui_protocol::SelectionTarget::None;
    let source = SourceEpoch::default();
    prepare(&mut c, &state, &source);
    exact(&mut c, "crossing baseline", false);
    r3::pointer(&mut c, Some((0.3, 0.4)));
    exact_samples(&mut c, "fractional crosshair enter", true);
    r3::pointer(&mut c, Some((0.7, 0.6)));
    exact_samples(&mut c, "old/new crossing alpha", true);
    reject(&mut c, Fault::OverlappingSuffix);
    reject(&mut c, Fault::MissingSuffixMask);
}

fn hover_dependencies() {
    let mut c = reference_capture8();
    c.renderer.surface_attachments.enable_sample_readback();
    c.width = 1280;
    c.height = 800;
    let mut state = native_doa();
    state.selection = datum_gui_protocol::SelectionTarget::None;
    let source = SourceEpoch::default();
    prepare(&mut c, &state, &source);
    exact(&mut c, "hover baseline", false);
    let uploads = world_uploads(&c);
    let archive = include_str!(
        "../../../../docs/reviews/gui-performance/gpu-redraw-proposal/timed-native-result/cold-demands.json"
    );
    let ids: Vec<_> = archive
        .lines()
        .filter_map(|line| {
            line.trim()
                .strip_prefix("\"after\": \"")
                .and_then(|v| v.strip_suffix("\","))
        })
        .collect();
    assert_eq!(ids.len(), 14);
    for (ordinal, id) in ids.into_iter().enumerate() {
        state.ui.hovered_object = (!id.is_empty()).then(|| HoverTarget {
            object_id: id.into(),
            surface: PaneContent::Board,
        });
        let session = c.renderer.render_session_mut();
        assert!(session.update_pointer(PointerUpdate {
            generation: session.pointer_generation(),
            cursor: None,
            hover: state.ui.hovered_object.as_ref(),
            style: CrosshairStyle::FullViewport,
        }));
        if ordinal == 0 {
            exact_samples(&mut c, "hover ring alpha", true);
        } else {
            exact(&mut c, "archived non-pad hover", true);
        }
        assert_eq!(world_uploads(&c), uploads);
    }
    for pad in [Some(0), Some(1), None] {
        state.ui.hovered_object = pad.map(|i| HoverTarget {
            object_id: state.scene.pads[i].object_id.clone(),
            surface: PaneContent::Board,
        });
        let session = c.renderer.render_session_mut();
        assert!(!session.update_pointer(PointerUpdate {
            generation: session.pointer_generation(),
            cursor: None,
            hover: state.ui.hovered_object.as_ref(),
            style: CrosshairStyle::Local,
        }));
        prepare(&mut c, &state, &source);
        exact_samples(&mut c, "pad material/text change stays cold", false);
        exact(&mut c, "stable material dependency", true);
    }
}

#[test]
#[ignore = "one approved four-group GPU batch, first failure stops; no native timing trial"]
fn painter_hover_p630_four_group_batch() {
    eprintln!("GROUP 1: crossing alpha, exact samples and overlapping/missing restrictions");
    crossings();
    eprintln!("GROUP 2: Console, text, menu and terminal painter clips");
    p630_exact8_terminal_layers_and_equal_length_replacement();
    p630_exact8_console_covers_canvas_text_without_hiding_foreground();
    eprintln!("GROUP 3: archived non-pad and actual pad material/text dependencies");
    hover_dependencies();
    eprintln!("GROUP 4: stale/failed support, admission refusal and submitted retirement");
    r3::prove();
    let mut c = reference_capture8();
    let state = crate::gpu_surface_pass::board_fixture_state();
    prepare(&mut c, &state, &SourceEpoch::default());
    exact(&mut c, "overflow baseline", false);
    r3::pointer(&mut c, Some((0.4, 0.5)));
    set(Fault::DamageOverflow);
    exact(&mut c, "partition overflow full fallback", false);
    eprintln!("FOUR GROUPS PASSED; pointer performance and final qualification remain separate");
}
