use super::*;
use datum_gui_protocol::{PointNm, ZoneFillProjection};
#[test]
fn occupied_zone_paint_never_substitutes_authored_envelope_for_current_copper() {
    let mut state = datum_gui_protocol::load_fixture_workspace_state();
    let mut zone = state.scene.zones[0].clone();
    zone.layer_id = "zone-proof-copper".into();
    state.scene.layers = vec![datum_gui_protocol::SceneLayer {
        layer_id: zone.layer_id.clone(),
        name: "F.Cu".into(),
        kind: "copper".into(),
        render_order: 1,
        visible_by_default: true,
    }];
    state.ui.filters.show_authored = true;
    state.ui.filters.layer_visibility.clear();
    let triangle = vec![
        PointNm { x: 0, y: 0 },
        PointNm { x: 10, y: 0 },
        PointNm { x: 0, y: 10 },
    ];
    zone.polygon = vec![
        PointNm { x: -100, y: -100 },
        PointNm { x: 100, y: -100 },
        PointNm { x: 100, y: 100 },
        PointNm { x: -100, y: 100 },
    ];
    state.scene.zones = vec![zone];
    let projection = Projection::new(
        RectPx {
            x: 0.0,
            y: 0.0,
            width: 1280.0,
            height: 800.0,
        },
        &state.scene.bounds,
        CameraState::fit_to_bounds(&state.scene.bounds),
    );
    let paint = |fill| {
        let mut state = state.clone();
        state.scene.zones[0].fill_projection = fill;
        let mut quads = Vec::new();
        let mut strokes = Vec::new();
        let mut commands = Vec::new();
        for pass in [0, 1, 2] {
            push(
                &mut quads,
                &mut strokes,
                &mut commands,
                &state.scene,
                &projection,
                &state,
                pass,
                &[],
            );
        }
        (quads, strokes)
    };
    let (current, boundary) = paint(ZoneFillProjection::Current(vec![triangle]));
    assert!(
        !current.is_empty(),
        "certified triangles render occupied copper"
    );
    assert!(!boundary.is_empty(), "authored boundary remains visible");
    for fill in [
        ZoneFillProjection::Current(vec![]),
        ZoneFillProjection::Unavailable("Stale".into()),
    ] {
        let (quads, strokes) = paint(fill);
        assert!(
            quads.is_empty(),
            "empty/unavailable is never an envelope fill"
        );
        assert_eq!(strokes, boundary);
    }
    assert!(
        !paint(ZoneFillProjection::LegacyNominal).0.is_empty(),
        "recorded compatibility scenes retain nominal paint"
    );
}

