//! Shared physical pane/header/scene/divider geometry for painting and input.
use super::*;

impl ShellLayout {
    /// Tile in logical pixels, then publish physical header, scene and divider
    /// bounds together. Painting, world projection and pointer input share these
    /// bounds. Maximizing a leaf leaves the workspace tree unchanged.
    pub fn viewport_panes(&self, layout: &datum_gui_protocol::WorkspaceLayout) -> ViewportPanes {
        let scale = self.top_menu_bar.height / (design_tokens::spacing::SP_07 + 1.0);
        let viewport = self.viewport.scale_by(1.0 / scale);
        let mut panes = Vec::new();
        let mut dividers = Vec::new();
        if let Some(zoomed) = layout.zoomed {
            // Maximize: the zoomed leaf fills the viewport; no siblings, no
            // dividers. The tree is never mutated — this is transient view state.
            let content = leaf_pane_content(&layout.root, zoomed)
                .unwrap_or(datum_gui_protocol::PaneContent::Board);
            panes.push(LeafPane {
                id: zoomed,
                content,
                rect: PaneRect::from_frame(viewport),
            });
        } else {
            tile_pane_node(
                &layout.root,
                viewport,
                &mut panes,
                &mut dividers,
                &mut Vec::new(),
            );
        }
        for pane in &mut panes {
            pane.rect.frame = pane.rect.frame.scale_by(scale);
            pane.rect.header = pane.rect.header.scale_by(scale);
            pane.rect.scene = pane.rect.scene.scale_by(scale);
        }
        for divider in &mut dividers {
            divider.rect = divider.rect.scale_by(scale);
            divider.split_frame = divider.split_frame.scale_by(scale);
        }
        ViewportPanes {
            panes,
            dividers,
            focused: layout.focused,
        }
    }

    pub fn scene_viewport(&self, layout: &datum_gui_protocol::WorkspaceLayout) -> RectPx {
        // The world board scene renders into the BOARD leaf's canvas — the one that
        // owns the live PCB — NOT merely whichever leaf is focused. Returning that
        // scene rect means RetainedScene's reference_projection, gpu.rs
        // scissor/uniform, and `world_point_at_screen` all follow the board pane, so
        // the PCB stays visible in its pane while another pane (e.g. Schematic) is
        // focused. Falls back to the focused rect only when no board leaf exists
        // (nothing renders there — the board scene is gated off in that case).
        let panes = self.viewport_panes(layout);
        panes
            .scene_leaf()
            .map(|leaf| leaf.rect.scene)
            .unwrap_or_else(|| panes.focused_scene())
    }

    /// The Schematic leaf's scene canvas rect, if a Schematic pane exists — the
    /// static SECOND world scene's viewport for the P2.2a multi-scene GPU pass.
    /// Unlike `scene_viewport` (which follows the single live BOARD leaf and is
    /// focus-independent), this is simply the first Schematic leaf in walk order:
    /// the companion schematic scene projects into it additively, alongside the
    /// board. `None` when the layout has no Schematic pane (e.g. an all-Board
    /// split), in which case the second GPU pass is gated off and the pane keeps
    /// its "Schematic (coming)" placeholder.
    pub fn schematic_scene_viewport(
        &self,
        layout: &datum_gui_protocol::WorkspaceLayout,
    ) -> Option<RectPx> {
        self.viewport_panes(layout)
            .panes
            .iter()
            .find(|leaf| leaf.content == datum_gui_protocol::PaneContent::Schematic)
            .map(|leaf| leaf.rect.scene)
    }

    pub(crate) fn scale_by(self, scale: f32) -> Self {
        Self {
            top_menu_bar: self.top_menu_bar.scale_by(scale),
            viewport: self.viewport.scale_by(scale),
            left_sidebar: self.left_sidebar.scale_by(scale),
            right_sidebar: self.right_sidebar.scale_by(scale),
            bottom_strip: self.bottom_strip.scale_by(scale),
            status_bar: self.status_bar.scale_by(scale),
        }
    }
}
