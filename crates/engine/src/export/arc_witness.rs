//! Policy-independent source/output certification. No manufacturing allowance.
use crate::board::{
    Track,
    nominal_geometry::{CertifiedArc, GeometryError, Rational as R},
};
use crate::ir::geometry::Point;
use std::cmp::Ordering;
use uuid::Uuid;

/// Existing FSLAX46Y46/MOMM coordinates and I/J use a 1 nm grid.
pub const GERBER_GRID_NM: i64 = 1;
const MAX_COORD_NM: i64 = 9_999_999_999;
type Result<T> = std::result::Result<T, ArcWitnessError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArcWitnessError {
    NotAnArc,
    Geometry(GeometryError),
    OutputRange,
    EndpointChanged,
    WidthChanged,
    DirectionChanged,
    SourceConflict,
}
impl From<GeometryError> for ArcWitnessError {
    fn from(value: GeometryError) -> Self {
        Self::Geometry(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ArcOutputPrimitive {
    pub from: Point,
    pub to: Point,
    /// Signed I/J offsets from `from`, in nm, not absolute center coordinates.
    pub center_offset: Point,
    pub width_nm: i64,
    pub counterclockwise: bool,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArcOutputDisposition {
    Exact,
    AllowanceRequired,
    /// Unequal output radii or changed sweep: no certified curve is asserted.
    UncertifiedConstruction,
}

/// Immutable witness bound to ALL source geometry, Net, layer and UUID fields.
/// Bounds are conservative error observations, never acceptance criteria.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArcOutputWitness {
    source: Track,
    primitive: ArcOutputPrimitive,
    exact_center_offset: [R; 2],
    source_radius_squared_nm2: R,
    output_radius_squared_nm2: [R; 2],
    center_error_l1_nm: R,
    /// Start/end radius mismatch <= 2*center_error_l1, by reverse triangle.
    endpoint_radius_mismatch_upper_nm: R,
    /// Symmetric Hausdorff envelope for the centerline AND same-width copper.
    occupied_deviation_upper_nm: Option<R>,
    sweep_preserved: bool,
    disposition: ArcOutputDisposition,
    primitives: Vec<ArcOutputPrimitive>,
    endpoints_stable: bool,
}
impl ArcOutputWitness {
    pub fn source_id(&self) -> Uuid {
        self.source.uuid
    }
    pub fn source(&self) -> &Track {
        &self.source
    }
    pub fn primitive(&self) -> ArcOutputPrimitive {
        self.primitive
    }
    pub fn exact_center_offset(&self) -> [R; 2] {
        self.exact_center_offset
    }
    pub fn source_radius_squared_nm2(&self) -> R {
        self.source_radius_squared_nm2
    }
    pub fn output_radius_squared_nm2(&self) -> [R; 2] {
        self.output_radius_squared_nm2
    }
    pub fn center_error_l1_nm(&self) -> R {
        self.center_error_l1_nm
    }
    pub fn endpoint_radius_mismatch_upper_nm(&self) -> R {
        self.endpoint_radius_mismatch_upper_nm
    }
    pub fn occupied_deviation_upper_nm(&self) -> Option<R> {
        self.occupied_deviation_upper_nm
    }
    pub fn sweep_preserved(&self) -> bool {
        self.sweep_preserved
    }
    /// Major arcs split at the exact grid antipode: one semicircle plus a
    /// remaining arc below 180 degrees, avoiding near-full output instability.
    /// Gerber 2026.05 section 4.14: do not write arcs with endpoint
    /// separation below 1 micrometre. This is format guidance, not M1 allowance.
    pub fn endpoints_stable(&self) -> bool {
        self.endpoints_stable
    }
    pub fn output_primitives(&self) -> &[ArcOutputPrimitive] {
        &self.primitives
    }
    pub fn disposition(&self) -> ArcOutputDisposition {
        self.disposition
    }
}

fn abs(v: R) -> std::result::Result<R, GeometryError> {
    if v.numerator < 0 {
        v.checked_neg()
    } else {
        Ok(v)
    }
}
fn norm(v: [R; 2]) -> std::result::Result<R, GeometryError> {
    v[0].square()?.checked_add(v[1].square()?)
}
fn cross(v: [R; 2], w: [R; 2]) -> std::result::Result<R, GeometryError> {
    v[0].checked_mul(w[1])?.checked_sub(v[1].checked_mul(w[0])?)
}
fn rounded(v: R) -> Result<i64> {
    let q = v.numerator / v.denominator;
    let rem = v.numerator % v.denominator;
    // Nearest grid point; exact ties away from zero. Avoid 2*remainder overflow.
    let half = v.denominator / 2 + v.denominator % 2;
    let q = if rem.unsigned_abs() >= half as u128 {
        q.checked_add(v.numerator.signum())
            .ok_or(GeometryError::ArithmeticRange)?
    } else {
        q
    };
    i64::try_from(q).map_err(|_| ArcWitnessError::OutputRange)
}
fn in_range(p: Point) -> bool {
    p.x.unsigned_abs() <= MAX_COORD_NM as u64 && p.y.unsigned_abs() <= MAX_COORD_NM as u64
}

pub fn arc_output_witness(track: &Track) -> Result<ArcOutputWitness> {
    let arc = CertifiedArc::new(
        track.from,
        track.midpoint.ok_or(ArcWitnessError::NotAnArc)?,
        track.to,
    )?;
    validate_arc_output_primitive(
        track,
        ArcOutputPrimitive {
            from: track.from,
            to: track.to,
            center_offset: Point::new(rounded(arc.center[0])?, rounded(arc.center[1])?),
            width_nm: track.width,
            counterclockwise: arc.counterclockwise,
        },
    )
}

/// Validate a proposed primitive independently of the serialization code.
/// There is intentionally no API accepting a nonzero manufacturing allowance.
pub fn validate_arc_output_primitive(
    track: &Track,
    primitive: ArcOutputPrimitive,
) -> Result<ArcOutputWitness> {
    let arc = CertifiedArc::new(
        track.from,
        track.midpoint.ok_or(ArcWitnessError::NotAnArc)?,
        track.to,
    )?;
    if track.width <= 0 {
        return Err(GeometryError::InvalidWidth.into());
    }
    if primitive.from != track.from || primitive.to != track.to {
        return Err(ArcWitnessError::EndpointChanged);
    }
    if primitive.width_nm != track.width {
        return Err(ArcWitnessError::WidthChanged);
    }
    if primitive.counterclockwise != arc.counterclockwise {
        return Err(ArcWitnessError::DirectionChanged);
    }
    if !in_range(primitive.from) || !in_range(primitive.to) || !in_range(primitive.center_offset) {
        return Err(ArcWitnessError::OutputRange);
    }
    let center = [
        R::integer(i128::from(primitive.center_offset.x)),
        R::integer(i128::from(primitive.center_offset.y)),
    ];
    let start = [center[0].checked_neg()?, center[1].checked_neg()?];
    let end = [
        R::integer(i128::from(track.to.x) - i128::from(track.from.x)).checked_sub(center[0])?,
        R::integer(i128::from(track.to.y) - i128::from(track.from.y)).checked_sub(center[1])?,
    ];
    let radii = [norm(start)?, norm(end)?];
    let error = abs(center[0].checked_sub(arc.center[0])?)?
        .checked_add(abs(center[1].checked_sub(arc.center[1])?)?)?;
    let equal_radius = radii[0].compare(radii[1])? == Ordering::Equal && radii[0].numerator > 0;
    let orient = if arc.counterclockwise { 1 } else { -1 };
    let source_cross = cross(arc.radial_vector(arc.from)?, arc.radial_vector(arc.to)?)?
        .numerator
        .signum()
        * orient;
    let output_cross = cross(start, end)?.numerator.signum() * orient;
    let sweep_preserved = equal_radius && source_cross == output_cross;
    let bound = error.checked_mul(R::integer(2))?;
    // Equal-radius circles through two fixed endpoints share a chord and their
    // centers lie on its perpendicular bisector. From the chord midpoint, a ray
    // into either half-plane meets the selected arc at radial distance
    // h*s + sqrt(a^2 + h^2*s^2), where a>0 is the half-chord and |s|<=1.
    // Its derivative in h has magnitude <=2. Corresponding rays therefore give
    // Hausdorff distance <=2*|delta center| <=2*L1(delta center). Same endpoint,
    // direction and sweep checks prevent complement/full-circle substitution.
    // Minkowski addition by the identical radius width/2 cannot enlarge that
    // envelope. No floating point, samples or format fuzziness certify it.
    let mut disposition = if !sweep_preserved {
        ArcOutputDisposition::UncertifiedConstruction
    } else if error.numerator == 0 {
        ArcOutputDisposition::Exact
    } else {
        ArcOutputDisposition::AllowanceRequired
    };
    let mut primitives = vec![primitive];
    if sweep_preserved && output_cross < 0 {
        let coordinate = |start: i64, offset: i64| -> Result<i64> {
            i64::try_from(i128::from(start) + 2 * i128::from(offset))
                .map_err(|_| ArcWitnessError::OutputRange)
        };
        let opposite = Point::new(
            coordinate(primitive.from.x, primitive.center_offset.x)?,
            coordinate(primitive.from.y, primitive.center_offset.y)?,
        );
        let second = ArcOutputPrimitive {
            from: opposite,
            center_offset: Point::new(-primitive.center_offset.x, -primitive.center_offset.y),
            ..primitive
        };
        if !in_range(opposite) {
            return Err(ArcWitnessError::OutputRange);
        }
        primitives = vec![
            ArcOutputPrimitive {
                to: opposite,
                ..primitive
            },
            second,
        ];
    }
    let endpoints_stable = primitives.iter().all(|p| {
        let x = i128::from(p.to.x) - i128::from(p.from.x);
        let y = i128::from(p.to.y) - i128::from(p.from.y);
        // Validated FS46 endpoints bound both squares safely within i128.
        x * x + y * y >= 1_000_000
    });
    if !endpoints_stable {
        disposition = ArcOutputDisposition::UncertifiedConstruction;
    }
    Ok(ArcOutputWitness {
        source: track.clone(),
        primitive,
        exact_center_offset: arc.center,
        source_radius_squared_nm2: arc.radius_squared,
        output_radius_squared_nm2: radii,
        center_error_l1_nm: error,
        endpoint_radius_mismatch_upper_nm: if equal_radius { R::integer(0) } else { bound },
        occupied_deviation_upper_nm: sweep_preserved.then_some(bound),
        sweep_preserved,
        disposition,
        primitives,
        endpoints_stable,
    })
}
