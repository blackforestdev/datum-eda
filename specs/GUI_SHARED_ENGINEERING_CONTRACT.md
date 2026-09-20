# Shared GUI engineering contract

Status: proposed engineering specification for GUI-PERFORMANCE-SPEC. This is
the concrete contract requested by the owner's sequencing correction, not
ratification of a new mechanism or a declaration of production acceptance.
The parent is `GUI_PERFORMANCE_RECOVERY_PLAN.md`; its R01–R42 and HP01–HP25
remain obligations. Specification review approves definitions and feasible
proof. Implementation supplies the results, slice by slice. Complete adoption
and endurance belong to final acceptance.

## Scope and existing authority

Improve Datum's current Rust/wgpu renderer and shared viewport services.
Decision 023 controls the retained authored world, screen-space interaction,
per-pane camera identity and pointer/focus routing. The Rendering Book controls
material appearance, painter order, selection and typography. Terminal state,
row scrolling, focus, input and PTY operation retain their own authority.
Neither a class hierarchy, renderer replacement, new dependency nor separate
render thread is required by this contract.

Performance means correct, responsive, visually faithful output with the least
demonstrated necessary work. A low utilization result with missing input or
stale pixels fails. A timing pass does not justify avoidable preparation,
uploads, frames or allocations.

## E01 — Redraw ownership and state transitions

The application owns native-window routing; each native surface owns its damage,
pending redraw and presentation resources. A pane supplies damage to its host,
not to unrelated windows. Runtime view state and engine document state remain
separate. Existing `App`, `Runtime`, `GlobalPreferencesWindowSurface`,
`Renderer`, `EditorViewport` and `ScrollViewport` are the adoption points.

An event handler first applies input to authoritative view state, then reports
the affected consumer and changed dependencies. Scheduling must not decide
whether input is applied. Coalesce redraw requests, not wheel deltas, terminal
bytes, typed operations, button transitions or final pointer/camera state.
Pointer motion may retain the latest position only when intermediate motion
does not carry gesture semantics. Selection/authoring gestures keep their
required path processing.

For each window: clean -> damaged -> redraw pending -> rendering -> clean.
Damage arriving during a frame survives that frame's completion and schedules
one successor. A successful frame clears only the damage generation it consumed.
Failed acquisition retains damage. Closure removes pending window work and
releases owned resources; late callbacks cannot resurrect the window. Use one
pending redraw token per window; do not add an unbounded queue of frame snapshots.
The event loop sleeps when there is neither damage nor a required timer/task.

The target shared interface expresses `invalidate(window, consumer, domain)`
and `request_if_needed(window)`; spelling and representation are implementation
choices. Domains distinguish interaction, camera, layout, content, style/text,
and surface resources. A global setting targets its actual dependent windows.
Application-wide redraw is reserved for an identified application-wide change.
Focus redirection for input-modal Preferences remains intact: benchmarking must
not bypass it to manufacture a simultaneous editor gesture.

## E02 — Invalidation dependencies

| Change | Required work | Work forbidden when dependencies are unchanged |
|---|---|---|
| Pointer enters/moves/leaves a drawing pane | Shared hit/snap query; changed cursor/hover/preview overlay and readout | Shell solve, authored scene resolve/tessellation/upload, unrelated window preparation |
| Pointer over unchanged noninteractive chrome | Cursor transition only, if any | Content frame, layout solve, text shaping |
| Camera pan/zoom/fit changes state | Target pane transform, camera-dependent grid/LOD/overlay, projected hit coordinates and text placement | Authored-world rebuild/upload; unrelated pane camera reset; unchanged shell/control shaping |
| Wheel camera input leaves state identical, including zoom clamp, with no other UI change | Consume input and preserve route/state | Frame invalidation, encoding, submission |
| Continuous scroll changes offset | Owning content transform/visible interval, clip, thumb and hit regions | Unrelated host redraw, hidden world composition, unchanged label shaping |
| Boundary/horizontal-only scroll has no supported effect | Consume according to the control's routing policy | Frame request or accumulated boundary debt |
| Terminal output/selection/row scroll | Terminal model update; visible terminal rows/selection damage | Hidden terminal render snapshot; unrelated window redraw; reinterpretation as continuous document scrolling |
| Size/DPI/dock/pane geometry changes | Affected layout, clip, hit geometry, projections and size-dependent attachments | Authored model mutation; stale cached layout keyed to previous geometry |
| Design revision/layer/style/font/locale changes | Invalidate each actual dependent resource; changed label/layout keys propagate | Keeping a cache valid by pointer/length coincidence; broad invalidation without a dependency reason |
| Open/close/switch pane or window | Initialize/rebind identity, focus/capture, visible data and owned resources | Resetting surviving warm cameras; retaining closed consumer resources indefinitely |

No-op is exact state equivalence after applying the established operation, not
an epsilon dead zone that drops small valid input. Floating input validation
must reject non-finite values; no rounding-based resource optimization is allowed.
An overlay may require a frame even when the selected object has not changed,
for example a moving crosshair. Such frames are legitimate and must be measured.
Likewise a focused menu/keyboard zoom command can change menu or console feedback
even at the camera limit. Preserve that distinct UI damage; identical camera
state suppresses camera invalidation, not the command's other visible effects.

