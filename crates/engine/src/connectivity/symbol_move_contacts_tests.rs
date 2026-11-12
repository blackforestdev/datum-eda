//! Independent captures compare geometry, while source-to-destination order stays exact.
use super::*;

fn sheet() -> Sheet {
    serde_json::from_value(serde_json::json!({
        "uuid":Uuid::new_v4(),"name":"Contact order","frame":null,
        "symbols":{},"wires":{},"junctions":{},"labels":{},"buses":{},
        "bus_entries":{},"ports":{},"noconnects":{},"texts":{},"drawings":{}
    }))
    .unwrap()
}

#[test]
fn foreign_point_order_is_irrelevant_but_geometry_and_source_order_are_retained() {
    let sheet = sheet();
    let source = vec![Point::new(5, 0), Point::new(0, 0)];
    let foreign = vec![Point::new(30, 40), Point::new(30, 40), Point::new(10, 20)];
    let capture = |points| {
        SymbolContactBasis::capture_points(
            &sheet,
            source.clone(),
            points,
            ContactOrigin::SymbolPins,
        )
        .unwrap()
    };
    let original = capture(foreign.clone());
    let mut reversed = foreign;
    reversed.reverse();
    assert_eq!(original, capture(reversed));
    assert_eq!(original.source_points, source);
    assert_eq!(original.points.len(), 3, "Duplicate geometry is retained");
    assert_ne!(
        original,
        capture(vec![
            Point::new(10, 21),
            Point::new(30, 40),
            Point::new(30, 40)
        ])
    );
    assert!(
        original
            .require_no_foreign_contact(&[Point::new(10, 20)])
            .is_err()
    );
}
