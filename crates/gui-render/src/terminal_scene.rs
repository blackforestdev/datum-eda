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
        Self::from_workspace_with_terminal_renderer(
            state,
            width,
            height,
            scale_factor,
            camera,
            retained_scene,
            terminal_snapshot,
            &[],
            None,
        )
    }
}

pub(super) fn prepare_graphics(
    state: &ReviewWorkspaceState,
    layout: &ShellLayout,
    snapshot: Option<&datum_terminal_core::RenderSnapshot>,
    sink: &mut Vec<crate::PreparedTerminalGraphic>,
) {
    if !matches!(
        state.ui.active_dock_tab,
        Some(datum_gui_protocol::DockTab::Terminal)
    ) {
        return;
    }
    let Some(snapshot) = snapshot else {
        return;
    };
    let geometry = datum_gui_viewport::terminal_screen_geometry(layout.bottom_strip.into());
    crate::terminal_core_render::prepare_terminal_graphics(
        snapshot,
        &geometry,
        state.ui.terminal.scroll_offset,
        sink,
    );
}

impl PreparedScene {
    pub(super) fn terminal_graphics(&self) -> &[crate::PreparedTerminalGraphic] {
        &self.terminal_graphics
    }
}
