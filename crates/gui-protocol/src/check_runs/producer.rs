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

/// The two native profiles supported by the shared guarded producer. A profile
/// is part of job identity, never inferred from whichever pane has focus.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CheckProducerProfile {
    #[default]
    Erc,
    Drc,
}
impl CheckProducerProfile {
    pub fn id(self) -> &'static str {
        match self {
            Self::Erc => "erc",
            Self::Drc => "drc",
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::Erc => "ERC",
            Self::Drc => "DRC",
        }
    }
    pub fn from_id(id: &str) -> Option<Self> {
        match id {
            "erc" => Some(Self::Erc),
            "drc" => Some(Self::Drc),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CheckProducerRequest {
    #[serde(default)]
    pub profile: CheckProducerProfile,
    pub job_id: Uuid,
    pub project_id: Uuid,
    pub model_revision: String,
    pub project_root: PathBuf,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub zone_copper_fingerprint: Option<String>,
}

impl CheckProducerRequest {
    pub fn workspace_available(state: &ReviewWorkspaceState) -> bool {
        Self::source_basis(state).is_ok()
    }

    /// One preview admission rule for the menu, Inspector and actual producer.
    /// Native worker/project-open admission remains with the runtime job owner.
    pub fn editor_preview_idle(state: &ReviewWorkspaceState) -> bool {
        !state.schematic_authoring.active()
            && !state.schematic_authoring.picker
            && state.schematic_authoring.delete_review.is_none()
            && state.library.draft.is_none()
            && !state.authoring.gesture.is_active()
            && state.authoring.board_route.is_none()
            && state.authoring.board_zone.is_none()
    }

    pub fn from_workspace(state: &ReviewWorkspaceState) -> anyhow::Result<Self> {
        Self::for_profile(state, CheckProducerProfile::Erc)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckProducerAcceptance {
    pub job_id: Uuid,
    pub project_id: Uuid,
    pub model_revision: String,
    pub profile_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub zone_copper_fingerprint: Option<String>,
}

impl CheckProducerAcceptance {
    /// ERC retains its four fixed fields; native DRC also binds zone copper.
    /// Its maximum wire length is derived from this exact requested value, not
    /// an arbitrary output or performance limit. Other stderr is not authority.
    pub fn decode(line: &str, request: &CheckProducerRequest) -> anyhow::Result<Option<Self>> {
        let Some(value) = line.strip_prefix(ACCEPTED_PREFIX) else {
            return Ok(None);
        };
        request.validate_input_basis()?;
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
    fn drc_job_never_accepts_erc_acknowledgement_or_result() {
        let request = CheckProducerRequest {
            profile: CheckProducerProfile::Drc,
            job_id: Uuid::new_v4(),
            project_id: Uuid::new_v4(),
            model_revision: "current".into(),
            project_root: "/native".into(),
            zone_copper_fingerprint: Some(format!("sha256:{}", "a".repeat(64))),
        };
        let mut state = CheckProducerState::default();
        state.begin(request.clone());
        let mut wrong = request.acceptance();
        wrong.profile_id = "erc".into();
        assert!(!state.accept(wrong));
        assert_eq!(state.phase, CheckProducerPhase::Starting);
        assert!(state.accept(request.acceptance()));
        let mut run = CheckRunReviewState {
            persisted: true,
            project_id: Some(request.project_id.to_string()),
            model_revision: Some(request.model_revision.clone()),
            profile_id: Some("erc".into()),
            check_run_id: Some(Uuid::new_v4().to_string()),
            ..Default::default()
        };
        assert!(!state.complete(&request, &run));
        run.profile_id = Some("drc".into());
        assert!(
            !state.complete(&request, &run),
            "Legacy DRC lacks the actual input receipt"
        );
        run.native_drc_zone_copper_basis = Some(
            serde_json::json!({"version":2,"scope":"resolved_zone_fills","fingerprint":request.zone_copper_fingerprint}),
        );
        assert!(!state.complete(&request, &run));
        run.native_drc_zone_copper_basis = Some(
            serde_json::json!({"version":1,"scope":"resolved_zone_fills","fingerprint":request.zone_copper_fingerprint}),
        );
        assert!(state.complete(&request, &run));
    }

    #[test]
    fn starting_is_not_running_and_only_exact_producer_acceptance_advances() {
        let request = CheckProducerRequest {
            profile: CheckProducerProfile::Erc,
            job_id: Uuid::new_v4(),
            project_id: Uuid::new_v4(),
            model_revision: "c".repeat(64),
            project_root: PathBuf::from("/owned/native-project"),
            zone_copper_fingerprint: None,
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
            profile: CheckProducerProfile::Erc,
            job_id: Uuid::new_v4(),
            project_id: Uuid::new_v4(),
            model_revision: "d".repeat(64),
            project_root: PathBuf::from("/owned/native-project"),
            zone_copper_fingerprint: None,
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
