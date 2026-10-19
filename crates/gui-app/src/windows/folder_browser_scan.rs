//! One read-only filesystem worker per browser coordinator. Never joins on UI.
use std::{
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex,
        atomic::{AtomicU64, Ordering},
        mpsc,
    },
};

#[derive(Debug)]
pub(super) struct Listing {
    pub entries: Vec<datum_gui_protocol::BrowserEntry>,
    pub parent_identity: Option<(u64, u64)>,
    pub suggestions: Option<datum_gui_protocol::FolderSuggestions>,
    pub occupied: std::collections::BTreeSet<String>,
    pub notice: Option<String>,
}

pub(super) struct ScanResult {
    pub ticket: u64,
    pub input: Option<String>,
    pub directory: PathBuf,
    pub result: Result<Listing, String>,
}

type Request = (u64, PathBuf, Option<String>, bool);
#[path = "folder_browser_query.rs"]
mod query;

pub(super) struct Scanner {
    requests: mpsc::SyncSender<()>,
    latest: Arc<Mutex<Option<Request>>>,
    answers: Arc<Mutex<Option<ScanResult>>>,
    generation: Arc<AtomicU64>,
}

impl Scanner {
    pub fn new(wake: impl Fn() + Send + 'static) -> std::io::Result<Self> {
        let (requests, incoming) = mpsc::sync_channel::<()>(1);
        let latest = Arc::new(Mutex::new(None::<Request>));
        let pending = latest.clone();
        let answers = Arc::new(Mutex::new(None));
        let outgoing = answers.clone();
        let generation = Arc::new(AtomicU64::new(0));
        let active = generation.clone();
        std::thread::Builder::new()
            .name("datum-folder-browser".into())
            .spawn(move || {
                while incoming.recv().is_ok() {
                    let Some((ticket, directory, input, assisted)) = pending.lock().unwrap().take()
                    else {
                        continue;
                    };
                    let current = || active.load(Ordering::Acquire) == ticket;
                    if !current() {
                        continue;
                    }
                    let result = query::discover(&directory, input.as_deref(), assisted, current);
                    if current() {
                        *outgoing.lock().unwrap() = Some(ScanResult {
                            ticket,
                            input,
                            directory,
                            result,
                        });
                        wake();
                    }
                }
            })?;
        Ok(Self {
            requests,
            latest,
            answers,
            generation,
        })
    }

    pub fn cancel(&self) {
        self.generation.fetch_add(1, Ordering::AcqRel);
    }

    #[cfg(test)]
    pub fn request(&self, directory: PathBuf) -> Result<u64, String> {
        self.request_input(directory, None, false)
    }
    pub fn request_input(
        &self,
        directory: PathBuf,
        input: Option<String>,
        assisted: bool,
    ) -> Result<u64, String> {
        let ticket = self
            .generation
            .fetch_add(1, Ordering::AcqRel)
            .wrapping_add(1);
        *self.latest.lock().unwrap() = Some((ticket, directory, input, assisted));
        match self.requests.try_send(()) {
            Ok(()) | Err(mpsc::TrySendError::Full(())) => {}
            Err(mpsc::TrySendError::Disconnected(())) => {
                return Err("Folder reader is unavailable. Reopen New/Open to retry.".into());
            }
        }
        Ok(ticket)
    }

    pub fn receive(&self) -> Option<ScanResult> {
        self.answers.lock().unwrap().take()
    }
}

impl Drop for Scanner {
    fn drop(&mut self) {
        self.cancel();
    }
}

fn scan(directory: &Path, current: impl Fn() -> bool) -> Result<Listing, String> {
    use datum_gui_protocol::{BrowserEntry, BrowserEntryKind};
    use std::os::{fd::AsRawFd, unix::fs::MetadataExt};
    let parent = std::fs::File::open(directory).map_err(|e| {
        format!(
            "Cannot read folder: {e}. Folder: {}. Enter another folder path or use Up/Home.",
            directory.display()
        )
    })?;
    let metadata = parent.metadata().map_err(|e| e.to_string())?;
    let parent_identity = Some((metadata.dev(), metadata.ino()));
    let entries = std::fs::read_dir(format!("/proc/self/fd/{}", parent.as_raw_fd()))
        .map_err(|e| format!("Cannot read folder: {e}. Folder: {}.", directory.display()))?;
    let mut observed = vec![];
    let mut skipped = 0;
    let mut occupied = std::collections::BTreeSet::new();
    for entry in entries {
        if !current() {
            return Err("Folder scan cancelled".into());
        }
        let entry = match entry {
            Ok(entry) => entry,
            Err(_) => {
                skipped += 1;
                continue;
            }
        };
        let Some(name) = entry.file_name().to_str().map(str::to_owned) else {
            skipped += 1;
            continue;
        };
        occupied.insert(name.clone());
        let metadata = match std::fs::symlink_metadata(entry.path()) {
            Ok(metadata) => metadata,
            Err(_) => {
                skipped += 1;
                continue;
            }
        };
        let kind = if metadata.file_type().is_symlink() {
            BrowserEntryKind::Link {
                directory: std::fs::metadata(entry.path()).is_ok_and(|m| m.is_dir()),
            }
        } else if metadata.is_dir() {
            BrowserEntryKind::Directory
        } else if metadata.is_file() {
            BrowserEntryKind::File
        } else {
            BrowserEntryKind::Other
        };
        observed.push(BrowserEntry {
            name,
            kind,
            identity: Some((metadata.dev(), metadata.ino())),
        });
    }
    observed.sort_by_cached_key(|entry| {
        (
            !entry.navigable(),
            entry.name.to_lowercase(),
            entry.name.clone(),
        )
    });
    Ok(Listing { entries: observed, parent_identity, occupied, suggestions: None,
        notice: (skipped > 0).then(|| format!("{skipped} entries could not be listed (unreadable, disappeared or unsupported filename encoding).")) })
}

#[cfg(test)]
#[path = "folder_browser_scan_tests.rs"]
mod tests;
