# Workflow validator delivery contract

Frontier: WORKFLOW-DELIVERY-IMPLEMENTATION / WDQ-I02, then WDQ-READY.
Issue: `dat-wdq-rollout-implementation-ffy`.
Route: `workflow-delivery-infrastructure`.
Machine contract: `specs/workflow_delivery/rollout.contract.json`.

<!-- WDQ-INFRA-SCOPE -->
## Scope and authority

The consuming workflow is a contributor validating a staged transaction, an
exact candidate, or current roadmap delivery status through Datum's real
read-only entry points. The machine contract uses category `infrastructure`;
passing it is not native EDA delivery, owner acceptance, enrollment or activation.
PM042 is activation-conditional; preparation alone does not ratify it.

The CLI consumer is `check_workflow_delivery.main`, with `staged_cli` and
`candidate_cli` entry surfaces. The selector consumer is
`workflow_delivery_selector.selector_failures`, with `project_status` surface.
These are actual Python functions, not invented product dispatch verbs. Capture
real invocations and raw results; any correlation adapter must identify the
actual function and mode, not emit synthetic success in lieu of invoking it.

Readiness binds the assembled candidate's exact source inventory, fixture recipe,
authority closure and environment. Proof records source commit, input hashes,
interpreter and Git/toolchain identity, exact command arguments, exit status,
stdout/stderr and before/after repository state for each case. Retain all refusal
and failed attempts. Independent replay uses a distinct session and fresh events
against the same reviewed inputs and fixture identity. A test count or copied
producer log is insufficient.

The existing workflow tests provide fixture recipes and expected negative cases;
generated Git identities, timestamps and lease inputs must be recorded explicitly.
If regenerated fixtures differ, do not claim byte identity: freeze the actual
fixture input or use an explicitly reviewed deterministic recipe with recorded
parameters. Fixture factories may prepare inputs, never fabricate the observed
proof, registry, dispatch, environment or review record used for this contract.

The stable authority closure contains this specification, its machine contract,
the local evidence review and PM042. Progress, mapping proposals, runtime proof,
review and owner receipts are separate operational records. Normative changes
invalidate proof; recording a result must not rewrite its own requirements.
PM041 contract schema 1 and the prepared version-2 checkpoint mapping are reused.
The proposed PM042 environment-selection/headless extension is separate from
contract shape and remains unimplemented until its candidate work is verified.

<!-- WDQ-INFRA-FOUNDATIONS -->
## Foundation dispositions

CAD units/precision, numeric entry, selection identity, grid/snap and
library/connectivity are not applicable: this workflow authors no geometric
value, CAD subject, snapped point, library binding or net. Infrastructure IDs
and hashes do not substitute for those product foundations.

Undo/cancel is applicable only as nonmutation on refusal or process interruption;
no design undo capability is claimed. Persistence/recovery is applicable as
unchanged source/index/ref/trust state followed by a fresh successful invocation,
not project Save or crash recovery. Settings scope is applicable to reading only
the explicitly selected repository-local trust and requested snapshot; it does
not accept Global Preferences. Exact numeric types, full object IDs and byte
hashes are control-validation requirements, not CAD precision acceptance.

For every scenario, protect tracked and relevant untracked source bytes, captured
index, Git refs, local trust configuration, tracker and Frontier state. Generated
logs and Python caches are owned derived output, enumerated separately; they
must not hide changes to any protected state. Never run hostile cases against
the live shared repository. Promotion mutates state by separate owner authority
and is outside this read-only contract.

<!-- WDQ-INFRA-S01 -->
## INFRA-S01 — usable success and diagnostic contract

Invoke both real CLI modes and the project-status selector on a complete, valid
isolated enrolled fixture. Record zero findings and successful command status;
the selector must retain the actual selected task and completion step. Exercise
an invalid invocation as well: readable reason, input location and nonzero
enforce status, with no success claim. Report-only mode is explicitly diagnostic;
its zero status never authorizes an enforce-mode publication. JSON reports keep
readiness_asserted and acceptance_asserted false. Text diagnostics must be usable
without color, pointer input or a GUI; no native AT-SPI claim follows.

<!-- WDQ-INFRA-S02 -->
## INFRA-S02 — coverage, ownership and source permission

