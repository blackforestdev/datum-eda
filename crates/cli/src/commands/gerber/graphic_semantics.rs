//! Exact graphic primitives supported by the existing manufacturing comparator.
use super::*;
use crate::Point;
use anyhow::{Context, Result, bail};

pub(super) fn circle_aperture(params: &str) -> Result<ParsedGerberAperture> {
    let fields: Vec<_> = params.split('X').collect();
    let diameter = super::parse_mm_6_to_nm(fields[0])
        .filter(|n| *n > 0)
        .context("invalid circular aperture diameter")?;
    match fields.as_slice() {
        [_] => Ok(ParsedGerberAperture::Circle {
            diameter_nm: diameter,
        }),
        [_, hole] => {
            let hole = super::parse_mm_6_to_nm(hole)
                .filter(|n| *n > 0 && *n < diameter)
                .context("invalid transparent aperture hole")?;
            Ok(ParsedGerberAperture::Ring {
                diameter_nm: diameter,
                hole_nm: hole,
            })
        }
        _ => bail!("unsupported circular aperture parameters"),
    }
}
pub(super) fn macro_ring(line: &str, outer: bool) -> Result<Vec<Point>> {
    let fields: Vec<_> = line
        .strip_suffix('*')
        .context("unterminated polygon macro")?
        .split(',')
        .collect();
    if fields.len() < 12
        || fields[0] != "4"
        || fields[1] != if outer { "1" } else { "0" }
        || fields.last() != Some(&"0")
    {
        bail!("unsupported exact polygon macro primitive");
    }
    let count = fields[2]
        .parse::<usize>()
        .context("invalid polygon macro vertex count")?;
    if !(3..=5000).contains(&count) || fields.len() != 6 + 2 * count {
        bail!("invalid polygon macro vertex count");
    }
    let mut points = Vec::new();
    for pair in fields[3..fields.len() - 1].as_chunks::<2>().0 {
        points.push(Point::new(
            super::parse_mm_6_to_nm(pair[0]).context("inexact macro X")?,
            super::parse_mm_6_to_nm(pair[1]).context("inexact macro Y")?,
        ));
    }
    if points.first() != points.last() {
        bail!("open polygon macro contour");
    }
    points.pop();
    Ok(points)
}
pub(super) fn arc_command(line: &str, from: Point) -> Result<(Point, Point)> {
    let fields = line.strip_prefix('X').context("arc requires X")?;
    let (x, fields) = fields.split_once('Y').context("arc requires Y")?;
    let (y, fields) = fields.split_once('I').context("arc requires I")?;
    let (i, fields) = fields.split_once('J').context("arc requires J")?;
    let j = fields.strip_suffix("D01*").context("arc requires D01")?;
    let to = Point::new(x.parse()?, y.parse()?);
    let offset = Point::new(i.parse()?, j.parse()?);
    let squared = |x: i128, y: i128| {
        x.checked_mul(x)
            .and_then(|a| y.checked_mul(y).and_then(|b| a.checked_add(b)))
    };
    let start = squared(i128::from(offset.x), i128::from(offset.y))
        .filter(|n| *n > 0)
        .context("invalid arc start radius")?;
    let end = squared(
        i128::from(to.x) - i128::from(from.x) - i128::from(offset.x),
        i128::from(to.y) - i128::from(from.y) - i128::from(offset.y),
    )
    .context("arc radius arithmetic range")?;
    if start != end {
        bail!("arc radii differ; no approximate comparison authority");
    }
    Ok((to, offset))
}
pub(crate) fn arc_signature(geometry: &ParsedGerberGeometry) -> String {
    let ParsedGerberGeometry::Arc {
        width_nm,
        from,
        to,
        center_offset,
        counterclockwise,
    } = geometry
    else {
        unreachable!()
    };
    format!(
        "width_mm={} from=({}, {}) to=({}, {}) offset=({}, {}) ccw={counterclockwise}",
        render_mm_6(*width_nm),
        from.x,
        from.y,
        to.x,
        to.y,
        center_offset.x,
        center_offset.y
    )
}
pub(super) fn polygon_signature(outer: &[Point], holes: &[Vec<Point>], position: Point) -> String {
    let mut holes: Vec<_> = holes
        .iter()
        .map(|h| super::render_region_geometry(h))
        .collect();
    holes.sort();
    format!(
        "shape=polygon outer={} holes=[{}] at=({}, {})",
        super::render_region_geometry(outer),
        holes.join(";"),
        position.x,
        position.y
    )
}
