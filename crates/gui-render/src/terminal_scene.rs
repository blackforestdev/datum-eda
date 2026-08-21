use crate::{CameraState, PreparedScene, RetainedScene, ShellLayout};
use datum_gui_protocol::ReviewWorkspaceState;

impl PreparedScene {
    pub fn from_workspace(
        state: &ReviewWorkspaceState,
        width: u32,
        height: u32,
        camera: CameraState,
        retained_scene: &RetainedScene,
    ) -> Self {
        Self::from_workspace_for_surface(state, width, height, 1.0, camera, retained_scene)
    }

    pub fn from_workspace_for_surface(
        state: &ReviewWorkspaceState,
        width: u32,
        height: u32,
        scale_factor: f32,
        camera: CameraState,
        retained_scene: &RetainedScene,
    ) -> Self {
        Self::from_workspace_with_terminal_snapshot(
            state,
            width,
            height,
            scale_factor,
            camera,
            retained_scene,
            None,
        )
    }

    pub fn from_workspace_with_terminal_snapshot(
        state: &ReviewWorkspaceState,
        width: u32,
        height: u32,
        scale_factor: f32,
        camera: CameraState,
        retained_scene: &RetainedScene,
        terminal_snapshot: Option<&datum_terminal_core::RenderSnapshot>,
    ) -> Self {
        let terminal_panes = terminal_snapshot
            .map(|snapshot| {
                vec![crate::TerminalPaneRenderState {
                    session_id: state
                        .ui
                        .terminal
                        .active_session_id
                        .clone()
                        .unwrap_or_else(|| "terminal".to_string()),
                    focused: true,
                    lane: state.ui.terminal.clone(),
                    snapshot: snapshot.clone(),
                    damage: Vec::new(),
                }]
            })
            .unwrap_or_default();
        Self::from_workspace_with_terminal_renderer(
            state,
            width,
            height,
            scale_factor,
            camera,
            retained_scene,
            &terminal_panes,
            None,
        )
    }
}

pub(super) fn prepare_graphics(
    state: &ReviewWorkspaceState,
    layout: &ShellLayout,
    panes: &[crate::TerminalPaneRenderState],
    sink: &mut Vec<crate::PreparedTerminalGraphic>,
) {
    if !matches!(
        state.ui.active_dock_tab,
        Some(datum_gui_protocol::DockTab::Terminal)
    ) {
        return;
    }
    if panes.is_empty() {
        return;
    }
    let root_geometry = datum_gui_viewport::terminal_screen_geometry(layout.bottom_strip.into());
    let geometries = state
        .ui
        .terminal
        .active_tab_id
        .as_deref()
        .and_then(|tab_id| {
            state
                .ui
                .terminal
                .tab_layouts
                .iter()
                .find(|tab| tab.tab_id == tab_id)
        })
        .map(|tab| datum_gui_viewport::terminal_split_geometries(root_geometry, tab))
        .unwrap_or_default();
    for pane in panes {
        let geometry = geometries
            .iter()
            .find(|candidate| candidate.session_id == pane.session_id)
            .map(|candidate| candidate.geometry)
            .unwrap_or(root_geometry);
        crate::terminal_core_render::prepare_terminal_graphics(
            &pane.snapshot,
            &geometry,
            pane.lane.scroll_offset,
            sink,
        );
    }
}

impl PreparedScene {
    pub(super) fn terminal_graphics(&self) -> &[crate::PreparedTerminalGraphic] {
        &self.terminal_graphics
    }
}
