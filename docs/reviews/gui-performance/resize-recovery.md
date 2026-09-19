# Active resize defect recovery

<!-- REQ:GUI-PERFORMANCE-SPEC:GPS-C01R -->

Status: unresolved; execution paused for owner-directed GPS-C02–GPS-C06
specification development. See `specification-resumption-owner-direction.json`.
The historical activity and directions below describe the investigation; they
do not override the subsequent owner instruction or establish acceptance.

The owner directed: "I the owner are demanding that the resize defect is the active priority untill resolved. this is your new goal". This reopens GPS-C01R ahead of GPS-C02. Prior geometry-reuse measurements and review remain historical partial evidence; they do not close the resize defect. The owner rejects approximately 34% CPU and reported blinking. No unrelated development displaces this work.

Execution covers corrected native-window reproduction, bounded diagnostics, cause isolation, shared rendering lifecycle corrections and affected-consumer regression proof under E01-E10/R39-R42. Define each change's invariant and proof before product edits. No new dependency, protected visual edit, full renderer replacement or final product acceptance is granted by this recovery. Full specification mechanism ratification remains separate; existing-behavior corrections and diagnostic/harness work proceed under this explicit direction.

## Proof before implementation

First establish ordinary native-window resize behavior with the smoke flag disabled. Correct the explicit smoke harness so it runs once, requests real native sizes, observes resulting events and proves final state; resource-only artificial-size stress is not UX proof. Record actual window/backend/adapter identity. Preserve failed and contaminated trials.

Separate surface configuration/waits, size-dependent attachment allocation, CPU layout/text/uploads/encoding, queue submission, GPU execution and presentation. Compare empty host, shell and representative board at matched sizes/rates. Use existing timing/profiling/capture tools before adding bounded opt-in instrumentation. No concurrent compilation during resource measurement; use disposable fixtures and session-owned windows.

Acceptance requires continuous visible-output evidence through resize, with known capture cadence/loss/overhead and negative controls detecting blank/stale/mismatched frames; correct final size, geometry, hit testing and terminal state; and repeatable low-resource results against explicit workload budgets. Approximately 34% or a relative reduction alone is insufficient. Until a validated temporal method and justified resource budgets exist, measurements are investigative and the defect remains open. Preserve quality and input state; legitimate refresh-aware pacing is allowed.

Apply successful changes through common owners used by main and all three dialog hosts, recording any still-unadopted host rather than claiming global completion. Cross-window invalidation, recovery/zero extent/DPI and closure must retain their own proof. All HP01-HP25 remain allocated; run relevant slice proof now, full adoption/endurance at final acceptance. Independent review and native evidence are required before defect closure.

## Current evidence and open work

Historical receipts: resize-slice.md, resize-review.json, rendering-reopening.json and measurements/resize/. The smoke test's repeated artificial-size sequence is a confirmed harness defect, not yet proof of the ordinary application's flicker cause. Main and dialog surfaces configure per changed size, and attachment allocation follows changed extents; their contributions require isolation. No new runtime fix or acceptance is claimed by this activation.

## Harness correction slice H1 — predefined boundary

H1 changes only explicit smoke operation and frame-result bookkeeping. Extract
main redraw/capture ownership into a normal module. Advance the resize smoke
through actual native requests and successful application presentation attempts,
checking observed window size against configured size before advancing. Run once;
wait for resize events rather than applying several artificial sizes inside one
redraw. Preserve ordinary rendering behavior and the prior geometry reuse. Add
state-transition tests for mismatched/pending extent, failed acquisition and
no reentry after completion. The native run must log one begin/end and each
observed target; it remains application-level lifecycle proof, not a compositor
flicker or timing pass. Record actual winit backend and adapter identity at startup.

H1 verification: three focused state tests and optimized build passed. The same
final binary passed six native resize targets on X11 and Wayland, one begin/end
per process and no reentry during the two-second observation tail. Actual handles
identify X11 versus Wayland; both selected Intel P630/Vulkan/Mesa 25.0.7. Earlier
Wayland attempts timed out because startup compositor configuration superseded
the first request; those failures are preserved. A smoke-only 200ms native-size
quiet interval now precedes the first request, using the event-loop deadline
rather than a redraw loop. Locally returned sizes are recorded separately and
only rounding-sized differences can satisfy the requested target. Interaction
smoke explicitly fails when acquisition skips either required frame.

Receipts: measurements/resize-recovery-h1/. This is harness verification only:
no new resource comparison or continuous compositor capture has passed, and the
owner's CPU/flicker defect remains unresolved and selected. Existing traces show
substantial surface-configure wall time; wall time is not CPU attribution.

## Cost attribution slice H2 — predefined diagnostic boundary

