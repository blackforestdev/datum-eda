//! Shared development-only paths. Cargo output never owns native fixture data.
use std::path::{Component, Path, PathBuf};

/// A family label selects new external scratch output; an absolute or target/
/// path selects a preserved legacy input through its verified relocation map.
pub fn path(value: impl AsRef<Path>) -> PathBuf {
    let value = value.as_ref();
    if value.is_absolute() || value.starts_with("target") {
        relocated(value)
    } else {
        scratch(value.to_str().expect("workspace family must be UTF-8"))
    }
}

fn root() -> PathBuf {
    let base = std::env::var_os("DATUM_DEVELOPMENT_WORKSPACE")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(
                std::env::var_os("HOME").expect("HOME or DATUM_DEVELOPMENT_WORKSPACE required"),
            )
            .join("Documents/datum-eda-workspaces")
        });
    let checkout = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap();
    checked_external(&base, &checkout).expect("development workspace must be external")
}

pub(crate) fn checked_external(path: &Path, checkout: &Path) -> Result<PathBuf, String> {
    if !path.is_absolute() || path.components().any(|p| matches!(p, Component::ParentDir)) {
        return Err("absolute path without parent traversal required".into());
    }
    let mut ancestor = path;
    let mut tail = Vec::new();
    while !ancestor.exists() {
        if ancestor.is_symlink() {
            return Err("dangling workspace symlink".into());
        }
        tail.push(ancestor.file_name().ok_or("missing workspace ancestor")?);
        ancestor = ancestor.parent().ok_or("missing workspace parent")?;
    }
    let mut resolved = ancestor.canonicalize().map_err(|e| e.to_string())?;
    for part in tail.into_iter().rev() {
        resolved.push(part);
    }
    if resolved.starts_with(checkout) {
        return Err("workspace resolves inside checkout".into());
    }
    Ok(resolved)
}

/// New disposable test/capture output, separated from preserved replay inputs.
fn scratch(family: &str) -> PathBuf {
    let checkout = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap();
    scratch_at(&root(), family, &checkout).expect("invalid external scratch path")
}

pub(crate) fn scratch_at(base: &Path, family: &str, checkout: &Path) -> Result<PathBuf, String> {
    if family.is_empty()
        || !family
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
    {
        return Err("invalid scratch family".into());
    }
    let joined = base.join("scratch").join(family);
    let resolved = checked_external(&joined, checkout)?;
    if resolved != joined {
        return Err("scratch path contains a symlink alias".into());
    }
    Ok(resolved)
}

/// Resolve an old checkout path through the local, verified relocation map.
/// The original JSON/setup bytes and historical source pins remain unchanged.
fn relocated(path: impl AsRef<Path>) -> PathBuf {
    let checkout = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap();
    let map = std::env::var_os("DATUM_FIXTURE_RELOCATION_MAP")
        .map(PathBuf::from)
        .unwrap_or_else(|| checkout.join(".datum/history-cleanup/target-relocation-map.json"));
    let bytes = std::fs::read(&map)
        .unwrap_or_else(|e| panic!("fixture relocation map {} required: {e}", map.display()));
    let value: serde_json::Value =
        serde_json::from_slice(&bytes).expect("valid fixture relocation map");
    resolve(path.as_ref(), &checkout, &value).unwrap_or_else(|e| panic!("{e}"))
}

pub(crate) fn resolve(
    path: &Path,
    checkout: &Path,
    value: &serde_json::Value,
) -> Result<PathBuf, String> {
    if path.components().any(|p| matches!(p, Component::ParentDir)) {
        return Err("parent traversal in fixture path".into());
    }
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        checkout.join(path)
    };
    let relative = absolute
        .strip_prefix(checkout)
        .map_err(|_| "legacy fixture path outside original checkout")?;
    if value["schema_version"] != 1 {
        return Err("unsupported fixture relocation map".into());
    }
    for row in value["collections"]
        .as_array()
        .ok_or("missing relocation collections")?
    {
        let source = row["source_relative"]
            .as_str()
            .ok_or("missing source path")?;
        let source = Path::new(source);
        if !source.starts_with("target")
            || source.components().count() != 2
            || source
                .components()
                .any(|p| !matches!(p, Component::Normal(_)))
        {
            return Err("invalid relocation source collection".into());
        }
        if let Ok(suffix) = relative.strip_prefix(source) {
            if row["disposition"] == "retired-released-test-output" {
                return Err(
                    "released temporary output retired; regenerate with current test".into(),
                );
            }
            if row["state"] != "preserved" {
                return Err("fixture relocation is incomplete".into());
            }
            let destination = Path::new(row["destination"].as_str().ok_or("missing destination")?);
            let destination = checked_external(destination, checkout)?;
            return checked_external(&destination.join(suffix), checkout);
        }
    }
    Err(format!("no preserved relocation for {}", path.display()))
}
