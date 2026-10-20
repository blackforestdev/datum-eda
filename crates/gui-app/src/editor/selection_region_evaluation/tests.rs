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

// Reproduce inputs with scripts/provision_s5a_replay_inputs.py --output
// <fresh .datum/scratch/<owner>/s5a-replay-inputs>, then set DATUM_REPLAY_INPUTS.
#[path = "oracle_tests.rs"]
mod a3_oracle;
use a3_oracle::{A3Box, A3Geometry, a3_catalog, a3_expected};

const A3_INDEX: &[u8] =
    include_bytes!("../../../../test-harness/testdata/selection/s5a_v1/replay-inputs/index.json");
fn a3_hash(bytes: &[u8]) -> String {
    eda_engine::import::ids_sidecar::compute_source_hash_bytes(bytes)
        .trim_start_matches("sha256:")
        .to_owned()
}
fn a3_inputs(prefix: &str) -> (std::path::PathBuf, Vec<(std::path::PathBuf, String)>) {
    let root = std::path::PathBuf::from(
        std::env::var_os("DATUM_REPLAY_INPUTS")
            .expect("provision tracked replay inputs and set DATUM_REPLAY_INPUTS"),
    );
    assert!(root.is_absolute());
    assert_eq!(
        root.canonicalize().unwrap(),
        root,
        "replay symlink alias refused"
    );
    assert_eq!(std::fs::read(root.join("index.json")).unwrap(), A3_INDEX);
    let index: serde_json::Value = serde_json::from_slice(A3_INDEX).unwrap();
    let mut pins = Vec::new();
    for row in index["files"].as_array().unwrap().iter().filter(|r| {
        r["path"]
            .as_str()
            .unwrap()
            .starts_with(&format!("{prefix}/"))
    }) {
        let path = root.join(row["path"].as_str().unwrap());
        assert!(path.ancestors().all(|p| !p.is_symlink()));
        let bytes = std::fs::read(&path).unwrap();
        assert_eq!(bytes.len() as u64, row["bytes"].as_u64().unwrap());
        let expected = row["sha256"].as_str().unwrap().to_owned();
        assert_eq!(a3_hash(&bytes), expected, "frozen input {}", path.display());
        pins.push((path, expected));
    }
    assert!(!pins.is_empty());
    eprintln!(
        "A3 input={prefix} pins={} index_sha256={}",
        pins.len(),
        a3_hash(A3_INDEX)
    );
    (root.join(prefix), pins)
}
fn a3_json(path: impl AsRef<std::path::Path>) -> serde_json::Value {
    serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap()
}
fn a3_complete(
    basis: Arc<RegionBasis>,
    revision: &str,
    path: &[PointNm],
    budget: usize,
) -> BTreeSet<Id> {
    let mut filters = datum_gui_protocol::load_fixture_workspace_state()
        .ui
        .filters;
    filters.layer_visibility.clear();
    filters.excluded_selection_classes.clear();
    filters.show_authored = true;
    let mut query = RegionEvaluation::begin(
        PaneId(0),
        AcquisitionMode::Add,
        revision.into(),
        basis,
        filters,
        path,
    )
    .unwrap();
    loop {
        if let Some(ids) = query.advance_with_budget(budget).unwrap() {
            return ids;
        }
    }
}
fn a3_compare(
    basis: Arc<RegionBasis>,
    revision: &str,
    oracle: &BTreeMap<Id, A3Geometry>,
    path: &[PointNm],
    parts: &[A3Box],
    name: &str,
) -> BTreeSet<Id> {
    let expected: BTreeSet<_> = oracle
        .iter()
        .filter(|(_, g)| a3_expected(g, parts))
        .map(|(id, _)| id.clone())
        .collect();
    for budget in [7, DEFAULT_REGION_QUERY_BUDGET] {
        let actual = a3_complete(basis.clone(), revision, path, budget);
        assert!(
            actual == expected,
            "{name} budget={budget}: missing={:?}, extra={:?}",
            expected.difference(&actual).take(8).collect::<Vec<_>>(),
            actual.difference(&expected).take(8).collect::<Vec<_>>()
        );
        let mut per_class = BTreeMap::new();
        for id in &actual {
            *per_class.entry(id.class).or_insert(0usize) += 1;
        }
        eprintln!(
            "A3 {name} budget={budget} total={} classes={per_class:?} complete_ids_sha256={}",
            actual.len(),
            a3_hash(&serde_json::to_vec(&actual).unwrap())
        );
    }
    expected
}
fn a3_run(prefix: &str, variant: Option<&str>, dense: bool) {
    let (base, pins) = a3_inputs(prefix);
    let class = variant.map(|_| a3_json(base.join("s5a-fixture.json"))["class_variant"].clone());
    let root = variant.map_or_else(|| base.clone(), |name| base.join(name));
    let model = Arc::new(
        eda_engine::substrate::ProjectResolver::new(&root)
            .resolve()
            .unwrap(),
    );
    let request = datum_gui_protocol::LiveReviewRequest {
        project_root: root.clone(),
        board_file: None,
        artifact_path: None,
        net_uuid: None,
        from_anchor_pad_uuid: None,
        to_anchor_pad_uuid: None,
        profile: None,
        kicad_board_source: None,
    };
    let state = datum_gui_protocol::workspace_loading::project_board_editor_workspace(
        &request,
        model.clone(),
    )
    .unwrap();
    let authority = NativeSelectionResolution::capture_shared(model.clone()).unwrap();
    let board = a3_json(root.join("board/board.json"));
    let sheet = a3_json(root.join("schematic/selection.json"));
    assert_eq!(
        sheet["texts"].as_object().unwrap().len(),
        if dense { 100_000 } else { 1 }
    );
    for (surface, scene) in [
        ("board", &state.scene),
        ("schematic", state.schematic_scene.as_ref().unwrap()),
    ] {
        let oracle = a3_catalog(scene, &board, &sheet, class.as_ref());
        let basis = Arc::new(RegionBasis::capture(&model, &authority, scene));
        assert!(
            basis.unresolved.is_empty(),
            "governed input must have complete qualification geometry: {:?}",
            basis.unresolved
        );
        assert_eq!(
            basis
                .index
                .regions()
                .iter()
                .map(|s| s.target.id.clone())
                .collect::<BTreeSet<_>>(),
            oracle.keys().cloned().collect()
        );
        let (x0, y0, x1, y1, cut_x, cut_y) = if dense && surface == "schematic" {
            (
                -1_000_000,
                -1_000_000,
                401_000_000,
                201_000_000,
                201_000_000,
                101_000_000,
            )
        } else if dense {
            (
                -1_000_000,
                -11_000_000,
                51_000_000,
                11_000_000,
                15_000_000,
                -500_000,
            )
        } else if surface == "schematic" {
            (
                -10_000_000,
                -20_000_000,
                150_000_000,
                20_000_000,
                22_500_000,
                12_500_000,
            )
        } else {
            (
                -10_000_000,
                -20_000_000,
                150_000_000,
                20_000_000,
                100_000_000,
                0,
            )
        };
        let p = |x, y| PointNm { x, y };
        let rect = [p(x0, y0), p(x1, y0), p(x1, y1), p(x0, y1)];
        let lasso = [
            p(x0, y0),
            p(x1, y0),
            p(x1, cut_y),
            p(cut_x, cut_y),
            p(cut_x, y1),
            p(x0, y1),
        ];
        let whole = A3Box::nm(x0, y0, x1, y1);
        let parts = [
            A3Box::nm(x0, y0, x1, cut_y),
            A3Box::nm(x0, cut_y, cut_x, y1),
        ];
        let r = a3_compare(
            basis.clone(),
            &scene.source_revision,
            &oracle,
            &rect,
            &[whole],
            &format!("{prefix}/{surface}/rect"),
        );
        let l = a3_compare(
            basis.clone(),
            &scene.source_revision,
            &oracle,
            &lasso,
            &parts,
            &format!("{prefix}/{surface}/concave-lasso"),
        );
        assert!(
            !l.is_empty() && l.len() < r.len(),
            "lasso must distinguish the excluded AABB quadrant"
        );
        if !dense && surface == "board" {
            let pad = Id {
                class: Class::Pad,
                id: class.as_ref().unwrap()["pad_id"]
                    .as_str()
                    .unwrap()
                    .parse()
                    .unwrap(),
                instance_path: vec![],
            };
            for (name, bounds, contains_center) in [
                (
                    "pad-silhouette-overlap-center-out",
                    [99_000_000, -1_000_000, 99_500_000, 1_000_000],
                    false,
                ),
                (
                    "pad-center-in",
                    [99_900_000, -100_000, 100_100_000, 100_000],
                    true,
                ),
            ] {
                let [a, b, c, d] = bounds;
                let actual = a3_compare(
                    basis.clone(),
                    &scene.source_revision,
                    &oracle,
                    &[p(a, b), p(c, b), p(c, d), p(a, d)],
                    &[A3Box::nm(a, b, c, d)],
                    name,
                );
                assert_eq!(actual.contains(&pad), contains_center);
            }
        }
    }
    for (path, hash) in pins {
        assert_eq!(
            a3_hash(&std::fs::read(&path).unwrap()),
            hash,
            "source/history changed: {}",
            path.display()
        );
    }
}
#[test]
#[ignore = "requires reproducibly provisioned tracked C03 replay inputs via DATUM_REPLAY_INPUTS"]
fn native_region_membership_matches_independent_dense_oracle() {
    a3_run("s5a-c03-u5-v1/dense-and-cancel", None, true);
}
#[test]
#[ignore = "requires reproducibly provisioned tracked C03 class inputs via DATUM_REPLAY_INPUTS"]
fn native_region_membership_matches_independent_class_oracle() {
    a3_run(
        "s5a-c03-u5-v2/class-qualification",
        Some("variants/class-geometry"),
        false,
    );
}
