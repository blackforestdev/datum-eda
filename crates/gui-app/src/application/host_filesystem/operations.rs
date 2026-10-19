//! Parent-handle-relative, atomic no-replace publication and truthful receipts.
use super::*;
use std::{
    ffi::CString,
    fs::{File, OpenOptions},
    os::{fd::AsRawFd, unix::fs::OpenOptionsExt},
    path::{Component, Path},
};

pub(super) fn execute(job: Job) -> Outcome {
    match prepare(&job) {
        Ok((parent, path, inspection)) => {
            let result = publish(&parent, &path, &job, inspection);
            match result {
                Ok(change) if change.old_leaf.as_deref() == Some(change.new_leaf.as_str()) => {
                    Outcome::Unchanged(change)
                }
                Ok(change) => Outcome::Committed(change),
                Err(error) => Outcome::Refused(error),
            }
        }
        Err(error) => Outcome::Refused(error),
    }
}

fn prepare(job: &Job) -> Result<(File, PathBuf, safety::Inspection), String> {
    job.current()?;
    let request = &job.request;
    let leaf = match &request.command {
        Command::CreateDirectory { leaf } => leaf,
        Command::Rename { new_leaf, .. } => new_leaf,
    };
    valid_leaf(leaf)?;
    if matches!(leaf.as_str(), ".datum" | "project.json") {
        return Err("This name is reserved for Datum backing storage.".into());
    }
    if let Command::Rename { old_leaf, .. } = &request.command {
        valid_leaf(old_leaf)?;
    }
    let (parent, path) = open_parent(request)?;
    let inspection = safety::inspect(&parent, &path, &request.command, &job.protection)?;
    Ok((parent, path, inspection))
}

fn open_parent(request: &Request) -> Result<(File, PathBuf), String> {
    let path = request
        .parent
        .canonicalize()
        .map_err(|e| format!("Location is unavailable: {e}"))?;
    let parent = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC)
        .open(&path)
        .map_err(|e| format!("Cannot open Location: {e}"))?;
    check_parent(&parent, &path, request.parent_identity)?;
    Ok((parent, path))
}
/// Read-only availability uses the same pinned parent and safety classifier as
/// publication. Empty Create leaf deliberately means the parent itself here;
/// it never passes through the mutating request validator or syscall path.
pub(super) fn availability(request: &Request, protection: &Protection) -> Result<(), String> {
    let (parent, path) = open_parent(request)?;
    if let Command::Rename { old_leaf, .. } = &request.command {
        valid_leaf(old_leaf)?;
    }
    let inspection = safety::inspect(&parent, &path, &request.command, protection)?;
    inspection.recheck()?;
    check_parent(&parent, &path, request.parent_identity)
}

