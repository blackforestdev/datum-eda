# Current investigative baseline and first bounded correction

Recorded 2026-09-18. Candidate: `2874a8e1b20f7e55029b8dadbd64decb9e2be94e`.
Optimized GUI built with the guarded runner. Source was unchanged during the
initial baseline; planning/claim records were dirty. Binary and fixture hashes
are recorded in each report. This is current **investigative evidence**, not
completion of GPS-C01's full daily-Linux workload matrix or product qualification.

## Reproduction and evidence

Receipts live in `docs/reviews/gui-performance/measurements/`: exact driver
snapshots, per-trial CPU/RSS/DRM counter observations in `report.json`, compressed
raw timing logs, and captured window PNGs. `environment.json` records compiler,
kernel, CPU and installed GPUs. Each run copied the real DOA2526 KiCad board to
a disposable directory and isolated config/cache/TMPDIR. Engine-generated state
stayed outside scanned fixtures. Session-owned windows/processes were closed.

Build: `python3 scripts/run_cargo_guarded.py --workload proof -- cargo build
-p datum-gui-app --release`. Driver invocation: `python3 <driver> --mode main
--seconds 10 --trials 3 --trace --output <fresh-directory>`. The driver paths and
output labels identify each recipe below. `DATUM_TRACE_TIMING=1` records existing
production timing/cache observations. No new dependency or native input bypass
was introduced.

Environment: KDE Wayland desktop, X11/Xwayland GUI backend selected for existing
XTest input tooling, 1280x800 main window, Intel HD Graphics P630/i915 process
render-engine counters, Xeon E3-1505M v6. The other installed adapter is NVIDIA
Quadro M620. These results do not qualify a native Wayland event path, another
adapter, display refresh/presentation latency, or a demanding scale tier.

## Initial results and rejected measurements

| Receipt | Workload | CPU % of one core, three trials | Trace observation / disposition |
|---|---|---|---|
| baseline-global-v1 | Editor pointer/zoom with Preferences open | Not a valid editor workload | Input-modal focus redirection prevents intended owner input; retain failed measurement |
| baseline-global-v1 | Preferences scroll | 1.40 / 1.30 / 1.40 | 20 dialog submissions per trial, zero main submissions; reaches bounds frequently, not continuous-scroll qualification |
| baseline-main-v1 | Zoom targeted at x=600 | Not a valid zoom workload | Pointer on board/schematic divider; zero zoom frames; reject as target error |
| baseline-main-v2 | Board pointer, ~60 sent motions/s | 10.30 / 10.20 / 10.20 | 528 / 535 / 538 frame traces; zero prepared/retained cache misses |
| baseline-main-v2 | Board zoom, ~10 sent wheel events/s | 5.30 / 5.50 / 5.40 | 177 / 184 / 187 prepared misses; zero retained misses |
| baseline-boundary-v1 | Repeated wheel at upper zoom clamp | 4.90 / 5.00 / 5.30 | 178 / 187 / 184 prepared misses/submissions; zero retained misses; PNGs identical, but capture method later rejected as pixel proof |

For the corrected main workload, the pointer stays inside the board pane
(x=350..449, y=350..399); zoom targets (420,400). Clamp preconditioning sends
40 upward wheel events before sampling. The actual scripted count is retained;
OS/native event delivery can differ from XTest request count. In particular the
zoom trace contains more frames than sent button pairs, so it is not valid to
call the request count an acknowledged application-event count or divide it into
a claimed delivered frame rate.

Warm pointer render-engine duty was 17.98 / 19.07 / 18.53%; clamp duty was
10.34 / 11.70 / 11.52%. This is per-client DRM active time, not energy or
frequency-normalized utilization. Idle portions observed zero submissions in
the corrected main run, but ten seconds is insufficient for the 60-second idle
qualification rule. Trace logging is enabled; absolute budgets and small
percentage improvements require paired trace-off measurements.

## Selection and alternatives

