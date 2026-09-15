use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use eda_engine::ir::serialization::to_json_deterministic;
use serde::Serialize;

/// Test harness — golden test runners, corpus loaders, benchmark utilities.
/// Keeps reusable testing support out of engine code.
/// See docs/TEST_STRATEGY.md.
/// Serialize a value using the canonical deterministic JSON contract.
pub fn canonical_json<T: Serialize>(value: &T) -> Result<String> {
    to_json_deterministic(value).context("failed to serialize canonical JSON")
}

/// Assert that repeated canonical serialization produces byte-identical output.
pub fn assert_repeated_canonical<T: Serialize>(value: &T, repeats: usize) -> Result<String> {
    if repeats == 0 {
        bail!("repeats must be >= 1");
    }

    let first = canonical_json(value)?;
    for _ in 1..repeats {
        let next = canonical_json(value)?;
        if next != first {
            bail!("canonical serialization is not stable across repeated runs");
        }
    }
    Ok(first)
}

/// Read a UTF-8 golden file.
pub fn read_golden(path: impl AsRef<Path>) -> Result<String> {
    let path = path.as_ref();
    fs::read_to_string(path)
        .with_context(|| format!("failed to read golden file {}", path.display()))
}

/// Write a UTF-8 golden file, creating parent directories if needed.
pub fn write_golden(path: impl AsRef<Path>, contents: &str) -> Result<()> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create golden directory {}", parent.display()))?;
    }
    fs::write(path, contents)
        .with_context(|| format!("failed to write golden file {}", path.display()))
}

/// Compare a value against a golden file using canonical JSON.
pub fn compare_json_golden<T: Serialize>(path: impl AsRef<Path>, value: &T) -> Result<()> {
    let path = path.as_ref();
    let expected = read_golden(path)?;
    let actual = canonical_json(value)?;
    if expected != actual {
        bail!(
            "golden mismatch for {}:\nexpected: {}\nactual: {}",
            path.display(),
            expected,
            actual
        );
    }
    Ok(())
}

/// Build a path under a conventional testdata root.
pub fn testdata_path(relative: impl AsRef<Path>) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("testdata")
        .join(relative.as_ref())
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use serde::Serialize;

    use super::*;

    #[derive(Serialize)]
    struct Sample {
        map: HashMap<String, i32>,
    }

    fn sample_value() -> Sample {
        let mut map = HashMap::new();
        map.insert("z".into(), 1);
        map.insert("a".into(), 2);
        Sample { map }
    }

    #[test]
    fn repeated_canonical_is_stable() {
        let json = assert_repeated_canonical(&sample_value(), 3).unwrap();
        assert_eq!(json, r#"{"map":{"a":2,"z":1}}"#);
    }

    #[test]
    fn golden_round_trip_helpers_work() {
        let dir = std::env::temp_dir().join(format!("eda-test-harness-{}", std::process::id()));
        let path = dir.join("sample.json");

        write_golden(&path, r#"{"map":{"a":2,"z":1}}"#).unwrap();
        compare_json_golden(&path, &sample_value()).unwrap();

        let _ = fs::remove_file(&path);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn testdata_path_helper_scopes_to_crate() {
        let path = testdata_path("golden/example.json");
        assert!(path.ends_with("testdata/golden/example.json"));
    }
}
