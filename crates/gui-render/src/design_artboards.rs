use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use image::{Rgba, RgbaImage};

use crate::design_tokens::{chrome, content};
use crate::visual_diff::{DiffPolicy, compare_images, write_diff_image, write_report};

const ARTBOARD_DIR: &str = "crates/gui-render/testdata/golden/design-system";
const ARTBOARD_NAME: &str = "token-foundations";
const SCALES: &[f32] = &[1.0, 1.25, 1.5, 2.0];
const LOGICAL_WIDTH: u32 = 720;
const LOGICAL_HEIGHT: u32 = 420;

pub fn check_design_artboards() -> Result<()> {
    clean_design_artboard_artifacts()?;
    for scale in SCALES {
        let actual = render_token_foundations_artboard(*scale);
        let actual_path = artifact_path(*scale, "actual.png");
        let diff_path = artifact_path(*scale, "diff.png");
        let report_path = artifact_path(*scale, "report.txt");
        let golden_path = artifact_path(*scale, "golden.png");
        actual
            .save(&actual_path)
            .with_context(|| format!("write design artboard actual {}", actual_path.display()))?;
        let expected = image::open(&golden_path)
            .with_context(|| format!("read design artboard golden {}", golden_path.display()))?
            .to_rgba8();
        let policy = DiffPolicy::Exact;
        let result = compare_images(&expected, &actual, policy.clone())?;
        write_report(&result, &policy, &report_path)?;
        if !result.passed {
            write_diff_image(&expected, &actual, &diff_path)?;
            bail!(
                "design artboard {ARTBOARD_NAME} scale {scale} failed: {} differing pixels ({:.6}%)",
                result.differing_pixels,
                result.differing_pct
            );
        }
    }
    clean_design_artboard_artifacts()
}

pub fn bless_design_artboards() -> Result<()> {
    std::fs::create_dir_all(artboard_dir()).with_context(|| {
        format!(
            "create design artboard golden directory {}",
            artboard_dir().display()
        )
    })?;
    for scale in SCALES {
        let image = render_token_foundations_artboard(*scale);
        let golden_path = artifact_path(*scale, "golden.png");
        image
            .save(&golden_path)
            .with_context(|| format!("write design artboard golden {}", golden_path.display()))?;
    }
    clean_design_artboard_artifacts()
}

pub fn clean_design_artboard_artifacts() -> Result<()> {
    for scale in SCALES {
        for suffix in ["actual.png", "diff.png", "report.txt"] {
            let path = artifact_path(*scale, suffix);
            match std::fs::remove_file(&path) {
                Ok(()) => {}
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => {
                    return Err(error)
                        .with_context(|| format!("remove design artboard {}", path.display()));
                }
            }
        }
    }
    Ok(())
}

