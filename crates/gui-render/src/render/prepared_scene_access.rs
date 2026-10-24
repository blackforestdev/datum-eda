//! Buffer and hit-region access owned by the prepared-scene projection.
//!
//! Keeping these accessors together separates prepared-frame consumption from
//! scene construction while preserving one `PreparedScene` authority.

use super::*;
#[path = "pointer_selection.rs"]
mod pointer_selection;
pub use native_selection_overlay::mask::PaintRange as SelectedPaintRange;
pub use native_selection_overlay::visual_lod::SelectionVisualLod;
pub(super) use pointer_selection::SelectionHover;
#[cfg(feature = "visual")]
#[path = "native_content_text.rs"]
mod native_content_text;
#[cfg(feature = "visual")]
pub use native_content_text::NativeContentText;

#[derive(Debug, Clone, PartialEq)]
pub struct PreparedScene {
    pub(crate) schematic_sources: crate::renderer_state::SchematicSources,
    pub(super) admission_sources:
        Option<std::sync::Arc<geometry_admission::PreparedSourceIdentities>>,
    pub(super) consumers: crate::resource_consumers::FrameConsumers,
    pub layout: ShellLayout,
    pub hit_regions: Vec<HitRegion>,
    pub scene_viewport: RectPx,
    pub(super) surface_passes: Vec<PreparedSurfacePass>,
    pub(super) board_pane_id: datum_gui_protocol::PaneId,
    pub(super) scene_bounds: datum_gui_protocol::SceneBounds,
    pub(super) camera: CameraState,
    pub(super) panel_vertices: Vec<Vertex>,
    pub(super) menu_overlay_vertices: Vec<Vertex>,
    pub(super) menu_overlay_text_runs: Vec<TextRun>,
    pub(super) viewport_underlay_vertices: Vec<Vertex>,
    pub(super) viewport_overlay_vertices: Vec<Vertex>,
    pub(super) selection_overlay_split: usize,
    pub(super) selection_visual_lod: Option<SelectionVisualLod>,
    pub(super) selection_lod_row: Option<(RectPx, f32)>,
    pub(super) selection_masks: Vec<(
        datum_gui_protocol::PaneId,
        SceneSurface,
        native_selection_overlay::mask::Request,
    )>,
    pub(super) selection_strokes: Vec<selection_stroke::Stroke>,
    pub(super) pane_cues: Vec<(
        datum_gui_protocol::PaneId,
        std::sync::Arc<native_selection_overlay::cue_plan::PanePlan>,
    )>,
    pub(super) board_interaction_vertices: Vec<Vertex>,
    pub(super) console_overlay_vertices: Vec<Vertex>,
    pub(super) console_overlay_layout: Option<ConsoleOverlayLayout>,
    pub(super) visible_draw_commands: Vec<RetainedDrawCommand>,
    pub(super) world_filters: Option<datum_gui_protocol::WorkspaceFilterState>,
    pub(super) text_runs: Vec<TextRun>,
    pub(super) terminal_graphics: Vec<PreparedTerminalGraphic>,
    // Companion geometry remains resident. Its world, selected-material and
    // prepared-admission ranges all use the same snapshotted visibility below.
    pub(super) schematic_scene_viewport: Option<RectPx>,
    pub(super) schematic_pane_id: Option<datum_gui_protocol::PaneId>,
    pub(super) schematic_bounds: datum_gui_protocol::SceneBounds,
    pub(super) schematic_camera: CameraState,
    // S4 (HoverEngine): the immediate screen-space interaction overlays are class-A
    // `ScreenConstant` chrome — driven by live hover, so they are empty in the
    // offscreen visual-test capture, keeping the board frame byte-identical. Board
    // hover folds straight into `viewport_overlay_vertices` at construction; the
    // schematic hover rides `schematic_overlay_vertices`, rebuilt when the warm
    // schematic camera is applied (`set_schematic_camera`) — hence the hovered
    // symbol's world bbox is retained here to re-project it against that camera.
    pub(super) schematic_hover_bounds_nm: Option<datum_gui_protocol::RectNm>,
    /// Revision-qualified keys shared by prepared clones; pointer hover cannot
    /// add a second halo to an actual selected member.
    pub(super) selection_hover: SelectionHover,
    /// Source-qualified menu pre-highlight survives pointer suffix refreshes.
    pub(super) menu_hover: Option<(
        Option<datum_gui_protocol::RectNm>,
        Option<datum_gui_protocol::RectNm>,
    )>,
    /// Locked-placement hover anchors, so the pointer-only refresh can draw the
    /// governed padlock without re-preparing world content.
    pub(super) board_lock_anchors: crate::native_selection_overlay::lock_cue::HoverAnchors,
    // S4 cursor crosshair (decision 023 UVT-005): the live cursor in device-pixel
    // SCREEN space and the user-selected style, retained so `set_schematic_camera`
    // (which has no `state`) can rebuild the schematic overlay crosshair against
    // the warm camera. `None` cursor in the offscreen capture keeps the frame
    // byte-identical; both the board and schematic panes read these.
    pub(super) crosshair_cursor_screen: Option<(f32, f32)>,
    pub(super) crosshair_style: datum_gui_protocol::CrosshairStyle,
    pub(super) schematic_overlay_vertices: Vec<Vertex>,
}