Use opt-in phase probes to distinguish elapsed time from process CPU and UI-thread
CPU during surface configure, acquisition, scene preparation, renderer work and
presentation. Read existing libc CPU clocks; introduce no dependency. Disabled
probes must not read clocks or write logs. These time windows attribute when CPU
runs, not the causal origin of asynchronous driver work. Compare instrumented and
plain release runs before relying on percentages. Keep ordinary resize behavior
unchanged while measuring; isolate configure cost before selecting a correction.

H2 also permits an explicit diagnostic backend override for Vulkan/GL already
compiled into the existing wgpu dependency. Ordinary startup selection remains
unchanged. Compare backend identity, correctness, CPU/GPU and resource behavior;
a diagnostic result alone cannot change the product backend policy. No new
package, dependency, lower-quality rendering mode or acceptance is introduced.

H2 findings (investigative, not acceptance): the final identical optimized binary
used 34.8%/33.8% of one core for vertical/horizontal X11 resize without probes,
and 37.0%/36.4% with probes (one five-second trial per axis). No owned-window XI2
input arrived in these windows; idle measured 0%. The 2.2/2.6 percentage-point
increase prevents treating instrumented totals as uninstrumented performance.
Configure windows accounted for about 19.3/20.8 CPU percentage points; renderer
windows about 13.2/11.3. These are sequential measurement windows containing asynchronous work whose
origin may lie elsewhere, with partial calls at measurement boundaries; they
do not form an exact causal partition.

An earlier GNU gprofng diagnostic sample independently placed 43.36% of sampled
CPU under surface configuration and 27.43% under Vulkan queue submission. Device
filesystem discovery (`drmGetDevice2`) appeared within configuration (18.23% of
sampled CPU); kernel ioctls dominated submission. Sampling captured less CPU than
process counters, so these percentages are hotspot evidence only. This directs
further shared-lifecycle work toward redundant swapchain configuration, waits,
and allocation/submission churn, rather than another world-geometry rewrite.
Attachment and staging allocation contributions remain hypotheses to isolate.

Native Wayland programmatic geometry pilots confirmed the same configure path.
An explicitly selected GL pilot shifted costs into presentation; GL on X11
failed initial configuration with `Invalid surface`. Neither result changes the
normal backend policy. KWin geometry requests are not physical interactive border
drags. GDB/gprofng runs deliberately omitted screenshot preflight; their inherited
report labels are corrected in failed-attempts.json. No temporal visual pass is
claimed. Historical failed runs remain archived, not silently excluded.

H2 implementation keeps disabled probes free of clock reads/log writes, splits
encoder finish from queue submission in opt-in timing, and extracts frame encoding
into a normal module. The identical surface defaults now have one helper consumed
by main and all three dialog hosts: existing format preference, FIFO, latency two,
alpha and extent behavior are preserved. Diagnostic backend selection is explicit
and defaults to the unchanged wgpu instance policy. Historical `submit` timing
included encoder finish; new `submit_us` excludes it, so comparisons must sum
`finish_us + submit_us`. Independent source review found no remaining blocker.
All 173 renderer tests and three native smoke state tests passed. The final binary
passed six native resize targets once on both X11 and Wayland. These are lifecycle
checks, not visible-frame or resource acceptance. Raw receipts, source review,
control comparisons and failed attempts are under measurements/resize-recovery-h2/.
GPS-C01R and the resize defect remain active; GPS-C02 does not advance.

## Attachment isolation H3 — predefined diagnostic boundary

Before selecting a cache/bucket mechanism, compare unchanged production rendering
at a fixed physical extent with ordinary MSAA reuse versus forced same-size MSAA
replacement. Use an explicitly ignored Linux GPU diagnostic test, optimized build,
existing wgpu/pollster only, identical prepared scene/target/sample count, warmed
resources and alternating A/B/B/A phases at 60 requested frames per second. Wait
for each submission in both conditions; record process CPU, elapsed time and late
frames, and compare readback pixels outside the timed phases. This isolates an
allocation contribution under serialized submission, not native resize UX or a
production resource ceiling. No automatic backend fallback or quality reduction.
No product allocation policy changes are introduced by this diagnostic. Preserve
failed trials. A native recording pilot failed compositor window selection and
produced no video; it supplies no visual evidence.

H3 result: Intel P630/Vulkan, unchanged 8x MSAA, 1280x800, four 180-frame
phases. Reuse measured 5.00%/5.67% process CPU; forced replacement 6.00%/5.67%.
All readbacks matched. Replacement missed 4/3 requested deadlines versus 0/0
for reuse. This fixed-size serialized diagnostic does not establish native-resize
savings, physical allocation counts or a pooling policy. It does not support
attachment caching as the primary correction for the observed 34% native cost.

## Swapchain depth isolation H4 — predefined diagnostic boundary

