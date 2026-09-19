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
