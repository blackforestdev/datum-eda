use crate::terminal_transport::{
    ShutdownPhase, TerminalTransportEvent, TerminalTransportRequest, TerminalTransportSession,
    TerminalWakeGate, prepare_terminal_transport,
};
use serde::Serialize;
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

const GLOBAL_CLOSE: Duration = Duration::from_secs(7);
const IO_STALL: Duration = Duration::from_secs(30);
const LONG_AGGREGATE_BYTES: u64 = 1024 * 1024 * 1024;

#[derive(Clone, Copy)]
enum WorkloadRole {
    AgentBurst,
    StatusUpdate,
    FullScreen,
    Resize,
    Lifecycle,
    Saturation,
    Interactive,
    Idle,
}

impl WorkloadRole {
    const ALL: [Self; 8] = [
        Self::AgentBurst,
        Self::StatusUpdate,
        Self::FullScreen,
        Self::Resize,
        Self::Lifecycle,
        Self::Saturation,
        Self::Interactive,
        Self::Idle,
    ];

    fn index(self) -> usize {
        self as usize
    }

    fn name(self) -> &'static str {
        match self {
            Self::AgentBurst => "agent-burst",
            Self::StatusUpdate => "status-update",
            Self::FullScreen => "full-screen",
            Self::Resize => "resize",
            Self::Lifecycle => "lifecycle",
            Self::Saturation => "saturation",
            Self::Interactive => "interactive",
            Self::Idle => "idle",
        }
    }

    fn bytes(self, cycle: u64) -> usize {
        match self {
            Self::AgentBurst => 64 * 1024,
            Self::StatusUpdate => 1024,
            Self::FullScreen => 32 * 1024,
            Self::Resize | Self::Lifecycle => 16 * 1024,
            Self::Saturation if cycle % 600 == 5 => 4 * 1024 * 1024,
            Self::Saturation => 16 * 1024,
            Self::Interactive => 4 * 1024,
            Self::Idle => 0,
        }
    }
}

#[derive(Clone, Copy)]
struct SoakTier {
    name: &'static str,
    duration: Duration,
    sessions: usize,
    minimum_bytes_per_session: u64,
    resize_requests: usize,
    resource_interval: Duration,
}

impl SoakTier {
    fn from_environment() -> Self {
        let name = std::env::var("DATUM_P06_TIER")
            .expect("DATUM_P06_TIER is required for scheduled P06 evidence");
        Self::named(&name)
    }

    fn named(name: &str) -> Self {
        match name {
            "ci" => Self {
                name: "ci-10-minute",
                duration: Duration::from_secs(10 * 60),
                sessions: 8,
                minimum_bytes_per_session: 8 * 1024 * 1024,
                resize_requests: 1_000,
                resource_interval: Duration::from_secs(30),
            },
            "single-24h" => Self {
                name: "single-24-hour",
                duration: Duration::from_secs(24 * 60 * 60),
                sessions: 1,
                minimum_bytes_per_session: 128 * 1024 * 1024,
                resize_requests: 500,
                resource_interval: Duration::from_secs(60 * 60),
            },
            "max-4h" => Self {
                name: "maximum-16-session-4-hour",
                duration: Duration::from_secs(4 * 60 * 60),
                sessions: 16,
                minimum_bytes_per_session: 128 * 1024 * 1024,
                resize_requests: 10_000,
                resource_interval: Duration::from_secs(5 * 60),
            },
            other => panic!("unsupported scheduled P06 tier {other:?}"),
        }
    }
}

#[derive(Serialize)]
struct ResourcePoint {
    elapsed_seconds: u64,
    rss_kib: u64,
    file_descriptors: usize,
    threads: usize,
}

#[derive(Serialize)]
struct HostEvidence {
    kernel: String,
    libc: String,
    cpu: String,
    memory_kib: u64,
    architecture: &'static str,
}

#[derive(Serialize)]
struct RoleEvidence {
    name: &'static str,
    cycles: u64,
}

#[derive(Serialize)]
struct Percentiles {
    count: usize,
    min_us: u64,
    p50_us: u64,
    p95_us: u64,
    p99_us: u64,
    max_us: u64,
    raw_us: Vec<u64>,
}