## E03 — Geometry, encoding and upload reuse

Retain authored board/schematic geometry by immutable ownership plus its actual
content/style dependencies. A live retained owner or explicit generation is
required for identity: address and element count alone are insufficient.
Equal-length replacement and allocator reuse must change identity. Buffer
capacity reuse is allocation reuse, not evidence that bytes were not uploaded.

Separate retained world, shell/control geometry, text, camera-dependent geometry
and interaction data. Upload only changed resources; unchanged immutable world
upload bytes are exactly zero on warm camera and pointer frames. Eligible draw
encoding survives camera-only uniform changes. Device recreation invalidates
GPU identities and encoding, even if CPU content identity is unchanged.

Encoding keys include device/resource generation, pipelines, formats/sample
counts, bind groups, draw order/ranges and any baked clipping dependency.
Batch adjacent compatible commands only; no sorting across painter/layer order.
Triangle boundaries and blend/clip semantics must survive batching. A dialog-only
frame must not prepare an invisible board/terminal/shell backdrop; use the
existing Preferences dialog-only path as the starting point for New Project.
The established one-pass dialog schedule is preserved unless a measured visual
requirement needs another pass. Count passes and resolves separately.

## E04 — Text and cache lifetime

Text shaping keys include exact text, font identity/revision, size, style,
language/direction and applicable shaping features. Layout adds wrap width,
line metrics and alignment. Placement and clip changes do not by themselves
invalidate unchanged shaping. Width measurement and rendered layout must use
the same font and size authority. GPU glyph preparation additionally depends on
atlas generation, placement, clip and current target characteristics.

Cache insertion/eviction/reordering must invalidate any signature referencing
removed or changed glyph resources. Dialog/workspace transitions must never
leave a valid-looking stale glyph signature. Preserve the terminal's bounded
two-generation churn behavior unless an equivalent replacement is qualified.

Each cache declares an owner, full key, entry/byte cap, eviction rule and teardown:

| Resource | Owner/lifetime | Required bound and release proof |
|---|---|---|
| Compiled static menu inventory | Process | One parse per immutable inventory; mutable availability remains separate |
| Shell/hit layout | Existing shared layout service | Existing two-entry working set; correct misses on every geometry-key change |
| World CPU geometry | Document/surface generation | One live version per referenced generation; current session-history cache has six entries plus active references; old ownership drops when frames/caches release it |
| GPU world buffers/encoded draws | Renderer/device generation | Report live bytes and retained capacity separately; replacement and device reset release obsolete resources |
| Dialog shaped labels | Renderer | Existing post-frame bound: 128 buffers and 32 KiB key text; separately measure shaped payload, atlas and current-frame scratch |
| Label widths | Existing thread-local text measurement owner | Existing bound: 256 measurements and 64 KiB key text; exact font/size/text keys, least-recently-used eviction; overlarge keys bypass retention |
| Dialog layout/control data | Owning window/content generation | Bounded by admitted content and declared working set; release on close or bounded shared-cache eviction |
| Surface attachments | Window/device | Current size/sample configuration only, plus actual in-flight resources; release after completion/close |

Do not introduce a new cache until its actual byte/entry cap and eviction test
are supplied in its slice proof. A larger cache is not the default optimization.
Allocation pressure may evict reusable data; it must not evict authoritative
design or terminal state. Any expensive reconstruction after eviction is cold
work, explicitly separated from warm measurements.
The existing limits above come from `text_buffer_cache.rs`, `text_metrics.rs`,
`interaction_refresh.rs` and the layout cache. Key-text caps do not bound complete
shaped/GPU payloads. Complete payload and scale-tier budgets remain an explicit
approval gap; these observations must not be relabeled as complete memory proof.

## E05 — Scrolling, clipping and hit testing

Continuous surfaces use the existing `ScrollViewport` as their offset/extent/
thumb/capture owner. Offset is finite and clamped to
`[0, max(content_extent - viewport_extent, 0)]`. Every finite fractional delta
contributes until clamped. Physical pixel deltas convert once into the chosen
coordinate system; line steps convert once using scale. Reversing at a boundary
responds immediately, with no accumulated offscreen residual. Unsupported
horizontal movement must not fall through into another control's vertical scroll.

Terminal screen/history and row-based Layers keep their meaningful discrete row
authority. An adapter can share input conversion, capture cancellation, clipping
and geometry utilities without changing row-index, selection or PTY semantics.
Do not round Preferences offsets to satisfy terminal conventions or make terminal
screen contents follow a floating document-scroll offset.

One layout result supplies content extent, viewport, thumb, paint transform,
hit transform and clipping. Intersect each descendant clip with its ancestor
viewport. Test inverse-transformed pointer coordinates against the same visible
region before dispatch. Hidden/fully clipped controls cannot activate; partial
controls activate only through their visible region. Pinned search/chrome and
scrollbars use separate non-scrolling regions. Clip geometry without distorting
rounded/diagonal edges; clamping vertices is not clipping.

Thumb drag preserves the initial grab displacement. Track clicks page within
bounds. Release, focus loss, close and capture loss end dragging. Content,
section, search, expansion, size and DPI changes recompute extent and clamp the
offset before paint/hit dispatch. Keyboard reveal scrolls the focused row into
view without stealing focus or generating an idle animation loop. Honor reduced
motion and the existing Preferences settings/mutation semantics.

