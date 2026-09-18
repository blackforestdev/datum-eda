# Shared GUI engineering contract

Status: proposed engineering specification for GUI-PERFORMANCE-SPEC. This is
the concrete contract requested by the owner's sequencing correction, not
ratification of a new mechanism or a declaration of production acceptance.
The parent is `GUI_PERFORMANCE_RECOVERY_PLAN.md`; its R01–R38 and HP01–HP25
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
| S1 event invalidation | Main board/schematic camera and interaction handlers; `App::request_redraw_if_needed`; owned dialog dispatch | Production no-op and affected-window counts; pointer/focus/capture parity; current baseline selects the first bounded change |
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
| HP01 | E02/E04 | S1 | Warm production hit queries: zero new layout solves |
| HP02 | E02/E04 | S1 | All layout key changes miss correctly; two-key alternation remains warm |
| HP03 | E03/E04 | S2 | Menu parse/preparation counts through redraw and accessibility |
| HP04 | E02 | S1 | Alias query allocation count and exact identity parity |
| HP05 | E01/E06 | S4 | Notify-before-present in every native host plus native pacing |
| HP06 | E03 | S2 | Warm camera encoding count; dependency replacement invalidates |
| HP07 | E03 | S2 | Adjacent-only batching, mixed painter order and fractional pixels |
| HP08 | E03/E04 | S2 | Zero unchanged world upload bytes; equal-length replacement uploads |
| HP09 | E02/E04 | S4 | Hidden terminal preparation count; reopening current accumulated state |
| HP10 | E02/E05 | S1 | Board wheel avoids Layers preparation; Layers wheel cannot zoom board |
| HP11 | E01/E02/E05 | S1 | Owning-window-only changed scroll; no-op requests zero frames |
| HP12 | E03 | S3 | Both Preferences: zero hidden world/shell/terminal preparation |
| HP13 | E03 | S3 | Dialog pass/resolve counts and equivalent pixels |
| HP14 | E04 | S3 | Warm scroll-return shaping count, bounded keys/payloads and eviction |
| HP15 | E04 | S3 | Width cache misses/hits and uncached width parity |
| HP16 | E03 | S3 | Convex-control tessellation counts; holed geometry unchanged |
| HP17 | E04 | S2/S4 | Glyph signature survives pressure and dialog/workspace transitions |
| HP18 | E05 | S3 | Fractional delta sum/sign reversal; rounded-row negative fails |
| HP19 | E05 | S3 | Physical/line conversion exactly once at 1x/fractional/2x |
| HP20 | E05 | S3 | Content-minus-viewport maximum, thumb and final row |
| HP21 | E05/E06 | S3 | Thumb grab/page/cancel through production pointer routes |
| HP22 | E05 | S3 | Paint/hit clip parity, partial controls and pinned chrome |
| HP23 | E01/E05 | S3 | Local routing, focus reveal, changed extent and return to idle |
| HP24 | E07 | Every slice/S5 | Complete scheduled input and output accounting; dropped-input negative fails |
| HP25 | E06/E07 | S4/S5 | Serial proof separately reported; concurrent crash remains unresolved |

The existing concurrent GPU-test issue `dat-gpu-test-concurrency-6dt` stays open.
GPU tests run serially. Full historical defect replay is not required before
specification approval; its exact recipes, feasible methods and allocations are.
No case is marked passed by this table.
