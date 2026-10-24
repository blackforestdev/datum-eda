//! Shared runtime focus, split, close, preset and editor-fill lifecycle.
use super::*;

impl Runtime {
    /// Keep the live Board camera bound to its leaf across all focus changes.
    pub(super) fn swap_pane_focus(
        &mut self,
        f: impl FnOnce(&mut datum_gui_protocol::WorkspaceLayout),
    ) {
        self.change_pane_views(|workspace| f(&mut workspace.ui.layout));
    }

    pub(super) fn change_pane_views(
        &mut self,
        f: impl FnOnce(&mut datum_gui_protocol::ReviewWorkspaceState),
    ) {
        let documents_before = self
            .workspace()
            .schematic_documents
            .visible_keys(&self.workspace().ui.layout);
        let outgoing_scene = self.scene_leaf_id();
        let focused_pane = {
            let workspace = self.session.workspace_mut();
            f(workspace);
            workspace
                .schematic_documents
                .prune_views(&workspace.ui.layout);
            workspace.ui.layout.focused
        };
        self.set_application_focus(datum_gui_protocol::ApplicationFocus::Editor(focused_pane));
        let incoming_scene = self.scene_leaf_id();
        if let (Some(outgoing), Some(incoming)) = (outgoing_scene, incoming_scene)
            && outgoing != incoming
        {
            let bounds = self.workspace().scene.bounds.clone();
            self.camera = self.pane_cameras.focus_to(
                outgoing,
                datum_gui_protocol::PaneContent::Board,
                self.camera,
                incoming,
                datum_gui_protocol::PaneContent::Board,
                || CameraState::fit_to_bounds(&bounds),
            );
        }
        if documents_before
            != self
                .workspace()
                .schematic_documents
                .visible_keys(&self.workspace().ui.layout)
        {
            self.document_sources_changed();
        }
        self.invalidate_frame();
    }

    pub(super) fn pane_focus_next(&mut self) {
        self.swap_pane_focus(|layout| layout.focus_next());
    }

    pub(super) fn pane_focus_prev(&mut self) {
        self.swap_pane_focus(|layout| layout.focus_prev());
    }

    pub(super) fn pane_split_focused(&mut self, orientation: datum_gui_protocol::SplitOrientation) {
        let focused = self.workspace().ui.layout.focused;
        let content = self.workspace().ui.layout.focused_content();
        let inherited = self.pane_camera_snapshot(focused);
        let mut pane = None;
        self.change_pane_views(|state| {
            pane = Some(
                state
                    .schematic_documents
                    .split_view(&mut state.ui.layout, orientation),
            );
        });
        if let (Some(pane), Some(camera)) = (pane, inherited) {
            self.pane_cameras.inherit(pane, content, camera);
        }
    }

    pub(super) fn pane_close_focused(&mut self) {
        self.cancel_native_selection_gesture();
        self.swap_pane_focus(|layout| layout.close_focused());
        let live = self.workspace().ui.layout.leaves();
        // Prune after the focus transition, which stashes the outgoing camera.
        // Otherwise closing the active Board would reinsert its dead pane id.
        self.pane_cameras.retain_live(&live);
        self.renderer
            .render_session_mut()
            .retain_pane_projections(&live);
    }

    pub(super) fn pane_toggle_zoom(&mut self) {
        // Transient maximize of the focused leaf; focus and cameras are untouched.
        self.change_pane_views(|state| state.ui.layout.toggle_zoom());
    }

    pub(super) fn pane_apply_preset(&mut self, preset: datum_gui_protocol::WorkspacePreset) {
        self.change_pane_views(|state| {
            state.ui.layout.apply_preset(preset);
            state.schematic_documents.reset_views(&state.ui.layout);
        });
        // A preset rebuilds the tree with fresh ids; reset the warm store to the
        // new focused leaf and fit it (a preset is a deliberate layout reset).
        let focused = self.workspace().ui.layout.focused;
        let bounds = self.workspace().scene.bounds.clone();
        self.camera = CameraState::fit_to_bounds(&bounds);
        let content = self.workspace().ui.layout.focused_content();
        self.pane_cameras.reset(focused, content, self.camera);
        self.renderer.render_session_mut().reset_pane_projections();
        self.invalidate_frame();
    }

    pub(super) fn pane_set_focused_content(&mut self, content: datum_gui_protocol::PaneContent) {
        let documents_before = self
            .workspace()
            .schematic_documents
            .visible_keys(&self.workspace().ui.layout);
        self.cancel_native_selection_gesture();
        let retargeted_pane = self.workspace().ui.layout.focused;
        let outgoing_board = self.scene_leaf_id();
        let outgoing_camera = self.camera;
        self.session
            .workspace_mut()
            .ui
            .layout
            .set_focused_content(content);
        if content == datum_gui_protocol::PaneContent::Schematic {
            self.session
                .workspace_mut()
                .schematic_documents
                .bind_initial_view(retargeted_pane);
            self.prepare_focused_document();
        }
        self.set_application_focus(datum_gui_protocol::ApplicationFocus::Editor(
            retargeted_pane,
        ));
        self.renderer
            .render_session_mut()
            .retire_pane_projection(retargeted_pane, content);
        let incoming_board = self.scene_leaf_id();
        if outgoing_board != incoming_board {
            let bounds = self.workspace().scene.bounds.clone();
            match (outgoing_board, incoming_board) {
                (Some(outgoing), Some(incoming)) => {
                    self.camera = self.pane_cameras.focus_to(
                        outgoing,
                        datum_gui_protocol::PaneContent::Board,
                        outgoing_camera,
                        incoming,
                        datum_gui_protocol::PaneContent::Board,
                        || CameraState::fit_to_bounds(&bounds),
                    );
                }
                (Some(outgoing), None) => {
                    self.pane_cameras.inherit(
                        outgoing,
                        datum_gui_protocol::PaneContent::Board,
                        outgoing_camera,
                    );
                }
                (None, Some(incoming)) => {
                    self.camera = *self.pane_cameras.entry_or_insert_with(
                        incoming,
                        datum_gui_protocol::PaneContent::Board,
                        || CameraState::fit_to_bounds(&bounds),
                    );
                }
                (None, None) => {}
            }
        }
        // Retire the old binding after stashing the outgoing active camera,
        // independently of whether the new content has a resolved viewport.
        self.pane_cameras.retarget(retargeted_pane, content);
        self.initialize_focused_camera();
        if documents_before
            != self
                .workspace()
                .schematic_documents
                .visible_keys(&self.workspace().ui.layout)
        {
            self.document_sources_changed();
        }
        self.invalidate_frame();
    }

    fn prepare_focused_document(&mut self) {
        let state = self.workspace();
        let Some(key) = state
            .schematic_documents
            .panes
            .get(&state.ui.layout.focused)
            .cloned()
        else {
            return;
        };
        let Some(model) = state.library.basis.clone() else {
            return;
        };
        let Some(root) = state
            .backing
            .as_ref()
            .map(|b| b.request.project_root.clone())
        else {
            return;
        };
        if let Err(error) = self
            .session
            .workspace_mut()
            .schematic_documents
            .prepare_scene(&key, &root, &model)
        {
            self.log_console_refusal(ConsoleFeedbackSource::Viewport, error.to_string());
        }
    }
}
