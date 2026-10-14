use datum_gui_protocol::{DatumSelectionContext, ProductionStatus};

pub(super) fn visible_proposal_ids(status: &ProductionStatus) -> Vec<String> {
    status
        .proposals
        .iter()
        .map(|proposal| proposal.proposal_id.clone())
        .collect()
}

pub(super) fn latest_proposal_id(
    status: &ProductionStatus,
    selection: &DatumSelectionContext,
) -> Option<String> {
    (selection.kind == "proposal")
        .then(|| selection.id.clone())
        .flatten()
        .or_else(|| latest_status_proposal_id(status))
}

fn latest_status_proposal_id(status: &ProductionStatus) -> Option<String> {
    status
        .proposals
        .iter()
        .find(|proposal| matches!(proposal.status.as_str(), "accepted" | "draft" | "deferred"))
        .or_else(|| status.proposals.first())
        .map(|proposal| proposal.proposal_id.clone())
}
