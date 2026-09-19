# GPS-C01 current consumer and measurement reconciliation

<!-- EVIDENCE:GUI-PERFORMANCE-SPEC:GPS-C01-RECONCILIATION -->

Inspected revision: `38b411d6`. The workspace was clean before the synchronized
GPS-C01 claim. This audit changes measurement/research records, not production
Rust or rendering semantics. The optimized binary is the previously verified
candidate with SHA-256
`d476c5769b4d55ca2afae0ee71e0f701da753c8200d14fd851e7c73d3fff3042`;
its embedded revision predates the commit that recorded its source/proof.
`measurements/candidate-sources.json` binds the changed Rust source. No rebuild
is needed to relabel that same binary as a new implementation.

## Consumer inventory and actual ownership

This is current implementation evidence, not an override of the Rendering Book,
viewport, Preferences, terminal or revision-workspace decisions. In particular,
partial panes and static examples are not complete product implementations.

Main-host ownership means `App`/`Runtime` in `crates/gui-app/src/main.rs`,
`app_shell.rs`, `interaction_refresh.rs` and `runtime_present.rs`. The event loop
uses `ControlFlow::Wait`; `redraw_pending` coalesces main requests. The general
`App::request_redraw_if_needed` also invalidates every open owned product window.
An explicit Preferences wheel route avoids that broadcast. A terminal wake can
still reach it. This is a confirmed reachability observation, not a measured
claim that every main event wastes work in every window.

Main preparation means `crates/gui-render/src/render/scene.rs`, retained authored
board/schematic data, shell/pane overlays and the shared `Renderer`. Main upload
means `gpu_vertex_upload.rs` and `gpu_strokes.rs`; main encoding means
`gpu_surface_pass.rs` and `gpu_surface_pass/world_bundles.rs`. Consumers sharing
the host also share its device/surface lifecycle; they do not own separate GPUs.