Measure an explicit diagnostic maximum-frame-latency hint of one versus the
unchanged default two through the common surface configuration helper. The local
wgpu28 surface contract describes one as appropriate for low-latency GUI work and
maps Vulkan image count to the hint plus one, subject to backend clamping; GL
ignores it. This is a hint, not proof of actual images/latency. Keep FIFO, native
extent, AA quality and rendering work unchanged. Record backend identity and hint,
compare same-binary CPU/GPU and successful frame counts with trace overhead
identified, and retain any failed or slower result. No default-policy change or
acceptance follows solely from the experiment. The prior coalescing-only CPU/GPU
regression remains rejected; do not relabel it as a performance improvement.

H3 follow-up isolates changing attachment extents: same-size replacement can hit
internal driver allocation caches and cannot rule out changing-size costs. Compare
1280x800/1288x808 alternating production renders with replacement versus a test-only
two-entry exact-size MSAA cache. Both conditions use the same two preallocated
single-sample targets and prepared scenes; pixel parity is checked per extent.
Bound the test cache to those two attachments, use the same ABBA/60Hz/CPU method,
and retain the serialized/offscreen limitation. This is evidence for selecting a
mechanism, not adoption of a production cache or evidence for arbitrary resizing.

## Owner-requested research reset and bounded recovery plan

The owner rejected the duration and lack of a resolved defect, and reiterated
that deep research or a concrete resolution plan was requested. Suspend the
expanding experiment sequence. GPS-C01R remains the active defect; no acceptance
or GPS-C02 advancement is implied. The interrupted allocation comparison was
not running when checked; no result is claimed for that interrupted run.

Research findings from primary references:

- wgpu documents that configure on an existing surface waits for GPU idle before
  swapchain recreation. This matches the local wgpu28 source and observed hot
  path, but elapsed waits must not be confused with CPU consumption.
  https://docs.rs/wgpu/latest/wgpu/struct.Surface.html#method.configure
- winit0.30.13 documents that pre_present_notify schedules Wayland frame callbacks;
  it does not implement equivalent X11 throttling. Native backend behavior is a
  necessary part of the scheduling design, not just an environment label.
  https://docs.rs/winit/0.30.13/winit/window/struct.Window.html#method.pre_present_notify
- egui keeps surface state per viewport and centralizes configuration, attachment
  lifetime and recovery. Its resize entry point still recreates attachments;
  this is not evidence that centralization alone eliminates resize cost. Its
  platform-specific macOS remedy is not a Linux remedy.
  https://raw.githubusercontent.com/emilk/egui/master/crates/egui-wgpu/src/winit.rs
- Iced centralizes window configuration/presentation and requests latency one.
  That setting is not a portable fix: the local Intel X11 capability query reports
  minimum image count three, while wgpu28 maps/clamps the latency hint to image
  count minus one. The measured one-versus-two result is therefore consistent
  with clamping, not evidence for spending more runs tuning the same hint.
  https://raw.githubusercontent.com/iced-rs/iced/master/wgpu/src/window/compositor.rs

These current upstream implementations are design references, not proposed new
dependencies or source to copy blindly across wgpu versions.

Bounded work and decision points:

1. Complete one focused reference review (60-minute active-work limit): trace
   upstream event-to-redraw scheduling, configure placement and presentation
   feedback against Datum's main and three owned hosts. Produce a concrete
   difference table with source links, exact Datum owners, a causal hypothesis
   for each proposed change, and explicit unsupported platform assumptions.
   Reuse the existing measurements. Do not reopen attachment/backend/latency
   sweeps without a newly evidenced reason.
2. Write the shared lifecycle implementation design before another release
   build: latest input extent, desired/configured/presented generations,
   drawable/zero-extent state, one pending redraw, frame-bound configuration,
   per-window damage, and bounded recovery/retry. Preserve final gesture state,
   AA/text quality and terminal semantics. The earlier coalescing-only candidate
   increased CPU/GPU; the design must explain what additional scheduling or
   resource change addresses that failure. Moving configure alone is not an
   accepted performance fix.
3. Implement one bounded candidate through the shared native-host owner, with
   main and all three dialog adapters using the same transition code. Add the
   affected state-transition and isolation regressions. Keep unrelated renderer
   features and broad cache/backend replacements out of this slice.
4. Use one predeclared baseline/candidate comparison after the focused tests:
   same release settings, actual backend, workload and delivered-frame accounting;
   report absolute CPU/GPU, frame intervals, final extent and visual defects.
   Retain a candidate only if responsiveness and resource use improve together.
   Approximately34% remains rejected. Capture must detect deliberately inserted
   blank/stale frames before it can support a no-flicker claim; the successful
   window recording is presently feasibility evidence only.
5. If that candidate fails, stop that candidate and return its falsified
   hypothesis and the specific dependency/platform question to resolve. Do not
   silently start another experiment tree. If evidence localizes a dependency
   defect, prepare a minimal reproducer and a concrete workaround/upstream-fix
   decision; no new dependency or upgrade is authorized by this plan alone.
