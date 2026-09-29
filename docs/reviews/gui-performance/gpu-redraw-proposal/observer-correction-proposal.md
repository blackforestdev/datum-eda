# R3 input observer correction proposal

Status: proposed, not implementation or native-run authorization. Owner requested
this proposal after the stopped crosshair replay. GPI-S4 and
`dat-gui-performance-implementation-vkq` remain open. This is a measurement-method
amendment to PM050, not a new renderer mechanism or a CPU optimization.

## Problem and evidence

At commit `5ab4803a`, the one approved replay was preserved in
[r3-crosshair-replay/receipt.json](r3-crosshair-replay/receipt.json). The native
application rejected a GPU sample with missing workload demand identity before
final capture. Its last accepted demand IDs were 4071, 4082 and 4093. The observer
has a 4096-record lifetime cap; it records all Main window events plus meaningful
native rounds, not merely the 3600 producer requests (1280 integer changes).
Capacity exhaustion is strongly supported, but neither the actual overflow flag
nor the rejected tuple survived. Other unscoped changes can produce this error.
The earlier missing-crosshair image remains unexplained.

Source ownership and demonstrated defects:

- `native_input_observation.rs` owns capture, admission, routing records and
  export. Refused admission still permits dispatch without a workload scope.
  Full storage also rejects a provisional round before finding it did no work.
- `frame_workload.rs` correctly invalidates unscoped changes. Its strict causal
  tags must remain unchanged; stale tags must never substitute for missing ones.
- `native_gpu_measurements.rs::Host::poll` validates before serializing a sample,
  losing the offending sample when validation fails.
- `app_shell.rs::fatal_gui_error` exits the process immediately. Normal
  `finish_measurement_observation` is bypassed; even a direct call to the current
  input export requires a successful drained manifest before writing anything.
- `gpu_r3_native_trial.py` continues its timed waits after process exit and later
  reports a focus assertion. Producer sends after exit prove no native delivery.

The solution must preserve event history across the workload without an
unbounded allocation, keep failed evidence usable, and retain strict rejection
of incomplete results. Increasing 4096 alone does not address these defects.

## Proposed ownership and storage

Keep one opt-in Main input observer. Extract record/state capture and journal
storage into real owned modules under `crates/gui-app/src/`; the current
519-line module must shrink under the source-health policy. Do not add a second
observer, dependency or alternate render path. Disabled observation allocates
no journal, starts no writer and performs no observer disk writes.

Replace the lifetime `Vec<Record>` with a preallocated page pool and one writer
thread using the standard library. The event thread captures fixed-size records;
the writer serializes completed pages to an exclusively created JSONL sidecar.
It never calls the renderer or owns GPU resources. No formatting, file I/O,
blocking channel send or writer acknowledgement occurs in event admission.

Proposed fixed bounds (all are failure limits, not claims of measured capacity):

| Resource | Bound and admission rule |
| --- | --- |
| Record representation | At most 1024 bytes by `size_of`, including exact bounded event-kind code and existing before/after state; retain 128-byte hover bound and fail on truncation |
| Pages | Exactly 16 pages of 256 records; 4096 slots, at most 4 MiB payload total across producer, queue and writer; pages are moved/recycled, never copied into an extra full-page queue |
| Pending demand | One separate provisional record, at most 1024 bytes; first-failure metadata has a separate bounded control slot and cannot be denied by a full data queue |
| Serialization | At most 4096 bytes per JSONL record, checked with a capped writer; bounded header/footer, each at most 16 KiB; no whole-history JSON allocation |
| Journal | At most 128 MiB total, including header/footer; at most 131072 committed records; whichever limit is reached first stops admission; reserve footer bytes before data writes |
| Writer and control | One thread with requested 256 KiB stack; metadata/control allocations at most 64 KiB, plus one 4096-byte serialization buffer; record actual allocations, queue high-water, persisted counts/bytes and write errors |
| Termination | At most 2 seconds to acknowledge journal finalization after event dispatch ends; no unbounded thread join; retain the existing outer 150-second trial guard |

These bounds limit observer-owned storage, not process RSS, page cache, allocator
or OS thread overhead. Those remain part of later observer-cost accounting.
Reuse the existing 4096 slots as burst capacity, rather than a lifetime history
limit. A recycled pool can record more than 4096 total demands. No finite buffer
can guarantee an arbitrary native event rate or disk stall; pool exhaustion is
an explicit failed trial, not permission to drop records or block the event loop.
The journal's byte and count caps bound history even if events arrive unusually
fast. The existing replay cannot establish a worst-case rate, so no throughput
or native feasibility claim follows from these chosen bounds.

