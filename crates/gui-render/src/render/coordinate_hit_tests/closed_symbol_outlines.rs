use super::*;
use datum_gui_protocol::BoardGraphicPrimitive as Graphic;

fn point(x: i64, y: i64) -> PointNm {
    PointNm {
        x: x * 1_000_000,
        y: y * 1_000_000,
    }
}

fn line(symbol: &str, index: usize, a: PointNm, b: PointNm) -> Graphic {
    Graphic {
        object_id: format!("schematic-symbol:{symbol}:library:{index}"),
        object_kind: "schematic_symbol".into(),
        primitive_kind: "line".into(),
        source_object_uuid: symbol.into(),
        layer_id: "schematic-symbol".into(),
        path: vec![a, b],
        holes: vec![],
        width_nm: Some(100_000),
    }
}

fn triangle(symbol: &str) -> Vec<Graphic> {
    vec![
        line(symbol, 0, point(-2, -2), point(2, 0)),
        line(symbol, 1, point(-2, 2), point(2, 0)),
        line(symbol, 2, point(-2, -2), point(-2, 2)),
    ]
}

fn scene(graphics: Vec<Graphic>) -> BoardReviewSceneV1 {
    let mut scene = schematic_workspace_state().schematic_scene.unwrap();
    scene.board_graphics = graphics;
    scene.board_text_geometries.clear();
    scene
}

fn regions(graphics: Vec<Graphic>) -> Vec<WorldHitRegion> {
    schematic_hit_regions(&scene(graphics), |_| Ok(())).unwrap()
}

fn polygon_count(regions: &[WorldHitRegion]) -> usize {
    regions
        .iter()
        .filter(|r| matches!(r.shape, WorldHitShape::Polygon(_)))
        .count()
}

#[test]
fn closed_native_symbol_interior_follows_contour_instead_of_bounding_box() {
    let regions = regions(triangle("root"));
    assert_eq!(polygon_count(&regions), 1);
    let index = datum_gui_viewport::SpatialHitIndex::new(regions);
    let interior = point(-1, 0);
    assert_eq!(
        index
            .hit_test(interior, |_| true)
            .target
            .map(String::as_str),
        Some("schematic-symbol:root:library:2")
    );
    // Inside AABB, well beyond the sloped edge and its stroke tolerance.
    assert!(index.hit_test(point(1, 1), |_| true).target.is_none());
    assert!(index.hit_test(point(-3, 0), |_| true).target.is_none());
    assert!(index.hit_test(interior, |_| false).target.is_none());
    // Existing visible strokes still participate.
    assert_eq!(
        index
            .hit_test(point(-2, 0), |_| true)
            .target
            .map(String::as_str),
        Some("schematic-symbol:root:library:2")
    );
}

#[test]
fn closed_native_symbol_edges_can_be_reordered_reversed_and_interleaved() {
    let original = triangle("child-occurrence");
    for order in [[0, 1, 2], [2, 0, 1], [1, 2, 0]] {
        let mut graphics = Vec::new();
        for i in order {
            let mut graphic = original[i].clone();
            graphic.path.reverse();
            graphics.push(graphic);
            graphics.push(line(
                "unrelated",
                i,
                point(20, i as i64),
                point(22, i as i64),
            ));
        }
        let regions = regions(graphics);
        assert_eq!(polygon_count(&regions), 1);
        let body = regions
            .iter()
            .find(|r| matches!(r.shape, WorldHitShape::Polygon(_)))
            .unwrap();
        assert!(body.shape.contains(point(-1, 0)));
        assert_eq!(body.target, original[order[2]].object_id);
    }
}

