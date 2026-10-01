# Current-host resize repair

<!-- REQ:GUI-RESIZE-REPAIR:RZ-C01 -->
<!-- REQ:GUI-RESIZE-REPAIR:RZ-C02 -->
<!-- REQ:GUI-RESIZE-REPAIR:RZ-C03 -->
<!-- EVIDENCE:GUI-RESIZE-REPAIR:OWNER-GOAL -->

The owner's explicit continuing repair goal is recorded verbatim in
`owner-direction.json` (`resize_repair_goal`). This grants narrow execution,
including a demonstrated code correction and necessary builds/verification.
It does not set an acceptable CPU percentage, authorize dependency or driver
changes, or reopen S4 or complete S5. Earlier bounded failures remain evidence.
The original diagnostic proposal remains historical; this separate repair lane
prevents the selector redirecting work to its superseded proposal boundary.

## Existing diagnosis and bounded first correction

Reuse the completed audit/derivation and current R4 evidence under `target/`:
`cpu-resize-diagnostic-20260930-measured`, `cpu-resize-ioctl-20260930`,
`cpu-resize-kernel-indexed-20260930`,
`cpu-resize-page-acquisition-v2-20260930`, and
`cpu-resize-resource-correlation-20260930`. Ordinary resize costs roughly
40–44% of one core at the measured cadence. Kernel captures lost events; only
locally intact episodes support attribution. Surviving episodes demonstrate
new shmem backing-page allocation; their sizes match tiled presentation-image
sizes. Driver source supports that explanation, without individual image/GEM
identity or complete CPU partition. No further privileged tracing is required
for the next decision.

The attempted full GL backend comparison under
`cpu-resize-backend-feasibility-20260930-gl` produced no visible content.
The owner confirmed there was no window. Its CPU figures cannot demonstrate
savings or equivalent required work. The method failed to inspect readiness
pixels before measured intervals; all intervals are preserved without replacement.

Two subsequent no-resize readiness launches isolate the output failure:
`cpu-resize-gl-clear-readiness-20260930` visibly presents the clear color;
`cpu-resize-gl-full-errors-20260930` reports
`GL_INVALID_ENUM in glTexStorage2DMultisample(target=GL_TEXTURE_2D)` followed by
incomplete-framebuffer draw/resolve errors. Shared attachment creation requests
optional COPY_DST solely from size/format/sample eligibility, even when actual
restoration admission was refused. In the cached wgpu GLES implementation this
usage selects a multisample texture rather than a renderbuffer. Gate optional
copy usage on restoration admission; preserve the existing full-render fallback,
exact8x quality, lifetime owners and Vulkan admitted path. This is an existing
capability-fallback correction, not a new rendering mechanism or a dependency patch.

RZ-C01: Make that correction, build one optimized candidate through the guarded
runner, and verify a visible full GL board before any CPU interval. Run focused
attachment tests to preserve capability refusal and admitted-path behavior.

RZ-C02: If visible output works, one launch per GL/Vulkan backend of that same
candidate, one ten-second horizontal and one ten-second vertical interval each.
Use the existing PID-targeted KWin controller, same 1280x800 logical dimensions,
actual 1344x840 extent/1.05 desktop scale, FIFO/exact8x, fixture, 17ms triangular
+180 logical cadence and cgroup boundaries. Inspect readiness, changed-size and
restored captures; require nonempty usable content before timing. No measured
failure is replaced; no profiling, tuning or parameter sweep. Record actual
CPU/user/system/elapsed, delivered API counts and complete descendant accounting.
Endpoint captures do not prove continuous input processing, flicker absence or
compositor display acknowledgement. Equivalent geometry requests do not establish
identical per-frame work; no causal observer-overhead or subtractive CPU partition.
The result decides whether a working existing backend is a credible resize-cost
remedy. If it is not, use the supported source/driver allocation location to select
one further concrete code correction, with its method and stopping condition
recorded before execution; do not start a general performance program.

RZ-C03: Verify any selected correction against the unchanged fixture and resize
recipe, report improvement or failure without invented numerical acceptance,
and hand the concrete visible result to the owner. Full historical resize QA
closure and broad qualification remain separate from this current-host repair.

Unchanged reviewed evidence route is reused: evidence traceability passed before
this amendment (28 routes); no protected prototype or previous evidence changed.

