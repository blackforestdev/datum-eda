//! One keyboard-focus authority for the gui-app window (decision 024 Phase 0,
//! TF-01, bead dat-terminal-focus-authority-6aw).
//!
//! Keyboard routing consults WHO OWNS KEYS — never whether a dock is visible.
//! Opening the terminal dock must not steal the keyboard, and workspace hotkeys
//! must not type into a hidden shell buffer. `Editor` delegates pane identity to
//! the existing single-source pane focus (`workspace().ui.layout.focused`)
//! rather than duplicating it — one source of truth for pane focus (decision
//! 021). `Overlay` = transient key owners (marking menu today; more in TF-02+).
//!
//! The routing DECISION is the pure `key_route` function so it is testable
//! without winit or a `Runtime`; `handle_keyboard_input` is the window-event
//! dispatcher that applies those decisions to the running app.

use winit::event::{ElementState, KeyEvent};
use winit::keyboard::{Key, KeyCode, NamedKey, PhysicalKey};

use datum_gui_protocol::{ApplicationFocus, PaneId, SessionCommand};
use datum_gui_render::HitTarget;

use crate::app_shell::App;
use crate::{Runtime, terminal_input::terminal_new_session_shortcut};

/// What kind of keyboard traffic a key event represents, for routing purposes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum KeyClass {
    /// Raw PTY byte stream to the attached terminal session. The one class
    /// that also requires the terminal tab to exist on screen (raw input has
    /// nowhere visible to land otherwise).
    RawPty,
    /// Workspace hotkeys: tool keys, fit, zoom, crosshair, pane cycling,
    /// review navigation, and the Space pan chord.
    WorkspaceHotkey,
    /// Escape release after the PTY received the press; returns focus to the editor.
    TerminalFocusExit,
}

/// Exclusive terminal input recipient (TI-02).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TerminalInputOwner {
    AttachedPty,
    Unowned,
}

pub(crate) fn terminal_input_owner(
    focus: ApplicationFocus,
    terminal_tab_visible: bool,
) -> TerminalInputOwner {
    if focus != ApplicationFocus::Terminal || !terminal_tab_visible {
        return TerminalInputOwner::Unowned;
    }
    TerminalInputOwner::AttachedPty
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
/// `terminal_tab_visible` matters only for raw PTY routing; the remaining
/// classes preserve the TF-01 focus contract while `terminal_input_owner`
/// selects attached PTY or unowned behavior.
pub(crate) fn key_route(
    focus: ApplicationFocus,
    class: KeyClass,
    terminal_tab_visible: bool,
) -> RouteDecision {
    match (focus, class) {
        (ApplicationFocus::Terminal, KeyClass::RawPty) if terminal_tab_visible => {
            RouteDecision::Terminal
        }
        (ApplicationFocus::Terminal, KeyClass::RawPty) => RouteDecision::Unrouted,
        (ApplicationFocus::Terminal, KeyClass::TerminalFocusExit) => RouteDecision::ReleaseToEditor,
        (ApplicationFocus::Terminal, KeyClass::WorkspaceHotkey) => RouteDecision::Unrouted,
        (ApplicationFocus::Editor(_), KeyClass::WorkspaceHotkey) => RouteDecision::Editor,
        (ApplicationFocus::Editor(_), _) => RouteDecision::Unrouted,
        // Overlay owners (marking menu) consume keys through their own guard
        // arms (e.g. the marking-menu Escape arm); no class routes here.
        (ApplicationFocus::Overlay, _) => RouteDecision::Unrouted,
    }
}

/// Whether clicking this hit target is deliberate keyboard entry into the
/// terminal (T0-C02; decision 027 FT-008; spec §5 interaction contract):
/// the SHELL CONTENT cell rectangle itself, the dock TERMINAL TAB (owner
/// decision 2026-08-14, bead dat-pan-trace-terminal-pollution-0j0: clicking
/// the terminal tab is a deliberate "go to the terminal" gesture whose
/// resulting behavior expects terminal typing — ghostty/kitty style), plus
/// the session actions whose resulting behavior expects terminal typing —
/// switching to a session or opening a new one. Chrome that ends/suspends a
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
    )
}