6. Reconcile successful mechanisms and failed assumptions into E10/R39-R42,
   maintain HP01-HP25 allocation, obtain independent review, then qualify each
   host and resize lifecycle. Broader adoption/endurance remains final acceptance;
   completing this research/design packet does not close the defect.

This section is a working research/design packet. It does not assert that deep
research is complete, a candidate is proven, or production resizing is fixed.

### Focused scheduling review disposition

The focused source review does **not** substantiate proceeding directly to the
shared generation/pacing candidate proposed above. Steps 2–3 are conditional on
resolving that causal gap; they are not an instruction to repeat the rejected
coalescing change in a larger abstraction. No new build or measurement was run
for this review.

| Observation | Evidence | Engineering consequence |
| --- | --- | --- |
| Changed main-window sizes configure synchronously before redraw requests coalesce. | `runtime_surface.rs::apply_resize`, `main.rs` resize dispatch | Intermediate extent configuration is a confirmed expensive path; moving it alone has already failed the resource comparison. |
| Main redraw requests already coalesce. | `app_shell.rs::request_redraw_if_needed` sets `redraw_pending`; `app_frame.rs` clears it on redraw. | Adding another pending boolean is not a new performance mechanism. Native redraw events still need correct servicing. |
| Main-window requests invalidate all three open dialog hosts. | `app_shell.rs::request_redraw_if_needed` | Separate host-local damage from deliberate application-wide invalidation. This establishes isolation correctness, not a main-only CPU improvement. |
| Acquisition failure leaves content invalidated without an explicit retry request. | `runtime_present.rs::render`; background polling requests redraw only when its own work changes. | A shared retry transition must preserve pending damage and schedule bounded recovery. This is a possible stale-frame mechanism, not proof of the reported flicker cause. |
| Background polling owns the event-loop wait deadline. | `production_status_refresh.rs::poll_background_work` | Any later presentation/retry deadline must join the existing minimum-deadline calculation; setting a separate deadline can be overwritten. |
| Ordinary background polling uses `Wait` or `WaitUntil`. | Same function and main initialization. | Source review does not establish an idle busy loop as the resize cause. |
| Main presentation already calls `pre_present_notify`. | `runtime_present.rs`, immediately before `frame.present()` | Do not propose adding the existing call as a fix. Wayland callback behavior must not be assumed on X11. |
| The coalescing candidates lack frame accounting. | Archived `candidate-plain-idle` and `candidate-reuse-plain` reports and both compressed logs. | Neither increased useful frames nor runaway presentation is established. Configuration counts are not delivered-frame counts. |

The archived candidate diagnostic logs contain 1,939 and 2,401 configure-begin
records respectively, but no render/presentation completion records; their
`gui.log.gz` files supply no submission records. These are whole-log totals,
including setup, not comparable timed rates. The `final-traced` GUI log contains
presentation records, but that cannot reconstruct the missing candidate data.

Prior-art comparison: Iced's window event loop handles resize by requesting a
redraw, checks the latest physical extent and surface version during redraw, and
skips zero extents. Its per-window deadlines participate in event-loop waiting.
Datum currently clamps zero extents and configures during resize dispatch.
These are useful lifecycle design differences, not evidence that copying Iced's
scheduling will meet Datum's resource target. egui's immediate resize allocation
path also rules out claiming a universal framework practice of deferring every
configuration or pooling attachments.

Reference for the Iced event-loop comparison:
https://raw.githubusercontent.com/iced-rs/iced/master/winit/src/lib.rs

Independent read-only review reached the same disposition: host-local damage
and bounded failed-frame retry have direct source justification; a new pacing
policy, attachment cache or lifecycle rewrite is not yet demonstrated to remedy
the rejected main-only CPU/GPU result.

Decision: retain the existing measured geometry reuse and reject backend,
latency-hint and unproven pacing changes as a claimed resolution. Before selecting
the main-window performance candidate, resolve the specific missing distinction
between expensive swapchain turnover and expensive delivered frames. A bounded
native minimal-reproducer comparison, if needed, must preserve the actual
backend/extent workload and count configurations, successful submissions and
presentation intervals together. It must distinguish a clear-only native frame
from Datum scene rendering without changing the scheduling between conditions.
This would locate the remaining cost at the presentation boundary or in scene
work; it would not supply visual acceptance or authorize an experiment sweep.
No such additional experiment has started. The global isolation and retry
corrections remain necessary but cannot substitute for resolving main-only CPU
consumption and ordinary resize flicker.

### Native-frame isolation result

The bounded comparison above has now run once, in full/clear/clear/full order.
An opt-in `DATUM_DIAGNOSTIC_NATIVE_FRAME=clear` path bypasses preparation and
product rendering after ordinary surface acquisition, submits a clear pass and
uses the same native presentation helper as the full scene. The default/full
path preserves product rendering. No pacing or configuration policy changed.
Independent source review found no blocker for this no-probe diagnostic.

