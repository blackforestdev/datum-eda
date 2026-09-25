# GUI performance acceptance matrix

Status: GPS-C03 reviewable acceptance specification; final independent packet
review and owner ratification remain pending. No runtime qualification claimed.
Parent: `GUI_PERFORMANCE_RECOVERY_PLAN.md`, R11–R20/R38/R40–R41.
Engineering definitions: `GUI_SHARED_ENGINEERING_CONTRACT.md`, E01–E12.
The numerical targets below are proposals for review, not measured achievements.

## Owner-approved initial qualification limits

The exact owner responses are recorded in
`docs/reviews/gui-performance/initial-qualification-owner-direction.json`.
Under R12/R14, initial qualification is limited to the pinned F-DOA small-board
workloads and declared T0/T1 configurations below. It does not establish capacity
for arbitrary projects below a footprint count, larger projects, or unadmitted
schematic content. The 39-package fixture is an evidence boundary, not a claim
that every 39-package design has equivalent cost.

Initial qualification explicitly excludes resize flicker/blink freedom and
observed-display latency, frame age and pacing acceptance. VIS-01–03 and the
corresponding MET-05 display thresholds remain specified **unqualified future
requirements**, not initial pass criteria or zero-valued results. Static visual
parity, clipping/hits, input/focus, final state, CPU/GPU/memory and work-count
proof remain required. Application timestamps cannot substitute for excluded
display observations. Any report must print these scope exclusions alongside
results; it cannot claim complete visual responsiveness or enterprise capacity.

All R01–R42 and HP01–HP25 remain mapped for later implementation/final acceptance.
The resize QA issue stays open until both resource and temporal criteria pass.
These answers do not approve numerical targets, waive GPU accounting, authorize
a renderer change or constitute GPS-C06 approval. The capture-calibration
proposal is not selected; no calibration run follows from this amendment.

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
| T2 | Demanding real resolved board and schematic, with pinned source/model hashes and measured geometry/text counts; four leaves plus permitted auxiliary hosts | Outside owner-approved initial scope; larger-project qualification remains unaccepted and requires later fixture admission and proof |

The owner has selected limited initial scope under R12/R14. T2 still requires
an actual qualifying project and separate qualification before any larger-project claim. Repeating a small project across panes exercises multi-pane cost,
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

Initial W-RESIZE runs T0 minimal, shell and T1, plus each native dialog; T2 is
reserved for later admitted demanding-project qualification. Include
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
unchanged. T2 numbers are outside initial scope and cannot be ratified before
fixture admission and rationale.

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
do not establish a formal comparative claim; STAT-01 prescribes a fixed seven-pair
comparison and inconclusive disposition rather than adaptive significance testing.
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

Private text-library construction exception: allocations performed internally by the existing font selection/loading/cache and glyph rasterization calls may use monitored construction rather than pre-call capacity admission. This includes raster output until ownership transfers to Datum. It does not cover Datum-controlled allocations, permit new dependencies, or change any numerical local, aggregate, RSS or GPU bound. Full instantaneous live and peak accounting, including allocation overhead and concurrent host incidence, remains required. All applicable subcaps apply together. A construction-time overrun is a recorded tier failure, not successful admission or a passing memory result. This exception does not guarantee preventing a transient overrun or recovering from process-level allocation failure inside an opaque call.

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

## C03 specification dispositions and runtime prerequisites

| ID | Specification disposition | Required before affected runtime claim |
|---|---|---|
| C03-G01 | Owner limits scope to pinned T0/T1; historical counts are in ADM-01 and the receipt; complete count schema and source boundaries specified | Production admission counts including shaped/visible fields; separately admit schematic content before schematic claims; T2 remains unaccepted |
| C03-G02 | Explicit owner exclusion of temporal qualification; VIS methods retained for later acceptance | Validated temporal method and proof before flicker/display-latency acceptance or resize closure; no calibration selected now |
| C03-G03 | GPU-01–03 and ACC-01–03 provide source/capability-validated feasible methods, bounded instrumentation and exact failure rules | Enabled-device feature check, query conformance, overhead and lifetime-complete runtime receipts; unsupported/missing results cannot pass |
| C03-G04 | MET/MEM define proposed T1 absolute duty, execution, latency and memory limits with accounting boundaries and admission rationale | C05 review/C06 numerical ratification, then actual per-workload runtime compliance; demanding tier remains outside initial scope |
| C03-G05 | STAT-01 defines fixed-count inference, uncertainty, invalid trials and zero-baseline behavior; all HP obligations retained | Independent complete-packet review and owner specification-only decision; no achieved-budget claim from draft values |

