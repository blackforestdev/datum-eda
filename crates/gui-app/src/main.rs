mod native_device_recovery;
mod runtime_state;
use anyhow::{Context, Result};
use arboard::{Clipboard, GetExtLinux, LinuxClipboardKind, SetExtLinux};
use clap::Parser;
use datum_gui_protocol::{
    BoardTextAlignmentField, BoardTextBooleanField, BoardTextCycleField, BoardTextHeightStep,
    BoardTextLineSpacingStep, BoardTextRotationStep, ConsoleFeedbackSource, DockTab, HoverTarget,
    LiveDesignSession, LiveReviewRequest, MarkingMenuState, PaneContent, PointNm, RectNm,
    SceneBounds, SessionCommand, SessionEvent, TerminalCommandHandoff, WorkspaceTool,
    ensure_known_good_demo_request, load_board_editor_workspace_state,
    load_kicad_schematic_workspace_state, load_live_workspace_state,
    materialize_kicad_board_request,
};
#[cfg(feature = "visual")]
use datum_gui_render::visual_capture::OffscreenRenderer;
use datum_gui_render::{
    CameraState, HitTarget, PreparedScene, Renderer, SceneSurface, ShellLayout,
};
use gui_runtime_support::native_surface_transaction::SurfaceTransaction;
use runtime_state::Runtime;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
#[cfg(feature = "visual")]
use std::sync::mpsc;
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::{ElementState, KeyEvent, MouseButton, MouseScrollDelta, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{Key, KeyCode, ModifiersState, NamedKey, PhysicalKey},
    window::{Window, WindowAttributes, WindowId},
};
mod app_bootstrap;
mod app_frame;
mod app_native_events;
mod app_shell;
mod application_terminal_shutdown;
mod artifact_preview_controls;
mod board_text_terminal_commands;
mod console_accessibility;
mod console_feedback;
mod global_preferences_projection;
mod global_preferences_runtime;
mod global_preferences_window;
mod gui_runtime_support;
mod interaction_refresh;
mod keyboard_focus;
mod native_frame_adapters;
mod native_frame_coordinator;
mod native_gpu;
mod native_gpu_measurements;
mod native_resize_repro;
mod new_project_window;
mod owned_window_policy;
mod pan_gesture;
mod pane_cameras;
mod pane_grid_lod;
mod pane_resize;
mod production_status_refresh;
mod project_preferences_runtime;
mod project_preferences_window;
mod resize_smoke;
mod retained_scene_lifecycle;
mod runtime_capture;
use runtime_capture::run_offscreen_visual_test;
mod runtime_board_text_edit;
mod runtime_camera_fit_targets;
mod runtime_camera_pane;
mod runtime_geometry_helpers;
mod runtime_init;
mod runtime_menu_actions;
mod runtime_present;
mod runtime_primary_button;
mod runtime_primary_pointer;
mod runtime_revision_workspace;
mod runtime_surface;
mod runtime_terminal_clipboard;
mod runtime_terminal_context;
mod runtime_terminal_dock;
mod runtime_terminal_font;
mod runtime_terminal_geometry;
mod runtime_terminal_input;
mod runtime_terminal_links;
mod runtime_terminal_notifications;
mod runtime_terminal_pointer;
mod runtime_terminal_profile;
mod runtime_terminal_render;
mod runtime_terminal_search;
mod runtime_terminal_theme;
mod runtime_view_actions;
mod terminal_accessibility;
mod terminal_accessibility_bridge;
mod terminal_accessibility_platform;
mod terminal_active_context;
mod terminal_activity_snapshot;
mod terminal_agent_authority;
mod terminal_agent_credential;
mod terminal_capability;
mod terminal_check_context;
mod terminal_context;
mod terminal_context_contract;
mod terminal_context_io;
mod terminal_core_adapter;
mod terminal_input;
mod terminal_process;
mod terminal_profile;
mod terminal_proposal_context;
mod terminal_session;
mod terminal_session_context;
mod terminal_session_controls;
mod terminal_session_events;
mod terminal_split_drag;
mod terminal_tab_drag;
mod terminal_transport;
mod terminal_working_directory;
mod workspace_keyboard;
use app_bootstrap::{GuiArgs, LaunchState};
use app_shell::{App, fatal_gui_error, terminal_scrollback_page_step};
use board_text_terminal_commands::{
    BoardTextEditTerminalField, BoardTextQuickEditTerminalAction, board_text_edit_terminal_command,
    board_text_quick_edit_terminal_command,
};
use datum_gui_protocol::ApplicationFocus;
pub(crate) use gui_runtime_support::*;
use pan_gesture::PanGestureState;
use pane_cameras::PaneCameras;
use pane_resize::DividerDrag;
use runtime_geometry_helpers::*;
#[cfg(feature = "visual")]
use std::fs;
use terminal_input::{TerminalKeyAction, terminal_key_action};
use terminal_session::{
    TerminalLaunchContext, TerminalSessionRegistry, terminal_launch_context_from_state,
};
use terminal_session_events::{
    prepare_terminal_command_execution, record_manual_terminal_command_handoff,
};
use terminal_split_drag::TerminalSplitDividerDrag;
#[cfg(feature = "visual")]
const COPY_BYTES_PER_PIXEL: u32 = 4;
#[cfg(feature = "visual")]
const WGPU_COPY_BYTES_PER_ROW_ALIGNMENT: u32 = 256;
const ASSISTANT_ACTIVITY_COMMAND: &str =
    "datum-eda context session-activity --session \"$DATUM_SESSION_ID\" --limit 20";