One guarded release build passed in 7m00s. Source health passed for 2,031 files.
The measurement began only after the build exited and no rustc process remained.
All four runs used binary SHA256
`22ebe5dcda405b73fa89aee9e9836bb79a87b71ebe9deca2a065667c74cfb498`,
explicit Vulkan/X11 on the existing desktop, FIFO/default latency two, the same
board and 5-second samples for each axis. Timing trace was enabled for both
conditions; verbose logging and CPU probes were disabled. Full rendering retains
its normal MSAA; clear-only deliberately bypasses MSAA and all product drawing.

| Condition | CPU, one-core percent | GPU render-engine busy percent | Completed presentation calls per 5s | Configurations per 5s |
| --- | --- | --- | --- | --- |
| Full scene, both runs/axes | 34.60–35.20 | 16.81–17.23 | 110–114 | 279–280 |
| Clear-only, both runs/axes | 39.20–40.60 | 2.90–3.25 | 231–250 | 280–281 |

All samples recorded zero observed XI keyboard/button/motion events; idle samples
recorded 0–0.2% process CPU and no presentation calls/configurations. Timed resize
samples recorded no acquisition recovery or timeout. Reports, raw counters,
compressed logs, exact harness scripts and hashes are archived in
`measurements/resize-native-frame-isolation/`; `summary.json` provides each axis
and run separately.

This changes the diagnostic decision: product scene rendering is not necessary
to reproduce high CPU during native resizing. Removing it reduced GPU activity
but increased presentation throughput and left CPU high. Native resize,
configuration, acquisition/submission/presentation, driver work and remaining
application handling must therefore remain the primary investigation boundary.
Further text/geometry/attachment tuning alone is not a supported resolution.
This is new evidence that throughput can change materially under identical
event-loop code, not retrospective proof of why the old coalescing candidate
regressed. It does not establish a specific driver defect.

Do not subtract the two CPU values to assign scene cost: their actual frame
counts differ. Completed present calls are not compositor display acknowledgements
or measured frame intervals. The clear run's `renderer=0ms` is a bypass marker,
not zero submission/GPU work. This remains a Datum-hosted diagnostic, not an
independent minimal wgpu reproducer. Neither clear frames nor settled captures
prove ordinary-resize flicker absent. All four runs completed and the harness
exited; no follow-on experiment was automatically started. GPS-C01R remains open.

### Follow-through: repeated native device discovery

Source review after the native-frame comparison identifies a concrete repeated
operation in the measured X11/Vulkan stack. This is not a new runtime experiment.

The existing H2 `calltree.txt` attributes 0.515 of its 2.825 sampled CPU seconds
to `drmGetDevice2` beneath `NativeSurface::create_swapchain`, the Vulkan loader
and Intel driver. Its descendants include device enumeration, `realpath` and
`readlink`. Those samples are inside creation, not surface-capability queries;
do not attribute that hotspot to the latter merely because wgpu queries them on
every configure. The profile's sampling/coverage limitations recorded in H2
still apply; 18.23% is a fraction of sampled CPU, not a process percentage point.

Mesa's upstream 25.0.7 X11 WSI source, matching the installed base version,
calls `wsi_x11_check_dri3_compatible` from `x11_surface_create_swapchain` when
constructing the DRM image parameters. That check opens the X server's DRI3
device, calls the driver's `can_present_on_device` callback, and closes the
descriptor. The common DRM helpers contain `drmGetDevice2` calls for device
matching. The stripped profile does not resolve the exact Intel callback, so
the callback-to-helper edge is not asserted as a fully symbolized stack.

Primary source references:

- https://gitlab.freedesktop.org/mesa/mesa/-/blob/mesa-25.0.7/src/vulkan/wsi/wsi_common_x11.c
- https://gitlab.freedesktop.org/mesa/mesa/-/blob/mesa-25.0.7/src/vulkan/wsi/wsi_common_drm.c
- https://github.com/gfx-rs/wgpu/blob/v28.0.0/wgpu-hal/src/vulkan/swapchain/native.rs

The inspected current Mesa main sources retain this compatibility check during
swapchain creation and the common DRM device-discovery helpers. The inspected
current wgpu trunk also retains surface support/capability queries and swapchain
recreation. This provides no evidence for recommending a blind dependency or
driver upgrade. No dependency, driver, system setting or external code was
installed or modified during this review.

Engineering consequence: the shared lifecycle must account for actual swapchain
turnover and successful submissions together; reducing scene complexity alone
leaves repeated native device-discovery work reachable. An application-side
capability cache would not eliminate the driver work inside swapchain creation.
Any proposal to cache or bypass device compatibility in the driver must address
hybrid-GPU identity and device/display changes and requires separate evidence;
it is not a safe application shortcut. Likewise, lowering AA or freezing the
scene during resize does not address this repeated operation.

