//! Read-only exact Run queries leave the native input thread runnable.
use super::*;
use datum_gui_protocol::selection_resolution::NativeSelectionResolution;
use datum_gui_protocol::{AuthoredSelectionIdentity, SelectionSubject};
use eda_engine::{ir::geometry::Point, substrate::DesignModel};
use std::sync::{Arc, mpsc};

#[derive(Clone)]
pub(super) struct Request {
    pub model: Arc<DesignModel>,
    pub origin: AuthoredSelectionIdentity,
    pub point: Option<Point>,
    pub depth: u8,
}
impl Request {
    fn same_query(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.model, &other.model)
            && self.origin == other.origin
            && self.point == other.point
            && self.depth == other.depth
    }
    fn matches_model(&self, current: &DesignModel) -> bool {
        current.project.project_id == self.model.project.project_id
            && current.model_revision == self.model.model_revision
    }
}

type Complete = (Arc<NativeSelectionResolution>, SelectionSubject);
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
                    let authority = NativeSelectionResolution::capture(&model)?;
                    let mut subject = authority.acquire_run(origin, point)?;
                    if depth >= 3 {
                        let SelectionSubject::Run(run) = &subject else {
                            unreachable!()
                        };
                        subject = authority.widen_run(run)?;
                    }
                    Ok((Arc::new(authority), subject))
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
        match answer {
            Ok((authority, subject)) => {
                native.authority = authority;
                crate::append_gui_verbose_diagnostic_line(
                    || "native Run complete result ready for publication",
                );
                self.publish_native_acquisition(
                    [request.origin].into_iter().collect(),
                    datum_gui_viewport::selection_acquisition::AcquisitionMode::Replace,
                    datum_gui_viewport::selection_acquisition::AcquisitionEntry::Individual,
                    Some(subject),
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
mod tests {
    use super::*;
    use crate::terminal_accessibility_platform::selection_fixture as fixture;
    use datum_gui_protocol::AuthoredSelectionClass;
    fn setup() -> (fixture::Fixture, Request) {
        let mut f = fixture::Fixture::new("run-worker");
        f.line(fixture::id(1), Point::new(0, 0), Point::new(10, 0));
        f.line(fixture::id(2), Point::new(10, 0), Point::new(20, 0));
        let model = Arc::new(
            eda_engine::substrate::ProjectResolver::new(&f.root)
                .resolve()
                .unwrap(),
        );
        let request = Request {
            model,
            origin: AuthoredSelectionIdentity {
                class: AuthoredSelectionClass::Track,
                id: fixture::id(1),
                instance_path: vec![],
            },
            point: None,
            depth: 2,
        };
        (f, request)
    }
    fn work() -> RunWork {
        RunWork {
            generation: 0,
            active: None,
            queued: None,
            completed: None,
            wake: None,
        }
    }
    fn await_result(work: &mut RunWork) -> (Request, Answer) {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
        loop {
            if let Some(result) = work.poll() {
                return result;
            }
            assert!(
                std::time::Instant::now() < deadline,
                "worker failed to complete"
            );
            std::thread::yield_now();
        }
    }
    #[test]
    fn worker_delivers_complete_authoritative_run_and_cancel_discards_completion() {
        let (mut f, request) = setup();
        let guard = request.clone();
        let expected = NativeSelectionResolution::capture(&request.model).unwrap();
        let subject = expected.acquire_run(request.origin.clone(), None).unwrap();
        let projection = expected.resolve(&subject).unwrap();
        assert_eq!(projection.members.len(), 2);
        let mut work = work();
        work.schedule(request).unwrap();
        let (_, answer) = await_result(&mut work);
        let (authority, actual) = answer.unwrap();
        assert_eq!(actual, subject);
        assert_eq!(authority.resolve(&actual).unwrap(), projection);
        assert!(guard.matches_model(&f.model));
        f.line(fixture::id(3), Point::new(20, 0), Point::new(30, 0));
        assert!(
            !guard.matches_model(&f.model),
            "changed canonical source must discard the result"
        );
        let (_f2, request2) = setup();
        work.schedule(request2).unwrap();
        work.cancel();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
        while work.active.is_some() {
            assert!(work.poll().is_none(), "cancelled Run must not publish");
            assert!(std::time::Instant::now() < deadline);
            std::thread::yield_now();
        }
    }
    #[test]
    fn identical_completed_query_reuses_authority_and_retains_cancellation_guards() {
        let (_f, request) = setup();
        let mut work = work();
        work.schedule(request.clone()).unwrap();
        let (_, first) = await_result(&mut work);
        let first = first.unwrap();
        work.schedule(request.clone()).unwrap();
        let (_, reused) = work.poll().expect("completed query needs no worker wait");
        let reused = reused.unwrap();
        assert!(Arc::ptr_eq(&first.0, &reused.0));
        assert_eq!(first.1, reused.1);
        assert_eq!(
            first.0.resolve(&first.1).unwrap(),
            reused.0.resolve(&reused.1).unwrap()
        );
        let mut changed = request.clone();
        changed.point = Some(Point::new(5, 0));
        assert!(!request.same_query(&changed));
        changed = request.clone();
        changed.depth = 3;
        assert!(!request.same_query(&changed));
        changed = request.clone();
        changed.model = Arc::new((*request.model).clone());
        assert!(
            !request.same_query(&changed),
            "different captured preimage cannot reuse"
        );
        work.schedule(request).unwrap();
        work.cancel();
        assert!(
            work.poll().is_none(),
            "cached completion must honor cancellation"
        );
    }
    #[test]
    fn busy_worker_retains_only_latest_superseding_request() {
        let (_f, request) = setup();
        let model = request.model.clone();
        let mut work = work();
        // A held completion models an exact query still running while GUI input
        // replaces its request repeatedly; no extra worker can be started.
        let (sender, receiver) = mpsc::sync_channel(1);
        work.active = Some(Active {
            ticket: 0,
            request,
            receiver,
        });
        for id in [1, 2, 1, 2] {
            work.schedule(Request {
                model: model.clone(),
                origin: AuthoredSelectionIdentity {
                    class: AuthoredSelectionClass::Track,
                    id: fixture::id(id),
                    instance_path: vec![],
                },
                point: None,
                depth: 2,
            })
            .unwrap();
        }
        assert_eq!(work.queued.as_ref().unwrap().origin.id, fixture::id(2));
        assert_eq!(work.active.as_ref().unwrap().ticket, 0);
        sender.send(Err("obsolete".into())).unwrap();
        assert!(work.poll().is_none());
        let (request, answer) = await_result(&mut work);
        assert_eq!(request.origin.id, fixture::id(2));
        assert!(matches!(answer.unwrap().1, SelectionSubject::Run(_)));
    }
}
