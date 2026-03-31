use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};
use serde::de::{DeserializeOwned, Deserializer};
use serde::{Deserialize, Serialize};

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
    pub bounds: SceneBounds,
    pub layers: Vec<SceneLayer>,
    pub outline: Vec<OutlinePolyline>,
    pub components: Vec<ComponentBounds>,
    pub pads: Vec<PadPrimitive>,
    pub tracks: Vec<TrackPrimitive>,
    pub vias: Vec<ViaPrimitive>,
    pub zones: Vec<ZonePrimitive>,
    pub proposal_overlay_primitives: Vec<ProposalOverlayPrimitive>,
    pub review_primitives: Vec<ReviewPrimitive>,
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
    pub path: Vec<PointNm>,
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
    pub center: PointNm,
    pub bounds: RectNm,
    pub shape_kind: String,
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
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReviewWorkspaceState {
    pub scene: BoardReviewSceneV1,
    pub review: RouteProposalReviewPayload,
    pub selection: SelectionTarget,
    pub active_review_target_id: String,
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
    pub net_uuid: String,
    pub from_anchor_pad_uuid: String,
    pub to_anchor_pad_uuid: String,
    pub profile: Option<String>,
}

pub fn ensure_known_good_demo_request() -> Result<LiveReviewRequest> {
    let root = std::env::temp_dir().join("datum-gui-m7-known-good");
    write_known_good_demo_project(&root)?;
    Ok(LiveReviewRequest {
        project_root: root,
        net_uuid: "00000000-0000-0000-0000-00000000c200".to_string(),
        from_anchor_pad_uuid: "00000000-0000-0000-0000-00000000c205".to_string(),
        to_anchor_pad_uuid: "00000000-0000-0000-0000-00000000c206".to_string(),
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
    shape: String,
    diameter: i64,
    width: i64,
    height: i64,
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
                    { "x": 5000000, "y": 0 },
                    { "x": 5000000, "y": 3000000 },
                    { "x": 0, "y": 3000000 }
                ],
                "closed": true
            },
            "packages": {
                "00000000-0000-0000-0000-00000000c203": {
                    "uuid": "00000000-0000-0000-0000-00000000c203",
                    "package": "10000000-0000-0000-0000-00000000c203",
                    "part": "20000000-0000-0000-0000-00000000c203",
                    "reference": "U1",
                    "value": "SRC",
                    "position": { "x": 500000, "y": 600000 },
                    "rotation": 0,
                    "layer": 1,
                    "locked": false
                },
                "00000000-0000-0000-0000-00000000c204": {
                    "uuid": "00000000-0000-0000-0000-00000000c204",
                    "package": "10000000-0000-0000-0000-00000000c204",
                    "part": "20000000-0000-0000-0000-00000000c204",
                    "reference": "U2",
                    "value": "DST",
                    "position": { "x": 4500000, "y": 2400000 },
                    "rotation": 0,
                    "layer": 1,
                    "locked": false
                },
                "00000000-0000-0000-0000-00000000c208": {
                    "uuid": "00000000-0000-0000-0000-00000000c208",
                    "package": "10000000-0000-0000-0000-00000000c208",
                    "part": "20000000-0000-0000-0000-00000000c208",
                    "reference": "J1",
                    "value": "GND_HDR",
                    "position": { "x": 900000, "y": 2500000 },
                    "rotation": 0,
                    "layer": 1,
                    "locked": false
                },
                "00000000-0000-0000-0000-00000000c209": {
                    "uuid": "00000000-0000-0000-0000-00000000c209",
                    "package": "10000000-0000-0000-0000-00000000c209",
                    "part": "20000000-0000-0000-0000-00000000c209",
                    "reference": "TP1",
                    "value": "GND",
                    "position": { "x": 1550000, "y": 2650000 },
                    "rotation": 0,
                    "layer": 1,
                    "locked": false
                }
            },
            "pads": {
                "00000000-0000-0000-0000-00000000c205": {
                    "uuid": "00000000-0000-0000-0000-00000000c205",
                    "package": "00000000-0000-0000-0000-00000000c203",
                    "name": "1",
                    "net": "00000000-0000-0000-0000-00000000c200",
                    "position": { "x": 500000, "y": 600000 },
                    "layer": 1,
                    "shape": "circle",
                    "diameter": 450000,
                    "width": 0,
                    "height": 0
                },
                "00000000-0000-0000-0000-00000000c206": {
                    "uuid": "00000000-0000-0000-0000-00000000c206",
                    "package": "00000000-0000-0000-0000-00000000c204",
                    "name": "1",
                    "net": "00000000-0000-0000-0000-00000000c200",
                    "position": { "x": 4500000, "y": 2400000 },
                    "layer": 1,
                    "shape": "circle",
                    "diameter": 450000,
                    "width": 0,
                    "height": 0
                },
                "00000000-0000-0000-0000-00000000c20a": {
                    "uuid": "00000000-0000-0000-0000-00000000c20a",
                    "package": "00000000-0000-0000-0000-00000000c208",
                    "name": "1",
                    "net": "00000000-0000-0000-0000-00000000c201",
                    "position": { "x": 800000, "y": 2500000 },
                    "layer": 1,
                    "shape": "rect",
                    "diameter": 0,
                    "width": 320000,
                    "height": 320000
                },
                "00000000-0000-0000-0000-00000000c20b": {
                    "uuid": "00000000-0000-0000-0000-00000000c20b",
                    "package": "00000000-0000-0000-0000-00000000c208",
                    "name": "2",
                    "net": "00000000-0000-0000-0000-00000000c201",
                    "position": { "x": 1000000, "y": 2500000 },
                    "layer": 1,
                    "shape": "rect",
                    "diameter": 0,
                    "width": 320000,
                    "height": 320000
                },
                "00000000-0000-0000-0000-00000000c20c": {
                    "uuid": "00000000-0000-0000-0000-00000000c20c",
                    "package": "00000000-0000-0000-0000-00000000c209",
                    "name": "1",
                    "net": "00000000-0000-0000-0000-00000000c201",
                    "position": { "x": 1550000, "y": 2650000 },
                    "layer": 1,
                    "shape": "circle",
                    "diameter": 320000,
                    "width": 0,
                    "height": 0
                }
            },
            "tracks": {},
            "vias": {},
            "zones": {},
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
    Ok(())
}

