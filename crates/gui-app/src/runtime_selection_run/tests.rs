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
        inputs: Arc::new(ProjectionInputs::capture(
            &datum_gui_protocol::load_fixture_workspace_state(),
        )),
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
    assert_eq!(actual.subject, subject);
    assert_eq!(authority.resolve(&actual.subject).unwrap(), projection);
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
        first.0.resolve(&first.1.subject).unwrap(),
        reused.0.resolve(&reused.1.subject).unwrap()
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
fn changed_visibility_reprojects_complete_view_without_requerying_geometry() {
    let (_f, request) = setup();
    let mut work = work();
    work.schedule(request.clone()).unwrap();
    let (_, original) = await_result(&mut work);
    let (authority, view) = original.unwrap();
    let mut state = datum_gui_protocol::load_fixture_workspace_state();
    state.ui.filters.show_authored = false;
    assert!(!request.inputs.matches_workspace(&state));
    let updated = Request {
        inputs: Arc::new(ProjectionInputs::capture(&state)),
        ..request.clone()
    };
    assert!(request.same_source_query(&updated));
    assert!(!request.same_query(&updated));
    work.schedule(updated).unwrap();
    let (_, prepared) = await_result(&mut work);
    let (reused, actual) = prepared.unwrap();
    assert!(Arc::ptr_eq(&authority, &reused));
    let expected = datum_gui_protocol::selection_artifacts::build_workspace_selection_view(
        &request.model,
        &authority,
        view.subject,
        &state,
    )
    .unwrap();
    assert_eq!(actual, expected);
    assert_eq!(actual.members.len(), 2);
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
            inputs: Arc::new(ProjectionInputs::capture(
                &datum_gui_protocol::load_fixture_workspace_state(),
            )),
        })
        .unwrap();
    }
    assert_eq!(work.queued.as_ref().unwrap().origin.id, fixture::id(2));
    assert_eq!(work.active.as_ref().unwrap().ticket, 0);
    sender.send(Err("obsolete".into())).unwrap();
    assert!(work.poll().is_none());
    let (request, answer) = await_result(&mut work);
    assert_eq!(request.origin.id, fixture::id(2));
    assert!(matches!(
        answer.unwrap().1.subject,
        SelectionSubject::Run(_)
    ));
}
