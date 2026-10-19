//! Read-only candidate classification. Never resolves/recoveries engine data.
use super::*;
use std::{
    ffi::CString,
    fs::File,
    os::{
        fd::{AsRawFd, FromRawFd},
        unix::{ffi::OsStrExt, fs::MetadataExt},
    },
    path::{Component, Path},
};

struct Stamp {
    path: PathBuf,
    identity: Identity,
    change: (i64, i64),
    pin: Option<File>,
    entries: Option<Vec<(std::ffi::OsString, Identity)>>,
    contents: Option<Vec<u8>>,
}
impl Stamp {
    fn read(path: &Path) -> Result<Self, String> {
        let metadata = fs::symlink_metadata(path)
            .map_err(|e| format!("Cannot inspect {}: {e}", path.display()))?;
        Ok(Self {
            path: path.into(),
            identity: Identity::of(&metadata),
            change: (metadata.ctime(), metadata.ctime_nsec()),
            pin: None,
            entries: None,
            contents: None,
        })
    }
    fn directory(parent: &File, name: &std::ffi::OsStr) -> Result<Self, String> {
        let leaf = CString::new(name.as_bytes()).map_err(|_| "Invalid folder entry name.")?;
        let descriptor = unsafe {
            libc::openat(
                parent.as_raw_fd(),
                leaf.as_ptr(),
                libc::O_RDONLY | libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC,
            )
        };
        if descriptor < 0 {
            return Err(format!(
                "Cannot pin inspected folder: {}",
                std::io::Error::last_os_error()
            ));
        }
        let pin = unsafe { File::from_raw_fd(descriptor) };
        let metadata = pin.metadata().map_err(|e| e.to_string())?;
        Ok(Self {
            path: PathBuf::from(format!("/proc/self/fd/{}", parent.as_raw_fd())).join(name),
            identity: Identity::of(&metadata),
            change: (metadata.ctime(), metadata.ctime_nsec()),
            pin: Some(pin),
            entries: None,
            contents: None,
        })
    }
    fn recheck(&self) -> Result<(), String> {
        let current = Self::read(&self.path)?;
        if let Some(contents) = &self.contents {
            if fs::read(&self.path).map_err(|e| e.to_string())? != *contents {
                return Err("Project classification changed during inspection.".into());
            }
        }
        if let Some(pin) = &self.pin {
            if let Some(entries) = &self.entries {
                if &directory_entries(pin)? != entries {
                    return Err(
                        "Directory entries changed during safety inspection. Re-read Location."
                            .into(),
                    );
                }
            }
            let pinned = pin.metadata().map_err(|e| e.to_string())?;
            if Identity::of(&pinned) != self.identity
                || (pinned.ctime(), pinned.ctime_nsec()) != self.change
            {
                return Err("An inspected directory changed.".into());
            }
        }
        if self.identity != current.identity || self.change != current.change {
            return Err(
                "Filesystem entries changed during safety inspection. Re-read Location.".into(),
            );
        }
        Ok(())
    }
}

pub(super) struct Inspection {
    stamps: Vec<Stamp>,
}
impl Inspection {
    pub fn recheck(&self) -> Result<(), String> {
        for stamp in &self.stamps {
            stamp.recheck()?;
        }
        Ok(())
    }
}

