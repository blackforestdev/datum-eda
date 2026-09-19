# GUI performance acceptance matrix

Status: GPS-C03 working draft. Not complete, ratified, or runtime-qualified.
Parent: `GUI_PERFORMANCE_RECOVERY_PLAN.md`, R11–R20/R38/R40–R41.
Engineering definitions: `GUI_SHARED_ENGINEERING_CONTRACT.md`, E01–E12.
The numerical targets below are proposals for review, not measured achievements.

## Reference and fixture admission

Use the recorded daily reference: Xeon E3-1505M v6, Intel P630/i915,
Debian kernel 6.12.107+deb13-amd64, Mesa 25.0.7-2+deb13u1, KWin 6.3.6,
wgpu 28.0.0 and winit 0.30.13. Pin actual runtime adapter/backend/driver and
refresh/scale for every trial; these historical versions are not an instruction
to downgrade a changed machine. Native Wayland and X11/Xwayland are separate rows.
The NVIDIA adapter, other drivers/displays and other platforms remain unqualified.
Release profile and features must match between baseline and candidate; the
owner's command without `--release` is a separate development-profile run.

Existing representative fixture **F-DOA** is the real DOA2526 baseline:
source SHA256 `7df67d426cc0cd808096824b0b7a05dcde351a1305fd41244d8c05a15801c08d`,
normalized resolved board SHA256
`33e62de1c1da2020f0608444a4802eac23fb97a9f56cc8cf87c844b6077499ed`.
The latter excludes only root UUID, preserving object identities and ordering.
Counts: 39 packages, 81 pads, 148 tracks, 46 vias, 20 zones, 24 nets and 24
layer declarations. Source: `measurements/c01/environment.json` under
`docs/reviews/gui-performance/`. Baseline import identity must be reproduced
from the archived copy, not whatever bytes now occupy the owner's live path.

Do not call these GPU primitive, visible-object or glyph counts. Admission must
record those separate counts from the actual resolved/rendered scene, as well
as text runs/unique glyphs, total geometry bytes and visible geometry bytes.
A board-only schematic placeholder does not qualify a schematic workload.

| Tier | Required content/configuration | Present disposition |
|---|---|---|
| T0 | Independent minimal native grid, no model; then empty product shell | Existing diagnostic isolation, no product or temporal acceptance |
| T1 | F-DOA, one/two/four visible leaves; main plus each real dialog; terminal hidden/visible | Board investigative baseline available; complete scale/glyph accounting and native qualification pending |
| T2 | Demanding real resolved board and schematic, with pinned source/model hashes and measured geometry/text counts; four leaves plus permitted auxiliary hosts | Required coverage gap: local populated boards found have only 11–48 footprints; no demanding enterprise tier is established |

T2 requires an actual qualifying project or explicit owner-limited initial scope
under R12/R14. Repeating a small project across panes exercises multi-pane cost,
but does not establish large-document capacity. No fabricated project or
unsupported enterprise-capacity claim substitutes for that decision.

## Workload definitions

Every row runs independently on both daily native backends at 60 Hz, 1x, 1.5x
and 2x where the display supports those configurations. Unsupported display
combinations are explicit missing/unsupported rows, not silently rescaled results.
Default client size is 1280x800 physical pixels; record native enforced minimums
and actual accepted dimensions before timing. Keep production modality intact.

