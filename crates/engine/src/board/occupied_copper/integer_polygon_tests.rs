use super::*;
use crate::board::{
    nominal_geometry::{DistanceBoundary, Rational as R},
    occupied_copper::{inside, segment_pair, segment_pair_rational},
};
fn p(x: i128, y: i128) -> P {
    [R::integer(x), R::integer(y)]
}
fn shifted(p: P) -> P {
    p.map(|v| v.checked_add(R::new(1, 3).unwrap()).unwrap())
}
#[test]
fn integer_contacts_match_fractional_translation_of_existing_rational_predicate() {
    let points = [p(-2, -2), p(-2, 2), p(2, -2), p(2, 2), p(0, 0)];
    for a in points {
        for b in points {
            for c in points {
                for d in points {
                    let reference = segment_pair_rational(
                        shifted(a),
                        shifted(b),
                        shifted(c),
                        shifted(d),
                        R::integer(0),
                        DistanceBoundary::Inclusive,
                    )
                    .unwrap();
                    let closing = segment_pair_rational(
                        shifted(b),
                        shifted(c),
                        shifted(d),
                        shifted(a),
                        R::integer(0),
                        DistanceBoundary::Inclusive,
                    )
                    .unwrap();
                    assert_eq!(simple(&[a, b, c, d]), Some(!reference && !closing));
                    assert_eq!(
                        segments_intersect(a, b, c, d),
                        Some(reference),
                        "{a:?} {b:?} {c:?} {d:?}"
                    );
                }
            }
        }
    }
}
#[test]
fn integer_winding_matches_rational_boundary_and_concavity_in_both_orientations() {
    let mut vertices = vec![p(0, 0), p(4, 0), p(4, 4), p(2, 2), p(0, 4)];
    for _ in 0..2 {
        let reference: Vec<_> = vertices.iter().copied().map(shifted).collect();
        for x in -1..=5 {
            for y in -1..=5 {
                assert_eq!(
                    contains(p(x, y), &vertices),
                    Some(inside(shifted(p(x, y)), &reference).unwrap())
                );
            }
        }
        vertices.reverse();
    }
}
#[test]
fn fractional_and_overflow_inputs_do_not_enter_integer_certificates() {
    let a = p(0, 0);
    let b = p(2, 2);
    assert_eq!(segments_intersect(shifted(a), b, a, b), None);
    assert_eq!(contains(shifted(a), &[a, b, p(2, 0)]), None);
    assert_eq!(
        segments_intersect(p(i128::MIN, 0), p(i128::MAX, 1), p(0, -1), p(0, 2)),
        None
    );
}

#[test]
fn separated_half_nm_bounds_preserve_inclusive_and_strict_widths() {
    for gap in 1..=3 {
        for width in 1..=6 {
            for boundary in [DistanceBoundary::Inclusive, DistanceBoundary::Strict] {
                let (a, b, c, d) = (p(0, 0), p(4, 0), p(0, gap), p(4, gap));
                let half = R::new(width, 2).unwrap();
                assert_eq!(
                    segment_pair(a, b, c, d, half, boundary),
                    segment_pair_rational(
                        shifted(a),
                        shifted(b),
                        shifted(c),
                        shifted(d),
                        half,
                        boundary
                    )
                );
            }
        }
    }
    let a = [R::new(1, 2).unwrap(), R::integer(0)];
    let b = [R::new(3, 2).unwrap(), R::integer(0)];
    let c = [R::integer(2), R::integer(0)];
    let d = [R::integer(3), R::integer(0)];
    assert!(
        segment_pair(
            a,
            b,
            c,
            d,
            R::new(1, 2).unwrap(),
            DistanceBoundary::Inclusive
        )
        .unwrap()
    );
    assert!(!segment_pair(a, b, c, d, R::new(1, 2).unwrap(), DistanceBoundary::Strict).unwrap());
}
