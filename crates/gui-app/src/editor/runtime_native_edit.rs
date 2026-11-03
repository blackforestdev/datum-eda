//! First native footprint edit. Input prepares intent; one background worker
//! performs typed canonical mutations and prepares an atomic refreshed scene.
use super::*;
use datum_gui_protocol::{
    EditorCommandStatus,
    footprint_move_preview::FootprintMovePreview,
    footprint_move_preview::native::NativeMovePreview,
    native_edit_client::{NativeEditAction, NativeEditIntent, NativeProjectClient},
};
use std::sync::{Arc, Mutex, mpsc};
#[path = "runtime_native_edit/actions.rs"]
mod actions;
#[path = "runtime_native_edit/catalog.rs"]
mod catalog;
use catalog::native_write_verb_ids;
#[path = "runtime_native_edit/move_acquisition.rs"]
mod move_acquisition;
#[path = "runtime_native_edit/worker.rs"]
mod worker;

struct Answer {
    proposal:
        Option<eda_engine::api::native_write::schematic_delete_proposal::NativeProposalResponse>,
    intent: NativeEditIntent,
    result: Result<String, String>,
    next: Option<datum_gui_protocol::ReviewWorkspaceState>,
    selection: Option<native_edit_selection::Prepared>,
}

fn finish_owned_edit<R>(
    client: R,
    sender: mpsc::SyncSender<Answer>,
    answer: Answer,
    wake: impl FnOnce(),
) {
    // Completion lets the UI finish closing. Release the worker's client first
    // so its thread cannot retain the final daemon owner past Runtime teardown.
    drop(client);
    let _ = sender.send(answer);
    wake();
}

pub(super) struct EditWork {
    pub preview: Option<NativeMovePreview>,
    preview_tip: Option<String>,
    pending_position: Option<PointNm>,
    preview_awaiting_presentation: bool,
    pending_move_base: Option<(bool, datum_gui_protocol::BoardReviewSceneV1)>,
    pending: Option<mpsc::Receiver<Answer>>,
    /// Startup verb catalog from the warmup describe, fed back through the wake
    /// so native-edit menus reflect engine capability from cold open rather than
    /// only after the first edit populates it.
    catalog_pending: Option<mpsc::Receiver<catalog::CatalogReply>>,
    client: Arc<Mutex<Option<NativeProjectClient>>>,
    wake: winit::event_loop::EventLoopProxy<()>,
    pub blocked: bool,
}
impl EditWork {
    pub fn new(wake: winit::event_loop::EventLoopProxy<()>) -> Self {
        Self {
            preview: None,
            preview_tip: None,
            pending_position: None,
            preview_awaiting_presentation: false,
            pending_move_base: None,
            pending: None,
            catalog_pending: None,
            client: Arc::new(Mutex::new(None)),
            wake,
            blocked: false,
        }
    }
    pub(super) fn reset_project(&mut self) {
        debug_assert!(!self.pending());
        self.preview = None;
        self.preview_tip = None;
        self.pending_position = None;
        self.preview_awaiting_presentation = false;
        self.pending_move_base = None;
        self.blocked = false;
        self.catalog_pending = None;
    }
    pub fn pending(&self) -> bool {
        self.pending.is_some()
    }
    fn reconcile_again(
        &mut self,
        mut answer: Answer,
        inputs: native_edit_selection::Inputs,
    ) -> Result<()> {
        let (sender, receiver) = mpsc::sync_channel(1);
        let client = self.client.clone();
        let wake = self.wake.clone();
        std::thread::Builder::new()
            .name("datum-edit-selection".into())
            .spawn(move || {
                let result = (|| -> Result<()> {
                    let next = answer.next.as_mut().context("Edit scene unavailable")?;
                    // Read-only superseding projection. The committed mutation is
                    // never resent when input selection changed during preparation.
                    let prepared = answer
                        .selection
                        .take()
                        .context("Qualified selection basis unavailable")?;
                    let model = prepared.session.model.clone();
                    answer.selection = Some(native_edit_selection::prepare_from_model(
                        next,
                        inputs,
                        model,
                        prepared.cache,
                    )?);
                    let owner = client
                        .lock()
                        .map_err(|_| anyhow::anyhow!("Native writer owner unavailable"))?;
                    let current = owner
                        .as_ref()
                        .context("Native writer unavailable")?
                        .describe(&answer.intent.project_root)?;
                    anyhow::ensure!(
                        current
                            .get("model_revision")
                            .and_then(serde_json::Value::as_str)
                            == Some(next.scene.source_revision.as_str())
                            && current
                                .get("transaction_tip")
                                .and_then(serde_json::Value::as_str)
                                == next.supervision.journal.accepted_transaction_tip.as_deref(),
                        "Project changed during selection preparation; reopen before editing"
                    );
                    Ok(())
                })();
                if let Err(error) = result {
                    answer.next = None;
                    answer.selection = None;
                    answer.result = Err(format!("{error:#}"));
                }
                finish_owned_edit(client, sender, answer, || {
                    let _ = wake.send_event(());
                });
            })?;
        self.pending = Some(receiver);
        Ok(())
    }
}

