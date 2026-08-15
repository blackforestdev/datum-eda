use super::{TerminalTransportEvent, TerminalWakeGate};
use std::{
    fs::File,
    io::Read,
    process::Child,
    sync::mpsc::{self, Receiver, Sender},
    thread,
};

pub(super) fn spawn_event_threads(
    reader: File,
    mut child: Child,
    wake: TerminalWakeGate,
) -> Receiver<TerminalTransportEvent> {
    let (tx, rx) = mpsc::channel();
    let reader_tx = tx.clone();
    let reader_wake = wake.clone();
    thread::spawn(move || read_output(reader, reader_tx, reader_wake));
    thread::spawn(move || {
        let code = child.wait().ok().and_then(|status| status.code());
        publish_event(&tx, TerminalTransportEvent::Exited(code), || wake.request());
    });
    rx
}

fn read_output(mut reader: File, tx: Sender<TerminalTransportEvent>, wake: TerminalWakeGate) {
    let mut buffer = [0_u8; 4096];
    loop {
        match reader.read(&mut buffer) {
            Ok(0) | Err(_) => break,
            Ok(count) => publish_event(
                &tx,
                TerminalTransportEvent::Output(buffer[..count].to_vec()),
                || wake.request(),
            ),
        }
    }
}

/// Publish before waking so the GUI can never observe an empty wake.
fn publish_event(
    tx: &Sender<TerminalTransportEvent>,
    event: TerminalTransportEvent,
    wake: impl FnOnce(),
) {
    if tx.send(event).is_ok() {
        wake();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        fs::OpenOptions,
        io::{Seek, SeekFrom, Write},
        sync::atomic::{AtomicUsize, Ordering},
        time::{SystemTime, UNIX_EPOCH},
    };

    #[test]
    fn every_published_event_wakes_the_consumer() {
        let (tx, rx) = mpsc::channel();
        let wakes = AtomicUsize::new(0);
        publish_event(&tx, TerminalTransportEvent::Output(vec![1, 2]), || {
            wakes.fetch_add(1, Ordering::SeqCst);
        });
        publish_event(&tx, TerminalTransportEvent::Exited(Some(0)), || {
            wakes.fetch_add(1, Ordering::SeqCst);
        });
        assert!(matches!(rx.recv(), Ok(TerminalTransportEvent::Output(bytes)) if bytes == [1, 2]));
        assert!(matches!(
            rx.recv(),
            Ok(TerminalTransportEvent::Exited(Some(0)))
        ));
        assert_eq!(wakes.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn reader_preserves_nul_escape_and_invalid_utf8_bytes_exactly() {
        let opaque = vec![
            0x00, 0x1b, b'[', b'3', b'1', b'm', 0x1b, b']', b'8', b';', b';', 0xff, 0xfe,
        ];
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock after epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "datum-terminal-reader-{}-{nonce}.bin",
            std::process::id()
        ));
        let mut reader = OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(true)
            .open(&path)
            .expect("create byte stream");
        reader.write_all(&opaque).expect("write byte stream");
        reader.seek(SeekFrom::Start(0)).expect("rewind byte stream");

        let (tx, rx) = mpsc::channel();
        read_output(reader, tx, TerminalWakeGate::new(None));
        std::fs::remove_file(path).expect("remove byte stream");
        let actual = rx
            .try_iter()
            .flat_map(|event| match event {
                TerminalTransportEvent::Output(bytes) => bytes,
                TerminalTransportEvent::Exited(_) => Vec::new(),
            })
            .collect::<Vec<_>>();
        assert_eq!(actual, opaque);
    }

    #[test]
    fn disconnected_consumer_does_not_schedule_a_spurious_wake() {
        let (tx, rx) = mpsc::channel();
        drop(rx);
        let wakes = AtomicUsize::new(0);
        publish_event(&tx, TerminalTransportEvent::Exited(Some(0)), || {
            wakes.fetch_add(1, Ordering::SeqCst);
        });
        assert_eq!(wakes.load(Ordering::SeqCst), 0);
    }
}