## E06 — Surface lifecycle and fairness

| Event | Required transition | Proof and resource expectation |
|---|---|---|
| Zero size/minimize/known occlusion | Suspend acquisitions for an undrawable surface; retain logical damage | No render loop or zero-size configure; restore presents current state |
| Resize/DPI/display move | Reconcile geometry and attachments before the next frame | Correct physical stroke/text/hit geometry; no retained old-size attachment growth |
| Lost/outdated surface | Mark reconfiguration needed, retain damage, retry through scheduling | Inject error then success; no tight retry loop or lost final state |
| Timeout/repeated timeout | Retain damage; bounded retry deadline, not immediate spin | Proposed retry 16/32/64/128/250 ms capped; stop automatic retries after 2 s and expose recoverable failure |
| Device/allocation failure | Stop submissions, preserve engine/terminal data, report failure; recreate only through a bounded recovery path | At most one automatic recreation per failure episode; failed recreation ends in controlled close/retry action |
| Suspend/resume | Suspend device/surface work; recreate invalid resources on resume | No background render loop; current input/model state restored |
| Close with pending frame | Cancel consumer scheduling, release resources after in-flight completion | Late completion ignored; no callback to destroyed window |

These retry numbers are proposed review values, not existing implementation
claims. Platform signals absent on a backend require explicit applicability
records. Fault injection proves application transitions, not driver reliability.
Window activity must not starve another window's input. Bound work per dispatch;
terminal processing may yield/requeue while retaining bytes. Do not sleep inside
an event callback or synchronously wait for a GPU readback in ordinary rendering.

## E07 — Measurement and slice acceptance

Baseline and candidate use the same optimized build configuration, fixture hash,
window geometry, adapter/backend, input schedule and visible result. Use existing
timing logs, OS CPU/RSS/DRM counters and native input tools first. Measurement
failures remain in the evidence: an input-modal window or a pointer outside its
target invalidates the intended workload, even if CPU use looks excellent.
Pin resolved object identity and painter ordering too. Imported file hashes can
match while path-derived IDs change ordering; use one frozen native model or
the same controlled import identity for baseline and candidate. Record any
non-rendered metadata excluded from model or pixel comparison explicitly.

Report total GUI CPU seconds divided by elapsed seconds (one core = 100%), CPU
seconds per accepted action, and cooperating engine CPU separately. DRM active
nanoseconds divided by elapsed nanoseconds measures engine duty, not frequency-
normalized utilization, energy or per-frame GPU time. Trace submissions are not
delivered frames. Log acquisition wait separately from preparation and encoding.

Use five-second warmup and three 30-second trials for qualification; shorter
10-second investigative runs can choose the next diagnostic but cannot qualify
the full product. Retain every repeat and report median and range. Trace-on/off
paired trials quantify instrumentation overhead. A workload without verified
input and expected output is a failed measurement, not a performance pass.

Proposed 60 Hz review rules remain R19's p95 <=4 ms and p99 <=8 ms CPU
event/preparation/encoding, >=95% presentation intervals <=20 ms, no unexplained
warm application stall >50 ms, and <=1% one-core CPU over 60 s quiet idle with
zero application-generated submissions. Proposed input-to-display limits are
p95 <=33.4 ms and p99 <=50 ms for warm pointer/camera/scroll actions; opening,
closing and pane switching require their own cold/warm distributions and a
proposed warm p95 <=100 ms. Native presentation/latency methods must be validated
before those rows can pass. Event timestamps alone do not measure visible output.

A slice must satisfy its exact work-count/correctness oracle and preserve
matched visual/input output. Investigate repeatable >5% CPU or GPU regressions;
if noise exceeds 5%, retain the inconclusive result and repeat with a stronger
method rather than claiming a small win. No universal GPU percentage is assigned
without workload/adapter evidence. Every adopted cache needs concrete memory
bounds; final tier budgets and demanding fixtures remain specification work.

The final endurance recipe is proposed as 60 minutes at the representative tier,
200 open/close cycles, and 20 injected recoveries, with five-minute samples after
a five-minute warmup. Live resource counts must return to the declared working
set; no monotonic live-byte growth is allowed. Allocator/driver capacity plateaus
must be explained and budgeted. Compare first/last ten-minute latency and CPU
windows; >5% drift requires investigation. This is final qualification, not a
prerequisite to the first implementation slice.

## E08 — Adoption and bounded implementation

| Slice | Consumers and existing paths | Proof required when implemented |
|---|---|---|
| S0 measurement closure | All native hosts; existing trace and OS tools | Verified target/input, baseline raw receipts, trace overhead and missing-metric register |
| S1 shared surface lifecycle and event invalidation | Main runtime and all three dialog instances; shared surface state/scheduling owner; board/schematic camera and interaction handlers; `App::request_redraw_if_needed`; owned dialog dispatch | E10 lifecycle and temporal proof in every host; production no-op and affected-window counts; pointer/focus/capture parity; measured attribution selects bounded changes |
| S2 retained preparation/upload | `PreparedScene`, retained board/schematic resources, screen-space buffers and encoded draws | Rebuild/upload/encoding counters; identity replacement and eviction negative controls; painter/AA/text parity |
| S3 shared controls/dialogs | Both Preferences, New Project, Layers, navigator, Inspector, menus/popovers | Dialog-only composition, shared continuous/row adapters, clipping/hit/keyboard tests; remove obsolete duplicated paths |
| S4 terminal and lifecycle | Terminal host plus every native surface/device | Hidden-work suppression, terminal state/input parity, fair dispatch, bounded fault/recovery and resource release |
| S5 final adoption/qualification | All preceding consumers, supported scale/backend tiers | Complete HP replay, native owner UX, independent performance replay and endurance |

