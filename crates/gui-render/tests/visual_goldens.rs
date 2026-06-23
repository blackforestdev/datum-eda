#![cfg(feature = "visual")]

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use datum_gui_render::visual_runner::{
    bless_workspace_golden, render_workspace_image, run_fixture, run_workspace_golden,
};

// Supervision-reflection panel golden viewport. The dock must be tall enough to
// show the banner plus the five-source provenance ledger; a fixed deterministic
// size keeps the golden byte-stable.
const SUPERVISION_GOLDEN_WIDTH: u32 = 1100;
const SUPERVISION_GOLDEN_HEIGHT: u32 = 720;

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

fn supervision_golden_path(name: &str) -> PathBuf {
    repo_root()
        .join("crates/gui-render/testdata/golden/supervision")
        .join(format!("{name}.golden.png"))
}

/// PS-SR-4 / PS-SR-6 visual goldens for the READ-ONLY supervision-reflection
/// panel. Renders the R12 journal ledger (one row per `CommitSource`, with the
/// §4.7 human-vs-agent accents) over a clean resolver banner, and the R13
/// recovery layout (diagnostics primary, journal suppressed). Each golden is
/// rendered through the production renderer; determinism is asserted by rendering
/// twice and requiring byte-identical output, and the checked-in PNG is matched
/// under the exact diff policy.
///
/// Set `DATUM_BLESS_SUPERVISION_GOLDENS=1` to (re)write the checked-in PNGs.
#[test]
#[ignore = "requires local wgpu rendering authority; run explicitly until visual CI is pinned"]
fn supervision_panel_visual_goldens_match() -> Result<()> {
    let cases: &[(&str, fn() -> datum_gui_protocol::ReviewWorkspaceState)] = &[
        (
            "supervision_activity_provenance",
            datum_gui_protocol::supervision_fixture_workspace_state_resolved,
        ),
        (
            "supervision_resolver_recovery",
            datum_gui_protocol::supervision_fixture_workspace_state_recovery,
        ),
    ];

    let bless = std::env::var("DATUM_BLESS_SUPERVISION_GOLDENS").is_ok();
    for (name, build_state) in cases {
        let state = build_state();
        let golden = supervision_golden_path(name);

        // Determinism gate: the production renderer must produce byte-identical
        // pixels across two independent renders of the same read-only state.
        let first = render_workspace_image(
            &state,
            SUPERVISION_GOLDEN_WIDTH,
            SUPERVISION_GOLDEN_HEIGHT,
        )
        .with_context(|| format!("render supervision golden {name} (pass 1)"))?;
        let second = render_workspace_image(
            &state,
            SUPERVISION_GOLDEN_WIDTH,
            SUPERVISION_GOLDEN_HEIGHT,
        )
        .with_context(|| format!("render supervision golden {name} (pass 2)"))?;
        assert_eq!(
            first.as_raw(),
            second.as_raw(),
            "supervision golden {name} render is non-deterministic"
        );

        if bless {
            bless_workspace_golden(
                &state,
                SUPERVISION_GOLDEN_WIDTH,
                SUPERVISION_GOLDEN_HEIGHT,
                &golden,
            )
            .with_context(|| format!("bless supervision golden {name}"))?;
            continue;
        }

        let result = run_workspace_golden(
            &state,
            SUPERVISION_GOLDEN_WIDTH,
            SUPERVISION_GOLDEN_HEIGHT,
            &golden,
        )
        .with_context(|| format!("match supervision golden {name}"))?;
        assert_eq!(
            result.differing_pixels, 0,
            "supervision golden {name} must match exactly"
        );
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