| Consumer | Layout/preparation and rendering owner | Input, clipping and lifetime | Current reconciliation / proof boundary |
|---|---|---|---|
| Main native window/shell | Main host; `runtime_camera_pane/layout_cache.rs`; shared main preparation/encoding/upload | `main.rs`, `runtime_primary_pointer.rs`, `keyboard_focus.rs`; window-sized shell geometry; main renderer lives for Runtime lifetime | HP01–05. Pointer-only refresh retains prepared/world data. Native resize/DPI currently invokes `invalidate_scene`, clearing authored caches as well as layout. Main window reference is process-lived; do not claim per-window teardown proof. |
| Board pane, including additional board leaves | `runtime_camera_pane.rs`, `pane_cameras.rs`, `gui-viewport` camera/hit services; retained board plus per-pane uniform/scissor and world bundle | Pointer-pane camera route versus focused-pane commands; stable PaneId; pane close drops camera references through pane operations | HP01/02/04/06–10. Landed zoom comparison suppresses unchanged camera invalidation. Screen-space uploads remain per-frame. Tests for camera math/cache identity are not complete native work-count proof. |
| Schematic pane, including additional leaves | `runtime_camera_pane.rs`, `schematic_retained.rs`, same Renderer and pane-camera services | Schematic coordinates and bounds remain distinct; current DOA2526 board-only fixture displays an unresolved schematic placeholder | HP06–08 apply to actual resolved schematic content. Board-only baseline cannot qualify schematic geometry/text or cross-probe behavior. |
| Revision surface panes and witness | `revision_workspace.rs::render_pane`; main host/preparation; screen-space quads/text | `runtime_revision_workspace.rs::apply_revision_hit`; `clip_pane_output`; close/open-beside are consumer state | Partial/static revision surfaces inherit host scheduling/lifecycle. Do not describe their fixed example strings as engine-backed acceptance. Future adoption must preserve revision authority. |
| Global Preferences native window | `global_preferences_window.rs`, `preferences_scene.rs`, `preferences_rows.rs`; separate Renderer on shared device, dialog-only one-pass path | `global_preferences_window_event`; `ScrollViewport`; common viewport feeds paint/hits/scrollbar, but current quad/hit clipping is vertical while text receives rectangular clipping; full horizontal/nested parity is unproved; renderer/surface dropped when host closes | HP05/11–23. No workspace clone/world composition on this path. Input modality must remain during measurements. Global Units is one section, not every settings configuration. |
| Project Preferences native window | `project_preferences_window.rs` state/event adapter, shared `GlobalPreferencesWindowSurface` and dialog renderer | Project's native event dispatcher invokes the shared wheel/capture helpers; independent dialog state/host lifecycle | Same HP obligations, independently exercised. A test that merely changes the shared fixture title to Project does not exercise this adapter. |
| New Project native window | `new_project_window.rs`; shared native surface type, but its `new_project` branch clones workspace and constructs retained/general prepared scene | Native form handling and focus under New Project authority; host close releases owned Renderer; no project creation during baseline | HP05 and R37 adoption gap. Preferences dialog-only optimization deliberately did not replace this branch. Measure opening/closing independently; do not silently include it in HP12's claimed fix. |
| Layers sidebar | `side_panels/render_project_filters.rs`, `side_panels/layer_scroll.rs`; main prepared scene | `runtime_revision_workspace.rs::handle_layer_scroll`; discrete first-row authority and `LayerScrollRegion` hit rectangle | HP01/02/10. Board wheel fast-rejects before Layers preparation. Layers currently invalidates after supported wheel even when effective visible first row cannot change; requested offset uses layer_count-1 while paint clamps to layer_count-visible_capacity. Investigate separately; preserve meaningful row semantics. |
| Project panel / navigator region | `side_panels.rs`, `side_panels/layout.rs`, `render_project_filters.rs`; main shell projection | Project label/scene summary is current visible implementation; source comments reserve fit-button space for the revision navigator | Partial consumer, not a complete navigator. Must inherit scheduling, common clipping/hits and resource rules as implemented. No invented scrolling workload for an absent tree. |
| Inspector selection/review/check/evidence projections | `side_panels/inspector_dispatch.rs`, `render_inspector.rs`, `inspector_check_finding.rs`, `revision_workspace.rs::render_evidence_inspector` | Main prepared hit regions and current selection dispatch; layout recalculated with prepared shell | HP04 and R37. Populated component editing remains separate product work. Content/layout/selection dependencies need explicit adoption; current absence of a general scroll owner is a gap, not permission to invent one per panel. |
| Menubar, submenus and popup action surfaces | `menu_chrome.rs`, `marking_menu.rs`; cached `gui_menu_model.rs` inventory; main overlay buffers and shared text owner | `runtime_menu_actions.rs`, keyboard/accessibility dispatch; popup hit regions and availability; mutable session menu state independent of static inventory | HP03/04/17. Static inventory reuse exists; production parse/preparation and closed-menu allocation counters remain missing. Dynamic availability must update even when inventory is retained. |
| Console feedback/history overlay | `production_status_refresh.rs`, main prepared console overlay and `console_gpu` | Consumer feedback records and optional expanded history; hidden inspection fast-rejects; main host lifecycle | HP09. Hidden-history skipping is not removal of feedback records. Focused zoom may legitimately produce console damage even when camera is clamped. |
| Terminal dock, tabs, split terminal leaves, selection and search | `runtime_terminal_render.rs`, `runtime_terminal_geometry.rs`, `terminal_session_render.rs`, `terminal_scene.rs`, `bottom_dock.rs`, terminal renderer/cache | `TerminalCore`/PTY own rows/history; terminal input/focus routes are separate from editor actions; hidden dock skips snapshots while transport/core continue | HP01/02/09/17/23/25. Row semantics remain legitimate. Bounded two-generation text policy must survive shared-cache changes. Terminal wake fairness and window-broadcast interaction need mixed-activity proof. |
| Terminal clipboard/context popup and graphics | `terminal_clipboard_menu.rs`, `terminal_scene.rs`, shared main overlay/text/graphics paths | Terminal selection, hyperlink/clipboard confirmation, cell/viewport clipping; session/renderer resources follow their declared owners | Same terminal authority; not a separate generic scheduler. Graphics allocation/eviction and nested clipping need their own workload evidence within terminal qualification. |

`PaneContent` currently enumerates Board, Schematic and Revision; `DockTab`
currently enumerates Terminal. This inventory includes their auxiliary native
windows, shell panels and transient surfaces. A future 3D/library pane is not
silently counted as a currently implemented consumer.