Every slice records exact touched paths, dependency, invariant, negative control,
baseline/candidate receipts and rollback boundary before product edits. Undoing
an optimization restores its previous implementation without changing document
formats, accepted settings or authored data. Keep rollback possible by avoiding
unrelated renderer rewrites in a performance slice.

Future panes must declare host/window identity, viewport profile or bounded
semantic exception, damage dependencies, resource owners, cache bounds,
scroll/clip/hit/focus behavior and proof scenarios before admission. Partial
navigator/Inspector surfaces inherit these obligations as they expand; their
absence today cannot be described as completed adoption.

## E09 — Historical case allocation

The parent matrix supplies the full positive oracle and historical negative
candidate. The following assigns execution without waiving any case. A slice
touching another row's mechanism must also execute that row's relevant proof.

| Case | Contract | Primary slice | Defect-sensitive observation |
|---|---|---|---|
| HP01 | E02/E04 | S2 | Warm production hit queries: zero new layout solves |
| HP02 | E02/E04 | S2 | All layout key changes miss correctly; two-key alternation remains warm |
| HP03 | E03/E04 | S2 | Menu parse/preparation counts through redraw and accessibility |
| HP04 | E02 | S2 | Alias query allocation count and exact identity parity |
| HP05 | E01/E06 | S1 | Notify-before-present in every native host plus native pacing |
| HP06 | E03 | S2 | Warm camera encoding count; dependency replacement invalidates |
| HP07 | E03 | S2 | Adjacent-only batching, mixed painter order and fractional pixels |
| HP08 | E03/E04 | S2 | Zero unchanged world upload bytes; equal-length replacement uploads |
| HP09 | E02/E04 | S4 | Hidden terminal preparation count; reopening current accumulated state |
| HP10 | E02/E05 | S3 | Board wheel avoids Layers preparation; Layers wheel cannot zoom board |
| HP11 | E01/E02/E05 | S1 | Owning-window-only changed scroll; no-op requests zero frames |
| HP12 | E03 | S3 | Both Preferences: zero hidden world/shell/terminal preparation |
| HP13 | E03 | S3 | Dialog pass/resolve counts and equivalent pixels |
| HP14 | E04 | S3 | Warm scroll-return shaping count, bounded keys/payloads and eviction |
| HP15 | E04 | S2 | Width cache misses/hits and uncached width parity |
| HP16 | E03 | S3 | Convex-control tessellation counts; holed geometry unchanged |
| HP17 | E04 | S2 | Glyph signature survives pressure and dialog/workspace transitions |
| HP18 | E05 | S3 | Fractional delta sum/sign reversal; rounded-row negative fails |
| HP19 | E05 | S3 | Physical/line conversion exactly once at 1x/fractional/2x |
| HP20 | E05 | S3 | Content-minus-viewport maximum, thumb and final row |
| HP21 | E05/E06 | S3 | Thumb grab/page/cancel through production pointer routes |
| HP22 | E05 | S3 | Paint/hit clip parity, partial controls and pinned chrome |
| HP23 | E01/E05 | S3 | Local routing, focus reveal, changed extent and return to idle |
| HP24 | E07 | S0 | Complete scheduled input and output accounting; dropped-input negative fails |
| HP25 | E06/E07 | S4 | Serial proof separately reported; concurrent crash remains unresolved |

The existing concurrent GPU-test issue `dat-gpu-test-concurrency-6dt` stays open.
GPU tests run serially. Full historical defect replay is not required before
specification approval; its exact recipes, feasible methods and allocations are.
No case is marked passed by this table.

## E10 — Reopened shared surface and resize contract

Owner direction reopens rendering-engine development after resize QA exposed
unacceptable approximately 34% GUI CPU and visible blinking. GPS-C01R proved a
bounded retained-geometry improvement only; neither that reduction nor settled
pixel equality qualifies resize UX. This section is a proposed common engineering
contract under R39–R42, not ratification of a mechanism or new numeric ceiling.
The exact direction and current-code findings are recorded in
`docs/reviews/gui-performance/rendering-reopening.json`.

### One surface lifecycle implementation, all native hosts

Define one shared implementation for native surface state, resize scheduling,
configuration/acquisition/recovery and presentation bookkeeping. Main `Runtime`
and `GlobalPreferencesWindowSurface` are adapters to it; Global Preferences,
Project Preferences and New Project must all adopt it. Board, schematic,
terminal, dock, menus and pane layout report damage to their actual host. Future
native windows inherit the same implementation. A utility that leaves competing
configuration loops in place does not satisfy adoption. Device sharing must not
merge window damage or lifetime. Terminal PTY/state and continuous-scroll
semantics remain domain-owned.