fn main() -> Result<()> {
    install_gui_panic_hook();
    reset_gui_diagnostic_log();
    if let Some(result) = native_resize_repro::run_if_requested() {
        return result;
    }
    let args = GuiArgs::parse();
    append_gui_diagnostic_line(format!("startup args={args:?}"));
    if args.visual_test && args.exit_after_screenshot && !args.window_visual_test {
        return run_offscreen_visual_test(&args);
    }
    let event_loop = EventLoop::new().context("failed to create event loop")?;
    App::new(args, event_loop.create_proxy()).run(event_loop)
}
impl Runtime {
    // T0-C01 (DATUM_NATIVE_TERMINAL_SPEC.md) / decision 027 FT-001: there is
    // deliberately NO `push_terminal_line` here. Terminal cells are mutated
    // only by PTY bytes interpreted by the terminal core. Terminal lifecycle
    // and diagnostic messages stay in terminal chrome, never the Datum Console
    // or the terminal grid.

    fn handle_terminal_key_input(&mut self, event: &KeyEvent) -> bool {
        if self.handle_terminal_clipboard_confirmation_key(event) {
            return true;
        }
        if self.handle_terminal_link_confirmation_key(event) {
            return true;
        }
        if self.handle_terminal_search_key(event) {
            return true;
        }
        let application_cursor_keys = self.workspace().ui.terminal.application_cursor_keys;
        let application_keypad = self.workspace().ui.terminal.application_keypad;
        let action = terminal_key_action(
            event,
            self.modifiers,
            application_cursor_keys,
            application_keypad,
        );
        if let Some(handled) = self.handle_close_confirmation_action(&action) {
            return handled;
        }
        match action {
            TerminalKeyAction::CoreKey(input) => {
                match self.terminal_sessions.encode_active_key(&input) {
                    Ok(Some(bytes)) => self.write_foreign_shell_bytes(&bytes),
                    Ok(None) => true,
                    Err(err) => {
                        self.log_terminal_event(format!("terminal key encoding failed: {err}"));
                        true
                    }
                }
            }
            TerminalKeyAction::NewSession => self.spawn_terminal_session_tab(),
            TerminalKeyAction::SplitRight => {
                self.spawn_terminal_split(datum_gui_protocol::TerminalSplitDirection::SideBySide)
            }
            TerminalKeyAction::SplitDown => {
                self.spawn_terminal_split(datum_gui_protocol::TerminalSplitDirection::Stacked)
            }
            TerminalKeyAction::TerminateSession => {
                self.terminate_terminal_session();
                true
            }
            TerminalKeyAction::CloseSession => self.close_active_terminal_session(),
            TerminalKeyAction::RestartSession => {
                self.restart_terminal_session();
                true
            }
            TerminalKeyAction::ScrollbackPageUp => {
                self.scroll_terminal_scrollback(terminal_scrollback_page_step(self.workspace()));
                true
            }
            TerminalKeyAction::ScrollbackPageDown => {
                self.scroll_terminal_scrollback_down(terminal_scrollback_page_step(
                    self.workspace(),
                ));
                true
            }
            TerminalKeyAction::ScrollbackTop => {
                self.scroll_terminal_scrollback_to_top();
                true
            }
            TerminalKeyAction::ScrollbackBottom => {
                self.scroll_terminal_scrollback_to_bottom();
                true
            }
            TerminalKeyAction::CopyClipboard => self.copy_terminal_scrollback(),
            TerminalKeyAction::PasteClipboard => self.paste_terminal_input(),
            TerminalKeyAction::Search => {
                self.begin_terminal_search();
                true
            }
            TerminalKeyAction::FontZoomIn => self.adjust_terminal_font_zoom(1),
            TerminalKeyAction::FontZoomOut => self.adjust_terminal_font_zoom(-1),
            TerminalKeyAction::FontZoomReset => self.reset_terminal_font_zoom(),
            TerminalKeyAction::Ignore => false,
        }
    }

    fn scroll_terminal_scrollback(&mut self, delta: usize) {
        let max = self.terminal_sessions.active_render_row_count();
        let terminal = &mut self.session.workspace_mut().ui.terminal;
        terminal.scroll_offset = (terminal.scroll_offset + delta).min(max);
        self.invalidate_frame();
    }

    fn scroll_terminal_scrollback_down(&mut self, delta: usize) {
        let terminal = &mut self.session.workspace_mut().ui.terminal;
        terminal.scroll_offset = terminal.scroll_offset.saturating_sub(delta);
        self.invalidate_frame();
    }

