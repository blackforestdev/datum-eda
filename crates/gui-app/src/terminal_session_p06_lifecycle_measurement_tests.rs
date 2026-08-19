use super::*;
use serde::Serialize;
use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

const P06_LIFECYCLE_CYCLES: usize = 1_000;
const CYCLE_DEADLINE: Duration = Duration::from_secs(7);

#[derive(Serialize)]
struct ResourcePoint {
    cycle: usize,
    rss_kib: u64,
    file_descriptors: usize,
    threads: usize,
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
struct LifecycleEvidence {
    contract: &'static str,
    revision: String,
    seed: u64,
    display_backend: String,
    occurred_unix_ms: u128,
    requested_cycles: usize,
    completed_cycles: usize,
    unique_session_ids: usize,
    elapsed_seconds: u64,
    cooperative_exit_latency: Distribution,
    baseline: ResourcePoint,
    resources: Vec<ResourcePoint>,
    final_file_descriptors: usize,
    final_threads: usize,
    survivors: Vec<String>,
    failures: Vec<String>,
}

fn unique_root() -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time after epoch")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "datum-terminal-p06-lifecycle-{}-{nonce}",
        std::process::id()
    ))
}

fn resource(cycle: usize) -> ResourcePoint {
    ResourcePoint {
        cycle,
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

fn wait_for_exact_exit(
    registry: &mut TerminalSessionRegistry,
    lane: &mut TerminalLaneState,
    expected: &str,
) {
    let deadline = Instant::now() + CYCLE_DEADLINE;
    while Instant::now() < deadline {
        registry.drain_all(lane);
        let slot = &registry.sessions[registry.active_index];
        if slot.exact_exit_status.as_deref() == Some(expected)
            && slot.session.presentation_complete()
        {
            let snapshot = slot
                .session
                .shutdown_snapshot()
                .expect("cycle shutdown snapshot");
            assert_eq!(
                snapshot.phase,
                crate::terminal_transport::ShutdownPhase::Closed
            );
            assert!(snapshot.leader_reaped);
            assert!(snapshot.surviving_processes.is_empty());
            return;
        }
        std::thread::yield_now();
    }
    panic!("lifecycle cycle did not reach exact presentation completion");
}

fn distribution(mut raw_us: Vec<u64>) -> Distribution {
    raw_us.sort_unstable();
    let at = |percent: usize| {
        let index = (raw_us.len() * percent).div_ceil(100).saturating_sub(1);
        raw_us[index.min(raw_us.len() - 1)]
    };
    Distribution {
        count: raw_us.len(),
        min_us: raw_us[0],
        p50_us: at(50),
        p95_us: at(95),
        p99_us: at(99),
        max_us: *raw_us.last().unwrap(),
        raw_us,
    }
}

fn duration_us(duration: Duration) -> u64 {
    duration.as_micros().min(u128::from(u64::MAX)) as u64
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
        .unwrap_or_else(|| Path::new("target/p06-evidence/lifecycle-1000.json").to_path_buf())
}

#[test]
#[ignore = "DTC-P06D 1000-cycle release lifecycle proof; run through the proof gate runner"]
fn p06_one_thousand_spawn_exit_restart_cycles_emit_reproducible_json() {
    if std::hint::black_box(cfg!(debug_assertions)) {
        panic!("P06 lifecycle evidence must use --release");
    }
    let root = unique_root();
    fs::create_dir_all(&root).expect("create lifecycle evidence root");
    let context = TerminalLaunchContext::for_project_root(&root);
    let baseline = resource(0);
    let started = Instant::now();
    let mut registry = TerminalSessionRegistry::spawn(&context).expect("spawn lifecycle shell");
    let mut lane = TerminalLaneState::default();
    let mut session_ids = HashSet::new();
    let mut exit_latency = Vec::with_capacity(P06_LIFECYCLE_CYCLES);
    let mut resources = vec![resource(0)];

    for cycle in 0..P06_LIFECYCLE_CYCLES {
        session_ids.insert(registry.active().session_id().to_string());
        let code = (cycle % 64) as i32;
        let exit_started = Instant::now();
        registry
            .active()
            .write_bytes(format!("exit {code}\n").as_bytes())
            .expect("request lifecycle exit");
        wait_for_exact_exit(&mut registry, &mut lane, &format!("exited {code}"));
        exit_latency.push(duration_us(exit_started.elapsed()));
        if (cycle + 1).is_multiple_of(10) {
            resources.push(resource(cycle + 1));
        }
        if cycle + 1 < P06_LIFECYCLE_CYCLES {
            registry
                .restart_active(&mut lane, &context)
                .expect("restart lifecycle session after verified closure");
        }
    }

    let final_snapshot = registry
        .active()
        .shutdown_snapshot()
        .expect("final lifecycle snapshot");
    let mut survivors = final_snapshot
        .surviving_processes
        .iter()
        .map(|process| {
            format!(
                "pid={} pgid={} sid={}",
                process.pid, process.process_group_id, process.session_id
            )
        })
        .collect::<Vec<_>>();
    if final_snapshot.phase != crate::terminal_transport::ShutdownPhase::Closed {
        survivors.push(format!("final phase={:?}", final_snapshot.phase));
    }
    drop(registry);
    let settle_deadline = Instant::now() + Duration::from_secs(1);
    while Instant::now() < settle_deadline
        && (proc_count("/proc/self/fd") > baseline.file_descriptors + 2
            || proc_count("/proc/self/task") > baseline.threads + 2)
    {
        std::thread::yield_now();
    }
    let evidence = LifecycleEvidence {
        contract: "datum_terminal_p06_lifecycle_v1",
        revision: command_output("git", &["rev-parse", "HEAD"]),
        seed: std::env::var("DATUM_P06_SEED")
            .ok()
            .and_then(|value| value.parse().ok())
            .unwrap_or(0xD06D_2026_0818),
        display_backend: std::env::var("XDG_SESSION_TYPE").unwrap_or_else(|_| "unknown".into()),
        occurred_unix_ms: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time after epoch")
            .as_millis(),
        requested_cycles: P06_LIFECYCLE_CYCLES,
        completed_cycles: exit_latency.len(),
        unique_session_ids: session_ids.len(),
        elapsed_seconds: started.elapsed().as_secs(),
        cooperative_exit_latency: distribution(exit_latency),
        baseline,
        resources,
        final_file_descriptors: proc_count("/proc/self/fd"),
        final_threads: proc_count("/proc/self/task"),
        survivors,
        failures: Vec::new(),
    };
    let path = evidence_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create lifecycle evidence directory");
    }
    fs::write(
        &path,
        serde_json::to_vec_pretty(&evidence).expect("serialize lifecycle evidence"),
    )
    .expect("write lifecycle evidence");
    fs::remove_dir_all(root).expect("remove lifecycle evidence root");
    println!("DTC-P06D lifecycle evidence: {}", path.display());
}