/// Apply the production click-focus law without duplicating it in handlers or
/// tests. A canvas click always returns keys to the editor. A handled terminal
/// entry target arms terminal focus; every other hit preserves the current
/// owner. Programmatic command handoffs are observation targets, so they never
/// steal keyboard focus.
pub(crate) fn focus_after_canvas_click(pane: PaneId) -> ApplicationFocus {
    ApplicationFocus::Editor(pane)
}

pub(crate) fn focus_after_hit_target(
    current: ApplicationFocus,
    handled: bool,
    target: &HitTarget,
) -> ApplicationFocus {
    if handled && hit_target_is_terminal_entry(target) {
        ApplicationFocus::Terminal
    } else {
        current
    }
}

pub(crate) fn focus_before_terminal_mouse_press(
    current: ApplicationFocus,
    terminal_visible: bool,
    child_mouse_reporting: bool,
    pointer_over_screen: bool,
) -> ApplicationFocus {
    if terminal_visible && child_mouse_reporting && pointer_over_screen {
        ApplicationFocus::Terminal
    } else {
        current
    }
}

pub(crate) fn terminal_mouse_report_allowed(
    focus: ApplicationFocus,
    child_mouse_reporting: bool,
    session_attached: bool,
    pointer_over_screen: bool,
) -> bool {
    focus == ApplicationFocus::Terminal
        && child_mouse_reporting
        && session_attached
        && pointer_over_screen
}

/// Resolve the one focus-exit event that must run before raw PTY routing.
/// The press still reaches the child as ESC; the release transfers ownership
/// back to the editor after terminal-local rename/input state is dismissed.
pub(crate) fn pre_raw_escape_route(
    focus: ApplicationFocus,
    terminal_visible: bool,
    escape_released: bool,
) -> Option<RouteDecision> {
    if escape_released {
        let route = key_route(focus, KeyClass::TerminalFocusExit, terminal_visible);
        (route == RouteDecision::ReleaseToEditor).then_some(route)
    } else {
        None
    }
}

/// Return the child focus-report transition for a keyboard-owner change.
/// OS-window activation is intentionally absent: only crossing the Terminal
/// ownership boundary emits CSI I/O when the child enabled mode 1004.
pub(crate) fn terminal_focus_report_transition(
    previous: ApplicationFocus,
    next: ApplicationFocus,
) -> Option<bool> {
    let was_terminal = previous == ApplicationFocus::Terminal;
    let is_terminal = next == ApplicationFocus::Terminal;
    (was_terminal != is_terminal).then_some(is_terminal)
}

/// Workspace commands fire once on the initial key press. Terminal ownership
/// never satisfies this predicate, so the same physical hotkey is PTY input
/// rather than a leaked editor action (TF-05).
pub(crate) fn workspace_action_should_fire(
    focus: ApplicationFocus,
    terminal_tab_visible: bool,
    state: ElementState,
    repeat: bool,
) -> bool {
    state == ElementState::Pressed
        && !repeat
        && key_route(focus, KeyClass::WorkspaceHotkey, terminal_tab_visible)
            == RouteDecision::Editor
}

pub(crate) fn editor_new_terminal_shortcut(
    focus: ApplicationFocus,
    state: ElementState,
    repeat: bool,
    physical_key: PhysicalKey,
    modifiers: winit::keyboard::ModifiersState,
) -> bool {
    focus != ApplicationFocus::Terminal
        && focus != ApplicationFocus::Overlay
        && terminal_new_session_shortcut(state, repeat, physical_key, modifiers)
}

impl Runtime {
    pub(crate) fn application_focus(&self) -> ApplicationFocus {
        self.workspace().ui.focus
    }

    pub(crate) fn set_application_focus(&mut self, focus: ApplicationFocus) {
        let report = terminal_focus_report_transition(self.application_focus(), focus);
        self.session.workspace_mut().ui.focus = focus;
        if focus != ApplicationFocus::Terminal {
            self.session.workspace_mut().ui.terminal.ime_preedit = None;
        }
        if let Some(focused) = report {
            self.report_terminal_focus_event(focused);
            self.refresh_terminal_accessibility();
        }
    }
}