#[test]
fn native_zone_boundary_selects_without_inventing_occupied_copper() {
    let mut state = datum_gui_protocol::load_fixture_workspace_state();
    let mut zone = state.scene.zones[0].clone();
    zone.layer_id = "zone-proof-copper".into();
    zone.polygon = vec![
        PointNm { x: 0, y: 0 },
        PointNm {
            x: 10_000_000,
            y: 0,
        },
        PointNm {
            x: 10_000_000,
            y: 10_000_000,
        },
        PointNm {
            x: 0,
            y: 10_000_000,
        },
    ];
    state.scene.components.clear();
    state.scene.pads.clear();
    state.scene.tracks.clear();
    state.scene.vias.clear();
    state.scene.component_graphics.clear();
    state.scene.component_texts.clear();
    state.scene.board_texts.clear();
    state.scene.board_graphics.clear();
    state.scene.outline.clear();
    state.scene.layers = vec![datum_gui_protocol::SceneLayer {
        layer_id: zone.layer_id.clone(),
        name: "F.Cu".into(),
        kind: "copper".into(),
        render_order: 1,
        visible_by_default: true,
    }];
    state.ui.filters.show_authored = true;
    state.ui.filters.layer_visibility.clear();
    state.scene.zones = vec![zone];
    let hits = |state: &datum_gui_protocol::ReviewWorkspaceState| {
        crate::board_hit_construction::build(&state.scene, state, |_| Ok(())).unwrap()
    };
    let middle = PointNm {
        x: 5_000_000,
        y: 5_000_000,
    };
    for fill in [
        ZoneFillProjection::Unavailable("Unfilled".into()),
        ZoneFillProjection::Unavailable("Stale".into()),
        ZoneFillProjection::Unavailable("Unsupported".into()),
        ZoneFillProjection::Current(vec![]),
    ] {
        state.scene.zones[0].fill_projection = fill;
        let regions = hits(&state);
        let boundary = regions
            .iter()
            .find(|r| r.target == state.scene.zones[0].object_id)
            .unwrap();
        assert_eq!(boundary.layer_id.as_deref(), Some("zone-proof-copper"));
        assert!(
            boundary.shape.contains(PointNm { x: 0, y: 5_000_000 }),
            "closing edge selects"
        );
        assert!(boundary.shape.contains(PointNm { x: 5_000_000, y: 0 }));
        assert!(
            !regions.iter().any(|r| r.shape.contains(middle)),
            "boundary is not copper occupancy"
        );
        state
            .ui
            .filters
            .layer_visibility
            .insert("zone-proof-copper".into(), false);
        assert!(!hits(&state).iter().any(|r| r.target == boundary.target));
        state.ui.filters.layer_visibility.clear();
        state.ui.filters.show_authored = false;
        assert!(hits(&state).is_empty());
        state.ui.filters.show_authored = true;
    }
    state.scene.zones[0].fill_projection =
        ZoneFillProjection::Current(vec![state.scene.zones[0].polygon.clone()]);
    assert!(
        hits(&state)
            .iter()
            .any(|r| matches!(r.shape, WorldHitShape::Polygon(_)) && r.shape.contains(middle))
    );
}

#[test]
fn closed_boundary_hit_admission_accounts_for_closing_point_before_ownership() {
    use crate::hit_construction::{Region, Shape, build};
    let path = [
        PointNm { x: 0, y: 0 },
        PointNm { x: 10, y: 0 },
        PointNm { x: 0, y: 10 },
    ];
    let mut sizes = Vec::new();
    for source in [&path[..], &path[..0]] {
        let regions = build(
            |emit| {
                emit(Region {
                    target: "zone",
                    layer_id: None,
                    shape: Shape::ClosedPolyline {
                        path: source,
                        half_width_nm: 1.0,
                    },
                })
            },
            |bytes| {
                sizes.push(bytes);
                Ok(())
            },
        )
        .unwrap();
        let WorldHitShape::Polyline { path: owned, .. } = &regions[0].shape else {
            panic!("shared polyline required")
        };
        assert_eq!(owned.len(), if source.is_empty() { 0 } else { 4 });
        assert_eq!(owned.first(), owned.last());
    }
    assert!(sizes[0] >= sizes[1] + 4 * std::mem::size_of::<PointNm>());
    let mut visits = 0;
    assert!(
        build(
            |emit| {
                visits += 1;
                emit(Region {
                    target: "zone",
                    layer_id: None,
                    shape: Shape::ClosedPolyline {
                        path: &path,
                        half_width_nm: 1.0,
                    },
                })
            },
            |_| anyhow::bail!("refuse before ownership")
        )
        .is_err()
    );
    assert_eq!(visits, 1);
    let closed = [path[0], path[1], path[2], path[0]];
    let regions = build(
        |emit| {
            emit(Region {
                target: "zone",
                layer_id: None,
                shape: Shape::ClosedPolyline {
                    path: &closed,
                    half_width_nm: 1.0,
                },
            })
        },
        |_| Ok(()),
    )
    .unwrap();
    let WorldHitShape::Polyline { path: owned, .. } = &regions[0].shape else {
        panic!()
    };
    assert_eq!(owned, &closed);
}
