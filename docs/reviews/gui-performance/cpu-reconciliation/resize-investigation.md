# Current-host horizontal and vertical resize CPU investigation

<!-- EVIDENCE:GUI-PERFORMANCE-IMPLEMENTATION:CPU-RESIZE-PLAN -->
Status: concrete execution proposal, not execution authorization. Exact owner
reset: `owner-direction.json` → `resize_reset`. Tracking:
`dat-gui-vertical-resize-cpu-toj`, related implementation task
`dat-gui-performance-implementation-vkq`. This is one investigation of the
owner-reported resize cost, not a performance qualification program.

## Objective and authority

Determine whether substantial CPU consumption during horizontal and vertical
window resizing persists on the owner's current host and current R4; if it
does, locate the current cost and identify a cause only as far as the evidence
supports. Preserve correct, responsive interaction, authored state, quality,
shared rendering ownership and normal engine mutation authority.

Numerical acceptance budgets remain undecided for this investigation. The old
CPU duty/tail figures, withdrawn GPU figures and rejected historical percentages
are neither entry gates nor pass/fail targets here. No replacement number is
derived or ratified. Formal S5 obligations and PM051's non-withdrawn requirements
remain separate; this proposal does not waive them. S4 stays closed.

The completed CPU audit and derivation assessment are reused. Their postponed
six-launch pointer/control recommendation is superseded. The subsequently
approved single ordinary idle/pointer observation is preserved in
`ordinary-observation.json`; it supplies no resize finding, causal overhead
claim or reason to schedule another pointer test. Broader hardware, pan/zoom,
concurrent workloads, GPU attribution, renderer redesign and optimization are
outside this packet.

## Historical evidence and current source

| Existing evidence | Credit and limitation |
|---|---|
| `../resize-slice.md` | Geometry reuse reduced logging-disabled median CPU from about 40% to about 34–35% of one core in each axis at the recorded candidate. The owner rejected the residual cost. Owner-reported >72% peak has a different, unspecified monitor boundary. Neither establishes current R4 behavior. |
| `../measurements/resize-recovery-h2/summary.json` | Same-binary ordinary five-second height/width trials measured 34.799%/33.799%; diagnostic phase clocks exposed configuration and rendering cost. Asynchronous/process-clock overlap and diagnostic effects prohibit exact causal partition. |
| `../measurements/resize-native-frame-isolation/summary.json` and `../resize-recovery.md` | Full rendering measured 34.60–35.20%, clear-only 39.20–40.60%, with different throughput. Product scene rendering was unnecessary for that historical reproduction; subtracting conditions cannot isolate scene CPU. |
| `../measurements/resize-minimal-native/diagnostic-context.json`, `cpu-phases/accounting-summary.json`, `ioctl/attribution.json` | A native minimal loop reproduced cost without the board; sequential phase windows placed much CPU in submission. Whole-run ioctl correlation identified i915 EXECBUFFER2 at its recorded boundary, not an internal kernel cause or current R4 attribution. |
| `../measurements/resize-minimal-native/profile/qualification.json` | Sampled stacks covered only about 5% of accounted CPU and attribution was rejected. Do not repeat that collector or rank a dominant cause from its incomplete stacks. Kernel-capture permission failure is a method blocker, not hardware infeasibility. |
| `../resize-recovery.md` normal-launch coverage correction | Historical detailed X11 findings do not automatically explain normal Wayland launch behavior. Native Wayland geometry-driven records also exist, but their timed window includes script/restoration overhead and is not physical border-drag qualification. |

Source reviewed at the reset base, GUI tree still matching corrected R4
`c3949e53`: `app_native_events.rs::dispatch_native_window_event` forwards native
sizes to `Runtime::resize`; `native_frame_coordinator.rs::window_event` owns
extent damage; `runtime_surface.rs::apply_resize` updates logical dimensions and
terminal dimensions when applicable; `interaction_refresh.rs::invalidate_surface_size`
calls shared `resize_content`. Size-independent world geometry remains retained.
`native_surface_transaction.rs::begin_frame` admits configuration at the frame
boundary only when the configured extent differs; `configure` calls
`surface.configure`. `runtime_present.rs::render` then handles surface attachments,
preparation, rendering/acquisition and presentation through the shared owners.

