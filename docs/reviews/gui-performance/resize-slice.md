# Rapid native-window resize correction

<!-- REQ:GUI-PERFORMANCE-SPEC:GPS-C01R -->

Status: historical partial correction. The owner has reopened GPS-C01R as the
active resize defect until resolved; see `resize-recovery.md`. The measurements
below do not establish acceptable resize resource use or smooth presentation.

<!-- EVIDENCE:GUI-PERFORMANCE-SPEC:GPS-C01R-RESULT -->
Issue: `dat-gui-vertical-resize-cpu-toj`, related to `dat-gui-performance-spec-u9n`.

The owner reported vertical and horizontal stretching above 72% on one core in
an optimized build and explicitly directed reproduction, diagnosis and a fix.
GPS-C01 remains complete. GPS-C01R is this bounded correction before GPS-C02;
it does not approve the complete specification or accept the rendering engine.
All HP01–HP25 obligations remain, with relevant proof per implementation slice
and complete adoption/endurance reserved for final acceptance.

## Proof defined before implementation

Use unchanged optimized baseline and candidate binaries, the same disposable
board/import identity, isolated configuration and session-owned windows. After
warmup, request each resize axis at approximately 60Hz for three ten-second
trials, with quiet idle controls. Preserve raw process CPU samples, child CPU,
DRM engine counters, diagnostic build/frame counts, binary/source/fixture hashes
and compositor screenshots. Separate traced diagnosis from logging-disabled
resource trials; exclude compilation and reject unrelated input contamination.

Choose the correction from measured work and verified dependencies. Preserve
final dimensions, layout, hit testing, camera, geometry appearance, terminal
cell/PTY dimensions and surface recovery. Do not suppress valid frames simply
to lower utilization. Compare fresh and reused geometry, including genuinely
size-dependent negative controls; compare matched final pixels. Build optimized,
run affected behavioral tests and native smoke, and obtain independent review.
Controlled X11/Xwayland resize requests are not native decoration-drag latency
qualification. Numerical budgets, demanding tiers and endurance remain open.

## Implemented boundary

Every changed physical size previously discarded all retained world geometry.
The baseline traced 1310 rendered frames and 1310 retained misses; median retained
construction was 7ms. Physical resizing now preserves active retained geometry
only when construction has no reference-projection-size dependency. Current
production exceptions are unrouted primitives and closed mechanical component
graphics (endpoint/width and dash/gap lengths). The conservative predicate rejects
reuse even when these primitives are hidden; board and schematic classify their
own scenes independently. Future reference-scale consumers must extend this
predicate and the negative-control coverage.

Prepared projections/layout and old-size cache history are still discarded.
DPI and content changes retain full invalidation. Surface configuration, input
dimensions and terminal resizing retain their original immediate timing. The
cohesive `runtime_surface.rs` module owns dimensions/configuration/DPI behavior;
main.rs falls from 2224 to 2165 pre-test lines. Retained construction moves into
the existing `scene_retained_access` module, reducing gui-render/lib.rs logical
expanded size from 8379 to 8288 lines. No dependency, renderer replacement,
protected prototype, appearance policy or frame throttling is introduced.

## Measurements

The accepted logging-disabled runs use three ten-second trials per axis. CPU
is GUI process CPU summed across threads, expressed relative to one core;
it is not the owner's physical-core monitor metric.

| Metric | Vertical baseline → candidate | Horizontal baseline → candidate |
|---|---|---|
| Median CPU | 40.20% → 34.90% | 39.40% → 34.20% |
| Largest approximately 100ms CPU sample | 63.40% → 56.95% | 64.20% → 56.01% |
| Median DRM render-engine active time | 14.53% → 15.56% | 15.23% → 14.80% |

