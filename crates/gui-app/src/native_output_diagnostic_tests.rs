use super::super::tests::{motion, observer, state};
use super::*;
fn diagnostic() -> InputObservation {
    let mut o = observer();
    o.diagnostic = Some(Diagnostic::default());
    o
}
#[test]
fn irrelevant_rounds_reuse_provisional_storage_and_real_transitions_survive() {
    let mut o = diagnostic();
    let mut current = state();
    for _ in 0..LIMIT + 10 {
        assert!(o.begin_demand(None, current));
        current.render_revision += 1;
        current.render_activity[0] += 1;
        o.complete(Some(current));
    }
    assert!(o.records.is_empty());
    assert!(o.begin_demand(None, current));
    current.cursor = Some([1., 2.]);
    o.complete(Some(current));
    assert_eq!(o.records.len(), 1);
    assert_eq!(o.records[0].demand_kind, "native_round");
    for event in [
        motion(),
        WindowEvent::Focused(false),
        WindowEvent::CursorLeft {
            device_id: winit::event::DeviceId::dummy(),
        },
    ] {
        assert!(o.begin(&event, current));
        o.route("authoring_hover");
        o.complete(Some(current));
    }
    assert_eq!(
        o.records.iter().map(|r| r.demand_kind).collect::<Vec<_>>(),
        ["native_round", "pointer", "focus_loss", "cursor_leave"]
    );
    assert!(o.complete_receipt());
}
#[test]
fn full_capacity_noop_succeeds_but_retained_input_and_round_fail_explicitly() {
    for event in [Some(motion()), None] {
        let mut o = diagnostic();
        for _ in 0..LIMIT {
            assert!(o.begin(&motion(), state()));
            o.complete(Some(state()));
        }
        let cap = o.records.capacity();
        assert!(o.begin_demand(None, state()));
        o.complete(Some(state()));
        assert!(o.complete_receipt());
        if let Some(event) = event {
            assert!(
                !o.begin(&event, state()),
                "failed admission must not dispatch"
            );
        } else {
            assert!(o.begin_demand(None, state()));
            let mut after = state();
            after.cursor = Some([1., 2.]);
            o.complete(Some(after));
        }
        assert_eq!(o.records.capacity(), cap);
        assert!(!o.complete_receipt());
        let report = o.diagnostic_report(Some(state()));
        assert_eq!(
            report["first_error"],
            "diagnostic record capacity exhausted"
        );
        assert!(!report["pending"].is_null());
        assert_eq!(report["complete"], false);
    }
}
#[test]
fn coverage_gap_and_option_conflicts_are_failures_not_gpu_contexts() {
    let mut o = diagnostic();
    assert!(o.begin_demand(None, state()));
    o.complete(Some(state()));
    let mut changed = state();
    changed.focused = false;
    assert!(!o.begin(&motion(), changed));
    assert_eq!(
        o.diagnostic_report(Some(changed))["first_error"],
        "unobserved semantic state transition"
    );
    assert!(validate_options(true, Some("0"), false).is_ok());
    for (gpu, workload) in [(Some("1"), false), (Some("0"), true), (None, false)] {
        assert!(validate_options(true, gpu, workload).is_err());
    }
}
#[test]
fn diagnostic_snapshot_requires_no_drain_or_runtime_and_survives_later_errors() {
    let mut o = diagnostic();
    o.path = std::env::temp_dir()
        .join(format!("datum-output-{}", uuid::Uuid::new_v4()))
        .join("input-receipt.json");
    std::fs::create_dir(o.path.parent().unwrap()).unwrap();
    assert!(o.begin(&motion(), state()));
    o.complete(Some(state()));
    o.write_diagnostic(Some(state())).unwrap();
    let path = o.path.with_file_name("input-diagnostic.json");
    let bytes = std::fs::read(&path).unwrap();
    let report: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(report["coverage_complete"], true);
    assert_eq!(report["complete"], false);
    assert!(report["gpu_drained"].is_null());
    // A subsequent handshake/drain error cannot remove the separate snapshot.
    let later: Result<()> = Err(anyhow::anyhow!("missing handshake"));
    assert!(later.is_err());
    assert_eq!(std::fs::read(&path).unwrap(), bytes);
    let missing = o.diagnostic_report(None);
    assert_eq!(missing["coverage_complete"], false);
    assert_eq!(missing["complete"], false);
    std::fs::remove_dir_all(o.path.parent().unwrap()).unwrap();
}
