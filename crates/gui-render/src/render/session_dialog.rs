//! Typed world-free preparation; only this owner admits the empty source profile.
use super::{RetainedScene, ReviewWorkspaceState, session_hover::BoardHoverDependency};
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
    OpenProject {
        dialog: &'a datum_gui_protocol::ProjectOpenDialogState,
        navigation: Option<&'a datum_gui_protocol::FolderBrowserState>,
        list_scroll: &'a mut ScrollViewport,
        preview_scroll: &'a mut ScrollViewport,
        reveal_focus: bool,
    },
    NewProject {
        dialog: &'a NewProjectDialogState,
        navigation: Option<&'a datum_gui_protocol::FolderBrowserState>,
        list_scroll: &'a mut ScrollViewport,
        preview_scroll: &'a mut ScrollViewport,
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
    pub hover: BoardHoverDependency,
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
            hover: BoardHoverDependency::capture(state, retained),
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
            DialogInput::OpenProject {
                dialog,
                navigation,
                list_scroll,
                preview_scroll,
                reveal_focus,
            } => (
                self.prepare_native_project_form(
                    crate::new_project_dialog::FormData::Open(dialog),
                    navigation,
                    view,
                    scroll,
                    list_scroll,
                    preview_scroll,
                    reveal_focus,
                )?,
                Consumer::New,
            ),
            DialogInput::NewProject {
                dialog,
                navigation,
                list_scroll,
                preview_scroll,
                reveal_focus,
            } => (
                self.prepare_native_project_form(
                    crate::new_project_dialog::FormData::New(dialog),
                    navigation,
                    view,
                    scroll,
                    list_scroll,
                    preview_scroll,
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
                hover: BoardHoverDependency::Disabled,
            },
        );
        Ok(())
    }
}
