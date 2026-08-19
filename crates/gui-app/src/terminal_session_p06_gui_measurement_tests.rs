use super::*;
use serde::Serialize;
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

const SESSION_COUNT: usize = 8;
const OUTPUT_BYTES_PER_SESSION: usize = 1024 * 1024;
const DEADLINE: Duration = Duration::from_secs(30);

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
struct GuiPathMeasurement {
    sessions: usize,
    bytes: u64,
    observed_output_bytes: u64,
    elapsed_us: u64,
    mib_per_second: f64,
    drain_turns: usize,
    drain_work: Distribution,
}

#[derive(Serialize)]
struct GuiEvidence {
    contract: &'static str,
    revision: String,
    seed: u64,
    display_backend: String,
    occurred_unix_ms: u128,
    single_session: GuiPathMeasurement,
    eight_session_aggregate: GuiPathMeasurement,
    failures: Vec<String>,
}

fn unique_root(label: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time after epoch")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "datum-terminal-p06-gui-{label}-{}-{nonce}",
        std::process::id()
    ))
}

fn projection_contains(
    registry: &TerminalSessionRegistry,
    lane: &TerminalLaneState,
    index: usize,
    marker: &str,
) -> bool {
    let projection = if index == registry.active_index && registry.active_pending_id.is_none() {
        lane
    } else {
        &registry.sessions[index].parked_lane
    };
    projection
        .grid_lines()
        .iter()
        .any(|line| line.contains(marker))
}

fn prepare_registry(
    count: usize,
    label: &str,
) -> (
    PathBuf,
    TerminalLaunchContext,
    TerminalSessionRegistry,
    TerminalLaneState,
) {
    let root = unique_root(label);
    fs::create_dir_all(&root).expect("create GUI measurement root");
    let context = TerminalLaunchContext::for_project_root(&root);
    let mut registry = TerminalSessionRegistry::spawn(&context).expect("spawn measured registry");
    let mut lane = TerminalLaneState::default();
    for _ in 1..count {
        registry
            .spawn_and_activate_with_lane(&context, &mut lane)
            .expect("spawn measured GUI session");
    }
    for index in 0..count {
        let id = registry.sessions[index].session.session_id().to_string();
        registry
            .activate_with_lane(&id, &mut lane)
            .expect("activate measured GUI session");
        registry
            .resize_active(164, 24)
            .expect("apply production-equivalent initial terminal geometry");
        registry
            .active()
            .write_bytes(format!("stty -echo; printf '\\nDTC06GUIREADY{index}\\n'\n").as_bytes())
            .expect("prepare measured shell");
    }
    let deadline = Instant::now() + DEADLINE;
    while Instant::now() < deadline {
        registry.drain_all(&mut lane);
        if (0..count).all(|index| {
            projection_contains(&registry, &lane, index, &format!("DTC06GUIREADY{index}"))
        }) {
            break;
        }
        std::thread::yield_now();
    }
    assert!((0..count).all(|index| {
        projection_contains(&registry, &lane, index, &format!("DTC06GUIREADY{index}"))
    }));
    let settle_deadline = Instant::now() + Duration::from_secs(1);
    let mut quiet_since = Instant::now();
    while Instant::now() < settle_deadline && quiet_since.elapsed() < Duration::from_millis(20) {
        let report = registry.drain_all(&mut lane);
        if report.events > 0 {
            quiet_since = Instant::now();
        } else {
            std::thread::yield_now();
        }
    }
    (root, context, registry, lane)
}

