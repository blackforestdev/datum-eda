//! A read projection of existing canonical persistence and live editor owners.
use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InspectorActivity {
    Quiet,
    Pending,
    Preview,
    Unconfirmed,
}

pub fn saved_state(state: &ReviewWorkspaceState) -> &'static str {
    match state.inspector_context.activity {
        Some(InspectorActivity::Pending) => "Unavailable — Project operation pending",
        Some(InspectorActivity::Preview) => {
            "Uncommitted editor preview; captured canonical basis is separate"
        }
        Some(InspectorActivity::Unconfirmed) => "Unavailable — last edit outcome is unconfirmed",
        None => "Unavailable — live editor activity was not captured",
        Some(InspectorActivity::Quiet) => {
            let qualified = state
                .backing
                .as_ref()
                .is_some_and(|backing| backing.request.board_file.is_none())
                && !state.project_unopened
                && state.library.basis.as_ref().is_some_and(|model| {
                    let tip = model
                        .journal_cursor
                        .applied_transaction_count
                        .checked_sub(1)
                        .and_then(|index| model.journal.get(index))
                        .map(|transaction| transaction.transaction_id.to_string());
                    model.project.project_id.to_string() == state.scene.project_uuid
                        && model.model_revision.0 == state.scene.source_revision
                        && model.journal_cursor.applied_transaction_count
                            == state.supervision.journal.applied_transaction_count
                        && tip == state.supervision.journal.accepted_transaction_tip
                });
            if qualified {
                "Saved captured canonical basis"
            } else {
                "Unavailable — canonical persistence basis is not qualified"
            }
        }
    }
}

pub fn materialization_state(state: &ReviewWorkspaceState) -> String {
    let source = &state.source_shards;
    format!(
        "{} source shards: {} matching materialization, {} differing, {} missing, {} unknown; differences do not imply unsaved edits",
        source.total, source.clean, source.dirty, source.missing, source.unknown
    )
}
