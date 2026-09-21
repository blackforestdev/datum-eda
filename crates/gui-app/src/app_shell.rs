use super::*;

pub(super) struct App {
    pub(super) args: GuiArgs,
    pub(super) device_recovery: native_device_recovery::DeviceRecovery,
    pub(super) frames: native_frame_coordinator::NativeFrameCoordinator,
    pub(super) window: Option<std::sync::Arc<Window>>,
    pub(super) runtime: Option<Runtime>,
    pub(super) global_preferences_window: Option<std::sync::Arc<Window>>,
    pub(super) global_preferences_surface:
        Option<crate::global_preferences_window::GlobalPreferencesWindowSurface>,
    pub(super) project_preferences_window: Option<std::sync::Arc<Window>>,
    pub(super) project_preferences_surface:
        Option<crate::global_preferences_window::GlobalPreferencesWindowSurface>,
    pub(super) new_project_window: Option<std::sync::Arc<Window>>,
    pub(super) new_project_surface:
        Option<crate::global_preferences_window::GlobalPreferencesWindowSurface>,
    /// Last cursor icon set on the window, so hover-driven cursor changes only call
    /// `set_cursor` on a transition (no per-move spam).
    current_cursor: winit::window::CursorIcon,
    kwin_lifecycle_smoke_step: usize,
    pub(super) resize_smoke: crate::resize_smoke::ResizeSmoke,
    pub(super) terminal_event_proxy: winit::event_loop::EventLoopProxy<()>,
}

impl App {
    pub(super) fn run(mut self, event_loop: EventLoop<()>) -> Result<()> {
        let result = event_loop.run_app(&mut self).context("failed to run app");
        // Observe ownership only for existing verbose native proof. This weak
        // handle cannot keep the window alive during runtime/surface teardown.
        let window = std::env::var_os("DATUM_GUI_VERBOSE_LOG")
            .and_then(|_| self.window.as_ref().map(std::sync::Arc::downgrade));
        drop(self);
        if let Some(window) = window {
            append_gui_diagnostic_line(format!(
                "native Main window ownership released={}",
                window.strong_count() == 0
            ));
        }
        result
    }

    pub(super) fn request_controlled_close(&mut self, event_loop: &ActiveEventLoop) {
        append_gui_diagnostic_line("close requested");
        if let Some(runtime) = &mut self.runtime {
            runtime.begin_application_terminal_shutdown();
            self.request_redraw_if_needed();
        } else {
            event_loop.exit();
        }
    }

    pub(super) fn new(
        args: GuiArgs,
        terminal_event_proxy: winit::event_loop::EventLoopProxy<()>,
    ) -> Self {
        Self {
            args,
            frames: Default::default(),
            device_recovery: Default::default(),
            window: None,
            runtime: None,
            global_preferences_window: None,
            global_preferences_surface: None,
            project_preferences_window: None,
            project_preferences_surface: None,
            new_project_window: None,
            new_project_surface: None,
            current_cursor: winit::window::CursorIcon::Default,
            kwin_lifecycle_smoke_step: 0,
            resize_smoke: Default::default(),
            terminal_event_proxy,
        }
    }

    /// Set the window cursor for a divider-hover/drag orientation: a vertical split
    /// (left|right, dragged horizontally) reads east-west; a horizontal split
    /// (top/bottom, dragged vertically) reads north-south; `None` restores the
    /// default. Only touches the window on a change (tracked in `current_cursor`).
    pub(super) fn apply_cursor(
        &mut self,
        orientation: Option<datum_gui_protocol::SplitOrientation>,
    ) {
        use datum_gui_protocol::SplitOrientation;
        use winit::window::CursorIcon;
        let icon = match orientation {
            Some(SplitOrientation::Vertical) => CursorIcon::EwResize,
            Some(SplitOrientation::Horizontal) => CursorIcon::NsResize,
            None => CursorIcon::Default,
        };
        self.apply_cursor_icon(icon);
    }

