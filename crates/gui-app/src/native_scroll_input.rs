//! Native wheel units are normalized once; consumers retain their row/pixel policy.
use winit::event::MouseScrollDelta;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum VerticalWheel {
    Lines(f32),
    LogicalPixels(f32),
}

impl VerticalWheel {
    pub(crate) fn from_native(delta: MouseScrollDelta, scale: f32) -> Option<Self> {
        let normalized = match delta {
            MouseScrollDelta::LineDelta(_, y) => Self::Lines(y),
            MouseScrollDelta::PixelDelta(position) if scale.is_finite() && scale > 0.0 => {
                Self::LogicalPixels((position.y / f64::from(scale)) as f32)
            }
            MouseScrollDelta::PixelDelta(_) => return None,
        };
        let (Self::Lines(value) | Self::LogicalPixels(value)) = normalized;
        (value.is_finite() && value != 0.0).then_some(normalized)
    }

    pub(crate) fn pixels(self, pixels_per_line: f32) -> f32 {
        match self {
            Self::Lines(lines) => lines * pixels_per_line,
            Self::LogicalPixels(pixels) => pixels,
        }
    }

    pub(crate) fn lines(self, pixels_per_line: f32) -> f32 {
        match self {
            Self::Lines(lines) => lines,
            Self::LogicalPixels(pixels) => pixels / pixels_per_line,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use winit::dpi::PhysicalPosition;

    #[test]
    fn physical_motion_matches_logical_motion_across_scale_and_consumer_units() {
        for scale in [1.0, 1.5, 2.0] {
            for logical in [-40.0, -0.25, 0.25, 40.0] {
                let wheel = VerticalWheel::from_native(
                    MouseScrollDelta::PixelDelta(PhysicalPosition::new(0.0, logical * scale)),
                    scale as f32,
                )
                .unwrap();
                assert_eq!(wheel.pixels(40.0), logical as f32);
                assert_eq!(wheel.lines(20.0), logical as f32 / 20.0);
            }
            let wheel =
                VerticalWheel::from_native(MouseScrollDelta::LineDelta(0.0, -0.5), scale as f32)
                    .unwrap();
            assert_eq!(wheel.lines(20.0), -0.5);
            assert_eq!(wheel.pixels(40.0), -20.0);
        }
    }

    #[test]
    fn horizontal_nonfinite_and_invalid_physical_scale_do_not_become_vertical_input() {
        for y in [0.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert!(
                VerticalWheel::from_native(
                    MouseScrollDelta::PixelDelta(PhysicalPosition::new(20.0, y)),
                    1.0
                )
                .is_none()
            );
            assert!(
                VerticalWheel::from_native(MouseScrollDelta::LineDelta(20.0, y as f32), 1.0)
                    .is_none()
            );
        }
        for scale in [0.0, -1.0, f32::NAN, f32::INFINITY] {
            assert!(
                VerticalWheel::from_native(
                    MouseScrollDelta::PixelDelta(PhysicalPosition::new(0.0, 20.0)),
                    scale
                )
                .is_none()
            );
        }
    }
}