| ID | Exact proposed schedule after five-second warmup | Expected state/output |
|---|---|---|
| W-IDLE | 60 s no input, terminal animation disabled or separately classified; repeat three times | No application-generated submission; unchanged content; no hidden polling loop |
| W-POINTER | 30 s, 120 pointer positions/s on a 400x240 px rectangle inside one pane; four equal-duration edges; then 5 s still | Correct hover/crosshair/hits and final position; no shell solves or world uploads for unchanged dependencies |
| W-ZOOM | 30 s, 60 finite fractional line-wheel deltas/s, +0.1 line for 2 s then -0.1 line for 2 s repeatedly; one target pane | Camera follows every accepted delta using existing anchor math; unrelated cameras stable |
| W-CLAMP | Reach each camera bound outside timing; 30 s of outward wheel at 60/s; one inward reversal in a separately timed positive-control window after the no-op trial | Zero camera-induced frames while identical, immediate representable reversal; independent console/menu damage counted |
| W-PAN | 30 s, 120 pointer positions/s on same rectangle with established pan gesture held, then release | Camera/hit projection and final gesture state correct; no unchanged world upload |
| W-SCROLL | Actual Global and Project hosts separately: 30 s at 120 pixel deltas/s, +0.25 for 2 s then -0.25 for 2 s; repeat at both boundaries; separate line-delta recipe uses ±0.25 line at 120/s and pins the existing 40 logical pixels/line × scale conversion exactly once (native positive y moves offset negatively) | Fractional sum, boundary reversal, local redraw, pinned chrome and clip/hit parity |
| W-CONTROLS | Each Preferences host: focus/reveal last row, expand, search, clear search; thumb grab at 25/75%, track page, focus-loss cancel; 30 cycles | No clipped-control activation; focus and bounds correct after each transition |
| W-PANES | 30 s: every 500 ms alternate split, switch, close, switch; start/end two leaves | Correct PaneId/focus/camera retention; no resource leak |
| W-WINDOWS | Each of Global, Project Preferences and New Project: open, wait for visible readiness, close/cancel, wait for closure; repeat 30 cycles | Actual adapter exercised, no project created, focus restored, resources released |
| W-RESIZE | Each axis and corner separately: triangular native size requests at 60/s for 30 s, 2 s period, 1280↔1536 width / 800↔960 height; reverse and restore; separate physical-decoration drag recording | Continuous correct interior content and final native extent; no blank/jump, lost final damage or unrelated host work |
| W-LIFECYCLE | 20 sequences: maximize/restore, minimize/restore, display/DPI move, suspend/resume where supported, injected faults from REC matrix, close while pending | Exact REC transitions, preserved data/focus, bounded retries and resource release |
| W-MIXED | W-ZOOM or W-SCROLL with terminal output from a bounded recorded 64 KiB/s PTY stream; 4 KiB bursts, plus a one-second 1 MiB burst | No byte loss or input starvation; terminal rows remain terminal-owned; hidden terminal prepares nothing |

W-RESIZE runs T0 minimal, shell, T1 and admitted T2, plus each native dialog,
visible/hidden terminal, and other unchanged windows present. When production
modality prevents editor interaction, resize via supported native geometry
operations and classify it separately from physical input. Never disable modality.

## Measurement rules and proposed numerical targets

**MET-01.** Three independent 30-second warm trials per active workload, with
five-second warmup and return-to-idle tail; W-IDLE and cycle counts override that
duration. Alternate baseline/candidate order across trials. Preserve failures,
raw schedules, receipts, logs and binary/model hashes. Cold startup/first use
are separate ten-start distributions; import/engine startup cost is reported,
not subtracted from a claimed end-to-end launch result.

**MET-02.** CPU duty = 100 × sum of all process-thread CPU seconds / actual elapsed
seconds. Include kernel time. Report cooperating engine CPU separately and sum
GUI+engine for total application limits. CPU/action divides total timed CPU by
*acknowledged semantic actions*, not submitted frames. Also report scheduled and
received counts; coalesced pointer samples retain path/final-state accounting.
Use `/proc` process counters or existing direct CPU clocks, retaining raw values,
clock tick resolution and PID lifetime. Sample descendants and collect exit
accounting; a disappeared unaccounted process makes the total incomplete.

**MET-03.** Proposed warm limits below apply per trial to total GUI+engine CPU.
At 60 semantic updates/s, a 10% duty budget allows 1.667 ms CPU per update;
this is a target substantially below the rejected 32–35% resize result, not an
assertion that present driver costs already meet it. Required output/latency is
unchanged. T2 numbers cannot be ratified before fixture admission and rationale.

| Workload | CPU duty ceiling | CPU/action ceiling | GPU engine duty ceiling | GPU execution p95/p99 target |
|---|---:|---:|---:|---:|
| W-IDLE | 1% over 60 s | Not applicable: no actions | Zero Datum-attributed active delta within documented counter resolution | No application work |
| W-CLAMP | 1% | 0.167 ms at 60/s | Zero camera-induced work during outward no-op window | No camera-induced work during outward no-op window |
| T0 minimal/shell resize | 5% | 0.833 ms at 60/s | 5% | 0.8/1.6 ms |
| T1 pointer/pan/zoom/resize | 10% | 1.667 ms at 60/s; 0.833 ms at 120/s | 25% | 4/8 ms |
| Preferences scroll/controls | 5% | 0.417 ms at 120/s; controls measured separately per completed transition | 10% | 1.6/3.2 ms |
| W-PANES | 10% | 50 ms at 2 actions/s | 25% | 4/8 ms |
| W-WINDOWS | Report duty over actual cycles | Warm open ≤50 ms CPU; close ≤20 ms CPU | 25% during active cycles | 4/8 ms |
| W-MIXED T1 | 15% total | Report input and PTY-byte denominators separately | 30% | 5/8 ms |