pub(super) fn inspect(
    parent_handle: &File,
    parent: &Path,
    command: &Command,
    protection: &Protection,
) -> Result<Inspection, String> {
    if parent
        .components()
        .any(|c| matches!(c, Component::Normal(p) if p == ".datum"))
    {
        return Err("Datum backing storage is protected from host naming actions.".into());
    }
    let selected = match command {
        Command::CreateDirectory { leaf } => parent.join(leaf),
        Command::Rename { old_leaf, .. } => parent.join(old_leaf),
    };
    let destination = match command {
        Command::CreateDirectory { leaf } => parent.join(leaf),
        Command::Rename { new_leaf, .. } => parent.join(new_leaf),
    };
    let rename = matches!(command, Command::Rename { .. });
    let protected = |namespace: &Path| {
        selected.starts_with(namespace)
            || destination.starts_with(namespace)
            || rename && (namespace.starts_with(&selected) || namespace.starts_with(&destination))
    };
    let mut stamps = vec![];
    for root in &protection.managed {
        let root = guard_path(root, &selected)?;
        if protected(&root) {
            return Err("Managed Project/library backing storage is protected.".into());
        }
    }
    // Read only manifest path declarations; resolving could create/recover data.
    // A valid Project does not make unrelated host notes an engine namespace.
    for ancestor in parent.ancestors() {
        for namespace in project_namespaces(ancestor, &mut stamps)? {
            if protected(&namespace) {
                return Err("Canonical Project manifest/shard/storage paths are protected.".into());
            }
        }
    }
    if let Command::Rename {
        old_leaf, source, ..
    } = command
    {
        let selected = parent.join(old_leaf);
        let stamp = Stamp::read(&selected)?;
        if stamp.identity != *source
            || !matches!(source.kind, EntryKind::Directory | EntryKind::File)
        {
            return Err("The selected entry changed or is not a regular file/folder.".into());
        }
        for anchor in protection.anchors.iter().chain(&protection.managed) {
            let anchor = guard_path(anchor, &selected)?;
            if anchor.starts_with(&selected) || anchor.starts_with(&destination) {
                return Err(
                    "This entry contains an active Project, library or session origin/cwd.".into(),
                );
            }
        }
        if old_leaf == "project.json" || old_leaf == ".datum" {
            return Err("Datum backing names cannot be renamed here.".into());
        }
        if source.kind == EntryKind::Directory {
            inspect_tree(
                parent_handle,
                std::ffi::OsStr::new(old_leaf),
                *source,
                &mut stamps,
            )?;
        }
        stamps.push(stamp);
    }
    Ok(Inspection { stamps })
}

/// Resolve existing prefixes, retaining missing suffixes. A disappeared,
/// unrelated managed path must not disable the whole host browser.
fn guard_path(path: &Path, selected: &Path) -> Result<PathBuf, String> {
    let mut prefix = path.to_path_buf();
    let mut suffix = vec![];
    loop {
        match prefix.canonicalize() {
            Ok(mut resolved) => {
                for part in suffix.into_iter().rev() {
                    resolved.push(part);
                }
                return Ok(resolved);
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                let Some(leaf) = prefix.file_name().map(|s| s.to_owned()) else {
                    return Err("Safety path cannot be resolved.".into());
                };
                suffix.push(leaf);
                prefix.pop();
            }
            Err(error) => {
                if path.starts_with(selected) || selected.starts_with(path) {
                    return Err(format!(
                        "Relevant backing/session path cannot be inspected: {error}"
                    ));
                }
                return Ok(path.to_path_buf());
            }
        }
    }
}

