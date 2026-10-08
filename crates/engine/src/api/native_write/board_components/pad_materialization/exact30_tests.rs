//! PM058 query geometry reuses PM057's registered canonical materialization.
use super::tests::{fixture, provenance};
use super::*;
use crate::api::native_write::commit_prepared;
use crate::board::{
    PadShape, Track, Via, nominal_geometry::Rational as R, quadratic3::Quadratic3 as Q,
    query_pad::QueryPadGeometry,
};
use crate::ir::geometry::{Point, Polygon};
use crate::substrate::{SourceShardKind, component_pad_source::component_pad_projection};
#[test]
fn thirty_degree_exact_contacts_and_legacy_eighth_turn_survive() {
    let (root, mut model, request, _) = fixture();
    let write = build_materialize_component_pad_sources(&model, provenance(), &request).unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    let pre_rotation = std::fs::read(root.join("board/board.json")).unwrap();
    let write = super::super::build_edit_board_package(
        &model,
        provenance(),
        request.placement.uuid,
        BoardPackageEdit::Rotation { rotation: 30 },
    )
    .unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    let board = model
        .materialized_source_shard_value(SourceShardKind::BoardRoot)
        .unwrap();
    let projection = component_pad_projection(&model, &board).unwrap();
    assert!(projection.unavailable.is_empty());
    let query = projection.query_pads.values().next().unwrap();
    let QueryPadGeometry::SourceRelative(exact) = query else {
        panic!("source relative")
    };
    let delta = exact
        .displacement([R::integer(123000), R::integer(456000)])
        .unwrap();
    assert_eq!(
        exact.center[0],
        Q::rational(R::integer(request.placement.position.x.into()))
            .add(delta[0])
            .unwrap()
    );
    let bytes = std::fs::read(root.join("board/board.json")).unwrap();
    // Translation into a zero-centred intrinsic query exercises exact boundaries
    // independently of identity/binding checks already established above.
    let mut p = exact.clone();
    p.local.position = Point::zero();
    p.center = [Q::rational(R::integer(0)); 2];
    p.local.shape = PadShape::Rect;
    p.local.width = 2_000_000;
    p.local.height = 2_000_000;
    p.local.drill = 0;
    let q = QueryPadGeometry::SourceRelative(p.clone());
    assert!(q.contains(Point::new(0, 1_100_000)).unwrap());
    assert!(!q.contains(Point::new(0, 1_400_000)).unwrap());
    let track = |a, b, w| Track::straight(Uuid::new_v4(), Uuid::nil(), a, b, w, 1);
    assert!(
        q.contact_track(&track(
            Point::new(-2_000_000, 0),
            Point::new(2_000_000, 0),
            1
        ))
        .unwrap()
    );
    assert!(
        !q.contact_track(&track(
            Point::new(-2_000_000, 3_000_000),
            Point::new(2_000_000, 3_000_000),
            1
        ))
        .unwrap()
    );
    let mut arc = track(
        Point::new(-500_000, 1_600_000),
        Point::new(500_000, 1_600_000),
        1,
    );
    arc.midpoint = Some(Point::new(0, 1_100_000));
    assert!(q.contact_track(&arc).unwrap());
    std::mem::swap(&mut arc.from, &mut arc.to);
    assert!(q.contact_track(&arc).unwrap());
    arc.midpoint = Some(Point::new(0, 2_100_000));
    assert!(!q.contact_track(&arc).unwrap());
    let mut old = p.local.clone();
    old.rotation = 45;
    old.position = Point::new(1_800_000, 0);
    assert!(
        q.contact_pad(&QueryPadGeometry::Integer(old.clone()))
            .unwrap()
    );
    old.position.x = 3_000_000;
    assert!(!q.contact_pad(&QueryPadGeometry::Integer(old)).unwrap());
    let poly = crate::board::occupied_copper::CertifiedPolygon::new(&Polygon::new(vec![
        Point::new(-10, -10),
        Point::new(10, -10),
        Point::new(10, 10),
        Point::new(-10, 10),
    ]))
    .unwrap();
    assert!(q.contact_polygon(&poly).unwrap());
    p.local.shape = PadShape::Circle;
    p.local.diameter = 2_000_000;
    p.local.width = 2_000_000;
    p.local.height = 2_000_000;
    let q = QueryPadGeometry::SourceRelative(p.clone());
    let mut other = p.clone();
    other.center = [
        Q::rational(R::integer(2_000_000)),
        Q::rational(R::integer(0)),
    ];
    assert!(
        q.contact_pad(&QueryPadGeometry::SourceRelative(other.clone()))
            .unwrap()
    );
    other.center[0] = Q::rational(R::integer(2_000_001));
    assert!(
        !q.contact_pad(&QueryPadGeometry::SourceRelative(other))
            .unwrap()
    );
    p.local.drill = 800_000;
    let q = QueryPadGeometry::SourceRelative(p.clone());
    assert!(!q.contains(Point::zero()).unwrap());
    assert!(q.contains(Point::new(400_000, 0)).unwrap());
    assert!(
        !q.contact_track(&track(Point::new(-100_000, 0), Point::new(100_000, 0), 1))
            .unwrap()
    );
    let mut drilled_arc = track(Point::new(-100_000, 0), Point::new(100_000, 0), 1);
    drilled_arc.midpoint = Some(Point::new(0, 100_000));
    assert!(!q.contact_track(&drilled_arc).unwrap());
    let via:Via=serde_json::from_value(serde_json::json!({"uuid":Uuid::new_v4(),"net":Uuid::nil(),"position":{"x":0,"y":0},"diameter":100000,"drill":0,"from_layer":1,"to_layer":2})).unwrap();
    assert!(!q.contact_via(&via).unwrap());
    assert_eq!(std::fs::read(root.join("board/board.json")).unwrap(), bytes);
    let expected_centers = projection
        .query_pads
        .iter()
        .map(|(id, pad)| (*id, pad.center()))
        .collect::<std::collections::BTreeMap<_, _>>();
    model
        .commit_journal_undo(&root, provenance().into())
        .unwrap();
    assert_eq!(
        std::fs::read(root.join("board/board.json")).unwrap(),
        pre_rotation
    );
    model
        .commit_journal_redo(&root, provenance().into())
        .unwrap();
    assert_eq!(std::fs::read(root.join("board/board.json")).unwrap(), bytes);
    let reopened = crate::substrate::ProjectResolver::new(&root)
        .resolve()
        .unwrap();
    let reopened_board = reopened
        .materialized_source_shard_value(SourceShardKind::BoardRoot)
        .unwrap();
    let replayed = component_pad_projection(&reopened, &reopened_board).unwrap();
    assert_eq!(
        replayed
            .query_pads
            .iter()
            .map(|(id, pad)| (*id, pad.center()))
            .collect::<std::collections::BTreeMap<_, _>>(),
        expected_centers
    );
    std::fs::remove_dir_all(root).unwrap();
}
