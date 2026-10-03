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
