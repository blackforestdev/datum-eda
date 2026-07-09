#![cfg(feature = "visual")]

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use datum_gui_render::design_artboards::check_design_artboards;
use datum_gui_render::visual_runner::{VisualFixtureRun, run_fixture};

const FIXTURE_NAMES: &[&str] = &[
    "datum-test",
    "text-density-repro",
    "text-fidelity-repro",
    "text-intent-repro",
    "text-transform-repro",
];

#[test]
#[ignore = "requires local visual rendering authority; run explicitly until visual CI is pinned"]
fn board_visual_goldens_match() -> Result<()> {
    for fixture_name in FIXTURE_NAMES {
        let manifest = fixture_manifest_path(fixture_name);
        let outcomes = run_fixture(&manifest)
            .with_context(|| format!("run visual fixture {}", manifest.display()))?;
        for outcome in outcomes {
            assert_eq!(
                outcome.result.differing_pixels, 0,
                "fixture {fixture_name} scale {} should have visual parity under its fixture diff policy",
                outcome.scale_factor
            );
        }
        assert_no_generated_artifacts(&manifest)?;
    }
    Ok(())
}

#[test]
#[ignore = "requires local visual rendering authority; exercises non-golden HiDPI scale rendering"]
fn board_multi_scale_visual_smoke_renders_nonblank() -> Result<()> {
    let scales = [1.0_f32, 1.25, 1.5, 2.0];
    let manifest = fixture_manifest_path("text-density-repro");
    let run = VisualFixtureRun::load(&manifest)
        .with_context(|| format!("load visual fixture {}", manifest.display()))?;

    for scale_factor in scales {
        let image = run
            .render_actual_at_scale(scale_factor)
            .with_context(|| format!("render scale {scale_factor}"))?;
        assert!(
            image.pixels().any(|pixel| pixel.0 != [0, 0, 0, 0]),
            "scale {scale_factor} render should not be blank"
        );
    }
    Ok(())
}

#[test]
#[ignore = "requires local visual rendering authority; checks Design Book artboard goldens"]
fn design_system_artboards_match() -> Result<()> {
    check_design_artboards()
}

fn fixture_manifest_path(fixture_name: &str) -> PathBuf {
    repo_root()
        .join("crates/gui-render/testdata/golden/board")
        .join(format!("{fixture_name}.fixture.toml"))
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("gui-render crate should live under <repo>/crates/gui-render")
        .to_path_buf()
}

fn assert_no_generated_artifacts(manifest: &Path) -> Result<()> {
    let dir = manifest
        .parent()
        .with_context(|| format!("fixture manifest has no parent: {}", manifest.display()))?;
    let stem = manifest
        .file_stem()
        .and_then(|stem| stem.to_str())
        .with_context(|| format!("fixture manifest has no UTF-8 stem: {}", manifest.display()))?;
    let artifact_stem = stem.strip_suffix(".fixture").unwrap_or(stem);
    for suffix in ["actual.png", "diff.png", "report.txt"] {
        let path = dir.join(format!("{artifact_stem}.{suffix}"));
        assert!(
            !path.exists(),
            "passing visual fixture left generated artifact {}",
            path.display()
        );
    }
    Ok(())
}