## Cache and surface findings

- **Confirmed retained ownership:** board/schematic triangle vertices use
  `Arc<[Vertex]>`; world encoding tracks buffers, bindings and command sequence.
  Closed panes truncate eligible bundle storage. Warm camera traces show no
  retained rebuild or bundle re-encoding on the measured board workload.
- **Suspected correctness risk, not reproduced corruption:** board/schematic
  stroke upload identities still compare source address and length in
  `gpu_strokes.rs`. A Vec allocation can be replaced; the triangle Arc identity
  test is not proof of stroke lifetime safety. HP08 must include this path.
- **Confirmed repeated screen work, unquantified bytes:** general rendering
  invokes screen-space uploads for panel, underlay, overlay, interaction,
  console/menu and applicable schematic overlays. Capacity reuse does not prove
  byte reuse. The existing traces lack upload-byte counters.
- **Bounded keys are not bounded payload proof:** layout has two entries;
  retained scene history has six plus active references; dialog labels have
  128 entries/32 KiB key text; width measurements have 256/64 KiB key text.
  Shaped buffers, glyph atlas, active frames and retained GPU capacity need
  separate accounting. Process/thread-local caches need eviction proof in
  addition to host close/drop checks.
- **Surface asymmetry:** main lost/outdated/timeout paths invalidate pending
  content; owned surfaces reconfigure on lost/outdated and return on timeout.
  Neither path currently implements the draft's full bounded retry/failure
  matrix. Main `resumed` ignores an existing window; absence of dedicated
  suspended/occlusion handling cannot be described as qualified recovery.
  These are current observations, not authorization to install the proposed
  retry policy during GPS-C01.

## Measurement recipe and honesty boundaries

Continue using the pinned disposable DOA2526 input path from the accepted zoom
comparison, isolated TMPDIR/config/cache, optimized binary and existing XTest,
Spectacle, timing/state diagnostics and `/proc` counters. Never run a mutating
fixture check on tracked fixture state. Session-owned instances alone receive
input. Three ten-second investigative samples follow five-second warmup;
preflight screenshots occur outside timing. Reuse is limited to the matched `baseline-main-v4` / `candidate-main-v3`
ordinary pointer/zoom comparison and `matched-model-pixels.json` control, rather
than rerunning them solely to change a commit label. Early unmatched import
comparisons and rejected capture methods remain invalid for causal pixel proof. These are baseline observations, not R19's later
thirty-second/full qualification or endurance results.

New workloads: Space+primary pan, native focus switching, alternating main
resize, View split/close, New Project open/cancel, and alternating interior
scroll input independently in both Preferences hosts. Scripts record requested
schedule and actual sent actions. Opening/closing may take longer than one
nominal period; actual elapsed time remains the denominator. Use existing
action-state diagnostics for camera/pane transitions where available; trace-off
samples distinguish diagnostic cost from ordinary application cost.

The pilot revealed why output verification must be specific: clicking a pane
header did not change focus although cursor pixels changed. That pilot focus
row is rejected and the recipe uses the actual Tab focus route. A generic
different-image check alone does not prove the intended operation.

GUI CPU sums all process threads. Child accounting records waited-for child CPU
plus observed live descendants, retaining raw start/end values; it does not
claim coverage of unrelated daemons or eliminate exit races. Existing DRM
counters identify clients, but newly created/closed window clients can disappear
between samples. Surviving-client duty for window cycles is incomplete and must
not be presented as total GPU activity. Stable-host observations remain useful.

Native Wayland input, compositor presentation opportunities/input-to-display
latency, demanding-scale fixtures, full live GPU/upload accounting, exact
diagnostic overhead and endurance remain distinct qualification work. XTest
drives the Xwayland native event route on the daily KDE desktop; it does not
become native Wayland evidence because the compositor is Wayland. No missing
metric is marked passed and no approval scope is silently narrowed.

## Historical reconciliation

`docs/reviews/gui-performance/c01-historical-hunk-review.json` records the separate
reviewer's enumeration of all 101 diff hunks across the six corrective commits.
Each hunk has its exact first parent, path, header, content hash and HP mapping
or explicit non-performance disposition. Compound/extraction hunks identify
which behavior changed. All 25 HP cases have current-test and missing-proof
records; 42 test references were verified against exact current source hashes.
The author independently checked the diff hashes, hunk counts/hashes and test
definitions. This establishes coverage reconciliation, not historical runtime
replay or a claim that all HP cases currently pass.