    fn scroll_terminal_scrollback_to_top(&mut self) {
        self.session.workspace_mut().ui.terminal.scroll_offset =
            self.terminal_sessions.active_render_row_count();
        self.invalidate_frame();
    }

    fn scroll_terminal_scrollback_to_bottom(&mut self) {
        self.session.workspace_mut().ui.terminal.scroll_offset = 0;
        self.invalidate_frame();
    }

    fn report_terminal_focus_event(&mut self, focused: bool) {
        if !self.terminal_sessions.active_attached() {
            return;
        }
        let input = if focused {
            datum_terminal_core::FocusInput::Gained
        } else {
            datum_terminal_core::FocusInput::Lost
        };
        match self.terminal_sessions.encode_active_focus(input) {
            Ok(Some(bytes)) => {
                if let Err(err) = self.terminal_sessions.active().write_bytes(&bytes) {
                    self.log_terminal_event(format!("terminal focus report failed: {err}"));
                }
            }
            Ok(None) => {}
            Err(err) => self.log_terminal_event(format!("terminal focus encoding failed: {err}")),
        }
    }

    fn terminate_terminal_session(&mut self) {
        match self
            .terminal_sessions
            .terminate_active(&mut self.session.workspace_mut().ui.terminal)
        {
            Ok(()) => {}
            Err(err) => self.log_terminal_event(format!("terminal terminate failed: {err}")),
        }
        self.sync_terminal_tabs();
        self.invalidate_frame();
    }

    fn restart_terminal_session(&mut self) {
        match self.terminal_sessions.restart_active(
            &mut self.session.workspace_mut().ui.terminal,
            &self.terminal_launch_context,
        ) {
            Ok(()) => {
                self.log_terminal_event(
                    "terminal restart requested; waiting for verified teardown",
                );
                self.resize_terminal_to_dock();
            }
            Err(err) => self.log_terminal_event(format!("terminal restart failed: {err}")),
        }
        self.terminal_production_refresh_pending = false;
        self.terminal_workspace_refresh_pending = false;
        self.terminal_production_refresh_due = None;
        self.terminal_production_refresh_attempts = 0;
        self.sync_terminal_tabs();
        self.invalidate_frame();
    }

    fn activate_terminal_session(&mut self, session_id: &str) -> bool {
        if let Err(err) = self
            .terminal_sessions
            .activate_with_lane(session_id, &mut self.session.workspace_mut().ui.terminal)
        {
            self.log_terminal_event(format!("terminal session activate failed: {err}"));
            return true;
        }
        self.set_active_dock(DockTab::Terminal);
        self.refresh_terminal_activity_summary();
        self.sync_terminal_tabs();
        self.resize_terminal_to_dock();
        self.invalidate_frame();
        true
    }

    fn copy_terminal_scrollback(&mut self) -> bool {
        if !matches!(self.workspace().ui.active_dock_tab, Some(DockTab::Terminal)) {
            return false;
        }
        let text = match self.terminal_sessions.copy_active_selection() {
            Ok(text) => text,
            Err(_) => return false,
        };
        if self.write_clipboard_text(&text).is_err() {
            self.log_terminal_event("clipboard copy failed".to_string());
            return true;
        }
        self.log_terminal_event("terminal text copied".to_string());
        true
    }

    fn paste_terminal_input(&mut self) -> bool {
        let Ok(text) = self.read_clipboard_text() else {
            self.log_terminal_event("clipboard paste failed".to_string());
            return false;
        };
        if text.is_empty() {
            return false;
        }
        match self.terminal_input_owner() {
            keyboard_focus::TerminalInputOwner::AttachedPty => {
                match self.terminal_sessions.encode_active_paste(&text) {
                    Ok(Some(bytes)) => self.write_foreign_shell_bytes(&bytes),
                    Ok(None) => false,
                    Err(err) => {
                        self.log_terminal_event(format!("terminal paste encoding failed: {err}"));
                        true
                    }
                }
            }
            keyboard_focus::TerminalInputOwner::Unowned => false,
        }
    }

    fn read_clipboard_text(&mut self) -> Result<String> {
        if let Some(clipboard) = &mut self.clipboard
            && let Ok(text) = clipboard
                .get()
                .clipboard(LinuxClipboardKind::Clipboard)
                .text()
            && !text.is_empty()
        {
            return Ok(text);
        }
        self.read_clipboard_text_fallback()
    }

    fn write_clipboard_text(&mut self, text: &str) -> Result<()> {
        self.write_terminal_clipboard_text(
            datum_gui_protocol::TerminalClipboardSelection::Clipboard,
            text,
        )
    }

    fn write_terminal_clipboard_text(
        &mut self,
        selection: datum_gui_protocol::TerminalClipboardSelection,
        text: &str,
    ) -> Result<()> {
        let kind = match selection {
            datum_gui_protocol::TerminalClipboardSelection::Clipboard => {
                LinuxClipboardKind::Clipboard
            }
            datum_gui_protocol::TerminalClipboardSelection::Primary => LinuxClipboardKind::Primary,
        };
        if let Some(clipboard) = &mut self.clipboard
            && clipboard
                .set()
                .clipboard(kind)
                .text(text.to_string())
                .is_ok()
        {
            return Ok(());
        }
        self.write_clipboard_text_fallback(selection, text)
    }

