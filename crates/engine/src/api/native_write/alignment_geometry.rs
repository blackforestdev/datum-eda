//! Editor-independent exact alignment of complete body extents.
use crate::{error::EngineError, ir::geometry::Point};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AlignmentAxis {
    Horizontal,
    Vertical,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AlignmentPosition {
    Left,
    Center,
    Right,
    Top,
    Bottom,
}

/// Twice-nanometer extents preserve odd authored stroke widths exactly.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct BodyBounds {
    pub min: [i128; 2],
    pub max: [i128; 2],
}

pub(crate) fn destinations(
    bodies: &[(Point, BodyBounds)],
    axis: AlignmentAxis,
    position: AlignmentPosition,
) -> Result<Vec<Point>, EngineError> {
    use AlignmentAxis::*;
    use AlignmentPosition::*;
    let dimension = match (axis, position) {
        (Horizontal, Left | Center | Right) => 0,
        (Vertical, Top | Center | Bottom) => 1,
        _ => {
            return Err(invalid(
                "Alignment position does not belong to the chosen axis",
            ));
        }
    };
    if bodies.len() < 2 {
        return Err(invalid("Alignment requires at least two distinct bodies"));
    }
    if bodies
        .iter()
        .any(|(_, b)| (0..2).any(|i| b.min[i] > b.max[i]))
    {
        return Err(invalid("Alignment body bounds are invalid"));
    }
    let min = bodies.iter().map(|(_, b)| b.min[dimension]).min().unwrap();
    let max = bodies.iter().map(|(_, b)| b.max[dimension]).max().unwrap();
    bodies
        .iter()
        .map(|(anchor, body)| {
            let (numerator, denominator) = match position {
                Left | Top => (min - body.min[dimension], 2),
                Right | Bottom => (max - body.max[dimension], 2),
                Center => (min + max - body.min[dimension] - body.max[dimension], 4),
            };
            if numerator % denominator != 0 {
                return Err(invalid(
                    "Exact body alignment requires a fractional-nanometer translation",
                ));
            }
            let mut result = *anchor;
            let coordinate = if dimension == 0 {
                &mut result.x
            } else {
                &mut result.y
            };
            *coordinate = i64::try_from(i128::from(*coordinate) + numerator / denominator)
                .map_err(|_| invalid("Alignment exceeds integer coordinates"))?;
            Ok(result)
        })
        .collect()
}

fn invalid(message: &str) -> EngineError {
    EngineError::Validation(message.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn body_edges_and_centers_align_without_using_anchor_extrema() {
        let bodies = [
            (
                Point::new(10, 30),
                BodyBounds {
                    min: [16, 56],
                    max: [28, 64],
                },
            ),
            (
                Point::new(40, 50),
                BodyBounds {
                    min: [60, 90],
                    max: [100, 110],
                },
            ),
        ];
        for (axis, position, expected) in [
            (
                AlignmentAxis::Horizontal,
                AlignmentPosition::Left,
                [Point::new(10, 30), Point::new(18, 50)],
            ),
            (
                AlignmentAxis::Horizontal,
                AlignmentPosition::Right,
                [Point::new(46, 30), Point::new(40, 50)],
            ),
            (
                AlignmentAxis::Horizontal,
                AlignmentPosition::Center,
                [Point::new(28, 30), Point::new(29, 50)],
            ),
            (
                AlignmentAxis::Vertical,
                AlignmentPosition::Top,
                [Point::new(10, 30), Point::new(40, 33)],
            ),
            (
                AlignmentAxis::Vertical,
                AlignmentPosition::Bottom,
                [Point::new(10, 53), Point::new(40, 50)],
            ),
            (
                AlignmentAxis::Vertical,
                AlignmentPosition::Center,
                [Point::new(10, 42), Point::new(40, 42)],
            ),
        ] {
            // The center Y case has a half-nm result and must refuse atomically.
            if axis == AlignmentAxis::Vertical && position == AlignmentPosition::Center {
                assert!(destinations(&bodies, axis, position).is_err());
            } else {
                assert_eq!(destinations(&bodies, axis, position).unwrap(), expected);
            }
        }
        assert!(destinations(&bodies, AlignmentAxis::Horizontal, AlignmentPosition::Top).is_err());
    }
}
