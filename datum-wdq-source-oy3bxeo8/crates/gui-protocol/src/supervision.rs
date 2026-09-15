//! Stable GUI supervision snapshot types.
//!
//! The snapshot is a read-only projection of resolver, journal, scene, check,
//! and production truth. Keeping its schema together prevents the workspace
//! protocol root from owning every supervision sub-domain.

use serde::Serialize;

use crate::{ConsoleJournalProjectionRecord, SourceShardStatusSummary};

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
    pub projection: Vec<ConsoleJournalProjectionRecord>,
    pub projection_omitted_count: usize,
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