impl Runtime {
    pub(super) fn native_edit_available(&self) -> bool {
        datum_gui_protocol::gui_menu_model::native_project_edit_available(self.workspace())
    }
    pub(super) fn cancel_native_move(&mut self) -> bool {
        self.native_edit.pending_position = None;
        self.native_edit.preview_awaiting_presentation = false;
        if let Some(preview) = self.native_edit.preview.take() {
            self.native_edit.preview_tip = None;
            self.set_move_scene(preview.is_schematic(), preview.base().clone());
            self.session.workspace_mut().cancel_authoring_gesture();
            self.invalidate_scene();
            return true;
        }
        false
    }
    fn restore_pending_move_base(&mut self) {
        if let Some((schematic, base)) = self.native_edit.pending_move_base.take()
            && base.project_uuid == self.workspace().scene.project_uuid
        {
            self.set_move_scene(schematic, base);
            self.invalidate_scene();
        }
    }
    fn set_move_scene(&mut self, schematic: bool, scene: datum_gui_protocol::BoardReviewSceneV1) {
        if schematic {
            self.session.workspace_mut().schematic_scene = Some(scene);
        } else {
            self.session.workspace_mut().scene = scene;
        }
    }
    pub(super) fn native_move_pointer(&mut self, world: PointNm) -> bool {
        if self.native_edit.pending() {
            return true;
        }
        if self.native_edit.preview.is_some() {
            let pane = self
                .last_cursor_pos
                .and_then(|(x, y)| self.pane_at_screen(x, y));
            if !self.guard_native_authoring_view(pane) {
                return true;
            }
        }
        let Some(preview) = &self.native_edit.preview else {
            return false;
        };
        match preview.destination(world, self.workspace().authoring.snap) {
            Ok(position) => self.native_edit.pending_position = Some(position),
            Err(error) => self.log_console_refusal(ConsoleFeedbackSource::Tool, error.to_string()),
        }
        // Input captures the latest complete intent. The native frame materializes
        // it once, so accumulated motion does not rebuild copper on every event.
        true
    }
    /// Keep the complete source stable across upload-only native turns. New
    /// pointer intent supersedes only the queued destination, not the source
    /// whose upload/presentation is already in progress.
    pub(super) fn prepare_native_move_frame(&mut self) {
        if self.native_edit.preview.is_some()
            && !self.native_edit.pending()
            && !self.guard_native_authoring_view(Some(self.workspace().ui.layout.focused))
        {
            return;
        }
        if self.native_edit.preview_awaiting_presentation {
            return;
        }
        let Some(position) = self.native_edit.pending_position.take() else {
            return;
        };
        let Some(preview) = &self.native_edit.preview else {
            return;
        };
        match preview.scene_at(position) {
            Ok(scene) => {
                let schematic = preview.is_schematic();
                if matches!(preview, NativeMovePreview::Symbols(_)) {
                    self.session.workspace_mut().authoring.gesture.preview = Some(position);
                }
                self.set_move_scene(schematic, scene);
                self.native_edit.preview_awaiting_presentation = true;
                self.invalidate_scene();
            }
            Err(error) => self.log_console_refusal(ConsoleFeedbackSource::Tool, error.to_string()),
        }
    }
    pub(super) fn native_move_frame_presented(&mut self) {
        self.native_edit.preview_awaiting_presentation = false;
    }
    pub(super) fn native_move_frame_needed(&self) -> bool {
        self.native_edit.preview.is_some()
            && self.native_edit.pending_position.is_some()
            && !self.native_edit.preview_awaiting_presentation
    }
    pub(super) fn queue_native_edit(&mut self, action: NativeEditAction) -> bool {
        if !matches!(action, NativeEditAction::Proposal)
            && self.workspace().schematic_authoring.delete_review.is_some()
        {
            self.log_console_refusal(
                ConsoleFeedbackSource::Tool,
                "Accept or dismiss the Proposal first",
            );
            return true;
        }
        let creating = matches!(action, NativeEditAction::Schematic)
            && matches!(
                self.workspace().schematic_authoring.pending_request,
                Some(
                    datum_gui_protocol::schematic_authoring::SchematicEditRequest::CreateSheet { .. }
                )
            );
        if !creating
            && !self.native_edit.pending()
            && matches!(
                action,
                NativeEditAction::Move { .. }
                    | NativeEditAction::MoveSymbol { .. }
                    | NativeEditAction::MoveSymbols { .. }
                    | NativeEditAction::MoveText { .. }
                    | NativeEditAction::MoveLabel { .. }
                    | NativeEditAction::Schematic
            )
            && !self.guard_native_authoring_view(Some(self.workspace().ui.layout.focused))
        {
            return true;
        }
        if !self.native_edit_available()
            || self.native_edit.pending()
            || self.check_work.pending()
            || self.native_edit.blocked
            || self.application_shutdown_started.is_some()
        {
            self.log_console_refusal(
                ConsoleFeedbackSource::Tool,
                "Editing requires an open persistent Datum Project with no pending edit",
            );
            return true;
        }
        if !matches!(
            action,
            NativeEditAction::Move { .. }
                | NativeEditAction::MoveSymbol { .. }
                | NativeEditAction::MoveSymbols { .. }
                | NativeEditAction::MoveText { .. }
                | NativeEditAction::MoveLabel { .. }
        ) && self.native_edit.preview.is_some()
        {
            self.log_console_refusal(
                ConsoleFeedbackSource::Tool,
                "Finish or cancel the Move preview first",
            );
            return true;
        }
        if !matches!(action, NativeEditAction::Schematic)
            && (self.workspace().schematic_authoring.active()
                || self.workspace().schematic_authoring.picker
                || self.workspace().schematic_authoring.creation.is_some())
        {
            self.log_console_refusal(
                ConsoleFeedbackSource::Tool,
                "Finish or cancel the schematic preview first",
            );
            return true;
        }
        if !matches!(action, NativeEditAction::Library) && self.workspace().library.draft.is_some()
        {
            self.log_console_refusal(
                ConsoleFeedbackSource::Tool,
                "Finish or cancel the library preview first",
            );
            return true;
        }
        let Some(backing) = self.workspace().backing.clone() else {
            return false;
        };
        let project_id = match self.workspace().scene.project_uuid.parse() {
            Ok(id) => id,
            Err(_) => return false,
        };
        let expected_revision = self
            .native_edit
            .preview
            .as_ref()
            .map(|p| p.base().source_revision.clone())
            .unwrap_or_else(|| self.workspace().scene.source_revision.clone());
        let expected_tip = if self.native_edit.preview.is_some() {
            self.native_edit.preview_tip.clone()
        } else {
            self.workspace()
                .supervision
                .journal
                .accepted_transaction_tip
                .clone()
        }
        .and_then(|tip| tip.parse().ok());
        let mut intent = NativeEditIntent::new(
            backing.request.project_root.clone(),
            project_id,
            expected_revision,
            expected_tip,
            action,
        );
        if !matches!(action, NativeEditAction::LibraryAcquire)
            && self.workspace().library.acquisition.is_some()
        {
            self.log_console_refusal(
                ConsoleFeedbackSource::Tool,
                "Import or cancel the library acquisition first",
            );
            return true;
        }
        if matches!(action, NativeEditAction::Proposal) {
            let Some(review) = &self.workspace().schematic_authoring.delete_review else {
                return false;
            };
            intent.proposal = review.pending.clone();
            intent.expected_revision = review.revision.clone();
            intent.expected_tip = review.tip;
        }
        if matches!(action, NativeEditAction::LibraryAcquire) {
            let Some(acquisition) = &self.workspace().library.acquisition else {
                return false;
            };
            match acquisition.request() {
                Ok(request) => intent.library_acquisition = Some(request),
                Err(error) => {
                    self.log_console_refusal(ConsoleFeedbackSource::Tool, error);
                    return true;
                }
            }
            intent.expected_revision = acquisition.revision.clone();
            intent.expected_tip = acquisition.tip;
        }
        if matches!(action, NativeEditAction::Library) {
            let Some(draft) = &self.workspace().library.draft else {
                return false;
            };
            match draft.request() {
                Ok(request) => intent.library = Some((draft.create, request)),
                Err(e) => {
                    self.log_console_refusal(ConsoleFeedbackSource::Tool, e);
                    return true;
                }
            }
            intent.expected_revision = draft.revision.clone();
            intent.expected_tip = draft.tip;
        }
        if let Some(preview) = &self.native_edit.preview {
            intent.instance_path = preview.instance_path();
            if let NativeEditAction::MoveSymbols { x_nm, y_nm, .. } = action {
                intent.symbol_group = preview.group_request(PointNm { x: x_nm, y: y_nm });
            }
        }
        if creating && let Some(draft) = &self.workspace().schematic_authoring.creation {
            intent.expected_revision = draft.revision.clone();
            intent.expected_tip = draft.tip;
        }
        if matches!(action, NativeEditAction::Schematic)
            && let Some((basis, tip)) = self.selected_schematic_edit_basis()
        {
            intent.expected_revision = basis.revision.clone();
            intent.expected_tip = tip;
        }
        if matches!(action, NativeEditAction::Schematic) {
            intent.schematic = self
                .session
                .workspace_mut()
                .schematic_authoring
                .pending_request
                .take();
        }
        // Build the final click-derived preview before dispatching the writer.
        // Ending a gesture is not cancellation: keep this complete pending pose
        // visible until the qualified canonical response replaces it.
        let pending_move = match (action, self.native_edit.preview.as_ref()) {
            (
                NativeEditAction::Move { x_nm, y_nm, .. }
                | NativeEditAction::MoveSymbol { x_nm, y_nm, .. }
                | NativeEditAction::MoveSymbols { x_nm, y_nm, .. }
                | NativeEditAction::MoveText { x_nm, y_nm, .. }
                | NativeEditAction::MoveLabel { x_nm, y_nm, .. },
                Some(preview),
            ) => match preview.scene_at(PointNm { x: x_nm, y: y_nm }) {
                Ok(scene) => Some((preview.is_schematic(), preview.base().clone(), scene)),
                Err(error) => {
                    self.log_console_refusal(ConsoleFeedbackSource::Tool, error.to_string());
                    return true;
                }
            },
            _ => None,
        };
        let inputs = native_edit_selection::Inputs::capture(self);
        let previous = (
            self.workspace().library.basis.clone(),
            self.workspace().checks.clone(),
        );
        match self
            .native_edit
            .start(intent, backing.request, inputs, previous)
        {
            Ok(()) => {
                if let Some((schematic, base, scene)) = pending_move {
                    self.native_edit.pending_move_base = Some((schematic, base));
                    self.native_edit.preview = None;
                    self.native_edit.preview_tip = None;
                    self.native_edit.pending_position = None;
                    self.native_edit.preview_awaiting_presentation = false;
                    self.set_move_scene(schematic, scene);
                    self.session.workspace_mut().cancel_authoring_gesture();
                    self.invalidate_scene();
                } else {
                    self.cancel_native_move();
                }
                self.session.workspace_mut().last_command_status = Some(EditorCommandStatus {
                    action: "native_project_edit".into(),
                    detail: "Applying Project action".into(),
                });
                self.log_console_echo(ConsoleFeedbackSource::Tool, "Applying Project action");
            }
            Err(error) => {
                if let Some(review) = &mut self
                    .session
                    .workspace_mut()
                    .schematic_authoring
                    .delete_review
                {
                    review.pending = None;
                    review.refusal = Some(error.to_string());
                }
                self.log_console_refusal(ConsoleFeedbackSource::Tool, error.to_string());
            }
        }
        self.invalidate_frame();
        true
    }
    pub(super) fn poll_native_edit(&mut self) -> bool {
        let Some(receiver) = &self.native_edit.pending else {
            return false;
        };
        let answer = match receiver.try_recv() {
            Ok(answer) => answer,
            Err(mpsc::TryRecvError::Empty) => return false,
            Err(_) => {
                self.native_edit.pending = None;
                self.native_edit.blocked = true;
                self.restore_pending_move_base();
                if let Some(a) = &mut self.session.workspace_mut().library.acquisition {
                    a.refusal =
                        Some("Edit worker stopped; reopen the Project before editing".into());
                }
                self.log_console_refusal(
                    ConsoleFeedbackSource::Tool,
                    "Edit worker stopped; reopen the Project before editing",
                );
                return true;
            }
        };
        self.native_edit.pending = None;
        if self.workspace().scene.project_uuid != answer.intent.project_id.to_string() {
            self.native_edit.pending_move_base = None;
            return false;
        }
        if answer
            .selection
            .as_ref()
            .is_some_and(|prepared| !prepared.inputs.matches(self))
        {
            let inputs = native_edit_selection::Inputs::capture(self);
            if let Err(error) = self.native_edit.reconcile_again(answer, inputs) {
                self.native_edit.blocked = true;
                self.restore_pending_move_base();
                self.log_console_refusal(ConsoleFeedbackSource::Tool, error.to_string());
            }
            return true;
        }
        let created = match (&answer.result, &answer.intent.schematic) {
            (
                Ok(_),
                Some(datum_gui_protocol::schematic_authoring::SchematicEditRequest::CreateSheet {
                    sheet,
                    ..
                }),
            ) if answer.next.is_some() && answer.selection.is_some() => Some(*sheet),
            _ => None,
        };
        let review = self.workspace().schematic_authoring.delete_review.clone();
        let draft = self.workspace().schematic_authoring.creation.clone();
        let editing = self.selected_schematic_edit_snapshot();
        let placement = matches!(answer.intent.action, NativeEditAction::LibraryAcquire)
            .then(|| self.acquisition_placement_origin())
            .flatten();
        let placement_symbol = self
            .workspace()
            .library
            .acquisition
            .as_ref()
            .and_then(|a| a.preview.as_ref())
            .and_then(|p| p.symbol.get("uuid"))
            .and_then(serde_json::Value::as_str)
            .and_then(|s| s.parse().ok());
        let acquired = answer.result.is_ok() && answer.next.is_some() && answer.selection.is_some();
        // A qualified refresh can follow a refused write. It must not consume
        // the acquisition or select its preview as if Import had succeeded.
        let refused_acquisition = if answer.result.is_err()
            && matches!(answer.intent.action, NativeEditAction::LibraryAcquire)
        {
            self.session.workspace_mut().library.acquisition.take()
        } else {
            None
        };
        if let (Some(next), Some(prepared)) = (answer.next, answer.selection) {
            self.native_edit.pending_move_base = None;
            self.publish_prepared_workspace(next, prepared);
        } else {
            self.native_edit.blocked = true;
            self.restore_pending_move_base();
        }
        self.restore_delete_review(review, answer.proposal, &answer.result);
        if let (Err(reason), Some(mut acquisition)) = (&answer.result, refused_acquisition) {
            acquisition.refusal = Some(reason.clone());
            self.session.workspace_mut().library.acquisition = Some(acquisition);
        }
        if let Some(sheet) = created {
            self.open_created_schematic(sheet);
        } else if let (Err(reason), Some(mut draft)) = (&answer.result, draft) {
            draft.refusal = Some(reason.clone());
            self.session.workspace_mut().schematic_authoring.creation = Some(draft);
        }
        if let (Err(reason), Some(editing)) = (&answer.result, editing) {
            self.restore_selected_schematic_edit(editing, reason);
        }
        if acquired && let Some(origin) = placement {
            self.resume_library_placement(origin, placement_symbol);
        }
        match answer.result {
            Ok(message) => {
                self.session.workspace_mut().last_command_status = Some(EditorCommandStatus {
                    action: "native_project_edit".into(),
                    detail: message.clone(),
                });
                self.log_console_echo(ConsoleFeedbackSource::Tool, message);
            }
            Err(message) => {
                self.session.workspace_mut().last_command_status = Some(EditorCommandStatus {
                    action: "native_project_edit".into(),
                    detail: message.clone(),
                });
                self.log_console_refusal(ConsoleFeedbackSource::Tool, message);
            }
        }
        self.invalidate_frame();
        true
    }
}