#[test]
fn open_bars_and_different_qualified_source_fragments_never_form_bodies() {
    let mut graphics = triangle("first");
    graphics[1].object_id = "schematic-symbol:second:library:1".into();
    // Even equal source UUIDs must not cross qualified target boundaries.
    assert_eq!(polygon_count(&regions(graphics)), 0);
    let mut graphics = triangle("first");
    graphics[1].source_object_uuid = "other-source".into();
    assert_eq!(polygon_count(&regions(graphics)), 0);
    let mut graphics = triangle("first");
    graphics.pop();
    assert_eq!(polygon_count(&regions(graphics)), 0);
    let graphics = vec![
        line("power", 0, point(-2, 0), point(2, 0)),
        line("power", 1, point(-1, 1), point(1, 1)),
        line("power", 2, point(0, 0), point(0, -2)),
    ];
    let index = datum_gui_viewport::SpatialHitIndex::new(regions(graphics));
    assert!(index.hit_test(point(-1, -1), |_| true).target.is_none());
    assert!(index.hit_test(point(0, -1), |_| true).target.is_some());
}

#[test]
fn branching_crossing_overlapping_and_degenerate_outlines_stay_strokes() {
    let mut cases = Vec::new();
    let mut branch = triangle("s");
    branch.push(line("s", 3, point(-2, -2), point(-3, -3)));
    cases.push(branch);
    let mut crossing = triangle("s");
    crossing.push(line("s", 3, point(-3, 0), point(3, 0)));
    cases.push(crossing);
    let mut overlapping = triangle("s");
    overlapping.push(line("s", 3, point(-2, -1), point(-2, 1)));
    cases.push(overlapping);
    cases.push(vec![
        line("s", 0, point(0, 0), point(1, 0)),
        line("s", 1, point(1, 0), point(2, 0)),
        line("s", 2, point(2, 0), point(0, 0)),
    ]);
    cases.push(vec![
        line("s", 0, point(-2, -2), point(2, 2)),
        line("s", 1, point(2, 2), point(-2, 2)),
        line("s", 2, point(-2, 2), point(2, -2)),
        line("s", 3, point(2, -2), point(-2, -2)),
    ]);
    let mut zero_edge = triangle("s");
    zero_edge.push(line("s", 3, point(-2, -2), point(-2, -2)));
    cases.push(zero_edge);
    for graphics in cases {
        let count = graphics.len();
        let regions = regions(graphics);
        assert_eq!(polygon_count(&regions), 0);
        assert_eq!(regions.len(), count, "original stroke targets retained");
    }
}

#[test]
fn supplemental_body_preserves_painter_order_and_eligibility() {
    let mut graphics = triangle("under");
    graphics.extend(triangle("over"));
    let regions = regions(graphics);
    assert_eq!(polygon_count(&regions), 2);
    let index = datum_gui_viewport::SpatialHitIndex::new(regions);
    assert_eq!(
        index
            .hit_test(point(-1, 0), |_| true)
            .target
            .map(String::as_str),
        Some("schematic-symbol:over:library:2")
    );
    assert_eq!(
        index
            .hit_test(point(-1, 0), |r| r.target.contains(":under:"))
            .target
            .map(String::as_str),
        Some("schematic-symbol:under:library:2")
    );
}

#[test]
fn closed_body_is_included_in_exact_preflight_and_can_be_refused() {
    use std::cell::Cell;
    let mut graphics = triangle("s");
    let closed = scene(graphics.clone());
    graphics[2].path[1] = point(-3, 2);
    let open = scene(graphics);
    let required = Cell::new(0);
    let measure = |scene: &BoardReviewSceneV1| {
        assert!(
            schematic_hit_regions(scene, |bytes| {
                required.set(bytes);
                anyhow::bail!("test budget refusal")
            })
            .is_err()
        );
        required.get()
    };
    let open_bytes = measure(&open);
    let closed_bytes = measure(&closed);
    assert!(closed_bytes > open_bytes);
    let owned = schematic_hit_regions(&closed, |bytes| {
        assert_eq!(bytes, closed_bytes);
        Ok(())
    })
    .unwrap();
    assert_eq!(polygon_count(&owned), 1);
}
