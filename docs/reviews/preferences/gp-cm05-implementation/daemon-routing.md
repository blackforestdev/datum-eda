# GP-CM05E owning-daemon routing correction

Status: scope promotion installed; runtime correction prepared and tested but
not landed because the accepted workflow pilot's build inputs would be stale.

## Exact owner authority

In this Preferences session on 2026-09-16, the owner approved scope candidate
`a467ccc4dc87df54a6fa26926c26cd8d8ef4cf68`, request SHA-256
`84ed1af5f8b054b7f1d7ceae75d7026fc187178935f0e92dbab0b3903fea695c`,
with the verbatim response:

> Approve exact promotion; other writers remain paused

The exact transaction published that candidate and installed its pinned trust
configuration. Prospective enforcement, installed enforcement, the enabled commit
hook, project-state validation and the current-task selector all passed.
The retained transaction log is
`/tmp/datum-gp-cm05-daemon-routing-proposal/activation-1789588018607746961.jsonl`.
The extension adds only the previously missing daemon dispatch file to GP-CM05E.
The existing live claim and execution authorization remain active; GP-CM05E is
unfinished and GP-CM05A remains pending. No product acceptance is recorded.

The first commit attempt exposed that the existing claim omitted dispatch even
though the newly promoted source inventory included it (WDQ-COVERAGE). The file
was restored before synchronizing the Frontier lease and beads claim with the
already-approved path. The lease heartbeat/head were refreshed; no new task,
authorization, completion or acceptance was created.

## Runtime correction

In the preserved patch, ordinary CLI queries, mutations and proposal operations select
the configured reachable owner before constructing a local product service.
The daemon executes the typed request through its existing product service.
The CLI's foreground-TTY confirmation remains mandatory for direct human writes.
Script actors cannot mutate directly; the MCP adapter retains its separate
proposal-only route, broker and existing host-authorization doorway.

Only an absent socket or connection refusal during initial discovery permits
standalone operation. A connected owner's product/RPC refusal, malformed reply,
response-identity mismatch, timeout or later disappearance never selects a local
writer. Product schemas and success/error shape are checked on reply.

The prepared implementation was further corrected for the daemon's serial socket
server: each RPC connection closes before the CLI waits for human confirmation.
Subsequent requests reconnect to the same selected endpoint without rediscovery
or fallback. This prevents a waiting user from monopolizing the daemon. No new
writer, dependency, public MCP mutation verb or product acceptance is introduced.

## Verification

Guarded focused Rust tests passed: five CLI tests and six daemon tests. These
cover host authorization, initial discovery, response identity, connection loss,
RPC refusals, per-request connection release, selected-owner disappearance,
script/MCP mutation restrictions, stale-generation refusal, shared service state
and existing broker/restart replay. Four MCP adapter tests also passed.

The rebuilt main-checkout CLI and daemon passed the real-process routing proof
at `/tmp/datum-gp-cm05-daemon-routing-live-proof/report.json`: all six queries,
confirmed Set/Reset, stale-generation refusal without creating a second local
repository, and standalone reads after owner shutdown. Binary hashes, all ten
command observations and daemon logs are retained in that bundle.

Guarded CLI/daemon strict Clippy passed for all targets with warnings denied.
Source health, Preferences private-writer boundary, daemon write parity,
dependency authority, Cargo resource policy, specification parity and evidence
traceability checks passed. These focused checks are implementation evidence;
they do not assert full-workspace or final production proof completion.

The actual foreground-TTY concurrency proof passed at
`/tmp/datum-gp-cm05-daemon-routing-concurrency-proof.json`: while one CLI waited
at its APPLY prompt, a second CLI queried the same daemon successfully; the
first mutation then completed. The owning repository and configuration roots
were disposable project-backed fixtures, never the owner's Preferences.

This tested correction addresses the routing behavior tracked by
`dat-preferences-cli-daemon-routing-i4d`, which remains open until landing.
It does not close GP-CM05E or replace
its full same-candidate corpus, native/accessibility, resource, independent
review and release-handoff requirements. The captured generation-growth failure
`dat-preferences-generation-growth-764` remains unresolved; no limit is waived.

## Enforced landing blocker and preserved handoff

The subsequent project-state/render check reported WDQ-STALE against
`docs/reviews/workflow-delivery-pilot/corrected-build/input-manifest.json` for
the changed CLI and daemon Preferences sources. Those files are in the accepted
pilot's input closure. The published scope grants source ownership, but it does
not exempt those inputs from the existing accepted-consumer freshness checks.

The pilot's proof validator requires a matching fresh input manifest and build
receipt. Its independent-review validator binds both producer and replay proof,
and its acceptance validator requires the exact promoted owner receipt for that
packet/review hash. Updating a digest or reusing old acceptance is insufficient.
The delivery-contract/evidence owner must reconcile that affected-consumer proof
through its existing review and owner boundary before this Rust change can land.
This session has not resumed the halted workflow repair, edited pilot evidence,
changed another claim, disabled enforcement or recorded substitute acceptance.

All four runtime files were restored to committed bytes after retaining the
complete tested patch at `/tmp/datum-gp-cm05-daemon-routing-tested.patch`, SHA-256
`a44741f90252283dbd596ef1ce6c056335329a6a026bb9ee15443c2ef52e6c91`.
The isolated source checkout and both real-process proofs remain available.
Only the approved claim synchronization and this truthful handoff are being
committed; the blocked runtime correction is not represented as landed code.