Seal partial pages when full and at existing workload phase transitions and
normal/failure finalization. The currently filling page and pending record must
be included in controlled failure finalization. Do not add redraws, timers or
wakeups to flush records. An abrupt kill or storage failure can still leave an
unwritten tail: report that loss explicitly; never claim a complete receipt.

## Admission and identity rules

Replace ambiguous boolean admission with Disabled/Admitted/Failed outcomes.
Maintain committed sequence independently of page length. Preserve current
contiguous one-based workload demand IDs and zero-based exported sequence IDs.
Do not infer workload phases from receipt arrival time.

For a native event, reserve a storage slot and establish the renderer workload
context before dispatch. Clock, nesting, allocation/admission or context errors
latch a specific first failure and abort the instrumented trial before that
event executes. Report the unhandled event and pending state; do not continue
with an unscoped mutation. This failure path is permitted only for the opt-in
measurement run; ordinary product input behavior is unchanged.

A native round uses the separate provisional record and next tentative ID,
without first consuming a page slot. After the round, retain it only if existing
render-activity tracking says it did work. A no-op discards the record and rolls
back context exactly as today, including when the page pool is full. A meaningful
round commits the tentative ID and record; if no slot is available, latch a
specific failure, preserve the provisional record and stop. Any work already
submitted in that round remains incomplete diagnostic evidence; it cannot pass
qualification. Never overwrite an older event, reuse a committed ID or silently
coalesce events. Event-kind codes must distinguish cursor enter/leave, focus,
occlusion, resize/scale, redraw and close without recording unbounded payloads.

## Failure evidence and strict completion

Introduce `datum.input-journal/v2` plus `datum.input-receipt/v2` summary. The
journal has a bounded header (PID, workload identity, schema and storage limits),
ordered records and terminal status/counts. The summary references its exact
sidecar basename, byte length and record count; the runner archives SHA256 pins
for both. Completion requires full parsing, matching identity/counts, contiguous
sequences, no truncation or missing tail, successful writer acknowledgement,
existing semantic input checks, controlled shutdown/observer handshake and the
original strict GPU drain seal. A hash or writer acknowledgement alone is not
completion. Reject unknown schemas. Preserve existing v1 historical validation.
Use a bounded post-exit parser to present v2 records to existing semantic checks;
never reconstruct a missing record or relaxed causal tag.

The v2 failure summary is independent of the drain seal: `complete=false`, first
failure code/context/time, subsequent errors, admitted/completed/persisted counts,
limits/high-water, pending record, last available state, and explicit
`gpu_drained=null` plus drain-unavailable/error reason. The writer owns ordered
serialization and final status. Missing runtime/device/handshake cannot suppress
this summary. A failed file write or stuck writer can prevent export; preserve
that fact in process/runner diagnostics, and retain the readable sidecar prefix.
Do not promise durability against SIGKILL, disk failure or power loss.

Refactor opted-in fatal handling at App-owned call sites so it latches and
requests failure export before the current fatal exit. Apply this to GPU poll,
event/round admission and other fatal exits reachable during the measured Main
run. Preserve original failure text and process failure status. Finalization
errors append to the first failure. No GPU drain, new frame, device recovery or
successful shutdown acknowledgement is invented on a fatal path. Normal shutdown
continues to require all existing drain/handshake conditions; failure there also
requests the invalid export. Writer failure terminates the measured trial at the
next existing event/round checkpoint or finalization; it schedules no new GUI work.

In `Host::poll`, preserve rejected sample metadata before returning validation
failure, through a separate `gpu_measurement_rejected` record: reason, host,
epoch, frame, submission, raw workload/attempt identities, ticks and manifest.
Bound to the existing maximum six submissions and timer-slot capacity; explicit
truncation makes the diagnostic incomplete. A rejection must never appear as an
accepted `gpu_measurement` or disappear from the runner's invalidity checks.
Log-write failure remains an additional error. Keep strict renderer validation.

