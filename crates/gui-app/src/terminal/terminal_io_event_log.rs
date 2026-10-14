use anyhow::{Context, Result};
use serde::Serialize;
use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    sync::Mutex,
};

pub(crate) const IO_SEGMENT_BYTES: u64 = 16 * 1024 * 1024;
pub(crate) const IO_SEGMENT_COUNT: usize = 4;

static IO_LOG_WRITE: Mutex<()> = Mutex::new(());

#[derive(Debug, Serialize)]
struct TerminalIoEvent<'a> {
    event: &'static str,
    schema_version: u64,
    session_id: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    execution_id: Option<&'a str>,
    direction: &'static str,
    byte_count: usize,
    text_preview: String,
    truncated: bool,
    occurred_unix_ms: u128,
}

#[derive(Debug, Serialize)]
struct TerminalIoRotationEvent<'a> {
    event: &'static str,
    schema_version: u64,
    session_id: &'a str,
    retained_segments: usize,
    segment_bytes: u64,
    occurred_unix_ms: u128,
}

pub(crate) fn append_terminal_io_event(
    metadata_path: &Path,
    session_id: &str,
    execution_id: Option<&str>,
    direction: &'static str,
    bytes: &[u8],
) -> Result<()> {
    append_terminal_io_event_with_limit(
        metadata_path,
        session_id,
        execution_id,
        direction,
        bytes,
        IO_SEGMENT_BYTES,
    )
}

fn append_terminal_io_event_with_limit(
    metadata_path: &Path,
    session_id: &str,
    execution_id: Option<&str>,
    direction: &'static str,
    bytes: &[u8],
    segment_bytes: u64,
) -> Result<()> {
    let occurred_unix_ms = unix_time_ms()?;
    let (text_preview, truncated) = terminal_text_preview(bytes);
    let event = TerminalIoEvent {
        event: "terminal_io",
        schema_version: 1,
        session_id,
        execution_id,
        direction,
        byte_count: bytes.len(),
        text_preview,
        truncated,
        occurred_unix_ms,
    };
    let mut encoded = serde_json::to_vec(&event).context("serialize terminal I/O event")?;
    encoded.push(b'\n');

    let _guard = IO_LOG_WRITE
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let current = io_segment_path(metadata_path, 0);
    let current_len = current
        .metadata()
        .map(|metadata| metadata.len())
        .unwrap_or(0);
    let rotated =
        current_len > 0 && current_len.saturating_add(encoded.len() as u64) > segment_bytes;
    if rotated {
        rotate_segments(metadata_path)?;
    }
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&current)
        .with_context(|| format!("open terminal I/O segment {}", current.display()))?;
    file.write_all(&encoded)
        .with_context(|| format!("append terminal I/O segment {}", current.display()))?;
    if rotated {
        append_rotation_fact(metadata_path, session_id, occurred_unix_ms)?;
    }
    Ok(())
}

fn rotate_segments(metadata_path: &Path) -> Result<()> {
    let oldest = io_segment_path(metadata_path, IO_SEGMENT_COUNT - 1);
    if oldest.exists() {
        fs::remove_file(&oldest)
            .with_context(|| format!("remove oldest terminal I/O segment {}", oldest.display()))?;
    }
    for index in (0..IO_SEGMENT_COUNT - 1).rev() {
        let source = io_segment_path(metadata_path, index);
        if source.exists() {
            let destination = io_segment_path(metadata_path, index + 1);
            fs::rename(&source, &destination).with_context(|| {
                format!(
                    "rotate terminal I/O segment {} to {}",
                    source.display(),
                    destination.display()
                )
            })?;
        }
    }
    Ok(())
}

fn append_rotation_fact(path: &Path, session_id: &str, occurred_unix_ms: u128) -> Result<()> {
    let event = TerminalIoRotationEvent {
        event: "terminal_io_rotation",
        schema_version: 1,
        session_id,
        retained_segments: IO_SEGMENT_COUNT,
        segment_bytes: IO_SEGMENT_BYTES,
        occurred_unix_ms,
    };
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .with_context(|| format!("open terminal metadata log {}", path.display()))?;
    writeln!(
        file,
        "{}",
        serde_json::to_string(&event).context("serialize terminal I/O rotation fact")?
    )
    .with_context(|| format!("append terminal I/O rotation fact {}", path.display()))
}

