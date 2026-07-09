use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use datum_gui_protocol::{LiveReviewRequest, ReviewWorkspaceState};
use image::RgbaImage;

use crate::visual_capture::OffscreenRenderer;
use crate::visual_diff::{DiffPolicy, DiffResult, compare_images, write_diff_image, write_report};
use crate::visual_manifest::FixtureManifest;
use crate::{CameraState, ShellLayout};

const NM_PER_MM: f32 = 1_000_000.0;

#[derive(Debug, Clone, PartialEq)]
pub struct VisualFixtureRun {
    pub manifest_path: PathBuf,
    pub manifest: FixtureManifest,
    pub golden_path: PathBuf,
    pub actual_path: PathBuf,
    pub diff_path: PathBuf,
    pub report_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VisualFixtureOutcome {
    pub run: VisualFixtureRun,
    pub scale_factor: f32,
    pub result: DiffResult,
}

impl VisualFixtureRun {
    pub fn load(manifest_path: impl AsRef<Path>) -> Result<Self> {
        let manifest_path = manifest_path.as_ref();
        let manifest = FixtureManifest::load(manifest_path)?;
        let manifest_dir = manifest_path.parent().with_context(|| {
            format!(
                "fixture manifest has no parent: {}",
                manifest_path.display()
            )
        })?;
        let stem = manifest_path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .with_context(|| {
                format!(
                    "fixture manifest has no UTF-8 file stem: {}",
                    manifest_path.display()
                )
            })?;
        let artifact_stem = stem.strip_suffix(".fixture").unwrap_or(stem);
        let golden_path = manifest_dir.join(&manifest.golden.filename);

        Ok(Self {
            manifest_path: manifest_path.to_path_buf(),
            manifest,
            golden_path,
            actual_path: manifest_dir.join(format!("{artifact_stem}.actual.png")),
            diff_path: manifest_dir.join(format!("{artifact_stem}.diff.png")),
            report_path: manifest_dir.join(format!("{artifact_stem}.report.txt")),
        })
    }

    pub fn render_actual(&self) -> Result<RgbaImage> {
        self.render_actual_at_scale(1.0)
    }

    pub fn render_actual_at_scale(&self, scale_factor: f32) -> Result<RgbaImage> {
        let state = load_state_for_manifest(&self.manifest)?;
        let camera = camera_for_manifest(&self.manifest, &state);
        let mut renderer = OffscreenRenderer::new(
            self.manifest.viewport.width_px,
            self.manifest.viewport.height_px,
        )?;
        renderer.render_workspace_for_surface_scale(&state, Some(camera), scale_factor)
    }

    pub fn bless(&self) -> Result<()> {
        for scale_factor in &self.manifest.ui_scale_factors {
            self.bless_at_scale(*scale_factor)?;
        }
        self.clean_artifacts()
    }

    pub fn bless_at_scale(&self, scale_factor: f32) -> Result<()> {
        let actual = self.render_actual_at_scale(scale_factor)?;
        assert_non_blank(&actual, self.manifest.blank_check.expect_non_blank_pct)?;
        let golden_path = self.golden_path_for_scale(scale_factor);
        actual
            .save(&golden_path)
            .with_context(|| format!("write visual golden {}", golden_path.display()))?;
        self.clean_artifacts()
    }

    pub fn run(&self) -> Result<Vec<VisualFixtureOutcome>> {
        self.clean_artifacts()?;
        let mut outcomes = Vec::new();
        for scale_factor in &self.manifest.ui_scale_factors {
            let actual = self.render_actual_at_scale(*scale_factor)?;
            assert_non_blank(&actual, self.manifest.blank_check.expect_non_blank_pct)?;
            let result = self.compare_actual_at_scale(actual, *scale_factor)?;
            outcomes.push(VisualFixtureOutcome {
                run: self.clone(),
                scale_factor: *scale_factor,
                result,
            });
        }

        Ok(outcomes)
    }

