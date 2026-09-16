# GP-CM05E continuous storage capture

The owner auditor confirmed both findings against c243eb5e resolved by 98ddad83
and requested continuation of GP-CM05E. This unit adds actual continuous
generation-storage capture to the existing diagnostic collector. It does not
declare the candidate ready or change any Frontier completion state.

Each of three trials uses a fresh private configuration root on project-backed
storage, ten warm-ups and 100 measured alternating Set operations. Every
observation retains the actual process result, complete before/after file
manifests and each file's exact bytes. Embedded receipts/request indexes count
once as generation bytes; mutable head/lock metadata is separate. The collector
refuses redirected/special files and rewritten or removed immutable history.
The ordinary mutation timer now alternates Set values and requires changed:true
so that repeated no-ops cannot substitute for durable mutation measurements.

The retained diagnostic at
`/tmp/datum-gp-cm05-storage-diagnostic-20260916/capture.json` contains all 330
operations. With the existing September 7 release binary, all three trials
first exceed the 32 KiB generation-growth limit at operation index 23 (the 24th
operation, including warm-ups). Maximum new-generation sizes are 154948,
155254 and 155254 bytes. This is diagnostic evidence against that recorded
binary hash, not a final same-candidate production run.

The repository writes every accumulated receipt into every new generation.
`dat-preferences-generation-growth-764` records the demonstrated failure and
requires a correction preserving durable audit, idempotency, accepted-proposal
replay, immutable old generations, compatibility and all numeric limits.
No failed sample was discarded or relabeled. The existing collector remains
partial_diagnostic and still requires daemon, native and complete-candidate
proof before readiness.

Six focused storage/process regression tests passed, including exact retained
byte accounting, immutable-history removal/rewrite refusal, symlink/special-file
refusal, timeout retention and environment isolation. The real diagnostic
exercised the complete continuous three-trial storage protocol. Preferences
boundary, source health, dependency authority, Cargo resource, parity and
traceability checks passed.
The complete collector integration with one measured sample plus ten warm-ups
per variant also passed across three trials; its 72 diagnostic rows are retained
at `/tmp/datum-gp-cm05-storage-collector-integration-20260916/capture.json`.
This checks integration and changed:true enforcement, not full budget acceptance.
Project-state validation passed for all 57 Frontier items.

The owner also authorized preparation of the narrow missing daemon-dispatch
scope extension and runtime fix. That proposal is isolated at
`/tmp/datum-gp-cm05-daemon-routing-proposal` and
`/tmp/datum-gp-cm05-daemon-routing-preparation`; it has not selected installed
trust or changed the protected live dispatch file. The current claim,
GP-CM05E unfinished status and pending GP-CM05A owner review remain unchanged.
