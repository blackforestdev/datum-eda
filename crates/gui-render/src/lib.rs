use datum_gui_protocol::{
    Affine2DFixedPrimitive, BoardGraphicPrimitive, BoardReviewSceneV1, BoardTextGeometryPrimitive,
    BoardTextPrimitive, ComponentGraphicPrimitive, ComponentTextPrimitive, GlyphMeshAssetPrimitive,
    GlyphMeshHandlePrimitive, PointNm, ProposalOverlayPrimitive, ReviewWorkspaceState,
    SelectionTarget, UnroutedPrimitive, WorkspaceTool,
};
use eda_engine::board::BoardText;
use eda_engine::export::render_silkscreen_text_strokes;
use eda_engine::ir::geometry::{LayerId, Point};
use glyphon::{
    Attrs, Buffer, Cache, Color, Family, FontSystem, Metrics, Resolution, Shaping, SwashCache,
    TextArea, TextAtlas, TextBounds, TextRenderer, Viewport,
};
use std::collections::BTreeMap;
use std::ops::Range;
use taffy::prelude::*;
use uuid::Uuid;
use wgpu::util::DeviceExt;

mod bottom_dock;
#[cfg(feature = "visual")]
pub mod design_artboards;
mod design_tokens;
mod inspector_check_finding;
mod marking_menu;
mod menu_chrome;
mod side_panels;
mod source_shard_panel;
#[cfg(feature = "visual")]
pub mod visual_capture;
use bottom_dock::render_bottom_tabs;
use marking_menu::render_marking_menu;
use menu_chrome::render_menu_bar;
use side_panels::render_side_panels;
use source_shard_panel::{
    render_source_shard_attention_rows as render_shard_rows, source_shard_health_label,
};
#[cfg(feature = "visual")]
pub mod visual_diff;
#[cfg(feature = "visual")]
pub mod visual_manifest;
#[cfg(feature = "visual")]
pub mod visual_runner;

const UI_CARD_MARGIN: f32 = design_tokens::spacing::SP_04;
const UI_CARD_PADDING_X: f32 = design_tokens::spacing::SP_04;
const UI_CARD_TITLE_Y: f32 = design_tokens::spacing::SP_04;
const UI_CARD_DIVIDER_Y: f32 = design_tokens::spacing::SP_07 - design_tokens::spacing::SP_02;
const UI_CARD_CONTENT_TOP: f32 = design_tokens::spacing::SP_07 + design_tokens::spacing::SP_01;
const UI_CARD_CONTENT_BOTTOM: f32 = design_tokens::typography::BODY_LINE;
const UI_ROW_PROJECT_TITLE: f32 = design_tokens::typography::BODY_LINE;
const UI_ROW_BOARD_SUBTITLE: f32 = design_tokens::typography::HEADER_LINE;
const UI_ROW_NET: f32 = design_tokens::typography::BODY_LINE;
const UI_ROW_SOURCE_LABEL: f32 = design_tokens::typography::CAPTION_LINE;
const UI_ROW_SOURCE_ATTENTION: f32 = design_tokens::typography::CAPTION_LINE;
const UI_ROW_BUTTON: f32 = design_tokens::spacing::SP_06 - design_tokens::spacing::SP_02;
const UI_ROW_TOOL_LABEL: f32 = design_tokens::typography::CAPTION_LINE;
const UI_ROW_TOOL_GRID: f32 = design_tokens::spacing::SP_08 + design_tokens::spacing::SP_02;
const UI_ROW_NOTICE: f32 = design_tokens::typography::CAPTION_LINE;
const UI_STACK_GAP_SMALL: f32 = design_tokens::spacing::SP_03;
const UI_STACK_GAP_MEDIUM: f32 = design_tokens::spacing::SP_04;

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

    fn scale_by(self, scale: f32) -> Self {
        Self {
            x: self.x * scale,
            y: self.y * scale,
            width: self.width * scale,
            height: self.height * scale,
        }
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
    pub top_menu_bar: RectPx,
    pub tool_rail: RectPx,
    pub viewport: RectPx,
    pub left_sidebar: RectPx,
    pub right_sidebar: RectPx,
    pub bottom_strip: RectPx,
    pub status_bar: RectPx,
}

impl ShellLayout {
    pub fn for_surface(
        physical_width: u32,
        physical_height: u32,
        scale_factor: f32,
        dock_height_px: Option<u32>,
    ) -> Self {
        let scale = scale_factor.max(0.01);
        let logical_width = ((physical_width as f32) / scale).round().max(1.0) as u32;
        let logical_height = ((physical_height as f32) / scale).round().max(1.0) as u32;
        Self::for_window(logical_width, logical_height, dock_height_px).scale_by(scale)
    }

    pub fn for_window(width: u32, height: u32, dock_height_px: Option<u32>) -> Self {
        let width = width as f32;
        let height = height as f32;
        let menu_height = design_tokens::spacing::SP_07 + design_tokens::spacing::SP_01;
        let rail_width = design_tokens::spacing::SP_09;
        let status_height = design_tokens::spacing::SP_06 + design_tokens::spacing::SP_01;
        let left_width = 228.0_f32.min(width * 0.3);
        let right_width = 300.0_f32.min(width * 0.35);
        let bottom_height = match dock_height_px {
            Some(h) => (h as f32).clamp(44.0, height * 0.6),
            None => 44.0_f32.min(height * 0.25),
        };
        if let Some(layout) = solve_shell_layout_with_taffy(
            width,
            height,
            menu_height,
            rail_width,
            left_width,
            right_width,
            bottom_height,
            status_height,
        ) {
            return layout;
        }
        // Taffy is the adopted shell solver; keep a manual fallback so a
        // malformed runtime input cannot prevent the GUI from opening.
        Self {
            top_menu_bar: RectPx {
                x: 0.0,
                y: 0.0,
                width,
                height: menu_height,
            },
            tool_rail: RectPx {
                x: 0.0,
                y: menu_height,
                width: rail_width,
                height: (height - menu_height - bottom_height - status_height).max(0.0),
            },
            left_sidebar: RectPx {
                x: rail_width,
                y: menu_height,
                width: left_width,
                height: (height - menu_height - bottom_height - status_height).max(0.0),
            },
            viewport: RectPx {
                x: rail_width + left_width,
                y: menu_height,
                width: (width - rail_width - left_width - right_width).max(0.0),
                height: (height - menu_height - bottom_height - status_height).max(0.0),
            },
            right_sidebar: RectPx {
                x: (width - right_width).max(0.0),
                y: menu_height,
                width: right_width,
                height: (height - menu_height - bottom_height - status_height).max(0.0),
            },
            bottom_strip: RectPx {
                x: 0.0,
                y: height - bottom_height - status_height,
                width,
                height: bottom_height,
            },
            status_bar: RectPx {
                x: 0.0,
                y: height - status_height,
                width,
                height: status_height,
            },
        }
    }

    pub fn scene_viewport(&self) -> RectPx {
        inset_rect(self.viewport, 16.0, 76.0, 16.0, 16.0)
    }

    fn scale_by(self, scale: f32) -> Self {
        Self {
            top_menu_bar: self.top_menu_bar.scale_by(scale),
            tool_rail: self.tool_rail.scale_by(scale),
            viewport: self.viewport.scale_by(scale),
            left_sidebar: self.left_sidebar.scale_by(scale),
            right_sidebar: self.right_sidebar.scale_by(scale),
            bottom_strip: self.bottom_strip.scale_by(scale),
            status_bar: self.status_bar.scale_by(scale),
        }
    }
}

