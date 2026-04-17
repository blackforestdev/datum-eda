use datum_gui_protocol::{
    BoardGraphicPrimitive, BoardReviewSceneV1, ComponentGraphicPrimitive, ComponentTextPrimitive,
    DockTab, PointNm,
    ProposalOverlayPrimitive, ReviewActionRow, ReviewWorkspaceState, SelectionTarget,
    UnroutedPrimitive, WorkspaceTool,
};
use eda_engine::board::BoardText;
use eda_engine::export::render_silkscreen_text_strokes;
use eda_engine::ir::geometry::{LayerId, Point};
use glyphon::{
    Attrs, Buffer, Cache, Color, Family, FontSystem, Metrics, Resolution, Shaping, SwashCache,
    TextArea, TextAtlas, TextBounds, TextRenderer, Viewport,
};
use uuid::Uuid;
use wgpu::util::DeviceExt;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RectPx {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl RectPx {
    pub fn contains(self, x: f32, y: f32) -> bool {
        x >= self.x && x <= self.x + self.width && y >= self.y && y <= self.y + self.height
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CameraState {
    pub center_x_nm: f32,
    pub center_y_nm: f32,
    pub zoom: f32,
}

impl CameraState {
    pub fn fit_to_bounds(bounds: &datum_gui_protocol::SceneBounds) -> Self {
        Self {
            center_x_nm: ((bounds.min_x + bounds.max_x) as f32) * 0.5,
            center_y_nm: ((bounds.min_y + bounds.max_y) as f32) * 0.5,
            zoom: 1.0,
        }
    }

    pub fn pan_pixels(
        &mut self,
        viewport: RectPx,
        bounds: &datum_gui_protocol::SceneBounds,
        delta_x_px: f32,
        delta_y_px: f32,
    ) {
        let projection = Projection::new(viewport, bounds, *self);
        self.center_x_nm -= delta_x_px / projection.scale;
        self.center_y_nm -= delta_y_px / projection.scale;
    }

    pub fn zoom_about_screen_point(
        &mut self,
        viewport: RectPx,
        bounds: &datum_gui_protocol::SceneBounds,
        screen_x: f32,
        screen_y: f32,
        zoom_delta: f32,
    ) {
        let before = Projection::new(viewport, bounds, *self).screen_to_world(screen_x, screen_y);
        self.zoom = (self.zoom * zoom_delta).clamp(0.35, 8.0);
        let after = Projection::new(viewport, bounds, *self).screen_to_world(screen_x, screen_y);
        self.center_x_nm += before.x as f32 - after.x as f32;
        self.center_y_nm += before.y as f32 - after.y as f32;
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ShellLayout {
    pub viewport: RectPx,
    pub left_sidebar: RectPx,
    pub right_sidebar: RectPx,
    pub bottom_strip: RectPx,
}

impl ShellLayout {
    pub fn for_window(width: u32, height: u32, dock_height_px: Option<u32>) -> Self {
        let width = width as f32;
        let height = height as f32;
        let left_width = 280.0_f32.min(width * 0.3);
        let right_width = 340.0_f32.min(width * 0.35);
        let bottom_height = match dock_height_px {
            Some(h) => (h as f32).clamp(44.0, height * 0.6),
            None => 44.0_f32.min(height * 0.25),
        };
        Self {
            left_sidebar: RectPx {
                x: 0.0,
                y: 0.0,
                width: left_width,
                height: height - bottom_height,
            },
            viewport: RectPx {
                x: left_width,
                y: 0.0,
                width: (width - left_width - right_width).max(0.0),
                height: height - bottom_height,
            },
            right_sidebar: RectPx {
                x: (width - right_width).max(0.0),
                y: 0.0,
                width: right_width,
                height: height - bottom_height,
            },
            bottom_strip: RectPx {
                x: 0.0,
                y: height - bottom_height,
                width,
                height: bottom_height,
            },
        }
    }

    pub fn scene_viewport(&self) -> RectPx {
        inset_rect(self.viewport, 16.0, 76.0, 16.0, 16.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HitTarget {
    ReviewAction(String),
    AuthoredObject(String),
    FitBoard,
    FitReviewTarget,
    ReviewPrev,
    ReviewNext,
    ToggleShowAuthored,
    ToggleShowProposed,
    ToggleShowUnrouted,
    ToggleDimUnrelated,
    ToggleLayer(String),
    TerminalTab,
    AssistantTab,
    DockResizeHandle,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HitRegion {
    pub target: HitTarget,
    pub rect: RectPx,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PreparedScene {
    pub layout: ShellLayout,
    pub hit_regions: Vec<HitRegion>,
    pub scene_viewport: RectPx,
    scene_bounds: datum_gui_protocol::SceneBounds,
    camera: CameraState,
    panel_vertices: Vec<Vertex>,
    viewport_underlay_vertices: Vec<Vertex>,
    viewport_overlay_vertices: Vec<Vertex>,
    text_runs: Vec<TextRun>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetainedScene {
    world_vertices: Vec<Vertex>,
    world_hit_regions: Vec<WorldHitRegion>,
}

#[derive(Debug, Clone, PartialEq)]
struct WorldHitRegion {
    target: HitTarget,
    shape: WorldHitShape,
}

#[derive(Debug, Clone, PartialEq)]
enum WorldHitShape {
    Rect(datum_gui_protocol::RectNm),
    Polyline {
        path: Vec<PointNm>,
        half_width_nm: f32,
    },
    Circle {
        center: PointNm,
        radius_nm: f32,
    },
}

#[derive(Debug, Clone, PartialEq)]
struct Projection {
    viewport: RectPx,
    bounds: datum_gui_protocol::SceneBounds,
    camera: CameraState,
    scale: f32,
    offset_x: f32,
    offset_y: f32,
}

impl Projection {
    fn new(
        viewport: RectPx,
        bounds: &datum_gui_protocol::SceneBounds,
        camera: CameraState,
    ) -> Self {
        let scene_width = (bounds.max_x - bounds.min_x).max(1) as f32;
        let scene_height = (bounds.max_y - bounds.min_y).max(1) as f32;
        let fit_scale = (viewport.width / scene_width)
            .min(viewport.height / scene_height)
            .max(0.000_001);
        let scale = (fit_scale * camera.zoom).max(0.000_001);
        let center_x = viewport.x + viewport.width * 0.5;
        let center_y = viewport.y + viewport.height * 0.5;
        let offset_x = center_x - (camera.center_x_nm - bounds.min_x as f32) * scale;
        let offset_y = center_y - (camera.center_y_nm - bounds.min_y as f32) * scale;
        Self {
            viewport,
            bounds: bounds.clone(),
            camera,
            scale,
            offset_x,
            offset_y,
        }
    }

    fn project_point(&self, point: PointNm) -> (f32, f32) {
        (
            self.offset_x + (point.x - self.bounds.min_x) as f32 * self.scale,
            self.offset_y + (point.y - self.bounds.min_y) as f32 * self.scale,
        )
    }

    fn project_rect(&self, rect: datum_gui_protocol::RectNm) -> RectPx {
        let (x0, y0) = self.project_point(PointNm {
            x: rect.min_x,
            y: rect.min_y,
        });
        let (x1, y1) = self.project_point(PointNm {
            x: rect.max_x,
            y: rect.max_y,
        });
        RectPx {
            x: x0,
            y: y0,
            width: (x1 - x0).max(1.0),
            height: (y1 - y0).max(1.0),
        }
    }

    fn world_length_to_px(&self, length_nm: i64) -> f32 {
        length_nm as f32 * self.scale
    }

    fn screen_to_world(&self, x: f32, y: f32) -> PointNm {
        PointNm {
            x: ((x - self.offset_x) / self.scale + self.bounds.min_x as f32).round() as i64,
            y: ((y - self.offset_y) / self.scale + self.bounds.min_y as f32).round() as i64,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Quad {
    points: [(f32, f32); 4],
    color: [f32; 3],
}

impl Quad {
    fn from_rect(rect: RectPx, color: [f32; 3]) -> Self {
        Self {
            points: [
                (rect.x, rect.y),
                (rect.x + rect.width, rect.y),
                (rect.x + rect.width, rect.y + rect.height),
                (rect.x, rect.y + rect.height),
            ],
            color,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum TextFace {
    Ui,
    Mono,
}

#[derive(Debug, Clone, PartialEq)]
struct TextRun {
    text: String,
    x: f32,
    y: f32,
    size: f32,
    color: [f32; 3],
    face: TextFace,
    clip_bounds: Option<RectPx>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct TextBufferKey {
    text: String,
    size_bits: u32,
    face: TextFace,
    width_px: u32,
    height_px: u32,
}

struct CachedTextBuffer {
    key: TextBufferKey,
    buffer: Buffer,
}

const APP_BG: [f32; 3] = [0.07, 0.08, 0.09];
const PANEL_BG: [f32; 3] = [0.11, 0.12, 0.14];
const PANEL_CARD_BG: [f32; 3] = [0.14, 0.15, 0.18];
const PANEL_CARD_BORDER: [f32; 3] = [0.20, 0.22, 0.26];
const VIEWPORT_BG: [f32; 3] = [0.08, 0.09, 0.11];
const VIEWPORT_FRAME: [f32; 3] = [0.16, 0.18, 0.21];
const BOARD_OUTER_FIELD: [f32; 3] = [0.07, 0.10, 0.11];
const BOARD_INNER_FIELD: [f32; 3] = [0.16, 0.22, 0.21];
const BOARD_GRID_MAJOR: [f32; 3] = [0.22, 0.30, 0.29];
const BOARD_GRID_MINOR: [f32; 3] = [0.17, 0.24, 0.23];
const BOARD_EDGE: [f32; 3] = [0.88, 0.90, 0.84];
const TEXT_PRIMARY: [f32; 3] = [0.92, 0.93, 0.95];
const TEXT_SECONDARY: [f32; 3] = [0.62, 0.66, 0.72];
const TEXT_MUTED: [f32; 3] = [0.48, 0.52, 0.58];
const TEXT_ACCENT: [f32; 3] = [0.96, 0.78, 0.41];
const TEXT_PANEL_VALUE: [f32; 3] = [0.95, 0.96, 0.98];
const COMPONENT_BODY: [f32; 3] = [0.14, 0.18, 0.17];
const COMPONENT_BODY_RELATED: [f32; 3] = [0.18, 0.23, 0.22];
const COMPONENT_BODY_SELECTED: [f32; 3] = [0.24, 0.30, 0.28];
const COMPONENT_HEADER: [f32; 3] = [0.09, 0.10, 0.11];
const COMPONENT_OUTLINE: [f32; 3] = [0.72, 0.74, 0.79];
const COMPONENT_MECHANICAL: [f32; 3] = [0.42, 0.49, 0.46];
const COMPONENT_MECHANICAL_RELATED: [f32; 3] = [0.73, 0.82, 0.74];
const COMPONENT_SILK: [f32; 3] = [0.89, 0.91, 0.82];
const COMPONENT_SILK_RELATED: [f32; 3] = [0.98, 0.97, 0.87];
const PAD_COPPER: [f32; 3] = [0.84, 0.48, 0.22];
const PAD_COPPER_RELATED: [f32; 3] = [0.93, 0.68, 0.39];
const TOP_MASK_OPENING: [f32; 3] = [0.70, 0.44, 0.78];
const BOTTOM_MASK_OPENING: [f32; 3] = [0.44, 0.72, 0.82];
const TOP_PASTE_OPENING: [f32; 3] = [0.89, 0.86, 0.76];
const BOTTOM_PASTE_OPENING: [f32; 3] = [0.72, 0.83, 0.87];
const AUTHOR_BASE: [f32; 3] = [0.84, 0.48, 0.22];
const AUTHOR_RELATED: [f32; 3] = [0.93, 0.72, 0.47];
const AUTHOR_SELECTED: [f32; 3] = [0.85, 0.95, 1.00];
const PROPOSAL_BASE: [f32; 3] = [0.98, 0.72, 0.22];
const PROPOSAL_FOCUS: [f32; 3] = [1.00, 0.88, 0.48];
const PROPOSAL_UNDERLAY: [f32; 3] = [0.47, 0.26, 0.07];
const PROPOSAL_OUTER: [f32; 3] = [0.86, 0.58, 0.16];
const PROPOSAL_CENTERLINE: [f32; 3] = [1.00, 0.97, 0.86];
const PROPOSAL_ANCHOR_RING: [f32; 3] = [1.00, 0.89, 0.58];
const PROPOSAL_ANCHOR_CORE: [f32; 3] = [0.31, 0.19, 0.08];
const DIAGNOSTIC_BASE: [f32; 3] = [0.48, 0.78, 0.82];
const DIAGNOSTIC_FOCUS: [f32; 3] = [0.72, 0.93, 0.97];
const UNROUTED_BASE: [f32; 3] = [0.66, 0.86, 0.90];
const UNROUTED_FOCUS: [f32; 3] = [0.86, 0.96, 0.98];
const DIAGNOSTIC_UNDERLAY: [f32; 3] = [0.18, 0.32, 0.35];
const AUTHORED_DIM_FACTOR: f32 = 0.82;
const PROCESS_DIM_FACTOR: f32 = 0.88;
const STRUCTURAL_DIM_FACTOR: f32 = 0.74;
const CONTEXT_DIM_FACTOR: f32 = 0.90;
const REVIEW_ROW_BG: [f32; 3] = [0.16, 0.17, 0.20];
const REVIEW_ROW_ACTIVE_BG: [f32; 3] = [0.27, 0.19, 0.11];
const REVIEW_ROW_BADGE: [f32; 3] = [0.23, 0.25, 0.29];
const REVIEW_ROW_BADGE_ACTIVE: [f32; 3] = [0.63, 0.43, 0.16];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LayerFamily {
    TopCopper,
    InnerCopper,
    BottomCopper,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum RenderStage {
    BottomCopper,
    InnerCopper,
    TopCopper,
    BottomPaste,
    TopPaste,
    BottomMask,
    TopMask,
    BottomSilk,
    TopSilk,
    Mechanical,
    Edge,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct LayerAppearance {
    authored_track: [f32; 3],
    pad_copper: [f32; 3],
    pad_related: [f32; 3],
    zone_fill: [f32; 3],
    zone_outline: [f32; 3],
    proposal: [f32; 3],
    silkscreen: [f32; 3],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BoardSurfaceRole {
    OuterField,
    InnerField,
    GridMajor,
    GridMinor,
    Edge,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum DetailTier {
    Coarse,
    Normal,
    Fine,
}

impl PreparedScene {
    pub fn from_workspace(
        state: &ReviewWorkspaceState,
        width: u32,
        height: u32,
        camera: CameraState,
        _retained_scene: &RetainedScene,
    ) -> Self {
        let layout = ShellLayout::for_window(width, height, dock_height_for_state(state));
        let mut panel_quads = Vec::new();
        let mut viewport_underlay_quads = Vec::new();
        let mut viewport_overlay_quads = Vec::new();
        let mut text_runs = Vec::new();
        let mut hit_regions = Vec::new();
        let scene_viewport = layout.scene_viewport();

        panel_quads.push(Quad::from_rect(layout.left_sidebar, PANEL_BG));
        panel_quads.push(Quad::from_rect(layout.right_sidebar, PANEL_BG));
        panel_quads.push(Quad::from_rect(layout.bottom_strip, PANEL_BG));
        viewport_underlay_quads.push(Quad::from_rect(layout.viewport, VIEWPORT_BG));

        render_side_panels(
            state,
            &layout,
            &mut panel_quads,
            &mut text_runs,
            &mut hit_regions,
        );
        render_bottom_tabs(
            state,
            &layout,
            &mut panel_quads,
            &mut text_runs,
            &mut hit_regions,
        );
        render_scene(
            state,
            &layout,
            scene_viewport,
            camera,
            &mut viewport_underlay_quads,
            &mut viewport_overlay_quads,
            &mut text_runs,
            &mut hit_regions,
        );
        let panel_vertices = quads_to_vertices(&panel_quads);
        let viewport_underlay_vertices = quads_to_vertices(&viewport_underlay_quads);
        let viewport_overlay_vertices = quads_to_vertices(&viewport_overlay_quads);

        Self {
            layout,
            hit_regions,
            scene_viewport,
            scene_bounds: state.scene.bounds.clone(),
            camera,
            panel_vertices,
            viewport_underlay_vertices,
            viewport_overlay_vertices,
            text_runs,
        }
    }

    pub fn hit_test(&self, x: f32, y: f32) -> Option<&HitTarget> {
        self.hit_regions
            .iter()
            .rev()
            .find(|region| region.rect.contains(x, y))
            .map(|region| &region.target)
    }

    pub fn world_point_at_screen(&self, x: f32, y: f32) -> Option<PointNm> {
        if !self.scene_viewport.contains(x, y) {
            return None;
        }
        let board_field = inset_rect(self.scene_viewport, 10.0, 10.0, 10.0, 10.0);
        let projection = Projection::new(board_field, &self.scene_bounds, self.camera);
        Some(projection.screen_to_world(x, y))
    }

    fn panel_vertices(&self) -> &[Vertex] {
        &self.panel_vertices
    }

    fn viewport_underlay_vertices(&self) -> &[Vertex] {
        &self.viewport_underlay_vertices
    }

    fn viewport_overlay_vertices(&self) -> &[Vertex] {
        &self.viewport_overlay_vertices
    }
}

impl RetainedScene {
    pub fn from_workspace(state: &ReviewWorkspaceState, width: u32, height: u32) -> Self {
        let layout = ShellLayout::for_window(width, height, dock_height_for_state(state));
        let scene_viewport = layout.scene_viewport();
        let board_field = inset_rect(scene_viewport, 10.0, 10.0, 10.0, 10.0);
        let reference_projection = Projection::new(
            board_field,
            &state.scene.bounds,
            CameraState::fit_to_bounds(&state.scene.bounds),
        );
        let mut world_quads = Vec::new();
        let mut world_hit_regions = Vec::new();
        push_retained_scene_geometry(&mut world_quads, &state.scene, &reference_projection, state);
        push_retained_world_hit_regions(&mut world_hit_regions, &state.scene, state);
        Self {
            world_vertices: quads_to_vertices(&world_quads),
            world_hit_regions,
        }
    }

    fn world_vertices(&self) -> &[Vertex] {
        &self.world_vertices
    }

    pub fn hit_test_authored_world(&self, point: PointNm) -> Option<&HitTarget> {
        self.world_hit_regions
            .iter()
            .rev()
            .find(|region| region.shape.contains(point))
            .map(|region| &region.target)
    }
}

impl WorldHitShape {
    fn contains(&self, point: PointNm) -> bool {
        match self {
            Self::Rect(rect) => point_in_rect(point, *rect),
            Self::Polyline {
                path,
                half_width_nm,
            } => polyline_contains_world_point(path, point, *half_width_nm),
            Self::Circle { center, radius_nm } => {
                let dx = point.x as f32 - center.x as f32;
                let dy = point.y as f32 - center.y as f32;
                dx * dx + dy * dy <= radius_nm * radius_nm
            }
        }
    }
}

fn render_side_panels(
    state: &ReviewWorkspaceState,
    layout: &ShellLayout,
    panel_quads: &mut Vec<Quad>,
    text_runs: &mut Vec<TextRun>,
    hit_regions: &mut Vec<HitRegion>,
) {
    let left = layout.left_sidebar;
    let right = layout.right_sidebar;

    let project_rect = RectPx {
        x: left.x + 14.0,
        y: left.y + 14.0,
        width: left.width - 28.0,
        height: 124.0,
    };
    let filters_rect = RectPx {
        x: left.x + 14.0,
        y: left.y + 150.0,
        width: left.width - 28.0,
        height: (left.height - 164.0).max(100.0),
    };
    let inspector_rect = RectPx {
        x: right.x + 14.0,
        y: right.y + 14.0,
        width: right.width - 28.0,
        height: 150.0,
    };
    let review_rect = RectPx {
        x: right.x + 14.0,
        y: right.y + 176.0,
        width: right.width - 28.0,
        height: right.height - 190.0,
    };

    for (rect, title) in [
        (project_rect, "PROJECT"),
        (filters_rect, "FILTERS"),
        (inspector_rect, "INSPECTOR"),
        (review_rect, "REVIEW"),
    ] {
        panel_quads.push(Quad::from_rect(rect, PANEL_CARD_BG));
        push_rect_border(panel_quads, rect, PANEL_CARD_BORDER, 1.0);
        draw_text(
            title,
            rect.x + 12.0,
            rect.y + 12.0,
            12.0,
            TEXT_SECONDARY,
            TextFace::Ui,
            text_runs,
        );
        push_section_divider(
            panel_quads,
            rect.x + 12.0,
            rect.y + 28.0,
            rect.width - 24.0,
            PANEL_CARD_BORDER,
        );
    }

    draw_text(
        &truncate_text(&state.scene.project_name.to_uppercase(), 22),
        project_rect.x + 12.0,
        project_rect.y + 34.0,
        16.0,
        TEXT_PRIMARY,
        TextFace::Ui,
        text_runs,
    );
    draw_text(
        &format!(
            "BOARD {}",
            truncate_text(&state.scene.board_name.to_uppercase(), 18)
        ),
        project_rect.x + 12.0,
        project_rect.y + 54.0,
        12.5,
        TEXT_SECONDARY,
        TextFace::Ui,
        text_runs,
    );
    if let Some(action) = state.selected_review_action() {
        draw_text(
            &format!("NET {}", truncate_text(&action.net_name.to_uppercase(), 18)),
            project_rect.x + 12.0,
            project_rect.y + 74.0,
            13.0,
            TEXT_ACCENT,
            TextFace::Ui,
            text_runs,
        );
    }
    draw_text(
        &format!("TOOL {}", workspace_tool_label(state.tool)),
        project_rect.x + 12.0,
        project_rect.y + 94.0,
        12.0,
        TEXT_MUTED,
        TextFace::Mono,
        text_runs,
    );
    let fit_board_rect = RectPx {
        x: project_rect.x + 12.0,
        y: project_rect.y + 106.0,
        width: 72.0,
        height: 20.0,
    };
    let fit_review_rect = RectPx {
        x: project_rect.x + 92.0,
        y: project_rect.y + 106.0,
        width: 92.0,
        height: 20.0,
    };
    for (rect, label, target) in [
        (fit_board_rect, "FIT BOARD", HitTarget::FitBoard),
        (fit_review_rect, "FIT REVIEW", HitTarget::FitReviewTarget),
    ] {
        panel_quads.push(Quad::from_rect(rect, REVIEW_ROW_BADGE));
        push_rect_border(panel_quads, rect, PANEL_CARD_BORDER, 1.0);
        draw_text(
            label,
            rect.x + 7.0,
            rect.y + 5.0,
            10.0,
            TEXT_SECONDARY,
            TextFace::Ui,
            text_runs,
        );
        hit_regions.push(HitRegion { target, rect });
    }
    if let Some(status) = &state.last_command_status {
        draw_text(
            &truncate_text(&format!("LAST {}", status.action.to_uppercase()), 24),
            project_rect.x + 12.0,
            project_rect.y + 132.0,
            11.0,
            TEXT_MUTED,
            TextFace::Mono,
            text_runs,
        );
    }
    push_boolean_row(
        filters_rect.x + 12.0,
        filters_rect.y + 36.0,
        "AUTHORED",
        state.ui.filters.show_authored,
        text_runs,
    );
    hit_regions.push(HitRegion {
        target: HitTarget::ToggleShowAuthored,
        rect: RectPx {
            x: filters_rect.x + 8.0,
            y: filters_rect.y + 30.0,
            width: filters_rect.width - 16.0,
            height: 18.0,
        },
    });
    push_boolean_row(
        filters_rect.x + 12.0,
        filters_rect.y + 56.0,
        "PROPOSED",
        state.ui.filters.show_proposed,
        text_runs,
    );
    hit_regions.push(HitRegion {
        target: HitTarget::ToggleShowProposed,
        rect: RectPx {
            x: filters_rect.x + 8.0,
            y: filters_rect.y + 50.0,
            width: filters_rect.width - 16.0,
            height: 18.0,
        },
    });
    push_boolean_row(
        filters_rect.x + 12.0,
        filters_rect.y + 76.0,
        "UNROUTED",
        state.ui.filters.show_unrouted,
        text_runs,
    );
    hit_regions.push(HitRegion {
        target: HitTarget::ToggleShowUnrouted,
        rect: RectPx {
            x: filters_rect.x + 8.0,
            y: filters_rect.y + 70.0,
            width: filters_rect.width - 16.0,
            height: 18.0,
        },
    });
    push_boolean_row(
        filters_rect.x + 12.0,
        filters_rect.y + 96.0,
        "DIM UNRELATED",
        state.ui.filters.dim_unrelated,
        text_runs,
    );
    hit_regions.push(HitRegion {
        target: HitTarget::ToggleDimUnrelated,
        rect: RectPx {
            x: filters_rect.x + 8.0,
            y: filters_rect.y + 90.0,
            width: filters_rect.width - 16.0,
            height: 18.0,
        },
    });
    let mut layer_y = filters_rect.y + 120.0;
    let max_layer_rows = ((filters_rect.height - 140.0) / 20.0).floor().max(1.0) as usize;
    // Show all layers — copper first, then non-copper
    let mut display_layers: Vec<&_> = state.scene.layers.iter().collect();
    display_layers.sort_by_key(|l| {
        (
            !l.visible_by_default,
            scene_layer_stack_priority(&l.layer_id, &state.scene.layers),
            l.render_order,
        )
    });
    for layer in display_layers.iter().take(max_layer_rows) {
        let visible = state
            .ui
            .filters
            .layer_visibility
            .get(&layer.layer_id)
            .copied()
            .unwrap_or(layer.visible_by_default);
        push_boolean_row(
            filters_rect.x + 12.0,
            layer_y,
            &truncate_text(&layer.name.to_uppercase(), 18),
            visible,
            text_runs,
        );
        hit_regions.push(HitRegion {
            target: HitTarget::ToggleLayer(layer.layer_id.clone()),
            rect: RectPx {
                x: filters_rect.x + 8.0,
                y: layer_y - 6.0,
                width: filters_rect.width - 16.0,
                height: 18.0,
            },
        });
        layer_y += 20.0;
    }
    if let Some(action) = state.selected_review_action() {
        draw_text(
            &format!(
                "ACTIVE {}",
                truncate_text(&suffix_id(&action.action_id).to_uppercase(), 14)
            ),
            filters_rect.x + 12.0,
            filters_rect.y + 164.0,
            11.0,
            TEXT_MUTED,
            TextFace::Mono,
            text_runs,
        );
    }
    draw_text(
        &format!("LAYERS {}", state.scene.layers.len()),
        filters_rect.x + 12.0,
        filters_rect.y + 182.0,
        11.0,
        TEXT_MUTED,
        TextFace::Mono,
        text_runs,
    );
    draw_text(
        &format!("FOCUS {}", if has_review_focus(state) { "ON" } else { "OFF" }),
        filters_rect.x + 12.0,
        filters_rect.y + 198.0,
        11.0,
        TEXT_MUTED,
        TextFace::Mono,
        text_runs,
    );

    draw_text(
        "SELECTION",
        inspector_rect.x + 12.0,
        inspector_rect.y + 34.0,
        12.0,
        TEXT_SECONDARY,
        TextFace::Ui,
        text_runs,
    );
    match &state.selection {
        SelectionTarget::ReviewAction(action_id) => {
            draw_text(
                &format!(
                    "ACTION {}",
                    truncate_text(&suffix_id(action_id).to_uppercase(), 14)
                ),
                inspector_rect.x + 12.0,
                inspector_rect.y + 54.0,
                15.0,
                TEXT_PRIMARY,
                TextFace::Mono,
                text_runs,
            );
        }
        SelectionTarget::AuthoredObject(object_id) => {
            let mut y = inspector_rect.y + 54.0;
            if let Some(comp) = state.scene.components.iter().find(|c| &c.object_id == object_id) {
                draw_text(
                    &comp.reference.to_uppercase(),
                    inspector_rect.x + 12.0,
                    y,
                    15.0,
                    TEXT_PRIMARY,
                    TextFace::Mono,
                    text_runs,
                );
                y += 20.0;
                if let Some(value) = &comp.value {
                    push_key_value(inspector_rect.x + 12.0, y, "VALUE", &value.to_uppercase(), text_runs, TextFace::Ui);
                    y += 18.0;
                }
                push_key_value(inspector_rect.x + 12.0, y, "LAYER", &comp.placement_layer.to_uppercase(), text_runs, TextFace::Mono);
                y += 18.0;
                let pos = format!("{:.2}, {:.2} mm", comp.position.x as f64 / 1_000_000.0, comp.position.y as f64 / 1_000_000.0);
                push_key_value(inspector_rect.x + 12.0, y, "POS", &pos, text_runs, TextFace::Mono);
            } else if let Some(pad) = state.scene.pads.iter().find(|p| &p.object_id == object_id) {
                draw_text(
                    &format!("PAD {}", pad.shape_kind.to_uppercase()),
                    inspector_rect.x + 12.0,
                    y,
                    15.0,
                    TEXT_PRIMARY,
                    TextFace::Mono,
                    text_runs,
                );
                y += 20.0;
                push_key_value(inspector_rect.x + 12.0, y, "LAYER", &pad.layer_id.to_uppercase(), text_runs, TextFace::Mono);
                y += 18.0;
                let w = (pad.bounds.max_x - pad.bounds.min_x) as f64 / 1_000_000.0;
                let h = (pad.bounds.max_y - pad.bounds.min_y) as f64 / 1_000_000.0;
                push_key_value(inspector_rect.x + 12.0, y, "SIZE", &format!("{w:.2} x {h:.2} mm"), text_runs, TextFace::Mono);
                y += 18.0;
                if let Some(drill) = pad.drill_nm {
                    push_key_value(inspector_rect.x + 12.0, y, "DRILL", &format!("{:.2} mm", drill as f64 / 1_000_000.0), text_runs, TextFace::Mono);
                }
            } else if let Some(track) = state.scene.tracks.iter().find(|t| &t.object_id == object_id) {
                draw_text(
                    "TRACK",
                    inspector_rect.x + 12.0,
                    y,
                    15.0,
                    TEXT_PRIMARY,
                    TextFace::Mono,
                    text_runs,
                );
                y += 20.0;
                push_key_value(inspector_rect.x + 12.0, y, "LAYER", &track.layer_id.to_uppercase(), text_runs, TextFace::Mono);
                y += 18.0;
                push_key_value(inspector_rect.x + 12.0, y, "WIDTH", &format!("{:.2} mm", track.width_nm as f64 / 1_000_000.0), text_runs, TextFace::Mono);
            } else if let Some(via) = state.scene.vias.iter().find(|v| &v.object_id == object_id) {
                draw_text(
                    "VIA",
                    inspector_rect.x + 12.0,
                    y,
                    15.0,
                    TEXT_PRIMARY,
                    TextFace::Mono,
                    text_runs,
                );
                y += 20.0;
                push_key_value(inspector_rect.x + 12.0, y, "DIA", &format!("{:.2} mm", via.diameter_nm as f64 / 1_000_000.0), text_runs, TextFace::Mono);
                y += 18.0;
                push_key_value(inspector_rect.x + 12.0, y, "DRILL", &format!("{:.2} mm", via.drill_nm as f64 / 1_000_000.0), text_runs, TextFace::Mono);
                y += 18.0;
                push_key_value(inspector_rect.x + 12.0, y, "LAYERS", &format!("{} → {}", via.start_layer_id.to_uppercase(), via.end_layer_id.to_uppercase()), text_runs, TextFace::Mono);
            } else {
                draw_text(
                    &format!("OBJECT {}", truncate_text(&suffix_id(object_id).to_uppercase(), 14)),
                    inspector_rect.x + 12.0,
                    y,
                    15.0,
                    TEXT_PRIMARY,
                    TextFace::Mono,
                    text_runs,
                );
            }
            let _ = y;
        }
        SelectionTarget::None => draw_text(
            "NONE",
            inspector_rect.x + 12.0,
            inspector_rect.y + 54.0,
            15.0,
            TEXT_MUTED,
            TextFace::Ui,
            text_runs,
        ),
    }
    if let Some(action) = state.selected_review_action() {
        push_section_divider(
            panel_quads,
            inspector_rect.x + 12.0,
            inspector_rect.y + 76.0,
            inspector_rect.width - 24.0,
            [0.23, 0.25, 0.29],
        );
        push_key_value(
            inspector_rect.x + 12.0,
            inspector_rect.y + 84.0,
            "CONTRACT",
            &truncate_text(&action.contract.to_uppercase(), 18),
            text_runs,
            TextFace::Mono,
        );
        push_key_value(
            inspector_rect.x + 12.0,
            inspector_rect.y + 104.0,
            "NET",
            &truncate_text(&action.net_name.to_uppercase(), 16),
            text_runs,
            TextFace::Ui,
        );
        push_key_value(
            inspector_rect.x + 12.0,
            inspector_rect.y + 124.0,
            "SEGMENT",
            &format!(
                "{} OF {}",
                action.selected_path_segment_index + 1,
                action.selected_path_segment_count
            ),
            text_runs,
            TextFace::Mono,
        );
    }
    if let Some(segment) = state.selected_segment_evidence() {
        push_key_value(
            inspector_rect.x + 12.0,
            inspector_rect.y + 144.0,
            "LAYER",
            &segment.layer.to_string(),
            text_runs,
            TextFace::Mono,
        );
    }
    if let Some(status) = &state.last_command_status {
        push_key_value(
            inspector_rect.x + 12.0,
            inspector_rect.y + 164.0,
            "LAST",
            &truncate_text(&status.detail.to_uppercase(), 20),
            text_runs,
            TextFace::Mono,
        );
    }

    draw_text(
        &format!(
            "SOURCE {}",
            truncate_text(&state.review.review_source.to_uppercase(), 20)
        ),
        review_rect.x + 12.0,
        review_rect.y + 34.0,
        12.0,
        TEXT_SECONDARY,
        TextFace::Mono,
        text_runs,
    );
    let prev_rect = RectPx {
        x: review_rect.x + review_rect.width - 98.0,
        y: review_rect.y + 30.0,
        width: 36.0,
        height: 20.0,
    };
    let next_rect = RectPx {
        x: review_rect.x + review_rect.width - 54.0,
        y: review_rect.y + 30.0,
        width: 36.0,
        height: 20.0,
    };
    for (rect, label, target) in [
        (prev_rect, "PREV", HitTarget::ReviewPrev),
        (next_rect, "NEXT", HitTarget::ReviewNext),
    ] {
        panel_quads.push(Quad::from_rect(rect, REVIEW_ROW_BADGE));
        push_rect_border(panel_quads, rect, PANEL_CARD_BORDER, 1.0);
        draw_text(
            label,
            rect.x + 7.0,
            rect.y + 5.0,
            10.5,
            TEXT_SECONDARY,
            TextFace::Ui,
            text_runs,
        );
        hit_regions.push(HitRegion { target, rect });
    }
    draw_text(
        &format!("{} ACTIONS", state.review.proposal_actions.len()),
        review_rect.x + 12.0,
        review_rect.y + 54.0,
        15.0,
        TEXT_PRIMARY,
        TextFace::Ui,
        text_runs,
    );
    push_section_divider(
        panel_quads,
        review_rect.x + 12.0,
        review_rect.y + 72.0,
        review_rect.width - 24.0,
        [0.23, 0.25, 0.29],
    );

    let rows: Vec<ReviewActionRow> = state.review_rows();
    let mut row_y = review_rect.y + 82.0;
    for (index, row) in rows.into_iter().enumerate() {
        let selected = row.action_id == state.active_review_target_id;
        let row_rect = RectPx {
            x: review_rect.x + 8.0,
            y: row_y,
            width: review_rect.width - 16.0,
            height: 52.0,
        };
        let badge_rect = RectPx {
            x: row_rect.x + 10.0,
            y: row_rect.y + 10.0,
            width: 30.0,
            height: 30.0,
        };
        let accent_rect = RectPx {
            x: row_rect.x,
            y: row_rect.y,
            width: 4.0,
            height: row_rect.height,
        };
        panel_quads.push(Quad::from_rect(
            row_rect,
            if selected {
                REVIEW_ROW_ACTIVE_BG
            } else {
                REVIEW_ROW_BG
            },
        ));
        panel_quads.push(Quad::from_rect(
            accent_rect,
            if selected {
                PROPOSAL_BASE
            } else {
                PANEL_CARD_BORDER
            },
        ));
        panel_quads.push(Quad::from_rect(
            badge_rect,
            if selected {
                REVIEW_ROW_BADGE_ACTIVE
            } else {
                REVIEW_ROW_BADGE
            },
        ));
        push_rect_border(
            panel_quads,
            row_rect,
            if selected {
                PROPOSAL_BASE
            } else {
                PANEL_CARD_BORDER
            },
            1.0,
        );
        draw_text(
            &(index + 1).to_string(),
            badge_rect.x + 11.0,
            badge_rect.y + 7.0,
            14.0,
            if selected {
                TEXT_PRIMARY
            } else {
                TEXT_SECONDARY
            },
            TextFace::Mono,
            text_runs,
        );
        draw_text(
            &truncate_text(&row.title, 22),
            row_rect.x + 52.0,
            row_rect.y + 10.0,
            14.0,
            if selected {
                TEXT_ACCENT
            } else {
                TEXT_PANEL_VALUE
            },
            TextFace::Ui,
            text_runs,
        );
        draw_text(
            &truncate_text(&row.subtitle, 28),
            row_rect.x + 52.0,
            row_rect.y + 28.0,
            11.0,
            TEXT_SECONDARY,
            TextFace::Mono,
            text_runs,
        );
        if selected {
            draw_text(
                "ACTIVE",
                row_rect.x + row_rect.width - 48.0,
                row_rect.y + 11.0,
                10.5,
                TEXT_ACCENT,
                TextFace::Ui,
                text_runs,
            );
        }
        hit_regions.push(HitRegion {
            target: HitTarget::ReviewAction(row.action_id),
            rect: row_rect,
        });
        row_y += 54.0;
    }
}

fn render_bottom_tabs(
    state: &ReviewWorkspaceState,
    layout: &ShellLayout,
    panel_quads: &mut Vec<Quad>,
    text_runs: &mut Vec<TextRun>,
    hit_regions: &mut Vec<HitRegion>,
) {
    let terminal_rect = RectPx {
        x: layout.bottom_strip.x + 12.0,
        y: layout.bottom_strip.y + 8.0,
        width: 120.0,
        height: layout.bottom_strip.height - 16.0,
    };
    let assistant_rect = RectPx {
        x: layout.bottom_strip.x + 140.0,
        y: layout.bottom_strip.y + 8.0,
        width: 120.0,
        height: layout.bottom_strip.height - 16.0,
    };
    for (rect, label, target, active) in [
        (
            terminal_rect,
            "TERMINAL",
            HitTarget::TerminalTab,
            matches!(state.ui.active_dock_tab, Some(DockTab::Terminal)),
        ),
        (
            assistant_rect,
            "ASSISTANT",
            HitTarget::AssistantTab,
            matches!(state.ui.active_dock_tab, Some(DockTab::Assistant)),
        ),
    ] {
        panel_quads.push(Quad::from_rect(
            rect,
            if active {
                [0.20, 0.22, 0.27]
            } else {
                [0.15, 0.16, 0.19]
            },
        ));
        push_rect_border(panel_quads, rect, PANEL_CARD_BORDER, 1.0);
        draw_text(
            label,
            rect.x + 12.0,
            rect.y + 10.0,
            12.0,
            if active { TEXT_PRIMARY } else { TEXT_SECONDARY },
            TextFace::Ui,
            text_runs,
        );
        hit_regions.push(HitRegion { target, rect });
    }

    let Some(active_tab) = state.ui.active_dock_tab else {
        return;
    };
    // Drag handle at top of expanded dock
    let handle_rect = RectPx {
        x: layout.bottom_strip.x,
        y: layout.bottom_strip.y,
        width: layout.bottom_strip.width,
        height: 6.0,
    };
    panel_quads.push(Quad::from_rect(handle_rect, [0.24, 0.26, 0.30]));
    hit_regions.push(HitRegion {
        target: HitTarget::DockResizeHandle,
        rect: handle_rect,
    });
    let content_rect = RectPx {
        x: layout.bottom_strip.x + 12.0,
        y: layout.bottom_strip.y + 44.0,
        width: layout.bottom_strip.width - 24.0,
        height: (layout.bottom_strip.height - 56.0).max(0.0),
    };
    panel_quads.push(Quad::from_rect(content_rect, [0.11, 0.12, 0.15]));
    push_rect_border(panel_quads, content_rect, PANEL_CARD_BORDER, 1.0);
    match active_tab {
        DockTab::Terminal => render_terminal_lane(state, content_rect, text_runs),
        DockTab::Assistant => render_assistant_lane(state, content_rect, text_runs),
    }
}

fn render_terminal_lane(state: &ReviewWorkspaceState, rect: RectPx, text_runs: &mut Vec<TextRun>) {
    draw_text(
        "DETERMINISTIC LOG",
        rect.x + 12.0,
        rect.y + 12.0,
        12.0,
        TEXT_SECONDARY,
        TextFace::Ui,
        text_runs,
    );
    draw_text(
        "READ-ONLY WORKFLOW / STATUS OUTPUT",
        rect.x + 12.0,
        rect.y + 28.0,
        10.5,
        TEXT_MUTED,
        TextFace::Mono,
        text_runs,
    );
    let max_lines = ((rect.height - 56.0) / 16.0).floor().max(1.0) as usize;
    let total = state.ui.terminal.lines.len();
    let scroll = state.ui.terminal.scroll_offset.min(total.saturating_sub(max_lines));
    let tail_start = total.saturating_sub(max_lines + scroll);
    let mut y = rect.y + 46.0;
    for line in state.ui.terminal.lines.iter().skip(tail_start).take(max_lines) {
        draw_text(
            &truncate_text(line, 180),
            rect.x + 12.0,
            y,
            11.0,
            TEXT_PANEL_VALUE,
            TextFace::Mono,
            text_runs,
        );
        y += 16.0;
    }
}

fn render_assistant_lane(state: &ReviewWorkspaceState, rect: RectPx, text_runs: &mut Vec<TextRun>) {
    draw_text(
        "SESSION ASSISTANT",
        rect.x + 12.0,
        rect.y + 12.0,
        12.0,
        TEXT_SECONDARY,
        TextFace::Ui,
        text_runs,
    );
    draw_text(
        &format!(
            "CONTEXT  TOOL {}  SELECTION {}",
            workspace_tool_label(state.tool),
            selection_summary(state)
        ),
        rect.x + 12.0,
        rect.y + 28.0,
        10.5,
        TEXT_MUTED,
        TextFace::Mono,
        text_runs,
    );
    let max_lines = ((rect.height - 58.0) / 18.0).floor().max(1.0) as usize;
    let total = state.ui.assistant.transcript.len();
    let scroll = state.ui.assistant.scroll_offset.min(total.saturating_sub(max_lines));
    let tail_start = total.saturating_sub(max_lines + scroll);
    let mut y = rect.y + 48.0;
    for msg in state.ui.assistant.transcript.iter().skip(tail_start).take(max_lines) {
        draw_text(
            &truncate_text(
                &format!("{}: {}", msg.role.to_uppercase(), msg.content),
                160,
            ),
            rect.x + 12.0,
            y,
            11.0,
            if msg.role == "assistant" {
                TEXT_PANEL_VALUE
            } else {
                TEXT_PRIMARY
            },
            if msg.role == "assistant" {
                TextFace::Ui
            } else {
                TextFace::Mono
            },
            text_runs,
        );
        y += 18.0;
    }
    let display_input = if state.ui.assistant.awaiting_api_key {
        if state.ui.assistant.input.is_empty() {
            "[enter api key]".to_string()
        } else {
            "[hidden]".to_string()
        }
    } else {
        let (before, after) = split_at_cursor(
            &state.ui.assistant.input,
            state.ui.assistant.cursor,
        );
        format!("{before}|{after}")
    };
    draw_text(
        &format!("> {display_input}"),
        rect.x + 12.0,
        rect.y + rect.height - 18.0,
        11.5,
        TEXT_PRIMARY,
        TextFace::Mono,
        text_runs,
    );
}

fn split_at_cursor(input: &str, cursor: usize) -> (&str, &str) {
    let byte_pos = input
        .char_indices()
        .nth(cursor)
        .map(|(i, _)| i)
        .unwrap_or(input.len());
    (&input[..byte_pos], &input[byte_pos..])
}

fn dock_height_for_state(state: &ReviewWorkspaceState) -> Option<u32> {
    if state.ui.active_dock_tab.is_some() {
        Some(state.ui.dock_height_px)
    } else {
        None
    }
}

fn selection_summary(state: &ReviewWorkspaceState) -> String {
    match &state.selection {
        SelectionTarget::ReviewAction(action_id) => truncate_text(
            &format!("REVIEW {}", suffix_id(action_id).to_uppercase()),
            28,
        ),
        SelectionTarget::AuthoredObject(object_id) => truncate_text(
            &format!("OBJECT {}", suffix_id(object_id).to_uppercase()),
            28,
        ),
        SelectionTarget::None => "NONE".to_string(),
    }
}

fn render_scene(
    state: &ReviewWorkspaceState,
    layout: &ShellLayout,
    scene_viewport: RectPx,
    camera: CameraState,
    viewport_underlay_quads: &mut Vec<Quad>,
    viewport_overlay_quads: &mut Vec<Quad>,
    text_runs: &mut Vec<TextRun>,
    hit_regions: &mut Vec<HitRegion>,
) {
    push_scene_underlay(
        viewport_underlay_quads,
        &state.scene,
        scene_viewport,
        camera,
    );
    push_scene_overlay_and_hits(
        viewport_overlay_quads,
        &state.scene,
        scene_viewport,
        camera,
        state,
        text_runs,
        hit_regions,
    );
    draw_text(
        &truncate_text(&state.scene.board_name.to_uppercase(), 28),
        layout.viewport.x + 16.0,
        layout.viewport.y + 16.0,
        12.0,
        TEXT_SECONDARY,
        TextFace::Ui,
        text_runs,
    );
    if let Some(action) = state.selected_review_action() {
        draw_text(
            &format!(
                "ACTIVE {}",
                truncate_text(&suffix_id(&action.action_id).to_uppercase(), 16)
            ),
            layout.viewport.x + 16.0,
            layout.viewport.y + 32.0,
            13.0,
            TEXT_ACCENT,
            TextFace::Mono,
            text_runs,
        );
        draw_text(
            &format!("NET {}", truncate_text(&action.net_name.to_uppercase(), 20)),
            layout.viewport.x + 16.0,
            layout.viewport.y + 50.0,
            10.5,
            TEXT_MUTED,
            TextFace::Ui,
            text_runs,
        );
    }
    {
        draw_text(
            "F FIT  [ ] REVIEW NAV  CLICK SELECT  SCROLL ZOOM  ESC CLEAR",
            layout.viewport.x + 16.0,
            layout.viewport.y + 66.0,
            10.5,
            TEXT_MUTED,
            TextFace::Mono,
            text_runs,
        );
    }
    // Status bar at bottom of viewport
    let status_y = layout.viewport.y + layout.viewport.height - 20.0;
    let zoom_pct = (camera.zoom * 100.0).round() as i32;
    let tool_label = workspace_tool_label(state.tool);
    let sel_label = match &state.selection {
        SelectionTarget::None => "NONE".to_string(),
        SelectionTarget::ReviewAction(id) => truncate_text(&suffix_id(id).to_uppercase(), 12),
        SelectionTarget::AuthoredObject(id) => truncate_text(&suffix_id(id).to_uppercase(), 12),
    };
    let status_bg = RectPx {
        x: layout.viewport.x,
        y: status_y - 2.0,
        width: layout.viewport.width,
        height: 22.0,
    };
    viewport_overlay_quads.push(Quad::from_rect(status_bg, [0.07, 0.08, 0.10]));
    let status_text = format!(
        "TOOL {}  |  ZOOM {}%  |  SEL {}",
        tool_label, zoom_pct, sel_label
    );
    draw_text(
        &status_text,
        layout.viewport.x + 16.0,
        status_y,
        10.5,
        TEXT_MUTED,
        TextFace::Mono,
        text_runs,
    );
    if let Some(status) = &state.last_command_status {
        let is_error = status.detail.contains("failed") || status.detail.contains("error");
        let color = if is_error {
            [0.85, 0.40, 0.35]
        } else {
            [0.45, 0.72, 0.45]
        };
        draw_text(
            &truncate_text(
                &format!("{}  {}", status.action.to_uppercase(), status.detail),
                40,
            ),
            layout.viewport.x + layout.viewport.width - 340.0,
            status_y,
            10.5,
            color,
            TextFace::Mono,
            text_runs,
        );
    }
}

fn push_scene_underlay(
    out: &mut Vec<Quad>,
    scene: &BoardReviewSceneV1,
    scene_viewport: RectPx,
    camera: CameraState,
) {
    out.push(Quad::from_rect(
        scene_viewport,
        board_surface_color(BoardSurfaceRole::OuterField),
    ));
    push_rect_border(out, scene_viewport, VIEWPORT_FRAME, 1.0);
    let board_field = inset_rect(scene_viewport, 10.0, 10.0, 10.0, 10.0);
    let projection = Projection::new(board_field, &scene.bounds, camera);
    out.push(Quad::from_rect(
        board_field,
        board_surface_color(BoardSurfaceRole::InnerField),
    ));
    push_rect_border(out, board_field, [0.46, 0.49, 0.53], 1.0);
    push_scene_grid(out, &projection);
}

fn authored_visible(state: &ReviewWorkspaceState) -> bool {
    state.ui.filters.show_authored
}

fn proposed_visible(state: &ReviewWorkspaceState) -> bool {
    state.ui.filters.show_proposed
}

fn unrouted_visible(state: &ReviewWorkspaceState) -> bool {
    state.ui.filters.show_unrouted
}

fn layer_visible(state: &ReviewWorkspaceState, layer_id: &str) -> bool {
    state
        .ui
        .filters
        .layer_visibility
        .get(layer_id)
        .copied()
        .unwrap_or(true)
}

fn via_visible(state: &ReviewWorkspaceState, start_layer_id: &str, end_layer_id: &str) -> bool {
    layer_visible(state, start_layer_id) || layer_visible(state, end_layer_id)
}

fn pad_copper_layer_ids<'a>(pad: &'a datum_gui_protocol::PadPrimitive) -> Vec<&'a str> {
    if pad.copper_layer_ids.is_empty() {
        vec![pad.layer_id.as_str()]
    } else {
        pad.copper_layer_ids.iter().map(String::as_str).collect()
    }
}

fn pad_visible_on_any_copper_layer(
    state: &ReviewWorkspaceState,
    pad: &datum_gui_protocol::PadPrimitive,
) -> bool {
    pad_copper_layer_ids(pad)
        .into_iter()
        .any(|layer_id| layer_visible(state, layer_id))
}

fn dim_unrelated_active(state: &ReviewWorkspaceState) -> bool {
    if !state.ui.filters.dim_unrelated {
        return false;
    }
    has_review_focus(state) || !matches!(state.selection, SelectionTarget::None)
}

fn is_hovered(state: &ReviewWorkspaceState, object_id: &str) -> bool {
    if !matches!(state.selection, SelectionTarget::None) {
        return false;
    }
    state
        .ui
        .hovered_object_id
        .as_deref()
        .is_some_and(|id| id == object_id)
}

fn unrouted_matches_active_action(
    unrouted: &UnroutedPrimitive,
    state: &ReviewWorkspaceState,
) -> bool {
    state
        .selected_review_action()
        .is_some_and(|action| action.net_uuid == unrouted.net_uuid)
}

fn unrouted_base_color(scene: &BoardReviewSceneV1, unrouted: &UnroutedPrimitive) -> [f32; 3] {
    scene.net_display
        .iter()
        .find(|entry| entry.net_uuid == unrouted.net_uuid)
        .map(|entry| entry.airwire_color_rgb)
        .unwrap_or(UNROUTED_BASE)
}

fn selected_component_uuid<'a>(
    scene: &'a BoardReviewSceneV1,
    state: &ReviewWorkspaceState,
) -> Option<&'a str> {
    let SelectionTarget::AuthoredObject(object_id) = &state.selection else {
        return None;
    };
    scene.components.iter().find_map(|component| {
        ((&component.object_id == object_id)
            || (format!("component:{}", component.component_uuid) == *object_id))
            .then_some(component.component_uuid.as_str())
    })
}

fn component_is_selection_related(
    component_uuid: &str,
    scene: &BoardReviewSceneV1,
    state: &ReviewWorkspaceState,
) -> bool {
    selected_component_uuid(scene, state).is_some_and(|selected| selected == component_uuid)
}

fn component_is_selection_active(
    component_uuid: &str,
    scene: &BoardReviewSceneV1,
    state: &ReviewWorkspaceState,
) -> bool {
    component_is_selection_related(component_uuid, scene, state)
}

fn component_object_id_for_uuid<'a>(
    scene: &'a BoardReviewSceneV1,
    component_uuid: &str,
) -> Option<&'a str> {
    scene.components.iter().find_map(|component| {
        (component.component_uuid == component_uuid).then_some(component.object_id.as_str())
    })
}


fn push_retained_scene_geometry(
    out: &mut Vec<Quad>,
    scene: &BoardReviewSceneV1,
    reference_projection: &Projection,
    state: &ReviewWorkspaceState,
) {
    let active_move_component_uuid: Option<String> = None;
    let sl = &scene.layers;
    let layer_app = |id: &str| resolve_layer_appearance_with_scene(Some(id), sl);
    // Render copper in physical stack order first; later stages (paste/mask/silk/mechanical/edge)
    // are handled by explicit render-stage grouping below.
    for pass_priority in [0u32, 1, 2] {
        for zone in &scene.zones {
            if copper_pass_priority_for_layer(&zone.layer_id, sl) != Some(pass_priority) {
                continue;
            }
            if !authored_visible(state) || !layer_visible(state, &zone.layer_id) {
                continue;
            }
            let related = zone_matches_active_action(zone, state);
            let dimmed = dim_unrelated_active(state) && !related;
            if zone.polygon.len() >= 4 {
                let za = layer_app(&zone.layer_id);
                let (fill_color, outline_color) = (za.zone_fill, za.zone_outline);
                push_world_polygon_fill(out, &zone.polygon, dim_authored_color(fill_color, dimmed));
                push_world_polyline_mitered(
                    out,
                    &close_path(&zone.polygon),
                    world_stroke_nm(2.0, reference_projection),
                    dim_authored_color(outline_color, dimmed),
                );
            }
        }
        for track in &scene.tracks {
            if copper_pass_priority_for_layer(&track.layer_id, sl) != Some(pass_priority) {
                continue;
            }
            if !authored_visible(state) || !layer_visible(state, &track.layer_id) {
                continue;
            }
            let related = track_matches_active_action(track, state);
            let selected = matches!(state.selection, SelectionTarget::AuthoredObject(ref id) if id == &track.object_id);
            let color = if selected {
                selected_copper_color(layer_app(&track.layer_id).authored_track)
            } else if related {
                AUTHOR_RELATED
            } else {
                dim_authored_color(
                    layer_app(&track.layer_id).authored_track,
                    dim_unrelated_active(state) && !selected && !related,
                )
            };
            let track_width_nm = (track.width_nm as f32).max(
                world_stroke_nm(if selected { 3.0 } else { 2.0 }, reference_projection),
            );
            push_world_polyline_segments(out, &track.path, track_width_nm, color);
            let half = (track_width_nm * 0.5).round() as i64;
            for point in &track.path {
                push_world_ellipse_nm(
                    out,
                    datum_gui_protocol::RectNm {
                        min_x: point.x - half,
                        min_y: point.y - half,
                        max_x: point.x + half,
                        max_y: point.y + half,
                    },
                    color,
                    64,
                );
            }
        }
        for pad in &scene.pads {
            if !authored_visible(state) {
                continue;
            }
            if active_move_component_uuid.as_deref() == Some(pad.component_uuid.as_str()) {
                continue;
            }
            let active = matches!(state.selection, SelectionTarget::AuthoredObject(ref id) if id == &pad.object_id)
                || component_is_selection_active(&pad.component_uuid, scene, state);
            let related = pad_matches_active_action(pad, state)
                || component_is_selection_related(&pad.component_uuid, scene, state);
            let hovered = is_hovered(state, &pad.object_id);
            let dimmed = dim_unrelated_active(state) && !active && !related && !hovered;
            for render_layer in pad_copper_layer_ids(pad) {
                if copper_pass_priority_for_layer(render_layer, sl) != Some(pass_priority) {
                    continue;
                }
                if !layer_visible(state, render_layer) {
                    continue;
                }
                push_pad_primitive_world(
                    out,
                    pad,
                    render_layer,
                    if active {
                        selected_copper_color(layer_app(render_layer).pad_copper)
                    } else if hovered {
                        layer_app(render_layer).pad_related
                    } else if related {
                        layer_app(render_layer).pad_related
                    } else {
                        dim_authored_color(layer_app(render_layer).pad_copper, dimmed)
                    },
                    pad.drill_nm,
                    dimmed,
                    reference_projection,
                );
            }
        }
        for via in &scene.vias {
            if !authored_visible(state)
                || !via_visible(state, &via.start_layer_id, &via.end_layer_id)
            {
                continue;
            }
            let display_layer = if layer_visible(state, &via.start_layer_id) {
                via.start_layer_id.as_str()
            } else if layer_visible(state, &via.end_layer_id) {
                via.end_layer_id.as_str()
            } else {
                continue;
            };
            if copper_pass_priority_for_layer(display_layer, sl) != Some(pass_priority) {
                continue;
            }
            let selected = matches!(
                state.selection,
                SelectionTarget::AuthoredObject(ref id) if id == &via.object_id
            );
            let related = via_matches_active_action(via, state);
            let dimmed = dim_unrelated_active(state) && !selected && !related;
            push_via_primitive_world(
                out,
                via,
                layer_app(display_layer).pad_copper,
                selected,
                dimmed,
                reference_projection,
            );
        }
    }
    let mechanical_graphics: Vec<_> = scene.component_graphics.iter().filter(|graphic| {
        graphic.render_role == "component_mechanical"
            && active_move_component_uuid.as_deref() != Some(graphic.component_uuid.as_str())
    }).collect();
    let mut process_layers: Vec<_> = scene
        .layers
        .iter()
        .filter_map(|layer| match render_stage_for_layer(&layer.layer_id, sl) {
            RenderStage::BottomPaste | RenderStage::TopPaste => {
                Some((layer.layer_id.clone(), PadProcessLayerKind::Paste))
            }
            RenderStage::BottomMask | RenderStage::TopMask => {
                Some((layer.layer_id.clone(), PadProcessLayerKind::Mask))
            }
            _ => None,
        })
        .collect();
    process_layers.sort_by_key(|(layer_id, _)| scene_layer_stack_priority(layer_id, sl));
    let silkscreen_graphics: Vec<_> = scene.component_graphics.iter().filter(|graphic| {
        graphic.render_role == "component_silkscreen"
            && active_move_component_uuid.as_deref() != Some(graphic.component_uuid.as_str())
    }).collect();
    let post_copper_stages = [
        RenderStage::BottomMask,
        RenderStage::TopMask,
        RenderStage::BottomPaste,
        RenderStage::TopPaste,
        RenderStage::BottomSilk,
        RenderStage::TopSilk,
        RenderStage::Mechanical,
        RenderStage::Edge,
    ];
    for stage in post_copper_stages {
        for (layer_id, kind) in process_layers
            .iter()
            .filter(|(layer_id, _)| render_stage_for_layer(layer_id, sl) == stage)
        {
            if !authored_visible(state) || !layer_visible(state, layer_id) {
                continue;
            }
            for pad in &scene.pads {
                let active = matches!(state.selection, SelectionTarget::AuthoredObject(ref id) if id == &pad.object_id)
                    || component_is_selection_active(&pad.component_uuid, scene, state);
                let related = pad_matches_active_action(pad, state);
                let hovered = is_hovered(state, &pad.object_id);
                let dimmed = dim_unrelated_active(state) && !active && !related && !hovered;
                let membership = match kind {
                    PadProcessLayerKind::Mask => &pad.mask_layer_ids,
                    PadProcessLayerKind::Paste => &pad.paste_layer_ids,
                };
                if !membership.iter().any(|member| member == layer_id) {
                    continue;
                }
                let derived =
                    derived_process_pad(pad, layer_id, *kind, &scene.pad_expansion_setup);
                push_pad_primitive_world(
                    out,
                    &derived,
                    layer_id,
                    if active {
                        selected_silk_color(mask_or_paste_layer_color(layer_id, sl))
                    } else {
                        dim_process_color(mask_or_paste_layer_color(layer_id, sl), dimmed)
                    },
                    None,
                    false,
                    reference_projection,
                );
            }
        }
        for graphic in mechanical_graphics.iter().filter(|graphic| {
            graphic_render_stage(graphic.layer_id.as_deref(), sl, RenderStage::Mechanical) == stage
        }) {
            if !authored_visible(state) {
                continue;
            }
            if let Some(lid) = graphic.layer_id.as_deref() && !layer_visible(state, lid) {
                continue;
            }
            let selected_body_graphic_id =
                selected_component_body_graphic_id(scene, &graphic.component_uuid);
            if selected_body_graphic_id.is_some_and(|id| id == graphic.graphic_id) {
                continue;
            }
            let related = component_graphic_matches_active_action(graphic, scene, state)
                || component_is_selection_related(&graphic.component_uuid, scene, state);
            let selected_component = matches!(
                state.selection,
                SelectionTarget::AuthoredObject(ref id)
                    if id == &format!("component:{}", graphic.component_uuid)
            ) || component_is_selection_active(&graphic.component_uuid, scene, state);
            let selected = false;
            push_component_graphic_primitive_world(
                out,
                graphic,
                sl,
                selected,
                related || selected_component,
                dim_unrelated_active(state) && !selected_component && !related,
                reference_projection,
            );
        }
        for graphic in silkscreen_graphics.iter().filter(|graphic| {
            graphic_render_stage(graphic.layer_id.as_deref(), sl, RenderStage::TopSilk) == stage
        }) {
            if !authored_visible(state) {
                continue;
            }
            if let Some(lid) = graphic.layer_id.as_deref() && !layer_visible(state, lid) {
                continue;
            }
            let related = component_graphic_matches_active_action(graphic, scene, state)
                || component_is_selection_related(&graphic.component_uuid, scene, state);
            let selected = matches!(
                state.selection,
                SelectionTarget::AuthoredObject(ref id)
                    if id == &format!("component:{}", graphic.component_uuid)
            ) || component_is_selection_active(&graphic.component_uuid, scene, state);
            push_component_graphic_primitive_world(
                out,
                graphic,
                sl,
                selected,
                related,
                dim_unrelated_active(state) && !selected && !related,
                reference_projection,
            );
        }
        if authored_visible(state) {
            for gfx in &scene.board_graphics {
                if render_stage_for_layer(&gfx.layer_id, sl) != stage {
                    continue;
                }
                if !layer_visible(state, &gfx.layer_id) {
                    continue;
                }
                push_board_graphic_primitive_world(
                    out,
                    gfx,
                    sl,
                    dim_unrelated_active(state),
                    reference_projection,
                );
            }
        }
    }
    if let Some(active_component_uuid) = active_move_component_uuid.as_deref()
        && let Some(component) = scene
            .components
            .iter()
            .find(|component| component.component_uuid == active_component_uuid)
    {
        let selected = true;
        let related = component_overlaps_active_action(component, state)
            || component_is_selection_related(&component.component_uuid, scene, state);
        let dimmed = false;
        let selected_body_graphic_id =
            selected_component_body_graphic_id(scene, &component.component_uuid);
        for graphic in scene
            .component_graphics
            .iter()
            .filter(|graphic| graphic.component_uuid == component.component_uuid)
            .filter(|graphic| graphic.render_role == "component_mechanical")
        {
            if selected_body_graphic_id.is_some_and(|id| id == graphic.graphic_id) {
                continue;
            }
            push_component_graphic_primitive_world(
                out,
                graphic,
                sl,
                false,
                related,
                dimmed,
                reference_projection,
            );
        }
        for pad in scene
            .pads
            .iter()
            .filter(|pad| pad.component_uuid == component.component_uuid)
        {
            for render_layer in pad_copper_layer_ids(pad) {
                if !layer_visible(state, render_layer) {
                    continue;
                }
                push_pad_primitive_world(
                    out,
                    pad,
                    render_layer,
                    selected_copper_color(layer_app(render_layer).pad_copper),
                    pad.drill_nm,
                    dimmed,
                    reference_projection,
                );
            }
        }
        for graphic in scene
            .component_graphics
            .iter()
            .filter(|graphic| graphic.component_uuid == component.component_uuid)
            .filter(|graphic| graphic.render_role == "component_silkscreen")
        {
            push_component_graphic_primitive_world(
                out,
                graphic,
                sl,
                selected,
                related,
                dimmed,
                reference_projection,
            );
        }
    }
    if unrouted_visible(state) {
        let mut unrouted_batches: Vec<(
            Vec<PointNm>,
            [f32; 3],
            [f32; 3],
            f32,
            f32,
            f32,
            f32,
        )> = Vec::new();
        for unrouted in &scene.unrouted_primitives {
            let related = unrouted_matches_active_action(unrouted, state);
            let dimmed = dim_unrelated_active(state) && !related;
            let net_color = unrouted_base_color(scene, unrouted);
            let base_color = if related {
                mix_color(net_color, UNROUTED_FOCUS, 0.35)
            } else {
                dim_context_color(net_color, dimmed)
            };
            let color = mix_color(base_color, BOARD_INNER_FIELD, 0.18);
            let under_color = mix_color(BOARD_OUTER_FIELD, color, if related { 0.28 } else { 0.22 });
            let width_px = if related { 1.55 } else { 1.2 };
            let width_nm = world_stroke_nm(width_px, reference_projection).max(1.0);
            let under_width_nm =
                world_stroke_nm(width_px + if related { 0.9 } else { 0.7 }, reference_projection)
                    .max(width_nm + 1.0);
            let endpoint_radius_nm =
                world_stroke_nm(if related { 1.15 } else { 0.95 }, reference_projection).max(1.0);
            let endpoint_under_radius_nm =
                (endpoint_radius_nm + ((under_width_nm - width_nm) * 0.5)).max(endpoint_radius_nm + 0.5);
            unrouted_batches.push((
                unrouted.path.clone(),
                color,
                under_color,
                width_nm,
                under_width_nm,
                endpoint_radius_nm,
                endpoint_under_radius_nm,
            ));
        }
        for (path, _color, under_color, _width_nm, under_width_nm, _endpoint_radius_nm, _endpoint_under_radius_nm) in
            &unrouted_batches
        {
            push_world_polyline_segments_capped(out, path, *under_width_nm, *under_color);
        }
        for (path, _color, under_color, _width_nm, _under_width_nm, _endpoint_radius_nm, endpoint_under_radius_nm) in
            &unrouted_batches
        {
            for point in path.first().into_iter().chain(path.last()) {
                let under_r = endpoint_under_radius_nm.round() as i64;
                push_world_ellipse_nm(
                    out,
                    datum_gui_protocol::RectNm {
                        min_x: point.x - under_r,
                        min_y: point.y - under_r,
                        max_x: point.x + under_r,
                        max_y: point.y + under_r,
                    },
                    *under_color,
                    24,
                );
            }
        }
        for (path, color, _under_color, width_nm, _under_width_nm, _endpoint_radius_nm, _endpoint_under_radius_nm) in
            &unrouted_batches
        {
            push_world_polyline_segments_capped(out, path, *width_nm, *color);
        }
        for (path, color, _under_color, _width_nm, _under_width_nm, endpoint_radius_nm, _endpoint_under_radius_nm) in
            &unrouted_batches
        {
            for point in path.first().into_iter().chain(path.last()) {
                let r = endpoint_radius_nm.round() as i64;
                push_world_ellipse_nm(
                    out,
                    datum_gui_protocol::RectNm {
                        min_x: point.x - r,
                        min_y: point.y - r,
                        max_x: point.x + r,
                        max_y: point.y + r,
                    },
                    *color,
                    24,
                );
            }
        }
    }
    // Re-render board outline on top of all geometry so it's not covered by
    // zones. Outline participates in the authored/layer visibility model: it
    // must respect the AUTHORED toggle and the Edge.Cuts (or carrying) layer
    // toggle so users can hide it intentionally during review. This is the
    // board-boundary view; per-contributor Edge.Cuts graphics are drawn above
    // via scene.board_graphics.
    if authored_visible(state) {
        for outline in &scene.outline {
            if !layer_visible(state, &outline.layer_id) {
                continue;
            }
            push_world_polyline_segments_capped(
                out,
                &outline.path,
                world_stroke_nm(1.6, reference_projection),
                board_surface_color(BoardSurfaceRole::Edge),
            );
        }
    }
}

fn push_retained_world_hit_regions(
    out: &mut Vec<WorldHitRegion>,
    scene: &BoardReviewSceneV1,
    state: &ReviewWorkspaceState,
) {
    if !authored_visible(state) {
        return;
    }
    for track in &scene.tracks {
        if !layer_visible(state, &track.layer_id) {
            continue;
        }
        out.push(WorldHitRegion {
            target: HitTarget::AuthoredObject(track.object_id.clone()),
            shape: WorldHitShape::Polyline {
                path: track.path.clone(),
                half_width_nm: (track.width_nm as f32 * 0.5).max(150_000.0),
            },
        });
    }
    for via in &scene.vias {
        if !via_visible(state, &via.start_layer_id, &via.end_layer_id) {
            continue;
        }
        out.push(WorldHitRegion {
            target: HitTarget::AuthoredObject(via.object_id.clone()),
            shape: WorldHitShape::Circle {
                center: via.position,
                radius_nm: (via.diameter_nm as f32 * 0.5).max(250_000.0),
            },
        });
    }
    for component in &scene.components {
        if !layer_visible(state, &component.placement_layer) {
            continue;
        }
        let component_pads: Vec<_> = scene
            .pads
            .iter()
            .filter(|pad| pad.component_uuid == component.component_uuid)
            .collect();
        let has_non_edge_graphics = scene.component_graphics.iter().any(|graphic| {
            graphic.component_uuid == component.component_uuid
                && !graphic.layer_id.as_deref().is_some_and(|layer_id| {
                    scene
                        .layers
                        .iter()
                        .find(|layer| layer.layer_id == layer_id)
                        .is_some_and(|layer| layer.name == "Edge.Cuts")
                })
        });
        let has_text = scene
            .component_texts
            .iter()
            .any(|text| text.component_uuid == component.component_uuid);
        if let Some(hit_rect) = compact_component_body_bounds(&component_pads)
            && !has_non_edge_graphics
            && !has_text
        {
            out.push(WorldHitRegion {
                target: HitTarget::AuthoredObject(component.object_id.clone()),
                shape: WorldHitShape::Rect(hit_rect),
            });
            continue;
        }
        if has_non_edge_graphics || has_text {
            continue;
        }
        let hit_rect = inferred_component_body_bounds(&component_pads).unwrap_or(component.bounds);
        out.push(WorldHitRegion {
            target: HitTarget::AuthoredObject(component.object_id.clone()),
            shape: WorldHitShape::Rect(hit_rect),
        });
    }
    for pad in &scene.pads {
        let pad_visible = pad_visible_on_any_copper_layer(state, pad);
        if !pad_visible {
            continue;
        }
        let target_id = component_object_id_for_uuid(scene, &pad.component_uuid)
            .unwrap_or(pad.object_id.as_str());
        out.push(WorldHitRegion {
            target: HitTarget::AuthoredObject(target_id.to_string()),
            shape: WorldHitShape::Rect(pad.bounds),
        });
    }
    for graphic in &scene.component_graphics {
        let Some(target_id) = component_object_id_for_uuid(scene, &graphic.component_uuid) else {
            continue;
        };
        if let Some(layer_id) = graphic.layer_id.as_deref() && !layer_visible(state, layer_id) {
            continue;
        }
        if graphic
            .layer_id
            .as_deref()
            .is_some_and(|layer_id| {
                scene
                    .layers
                    .iter()
                    .find(|layer| layer.layer_id == layer_id)
                    .is_some_and(|layer| layer.name == "Edge.Cuts")
            })
        {
            continue;
        }
        let width = graphic.width_nm.unwrap_or(100_000);
        match graphic.primitive_kind.as_str() {
            "polygon" => {
                let (min_x, min_y, max_x, max_y) = graphic.path.iter().fold(
                    (i64::MAX, i64::MAX, i64::MIN, i64::MIN),
                    |(min_x, min_y, max_x, max_y), point| {
                        (
                            min_x.min(point.x),
                            min_y.min(point.y),
                            max_x.max(point.x),
                            max_y.max(point.y),
                        )
                    },
                );
                if min_x <= max_x && min_y <= max_y {
                    out.push(WorldHitRegion {
                        target: HitTarget::AuthoredObject(target_id.to_string()),
                        shape: WorldHitShape::Rect(datum_gui_protocol::RectNm {
                            min_x,
                            min_y,
                            max_x,
                            max_y,
                        }),
                    });
                }
            }
            _ => {
                out.push(WorldHitRegion {
                    target: HitTarget::AuthoredObject(target_id.to_string()),
                    shape: WorldHitShape::Polyline {
                        path: graphic.path.clone(),
                        half_width_nm: (width as f32 * 0.5).max(180_000.0),
                    },
                });
            }
        }
    }
    // M7-SCN-007: per-contributor Edge.Cuts authored graphics are the
    // primary pick target for imported Edge.Cuts geometry. The assembled
    // board-boundary (scene.outline) is intentionally NOT registered as a
    // hit target under this ticket.
    for gfx in &scene.board_graphics {
        if !layer_visible(state, &gfx.layer_id) {
            continue;
        }
        let width = gfx.width_nm.unwrap_or(100_000);
        out.push(WorldHitRegion {
            target: HitTarget::AuthoredObject(gfx.object_id.clone()),
            shape: WorldHitShape::Polyline {
                path: gfx.path.clone(),
                half_width_nm: (width as f32 * 0.5).max(150_000.0),
            },
        });
    }
}

fn push_scene_overlay_and_hits(
    out: &mut Vec<Quad>,
    scene: &BoardReviewSceneV1,
    scene_viewport: RectPx,
    camera: CameraState,
    state: &ReviewWorkspaceState,
    text_runs: &mut Vec<TextRun>,
    hit_regions: &mut Vec<HitRegion>,
) {
    let board_field = inset_rect(scene_viewport, 10.0, 10.0, 10.0, 10.0);
    let projection = Projection::new(board_field, &scene.bounds, camera);
    let active_move_component_uuid: Option<String> = None;
    for component in &scene.components {
        if !authored_visible(state) || !layer_visible(state, &component.placement_layer) {
            continue;
        }
        if active_move_component_uuid.as_deref() == Some(component.component_uuid.as_str()) {
            continue;
        }
        let has_detail_text = component_has_detail_text(scene, &component.component_uuid);
        // Skip synthetic labels when imported silk text exists — silk handles it
        if has_detail_text {
            continue;
        }
        let selected = matches!(state.selection, SelectionTarget::AuthoredObject(ref id) if id == &component.object_id)
            || component_is_selection_active(&component.component_uuid, scene, state);
        let related = component_overlaps_active_action(component, state)
            || component_is_selection_related(&component.component_uuid, scene, state);
        let dimmed = dim_unrelated_active(state) && !selected && !related;
        let label_rect = project_rect(component.bounds, &projection);
        let label_text = truncate_text(&component.reference.to_uppercase(), 6);
        let label_size = if selected || related { 11.0 } else { 10.0 };
        let label_color = if selected {
            selected_silk_color(COMPONENT_SILK)
        } else if related {
            PAD_COPPER_RELATED
        } else {
            dim_context_color(COMPONENT_SILK, dimmed)
        };
        // Center label inside component body
        let label_x = label_rect.x + (label_rect.width * 0.5) - (label_text.len() as f32 * label_size * 0.32);
        let label_y = label_rect.y + (label_rect.height * 0.5) - (label_size * 0.5);
        draw_text_clipped(
            &label_text,
            label_x.max(label_rect.x + 4.0),
            label_y.max(board_field.y + 6.0),
            label_size,
            label_color,
            TextFace::Mono,
            scene_viewport,
            text_runs,
        );
        let label_hit = RectPx {
            x: label_x.max(label_rect.x + 2.0) - 4.0,
            y: label_y.max(board_field.y + 6.0) - label_size,
            width: (label_text.len() as f32 * label_size * 0.64).max(20.0),
            height: (label_size + 6.0).max(14.0),
        };
        hit_regions.push(HitRegion {
            target: HitTarget::AuthoredObject(component.object_id.clone()),
            rect: label_hit,
        });
        if !has_detail_text {
            let (label_x, label_y) = project_point(component.position, &projection);
            draw_text_clipped(
                &truncate_text(&component.reference.to_uppercase(), 6),
                label_x - 9.0,
                (label_y - 4.0).max(board_field.y + 6.0),
                9.0,
                if selected {
                    selected_silk_color([0.80, 0.82, 0.86])
                } else if related {
                    PAD_COPPER_RELATED
                } else {
                    dim_context_color([0.80, 0.82, 0.86], dimmed)
                },
                TextFace::Mono,
                scene_viewport,
                text_runs,
            );
            hit_regions.push(HitRegion {
                target: HitTarget::AuthoredObject(component.object_id.clone()),
                rect: RectPx {
                    x: label_x - 12.0,
                    y: (label_y - 4.0).max(board_field.y + 6.0) - 10.0,
                    width: 32.0,
                    height: 18.0,
                },
            });
        }
    }
    for text in &scene.component_texts {
        if !authored_visible(state) {
            continue;
        }
        if let Some(lid) = text.layer_id.as_deref() {
            if !layer_visible(state, lid) {
                continue;
            }
        }
        if active_move_component_uuid.as_deref() == Some(text.component_uuid.as_str()) {
            continue;
        }
        let related = scene.components.iter().any(|component| {
            component.component_uuid == text.component_uuid
                && component_overlaps_active_action(component, state)
        }) || component_is_selection_related(&text.component_uuid, scene, state);
        let selected = matches!(
            state.selection,
            SelectionTarget::AuthoredObject(ref id)
                if id == &format!("component:{}", text.component_uuid)
        ) || component_is_selection_active(&text.component_uuid, scene, state);
        let dimmed = dim_unrelated_active(state) && !selected && !related;
        push_component_text_world(
            out,
            text_runs,
            text,
            &scene.layers,
            &projection,
            scene_viewport,
            selected,
            related,
            dimmed,
        );
        let (tx, ty) = project_point(text.position, &projection);
        let text_size = footprint_text_size_px(text.height_nm, &projection);
        hit_regions.push(HitRegion {
            target: HitTarget::AuthoredObject(format!("component:{}", text.component_uuid)),
            rect: RectPx {
                x: tx - (text.text.len() as f32 * text_size * 0.36).max(10.0),
                y: ty - text_size,
                width: (text.text.len() as f32 * text_size * 0.72).max(24.0),
                height: (text_size + 6.0).max(14.0),
            },
        });
    }
    // Show net name for selected or hovered pads
    for pad in &scene.pads {
        let selected_pad = matches!(&state.selection, SelectionTarget::AuthoredObject(id) if id == &pad.object_id);
        let hovered_pad = is_hovered(state, &pad.object_id);
        if (selected_pad || hovered_pad) && pad.net_uuid.is_some() {
            let net_label = state
                .review
                .proposal_actions
                .iter()
                .find(|a| Some(&a.net_uuid) == pad.net_uuid.as_ref())
                .map(|a| a.net_name.clone())
                .unwrap_or_else(|| "NET".to_string());
            let (px, py) = project_point(pad.center, &projection);
            draw_text_clipped(
                &net_label.to_uppercase(),
                px + 8.0,
                py - 14.0,
                10.0,
                TEXT_ACCENT,
                TextFace::Mono,
                scene_viewport,
                text_runs,
            );
        }
    }
    if let Some(active_component_uuid) = active_move_component_uuid.as_deref()
        && let Some(component) = scene
            .components
            .iter()
            .find(|component| component.component_uuid == active_component_uuid)
    {
        let has_detail_text = component_has_detail_text(scene, &component.component_uuid);
        let selected = true;
        let related = component_overlaps_active_action(component, state);
        let dimmed = false;
        let label_rect = project_rect(component.bounds, &projection);
        draw_text_clipped(
            &truncate_text(&component.reference.to_uppercase(), 6),
            label_rect.x + 6.0,
            (label_rect.y - 11.0).max(board_field.y + 6.0),
            10.0,
            selected_silk_color(COMPONENT_SILK),
            TextFace::Mono,
            scene_viewport,
            text_runs,
        );
        if !has_detail_text {
            let (label_x, label_y) = project_point(component.position, &projection);
            draw_text_clipped(
                &truncate_text(&component.reference.to_uppercase(), 6),
                label_x - 9.0,
                (label_y - 4.0).max(board_field.y + 6.0),
                9.0,
                selected_silk_color([0.80, 0.82, 0.86]),
                TextFace::Mono,
                scene_viewport,
                text_runs,
            );
        }
        for text in scene
            .component_texts
            .iter()
            .filter(|text| text.component_uuid == component.component_uuid)
        {
            push_component_text_world(
                out,
                text_runs,
                text,
                &scene.layers,
                &projection,
                scene_viewport,
                selected,
                related,
                dimmed,
            );
        }
    }
    if proposed_visible(state) {
        for overlay in &scene.proposal_overlay_primitives {
            if !overlay
                .layer_id
                .as_deref()
                .is_none_or(|layer_id| layer_visible(state, layer_id))
            {
                continue;
            }
        let selected = overlay.proposal_action_id == state.active_review_target_id;
        let color = match overlay.render_role.as_str() {
            "proposed_focus" if selected => PROPOSAL_FOCUS,
            "proposed_overlay" if selected => PROPOSAL_FOCUS,
            "proposed_overlay" => PROPOSAL_BASE,
            "authored_related" => AUTHOR_RELATED,
            _ => PROPOSAL_BASE,
        };
        let rects = push_overlay(
            out,
            overlay,
            &projection,
            color,
            selected,
            false,
        );
        for rect in rects {
            hit_regions.push(HitRegion {
                target: HitTarget::ReviewAction(overlay.proposal_action_id.clone()),
                rect,
            });
        }
    }
    }
    let active_evidence_key = state
        .selected_review_action()
        .map(|action| format!("segment:{}", action.selected_path_segment_index));
    for review in &scene.review_primitives {
        let active = review.evidence_key.as_ref() == active_evidence_key.as_ref();
        push_dashed_polyline_segments(
            out,
            &review.path,
            &projection,
            DIAGNOSTIC_UNDERLAY,
            if active { 2.1 } else { 1.6 },
            10.0,
            6.0,
        );
        push_dashed_polyline_segments(
            out,
            &review.path,
            &projection,
            if active {
                DIAGNOSTIC_FOCUS
            } else {
                DIAGNOSTIC_BASE
            },
            if active { 1.2 } else { 0.9 },
            10.0,
            6.0,
        );
        push_points(
            out,
            &review.path,
            &projection,
            if active {
                DIAGNOSTIC_FOCUS
            } else {
                DIAGNOSTIC_BASE
            },
            if active { 4.0 } else { 3.0 },
        );
    }
}

fn pad_matches_active_action(
    pad: &datum_gui_protocol::PadPrimitive,
    state: &ReviewWorkspaceState,
) -> bool {
    let Some(action) = state.selected_review_action() else {
        return false;
    };
    pad.pad_uuid == action.from_anchor_pad_uuid || pad.pad_uuid == action.to_anchor_pad_uuid
}

fn track_matches_active_action(
    track: &datum_gui_protocol::TrackPrimitive,
    state: &ReviewWorkspaceState,
) -> bool {
    let Some(action) = state.selected_review_action() else {
        return false;
    };
    let Some(net_uuid) = &track.net_uuid else {
        return false;
    };
    net_uuid == &action.net_uuid
}

fn via_matches_active_action(
    via: &datum_gui_protocol::ViaPrimitive,
    state: &ReviewWorkspaceState,
) -> bool {
    let Some(action) = state.selected_review_action() else {
        return false;
    };
    let Some(net_uuid) = &via.net_uuid else {
        return false;
    };
    net_uuid == &action.net_uuid
}

fn zone_matches_active_action(
    zone: &datum_gui_protocol::ZonePrimitive,
    state: &ReviewWorkspaceState,
) -> bool {
    let Some(action) = state.selected_review_action() else {
        return false;
    };
    let Some(net_uuid) = &zone.net_uuid else {
        return false;
    };
    net_uuid == &action.net_uuid
}

fn has_review_focus(state: &ReviewWorkspaceState) -> bool {
    state.selected_review_action().is_some()
}

fn component_overlaps_active_action(
    component: &datum_gui_protocol::ComponentBounds,
    state: &ReviewWorkspaceState,
) -> bool {
    let Some(action) = state.selected_review_action() else {
        return false;
    };
    point_in_rect(action.from, component.bounds) || point_in_rect(action.to, component.bounds)
}

fn point_in_rect(point: PointNm, rect: datum_gui_protocol::RectNm) -> bool {
    point.x >= rect.min_x && point.x <= rect.max_x && point.y >= rect.min_y && point.y <= rect.max_y
}

fn polyline_contains_world_point(path: &[PointNm], point: PointNm, half_width_nm: f32) -> bool {
    let px = point.x as f32;
    let py = point.y as f32;
    let threshold_sq = half_width_nm * half_width_nm;
    path.windows(2).any(|segment| {
        let ax = segment[0].x as f32;
        let ay = segment[0].y as f32;
        let bx = segment[1].x as f32;
        let by = segment[1].y as f32;
        let dx = bx - ax;
        let dy = by - ay;
        let len_sq = dx * dx + dy * dy;
        if len_sq <= 1.0 {
            let ddx = px - ax;
            let ddy = py - ay;
            return ddx * ddx + ddy * ddy <= threshold_sq;
        }
        let t = (((px - ax) * dx + (py - ay) * dy) / len_sq).clamp(0.0, 1.0);
        let cx = ax + dx * t;
        let cy = ay + dy * t;
        let ddx = px - cx;
        let ddy = py - cy;
        ddx * ddx + ddy * ddy <= threshold_sq
    })
}

fn component_graphic_matches_active_action(
    graphic: &ComponentGraphicPrimitive,
    scene: &BoardReviewSceneV1,
    state: &ReviewWorkspaceState,
) -> bool {
    scene.components.iter().any(|component| {
        component.component_uuid == graphic.component_uuid
            && component_overlaps_active_action(component, state)
    })
}

fn dim_with_policy(
    color: [f32; 3],
    dimmed: bool,
    factor: f32,
    floor: f32,
    board_mix: f32,
) -> [f32; 3] {
    if !dimmed {
        return color;
    }
    let dimmed = [
        (color[0] * factor).max(floor),
        (color[1] * factor).max(floor),
        (color[2] * factor).max(floor),
    ];
    mix_color(dimmed, BOARD_INNER_FIELD, board_mix)
}

fn dim_authored_color(color: [f32; 3], dimmed: bool) -> [f32; 3] {
    dim_with_policy(color, dimmed, AUTHORED_DIM_FACTOR, 0.14, 0.08)
}

fn dim_process_color(color: [f32; 3], dimmed: bool) -> [f32; 3] {
    dim_with_policy(color, dimmed, PROCESS_DIM_FACTOR, 0.18, 0.05)
}

fn dim_structural_color(color: [f32; 3], dimmed: bool) -> [f32; 3] {
    dim_with_policy(color, dimmed, STRUCTURAL_DIM_FACTOR, 0.12, 0.10)
}

fn dim_context_color(color: [f32; 3], dimmed: bool) -> [f32; 3] {
    dim_with_policy(color, dimmed, CONTEXT_DIM_FACTOR, 0.20, 0.04)
}

fn selected_copper_color(base: [f32; 3]) -> [f32; 3] {
    let tinted = mix_color(base, [0.95, 0.50, 0.92], 0.52);
    mix_color(tinted, [0.98, 0.94, 1.0], 0.12)
}

fn selected_silk_color(base: [f32; 3]) -> [f32; 3] {
    mix_color(base, [0.98, 0.98, 0.99], 0.72)
}

fn selected_mechanical_color(base: [f32; 3]) -> [f32; 3] {
    mix_color(base, [0.90, 0.94, 0.98], 0.45)
}

fn width_to_px(width_nm: i64) -> f32 {
    ((width_nm as f32) / 120_000.0).clamp(0.9, 3.6)
}

fn overlay_route_width_px(
    width_nm: Option<i64>,
    selected: bool,
    projection: Option<&Projection>,
) -> f32 {
    // If we have real width and a projection, use camera-aware sizing.
    // Preserve true proportional width down to a sub-pixel legibility floor so
    // distinct physical widths remain visually distinct at wide zoom.
    if let (Some(w), Some(proj)) = (width_nm, projection) {
        let projected = proj.world_length_to_px(w);
        let floor = if selected { 2.5 } else { 2.0 };
        return projected.max(floor).clamp(2.0, 32.0);
    }
    let base = width_nm.map(width_to_px).unwrap_or(2.4);
    let scaled = if selected { base * 3.2 } else { base * 2.0 };
    scaled.clamp(
        if selected { 4.5 } else { 3.2 },
        if selected { 10.0 } else { 7.0 },
    )
}

fn push_overlay(
    out: &mut Vec<Quad>,
    overlay: &ProposalOverlayPrimitive,
    projection: &Projection,
    color: [f32; 3],
    selected: bool,
    editor_move_preview: bool,
) -> Vec<RectPx> {
    if editor_move_preview {
        return push_overlay_move_preview(out, overlay, projection, color, selected);
    }
    let layer_color = proposal_layer_color(overlay.layer_id.as_deref());
    let outer_color = if selected {
        PROPOSAL_OUTER
    } else {
        mix_color(color, layer_color, 0.45)
    };
    let underlay_color = if selected {
        PROPOSAL_UNDERLAY
    } else {
        mix_color(PROPOSAL_UNDERLAY, layer_color, 0.18)
    };
    let fill_color = if selected { PROPOSAL_FOCUS } else { color };
    match overlay.primitive_kind.as_str() {
        "anchor_marker" => {
            let outer_size = if selected { 17.0 } else { 12.0 };
            let ring_size = if selected { 10.0 } else { 7.0 };
            let core_size = if selected { 4.2 } else { 3.0 };
            let mut rects = push_points(
                out,
                &overlay.path,
                projection,
                if selected {
                    PROPOSAL_UNDERLAY
                } else {
                    [0.30, 0.22, 0.12]
                },
                outer_size,
            );
            rects.extend(push_points(
                out,
                &overlay.path,
                projection,
                if selected {
                    PROPOSAL_FOCUS
                } else {
                    PROPOSAL_ANCHOR_RING
                },
                ring_size,
            ));
            rects.extend(push_points(
                out,
                &overlay.path,
                projection,
                PROPOSAL_ANCHOR_CORE,
                core_size,
            ));
            rects
        }
        _ => {
            let route_width = overlay_route_width_px(overlay.width_nm, selected, Some(projection));
            let underlay_width = if selected {
                route_width + 5.2
            } else {
                route_width + 1.8
            };
            let outer_width = if selected {
                route_width + 2.2
            } else {
                route_width + 0.55
            };
            let inner_width = if selected {
                route_width + 0.45
            } else {
                route_width.max(1.5)
            };
            let mut hit_rects = push_polyline_segments(
                out,
                &overlay.path,
                projection,
                underlay_color,
                underlay_width,
            );
            if selected || overlay.path.len() == 2 {
                hit_rects.extend(push_polyline_endcaps(
                    out,
                    &overlay.path,
                    projection,
                    outer_color,
                    outer_width,
                    (route_width * 2.7).clamp(10.0, 18.0),
                ));
            }
            if let (Some(first), Some(last)) = (overlay.path.first(), overlay.path.last()) {
                let endpoint_radius = if selected {
                    (route_width + 1.8).clamp(4.8, 8.0)
                } else {
                    (route_width + 1.0).clamp(3.8, 5.8)
                };
                hit_rects.extend(push_points(
                    out,
                    &[*first, *last],
                    projection,
                    outer_color,
                    endpoint_radius,
                ));
                hit_rects.extend(push_points(
                    out,
                    &[*first, *last],
                    projection,
                    fill_color,
                    (endpoint_radius * 0.42).clamp(1.8, 3.2),
                ));
            }
            hit_rects.extend(push_polyline_segments(
                out,
                &overlay.path,
                projection,
                outer_color,
                outer_width,
            ));
            hit_rects.extend(push_polyline_segments(
                out,
                &overlay.path,
                projection,
                if selected { PROPOSAL_FOCUS } else { color },
                inner_width,
            ));
            if selected {
                hit_rects.extend(push_polyline_endcaps(
                    out,
                    &overlay.path,
                    projection,
                    PROPOSAL_FOCUS,
                    route_width + 0.8,
                    (route_width * 2.0).clamp(8.0, 14.0),
                ));
            }
            let corner_fill = if selected {
                (inner_width - 0.25).max(1.2)
            } else {
                (inner_width - 0.35).max(1.0)
            };
            if overlay.path.len() > 2 {
                hit_rects.extend(push_points(
                    out,
                    &overlay.path[1..overlay.path.len() - 1],
                    projection,
                    fill_color,
                    corner_fill,
                ));
            }
            hit_rects
        }
    }
}

fn push_overlay_move_preview(
    out: &mut Vec<Quad>,
    overlay: &ProposalOverlayPrimitive,
    projection: &Projection,
    color: [f32; 3],
    selected: bool,
) -> Vec<RectPx> {
    match overlay.primitive_kind.as_str() {
        "anchor_marker" => push_points(
            out,
            &overlay.path,
            projection,
            if selected {
                PROPOSAL_FOCUS
            } else {
                AUTHOR_RELATED
            },
            if selected { 5.0 } else { 4.0 },
        ),
        _ => {
            let guide_color = if selected {
                mix_color(PROPOSAL_FOCUS, PROPOSAL_CENTERLINE, 0.35)
            } else {
                mix_color(color, PROPOSAL_CENTERLINE, 0.55)
            };
            let underlay = if selected {
                mix_color(PROPOSAL_UNDERLAY, guide_color, 0.25)
            } else {
                mix_color(DIAGNOSTIC_UNDERLAY, guide_color, 0.18)
            };
            let base_width = if selected { 1.4 } else { 1.1 };
            let mut rects = push_dashed_polyline_segments(
                out,
                &overlay.path,
                projection,
                underlay,
                base_width + 0.9,
                10.0,
                6.0,
            );
            rects.extend(push_dashed_polyline_segments(
                out,
                &overlay.path,
                projection,
                guide_color,
                base_width,
                10.0,
                6.0,
            ));
            rects.extend(push_points(
                out,
                &overlay.path,
                projection,
                guide_color,
                if selected { 3.4 } else { 3.0 },
            ));
            rects
        }
    }
}

fn push_scene_grid(out: &mut Vec<Quad>, projection: &Projection) {
    let detail = detail_tier(projection);
    let major_pitch_nm = match detail {
        DetailTier::Fine => 2_500_000,
        DetailTier::Normal => 5_000_000,
        DetailTier::Coarse => 10_000_000,
    };
    let minor_pitch_nm = match detail {
        DetailTier::Fine => Some(1_250_000),
        DetailTier::Normal => Some(2_500_000),
        DetailTier::Coarse => None,
    };
    if let Some(minor_pitch) = minor_pitch_nm {
        push_grid_axis_lines(
            out,
            projection,
            minor_pitch,
            board_surface_color(BoardSurfaceRole::GridMinor),
        );
    }
    push_grid_axis_lines(
        out,
        projection,
        major_pitch_nm,
        board_surface_color(BoardSurfaceRole::GridMajor),
    );
}

fn push_grid_axis_lines(
    out: &mut Vec<Quad>,
    projection: &Projection,
    pitch_nm: i64,
    color: [f32; 3],
) {
    if pitch_nm <= 0 {
        return;
    }
    let start_x = floor_multiple(projection.bounds.min_x, pitch_nm);
    let end_x = ceil_multiple(projection.bounds.max_x, pitch_nm);
    let mut x = start_x;
    while x <= end_x {
        let x_px = projection
            .project_point(PointNm {
                x,
                y: projection.bounds.min_y,
            })
            .0;
        out.push(Quad::from_rect(
            RectPx {
                x: x_px,
                y: projection.viewport.y,
                width: 1.0,
                height: projection.viewport.height,
            },
            color,
        ));
        x += pitch_nm;
    }
    let start_y = floor_multiple(projection.bounds.min_y, pitch_nm);
    let end_y = ceil_multiple(projection.bounds.max_y, pitch_nm);
    let mut y = start_y;
    while y <= end_y {
        let y_px = projection
            .project_point(PointNm {
                x: projection.bounds.min_x,
                y,
            })
            .1;
        out.push(Quad::from_rect(
            RectPx {
                x: projection.viewport.x,
                y: y_px,
                width: projection.viewport.width,
                height: 1.0,
            },
            color,
        ));
        y += pitch_nm;
    }
}

#[allow(dead_code)]
fn push_polygon_fill(
    out: &mut Vec<Quad>,
    polygon: &[PointNm],
    projection: &Projection,
    color: [f32; 3],
) {
    if polygon.len() < 3 {
        return;
    }
    let projected: Vec<(f32, f32)> = polygon
        .iter()
        .map(|point| projection.project_point(*point))
        .collect();
    push_projected_polygon_fill(out, &projected, color);
}

#[allow(dead_code)]
fn push_component_primitive(
    out: &mut Vec<Quad>,
    component: &datum_gui_protocol::ComponentBounds,
    projection: &Projection,
    selected: bool,
    related: bool,
    dimmed: bool,
) -> RectPx {
    let body = push_world_rect(
        out,
        component.bounds,
        projection,
        dim_structural_color(
            if selected {
                COMPONENT_BODY_SELECTED
            } else if related {
                COMPONENT_BODY_RELATED
            } else {
                COMPONENT_BODY
            },
            dimmed,
        ),
    );
    let header_h = body.height.clamp(6.0, 12.0);
    let header = RectPx {
        x: body.x + 1.0,
        y: body.y + 1.0,
        width: (body.width - 2.0).max(1.0),
        height: (header_h - 1.0).max(1.0),
    };
    out.push(Quad::from_rect(header, dim_structural_color(COMPONENT_HEADER, dimmed)));
    let inner = inset_rect(body, 2.0, header_h + 1.0, 2.0, 2.0);
    if inner.width > 2.0 && inner.height > 2.0 {
        out.push(Quad::from_rect(
            inner,
            dim_structural_color([0.30, 0.32, 0.36], dimmed),
        ));
    }
    let pin1 = RectPx {
        x: body.x + 4.0,
        y: body.y + 4.0,
        width: 3.0,
        height: 3.0,
    };
    out.push(Quad::from_rect(
        pin1,
        dim_structural_color(
            if selected || related {
                PAD_COPPER_RELATED
            } else {
                PAD_COPPER
            },
            dimmed,
        ),
    ));
    push_rect_border(
        out,
        body,
        dim_structural_color(
            if selected {
                AUTHOR_SELECTED
            } else if related {
                AUTHOR_RELATED
            } else {
                COMPONENT_OUTLINE
            },
            dimmed,
        ),
        1.0,
    );
    body
}

#[allow(dead_code)]
fn push_component_primitive_world(
    out: &mut Vec<Quad>,
    component: &datum_gui_protocol::ComponentBounds,
    selected: bool,
    related: bool,
    dimmed: bool,
    reference_projection: &Projection,
) {
    let body_color = dim_structural_color(
        if selected {
            COMPONENT_BODY_SELECTED
        } else if related {
            COMPONENT_BODY_RELATED
        } else {
            COMPONENT_BODY
        },
        dimmed,
    );
    push_world_rect_nm(out, component.bounds, body_color);
    let stroke_nm = world_stroke_nm(if selected { 2.5 } else { 1.0 }, reference_projection);
    let header_size_nm = world_stroke_nm(10.0, reference_projection);
    let s = stroke_nm.round() as i64;
    let h = header_size_nm.round() as i64;
    let rotation = component.rotation_degrees.round() as i32;
    let header = match rotation.rem_euclid(360) {
        180 => datum_gui_protocol::RectNm {
            min_x: component.bounds.min_x + s,
            min_y: component.bounds.max_y - h,
            max_x: component.bounds.max_x - s,
            max_y: component.bounds.max_y - s,
        },
        90 => datum_gui_protocol::RectNm {
            min_x: component.bounds.max_x - h,
            min_y: component.bounds.min_y + s,
            max_x: component.bounds.max_x - s,
            max_y: component.bounds.max_y - s,
        },
        270 => datum_gui_protocol::RectNm {
            min_x: component.bounds.min_x + s,
            min_y: component.bounds.min_y + s,
            max_x: component.bounds.min_x + h,
            max_y: component.bounds.max_y - s,
        },
        _ => datum_gui_protocol::RectNm {
            min_x: component.bounds.min_x + s,
            min_y: component.bounds.min_y + s,
            max_x: component.bounds.max_x - s,
            max_y: component.bounds.min_y + h,
        },
    };
    if header.max_x > header.min_x && header.max_y > header.min_y {
        push_world_rect_nm(out, header, dim_structural_color(COMPONENT_HEADER, dimmed));
    }
    push_world_rect_border_nm(
        out,
        component.bounds,
        dim_structural_color(
            if selected {
                AUTHOR_SELECTED
            } else if related {
                AUTHOR_RELATED
            } else {
                COMPONENT_OUTLINE
            },
            dimmed,
        ),
        stroke_nm,
    );
}

#[allow(dead_code)]
fn push_component_graphic_primitive(
    out: &mut Vec<Quad>,
    graphic: &ComponentGraphicPrimitive,
    projection: &Projection,
    selected: bool,
    related: bool,
    dimmed: bool,
) {
    let (base_color, width_scale) = match graphic.render_role.as_str() {
        "component_mechanical" => (
            if selected {
                selected_mechanical_color(COMPONENT_MECHANICAL)
            } else if related {
                COMPONENT_MECHANICAL_RELATED
            } else {
                COMPONENT_MECHANICAL
            },
            1.0,
        ),
        _ => (
            if selected {
                selected_silk_color(component_silk_color(graphic.layer_id.as_deref()))
            } else if related {
                COMPONENT_SILK_RELATED
            } else {
                component_silk_color(graphic.layer_id.as_deref())
            },
            1.15,
        ),
    };
    let color = dim_context_color(base_color, dimmed);
    if graphic.primitive_kind == "polygon" && graphic.path.len() >= 3 {
        let fill_color = match graphic.render_role.as_str() {
            "component_mechanical" => mix_color(color, BOARD_INNER_FIELD, 0.55),
            _ if graphic.width_nm.is_none() => color,
            _ => mix_color(color, BOARD_INNER_FIELD, 0.20),
        };
        push_polygon_fill(out, &graphic.path, projection, fill_color);
        if graphic.width_nm.is_none() && graphic.render_role != "component_mechanical" {
            return;
        }
    }
    let width = graphic.width_nm.map(width_to_px).unwrap_or(1.1) * width_scale;
    let path = if graphic.closed {
        close_path(&graphic.path)
    } else {
        graphic.path.clone()
    };
    if graphic.closed && graphic.render_role == "component_mechanical" {
        push_dashed_polyline_segments(out, &path, projection, color, width.max(0.8), 10.0, 7.0);
        return;
    }
    push_polyline_segments(out, &path, projection, color, width.max(1.0));
}

fn push_component_graphic_primitive_world(
    out: &mut Vec<Quad>,
    graphic: &ComponentGraphicPrimitive,
    scene_layers: &[datum_gui_protocol::SceneLayer],
    selected: bool,
    related: bool,
    dimmed: bool,
    reference_projection: &Projection,
) {
    let (base_color, width_scale) = match graphic.render_role.as_str() {
        "component_mechanical" => (
            if selected {
                selected_mechanical_color(COMPONENT_MECHANICAL)
            } else if related {
                COMPONENT_MECHANICAL_RELATED
            } else {
                COMPONENT_MECHANICAL
            },
            1.0,
        ),
        _ => (
            if selected {
                selected_silk_color(
                    resolve_layer_appearance_with_scene(graphic.layer_id.as_deref(), scene_layers)
                        .silkscreen,
                )
            } else if related {
                COMPONENT_SILK_RELATED
            } else {
                resolve_layer_appearance_with_scene(graphic.layer_id.as_deref(), scene_layers)
                    .silkscreen
            },
            1.15,
        ),
    };
    let color = dim_context_color(base_color, dimmed);
    if graphic.primitive_kind == "polygon" && graphic.path.len() >= 3 {
        let fill_color = match graphic.render_role.as_str() {
            "component_mechanical" => mix_color(color, BOARD_INNER_FIELD, 0.55),
            _ if graphic.width_nm.is_none() => color,
            _ => mix_color(color, BOARD_INNER_FIELD, 0.20),
        };
        push_world_polygon_fill(out, &graphic.path, fill_color);
        if graphic.width_nm.is_none() && graphic.render_role != "component_mechanical" {
            return;
        }
    }
    let width_nm = world_stroke_nm(
        graphic.width_nm.map(width_to_px).unwrap_or(1.1) * width_scale,
        reference_projection,
    );
    let path = if graphic.closed {
        close_path(&graphic.path)
    } else {
        graphic.path.clone()
    };
    if graphic.closed && graphic.render_role == "component_mechanical" {
        push_world_dashed_polyline_segments(
            out,
            &path,
            width_nm.max(1.0),
            world_stroke_nm(10.0, reference_projection),
            world_stroke_nm(7.0, reference_projection),
            color,
        );
        return;
    }
    let w = width_nm.max(1.0);
    push_world_polyline_segments(out, &path, w, color);
    // Round-cap each vertex so that separate fp_line segments sharing an
    // endpoint don't leave diagonal gaps at 90-degree corners. Each cap is
    // a small filled circle matching the stroke width.
    let half = (w * 0.5) as i64;
    for pt in &path {
        push_world_ellipse_nm(
            out,
            datum_gui_protocol::RectNm {
                min_x: pt.x - half,
                min_y: pt.y - half,
                max_x: pt.x + half,
                max_y: pt.y + half,
            },
            color,
            16,
        );
    }
}

fn push_board_graphic_primitive_world(
    out: &mut Vec<Quad>,
    graphic: &BoardGraphicPrimitive,
    scene_layers: &[datum_gui_protocol::SceneLayer],
    dimmed: bool,
    reference_projection: &Projection,
) {
    let app = resolve_layer_appearance_with_scene(Some(&graphic.layer_id), scene_layers);
    let layer_name = scene_layers
        .iter()
        .find(|layer| layer.layer_id == graphic.layer_id)
        .map(|layer| layer.name.as_str())
        .unwrap_or("");
    let base_color = if layer_name.ends_with(".SilkS") {
        app.silkscreen
    } else {
        app.authored_track
    };
    let color = dim_context_color(base_color, dimmed);
    if graphic.primitive_kind == "polygon" && graphic.path.len() >= 3 {
        push_world_polygon_fill(out, &graphic.path, color);
        if graphic.width_nm.is_none() {
            return;
        }
    }
    let width_nm = world_stroke_nm(
        graphic.width_nm.map(width_to_px).unwrap_or(1.1),
        reference_projection,
    )
    .max(1.0);
    let path = if graphic.primitive_kind == "polygon" {
        close_path(&graphic.path)
    } else {
        graphic.path.clone()
    };
    push_world_polyline_segments(out, &path, width_nm, color);
    let half = (width_nm * 0.5) as i64;
    for pt in &path {
        push_world_ellipse_nm(
            out,
            datum_gui_protocol::RectNm {
                min_x: pt.x - half,
                min_y: pt.y - half,
                max_x: pt.x + half,
                max_y: pt.y + half,
            },
            color,
            16,
        );
    }
}

#[allow(dead_code)]
fn push_pad_primitive(
    out: &mut Vec<Quad>,
    pad: &datum_gui_protocol::PadPrimitive,
    projection: &Projection,
    _layer_id: &str,
    outer_color: [f32; 3],
    drill_nm: Option<i64>,
    dimmed: bool,
) -> RectPx {
    let outer_color = dim_authored_color(outer_color, dimmed);
    let px = project_rect(pad.bounds, projection);
    let is_ellipse = matches!(pad.shape_kind.as_str(), "circle" | "oval");
    let copper_outline = projected_pad_outline(pad, projection, 0.0);
    push_convex_polygon_fill(out, &copper_outline, outer_color);
    if is_ellipse && drill_nm.is_none() {
        let inner = inset_rect(
            px,
            px.width * 0.22,
            px.height * 0.22,
            px.width * 0.22,
            px.height * 0.22,
        );
        if inner.width > 1.0 && inner.height > 1.0 {
            push_projected_ellipse(out, inner, dim_authored_color([0.79, 0.49, 0.26], dimmed), 24);
        }
    }
    if let Some(drill_nm) = drill_nm.filter(|value| *value > 0) {
        let drill_px =
            world_length_to_px(drill_nm, projection).clamp(4.0, px.width.min(px.height) - 2.0);
        let hole = RectPx {
            x: px.x + (px.width - drill_px) * 0.5,
            y: px.y + (px.height - drill_px) * 0.5,
            width: drill_px,
            height: drill_px,
        };
        push_projected_ellipse(out, hole, dim_structural_color([0.10, 0.11, 0.12], dimmed), 22);
        let hole_border = inset_rect(hole, 0.8, 0.8, 0.8, 0.8);
        if hole_border.width > 1.0 && hole_border.height > 1.0 {
            push_projected_ellipse(out, hole_border, dim_structural_color([0.62, 0.66, 0.70], dimmed), 22);
            let hole_inner = inset_rect(hole_border, 1.0, 1.0, 1.0, 1.0);
            if hole_inner.width > 1.0 && hole_inner.height > 1.0 {
                push_projected_ellipse(out, hole_inner, dim_structural_color([0.10, 0.11, 0.12], dimmed), 22);
            }
        }
    }
    px
}

fn push_pad_primitive_world(
    out: &mut Vec<Quad>,
    pad: &datum_gui_protocol::PadPrimitive,
    layer_id: &str,
    outer_color: [f32; 3],
    drill_nm: Option<i64>,
    dimmed: bool,
    reference_projection: &Projection,
) {
    let outer_color = dim_authored_color(outer_color, dimmed);
    let _ = layer_id;
    let copper_outline = world_pad_outline(pad, 0.0, reference_projection);
    push_world_polygon_fill(out, &copper_outline, outer_color);
    if let Some(drill_nm) = drill_nm.filter(|value| *value > 0) {
        let half = drill_nm as f32 * 0.5;
        let center_x = (pad.bounds.min_x + pad.bounds.max_x) as f32 * 0.5;
        let center_y = (pad.bounds.min_y + pad.bounds.max_y) as f32 * 0.5;
        let hole = datum_gui_protocol::RectNm {
            min_x: (center_x - half).round() as i64,
            min_y: (center_y - half).round() as i64,
            max_x: (center_x + half).round() as i64,
            max_y: (center_y + half).round() as i64,
        };
        push_world_ellipse_nm(out, hole, dim_structural_color([0.10, 0.11, 0.12], dimmed), 128);
    }
}

fn pad_dimensions_nm(pad: &datum_gui_protocol::PadPrimitive) -> (f32, f32) {
    (
        (pad.bounds.max_x - pad.bounds.min_x).max(1) as f32,
        (pad.bounds.max_y - pad.bounds.min_y).max(1) as f32,
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PadProcessLayerKind {
    Mask,
    Paste,
}

fn derived_process_pad(
    pad: &datum_gui_protocol::PadPrimitive,
    process_layer_id: &str,
    kind: PadProcessLayerKind,
    _setup: &datum_gui_protocol::ScenePadExpansionSetup,
) -> datum_gui_protocol::PadPrimitive {
    let (width_nm, height_nm) = pad_dimensions_nm(pad);
    let (expanded_width_nm, expanded_height_nm) = match kind {
        PadProcessLayerKind::Mask => {
            let clearance = pad.solder_mask_margin_nm as f32;
            (
                (width_nm + clearance * 2.0).max(1.0),
                (height_nm + clearance * 2.0).max(1.0),
            )
        }
        PadProcessLayerKind::Paste => {
            let clearance = pad.solder_paste_margin_nm as f32;
            let ratio = pad.solder_paste_margin_ratio_ppm as f32 / 1_000_000.0;
            (
                (width_nm + width_nm * ratio + clearance * 2.0).max(1.0),
                (height_nm + height_nm * ratio + clearance * 2.0).max(1.0),
            )
        }
    };
    let half_w = expanded_width_nm * 0.5;
    let half_h = expanded_height_nm * 0.5;
    let center_x = pad.center.x as f32;
    let center_y = pad.center.y as f32;
    let mut derived = pad.clone();
    derived.layer_id = process_layer_id.to_string();
    derived.bounds = datum_gui_protocol::RectNm {
        min_x: (center_x - half_w).round() as i64,
        min_y: (center_y - half_h).round() as i64,
        max_x: (center_x + half_w).round() as i64,
        max_y: (center_y + half_h).round() as i64,
    };
    // Process apertures are not annular copper objects; render as the opening/aperture shape.
    derived.drill_nm = None;
    derived
}

fn pad_corner_radius_nm(
    pad: &datum_gui_protocol::PadPrimitive,
    width_nm: f32,
    height_nm: f32,
    reference_projection: &Projection,
    inset_nm: f32,
) -> f32 {
    let width_nm = (width_nm - inset_nm * 2.0).max(1.0);
    let height_nm = (height_nm - inset_nm * 2.0).max(1.0);
    match pad.shape_kind.as_str() {
        "circle" => width_nm.min(height_nm) * 0.5,
        "oval" => width_nm.min(height_nm) * 0.5,
        "roundrect" => {
            let ratio = (pad.roundrect_rratio_ppm as f32 / 1_000_000.0).clamp(0.0, 0.5);
            let radius = width_nm.min(height_nm) * ratio;
            radius.max(world_stroke_nm(1.0, reference_projection))
        }
        _ => 0.0,
    }
}

fn rotate_point_about_center(
    center: (f32, f32),
    local: (f32, f32),
    rotation_degrees: f32,
) -> (f32, f32) {
    let rad = (-rotation_degrees).to_radians();
    let cos = rad.cos();
    let sin = rad.sin();
    (
        center.0 + local.0 * cos - local.1 * sin,
        center.1 + local.0 * sin + local.1 * cos,
    )
}

fn rounded_rect_points(
    center: (f32, f32),
    width: f32,
    height: f32,
    rotation_degrees: f32,
    radius: f32,
) -> Vec<(f32, f32)> {
    let half_w = width * 0.5;
    let half_h = height * 0.5;
    let radius = radius.min(half_w).min(half_h).max(0.0);
    if radius <= 0.5 {
        return [
            (-half_w, -half_h),
            (half_w, -half_h),
            (half_w, half_h),
            (-half_w, half_h),
        ]
        .into_iter()
        .map(|local| rotate_point_about_center(center, local, rotation_degrees))
        .collect();
    }

    let segments_per_corner = 8usize;
    let arc_step = std::f32::consts::FRAC_PI_2 / segments_per_corner as f32;
    let corner_centers = [
        (half_w - radius, -half_h + radius, -std::f32::consts::FRAC_PI_2),
        (half_w - radius, half_h - radius, 0.0),
        (-(half_w - radius), half_h - radius, std::f32::consts::FRAC_PI_2),
        (
            -(half_w - radius),
            -(half_h - radius),
            std::f32::consts::PI,
        ),
    ];
    let mut points = Vec::with_capacity(corner_centers.len() * (segments_per_corner + 1));
    for (cx, cy, start) in corner_centers {
        for step in 0..=segments_per_corner {
            let angle = start + arc_step * step as f32;
            let local = (cx + radius * angle.cos(), cy + radius * angle.sin());
            points.push(rotate_point_about_center(center, local, rotation_degrees));
        }
    }
    points
}

fn ellipse_points(
    center: (f32, f32),
    width: f32,
    height: f32,
    rotation_degrees: f32,
    segments: usize,
) -> Vec<(f32, f32)> {
    let rx = width * 0.5;
    let ry = height * 0.5;
    let segments = segments.max(24);
    (0..segments)
        .map(|i| {
            let theta = std::f32::consts::TAU * (i as f32) / (segments as f32);
            let local = (rx * theta.cos(), ry * theta.sin());
            rotate_point_about_center(center, local, rotation_degrees)
        })
        .collect()
}

fn world_pad_outline(
    pad: &datum_gui_protocol::PadPrimitive,
    inset_nm: f32,
    reference_projection: &Projection,
) -> Vec<PointNm> {
    let (width_nm, height_nm) = pad_dimensions_nm(pad);
    let center = (pad.center.x as f32, pad.center.y as f32);
    let width_nm = (width_nm - inset_nm * 2.0).max(1.0);
    let height_nm = (height_nm - inset_nm * 2.0).max(1.0);
    let points = match pad.shape_kind.as_str() {
        "circle" | "oval" => ellipse_points(center, width_nm, height_nm, pad.rotation_degrees, 64),
        _ => {
            let radius_nm =
                pad_corner_radius_nm(pad, width_nm, height_nm, reference_projection, inset_nm);
            rounded_rect_points(center, width_nm, height_nm, pad.rotation_degrees, radius_nm)
        }
    };
    points
    .into_iter()
    .map(|(x, y)| PointNm {
        x: x.round() as i64,
        y: y.round() as i64,
    })
    .collect()
}

fn projected_pad_outline(
    pad: &datum_gui_protocol::PadPrimitive,
    projection: &Projection,
    inset_px: f32,
) -> Vec<(f32, f32)> {
    let (width_nm, height_nm) = pad_dimensions_nm(pad);
    let center = projection.project_point(pad.center);
    let width_px = (projection.world_length_to_px(width_nm.round() as i64) - inset_px * 2.0).max(1.0);
    let height_px =
        (projection.world_length_to_px(height_nm.round() as i64) - inset_px * 2.0).max(1.0);
    match pad.shape_kind.as_str() {
        "circle" | "oval" => ellipse_points(center, width_px, height_px, pad.rotation_degrees, 48),
        _ => {
            let min_dim_px = width_px.min(height_px);
            let radius_px = match pad.shape_kind.as_str() {
                "roundrect" => {
                    let ratio = (pad.roundrect_rratio_ppm as f32 / 1_000_000.0).clamp(0.0, 0.5);
                    (min_dim_px * ratio).max(1.0)
                }
                _ => 0.0,
            };
            rounded_rect_points(center, width_px, height_px, pad.rotation_degrees, radius_px)
        }
    }
}

#[allow(dead_code)]
fn component_should_draw_package_body(scene: &BoardReviewSceneV1, component_uuid: &str) -> bool {
    let pads: Vec<_> = scene
        .pads
        .iter()
        .filter(|pad| pad.component_uuid == component_uuid)
        .collect();
    let has_closed_outline = scene.component_graphics.iter().any(|graphic| {
        graphic.component_uuid == component_uuid
            && graphic.render_role == "component_mechanical"
            && graphic.closed
            && graphic.path.len() >= 4
    });
    let compact_inferred_body = compact_component_body_bounds(&pads).is_some();
    !pads.is_empty()
        && !pads.iter().any(|pad| pad.drill_nm.unwrap_or(0) > 0)
        && (has_closed_outline || compact_inferred_body)
}

fn compact_component_body_bounds(
    pads: &[&datum_gui_protocol::PadPrimitive],
) -> Option<datum_gui_protocol::RectNm> {
    inferred_component_body_bounds(pads).filter(|body| {
        let width = body.max_x - body.min_x;
        let height = body.max_y - body.min_y;
        width > 0 && height > 0 && width <= 4_500_000 && height <= 4_500_000
    })
}

fn inferred_component_body_bounds(
    pads: &[&datum_gui_protocol::PadPrimitive],
) -> Option<datum_gui_protocol::RectNm> {
    if pads.is_empty() {
        return None;
    }
    let pad_union = pads.iter().fold(
        datum_gui_protocol::RectNm {
            min_x: i64::MAX,
            min_y: i64::MAX,
            max_x: i64::MIN,
            max_y: i64::MIN,
        },
        |mut acc, pad| {
            acc.min_x = acc.min_x.min(pad.bounds.min_x);
            acc.min_y = acc.min_y.min(pad.bounds.min_y);
            acc.max_x = acc.max_x.max(pad.bounds.max_x);
            acc.max_y = acc.max_y.max(pad.bounds.max_y);
            acc
        },
    );
    let spread_x = (pad_union.max_x - pad_union.min_x) as f32;
    let spread_y = (pad_union.max_y - pad_union.min_y) as f32;
    let body = if spread_x >= spread_y {
        datum_gui_protocol::RectNm {
            min_x: (pad_union.min_x as f32 + spread_x * 0.28).round() as i64,
            min_y: (pad_union.min_y as f32 + spread_y * 0.06).round() as i64,
            max_x: (pad_union.max_x as f32 - spread_x * 0.28).round() as i64,
            max_y: (pad_union.max_y as f32 - spread_y * 0.06).round() as i64,
        }
    } else {
        datum_gui_protocol::RectNm {
            min_x: (pad_union.min_x as f32 + spread_x * 0.08).round() as i64,
            min_y: (pad_union.min_y as f32 + spread_y * 0.28).round() as i64,
            max_x: (pad_union.max_x as f32 - spread_x * 0.08).round() as i64,
            max_y: (pad_union.max_y as f32 - spread_y * 0.28).round() as i64,
        }
    };
    (body.max_x > body.min_x && body.max_y > body.min_y).then_some(body)
}

fn closed_component_body_graphic<'a>(
    scene: &'a BoardReviewSceneV1,
    component_uuid: &str,
) -> Option<&'a ComponentGraphicPrimitive> {
    scene
        .component_graphics
        .iter()
        .filter(|graphic| {
            graphic.component_uuid == component_uuid
                && graphic.render_role == "component_mechanical"
                && graphic.closed
                && graphic.path.len() >= 3
        })
        .max_by_key(|graphic| {
            let min_x = graphic.path.iter().map(|p| p.x).min().unwrap_or(0);
            let max_x = graphic.path.iter().map(|p| p.x).max().unwrap_or(0);
            let min_y = graphic.path.iter().map(|p| p.y).min().unwrap_or(0);
            let max_y = graphic.path.iter().map(|p| p.y).max().unwrap_or(0);
            (max_x - min_x) * (max_y - min_y)
        })
}

fn selected_component_body_graphic_id<'a>(
    scene: &'a BoardReviewSceneV1,
    component_uuid: &str,
) -> Option<&'a str> {
    closed_component_body_graphic(scene, component_uuid).map(|graphic| graphic.graphic_id.as_str())
}

#[allow(dead_code)]
fn push_inferred_package_body_from_pads(
    out: &mut Vec<Quad>,
    _component: &datum_gui_protocol::ComponentBounds,
    pads: &[&datum_gui_protocol::PadPrimitive],
    projection: &Projection,
    selected: bool,
    related: bool,
    dimmed: bool,
) {
    if pads.is_empty() {
        return;
    }
    let Some(body_nm) = inferred_component_body_bounds(pads) else {
        return;
    };
    let body = project_rect(body_nm, projection);
    if body.width <= 2.0 || body.height <= 2.0 {
        return;
    }
    let fill = dim_structural_color(
        if selected {
            [0.30, 0.32, 0.34]
        } else if related {
            [0.25, 0.27, 0.29]
        } else {
            [0.18, 0.19, 0.21]
        },
        dimmed,
    );
    let accent = dim_structural_color(
        if selected {
            AUTHOR_SELECTED
        } else if related {
            PAD_COPPER_RELATED
        } else {
            [0.56, 0.58, 0.62]
        },
        dimmed,
    );
    out.push(Quad::from_rect(body, fill));
    push_rect_border(out, body, accent, 1.0);
    if pads.len() >= 4 {
        let marker = RectPx {
            x: body.x + 4.0,
            y: body.y + 4.0,
            width: 4.0,
            height: 4.0,
        };
        push_projected_ellipse(
            out,
            marker,
            dim_structural_color(
                if selected || related {
                    PAD_COPPER_RELATED
                } else {
                    [0.96, 0.74, 0.44]
                },
                dimmed,
            ),
            14,
        );
    }
    let body_outline = inset_rect(body, 1.0, 1.0, 1.0, 1.0);
    push_rect_border(
        out,
        body_outline,
        dim_structural_color([0.47, 0.52, 0.57], dimmed),
        1.0,
    );
}

#[allow(dead_code)]
fn push_inferred_package_body_from_pads_world(
    out: &mut Vec<Quad>,
    component: &datum_gui_protocol::ComponentBounds,
    pads: &[&datum_gui_protocol::PadPrimitive],
    selected: bool,
    related: bool,
    dimmed: bool,
    reference_projection: &Projection,
) {
    if pads.is_empty() {
        return;
    }
    let Some((center, width, height, rotation_degrees)) =
        inferred_component_body_geometry(pads, component.rotation_degrees)
    else {
        return;
    };
    let body_polygon: Vec<PointNm> = rounded_rect_points(
        center,
        width,
        height,
        rotation_degrees,
        0.0,
    )
    .into_iter()
    .map(|(x, y)| PointNm {
        x: x.round() as i64,
        y: y.round() as i64,
    })
    .collect();
    let fill = dim_structural_color(
        if selected {
            [0.30, 0.32, 0.34]
        } else if related {
            [0.25, 0.27, 0.29]
        } else {
            [0.18, 0.19, 0.21]
        },
        dimmed,
    );
    let accent = dim_structural_color(
        if selected {
            AUTHOR_SELECTED
        } else if related {
            PAD_COPPER_RELATED
        } else {
            [0.56, 0.58, 0.62]
        },
        dimmed,
    );
    push_world_polygon_fill(out, &body_polygon, fill);
    let border_stroke = world_stroke_nm(if selected { 2.5 } else { 1.0 }, reference_projection);
    push_world_polyline_segments(out, &close_path(&body_polygon), border_stroke, accent);
    let inset = border_stroke.max(1.0) * 2.0;
    let inner_width = (width - inset * 2.0).max(1.0);
    let inner_height = (height - inset * 2.0).max(1.0);
    if inner_width > 1.0 && inner_height > 1.0 {
        let inner_polygon: Vec<PointNm> = rounded_rect_points(
            center,
            inner_width,
            inner_height,
            rotation_degrees,
            0.0,
        )
        .into_iter()
        .map(|(x, y)| PointNm {
            x: x.round() as i64,
            y: y.round() as i64,
        })
        .collect();
        push_world_polyline_segments(
            out,
            &close_path(&inner_polygon),
            border_stroke,
            dim_structural_color([0.47, 0.52, 0.57], dimmed),
        );
    }
}

#[allow(dead_code)]
fn push_selected_component_body_from_graphic_world(
    out: &mut Vec<Quad>,
    graphic: &ComponentGraphicPrimitive,
    selected: bool,
    related: bool,
    dimmed: bool,
    reference_projection: &Projection,
) {
    let fill = dim_structural_color(
        if selected {
            [0.30, 0.32, 0.34]
        } else if related {
            [0.25, 0.27, 0.29]
        } else {
            [0.18, 0.19, 0.21]
        },
        dimmed,
    );
    let accent = dim_structural_color(
        if selected {
            AUTHOR_SELECTED
        } else if related {
            PAD_COPPER_RELATED
        } else {
            [0.56, 0.58, 0.62]
        },
        dimmed,
    );
    push_world_convex_polygon_fill(out, &graphic.path, fill);
    let border_stroke = world_stroke_nm(if selected { 2.5 } else { 1.0 }, reference_projection);
    push_world_polyline_segments(out, &close_path(&graphic.path), border_stroke, accent);
}

#[allow(dead_code)]
fn push_world_convex_polygon_fill(out: &mut Vec<Quad>, polygon: &[PointNm], color: [f32; 3]) {
    if polygon.len() < 3 {
        return;
    }
    let center = (
        polygon.iter().map(|p| p.x as f32).sum::<f32>() / polygon.len() as f32,
        polygon.iter().map(|p| p.y as f32).sum::<f32>() / polygon.len() as f32,
    );
    for edge in polygon.windows(2) {
        push_world_triangle(
            out,
            center,
            (edge[0].x as f32, edge[0].y as f32),
            (edge[1].x as f32, edge[1].y as f32),
            color,
        );
    }
    push_world_triangle(
        out,
        center,
        (
            polygon[polygon.len() - 1].x as f32,
            polygon[polygon.len() - 1].y as f32,
        ),
        (polygon[0].x as f32, polygon[0].y as f32),
        color,
    );
}

#[allow(dead_code)]
fn inferred_component_body_geometry(
    pads: &[&datum_gui_protocol::PadPrimitive],
    fallback_rotation_degrees: f32,
) -> Option<((f32, f32), f32, f32, f32)> {
    let body = inferred_component_body_bounds(pads)?;
    let center = (
        ((body.min_x + body.max_x) as f32) * 0.5,
        ((body.min_y + body.max_y) as f32) * 0.5,
    );
    let rotation_degrees = fallback_rotation_degrees;

    let local_points: Vec<(f32, f32)> = pads
        .iter()
        .flat_map(|pad| {
            let corners = [
                (pad.bounds.min_x as f32, pad.bounds.min_y as f32),
                (pad.bounds.max_x as f32, pad.bounds.min_y as f32),
                (pad.bounds.max_x as f32, pad.bounds.max_y as f32),
                (pad.bounds.min_x as f32, pad.bounds.max_y as f32),
            ];
            corners.into_iter().map(move |point| {
                let dx = point.0 - center.0;
                let dy = point.1 - center.1;
                let rad = rotation_degrees.to_radians();
                let cos = rad.cos();
                let sin = rad.sin();
                // Convert world-space points back into the component's local frame.
                // Using the forward rotation here swaps quarter-turn parts.
                (dx * cos + dy * sin, -dx * sin + dy * cos)
            })
        })
        .collect();

    if local_points.is_empty() {
        return None;
    }

    let mut min_x = f32::MAX;
    let mut min_y = f32::MAX;
    let mut max_x = f32::MIN;
    let mut max_y = f32::MIN;
    for (x, y) in local_points {
        min_x = min_x.min(x);
        min_y = min_y.min(y);
        max_x = max_x.max(x);
        max_y = max_y.max(y);
    }
    let spread_x = max_x - min_x;
    let spread_y = max_y - min_y;
    let (body_min_x, body_max_x, body_min_y, body_max_y) = if spread_x >= spread_y {
        (
            min_x + spread_x * 0.28,
            max_x - spread_x * 0.28,
            min_y + spread_y * 0.06,
            max_y - spread_y * 0.06,
        )
    } else {
        (
            min_x + spread_x * 0.08,
            max_x - spread_x * 0.08,
            min_y + spread_y * 0.28,
            max_y - spread_y * 0.28,
        )
    };
    let width = (body_max_x - body_min_x).max(1.0);
    let height = (body_max_y - body_min_y).max(1.0);
    Some((center, width, height, rotation_degrees))
}

fn push_component_text_primitive(
    text_runs: &mut Vec<TextRun>,
    text: &ComponentTextPrimitive,
    scene_layers: &[datum_gui_protocol::SceneLayer],
    projection: &Projection,
    clip_bounds: RectPx,
    selected: bool,
    related: bool,
    dimmed: bool,
) {
    let (x, y) = project_point(text.position, projection);
    let color = component_text_color(text, scene_layers, selected, related, dimmed);
    let size = footprint_text_size_px(text.height_nm, projection);
    draw_text_clipped(
        &truncate_text(&text.text.to_uppercase(), 10),
        x - size * 1.2,
        y - size * 0.45,
        size,
        color,
        TextFace::Mono,
        clip_bounds,
        text_runs,
    );
}

fn component_text_color(
    text: &ComponentTextPrimitive,
    scene_layers: &[datum_gui_protocol::SceneLayer],
    selected: bool,
    related: bool,
    dimmed: bool,
) -> [f32; 3] {
    dim_context_color(
        match text.render_role.as_str() {
            "component_mechanical" => {
                if selected {
                    selected_mechanical_color(COMPONENT_MECHANICAL)
                } else if related {
                    COMPONENT_MECHANICAL_RELATED
                } else {
                    COMPONENT_MECHANICAL
                }
            }
            _ => {
                if selected {
                    selected_silk_color(
                        resolve_layer_appearance_with_scene(text.layer_id.as_deref(), scene_layers)
                            .silkscreen,
                    )
                } else if related {
                    COMPONENT_SILK_RELATED
                } else {
                    resolve_layer_appearance_with_scene(text.layer_id.as_deref(), scene_layers)
                        .silkscreen
                }
            }
        },
        dimmed,
    )
}

fn component_has_detail_text(scene: &BoardReviewSceneV1, component_uuid: &str) -> bool {
    scene
        .component_texts
        .iter()
        .any(|text| text.component_uuid == component_uuid)
        || scene.component_graphics.iter().any(|graphic| {
            graphic.component_uuid == component_uuid
                && graphic.primitive_kind == "polygon"
                && graphic.width_nm.is_none()
        })
}

fn push_component_text_world(
    out: &mut Vec<Quad>,
    text_runs: &mut Vec<TextRun>,
    text: &ComponentTextPrimitive,
    scene_layers: &[datum_gui_protocol::SceneLayer],
    projection: &Projection,
    clip_bounds: RectPx,
    selected: bool,
    related: bool,
    dimmed: bool,
) {
    let normalized = text.text.to_uppercase();
    let board_text = BoardText {
        uuid: Uuid::nil(),
        text: normalized,
        position: Point {
            x: text.position.x,
            y: text.position.y,
        },
        rotation: text.rotation_degrees.round() as i32,
        layer: 0 as LayerId,
        height_nm: text.height_nm,
        stroke_width_nm: (text.height_nm / 10).clamp(80_000, 250_000),
    };
    let color = component_text_color(text, scene_layers, selected, related, dimmed);
    match render_silkscreen_text_strokes(&board_text) {
        Ok(strokes) if !strokes.is_empty() => {
            for stroke in strokes {
                let path = [
                    stroke_text_point_to_board_space(text.position, stroke.from),
                    stroke_text_point_to_board_space(text.position, stroke.to),
                ];
                let thickness_px = projection.world_length_to_px(stroke.width_nm).clamp(1.0, 6.0);
                push_polyline_segments(out, &path, projection, color, thickness_px);
            }
        }
        _ => push_component_text_primitive(
            text_runs,
            text,
            scene_layers,
            projection,
            clip_bounds,
            selected,
            related,
            dimmed,
        ),
    }
}

fn stroke_text_point_to_board_space(origin: PointNm, point: Point) -> PointNm {
    // The engine silkscreen stroke font is authored in a conventional
    // Cartesian Y-up frame. Datum's board/world render space is Y-down, so
    // reflected text strokes are needed before projection into the viewport.
    PointNm {
        x: point.x,
        y: origin.y * 2 - point.y,
    }
}

#[allow(dead_code)]
fn push_via_primitive(
    out: &mut Vec<Quad>,
    via: &datum_gui_protocol::ViaPrimitive,
    projection: &Projection,
    selected: bool,
    dimmed: bool,
) -> RectPx {
    let outer_size = world_length_to_px(via.diameter_nm, projection).clamp(7.0, 18.0);
    let (x, y) = project_point(via.position, projection);
    let rect = RectPx {
        x: x - outer_size * 0.5,
        y: y - outer_size * 0.5,
        width: outer_size,
        height: outer_size,
    };
    push_projected_ellipse(
        out,
        rect,
        dim_authored_color(
            if selected {
                AUTHOR_SELECTED
            } else {
                resolve_layer_appearance(Some(&via.start_layer_id)).pad_copper
            },
            dimmed,
        ),
        128,
    );
    let ring = outer_size * 0.14;
    let copper = inset_rect(rect, ring, ring, ring, ring);
    push_projected_ellipse(
        out,
        copper,
        dim_authored_color(
            if selected {
                [0.72, 0.86, 0.93]
            } else {
                resolve_layer_appearance(Some(&via.start_layer_id)).pad_copper
            },
            dimmed,
        ),
        128,
    );
    let drill_px =
        world_length_to_px(via.drill_nm, projection).clamp(3.2, (outer_size - ring * 2.0).max(3.2));
    let drill = RectPx {
        x: x - drill_px * 0.5,
        y: y - drill_px * 0.5,
        width: drill_px,
        height: drill_px,
    };
    push_projected_ellipse(out, drill, dim_structural_color([0.13, 0.14, 0.16], dimmed), 18);
    rect
}

fn push_via_primitive_world(
    out: &mut Vec<Quad>,
    via: &datum_gui_protocol::ViaPrimitive,
    copper_color: [f32; 3],
    selected: bool,
    dimmed: bool,
    _reference_projection: &Projection,
) {
    let half = via.diameter_nm as f32 * 0.5;
    let rect = datum_gui_protocol::RectNm {
        min_x: (via.position.x as f32 - half).round() as i64,
        min_y: (via.position.y as f32 - half).round() as i64,
        max_x: (via.position.x as f32 + half).round() as i64,
        max_y: (via.position.y as f32 + half).round() as i64,
    };
    push_world_ellipse_nm(
        out,
        rect,
        dim_authored_color(
            if selected {
                AUTHOR_SELECTED
            } else {
                copper_color
            },
            dimmed,
        ),
        128,
    );
    let ring = via.diameter_nm as f32 * 0.14;
    let copper = world_inset_rect(rect, ring);
    push_world_ellipse_nm(
        out,
        copper,
        dim_authored_color(
            if selected {
                [0.72, 0.86, 0.93]
            } else {
                copper_color
            },
            dimmed,
        ),
        128,
    );
    let drill_half = via.drill_nm as f32 * 0.5;
    push_world_ellipse_nm(
        out,
        datum_gui_protocol::RectNm {
            min_x: (via.position.x as f32 - drill_half).round() as i64,
            min_y: (via.position.y as f32 - drill_half).round() as i64,
            max_x: (via.position.x as f32 + drill_half).round() as i64,
            max_y: (via.position.y as f32 + drill_half).round() as i64,
        },
        dim_structural_color([0.13, 0.14, 0.16], dimmed),
        128,
    );
}

fn board_surface_color(role: BoardSurfaceRole) -> [f32; 3] {
    match role {
        BoardSurfaceRole::OuterField => BOARD_OUTER_FIELD,
        BoardSurfaceRole::InnerField => BOARD_INNER_FIELD,
        BoardSurfaceRole::GridMajor => BOARD_GRID_MAJOR,
        BoardSurfaceRole::GridMinor => BOARD_GRID_MINOR,
        BoardSurfaceRole::Edge => BOARD_EDGE,
    }
}

fn mix_color(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    let t = t.clamp(0.0, 1.0);
    [
        a[0] * (1.0 - t) + b[0] * t,
        a[1] * (1.0 - t) + b[1] * t,
        a[2] * (1.0 - t) + b[2] * t,
    ]
}

fn resolve_layer_family_with_scene(
    layer_id: Option<&str>,
    scene_layers: &[datum_gui_protocol::SceneLayer],
) -> LayerFamily {
    let Some(id) = layer_id else {
        return LayerFamily::Unknown;
    };
    // Look up the real layer name from the scene
    if let Some(layer) = scene_layers.iter().find(|l| l.layer_id == id) {
        return match layer.name.as_str() {
            "F.Cu" => LayerFamily::TopCopper,
            "B.Cu" => LayerFamily::BottomCopper,
            name if name.ends_with(".Cu") => LayerFamily::InnerCopper,
            _ => LayerFamily::Unknown,
        };
    }
    // Fallback
    match id {
        "L0" | "F.Cu" => LayerFamily::TopCopper,
        "L31" | "B.Cu" => LayerFamily::BottomCopper,
        name if name.ends_with(".Cu") => LayerFamily::InnerCopper,
        _ => LayerFamily::Unknown,
    }
}

fn resolve_layer_appearance_with_scene(
    layer_id: Option<&str>,
    scene_layers: &[datum_gui_protocol::SceneLayer],
) -> LayerAppearance {
    match resolve_layer_family_with_scene(layer_id, scene_layers) {
        LayerFamily::TopCopper => {
            let copper = [0.86, 0.55, 0.24];
            LayerAppearance {
                authored_track: copper,
                pad_copper: copper,
                pad_related: [1.00, 0.84, 0.56],
                zone_fill: copper,
                zone_outline: copper,
                proposal: [0.98, 0.71, 0.30],
                silkscreen: [0.93, 0.92, 0.82],
            }
        }
        LayerFamily::InnerCopper => {
            let copper = [0.67, 0.68, 0.30];
            LayerAppearance {
                authored_track: copper,
                pad_copper: copper,
                pad_related: [0.92, 0.86, 0.54],
                zone_fill: copper,
                zone_outline: copper,
                proposal: [0.84, 0.80, 0.40],
                silkscreen: [0.86, 0.89, 0.82],
            }
        }
        LayerFamily::BottomCopper => {
            let copper = [0.30, 0.76, 0.88];
            LayerAppearance {
                authored_track: copper,
                pad_copper: copper,
                pad_related: [0.71, 0.95, 1.00],
                zone_fill: copper,
                zone_outline: copper,
                proposal: [0.46, 0.88, 0.96],
                silkscreen: [0.78, 0.92, 0.98],
            }
        }
        LayerFamily::Unknown => LayerAppearance {
            authored_track: AUTHOR_BASE,
            pad_copper: PAD_COPPER,
            pad_related: PAD_COPPER_RELATED,
            zone_fill: [0.26, 0.12, 0.24],
            zone_outline: [0.57, 0.24, 0.53],
            proposal: PROPOSAL_BASE,
            silkscreen: COMPONENT_SILK,
        },
    }
}

fn proposal_layer_color(layer_id: Option<&str>) -> [f32; 3] {
    resolve_layer_appearance(layer_id).proposal
}


fn resolve_layer_appearance(layer_id: Option<&str>) -> LayerAppearance {
    resolve_layer_appearance_with_scene(layer_id, &[])
}

fn scene_layer_name<'a>(
    layer_id: &str,
    scene_layers: &'a [datum_gui_protocol::SceneLayer],
) -> Option<&'a str> {
    scene_layers
        .iter()
        .find(|layer| layer.layer_id == layer_id)
        .map(|layer| layer.name.as_str())
}

fn render_stage_for_layer(layer_id: &str, scene_layers: &[datum_gui_protocol::SceneLayer]) -> RenderStage {
    match scene_layer_name(layer_id, scene_layers).unwrap_or(layer_id) {
        "B.Cu" => RenderStage::BottomCopper,
        name if name.ends_with(".Cu") && name != "F.Cu" => RenderStage::InnerCopper,
        "F.Cu" => RenderStage::TopCopper,
        "B.Paste" => RenderStage::BottomPaste,
        "F.Paste" => RenderStage::TopPaste,
        "B.Mask" => RenderStage::BottomMask,
        "F.Mask" => RenderStage::TopMask,
        "B.SilkS" => RenderStage::BottomSilk,
        "F.SilkS" => RenderStage::TopSilk,
        "Edge.Cuts" => RenderStage::Edge,
        name if name.ends_with(".CrtYd") || name.ends_with(".Fab") => RenderStage::Mechanical,
        _ => RenderStage::Other,
    }
}

fn render_stage_priority(stage: RenderStage) -> u32 {
    match stage {
        RenderStage::BottomCopper => 0,
        RenderStage::InnerCopper => 1,
        RenderStage::TopCopper => 2,
        RenderStage::BottomMask => 3,
        RenderStage::TopMask => 4,
        RenderStage::BottomPaste => 5,
        RenderStage::TopPaste => 6,
        RenderStage::BottomSilk => 7,
        RenderStage::TopSilk => 8,
        RenderStage::Mechanical => 9,
        RenderStage::Edge => 10,
        RenderStage::Other => 11,
    }
}

fn scene_layer_stack_priority(layer_id: &str, scene_layers: &[datum_gui_protocol::SceneLayer]) -> u32 {
    render_stage_priority(render_stage_for_layer(layer_id, scene_layers))
}

fn graphic_render_stage(
    layer_id: Option<&str>,
    scene_layers: &[datum_gui_protocol::SceneLayer],
    default_stage: RenderStage,
) -> RenderStage {
    layer_id
        .map(|id| render_stage_for_layer(id, scene_layers))
        .unwrap_or(default_stage)
}

fn copper_pass_priority_for_layer(
    layer_id: &str,
    scene_layers: &[datum_gui_protocol::SceneLayer],
) -> Option<u32> {
    match render_stage_for_layer(layer_id, scene_layers) {
        RenderStage::BottomCopper => Some(0),
        RenderStage::InnerCopper => Some(1),
        RenderStage::TopCopper => Some(2),
        _ => None,
    }
}

fn mask_or_paste_layer_color(
    layer_id: &str,
    scene_layers: &[datum_gui_protocol::SceneLayer],
) -> [f32; 3] {
    match scene_layer_name(layer_id, scene_layers) {
        Some("F.Mask") => TOP_MASK_OPENING,
        Some("B.Mask") => BOTTOM_MASK_OPENING,
        Some("F.Paste") => TOP_PASTE_OPENING,
        Some("B.Paste") => BOTTOM_PASTE_OPENING,
        _ => resolve_layer_appearance_with_scene(Some(layer_id), scene_layers).pad_copper,
    }
}

fn footprint_text_size_px(height_nm: i64, projection: &Projection) -> f32 {
    world_length_to_px(height_nm, projection).max(1.0)
}

fn world_length_to_px(length_nm: i64, projection: &Projection) -> f32 {
    projection.world_length_to_px(length_nm)
}

fn component_silk_color(layer_id: Option<&str>) -> [f32; 3] {
    resolve_layer_appearance(layer_id).silkscreen
}

fn detail_tier(projection: &Projection) -> DetailTier {
    let px_per_mm = world_length_to_px(1_000_000, projection);
    if px_per_mm >= 18.0 {
        DetailTier::Fine
    } else if px_per_mm >= 8.0 {
        DetailTier::Normal
    } else {
        DetailTier::Coarse
    }
}

fn floor_multiple(value: i64, pitch: i64) -> i64 {
    value.div_euclid(pitch) * pitch
}

fn ceil_multiple(value: i64, pitch: i64) -> i64 {
    if value.rem_euclid(pitch) == 0 {
        value
    } else {
        value.div_euclid(pitch) * pitch + pitch
    }
}

fn push_points(
    out: &mut Vec<Quad>,
    points: &[PointNm],
    projection: &Projection,
    color: [f32; 3],
    size_px: f32,
) -> Vec<RectPx> {
    let mut rects = Vec::new();
    for point in points {
        rects.push(push_point_square(out, *point, projection, size_px, color));
    }
    rects
}

#[allow(dead_code)]
fn push_projected_round_rect(out: &mut Vec<Quad>, rect: RectPx, color: [f32; 3], radius_px: f32) {
    let radius = radius_px.min(rect.width * 0.5).min(rect.height * 0.5);
    if radius <= 0.75 {
        out.push(Quad::from_rect(rect, color));
        return;
    }
    let center = RectPx {
        x: rect.x + radius,
        y: rect.y,
        width: (rect.width - radius * 2.0).max(0.0),
        height: rect.height,
    };
    if center.width > 0.0 && center.height > 0.0 {
        out.push(Quad::from_rect(center, color));
    }
    let middle = RectPx {
        x: rect.x,
        y: rect.y + radius,
        width: rect.width,
        height: (rect.height - radius * 2.0).max(0.0),
    };
    if middle.width > 0.0 && middle.height > 0.0 {
        out.push(Quad::from_rect(middle, color));
    }
    let diameter = radius * 2.0;
    for (x, y) in [
        (rect.x, rect.y),
        (rect.x + rect.width - diameter, rect.y),
        (rect.x, rect.y + rect.height - diameter),
        (
            rect.x + rect.width - diameter,
            rect.y + rect.height - diameter,
        ),
    ] {
        push_projected_ellipse(
            out,
            RectPx {
                x,
                y,
                width: diameter,
                height: diameter,
            },
            color,
            48,
        );
    }
}

fn push_dashed_polyline_segments(
    out: &mut Vec<Quad>,
    path: &[PointNm],
    projection: &Projection,
    color: [f32; 3],
    thickness_px: f32,
    dash_px: f32,
    gap_px: f32,
) -> Vec<RectPx> {
    let mut rects = Vec::new();
    for segment in path.windows(2) {
        let a = project_point(segment[0], projection);
        let b = project_point(segment[1], projection);
        let dx = b.0 - a.0;
        let dy = b.1 - a.1;
        let len = (dx * dx + dy * dy).sqrt().max(1.0);
        let ux = dx / len;
        let uy = dy / len;
        let step = (dash_px + gap_px).max(1.0);
        let mut start = 0.0;
        while start < len {
            let end = (start + dash_px).min(len);
            if end > start {
                let start_point = (a.0 + ux * start, a.1 + uy * start);
                let end_point = (a.0 + ux * end, a.1 + uy * end);
                let seg_dx = end_point.0 - start_point.0;
                let seg_dy = end_point.1 - start_point.1;
                let seg_len = (seg_dx * seg_dx + seg_dy * seg_dy).sqrt().max(1.0);
                let nx = -seg_dy / seg_len * thickness_px * 0.5;
                let ny = seg_dx / seg_len * thickness_px * 0.5;
                let quad = [
                    (start_point.0 + nx, start_point.1 + ny),
                    (end_point.0 + nx, end_point.1 + ny),
                    (end_point.0 - nx, end_point.1 - ny),
                    (start_point.0 - nx, start_point.1 - ny),
                ];
                rects.push(bounds_from_projected_points(&quad));
                push_projected_quad(out, &quad, color);
            }
            start += step;
        }
    }
    rects
}

fn push_polyline_endcaps(
    out: &mut Vec<Quad>,
    path: &[PointNm],
    projection: &Projection,
    color: [f32; 3],
    thickness_px: f32,
    cap_length_px: f32,
) -> Vec<RectPx> {
    let mut rects = Vec::new();
    if path.len() < 2 {
        return rects;
    }

    let first_a = project_point(path[0], projection);
    let first_b = project_point(path[1], projection);
    if let Some(quad) = projected_cap_quad(first_a, first_b, thickness_px, cap_length_px) {
        rects.push(bounds_from_projected_points(&quad));
        push_projected_quad(out, &quad, color);
    }

    let last_a = project_point(path[path.len() - 1], projection);
    let last_b = project_point(path[path.len() - 2], projection);
    if let Some(quad) = projected_cap_quad(last_a, last_b, thickness_px, cap_length_px) {
        rects.push(bounds_from_projected_points(&quad));
        push_projected_quad(out, &quad, color);
    }

    rects
}

fn push_polyline_segments(
    out: &mut Vec<Quad>,
    path: &[PointNm],
    projection: &Projection,
    color: [f32; 3],
    thickness_px: f32,
) -> Vec<RectPx> {
    let mut rects = Vec::new();
    for segment in path.windows(2) {
        let a = project_point(segment[0], projection);
        let b = project_point(segment[1], projection);
        let dx = b.0 - a.0;
        let dy = b.1 - a.1;
        let len = (dx * dx + dy * dy).sqrt().max(1.0);
        let nx = -dy / len * thickness_px * 0.5;
        let ny = dx / len * thickness_px * 0.5;
        let quad = [
            (a.0 + nx, a.1 + ny),
            (b.0 + nx, b.1 + ny),
            (b.0 - nx, b.1 - ny),
            (a.0 - nx, a.1 - ny),
        ];
        let rect = bounds_from_projected_points(&quad);
        rects.push(rect);
        push_projected_quad(out, &quad, color);
    }
    rects
}

fn projected_cap_quad(
    start: (f32, f32),
    toward: (f32, f32),
    thickness_px: f32,
    cap_length_px: f32,
) -> Option<[(f32, f32); 4]> {
    let dx = toward.0 - start.0;
    let dy = toward.1 - start.1;
    let len = (dx * dx + dy * dy).sqrt();
    if len <= 0.01 {
        return None;
    }
    let ux = dx / len;
    let uy = dy / len;
    let end = (
        start.0 + ux * cap_length_px.min(len),
        start.1 + uy * cap_length_px.min(len),
    );
    let nx = -uy * thickness_px * 0.5;
    let ny = ux * thickness_px * 0.5;
    Some([
        (start.0 + nx, start.1 + ny),
        (end.0 + nx, end.1 + ny),
        (end.0 - nx, end.1 - ny),
        (start.0 - nx, start.1 - ny),
    ])
}

fn close_path(points: &[PointNm]) -> Vec<PointNm> {
    let mut out = points.to_vec();
    if let (Some(first), Some(last)) = (out.first().copied(), out.last().copied())
        && first != last
    {
        out.push(first);
    }
    out
}

#[allow(dead_code)]
fn push_world_rect(
    out: &mut Vec<Quad>,
    rect: datum_gui_protocol::RectNm,
    projection: &Projection,
    color: [f32; 3],
) -> RectPx {
    let (x0, y0) = project_point(
        PointNm {
            x: rect.min_x,
            y: rect.min_y,
        },
        projection,
    );
    let (x1, y1) = project_point(
        PointNm {
            x: rect.max_x,
            y: rect.max_y,
        },
        projection,
    );
    let px = RectPx {
        x: x0,
        y: y0,
        width: (x1 - x0).max(1.0),
        height: (y1 - y0).max(1.0),
    };
    out.push(Quad::from_rect(px, color));
    px
}

fn project_rect(rect: datum_gui_protocol::RectNm, projection: &Projection) -> RectPx {
    projection.project_rect(rect)
}

fn push_point_square(
    out: &mut Vec<Quad>,
    point: PointNm,
    projection: &Projection,
    size_px: f32,
    color: [f32; 3],
) -> RectPx {
    let (x, y) = project_point(point, projection);
    let rect = RectPx {
        x: x - size_px * 0.5,
        y: y - size_px * 0.5,
        width: size_px.max(1.0),
        height: size_px.max(1.0),
    };
    out.push(Quad::from_rect(rect, color));
    rect
}

fn project_point(point: PointNm, projection: &Projection) -> (f32, f32) {
    projection.project_point(point)
}

fn world_stroke_nm(thickness_px: f32, projection: &Projection) -> f32 {
    (thickness_px / projection.scale).max(1.0)
}

fn push_world_quad(out: &mut Vec<Quad>, quad: &[(f32, f32); 4], color: [f32; 3]) {
    out.push(Quad {
        points: *quad,
        color,
    });
}

fn push_world_triangle(
    out: &mut Vec<Quad>,
    a: (f32, f32),
    b: (f32, f32),
    c: (f32, f32),
    color: [f32; 3],
) {
    out.push(Quad {
        points: [a, b, c, c],
        color,
    });
}

fn push_convex_polygon_fill(out: &mut Vec<Quad>, polygon: &[(f32, f32)], color: [f32; 3]) {
    if polygon.len() < 3 {
        return;
    }
    let origin = polygon[0];
    for edge in polygon[1..].windows(2) {
        push_world_triangle(out, origin, edge[0], edge[1], color);
    }
}

fn push_world_rect_nm(out: &mut Vec<Quad>, rect: datum_gui_protocol::RectNm, color: [f32; 3]) {
    out.push(Quad {
        points: [
            (rect.min_x as f32, rect.min_y as f32),
            (rect.max_x as f32, rect.min_y as f32),
            (rect.max_x as f32, rect.max_y as f32),
            (rect.min_x as f32, rect.max_y as f32),
        ],
        color,
    });
}

#[allow(dead_code)]
fn push_world_rect_border_nm(
    out: &mut Vec<Quad>,
    rect: datum_gui_protocol::RectNm,
    color: [f32; 3],
    thickness_nm: f32,
) {
    let t = thickness_nm.max(1.0).round() as i64;
    let top = datum_gui_protocol::RectNm {
        min_x: rect.min_x,
        min_y: rect.min_y,
        max_x: rect.max_x,
        max_y: rect.min_y + t,
    };
    let bottom = datum_gui_protocol::RectNm {
        min_x: rect.min_x,
        min_y: rect.max_y - t,
        max_x: rect.max_x,
        max_y: rect.max_y,
    };
    let left = datum_gui_protocol::RectNm {
        min_x: rect.min_x,
        min_y: rect.min_y,
        max_x: rect.min_x + t,
        max_y: rect.max_y,
    };
    let right = datum_gui_protocol::RectNm {
        min_x: rect.max_x - t,
        min_y: rect.min_y,
        max_x: rect.max_x,
        max_y: rect.max_y,
    };
    for edge in [top, bottom, left, right] {
        if edge.max_x > edge.min_x && edge.max_y > edge.min_y {
            push_world_rect_nm(out, edge, color);
        }
    }
}

fn world_inset_rect(rect: datum_gui_protocol::RectNm, inset_nm: f32) -> datum_gui_protocol::RectNm {
    let inset = inset_nm.max(0.0).round() as i64;
    datum_gui_protocol::RectNm {
        min_x: rect.min_x + inset,
        min_y: rect.min_y + inset,
        max_x: rect.max_x - inset,
        max_y: rect.max_y - inset,
    }
}

fn push_world_ellipse_nm(
    out: &mut Vec<Quad>,
    rect: datum_gui_protocol::RectNm,
    color: [f32; 3],
    segments: usize,
) {
    let width = (rect.max_x - rect.min_x) as f32;
    let height = (rect.max_y - rect.min_y) as f32;
    if width <= 1.0 || height <= 1.0 || segments < 3 {
        return;
    }
    let cx = (rect.min_x + rect.max_x) as f32 * 0.5;
    let cy = (rect.min_y + rect.max_y) as f32 * 0.5;
    let rx = width * 0.5;
    let ry = height * 0.5;
    let step = std::f32::consts::TAU / segments as f32;
    let mut prev = (cx + rx, cy);
    for i in 1..=segments {
        let angle = step * i as f32;
        let next = (cx + rx * angle.cos(), cy + ry * angle.sin());
        push_world_triangle(out, (cx, cy), prev, next, color);
        prev = next;
    }
}

#[allow(dead_code)]
fn push_world_round_rect_nm(
    out: &mut Vec<Quad>,
    rect: datum_gui_protocol::RectNm,
    color: [f32; 3],
    radius_nm: f32,
) {
    let width = (rect.max_x - rect.min_x) as f32;
    let height = (rect.max_y - rect.min_y) as f32;
    let radius = radius_nm.min(width * 0.5).min(height * 0.5);
    if radius <= 1.0 {
        push_world_rect_nm(out, rect, color);
        return;
    }
    push_world_rect_nm(
        out,
        datum_gui_protocol::RectNm {
            min_x: (rect.min_x as f32 + radius).round() as i64,
            min_y: rect.min_y,
            max_x: (rect.max_x as f32 - radius).round() as i64,
            max_y: rect.max_y,
        },
        color,
    );
    push_world_rect_nm(
        out,
        datum_gui_protocol::RectNm {
            min_x: rect.min_x,
            min_y: (rect.min_y as f32 + radius).round() as i64,
            max_x: rect.max_x,
            max_y: (rect.max_y as f32 - radius).round() as i64,
        },
        color,
    );
    let diameter = (radius * 2.0).round() as i64;
    for (x, y) in [
        (rect.min_x, rect.min_y),
        (rect.max_x - diameter, rect.min_y),
        (rect.min_x, rect.max_y - diameter),
        (rect.max_x - diameter, rect.max_y - diameter),
    ] {
        push_world_ellipse_nm(
            out,
            datum_gui_protocol::RectNm {
                min_x: x,
                min_y: y,
                max_x: x + diameter,
                max_y: y + diameter,
            },
            color,
            48,
        );
    }
}

fn push_world_polyline_segments(
    out: &mut Vec<Quad>,
    path: &[PointNm],
    thickness_nm: f32,
    color: [f32; 3],
) {
    for segment in path.windows(2) {
        let a = (segment[0].x as f32, segment[0].y as f32);
        let b = (segment[1].x as f32, segment[1].y as f32);
        let dx = b.0 - a.0;
        let dy = b.1 - a.1;
        let len = (dx * dx + dy * dy).sqrt().max(1.0);
        let nx = -dy / len * thickness_nm * 0.5;
        let ny = dx / len * thickness_nm * 0.5;
        push_world_quad(
            out,
            &[
                (a.0 + nx, a.1 + ny),
                (b.0 + nx, b.1 + ny),
                (b.0 - nx, b.1 - ny),
                (a.0 - nx, a.1 - ny),
            ],
            color,
        );
    }
}

fn push_world_polyline_mitered(
    out: &mut Vec<Quad>,
    path: &[PointNm],
    thickness_nm: f32,
    color: [f32; 3],
) {
    let n = path.len();
    if n < 2 {
        return;
    }
    let h = thickness_nm * 0.5;
    let is_closed = path[0].x == path[n - 1].x && path[0].y == path[n - 1].y;
    let unit = |a: PointNm, b: PointNm| -> (f32, f32) {
        let dx = (b.x - a.x) as f32;
        let dy = (b.y - a.y) as f32;
        let l = (dx * dx + dy * dy).sqrt().max(1.0);
        (dx / l, dy / l)
    };
    let perp = |d: (f32, f32)| -> (f32, f32) { (-d.1, d.0) };
    let mut offsets: Vec<(f32, f32)> = Vec::with_capacity(n);
    for i in 0..n {
        let prev_idx = if i == 0 {
            if is_closed { Some(n - 2) } else { None }
        } else {
            Some(i - 1)
        };
        let next_idx = if i + 1 == n {
            if is_closed { Some(1) } else { None }
        } else {
            Some(i + 1)
        };
        let n_in = prev_idx.map(|p| perp(unit(path[p], path[i])));
        let n_out = next_idx.map(|q| perp(unit(path[i], path[q])));
        let o = match (n_in, n_out) {
            (Some(a), Some(b)) => {
                let dot = a.0 * b.0 + a.1 * b.1;
                let denom = (1.0 + dot).max(0.2);
                ((a.0 + b.0) * h / denom, (a.1 + b.1) * h / denom)
            }
            (Some(a), None) => (a.0 * h, a.1 * h),
            (None, Some(b)) => (b.0 * h, b.1 * h),
            _ => (0.0, 0.0),
        };
        offsets.push(o);
    }
    for i in 0..(n - 1) {
        let a = path[i];
        let b = path[i + 1];
        let (ax, ay) = (a.x as f32, a.y as f32);
        let (bx, by) = (b.x as f32, b.y as f32);
        let oa = offsets[i];
        let ob = offsets[i + 1];
        push_world_quad(
            out,
            &[
                (ax + oa.0, ay + oa.1),
                (bx + ob.0, by + ob.1),
                (bx - ob.0, by - ob.1),
                (ax - oa.0, ay - oa.1),
            ],
            color,
        );
    }
}

fn push_world_polyline_segments_capped(
    out: &mut Vec<Quad>,
    path: &[PointNm],
    thickness_nm: f32,
    color: [f32; 3],
) {
    let ext = thickness_nm * 0.5;
    for segment in path.windows(2) {
        let a = (segment[0].x as f32, segment[0].y as f32);
        let b = (segment[1].x as f32, segment[1].y as f32);
        let dx = b.0 - a.0;
        let dy = b.1 - a.1;
        let len = (dx * dx + dy * dy).sqrt().max(1.0);
        let ux = dx / len;
        let uy = dy / len;
        let nx = -uy * thickness_nm * 0.5;
        let ny = ux * thickness_nm * 0.5;
        let a_ext = (a.0 - ux * ext, a.1 - uy * ext);
        let b_ext = (b.0 + ux * ext, b.1 + uy * ext);
        push_world_quad(
            out,
            &[
                (a_ext.0 + nx, a_ext.1 + ny),
                (b_ext.0 + nx, b_ext.1 + ny),
                (b_ext.0 - nx, b_ext.1 - ny),
                (a_ext.0 - nx, a_ext.1 - ny),
            ],
            color,
        );
    }
}

fn push_world_dashed_polyline_segments(
    out: &mut Vec<Quad>,
    path: &[PointNm],
    thickness_nm: f32,
    dash_nm: f32,
    gap_nm: f32,
    color: [f32; 3],
) {
    for segment in path.windows(2) {
        let a = (segment[0].x as f32, segment[0].y as f32);
        let b = (segment[1].x as f32, segment[1].y as f32);
        let dx = b.0 - a.0;
        let dy = b.1 - a.1;
        let len = (dx * dx + dy * dy).sqrt().max(1.0);
        let ux = dx / len;
        let uy = dy / len;
        let step = (dash_nm + gap_nm).max(1.0);
        let mut start = 0.0;
        while start < len {
            let end = (start + dash_nm).min(len);
            let start_point = PointNm {
                x: (a.0 + ux * start).round() as i64,
                y: (a.1 + uy * start).round() as i64,
            };
            let end_point = PointNm {
                x: (a.0 + ux * end).round() as i64,
                y: (a.1 + uy * end).round() as i64,
            };
            push_world_polyline_segments_capped(
                out,
                &[start_point, end_point],
                thickness_nm,
                color,
            );
            start += step;
        }
    }
}

#[allow(dead_code)]
fn push_world_points(out: &mut Vec<Quad>, points: &[PointNm], size_nm: f32, color: [f32; 3]) {
    for point in points {
        let half = size_nm * 0.5;
        push_world_rect_nm(
            out,
            datum_gui_protocol::RectNm {
                min_x: (point.x as f32 - half).round() as i64,
                min_y: (point.y as f32 - half).round() as i64,
                max_x: (point.x as f32 + half).round() as i64,
                max_y: (point.y as f32 + half).round() as i64,
            },
            color,
        );
    }
}

fn push_world_polygon_fill(out: &mut Vec<Quad>, polygon: &[PointNm], color: [f32; 3]) {
    if polygon.len() < 3 {
        return;
    }
    let mut cleaned: Vec<PointNm> = Vec::with_capacity(polygon.len());
    for &point in polygon {
        if cleaned.last().is_some_and(|last| last.x == point.x && last.y == point.y) {
            continue;
        }
        cleaned.push(point);
    }
    if cleaned.len() >= 2
        && cleaned.first().is_some_and(|first| {
            cleaned
                .last()
                .is_some_and(|last| last.x == first.x && last.y == first.y)
        })
    {
        cleaned.pop();
    }
    if cleaned.len() < 3 {
        return;
    }
    push_world_polygon_fill_scanline(out, &cleaned, color);
}

fn push_projected_quad(out: &mut Vec<Quad>, quad: &[(f32, f32); 4], color: [f32; 3]) {
    out.push(Quad {
        points: *quad,
        color,
    });
}

#[allow(dead_code)]
fn push_projected_triangle(
    out: &mut Vec<Quad>,
    a: (f32, f32),
    b: (f32, f32),
    c: (f32, f32),
    color: [f32; 3],
) {
    out.push(Quad {
        points: [a, b, c, c],
        color,
    });
}

fn push_projected_polygon_fill(out: &mut Vec<Quad>, polygon: &[(f32, f32)], color: [f32; 3]) {
    if polygon.len() < 3 {
        return;
    }
    let mut cleaned: Vec<(f32, f32)> = Vec::with_capacity(polygon.len());
    for &point in polygon {
        if cleaned.last().is_some_and(|last| {
            (last.0 - point.0).abs() < 0.001 && (last.1 - point.1).abs() < 0.001
        }) {
            continue;
        }
        cleaned.push(point);
    }
    if cleaned.len() >= 2
        && cleaned.first().is_some_and(|first| {
            cleaned.last().is_some_and(|last| {
                (last.0 - first.0).abs() < 0.001 && (last.1 - first.1).abs() < 0.001
            })
        })
    {
        cleaned.pop();
    }
    if cleaned.len() < 3 {
        return;
    }
    push_projected_polygon_fill_scanline(out, &cleaned, color);
}

fn push_world_polygon_fill_scanline(out: &mut Vec<Quad>, polygon: &[PointNm], color: [f32; 3]) {
    const EPS: f64 = 1e-6;
    let mut ys: Vec<f64> = polygon.iter().map(|p| p.y as f64).collect();
    ys.sort_by(|a, b| a.total_cmp(b));
    ys.dedup_by(|a, b| (*a - *b).abs() <= EPS);
    if ys.len() < 2 {
        return;
    }

    for band in ys.windows(2) {
        let y0 = band[0];
        let y1 = band[1];
        if y1 - y0 <= EPS {
            continue;
        }
        let y_mid = (y0 + y1) * 0.5;
        let mut spans: Vec<(f64, f64, f64)> = Vec::new();
        for i in 0..polygon.len() {
            let a = polygon[i];
            let b = polygon[(i + 1) % polygon.len()];
            let ay = a.y as f64;
            let by = b.y as f64;
            if (ay - by).abs() <= EPS {
                continue;
            }
            let min_y = ay.min(by);
            let max_y = ay.max(by);
            if y_mid < min_y || y_mid >= max_y {
                continue;
            }
            let x_at = |y: f64| {
                let t = (y - ay) / (by - ay);
                a.x as f64 + (b.x as f64 - a.x as f64) * t
            };
            spans.push((x_at(y_mid), x_at(y0), x_at(y1)));
        }
        spans.sort_by(|a, b| a.0.total_cmp(&b.0));
        for pair in spans.chunks_exact(2) {
            let left = pair[0];
            let right = pair[1];
            push_world_quad(
                out,
                &[
                    (left.1 as f32, y0 as f32),
                    (right.1 as f32, y0 as f32),
                    (right.2 as f32, y1 as f32),
                    (left.2 as f32, y1 as f32),
                ],
                color,
            );
        }
    }
}

fn push_projected_polygon_fill_scanline(
    out: &mut Vec<Quad>,
    polygon: &[(f32, f32)],
    color: [f32; 3],
) {
    const EPS: f32 = 1e-4;
    let mut ys: Vec<f32> = polygon.iter().map(|p| p.1).collect();
    ys.sort_by(|a, b| a.total_cmp(b));
    ys.dedup_by(|a, b| (*a - *b).abs() <= EPS);
    if ys.len() < 2 {
        return;
    }

    for band in ys.windows(2) {
        let y0 = band[0];
        let y1 = band[1];
        if y1 - y0 <= EPS {
            continue;
        }
        let y_mid = (y0 + y1) * 0.5;
        let mut spans: Vec<(f32, f32, f32)> = Vec::new();
        for i in 0..polygon.len() {
            let a = polygon[i];
            let b = polygon[(i + 1) % polygon.len()];
            if (a.1 - b.1).abs() <= EPS {
                continue;
            }
            let min_y = a.1.min(b.1);
            let max_y = a.1.max(b.1);
            if y_mid < min_y || y_mid >= max_y {
                continue;
            }
            let x_at = |y: f32| {
                let t = (y - a.1) / (b.1 - a.1);
                a.0 + (b.0 - a.0) * t
            };
            spans.push((x_at(y_mid), x_at(y0), x_at(y1)));
        }
        spans.sort_by(|a, b| a.0.total_cmp(&b.0));
        for pair in spans.chunks_exact(2) {
            let left = pair[0];
            let right = pair[1];
            push_projected_quad(
                out,
                &[
                    (left.1, y0),
                    (right.1, y0),
                    (right.2, y1),
                    (left.2, y1),
                ],
                color,
            );
        }
    }
}

#[allow(dead_code)]
fn push_projected_ellipse(out: &mut Vec<Quad>, rect: RectPx, color: [f32; 3], segments: usize) {
    if rect.width <= 0.5 || rect.height <= 0.5 || segments < 3 {
        return;
    }
    let cx = rect.x + rect.width * 0.5;
    let cy = rect.y + rect.height * 0.5;
    let rx = rect.width * 0.5;
    let ry = rect.height * 0.5;
    let step = std::f32::consts::TAU / segments as f32;
    let mut prev = (cx + rx, cy);
    for i in 1..=segments {
        let angle = step * i as f32;
        let next = (cx + rx * angle.cos(), cy + ry * angle.sin());
        push_projected_triangle(out, (cx, cy), prev, next, color);
        prev = next;
    }
}

fn bounds_from_projected_points(points: &[(f32, f32); 4]) -> RectPx {
    let mut min_x = f32::MAX;
    let mut min_y = f32::MAX;
    let mut max_x = f32::MIN;
    let mut max_y = f32::MIN;
    for (x, y) in points {
        min_x = min_x.min(*x);
        min_y = min_y.min(*y);
        max_x = max_x.max(*x);
        max_y = max_y.max(*y);
    }
    RectPx {
        x: min_x,
        y: min_y,
        width: (max_x - min_x).max(1.0),
        height: (max_y - min_y).max(1.0),
    }
}

fn inset_rect(rect: RectPx, left: f32, top: f32, right: f32, bottom: f32) -> RectPx {
    RectPx {
        x: rect.x + left,
        y: rect.y + top,
        width: (rect.width - left - right).max(1.0),
        height: (rect.height - top - bottom).max(1.0),
    }
}

fn push_rect_border(out: &mut Vec<Quad>, rect: RectPx, color: [f32; 3], thickness: f32) {
    out.push(Quad::from_rect(
        RectPx {
            x: rect.x,
            y: rect.y,
            width: rect.width,
            height: thickness,
        },
        color,
    ));
    out.push(Quad::from_rect(
        RectPx {
            x: rect.x,
            y: rect.y + rect.height - thickness,
            width: rect.width,
            height: thickness,
        },
        color,
    ));
    out.push(Quad::from_rect(
        RectPx {
            x: rect.x,
            y: rect.y,
            width: thickness,
            height: rect.height,
        },
        color,
    ));
    out.push(Quad::from_rect(
        RectPx {
            x: rect.x + rect.width - thickness,
            y: rect.y,
            width: thickness,
            height: rect.height,
        },
        color,
    ));
}

fn push_section_divider(out: &mut Vec<Quad>, x: f32, y: f32, width: f32, color: [f32; 3]) {
    out.push(Quad::from_rect(
        RectPx {
            x,
            y,
            width,
            height: 1.0,
        },
        color,
    ));
}

fn push_boolean_row(x: f32, y: f32, label: &str, enabled: bool, text_runs: &mut Vec<TextRun>) {
    draw_text(label, x, y, 13.0, TEXT_SECONDARY, TextFace::Ui, text_runs);
    draw_text(
        if enabled { "ON" } else { "OFF" },
        x + 132.0,
        y,
        13.0,
        if enabled { TEXT_PRIMARY } else { TEXT_MUTED },
        TextFace::Ui,
        text_runs,
    );
}

fn push_key_value(
    x: f32,
    y: f32,
    key: &str,
    value: &str,
    text_runs: &mut Vec<TextRun>,
    value_face: TextFace,
) {
    draw_text(key, x, y, 11.5, TEXT_MUTED, TextFace::Ui, text_runs);
    draw_text(
        value,
        x + 74.0,
        y,
        12.5,
        TEXT_PANEL_VALUE,
        value_face,
        text_runs,
    );
}

fn workspace_tool_label(tool: WorkspaceTool) -> &'static str {
    match tool {
        WorkspaceTool::Select => "SELECT",
    }
}

fn truncate_text(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        return text.to_string();
    }
    if max_chars <= 3 {
        return text.chars().take(max_chars).collect();
    }
    let keep = max_chars - 3;
    let front = keep / 2;
    let back = keep - front;
    let head: String = text.chars().take(front).collect();
    let tail: String = text
        .chars()
        .rev()
        .take(back)
        .collect::<String>()
        .chars()
        .rev()
        .collect();
    format!("{head}...{tail}")
}

fn text_buffer_key(run: &TextRun, width: u32, height: u32) -> TextBufferKey {
    TextBufferKey {
        text: run.text.clone(),
        size_bits: run.size.to_bits(),
        face: run.face,
        width_px: width,
        height_px: height,
    }
}

fn text_attrs(face: TextFace) -> Attrs<'static> {
    match face {
        TextFace::Ui => Attrs::new().family(Family::SansSerif),
        TextFace::Mono => Attrs::new().family(Family::Monospace),
    }
}

fn text_color(color: [f32; 3]) -> Color {
    Color::rgb(
        (color[0].clamp(0.0, 1.0) * 255.0).round() as u8,
        (color[1].clamp(0.0, 1.0) * 255.0).round() as u8,
        (color[2].clamp(0.0, 1.0) * 255.0).round() as u8,
    )
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    pos: [f32; 2],
    color: [f32; 3],
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
struct ScreenUniform {
    resolution: [f32; 2],
    _pad: [f32; 2],
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
struct SceneUniform {
    resolution: [f32; 4],
    viewport_origin: [f32; 4],
    viewport_size: [f32; 4],
    camera_center_scale: [f32; 4],
}

impl Vertex {
    fn layout<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: 8,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

fn quad_to_vertices(out: &mut Vec<Vertex>, quad: Quad) {
    let [a, b, c, d] = quad.points;
    out.extend_from_slice(&[
        Vertex {
            pos: [a.0, a.1],
            color: quad.color,
        },
        Vertex {
            pos: [b.0, b.1],
            color: quad.color,
        },
        Vertex {
            pos: [c.0, c.1],
            color: quad.color,
        },
        Vertex {
            pos: [a.0, a.1],
            color: quad.color,
        },
        Vertex {
            pos: [c.0, c.1],
            color: quad.color,
        },
        Vertex {
            pos: [d.0, d.1],
            color: quad.color,
        },
    ]);
}

fn quads_to_vertices(quads: &[Quad]) -> Vec<Vertex> {
    let mut out = Vec::with_capacity(quads.len() * 6);
    for quad in quads {
        quad_to_vertices(&mut out, *quad);
    }
    out
}

pub struct Renderer {
    pipeline: wgpu::RenderPipeline,
    world_pipeline: wgpu::RenderPipeline,
    uniform_bind_group: wgpu::BindGroup,
    uniform_buffer: wgpu::Buffer,
    scene_bind_group: wgpu::BindGroup,
    scene_uniform_buffer: wgpu::Buffer,
    font_system: FontSystem,
    swash_cache: SwashCache,
    viewport: Viewport,
    atlas: TextAtlas,
    text_renderer: TextRenderer,
    text_buffer_cache: Vec<CachedTextBuffer>,
    panel_vertex_buffer: Option<wgpu::Buffer>,
    panel_vertex_capacity: usize,
    viewport_underlay_vertex_buffer: Option<wgpu::Buffer>,
    viewport_underlay_vertex_capacity: usize,
    viewport_overlay_vertex_buffer: Option<wgpu::Buffer>,
    viewport_overlay_vertex_capacity: usize,
    world_vertex_buffer: Option<wgpu::Buffer>,
    world_vertex_capacity: usize,
    world_vertex_source_ptr: usize,
    world_vertex_source_len: usize,
    msaa_view: Option<wgpu::TextureView>,
    msaa_size: (u32, u32),
    msaa_format: wgpu::TextureFormat,
    msaa_samples: u32,
}

impl Renderer {
    fn upload_vertices(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        buffer: &mut Option<wgpu::Buffer>,
        capacity: &mut usize,
        label: &str,
        vertices: &[Vertex],
    ) {
        let bytes = bytemuck::cast_slice(vertices);
        if buffer.is_none() || *capacity < bytes.len() {
            *buffer = Some(
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some(label),
                    contents: bytes,
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                }),
            );
            *capacity = bytes.len();
            return;
        }
        if let Some(buffer) = buffer.as_ref() {
            queue.write_buffer(buffer, 0, bytes);
        }
    }

    fn sync_world_vertices(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        vertices: &[Vertex],
    ) {
        Self::upload_vertices(
            device,
            queue,
            &mut self.world_vertex_buffer,
            &mut self.world_vertex_capacity,
            "datum-gui-render-world-vertex-buffer",
            vertices,
        );
        self.world_vertex_source_ptr = vertices.as_ptr() as usize;
        self.world_vertex_source_len = vertices.len();
    }

    fn cached_text_buffer_indices(
        &mut self,
        text_runs: &[TextRun],
        width: u32,
        height: u32,
    ) -> Vec<usize> {
        let mut indices = Vec::with_capacity(text_runs.len());
        for run in text_runs {
            let key = text_buffer_key(run, width, height);
            if let Some(index) = self
                .text_buffer_cache
                .iter()
                .position(|entry| entry.key == key)
            {
                indices.push(index);
                continue;
            }
            let mut buffer = Buffer::new(
                &mut self.font_system,
                Metrics::new(run.size, run.size * 1.22),
            );
            buffer.set_size(
                &mut self.font_system,
                Some(width as f32),
                Some(height as f32),
            );
            let attrs = text_attrs(run.face);
            buffer.set_text(
                &mut self.font_system,
                &run.text,
                &attrs,
                Shaping::Advanced,
                None,
            );
            buffer.shape_until_scroll(&mut self.font_system, false);
            self.text_buffer_cache
                .push(CachedTextBuffer { key, buffer });
            indices.push(self.text_buffer_cache.len() - 1);
        }
        indices
    }

    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        msaa_samples: u32,
    ) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("datum-gui-render-shader"),
            source: wgpu::ShaderSource::Wgsl(
                r#"
struct ScreenUniform {
    resolution: vec2<f32>,
    _pad: vec2<f32>,
};

@group(0) @binding(0)
var<uniform> screen: ScreenUniform;

struct VsIn {
    @location(0) pos: vec2<f32>,
    @location(1) color: vec3<f32>,
};

struct VsOut {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec3<f32>,
};

@vertex
fn vs_main(in: VsIn) -> VsOut {
    var out: VsOut;
    let clip = vec2<f32>(
        (in.pos.x / screen.resolution.x) * 2.0 - 1.0,
        1.0 - (in.pos.y / screen.resolution.y) * 2.0
    );
    out.position = vec4<f32>(clip, 0.0, 1.0);
    out.color = in.color;
    return out;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}
"#
                .into(),
            ),
        });
        let world_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("datum-gui-render-world-shader"),
            source: wgpu::ShaderSource::Wgsl(
                r#"
struct SceneUniform {
    resolution: vec4<f32>,
    viewport_origin: vec4<f32>,
    viewport_size: vec4<f32>,
    camera_center_scale: vec4<f32>,
};

@group(0) @binding(0)
var<uniform> scene: SceneUniform;

struct VsIn {
    @location(0) pos: vec2<f32>,
    @location(1) color: vec3<f32>,
};

struct VsOut {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec3<f32>,
};

@vertex
fn vs_main(in: VsIn) -> VsOut {
    var out: VsOut;
    let screen = vec2<f32>(
        scene.viewport_origin.x + scene.viewport_size.x * 0.5 + (in.pos.x - scene.camera_center_scale.x) * scene.camera_center_scale.z,
        scene.viewport_origin.y + scene.viewport_size.y * 0.5 + (in.pos.y - scene.camera_center_scale.y) * scene.camera_center_scale.z
    );
    let clip = vec2<f32>(
        (screen.x / scene.resolution.x) * 2.0 - 1.0,
        1.0 - (screen.y / scene.resolution.y) * 2.0
    );
    out.position = vec4<f32>(clip, 0.0, 1.0);
    out.color = in.color;
    return out;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}
"#
                .into(),
            ),
        });
        let uniform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("datum-gui-render-uniform-bgl"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });
        let scene_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("datum-gui-render-scene-bgl"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });
        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("datum-gui-render-uniform-buffer"),
            contents: bytemuck::bytes_of(&ScreenUniform {
                resolution: [1.0, 1.0],
                _pad: [0.0, 0.0],
            }),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("datum-gui-render-uniform-bg"),
            layout: &uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });
        let scene_uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("datum-gui-render-scene-uniform-buffer"),
            contents: bytemuck::bytes_of(&SceneUniform {
                resolution: [1.0, 1.0, 0.0, 0.0],
                viewport_origin: [0.0, 0.0, 0.0, 0.0],
                viewport_size: [1.0, 1.0, 0.0, 0.0],
                camera_center_scale: [0.0, 0.0, 1.0, 0.0],
            }),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let scene_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("datum-gui-render-scene-bg"),
            layout: &scene_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: scene_uniform_buffer.as_entire_binding(),
            }],
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("datum-gui-render-pipeline-layout"),
            bind_group_layouts: &[&uniform_bind_group_layout],
            immediate_size: 0,
        });
        let world_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("datum-gui-render-world-pipeline-layout"),
                bind_group_layouts: &[&scene_bind_group_layout],
                immediate_size: 0,
            });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("datum-gui-render-pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::layout()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: msaa_samples,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview_mask: None,
            cache: None,
        });
        let world_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("datum-gui-render-world-pipeline"),
            layout: Some(&world_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &world_shader,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::layout()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &world_shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: msaa_samples,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview_mask: None,
            cache: None,
        });
        let font_system = FontSystem::new();
        let swash_cache = SwashCache::new();
        let cache = Cache::new(device);
        let viewport = Viewport::new(device, &cache);
        let mut atlas = TextAtlas::new(device, queue, &cache, format);
        let text_renderer =
            TextRenderer::new(
                &mut atlas,
                device,
                wgpu::MultisampleState {
                    count: msaa_samples,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                None,
            );
        Self {
            pipeline,
            world_pipeline,
            uniform_bind_group,
            uniform_buffer,
            scene_bind_group,
            scene_uniform_buffer,
            font_system,
            swash_cache,
            viewport,
            atlas,
            text_renderer,
            text_buffer_cache: Vec::new(),
            panel_vertex_buffer: None,
            panel_vertex_capacity: 0,
            viewport_underlay_vertex_buffer: None,
            viewport_underlay_vertex_capacity: 0,
            viewport_overlay_vertex_buffer: None,
            viewport_overlay_vertex_capacity: 0,
            world_vertex_buffer: None,
            world_vertex_capacity: 0,
            world_vertex_source_ptr: 0,
            world_vertex_source_len: 0,
            msaa_view: None,
            msaa_size: (0, 0),
            msaa_format: format,
            msaa_samples,
        }
    }

    fn ensure_msaa(&mut self, device: &wgpu::Device, width: u32, height: u32) -> &wgpu::TextureView {
        if self.msaa_size != (width, height) || self.msaa_view.is_none() {
            let texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("datum-gui-render-msaa"),
                size: wgpu::Extent3d {
                    width: width.max(1),
                    height: height.max(1),
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: self.msaa_samples,
                dimension: wgpu::TextureDimension::D2,
                format: self.msaa_format,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            });
            self.msaa_view = Some(texture.create_view(&wgpu::TextureViewDescriptor::default()));
            self.msaa_size = (width, height);
        }
        self.msaa_view.as_ref().unwrap()
    }

    pub fn render(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        target: &wgpu::TextureView,
        prepared: &PreparedScene,
        retained: &RetainedScene,
        width: u32,
        height: u32,
    ) -> anyhow::Result<()> {
        let panel_vertices = prepared.panel_vertices();
        let viewport_underlay_vertices = prepared.viewport_underlay_vertices();
        let viewport_overlay_vertices = prepared.viewport_overlay_vertices();
        let world_vertices = retained.world_vertices();
        let board_field = inset_rect(prepared.scene_viewport, 10.0, 10.0, 10.0, 10.0);
        let projection = Projection::new(board_field, &prepared.scene_bounds, prepared.camera);
        queue.write_buffer(
            &self.uniform_buffer,
            0,
            bytemuck::bytes_of(&ScreenUniform {
                resolution: [width as f32, height as f32],
                _pad: [0.0, 0.0],
            }),
        );
        queue.write_buffer(
            &self.scene_uniform_buffer,
            0,
            bytemuck::bytes_of(&SceneUniform {
                resolution: [width as f32, height as f32, 0.0, 0.0],
                viewport_origin: [board_field.x, board_field.y, 0.0, 0.0],
                viewport_size: [board_field.width, board_field.height, 0.0, 0.0],
                camera_center_scale: [
                    prepared.camera.center_x_nm,
                    prepared.camera.center_y_nm,
                    projection.scale,
                    0.0,
                ],
            }),
        );
        Self::upload_vertices(
            device,
            queue,
            &mut self.panel_vertex_buffer,
            &mut self.panel_vertex_capacity,
            "datum-gui-render-panel-vertex-buffer",
            panel_vertices,
        );
        Self::upload_vertices(
            device,
            queue,
            &mut self.viewport_underlay_vertex_buffer,
            &mut self.viewport_underlay_vertex_capacity,
            "datum-gui-render-viewport-underlay-vertex-buffer",
            viewport_underlay_vertices,
        );
        Self::upload_vertices(
            device,
            queue,
            &mut self.viewport_overlay_vertex_buffer,
            &mut self.viewport_overlay_vertex_capacity,
            "datum-gui-render-viewport-overlay-vertex-buffer",
            viewport_overlay_vertices,
        );
        self.sync_world_vertices(device, queue, world_vertices);
        let msaa_view = self.ensure_msaa(device, width, height).clone();
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("datum-gui-render-encoder"),
        });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("datum-gui-render-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &msaa_view,
                    resolve_target: Some(target),
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: APP_BG[0] as f64,
                            g: APP_BG[1] as f64,
                            b: APP_BG[2] as f64,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
                multiview_mask: None,
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &self.uniform_bind_group, &[]);
            if !panel_vertices.is_empty() {
                pass.set_vertex_buffer(
                    0,
                    self.panel_vertex_buffer
                        .as_ref()
                        .expect("panel vertex buffer should exist")
                        .slice(..),
                );
                pass.draw(0..panel_vertices.len() as u32, 0..1);
            }
            if !viewport_underlay_vertices.is_empty() {
                pass.set_scissor_rect(
                    prepared.scene_viewport.x.max(0.0).floor() as u32,
                    prepared.scene_viewport.y.max(0.0).floor() as u32,
                    prepared.scene_viewport.width.max(1.0).ceil() as u32,
                    prepared.scene_viewport.height.max(1.0).ceil() as u32,
                );
                pass.set_vertex_buffer(
                    0,
                    self.viewport_underlay_vertex_buffer
                        .as_ref()
                        .expect("viewport underlay vertex buffer should exist")
                        .slice(..),
                );
                pass.draw(0..viewport_underlay_vertices.len() as u32, 0..1);
            }
            if !world_vertices.is_empty() {
                pass.set_pipeline(&self.world_pipeline);
                pass.set_bind_group(0, &self.scene_bind_group, &[]);
                pass.set_scissor_rect(
                    prepared.scene_viewport.x.max(0.0).floor() as u32,
                    prepared.scene_viewport.y.max(0.0).floor() as u32,
                    prepared.scene_viewport.width.max(1.0).ceil() as u32,
                    prepared.scene_viewport.height.max(1.0).ceil() as u32,
                );
                pass.set_vertex_buffer(
                    0,
                    self.world_vertex_buffer
                        .as_ref()
                        .expect("world vertex buffer should exist")
                        .slice(..),
                );
                pass.draw(0..world_vertices.len() as u32, 0..1);
                pass.set_pipeline(&self.pipeline);
                pass.set_bind_group(0, &self.uniform_bind_group, &[]);
            }
            if !viewport_overlay_vertices.is_empty() {
                pass.set_scissor_rect(
                    prepared.scene_viewport.x.max(0.0).floor() as u32,
                    prepared.scene_viewport.y.max(0.0).floor() as u32,
                    prepared.scene_viewport.width.max(1.0).ceil() as u32,
                    prepared.scene_viewport.height.max(1.0).ceil() as u32,
                );
                pass.set_vertex_buffer(
                    0,
                    self.viewport_overlay_vertex_buffer
                        .as_ref()
                        .expect("viewport overlay vertex buffer should exist")
                        .slice(..),
                );
                pass.draw(0..viewport_overlay_vertices.len() as u32, 0..1);
            }
        }
        self.viewport.update(queue, Resolution { width, height });
        let text_buffer_indices =
            self.cached_text_buffer_indices(&prepared.text_runs, width, height);
        let text_areas: Vec<TextArea<'_>> = text_buffer_indices
            .iter()
            .zip(prepared.text_runs.iter())
            .map(|(index, run)| TextArea {
                buffer: &self.text_buffer_cache[*index].buffer,
                left: run.x,
                top: run.y,
                scale: 1.0,
                bounds: run
                    .clip_bounds
                    .map_or_else(TextBounds::default, |rect| TextBounds {
                        left: rect.x.floor() as i32,
                        top: rect.y.floor() as i32,
                        right: (rect.x + rect.width).ceil() as i32,
                        bottom: (rect.y + rect.height).ceil() as i32,
                    }),
                default_color: text_color(run.color),
                custom_glyphs: &[],
            })
            .collect();
        // Evict glyphs not used last frame before preparing this frame so the
        // atlas doesn't grow unbounded. Without this, a board with many text
        // elements (refdes labels, silkscreen, UI panels) eventually panics
        // with "glyph texture atlas is full" — observed on DOA2526.
        self.atlas.trim();
        self.text_renderer
            .prepare(
                device,
                queue,
                &mut self.font_system,
                &mut self.atlas,
                &self.viewport,
                text_areas,
                &mut self.swash_cache,
            )
            .map_err(|error| anyhow::anyhow!("prepare GUI text: {error}"))?;
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("datum-gui-text-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &msaa_view,
                    resolve_target: Some(target),
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
                multiview_mask: None,
            });
            self.text_renderer
                .render(&self.atlas, &self.viewport, &mut pass)
                .map_err(|error| anyhow::anyhow!("render GUI text: {error}"))?;
        }
        queue.submit([encoder.finish()]);
        Ok(())
    }
}