    fn read_clipboard_text_fallback(&self) -> Result<String> {
        let output = Command::new("/usr/bin/xclip")
            .args(["-selection", "clipboard", "-o"])
            .output()
            .context("read clipboard with xclip")?;
        if !output.status.success() {
            anyhow::bail!("xclip clipboard read failed");
        }
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    fn write_clipboard_text_fallback(
        &self,
        selection: datum_gui_protocol::TerminalClipboardSelection,
        text: &str,
    ) -> Result<()> {
        let selection = match selection {
            datum_gui_protocol::TerminalClipboardSelection::Clipboard => "clipboard",
            datum_gui_protocol::TerminalClipboardSelection::Primary => "primary",
        };
        let mut child = Command::new("/usr/bin/xclip")
            .args(["-selection", selection])
            .stdin(Stdio::piped())
            .spawn()
            .context("spawn xclip for clipboard write")?;
        let mut stdin = child.stdin.take().context("take xclip stdin")?;
        stdin
            .write_all(text.as_bytes())
            .context("write clipboard text to xclip")?;
        drop(stdin);
        let status = child.wait().context("wait for xclip clipboard write")?;
        if !status.success() {
            anyhow::bail!("xclip clipboard write failed");
        }
        Ok(())
    }

    fn log_terminal_event(&mut self, message: impl Into<String>) {
        self.session.workspace_mut().ui.terminal.status = message.into();
        self.invalidate_frame();
    }

    fn apply_session_result(&mut self, result: datum_gui_protocol::SessionCommandResult) -> bool {
        if !result.handled {
            return false;
        }
        for event in result.events {
            match event {
                // Shared source dependency validation owns retained reuse/history.
                SessionEvent::SceneChanged | SessionEvent::SelectionChanged(_) => {
                    self.invalidate_frame()
                }
                SessionEvent::FrameChanged => self.invalidate_frame(),
                SessionEvent::ToolChanged(_) => self.invalidate_frame(),
            }
        }
        true
    }

    fn dispatch_session_command(&mut self, command: SessionCommand) -> bool {
        let result = self.session.apply(command);
        self.apply_session_result(result)
    }

    fn set_workspace_tool(&mut self, tool: WorkspaceTool) -> bool {
        if !matches!(tool, WorkspaceTool::Select) {
            self.log_console_refusal(
                ConsoleFeedbackSource::Tool,
                format!("{} is disabled in the Phase 1 read-only GUI", tool.label()),
            );
            self.invalidate_frame();
            return true;
        }
        let handled = self.dispatch_session_command(SessionCommand::SetTool(tool));
        if handled {
            self.log_console_echo(ConsoleFeedbackSource::Tool, "Selection tool active");
        }
        handled
    }

    fn active_tool_is_authoring(&self) -> bool {
        false
    }

    fn handle_authoring_pointer_move(&mut self, screen_pos: (f32, f32)) -> bool {
        if !self.active_tool_is_authoring() || !self.workspace().authoring.gesture.is_active() {
            return false;
        }
        let Some(prepared) = self.prepared_scene() else {
            return false;
        };
        // Authoring is a board-scene gesture; only the board surface drives it.
        let Some((world, SceneSurface::Board)) =
            prepared.world_point_at_screen(screen_pos.0, screen_pos.1)
        else {
            return false;
        };
        let target_object_id = self.authoring_target_object_id(world);
        self.dispatch_session_command(SessionCommand::PreviewAuthoringGesture {
            world,
            target_object_id,
        })
    }

    fn handle_authoring_canvas_click(
        &mut self,
        world: PointNm,
        target_object_id: Option<String>,
    ) -> bool {
        if !self.active_tool_is_authoring() {
            return false;
        }
        match self.workspace().tool {
            WorkspaceTool::DrawBoardTrack
                if self.workspace().authoring.gesture.anchor.is_some() =>
            {
                let Some(handoff) = self
                    .session
                    .workspace_mut()
                    .finish_draw_board_track_handoff(world)
                else {
                    self.invalidate_frame();
                    return true;
                };
                self.queue_authoring_terminal_handoff(handoff, "draw-board-track");
                self.invalidate_scene();
                return true;
            }
            WorkspaceTool::PlaceBoardVia => {
                let Some(handoff) = self
                    .session
                    .workspace_mut()
                    .finish_place_board_via_handoff(world)
                else {
                    self.log_console_refusal(
                        ConsoleFeedbackSource::Tool,
                        "place via requires a board net context".to_string(),
                    );
                    self.invalidate_frame();
                    return true;
                };
                self.queue_authoring_terminal_handoff(handoff, "place-board-via");
                self.invalidate_scene();
                return true;
            }
            WorkspaceTool::PlaceBoardText => {
                let Some(handoff) = self
                    .session
                    .workspace_mut()
                    .finish_place_board_text_handoff(world)
                else {
                    self.log_console_refusal(
                        ConsoleFeedbackSource::Tool,
                        "place text requires project backing".to_string(),
                    );
                    self.invalidate_frame();
                    return true;
                };
                self.queue_authoring_terminal_handoff(handoff, "place-board-text");
                self.invalidate_scene();
                return true;
            }
            WorkspaceTool::Move if self.workspace().authoring.gesture.anchor.is_some() => {
                let Some(handoff) = self
                    .session
                    .workspace_mut()
                    .finish_move_component_handoff(world)
                else {
                    self.log_console_refusal(
                        ConsoleFeedbackSource::Tool,
                        "move requires a selected component target".to_string(),
                    );
                    self.invalidate_frame();
                    return true;
                };
                self.queue_authoring_terminal_handoff(handoff, "move-board-component");
                self.invalidate_scene();
                return true;
            }
            WorkspaceTool::Move => {
                let Some(target) = target_object_id.clone() else {
                    self.log_console_refusal(
                        ConsoleFeedbackSource::Tool,
                        "move requires clicking a component first".to_string(),
                    );
                    return true;
                };
                if !target.starts_with("component:") {
                    self.log_console_refusal(
                        ConsoleFeedbackSource::Tool,
                        "move currently supports components only".to_string(),
                    );
                    return true;
                }
            }
            WorkspaceTool::Delete => {
                let Some(target) = target_object_id else {
                    self.log_console_refusal(
                        ConsoleFeedbackSource::Tool,
                        "delete requires an authored object target".to_string(),
                    );
                    return true;
                };
                let Some(handoff) = self.workspace().delete_authored_object_handoff(&target) else {
                    self.log_console_refusal_for_target(
                        ConsoleFeedbackSource::Tool,
                        target,
                        "Delete is unavailable for this object type",
                    );
                    return true;
                };
                self.queue_authoring_terminal_handoff(handoff, "delete-authored-object");
                self.invalidate_scene();
                return true;
            }
            WorkspaceTool::Select | WorkspaceTool::DrawBoardTrack => {}
        }
        self.dispatch_session_command(SessionCommand::BeginAuthoringGesture {
            world,
            target_object_id,
        })
    }

    fn authoring_target_object_id(&mut self, world: PointNm) -> Option<String> {
        let target = {
            if !self.ensure_retained_scene() {
                return None;
            }
            let retained = self
                .renderer
                .render_session()
                .board()
                .expect("retained scene initialized");
            retained
                .hit_test_authored_world(world, self.session.workspace())
                .cloned()
        };
        match target {
            Some(HitTarget::AuthoredObject(id)) => Some(id),
            Some(HitTarget::ReviewAction(id)) => Some(id),
            _ => None,
        }
    }

    fn select_hit_target(&mut self, target: &HitTarget) -> bool {
        let started = std::time::Instant::now();
        let handled = self.select_hit_target_inner(target);
        let next_focus =
            keyboard_focus::focus_after_hit_target(self.application_focus(), handled, target);
        if next_focus != self.application_focus() {
            self.set_application_focus(next_focus);
        }
        self.trace_timing(|| {
            format!(
                "select target {target:?} handled={handled} {}ms",
                started.elapsed().as_millis()
            )
        });
        handled
    }

    fn select_hit_target_inner(&mut self, target: &HitTarget) -> bool {
        if matches!(
            target,
            HitTarget::NewProjectModal
                | HitTarget::NewProjectName
                | HitTarget::NewProjectDestination
                | HitTarget::NewProjectUnitsChoice(_)
                | HitTarget::NewProjectUnitsSummary
                | HitTarget::NewProjectRetryGlobal
                | HitTarget::NewProjectCancel
                | HitTarget::NewProjectCreate
        ) {
            return self.activate_new_project_hit(target);
        }
        if let Some(handled) = self.apply_revision_hit(target) {
            return handled;
        }
        if let Some(handled) = self.activate_application_overlay_hit_target(target) {
            return handled;
        }
        match target {
            HitTarget::NewProjectModal
            | HitTarget::NewProjectName
            | HitTarget::NewProjectDestination
            | HitTarget::NewProjectUnitsChoice(_)
            | HitTarget::NewProjectUnitsSummary
            | HitTarget::NewProjectRetryGlobal
            | HitTarget::NewProjectCancel
            | HitTarget::NewProjectCreate => unreachable!("New Project targets return above"),
            HitTarget::CloseRevisionSurface
            | HitTarget::ToggleRevisionIssuanceArm
            | HitTarget::OpenRevisionWitness(_) => unreachable!("revision targets return above"),
            HitTarget::ReviewAction(action_id) => {
                let handled = self.dispatch_session_command(SessionCommand::SelectReviewAction(
                    action_id.clone(),
                ));
                if handled {
                    self.log_console_echo_for_action(
                        ConsoleFeedbackSource::Selection,
                        action_id,
                        "Review action selected",
                    );
                }
                handled
            }
            HitTarget::AuthoredObject(object_id) => {
                let handled = self.dispatch_session_command(SessionCommand::SelectAuthoredObject(
                    object_id.clone(),
                ));
                if handled {
                    self.session.workspace_mut().ui.hovered_object = None;
                    self.log_console_echo_for_target(
                        ConsoleFeedbackSource::Selection,
                        object_id,
                        "Object selected",
                    );
                }
                handled
            }
            HitTarget::CheckFinding(fingerprint) => {
                let handled = self.dispatch_session_command(SessionCommand::SelectCheckFinding(
                    fingerprint.clone(),
                ));
                if handled {
                    let target = self
                        .session
                        .workspace()
                        .checks
                        .findings
                        .iter()
                        .find(|finding| finding.fingerprint == *fingerprint)
                        .and_then(|finding| {
                            datum_gui_protocol::check_finding_scene_target_object_id(
                                &self.session.workspace().scene,
                                finding,
                            )
                        });
                    self.session.workspace_mut().ui.hovered_object =
                        target.clone().map(|object_id| HoverTarget {
                            object_id,
                            surface: PaneContent::Board,
                        });
                    if let Some(target) = target {
                        self.fit_scene_object(&target);
                    }
                }
                handled
            }
            HitTarget::FitBoard => {
                self.fit_camera();
                self.log_console_echo(ConsoleFeedbackSource::Viewport, "fit board");
                true
            }
            HitTarget::FitReviewTarget => {
                let handled = self.fit_review_target();
                if handled {
                    self.log_console_echo(
                        ConsoleFeedbackSource::Viewport,
                        "fit active review target",
                    );
                }
                handled
            }
            HitTarget::SetWorkspaceTool(tool) => self.set_workspace_tool(*tool),
            HitTarget::ReviewPrev => {
                let handled =
                    self.dispatch_session_command(SessionCommand::SelectPreviousReviewAction);
                if handled {
                    self.log_console_echo(
                        ConsoleFeedbackSource::Selection,
                        "selected previous review action",
                    );
                }
                handled
            }
            HitTarget::ReviewNext => {
                let handled = self.dispatch_session_command(SessionCommand::SelectNextReviewAction);
                if handled {
                    self.log_console_echo(
                        ConsoleFeedbackSource::Selection,
                        "selected next review action",
                    );
                }
                handled
            }
            HitTarget::ToggleShowAuthored => {
                let handled = self.dispatch_session_command(SessionCommand::ToggleShowAuthored);
                if handled {
                    let state = if self.workspace().ui.filters.show_authored {
                        "on"
                    } else {
                        "off"
                    };
                    self.log_console_echo(
                        ConsoleFeedbackSource::Viewport,
                        format!("authored visibility {state}"),
                    );
                }
                handled
            }
            HitTarget::ToggleShowProposed => {
                let handled = self.dispatch_session_command(SessionCommand::ToggleShowProposed);
                if handled {
                    let state = if self.workspace().ui.filters.show_proposed {
                        "on"
                    } else {
                        "off"
                    };
                    self.log_console_echo(
                        ConsoleFeedbackSource::Viewport,
                        format!("proposal visibility {state}"),
                    );
                }
                handled
            }
            HitTarget::ToggleShowUnrouted => {
                let handled = self.dispatch_session_command(SessionCommand::ToggleShowUnrouted);
                if handled {
                    let state = if self.workspace().ui.filters.show_unrouted {
                        "on"
                    } else {
                        "off"
                    };
                    self.log_console_echo(
                        ConsoleFeedbackSource::Viewport,
                        format!("unrouted visibility {state}"),
                    );
                }
                handled
            }
            HitTarget::ToggleDimUnrelated => {
                let handled = self.dispatch_session_command(SessionCommand::ToggleDimUnrelated);
                if handled {
                    let state = if self.workspace().ui.filters.dim_unrelated {
                        "on"
                    } else {
                        "off"
                    };
                    self.log_console_echo(
                        ConsoleFeedbackSource::Viewport,
                        format!("dim unrelated {state}"),
                    );
                }
                handled
            }
            HitTarget::ToggleLayer(layer_id) => {
                let handled = self.dispatch_session_command(SessionCommand::ToggleLayerVisibility(
                    layer_id.clone(),
                ));
                if handled {
                    self.session.workspace_mut().ui.filters.active_layer_id =
                        Some(layer_id.clone());
                    let visible = self
                        .workspace()
                        .ui
                        .filters
                        .layer_visibility
                        .get(layer_id)
                        .copied()
                        .unwrap_or(true);
                    let state = if visible { "visible" } else { "hidden" };
                    self.log_console_echo_for_target(
                        ConsoleFeedbackSource::Viewport,
                        layer_id,
                        format!("Layer visibility {state}"),
                    );
                    self.invalidate_scene();
                }
                handled
            }
            HitTarget::ToggleSelectedBoardTextMirrored => {
                self.toggle_selected_board_text_boolean(BoardTextBooleanField::Mirrored)
            }
            HitTarget::ToggleSelectedBoardTextKeepUpright => {
                self.toggle_selected_board_text_boolean(BoardTextBooleanField::KeepUpright)
            }
            HitTarget::ToggleSelectedBoardTextBold => {
                self.toggle_selected_board_text_boolean(BoardTextBooleanField::Bold)
            }
            HitTarget::CycleSelectedBoardTextRenderIntent => {
                self.cycle_selected_board_text_field(BoardTextCycleField::RenderIntent)
            }
            HitTarget::CycleSelectedBoardTextFamily => {
                self.cycle_selected_board_text_field(BoardTextCycleField::Family)
            }
            HitTarget::CycleSelectedBoardTextHAlign => {
                self.cycle_selected_board_text_alignment(BoardTextAlignmentField::Horizontal)
            }
            HitTarget::CycleSelectedBoardTextVAlign => {
                self.cycle_selected_board_text_alignment(BoardTextAlignmentField::Vertical)
            }
            HitTarget::DecreaseSelectedBoardTextHeight => {
                self.step_selected_board_text_height(BoardTextHeightStep::Decrease)
            }
            HitTarget::IncreaseSelectedBoardTextHeight => {
                self.step_selected_board_text_height(BoardTextHeightStep::Increase)
            }
            HitTarget::RotateSelectedBoardTextCounterClockwise90 => {
                self.step_selected_board_text_rotation(BoardTextRotationStep::CounterClockwise90)
            }
            HitTarget::RotateSelectedBoardTextClockwise90 => {
                self.step_selected_board_text_rotation(BoardTextRotationStep::Clockwise90)
            }
            HitTarget::DecreaseSelectedBoardTextLineSpacing => {
                self.step_selected_board_text_line_spacing(BoardTextLineSpacingStep::Decrease)
            }
            HitTarget::IncreaseSelectedBoardTextLineSpacing => {
                self.step_selected_board_text_line_spacing(BoardTextLineSpacingStep::Increase)
            }
            HitTarget::EditSelectedBoardTextContent => {
                self.begin_selected_board_text_content_edit()
            }
            HitTarget::EditSelectedBoardTextHeight => self.begin_selected_board_text_height_edit(),
            HitTarget::EditSelectedBoardTextRotation => {
                self.begin_selected_board_text_rotation_edit()
            }
            HitTarget::EditSelectedBoardTextLineSpacing => {
                self.begin_selected_board_text_line_spacing_edit()
            }
            HitTarget::EditSelectedBoardTextRenderIntent => {
                self.begin_selected_board_text_render_intent_edit()
            }
            HitTarget::EditSelectedBoardTextFamily => self.begin_selected_board_text_family_edit(),
            HitTarget::EditSelectedBoardTextAlignment => {
                self.begin_selected_board_text_alignment_edit()
            }
            HitTarget::TerminalTab => {
                // Owner decision 2026-08-14 (bead
                // dat-pan-trace-terminal-pollution-0j0): tab-click is
                // deliberate terminal entry, so the click stays handled even
                // when the dock is already showing the terminal —
                // `select_hit_target` then arms focus via
                // `hit_target_is_terminal_entry`.
                self.set_active_dock(DockTab::Terminal);
                true
            }
            HitTarget::TerminalSessionTab(session_id) => self.activate_terminal_session(session_id),
            HitTarget::TerminalPaneScreen(session_id) => self.activate_terminal_session(session_id),
            HitTarget::TerminalSessionClose(session_id) => self.close_terminal_session(session_id),
            HitTarget::TerminalSessionNew => self.spawn_terminal_session_tab(),
            target @ (HitTarget::TerminalSessionTerminateActive
            | HitTarget::TerminalSessionForceKillActive
            | HitTarget::TerminalSessionRetryTermination
            | HitTarget::TerminalShutdownCancel) => self.handle_terminal_lifecycle_target(target),
            HitTarget::TerminalScreen => self.click_terminal_screen(),
            HitTarget::TerminalClipboardCopy => {
                self.dismiss_terminal_clipboard_menu();
                self.copy_terminal_scrollback();
                true
            }
            HitTarget::TerminalClipboardPaste => {
                self.dismiss_terminal_clipboard_menu();
                self.paste_terminal_input();
                true
            }
            HitTarget::TerminalThemeNext => {
                self.dismiss_terminal_clipboard_menu();
                self.cycle_terminal_theme()
            }
            HitTarget::TerminalProfileNext => {
                self.dismiss_terminal_clipboard_menu();
                self.cycle_terminal_profile()
            }
            HitTarget::TerminalLinkCopy => {
                let target = self.terminal_clipboard_link_target();
                self.dismiss_terminal_clipboard_menu();
                if let Some(target) = target {
                    self.copy_terminal_link_target(&target);
                }
                true
            }
            HitTarget::TerminalLinkOpen => {
                let target = self.terminal_clipboard_link_target();
                self.dismiss_terminal_clipboard_menu();
                if let Some(target) = target {
                    self.arm_terminal_link_target(target);
                }
                true
            }
            HitTarget::TerminalLinkConfirmOpen => self.confirm_terminal_link_open(),
            HitTarget::TerminalLinkCancel => {
                self.cancel_terminal_link_confirmation();
                true
            }
            HitTarget::TerminalClipboardConfirmWrite => self.confirm_terminal_clipboard_write(),
            HitTarget::TerminalClipboardCancelWrite => self.cancel_terminal_clipboard_write(),
            HitTarget::ProductionArtifact(artifact_id) => {
                let handled = self.dispatch_session_command(
                    SessionCommand::FocusProductionArtifact(artifact_id.clone()),
                );
                if handled {
                    self.log_console_echo_for_target(
                        ConsoleFeedbackSource::Production,
                        artifact_id,
                        "Production artifact selected",
                    );
                }
                handled
            }
            HitTarget::ProductionArtifactFile(path) => {
                let handled = self.dispatch_session_command(
                    SessionCommand::FocusProductionArtifactFile(path.clone()),
                );
                if handled {
                    self.log_console_echo_for_target(
                        ConsoleFeedbackSource::Production,
                        path,
                        "Production artifact file selected",
                    );
                }
                handled
            }
            HitTarget::ProductionOutputJobRun(handoff) => {
                self.set_active_dock(DockTab::Terminal);
                let command = prepare_terminal_command_execution(
                    self.terminal_sessions.active(),
                    "production_output_job_run",
                    handoff,
                )
                .unwrap_or_else(|err| {
                    self.log_terminal_event(format!("terminal handoff prepare failed: {err}"));
                    handoff.command.clone()
                });
                let mut bytes = command.into_bytes();
                bytes.push(b'\r');
                if self.write_foreign_shell_bytes(&bytes) {
                    self.log_console_echo_for_action(
                        ConsoleFeedbackSource::Production,
                        &handoff.command,
                        "Production output command sent to Terminal",
                    );
                } else {
                    self.log_console_critical_refusal_for_action(
                        ConsoleFeedbackSource::Production,
                        &handoff.command,
                        "Production output command could not be sent to Terminal",
                    );
                }
                true
            }
            HitTarget::ProductionTerminalCommand(handoff) => {
                self.set_active_dock(DockTab::Terminal);
                let command = prepare_terminal_command_execution(
                    self.terminal_sessions.active(),
                    "production_terminal_command",
                    handoff,
                )
                .unwrap_or_else(|err| {
                    self.log_terminal_event(format!("terminal handoff prepare failed: {err}"));
                    handoff.command.clone()
                });
                let mut bytes = command.into_bytes();
                bytes.push(b'\r');
                if self.write_foreign_shell_bytes(&bytes) {
                    self.log_console_echo_for_action(
                        ConsoleFeedbackSource::Production,
                        &handoff.command,
                        "Production command sent to Terminal",
                    );
                } else {
                    self.log_console_critical_refusal_for_action(
                        ConsoleFeedbackSource::Production,
                        &handoff.command,
                        "Production command could not be sent to Terminal",
                    );
                }
                true
            }
            HitTarget::ArtifactPreviewZoomIn
            | HitTarget::ArtifactPreviewZoomOut
            | HitTarget::ArtifactPreviewReset
            | HitTarget::ToggleArtifactPreviewGeometry
            | HitTarget::ToggleArtifactPreviewDrills => self
                .select_artifact_preview_hit_target(target)
                .unwrap_or(false),
            HitTarget::ArtifactPreviewViewport => false,
            HitTarget::ConsoleHistoryToggle => self.toggle_console_history(),
            HitTarget::ConsoleHistoryFilter(filter) => {
                self.session
                    .workspace_mut()
                    .ui
                    .console
                    .set_history_filter(*filter);
                self.invalidate_frame();
                true
            }
            HitTarget::MenuTitle(_)
            | HitTarget::MenuItem { .. }
            | HitTarget::GlobalPreferencesModal
            | HitTarget::GlobalPreferencesSection(_)
            | HitTarget::GlobalPreferencesSearch
            | HitTarget::GlobalPreferencesSettingName(_)
            | HitTarget::GlobalPreferencesControl(_)
            | HitTarget::GlobalPreferencesChoice { .. }
            | HitTarget::GlobalPreferencesReset(_)
            | HitTarget::GlobalPreferencesExplanationClose
            | HitTarget::MarkingMenuItem { .. } => {
                unreachable!("application overlay targets return above")
            }
            // Divider gestures are handled directly by mouse press/release.
            HitTarget::DockResizeHandle
            | HitTarget::TerminalSplitDivider(_)
            | HitTarget::LayerScrollRegion => false,
        }
    }
}

#[cfg(test)]
mod terminal_control_input;
#[cfg(test)]
mod terminal_new_session_cwd_tests;

#[cfg(test)]
fn terminal_paste_bytes(text: &str, bracketed_paste: bool) -> Vec<u8> {
    if !bracketed_paste {
        return text.as_bytes().to_vec();
    }
    let mut bytes = Vec::with_capacity(text.len() + 12);
    bytes.extend_from_slice(b"\x1b[200~");
    bytes.extend_from_slice(text.as_bytes());
    bytes.extend_from_slice(b"\x1b[201~");
    bytes
}

#[cfg(test)]
#[path = "main_tests.rs"]
mod tests;