This is specification completion, not completion of implementation, instrumentation
conformance or runtime performance. No missing field has been converted into a
measurement. Initial approval cannot close the resize defect or authorize broader
implementation. C04 must map every method, deferred case and runtime prerequisite.

## Bounded method-calibration proposal for C03-G02/G03

Status: preserved future proposal, not selected under the owner-approved initial
qualification limit. It supplies no current calibration or acceptance evidence.
GPU-method requirements in C03-G03 remain separate and are not waived by the
visual limit. If this proposal is later selected, use existing native capture,
wgpu diagnostics and OS tools.
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

## Specification-level method validation and implementation conformance

<!-- EVIDENCE:GUI-PERFORMANCE-SPEC:GPS-C03-METHODS -->

**ADM-01.** `docs/reviews/gui-performance/measurements/c03-methods/baseline-counts.json`
recounts all 2,061 records in the pinned historical candidate log: retained world
vertices are 138,444 throughout; panel vertices 906–966; underlay 78–432;
reported overlay zero; reported text runs 142–144. The current repr(C) Vertex
contains five f32 values, so base world vertex payload is 2,768,880 bytes.
This excludes stroke data, capacities, GPU copies and all other allocations.
The record identifies its exact input hash and extraction. These are measured
historical count ranges, not maximum capacities or counts for every T1 state.

The scale boundary is the exact admitted fixture and configurations, not an
unmeasured glyph ceiling. Initial measurement admission must output, per host,
pane, model generation and prepared frame: retained vertices/stroke instances;
submitted commands/ranges and primitive counts; primitives surviving CPU culling;
clipped raster coverage when that is the metric; text runs, shaped glyph instances
and unique `(font generation, glyph id, size/raster mode)` keys. Never label a
CPU-visible primitive as a raster-visible pixel. Use `PreparedScene`, retained
scene builders, surface draw commands and shaped buffer `layout_runs` at their
existing production preparation boundaries; no alternate scene implementation.
Record before/after layout, font, layer, pane and DPI changes. Missing counts
invalidate runtime admission, not become zero. No glyph/visible-count observation
is fabricated in the present baseline receipt. Counting instrumentation and conformance are delivered with the production
components identified in the implementation contract's S0–S4 allocation.
Counts needed for a bounded patch's proof are due with that patch; full counts
remain mandatory before runtime tier claims and S5 completion.
This separates a completely specified method from the results it must produce.

**GPU-01.** The archived identified Intel P630 capability report in
`measurements/c03-methods/vulkan-capabilities.txt` reports graphics timestamps,
83.3333 ns/tick and 36 valid bits. It supports feasibility on the reference Vulkan
adapter; it does not prove timestamps enabled on Datum's wgpu device. Two fresh
bounded read-only summary attempts failed surface initialization and are retained
in `capability-disposition.json`; neither is a successful current native test.

Pinned wgpu 28.0.0 API source validates the proposed mechanism:
`RenderPassDescriptor::timestamp_writes` requires `Features::TIMESTAMP_QUERY`;
`CommandEncoder::resolve_query_set` writes eight bytes/query with aligned resolve
offsets; `Queue::get_timestamp_period` supplies nanoseconds/tick (zero unsupported).
`Queue::on_submitted_work_done` callbacks need submit/poll progress and must be
short. References are the installed crate's `api/render_pass.rs`,
`api/command_encoder.rs`, `api/queue.rs`, and
https://docs.rs/wgpu/28.0.0/wgpu/struct.Queue.html#method.get_timestamp_period.
No new dependency or arbitrary inside-pass/inside-encoder timestamp feature is
needed for pass-boundary timing. Production passes currently use None; results
are therefore absent today, not zero GPU time.