fn project_namespaces(root: &Path, stamps: &mut Vec<Stamp>) -> Result<Vec<PathBuf>, String> {
    let manifest = root.join("project.json");
    match fs::symlink_metadata(&manifest) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            // A global ~/.datum/revision store does not make HOME a Project.
            // Reserved storage is protected directly; ancestry needs a manifest.
            return Ok(vec![]);
        }
        Err(error) => return Err(format!("Project classification is incomplete: {error}")),
        Ok(metadata) if !metadata.is_file() || metadata.file_type().is_symlink() => {
            return Err("Project manifest classification is ambiguous.".into());
        }
        Ok(_) => {}
    }
    let mut stamp = Stamp::read(&manifest)?;
    let contents = fs::read(&manifest).map_err(|e| e.to_string())?;
    let value: serde_json::Value = serde_json::from_slice(&contents)
        .map_err(|e| format!("Project manifest cannot be classified safely: {e}"))?;
    let mut namespaces = vec![manifest.clone(), root.join(".datum"), root.join(".ids")];
    for key in ["board", "schematic", "rules"] {
        let path = value
            .get(key)
            .and_then(serde_json::Value::as_str)
            .filter(|s| !s.is_empty())
            .ok_or_else(|| format!("Project manifest has no inspectable {key} storage path."))?;
        namespaces.push(storage_namespace(root, path)?);
    }
    if let Some(pools) = value.get("pools") {
        for pool in pools
            .as_array()
            .ok_or("Project pool paths are ambiguous.")?
        {
            let path = pool
                .get("path")
                .and_then(serde_json::Value::as_str)
                .ok_or("Project pool path is ambiguous.")?;
            namespaces.push(storage_namespace(root, path)?);
        }
    }
    if let Some(settings) = value.get("settings") {
        let settings = settings
            .as_object()
            .ok_or("Project settings paths are ambiguous.")?;
        for key in ["checking", "output"] {
            if let Some(path) = settings.get(key) {
                let path = path
                    .as_str()
                    .filter(|p| !p.is_empty())
                    .ok_or("Project settings path is ambiguous.")?;
                namespaces.push(storage_namespace(root, path)?);
            }
        }
    }
    stamp.contents = Some(contents);
    stamp.recheck()?;
    stamps.push(stamp);
    Ok(namespaces)
}

fn storage_namespace(root: &Path, relative: &str) -> Result<PathBuf, String> {
    let raw = root.join(relative);
    let path = guard_path(&raw, root)?;
    match fs::metadata(&path) {
        Ok(metadata) if metadata.is_dir() => return Ok(path),
        Ok(_) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            if path.extension().is_none() {
                return Ok(path);
            }
        }
        Err(error) => return Err(format!("Project backing path cannot be checked: {error}")),
    }
    let parent = path.parent().ok_or("Project backing path has no parent.")?;
    if parent == root {
        Ok(path)
    } else {
        Ok(parent.to_path_buf())
    }
}

/// Iterative, no-follow traversal of only the selected folder. Directory stamps
/// detect namespace changes during inspection, without resolving any Project.
fn inspect_tree(
    parent: &File,
    name: &std::ffi::OsStr,
    source: Identity,
    stamps: &mut Vec<Stamp>,
) -> Result<(), String> {
    let first = Stamp::directory(parent, name)?;
    if first.identity != source {
        return Err("Selected directory changed before inspection.".into());
    }
    let mut pending = vec![first];
    while let Some(mut stamp) = pending.pop() {
        let handle = stamp
            .pin
            .as_ref()
            .ok_or("Inspected directory handle is unavailable.")?;
        let entries = directory_entries(handle)?;
        for (name, identity) in &entries {
            if name == "project.json" || name == ".datum" {
                return Err("A Datum Project root/container or backing namespace cannot be renamed by this browser.".into());
            }
            if identity.kind == EntryKind::Directory {
                let child = Stamp::directory(handle, name)?;
                if child.identity != *identity {
                    return Err("Inspected directory entry changed.".into());
                }
                pending.push(child);
            }
        }
        stamp.entries = Some(entries);
        stamps.push(stamp);
    }
    Ok(())
}

fn directory_entries(handle: &File) -> Result<Vec<(std::ffi::OsString, Identity)>, String> {
    let directory = PathBuf::from(format!("/proc/self/fd/{}", handle.as_raw_fd()));
    let mut entries = vec![];
    for entry in fs::read_dir(directory)
        .map_err(|e| format!("Cannot complete folder safety inspection: {e}"))?
    {
        let entry = entry.map_err(|e| e.to_string())?;
        let metadata = fs::symlink_metadata(entry.path()).map_err(|e| e.to_string())?;
        entries.push((entry.file_name(), Identity::of(&metadata)));
    }
    entries.sort_unstable_by(|a, b| a.0.cmp(&b.0));
    Ok(entries)
}
