use super::*;
fn source(id: u128, geometry: G, width: i64) -> BoardGraphic {
    BoardGraphic {
        uuid: uuid::Uuid::from_u128(id),
        layer: 37,
        width,
        geometry,
        import_key: None,
    }
}
fn p(x: i64, y: i64) -> Point {
    Point::new(x * 1_000_000, y * 1_000_000)
}
fn base() -> String {
    super::super::render_rs274x_mechanical_layer(37, &[], &[]).unwrap()
}
#[test]
fn exact_graphics_preserve_source_shapes_and_transparent_holes() {
    let sources = vec![
        source(
            1,
            G::Line {
                from: p(0, 0),
                to: p(2, 0),
            },
            100001,
        ),
        source(
            2,
            G::Polyline {
                points: vec![p(0, 0), p(1, 2), p(2, 0)],
            },
            100001,
        ),
        source(
            3,
            G::Arc {
                from: p(0, 0),
                midpoint: p(1, 1),
                to: p(2, 0),
            },
            100001,
        ),
        source(
            4,
            G::Rectangle {
                from: p(0, 0),
                to: p(2, 3),
                filled: true,
            },
            0,
        ),
        source(
            5,
            G::Circle {
                center: p(0, 0),
                radius: 5_000_000,
                filled: false,
            },
            100001,
        ),
        source(
            6,
            G::Polygon {
                outer: vec![p(0, 0), p(10, 0), p(10, 10), p(0, 10)],
                holes: vec![vec![p(2, 2), p(2, 4), p(4, 4), p(4, 2)]],
                filled: true,
            },
            0,
        ),
    ];
    let output = append_rs274x_board_graphics(&base(), 37, &sources).unwrap();
    assert_eq!(output.matches("G04 BoardGraphic").count(), 6);
    assert!(output.contains("C,10.100001X9.899999*%"));
    assert!(output.contains("I1000000J0D01*"));
    assert!(output.contains("4,1,4,0.000000,0.000000,10.000000"));
    assert!(output.contains("4,0,4,2.000000,2.000000"));
    assert!(
        !output.contains("%LPC"),
        "holes must not clear unrelated objects"
    );
    assert!(output.find("%AM").unwrap() < output.find("D10*").unwrap());
    assert_eq!(
        append_rs274x_board_graphics(&base(), 49, &sources).unwrap(),
        base()
    );
}
#[test]
fn rational_centre_and_nonrobust_contours_remain_explicit_obligations() {
    let source = source(
        1,
        G::Arc {
            from: p(0, 0),
            midpoint: p(1, 3),
            to: p(3, 0),
        },
        200001,
    );
    let error = append_rs274x_board_graphics(&base(), 37, &[source])
        .unwrap_err()
        .to_string();
    assert!(error.contains("CAM/T04"), "{error}");
    let source = super::tests::source(
        2,
        G::Rectangle {
            from: Point::new(0, 0),
            to: Point::new(500, 2_000_000),
            filled: true,
        },
        0,
    );
    let error = append_rs274x_board_graphics(&base(), 37, &[source])
        .unwrap_err()
        .to_string();
    assert!(error.contains("robustness"), "{error}");
}

#[test]
fn hole_to_outer_separation_is_certified_without_moving_source() {
    let graphic = source(
        3,
        G::Polygon {
            outer: vec![p(0, 0), p(10, 0), p(10, 10), p(0, 10)],
            holes: vec![vec![
                Point::new(9_000_000, 2_000_000),
                Point::new(9_999_500, 2_000_000),
                Point::new(9_999_500, 4_000_000),
                Point::new(9_000_000, 4_000_000),
            ]],
            filled: true,
        },
        0,
    );
    graphic.validate().unwrap();
    let error = append_rs274x_board_graphics(&base(), 37, &[graphic])
        .unwrap_err()
        .to_string();
    assert!(error.contains("distinct contour separation"), "{error}");
}