Separate latest logical/input extent, desired physical surface configuration,
configured resource generation, submitted generation, GPU-completed generation
and independently observed displayed generation. E11 defines these milestones;
an application `present()` return is never a display acknowledgement.
Track real zero extent/undrawable state rather than silently converting it into
perpetual 1×1 rendering. Each frame uses one coherent extent/DPI/resource snapshot.
Input and layout state follow accepted events promptly, including final gesture
state; superseded unpresented resource work may be coalesced. Configuration work
belongs to the drawable frame lifecycle, not an unconditional GPU operation for
every size notification. Ordinarily allow at most one necessary configuration
per frame attempt; initialization and bounded recovery are separately classified
and counted. Do not retain a live acquired texture across reconfiguration.

Surface acquisition failure retains damage and schedules a bounded retry under
E06; no busy retry, lost final resize, stale-generation completion or late work
after close. Notify/present through the backend-appropriate boundary. Preserve
coherent visible content while a replacement is pending, with explicit backend
behavior rather than assuming old-size swapchain contents remain valid. Rendering
must never intentionally publish an empty intermediate frame. Real resize,
DPI/display changes, maximize/restore, minimize/restore and close during resize
must use these transitions. Backend capability differences are explicit adapters,
not independently invented scheduling policies.

### Resource work and isolation

Profile surface configuration/waits separately from attachment allocation,
scene/layout/text preparation, upload, encoding, queue submission, GPU execution
and compositor presentation. Ordinary resize must not invalidate an unrelated
window; exact work-count proof requires zero induced preparation/submissions for
an unchanged unrelated host. Genuine application-wide changes name dependents.
Preserve event delivery and modal focus rather than manufacturing unsupported
simultaneous gestures to satisfy a benchmark.

Attachment recreation is governed by actual extent/format/sample/device changes
and the committed frame snapshot. Audit the current full-size MSAA replacement,
pass/resolve cost, transient allocation and bytes retained in flight. Specify a
bounded lifetime and release proof before introducing pooling or size buckets;
resolve compatibility must remain valid. Unchanged frames allocate zero new
size-dependent attachments. Distinguish unavoidable changed-size allocation from
superseded work that can be eliminated. Do not lower AA/text fidelity, freeze
updates during drag, or cap input/render delivery merely to improve CPU figures. Refresh-aware pacing
and bounded backpressure are legitimate when they preserve accepted input and
final state and satisfy the agreed presentation/latency budgets; they must not
manufacture a resource pass by reducing the required work delivered.
Text, layout, screen geometry and uploads obey their own dependency keys; world
reuse alone is not the shared-engine solution. Include diagnostics/file I/O in
cost attribution and measure tracing overhead.

### Temporal proof and resource qualification

Before changing product rendering behavior, define a native-resize reproduction
and validate the temporal oracle used to accept that change. R03-authorized
opt-in diagnostics and isolated test-harness corrections may be implemented to
establish that oracle; completed production proof is not required to begin
those measurement changes. The corpus covers empty/minimal host, shell-only,
representative and demanding board/schematic scenes, both Preferences windows,
New Project, visible/hidden terminal, and multiple open windows. Exercise both
axes, corners, direction reversal, fractional DPI, maximize/restore, recovery,
close while pending and return to quiet idle. Record actual backend/adapter and
display refresh from runtime evidence; environment variables alone are not
backend proof. Keep X11/Xwayland and native Wayland results separate.

Capture continuous compositor output or independently validated display evidence
through the full interaction, synchronized with input and application events.
Detect unintended blank/clear flashes, content disappearance, stale/mismatched
extents and long repeats; distinguish legitimate newly exposed background and
moving geometry. Report presented-frame intervals, frame age and input-to-visible
latency distributions, worst stalls and final settled state. Capture cadence,
dropped frames and recorder overhead must be known: missing or undersampled
coverage is inconclusive, not zero flicker. Settled screenshots, submit counts,
`present()` calls and successful acquisition cannot satisfy this oracle.

GPS-C03 must supply explicit per-workload CPU/GPU/memory and presentation budgets
on the daily Linux reference setup, separating normal required work from fixed
shell overhead and avoidable resize churn. Approximately34% CPU is an owner-
rejected observed result, not a newly accepted ceiling; a relative improvement
alone cannot close the issue. Existing E07 numbers remain proposals. Both visual
quality and low resource consumption must pass together; reducing CPU while
increasing GPU work requires investigation, not automatic acceptance. Preserve
short investigative runs as such; qualification uses E07 repeated warm trials,
with full adoption/endurance later. No provable universal minimum is claimed.

The smoke harness must run once per explicit invocation, request real native
window sizes, observe resulting events and verify configured/presented extent
agreement. A resource-only stress test may deliberately use artificial dimensions
but must be identified as such and must not serve as native resize UX evidence.
Negative controls must demonstrate detection of a blank intermediate frame,
stale frame/extent, repeated smoke reentry, discarded final damage, cross-window
redraw and per-event reconfiguration churn. Fault injection stays in test paths.
GPS-C04 maps every invariant to shared owner, each current consumer, exact proof,
legacy path removal and artifact. Slice proof must cover all hosts changed by
that slice; remaining hosts stay explicitly unadopted. Full shared-engine
completion requires every current host and independent native replay.

