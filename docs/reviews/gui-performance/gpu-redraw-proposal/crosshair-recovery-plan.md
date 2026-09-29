# Crosshair recovery: one output diagnostic before more measurement machinery

Status: proposed scope for owner review, not execution authorization.
This replaces the pending streaming-observer proposal as the recommended next
GPU recovery action. GPI-S4 / `dat-gui-performance-implementation-vkq` remain open.
The streaming proposal was committed at `a6171591`, not implemented; the worktree
was clean at this reassessment. Its proposed writer/journal is withdrawn from
the requested execution scope, without deleting the historical proposal.

## What went wrong and what must be answered

Recent blockers include new measurement lineage validation, runner assertions
that prevented evidence export, and likely exhaustion of the new causal input
observer. These are instrumentation defects, not proof of legacy renderer bugs.
The one preserved native output failure is more specific: 2753 pixels differ,
exactly the missing reference crosshair; all other pixels match. The application
state at that capture is unknown. The subsequent replay failed before capture
and did not resolve that uncertainty.

The immediate question is: at the final screenshot, was the native cursor and
interaction state cleared, or was correct state absent from rendered output?
Full per-demand GPU timing history is unnecessary to answer that question.
Source already supplies concrete alternatives: Main `Focused(false)` clears the
interaction overlay; `CursorLeft` also clears native cursor and overlay. Neither
is established as the observed cause. Correct native state with missing pixels
would narrow the problem to render/preparation/presentation, not prove a specific
shader or damage bug.

## Comparison and decision

| Approach | What it answers | Cost and limitation | Disposition |
| --- | --- | --- | --- |
| Existing binary with all observers off | Whether the final image matches in ordinary rendering | No native receipt of input application or state-clearing event; a mismatch still needs another diagnostic | Too little evidence for the single next run |
| Increase4096, shorten the workload, or retry current GPU mode | May avoid the last failure temporarily | Does not fix diagnostic loss or prove capacity; changes the question or repeats defective machinery | Reject |
| Streaming journal/writer/new receipt format | Complete causal history for a long instrumented GPU measurement, subject to writer/storage limits | Adds threading, I/O, parsing, shutdown and overhead-accounting obligations; does not itself diagnose rendering | Not necessary now; withdraw from next scope |
| Existing bounded buffer, output-only mode, semantic transitions | Final exact output plus native pointer delivery and cursor/hover/camera/focus changes around capture | Small observer and runner changes; no GPU performance result, and no guarantee of reproducing the old failure | Recommended |

The earlier proposal conflated repairing full measurement infrastructure with
answering the immediate rendering question. Separate them. Disabling timestamps
and causal workload tags for this diagnostic is an explicit method change, not
passing the failing instrumentation or qualifying the GPU repair.

## Small diagnostic change

Use an explicit opt-in output-diagnostic mode within the existing input observer:
`DATUM_GPU_MEASUREMENTS=0`, no `DATUM_WORKLOAD_MANIFEST`, and reject conflicting
options. Keep the existing input-receipt path and controlled shutdown. The
renderer therefore allocates no timestamp queries, and its drained measurement
manifest is `None` by the existing API. Do not call causal workload context
set/end in this mode. The production rendering algorithm is unchanged.

Reuse the4096-record in-memory capacity; no writer, streaming file, new thread,
new dependency, larger cap or v2 format. Use one provisional before/after record
outside that capacity so no-op event/round observation can be discarded even
when storage is full. Retain:

- Every Main pointer/button input, including routing and before/after state.
- Exact event-kind labels for cursor enter/leave, focus, occlusion, resize/scale
  and close, retained regardless of whether state changed.
- Other window events and native rounds only when semantic state changes:
  cursor/native cursor, hover, camera, pan, focus, scale/extent or device epoch.
  Merely submitting a frame or advancing a render revision does not consume a
  record. Preserve render revision/activity in retained snapshots as context.

Maintain monotonically ordered diagnostic record IDs and timestamps; these are
not GPU causal demand IDs. Capture any unexplained gap between a preceding after
state and the next before state as a coverage failure, not an inferred event.
Retain final state before shutdown; capture time is bracketed by the runner's
monotonic screenshot timestamps and the state-transition history. A transition
within the capture interval makes the state/image association ambiguous.

The existing pointer oracle expects1280 integer-coordinate changes for3600
scheduled requests. This explains why filtering unrelated redraw/round records
removes the demonstrated pressure; it does not prove an upper bound on platform
events. If retained records exceed4096, stop with an explicit diagnostic
capacity failure, preserve the pending record and incomplete buffer, and do not
raise the cap or rerun. No overwriting, sampling or silent record loss.

Export the existing-shaped records to a separate `input-diagnostic.json` with
`complete=false` and explicit output-diagnostic mode/first error/count/cap, before
normal shutdown drain/handshake. This file is never accepted as a qualification
receipt. Normal healthy shutdown may still write the ordinary input receipt;
its failure must not erase the earlier diagnostic snapshot. Buffer/admission
failure should request the same snapshot before abort. This does not expand
into a universal fatal-handler redesign: unrelated abrupt exits may still lose
the in-memory tail, which must be reported as unavailable.

