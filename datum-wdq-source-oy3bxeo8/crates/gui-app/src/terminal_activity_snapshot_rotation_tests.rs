use super::*;
use std::fs;

fn unique_path() -> PathBuf {
    std::env::temp_dir().join(format!(
        "datum-terminal-activity-rotation-{}-{:?}.events.jsonl",
        std::process::id(),
        std::thread::current().id()
    ))
}

#[test]
fn rotated_io_family_is_folded_chronologically_without_replay() {
    let metadata = unique_path();
    let current = crate::terminal_session_events::io_event_log::io_segment_path(&metadata, 0);
    let retained = crate::terminal_session_events::io_event_log::io_segment_path(&metadata, 1);
    for path in [&metadata, &current, &retained] {
        let _ = fs::remove_file(path);
    }

    fs::write(
        &metadata,
        concat!(
            r#"{"event":"terminal_command_handoff","command_id":"datum.check.run","occurred_unix_ms":1}"#,
            "\n",
            r#"{"event":"terminal_command_lifecycle","command_id":"datum.check.run","lifecycle":"finished","process_exit_code":0,"occurred_unix_ms":4}"#,
            "\n"
        ),
    )
    .expect("write durable metadata");
    fs::write(
        &current,
        concat!(
            r#"{"event":"terminal_io","direction":"input","byte_count":3,"text_preview":"go\\r","occurred_unix_ms":2}"#,
            "\n",
            r#"{"event":"terminal_io","direction":"output","byte_count":7,"text_preview":"done\\n","occurred_unix_ms":3}"#,
            "\n"
        ),
    )
    .expect("write current I/O segment");

    let mut cache = TerminalActivitySummaryCache::default();
    cache.refresh(&metadata);
    let initial = cache.summary_lines(4).expect("initial rotated summary");
    assert_eq!(initial.len(), 1);
    assert!(initial[0].contains("in:3B out:7B"));
    assert!(initial[0].contains("command:finished"));
    assert_eq!(cache.event_count(), 4);

    fs::rename(&current, &retained).expect("rotate current segment");
    fs::write(
        &current,
        concat!(
            r#"{"event":"terminal_io","direction":"input","byte_count":4,"text_preview":"next\\r","occurred_unix_ms":5}"#,
            "\n"
        ),
    )
    .expect("write replacement current segment");
    cache.refresh(&metadata);
    let after_rotation = cache.summary_lines(4).expect("summary after rollover");
    assert_eq!(after_rotation.len(), 2);
    assert!(after_rotation[1].contains("in:4B out:0B"));
    assert_eq!(cache.event_count(), 5, "retained segment must not replay");

    for path in [&metadata, &current, &retained] {
        let _ = fs::remove_file(path);
    }
}