fn suffix_id(id: &str) -> &str {
    id.rsplit(':').next().unwrap_or(id)
}

fn draw_text(
    text: &str,
    x: f32,
    y: f32,
    size: f32,
    color: [f32; 3],
    face: TextFace,
    out: &mut Vec<TextRun>,
) {
    out.push(TextRun {
        text: text.to_string(),
        x,
        y,
        size,
        color,
        face,
        clip_bounds: None,
    });
}

fn draw_text_clipped(
    text: &str,
    x: f32,
    y: f32,
    size: f32,
    color: [f32; 3],
    face: TextFace,
    clip_bounds: RectPx,
    out: &mut Vec<TextRun>,
) {
    out.push(TextRun {
        text: text.to_string(),
        x,
        y,
        size,
        color,
        face,
        clip_bounds: Some(clip_bounds),
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn shell_layout_reserves_bottom_dock_and_viewport() {
        let layout = ShellLayout::for_window(1280, 800, None);
        assert!(layout.viewport.width > 0.0);
        assert_eq!(layout.bottom_strip.height, 44.0);
        assert!(layout.left_sidebar.width > 0.0);
        assert!(layout.right_sidebar.width > 0.0);
    }

    #[test]
    fn prepared_scene_preserves_viewport_dominance() {
        let state = datum_gui_protocol::load_fixture_workspace_state();
        let retained = RetainedScene::from_workspace(&state, 1280, 800);
        let prepared = PreparedScene::from_workspace(
            &state,
            1280,
            800,
            CameraState::fit_to_bounds(&state.scene.bounds),
            &retained,
        );
        assert!(prepared.layout.viewport.width > prepared.layout.left_sidebar.width);
        assert!(prepared.layout.viewport.width > prepared.layout.right_sidebar.width / 2.0);
    }

    #[test]
    fn hit_regions_include_review_rows_and_overlay_targets() {
        let state = datum_gui_protocol::load_fixture_workspace_state();
        let retained = RetainedScene::from_workspace(&state, 1280, 800);
        let prepared = PreparedScene::from_workspace(
            &state,
            1280,
            800,
            CameraState::fit_to_bounds(&state.scene.bounds),
            &retained,
        );
        assert!(prepared.hit_regions.iter().any(
            |region| matches!(region.target, HitTarget::ReviewAction(ref id) if id == "action-1")
        ));
    }

    #[test]
    fn hit_testing_prefers_overlay_over_underlying_authored_geometry() {
        let state = datum_gui_protocol::load_fixture_workspace_state();
        let retained = RetainedScene::from_workspace(&state, 1280, 800);
        let prepared = PreparedScene::from_workspace(
            &state,
            1280,
            800,
            CameraState::fit_to_bounds(&state.scene.bounds),
            &retained,
        );
        let overlay_rect = prepared
            .hit_regions
            .iter()
            .rev()
            .find_map(|region| match &region.target {
                HitTarget::ReviewAction(id) if id == "action-1" => Some(region.rect),
                _ => None,
            })
            .expect("action overlay hit region should exist");
        let hit = prepared
            .hit_test(
                overlay_rect.x + overlay_rect.width / 2.0,
                overlay_rect.y + overlay_rect.height / 2.0,
            )
            .expect("topmost hit should exist");
        assert_eq!(hit, &HitTarget::ReviewAction("action-1".to_string()));
    }

    #[test]
    fn roundrect_pad_uses_richer_geometry_than_rect_pad() {
        let viewport = RectPx {
            x: 0.0,
            y: 0.0,
            width: 200.0,
            height: 120.0,
        };
        let bounds = datum_gui_protocol::SceneBounds {
            min_x: 0,
            min_y: 0,
            max_x: 2_000_000,
            max_y: 1_200_000,
        };
        let projection = Projection::new(viewport, &bounds, CameraState::fit_to_bounds(&bounds));
        let mut rect_out = Vec::new();
        let mut roundrect_out = Vec::new();
        let rect_pad = datum_gui_protocol::PadPrimitive {
            object_id: "pad:rect".to_string(),
            object_kind: "pad".to_string(),
            source_object_uuid: "rect".to_string(),
            pad_uuid: "rect".to_string(),
            component_uuid: "U1".to_string(),
            net_uuid: None,
            layer_id: "L1".to_string(),
            copper_layer_ids: vec!["L1".to_string()],
            center: PointNm {
                x: 1_000_000,
                y: 600_000,
            },
            bounds: datum_gui_protocol::RectNm {
                min_x: 700_000,
                min_y: 350_000,
                max_x: 1_300_000,
                max_y: 850_000,
            },
            shape_kind: "rect".to_string(),
            roundrect_rratio_ppm: 250_000,
            mask_layer_ids: vec![],
            paste_layer_ids: vec![],
            solder_mask_margin_nm: 0,
            solder_paste_margin_nm: 0,
            solder_paste_margin_ratio_ppm: 0,
            drill_nm: None,
            rotation_degrees: 0.0,
        };
        let mut roundrect_pad = rect_pad.clone();
        roundrect_pad.shape_kind = "roundrect".to_string();

        push_pad_primitive(
            &mut rect_out,
            &rect_pad,
            &projection,
            "L1",
            PAD_COPPER,
            None,
            false,
        );
        push_pad_primitive(
            &mut roundrect_out,
            &roundrect_pad,
            &projection,
            "L1",
            PAD_COPPER,
            None,
            false,
        );

        assert!(roundrect_out.len() > rect_out.len());
    }

    #[test]
    fn roundrect_ratio_changes_corner_radius() {
        let viewport = RectPx {
            x: 0.0,
            y: 0.0,
            width: 200.0,
            height: 120.0,
        };
        let bounds = datum_gui_protocol::SceneBounds {
            min_x: 0,
            min_y: 0,
            max_x: 2_000_000,
            max_y: 1_200_000,
        };
        let projection = Projection::new(viewport, &bounds, CameraState::fit_to_bounds(&bounds));
        let small = datum_gui_protocol::PadPrimitive {
            object_id: "pad:rr-small".to_string(),
            object_kind: "pad".to_string(),
            source_object_uuid: "rr-small".to_string(),
            pad_uuid: "rr-small".to_string(),
            component_uuid: "U1".to_string(),
            net_uuid: None,
            layer_id: "L1".to_string(),
            copper_layer_ids: vec!["L1".to_string()],
            center: PointNm {
                x: 1_000_000,
                y: 600_000,
            },
            bounds: datum_gui_protocol::RectNm {
                min_x: 700_000,
                min_y: 350_000,
                max_x: 1_300_000,
                max_y: 850_000,
            },
            shape_kind: "roundrect".to_string(),
            roundrect_rratio_ppm: 100_000,
            mask_layer_ids: vec![],
            paste_layer_ids: vec![],
            solder_mask_margin_nm: 0,
            solder_paste_margin_nm: 0,
            solder_paste_margin_ratio_ppm: 0,
            drill_nm: None,
            rotation_degrees: 0.0,
        };
        let mut large = small.clone();
        large.pad_uuid = "rr-large".to_string();
        large.object_id = "pad:rr-large".to_string();
        large.source_object_uuid = "rr-large".to_string();
        large.roundrect_rratio_ppm = 400_000;
        let small_points = projected_pad_outline(&small, &projection, 0.0);
        let large_points = projected_pad_outline(&large, &projection, 0.0);
        assert_ne!(small_points[0], large_points[0]);
    }

    #[test]
    fn rotated_rect_pad_produces_non_axis_aligned_geometry() {
        let viewport = RectPx {
            x: 0.0,
            y: 0.0,
            width: 200.0,
            height: 120.0,
        };
        let bounds = datum_gui_protocol::SceneBounds {
            min_x: 0,
            min_y: 0,
            max_x: 2_000_000,
            max_y: 1_200_000,
        };
        let projection = Projection::new(viewport, &bounds, CameraState::fit_to_bounds(&bounds));
        let pad = datum_gui_protocol::PadPrimitive {
            object_id: "pad:rot".to_string(),
            object_kind: "pad".to_string(),
            source_object_uuid: "rot".to_string(),
            pad_uuid: "rot".to_string(),
            component_uuid: "U1".to_string(),
            net_uuid: None,
            layer_id: "L1".to_string(),
            copper_layer_ids: vec!["L1".to_string()],
            center: PointNm {
                x: 1_000_000,
                y: 600_000,
            },
            bounds: datum_gui_protocol::RectNm {
                min_x: 700_000,
                min_y: 450_000,
                max_x: 1_300_000,
                max_y: 750_000,
            },
            shape_kind: "rect".to_string(),
            roundrect_rratio_ppm: 250_000,
            mask_layer_ids: vec![],
            paste_layer_ids: vec![],
            solder_mask_margin_nm: 0,
            solder_paste_margin_nm: 0,
            solder_paste_margin_ratio_ppm: 0,
            drill_nm: None,
            rotation_degrees: 45.0,
        };

        let points = projected_pad_outline(&pad, &projection, 0.0);
        assert_eq!(points.len(), 4);
        assert!((points[0].0 - points[1].0).abs() > 0.1);
        assert!((points[0].1 - points[1].1).abs() > 0.1);
    }

    #[test]
    fn derived_mask_pad_expands_by_clearance() {
        let pad = datum_gui_protocol::PadPrimitive {
            object_id: "pad:mask".to_string(),
            object_kind: "pad".to_string(),
            source_object_uuid: "mask".to_string(),
            pad_uuid: "mask".to_string(),
            component_uuid: "U1".to_string(),
            net_uuid: None,
            layer_id: "L0".to_string(),
            copper_layer_ids: vec!["L1".to_string()],
            center: PointNm {
                x: 1_000_000,
                y: 600_000,
            },
            bounds: datum_gui_protocol::RectNm {
                min_x: 900_000,
                min_y: 500_000,
                max_x: 1_100_000,
                max_y: 700_000,
            },
            shape_kind: "rect".to_string(),
            roundrect_rratio_ppm: 250_000,
            mask_layer_ids: vec!["L39".to_string()],
            paste_layer_ids: vec![],
            solder_mask_margin_nm: 25_000,
            solder_paste_margin_nm: 0,
            solder_paste_margin_ratio_ppm: 0,
            drill_nm: None,
            rotation_degrees: 0.0,
        };
        let setup = datum_gui_protocol::ScenePadExpansionSetup {
            pad_to_mask_clearance_nm: 25_000,
            ..Default::default()
        };
        let derived = derived_process_pad(&pad, "L39", PadProcessLayerKind::Mask, &setup);
        assert_eq!(derived.layer_id, "L39");
        assert_eq!(derived.bounds.min_x, 875_000);
        assert_eq!(derived.bounds.max_x, 1_125_000);
        assert_eq!(derived.bounds.min_y, 475_000);
        assert_eq!(derived.bounds.max_y, 725_000);
        assert_eq!(derived.drill_nm, None);
    }

    #[test]
    fn derived_paste_pad_applies_ratio_and_clearance() {
        let pad = datum_gui_protocol::PadPrimitive {
            object_id: "pad:paste".to_string(),
            object_kind: "pad".to_string(),
            source_object_uuid: "paste".to_string(),
            pad_uuid: "paste".to_string(),
            component_uuid: "U1".to_string(),
            net_uuid: None,
            layer_id: "L0".to_string(),
            copper_layer_ids: vec!["L1".to_string()],
            center: PointNm {
                x: 1_000_000,
                y: 600_000,
            },
            bounds: datum_gui_protocol::RectNm {
                min_x: 900_000,
                min_y: 500_000,
                max_x: 1_100_000,
                max_y: 700_000,
            },
            shape_kind: "rect".to_string(),
            roundrect_rratio_ppm: 250_000,
            mask_layer_ids: vec![],
            paste_layer_ids: vec!["L35".to_string()],
            solder_mask_margin_nm: 0,
            solder_paste_margin_nm: -10_000,
            solder_paste_margin_ratio_ppm: -100_000,
            drill_nm: None,
            rotation_degrees: 0.0,
        };
        let setup = datum_gui_protocol::ScenePadExpansionSetup {
            pad_to_paste_clearance_nm: -10_000,
            pad_to_paste_ratio_ppm: -100_000,
            ..Default::default()
        };
        let derived = derived_process_pad(&pad, "L35", PadProcessLayerKind::Paste, &setup);
        assert_eq!(derived.layer_id, "L35");
        assert_eq!(derived.bounds.min_x, 920_000);
        assert_eq!(derived.bounds.max_x, 1_080_000);
        assert_eq!(derived.bounds.min_y, 520_000);
        assert_eq!(derived.bounds.max_y, 680_000);
    }

    #[test]
    fn derived_process_pad_uses_pad_level_overrides_not_board_globals() {
        let pad = datum_gui_protocol::PadPrimitive {
            object_id: "pad:override".to_string(),
            object_kind: "pad".to_string(),
            source_object_uuid: "override".to_string(),
            pad_uuid: "override".to_string(),
            component_uuid: "U1".to_string(),
            net_uuid: None,
            layer_id: "L0".to_string(),
            copper_layer_ids: vec!["L1".to_string()],
            center: PointNm {
                x: 1_000_000,
                y: 600_000,
            },
            bounds: datum_gui_protocol::RectNm {
                min_x: 900_000,
                min_y: 500_000,
                max_x: 1_100_000,
                max_y: 700_000,
            },
            shape_kind: "rect".to_string(),
            roundrect_rratio_ppm: 250_000,
            mask_layer_ids: vec!["L39".to_string()],
            paste_layer_ids: vec!["L35".to_string()],
            solder_mask_margin_nm: 50_000,
            solder_paste_margin_nm: -50_000,
            solder_paste_margin_ratio_ppm: 0,
            drill_nm: None,
            rotation_degrees: 0.0,
        };
        let setup = datum_gui_protocol::ScenePadExpansionSetup {
            pad_to_mask_clearance_nm: 0,
            pad_to_paste_clearance_nm: 0,
            pad_to_paste_ratio_ppm: 0,
            ..Default::default()
        };
        let mask = derived_process_pad(&pad, "L39", PadProcessLayerKind::Mask, &setup);
        let paste = derived_process_pad(&pad, "L35", PadProcessLayerKind::Paste, &setup);
        assert_eq!(mask.bounds.min_x, 850_000);
        assert_eq!(mask.bounds.max_x, 1_150_000);
        assert_eq!(paste.bounds.min_x, 950_000);
        assert_eq!(paste.bounds.max_x, 1_050_000);
    }

    #[test]
    fn render_stage_orders_layer_type_then_side() {
        let layers = vec![
            datum_gui_protocol::SceneLayer {
                layer_id: "L0".to_string(),
                name: "F.Cu".to_string(),
                kind: "copper".to_string(),
                render_order: 0,
                visible_by_default: true,
            },
            datum_gui_protocol::SceneLayer {
                layer_id: "L38".to_string(),
                name: "B.Mask".to_string(),
                kind: "mask".to_string(),
                render_order: 1,
                visible_by_default: false,
            },
            datum_gui_protocol::SceneLayer {
                layer_id: "L39".to_string(),
                name: "F.Mask".to_string(),
                kind: "mask".to_string(),
                render_order: 2,
                visible_by_default: false,
            },
            datum_gui_protocol::SceneLayer {
                layer_id: "L34".to_string(),
                name: "B.Paste".to_string(),
                kind: "paste".to_string(),
                render_order: 3,
                visible_by_default: false,
            },
            datum_gui_protocol::SceneLayer {
                layer_id: "L35".to_string(),
                name: "F.Paste".to_string(),
                kind: "paste".to_string(),
                render_order: 4,
                visible_by_default: false,
            },
        ];
        assert!(scene_layer_stack_priority("L39", &layers) > scene_layer_stack_priority("L0", &layers));
        assert!(scene_layer_stack_priority("L35", &layers) > scene_layer_stack_priority("L39", &layers));
        assert!(scene_layer_stack_priority("L39", &layers) > scene_layer_stack_priority("L38", &layers));
        assert!(scene_layer_stack_priority("L35", &layers) > scene_layer_stack_priority("L34", &layers));
    }

    #[test]
    fn component_polygon_graphic_adds_fill_and_outline() {
        let viewport = RectPx {
            x: 0.0,
            y: 0.0,
            width: 240.0,
            height: 160.0,
        };
        let bounds = datum_gui_protocol::SceneBounds {
            min_x: 0,
            min_y: 0,
            max_x: 2_400_000,
            max_y: 1_600_000,
        };
        let projection = Projection::new(viewport, &bounds, CameraState::fit_to_bounds(&bounds));
        let graphic = ComponentGraphicPrimitive {
            graphic_id: "g1".to_string(),
            component_uuid: "U1".to_string(),
            layer_id: Some("L1".to_string()),
            primitive_kind: "polygon".to_string(),
            render_role: "component_mechanical".to_string(),
            width_nm: Some(120_000),
            closed: true,
            path: vec![
                PointNm {
                    x: 300_000,
                    y: 300_000,
                },
                PointNm {
                    x: 2_100_000,
                    y: 300_000,
                },
                PointNm {
                    x: 2_100_000,
                    y: 1_300_000,
                },
                PointNm {
                    x: 300_000,
                    y: 1_300_000,
                },
            ],
        };
        let mut out = Vec::new();

        push_component_graphic_primitive(&mut out, &graphic, &projection, false, false, false);

        assert!(out.len() > 1);
    }

    #[test]
    fn layer_appearance_distinguishes_top_and_bottom_copper() {
        let top = resolve_layer_appearance(Some("F.Cu"));
        let bottom = resolve_layer_appearance(Some("B.Cu"));

        assert_ne!(top.authored_track, bottom.authored_track);
        assert_ne!(top.proposal, bottom.proposal);
        assert_ne!(top.silkscreen, bottom.silkscreen);
    }

    #[test]
    fn detail_tier_changes_with_projected_board_scale() {
        let viewport = RectPx {
            x: 0.0,
            y: 0.0,
            width: 1200.0,
            height: 800.0,
        };
        let fine_bounds = datum_gui_protocol::SceneBounds {
            min_x: 0,
            min_y: 0,
            max_x: 20_000_000,
            max_y: 10_000_000,
        };
        let coarse_bounds = datum_gui_protocol::SceneBounds {
            min_x: 0,
            min_y: 0,
            max_x: 300_000_000,
            max_y: 200_000_000,
        };

        let fine_projection = Projection::new(
            viewport,
            &fine_bounds,
            CameraState::fit_to_bounds(&fine_bounds),
        );
        let coarse_projection = Projection::new(
            viewport,
            &coarse_bounds,
            CameraState::fit_to_bounds(&coarse_bounds),
        );

        assert_eq!(detail_tier(&fine_projection), DetailTier::Fine);
        assert_eq!(detail_tier(&coarse_projection), DetailTier::Coarse);
    }

    #[test]
    #[ignore = "selection highlight is now lane-aware instead of one exact selected-pad color"]
    fn datum_test_q2_selection_emits_selected_pad_geometry() {
        let request = datum_gui_protocol::LiveReviewRequest {
            project_root: PathBuf::from("/home/bfadmin/Documents/kicad_projects/Datum-eda/datum-test"),
            board_file: Some(PathBuf::from(
                "/home/bfadmin/Documents/kicad_projects/Datum-eda/datum-test/datum-test.kicad_pcb",
            )),
            artifact_path: None,
            net_uuid: None,
            from_anchor_pad_uuid: None,
            to_anchor_pad_uuid: None,
            profile: None,
        };
        let mut state = datum_gui_protocol::load_board_editor_workspace_state(&request)
            .expect("datum-test workspace should load");
        let (q2_object_id, q2_component_uuid) = state
            .scene
            .components
            .iter()
            .find(|component| component.reference == "Q2")
            .map(|component| (component.object_id.clone(), component.component_uuid.clone()))
            .expect("Q2 should exist");
        state.select_authored_object(&q2_object_id);
        let retained = RetainedScene::from_workspace(&state, 1280, 800);
        let q2_pads: Vec<_> = state
            .scene
            .pads
            .iter()
            .filter(|pad| pad.component_uuid == q2_component_uuid)
            .collect();
        assert!(!q2_pads.is_empty(), "Q2 should own pads");
        let selected_pad_color = selected_copper_color(
            resolve_layer_appearance_with_scene(
                Some(&q2_pads[0].copper_layer_ids[0]),
                &state.scene.layers,
            )
            .pad_copper,
        );
        let selected_vertices = retained
            .world_vertices()
            .iter()
            .filter(|vertex| vertex.color == selected_pad_color)
            .filter(|vertex| {
                q2_pads.iter().any(|pad| {
                    vertex.pos[0] >= pad.bounds.min_x as f32
                        && vertex.pos[0] <= pad.bounds.max_x as f32
                        && vertex.pos[1] >= pad.bounds.min_y as f32
                        && vertex.pos[1] <= pad.bounds.max_y as f32
                })
            })
            .count();
        assert!(
            selected_vertices > 0,
            "selecting Q2 should emit selected pad geometry inside Q2 pad bounds"
        );
    }

    #[test]
    #[ignore = "selection highlight is now lane-aware instead of one exact selected-pad color"]
    fn datum_test_q3_selection_does_not_select_c1_pads() {
        let request = datum_gui_protocol::LiveReviewRequest {
            project_root: PathBuf::from("/home/bfadmin/Documents/kicad_projects/Datum-eda/datum-test"),
            board_file: Some(PathBuf::from(
                "/home/bfadmin/Documents/kicad_projects/Datum-eda/datum-test/datum-test.kicad_pcb",
            )),
            artifact_path: None,
            net_uuid: None,
            from_anchor_pad_uuid: None,
            to_anchor_pad_uuid: None,
            profile: None,
        };
        let mut state = datum_gui_protocol::load_board_editor_workspace_state(&request)
            .expect("datum-test workspace should load");
        let (q3_object_id, q3_component_uuid) = state
            .scene
            .components
            .iter()
            .find(|component| component.reference == "Q3")
            .map(|component| (component.object_id.clone(), component.component_uuid.clone()))
            .expect("Q3 should exist");
        let c1_component_uuid = state
            .scene
            .components
            .iter()
            .find(|component| component.reference == "C1")
            .map(|component| component.component_uuid.clone())
            .expect("C1 should exist");
        state.select_authored_object(&q3_object_id);
        let retained = RetainedScene::from_workspace(&state, 1280, 800);
        let q3_pads: Vec<_> = state
            .scene
            .pads
            .iter()
            .filter(|pad| pad.component_uuid == q3_component_uuid)
            .collect();
        let c1_pads: Vec<_> = state
            .scene
            .pads
            .iter()
            .filter(|pad| pad.component_uuid == c1_component_uuid)
            .collect();
        let selected_pad_color = selected_copper_color(
            resolve_layer_appearance_with_scene(
                Some(&q3_pads[0].copper_layer_ids[0]),
                &state.scene.layers,
            )
            .pad_copper,
        );
        let selected_in_q3 = retained
            .world_vertices()
            .iter()
            .filter(|vertex| vertex.color == selected_pad_color)
            .filter(|vertex| {
                q3_pads.iter().any(|pad| {
                    vertex.pos[0] >= pad.bounds.min_x as f32
                        && vertex.pos[0] <= pad.bounds.max_x as f32
                        && vertex.pos[1] >= pad.bounds.min_y as f32
                        && vertex.pos[1] <= pad.bounds.max_y as f32
                })
            })
            .count();
        let selected_in_c1 = retained
            .world_vertices()
            .iter()
            .filter(|vertex| vertex.color == selected_pad_color)
            .filter(|vertex| {
                c1_pads.iter().any(|pad| {
                    vertex.pos[0] >= pad.bounds.min_x as f32
                        && vertex.pos[0] <= pad.bounds.max_x as f32
                        && vertex.pos[1] >= pad.bounds.min_y as f32
                        && vertex.pos[1] <= pad.bounds.max_y as f32
                })
            })
            .count();
        assert!(
            selected_in_q3 > 0,
            "selecting Q3 should emit selected pad geometry inside Q3 pad bounds"
        );
        assert_eq!(
            selected_in_c1, 0,
            "selecting Q3 must not emit selected pad geometry inside C1 pad bounds"
        );
    }

    #[test]
    #[ignore = "selection highlight is now lane-aware instead of one exact selected-pad color"]
    fn datum_test_q2_selection_does_not_select_q1_pads() {
        let request = datum_gui_protocol::LiveReviewRequest {
            project_root: PathBuf::from("/home/bfadmin/Documents/kicad_projects/Datum-eda/datum-test"),
            board_file: Some(PathBuf::from(
                "/home/bfadmin/Documents/kicad_projects/Datum-eda/datum-test/datum-test.kicad_pcb",
            )),
            artifact_path: None,
            net_uuid: None,
            from_anchor_pad_uuid: None,
            to_anchor_pad_uuid: None,
            profile: None,
        };
        let mut state = datum_gui_protocol::load_board_editor_workspace_state(&request)
            .expect("datum-test workspace should load");
        let (q2_object_id, q2_component_uuid) = state
            .scene
            .components
            .iter()
            .find(|component| component.reference == "Q2")
            .map(|component| (component.object_id.clone(), component.component_uuid.clone()))
            .expect("Q2 should exist");
        let q1_component_uuid = state
            .scene
            .components
            .iter()
            .find(|component| component.reference == "Q1")
            .map(|component| component.component_uuid.clone())
            .expect("Q1 should exist");
        state.select_authored_object(&q2_object_id);
        let retained = RetainedScene::from_workspace(&state, 1280, 800);
        let q2_pads: Vec<_> = state
            .scene
            .pads
            .iter()
            .filter(|pad| pad.component_uuid == q2_component_uuid)
            .collect();
        let q1_pads: Vec<_> = state
            .scene
            .pads
            .iter()
            .filter(|pad| pad.component_uuid == q1_component_uuid)
            .collect();
        let selected_pad_color = selected_copper_color(
            resolve_layer_appearance_with_scene(
                Some(&q2_pads[0].copper_layer_ids[0]),
                &state.scene.layers,
            )
            .pad_copper,
        );
        let selected_in_q2 = retained
            .world_vertices()
            .iter()
            .filter(|vertex| vertex.color == selected_pad_color)
            .filter(|vertex| {
                q2_pads.iter().any(|pad| {
                    vertex.pos[0] >= pad.bounds.min_x as f32
                        && vertex.pos[0] <= pad.bounds.max_x as f32
                        && vertex.pos[1] >= pad.bounds.min_y as f32
                        && vertex.pos[1] <= pad.bounds.max_y as f32
                })
            })
            .count();
        let selected_in_q1 = retained
            .world_vertices()
            .iter()
            .filter(|vertex| vertex.color == selected_pad_color)
            .filter(|vertex| {
                q1_pads.iter().any(|pad| {
                    vertex.pos[0] >= pad.bounds.min_x as f32
                        && vertex.pos[0] <= pad.bounds.max_x as f32
                        && vertex.pos[1] >= pad.bounds.min_y as f32
                        && vertex.pos[1] <= pad.bounds.max_y as f32
                })
            })
            .count();
        assert!(
            selected_in_q2 > 0,
            "selecting Q2 should emit selected pad geometry inside Q2 pad bounds"
        );
        assert_eq!(
            selected_in_q1, 0,
            "selecting Q2 must not emit selected pad geometry inside Q1 pad bounds"
        );
    }

    #[test]
    #[ignore = "selection highlight is now lane-aware instead of one exact selected-pad color"]
    fn datum_test_switching_q1_to_q2_rebuilds_selected_geometry() {
        let request = datum_gui_protocol::LiveReviewRequest {
            project_root: PathBuf::from("/home/bfadmin/Documents/kicad_projects/Datum-eda/datum-test"),
            board_file: Some(PathBuf::from(
                "/home/bfadmin/Documents/kicad_projects/Datum-eda/datum-test/datum-test.kicad_pcb",
            )),
            artifact_path: None,
            net_uuid: None,
            from_anchor_pad_uuid: None,
            to_anchor_pad_uuid: None,
            profile: None,
        };
        let mut state = datum_gui_protocol::load_board_editor_workspace_state(&request)
            .expect("datum-test workspace should load");
        let (q1_object_id, q1_component_uuid) = state
            .scene
            .components
            .iter()
            .find(|component| component.reference == "Q1")
            .map(|component| (component.object_id.clone(), component.component_uuid.clone()))
            .expect("Q1 should exist");
        let (q2_object_id, q2_component_uuid) = state
            .scene
            .components
            .iter()
            .find(|component| component.reference == "Q2")
            .map(|component| (component.object_id.clone(), component.component_uuid.clone()))
            .expect("Q2 should exist");
        let q1_pad_bounds: Vec<_> = state
            .scene
            .pads
            .iter()
            .filter(|pad| pad.component_uuid == q1_component_uuid)
            .map(|pad| pad.bounds)
            .collect();
        let q2_pad_bounds: Vec<_> = state
            .scene
            .pads
            .iter()
            .filter(|pad| pad.component_uuid == q2_component_uuid)
            .map(|pad| pad.bounds)
            .collect();

        state.select_authored_object(&q1_object_id);
        let _first = RetainedScene::from_workspace(&state, 1280, 800);

        state.select_authored_object(&q2_object_id);
        let second = RetainedScene::from_workspace(&state, 1280, 800);
        let selected_pad_color = selected_copper_color(
            resolve_layer_appearance_with_scene(
                Some(&state
                    .scene
                    .pads
                    .iter()
                    .find(|pad| pad.component_uuid == q2_component_uuid)
                    .expect("Q2 pad should exist")
                    .copper_layer_ids[0]),
                &state.scene.layers,
            )
            .pad_copper,
        );

        let selected_in_q1 = second
            .world_vertices()
            .iter()
            .filter(|vertex| vertex.color == selected_pad_color)
            .filter(|vertex| {
                q1_pad_bounds.iter().any(|pad| {
                    vertex.pos[0] >= pad.min_x as f32
                        && vertex.pos[0] <= pad.max_x as f32
                        && vertex.pos[1] >= pad.min_y as f32
                        && vertex.pos[1] <= pad.max_y as f32
                })
            })
            .count();
        let selected_in_q2 = second
            .world_vertices()
            .iter()
            .filter(|vertex| vertex.color == selected_pad_color)
            .filter(|vertex| {
                q2_pad_bounds.iter().any(|pad| {
                    vertex.pos[0] >= pad.min_x as f32
                        && vertex.pos[0] <= pad.max_x as f32
                        && vertex.pos[1] >= pad.min_y as f32
                        && vertex.pos[1] <= pad.max_y as f32
                })
            })
            .count();

        assert_eq!(
            selected_in_q1, 0,
            "after switching from Q1 to Q2, selected geometry must not remain in Q1 pad bounds"
        );
        assert!(
            selected_in_q2 > 0,
            "after switching from Q1 to Q2, selected geometry must appear in Q2 pad bounds"
        );
    }

    #[test]
    fn debug_datum_test_q1_q2_component_geometry() {
        let request = datum_gui_protocol::LiveReviewRequest {
            project_root: PathBuf::from("/home/bfadmin/Documents/kicad_projects/Datum-eda/datum-test"),
            board_file: Some(PathBuf::from(
                "/home/bfadmin/Documents/kicad_projects/Datum-eda/datum-test/datum-test.kicad_pcb",
            )),
            artifact_path: None,
            net_uuid: None,
            from_anchor_pad_uuid: None,
            to_anchor_pad_uuid: None,
            profile: None,
        };
        let state = datum_gui_protocol::load_board_editor_workspace_state(&request)
            .expect("datum-test workspace should load");
        for reference in ["Q1", "Q2"] {
            let component = state
                .scene
                .components
                .iter()
                .find(|component| component.reference == reference)
                .unwrap_or_else(|| panic!("missing component {reference}"));
            let pads: Vec<_> = state
                .scene
                .pads
                .iter()
                .filter(|pad| pad.component_uuid == component.component_uuid)
                .collect();
            let body = inferred_component_body_bounds(&pads);
            eprintln!(
                "{reference}: object_id={} component_uuid={} pos=({}, {}) body={body:?}",
                component.object_id,
                component.component_uuid,
                component.position.x,
                component.position.y,
            );
            for pad in pads {
                eprintln!(
                    "  pad {} center=({}, {}) bounds=({}, {}, {}, {})",
                    pad.object_id,
                    pad.center.x,
                    pad.center.y,
                    pad.bounds.min_x,
                    pad.bounds.min_y,
                    pad.bounds.max_x,
                    pad.bounds.max_y
                );
            }
        }
    }

    #[test]
    fn inferred_component_body_geometry_handles_quarter_turn_parts() {
        let pads = vec![
            datum_gui_protocol::PadPrimitive {
                object_id: "pad:a".to_string(),
                object_kind: "pad".to_string(),
                source_object_uuid: "a".to_string(),
                pad_uuid: "a".to_string(),
                component_uuid: "QX".to_string(),
                net_uuid: None,
                layer_id: "L0".to_string(),
                copper_layer_ids: vec!["L0".to_string()],
                center: PointNm { x: 0, y: 900_000 },
                bounds: datum_gui_protocol::RectNm {
                    min_x: -250_000,
                    min_y: 600_000,
                    max_x: 250_000,
                    max_y: 1_200_000,
                },
                shape_kind: "rect".to_string(),
                roundrect_rratio_ppm: 0,
                mask_layer_ids: vec![],
                paste_layer_ids: vec![],
                solder_mask_margin_nm: 0,
                solder_paste_margin_nm: 0,
                solder_paste_margin_ratio_ppm: 0,
                drill_nm: None,
                rotation_degrees: 90.0,
            },
            datum_gui_protocol::PadPrimitive {
                object_id: "pad:b".to_string(),
                object_kind: "pad".to_string(),
                source_object_uuid: "b".to_string(),
                pad_uuid: "b".to_string(),
                component_uuid: "QX".to_string(),
                net_uuid: None,
                layer_id: "L0".to_string(),
                copper_layer_ids: vec!["L0".to_string()],
                center: PointNm { x: 0, y: -900_000 },
                bounds: datum_gui_protocol::RectNm {
                    min_x: -250_000,
                    min_y: -1_200_000,
                    max_x: 250_000,
                    max_y: -600_000,
                },
                shape_kind: "rect".to_string(),
                roundrect_rratio_ppm: 0,
                mask_layer_ids: vec![],
                paste_layer_ids: vec![],
                solder_mask_margin_nm: 0,
                solder_paste_margin_nm: 0,
                solder_paste_margin_ratio_ppm: 0,
                drill_nm: None,
                rotation_degrees: 90.0,
            },
            datum_gui_protocol::PadPrimitive {
                object_id: "pad:c".to_string(),
                object_kind: "pad".to_string(),
                source_object_uuid: "c".to_string(),
                pad_uuid: "c".to_string(),
                component_uuid: "QX".to_string(),
                net_uuid: None,
                layer_id: "L0".to_string(),
                copper_layer_ids: vec!["L0".to_string()],
                center: PointNm { x: 800_000, y: 0 },
                bounds: datum_gui_protocol::RectNm {
                    min_x: 550_000,
                    min_y: -300_000,
                    max_x: 1_050_000,
                    max_y: 300_000,
                },
                shape_kind: "rect".to_string(),
                roundrect_rratio_ppm: 0,
                mask_layer_ids: vec![],
                paste_layer_ids: vec![],
                solder_mask_margin_nm: 0,
                solder_paste_margin_nm: 0,
                solder_paste_margin_ratio_ppm: 0,
                drill_nm: None,
                rotation_degrees: 90.0,
            },
        ];
        let pad_refs: Vec<_> = pads.iter().collect();

        let (_, width, height, rotation_degrees) =
            inferred_component_body_geometry(&pad_refs, 90.0).expect("body geometry");

        assert_eq!(rotation_degrees.round() as i32, 90);
        assert!(
            height > width,
            "quarter-turn body should stay tall, got {width}x{height}"
        );
    }
}