These differ from the old immediate-configure/invalidate-all path. Do not
rediscover frame-bound configuration or geometry retention as missing fixes.
Remaining candidate costs are changed-extent surface/driver work, attachment and
screen preparation, renderer submission, or other event/background work. Source
reachability and historical hotspots do not rank their current contributions.

## One bounded execution packet for owner approval

Maximum two GUI launches, no rebuild or product edit. At most two hours of active
harness preparation and analysis, and 180 seconds per launch. Execution starts
only after approval of this packet. Routine pre-measurement harness corrections
within this scope are recorded; measured failures are never replaced.

### Preflight and common method

Use the existing release R4 binary SHA256
`5c1c8f35746b55397f95280799be3b7bc4f28984b76f94514aaac6ef4933816b`
and the authored F-DOA project pinned by
`../baseline-budget/pointer-declaration.json`. Invoke the binary directly;
the owner-local launcher normally runs Cargo and is not an execution method
for this no-build packet. Pin CLI/fixture bytes, kernel, CPU, Mesa, desktop and
source identities again; do not rebuild or substitute a candidate if absent.

Preserve the current desktop's normal backend selection rather than forcing
X11 to reuse pointer tooling. Expected normal route is Wayland on this KDE
session; verify the actual native handle, adapter and surface configuration
from startup. Use a single 1280×800/scale-1 board host, unchanged FIFO/exact8x
quality and isolated config/cache. Do not open auxiliary windows or terminal
workloads. Retain the integrated shell's CPU in application accounting.

Reuse the PID-targeted KWin declarative geometry method already archived under
`../measurements/resize-minimal-native/harness.py`. Adapt only its controller,
project launch and accounting: omit historical clear/minimal modes, verbose
tracing, inherited unrelated flags and debug CLI; do not copy its mixed
restoration interval or mislabeled fixture fields. Prepare and validate QML
without running it before launch. Require exactly one unmaximized target with
the launched PID, accessible KWin scripting and a supported existing compositor
capture tool. Refuse ambiguous window selection or unavailable capture/accounting.

Start the GUI inside a fresh owned cgroup before exec. Record cumulative
`cpu.stat` total/user/system, monotonic read brackets, PID identities/membership
and final counters before group removal. Cumulative totals retain exited
descendants; external controller/capture and compositor CPU are excluded.
Record KWin cumulative process CPU separately over the same windows when
accessible; do not add all desktop CPU to Datum or claim subtraction isolates
driver cost. Identify any cooperating application process outside the group;
unaccounted application work makes the result incomplete. Use existing ordinary
startup/resize logs, not optional semantic receipts, GPU queries or resource
traces. No system tuning, new dependencies, kernel access changes or profiler
installation.

### Launch 1: ordinary reproduction

After readiness/output inspection and five seconds of no-input preparation,
record a ten-second quiet control. Then request width-only resizing for ten
seconds and height-only resizing for ten seconds, with restoration and five
seconds of settling outside each active window. For each axis use 600 planned
geometry updates at 60/s, triangular deltas 0…180…0 pixels over two-second cycles;
hold the other client dimension constant. Actual KWin timer delivery is not
assumed to be 60/s: retain emitted update counts and actual phase start/end,
geometry changes and completion. After restoration, record a ten-second quiet
tail. One interval per axis; no repetitions.

Application CPU and KWin CPU use actual elapsed denominators. Retain a 100ms
external cgroup counter series to distinguish sustained cost from short spikes;
report the actual sample widths and read uncertainty with any peak. Do not
compare such peaks directly with the owner's unspecified monitor peak. Keep
script dispatch, restoration, screenshots and settling outside active intervals.
If exact timed boundary delivery is unavailable, report the actual enclosed
window and included work rather than claiming an exact ten seconds of resize.

