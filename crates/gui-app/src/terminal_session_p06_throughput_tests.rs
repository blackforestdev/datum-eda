use crate::terminal_transport::{
    MAX_OUTPUT_CHUNK_BYTES, TerminalTransportEvent, TerminalTransportRequest,
    TerminalTransportSession, TerminalWakeGate, prepare_terminal_transport,
};
use serde::Serialize;
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

const SESSION_COUNT: usize = 8;
const BLOCK_BYTES: usize = 256 * 1024;
const OUTPUT_CAPACITY_BYTES: usize = 4 * 1024 * 1024;
const RUN_DURATION: Duration = Duration::from_secs(60);
const STALL: Duration = Duration::from_secs(30);

#[derive(Serialize)]
struct SustainedRun {
    duration_us: u64,
    input_bytes: u64,
    output_bytes: u64,
    mib_per_second: f64,
    per_session_bytes: Vec<u64>,
    checksums: Vec<u64>,
    max_fairness_gap_us: u64,
}

#[derive(Serialize)]
struct BacklogRecovery {
    high_water_bytes: usize,
    below_64_kib_us: u64,
    zero_us: u64,
}

#[derive(Serialize)]
struct ThroughputEvidence {
    contract: &'static str,
    revision: String,
    seed: u64,
    run_ordinal: u8,
    occurred_unix_ms: u128,
    display_backend: String,
    single_session: SustainedRun,
    eight_session_aggregate: SustainedRun,
    backlog_recovery: BacklogRecovery,
    failures: Vec<String>,
}

fn spawn_echo() -> TerminalTransportSession {
    let session = prepare_terminal_transport(
        TerminalTransportRequest::new("/bin/sh", PathBuf::from("/tmp"))
            .args(["-c", "stty raw -echo; printf R; exec cat"]),
    )
    .expect("prepare sustained PTY")
    .start(TerminalWakeGate::new(None));
    let deadline = Instant::now() + STALL;
    while Instant::now() < deadline {
        match session.recv_event_timeout(Duration::from_millis(20)) {
            Ok(TerminalTransportEvent::Output(bytes)) if bytes.contains(&b'R') => return session,
            Ok(TerminalTransportEvent::Output(_)) => {}
            Ok(other) => panic!("sustained PTY setup event: {other:?}"),
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {}
            Err(error) => panic!("sustained PTY setup disconnected: {error:?}"),
        }
    }
    panic!("sustained PTY setup timed out");
}

fn payload(seed: u64, session: usize, cycle: u64) -> Vec<u8> {
    let mut state = seed ^ (session as u64).wrapping_mul(0x9e37_79b9) ^ cycle.rotate_left(17);
    (0..BLOCK_BYTES)
        .map(|_| {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            state as u8
        })
        .collect()
}

fn enqueue(session: &TerminalTransportSession, bytes: &[u8]) {
    let deadline = Instant::now() + STALL;
    loop {
        match session.write_bytes(bytes) {
            Ok(()) => return,
            Err(crate::terminal_transport::TerminalInputError::Busy) => std::thread::yield_now(),
            Err(error) => panic!("sustained input admission failed: {error}"),
        }
        assert!(
            Instant::now() < deadline,
            "sustained input admission stalled"
        );
    }
}

fn close_all(sessions: &[TerminalTransportSession]) {
    let deadline = Instant::now() + Duration::from_secs(7);
    for session in sessions {
        session
            .terminate_by(deadline)
            .expect("terminate sustained PTY");
    }
    while Instant::now() < deadline
        && !sessions
            .iter()
            .all(TerminalTransportSession::presentation_complete)
    {
        for session in sessions {
            while let Ok(event) = session.try_recv_event() {
                if let TerminalTransportEvent::Error(error) = event {
                    panic!("sustained shutdown error: {error:?}");
                }
            }
        }
        std::thread::yield_now();
    }
    assert!(
        sessions
            .iter()
            .all(TerminalTransportSession::presentation_complete),
        "sustained sessions did not close"
    );
}

fn sustained(seed: u64, session_count: usize) -> SustainedRun {
    let sessions = (0..session_count).map(|_| spawn_echo()).collect::<Vec<_>>();
    let started = Instant::now();
    let mut per_session_bytes = vec![0u64; session_count];
    let mut checksums = vec![0xcbf29ce484222325; session_count];
    let mut last_progress = vec![started; session_count];
    let mut max_fairness_gap = Duration::ZERO;
    let mut cycle = 0u64;
    while started.elapsed() < RUN_DURATION {
        let expected = (0..session_count)
            .map(|index| payload(seed, index, cycle))
            .collect::<Vec<_>>();
        for (session, bytes) in sessions.iter().zip(&expected) {
            enqueue(session, bytes);
        }
        let deadline = Instant::now() + STALL;
        let mut received = vec![Vec::new(); session_count];
        while Instant::now() < deadline
            && received
                .iter()
                .zip(&expected)
                .any(|(actual, wanted)| actual.len() < wanted.len())
        {
            let mut progressed = false;
            for (index, session) in sessions.iter().enumerate() {
                while let Ok(event) = session.try_recv_event() {
                    progressed = true;
                    match event {
                        TerminalTransportEvent::Output(bytes) => {
                            let now = Instant::now();
                            max_fairness_gap = max_fairness_gap.max(now - last_progress[index]);
                            last_progress[index] = now;
                            received[index].extend(bytes);
                        }
                        other => panic!("sustained data event: {other:?}"),
                    }
                }
            }
            if !progressed {
                std::thread::yield_now();
            }
        }
        for (index, (actual, wanted)) in received.iter().zip(&expected).enumerate() {
            assert_eq!(actual, wanted, "sustained session {index} exact stream");
            per_session_bytes[index] += actual.len() as u64;
            checksums[index] = fnv_extend(checksums[index], actual);
        }
        cycle += 1;
    }
    let duration = started.elapsed();
    close_all(&sessions);
    let output_bytes = per_session_bytes.iter().sum::<u64>();
    SustainedRun {
        duration_us: duration_us(duration),
        input_bytes: output_bytes,
        output_bytes,
        mib_per_second: output_bytes as f64 / (1024.0 * 1024.0) / duration.as_secs_f64(),
        per_session_bytes,
        checksums,
        max_fairness_gap_us: duration_us(max_fairness_gap),
    }
}

