//! Latest-only read-only menu qualification, separate from retained mutations.
use super::*;
use std::sync::Mutex;

pub(crate) struct Availability {
    pub ticket: u64,
    pub create: Result<(), String>,
    pub rename: Option<Result<(), String>>,
}
struct Query {
    ticket: u64,
    parent: Request,
    rename: Option<Command>,
    protection: Protection,
}
pub(crate) struct Inspector {
    requests: mpsc::SyncSender<()>,
    latest: Arc<Mutex<Option<Query>>>,
    answers: Arc<Mutex<Option<Availability>>>,
    generation: Arc<AtomicU64>,
}
impl Inspector {
    pub fn new(wake: impl Fn() + Send + 'static) -> std::io::Result<Self> {
        let (requests, incoming) = mpsc::sync_channel::<()>(1);
        let latest = Arc::new(Mutex::new(None::<Query>));
        let pending = latest.clone();
        let answers = Arc::new(Mutex::new(None));
        let outgoing = answers.clone();
        let generation = Arc::new(AtomicU64::new(0));
        let active = generation.clone();
        std::thread::Builder::new()
            .name("datum-host-file-inspection".into())
            .spawn(move || {
                while incoming.recv().is_ok() {
                    let Some(mut query) = pending.lock().unwrap().take() else {
                        continue;
                    };
                    let current = || active.load(Ordering::Acquire) == query.ticket;
                    if !current() {
                        continue;
                    }
                    let create = operations::availability(&query.parent, &query.protection);
                    let rename = query.rename.take().map(|command| {
                        query.parent.command = command;
                        operations::availability(&query.parent, &query.protection)
                    });
                    if current() {
                        *outgoing.lock().unwrap() = Some(Availability {
                            ticket: query.ticket,
                            create,
                            rename,
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
    pub fn request(
        &self,
        parent: Request,
        rename: Option<Command>,
        protection: Protection,
    ) -> Result<u64, String> {
        let ticket = self
            .generation
            .fetch_add(1, Ordering::AcqRel)
            .wrapping_add(1);
        *self.latest.lock().unwrap() = Some(Query {
            ticket,
            parent,
            rename,
            protection,
        });
        match self.requests.try_send(()) {
            Ok(()) | Err(mpsc::TrySendError::Full(())) => Ok(ticket),
            Err(_) => Err("Read-only safety inspection worker is unavailable.".into()),
        }
    }
    pub fn receive(&self) -> Option<Availability> {
        self.answers.lock().unwrap().take()
    }
}