pub(crate) fn io_segment_path(metadata_path: &Path, index: usize) -> PathBuf {
    let file_name = metadata_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("terminal.events.jsonl");
    metadata_path.with_file_name(format!("{file_name}.io.{index}.jsonl"))
}

pub(crate) fn io_segment_paths_oldest_first(metadata_path: &Path) -> Vec<PathBuf> {
    (0..IO_SEGMENT_COUNT)
        .rev()
        .map(|index| io_segment_path(metadata_path, index))
        .filter(|path| path.is_file())
        .collect()
}

#[cfg(test)]
pub(crate) fn read_event_log_family_text(metadata_path: &Path) -> String {
    let mut events = Vec::new();
    let paths = std::iter::once(metadata_path.to_path_buf())
        .chain(io_segment_paths_oldest_first(metadata_path));
    for path in paths {
        let Ok(text) = fs::read_to_string(path) else {
            continue;
        };
        for line in text.lines().filter(|line| !line.trim().is_empty()) {
            if let Ok(value) = serde_json::from_str::<serde_json::Value>(line) {
                let timestamp = value
                    .get("occurred_unix_ms")
                    .and_then(serde_json::Value::as_u64)
                    .unwrap_or(0);
                events.push((timestamp, line.to_string()));
            }
        }
    }
    events.sort_by_key(|(timestamp, _)| *timestamp);
    events.into_iter().map(|(_, line)| line + "\n").collect()
}

fn terminal_text_preview(bytes: &[u8]) -> (String, bool) {
    const MAX_PREVIEW_CHARS: usize = 512;
    let text = String::from_utf8_lossy(bytes);
    let mut preview = String::new();
    let mut truncated = false;
    for (index, ch) in text.chars().enumerate() {
        if index == MAX_PREVIEW_CHARS {
            truncated = true;
            break;
        }
        preview.push(ch);
    }
    (preview, truncated || text.len() != bytes.len())
}

fn unix_time_ms() -> Result<u128> {
    Ok(std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .context("terminal I/O event timestamp")?
        .as_millis())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unique_path(label: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "datum-terminal-{label}-{}-{}.events.jsonl",
            std::process::id(),
            unix_time_ms().unwrap()
        ))
    }

    #[test]
    fn io_preview_is_bounded_and_opaque_byte_count_is_exact() {
        let metadata = unique_path("io-preview");
        append_terminal_io_event(
            &metadata,
            "session",
            Some("execution"),
            "output",
            &vec![b'a'; 600],
        )
        .unwrap();
        let text = fs::read_to_string(io_segment_path(&metadata, 0)).unwrap();
        let event: serde_json::Value = serde_json::from_str(text.trim()).unwrap();
        assert_eq!(event["byte_count"], 600);
        assert_eq!(event["text_preview"].as_str().unwrap().len(), 512);
        assert_eq!(event["truncated"], true);
        let _ = fs::remove_file(io_segment_path(&metadata, 0));
    }

    #[test]
    fn four_segments_rotate_oldest_first_and_metadata_records_the_fact() {
        let metadata = unique_path("io-rotation");
        for ordinal in 0..6 {
            append_terminal_io_event_with_limit(
                &metadata,
                "session",
                None,
                "output",
                format!("record-{ordinal}-padding").as_bytes(),
                1,
            )
            .unwrap();
        }
        let paths = io_segment_paths_oldest_first(&metadata);
        assert_eq!(paths.len(), IO_SEGMENT_COUNT);
        let combined = paths
            .iter()
            .map(|path| fs::read_to_string(path).unwrap())
            .collect::<String>();
        assert!(!combined.contains("record-0-padding"));
        assert!(!combined.contains("record-1-padding"));
        for ordinal in 2..6 {
            assert!(combined.contains(&format!("record-{ordinal}-padding")));
        }
        let rotations = fs::read_to_string(&metadata).unwrap();
        assert_eq!(rotations.matches("terminal_io_rotation").count(), 5);
        for path in paths {
            let _ = fs::remove_file(path);
        }
        let _ = fs::remove_file(metadata);
    }
}
