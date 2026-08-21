//! Deliberate terminal link/file interaction over TerminalCore's inert targets.
//!
//! Terminal output never launches anything. The focused user may copy a
//! detected target directly or arm an exact HTTP(S) target for a second,
//! explicit confirmation before the Linux desktop handoff.

use anyhow::{Context, Result, bail};
use datum_gui_protocol::{ApplicationFocus, DockTab, TerminalLinkKind, TerminalLinkTarget};
use winit::{
    event::{ElementState, KeyEvent},
    keyboard::{Key, NamedKey},
};

use crate::Runtime;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LinkConfirmationKeyAction {
    Unowned,
    Consume,
    Cancel,
    Confirm,
    FinishEscapeRelease,
}

impl Runtime {
    pub(super) fn terminal_link_target_at_cursor(&self) -> Option<TerminalLinkTarget> {
        let pointer = self.last_cursor_pos?;
        self.terminal_link_target_at(pointer)
    }

    pub(super) fn terminal_link_cursor_icon(
        &self,
        pointer: (f32, f32),
    ) -> Option<winit::window::CursorIcon> {
        (self.modifiers.control_key()
            && self.application_focus() == ApplicationFocus::Terminal
            && self.terminal_link_target_at(pointer).is_some())
        .then_some(winit::window::CursorIcon::Pointer)
    }

    fn terminal_link_target_at(&self, pointer: (f32, f32)) -> Option<TerminalLinkTarget> {
        let geometry = self.terminal_screen_geometry();
        let (column, visible_row) = geometry.cell_at(pointer.0, pointer.1)?;
        let lane = &self.workspace().ui.terminal;
        self.terminal_sessions
            .active_link_target_at(
                usize::from(geometry.rows),
                lane.scroll_offset,
                usize::from(visible_row),
                usize::from(column),
                lane.current_working_directory.as_deref(),
            )
            .ok()
            .flatten()
    }

    pub(super) fn arm_terminal_link_at_cursor(&mut self) -> bool {
        if self.application_focus() != ApplicationFocus::Terminal
            || self.workspace().ui.active_dock_tab != Some(DockTab::Terminal)
        {
            return false;
        }
        let Some(target) = self.terminal_link_target_at_cursor() else {
            return false;
        };
        self.arm_terminal_link_target(target)
    }

    pub(super) fn arm_terminal_link_target(&mut self, target: TerminalLinkTarget) -> bool {
        if validate_http_target(&target).is_err()
            || self.application_focus() != ApplicationFocus::Terminal
            || self.workspace().ui.active_dock_tab != Some(DockTab::Terminal)
            || self.workspace().ui.terminal.status != "running"
            || self
                .workspace()
                .ui
                .terminal
                .application_shutdown_blocked
                .is_some()
        {
            return false;
        }
        let terminal = &mut self.session.workspace_mut().ui.terminal;
        terminal.link_confirmation = Some(target);
        terminal.link_escape_release_pending = false;
        self.invalidate_frame();
        true
    }

    pub(super) fn handle_terminal_link_confirmation_key(&mut self, event: &KeyEvent) -> bool {
        if self.workspace().ui.terminal.link_confirmation.is_some()
            && (self.workspace().ui.terminal.status != "running"
                || self
                    .workspace()
                    .ui
                    .terminal
                    .application_shutdown_blocked
                    .is_some())
        {
            self.cancel_terminal_link_confirmation();
            return false;
        }
        let terminal = &self.workspace().ui.terminal;
        match link_confirmation_key_action(
            terminal.link_confirmation.is_some(),
            terminal.link_escape_release_pending,
            event.state,
            &event.logical_key,
        ) {
            LinkConfirmationKeyAction::Unowned => false,
            LinkConfirmationKeyAction::FinishEscapeRelease => {
                self.session
                    .workspace_mut()
                    .ui
                    .terminal
                    .link_escape_release_pending = false;
                true
            }
            LinkConfirmationKeyAction::Cancel => {
                self.session
                    .workspace_mut()
                    .ui
                    .terminal
                    .link_escape_release_pending = true;
                self.cancel_terminal_link_confirmation();
                true
            }
            LinkConfirmationKeyAction::Confirm => {
                self.confirm_terminal_link_open();
                true
            }
            LinkConfirmationKeyAction::Consume => true,
        }
    }

    pub(super) fn cancel_terminal_link_confirmation(&mut self) {
        self.session.workspace_mut().ui.terminal.link_confirmation = None;
        self.invalidate_frame();
    }