CPU medians decrease about 13% in both axes. The matched request recipe yields
560–563 baseline versus 555–559 candidate requests per trial, not identical
event-by-event delivery. GPU results are mixed: vertical duty
increases 1.03 percentage points, horizontal decreases 0.43. This is not evidence
of universally lower GPU cost or fulfillment of the final resource requirement.
Three candidate quiet-idle trials record zero GUI CPU ticks; the fourth records
one tick (0.01 CPU seconds, approximately 0.10%). All four record zero surviving
DRM-engine activity at this sampling resolution. All accepted resize
trials used for the active comparison, and all candidate idle trials, have zero
observed unrelated keyboard/button/motion events. Baseline idle sample 7 contains
41 motion events and 0.30% CPU and is excluded from quiet-idle claims. These short
idle samples do not constitute endurance proof or close prior idle anomalies.

The final traced diagnostic records 232 vertical and 246 horizontal frames,
with zero retained misses in either axis. Traced CPU is diagnostic only. The
same fixture bytes and normalized model are verified; only the generated root
UUID is excluded from model comparison, preserving all object IDs and ordering.
Compositor images at vertical, horizontal and restored dimensions match pixel
for pixel outside the two displayed revision identifiers. Those six-character
labels differ because the disposable imports have different root revisions;
no product golden or visual authority is changed.

Rejected experiments remain available: `baseline-plain` overlapped compilation;
`baseline-plain-v2` received unrelated input; the coalescing-only candidate raised
CPU to 48.7%/47.5% medians; coalescing plus reuse increased GPU duty to about 22%.
The selected implementation restores original configuration timing and keeps
only dependency-aware reuse. The five-second reuse-only pilot informed selection
but the final three-trial runs are the reported comparison. Untraced v3 pilot
prepared/retained counters of zero mean unobserved, not proof of zero work; v4
corrects them to null unless the required logging is enabled.

## Verification and limits

All 173 renderer unit tests passed, including four new resize cases: nonempty
size-independent geometry equality across both axes, visible scale-dependent
airwire and mechanical-dash negative controls, and separate schematic ownership.
These compare vertices, strokes, draw commands and board world-hit data.
Three terminal focus regressions and seven dock tests also passed. An initial
filename-based dock filter matched zero tests; the corrected Rust module filter
ran all seven, with both invocation logs preserved. Native
burst/DPI smoke passed in author-recorded X11 and Wayland launch configurations
(the receipts alone do not independently identify the actual backend): eighteen
immediate configurations, seven presentations and only the DPI retained rebuild;
no recovery or timeout. A separate visible-terminal probe compared layout with
read-only kernel PTY dimensions after both axes and restoration: 155×9, 186×9, 155×9,
all matching. PTY pixel dimensions are zero and split terminals are not qualified.
Independent review supports bounded completion after correcting idle/backend
reporting and passing focused terminal regressions. Workspace Clippy, optimized
build, formatting, alignment, source-health, dependency, roadmap, parity and
evidence checks passed. The required drift suite failed at the existing terminal
convergence guard: focus assignment count and font source-marker expectations.
The same two messages reproduce against unchanged HEAD; tracked separately as
`dat-terminal-convergence-guard-xke`. Subsequent drift stages were not reached
(terminal proof families, GUI conformance/reference, menus, migration and final
MCP checks); they are not claimed as passed. The separate earlier MCP self-test
ran 415 tests successfully with 45 skipped. No gate was bypassed or weakened.

Raw evidence, source/binary hashes, driver snapshots, rejected receipts and
screenshots are indexed under `measurements/resize/` by `measurements/SHA256SUMS`.
DRM activity is not frequency-normalized GPU utilization or energy. CPU samples
have 10ms tick quantization. Sequential same-host runs are not randomized trials.
X11/Xwayland stress and native smoke do not qualify physical Wayland decoration
interaction, presentation latency, all scene types, demanding boards or endurance.
Scenes with reference-scale dependencies intentionally retain their old rebuild
path. The QA issue remains open for remaining resource and native-interaction
qualification; this slice must not be described as eliminating every CPU spike.
