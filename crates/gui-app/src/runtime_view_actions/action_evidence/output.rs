//! Lossless opt-in evidence output; encoding/I/O never run in input dispatch.
use serde_json::Value;
use std::io::Write;
use std::sync::mpsc;
use std::thread::JoinHandle;

pub(super) struct Output {
    sender: Option<mpsc::Sender<Value>>,
    worker: Option<JoinHandle<()>>,
}
impl Output {
    pub(super) fn start(writer: impl Write + Send + 'static) -> std::io::Result<Self> {
        // The diagnostic FIFO is lossless: it has no blocking send, record cap,
        // sampling or coalescing of required events. Ordinary launches create
        // neither this worker nor this queue. Complete owned records are queued.
        let (sender, receiver) = mpsc::channel();
        let worker = std::thread::Builder::new()
            .name("datum-action-evidence".into())
            .spawn(move || write_records(writer, receiver))?;
        Ok(Self {
            sender: Some(sender),
            worker: Some(worker),
        })
    }
    pub(super) fn submit(&self, record: Value) {
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

fn write_records(mut writer: impl Write, receiver: mpsc::Receiver<Value>) {
    for record in receiver {
        let mut line = b"DATUM_ACTION_EVIDENCE ".to_vec();
        let encoded = {
            let _probe = crate::gui_runtime_support::phase_probe::Probe::start("evidence_encode");
            serde_json::to_writer(&mut line, &record)
        };
        if encoded.is_err() {
            continue;
        }
        line.push(b'\n');
        let _probe = crate::gui_runtime_support::phase_probe::Probe::start("evidence_write");
        if writer.write_all(&line).is_err() {
            // An unwritable diagnostic sink cannot preserve further records.
            // Stop consuming CPU; subsequent sends fail without blocking input.
            break;
        }
    }
    let _ = writer.flush();
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};
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
            "sequence": sequence, "value": {"selection": "Run: complete\n\"52\" λ", "members": [1,2,3]}
        })).collect();
        output.submit(records[0].clone());
        waiting
            .recv_timeout(std::time::Duration::from_secs(2))
            .unwrap();
        // The sink remains held while all later native-event records enqueue.
        output.submit(records[1].clone());
        output.submit(records[2].clone());
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
        output.submit(serde_json::json!({"sequence":1}));
        output.submit(serde_json::json!({"sequence":2}));
        output.finish();
    }
}