    pub(super) fn confirm_terminal_link_open(&mut self) -> bool {
        if self.application_focus() != ApplicationFocus::Terminal
            || self.workspace().ui.active_dock_tab != Some(DockTab::Terminal)
            || self.workspace().ui.terminal.status != "running"
            || self
                .workspace()
                .ui
                .terminal
                .application_shutdown_blocked
                .is_some()
        {
            return false;
        }
        let Some(target) = self.workspace().ui.terminal.link_confirmation.clone() else {
            return false;
        };
        if let Err(error) = launch_http_target(&target) {
            self.log_review_event(format!("terminal link handoff failed: {error}"));
            return true;
        }
        self.session.workspace_mut().ui.terminal.link_confirmation = None;
        self.log_review_event("confirmed terminal HTTP link handed to desktop".to_string());
        self.invalidate_frame();
        true
    }

    pub(super) fn copy_terminal_link_target(&mut self, target: &TerminalLinkTarget) -> bool {
        if self.write_clipboard_text(&target.target).is_err() {
            self.log_review_event("terminal link copy failed".to_string());
            return true;
        }
        self.log_review_event("terminal link target copied".to_string());
        true
    }
}

fn link_confirmation_key_action(
    active: bool,
    escape_release_pending: bool,
    state: ElementState,
    key: &Key,
) -> LinkConfirmationKeyAction {
    if !active {
        return if escape_release_pending
            && state == ElementState::Released
            && matches!(key, Key::Named(NamedKey::Escape))
        {
            LinkConfirmationKeyAction::FinishEscapeRelease
        } else {
            LinkConfirmationKeyAction::Unowned
        };
    }
    if state == ElementState::Released {
        return LinkConfirmationKeyAction::Consume;
    }
    match key {
        Key::Named(NamedKey::Escape) => LinkConfirmationKeyAction::Cancel,
        Key::Named(NamedKey::Enter) => LinkConfirmationKeyAction::Confirm,
        _ => LinkConfirmationKeyAction::Consume,
    }
}

fn validate_http_target(target: &TerminalLinkTarget) -> Result<()> {
    if target.kind != TerminalLinkKind::HttpUri {
        bail!("only HTTP(S) terminal targets may be opened");
    }
    let http = target
        .target
        .get(..7)
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case("http://"));
    let https = target
        .target
        .get(..8)
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case("https://"));
    if !(http || https) {
        bail!("terminal target is not an HTTP(S) URI");
    }
    if target.target.chars().any(char::is_control) {
        bail!("terminal URI contains a control character");
    }
    Ok(())
}

fn launch_http_target(target: &TerminalLinkTarget) -> Result<()> {
    validate_http_target(target)?;
    let mut child = std::process::Command::new("/usr/bin/xdg-open")
        .arg(&target.target)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .context("start the Linux desktop URI handoff")?;
    std::thread::spawn(move || {
        let _ = child.wait();
    });
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_exact_http_and_https_targets_are_openable() {
        for target in ["https://example.test/a;b", "HTTP://example.test"] {
            assert!(
                validate_http_target(&TerminalLinkTarget {
                    kind: TerminalLinkKind::HttpUri,
                    target: target.to_string(),
                })
                .is_ok()
            );
        }
        for (kind, target) in [
            (TerminalLinkKind::Path, "/tmp/report.html"),
            (TerminalLinkKind::HttpUri, "file:///tmp/report.html"),
            (TerminalLinkKind::HttpUri, "javascript:alert(1)"),
            (TerminalLinkKind::HttpUri, "https://example.test/\nnext"),
        ] {
            assert!(
                validate_http_target(&TerminalLinkTarget {
                    kind,
                    target: target.to_string(),
                })
                .is_err()
            );
        }
    }

    #[test]
    fn confirmation_owns_all_keys_and_the_matching_escape_release() {
        assert_eq!(
            link_confirmation_key_action(
                true,
                false,
                ElementState::Pressed,
                &Key::Named(NamedKey::Enter),
            ),
            LinkConfirmationKeyAction::Confirm
        );
        assert_eq!(
            link_confirmation_key_action(
                true,
                false,
                ElementState::Pressed,
                &Key::Character("x".into()),
            ),
            LinkConfirmationKeyAction::Consume
        );
        assert_eq!(
            link_confirmation_key_action(
                true,
                false,
                ElementState::Pressed,
                &Key::Named(NamedKey::Escape),
            ),
            LinkConfirmationKeyAction::Cancel
        );
        assert_eq!(
            link_confirmation_key_action(
                false,
                true,
                ElementState::Released,
                &Key::Named(NamedKey::Escape),
            ),
            LinkConfirmationKeyAction::FinishEscapeRelease
        );
    }
}