fn measure_gui_path(count: usize, label: &str) -> GuiPathMeasurement {
    let (root, _context, mut registry, mut lane) = prepare_registry(count, label);
    let ids = registry
        .sessions
        .iter()
        .map(|slot| slot.session.session_id().to_string())
        .collect::<Vec<_>>();
    let started = Instant::now();
    for (index, id) in ids.iter().enumerate() {
        registry
            .activate_with_lane(id, &mut lane)
            .expect("activate GUI throughput session");
        let output = b'A' + index as u8;
        registry
            .active()
            .write_bytes(
                format!(
                    "head -c {OUTPUT_BYTES_PER_SESSION} /dev/zero | tr '\\000' '{}'; printf '\\nDTC06GUI%s\\n' 'DONE{index}'; read -r _\n",
                    char::from(output),
                )
                .as_bytes(),
            )
            .expect("start GUI throughput output");
    }
    let expected = count * OUTPUT_BYTES_PER_SESSION;
    let deadline = Instant::now() + DEADLINE;
    let mut bytes = 0usize;
    let mut turns = Vec::new();
    while Instant::now() < deadline {
        let turn = Instant::now();
        let report = registry.drain_all(&mut lane);
        turns.push(duration_us(turn.elapsed()));
        bytes += report.output_bytes;
        if (0..count).all(|index| {
            projection_contains(&registry, &lane, index, &format!("DTC06GUIDONE{index}"))
        }) {
            break;
        }
        if report.output_bytes == 0 {
            std::thread::yield_now();
        }
    }
    let elapsed = started.elapsed();
    assert!(
        bytes >= expected,
        "GUI drain did not consume every payload byte"
    );
    assert!((0..count).all(|index| {
        projection_contains(&registry, &lane, index, &format!("DTC06GUIDONE{index}"))
    }));

    let close_deadline = Instant::now() + Duration::from_secs(7);
    for slot in &registry.sessions {
        slot.session
            .terminate_by(close_deadline)
            .expect("terminate GUI measurement session");
    }
    while Instant::now() < close_deadline
        && !registry
            .sessions
            .iter()
            .all(|slot| slot.session.presentation_complete())
    {
        registry.drain_all(&mut lane);
        std::thread::yield_now();
    }
    for slot in &registry.sessions {
        let snapshot = slot
            .session
            .shutdown_snapshot()
            .expect("GUI shutdown snapshot");
        assert_eq!(
            snapshot.phase,
            crate::terminal_transport::ShutdownPhase::Closed
        );
        assert!(snapshot.leader_reaped);
        assert!(snapshot.surviving_processes.is_empty());
        assert!(slot.session.presentation_complete());
    }
    drop(registry);
    fs::remove_dir_all(root).expect("remove GUI measurement root");

    GuiPathMeasurement {
        sessions: count,
        bytes: expected as u64,
        observed_output_bytes: bytes as u64,
        elapsed_us: duration_us(elapsed),
        mib_per_second: (count * OUTPUT_BYTES_PER_SESSION) as f64
            / (1024.0 * 1024.0)
            / elapsed.as_secs_f64(),
        drain_turns: turns.len(),
        drain_work: distribution(turns),
    }
}

fn distribution(mut raw_us: Vec<u64>) -> Distribution {
    assert!(!raw_us.is_empty());
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
        .unwrap_or_else(|| Path::new("target/p06-evidence/gui.json").to_path_buf())
}

#[test]
#[ignore = "DTC-P06D release GUI-path measurement; run through the proof gate runner"]
fn p06_provisional_gui_path_emits_reproducible_json() {
    if std::hint::black_box(cfg!(debug_assertions)) {
        panic!("P06 GUI-path evidence must use --release");
    }
    let evidence = GuiEvidence {
        contract: "datum_terminal_p06_gui_measurement_v1",
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
        single_session: measure_gui_path(1, "single"),
        eight_session_aggregate: measure_gui_path(SESSION_COUNT, "aggregate"),
        failures: Vec::new(),
    };
    let path = evidence_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create GUI evidence directory");
    }
    fs::write(
        &path,
        serde_json::to_vec_pretty(&evidence).expect("serialize GUI evidence"),
    )
    .expect("write GUI evidence");
    println!("DTC-P06D GUI evidence: {}", path.display());
}
