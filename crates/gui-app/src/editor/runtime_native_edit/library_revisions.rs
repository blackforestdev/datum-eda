//! Library revision previews share the native writer's client and wake route.
use super::*;
use datum_gui_protocol::library_browser::revision_workflow::{
    LibraryRevisionWorkflow, QualifiedLibraryPreview, RevisionControl as C,
};

pub(super) struct PreviewReply {
    request_id: eda_engine::substrate::ObjectId,
    result: Result<QualifiedLibraryPreview, String>,
}

impl Runtime {
    pub(super) fn reject_unqueued_library_revision(
        &mut self,
        request_id: eda_engine::substrate::ObjectId,
    ) {
        let browser = &mut self.session.workspace_mut().library;
        if browser
            .revision_workflow
            .as_mut()
            .is_some_and(|review| review.reject_unqueued_submission(request_id))
        {
            browser.refusal = Some(
                "The change was not queued. Preview again before applying; no retry was sent."
                    .into(),
            );
        }
    }

    pub(crate) fn handle_library_revision_navigation(&mut self, key: &Key) -> bool {
        let Some(review) = self.workspace().library.revision_workflow.as_ref() else {
            return false;
        };
        match key {
            Key::Named(NamedKey::Escape) => {
                let control = C::Cancel(review.session_id());
                self.activate_library_revision(control);
            }
            Key::Named(NamedKey::Tab) if !review.is_submitted() => {
                let controls = review.review_controls();
                let current = self
                    .workspace()
                    .library
                    .focus
                    .as_ref()
                    .and_then(|focus| {
                        controls.iter().position(|control| {
                            *focus
                                == datum_gui_protocol::library_browser::LibraryControl::Revision(
                                    *control,
                                )
                        })
                    })
                    .unwrap_or(0);
                let next = if self.modifiers.shift_key() {
                    (current + controls.len() - 1) % controls.len()
                } else {
                    (current + 1) % controls.len()
                };
                let browser = &mut self.session.workspace_mut().library;
                browser.focus = Some(
                    datum_gui_protocol::library_browser::LibraryControl::Revision(controls[next]),
                );
                browser.reveal_focus = true;
                self.invalidate_frame();
            }
            Key::Named(NamedKey::Enter) if !review.is_submitted() => {
                let control = match self.workspace().library.focus {
                    Some(datum_gui_protocol::library_browser::LibraryControl::Revision(
                        C::Purpose(_),
                    )) => review.preview_control(),
                    Some(datum_gui_protocol::library_browser::LibraryControl::Revision(
                        control,
                    )) => control,
                    _ => return true,
                };
                self.activate_library_revision(control);
            }
            _ => {}
        }
        true
    }

    pub(crate) fn commit_library_draft(&mut self) -> bool {
        if self
            .workspace()
            .library
            .basis
            .as_ref()
            .is_some_and(|model| model.project.schema_version == Some(2))
            && self
                .workspace()
                .library
                .draft
                .as_ref()
                .is_some_and(|draft| !draft.create)
        {
            return self.activate_library_revision(datum_gui_protocol::library_browser::revision_workflow::RevisionControl::SaveCandidate);
        }
        if self
            .workspace()
            .library
            .draft
            .as_ref()
            .is_some_and(|d| d.preview.is_some())
        {
            return self.queue_native_edit(
                datum_gui_protocol::native_edit_client::NativeEditAction::Library,
            );
        }
        self.session.workspace_mut().library.refusal =
            Some("Preview the current fields before committing".into());
        self.invalidate_frame();
        true
    }

    pub(crate) fn activate_library_revision(&mut self, control: C) -> bool {
        let result = self.library_revision_control(control);
        if let Err(error) = result {
            self.session.workspace_mut().library.refusal = Some(error.clone());
            self.log_console_refusal(ConsoleFeedbackSource::Tool, error);
        }
        self.invalidate_frame();
        true
    }

