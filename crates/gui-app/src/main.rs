#[path = "rendering/native_device_recovery.rs"]
mod native_device_recovery;
#[path = "editor/native_edit_selection.rs"]
mod native_edit_selection;
#[path = "editor/runtime_library.rs"]
mod runtime_library;
#[path = "editor/runtime_native_edit.rs"]
mod runtime_native_edit;
#[path = "editor/runtime_schematic.rs"]
mod runtime_schematic;
#[path = "editor/runtime_schematic_navigation.rs"]
mod runtime_schematic_navigation;
#[path = "application/runtime_state.rs"]
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
#[path = "application/app_bootstrap.rs"]
mod app_bootstrap;
#[path = "application/app_frame.rs"]
mod app_frame;
#[path = "application/app_native_events.rs"]
mod app_native_events;
#[path = "application/app_shell.rs"]
mod app_shell;
#[path = "terminal/application_terminal_shutdown.rs"]
mod application_terminal_shutdown;
#[path = "editor/artifact_preview_controls.rs"]
mod artifact_preview_controls;
#[path = "editor/board_text_terminal_commands.rs"]
mod board_text_terminal_commands;
#[path = "terminal/console_accessibility.rs"]
mod console_accessibility;
#[path = "terminal/console_feedback.rs"]
mod console_feedback;
#[path = "windows/folder_browser.rs"]
mod folder_browser;
#[path = "windows/global_preferences_projection.rs"]
mod global_preferences_projection;
#[path = "windows/global_preferences_runtime.rs"]
mod global_preferences_runtime;
#[path = "windows/global_preferences_window.rs"]
mod global_preferences_window;
#[path = "rendering/gui_runtime_support.rs"]
mod gui_runtime_support;
#[path = "application/host_filesystem.rs"]
mod host_filesystem;
#[path = "viewport/interaction_refresh.rs"]
mod interaction_refresh;
#[path = "terminal/keyboard_focus.rs"]
mod keyboard_focus;
#[path = "application/local_path.rs"]
mod local_path;
#[path = "rendering/native_content_probe.rs"]
mod native_content_probe;
#[path = "rendering/native_frame_adapters.rs"]
mod native_frame_adapters;
#[path = "rendering/native_frame_coordinator.rs"]
mod native_frame_coordinator;
#[path = "rendering/native_gpu.rs"]
mod native_gpu;
#[path = "rendering/native_gpu_measurements.rs"]
mod native_gpu_measurements;
#[path = "rendering/native_resize_repro.rs"]
mod native_resize_repro;
#[path = "windows/new_project_window.rs"]
mod new_project_window;
#[path = "application/owned_window_policy.rs"]
mod owned_window_policy;
#[path = "viewport/pan_gesture.rs"]
mod pan_gesture;
#[path = "viewport/pane_cameras.rs"]
mod pane_cameras;
#[path = "viewport/pane_grid_lod.rs"]
mod pane_grid_lod;
#[path = "viewport/pane_resize.rs"]
mod pane_resize;
#[path = "application/production_status_refresh.rs"]
mod production_status_refresh;
#[path = "windows/project_open.rs"]
mod project_open;
#[path = "windows/project_open_window.rs"]
mod project_open_window;
#[path = "windows/project_preferences_runtime.rs"]
mod project_preferences_runtime;
#[path = "windows/project_preferences_window.rs"]
mod project_preferences_window;
#[path = "rendering/resize_smoke.rs"]
mod resize_smoke;
#[path = "rendering/retained_scene_lifecycle.rs"]
mod retained_scene_lifecycle;
#[path = "rendering/runtime_capture.rs"]
mod runtime_capture;
#[path = "application/runtime_clipboard.rs"]
mod runtime_clipboard;
use runtime_capture::run_offscreen_visual_test;
#[path = "editor/runtime_board_text_edit.rs"]
mod runtime_board_text_edit;
#[path = "viewport/runtime_camera_fit_targets.rs"]
mod runtime_camera_fit_targets;
#[path = "viewport/runtime_camera_pane.rs"]
mod runtime_camera_pane;
#[path = "editor/runtime_check_runs.rs"]
mod runtime_check_runs;
#[path = "viewport/runtime_geometry_helpers.rs"]
mod runtime_geometry_helpers;
#[path = "editor/runtime_hit_actions.rs"]
mod runtime_hit_actions;
#[path = "application/runtime_init.rs"]
mod runtime_init;
#[path = "application/runtime_menu_actions.rs"]
mod runtime_menu_actions;
#[path = "viewport/runtime_pane_lifecycle.rs"]
mod runtime_pane_lifecycle;
#[path = "rendering/runtime_present.rs"]
mod runtime_present;
#[path = "editor/runtime_primary_button.rs"]
mod runtime_primary_button;
#[path = "editor/runtime_primary_pointer.rs"]
mod runtime_primary_pointer;
#[path = "editor/runtime_revision_workspace.rs"]
mod runtime_revision_workspace;
#[path = "editor/runtime_selection.rs"]
mod runtime_selection;
#[path = "editor/runtime_selection_artifacts.rs"]
mod runtime_selection_artifacts;
#[path = "editor/runtime_selection_hit.rs"]
mod runtime_selection_hit;
#[path = "editor/runtime_selection_inspector.rs"]
mod runtime_selection_inspector;
#[path = "editor/runtime_selection_menu.rs"]
mod runtime_selection_menu;
#[path = "editor/runtime_selection_region.rs"]
mod runtime_selection_region;
#[path = "editor/runtime_selection_run.rs"]
mod runtime_selection_run;
#[path = "rendering/runtime_surface.rs"]
mod runtime_surface;
#[path = "terminal/runtime_terminal_clipboard.rs"]
mod runtime_terminal_clipboard;
#[path = "terminal/runtime_terminal_context.rs"]
mod runtime_terminal_context;
#[path = "terminal/runtime_terminal_control.rs"]
mod runtime_terminal_control;
#[path = "terminal/runtime_terminal_dock.rs"]
mod runtime_terminal_dock;
#[path = "terminal/runtime_terminal_font.rs"]
mod runtime_terminal_font;
#[path = "terminal/runtime_terminal_geometry.rs"]
mod runtime_terminal_geometry;
#[path = "terminal/runtime_terminal_input.rs"]
mod runtime_terminal_input;
#[path = "terminal/runtime_terminal_links.rs"]
mod runtime_terminal_links;
#[path = "terminal/runtime_terminal_notifications.rs"]
mod runtime_terminal_notifications;
#[path = "terminal/runtime_terminal_pointer.rs"]
mod runtime_terminal_pointer;
#[path = "terminal/runtime_terminal_profile.rs"]
mod runtime_terminal_profile;
#[path = "terminal/runtime_terminal_render.rs"]
mod runtime_terminal_render;
#[path = "terminal/runtime_terminal_search.rs"]
mod runtime_terminal_search;
#[path = "terminal/runtime_terminal_theme.rs"]
mod runtime_terminal_theme;
#[path = "editor/runtime_tool_actions.rs"]
mod runtime_tool_actions;
#[path = "editor/runtime_view_actions.rs"]
mod runtime_view_actions;
#[path = "editor/selection_accessibility.rs"]
mod selection_accessibility;
#[path = "editor/selection_region_evaluation.rs"]
mod selection_region_evaluation;
#[path = "terminal/terminal_accessibility.rs"]
mod terminal_accessibility;
#[path = "terminal/terminal_accessibility_bridge.rs"]
mod terminal_accessibility_bridge;
#[path = "terminal/terminal_accessibility_platform/mod.rs"]
mod terminal_accessibility_platform;
#[path = "terminal/terminal_active_context.rs"]
mod terminal_active_context;
#[path = "terminal/terminal_activity_snapshot.rs"]
mod terminal_activity_snapshot;
#[path = "terminal/terminal_agent_authority.rs"]
mod terminal_agent_authority;
#[path = "terminal/terminal_agent_credential.rs"]
mod terminal_agent_credential;
#[path = "terminal/terminal_capability.rs"]
mod terminal_capability;
#[path = "terminal/terminal_check_context.rs"]
mod terminal_check_context;
#[path = "terminal/terminal_context.rs"]
mod terminal_context;
#[path = "terminal/terminal_context_contract.rs"]
mod terminal_context_contract;
#[path = "terminal/terminal_context_io.rs"]
mod terminal_context_io;
#[path = "terminal/terminal_core_adapter.rs"]
mod terminal_core_adapter;
#[path = "terminal/terminal_input.rs"]
mod terminal_input;
#[path = "terminal/terminal_process.rs"]
mod terminal_process;
#[path = "terminal/terminal_profile.rs"]
mod terminal_profile;
#[path = "terminal/terminal_proposal_context.rs"]
mod terminal_proposal_context;
#[path = "terminal/terminal_session.rs"]
mod terminal_session;
#[path = "terminal/terminal_session_context.rs"]
mod terminal_session_context;
#[path = "terminal/terminal_session_controls.rs"]
mod terminal_session_controls;
#[path = "terminal/terminal_session_events.rs"]
mod terminal_session_events;
#[path = "terminal/terminal_split_drag.rs"]
mod terminal_split_drag;
#[path = "terminal/terminal_tab_drag.rs"]
mod terminal_tab_drag;
#[path = "terminal/terminal_transport/mod.rs"]
mod terminal_transport;
#[path = "terminal/terminal_working_directory.rs"]
mod terminal_working_directory;
#[path = "application/workspace_keyboard.rs"]
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

#[cfg(test)]
#[path = "terminal/terminal_control_input.rs"]
mod terminal_control_input;
#[cfg(test)]
#[path = "terminal/terminal_new_session_cwd_tests.rs"]
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
#[path = "application/main_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "editor/native_move_frame_tests.rs"]
mod native_move_frame_tests;

#[cfg(all(test, feature = "visual"))]
#[path = "editor/native_library_frame_tests.rs"]
mod native_library_frame_tests;
#[cfg(test)]
#[path = "editor/native_schematic_frame_tests.rs"]
mod native_schematic_frame_tests;

#[path = "application/native_project_startup.rs"]
mod native_project_startup;
