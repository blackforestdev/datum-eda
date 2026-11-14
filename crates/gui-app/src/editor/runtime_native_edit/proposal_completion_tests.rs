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
            board_connectivity_revision: None,
            board_field_revision: None,
            project_annotation: None,
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
            fields: None,
            disposition: None,
            gate_placement: None,
            child_sheet: None,
            interface_binding: None,
        };
        let review = DeleteReview {
            board_field_revision: None,
            board_connectivity_revision: None,
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
            fields: None,
            disposition: None,
            gate_placement: None,
            child_sheet: None,
            interface_binding: None,
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

/// Observe the real successful Runtime publication; no synthetic write/receipt.
pub(crate) fn completed_echo_precedes_workspace_warning(runtime: &Runtime, prefix: &str) {
    use datum_gui_protocol::{
        CONSOLE_FEEDBACK_CAPACITY, ConsoleFeedbackCategory as Category, ConsoleFeedbackDuration,
        ConsoleFeedbackLifetime as Lifetime, ConsoleFeedbackSeverity as Severity,
        ConsoleFeedbackSource as Source,
    };
    let console = &runtime.workspace().ui.console;
    let echo = console
        .records()
        .rev()
        .find(|record| {
            record.source == Source::Tool
                && record.category == Category::ActionEcho
                && record.message.starts_with(prefix)
        })
        .expect("Actual completed action must reach Console history");
    let warning = console
        .visible_latest()
        .expect("Fresh workspace warning must remain visible");
    assert_eq!(warning.source, Source::Workspace);
    assert_eq!(warning.category, Category::ActionRefusal);
    assert_eq!(warning.severity, Severity::Error);
    assert_eq!(warning.lifetime, Lifetime::PersistentUntilNextAction);
    assert!(warning.message.contains("Unrouted display unavailable"));
    let model = &runtime
        .native_selection
        .as_ref()
        .expect("Qualified refreshed model")
        .model;
    let error = eda_engine::substrate::ElectricalSelectionSnapshot::capture(model)
        .and_then(|snapshot| snapshot.unrouted_display(&model.model_revision))
        .expect_err("This actual source still has unavailable unrouted display");
    assert_eq!(
        warning.message,
        format!("Unrouted display unavailable; no connected-state conclusion: {error:?}")
    );
    eprintln!(
        "completed action echo: {} · preserved workspace warning: {}",
        echo.message, warning.message
    );
    assert!(
        !warning.message.starts_with(prefix),
        "Successful action must not be relabelled Refused"
    );
    assert!(echo.sequence < warning.sequence);
    assert!(console.len() <= CONSOLE_FEEDBACK_CAPACITY);
    let mut elapsed = console.clone();
    elapsed.set_duration_preference(ConsoleFeedbackDuration::FourSeconds);
    elapsed.advance_visibility(warning.occurred_unix_ms.saturating_add(100_000), false);
    assert_eq!(elapsed.visible_latest(), Some(warning));
}