## First comparison result and next bounded correction

The fallback correction restores visible GL output. All five readiness/changed/
restored endpoint captures match Vulkan exactly. GL app CPU is31.09%/33.82%
(width/height); Vulkan34.25%/37.65%. API presentation counts differ394/379 versus
218/256, so these totals are not a causal partition or identical delivered-frame
comparison. KWin CPU22.71%/22.51% versus23.82%/26.02% includes the whole desktop;
its GPU counters are permission-denied. Each is one valid interval, not a stable
percentile baseline. Raw results and GPU/client boundaries:
`target/cpu-resize-repair-20260930/backend-comparison.json`.

This is insufficient to call a backend switch a solution. Current shared source
requests optional prefix/composition storage even on a changed native
configuration. Those images are invalidated at the next extent. Next correction:
on a changed target after an accepted native presentation, render the required
full graph without optional retained images; admit retention again on a subsequent
frame with the same successfully presented target. No skipped resize frame, timer,
new refresh limit, lowered sample count, idle redraw or editor-specific policy.
Startup and offscreen capture retain their existing graph. Track only one fixed
previous successful Target within the already accounted prefix metadata. Failed,
stale and capture receipts cannot advance it.

Before rebuilding, preserve the GL-fallback-corrected control executable. Run the
existing frame/attachment tests plus a transition control for failed/capture/stale
presentation. One guarded optimized build, then one Vulkan launch per preserved
control/new candidate, each with one10s horizontal and one10s vertical interval
under the same controller, visible readiness and accounting method. No replacement
intervals or profiling. Exact endpoint equality and reduced resource work support
retaining the correction; failure preserves the evidence and triggers code/method
review, not retries. S4 closure and budgets remain unchanged.

<!-- EVIDENCE:GUI-RESIZE-REPAIR:GL-READINESS -->
GL readiness/output correction verified as described above. Eight frame-state
and six attachment-lifetime tests pass on the source with both corrections;
initial zero-test filter is preserved as setup error, not proof.

## Retained-image candidate disposition

The one paired observation did not demonstrate application CPU reduction:
width38.781% control versus40.917% candidate; height38.292% versus38.240%.
All five endpoint images match exactly. Candidate source/executable is archived
in `target/cpu-resize-repair-20260930/rejected-retention-candidate`; its three
tracked source edits were removed. Preserve all measurements; no replacement
intervals. The verified GL capability fallback correction remains. This result
is not resize repair acceptance and does not establish a hardware ceiling.

Next work is source-level method feasibility for separating logical layout size
from reusable physical presentation storage. No new collector or third-party
code is introduced. A concrete diagnostic candidate, method and finite validation
will be recorded before any build/run. Default production rendering stays exact.

## One diagnostic allocation-reuse prototype

Disabled by default: `DATUM_DIAGNOSTIC_RESIZE_ALLOCATION=quantized`. Require
native Wayland with fractional scaling on this reference host. Logical/input/
terminal sizes update immediately, independently of physical buffers. After a
changed native size, round physical storage up to32pixels per axis, matching
the observed Intel tiled backing allocation granularity. Keep original logical
projection, scale painter scissors into the actual raster extent, use the full
existing8x painter/resolve graph when extents differ, and let winit's existing
viewporter present to actual window dimensions. No new native protocol client,
dependency or renderer graph. Use a quiet deadline of three observed monitor
refresh intervals to restore exact storage, via the existing native redraw
coordinator. This is a diagnostic settling rule, not a numerical acceptance
budget; suspended/hidden/zero extents cannot leave an idle timer running.

Transient resampling is explicit; do not claim exact intermediate target pixels
or production quality acceptance. Exact stable captures and restored hit/layout
state are mandatory. Test raster clips, immediate state updates, settling and
failed/retired ownership. Build one diagnostic candidate; one Vulkan launch with
one10s width and one10s height interval, otherwise unchanged controller/
accounting/settings. Compare the already preserved latest control; do not repeat
it merely for a new handoff. Stop on output/accounting/protocol failure and
preserve it, no replacement. Report actual configuration/presentation counts
and resource use. A large allocation-work reduction with responsive correct
output enables an owner review of this concrete workaround; this prototype
itself does not ratify a new default, S4/S5 acceptance or hardware ceiling.