    fn compare_actual_at_scale(&self, actual: RgbaImage, scale_factor: f32) -> Result<DiffResult> {
        let actual_path = self.actual_path_for_scale(scale_factor);
        let diff_path = self.diff_path_for_scale(scale_factor);
        let report_path = self.report_path_for_scale(scale_factor);
        let golden_path = self.golden_path_for_scale(scale_factor);
        actual
            .save(&actual_path)
            .with_context(|| format!("write visual actual {}", actual_path.display()))?;

        let expected = image::open(&golden_path)
            .with_context(|| format!("read visual golden {}", golden_path.display()))?
            .to_rgba8();
        let policy = diff_policy_for_manifest(&self.manifest)?;
        let result = compare_images(&expected, &actual, policy.clone())?;
        write_report(&result, &policy, &report_path)?;
        if !result.passed {
            write_diff_image(&expected, &actual, &diff_path)?;
            bail!(
                "visual fixture {} scale {} failed: {} differing pixels ({:.6}%)",
                self.manifest.name,
                scale_factor,
                result.differing_pixels,
                result.differing_pct
            );
        }
        self.clean_artifacts()?;
        Ok(result)
    }

    pub fn clean_artifacts(&self) -> Result<()> {
        for path in [&self.actual_path, &self.diff_path, &self.report_path] {
            remove_file_if_exists(path)?;
        }
        for scale_factor in &self.manifest.ui_scale_factors {
            for path in [
                self.actual_path_for_scale(*scale_factor),
                self.diff_path_for_scale(*scale_factor),
                self.report_path_for_scale(*scale_factor),
            ] {
                remove_file_if_exists(&path)?;
            }
        }
        Ok(())
    }

    fn golden_path_for_scale(&self, scale_factor: f32) -> PathBuf {
        if self.uses_legacy_single_scale_path(scale_factor) {
            return self.golden_path.clone();
        }
        self.scaled_artifact_path(scale_factor, "golden.png")
    }

    fn actual_path_for_scale(&self, scale_factor: f32) -> PathBuf {
        if self.uses_legacy_single_scale_path(scale_factor) {
            return self.actual_path.clone();
        }
        self.scaled_artifact_path(scale_factor, "actual.png")
    }

    fn diff_path_for_scale(&self, scale_factor: f32) -> PathBuf {
        if self.uses_legacy_single_scale_path(scale_factor) {
            return self.diff_path.clone();
        }
        self.scaled_artifact_path(scale_factor, "diff.png")
    }

    fn report_path_for_scale(&self, scale_factor: f32) -> PathBuf {
        if self.uses_legacy_single_scale_path(scale_factor) {
            return self.report_path.clone();
        }
        self.scaled_artifact_path(scale_factor, "report.txt")
    }

    fn scaled_artifact_path(&self, scale_factor: f32, suffix: &str) -> PathBuf {
        let manifest_dir = self
            .manifest_path
            .parent()
            .expect("loaded fixture manifests always have a parent");
        let stem = self
            .manifest_path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .expect("loaded fixture manifests always have UTF-8 stems");
        let artifact_stem = stem.strip_suffix(".fixture").unwrap_or(stem);
        manifest_dir.join(format!(
            "{artifact_stem}.scale-{}.{}",
            scale_suffix(scale_factor),
            suffix
        ))
    }

    fn uses_legacy_single_scale_path(&self, scale_factor: f32) -> bool {
        self.manifest.ui_scale_factors.len() == 1 && scale_is_one(scale_factor)
    }
}

pub fn run_fixture(manifest_path: impl AsRef<Path>) -> Result<Vec<VisualFixtureOutcome>> {
    VisualFixtureRun::load(manifest_path)?.run()
}

pub fn bless_fixture(manifest_path: impl AsRef<Path>) -> Result<()> {
    VisualFixtureRun::load(manifest_path)?.bless()
}

pub fn clean_fixture(manifest_path: impl AsRef<Path>) -> Result<()> {
    VisualFixtureRun::load(manifest_path)?.clean_artifacts()
}

fn remove_file_if_exists(path: &Path) -> Result<()> {
    match std::fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => {
            Err(error).with_context(|| format!("remove visual artifact {}", path.display()))
        }
    }
}

fn scale_is_one(scale_factor: f32) -> bool {
    (scale_factor - 1.0).abs() < f32::EPSILON
}

fn scale_suffix(scale_factor: f32) -> String {
    let rounded = (scale_factor * 100.0).round() / 100.0;
    format!("{rounded:.2}").replace('.', "_")
}