#[cfg(test)]
#[path = "native_first_edit_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "runtime_native_edit/project_open_busy_tests.rs"]
mod project_open_busy_tests;

#[cfg(test)]
mod completion_ownership_tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::time::Duration;

    struct ClientDropProbe(Arc<AtomicBool>);

    impl Drop for ClientDropProbe {
        fn drop(&mut self) {
            self.0.store(true, Ordering::SeqCst);
        }
    }

    fn answer() -> Answer {
        Answer {
            proposal: None,
            intent: NativeEditIntent::new(
                PathBuf::from("unopened-test-project"),
                uuid::Uuid::new_v4(),
                "unchanged".into(),
                None,
                NativeEditAction::Save,
            ),
            result: Ok("completion".into()),
            next: None,
            selection: None,
        }
    }

    #[test]
    fn completion_releases_client_before_the_worker_wakes_and_returns() {
        let dropped = Arc::new(AtomicBool::new(false));
        let ui_client = Arc::new(ClientDropProbe(dropped.clone()));
        let worker_client = ui_client.clone();
        let (sender, receiver) = mpsc::sync_channel(1);
        let (awake, awakened) = mpsc::channel();
        let (release, released) = mpsc::channel();
        let timeout = Duration::from_secs(5);
        let worker = std::thread::spawn(move || {
            finish_owned_edit(worker_client, sender, answer(), || {
                awake.send(()).unwrap();
                released.recv_timeout(timeout).unwrap();
            });
        });
        let answer = receiver.recv_timeout(timeout).unwrap();
        awakened.recv_timeout(timeout).unwrap();
        // The worker is parked inside wake: it has neither returned nor dropped
        // its stack. The UI must already be the sole client owner at this point.
        let owners_at_completion = Arc::strong_count(&ui_client);
        drop(ui_client);
        let destroyed_before_worker_return = dropped.load(Ordering::SeqCst);
        release.send(()).unwrap();
        worker.join().unwrap();
        assert_eq!(answer.result.unwrap(), "completion");
        assert_eq!(owners_at_completion, 1);
        assert!(destroyed_before_worker_return);
    }

    #[test]
    fn completion_drops_owned_resource_before_publishing_answer() {
        struct DropNotifier(mpsc::Sender<()>);
        impl Drop for DropNotifier {
            fn drop(&mut self) {
                let _ = self.0.send(());
            }
        }
        let (destroyed, destruction) = mpsc::channel();
        // Sending cannot finish until the UI receives. If ownership release
        // moves after publication, the destruction notice cannot arrive first.
        let (sender, receiver) = mpsc::sync_channel(0);
        let worker = std::thread::spawn(move || {
            finish_owned_edit(DropNotifier(destroyed), sender, answer(), || {});
        });
        let timeout = Duration::from_secs(5);
        let destroyed_before_receiving = destruction.recv_timeout(timeout).is_ok();
        let answer = receiver.recv_timeout(timeout).unwrap();
        worker.join().unwrap();
        assert!(destroyed_before_receiving);
        assert_eq!(answer.result.unwrap(), "completion");
    }
}
