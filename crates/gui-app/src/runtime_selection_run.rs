//! Read-only exact Run queries leave the native input thread runnable.
use super::*;
use datum_gui_protocol::selection_resolution::NativeSelectionResolution;
use datum_gui_protocol::{AuthoredSelectionIdentity, SelectionSubject};
use eda_engine::{ir::geometry::Point, substrate::DesignModel};
use std::sync::{Arc, mpsc};
mod projection;
pub(super) use projection::ProjectionInputs;

#[derive(Clone)]
pub(super) struct Request {
    pub model: Arc<DesignModel>,
    pub origin: AuthoredSelectionIdentity,
    pub point: Option<Point>,
    pub depth: u8,
    pub inputs: Arc<ProjectionInputs>,
}
impl Request {
    fn same_source_query(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.model, &other.model)
            && self.origin == other.origin
            && self.point == other.point
            && self.depth == other.depth
    }
    fn same_query(&self, other: &Self) -> bool {
        self.same_source_query(other) && self.inputs.same_projection(&other.inputs)
    }
    fn matches_model(&self, current: &DesignModel) -> bool {
        current.project.project_id == self.model.project.project_id
            && current.model_revision == self.model.model_revision
    }
}

type Complete = (
    Arc<NativeSelectionResolution>,
    datum_gui_protocol::selection_view::SelectionView,
);
type Answer = Result<Complete, String>;
struct Active {
    ticket: u64,
    request: Request,
    receiver: mpsc::Receiver<Answer>,
}
pub(super) struct RunWork {
    generation: u64,
    active: Option<Active>,
    queued: Option<Request>,
    completed: Option<(Request, Complete)>,
    wake: Option<winit::event_loop::EventLoopProxy<()>>,
}
impl RunWork {
    pub(super) fn new(wake: winit::event_loop::EventLoopProxy<()>) -> Self {
        Self {
            generation: 0,
            active: None,
            queued: None,
            completed: None,
            wake: Some(wake),
        }
    }
    // Cancel publication, without joining or interrupting an exact geometry
    // query. There is at most one worker and one superseding request.
    pub(super) fn cancel(&mut self) {
        self.generation = self.generation.wrapping_add(1);
        self.queued = None;
    }
    pub(super) fn schedule(&mut self, request: Request) -> Result<(), String> {
        self.cancel();
        if self.active.is_some() {
            self.queued = Some(request);
            Ok(())
        } else {
            self.start(request)
        }
    }
    fn start(&mut self, request: Request) -> Result<(), String> {
        if let Some((previous, answer)) = &self.completed
            && previous.same_query(&request)
        {
            let (sender, receiver) = mpsc::sync_channel(1);
            let _ = sender.send(Ok(answer.clone()));
            self.active = Some(Active {
                ticket: self.generation,
                request,
                receiver,
            });
            crate::append_gui_verbose_diagnostic_line(
                || "native Run reused complete identical query",
            );
            return Ok(());
        }
        let reusable = self
            .completed
            .as_ref()
            .filter(|(previous, _)| previous.same_source_query(&request))
            .map(|(_, (authority, view))| (authority.clone(), view.subject.clone()));
        let inputs = request.inputs.clone();
        let model = request.model.clone();
        let origin = request.origin.clone();
        let point = request.point;
        let depth = request.depth;
        let wake = self.wake.clone();
        let (sender, receiver) = mpsc::sync_channel(1);
        std::thread::Builder::new()
            .name("datum-selection-run".into())
            .spawn(move || {
                let started = std::time::Instant::now();
                crate::append_gui_verbose_diagnostic_line(|| "native Run worker query begin");
                let answer = (|| {
                    let (authority, subject) = if let Some(reused) = reusable {
                        reused
                    } else {
                        let authority = Arc::new(NativeSelectionResolution::capture(&model)?);
                        let mut subject = authority.acquire_run(origin, point)?;
                        if depth >= 3 {
                            let SelectionSubject::Run(run) = &subject else {
                                unreachable!()
                            };
                            subject = authority.widen_run(run)?;
                        }
                        (authority, subject)
                    };
                    let view = inputs.build(&model, &authority, subject)?;
                    Ok((authority, view))
                })()
                .map_err(
                    |error: datum_gui_protocol::selection_resolution::SelectionResolutionError| {
                        format!("Electrical selection refused: {error:?}")
                    },
                );
                crate::append_gui_verbose_diagnostic_line(|| {
                    format!(
                        "native Run worker query end {}us",
                        started.elapsed().as_micros()
                    )
                });
                let _ = sender.send(answer);
                if let Some(wake) = wake {
                    let _ = wake.send_event(());
                }
            })
            .map_err(|error| format!("Electrical selection worker unavailable: {error}"))?;
        self.active = Some(Active {
            ticket: self.generation,
            request,
            receiver,
        });
        crate::append_gui_verbose_diagnostic_line(|| "native Run query scheduled off input thread");
        Ok(())
    }
    fn poll(&mut self) -> Option<(Request, Answer)> {
        let active = self.active.as_ref()?;
        let answer = match active.receiver.try_recv() {
            Ok(answer) => answer,
            Err(mpsc::TryRecvError::Empty) => return None,
            Err(mpsc::TryRecvError::Disconnected) => {
                Err("Electrical selection worker stopped".into())
            }
        };
        let active = self.active.take().unwrap();
        let current = active.ticket == self.generation;
        if current && let Ok(answer) = &answer {
            self.completed = Some((active.request.clone(), answer.clone()));
        }
        if let Some(request) = self.queued.take()
            && let Err(error) = self.start(request.clone())
        {
            // A failed launch must be reported, never leave a pending selection.
            return Some((request, Err(error)));
        }
        current.then_some((active.request, answer))
    }
}
impl Runtime {
    pub(super) fn poll_native_run(&mut self) -> bool {
        let Some((request, answer)) = self.selection_run.poll() else {
            return false;
        };
        let _probe = crate::gui_runtime_support::phase_probe::Probe::start("run_publication");
        // The immutable query proves complete membership. Publication separately
        // rechecks canonical source bytes after intervening native input/writes.
        match self.ensure_selection_authority() {
            Ok(true) => {}
            Ok(false) => return false,
            Err(error) => {
                self.selection_notice(error.to_string());
                return true;
            }
        }
        let Some(native) = self.native_selection.as_mut() else {
            return false;
        };
        if !request.matches_model(&native.model) {
            return false;
        }
        if !request.inputs.matches_workspace(self.workspace()) {
            let inputs = Arc::new(ProjectionInputs::capture(self.workspace()));
            if let Err(error) = self.selection_run.schedule(Request { inputs, ..request }) {
                self.selection_notice(error);
                return true;
            }
            return false;
        }
        match answer {
            Ok((authority, view)) => {
                self.native_selection.as_mut().unwrap().authority = authority;
                crate::append_gui_verbose_diagnostic_line(
                    || "native Run complete result ready for publication",
                );
                self.publish_native_acquisition_prepared(
                    [request.origin].into_iter().collect(),
                    datum_gui_viewport::selection_acquisition::AcquisitionMode::Replace,
                    datum_gui_viewport::selection_acquisition::AcquisitionEntry::Individual,
                    Some(view.subject.clone()),
                    Some(view),
                )
            }
            Err(error) => {
                self.selection_notice(error);
                true
            }
        }
    }
}

#[cfg(test)]
mod tests;