fn load_state_for_manifest(manifest: &FixtureManifest) -> Result<ReviewWorkspaceState> {
    let project_root = repo_root().join(&manifest.project_path);
    let board_file = manifest
        .board_file
        .as_ref()
        .map(|path| repo_root().join(path));
    let request = LiveReviewRequest {
        project_root,
        board_file,
        artifact_path: None,
        net_uuid: None,
        from_anchor_pad_uuid: None,
        to_anchor_pad_uuid: None,
        profile: None,
    };
    datum_gui_protocol::load_board_editor_workspace_state(&request)
        .with_context(|| format!("load fixture project {}", request.project_root.display()))
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("gui-render crate should live under <repo>/crates/gui-render")
        .to_path_buf()
}

fn camera_for_manifest(manifest: &FixtureManifest, state: &ReviewWorkspaceState) -> CameraState {
    let viewport = manifest.viewport;
    let layout = ShellLayout::for_window(viewport.width_px, viewport.height_px, None);
    let scene_viewport = layout.scene_viewport();
    let board_field_width = (scene_viewport.width - 20.0).max(1.0);
    let board_field_height = (scene_viewport.height - 20.0).max(1.0);
    let scene_width = (state.scene.bounds.max_x - state.scene.bounds.min_x).max(1) as f32;
    let scene_height = (state.scene.bounds.max_y - state.scene.bounds.min_y).max(1) as f32;
    let fit_scale = (board_field_width / scene_width)
        .min(board_field_height / scene_height)
        .max(0.000_001);
    let desired_scale = (1.0 / ((viewport.zoom_mm_per_px as f32) * NM_PER_MM)).max(0.000_001);

    CameraState {
        center_x_nm: (viewport.center_mm[0] as f32) * NM_PER_MM,
        center_y_nm: (viewport.center_mm[1] as f32) * NM_PER_MM,
        zoom: desired_scale / fit_scale,
    }
}

fn diff_policy_for_manifest(manifest: &FixtureManifest) -> Result<DiffPolicy> {
    match manifest.golden.diff_policy.as_str() {
        "exact" => Ok(DiffPolicy::Exact),
        "tolerance" => Ok(DiffPolicy::Tolerance {
            per_pixel: manifest.golden.diff_tolerance_per_pixel,
            total_px_pct: manifest.golden.diff_tolerance_total_px_pct,
        }),
        other => bail!("visual runner does not implement diff policy {other:?} yet"),
    }
}