    pub(super) fn apply_cursor_icon(&mut self, icon: winit::window::CursorIcon) {
        if self.current_cursor != icon {
            self.current_cursor = icon;
            if let Some(window) = self.window.as_deref() {
                window.set_cursor(icon);
            }
        }
    }

    /// Local damage must not invalidate independent native dialog surfaces.
    pub(super) fn request_main_redraw_if_needed(&mut self) {
        if let Some(window) = self.window.as_deref() {
            self.frames.invalidate(window);
        }
    }

    /// Workspace/terminal/console changes do not change native dialog content.
    pub(super) fn request_workspace_redraw(&mut self) {
        self.request_main_redraw_if_needed();
    }

    pub(super) fn request_redraw_if_needed(&mut self) {
        self.request_workspace_redraw();
        if let (Some(surface), Some(window)) =
            (&mut self.new_project_surface, &self.new_project_window)
        {
            surface.invalidate();
            self.frames.invalidate(window);
        }
        if let (Some(surface), Some(window)) = (
            &mut self.global_preferences_surface,
            &self.global_preferences_window,
        ) {
            surface.invalidate();
            self.frames.invalidate(window);
        }
        if let (Some(surface), Some(window)) = (
            &mut self.project_preferences_surface,
            &self.project_preferences_window,
        ) {
            surface.invalidate();
            self.frames.invalidate(window);
        }
    }

    pub(super) fn advance_kwin_lifecycle_smoke(&mut self, event_loop: &ActiveEventLoop) -> bool {
        if !self.args.kwin_lifecycle_smoke {
            return false;
        }
        let Some(window) = self.window.as_deref() else {
            return false;
        };
        match self.kwin_lifecycle_smoke_step {
            0 => {
                append_gui_diagnostic_line("kwin lifecycle smoke maximize");
                window.set_maximized(true);
            }
            1 => {
                append_gui_diagnostic_line("kwin lifecycle smoke restore 1344x806");
                window.set_maximized(false);
                let _ = window.request_inner_size(LogicalSize::new(1344.0, 806.0));
            }
            2 => {
                append_gui_diagnostic_line("kwin lifecycle smoke maximize second pass");
                window.set_maximized(true);
            }
            3 => {
                append_gui_diagnostic_line("kwin lifecycle smoke restore 1280x768");
                window.set_maximized(false);
                let _ = window.request_inner_size(LogicalSize::new(1280.0, 768.0));
            }
            _ => {
                append_gui_diagnostic_line("kwin lifecycle smoke close");
                window.set_visible(false);
                event_loop.exit();
                return true;
            }
        }
        self.kwin_lifecycle_smoke_step += 1;
        self.frames.invalidate(window);
        true
    }
}

pub(super) fn fatal_gui_error(
    event_loop: &ActiveEventLoop,
    context: &str,
    err: impl std::fmt::Display,
) -> ! {
    let message = gui_error_message(context, err);
    append_gui_diagnostic_line(format!("fatal {message}"));
    eprintln!("datum-gui error: {message}");
    event_loop.exit();
    std::process::exit(1);
}

pub(super) fn terminal_scrollback_page_step(
    workspace: &datum_gui_protocol::ReviewWorkspaceState,
) -> usize {
    usize::from(workspace.ui.terminal.rows)
        .saturating_sub(1)
        .max(1)
}

fn gui_error_message(context: &str, err: impl std::fmt::Display) -> String {
    format!("{context}: {err:#}")
}

#[cfg(test)]
mod startup_error_tests {
    #[test]
    fn startup_error_includes_underlying_filesystem_cause() {
        let error = anyhow::Error::new(std::io::Error::from(std::io::ErrorKind::NotFound))
            .context("create native project")
            .context("resolve GUI launch review context");
        let message = super::gui_error_message("launch state load failed", error);
        assert!(message.contains(
            "launch state load failed: resolve GUI launch review context: create native project:"
        ));
        assert!(message.contains("entity not found"));
    }
}
