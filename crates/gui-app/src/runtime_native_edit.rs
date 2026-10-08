//! First native footprint edit. Input prepares intent; one background worker
//! performs typed canonical mutations and prepares an atomic refreshed scene.
use super::*;
use datum_gui_protocol::{
    EditorCommandStatus,
    footprint_move_preview::FootprintMovePreview,
    native_edit_client::{NativeEditAction, NativeEditIntent, NativeProjectClient},
};
use std::sync::{Arc, Mutex, mpsc};

struct Answer {
    intent: NativeEditIntent,
    result: Result<String, String>,
    next: Option<datum_gui_protocol::ReviewWorkspaceState>,
    selection: Option<native_edit_selection::Prepared>,
}
pub(super) struct EditWork {
    pub preview: Option<FootprintMovePreview>,
    preview_tip: Option<String>,
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
            pending: None,
            client: Arc::new(Mutex::new(None)),
            wake,
            blocked: false,
        }
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
                    let operation=client.execute(&intent);
                    // Qualify the entire refresh against both revision and tip,
                    // including an undo/redo cycle that restores the same bytes.
                    let before=client.describe(&intent.project_root)?;
                    let mut next=load_board_editor_workspace_state(&request)?;
                    let selection=native_edit_selection::prepare(&intent.project_root,&mut next,inputs)?;
                    let after=client.describe(&intent.project_root)?;
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
                    answer.selection = Some(native_edit_selection::prepare(
                        &answer.intent.project_root,
                        next,
                        inputs,
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
        if let Some(preview) = self.native_edit.preview.take() {
            self.native_edit.preview_tip = None;
            self.session.workspace_mut().scene = preview.base;
            self.session.workspace_mut().cancel_authoring_gesture();
            self.invalidate_scene();
            return true;
        }
        false
    }
    // The bounded first-edit capability moves one complete Footprint. Never
    // silently reduce a Run, compound selection or another object to a subset.
    pub(super) fn begin_selected_native_move(&mut self) {
        use datum_gui_protocol::{AuthoredSelectionClass, SelectionSubject, SelectionTarget};
        let SelectionTarget::Native(selected) = &self.workspace().selection else {
            return;
        };
        if selected.stale {
            self.log_console_refusal(
                ConsoleFeedbackSource::Tool,
                "Select a current Footprint before moving",
            );
            return;
        }
        let SelectionSubject::Object(id) = &selected.subject else {
            return;
        };
        if id.class != AuthoredSelectionClass::Footprint {
            return;
        }
        let Some(target) = self
            .workspace()
            .scene
            .components
            .iter()
            .find(|c| c.component_uuid == id.id.to_string())
            .map(|c| c.object_id.clone())
        else {
            return;
        };
        let Some((x, y)) = self.last_cursor_pos else {
            return;
        };
        let Some((world, SceneSurface::Board)) = self
            .prepared_scene()
            .and_then(|s| s.world_point_at_screen(x, y))
        else {
            return;
        };
        self.native_move_click(world, Some(target));
    }
    pub(super) fn native_move_click(&mut self, world: PointNm, target: Option<String>) -> bool {
        if self.native_edit.pending() || self.native_edit.blocked {
            self.log_console_refusal(
                ConsoleFeedbackSource::Tool,
                "Wait for the current edit or reopen the Project after an unconfirmed outcome",
            );
            return true;
        }
        if let Some(preview) = &self.native_edit.preview {
            let destination = match preview.destination(world, self.workspace().authoring.snap) {
                Ok(p) => p,
                Err(error) => {
                    self.log_console_refusal(ConsoleFeedbackSource::Tool, error.to_string());
                    return true;
                }
            };
            let component = preview.component.component_uuid.parse();
            let action = match component {
                Ok(component) => NativeEditAction::Move {
                    component,
                    x_nm: destination.x,
                    y_nm: destination.y,
                },
                Err(_) => return true,
            };
            return self.queue_native_edit(action);
        }
        let Some(target) = target else {
            self.log_console_refusal(
                ConsoleFeedbackSource::Tool,
                "Move requires clicking an existing Footprint",
            );
            return true;
        };
        let preview = match FootprintMovePreview::begin(
            &self.workspace().scene,
            &target,
            world,
            self.workspace().authoring.snap,
        ) {
            Ok(preview) => preview,
            Err(error) => {
                self.log_console_refusal(ConsoleFeedbackSource::Tool, error.to_string());
                return true;
            }
        };
        let target = preview.component.object_id.clone();
        self.dispatch_session_command(SessionCommand::BeginAuthoringGesture {
            world,
            target_object_id: Some(target),
        });
        self.native_edit.preview_tip = self
            .workspace()
            .supervision
            .journal
            .accepted_transaction_tip
            .clone();
        self.native_edit.preview = Some(preview);
        self.invalidate_frame();
        true
    }
    pub(super) fn native_move_pointer(&mut self, world: PointNm) -> bool {
        let Some(preview) = &self.native_edit.preview else {
            return false;
        };
        let next = preview
            .destination(world, self.workspace().authoring.snap)
            .and_then(|p| preview.scene_at(p));
        match next {
            Ok(scene) => {
                self.session.workspace_mut().scene = scene;
                self.invalidate_scene();
            }
            Err(error) => self.log_console_refusal(ConsoleFeedbackSource::Tool, error.to_string()),
        }
        true
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
        if !self.native_edit_available()
            || self.native_edit.pending()
            || self.native_edit.blocked
            || self.application_shutdown_started.is_some()
        {
            self.log_console_refusal(
                ConsoleFeedbackSource::Tool,
                "Editing requires an open persistent Datum Project with no pending edit",
            );
            return true;
        }
        if !matches!(action, NativeEditAction::Move { .. }) && self.native_edit.preview.is_some() {
            self.log_console_refusal(
                ConsoleFeedbackSource::Tool,
                "Finish or cancel the Move preview first",
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
            .map(|p| p.base.source_revision.clone())
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
        let intent = NativeEditIntent::new(
            backing.request.project_root.clone(),
            project_id,
            expected_revision,
            expected_tip,
            action,
        );
        let inputs = native_edit_selection::Inputs::capture(self);
        match self.native_edit.start(intent, backing.request, inputs) {
            Ok(()) => {
                self.cancel_native_move();
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
                self.log_console_refusal(
                    ConsoleFeedbackSource::Tool,
                    "Edit worker stopped; reopen the Project before editing",
                );
                return true;
            }
        };
        self.native_edit.pending = None;
        if self.workspace().scene.project_uuid != answer.intent.project_id.to_string() {
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
                self.log_console_refusal(ConsoleFeedbackSource::Tool, error.to_string());
            }
            return true;
        }
        if let (Some(next), Some(prepared)) = (answer.next, answer.selection) {
            let workspace = self.session.workspace_mut();
            workspace.scene = next.scene;
            workspace.schematic_scene = next.schematic_scene;
            workspace.review = next.review;
            workspace.production = next.production;
            workspace.source_shards = next.source_shards;
            workspace.checks = next.checks;
            workspace.ui.console_journal.reconcile(
                &next.supervision.journal.projection,
                next.supervision.journal.applied_transaction_count,
            );
            workspace.supervision = next.supervision;
            workspace.backing = next.backing;
            workspace.selection = prepared.selected;
            self.selection_run.cancel();
            self.native_selection = Some(prepared.session);
            self.selection_read_cache = prepared.cache;
            self.refresh_selection_accessibility();
            self.invalidate_scene();
        } else {
            self.native_edit.blocked = true;
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