fn backlog_recovery() -> BacklogRecovery {
    let session = prepare_terminal_transport(
        TerminalTransportRequest::new("/bin/sh", PathBuf::from("/tmp"))
            .args(["-c", "head -c 4194304 /dev/zero | tr '\\000' B"]),
    )
    .expect("prepare backlog PTY")
    .start(TerminalWakeGate::new(None));
    let fill_deadline = Instant::now() + STALL;
    while session.output_queued_bytes_for_test() < OUTPUT_CAPACITY_BYTES
        && Instant::now() < fill_deadline
    {
        std::thread::yield_now();
    }
    let high_water_bytes = session.output_queued_bytes_for_test();
    assert_eq!(high_water_bytes, OUTPUT_CAPACITY_BYTES);
    let started = Instant::now();
    let mut received = 0usize;
    let mut below_64_kib = None;
    while received < OUTPUT_CAPACITY_BYTES && started.elapsed() < STALL {
        match session.recv_event_timeout(Duration::from_millis(20)) {
            Ok(TerminalTransportEvent::Output(bytes)) => {
                assert!(bytes.iter().all(|byte| *byte == b'B'));
                received += bytes.len();
                if OUTPUT_CAPACITY_BYTES - received < 64 * 1024 {
                    below_64_kib.get_or_insert_with(|| started.elapsed());
                }
            }
            Ok(TerminalTransportEvent::Exited(_)) => {}
            Ok(TerminalTransportEvent::Error(error)) => panic!("backlog error: {error:?}"),
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {}
            Err(error) => panic!("backlog disconnected: {error:?}"),
        }
    }
    assert_eq!(received, OUTPUT_CAPACITY_BYTES);
    let zero = started.elapsed();
    while !session.presentation_complete() && started.elapsed() < STALL {
        let _ = session.recv_event_timeout(Duration::from_millis(20));
    }
    BacklogRecovery {
        high_water_bytes,
        below_64_kib_us: duration_us(below_64_kib.expect("backlog below 64 KiB")),
        zero_us: duration_us(zero),
    }
}

fn duration_us(duration: Duration) -> u64 {
    duration.as_micros().min(u128::from(u64::MAX)) as u64
}

fn fnv_extend(mut state: u64, bytes: &[u8]) -> u64 {
    for byte in bytes {
        state ^= u64::from(*byte);
        state = state.wrapping_mul(0x100000001b3);
    }
    state
}

fn command_output(program: &str, args: &[&str]) -> String {
    Command::new(program)
        .args(args)
        .output()
        .ok()
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
        .unwrap_or_else(|| "unavailable".to_string())
}

fn evidence_path() -> PathBuf {
    std::env::var_os("DATUM_P06_EVIDENCE")
        .map(PathBuf::from)
        .unwrap_or_else(|| Path::new("target/p06-evidence").join("throughput-60s.json"))
}

#[test]
fn p06_sustained_throughput_budgets_are_literal() {
    assert_eq!(RUN_DURATION, Duration::from_secs(60));
    assert_eq!(SESSION_COUNT, 8);
    assert_eq!(OUTPUT_CAPACITY_BYTES, 4 << 20);
    assert_eq!(MAX_OUTPUT_CHUNK_BYTES, 16 << 10);
}

#[test]
#[ignore = "DTC-P06D sustained release measurement; run through the proof gate runner"]
fn p06_sustained_throughput_and_backlog_emit_reproducible_json() {
    assert!(
        !std::hint::black_box(cfg!(debug_assertions)),
        "P06 throughput evidence must use --release"
    );
    let seed = std::env::var("DATUM_P06_SEED")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(0xD06D_2026_0818);
    let run_ordinal = std::env::var("DATUM_P06_RUN_ORDINAL")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(1);
    let evidence = ThroughputEvidence {
        contract: "datum_terminal_p06_sustained_v1",
        revision: command_output("git", &["rev-parse", "HEAD"]),
        seed,
        run_ordinal,
        occurred_unix_ms: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time after epoch")
            .as_millis(),
        display_backend: std::env::var("XDG_SESSION_TYPE").unwrap_or_else(|_| "unknown".into()),
        single_session: sustained(seed, 1),
        eight_session_aggregate: sustained(seed, SESSION_COUNT),
        backlog_recovery: backlog_recovery(),
        failures: Vec::new(),
    };
    let path = evidence_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create sustained evidence directory");
    }
    fs::write(
        &path,
        serde_json::to_vec_pretty(&evidence).expect("serialize sustained evidence"),
    )
    .expect("write sustained evidence");
    println!("DTC-P06D sustained evidence: {}", path.display());
}
