use super::*;
use eda_engine::api::native_write::schematic_wire_edit::{
    WireCornerOrder, WireRouteConfig, WireRouting,
};
fn p(x: i64, y: i64) -> PointNm {
    PointNm { x, y }
}
fn draft(bus: bool) -> SchematicAuthoring {
    SchematicAuthoring {
        wire: true,
        bus,
        sheet: Some(Uuid::from_u128(10)),
        reference: "DATA".into(),
        value: "D0,D1".into(),
        ..Default::default()
    }
}
fn request_points(state: &SchematicAuthoring) -> Vec<PointNm> {
    let points = match state.polyline_request().unwrap() {
        SchematicEditRequest::DrawWire { points, .. } => points,
        SchematicEditRequest::CreateBus {
            points: Some(points),
            ..
        } => points,
        other => panic!("unexpected request: {other:?}"),
    };
    points
        .into_iter()
        .map(|point| p(point.x, point.y))
        .collect()
}

#[test]
fn preview_and_anchoring_use_identical_geometry_for_wire_and_bus_all_postures() {
    for bus in [false, true] {
        for routing in [
            WireRouting::Free,
            WireRouting::Orthogonal,
            WireRouting::Diagonal45,
        ] {
            for corner in [WireCornerOrder::Start, WireCornerOrder::End] {
                let mut state = draft(bus);
                state.wire_route = WireRouteConfig { routing, corner };
                state.append_wire_vertex(p(10, 20)).unwrap();
                state.position = Some(p(123_457, -987_653));
                let preview = state.preview_wire_points().unwrap();
                assert_eq!(
                    *preview.last().unwrap(),
                    p(123_457, -987_653),
                    "off-grid electrical contact must survive"
                );
                state.append_wire_vertex(p(123_457, -987_653)).unwrap();
                assert_eq!(state.vertices, preview);
                assert_eq!(request_points(&state), preview);
                assert!(preview.windows(2).all(|pair| pair[0] != pair[1]));
                assert!(state.pending_request.is_none());
                assert!(state.source.is_none());
            }
        }
    }
}

#[test]
fn remove_last_vertex_pops_actual_corner_or_anchor_and_resets_repeat_and_position() {
    let mut state = draft(false);
    state.wire_route = WireRouteConfig {
        routing: WireRouting::Orthogonal,
        corner: WireCornerOrder::Start,
    };
    state.append_wire_vertex(p(0, 0)).unwrap();
    state.append_wire_vertex(p(1000, 500)).unwrap();
    assert_eq!(state.vertices, vec![p(0, 0), p(1000, 0), p(1000, 500)]);
    state.last_wire_click = Some((p(1000, 500), std::time::Instant::now()));
    state.preview_dirty = false;
    assert!(state.remove_last_wire_vertex());
    assert_eq!(state.vertices, vec![p(0, 0), p(1000, 0)]);
    assert_eq!(state.position, Some(p(1000, 0)));
    assert!(state.last_wire_click.is_none());
    assert!(state.preview_dirty);
    assert_eq!(request_points(&state), state.vertices);
    assert!(state.remove_last_wire_vertex());
    assert!(state.polyline_request().is_err());
    assert!(state.remove_last_wire_vertex());
    assert_eq!(state.position, None);
    assert!(state.vertices.is_empty());
    assert!(!state.remove_last_wire_vertex());
}

#[test]
fn posture_changes_affect_only_unanchored_preview_until_next_click() {
    let mut state = draft(false);
    state.append_wire_vertex(p(0, 0)).unwrap();
    state.position = Some(p(1000, 600));
    let anchored = state.vertices.clone();
    let free = state.preview_wire_points().unwrap();
    assert_eq!(free, vec![p(0, 0), p(1000, 600)]);
    state.wire_route = WireRouteConfig {
        routing: WireRouting::Orthogonal,
        corner: WireCornerOrder::End,
    };
    let orthogonal = state.preview_wire_points().unwrap();
    assert_eq!(orthogonal, vec![p(0, 0), p(0, 600), p(1000, 600)]);
    assert_eq!(state.vertices, anchored);
    assert!(state.pending_request.is_none());
    state.append_wire_vertex(p(1000, 600)).unwrap();
    assert_eq!(state.vertices, orthogonal);
}

#[test]
fn repeated_anchor_has_no_zero_edge_and_pending_write_refuses_draft_mutation() {
    let mut state = draft(false);
    state.append_wire_vertex(p(0, 0)).unwrap();
    state.append_wire_vertex(p(1000, 0)).unwrap();
    let before = state.vertices.clone();
    state.append_wire_vertex(p(1000, 0)).unwrap();
    assert_eq!(state.vertices, before);
    state.pending_request = Some(state.polyline_request().unwrap());
    let pending = state.pending_request.clone();
    assert!(state.append_wire_vertex(p(1000, 1000)).is_err());
    assert!(!state.remove_last_wire_vertex());
    assert_eq!(state.vertices, before);
    assert_eq!(state.pending_request, pending);
    let mut inactive = SchematicAuthoring::default();
    assert!(inactive.append_wire_vertex(p(0, 0)).is_err());
    assert!(!inactive.remove_last_wire_vertex());
}

#[test]
fn qualified_child_request_keeps_basis_and_occurrence_through_posture_and_pop() {
    let mut state = draft(false);
    let project = Uuid::from_u128(1);
    let sheet = Uuid::from_u128(10);
    let path = vec![Uuid::from_u128(20)];
    state.circuit_basis = Some(crate::AuthoringViewBasis {
        project,
        revision: "captured-source".into(),
        pane: crate::PaneId(3),
        content: crate::PaneContent::Schematic,
        source: sheet,
        document: Some(crate::schematic_documents::SchematicDocumentKey {
            project,
            schematic: Uuid::from_u128(2),
            source_sheet: sheet,
            instance_path: path.clone(),
        }),
    });
    let basis = state.circuit_basis.clone();
    state.wire_route = WireRouteConfig {
        routing: WireRouting::Diagonal45,
        corner: WireCornerOrder::End,
    };
    state.append_wire_vertex(p(7, 11)).unwrap();
    state.append_wire_vertex(p(107, 211)).unwrap();
    let SchematicEditRequest::DrawWire {
        sheet: actual,
        instance_path,
        ..
    } = state.polyline_request().unwrap()
    else {
        panic!("wire request")
    };
    assert_eq!(actual, sheet);
    assert_eq!(instance_path, path);
    assert_eq!(state.circuit_basis, basis);
    assert!(state.remove_last_wire_vertex());
    assert_eq!(state.circuit_basis, basis);
}
