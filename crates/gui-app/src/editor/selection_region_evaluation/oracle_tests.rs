//! Independent canonical-source geometry oracle for A3 test inputs.
//! Uses integer area/intersection and analytic circles, never production region predicates.
use super::{BTreeMap, BoardReviewSceneV1, Class, Id};
const A3_D: i128 = 1_000_000;
#[derive(Clone, Copy, Debug)]
pub(super) struct A3Box([i128; 4]);
impl A3Box {
    pub(super) fn nm(x0: i64, y0: i64, x1: i64, y1: i64) -> Self {
        Self([x0.into(), y0.into(), x1.into(), y1.into()].map(|n: i128| n * A3_D))
    }
    fn area(self) -> i128 {
        (self.0[2] - self.0[0]) * (self.0[3] - self.0[1])
    }
    fn overlap(self, other: Self) -> i128 {
        (self.0[2].min(other.0[2]) - self.0[0].max(other.0[0])).max(0)
            * (self.0[3].min(other.0[3]) - self.0[1].max(other.0[1])).max(0)
    }
    fn contains(self, p: [i128; 2]) -> bool {
        self.0[0] <= p[0] && p[0] <= self.0[2] && self.0[1] <= p[1] && p[1] <= self.0[3]
    }
}
pub(super) enum A3Geometry {
    Anchors(Vec<[i128; 2]>),
    Layout(A3Box),
    Filled(Vec<A3Box>),
    Disk([i128; 2], i128),
}
pub(super) fn a3_expected(geometry: &A3Geometry, disjoint: &[A3Box]) -> bool {
    let area = |bounds: A3Box| disjoint.iter().map(|r| bounds.overlap(*r)).sum::<i128>();
    match geometry {
        A3Geometry::Anchors(points) => {
            !points.is_empty()
                && points
                    .iter()
                    .filter(|p| disjoint.iter().any(|r| r.contains(**p)))
                    .count()
                    * 2
                    > points.len()
        }
        A3Geometry::Layout(bounds) => area(*bounds) * 2 > bounds.area(),
        A3Geometry::Filled(islands) => {
            !islands.is_empty() && islands.iter().all(|b| area(*b) == b.area())
        }
        A3Geometry::Disk(center, radius) => {
            assert!(*radius >= 0);
            if *radius == 0 {
                return disjoint.iter().any(|r| r.contains(*center));
            }
            let outer = A3Box([
                disjoint.iter().map(|r| r.0[0]).min().unwrap(),
                disjoint.iter().map(|r| r.0[1]).min().unwrap(),
                disjoint.iter().map(|r| r.0[2]).max().unwrap(),
                disjoint.iter().map(|r| r.0[3]).max().unwrap(),
            ]);
            if !outer.contains([center[0] - radius, center[1] - radius])
                || !outer.contains([center[0] + radius, center[1] + radius])
            {
                return false;
            }
            if disjoint.len() == 1 {
                return true;
            }
            assert_eq!(disjoint.len(), 2);
            // The L-lasso excludes the open upper-right quadrant. A circle
            // fits iff its distance to that quadrant is at least its radius.
            let dx = (disjoint[1].0[2] - center[0]).max(0);
            let dy = (disjoint[0].0[3] - center[1]).max(0);
            dx * dx + dy * dy >= radius * radius
        }
    }
}
fn a3_point(v: &serde_json::Value) -> [i128; 2] {
    [
        v["x"].as_i64().unwrap() as i128 * A3_D,
        v["y"].as_i64().unwrap() as i128 * A3_D,
    ]
}
fn a3_id(class: Class, id: &str) -> Id {
    Id {
        class,
        id: id.parse().unwrap(),
        instance_path: vec![],
    }
}
fn a3_layout(g: &datum_gui_protocol::BoardTextGeometryPrimitive) -> A3Box {
    let t = g.world_transform_nm.unwrap();
    let b = g.block_bbox_em_nm.unwrap();
    assert_eq!(
        (t.m12_ppm, t.m21_ppm),
        (0, 0),
        "governed axis-aligned text input"
    );
    // The captured affine/em basis is input authority. The oracle performs
    // independent integer rectangle intersection, not production triangulation.
    let x = [b.min_x_em_nm, b.max_x_em_nm]
        .map(|n| t.tx_nm as i128 * A3_D + t.m11_ppm as i128 * n as i128);
    let y = [b.min_y_em_nm, b.max_y_em_nm]
        .map(|n| t.ty_nm as i128 * A3_D + t.m22_ppm as i128 * n as i128);
    let result = A3Box([
        x[0].min(x[1]),
        y[0].min(y[1]),
        x[0].max(x[1]),
        y[0].max(y[1]),
    ]);
    assert!(result.area() > 0);
    result
}
pub(super) fn a3_catalog(
    scene: &BoardReviewSceneV1,
    board: &serde_json::Value,
    sheet: &serde_json::Value,
    class: Option<&serde_json::Value>,
) -> BTreeMap<Id, A3Geometry> {
    let mut result = BTreeMap::new();
    if scene.board_uuid == board["uuid"].as_str().unwrap() {
        for (uuid, track) in board["tracks"].as_object().unwrap() {
            let mut anchors = vec![a3_point(&track["from"]), a3_point(&track["to"])];
            if track.get("midpoint").is_some_and(|v| !v.is_null()) {
                anchors.push(a3_point(&track["midpoint"]));
            }
            result.insert(a3_id(Class::Track, uuid), A3Geometry::Anchors(anchors));
        }
        for (uuid, via) in board["vias"].as_object().unwrap() {
            result.insert(
                a3_id(Class::Via, uuid),
                A3Geometry::Anchors(vec![a3_point(&via["position"])]),
            );
        }
        // OPEN-4 qualifies an independent Pad by its canonical center.
        // Historical fixture perimeter metadata does not govern membership.
        for (uuid, pad) in board["pads"].as_object().unwrap() {
            result.insert(
                a3_id(Class::Pad, uuid),
                A3Geometry::Anchors(vec![a3_point(&pad["position"])]),
            );
        }
        for (uuid, _) in board["zones"].as_object().unwrap() {
            let class = class.unwrap();
            assert_eq!(uuid, class["zone_id"].as_str().unwrap());
            let islands =
                class["fill_islands"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|p| {
                        let points: Vec<_> = p["vertices"]
                            .as_array()
                            .unwrap()
                            .iter()
                            .map(a3_point)
                            .collect();
                        let b = A3Box([
                            points.iter().map(|p| p[0]).min().unwrap(),
                            points.iter().map(|p| p[1]).min().unwrap(),
                            points.iter().map(|p| p[0]).max().unwrap(),
                            points.iter().map(|p| p[1]).max().unwrap(),
                        ]);
                        assert_eq!(points.len(), 4);
                        assert!(points.iter().all(|p| [b.0[0], b.0[2]].contains(&p[0])
                            && [b.0[1], b.0[3]].contains(&p[1])));
                        b
                    })
                    .collect();
            result.insert(a3_id(Class::Zone, uuid), A3Geometry::Filled(islands));
        }
        assert!(board["outline"]["vertices"].as_array().unwrap().is_empty());
        for name in ["packages", "texts", "graphics"] {
            assert!(
                board
                    .get(name)
                    .and_then(|v| v.as_object())
                    .is_none_or(|m| m.is_empty())
            );
        }
    } else {
        assert_eq!(scene.board_uuid, sheet["uuid"].as_str().unwrap());
        for name in [
            "symbols",
            "wires",
            "buses",
            "labels",
            "ports",
            "junctions",
            "noconnects",
        ] {
            assert!(
                sheet
                    .get(name)
                    .and_then(|v| v.as_object())
                    .is_none_or(|m| m.is_empty())
            );
        }
        let texts: BTreeMap<_, _> = scene
            .board_texts
            .iter()
            .map(|t| (t.source_object_uuid.as_deref().unwrap_or(&t.text_uuid), t))
            .collect();
        let layouts: BTreeMap<_, _> = scene
            .board_text_geometries
            .iter()
            .map(|g| (g.object_id.as_str(), g))
            .collect();
        for (uuid, drawing) in sheet["drawings"].as_object().unwrap() {
            let variants = drawing.as_object().unwrap();
            assert_eq!(variants.len(), 1);
            let (kind, data) = variants.iter().next().unwrap();
            let geometry = match kind.as_str() {
                "Line" => A3Geometry::Anchors(vec![a3_point(&data["from"]), a3_point(&data["to"])]),
                "Rect" => {
                    let lo = a3_point(&data["min"]);
                    let hi = a3_point(&data["max"]);
                    A3Geometry::Filled(vec![A3Box([lo[0], lo[1], hi[0], hi[1]])])
                }
                "Circle" => A3Geometry::Disk(
                    a3_point(&data["center"]),
                    data["radius"].as_i64().unwrap() as i128 * A3_D,
                ),
                "Arc" => {
                    let arc = &data["arc"];
                    // Frozen canonical half-circle: source authored endpoint,
                    // quarter-turn midpoint, endpoint; no production arc helper.
                    assert_eq!(
                        (arc["start_angle"].as_i64(), arc["end_angle"].as_i64()),
                        (Some(0), Some(1800))
                    );
                    let c = a3_point(&arc["center"]);
                    let r = arc["radius"].as_i64().unwrap() as i128 * A3_D;
                    A3Geometry::Anchors(vec![[c[0] + r, c[1]], [c[0], c[1] + r], [c[0] - r, c[1]]])
                }
                other => panic!("unaccounted canonical drawing {other}"),
            };
            result.insert(a3_id(Class::SchematicGraphic, uuid), geometry);
        }
        for (uuid, source) in sheet["texts"].as_object().unwrap() {
            let t = texts[uuid.as_str()];
            assert_eq!(
                a3_point(&source["position"]),
                [t.position.x as i128 * A3_D, t.position.y as i128 * A3_D]
            );
            assert_eq!(source["rotation"].as_i64().unwrap(), 0);
            let bounds = a3_layout(layouts[t.object_id.as_str()]);
            result.insert(
                a3_id(Class::SchematicText, uuid),
                A3Geometry::Layout(bounds),
            );
        }
    }
    result
}
