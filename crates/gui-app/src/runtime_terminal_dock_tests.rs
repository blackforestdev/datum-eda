use super::*;

#[test]
fn only_the_per_tab_close_target_owns_close_hover() {
    let close = HitTarget::TerminalSessionClose("terminal-2".to_string());
    assert_eq!(
        hovered_terminal_close_session(Some(&close)).as_deref(),
        Some("terminal-2")
    );
    assert_eq!(
        hovered_terminal_close_session(Some(&HitTarget::TerminalSessionNew)),
        None
    );
    assert_eq!(hovered_terminal_close_session(None), None);
}

#[test]
fn tab_body_starts_reorder_but_close_control_remains_exclusive() {
    let tab = HitTarget::TerminalSessionTab("terminal-2".to_string());
    let close = HitTarget::TerminalSessionClose("terminal-2".to_string());
    assert_eq!(
        terminal_tab_drag_start(Some(&tab)).as_deref(),
        Some("terminal-2")
    );
    assert_eq!(terminal_tab_drag_start(Some(&close)), None);
    assert_eq!(terminal_tab_session(Some(&close)), Some("terminal-2"));
}

#[test]
fn dock_boundary_and_active_drag_own_north_south_resize_cursor() {
    assert!(owns_dock_resize_cursor(
        Some(&HitTarget::DockResizeHandle),
        false
    ));
    assert!(owns_dock_resize_cursor(
        Some(&HitTarget::TerminalScreen),
        true
    ));
    assert!(!owns_dock_resize_cursor(
        Some(&HitTarget::TerminalScreen),
        false
    ));
}

#[test]
fn dock_drag_previews_frames_and_commits_one_pty_resize_on_release() {
    let source = include_str!("runtime_terminal_geometry.rs");
    let drag = source
        .split("pub(super) fn handle_dock_resize_drag")
        .nth(1)
        .unwrap()
        .split("pub(super) fn finish_dock_resize_drag")
        .next()
        .unwrap();
    assert!(drag.contains("self.invalidate_frame()"));
    assert!(!drag.contains("self.invalidate_scene()"));
    assert!(!drag.contains("self.resize_terminal_to_dock()"));

    let finish = source
        .split("pub(super) fn finish_dock_resize_drag")
        .nth(1)
        .unwrap()
        .split("pub(super) fn terminal_screen_geometry")
        .next()
        .unwrap();
    assert_eq!(finish.matches("self.resize_terminal_to_dock()").count(), 1);
}

#[test]
fn terminal_split_drag_previews_layout_and_commits_one_pty_resize_on_release() {
    let source = include_str!("runtime_terminal_dock.rs");
    let drag = source
        .split("pub(super) fn advance_terminal_split_drag")
        .nth(1)
        .unwrap()
        .split("pub(super) fn finish_terminal_split_drag")
        .next()
        .unwrap();
    assert!(drag.contains("set_active_split_ratio"));
    assert!(drag.contains("self.sync_terminal_tabs()"));
    assert!(drag.contains("self.invalidate_frame()"));
    assert!(!drag.contains("self.resize_terminal_to_dock()"));

    let finish = source
        .split("pub(super) fn finish_terminal_split_drag")
        .nth(1)
        .unwrap()
        .split("pub(super) fn dock_resize_cursor_icon")
        .next()
        .unwrap();
    assert_eq!(finish.matches("self.resize_terminal_to_dock()").count(), 1);
}

#[test]
fn terminal_maximize_is_transient_and_preserves_the_normal_dock_height() {
    let mut state = datum_gui_protocol::load_fixture_workspace_state();
    state.ui.active_dock_tab = Some(DockTab::Terminal);
    state.ui.dock_height_px = 287;
    assert!(toggle_terminal_maximized(&mut state.ui));
    assert!(state.ui.terminal_maximized);
    assert_eq!(state.ui.dock_height_px, 287);
    assert_eq!(state.ui.effective_dock_height_px(), u32::MAX);
    assert!(toggle_terminal_maximized(&mut state.ui));
    assert!(!state.ui.terminal_maximized);
    assert_eq!(state.ui.effective_dock_height_px(), 287);
}

#[test]
fn hidden_or_nonterminal_dock_cannot_enter_terminal_maximize() {
    let mut state = datum_gui_protocol::load_fixture_workspace_state();
    state.ui.active_dock_tab = None;
    assert!(!toggle_terminal_maximized(&mut state.ui));
    assert!(!state.ui.terminal_maximized);
}
