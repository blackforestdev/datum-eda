use super::*;
use crate::terminal_accessibility_platform::selection_fixture as fixture;
use datum_gui_protocol::{
    AuthoredSelectionClass as Class, PointNm, selection_resolution::NativeSelectionResolution,
};
use eda_engine::{board::Track, ir::geometry::Point, substrate::Operation};
fn setup() -> (fixture::Fixture, Arc<RegionBasis>, WorkspaceFilterState) {
    let mut f = fixture::Fixture::new("bounded-native-region");
    let tracks: Vec<_> = (1..=300)
        .map(|n| {
            Track::straight(
                fixture::id(n),
                f.net,
                Point::new(n as i64 * 20, 0),
                Point::new(n as i64 * 20 + 10, 0),
                2,
                1,
            )
        })
        .collect();
    f.commit(tracks.iter().map(|track| Operation::CreateBoardTrack {
        track_id: track.uuid,
        track: serde_json::to_value(track).unwrap(),
    }));
    let authority = NativeSelectionResolution::capture(&f.model).unwrap();
    let mut state = datum_gui_protocol::load_fixture_workspace_state();
    let scene = &mut state.scene;
    scene.project_uuid = f.model.project.project_id.to_string();
    scene.board_uuid = f.board.to_string();
    scene.source_revision = f.model.model_revision.0.clone();
    scene.components.clear();
    scene.pads.clear();
    scene.vias.clear();
    scene.zones.clear();
    scene.board_texts.clear();
    scene.outline.clear();
    scene.board_graphics.clear();
    let template = scene.tracks[0].clone();
    scene.tracks = tracks
        .iter()
        .map(|track| {
            let mut projected = template.clone();
            projected.object_id = format!("untrusted-display-{}", track.uuid);
            projected.source_object_uuid = track.uuid.to_string();
            projected.track_uuid = track.uuid.to_string();
            // Deliberately incorrect display geometry cannot establish qualification.
            projected.path = vec![
                PointNm {
                    x: 1_000_000,
                    y: 1_000_000,
                },
                PointNm {
                    x: 2_000_000,
                    y: 2_000_000,
                },
            ];
            projected
        })
        .collect();
    let basis = Arc::new(RegionBasis::capture(&f.model, &authority, scene));
    (f, basis, state.ui.filters)
}
fn boundary() -> Vec<PointNm> {
    vec![
        PointNm { x: 0, y: -10 },
        PointNm { x: 7000, y: -10 },
        PointNm { x: 7000, y: 10 },
        PointNm { x: 0, y: 10 },
    ]
}
#[test]
fn native_region_evaluation_is_source_qualified_complete_and_atomic_across_ticks() {
    let (f, basis, filters) = setup();
    let bytes = f.source_bytes();
    let mut evaluation = RegionEvaluation::begin(
        PaneId(0),
        AcquisitionMode::Add,
        f.model.model_revision.0.clone(),
        basis,
        filters,
        &boundary(),
    )
    .unwrap();
    assert!(evaluation.advance_with_budget(7).unwrap().is_none());
    let result = loop {
        if let Some(result) = evaluation.advance_with_budget(7).unwrap() {
            break result;
        }
    };
    assert_eq!(result.len(), 300);
    assert_eq!(
        result.iter().map(|id| id.id).collect::<BTreeSet<_>>(),
        (1..=300).map(fixture::id).collect()
    );
    assert!(
        result
            .iter()
            .all(|id| id.class == Class::Track && id.instance_path.is_empty())
    );
    fixture::Fixture::assert_bytes(&bytes);
}
#[test]
fn native_region_evaluation_filters_origins_and_cancel_keeps_prior_authority() {
    let (f, basis, mut filters) = setup();
    let bytes = f.source_bytes();
    let prior = BTreeSet::from([Id {
        class: Class::Track,
        id: fixture::id(1),
        instance_path: vec![],
    }]);
    let mut pending = RegionEvaluation::begin(
        PaneId(0),
        AcquisitionMode::Remove,
        f.model.model_revision.0.clone(),
        basis.clone(),
        filters.clone(),
        &boundary(),
    )
    .unwrap();
    assert!(pending.advance_with_budget(7).unwrap().is_none());
    drop(pending);
    assert_eq!(prior.len(), 1);
    filters.excluded_selection_classes.insert(Class::Track);
    let mut excluded = RegionEvaluation::begin(
        PaneId(0),
        AcquisitionMode::Add,
        f.model.model_revision.0.clone(),
        basis,
        filters,
        &boundary(),
    )
    .unwrap();
    assert_eq!(excluded.advance().unwrap().unwrap().len(), 0);
    fixture::Fixture::assert_bytes(&bytes);
}
#[test]
fn region_source_bound_rounds_rational_layout_outwards_and_refuses_overflow() {
    use datum_gui_protocol::selection_geometry::Rational as R;
    let p = |x, y| [R::new(x, 3).unwrap(), R::integer(y)];
    assert_eq!(
        geometry_bounds(&Geometry::Layout(vec![p(-1, -2), p(1, 2)])).unwrap(),
        Some(RectNm {
            min_x: -1,
            max_x: 1,
            min_y: -2,
            max_y: 2
        })
    );
    assert!(
        geometry_bounds(&Geometry::Disk {
            center: PointNm { x: i64::MAX, y: 0 },
            radius: 1
        })
        .is_err()
    );
    assert_eq!(geometry_bounds(&Geometry::Filled(vec![])).unwrap(), None);
}

#[test]
fn native_region_missing_geometry_refuses_atomically_instead_of_omitting_member() {
    let (f, basis, filters) = setup();
    let mut basis = Arc::try_unwrap(basis).unwrap();
    let source = &basis.index.regions()[0].target;
    basis.unresolved.push((
        source.id.clone(),
        source.layers.clone(),
        "Certified geometry unavailable".into(),
    ));
    assert!(
        matches!(RegionEvaluation::begin(PaneId(0),AcquisitionMode::Add,f.model.model_revision.0.clone(),Arc::new(basis),filters,&boundary()),Err(reason) if reason=="Certified geometry unavailable")
    );
}

#[test]
fn native_degenerate_region_is_empty_even_when_unresolved_source_exists() {
    let (f, basis, filters) = setup();
    let mut basis = Arc::try_unwrap(basis).unwrap();
    let source = &basis.index.regions()[0].target;
    basis.unresolved.push((
        source.id.clone(),
        source.layers.clone(),
        "Unavailable".into(),
    ));
    let mut evaluation = RegionEvaluation::begin(
        PaneId(0),
        AcquisitionMode::Add,
        f.model.model_revision.0.clone(),
        Arc::new(basis),
        filters,
        &[PointNm { x: 0, y: 0 }, PointNm { x: 10, y: 0 }],
    )
    .unwrap();
    assert!(evaluation.advance().unwrap().unwrap().is_empty());
}