## E11 — Shared scheduling, completion and recovery detail

<!-- EVIDENCE:GUI-PERFORMANCE-SPEC:GPS-C02-CONTRACT -->

This section specifies proposed behavior, not acceptance of the opt-in resize
transaction currently in the worktree. The interface names below name ownership
boundaries; implementation may use ordinary Rust structs/enums and composition.

**SCH-01.** One application-owned `NativeFrameCoordinator` owns the event-loop
work registry and minimum wake deadline. Each entry is addressed by
`(WindowId, host_generation, device_generation)` and contains dependency damage,
one pending native redraw token, latest size/DPI, retry episode, and frame state.
`invalidate`, `set_extent`, `set_drawable`, `redraw_received`, `frame_submitted`,
`gpu_completed`, `acquire_failed` and `close` are its transition inputs. Native
hosts adapt events and prepare their own content; they do not keep competing
retry loops. Pure transition tests use an injected monotonic clock and effects
such as request-redraw, configure-needed, wake-at and report-failure.

**SCH-02.** The coordinator merges deadlines from surface retries, device progress,
terminal transport/blink, engine supervision and other existing timers by taking
the earliest *eligible* deadline once at the event-loop wait boundary. A host
cannot overwrite another host's earlier deadline. With no eligible work use
`Wait`. Immediate readiness requests one redraw, never repeated zero-duration
wakeups for the same token. No deadline exists solely to redraw unchanged pixels.

**SCH-03.** Ready hosts receive round-robin service, at most one frame attempt per
host per dispatch round. A host waiting on GPU work or an undrawable surface is
ineligible and yields without discarding damage. Terminal/background work yields
at an initial proposed budget of 1 ms or 256 messages per dispatch, whichever is
first, preserving remaining bytes/messages and waking the next round. No handler
sleeps or waits synchronously for readback. A single indivisible over-budget call
is recorded as a stall and must be addressed, not hidden by the average budget.
A continuously busy terminal must not postpone another eligible host by more
than one ready-host round; native end-to-end latency still has to meet C03.

**SCH-04.** The common device/queue owner serializes application submissions and
surface configurations. Ordinary configuration waits for already-submitted work
through completion notification plus bounded nonblocking progress checks; new
submissions cannot race an active configuration. A pending configuration obtains a FIFO admission ticket
before draining outstanding work. Once admitted, new submissions are held until
that bounded prior work completes and the configuration attempt finishes; later
hosts cannot continually put more work ahead of it. Input/model processing
continues. At most one configuration ticket per host is retained; newer extents
replace its requested extent without moving its queue position. Closed or
undrawable hosts release their ticket. A stalled drain enters REC-01 failure,
not indefinite global blocking. Deterministic proof continuously submits from
host A while host B requests resize: B is admitted within one host round, no
new A submission overtakes its drain, and both resume after configuration.
Do not infer queue independence
from separate windows. Queue-wide completion can delay a host, but cannot clear
another host's damage. Start with at most one outstanding application frame per
host, no queued prepared snapshots, and at most one outstanding completion
notification per host generation. This bounds Datum work, not the driver's image
count. A backend whose configure call blocks despite readiness is an explicit
measured limitation requiring remediation; wrapping it in a shared type is not
performance proof. No separate rendering thread or backend switch is ratified.

**SCH-05.** Completion milestones have different meanings and release rules:

| Milestone | Permitted state change | Forbidden inference |
|---|---|---|
| Coherent frame prepared | Capture consumed dependency generations and physical extent/DPI | Input after this snapshot was rendered |
| Queue submission and native present call completed | Retire only consumed application damage; retain the submission receipt and resource references; newer damage remains pending | Pixels were displayed or GPU references can be reused unsafely |
| GPU completion for that submission | Release eligible transient allocations; retire in-flight accounting; enable a pending configure after acquired surface texture ownership is released | Compositor showed this frame |
| Independently observed display | Record displayed generation, extent and timestamp in qualification evidence where the backend supports observation | A missing observation is a successful frame or an application failure without further evidence |

Failure before successful submission/present leaves consumed damage pending.
A completion message carries its host/device generation; close or device reset
invalidates old messages. Callbacks only enqueue a bounded completion flag/event,
never perform GPU work, retain a window indefinitely or recreate a closed host.
Lost/outdated/suspend transitions invalidate submitted-content assumptions and
require a fresh current-state frame even if old damage was retired. GPU resources
may be dropped using wgpu's lifetime guarantees; logical in-flight byte accounting
remains until completion or documented device teardown. An acquired
`SurfaceTexture` must be released before reconfiguration.

**REC-01.** Retry episodes use monotonic **drawable active time**, accumulating
only while the host is nonzero, resumed and not known occluded. Zero extent,
known occlusion and suspension cancel scheduled retries and pause that clock.
A restore resets the backoff to 16 ms but retains accumulated failed active time;
it does not grant endless fresh two-second episodes. Successful acquisition and
submission reset the episode. A size change alone does not. Manual Retry or a
new device generation starts a new episode. Automatic delays are 16, 32, 64, 128,
then 250 ms; no automatic attempt after two seconds of active failed time.
Device recreation is limited to one automatic attempt per episode. GPU-progress
waiting uses the same bounded episode; never poll a hung queue indefinitely.

