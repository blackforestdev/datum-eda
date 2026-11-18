//! Private sibling staging and atomic no-replace publication through a pinned parent.
use std::{collections::BTreeMap, path::Path};

use crate::{error::EngineError, substrate::library_asset_operations::refusal};

pub(super) fn supported() -> Result<(), EngineError> {
    if cfg!(target_os = "linux") {
        Ok(())
    } else {
        Err(refusal(
            "atomic no-replace library export is unsupported on this platform",
        ))
    }
}

#[cfg(target_os = "linux")]
mod linux {
    use super::*;
    use std::{
        ffi::CString,
        fs::{self, DirBuilder, File, OpenOptions},
        io::Write,
        os::{
            fd::AsRawFd,
            unix::{
                ffi::OsStrExt,
                fs::{DirBuilderExt, MetadataExt, OpenOptionsExt},
            },
        },
        path::PathBuf,
    };

    // First-party declaration of the already-linked Linux C interface. No new
    // library, crate, provider or replacing-rename fallback is introduced.
    unsafe extern "C" {
        fn geteuid() -> u32;
        fn renameat2(
            oldfd: i32,
            old: *const std::ffi::c_char,
            newfd: i32,
            new: *const std::ffi::c_char,
            flags: u32,
        ) -> i32;
    }

    pub(in super::super) struct Stage {
        parent: File,
        parent_path: PathBuf,
        source: PathBuf,
        destination: String,
        leaf: CString,
        name: CString,
        root: PathBuf,
        identity: (u64, u64),
        published: bool,
    }

    impl Stage {
        pub(in super::super) fn new(
            source: &Path,
            destination: &Path,
        ) -> Result<Self, EngineError> {
            let leaf = destination
                .file_name()
                .ok_or_else(|| refusal("export destination needs a fresh directory name"))?;
            let leaf =
                CString::new(leaf.as_bytes()).map_err(|_| refusal("invalid destination name"))?;
            let parent_path = destination
                .parent()
                .filter(|p| !p.as_os_str().is_empty())
                .unwrap_or(Path::new("."))
                .canonicalize()?;
            if parent_path.starts_with(source) {
                return Err(refusal(
                    "export destination must be outside the source project",
                ));
            }
            let parent = File::open(&parent_path)?;
            admit_parent(&parent)?;
            let alias = PathBuf::from(format!("/proc/self/fd/{}", parent.as_raw_fd()));
            if alias.canonicalize()? != parent_path {
                return Err(refusal("export parent changed while being opened"));
            }
            let destination = parent_path.join(destination.file_name().unwrap());
            let destination_text = destination
                .to_str()
                .ok_or_else(|| {
                    refusal("library export requires a UTF-8 canonical destination path")
                })?
                .to_owned();
            match fs::symlink_metadata(alias.join(destination.file_name().unwrap())) {
                Ok(_) => return Err(refusal("export destination already exists")),
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                Err(e) => return Err(e.into()),
            }
            let name = format!(".datum-library-export-{}", uuid::Uuid::new_v4());
            let root = alias.join(&name);
            DirBuilder::new().mode(0o700).create(&root)?;
            let metadata = fs::symlink_metadata(&root)?;
            let identity = (metadata.dev(), metadata.ino());
            Ok(Self {
                parent,
                parent_path,
                source: source.to_path_buf(),
                destination: destination_text,
                leaf,
                name: CString::new(name).unwrap(),
                root,
                identity,
                published: false,
            })
        }

        fn recheck(&self) -> Result<(), EngineError> {
            admit_parent(&self.parent)?;
            let parent = self.root.parent().unwrap();
            let current = parent.canonicalize()?;
            if current != self.parent_path || current.starts_with(&self.source) {
                return Err(refusal("export parent moved; no publication performed"));
            }
            let metadata = fs::symlink_metadata(&self.root)?;
            if !metadata.is_dir() || (metadata.dev(), metadata.ino()) != self.identity {
                return Err(refusal(
                    "owned export stage changed; no publication performed",
                ));
            }
            Ok(())
        }

        pub(in super::super) fn write_files(
            &mut self,
            files: &BTreeMap<String, Vec<u8>>,
        ) -> Result<(), EngineError> {
            self.recheck()?;
            for directory in ["objects", "objects/sha256", "blobs", "blobs/sha256"] {
                DirBuilder::new()
                    .mode(0o700)
                    .create(self.root.join(directory))?;
            }
            for (relative, bytes) in files {
                if !relative
                    .split('/')
                    .all(|s| !s.is_empty() && s != "." && s != "..")
                    || relative.starts_with('/')
                    || relative.contains('\\')
                {
                    return Err(refusal("unsafe library export path"));
                }
                let mut file = OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .mode(0o600)
                    .open(self.root.join(relative))?;
                file.write_all(bytes)?;
                file.sync_all()?;
            }
            for directory in ["objects/sha256", "blobs/sha256", "objects", "blobs", ""] {
                File::open(self.root.join(directory))?.sync_all()?;
            }
            Ok(())
        }

        pub(in super::super) fn publish(mut self) -> Result<(String, Option<String>), EngineError> {
            self.recheck()?;
            // SAFETY: live directory descriptor and NUL-terminated owned leaf
            // names remain valid throughout the call; no pointers are retained.
            let result = unsafe {
                renameat2(
                    self.parent.as_raw_fd(),
                    self.name.as_ptr(),
                    self.parent.as_raw_fd(),
                    self.leaf.as_ptr(),
                    1, /* RENAME_NOREPLACE */
                )
            };
            if result != 0 {
                return Err(refusal(format!(
                    "library export not published (atomic no-replace): {}",
                    std::io::Error::last_os_error()
                )));
            }
            self.published = true;
            // Publication happened. Report a flush failure truthfully as a
            // durability warning, never as a refusal suggesting nothing exists.
            let warning = self
                .parent
                .sync_all()
                .err()
                .map(|e| format!("Export completed; parent directory flush failed: {e}"));
            Ok((self.destination.clone(), warning))
        }
    }

    fn admit_parent(parent: &File) -> Result<(), EngineError> {
        let metadata = parent.metadata()?;
        // SAFETY: geteuid takes no arguments and retains no pointers.
        let caller = unsafe { geteuid() };
        // Output-parent admission protects the mutable staging name from other
        // principals. Same-UID and privileged processes remain trusted; this
        // check does not claim isolation from the caller's own processes.
        if !metadata.is_dir() || metadata.uid() != caller || metadata.mode() & 0o022 != 0 {
            return Err(refusal(
                "library export requires a caller-owned parent without group or other write permission",
            ));
        }
        Ok(())
    }

    impl Drop for Stage {
        fn drop(&mut self) {
            if !self.published
                && let Ok(metadata) = fs::symlink_metadata(&self.root)
                && metadata.is_dir()
                && (metadata.dev(), metadata.ino()) == self.identity
            {
                // Never remove an existing destination or a replaced stage.
                let _ = fs::remove_dir_all(&self.root);
            }
        }
    }
}

#[cfg(target_os = "linux")]
pub(super) use linux::Stage;

#[cfg(not(target_os = "linux"))]
pub(super) struct Stage;

#[cfg(not(target_os = "linux"))]
impl Stage {
    pub(super) fn new(_: &Path, _: &Path) -> Result<Self, EngineError> {
        supported()?;
        unreachable!()
    }
    pub(super) fn write_files(&mut self, _: &BTreeMap<String, Vec<u8>>) -> Result<(), EngineError> {
        supported()
    }
    pub(super) fn publish(self) -> Result<(String, Option<String>), EngineError> {
        supported()?;
        unreachable!()
    }
}
