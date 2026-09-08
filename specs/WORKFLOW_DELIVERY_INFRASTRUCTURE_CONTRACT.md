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

<!-- WDQ-INFRA-CASE-MATRIX -->
## Required case inventory

This inventory defines future observations, not completed tests. Case IDs remain
stable in the producer and independent-replay records. `C` means staged CLI,
`R` exact-candidate CLI, `S` both project-status check and named task details,
and `H` the prepared owner hook. Execute every named surface separately; one
function-call test does not stand for them. For each refusal, preserve state and
repeat a restored valid case. Keep all other prerequisites valid enough to reach
the intended check: an unrelated earlier error does not prove that check works.

`PASS` means zero status and the expected unchanged selection/false assertion
flags; `REFUSE` means nonzero enforcement status and a diagnostic identifying
the intended failure. Record exact actual codes, not merely nonzero exit.
Invocation/termination cases retain their real process status separately.
Where a row lists several variants, each variant requires its own case record
with the row ID plus a stable variant suffix; none may be sampled away.

| Case ID | Input or observation | Required result | Surfaces |
| --- | --- | --- | --- |
| INFRA-S01-01 | Complete valid enrolled fixture | PASS | C R S H |
| INFRA-S01-02 | Invalid CLI argument and conflicting snapshot modes, separately | Invocation error; no successful check claim | C R |
| INFRA-S01-03 | Same invalid delivery input in report-only and enforce modes | Report-only diagnostic is not permission; enforcement REFUSE | C R |
| INFRA-S01-04 | Valid and refused cases with no color, GUI or pointer input | Text identifies result, path and responsible context; state unchanged | C R S H |
| INFRA-S02-01 | Missing, duplicate and newly unclassified Frontier rows, separately | REFUSE each classification defect | C R S H |
| INFRA-S02-02 | Reopen a historical item or add execution to its completion, separately | REFUSE without reclassification | C R S H |
| INFRA-S02-03 | Give a specification execution authorization or an execution step, separately | REFUSE both forms | C R S H |
| INFRA-S02-04 | Reactivate deferred work; change its retained completed history, separately | REFUSE; unchanged dormant history still PASS | C R S H |
| INFRA-S02-05 | Cross external-lane authorization, selected-step or requirement boundary, separately | REFUSE without reassignment | C R S H |
| INFRA-S02-06 | Modify production source in an unpermitted external-lane path | REFUSE; no session-name exemption | C R S H |
| INFRA-S02-07 | Authorize product or infrastructure execution without enrollment, separately | REFUSE both categories | C R S H |
| INFRA-S02-08 | Authorized source path with absent, expired or mismatched claim, separately | REFUSE each ownership defect | C R S H |
| INFRA-S02-09 | Source outside any scope; prefix-lookalike path, separately | REFUSE both escapes | C R S H |
| INFRA-S02-10 | Candidate changes policy category, scope or enrollment, separately | REFUSE candidate self-permission | C R S H |
| INFRA-S02-11 | New Rust, MCP and workflow source outside permissions, separately | REFUSE every production root | C R S H |
| INFRA-S03-01 | Complete preflight with no delivery declaration | REFUSE before execution | C R S H |
| INFRA-S03-02 | Stale route digest; missing authority marker, separately | REFUSE stale or absent authority | C R S H |
| INFRA-S03-03 | Unresolved mandatory owner question; missing foundation answer, separately | REFUSE incomplete readiness | C R S H |
| INFRA-S03-04 | Required normal consumer has no real handler | REFUSE readiness | C R S H |
| INFRA-S03-05 | Product contract makes every normal dimension not applicable | REFUSE scope evasion | C R S H |
| INFRA-S03-06 | Specification lacks clause inventory | REFUSE | C R S H |
| INFRA-S03-07 | Completed authorship omits or duplicates a clause disposition, separately | REFUSE incomplete accounting | C R S H |
| INFRA-S03-08 | Candidate claims an unratified mechanism is ratified | REFUSE candidate-only authority | C R S H |
| INFRA-S03-09 | Pending-owner disposition names absent, completed or non-dependent decision, separately | REFUSE each invalid link | C R S H |
| INFRA-S03-10 | Valid pending authorship with no future output | PASS without invented completion | C R S H |
| INFRA-S03-11 | Unassigned, unenrolled product remains pending planning | PASS planning; separate execution attempt REFUSE | C R S H |
| INFRA-S04-01 | Skip review or complete review without its record, separately | REFUSE | C R S H |
| INFRA-S04-02 | Reviewer equals an implementation session or original producer, separately | REFUSE self-review | C R S H |
| INFRA-S04-03 | Copy producer events into purported replay | REFUSE reused observations | C R S H |
| INFRA-S04-04 | Replay changes fixture, source/input or executable identity, separately | REFUSE changed replay | C R S H |
| INFRA-S04-05 | Omit a producer or replay defect from review findings, separately | REFUSE missing accounting | C R S H |
| INFRA-S04-06 | Required normal product observations are unavailable-only | REFUSE producer and replay variants | C R S H |
| INFRA-S04-07 | Valid independent review with owner disposition still pending | PASS review only; acceptance REFUSE | C R S H |
| INFRA-S04-08 | Acceptance receipt absent, wrong or candidate-only, separately | REFUSE acceptance | C R S H |
| INFRA-S04-09 | Unresolved blocking defect at acceptance | REFUSE | C R S H |
| INFRA-S04-10 | Exact authorized nonblocking deferral with otherwise valid acceptance | PASS without deleting the finding | C R S H |
| INFRA-S04-11 | Legacy accepted pilot mapping and evidence unchanged | PASS; no retroactive new mapping obligation | C R S H |
| INFRA-S04-12 | Valid infrastructure review with null activate/accept mapping | PASS review, never product acceptance | C R S H |
| INFRA-S05-01 | Bad staged record repaired only in worktree | C/H REFUSE; S judges its actual worktree | C S H |
| INFRA-S05-02 | Valid staged record with bad unstaged worktree version | C/H PASS; S REFUSE the worktree defect | C S H |
| INFRA-S05-03 | Explicit candidate differs from both index and worktree | R judges only requested commit and required history | R |
| INFRA-S05-04 | Missing or nonancestor candidate base, separately | REFUSE ambiguous history | R |
| INFRA-S05-05 | Ignored/untracked production input | S REFUSE uncovered worktree input; C/H exclude unstaged input | C S H |
| INFRA-S05-06 | Delete or rename an input across allowed/unallowed boundaries, separately | REFUSE escape and retain deletion/rename identity | C R S H |
| INFRA-S05-07 | Intermediate unscoped add/delete or change/revert, separately | REFUSE history even if final bytes match | R |
| INFRA-S05-08 | Unpermitted source change reachable through a merge parent | REFUSE hidden parent history | R |
| INFRA-S05-09 | Empty new transaction after historical change | PASS only the new transaction; no historical certification | C S H |
| INFRA-S06-01 | Missing or moving authority/base references, separately | REFUSE missing/ambiguous trust | C R S H |
| INFRA-S06-02 | Changed gate/runner bytes or missing selected runner, separately | REFUSE; no candidate fallback | C R S H |
| INFRA-S06-03 | Changed governing authority or relevant source input, separately | REFUSE stale proof | C R S H |
| INFRA-S06-04 | Wrong fixture, binary, build receipt or registry identity, separately | REFUSE each stale identity | C R S H |
| INFRA-S06-05 | Event input, dispatch, visible/state or artifact correlation differs, separately | REFUSE each mismatch | C R S H |
| INFRA-S06-06 | Interrupt this run's observed live child; then invoke a fresh valid case | Termination recorded, no success claim; state preserved; fresh PASS | C R S H |
| INFRA-S06-07 | Fresh valid invocation after an ordinary refusal | PASS; no automatic repair or authority refresh | C R S H |
| INFRA-S06-08 | Legacy pilot plus distinct headless infrastructure environments | PASS exact per-enrollment selection | C R S H |
| INFRA-S06-09 | Environment selection missing/duplicate/unknown enrolled key, separately | REFUSE every selection defect | C R S H |
| INFRA-S06-10 | Swap environments between two enrolled identities | REFUSE; no inferred equivalence | C R S H |
| INFRA-S06-11 | Change selected document, selected Blob or authority membership, separately | REFUSE candidate-selected request | C R S H |
| INFRA-S06-12 | Malformed selection/environment version, kind or fields, separately | REFUSE closed-shape violation | C R S H |
| INFRA-S06-13 | Changed recorded toolchain or interpreter digest, separately | REFUSE mismatch with selected environment/build | C R S H |
| INFRA-S06-14 | Invent terminal dimensions for pipes; omit actual PTY dimensions, separately | REFUSE headless record inconsistency | C R S H |
| INFRA-S06-15 | Product tries to use headless environment shape | REFUSE; native GUI evidence still required | C R S H |

