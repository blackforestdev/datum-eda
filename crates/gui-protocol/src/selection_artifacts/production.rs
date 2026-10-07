//! Canonical whole-batch producer authority, independent of route-action keys.
use super::*;
use serde::{Deserialize, Serialize};
/// Existing canonical producer fields; never an authored or scene-generated ID.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProductionProposalActionIdentity {
    pub project_id: uuid::Uuid,
    pub proposal_id: uuid::Uuid,
    pub batch_id: uuid::Uuid,
    pub prepared_against: eda_engine::substrate::ModelRevision,
}
impl ProductionProposalActionIdentity {
    pub fn selection_key(&self) -> String {
        serde_json::to_string(self).expect("typed producer identity")
    }
}

use eda_engine::substrate::{PROPOSAL_SCHEMA_VERSION, Proposal, ProposalStatus};

impl WorkspaceSelectionAuthority<'_> {
    pub(super) fn production_proposal(
        &self,
        identity: &ProductionProposalActionIdentity,
    ) -> Result<Option<&Proposal>, Error> {
        if identity.project_id != self.native.project() {
            return Err(Error::MissingArtifactAuthority);
        }
        let Some(proposal) = self.model.proposals.get(&identity.proposal_id) else {
            return Ok(None);
        };
        if proposal.project_id != identity.project_id
            || proposal.proposal_id != identity.proposal_id
            || proposal.schema_version != PROPOSAL_SCHEMA_VERSION
            || proposal.batch.batch_id != identity.batch_id
            || proposal.prepared_against != identity.prepared_against
            || proposal.batch.expected_model_revision.as_ref() != Some(&identity.prepared_against)
        {
            return Err(Error::MissingArtifactAuthority);
        }
        if matches!(
            proposal.status,
            ProposalStatus::Applied | ProposalStatus::Rejected
        ) {
            return Ok(None);
        }
        let mut summaries = self
            .workspace
            .production
            .proposals
            .iter()
            .filter(|p| p.proposal_id == identity.proposal_id.to_string());
        let summary = summaries.next().ok_or(Error::MissingArtifactAuthority)?;
        if summaries.next().is_some() || summary.action_identity.as_ref() != Some(identity) {
            return Err(Error::MissingArtifactAuthority);
        }
        Ok(Some(proposal))
    }
}
