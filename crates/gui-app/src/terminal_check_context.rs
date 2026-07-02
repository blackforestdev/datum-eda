use crate::terminal_context_contract::TerminalCheckRunProfileLatest;
use datum_gui_protocol::CheckRunReviewState;

pub(super) fn profile_latest_check_runs_context(
    check_status: &CheckRunReviewState,
) -> Vec<TerminalCheckRunProfileLatest> {
    let (Some(profile_id), Some(check_run_id)) =
        (&check_status.profile_id, &check_status.check_run_id)
    else {
        return Vec::new();
    };
    vec![TerminalCheckRunProfileLatest {
        profile_id: profile_id.clone(),
        check_run_id: check_run_id.clone(),
        model_revision: check_status.model_revision.clone(),
        status: check_status.status.clone(),
        finding_count: check_status.finding_count,
    }]
}
