//! One keyboard-focus authority for the gui-app window (decision 024 Phase 0,
//! TF-01, bead dat-terminal-focus-authority-6aw).
//!
//! Keyboard routing consults WHO OWNS KEYS — never whether a dock is visible.
//! Opening the terminal dock must not steal the keyboard, and workspace hotkeys
//! must not type into a hidden line editor. `Editor` delegates pane identity to
//! the existing single-source pane focus (`workspace().ui.layout.focused`)
//! rather than duplicating it — one source of truth for pane focus (decision
//! 021). `Overlay` = transient key owners (marking menu today; more in TF-02+).
//!
//! The routing DECISION is the pure `key_route` function so it is testable
//! without winit or a `Runtime`; `handle_keyboard_input` is the window-event
//! dispatcher that applies those decisions to the running app.

use winit::event::{ElementState, KeyEvent};
use winit::keyboard::{Key, NamedKey};

use datum_gui_protocol::SessionCommand;
use datum_gui_render::HitTarget;

use crate::app_shell::App;
use crate::{Runtime, terminal_raw_input_should_handle};

/// The single keyboard-focus owner. Exactly one of these owns key events at any
/// time; dock visibility plays no role in the decision (the raw-PTY route is
/// the one class that additionally requires the terminal tab on screen).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum KeyboardFocus {
    #[default]
    Editor,
    Terminal,
    Overlay,
}

/// What kind of keyboard traffic a key event represents, for routing purposes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum KeyClass {
    /// Raw PTY byte stream to the attached terminal session. The one class
    /// that also requires the terminal tab to exist on screen (raw input has
    /// nowhere visible to land otherwise).
    RawPty,
    /// Dock line-editor editing/navigation keys: text, space, backspace,
    /// enter, arrows, home/end, tab-complete, and clipboard shortcuts.
    DockLineEdit,
    /// Workspace hotkeys: tool keys, fit, zoom, crosshair, pane cycling,
    /// review navigation, and the Space pan chord.
    WorkspaceHotkey,
    /// Escape released while the dock line editor is already empty.
    EscapeWithEmptyInput,
}

/// The routing outcome for a key class under a focus owner.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RouteDecision {
    /// The terminal owns the key.
    Terminal,
    /// The editor (focused pane) owns the key.
    Editor,
    /// Consume the key and hand keyboard ownership back to the editor.
    ReleaseToEditor,
    /// No owner for this class under this focus; the key falls through.
    Unrouted,
}

/// The pure routing decision: who owns a key of `class` under `focus`.
/// `terminal_tab_visible` matters ONLY for the raw-PTY class — dock visibility
/// must never change any other routing outcome (the TF-01 invariant).
pub(crate) fn key_route(
    focus: KeyboardFocus,
    class: KeyClass,
    terminal_tab_visible: bool,
) -> RouteDecision {
    match (focus, class) {
        (KeyboardFocus::Terminal, KeyClass::RawPty) if terminal_tab_visible => {
            RouteDecision::Terminal
        }
        (KeyboardFocus::Terminal, KeyClass::RawPty) => RouteDecision::Unrouted,
        (KeyboardFocus::Terminal, KeyClass::DockLineEdit) => RouteDecision::Terminal,
        (KeyboardFocus::Terminal, KeyClass::EscapeWithEmptyInput) => RouteDecision::ReleaseToEditor,
        (KeyboardFocus::Terminal, KeyClass::WorkspaceHotkey) => RouteDecision::Unrouted,
        (KeyboardFocus::Editor, KeyClass::WorkspaceHotkey) => RouteDecision::Editor,
        (KeyboardFocus::Editor, _) => RouteDecision::Unrouted,
        // Overlay owners (marking menu) consume keys through their own guard
        // arms (e.g. the marking-menu Escape arm); no class routes here.
        (KeyboardFocus::Overlay, _) => RouteDecision::Unrouted,
    }
}

