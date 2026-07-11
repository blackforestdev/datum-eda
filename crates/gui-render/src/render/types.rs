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

/// Which pane's scene a resolved screen point / world hit belongs to (UVT-004,
/// the CoordinateHit keystone). `world_point_at_screen` resolves the FOCUSED-vs-
/// containing pane and reports the surface so callers route the follow-up world
/// hit-test to the matching retained scene (the board `RetainedScene` vs the
/// companion schematic one) in that surface's own camera/space. Board is the
/// only interactive surface pre-S3; Schematic is the new one this slice unblocks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SceneSurface {
    Board,
    Schematic,
}
#[derive(Debug, Clone, PartialEq)]
pub struct PreparedScene {
    pub layout: ShellLayout,
    pub hit_regions: Vec<HitRegion>,
    pub scene_viewport: RectPx,
    scene_bounds: datum_gui_protocol::SceneBounds,
    camera: CameraState,
    panel_vertices: Vec<Vertex>,
    menu_overlay_vertices: Vec<Vertex>,
    menu_overlay_text_runs: Vec<TextRun>,
    viewport_underlay_vertices: Vec<Vertex>,
    viewport_overlay_vertices: Vec<Vertex>,
    visible_world_ranges: Vec<Range<u32>>,
    text_runs: Vec<TextRun>,
    // P2.2a bounded second-scene descriptor: the STATIC companion schematic pass.
    // `Some` only when the layout has a Schematic pane AND the workspace carries a
    // projected `schematic_scene`; gates the additive second world GPU pass. The
    // camera is a fixed fit-to-schematic-bounds (no interactive pan/zoom on pane B
    // this slice). Ranges for that pass are derived in gpu.rs from the threaded
    // schematic RetainedScene (render() has no `state`), so they are not stored
    // here — the schematic renders all of its batches (its layers are always
    // visible, not board-layer-toggle governed).
    schematic_scene_viewport: Option<RectPx>,
    schematic_bounds: datum_gui_protocol::SceneBounds,
    schematic_camera: CameraState,
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
    UiMedium,
    UiStrong,
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
// Passive panel bodies read as the flat SURFACE_01 material (flush stacked
// panels). SURFACE_02 is reserved for interactive fields/hover/tool-buttons and
// SURFACE_03 for active chips — see the surface-ladder note in VISUAL_LANGUAGE.
const PANEL_CARD_BG: [f32; 3] = design_tokens::chrome::SURFACE_01;
const PANEL_CARD_BORDER: [f32; 3] = design_tokens::chrome::BORDER_SUBTLE;
const VIEWPORT_BG: [f32; 3] = design_tokens::chrome::CANVAS;
// Retained token: the board pane no longer draws an outer viewport frame (the
// only outline is the inner board edge), but keep the binding documented.
#[allow(dead_code)]
const VIEWPORT_FRAME: [f32; 3] = design_tokens::chrome::BORDER_STRONG;
const BOARD_OUTER_FIELD: [f32; 3] = design_tokens::chrome::CANVAS;
const BOARD_INNER_FIELD: [f32; 3] = design_tokens::content::BOARD_SUBSTRATE;
const BOARD_GRID_MAJOR: [f32; 3] = design_tokens::content::BOARD_GRID_MAJOR;
const BOARD_GRID_MINOR: [f32; 3] = design_tokens::content::BOARD_GRID_MINOR;
// Schematic-pane grid (P2.2f). The companion schematic pass draws its own square
// underlay; these mirror the schematic prototype's `#sgrid` whisper and never
// touch the board grid path.
const SCHEMATIC_GRID_MAJOR: [f32; 3] = design_tokens::schematic::GRID_MAJOR;
const SCHEMATIC_GRID_MINOR: [f32; 3] = design_tokens::schematic::GRID_MINOR;
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