The runner checks application liveness during existing timed waits and before
focus/capture. On unexpected exit, latch exit code and earliest native failure,
stop the owned producer, preserve partial producer output and all raw sidecars,
then perform bounded cleanup. Producer cancellation must flush its partial
schedule in `finally`; it cannot pretend to have sent all 3600 requests. Do not
replace the application failure with later focus/cleanup errors. Successful
workload coordinates, cadence, duration, screenshots and tolerances stay fixed.

## Exact requested execution scope

Approval would authorize only this observer correction and offline proof:

1. Modify the named app observer/shutdown/fatal integration files and extracted
   observer-owned modules, the receipt parser and runner/producer cancellation
   helpers, plus their focused tests and required source-health/governance files.
   Necessary App-owned fatal call sites include `app_native_events.rs`,
   `app_frame.rs`, `production_status_refresh.rs`, `app_shell.rs` and the three
   product-window hosts. No renderer algorithm, GPU resource lifetime, CPU event
   scheduling, golden/prototype, workload, dependency or system-tuning change.
2. Compile serially with `run_cargo_guarded.py --workload proof`, Cargo offline,
   and disk-backed targets. Run only the new/affected observer, workload-tag,
   receipt and runner controls below, then scoped default/visual Clippy with
   warnings denied. No GUI startup, GPU-backed test, release candidate/baseline
   build, benchmark, native trial or DRM observer execution.
3. Stop the proof batch on its first failure, preserve it and resolve only defects
   within this mechanism; rerun only the affected offline controls after a
   documented correction. A mechanism or scope change returns for review.
4. Produce an implementation/method receipt, updated source/bounds inventory and
   remaining gaps. Only then prepare a separately reviewed native validation
   packet with new binary/method pins and fixed quota. Do not reopen any stopped
   campaign, mint an executable declaration or infer native approval from this
   offline result. Independent review is advisory, not owner ratification.

## Offline proof and decisions enabled

| Cohesive batch | Required evidence and decision |
| --- | --- |
| Storage/admission | Deterministic writer control, more than 4096 total records with recycling, exact ordering; cap-1/cap/full pool, disk-byte/count caps, allocation and write failure; reject without blocking/dropping. Proves bounded storage rather than a larger quota. |
| Native scope/round coupling | Full-pool no-op round succeeds without overflow; meaningful full-pool round preserves pending state and fails; rejected event does not dispatch; clock/nesting/context failures stay specific; committed IDs and causal phase unions remain strict. Tests must exercise shared admission/dispatch decisions, not only disconnected counters. |
| Failure/export | Missing runtime/drain, lost handshake, malformed GPU tags, failed/partial writes and stuck writer retain first failure and available prefix; export never yields complete=true. Complete healthy fixture still requires the original seal and semantic checks. No actual GPU required. |
| Consumer/runner | v1 compatibility, complete v2 fixture and rejection of gaps, wrong identity, truncation, missing footer/seal, rejected GPU samples; fake child exit cancels a fake producer and preserves partial schedule before focus/capture; existing mismatch remains fatal. No desktop or native application launched. |
| Build/governance | Serial guarded scoped compile/lint, source-health extraction/ratchet, resource policy, traceability, spec governance and Frontier checks. These permit review of the correction; they establish no rendering or performance acceptance. |

No test is counted as already passed. Reuse prior exact8 component evidence at
its original scope; observer-only edits do not justify repeating those GPU runs.

## Method amendment and remaining repair

This deliberately changes the prior no-event-time-I/O method to opt-in worker
streaming. Worker CPU, disk/page-cache work and contention must be included in
future measurement accounting; no overhead subtraction or comparison with the
old buffered observer is valid. Any later baseline must use identical new
instrumentation and accounting, including the writer thread in the application
process/cgroup. Native readiness and observer cost remain unproved after offline
success. If overhead makes the method unsuitable, stop and review it rather than
performing an optimization sweep.

The observer gap blocks another useful crosshair diagnostic and final
qualification. Complete DRM lifetime accounting separately blocks final GPU duty
qualification; it does not block offline observer correction or a later explicitly
nonqualifying crosshair diagnostic. See the preserved DRM finalization source
review; this proposal does not ratify its rejected observer draft. Exact 8x
antialiasing, painter order, shared renderer ownership, invalidation/recovery,
live/retiring GPU accounting and original GPU budgets remain unchanged. S4/S5 and
the GPU issue cannot close on this proposal or its offline proof. CPU remains
deferred.

Owner response for this scope: `approve GPU observer correction and offline proof`
or request a revision. Native replay is excluded from that approval.
