use super::*;
use crate::ir::geometry::Point;
fn polygon(points: &[(i64, i64)]) -> Polygon {
    Polygon::new(points.iter().map(|(x, y)| Point::new(*x, *y)).collect())
}
fn rect(x: i64, y: i64, w: i64, h: i64) -> Polygon {
    polygon(&[(x, y), (x + w, y), (x + w, y + h), (x, y + h)])
}
#[test]
fn exact_union_equivalence_ignores_decomposition_overlap_order_and_winding() {
    let whole = [rect(0, 0, 12, 12)];
    let mut reversed = rect(0, 0, 6, 12);
    reversed.vertices.reverse();
    assert!(equivalent(&whole, &[rect(6, 0, 6, 12), reversed]).unwrap());
    assert!(equivalent(&whole, &[rect(0, 0, 8, 12), rect(4, 0, 8, 12)]).unwrap());
    let triangles = [
        polygon(&[(0, 0), (12, 0), (12, 12)]),
        polygon(&[(0, 0), (12, 12), (0, 12)]),
    ];
    assert!(equivalent(&whole, &triangles).unwrap());
    // Crossing edges split at rational coordinates; an interior redundant cell
    // cannot change the exterior boundary or exact occupied region.
    assert!(
        equivalent(
            &whole,
            &[whole[0].clone(), polygon(&[(1, 2), (10, 3), (4, 11)])]
        )
        .unwrap()
    );
    assert!(equivalent(&[], &[]).unwrap());
    assert!(!equivalent(&whole, &[]).unwrap());
}
#[test]
fn closed_union_comparison_detects_holes_one_nm_gaps_and_actual_locus_changes() {
    let whole = [rect(0, 0, 12, 12)];
    let frame = [
        rect(0, 0, 12, 4),
        rect(0, 8, 12, 4),
        rect(0, 4, 4, 4),
        rect(8, 4, 4, 4),
    ];
    assert!(!equivalent(&whole, &frame).unwrap());
    let decomposed = [
        rect(0, 0, 6, 4),
        rect(6, 0, 6, 4),
        rect(0, 8, 12, 4),
        rect(0, 4, 4, 4),
        rect(8, 4, 4, 4),
    ];
    assert!(equivalent(&frame, &decomposed).unwrap());
    assert!(!equivalent(&whole, &[rect(0, 0, 5, 12), rect(6, 0, 6, 12)]).unwrap());
    assert!(!equivalent(&whole, &[rect(1, 0, 12, 12)]).unwrap());
    assert!(
        !equivalent(
            &[polygon(&[(0, 0), (10, 0), (0, 10)])],
            &[polygon(&[(0, 0), (10, 0), (0, 11)])]
        )
        .unwrap()
    );
}
#[test]
fn invalid_or_out_of_range_cells_never_claim_equivalence() {
    let invalid = polygon(&[(0, 0), (4, 4), (0, 4), (4, 0)]);
    assert_eq!(
        equivalent(
            std::slice::from_ref(&invalid),
            std::slice::from_ref(&invalid)
        ),
        Err(GeometryError::InvalidPolygon)
    );
    let large = polygon(&[
        (i64::MIN, i64::MIN),
        (i64::MAX, i64::MIN),
        (i64::MAX, i64::MAX),
        (i64::MIN, i64::MAX),
    ]);
    assert!(equivalent(std::slice::from_ref(&large), std::slice::from_ref(&large)).is_err());
}
