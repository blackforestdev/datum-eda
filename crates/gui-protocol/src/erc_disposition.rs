//! Current ERC evidence configures the shared Proposal review, never a direct waiver writer.
use crate::{CheckFindingSummary, ReviewWorkspaceState, SelectionTarget};
use eda_engine::api::native_write::schematic_delete_proposal::{
    DispositionKind, DispositionSummary, NativeProposalAction,
};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq)]
pub struct DispositionEvidence {
    pub check_run_id: Uuid,
    pub finding_id: Uuid,
    pub fingerprint: String,
    pub rule_id: String,
    pub message: String,
    pub disposition: DispositionKind,
    pub rationale: String,
}
impl DispositionEvidence {
    pub fn capture(
        state: &ReviewWorkspaceState,
        disposition: DispositionKind,
    ) -> Result<Self, &'static str> {
        let fingerprint = selected_fingerprint(state)?;
        let finding = current_finding(state, fingerprint)?;
        Ok(Self {
            check_run_id: state
                .checks
                .check_run_id
                .as_ref()
                .and_then(|id| id.parse().ok())
                .ok_or("This finding has no persisted CheckRun identity")?,
            finding_id: finding
                .finding_id
                .as_ref()
                .and_then(|id| id.parse().ok())
                .ok_or("This finding has no canonical finding identity")?,
            fingerprint: fingerprint.to_owned(),
            rule_id: finding.rule_id.clone(),
            message: finding.message.clone(),
            disposition,
            rationale: String::new(),
        })
    }
    pub fn current(&self, state: &ReviewWorkspaceState) -> bool {
        state
            .checks
            .check_run_id
            .as_deref()
            .and_then(|id| id.parse().ok())
            == Some(self.check_run_id)
            && current_finding(state, &self.fingerprint).is_ok_and(|f| {
                f.finding_id.as_deref().and_then(|id| id.parse().ok()) == Some(self.finding_id)
                    && f.rule_id == self.rule_id
                    && f.message == self.message
            })
    }
    pub fn label(&self) -> &'static str {
        match self.disposition {
            DispositionKind::Waiver => "Waiver",
            DispositionKind::Deviation => "Accepted deviation",
        }
    }
    pub fn cycle(&mut self) {
        self.disposition = match self.disposition {
            DispositionKind::Waiver => DispositionKind::Deviation,
            DispositionKind::Deviation => DispositionKind::Waiver,
        };
    }
    pub fn action(
        &mut self,
        proposal_id: Uuid,
        rationale: &str,
    ) -> Result<NativeProposalAction, String> {
        if rationale.trim().is_empty() || rationale.chars().any(char::is_control) {
            return Err("Enter a nonblank, single-line rationale".into());
        }
        self.rationale = rationale.to_owned();
        Ok(NativeProposalAction::CreateDisposition {
            proposal_id,
            check_run_id: self.check_run_id,
            fingerprint: self.fingerprint.clone(),
            disposition: self.disposition,
            rationale: self.rationale.clone(),
        })
    }
    pub fn matches(&self, summary: &DispositionSummary) -> bool {
        summary.check_run_id == self.check_run_id
            && summary.finding_id == self.finding_id
            && summary.fingerprint == self.fingerprint
            && summary.disposition == self.disposition
            && summary.rationale == self.rationale
            && summary.rule_id == self.rule_id
            && summary.message == self.message
    }
}

/// Native finding clicks retain a Diagnostic subject, never an authored object.
fn selected_fingerprint(state: &ReviewWorkspaceState) -> Result<&str, &'static str> {
    match &state.selection {
        SelectionTarget::CheckFinding(fingerprint) => Ok(fingerprint),
        SelectionTarget::Native(view) => {
            let crate::SelectionSubject::Diagnostic(fingerprint) = &view.subject else {
                return Err("Select one current ERC finding first");
            };
            if view.stale
                || !view.members.is_empty()
                || view.project.to_string() != state.scene.project_uuid
                || view.revision.0 != state.scene.source_revision
            {
                return Err("Finding selection changed; select the current ERC finding again");
            }
            let Some(crate::selection_view::InspectionValue::Common(basis)) =
                view.field_ref("artifact_basis")
            else {
                return Err("Finding selection has no producing CheckRun basis");
            };
            if basis["check_run_id"].as_str() != state.checks.check_run_id.as_deref()
                || basis["model_revision"].as_str() != Some(state.scene.source_revision.as_str())
            {
                return Err("Finding CheckRun changed; select the current ERC finding again");
            }
            Ok(fingerprint)
        }
        _ => Err("Select one current ERC finding first"),
    }
}

fn current_finding<'a>(
    state: &'a ReviewWorkspaceState,
    fingerprint: &str,
) -> Result<&'a CheckFindingSummary, &'static str> {
    let run = &state.checks;
    if !run.persisted
        || run.project_id.as_deref() != Some(state.scene.project_uuid.as_str())
        || run.model_revision.as_deref() != Some(state.scene.source_revision.as_str())
    {
        return Err("Run ERC at the current Project revision before recording a disposition");
    }
    let mut matching = run.findings.iter().filter(|f| f.fingerprint == fingerprint);
    let finding = matching
        .next()
        .ok_or("This finding is no longer available")?;
    if matching.next().is_some() || fingerprint.is_empty() {
        return Err("This finding fingerprint is not unique");
    }
    if finding.domain != "erc"
        || finding.status != "active"
        || !finding.waiver_refs.is_empty()
        || !finding.deviation_refs.is_empty()
    {
        return Err("Choose an open ERC finding with no existing disposition");
    }
    Ok(finding)
}
