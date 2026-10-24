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
#[path = "runtime_native_edit/move_acquisition.rs"]
mod move_acquisition;

struct Answer {
    intent: NativeEditIntent,
    result: Result<String, String>,
    next: Option<datum_gui_protocol::ReviewWorkspaceState>,
    selection: Option<native_edit_selection::Prepared>,
}
pub(super) struct EditWork {
    pub preview: Option<NativeMovePreview>,
    preview_tip: Option<String>,
    pending_position: Option<PointNm>,
    preview_awaiting_presentation: bool,
    pending_move_base: Option<(bool, datum_gui_protocol::BoardReviewSceneV1)>,
    pending: Option<mpsc::Receiver<Answer>>,
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
            client: Arc::new(Mutex::new(None)),
            wake,
            blocked: false,
        }
    }
    /// Begin the existing read-only companion preparation before first edit.
    /// This creates no pending action and supplies no reusable write receipt.
    pub(super) fn prepare_read(
        &self,
        root: std::path::PathBuf,
    ) -> Option<std::thread::JoinHandle<()>> {
        let client = self.client.clone();
        let spawn = std::thread::Builder::new()
            .name("datum-native-prepare".into())
            .spawn(move || {
                let started = std::time::Instant::now();
                let prepared = (|| -> Result<()> {
                    let mut owner = client
                        .lock()
                        .map_err(|_| anyhow::anyhow!("Native editor worker unavailable"))?;
                    if owner.is_none() {
                        *owner = Some(NativeProjectClient::spawn()?);
                    }
                    owner.as_ref().unwrap().prepare_read(&root)
                })();
                trace_startup_timing(format!(
                    "native companion preparation {}ms result={prepared:?}",
                    started.elapsed().as_millis()
                ));
            });
        match spawn {
            Ok(handle) => Some(handle),
            Err(error) => {
                trace_startup_timing(format!("native companion preparation unavailable: {error}"));
                None
            }
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
    }
    pub fn pending(&self) -> bool {
        self.pending.is_some()
    }
    fn start(
        &mut self,
        intent: NativeEditIntent,
        request: LiveReviewRequest,
        inputs: native_edit_selection::Inputs,
    ) -> Result<()> {
        anyhow::ensure!(!self.pending(), "An edit is already pending");
        let (sender, receiver) = mpsc::sync_channel(1);
        let client = self.client.clone();
        let wake = self.wake.clone();
        std::thread::Builder::new()
            .name("datum-native-edit".into())
            .spawn(move || {
                let prepared = (|| -> Result<Answer> {
                    let mut owner = client.lock().map_err(|_|anyhow::anyhow!("Native editor worker unavailable"))?;
                    if owner.is_none() {*owner=Some(NativeProjectClient::spawn()?);}
                    let client=owner.as_ref().unwrap();
                    let started=std::time::Instant::now();
                    let operation=client.execute(&intent);
                    trace_startup_timing(format!("native edit canonical operation {}ms",started.elapsed().as_millis()));
                    // Qualify the entire refresh against both revision and tip,
                    // including an undo/redo cycle that restores the same bytes.
                    // A confirmed receipt already carries a freshly resolved, durable
                    // post-write description. It is the pre-refresh boundary;
                    // failures still require a fresh recovery-qualified description.
                    let recovered;
                    let before=match &operation {
                        Ok(receipt)=>&receipt.description,
                        Err(_)=>{recovered=client.describe(&intent.project_root)?; &recovered}
                    };
                    let refreshed=std::time::Instant::now();
                    // Populate existing byte-qualified read evidence on the worker;
                    // publishing an empty cache would cold-resolve on the next click.
                    let mut cache=runtime_selection::SelectionReadCache::default();
                    let model=cache.resolve_with_engine_reader(&intent.project_root,|| client.read_model(&intent.project_root,before))?;
                    let mut next=datum_gui_protocol::workspace_loading::project_board_editor_workspace(&request,model.clone())?;
                    let selection=native_edit_selection::prepare_from_model(&mut next,inputs,model,cache)?;
                    trace_startup_timing(format!("native edit complete workspace and selection {}ms",refreshed.elapsed().as_millis()));
                    let after=client.describe(&intent.project_root)?;
                    trace_startup_timing(format!("native edit qualified answer {}ms",started.elapsed().as_millis()));
                    let qualified=before.get("model_revision")==after.get("model_revision")
                        && before.get("transaction_tip")==after.get("transaction_tip")
                        && after.get("project_id")==Some(&serde_json::json!(intent.project_id))
                        && next.scene.project_uuid==intent.project_id.to_string()
                        && next.supervision.model_revision==next.scene.source_revision
                        && after.get("model_revision").and_then(serde_json::Value::as_str)==Some(next.scene.source_revision.as_str())
                        && after.get("transaction_tip").and_then(serde_json::Value::as_str)==next.supervision.journal.accepted_transaction_tip.as_deref();
                    let result=match operation {
                        Ok(receipt) if qualified && receipt.description.get("model_revision")==after.get("model_revision")
                            && receipt.description.get("transaction_tip")==after.get("transaction_tip") =>
                            Ok(if receipt.applied {if receipt.reconciled {"Edit committed; recovered receipt confirmed"} else {"Edit committed"}} else {"Project saved"}.into()),
                        Ok(_)=>Err("Edit completed but the Project changed during refresh; reopen before editing".into()),
                        Err(error)=>Err(format!("{error:#}")),
                    };
                    Ok(Answer {intent:intent.clone(),result,next:qualified.then_some(next),selection:qualified.then_some(selection)})
                })();
                let answer=prepared.unwrap_or_else(|error|Answer {intent,result:Err(format!("{error:#}; reopen the Project before editing")),next:None,selection:None});
                let _=sender.send(answer);
                let _=wake.send_event(());
            })?;
        self.pending = Some(receiver);
        Ok(())
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
                let _ = sender.send(answer);
                let _ = wake.send_event(());
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
                self.set_move_scene(preview.is_schematic(), scene);
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
    pub(super) fn queue_native_history(&mut self, redo: bool) -> bool {
        let tip = self
            .workspace()
            .supervision
            .journal
            .accepted_transaction_tip
            .clone();
        let Some(tip) = tip.and_then(|t| t.parse().ok()) else {
            self.log_console_refusal(
                ConsoleFeedbackSource::Tool,
                "No journal transaction is available",
            );
            return true;
        };
        self.queue_native_edit(if redo {
            NativeEditAction::Redo { expected_tip: tip }
        } else {
            NativeEditAction::Undo { expected_tip: tip }
        })
    }
    pub(super) fn queue_native_edit(&mut self, action: NativeEditAction) -> bool {
        if !self.native_edit.pending()
            && matches!(
                action,
                NativeEditAction::Move { .. }
                    | NativeEditAction::MoveSymbol { .. }
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
            NativeEditAction::Move { .. } | NativeEditAction::MoveSymbol { .. }
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
                || self.workspace().schematic_authoring.picker)
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
                | NativeEditAction::MoveSymbol { x_nm, y_nm, .. },
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
        match self.native_edit.start(intent, backing.request, inputs) {
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
            Err(error) => self.log_console_refusal(ConsoleFeedbackSource::Tool, error.to_string()),
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
        if let (Some(next), Some(prepared)) = (answer.next, answer.selection) {
            self.native_edit.pending_move_base = None;
            self.publish_prepared_workspace(next, prepared);
        } else {
            self.native_edit.blocked = true;
            self.restore_pending_move_base();
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
