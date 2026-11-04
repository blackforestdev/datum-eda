//! Deterministic completion failures at the actual Runtime publication seam.
use super::*;
use datum_gui_protocol::schematic_authoring::delete_review::DeleteReview;
use datum_gui_protocol::{AuthoringViewBasis, PaneContent, PaneId};
use eda_engine::api::native_write::schematic_delete_proposal::{
    NativeProposalAction, NativeProposalResponse,
};
use eda_engine::substrate::ProposalStatus;
use uuid::Uuid;

pub(super) fn failed_refresh_consumes_only_terminal_reviews(runtime: &mut Runtime) {
    let root = runtime
        .workspace()
        .backing
        .as_ref()
        .unwrap()
        .request
        .project_root
        .clone();
    let project: Uuid = runtime.workspace().scene.project_uuid.parse().unwrap();
    let revision = runtime.workspace().scene.source_revision.clone();
    let scene = runtime.workspace().scene.clone();
    for status in [ProposalStatus::Rejected, ProposalStatus::Draft] {
        let id = Uuid::new_v4();
        let sheet = Uuid::new_v4();
        let response = NativeProposalResponse {
            proposal_id: id,
            status,
            prepared_against: revision.clone(),
            current_model_revision: revision.clone(),
            transaction_tip: None,
            transaction_id: None,
            selected_count: Some(0),
            owned_count: Some(0),
            deleted: vec![],
            assignment: None,
            annotation: None,
            disposition: None,
            gate_placement: None,
        };
        let review = DeleteReview {
            basis: AuthoringViewBasis {
                project,
                revision: revision.clone(),
                pane: PaneId(0),
                content: PaneContent::Schematic,
                source: sheet,
                document: None,
            },
            source: serde_json::from_value(
                serde_json::json!({"uuid":sheet,"name":"completion state",
                "frame":null,"symbols":{},"wires":{},"junctions":{},"labels":{},"buses":{},
                "bus_entries":{},"ports":{},"noconnects":{},"texts":{},"drawings":{}}),
            )
            .unwrap(),
            scene: scene.clone(),
            targets: vec![],
            proposal_id: id,
            revision: revision.clone(),
            tip: None,
            response: None,
            pending: Some(NativeProposalAction::Reject { proposal_id: id }),
            refusal: None,
            assignment: None,
            annotation: None,
            disposition: None,
            gate_placement: None,
        };
        runtime
            .session
            .workspace_mut()
            .schematic_authoring
            .delete_review = Some(review);
        runtime.native_edit.blocked = false;
        let (send, receive) = mpsc::sync_channel(1);
        runtime.native_edit.pending = Some(receive);
        send.send(Answer {
            proposal: Some(response),
            intent: NativeEditIntent::new(
                root.clone(),
                project,
                revision.clone(),
                None,
                NativeEditAction::Proposal,
            ),
            result: Err("forced refresh failure; reopen the Project before editing".into()),
            next: None,
            selection: None,
        })
        .unwrap();
        assert!(runtime.poll_native_edit());
        assert!(!runtime.native_edit.pending());
        assert!(
            runtime.native_edit.blocked,
            "failed refresh must not enable stale writes"
        );
        let current = &runtime.workspace().schematic_authoring.delete_review;
        if status == ProposalStatus::Rejected {
            assert!(
                current.is_none(),
                "terminal receipt must consume the old pending overlay"
            );
        } else {
            let current = current
                .as_ref()
                .expect("nonterminal review must remain dismissible");
            assert!(current.pending.is_none());
            assert!(
                current
                    .refusal
                    .as_deref()
                    .unwrap()
                    .contains("forced refresh failure")
            );
        }
        assert!(
            runtime
                .workspace()
                .ui
                .console
                .records()
                .next_back()
                .unwrap()
                .message
                .contains("forced refresh failure")
        );
    }
    runtime
        .session
        .workspace_mut()
        .schematic_authoring
        .delete_review = None;
}