fn render_token_foundations_artboard(scale: f32) -> RgbaImage {
    let width = scaled(LOGICAL_WIDTH as f32, scale) as u32;
    let height = scaled(LOGICAL_HEIGHT as f32, scale) as u32;
    let mut image = RgbaImage::from_pixel(width, height, rgba(chrome::BG_BASE));

    fill_rect(
        &mut image,
        scale,
        24.0,
        24.0,
        672.0,
        372.0,
        chrome::SURFACE_01,
    );
    stroke_rect(
        &mut image,
        scale,
        24.0,
        24.0,
        672.0,
        372.0,
        chrome::BORDER_SUBTLE,
    );

    let surfaces = [
        chrome::CANVAS,
        chrome::BG_BASE,
        chrome::SURFACE_01,
        chrome::SURFACE_02,
        chrome::SURFACE_03,
        chrome::BORDER_SUBTLE,
        chrome::BORDER_STRONG,
    ];
    for (index, color) in surfaces.iter().enumerate() {
        let x = 48.0 + index as f32 * 88.0;
        fill_rect(&mut image, scale, x, 54.0, 64.0, 54.0, *color);
        stroke_rect(
            &mut image,
            scale,
            x,
            54.0,
            64.0,
            54.0,
            chrome::BORDER_STRONG,
        );
    }

    let text_samples = [
        chrome::TEXT_PRIMARY,
        chrome::TEXT_SECONDARY,
        chrome::TEXT_MUTED,
        chrome::ACCENT,
    ];
    for (index, color) in text_samples.iter().enumerate() {
        let y = 140.0 + index as f32 * 24.0;
        fill_rect(&mut image, scale, 52.0, y, 180.0, 10.0, *color);
        fill_rect(
            &mut image,
            scale,
            248.0,
            y - 6.0,
            98.0,
            22.0,
            chrome::SURFACE_02,
        );
        fill_rect(&mut image, scale, 258.0, y, 76.0, 10.0, *color);
    }

    fill_rect(&mut image, scale, 384.0, 136.0, 132.0, 40.0, chrome::ACCENT);
    fill_rect(
        &mut image,
        scale,
        402.0,
        151.0,
        96.0,
        10.0,
        chrome::TEXT_ON_ACCENT,
    );
    fill_rect(
        &mut image,
        scale,
        532.0,
        136.0,
        132.0,
        40.0,
        chrome::ACCENT_TINT,
    );
    stroke_rect(&mut image, scale, 532.0, 136.0, 132.0, 40.0, chrome::ACCENT);

    let content_tokens = [
        content::COPPER_FRONT,
        content::COPPER_BACK,
        content::COPPER_IN1,
        content::COPPER_IN2,
        content::SILK_TOP,
        content::MASK,
        content::PASTE,
        content::EDGE,
        content::PAD,
        content::VIA,
        content::RATSNEST,
        content::SELECTION,
    ];
    for (index, color) in content_tokens.iter().enumerate() {
        let x = 48.0 + (index % 6) as f32 * 104.0;
        let y = 238.0 + (index / 6) as f32 * 60.0;
        fill_rect(&mut image, scale, x, y, 76.0, 34.0, *color);
        stroke_rect(&mut image, scale, x, y, 76.0, 34.0, chrome::BORDER_STRONG);
    }

    image
}

fn fill_rect(image: &mut RgbaImage, scale: f32, x: f32, y: f32, w: f32, h: f32, color: [f32; 3]) {
    let x0 = scaled(x, scale).max(0.0) as u32;
    let y0 = scaled(y, scale).max(0.0) as u32;
    let x1 = scaled(x + w, scale).min(image.width() as f32) as u32;
    let y1 = scaled(y + h, scale).min(image.height() as f32) as u32;
    let pixel = rgba(color);
    for yy in y0..y1 {
        for xx in x0..x1 {
            image.put_pixel(xx, yy, pixel);
        }
    }
}

fn stroke_rect(image: &mut RgbaImage, scale: f32, x: f32, y: f32, w: f32, h: f32, color: [f32; 3]) {
    fill_rect(image, scale, x, y, w, 1.0, color);
    fill_rect(image, scale, x, y + h - 1.0, w, 1.0, color);
    fill_rect(image, scale, x, y, 1.0, h, color);
    fill_rect(image, scale, x + w - 1.0, y, 1.0, h, color);
}

fn scaled(value: f32, scale: f32) -> f32 {
    (value * scale).round()
}

fn rgba(color: [f32; 3]) -> Rgba<u8> {
    Rgba([
        (color[0].clamp(0.0, 1.0) * 255.0).round() as u8,
        (color[1].clamp(0.0, 1.0) * 255.0).round() as u8,
        (color[2].clamp(0.0, 1.0) * 255.0).round() as u8,
        255,
    ])
}

fn artifact_path(scale: f32, suffix: &str) -> PathBuf {
    artboard_dir().join(format!(
        "{ARTBOARD_NAME}.scale-{}.{}",
        scale_suffix(scale),
        suffix
    ))
}

fn artboard_dir() -> PathBuf {
    repo_root().join(ARTBOARD_DIR)
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("gui-render crate should live under <repo>/crates/gui-render")
        .to_path_buf()
}

fn scale_suffix(scale_factor: f32) -> String {
    let rounded = (scale_factor * 100.0).round() / 100.0;
    format!("{rounded:.2}").replace('.', "_")
}