#[derive(Serialize)]
struct SoakEvidence {
    contract: &'static str,
    revision: String,
    seed: u64,
    run_ordinal: u8,
    tier: &'static str,
    display_backend: String,
    host: HostEvidence,
    started_unix_ms: u128,
    elapsed_seconds: u64,
    sessions: usize,
    bytes_per_session: Vec<u64>,
    input_bytes_per_session: Vec<u64>,
    output_bytes_per_session: Vec<u64>,
    aggregate_input_bytes: u64,
    aggregate_output_bytes: u64,
    checksums: Vec<u64>,
    roles: Vec<RoleEvidence>,
    restart_count: u64,
    roundtrip_latency: Percentiles,
    resize_latency: Percentiles,
    resize_requests: usize,
    baseline: ResourcePoint,
    resources: Vec<ResourcePoint>,
    final_file_descriptors: usize,
    final_threads: usize,
    survivors: Vec<String>,
    failures: Vec<String>,
}

fn spawn_echo() -> TerminalTransportSession {
    let session = prepare_terminal_transport(
        TerminalTransportRequest::new("/bin/sh", PathBuf::from("/tmp"))
            .args(["-c", "stty raw -echo; printf R; exec cat"]),
    )
    .expect("prepare soak PTY")
    .start(TerminalWakeGate::new(None));
    let deadline = Instant::now() + IO_STALL;
    while Instant::now() < deadline {
        match session.recv_event_timeout(Duration::from_millis(20)) {
            Ok(TerminalTransportEvent::Output(bytes)) if bytes.contains(&b'R') => return session,
            Ok(TerminalTransportEvent::Output(_)) => {}
            Ok(other) => panic!("soak PTY setup event: {other:?}"),
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {}
            Err(error) => panic!("soak PTY setup disconnected: {error:?}"),
        }
    }
    panic!("soak PTY setup timed out");
}

fn payload(seed: u64, session: usize, cycle: u64, bytes: usize) -> Vec<u8> {
    let mut state = seed ^ (session as u64).wrapping_mul(0x9e37_79b9) ^ cycle.rotate_left(17);
    (0..bytes)
        .map(|_| {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            state as u8
        })
        .collect()
}

fn enqueue_exact(session: &TerminalTransportSession, bytes: &[u8]) {
    let deadline = Instant::now() + IO_STALL;
    loop {
        match session.write_bytes(bytes) {
            Ok(()) => return,
            Err(crate::terminal_transport::TerminalInputError::Busy) => std::thread::yield_now(),
            Err(error) => panic!("soak input admission failed: {error}"),
        }
        assert!(Instant::now() < deadline, "soak input admission stalled");
    }
}

fn drain_cycle(
    sessions: &[TerminalTransportSession],
    expected: &[Vec<u8>],
    started: &[Instant],
) -> Vec<Option<u64>> {
    let deadline = Instant::now() + IO_STALL;
    let mut received = (0..sessions.len()).map(|_| Vec::new()).collect::<Vec<_>>();
    let mut first = vec![None; sessions.len()];
    while Instant::now() < deadline {
        let mut progressed = false;
        for (index, session) in sessions.iter().enumerate() {
            while let Ok(event) = session.try_recv_event() {
                progressed = true;
                match event {
                    TerminalTransportEvent::Output(bytes) => {
                        first[index].get_or_insert_with(Instant::now);
                        received[index].extend(bytes);
                    }
                    other => panic!("soak data cycle event: {other:?}"),
                }
            }
        }
        if received
            .iter()
            .zip(expected)
            .all(|(actual, wanted)| actual.len() >= wanted.len())
        {
            break;
        }
        if !progressed {
            std::thread::yield_now();
        }
    }
    received
        .iter()
        .zip(expected)
        .enumerate()
        .map(|(index, (actual, wanted))| {
            assert_eq!(actual, wanted, "soak session {index} byte stream");
            first[index].map(|first| duration_us(first - started[index]))
        })
        .collect()
}

fn restart_echo(session: &mut TerminalTransportSession) {
    let deadline = Instant::now() + GLOBAL_CLOSE;
    session
        .terminate_by(deadline)
        .expect("begin scheduled soak restart");
    while Instant::now() < deadline && !session.presentation_complete() {
        while let Ok(event) = session.try_recv_event() {
            if let TerminalTransportEvent::Error(error) = event {
                panic!("scheduled soak restart error: {error:?}");
            }
        }
        std::thread::yield_now();
    }
    assert!(
        session.presentation_complete(),
        "scheduled restart timed out"
    );
    *session = spawn_echo();
}

