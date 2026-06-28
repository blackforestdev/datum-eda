use crate::terminal_session::TerminalLaunchContext;
use datum_gui_protocol::{LiveReviewRequest, refresh_accepted_transaction_tip};

pub(super) fn accepted_transaction_tip(context: &TerminalLaunchContext) -> Option<String> {
    refresh_accepted_transaction_tip(&LiveReviewRequest {
        project_root: context.project_root.clone(),
        board_file: None,
        artifact_path: None,
        net_uuid: None,
        from_anchor_pad_uuid: None,
        to_anchor_pad_uuid: None,
        profile: None,
    })
    .ok()
    .flatten()
}