History-specific R cases do not impose history certification on the worktree
selector or an empty new staged transaction. Conversely, omission of R from
worktree/index divergence cases does not waive exact candidate isolation: case
INFRA-S05-03 covers it. Hook launch/config failures may precede a WDQ JSON report;
retain and assess their actual visible nonzero diagnostic rather than inventing
a downstream code. Do not bypass the earlier file-lane or formatting checks to
reach WDQ: construct valid non-prototype fixture paths for those cases.

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

The 76-file input proposal includes the original 69 paths plus the two hook
checks, exemption manifest and four referenced Rust files. Those seven additions
are read dependencies only; the 69-path source-permission proposal is unchanged.
This covers the inspected local imports and concrete repository hook inputs,
not the complete future capture toolchain. Final closure must add actual capture/fixture
modules and their imports, shell subprocess inputs, environment selection files,
and the owner's hook implementation. Hook proof also consumes
`scripts/check_file_lane_ownership.py`, `scripts/check_rustfmt.py`,
`specs/rustfmt_exemption_manifest.json`, and staged-file inputs those checks read.
Record external interpreter, Git, shell, realpath and any invoked rustfmt identity.
Do not introduce Cargo compilation merely to exercise the read-only hook; if
compilation becomes necessary, the guarded resource policy remains mandatory.
No broad permission for scripts, crates or prototype edits follows from this list.

For the external toolchain, record interpreter/Git/shell/realpath and any invoked
rustfmt executable identity separately from repository input hashes. Python
capture must also inventory actual loaded standard-library/extension modules
and shared-library inputs; hashing only the interpreter is insufficient. Disable
unreviewed Python startup/user-site injection in the controlled capture process
and record the exact flags/environment. Do not fetch a dependency to perform this
audit. A supplemental full drift/CI run has its own much wider build/input scope;
the presence of its wiring in this contract is not evidence that all its Rust
and GUI gates ran in the bounded infrastructure capture.

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