Keep this finding explicitly scoped to the measured X11/Xwayland Vulkan stack.
It cannot explain native Wayland flicker without corresponding evidence. The
required global adoption remains one shared lifecycle with explicit backend
adapters, not forcing every host onto the measured backend or treating a
platform-specific workaround as program-wide qualification.

### QA launch-path coverage correction

The current desktop exports both `DISPLAY=:1` and
`WAYLAND_DISPLAY=wayland-0`, with `XDG_SESSION_TYPE=wayland`. The inspected
`scripts/run_gui_doa2526.sh` launch command preserves that environment. The
installed winit 0.30.13 Linux event-loop selector prefers Wayland when its display
variable is present unless the builder explicitly forces another backend.
These observations support a normal-launch Wayland expectation, not proof of
the owner's earlier QA backend. The older default diagnostic log lacks a
surface-identity record and cannot resolve that uncertainty retrospectively.

The detailed resize CPU comparisons deliberately removed `WAYLAND_DISPLAY` and
identified X11 at runtime. They reproduce a real X11 defect, but cannot serve as
the normal-launch baseline by assumption. Existing H2 native Wayland Vulkan/GL
pilot reports explicitly qualify only targeted compositor-driven native resize
delivery; they do not contain fixed-window process CPU/GPU samples or validated
temporal acceptance. Their aggregate phase timings do not fill that gap.

Consequently, a proposed correction must be evaluated on the actual QA launch
backend and the normal desktop backend, retaining X11 coverage as a separate
supported-host case. Confirm the owner's launch command or capture identity from
a fresh normal launch before treating an X11-only driver finding as the cause of
the owner-reported defect. Do not change backend selection merely to make a
benchmark easier to drive. The existing PID-targeted KWin resize pilot can supply
native Wayland event injection; its missing resource/input/temporal accounting
must be explicit when extending it. This correction adds no new experiment and
does not invalidate the separately identified X11 measurements.

Owner follow-up supplied the QA command:
`cargo run -p datum-gui-app --bin datum-gui -- --board ~/Documents/kicad_projects/DOA2526/hardware/DOA2526/DOA2526.kicad_pcb`.
It contains no backend override, supporting the Wayland launch expectation on
this desktop. It also omits `--release`, so that exact command selects Cargo's
development profile; preserve the earlier optimized CPU reproductions rather
than dismissing the defect as debug-only. The owner additionally reports flicker
on almost every observed agent launch/test run. Those observations remain open
visual failures, not superseded by successful harness execution or CPU receipts.

### Native Wayland accounting on the default selection path

After the owner identified the launch command, the existing PID-targeted KWin
pilot was extended with fixed-window process CPU, DRM engine counters, native
resize counts, completed presentation calls and input-event counts. Independent
review required removal of forced GPU selection and inherited preferences
overrides, and observed completion/restoration rather than trusting a timer.
The corrected harness passed review before execution. It ran the existing
optimized binary; no new build or backend setting change was needed.

Runtime identity was Wayland/Vulkan on Intel P630/Mesa 25.0.7-2+deb13u1. The
binary hash remained
`22ebe5dcda405b73fa89aee9e9836bb79a87b71ebe9deca2a065667c74cfb498`.
The original physical extent was 1344x840. A two-axis size sentinel followed by
restoration was observed through the application's native resize log before
advancing to the next axis. Each approximately5.5-second sample includes script
dispatch, the resize sequence, sentinel/restoration and settling tail.

| Phase | GUI CPU, one-core percent | GPU render busy percent | Configurations | Completed presentation calls |
| --- | --- | --- | --- | --- |
| Initial idle | 0.00 | 0.00 | 0 | 0 |
| Vertical resize | 33.45 | 19.08 | 239 | 136 |
| Horizontal resize | 34.90 | 18.67 | 236 | 117 |
| Final idle | 0.00 | 0.00 | 0 | 0 |

No observed keyboard, button, wheel, pointer-motion, IME or modifier events;
no acquisition timeout/recovery; no lost DRM clients. All phases completed and
the owned script/application were cleaned up. Reports, counters, exact QML,
harness and compressed logs are in `measurements/resize-wayland-accounting/`.

This confirms high optimized-process CPU during native Wayland resize as well
as the separately measured X11 case. An X11-only device-discovery workaround
cannot be selected as the global resolution. The app still configures more
often than it completes presentation calls on both paths. That observation
supports eliminating superseded configuration work in the shared lifecycle,
but the earlier coalescing regression still requires frame/resource accounting
and cannot be declared fixed by moving a call.

Verbose and timing diagnostics were enabled to observe native events. This is
not uninstrumented acceptance, a matched X11/Wayland comparison, physical border
dragging, displayed-frame accounting or temporal flicker proof. The owner's
repeated visual failure reports remain outstanding. The exact supplied Cargo
command uses the development profile; the optimized reproduction means profile
selection does not dispose of this defect. GPS-C01R remains active.

### Implemented main-resize dialog isolation