fn write_json_file(path: &Path, value: &serde_json::Value) -> Result<()> {
    let payload = serde_json::to_string_pretty(value).context("failed to serialize demo JSON")?;
    std::fs::write(path, format!("{payload}\n"))
        .with_context(|| format!("failed to write {}", path.display()))
}

impl ReviewWorkspaceState {
    pub fn new(scene: BoardReviewSceneV1, review: RouteProposalReviewPayload) -> Self {
        let active_review_target_id = review
            .proposal_actions
            .first()
            .map(|action| action.action_id.clone())
            .unwrap_or_else(|| "no-proposal-action".to_string());
        Self {
            scene,
            review,
            selection: SelectionTarget::ReviewAction(active_review_target_id.clone()),
            active_review_target_id,
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
        let exists = self
            .scene
            .components
            .iter()
            .any(|c| c.object_id == object_id)
            || self.scene.pads.iter().any(|p| p.object_id == object_id)
            || self.scene.tracks.iter().any(|t| t.object_id == object_id)
            || self.scene.vias.iter().any(|v| v.object_id == object_id)
            || self.scene.zones.iter().any(|z| z.object_id == object_id);
        if exists {
            self.selection = SelectionTarget::AuthoredObject(object_id.to_string());
            true
        } else {
            false
        }
    }

    pub fn clear_selection(&mut self) {
        self.selection = SelectionTarget::None;
    }
}

pub fn load_live_workspace_state(request: &LiveReviewRequest) -> Result<ReviewWorkspaceState> {
    let cli = cli_prefix();
    let project_root = request.project_root.to_string_lossy().into_owned();
    let inspect: ProjectInspectPayload =
        run_cli_json(&cli, &["project", "inspect", &project_root])?;
    let review = load_live_route_review(&cli, request)?;
    let outline: OutlinePayload =
        run_cli_json(&cli, &["project", "query", &project_root, "board-outline"])?;
    let components: Vec<BoardComponentPayload> = run_cli_json(
        &cli,
        &["project", "query", &project_root, "board-components"],
    )?;
    let pads: Vec<BoardPadPayload> =
        run_cli_json(&cli, &["project", "query", &project_root, "board-pads"])?;
    let tracks: Vec<BoardTrackPayload> =
        run_cli_json(&cli, &["project", "query", &project_root, "board-tracks"])?;
    let vias: Vec<BoardViaPayload> =
        run_cli_json(&cli, &["project", "query", &project_root, "board-vias"])?;
    let zones: Vec<BoardZonePayload> =
        run_cli_json(&cli, &["project", "query", &project_root, "board-zones"])?;

    let mut scene =
        build_board_review_scene(&inspect, outline, components, pads, tracks, vias, zones);
    attach_review_primitives(&mut scene, &review);
    Ok(ReviewWorkspaceState::new(scene, review))
}

fn load_live_route_review(
    cli: &[String],
    request: &LiveReviewRequest,
) -> Result<RouteProposalReviewPayload> {
    let project_root = request.project_root.to_string_lossy().into_owned();
    let mut args = vec![
        "project".to_string(),
        "review-route-proposal".to_string(),
        project_root,
        "--net".to_string(),
        request.net_uuid.clone(),
        "--from-anchor".to_string(),
        request.from_anchor_pad_uuid.clone(),
        "--to-anchor".to_string(),
        request.to_anchor_pad_uuid.clone(),
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
    pads: Vec<BoardPadPayload>,
    tracks: Vec<BoardTrackPayload>,
    vias: Vec<BoardViaPayload>,
    zones: Vec<BoardZonePayload>,
) -> BoardReviewSceneV1 {
    let layer_ids = collect_layer_ids(&components, &pads, &tracks, &vias, &zones);
    let pads_by_component = pads.iter().fold(
        BTreeMap::<String, Vec<&BoardPadPayload>>::new(),
        |mut acc, pad| {
            acc.entry(pad.package.clone()).or_default().push(pad);
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
        .map(|pad| PadPrimitive {
            object_id: format!("pad:{}", pad.uuid),
            object_kind: "pad".to_string(),
            source_object_uuid: pad.uuid.clone(),
            pad_uuid: pad.uuid.clone(),
            component_uuid: pad.package.clone(),
            net_uuid: pad.net.clone(),
            layer_id: layer_id(pad.layer),
            center: pad.position,
            bounds: pad_bounds(&pad),
            shape_kind: pad.shape,
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
        bounds,
        layers: layer_ids
            .into_iter()
            .enumerate()
            .map(|(render_order, layer)| SceneLayer {
                layer_id: layer.clone(),
                name: layer.clone(),
                kind: "copper".to_string(),
                render_order: render_order as u32,
                visible_by_default: true,
            })
            .collect(),
        outline: vec![OutlinePolyline {
            object_id: format!("outline:{}", inspect.board_uuid),
            object_kind: "outline".to_string(),
            source_object_uuid: inspect.board_uuid.clone(),
            path: outline_path,
        }],
        components,
        pads,
        tracks,
        vias,
        zones,
        proposal_overlay_primitives: Vec::new(),
        review_primitives: Vec::new(),
    }
}

fn attach_review_primitives(scene: &mut BoardReviewSceneV1, review: &RouteProposalReviewPayload) {
    let first_action_id = review
        .proposal_actions
        .first()
        .map(|action| action.action_id.as_str());
    scene.proposal_overlay_primitives = review
        .proposal_actions
        .iter()
        .map(|action| ProposalOverlayPrimitive {
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
            path: vec![action.from, action.to],
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
                path: vec![first.from],
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
                path: vec![last.to],
            });
    }
    let mut seen_segments = BTreeSet::new();
    scene.review_primitives = review
        .proposal_actions
        .iter()
        .filter(|action| seen_segments.insert(action.selected_path_segment_index))
        .map(|action| ReviewPrimitive {
            review_primitive_id: format!(
                "review:segment-{}",
                action.selected_path_segment_index + 1
            ),
            primitive_kind: "selected_segment_highlight".to_string(),
            evidence_key: Some(format!("segment:{}", action.selected_path_segment_index)),
            path: vec![action.from, action.to],
        })
        .collect();
}

fn run_cli_json<T: DeserializeOwned>(cli_prefix: &[String], args: &[&str]) -> Result<T> {
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
    vec![
        "cargo".to_string(),
        "run".to_string(),
        "--quiet".to_string(),
        "-p".to_string(),
        "eda-cli".to_string(),
        "--bin".to_string(),
        "eda".to_string(),
        "--".to_string(),
    ]
}

fn collect_layer_ids(
    components: &[BoardComponentPayload],
    pads: &[BoardPadPayload],
    tracks: &[BoardTrackPayload],
    vias: &[BoardViaPayload],
    zones: &[BoardZonePayload],
) -> Vec<String> {
    let mut layers = BTreeSet::new();
    for component in components {
        layers.insert(layer_id(component.layer));
    }
    for pad in pads {
        layers.insert(layer_id(pad.layer));
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
    if layers.is_empty() {
        layers.insert(layer_id(0));
    }
    layers.into_iter().collect()
}

fn layer_id(layer: i32) -> String {
    format!("L{}", layer)
}

fn component_bounds(component: &BoardComponentPayload, pads: &[&BoardPadPayload]) -> RectNm {
    if pads.is_empty() {
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
    let margin = 250_000;
    RectNm {
        min_x: rect.min_x - margin,
        min_y: rect.min_y - margin,
        max_x: rect.max_x + margin,
        max_y: rect.max_y + margin,
    }
}

fn pad_bounds(pad: &BoardPadPayload) -> RectNm {
    let half_width = match pad.shape.as_str() {
        "rect" => (pad.width.max(1)) / 2,
        _ => (pad.diameter.max(1)) / 2,
    };
    let half_height = match pad.shape.as_str() {
        "rect" => (pad.height.max(1)) / 2,
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
    tracks: impl Iterator<Item = &'a PointNm>,
    vias: impl Iterator<Item = &'a PointNm>,
    zones: impl Iterator<Item = &'a PointNm>,
) -> SceneBounds {
    let mut points: Vec<PointNm> = Vec::new();
    points.extend(outline.copied());
    points.extend(components.copied());
    points.extend(pads.copied());
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
    fn attach_review_primitives_builds_overlay_from_review_payload() {
        let mut scene: BoardReviewSceneV1 =
            serde_json::from_str(include_str!("../testdata/board_review_scene_v1.json"))
                .expect("scene fixture should decode");
        scene.proposal_overlay_primitives.clear();
        scene.review_primitives.clear();
        let review: RouteProposalReviewPayload =
            serde_json::from_str(include_str!("../testdata/review_route_proposal.json"))
                .expect("review fixture should decode");

        attach_review_primitives(&mut scene, &review);

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
            vec![BoardPadPayload {
                uuid: "P1".to_string(),
                package: "U1".to_string(),
                name: "1".to_string(),
                net: Some("net-1".to_string()),
                position: PointNm { x: 40, y: 50 },
                layer: 0,
                shape: "rect".to_string(),
                diameter: 0,
                width: 10,
                height: 10,
            }],
            vec![],
            vec![],
            vec![],
        );
        assert_eq!(scene.components.len(), 1);
        assert!(scene.components[0].bounds.min_x < 40);
        assert_eq!(scene.board_uuid, "board-1");
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
            shape: "circle".to_string(),
            diameter: 450_000,
            width: 0,
            height: 0,
        };
        let bounds = component_bounds(&component, &[&pad]);
        assert_eq!(bounds.min_x, 525_000);
        assert_eq!(bounds.min_y, 525_000);
        assert_eq!(bounds.max_x, 1_475_000);
        assert_eq!(bounds.max_y, 1_475_000);
    }

    #[test]
    fn known_good_demo_request_materializes_project_scaffold() {
        let request = ensure_known_good_demo_request().expect("demo request should materialize");
        assert!(request.project_root.join("project.json").is_file());
        assert!(request.project_root.join("board/board.json").is_file());
        assert_eq!(request.net_uuid, "00000000-0000-0000-0000-00000000c200");
        assert_eq!(
            request.from_anchor_pad_uuid,
            "00000000-0000-0000-0000-00000000c205"
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
    }
}
