use std::time::Instant;

use datum_terminal_core::{
    Action, CoreLimitValues, CoreLimits, StreamingParser, TerminalCore, TerminalSize,
};

const DEFAULT_PAYLOAD_BYTES: usize = 8 * 1024 * 1024;
const FEED_BYTES: usize = 64 * 1024;
const LINE: &[u8] = b"\x1b[38;2;88;166;255mDTC-P21\x1b[0m opaque terminal proof line \xf0\x9f\x91\xa9\xe2\x80\x8d\xf0\x9f\x92\xbb\r\n";

fn limits() -> CoreLimits {
    let value = 1_048_576;
    CoreLimits::try_from(CoreLimitValues {
        parameter_count: value,
        parameter_digits: value,
        parameter_value: value,
        subparameter_count: value,
        intermediate_bytes: value,
        control_string_bytes: value,
        cluster_bytes: value,
        title_bytes: value,
        working_directory_bytes: value,
        clipboard_bytes: value,
        hyperlink_bytes: value,
        input_bytes: value,
        keyboard_stack: value,
        notification_bytes: value,
        reply_bytes: value,
        pending_events: value,
        pending_damage: value,
        history_lines: 4_096,
        history_bytes: 8 * 1024 * 1024,
        graphic_objects: 64,
        graphic_pixels: 4 * 1024 * 1024,
        graphic_decoded_bytes: 16 * 1024 * 1024,
        graphic_frames: 256,
        compression_ratio: 1_024,
        parser_work: value,
        search_work: value,
        reflow_work: value,
        screen_cells: value,
        snapshot_cells: value,
    })
    .expect("probe limits are valid")
}

fn apply_actions(
    core: &mut TerminalCore,
    actions: Vec<Action>,
    action_count: &mut usize,
    reply_count: &mut usize,
    event_count: &mut usize,
    error_count: &mut usize,
) {
    for action in actions {
        *action_count += 1;
        match core.apply(action) {
            Ok(update) => {
                *reply_count += update.replies().len();
                *event_count += update.events().len();
            }
            Err(_) => *error_count += 1,
        }
    }
}

fn main() {
    let payload_bytes = std::env::var("DTC_P21_PROBE_BYTES")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(DEFAULT_PAYLOAD_BYTES);
    let limits = limits();
    let mut parser = StreamingParser::new(limits);
    let mut core = TerminalCore::new(
        limits,
        TerminalSize::new(120, 40, 1_200, 800).expect("probe geometry is valid"),
    )
    .expect("probe core is valid");
    let mut feed = Vec::with_capacity(FEED_BYTES);
    while feed.len() + LINE.len() <= FEED_BYTES {
        feed.extend_from_slice(LINE);
    }
    feed.resize(FEED_BYTES, b' ');

    let mut action_count = 0;
    let mut reply_count = 0;
    let mut event_count = 0;
    let mut error_count = 0;
    let started = Instant::now();
    let mut supplied = 0;
    while supplied < payload_bytes {
        let length = (payload_bytes - supplied).min(feed.len());
        let mut input = &feed[..length];
        while !input.is_empty() {
            let mut actions = Vec::new();
            let report = parser.feed(input, |action| actions.push(action));
            assert!(
                report.consumed > 0,
                "release probe parser stopped making progress"
            );
            apply_actions(
                &mut core,
                actions,
                &mut action_count,
                &mut reply_count,
                &mut event_count,
                &mut error_count,
            );
            input = &input[report.consumed..];
        }
        supplied += length;
    }
    let mut actions = Vec::new();
    parser.finish(|action| actions.push(action));
    apply_actions(
        &mut core,
        actions,
        &mut action_count,
        &mut reply_count,
        &mut event_count,
        &mut error_count,
    );
    let snapshot = core
        .render_snapshot()
        .expect("release probe snapshot is bounded");
    let elapsed = started.elapsed();
    let elapsed_ns = elapsed.as_nanos();
    let mib_per_second = payload_bytes as f64 / (1024.0 * 1024.0) / elapsed.as_secs_f64();
    println!(
        "{{\"schema\":\"datum-terminal-core-proof-v1\",\"payload_bytes\":{payload_bytes},\"actions\":{action_count},\"replies\":{reply_count},\"events\":{event_count},\"errors\":{error_count},\"elapsed_ns\":{elapsed_ns},\"mib_per_second\":{mib_per_second:.3},\"history_rows\":{},\"snapshot_rows\":{}}}",
        snapshot.history_row_count(),
        snapshot.rows().count(),
    );
}