The confirmed E01/E10 isolation defect now has a bounded Rust correction.
`App::request_main_redraw_if_needed` contains the existing main-window pending
redraw check. Main `WindowEvent::Resized` calls that local helper. The existing
broad request method calls the same helper and preserves its prior dialog
invalidation for other callers. All three owned-dialog resize handlers already
request only their own native window. Surface configuration timing, geometry,
text, AA, input/modal policy and backend selection are unchanged by this fix.

The optimized build passed in19.48s; exact-file formatting and source-health
checks passed. Independent source review accepted the correction. Before native
proof, independent harness review required actual dialog presence throughout
resize and settled counting boundaries, resolving those false-pass risks.

The previous binary is a negative control: with Global Preferences open, main
vertical/horizontal resizing caused126/130 extra dialog renderer invocations.
The candidate produced **zero extra renderer invocations** on both axes with
Global Preferences, Project Preferences and New Project individually open.
Main presentation calls continued:148/139,161/145 and124/131 respectively.
All native completion/restoration sequences were observed, counting boundaries
were settled, and no acquisition timeout/recovery occurred. Each test verified
two distinct native surfaces and checked the expected dialog's presence on every
compositor timer tick. The four runs completed and cleaned up their owned GUI
processes/scripts. Exact source patch, reports, counters, scripts, logs and hashes
are archived in `measurements/resize-dialog-isolation/`.

This proof covers induced renderer work during programmatic main resize with
each actual owned dialog present. Main post-dispatch input counters cannot
observe dialog-consumed or modal-filtered input; do not label this all-window
input observation. Logging was enabled, and unequal delivered-frame counts
prevent treating these short CPU results as a qualified performance comparison.
Candidate resize samples still used roughly32–35.5% of one CPU core. This fixes
unnecessary cross-window work; it does **not** resolve main-only CPU or flicker,
retire every broad redraw caller, implement the complete shared lifecycle, or
close GPS-C01R. No new production dependency was introduced.

### Shared resize transaction candidate and comparison boundary

The owner confirmed that the visible failure is content blinking/jumping inside
the window. Keep that specific symptom in temporal qualification; checking only
newly exposed edges or final window size is insufficient.

An opt-in candidate, `DATUM_DIAGNOSTIC_RESIZE_TRANSACTION=1`, now shares
`SurfaceTransaction` between main Runtime and all three dialog instances.
Latest size/input state updates immediately; surface configuration moves to
the next drawable frame attempt. Before that attempt, the host checks completion
of its preceding queue submission, using a short callback retaining only an
atomic completion flag. A nonblocking device poll services completion; an
unfinished attempt defers to an event-loop retry deadline. The existing
background deadline calculation takes the minimum across all hosts and its
other tasks. No fixed frame-rate cap, input loss or quality reduction is added.

Acquisition failure keeps damage and, when required, invalidates configured
extent. Retries back off from8ms to250ms and terminate through the existing GUI
error path after eight failed acquisitions. A GPU completion episode terminates
after1,024 unsuccessful retry attempts. Limits count actual attempts rather than
elapsed wall time; delayed compositor callbacks alone do not exhaust them.
Raw native zero extent suppresses rendering and cancels pending retries. The
existing one-time clamped initial configuration remains; no1x1 candidate frame
is rendered while native extent is zero. Closure drops host state; callbacks
cannot request work for a closed window. Independent review accepted the fixes
to initially unbounded retries and initially lost zero extent.

This differs from moving configure alone by explicitly retaining damage while
GPU ownership is pending and providing bounded recovery scheduling. It is still
a hypothesis, not an explanation of the previous coalescing regression. Hosts
share a queue: a callback covers preceding queue work, and configure may still
wait on another host's later submission. Do not claim fully nonblocking or
independent per-host GPU queues. Disabled mode retains the prior policy with
additional bookkeeping; native dialog configuration now also emits diagnostic
markers. Compare modes in the same new release binary.

Predeclared comparison: run the reviewed native Wayland accounting harness once
with flag0 and once with flag1, both axes and idle phases, same binary/fixture/
logging. Compare GUI CPU, surviving-client GPU busy counters, configurations and
completed presentation calls together. Reject resource regression or reduced
delivered work; approximately32–34% remains unacceptable for final qualification.
If the primary Wayland comparison fails, stop this candidate before expanding
to X11 or host/endurance proof and report the failed hypothesis. If it improves
resource use and delivery together, extend that same candidate to X11 and all
hosts and validate temporal content stability before any default-policy change.
The existing recording has no validated blink/jump oracle, so this accounting
comparison cannot alone authorize adoption or closure.

### Independent native-loop isolation after owner-requested change of direction

The owner stopped the repeated application-candidate cycle. The transaction
candidate remains opt-in and unqualified; its proposed comparison did not run.
`native_resize_repro.rs`, selected before application startup by
`DATUM_DIAGNOSTIC_MINIMAL_WINDOW=1`, now provides a separate winit event loop
and a fixed physical-pixel grid. It loads no board, model, layout, text,
terminal or Datum renderer. It retains the existing adapter/device requests
and surface defaults; configuration follows the latest extent at redraw.
Acquisition errors end this diagnostic rather than hiding recovery churn.