/// Whether clicking this hit target is deliberate keyboard entry into the
/// terminal (T0-C02; decision 027 FT-008; spec §5 interaction contract):
/// the SHELL CONTENT cell rectangle itself, the dock TERMINAL TAB (owner
/// decision 2026-08-14, bead dat-pan-trace-terminal-pollution-0j0: clicking
/// the terminal tab is a deliberate "go to the terminal" gesture whose
/// resulting behavior expects terminal typing — ghostty/kitty style), plus
/// the session actions whose resulting behavior expects terminal typing —
/// switching to a session, opening a new one, and renaming (the rename editor
/// routes through terminal-owned line editing). Chrome that ends/suspends a
/// session (restart/detach/close) still never arms focus — the over-broad
/// TF-01 entry rule was the silent focus-steal defect diagnosed on
/// dat-terminal-focus-authority-6aw. Opening the dock programmatically never
/// steals keys.
pub(crate) fn hit_target_is_terminal_entry(target: &HitTarget) -> bool {
    matches!(
        target,
        HitTarget::TerminalScreen
            | HitTarget::TerminalTab
            | HitTarget::TerminalSessionTab(_)
            | HitTarget::TerminalSessionNew
            | HitTarget::TerminalSessionRenameActive
    )
}

/// Apply the production click-focus law without duplicating it in handlers or
/// tests. A canvas click always returns keys to the editor. A handled terminal
/// entry target arms terminal focus; every other hit preserves the current
/// owner. Programmatic command handoffs are observation targets, so they never
/// steal keyboard focus.
pub(crate) fn focus_after_canvas_click() -> KeyboardFocus {
    KeyboardFocus::Editor
}

pub(crate) fn focus_after_hit_target(
    current: KeyboardFocus,
    handled: bool,
    target: &HitTarget,
) -> KeyboardFocus {
    if handled && hit_target_is_terminal_entry(target) {
        KeyboardFocus::Terminal
    } else {
        current
    }
}

/// Resolve the one focus-exit event that must run before raw PTY routing.
/// The press still reaches the child as ESC; the release transfers ownership
/// back to the editor after terminal-local rename/input state is dismissed.
pub(crate) fn pre_raw_escape_route(
    focus: KeyboardFocus,
    terminal_visible: bool,
    escape_released: bool,
) -> Option<RouteDecision> {
    if escape_released {
        let route = key_route(focus, KeyClass::EscapeWithEmptyInput, terminal_visible);
        (route == RouteDecision::ReleaseToEditor).then_some(route)
    } else {
        None
    }
}

/// Return the child focus-report transition for a keyboard-owner change.
/// OS-window activation is intentionally absent: only crossing the Terminal
/// ownership boundary emits CSI I/O when the child enabled mode 1004.
pub(crate) fn terminal_focus_report_transition(
    previous: KeyboardFocus,
    next: KeyboardFocus,
) -> Option<bool> {
    let was_terminal = previous == KeyboardFocus::Terminal;
    let is_terminal = next == KeyboardFocus::Terminal;
    (was_terminal != is_terminal).then_some(is_terminal)
}

fn terminal_owns_keyboard(focus: KeyboardFocus) -> bool {
    focus == KeyboardFocus::Terminal
}

impl Runtime {
    pub(crate) fn keyboard_focus(&self) -> KeyboardFocus {
        self.keyboard_focus
    }

    pub(crate) fn set_keyboard_focus(&mut self, focus: KeyboardFocus) {
        let report = terminal_focus_report_transition(self.keyboard_focus, focus);
        self.keyboard_focus = focus;
        // Renderer projection only: KeyboardFocus remains the sole authority.
        // Focus changes fill/outline the child-selected cursor shape; they do
        // not replace DECSCUSR shape or DEC cursor visibility state (TF-04).
        self.session
            .workspace_mut()
            .ui
            .terminal
            .has_keyboard_focus = terminal_owns_keyboard(focus);
        if let Some(focused) = report {
            self.report_terminal_focus_event(focused);
        }
    }
}

