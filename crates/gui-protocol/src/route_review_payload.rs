//! Producing route artifact/action basis shared by every workspace consumer.
use crate::{PointNm, deserialize_null_default};
use serde::{Deserialize, Serialize};

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
    #[serde(default)]
    pub producer_revision: Option<String>,
    #[serde(default)]
    pub source_fingerprint: Option<String>,
    #[serde(default)]
    pub producer_proposal: Option<eda_engine::substrate::Proposal>,
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
