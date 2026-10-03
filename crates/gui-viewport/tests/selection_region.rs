use datum_gui_protocol::{PointNm, selection_geometry::Rational as R};
use datum_gui_viewport::selection_region::{SelectionRegion, point};
fn p(x: i64, y: i64) -> PointNm {
    PointNm { x, y }
}
fn rect(x: i64, y: i64, w: i64, h: i64) -> Vec<PointNm> {
    vec![p(x, y), p(x + w, y), p(x + w, y + h), p(x, y + h)]
}
#[test]
fn strict_anchor_majority_cannot_be_inflated_by_width_or_parent_artwork() {
    let region = SelectionRegion::new(&rect(0, 0, 10, 10)).unwrap();
    let anchors = vec![
        p(1, 1),
        p(2, 2),
        p(3, 3),
        p(4, 4),
        p(11, 1),
        p(11, 2),
        p(11, 3),
        p(11, 4),
    ];
    assert!(!region.anchors(&anchors).unwrap());
    let mut five = anchors;
    five[4] = p(5, 5);
    assert!(region.anchors(&five).unwrap());
    assert!(!region.anchors(&[p(5, 5), p(11, 5)]).unwrap());
    assert!(region.anchors(&[p(5, 5), p(11, 5), p(5, 6)]).unwrap());
}
#[test]
fn text_exact_half_is_not_selected_and_rational_rotated_layout_uses_area() {
    let region = SelectionRegion::new(&rect(0, 0, 10, 10)).unwrap();
    let half: Vec<_> = rect(0, 0, 20, 10).into_iter().map(point).collect();
    assert!(!region.majority_layout(&half).unwrap());
    let more: Vec<_> = rect(0, 0, 19, 10).into_iter().map(point).collect();
    assert!(region.majority_layout(&more).unwrap());
    let diamond = vec![
        point(p(5, -5)),
        point(p(15, 5)),
        point(p(5, 15)),
        point(p(-5, 5)),
    ];
    assert!(!region.majority_layout(&diamond).unwrap());
    let shifted: Vec<_> = diamond
        .into_iter()
        .map(|[x, y]| {
            [
                x.checked_div(R::integer(2))
                    .unwrap()
                    .checked_add(R::new(5, 2).unwrap())
                    .unwrap(),
                y.checked_div(R::integer(2))
                    .unwrap()
                    .checked_add(R::new(5, 2).unwrap())
                    .unwrap(),
            ]
        })
        .collect();
    assert!(region.majority_layout(&shifted).unwrap());
}
#[test]
fn filled_polygon_and_disk_require_complete_enclosure_with_tangency_allowed() {
    let region = SelectionRegion::new(&rect(0, 0, 10, 10)).unwrap();
    assert!(
        region
            .encloses(
                &rect(0, 0, 10, 10)
                    .into_iter()
                    .map(point)
                    .collect::<Vec<_>>()
            )
            .unwrap()
    );
    assert!(
        !region
            .encloses(
                &rect(0, 0, 11, 10)
                    .into_iter()
                    .map(point)
                    .collect::<Vec<_>>()
            )
            .unwrap()
    );
    assert!(region.encloses_disk(p(5, 5), 5).unwrap());
    assert!(!region.encloses_disk(p(5, 5), 6).unwrap());
}
#[test]
fn concave_and_self_crossing_lasso_use_even_odd_area() {
    let region =
        SelectionRegion::new(&[p(0, 0), p(10, 0), p(10, 4), p(4, 4), p(4, 10), p(0, 10)]).unwrap();
    assert!(region.contains(p(2, 8)).unwrap());
    assert!(!region.contains(p(8, 8)).unwrap());
    assert!(!region.encloses_disk(p(3, 3), 2).unwrap());
    let bow = SelectionRegion::new(&[p(0, 0), p(10, 10), p(0, 10), p(10, 0)]).unwrap();
    assert!(bow.contains(p(5, 2)).unwrap());
    assert!(!bow.contains(p(1, 5)).unwrap());
    let square: Vec<_> = rect(0, 0, 10, 10).into_iter().map(point).collect();
    assert_eq!(bow.covered_area(&square).unwrap(), R::integer(50));
    assert!(!bow.majority_layout(&square).unwrap());
    let zero = SelectionRegion::new(&[p(0, 0), p(5, 0), p(10, 0)]).unwrap();
    assert!(!zero.contains(p(5, 0)).unwrap());
}

#[test]
fn oriented_hit_excludes_bbox_corners_and_preserves_exact_fractional_boundary() {
    use datum_gui_viewport::hit::HitShape;
    let diamond = vec![
        point(p(0, 5)),
        point(p(5, 0)),
        point(p(10, 5)),
        point(p(5, 10)),
    ];
    let shape = HitShape::Layout(diamond);
    assert!(shape.contains(p(5, 5)));
    assert!(shape.contains(p(0, 5)));
    assert!(!shape.contains(p(0, 0)));
    assert_eq!(
        shape.bounds().unwrap(),
        datum_gui_protocol::RectNm {
            min_x: 0,
            min_y: 0,
            max_x: 10,
            max_y: 10
        }
    );
}