**GPU-02.** The future opt-in measurement adapter checks adapter support, explicitly
requests TIMESTAMP_QUERY at diagnostic-device creation, verifies enabled device
features and a finite positive period, and records adapter/backend/device epoch.
Allocate at most three query/readback slots per host, each holding 32 u64 values
(16 pass pairs), plus aligned resolve and map-readable copy buffers. Four native
hosts means at most 12 slots and 384 query values. Logical query data is 3,072
bytes; each resolve/copy allocation and opaque driver overhead is separately
accounted. Do not claim that query-data size equals total residency.

Assign host/frame/device epoch, submission IDs and pass names before encoding.
Write start/end in each contributing pass descriptor; resolve/copy after writes.
Map asynchronously after submission, harvest with nonblocking polling and a
bounded two-second active-time failure deadline. Do not reuse a slot before
unmap/completion. On ring exhaustion record missing data and fail that timed
trial; never stall the UI or silently discard a slow sample. Device reset cancels
old slots and invalidates their observations. Missing/zero/invalid periods,
validation errors, unsupported features or missing pass pairs make the metric
unavailable and block the affected runtime result.

Use u64 tick differences within one device epoch and convert with the measured
wgpu period. The historical 36-bit Vulkan counter wraps in about 5,726 seconds;
keep sample spans below two seconds and reject reversed/unexplained values rather
than guessing a backend mask through portable wgpu. A wrap sample can be rejected
and retried within the declared trial cap; it is retained as invalid evidence.
Never combine epochs or use GPU ticks as a CPU/input/display clock.
Report own-pass sum and first-start/last-end frame span separately as MET-04
requires. Pass-boundary timing excludes preceding implicit upload transfers;
report that limitation and use transfer byte counts plus complete DRM duty for
those transfers. No claim of total GPU execution follows from pass time alone.

**GPU-03.** Before a timestamp-based runtime pass, instrumentation conformance must
exercise a real empty pass, a real drawing pass, multiple ordered passes, missing
query, slot exhaustion, delayed map, device epoch change and simulated wrap.
Verify units/order/association against raw query values, and separately measure
query/readback overhead on matched diagnostics-off output. Empty work need not
measure exactly zero and more work need not always run slower; those assumptions
are not calibration oracles. Missing/invalid samples must fail the report. These
are implementation tests of this source-validated method, not completed trials.
A device lacking the method stays unqualified until a validated alternate or
explicit owner scope change exists; temporal exclusions do not waive this rule.

**ACC-01.** Resource accounting uses stable allocation IDs plus owner and device
or document generation. Instrument existing buffer/texture creation and replacement
in `gpu_init`, `gpu_data`, `gpu_vertex_upload`, `gpu_strokes`, `gpu_surface`,
`terminal_graphics`, and text/atlas owners. Every allocation has requested payload,
allocated capacity, reference owners, live/retiring state and release reason.
Shared references count bytes once; host attribution is a separate incidence
relation. Releasing the last CPU handle retires an allocation only after GPU use
completes or documented device teardown. Report API-live allocated bytes, driver
resident counters and RSS separately; none is a substitute for the others.
Count queue write_buffer/write_texture, staging writes and encoder copies at the
actual boundary, including glyph/terminal uploads and alignment. Distinguish an
upload scheduled by queue.write_buffer from its later execution on submit.

**ACC-02.** Count CPU cache keys and payload capacities, including strings,
shaped glyph/layout vectors and transient scratch. Counter conformance creates,
shares, replaces with equal-size different data, evicts, closes and recreates a
resource; known-byte totals must reconcile and never go negative. Zero unchanged
upload and allocation assertions use explicit production counters plus a negative
control that deliberately repeats the upload/allocation. RSS alone cannot pass
those assertions. No full-system allocator hook is required to count owned caches.

For the MEM-02 private text construction exception, a shared Datum call guard records initial/live/peak/final owned bytes and exact concurrent host/process incidence over the entire call. It must distinguish retained input/output from transient scratch without double counting shared allocations. After the call, before publishing newly prepared resources or submitting their GPU work, a detected local or aggregate overrun fails preparation, releases newly derived work as appropriate, and preserves authoritative text and pending damage for explicit retry. Do not continue a rejected batch, silently omit content, turn allocation failure into empty output, or report an overrun as compliant. Positive and deliberately over-budget negative controls must verify the accounting and failure path. A before/after live-byte snapshot or lifetime high-water alone is insufficient to recover an individual call peak.