impl PreparedScene {
    pub fn selection_visual_lod(&self) -> Option<SelectionVisualLod> {
        self.selection_visual_lod
    }
    pub(super) fn schematic_overlay_vertices(&self) -> &[Vertex] {
        &self.schematic_overlay_vertices
    }

    pub fn hit_test(&self, x: f32, y: f32) -> Option<&HitTarget> {
        self.hit_regions
            .iter()
            .rev()
            .find(|region| region.rect.contains(x, y))
            .map(|region| &region.target)
    }

    // `world_point_at_screen` (per-pane screen->world resolve, UVT-004) lives in
    // the `coordinate_hit` module alongside the world hit-test.

    pub(super) fn panel_vertices(&self) -> &[Vertex] {
        &self.panel_vertices
    }

    /// Top-overlay quads for menus and application dialogs, composited after
    /// scissored viewport passes so work-pane content cannot overpaint them.
    pub(super) fn menu_overlay_vertices(&self) -> &[Vertex] {
        &self.menu_overlay_vertices
    }

    /// Text belonging to the top overlay, rendered after its opaque cards.
    pub(super) fn menu_overlay_text_runs(&self) -> &[TextRun] {
        &self.menu_overlay_text_runs
    }

    pub(super) fn viewport_underlay_vertices(&self) -> &[Vertex] {
        &self.viewport_underlay_vertices
    }

    pub(super) fn viewport_overlay_vertices(&self) -> &[Vertex] {
        &self.viewport_overlay_vertices
    }

    pub(super) fn board_interaction_vertices(&self) -> &[Vertex] {
        &self.board_interaction_vertices
    }

    pub(super) fn visible_draw_commands(&self) -> &[RetainedDrawCommand] {
        &self.visible_draw_commands
    }

    pub(crate) fn schematic_for_pass<'a>(
        &'a self,
        pass: &PreparedSurfacePass,
        legacy: Option<&'a RetainedScene>,
    ) -> Option<&'a RetainedScene> {
        if pass.surface != SceneSurface::Schematic {
            return None;
        }
        match pass.schematic_document.as_ref() {
            Some(document) => self
                .schematic_sources
                .scene(Some(document), pass.scene_viewport),
            None => self
                .schematic_sources
                .scene(None, pass.scene_viewport)
                .or(legacy),
        }
    }
    pub(super) fn draw_commands_for_pass<'a>(
        &'a self,
        pass: &PreparedSurfacePass,
        legacy: Option<&'a RetainedScene>,
    ) -> VisibleDrawCommands<'a> {
        self.surface_draw_commands(pass.surface, self.schematic_for_pass(pass, legacy))
    }

    pub(super) fn surface_draw_commands<'a>(
        &'a self,
        surface: SceneSurface,
        schematic: Option<&'a RetainedScene>,
    ) -> VisibleDrawCommands<'a> {
        match surface {
            SceneSurface::Board => VisibleDrawCommands {
                commands: self.visible_draw_commands(),
                filters: None,
            },
            SceneSurface::Schematic => VisibleDrawCommands {
                commands: schematic.map_or(&[][..], RetainedScene::all_draw_commands),
                filters: self.world_filters.as_ref(),
            },
        }
    }
}