**REC-02.** Exhaustion produces a recoverable host failure with Retry and Close
through existing error reporting. If GPU drawing is unavailable, preserve a
textual diagnostic through the existing error channel; no claim that a GPU error
can always be painted. Preserve engine edits, unsaved settings and terminal core
state. Cancel active pointer capture/drag, keep logical focus identity where it
still exists, and restore IME/caret geometry after a successful current-layout
frame. Closing an auxiliary host returns focus under existing modal ownership;
failure in a main renderer does not commit, discard or replay design operations.

| Fault scenario (production owner with injected backend result) | Required observation |
|---|---|
| Zero extent after damage, then restore | No configure/acquire while zero; latest nonzero frame eventually submitted; final input retained |
| Known occlusion or suspend midway through retry | No render/retry loop while hidden; active-time counter paused; one fresh restore request |
| Lost/outdated once, then success | Config generation changes once as needed; no live old acquired texture; no lost final damage |
| Timeout continuously, including resize events | Backoff and two-second active deadline terminate retries; resize cannot reset exhaustion |
| Occlude/restore repeatedly during timeouts | Paused time excluded; accumulated active time still reaches exhaustion |
| GPU completion never arrives | Other eligible work dispatches; bounded failure; no readback wait or unbounded callbacks |
| Device lost with multiple hosts | Old callback generations rejected; all affected GPU caches invalidated; one coordinated recreation |
| Allocation failure during replacement | No empty submitted frame or unbounded retry; data and old eligible resources preserved safely |
| Close between submission and callback | No redraw/reconfigure after close; resources retire without retaining the native host |
| DPI change during thumb drag or camera gesture | Coherent final paint/hit extent; capture reconciled/cancelled by established input policy; no synthetic design mutation |

Upstream constraints checked against pinned wgpu 28.0.0 `api/surface.rs` and
`api/queue.rs`: configure waits for GPU idle, concurrent submissions may invalidate
that wait, and an old live acquired surface texture forbids reconfiguration.
Queue completion callbacks need submit/poll progress and must be short. Winit
0.30.13 `Window::pre_present_notify` schedules Wayland frame callbacks; it is not
an X11 display acknowledgement. References:
https://docs.rs/wgpu/28.0.0/wgpu/struct.Queue.html#method.on_submitted_work_done
and https://docs.rs/winit/0.30.13/winit/window/struct.Window.html#method.pre_present_notify.
These API constraints inform the proposed design; they do not prove its resource
benefit on the daily Intel/KWin environment.

## E12 — Retained controls and explicit consumer profiles

**CTL-01.** The renderer owns a bounded shared control-mesh cache. Its complete
key contains contour identity/revision, physical dimensions, corner radii,
border width, DPI, tessellation tolerance and geometry-affecting style generation.
Color/opacity enters the key only when baked into vertices; otherwise use live
uniforms. Translation and scissor-only changes reuse the mesh with current
transform/clip. Clip-baked geometry includes the clip identity in its key.
Convex controls may use the existing fan path; concave/holed authored geometry
keeps its valid general tessellator. Shape equivalence includes winding, border,
AA coverage and fractional-pixel edges. Never clamp contour vertices to clip.

**CTL-02.** Proposed initial retention is 256 mesh entries and 4 MiB of complete
CPU mesh payload per renderer, plus 4 MiB GPU retained capacity. Count keys and
payload separately; account for pinned in-flight bytes outside evictable cache
usage. Evict least-recently-used unpinned entries before insertion; oversized
meshes bypass retention, are bounded by admitted frame geometry, and retire at
completion. No eviction removes authoritative control/model state. Close/device
reset releases the appropriate owner. C03 must reconcile these proposed caps
with total working-set and scale budgets before specification approval.

**ADP-01.** All rows below use SCH/REC for their host and E02–E05 for dependencies,
resource identity and clipping. The profile is a semantic adapter, not a private
scheduler or renderer. `Main` means the common main native host, not a separate
surface for each pane. Product appearance remains under existing visual authority.

