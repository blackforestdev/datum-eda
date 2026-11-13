//! Explicit PCB clearance setup through the existing canonical project rule verb.
use super::*;
use eda_engine::api::native_write::{WriteProvenance, project::build_create_project_rule};
use eda_engine::substrate::{CommitSource, DesignModel, SourceShardKind};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BoardClearanceRule {
    pub project: Uuid,
    pub revision: String,
    pub tip: Option<Uuid>,
    pub rule: Uuid,
    pub net: Uuid,
    pub minimum_nm: i64,
    pub priority: u64,
}
impl BoardClearanceRule {
    pub fn capture(model: &DesignModel, net: Uuid, minimum_nm: i64, priority: u64) -> Result<Self> {
        anyhow::ensure!(minimum_nm > 0, "Clearance must be positive");
        let shard = model
            .source_shards
            .iter()
            .find(|s| s.kind == SourceShardKind::BoardRoot)
            .context("Captured Board source unavailable")?;
        let board = model.capture_source_shard_value(shard.shard_id)?;
        let source: eda_engine::board::Net =
            serde_json::from_value(board["nets"][net.to_string()].clone())?;
        anyhow::ensure!(
            source.uuid == net && !net.is_nil(),
            "Choose an existing Net"
        );
        let request = Self {
            project: model.project.project_id,
            revision: model.model_revision.0.clone(),
            tip: eda_engine::revision::transaction_tip(&model.journal),
            rule: Uuid::new_v4(),
            net,
            minimum_nm,
            priority,
        };
        let rules = model
            .source_shards
            .iter()
            .find(|s| s.kind == SourceShardKind::RulesRoot)
            .context("Captured Rules source unavailable")?;
        model.capture_source_shard_value(rules.shard_id)?;
        build_create_project_rule(
            model,
            WriteProvenance::new(
                "datum-gui",
                CommitSource::Manual,
                "Manual PCB clearance setup",
            ),
            request.value()?,
        )?;
        Ok(request)
    }
    pub fn value(&self) -> Result<Value> {
        anyhow::ensure!(
            self.minimum_nm > 0 && !self.net.is_nil() && !self.rule.is_nil(),
            "Clearance requires a positive minimum and explicit Net"
        );
        Ok(
            json!({"uuid":self.rule,"name":format!("PCB clearance {}",self.net),
            "scope":{"Net":self.net},"enabled":true,"priority":self.priority,
            "rule_type":"clearance_copper","params":{"min_nm":self.minimum_nm}}),
        )
    }
}
impl NativeEditIntent {
    pub(super) fn validate_board_rule_payload(&self) -> Result<()> {
        if !matches!(self.action, NativeEditAction::CreateBoardClearance) {
            anyhow::ensure!(
                self.clearance_rule.is_none(),
                "Rule payload does not match edit action"
            );
            return Ok(());
        }
        let rule = self
            .clearance_rule
            .as_ref()
            .context("Clearance rule payload missing")?;
        anyhow::ensure!(
            rule.project == self.project_id
                && rule.revision == self.expected_revision
                && rule.tip == self.expected_tip,
            "Clearance rule source basis changed"
        );
        anyhow::ensure!(
            self.board_route.is_none()
                && self.board_zone.is_none()
                && self.board_transfer.is_none()
                && self.library.is_none()
                && self.library_acquisition.is_none()
                && self.schematic.is_none()
                && self.proposal.is_none()
                && self.symbol_group.is_none()
                && self.symbol_alignment.is_none()
                && self.instance_path.is_empty(),
            "Clearance intent contains another editor's payload"
        );
        rule.value()?;
        Ok(())
    }
}
