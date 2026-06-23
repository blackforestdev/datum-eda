use anyhow::{Context, Result, bail};
use eda_engine::board::BoardText;
use eda_engine::text::{
    TextAttributes, TextFamilyId, TextGeometryPrimitive, TextHAlign, TextRenderIntent, TextStyleId,
    TextVAlign, default_stroke_width_nm, layout_text_geometry, layout_text_mesh_from_board_text,
};
use serde::de::{DeserializeOwned, Deserializer};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Mutex, OnceLock};
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
mod production_proposals;
pub use production_proposals::{
    ProductionProposalPreviewSummary, ProductionProposalRenderDeltaSummary,
    ProductionProposalSummary, production_status_from_proposals_json,
};
use production_proposals::{ProposalsPayload, attach_proposal_validation, proposal_summaries};
mod check_runs;
pub use check_runs::{
    CheckFindingSummary, CheckRunCoverageSummary, CheckRunProfileBasisSummary, CheckRunReviewState,
    check_finding_scene_target_object_id, check_run_review_state_from_json,
};
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
pub struct BoardTextPrimitive {
    pub object_id: String,
    pub object_kind: String,
    pub text_uuid: String,
    pub text: String,
    pub layer_id: String,
    pub position: PointNm,
    pub rotation_degrees: i32,
    pub height_nm: i64,
    pub stroke_width_nm: i64,
    pub render_intent: String,
    pub family: String,
    pub style: String,
    #[serde(default)]
    pub style_class: Option<String>,
    pub h_align: String,
    pub v_align: String,
    pub mirrored: bool,
    pub keep_upright: bool,
    pub line_spacing_ratio_ppm: i32,
    pub bold: bool,
    pub italic: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BoardTextGeometryPrimitive {
    pub object_id: String,
    pub object_kind: String,
    pub text_uuid: String,
    pub layer_id: String,
    #[serde(default)]
    pub world_transform_nm: Option<Affine2DFixedPrimitive>,
    #[serde(default)]
    pub block_bbox_em_nm: Option<MeshRectEmPrimitive>,
    #[serde(default)]
    pub glyphs: Vec<TextGlyphInstancePrimitive>,
    #[serde(default)]
    pub fills: Vec<BoardTextFillPrimitive>,
    #[serde(default)]
    pub strokes: Vec<BoardTextStrokePrimitive>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct GlyphMeshHandlePrimitive {
    pub font_id: u32,
    pub glyph_id: u32,
    pub tolerance_class: u8,
    pub epoch: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GlyphMeshAssetPrimitive {
    pub handle: GlyphMeshHandlePrimitive,
    pub vertices: Vec<MeshVertexEmPrimitive>,
    pub indices: Vec<u32>,
    pub bbox_em_nm: MeshRectEmPrimitive,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct TextGlyphInstancePrimitive {
    pub glyph_handle: GlyphMeshHandlePrimitive,
    pub origin_em_nm_x: i64,
    pub origin_em_nm_y: i64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct MeshVertexEmPrimitive {
    pub x_em_nm: i64,
    pub y_em_nm: i64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct MeshRectEmPrimitive {
    pub min_x_em_nm: i64,
    pub min_y_em_nm: i64,
    pub max_x_em_nm: i64,
    pub max_y_em_nm: i64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct Affine2DFixedPrimitive {
    pub m11_ppm: i64,
    pub m12_ppm: i64,
    pub m21_ppm: i64,
    pub m22_ppm: i64,
    pub tx_nm: i64,
    pub ty_nm: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BoardTextFillPrimitive {
    pub outer: Vec<PointNm>,
    #[serde(default)]
    pub holes: Vec<Vec<PointNm>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BoardTextStrokePrimitive {
    pub from: PointNm,
    pub to: PointNm,
    pub width_nm: i64,
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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditorCommandStatus {
    pub action: String,
    pub detail: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DockTab {
    Terminal,
    Assistant,
    Outputs,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerminalLaneState {
    pub lines: Vec<String>,
    pub activity_summary: Vec<String>,
    pub input: String,
    pub cursor: usize,
    pub scroll_offset: usize,
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssistantMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssistantLaneState {
    pub transcript: Vec<AssistantMessage>,
    pub input: String,
    pub cursor: usize,
    pub awaiting_api_key: bool,
    pub scroll_offset: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceFilterState {
    pub show_authored: bool,
    pub show_proposed: bool,
    pub show_unrouted: bool,
    pub dim_unrelated: bool,
    pub layer_visibility: BTreeMap<String, bool>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceUiState {
    pub active_dock_tab: Option<DockTab>,
    pub dock_height_px: u32,
    pub hovered_object_id: Option<String>,
    pub filters: WorkspaceFilterState,
    pub terminal: TerminalLaneState,
    pub assistant: AssistantLaneState,
    pub artifact_preview: ArtifactPreviewViewportState,
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
    PanArtifactPreview { delta_x_ppm: i32, delta_y_ppm: i32 },
    ResetArtifactPreviewViewport,
    ToggleArtifactPreviewGeometry,
    ToggleArtifactPreviewDrills,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionEvent {
    SelectionChanged(SelectionTarget),
    SceneChanged,
    FrameChanged,
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
    pub review: RouteProposalReviewPayload,
    pub production: ProductionStatus,
    pub checks: CheckRunReviewState,
    pub selection: SelectionTarget,
    pub active_review_target_id: String,
    pub tool: WorkspaceTool,
    pub backing: Option<WorkspaceBacking>,
    pub last_command_status: Option<EditorCommandStatus>,
    pub ui: WorkspaceUiState,
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
}

pub fn ensure_known_good_demo_request() -> Result<LiveReviewRequest> {
    static DEMO_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    let _guard = DEMO_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .expect("demo lock should not be poisoned");
    let root = std::env::temp_dir().join("datum-gui-m7-known-good");
    write_known_good_demo_project(&root)?;
    Ok(LiveReviewRequest {
        project_root: root,
        board_file: None,
        artifact_path: None,
        net_uuid: Some("00000000-0000-0000-0000-00000000c200".to_string()),
        from_anchor_pad_uuid: Some("00000000-0000-0000-0000-00000000c218".to_string()),
        to_anchor_pad_uuid: Some("00000000-0000-0000-0000-00000000c219".to_string()),
        profile: Some("default".to_string()),
    })
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

fn write_known_good_demo_project(root: &Path) -> Result<()> {
    let schematic_dir = root.join("schematic");
    let board_dir = root.join("board");
    let rules_dir = root.join("rules");
    std::fs::create_dir_all(&schematic_dir)
        .with_context(|| format!("failed to create {}", schematic_dir.display()))?;
    std::fs::create_dir_all(&board_dir)
        .with_context(|| format!("failed to create {}", board_dir.display()))?;
    std::fs::create_dir_all(&rules_dir)
        .with_context(|| format!("failed to create {}", rules_dir.display()))?;

    write_json_file(
        &root.join("project.json"),
        &serde_json::json!({
            "schema_version": 1,
            "uuid": "00000000-0000-0000-0000-00000000c100",
            "name": "Datum GUI Known Good",
            "pools": [],
            "schematic": "schematic/schematic.json",
            "board": "board/board.json",
            "rules": "rules/rules.json",
            "forward_annotation_review": {}
        }),
    )?;
    write_json_file(
        &schematic_dir.join("schematic.json"),
        &serde_json::json!({
            "schema_version": 1,
            "uuid": "00000000-0000-0000-0000-00000000c101",
            "sheets": {},
            "definitions": {},
            "instances": [],
            "variants": {},
            "waivers": []
        }),
    )?;
    write_json_file(
        &rules_dir.join("rules.json"),
        &serde_json::json!({
            "schema_version": 1,
            "rules": []
        }),
    )?;
    write_json_file(
        &board_dir.join("board.json"),
        &serde_json::json!({
            "schema_version": 1,
            "uuid": "00000000-0000-0000-0000-00000000c207",
            "name": "Route Path Candidate Proposal Artifact Demo Board",
            "stackup": {
                "layers": [
                    { "id": 1, "name": "Top Copper", "layer_type": "Copper", "thickness_nm": 35000 },
                    { "id": 2, "name": "Core", "layer_type": "Dielectric", "thickness_nm": 1600000 },
                    { "id": 3, "name": "Bottom Copper", "layer_type": "Copper", "thickness_nm": 35000 }
                ]
            },
            "outline": {
                "vertices": [
                    { "x": 0, "y": 0 },
                    { "x": 24000000, "y": 0 },
                    { "x": 24000000, "y": 14000000 },
                    { "x": 0, "y": 14000000 }
                ],
                "closed": true
            },
            "packages": {
                "00000000-0000-0000-0000-00000000c203": {
                    "uuid": "00000000-0000-0000-0000-00000000c203",
                    "package": "10000000-0000-0000-0000-00000000c203",
                    "part": "20000000-0000-0000-0000-00000000c203",
                    "reference": "U1",
                    "value": "SOIC-8_3.9x4.9mm_P1.27mm",
                    "position": { "x": 4500000, "y": 3365000 },
                    "rotation": 0,
                    "layer": 1,
                    "locked": false
                },
                "00000000-0000-0000-0000-00000000c204": {
                    "uuid": "00000000-0000-0000-0000-00000000c204",
                    "package": "10000000-0000-0000-0000-00000000c204",
                    "part": "20000000-0000-0000-0000-00000000c204",
                    "reference": "J2",
                    "value": "PinHeader_1x03_P2.54mm_Vertical",
                    "position": { "x": 18000000, "y": 1460000 },
                    "rotation": 0,
                    "layer": 1,
                    "locked": false
                },
                "00000000-0000-0000-0000-00000000c208": {
                    "uuid": "00000000-0000-0000-0000-00000000c208",
                    "package": "10000000-0000-0000-0000-00000000c208",
                    "part": "20000000-0000-0000-0000-00000000c208",
                    "reference": "R1",
                    "value": "R_0805_2012Metric",
                    "position": { "x": 7000000, "y": 10200000 },
                    "rotation": 0,
                    "layer": 1,
                    "locked": false
                },
                "00000000-0000-0000-0000-00000000c209": {
                    "uuid": "00000000-0000-0000-0000-00000000c209",
                    "package": "10000000-0000-0000-0000-00000000c209",
                    "part": "20000000-0000-0000-0000-00000000c209",
                    "reference": "TP1",
                    "value": "TestPoint_Loop_D2.60mm_Drill1.4mm_Beaded",
                    "position": { "x": 12500000, "y": 10200000 },
                    "rotation": 0,
                    "layer": 1,
                    "locked": false
                }
            },
            "pads": {
                "00000000-0000-0000-0000-00000000c205": {
                    "uuid": "00000000-0000-0000-0000-00000000c205",
                    "package": "00000000-0000-0000-0000-00000000c203",
                    "name": "6",
                    "net": "00000000-0000-0000-0000-00000000c201",
                    "position": { "x": 6975000, "y": 4000000 },
                    "layer": 1,
                    "shape": "rect",
                    "diameter": 0,
                    "width": 1950000,
                    "height": 600000
                },
                "00000000-0000-0000-0000-00000000c212": {
                    "uuid": "00000000-0000-0000-0000-00000000c212",
                    "package": "00000000-0000-0000-0000-00000000c203",
                    "name": "1",
                    "net": "00000000-0000-0000-0000-00000000c201",
                    "position": { "x": 2025000, "y": 1460000 },
                    "layer": 1,
                    "shape": "rect",
                    "diameter": 0,
                    "width": 1950000,
                    "height": 600000
                },
                "00000000-0000-0000-0000-00000000c213": {
                    "uuid": "00000000-0000-0000-0000-00000000c213",
                    "package": "00000000-0000-0000-0000-00000000c203",
                    "name": "2",
                    "net": "00000000-0000-0000-0000-00000000c201",
                    "position": { "x": 2025000, "y": 2730000 },
                    "layer": 1,
                    "shape": "rect",
                    "diameter": 0,
                    "width": 1950000,
                    "height": 600000
                },
                "00000000-0000-0000-0000-00000000c214": {
                    "uuid": "00000000-0000-0000-0000-00000000c214",
                    "package": "00000000-0000-0000-0000-00000000c203",
                    "name": "3",
                    "net": "00000000-0000-0000-0000-00000000c201",
                    "position": { "x": 2025000, "y": 4000000 },
                    "layer": 1,
                    "shape": "rect",
                    "diameter": 0,
                    "width": 1950000,
                    "height": 600000
                },
                "00000000-0000-0000-0000-00000000c215": {
                    "uuid": "00000000-0000-0000-0000-00000000c215",
                    "package": "00000000-0000-0000-0000-00000000c203",
                    "name": "7",
                    "net": "00000000-0000-0000-0000-00000000c201",
                    "position": { "x": 6975000, "y": 2730000 },
                    "layer": 1,
                    "shape": "rect",
                    "diameter": 0,
                    "width": 1950000,
                    "height": 600000
                },
                "00000000-0000-0000-0000-00000000c218": {
                    "uuid": "00000000-0000-0000-0000-00000000c218",
                    "package": "00000000-0000-0000-0000-00000000c203",
                    "name": "8",
                    "net": "00000000-0000-0000-0000-00000000c200",
                    "position": { "x": 6975000, "y": 1460000 },
                    "layer": 1,
                    "shape": "rect",
                    "diameter": 0,
                    "width": 1950000,
                    "height": 600000
                },
                "00000000-0000-0000-0000-00000000c221": {
                    "uuid": "00000000-0000-0000-0000-00000000c221",
                    "package": "00000000-0000-0000-0000-00000000c203",
                    "name": "4",
                    "net": "00000000-0000-0000-0000-00000000c201",
                    "position": { "x": 2025000, "y": 5270000 },
                    "layer": 1,
                    "shape": "rect",
                    "diameter": 0,
                    "width": 1950000,
                    "height": 600000
                },
                "00000000-0000-0000-0000-00000000c222": {
                    "uuid": "00000000-0000-0000-0000-00000000c222",
                    "package": "00000000-0000-0000-0000-00000000c203",
                    "name": "5",
                    "net": "00000000-0000-0000-0000-00000000c201",
                    "position": { "x": 6975000, "y": 5270000 },
                    "layer": 1,
                    "shape": "rect",
                    "diameter": 0,
                    "width": 1950000,
                    "height": 600000
                },
                "00000000-0000-0000-0000-00000000c206": {
                    "uuid": "00000000-0000-0000-0000-00000000c206",
                    "package": "00000000-0000-0000-0000-00000000c204",
                    "name": "2",
                    "net": "00000000-0000-0000-0000-00000000c201",
                    "position": { "x": 18000000, "y": 4000000 },
                    "layer": 1,
                    "shape": "circle",
                    "diameter": 1700000,
                    "width": 0,
                    "height": 0,
                    "drill": 1000000
                },
                "00000000-0000-0000-0000-00000000c219": {
                    "uuid": "00000000-0000-0000-0000-00000000c219",
                    "package": "00000000-0000-0000-0000-00000000c204",
                    "name": "1",
                    "net": "00000000-0000-0000-0000-00000000c200",
                    "position": { "x": 18000000, "y": 1460000 },
                    "layer": 1,
                    "shape": "rect",
                    "diameter": 0,
                    "width": 1700000,
                    "height": 1700000,
                    "drill": 1000000
                },
                "00000000-0000-0000-0000-00000000c220": {
                    "uuid": "00000000-0000-0000-0000-00000000c220",
                    "package": "00000000-0000-0000-0000-00000000c204",
                    "name": "3",
                    "net": "00000000-0000-0000-0000-00000000c201",
                    "position": { "x": 18000000, "y": 6540000 },
                    "layer": 1,
                    "shape": "circle",
                    "diameter": 1700000,
                    "width": 0,
                    "height": 0,
                    "drill": 1000000
                },
                "00000000-0000-0000-0000-00000000c20a": {
                    "uuid": "00000000-0000-0000-0000-00000000c20a",
                    "package": "00000000-0000-0000-0000-00000000c208",
                    "name": "1",
                    "net": "00000000-0000-0000-0000-00000000c201",
                    "position": { "x": 6087500, "y": 10200000 },
                    "layer": 1,
                    "shape": "rect",
                    "diameter": 0,
                    "width": 1025000,
                    "height": 1400000
                },
                "00000000-0000-0000-0000-00000000c20b": {
                    "uuid": "00000000-0000-0000-0000-00000000c20b",
                    "package": "00000000-0000-0000-0000-00000000c208",
                    "name": "2",
                    "net": "00000000-0000-0000-0000-00000000c201",
                    "position": { "x": 7912500, "y": 10200000 },
                    "layer": 1,
                    "shape": "rect",
                    "diameter": 0,
                    "width": 1025000,
                    "height": 1400000
                },
                "00000000-0000-0000-0000-00000000c20c": {
                    "uuid": "00000000-0000-0000-0000-00000000c20c",
                    "package": "00000000-0000-0000-0000-00000000c209",
                    "name": "1",
                    "net": "00000000-0000-0000-0000-00000000c201",
                    "position": { "x": 12500000, "y": 10200000 },
                    "layer": 1,
                    "shape": "circle",
                    "diameter": 2800000,
                    "width": 0,
                    "height": 0,
                    "drill": 1400000
                }
            },
            "component_silkscreen": {
                "00000000-0000-0000-0000-00000000c203": [
                    {
                        "from": { "x": -2060000, "y": -2560000 },
                        "to": { "x": 2060000, "y": -2560000 },
                        "width_nm": 120000,
                        "layer": 1
                    },
                    {
                        "from": { "x": -2060000, "y": 2560000 },
                        "to": { "x": 2060000, "y": 2560000 },
                        "width_nm": 120000,
                        "layer": 1
                    },
                    {
                        "from": { "x": -2060000, "y": -2560000 },
                        "to": { "x": -2060000, "y": -2465000 },
                        "width_nm": 120000,
                        "layer": 1
                    },
                    {
                        "from": { "x": -2060000, "y": 2465000 },
                        "to": { "x": -2060000, "y": 2560000 },
                        "width_nm": 120000,
                        "layer": 1
                    },
                    {
                        "from": { "x": 2060000, "y": -2560000 },
                        "to": { "x": 2060000, "y": -2465000 },
                        "width_nm": 120000,
                        "layer": 1
                    },
                    {
                        "from": { "x": 2060000, "y": 2465000 },
                        "to": { "x": 2060000, "y": 2560000 },
                        "width_nm": 120000,
                        "layer": 1
                    }
                ],
                "00000000-0000-0000-0000-00000000c204": [
                    {
                        "from": { "x": -1380000, "y": -1380000 },
                        "to": { "x": 0, "y": -1380000 },
                        "width_nm": 120000,
                        "layer": 1
                    },
                    {
                        "from": { "x": -1380000, "y": 1270000 },
                        "to": { "x": -1380000, "y": 6460000 },
                        "width_nm": 120000,
                        "layer": 1
                    },
                    {
                        "from": { "x": -1380000, "y": 1270000 },
                        "to": { "x": 1380000, "y": 1270000 },
                        "width_nm": 120000,
                        "layer": 1
                    },
                    {
                        "from": { "x": -1380000, "y": 6460000 },
                        "to": { "x": 1380000, "y": 6460000 },
                        "width_nm": 120000,
                        "layer": 1
                    },
                    {
                        "from": { "x": 1380000, "y": 1270000 },
                        "to": { "x": 1380000, "y": 6460000 },
                        "width_nm": 120000,
                        "layer": 1
                    }
                ],
                "00000000-0000-0000-0000-00000000c208": [
                    {
                        "from": { "x": -227064, "y": -735000 },
                        "to": { "x": 227064, "y": -735000 },
                        "width_nm": 120000,
                        "layer": 1
                    },
                    {
                        "from": { "x": -227064, "y": 735000 },
                        "to": { "x": 227064, "y": 735000 },
                        "width_nm": 120000,
                        "layer": 1
                    }
                ],
                "00000000-0000-0000-0000-00000000c209": [
                    {
                        "from": { "x": -900000, "y": 2200000 },
                        "to": { "x": 900000, "y": 2200000 },
                        "width_nm": 120000,
                        "layer": 1
                    }
                ]
            },
            "component_silkscreen_arcs": {},
            "component_silkscreen_circles": {
                "00000000-0000-0000-0000-00000000c203": [
                    {
                        "center": { "x": -2600000, "y": -2470000 },
                        "radius_nm": 70000,
                        "width_nm": 120000,
                        "layer": 1
                    }
                ],
                "00000000-0000-0000-0000-00000000c209": [
                    {
                        "center": { "x": 0, "y": 0 },
                        "radius_nm": 1700000,
                        "width_nm": 120000,
                        "layer": 1
                    }
                ]
            },
            "component_silkscreen_polygons": {
                "00000000-0000-0000-0000-00000000c203": [
                    {
                        "vertices": [
                            { "x": -2600000, "y": -2470000 },
                            { "x": -2840000, "y": -2800000 },
                            { "x": -2360000, "y": -2800000 }
                        ],
                        "width_nm": 120000,
                        "layer": 1
                    }
                ]
            },
            "component_silkscreen_polylines": {
                "00000000-0000-0000-0000-00000000c208": []
            },
            "component_silkscreen_texts": {
                "00000000-0000-0000-0000-00000000c203": [
                    {
                        "text": "SRC",
                        "position": { "x": -220000, "y": -340000 },
                        "rotation": 0,
                        "height_nm": 160000,
                        "stroke_width_nm": 35000,
                        "layer": 1
                    }
                ],
                "00000000-0000-0000-0000-00000000c204": [
                    {
                        "text": "DST",
                        "position": { "x": -220000, "y": -360000 },
                        "rotation": 0,
                        "height_nm": 160000,
                        "stroke_width_nm": 35000,
                        "layer": 1
                    }
                ],
                "00000000-0000-0000-0000-00000000c208": [
                    {
                        "text": "R1",
                        "position": { "x": 0, "y": -1200000 },
                        "rotation": 0,
                        "height_nm": 180000,
                        "stroke_width_nm": 35000,
                        "layer": 1
                    }
                ],
                "00000000-0000-0000-0000-00000000c209": [
                    {
                        "text": "TP1",
                        "position": { "x": 0, "y": 2600000 },
                        "rotation": 0,
                        "height_nm": 180000,
                        "stroke_width_nm": 35000,
                        "layer": 1
                    }
                ]
            },
            "component_mechanical_lines": {},
            "component_mechanical_arcs": {},
            "component_mechanical_circles": {
                "00000000-0000-0000-0000-00000000c209": [
                    {
                        "center": { "x": 0, "y": 0 },
                        "radius_nm": 2000000,
                        "width_nm": 50000,
                        "layer": 41
                    }
                ]
            },
            "component_mechanical_texts": {},
            "component_mechanical_polylines": {},
            "component_mechanical_polygons": {
                "00000000-0000-0000-0000-00000000c203": [
                    {
                        "vertices": [
                            { "x": -3700000, "y": -2700000 },
                            { "x": -2200000, "y": -2700000 },
                            { "x": -2200000, "y": -2460000 },
                            { "x": 2200000, "y": -2460000 },
                            { "x": 2200000, "y": -2700000 },
                            { "x": 3700000, "y": -2700000 },
                            { "x": 3700000, "y": 2460000 },
                            { "x": 2200000, "y": 2460000 },
                            { "x": 2200000, "y": 2700000 },
                            { "x": -2200000, "y": 2700000 },
                            { "x": -2200000, "y": 2460000 },
                            { "x": -3700000, "y": 2460000 }
                        ],
                        "layer": 41
                    }
                ],
                "00000000-0000-0000-0000-00000000c204": [
                    {
                        "vertices": [
                            { "x": -1770000, "y": -1770000 },
                            { "x": 1770000, "y": -1770000 },
                            { "x": 1770000, "y": 6850000 },
                            { "x": -1770000, "y": 6850000 }
                        ],
                        "layer": 41
                    }
                ],
                "00000000-0000-0000-0000-00000000c208": [
                    {
                        "vertices": [
                            { "x": -1680000, "y": -950000 },
                            { "x": 1680000, "y": -950000 },
                            { "x": 1680000, "y": 950000 },
                            { "x": -1680000, "y": 950000 }
                        ],
                        "layer": 41
                    }
                ]
            },
            "tracks": {
                "00000000-0000-0000-0000-00000000c20d": {
                    "uuid": "00000000-0000-0000-0000-00000000c20d",
                    "net": "00000000-0000-0000-0000-00000000c201",
                    "from": { "x": 5200000, "y": 10200000 },
                    "to": { "x": 10200000, "y": 10200000 },
                    "width": 220000,
                    "layer": 1
                },
                "00000000-0000-0000-0000-00000000c20e": {
                    "uuid": "00000000-0000-0000-0000-00000000c20e",
                    "net": "00000000-0000-0000-0000-00000000c201",
                    "from": { "x": 10200000, "y": 10200000 },
                    "to": { "x": 12600000, "y": 8900000 },
                    "width": 220000,
                    "layer": 1
                },
                "00000000-0000-0000-0000-00000000c20f": {
                    "uuid": "00000000-0000-0000-0000-00000000c20f",
                    "net": "00000000-0000-0000-0000-00000000c201",
                    "from": { "x": 12600000, "y": 8900000 },
                    "to": { "x": 16000000, "y": 8900000 },
                    "width": 220000,
                    "layer": 1
                },
                "00000000-0000-0000-0000-00000000c216": {
                    "uuid": "00000000-0000-0000-0000-00000000c216",
                    "net": "00000000-0000-0000-0000-00000000c201",
                    "from": { "x": 12000000, "y": 2400000 },
                    "to": { "x": 12000000, "y": 5600000 },
                    "width": 320000,
                    "layer": 1
                },
                "00000000-0000-0000-0000-00000000c217": {
                    "uuid": "00000000-0000-0000-0000-00000000c217",
                    "net": "00000000-0000-0000-0000-00000000c201",
                    "from": { "x": 12000000, "y": 2400000 },
                    "to": { "x": 12000000, "y": 5600000 },
                    "width": 320000,
                    "layer": 3
                }
            },
            "vias": {
                "00000000-0000-0000-0000-00000000c210": {
                    "uuid": "00000000-0000-0000-0000-00000000c210",
                    "net": "00000000-0000-0000-0000-00000000c201",
                    "position": { "x": 10200000, "y": 10200000 },
                    "drill": 250000,
                    "diameter": 520000,
                    "from_layer": 1,
                    "to_layer": 3
                }
            },
            "zones": {
                "00000000-0000-0000-0000-00000000c211": {
                    "uuid": "00000000-0000-0000-0000-00000000c211",
                    "net": "00000000-0000-0000-0000-00000000c201",
                    "polygon": {
                        "vertices": [
                            { "x": 3500000, "y": 8000000 },
                            { "x": 22000000, "y": 8000000 },
                            { "x": 22000000, "y": 12600000 },
                            { "x": 3500000, "y": 12600000 }
                        ],
                        "closed": true
                    },
                    "layer": 1,
                    "priority": 1,
                    "thermal_relief": false,
                    "thermal_gap": 200000,
                    "thermal_spoke_width": 200000
                }
            },
            "nets": {
                "00000000-0000-0000-0000-00000000c200": {
                    "uuid": "00000000-0000-0000-0000-00000000c200",
                    "name": "SIG",
                    "class": "00000000-0000-0000-0000-00000000c202"
                },
                "00000000-0000-0000-0000-00000000c201": {
                    "uuid": "00000000-0000-0000-0000-00000000c201",
                    "name": "GND",
                    "class": "00000000-0000-0000-0000-00000000c202"
                }
            },
            "net_classes": {
                "00000000-0000-0000-0000-00000000c202": {
                    "uuid": "00000000-0000-0000-0000-00000000c202",
                    "name": "Default",
                    "clearance": 150000,
                    "track_width": 200000,
                    "via_drill": 300000,
                    "via_diameter": 600000,
                    "diffpair_width": 0,
                    "diffpair_gap": 0
                }
            },
            "keepouts": [],
            "dimensions": [],
            "texts": []
        }),
    )?;
    apply_kicad_reference_geometry(&board_dir.join("board.json"))?;
    Ok(())
}

fn write_json_file(path: &Path, value: &serde_json::Value) -> Result<()> {
    let payload = serde_json::to_string_pretty(value).context("failed to serialize demo JSON")?;
    std::fs::write(path, format!("{payload}\n"))
        .with_context(|| format!("failed to write {}", path.display()))
}

fn apply_kicad_reference_geometry(board_path: &Path) -> Result<()> {
    let mut board: Value = serde_json::from_str(
        &std::fs::read_to_string(board_path)
            .with_context(|| format!("failed to read {}", board_path.display()))?,
    )
    .context("failed to decode known-good board JSON for KiCad geometry patching")?;

    let specs = [
        (
            "00000000-0000-0000-0000-00000000c203",
            "U1",
            Path::new(
                "/usr/share/kicad/footprints/Package_SO.pretty/SOIC-8_3.9x4.9mm_P1.27mm.kicad_mod",
            ),
        ),
        (
            "00000000-0000-0000-0000-00000000c204",
            "J2",
            Path::new(
                "/usr/share/kicad/footprints/Connector_PinHeader_2.54mm.pretty/PinHeader_1x03_P2.54mm_Vertical.kicad_mod",
            ),
        ),
        (
            "00000000-0000-0000-0000-00000000c208",
            "R1",
            Path::new(
                "/usr/share/kicad/footprints/Resistor_SMD.pretty/R_0805_2012Metric.kicad_mod",
            ),
        ),
        (
            "00000000-0000-0000-0000-00000000c209",
            "TP1",
            Path::new(
                "/usr/share/kicad/footprints/TestPoint.pretty/TestPoint_Loop_D2.60mm_Drill1.4mm_Beaded.kicad_mod",
            ),
        ),
    ];

    for (component_uuid, reference, path) in specs {
        let geometry = load_kicad_demo_geometry(path, reference)?;
        replace_component_geometry(&mut board, component_uuid, &geometry)?;
    }

    write_json_file(board_path, &board)?;
    Ok(())
}

#[derive(Default)]
struct KicadDemoGeometry {
    silk_lines: Vec<Value>,
    silk_polylines: Vec<Value>,
    silk_circles: Vec<Value>,
    silk_polygons: Vec<Value>,
    silk_arcs: Vec<Value>,
    silk_texts: Vec<Value>,
    mechanical_lines: Vec<Value>,
    mechanical_polylines: Vec<Value>,
    mechanical_circles: Vec<Value>,
    mechanical_polygons: Vec<Value>,
    mechanical_arcs: Vec<Value>,
    mechanical_texts: Vec<Value>,
}

fn load_kicad_demo_geometry(path: &Path, reference: &str) -> Result<KicadDemoGeometry> {
    let (imported, _report) = eda_engine::import::kicad::import_footprint_document(path)
        .with_context(|| format!("failed to import KiCad footprint {}", path.display()))?;
    let mut out = KicadDemoGeometry::default();
    append_primitive_geometry(&mut out, &imported.package.silkscreen, true, reference);
    append_primitive_geometry(&mut out, &imported.mechanical, false, reference);
    if !imported.package.courtyard.vertices.is_empty() {
        out.mechanical_polygons.push(json!({
            "vertices": imported.package.courtyard.vertices.iter().map(|point| point_to_json(PointNm { x: point.x, y: point.y })).collect::<Vec<_>>(),
            "layer": 41
        }));
    }
    Ok(out)
}

fn replace_component_geometry(
    board: &mut Value,
    component_uuid: &str,
    geometry: &KicadDemoGeometry,
) -> Result<()> {
    replace_component_section(
        board,
        "component_silkscreen",
        component_uuid,
        &geometry.silk_lines,
    )?;
    replace_component_section(
        board,
        "component_silkscreen_polylines",
        component_uuid,
        &geometry.silk_polylines,
    )?;
    replace_component_section(
        board,
        "component_silkscreen_circles",
        component_uuid,
        &geometry.silk_circles,
    )?;
    replace_component_section(
        board,
        "component_silkscreen_polygons",
        component_uuid,
        &geometry.silk_polygons,
    )?;
    replace_component_section(
        board,
        "component_silkscreen_arcs",
        component_uuid,
        &geometry.silk_arcs,
    )?;
    replace_component_section(
        board,
        "component_silkscreen_texts",
        component_uuid,
        &geometry.silk_texts,
    )?;
    replace_component_section(
        board,
        "component_mechanical_lines",
        component_uuid,
        &geometry.mechanical_lines,
    )?;
    replace_component_section(
        board,
        "component_mechanical_polylines",
        component_uuid,
        &geometry.mechanical_polylines,
    )?;
    replace_component_section(
        board,
        "component_mechanical_circles",
        component_uuid,
        &geometry.mechanical_circles,
    )?;
    replace_component_section(
        board,
        "component_mechanical_polygons",
        component_uuid,
        &geometry.mechanical_polygons,
    )?;
    replace_component_section(
        board,
        "component_mechanical_arcs",
        component_uuid,
        &geometry.mechanical_arcs,
    )?;
    replace_component_section(
        board,
        "component_mechanical_texts",
        component_uuid,
        &geometry.mechanical_texts,
    )?;
    Ok(())
}

fn replace_component_section(
    board: &mut Value,
    key: &str,
    component_uuid: &str,
    values: &[Value],
) -> Result<()> {
    let section = board
        .get_mut(key)
        .and_then(Value::as_object_mut)
        .ok_or_else(|| anyhow::anyhow!("known-good board missing object section {key}"))?;
    section.insert(component_uuid.to_string(), Value::Array(values.to_vec()));
    Ok(())
}

fn append_primitive_geometry(
    out: &mut KicadDemoGeometry,
    primitives: &[eda_engine::pool::Primitive],
    silkscreen: bool,
    reference: &str,
) {
    for primitive in primitives {
        match primitive {
            eda_engine::pool::Primitive::Line { from, to, width } => {
                target_lines(out, silkscreen).push(json!({
                    "from": point_to_json(PointNm { x: from.x, y: from.y }),
                    "to": point_to_json(PointNm { x: to.x, y: to.y }),
                    "width_nm": *width,
                    "layer": if silkscreen { 1 } else { 41 }
                }));
            }
            eda_engine::pool::Primitive::Rect { min, max, width } => {
                target_polygons(out, silkscreen).push(json!({
                    "vertices": vec![
                        point_to_json(PointNm { x: min.x, y: min.y }),
                        point_to_json(PointNm { x: max.x, y: min.y }),
                        point_to_json(PointNm { x: max.x, y: max.y }),
                        point_to_json(PointNm { x: min.x, y: max.y }),
                    ],
                    "width_nm": *width,
                    "layer": if silkscreen { 1 } else { 41 }
                }));
            }
            eda_engine::pool::Primitive::Circle {
                center,
                radius,
                width,
            } => {
                target_circles(out, silkscreen).push(json!({
                    "center": point_to_json(PointNm { x: center.x, y: center.y }),
                    "radius_nm": *radius,
                    "width_nm": *width,
                    "layer": if silkscreen { 1 } else { 41 }
                }));
            }
            eda_engine::pool::Primitive::Polygon { polygon, width } => {
                target_polygons(out, silkscreen).push(json!({
                    "vertices": polygon.vertices.iter().map(|point| point_to_json(PointNm { x: point.x, y: point.y })).collect::<Vec<_>>(),
                    "width_nm": *width,
                    "layer": if silkscreen { 1 } else { 41 }
                }));
            }
            eda_engine::pool::Primitive::Arc { arc, width } => {
                target_arcs(out, silkscreen).push(json!({
                    "center": point_to_json(PointNm { x: arc.center.x, y: arc.center.y }),
                    "radius_nm": arc.radius,
                    "start_angle": arc.start_angle,
                    "end_angle": arc.end_angle,
                    "width_nm": *width,
                    "layer": if silkscreen { 1 } else { 41 }
                }));
            }
            eda_engine::pool::Primitive::Text {
                text,
                position,
                rotation,
            } => {
                let normalized = normalize_reference_text(text, reference);
                if normalized != reference {
                    continue;
                }
                target_texts(out, silkscreen).push(json!({
                    "text": normalized,
                    "position": point_to_json(PointNm { x: position.x, y: position.y }),
                    "rotation": *rotation,
                    "height_nm": 1_000_000,
                    "stroke_width_nm": 150_000,
                    "layer": if silkscreen { 1 } else { 41 }
                }));
            }
        }
    }
}

fn normalize_reference_text(text: &str, reference: &str) -> String {
    if text.contains("REF")
        || text.contains("Reference")
        || text.contains('?')
        || text.contains("${REFERENCE}")
    {
        reference.to_string()
    } else {
        text.to_string()
    }
}

fn overlay_path_for_action(
    action_index: usize,
    action: &RouteProposalActionPayload,
    review: &RouteProposalReviewPayload,
    selected_path_points: Option<&[PointNm]>,
) -> Vec<PointNm> {
    if let Some(points) = selected_path_points
        && review.proposal_actions.len() > 1
        && points.len() >= review.proposal_actions.len() + 1
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

fn target_lines(out: &mut KicadDemoGeometry, silkscreen: bool) -> &mut Vec<Value> {
    if silkscreen {
        &mut out.silk_lines
    } else {
        &mut out.mechanical_lines
    }
}

fn target_circles(out: &mut KicadDemoGeometry, silkscreen: bool) -> &mut Vec<Value> {
    if silkscreen {
        &mut out.silk_circles
    } else {
        &mut out.mechanical_circles
    }
}

fn target_polygons(out: &mut KicadDemoGeometry, silkscreen: bool) -> &mut Vec<Value> {
    if silkscreen {
        &mut out.silk_polygons
    } else {
        &mut out.mechanical_polygons
    }
}

fn target_arcs(out: &mut KicadDemoGeometry, silkscreen: bool) -> &mut Vec<Value> {
    if silkscreen {
        &mut out.silk_arcs
    } else {
        &mut out.mechanical_arcs
    }
}

fn target_texts(out: &mut KicadDemoGeometry, silkscreen: bool) -> &mut Vec<Value> {
    if silkscreen {
        &mut out.silk_texts
    } else {
        &mut out.mechanical_texts
    }
}

fn point_to_json(point: PointNm) -> Value {
    json!({ "x": point.x, "y": point.y })
}

impl ReviewWorkspaceState {
    pub fn new(scene: BoardReviewSceneV1, review: RouteProposalReviewPayload) -> Self {
        let layer_visibility = scene
            .layers
            .iter()
            .map(|layer| (layer.layer_id.clone(), layer.visible_by_default))
            .collect();
        let has_review_actions = !review.proposal_actions.is_empty();
        let active_review_target_id = review
            .proposal_actions
            .first()
            .map(|action| action.action_id.clone())
            .unwrap_or_else(|| "no-proposal-action".to_string());
        Self {
            scene,
            review,
            production: ProductionStatus::default(),
            checks: CheckRunReviewState::default(),
            selection: if has_review_actions {
                SelectionTarget::ReviewAction(active_review_target_id.clone())
            } else {
                SelectionTarget::None
            },
            active_review_target_id,
            tool: WorkspaceTool::Select,
            backing: None,
            last_command_status: None,
            ui: WorkspaceUiState {
                active_dock_tab: None,
                dock_height_px: 220,
                hovered_object_id: None,
                filters: WorkspaceFilterState {
                    show_authored: true,
                    show_proposed: true,
                    show_unrouted: true,
                    dim_unrelated: has_review_actions,
                    layer_visibility,
                },
                terminal: TerminalLaneState {
                    lines: vec![
                        "datum terminal ready".to_string(),
                        "shell session starts in the active project root".to_string(),
                    ],
                    activity_summary: Vec::new(),
                    input: String::new(),
                    cursor: 0,
                    scroll_offset: 0,
                    status: "running".to_string(),
                },
                assistant: AssistantLaneState {
                    transcript: vec![AssistantMessage {
                        role: "assistant".to_string(),
                        content:
                            "assistant lane ready; use /activity, /config status, or /config api-key <key>"
                                .to_string(),
                    }],
                    input: String::new(),
                    cursor: 0,
                    awaiting_api_key: false,
                    scroll_offset: 0,
                },
                artifact_preview: ArtifactPreviewViewportState::default(),
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
        true
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
    state.production = load_production_status(&cli, request)?;
    state.checks = load_check_run_review_state(&cli, request)?;
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
    ) {
        if let Some(state) = check_runs::check_run_review_state_from_context_value(&context) {
            return Ok(state);
        }
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

fn focused_artifact_id(status: &ProductionStatus) -> Option<String> {
    status
        .output_jobs
        .iter()
        .find_map(|job| {
            job.artifacts
                .first()
                .map(|artifact| artifact.artifact_id.clone())
        })
        .or_else(|| {
            status
                .artifact_runs
                .last()
                .map(|run| run.artifact_id.clone())
        })
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
        output_jobs,
        artifact_runs,
        proposals,
        manufacturing_plans,
        panel_projections,
        focused_artifact: None,
    }
}

/// Load a KiCad .kicad_pcb board via the engine import path.
fn load_scene_from_kicad_import(board_file: &Path) -> Result<(BoardReviewSceneV1, PathBuf)> {
    let import_started = std::time::Instant::now();
    let engine_started = std::time::Instant::now();
    let mut engine =
        eda_engine::api::Engine::new().map_err(|e| anyhow::anyhow!("engine init: {e}"))?;
    trace_protocol_timing(format!(
        "kicad engine init {}ms",
        engine_started.elapsed().as_millis()
    ));
    let engine_import_started = std::time::Instant::now();
    let import_report = engine
        .import(board_file)
        .map_err(|e| anyhow::anyhow!("import {}: {e}", board_file.display()))?;
    // Import warnings are fidelity signals (dropped objects, accounting
    // mismatches). They must surface, not vanish with the report.
    for warning in &import_report.warnings {
        eprintln!("datum-import warning [{}]: {warning}", board_file.display());
    }
    trace_protocol_timing(format!(
        "kicad engine import {}ms warnings={}",
        engine_import_started.elapsed().as_millis(),
        import_report.warnings.len()
    ));
    let board_borrow_started = std::time::Instant::now();
    let board = engine
        .board()
        .map_err(|e| anyhow::anyhow!("no board after import: {e}"))?;
    trace_protocol_timing(format!(
        "kicad board borrow {}ms",
        board_borrow_started.elapsed().as_millis()
    ));

    let board_uuid = board.uuid.to_string();
    let project_name = board_file
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "imported".to_string());

    let stackup_started = std::time::Instant::now();
    let stackup = engine
        .get_stackup()
        .map_err(|e| anyhow::anyhow!("stackup: {e}"))?;
    trace_protocol_timing(format!(
        "kicad stackup {}ms",
        stackup_started.elapsed().as_millis()
    ));
    let layer_name_map: std::collections::HashMap<i32, String> = stackup
        .layers
        .iter()
        .map(|l| (l.id, l.name.clone()))
        .collect();
    let _layer_name = |id: i32| -> String {
        layer_name_map
            .get(&id)
            .cloned()
            .unwrap_or_else(|| format!("L{}", id))
    };
    let components_started = std::time::Instant::now();
    let components = engine
        .get_components()
        .map_err(|e| anyhow::anyhow!("components: {e}"))?;
    trace_protocol_timing(format!(
        "kicad components {}ms count={}",
        components_started.elapsed().as_millis(),
        components.len()
    ));

    // Re-borrow board after the method calls above (they borrow &self temporarily).
    let board_reborrow_started = std::time::Instant::now();
    let board = engine.board().map_err(|e| anyhow::anyhow!("board: {e}"))?;
    trace_protocol_timing(format!(
        "kicad board reborrow {}ms",
        board_reborrow_started.elapsed().as_millis()
    ));

    let payload_started = std::time::Instant::now();
    let outline_vertices: Vec<PointNm> = board
        .outline
        .vertices
        .iter()
        .map(|p| PointNm { x: p.x, y: p.y })
        .collect();

    let outline_payload = OutlinePayload {
        vertices: outline_vertices,
        closed: !board.outline.vertices.is_empty(),
    };
    let pad_expansion_setup = ScenePadExpansionSetup {
        pad_to_mask_clearance_nm: board.pad_expansion_setup.pad_to_mask_clearance_nm,
        pad_to_paste_clearance_nm: board.pad_expansion_setup.pad_to_paste_clearance_nm,
        pad_to_paste_ratio_ppm: board.pad_expansion_setup.pad_to_paste_ratio_ppm,
        solder_mask_min_width_nm: board.pad_expansion_setup.solder_mask_min_width_nm,
    };

    let component_payloads: Vec<BoardComponentPayload> = components
        .iter()
        .map(|c| BoardComponentPayload {
            uuid: c.uuid.to_string(),
            reference: c.reference.clone(),
            value: c.value.clone(),
            position: PointNm {
                x: c.position.x,
                y: c.position.y,
            },
            rotation: c.rotation,
            layer: c.layer,
            locked: c.locked,
        })
        .collect();

    let pad_payloads: Vec<BoardPadPayload> = board
        .pads
        .values()
        .map(|p| {
            let shape_str = match p.shape {
                eda_engine::board::PadShape::Circle => "circle",
                eda_engine::board::PadShape::Rect => "rect",
                eda_engine::board::PadShape::Oval => "oval",
                eda_engine::board::PadShape::RoundRect => "roundrect",
            };
            BoardPadPayload {
                uuid: p.uuid.to_string(),
                package: p.package.to_string(),
                name: p.name.clone(),
                net: p.net.map(|n| n.to_string()),
                position: PointNm {
                    x: p.position.x,
                    y: p.position.y,
                },
                layer: p.layer,
                copper_layers: p.copper_layers.clone(),
                shape: shape_str.to_string(),
                diameter: p.diameter,
                width: p.width,
                height: p.height,
                roundrect_rratio_ppm: p.roundrect_rratio_ppm,
                mask_layers: p.mask_layers.clone(),
                paste_layers: p.paste_layers.clone(),
                solder_mask_margin_nm: p.solder_mask_margin_nm,
                solder_paste_margin_nm: p.solder_paste_margin_nm,
                solder_paste_margin_ratio_ppm: p.solder_paste_margin_ratio_ppm,
                drill: if p.drill > 0 { Some(p.drill) } else { None },
                rotation: p.rotation,
            }
        })
        .collect();

    let track_payloads: Vec<BoardTrackPayload> = board
        .tracks
        .values()
        .map(|t| BoardTrackPayload {
            uuid: t.uuid.to_string(),
            net: t.net.to_string(),
            from: PointNm {
                x: t.from.x,
                y: t.from.y,
            },
            to: PointNm {
                x: t.to.x,
                y: t.to.y,
            },
            width: t.width,
            layer: t.layer,
        })
        .collect();

    let via_payloads: Vec<BoardViaPayload> = board
        .vias
        .values()
        .map(|v| BoardViaPayload {
            uuid: v.uuid.to_string(),
            net: v.net.to_string(),
            position: PointNm {
                x: v.position.x,
                y: v.position.y,
            },
            drill: v.drill,
            diameter: v.diameter,
            from_layer: v.from_layer,
            to_layer: v.to_layer,
        })
        .collect();

    let zone_payloads: Vec<BoardZonePayload> = board
        .zones
        .values()
        .map(|z| BoardZonePayload {
            uuid: z.uuid.to_string(),
            net: z.net.to_string(),
            layer: z.layer,
            polygon: OutlinePayload {
                vertices: z
                    .polygon
                    .vertices
                    .iter()
                    .map(|p| PointNm { x: p.x, y: p.y })
                    .collect(),
                closed: true,
            },
        })
        .collect();
    let unrouted_primitives = unrouted_primitives_from_airwires(&board.unrouted());
    let net_display = net_display_from_imported_board(board);
    trace_protocol_timing(format!(
        "kicad payload build {}ms components={} pads={} tracks={} vias={} zones={}",
        payload_started.elapsed().as_millis(),
        component_payloads.len(),
        pad_payloads.len(),
        track_payloads.len(),
        via_payloads.len(),
        zone_payloads.len()
    ));

    let inspect = ProjectInspectPayload {
        project_root: board_file
            .parent()
            .unwrap_or(Path::new("."))
            .display()
            .to_string(),
        project_name,
        project_uuid: board_uuid.clone(),
        board_uuid,
        board_path: board_file.display().to_string(),
    };

    // --- Footprint graphics (silkscreen, fab, courtyard) + board-level
    // Edge.Cuts authored graphics (M7-SCN-007 Option B). Resolve Edge.Cuts to
    // its numeric id from the PCB's own layer table so the scene-level
    // `L{n}` key matches the visibility map for both the outline primitive
    // and the authored board_graphics primitives. KiCad 7 canonically uses
    // id 44; KiCad 9 may renumber — DOA2526 uses id 25 for Edge.Cuts.
    let (
        kicad_graphics,
        kicad_texts,
        mut imported_board_texts,
        mut imported_board_text_geometries,
        mut imported_glyph_mesh_assets,
        board_graphics,
        mut imported_gr_texts,
        mut imported_gr_text_geometries,
        imported_gr_glyph_mesh_assets,
        edge_cuts_layer_key,
    ) = {
        let direct_parse_started = std::time::Instant::now();
        let read_started = std::time::Instant::now();
        let contents = std::fs::read_to_string(board_file)
            .with_context(|| format!("failed to read {}", board_file.display()))?;
        trace_protocol_timing(format!(
            "kicad direct read {}ms bytes={}",
            read_started.elapsed().as_millis(),
            contents.len()
        ));
        let layer_table_started = std::time::Instant::now();
        let layer_table = kicad_parse_layer_table(&contents);
        trace_protocol_timing(format!(
            "kicad layer table parse {}ms layers={}",
            layer_table_started.elapsed().as_millis(),
            layer_table.len()
        ));
        let edge_cuts_key = layer_table
            .get("Edge.Cuts")
            .copied()
            .map(layer_id)
            .unwrap_or_else(|| layer_id(44));
        let footprint_parse_started = std::time::Instant::now();
        let (g, t, bt, btg, gma) =
            extract_kicad_footprint_graphics(&contents, &component_payloads, &layer_table);
        trace_protocol_timing(format!(
            "kicad footprint graphics/text parse {}ms graphics={} texts={} board_texts={} geometries={} glyph_assets={}",
            footprint_parse_started.elapsed().as_millis(),
            g.len(),
            t.len(),
            bt.len(),
            btg.len(),
            gma.len()
        ));
        let board_graphics_started = std::time::Instant::now();
        let bg = extract_kicad_board_graphics(&contents, &inspect.board_uuid, &layer_table);
        trace_protocol_timing(format!(
            "kicad board graphics parse {}ms graphics={}",
            board_graphics_started.elapsed().as_millis(),
            bg.len()
        ));
        let board_text_started = std::time::Instant::now();
        let (gr_texts, gr_geometries, gr_assets) =
            extract_kicad_board_texts(&contents, &layer_table);
        trace_protocol_timing(format!(
            "kicad board text parse {}ms texts={} geometries={} glyph_assets={}",
            board_text_started.elapsed().as_millis(),
            gr_texts.len(),
            gr_geometries.len(),
            gr_assets.len()
        ));
        trace_protocol_timing(format!(
            "kicad direct parse total {}ms",
            direct_parse_started.elapsed().as_millis()
        ));
        (
            g,
            t,
            bt,
            btg,
            gma,
            bg,
            gr_texts,
            gr_geometries,
            gr_assets,
            edge_cuts_key,
        )
    };
    let merge_started = std::time::Instant::now();
    imported_board_texts.append(&mut imported_gr_texts);
    imported_board_text_geometries.append(&mut imported_gr_text_geometries);
    merge_glyph_mesh_assets(
        &mut imported_glyph_mesh_assets,
        imported_gr_glyph_mesh_assets,
    );
    trace_protocol_timing(format!(
        "kicad text merge {}ms board_texts={} geometries={} glyph_assets={}",
        merge_started.elapsed().as_millis(),
        imported_board_texts.len(),
        imported_board_text_geometries.len(),
        imported_glyph_mesh_assets.len()
    ));
    let scene_build_started = std::time::Instant::now();
    let mut scene = build_board_review_scene(
        &inspect,
        outline_payload,
        component_payloads,
        kicad_graphics,
        kicad_texts,
        pad_expansion_setup,
        pad_payloads,
        track_payloads,
        via_payloads,
        zone_payloads,
        board_graphics,
        imported_board_texts,
        imported_board_text_geometries,
        imported_glyph_mesh_assets,
        unrouted_primitives,
        net_display,
        edge_cuts_layer_key,
    );
    trace_protocol_timing(format!(
        "kicad scene build {}ms",
        scene_build_started.elapsed().as_millis()
    ));
    // Replace auto-generated L0/L31 layers with real stackup names
    let layer_replace_started = std::time::Instant::now();
    scene.layers = stackup
        .layers
        .iter()
        .enumerate()
        .map(|(i, l)| SceneLayer {
            layer_id: layer_id(l.id),
            name: l.name.clone(),
            kind: match l.layer_type {
                eda_engine::board::StackupLayerType::Copper => "copper",
                eda_engine::board::StackupLayerType::Silkscreen => "silkscreen",
                eda_engine::board::StackupLayerType::SolderMask => "mask",
                eda_engine::board::StackupLayerType::Paste => "paste",
                eda_engine::board::StackupLayerType::Mechanical => "mechanical",
                eda_engine::board::StackupLayerType::Dielectric => "dielectric",
            }
            .to_string(),
            render_order: i as u32,
            visible_by_default: matches!(l.layer_type, eda_engine::board::StackupLayerType::Copper)
                || l.name.ends_with(".Cu")
                || l.name == "F.Cu"
                || l.name == "B.Cu"
                || l.name == "Edge.Cuts"
                || l.name == "F.SilkS",
        })
        .collect();
    trace_protocol_timing(format!(
        "kicad layer replace {}ms scene_layers={}",
        layer_replace_started.elapsed().as_millis(),
        scene.layers.len()
    ));
    trace_protocol_timing(format!(
        "kicad scene import total {}ms",
        import_started.elapsed().as_millis()
    ));
    Ok((scene, board_file.to_path_buf()))
}

// ---------------------------------------------------------------------------
// KiCad footprint graphics extraction (direct file parsing)
// ---------------------------------------------------------------------------

/// Parse the `(layers ...)` section from a KiCad PCB file to build a
/// layer-name to numeric-id map.
fn kicad_parse_layer_table(contents: &str) -> std::collections::HashMap<String, i32> {
    let mut map = std::collections::HashMap::new();
    let start = match contents.find("(layers") {
        Some(s) => s,
        None => return map,
    };
    let rest = &contents[start..];
    // Walk until balanced parens close the (layers ...) block.
    let mut depth: i32 = 0;
    let mut block_end = rest.len();
    for (i, ch) in rest.char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    block_end = i + 1;
                    break;
                }
            }
            _ => {}
        }
    }
    let block = &rest[..block_end];
    for line in block.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('(') && !trimmed.starts_with("(layers") {
            let inner = trimmed.trim_start_matches('(').trim_end_matches(')');
            let mut parts = inner.split_whitespace();
            if let Some(id_str) = parts.next() {
                if let Ok(id) = id_str.parse::<i32>() {
                    if let Some(name) = parts.next() {
                        let name = canonicalize_kicad_layer_name(name.trim_matches('"'));
                        map.insert(name.to_string(), id);
                    }
                }
            }
        }
    }
    map
}

fn canonicalize_kicad_layer_name(name: &str) -> String {
    match name.to_ascii_lowercase().as_str() {
        "f.cu" => "F.Cu".to_string(),
        "b.cu" => "B.Cu".to_string(),
        "b.adhes" => "B.Adhes".to_string(),
        "f.adhes" => "F.Adhes".to_string(),
        "b.paste" => "B.Paste".to_string(),
        "f.paste" => "F.Paste".to_string(),
        "b.silks" => "B.SilkS".to_string(),
        "f.silks" => "F.SilkS".to_string(),
        "b.mask" => "B.Mask".to_string(),
        "f.mask" => "F.Mask".to_string(),
        "dwgs.user" => "Dwgs.User".to_string(),
        "cmts.user" => "Cmts.User".to_string(),
        "eco1.user" => "Eco1.User".to_string(),
        "eco2.user" => "Eco2.User".to_string(),
        "edge.cuts" => "Edge.Cuts".to_string(),
        "margin" => "Margin".to_string(),
        "b.crtyd" => "B.CrtYd".to_string(),
        "f.crtyd" => "F.CrtYd".to_string(),
        "b.fab" => "B.Fab".to_string(),
        "f.fab" => "F.Fab".to_string(),
        _ => name.to_string(),
    }
}

fn kicad_resolve_layer_id(name: &str, table: &std::collections::HashMap<String, i32>) -> i32 {
    let canonical_name = canonicalize_kicad_layer_name(name);
    if let Some(&id) = table.get(&canonical_name) {
        return id;
    }
    // Hardcoded fallbacks for common layers.
    match canonical_name.as_str() {
        "F.Cu" => 0,
        "B.Cu" => 31,
        "B.SilkS" => 36,
        "F.SilkS" => 37,
        "B.Fab" => 35,
        "F.Fab" => 38,
        "B.CrtYd" => 34,
        "F.CrtYd" => 39,
        "Edge.Cuts" => 44,
        _ => 0,
    }
}

fn kicad_render_role(layer_name: &str) -> Option<&'static str> {
    match canonicalize_kicad_layer_name(layer_name).as_str() {
        "F.SilkS" | "B.SilkS" => Some("component_silkscreen"),
        "F.CrtYd" | "B.CrtYd" | "F.Fab" | "B.Fab" => Some("component_mechanical"),
        _ => None,
    }
}

#[derive(Default)]
struct KicadImportTextTrace {
    fp_text_total: usize,
    fp_text_template_skipped: usize,
    fp_text_hidden_skipped: usize,
    fp_text_imported: usize,
    property_total: usize,
    property_metadata_skipped: usize,
    property_empty_skipped: usize,
    property_hidden_skipped: usize,
    property_reference_imported: usize,
    property_value_imported: usize,
    gr_text_total: usize,
    gr_text_hidden_skipped: usize,
    gr_text_imported: usize,
    by_kind: BTreeMap<String, usize>,
    by_layer: BTreeMap<String, usize>,
    samples: Vec<String>,
}

impl KicadImportTextTrace {
    fn record_import(&mut self, kind: &str, layer_name: &str, layer: i32, text: &str) {
        *self.by_kind.entry(kind.to_string()).or_insert(0) += 1;
        *self
            .by_layer
            .entry(format!(
                "{}:{}",
                canonicalize_kicad_layer_name(layer_name),
                layer_id(layer)
            ))
            .or_insert(0) += 1;
        if self.samples.len() < 16 {
            self.samples.push(format!(
                "{}:{}:{}",
                kind,
                canonicalize_kicad_layer_name(layer_name),
                text
            ));
        }
    }

    fn emit(&self, scope: &str, board_texts: usize, geometries: usize, glyph_assets: usize) {
        if !kicad_import_text_trace_enabled() {
            return;
        }
        eprintln!(
            "[datum-import-text] {scope} fp_text total={} imported={} skipped_template={} skipped_hidden={} property total={} ref={} value={} skipped_metadata={} skipped_empty={} skipped_hidden={} gr_text total={} imported={} skipped_hidden={} board_texts={} geometries={} glyph_assets={} by_kind={:?} by_layer={:?} samples={:?}",
            self.fp_text_total,
            self.fp_text_imported,
            self.fp_text_template_skipped,
            self.fp_text_hidden_skipped,
            self.property_total,
            self.property_reference_imported,
            self.property_value_imported,
            self.property_metadata_skipped,
            self.property_empty_skipped,
            self.property_hidden_skipped,
            self.gr_text_total,
            self.gr_text_imported,
            self.gr_text_hidden_skipped,
            board_texts,
            geometries,
            glyph_assets,
            self.by_kind,
            self.by_layer,
            self.samples,
        );
    }
}

fn kicad_import_text_trace_enabled() -> bool {
    std::env::var_os("DATUM_TRACE_IMPORT_TEXT").is_some()
        || std::env::var_os("DATUM_TRACE_TIMING").is_some()
}

fn trace_protocol_timing(message: String) {
    if std::env::var_os("DATUM_TRACE_TIMING").is_some() {
        eprintln!("[datum-protocol] {message}");
    }
}

fn kicad_block_hidden(block: &str) -> bool {
    block.contains("(hide yes)")
}

/// Convert mm to nm.
fn kicad_mm_to_nm(mm: f64) -> i64 {
    (mm * 1_000_000.0).round() as i64
}

/// Parse a `(form x y ...)` anywhere in a line and return the (x, y) in nm.
fn kicad_parse_xy_anywhere(line: &str, form: &str) -> Option<PointNm> {
    let marker = format!("({form} ");
    let start = line.find(&marker)? + marker.len();
    let rest = &line[start..];
    let end = rest.find(')').unwrap_or(rest.len());
    let mut parts = rest[..end].split_whitespace();
    let x = parts.next()?.parse::<f64>().ok()?;
    let y = parts.next()?.parse::<f64>().ok()?;
    Some(PointNm {
        x: kicad_mm_to_nm(x),
        y: kicad_mm_to_nm(y),
    })
}

/// Parse the stroke/line width from a KiCad block.
/// Handles both old-style `(width 0.12)` and new-style `(stroke (width 0.12) ...)`.
fn kicad_parse_width_nm(block: &str) -> i64 {
    // Try `(stroke (width N) ...)` first (KiCad 7+).
    for line in block.lines() {
        let trimmed = line.trim();
        if let Some(pos) = trimmed.find("(stroke ") {
            let rest = &trimmed[pos..];
            if let Some(w_pos) = rest.find("(width ") {
                let after = &rest[w_pos + "(width ".len()..];
                let end = after.find(')').unwrap_or(after.len());
                if let Ok(mm) = after[..end].trim().parse::<f64>() {
                    return kicad_mm_to_nm(mm);
                }
            }
        }
    }
    // Fall back to top-level `(width N)`.
    for line in block.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("(width ") {
            let rest = trimmed.trim_start_matches("(width ").trim_end_matches(')');
            if let Ok(mm) = rest.split_whitespace().next().unwrap_or("").parse::<f64>() {
                return kicad_mm_to_nm(mm);
            }
        }
    }
    120_000 // default 0.12mm
}

/// Parse a `(layer "Name")` from anywhere in a block line.
fn kicad_parse_layer_anywhere(block: &str) -> Option<String> {
    block.lines().find_map(|line| {
        let trimmed = line.trim();
        let start = trimmed.find("(layer ")? + "(layer ".len();
        let rest = &trimmed[start..];
        // Quoted name
        if rest.starts_with('"') {
            let inner = &rest[1..];
            let end = inner.find('"')?;
            Some(canonicalize_kicad_layer_name(&inner[..end]))
        } else {
            let end = rest.find(')')?;
            Some(canonicalize_kicad_layer_name(rest[..end].trim()))
        }
    })
}

/// Parse a `(uuid "...")` from a block.
fn kicad_parse_uuid(block: &str) -> Option<String> {
    block.lines().find_map(|line| {
        let trimmed = line.trim();
        let start = trimmed.find("(uuid ")? + "(uuid ".len();
        let rest = &trimmed[start..];
        if rest.starts_with('"') {
            let inner = &rest[1..];
            let end = inner.find('"')?;
            Some(inner[..end].to_string())
        } else {
            let end = rest.find(')')?;
            Some(rest[..end].trim().to_string())
        }
    })
}

/// Parse `(at x y [rotation])` from a block's first `(at ...)` line.
fn kicad_parse_at(block: &str) -> Option<(PointNm, i32)> {
    let line = block.lines().find(|l| l.trim().contains("(at "))?;
    let trimmed = line.trim();
    let start = trimmed.find("(at ")? + "(at ".len();
    let rest = &trimmed[start..];
    let end = rest.find(')').unwrap_or(rest.len());
    let mut parts = rest[..end].split_whitespace();
    let x = parts.next()?.parse::<f64>().ok()?;
    let y = parts.next()?.parse::<f64>().ok()?;
    let rotation = parts
        .next()
        .and_then(|s| s.parse::<f64>().ok())
        .map(|r| r.round() as i32)
        .unwrap_or(0);
    Some((
        PointNm {
            x: kicad_mm_to_nm(x),
            y: kicad_mm_to_nm(y),
        },
        rotation,
    ))
}

/// Parse `(xy x y)` points from a block (used for polygons).
fn kicad_parse_xy_points(block: &str) -> Vec<PointNm> {
    let mut points = Vec::new();
    let mut rest = block;
    let marker = "(xy ";
    while let Some(start) = rest.find(marker) {
        let after = &rest[start + marker.len()..];
        let Some(end) = after.find(')') else { break };
        let mut parts = after[..end].split_whitespace();
        if let (Some(x), Some(y)) = (
            parts.next().and_then(|v| v.parse::<f64>().ok()),
            parts.next().and_then(|v| v.parse::<f64>().ok()),
        ) {
            points.push(PointNm {
                x: kicad_mm_to_nm(x),
                y: kicad_mm_to_nm(y),
            });
        }
        rest = &after[end + 1..];
    }
    points
}

/// Extract nested s-expression blocks for a given form within a parent block.
fn kicad_nested_blocks(contents: &str, form: &str) -> Vec<String> {
    let mut blocks = Vec::new();
    let mut current = Vec::new();
    let mut capturing = false;
    let mut depth: i32 = 0;
    let prefix = format!("({form}");

    for line in contents.lines() {
        let trimmed = line.trim_start();

        if !capturing
            && trimmed.starts_with(&prefix)
            && matches!(
                trimmed.as_bytes().get(prefix.len()),
                Some(b' ') | Some(b'\t') | Some(b')') | None
            )
        {
            capturing = true;
            current.clear();
            depth = 0;
        }

        if capturing {
            current.push(line.to_string());
            let opens = line.chars().filter(|c| *c == '(').count() as i32;
            let closes = line.chars().filter(|c| *c == ')').count() as i32;
            depth += opens - closes;
            if depth <= 0 {
                blocks.push(current.join("\n"));
                current.clear();
                capturing = false;
            }
        }
    }
    blocks
}

/// Extract nested s-expression blocks for several forms with one parent scan.
fn kicad_nested_blocks_by_form(contents: &str, forms: &[&str]) -> BTreeMap<String, Vec<String>> {
    let mut blocks = forms
        .iter()
        .map(|form| ((*form).to_string(), Vec::new()))
        .collect::<BTreeMap<_, _>>();
    let prefixes = forms
        .iter()
        .map(|form| ((*form).to_string(), format!("({form}")))
        .collect::<Vec<_>>();
    let mut current = Vec::new();
    let mut capturing_form: Option<String> = None;
    let mut depth: i32 = 0;

    for line in contents.lines() {
        let trimmed = line.trim_start();

        if capturing_form.is_none() {
            if let Some((form, _)) = prefixes.iter().find(|(_, prefix)| {
                trimmed.starts_with(prefix)
                    && matches!(
                        trimmed.as_bytes().get(prefix.len()),
                        Some(b' ') | Some(b'\t') | Some(b')') | None
                    )
            }) {
                capturing_form = Some(form.clone());
                current.clear();
                depth = 0;
            }
        }

        if let Some(form) = capturing_form.as_ref() {
            current.push(line.to_string());
            let opens = line.chars().filter(|c| *c == '(').count() as i32;
            let closes = line.chars().filter(|c| *c == ')').count() as i32;
            depth += opens - closes;
            if depth <= 0 {
                blocks
                    .entry(form.clone())
                    .or_default()
                    .push(current.join("\n"));
                current.clear();
                capturing_form = None;
            }
        }
    }

    blocks
}

/// Compute arc center, radius, start_angle_tenths, end_angle_tenths from three
/// points (start, mid, end), all in nm. Returns None for collinear points.
fn kicad_arc_from_three_points(
    start: &PointNm,
    mid: &PointNm,
    end: &PointNm,
) -> Option<(PointNm, i64, i32, i32)> {
    let (x1, y1) = (start.x as f64, start.y as f64);
    let (x2, y2) = (mid.x as f64, mid.y as f64);
    let (x3, y3) = (end.x as f64, end.y as f64);
    let d = 2.0 * (x1 * (y2 - y3) + x2 * (y3 - y1) + x3 * (y1 - y2));
    if d.abs() < f64::EPSILON {
        return None;
    }
    let ux = ((x1 * x1 + y1 * y1) * (y2 - y3)
        + (x2 * x2 + y2 * y2) * (y3 - y1)
        + (x3 * x3 + y3 * y3) * (y1 - y2))
        / d;
    let uy = ((x1 * x1 + y1 * y1) * (x3 - x2)
        + (x2 * x2 + y2 * y2) * (x1 - x3)
        + (x3 * x3 + y3 * y3) * (x2 - x1))
        / d;
    let center = PointNm {
        x: ux.round() as i64,
        y: uy.round() as i64,
    };
    let radius = ((x1 - ux).powi(2) + (y1 - uy).powi(2)).sqrt().round() as i64;
    let start_angle =
        (((y1 - uy).atan2(x1 - ux).to_degrees() * 10.0).round() as i32).rem_euclid(3600);
    let end_angle =
        (((y3 - uy).atan2(x3 - ux).to_degrees() * 10.0).round() as i32).rem_euclid(3600);
    Some((center, radius, start_angle, end_angle))
}

/// Parse font size from `(effects (font (size H W) ...))`.
fn kicad_parse_font_height_nm(block: &str) -> i64 {
    for line in block.lines() {
        let trimmed = line.trim();
        if let Some(pos) = trimmed.find("(size ") {
            let rest = &trimmed[pos + "(size ".len()..];
            let end = rest.find(')').unwrap_or(rest.len());
            let mut parts = rest[..end].split_whitespace();
            if let Some(h) = parts.next().and_then(|v| v.parse::<f64>().ok()) {
                return kicad_mm_to_nm(h);
            }
        }
    }
    1_000_000 // default 1mm
}

fn kicad_parse_font_thickness_nm(block: &str) -> Option<i64> {
    for line in block.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with("(thickness ") {
            continue;
        }
        let inner = trimmed
            .trim_start_matches("(thickness ")
            .trim_end_matches(')');
        if let Ok(mm) = inner.trim().parse::<f64>() {
            return Some(kicad_mm_to_nm(mm));
        }
    }
    None
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum KicadTextHJustify {
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum KicadTextVJustify {
    Top,
    Center,
    Bottom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct KicadTextJustify {
    h: KicadTextHJustify,
    v: KicadTextVJustify,
    mirrored: bool,
    keep_upright: bool,
}

impl Default for KicadTextJustify {
    fn default() -> Self {
        Self {
            h: KicadTextHJustify::Center,
            v: KicadTextVJustify::Center,
            mirrored: false,
            keep_upright: false,
        }
    }
}

fn kicad_parse_text_justify(block: &str) -> KicadTextJustify {
    let mut justify = KicadTextJustify::default();
    for line in block.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with("(justify ") {
            continue;
        }
        if trimmed.contains(" left") {
            justify.h = KicadTextHJustify::Left;
        } else if trimmed.contains(" right") {
            justify.h = KicadTextHJustify::Right;
        }
        if trimmed.contains(" top") {
            justify.v = KicadTextVJustify::Top;
        } else if trimmed.contains(" bottom") {
            justify.v = KicadTextVJustify::Bottom;
        }
        if trimmed.contains(" mirror") {
            justify.mirrored = true;
        }
    }
    justify
}

fn kicad_text_attributes(
    anchor_position: PointNm,
    rotation_degrees: i32,
    height_nm: i64,
    stroke_width_nm: Option<i64>,
    justify: KicadTextJustify,
) -> TextAttributes {
    TextAttributes {
        position: eda_engine::ir::geometry::Point {
            x: anchor_position.x,
            y: anchor_position.y,
        },
        rotation_degrees,
        height_nm,
        stroke_width_nm: stroke_width_nm.unwrap_or(default_stroke_width_nm(height_nm)),
        h_align: match justify.h {
            KicadTextHJustify::Left => TextHAlign::Left,
            KicadTextHJustify::Center => TextHAlign::Center,
            KicadTextHJustify::Right => TextHAlign::Right,
        },
        v_align: match justify.v {
            KicadTextVJustify::Top => TextVAlign::Top,
            KicadTextVJustify::Center => TextVAlign::Center,
            KicadTextVJustify::Bottom => TextVAlign::Bottom,
        },
        mirrored: justify.mirrored,
        keep_upright: justify.keep_upright,
        line_spacing_ratio_ppm: 1_350_000,
        render_intent: TextRenderIntent::Manufacturing,
        family: TextFamilyId::default(),
        family_source: eda_engine::text::TextFamilySource::ImplicitDefault,
        style: TextStyleId::default(),
        italic: false,
        bold: false,
        style_class: None,
    }
}

fn kicad_text_rotation_degrees(rotation_degrees: i32) -> i32 {
    -rotation_degrees
}

fn board_text_primitive(text: &BoardText) -> BoardTextPrimitive {
    BoardTextPrimitive {
        object_id: format!("board-text:{}", text.uuid),
        object_kind: "board_text".to_string(),
        text_uuid: text.uuid.to_string(),
        text: text.text.clone(),
        layer_id: layer_id(text.layer),
        position: PointNm {
            x: text.position.x,
            y: text.position.y,
        },
        rotation_degrees: text.rotation,
        height_nm: text.height_nm,
        stroke_width_nm: text.stroke_width_nm,
        render_intent: render_intent_to_string(text.render_intent).to_string(),
        family: text.family.0.clone(),
        style: text.style.0.clone(),
        style_class: text.style_class.clone(),
        h_align: h_align_to_string(text.h_align).to_string(),
        v_align: v_align_to_string(text.v_align).to_string(),
        mirrored: text.mirrored,
        keep_upright: text.keep_upright,
        line_spacing_ratio_ppm: text.line_spacing_ratio_ppm,
        bold: text.bold,
        italic: text.italic,
    }
}

fn render_intent_to_string(intent: TextRenderIntent) -> &'static str {
    match intent {
        TextRenderIntent::Manufacturing => "manufacturing",
        TextRenderIntent::Annotation => "annotation",
        TextRenderIntent::Branding => "branding",
        TextRenderIntent::Documentation => "documentation",
        TextRenderIntent::UiPreview => "ui_preview",
    }
}

fn h_align_to_string(align: TextHAlign) -> &'static str {
    match align {
        TextHAlign::Left => "left",
        TextHAlign::Center => "center",
        TextHAlign::Right => "right",
    }
}

fn v_align_to_string(align: TextVAlign) -> &'static str {
    match align {
        TextVAlign::Top => "top",
        TextVAlign::Center => "center",
        TextVAlign::Bottom => "bottom",
    }
}

fn board_text_geometry(
    text: &BoardText,
) -> (BoardTextGeometryPrimitive, Vec<GlyphMeshAssetPrimitive>) {
    let mesh_scene = match layout_text_mesh_from_board_text(text) {
        Ok(scene) => Some(scene),
        Err(error) => {
            trace_board_text_geometry_fallback(text, "mesh", &error);
            None
        }
    };
    let glyph_mesh_assets = mesh_scene
        .as_ref()
        .map(|scene| {
            scene
                .glyph_mesh_assets
                .iter()
                .map(glyph_mesh_asset_primitive)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let world_transform_nm = mesh_scene
        .as_ref()
        .map(|scene| affine_2d_fixed_primitive(scene.batch.world_transform));
    let block_bbox_em_nm = mesh_scene
        .as_ref()
        .map(|scene| mesh_rect_em_primitive(scene.batch.block_bbox_em_nm));
    let glyphs = mesh_scene
        .as_ref()
        .map(|scene| {
            scene
                .batch
                .glyphs
                .iter()
                .map(text_glyph_instance_primitive)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let mut fills = Vec::new();
    let mut strokes = Vec::new();

    // The renderer consumes glyph meshes when available. Legacy fill/stroke
    // fragments are only a fallback; generating both paths doubles import-time
    // text tessellation cost on real KiCad boards.
    if glyphs.is_empty() {
        match layout_text_geometry(&text.text, &TextAttributes::from_board_text(text)) {
            Ok(primitives) => {
                for primitive in primitives {
                    match primitive {
                        TextGeometryPrimitive::Stroke(stroke) => {
                            strokes.push(BoardTextStrokePrimitive {
                                from: PointNm {
                                    x: stroke.from.x,
                                    y: stroke.from.y,
                                },
                                to: PointNm {
                                    x: stroke.to.x,
                                    y: stroke.to.y,
                                },
                                width_nm: stroke.width_nm,
                            });
                        }
                        TextGeometryPrimitive::FilledPolygon(polygon) => {
                            fills.push(BoardTextFillPrimitive {
                                outer: polygon
                                    .outer
                                    .into_iter()
                                    .map(|point| PointNm {
                                        x: point.x,
                                        y: point.y,
                                    })
                                    .collect(),
                                holes: polygon
                                    .holes
                                    .into_iter()
                                    .map(|ring| {
                                        ring.into_iter()
                                            .map(|point| PointNm {
                                                x: point.x,
                                                y: point.y,
                                            })
                                            .collect()
                                    })
                                    .collect(),
                            });
                        }
                    }
                }
            }
            Err(error) => trace_board_text_geometry_fallback(text, "legacy", &error),
        }
    }

    (
        BoardTextGeometryPrimitive {
            object_id: format!("board-text:{}", text.uuid),
            object_kind: "board_text_geometry".to_string(),
            text_uuid: text.uuid.to_string(),
            layer_id: layer_id(text.layer),
            world_transform_nm,
            block_bbox_em_nm,
            glyphs,
            fills,
            strokes,
        },
        glyph_mesh_assets,
    )
}

fn trace_board_text_geometry_fallback(
    text: &BoardText,
    stage: &str,
    error: &dyn std::fmt::Display,
) {
    if !kicad_import_text_trace_enabled() {
        return;
    }
    eprintln!(
        "[datum-import-text] board_text_geometry_fallback stage={stage} text_uuid={} layer={} family={} intent={} chars={} error={error}",
        text.uuid,
        layer_id(text.layer),
        text.family.0,
        render_intent_to_string(text.render_intent),
        text.text.chars().count(),
    );
}

fn push_board_text_scene_primitives(
    board_text: &BoardText,
    primitives: &mut Vec<BoardTextPrimitive>,
    geometries: &mut Vec<BoardTextGeometryPrimitive>,
    mesh_assets_by_handle: &mut BTreeMap<GlyphMeshHandlePrimitive, GlyphMeshAssetPrimitive>,
) {
    primitives.push(board_text_primitive(board_text));
    let (geometry, mesh_assets) = board_text_geometry(board_text);
    geometries.push(geometry);
    for asset in mesh_assets {
        mesh_assets_by_handle.entry(asset.handle).or_insert(asset);
    }
}

fn kicad_import_text_uuid(kind: &str, key: &str) -> uuid::Uuid {
    uuid::Uuid::new_v5(
        &uuid::Uuid::NAMESPACE_URL,
        format!("datum:kicad-import-text:{kind}:{key}").as_bytes(),
    )
}

fn kicad_board_text(
    uuid: uuid::Uuid,
    text: String,
    layer: i32,
    position: PointNm,
    rotation_degrees: i32,
    height_nm: i64,
    stroke_width_nm: Option<i64>,
    justify: KicadTextJustify,
    style_class: Option<String>,
) -> BoardText {
    let attrs = kicad_text_attributes(
        position,
        rotation_degrees,
        height_nm,
        stroke_width_nm,
        justify,
    );
    BoardText {
        uuid,
        text,
        position: attrs.position,
        rotation: attrs.rotation_degrees,
        layer,
        render_intent: attrs.render_intent,
        family: attrs.family,
        family_source: attrs.family_source,
        style: attrs.style,
        height_nm: attrs.height_nm,
        stroke_width_nm: attrs.stroke_width_nm,
        h_align: attrs.h_align,
        v_align: attrs.v_align,
        mirrored: attrs.mirrored,
        keep_upright: attrs.keep_upright,
        line_spacing_ratio_ppm: attrs.line_spacing_ratio_ppm,
        italic: attrs.italic,
        bold: attrs.bold,
        style_class,
    }
}

fn merge_glyph_mesh_assets(
    target: &mut Vec<GlyphMeshAssetPrimitive>,
    incoming: Vec<GlyphMeshAssetPrimitive>,
) {
    let mut seen: BTreeSet<GlyphMeshHandlePrimitive> =
        target.iter().map(|asset| asset.handle).collect();
    for asset in incoming {
        if seen.insert(asset.handle) {
            target.push(asset);
        }
    }
}

fn glyph_mesh_asset_primitive(asset: &eda_engine::text::GlyphMeshAsset) -> GlyphMeshAssetPrimitive {
    GlyphMeshAssetPrimitive {
        handle: glyph_mesh_handle_primitive(asset.handle),
        vertices: asset
            .vertices
            .iter()
            .map(|vertex| MeshVertexEmPrimitive {
                x_em_nm: vertex.x_em_nm,
                y_em_nm: vertex.y_em_nm,
            })
            .collect(),
        indices: asset.indices.clone(),
        bbox_em_nm: mesh_rect_em_primitive(asset.bbox_em_nm),
    }
}

fn text_glyph_instance_primitive(
    glyph: &eda_engine::text::TextGlyphInstance,
) -> TextGlyphInstancePrimitive {
    TextGlyphInstancePrimitive {
        glyph_handle: glyph_mesh_handle_primitive(glyph.glyph_handle),
        origin_em_nm_x: glyph.origin_em_nm_x,
        origin_em_nm_y: glyph.origin_em_nm_y,
    }
}

fn glyph_mesh_handle_primitive(
    handle: eda_engine::text::GlyphMeshHandle,
) -> GlyphMeshHandlePrimitive {
    GlyphMeshHandlePrimitive {
        font_id: handle.font_id,
        glyph_id: handle.glyph_id,
        tolerance_class: handle.tolerance_class,
        epoch: handle.epoch,
    }
}

fn mesh_rect_em_primitive(rect: eda_engine::text::MeshRectEm) -> MeshRectEmPrimitive {
    MeshRectEmPrimitive {
        min_x_em_nm: rect.min_x_em_nm,
        min_y_em_nm: rect.min_y_em_nm,
        max_x_em_nm: rect.max_x_em_nm,
        max_y_em_nm: rect.max_y_em_nm,
    }
}

fn affine_2d_fixed_primitive(transform: eda_engine::text::Affine2DFixed) -> Affine2DFixedPrimitive {
    Affine2DFixedPrimitive {
        m11_ppm: transform.m11_ppm,
        m12_ppm: transform.m12_ppm,
        m21_ppm: transform.m21_ppm,
        m22_ppm: transform.m22_ppm,
        tx_nm: transform.tx_nm,
        ty_nm: transform.ty_nm,
    }
}

/// Extract footprint graphics from KiCad board file content.
fn extract_kicad_footprint_graphics(
    contents: &str,
    components: &[BoardComponentPayload],
    layer_table: &std::collections::HashMap<String, i32>,
) -> (
    Vec<ComponentGraphicPrimitive>,
    Vec<ComponentTextPrimitive>,
    Vec<BoardTextPrimitive>,
    Vec<BoardTextGeometryPrimitive>,
    Vec<GlyphMeshAssetPrimitive>,
) {
    let mut all_graphics = Vec::new();
    let mut board_texts = Vec::new();
    let mut board_text_geometries = Vec::new();
    let mut glyph_mesh_assets_by_handle = BTreeMap::new();
    let mut text_trace = KicadImportTextTrace::default();

    // Build a lookup from UUID string to component.
    let comp_by_uuid: std::collections::HashMap<&str, &BoardComponentPayload> =
        components.iter().map(|c| (c.uuid.as_str(), c)).collect();

    for fp_block in kicad_nested_blocks(contents, "footprint") {
        // Find the footprint UUID and match to a known component.
        let fp_uuid = match kicad_parse_uuid(&fp_block) {
            Some(u) => u,
            None => continue,
        };
        let component = match comp_by_uuid.get(fp_uuid.as_str()) {
            Some(c) => *c,
            None => continue,
        };

        let mut graphic_index = 0usize;
        let mut text_index = 0usize;
        let fp_blocks = kicad_nested_blocks_by_form(
            &fp_block,
            &[
                "fp_line",
                "fp_rect",
                "fp_circle",
                "fp_arc",
                "fp_poly",
                "fp_text",
                "property",
            ],
        );

        // --- fp_line ---
        for block in fp_blocks.get("fp_line").into_iter().flatten() {
            let layer_name = match kicad_parse_layer_anywhere(&block) {
                Some(n) => n,
                None => continue,
            };
            let role = match kicad_render_role(&layer_name) {
                Some(r) => r,
                None => continue,
            };
            let start = match kicad_parse_xy_anywhere(&block, "start") {
                Some(p) => p,
                None => continue,
            };
            let end = match kicad_parse_xy_anywhere(&block, "end") {
                Some(p) => p,
                None => continue,
            };
            let width = kicad_parse_width_nm(&block);
            let lid = kicad_resolve_layer_id(&layer_name, layer_table);
            all_graphics.push(ComponentGraphicPrimitive {
                graphic_id: format!("component-graphic:{}:kicad-line:{graphic_index}", fp_uuid),
                component_uuid: fp_uuid.clone(),
                layer_id: Some(layer_id(lid)),
                primitive_kind: "polyline".to_string(),
                render_role: role.to_string(),
                width_nm: Some(width),
                closed: false,
                path: vec![
                    transform_component_local_point(component, start),
                    transform_component_local_point(component, end),
                ],
                holes: Vec::new(),
            });
            graphic_index += 1;
        }

        // --- fp_rect ---
        for block in fp_blocks.get("fp_rect").into_iter().flatten() {
            let layer_name = match kicad_parse_layer_anywhere(&block) {
                Some(n) => n,
                None => continue,
            };
            let role = match kicad_render_role(&layer_name) {
                Some(r) => r,
                None => continue,
            };
            let s = match kicad_parse_xy_anywhere(&block, "start") {
                Some(p) => p,
                None => continue,
            };
            let e = match kicad_parse_xy_anywhere(&block, "end") {
                Some(p) => p,
                None => continue,
            };
            let width = kicad_parse_width_nm(&block);
            let lid = kicad_resolve_layer_id(&layer_name, layer_table);
            let min_x = s.x.min(e.x);
            let min_y = s.y.min(e.y);
            let max_x = s.x.max(e.x);
            let max_y = s.y.max(e.y);
            let corners = [
                PointNm { x: min_x, y: min_y },
                PointNm { x: max_x, y: min_y },
                PointNm { x: max_x, y: max_y },
                PointNm { x: min_x, y: max_y },
                PointNm { x: min_x, y: min_y },
            ];
            all_graphics.push(ComponentGraphicPrimitive {
                graphic_id: format!("component-graphic:{}:kicad-rect:{graphic_index}", fp_uuid),
                component_uuid: fp_uuid.clone(),
                layer_id: Some(layer_id(lid)),
                primitive_kind: "polyline".to_string(),
                render_role: role.to_string(),
                width_nm: Some(width),
                closed: true,
                path: corners
                    .iter()
                    .map(|p| transform_component_local_point(component, *p))
                    .collect(),
                holes: Vec::new(),
            });
            graphic_index += 1;
        }

        // --- fp_circle ---
        for block in fp_blocks.get("fp_circle").into_iter().flatten() {
            let layer_name = match kicad_parse_layer_anywhere(&block) {
                Some(n) => n,
                None => continue,
            };
            let role = match kicad_render_role(&layer_name) {
                Some(r) => r,
                None => continue,
            };
            let center = match kicad_parse_xy_anywhere(&block, "center") {
                Some(p) => p,
                None => continue,
            };
            let end_pt = match kicad_parse_xy_anywhere(&block, "end") {
                Some(p) => p,
                None => continue,
            };
            let dx = end_pt.x - center.x;
            let dy = end_pt.y - center.y;
            let radius = ((dx as f64 * dx as f64 + dy as f64 * dy as f64).sqrt()).round() as i64;
            let width = kicad_parse_width_nm(&block);
            let lid = kicad_resolve_layer_id(&layer_name, layer_table);
            all_graphics.push(ComponentGraphicPrimitive {
                graphic_id: format!("component-graphic:{}:kicad-circle:{graphic_index}", fp_uuid),
                component_uuid: fp_uuid.clone(),
                layer_id: Some(layer_id(lid)),
                primitive_kind: "polyline".to_string(),
                render_role: role.to_string(),
                width_nm: Some(width),
                closed: true,
                path: approximate_circle_path(component, center, radius),
                holes: Vec::new(),
            });
            graphic_index += 1;
        }

        // --- fp_arc ---
        for block in fp_blocks.get("fp_arc").into_iter().flatten() {
            let layer_name = match kicad_parse_layer_anywhere(&block) {
                Some(n) => n,
                None => continue,
            };
            let role = match kicad_render_role(&layer_name) {
                Some(r) => r,
                None => continue,
            };
            let start = match kicad_parse_xy_anywhere(&block, "start") {
                Some(p) => p,
                None => continue,
            };
            let mid = match kicad_parse_xy_anywhere(&block, "mid") {
                Some(p) => p,
                None => continue,
            };
            let end = match kicad_parse_xy_anywhere(&block, "end") {
                Some(p) => p,
                None => continue,
            };
            let width = kicad_parse_width_nm(&block);
            let lid = kicad_resolve_layer_id(&layer_name, layer_table);
            let path = if let Some((center, radius, start_angle, end_angle)) =
                kicad_arc_from_three_points(&start, &mid, &end)
            {
                approximate_arc_path(component, center, radius, start_angle, end_angle)
            } else {
                // Collinear fallback — just draw start→mid→end.
                vec![
                    transform_component_local_point(component, start),
                    transform_component_local_point(component, mid),
                    transform_component_local_point(component, end),
                ]
            };
            all_graphics.push(ComponentGraphicPrimitive {
                graphic_id: format!("component-graphic:{}:kicad-arc:{graphic_index}", fp_uuid),
                component_uuid: fp_uuid.clone(),
                layer_id: Some(layer_id(lid)),
                primitive_kind: "polyline".to_string(),
                render_role: role.to_string(),
                width_nm: Some(width),
                closed: false,
                path,
                holes: Vec::new(),
            });
            graphic_index += 1;
        }

        // --- fp_poly ---
        for block in fp_blocks.get("fp_poly").into_iter().flatten() {
            let layer_name = match kicad_parse_layer_anywhere(&block) {
                Some(n) => n,
                None => continue,
            };
            let role = match kicad_render_role(&layer_name) {
                Some(r) => r,
                None => continue,
            };
            let vertices = kicad_parse_xy_points(&block);
            if vertices.is_empty() {
                continue;
            }
            let width = kicad_parse_width_nm(&block);
            let lid = kicad_resolve_layer_id(&layer_name, layer_table);
            all_graphics.push(ComponentGraphicPrimitive {
                graphic_id: format!("component-graphic:{}:kicad-poly:{graphic_index}", fp_uuid),
                component_uuid: fp_uuid.clone(),
                layer_id: Some(layer_id(lid)),
                primitive_kind: "polygon".to_string(),
                render_role: role.to_string(),
                width_nm: Some(width),
                closed: true,
                path: vertices
                    .into_iter()
                    .map(|p| transform_component_local_point(component, p))
                    .collect(),
                holes: Vec::new(),
            });
            graphic_index += 1;
        }

        // --- fp_text (literal text only, skip ${REFERENCE} and ${VALUE}) ---
        for block in fp_blocks.get("fp_text").into_iter().flatten() {
            text_trace.fp_text_total += 1;
            let first_line = match block.lines().next() {
                Some(l) => l.trim(),
                None => continue,
            };
            // Extract the text content — it is the second quoted token.
            // Format: (fp_text TYPE "text" (at ...) ...)
            let text = match kicad_extract_fp_text_content(first_line) {
                Some(t) => t,
                None => continue,
            };
            // Skip template references handled by the label system.
            if text.contains("${REFERENCE}")
                || text.contains("${VALUE}")
                || text == "%R"
                || text == "%V"
            {
                text_trace.fp_text_template_skipped += 1;
                continue;
            }
            if kicad_block_hidden(&block) {
                text_trace.fp_text_hidden_skipped += 1;
                continue;
            }
            let layer_name = match kicad_parse_layer_anywhere(&block) {
                Some(n) => n,
                None => continue,
            };
            let role = match kicad_render_role(&layer_name) {
                Some(r) => r,
                None => continue,
            };
            let (local_pos, local_rot) = match kicad_parse_at(&block) {
                Some(v) => v,
                None => continue,
            };
            let lid = kicad_resolve_layer_id(&layer_name, layer_table);
            let height = kicad_parse_font_height_nm(&block);
            let stroke_width_nm = kicad_parse_font_thickness_nm(&block);
            let board_pos = transform_component_local_point(component, local_pos);
            let board_rot = kicad_text_rotation_degrees(component.rotation + local_rot);
            let mut justify = kicad_parse_text_justify(&block);
            justify.keep_upright = true;
            let source_uuid = kicad_parse_uuid(&block).unwrap_or_else(|| {
                kicad_import_text_uuid("fp_text", &format!("{fp_uuid}/{text_index}/{text}/{lid}"))
                    .to_string()
            });
            let Ok(text_uuid) = uuid::Uuid::parse_str(&source_uuid) else {
                continue;
            };
            let board_text = kicad_board_text(
                text_uuid,
                text,
                lid,
                board_pos,
                board_rot,
                height,
                stroke_width_nm,
                justify,
                Some(format!("imported_kicad_fp_text:{fp_uuid}:{role}")),
            );
            push_board_text_scene_primitives(
                &board_text,
                &mut board_texts,
                &mut board_text_geometries,
                &mut glyph_mesh_assets_by_handle,
            );
            text_trace.fp_text_imported += 1;
            text_trace.record_import("fp_text", &layer_name, lid, &board_text.text);
            text_index += 1;
        }

        // --- property blocks (Reference/Value on silkscreen/fab layers) ---
        for prop_section in fp_blocks.get("property").into_iter().flatten() {
            text_trace.property_total += 1;
            let first_line = match prop_section.lines().next() {
                Some(line) => line.trim(),
                None => continue,
            };
            let mut quoted = Vec::new();
            let mut rest = first_line;
            while let Some(start) = rest.find('"') {
                let after = &rest[start + 1..];
                if let Some(end) = after.find('"') {
                    quoted.push(after[..end].to_string());
                    rest = &after[end + 1..];
                } else {
                    break;
                }
            }
            if quoted.len() < 2 {
                continue;
            }
            let key = &quoted[0];
            if key != "Reference" && key != "Value" {
                text_trace.property_metadata_skipped += 1;
                continue;
            }
            let text = quoted[1].clone();
            if text.is_empty() || text.starts_with('~') {
                text_trace.property_empty_skipped += 1;
                continue;
            }
            if kicad_block_hidden(&prop_section) {
                text_trace.property_hidden_skipped += 1;
                continue;
            }
            let layer_name = match kicad_parse_layer_anywhere(&prop_section) {
                Some(n) => n,
                None => continue,
            };
            let role = match kicad_render_role(&layer_name) {
                Some(r) => r,
                None => continue,
            };
            let layer_id = kicad_resolve_layer_id(&layer_name, layer_table);
            let (local_pos, local_rot) = match kicad_parse_at(&prop_section) {
                Some(v) => v,
                None => continue,
            };
            let board_pos = transform_component_local_point(component, local_pos);
            let height_nm = kicad_parse_font_height_nm(&prop_section);
            let stroke_width_nm = kicad_parse_font_thickness_nm(&prop_section);
            let board_rot = kicad_text_rotation_degrees(component.rotation + local_rot);
            let mut justify = kicad_parse_text_justify(&prop_section);
            justify.keep_upright = true;
            let text_uuid = kicad_import_text_uuid(
                "property_text",
                &format!(
                    "{}/{}/{text}/{layer_id}",
                    component.uuid,
                    key.to_lowercase()
                ),
            );
            let board_text = kicad_board_text(
                text_uuid,
                text,
                layer_id,
                board_pos,
                board_rot,
                height_nm,
                stroke_width_nm,
                justify,
                Some(format!(
                    "imported_kicad_property_text:{}:{}:{role}",
                    component.uuid,
                    key.to_lowercase()
                )),
            );
            push_board_text_scene_primitives(
                &board_text,
                &mut board_texts,
                &mut board_text_geometries,
                &mut glyph_mesh_assets_by_handle,
            );
            if key == "Reference" {
                text_trace.property_reference_imported += 1;
                text_trace.record_import(
                    "property_reference",
                    &layer_name,
                    layer_id,
                    &board_text.text,
                );
            } else {
                text_trace.property_value_imported += 1;
                text_trace.record_import("property_value", &layer_name, layer_id, &board_text.text);
            }
        }
    }

    text_trace.emit(
        "footprints",
        board_texts.len(),
        board_text_geometries.len(),
        glyph_mesh_assets_by_handle.len(),
    );

    (
        all_graphics,
        Vec::new(),
        board_texts,
        board_text_geometries,
        glyph_mesh_assets_by_handle.into_values().collect(),
    )
}

/// Interpolate an arc from three world-space points into a polyline of ~64
/// segments. Mirrors the segment count used by the engine's outline assembly.
fn kicad_interpolate_arc_world(start: PointNm, mid: PointNm, end: PointNm) -> Vec<PointNm> {
    let Some((center, radius, start_tenths, end_tenths)) =
        kicad_arc_from_three_points(&start, &mid, &end)
    else {
        return vec![start, mid, end];
    };
    let mut sweep_tenths = end_tenths - start_tenths;
    // Pick the sweep direction that includes the mid-angle.
    let mid_tenths = (((mid.y as f64 - center.y as f64)
        .atan2(mid.x as f64 - center.x as f64)
        .to_degrees()
        * 10.0)
        .round() as i32)
        .rem_euclid(3600);
    let includes_mid = |s_t: i32, sweep: i32, m_t: i32| -> bool {
        let mut rel = (m_t - s_t).rem_euclid(3600);
        if sweep >= 0 {
            rel <= sweep
        } else {
            rel = rel - 3600;
            rel >= sweep
        }
    };
    if !includes_mid(start_tenths, sweep_tenths, mid_tenths) {
        if sweep_tenths > 0 {
            sweep_tenths -= 3600;
        } else {
            sweep_tenths += 3600;
        }
    }
    const SEGMENT_ANGLE_TENTHS: i32 = 100; // ~10 deg → ≈36 segments for a full circle
    let segment_count = (sweep_tenths.abs() / SEGMENT_ANGLE_TENTHS).max(1);
    let mut out: Vec<PointNm> = (0..=segment_count)
        .map(|idx| {
            let t = start_tenths + sweep_tenths * idx / segment_count;
            let rad = (f64::from(t) / 10.0).to_radians();
            PointNm {
                x: (center.x as f64 + radius as f64 * rad.cos()).round() as i64,
                y: (center.y as f64 + radius as f64 * rad.sin()).round() as i64,
            }
        })
        .collect();
    // Force first/last to exact source endpoints so chaining against adjacent
    // contributors remains precise.
    if let Some(first) = out.first_mut() {
        *first = start;
    }
    if let Some(last) = out.last_mut() {
        *last = end;
    }
    out
}

/// Extract imported Edge.Cuts contributors as authored board-level graphics.
/// One walk produces primitives for top-level `gr_line` / `gr_arc` and
/// footprint-embedded `fp_line` / `fp_arc` on Edge.Cuts, under the footprint
/// `(at x y rot)` transform where applicable. See M7-SCN-007 brief.
///
/// `edge_cuts_layer_key` is the scene-level layer-id key under which the
/// Edge.Cuts layer is indexed (the `"L{n}"` form used by `scene.layers` and
/// the layer-visibility map). This must match the rest of the scene's
/// layer-id convention so visibility toggles actually gate these primitives.
fn extract_kicad_board_graphics(
    contents: &str,
    board_uuid: &str,
    layer_table: &std::collections::HashMap<String, i32>,
) -> Vec<BoardGraphicPrimitive> {
    let mut out: Vec<BoardGraphicPrimitive> = Vec::new();
    let mut ordinal: usize = 0;

    let mut stable_id = |kind: &str, src_uuid: &str| -> (String, String) {
        let src = if src_uuid.is_empty() {
            format!("{board_uuid}:edge-cuts:{kind}:{ordinal}")
        } else {
            src_uuid.to_string()
        };
        let oid = format!("board-graphic:{src}");
        ordinal += 1;
        (oid, src)
    };

    // Top-level contributors (no transform).
    for block in kicad_nested_blocks(contents, "gr_line") {
        let Some(layer_name) = kicad_parse_layer_anywhere(&block) else {
            continue;
        };
        let Some(layer_key) = kicad_board_graphic_layer_key(&layer_name, layer_table) else {
            continue;
        };
        let (Some(start), Some(end)) = (
            kicad_parse_xy_anywhere_block(&block, "start"),
            kicad_parse_xy_anywhere_block(&block, "end"),
        ) else {
            continue;
        };
        let width = kicad_parse_width_nm(&block);
        let uuid = kicad_parse_uuid(&block).unwrap_or_default();
        let (object_id, source) = stable_id("line", &uuid);
        out.push(BoardGraphicPrimitive {
            object_id,
            object_kind: "board_graphic".to_string(),
            primitive_kind: "polyline".to_string(),
            source_object_uuid: source,
            layer_id: layer_key,
            path: vec![start, end],
            holes: Vec::new(),
            width_nm: Some(width),
        });
    }
    for block in kicad_nested_blocks(contents, "gr_arc") {
        let Some(layer_name) = kicad_parse_layer_anywhere(&block) else {
            continue;
        };
        let Some(layer_key) = kicad_board_graphic_layer_key(&layer_name, layer_table) else {
            continue;
        };
        let (Some(start), Some(mid), Some(end)) = (
            kicad_parse_xy_anywhere_block(&block, "start"),
            kicad_parse_xy_anywhere_block(&block, "mid"),
            kicad_parse_xy_anywhere_block(&block, "end"),
        ) else {
            continue;
        };
        let width = kicad_parse_width_nm(&block);
        let uuid = kicad_parse_uuid(&block).unwrap_or_default();
        let (object_id, source) = stable_id("arc", &uuid);
        out.push(BoardGraphicPrimitive {
            object_id,
            object_kind: "board_graphic".to_string(),
            primitive_kind: "polyline".to_string(),
            source_object_uuid: source,
            layer_id: layer_key,
            path: kicad_interpolate_arc_world(start, mid, end),
            holes: Vec::new(),
            width_nm: Some(width),
        });
    }
    for block in kicad_nested_blocks(contents, "gr_poly") {
        let Some(layer_name) = kicad_parse_layer_anywhere(&block) else {
            continue;
        };
        let Some(layer_key) = kicad_board_graphic_layer_key(&layer_name, layer_table) else {
            continue;
        };
        let mut path = kicad_parse_xy_points(&block);
        if path.len() < 2 {
            continue;
        }
        let width = kicad_parse_width_nm(&block);
        let uuid = kicad_parse_uuid(&block).unwrap_or_default();
        let (object_id, source) = stable_id("poly", &uuid);
        let filled = block.contains("(fill yes)");
        if !filled
            && path
                .first()
                .zip(path.last())
                .is_some_and(|(first, last)| first != last)
            && let Some(first) = path.first().copied()
        {
            path.push(first);
        }
        out.push(BoardGraphicPrimitive {
            object_id,
            object_kind: "board_graphic".to_string(),
            primitive_kind: if filled { "polygon" } else { "polyline" }.to_string(),
            source_object_uuid: source,
            layer_id: layer_key,
            path,
            holes: Vec::new(),
            width_nm: Some(width),
        });
    }
    for block in kicad_nested_blocks(contents, "gr_circle") {
        let Some(layer_name) = kicad_parse_layer_anywhere(&block) else {
            continue;
        };
        let Some(layer_key) = kicad_board_graphic_layer_key(&layer_name, layer_table) else {
            continue;
        };
        let (Some(center), Some(end_pt)) = (
            kicad_parse_xy_anywhere_block(&block, "center"),
            kicad_parse_xy_anywhere_block(&block, "end"),
        ) else {
            continue;
        };
        let dx = end_pt.x - center.x;
        let dy = end_pt.y - center.y;
        let radius = ((dx as f64 * dx as f64 + dy as f64 * dy as f64).sqrt()).round() as i64;
        let width = kicad_parse_width_nm(&block);
        let uuid = kicad_parse_uuid(&block).unwrap_or_default();
        let (object_id, source) = stable_id("circle", &uuid);
        let filled = block.contains("(fill yes)");
        let path = approximate_world_circle_path(center, radius);
        out.push(BoardGraphicPrimitive {
            object_id,
            object_kind: "board_graphic".to_string(),
            primitive_kind: if filled { "polygon" } else { "polyline" }.to_string(),
            source_object_uuid: source,
            layer_id: layer_key,
            path,
            holes: Vec::new(),
            width_nm: Some(width),
        });
    }
    out
}

fn extract_kicad_board_texts(
    contents: &str,
    layer_table: &std::collections::HashMap<String, i32>,
) -> (
    Vec<BoardTextPrimitive>,
    Vec<BoardTextGeometryPrimitive>,
    Vec<GlyphMeshAssetPrimitive>,
) {
    let mut board_texts = Vec::new();
    let mut board_text_geometries = Vec::new();
    let mut glyph_mesh_assets_by_handle = BTreeMap::new();
    let mut text_trace = KicadImportTextTrace::default();
    for (index, block) in kicad_nested_blocks(contents, "gr_text")
        .into_iter()
        .enumerate()
    {
        text_trace.gr_text_total += 1;
        if kicad_block_hidden(&block) {
            text_trace.gr_text_hidden_skipped += 1;
            continue;
        }
        let Some(layer_name) = kicad_parse_layer_anywhere(&block) else {
            continue;
        };
        let layer = kicad_resolve_layer_id(&layer_name, layer_table);
        let Some((position, rotation)) = kicad_parse_at(&block) else {
            continue;
        };
        let Some(first_line) = block.lines().next().map(str::trim) else {
            continue;
        };
        let Some(start) = first_line.find('"') else {
            continue;
        };
        let rest = &first_line[start + 1..];
        let Some(end) = rest.find('"') else {
            continue;
        };
        let text = rest[..end].to_string();
        let uuid = kicad_parse_uuid(&block)
            .and_then(|value| uuid::Uuid::parse_str(&value).ok())
            .unwrap_or_else(|| {
                kicad_import_text_uuid(
                    "gr_text",
                    &format!("{index}/{text}/{}/{}/{}", position.x, position.y, layer),
                )
            });
        let board_text = kicad_board_text(
            uuid,
            text,
            layer,
            position,
            kicad_text_rotation_degrees(rotation),
            kicad_parse_font_height_nm(&block),
            kicad_parse_font_thickness_nm(&block),
            kicad_parse_text_justify(&block),
            Some("imported_kicad_gr_text".to_string()),
        );
        push_board_text_scene_primitives(
            &board_text,
            &mut board_texts,
            &mut board_text_geometries,
            &mut glyph_mesh_assets_by_handle,
        );
        text_trace.gr_text_imported += 1;
        text_trace.record_import("gr_text", &layer_name, layer, &board_text.text);
    }

    text_trace.emit(
        "board",
        board_texts.len(),
        board_text_geometries.len(),
        glyph_mesh_assets_by_handle.len(),
    );

    (
        board_texts,
        board_text_geometries,
        glyph_mesh_assets_by_handle.into_values().collect(),
    )
}

fn kicad_board_graphic_layer_key(
    layer_name: &str,
    layer_table: &std::collections::HashMap<String, i32>,
) -> Option<String> {
    match layer_name {
        "F.SilkS" | "B.SilkS" | "F.Fab" | "B.Fab" | "F.CrtYd" | "B.CrtYd" | "Edge.Cuts" => {
            Some(layer_id(kicad_resolve_layer_id(layer_name, layer_table)))
        }
        _ => None,
    }
}

fn approximate_world_circle_path(center: PointNm, radius: i64) -> Vec<PointNm> {
    let segments = 32usize;
    (0..=segments)
        .map(|i| {
            let angle = std::f64::consts::TAU * (i as f64) / (segments as f64);
            PointNm {
                x: center.x + (radius as f64 * angle.cos()).round() as i64,
                y: center.y + (radius as f64 * angle.sin()).round() as i64,
            }
        })
        .collect()
}

/// Native-project parity helper for M7-SCN-007 Option B.
///
/// Native board JSON persists the assembled board outline polygon but does not
/// currently preserve the original per-contributor Edge.Cuts primitives or
/// their source identities. For native projects, derive stable board-scoped
/// Edge.Cuts line primitives from the persisted outline so authored-layer
/// visibility, stacking, and picking behave consistently with imported boards.
fn outline_board_graphics_from_outline(
    outline: &OutlinePayload,
    board_uuid: &str,
    edge_cuts_layer_key: &str,
) -> Vec<BoardGraphicPrimitive> {
    let mut vertices = outline.vertices.clone();
    if vertices.len() < 2 {
        return Vec::new();
    }
    if outline.closed && vertices.first() != vertices.last() {
        if let Some(first) = vertices.first().copied() {
            vertices.push(first);
        }
    }
    vertices
        .windows(2)
        .enumerate()
        .map(|(index, segment)| {
            let source = format!("{board_uuid}:outline-segment:{index}");
            BoardGraphicPrimitive {
                object_id: format!("board-graphic:{source}"),
                object_kind: "board_graphic".to_string(),
                primitive_kind: "line".to_string(),
                source_object_uuid: source,
                layer_id: edge_cuts_layer_key.to_string(),
                path: vec![segment[0], segment[1]],
                holes: Vec::new(),
                width_nm: None,
            }
        })
        .collect()
}

fn unrouted_primitives_from_airwires(
    airwires: &[eda_engine::board::Airwire],
) -> Vec<UnroutedPrimitive> {
    airwires
        .iter()
        .map(|airwire| {
            let source = format!(
                "{}:{}:{}:{}:{}",
                airwire.net,
                airwire.from.component,
                airwire.from.pin,
                airwire.to.component,
                airwire.to.pin
            );
            UnroutedPrimitive {
                object_id: format!("unrouted:{source}"),
                object_kind: "unrouted".to_string(),
                source_object_uuid: source,
                net_uuid: airwire.net.to_string(),
                from_component: airwire.from.component.clone(),
                from_pin: airwire.from.pin.clone(),
                to_component: airwire.to.component.clone(),
                to_pin: airwire.to.pin.clone(),
                path: vec![
                    PointNm {
                        x: airwire.from_position.x,
                        y: airwire.from_position.y,
                    },
                    PointNm {
                        x: airwire.to_position.x,
                        y: airwire.to_position.y,
                    },
                ],
            }
        })
        .collect()
}

fn net_display_from_imported_board(board: &eda_engine::board::Board) -> Vec<NetDisplayEntry> {
    let mut nets: Vec<_> = board.nets.values().collect();
    nets.sort_by(|a, b| a.name.cmp(&b.name).then_with(|| a.uuid.cmp(&b.uuid)));
    nets.into_iter()
        .map(|net| NetDisplayEntry {
            net_uuid: net.uuid.to_string(),
            net_name: net.name.clone(),
            airwire_color_rgb: deterministic_airwire_color(net.uuid.as_bytes()),
        })
        .collect()
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

/// Block-level variant of `kicad_parse_xy_anywhere`: scan every line of the
/// block to locate the first `(form x y ...)` occurrence.
fn kicad_parse_xy_anywhere_block(block: &str, form: &str) -> Option<PointNm> {
    block
        .lines()
        .find_map(|line| kicad_parse_xy_anywhere(line.trim_start(), form))
}

/// Extract the text content from an `fp_text` first line.
/// Format: `(fp_text TYPE "text content" (at ...`
fn kicad_extract_fp_text_content(first_line: &str) -> Option<String> {
    let trimmed = first_line.trim();
    if !trimmed.starts_with("(fp_text ") {
        return None;
    }
    let after = &trimmed["(fp_text ".len()..];
    // Skip the type token (reference, value, user).
    let rest = after.trim_start();
    let rest = if rest.starts_with('"') {
        // Type is quoted (rare).
        let end = rest[1..].find('"')?;
        rest[end + 2..].trim_start()
    } else {
        let end = rest.find(|c: char| c.is_whitespace())?;
        rest[end..].trim_start()
    };
    // Now the text content should be quoted.
    if !rest.starts_with('"') {
        return None;
    }
    let inner = &rest[1..];
    let end = inner.find('"')?;
    Some(inner[..end].to_string())
}

/// Load the board scene directly from native project JSON files, bypassing
/// CLI subprocess invocations. Returns the built scene and the resolved
/// board file path.
fn load_scene_from_engine(request: &LiveReviewRequest) -> Result<(BoardReviewSceneV1, PathBuf)> {
    let root = &request.project_root;
    // --- Read project manifest ---
    let manifest_path = root.join("project.json");
    let manifest_text = std::fs::read_to_string(&manifest_path)
        .with_context(|| format!("failed to read {}", manifest_path.display()))?;
    let manifest: NativeManifest = serde_json::from_str(&manifest_text)
        .with_context(|| format!("failed to parse {}", manifest_path.display()))?;

    let board_path = root.join(&manifest.board);
    let board_text = std::fs::read_to_string(&board_path)
        .with_context(|| format!("failed to read {}", board_path.display()))?;
    let board_value: Value = serde_json::from_str(&board_text)
        .with_context(|| format!("failed to parse {}", board_path.display()))?;

    let board_uuid = board_value
        .get("uuid")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let inspect = ProjectInspectPayload {
        project_root: root.display().to_string(),
        project_name: manifest.name,
        project_uuid: manifest.uuid.to_string(),
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
    let net_display: Vec<NetDisplayEntry> = Vec::new();

    let scene = build_board_review_scene(
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
    Ok((scene, board_path))
}

/// Minimal native project manifest for scene loading.
#[derive(Debug, Clone, Deserialize)]
struct NativeManifest {
    uuid: uuid::Uuid,
    name: String,
    board: String,
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
            EnginePadShape::RoundRect => write!(f, "round_rect"),
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
    if let Some(path) = explain.selected_path {
        if path.points.len() >= 2 {
            return Ok(Some(path.points));
        }
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
            !graphic
                .layer_id
                .as_deref()
                .is_some_and(|layer_id| inferred_scene_layer_name(layer_id) == "Edge.Cuts")
        })
        .collect();
    let texts: Vec<&ComponentTextPrimitive> = texts
        .iter()
        .copied()
        .filter(|text| {
            !text
                .layer_id
                .as_deref()
                .is_some_and(|layer_id| inferred_scene_layer_name(layer_id) == "Edge.Cuts")
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
        "rect" | "oval" | "roundrect" => (pad.width.max(1)) / 2,
        _ => (pad.diameter.max(1)) / 2,
    };
    let half_height = match pad.shape.as_str() {
        "rect" | "oval" | "roundrect" => (pad.height.max(1)) / 2,
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
mod tests {
    use super::*;
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
    fn kicad_text_justify_defaults_to_center_center() {
        let justify = kicad_parse_text_justify(
            r#"(property "Reference" "R2"
  (effects
    (font (size 1 1) (thickness 0.15))
  )
)"#,
        );
        assert_eq!(justify.h, KicadTextHJustify::Center);
        assert_eq!(justify.v, KicadTextVJustify::Center);
        assert!(!justify.mirrored);
        assert!(!justify.keep_upright);
    }

    #[test]
    fn kicad_text_justify_parses_left_and_bottom_tokens() {
        let justify = kicad_parse_text_justify(
            r#"(fp_text user "OUT"
  (effects
    (font (size 0.8 0.8) (thickness 0.12))
    (justify left bottom)
  )
)"#,
        );
        assert_eq!(justify.h, KicadTextHJustify::Left);
        assert_eq!(justify.v, KicadTextVJustify::Bottom);
    }

    #[test]
    fn kicad_text_justify_parses_mirror_token() {
        let justify = kicad_parse_text_justify(
            r#"(fp_text user "OUT"
  (effects
    (font (size 0.8 0.8) (thickness 0.12))
    (justify left mirror)
  )
)"#,
        );
        assert_eq!(justify.h, KicadTextHJustify::Left);
        assert!(justify.mirrored);
    }

    #[test]
    fn kicad_text_attributes_maps_justify_into_engine_alignments() {
        let left = kicad_text_attributes(
            PointNm { x: 0, y: 0 },
            0,
            800_000,
            None,
            KicadTextJustify {
                h: KicadTextHJustify::Left,
                v: KicadTextVJustify::Center,
                mirrored: false,
                keep_upright: false,
            },
        );
        let center = kicad_text_attributes(
            PointNm { x: 0, y: 0 },
            0,
            800_000,
            None,
            KicadTextJustify {
                h: KicadTextHJustify::Center,
                v: KicadTextVJustify::Center,
                mirrored: false,
                keep_upright: false,
            },
        );
        let right = kicad_text_attributes(
            PointNm { x: 0, y: 0 },
            0,
            800_000,
            None,
            KicadTextJustify {
                h: KicadTextHJustify::Right,
                v: KicadTextVJustify::Center,
                mirrored: false,
                keep_upright: false,
            },
        );
        assert_eq!(left.h_align, TextHAlign::Left);
        assert_eq!(center.h_align, TextHAlign::Center);
        assert_eq!(right.h_align, TextHAlign::Right);
        assert_eq!(left.v_align, TextVAlign::Center);
        assert!(!left.keep_upright);
    }

    #[test]
    fn kicad_text_attributes_preserves_mirror_and_keep_upright() {
        let attrs = kicad_text_attributes(
            PointNm { x: 0, y: 0 },
            180,
            800_000,
            None,
            KicadTextJustify {
                h: KicadTextHJustify::Center,
                v: KicadTextVJustify::Center,
                mirrored: true,
                keep_upright: true,
            },
        );
        assert!(attrs.mirrored);
        assert!(attrs.keep_upright);
    }

    #[test]
    fn kicad_text_rotation_is_converted_into_engine_sign_convention() {
        assert_eq!(kicad_text_rotation_degrees(0), 0);
        assert_eq!(kicad_text_rotation_degrees(90), -90);
        assert_eq!(kicad_text_rotation_degrees(-90), 90);
        assert_eq!(kicad_text_rotation_degrees(180), -180);
    }

    #[test]
    fn kicad_layer_parser_canonicalizes_uppercase_silkscreen_name() {
        let block = r#"(property "Reference" "U1" (at 0 0 0) (layer "F.SILKS"))"#;
        assert_eq!(
            kicad_parse_layer_anywhere(block).as_deref(),
            Some("F.SilkS")
        );
        assert_eq!(kicad_render_role("F.SILKS"), Some("component_silkscreen"));
        assert_eq!(
            kicad_resolve_layer_id("F.SILKS", &std::collections::HashMap::new()),
            37
        );
    }

    #[test]
    fn imported_kicad_component_text_ignores_render_cache_for_final_geometry() {
        let board = r#"(kicad_pcb
  (footprint "Demo:Part"
    (layer "F.Cu")
    (uuid "00000000-0000-0000-0000-00000000cafe")
    (at 10 20 0)
    (property "Reference" "U1"
      (at 1 2 90)
      (layer "F.SilkS")
      (effects
        (font (size 1 1) (thickness 0.15))
      )
      (render_cache "U1" 90
        (polygon
          (pts
            (xy 1 2)
            (xy 3 2)
            (xy 3 4)
            (xy 1 4)
          )
        )
      )
    )
  )
)"#;
        let components = vec![BoardComponentPayload {
            uuid: "00000000-0000-0000-0000-00000000cafe".to_string(),
            reference: "U1".to_string(),
            value: "Demo".to_string(),
            position: PointNm {
                x: 10_000_000,
                y: 20_000_000,
            },
            rotation: 0,
            layer: 0,
            locked: false,
        }];
        let (graphics, texts, board_texts, board_text_geometries, glyph_mesh_assets) =
            extract_kicad_footprint_graphics(board, &components, &std::collections::HashMap::new());
        assert!(
            texts.is_empty(),
            "imported KiCad text should materialize through geometry only, not the overlay text path"
        );
        assert!(
            board_texts.iter().any(|entry| entry.text == "U1"),
            "imported KiCad property text should materialize through the structured board-text path"
        );
        assert!(
            board_text_geometries
                .iter()
                .any(|entry| entry.layer_id == "L37" && !entry.glyphs.is_empty()),
            "imported KiCad property text should produce mesh-backed board text geometry"
        );
        assert!(
            !glyph_mesh_assets.is_empty(),
            "imported KiCad property text should contribute glyph mesh assets"
        );
        assert!(
            graphics
                .iter()
                .all(|entry| !entry.graphic_id.contains(":prop-stroke:")
                    && !entry.graphic_id.contains(":prop-cache:")
                    && !entry.graphic_id.contains(":kicad-text-cache:")),
            "cache-derived and stroke-derived imported text ids should disappear once Datum owns final text geometry"
        );
    }

    #[test]
    fn imported_kicad_gr_text_uses_structured_text_geometry_path() {
        let board = r#"(kicad_pcb
  (layers
    (37 "F.SilkS" user "F.SilkS")
  )
  (gr_text "Demo"
    (at 5 5 0)
    (layer "F.SilkS")
    (effects
      (font (size 1 1) (thickness 0.15))
    )
    (uuid "66666666-7777-8888-9999-000000000000"))
)"#;
        let layer_table = kicad_parse_layer_table(board);
        let (texts, geometries, glyph_mesh_assets) = extract_kicad_board_texts(board, &layer_table);

        assert_eq!(texts.len(), 1);
        assert_eq!(texts[0].text, "Demo");
        assert_eq!(texts[0].layer_id, "L37");
        assert_eq!(geometries.len(), 1);
        assert_eq!(geometries[0].text_uuid, texts[0].text_uuid);
        assert!(
            !geometries[0].glyphs.is_empty(),
            "imported gr_text should produce mesh-backed text geometry"
        );
        assert!(
            geometries[0].strokes.is_empty(),
            "imported gr_text must not route through Newstroke stroke geometry"
        );
        assert!(
            !glyph_mesh_assets.is_empty(),
            "imported gr_text should contribute glyph mesh assets"
        );
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
        let u1_silk_lines = board["component_silkscreen"]["00000000-0000-0000-0000-00000000c203"]
            .as_array()
            .expect("U1 silkscreen lines should be an array");
        assert!(
            u1_silk_lines.len() >= 6,
            "KiCad-backed U1 silkscreen should replace the minimal demo geometry"
        );
        let j2_texts = board["component_silkscreen_texts"]["00000000-0000-0000-0000-00000000c204"]
            .as_array()
            .expect("J2 silkscreen texts should be an array");
        assert!(
            j2_texts
                .iter()
                .any(|entry| entry["text"] == serde_json::Value::String("J2".to_string())),
            "KiCad-backed reference text should be materialized for J2"
        );
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
