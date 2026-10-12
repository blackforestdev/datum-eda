//! Lossless opt-in evidence output; encoding/I/O never run in input dispatch.
use serde::Serialize;
use serde_json::Value;
use std::io::{BufWriter, Write};
use std::sync::{Arc, mpsc};
use std::thread::JoinHandle;

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
        // The diagnostic FIFO is lossless: it has no blocking send, record cap,
        // sampling or coalescing of required events. Ordinary launches create
        // neither this worker nor this queue. Complete owned records are queued.
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
    mut write: impl FnMut(&Record) -> std::io::Result<()>,
) {
    for record in receiver {
        if write(&record).is_err() {
            return;
        }
    }
}

fn write_record(writer: &mut impl Write, record: &Record) -> std::io::Result<()> {
    let mut writer = BufWriter::new(writer);
    let envelope = Envelope {
        event: record.event,
        kind: "datum.action-evidence/v1",
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
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};
    fn record(sequence: u64, value: Value) -> Record {
        Record {
            event: "state",
            sequence,
            value: Arc::new(value),
        }
    }
    struct HeldWriter {
        data: Arc<Mutex<Vec<u8>>>,
        arrived: mpsc::Sender<()>,
        release: Option<mpsc::Receiver<()>>,
    }
    impl Write for HeldWriter {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            if let Some(release) = self.release.take() {
                let _ = self.arrived.send(());
                release.recv().unwrap();
            }
            self.data.lock().unwrap().extend_from_slice(bytes);
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    #[test]
    fn blocked_sink_does_not_block_submission_and_finish_preserves_order_and_values() {
        let data = Arc::new(Mutex::new(Vec::new()));
        let (arrived, waiting) = mpsc::channel();
        let (release, held) = mpsc::channel();
        let output = Output::start(HeldWriter {
            data: data.clone(),
            arrived,
            release: Some(held),
        })
        .unwrap();
        let records: Vec<_> = (1..=3).map(|sequence| serde_json::json!({
            "event": "state", "kind": "datum.action-evidence/v1",
            "sequence": sequence, "value": {"selection": "Run: complete\n\"52\" λ", "members": [1,2,3]}
        })).collect();
        output.submit(record(1, records[0]["value"].clone()));
        waiting
            .recv_timeout(std::time::Duration::from_secs(2))
            .unwrap();
        // The sink remains held while all later native-event records enqueue.
        output.submit(record(2, records[1]["value"].clone()));
        output.submit(record(3, records[2]["value"].clone()));
        release.send(()).unwrap();
        output.finish();
        let bytes = data.lock().unwrap();
        let actual: Vec<Value> = std::str::from_utf8(&bytes)
            .unwrap()
            .lines()
            .map(|line| {
                serde_json::from_str(line.strip_prefix("DATUM_ACTION_EVIDENCE ").unwrap()).unwrap()
            })
            .collect();
        assert_eq!(actual, records);
    }
    #[test]
    fn direct_envelope_matches_prior_wire_bytes_with_unicode_and_escapes() {
        let data = Arc::new(Mutex::new(Vec::new()));
        let (arrived, _) = mpsc::channel();
        let output = Output::start(HeldWriter {
            data: data.clone(),
            arrived,
            release: None,
        })
        .unwrap();
        let value = serde_json::json!({
            "selection": "Compound λ雪\n\t\"quoted\"\\path\u{0000}",
            "members": [1, 2, 3], "nested": {"z": false, "a": null}
        });
        let prior = serde_json::json!({
            "kind": "datum.action-evidence/v1", "sequence": 42,
            "event": "state", "value": value
        });
        let mut expected = b"DATUM_ACTION_EVIDENCE ".to_vec();
        serde_json::to_writer(&mut expected, &prior).unwrap();
        expected.push(b'\n');
        output.submit(record(42, prior["value"].clone()));
        output.finish();
        assert_eq!(*data.lock().unwrap(), expected);
    }
    struct ClosedWriter;
    impl Write for ClosedWriter {
        fn write(&mut self, _: &[u8]) -> std::io::Result<usize> {
            Err(std::io::ErrorKind::BrokenPipe.into())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    #[test]
    fn closed_sink_remains_nonfatal_during_submission_and_drain() {
        let output = Output::start(ClosedWriter).unwrap();
        output.submit(record(1, serde_json::json!({})));
        output.submit(record(2, serde_json::json!({})));
        output.finish();
    }
    struct PartialWriter(Arc<Mutex<(Vec<u8>, usize)>>);
    impl Write for PartialWriter {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            let mut state = self.0.lock().unwrap();
            let remaining = 32 - state.0.len();
            if remaining == 0 {
                state.1 += 1;
                return Err(std::io::ErrorKind::BrokenPipe.into());
            }
            let written = bytes.len().min(remaining);
            state.0.extend_from_slice(&bytes[..written]);
            Ok(written)
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    #[test]
    fn partial_sink_failure_stops_before_next_record_without_buffered_retry() {
        let state = Arc::new(Mutex::new((Vec::new(), 0)));
        let output = Output::start(PartialWriter(state.clone())).unwrap();
        output.submit(record(
            1,
            serde_json::json!({"selection":"x".repeat(16384)}),
        ));
        output.submit(record(2, serde_json::json!({"selection":"second"})));
        output.finish();
        let state = state.lock().unwrap();
        assert_eq!(state.0.len(), 32);
        assert!(state.0.starts_with(b"DATUM_ACTION_EVIDENCE "));
        assert_eq!(state.1, 1);
    }
    struct GatedPartialWriter<'a> {
        data: &'a mut Vec<u8>,
        gate: Option<(mpsc::Sender<()>, mpsc::Receiver<()>)>,
    }
    impl Write for GatedPartialWriter<'_> {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            let written = if let Some((arrived, release)) = self.gate.take() {
                let written = bytes.len().min(16);
                self.data.extend_from_slice(&bytes[..written]);
                arrived.send(()).unwrap();
                release.recv().unwrap();
                return Ok(written);
            } else {
                bytes.len()
            };
            self.data.extend_from_slice(&bytes[..written]);
            Ok(written)
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    #[test]
    fn per_record_lock_excludes_partial_logger_and_releases_while_waiting_for_next_event() {
        let data = Arc::new(Mutex::new(Vec::new()));
        let (arrived, waiting) = mpsc::channel();
        let (release, held) = mpsc::channel();
        let sink = data.clone();
        let mut gate = Some((arrived, held));
        let output = Output::start_record_writer(move |record| {
            let mut lock = sink.lock().unwrap();
            write_record(
                &mut GatedPartialWriter {
                    data: &mut lock,
                    gate: gate.take(),
                },
                record,
            )
        })
        .unwrap();
        let first = serde_json::json!({"selection":"Compound λ\n\"quoted\"".repeat(1024)});
        let second = serde_json::json!({"selection":"None"});
        output.submit(record(1, first.clone()));
        waiting
            .recv_timeout(std::time::Duration::from_secs(2))
            .unwrap();
        let (attempted, attempting) = mpsc::channel();
        let (complete, completed) = mpsc::channel();
        let logger_sink = data.clone();
        let logger = std::thread::spawn(move || {
            attempted.send(()).unwrap();
            logger_sink
                .lock()
                .unwrap()
                .extend_from_slice(b"renderer log\n");
            complete.send(()).unwrap();
        });
        attempting
            .recv_timeout(std::time::Duration::from_secs(2))
            .unwrap();
        assert!(completed.try_recv().is_err());
        release.send(()).unwrap();
        // No second event has been submitted: another logger must be able to
        // write while the evidence worker is waiting for its next queued record.
        completed
            .recv_timeout(std::time::Duration::from_secs(2))
            .unwrap();
        logger.join().unwrap();
        output.submit(record(2, second.clone()));
        output.finish();
        let mut expected = Vec::new();
        for (sequence, value) in [(1, first), (2, second)] {
            if sequence == 2 {
                expected.extend_from_slice(b"renderer log\n");
            }
            expected.extend_from_slice(b"DATUM_ACTION_EVIDENCE ");
            serde_json::to_writer(
                &mut expected,
                &serde_json::json!({
                    "event":"state", "kind":"datum.action-evidence/v1",
                    "sequence":sequence, "value":value
                }),
            )
            .unwrap();
            expected.push(b'\n');
        }
        assert_eq!(*data.lock().unwrap(), expected);
    }
}