Prototype launch stopped during construction, before any CPU interval: hidden
Wayland surface had no current-monitor association. Preserved in
`target/cpu-resize-quantized-20260930-candidate`. Correct setup uses the current
monitor or exactly one available display, using a tentative single-output rule; multi-display ambiguity refuses. One corrected build/launch is allowed
under the explicit continuing repair goal, no measured interval replacement.

The tentative output fallback also refused before measurement: winit's available
output list is multiple, so the single-display assumption is withdrawn. Evidence:
`target/cpu-resize-quantized-20260930-setup-corrected`. Correct lifecycle setup
now starts exact and initializes the diagnostic only after the mapped surface
receives its actual current-output association. The controller requires that
initialized-output record before readiness/timing. One corrected startup, no
measured replacement. Display/hardware policy is not inferred from these failures.


## Allocation-reuse observation and decision

Mapped-output startup succeeded. Evidence is sealed in
`target/cpu-resize-quantized-20260930-mapped-output`; source and executable are
preserved separately. One interval per axis, reused latest exact Vulkan control;
no replacement or extra sampling. Both runs used588geometry requests,17ms
cadence, the same fixture, dimensions, FIFO,8x and accounting boundaries.

| Axis/candidate | Elapsed s | CPU s | User s | System s | CPU % one core | Configurations | Presentation completions | App GPU engine s | KWin CPU % one core |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Width/control |10.107810|3.919910|1.299441|2.620469|38.781|278|277|2.298209|24.338|
| Width/reuse |10.124416|2.748392|1.770990|0.977401|27.146|53|454|4.328829|24.989|
| Height/control |10.175930|3.896592|1.270795|2.625797|38.292|257|257|2.406672|23.782|
| Height/reuse |10.119259|2.635779|1.707845|0.927935|26.047|61|433|4.461564|24.014|

CPU percentages normalize to one core, not whole-host processor capacity.
App descendants were placed in the owned cgroup before exec; exited descendants
remain accounted, controller/KWin excluded. GUI exited0, final cgroup empty,
inputs unchanged and no throttling. Read boundaries enclose geometry markers;
actual padded elapsed is reported. GPU is deduplicated readable app DRM client
engine activity, not total utilization, energy or a ceiling. Some control FD
reads raced descriptor closure; matched client identities/counters remain
available. KWin CPU includes other desktop work; KWin GPU is permission-denied.
Timing logs are present in both runs; no observer-overhead attribution. These
single observations establish neither variability nor a numerical acceptance.

All five candidate images show the board correctly at expected dimensions.
Each restored image exactly matches its own ready image. Cross-launch ready
image differs in1482pixels along the cursor crosshair; no exact cross-launch
image or continuous-path equivalence claim. Endpoint captures do not validate
transient resampling, complete input processing or displayed-frame acknowledgements.

The supported decision: presentation-storage reuse is a concrete CPU-cost
lever, consistent with the earlier page-acquisition location. CPU fell30.0% /
32.0% relative despite increased API presentation completions; system CPU fell
substantially and configuration count dropped. This is not a complete CPU
partition or an overall-resource solution. GPU engine activity increased from
about23% to43–44% of interval elapsed as more full frames were rendered, and
transient exact pixels are not preserved. Keep the prototype disabled by default;
no production-quality, numerical-budget, S4/S5 or resize closure claim. The
initial owner question incorrectly framed retaining the CPU gain and preserving
quality as opposing choices. The owner's subsequent explicit clarification
supersedes that interpretation: retain this successful candidate and use it as
the baseline for further resize CPU improvements. It is not rejected. Continuing
repair remains limited to this current-host issue; no general campaign follows.

Focused verification for this checkpoint: mapped-output lifecycle test1passed;
raster-clip tests3passed; existing attachment tests6passed; optimized GUI build
succeeded; focused GUI app/render Clippy all-targets passed with warnings denied.
Project-state, evidence, spec governance/parity, progress coverage, source health,
dependency-authority and Cargo-resource-policy checks passed. A mistyped
`check_project_state.py` invocation named a nonexistent script; corrected to
`project_status.py check`, which passed. No full UI qualification was run or claimed.


## Owner clarification: retain the first CPU gain