| Candidate | Evidence | Benefit, cost and falsifiable check | Disposition |
|---|---|---|---|
| Suppress unchanged-camera invalidation | Clamp trials consume CPU/GPU and rebuild frames with identical pixels; `runtime_camera_pane.rs` unconditionally invalidates | Exact post-operation state comparison; zero clamp frames, reversed/in-range zoom still works; no extra cache or dependency | First bounded correction |
| Refine all window invalidation | `app_shell.rs` broadcasts invalidation; initial modal run cannot demonstrate active editor plus Preferences interaction | Potential unrelated-window savings; needs representative background/update route, not bypass of modality | Remains candidate, not selected first |
| Reuse more screen-space uploads / frame work | Pointer retains prepared/world scenes but still consumes resources; `gpu_vertex_upload.rs` writes several buffers each frame | Needs actual upload-byte and encoding counters; added cache identity/lifetime complexity | Measure next before choosing mechanism |
| Eliminate New Project hidden workspace preparation | `global_preferences_window.rs` clones workspace/builds world in New Project branch; Preferences already avoids it | Bound dialog-only work and retain hit/visual parity; cost not yet measured here | Later measured adoption candidate |
| Replace renderer/framework or add render thread | No measured necessity; working retained renderer and shared toolkit exist | Large migration/coordination/resource cost with no demonstrated priority benefit | Not justified |

The relevant prior art supports investigating retained work and avoiding copies,
not adopting another framework: [Qt's renderer documentation](https://doc.qt.io/qt-6/qtquick-visualcanvas-scenegraph-renderer.html)
describes batching and retained GPU geometry; [wgpu Queue](https://docs.rs/wgpu/28.0.0/wgpu/struct.Queue.html#method.write_buffer)
documents staging-copy behavior; [winit Window](https://docs.rs/winit/0.30.13/winit/window/struct.Window.html#method.pre_present_notify)
documents presentation notification. These are external behavioral references,
not dependency authorization. Datum's existing decisions continue to control.

## Narrow proof and remaining coverage

The preimplementation contract and positive/negative proof are recorded in
`docs/reviews/gui-performance/zoom-noop-slice.md`. The old production route is
the negative candidate. Candidate measurement adds a same-instance reverse-wheel
control after each clamp trial, outside its timing interval, to reject a false
zero-work result caused by lost input. Unit tests additionally preserve the
existing anchor math and small representable changes.

The first candidate reverse control failed: ImageMagick's X11 window capture
returned identical decoded pixels despite changing render traces. This invalidates
that capture method as displayed-output evidence, including the initial baseline
PNG equality. The failed run is retained as `candidate-boundary-v2-rejected`.
The existing KDE Spectacle active-window compositor capture then passed the
same test: clamp pixels stayed identical, reverse input changed visible board
pixels and generated six production frame traces. The final paired recipe uses
that capture method, checks the focused window is session-owned before capture,
compares decoded RGBA bytes and records the binary hash before launch. Captures
occur outside resource timing intervals. This corrects measurement tooling; it
is not a product rendering fix or proof of presentation latency.

Cross-run reversal comparison also exposed path-sensitive import identity:
identical source bytes produced different object IDs and UUID-based painter
order. The initial comparison differed at 428 central pixels near overlapping
copper/via geometry. A baseline replay using the candidate's exact disposable
input path produced identical drawable model fields (only the root board UUID
differed), and exact central before/after/reverse pixel equality. See
`measurements/matched-model-pixels.json`. The comparison excludes only the top
34 and bottom 60 pixels containing changing project revision labels; all central
design/pane content is compared. Per-run clamp equality still compares full
window RGBA. This independent issue is captured as
`dat-import-path-painter-order-u6i`, not repaired by this slice. Future baselines
must pin the resolved model's identities/order, not merely the import file hash.

Missing full-baseline work remains explicit: native Wayland input, panning,
pane/window transitions, switching, sustained non-boundary Preferences scrolling,
terminal mixed activity, demanding fixtures/scale counts, engine-process CPU,
live GPU allocations/upload bytes, compositor presentation and input-to-display
accounting, precise trace overhead, longer samples and endurance. No missing item is
declared passed. Complete feasible proof methods and tier budgets are required
before full specification approval; actual complete adoption/endurance results
are required at final implementation acceptance. The measured clamp defect does
not require waiting for that unrelated final production proof before correction.

The final logging-disabled matched-input comparison preserves the measured clamp
resource improvement and all compositor positive controls. Raw paired reports
and their summary are retained under `measurements/*-boundary-trace-off` and
`measurements/comparison.json`; frame counters are explicitly unavailable in
those runs. This checks that the optimization is useful without trace logging;
it does not establish a precise trace overhead or expand qualification scope.
