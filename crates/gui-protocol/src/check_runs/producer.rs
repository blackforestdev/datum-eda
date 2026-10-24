//! A check job is explicitly requested and source-qualified; inspection never starts one.
use super::CheckRunReviewState;
use crate::ReviewWorkspaceState;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;
mod acceptance_stream;
mod worker;
pub use worker::run;

/// An acknowledged successful child supplied this output. It is deliberately
/// not a completion result and exposes no run to the GUI until the canonical
/// record, source and journal snapshot have been read back under exclusion.
pub struct CheckProducerCandidate {
    pub(crate) run: CheckRunReviewState,
}
impl CheckProducerCandidate {
    /// Admit process output only; persistence is verified by `readback`.
    pub fn produce(
        request: &CheckProducerRequest,
        accepted: impl FnMut(CheckProducerAcceptance),
    ) -> anyhow::Result<Self> {
        let cli = crate::cli_bridge::cli_prefix()?;
        worker::produce_candidate_with_cli(&cli, request, accepted)
    }

    pub fn readback(
        self,
        previous: ReviewWorkspaceState,
        request: &CheckProducerRequest,
    ) -> anyhow::Result<ReviewWorkspaceState> {
        crate::workspace_loading::load_check_completion_workspace(previous, request, &self.run)
    }
}

pub const ACCEPTED_PREFIX: &str = "DATUM_CHECK_JOB_ACCEPTED ";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CheckProducerRequest {
    pub job_id: Uuid,
    pub project_id: Uuid,
    pub model_revision: String,
    pub project_root: PathBuf,
}

impl CheckProducerRequest {
    fn source_basis(
        state: &ReviewWorkspaceState,
    ) -> anyhow::Result<(&std::path::Path, &eda_engine::substrate::DesignModel)> {
        anyhow::ensure!(
            !state.project_unopened,
            "Open a native Project before running checks"
        );
        let backing = state
            .backing
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Native Project source unavailable"))?;
        anyhow::ensure!(
            backing.request.board_file.is_none(),
            "Guarded ERC requires a native Project"
        );
        let model = state
            .library
            .basis
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Current Project basis unavailable"))?;
        anyhow::ensure!(
            state.scene.project_uuid == model.project.project_id.to_string()
                && state.scene.source_revision == model.model_revision.0,
            "Project view basis is stale"
        );
        Ok((&backing.request.project_root, model))
    }

    pub fn workspace_available(state: &ReviewWorkspaceState) -> bool {
        Self::source_basis(state).is_ok()
    }

    pub fn from_workspace(state: &ReviewWorkspaceState) -> anyhow::Result<Self> {
        let (root, model) = Self::source_basis(state)?;
        Ok(Self {
            job_id: Uuid::new_v4(),
            project_id: model.project.project_id,
            model_revision: model.model_revision.0.clone(),
            project_root: root.to_owned(),
        })
    }

    pub fn matches_workspace(&self, state: &ReviewWorkspaceState) -> bool {
        state
            .backing
            .as_ref()
            .is_some_and(|backing| backing.request.project_root == self.project_root)
            && !state.project_unopened
            && state.scene.project_uuid == self.project_id.to_string()
            && state.scene.source_revision == self.model_revision
            && state.library.basis.as_ref().is_some_and(|model| {
                model.project.project_id == self.project_id
                    && model.model_revision.0 == self.model_revision
            })
    }

    pub fn acceptance(&self) -> CheckProducerAcceptance {
        CheckProducerAcceptance {
            job_id: self.job_id,
            project_id: self.project_id,
            model_revision: self.model_revision.clone(),
            profile_id: "erc".into(),
        }
    }