The owner explicitly directs retention and further improvement; verbatim direction
is in `owner-direction.json`. The earlier option selection must not be read as
rejection, nor as authorization to remove the prototype. No removal occurred.
Commit86228807 retains the active diagnostic source; the sealed measured candidate
has SHA256 `b0c291a45086fe9b066be46ff2e35ce0832bfb62d65393469a48c910a5bedb12`.
Both match the evidence record. Treat the measured30–32% CPU reduction as the
first demonstrated improvement and preserve it as the development baseline.

RZ-C02 now continues from this candidate: identify remaining avoidable work using
existing source and phase evidence first, then record one justified bounded
correction and comparison before execution. Compare future candidates with this
preserved successful executable, rather than discarding the gain or rerunning old
controls by default. Preserve exact settled output; investigate transient quality
and GPU work alongside CPU. Higher GPU activity with more completed frames is a
tradeoff to explain, not sufficient evidence to discard the CPU improvement.
Default activation and production-quality acceptance are not asserted by retention.
No numerical budget, dependency adoption, tuning, S4 reopening or S5 acceptance.


## Next bounded correction: retain peak storage through one resize

Existing candidate evidence shows53/61configurations despite454/433completed
presentation calls. The source rounds the current extent in both directions,
so each descent through a32pixel boundary recreates storage already allocated
on ascent. Next opt-in variant `quantized-retained` retains the largest required
physical extent until the existing three-display-interval quiet deadline; grow
only axes whose current requirement exceeds capacity, using the same32pixel
allocation granularity. Static axes remain exact. Logical/input sizes remain
immediate; required full8x rendering and existing viewporter resampling continue.
No additional delay, frame cap, policy threshold, dependency or protocol client.
Quiet, hidden and zero-size paths release peak state and restore exact storage.
The previous `quantized` implementation/flag and successful binary stay available.

This correction targets repeated downsizing allocation and unnecessary static-axis
raster expansion. Peak storage may render more pixels while shrinking, so report
GPU activity and transient-quality limits, not just CPU. The finite method is:
focused state-transition tests, one guarded optimized build, then one Vulkan
candidate launch with one10s width and one10s height observation using the same
fixture, sizes,17ms cadence, FIFO/8x, readiness inspection and process accounting.
Compare the sealed successful quantized observation; do not repeat it. Stop and
preserve any substantive output/accounting failure, no interval replacement.
Inspect all endpoint images and require restored pixels equal to ready. Changes
in API work counts remain explicit; no complete-path or observer-overhead claim.
Lower configuration work and CPU without a substantive correctness failure
support retaining this variant for further repair; no broad acceptance follows.


## Retained-peak result

Sealed packet: `target/cpu-resize-peak-retained-20260930-candidate`. Both measured
intervals completed, GUI exited0, final cgroup empty and pinned inputs unchanged.
Ready pixels match the successful quantized control exactly; both restored images
match own readiness exactly. All endpoints visually correct at expected sizes.
No measured retry. State-transition tests2passed; optimized build and GUI-app
all-target Clippy with warnings denied passed.

| Axis | Elapsed s | CPU s | User s | System s | CPU % one core | Configurations | API presentation completions | App GPU engine s | KWin CPU % one core |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Width |10.091409|2.203981|1.723704|0.480277|21.840|13|434|3.103528|24.575|
| Height |10.165668|2.002469|1.582630|0.419839|19.698|8|369|3.178182|22.232|

Relative to the successful quantized packet, CPU percentage fell19.55%/24.37%,
configuration count53/61fell to13/8, and app GPU engine activity fell from
42.76%/44.09% to30.75%/31.26% of actual interval elapsed. API completion counts
also fell454/433to434/369; no identical delivered-work or causal CPU/GPU partition.
Same geometry588requests/17ms, same capture and descendant accounting method.
Retain this as the next demonstrated improvement without discarding `quantized`.
One mid-width exact reset appears in the native log; review before changing the
settling rule, rather than inventing another timer. No acceptance budget or
continuous quality/idle/input qualification is inferred.

## Owner manual QA of successful quantized mode