fn close_all(sessions: &[TerminalTransportSession]) -> Vec<String> {
    let deadline = Instant::now() + GLOBAL_CLOSE;
    for session in sessions {
        session
            .terminate_by(deadline)
            .expect("begin concurrent soak close");
    }
    let mut survivors = Vec::new();
    while Instant::now() < deadline {
        for session in sessions {
            while let Ok(event) = session.try_recv_event() {
                if let TerminalTransportEvent::Error(error) = event {
                    panic!("soak shutdown transport error: {error:?}");
                }
            }
        }
        if sessions
            .iter()
            .all(TerminalTransportSession::presentation_complete)
        {
            break;
        }
        std::thread::yield_now();
    }
    for session in sessions {
        let snapshot = session.shutdown_snapshot().expect("soak shutdown snapshot");
        if snapshot.phase != ShutdownPhase::Closed || !snapshot.leader_reaped {
            survivors.push(format!(
                "pgid={} phase={:?}",
                session.process_group_id(),
                snapshot.phase
            ));
        }
        survivors.extend(snapshot.surviving_processes.iter().map(|process| {
            format!(
                "pid={} pgid={} sid={}",
                process.pid, process.process_group_id, process.session_id
            )
        }));
    }
    survivors
}

fn resource(elapsed: Duration) -> ResourcePoint {
    ResourcePoint {
        elapsed_seconds: elapsed.as_secs(),
        rss_kib: rss_kib(),
        file_descriptors: proc_count("/proc/self/fd"),
        threads: proc_count("/proc/self/task"),
    }
}

fn proc_count(path: &str) -> usize {
    fs::read_dir(path)
        .expect("read Linux resource directory")
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

fn percentiles(mut raw_us: Vec<u64>) -> Percentiles {
    raw_us.sort_unstable();
    let at = |percent: usize| {
        let index = (raw_us.len() * percent).div_ceil(100).saturating_sub(1);
        raw_us[index.min(raw_us.len() - 1)]
    };
    Percentiles {
        count: raw_us.len(),
        min_us: raw_us[0],
        p50_us: at(50),
        p95_us: at(95),
        p99_us: at(99),
        max_us: *raw_us.last().unwrap(),
        raw_us,
    }
}

fn command_output(program: &str, args: &[&str]) -> String {
    Command::new(program)
        .args(args)
        .output()
        .ok()
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
        kernel: command_output("uname", &["-srvmo"]),
        libc: command_output("getconf", &["GNU_LIBC_VERSION"]),
        cpu,
        memory_kib,
        architecture: std::env::consts::ARCH,
    }
}

fn evidence_path(tier: &str) -> PathBuf {
    std::env::var_os("DATUM_P06_EVIDENCE")
        .map(PathBuf::from)
        .unwrap_or_else(|| Path::new("target/p06-evidence").join(format!("{tier}.json")))
}

#[test]
fn p06_soak_tier_budgets_are_literal() {
    let ci = SoakTier::named("ci");
    assert_eq!((ci.duration.as_secs(), ci.sessions), (600, 8));
    assert_eq!(
        (ci.minimum_bytes_per_session, ci.resize_requests),
        (8 << 20, 1_000)
    );

    let single = SoakTier::named("single-24h");
    assert_eq!((single.duration.as_secs(), single.sessions), (86_400, 1));
    assert_eq!(single.minimum_bytes_per_session, 128 << 20);

    let maximum = SoakTier::named("max-4h");
    assert_eq!((maximum.duration.as_secs(), maximum.sessions), (14_400, 16));
    assert_eq!(
        (maximum.minimum_bytes_per_session, maximum.resize_requests),
        (128 << 20, 10_000)
    );
    assert_eq!(LONG_AGGREGATE_BYTES, 1 << 30);
    assert_eq!(WorkloadRole::Saturation.bytes(5), 4 << 20);
    assert_eq!(WorkloadRole::Idle.bytes(0), 0);
}