    pub fn admits_run(&self, run: &CheckRunReviewState) -> bool {
        run.persisted
            && run.project_id.as_deref() == Some(self.project_id.to_string().as_str())
            && run.model_revision.as_deref() == Some(self.model_revision.as_str())
            && run.profile_id.as_deref() == Some("erc")
            && run
                .check_run_id
                .as_deref()
                .is_some_and(|id| Uuid::parse_str(id).is_ok())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckProducerAcceptance {
    pub job_id: Uuid,
    pub project_id: Uuid,
    pub model_revision: String,
    pub profile_id: String,
}

impl CheckProducerAcceptance {
    /// The producer emits one canonical JSON value with four fixed typed fields.
    /// Its maximum wire length is derived from this exact requested value, not
    /// an arbitrary output or performance limit. Other stderr is not authority.
    pub fn decode(line: &str, request: &CheckProducerRequest) -> anyhow::Result<Option<Self>> {
        let Some(value) = line.strip_prefix(ACCEPTED_PREFIX) else {
            return Ok(None);
        };
        let expected = request.acceptance();
        anyhow::ensure!(
            value.len() <= serde_json::to_vec(&expected)?.len(),
            "Oversized check job acknowledgement"
        );
        let actual: Self = serde_json::from_str(value)?;
        anyhow::ensure!(
            actual == expected,
            "Foreign or stale check job acknowledgement"
        );
        Ok(Some(actual))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum CheckProducerPhase {
    #[default]
    Idle,
    Starting,
    Running,
    Complete,
    Unavailable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct CheckProducerState {
    pub request: Option<CheckProducerRequest>,
    pub phase: CheckProducerPhase,
    pub reason: Option<String>,
    pub accepted: Option<CheckProducerAcceptance>,
}

impl CheckProducerState {
    pub fn begin(&mut self, request: CheckProducerRequest) {
        *self = Self {
            request: Some(request),
            phase: CheckProducerPhase::Starting,
            reason: None,
            accepted: None,
        };
    }

    pub fn accept(&mut self, accepted: CheckProducerAcceptance) -> bool {
        if self.phase != CheckProducerPhase::Starting
            || self
                .request
                .as_ref()
                .is_none_or(|request| request.acceptance() != accepted)
        {
            return false;
        }
        self.accepted = Some(accepted);
        self.phase = CheckProducerPhase::Running;
        true
    }

    pub fn refuse(&mut self, reason: String) {
        self.phase = CheckProducerPhase::Unavailable;
        self.reason = Some(reason);
    }

    pub fn complete(&mut self, request: &CheckProducerRequest, run: &CheckRunReviewState) -> bool {
        if self.request.as_ref() != Some(request)
            || self.phase != CheckProducerPhase::Running
            || self.accepted.as_ref() != Some(&request.acceptance())
            || !request.admits_run(run)
        {
            return false;
        }
        self.phase = CheckProducerPhase::Complete;
        self.reason = None;
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starting_is_not_running_and_only_exact_producer_acceptance_advances() {
        let request = CheckProducerRequest {
            job_id: Uuid::new_v4(),
            project_id: Uuid::new_v4(),
            model_revision: "c".repeat(64),
            project_root: PathBuf::from("/owned/native-project"),
        };
        let mut state = CheckProducerState::default();
        state.begin(request.clone());
        assert_eq!(state.phase, CheckProducerPhase::Starting);
        assert!(state.accepted.is_none());
        let mut foreign = request.acceptance();
        foreign.job_id = Uuid::new_v4();
        assert!(!state.accept(foreign));
        assert_eq!(state.phase, CheckProducerPhase::Starting);
        let line = format!(
            "{ACCEPTED_PREFIX}{}",
            serde_json::to_string(&request.acceptance()).unwrap()
        );
        let actual = CheckProducerAcceptance::decode(&line, &request)
            .unwrap()
            .unwrap();
        assert!(state.accept(actual));
        assert_eq!(state.phase, CheckProducerPhase::Running);
        assert!(!state.accept(request.acceptance()));
        let mut run = CheckRunReviewState {
            persisted: true,
            project_id: Some(request.project_id.to_string()),
            model_revision: Some(request.model_revision.clone()),
            profile_id: Some("erc".into()),
            check_run_id: Some(Uuid::new_v4().to_string()),
            ..Default::default()
        };
        run.model_revision = Some("stale".into());
        assert!(!state.complete(&request, &run));
        run.model_revision = Some(request.model_revision.clone());
        assert!(state.complete(&request, &run));
        assert_eq!(state.phase, CheckProducerPhase::Complete);
    }

    #[test]
    fn malformed_foreign_and_unrelated_process_text_never_acknowledges_a_job() {
        let request = CheckProducerRequest {
            job_id: Uuid::new_v4(),
            project_id: Uuid::new_v4(),
            model_revision: "d".repeat(64),
            project_root: PathBuf::from("/owned/native-project"),
        };
        assert!(
            CheckProducerAcceptance::decode("terminal session Running", &request)
                .unwrap()
                .is_none()
        );
        assert!(
            CheckProducerAcceptance::decode(&format!("{ACCEPTED_PREFIX}{{}}"), &request).is_err()
        );
        let mut foreign = request.acceptance();
        foreign.project_id = Uuid::new_v4();
        let line = format!(
            "{ACCEPTED_PREFIX}{}",
            serde_json::to_string(&foreign).unwrap()
        );
        assert!(CheckProducerAcceptance::decode(&line, &request).is_err());
        let mut stale = request.acceptance();
        stale.model_revision = "e".repeat(64);
        let line = format!(
            "{ACCEPTED_PREFIX}{}",
            serde_json::to_string(&stale).unwrap()
        );
        assert!(CheckProducerAcceptance::decode(&line, &request).is_err());
    }
}
