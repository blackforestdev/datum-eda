//! One caller-owned read snapshot. Reuse proves identical complete local input
//! bytes; metadata supplements byte comparison and never replaces validation.
use super::{DesignModel, EngineError, ProjectResolver};
use std::io::Read;
use std::path::{Component, Path, PathBuf};
use std::sync::Arc;

#[derive(Default)]
pub struct ProjectReadCache {
    entry: Option<Entry>,
}

struct Entry {
    root: PathBuf,
    input: Vec<u8>,
    model: Arc<DesignModel>,
}

impl ProjectReadCache {
    /// Reads remain authoritative through ProjectResolver. Unsupported trees,
    /// transient diagnostics and pending recovery bypass reuse, not validation. The cache
    /// retains at most one model and has no global project registry.
    pub fn resolve(&mut self, root: &Path) -> Result<Arc<DesignModel>, EngineError> {
        if let Some(entry) = &self.entry
            && entry.root == root
            && unchanged(root, &entry.input)
        {
            return Ok(entry.model.clone());
        }
        // Release the old snapshot before potentially expensive reconstruction.
        self.entry = None;
        let before = capture(root);
        let model = Arc::new(ProjectResolver::new(root).resolve()?);
        if let Some(before) = before
            // These join diagnostics derive only from the compared model inputs.
            // Preserve them verbatim; all IO, integrity and unknown diagnostics
            // still force full resolution on every read.
            && model.diagnostics.iter().all(|diagnostic| {
                matches!(
                    diagnostic.code.as_str(),
                    "component_instance_unmatched_package"
                        | "component_instance_unmatched_symbol"
                        | "component_instance_ambiguous_join"
                )
            })
            && model
                .source_shards
                .iter()
                .all(|shard| shard.path.strip_prefix(root).is_ok_and(local_relative))
            && unchanged(root, &before)
        {
            self.entry = Some(Entry {
                root: root.to_owned(),
                input: before,
                model: model.clone(),
            });
        }
        Ok(model)
    }
}

fn local_relative(path: &Path) -> bool {
    !path.as_os_str().is_empty()
        && path
            .components()
            .all(|part| matches!(part, Component::Normal(_)))
}

// These bound optional input scanning/retention, not project admission or resource
// acceptance. Exceeding either bound falls back to the ordinary full resolver.
const MAX_ENTRIES: usize = 4096;
const MAX_BYTES: u64 = 32 * 1024 * 1024;
const MAX_RETAINED_INPUT: usize = 8 * 1024 * 1024;

fn capture(root: &Path) -> Option<Vec<u8>> {
    let mut input = Input::Capture(Vec::new());
    scan(root, &mut input)?;
    let Input::Capture(bytes) = input else {
        unreachable!()
    };
    Some(bytes)
}

fn unchanged(root: &Path, expected: &[u8]) -> bool {
    let mut input = Input::Compare {
        expected,
        position: 0,
    };
    scan(root, &mut input).is_some()
        && matches!(input, Input::Compare { position, .. } if position == expected.len())
}

// Capture and warm comparison use exactly the same encoding. Warm reads retain
// no second input copy and compare every byte, without probabilistic hash reuse.
enum Input<'a> {
    Capture(Vec<u8>),
    Compare { expected: &'a [u8], position: usize },
}

impl Input<'_> {
    fn append(&mut self, bytes: &[u8]) -> Option<()> {
        match self {
            Self::Capture(captured) => {
                let end = captured.len().checked_add(bytes.len())?;
                if end > MAX_RETAINED_INPUT {
                    return None;
                }
                if end > captured.capacity() {
                    let capacity = end.checked_next_power_of_two()?.min(MAX_RETAINED_INPUT);
                    captured.try_reserve_exact(capacity - captured.len()).ok()?;
                    if captured.capacity() > MAX_RETAINED_INPUT {
                        return None;
                    }
                }
                captured.extend_from_slice(bytes);
            }
            Self::Compare { expected, position } => {
                let end = position.checked_add(bytes.len())?;
                if expected.get(*position..end)? != bytes {
                    return None;
                }
                *position = end;
            }
        }
        Some(())
    }
}