/// Route one window keyboard event through the focus authority. Returns true
/// when the event was consumed (a redraw may have been requested).
pub(crate) fn handle_keyboard_input(app: &mut App, event: &KeyEvent) -> bool {
    let Some(focus) = app
        .runtime
        .as_ref()
        .map(|runtime| runtime.application_focus())
    else {
        return false;
    };
    let confirms_armed_close = app.runtime.as_ref().is_some_and(|runtime| {
        armed_close_shortcut(
            runtime.terminal_sessions.active_close_confirmation_armed(),
            runtime.modifiers.control_key(),
            runtime.modifiers.shift_key(),
            event.state,
            event.repeat,
            event.physical_key,
        )
    });
    if confirms_armed_close {
        if let Some(runtime) = &mut app.runtime {
            let _ = runtime
                .terminal_sessions
                .confirm_close_active(&mut runtime.session.workspace_mut().ui.terminal);
            runtime.sync_terminal_tabs();
            runtime.invalidate_frame();
        }
        app.request_redraw_if_needed();
        return true;
    }
    let opens_terminal_session = app.runtime.as_ref().is_some_and(|runtime| {
        editor_new_terminal_shortcut(
            focus,
            event.state,
            event.repeat,
            event.physical_key,
            runtime.modifiers,
        )
    });
    if opens_terminal_session {
        if let Some(runtime) = &mut app.runtime {
            runtime.spawn_terminal_session_tab();
            runtime.set_application_focus(ApplicationFocus::Terminal);
            runtime.invalidate_frame();
        }
        app.request_redraw_if_needed();
        return true;
    }
    let dock_visible = app
        .runtime
        .as_ref()
        .is_some_and(|runtime| runtime.workspace().ui.active_dock_tab.is_some());
    let editor_owns_hotkeys =
        key_route(focus, KeyClass::WorkspaceHotkey, dock_visible) == RouteDecision::Editor;
    let workspace_action_pressed =
        workspace_action_should_fire(focus, dock_visible, event.state, event.repeat);
    let terminal_input_owner = app
        .runtime
        .as_ref()
        .map_or(TerminalInputOwner::Unowned, Runtime::terminal_input_owner);
    let terminal_owns_attached_pty = terminal_input_owner == TerminalInputOwner::AttachedPty
        && key_route(focus, KeyClass::RawPty, dock_visible) == RouteDecision::Terminal;
    let escape_released = matches!(event.logical_key, Key::Named(NamedKey::Escape))
        && event.state == ElementState::Released;
    let terminal_search_owns_escape = app.runtime.as_ref().is_some_and(|runtime| {
        let search = &runtime.workspace().ui.terminal.search;
        search.active || search.escape_release_pending
    });

    // TF-02: focus-exit ordering is part of the authority. Raw PTY routing
    // owns the Escape press, but must not consume its release before this arm.
    if !terminal_search_owns_escape
        && pre_raw_escape_route(focus, dock_visible, escape_released).is_some()
    {
        if let Some(runtime) = &mut app.runtime {
            let pane = runtime.workspace().ui.layout.focused;
            runtime.set_application_focus(ApplicationFocus::Editor(pane));
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
    if terminal_owns_attached_pty {
        if let Some(runtime) = &mut app.runtime
            && runtime.handle_terminal_key_input(event)
        {
            app.request_redraw_if_needed();
        }
        return true;
    }
    if escape_released && app.runtime.as_mut().is_some_and(Runtime::cancel_active_pan) {
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
    // Pane focus cycling (decision 021): Tab -> next leaf, Shift+Tab ->
    // previous leaf, when the dock does not own the keyboard. Reuses the
    // FEEL warm-camera focus swap; workspace view state, never journaled.
    if matches!(event.logical_key, Key::Named(NamedKey::Tab)) && workspace_action_pressed {
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
        && workspace_action_pressed
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

fn armed_close_shortcut(
    armed: bool,
    control: bool,
    shift: bool,
    state: ElementState,
    repeat: bool,
    physical_key: PhysicalKey,
) -> bool {
    armed
        && control
        && shift
        && state == ElementState::Pressed
        && !repeat
        && matches!(physical_key, PhysicalKey::Code(KeyCode::KeyW))
}

#[cfg(test)]
#[path = "terminal_focus_convergence_tests.rs"]
mod convergence_tests;
#[cfg(test)]
#[path = "keyboard_focus_tests.rs"]
mod tests;
