#![cfg(feature = "visual")]

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use datum_gui_render::visual_runner::run_fixture;

const FIXTURE_NAMES: &[&str] = &[
    "text-density-repro",
    "text-fidelity-repro",
    "text-intent-repro",
    "text-transform-repro",
];

#[test]
#[ignore = "requires local visual rendering authority; run explicitly until visual CI is pinned"]
fn board_text_visual_goldens_match() -> Result<()> {
    for fixture_name in FIXTURE_NAMES {
        let manifest = fixture_manifest_path(fixture_name);
        let outcome = run_fixture(&manifest)
            .with_context(|| format!("run visual fixture {}", manifest.display()))?;
        assert_eq!(
            outcome.result.differing_pixels, 0,
            "fixture {fixture_name} should have visual parity under its fixture diff policy"
        );
        assert_no_generated_artifacts(&manifest)?;
    }
    Ok(())
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
