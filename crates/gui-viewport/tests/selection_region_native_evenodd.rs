use datum_gui_protocol::{PointNm, selection_geometry::Rational as R};
use datum_gui_viewport::selection_region::SelectionRegion;

#[test]
fn native_bowtie_clips_complete_rotated_text_without_artificial_layout_diagonal() {
    // C04 n03-lasso-5: unchanged C393/f044 scene layout and actual inverse-nm
    // native vertices. The independent oracle clips two disjoint parity lobes.
    let layout = vec![
        [R::integer(30_376_000), R::new(46_743_746, 5).unwrap()],
        [
            R::new(730_530_191, 20).unwrap(),
            R::new(64_498_746, 5).unwrap(),
        ],
        [R::new(723_010_191, 20).unwrap(), R::integer(13_551_000)],
        [R::integer(30_000_000), R::integer(10_000_000)],
    ];
    let path = [
        PointNm {
            x: 38_243_288,
            y: 2_936_253,
        },
        PointNm {
            x: 28_710_357,
            y: 19_068_905,
        },
        PointNm {
            x: 38_243_288,
            y: 19_068_905,
        },
        PointNm {
            x: 28_710_357,
            y: 2_936_253,
        },
    ];
    let region = SelectionRegion::new(&path).unwrap();
    let refusal_area = R::new(
        285_746_699_671_767_653_832_478_840_895_094_958,
        431_193_675_077_698_501_682_225,
    )
    .unwrap();
    assert_eq!(region.covered_area(&layout).unwrap(), refusal_area);
    assert!(!region.majority_layout(&layout).unwrap());
    let mut reversed = layout.clone();
    reversed.reverse();
    assert_eq!(region.covered_area(&reversed).unwrap(), refusal_area);
    assert!(!region.majority_layout(&reversed).unwrap());

    // Same ordinary camera, declared >half trial; its native capture remains
    // pending. This verifies the exact source predicate, not native delivery.
    let positive = SelectionRegion::new(&[
        PointNm {
            x: 40_443_195,
            y: 5_136_160,
        },
        PointNm {
            x: 32_743_520,
            y: 17_602_300,
        },
        PointNm {
            x: 34_210_125,
            y: 17_602_300,
        },
        PointNm {
            x: 26_510_450,
            y: 5_136_160,
        },
    ])
    .unwrap();
    let positive_area = R::new(1_397_213_806_042_467_785_822_463, 292_916_527_870).unwrap();
    assert_eq!(positive.covered_area(&layout).unwrap(), positive_area);
    assert_eq!(positive.covered_area(&reversed).unwrap(), positive_area);
    assert!(positive.majority_layout(&layout).unwrap());
    assert!(positive.majority_layout(&reversed).unwrap());
}

#[test]
fn concave_subject_retains_exact_triangulation_and_enclosure() {
    use datum_gui_viewport::selection_region::point;
    let p = |x, y| PointNm { x, y };
    let layout = vec![p(0, 0), p(10, 0), p(10, 4), p(4, 4), p(4, 10), p(0, 10)]
        .into_iter()
        .map(point)
        .collect::<Vec<_>>();
    let full = SelectionRegion::new(&[p(0, 0), p(10, 0), p(10, 10), p(0, 10)]).unwrap();
    assert_eq!(full.covered_area(&layout).unwrap(), R::integer(64));
    assert!(full.encloses(&layout).unwrap());
    let partial = SelectionRegion::new(&[p(0, 0), p(5, 0), p(5, 10), p(0, 10)]).unwrap();
    assert_eq!(partial.covered_area(&layout).unwrap(), R::integer(44));
    assert!(partial.majority_layout(&layout).unwrap());
    assert!(!partial.encloses(&layout).unwrap());
    let mut reversed = layout;
    reversed.reverse();
    assert_eq!(partial.covered_area(&reversed).unwrap(), R::integer(44));
}