The owner first reported improved responsiveness from a development-profile
`--board` launch without explicit diagnostic flags; attribute that only as manual
ordinary-launch evidence, inherited environment not established. Subsequently the
owner ran the supplied release/direct-board command explicitly selecting
`quantized` and Vulkan and reported much better resizing, faster zooming and a
CPU display below4.4% during heavy abuse. Exact statement and command are saved
in `target/cpu-resize-repair-20260930/owner-quantized-manual-qa.json`.
This is practical positive evidence for retaining the first win. CPU display
normalization/tool/cadence are unknown; do not compare4.4% directly with controlled
one-core figures or turn it into a numerical budget. Zoom was an owner observation,
not a new automated qualification scope. Preserve both successful candidates.


## One remaining-cost localization observation

Current retained-candidate wall logs round most upload/encode/preparation values
to zero milliseconds. They do not partition remaining CPU. Existing opt-in
`DATUM_RESIZE_CPU_PROBE` provides process/main-thread CPU windows for configure,
prepare, acquire, renderer, present and enclosing event rounds. Reuse it unchanged:
one current retained-candidate Vulkan launch, one10s horizontal observation with
same fixture/geometry/cadence, readiness/output checks and cgroup boundaries.
No new build, profiler, dependency, vertical repeat or modified rendering policy.
Preserve the outcome; no replacement interval or extra sampling. This diagnostic
identifies the dominant remaining phase and whether work is on the main thread,
not another comparative speedup or causal observer-overhead assessment. Process
phase clocks exclude descendants, overlap child scopes, and include activity from
all application threads; never sum inclusive event-round/renderer/acquire windows
or equate them to complete cgroup accounting. Log overhead and scope gaps remain
explicit. Only a supported concrete avoidable-work finding warrants a next correction.


Remaining-cost observation completed and sealed in
`target/cpu-resize-retained-phase-20260930`. Cgroup CPU2.661838s over10.164072s;
383API presentations,7configurations,577native resize applications. Main-thread
CPU windows: renderer1.093864s, prepare0.182305s, configure0.017740s,
present0.060857s. Acquire0.062657s is nested in renderer; enclosing event_round
1.723960s overlaps those windows. Process CPU nearly matches main-thread windows.
Do not sum inclusive scopes, infer observer overhead or compare this logged
packet as another speedup. Ready/changed/restored captures match the retained
control; restored equals own ready. Exit0, empty group, unchanged pins.

This localizes most observed remaining phase work to main-thread rendering,
not configuration or scene preparation. It does not yet identify an avoidable
inner renderer operation. Internal wall labels truncate to whole milliseconds;
CPU accounting uses microsecond cgroup counters and nanosecond phase clocks.
The measured CPU reductions are not artifacts of those wall-label roundings.


## Post-resize quiet-idle verification

One current retained-candidate launch, no build: use the existing controller and
accounting, readiness inspection and restored-pixel checks. Two unmeasured owned
window geometry requests grow width by180logical pixels then restore it. After
three seconds and the existing exact-extent settling check, observe one five-second
idle interval. Disable the phase CPU probe; retain existing presentation logging.
Report CPU/user/system/elapsed, compositor and available GPU counters separately.
No replacement interval, new resize comparison or numerical acceptance criterion.
This checks whether retained allocation returns to exact quiet storage without
recurring configuration or presentation work after interaction. It does not
qualify long-duration idle, continuous output or all event paths.


Post-resize idle packet `target/cpu-resize-retained-idle-20260930` completed once.
Actual elapsed5.036309s; application/descendant CPU0.001696s, user0.001286s,
system0.000411s (independent counter rounding accounts for1microsecond sum
difference),0.033675% of one core. No applied resize, configuration or API
presentation completions during the interval. Available app DRM engine counters
unchanged. Ready/setup-restored/idle-restored images identical; exit0, final
cgroup empty, pins unchanged. KWin CPU0.370s (user0.250,system0.120),7.346650%
of one core covers the whole desktop, not work attributed to Datum; KWin GPU
counters permission-denied. This supports correct return to quiet exact storage
after this resize. It does not establish long-duration idle or observer overhead.

Both allocation-reuse wins remain in the codebase and archived executables. The
retained variant now has width/height resource evidence and post-resize exact
quiet-idle evidence. Ordinary default still uses exact-sized allocation; diagnostic
retention is not silent production adoption. Remaining integration must explicitly
address the transient resampling boundary and current-host scope; no broader
hardware policy, new CPU budget, S4 reopening or broad UI acceptance follows.
