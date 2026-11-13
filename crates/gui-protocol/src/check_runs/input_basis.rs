//! One currentness predicate for persisted and live native Check projections.
//! No frontend hashing and no qualification inferred from a missing receipt.
use super::producer::{CheckProducerAcceptance, CheckProducerProfile, CheckProducerRequest};
use super::*;
use crate::ReviewWorkspaceState;
use uuid::Uuid;

impl CheckRunReviewState {
    pub fn qualification(&self, state: &ReviewWorkspaceState) -> Result<(), String> {
        if self.project_id.as_deref() != Some(state.scene.project_uuid.as_str())
            || self.model_revision.as_deref() != Some(state.scene.source_revision.as_str())
        {
            return Err("Source changed; rerun checks on the current design".into());
        }
        if self.profile_id.as_deref() == Some("drc") {
            let model = state
                .library
                .basis
                .as_ref()
                .ok_or("Native DRC zone-copper basis unavailable; refresh the design")?;
            if model.project.project_id.to_string() != state.scene.project_uuid
                || model.model_revision.0 != state.scene.source_revision
            {
                return Err("Native source view is stale; refresh the design".into());
            }
            model
                .qualify_native_drc_zone_copper_basis(self.native_drc_zone_copper_basis.as_ref())?;
        }
        Ok(())
    }

    pub fn is_current(&self, state: &ReviewWorkspaceState) -> bool {
        self.qualification(state).is_ok()
    }
}

impl CheckRunPayload {
    pub(super) fn into_state(
        self,
        fallback_project_id: Option<String>,
        fallback_model_revision: Option<String>,
    ) -> CheckRunReviewState {
        let finding_count = if self.finding_count == 0 {
            self.findings.len()
        } else {
            self.finding_count
        };
        CheckRunReviewState {
            check_run_id: self.check_run_id,
            project_id: self.project_id.or(fallback_project_id),
            model_revision: self.model_revision.or(fallback_model_revision),
            profile_id: self.profile_id,
            status: self.status,
            persisted: self.persisted,
            summary: self.summary,
            finding_count,
            findings: self.findings,
            proposal_refs: self.proposal_refs,
            proposal_links: self.proposal_links,
            profile_basis: self.profile_basis,
            coverage: self.coverage,
            native_drc_zone_copper_basis: self
                .raw_report
                .get(eda_engine::substrate::CheckRun::NATIVE_DRC_ZONE_COPPER_BASIS_KEY)
                .cloned(),
        }
    }
}

// Requests and readback share this same source and input qualification owner.
impl CheckProducerRequest {
    pub(super) fn source_basis(
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
            "Guarded checks require a native Project"
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

    pub fn for_profile(
        state: &ReviewWorkspaceState,
        profile: CheckProducerProfile,
    ) -> anyhow::Result<Self> {
        let (root, model) = Self::source_basis(state)?;
        Ok(Self {
            profile,
            job_id: Uuid::new_v4(),
            project_id: model.project.project_id,
            model_revision: model.model_revision.0.clone(),
            project_root: root.to_owned(),
            zone_copper_fingerprint: if profile == CheckProducerProfile::Drc {
                Some(model.native_drc_zone_copper_fingerprint()?)
            } else {
                None
            },
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
                    && (self.profile != CheckProducerProfile::Drc
                        || model.native_drc_zone_copper_fingerprint().ok().as_ref()
                            == self.zone_copper_fingerprint.as_ref())
            })
    }

    pub fn acceptance(&self) -> CheckProducerAcceptance {
        CheckProducerAcceptance {
            job_id: self.job_id,
            project_id: self.project_id,
            model_revision: self.model_revision.clone(),
            profile_id: self.profile.id().into(),
            zone_copper_fingerprint: self.zone_copper_fingerprint.clone(),
        }
    }

    pub fn admits_run(&self, run: &CheckRunReviewState) -> bool {
        run.persisted
            && run.project_id.as_deref() == Some(self.project_id.to_string().as_str())
            && run.model_revision.as_deref() == Some(self.model_revision.as_str())
            && run.profile_id.as_deref() == Some(self.profile.id())
            && (self.profile != CheckProducerProfile::Drc
                || self
                    .zone_copper_fingerprint
                    .as_ref()
                    .is_some_and(|expected| {
                        eda_engine::substrate::CheckRun::native_drc_zone_copper_basis_fingerprint(
                            run.native_drc_zone_copper_basis.as_ref(),
                        )
                        .is_ok_and(|actual| &actual == expected)
                    }))
            && run
                .check_run_id
                .as_deref()
                .is_some_and(|id| Uuid::parse_str(id).is_ok())
    }

    pub(super) fn validate_input_basis(&self) -> anyhow::Result<()> {
        match self.profile {
            CheckProducerProfile::Erc => anyhow::ensure!(
                self.zone_copper_fingerprint.is_none(),
                "ERC request contains a DRC zone-copper basis"
            ),
            CheckProducerProfile::Drc => {
                let fingerprint = self.zone_copper_fingerprint.as_deref().ok_or_else(|| {
                    anyhow::anyhow!(
                        "Native DRC zone-copper basis unavailable; refresh before running DRC"
                    )
                })?;
                eda_engine::substrate::CheckRun::validate_native_drc_zone_copper_fingerprint(
                    fingerprint,
                )
                .map_err(anyhow::Error::msg)?;
            }
        }
        Ok(())
    }
}
