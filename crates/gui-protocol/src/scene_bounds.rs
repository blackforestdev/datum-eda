//! Shared scene envelope for existing board geometry and recorded graphic paths.
use crate::{PointNm, RectNm, SceneBounds};

// Scene import threads many primitive-geometry parameters.
#[allow(clippy::too_many_arguments)]
pub(super) fn scene_bounds<'a>(
    outline: impl Iterator<Item = &'a PointNm>,
    components: impl Iterator<Item = &'a PointNm>,
    pads: impl Iterator<Item = &'a PointNm>,
    component_graphics: impl Iterator<Item = &'a PointNm>,
    component_texts: impl Iterator<Item = &'a PointNm>,
    board_texts: impl Iterator<Item = &'a PointNm>,
    tracks: impl Iterator<Item = &'a PointNm>,
    vias: impl Iterator<Item = &'a PointNm>,
    zones: impl Iterator<Item = &'a PointNm>,
) -> SceneBounds {
    let mut points: Vec<PointNm> = Vec::new();
    points.extend(outline.copied());
    points.extend(components.copied());
    points.extend(pads.copied());
    points.extend(component_graphics.copied());
    points.extend(component_texts.copied());
    points.extend(board_texts.copied());
    points.extend(tracks.copied());
    points.extend(vias.copied());
    points.extend(zones.copied());
    if points.is_empty() {
        return SceneBounds {
            min_x: 0,
            min_y: 0,
            max_x: 1,
            max_y: 1,
        };
    }
    let mut min_x = i64::MAX;
    let mut min_y = i64::MAX;
    let mut max_x = i64::MIN;
    let mut max_y = i64::MIN;
    for point in points {
        min_x = min_x.min(point.x);
        min_y = min_y.min(point.y);
        max_x = max_x.max(point.x);
        max_y = max_y.max(point.y);
    }
    SceneBounds {
        min_x,
        min_y,
        max_x,
        max_y,
    }
}

pub(super) fn rect_corners(rect: RectNm) -> [PointNm; 4] {
    [
        PointNm {
            x: rect.min_x,
            y: rect.min_y,
        },
        PointNm {
            x: rect.max_x,
            y: rect.min_y,
        },
        PointNm {
            x: rect.max_x,
            y: rect.max_y,
        },
        PointNm {
            x: rect.min_x,
            y: rect.max_y,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn graphic_paths_outside_the_board_expand_fit_without_changing_empty_graphics() {
        let board = [
            PointNm {
                x: 137_664_214,
                y: 77_145_786,
            },
            PointNm {
                x: 165_604_214,
                y: 113_100_000,
            },
        ];
        let graphic = [PointNm {
            x: 133_050_000,
            y: 105_900_000,
        }];
        let bounds = |graphics: &[PointNm]| {
            scene_bounds(
                board.iter(),
                [].iter(),
                [].iter(),
                graphics.iter(),
                [].iter(),
                [].iter(),
                [].iter(),
                [].iter(),
                [].iter(),
            )
        };
        let before = bounds(&[]);
        let after = bounds(&graphic);
        assert_eq!(before.min_x, board[0].x);
        assert_eq!(after.min_x, graphic[0].x);
        assert_eq!(
            (before.min_y, before.max_x, before.max_y),
            (after.min_y, after.max_x, after.max_y)
        );
    }
}