Controls without a fixed action rate use a proposed 5 ms CPU/transition ceiling;
the duty ceiling applies to the timed stream, not an artificially extended idle
denominator. Lifecycle failure workloads use REC bounded-work rules and idle
limits while undrawable; recovery targets are defined in MEM-02; successful recovery uses the T1 4/8 ms
GPU execution limits per submitted frame. Faults do not excuse unbounded retries.

**MET-04.** DRM active nanoseconds / elapsed nanoseconds gives engine duty, not
energy, frequency-normalized utilization or per-frame execution. Sum distinct
clients without double-counting duplicate fds; collect final counters before
client close. Counter loss makes window-cycle totals inconclusive. Per-frame GPU
execution is the timestamp span from the first frame GPU pass beginning through
the last frame GPU pass ending, including intervening GPU commands but excluding
CPU acquisition and queue waiting before GPU execution. Also record individual
pass durations; their sum is a different metric and must not replace the span.
For multiple submissions, retain a single frame ID, timestamp each submission,
and report the first-start/last-end span plus the sum of this frame's GPU pass
durations separately. Intervening other-host work or gaps are included in the
span and identified, not charged as exclusive execution. Missing a submission
instrumentation boundary makes that frame observation incomplete.
It needs timestamp queries resolved asynchronously outside the normal frame path, with capability/period/disjoint handling and calibration. This is a
specified future bounded measurement change using existing wgpu, not an already
validated method. Unsupported timestamps need an independently validated alternate
or explicit owner-limited qualification under R14; duty cannot silently replace it.

**MET-05.** CPU event/preparation/encoding p95≤4 ms, p99≤8 ms; no unexplained warm
application stall >50 ms. Input-to-observed-display p95≤33.4 ms and p99≤50 ms;
warm open/close/switch p95≤100 ms and p99≤150 ms. At 60 Hz, ≥95% expected display
opportunities have a current eligible update within 20 ms, and no final accepted
warm continuous pointer/camera/scroll/resize state remains undisplayed after 50 ms
without a classified external interruption. Open/close/switch and recovery instead
use their explicitly separate latency limits.
Count scheduled input and missed opportunities, including intervals with no
successful frame. Compute nearest-rank percentiles (sorted sample ceil(p×N)-1),
report N/max and each trial separately. Do not interpolate fabricated precision
or pool trials to hide a failed run.

**MET-06.** CPU/GPU resource ratios versus baseline report median/range and a
paired confidence interval with the raw per-trial pairs. Three repeats alone
cannot establish a narrow 95% confidence interval: label uncertain comparisons
inconclusive and use a predeclared additional repeat count before claiming a win.
Any repeatable >5% regression requires explanation and review even below ceilings.
Tracing on/off comparisons require matched schedules **and delivered output**;
no universal overhead subtraction. Resource acceptance uses diagnostics off;
structural counters use separate matched diagnostics-on trials.

## Upload, allocation and endurance requirements

**MEM-01.** Warm unchanged authored geometry: zero rebuilds, zero upload bytes,
zero new world-buffer allocations. Count every queue write/copy and mapped upload
by consumer/resource generation; report bytes requested and actual transfer
alignment separately. Screen/control/text uploads are only changed ranges; compare
with exact dirty-range bytes plus documented alignment. Capacity reuse is not
zero upload. Device reset/cold load is separately counted.

**MEM-02.** Proposed T1 memory limits below are admission ceilings, not measured
payload sizes. The existing uninstrumented pan report samples approximately
156 MiB GUI RSS; a 256 MiB steady single-host ceiling provides explicit headroom
without normalizing unconstrained cache growth. Every cache cap is a maximum,
not a preallocation target. T2 needs separate numbers after fixture admission.

