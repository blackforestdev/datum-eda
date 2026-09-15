use crate::terminal_transport::{
    ShutdownPhase, TerminalExitStatus, TerminalTransportEvent, TerminalTransportRequest,
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
const LATENCY_SAMPLES: usize = 40;
const RESIZE_SAMPLES: usize = 500;
const EVENT_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Clone, Copy)]
struct Tier {
    name: &'static str,
    output_bytes_per_session: usize,
    input_bytes_per_session: usize,
    latency_samples: usize,
    resize_samples: usize,
}

impl Tier {
    fn from_environment() -> Self {
        match std::env::var("DATUM_P06_TIER").as_deref() {
            Ok("smoke") | Err(_) => Self {
                name: "smoke",
                output_bytes_per_session: 1024 * 1024,
                input_bytes_per_session: 256 * 1024,
                latency_samples: LATENCY_SAMPLES,
                resize_samples: RESIZE_SAMPLES,
            },
            Ok(other) => panic!("unsupported P06 measurement tier {other:?}"),
        }
    }
}

#[derive(Serialize)]
struct HostEvidence {
    revision: String,
    kernel: String,
    libc: String,
    cpu: String,
    memory_kib: u64,
    architecture: String,
    display_backend: String,
    wayland_display: Option<String>,
    x11_display: Option<String>,
    build_profile: &'static str,
}

#[derive(Serialize)]
struct Distribution {
    count: usize,
    min_us: u64,
    p50_us: u64,
    p95_us: u64,
    p99_us: u64,
    max_us: u64,
    raw_us: Vec<u64>,
}

#[derive(Serialize)]
struct ThroughputEvidence {
    bytes: u64,
    elapsed_us: u64,
    mib_per_second: f64,
    checksum: u64,
}

#[derive(Serialize)]
struct ResourceSample {
    label: &'static str,
    rss_kib: u64,
    file_descriptors: usize,
    threads: usize,
}

#[derive(Serialize)]
struct MeasurementEvidence {
    contract: &'static str,
    tier: &'static str,
    seed: u64,
    occurred_unix_ms: u128,
    host: HostEvidence,
    sessions: usize,
    output_bytes_per_session: usize,
    input_bytes_per_session: usize,
    idle_latency: Distribution,
    resize_latency: Distribution,
    single_output: ThroughputEvidence,
    aggregate_output: ThroughputEvidence,
    input_roundtrip: ThroughputEvidence,
    resources: Vec<ResourceSample>,
    failures: Vec<String>,
}

fn spawn_script(script: &str) -> TerminalTransportSession {
    prepare_terminal_transport(
        TerminalTransportRequest::new("/bin/sh", PathBuf::from("/tmp")).args(["-c", script]),
    )
    .expect("prepare measured PTY")
    .start(TerminalWakeGate::new(None))
}

fn pattern_script(bytes: usize, byte: u8) -> String {
    format!(
        "head -c {bytes} /dev/zero | tr '\\000' '{}'",
        char::from(byte)
    )
}

fn finish_session(session: &TerminalTransportSession) {
    let deadline = Instant::now() + EVENT_TIMEOUT;
    while Instant::now() < deadline {
        if session.presentation_complete() {
            let snapshot = session.shutdown_snapshot().expect("shutdown snapshot");
            assert_eq!(snapshot.phase, ShutdownPhase::Closed);
            assert!(snapshot.leader_reaped);
            assert!(snapshot.surviving_processes.is_empty());
            return;
        }
        let _ = session.recv_event_timeout(Duration::from_millis(10));
    }
    panic!("measured PTY did not reach presentation completion");
}

