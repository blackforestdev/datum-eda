//! Query-local certified vertices. Construction retains every existing refusal.
use super::*;
#[derive(Clone)]
pub(crate) struct CertifiedPolygon {
    source: Polygon,
    vertices: Vec<P>,
    bounds: [[i128; 2]; 2],
}
impl CertifiedPolygon {
    pub(crate) fn new(source: &Polygon) -> Result<Self> {
        let vertices = polygon_vertices(source)?;
        let bounds = std::array::from_fn(|axis| {
            [
                vertices.iter().map(|p| p[axis].numerator).min().unwrap(),
                vertices.iter().map(|p| p[axis].numerator).max().unwrap(),
            ]
        });
        Ok(Self {
            vertices,
            bounds,
            source: source.clone(),
        })
    }
    pub(crate) fn bounds(&self) -> (P, P) {
        (
            self.bounds.map(|v| R::integer(v[0])),
            self.bounds.map(|v| R::integer(v[1])),
        )
    }
    pub(crate) fn source(&self) -> &Polygon {
        &self.source
    }
    pub(crate) fn contains(&self, p: Point) -> Result<bool> {
        if (0..2).any(|axis| {
            let v = i128::from(if axis == 0 { p.x } else { p.y });
            v < self.bounds[axis][0] || v > self.bounds[axis][1]
        }) {
            return Ok(false);
        }
        inside(point(p), &self.vertices)
    }
    pub(crate) fn track_within(
        &self,
        track: &Track,
        extra: i64,
        boundary: DistanceBoundary,
    ) -> Result<bool> {
        if extra < 0 {
            return Err(GeometryError::InvalidWidth);
        }
        validate_track(track)?;
        if track.midpoint.is_none() {
            let a = point(track.from);
            let b = point(track.to);
            let min = std::array::from_fn(|i| R::integer(a[i].numerator.min(b[i].numerator)));
            let max = std::array::from_fn(|i| R::integer(a[i].numerator.max(b[i].numerator)));
            if self.separated(
                min,
                max,
                track_half(track)?.checked_add(R::integer(i128::from(extra)))?,
            )? {
                return Ok(false);
            }
        }
        track_vertices(track, &self.vertices, extra, boundary)
    }
    pub(crate) fn pad_within(
        &self,
        pad: &PlacedPad,
        extra: i64,
        boundary: DistanceBoundary,
    ) -> Result<bool> {
        certify_pad(pad, |shape| self.box_within(shape, extra, boundary))
    }
    pub(crate) fn via_within(
        &self,
        via: &Via,
        extra: i64,
        boundary: DistanceBoundary,
    ) -> Result<bool> {
        self.box_within(RoundedBox::via(via)?, extra, boundary)
    }
    fn separated(&self, min: P, max: P, radius: R) -> Result<bool> {
        for i in 0..2 {
            if R::integer(self.bounds[i][1]).compare(min[i].checked_sub(radius)?)? == Ordering::Less
                || R::integer(self.bounds[i][0]).compare(max[i].checked_add(radius)?)?
                    == Ordering::Greater
            {
                return Ok(true);
            }
        }
        Ok(false)
    }
    fn box_within(
        &self,
        shape: RoundedBox,
        extra: i64,
        boundary: DistanceBoundary,
    ) -> Result<bool> {
        if extra < 0 {
            return Err(GeometryError::InvalidWidth);
        }
        let half = shape.radius.checked_add(R::integer(i128::from(extra)))?;
        if half.numerator == 0 && boundary == DistanceBoundary::Strict {
            return Err(GeometryError::UnresolvedPredicate);
        }
        if self.separated(shape.min, shape.max, half)? {
            return Ok(false);
        }
        polygon_vertices_box(&self.vertices, shape, extra, boundary)
    }
    pub(crate) fn within(
        &self,
        other: &Self,
        extra: i64,
        boundary: DistanceBoundary,
    ) -> Result<bool> {
        if extra >= 0
            && !(extra == 0 && boundary == DistanceBoundary::Strict)
            && (0..2).any(|axis| {
                self.bounds[axis][0] - other.bounds[axis][1] > i128::from(extra)
                    || other.bounds[axis][0] - self.bounds[axis][1] > i128::from(extra)
            })
        {
            return Ok(false);
        }
        vertices_within(&self.vertices, &other.vertices, extra, boundary)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn rect(x: i64) -> Polygon {
        Polygon::new(vec![
            Point::new(x, 0),
            Point::new(x + 10, 0),
            Point::new(x + 10, 10),
            Point::new(x, 10),
        ])
    }
    #[test]
    fn certificates_preserve_source_tangency_gap_and_refusals() {
        let mut source = rect(0);
        source.vertices.push(source.vertices[0]);
        let a = CertifiedPolygon::new(&source).unwrap();
        assert_eq!(a.source(), &source);
        assert!(a.contains(Point::new(10, 5)).unwrap());
        assert!(!a.contains(Point::new(11, 5)).unwrap());
        let touching = CertifiedPolygon::new(&rect(10)).unwrap();
        let gap = CertifiedPolygon::new(&rect(11)).unwrap();
        assert!(a.within(&touching, 0, DistanceBoundary::Inclusive).unwrap());
        assert!(!a.within(&gap, 0, DistanceBoundary::Inclusive).unwrap());
        assert!(a.within(&gap, 1, DistanceBoundary::Inclusive).unwrap());
        assert_eq!(
            a.within(&gap, 0, DistanceBoundary::Strict),
            Err(GeometryError::UnresolvedPredicate)
        );
        assert_eq!(
            a.within(&gap, -1, DistanceBoundary::Inclusive),
            Err(GeometryError::InvalidWidth)
        );
        let crossing = Polygon::new(vec![
            Point::new(0, 0),
            Point::new(4, 4),
            Point::new(0, 4),
            Point::new(3, 0),
        ]);
        assert!(matches!(
            CertifiedPolygon::new(&crossing),
            Err(GeometryError::InvalidPolygon)
        ));
    }
}
