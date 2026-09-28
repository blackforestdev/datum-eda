//! Typed world-free preparation; only this owner admits the empty source profile.
use super::{RetainedScene, ReviewWorkspaceState, session_hover::BoardHover};
use crate::Renderer;
use datum_gui_protocol::{GlobalPreferencesDialogState, NewProjectDialogState};
use datum_gui_viewport::scroll::ScrollViewport;

#[derive(Clone, Copy)]
pub struct DialogView {
    pub width: u32,
    pub height: u32,
    pub scale: f32,
}

pub enum DialogInput<'a> {
    GlobalPreferences {
        dialog: &'a GlobalPreferencesDialogState,
        reveal_row: Option<usize>,
    },
    ProjectPreferences {
        dialog: &'a GlobalPreferencesDialogState,
        reveal_row: Option<usize>,
    },
    NewProject {
        dialog: &'a NewProjectDialogState,
        reveal_focus: bool,
    },
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum PreparedProfile {
    Workspace,
    Dialog,
}
#[derive(Clone, Copy)]
pub(super) struct Preparation {
    pub extent: [u32; 2],
    pub profile: PreparedProfile,
    pub hover: BoardHover,
}

impl Preparation {
    pub(super) fn workspace(
        state: &ReviewWorkspaceState,
        retained: &RetainedScene,
        extent: [u32; 2],
    ) -> Self {
        Self {
            extent,
            profile: PreparedProfile::Workspace,
            hover: BoardHover::capture(state, retained),
        }
    }
}

impl Renderer {
    /// The platform supplies semantic dialog state and scroll intent, never a
    /// mutable prepared scene or a caller-selected rendering graph.
    pub fn prepare_session_dialog(
        &mut self,
        input: DialogInput<'_>,
        view: DialogView,
        scroll: &mut ScrollViewport,
    ) -> anyhow::Result<()> {
        use crate::resource_consumers::Consumer;
        let (mut prepared, consumer) = match input {
            DialogInput::GlobalPreferences { dialog, reveal_row } => (
                self.prepare_native_preferences_scrolled(
                    dialog,
                    view.width,
                    view.height,
                    view.scale,
                    scroll,
                    reveal_row,
                )?,
                Consumer::Global,
            ),
            DialogInput::ProjectPreferences { dialog, reveal_row } => (
                self.prepare_native_preferences_scrolled(
                    dialog,
                    view.width,
                    view.height,
                    view.scale,
                    scroll,
                    reveal_row,
                )?,
                Consumer::Project,
            ),
            DialogInput::NewProject {
                dialog,
                reveal_focus,
            } => (
                self.prepare_native_new_project_scrolled(
                    dialog,
                    view.width,
                    view.height,
                    view.scale,
                    scroll,
                    reveal_focus,
                )?,
                Consumer::New,
            ),
        };
        prepared.set_native_consumer(consumer);
        self.render_session.install_prepared(
            prepared,
            Preparation {
                extent: [view.width, view.height],
                profile: PreparedProfile::Dialog,
                hover: BoardHover::Disabled,
            },
        );
        Ok(())
    }
}