/// Route one window keyboard event through the focus authority. Returns true
/// when the event was consumed (a redraw may have been requested).
pub(crate) fn handle_keyboard_input(app: &mut App, event: &KeyEvent) -> bool {
    let Some(focus) = app
        .runtime
        .as_ref()
        .map(|runtime| runtime.keyboard_focus())
    else {
        return false;
    };
    let dock_visible = app
        .runtime
        .as_ref()
        .is_some_and(|runtime| runtime.workspace().ui.active_dock_tab.is_some());
    let editor_owns_hotkeys =
        key_route(focus, KeyClass::WorkspaceHotkey, dock_visible) == RouteDecision::Editor;
    let terminal_owns_line_edit =
        key_route(focus, KeyClass::DockLineEdit, dock_visible) == RouteDecision::Terminal;
    let escape_released = matches!(event.logical_key, Key::Named(NamedKey::Escape))
        && event.state == ElementState::Released;

    // TF-02: focus-exit ordering is part of the authority. Raw PTY routing
    // owns the Escape press, but must not consume its release before this arm.
    if pre_raw_escape_route(focus, dock_visible, escape_released).is_some() {
        if let Some(runtime) = &mut app.runtime {
            if runtime.cancel_terminal_rename() {
                app.request_redraw_if_needed();
                return true;
            }
            let input_was_empty = runtime.current_dock_input().is_none_or(|s| s.is_empty());
            if input_was_empty {
                runtime.set_keyboard_focus(KeyboardFocus::Editor);
            } else {
                let terminal = &mut runtime.session.workspace_mut().ui.terminal;
                terminal.input.clear();
                terminal.cursor = 0;
            }
            runtime.invalidate_frame();
            app.request_redraw_if_needed();
        }
        return true;
    }

    // Space pan chord — an editor gesture, so it runs only when the editor owns
    // keys; it must never swallow Space typed into the terminal (TF-01).
    if editor_owns_hotkeys
        && app
            .runtime
            .as_mut()
            .is_some_and(|runtime| runtime.handle_pan_key(event))
    {
        return true;
    }
    if app.runtime.as_ref().is_some_and(|runtime| {
        terminal_raw_input_should_handle(
            runtime.terminal_accepts_raw_input(),
            runtime.is_paste_shortcut(event),
            runtime.is_copy_shortcut(event),
        )
    }) {
        if let Some(runtime) = &mut app.runtime
            && runtime.handle_terminal_key_input(event)
        {
            app.request_redraw_if_needed();
        }
        return true;
    }
    if terminal_owns_line_edit
        && app
            .runtime
            .as_ref()
            .is_some_and(|runtime| runtime.is_paste_shortcut(event))
    {
        if let Some(runtime) = &mut app.runtime
            && runtime.paste_dock_input()
        {
            app.request_redraw_if_needed();
        }
        return true;
    }
    if terminal_owns_line_edit
        && app
            .runtime
            .as_ref()
            .is_some_and(|runtime| runtime.is_copy_shortcut(event))
    {
        if let Some(runtime) = &mut app.runtime
            && runtime.copy_dock_input()
        {
            app.request_redraw_if_needed();
        }
        return true;
    }
    if terminal_owns_line_edit
        && app
            .runtime
            .as_ref()
            .is_some_and(|runtime| runtime.is_cut_shortcut(event))
    {
        if let Some(runtime) = &mut app.runtime
            && runtime.cut_dock_input()
        {
            app.request_redraw_if_needed();
        }
        return true;
    }
    if let Key::Character(text) = &event.logical_key
        && event.state == ElementState::Pressed
        && app.runtime.as_ref().is_some_and(|runtime| {
            runtime.dock_accepts_text_input() && !runtime.modifiers.control_key()
        })
    {
        if let Some(runtime) = &mut app.runtime
            && runtime.append_dock_text(text)
        {
            app.request_redraw_if_needed();
        }
        return true;
    }
    if matches!(event.logical_key, Key::Named(NamedKey::Space))
        && event.state == ElementState::Pressed
        && app
            .runtime
            .as_ref()
            .is_some_and(|runtime| runtime.dock_accepts_text_input())
    {
        if let Some(runtime) = &mut app.runtime
            && runtime.append_dock_text(" ")
        {
            app.request_redraw_if_needed();
        }
        return true;
    }
    if escape_released && app.runtime.as_mut().is_some_and(Runtime::cancel_active_pan) {
        return true;
    }
    if matches!(event.logical_key, Key::Named(NamedKey::Backspace))
        && event.state == ElementState::Released
        && terminal_owns_line_edit
    {
        if let Some(runtime) = &mut app.runtime
            && runtime.backspace_dock_input()
        {
            app.request_redraw_if_needed();
        }
        return true;
    }
    if matches!(event.logical_key, Key::Named(NamedKey::Enter))
        && event.state == ElementState::Released
        && terminal_owns_line_edit
    {
        if let Some(runtime) = &mut app.runtime
            && runtime.submit_dock_input()
        {
            app.request_redraw_if_needed();
        }
        return true;
    }
    if escape_released
        && app
            .runtime
            .as_ref()
            .is_some_and(|runtime| runtime.marking_menu_active())
    {
        if let Some(runtime) = &mut app.runtime
            && runtime.dismiss_marking_menu()
        {
            app.request_redraw_if_needed();
        }
        return true;
    }
    if matches!(event.logical_key, Key::Named(NamedKey::ArrowLeft))
        && event.state == ElementState::Released
        && terminal_owns_line_edit
    {
        if let Some(runtime) = &mut app.runtime
            && runtime.move_dock_cursor(-1)
        {
            app.request_redraw_if_needed();
        }
        return true;
    }
    if matches!(event.logical_key, Key::Named(NamedKey::ArrowRight))
        && event.state == ElementState::Released
        && terminal_owns_line_edit
    {
        if let Some(runtime) = &mut app.runtime
            && runtime.move_dock_cursor(1)
        {
            app.request_redraw_if_needed();
        }
        return true;
    }
    if matches!(event.logical_key, Key::Named(NamedKey::Home))
        && event.state == ElementState::Released
        && terminal_owns_line_edit
    {
        if let Some(runtime) = &mut app.runtime
            && runtime.move_dock_cursor_to_edge(true)
        {
            app.request_redraw_if_needed();
        }
        return true;
    }
    if matches!(event.logical_key, Key::Named(NamedKey::End))
        && event.state == ElementState::Released
        && terminal_owns_line_edit
    {
        if let Some(runtime) = &mut app.runtime
            && runtime.move_dock_cursor_to_edge(false)
        {
            app.request_redraw_if_needed();
        }
        return true;
    }
    if matches!(event.logical_key, Key::Named(NamedKey::Tab))
        && event.state == ElementState::Released
        && terminal_owns_line_edit
    {
        if let Some(runtime) = &mut app.runtime
            && runtime.complete_dock_input()
        {
            app.request_redraw_if_needed();
        }
        return true;
    }
    // Pane focus cycling (decision 021): Tab -> next leaf, Shift+Tab ->
    // previous leaf, when the dock does not own the keyboard. Reuses the
    // FEEL warm-camera focus swap; workspace view state, never journaled.
    if matches!(event.logical_key, Key::Named(NamedKey::Tab))
        && event.state == ElementState::Released
        && editor_owns_hotkeys
    {
        if let Some(runtime) = &mut app.runtime {
            if runtime.modifiers.shift_key() {
                runtime.pane_focus_prev();
            } else {
                runtime.pane_focus_next();
            }
            app.request_redraw_if_needed();
        }
        return true;
    }
    if let Key::Character(text) = &event.logical_key
        && event.state == ElementState::Released
        && let Some(action) = crate::workspace_keyboard::character_action(
            text,
            app.runtime
                .as_ref()
                .is_some_and(|runtime| runtime.modifiers.control_key()),
        )
    {
        if let Some(runtime) = &mut app.runtime
            && editor_owns_hotkeys
            && crate::workspace_keyboard::apply(runtime, action)
        {
            app.request_redraw_if_needed();
        }
        return true;
    }
    if escape_released {
        if let Some(runtime) = &mut app.runtime {
            if runtime.dispatch_session_command(SessionCommand::CancelAuthoringGesture) {
                app.request_redraw_if_needed();
                return true;
            }
            if !matches!(
                runtime.workspace().selection,
                datum_gui_protocol::SelectionTarget::None
            ) && runtime.dispatch_session_command(SessionCommand::ClearSelection)
            {
                app.request_redraw_if_needed();
            }
        }
        return true;
    }
    false
}

#[cfg(test)]
#[path = "keyboard_focus_tests.rs"]
mod tests;
