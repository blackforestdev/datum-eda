# Current-source host-resize CPU finding (GPI-CPU-RESIZE-INVESTIGATE)

<!-- EVIDENCE:GUI-PERFORMANCE-IMPLEMENTATION:CPU-RESIZE-FINDING -->

Status: completed bounded execution of the owner-approved, PM-re-pinned resize
packet (option b). This is an explicitly **current-source** reproduction of the
host-resize CPU behavior for the shipping code — **not** an R4 replay. No
numeric budget, threshold or qualification is decided (PM051). The downstream
owner gate **GPI-CPU-RESIZE-DISPOSITION remains pending**; resize qualification
stays separately nonblocking.

- Authority: `gpi-resize-repin-decision.json` (commit 9bc2c86f,
  `EVIDENCE:GPI-RESIZE-REPIN:2026-10-29`), under the approved packet in
  `resize-investigation.md`.
- Artifact provenance: `resize-repin-delta.json`.
- Raw evidence: `../measurements/resize-current-source/` — `harness.py`,
  `finding.json`, `launch1-20261029T204831Z/` (ordinary), and
  `launch2-20261029T205233Z/` (probe). A preserved pre-measurement attempt
  (`launch1-20261029T204727Z/`) failed before any measurement on a relative
  `XDG_CONFIG_HOME` the engine rejected — a recorded harness correction, not a
  measured failure.

## Method (two launches, no rebuild beyond the one authorized build)

Real product `datum-gui` (release opt-level 3, features `[visual]`, source
`8afa4062`, sha256 `47a9082f…`) launched on the DOA2526 board (sha256
`7df67d42…`, drifted from the R4-era pin — see `resize-repin-delta.json`) at
1280×800, Wayland (verified `window_backend=wayland`, Intel/Vulkan). The GUI ran
inside a fresh owned cgroup v2 scope (`systemd-run --user --scope`); application
CPU is `cpu.stat`, KWin CPU is accounted separately from `/proc`. Per axis: a
~60 Hz triangular 0..180 px native-window resize (KWin declarative geometry
injection) for ~10 s, 600 planned updates, bracketed by 10 s quiet control and
tail. Launch 1 is ordinary (no verbose/trace/GPU/minimal knobs). Launch 2
repeats each axis once with `DATUM_RESIZE_CPU_PROBE=1` (existing per-phase CPU
clocks only).

## Result 1 — the cost reproduces on current source

| Phase (Launch 1, ordinary) | App CPU %/core | KWin %/core | peak 100 ms %/core | resize applies | extent grew |
|---|---|---|---|---|---|
| quiet control | 0.04 | 0.0 | 0.07 | 0 | — |
| width resize | 34.59 (user 32.26 / sys 2.33) | 16.08 | 56.01 | 593 | yes |
| height resize | 32.15 (user 29.56 / sys 2.59) | 16.88 | 48.71 | 597 | yes |
| quiet tail | 0.04 | 1.59 | 0.06 | 0 | — |

A substantial, resize-correlated application CPU cost (≈ a third of one core
sustained, ~0.04 %/core at idle), predominantly user-time, plus ~16 %/core of
compositor CPU, reproduces on both axes. Changed extents were verified in the
ordinary `resize apply` records and all four captures are distinct (real render
response, not stale). This resembles the owner-reported concern. It does not
compare to the owner's unspecified-boundary >72 % peak, and KWin geometry
injection is native-window resize delivery, not physical decoration dragging —
non-reproduction would not have refuted a drag-specific observation, and this
reproduction does not quantify one.

## Result 2 — localization (Launch 2, diagnostics on)

Per-phase process CPU clocks over each ~10 s active axis. **Phases nest**
(`event_round` ⊃ `renderer`, with `prepare`/`acquire`/`present` inside the round)
so they are **not** summed into an exact partition; shares are of measured app
CPU in the window, shown for magnitude only.