**ACC-03.** OS observation keys processes by PID plus start-time and DRM clients
by device identity plus client ID plus lifecycle epoch. Shared/duplicated fds are
deduplicated. Start the process observer before spawning GUI/engine children;
combine live counters and child exit usage exactly once. At controlled shutdown,
stop adding workload actions but keep the accounting window open, complete queued
work through the bounded normal drain, collect final CPU and client counters while
fds remain live, then close the common elapsed-time window. Numerators and
denominators include that same tail. Report action-phase and drain duration/cost
separately, without charging post-window work to a shorter denominator. Clients
closed earlier retain their final counters within the same overall window. If an
external crash or client exit removes the final reading, flag the GPU total as
incomplete; do not extrapolate it from surviving clients. Keep client start/end
receipts and epoch/reset checks. Sources are the existing `/proc` harness plus
explicit spawn/teardown lifecycle hooks in the future measurement slice.
This closes the specification's lost-client ambiguity; it does not rehabilitate
old window-cycle GPU reports.

**STAT-01.** Absolute limits apply to every valid trial, not just its median.
An acknowledged action is one accepted semantic state transition; a pointer
sample that is legally coalesced retains its scheduled/received/final-state
counts and is not invented as an extra completed action. Report total CPU per
scheduled input and per acknowledged action separately. No-op and idle workloads
use scheduled-input and elapsed-time denominators; there is no division by zero
or fabricated completed-action count. Work correctness is evaluated independently.

For relative comparisons predeclare seven baseline/candidate pairs in alternating
AB/BA order, identical fixture/state/input schedule and comparable completed
application work. The first three pairs may be reported descriptively but cannot
stop early with a formal confidence claim. All formal relative claims use the
fixed seven pairs: for positive costs calculate each log ratio `log(C/B)`, its
mean and sample standard deviation, and the two-sided 95% Student-t interval
`exp(mean ± 2.447 * sd/sqrt(7))`. This assumes independent approximately normal
log ratios; report that assumption and inspect paired residuals. If it is not
credible, the interval is descriptive and no formal claim is made. Compare the
interval with 1.00 for improvement and 1.05 for regression. Stop after seven
valid pairs; uncertainty is inconclusive, not permission to select a favorable
subset or keep sampling. At most ten attempted pairs may supply seven valid
pairs; retain failed/contaminated attempts with reasons fixed before inspecting
outcomes. Absolute-limit qualification retains MET-01's three trials and cannot
be replaced by this relative estimator. No repeated experiment tree follows an
inconclusive result.

A zero or below-resolution baseline **or candidate** disables log ratios. Never
substitute an epsilon to manufacture a finite ratio. Report absolute differences
with the observer's
resolution bound (including both endpoint errors) and the absolute acceptance
limit; values below resolution are '<resolution', not proof of zero work.
A comparative resource-only result uses matched application state and completed
work and explicitly retains the owner's display-qualification exclusion; it is
not a claim that displayed throughput was equal. Static pixel parity and final
state cannot fill that excluded observation. Complete adoption/endurance remains
future runtime evidence.

## Owner amendment: resize investigation is nonblocking

The subsequent owner instruction in
`docs/reviews/gui-performance/resize-deferral-owner-direction.json` pins
`dat-gui-vertical-resize-cpu-toj` as unresolved investigation and removes resize
resolution, kernel/driver attribution and completed temporal calibration as
development gates for S0–S5. This amendment supersedes earlier E10 pre-change
calibration requirements and GPS-C01R blocking sequence. It does not mark resize
budgets, flicker or displayed-frame criteria passed or change their numerical
limits. The Linux-kernel/Intel-driver/hardware explanation is an unconfirmed
owner hypothesis, not an established root cause.

Shared application lifecycle and reuse fixes may proceed under the separate
execution grant, with bounded relevant production regressions, available
before/after resource observations and final-state/extent/hit/focus/PTY/static
correctness checks. Missing temporal proof is reported as unqualified. It cannot
be used to claim resize closure, but it does not stop other development.
All non-resize performance/correctness, adoption, independent replay and
endurance requirements remain. S5 can conclude only its explicitly limited
non-resize qualification; full resize resource/temporal closure stays on the
pinned issue. No new kernel investigation or renderer execution is authorized
by this amendment.
