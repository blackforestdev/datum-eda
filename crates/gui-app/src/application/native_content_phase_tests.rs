use crate::native_content_probe::Phase;

#[test]
fn native_probe_arms_only_sixth_prescribed_wheel_without_outcome_search() {
    let mut phase = Phase::Zoom(0);
    for expected in 1..6 {
        phase.wheel(true).unwrap();
        assert_eq!(phase, Phase::Zoom(expected));
    }
    phase.wheel(true).unwrap();
    assert_eq!(phase, Phase::BaselineArmed);
    assert!(phase.wheel(true).is_err());
    assert!(Phase::Zoom(0).wheel(false).is_err());
}

#[test]
fn native_probe_release_is_fixed_boundary_not_desired_selection_predicate() {
    let mut phase = Phase::ReleasedReady;
    let mut held = false;
    phase.primary(true, true, &mut held).unwrap();
    phase.primary(false, true, &mut held).unwrap();
    assert_eq!(phase, Phase::ReleasedArmed);
    assert!(phase.primary(false, true, &mut held).is_err());
    let mut before_baseline = Phase::BaselinePending;
    assert!(before_baseline.primary(true, true, &mut false).is_err());
}

#[test]
fn native_probe_startup_and_finished_cleanup_do_not_arm_extra_slots() {
    let mut phase = Phase::Zoom(0);
    phase.primary(false, false, &mut false).unwrap();
    assert_eq!(phase, Phase::Zoom(0));
    let mut phase = Phase::Done;
    phase.primary(true, false, &mut false).unwrap();
    phase.wheel(false).unwrap();
    assert_eq!(phase, Phase::Done);
}