| Existing consumer | Host / shared profile | Specific preserved semantics | Pass/resolve requirement |
|---|---|---|---|
| Main shell and pane chrome | Main / shell layout and controls | One shared two-key layout result for paint/hit geometry | Shell/overlay portion of existing general frame; no standalone empty world pass |
| Board leaves | Main / retained world + CameraEngine | PaneId camera, pointer pane vs focused command, authored identity | Existing world/general schedule; preserve painter order, AA and dependent overlays |
| Schematic leaves | Main / retained world + CameraEngine | Schematic units/bounds, resolved content required | Same general schedule with schematic resources; placeholder is not schematic proof |
| Revision panes and witness | Main / screen content | Revision authority and current partial/static status | Existing ordered screen/overlay passes; no new world resolve for screen-only content |
| Global Preferences | Own native host / continuous ScrollViewport | Global settings/focus and modal routing | One dialog pass, at most one MSAA resolve; zero world/terminal/shell-backdrop work |
| Project Preferences | Own native host / continuous ScrollViewport | Actual Project event adapter and project scope | Same dialog-only schedule, independently exercised |
| New Project | Own native host / form/control profile | Existing form focus and project creation authority | Adopt one dialog pass/at most one resolve; retire workspace clone/general backdrop |
| Layers sidebar | Main / discrete-row adapter | Effective first row clamped by visible capacity; wheel stays local | Main screen/overlay contribution only |
| Project panel/navigator | Main / shell/control profile | Partial navigator remains partial; no invented tree state | Main screen/overlay contribution only |
| Inspector projections | Main / shell/control, continuous when content scrolls | Selection/check/review authority and keyboard focus | Main screen/overlay contribution only |
| Menubar/submenus/action popups | Main / retained menu inventory + transient overlay | Dynamic availability separate from immutable inventory; keyboard/accessibility parity | Existing overlay order; closed surfaces prepare nothing |
| Console feedback/history | Main / bounded screen content | Retain feedback when hidden, independent command feedback damage | Existing overlay contribution; hidden history prepares nothing |
| Terminal dock/tabs/split leaves | Main / terminal-row adapter | PTY/core rows, two-generation text churn, selection/search/input | Existing terminal text/graphics and main composition; no generic document-scroll conversion |
| Terminal clipboard/context popup/graphics | Main / terminal overlay adapter | Confirmation, links, cell clip and session graphics ownership | Existing terminal/overlay order; graphics allocation separately bounded |

**ADP-02.** General-frame passes remain as required by existing world composition,
selection, text and blending. Before modifying them, record the actual pass graph,
per-pass inputs/load/store/resolve and the visual necessity of every retained
pass. The contract does not prescribe a fabricated universal one-pass world
renderer. C04 binds each row to exact implementation paths and pass-count proof;
a duplicated dialog pass fails even when pixels match. Future consumers must
select an existing profile or document the minimal semantic difference and its
proof, while inheriting shared scheduling/resource ownership.

**PASS-01.** The inspected `render/gpu_frame.rs` and `terminal_graphics.rs`
encode the following general-frame schedule, shared by Main consumer rows:

| Ordered pass | Activation | Ordering reason | Current resolve count |
|---|---|---|---|
| Geometry (`datum-gui-render-pass`) | Every general frame | Clear target; panel, grid, ordered authored world, interaction and console geometry | 1 |
| Terminal background graphics | Nonempty admitted background graphics layer | Images below terminal text | 1 if active |
| Main text (`datum-gui-text-pass`) | Every current general frame | Text above base geometry/background images | 1 |
| Terminal foreground graphics | Nonempty admitted foreground graphics layer | Preserve terminal image z-order above text | 1 if active |
| Menu card (`datum-gui-menu-overlay-pass`) | Nonempty menu overlay vertices | Occlude underlying geometry **and main text** | 1 if active |
| Menu text (`datum-gui-menu-overlay-text-pass`) | Menu card active and nonempty menu text | Labels above their card | 1 if active |

Thus current general frames have `2 + B + F + M + MT` passes and resolves,
where each indicator is zero or one under its activation condition; maximum six.
Dialog-only frames instead use exactly one pass/resolve, drawing card and its
text together. Counts are per submitted frame, not per pane; board/schematic
leaves share the geometry pass through per-pane scissor/camera bindings.
Terminal graphics layers are absent when no applicable graphics are admitted.

These counts describe the inspected implementation ceiling, not proof that every
resolve or empty text pass is necessary. A future slice must remove a pass with
no contributing commands and may merge compatible stages or defer intermediate
resolves only after proving load/store, MSAA, atlas lifetime and painter parity.
Any retained extra stage needs a named ordering/resource dependency; no duplicated
resolve is justified merely because the old implementation performed it. Tests
count passes/resolves and reject deliberately redundant identical-output passes;
visual tests additionally expose a menu card drawn before underlying text and
terminal foreground/background inversion. Quality stays at the existing sample
count. This contract neither reduces AA nor claims a pass-count change alone
will fix the measured submission CPU cost.

## Initial qualification scope disposition

The subsequent owner scope record
`docs/reviews/gui-performance/initial-qualification-owner-direction.json` limits
initial qualification to the acceptance matrix's pinned small-project scope and
excludes flicker/displayed-frame latency acceptance. E07/E10/E11 displayed-frame
oracles and demanding-tier proof remain deferred, unqualified requirements for
full acceptance. They do not block drafting the limited initial specification.
Keep CPU/GPU/memory, static appearance, input/focus, clip/hit, final-state and
shared-adoption obligations. No application milestone is relabeled as observed
display, and no initial pass closes the unresolved resize defect. This subsequent
scope disposition does not change the independently reviewed C02 mechanisms.

## C04 allocation reconciliation

The E09 slice column is reconciled to the individual implementation map. Earlier
allocation was provisional: layout/menu/width reuse is S2, native notification
is S1, Layers routing is S3, and terminal hidden work is S4. Every slice still
runs its relevant HP regressions; S5 replays complete in-scope adoption.
The acceptance matrix MET/MEM/STAT methods supersede E07 starting-point sampling
where more specific: fixed seven pairs for relative inference, no adaptive
significance loop, three valid trials for absolute limits. Owner scope exclusions
remain; no historical trial or implementation is requalified by this allocation.
