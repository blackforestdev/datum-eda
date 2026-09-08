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
