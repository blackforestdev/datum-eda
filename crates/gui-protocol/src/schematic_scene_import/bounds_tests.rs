//! Native source entrypoint regression: padding never changes authored endpoints.
use super::*;
use eda_engine::ir::geometry::Point;
use eda_engine::schematic::SchematicWire;
use std::collections::HashMap;
use uuid::Uuid;
fn source(endpoints: Option<(Point, Point)>) -> Schematic {
    let id = Uuid::new_v4();
    let mut sheet = Sheet {
        uuid: id,
        name: "Bounds".into(),
        frame: None,
        symbols: HashMap::new(),
        wires: HashMap::new(),
        junctions: HashMap::new(),
        labels: HashMap::new(),
        buses: HashMap::new(),
        bus_entries: HashMap::new(),
        net_declarations: HashMap::new(),
        ports: HashMap::new(),
        noconnects: HashMap::new(),
        texts: HashMap::new(),
        drawings: HashMap::new(),
    };
    if let Some((from, to)) = endpoints {
        let uuid = Uuid::new_v4();
        sheet.wires.insert(uuid, SchematicWire { uuid, from, to });
    }
    Schematic {
        uuid: Uuid::new_v4(),
        sheets: HashMap::from([(id, sheet)]),
        sheet_definitions: HashMap::new(),
        sheet_instances: HashMap::new(),
        variants: HashMap::new(),
        waivers: Vec::new(),
    }
}
fn project(endpoints: Option<(Point, Point)>) -> BoardReviewSceneV1 {
    let schematic = source(endpoints);
    let before = schematic.clone();
    let scene = project_schematic_source(&schematic, Path::new("native-bounds.json")).unwrap();
    assert_eq!(schematic, before);
    if let Some((from, to)) = endpoints {
        let wire = scene
            .board_graphics
            .iter()
            .find(|g| g.object_kind == "schematic_wire")
            .unwrap();
        assert_eq!(
            wire.path,
            vec![
                PointNm {
                    x: from.x,
                    y: from.y
                },
                PointNm { x: to.x, y: to.y }
            ]
        );
    }
    scene
}
#[test]
fn native_bounds_projection_preserves_ordinary_and_empty_padding() {
    let scene = project(Some((Point::new(100, 200), Point::new(300, 400))));
    let p = SCHEMATIC_BOUNDS_PAD_NM;
    assert_eq!(
        (
            scene.bounds.min_x,
            scene.bounds.min_y,
            scene.bounds.max_x,
            scene.bounds.max_y
        ),
        (100 - p, 200 - p, 300 + p, 400 + p)
    );
    let empty = project(None);
    assert_eq!(
        (
            empty.bounds.min_x,
            empty.bounds.min_y,
            empty.bounds.max_x,
            empty.bounds.max_y
        ),
        (-p, -p, p, p)
    );
}
#[test]
fn native_bounds_projection_handles_positive_negative_coordinate_edges() {
    let mut failed = Vec::new();
    for (name, from, to) in [
        (
            "positive",
            Point::new(i64::MAX - 1000000, i64::MAX),
            Point::new(i64::MAX, i64::MAX),
        ),
        (
            "negative",
            Point::new(i64::MIN, i64::MIN),
            Point::new(i64::MIN + 1000000, i64::MIN),
        ),
        (
            "mixed_x_positive",
            Point::new(i64::MAX - 1000000, i64::MIN),
            Point::new(i64::MAX, i64::MIN),
        ),
        (
            "mixed_x_negative",
            Point::new(i64::MIN, i64::MAX),
            Point::new(i64::MIN + 1000000, i64::MAX),
        ),
    ] {
        let result = std::panic::catch_unwind(|| project(Some((from, to))));
        if let Ok(scene) = result {
            assert!(
                scene.bounds.min_x <= scene.bounds.max_x
                    && scene.bounds.min_y <= scene.bounds.max_y
            );
            for p in [from, to] {
                assert!((scene.bounds.min_x..=scene.bounds.max_x).contains(&p.x));
                assert!((scene.bounds.min_y..=scene.bounds.max_y).contains(&p.y));
            }
            if to.x == i64::MAX {
                assert_eq!(scene.bounds.max_x, i64::MAX);
            } else {
                assert_eq!(scene.bounds.min_x, i64::MIN);
            }
            if from.y == i64::MAX {
                assert_eq!(scene.bounds.max_y, i64::MAX);
            } else {
                assert_eq!(scene.bounds.min_y, i64::MIN);
            }
        } else {
            failed.push(name);
        }
    }
    assert!(
        failed.is_empty(),
        "native source projection overflowed for {failed:?}"
    );
}
#[test]
fn native_bounds_projection_preserves_exact_padding_at_representable_margin() {
    let p = SCHEMATIC_BOUNDS_PAD_NM;
    for (from, to) in [
        (
            Point::new(i64::MAX - p - 1000, 0),
            Point::new(i64::MAX - p, 0),
        ),
        (
            Point::new(i64::MIN + p, 0),
            Point::new(i64::MIN + p + 1000, 0),
        ),
    ] {
        let scene = project(Some((from, to)));
        assert_eq!(scene.bounds.min_x, from.x - p);
        assert_eq!(scene.bounds.max_x, to.x + p);
        assert_eq!((scene.bounds.min_y, scene.bounds.max_y), (-p, p));
    }
}