fn collect_output(
    session: &TerminalTransportSession,
    expected: usize,
    byte: u8,
) -> (u64, Duration) {
    let deadline = Instant::now() + EVENT_TIMEOUT;
    let mut bytes = 0usize;
    let mut checksum = fnv_offset();
    let mut first = None;
    let mut last = None;
    let mut exit = None;
    while Instant::now() < deadline {
        match session.recv_event_timeout(Duration::from_millis(20)) {
            Ok(TerminalTransportEvent::Output(chunk)) => {
                let now = Instant::now();
                first.get_or_insert(now);
                last = Some(now);
                assert!(chunk.iter().all(|value| *value == byte));
                bytes += chunk.len();
                checksum = fnv_extend(checksum, &chunk);
            }
            Ok(TerminalTransportEvent::Exited(status)) => exit = Some(status),
            Ok(TerminalTransportEvent::Error(error)) => panic!("measured output error: {error:?}"),
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {}
            Err(error) => panic!("measured output disconnected: {error:?}"),
        }
        if bytes == expected && exit.is_some() && session.presentation_complete() {
            break;
        }
    }
    assert_eq!(bytes, expected);
    assert_eq!(exit, Some(TerminalExitStatus::Code(0)));
    let elapsed = last
        .expect("last output timestamp")
        .saturating_duration_since(first.expect("first output timestamp"));
    (checksum, elapsed.max(Duration::from_nanos(1)))
}

fn measure_single_output(bytes: usize) -> ThroughputEvidence {
    let session = spawn_script(&pattern_script(bytes, b'S'));
    let (checksum, elapsed) = collect_output(&session, bytes, b'S');
    throughput(bytes, elapsed, checksum)
}

fn measure_aggregate_output(bytes_per_session: usize) -> (ThroughputEvidence, ResourceSample) {
    let sessions = (0..SESSION_COUNT)
        .map(|index| spawn_script(&pattern_script(bytes_per_session, b'A' + index as u8)))
        .collect::<Vec<_>>();
    let peak_resources = resource_sample("eight_session_peak");
    let start = Instant::now();
    let mut counts = [0usize; SESSION_COUNT];
    let mut checksums = [fnv_offset(); SESSION_COUNT];
    let mut exits = [false; SESSION_COUNT];
    let deadline = start + EVENT_TIMEOUT;
    while Instant::now() < deadline {
        let mut progressed = false;
        for (index, session) in sessions.iter().enumerate() {
            while let Ok(event) = session.try_recv_event() {
                progressed = true;
                match event {
                    TerminalTransportEvent::Output(chunk) => {
                        assert!(chunk.iter().all(|byte| *byte == b'A' + index as u8));
                        counts[index] += chunk.len();
                        checksums[index] = fnv_extend(checksums[index], &chunk);
                    }
                    TerminalTransportEvent::Exited(status) => {
                        assert_eq!(status, TerminalExitStatus::Code(0));
                        exits[index] = true;
                    }
                    TerminalTransportEvent::Error(error) => {
                        panic!("aggregate session {index} error: {error:?}")
                    }
                }
            }
        }
        if counts.iter().all(|count| *count == bytes_per_session)
            && exits.iter().all(|exited| *exited)
            && sessions
                .iter()
                .all(TerminalTransportSession::presentation_complete)
        {
            break;
        }
        if !progressed {
            std::thread::yield_now();
        }
    }
    for (index, count) in counts.iter().enumerate() {
        assert_eq!(*count, bytes_per_session, "aggregate session {index}");
    }
    assert!(exits.iter().all(|exited| *exited));
    let checksum = checksums.into_iter().fold(fnv_offset(), |state, value| {
        fnv_extend(state, &value.to_le_bytes())
    });
    let evidence = throughput(bytes_per_session * SESSION_COUNT, start.elapsed(), checksum);
    drop(sessions);
    (evidence, peak_resources)
}

fn start_echo_session() -> TerminalTransportSession {
    let session = spawn_script("stty raw -echo; printf R; exec cat");
    let deadline = Instant::now() + EVENT_TIMEOUT;
    while Instant::now() < deadline {
        match session.recv_event_timeout(Duration::from_millis(20)) {
            Ok(TerminalTransportEvent::Output(bytes)) if bytes.contains(&b'R') => return session,
            Ok(TerminalTransportEvent::Output(_)) => {}
            Ok(other) => panic!("echo setup failed: {other:?}"),
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {}
            Err(error) => panic!("echo setup disconnected: {error:?}"),
        }
    }
    panic!("echo session did not become ready");
}

