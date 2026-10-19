//! Explicit host naming operations. No DesignModel, genesis or resolver writes.
use std::os::unix::fs::MetadataExt;
use std::{
    fs,
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
        mpsc,
    },
};

#[path = "host_filesystem/availability.rs"]
mod availability;
pub(crate) use availability::{Availability, Inspector};

#[path = "host_filesystem/operations.rs"]
mod operations;
#[path = "host_filesystem/safety.rs"]
mod safety;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum EntryKind {
    File,
    Directory,
    Link,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Identity {
    pub device: u64,
    pub inode: u64,
    pub kind: EntryKind,
}
impl Identity {
    pub fn of(metadata: &fs::Metadata) -> Self {
        let kind = if metadata.file_type().is_symlink() {
            EntryKind::Link
        } else if metadata.is_dir() {
            EntryKind::Directory
        } else if metadata.is_file() {
            EntryKind::File
        } else {
            EntryKind::Other
        };
        Self {
            device: metadata.dev(),
            inode: metadata.ino(),
            kind,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct Protection {
    /// Neither host action writes these backing-storage subtrees.
    pub managed: Vec<PathBuf>,
    /// Rename cannot move these origins/cwds or their ancestors.
    pub anchors: Vec<PathBuf>,
}

#[derive(Debug, Clone)]
pub(crate) enum Command {
    CreateDirectory {
        leaf: String,
    },
    Rename {
        old_leaf: String,
        new_leaf: String,
        source: Identity,
    },
}

#[derive(Debug, Clone)]
pub(crate) struct Request {
    pub scope: u64,
    pub parent: PathBuf,
    pub parent_identity: Identity,
    pub command: Command,
}

struct Job {
    ticket: u64,
    request: Request,
    protection: Protection,
    epoch: Arc<AtomicU64>,
    generation: u64,
}
impl Job {
    fn current(&self) -> Result<(), String> {
        if self.epoch.load(Ordering::Acquire) != self.generation {
            return Err(
                "Project or session safety context changed. Re-read Location before retrying."
                    .into(),
            );
        }
        Ok(())
    }
}

#[derive(Debug)]
pub(crate) struct Change {
    pub old_leaf: Option<String>,
    pub new_leaf: String,
    pub identity: Option<Identity>,
    pub warning: Option<String>,
}

#[derive(Debug)]
pub(crate) enum Outcome {
    Committed(Change),
    Unchanged(Change),
    Refused(String),
    /// A lost worker cannot truthfully assert whether a syscall committed.
    Indeterminate(String),
}

#[derive(Debug)]
pub(crate) struct Completion {
    pub ticket: u64,
    pub scope: u64,
    pub outcome: Outcome,
}

/// One in-flight command until its retained receipt is consumed. Browser scan
/// cancellation and dialog retirement never overwrite this completion channel.
pub(crate) struct Service {
    requests: mpsc::SyncSender<Job>,
    answers: mpsc::Receiver<Completion>,
    pending: Option<(u64, u64)>,
    next_ticket: u64,
    protection: Protection,
    epoch: Arc<AtomicU64>,
}
impl Service {
    pub fn new(wake: impl Fn() + Send + 'static) -> std::io::Result<Self> {
        let (requests, incoming) = mpsc::sync_channel::<Job>(1);
        let (outgoing, answers) = mpsc::channel();
        std::thread::Builder::new()
            .name("datum-host-file-actions".into())
            .spawn(move || {
                while let Ok(job) = incoming.recv() {
                    let scope = job.request.scope;
                    let ticket = job.ticket;
                    let outcome = operations::execute(job);
                    if outgoing
                        .send(Completion {
                            ticket,
                            scope,
                            outcome,
                        })
                        .is_err()
                    {
                        break;
                    }
                    wake();
                }
            })?;
        Ok(Self {
            requests,
            answers,
            pending: None,
            next_ticket: 0,
            protection: Protection::default(),
            epoch: Arc::new(AtomicU64::new(1)),
        })
    }

    pub fn set_protection(&mut self, mut protection: Protection) {
        protection.managed.sort();
        protection.managed.dedup();
        protection.anchors.sort();
        protection.anchors.dedup();
        if protection != self.protection {
            self.epoch.fetch_add(1, Ordering::AcqRel);
            self.protection = protection;
        }
    }

    pub fn busy(&self) -> bool {
        self.pending.is_some()
    }

    pub fn submit(&mut self, request: Request) -> Result<u64, String> {
        if self.busy() {
            return Err("Wait for the current folder action to finish.".into());
        }
        let ticket = self
            .next_ticket
            .checked_add(1)
            .ok_or("Folder action IDs exhausted.")?;
        let scope = request.scope;
        self.requests
            .try_send(Job {
                ticket,
                request,
                protection: self.protection.clone(),
                epoch: self.epoch.clone(),
                generation: self.epoch.load(Ordering::Acquire),
            })
            .map_err(|_| "Folder action worker is unavailable.".to_owned())?;
        self.next_ticket = ticket;
        self.pending = Some((ticket, scope));
        Ok(ticket)
    }

    pub fn receive(&mut self) -> Option<Completion> {
        match self.answers.try_recv() {
            Ok(completion) => { self.pending = None; Some(completion) }
            Err(mpsc::TryRecvError::Empty) => None,
            Err(mpsc::TryRecvError::Disconnected) => self.pending.take().map(|(ticket, scope)| Completion {
                ticket, scope, outcome: Outcome::Indeterminate("Folder action worker stopped. Re-read Location to inspect the actual filesystem before retrying.".into()),
            }),
        }
    }
}

#[cfg(test)]
#[path = "host_filesystem/tests.rs"]
mod tests;
