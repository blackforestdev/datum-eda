use anyhow::{Context, Result, bail};
use eda_engine::board::BoardText;
use eda_engine::substrate::{ProjectResolver, SourceShardKind};
use eda_engine::text::{
    TextAttributes, TextFamilyId, TextGeometryPrimitive, TextHAlign, TextRenderIntent, TextStyleId,
    TextVAlign, default_stroke_width_nm, layout_text_geometry, layout_text_mesh_from_board_text,
};
use serde::de::{DeserializeOwned, Deserializer};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::hash_map::DefaultHasher;
use std::collections::{BTreeMap, BTreeSet};
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Mutex, OnceLock};
pub mod gui_menu_model;
pub use gui_menu_model::{
    GuiIconDef, GuiIconSet, GuiMarkingMenu, GuiMenu, GuiMenuBinding, GuiMenuItem, GuiMenuModel,
    load_default_gui_icon_set, load_default_gui_menu_model,
};
mod kicad_scene_import;
use kicad_scene_import::{
    load_scene_from_kicad_import, outline_board_graphics_from_outline,
    push_board_text_scene_primitives, trace_protocol_timing,
};
mod schematic_scene_import;
pub use schematic_scene_import::load_kicad_schematic_workspace_state;
mod artifact_preview_viewport;
pub use artifact_preview_viewport::ArtifactPreviewViewportState;
mod context_envelope;
pub use context_envelope::*;
mod terminal_command_catalog;
pub use terminal_command_catalog::*;
mod production_artifacts;
pub use production_artifacts::*;
mod production_artifact_runs;
pub use production_artifact_runs::ProductionArtifactRunSummary;
use production_artifact_runs::{ArtifactListPayload, artifact_run_summaries};
mod production_focus;
use production_focus::focused_artifact_id;
mod source_shard_status;
pub use source_shard_status::{
    SourceShardAttentionItem, SourceShardStatusSummary,
    load_accepted_transaction_tip as refresh_accepted_transaction_tip,
    load_source_shard_status as refresh_source_shard_status,
};
mod terminal_lane;
pub use terminal_lane::{
    TerminalLaneState, TerminalStyleSpan, TerminalStyledLine, TerminalTabState, TerminalTextStyle,
};
mod workspace_layout;
pub use workspace_layout::{
    ConsoleLaneState, CrosshairStyle, DockTab, MarkingMenuState, PaneContent, PaneId, PaneNode,
    SplitChild, SplitOrientation, WorkspaceFilterState, WorkspaceLayout, WorkspacePreset,
    WorkspaceUiState, PANE_RATIO_MAX, PANE_RATIO_MIN,
};
mod production_proposals;
pub use production_proposals::{
    ProductionProposalPreviewSummary, ProductionProposalRenderDeltaSummary,
    ProductionProposalSummary, production_status_from_proposals_json,
};
use production_proposals::{ProposalsPayload, attach_proposal_validation, proposal_summaries};
mod board_text_primitives;
pub use board_text_primitives::{
    Affine2DFixedPrimitive, BoardTextFillPrimitive, BoardTextGeometryPrimitive, BoardTextPrimitive,
    BoardTextStrokePrimitive, GlyphMeshAssetPrimitive, GlyphMeshHandlePrimitive,
    MeshRectEmPrimitive, MeshVertexEmPrimitive, TextGlyphInstancePrimitive,
};
mod check_runs;
pub use check_runs::{
    CheckFindingSummary, CheckRunCoverageSummary, CheckRunProfileBasisSummary, CheckRunReviewState,
    check_finding_scene_target_object_id, check_run_review_state_from_json,
};
mod known_good_demo;
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BoardReviewSceneV1 {
    pub kind: String,
    pub version: u32,
    pub scene_id: String,
    pub project_uuid: String,
    pub project_name: String,
    pub board_uuid: String,
    pub board_name: String,
    pub units: String,
    pub source_revision: String,
    #[serde(default)]
    pub pad_expansion_setup: ScenePadExpansionSetup,
    pub bounds: SceneBounds,
    pub layers: Vec<SceneLayer>,
    pub outline: Vec<OutlinePolyline>,
    pub components: Vec<ComponentBounds>,
    #[serde(default)]
    pub component_graphics: Vec<ComponentGraphicPrimitive>,
    #[serde(default)]
    pub component_texts: Vec<ComponentTextPrimitive>,
    pub pads: Vec<PadPrimitive>,
    pub tracks: Vec<TrackPrimitive>,
    pub vias: Vec<ViaPrimitive>,
    pub zones: Vec<ZonePrimitive>,
    /// Board-level authored graphics tied to a named layer (e.g. imported
    /// Edge.Cuts contributors under M7-SCN-007 Option B). Distinct from
    /// `component_graphics` which are footprint-scoped; these are board-scoped
    /// and participate in the normal authored-layer visibility/appearance
    /// model.
    #[serde(default)]
    pub board_graphics: Vec<BoardGraphicPrimitive>,
    #[serde(default)]
    pub board_texts: Vec<BoardTextPrimitive>,
    #[serde(default)]
    pub board_text_geometries: Vec<BoardTextGeometryPrimitive>,
    #[serde(default)]
    pub glyph_mesh_assets: Vec<GlyphMeshAssetPrimitive>,
    #[serde(default)]
    pub unrouted_primitives: Vec<UnroutedPrimitive>,
    #[serde(default)]
    pub net_display: Vec<NetDisplayEntry>,
    pub proposal_overlay_primitives: Vec<ProposalOverlayPrimitive>,
    pub review_primitives: Vec<ReviewPrimitive>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ScenePadExpansionSetup {
    #[serde(default)]
    pub pad_to_mask_clearance_nm: i64,
    #[serde(default)]
    pub pad_to_paste_clearance_nm: i64,
    #[serde(default)]
    pub pad_to_paste_ratio_ppm: i32,
    #[serde(default)]
    pub solder_mask_min_width_nm: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BoardGraphicPrimitive {
    pub object_id: String,
    /// Always `"board_graphic"` — coarse selection/filtering vocabulary.
    pub object_kind: String,
    /// Fine-grained shape class: `"line"`, `"arc"`, `"polyline"`, or `"polygon"`.
    pub primitive_kind: String,
    pub source_object_uuid: String,
    pub layer_id: String,
    pub path: Vec<PointNm>,
    #[serde(default)]
    pub holes: Vec<Vec<PointNm>>,
    #[serde(default)]
    pub width_nm: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UnroutedPrimitive {
    pub object_id: String,
    pub object_kind: String,
    pub source_object_uuid: String,
    pub net_uuid: String,
    pub from_component: String,
    pub from_pin: String,
    pub to_component: String,
    pub to_pin: String,
    pub path: Vec<PointNm>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NetDisplayEntry {
    pub net_uuid: String,
    pub net_name: String,
    pub airwire_color_rgb: [f32; 3],
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ComponentGraphicPrimitive {
    pub graphic_id: String,
    pub component_uuid: String,
    pub layer_id: Option<String>,
    pub primitive_kind: String,
    pub render_role: String,
    pub width_nm: Option<i64>,
    pub closed: bool,
    pub path: Vec<PointNm>,
    #[serde(default)]
    pub holes: Vec<Vec<PointNm>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ComponentTextPrimitive {
    pub text_id: String,
    pub component_uuid: String,
    pub layer_id: Option<String>,
    pub render_role: String,
    pub text: String,
    pub position: PointNm,
    pub rotation_degrees: f32,
    pub height_nm: i64,
    #[serde(default)]
    pub face_name: Option<String>,
    #[serde(default)]
    pub stroke_width_nm: Option<i64>,
    #[serde(default)]
    pub cached_polygons: Vec<Vec<PointNm>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SceneBounds {
    pub min_x: i64,
    pub min_y: i64,
    pub max_x: i64,
    pub max_y: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SceneLayer {
    pub layer_id: String,
    pub name: String,
    pub kind: String,
    pub render_order: u32,
    pub visible_by_default: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OutlinePolyline {
    pub object_id: String,
    pub object_kind: String,
    pub source_object_uuid: String,
    #[serde(default = "default_outline_layer_id")]
    pub layer_id: String,
    pub path: Vec<PointNm>,
}

fn default_outline_layer_id() -> String {
    // Scene-level layer key for Edge.Cuts under the `L{n}` convention used by
    // `scene.layers` and the layer-visibility map. KiCad's Edge.Cuts is
    // canonically layer id 44; we use that as the default so existing fixture
    // JSON round-trips to a key that matches the visibility map.
    "L44".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ComponentBounds {
    pub object_id: String,
    pub object_kind: String,
    pub source_object_uuid: String,
    pub component_uuid: String,
    pub reference: String,
    pub value: Option<String>,
    pub placement_layer: String,
    pub position: PointNm,
    pub rotation_degrees: f32,
    pub bounds: RectNm,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PadPrimitive {
    pub object_id: String,
    pub object_kind: String,
    pub source_object_uuid: String,
    pub pad_uuid: String,
    pub component_uuid: String,
    pub net_uuid: Option<String>,
    pub layer_id: String,
    #[serde(default)]
    pub copper_layer_ids: Vec<String>,
    pub center: PointNm,
    pub bounds: RectNm,
    pub shape_kind: String,
    #[serde(default = "default_roundrect_rratio_ppm")]
    pub roundrect_rratio_ppm: u32,
    #[serde(default)]
    pub mask_layer_ids: Vec<String>,
    #[serde(default)]
    pub paste_layer_ids: Vec<String>,
    #[serde(default)]
    pub solder_mask_margin_nm: i64,
    #[serde(default)]
    pub solder_paste_margin_nm: i64,
    #[serde(default)]
    pub solder_paste_margin_ratio_ppm: i32,
    #[serde(default)]
    pub drill_nm: Option<i64>,
    #[serde(default)]
    pub rotation_degrees: f32,
}

fn default_roundrect_rratio_ppm() -> u32 {
    250_000
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TrackPrimitive {
    pub object_id: String,
    pub object_kind: String,
    pub source_object_uuid: String,
    pub track_uuid: String,
    pub net_uuid: Option<String>,
    pub layer_id: String,
    pub width_nm: i64,
    pub path: Vec<PointNm>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ViaPrimitive {
    pub object_id: String,
    pub object_kind: String,
    pub source_object_uuid: String,
    pub via_uuid: String,
    pub net_uuid: Option<String>,
    pub position: PointNm,
    pub drill_nm: i64,
    pub diameter_nm: i64,
    pub start_layer_id: String,
    pub end_layer_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ZonePrimitive {
    pub object_id: String,
    pub object_kind: String,
    pub source_object_uuid: String,
    pub zone_uuid: String,
    pub net_uuid: Option<String>,
    pub layer_id: String,
    pub polygon: Vec<PointNm>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProposalOverlayPrimitive {
    pub overlay_id: String,
    pub primitive_kind: String,
    pub proposal_action_id: String,
    pub layer_id: Option<String>,
    pub render_role: String,
    pub width_nm: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub drill_nm: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub diameter_nm: Option<i64>,
    pub path: Vec<PointNm>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ReviewPrimitive {
    pub review_primitive_id: String,
    pub primitive_kind: String,
    pub evidence_key: Option<String>,
    pub path: Vec<PointNm>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct PointNm {
    pub x: i64,
    pub y: i64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct RectNm {
    pub min_x: i64,
    pub min_y: i64,
    pub max_x: i64,
    pub max_y: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RouteProposalReviewPayload {
    pub action: String,
    pub review_source: String,
    pub status: String,
    pub explanation: String,
    pub project_root: Option<String>,
    pub artifact_path: Option<String>,
    pub kind: Option<String>,
    pub source_version: Option<u32>,
    pub version: Option<u32>,
    pub project_uuid: Option<String>,
    pub project_name: Option<String>,
    pub net_uuid: Option<String>,
    pub from_anchor_pad_uuid: Option<String>,
    pub to_anchor_pad_uuid: Option<String>,
    pub selection_profile: Option<String>,
    pub selection_rule: Option<String>,
    pub selected_candidate: Option<String>,
    pub selected_policy: Option<String>,
    pub contract: String,
    pub actions: usize,
    pub draw_track_actions: usize,
    pub selected_path_bend_count: usize,
    pub selected_path_point_count: usize,
    pub selected_path_segment_count: usize,
    #[serde(default, deserialize_with = "deserialize_null_default")]
    pub segment_evidence: Vec<RouteProposalSegmentEvidence>,
    pub proposal_actions: Vec<RouteProposalActionPayload>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RouteProposalSegmentEvidence {
    pub layer_segment_index: usize,
    pub layer_segment_count: usize,
    pub layer: i32,
    pub bend_count: usize,
    pub point_count: usize,
    pub track_action_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RouteProposalActionPayload {
    pub action_id: String,
    pub proposal_action: String,
    pub reason: String,
    pub contract: String,
    pub net_uuid: String,
    pub net_name: String,
    pub from_anchor_pad_uuid: String,
    pub to_anchor_pad_uuid: String,
    pub layer: i32,
    pub width_nm: i64,
    pub from: PointNm,
    pub to: PointNm,
    pub reused_via_uuid: Option<String>,
    #[serde(default)]
    pub reused_via_uuids: Vec<String>,
    #[serde(default)]
    pub reused_object_kind: Option<String>,
    #[serde(default)]
    pub reused_object_uuid: Option<String>,
    #[serde(default)]
    pub reused_object_from_layer: Option<i32>,
    #[serde(default)]
    pub reused_object_to_layer: Option<i32>,
    #[serde(default)]
    pub selected_path_bend_count: usize,
    pub selected_path_point_count: usize,
    pub selected_path_segment_index: usize,
    pub selected_path_segment_count: usize,
    #[serde(default)]
    pub selected_path_layer_segment_index: Option<usize>,
    #[serde(default)]
    pub selected_path_layer_segment_count: Option<usize>,
    #[serde(default)]
    pub selected_path_layer_segment_bend_count: Option<usize>,
    #[serde(default)]
    pub selected_path_layer_segment_point_count: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SelectionTarget {
    None,
    ReviewAction(String),
    AuthoredObject(String),
    CheckFinding(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceBacking {
    pub request: LiveReviewRequest,
    pub board_path: PathBuf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoardTextBooleanField {
    Mirrored,
    KeepUpright,
    Bold,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoardTextAlignmentField {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoardTextLineSpacingStep {
    Decrease,
    Increase,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoardTextHeightStep {
    Decrease,
    Increase,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoardTextRotationStep {
    CounterClockwise90,
    Clockwise90,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoardTextCycleField {
    RenderIntent,
    Family,
}

pub const BOARD_TEXT_LINE_SPACING_MIN_PPM: i32 = 500_000;
pub const BOARD_TEXT_LINE_SPACING_MAX_PPM: i32 = 2_000_000;
pub const BOARD_TEXT_LINE_SPACING_STEP_PPM: i32 = 100_000;
pub const BOARD_TEXT_HEIGHT_MIN_NM: i64 = 50_000;
pub const BOARD_TEXT_HEIGHT_MAX_NM: i64 = 100_000_000;
pub const BOARD_TEXT_HEIGHT_STEP_PPM: i64 = 100_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkspaceTool {
    Select,
    DrawBoardTrack,
    PlaceBoardVia,
    PlaceBoardText,
    Move,
    Delete,
}

impl WorkspaceTool {
    pub fn label(self) -> &'static str {
        match self {
            WorkspaceTool::Select => "select",
            WorkspaceTool::DrawBoardTrack => "draw-board-track",
            WorkspaceTool::PlaceBoardVia => "place-board-via",
            WorkspaceTool::PlaceBoardText => "place-board-text",
            WorkspaceTool::Move => "move",
            WorkspaceTool::Delete => "delete",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AuthoringSnapState {
    pub enabled: bool,
    pub grid_nm: i64,
}

impl Default for AuthoringSnapState {
    fn default() -> Self {
        Self {
            enabled: true,
            grid_nm: 100_000,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthoringGestureState {
    pub tool: WorkspaceTool,
    pub anchor: Option<PointNm>,
    pub preview: Option<PointNm>,
    pub target_object_id: Option<String>,
}

impl AuthoringGestureState {
    pub fn idle(tool: WorkspaceTool) -> Self {
        Self {
            tool,
            anchor: None,
            preview: None,
            target_object_id: None,
        }
    }

    pub fn is_active(&self) -> bool {
        self.anchor.is_some() || self.preview.is_some() || self.target_object_id.is_some()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthoringToolState {
    pub snap: AuthoringSnapState,
    pub gesture: AuthoringGestureState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditorCommandStatus {
    pub action: String,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq, Default)]
pub struct ProductionStatus {
    pub output_job_count: usize,
    pub artifact_count: usize,
    pub artifact_run_count: usize,
    pub proposal_count: usize,
    pub manufacturing_plan_count: usize,
    pub panel_projection_count: usize,
    pub latest_status: Option<String>,
    pub latest_run_id: Option<String>,
    pub latest_artifact_id: Option<String>,
    pub latest_artifact_run_id: Option<String>,
    pub latest_output_job_run_id: Option<String>,
    pub output_jobs: Vec<ProductionOutputJobSummary>,
    pub artifact_runs: Vec<ProductionArtifactRunSummary>,
    pub proposals: Vec<ProductionProposalSummary>,
    pub manufacturing_plans: Vec<ProductionManufacturingPlanSummary>,
    pub panel_projections: Vec<ProductionPanelProjectionSummary>,
    pub focused_artifact: Option<ProductionArtifactDetail>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ProductionOutputJobSummary {
    pub id: String,
    pub name: String,
    pub include: Vec<String>,
    pub prefix: String,
    pub output_dir: Option<String>,
    pub family: String,
    pub status: String,
    pub execution_count: usize,
    pub artifact_count: usize,
    pub latest_run_id: Option<String>,
    pub latest_run_artifact_id: Option<String>,
    pub artifacts: Vec<ProductionArtifactSummary>,
}
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ProductionManufacturingPlanSummary {
    pub id: String,
    pub name: String,
    pub prefix: String,
    pub board_or_panel: String,
    pub variant: Option<String>,
    pub object_revision: u64,
}
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ProductionPanelProjectionSummary {
    pub id: String,
    pub name: String,
    pub board_instance_count: usize,
    pub first_board: Option<String>,
    pub first_x_nm: Option<i64>,
    pub first_y_nm: Option<i64>,
    pub first_rotation_deg: Option<i32>,
    pub object_revision: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionCommand {
    SetTool(WorkspaceTool),
    BeginAuthoringGesture {
        world: PointNm,
        target_object_id: Option<String>,
    },
    PreviewAuthoringGesture {
        world: PointNm,
        target_object_id: Option<String>,
    },
    CancelAuthoringGesture,
    SelectReviewAction(String),
    SelectAuthoredObject(String),
    SelectCheckFinding(String),
    ClearSelection,
    SelectPreviousReviewAction,
    SelectNextReviewAction,
    ToggleShowAuthored,
    ToggleShowProposed,
    ToggleShowUnrouted,
    ToggleDimUnrelated,
    ToggleLayerVisibility(String),
    FocusProductionArtifact(String),
    FocusProductionArtifactFile(String),
    ZoomArtifactPreviewIn,
    ZoomArtifactPreviewOut,
    PanArtifactPreview {
        delta_x_ppm: i32,
        delta_y_ppm: i32,
    },
    ResetArtifactPreviewViewport,
    ToggleArtifactPreviewGeometry,
    ToggleArtifactPreviewDrills,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionEvent {
    SelectionChanged(SelectionTarget),
    SceneChanged,
    FrameChanged,
    ToolChanged(WorkspaceTool),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionCommandResult {
    pub handled: bool,
    pub events: Vec<SessionEvent>,
}

fn layer_visibility_change_is_frame_only(scene: &BoardReviewSceneV1, layer_id: &str) -> bool {
    let retained_base_uses_layer = scene
        .components
        .iter()
        .any(|component| component.placement_layer == layer_id)
        || scene
            .component_graphics
            .iter()
            .any(|graphic| graphic.layer_id.as_deref() == Some(layer_id))
        || scene.pads.iter().any(|pad| {
            pad.layer_id == layer_id
                || pad.copper_layer_ids.iter().any(|id| id == layer_id)
                || pad.mask_layer_ids.iter().any(|id| id == layer_id)
                || pad.paste_layer_ids.iter().any(|id| id == layer_id)
        })
        || scene.tracks.iter().any(|track| track.layer_id == layer_id)
        || scene
            .vias
            .iter()
            .any(|via| via.start_layer_id == layer_id || via.end_layer_id == layer_id)
        || scene.zones.iter().any(|zone| zone.layer_id == layer_id);
    !retained_base_uses_layer
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReviewWorkspaceState {
    pub scene: BoardReviewSceneV1,
    /// Optional companion schematic scene, projected from the board project's
    /// sibling `.kicad_sch` alongside (not replacing) `scene`. Carried in
    /// workspace state so pane B can render real schematic geometry; `None`
    /// when no schematic is found. Consumer/scene state, not a journaled op.
    pub schematic_scene: Option<BoardReviewSceneV1>,
    pub review: RouteProposalReviewPayload,
    pub production: ProductionStatus,
    pub source_shards: SourceShardStatusSummary,
    pub checks: CheckRunReviewState,
    pub supervision: GuiSupervisionSnapshot,
    pub selection: SelectionTarget,
    pub active_review_target_id: String,
    pub tool: WorkspaceTool,
    pub authoring: AuthoringToolState,
    pub backing: Option<WorkspaceBacking>,
    pub last_command_status: Option<EditorCommandStatus>,
    pub ui: WorkspaceUiState,
}

pub const GUI_SUPERVISION_SNAPSHOT_CONTRACT: &str = "datum_gui_supervision_snapshot_v1";

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct GuiSupervisionSnapshot {
    pub contract: String,
    pub project_root: String,
    pub project_uuid: String,
    pub project_name: String,
    pub model_revision: String,
    pub scene_kind: String,
    pub read_only: bool,
    pub journal: GuiJournalSupervision,
    pub source_shards: SourceShardStatusSummary,
    pub scene: GuiSceneSupervision,
    pub checks: GuiCheckSupervision,
    pub data: GuiDataSupervision,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq, Default)]
pub struct GuiJournalSupervision {
    pub applied_transaction_count: usize,
    pub accepted_transaction_tip: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq, Default)]
pub struct GuiSceneSupervision {
    pub component_count: usize,
    pub pad_count: usize,
    pub track_count: usize,
    pub via_count: usize,
    pub zone_count: usize,
    pub board_text_count: usize,
    pub board_graphic_count: usize,
    pub layer_count: usize,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq, Default)]
pub struct GuiCheckSupervision {
    pub check_run_id: Option<String>,
    pub model_revision: Option<String>,
    pub profile_id: Option<String>,
    pub status: Option<String>,
    pub finding_count: usize,
    pub proposal_ref_count: usize,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq, Default)]
pub struct GuiDataSupervision {
    pub output_job_count: usize,
    pub artifact_count: usize,
    pub artifact_run_count: usize,
    pub proposal_count: usize,
    pub manufacturing_plan_count: usize,
    pub panel_projection_count: usize,
    pub latest_status: Option<String>,
}

impl Default for GuiSupervisionSnapshot {
    fn default() -> Self {
        Self {
            contract: GUI_SUPERVISION_SNAPSHOT_CONTRACT.to_string(),
            project_root: String::new(),
            project_uuid: String::new(),
            project_name: String::new(),
            model_revision: String::new(),
            scene_kind: String::new(),
            read_only: true,
            journal: GuiJournalSupervision::default(),
            source_shards: SourceShardStatusSummary::default(),
            scene: GuiSceneSupervision::default(),
            checks: GuiCheckSupervision::default(),
            data: GuiDataSupervision::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LiveDesignSession {
    workspace: ReviewWorkspaceState,
}

impl LiveDesignSession {
    pub fn new(workspace: ReviewWorkspaceState) -> Self {
        Self { workspace }
    }

    pub fn workspace(&self) -> &ReviewWorkspaceState {
        &self.workspace
    }

    pub fn workspace_mut(&mut self) -> &mut ReviewWorkspaceState {
        &mut self.workspace
    }

    pub fn apply(&mut self, command: SessionCommand) -> SessionCommandResult {
        match command {
            SessionCommand::SetTool(tool) => {
                if self.workspace.set_tool(tool) {
                    SessionCommandResult {
                        handled: true,
                        events: vec![SessionEvent::ToolChanged(tool), SessionEvent::FrameChanged],
                    }
                } else {
                    SessionCommandResult {
                        handled: false,
                        events: Vec::new(),
                    }
                }
            }
            SessionCommand::BeginAuthoringGesture {
                world,
                target_object_id,
            } => {
                if self
                    .workspace
                    .begin_authoring_gesture(world, target_object_id)
                {
                    SessionCommandResult {
                        handled: true,
                        events: vec![SessionEvent::FrameChanged],
                    }
                } else {
                    SessionCommandResult {
                        handled: false,
                        events: Vec::new(),
                    }
                }
            }
            SessionCommand::PreviewAuthoringGesture {
                world,
                target_object_id,
            } => {
                if self
                    .workspace
                    .preview_authoring_gesture(world, target_object_id)
                {
                    SessionCommandResult {
                        handled: true,
                        events: vec![SessionEvent::FrameChanged],
                    }
                } else {
                    SessionCommandResult {
                        handled: false,
                        events: Vec::new(),
                    }
                }
            }
            SessionCommand::CancelAuthoringGesture => {
                if self.workspace.cancel_authoring_gesture() {
                    SessionCommandResult {
                        handled: true,
                        events: vec![SessionEvent::FrameChanged],
                    }
                } else {
                    SessionCommandResult {
                        handled: false,
                        events: Vec::new(),
                    }
                }
            }
            SessionCommand::SelectReviewAction(action_id) => {
                if self.workspace.select_review_action(&action_id) {
                    SessionCommandResult {
                        handled: true,
                        events: vec![
                            SessionEvent::SelectionChanged(self.workspace.selection.clone()),
                            SessionEvent::FrameChanged,
                        ],
                    }
                } else {
                    SessionCommandResult {
                        handled: false,
                        events: Vec::new(),
                    }
                }
            }
            SessionCommand::SelectAuthoredObject(object_id) => {
                if self.workspace.select_authored_object(&object_id) {
                    SessionCommandResult {
                        handled: true,
                        events: vec![
                            SessionEvent::SelectionChanged(self.workspace.selection.clone()),
                            SessionEvent::FrameChanged,
                        ],
                    }
                } else {
                    SessionCommandResult {
                        handled: false,
                        events: Vec::new(),
                    }
                }
            }
            SessionCommand::SelectCheckFinding(fingerprint) => {
                if self.workspace.select_check_finding(&fingerprint) {
                    SessionCommandResult {
                        handled: true,
                        events: vec![
                            SessionEvent::SelectionChanged(self.workspace.selection.clone()),
                            SessionEvent::FrameChanged,
                        ],
                    }
                } else {
                    SessionCommandResult {
                        handled: false,
                        events: Vec::new(),
                    }
                }
            }
            SessionCommand::ClearSelection => {
                self.workspace.clear_selection();
                SessionCommandResult {
                    handled: true,
                    events: vec![
                        SessionEvent::SelectionChanged(self.workspace.selection.clone()),
                        SessionEvent::FrameChanged,
                    ],
                }
            }
            SessionCommand::SelectPreviousReviewAction => {
                if self.workspace.select_previous_review_action() {
                    SessionCommandResult {
                        handled: true,
                        events: vec![
                            SessionEvent::SelectionChanged(self.workspace.selection.clone()),
                            SessionEvent::FrameChanged,
                        ],
                    }
                } else {
                    SessionCommandResult {
                        handled: false,
                        events: Vec::new(),
                    }
                }
            }
            SessionCommand::SelectNextReviewAction => {
                if self.workspace.select_next_review_action() {
                    SessionCommandResult {
                        handled: true,
                        events: vec![
                            SessionEvent::SelectionChanged(self.workspace.selection.clone()),
                            SessionEvent::FrameChanged,
                        ],
                    }
                } else {
                    SessionCommandResult {
                        handled: false,
                        events: Vec::new(),
                    }
                }
            }
            SessionCommand::ToggleShowAuthored => {
                if self.workspace.toggle_show_authored() {
                    SessionCommandResult {
                        handled: true,
                        events: vec![SessionEvent::FrameChanged],
                    }
                } else {
                    SessionCommandResult {
                        handled: false,
                        events: Vec::new(),
                    }
                }
            }
            SessionCommand::ToggleShowProposed => {
                if self.workspace.toggle_show_proposed() {
                    SessionCommandResult {
                        handled: true,
                        events: vec![SessionEvent::FrameChanged],
                    }
                } else {
                    SessionCommandResult {
                        handled: false,
                        events: Vec::new(),
                    }
                }
            }
            SessionCommand::ToggleShowUnrouted => {
                if self.workspace.toggle_show_unrouted() {
                    let event = if self.workspace.scene.unrouted_primitives.is_empty() {
                        SessionEvent::FrameChanged
                    } else {
                        SessionEvent::SceneChanged
                    };
                    SessionCommandResult {
                        handled: true,
                        events: vec![event],
                    }
                } else {
                    SessionCommandResult {
                        handled: false,
                        events: Vec::new(),
                    }
                }
            }
            SessionCommand::ToggleDimUnrelated => {
                if self.workspace.toggle_dim_unrelated() {
                    let event = if self.workspace.selected_review_action().is_none()
                        && matches!(self.workspace.selection, SelectionTarget::None)
                    {
                        SessionEvent::FrameChanged
                    } else {
                        SessionEvent::SceneChanged
                    };
                    SessionCommandResult {
                        handled: true,
                        events: vec![event],
                    }
                } else {
                    SessionCommandResult {
                        handled: false,
                        events: Vec::new(),
                    }
                }
            }
            SessionCommand::ToggleLayerVisibility(layer_id) => {
                if self.workspace.toggle_layer_visibility(&layer_id) {
                    let event = if layer_visibility_change_is_frame_only(
                        &self.workspace.scene,
                        &layer_id,
                    ) {
                        SessionEvent::FrameChanged
                    } else {
                        SessionEvent::SceneChanged
                    };
                    SessionCommandResult {
                        handled: true,
                        events: vec![event],
                    }
                } else {
                    SessionCommandResult {
                        handled: false,
                        events: Vec::new(),
                    }
                }
            }
            SessionCommand::FocusProductionArtifact(artifact_id) => {
                if self.workspace.focus_production_artifact(&artifact_id) {
                    SessionCommandResult {
                        handled: true,
                        events: vec![SessionEvent::FrameChanged],
                    }
                } else {
                    SessionCommandResult {
                        handled: false,
                        events: Vec::new(),
                    }
                }
            }
            SessionCommand::FocusProductionArtifactFile(path) => {
                if self.workspace.focus_production_artifact_file(&path) {
                    SessionCommandResult {
                        handled: true,
                        events: vec![SessionEvent::FrameChanged],
                    }
                } else {
                    SessionCommandResult {
                        handled: false,
                        events: Vec::new(),
                    }
                }
            }
            command => {
                self.apply_artifact_preview_command(command)
                    .unwrap_or(SessionCommandResult {
                        handled: false,
                        events: Vec::new(),
                    })
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReviewActionRow {
    pub action_id: String,
    pub title: String,
    pub subtitle: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LiveReviewRequest {
    pub project_root: PathBuf,
    pub board_file: Option<PathBuf>,
    pub artifact_path: Option<PathBuf>,
    pub net_uuid: Option<String>,
    pub from_anchor_pad_uuid: Option<String>,
    pub to_anchor_pad_uuid: Option<String>,
    pub profile: Option<String>,
    /// The original KiCad `.kicad_pcb` source this request was materialized
    /// from, if any. Used solely to locate the companion `.kicad_sch` for
    /// display in pane B when the board itself now loads from a native project
    /// whose sibling directory holds no schematic. `None` for direct board-file
    /// loads and for native projects with no KiCad origin.
    pub kicad_board_source: Option<PathBuf>,
}

pub fn ensure_known_good_demo_request() -> Result<LiveReviewRequest> {
    static DEMO_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    let _guard = DEMO_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .expect("demo lock should not be poisoned");
    let root = std::env::temp_dir().join("datum-gui-m7-known-good");
    known_good_demo::write_known_good_demo_project(&root)?;
    Ok(LiveReviewRequest {
        project_root: root,
        board_file: None,
        artifact_path: None,
        net_uuid: Some("00000000-0000-0000-0000-00000000c200".to_string()),
        from_anchor_pad_uuid: Some("00000000-0000-0000-0000-00000000c218".to_string()),
        to_anchor_pad_uuid: Some("00000000-0000-0000-0000-00000000c219".to_string()),
        profile: Some("default".to_string()),
        kicad_board_source: None,
    })
}

pub fn materialize_kicad_board_request(
    board_file: &Path,
    project_root: Option<PathBuf>,
) -> Result<LiveReviewRequest> {
    let source = board_file
        .canonicalize()
        .with_context(|| format!("failed to resolve KiCad board {}", board_file.display()))?;
    let root =
        project_root.unwrap_or_else(|| default_materialized_kicad_board_project_root(&source));
    let root_display = root.display().to_string();
    let source_display = source.display().to_string();
    let cli = cli_prefix();

    if !root.join("project.json").is_file() {
        let project_name = materialized_kicad_board_project_name(&source);
        run_cli_json_owned::<Value>(
            &cli,
            &[
                "project".to_string(),
                "new".to_string(),
                root_display.clone(),
                "--name".to_string(),
                project_name,
            ],
        )
        .with_context(|| {
            format!(
                "failed to create native Datum project at {}",
                root.display()
            )
        })?;
    }

    run_cli_json_owned::<Value>(
        &cli,
        &[
            "project".to_string(),
            "import-kicad-board".to_string(),
            root_display,
            "--source".to_string(),
            source_display,
        ],
    )
    .with_context(|| {
        format!(
            "failed to materialize KiCad board {} into native Datum project {}",
            source.display(),
            root.display()
        )
    })?;

    Ok(LiveReviewRequest {
        project_root: root,
        board_file: None,
        artifact_path: None,
        net_uuid: None,
        from_anchor_pad_uuid: None,
        to_anchor_pad_uuid: None,
        profile: None,
        // Carry the original KiCad board so pane B can draw its companion
        // `.kicad_sch`; the materialized project holds no sibling schematic.
        kicad_board_source: Some(source),
    })
}

fn default_materialized_kicad_board_project_root(source: &Path) -> PathBuf {
    let mut hasher = DefaultHasher::new();
    source.display().to_string().hash(&mut hasher);
    let digest = hasher.finish();
    let stem = source
        .file_stem()
        .and_then(|value| value.to_str())
        .filter(|value| !value.is_empty())
        .unwrap_or("board");
    std::env::temp_dir()
        .join("datum-eda")
        .join("gui-imports")
        .join(format!("{stem}-{digest:016x}"))
}

fn materialized_kicad_board_project_name(source: &Path) -> String {
    let stem = source
        .file_stem()
        .and_then(|value| value.to_str())
        .filter(|value| !value.is_empty())
        .unwrap_or("Imported Board");
    format!("{stem} Datum Workspace")
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
struct ProjectInspectPayload {
    project_root: String,
    project_name: String,
    project_uuid: String,
    board_uuid: String,
    board_path: String,
}
#[derive(Debug, Clone, Deserialize, PartialEq, Eq, Default)]
struct OutputJobsPayload {
    output_job_count: usize,
    #[serde(default)]
    output_jobs: Vec<OutputJobStatusPayload>,
}
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
struct OutputJobStatusPayload {
    id: String,
    name: String,
    #[serde(default)]
    include: Vec<String>,
    #[serde(default)]
    prefix: String,
    #[serde(default)]
    output_dir: Option<PathBuf>,
    status: String,
    execution_count: usize,
    #[serde(default)]
    latest_run: Option<OutputJobRunPayload>,
    #[serde(default)]
    artifacts: Vec<OutputJobArtifactPayload>,
}
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
struct OutputJobRunPayload {
    run_id: String,
    #[serde(default)]
    run_sequence: u64,
    #[serde(default)]
    artifact_id: Option<String>,
}
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
struct OutputJobArtifactPayload {
    artifact_id: String,
    kind: String,
    #[serde(default)]
    project_id: Option<String>,
    #[serde(default)]
    model_revision: Option<String>,
    #[serde(default)]
    output_job: Option<String>,
    #[serde(default)]
    variant: Option<String>,
    #[serde(default)]
    generator_version: Option<String>,
    #[serde(default)]
    output_dir: Option<String>,
    #[serde(default)]
    validation_state: Option<String>,
    #[serde(default)]
    files: Vec<OutputJobArtifactFilePayload>,
    #[serde(default)]
    production_projections: Vec<OutputJobArtifactProjectionPayload>,
}
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
struct OutputJobArtifactFilePayload {
    path: PathBuf,
    sha256: String,
}
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
struct OutputJobArtifactProjectionPayload {
    projection_kind: String,
    projection_contract: String,
    model_revision: String,
    byte_count: usize,
    sha256: String,
}
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
struct ArtifactFilesPayload {
    artifact_id: String,
    kind: String,
    #[serde(default)]
    output_dir: Option<String>,
    validation_state: String,
    #[serde(default)]
    files: Vec<OutputJobArtifactFilePayload>,
    #[serde(default)]
    production_projections: Vec<OutputJobArtifactProjectionPayload>,
}
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
struct ArtifactFilePreviewPayload {
    file: std::path::PathBuf,
    preview_kind: String,
    hash_matches_metadata: bool,
    #[serde(default)]
    primitive_count: usize,
    #[serde(default)]
    primitives: Vec<ArtifactFilePreviewPrimitivePayload>,
    #[serde(default)]
    inspection: serde_json::Value,
}
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
struct ArtifactFilePreviewPrimitivePayload {
    kind: String,
    #[serde(default)]
    aperture_diameter_nm: Option<i64>,
    #[serde(default)]
    aperture_width_nm: Option<i64>,
    #[serde(default)]
    aperture_height_nm: Option<i64>,
    #[serde(default)]
    tool: Option<String>,
    #[serde(default)]
    diameter_mm: Option<String>,
    #[serde(default)]
    points: Vec<ArtifactFilePreviewPointPayload>,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
struct ArtifactFilePreviewPointPayload {
    x_nm: i64,
    y_nm: i64,
}
#[derive(Debug, Clone, Deserialize, PartialEq, Eq, Default)]
struct ManufacturingPlansPayload {
    manufacturing_plan_count: usize,
    #[serde(default)]
    manufacturing_plans: Vec<ManufacturingPlanPayload>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
struct ManufacturingPlanPayload {
    id: String,
    name: String,
    board_or_panel: String,
    #[serde(default)]
    variant: Option<String>,
    prefix: String,
    object_revision: u64,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq, Default)]
struct PanelProjectionsPayload {
    panel_projection_count: usize,
    #[serde(default)]
    panel_projections: Vec<PanelProjectionPayload>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
struct PanelProjectionPayload {
    id: String,
    name: String,
    #[serde(default)]
    board_instances: Vec<PanelBoardInstancePayload>,
    object_revision: u64,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
struct PanelBoardInstancePayload {
    board: String,
    x_nm: i64,
    y_nm: i64,
    rotation_deg: i32,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
struct OutlinePayload {
    vertices: Vec<PointNm>,
    closed: bool,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
struct BoardComponentPayload {
    uuid: String,
    reference: String,
    value: String,
    position: PointNm,
    rotation: i32,
    layer: i32,
    locked: bool,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
struct BoardPadPayload {
    uuid: String,
    package: String,
    name: String,
    net: Option<String>,
    position: PointNm,
    layer: i32,
    #[serde(default)]
    copper_layers: Vec<i32>,
    shape: String,
    diameter: i64,
    width: i64,
    height: i64,
    #[serde(default = "default_roundrect_rratio_ppm")]
    roundrect_rratio_ppm: u32,
    #[serde(default)]
    mask_layers: Vec<i32>,
    #[serde(default)]
    paste_layers: Vec<i32>,
    #[serde(default)]
    solder_mask_margin_nm: i64,
    #[serde(default)]
    solder_paste_margin_nm: i64,
    #[serde(default)]
    solder_paste_margin_ratio_ppm: i32,
    #[serde(default)]
    drill: Option<i64>,
    #[serde(default)]
    rotation: i32,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
struct BoardTrackPayload {
    uuid: String,
    net: String,
    from: PointNm,
    to: PointNm,
    width: i64,
    layer: i32,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
struct BoardViaPayload {
    uuid: String,
    net: String,
    position: PointNm,
    drill: i64,
    diameter: i64,
    from_layer: i32,
    to_layer: i32,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
struct BoardZonePayload {
    uuid: String,
    net: String,
    polygon: OutlinePayload,
    layer: i32,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
struct CandidateExplainSelectedPathPayload {
    points: Vec<PointNm>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
struct CandidateExplainSelectedSpanPayload {
    from: PointNm,
    to: PointNm,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
struct CandidateExplainPayload {
    #[serde(default)]
    selected_path: Option<CandidateExplainSelectedPathPayload>,
    #[serde(default)]
    selected_span: Option<CandidateExplainSelectedSpanPayload>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
struct ComponentSilkscreenPayload {
    component_uuid: String,
    #[serde(default)]
    lines: Vec<ComponentGraphicLinePayload>,
    #[serde(default)]
    arcs: Vec<ComponentGraphicArcPayload>,
    #[serde(default)]
    circles: Vec<ComponentGraphicCirclePayload>,
    #[serde(default)]
    polygons: Vec<ComponentGraphicPolygonPayload>,
    #[serde(default)]
    polylines: Vec<ComponentGraphicPolylinePayload>,
    #[serde(default)]
    texts: Vec<ComponentGraphicTextPayload>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
struct ComponentMechanicalPayload {
    component_uuid: String,
    #[serde(default)]
    lines: Vec<ComponentGraphicLinePayload>,
    #[serde(default)]
    arcs: Vec<ComponentGraphicArcPayload>,
    #[serde(default)]
    circles: Vec<ComponentGraphicCirclePayload>,
    #[serde(default)]
    polygons: Vec<ComponentGraphicPolygonPayload>,
    #[serde(default)]
    polylines: Vec<ComponentGraphicPolylinePayload>,
    #[serde(default)]
    texts: Vec<ComponentGraphicTextPayload>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
struct ComponentGraphicTextPayload {
    text: String,
    position: PointNm,
    rotation: i32,
    height_nm: i64,
    stroke_width_nm: i64,
    layer: i32,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
struct ComponentGraphicLinePayload {
    from: PointNm,
    to: PointNm,
    width_nm: i64,
    layer: i32,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
struct ComponentGraphicArcPayload {
    center: PointNm,
    radius_nm: i64,
    start_angle: i32,
    end_angle: i32,
    width_nm: i64,
    layer: i32,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
struct ComponentGraphicCirclePayload {
    center: PointNm,
    radius_nm: i64,
    width_nm: i64,
    layer: i32,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
struct ComponentGraphicPolygonPayload {
    vertices: Vec<PointNm>,
    layer: i32,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
struct ComponentGraphicPolylinePayload {
    vertices: Vec<PointNm>,
    width_nm: i64,
    layer: i32,
}

fn deserialize_null_default<'de, D, T>(deserializer: D) -> std::result::Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de> + Default,
{
    Ok(Option::<T>::deserialize(deserializer)?.unwrap_or_default())
}

fn overlay_path_for_action(
    action_index: usize,
    action: &RouteProposalActionPayload,
    review: &RouteProposalReviewPayload,
    selected_path_points: Option<&[PointNm]>,
) -> Vec<PointNm> {
    if let Some(points) = selected_path_points
        && review.proposal_actions.len() > 1
        && points.len() > review.proposal_actions.len()
    {
        let start = action
            .selected_path_segment_index
            .min(points.len().saturating_sub(2));
        let end = (start + 1).min(points.len() - 1);
        if end > start {
            return vec![points[start], points[end]];
        }
    }
    if action_index == 0 && review.proposal_actions.len() == 1 {
        selected_path_points
            .map(|points| points.to_vec())
            .unwrap_or_else(|| vec![action.from, action.to])
    } else {
        vec![action.from, action.to]
    }
}

impl ReviewWorkspaceState {
    pub fn new(scene: BoardReviewSceneV1, review: RouteProposalReviewPayload) -> Self {
        let layer_visibility = scene
            .layers
            .iter()
            .map(|layer| (layer.layer_id.clone(), layer.visible_by_default))
            .collect();
        let active_layer_id = scene
            .layers
            .iter()
            .find(|layer| layer.visible_by_default)
            .or_else(|| scene.layers.first())
            .map(|layer| layer.layer_id.clone());
        let has_review_actions = !review.proposal_actions.is_empty();
        let active_review_target_id = review
            .proposal_actions
            .first()
            .map(|action| action.action_id.clone())
            .unwrap_or_else(|| "no-proposal-action".to_string());
        Self {
            scene,
            schematic_scene: None,
            review,
            production: ProductionStatus::default(),
            source_shards: SourceShardStatusSummary::default(),
            checks: CheckRunReviewState::default(),
            selection: if has_review_actions {
                SelectionTarget::ReviewAction(active_review_target_id.clone())
            } else {
                SelectionTarget::None
            },
            active_review_target_id,
            tool: WorkspaceTool::Select,
            authoring: AuthoringToolState {
                snap: AuthoringSnapState::default(),
                gesture: AuthoringGestureState::idle(WorkspaceTool::Select),
            },
            backing: None,
            last_command_status: None,
            supervision: GuiSupervisionSnapshot::default(),
            ui: WorkspaceUiState {
                active_dock_tab: None,
                active_menu: None,
                marking_menu: None,
                dock_height_px: 220,
                hovered_object_id: None,
                cursor_pos: None,
                crosshair_style: CrosshairStyle::default(),
                filters: WorkspaceFilterState {
                    show_authored: true,
                    show_proposed: true,
                    show_unrouted: true,
                    dim_unrelated: has_review_actions,
                    active_layer_id,
                    layer_visibility,
                },
                terminal: TerminalLaneState {
                    lines: vec![
                        "datum terminal ready".to_string(),
                        "shell session starts in the active project root".to_string(),
                    ],
                    styled_lines: Vec::new(),
                    activity_summary: Vec::new(),
                    tabs: Vec::new(),
                    active_session_id: None,
                    rename_session_id: None,
                    title: None,
                    current_working_directory: None,
                    bell_count: 0,
                    input: String::new(),
                    cursor: 0,
                    columns: 80,
                    rows: 24,
                    screen_cursor_row: 0,
                    screen_cursor_col: 0,
                    screen_cursor_visible: true,
                    screen_cursor_style: None,
                    application_cursor_keys: false,
                    application_keypad: false,
                    focus_event_reporting: false,
                    mouse_reporting_mode: None,
                    mouse_coordinate_encoding: None,
                    scroll_offset: 0,
                    status: "running".to_string(),
                },
                console: ConsoleLaneState::default(),
                artifact_preview: ArtifactPreviewViewportState::default(),
                layout: WorkspaceLayout::default(),
            },
        }
    }

    pub fn review_rows(&self) -> Vec<ReviewActionRow> {
        self.review
            .proposal_actions
            .iter()
            .map(|action| ReviewActionRow {
                action_id: action.action_id.clone(),
                title: format!(
                    "{} {}",
                    action.proposal_action.to_uppercase(),
                    action.selected_path_segment_index + 1
                ),
                subtitle: format!("LAYER {} {} NM", action.layer, action.width_nm),
            })
            .collect()
    }

    pub fn selected_review_action(&self) -> Option<&RouteProposalActionPayload> {
        self.review
            .proposal_actions
            .iter()
            .find(|action| action.action_id == self.active_review_target_id)
    }

    pub fn select_previous_review_action(&mut self) -> bool {
        let Some(index) = self
            .review
            .proposal_actions
            .iter()
            .position(|action| action.action_id == self.active_review_target_id)
        else {
            return false;
        };
        if index == 0 {
            return false;
        }
        let action_id = self.review.proposal_actions[index - 1].action_id.clone();
        self.select_review_action(&action_id)
    }

    pub fn select_next_review_action(&mut self) -> bool {
        let Some(index) = self
            .review
            .proposal_actions
            .iter()
            .position(|action| action.action_id == self.active_review_target_id)
        else {
            return false;
        };
        if index + 1 >= self.review.proposal_actions.len() {
            return false;
        }
        let action_id = self.review.proposal_actions[index + 1].action_id.clone();
        self.select_review_action(&action_id)
    }

    pub fn toggle_show_authored(&mut self) -> bool {
        self.ui.filters.show_authored = !self.ui.filters.show_authored;
        true
    }

    pub fn toggle_show_proposed(&mut self) -> bool {
        self.ui.filters.show_proposed = !self.ui.filters.show_proposed;
        true
    }

    pub fn toggle_show_unrouted(&mut self) -> bool {
        self.ui.filters.show_unrouted = !self.ui.filters.show_unrouted;
        true
    }

    pub fn toggle_dim_unrelated(&mut self) -> bool {
        self.ui.filters.dim_unrelated = !self.ui.filters.dim_unrelated;
        true
    }

    pub fn toggle_layer_visibility(&mut self, layer_id: &str) -> bool {
        let entry = self
            .ui
            .filters
            .layer_visibility
            .entry(layer_id.to_string())
            .or_insert(true);
        *entry = !*entry;
        true
    }

    pub fn selected_segment_evidence(&self) -> Option<&RouteProposalSegmentEvidence> {
        self.selected_review_action().and_then(|action| {
            self.review
                .segment_evidence
                .iter()
                .find(|segment| {
                    segment.layer_segment_index
                        == action.selected_path_layer_segment_index.unwrap_or(0)
                })
                .or_else(|| self.review.segment_evidence.first())
        })
    }

    pub fn review_action_index(&self, action_id: &str) -> Option<usize> {
        self.review
            .proposal_actions
            .iter()
            .position(|action| action.action_id == action_id)
    }

    pub fn select_review_action(&mut self, action_id: &str) -> bool {
        if self
            .review
            .proposal_actions
            .iter()
            .any(|action| action.action_id == action_id)
        {
            self.active_review_target_id = action_id.to_string();
            self.selection = SelectionTarget::ReviewAction(action_id.to_string());
            true
        } else {
            false
        }
    }

    pub fn select_authored_object(&mut self, object_id: &str) -> bool {
        let normalized_object_id = object_id
            .strip_prefix("component:")
            .and_then(|component_uuid| {
                self.scene
                    .components
                    .iter()
                    .find(|component| component.component_uuid == component_uuid)
                    .map(|component| component.object_id.as_str())
            })
            .unwrap_or(object_id);
        let exists = self
            .scene
            .components
            .iter()
            .any(|c| c.object_id == normalized_object_id)
            || self
                .scene
                .pads
                .iter()
                .any(|p| p.object_id == normalized_object_id)
            || self
                .scene
                .tracks
                .iter()
                .any(|t| t.object_id == normalized_object_id)
            || self
                .scene
                .vias
                .iter()
                .any(|v| v.object_id == normalized_object_id)
            || self
                .scene
                .zones
                .iter()
                .any(|z| z.object_id == normalized_object_id)
            || self
                .scene
                .board_graphics
                .iter()
                .any(|g| g.object_id == normalized_object_id)
            || self
                .scene
                .outline
                .iter()
                .any(|outline| outline.object_id == normalized_object_id)
            || self
                .scene
                .board_texts
                .iter()
                .any(|t| t.object_id == normalized_object_id);
        if exists {
            self.selection = SelectionTarget::AuthoredObject(normalized_object_id.to_string());
            true
        } else {
            false
        }
    }

    pub fn select_check_finding(&mut self, fingerprint: &str) -> bool {
        if fingerprint.is_empty() {
            return false;
        }
        let exists = self
            .checks
            .findings
            .iter()
            .any(|finding| finding.fingerprint == fingerprint);
        if exists {
            self.selection = SelectionTarget::CheckFinding(fingerprint.to_string());
            true
        } else {
            false
        }
    }

    pub fn clear_selection(&mut self) {
        self.selection = SelectionTarget::None;
    }

    pub fn set_tool(&mut self, tool: WorkspaceTool) -> bool {
        if self.tool == tool {
            return false;
        }
        self.tool = tool;
        self.authoring.gesture = AuthoringGestureState::idle(tool);
        self.last_command_status = Some(EditorCommandStatus {
            action: "set_tool".to_string(),
            detail: format!("tool {}", tool.label()),
        });
        true
    }

    pub fn snap_authoring_point(&self, point: PointNm) -> PointNm {
        let snap = self.authoring.snap;
        if !snap.enabled || snap.grid_nm <= 0 {
            return point;
        }
        PointNm {
            x: snap_nm(point.x, snap.grid_nm),
            y: snap_nm(point.y, snap.grid_nm),
        }
    }

    pub fn begin_authoring_gesture(
        &mut self,
        world: PointNm,
        target_object_id: Option<String>,
    ) -> bool {
        if self.tool == WorkspaceTool::Select {
            return false;
        }
        let point = self.snap_authoring_point(world);
        self.authoring.gesture = AuthoringGestureState {
            tool: self.tool,
            anchor: Some(point),
            preview: Some(point),
            target_object_id,
        };
        self.last_command_status = Some(EditorCommandStatus {
            action: "begin_authoring_gesture".to_string(),
            detail: format!("{} @ {},{}", self.tool.label(), point.x, point.y),
        });
        true
    }

    pub fn preview_authoring_gesture(
        &mut self,
        world: PointNm,
        target_object_id: Option<String>,
    ) -> bool {
        if !self.authoring.gesture.is_active() {
            return false;
        }
        let point = self.snap_authoring_point(world);
        if self.authoring.gesture.preview == Some(point)
            && self.authoring.gesture.target_object_id == target_object_id
        {
            return false;
        }
        self.authoring.gesture.preview = Some(point);
        self.authoring.gesture.target_object_id = target_object_id;
        true
    }

    pub fn cancel_authoring_gesture(&mut self) -> bool {
        if !self.authoring.gesture.is_active() {
            return false;
        }
        self.authoring.gesture = AuthoringGestureState::idle(self.tool);
        self.last_command_status = Some(EditorCommandStatus {
            action: "cancel_authoring_gesture".to_string(),
            detail: format!("cancelled {}", self.tool.label()),
        });
        true
    }

    pub fn finish_draw_board_track_handoff(
        &mut self,
        world: PointNm,
    ) -> Option<TerminalCommandHandoff> {
        if self.tool != WorkspaceTool::DrawBoardTrack {
            return None;
        }
        let from = self.authoring.gesture.anchor?;
        let to = self.snap_authoring_point(world);
        if from == to {
            self.authoring.gesture.preview = Some(to);
            self.last_command_status = Some(EditorCommandStatus {
                action: "draw_board_track".to_string(),
                detail: "track end must differ from start".to_string(),
            });
            return None;
        }
        let handoff = self.draw_board_track_handoff(from, to)?;
        self.authoring.gesture = AuthoringGestureState::idle(self.tool);
        self.last_command_status = Some(EditorCommandStatus {
            action: "draw_board_track".to_string(),
            detail: format!("queued track {},{} -> {},{}", from.x, from.y, to.x, to.y),
        });
        Some(handoff)
    }

    pub fn finish_place_board_via_handoff(
        &mut self,
        world: PointNm,
    ) -> Option<TerminalCommandHandoff> {
        if self.tool != WorkspaceTool::PlaceBoardVia {
            return None;
        }
        let point = self.snap_authoring_point(world);
        let handoff = self.place_board_via_handoff(point)?;
        self.authoring.gesture = AuthoringGestureState::idle(self.tool);
        self.last_command_status = Some(EditorCommandStatus {
            action: "place_board_via".to_string(),
            detail: format!("queued via @ {},{}", point.x, point.y),
        });
        Some(handoff)
    }

    pub fn finish_place_board_text_handoff(
        &mut self,
        world: PointNm,
    ) -> Option<TerminalCommandHandoff> {
        if self.tool != WorkspaceTool::PlaceBoardText {
            return None;
        }
        let point = self.snap_authoring_point(world);
        let handoff = self.place_board_text_handoff(point)?;
        self.authoring.gesture = AuthoringGestureState::idle(self.tool);
        self.last_command_status = Some(EditorCommandStatus {
            action: "place_board_text".to_string(),
            detail: format!("queued board text @ {},{}", point.x, point.y),
        });
        Some(handoff)
    }

    pub fn finish_move_component_handoff(
        &mut self,
        world: PointNm,
    ) -> Option<TerminalCommandHandoff> {
        if self.tool != WorkspaceTool::Move {
            return None;
        }
        let target = self.authoring.gesture.target_object_id.clone()?;
        let point = self.snap_authoring_point(world);
        let handoff = self.move_component_handoff(&target, point)?;
        self.authoring.gesture = AuthoringGestureState::idle(self.tool);
        self.last_command_status = Some(EditorCommandStatus {
            action: "move_board_component".to_string(),
            detail: format!("queued component move {target} -> {},{}", point.x, point.y),
        });
        Some(handoff)
    }

    pub fn draw_board_track_handoff(
        &self,
        from: PointNm,
        to: PointNm,
    ) -> Option<TerminalCommandHandoff> {
        let backing = self.backing.as_ref()?;
        let net_uuid = self.review.net_uuid.as_deref().or_else(|| {
            self.scene
                .net_display
                .first()
                .map(|net| net.net_uuid.as_str())
        })?;
        let layer = self
            .review
            .proposal_actions
            .first()
            .map(|action| action.layer)
            .unwrap_or(1);
        let width_nm = self
            .review
            .proposal_actions
            .first()
            .map(|action| action.width_nm)
            .unwrap_or(200_000);
        let root = backing.request.project_root.to_string_lossy();
        let command = [
            shell_quote_arg("datum-eda"),
            shell_quote_arg("project"),
            shell_quote_arg("draw-board-track"),
            shell_quote_arg(&root),
            shell_quote_arg("--net"),
            shell_quote_arg(net_uuid),
            shell_quote_arg("--from-x-nm"),
            shell_quote_arg(&from.x.to_string()),
            shell_quote_arg("--from-y-nm"),
            shell_quote_arg(&from.y.to_string()),
            shell_quote_arg("--to-x-nm"),
            shell_quote_arg(&to.x.to_string()),
            shell_quote_arg("--to-y-nm"),
            shell_quote_arg(&to.y.to_string()),
            shell_quote_arg("--width-nm"),
            shell_quote_arg(&width_nm.to_string()),
            shell_quote_arg("--layer"),
            shell_quote_arg(&layer.to_string()),
        ]
        .join(" ");
        Some(TerminalCommandHandoff {
            command_id: "datum.pcb.draw_board_track".to_string(),
            mcp_alias: Some("datum.pcb.draw_board_track".to_string()),
            command,
        })
    }

    pub fn place_board_via_handoff(&self, at: PointNm) -> Option<TerminalCommandHandoff> {
        let backing = self.backing.as_ref()?;
        let net_uuid = self.default_authoring_net_uuid()?;
        let root = backing.request.project_root.to_string_lossy();
        let command = [
            shell_quote_arg("datum-eda"),
            shell_quote_arg("project"),
            shell_quote_arg("place-board-via"),
            shell_quote_arg(&root),
            shell_quote_arg("--net"),
            shell_quote_arg(net_uuid),
            shell_quote_arg("--x-nm"),
            shell_quote_arg(&at.x.to_string()),
            shell_quote_arg("--y-nm"),
            shell_quote_arg(&at.y.to_string()),
            shell_quote_arg("--drill-nm"),
            shell_quote_arg("300000"),
            shell_quote_arg("--diameter-nm"),
            shell_quote_arg("600000"),
            shell_quote_arg("--from-layer"),
            shell_quote_arg("1"),
            shell_quote_arg("--to-layer"),
            shell_quote_arg("16"),
        ]
        .join(" ");
        Some(TerminalCommandHandoff {
            command_id: "datum.pcb.place_board_via".to_string(),
            mcp_alias: Some("datum.pcb.place_board_via".to_string()),
            command,
        })
    }

    pub fn place_board_text_handoff(&self, at: PointNm) -> Option<TerminalCommandHandoff> {
        let backing = self.backing.as_ref()?;
        let root = backing.request.project_root.to_string_lossy();
        let command = [
            shell_quote_arg("datum-eda"),
            shell_quote_arg("project"),
            shell_quote_arg("place-board-text"),
            shell_quote_arg(&root),
            shell_quote_arg("--text"),
            shell_quote_arg("TEXT"),
            shell_quote_arg("--x-nm"),
            shell_quote_arg(&at.x.to_string()),
            shell_quote_arg("--y-nm"),
            shell_quote_arg(&at.y.to_string()),
            shell_quote_arg("--render-intent"),
            shell_quote_arg("annotation"),
            shell_quote_arg("--h-align"),
            shell_quote_arg("center"),
            shell_quote_arg("--v-align"),
            shell_quote_arg("center"),
            shell_quote_arg("--layer"),
            shell_quote_arg("21"),
        ]
        .join(" ");
        Some(TerminalCommandHandoff {
            command_id: "datum.pcb.place_board_text".to_string(),
            mcp_alias: Some("datum.pcb.place_board_text".to_string()),
            command,
        })
    }

    pub fn delete_authored_object_handoff(
        &self,
        target_object_id: &str,
    ) -> Option<TerminalCommandHandoff> {
        let backing = self.backing.as_ref()?;
        let root = backing.request.project_root.to_string_lossy();
        let (command_id, verb, flag, uuid) = delete_command_parts_from_object_id(target_object_id)?;
        let command = [
            shell_quote_arg("datum-eda"),
            shell_quote_arg("project"),
            shell_quote_arg(verb),
            shell_quote_arg(&root),
            shell_quote_arg(flag),
            shell_quote_arg(uuid),
        ]
        .join(" ");
        Some(TerminalCommandHandoff {
            command_id: command_id.to_string(),
            mcp_alias: Some(command_id.to_string()),
            command,
        })
    }

    pub fn move_component_handoff(
        &self,
        target_object_id: &str,
        to: PointNm,
    ) -> Option<TerminalCommandHandoff> {
        let backing = self.backing.as_ref()?;
        let component = self.component_from_target_object_id(target_object_id)?;
        let root = backing.request.project_root.to_string_lossy();
        let command = [
            shell_quote_arg("datum-eda"),
            shell_quote_arg("project"),
            shell_quote_arg("move-board-component"),
            shell_quote_arg(&root),
            shell_quote_arg("--component"),
            shell_quote_arg(&component.component_uuid),
            shell_quote_arg("--x-nm"),
            shell_quote_arg(&to.x.to_string()),
            shell_quote_arg("--y-nm"),
            shell_quote_arg(&to.y.to_string()),
        ]
        .join(" ");
        Some(TerminalCommandHandoff {
            command_id: "datum.pcb.move_board_component".to_string(),
            mcp_alias: Some("datum.pcb.move_board_component".to_string()),
            command,
        })
    }

    fn default_authoring_net_uuid(&self) -> Option<&str> {
        self.review.net_uuid.as_deref().or_else(|| {
            self.scene
                .net_display
                .first()
                .map(|net| net.net_uuid.as_str())
        })
    }

    fn component_from_target_object_id(&self, target_object_id: &str) -> Option<&ComponentBounds> {
        let normalized = target_object_id
            .strip_prefix("component:")
            .and_then(|component_uuid| {
                self.scene
                    .components
                    .iter()
                    .find(|component| component.component_uuid == component_uuid)
                    .map(|component| component.object_id.as_str())
            })
            .unwrap_or(target_object_id);
        self.scene
            .components
            .iter()
            .find(|component| component.object_id == normalized)
    }

    pub fn selected_component(&self) -> Option<&ComponentBounds> {
        let SelectionTarget::AuthoredObject(object_id) = &self.selection else {
            return None;
        };
        self.scene
            .components
            .iter()
            .find(|component| &component.object_id == object_id)
    }
}

fn snap_nm(value: i64, grid_nm: i64) -> i64 {
    let half = grid_nm / 2;
    if value >= 0 {
        ((value + half) / grid_nm) * grid_nm
    } else {
        ((value - half) / grid_nm) * grid_nm
    }
}

fn shell_quote_arg(value: &str) -> String {
    if value
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-' | '.' | '/' | ':' | '='))
    {
        value.to_string()
    } else {
        format!("'{}'", value.replace('\'', "'\\''"))
    }
}

fn delete_command_parts_from_object_id(
    object_id: &str,
) -> Option<(&'static str, &'static str, &'static str, &str)> {
    if let Some(uuid) = object_id.strip_prefix("track:") {
        return Some((
            "datum.pcb.delete_board_track",
            "delete-board-track",
            "--track",
            uuid,
        ));
    }
    if let Some(uuid) = object_id.strip_prefix("via:") {
        return Some((
            "datum.pcb.delete_board_via",
            "delete-board-via",
            "--via",
            uuid,
        ));
    }
    if let Some(uuid) = object_id.strip_prefix("component:") {
        return Some((
            "datum.pcb.delete_board_component",
            "delete-board-component",
            "--component",
            uuid,
        ));
    }
    if let Some(uuid) = object_id.strip_prefix("board-text:") {
        return Some((
            "datum.pcb.delete_board_text",
            "delete-board-text",
            "--text",
            uuid,
        ));
    }
    None
}
pub fn load_live_workspace_state(request: &LiveReviewRequest) -> Result<ReviewWorkspaceState> {
    load_workspace_state_impl(request, true)
}
pub fn load_board_editor_workspace_state(
    request: &LiveReviewRequest,
) -> Result<ReviewWorkspaceState> {
    load_workspace_state_impl(request, false)
}
pub fn refresh_production_status(request: &LiveReviewRequest) -> Result<ProductionStatus> {
    load_production_status(&cli_prefix(), request)
}
pub fn refresh_check_run_review_state(request: &LiveReviewRequest) -> Result<CheckRunReviewState> {
    load_check_run_review_state(&cli_prefix(), request)
}
pub fn production_status_from_output_jobs_json(payload: &str) -> Result<ProductionStatus> {
    let payload: OutputJobsPayload =
        serde_json::from_str(payload).context("failed to decode output-job list JSON")?;
    Ok(production_payloads_to_production_status(
        payload,
        ArtifactListPayload::default(),
        ProposalsPayload::default(),
        ManufacturingPlansPayload::default(),
        PanelProjectionsPayload::default(),
    ))
}
pub fn production_status_from_artifacts_json(payload: &str) -> Result<ProductionStatus> {
    let payload: ArtifactListPayload =
        serde_json::from_str(payload).context("failed to decode artifact list JSON")?;
    Ok(production_payloads_to_production_status(
        OutputJobsPayload::default(),
        payload,
        ProposalsPayload::default(),
        ManufacturingPlansPayload::default(),
        PanelProjectionsPayload::default(),
    ))
}
pub fn production_artifact_detail_from_files_json(
    payload: &str,
) -> Result<ProductionArtifactDetail> {
    let payload: ArtifactFilesPayload =
        serde_json::from_str(payload).context("failed to decode artifact files JSON")?;
    Ok(artifact_files_payload_to_detail(payload))
}
pub fn production_artifact_file_preview_from_json(
    payload: &str,
) -> Result<ProductionArtifactFilePreviewSummary> {
    let payload: ArtifactFilePreviewPayload =
        serde_json::from_str(payload).context("failed to decode artifact preview JSON")?;
    Ok(artifact_preview_payload_to_summary(payload))
}
pub fn production_status_from_production_json(
    output_jobs: &str,
    manufacturing_plans: &str,
    panel_projections: &str,
) -> Result<ProductionStatus> {
    let output_jobs: OutputJobsPayload =
        serde_json::from_str(output_jobs).context("failed to decode output-job list JSON")?;
    let manufacturing_plans: ManufacturingPlansPayload = serde_json::from_str(manufacturing_plans)
        .context("failed to decode manufacturing-plan list JSON")?;
    let panel_projections: PanelProjectionsPayload = serde_json::from_str(panel_projections)
        .context("failed to decode panel-projection list JSON")?;
    Ok(production_payloads_to_production_status(
        output_jobs,
        ArtifactListPayload::default(),
        ProposalsPayload::default(),
        manufacturing_plans,
        panel_projections,
    ))
}
fn load_workspace_state_impl(
    request: &LiveReviewRequest,
    include_review: bool,
) -> Result<ReviewWorkspaceState> {
    let workspace_started = std::time::Instant::now();
    let cli = cli_prefix();
    let review_started = std::time::Instant::now();
    let review = if include_review && request.board_file.is_none() {
        load_live_route_review(&cli, request)?
    } else {
        empty_route_review_payload(request)
    };
    trace_protocol_timing(format!(
        "workspace review load {}ms",
        review_started.elapsed().as_millis()
    ));
    let selected_path_started = std::time::Instant::now();
    let selected_path_points = if include_review && request.board_file.is_none() {
        load_selected_candidate_path(&cli, request, review.selected_candidate.as_deref())?
    } else {
        None
    };
    trace_protocol_timing(format!(
        "workspace selected path load {}ms",
        selected_path_started.elapsed().as_millis()
    ));
    let scene_started = std::time::Instant::now();
    let (scene, board_path) = if let Some(board_file) = &request.board_file {
        load_scene_from_kicad_import(board_file)?
    } else {
        load_scene_from_engine(request)?
    };
    trace_protocol_timing(format!(
        "workspace scene load {}ms",
        scene_started.elapsed().as_millis()
    ));
    let mut scene = scene;
    let review_attach_started = std::time::Instant::now();
    attach_review_primitives(&mut scene, &review, selected_path_points.as_deref());
    trace_protocol_timing(format!(
        "workspace review attach {}ms",
        review_attach_started.elapsed().as_millis()
    ));
    let mut state = ReviewWorkspaceState::new(scene, review);
    // Locate the companion schematic from the ORIGINAL KiCad source when this
    // request was materialized into a native project (the materialized
    // `board/board.json` has no sibling `.kicad_sch`); otherwise derive it from
    // the loaded board path (the direct `--board <file>.kicad_pcb` path).
    let schematic_companion_base = request
        .kicad_board_source
        .as_deref()
        .unwrap_or(board_path.as_path());
    state.schematic_scene = load_sibling_schematic_scene(schematic_companion_base);
    state.production = load_production_status(&cli, request)?;
    state.source_shards = refresh_source_shard_status(request)?;
    state.checks = load_check_run_review_state(&cli, request)?;
    state.supervision = load_gui_supervision_snapshot(
        request,
        &state.scene,
        &state.production,
        &state.source_shards,
        &state.checks,
    )?;
    state.backing = Some(WorkspaceBacking {
        request: request.clone(),
        board_path,
    });
    trace_protocol_timing(format!(
        "workspace total {}ms",
        workspace_started.elapsed().as_millis()
    ));
    Ok(state)
}

/// Project the board project's companion schematic into a review scene, if
/// one exists next to the loaded board file (`<stem>.kicad_sch`). This is
/// carried alongside the board `scene` so pane B can draw real schematic
/// geometry; it reuses the existing schematic projector rather than authoring
/// new geometry. Robust by design: a missing or unparsable schematic yields
/// `None` and never disturbs the board load.
fn load_sibling_schematic_scene(board_path: &Path) -> Option<BoardReviewSceneV1> {
    let sibling = board_path.with_extension("kicad_sch");
    if !sibling.is_file() {
        return None;
    }
    match schematic_scene_import::load_scene_from_kicad_schematic_import(&sibling) {
        Ok((scene, _)) => Some(scene),
        Err(error) => {
            eprintln!(
                "datum-gui warning: skipped companion schematic {}: {error:#}",
                sibling.display()
            );
            None
        }
    }
}

fn load_gui_supervision_snapshot(
    request: &LiveReviewRequest,
    scene: &BoardReviewSceneV1,
    production: &ProductionStatus,
    source_shards: &SourceShardStatusSummary,
    checks: &CheckRunReviewState,
) -> Result<GuiSupervisionSnapshot> {
    let mut snapshot = GuiSupervisionSnapshot {
        contract: GUI_SUPERVISION_SNAPSHOT_CONTRACT.to_string(),
        project_root: request.project_root.display().to_string(),
        project_uuid: scene.project_uuid.clone(),
        project_name: scene.project_name.clone(),
        model_revision: scene.source_revision.clone(),
        scene_kind: scene.kind.clone(),
        read_only: true,
        journal: GuiJournalSupervision::default(),
        source_shards: source_shards.clone(),
        scene: GuiSceneSupervision {
            component_count: scene.components.len(),
            pad_count: scene.pads.len(),
            track_count: scene.tracks.len(),
            via_count: scene.vias.len(),
            zone_count: scene.zones.len(),
            board_text_count: scene.board_texts.len(),
            board_graphic_count: scene.board_graphics.len(),
            layer_count: scene.layers.len(),
        },
        checks: GuiCheckSupervision {
            check_run_id: checks.check_run_id.clone(),
            model_revision: checks.model_revision.clone(),
            profile_id: checks.profile_id.clone(),
            status: checks.status.clone(),
            finding_count: checks.finding_count,
            proposal_ref_count: checks.proposal_refs.len(),
        },
        data: GuiDataSupervision {
            output_job_count: production.output_job_count,
            artifact_count: production.artifact_count,
            artifact_run_count: production.artifact_run_count,
            proposal_count: production.proposal_count,
            manufacturing_plan_count: production.manufacturing_plan_count,
            panel_projection_count: production.panel_projection_count,
            latest_status: production.latest_status.clone(),
        },
    };
    if request.board_file.is_none() {
        let model = ProjectResolver::new(&request.project_root).resolve()?;
        snapshot.project_uuid = model.project.project_id.to_string();
        snapshot.project_name = model.project.name;
        snapshot.model_revision = model.model_revision.0;
        snapshot.journal.applied_transaction_count = model.journal_cursor.applied_transaction_count;
        snapshot.journal.accepted_transaction_tip = model
            .journal_cursor
            .applied_transaction_count
            .checked_sub(1)
            .and_then(|index| model.journal.get(index))
            .map(|transaction| transaction.transaction_id.to_string());
    }
    Ok(snapshot)
}

fn load_check_run_review_state(
    cli: &[String],
    request: &LiveReviewRequest,
) -> Result<CheckRunReviewState> {
    if request.board_file.is_some() {
        return Ok(CheckRunReviewState::default());
    }
    let project_root = request.project_root.display().to_string();
    if let Ok(context) = run_cli_json::<Value>(
        cli,
        &["context", "refresh", "--project-root", &project_root],
    )
        && let Some(state) = check_runs::check_run_review_state_from_context_value(&context) {
            return Ok(state);
        }
    run_cli_json(cli, &["check", "run", &project_root])
        .or_else(|_| Ok(CheckRunReviewState::default()))
}

fn load_production_status(cli: &[String], request: &LiveReviewRequest) -> Result<ProductionStatus> {
    if request.board_file.is_some() {
        return Ok(ProductionStatus::default());
    }
    let project_root = request.project_root.display().to_string();
    let output_jobs: OutputJobsPayload =
        match run_cli_json(cli, &["project", "query", &project_root, "output-jobs"]) {
            Ok(payload) => payload,
            Err(_) => return Ok(ProductionStatus::default()),
        };
    let manufacturing_plans = run_cli_json(
        cli,
        &["project", "query", &project_root, "manufacturing-plans"],
    )
    .unwrap_or_default();
    let panel_projections = run_cli_json(
        cli,
        &["project", "query", &project_root, "panel-projections"],
    )
    .unwrap_or_default();
    let artifact_list = run_cli_json(cli, &["artifact", "list", &project_root]).unwrap_or_default();
    let proposals = run_cli_json(cli, &["proposal", "list", &project_root]).unwrap_or_default();
    let mut status = production_payloads_to_production_status(
        output_jobs,
        artifact_list,
        proposals,
        manufacturing_plans,
        panel_projections,
    );
    attach_proposal_validation(cli, &project_root, &mut status);
    if let Some(artifact_id) = focused_artifact_id(&status) {
        let args = vec![
            "artifact".to_string(),
            "files".to_string(),
            project_root.clone(),
            "--artifact".to_string(),
            artifact_id,
        ];
        if let Ok(payload) = run_cli_json_owned::<ArtifactFilesPayload>(cli, &args) {
            let mut detail = artifact_files_payload_to_detail(payload);
            if let Some(file) = detail.focused_file.as_ref() {
                detail.focused_preview =
                    load_artifact_file_preview(cli, &project_root, &detail.artifact_id, &file.path)
                        .ok();
            }
            status.focused_artifact = Some(detail);
        }
    }
    Ok(status)
}

fn artifact_files_payload_to_detail(payload: ArtifactFilesPayload) -> ProductionArtifactDetail {
    let files = payload
        .files
        .into_iter()
        .map(|file| ProductionArtifactFileSummary {
            path: file.path.display().to_string(),
            sha256: file.sha256,
        })
        .collect::<Vec<_>>();
    let focused_file = files.first().cloned();
    ProductionArtifactDetail {
        artifact_id: payload.artifact_id,
        kind: payload.kind,
        output_dir: payload.output_dir,
        validation_state: payload.validation_state,
        file_count: files.len(),
        files,
        focused_file,
        focused_preview: None,
        production_projection_count: payload.production_projections.len(),
        production_projections: payload
            .production_projections
            .into_iter()
            .map(|projection| ProductionArtifactProjectionSummary {
                projection_kind: projection.projection_kind,
                projection_contract: projection.projection_contract,
                model_revision: projection.model_revision,
                byte_count: projection.byte_count,
                sha256: projection.sha256,
            })
            .collect(),
    }
}

fn load_artifact_file_preview(
    cli: &[String],
    project_root: &str,
    artifact_id: &str,
    file: &str,
) -> Result<ProductionArtifactFilePreviewSummary> {
    let args = vec![
        "artifact".to_string(),
        "preview".to_string(),
        project_root.to_string(),
        "--artifact".to_string(),
        artifact_id.to_string(),
        "--file".to_string(),
        file.to_string(),
    ];
    let payload = run_cli_json_owned::<ArtifactFilePreviewPayload>(cli, &args)?;
    Ok(artifact_preview_payload_to_summary(payload))
}

fn artifact_preview_payload_to_summary(
    payload: ArtifactFilePreviewPayload,
) -> ProductionArtifactFilePreviewSummary {
    ProductionArtifactFilePreviewSummary {
        file: payload.file.display().to_string(),
        preview_kind: payload.preview_kind,
        hash_matches_metadata: payload.hash_matches_metadata,
        primitive_count: payload.primitive_count,
        primitives: payload
            .primitives
            .into_iter()
            .map(|primitive| ProductionArtifactPreviewPrimitive {
                kind: primitive.kind,
                aperture_diameter_nm: primitive.aperture_diameter_nm,
                aperture_width_nm: primitive.aperture_width_nm,
                aperture_height_nm: primitive.aperture_height_nm,
                tool: primitive.tool,
                diameter_mm: primitive.diameter_mm,
                points: primitive
                    .points
                    .into_iter()
                    .map(|point| ProductionArtifactPreviewPoint {
                        x_nm: point.x_nm,
                        y_nm: point.y_nm,
                    })
                    .collect(),
            })
            .collect(),
        geometry_count: payload
            .inspection
            .get("geometry_count")
            .and_then(serde_json::Value::as_u64)
            .map(|value| value as usize),
        hit_count: payload
            .inspection
            .get("hit_count")
            .and_then(serde_json::Value::as_u64)
            .map(|value| value as usize),
        row_count: payload
            .inspection
            .get("row_count")
            .and_then(serde_json::Value::as_u64)
            .map(|value| value as usize),
        csv_columns: payload
            .inspection
            .get("columns")
            .and_then(serde_json::Value::as_array)
            .map(|values| string_array_values(values))
            .unwrap_or_default(),
        csv_rows: payload
            .inspection
            .get("rows")
            .and_then(serde_json::Value::as_array)
            .map(|values| csv_row_values(values))
            .unwrap_or_default(),
    }
}

fn string_array_values(values: &[serde_json::Value]) -> Vec<String> {
    values
        .iter()
        .filter_map(serde_json::Value::as_str)
        .map(ToString::to_string)
        .collect()
}

fn csv_row_values(values: &[serde_json::Value]) -> Vec<Vec<String>> {
    values
        .iter()
        .filter_map(serde_json::Value::as_array)
        .map(|row| string_array_values(row))
        .collect()
}

fn production_payloads_to_production_status(
    output_jobs_payload: OutputJobsPayload,
    artifact_list_payload: ArtifactListPayload,
    proposals_payload: ProposalsPayload,
    manufacturing_plans_payload: ManufacturingPlansPayload,
    panel_projections_payload: PanelProjectionsPayload,
) -> ProductionStatus {
    let latest = output_jobs_payload
        .output_jobs
        .iter()
        .filter_map(|job| {
            job.latest_run
                .as_ref()
                .map(|run| (&job.status, run.run_sequence, run.run_id.clone()))
        })
        .max_by(|(_, a_sequence, a_id), (_, b_sequence, b_id)| {
            a_sequence.cmp(b_sequence).then_with(|| a_id.cmp(b_id))
        });
    let (latest_status, latest_run_id) = latest
        .map(|(status, _, run_id)| (Some(status.clone()), Some(run_id)))
        .unwrap_or((None, None));
    let output_jobs = output_jobs_payload
        .output_jobs
        .iter()
        .map(|job| ProductionOutputJobSummary {
            id: job.id.clone(),
            name: job.name.clone(),
            include: job.include.clone(),
            prefix: job.prefix.clone(),
            output_dir: job
                .output_dir
                .as_deref()
                .map(|path| path.display().to_string()),
            family: job
                .include
                .first()
                .map(|value| value.replace(['_', '-'], " ").to_uppercase())
                .unwrap_or_else(|| "UNSCOPED".to_string()),
            status: job.status.clone(),
            execution_count: job.execution_count,
            artifact_count: job.artifacts.len(),
            latest_run_id: job.latest_run.as_ref().map(|run| run.run_id.clone()),
            latest_run_artifact_id: job
                .latest_run
                .as_ref()
                .and_then(|run| run.artifact_id.clone()),
            artifacts: job
                .artifacts
                .iter()
                .map(|artifact| ProductionArtifactSummary {
                    artifact_id: artifact.artifact_id.clone(),
                    kind: artifact.kind.clone(),
                    project_id: artifact.project_id.clone(),
                    model_revision: artifact.model_revision.clone(),
                    output_job: artifact.output_job.clone(),
                    variant: artifact.variant.clone(),
                    generator_version: artifact.generator_version.clone(),
                    output_dir: artifact.output_dir.clone(),
                    validation_state: artifact.validation_state.clone(),
                    file_count: artifact.files.len(),
                    files: artifact
                        .files
                        .iter()
                        .map(|file| ProductionArtifactFileSummary {
                            path: file.path.display().to_string(),
                            sha256: file.sha256.clone(),
                        })
                        .collect(),
                    production_projection_count: artifact.production_projections.len(),
                    production_projections: artifact
                        .production_projections
                        .iter()
                        .map(|projection| ProductionArtifactProjectionSummary {
                            projection_kind: projection.projection_kind.clone(),
                            projection_contract: projection.projection_contract.clone(),
                            model_revision: projection.model_revision.clone(),
                            byte_count: projection.byte_count,
                            sha256: projection.sha256.clone(),
                        })
                        .collect(),
                })
                .collect(),
        })
        .collect::<Vec<_>>();
    let artifact_runs = artifact_run_summaries(&artifact_list_payload);
    let proposals = proposal_summaries(&proposals_payload);
    let manufacturing_plans = manufacturing_plans_payload
        .manufacturing_plans
        .iter()
        .map(|plan| ProductionManufacturingPlanSummary {
            id: plan.id.clone(),
            name: plan.name.clone(),
            prefix: plan.prefix.clone(),
            board_or_panel: plan.board_or_panel.clone(),
            variant: plan.variant.clone(),
            object_revision: plan.object_revision,
        })
        .collect::<Vec<_>>();
    let panel_projections = panel_projections_payload
        .panel_projections
        .iter()
        .map(|panel| {
            let first = panel.board_instances.first();
            ProductionPanelProjectionSummary {
                id: panel.id.clone(),
                name: panel.name.clone(),
                board_instance_count: panel.board_instances.len(),
                first_board: first.map(|instance| instance.board.clone()),
                first_x_nm: first.map(|instance| instance.x_nm),
                first_y_nm: first.map(|instance| instance.y_nm),
                first_rotation_deg: first.map(|instance| instance.rotation_deg),
                object_revision: panel.object_revision,
            }
        })
        .collect::<Vec<_>>();
    ProductionStatus {
        output_job_count: output_jobs_payload.output_job_count,
        artifact_count: artifact_list_payload.artifact_count.max(
            output_jobs_payload
                .output_jobs
                .iter()
                .map(|job| job.artifacts.len())
                .sum(),
        ),
        artifact_run_count: artifact_runs.len(),
        proposal_count: proposals_payload.proposal_count,
        manufacturing_plan_count: manufacturing_plans_payload.manufacturing_plan_count,
        panel_projection_count: panel_projections_payload.panel_projection_count,
        latest_status,
        latest_run_id,
        latest_artifact_id: artifact_list_payload.latest_artifact_id,
        latest_artifact_run_id: artifact_list_payload.latest_artifact_run_id,
        latest_output_job_run_id: artifact_list_payload.latest_output_job_run_id,
        output_jobs,
        artifact_runs,
        proposals,
        manufacturing_plans,
        panel_projections,
        focused_artifact: None,
    }
}

fn net_display_from_native_board_value(board: &Value) -> Result<Vec<NetDisplayEntry>> {
    let nets_map = board
        .get("nets")
        .and_then(|value| value.as_object())
        .cloned()
        .unwrap_or_default();
    let mut nets = Vec::with_capacity(nets_map.len());
    for (_key, value) in nets_map {
        let net: EngineNetPayload =
            serde_json::from_value(value).context("failed to parse native board net")?;
        nets.push(NetDisplayEntry {
            net_uuid: net.uuid.to_string(),
            net_name: net.name,
            airwire_color_rgb: deterministic_airwire_color(net.uuid.as_bytes()),
        });
    }
    nets.sort_by(|a, b| {
        a.net_name
            .cmp(&b.net_name)
            .then_with(|| a.net_uuid.cmp(&b.net_uuid))
    });
    Ok(nets)
}

#[derive(Debug, Clone, Deserialize)]
struct EngineNetPayload {
    uuid: uuid::Uuid,
    name: String,
}

fn deterministic_airwire_color(bytes: &[u8]) -> [f32; 3] {
    let mut hash: u64 = 0xcbf29ce484222325;
    for b in bytes {
        hash ^= *b as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    let hue = (hash % 360) as f32 / 360.0;
    let sat = 0.42 + (((hash >> 8) & 0xff) as f32 / 255.0) * 0.18;
    let val = 0.84 + (((hash >> 16) & 0xff) as f32 / 255.0) * 0.10;
    hsv_to_rgb(hue, sat.clamp(0.38, 0.62), val.clamp(0.84, 0.94))
}

fn hsv_to_rgb(h: f32, s: f32, v: f32) -> [f32; 3] {
    let h6 = (h.rem_euclid(1.0) * 6.0).clamp(0.0, 6.0);
    let i = h6.floor() as i32;
    let f = h6 - i as f32;
    let p = v * (1.0 - s);
    let q = v * (1.0 - s * f);
    let t = v * (1.0 - s * (1.0 - f));
    match i.rem_euclid(6) {
        0 => [v, t, p],
        1 => [q, v, p],
        2 => [p, v, t],
        3 => [p, q, v],
        4 => [t, p, v],
        _ => [v, p, q],
    }
}

/// Load the board scene from the engine-resolved native project model,
/// bypassing CLI subprocess invocations. Returns the built scene and the
/// board file path that backs the native board shard.
fn load_scene_from_engine(request: &LiveReviewRequest) -> Result<(BoardReviewSceneV1, PathBuf)> {
    let root = &request.project_root;
    let model = ProjectResolver::new(root).resolve()?;
    let board_shard = model
        .source_shards
        .iter()
        .find(|shard| shard.kind == SourceShardKind::BoardRoot)
        .context("resolved native project model is missing board root shard")?;
    let board_path = board_shard.path.clone();
    let board_value = model
        .materialized_source_shard_value(SourceShardKind::BoardRoot)
        .context("failed to materialize resolved board root shard")?;

    let board_uuid = board_value
        .get("uuid")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let inspect = ProjectInspectPayload {
        project_root: root.display().to_string(),
        project_name: model.project.name.clone(),
        project_uuid: model.project.project_id.to_string(),
        board_uuid: board_uuid.clone(),
        board_path: board_path.display().to_string(),
    };

    // --- Outline ---
    let outline = extract_outline(&board_value)?;

    // --- Components (packages) ---
    let components = extract_components(&board_value)?;

    // --- Pads ---
    let pads = extract_pads(&board_value)?;
    let pad_expansion_setup = extract_pad_expansion_setup(&board_value)?;

    // --- Tracks ---
    let tracks = extract_tracks(&board_value)?;

    // --- Vias ---
    let vias = extract_vias(&board_value)?;

    // --- Zones ---
    let zones = extract_zones(&board_value)?;
    let (native_board_texts, native_board_text_geometries, glyph_mesh_assets) =
        extract_native_board_texts(&board_value)?;

    // --- Component graphics (silkscreen + mechanical) ---
    let (component_graphics, component_texts) =
        extract_component_graphics(&board_value, &components)?;

    // Native-project path: resolve Edge.Cuts from the persisted stackup JSON
    // when present, else fall back to the KiCad 7 canonical id 44. A better
    // resolution (via typed engine Board access) would be a follow-on.
    let edge_cuts_layer_key = board_value
        .get("stackup")
        .and_then(|s| s.get("layers"))
        .and_then(|arr| arr.as_array())
        .and_then(|layers| {
            layers.iter().find_map(|l| {
                let name = l.get("name").and_then(|n| n.as_str())?;
                if name != "Edge.Cuts" {
                    return None;
                }
                let id = l.get("id").and_then(|v| v.as_i64())? as i32;
                Some(layer_id(id))
            })
        })
        .unwrap_or_else(|| layer_id(44));

    // Native-project path does not currently persist the original
    // per-contributor Edge.Cuts identities from import. Derive board-scoped
    // authored primitives from the persisted assembled outline so native
    // projects participate in the same Edge.Cuts authored-layer lane as
    // imported boards, while keeping scene.outline as the board-boundary view.
    let board_graphics =
        outline_board_graphics_from_outline(&outline, &inspect.board_uuid, &edge_cuts_layer_key);
    let unrouted_primitives: Vec<UnroutedPrimitive> = Vec::new();
    let net_display = net_display_from_native_board_value(&board_value)?;

    let mut scene = build_board_review_scene(
        &inspect,
        outline,
        components,
        component_graphics,
        component_texts,
        pad_expansion_setup,
        pads,
        tracks,
        vias,
        zones,
        board_graphics,
        native_board_texts,
        native_board_text_geometries,
        glyph_mesh_assets,
        unrouted_primitives,
        net_display,
        edge_cuts_layer_key,
    );
    if let Some(layers) = scene_layers_from_native_stackup_value(&board_value) {
        scene.layers = layers;
    }
    scene.source_revision = model.model_revision.0.clone();
    Ok((scene, board_path))
}

fn extract_outline(board: &Value) -> Result<OutlinePayload> {
    let outline_obj = board
        .get("outline")
        .ok_or_else(|| anyhow::anyhow!("board JSON missing 'outline' field"))?;
    let vertices: Vec<PointNm> = serde_json::from_value(
        outline_obj
            .get("vertices")
            .cloned()
            .unwrap_or(Value::Array(vec![])),
    )
    .context("failed to parse outline vertices")?;
    let closed = outline_obj
        .get("closed")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);
    Ok(OutlinePayload { vertices, closed })
}

fn extract_components(board: &Value) -> Result<Vec<BoardComponentPayload>> {
    let packages = board
        .get("packages")
        .and_then(|v| v.as_object())
        .cloned()
        .unwrap_or_default();
    let mut components = Vec::with_capacity(packages.len());
    for (_key, value) in packages {
        let pkg: EnginePackagePayload =
            serde_json::from_value(value).context("failed to parse board package")?;
        components.push(BoardComponentPayload {
            uuid: pkg.uuid.to_string(),
            reference: pkg.reference,
            value: pkg.value,
            position: PointNm {
                x: pkg.position.x,
                y: pkg.position.y,
            },
            rotation: pkg.rotation,
            layer: pkg.layer,
            locked: pkg.locked,
        });
    }
    components.sort_by(|a, b| {
        a.reference
            .cmp(&b.reference)
            .then_with(|| a.uuid.cmp(&b.uuid))
    });
    Ok(components)
}

/// Intermediate type matching the engine PlacedPackage JSON shape.
#[derive(Debug, Clone, Deserialize)]
struct EnginePackagePayload {
    uuid: uuid::Uuid,
    reference: String,
    value: String,
    position: EnginePointPayload,
    rotation: i32,
    layer: i32,
    locked: bool,
}

#[derive(Debug, Clone, Deserialize)]
struct EnginePointPayload {
    x: i64,
    y: i64,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct EnginePadExpansionSetupPayload {
    #[serde(default)]
    pad_to_mask_clearance_nm: i64,
    #[serde(default)]
    pad_to_paste_clearance_nm: i64,
    #[serde(default)]
    pad_to_paste_ratio_ppm: i32,
    #[serde(default)]
    solder_mask_min_width_nm: i64,
}

fn extract_pads(board: &Value) -> Result<Vec<BoardPadPayload>> {
    let pads_map = board
        .get("pads")
        .and_then(|v| v.as_object())
        .cloned()
        .unwrap_or_default();
    let mut pads = Vec::with_capacity(pads_map.len());
    for (_key, value) in pads_map {
        let pad: EnginePadPayload =
            serde_json::from_value(value).context("failed to parse board pad")?;
        pads.push(BoardPadPayload {
            uuid: pad.uuid.to_string(),
            package: pad.package.to_string(),
            name: pad.name,
            net: pad.net.map(|u| u.to_string()),
            position: PointNm {
                x: pad.position.x,
                y: pad.position.y,
            },
            layer: pad.layer,
            copper_layers: pad.copper_layers,
            shape: pad.shape.to_string(),
            diameter: pad.diameter,
            width: pad.width,
            height: pad.height,
            roundrect_rratio_ppm: pad.roundrect_rratio_ppm,
            mask_layers: pad.mask_layers,
            paste_layers: pad.paste_layers,
            solder_mask_margin_nm: pad.solder_mask_margin_nm,
            solder_paste_margin_nm: pad.solder_paste_margin_nm,
            solder_paste_margin_ratio_ppm: pad.solder_paste_margin_ratio_ppm,
            drill: Some(pad.drill),
            rotation: pad.rotation.unwrap_or(0),
        });
    }
    pads.sort_by(|a, b| {
        a.package
            .cmp(&b.package)
            .then_with(|| a.name.cmp(&b.name))
            .then_with(|| a.uuid.cmp(&b.uuid))
    });
    Ok(pads)
}

fn extract_pad_expansion_setup(board: &Value) -> Result<ScenePadExpansionSetup> {
    let value = board
        .get("pad_expansion_setup")
        .cloned()
        .unwrap_or(Value::Null);
    let setup: EnginePadExpansionSetupPayload = if value.is_null() {
        EnginePadExpansionSetupPayload::default()
    } else {
        serde_json::from_value(value).context("failed to parse board pad expansion setup")?
    };
    Ok(ScenePadExpansionSetup {
        pad_to_mask_clearance_nm: setup.pad_to_mask_clearance_nm,
        pad_to_paste_clearance_nm: setup.pad_to_paste_clearance_nm,
        pad_to_paste_ratio_ppm: setup.pad_to_paste_ratio_ppm,
        solder_mask_min_width_nm: setup.solder_mask_min_width_nm,
    })
}

#[derive(Debug, Clone, Deserialize)]
struct EnginePadPayload {
    uuid: uuid::Uuid,
    package: uuid::Uuid,
    name: String,
    net: Option<uuid::Uuid>,
    position: EnginePointPayload,
    layer: i32,
    #[serde(default)]
    shape: EnginePadShape,
    #[serde(default)]
    diameter: i64,
    #[serde(default)]
    width: i64,
    #[serde(default)]
    height: i64,
    #[serde(default = "default_roundrect_rratio_ppm")]
    roundrect_rratio_ppm: u32,
    #[serde(default)]
    copper_layers: Vec<i32>,
    #[serde(default)]
    mask_layers: Vec<i32>,
    #[serde(default)]
    paste_layers: Vec<i32>,
    #[serde(default)]
    solder_mask_margin_nm: i64,
    #[serde(default)]
    solder_paste_margin_nm: i64,
    #[serde(default)]
    solder_paste_margin_ratio_ppm: i32,
    #[serde(default)]
    drill: i64,
    #[serde(default)]
    rotation: Option<i32>,
}

#[derive(Debug, Clone, Copy, Default, Deserialize)]
#[serde(rename_all = "snake_case")]
enum EnginePadShape {
    #[default]
    Circle,
    Rect,
    Oval,
    RoundRect,
}

impl std::fmt::Display for EnginePadShape {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EnginePadShape::Circle => write!(f, "circle"),
            EnginePadShape::Rect => write!(f, "rect"),
            EnginePadShape::Oval => write!(f, "oval"),
            EnginePadShape::RoundRect => write!(f, "roundrect"),
        }
    }
}

fn extract_tracks(board: &Value) -> Result<Vec<BoardTrackPayload>> {
    let tracks_map = board
        .get("tracks")
        .and_then(|v| v.as_object())
        .cloned()
        .unwrap_or_default();
    let mut tracks = Vec::with_capacity(tracks_map.len());
    for (_key, value) in tracks_map {
        let track: EngineTrackPayload =
            serde_json::from_value(value).context("failed to parse board track")?;
        tracks.push(BoardTrackPayload {
            uuid: track.uuid.to_string(),
            net: track.net.to_string(),
            from: PointNm {
                x: track.from.x,
                y: track.from.y,
            },
            to: PointNm {
                x: track.to.x,
                y: track.to.y,
            },
            width: track.width,
            layer: track.layer,
        });
    }
    tracks.sort_by(|a, b| a.uuid.cmp(&b.uuid));
    Ok(tracks)
}

#[derive(Debug, Clone, Deserialize)]
struct EngineTrackPayload {
    uuid: uuid::Uuid,
    net: uuid::Uuid,
    from: EnginePointPayload,
    to: EnginePointPayload,
    width: i64,
    layer: i32,
}

fn extract_vias(board: &Value) -> Result<Vec<BoardViaPayload>> {
    let vias_map = board
        .get("vias")
        .and_then(|v| v.as_object())
        .cloned()
        .unwrap_or_default();
    let mut vias = Vec::with_capacity(vias_map.len());
    for (_key, value) in vias_map {
        let via: EngineViaPayload =
            serde_json::from_value(value).context("failed to parse board via")?;
        vias.push(BoardViaPayload {
            uuid: via.uuid.to_string(),
            net: via.net.to_string(),
            position: PointNm {
                x: via.position.x,
                y: via.position.y,
            },
            drill: via.drill,
            diameter: via.diameter,
            from_layer: via.from_layer,
            to_layer: via.to_layer,
        });
    }
    vias.sort_by(|a, b| a.uuid.cmp(&b.uuid));
    Ok(vias)
}

#[derive(Debug, Clone, Deserialize)]
struct EngineViaPayload {
    uuid: uuid::Uuid,
    net: uuid::Uuid,
    position: EnginePointPayload,
    drill: i64,
    diameter: i64,
    from_layer: i32,
    to_layer: i32,
}

fn extract_zones(board: &Value) -> Result<Vec<BoardZonePayload>> {
    let zones_map = board
        .get("zones")
        .and_then(|v| v.as_object())
        .cloned()
        .unwrap_or_default();
    let mut zones = Vec::with_capacity(zones_map.len());
    for (_key, value) in zones_map {
        let zone: EngineZonePayload =
            serde_json::from_value(value).context("failed to parse board zone")?;
        let vertices: Vec<PointNm> = zone
            .polygon
            .vertices
            .into_iter()
            .map(|p| PointNm { x: p.x, y: p.y })
            .collect();
        zones.push(BoardZonePayload {
            uuid: zone.uuid.to_string(),
            net: zone.net.to_string(),
            polygon: OutlinePayload {
                vertices,
                closed: zone.polygon.closed,
            },
            layer: zone.layer,
        });
    }
    zones.sort_by(|a, b| a.uuid.cmp(&b.uuid));
    Ok(zones)
}

#[derive(Debug, Clone, Deserialize)]
struct EngineZonePayload {
    uuid: uuid::Uuid,
    net: uuid::Uuid,
    polygon: EnginePolygonPayload,
    layer: i32,
}

#[derive(Debug, Clone, Deserialize)]
struct EnginePolygonPayload {
    vertices: Vec<EnginePointPayload>,
    #[serde(default = "default_true")]
    closed: bool,
}

#[derive(Debug, Clone, Deserialize)]
struct EngineBoardTextPayload {
    uuid: uuid::Uuid,
    text: String,
    position: EnginePointPayload,
    rotation: i32,
    layer: i32,
    #[serde(default)]
    render_intent: TextRenderIntent,
    #[serde(default)]
    family: TextFamilyId,
    #[serde(default)]
    family_source: eda_engine::text::TextFamilySource,
    #[serde(default)]
    style: TextStyleId,
    #[serde(default = "default_board_text_height_nm")]
    height_nm: i64,
    #[serde(default)]
    stroke_width_nm: i64,
    #[serde(default = "default_board_text_h_align")]
    h_align: TextHAlign,
    #[serde(default = "default_board_text_v_align")]
    v_align: TextVAlign,
    #[serde(default)]
    mirrored: bool,
    #[serde(default)]
    keep_upright: bool,
    #[serde(default = "default_board_text_line_spacing_ratio_ppm")]
    line_spacing_ratio_ppm: i32,
    #[serde(default)]
    italic: bool,
    #[serde(default)]
    bold: bool,
    #[serde(default)]
    style_class: Option<String>,
}

fn default_true() -> bool {
    true
}

fn default_board_text_height_nm() -> i64 {
    1_000_000
}

fn default_board_text_h_align() -> TextHAlign {
    TextHAlign::Left
}

fn default_board_text_v_align() -> TextVAlign {
    TextVAlign::Bottom
}

fn default_board_text_line_spacing_ratio_ppm() -> i32 {
    1_000_000
}

fn extract_native_board_texts(
    board: &Value,
) -> Result<(
    Vec<BoardTextPrimitive>,
    Vec<BoardTextGeometryPrimitive>,
    Vec<GlyphMeshAssetPrimitive>,
)> {
    let texts = board
        .get("texts")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    let mut primitives = Vec::new();
    let mut geometries = Vec::new();
    let mut mesh_assets_by_handle = BTreeMap::new();
    for value in texts {
        let text: EngineBoardTextPayload =
            serde_json::from_value(value).context("failed to parse native board text")?;
        let board_text = BoardText {
            uuid: text.uuid,
            text: text.text,
            position: eda_engine::ir::geometry::Point {
                x: text.position.x,
                y: text.position.y,
            },
            rotation: text.rotation,
            layer: text.layer,
            render_intent: text.render_intent,
            family: text.family,
            family_source: text.family_source,
            style: text.style,
            height_nm: text.height_nm,
            stroke_width_nm: if text.stroke_width_nm > 0 {
                text.stroke_width_nm
            } else {
                default_stroke_width_nm(text.height_nm)
            },
            h_align: text.h_align,
            v_align: text.v_align,
            mirrored: text.mirrored,
            keep_upright: text.keep_upright,
            line_spacing_ratio_ppm: text.line_spacing_ratio_ppm,
            italic: text.italic,
            bold: text.bold,
            style_class: text.style_class,
        };
        push_board_text_scene_primitives(
            &board_text,
            &mut primitives,
            &mut geometries,
            &mut mesh_assets_by_handle,
        );
    }
    Ok((
        primitives,
        geometries,
        mesh_assets_by_handle.into_values().collect(),
    ))
}

fn extract_component_graphics(
    board: &Value,
    components: &[BoardComponentPayload],
) -> Result<(Vec<ComponentGraphicPrimitive>, Vec<ComponentTextPrimitive>)> {
    let mut all_graphics = Vec::new();
    let mut all_texts = Vec::new();

    for component in components {
        let comp_uuid = &component.uuid;

        // --- Silkscreen ---
        let silk_payload = extract_silkscreen_payload(board, comp_uuid, "component_silkscreen")?;
        let (g, t) = component_silkscreen_primitives(component, silk_payload);
        all_graphics.extend(g);
        all_texts.extend(t);

        // --- Mechanical ---
        let mech_payload = extract_mechanical_payload(board, comp_uuid, "component_mechanical")?;
        let (g, t) = component_mechanical_primitives(component, mech_payload);
        all_graphics.extend(g);
        all_texts.extend(t);
    }

    Ok((all_graphics, all_texts))
}

/// Helper: read per-component graphic arrays from the native board JSON.
// Established multi-value signature; a tuple type alias would not improve clarity.
#[allow(clippy::type_complexity)]
fn read_graphic_arrays(
    board: &Value,
    component_uuid: &str,
    prefix: &str,
) -> (
    Vec<ComponentGraphicLinePayload>,
    Vec<ComponentGraphicTextPayload>,
    Vec<ComponentGraphicArcPayload>,
    Vec<ComponentGraphicCirclePayload>,
    Vec<ComponentGraphicPolygonPayload>,
    Vec<ComponentGraphicPolylinePayload>,
) {
    let lines_key = prefix;
    let texts_key = format!("{}_texts", prefix);
    let arcs_key = format!("{}_arcs", prefix);
    let circles_key = format!("{}_circles", prefix);
    let polygons_key = format!("{}_polygons", prefix);
    let polylines_key = format!("{}_polylines", prefix);

    let get_vec = |map_key: &str| -> Vec<Value> {
        board
            .get(map_key)
            .and_then(|m| m.get(component_uuid))
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default()
    };

    let lines: Vec<ComponentGraphicLinePayload> =
        serde_json::from_value(Value::Array(get_vec(lines_key))).unwrap_or_default();
    let texts: Vec<ComponentGraphicTextPayload> =
        serde_json::from_value(Value::Array(get_vec(&texts_key))).unwrap_or_default();
    let arcs: Vec<ComponentGraphicArcPayload> =
        serde_json::from_value(Value::Array(get_vec(&arcs_key))).unwrap_or_default();
    let circles: Vec<ComponentGraphicCirclePayload> =
        serde_json::from_value(Value::Array(get_vec(&circles_key))).unwrap_or_default();
    let polygons: Vec<ComponentGraphicPolygonPayload> =
        serde_json::from_value(Value::Array(get_vec(&polygons_key))).unwrap_or_default();
    let polylines: Vec<ComponentGraphicPolylinePayload> =
        serde_json::from_value(Value::Array(get_vec(&polylines_key))).unwrap_or_default();

    (lines, texts, arcs, circles, polygons, polylines)
}

fn extract_silkscreen_payload(
    board: &Value,
    component_uuid: &str,
    prefix: &str,
) -> Result<ComponentSilkscreenPayload> {
    let (lines, texts, arcs, circles, polygons, polylines) =
        read_graphic_arrays(board, component_uuid, prefix);
    Ok(ComponentSilkscreenPayload {
        component_uuid: component_uuid.to_string(),
        lines,
        arcs,
        circles,
        polygons,
        polylines,
        texts,
    })
}

fn extract_mechanical_payload(
    board: &Value,
    component_uuid: &str,
    prefix: &str,
) -> Result<ComponentMechanicalPayload> {
    let (lines, texts, arcs, circles, polygons, polylines) =
        read_graphic_arrays(board, component_uuid, prefix);
    Ok(ComponentMechanicalPayload {
        component_uuid: component_uuid.to_string(),
        lines,
        arcs,
        circles,
        polygons,
        polylines,
        texts,
    })
}

fn empty_route_review_payload(request: &LiveReviewRequest) -> RouteProposalReviewPayload {
    RouteProposalReviewPayload {
        action: "review_route_proposal".to_string(),
        review_source: "live".to_string(),
        status: "no_selectable_route".to_string(),
        explanation: "no selectable route proposal is currently active; board editor view only"
            .to_string(),
        project_root: Some(request.project_root.display().to_string()),
        artifact_path: request
            .artifact_path
            .as_ref()
            .map(|path| path.display().to_string()),
        kind: None,
        source_version: None,
        version: None,
        project_uuid: None,
        project_name: None,
        net_uuid: request.net_uuid.clone(),
        from_anchor_pad_uuid: request.from_anchor_pad_uuid.clone(),
        to_anchor_pad_uuid: request.to_anchor_pad_uuid.clone(),
        selection_profile: request.profile.clone(),
        selection_rule: None,
        selected_candidate: None,
        selected_policy: None,
        contract: "board_editor_no_active_route_proposal_v1".to_string(),
        actions: 0,
        draw_track_actions: 0,
        selected_path_bend_count: 0,
        selected_path_point_count: 0,
        selected_path_segment_count: 0,
        segment_evidence: Vec::new(),
        proposal_actions: Vec::new(),
    }
}

fn load_selected_candidate_path(
    cli: &[String],
    request: &LiveReviewRequest,
    selected_candidate: Option<&str>,
) -> Result<Option<Vec<PointNm>>> {
    if request.artifact_path.is_some() {
        return Ok(None);
    }
    let Some(candidate) = selected_candidate else {
        return Ok(None);
    };
    let Some(net_uuid) = request.net_uuid.as_ref() else {
        return Ok(None);
    };
    let Some(from_anchor_pad_uuid) = request.from_anchor_pad_uuid.as_ref() else {
        return Ok(None);
    };
    let Some(to_anchor_pad_uuid) = request.to_anchor_pad_uuid.as_ref() else {
        return Ok(None);
    };
    let project_root = request.project_root.to_string_lossy().into_owned();
    let args = vec![
        "project".to_string(),
        "query".to_string(),
        project_root,
        "route-path-candidate-explain".to_string(),
        "--net".to_string(),
        net_uuid.clone(),
        "--from-anchor".to_string(),
        from_anchor_pad_uuid.clone(),
        "--to-anchor".to_string(),
        to_anchor_pad_uuid.clone(),
        "--candidate".to_string(),
        candidate.to_string(),
    ];
    let refs: Vec<&str> = args.iter().map(String::as_str).collect();
    let explain: CandidateExplainPayload = run_cli_json(cli, &refs)?;
    if let Some(path) = explain.selected_path
        && path.points.len() >= 2 {
            return Ok(Some(path.points));
        }
    if let Some(span) = explain.selected_span {
        return Ok(Some(vec![span.from, span.to]));
    }
    Ok(None)
}

#[allow(dead_code)]
fn load_component_graphics(
    cli: &[String],
    project_root: &str,
    components: &[BoardComponentPayload],
) -> Result<(Vec<ComponentGraphicPrimitive>, Vec<ComponentTextPrimitive>)> {
    use std::thread;
    let handles: Vec<_> = components
        .iter()
        .map(|component| {
            let cli = cli.to_vec();
            let root = project_root.to_string();
            let comp = component.clone();
            thread::spawn(
                move || -> Result<(Vec<ComponentGraphicPrimitive>, Vec<ComponentTextPrimitive>)> {
                    let mut graphics = Vec::new();
                    let mut texts = Vec::new();
                    let refs_silk: Vec<&str> = vec![
                        "project",
                        "query",
                        &root,
                        "board-component-silkscreen",
                        "--component",
                        &comp.uuid,
                    ];
                    if let Ok(silkscreen) =
                        run_cli_json::<ComponentSilkscreenPayload>(&cli, &refs_silk)
                    {
                        let (g, t) = component_silkscreen_primitives(&comp, silkscreen);
                        graphics.extend(g);
                        texts.extend(t);
                    }
                    let refs_mech: Vec<&str> = vec![
                        "project",
                        "query",
                        &root,
                        "board-component-mechanical",
                        "--component",
                        &comp.uuid,
                    ];
                    if let Ok(mechanical) =
                        run_cli_json::<ComponentMechanicalPayload>(&cli, &refs_mech)
                    {
                        let (g, t) = component_mechanical_primitives(&comp, mechanical);
                        graphics.extend(g);
                        texts.extend(t);
                    }
                    Ok((graphics, texts))
                },
            )
        })
        .collect();
    let mut all_graphics = Vec::new();
    let mut all_texts = Vec::new();
    for handle in handles {
        if let Ok(Ok((g, t))) = handle.join() {
            all_graphics.extend(g);
            all_texts.extend(t);
        }
    }
    Ok((all_graphics, all_texts))
}

fn load_live_route_review(
    cli: &[String],
    request: &LiveReviewRequest,
) -> Result<RouteProposalReviewPayload> {
    if let Some(artifact_path) = &request.artifact_path {
        return run_cli_json_owned(
            cli,
            &[
                "project".to_string(),
                "review-route-proposal".to_string(),
                "--artifact".to_string(),
                artifact_path.display().to_string(),
            ],
        );
    }
    let project_root = request.project_root.to_string_lossy().into_owned();
    let net_uuid = request
        .net_uuid
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("live route review requires a net UUID"))?;
    let from_anchor_pad_uuid = request
        .from_anchor_pad_uuid
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("live route review requires a from-anchor pad UUID"))?;
    let to_anchor_pad_uuid = request
        .to_anchor_pad_uuid
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("live route review requires a to-anchor pad UUID"))?;
    let mut args = vec![
        "project".to_string(),
        "review-route-proposal".to_string(),
        project_root,
        "--net".to_string(),
        net_uuid.clone(),
        "--from-anchor".to_string(),
        from_anchor_pad_uuid.clone(),
        "--to-anchor".to_string(),
        to_anchor_pad_uuid.clone(),
    ];
    if let Some(profile) = &request.profile {
        args.push("--profile".to_string());
        args.push(profile.clone());
    }
    let refs: Vec<&str> = args.iter().map(String::as_str).collect();
    run_cli_json(cli, &refs)
}

// Scene import threads many primitive-geometry parameters.
#[allow(clippy::too_many_arguments)]
fn build_board_review_scene(
    inspect: &ProjectInspectPayload,
    outline: OutlinePayload,
    components: Vec<BoardComponentPayload>,
    component_graphics: Vec<ComponentGraphicPrimitive>,
    component_texts: Vec<ComponentTextPrimitive>,
    pad_expansion_setup: ScenePadExpansionSetup,
    pads: Vec<BoardPadPayload>,
    tracks: Vec<BoardTrackPayload>,
    vias: Vec<BoardViaPayload>,
    zones: Vec<BoardZonePayload>,
    board_graphics: Vec<BoardGraphicPrimitive>,
    board_texts: Vec<BoardTextPrimitive>,
    board_text_geometries: Vec<BoardTextGeometryPrimitive>,
    glyph_mesh_assets: Vec<GlyphMeshAssetPrimitive>,
    unrouted_primitives: Vec<UnroutedPrimitive>,
    net_display: Vec<NetDisplayEntry>,
    outline_layer_key: String,
) -> BoardReviewSceneV1 {
    let layer_ids = collect_layer_ids(
        &components,
        &component_graphics,
        &pads,
        &tracks,
        &vias,
        &zones,
        &board_graphics,
        &board_text_geometries,
    );
    let pads_by_component = pads.iter().fold(
        BTreeMap::<String, Vec<&BoardPadPayload>>::new(),
        |mut acc, pad| {
            acc.entry(pad.package.clone()).or_default().push(pad);
            acc
        },
    );
    let graphics_by_component = component_graphics.iter().fold(
        BTreeMap::<String, Vec<&ComponentGraphicPrimitive>>::new(),
        |mut acc, graphic| {
            acc.entry(graphic.component_uuid.clone())
                .or_default()
                .push(graphic);
            acc
        },
    );
    let texts_by_component = component_texts.iter().fold(
        BTreeMap::<String, Vec<&ComponentTextPrimitive>>::new(),
        |mut acc, text| {
            acc.entry(text.component_uuid.clone())
                .or_default()
                .push(text);
            acc
        },
    );
    let components: Vec<ComponentBounds> = components
        .into_iter()
        .map(|component| {
            let bounds = component_bounds(
                &component,
                pads_by_component
                    .get(&component.uuid)
                    .map(Vec::as_slice)
                    .unwrap_or(&[]),
                graphics_by_component
                    .get(&component.uuid)
                    .map(Vec::as_slice)
                    .unwrap_or(&[]),
                texts_by_component
                    .get(&component.uuid)
                    .map(Vec::as_slice)
                    .unwrap_or(&[]),
            );
            ComponentBounds {
                object_id: format!("component:{}", component.uuid),
                object_kind: "component".to_string(),
                source_object_uuid: component.uuid.clone(),
                component_uuid: component.uuid.clone(),
                reference: component.reference,
                value: Some(component.value),
                placement_layer: layer_id(component.layer),
                position: component.position,
                rotation_degrees: component.rotation as f32,
                bounds,
            }
        })
        .collect();
    let pads: Vec<PadPrimitive> = pads
        .into_iter()
        .map(|pad| {
            let bounds = pad_bounds(&pad);
            PadPrimitive {
                object_id: format!("pad:{}", pad.uuid),
                object_kind: "pad".to_string(),
                source_object_uuid: pad.uuid.clone(),
                pad_uuid: pad.uuid.clone(),
                component_uuid: pad.package.clone(),
                net_uuid: pad.net.clone(),
                layer_id: layer_id(pad.layer),
                copper_layer_ids: pad.copper_layers.into_iter().map(layer_id).collect(),
                center: pad.position,
                bounds,
                shape_kind: pad.shape,
                roundrect_rratio_ppm: pad.roundrect_rratio_ppm,
                mask_layer_ids: pad.mask_layers.into_iter().map(layer_id).collect(),
                paste_layer_ids: pad.paste_layers.into_iter().map(layer_id).collect(),
                solder_mask_margin_nm: pad.solder_mask_margin_nm,
                solder_paste_margin_nm: pad.solder_paste_margin_nm,
                solder_paste_margin_ratio_ppm: pad.solder_paste_margin_ratio_ppm,
                drill_nm: pad.drill,
                rotation_degrees: pad.rotation as f32,
            }
        })
        .collect();
    let tracks: Vec<TrackPrimitive> = tracks
        .into_iter()
        .map(|track| TrackPrimitive {
            object_id: format!("track:{}", track.uuid),
            object_kind: "track".to_string(),
            source_object_uuid: track.uuid.clone(),
            track_uuid: track.uuid.clone(),
            net_uuid: Some(track.net),
            layer_id: layer_id(track.layer),
            width_nm: track.width,
            path: vec![track.from, track.to],
        })
        .collect();
    let vias: Vec<ViaPrimitive> = vias
        .into_iter()
        .map(|via| ViaPrimitive {
            object_id: format!("via:{}", via.uuid),
            object_kind: "via".to_string(),
            source_object_uuid: via.uuid.clone(),
            via_uuid: via.uuid.clone(),
            net_uuid: Some(via.net),
            position: via.position,
            drill_nm: via.drill,
            diameter_nm: via.diameter,
            start_layer_id: layer_id(via.from_layer),
            end_layer_id: layer_id(via.to_layer),
        })
        .collect();
    let zones: Vec<ZonePrimitive> = zones
        .into_iter()
        .map(|zone| ZonePrimitive {
            object_id: format!("zone:{}", zone.uuid),
            object_kind: "zone".to_string(),
            source_object_uuid: zone.uuid.clone(),
            zone_uuid: zone.uuid.clone(),
            net_uuid: Some(zone.net),
            layer_id: layer_id(zone.layer),
            polygon: zone.polygon.vertices,
        })
        .collect();
    let outline_path = close_outline_path(outline.vertices, outline.closed);
    let bounds = scene_bounds(
        outline_path.iter(),
        components
            .iter()
            .flat_map(|c| rect_corners(c.bounds))
            .collect::<Vec<_>>()
            .iter(),
        pads.iter()
            .flat_map(|p| rect_corners(p.bounds))
            .collect::<Vec<_>>()
            .iter(),
        component_graphics
            .iter()
            .flat_map(|graphic| graphic.path.iter().copied())
            .collect::<Vec<_>>()
            .iter(),
        component_texts
            .iter()
            .map(|text| text.position)
            .collect::<Vec<_>>()
            .iter(),
        board_texts
            .iter()
            .map(|text| text.position)
            .collect::<Vec<_>>()
            .iter(),
        tracks
            .iter()
            .flat_map(|t| t.path.iter().copied())
            .collect::<Vec<_>>()
            .iter(),
        vias.iter().map(|v| v.position).collect::<Vec<_>>().iter(),
        zones
            .iter()
            .flat_map(|z| z.polygon.iter().copied())
            .collect::<Vec<_>>()
            .iter(),
    );

    BoardReviewSceneV1 {
        kind: "board_review_scene".to_string(),
        version: 1,
        scene_id: format!("board-review-scene:{}", inspect.board_uuid),
        project_uuid: inspect.project_uuid.clone(),
        project_name: inspect.project_name.clone(),
        board_uuid: inspect.board_uuid.clone(),
        board_name: Path::new(&inspect.board_path)
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or("board")
            .to_string(),
        units: "nm".to_string(),
        source_revision: format!(
            "project:{}:board:{}",
            inspect.project_uuid, inspect.board_uuid
        ),
        pad_expansion_setup,
        bounds,
        layers: layer_ids
            .into_iter()
            .enumerate()
            .map(|(render_order, layer)| {
                let name = inferred_scene_layer_name(&layer);
                SceneLayer {
                    layer_id: layer,
                    name: name.clone(),
                    kind: inferred_scene_layer_kind(&name).to_string(),
                    render_order: render_order as u32,
                    visible_by_default: inferred_scene_layer_visible_by_default(&name),
                }
            })
            .collect(),
        outline: vec![OutlinePolyline {
            object_id: format!("outline:{}", inspect.board_uuid),
            object_kind: "outline".to_string(),
            source_object_uuid: inspect.board_uuid.clone(),
            // `outline_layer_key` is the scene-level `L{n}` key for the layer
            // that owns the outline (typically Edge.Cuts). It is resolved by
            // the caller from the imported board's actual layer table so the
            // visibility toggle gates this primitive correctly under any
            // KiCad layer-numbering scheme (KiCad 7 uses id 44, KiCad 9 may
            // renumber). See M7-SCN-006/007 + the DOA2526 key-alignment fix.
            layer_id: outline_layer_key,
            path: outline_path,
        }],
        components,
        component_graphics,
        component_texts,
        pads,
        tracks,
        vias,
        zones,
        board_graphics,
        board_texts,
        board_text_geometries,
        glyph_mesh_assets,
        unrouted_primitives,
        net_display,
        proposal_overlay_primitives: Vec::new(),
        review_primitives: Vec::new(),
    }
}

fn component_silkscreen_primitives(
    component: &BoardComponentPayload,
    payload: ComponentSilkscreenPayload,
) -> (Vec<ComponentGraphicPrimitive>, Vec<ComponentTextPrimitive>) {
    let mut graphics = Vec::new();
    let mut texts = Vec::new();
    graphics.extend(payload.lines.into_iter().enumerate().map(|(index, line)| {
        ComponentGraphicPrimitive {
            graphic_id: format!(
                "component-graphic:{}:silk-line:{index}",
                payload.component_uuid
            ),
            component_uuid: payload.component_uuid.clone(),
            layer_id: Some(layer_id(line.layer)),
            primitive_kind: "polyline".to_string(),
            render_role: "component_silkscreen".to_string(),
            width_nm: Some(line.width_nm),
            closed: false,
            path: vec![
                transform_component_local_point(component, line.from),
                transform_component_local_point(component, line.to),
            ],
            holes: Vec::new(),
        }
    }));
    graphics.extend(
        payload
            .polylines
            .into_iter()
            .enumerate()
            .map(|(index, polyline)| ComponentGraphicPrimitive {
                graphic_id: format!(
                    "component-graphic:{}:silk-polyline:{index}",
                    payload.component_uuid
                ),
                component_uuid: payload.component_uuid.clone(),
                layer_id: Some(layer_id(polyline.layer)),
                primitive_kind: "polyline".to_string(),
                render_role: "component_silkscreen".to_string(),
                width_nm: Some(polyline.width_nm),
                closed: false,
                path: polyline
                    .vertices
                    .into_iter()
                    .map(|point| transform_component_local_point(component, point))
                    .collect(),
                holes: Vec::new(),
            }),
    );
    graphics.extend(
        payload
            .polygons
            .into_iter()
            .enumerate()
            .map(|(index, polygon)| ComponentGraphicPrimitive {
                graphic_id: format!(
                    "component-graphic:{}:silk-polygon:{index}",
                    payload.component_uuid
                ),
                component_uuid: payload.component_uuid.clone(),
                layer_id: Some(layer_id(polygon.layer)),
                primitive_kind: "polygon".to_string(),
                render_role: "component_silkscreen".to_string(),
                width_nm: Some(45_000),
                closed: true,
                path: polygon
                    .vertices
                    .into_iter()
                    .map(|point| transform_component_local_point(component, point))
                    .collect(),
                holes: Vec::new(),
            }),
    );
    graphics.extend(
        payload
            .circles
            .into_iter()
            .enumerate()
            .map(|(index, circle)| ComponentGraphicPrimitive {
                graphic_id: format!(
                    "component-graphic:{}:silk-circle:{index}",
                    payload.component_uuid
                ),
                component_uuid: payload.component_uuid.clone(),
                layer_id: Some(layer_id(circle.layer)),
                primitive_kind: "polyline".to_string(),
                render_role: "component_silkscreen".to_string(),
                width_nm: Some(circle.width_nm),
                closed: true,
                path: approximate_circle_path(component, circle.center, circle.radius_nm),
                holes: Vec::new(),
            }),
    );
    graphics.extend(payload.arcs.into_iter().enumerate().map(|(index, arc)| {
        ComponentGraphicPrimitive {
            graphic_id: format!(
                "component-graphic:{}:silk-arc:{index}",
                payload.component_uuid
            ),
            component_uuid: payload.component_uuid.clone(),
            layer_id: Some(layer_id(arc.layer)),
            primitive_kind: "polyline".to_string(),
            render_role: "component_silkscreen".to_string(),
            width_nm: Some(arc.width_nm),
            closed: false,
            path: approximate_arc_path(
                component,
                arc.center,
                arc.radius_nm,
                arc.start_angle,
                arc.end_angle,
            ),
            holes: Vec::new(),
        }
    }));
    texts.extend(payload.texts.into_iter().enumerate().map(|(index, text)| {
        ComponentTextPrimitive {
            text_id: format!("component-text:{}:silk:{index}", payload.component_uuid),
            component_uuid: payload.component_uuid.clone(),
            layer_id: Some(layer_id(text.layer)),
            render_role: "component_silkscreen".to_string(),
            text: text.text,
            position: transform_component_local_point(component, text.position),
            rotation_degrees: text.rotation as f32,
            height_nm: text.height_nm.max(text.stroke_width_nm * 3),
            face_name: None,
            stroke_width_nm: Some(text.stroke_width_nm),
            cached_polygons: Vec::new(),
        }
    }));
    (graphics, texts)
}

fn component_mechanical_primitives(
    component: &BoardComponentPayload,
    payload: ComponentMechanicalPayload,
) -> (Vec<ComponentGraphicPrimitive>, Vec<ComponentTextPrimitive>) {
    let mut graphics = Vec::new();
    let mut texts = Vec::new();
    graphics.extend(payload.lines.into_iter().enumerate().map(|(index, line)| {
        ComponentGraphicPrimitive {
            graphic_id: format!(
                "component-graphic:{}:mech-line:{index}",
                payload.component_uuid
            ),
            component_uuid: payload.component_uuid.clone(),
            layer_id: Some(layer_id(line.layer)),
            primitive_kind: "polyline".to_string(),
            render_role: "component_mechanical".to_string(),
            width_nm: Some(line.width_nm),
            closed: false,
            path: vec![
                transform_component_local_point(component, line.from),
                transform_component_local_point(component, line.to),
            ],
            holes: Vec::new(),
        }
    }));
    graphics.extend(
        payload
            .polylines
            .into_iter()
            .enumerate()
            .map(|(index, polyline)| ComponentGraphicPrimitive {
                graphic_id: format!(
                    "component-graphic:{}:mech-polyline:{index}",
                    payload.component_uuid
                ),
                component_uuid: payload.component_uuid.clone(),
                layer_id: Some(layer_id(polyline.layer)),
                primitive_kind: "polyline".to_string(),
                render_role: "component_mechanical".to_string(),
                width_nm: Some(polyline.width_nm),
                closed: false,
                path: polyline
                    .vertices
                    .into_iter()
                    .map(|point| transform_component_local_point(component, point))
                    .collect(),
                holes: Vec::new(),
            }),
    );
    graphics.extend(
        payload
            .polygons
            .into_iter()
            .enumerate()
            .map(|(index, polygon)| ComponentGraphicPrimitive {
                graphic_id: format!(
                    "component-graphic:{}:mech-polygon:{index}",
                    payload.component_uuid
                ),
                component_uuid: payload.component_uuid.clone(),
                layer_id: Some(layer_id(polygon.layer)),
                primitive_kind: "polygon".to_string(),
                render_role: "component_mechanical".to_string(),
                width_nm: Some(35_000),
                closed: true,
                path: polygon
                    .vertices
                    .into_iter()
                    .map(|point| transform_component_local_point(component, point))
                    .collect(),
                holes: Vec::new(),
            }),
    );
    graphics.extend(
        payload
            .circles
            .into_iter()
            .enumerate()
            .map(|(index, circle)| ComponentGraphicPrimitive {
                graphic_id: format!(
                    "component-graphic:{}:mech-circle:{index}",
                    payload.component_uuid
                ),
                component_uuid: payload.component_uuid.clone(),
                layer_id: Some(layer_id(circle.layer)),
                primitive_kind: "polyline".to_string(),
                render_role: "component_mechanical".to_string(),
                width_nm: Some(circle.width_nm),
                closed: true,
                path: approximate_circle_path(component, circle.center, circle.radius_nm),
                holes: Vec::new(),
            }),
    );
    graphics.extend(payload.arcs.into_iter().enumerate().map(|(index, arc)| {
        ComponentGraphicPrimitive {
            graphic_id: format!(
                "component-graphic:{}:mech-arc:{index}",
                payload.component_uuid
            ),
            component_uuid: payload.component_uuid.clone(),
            layer_id: Some(layer_id(arc.layer)),
            primitive_kind: "polyline".to_string(),
            render_role: "component_mechanical".to_string(),
            width_nm: Some(arc.width_nm),
            closed: false,
            path: approximate_arc_path(
                component,
                arc.center,
                arc.radius_nm,
                arc.start_angle,
                arc.end_angle,
            ),
            holes: Vec::new(),
        }
    }));
    texts.extend(payload.texts.into_iter().enumerate().map(|(index, text)| {
        ComponentTextPrimitive {
            text_id: format!("component-text:{}:mech:{index}", payload.component_uuid),
            component_uuid: payload.component_uuid.clone(),
            layer_id: Some(layer_id(text.layer)),
            render_role: "component_mechanical".to_string(),
            text: text.text,
            position: transform_component_local_point(component, text.position),
            rotation_degrees: text.rotation as f32,
            height_nm: text.height_nm.max(text.stroke_width_nm * 3),
            face_name: None,
            stroke_width_nm: Some(text.stroke_width_nm),
            cached_polygons: Vec::new(),
        }
    }));
    (graphics, texts)
}

fn transform_component_local_point(component: &BoardComponentPayload, point: PointNm) -> PointNm {
    transform_component_local_xy(component, point.x, point.y)
}

fn transform_component_local_xy(
    component: &BoardComponentPayload,
    x_nm: i64,
    y_nm: i64,
) -> PointNm {
    let radians = -(f64::from(component.rotation)).to_radians();
    let x = x_nm as f64;
    let y = y_nm as f64;
    let rotated_x = x * radians.cos() - y * radians.sin();
    let rotated_y = x * radians.sin() + y * radians.cos();
    PointNm {
        x: component.position.x + rotated_x.round() as i64,
        y: component.position.y + rotated_y.round() as i64,
    }
}

fn approximate_circle_path(
    component: &BoardComponentPayload,
    center: PointNm,
    radius_nm: i64,
) -> Vec<PointNm> {
    const CIRCLE_SEGMENTS: usize = 24;
    (0..=CIRCLE_SEGMENTS)
        .map(|idx| {
            let radians = ((idx as f64 / CIRCLE_SEGMENTS as f64) * 360.0).to_radians();
            let local_x = center.x as f64 + radius_nm as f64 * radians.cos();
            let local_y = center.y as f64 + radius_nm as f64 * radians.sin();
            transform_component_local_xy(component, local_x.round() as i64, local_y.round() as i64)
        })
        .collect()
}

fn approximate_arc_path(
    component: &BoardComponentPayload,
    center: PointNm,
    radius_nm: i64,
    start_angle_tenths: i32,
    end_angle_tenths: i32,
) -> Vec<PointNm> {
    const ARC_SEGMENT_ANGLE_TENTHS: i32 = 150;
    let mut sweep = end_angle_tenths - start_angle_tenths;
    if sweep <= 0 {
        sweep += 3600;
    }
    let segment_count = ((sweep + ARC_SEGMENT_ANGLE_TENTHS - 1) / ARC_SEGMENT_ANGLE_TENTHS).max(1);
    (0..=segment_count)
        .map(|idx| {
            let angle_tenths = start_angle_tenths + sweep * idx / segment_count;
            let radians = (f64::from(angle_tenths) / 10.0).to_radians();
            let local_x = center.x as f64 + radius_nm as f64 * radians.cos();
            let local_y = center.y as f64 + radius_nm as f64 * radians.sin();
            transform_component_local_xy(component, local_x.round() as i64, local_y.round() as i64)
        })
        .collect()
}

fn attach_review_primitives(
    scene: &mut BoardReviewSceneV1,
    review: &RouteProposalReviewPayload,
    selected_path_points: Option<&[PointNm]>,
) {
    let first_action_id = review
        .proposal_actions
        .first()
        .map(|action| action.action_id.as_str());
    scene.proposal_overlay_primitives = review
        .proposal_actions
        .iter()
        .enumerate()
        .map(|(index, action)| ProposalOverlayPrimitive {
            overlay_id: format!("proposal:{}:path", action.action_id),
            primitive_kind: "track_path".to_string(),
            proposal_action_id: action.action_id.clone(),
            layer_id: Some(layer_id(action.layer)),
            render_role: if Some(action.action_id.as_str()) == first_action_id {
                "proposed_focus".to_string()
            } else {
                "proposed_overlay".to_string()
            },
            width_nm: Some(action.width_nm),
            drill_nm: None,
            diameter_nm: None,
            path: overlay_path_for_action(index, action, review, selected_path_points),
        })
        .collect();
    if let Some(first) = review.proposal_actions.first() {
        scene
            .proposal_overlay_primitives
            .push(ProposalOverlayPrimitive {
                overlay_id: "proposal:from-anchor".to_string(),
                primitive_kind: "anchor_marker".to_string(),
                proposal_action_id: first.action_id.clone(),
                layer_id: Some(layer_id(first.layer)),
                render_role: "authored_related".to_string(),
                width_nm: None,
                drill_nm: None,
                diameter_nm: None,
                path: vec![
                    selected_path_points
                        .and_then(|points| points.first().copied())
                        .unwrap_or(first.from),
                ],
            });
    }
    if let Some(last) = review.proposal_actions.last() {
        scene
            .proposal_overlay_primitives
            .push(ProposalOverlayPrimitive {
                overlay_id: "proposal:to-anchor".to_string(),
                primitive_kind: "anchor_marker".to_string(),
                proposal_action_id: last.action_id.clone(),
                layer_id: Some(layer_id(last.layer)),
                render_role: "authored_related".to_string(),
                width_nm: None,
                drill_nm: None,
                diameter_nm: None,
                path: vec![
                    selected_path_points
                        .and_then(|points| points.last().copied())
                        .unwrap_or(last.to),
                ],
            });
    }
    let mut seen_segments = BTreeSet::new();
    scene.review_primitives = review
        .proposal_actions
        .iter()
        .enumerate()
        .filter(|(_, action)| seen_segments.insert(action.selected_path_segment_index))
        .map(|(index, action)| ReviewPrimitive {
            review_primitive_id: format!(
                "review:segment-{}",
                action.selected_path_segment_index + 1
            ),
            primitive_kind: "selected_segment_highlight".to_string(),
            evidence_key: Some(format!("segment:{}", action.selected_path_segment_index)),
            path: overlay_path_for_action(index, action, review, selected_path_points),
        })
        .collect();
}

fn run_cli_json<T: DeserializeOwned>(cli_prefix: &[String], args: &[&str]) -> Result<T> {
    let owned = args
        .iter()
        .map(|value| value.to_string())
        .collect::<Vec<_>>();
    run_cli_json_owned(cli_prefix, &owned)
}

fn run_cli_json_owned<T: DeserializeOwned>(cli_prefix: &[String], args: &[String]) -> Result<T> {
    let (program, prefix_args) = cli_prefix
        .split_first()
        .ok_or_else(|| anyhow::anyhow!("EDA_CLI_BIN resolved to an empty command"))?;
    let output = Command::new(program)
        .args(prefix_args)
        .arg("--format")
        .arg("json")
        .args(args)
        .output()
        .with_context(|| {
            format!(
                "failed to execute GUI data loader command: {} {}",
                program,
                args.join(" ")
            )
        })?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let detail = if stderr.trim().is_empty() {
            stdout.trim()
        } else {
            stderr.trim()
        };
        bail!("GUI data loader command failed: {}", detail);
    }
    let stdout =
        String::from_utf8(output.stdout).context("GUI data loader stdout was not UTF-8")?;
    serde_json::from_str(stdout.trim()).with_context(|| {
        format!(
            "failed to decode GUI data loader JSON for args: {}",
            args.join(" ")
        )
    })
}

fn cli_prefix() -> Vec<String> {
    if let Ok(configured) = std::env::var("EDA_CLI_BIN") {
        let parts: Vec<String> = configured
            .split_whitespace()
            .map(ToOwned::to_owned)
            .collect();
        if !parts.is_empty() {
            return parts;
        }
    }
    if let Some(binary) = resolve_workspace_eda_binary() {
        return vec![binary];
    }
    vec![
        "cargo".to_string(),
        "run".to_string(),
        "--quiet".to_string(),
        "-p".to_string(),
        "datum-eda-cli".to_string(),
        "--bin".to_string(),
        "datum-eda".to_string(),
        "--".to_string(),
    ]
}

fn resolve_workspace_eda_binary() -> Option<String> {
    let exe = std::env::current_exe().ok()?;
    let exe_dir = exe.parent()?;

    let direct = exe_dir.join("datum-eda");
    if direct.is_file() {
        return Some(direct.to_string_lossy().into_owned());
    }

    let deps_sibling = exe_dir.parent()?.join("datum-eda");
    if deps_sibling.is_file() {
        return Some(deps_sibling.to_string_lossy().into_owned());
    }

    None
}

// Scene import threads many primitive-geometry parameters.
#[allow(clippy::too_many_arguments)]
fn collect_layer_ids(
    components: &[BoardComponentPayload],
    component_graphics: &[ComponentGraphicPrimitive],
    pads: &[BoardPadPayload],
    tracks: &[BoardTrackPayload],
    vias: &[BoardViaPayload],
    zones: &[BoardZonePayload],
    board_graphics: &[BoardGraphicPrimitive],
    board_text_geometries: &[BoardTextGeometryPrimitive],
) -> Vec<String> {
    let mut layers = BTreeSet::new();
    for component in components {
        layers.insert(layer_id(component.layer));
    }
    for graphic in component_graphics {
        if let Some(layer) = &graphic.layer_id {
            layers.insert(layer.clone());
        }
    }
    for pad in pads {
        layers.insert(layer_id(pad.layer));
        for layer in &pad.copper_layers {
            layers.insert(layer_id(*layer));
        }
        for layer in &pad.mask_layers {
            layers.insert(layer_id(*layer));
        }
        for layer in &pad.paste_layers {
            layers.insert(layer_id(*layer));
        }
    }
    for track in tracks {
        layers.insert(layer_id(track.layer));
    }
    for via in vias {
        layers.insert(layer_id(via.from_layer));
        layers.insert(layer_id(via.to_layer));
    }
    for zone in zones {
        layers.insert(layer_id(zone.layer));
    }
    for graphic in board_graphics {
        layers.insert(graphic.layer_id.clone());
    }
    for text_geometry in board_text_geometries {
        layers.insert(text_geometry.layer_id.clone());
    }
    if layers.is_empty() {
        layers.insert(layer_id(0));
    }
    layers.into_iter().collect()
}

fn layer_id(layer: i32) -> String {
    format!("L{}", layer)
}

fn parse_layer_key(layer_key: &str) -> Option<i32> {
    layer_key.strip_prefix('L')?.parse::<i32>().ok()
}

fn standard_layer_name(id: i32) -> Option<&'static str> {
    match id {
        0 => Some("F.Cu"),
        31 => Some("B.Cu"),
        32 => Some("B.Adhes"),
        33 => Some("F.Adhes"),
        34 => Some("B.Paste"),
        35 => Some("F.Paste"),
        36 => Some("B.SilkS"),
        37 => Some("F.SilkS"),
        38 => Some("B.Mask"),
        39 => Some("F.Mask"),
        40 => Some("Dwgs.User"),
        41 => Some("Cmts.User"),
        42 => Some("Eco1.User"),
        43 => Some("Eco2.User"),
        44 => Some("Edge.Cuts"),
        45 => Some("Margin"),
        46 => Some("B.CrtYd"),
        47 => Some("F.CrtYd"),
        48 => Some("B.Fab"),
        49 => Some("F.Fab"),
        _ => None,
    }
}

fn inferred_scene_layer_name(layer_key: &str) -> String {
    parse_layer_key(layer_key)
        .and_then(standard_layer_name)
        .unwrap_or(layer_key)
        .to_string()
}

fn inferred_scene_layer_kind(layer_name: &str) -> &'static str {
    if layer_name == "F.Cu" || layer_name == "B.Cu" || layer_name.ends_with(".Cu") {
        "copper"
    } else if layer_name.ends_with(".Mask") {
        "mask"
    } else if layer_name.ends_with(".Paste") {
        "paste"
    } else if layer_name.ends_with(".SilkS") {
        "silkscreen"
    } else if layer_name == "Edge.Cuts"
        || layer_name.ends_with(".CrtYd")
        || layer_name.ends_with(".Fab")
    {
        "mechanical"
    } else {
        "other"
    }
}

fn inferred_scene_layer_visible_by_default(layer_name: &str) -> bool {
    layer_name == "F.Cu"
        || layer_name == "B.Cu"
        || layer_name.ends_with(".Cu")
        || layer_name == "Edge.Cuts"
        || layer_name == "F.SilkS"
}

fn scene_layers_from_native_stackup_value(board: &Value) -> Option<Vec<SceneLayer>> {
    let layers = board
        .get("stackup")
        .and_then(|stackup| stackup.get("layers"))
        .and_then(|layers| layers.as_array())?;
    let mut scene_layers = Vec::with_capacity(layers.len());
    for (render_order, layer) in layers.iter().enumerate() {
        let id = layer.get("id").and_then(|value| value.as_i64())? as i32;
        let name = layer
            .get("name")
            .and_then(|value| value.as_str())
            .unwrap_or_else(|| standard_layer_name(id).unwrap_or("layer"))
            .to_string();
        let kind = layer
            .get("layer_type")
            .and_then(|value| value.as_str())
            .map(native_stackup_layer_kind)
            .unwrap_or_else(|| inferred_scene_layer_kind(&name))
            .to_string();
        scene_layers.push(SceneLayer {
            layer_id: layer_id(id),
            name: name.clone(),
            kind,
            render_order: render_order as u32,
            visible_by_default: inferred_scene_layer_visible_by_default(&name),
        });
    }
    if scene_layers.is_empty() {
        None
    } else {
        Some(scene_layers)
    }
}

fn native_stackup_layer_kind(layer_type: &str) -> &'static str {
    match layer_type {
        "Copper" => "copper",
        "SolderMask" => "mask",
        "Paste" => "paste",
        "Silkscreen" => "silkscreen",
        "Mechanical" => "mechanical",
        "Dielectric" => "dielectric",
        _ => "other",
    }
}

fn component_bounds(
    component: &BoardComponentPayload,
    pads: &[&BoardPadPayload],
    graphics: &[&ComponentGraphicPrimitive],
    texts: &[&ComponentTextPrimitive],
) -> RectNm {
    let graphics: Vec<&ComponentGraphicPrimitive> = graphics
        .iter()
        .copied()
        .filter(|graphic| {
            graphic
                .layer_id
                .as_deref().is_none_or(|layer_id| inferred_scene_layer_name(layer_id) != "Edge.Cuts")
        })
        .collect();
    let texts: Vec<&ComponentTextPrimitive> = texts
        .iter()
        .copied()
        .filter(|text| {
            text
                .layer_id
                .as_deref().is_none_or(|layer_id| inferred_scene_layer_name(layer_id) != "Edge.Cuts")
        })
        .collect();
    if pads.is_empty() && graphics.is_empty() && texts.is_empty() {
        let radius = 600_000;
        return RectNm {
            min_x: component.position.x - radius,
            min_y: component.position.y - radius,
            max_x: component.position.x + radius,
            max_y: component.position.y + radius,
        };
    }
    let mut rect = RectNm {
        min_x: i64::MAX,
        min_y: i64::MAX,
        max_x: i64::MIN,
        max_y: i64::MIN,
    };
    for pad in pads {
        let pad_rect = pad_bounds(pad);
        rect.min_x = rect.min_x.min(pad_rect.min_x);
        rect.min_y = rect.min_y.min(pad_rect.min_y);
        rect.max_x = rect.max_x.max(pad_rect.max_x);
        rect.max_y = rect.max_y.max(pad_rect.max_y);
    }
    for graphic in &graphics {
        for point in &graphic.path {
            rect.min_x = rect.min_x.min(point.x);
            rect.min_y = rect.min_y.min(point.y);
            rect.max_x = rect.max_x.max(point.x);
            rect.max_y = rect.max_y.max(point.y);
        }
    }
    for text in &texts {
        rect.min_x = rect.min_x.min(text.position.x);
        rect.min_y = rect.min_y.min(text.position.y);
        rect.max_x = rect.max_x.max(text.position.x);
        rect.max_y = rect.max_y.max(text.position.y);
    }
    let has_graphics = !graphics.is_empty() || !texts.is_empty();
    let margin = if has_graphics { 120_000 } else { 250_000 };
    RectNm {
        min_x: rect.min_x - margin,
        min_y: rect.min_y - margin,
        max_x: rect.max_x + margin,
        max_y: rect.max_y + margin,
    }
}

fn pad_bounds(pad: &BoardPadPayload) -> RectNm {
    let half_width = match pad.shape.as_str() {
        "rect" | "oval" | "roundrect" | "round_rect" => (pad.width.max(1)) / 2,
        _ => (pad.diameter.max(1)) / 2,
    };
    let half_height = match pad.shape.as_str() {
        "rect" | "oval" | "roundrect" | "round_rect" => (pad.height.max(1)) / 2,
        _ => (pad.diameter.max(1)) / 2,
    };
    RectNm {
        min_x: pad.position.x - half_width,
        min_y: pad.position.y - half_height,
        max_x: pad.position.x + half_width,
        max_y: pad.position.y + half_height,
    }
}

fn close_outline_path(mut vertices: Vec<PointNm>, closed: bool) -> Vec<PointNm> {
    if closed
        && let (Some(first), Some(last)) = (vertices.first().copied(), vertices.last().copied())
        && first != last
    {
        vertices.push(first);
    }
    vertices
}

// Scene import threads many primitive-geometry parameters.
#[allow(clippy::too_many_arguments)]
fn scene_bounds<'a>(
    outline: impl Iterator<Item = &'a PointNm>,
    components: impl Iterator<Item = &'a PointNm>,
    pads: impl Iterator<Item = &'a PointNm>,
    component_graphics: impl Iterator<Item = &'a PointNm>,
    component_texts: impl Iterator<Item = &'a PointNm>,
    board_texts: impl Iterator<Item = &'a PointNm>,
    tracks: impl Iterator<Item = &'a PointNm>,
    vias: impl Iterator<Item = &'a PointNm>,
    zones: impl Iterator<Item = &'a PointNm>,
) -> SceneBounds {
    let mut points: Vec<PointNm> = Vec::new();
    points.extend(outline.copied());
    points.extend(components.copied());
    points.extend(pads.copied());
    points.extend(component_graphics.copied());
    points.extend(component_texts.copied());
    points.extend(board_texts.copied());
    points.extend(tracks.copied());
    points.extend(vias.copied());
    points.extend(zones.copied());
    if points.is_empty() {
        return SceneBounds {
            min_x: 0,
            min_y: 0,
            max_x: 1,
            max_y: 1,
        };
    }
    let mut min_x = i64::MAX;
    let mut min_y = i64::MAX;
    let mut max_x = i64::MIN;
    let mut max_y = i64::MIN;
    for point in points {
        min_x = min_x.min(point.x);
        min_y = min_y.min(point.y);
        max_x = max_x.max(point.x);
        max_y = max_y.max(point.y);
    }
    SceneBounds {
        min_x,
        min_y,
        max_x,
        max_y,
    }
}

fn rect_corners(rect: RectNm) -> [PointNm; 4] {
    [
        PointNm {
            x: rect.min_x,
            y: rect.min_y,
        },
        PointNm {
            x: rect.max_x,
            y: rect.min_y,
        },
        PointNm {
            x: rect.max_x,
            y: rect.max_y,
        },
        PointNm {
            x: rect.min_x,
            y: rect.max_y,
        },
    ]
}

pub fn load_fixture_workspace_state() -> ReviewWorkspaceState {
    let scene: BoardReviewSceneV1 =
        serde_json::from_str(include_str!("../testdata/board_review_scene_v1.json"))
            .expect("board review scene fixture should decode");
    let review: RouteProposalReviewPayload =
        serde_json::from_str(include_str!("../testdata/review_route_proposal.json"))
            .expect("route proposal review fixture should decode");
    ReviewWorkspaceState::new(scene, review)
}

#[cfg(test)]
mod kicad_text_import_tests;

#[cfg(test)]
mod tests {
    use super::kicad_scene_import::*;
    use super::*;
    use eda_engine::substrate::{CommitProvenance, CommitSource, Operation, OperationBatch};
    use serde_json::json;
    use uuid::Uuid;

    #[test]
    fn gui_action_narration_lands_in_console_never_on_terminal() {
        let mut state = load_fixture_workspace_state();
        let terminal_before = state.ui.terminal.lines.clone();

        let echo = "fit board".to_string();
        state.ui.push_console_line(echo.clone());

        // The echo lands in the invisible console sink.
        assert!(
            state.ui.console.lines.contains(&echo),
            "GUI-action narration should land in the console sink"
        );
        // The real PTY terminal display buffer is byte-for-byte unchanged and
        // never carries the GUI-action echo.
        assert_eq!(
            state.ui.terminal.lines, terminal_before,
            "GUI-action narration must not mutate the terminal display buffer"
        );
        assert!(
            !state.ui.terminal.lines.contains(&echo),
            "GUI-action narration must never appear in the terminal lane"
        );
    }

    fn unique_project_root(label: &str) -> PathBuf {
        std::env::temp_dir().join(format!("{label}-{}", Uuid::new_v4()))
    }

    fn write_json(path: &Path, value: serde_json::Value) {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).expect("fixture directory should create");
        }
        std::fs::write(path, format!("{value}\n")).expect("fixture JSON should write");
    }

    fn write_minimal_native_project(root: &Path, project_id: Uuid, board_id: Uuid) {
        write_json(
            &root.join("project.json"),
            json!({
                "schema_version": 1,
                "uuid": project_id,
                "name": "GUI Engine Scene Demo",
                "pools": [],
                "schematic": "schematic/schematic.json",
                "board": "board/board.json",
                "rules": "rules/rules.json"
            }),
        );
        write_json(
            &root.join("schematic/schematic.json"),
            json!({
                "schema_version": 1,
                "uuid": Uuid::new_v4(),
                "sheets": {},
                "definitions": {},
                "instances": [],
                "variants": {},
                "waivers": [],
                "deviations": []
            }),
        );
        write_json(
            &root.join("board/board.json"),
            json!({
                "schema_version": 1,
                "uuid": board_id,
                "name": "GUI Engine Scene Demo Board",
                "stackup": { "layers": [] },
                "outline": { "vertices": [], "closed": true },
                "packages": {},
                "component_silkscreen": {},
                "component_pads": {},
                "pads": {},
                "tracks": {},
                "vias": {},
                "zones": {},
                "nets": {},
                "net_classes": {},
                "dimensions": [],
                "texts": [],
                "keepouts": {}
            }),
        );
        write_json(
            &root.join("rules/rules.json"),
            json!({
                "schema_version": 1,
                "uuid": Uuid::new_v4(),
                "object_revision": 0,
                "rules": []
            }),
        );
    }

    #[test]
    fn native_board_scene_loads_resolver_materialized_board_state() {
        let root = unique_project_root("datum-gui-engine-scene");
        let project_id = Uuid::new_v4();
        let board_id = Uuid::new_v4();
        let text_id = Uuid::new_v4();
        write_minimal_native_project(&root, project_id, board_id);
        let mut model = ProjectResolver::new(&root)
            .resolve()
            .expect("native fixture should resolve");
        model
            .commit_journaled(
                &root,
                OperationBatch {
                    batch_id: Uuid::new_v4(),
                    expected_model_revision: Some(model.model_revision.clone()),
                    provenance: CommitProvenance {
                        actor: "gui-protocol-test".to_string(),
                        source: CommitSource::Test,
                        reason: "place board text for resolver-backed GUI scene".to_string(),
                    },
                    operations: vec![Operation::CreateBoardText {
                        text_id,
                        text: json!({
                            "uuid": text_id,
                            "text": "Resolver Truth",
                            "position": { "x": 1_000_000, "y": 2_000_000 },
                            "rotation": 0,
                            "layer": 37,
                            "height_nm": 1_000_000
                        }),
                    }],
                },
            )
            .expect("board text should commit through substrate");
        write_json(
            &root.join("board/board.json"),
            json!({
                "schema_version": 1,
                "uuid": board_id,
                "name": "GUI Engine Scene Demo Board",
                "stackup": { "layers": [] },
                "outline": { "vertices": [], "closed": true },
                "packages": {},
                "component_silkscreen": {},
                "component_pads": {},
                "pads": {},
                "tracks": {},
                "vias": {},
                "zones": {},
                "nets": {},
                "net_classes": {},
                "dimensions": [],
                "texts": [],
                "keepouts": {}
            }),
        );

        let (scene, board_path) = load_scene_from_engine(&LiveReviewRequest {
            project_root: root.clone(),
            board_file: None,
            artifact_path: None,
            net_uuid: None,
            from_anchor_pad_uuid: None,
            to_anchor_pad_uuid: None,
            profile: None,
            kicad_board_source: None,
        })
        .expect("resolver-backed native scene should load");

        assert_eq!(board_path, root.join("board/board.json"));
        assert_eq!(scene.project_uuid, project_id.to_string());
        assert_eq!(scene.board_uuid, board_id.to_string());
        assert_eq!(scene.source_revision, model.model_revision.0);
        assert!(
            scene
                .board_texts
                .iter()
                .any(|text| text.text_uuid == text_id.to_string() && text.text == "Resolver Truth"),
            "native GUI scene should reflect journal-materialized board text, not stale promoted board JSON"
        );
        let snapshot = load_gui_supervision_snapshot(
            &LiveReviewRequest {
                project_root: root.clone(),
                board_file: None,
                artifact_path: None,
                net_uuid: None,
                from_anchor_pad_uuid: None,
                to_anchor_pad_uuid: None,
                profile: None,
                kicad_board_source: None,
            },
            &scene,
            &ProductionStatus::default(),
            &SourceShardStatusSummary::default(),
            &CheckRunReviewState::default(),
        )
        .expect("supervision snapshot should load from resolver");

        assert_eq!(snapshot.contract, GUI_SUPERVISION_SNAPSHOT_CONTRACT);
        assert!(snapshot.read_only);
        assert_eq!(snapshot.project_uuid, project_id.to_string());
        assert_eq!(snapshot.model_revision, model.model_revision.0);
        assert_eq!(snapshot.journal.applied_transaction_count, 1);
        let expected_transaction_tip = model
            .journal
            .first()
            .map(|transaction| transaction.transaction_id.to_string());
        assert_eq!(
            snapshot.journal.accepted_transaction_tip.as_deref(),
            expected_transaction_tip.as_deref()
        );
        assert_eq!(snapshot.scene.board_text_count, scene.board_texts.len());
    }

    #[test]
    fn focused_artifact_prefers_latest_artifact_navigation_context() {
        let status = ProductionStatus {
            latest_artifact_id: Some("artifact-latest".to_string()),
            output_jobs: vec![production_output_job_for_focus(
                "job-old",
                Some("run-old"),
                Some("artifact-old"),
                "artifact-first",
            )],
            artifact_runs: vec![ProductionArtifactRunSummary {
                run_id: "run-latest".to_string(),
                artifact_id: "artifact-run-latest".to_string(),
                run_source: "artifact_run".to_string(),
                output_job_id: None,
                run_sequence: 2,
                status: "succeeded".to_string(),
                exit_code: Some(0),
            }],
            ..ProductionStatus::default()
        };

        assert_eq!(
            focused_artifact_id(&status).as_deref(),
            Some("artifact-latest")
        );
    }

    #[test]
    fn focused_artifact_uses_latest_output_job_run_before_first_job_artifact() {
        let status = ProductionStatus {
            latest_output_job_run_id: Some("run-new".to_string()),
            output_jobs: vec![
                production_output_job_for_focus(
                    "job-old",
                    Some("run-old"),
                    Some("artifact-old-run"),
                    "artifact-first-old",
                ),
                production_output_job_for_focus(
                    "job-new",
                    Some("run-new"),
                    Some("artifact-new-run"),
                    "artifact-first-new",
                ),
            ],
            artifact_runs: vec![ProductionArtifactRunSummary {
                run_id: "artifact-run-only".to_string(),
                artifact_id: "artifact-run-fallback".to_string(),
                run_source: "artifact_run".to_string(),
                output_job_id: None,
                run_sequence: 1,
                status: "succeeded".to_string(),
                exit_code: Some(0),
            }],
            ..ProductionStatus::default()
        };

        assert_eq!(
            focused_artifact_id(&status).as_deref(),
            Some("artifact-new-run")
        );
    }

    fn production_output_job_for_focus(
        id: &str,
        latest_run_id: Option<&str>,
        latest_run_artifact_id: Option<&str>,
        first_artifact_id: &str,
    ) -> ProductionOutputJobSummary {
        ProductionOutputJobSummary {
            id: id.to_string(),
            name: id.to_string(),
            include: Vec::new(),
            prefix: "test".to_string(),
            output_dir: None,
            family: "test".to_string(),
            status: "succeeded".to_string(),
            execution_count: usize::from(latest_run_id.is_some()),
            artifact_count: 1,
            latest_run_id: latest_run_id.map(str::to_string),
            latest_run_artifact_id: latest_run_artifact_id.map(str::to_string),
            artifacts: vec![ProductionArtifactSummary {
                artifact_id: first_artifact_id.to_string(),
                kind: "gerber_set".to_string(),
                project_id: None,
                model_revision: None,
                output_job: Some(id.to_string()),
                variant: None,
                generator_version: None,
                output_dir: None,
                validation_state: None,
                file_count: 0,
                files: Vec::new(),
                production_projection_count: 0,
                production_projections: Vec::new(),
            }],
        }
    }

    #[test]
    fn route_review_fixture_decodes_real_payload_shape() {
        let review: RouteProposalReviewPayload =
            serde_json::from_str(include_str!("../testdata/review_route_proposal.json"))
                .expect("review fixture should decode");
        assert_eq!(review.action, "review_route_proposal");
        assert_eq!(review.review_source, "selected_route_proposal");
        assert_eq!(review.proposal_actions.len(), 3);
        assert_eq!(review.proposal_actions[0].action_id, "action-1");
    }

    #[test]
    fn route_review_payload_accepts_null_segment_evidence() {
        let payload = r#"
        {
          "action": "review_route_proposal",
          "review_source": "selected_route_proposal",
          "status": "deterministic_route_proposal_ready",
          "explanation": "reviewing selected proposal",
          "project_root": "/tmp/datum-gui-m7-known-good",
          "artifact_path": null,
          "kind": null,
          "source_version": null,
          "version": null,
          "project_uuid": "project-fixture",
          "project_name": "Datum GUI Known Good",
          "net_uuid": "00000000-0000-0000-0000-00000000c200",
          "from_anchor_pad_uuid": "00000000-0000-0000-0000-00000000c205",
          "to_anchor_pad_uuid": "00000000-0000-0000-0000-00000000c206",
          "selection_profile": "default",
          "selection_rule": "select the first deterministic route proposal in accepted candidate order",
          "selected_candidate": "route-path-candidate",
          "selected_policy": null,
          "contract": "m5_route_path_candidate_v2",
          "actions": 1,
          "draw_track_actions": 1,
          "selected_path_bend_count": 0,
          "selected_path_point_count": 2,
          "selected_path_segment_count": 1,
          "segment_evidence": null,
          "proposal_actions": [
            {
              "action_id": "action-1",
              "proposal_action": "draw_track",
              "reason": "route_path_candidate",
              "contract": "m5_route_path_candidate_v2",
              "net_uuid": "00000000-0000-0000-0000-00000000c200",
              "net_name": "SIG",
              "from_anchor_pad_uuid": "00000000-0000-0000-0000-00000000c205",
              "to_anchor_pad_uuid": "00000000-0000-0000-0000-00000000c206",
              "layer": 1,
              "width_nm": 200000,
              "from": { "x": 500000, "y": 600000 },
              "to": { "x": 4500000, "y": 2400000 },
              "reused_via_uuid": null,
              "reused_via_uuids": [],
              "reused_object_kind": null,
              "reused_object_uuid": null,
              "reused_object_from_layer": null,
              "reused_object_to_layer": null,
              "selected_path_bend_count": 0,
              "selected_path_point_count": 2,
              "selected_path_segment_index": 0,
              "selected_path_segment_count": 1,
              "selected_path_layer_segment_index": null,
              "selected_path_layer_segment_count": null,
              "selected_path_layer_segment_bend_count": null,
              "selected_path_layer_segment_point_count": null
            }
          ]
        }"#;
        let review: RouteProposalReviewPayload =
            serde_json::from_str(payload).expect("null segment_evidence should decode");
        assert!(review.segment_evidence.is_empty());
        assert_eq!(review.proposal_actions.len(), 1);
    }

    #[test]
    fn board_review_scene_fixture_round_trips() {
        let scene: BoardReviewSceneV1 =
            serde_json::from_str(include_str!("../testdata/board_review_scene_v1.json"))
                .expect("scene fixture should decode");
        let json = serde_json::to_string_pretty(&scene).expect("scene should serialize");
        let decoded: BoardReviewSceneV1 =
            serde_json::from_str(&json).expect("scene should deserialize");
        assert_eq!(decoded, scene);
    }

    /// Pure contract round-trip for the M7-SCN-007 `BoardGraphicPrimitive`
    /// shape. Uses a hand-authored scene-JSON snippet — not a synthesized KiCad
    /// fixture — to verify the serde contract is stable.
    #[test]
    fn board_graphic_primitive_round_trips() {
        let json = r#"{
            "object_id": "board-graphic:abc123",
            "object_kind": "board_graphic",
            "primitive_kind": "line",
            "source_object_uuid": "abc123",
            "layer_id": "L44",
            "path": [
                { "x": 0, "y": 0 },
                { "x": 1000, "y": 0 }
            ],
            "width_nm": 50000
        }"#;
        let decoded: BoardGraphicPrimitive =
            serde_json::from_str(json).expect("board graphic primitive should decode");
        assert_eq!(decoded.object_kind, "board_graphic");
        assert_eq!(decoded.primitive_kind, "line");
        assert_eq!(decoded.layer_id, "L44");
        assert_eq!(decoded.path.len(), 2);
        assert_eq!(decoded.width_nm, Some(50_000));
        let re = serde_json::to_string(&decoded).expect("should re-serialize");
        let re_decoded: BoardGraphicPrimitive =
            serde_json::from_str(&re).expect("should re-decode");
        assert_eq!(re_decoded, decoded);
    }

    #[test]
    fn native_board_text_annotation_reaches_outline_geometry() {
        let board = json!({
            "texts": [
                {
                    "uuid": "00000000-0000-0000-0000-000000000001",
                    "text": "O",
                    "position": { "x": 0, "y": 0 },
                    "rotation": 0,
                    "layer": 37,
                    "render_intent": "annotation",
                    "height_nm": 1_000_000,
                    "stroke_width_nm": 152_000
                }
            ]
        });
        let (texts, geometries, glyph_mesh_assets) =
            extract_native_board_texts(&board).expect("native board text should extract");
        assert_eq!(texts.len(), 1);
        assert_eq!(
            texts[0].object_id,
            "board-text:00000000-0000-0000-0000-000000000001"
        );
        assert_eq!(texts[0].text, "O");
        assert_eq!(texts[0].render_intent, "annotation");
        assert_eq!(geometries.len(), 1);
        assert_eq!(
            geometries[0].object_id,
            "board-text:00000000-0000-0000-0000-000000000001"
        );
        assert!(!geometries[0].glyphs.is_empty());
        assert!(!glyph_mesh_assets.is_empty());
        assert!(
            geometries[0].fills.is_empty(),
            "mesh-backed text should not duplicate legacy fill fragments"
        );
        assert!(geometries[0].strokes.is_empty());
        assert!(geometries[0].glyphs.iter().all(|glyph| {
            glyph_mesh_assets
                .iter()
                .any(|asset| asset.handle == glyph.glyph_handle)
        }));
        assert!(
            glyph_mesh_assets
                .iter()
                .any(|asset| { asset.vertices.len() >= 3 && !asset.indices.is_empty() })
        );
    }

    /// `scene.board_graphics` must default to empty when a saved scene JSON
    /// predates the field — preserves back-compat with the existing checked-in
    /// `board_review_scene_v1.json` fixture.
    #[test]
    fn scene_board_graphics_defaults_empty_when_field_absent() {
        let scene: BoardReviewSceneV1 =
            serde_json::from_str(include_str!("../testdata/board_review_scene_v1.json"))
                .expect("scene fixture should decode");
        assert!(
            scene.board_graphics.is_empty(),
            "board_graphics must default to empty on pre-M7-SCN-007 scene fixtures"
        );
        assert!(
            scene.board_texts.is_empty(),
            "board_texts must default to empty on pre-Layer-B scene fixtures"
        );
        assert!(
            scene.board_text_geometries.is_empty(),
            "board_text_geometries must default to empty on pre-Phase-3 scene fixtures"
        );
        assert!(
            scene.glyph_mesh_assets.is_empty(),
            "glyph_mesh_assets must default to empty on pre-Phase-3 scene fixtures"
        );
    }

    #[test]
    fn outline_board_graphics_from_closed_outline_emits_edge_cuts_segments() {
        let outline = OutlinePayload {
            vertices: vec![
                PointNm { x: 0, y: 0 },
                PointNm { x: 10, y: 0 },
                PointNm { x: 10, y: 20 },
                PointNm { x: 0, y: 20 },
            ],
            closed: true,
        };
        let graphics = outline_board_graphics_from_outline(&outline, "board-123", "L44");
        assert_eq!(graphics.len(), 4);
        assert!(graphics.iter().all(|g| g.object_kind == "board_graphic"));
        assert!(graphics.iter().all(|g| g.primitive_kind == "line"));
        assert!(graphics.iter().all(|g| g.layer_id == "L44"));
        assert_eq!(
            graphics.first().expect("segment").path,
            vec![PointNm { x: 0, y: 0 }, PointNm { x: 10, y: 0 }]
        );
        assert_eq!(
            graphics.last().expect("segment").path,
            vec![PointNm { x: 0, y: 20 }, PointNm { x: 0, y: 0 }]
        );
    }

    #[test]
    fn workspace_state_defaults_to_first_proposal_action() {
        let state = load_fixture_workspace_state();
        assert_eq!(state.active_review_target_id, "action-1");
        assert_eq!(
            state.selection,
            SelectionTarget::ReviewAction("action-1".to_string())
        );
    }

    #[test]
    fn review_action_selection_updates_state() {
        let mut state = load_fixture_workspace_state();
        assert!(state.select_review_action("action-2"));
        assert_eq!(state.active_review_target_id, "action-2");
        assert_eq!(
            state.selection,
            SelectionTarget::ReviewAction("action-2".to_string())
        );
    }

    #[test]
    fn selected_segment_evidence_tracks_active_review_target() {
        let mut state = load_fixture_workspace_state();
        assert_eq!(
            state
                .selected_segment_evidence()
                .expect("fixture evidence should exist")
                .layer_segment_index,
            0
        );
        assert!(state.select_review_action("action-3"));
        assert_eq!(
            state
                .selected_segment_evidence()
                .expect("fixture evidence should stay addressable")
                .track_action_count,
            3
        );
    }

    #[test]
    fn authored_object_selection_preserves_active_review_target() {
        let mut state = load_fixture_workspace_state();
        assert!(state.select_authored_object("pad:P1"));
        assert_eq!(state.active_review_target_id, "action-1");
        assert_eq!(
            state.selection,
            SelectionTarget::AuthoredObject("pad:P1".to_string())
        );
    }

    #[test]
    fn select_flag_reference_resolves_to_authored_component_selection() {
        // Locks the Phase-2 `--select R1`-style flow the datum-test parity capture
        // relies on: resolve a human-stable reference designator against the loaded
        // scene (mirroring app_bootstrap's `--select` handling), then
        // select_authored_object must yield an AuthoredObject selection for that
        // component so the single-pane populated component inspector renders. The
        // gui-protocol fixture's sole component (U1) stands in for the datum-test
        // board's R1; both exercise the identical reference -> object_id path.
        let mut state = load_fixture_workspace_state();
        let reference = state.scene.components[0].reference.clone();
        let object_id = state
            .scene
            .components
            .iter()
            .find(|c| c.reference == reference)
            .map(|c| c.object_id.clone())
            .expect("fixture component reference should resolve to an object_id");
        assert!(
            state.select_authored_object(&object_id),
            "reference-resolved object_id should confirm scene membership"
        );
        assert_eq!(
            state.selection,
            SelectionTarget::AuthoredObject(object_id)
        );
    }

    #[test]
    fn check_finding_selection_preserves_active_review_target() {
        let mut state = load_fixture_workspace_state();
        state.checks.findings = vec![CheckFindingSummary {
            fingerprint: "sha256:finding-a".to_string(),
            rule_id: "process_aperture_policy".to_string(),
            ..CheckFindingSummary::default()
        }];
        assert!(state.select_check_finding("sha256:finding-a"));
        assert_eq!(state.active_review_target_id, "action-1");
        assert_eq!(
            state.selection,
            SelectionTarget::CheckFinding("sha256:finding-a".to_string())
        );
        assert!(!state.select_check_finding("sha256:missing"));
    }

    #[test]
    fn check_finding_target_resolves_scene_object_id() {
        let state = load_fixture_workspace_state();
        let pad_id = state.scene.pads[0].object_id.clone();
        let pad_uuid = state.scene.pads[0].pad_uuid.clone();
        let finding = CheckFindingSummary {
            fingerprint: "sha256:finding-a".to_string(),
            primary_target: json!({
                "kind": "pad_uuid",
                "id": pad_uuid
            }),
            ..CheckFindingSummary::default()
        };

        assert_eq!(
            check_finding_scene_target_object_id(&state.scene, &finding),
            Some(pad_id)
        );
    }

    #[test]
    fn clearing_authored_selection_preserves_active_review_target() {
        let mut state = load_fixture_workspace_state();
        assert!(state.select_authored_object("pad:P1"));
        state.clear_selection();
        assert_eq!(state.active_review_target_id, "action-1");
        assert_eq!(state.selection, SelectionTarget::None);
    }

    #[test]
    fn live_session_emits_selection_navigation_and_filter_events() {
        let workspace = load_fixture_workspace_state();
        let mut session = LiveDesignSession::new(workspace);

        let select = session.apply(SessionCommand::SelectAuthoredObject(
            "component:U1".to_string(),
        ));
        assert!(select.handled);
        assert!(
            select
                .events
                .iter()
                .any(|event| matches!(event, SessionEvent::SelectionChanged(_)))
        );

        let next = session.apply(SessionCommand::SelectNextReviewAction);
        assert!(next.handled);
        assert!(
            next.events
                .iter()
                .any(|event| matches!(event, SessionEvent::SelectionChanged(_)))
        );

        let filter = session.apply(SessionCommand::ToggleShowProposed);
        assert!(filter.handled);
        assert!(
            filter
                .events
                .iter()
                .any(|event| matches!(event, SessionEvent::FrameChanged))
        );
    }

    #[test]
    fn authoring_tool_switch_resets_gesture_and_reports_status() {
        let workspace = load_fixture_workspace_state();
        let mut session = LiveDesignSession::new(workspace);

        let set = session.apply(SessionCommand::SetTool(WorkspaceTool::DrawBoardTrack));
        assert!(set.handled);
        assert!(set.events.iter().any(|event| matches!(
            event,
            SessionEvent::ToolChanged(WorkspaceTool::DrawBoardTrack)
        )));

        let begin = session.apply(SessionCommand::BeginAuthoringGesture {
            world: PointNm {
                x: 151_000,
                y: 249_000,
            },
            target_object_id: None,
        });
        assert!(begin.handled);
        assert_eq!(
            session.workspace().authoring.gesture.anchor,
            Some(PointNm {
                x: 200_000,
                y: 200_000
            })
        );

        let select = session.apply(SessionCommand::SetTool(WorkspaceTool::Select));
        assert!(select.handled);
        assert!(!session.workspace().authoring.gesture.is_active());
        assert_eq!(
            session
                .workspace()
                .last_command_status
                .as_ref()
                .expect("status")
                .detail,
            "tool select"
        );
    }

    #[test]
    fn draw_board_track_handoff_uses_backing_and_review_defaults() {
        let mut state = load_fixture_workspace_state();
        state.backing = Some(WorkspaceBacking {
            request: LiveReviewRequest {
                project_root: PathBuf::from("/tmp/datum authoring demo"),
                board_file: None,
                artifact_path: None,
                net_uuid: None,
                from_anchor_pad_uuid: None,
                to_anchor_pad_uuid: None,
                profile: None,
                kicad_board_source: None,
            },
            board_path: PathBuf::from("/tmp/datum authoring demo/board/board.json"),
        });

        let handoff = state
            .draw_board_track_handoff(
                PointNm {
                    x: 100_000,
                    y: 200_000,
                },
                PointNm {
                    x: 300_000,
                    y: 400_000,
                },
            )
            .expect("handoff");

        assert_eq!(handoff.command_id, "datum.pcb.draw_board_track");
        assert!(handoff.command.contains("draw-board-track"));
        assert!(handoff.command.contains("'/tmp/datum authoring demo'"));
        assert!(handoff.command.contains("--from-x-nm 100000"));
        assert!(handoff.command.contains("--to-y-nm 400000"));
        assert!(handoff.command.contains("--width-nm"));
    }

    #[test]
    fn via_move_and_delete_authoring_handoffs_use_scene_context() {
        let mut state = load_fixture_workspace_state();
        state.backing = Some(WorkspaceBacking {
            request: LiveReviewRequest {
                project_root: PathBuf::from("/tmp/datum authoring demo"),
                board_file: None,
                artifact_path: None,
                net_uuid: None,
                from_anchor_pad_uuid: None,
                to_anchor_pad_uuid: None,
                profile: None,
                kicad_board_source: None,
            },
            board_path: PathBuf::from("/tmp/datum authoring demo/board/board.json"),
        });

        let via = state
            .place_board_via_handoff(PointNm {
                x: 500_000,
                y: 600_000,
            })
            .expect("via handoff");
        assert_eq!(via.command_id, "datum.pcb.place_board_via");
        assert!(via.command.contains("place-board-via"));
        assert!(via.command.contains("--x-nm 500000"));
        assert!(via.command.contains("--diameter-nm 600000"));

        let component = state.scene.components.first().expect("fixture component");
        let move_component = state
            .move_component_handoff(
                &component.object_id,
                PointNm {
                    x: 700_000,
                    y: 800_000,
                },
            )
            .expect("move handoff");
        assert_eq!(move_component.command_id, "datum.pcb.move_board_component");
        assert!(move_component.command.contains("move-board-component"));
        assert!(move_component.command.contains("--component"));
        assert!(move_component.command.contains(&component.component_uuid));

        let delete_track = state
            .delete_authored_object_handoff("track:00000000-0000-0000-0000-000000000123")
            .expect("delete track handoff");
        assert_eq!(delete_track.command_id, "datum.pcb.delete_board_track");
        assert!(delete_track.command.contains("delete-board-track"));
        assert!(delete_track.command.contains("--track"));

        let text = state
            .place_board_text_handoff(PointNm {
                x: 900_000,
                y: 1_000_000,
            })
            .expect("text handoff");
        assert_eq!(text.command_id, "datum.pcb.place_board_text");
        assert!(text.command.contains("place-board-text"));
        assert!(text.command.contains("--text TEXT"));
        assert!(text.command.contains("--x-nm 900000"));
        assert!(text.command.contains("--render-intent annotation"));

        let delete_text = state
            .delete_authored_object_handoff("board-text:00000000-0000-0000-0000-000000000456")
            .expect("delete text handoff");
        assert_eq!(delete_text.command_id, "datum.pcb.delete_board_text");
        assert!(delete_text.command.contains("delete-board-text"));
        assert!(delete_text.command.contains("--text"));
    }

    #[test]
    fn authored_toggle_is_frame_only() {
        let workspace = load_fixture_workspace_state();
        let mut session = LiveDesignSession::new(workspace);

        let result = session.apply(SessionCommand::ToggleShowAuthored);

        assert!(result.handled);
        assert_eq!(result.events, vec![SessionEvent::FrameChanged]);
    }

    #[test]
    fn proposed_toggle_is_frame_only() {
        let workspace = load_fixture_workspace_state();
        let mut session = LiveDesignSession::new(workspace);

        let result = session.apply(SessionCommand::ToggleShowProposed);

        assert!(result.handled);
        assert_eq!(result.events, vec![SessionEvent::FrameChanged]);
    }

    #[test]
    fn show_unrouted_toggle_is_frame_only_when_scene_has_no_unrouted_geometry() {
        let workspace = load_fixture_workspace_state();
        assert!(workspace.scene.unrouted_primitives.is_empty());
        let mut session = LiveDesignSession::new(workspace);

        let result = session.apply(SessionCommand::ToggleShowUnrouted);

        assert!(result.handled);
        assert_eq!(result.events, vec![SessionEvent::FrameChanged]);
    }

    #[test]
    fn show_unrouted_toggle_changes_scene_when_unrouted_geometry_exists() {
        let mut workspace = load_fixture_workspace_state();
        workspace.scene.unrouted_primitives.push(UnroutedPrimitive {
            object_id: "unrouted:test".to_string(),
            object_kind: "unrouted".to_string(),
            source_object_uuid: "test".to_string(),
            net_uuid: "net:test".to_string(),
            from_component: "U1".to_string(),
            from_pin: "1".to_string(),
            to_component: "U2".to_string(),
            to_pin: "2".to_string(),
            path: vec![PointNm { x: 0, y: 0 }, PointNm { x: 1_000, y: 0 }],
        });
        let mut session = LiveDesignSession::new(workspace);

        let result = session.apply(SessionCommand::ToggleShowUnrouted);

        assert!(result.handled);
        assert_eq!(result.events, vec![SessionEvent::SceneChanged]);
    }

    #[test]
    fn dim_unrelated_toggle_is_frame_only_without_focus_or_selection() {
        let mut workspace = load_fixture_workspace_state();
        workspace.review.proposal_actions.clear();
        workspace.selection = SelectionTarget::None;
        workspace.active_review_target_id = "no-proposal-action".to_string();
        let mut session = LiveDesignSession::new(workspace);

        let result = session.apply(SessionCommand::ToggleDimUnrelated);

        assert!(result.handled);
        assert_eq!(result.events, vec![SessionEvent::FrameChanged]);
    }

    #[test]
    fn dim_unrelated_toggle_changes_scene_with_authored_selection() {
        let mut workspace = load_fixture_workspace_state();
        workspace.review.proposal_actions.clear();
        workspace.selection = SelectionTarget::AuthoredObject("pad:P1".to_string());
        workspace.active_review_target_id = "no-proposal-action".to_string();
        let mut session = LiveDesignSession::new(workspace);

        let result = session.apply(SessionCommand::ToggleDimUnrelated);

        assert!(result.handled);
        assert_eq!(result.events, vec![SessionEvent::SceneChanged]);
    }

    #[test]
    fn layer_toggle_is_frame_only_for_board_graphic_only_layer() {
        let mut workspace = load_fixture_workspace_state();
        let layer_id = "L777".to_string();
        workspace.scene.layers.push(SceneLayer {
            layer_id: layer_id.clone(),
            name: "F.SilkS".to_string(),
            kind: "silkscreen".to_string(),
            render_order: 777,
            visible_by_default: true,
        });
        workspace
            .ui
            .filters
            .layer_visibility
            .insert(layer_id.clone(), true);
        workspace.scene.board_graphics.push(BoardGraphicPrimitive {
            object_id: "board-graphic:text-only".to_string(),
            object_kind: "board_graphic".to_string(),
            primitive_kind: "polyline".to_string(),
            source_object_uuid: "text-only".to_string(),
            layer_id: layer_id.clone(),
            path: vec![PointNm { x: 0, y: 0 }, PointNm { x: 1_000, y: 0 }],
            holes: Vec::new(),
            width_nm: Some(100_000),
        });
        let mut session = LiveDesignSession::new(workspace);

        let result = session.apply(SessionCommand::ToggleLayerVisibility(layer_id));

        assert!(result.handled);
        assert_eq!(result.events, vec![SessionEvent::FrameChanged]);
    }

    #[test]
    fn layer_toggle_changes_scene_for_copper_layer() {
        let workspace = load_fixture_workspace_state();
        let layer_id = workspace
            .scene
            .tracks
            .first()
            .map(|track| track.layer_id.clone())
            .unwrap_or_else(|| "L0".to_string());
        let mut session = LiveDesignSession::new(workspace);

        let result = session.apply(SessionCommand::ToggleLayerVisibility(layer_id));

        assert!(result.handled);
        assert_eq!(result.events, vec![SessionEvent::SceneChanged]);
    }

    #[test]
    fn attach_review_primitives_builds_overlay_from_review_payload() {
        let mut scene: BoardReviewSceneV1 =
            serde_json::from_str(include_str!("../testdata/board_review_scene_v1.json"))
                .expect("scene fixture should decode");
        scene.proposal_overlay_primitives.clear();
        scene.review_primitives.clear();
        let review: RouteProposalReviewPayload =
            serde_json::from_str(include_str!("../testdata/review_route_proposal.json"))
                .expect("review fixture should decode");

        attach_review_primitives(&mut scene, &review, None);

        assert_eq!(scene.review_primitives.len(), 3);
        assert!(
            scene
                .proposal_overlay_primitives
                .iter()
                .any(|primitive| primitive.primitive_kind == "anchor_marker")
        );
        assert_eq!(
            scene.proposal_overlay_primitives[0].proposal_action_id,
            "action-1"
        );
    }

    #[test]
    fn attach_review_primitives_prefers_selected_candidate_path_points() {
        let mut scene: BoardReviewSceneV1 =
            serde_json::from_str(include_str!("../testdata/board_review_scene_v1.json"))
                .expect("scene fixture should decode");
        scene.proposal_overlay_primitives.clear();
        scene.review_primitives.clear();
        let review = RouteProposalReviewPayload {
            proposal_actions: vec![RouteProposalActionPayload {
                action_id: "action-1".to_string(),
                proposal_action: "draw_track".to_string(),
                reason: "route_path_candidate_orthogonal_two_bend".to_string(),
                contract: "m5_route_path_candidate_orthogonal_two_bend_v1".to_string(),
                net_uuid: "net-1".to_string(),
                net_name: "SIG".to_string(),
                from_anchor_pad_uuid: "pad-1".to_string(),
                to_anchor_pad_uuid: "pad-2".to_string(),
                layer: 1,
                width_nm: 200_000,
                from: PointNm { x: 0, y: 0 },
                to: PointNm { x: 1_000_000, y: 0 },
                reused_via_uuid: None,
                reused_via_uuids: Vec::new(),
                reused_object_kind: None,
                reused_object_uuid: None,
                reused_object_from_layer: None,
                reused_object_to_layer: None,
                selected_path_bend_count: 2,
                selected_path_point_count: 4,
                selected_path_segment_index: 0,
                selected_path_segment_count: 1,
                selected_path_layer_segment_index: None,
                selected_path_layer_segment_count: None,
                selected_path_layer_segment_bend_count: None,
                selected_path_layer_segment_point_count: None,
            }],
            ..serde_json::from_str(include_str!("../testdata/review_route_proposal.json"))
                .expect("review fixture should decode")
        };

        let richer_path = vec![
            PointNm { x: 0, y: 0 },
            PointNm { x: 0, y: 400_000 },
            PointNm {
                x: 1_000_000,
                y: 400_000,
            },
            PointNm { x: 1_000_000, y: 0 },
        ];
        attach_review_primitives(&mut scene, &review, Some(&richer_path));

        assert_eq!(scene.proposal_overlay_primitives[0].path, richer_path);
        assert_eq!(scene.review_primitives[0].path, richer_path);
    }

    #[test]
    fn attach_review_primitives_slices_multi_action_candidate_path_points() {
        let mut scene: BoardReviewSceneV1 =
            serde_json::from_str(include_str!("../testdata/board_review_scene_v1.json"))
                .expect("scene fixture should decode");
        scene.proposal_overlay_primitives.clear();
        scene.review_primitives.clear();
        let mut review: RouteProposalReviewPayload =
            serde_json::from_str(include_str!("../testdata/review_route_proposal.json"))
                .expect("review fixture should decode");
        review.proposal_actions = vec![
            RouteProposalActionPayload {
                action_id: "action-1".to_string(),
                proposal_action: "draw_track".to_string(),
                reason: "orthogonal".to_string(),
                contract: "m5_route_path_candidate_orthogonal_two_bend_v1".to_string(),
                net_uuid: "net-1".to_string(),
                net_name: "SIG".to_string(),
                from_anchor_pad_uuid: "pad-1".to_string(),
                to_anchor_pad_uuid: "pad-2".to_string(),
                layer: 1,
                width_nm: 200_000,
                from: PointNm { x: 0, y: 0 },
                to: PointNm { x: 1_000_000, y: 0 },
                reused_via_uuid: None,
                reused_via_uuids: Vec::new(),
                reused_object_kind: None,
                reused_object_uuid: None,
                reused_object_from_layer: None,
                reused_object_to_layer: None,
                selected_path_bend_count: 2,
                selected_path_point_count: 4,
                selected_path_segment_index: 0,
                selected_path_segment_count: 3,
                selected_path_layer_segment_index: Some(0),
                selected_path_layer_segment_count: Some(3),
                selected_path_layer_segment_bend_count: Some(0),
                selected_path_layer_segment_point_count: Some(2),
            },
            RouteProposalActionPayload {
                action_id: "action-2".to_string(),
                proposal_action: "draw_track".to_string(),
                reason: "orthogonal".to_string(),
                contract: "m5_route_path_candidate_orthogonal_two_bend_v1".to_string(),
                net_uuid: "net-1".to_string(),
                net_name: "SIG".to_string(),
                from_anchor_pad_uuid: "pad-1".to_string(),
                to_anchor_pad_uuid: "pad-2".to_string(),
                layer: 1,
                width_nm: 200_000,
                from: PointNm { x: 1_000_000, y: 0 },
                to: PointNm { x: 2_000_000, y: 0 },
                reused_via_uuid: None,
                reused_via_uuids: Vec::new(),
                reused_object_kind: None,
                reused_object_uuid: None,
                reused_object_from_layer: None,
                reused_object_to_layer: None,
                selected_path_bend_count: 2,
                selected_path_point_count: 4,
                selected_path_segment_index: 1,
                selected_path_segment_count: 3,
                selected_path_layer_segment_index: Some(1),
                selected_path_layer_segment_count: Some(3),
                selected_path_layer_segment_bend_count: Some(1),
                selected_path_layer_segment_point_count: Some(2),
            },
            RouteProposalActionPayload {
                action_id: "action-3".to_string(),
                proposal_action: "draw_track".to_string(),
                reason: "orthogonal".to_string(),
                contract: "m5_route_path_candidate_orthogonal_two_bend_v1".to_string(),
                net_uuid: "net-1".to_string(),
                net_name: "SIG".to_string(),
                from_anchor_pad_uuid: "pad-1".to_string(),
                to_anchor_pad_uuid: "pad-2".to_string(),
                layer: 1,
                width_nm: 200_000,
                from: PointNm { x: 2_000_000, y: 0 },
                to: PointNm { x: 3_000_000, y: 0 },
                reused_via_uuid: None,
                reused_via_uuids: Vec::new(),
                reused_object_kind: None,
                reused_object_uuid: None,
                reused_object_from_layer: None,
                reused_object_to_layer: None,
                selected_path_bend_count: 2,
                selected_path_point_count: 4,
                selected_path_segment_index: 2,
                selected_path_segment_count: 3,
                selected_path_layer_segment_index: Some(2),
                selected_path_layer_segment_count: Some(3),
                selected_path_layer_segment_bend_count: Some(0),
                selected_path_layer_segment_point_count: Some(2),
            },
        ];

        let richer_path = vec![
            PointNm { x: 0, y: 0 },
            PointNm { x: 0, y: 500_000 },
            PointNm {
                x: 2_500_000,
                y: 500_000,
            },
            PointNm { x: 2_500_000, y: 0 },
        ];
        attach_review_primitives(&mut scene, &review, Some(&richer_path));

        assert_eq!(
            scene.proposal_overlay_primitives[0].path,
            vec![richer_path[0], richer_path[1]]
        );
        assert_eq!(
            scene.proposal_overlay_primitives[1].path,
            vec![richer_path[1], richer_path[2]]
        );
        assert_eq!(
            scene.proposal_overlay_primitives[2].path,
            vec![richer_path[2], richer_path[3]]
        );
        assert_eq!(
            scene.review_primitives[1].path,
            vec![richer_path[1], richer_path[2]]
        );
    }

    #[test]
    fn build_board_review_scene_derives_component_bounds_from_pads() {
        let inspect = ProjectInspectPayload {
            project_root: "/tmp/demo".to_string(),
            project_name: "Demo".to_string(),
            project_uuid: "project-1".to_string(),
            board_uuid: "board-1".to_string(),
            board_path: "/tmp/demo/board/board.json".to_string(),
        };
        let scene = build_board_review_scene(
            &inspect,
            OutlinePayload {
                vertices: vec![PointNm { x: 0, y: 0 }, PointNm { x: 100, y: 0 }],
                closed: false,
            },
            vec![BoardComponentPayload {
                uuid: "U1".to_string(),
                reference: "U1".to_string(),
                value: "IC".to_string(),
                position: PointNm { x: 50, y: 50 },
                rotation: 0,
                layer: 0,
                locked: false,
            }],
            vec![],
            vec![],
            ScenePadExpansionSetup::default(),
            vec![BoardPadPayload {
                uuid: "P1".to_string(),
                package: "U1".to_string(),
                name: "1".to_string(),
                net: Some("net-1".to_string()),
                position: PointNm { x: 40, y: 50 },
                layer: 0,
                copper_layers: vec![0],
                shape: "rect".to_string(),
                diameter: 0,
                width: 10,
                height: 10,
                roundrect_rratio_ppm: 250_000,
                mask_layers: vec![39],
                paste_layers: vec![35],
                solder_mask_margin_nm: 0,
                solder_paste_margin_nm: 0,
                solder_paste_margin_ratio_ppm: 0,
                drill: None,
                rotation: 0,
            }],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![NetDisplayEntry {
                net_uuid: "net-1".to_string(),
                net_name: "SIG".to_string(),
                airwire_color_rgb: [0.1, 0.2, 0.3],
            }],
            "L44".to_string(),
        );
        assert_eq!(scene.components.len(), 1);
        assert!(scene.components[0].bounds.min_x < 40);
        assert_eq!(scene.board_uuid, "board-1");
        assert!(scene.layers.iter().any(|layer| {
            layer.layer_id == "L39" && layer.name == "F.Mask" && !layer.visible_by_default
        }));
        assert!(scene.layers.iter().any(|layer| {
            layer.layer_id == "L35" && layer.name == "F.Paste" && !layer.visible_by_default
        }));
        assert_eq!(scene.net_display[0].net_uuid, "net-1");
        assert_eq!(scene.net_display[0].airwire_color_rgb, [0.1, 0.2, 0.3]);
    }

    #[test]
    fn deterministic_airwire_color_is_stable_for_same_net() {
        let color_a = deterministic_airwire_color(b"net-a");
        let color_b = deterministic_airwire_color(b"net-a");
        let color_c = deterministic_airwire_color(b"net-b");
        assert_eq!(color_a, color_b);
        assert_ne!(color_a, color_c);
    }

    #[test]
    fn native_stackup_scene_layers_preserve_nonstandard_kicad_layer_ids() {
        let board = json!({
            "stackup": {
                "layers": [
                    { "id": 0, "name": "F.Cu", "layer_type": "Copper", "thickness_nm": 0 },
                    { "id": 2, "name": "B.Cu", "layer_type": "Copper", "thickness_nm": 0 },
                    { "id": 13, "name": "F.Paste", "layer_type": "Paste", "thickness_nm": 0 },
                    { "id": 25, "name": "Edge.Cuts", "layer_type": "Mechanical", "thickness_nm": 0 }
                ]
            }
        });
        let layers = scene_layers_from_native_stackup_value(&board).expect("scene layers");

        assert!(layers.iter().any(|layer| {
            layer.layer_id == "L2"
                && layer.name == "B.Cu"
                && layer.kind == "copper"
                && layer.visible_by_default
        }));
        assert!(layers.iter().any(|layer| {
            layer.layer_id == "L13"
                && layer.name == "F.Paste"
                && layer.kind == "paste"
                && !layer.visible_by_default
        }));
        assert!(layers.iter().any(|layer| {
            layer.layer_id == "L25"
                && layer.name == "Edge.Cuts"
                && layer.kind == "mechanical"
                && layer.visible_by_default
        }));
    }

    #[test]
    fn component_graphics_transform_into_board_space() {
        let component = BoardComponentPayload {
            uuid: "U1".to_string(),
            reference: "U1".to_string(),
            value: "IC".to_string(),
            position: PointNm { x: 1_000, y: 2_000 },
            rotation: 90,
            layer: 1,
            locked: false,
        };
        let payload = ComponentSilkscreenPayload {
            component_uuid: "U1".to_string(),
            lines: vec![ComponentGraphicLinePayload {
                from: PointNm { x: 100, y: 0 },
                to: PointNm { x: 100, y: 200 },
                width_nm: 10,
                layer: 1,
            }],
            arcs: vec![],
            circles: vec![],
            polygons: vec![],
            polylines: vec![],
            texts: vec![],
        };
        let (graphics, texts) = component_silkscreen_primitives(&component, payload);
        assert_eq!(graphics.len(), 1);
        assert!(texts.is_empty());
        assert_eq!(graphics[0].path[0], PointNm { x: 1_000, y: 1_900 });
        assert_eq!(graphics[0].path[1], PointNm { x: 1_200, y: 1_900 });
    }

    #[test]
    fn component_bounds_use_tight_margin_around_attached_pads() {
        let component = BoardComponentPayload {
            uuid: "U1".to_string(),
            reference: "U1".to_string(),
            value: "IC".to_string(),
            position: PointNm {
                x: 1_000_000,
                y: 1_000_000,
            },
            rotation: 0,
            layer: 0,
            locked: false,
        };
        let pad = BoardPadPayload {
            uuid: "P1".to_string(),
            package: "U1".to_string(),
            name: "1".to_string(),
            net: Some("net-1".to_string()),
            position: PointNm {
                x: 1_000_000,
                y: 1_000_000,
            },
            layer: 0,
            copper_layers: vec![0],
            shape: "circle".to_string(),
            diameter: 450_000,
            width: 0,
            height: 0,
            roundrect_rratio_ppm: 250_000,
            mask_layers: vec![],
            paste_layers: vec![],
            solder_mask_margin_nm: 0,
            solder_paste_margin_nm: 0,
            solder_paste_margin_ratio_ppm: 0,
            drill: None,
            rotation: 0,
        };
        let bounds = component_bounds(&component, &[&pad], &[], &[]);
        assert_eq!(bounds.min_x, 525_000);
        assert_eq!(bounds.min_y, 525_000);
        assert_eq!(bounds.max_x, 1_475_000);
        assert_eq!(bounds.max_y, 1_475_000);
    }

    #[test]
    fn component_bounds_include_graphics_and_text_with_tighter_margin() {
        let component = BoardComponentPayload {
            uuid: "U1".to_string(),
            reference: "U1".to_string(),
            value: "IC".to_string(),
            position: PointNm {
                x: 1_000_000,
                y: 1_000_000,
            },
            rotation: 0,
            layer: 0,
            locked: false,
        };
        let graphic = ComponentGraphicPrimitive {
            graphic_id: "g1".to_string(),
            component_uuid: "U1".to_string(),
            layer_id: Some("L1".to_string()),
            primitive_kind: "polyline".to_string(),
            render_role: "component_mechanical".to_string(),
            width_nm: Some(100_000),
            closed: true,
            path: vec![
                PointNm {
                    x: 800_000,
                    y: 700_000,
                },
                PointNm {
                    x: 1_200_000,
                    y: 700_000,
                },
                PointNm {
                    x: 1_200_000,
                    y: 1_300_000,
                },
                PointNm {
                    x: 800_000,
                    y: 1_300_000,
                },
            ],
            holes: Vec::new(),
        };
        let text = ComponentTextPrimitive {
            text_id: "t1".to_string(),
            component_uuid: "U1".to_string(),
            layer_id: Some("L1".to_string()),
            render_role: "component_silkscreen".to_string(),
            text: "U1".to_string(),
            position: PointNm {
                x: 1_000_000,
                y: 650_000,
            },
            rotation_degrees: 0.0,
            height_nm: 1_000_000,
            face_name: None,
            stroke_width_nm: None,
            cached_polygons: Vec::new(),
        };
        let bounds = component_bounds(&component, &[], &[&graphic], &[&text]);
        assert_eq!(bounds.min_x, 680_000);
        assert_eq!(bounds.min_y, 530_000);
        assert_eq!(bounds.max_x, 1_320_000);
        assert_eq!(bounds.max_y, 1_420_000);
    }

    #[test]
    fn datum_test_q_components_own_their_pads() {
        let request = LiveReviewRequest {
            project_root: PathBuf::from(
                "/home/bfadmin/Documents/kicad_projects/Datum-eda/datum-test",
            ),
            board_file: Some(PathBuf::from(
                "/home/bfadmin/Documents/kicad_projects/Datum-eda/datum-test/datum-test.kicad_pcb",
            )),
            artifact_path: None,
            net_uuid: None,
            from_anchor_pad_uuid: None,
            to_anchor_pad_uuid: None,
            profile: None,
            kicad_board_source: None,
        };
        let workspace =
            load_board_editor_workspace_state(&request).expect("datum-test workspace should load");
        for reference in ["Q1", "Q2", "Q3", "Q4", "R2", "C1"] {
            let component = workspace
                .scene
                .components
                .iter()
                .find(|component| component.reference == reference)
                .unwrap_or_else(|| panic!("missing component {reference}"));
            let owned_pad_count = workspace
                .scene
                .pads
                .iter()
                .filter(|pad| pad.component_uuid == component.component_uuid)
                .count();
            assert!(
                owned_pad_count >= 2,
                "{reference} should own at least two pads, got {owned_pad_count}"
            );
        }
    }

    #[test]
    fn datum_test_board_load_also_carries_companion_schematic_scene() {
        let request = LiveReviewRequest {
            project_root: PathBuf::from(
                "/home/bfadmin/Documents/kicad_projects/Datum-eda/datum-test",
            ),
            board_file: Some(PathBuf::from(
                "/home/bfadmin/Documents/kicad_projects/Datum-eda/datum-test/datum-test.kicad_pcb",
            )),
            artifact_path: None,
            net_uuid: None,
            from_anchor_pad_uuid: None,
            to_anchor_pad_uuid: None,
            profile: None,
            kicad_board_source: None,
        };
        let workspace =
            load_board_editor_workspace_state(&request).expect("datum-test workspace should load");

        // The board scene must be unaffected: it is still the board, with
        // placed components and copper — not the schematic projection.
        assert_ne!(
            workspace.scene.kind, "schematic_review_scene",
            "board scene must remain the board, not be replaced by the schematic"
        );
        assert!(
            workspace
                .scene
                .components
                .iter()
                .any(|component| component.reference == "Q1"),
            "board scene should still carry placed board components"
        );

        // The companion schematic is carried alongside the board scene.
        let schematic = workspace
            .schematic_scene
            .as_ref()
            .expect("sibling datum-test.kicad_sch should populate schematic_scene");
        assert_eq!(
            schematic.kind, "schematic_review_scene",
            "companion scene should be the projected schematic review scene"
        );
        assert!(
            schematic
                .board_graphics
                .iter()
                .any(|graphic| graphic.object_id.starts_with("schematic-wire:")),
            "companion schematic should carry projected wire geometry"
        );
        assert!(
            schematic
                .board_graphics
                .iter()
                .any(|graphic| graphic.object_id.starts_with("schematic-junction:")),
            "companion schematic should carry projected junction geometry"
        );
        assert!(
            schematic.board_graphics.len() > 40,
            "companion schematic should carry a plausible primitive count, got {}",
            schematic.board_graphics.len()
        );
    }

    #[test]
    fn datum_test_materialize_path_carries_companion_schematic_scene() {
        // The app's default `--board <file>.kicad_pcb` load routes through the
        // materialize path: the KiCad board is imported into a native Datum
        // project and the board now loads from `<project>/board/board.json`,
        // whose sibling directory holds no `.kicad_sch`. The original KiCad
        // source must still be threaded through so pane B can draw the companion
        // schematic.
        let source = PathBuf::from(
            "/home/bfadmin/Documents/kicad_projects/Datum-eda/datum-test/datum-test.kicad_pcb",
        );
        let request = materialize_kicad_board_request(&source, None)
            .expect("datum-test should materialize into a native Datum project");

        // The board loads from the native project, not the KiCad file, yet the
        // original KiCad source is carried for companion resolution.
        assert!(
            request.board_file.is_none(),
            "materialize path loads the board from the native project, not the KiCad file"
        );
        assert_eq!(
            request.kicad_board_source.as_deref(),
            Some(source.canonicalize().unwrap().as_path()),
            "materialize path must carry the original KiCad source for companion resolution"
        );

        let workspace = load_board_editor_workspace_state(&request)
            .expect("materialized datum-test workspace should load");

        // Board scene is unaffected: still the native board, not the schematic.
        assert_ne!(
            workspace.scene.kind, "schematic_review_scene",
            "board scene must remain the board, not be replaced by the schematic"
        );

        // The companion schematic is populated from the ORIGINAL source sibling.
        let schematic = workspace
            .schematic_scene
            .as_ref()
            .expect("materialize path should populate schematic_scene from the original source");
        assert_eq!(
            schematic.kind, "schematic_review_scene",
            "companion scene should be the projected schematic review scene"
        );
        assert!(
            schematic
                .board_graphics
                .iter()
                .any(|graphic| graphic.object_id.starts_with("schematic-wire:")),
            "companion schematic should carry projected wire geometry"
        );
        assert!(
            schematic
                .board_graphics
                .iter()
                .any(|graphic| graphic.object_id.starts_with("schematic-junction:")),
            "companion schematic should carry projected junction geometry"
        );
    }

    #[test]
    fn datum_test_r2_reference_text_materializes_through_component_graphics() {
        let request = LiveReviewRequest {
            project_root: PathBuf::from(
                "/home/bfadmin/Documents/kicad_projects/Datum-eda/datum-test",
            ),
            board_file: Some(PathBuf::from(
                "/home/bfadmin/Documents/kicad_projects/Datum-eda/datum-test/datum-test.kicad_pcb",
            )),
            artifact_path: None,
            net_uuid: None,
            from_anchor_pad_uuid: None,
            to_anchor_pad_uuid: None,
            profile: None,
            kicad_board_source: None,
        };
        let workspace =
            load_board_editor_workspace_state(&request).expect("datum-test workspace should load");
        let r2 = workspace
            .scene
            .components
            .iter()
            .find(|component| component.reference == "R2")
            .expect("R2 should exist");
        assert!(
            workspace
                .scene
                .component_texts
                .iter()
                .all(|text| text.component_uuid != r2.component_uuid),
            "R2 reference text should not remain on the component_texts branch"
        );
        assert!(
            workspace.scene.board_texts.iter().any(|text| {
                text.text == "R2"
                    && text
                        .style_class
                        .as_deref()
                        .is_some_and(|class| class.contains(&r2.component_uuid))
            }),
            "R2 reference text should materialize through the structured board-text geometry path"
        );
        assert!(
            workspace
                .scene
                .component_graphics
                .iter()
                .all(|graphic| !graphic.graphic_id.contains(":prop-stroke:")),
            "R2 reference text should not remain on the stroke component-graphics branch"
        );
    }

    #[test]
    fn datum_test_q1_reference_text_materializes_through_datum_geometry() {
        let request = LiveReviewRequest {
            project_root: PathBuf::from(
                "/home/bfadmin/Documents/kicad_projects/Datum-eda/datum-test",
            ),
            board_file: Some(PathBuf::from(
                "/home/bfadmin/Documents/kicad_projects/Datum-eda/datum-test/datum-test.kicad_pcb",
            )),
            artifact_path: None,
            net_uuid: None,
            from_anchor_pad_uuid: None,
            to_anchor_pad_uuid: None,
            profile: None,
            kicad_board_source: None,
        };
        let workspace =
            load_board_editor_workspace_state(&request).expect("datum-test workspace should load");
        let q1 = workspace
            .scene
            .components
            .iter()
            .find(|component| component.reference == "Q1")
            .expect("Q1 should exist");
        assert!(
            workspace
                .scene
                .component_texts
                .iter()
                .all(|text| text.component_uuid != q1.component_uuid),
            "Q1 should not remain on the component_texts branch"
        );
        assert!(
            workspace.scene.board_texts.iter().any(|text| {
                text.text == "Q1"
                    && text
                        .style_class
                        .as_deref()
                        .is_some_and(|class| class.contains(&q1.component_uuid))
            }),
            "Q1 reference text should synthesize into the structured board-text geometry path"
        );
        assert!(
            workspace
                .scene
                .board_text_geometries
                .iter()
                .any(|geometry| !geometry.glyphs.is_empty()),
            "Q1 reference text path should emit mesh-backed text geometry"
        );
    }

    #[test]
    fn doa2526_q1_reference_text_materializes_through_datum_geometry() {
        let request = LiveReviewRequest {
            project_root: PathBuf::from(
                "/home/bfadmin/Documents/kicad_projects/DOA2526/hardware/DOA2526",
            ),
            board_file: Some(PathBuf::from(
                "/home/bfadmin/Documents/kicad_projects/DOA2526/hardware/DOA2526/DOA2526.kicad_pcb",
            )),
            artifact_path: None,
            net_uuid: None,
            from_anchor_pad_uuid: None,
            to_anchor_pad_uuid: None,
            profile: None,
            kicad_board_source: None,
        };
        let workspace =
            load_board_editor_workspace_state(&request).expect("DOA2526 workspace should load");
        let q1 = workspace
            .scene
            .components
            .iter()
            .find(|component| component.reference == "Q1")
            .expect("Q1 should exist");
        assert!(
            workspace.scene.board_texts.iter().any(|text| {
                text.text == "Q1"
                    && text
                        .style_class
                        .as_deref()
                        .is_some_and(|class| class.contains(&q1.component_uuid))
            }),
            "DOA2526 Q1 reference text should materialize through the same structured text geometry path as cache-absent fixtures"
        );
        assert!(
            workspace
                .scene
                .component_graphics
                .iter()
                .all(|graphic| !graphic.graphic_id.contains(":prop-stroke:")
                    && !graphic.graphic_id.contains(":prop-cache:")
                    && !graphic.graphic_id.contains(":kicad-text-cache:")),
            "DOA2526 should not retain stroke-derived or cache-derived imported text geometry ids"
        );
    }

    #[test]
    fn known_good_demo_request_materializes_project_scaffold() {
        let request = ensure_known_good_demo_request().expect("demo request should materialize");
        assert!(request.project_root.join("project.json").is_file());
        assert!(request.project_root.join("board/board.json").is_file());
        assert_eq!(
            request.net_uuid.as_deref(),
            Some("00000000-0000-0000-0000-00000000c200")
        );
        assert_eq!(
            request.from_anchor_pad_uuid.as_deref(),
            Some("00000000-0000-0000-0000-00000000c218")
        );
        let board_json = std::fs::read_to_string(request.project_root.join("board/board.json"))
            .expect("known-good board file should exist");
        let board: serde_json::Value =
            serde_json::from_str(&board_json).expect("known-good board should decode");
        assert!(
            board["packages"]
                .as_object()
                .expect("packages should be an object")
                .len()
                >= 4
        );
        assert!(
            board["pads"]
                .as_object()
                .expect("pads should be an object")
                .len()
                >= 4
        );
        let u1_mechanical_lines =
            board["component_mechanical_lines"]["00000000-0000-0000-0000-00000000c203"]
                .as_array()
                .expect("U1 mechanical lines should be an array");
        assert!(
            u1_mechanical_lines.len() >= 6,
            "KiCad-backed U1 package geometry should replace the minimal demo geometry"
        );
        let j2_mechanical_lines =
            board["component_mechanical_lines"]["00000000-0000-0000-0000-00000000c204"]
                .as_array()
                .expect("J2 mechanical lines should be an array");
        assert!(
            j2_mechanical_lines.len() >= 5,
            "KiCad-backed J2 package geometry should be materialized"
        );
        let tp1_mechanical_circles =
            board["component_mechanical_circles"]["00000000-0000-0000-0000-00000000c209"]
                .as_array()
                .expect("TP1 mechanical circles should be an array");
        assert!(
            !tp1_mechanical_circles.is_empty(),
            "KiCad-backed TP1 circular geometry should be materialized"
        );
    }

    #[test]
    fn materialized_kicad_board_defaults_to_stable_native_workspace_root() {
        let source = PathBuf::from("/tmp/example boards/DOA2526.kicad_pcb");
        let first = default_materialized_kicad_board_project_root(&source);
        let second = default_materialized_kicad_board_project_root(&source);

        assert_eq!(first, second);
        assert!(first.starts_with(std::env::temp_dir().join("datum-eda/gui-imports")));
        assert!(
            first
                .file_name()
                .and_then(|value| value.to_str())
                .is_some_and(|value| value.starts_with("DOA2526-"))
        );
        assert_eq!(
            materialized_kicad_board_project_name(&source),
            "DOA2526 Datum Workspace"
        );
    }

    #[test]
    fn native_round_rect_pad_shape_normalizes_to_renderer_shape_name() {
        let pad: EnginePadPayload = serde_json::from_value(json!({
            "uuid": "00000000-0000-0000-0000-000000000001",
            "package": "00000000-0000-0000-0000-000000000002",
            "name": "1",
            "position": { "x": 0, "y": 0 },
            "layer": 0,
            "shape": "round_rect"
        }))
        .expect("engine pad should decode");

        assert_eq!(pad.shape.to_string(), "roundrect");
    }

    #[test]
    fn known_good_demo_request_loads_live_workspace_state() {
        let request = ensure_known_good_demo_request().expect("demo request should materialize");
        let workspace =
            load_live_workspace_state(&request).expect("known-good demo should load live state");
        assert!(!workspace.review.proposal_actions.is_empty());
        assert!(!workspace.scene.pads.is_empty());
        assert_eq!(
            workspace.active_review_target_id,
            workspace.review.proposal_actions[0].action_id
        );
    }
}
