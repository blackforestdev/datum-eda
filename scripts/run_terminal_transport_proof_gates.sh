#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

if [[ -n "$(git status --porcelain --untracked-files=normal)" ]]; then
  echo "DTC-P06D evidence requires a clean revision-addressed worktree" >&2
  exit 1
fi

tier="${1:-smoke}"
case "$tier" in
  backend-canary|smoke|gui|throughput-60s|lifecycle-1000|ci) ;;
  *)
    echo "usage: $0 {backend-canary|smoke|gui|throughput-60s|lifecycle-1000|ci}" >&2
    exit 2
    ;;
esac

if [[ "$(uname -s)" != Linux || "$(uname -m)" != x86_64 ]]; then
  echo "DTC-P06D requires the ratified Linux x86_64/glibc platform" >&2
  exit 1
fi

session_type="${XDG_SESSION_TYPE:-unknown}"
if [[ "$session_type" == wayland ]]; then
  backend="wayland-primary"
elif [[ "$session_type" == x11 ]]; then
  backend="x11-fallback"
else
  backend="headless-or-unknown"
fi

seed="${DATUM_P06_SEED:-229532293120280}"
run_ordinal="${DATUM_P06_RUN_ORDINAL:-1}"
if [[ ! "$run_ordinal" =~ ^[123]$ ]]; then
  echo "DATUM_P06_RUN_ORDINAL must be 1, 2, or 3" >&2
  exit 2