After each active axis, request and hold a known +180-pixel extent for its
changed-size capture, then restore; these endpoint checks are outside CPU windows.
Verify changed native extents in the existing ordinary `resize apply` records,
agreement with KWin client geometry, final restoration and unchanged fixture.
Capture/inspect readiness, changed-width, changed-height and restored output
outside timed windows; verify that each changed-size image is actually changed
and usable, not stale capture. Preserve board appearance, layout and endpoint
focus. This is a useful reproduction/output check, not continuous no-flicker,
input latency, hit-testing or full PTY acceptance. KWin geometry injection is
native-window resize delivery, not physical decoration dragging. Non-reproduction
does not refute a drag-specific owner observation.

Decision: report each axis's ordinary CPU, user/system split, peak windows, actual
geometry rate, quiet controls and output checks. A substantial resize-correlated
cost resembling the reported concern enables only the predeclared diagnostic
launch below. Small or ambiguous cost ends with scoped non-reproduction or an
inconclusive result; no numeric acceptance cutoff is invented to select it.

### Launch 2: conditional current-cost localization

Only if launch 1 validly reproduces the concern, use the identical binary,
backend/fixture, geometry recipe and quiet controls, repeating each affected axis
once. Enable the existing `DATUM_RESIZE_CPU_PROBE=1` process/thread CPU clocks.
Their current labels cover configure, acquire, prepare, renderer, present,
native events and event rounds. No new probes, collector, clear/minimal bypass,
GPU queries or additional workload is introduced. Record total CPU independently.

For every phase retain call count, process CPU, calling-thread CPU and wall time;
join complete probe records to the active geometry interval and identify
boundary-crossing calls. Pair realtime and monotonic clocks at both external
boundaries to map the existing timestamped probe end and recorded wall duration;
retain read uncertainty. A clock step or ambiguous join leaves attribution
incomplete rather than assigning boundary work to an axis. Nested
event-round/renderer/prepare/acquire windows and
asynchronous worker work overlap: do not sum them into an exact CPU partition.
Uncovered time stays explicit. Diagnostics-on totals are not ordinary performance
and this pair is not an observer-overhead experiment. Count configure and present
phase records for context, never displayed FPS or independently qualified
successful presentation. Retain any failed-call/error context.

Decision: distinguish a cost localized to surface configuration, preparation,
renderer/submission-containing work, presentation or outside observed phases.
CPU clocks distinguish consumed CPU from elapsed waits. Correlate the current
location with the reviewed call path and historical findings; name a concrete
cause only if that link is supported. For example, CPU in configure is not itself
proof of an Intel kernel defect, and CPU in the renderer window does not uniquely
identify queue submission. If a driver/kernel interior, worker attribution or
instrumentation boundary remains unresolved, state that exact gap and stop.
No attempt to obtain privileged kernel tracing or redo the undercovered profile
is included.

## Stop, report and roadmap handoff

Stop an affected observation on failed geometry delivery, unusable/stale output,
focus/fixture corruption, early exit, incomplete CPU accounting, deadline or
evidence-storage failure. Preserve attempted intervals and diagnostics; no retry,
replacement axis, backend sweep or automatic optimization. If ordinary reproduction
fails, the diagnostic launch is unused. Inconclusive attribution is an allowed
final outcome, not permission for another experiment.

Report one packet with pins, commands/environment, actual CPU and elapsed,
user/system and separate compositor accounting, source interpretation, output
checks and limitations. End with non-reproduction, a supported current cause,
or a specific unresolved cause/method gap and the owner's decision it enables.
No optimization or defect closure follows from a historical/unsupported cap.

<!-- OWNER:GUI-PERFORMANCE-IMPLEMENTATION:GPI-CPU-RESIZE-SCOPE:RESIZE -->
Owner decision requested: approve this exact two-launch maximum reproduction and
conditional existing-probe diagnosis, revise a named boundary, or defer. Approval
would permit only harness adaptation and these observations, not production
changes, rebuilds, tuning, dependency adoption, numerical ratification, S5
qualification or reopening S4. The original resize issue stays open; completed
pointer/audit/derivation evidence is retained and never rescheduled as a prerequisite.
