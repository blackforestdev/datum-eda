use super::{
    KeyClass, RouteDecision, focus_after_hit_target, focus_before_terminal_mouse_press, key_route,
    workspace_action_should_fire,
};
use crate::terminal_input::terminal_tab_sequence;
use datum_gui_protocol::{ApplicationFocus, DockTab, PaneId};
use datum_gui_render::{CameraState, HitTarget, PreparedScene, RetainedScene};
use winit::event::ElementState;
use winit::keyboard::ModifiersState;

fn terminal_screen_target_under_adversarial_board_overlay() -> HitTarget {
    let mut state = datum_gui_protocol::load_fixture_workspace_state();
    state.ui.active_dock_tab = Some(DockTab::Terminal);
    state.ui.dock_height_px = 260;
    state.ui.focus = ApplicationFocus::Editor(PaneId(0));
    let retained = RetainedScene::from_workspace(&state, 1280, 800);
    let mut camera = CameraState::fit_to_bounds(&state.scene.bounds);
    let scene_height = (state.scene.bounds.max_y - state.scene.bounds.min_y).max(1) as f32;
    camera.center_y_nm -= scene_height * 0.75;
    camera.zoom = 4.0;
    let prepared = PreparedScene::from_workspace(&state, 1280, 800, camera, &retained);
    let screen = prepared
        .hit_regions
        .iter()
        .find(|region| region.target == HitTarget::TerminalScreen)
        .expect("terminal screen region");
    prepared
        .hit_test(
            screen.rect.x + screen.rect.width * 0.5,
            screen.rect.y + screen.rect.height * 0.5,
        )
        .cloned()
        .expect("terminal screen center must remain selectable")
}

#[test]
fn non_mouse_child_click_selects_terminal_and_tab_never_cycles_editor_panes() {
    let target = terminal_screen_target_under_adversarial_board_overlay();
    assert_eq!(target, HitTarget::TerminalScreen);
    let focus = focus_after_hit_target(ApplicationFocus::Editor(PaneId(0)), true, &target);
    assert_eq!(focus, ApplicationFocus::Terminal);
    assert_eq!(
        key_route(focus, KeyClass::RawPty, true),
        RouteDecision::Terminal
    );
    assert!(!workspace_action_should_fire(
        focus,
        true,
        ElementState::Pressed,
        false,
    ));
    assert_eq!(
        terminal_tab_sequence(ModifiersState::empty()),
        Some(b"\t".to_vec())
    );
    assert_eq!(
        terminal_tab_sequence(ModifiersState::SHIFT),
        Some(b"\x1b[Z".to_vec())
    );
}

#[test]
fn mouse_reporting_press_selects_same_terminal_authority_before_forwarding() {
    let focus =
        focus_before_terminal_mouse_press(ApplicationFocus::Editor(PaneId(1)), true, true, true);
    assert_eq!(focus, ApplicationFocus::Terminal);
    assert_eq!(
        key_route(focus, KeyClass::RawPty, true),
        RouteDecision::Terminal
    );
    assert!(!workspace_action_should_fire(
        focus,
        true,
        ElementState::Pressed,
        false,
    ));
}

#[test]
fn outside_terminal_screen_preserves_editor_pane_authority() {
    let editor = ApplicationFocus::Editor(PaneId(1));
    assert_eq!(
        focus_before_terminal_mouse_press(editor, true, true, false),
        editor
    );
    assert_eq!(
        focus_after_hit_target(editor, false, &HitTarget::TerminalScreen),
        editor
    );
}