fi
revision="$(git rev-parse HEAD)"
evidence_dir="${DATUM_P06_EVIDENCE_DIR:-target/p06-evidence}"
if [[ "$evidence_dir" != /* ]]; then
  evidence_dir="$repo_root/$evidence_dir"
fi
evidence_path="$evidence_dir/${revision}-${tier}-${backend}-run${run_ordinal}.json"
mkdir -p "$evidence_dir"

echo "DTC-P06D tier=$tier backend=$backend revision=$revision seed=$seed run=$run_ordinal"
if [[ "$tier" == backend-canary ]]; then
  screenshot_path="${evidence_path%.json}.png"
  log_path="${evidence_path%.json}.log"
  gui_args=(
    cargo run -p datum-gui-app --release --locked --offline --features visual --bin datum-gui --
    --project-root crates/engine/testdata/golden/text/native/text-fidelity-repro
    --window-size 1280x768 --visual-scale-factor 1
    --visual-test --window-visual-test --screenshot-out "$screenshot_path"
    --exit-after-screenshot --interaction-smoke --resize-torture-smoke
  )
  if [[ "$backend" == wayland-primary ]]; then
    if [[ -z "${WAYLAND_DISPLAY:-}" ]]; then
      echo "Wayland primary canary requires WAYLAND_DISPLAY" >&2
      exit 1
    fi
    env -u DISPLAY WINIT_UNIX_BACKEND=wayland XDG_SESSION_TYPE=wayland \
      DATUM_GUI_LOG="$log_path" "${gui_args[@]}"
  elif [[ "$backend" == x11-fallback ]]; then
    if [[ -z "${DISPLAY:-}" ]]; then
      python3 - "$evidence_path" "$revision" <<'PY'
import json, pathlib, sys
pathlib.Path(sys.argv[1]).write_text(json.dumps({
    "contract": "datum_terminal_p06_backend_canary_v1",
    "revision": sys.argv[2],
    "backend": "x11-fallback",
    "status": "unavailable",
    "reason": "DISPLAY is not available",
}, indent=2) + "\n", encoding="utf-8")
PY
      echo "DTC-P06D X11 fallback explicitly unavailable: $evidence_path"
      exit 0
    fi
    env -u WAYLAND_DISPLAY WINIT_UNIX_BACKEND=x11 XDG_SESSION_TYPE=x11 \
      DATUM_GUI_LOG="$log_path" "${gui_args[@]}"
  else
    echo "backend canary requires XDG_SESSION_TYPE=wayland or x11" >&2
    exit 1
  fi
  test -s "$screenshot_path"
  grep -q "window created" "$log_path"
  grep -q "renderer init end" "$log_path"
  grep -q "window visible" "$log_path"
  python3 - "$evidence_path" "$revision" "$backend" "$screenshot_path" "$log_path" <<'PY'
import hashlib, json, pathlib, sys
evidence, revision, backend, screenshot_name, log_name = sys.argv[1:]
screenshot = pathlib.Path(screenshot_name)
payload = {
    "contract": "datum_terminal_p06_backend_canary_v1",
    "revision": revision,
    "backend": backend,
    "status": "passed",
    "screenshot": str(screenshot),
    "screenshot_bytes": screenshot.stat().st_size,
    "screenshot_sha256": hashlib.sha256(screenshot.read_bytes()).hexdigest(),
    "log": log_name,
    "window_created": True,
    "renderer_initialized": True,
    "window_visible": True,
}
pathlib.Path(evidence).write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
PY
  echo "DTC-P06D compositor evidence verified: $evidence_path"
  exit 0
fi
if [[ "$tier" == smoke ]]; then
  test_name="terminal_session::terminal_session_p06_measurement_tests::p06_release_measurement_emits_reproducible_json"
elif [[ "$tier" == gui ]]; then
  test_name="terminal_session::terminal_session_p06_gui_measurement_tests::p06_production_core_gui_path_emits_reproducible_json"
elif [[ "$tier" == lifecycle-1000 ]]; then
  test_name="terminal_session::terminal_session_p06_lifecycle_measurement_tests::p06_one_thousand_spawn_exit_restart_cycles_emit_reproducible_json"
elif [[ "$tier" == throughput-60s ]]; then
  test_name="terminal_session::terminal_session_p06_throughput_tests::p06_sustained_throughput_and_backlog_emit_reproducible_json"
else
  test_name="terminal_session::terminal_session_p06_soak_tests::p06_bounded_ci_emits_reproducible_json"
fi
DATUM_P06_TIER="$tier" \
DATUM_P06_SEED="$seed" \
DATUM_P06_RUN_ORDINAL="$run_ordinal" \
DATUM_P06_EVIDENCE="$evidence_path" \
cargo test -p datum-gui-app --release --locked --offline \
  "$test_name" -- --ignored --exact --nocapture --test-threads=1

python3 - "$evidence_path" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
evidence = json.loads(path.read_text(encoding="utf-8"))
if evidence.get("failures"):
    raise SystemExit(f"DTC-P06D evidence reports failures: {evidence['failures']}")
if evidence.get("contract") == "datum_terminal_p06_gui_measurement_v1":
    single = evidence["single_session"]
    aggregate = evidence["eight_session_aggregate"]
    checks = {
        "single production-core GUI throughput >=1MiB/s": single["mib_per_second"] >= 1.0,
        "aggregate production-core GUI throughput >=4MiB/s": aggregate["mib_per_second"] >= 4.0,
        "single drain p95 <=8ms": single["drain_work"]["p95_us"] <= 8_000,
        "single drain p99 <=16ms": single["drain_work"]["p99_us"] <= 16_000,
        "single drain max <=33ms": single["drain_work"]["max_us"] <= 33_000,
        "aggregate drain p95 <=8ms": aggregate["drain_work"]["p95_us"] <= 8_000,
        "aggregate drain p99 <=16ms": aggregate["drain_work"]["p99_us"] <= 16_000,
        "aggregate drain max <=33ms": aggregate["drain_work"]["max_us"] <= 33_000,
    }
    failed = [name for name, passed in checks.items() if not passed]
    if failed:
        raise SystemExit("DTC-P06D GUI threshold failures: " + ", ".join(failed))
    print(f"DTC-P06D GUI evidence verified: {path}")
    raise SystemExit(0)
if evidence.get("contract") == "datum_terminal_p06_lifecycle_v1":
    baseline = evidence["baseline"]
    exit_latency = evidence["cooperative_exit_latency"]
    resources = evidence["resources"]
    checks = {
        "exactly 1000 completed lifecycle cycles": evidence["requested_cycles"] == 1_000
            and evidence["completed_cycles"] == 1_000,
        "1000 unique restarted session identities": evidence["unique_session_ids"] == 1_000,
        "cooperative exit p99 <=500ms": exit_latency["p99_us"] <= 500_000,
        "zero original-SID survivors": not evidence["survivors"],
        "RSS returns within baseline+16MiB": resources[-1]["rss_kib"] <= baseline["rss_kib"] + 16 * 1024,
        "FDs never grow beyond one live-session shape": max(point["file_descriptors"] for point in resources)
            <= baseline["file_descriptors"] + 4 + 8,
        "workers never grow beyond one live-session shape": max(point["threads"] for point in resources)
            <= baseline["threads"] + 4 + 4,
        "closed FDs return to baseline+2": evidence["final_file_descriptors"] <= baseline["file_descriptors"] + 2,
        "closed workers return to baseline+2": evidence["final_threads"] <= baseline["threads"] + 2,
    }
    failed = [name for name, passed in checks.items() if not passed]
    if failed:
        raise SystemExit("DTC-P06D lifecycle threshold failures: " + ", ".join(failed))
    print(f"DTC-P06D lifecycle evidence verified: {path}")
    raise SystemExit(0)
if evidence.get("contract") == "datum_terminal_p06_sustained_v1":
    single = evidence["single_session"]
    aggregate = evidence["eight_session_aggregate"]
    recovery = evidence["backlog_recovery"]
    checks = {
        "single duration >=60s": single["duration_us"] >= 60_000_000,
        "aggregate duration >=60s": aggregate["duration_us"] >= 60_000_000,
        "single throughput >=20MiB/s": single["mib_per_second"] >= 20.0,
        "aggregate throughput >=40MiB/s": aggregate["mib_per_second"] >= 40.0,
        "single exact input/output": single["input_bytes"] == single["output_bytes"],
        "aggregate exact input/output": aggregate["input_bytes"] == aggregate["output_bytes"],
        "fairness gap <=100ms": aggregate["max_fairness_gap_us"] <= 100_000,
        "backlog fixture emits exact 4MiB burst": recovery["burst_bytes"] == 4 << 20,
        "backlog saturates a governed queue limit": recovery["high_water_chunks"] == 256
            or recovery["high_water_bytes"] == 4 << 20,
        "backlog high-water remains within 4MiB": 0 < recovery["high_water_bytes"] <= 4 << 20,
        "backlog falls below 64KiB <=2s": recovery["below_64_kib_us"] <= 2_000_000,
        "backlog reaches zero <=5s": recovery["zero_us"] <= 5_000_000,
    }
    failed = [name for name, passed in checks.items() if not passed]
    if failed:
        raise SystemExit("DTC-P06D sustained threshold failures: " + ", ".join(failed))
    print(f"DTC-P06D sustained evidence verified: {path}")
    raise SystemExit(0)
if evidence.get("contract") == "datum_terminal_p06_soak_v1":
    tiers = {"ci-10-minute": (8, 600, 8 * 1024 * 1024, 1_000)}
    expected = tiers.get(evidence.get("tier"))
    if expected is None:
        raise SystemExit("unknown DTC-P06D soak tier")
    sessions, duration, byte_floor, resize_count = expected
    baseline = evidence["baseline"]
    resources = evidence["resources"]
    peak_rss = max(point["rss_kib"] for point in resources)
    peak_fds = max(point["file_descriptors"] for point in resources)
    peak_threads = max(point["threads"] for point in resources)
    latency = evidence["roundtrip_latency"]
    resize = evidence["resize_latency"]
    checks = {
        "exact session cardinality": evidence["sessions"] == sessions,
        "minimum governed duration": evidence["elapsed_seconds"] >= duration,
        "per-session output byte floor": len(evidence["output_bytes_per_session"]) == sessions
            and all(value >= byte_floor for value in evidence["output_bytes_per_session"]),
        "per-session input byte floor": len(evidence["input_bytes_per_session"]) == sessions
            and all(value >= (1 << 20 if evidence["tier"] == "ci-10-minute" else 8 << 20)
                    for value in evidence["input_bytes_per_session"]),
        "input/output byte identity": evidence["input_bytes_per_session"]
            == evidence["output_bytes_per_session"],
        "exact resize count": evidence["resize_requests"] == resize_count,
        "all scheduled workload roles exercised": {role["name"] for role in evidence["roles"]}
            == {"agent-burst", "status-update", "full-screen", "resize", "lifecycle",
                "saturation", "interactive", "idle"}
            and all(role["cycles"] > 0 for role in evidence["roles"]),
        "zero original-SID survivors": not evidence["survivors"],
        "noisy p95 <=50ms": latency["p95_us"] <= 50_000,
        "noisy p99 <=100ms": latency["p99_us"] <= 100_000,
        "noisy max <=250ms": latency["max_us"] <= 250_000,
        "resize p95 <=2ms": resize["p95_us"] <= 2_000,
        "resize p99 <=5ms": resize["p99_us"] <= 5_000,
        "resize max <=20ms": resize["max_us"] <= 20_000,
        "RSS <=baseline+192MiB": peak_rss <= baseline["rss_kib"] + 192 * 1024,
        "FDs within governed shape": peak_fds <= baseline["file_descriptors"] + sessions * 4 + 8,
        "workers within governed shape": peak_threads <= baseline["threads"] + sessions * 4 + 4,
        "closed FDs return to baseline+2": evidence["final_file_descriptors"] <= baseline["file_descriptors"] + 2,
        "closed workers return to baseline+2": evidence["final_threads"] <= baseline["threads"] + 2,
    }
    failed = [name for name, passed in checks.items() if not passed]
    if failed:
        raise SystemExit("DTC-P06D owner threshold failures: " + ", ".join(failed))
    print(f"DTC-P06D soak evidence verified: {path}")
    raise SystemExit(0)
if evidence.get("contract") != "datum_terminal_p06_measurement_v1":
    raise SystemExit("invalid DTC-P06D evidence contract")
idle = evidence["idle_latency"]
resize = evidence["resize_latency"]
single = evidence["single_output"]
aggregate = evidence["aggregate_output"]
resources = {sample["label"]: sample for sample in evidence["resources"]}
baseline = resources["warm_baseline"]
peak = resources["eight_session_peak"]
after = resources["after_close"]
checks = {
    "idle p95 <=25ms": idle["p95_us"] <= 25_000,
    "idle p99 <=50ms": idle["p99_us"] <= 50_000,
    "idle max <=100ms": idle["max_us"] <= 100_000,
    "resize p95 <=2ms": resize["p95_us"] <= 2_000,
    "resize p99 <=5ms": resize["p99_us"] <= 5_000,
    "resize max <=20ms": resize["max_us"] <= 20_000,
    "single throughput >=20MiB/s": single["mib_per_second"] >= 20.0,
    "aggregate throughput >=40MiB/s": aggregate["mib_per_second"] >= 40.0,
    "eight-session RSS <=baseline+192MiB": peak["rss_kib"] <= baseline["rss_kib"] + 192 * 1024,
    "eight-session FDs within governed shape": peak["file_descriptors"] <= baseline["file_descriptors"] + 8 * 4 + 8,
    "eight-session workers within governed shape": peak["threads"] <= baseline["threads"] + 8 * 4 + 4,
    "closed FDs return to baseline+2": after["file_descriptors"] <= baseline["file_descriptors"] + 2,
    "closed workers return to baseline+2": after["threads"] <= baseline["threads"] + 2,
}
failed = [name for name, passed in checks.items() if not passed]
if failed:
    raise SystemExit("DTC-P06D owner threshold failures: " + ", ".join(failed))
print(f"DTC-P06D evidence verified: {path}")
PY