fn assert_non_blank(image: &RgbaImage, minimum_pct: f64) -> Result<()> {
    if minimum_pct <= 0.0 {
        return Ok(());
    }
    let Some(background) = image.pixels().next().copied() else {
        bail!("visual capture is empty");
    };
    let non_background = image.pixels().filter(|pixel| **pixel != background).count();
    let total = u64::from(image.width()) * u64::from(image.height());
    let pct = if total == 0 {
        0.0
    } else {
        (non_background as f64 / total as f64) * 100.0
    };
    if pct < minimum_pct {
        bail!(
            "visual capture blank check failed: {:.6}% non-background, expected at least {:.6}%",
            pct,
            minimum_pct
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture_path(name: &str) -> PathBuf {
        repo_root()
            .join("crates/gui-render/testdata/golden/board")
            .join(format!("{name}.fixture.toml"))
    }

    #[test]
    fn fixture_run_resolves_artifact_paths() {
        let run = VisualFixtureRun::load(fixture_path("text-density-repro"))
            .expect("fixture run should load");

        assert_eq!(run.manifest.name, "text-density-repro");
        assert!(run.golden_path.ends_with("text-density-repro.golden.png"));
        assert!(run.actual_path.ends_with("text-density-repro.actual.png"));
        assert!(run.diff_path.ends_with("text-density-repro.diff.png"));
        assert!(run.report_path.ends_with("text-density-repro.report.txt"));
    }

    #[test]
    fn scaled_artifacts_use_scale_suffixes() {
        let mut run = VisualFixtureRun::load(fixture_path("text-density-repro"))
            .expect("fixture run should load");
        run.manifest.ui_scale_factors = vec![1.0, 1.25, 1.5, 2.0];

        assert!(
            run.golden_path_for_scale(1.0)
                .ends_with("text-density-repro.scale-1_00.golden.png")
        );
        assert!(
            run.golden_path_for_scale(1.25)
                .ends_with("text-density-repro.scale-1_25.golden.png")
        );
        assert!(
            run.report_path_for_scale(2.0)
                .ends_with("text-density-repro.scale-2_00.report.txt")
        );
    }

    #[test]
    fn manifest_camera_uses_semantic_mm_per_pixel_scale() {
        let manifest = FixtureManifest::load(fixture_path("text-density-repro"))
            .expect("fixture manifest should load");
        let state = load_state_for_manifest(&manifest).expect("fixture project should load");
        let camera = camera_for_manifest(&manifest, &state);

        assert_eq!(camera.center_x_nm, 105_000_000.0);
        assert_eq!(camera.center_y_nm, 75_000_000.0);
        assert!(camera.zoom > 0.0);
    }

    #[test]
    fn blank_check_rejects_solid_image() {
        let image = RgbaImage::from_pixel(4, 4, image::Rgba([1, 2, 3, 255]));
        assert!(assert_non_blank(&image, 1.0).is_err());
    }

    #[test]
    fn clean_artifacts_removes_only_generated_outputs() {
        let (dir, run) = temp_fixture_run("datum-visual-clean-test");
        std::fs::write(&run.actual_path, b"actual").expect("write actual artifact");
        std::fs::write(&run.diff_path, b"diff").expect("write diff artifact");
        std::fs::write(&run.report_path, b"report").expect("write report artifact");
        std::fs::write(&run.golden_path, b"golden").expect("write golden");

        run.clean_artifacts().expect("clean generated artifacts");

        assert!(!run.actual_path.exists());
        assert!(!run.diff_path.exists());
        assert!(!run.report_path.exists());
        assert!(run.golden_path.exists());
        std::fs::remove_dir_all(&dir).expect("remove temp visual clean dir");
    }

    #[test]
    fn failed_compare_retains_debug_artifacts() {
        let (dir, run) = temp_fixture_run("datum-visual-failure-artifacts-test");
        let expected = image::RgbaImage::from_pixel(2, 2, image::Rgba([1, 2, 3, 255]));
        expected.save(&run.golden_path).expect("write golden");
        let actual = image::RgbaImage::from_pixel(2, 2, image::Rgba([9, 2, 3, 255]));

        let error = run
            .compare_actual_at_scale(actual, 1.0)
            .expect_err("different images should fail exact compare");

        assert!(error.to_string().contains("visual fixture fixture scale 1"));
        assert!(run.actual_path.exists());
        assert!(run.diff_path.exists());
        assert!(run.report_path.exists());
        assert!(run.golden_path.exists());
        std::fs::remove_dir_all(&dir).expect("remove temp visual failure dir");
    }

    fn temp_fixture_run(prefix: &str) -> (PathBuf, VisualFixtureRun) {
        let unique = format!("{prefix}-{}", uuid::Uuid::new_v4());
        let dir = std::env::temp_dir().join(unique);
        std::fs::create_dir_all(&dir).expect("create temp visual fixture dir");
        let manifest_path = dir.join("fixture.fixture.toml");
        std::fs::write(
            &manifest_path,
            r#"
[fixture]
name = "fixture"
lane = "A"
suite = "board-text"
fixture_format_version = 1

[input]
project_path = "crates/engine/testdata/golden/text/native/text-density-repro"
project_kind = "datum-native"

[viewport]
width_px = 64
height_px = 64
center_mm = [0.0, 0.0]
zoom_mm_per_px = 1.0

[golden]
filename = "fixture.golden.png"
diff_policy = "exact"
diff_tolerance_per_pixel = 0
diff_tolerance_total_px_pct = 0.0
ssim_threshold = 0.0
mask_filename = ""

[blank_check]
expect_non_blank_pct = 1.0
"#,
        )
        .expect("write temp visual manifest");
        let run = VisualFixtureRun::load(&manifest_path).expect("load temp visual manifest");
        (dir, run)
    }

    #[test]
    #[ignore = "requires a working local wgpu adapter and checked-in or generated visual goldens"]
    fn text_fixture_visual_goldens_match() {
        for name in [
            "text-intent-repro",
            "text-fidelity-repro",
            "text-transform-repro",
            "text-density-repro",
        ] {
            run_fixture(fixture_path(name)).unwrap_or_else(|error| {
                panic!("visual fixture {name} should match golden: {error:#}");
            });
        }
    }
}