The optimized native Wayland run reproduced 31.65% CPU vertically and 30.33%
horizontally, with 2.79%/3.09% GPU render-engine busy time. Both axes completed
148 configuration/presentation calls; native resize event counts were209/222.
Both idle samples were zero CPU/GPU. No observed application input, acquisition
errors or lost GPU clients occurred. Receipts and exact reproducer source are
in `measurements/resize-minimal-native/`. The inherited harness's `mode=full`
and board hash do not mean a fixture was loaded; the context receipt explicitly
corrects that interpretation. These logged, compositor-script runs are not
physical-drag, uninstrumented or temporal visual acceptance.

This establishes that high resize CPU does not require Datum editor rendering.
It does not establish a driver defect, a universal lower bound, or the cause of
content blinking/jumping. Scheduling also differs from the application, so
differences cannot be attributed solely to removing scene work. Investigation
now belongs in the isolated native path before further editor-level tuning.
Any resulting correction must still return through the shared R39-R42/E10
host lifecycle and retain all HP01-HP25 and temporal-content obligations.

One resize-only gprofng capture reproduced the CPU load but represented only
0.170s of3.300s process-accounted CPU in sampled stacks, with a SIGPROF collector
warning. Its call-tree percentages are rejected as a dominant-cost ranking;
the missing CPU cannot be assigned to a driver or application function.
The raw accounting, stack report and limitation are preserved in the `profile/`
subdirectory. No speculative renderer correction or acceptance follows from
that incomplete profile. Source health, dependency authority, diff checks and
the guarded optimized build passed; independent source review accepted this
as a diagnostic isolation tool only. GPS-C01R remains active and unresolved.

### Direct CPU accounting identifies the submission boundary

After the owner confirmed an idle desktop, the independently reviewed checked
harness completed both axes with five sequential CPU-clock windows. The first
attempt, stopped before resizing by54cursor movements, is preserved separately.
The successful run has zero observed input and idle CPU, matched phase/frame
counts, no acquisition errors and no lost GPU clients. Its direct accounting
represents89–90% of process CPU, unlike the rejected sampled profile.

Each axis consumed1.630s process CPU. Submission accounted for1.020s/1.008s
(62.6%/61.8%), configuration0.377s/0.380s(23.1%/23.3%). Submission averaged
6.80ms/6.72ms CPU per call, even for the independent single-triangle grid.
Other measured calls were small; unmeasured event processing, logging, destruction
and gaps remain. Process CPU includes concurrent workers; thread CPU is a subset,
not an additional quantity. Receipts: `resize-minimal-native/cpu-phases/`.

A local diagnostic ioctl interposer then identified request0x40406469,
`DRM_IOCTL_I915_GEM_EXECBUFFER2`, inside submission windows. Across the whole
run, including startup,303matched calls consumed2.050s thread CPU and returned
success. Exact logs, source, request mapping and timestamp/observer limitations
are in `resize-minimal-native/ioctl/`. This points below Datum scene drawing to
the Intel driver/kernel submission boundary; it does not identify the expensive
internal kernel function, prove a driver bug, or explain content blinking.

Further application tuning is paused pending targeted kernel-stack attribution
or upstream review of this minimal reproducer. Existing tracefs access is denied,
including outside the sandbox; noninteractive sudo requires local owner
authentication. No system settings, packages or driver versions were changed.
No speculative default-policy switch, renderer replacement, CPU acceptance,
temporal acceptance or GPS-C01R closure follows from these findings.

### Kernel-capture failure and specification resumption

The targeted kernel helper did not produce a usable capture. Its initial
per-instance graph-control assumption was false on this kernel. The revised
large function-filter setup consumed a core while still in setup, before the
advertised ten-second capture interval began. The owner stopped that process;
subsequent owner terminal output showed no process at PID376388 and an empty
`/sys/kernel/tracing/instances` directory. The capture JSON was empty. The
helper is disabled, with the failed source retained as diagnostic history under
`measurements/resize-minimal-native/`; do not recommend rerunning it. No kernel
hotspot or renderer acceptance can be inferred from the tracing-induced spike.

The owner subsequently directed completion of GPS-C02 through GPS-C06. The
canonical transaction records that exact direction in
`specification-resumption-owner-direction.json`; GPS-C01R is pending and
unresolved after that specification decision. The independent minimal-window
submission/configuration evidence informs the common engineering contract, but
is not proof that KDE, wgpu or the kernel alone caused the defect. The opt-in
transaction and diagnostic Rust changes remain unaccepted worktree candidates.
No additional kernel or GUI experiment is part of this specification-resumption
change. Final resize resource and continuous-content proof remain required.