#[test]
#[ignore = "DTC-P06D scheduled release soak; run through the proof gate runner"]
fn p06_scheduled_soak_emits_reproducible_json() {
    if std::hint::black_box(cfg!(debug_assertions)) {
        panic!("P06 soak evidence must use --release");
    }
    let tier = SoakTier::from_environment();
    let seed = std::env::var("DATUM_P06_SEED")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(0xD06D_2026_0818);
    let run_ordinal = std::env::var("DATUM_P06_RUN_ORDINAL")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(1);
    assert!((1..=3).contains(&run_ordinal));
    let started_unix_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time after epoch")
        .as_millis();
    let baseline = resource(Duration::ZERO);
    let started = Instant::now();
    let mut sessions = (0..tier.sessions).map(|_| spawn_echo()).collect::<Vec<_>>();
    let mut bytes_per_session = vec![0u64; tier.sessions];
    let mut checksums = vec![0xcbf29ce484222325; tier.sessions];
    let mut latency_us = Vec::new();
    let mut resize_us = Vec::with_capacity(tier.resize_requests);
    let mut resize_index = 0usize;
    let mut resources = vec![resource(Duration::ZERO)];
    let mut next_resource = tier.resource_interval;
    let mut cycle = 0u64;
    let mut role_cycles = [0u64; 8];
    let mut restart_count = 0u64;
    let mut next_cycle = Instant::now();

    while started.elapsed() < tier.duration
        || bytes_per_session
            .iter()
            .any(|bytes| *bytes < tier.minimum_bytes_per_session)
    {
        if tier.sessions > 1 && cycle > 0 && cycle.is_multiple_of(900) {
            let index = (cycle as usize / 900) % sessions.len();
            restart_echo(&mut sessions[index]);
            restart_count += 1;
        }
        let expected = (0..tier.sessions)
            .map(|index| {
                let role = WorkloadRole::ALL[(cycle as usize + index) % WorkloadRole::ALL.len()];
                role_cycles[role.index()] += 1;
                payload(seed, index, cycle, role.bytes(cycle))
            })
            .collect::<Vec<_>>();
        let admitted = (0..tier.sessions)
            .map(|_| Instant::now())
            .collect::<Vec<_>>();
        for (session, bytes) in sessions.iter().zip(&expected) {
            enqueue_exact(session, bytes);
        }
        latency_us.extend(
            drain_cycle(&sessions, &expected, &admitted)
                .into_iter()
                .flatten(),
        );
        for (index, bytes) in expected.iter().enumerate() {
            bytes_per_session[index] += bytes.len() as u64;
            checksums[index] = fnv_extend(checksums[index], bytes);
        }
        let resize_batch = tier
            .resize_requests
            .div_ceil(tier.duration.as_secs() as usize);
        for _ in 0..resize_batch {
            if resize_index >= tier.resize_requests {
                break;
            }
            let session = &sessions[resize_index % sessions.len()];
            let resize_started = Instant::now();
            session
                .resize(
                    80 + (resize_index % 80) as u16,
                    20 + (resize_index % 40) as u16,
                )
                .expect("scheduled resize");
            resize_us.push(duration_us(resize_started.elapsed()));
            resize_index += 1;
        }
        if started.elapsed() >= next_resource {
            resources.push(resource(started.elapsed()));
            next_resource += tier.resource_interval;
        }
        cycle += 1;
        next_cycle += Duration::from_secs(1);
        if let Some(wait) = next_cycle.checked_duration_since(Instant::now()) {
            std::thread::sleep(wait);
        }
    }

    let survivors = close_all(&sessions);
    drop(sessions);
    let settle_deadline = Instant::now() + Duration::from_secs(1);
    while Instant::now() < settle_deadline
        && (proc_count("/proc/self/fd") > baseline.file_descriptors + 2
            || proc_count("/proc/self/task") > baseline.threads + 2)
    {
        std::thread::yield_now();
    }
    resources.push(resource(started.elapsed()));
    let aggregate_bytes: u64 = bytes_per_session.iter().sum();
    let evidence = SoakEvidence {
        contract: "datum_terminal_p06_soak_v1",
        revision: command_output("git", &["rev-parse", "HEAD"]),
        seed,
        run_ordinal,
        tier: tier.name,
        display_backend: std::env::var("XDG_SESSION_TYPE").unwrap_or_else(|_| "unknown".into()),
        host: host_evidence(),
        started_unix_ms,
        elapsed_seconds: started.elapsed().as_secs(),
        sessions: tier.sessions,
        input_bytes_per_session: bytes_per_session.clone(),
        output_bytes_per_session: bytes_per_session.clone(),
        bytes_per_session,
        aggregate_input_bytes: aggregate_bytes,
        aggregate_output_bytes: aggregate_bytes,
        checksums,
        roles: WorkloadRole::ALL
            .into_iter()
            .map(|role| RoleEvidence {
                name: role.name(),
                cycles: role_cycles[role.index()],
            })
            .collect(),
        restart_count,
        roundtrip_latency: percentiles(latency_us),
        resize_latency: percentiles(resize_us),
        resize_requests: resize_index,
        baseline,
        resources,
        final_file_descriptors: proc_count("/proc/self/fd"),
        final_threads: proc_count("/proc/self/task"),
        survivors,
        failures: Vec::new(),
    };
    let path = evidence_path(tier.name);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create soak evidence directory");
    }
    fs::write(
        &path,
        serde_json::to_vec_pretty(&evidence).expect("serialize soak evidence"),
    )
    .expect("write soak evidence");
    println!("DTC-P06D soak evidence: {}", path.display());
}