fn publish(
    parent: &File,
    path: &Path,
    job: &Job,
    inspection: safety::Inspection,
) -> Result<Change, String> {
    // The inspector can take time; recheck the exact parent and inspected tree
    // immediately before mutation. This cannot control uncooperating OS clients.
    job.current()?;
    let request = &job.request;
    inspection.recheck()?;
    check_parent(parent, path, request.parent_identity)?;
    job.current()?;
    let (old_leaf, new_leaf, identity) = match &request.command {
        Command::CreateDirectory { leaf } => {
            let name = c_name(leaf)?;
            job.current()?;
            let result = unsafe { libc::mkdirat(parent.as_raw_fd(), name.as_ptr(), 0o777) };
            if result != 0 {
                return Err(format!(
                    "Folder was not created: {}",
                    std::io::Error::last_os_error()
                ));
            }
            (
                None,
                leaf.clone(),
                metadata_at(parent, leaf).ok().map(|m| Identity::of(&m)),
            )
        }
        Command::Rename {
            old_leaf,
            new_leaf,
            source,
        } => {
            let source_handle = OpenOptions::new()
                .read(true)
                .custom_flags(libc::O_PATH | libc::O_NOFOLLOW | libc::O_CLOEXEC)
                .open(PathBuf::from(format!("/proc/self/fd/{}", parent.as_raw_fd())).join(old_leaf))
                .map_err(|e| format!("Cannot pin selected entry: {e}"))?;
            let pinned = source_handle.metadata().map_err(|e| e.to_string())?;
            let before =
                metadata_at(parent, old_leaf).map_err(|e| format!("Entry is unavailable: {e}"))?;
            if Identity::of(&pinned) != *source
                || Identity::of(&before) != *source
                || !matches!(source.kind, EntryKind::File | EntryKind::Directory)
            {
                return Err(
                    "The entry changed or is not a regular file/folder. Re-read Location.".into(),
                );
            }
            if old_leaf == new_leaf {
                return Ok(Change {
                    old_leaf: Some(old_leaf.clone()),
                    new_leaf: new_leaf.clone(),
                    identity: Some(*source),
                    warning: None,
                });
            }
            let old = c_name(old_leaf)?;
            let new = c_name(new_leaf)?;
            job.current()?;
            let result = unsafe {
                libc::renameat2(
                    parent.as_raw_fd(),
                    old.as_ptr(),
                    parent.as_raw_fd(),
                    new.as_ptr(),
                    libc::RENAME_NOREPLACE,
                )
            };
            if result != 0 {
                return Err(format!(
                    "Entry was not renamed (no overwrite): {}",
                    std::io::Error::last_os_error()
                ));
            }
            (Some(old_leaf.clone()), new_leaf.clone(), Some(*source))
        }
    };
    // Publication already succeeded. A flush/observation failure is a warning,
    // never a fabricated refusal claiming that the side effect did not occur.
    let warning = parent
        .sync_all()
        .err()
        .map(|e| format!("Action completed, but Location flush failed: {e}"))
        .or_else(|| {
            identity.is_none().then(|| {
                "Folder was created but could not be observed again; re-read Location.".into()
            })
        });
    Ok(Change {
        old_leaf,
        new_leaf,
        identity,
        warning,
    })
}

fn check_parent(parent: &File, path: &Path, expected: Identity) -> Result<(), String> {
    let named = fs::symlink_metadata(path).map_err(|e| format!("Location changed: {e}"))?;
    let opened = parent.metadata().map_err(|e| e.to_string())?;
    if expected.kind != EntryKind::Directory
        || Identity::of(&named) != expected
        || Identity::of(&opened) != expected
    {
        return Err("Location identity changed. Re-read Location before retrying.".into());
    }
    Ok(())
}

pub(super) fn valid_leaf(leaf: &str) -> Result<(), String> {
    let mut parts = Path::new(leaf).components();
    if leaf.is_empty()
        || leaf.chars().any(char::is_control)
        || leaf.contains('/')
        || !matches!(parts.next(), Some(Component::Normal(_)))
        || parts.next().is_some()
    {
        return Err(
            "Enter one exact name without separators, control characters, '.' or '..'.".into(),
        );
    }
    Ok(())
}

fn c_name(name: &str) -> Result<CString, String> {
    CString::new(name).map_err(|_| "A name cannot contain NUL.".into())
}

/// Linux /proc pins this path lookup to our already-open parent descriptor;
/// lstat inspects the directory entry, never its link referent.
fn metadata_at(parent: &File, leaf: &str) -> std::io::Result<fs::Metadata> {
    fs::symlink_metadata(PathBuf::from(format!("/proc/self/fd/{}", parent.as_raw_fd())).join(leaf))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn host_filesystem_context_change_after_inspection_refuses_before_mutation() {
        let root = super::super::tests::Scratch::new();
        let epoch = Arc::new(AtomicU64::new(1));
        let job = Job {
            ticket: 1,
            request: root.request(Command::CreateDirectory {
                leaf: "must not appear".into(),
            }),
            protection: Protection::default(),
            epoch: epoch.clone(),
            generation: 1,
        };
        let (parent, path, inspection) = prepare(&job).unwrap();
        epoch.fetch_add(1, Ordering::AcqRel);
        assert!(
            publish(&parent, &path, &job, inspection)
                .unwrap_err()
                .contains("safety context changed")
        );
        assert!(!root.0.join("must not appear").exists());
    }
}