Keep pixel mismatch latched through bounded export/cleanup. Check process
liveness during existing runner waits and before focus/capture; unexpected exit
stops the owned producer and preserves partial schedule/stderr. No new polling
thread or event-loop wakeup. Use existing JSON structures with additive explicit
diagnostic fields, a focused diagnostic reader, and the existing pointer/input
oracle where applicable. Do not invoke GPU demand/statistics validators in this
mode or make the old performance campaign accept incomplete receipts.

## Shortest bounded execution packet

Owner approval of this plan would authorize the following sequence, stopping on
the first failed prerequisite or native result. No work below has run yet.

1. Implement only the observer mode/filter/provisional record/pre-drain snapshot
   and runner liveness/diagnostic-result changes. Primary files:
   `native_input_observation.rs`, `native_measurement_shutdown.rs`, the existing
   native-round call site, `gpu_r3_native_trial.py`, receipt/input helpers and
   narrowly necessary producer cancellation. Extract actual observer-owned
   record/filter logic if required to shrink the oversized module. No renderer
   algorithm or general measurement-framework repair.
2. Run focused offline controls: more than4096 irrelevant rounds retain no
   records; pointer records and named semantic transitions survive exactly;
   semantic changes in a native round are retained; full-cap no-op versus real
   overflow and coverage gaps fail correctly; missing drain/handshake cannot
   erase the diagnostic snapshot; mismatches/early child exit remain failures.
   Reuse existing pointer oracle controls. This batch determines whether one
   native run can yield interpretable evidence. Use serial guarded offline
   Cargo for the affected tests/checks and required source/governance gates;
   no GPU-backed component reruns or broad proof suite.
3. After those prerequisites pass, make exactly one source-pinned candidate
   release build with the diagnostic change. No baseline build. Seal one new
   declaration with fixed cap1 and unchanged fixture/CLI/reference-image pins;
   every previous campaign remains stopped. Use the existing reference-host
   P630/Vulkan/exact8/Fifo1280x800 configuration,40-second readiness allowance,
   5-second warmup,30-second120Hz rectangle,5-second still interval and150-second
   overall limit. No shortened workload, new screenshot tolerance, pointer
   correction after the workload, forced extra frame, parameter variant or retry.
4. Run once in output-diagnostic mode. Preserve readiness/final screenshots,
   capture timestamps, native identity, full producer schedule, applied-input
   records, semantic transitions, pre-drain snapshot, shutdown outcome and raw
   errors. No GPU percentiles, duty or CPU acceptance report.
5. Interpret and report against the table below. End this packet after the one
   result and source analysis. Do not automatically fix a guessed renderer bug
   or enlarge instrumentation. A concrete demonstrated defect supports a
   bounded corrective patch/proof proposal; a successful nonreproduction does
   not establish the original failure's cause.

## What a native result would establish

| Result | Interpretation and next decision |
| --- | --- |
| Zero changed pixels, expected pointer sequence/state, unambiguous capture interval, healthy shutdown and complete diagnostic coverage | One correct, interpretable native output result for this configuration. Timestamp-disabled success leaves the old instrumented failure unresolved; preserve it. Restore timing only through a reviewed measurement repair, not an immediate performance campaign. |
| Missing crosshair and cursor cleared before capture by a recorded event | Trace the named native/application handler and compare behavior with the interaction contract. Distinguish legitimate cursor-leave/focus behavior, an uncontrolled workload interruption and an application bug before changing rendering. |
| Missing crosshair with correct stable native state throughout capture | Focus subsequent source analysis on frame preparation, retained damage/restoration and presentation using the pinned image/state; do not blame platform or hardware. State alone does not locate the exact rendering defect. |
| Input mismatch, a state transition during capture, coverage gap, overflow, process death or missing export | Diagnostic is invalid or ambiguous. Preserve the specific failure and stop. No assertion that the renderer passed, no automatic retry or new infrastructure. |

This is the shortest justified path to *attempt* one correct, interpretable
native result. It cannot guarantee correct output before the unknown defect is
understood. An interpretable mismatch is useful evidence, but is not that success
criterion and cannot close the repair.

## Scope that remains separate

Full GPU timestamp lineage/observer correctness and measurement overhead remain
required before performance inference. A complete causal observer may eventually
need a different retention method, but this failure does not establish that a
streaming writer is necessary. Reconsider it only against a demonstrated remaining
measurement requirement and a comparison with simpler storage/admission fixes.

Full DRM lifetime accounting remains a separate final-qualification obligation;
its unresolved finalization accounting and unapproved execution method neither
block nor become authorized by this output-only diagnostic. Exact8, painter order,
invalidation/recovery, live/retiring GPU resources and original performance limits
remain required for final repair acceptance. CPU optimization stays deferred.

Approval response for this complete bounded packet:
`approve GPU crosshair focused recovery plan`.
This authorizes the listed focused changes, offline proof, one candidate build
and one diagnostic native run, not performance/DRM qualification or issue closure.