The old rounded-wheel no-op test and old fitting-dialog scroll test were
superseded by the fractional-scroll repair. They are explicitly excluded from
the current-test list. HP24 and HP25 remain measurement/failure obligations,
not invented source hunks. GPR-02's later full implementation requirement-to-
consumer-to-proof mapping remains separate from this GPS-C01 source audit.

The Layers boundary finding is tracked as `dat-layers-scroll-boundary-5fl`.
The earlier import-order finding remains `dat-import-path-painter-order-u6i`;
the concurrent GPU-test failure remains `dat-gpu-test-concurrency-6dt`.

## Remaining measurement methods and architecture impact

These dispositions satisfy the requirement to expose missing measurements; they
do not waive the later requirement to resolve feasible methods before approval.
GPS-C01 collects an investigative baseline. GPS-C03 defines complete budgets and
qualification methods; implementation supplies full production proof later.

| Metric / workload gap | Available observation | Missing method and effect on architecture choice |
|---|---|---|
| Native Wayland input | Xwayland production event path on daily KDE; native presentation notification exists in code | A validated compositor-native input recipe or coordinated physical-input capture is required. XTest evidence cannot qualify Wayland event conversion/pacing. Do not infer a backend replacement requirement. |
| End-to-end latency / displayed-frame opportunity accounting | Existing render/acquire/present-call timing and compositor screenshots | Need correlated input and compositor presentation/capture timestamps with uncertainty, final-state and missed-opportunity accounting. No displayed FPS or latency budget passes from submission traces. |
| Per-frame GPU execution | Per-client DRM engine active nanoseconds | Frequency varies; no GPU timestamp pipeline is currently measured. Use duty for matched investigative comparisons only; GPU timestamp/alternate method belongs to a bounded diagnostic slice. |
| GPU duty across recreated resources/windows | Start/end surviving-client counters | Preserve cumulative client accounting or capture before client teardown; until then window-cycle totals are incomplete. No claim that window opening costs only the surviving host's counter. |
| Upload bytes / live GPU allocation | Retained/prepared/bundle traces, CPU identity tests, process RSS | Instrument actual upload/allocation boundaries and lifetime accounting. Do not choose larger caches or claim zero upload solely from retained-scene hits. |
| CPU shaping/layout/allocation work by consumer | Existing helper tests, frame preparation timing, CPU totals | Production counters/allocation observations needed for HP01/03/04/12/14–16. Existing tests identify insertion points; they do not justify universal zero-work claims. |
| Demanding geometry/text tiers and resolved schematic | Small real board counts pinned; board-only schematic placeholder | Obtain valid existing representative large and schematic fixtures and define supported tiers. No fabricated large fixture or extrapolated capacity guarantee. |
| Terminal mixed activity / focus fairness | Source route and existing terminal tests; quiet child CPU in current trials | Controlled PTY output plus simultaneous allowed interaction needed. Do not bypass Preferences input modality to manufacture editor concurrency. |
| Surface loss/device/allocation/retry lifecycle | Inspected native error paths and ordinary resize/window cycles | Deterministic faults and native lifecycle qualification needed. Draft retry policy remains proposed; ordinary resize is not failure/recovery proof. |
| Long-session resources / endurance | Short sampled RSS and resource-owner inventory | Proposed long-run plateau/release tests remain final acceptance; ten-second samples do not establish leak freedom. |
| Instrumentation overhead | Pan runs with and without timing/action-state logs; earlier clamp trace-on/off pair | Counts and scheduling can differ; no precise universal overhead factor. Prefer diagnostics for work identification and logs-off trials for resource estimates. |

The alternatives remain incremental: first inspect screen-space preparation and
uploads on ordinary camera frames; examine resize invalidation dependencies;
measure/adopt dialog-only New Project composition; then narrow broadcast redraw
where mixed activity demonstrates irrelevant work. Source reachability alone
does not rank their resource benefit. The prior accepted no-op zoom correction
remains complete and is not repeated. Existing Qt/wgpu/winit references in the
parent audit support retained work and explicit ownership; they authorize no
new dependency, framework migration or renderer thread.

