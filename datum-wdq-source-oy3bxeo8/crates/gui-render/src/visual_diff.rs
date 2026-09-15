use std::path::Path;

use anyhow::{Context, Result, bail};
use image::{Rgba, RgbaImage};

#[derive(Debug, Clone, PartialEq)]
pub enum DiffPolicy {
    Exact,
    Tolerance { per_pixel: u8, total_px_pct: f64 },
}

#[derive(Debug, Clone, PartialEq)]
pub struct DiffResult {
    pub passed: bool,
    pub differing_pixels: u64,
    pub differing_pct: f64,
    pub max_channel_delta: u8,
    pub width: u32,
    pub height: u32,
}

pub fn compare_images(
    expected: &RgbaImage,
    actual: &RgbaImage,
    policy: DiffPolicy,
) -> Result<DiffResult> {
    if expected.dimensions() != actual.dimensions() {
        bail!(
            "image dimensions differ: expected {:?}, actual {:?}",
            expected.dimensions(),
            actual.dimensions()
        );
    }
    let (width, height) = expected.dimensions();
    let total_pixels = u64::from(width) * u64::from(height);
    let mut differing_pixels = 0_u64;
    let mut max_channel_delta = 0_u8;
    let per_pixel_tolerance = match policy {
        DiffPolicy::Exact => 0,
        DiffPolicy::Tolerance { per_pixel, .. } => per_pixel,
    };
    for (expected_px, actual_px) in expected.pixels().zip(actual.pixels()) {
        let mut pixel_differs = false;
        for channel in 0..4 {
            let delta = expected_px[channel].abs_diff(actual_px[channel]);
            max_channel_delta = max_channel_delta.max(delta);
            if delta > per_pixel_tolerance {
                pixel_differs = true;
            }
        }
        if pixel_differs {
            differing_pixels += 1;
        }
    }
    let differing_pct = if total_pixels == 0 {
        0.0
    } else {
        (differing_pixels as f64 / total_pixels as f64) * 100.0
    };
    let passed = match policy {
        DiffPolicy::Exact => differing_pixels == 0,
        DiffPolicy::Tolerance { total_px_pct, .. } => differing_pct <= total_px_pct,
    };
    Ok(DiffResult {
        passed,
        differing_pixels,
        differing_pct,
        max_channel_delta,
        width,
        height,
    })
}

pub fn write_diff_image(
    expected: &RgbaImage,
    actual: &RgbaImage,
    path: impl AsRef<Path>,
) -> Result<()> {
    if expected.dimensions() != actual.dimensions() {
        bail!(
            "image dimensions differ: expected {:?}, actual {:?}",
            expected.dimensions(),
            actual.dimensions()
        );
    }
    let (width, height) = expected.dimensions();
    let mut diff = RgbaImage::new(width, height);
    for y in 0..height {
        for x in 0..width {
            let expected_px = expected.get_pixel(x, y);
            let actual_px = actual.get_pixel(x, y);
            let out = if expected_px == actual_px {
                let grey =
                    ((u16::from(actual_px[0]) + u16::from(actual_px[1]) + u16::from(actual_px[2]))
                        / 3) as u8;
                Rgba([grey, grey, grey, actual_px[3]])
            } else {
                Rgba([255, 0, 0, 255])
            };
            diff.put_pixel(x, y, out);
        }
    }
    diff.save(path.as_ref())
        .with_context(|| format!("write visual diff image {}", path.as_ref().display()))
}

pub fn write_report(
    result: &DiffResult,
    policy: &DiffPolicy,
    path: impl AsRef<Path>,
) -> Result<()> {
    let policy_text = match policy {
        DiffPolicy::Exact => "exact".to_string(),
        DiffPolicy::Tolerance {
            per_pixel,
            total_px_pct,
        } => format!("tolerance(per_pixel={per_pixel}, total_px_pct={total_px_pct})"),
    };
    let report = format!(
        "policy: {policy_text}\npassed: {}\nsize: {}x{}\ndiffering_pixels: {}\ndiffering_pct: {:.6}\nmax_channel_delta: {}\n",
        result.passed,
        result.width,
        result.height,
        result.differing_pixels,
        result.differing_pct,
        result.max_channel_delta,
    );
    std::fs::write(path.as_ref(), report)
        .with_context(|| format!("write visual diff report {}", path.as_ref().display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_compare_passes_identical_images() {
        let expected = RgbaImage::from_pixel(2, 2, Rgba([1, 2, 3, 255]));
        let actual = expected.clone();
        let result =
            compare_images(&expected, &actual, DiffPolicy::Exact).expect("compare should run");
        assert!(result.passed);
        assert_eq!(result.differing_pixels, 0);
    }

    #[test]
    fn tolerance_compare_counts_pixels_over_threshold() {
        let expected = RgbaImage::from_pixel(2, 1, Rgba([10, 10, 10, 255]));
        let mut actual = expected.clone();
        actual.put_pixel(0, 0, Rgba([11, 10, 10, 255]));
        actual.put_pixel(1, 0, Rgba([20, 10, 10, 255]));
        let result = compare_images(
            &expected,
            &actual,
            DiffPolicy::Tolerance {
                per_pixel: 1,
                total_px_pct: 50.0,
            },
        )
        .expect("compare should run");
        assert!(result.passed);
        assert_eq!(result.differing_pixels, 1);
        assert_eq!(result.max_channel_delta, 10);
    }
}