| Probe phase | width: calls / process-ms / % app CPU | height: calls / process-ms / % app CPU |
|---|---|---|
| event_round (outer) | 2828 / 3436.8 / 87.5 % | 2712 / 2879.6 / 84.9 % |
| **renderer** | 451 / **2625.6 / 66.9 %** | 395 / **2054.8 / 60.6 %** |
| prepare | 451 / 475.7 / 12.1 % | 395 / 495.1 / 14.6 % |
| acquire | 451 / 58.7 / 1.5 % | 395 / 56.5 / 1.7 % |
| present | 451 / 57.2 / 1.5 % | 395 / 54.1 / 1.6 % |
| configure | 2 / 5.0 / 0.1 % | 2 / 5.6 / 0.2 % |
| outside event_round | — / 488.9 / 12.5 % | — / 510.3 / 15.1 % |

Observations (descriptive, not thresholds):
- The dominant resize CPU is the **renderer phase** — a full product scene
  re-render per resize frame (~395–451 renders over ~10 s), **main-thread**-bound
  (`process_ms ≈ thread_ms` throughout, so little worker-thread CPU). This
  matches the reviewed path: each native resize forwards to `Runtime::resize` →
  `invalidate_surface_size` → `resize_content`, and `runtime_present.rs::render`
  re-runs the shared renderer every frame.
- **Surface configuration is negligible** (`configure`: 2 calls, ~5 ms).
  Frame-bound configuration admits only changed extents at the frame boundary, as
  the source review expected; the cost is **not** per-update reconfiguration.
- `prepare` is a consistent second (~12–15 %); `acquire`/`present` are minor.
- ~12–15 % of app CPU falls outside any `event_round` window and stays
  explicitly uncovered.
- Diagnostics-on totals (35.4 % / 30.6 %) are not ordinary performance and are
  not an observer-overhead experiment.

## Disposition this enables (owner — GPI-CPU-RESIZE-DISPOSITION)

A **supported current cause with a bounded remaining gap**:

- **Supported:** the current-source host-resize CPU cost is real and is
  localized to full per-frame **scene re-rendering** on the main thread, not to
  surface configuration or compositor delivery. Each resize frame drives a
  complete renderer pass.
- **Remaining gap (method-bounded, in scope to state, out of scope to close):**
  the `renderer` window wraps scene build, encode, draw and submission together,
  so these phase clocks do **not** attribute the within-renderer dominant
  sub-cost (e.g. scene re-encode vs. necessary draw vs. queue submission). The
  historical undercovered sampling profiler and privileged kernel tracing are
  outside this packet and were not used. Whether the per-frame full render is
  reducible (retained/cached reuse) is the shared-renderer question owned
  elsewhere, not decided here.

No repair, optimization or further campaign is implied; any follow-up requires a
separate concrete owner scope.

### Tracking and lineage (corrected)

The original resize bug `dat-gui-vertical-resize-cpu-toj` is **CLOSED**
(2026-10-01) on an owner-approved **bounded default-workspace buffer-reuse win**
(recorded close commit `a89f94b2`); its close note explicitly directs continued
current-host resize investigation through exactly this lane. This finding and
that close **coexist**: the Oct-1 bounded win stands, and this current-source run
shows the host-resize CPU cost **persists** at ~1/3 of one core after it — no
implication that the close was wrong. Live tracking is the **GPI-CPU-RESIZE
frontier lane** plus the pending `GPI-CPU-RESIZE-DISPOSITION` gate; the persisting
cost is captured for separate repair scoping in bead **`dat-g2jg`** (open,
unassigned, unscheduled). An earlier draft of this finding incorrectly stated the
`toj` bead was open; corrected here per audit of `f0f3d248`.

### Known evidence defects (disclosed)

- Launch 2 `report.json` mislabels the two resize phases as `event_round` (a
  harness variable-shadow slip in the probe parser, fixed in `harness.py`
  after the run; CPU data unaffected). `finding.json` carries the corrected
  labels, recovered from each phase's `first_apply`.
- Launch 1's inter-axis endpoint restore used the 1280×800 content size rather
  than the ~1344×840 KWin frame base, so `verified_restored` reads false; the
  per-axis triangular driver self-restores to its own captured original, and the
  CPU windows are unaffected. Removed from the Launch 2 path.