fn measure_latency_and_input(
    latency_samples: usize,
    input_bytes: usize,
) -> (Distribution, ThroughputEvidence, TerminalTransportSession) {
    let session = start_echo_session();
    let mut samples = Vec::with_capacity(latency_samples);
    for index in 0..latency_samples {
        let marker = format!("L{index:08x}!");
        let started = Instant::now();
        session
            .write_bytes(marker.as_bytes())
            .expect("enqueue latency marker");
        let mut received = Vec::new();
        while received.len() < marker.len() {
            match session.recv_event_timeout(Duration::from_millis(20)) {
                Ok(TerminalTransportEvent::Output(bytes)) => received.extend(bytes),
                Ok(other) => panic!("latency probe event: {other:?}"),
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                    assert!(
                        started.elapsed() < EVENT_TIMEOUT,
                        "latency marker timed out"
                    )
                }
                Err(error) => panic!("latency probe disconnected: {error:?}"),
            }
        }
        assert_eq!(received, marker.as_bytes());
        samples.push(duration_us(started.elapsed()));
    }

    let payload = (0..input_bytes)
        .map(|index| (index as u8).wrapping_mul(31).wrapping_add(17))
        .collect::<Vec<_>>();
    let expected_checksum = fnv_extend(fnv_offset(), &payload);
    let started = Instant::now();
    session
        .write_bytes(&payload)
        .expect("enqueue measured input payload");
    let mut received = Vec::with_capacity(payload.len());
    while received.len() < payload.len() {
        match session.recv_event_timeout(Duration::from_millis(20)) {
            Ok(TerminalTransportEvent::Output(bytes)) => received.extend(bytes),
            Ok(other) => panic!("input roundtrip event: {other:?}"),
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                assert!(
                    started.elapsed() < EVENT_TIMEOUT,
                    "input roundtrip timed out"
                )
            }
            Err(error) => panic!("input roundtrip disconnected: {error:?}"),
        }
    }
    assert_eq!(received, payload);
    let elapsed = started.elapsed();
    let checksum = fnv_extend(fnv_offset(), &received);
    assert_eq!(checksum, expected_checksum);
    (
        distribution(samples),
        throughput(input_bytes, elapsed, checksum),
        session,
    )
}

fn measure_resize(session: &TerminalTransportSession, count: usize) -> Distribution {
    let samples = (0..count)
        .map(|index| {
            let started = Instant::now();
            session
                .resize(80 + (index % 40) as u16, 20 + (index % 20) as u16)
                .expect("resize measured PTY");
            duration_us(started.elapsed())
        })
        .collect();
    distribution(samples)
}

fn close_echo_session(session: &TerminalTransportSession) {
    session.terminate().expect("terminate echo session");
    finish_session(session);
}

fn distribution(mut samples: Vec<u64>) -> Distribution {
    assert!(!samples.is_empty());
    samples.sort_unstable();
    Distribution {
        count: samples.len(),
        min_us: samples[0],
        p50_us: percentile(&samples, 50),
        p95_us: percentile(&samples, 95),
        p99_us: percentile(&samples, 99),
        max_us: *samples.last().unwrap(),
        raw_us: samples,
    }
}

fn percentile(samples: &[u64], percentile: usize) -> u64 {
    let index = (samples.len() * percentile).div_ceil(100).saturating_sub(1);
    samples[index.min(samples.len() - 1)]
}

fn throughput(bytes: usize, elapsed: Duration, checksum: u64) -> ThroughputEvidence {
    ThroughputEvidence {
        bytes: bytes as u64,
        elapsed_us: duration_us(elapsed),
        mib_per_second: bytes as f64 / (1024.0 * 1024.0) / elapsed.as_secs_f64(),
        checksum,
    }
}

fn duration_us(duration: Duration) -> u64 {
    duration.as_micros().min(u128::from(u64::MAX)) as u64
}

fn fnv_offset() -> u64 {
    0xcbf29ce484222325
}

fn fnv_extend(mut state: u64, bytes: &[u8]) -> u64 {
    for byte in bytes {
        state ^= u64::from(*byte);
        state = state.wrapping_mul(0x100000001b3);
    }
    state
}

fn proc_count(path: &str) -> usize {
    fs::read_dir(path)
        .expect("read process resource directory")
        .count()
}

