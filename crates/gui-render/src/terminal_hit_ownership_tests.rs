use super::*;

fn rect_is_inside(inner: RectPx, outer: RectPx) -> bool {
    inner.x >= outer.x
        && inner.y >= outer.y
        && inner.x + inner.width <= outer.x + outer.width
        && inner.y + inner.height <= outer.y + outer.height
}

#[test]
fn rectangle_intersection_requires_positive_visible_area() {
    let viewport = RectPx {
        x: 10.0,
        y: 20.0,
        width: 100.0,
        height: 80.0,
    };
    assert_eq!(
        RectPx {
            x: 0.0,
            y: 0.0,
            width: 40.0,
            height: 40.0
        }
        .intersect(viewport),
        Some(RectPx {
            x: 10.0,
            y: 20.0,
            width: 30.0,
            height: 20.0
        })
    );
    assert_eq!(
        RectPx {
            x: 110.0,
            y: 30.0,
            width: 20.0,
            height: 20.0
        }
        .intersect(viewport),
        None,
        "edge contact is not a visible hit area"
    );
}

#[test]
fn status_projection_reads_the_shared_application_focus_authority() {
    let mut state = datum_gui_protocol::load_fixture_workspace_state();
    state.ui.focus = datum_gui_protocol::ApplicationFocus::Editor(datum_gui_protocol::PaneId(1));
    assert_eq!(
        crate::status_bar::application_focus_label(&state),
        "Schematic"
    );
    state.ui.focus = datum_gui_protocol::ApplicationFocus::Terminal;
    assert_eq!(
        crate::status_bar::application_focus_label(&state),
        "Terminal"
    );
    state.ui.focus = datum_gui_protocol::ApplicationFocus::Overlay;
    assert_eq!(
        crate::status_bar::application_focus_label(&state),
        "Overlay"
    );
}

#[test]
fn editor_scene_hits_cannot_shadow_terminal_screen_at_adversarial_cameras() {
    let mut state = datum_gui_protocol::load_fixture_workspace_state();
    state.ui.active_dock_tab = Some(datum_gui_protocol::DockTab::Terminal);
    state.ui.dock_height_px = 260;
    let retained = RetainedScene::from_workspace(&state, 1280, 800);
    let fit = CameraState::fit_to_bounds(&state.scene.bounds);
    let scene_height = (state.scene.bounds.max_y - state.scene.bounds.min_y).max(1) as f32;

    for (center_shift, zoom) in [
        (0.0, 1.0),
        (-scene_height * 0.75, 1.0),
        (scene_height * 0.75, 1.0),
        (-scene_height * 0.35, 4.0),
        (scene_height * 0.35, 4.0),
    ] {
        let camera = CameraState {
            center_y_nm: fit.center_y_nm + center_shift,
            zoom,
            ..fit
        };
        let prepared = PreparedScene::from_workspace(&state, 1280, 800, camera, &retained);
        let scene_viewport = prepared.scene_viewport;
        for region in &prepared.hit_regions {
            if matches!(
                region.target,
                HitTarget::AuthoredObject(_) | HitTarget::ReviewAction(_)
            ) {
                assert!(
                    rect_is_inside(region.rect, scene_viewport),
                    "editor hit {:?} escaped {:?} at shift={center_shift} zoom={zoom}",
                    region.rect,
                    scene_viewport,
                );
            }
        }

        let screen = prepared
            .hit_regions
            .iter()
            .find(|region| region.target == HitTarget::TerminalScreen)
            .expect("terminal screen hit target");
        for (x, y) in [
            (screen.rect.x + 0.5, screen.rect.y + 0.5),
            (
                screen.rect.x + screen.rect.width * 0.5,
                screen.rect.y + screen.rect.height * 0.5,
            ),
            (
                screen.rect.x + screen.rect.width - 0.5,
                screen.rect.y + screen.rect.height - 0.5,
            ),
        ] {
            assert_eq!(
                prepared.hit_test(x, y),
                Some(&HitTarget::TerminalScreen),
                "terminal screen was shadowed at ({x}, {y}), shift={center_shift}, zoom={zoom}",
            );
        }
    }
}
