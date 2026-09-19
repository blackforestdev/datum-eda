# Active resize defect recovery

<!-- REQ:GUI-PERFORMANCE-SPEC:GPS-C01R -->

Status: active owner-authorized execution; unresolved.

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