| Resource / ownership boundary | Proposed T1 bound | Measurement and release |
|---|---|---|
| GUI RSS, main host | 256 MiB steady, 384 MiB peak | OS high-water plus sampled RSS; distinguish allocator/driver mappings from live payload |
| Additional native host | ≤64 MiB steady incremental RSS per host; ≤512 MiB combined GUI peak | Measure each real host and all permitted hosts together; close returns within 16 MiB of warmed prior RSS within 5 s, with live allocations separately zero |
| Cooperating engine process | 256 MiB steady, 384 MiB peak for frozen T1 model, excluding separately reported cold import | Track complete PID lifetime; do not move GUI work off the accounting boundary |
| Retained CPU world geometry/history | 64 MiB total per loaded T1 document; six historical entries maximum plus live references within that total | Allocation/lifetime counters; evict unpinned history before insertion, no loss of authoritative model |
| Shaped text payload | 8 MiB per renderer, 32 MiB process total | Include key strings, glyph/layout vectors and owned capacities; existing entry/key caps also apply |
| Width measurement cache | 256 entries / 64 KiB keys / 128 KiB total owned payload per thread; enumerate all owning threads | Existing LRU and oversize bypass; total process ownership explicitly summed |
| Control meshes | 256 entries / 4 MiB CPU / 4 MiB GPU per renderer | CTL-02; pinned frame references remain in total live usage |
| GPU world geometry and encoded-resource retention | 64 MiB per document plus 16 MiB host-local screen buffers | Count allocated buffer capacities and shared resources once; cold replacement bounded to one old plus one new live generation |
| Glyph atlas | 32 MiB per renderer, 128 MiB aggregate | Actual texture format/mips/extent and allocation generation; eviction invalidates prepared glyph references |
| Terminal graphics | 64 MiB retained CPU decoded pixels and 64 MiB GPU total | Preserve stricter existing terminal protocol admission limits; oversize input follows established refusal semantics |
| Upload staging / scratch | 16 MiB per host, 64 MiB aggregate | One outstanding frame per host; retire after completion; oversized cold work must use bounded chunks |
| All application-owned GPU allocations | 512 MiB peak for T1, including attachments, buffers, atlases and staging | Shared allocations counted once; driver swapchain/residency reported separately, never assumed included |

Surface attachment bytes use actual physical width×height×format bytes×sample
count, multiplied by all simultaneously retained generations; one current plus
one retiring generation per host maximum. At 1536x960, 8x RGBA8 is 45 MiB per
attachment; do not preallocate that maximum for smaller windows. Record actual
swapchain image count/driver residency where available. API allocation arithmetic
is not physical driver-residency measurement. No changed-size generation backlog
or new unchanged-size attachment allocation is permitted. GPU caps and all
subcaps apply together: satisfying individual caps cannot exceed the aggregate.
If admitted required content cannot fit, record a failing tier and revise the
proposal through owner review; never silently omit content, reduce AA or evict
model/terminal authority to satisfy a renderer budget.

Lifecycle recovery target: one transient replacement generation, peak≤the above
combined peak caps, ≤100 ms application CPU per successful recovery, p95 visible
recovery≤250 ms after resources become available, maximum≤2 s active time before
controlled failure. No new GPU work while zero/suspended/known occluded, and no
more than one acquisition attempt at each REC backoff opportunity. Allocation
faults and a hung driver are reported as failures rather than folded into the
successful-recovery distribution. A blocked driver call exceeding these limits
fails qualification even when the application attempted to obey the schedule.

**MEM-03.** Final acceptance: 60 minutes per admitted representative/demanding tier,
200 open/close cycles and 20 injected recoveries. After five-minute warmup sample
live bytes/capacity every five minutes; first/last ten-minute CPU/latency windows
must not regress >5%. Closed consumer live resources reach zero after the bounded
in-flight drain or device teardown; shared caches return to declared caps. No
monotonic live-resource trend is accepted. Report allocator/driver plateaus and
peak RSS separately; high flat usage is not automatically efficient. Return to
idle within 250 ms after final required visible state, excluding explicitly
recorded legitimate terminal timers. Runtime endurance follows adoption, not C02.

## Temporal oracle validation boundary

**VIS-01.** Required observation is continuous displayed **interior content**,
not just borders or final screenshots. Correlate accepted input, extent/DPI,
frame generation, GPU completion and independently captured/observed presentation
using one monotonic timeline with measured clock error. Establish capture coverage
of every relevant display opportunity. Record dropped frames, duplicate timestamps,
fixed-size crop/scaling, recorder overhead and the actual native backend.

