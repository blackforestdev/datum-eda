//! Borrowed closed native Symbol line contours; open/ambiguous strokes stay strokes.
use datum_gui_protocol::{BoardGraphicPrimitive as Graphic, PointNm, SchematicHitKind};

#[derive(Clone, Copy)]
pub(crate) struct Outline<'a> {
    graphics: &'a [Graphic],
    seed: usize,
    vertices: usize,
}

fn identity(graphic: &Graphic) -> Option<(&str, &str)> {
    if graphic.schematic_hit_kind()? != SchematicHitKind::Symbol
        || graphic.source_object_uuid.is_empty()
    {
        return None;
    }
    let (base, index) = graphic.object_id.rsplit_once(":library:")?;
    index.parse::<usize>().ok()?;
    Some((base, &graphic.source_object_uuid))
}

fn same_identity(graphic: &Graphic, key: (&str, &str)) -> bool {
    // Most scene graphics belong to other symbols; avoid parsing their keys.
    graphic.source_object_uuid == key.1 && identity(graphic) == Some(key)
}

fn edge(graphic: &Graphic) -> Option<[PointNm; 2]> {
    (graphic.primitive_kind == "line" && graphic.path.len() == 2)
        .then(|| [graphic.path[0], graphic.path[1]])
}

impl<'a> Outline<'a> {
    /// Each accepted component is emitted at its last painted edge, exactly once.
    /// Traversal borrows scene geometry throughout both constructor passes.
    pub(crate) fn ending_at(graphics: &'a [Graphic], seed: usize) -> Option<Self> {
        let key = identity(&graphics[seed])?;
        let [start, mut point] = edge(&graphics[seed])?;
        if start == point {
            return None;
        }
        let mut previous = seed;
        let mut vertices = 1usize;
        let mut area = cross(start, point)?;
        while point != start {
            let mut next = None;
            let mut degree = 0;
            for (index, graphic) in graphics.iter().enumerate() {
                if !same_identity(graphic, key) {
                    continue;
                }
                let Some([a, b]) = edge(graphic) else {
                    continue;
                };
                if a == point || b == point {
                    if a == b {
                        return None;
                    }
                    degree += 1;
                    if index != previous {
                        next = Some((index, if a == point { b } else { a }));
                    }
                }
            }
            let (index, following) = next?;
            if degree != 2 || index > seed || vertices >= graphics.len() {
                return None;
            }
            area = area.checked_add(cross(point, following)?)?;
            vertices += 1;
            previous = index;
            point = following;
        }
        // Closure must also have exactly two incident edges (including the seed).
        let degree = graphics
            .iter()
            .filter(|g| same_identity(g, key))
            .filter_map(edge)
            .filter(|[a, b]| *a == start || *b == start)
            .count();
        if degree != 2 || vertices < 3 || area == 0 {
            return None;
        }
        let outline = Self {
            graphics,
            seed,
            vertices,
        };
        // Reject self intersections, overlapping edges and T contacts, including
        // other source lines meeting this component away from an exact endpoint.
        for (a, b, index) in outline.edges() {
            for (other, graphic) in graphics.iter().enumerate() {
                if other == index || !same_identity(graphic, key) {
                    continue;
                }
                let Some([c, d]) = edge(graphic) else {
                    continue;
                };
                if ambiguous_intersection(a, b, c, d)? {
                    return None;
                }
            }
        }
        Some(outline)
    }

    pub(crate) fn len(self) -> usize {
        self.vertices
    }

    pub(crate) fn into_points(self) -> anyhow::Result<Vec<PointNm>> {
        let mut points = Vec::new();
        points.try_reserve_exact(self.vertices)?;
        points.extend(self.edges().map(|(a, _, _)| a));
        anyhow::ensure!(
            points.len() == self.vertices,
            "Symbol outline changed after admission"
        );
        Ok(points)
    }

    fn edges(self) -> impl Iterator<Item = (PointNm, PointNm, usize)> + 'a {
        let key = identity(&self.graphics[self.seed]).expect("validated Symbol identity");
        let [mut from, mut to] = edge(&self.graphics[self.seed]).expect("validated line");
        let mut current = self.seed;
        let mut remaining = self.vertices;
        std::iter::from_fn(move || {
            if remaining == 0 {
                return None;
            }
            let result = (from, to, current);
            remaining -= 1;
            if remaining != 0 {
                let (next, [a, b]) = self
                    .graphics
                    .iter()
                    .enumerate()
                    .filter(|(index, g)| *index != current && same_identity(g, key))
                    .filter_map(|(index, g)| edge(g).map(|e| (index, e)))
                    .find(|(_, [a, b])| *a == to || *b == to)
                    .expect("validated closed degree-two contour");
                from = to;
                to = if a == from { b } else { a };
                current = next;
            }
            Some(result)
        })
    }
}

fn cross(a: PointNm, b: PointNm) -> Option<i128> {
    (a.x as i128 * b.y as i128).checked_sub(a.y as i128 * b.x as i128)
}

fn orientation(a: PointNm, b: PointNm, c: PointNm) -> Option<i128> {
    let (x, y) = (b.x as i128 - a.x as i128, b.y as i128 - a.y as i128);
    let (u, v) = (c.x as i128 - a.x as i128, c.y as i128 - a.y as i128);
    x.checked_mul(v)?.checked_sub(y.checked_mul(u)?)
}

fn on_segment(a: PointNm, b: PointNm, p: PointNm) -> bool {
    p.x >= a.x.min(b.x) && p.x <= a.x.max(b.x) && p.y >= a.y.min(b.y) && p.y <= a.y.max(b.y)
}

fn ambiguous_intersection(a: PointNm, b: PointNm, c: PointNm, d: PointNm) -> Option<bool> {
    let (abc, abd, cda, cdb) = (
        orientation(a, b, c)?,
        orientation(a, b, d)?,
        orientation(c, d, a)?,
        orientation(c, d, b)?,
    );
    let shared_endpoint = a == c || a == d || b == c || b == d;
    if shared_endpoint {
        // Exact adjoining endpoints are allowed; coincident or backtracking
        // collinear edges overlap rather than enclose an unambiguous body.
        return Some(
            abc == 0
                && abd == 0
                && ((c != a && c != b && on_segment(a, b, c))
                    || (d != a && d != b && on_segment(a, b, d))
                    || (a != c && a != d && on_segment(c, d, a))
                    || (b != c && b != d && on_segment(c, d, b))
                    || (a == c && b == d)
                    || (a == d && b == c)),
        );
    }
    Some(
        (abc == 0 && on_segment(a, b, c))
            || (abd == 0 && on_segment(a, b, d))
            || (cda == 0 && on_segment(c, d, a))
            || (cdb == 0 && on_segment(c, d, b))
            || (abc.signum() != abd.signum() && cda.signum() != cdb.signum()),
    )
}