fn scan(root: &Path, input: &mut Input<'_>) -> Option<()> {
    // A symlink target or a recovery lease can change without local file bytes
    // changing. Do not memoize either case, including symlinked ancestors.
    for ancestor in root.ancestors().filter(|p| !p.as_os_str().is_empty()) {
        if std::fs::symlink_metadata(ancestor)
            .ok()?
            .file_type()
            .is_symlink()
        {
            return None;
        }
    }
    if root.join(".datum/revision/write.lock").exists() {
        return None;
    }
    let stages = root.join(".datum/revision/v1/stage");
    match std::fs::read_dir(&stages) {
        Ok(mut entries) => {
            if entries.next().is_some() {
                return None;
            }
        }
        Err(error) if error.kind() != std::io::ErrorKind::NotFound => return None,
        _ => {}
    }
    // Declared external/missing roots must not become invisible dependencies.
    // Other discovered source paths are checked again on the resolved model.
    let manifest_path = root.join("project.json");
    let metadata = std::fs::symlink_metadata(&manifest_path).ok()?;
    if !metadata.is_file() || metadata.len() > MAX_BYTES {
        return None;
    }
    let mut manifest_bytes = Vec::new();
    std::fs::File::open(manifest_path)
        .ok()?
        .take(MAX_BYTES + 1)
        .read_to_end(&mut manifest_bytes)
        .ok()?;
    if manifest_bytes.len() as u64 > MAX_BYTES {
        return None;
    }
    let manifest: serde_json::Value = serde_json::from_slice(&manifest_bytes).ok()?;
    for key in ["board", "schematic", "rules"] {
        if !local_relative(Path::new(manifest.get(key)?.as_str()?)) {
            return None;
        }
    }
    if let Some(pools) = manifest.get("pools") {
        for pool in pools.as_array()? {
            if !local_relative(Path::new(pool.get("path")?.as_str()?)) {
                return None;
            }
        }
    }
    let mut scan = Scan {
        input,
        entries: 0,
        bytes: 0,
    };
    scan.metadata(&std::fs::symlink_metadata(root).ok()?)?;
    scan.directory(root, root, 0)?;
    Some(())
}

struct Scan<'a, 'b> {
    input: &'a mut Input<'b>,
    entries: usize,
    bytes: u64,
}

impl Scan<'_, '_> {
    // File identity/change time also detects a rewrite-and-restore between the
    // scans around resolution. Content comparison remains required on every hit;
    // timestamps alone never establish freshness. Unsupported platforms bypass.
    fn metadata(&mut self, metadata: &std::fs::Metadata) -> Option<()> {
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            self.input.append(&metadata.dev().to_le_bytes())?;
            self.input.append(&metadata.ino().to_le_bytes())?;
            self.input.append(&metadata.ctime().to_le_bytes())?;
            self.input.append(&metadata.ctime_nsec().to_le_bytes())?;
            self.input.append(&metadata.mode().to_le_bytes())?;
            Some(())
        }
        #[cfg(not(unix))]
        {
            let _ = metadata;
            None
        }
    }

    fn directory(&mut self, root: &Path, path: &Path, depth: usize) -> Option<()> {
        if depth > 32 {
            return None;
        }
        let mut entries = Vec::new();
        for entry in std::fs::read_dir(path).ok()? {
            self.entries += 1;
            if self.entries > MAX_ENTRIES {
                return None;
            }
            entries.push(entry.ok()?.path());
        }
        entries.sort();
        for path in entries {
            let metadata = std::fs::symlink_metadata(&path).ok()?;
            self.metadata(&metadata)?;
            let relative = path.strip_prefix(root).ok()?.as_os_str().as_encoded_bytes();
            self.input.append(&(relative.len() as u64).to_le_bytes())?;
            self.input.append(relative)?;
            if metadata.is_dir() {
                self.input.append(b"directory")?;
                self.directory(root, &path, depth + 1)?;
            } else if metadata.is_file() {
                self.input.append(b"file")?;
                self.input.append(&metadata.len().to_le_bytes())?;
                self.bytes = self.bytes.checked_add(metadata.len())?;
                if self.bytes > MAX_BYTES {
                    return None;
                }
                let mut file = std::fs::File::open(&path).ok()?;
                let mut buffer = [0; 16 * 1024];
                let mut read = 0u64;
                loop {
                    let n = file.read(&mut buffer).ok()?;
                    if n == 0 {
                        break;
                    }
                    read = read.checked_add(n as u64)?;
                    if read > metadata.len() {
                        return None;
                    }
                    self.input.append(&buffer[..n])?;
                }
                if read != metadata.len() {
                    return None;
                }
            } else {
                // Symlinks, sockets, devices and pipes are not stable local input.
                return None;
            }
        }
        Some(())
    }
}
