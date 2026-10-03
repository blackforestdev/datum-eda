use datum_gui_protocol::{PaneId, PointNm};
use datum_gui_viewport::{
    selection_acquisition::AcquisitionMode,
    selection_gesture::{RegionKind, SelectionGesture},
};
fn p(x: i64, y: i64) -> PointNm {
    PointNm { x, y }
}
#[test]
fn physical_threshold_shape_lock_and_world_anchor_are_one_shared_grammar() {
    let mut g = SelectionGesture::begin(
        PaneId(4),
        (10.0, 10.0),
        p(100, 100),
        true,
        false,
        false,
        "scene".into(),
    )
    .unwrap();
    assert!(!g.motion((13.99, 10.0), p(120, 100)));
    assert_eq!(g.kind, None);
    assert!(g.motion((14.0, 10.0), p(140, 100)));
    assert_eq!(g.kind, Some(RegionKind::Rectangle));
    g.motion((0.0, 20.0), p(-20, 500));
    assert_eq!(g.kind, Some(RegionKind::Rectangle));
    assert_eq!(g.origin, p(100, 100));
    assert_eq!(g.pane, PaneId(4));
    assert_eq!(
        g.boundary(),
        vec![p(100, 100), p(-20, 100), p(-20, 500), p(100, 500)]
    );
}
#[test]
fn initial_leftward_lasso_and_subtraction_are_not_changed_by_later_direction() {
    let mut g = SelectionGesture::begin(
        PaneId(1),
        (0.0, 0.0),
        p(0, 0),
        false,
        true,
        false,
        "scene".into(),
    )
    .unwrap();
    g.motion((-4.0, 0.0), p(-4, 0));
    g.motion((8.0, 2.0), p(8, 2));
    assert_eq!(g.kind, Some(RegionKind::Lasso));
    assert_eq!(g.mode, AcquisitionMode::Remove);
    assert_eq!(g.boundary().len(), 3);
}
#[test]
fn pan_and_dual_modifier_own_press_and_plain_drag_has_no_region_operation() {
    assert!(
        SelectionGesture::begin(
            PaneId(1),
            (0.0, 0.0),
            p(0, 0),
            true,
            false,
            true,
            "scene".into()
        )
        .is_none()
    );
    assert!(
        SelectionGesture::begin(
            PaneId(1),
            (0.0, 0.0),
            p(0, 0),
            true,
            true,
            false,
            "scene".into()
        )
        .is_none()
    );
    let mut plain = SelectionGesture::begin(
        PaneId(1),
        (0.0, 0.0),
        p(0, 0),
        false,
        false,
        false,
        "scene".into(),
    )
    .unwrap();
    plain.motion((4.0, 0.0), p(4, 0));
    assert_eq!(plain.mode, AcquisitionMode::Replace);
}
#[test]
fn auto_pan_band_is_owned_and_supports_diagonal_motion() {
    let mut g = SelectionGesture::begin(
        PaneId(2),
        (50.0, 50.0),
        p(0, 0),
        true,
        false,
        false,
        "scene".into(),
    )
    .unwrap();
    g.motion((54.0, 50.0), p(4, 0));
    assert_eq!(
        g.edge_pan((50.0, 50.0), [0.0, 0.0, 100.0, 100.0]),
        (0.0, 0.0)
    );
    let d = g.edge_pan((1.0, 1.0), [0.0, 0.0, 100.0, 100.0]);
    assert!(d.0 > 0.0 && d.1 > 0.0);
    let farther = g.edge_pan((-10.0, -10.0), [0.0, 0.0, 100.0, 100.0]);
    assert!(farther.0 > d.0 && farther.1 > d.1);
}
