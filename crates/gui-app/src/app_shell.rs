use super::*;

pub(super) struct App {
    pub(super) args: GuiArgs,
    pub(super) window: Option<&'static Window>,
    pub(super) runtime: Option<Runtime>,
    /// Last cursor icon set on the window, so hover-driven cursor changes only call
    /// `set_cursor` on a transition (no per-move spam).
    current_cursor: winit::window::CursorIcon,
    kwin_lifecycle_smoke_step: usize,
}

impl App {
    pub(super) fn new(args: GuiArgs) -> Self {
        Self {
            args,
            window: None,
            runtime: None,
            current_cursor: winit::window::CursorIcon::Default,
            kwin_lifecycle_smoke_step: 0,
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
        if self.current_cursor != icon {
            self.current_cursor = icon;
            if let Some(window) = self.window {
                window.set_cursor(icon);
            }
        }
    }

    pub(super) fn request_redraw_if_needed(&mut self) {
        if let (Some(runtime), Some(window)) = (&mut self.runtime, self.window)
            && !runtime.redraw_pending
        {
            runtime.redraw_pending = true;
            window.request_redraw();
        }
    }

    pub(super) fn advance_kwin_lifecycle_smoke(&mut self, event_loop: &ActiveEventLoop) -> bool {
        if !self.args.kwin_lifecycle_smoke {
            return false;
        }
        let Some(window) = self.window else {
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
        window.request_redraw();
        true
    }
}