    fn library_revision_control(&mut self, control: C) -> Result<(), String> {
        if self.native_edit.pending() {
            return Err("Wait for the current edit or preview".into());
        }
        if matches!(
            control,
            C::Purpose(_)
                | C::Preview { .. }
                | C::Apply { .. }
                | C::RecordDetails { .. }
                | C::Cancel(_)
        ) {
            self.workspace()
                .library
                .revision_workflow
                .as_ref()
                .ok_or("Revision review is no longer open")?
                .validate_review_control(control)?;
        }
        let root = self
            .workspace()
            .backing
            .as_ref()
            .ok_or("Open a persistent project before editing revisions")?
            .request
            .project_root
            .clone();
        match control {
            C::AdoptProfile(_) | C::RecordSource(_) | C::SaveCandidate | C::SelectUpdate { .. } => {
                let state = self.workspace();
                let browser = &state.library;
                browser.validate_revision_entry(&state.selection, control)?;
                if browser.revision_workflow.is_some() {
                    return Err("Finish or cancel the current revision review".into());
                }
                let workflow = if let C::SelectUpdate { index, .. } = control {
                    LibraryRevisionWorkflow::review_selection(browser, &root, index)?
                } else if matches!(control, C::AdoptProfile(_)) {
                    LibraryRevisionWorkflow::adopt_profile(browser, &root)?
                } else {
                    let entry = browser
                        .selected(&state.selection)
                        .ok_or("Select a local definition")?;
                    let index = browser
                        .entries
                        .iter()
                        .position(|candidate| candidate == entry)
                        .ok_or("Selected definition changed")?;
                    if matches!(control, C::RecordSource(_)) {
                        LibraryRevisionWorkflow::record_local_source(browser, &root, index)?
                    } else {
                        let draft = browser
                            .draft
                            .as_ref()
                            .ok_or("Edit a local definition first")?;
                        if draft.create || draft.id != entry.id {
                            return Err(
                                "Record this definition before creating its next revision".into()
                            );
                        }
                        let raw = draft
                            .preview
                            .as_ref()
                            .ok_or("Preview the current engineering fields first")?;
                        LibraryRevisionWorkflow::save_revision(
                            browser,
                            &root,
                            index,
                            serde_json::to_string(raw).map_err(|error| error.to_string())?,
                        )?
                    }
                };
                let purpose = workflow.purpose_control();
                let browser = &mut self.session.workspace_mut().library;
                browser.revision_workflow = Some(workflow);
                browser.refusal = None;
                browser.scroll_offset = 0;
                browser.reveal_focus = false;
                browser.focus =
                    Some(datum_gui_protocol::library_browser::LibraryControl::Revision(purpose));
            }
            C::Purpose(_) => {}
            C::RecordDetails { .. } => {
                let browser = &mut self.session.workspace_mut().library;
                browser
                    .revision_workflow
                    .as_mut()
                    .ok_or("Revision review is no longer open")?
                    .set_record_details(control)?;
                let next = browser
                    .revision_workflow
                    .as_ref()
                    .unwrap()
                    .review_controls()
                    .into_iter()
                    .find(|control| matches!(control, C::RecordDetails { .. }));
                browser.scroll_offset = 0;
                browser.reveal_focus = false;
                browser.focus =
                    next.map(datum_gui_protocol::library_browser::LibraryControl::Revision);
            }
            C::Preview { .. } => {
                let browser = &self.workspace().library;
                let intent = browser
                    .revision_workflow
                    .as_ref()
                    .ok_or("Open a revision review")?
                    .preview_intent(browser, &root)?;
                let request_id = intent.request_id;
                let client = self.native_edit.client.clone();
                let wake = self.native_edit.wake.clone();
                let (sender, receiver) = mpsc::sync_channel(1);
                std::thread::Builder::new()
                    .name("datum-library-preview".into())
                    .spawn(move || {
                        let result = (|| -> Result<QualifiedLibraryPreview> {
                            let mut owner = client
                                .lock()
                                .map_err(|_| anyhow::anyhow!("Native editor unavailable"))?;
                            if owner.is_none() {
                                *owner = Some(NativeProjectClient::spawn()?);
                            }
                            owner.as_ref().unwrap().preview_library_edit(&intent)
                        })()
                        .map_err(|error| format!("{error:#}"));
                        drop(client);
                        let _ = sender.send(PreviewReply { request_id, result });
                        let _ = wake.send_event(());
                    })
                    .map_err(|error| error.to_string())?;
                self.native_edit.library_pending = Some(receiver);
                self.session.workspace_mut().library.refusal = None;
            }
            C::Apply { .. } => {
                self.queue_native_edit(NativeEditAction::Library);
            }
            C::Cancel(_) => {
                let browser = &mut self.session.workspace_mut().library;
                if browser
                    .revision_workflow
                    .as_ref()
                    .is_some_and(|review| review.is_submitted())
                {
                    return Err("Reconcile the submitted change before closing this review".into());
                }
                browser.revision_workflow = None;
                browser.refusal = None;
                browser.focus = None;
            }
        }
        Ok(())
    }

    pub(super) fn prepare_library_revision_intent(
        &mut self,
        intent: &mut NativeEditIntent,
    ) -> Result<(), String> {
        let mut review = self
            .session
            .workspace_mut()
            .library
            .revision_workflow
            .take()
            .ok_or("Open a revision review")?;
        let result = review.submit(&self.workspace().library, &intent.project_root);
        self.session.workspace_mut().library.revision_workflow = Some(review);
        *intent = result?;
        Ok(())
    }

    pub(super) fn poll_library_revision_preview(&mut self) -> bool {
        let Some(receiver) = &self.native_edit.library_pending else {
            return false;
        };
        let reply = match receiver.try_recv() {
            Ok(reply) => Some(reply),
            Err(mpsc::TryRecvError::Empty) => return false,
            Err(mpsc::TryRecvError::Disconnected) => None,
        };
        self.native_edit.library_pending = None;
        let Some(root) = self
            .workspace()
            .backing
            .as_ref()
            .map(|b| b.request.project_root.clone())
        else {
            return false;
        };
        let Some(reply) = reply else {
            self.session.workspace_mut().library.refusal =
                Some("Library preview worker stopped; no write was submitted".into());
            self.invalidate_frame();
            return true;
        };
        let browser = &self.workspace().library;
        if !browser
            .revision_workflow
            .as_ref()
            .and_then(|review| review.preview_intent(browser, &root).ok())
            .is_some_and(|intent| intent.request_id == reply.request_id)
        {
            return false;
        }
        let mut review = self
            .session
            .workspace_mut()
            .library
            .revision_workflow
            .take()
            .unwrap();
        let result = reply
            .result
            .and_then(|preview| review.accept_preview(&self.workspace().library, &root, preview));
        let accepted = result.is_ok();
        let browser = &mut self.session.workspace_mut().library;
        browser.revision_workflow = Some(review);
        if accepted {
            browser.scroll_offset = 0;
            browser.reveal_focus = false;
        }
        browser.refusal = result.err();
        if accepted {
            self.log_console_echo(
                ConsoleFeedbackSource::Tool,
                "Library revision preview ready; no changes applied",
            );
        }
        self.invalidate_frame();
        true
    }
}