fn rss_kib() -> u64 {
    fs::read_to_string("/proc/self/status")
        .expect("read process status")
        .lines()
        .find_map(|line| line.strip_prefix("VmRSS:"))
        .and_then(|value| value.split_whitespace().next())
        .and_then(|value| value.parse().ok())
        .unwrap_or(0)
}

fn resource_sample(label: &'static str) -> ResourceSample {
    ResourceSample {
        label,
        rss_kib: rss_kib(),
        file_descriptors: proc_count("/proc/self/fd"),
        threads: proc_count("/proc/self/task"),
    }
}

fn command_output(program: &str, args: &[&str]) -> String {
    Command::new(program)
        .args(args)
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
        .unwrap_or_else(|| "unavailable".to_string())
}

fn host_evidence() -> HostEvidence {
    let cpu = fs::read_to_string("/proc/cpuinfo")
        .ok()
        .and_then(|text| {
            text.lines()
                .find_map(|line| line.strip_prefix("model name\t: "))
                .map(str::to_string)
        })
        .unwrap_or_else(|| "unavailable".to_string());
    let memory_kib = fs::read_to_string("/proc/meminfo")
        .ok()
        .and_then(|text| {
            text.lines()
                .find_map(|line| line.strip_prefix("MemTotal:"))
                .and_then(|value| value.split_whitespace().next())
                .and_then(|value| value.parse().ok())
        })
        .unwrap_or(0);
    HostEvidence {
        revision: command_output("git", &["rev-parse", "HEAD"]),
        kernel: command_output("uname", &["-srvmo"]),
        libc: command_output("getconf", &["GNU_LIBC_VERSION"]),
        cpu,
        memory_kib,
        architecture: std::env::consts::ARCH.to_string(),
        display_backend: std::env::var("XDG_SESSION_TYPE").unwrap_or_else(|_| "unknown".into()),
        wayland_display: std::env::var("WAYLAND_DISPLAY").ok(),
        x11_display: std::env::var("DISPLAY").ok(),
        build_profile: "release",
    }
}

fn evidence_path(tier: &str) -> PathBuf {
    std::env::var_os("DATUM_P06_EVIDENCE")
        .map(PathBuf::from)
        .unwrap_or_else(|| Path::new("target/p06-evidence").join(format!("{tier}.json")))
}

#[test]
#[ignore = "DTC-P06D release measurement; run through run_terminal_transport_proof_gates.sh"]
fn p06_release_measurement_emits_reproducible_json() {
    if std::hint::black_box(cfg!(debug_assertions)) {
        panic!("P06 evidence must use --release");
    }
    assert_eq!(std::env::consts::OS, "linux");
    assert_eq!(std::env::consts::ARCH, "x86_64");
    let tier = Tier::from_environment();
    let seed = std::env::var("DATUM_P06_SEED")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(0xD06D_2026_0818);
    let resources_before = resource_sample("warm_baseline");
    let single_output = measure_single_output(tier.output_bytes_per_session);
    let (aggregate_output, resources_peak) =
        measure_aggregate_output(tier.output_bytes_per_session);
    let (idle_latency, input_roundtrip, echo_session) =
        measure_latency_and_input(tier.latency_samples, tier.input_bytes_per_session);
    let resize_latency = measure_resize(&echo_session, tier.resize_samples);
    close_echo_session(&echo_session);
    drop(echo_session);
    let resources_after = resource_sample("after_close");
    let evidence = MeasurementEvidence {
        contract: "datum_terminal_p06_measurement_v1",
        tier: tier.name,
        seed,
        occurred_unix_ms: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time after epoch")
            .as_millis(),
        host: host_evidence(),
        sessions: SESSION_COUNT,
        output_bytes_per_session: tier.output_bytes_per_session,
        input_bytes_per_session: tier.input_bytes_per_session,
        idle_latency,
        resize_latency,
        single_output,
        aggregate_output,
        input_roundtrip,
        resources: vec![resources_before, resources_peak, resources_after],
        failures: Vec::new(),
    };
    let path = evidence_path(tier.name);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create P06 evidence directory");
    }
    fs::write(
        &path,
        serde_json::to_vec_pretty(&evidence).expect("serialize P06 evidence"),
    )
    .expect("write P06 evidence");
    println!("DTC-P06D evidence: {}", path.display());
}
