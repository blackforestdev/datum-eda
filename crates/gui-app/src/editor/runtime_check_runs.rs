//! Shared explicit ERC job owner. Inspection never invokes this writer.
use super::*;
use datum_gui_protocol::{CheckProducerAcceptance, CheckProducerCandidate, CheckProducerRequest};
use std::sync::mpsc;

struct Ready {
    run: datum_gui_protocol::CheckRunReviewState,
    next: datum_gui_protocol::ReviewWorkspaceState,
    selection: native_edit_selection::Prepared,
}
enum Event {
    Accepted(CheckProducerAcceptance),
    Finished(Result<Box<Ready>, String>),
}

pub(super) struct CheckWork {
    pending: Option<(CheckProducerRequest, mpsc::Receiver<Event>)>,
    wake: winit::event_loop::EventLoopProxy<()>,
}

fn prepare_readback(
    request: &CheckProducerRequest,
    mut next: datum_gui_protocol::ReviewWorkspaceState,
    inputs: native_edit_selection::Inputs,
) -> Result<Box<Ready>> {
    anyhow::ensure!(
        request.matches_workspace(&next),
        "Project changed during ERC source refresh"
    );
    let model = next
        .library
        .basis
        .clone()
        .ok_or_else(|| anyhow::anyhow!("ERC source model is unavailable"))?;
    let selection =
        native_edit_selection::prepare_from_model(&mut next, inputs, model, Default::default())?;
    let run = next.checks.clone();
    Ok(Box::new(Ready {
        run,
        next,
        selection,
    }))
}
impl CheckWork {
    pub fn new(wake: winit::event_loop::EventLoopProxy<()>) -> Self {
        Self {
            pending: None,
            wake,
        }
    }
    pub fn pending(&self) -> bool {
        self.pending.is_some()
    }
    fn start(
        &mut self,
        request: CheckProducerRequest,
        previous: datum_gui_protocol::ReviewWorkspaceState,
        inputs: native_edit_selection::Inputs,
    ) -> Result<()> {
        anyhow::ensure!(!self.pending(), "An ERC job is already running");
        self.spawn(request, previous, inputs, None)
    }
    fn spawn(
        &mut self,
        request: CheckProducerRequest,
        previous: datum_gui_protocol::ReviewWorkspaceState,
        inputs: native_edit_selection::Inputs,
        persisted: Option<datum_gui_protocol::CheckRunReviewState>,
    ) -> Result<()> {
        // One outstanding typed event: actual acceptance, then final readback.
        let (sender, receiver) = mpsc::sync_channel(1);
        let wake = self.wake.clone();
        let work = request.clone();
        std::thread::Builder::new()
            .name("datum-erc-job".into())
            .spawn(move || {
                let result =
                    (|| -> Result<Box<Ready>> {
                        let next = match persisted {
                        Some(run) => datum_gui_protocol::workspace_loading::load_check_completion_workspace(previous, &work, &run)?, // Navigation reconciliation is read-only.
                        None => CheckProducerCandidate::produce(&work, |accepted| {
                                let _ = sender.send(Event::Accepted(accepted));
                                let _ = wake.send_event(());
                            })?.readback(previous, &work)?,
                    };
                        prepare_readback(&work, next, inputs)
                    })()
                    .map_err(|error| format!("{error:#}"));
                let _ = sender.send(Event::Finished(result));
                let _ = wake.send_event(());
            })?;
        self.pending = Some((request, receiver));
        Ok(())
    }
}
impl Runtime {
    pub(super) fn start_erc_job(&mut self) -> bool {
        let result = (|| -> Result<()> {
            anyhow::ensure!(
                self.application_shutdown_started.is_none(),
                "Datum is closing"
            );
            anyhow::ensure!(
                !self.native_edit.pending() && !self.project_open.pending(),
                "Wait for the current Project operation before running ERC"
            );
            anyhow::ensure!(
                self.native_edit.preview.is_none()
                    && !self.workspace().schematic_authoring.active()
                    && !self.workspace().schematic_authoring.picker
                    && self.workspace().library.draft.is_none(),
                "Finish or cancel the current editor preview before running ERC"
            );
            let request = CheckProducerRequest::from_workspace(self.workspace())?;
            let previous = self.workspace().clone();
            let inputs = native_edit_selection::Inputs::capture(self);
            self.check_work.start(request.clone(), previous, inputs)?;
            self.session.workspace_mut().check_producer.begin(request);
            Ok(())
        })();
        if let Err(error) = result {
            self.log_console_refusal(ConsoleFeedbackSource::Viewport, error.to_string());
        }
        self.refresh_selection_accessibility();
        self.invalidate_frame();
        true
    }

    pub(super) fn poll_check_job(&mut self) -> bool {
        let Some((request, receiver)) = self.check_work.pending.as_ref() else {
            return false;
        };
        let request = request.clone();
        let mut events = Vec::new();
        let disconnected = loop {
            match receiver.try_recv() {
                Ok(event) => events.push(event),
                Err(mpsc::TryRecvError::Empty) => break false,
                Err(mpsc::TryRecvError::Disconnected) => break true,
            }
        };
        if events.is_empty() && !disconnected {
            return false;
        }
        let mut finished = false;
        for event in events {
            let current = request.matches_workspace(self.workspace());
            match event {
                Event::Accepted(value) => {
                    let producer = &mut self.session.workspace_mut().check_producer;
                    producer.accept(value);
                    if !current {
                        producer.reason = Some("ERC is running on its admitted previous source basis; completion cannot become current evidence here".into());
                    }
                }
                Event::Finished(result) => {
                    finished = true;
                    match result {
                        Ok(answer) if current => {
                            let answer = *answer;
                            if !answer.selection.inputs.matches(self) {
                                let inputs = native_edit_selection::Inputs::capture(self);
                                let previous = self.workspace().clone();
                                match self.check_work.spawn(
                                    request.clone(),
                                    previous,
                                    inputs,
                                    Some(answer.run),
                                ) {
                                    Ok(()) => {
                                        self.invalidate_frame();
                                        return true;
                                    }
                                    Err(error) => self
                                        .session
                                        .workspace_mut()
                                        .check_producer
                                        .refuse(error.to_string()),
                                }
                            } else {
                                self.publish_prepared_workspace(answer.next, answer.selection);
                                if !self
                                    .session
                                    .workspace_mut()
                                    .check_producer
                                    .complete(&request, &answer.run)
                                {
                                    self.session.workspace_mut().check_producer.refuse("ERC completion no longer matches its admitted source basis".into());
                                }
                            }
                        }
                        Ok(_) => self.session.workspace_mut().check_producer.refuse(
                            "ERC finished on another source basis; refresh the Project".into(),
                        ),
                        Err(reason) => self.session.workspace_mut().check_producer.refuse(reason),
                    }
                }
            }
        }
        if disconnected && !finished {
            self.session
                .workspace_mut()
                .check_producer
                .refuse("ERC worker ended without a result".into());
        }
        if finished || disconnected {
            self.check_work.pending = None;
        }
        self.refresh_selection_accessibility();
        self.invalidate_frame();
        true
    }
}