Exercise missing, duplicate and newly unclassified Frontier rows; historical
reopening; specification execution; deferred activation; unauthorized external
step/path changes; missing enrollment; expired, absent and mismatched claims;
uncovered files and prefix-lookalike scope escapes. Each actual entry point must
refuse the relevant fixture, preserve owning-lane context and pass a restored
authorized fixture. Do not change the real Preferences item to construct a case.

<!-- WDQ-INFRA-S03 -->
## INFRA-S03 — readiness and specification truth

Exercise completed preflight without a contract, stale routes, unresolved owner
questions, missing required normal handlers, all-normal-N/A product contracts,
missing clause inventory, omitted/duplicate clause dispositions, unratified
mechanism and an invalid pending-owner link. Exercise a correct pending authorship
fixture without inventing its future output. Refuse invalid cases before a
delivery transition and preserve distinct planning/implementation status.

<!-- WDQ-INFRA-S04 -->
## INFRA-S04 — independent review is not owner acceptance

Exercise skipped review, self-review, copied events, altered replay inputs,
unaccounted defects and unavailable-only required normal product observations.
A valid independent review without an owner receipt may complete review, never
acceptance. Acceptance still refuses absent/wrong receipts and unresolved blocking
defects. Explicit authorized nonblocking deferral remains possible. Preserve
legacy mapping behavior and infrastructure's null activate/accept boundaries.

<!-- WDQ-INFRA-S05 -->
## INFRA-S05 — exact snapshot and candidate history

Stage a bad record then repair only its worktree copy; staged validation must
still refuse. Reverse the setup and demonstrate the requested snapshot is used.
Exercise missing/nonancestor bases, ignored inputs, deletion/rename, intermediate
add/delete/revert and merge-parent path changes in an exact candidate. History
cannot erase a permission violation. The selector remains a current-worktree
diagnostic, not a certificate for historical commits. Never mutate a captured
snapshot as an automatic repair.

<!-- WDQ-INFRA-S06 -->
## INFRA-S06 — trust, correlation and recovery

Exercise missing or changed externally selected runner/trust, stale source and
authority, wrong fixture, binary/build/environment, registry identity and event
correlation. Enforce mode refuses; diagnostics do not upgrade trust. Interrupt
a running isolated invocation, record termination and protected-state hashes,
then invoke a valid fresh case successfully. Repeat after a refusal. No automatic
claim, digest refresh, index repair, owner receipt, config update or destructive
rollback is permitted.

Resolve `dat-wdq-environment-scope-kgq` using the proposed PM042 environment
section before producing mixed-enrollment proof. Exercise the unchanged accepted
pilot environment alongside a distinct observed headless infrastructure
environment through CLI, selector and owner-hook paths. Selection is explicit
per enrolled identity and pinned to owner-selected authority, never inferred
from candidate proof. Include every environment-shape, selection, identity,
toolchain and authority refusal listed in that section. Pipes have no terminal
dimensions; an actual PTY records character columns/rows, not fictitious pixels.
Headless infrastructure must not weaken product native evidence requirements.

<!-- WDQ-INFRA-FIXTURE -->
## Fixture recipe and observation procedure

The implementation unit in I03 must turn this reviewed procedure into retained,
replayable inputs and capture tooling. This section is a procedure, not a claim
that those inputs, tools or observations already exist.

1. Freeze the exact assembled source commit, authority/base IDs, policy,
   contracts, input manifest and explicit environment selection. Preserve the
   full 56-row real-roadmap coverage case, accepted pilot evidence and bounded
   Preferences observation. Refresh the source baseline before preparation;
   never rewrite a prior failed attempt to point at newer source.
2. Prepare two clearly separated fixture families: a complete real-roadmap
   snapshot for integration/unchanged-pilot checks, and small hostile repositories
   for individual refusal conditions. Existing Fixture, accepted_fixture and
   clause-inventory factories describe synthetic INPUT recipes only. Their fake
   binary, native events, reviewer and owner receipts must never be emitted as
   actual infrastructure OUTPUT evidence or presented as real product approval.
3. Make each mutation case a fresh owned repository under the resolved Git-common
   proposal store. Record initialization commands, exact changed paths, byte
   content, Git object/index identities and selected configuration. Case setup may
   create intentionally invalid data; the observed validator may not repair it.
   No case edits the shared worktree, another lane's files or a live trust bundle.
4. Record every clock/lease parameter. A frozen live-claim fixture can be replayed
   only while its recorded lease is genuinely valid. Do not monkeypatch the
   production clock or silently renew a fixture and claim identical inputs. If
   the replay window expires, prepare a new identified fixture and rerun producer
   and independent observations; retain the earlier attempt and reason.