fn solve_shell_layout_with_taffy(
    width: f32,
    height: f32,
    menu_height: f32,
    rail_width: f32,
    left_width: f32,
    right_width: f32,
    bottom_height: f32,
    status_height: f32,
) -> Option<ShellLayout> {
    let mut taffy: TaffyTree<()> = TaffyTree::new();
    let top_menu_bar = taffy
        .new_leaf(Style {
            grid_row: line(1),
            grid_column: span(4),
            ..Default::default()
        })
        .ok()?;
    let tool_rail = taffy
        .new_leaf(Style {
            grid_row: line(2),
            grid_column: line(1),
            ..Default::default()
        })
        .ok()?;
    let left_sidebar = taffy
        .new_leaf(Style {
            grid_row: line(2),
            grid_column: line(2),
            ..Default::default()
        })
        .ok()?;
    let viewport = taffy
        .new_leaf(Style {
            grid_row: line(2),
            grid_column: line(3),
            ..Default::default()
        })
        .ok()?;
    let right_sidebar = taffy
        .new_leaf(Style {
            grid_row: line(2),
            grid_column: line(4),
            ..Default::default()
        })
        .ok()?;
    let bottom_strip = taffy
        .new_leaf(Style {
            grid_row: line(3),
            grid_column: span(4),
            ..Default::default()
        })
        .ok()?;
    let status_bar = taffy
        .new_leaf(Style {
            grid_row: line(4),
            grid_column: span(4),
            ..Default::default()
        })
        .ok()?;
    let root = taffy
        .new_with_children(
            Style {
                display: Display::Grid,
                size: Size {
                    width: length(width),
                    height: length(height),
                },
                grid_template_columns: vec![
                    length(rail_width),
                    length(left_width),
                    fr(1.0),
                    length(right_width),
                ],
                grid_template_rows: vec![
                    length(menu_height),
                    fr(1.0),
                    length(bottom_height),
                    length(status_height),
                ],
                ..Default::default()
            },
            &[
                top_menu_bar,
                tool_rail,
                left_sidebar,
                viewport,
                right_sidebar,
                bottom_strip,
                status_bar,
            ],
        )
        .ok()?;
    taffy
        .compute_layout(
            root,
            Size {
                width: AvailableSpace::Definite(width),
                height: AvailableSpace::Definite(height),
            },
        )
        .ok()?;

    let rect_for = |tree: &TaffyTree<()>, node| -> Option<RectPx> {
        let layout = tree.layout(node).ok()?;
        Some(RectPx {
            x: layout.location.x,
            y: layout.location.y,
            width: layout.size.width,
            height: layout.size.height,
        })
    };
    Some(ShellLayout {
        top_menu_bar: rect_for(&taffy, top_menu_bar)?,
        tool_rail: rect_for(&taffy, tool_rail)?,
        left_sidebar: rect_for(&taffy, left_sidebar)?,
        viewport: rect_for(&taffy, viewport)?,
        right_sidebar: rect_for(&taffy, right_sidebar)?,
        bottom_strip: rect_for(&taffy, bottom_strip)?,
        status_bar: rect_for(&taffy, status_bar)?,
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HitTarget {
    ReviewAction(String),
    AuthoredObject(String),
    FitBoard,
    FitReviewTarget,
    SetWorkspaceTool(WorkspaceTool),
    ReviewPrev,
    ReviewNext,
    ToggleShowAuthored,
    ToggleShowProposed,
    ToggleShowUnrouted,
    ToggleDimUnrelated,
    ToggleLayer(String),
    ToggleSelectedBoardTextMirrored,
    ToggleSelectedBoardTextKeepUpright,
    ToggleSelectedBoardTextBold,
    CycleSelectedBoardTextRenderIntent,
    CycleSelectedBoardTextFamily,
    CycleSelectedBoardTextHAlign,
    CycleSelectedBoardTextVAlign,
    EditSelectedBoardTextRenderIntent,
    EditSelectedBoardTextFamily,
    EditSelectedBoardTextAlignment,
    DecreaseSelectedBoardTextHeight,
    IncreaseSelectedBoardTextHeight,
    RotateSelectedBoardTextCounterClockwise90,
    RotateSelectedBoardTextClockwise90,
    DecreaseSelectedBoardTextLineSpacing,
    IncreaseSelectedBoardTextLineSpacing,
    EditSelectedBoardTextContent,
    EditSelectedBoardTextHeight,
    EditSelectedBoardTextRotation,
    EditSelectedBoardTextLineSpacing,
    TerminalTab,
    TerminalSessionTab(String),
    TerminalSessionNew,
    TerminalSessionRenameActive,
    TerminalSessionRestartActive,
    TerminalSessionDetachActive,
    TerminalSessionCloseActive,
    TerminalActivitySummary(String),
    CheckFinding(String),
    ProductionArtifact(String),
    ProductionArtifactFile(String),
    ProductionOutputJobRun(datum_gui_protocol::TerminalCommandHandoff),
    ProductionTerminalCommand(datum_gui_protocol::TerminalCommandHandoff),
    ArtifactPreviewZoomIn,
    ArtifactPreviewZoomOut,
    ArtifactPreviewReset,
    ArtifactPreviewViewport,
    ToggleArtifactPreviewGeometry,
    ToggleArtifactPreviewDrills,
    MenuTitle(String),
    MenuItem {
        menu: String,
        label: String,
    },
    MarkingMenuItem {
        menu_key: String,
        slot: String,
        label: String,
    },
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
    visible_world_ranges: Vec<Range<u32>>,
    text_runs: Vec<TextRun>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetainedScene {
    world_vertices: Vec<Vertex>,
    world_batches: Vec<RetainedWorldBatch>,
    world_hit_regions: Vec<WorldHitRegion>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RetainedWorldBatch {
    layer_id: Option<String>,
    start: u32,
    len: u32,
}

#[derive(Debug, Clone, PartialEq)]
struct WorldHitRegion {
    target: HitTarget,
    layer_id: Option<String>,
    shape: WorldHitShape,
}

#[derive(Debug, Clone, PartialEq)]
enum WorldHitShape {
    Rect(datum_gui_protocol::RectNm),
    Polyline {
        path: Vec<PointNm>,
        half_width_nm: f32,
    },
    Polygon(Vec<PointNm>),
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

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct TextBufferCacheStats {
    hits: usize,
    misses: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TextPrepareSignature {
    width: u32,
    height: u32,
    runs: Vec<TextPrepareRunKey>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TextPrepareRunKey {
    buffer_index: usize,
    x_bits: u32,
    y_bits: u32,
    color_bits: [u32; 3],
    clip_bounds: Option<RectBits>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct RectBits {
    x_bits: u32,
    y_bits: u32,
    width_bits: u32,
    height_bits: u32,
}

const APP_BG: [f32; 3] = design_tokens::chrome::BG_BASE;
const PANEL_BG: [f32; 3] = design_tokens::chrome::SURFACE_01;
const PANEL_CARD_BG: [f32; 3] = design_tokens::chrome::SURFACE_02;
const PANEL_CARD_BORDER: [f32; 3] = design_tokens::chrome::BORDER_SUBTLE;
const VIEWPORT_BG: [f32; 3] = design_tokens::chrome::CANVAS;
const VIEWPORT_FRAME: [f32; 3] = design_tokens::chrome::BORDER_STRONG;
const BOARD_OUTER_FIELD: [f32; 3] = design_tokens::chrome::CANVAS;
const BOARD_INNER_FIELD: [f32; 3] = design_tokens::chrome::SURFACE_01;
const BOARD_GRID_MAJOR: [f32; 3] = design_tokens::chrome::BORDER_STRONG;
const BOARD_GRID_MINOR: [f32; 3] = design_tokens::chrome::BORDER_SUBTLE;
const BOARD_EDGE: [f32; 3] = design_tokens::content::EDGE;
const TEXT_PRIMARY: [f32; 3] = design_tokens::chrome::TEXT_PRIMARY;
const TEXT_SECONDARY: [f32; 3] = design_tokens::chrome::TEXT_SECONDARY;
const TEXT_MUTED: [f32; 3] = design_tokens::chrome::TEXT_MUTED;
const TEXT_ACCENT: [f32; 3] = design_tokens::chrome::ACCENT;
const TEXT_PANEL_VALUE: [f32; 3] = design_tokens::chrome::TEXT_PRIMARY;
const COMPONENT_BODY: [f32; 3] = design_tokens::chrome::SURFACE_02;
const COMPONENT_BODY_RELATED: [f32; 3] = design_tokens::chrome::SURFACE_03;
const COMPONENT_BODY_SELECTED: [f32; 3] = design_tokens::chrome::ACCENT_TINT;
const COMPONENT_HEADER: [f32; 3] = design_tokens::chrome::CANVAS;
const COMPONENT_OUTLINE: [f32; 3] = design_tokens::chrome::TEXT_SECONDARY;
const COMPONENT_MECHANICAL: [f32; 3] = design_tokens::content::EXCLUSION;
const COMPONENT_MECHANICAL_RELATED: [f32; 3] = design_tokens::chrome::TEXT_SECONDARY;
const COMPONENT_SILK: [f32; 3] = design_tokens::content::SILK_TOP;
const COMPONENT_SILK_RELATED: [f32; 3] = design_tokens::chrome::TEXT_PRIMARY;
const PAD_COPPER: [f32; 3] = design_tokens::content::PAD;
const PAD_COPPER_RELATED: [f32; 3] = design_tokens::content::VIA;
const TOP_MASK_OPENING: [f32; 3] = design_tokens::content::MASK;
const BOTTOM_MASK_OPENING: [f32; 3] = design_tokens::content::MASK;
const TOP_PASTE_OPENING: [f32; 3] = design_tokens::content::PASTE;
const BOTTOM_PASTE_OPENING: [f32; 3] = design_tokens::content::PASTE;
const AUTHOR_BASE: [f32; 3] = design_tokens::content::PAD;
const AUTHOR_RELATED: [f32; 3] = design_tokens::content::VIA;
const AUTHOR_SELECTED: [f32; 3] = design_tokens::content::SELECTION;
const PROPOSAL_BASE: [f32; 3] = design_tokens::chrome::STATUS_WARN;
const PROPOSAL_FOCUS: [f32; 3] = design_tokens::chrome::ACCENT_HOVER;
const PROPOSAL_UNDERLAY: [f32; 3] = design_tokens::chrome::ACCENT_TINT;
const PROPOSAL_OUTER: [f32; 3] = design_tokens::chrome::ACCENT;
const PROPOSAL_CENTERLINE: [f32; 3] = design_tokens::chrome::TEXT_PRIMARY;
const PROPOSAL_ANCHOR_RING: [f32; 3] = design_tokens::chrome::ACCENT_HOVER;
const PROPOSAL_ANCHOR_CORE: [f32; 3] = design_tokens::chrome::TEXT_ON_ACCENT;
const DIAGNOSTIC_BASE: [f32; 3] = design_tokens::chrome::STATUS_INFO;
const DIAGNOSTIC_FOCUS: [f32; 3] = design_tokens::chrome::TEXT_PRIMARY;
const UNROUTED_BASE: [f32; 3] = design_tokens::content::RATSNEST;
const UNROUTED_FOCUS: [f32; 3] = design_tokens::chrome::TEXT_PRIMARY;
const DIAGNOSTIC_UNDERLAY: [f32; 3] = design_tokens::chrome::SURFACE_03;
const AUTHORED_DIM_FACTOR: f32 = 0.82;
const PROCESS_DIM_FACTOR: f32 = 0.88;
const STRUCTURAL_DIM_FACTOR: f32 = 0.74;
const CONTEXT_DIM_FACTOR: f32 = 0.90;
const REVIEW_ROW_ACTIVE_BG: [f32; 3] = design_tokens::chrome::ACCENT_TINT;
const REVIEW_ROW_BADGE: [f32; 3] = design_tokens::chrome::SURFACE_03;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LayerFamily {
    TopCopper,
    InnerCopper,
    BottomCopper,
    Unknown,
}

/// Declared render-stack policy (`docs/gui/M7_RENDER_LAYER_DISCIPLINE_MEMO.md`,
/// 2026-04-16 rule): layer type group first, then back-to-front side, with
/// scene `render_order` only as a stable in-stage tie-breaker. Declaration
/// order IS the draw order; `render_stage_priority` derives from it. Do not
/// reintroduce a second ordering encoding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum RenderStage {
    BottomCopper,
    InnerCopper,
    TopCopper,
    BottomMask,
    TopMask,
    BottomPaste,
    TopPaste,
    BottomSilk,
    TopSilk,
    Mechanical,
    Edge,
    Other,
}

/// The shared post-copper stage walk, in declared draw order. Both retained
/// scene assembly and board-graphics emission iterate this one list.
const POST_COPPER_STAGES: [RenderStage; 8] = [
    RenderStage::BottomMask,
    RenderStage::TopMask,
    RenderStage::BottomPaste,
    RenderStage::TopPaste,
    RenderStage::BottomSilk,
    RenderStage::TopSilk,
    RenderStage::Mechanical,
    RenderStage::Edge,
];

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
        let scale = scale_factor.max(0.01);
        let layout = ShellLayout::for_surface(width, height, scale, dock_height_for_state(state));
        let mut panel_quads = Vec::new();
        let mut viewport_underlay_quads = Vec::new();
        let mut viewport_overlay_quads = Vec::new();
        let mut text_runs = Vec::new();
        let mut hit_regions = Vec::new();
        let scene_viewport = layout.scene_viewport();

        panel_quads.push(Quad::from_rect(layout.top_menu_bar, PANEL_BG));
        panel_quads.push(Quad::from_rect(layout.tool_rail, PANEL_CARD_BG));
        panel_quads.push(Quad::from_rect(layout.left_sidebar, PANEL_BG));
        panel_quads.push(Quad::from_rect(layout.right_sidebar, PANEL_BG));
        panel_quads.push(Quad::from_rect(layout.bottom_strip, PANEL_BG));
        panel_quads.push(Quad::from_rect(layout.status_bar, PANEL_CARD_BG));
        viewport_underlay_quads.push(Quad::from_rect(layout.viewport, VIEWPORT_BG));

        render_phase1_shell_chrome(state, &layout, &mut panel_quads, &mut text_runs);
        render_menu_bar(
            state,
            &layout,
            &mut panel_quads,
            &mut text_runs,
            &mut hit_regions,
        );
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
        render_marking_menu(
            state,
            &layout,
            &mut panel_quads,
            &mut text_runs,
            &mut hit_regions,
        );
        if (scale - 1.0).abs() > f32::EPSILON {
            scale_text_run_sizes(&mut text_runs, scale);
        }
        let panel_vertices = quads_to_vertices(&panel_quads);
        let viewport_underlay_vertices = quads_to_vertices(&viewport_underlay_quads);
        let viewport_overlay_vertices = quads_to_vertices(&viewport_overlay_quads);
        let visible_world_ranges = retained_scene.visible_world_ranges(state);

        Self {
            layout,
            hit_regions,
            scene_viewport,
            scene_bounds: state.scene.bounds.clone(),
            camera,
            panel_vertices,
            viewport_underlay_vertices,
            viewport_overlay_vertices,
            visible_world_ranges,
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

    fn visible_world_ranges(&self) -> &[Range<u32>] {
        &self.visible_world_ranges
    }
}

impl RetainedScene {
    pub fn from_workspace(state: &ReviewWorkspaceState, width: u32, height: u32) -> Self {
        Self::from_workspace_for_surface(state, width, height, 1.0)
    }

    pub fn from_workspace_for_surface(
        state: &ReviewWorkspaceState,
        width: u32,
        height: u32,
        scale_factor: f32,
    ) -> Self {
        let started = std::time::Instant::now();
        let layout =
            ShellLayout::for_surface(width, height, scale_factor, dock_height_for_state(state));
        let scene_viewport = layout.scene_viewport();
        let board_field = inset_rect(scene_viewport, 10.0, 10.0, 10.0, 10.0);
        let reference_projection = Projection::new(
            board_field,
            &state.scene.bounds,
            CameraState::fit_to_bounds(&state.scene.bounds),
        );
        let mut world_quads = Vec::new();
        let mut world_batches = Vec::new();
        let mut world_hit_regions = Vec::new();
        let geometry_started = std::time::Instant::now();
        push_retained_scene_geometry(&mut world_quads, &state.scene, &reference_projection, state);
        if !world_quads.is_empty() {
            world_batches.push(RetainedWorldBatch {
                layer_id: None,
                start: 0,
                len: (world_quads.len() * 6) as u32,
            });
        }
        let board_graphics_started = std::time::Instant::now();
        let board_graphics_before = world_quads.len();
        push_retained_board_text_geometry_batches(
            &mut world_quads,
            &mut world_batches,
            &state.scene,
            &reference_projection,
            state,
        );
        push_retained_board_graphic_batches(
            &mut world_quads,
            &mut world_batches,
            &state.scene,
            &reference_projection,
            state,
        );
        trace_render_timing(format!(
            "retained text+board_graphics batches={}ms/{}q",
            board_graphics_started.elapsed().as_millis(),
            world_quads.len().saturating_sub(board_graphics_before)
        ));
        let geometry_elapsed = geometry_started.elapsed();
        let hits_started = std::time::Instant::now();
        push_retained_world_hit_regions(&mut world_hit_regions, &state.scene, state);
        let hits_elapsed = hits_started.elapsed();
        let vertex_started = std::time::Instant::now();
        let world_vertices = quads_to_vertices(&world_quads);
        let vertex_elapsed = vertex_started.elapsed();
        trace_render_timing(format!(
            "retained total={}ms geometry={}ms hits={}ms vertices={}ms quads={} vertices={} hit_regions={}",
            started.elapsed().as_millis(),
            geometry_elapsed.as_millis(),
            hits_elapsed.as_millis(),
            vertex_elapsed.as_millis(),
            world_quads.len(),
            world_vertices.len(),
            world_hit_regions.len()
        ));
        Self {
            world_vertices,
            world_batches,
            world_hit_regions,
        }
    }

    pub fn world_vertices(&self) -> &[Vertex] {
        &self.world_vertices
    }

    fn visible_world_ranges(&self, state: &ReviewWorkspaceState) -> Vec<Range<u32>> {
        if !authored_visible(state) {
            return Vec::new();
        }
        self.world_batches
            .iter()
            .filter(|batch| {
                batch
                    .layer_id
                    .as_deref()
                    .is_none_or(|layer_id| layer_visible(state, layer_id))
            })
            .map(|batch| batch.start..batch.start + batch.len)
            .collect()
    }

    pub fn hit_test_authored_world(
        &self,
        point: PointNm,
        state: &ReviewWorkspaceState,
    ) -> Option<&HitTarget> {
        if !authored_visible(state) {
            return None;
        }
        self.world_hit_regions
            .iter()
            .rev()
            .find(|region| {
                region
                    .layer_id
                    .as_deref()
                    .is_none_or(|layer_id| layer_visible(state, layer_id))
                    && region.shape.contains(point)
            })
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
            Self::Polygon(path) => point_in_polygon_world(path, point),
            Self::Circle { center, radius_nm } => {
                let dx = point.x as f32 - center.x as f32;
                let dy = point.y as f32 - center.y as f32;
                dx * dx + dy * dy <= radius_nm * radius_nm
            }
        }
    }
}

fn dock_height_for_state(state: &ReviewWorkspaceState) -> Option<u32> {
    if state.ui.active_dock_tab.is_some() {
        Some(state.ui.dock_height_px)
    } else {
        None
    }
}

fn render_phase1_shell_chrome(
    state: &ReviewWorkspaceState,
    layout: &ShellLayout,
    panel_quads: &mut Vec<Quad>,
    text_runs: &mut Vec<TextRun>,
) {
    push_rect_border(panel_quads, layout.top_menu_bar, PANEL_CARD_BORDER, 1.0);
    draw_text(
        "Datum EDA",
        layout.top_menu_bar.x + design_tokens::spacing::SP_04,
        layout.top_menu_bar.y + design_tokens::spacing::SP_03,
        design_tokens::typography::BODY_SIZE,
        TEXT_PRIMARY,
        TextFace::Ui,
        text_runs,
    );
    draw_text(
        "Board",
        layout.top_menu_bar.x + design_tokens::spacing::SP_11,
        layout.top_menu_bar.y + design_tokens::spacing::SP_03,
        design_tokens::typography::BODY_SIZE,
        TEXT_SECONDARY,
        TextFace::Ui,
        text_runs,
    );
    draw_text(
        &truncate_text(&state.scene.project_name, 30),
        (layout.top_menu_bar.x + layout.top_menu_bar.width - 220.0).max(layout.top_menu_bar.x),
        layout.top_menu_bar.y + design_tokens::spacing::SP_03,
        design_tokens::typography::DATA_SIZE,
        TEXT_MUTED,
        TextFace::Mono,
        text_runs,
    );

    push_rect_border(panel_quads, layout.tool_rail, PANEL_CARD_BORDER, 1.0);
    let rail_tools = [("S", true), ("Z", false), ("H", false), ("L", false)];
    let mut y = layout.tool_rail.y + design_tokens::spacing::SP_04;
    for (label, active) in rail_tools {
        let rect = RectPx {
            x: layout.tool_rail.x + design_tokens::spacing::SP_03,
            y,
            width: (layout.tool_rail.width - design_tokens::spacing::SP_04).max(1.0),
            height: design_tokens::spacing::SP_07,
        };
        panel_quads.push(Quad::from_rect(
            rect,
            if active {
                REVIEW_ROW_ACTIVE_BG
            } else {
                PANEL_BG
            },
        ));
        push_rect_border(
            panel_quads,
            rect,
            if active {
                TEXT_ACCENT
            } else {
                PANEL_CARD_BORDER
            },
            1.0,
        );
        draw_text(
            label,
            rect.x + design_tokens::spacing::SP_04,
            rect.y + design_tokens::spacing::SP_03,
            design_tokens::typography::BODY_SIZE,
            if active { TEXT_PRIMARY } else { TEXT_MUTED },
            TextFace::Ui,
            text_runs,
        );
        y += design_tokens::spacing::SP_08;
    }

    push_rect_border(panel_quads, layout.status_bar, PANEL_CARD_BORDER, 1.0);
    let selection = match &state.selection {
        SelectionTarget::None => "none".to_string(),
        SelectionTarget::ReviewAction(id)
        | SelectionTarget::AuthoredObject(id)
        | SelectionTarget::CheckFinding(id) => truncate_text(id, 34),
    };
    let status = format!(
        "focus Board    tool {}    selection {}    layers {}",
        workspace_tool_label(state.tool),
        selection,
        state.scene.layers.len()
    );
    draw_text(
        &status,
        layout.status_bar.x + design_tokens::spacing::SP_04,
        layout.status_bar.y + design_tokens::spacing::SP_02 + 1.0,
        design_tokens::typography::DATA_SIZE,
        TEXT_SECONDARY,
        TextFace::Mono,
        text_runs,
    );
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
    let viewport_title = if is_schematic_scene(&state.scene) {
        format!("SCHEMATIC {}", state.scene.board_name.to_uppercase())
    } else {
        state.scene.board_name.to_uppercase()
    };
    draw_text(
        &truncate_text(&viewport_title, 28),
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
        SelectionTarget::CheckFinding(id) => truncate_text(&suffix_id(id).to_uppercase(), 12),
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

fn is_schematic_scene(scene: &BoardReviewSceneV1) -> bool {
    scene.kind == "schematic_review_scene"
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
    scene
        .net_display
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

fn proposal_preview_affected_ids(state: &ReviewWorkspaceState) -> Vec<&str> {
    state
        .production
        .proposals
        .iter()
        .filter_map(|proposal| proposal.preview.as_ref())
        .flat_map(|preview| preview.affected_objects.iter().map(String::as_str))
        .collect()
}

fn source_object_matches_preview(
    affected_ids: &[&str],
    object_id: &str,
    source_object_uuid: &str,
) -> bool {
    affected_ids
        .iter()
        .any(|affected| *affected == object_id || *affected == source_object_uuid)
}

fn component_matches_preview(
    component_uuid: &str,
    scene: &BoardReviewSceneV1,
    affected_ids: &[&str],
) -> bool {
    scene.components.iter().any(|component| {
        component.component_uuid == component_uuid
            && source_object_matches_preview(
                affected_ids,
                &component.object_id,
                &component.source_object_uuid,
            )
    })
}

fn component_object_id_for_uuid<'a>(
    scene: &'a BoardReviewSceneV1,
    component_uuid: &str,
) -> Option<&'a str> {
    scene.components.iter().find_map(|component| {
        (component.component_uuid == component_uuid).then_some(component.object_id.as_str())
    })
}

/// Retained authored-board geometry pass.
///
/// Contract (`M7-REN-006`, `docs/gui/M7_RENDER_LAYER_DISCIPLINE_MEMO.md`):
/// layer/material semantics are primary — layer ownership decides visibility,
/// the layer's material decides base appearance, and stage order follows the
/// declared `RenderStage` policy. Primitive class only refines stroke/fill.
///
/// Bounded exceptions (explicit, product-justified; do not grow this list
/// without a memo note):
/// - through-hole pads: drawn in a dedicated post-layer pass because their
///   copper spans multiple layers and must follow the visible-copper rule;
/// - vias: a distinct geometry family (annulus + hole), though their color
///   inherits the visible copper layer's material;
/// - board outline / `board_graphics` Edge overlay: the board-boundary view
///   is a product-level overlay on top of the authored stage walk;
/// - selection/hover/review emphasis: interaction-state styling deliberately
///   overrides material color for the owned object only;
/// - unknown-layer fallback appearance: deliberately divergent so unresolved
///   layer identity stays visible (see `resolve_layer_appearance_with_scene`).
fn push_retained_scene_geometry(
    out: &mut Vec<Quad>,
    scene: &BoardReviewSceneV1,
    reference_projection: &Projection,
    state: &ReviewWorkspaceState,
) {
    let active_move_component_uuid: Option<String> = None;
    let sl = &scene.layers;
    let preview_affected_ids = proposal_preview_affected_ids(state);
    let layer_app = |id: &str| resolve_layer_appearance_with_scene(Some(id), sl);
    // Render copper in physical stack order first; later stages (paste/mask/silk/mechanical/edge)
    // are handled by explicit render-stage grouping below.
    let copper_started = std::time::Instant::now();
    let copper_before = out.len();
    for pass_priority in [0u32, 1, 2] {
        for zone in &scene.zones {
            if copper_pass_priority_for_layer(&zone.layer_id, sl) != Some(pass_priority) {
                continue;
            }
            if !authored_visible(state) || !layer_visible(state, &zone.layer_id) {
                continue;
            }
            let related = zone_matches_active_action(zone, state)
                || source_object_matches_preview(
                    &preview_affected_ids,
                    &zone.object_id,
                    &zone.source_object_uuid,
                );
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
            let related = track_matches_active_action(track, state)
                || source_object_matches_preview(
                    &preview_affected_ids,
                    &track.object_id,
                    &track.source_object_uuid,
                );
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
            let track_width_nm = (track.width_nm as f32).max(world_stroke_nm(
                if selected { 3.0 } else { 2.0 },
                reference_projection,
            ));
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
                || component_is_selection_related(&pad.component_uuid, scene, state)
                || source_object_matches_preview(
                    &preview_affected_ids,
                    &pad.object_id,
                    &pad.source_object_uuid,
                )
                || component_matches_preview(&pad.component_uuid, scene, &preview_affected_ids);
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
            let related = via_matches_active_action(via, state)
                || source_object_matches_preview(
                    &preview_affected_ids,
                    &via.object_id,
                    &via.source_object_uuid,
                );
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
    trace_retained_stage("copper", copper_started, copper_before, out.len());
    let mechanical_graphics: Vec<_> = scene
        .component_graphics
        .iter()
        .filter(|graphic| {
            graphic.render_role == "component_mechanical"
                && active_move_component_uuid.as_deref() != Some(graphic.component_uuid.as_str())
        })
        .collect();
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
    let silkscreen_graphics: Vec<_> = scene
        .component_graphics
        .iter()
        .filter(|graphic| {
            graphic.render_role == "component_silkscreen"
                && active_move_component_uuid.as_deref() != Some(graphic.component_uuid.as_str())
        })
        .collect();
    let post_started = std::time::Instant::now();
    let post_before = out.len();
    let mut process_pad_elapsed = std::time::Duration::ZERO;
    let mut mechanical_elapsed = std::time::Duration::ZERO;
    let mut silkscreen_elapsed = std::time::Duration::ZERO;
    let board_graphics_elapsed = std::time::Duration::ZERO;
    let mut process_pad_quads = 0usize;
    let mut mechanical_quads = 0usize;
    let mut silkscreen_quads = 0usize;
    let board_graphics_quads = 0usize;
    for stage in POST_COPPER_STAGES {
        let process_before = out.len();
        let process_started = std::time::Instant::now();
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
                let related = pad_matches_active_action(pad, state)
                    || source_object_matches_preview(
                        &preview_affected_ids,
                        &pad.object_id,
                        &pad.source_object_uuid,
                    )
                    || component_matches_preview(&pad.component_uuid, scene, &preview_affected_ids);
                let hovered = is_hovered(state, &pad.object_id);
                let dimmed = dim_unrelated_active(state) && !active && !related && !hovered;
                let membership = match kind {
                    PadProcessLayerKind::Mask => &pad.mask_layer_ids,
                    PadProcessLayerKind::Paste => &pad.paste_layer_ids,
                };
                if !membership.iter().any(|member| member == layer_id) {
                    continue;
                }
                let derived = derived_process_pad(pad, layer_id, *kind, &scene.pad_expansion_setup);
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
        process_pad_elapsed += process_started.elapsed();
        process_pad_quads += out.len().saturating_sub(process_before);
        let mechanical_before = out.len();
        let mechanical_started = std::time::Instant::now();
        for graphic in mechanical_graphics.iter().filter(|graphic| {
            graphic_render_stage(graphic.layer_id.as_deref(), sl, RenderStage::Mechanical) == stage
        }) {
            if !authored_visible(state) {
                continue;
            }
            if let Some(lid) = graphic.layer_id.as_deref()
                && !layer_visible(state, lid)
            {
                continue;
            }
            let selected_body_graphic_id =
                selected_component_body_graphic_id(scene, &graphic.component_uuid);
            if selected_body_graphic_id.is_some_and(|id| id == graphic.graphic_id) {
                continue;
            }
            let related = component_graphic_matches_active_action(graphic, scene, state)
                || component_is_selection_related(&graphic.component_uuid, scene, state)
                || component_matches_preview(&graphic.component_uuid, scene, &preview_affected_ids);
            let selected_component =
                matches!(
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
        mechanical_elapsed += mechanical_started.elapsed();
        mechanical_quads += out.len().saturating_sub(mechanical_before);
        let silkscreen_before = out.len();
        let silkscreen_started = std::time::Instant::now();
        for graphic in silkscreen_graphics.iter().filter(|graphic| {
            graphic_render_stage(graphic.layer_id.as_deref(), sl, RenderStage::TopSilk) == stage
        }) {
            if !authored_visible(state) {
                continue;
            }
            if let Some(lid) = graphic.layer_id.as_deref()
                && !layer_visible(state, lid)
            {
                continue;
            }
            let related = component_graphic_matches_active_action(graphic, scene, state)
                || component_is_selection_related(&graphic.component_uuid, scene, state)
                || component_matches_preview(&graphic.component_uuid, scene, &preview_affected_ids);
            let selected =
                matches!(
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
        silkscreen_elapsed += silkscreen_started.elapsed();
        silkscreen_quads += out.len().saturating_sub(silkscreen_before);
    }
    trace_retained_stage("post-copper", post_started, post_before, out.len());
    trace_render_timing(format!(
        "retained detail process_pads={}ms/{}q mechanical={}ms/{}q component_silk={}ms/{}q board_graphics={}ms/{}q",
        process_pad_elapsed.as_millis(),
        process_pad_quads,
        mechanical_elapsed.as_millis(),
        mechanical_quads,
        silkscreen_elapsed.as_millis(),
        silkscreen_quads,
        board_graphics_elapsed.as_millis(),
        board_graphics_quads
    ));
    let active_started = std::time::Instant::now();
    let active_before = out.len();
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
    trace_retained_stage("active-component", active_started, active_before, out.len());
    let unrouted_started = std::time::Instant::now();
    let unrouted_before = out.len();
    if unrouted_visible(state) {
        let mut unrouted_batches: Vec<(Vec<PointNm>, [f32; 3], [f32; 3], f32, f32, f32, f32)> =
            Vec::new();
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
            let under_color =
                mix_color(BOARD_OUTER_FIELD, color, if related { 0.28 } else { 0.22 });
            let width_px = if related { 1.55 } else { 1.2 };
            let width_nm = world_stroke_nm(width_px, reference_projection).max(1.0);
            let under_width_nm = world_stroke_nm(
                width_px + if related { 0.9 } else { 0.7 },
                reference_projection,
            )
            .max(width_nm + 1.0);
            let endpoint_radius_nm =
                world_stroke_nm(if related { 1.15 } else { 0.95 }, reference_projection).max(1.0);
            let endpoint_under_radius_nm = (endpoint_radius_nm
                + ((under_width_nm - width_nm) * 0.5))
                .max(endpoint_radius_nm + 0.5);
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
        for (
            path,
            _color,
            under_color,
            _width_nm,
            under_width_nm,
            _endpoint_radius_nm,
            _endpoint_under_radius_nm,
        ) in &unrouted_batches
        {
            push_world_polyline_segments_capped(out, path, *under_width_nm, *under_color);
        }
        for (
            path,
            _color,
            under_color,
            _width_nm,
            _under_width_nm,
            _endpoint_radius_nm,
            endpoint_under_radius_nm,
        ) in &unrouted_batches
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
        for (
            path,
            color,
            _under_color,
            width_nm,
            _under_width_nm,
            _endpoint_radius_nm,
            _endpoint_under_radius_nm,
        ) in &unrouted_batches
        {
            push_world_polyline_segments_capped(out, path, *width_nm, *color);
        }
        for (
            path,
            color,
            _under_color,
            _width_nm,
            _under_width_nm,
            endpoint_radius_nm,
            _endpoint_under_radius_nm,
        ) in &unrouted_batches
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
    trace_retained_stage("unrouted", unrouted_started, unrouted_before, out.len());
    let outline_started = std::time::Instant::now();
    let outline_before = out.len();
    trace_retained_stage("outline", outline_started, outline_before, out.len());
}

fn push_retained_board_graphic_batches(
    out: &mut Vec<Quad>,
    batches: &mut Vec<RetainedWorldBatch>,
    scene: &BoardReviewSceneV1,
    reference_projection: &Projection,
    state: &ReviewWorkspaceState,
) {
    if !authored_visible(state) {
        return;
    }
    let sl = &scene.layers;
    out.reserve(
        scene
            .board_graphics
            .len()
            .saturating_add(scene.outline.len() * 32),
    );
    let trace_graphics = std::env::var_os("DATUM_TRACE_GRAPHICS").is_some();

    for stage in POST_COPPER_STAGES {
        let mut active_layer: Option<String> = None;
        let mut active_color = [0.0, 0.0, 0.0];
        let mut active_start = out.len();
        for gfx in scene
            .board_graphics
            .iter()
            .filter(|gfx| render_stage_for_layer(&gfx.layer_id, sl) == stage)
        {
            if active_layer.as_deref() != Some(gfx.layer_id.as_str()) {
                finish_retained_quad_batch(batches, active_layer.take(), active_start, out.len());
                active_layer = Some(gfx.layer_id.clone());
                active_color =
                    board_graphic_world_color(&gfx.layer_id, sl, dim_unrelated_active(state));
                active_start = out.len();
            }
            if trace_graphics {
                let graphic_started = std::time::Instant::now();
                let graphic_before = out.len();
                push_board_graphic_primitive_world(out, gfx, active_color, reference_projection);
                trace_graphic_timing(
                    gfx,
                    graphic_started,
                    out.len().saturating_sub(graphic_before),
                );
            } else {
                push_board_graphic_primitive_world(out, gfx, active_color, reference_projection);
            }
        }
        finish_retained_quad_batch(batches, active_layer.take(), active_start, out.len());
        let mut outline_layer: Option<String> = None;
        let mut outline_start = out.len();
        for outline in scene
            .outline
            .iter()
            .filter(|outline| render_stage_for_layer(&outline.layer_id, sl) == stage)
        {
            if outline_layer.as_deref() != Some(outline.layer_id.as_str()) {
                finish_retained_quad_batch(batches, outline_layer.take(), outline_start, out.len());
                outline_layer = Some(outline.layer_id.clone());
                outline_start = out.len();
            }
            push_world_polyline_segments_capped(
                out,
                &outline.path,
                world_stroke_nm(1.6, reference_projection),
                board_surface_color(BoardSurfaceRole::Edge),
            );
        }
        finish_retained_quad_batch(batches, outline_layer.take(), outline_start, out.len());
    }
}

fn push_retained_board_text_geometry_batches(
    out: &mut Vec<Quad>,
    batches: &mut Vec<RetainedWorldBatch>,
    scene: &BoardReviewSceneV1,
    reference_projection: &Projection,
    state: &ReviewWorkspaceState,
) {
    if !authored_visible(state) {
        return;
    }
    let sl = &scene.layers;
    let dimmed = dim_unrelated_active(state);
    let glyph_mesh_assets: BTreeMap<GlyphMeshHandlePrimitive, &GlyphMeshAssetPrimitive> = scene
        .glyph_mesh_assets
        .iter()
        .map(|asset| (asset.handle, asset))
        .collect();
    for stage in POST_COPPER_STAGES {
        let mut active_layer: Option<String> = None;
        let mut active_start = out.len();
        let mut active_color = [0.0, 0.0, 0.0];
        for text_geometry in scene
            .board_text_geometries
            .iter()
            .filter(|text| render_stage_for_layer(&text.layer_id, sl) == stage)
        {
            if !layer_visible(state, &text_geometry.layer_id) {
                continue;
            }
            let text_color = board_graphic_world_color(&text_geometry.layer_id, sl, dimmed);
            if active_layer.as_deref() != Some(text_geometry.layer_id.as_str())
                || active_color != text_color
            {
                finish_retained_quad_batch(batches, active_layer.take(), active_start, out.len());
                active_layer = Some(text_geometry.layer_id.clone());
                active_color = text_color;
                active_start = out.len();
            }
            push_board_text_geometry_world(
                out,
                text_geometry,
                &glyph_mesh_assets,
                active_color,
                reference_projection,
            );
        }
        finish_retained_quad_batch(batches, active_layer.take(), active_start, out.len());
    }
}

fn finish_retained_quad_batch(
    batches: &mut Vec<RetainedWorldBatch>,
    layer_id: Option<String>,
    start_quads: usize,
    end_quads: usize,
) {
    if end_quads <= start_quads {
        return;
    }
    batches.push(RetainedWorldBatch {
        layer_id,
        start: (start_quads * 6) as u32,
        len: ((end_quads - start_quads) * 6) as u32,
    });
}

fn trace_retained_stage(
    name: &str,
    started: std::time::Instant,
    before_quads: usize,
    after_quads: usize,
) {
    trace_render_timing(format!(
        "retained stage {name} {}ms +{}q total={}q",
        started.elapsed().as_millis(),
        after_quads.saturating_sub(before_quads),
        after_quads
    ));
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
            layer_id: Some(track.layer_id.clone()),
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
            layer_id: None,
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
                layer_id: Some(component.placement_layer.clone()),
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
            layer_id: Some(component.placement_layer.clone()),
            shape: WorldHitShape::Rect(hit_rect),
        });
    }
    for pad in &scene.pads {
        let pad_visible = pad_visible_on_any_copper_layer(state, pad);
        if !pad_visible {
            continue;
        }
        out.push(WorldHitRegion {
            target: HitTarget::AuthoredObject(pad.object_id.clone()),
            layer_id: None,
            shape: WorldHitShape::Rect(pad.bounds),
        });
    }
    for zone in &scene.zones {
        if !layer_visible(state, &zone.layer_id) || zone.polygon.len() < 3 {
            continue;
        }
        out.push(WorldHitRegion {
            target: HitTarget::AuthoredObject(zone.object_id.clone()),
            layer_id: Some(zone.layer_id.clone()),
            shape: WorldHitShape::Polygon(zone.polygon.clone()),
        });
    }
    for graphic in &scene.component_graphics {
        let Some(target_id) = component_object_id_for_uuid(scene, &graphic.component_uuid) else {
            continue;
        };
        if let Some(layer_id) = graphic.layer_id.as_deref()
            && !layer_visible(state, layer_id)
        {
            continue;
        }
        if graphic.layer_id.as_deref().is_some_and(|layer_id| {
            scene
                .layers
                .iter()
                .find(|layer| layer.layer_id == layer_id)
                .is_some_and(|layer| layer.name == "Edge.Cuts")
        }) {
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
                        layer_id: graphic.layer_id.clone(),
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
                    layer_id: graphic.layer_id.clone(),
                    shape: WorldHitShape::Polyline {
                        path: graphic.path.clone(),
                        half_width_nm: (width as f32 * 0.5).max(180_000.0),
                    },
                });
            }
        }
    }
    for text in &scene.board_texts {
        if !layer_visible(state, &text.layer_id) {
            continue;
        }
        out.push(WorldHitRegion {
            target: HitTarget::AuthoredObject(text.object_id.clone()),
            layer_id: Some(text.layer_id.clone()),
            shape: WorldHitShape::Rect(board_text_hit_rect(text)),
        });
    }
    for gfx in &scene.board_graphics {
        if gfx.object_id.starts_with("board-text:") {
            continue;
        }
        if !layer_visible(state, &gfx.layer_id) {
            continue;
        }
        let width = gfx.width_nm.unwrap_or(100_000);
        out.push(WorldHitRegion {
            target: HitTarget::AuthoredObject(gfx.object_id.clone()),
            layer_id: Some(gfx.layer_id.clone()),
            shape: WorldHitShape::Polyline {
                path: gfx.path.clone(),
                half_width_nm: (width as f32 * 0.5).max(150_000.0),
            },
        });
    }
    for outline in &scene.outline {
        if !layer_visible(state, &outline.layer_id) {
            continue;
        }
        out.push(WorldHitRegion {
            target: HitTarget::AuthoredObject(outline.object_id.clone()),
            layer_id: Some(outline.layer_id.clone()),
            shape: WorldHitShape::Polyline {
                path: outline.path.clone(),
                half_width_nm: 300_000.0,
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
    let preview_affected_ids = proposal_preview_affected_ids(state);
    push_lightweight_selection_overlay(out, scene, state, &projection);
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
            || component_is_selection_related(&component.component_uuid, scene, state)
            || source_object_matches_preview(
                &preview_affected_ids,
                &component.object_id,
                &component.source_object_uuid,
            );
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
        let label_x =
            label_rect.x + (label_rect.width * 0.5) - (label_text.len() as f32 * label_size * 0.32);
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
        }) || component_is_selection_related(&text.component_uuid, scene, state)
            || component_matches_preview(&text.component_uuid, scene, &preview_affected_ids);
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
        let selected_pad =
            matches!(&state.selection, SelectionTarget::AuthoredObject(id) if id == &pad.object_id);
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
            let rects = push_overlay(out, overlay, &projection, color, selected, false);
            for rect in rects {
                hit_regions.push(HitRegion {
                    target: HitTarget::ReviewAction(overlay.proposal_action_id.clone()),
                    rect,
                });
            }
        }
        for overlay in production_proposal_overlay_primitives(state) {
            if !overlay
                .layer_id
                .as_deref()
                .is_none_or(|layer_id| layer_visible(state, layer_id))
            {
                continue;
            }
            let rects = push_overlay(out, &overlay, &projection, PROPOSAL_BASE, false, false);
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
        // Diagnostic emphasis marks where the evidence span starts and ends.
        // Interior vertices stay unmarked: per-vertex dots read as generic
        // path-editing handles, which M7-REN-003 forbids over proposed copper.
        if let (Some(first), Some(last)) = (review.path.first(), review.path.last()) {
            push_points(
                out,
                &[*first, *last],
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
}

fn production_proposal_overlay_primitives(
    state: &ReviewWorkspaceState,
) -> Vec<ProposalOverlayPrimitive> {
    state
        .production
        .proposals
        .iter()
        .filter_map(|proposal| {
            proposal
                .preview
                .as_ref()
                .map(|preview| (proposal.proposal_id.as_str(), preview))
        })
        .flat_map(|(proposal_id, preview)| {
            preview
                .render_deltas
                .iter()
                .enumerate()
                .filter(|(_, delta)| {
                    (delta.primitive_kind == "track_path" && delta.path.len() >= 2)
                        || (delta.primitive_kind == "via" && !delta.path.is_empty())
                })
                .map(move |(index, delta)| ProposalOverlayPrimitive {
                    overlay_id: format!("proposal:{proposal_id}:preview:{index}"),
                    primitive_kind: delta.primitive_kind.clone(),
                    proposal_action_id: proposal_id.to_string(),
                    layer_id: Some(delta.layer_id.clone()),
                    render_role: "proposed_preview".to_string(),
                    width_nm: Some(delta.width_nm),
                    drill_nm: delta.drill_nm,
                    diameter_nm: delta.diameter_nm,
                    path: delta.path.clone(),
                })
        })
        .collect()
}

fn push_lightweight_selection_overlay(
    out: &mut Vec<Quad>,
    scene: &BoardReviewSceneV1,
    state: &ReviewWorkspaceState,
    projection: &Projection,
) {
    let SelectionTarget::AuthoredObject(object_id) = &state.selection else {
        return;
    };
    if let Some(text) = scene
        .board_texts
        .iter()
        .find(|text| &text.object_id == object_id)
    {
        if !authored_visible(state) || !layer_visible(state, &text.layer_id) {
            return;
        }
        let rect = project_rect(board_text_hit_rect(text), projection);
        let halo = RectPx {
            x: rect.x - 4.0,
            y: rect.y - 4.0,
            width: rect.width + 8.0,
            height: rect.height + 8.0,
        };
        push_rect_border(out, halo, selected_silk_color(COMPONENT_SILK), 2.0);
        return;
    }
    if let Some(outline) = scene
        .outline
        .iter()
        .find(|outline| &outline.object_id == object_id)
    {
        if !authored_visible(state) || !layer_visible(state, &outline.layer_id) {
            return;
        }
        push_polyline_segments(
            out,
            &outline.path,
            projection,
            selected_mechanical_color(board_surface_color(BoardSurfaceRole::Edge)),
            3.0,
        );
        return;
    }
    if let Some(graphic) = scene
        .board_graphics
        .iter()
        .find(|graphic| &graphic.object_id == object_id)
    {
        if !authored_visible(state) || !layer_visible(state, &graphic.layer_id) {
            return;
        }
        push_polyline_segments(
            out,
            &graphic.path,
            projection,
            selected_mechanical_color(board_graphic_world_color(
                &graphic.layer_id,
                &scene.layers,
                false,
            )),
            3.0,
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

fn point_in_polygon_world(path: &[PointNm], point: PointNm) -> bool {
    if path.len() < 3 {
        return false;
    }
    let mut inside = false;
    let px = point.x as f64;
    let py = point.y as f64;
    let mut j = path.len() - 1;
    for i in 0..path.len() {
        let xi = path[i].x as f64;
        let yi = path[i].y as f64;
        let xj = path[j].x as f64;
        let yj = path[j].y as f64;
        let crosses_y = (yi > py) != (yj > py);
        if crosses_y {
            let denom = yj - yi;
            if denom.abs() <= f64::EPSILON {
                j = i;
                continue;
            }
            let x_at_y = (xj - xi) * (py - yi) / denom + xi;
            if px < x_at_y {
                inside = !inside;
            }
        }
        j = i;
    }
    inside
}

fn board_text_hit_rect(text: &BoardTextPrimitive) -> datum_gui_protocol::RectNm {
    let lines: Vec<&str> = text.text.lines().collect();
    let line_count = lines.len().max(1) as f64;
    let max_chars = lines
        .iter()
        .map(|line| line.chars().count())
        .max()
        .unwrap_or_else(|| text.text.chars().count())
        .max(1) as f64;
    let height = text.height_nm.max(1) as f64;
    let line_spacing = (text.line_spacing_ratio_ppm.max(1) as f64) / 1_000_000.0;
    let width_nm = (max_chars * height * 0.72).max(height * 0.5);
    let height_nm = (height + (line_count - 1.0) * height * line_spacing).max(height);
    let x0 = match text.h_align.as_str() {
        "center" => -width_nm * 0.5,
        "right" => -width_nm,
        _ => 0.0,
    };
    let y0 = match text.v_align.as_str() {
        "center" => -height_nm * 0.5,
        "top" => 0.0,
        _ => -height_nm,
    };
    let x1 = x0 + width_nm;
    let y1 = y0 + height_nm;
    let theta = (text.rotation_degrees as f64).to_radians();
    let (sin_t, cos_t) = theta.sin_cos();
    let mut min_x = f64::INFINITY;
    let mut min_y = f64::INFINITY;
    let mut max_x = f64::NEG_INFINITY;
    let mut max_y = f64::NEG_INFINITY;
    for (x, y) in [(x0, y0), (x1, y0), (x1, y1), (x0, y1)] {
        let rx = text.position.x as f64 + x * cos_t - y * sin_t;
        let ry = text.position.y as f64 + x * sin_t + y * cos_t;
        min_x = min_x.min(rx);
        min_y = min_y.min(ry);
        max_x = max_x.max(rx);
        max_y = max_y.max(ry);
    }
    let padding = (height * 0.25).max(250_000.0);
    datum_gui_protocol::RectNm {
        min_x: (min_x - padding).floor() as i64,
        min_y: (min_y - padding).floor() as i64,
        max_x: (max_x + padding).ceil() as i64,
        max_y: (max_y + padding).ceil() as i64,
    }
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

mod dim_policy;
pub(crate) use dim_policy::*;

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
        "via" => {
            let Some(center) = overlay.path.first().copied() else {
                return Vec::new();
            };
            let diameter_nm = overlay
                .diameter_nm
                .or(overlay.width_nm)
                .unwrap_or(600_000)
                .max(1);
            let drill_nm = overlay.drill_nm.unwrap_or(diameter_nm / 2).max(1);
            let radius = (diameter_nm as f32 * 0.5).round() as i64;
            let drill_radius = (drill_nm as f32 * 0.5).round() as i64;
            let outer_rect = datum_gui_protocol::RectNm {
                min_x: center.x - radius,
                min_y: center.y - radius,
                max_x: center.x + radius,
                max_y: center.y + radius,
            };
            push_world_ellipse_nm(out, outer_rect, outer_color, 96);
            let ring_inset = (diameter_nm as f32 * 0.14).round().max(1.0);
            push_world_ellipse_nm(
                out,
                world_inset_rect(outer_rect, ring_inset),
                fill_color,
                96,
            );
            push_world_ellipse_nm(
                out,
                datum_gui_protocol::RectNm {
                    min_x: center.x - drill_radius,
                    min_y: center.y - drill_radius,
                    max_x: center.x + drill_radius,
                    max_y: center.y + drill_radius,
                },
                underlay_color,
                96,
            );
            vec![project_rect(outer_rect, projection)]
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
    out.push(Quad::from_rect(
        header,
        dim_structural_color(COMPONENT_HEADER, dimmed),
    ));
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
        push_world_polygon_fill_contours(out, &graphic.path, &graphic.holes, fill_color);
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
    color: [f32; 3],
    reference_projection: &Projection,
) {
    if graphic.primitive_kind == "polygon" && graphic.path.len() >= 3 {
        push_world_polygon_fill_contours(out, &graphic.path, &graphic.holes, color);
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

fn push_board_text_geometry_world(
    out: &mut Vec<Quad>,
    text_geometry: &BoardTextGeometryPrimitive,
    glyph_mesh_assets: &BTreeMap<GlyphMeshHandlePrimitive, &GlyphMeshAssetPrimitive>,
    color: [f32; 3],
    reference_projection: &Projection,
) {
    if let Some(transform) = text_geometry.world_transform_nm {
        if !text_geometry.glyphs.is_empty() {
            push_board_text_mesh_world(out, text_geometry, glyph_mesh_assets, transform, color);
            return;
        }
    }
    for fill in &text_geometry.fills {
        push_world_polygon_fill_contours(out, &fill.outer, &fill.holes, color);
    }
    for stroke in &text_geometry.strokes {
        push_world_polyline_segments(
            out,
            &[stroke.from, stroke.to],
            (stroke.width_nm as f32).max(world_stroke_nm(1.0, reference_projection)),
            color,
        );
    }
}

fn push_board_text_mesh_world(
    out: &mut Vec<Quad>,
    text_geometry: &BoardTextGeometryPrimitive,
    glyph_mesh_assets: &BTreeMap<GlyphMeshHandlePrimitive, &GlyphMeshAssetPrimitive>,
    transform: Affine2DFixedPrimitive,
    color: [f32; 3],
) {
    for glyph in &text_geometry.glyphs {
        let Some(asset) = glyph_mesh_assets.get(&glyph.glyph_handle) else {
            trace_text_mesh_skip(format!(
                "{} missing glyph mesh asset font={} glyph={} tolerance={} epoch={}",
                text_geometry.object_id,
                glyph.glyph_handle.font_id,
                glyph.glyph_handle.glyph_id,
                glyph.glyph_handle.tolerance_class,
                glyph.glyph_handle.epoch,
            ));
            continue;
        };
        for triangle in asset.indices.chunks_exact(3) {
            let Some(a) = asset.vertices.get(triangle[0] as usize) else {
                trace_text_mesh_skip(format!(
                    "{} glyph={} triangle references missing vertex {}",
                    text_geometry.object_id, glyph.glyph_handle.glyph_id, triangle[0],
                ));
                continue;
            };
            let Some(b) = asset.vertices.get(triangle[1] as usize) else {
                trace_text_mesh_skip(format!(
                    "{} glyph={} triangle references missing vertex {}",
                    text_geometry.object_id, glyph.glyph_handle.glyph_id, triangle[1],
                ));
                continue;
            };
            let Some(c) = asset.vertices.get(triangle[2] as usize) else {
                trace_text_mesh_skip(format!(
                    "{} glyph={} triangle references missing vertex {}",
                    text_geometry.object_id, glyph.glyph_handle.glyph_id, triangle[2],
                ));
                continue;
            };
            let a = transform_text_mesh_point(
                transform,
                glyph.origin_em_nm_x + a.x_em_nm,
                glyph.origin_em_nm_y + a.y_em_nm,
            );
            let b = transform_text_mesh_point(
                transform,
                glyph.origin_em_nm_x + b.x_em_nm,
                glyph.origin_em_nm_y + b.y_em_nm,
            );
            let c = transform_text_mesh_point(
                transform,
                glyph.origin_em_nm_x + c.x_em_nm,
                glyph.origin_em_nm_y + c.y_em_nm,
            );
            push_world_triangle(out, a, b, c, color);
        }
    }
}

fn trace_text_mesh_skip(message: String) {
    if std::env::var_os("DATUM_TRACE_GRAPHICS").is_some() {
        eprintln!("[datum-text-mesh] {message}");
    }
}

fn transform_text_mesh_point(
    transform: Affine2DFixedPrimitive,
    x_em_nm: i64,
    y_em_nm: i64,
) -> (f32, f32) {
    const EM_NM: i128 = 1_000_000;
    let x = (i128::from(transform.m11_ppm) * i128::from(x_em_nm)
        + i128::from(transform.m12_ppm) * i128::from(y_em_nm))
        / EM_NM
        + i128::from(transform.tx_nm);
    let y = (i128::from(transform.m21_ppm) * i128::from(x_em_nm)
        + i128::from(transform.m22_ppm) * i128::from(y_em_nm))
        / EM_NM
        + i128::from(transform.ty_nm);
    (x as f32, y as f32)
}

fn board_graphic_world_color(
    layer_id: &str,
    scene_layers: &[datum_gui_protocol::SceneLayer],
    dimmed: bool,
) -> [f32; 3] {
    let app = resolve_layer_appearance_with_scene(Some(layer_id), scene_layers);
    let layer_name = scene_layers
        .iter()
        .find(|layer| layer.layer_id == layer_id)
        .map(|layer| layer.name.as_str())
        .unwrap_or("");
    let base_color = if layer_name.ends_with(".SilkS") {
        app.silkscreen
    } else {
        app.authored_track
    };
    dim_context_color(base_color, dimmed)
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
            push_projected_ellipse(
                out,
                inner,
                dim_authored_color([0.79, 0.49, 0.26], dimmed),
                24,
            );
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
        push_projected_ellipse(
            out,
            hole,
            dim_structural_color([0.10, 0.11, 0.12], dimmed),
            22,
        );
        let hole_border = inset_rect(hole, 0.8, 0.8, 0.8, 0.8);
        if hole_border.width > 1.0 && hole_border.height > 1.0 {
            push_projected_ellipse(
                out,
                hole_border,
                dim_structural_color([0.62, 0.66, 0.70], dimmed),
                22,
            );
            let hole_inner = inset_rect(hole_border, 1.0, 1.0, 1.0, 1.0);
            if hole_inner.width > 1.0 && hole_inner.height > 1.0 {
                push_projected_ellipse(
                    out,
                    hole_inner,
                    dim_structural_color([0.10, 0.11, 0.12], dimmed),
                    22,
                );
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
        push_world_ellipse_nm(
            out,
            hole,
            dim_structural_color([0.10, 0.11, 0.12], dimmed),
            128,
        );
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
        "roundrect" | "round_rect" => {
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
        (
            half_w - radius,
            -half_h + radius,
            -std::f32::consts::FRAC_PI_2,
        ),
        (half_w - radius, half_h - radius, 0.0),
        (
            -(half_w - radius),
            half_h - radius,
            std::f32::consts::FRAC_PI_2,
        ),
        (-(half_w - radius), -(half_h - radius), std::f32::consts::PI),
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
    let width_px =
        (projection.world_length_to_px(width_nm.round() as i64) - inset_px * 2.0).max(1.0);
    let height_px =
        (projection.world_length_to_px(height_nm.round() as i64) - inset_px * 2.0).max(1.0);
    match pad.shape_kind.as_str() {
        "circle" | "oval" => ellipse_points(center, width_px, height_px, pad.rotation_degrees, 48),
        _ => {
            let min_dim_px = width_px.min(height_px);
            let radius_px = match pad.shape_kind.as_str() {
                "roundrect" | "round_rect" => {
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
    let body_polygon: Vec<PointNm> =
        rounded_rect_points(center, width, height, rotation_degrees, 0.0)
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
        let inner_polygon: Vec<PointNm> =
            rounded_rect_points(center, inner_width, inner_height, rotation_degrees, 0.0)
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
        || scene.board_texts.iter().any(|text| {
            text.style_class.as_deref().is_some_and(|style_class| {
                imported_board_text_belongs_to_component(style_class, component_uuid)
            })
        })
        || scene.component_graphics.iter().any(|graphic| {
            graphic.component_uuid == component_uuid
                && (graphic.graphic_id.contains(":kicad-text-cache:")
                    || graphic.graphic_id.contains(":prop-cache:")
                    || graphic.graphic_id.contains(":kicad-text-stroke:")
                    || graphic.graphic_id.contains(":prop-stroke:"))
        })
}

fn imported_board_text_belongs_to_component(style_class: &str, component_uuid: &str) -> bool {
    ["imported_kicad_property_text:", "imported_kicad_fp_text:"]
        .iter()
        .any(|prefix| {
            style_class
                .strip_prefix(prefix)
                .is_some_and(|rest| rest.starts_with(component_uuid))
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
    let color = component_text_color(text, scene_layers, selected, related, dimmed);
    if !text.cached_polygons.is_empty() {
        for polygon in &text.cached_polygons {
            if polygon.len() >= 3 {
                let projected: Vec<(f32, f32)> = polygon
                    .iter()
                    .map(|point| project_point(*point, projection))
                    .collect();
                push_projected_polygon_fill(out, &projected, color);
            }
        }
        return;
    }

    let rotation = text.rotation_degrees.round() as i32;
    if rotation.rem_euclid(180) == 0 {
        push_component_text_primitive(
            text_runs,
            text,
            scene_layers,
            projection,
            clip_bounds,
            selected,
            related,
            dimmed,
        );
        return;
    }

    let normalized = text.text.to_uppercase();
    let board_text = BoardText {
        uuid: Uuid::nil(),
        text: normalized,
        position: Point {
            x: text.position.x,
            y: text.position.y,
        },
        rotation,
        layer: 0 as LayerId,
        render_intent: eda_engine::text::TextRenderIntent::Manufacturing,
        family: eda_engine::text::TextFamilyId::default(),
        family_source: eda_engine::text::TextFamilySource::ImplicitDefault,
        style: eda_engine::text::TextStyleId::default(),
        height_nm: text.height_nm,
        stroke_width_nm: text
            .stroke_width_nm
            .unwrap_or((text.height_nm / 10).clamp(80_000, 250_000)),
        h_align: eda_engine::text::TextHAlign::Left,
        v_align: eda_engine::text::TextVAlign::Bottom,
        mirrored: false,
        keep_upright: false,
        line_spacing_ratio_ppm: 1_000_000,
        italic: false,
        bold: false,
        style_class: None,
    };
    match render_silkscreen_text_strokes(&board_text) {
        Ok(strokes) if !strokes.is_empty() => {
            for stroke in strokes {
                let path = [
                    stroke_text_point_to_board_space(text.position, stroke.from),
                    stroke_text_point_to_board_space(text.position, stroke.to),
                ];
                let thickness_px = projection
                    .world_length_to_px(stroke.width_nm)
                    .clamp(1.0, 6.0);
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
    push_projected_ellipse(
        out,
        drill,
        dim_structural_color([0.13, 0.14, 0.16], dimmed),
        18,
    );
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
        LayerFamily::TopCopper => LayerAppearance::from_copper_material(
            [0.86, 0.55, 0.24],
            [1.00, 0.84, 0.56],
            [0.98, 0.71, 0.30],
            [0.93, 0.92, 0.82],
        ),
        LayerFamily::InnerCopper => LayerAppearance::from_copper_material(
            [0.67, 0.68, 0.30],
            [0.92, 0.86, 0.54],
            [0.84, 0.80, 0.40],
            [0.86, 0.89, 0.82],
        ),
        LayerFamily::BottomCopper => LayerAppearance::from_copper_material(
            [0.30, 0.76, 0.88],
            [0.71, 0.95, 1.00],
            [0.46, 0.88, 0.96],
            [0.78, 0.92, 0.98],
        ),
        // Bounded exception: geometry whose layer cannot be resolved to a
        // known copper family keeps deliberately divergent fallback colors so
        // unresolved-layer drift stays visible instead of masquerading as a
        // real material lane.
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

fn render_stage_for_layer(
    layer_id: &str,
    scene_layers: &[datum_gui_protocol::SceneLayer],
) -> RenderStage {
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
    // The enum declaration order is the single encoding of the declared
    // render-stack policy; priority is its discriminant.
    stage as u32
}

fn scene_layer_stack_priority(
    layer_id: &str,
    scene_layers: &[datum_gui_protocol::SceneLayer],
) -> u32 {
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
    push_world_polygon_fill_contours(out, polygon, &[], color);
}

fn push_world_polygon_fill_contours(
    out: &mut Vec<Quad>,
    outer: &[PointNm],
    holes: &[Vec<PointNm>],
    color: [f32; 3],
) {
    if holes.is_empty() {
        if outer.len() == 3 && is_convex_polygon_nm(outer) {
            push_world_triangle(
                out,
                (outer[0].x as f32, outer[0].y as f32),
                (outer[1].x as f32, outer[1].y as f32),
                (outer[2].x as f32, outer[2].y as f32),
                color,
            );
            return;
        }
        if outer.len() == 4 && is_convex_polygon_nm(outer) {
            push_world_quad(
                out,
                &[
                    (outer[0].x as f32, outer[0].y as f32),
                    (outer[1].x as f32, outer[1].y as f32),
                    (outer[2].x as f32, outer[2].y as f32),
                    (outer[3].x as f32, outer[3].y as f32),
                ],
                color,
            );
            return;
        }
        match clean_polygon_ring_nm(outer) {
            Some(cleaned) if cleaned.len() == 3 && is_convex_polygon_nm(&cleaned) => {
                push_world_triangle(
                    out,
                    (cleaned[0].x as f32, cleaned[0].y as f32),
                    (cleaned[1].x as f32, cleaned[1].y as f32),
                    (cleaned[2].x as f32, cleaned[2].y as f32),
                    color,
                );
                return;
            }
            Some(cleaned) if cleaned.len() == 4 && is_convex_polygon_nm(&cleaned) => {
                push_world_quad(
                    out,
                    &[
                        (cleaned[0].x as f32, cleaned[0].y as f32),
                        (cleaned[1].x as f32, cleaned[1].y as f32),
                        (cleaned[2].x as f32, cleaned[2].y as f32),
                        (cleaned[3].x as f32, cleaned[3].y as f32),
                    ],
                    color,
                );
                return;
            }
            Some(cleaned) => {
                push_world_polygon_fill_scanline_contours(out, &[cleaned], color);
                return;
            }
            None => return,
        }
    }

    let mut contours = Vec::with_capacity(1 + holes.len());
    if let Some(cleaned_outer) = clean_polygon_ring_nm(outer) {
        contours.push(cleaned_outer);
    }
    for hole in holes {
        if let Some(cleaned_hole) = clean_polygon_ring_nm(hole) {
            contours.push(cleaned_hole);
        }
    }
    if contours.is_empty() {
        return;
    }
    push_world_polygon_fill_scanline_contours(out, &contours, color);
}

fn is_convex_polygon_nm(polygon: &[PointNm]) -> bool {
    if polygon.len() < 3 {
        return false;
    }
    let mut sign = 0_i128;
    for index in 0..polygon.len() {
        let a = polygon[index];
        let b = polygon[(index + 1) % polygon.len()];
        let c = polygon[(index + 2) % polygon.len()];
        let abx = (b.x - a.x) as i128;
        let aby = (b.y - a.y) as i128;
        let bcx = (c.x - b.x) as i128;
        let bcy = (c.y - b.y) as i128;
        let cross = abx * bcy - aby * bcx;
        if cross == 0 {
            continue;
        }
        if sign == 0 {
            sign = cross.signum();
        } else if cross.signum() != sign {
            return false;
        }
    }
    sign != 0
}

fn clean_polygon_ring_nm(polygon: &[PointNm]) -> Option<Vec<PointNm>> {
    if polygon.len() < 3 {
        return None;
    }
    let mut cleaned: Vec<PointNm> = Vec::with_capacity(polygon.len());
    for &point in polygon {
        if cleaned
            .last()
            .is_some_and(|last| last.x == point.x && last.y == point.y)
        {
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
        return None;
    }
    Some(cleaned)
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
    push_projected_polygon_fill_contours(out, polygon, &[], color);
}

fn push_projected_polygon_fill_contours(
    out: &mut Vec<Quad>,
    outer: &[(f32, f32)],
    holes: &[Vec<(f32, f32)>],
    color: [f32; 3],
) {
    let mut contours = Vec::with_capacity(1 + holes.len());
    if let Some(cleaned_outer) = clean_polygon_ring_projected(outer) {
        contours.push(cleaned_outer);
    }
    for hole in holes {
        if let Some(cleaned_hole) = clean_polygon_ring_projected(hole) {
            contours.push(cleaned_hole);
        }
    }
    if contours.is_empty() {
        return;
    }
    push_projected_polygon_fill_scanline_contours(out, &contours, color);
}

fn clean_polygon_ring_projected(polygon: &[(f32, f32)]) -> Option<Vec<(f32, f32)>> {
    if polygon.len() < 3 {
        return None;
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
        return None;
    }
    Some(cleaned)
}

fn push_world_polygon_fill_scanline_contours(
    out: &mut Vec<Quad>,
    contours: &[Vec<PointNm>],
    color: [f32; 3],
) {
    const EPS: f64 = 1e-6;
    #[derive(Clone, Copy)]
    struct ScanlineEdge {
        min_y: f64,
        max_y: f64,
        ax: f64,
        ay: f64,
        bx: f64,
        by: f64,
    }

    impl ScanlineEdge {
        fn x_at(self, y: f64) -> f64 {
            let t = (y - self.ay) / (self.by - self.ay);
            self.ax + (self.bx - self.ax) * t
        }
    }

    let mut ys: Vec<f64> = contours
        .iter()
        .flat_map(|polygon| polygon.iter().map(|p| p.y as f64))
        .collect();
    ys.sort_by(|a, b| a.total_cmp(b));
    ys.dedup_by(|a, b| (*a - *b).abs() <= EPS);
    if ys.len() < 2 {
        return;
    }

    let mut edges: Vec<ScanlineEdge> = Vec::new();
    for polygon in contours {
        for i in 0..polygon.len() {
            let a = polygon[i];
            let b = polygon[(i + 1) % polygon.len()];
            let ay = a.y as f64;
            let by = b.y as f64;
            if (ay - by).abs() <= EPS {
                continue;
            }
            edges.push(ScanlineEdge {
                min_y: ay.min(by),
                max_y: ay.max(by),
                ax: a.x as f64,
                ay,
                bx: b.x as f64,
                by,
            });
        }
    }
    edges.sort_by(|a, b| {
        a.min_y
            .total_cmp(&b.min_y)
            .then_with(|| a.max_y.total_cmp(&b.max_y))
            .then_with(|| a.ax.total_cmp(&b.ax))
            .then_with(|| a.bx.total_cmp(&b.bx))
    });

    let mut next_edge = 0;
    let mut active_edges: Vec<ScanlineEdge> = Vec::new();
    for band in ys.windows(2) {
        let y0 = band[0];
        let y1 = band[1];
        if y1 - y0 <= EPS {
            continue;
        }
        let y_mid = (y0 + y1) * 0.5;

        while next_edge < edges.len() && edges[next_edge].min_y <= y_mid {
            active_edges.push(edges[next_edge]);
            next_edge += 1;
        }
        active_edges.retain(|edge| y_mid < edge.max_y);

        let mut spans: Vec<(f64, f64, f64)> = Vec::with_capacity(active_edges.len());
        for edge in &active_edges {
            if y_mid < edge.min_y || y_mid >= edge.max_y {
                continue;
            }
            spans.push((edge.x_at(y_mid), edge.x_at(y0), edge.x_at(y1)));
        }
        spans.sort_by(|a, b| a.0.total_cmp(&b.0));
        for pair in spans.chunks_exact(2) {
            let left = pair[0];
            let right = pair[1];
            if right.0 - left.0 <= EPS {
                continue;
            }
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

fn push_projected_polygon_fill_scanline_contours(
    out: &mut Vec<Quad>,
    contours: &[Vec<(f32, f32)>],
    color: [f32; 3],
) {
    const EPS: f32 = 1e-4;
    let mut ys: Vec<f32> = contours
        .iter()
        .flat_map(|polygon| polygon.iter().map(|p| p.1))
        .collect();
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
        for polygon in contours {
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
        }
        spans.sort_by(|a, b| a.0.total_cmp(&b.0));
        for pair in spans.chunks_exact(2) {
            let left = pair[0];
            let right = pair[1];
            if right.0 - left.0 <= EPS {
                continue;
            }
            push_projected_quad(
                out,
                &[(left.1, y0), (right.1, y0), (right.2, y1), (left.2, y1)],
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

fn push_board_text_property_row(
    x: f32,
    y: f32,
    key: &str,
    value: &str,
    text_runs: &mut Vec<TextRun>,
) {
    draw_text(
        &format!("{key:<8} {value}"),
        x,
        y,
        12.5,
        TEXT_PANEL_VALUE,
        TextFace::Mono,
        text_runs,
    );
}

fn workspace_tool_label(tool: WorkspaceTool) -> &'static str {
    match tool {
        WorkspaceTool::Select => "SELECT",
        WorkspaceTool::DrawBoardTrack => "DRAW TRACK",
        WorkspaceTool::PlaceBoardVia => "PLACE VIA",
        WorkspaceTool::PlaceBoardText => "PLACE TEXT",
        WorkspaceTool::Move => "MOVE",
        WorkspaceTool::Delete => "DELETE",
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
    let (width_px, height_px) = text_buffer_extent(run, width, height);
    TextBufferKey {
        text: run.text.clone(),
        size_bits: run.size.to_bits(),
        face: run.face,
        width_px,
        height_px,
    }
}

fn text_buffer_extent(run: &TextRun, surface_width: u32, surface_height: u32) -> (u32, u32) {
    let max_width = surface_width.max(1);
    let max_height = surface_height.max(1);
    let width = run.clip_bounds.map_or_else(
        || estimated_text_run_width_px(&run.text, run.size, run.face),
        |bounds| bounds.width.ceil().max(1.0),
    );
    let height = run.clip_bounds.map_or_else(
        || run.size * 1.55 + 6.0,
        |bounds| bounds.height.ceil().max(1.0),
    );
    (
        (width.ceil() as u32).clamp(1, max_width),
        (height.ceil() as u32).clamp(1, max_height),
    )
}

fn estimated_text_run_width_px(text: &str, size: f32, face: TextFace) -> f32 {
    let advance_factor = match face {
        TextFace::Ui => 0.78,
        TextFace::Mono => 0.72,
    };
    let glyphs = text.chars().count().max(1) as f32;
    glyphs * size * advance_factor + 16.0
}

fn scale_text_run_sizes(text_runs: &mut [TextRun], scale: f32) {
    for run in text_runs {
        run.size *= scale;
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

fn build_text_areas<'a>(
    cache: &'a [CachedTextBuffer],
    indices: &[usize],
    runs: &[TextRun],
) -> Vec<TextArea<'a>> {
    indices
        .iter()
        .zip(runs.iter())
        .map(|(index, run)| TextArea {
            buffer: &cache[*index].buffer,
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
        .collect()
}

fn text_prepare_signature(
    indices: &[usize],
    runs: &[TextRun],
    width: u32,
    height: u32,
) -> TextPrepareSignature {
    TextPrepareSignature {
        width,
        height,
        runs: indices
            .iter()
            .zip(runs.iter())
            .map(|(index, run)| TextPrepareRunKey {
                buffer_index: *index,
                x_bits: run.x.to_bits(),
                y_bits: run.y.to_bits(),
                color_bits: run.color.map(f32::to_bits),
                clip_bounds: run.clip_bounds.map(|rect| RectBits {
                    x_bits: rect.x.to_bits(),
                    y_bits: rect.y.to_bits(),
                    width_bits: rect.width.to_bits(),
                    height_bits: rect.height.to_bits(),
                }),
            })
            .collect(),
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub pos: [f32; 2],
    pub color: [f32; 3],
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
    last_text_prepare_signature: Option<TextPrepareSignature>,
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
        let source_ptr = vertices.as_ptr() as usize;
        let source_len = vertices.len();
        if self.world_vertex_buffer.is_some()
            && self.world_vertex_source_ptr == source_ptr
            && self.world_vertex_source_len == source_len
        {
            return;
        }
        Self::upload_vertices(
            device,
            queue,
            &mut self.world_vertex_buffer,
            &mut self.world_vertex_capacity,
            "datum-gui-render-world-vertex-buffer",
            vertices,
        );
        self.world_vertex_source_ptr = source_ptr;
        self.world_vertex_source_len = source_len;
    }

    fn cached_text_buffer_indices(
        &mut self,
        text_runs: &[TextRun],
        width: u32,
        height: u32,
    ) -> (Vec<usize>, TextBufferCacheStats) {
        let mut indices = Vec::with_capacity(text_runs.len());
        let mut stats = TextBufferCacheStats::default();
        for run in text_runs {
            let (index, missed) = self.ensure_text_buffer(run, width, height);
            if missed {
                stats.misses += 1;
            } else {
                stats.hits += 1;
            }
            indices.push(index);
        }
        (indices, stats)
    }

    fn ensure_text_buffer(&mut self, run: &TextRun, width: u32, height: u32) -> (usize, bool) {
        let key = text_buffer_key(run, width, height);
        if let Some(index) = self
            .text_buffer_cache
            .iter()
            .position(|entry| entry.key == key)
        {
            return (index, false);
        }
        let mut buffer = Buffer::new(
            &mut self.font_system,
            Metrics::new(run.size, run.size * 1.22),
        );
        let (buffer_width, buffer_height) = text_buffer_extent(run, width, height);
        buffer.set_size(
            &mut self.font_system,
            Some(buffer_width as f32),
            Some(buffer_height as f32),
        );
        let attrs = text_attrs(run.face);
        buffer.set_text(
            &mut self.font_system,
            &run.text,
            &attrs,
            Shaping::Basic,
            None,
        );
        buffer.shape_until_scroll(&mut self.font_system, false);
        self.text_buffer_cache
            .push(CachedTextBuffer { key, buffer });
        (self.text_buffer_cache.len() - 1, true)
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
        let text_renderer = TextRenderer::new(
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
            last_text_prepare_signature: None,
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

    fn ensure_msaa(
        &mut self,
        device: &wgpu::Device,
        width: u32,
        height: u32,
    ) -> &wgpu::TextureView {
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
        let render_started = std::time::Instant::now();
        let panel_vertices = prepared.panel_vertices();
        let viewport_underlay_vertices = prepared.viewport_underlay_vertices();
        let viewport_overlay_vertices = prepared.viewport_overlay_vertices();
        let world_vertices = retained.world_vertices();
        let visible_world_ranges = prepared.visible_world_ranges();
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
        let upload_started = std::time::Instant::now();
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
        let upload_elapsed = upload_started.elapsed();
        let encode_started = std::time::Instant::now();
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
            if !world_vertices.is_empty() && !visible_world_ranges.is_empty() {
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
                for range in visible_world_ranges {
                    pass.draw(range.clone(), 0..1);
                }
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
        let encode_elapsed = encode_started.elapsed();
        self.viewport.update(queue, Resolution { width, height });
        let text_prepare_started = std::time::Instant::now();
        let (text_buffer_indices, text_cache_stats) =
            self.cached_text_buffer_indices(&prepared.text_runs, width, height);
        let text_signature =
            text_prepare_signature(&text_buffer_indices, &prepared.text_runs, width, height);
        let skipped_text_prepare = self
            .last_text_prepare_signature
            .as_ref()
            .is_some_and(|previous| previous == &text_signature)
            && text_cache_stats.misses == 0;
        if !skipped_text_prepare {
            let prepare_result = self.text_renderer.prepare(
                device,
                queue,
                &mut self.font_system,
                &mut self.atlas,
                &self.viewport,
                build_text_areas(
                    &self.text_buffer_cache,
                    &text_buffer_indices,
                    &prepared.text_runs,
                ),
                &mut self.swash_cache,
            );
            if let Err(initial_error) = prepare_result {
                // Keep the glyph atlas warm during normal interaction. Trim only
                // when prepare reports pressure, then retry with the same semantic
                // text areas. This preserves the DOA2526 atlas-safety behavior
                // without forcing avoidable re-rasterization on every selection.
                self.atlas.trim();
                self.text_renderer
                    .prepare(
                        device,
                        queue,
                        &mut self.font_system,
                        &mut self.atlas,
                        &self.viewport,
                        build_text_areas(
                            &self.text_buffer_cache,
                            &text_buffer_indices,
                            &prepared.text_runs,
                        ),
                        &mut self.swash_cache,
                    )
                    .map_err(|retry_error| {
                        anyhow::anyhow!(
                            "prepare GUI text after atlas trim: {retry_error}; initial: {initial_error}"
                        )
                    })?;
            }
            self.last_text_prepare_signature = Some(text_signature);
        }
        let text_prepare_elapsed = text_prepare_started.elapsed();
        let text_encode_started = std::time::Instant::now();
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
        let text_encode_elapsed = text_encode_started.elapsed();
        let submit_started = std::time::Instant::now();
        queue.submit([encoder.finish()]);
        let submit_elapsed = submit_started.elapsed();
        trace_render_timing(format!(
            "renderer total={}ms upload={}ms encode={}ms text_prepare={}ms text_encode={}ms submit={}ms vertices panel={} underlay={} world={} overlay={} text_runs={} text_cache={}/{} text_prepare_skipped={}",
            render_started.elapsed().as_millis(),
            upload_elapsed.as_millis(),
            encode_elapsed.as_millis(),
            text_prepare_elapsed.as_millis(),
            text_encode_elapsed.as_millis(),
            submit_elapsed.as_millis(),
            panel_vertices.len(),
            viewport_underlay_vertices.len(),
            world_vertices.len(),
            viewport_overlay_vertices.len(),
            prepared.text_runs.len(),
            text_cache_stats.hits,
            text_cache_stats.misses,
            skipped_text_prepare,
        ));
        Ok(())
    }
}

fn trace_render_timing(message: String) {
    if std::env::var_os("DATUM_TRACE_TIMING").is_some() {
        eprintln!("[datum-render] {message}");
    }
}

fn trace_graphic_timing(
    graphic: &BoardGraphicPrimitive,
    started: std::time::Instant,
    quad_count: usize,
) {
    let elapsed_ms = started.elapsed().as_millis();
    if std::env::var_os("DATUM_TRACE_GRAPHICS").is_some() && (elapsed_ms >= 5 || quad_count >= 1024)
    {
        eprintln!(
            "[datum-graphic] {} kind={} layer={} points={} holes={} quads={} {}ms",
            graphic.object_id,
            graphic.primitive_kind,
            graphic.layer_id,
            graphic.path.len(),
            graphic.holes.len(),
            quad_count,
            elapsed_ms
        );
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

fn text_row_height_for_size(size: f32) -> f32 {
    (size * 1.6).ceil().max(size + 4.0)
}

fn key_value_row_height() -> f32 {
    text_row_height_for_size(12.5)
}

#[allow(dead_code)]
fn sample_artifact_preview_primitives()
-> Vec<datum_gui_protocol::ProductionArtifactPreviewPrimitive> {
    use datum_gui_protocol::{
        ProductionArtifactPreviewPoint as P, ProductionArtifactPreviewPrimitive as Prim,
    };
    vec![
        Prim {
            kind: "stroke".to_string(),
            aperture_diameter_nm: Some(250_000),
            aperture_width_nm: None,
            aperture_height_nm: None,
            tool: None,
            diameter_mm: None,
            points: vec![
                P { x_nm: 0, y_nm: 0 },
                P {
                    x_nm: 1_000_000,
                    y_nm: 1_000_000,
                },
            ],
        },
        Prim {
            kind: "flash".to_string(),
            aperture_diameter_nm: Some(400_000),
            aperture_width_nm: None,
            aperture_height_nm: None,
            tool: None,
            diameter_mm: None,
            points: vec![P {
                x_nm: 500_000,
                y_nm: 250_000,
            }],
        },
    ]
}

#[allow(dead_code)]
fn panel_vertices_without_artifact_preview(mut state: ReviewWorkspaceState) -> usize {
    if let Some(artifact) = state.production.focused_artifact.as_mut()
        && let Some(preview) = artifact.focused_preview.as_mut()
    {
        preview.primitives.clear();
    }
    PreparedScene::from_workspace(
        &state,
        1280,
        800,
        CameraState::fit_to_bounds(&state.scene.bounds),
        &RetainedScene::from_workspace(&state, 1280, 800),
    )
    .panel_vertices()
    .len()
}

#[allow(dead_code)]
fn artifact_preview_adds_panel_vertices(
    prepared: &PreparedScene,
    state: ReviewWorkspaceState,
) -> bool {
    prepared.panel_vertices().len() > panel_vertices_without_artifact_preview(state)
}

#[allow(dead_code)]
fn prepared_has_artifact_preview_controls(prepared: &PreparedScene) -> bool {
    let has_zoom = prepared
        .hit_regions
        .iter()
        .any(|region| matches!(region.target, HitTarget::ArtifactPreviewZoomIn));
    let has_geometry = prepared
        .hit_regions
        .iter()
        .any(|region| matches!(region.target, HitTarget::ToggleArtifactPreviewGeometry));
    let has_viewport = prepared
        .hit_regions
        .iter()
        .any(|region| matches!(region.target, HitTarget::ArtifactPreviewViewport));
    has_zoom && has_geometry && has_viewport
}

#[allow(dead_code)]
fn outputs_dock_renders_csv_preview_table(mut state: ReviewWorkspaceState) -> bool {
    if let Some(artifact) = state.production.focused_artifact.as_mut()
        && let Some(preview) = artifact.focused_preview.as_mut()
    {
        preview.preview_kind = "bom_csv".to_string();
        preview.primitive_count = 0;
        preview.primitives.clear();
        preview.row_count = Some(2);
        preview.csv_columns = vec!["ref".to_string(), "value".to_string()];
        preview.csv_rows = vec![
            vec!["R1".to_string(), "10k".to_string()],
            vec!["C1".to_string(), "100n".to_string()],
        ];
    }
    let prepared = PreparedScene::from_workspace(
        &state,
        1280,
        800,
        CameraState::fit_to_bounds(&state.scene.bounds),
        &RetainedScene::from_workspace(&state, 1280, 800),
    );
    let text = prepared
        .text_runs
        .iter()
        .map(|run| run.text.as_str())
        .collect::<Vec<_>>();
    text.iter().any(|value| value.contains("TABLE 2 ROWS"))
        && text.iter().any(|value| value.contains("R1 | 10k"))
}

#[cfg(test)]
mod layout_invariant_tests;
#[cfg(test)]
mod lib_extra_tests;
#[cfg(test)]
mod render_contract_tests;
#[cfg(test)]
mod terminal_dock_contract_tests;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn shell_layout_reserves_bottom_dock_and_viewport() {
        let layout = ShellLayout::for_window(1280, 800, None);
        assert!(layout.viewport.width > 0.0);
        assert_eq!(layout.bottom_strip.height, 44.0);
        assert!(layout.left_sidebar.width > 0.0);
        assert!(layout.right_sidebar.width > 0.0);
    }

    #[test]
    fn text_buffer_key_ignores_position_and_color_but_tracks_content() {
        let base = TextRun {
            text: "PROJECT".to_string(),
            x: 12.0,
            y: 24.0,
            size: 12.0,
            color: TEXT_PRIMARY,
            face: TextFace::Ui,
            clip_bounds: None,
        };
        let mut moved = base.clone();
        moved.x += 100.0;
        moved.y += 50.0;
        moved.color = TEXT_SECONDARY;

        assert_eq!(
            text_buffer_key(&base, 1280, 768),
            text_buffer_key(&moved, 1280, 768)
        );

        let mut changed_text = base.clone();
        changed_text.text.push('!');
        assert_ne!(
            text_buffer_key(&base, 1280, 768),
            text_buffer_key(&changed_text, 1280, 768)
        );

        let mut changed_size = base.clone();
        changed_size.size = 13.0;
        assert_ne!(
            text_buffer_key(&base, 1280, 768),
            text_buffer_key(&changed_size, 1280, 768)
        );

        let mut clipped = base.clone();
        clipped.clip_bounds = Some(RectPx {
            x: 0.0,
            y: 0.0,
            width: 44.0,
            height: 18.0,
        });
        assert_ne!(
            text_buffer_key(&base, 1280, 768),
            text_buffer_key(&clipped, 1280, 768)
        );
    }

    #[test]
    fn text_prepare_signature_tracks_render_relevant_inputs() {
        let run = TextRun {
            text: "TERMINAL".to_string(),
            x: 12.0,
            y: 24.0,
            size: 12.0,
            color: TEXT_PRIMARY,
            face: TextFace::Ui,
            clip_bounds: None,
        };
        let base = text_prepare_signature(&[4], std::slice::from_ref(&run), 1280, 768);
        let mut moved = run.clone();
        moved.x += 1.0;
        assert_ne!(
            base,
            text_prepare_signature(&[4], std::slice::from_ref(&moved), 1280, 768)
        );
        let mut recolored = run.clone();
        recolored.color = TEXT_SECONDARY;
        assert_ne!(
            base,
            text_prepare_signature(&[4], std::slice::from_ref(&recolored), 1280, 768)
        );
        assert_ne!(
            base,
            text_prepare_signature(&[5], std::slice::from_ref(&run), 1280, 768)
        );
        assert_ne!(
            base,
            text_prepare_signature(&[4], std::slice::from_ref(&run), 1281, 768)
        );
    }

    #[test]
    fn shell_layout_is_solved_by_taffy_grid_contract() {
        let layout = ShellLayout::for_window(1280, 800, Some(260));

        assert_eq!(
            layout.left_sidebar,
            RectPx {
                x: 0.0,
                y: 0.0,
                width: 280.0,
                height: 540.0,
            }
        );
        assert_eq!(
            layout.viewport,
            RectPx {
                x: 280.0,
                y: 0.0,
                width: 660.0,
                height: 540.0,
            }
        );
        assert_eq!(
            layout.right_sidebar,
            RectPx {
                x: 940.0,
                y: 0.0,
                width: 340.0,
                height: 540.0,
            }
        );
        assert_eq!(
            layout.bottom_strip,
            RectPx {
                x: 0.0,
                y: 540.0,
                width: 1280.0,
                height: 260.0,
            }
        );
    }

    #[test]
    fn shell_layout_solves_logical_pixels_then_scales_to_surface_pixels() {
        let layout = ShellLayout::for_surface(1600, 1000, 1.25, Some(260));

        assert_eq!(layout.top_menu_bar.height, 42.5);
        assert_eq!(layout.tool_rail.width, 60.0);
        assert_eq!(layout.left_sidebar.width, 285.0);
        assert_eq!(layout.right_sidebar.width, 375.0);
        assert_eq!(layout.bottom_strip.height, 325.0);
        assert_eq!(layout.status_bar.height, 31.25);
        assert_eq!(layout.viewport.x, 345.0);
        assert_eq!(layout.viewport.width, 880.0);
        assert_eq!(layout.bottom_strip.y, 643.75);
    }

    #[test]
    fn prepared_scene_uses_surface_scale_for_layout_and_text() {
        let state = datum_gui_protocol::load_fixture_workspace_state();
        let retained = RetainedScene::from_workspace_for_surface(&state, 1600, 1000, 1.25);
        let prepared = PreparedScene::from_workspace_for_surface(
            &state,
            1600,
            1000,
            1.25,
            CameraState::fit_to_bounds(&state.scene.bounds),
            &retained,
        );

        assert_eq!(
            prepared.layout,
            ShellLayout::for_surface(1600, 1000, 1.25, dock_height_for_state(&state))
        );
        let project_title = prepared
            .text_runs
            .iter()
            .find(|run| run.text == "PROJECT")
            .expect("project title should render");
        assert_eq!(project_title.size, 15.0);
        assert!(
            prepared
                .hit_test(
                    prepared.layout.left_sidebar.x + 20.0,
                    prepared.layout.left_sidebar.y + 20.0,
                )
                .is_none()
        );
    }

    #[test]
    fn proposal_preview_affected_ids_match_scene_source_ids() {
        let mut state = datum_gui_protocol::load_fixture_workspace_state();
        let component = state
            .scene
            .components
            .first()
            .expect("fixture component should exist")
            .clone();
        state.production.proposals = vec![datum_gui_protocol::ProductionProposalSummary {
            proposal_id: "proposal-a".to_string(),
            status: "draft".to_string(),
            source: "check".to_string(),
            rationale: "highlight modified component".to_string(),
            operation_count: 1,
            can_apply: Some(false),
            blocker_codes: Vec::new(),
            preview: Some(datum_gui_protocol::ProductionProposalPreviewSummary {
                prepared_against: "rev-before".to_string(),
                preview_after_model_revision: "rev-after".to_string(),
                created_count: 0,
                modified_count: 1,
                deleted_count: 0,
                affected_object_count: 1,
                affected_objects: vec![component.source_object_uuid.clone()],
                render_deltas: Vec::new(),
            }),
        }];

        let affected = proposal_preview_affected_ids(&state);
        assert!(source_object_matches_preview(
            &affected,
            &component.object_id,
            &component.source_object_uuid
        ));
        assert!(component_matches_preview(
            &component.component_uuid,
            &state.scene,
            &affected
        ));
    }
    #[test]
    fn proposal_preview_render_deltas_become_overlay_primitives() {
        let mut state = datum_gui_protocol::load_fixture_workspace_state();
        state.production.proposals = vec![datum_gui_protocol::ProductionProposalSummary {
            proposal_id: "proposal-a".to_string(),
            status: "draft".to_string(),
            source: "check".to_string(),
            rationale: "ghost new track".to_string(),
            operation_count: 1,
            can_apply: Some(false),
            blocker_codes: Vec::new(),
            preview: Some(datum_gui_protocol::ProductionProposalPreviewSummary {
                prepared_against: "rev-before".to_string(),
                preview_after_model_revision: "rev-after".to_string(),
                created_count: 1,
                modified_count: 0,
                deleted_count: 0,
                affected_object_count: 1,
                affected_objects: vec!["track-a".to_string()],
                render_deltas: vec![
                    datum_gui_protocol::ProductionProposalRenderDeltaSummary {
                        delta_kind: "create".to_string(),
                        object_id: "track-a".to_string(),
                        primitive_kind: "track_path".to_string(),
                        layer_id: "L1".to_string(),
                        end_layer_id: None,
                        width_nm: 250_000,
                        drill_nm: None,
                        diameter_nm: None,
                        path: vec![
                            datum_gui_protocol::PointNm { x: 1000, y: 2000 },
                            datum_gui_protocol::PointNm { x: 3000, y: 4000 },
                        ],
                    },
                    datum_gui_protocol::ProductionProposalRenderDeltaSummary {
                        delta_kind: "create".to_string(),
                        object_id: "via-a".to_string(),
                        primitive_kind: "via".to_string(),
                        layer_id: "L1".to_string(),
                        end_layer_id: Some("L2".to_string()),
                        width_nm: 650_000,
                        drill_nm: Some(300_000),
                        diameter_nm: Some(650_000),
                        path: vec![datum_gui_protocol::PointNm { x: 5000, y: 6000 }],
                    },
                ],
            }),
        }];

        let overlays = production_proposal_overlay_primitives(&state);
        assert_eq!(overlays.len(), 2);
        assert_eq!(overlays[0].overlay_id, "proposal:proposal-a:preview:0");
        assert_eq!(overlays[0].primitive_kind, "track_path");
        assert_eq!(overlays[0].proposal_action_id, "proposal-a");
        assert_eq!(overlays[0].layer_id.as_deref(), Some("L1"));
        assert_eq!(overlays[0].width_nm, Some(250_000));
        assert_eq!(overlays[0].path.len(), 2);
        assert_eq!(overlays[1].overlay_id, "proposal:proposal-a:preview:1");
        assert_eq!(overlays[1].primitive_kind, "via");
        assert_eq!(overlays[1].proposal_action_id, "proposal-a");
        assert_eq!(overlays[1].layer_id.as_deref(), Some("L1"));
        assert_eq!(overlays[1].width_nm, Some(650_000));
        assert_eq!(overlays[1].drill_nm, Some(300_000));
        assert_eq!(overlays[1].diameter_nm, Some(650_000));
        assert_eq!(overlays[1].path.len(), 1);
    }

    #[test]
    fn prepared_scene_preserves_viewport_dominance() {
        let state = datum_gui_protocol::load_fixture_workspace_state();
        let retained = RetainedScene::from_workspace(&state, 1280, 800);
        let prepared = PreparedScene::from_workspace(
            &state,
            1280,
            960,
            CameraState::fit_to_bounds(&state.scene.bounds),
            &retained,
        );
        assert!(prepared.layout.viewport.width > prepared.layout.left_sidebar.width);
        assert!(prepared.layout.viewport.width > prepared.layout.right_sidebar.width / 2.0);
    }

    #[test]
    fn authoring_tool_buttons_are_not_rendered_in_read_only_phase_one() {
        let state = datum_gui_protocol::load_fixture_workspace_state();
        let retained = RetainedScene::from_workspace(&state, 1280, 800);
        let prepared = PreparedScene::from_workspace(
            &state,
            1280,
            800,
            CameraState::fit_to_bounds(&state.scene.bounds),
            &retained,
        );
        let tool_regions = prepared
            .hit_regions
            .iter()
            .filter(|region| matches!(region.target, HitTarget::SetWorkspaceTool(_)))
            .collect::<Vec<_>>();

        assert!(tool_regions.is_empty());
    }

    #[test]
    fn project_card_controls_flow_above_filter_controls() {
        let state = datum_gui_protocol::load_fixture_workspace_state();
        let retained = RetainedScene::from_workspace(&state, 1280, 800);
        let prepared = PreparedScene::from_workspace(
            &state,
            1280,
            800,
            CameraState::fit_to_bounds(&state.scene.bounds),
            &retained,
        );
        let fit_bottom = prepared
            .hit_regions
            .iter()
            .filter_map(|region| {
                if matches!(
                    region.target,
                    HitTarget::FitBoard | HitTarget::FitReviewTarget
                ) {
                    Some(region.rect.y + region.rect.height)
                } else {
                    None
                }
            })
            .fold(0.0_f32, f32::max);
        let filter_top = prepared
            .hit_regions
            .iter()
            .find(|region| matches!(region.target, HitTarget::ToggleShowAuthored))
            .expect("authored filter hit region should render")
            .rect
            .y;

        assert!(fit_bottom < filter_top);
    }

    #[test]
    fn populated_inspector_status_stays_inside_right_column() {
        let mut state = datum_gui_protocol::load_fixture_workspace_state();
        let action_id = "action-populated".to_string();
        state.active_review_target_id = action_id.clone();
        state.selection = SelectionTarget::ReviewAction(action_id.clone());
        state.review.proposal_actions = vec![datum_gui_protocol::RouteProposalActionPayload {
            action_id,
            proposal_action: "draw_track".to_string(),
            reason: "qa populated review state".to_string(),
            contract: "m7_populated_layout_contract".to_string(),
            net_uuid: "net-populated".to_string(),
            net_name: "BOARD_STATUS_NET".to_string(),
            from_anchor_pad_uuid: "pad-a".to_string(),
            to_anchor_pad_uuid: "pad-b".to_string(),
            layer: 1,
            width_nm: 200_000,
            from: datum_gui_protocol::PointNm { x: 0, y: 0 },
            to: datum_gui_protocol::PointNm { x: 1_000_000, y: 0 },
            reused_via_uuid: None,
            reused_via_uuids: Vec::new(),
            reused_object_kind: None,
            reused_object_uuid: None,
            reused_object_from_layer: None,
            reused_object_to_layer: None,
            selected_path_bend_count: 1,
            selected_path_point_count: 2,
            selected_path_segment_index: 0,
            selected_path_segment_count: 1,
            selected_path_layer_segment_index: Some(0),
            selected_path_layer_segment_count: Some(1),
            selected_path_layer_segment_bend_count: Some(1),
            selected_path_layer_segment_point_count: Some(2),
        }];
        state.review.segment_evidence = vec![datum_gui_protocol::RouteProposalSegmentEvidence {
            layer_segment_index: 0,
            layer_segment_count: 1,
            layer: 1,
            bend_count: 1,
            point_count: 2,
            track_action_count: 1,
        }];
        state.last_command_status = Some(datum_gui_protocol::EditorCommandStatus {
            action: "place_board_text".to_string(),
            detail: "queued board text @ 140700000,90100000".to_string(),
        });

        let layout = ShellLayout::for_window(1280, 800, dock_height_for_state(&state));
        let right_layout =
            side_panels::solve_right_panel_layout_with_taffy(&state, layout.right_sidebar)
                .expect("right panel layout should solve");
        let inspector_bottom = right_layout.inspector_rect.y + right_layout.inspector_rect.height;
        let retained = RetainedScene::from_workspace(&state, 1280, 800);
        let prepared = PreparedScene::from_workspace(
            &state,
            1280,
            800,
            CameraState::fit_to_bounds(&state.scene.bounds),
            &retained,
        );
        let last_row_y = prepared
            .text_runs
            .iter()
            .find(|run| run.text == "LAST")
            .expect("populated inspector should render LAST key")
            .y;

        assert!(last_row_y + 12.0 <= inspector_bottom);
    }

    #[test]
    fn filter_summary_renders_below_layer_rows() {
        let state = datum_gui_protocol::load_fixture_workspace_state();
        let retained = RetainedScene::from_workspace(&state, 1280, 800);
        let prepared = PreparedScene::from_workspace(
            &state,
            1280,
            800,
            CameraState::fit_to_bounds(&state.scene.bounds),
            &retained,
        );
        let layer_bottom = prepared
            .hit_regions
            .iter()
            .filter_map(|region| {
                if matches!(region.target, HitTarget::ToggleLayer(_)) {
                    Some(region.rect.y + region.rect.height)
                } else {
                    None
                }
            })
            .fold(0.0_f32, f32::max);
        let outputs_y = prepared
            .text_runs
            .iter()
            .find(|run| run.text.starts_with("OUTPUTS "))
            .expect("production output summary should render")
            .y;

        assert!(outputs_y > layer_bottom);
    }

    #[test]
    fn terminal_dock_does_not_surface_artifact_file_summaries() {
        let mut state = datum_gui_protocol::load_fixture_workspace_state();
        state.ui.active_dock_tab = Some(datum_gui_protocol::DockTab::Terminal);
        state.ui.dock_height_px = 560;
        state.production = datum_gui_protocol::ProductionStatus {
            output_job_count: 1,
            artifact_count: 1,
            latest_status: Some("succeeded".to_string()),
            latest_run_id: Some("00000000-0000-0000-0000-00000000run1".to_string()),
            manufacturing_plan_count: 1,
            panel_projection_count: 1,
            output_jobs: vec![datum_gui_protocol::ProductionOutputJobSummary {
                id: "00000000-0000-0000-0000-00000000job1".to_string(),
                name: "Release fabrication".to_string(),
                include: vec!["drill".to_string()],
                prefix: "release-a".to_string(),
                output_dir: None,
                family: "DRILL".to_string(),
                status: "succeeded".to_string(),
                execution_count: 1,
                artifact_count: 1,
                latest_run_id: Some("00000000-0000-0000-0000-00000000run1".to_string()),
                latest_run_artifact_id: Some("00000000-0000-0000-0000-00000000art1".to_string()),
                artifacts: vec![datum_gui_protocol::ProductionArtifactSummary {
                    artifact_id: "00000000-0000-0000-0000-00000000art1".to_string(),
                    kind: "drill".to_string(),
                    project_id: None,
                    model_revision: None,
                    output_job: None,
                    variant: None,
                    generator_version: None,
                    output_dir: Some("/tmp/fab".to_string()),
                    validation_state: None,
                    file_count: 1,
                    files: vec![datum_gui_protocol::ProductionArtifactFileSummary {
                        path: "fabrication/release-a-drill.drl".to_string(),
                        sha256: "sha256:abc123".to_string(),
                    }],
                    production_projection_count: 1,
                    production_projections: vec![
                        datum_gui_protocol::ProductionArtifactProjectionSummary {
                            projection_kind: "excellon_drill".to_string(),
                            projection_contract: "datum.production_projection.excellon_drill.v1"
                                .to_string(),
                            model_revision: "revision-a".to_string(),
                            byte_count: 128,
                            sha256: "sha256:def456".to_string(),
                        },
                    ],
                }],
            }],
            manufacturing_plans: vec![datum_gui_protocol::ProductionManufacturingPlanSummary {
                id: "00000000-0000-0000-0000-00000000fab1".to_string(),
                name: "Release fabrication".to_string(),
                prefix: "release-a".to_string(),
                board_or_panel: "00000000-0000-0000-0000-00000000pan1".to_string(),
                variant: None,
                object_revision: 2,
            }],
            panel_projections: vec![datum_gui_protocol::ProductionPanelProjectionSummary {
                id: "00000000-0000-0000-0000-00000000pan1".to_string(),
                name: "Release panel".to_string(),
                board_instance_count: 1,
                first_board: Some("00000000-0000-0000-0000-00000000brd1".to_string()),
                first_x_nm: Some(1000),
                first_y_nm: Some(2000),
                first_rotation_deg: Some(90),
                object_revision: 3,
            }],
            focused_artifact: Some(datum_gui_protocol::ProductionArtifactDetail {
                artifact_id: "00000000-0000-0000-0000-00000000art1".to_string(),
                kind: "gerber_set".to_string(),
                output_dir: Some("/tmp/fab".to_string()),
                validation_state: "valid".to_string(),
                file_count: 1,
                files: vec![datum_gui_protocol::ProductionArtifactFileSummary {
                    path: "fabrication/board-F_Cu.gbr".to_string(),
                    sha256: "sha256:abc123".to_string(),
                }],
                focused_file: Some(datum_gui_protocol::ProductionArtifactFileSummary {
                    path: "fabrication/board-F_Cu.gbr".to_string(),
                    sha256: "sha256:abc123".to_string(),
                }),
                focused_preview: Some(datum_gui_protocol::ProductionArtifactFilePreviewSummary {
                    file: "fabrication/board-F_Cu.gbr".to_string(),
                    preview_kind: "gerber_rs274x".to_string(),
                    hash_matches_metadata: true,
                    primitive_count: 4,
                    primitives: sample_artifact_preview_primitives(),
                    geometry_count: Some(4),
                    hit_count: None,
                    row_count: None,
                    csv_columns: Vec::new(),
                    csv_rows: Vec::new(),
                }),
                production_projection_count: 1,
                production_projections: vec![
                    datum_gui_protocol::ProductionArtifactProjectionSummary {
                        projection_kind: "gerber_copper_layer".to_string(),
                        projection_contract: "datum.production_projection.gerber_copper_layer.v1"
                            .to_string(),
                        model_revision: "revision-a".to_string(),
                        byte_count: 128,
                        sha256: "sha256:def456".to_string(),
                    },
                ],
            }),
            ..datum_gui_protocol::ProductionStatus::default()
        };
        let retained = RetainedScene::from_workspace(&state, 1280, 960);
        let prepared = PreparedScene::from_workspace(
            &state,
            1280,
            960,
            CameraState::fit_to_bounds(&state.scene.bounds),
            &retained,
        );
        let rendered_text = prepared
            .text_runs
            .iter()
            .map(|run| run.text.as_str())
            .collect::<Vec<_>>();
        assert!(
            !rendered_text.contains(&"OUTPUT JOBS"),
            "Phase 1 terminal dock must not render Output-lane summaries"
        );
        assert!(
            !prepared
                .hit_regions
                .iter()
                .any(|region| matches!(region.target, HitTarget::ProductionArtifact(_))),
            "Phase 1 terminal dock must not expose Output-lane artifact hit regions"
        );
    }

    #[test]
    fn marking_menu_shell_renders_manifest_items_without_command_targets() {
        let mut state = datum_gui_protocol::load_fixture_workspace_state();
        state.ui.marking_menu = Some(datum_gui_protocol::MarkingMenuState {
            menu_key: "pcb.component".to_string(),
            target_object_id: Some("component:demo".to_string()),
            anchor_x_px: 640,
            anchor_y_px: 360,
            preview_slot: Some("N".to_string()),
            gesture_dx_px: 0,
            gesture_dy_px: -72,
        });
        let retained = RetainedScene::from_workspace(&state, 1280, 768);
        let prepared = PreparedScene::from_workspace(
            &state,
            1280,
            768,
            CameraState::fit_to_bounds(&state.scene.bounds),
            &retained,
        );
        let rendered_text = prepared
            .text_runs
            .iter()
            .map(|run| run.text.as_str())
            .collect::<Vec<_>>();

        assert!(rendered_text.contains(&"Rotate"));
        assert!(rendered_text.contains(&"Delete"));
        assert!(rendered_text.contains(&"pcb.component"));
        assert!(prepared.hit_regions.iter().any(|region| matches!(
            &region.target,
            HitTarget::MarkingMenuItem {
                menu_key,
                slot,
                label
            } if menu_key == "pcb.component" && slot == "N" && label == "Rotate"
        )));
        assert!(
            prepared.hit_regions.iter().all(|region| {
                !matches!(
                    region.target,
                    HitTarget::ProductionTerminalCommand(_) | HitTarget::ProductionOutputJobRun(_)
                )
            }),
            "inert marking-menu shell must not expose terminal command hit targets"
        );
    }

    #[test]
    fn imported_board_text_counts_as_component_detail_text() {
        let component_uuid = "f7794004-b142-4fe8-aea4-5f3796f333a5";

        assert!(imported_board_text_belongs_to_component(
            &format!(
                "imported_kicad_property_text:{component_uuid}:reference:component_silkscreen"
            ),
            component_uuid,
        ));
        assert!(imported_board_text_belongs_to_component(
            &format!("imported_kicad_fp_text:{component_uuid}:component_silkscreen"),
            component_uuid,
        ));
        assert!(!imported_board_text_belongs_to_component(
            "imported_kicad_property_text:other-component:reference:component_silkscreen",
            component_uuid,
        ));
        assert!(!imported_board_text_belongs_to_component(
            "manual_board_text",
            component_uuid,
        ));
    }

    #[test]
    fn board_text_mesh_path_bypasses_legacy_fill_fragments() {
        let handle = GlyphMeshHandlePrimitive {
            font_id: 1,
            glyph_id: 42,
            tolerance_class: 1,
            epoch: 0,
        };
        let asset = GlyphMeshAssetPrimitive {
            handle,
            vertices: vec![
                datum_gui_protocol::MeshVertexEmPrimitive {
                    x_em_nm: 0,
                    y_em_nm: 0,
                },
                datum_gui_protocol::MeshVertexEmPrimitive {
                    x_em_nm: 1_000_000,
                    y_em_nm: 0,
                },
                datum_gui_protocol::MeshVertexEmPrimitive {
                    x_em_nm: 0,
                    y_em_nm: 1_000_000,
                },
            ],
            indices: vec![0, 1, 2],
            bbox_em_nm: datum_gui_protocol::MeshRectEmPrimitive {
                min_x_em_nm: 0,
                min_y_em_nm: 0,
                max_x_em_nm: 1_000_000,
                max_y_em_nm: 1_000_000,
            },
        };
        let text_geometry = BoardTextGeometryPrimitive {
            object_id: "board-text:test".to_string(),
            object_kind: "board_text".to_string(),
            text_uuid: "test".to_string(),
            layer_id: "L37".to_string(),
            world_transform_nm: Some(Affine2DFixedPrimitive {
                m11_ppm: 1_000_000,
                m12_ppm: 0,
                m21_ppm: 0,
                m22_ppm: 1_000_000,
                tx_nm: 10,
                ty_nm: 20,
            }),
            block_bbox_em_nm: None,
            glyphs: vec![datum_gui_protocol::TextGlyphInstancePrimitive {
                glyph_handle: handle,
                origin_em_nm_x: 0,
                origin_em_nm_y: 0,
            }],
            fills: vec![datum_gui_protocol::BoardTextFillPrimitive {
                outer: vec![
                    PointNm { x: 0, y: 0 },
                    PointNm { x: 10, y: 0 },
                    PointNm { x: 10, y: 10 },
                    PointNm { x: 0, y: 10 },
                ],
                holes: Vec::new(),
            }],
            strokes: Vec::new(),
        };
        let assets = BTreeMap::from([(handle, &asset)]);
        let projection = Projection::new(
            RectPx {
                x: 0.0,
                y: 0.0,
                width: 100.0,
                height: 100.0,
            },
            &datum_gui_protocol::SceneBounds {
                min_x: 0,
                min_y: 0,
                max_x: 100,
                max_y: 100,
            },
            CameraState {
                center_x_nm: 50.0,
                center_y_nm: 50.0,
                zoom: 1.0,
            },
        );
        let mut out = Vec::new();

        push_board_text_geometry_world(
            &mut out,
            &text_geometry,
            &assets,
            [1.0, 1.0, 1.0],
            &projection,
        );

        assert_eq!(
            out.len(),
            1,
            "mesh-backed text must render from glyph mesh triangles, not legacy fill fragments"
        );
        assert_eq!(
            out[0].points,
            [
                (10.0, 20.0),
                (1_000_010.0, 20.0),
                (10.0, 1_000_020.0),
                (10.0, 1_000_020.0),
            ]
        );
    }

    #[test]
    fn board_text_mesh_missing_asset_does_not_fall_back_to_legacy_fragments() {
        let handle = GlyphMeshHandlePrimitive {
            font_id: 1,
            glyph_id: 42,
            tolerance_class: 1,
            epoch: 0,
        };
        let text_geometry = BoardTextGeometryPrimitive {
            object_id: "board-text:test".to_string(),
            object_kind: "board_text".to_string(),
            text_uuid: "test".to_string(),
            layer_id: "L37".to_string(),
            world_transform_nm: Some(Affine2DFixedPrimitive {
                m11_ppm: 1_000_000,
                m12_ppm: 0,
                m21_ppm: 0,
                m22_ppm: 1_000_000,
                tx_nm: 10,
                ty_nm: 20,
            }),
            block_bbox_em_nm: None,
            glyphs: vec![datum_gui_protocol::TextGlyphInstancePrimitive {
                glyph_handle: handle,
                origin_em_nm_x: 0,
                origin_em_nm_y: 0,
            }],
            fills: vec![datum_gui_protocol::BoardTextFillPrimitive {
                outer: vec![
                    PointNm { x: 0, y: 0 },
                    PointNm { x: 10, y: 0 },
                    PointNm { x: 10, y: 10 },
                    PointNm { x: 0, y: 10 },
                ],
                holes: Vec::new(),
            }],
            strokes: Vec::new(),
        };
        let projection = Projection::new(
            RectPx {
                x: 0.0,
                y: 0.0,
                width: 100.0,
                height: 100.0,
            },
            &datum_gui_protocol::SceneBounds {
                min_x: 0,
                min_y: 0,
                max_x: 100,
                max_y: 100,
            },
            CameraState {
                center_x_nm: 50.0,
                center_y_nm: 50.0,
                zoom: 1.0,
            },
        );
        let mut out = Vec::new();
        let assets = BTreeMap::new();

        push_board_text_geometry_world(
            &mut out,
            &text_geometry,
            &assets,
            [1.0, 1.0, 1.0],
            &projection,
        );

        assert!(
            out.is_empty(),
            "malformed mesh-backed text should skip the bad glyph, not render stale legacy fragments"
        );
    }

    #[test]
    fn board_text_mesh_bad_indices_skip_bad_triangles_without_panic() {
        let handle = GlyphMeshHandlePrimitive {
            font_id: 1,
            glyph_id: 42,
            tolerance_class: 1,
            epoch: 0,
        };
        let asset = GlyphMeshAssetPrimitive {
            handle,
            vertices: vec![
                datum_gui_protocol::MeshVertexEmPrimitive {
                    x_em_nm: 0,
                    y_em_nm: 0,
                },
                datum_gui_protocol::MeshVertexEmPrimitive {
                    x_em_nm: 1_000_000,
                    y_em_nm: 0,
                },
            ],
            indices: vec![0, 1, 2],
            bbox_em_nm: datum_gui_protocol::MeshRectEmPrimitive {
                min_x_em_nm: 0,
                min_y_em_nm: 0,
                max_x_em_nm: 1_000_000,
                max_y_em_nm: 0,
            },
        };
        let text_geometry = BoardTextGeometryPrimitive {
            object_id: "board-text:test".to_string(),
            object_kind: "board_text".to_string(),
            text_uuid: "test".to_string(),
            layer_id: "L37".to_string(),
            world_transform_nm: Some(Affine2DFixedPrimitive {
                m11_ppm: 1_000_000,
                m12_ppm: 0,
                m21_ppm: 0,
                m22_ppm: 1_000_000,
                tx_nm: 10,
                ty_nm: 20,
            }),
            block_bbox_em_nm: None,
            glyphs: vec![datum_gui_protocol::TextGlyphInstancePrimitive {
                glyph_handle: handle,
                origin_em_nm_x: 0,
                origin_em_nm_y: 0,
            }],
            fills: Vec::new(),
            strokes: Vec::new(),
        };
        let assets = BTreeMap::from([(handle, &asset)]);
        let projection = Projection::new(
            RectPx {
                x: 0.0,
                y: 0.0,
                width: 100.0,
                height: 100.0,
            },
            &datum_gui_protocol::SceneBounds {
                min_x: 0,
                min_y: 0,
                max_x: 100,
                max_y: 100,
            },
            CameraState {
                center_x_nm: 50.0,
                center_y_nm: 50.0,
                zoom: 1.0,
            },
        );
        let mut out = Vec::new();

        push_board_text_geometry_world(
            &mut out,
            &text_geometry,
            &assets,
            [1.0, 1.0, 1.0],
            &projection,
        );

        assert!(
            out.is_empty(),
            "bad mesh indices should skip only the invalid triangle"
        );
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
    fn board_outline_hit_region_selects_assembled_outline() {
        let state = datum_gui_protocol::load_fixture_workspace_state();
        let retained = RetainedScene::from_workspace(&state, 1280, 800);
        let outline = state
            .scene
            .outline
            .first()
            .expect("fixture should include a board outline");
        assert!(
            outline.path.len() >= 2,
            "fixture outline should include at least one segment"
        );
        let a = outline.path[0];
        let b = outline.path[1];
        let hit_point = PointNm {
            x: (a.x + b.x) / 2,
            y: (a.y + b.y) / 2,
        };

        let hit = retained
            .hit_test_authored_world(hit_point, &state)
            .expect("board outline segment should be selectable");
        assert_eq!(hit, &HitTarget::AuthoredObject(outline.object_id.clone()));
    }

    #[test]
    fn selected_board_text_numeric_rows_have_step_and_center_edit_zones() {
        let mut state = datum_gui_protocol::load_fixture_workspace_state();
        let object_id = "board-text:test-hit-zones".to_string();
        state
            .scene
            .board_texts
            .push(datum_gui_protocol::BoardTextPrimitive {
                object_id: object_id.clone(),
                object_kind: "board_text".to_string(),
                text_uuid: "test-hit-zones".to_string(),
                text: "TEST".to_string(),
                layer_id: "F.Silks".to_string(),
                position: PointNm { x: 0, y: 0 },
                rotation_degrees: 0,
                height_nm: 1_000_000,
                stroke_width_nm: 100_000,
                render_intent: "annotation".to_string(),
                family: "inter".to_string(),
                style: "regular".to_string(),
                style_class: None,
                h_align: "center".to_string(),
                v_align: "center".to_string(),
                mirrored: false,
                keep_upright: true,
                line_spacing_ratio_ppm: 1_000_000,
                bold: false,
                italic: false,
            });
        state.selection = SelectionTarget::AuthoredObject(object_id);

        let retained = RetainedScene::from_workspace(&state, 1280, 800);
        let prepared = PreparedScene::from_workspace(
            &state,
            1280,
            800,
            CameraState::fit_to_bounds(&state.scene.bounds),
            &retained,
        );

        assert_three_zone_row(
            &prepared,
            HitTarget::DecreaseSelectedBoardTextHeight,
            HitTarget::EditSelectedBoardTextHeight,
            HitTarget::IncreaseSelectedBoardTextHeight,
        );
        assert_three_zone_row(
            &prepared,
            HitTarget::RotateSelectedBoardTextCounterClockwise90,
            HitTarget::EditSelectedBoardTextRotation,
            HitTarget::RotateSelectedBoardTextClockwise90,
        );
        assert_three_zone_row(
            &prepared,
            HitTarget::DecreaseSelectedBoardTextLineSpacing,
            HitTarget::EditSelectedBoardTextLineSpacing,
            HitTarget::IncreaseSelectedBoardTextLineSpacing,
        );
        assert_three_zone_row(
            &prepared,
            HitTarget::CycleSelectedBoardTextRenderIntent,
            HitTarget::EditSelectedBoardTextRenderIntent,
            HitTarget::CycleSelectedBoardTextRenderIntent,
        );
        assert_three_zone_row(
            &prepared,
            HitTarget::CycleSelectedBoardTextFamily,
            HitTarget::EditSelectedBoardTextFamily,
            HitTarget::CycleSelectedBoardTextFamily,
        );
        assert_three_zone_row(
            &prepared,
            HitTarget::CycleSelectedBoardTextHAlign,
            HitTarget::EditSelectedBoardTextAlignment,
            HitTarget::CycleSelectedBoardTextVAlign,
        );
    }

    fn assert_three_zone_row(
        prepared: &PreparedScene,
        left: HitTarget,
        center: HitTarget,
        right: HitTarget,
    ) {
        let left_rect = hit_rect(prepared, &left);
        let center_rect = hit_rect(prepared, &center);
        let right_rect = hit_rect_from_end(prepared, &right);
        assert!(
            (left_rect.y - center_rect.y).abs() < f32::EPSILON
                && (center_rect.y - right_rect.y).abs() < f32::EPSILON,
            "three-zone hit regions must share one row"
        );
        assert!(left_rect.x < center_rect.x);
        assert!(center_rect.x < right_rect.x);
        assert!(center_rect.width > left_rect.width);
        assert!(center_rect.width > right_rect.width);

        assert_eq!(hit_center(prepared, left_rect), left);
        assert_eq!(hit_center(prepared, center_rect), center);
        assert_eq!(hit_center(prepared, right_rect), right);
    }

    fn hit_rect(prepared: &PreparedScene, target: &HitTarget) -> RectPx {
        prepared
            .hit_regions
            .iter()
            .find(|region| &region.target == target)
            .map(|region| region.rect)
            .unwrap_or_else(|| panic!("expected hit region for {target:?}"))
    }

    fn hit_rect_from_end(prepared: &PreparedScene, target: &HitTarget) -> RectPx {
        prepared
            .hit_regions
            .iter()
            .rev()
            .find(|region| &region.target == target)
            .map(|region| region.rect)
            .unwrap_or_else(|| panic!("expected hit region for {target:?}"))
    }

    fn hit_center(prepared: &PreparedScene, rect: RectPx) -> HitTarget {
        prepared
            .hit_test(rect.x + rect.width * 0.5, rect.y + rect.height * 0.5)
            .cloned()
            .expect("hit target should exist at rect center")
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
    fn retained_pad_hit_regions_target_pads_for_inspection() {
        let state = datum_gui_protocol::load_fixture_workspace_state();
        let pad = state
            .scene
            .pads
            .first()
            .expect("fixture should include pads");
        let retained = RetainedScene::from_workspace(&state, 1280, 800);

        assert!(
            retained.world_hit_regions.iter().any(|region| {
                matches!(&region.target, HitTarget::AuthoredObject(id) if id == &pad.object_id)
            }),
            "pad hit region should select the pad object for read-only inspection"
        );
    }

    #[test]
    fn polygon_hit_shape_contains_zone_interior_only() {
        let polygon = vec![
            PointNm { x: 0, y: 0 },
            PointNm { x: 100, y: 0 },
            PointNm { x: 100, y: 100 },
            PointNm { x: 0, y: 100 },
        ];

        assert!(point_in_polygon_world(&polygon, PointNm { x: 50, y: 50 }));
        assert!(!point_in_polygon_world(&polygon, PointNm { x: 150, y: 50 }));
    }

    #[test]
    fn board_text_inspector_does_not_emit_edit_hit_regions_in_phase_one() {
        let mut state = datum_gui_protocol::load_fixture_workspace_state();
        let Some(text) = state.scene.board_texts.first() else {
            return;
        };
        state.selection = SelectionTarget::AuthoredObject(text.object_id.clone());
        let retained = RetainedScene::from_workspace(&state, 1280, 800);
        let prepared = PreparedScene::from_workspace(
            &state,
            1280,
            800,
            CameraState::fit_to_bounds(&state.scene.bounds),
            &retained,
        );

        assert!(
            prepared.hit_regions.iter().all(|region| {
                !matches!(
                    region.target,
                    HitTarget::ToggleSelectedBoardTextMirrored
                        | HitTarget::ToggleSelectedBoardTextKeepUpright
                        | HitTarget::ToggleSelectedBoardTextBold
                        | HitTarget::CycleSelectedBoardTextRenderIntent
                        | HitTarget::CycleSelectedBoardTextFamily
                        | HitTarget::CycleSelectedBoardTextHAlign
                        | HitTarget::CycleSelectedBoardTextVAlign
                        | HitTarget::EditSelectedBoardTextRenderIntent
                        | HitTarget::EditSelectedBoardTextFamily
                        | HitTarget::EditSelectedBoardTextAlignment
                        | HitTarget::DecreaseSelectedBoardTextHeight
                        | HitTarget::IncreaseSelectedBoardTextHeight
                        | HitTarget::RotateSelectedBoardTextCounterClockwise90
                        | HitTarget::RotateSelectedBoardTextClockwise90
                        | HitTarget::DecreaseSelectedBoardTextLineSpacing
                        | HitTarget::IncreaseSelectedBoardTextLineSpacing
                        | HitTarget::EditSelectedBoardTextContent
                        | HitTarget::EditSelectedBoardTextHeight
                        | HitTarget::EditSelectedBoardTextRotation
                        | HitTarget::EditSelectedBoardTextLineSpacing
                )
            }),
            "Phase 1 board-text inspector must be read-only"
        );
    }

    #[test]
    fn layers_panel_renders_active_layer_and_toggle_regions() {
        let state = datum_gui_protocol::load_fixture_workspace_state();
        let retained = RetainedScene::from_workspace(&state, 1280, 800);
        let prepared = PreparedScene::from_workspace(
            &state,
            1280,
            800,
            CameraState::fit_to_bounds(&state.scene.bounds),
            &retained,
        );
        let active_layer = state
            .ui
            .filters
            .active_layer_id
            .as_deref()
            .and_then(|active_id| {
                state
                    .scene
                    .layers
                    .iter()
                    .find(|layer| layer.layer_id == active_id)
            })
            .expect("fixture should set an active layer");
        let active_label = format!(
            "ACTIVE {}",
            truncate_text(&active_layer.name.to_uppercase(), 14)
        );

        assert!(
            prepared
                .text_runs
                .iter()
                .any(|run| run.text == active_label),
            "Layers panel should render the active layer summary"
        );
        assert!(
            prepared
                .hit_regions
                .iter()
                .any(|region| matches!(region.target, HitTarget::ToggleLayer(_))),
            "Layers panel should expose consumer-side layer toggle hit regions"
        );
    }
}