## Current baseline observations

Raw receipts, scripts, screenshots, current tool/source hashes and summaries are
under `docs/reviews/gui-performance/measurements/c01/`. The resolved board has
39 packages, 81 pads, 148 tracks, 46 vias, 20 zones, 24 nets and 24 declared
layers. These are model counts, not GPU primitive/visible glyph counts or a
demanding tier. The normalized board hash excludes only its root UUID; imported
object IDs and ordering remain included.

| Workload / receipt | CPU % of one core, three trials | Observation and limitation |
|---|---|---|
| Pan, `main-v2` | 30.7 / 29.0 / 29.6 | Timing and action-state logs enabled; 528 / 501 / 521 prepared rebuilds, zero retained/bundle rebuilds. Production camera state changes. |
| Pan, `pan-uninstrumented` | 16.2 / 17.2 / 17.0 | Both logs disabled; GPU duty 38.62 / 41.13 / 41.39%. Diagnostic overhead is material; changed scheduling/output trajectory prevents treating the difference as an exact additive overhead. |
| Focus switching, `main-v2` | 1.1 / 1.0 / 1.0 | Tab changes actual focused PaneId; 19 frames per trial, zero retained misses. Unresolved schematic placeholder is not schematic content qualification. |
| Alternating resize, `main-v2` | 4.8 / 5.0 / 4.9 | 19 / 20 / 20 retained rebuilds. Existing `apply_resize` clears authored caches. Actual runtime sizes are 1280x800 and 1333x833; requested X window sizes are subject to scale/minimum enforcement. |
| Split/close pane, `main-v2` | 3.8 / 3.7 / 3.6 | Actual state alternates two/three panes; zero retained misses. Bundle changes occur as pane membership/resources change; not every bundle rebuild is redundant. |
| New Project open/cancel, `main-v2` | 11.4 / 11.4 / 11.5 | Seven open/cancel cycles per trial; captures verify native form. No project created. Shared general renderer emits no dialog-only timing line, so a zero dialog-only counter is not zero owned-window frames. GPU total across client lifetimes remains incomplete. |
| Global Units scroll, `global-observed-v5` | 9.5 / 9.1 / 8.9 | XI2 observer sees 197 / 197 / 198 injected presses; 389 / 393 / 390 dialog submissions, zero main submissions. Observer delivery is not application acknowledgement or displayed FPS. |
| Project Units scroll, `project-v3` | 8.5 / 8.9 / 9.2 | Independently exercised native host; 393 / 392 / 391 dialog submissions, zero main submissions. No XI2 observer in this earlier run. |

All warm samples report zero additional waited/live child CPU at the available
clock-tick resolution. Startup already accumulated roughly ten child CPU-seconds
for import/initialization; it is recorded separately in the raw counters, not
credited as a warm rendering saving. RSS observations are samples, not complete
allocation or peak-memory instrumentation.

The initial Global scroll repeats vary (6.5 / 8.9 / 6.9% CPU), and its final idle
sample has 154 dialog submissions/4.3% CPU. A separate uninstrumented main run
also has activity in initial idle (7.6% CPU), but returns to zero measured duty.
Both anomalies are retained. The owner confirmed an idle desktop before the
controlled Global repeat: all three final ten-second samples then have zero
observed XI2 input, zero frames, zero CPU ticks and zero GPU counter delta.
Injected scroll presses validate that the observer receives events. This does
not explain the earlier activity or prove permanent absence of an idle loop.
It also does not establish the proposed sixty-second/endurance acceptance rule.

The corrected Global recipe waits for window resize/layout before selecting the
narrow-layout Units tab. The earlier Appearance/resize-race failures are kept
as rejected preflights. No low-duty failed scroll target is accepted as a pass.

The subsequent `main-idle-observed` repeat also records zero CPU ticks, GPU
activity, render submissions and observed XI2 events in its initial and three
final ten-second samples. Existing main-window verbose diagnostics are archived
with that report. The intermittent activity remains unresolved as
`dat-gui-idle-sample-activity-lhb`; a quiet repeat is not a retrospective pass for
the earlier samples. All session-owned measurement instances were terminated.
