use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use datum_gui_render::visual_runner::{bless_fixture, clean_fixture, run_fixture};

const BOARD_FIXTURE_DIR: &str = "crates/gui-render/testdata/golden/board";

fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let Some(command) = args.next() else {
        print_usage();
        bail!("missing command");
    };

    match command.as_str() {
        "check" => {
            let manifest = required_manifest_arg(args.next())?;
            let outcome = run_fixture(&manifest)?;
            println!(
                "VISUAL-CHECK: fixture={} passed=true differing_pixels={} differing_pct={:.6}",
                outcome.run.manifest.name,
                outcome.result.differing_pixels,
                outcome.result.differing_pct
            );
        }
        "bless" => {
            let manifest = required_manifest_arg(args.next())?;
            bless_fixture(&manifest)?;
            println!("VISUAL-BLESS: manifest={}", manifest.display());
        }
        "clean" => {
            let manifest = required_manifest_arg(args.next())?;
            clean_fixture(&manifest)?;
            println!("VISUAL-CLEAN: manifest={}", manifest.display());
        }
        "check-all" => {
            for manifest in fixture_manifests()? {
                let outcome = run_fixture(&manifest)?;
                println!(
                    "VISUAL-CHECK: fixture={} passed=true differing_pixels={} differing_pct={:.6}",
                    outcome.run.manifest.name,
                    outcome.result.differing_pixels,
                    outcome.result.differing_pct
                );
            }
        }
        "bless-all" => {
            for manifest in fixture_manifests()? {
                bless_fixture(&manifest)?;
                println!("VISUAL-BLESS: manifest={}", manifest.display());
            }
        }
        "clean-all" => {
            for manifest in fixture_manifests()? {
                clean_fixture(&manifest)?;
                println!("VISUAL-CLEAN: manifest={}", manifest.display());
            }
        }
        _ => {
            print_usage();
            bail!("unknown command {command:?}");
        }
    }
    Ok(())
}

fn required_manifest_arg(arg: Option<String>) -> Result<PathBuf> {
    let Some(path) = arg else {
        print_usage();
        bail!("missing fixture manifest path");
    };
    Ok(PathBuf::from(path))
}

fn fixture_manifests() -> Result<Vec<PathBuf>> {
    let root = repo_root().join(BOARD_FIXTURE_DIR);
    let mut manifests = Vec::new();
    for entry in std::fs::read_dir(&root)
        .with_context(|| format!("read visual fixture directory {}", root.display()))?
    {
        let entry = entry.with_context(|| format!("read entry under {}", root.display()))?;
        let path = entry.path();
        if path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.ends_with(".fixture.toml"))
        {
            manifests.push(path);
        }
    }
    manifests.sort();
    Ok(manifests)
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("gui-render crate should live under <repo>/crates/gui-render")
        .to_path_buf()
}

fn print_usage() {
    eprintln!(
        "usage:\n  datum-visual-fixture check <fixture.toml>\n  datum-visual-fixture bless <fixture.toml>\n  datum-visual-fixture clean <fixture.toml>\n  datum-visual-fixture check-all\n  datum-visual-fixture bless-all\n  datum-visual-fixture clean-all"
    );
}
