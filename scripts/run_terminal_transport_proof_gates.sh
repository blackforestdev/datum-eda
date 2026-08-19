#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

tier="${1:-smoke}"
case "$tier" in
  smoke) ;;
  *)
    echo "usage: $0 smoke" >&2
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
revision="$(git rev-parse HEAD)"
evidence_dir="${DATUM_P06_EVIDENCE_DIR:-target/p06-evidence}"
if [[ "$evidence_dir" != /* ]]; then
  evidence_dir="$repo_root/$evidence_dir"
fi
evidence_path="$evidence_dir/${revision}-${tier}-${backend}.json"
mkdir -p "$evidence_dir"

echo "DTC-P06D tier=$tier backend=$backend revision=$revision seed=$seed"
DATUM_P06_TIER="$tier" \
DATUM_P06_SEED="$seed" \
DATUM_P06_EVIDENCE="$evidence_path" \
cargo test -p datum-gui-app --release --locked --offline \
  terminal_session::terminal_session_p06_measurement_tests::p06_release_measurement_emits_reproducible_json \
  -- --ignored --exact --nocapture --test-threads=1

python3 - "$evidence_path" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
evidence = json.loads(path.read_text(encoding="utf-8"))
if evidence.get("contract") != "datum_terminal_p06_measurement_v1":
    raise SystemExit("invalid DTC-P06D evidence contract")
if evidence.get("failures"):
    raise SystemExit(f"DTC-P06D evidence reports failures: {evidence['failures']}")
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
