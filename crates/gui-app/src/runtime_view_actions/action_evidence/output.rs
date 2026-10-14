//! Bounded opt-in diagnostics; encoding/I/O never run in input dispatch.
use serde::Serialize;
use serde_json::Value;
use std::io::{BufWriter, Write};
use std::sync::{Arc, mpsc};
use std::thread::JoinHandle;

pub(super) const MAX_RECORD_BYTES: usize = 64 * 1024;
const MAX_SESSION_BYTES: usize = 8 * 1024 * 1024;
const INCOMPLETE_RESERVE: usize = 1024;
const KIND: &str = "datum.action-evidence/v2";

pub(super) struct Record {
    pub event: &'static str,
    pub sequence: u64,
    pub value: Arc<Value>,
}

// Match the old JSON object's lexicographic field order and exact wire shape.
#[derive(Serialize)]
struct Envelope<'a> {
    event: &'a str,
    kind: &'static str,
    sequence: u64,
    value: &'a Value,
}

pub(super) struct Output {
    sender: Option<mpsc::Sender<Record>>,
    worker: Option<JoinHandle<()>>,
}
impl Output {
    pub(super) fn start_stderr() -> std::io::Result<Self> {
        Self::start_record_writer(|record| {
            let stderr = std::io::stderr();
            // Competing GUI stderr logs may wait for this complete record. The
            // lock is released before the worker waits for another queued event.
            let mut lock = stderr.lock();
            write_record(&mut lock, record)
        })
    }
    #[cfg(test)]
    fn start(mut writer: impl Write + Send + 'static) -> std::io::Result<Self> {
        Self::start_record_writer(move |record| write_record(&mut writer, record))
    }
    fn start_record_writer(
        write: impl FnMut(&Record) -> std::io::Result<()> + Send + 'static,
    ) -> std::io::Result<Self> {
        // Ordinary launches create neither this worker nor this queue. These
        // are bounded diagnostics, not a full-selection proof transport.
        let (sender, receiver) = mpsc::channel();
        let worker = std::thread::Builder::new()
            .name("datum-action-evidence".into())
            .spawn(move || write_records(receiver, write))?;
        Ok(Self {
            sender: Some(sender),
            worker: Some(worker),
        })
    }
    pub(super) fn submit(&self, record: Record) {
        if let Some(sender) = &self.sender {
            // Closed diagnostic output must not change product behavior.
            let _ = sender.send(record);
        }
    }
    pub(super) fn finish(mut self) {
        self.sender.take();
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

fn write_records(
    receiver: mpsc::Receiver<Record>,
    write: impl FnMut(&Record) -> std::io::Result<()>,
) {
    write_records_with_limits(receiver, write, MAX_RECORD_BYTES, MAX_SESSION_BYTES);
}

fn write_records_with_limits(
    receiver: mpsc::Receiver<Record>,
    mut write: impl FnMut(&Record) -> std::io::Result<()>,
    record_limit: usize,
    session_limit: usize,
) {
    let mut total = 0;
    for record in receiver {
        let mut counter = SizeLimit {
            bytes: 0,
            limit: record_limit,
        };
        let result = write_record(&mut counter, &record);
        let reason = if result.is_err() {
            Some("record_limit")
        } else if total + counter.bytes > session_limit.saturating_sub(INCOMPLETE_RESERVE) {
            Some("session_limit")
        } else {
            None
        };
        if let Some(reason) = reason {
            let _ = write(&Record {
                event: "diagnostics_incomplete",
                sequence: record.sequence,
                value: Arc::new(serde_json::json!({"complete":false, "reason":reason,
                    "record_limit_bytes":record_limit, "session_limit_bytes":session_limit})),
            });
            return;
        }
        if write(&record).is_err() {
            return;
        }
        total += counter.bytes;
    }
}

struct SizeLimit {
    bytes: usize,
    limit: usize,
}
impl Write for SizeLimit {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        if bytes.len() > self.limit.saturating_sub(self.bytes) {
            return Err(std::io::ErrorKind::FileTooLarge.into());
        }
        self.bytes += bytes.len();
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

fn write_record(writer: &mut impl Write, record: &Record) -> std::io::Result<()> {
    let mut writer = BufWriter::new(writer);
    let envelope = Envelope {
        event: record.event,
        kind: KIND,
        sequence: record.sequence,
        value: record.value.as_ref(),
    };
    let _write = crate::gui_runtime_support::phase_probe::Probe::start("evidence_write");
    let written = (|| -> std::io::Result<()> {
        writer.write_all(b"DATUM_ACTION_EVIDENCE ")?;
        let _probe = crate::gui_runtime_support::phase_probe::Probe::start("evidence_encode");
        serde_json::to_writer(&mut writer, &envelope).map_err(std::io::Error::other)?;
        writer.write_all(b"\n")?;
        writer.flush()
    })();
    if written.is_err() {
        // A partial record cannot be resumed with another event. Disarm
        // buffered retry on drop; later submissions fail without blocking.
        let _ = writer.into_parts();
    }
    written
}

#[cfg(test)]
#[path = "output_tests.rs"]
mod tests;