**VIS-02.** Before relying on the oracle, reject isolated negative controls for:
one blank intermediate frame; disappearance of board/text/control content;
one stale extent/DPI frame; geometry jumping independently of intended layout;
held frames exceeding the latency limit; discarded final input/damage. Include
legitimate newly exposed background and normal layout movement as positive
controls so the detector does not fail correct resizing. Offline mutation can
test detector sensitivity, but cannot establish live capture cadence or missed
frames. A nominal 60 fps video header is not cadence proof.

**VIS-03.** Existing `/tmp/datum-resize-video-manual-v2/window.webm` is explicitly
feasibility-only: 154 decoded frames over 46.044 seconds, variable damage-driven
capture, fixed 1146x729 extent, duplicate timestamps and long gaps; compilation
overlapped part of capture. Its report already rejects temporal and resource
acceptance. It cannot be relabeled as a validated no-blink oracle. A complete
method requires bounded calibration on the actual capture/display route or a
validated independent display observer. No additional kernel tracing is required
for that calibration. Kernel attribution and temporal output are different
questions.

## Open C03 completion items

| ID | Required resolution | Blocks |
|---|---|---|
| C03-G01 | Admit a demanding real project with complete counts or record explicit owner scope limit; separately admit resolved schematic content | R12, tier numerical rationale |
| C03-G02 | Validate temporal capture boundaries/cadence/negative controls, or obtain explicit qualification limit under R14 | R14/R17/R41; no flicker acceptance |
| C03-G03 | Validate GPU execution and lifetime-complete accounting methods or approved alternate/scope | R14/R16 |
| C03-G04 | Review MEM-02 proposed T1 caps and reconcile demanding-tier caps after admission; implementation supplies allocation/lifetime receipts | R18/R40 |
| C03-G05 | Reconcile proposed targets and statistical rules through independent review, preserving all 25 HP cases | R19/R38 and later C05/C06 |

These are engineering/proof gaps, not a demand for completed production passes
before implementation. GPS-C03 is in progress; this document does not close it,
advance GPS-C04, approve the architecture or accept the unresolved resize defect.

## Bounded method-calibration proposal for C03-G02/G03

This is the concrete measurement task awaiting owner scope direction, not a new
approval framework. Use existing native capture, wgpu diagnostics and OS tools.
No root tracing, package/dependency change, driver tuning or product renderer
optimization is part of it. Preserve ordinary production flags/defaults.

1. Pin the owned diagnostic target and recorder source, actual backend, adapter,
   output refresh and full physical display extent. Prefer whole-output capture
   so window resize cannot be hidden by fixed-window cropping/rescaling.
2. Define an opt-in diagnostic sequence with frame/generation identifiers and
   known transitions. Check one correct sequence and one negative sequence that
   deliberately includes blank, stale-extent and held/final-state failures. The
   sequence is a capture calibration, not a fabricated EDA performance fixture.
3. Permit one 15-second positive and one 15-second negative capture per backend,
   with maximum 30 minutes total active preparation/analysis. Bound recorder
   startup/teardown separately; a time limit must cover setup as well as capture.
   No repeated parameter sweeps. If selection, timestamps, native extent or
   cadence cannot be established, preserve the failed receipt and stop.
4. Decode actual timestamps and frame identifiers; reconcile every intended
   display opportunity, report missing coverage and test single-frame negative
   sensitivity. Measure capture CPU/GPU overhead against the same sequence with
   recording off. An offline synthetic-negative pass alone cannot qualify live
   capture. Unknown scanout timing remains an uncertainty, not zero latency.
5. Where existing timestamp-query support permits, validate GPU observation
   boundaries with an empty versus known-work command sequence and asynchronous
   result collection; record queue/host identity and clock resolution. If support
   or calibration fails, retain C03-G03 and specify the exact unsupported metric.
6. End with a reviewed method receipt stating supported backend, uncertainty,
   detection limits and unresolved gaps. Passing calibration authorizes no
   product performance claim; failing it starts no automatic successor experiment.

The 30-minute bound limits investigation, not the evidence standard. Failure
requires a concrete method/scope decision; it cannot be converted into acceptance
or an assumed waiver of R14/R41. Full product proof remains with implementation.
