//! Resolve the GUI's CLI companion without silently using an older workspace build.
use anyhow::{Context, Result, bail};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

pub(super) fn cli_prefix() -> Result<Vec<String>> {
    if let Ok(configured) = std::env::var("EDA_CLI_BIN") {
        let parts: Vec<_> = configured
            .split_whitespace()
            .map(ToOwned::to_owned)
            .collect();
        if !parts.is_empty() {
            return Ok(parts);
        }
    }
    static COMPANION: OnceLock<Result<String, String>> = OnceLock::new();
    match COMPANION.get_or_init(|| resolve_companion().map_err(|e| format!("{e:#}"))) {
        Ok(binary) => Ok(vec![binary.clone()]),
        Err(error) => bail!("{error}"),
    }
}

fn resolve_companion() -> Result<String> {
    resolve_binary_companion("datum-eda", "datum-eda-cli")
}

pub(super) fn resolve_daemon_companion() -> Result<PathBuf> {
    resolve_binary_companion("eda-engine-daemon", "eda-engine-daemon").map(PathBuf::from)
}

fn resolve_binary_companion(name: &str, package: &str) -> Result<String> {
    let exe = std::env::current_exe().context("locate GUI executable")?;
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .ok();
    let profile_dir =
        executable_profile_directory(&exe).context("locate CLI companion directory")?;
    let binary = profile_dir.join(name);
    // Workspace launches may rebuild only the GUI. Cargo must establish the
    // companion's source freshness, rather than inferring compatibility from
    // the mere existence or modification time of a sibling executable.
    if let Some(root) = root.as_ref().filter(|root| {
        root.join("scripts/run_cargo_guarded.py").is_file()
            && exe.starts_with(root)
            && !companion_inputs_are_current(&binary, root)
    }) {
        let target = profile_dir
            .parent()
            .context("locate workspace target directory")?;
        let profile = match profile_dir.file_name().and_then(|s| s.to_str()) {
            Some("debug") => "dev",
            Some(name) => name,
            None => bail!("invalid workspace build profile"),
        };
        let output = std::process::Command::new("python3")
            .arg(root.join("scripts/run_cargo_guarded.py"))
            .args(["--workload", "interactive", "--target-dir"])
            .arg(target)
            .args([
                "--",
                "cargo",
                "build",
                "--quiet",
                "-p",
                package,
                "--bin",
                name,
                "--profile",
                profile,
            ])
            .current_dir(root)
            .output()
            .context("build matching workspace CLI companion")?;
        if !output.status.success() {
            bail!(
                "matching workspace CLI build failed: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            );
        }
    }
    if !binary.is_file() {
        bail!("Datum CLI companion is missing: {}", binary.display());
    }
    Ok(binary.to_string_lossy().into_owned())
}

// Cargo's dependency file names the compiler's complete source inputs. Missing,
// unreadable or escaped paths conservatively require Cargo to check the build.
fn companion_inputs_are_current(binary: &Path, root: &Path) -> bool {
    let Some(built) = std::fs::metadata(binary).and_then(|m| m.modified()).ok() else {
        return false;
    };
    let Ok(dependencies) = std::fs::read_to_string(binary.with_extension("d")) else {
        return false;
    };
    let Some((_, inputs)) = dependencies
        .lines()
        .next()
        .and_then(|line| line.split_once(": "))
    else {
        return false;
    };
    if inputs.contains('\\') || inputs.trim().is_empty() {
        return false;
    }
    let mut paths: std::collections::BTreeSet<PathBuf> =
        inputs.split_whitespace().map(PathBuf::from).collect();
    for path in paths.clone() {
        for parent in path.ancestors().skip(1).take_while(|p| p.starts_with(root)) {
            let manifest = parent.join("Cargo.toml");
            if manifest.is_file() {
                paths.insert(manifest);
            }
        }
    }
    paths.insert(root.join("Cargo.lock"));
    paths.iter().all(|p| {
        std::fs::metadata(p)
            .and_then(|m| m.modified())
            .is_ok_and(|changed| changed <= built)
    })
}

fn executable_profile_directory(exe: &Path) -> Option<PathBuf> {
    let directory = exe.parent()?;
    Some(
        if directory.file_name()? == "deps" {
            directory.parent()?
        } else {
            directory
        }
        .to_owned(),
    )
}

#[cfg(test)]
pub(super) fn resolve_workspace_eda_binary() -> Option<String> {
    let exe = std::env::current_exe().ok()?;
    let exe_dir = exe.parent()?;

    let direct = exe_dir.join("datum-eda");
    if direct.is_file() {
        return Some(direct.to_string_lossy().into_owned());
    }

    let deps_sibling = exe_dir.parent()?.join("datum-eda");
    if deps_sibling.is_file() {
        return Some(deps_sibling.to_string_lossy().into_owned());
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn companion_build_uses_the_running_executables_target_and_profile() {
        assert_eq!(
            executable_profile_directory(Path::new("/repo/target/debug/datum-gui")),
            Some(PathBuf::from("/repo/target/debug"))
        );
        assert_eq!(
            executable_profile_directory(Path::new(
                "/repo/target/proof/release/deps/protocol-test"
            )),
            Some(PathBuf::from("/repo/target/proof/release"))
        );
    }
    #[test]
    fn stale_companion_requires_cargo_and_fresh_inputs_do_not() {
        let root =
            std::env::temp_dir().join(format!("datum-cli-freshness-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir(&root).unwrap();
        let binary = root.join("datum-eda");
        let input = root.join("engine.rs");
        for path in [
            &binary,
            &input,
            &root.join("Cargo.toml"),
            &root.join("Cargo.lock"),
        ] {
            std::fs::write(path, "source").unwrap();
            std::fs::File::options()
                .write(true)
                .open(path)
                .unwrap()
                .set_modified(std::time::UNIX_EPOCH + std::time::Duration::from_secs(10))
                .unwrap();
        }
        std::fs::write(
            binary.with_extension("d"),
            format!("{}: {}\n", binary.display(), input.display()),
        )
        .unwrap();
        assert!(companion_inputs_are_current(&binary, &root));
        std::fs::File::options()
            .write(true)
            .open(&input)
            .unwrap()
            .set_modified(std::time::UNIX_EPOCH + std::time::Duration::from_secs(11))
            .unwrap();
        assert!(!companion_inputs_are_current(&binary, &root));
        std::fs::remove_file(&input).unwrap();
        assert!(!companion_inputs_are_current(&binary, &root));
        std::fs::remove_dir_all(root).unwrap();
    }
}
