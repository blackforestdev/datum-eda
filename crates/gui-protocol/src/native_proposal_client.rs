//! Typed manual Proposal lifecycle on the existing owned native transport.
use super::*;
use eda_engine::api::native_write::schematic_delete_proposal::{
    NativeProposalAction as Action, NativeProposalResponse,
};
use eda_engine::substrate::ProposalStatus;

impl NativeProjectClient {
    pub(super) fn execute_proposal(&self, intent: &NativeEditIntent) -> Result<NativeEditReceipt> {
        let action = intent
            .proposal
            .as_ref()
            .context("Proposal action missing")?;
        let id = match action {
            Action::CreateDelete { proposal_id, .. }
            | Action::CreateDisposition { proposal_id, .. }
            | Action::CreateAssignment { proposal_id, .. }
            | Action::Preview { proposal_id }
            | Action::AcceptApply { proposal_id }
            | Action::Reject { proposal_id } => *proposal_id,
        };
        let call = |action: &Action, revision: &str, tip: Option<Uuid>| {
            self.call(
                "native.proposal",
                json!({"project_root":intent.project_root,"expected_model_revision":revision,
            "expected_transaction_tip":tip,"action":action}),
                Uuid::new_v4(),
            )
        };
        let result = call(action, &intent.expected_revision, intent.expected_tip);
        let reconciled = result.is_err();
        let transport_error = result.as_ref().err().map(|e| format!("{e:#}"));
        let response: NativeProposalResponse = match result {
            Ok(value) => serde_json::from_value(value)?,
            Err(error) => {
                // No mutation retry: Preview reads the exact persisted Proposal only.
                let current = self.describe(&intent.project_root)?;
                let revision = current["model_revision"]
                    .as_str()
                    .context("Project revision unavailable")?;
                let tip = serde_json::from_value(current["transaction_tip"].clone())?;
                match call(&Action::Preview { proposal_id: id }, revision, tip) {
                    Ok(value) => serde_json::from_value(value)?,
                    Err(_) => {
                        return Err(
                            error.context("Proposal outcome unconfirmed; no mutation retry sent")
                        );
                    }
                }
            }
        };
        anyhow::ensure!(
            response.proposal_id == id,
            "Proposal receipt identity mismatch"
        );
        let expected = match action {
            Action::CreateDelete { .. }
            | Action::CreateDisposition { .. }
            | Action::CreateAssignment { .. } => ProposalStatus::Draft,
            Action::AcceptApply { .. } => ProposalStatus::Applied,
            Action::Reject { .. } => ProposalStatus::Rejected,
            Action::Preview { .. } => response.status,
        };
        Ok(proposal_receipt(
            response,
            expected,
            transport_error,
            reconciled,
            self.describe_snapshot(&intent.project_root, true),
        ))
    }
}

fn proposal_receipt(
    response: NativeProposalResponse,
    expected: ProposalStatus,
    transport_error: Option<String>,
    reconciled: bool,
    description: Result<Value>,
) -> NativeEditReceipt {
    let (description, refresh_error) = match description {
        Ok(description) => (description, None),
        Err(error) => (
            Value::Null,
            Some(format!(
                "Proposal status confirmed but refresh failed: {error:#}; close this review or reopen the Project"
            )),
        ),
    };
    let description_agrees = description["model_revision"]
        == json!(response.current_model_revision)
        && description["transaction_tip"] == json!(response.transaction_tip);
    let proposal_refusal = refresh_error.or_else(|| {
        proposal_refusal(
            response.status,
            expected,
            transport_error,
            description_agrees,
        )
    });
    NativeEditReceipt {
        applied: response.status == ProposalStatus::Applied,
        reconciled,
        proposal_refusal,
        description,
        proposal: Some(response),
    }
}

fn proposal_refusal(
    observed: ProposalStatus,
    expected: ProposalStatus,
    transport_error: Option<String>,
    description_agrees: bool,
) -> Option<String> {
    if !description_agrees {
        return Some("Project changed after the Proposal operation; inspect the refreshed design or close this review".into());
    }
    if observed != expected {
        return Some(transport_error.unwrap_or_else(|| {
            "Proposal did not reach the requested state; review its confirmed status".into()
        }));
    }
    None
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn batch_delete_review_retains_accepted_and_terminal_receipts_after_partial_failure() {
        for status in [
            ProposalStatus::Accepted,
            ProposalStatus::Applied,
            ProposalStatus::Rejected,
        ] {
            let response = NativeProposalResponse {
                assignment: None,
                disposition: None,
                proposal_id: Uuid::new_v4(),
                status,
                prepared_against: "before".into(),
                current_model_revision: "after".into(),
                transaction_tip: None,
                deleted: vec![],
                selected_count: None,
                owned_count: None,
                transaction_id: None,
            };
            let receipt = proposal_receipt(
                response.clone(),
                ProposalStatus::Applied,
                None,
                false,
                Err(anyhow::anyhow!("snapshot transport failed")),
            );
            assert_eq!(
                receipt.proposal,
                Some(response),
                "confirmed lifecycle survives failed durable snapshot"
            );
            assert!(
                receipt.description.is_null(),
                "missing refresh is never authority"
            );
            assert!(
                receipt
                    .proposal_refusal
                    .unwrap()
                    .contains("snapshot transport failed")
            );
        }

        assert!(
            proposal_refusal(
                ProposalStatus::Accepted,
                ProposalStatus::Applied,
                Some("apply refused".into()),
                true
            )
            .unwrap()
            .contains("apply refused")
        );
        for observed in [ProposalStatus::Applied, ProposalStatus::Rejected] {
            assert!(proposal_refusal(observed, observed, None, false).is_some());
            assert!(
                proposal_refusal(observed, observed, Some("lost response".into()), true).is_none()
            );
        }
    }
}