5. Retain a sanitized fixture archive or deterministic recipe plus exact parameter
   manifest. Include source/index objects, requested snapshots and required Git
   history, not merely final tracked files: history/revert/merge cases depend on
   intermediate states. Exclude credentials, host-global config and unrelated
   repositories. Verify restoration and hashes before calling it replayable.
6. Run real subprocess entry points: staged and candidate CLI, project-status
   check/details, and the exact prepared owner hook where applicable. Helper
   functions called inside unittest are regression evidence, not substitutes for
   these process invocations. Preserve real argv, cwd, exit/termination status,
   raw stdout/stderr and requested snapshot identity. No fixture success string
   can substitute for a child process's actual output.
7. Observe actual dispatch into the contract's named Python functions. I03 capture
   instrumentation must bind source file/function identity, interpreter binary,
   entry surface and invocation to real calls and retained raw outputs. Merely
   constructing a registry from the contract is not a production registry export;
   observing a subprocess start alone does not prove its handler ran. Normal
   enabled cases require actual handler invocation. Early refusal may record an
   uninvoked handler honestly, never reuse a normal-case event as its evidence.
8. For every case capture before/after source bytes and modes, tracked and relevant
   untracked inputs, exact index tree, refs, selected trust configuration, tracker
   and Frontier. Enumerate each permitted derived log/cache separately. Retain
   only sanitized configuration evidence; never publish credentials. A summary
   saying "unchanged" without the state comparison is insufficient.
9. Bind each result assertion to its exact case, command and raw observation.
   Every required normal success is followed by each distinct normative refusal
   and a restored valid run. A group passes only if all its subcases and required
   dimensions have evidence. Keep expected refusal separate from unexpected
   harness failure, timeout or missing executable; those cannot count as a pass.
10. Interrupt only a PID/process group created and observed live by this run.
    Retain the termination result and protected-state comparison, then run a
    fresh valid invocation. Do not kill by name, assume a timeout means exit,
    reset history or delete artifacts to manufacture recovery.
11. Independent replay restores the reviewed fixture and runs the same complete
    case matrix with a distinct eligible session and fresh invocation/event IDs.
    Copied producer logs, fabricated timestamps or a replay on changed toolchain,
    fixture or source cannot establish independence or equivalence.

The matrix is the complete INFRA-S01..S06 prose, including every enumerated
mutation and the proposed PM042 environment refusals, not just tests that already
exist. I03 must inventory subcase IDs and expected codes before capture. Record
inapplicability per entry point with a concrete reason: for example, a selector
cannot certify intermediate candidate history, but the candidate CLI must test
it. Do not mark whole scenario groups inapplicable to avoid missing coverage.

The 69-file input proposal covers the inspected local Python imports, not the
complete hook/capture toolchain. Final closure must add actual capture/fixture
modules and their imports, shell subprocess inputs, environment selection files,
and the owner's hook implementation. Hook proof also consumes
`scripts/check_file_lane_ownership.py`, `scripts/check_rustfmt.py`,
`specs/rustfmt_exemption_manifest.json`, and staged-file inputs those checks read.
Record external interpreter, Git, shell, realpath and any invoked rustfmt identity.
Do not introduce Cargo compilation merely to exercise the read-only hook; if
compilation becomes necessary, the guarded resource policy remains mandatory.
No broad permission for scripts, crates or prototype edits follows from this list.

<!-- WDQ-INFRA-LIMITS -->
## Remaining obligations and proof storage

Proof destination: `docs/reviews/workflow-delivery-rollout/infrastructure/proof.json`.
Independent review: `docs/reviews/workflow-delivery-rollout/infrastructure/review.json`.
Their absence is honest until the required real runs occur. This specification
does not assert readiness; WDQ-READY must validate the assembled exact candidate.

I03 and WDQ-REVIEW additionally prove the real owner-hook path and promotion
tooling: exact pins, clean-state refusal, no preparation mutation, visible partial
activation/interrupt handling and retained recoverable support. They must review
the complete coverage/enrollment and external-lane packet, not only these core
validator scenarios. I04 owner ratification/activation, installed-main validation,
three full native adoption cohorts and I06 acceptance remain separate obligations.
No product ownership, GUI prototype, licensing or dependency boundary changes.
