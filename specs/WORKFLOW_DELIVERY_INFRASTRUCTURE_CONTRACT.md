# Workflow validator delivery contract

Frontier: WORKFLOW-DELIVERY-IMPLEMENTATION / I02 preparation, WDQ-TOOLS,
WDQ-READY, then I03 producer verification.
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

The validator consumer is `check_workflow_delivery.main`, with `staged_cli`,
`candidate_cli` and `owner_hook` entry surfaces. The prepared owner hook reaches
that same function through its authenticated bootstrap after the file-lane and
formatting prerequisites; it does not introduce another validator handler.
The selector consumer is
`workflow_delivery_selector.selector_failures`, with `project_status` surface.
These are actual Python functions, not invented product dispatch verbs. Capture
real invocations and raw results; any correlation adapter must identify the
actual function and mode, not emit synthetic success in lieu of invoking it.

For normal hook cases, observe the actual pinned validator function as well as
the shell/bootstrap invocation. Optional tracing must preserve the ordinary
enforcement path and authenticate its observer before execution. Early trust or
prerequisite refusal can legitimately precede the validator: retain that actual
diagnostic and record the handler as uninvoked, not as a successful dispatch.
Neither an observer event nor the declared surface proves complete child-tool
tracing, installed activation or promotion authorization. Infrastructure input
review uses the bounded verification definition below, not an exhaustive
descendant/runtime-library tracing requirement.

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

<!-- WDQ-INFRA-TERMINAL-OWNER -->
Within INFRA-S02-05, distinguish unauthorized boundary changes from PM042's
permitted last-owner-step closeout. Through C/R/S/H, retain separately identified
unchanged-pending and valid terminal-closeout successes. Refuse rewritten selected
requirements, changed other steps, an unfinished predecessor, execution authority,
missing completion evidence, document-only evidence without review/decision,
unclosed tracker, and missing/invalid landing commit. INFRA-S02-06 must still
refuse production edits accompanying closeout. Use isolated input fixtures, never
fabricated real Preferences acceptance. Retain actual subprocess observations;
in-process validator substitution is regression evidence, not renewed delivery
proof. This refines the existing case, adds no milestone, and waives no original
case or product evidence requirement.
<!-- WDQ-INFRA-TERMINAL-OWNER:END -->

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

<!-- WDQ-INFRA-WORKSPACE -->
### Workspace compatibility proof within INFRA-S05

Apply proposed PM042's `WDQ-042-WORKSPACE` boundary without weakening any original
matrix case. In addition to uncovered ignored source, retain separately named
compatibility observations for each of the following conditions:

1. Clean strict-policy checkout, recurring verified caches at all supported
   optimization levels, standard Beads runtime, and exact owner-local baseline.
   Run actual CLI, selector and preflight; keep selected-index/history isolation.
2. Candidate-only, changed, removed, malformed, executable or redirected workspace
   policy refuses. Cache policy without verified fresh source-only startup refuses.
3. Changed or deleted owner-local pins, unknown ignored source, cache lookalikes,
   forged payloads and redirects refuse; restored reviewed inputs succeed.
4. Legacy caches may be genuinely absent or regenerate into fully verified
   current tracked-source output. Changed unverified bytes and false absence
   through redirects refuse. Do not delete or regenerate real owner caches to
   construct these cases; use owned isolated fixtures.
5. Captured bad local/cache bytes cannot borrow temporarily qualifying live
   bytes, and captured missing owner files cannot borrow later restoration.
   Preserve the complete before/after state and cache-source identity.
6. Proof roots retain owner-local files, legacy payloads and Beads data. A cache
   whose source is outside the proof closure stays an input. A pinned runtime
   file inside existing proof roots must stale older proof through the actual
   CLI and selector; workspace eligibility is not product input irrelevance.
7. Observe the actual Datum ignored-workspace inventory read-only; distinguish
   derived files from the exact proposed legacy/local pins. Reproduce that mixed
   state in the real-roadmap fixture without changing another lane or deleting
   files to recover a passing check. Preserve both clean and mixed-state results.
8. During COMPAT, retain producer observations of the supported entrypoints and
   exact pre-review source/history inspection (`--inspect-review`) against the
   reviewed policy, source and input identities in the producer proof. After
   producer verification, RECHECK independently repeats those observations and
   inspection, incorporating them into its independent replay and review.
   Producer proof does not require the later independent observations. Each
   inspection validates prerequisites, not its own future evidence.
   Retain refusals and corrected attempts; no old proof certifies changed
   requirements. Subsequent evidence/governance commits create a distinct final
   candidate requiring explicit delta reconciliation and strict full-evidence
   `--inspect-promotion` at claim-free I04 before activation; that future final
   inspection is not an input to either earlier S05 packet.

These additional observations belong to INFRA-S05, not a substitute scenario or
product cohort. Existing 62 matrix identities and all their surfaces remain
required. Activation, installed verification and native adoption remain separate.

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
| INFRA-S04-11 | Legacy accepted pilot mapping and evidence unchanged; full real-roadmap instance at I04 after packet assembly | PASS; no retroactive new mapping obligation; retain separate I04 integration receipts | C R S H |
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

<!-- WDQ-INFRA-NONCIRCULAR-RENEWAL -->
### Renewal at a completed review boundary

When the real roadmap has reached pending WDQ-I04, its completed review
checkpoint requires the assembled producer/reviewer packet. That packet cannot
contain the success of a command whose prerequisite is that same packet.
Preserve the actual lifecycle, claims, tracker and completed evidence. Do not
reopen COMPAT/RECHECK or manufacture a live lease to obtain an earlier phase.

For this renewal, separate the obligations in this order:

1. Inspect exact committed source, history, scope, complete 56-row coverage,
   accepted pilot and bounded external-lane state without asserting a full gate
   success. This is a read-only assessment, not a supported `--inspect-review`
   success at I04 and not a substitute for the following integration runs.
2. Assemble and independently review the bounded scenario proof. Its assertions
   cover the synthetic refusal/success cases and observed workspace behavior;
   explicitly exclude the pending full-real-roadmap instance of INFRA-S04-11
   and related current-inventory/clean/mixed integration observations.
3. Run strict exact promotion inspection against the assembled packet. Then
   run fresh C/R/S/H checks, including selector check and details, on the exact
   complete real-roadmap snapshot against that packet. Preserve all 56 rows and
   the actual ownership and pilot evidence. Preserve all required clean, mixed,
   refused/restored and full observed-inventory variants, with fresh independent
   execution against the retained inputs; generic success is insufficient.
   Retain these separate I04 integration
   receipts outside the prerequisite proof, followed by actual installed checks.
   No I04 completion is possible without both sets of successful observations.

A normative amendment invalidates the old typed packet. It need not discard
raw observations of byte-identical executed code, but reuse requires a fresh,
independently reviewed applicability assessment: compare all executed modules,
fixture/input and selected policy/environment bytes, tools, scenario requirements
and protected-state evidence. A change only to the placement of an unchanged
obligation in this sequence is not a change to its tested runtime assertion;
explicitly document that distinction in the applicability assessment.
Keep original source IDs, timestamps, authority,
invocation IDs and failures immutable. Changed behavior, inputs or scenario
requirements require new observations; unresolved equivalence is not a pass.

Any new assessment events must identify themselves as current assessments of
referenced historical observations, not new subprocess executions or independent
replay. They bind the final contract, authority and fresh construction manifest;
they cannot relabel old typed events, manufacture dispatch, supply missing cases,
or substitute for actual independent replay or fresh final integration. Retain
the original raw records and the explicit comparison alongside the new packet.
Product acceptance and installed-trust requirements are unchanged.
<!-- WDQ-INFRA-NONCIRCULAR-RENEWAL-END -->

WDQ-TOOLS must turn this reviewed procedure into retained, replayable inputs and
capture tooling before final WDQ-READY review; I03 then produces the required
verification evidence. This section is a procedure, not a claim
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
7. Observe actual dispatch into the contract's named Python functions. The capture
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
exist. WDQ-TOOLS must bind subcase IDs and expected codes in the prepared runner
before readiness review and I03 capture. Record
inapplicability per entry point with a concrete reason: for example, a selector
cannot certify intermediate candidate history, but the candidate CLI must test
it. Do not mark whole scenario groups inapplicable to avoid missing coverage.

The 152-file input proposal retains the previous 76 paths and adds the 66
implemented workflow/capture/promotion Python modules and tests omitted from that
baseline, plus three rollout contract/mapping/proposal tests. The prepared owner
hook shell source, explicit environment-selection document and its two selected
environment Blobs, coverage reconciliation test, activation implementation and
activation integration test are also included. Their
local Python import graph is included. These are read dependencies only. The
separately reviewed coverage proposal names 141 explicit permission paths;
read-input membership does not grant write or owner-publication authority.
Final bounded input review must include
any subsequent capture/fixture modules and imports, shell subprocess inputs and
changed environment selections or hook implementation. Hook proof also consumes
`scripts/check_file_lane_ownership.py`, `scripts/check_rustfmt.py`,
`specs/rustfmt_exemption_manifest.json`, and staged-file inputs those checks read.
Record external interpreter, Git, shell, realpath and any invoked rustfmt identity.
Do not introduce Cargo compilation merely to exercise the read-only hook; if
compilation becomes necessary, the guarded resource policy remains mandatory.
No broad permission for scripts, crates or prototype edits follows from this list.

<!-- WDQ-INFRA-BOUNDED-VERIFICATION -->
### Bounded source and tool verification

For this infrastructure contract, record interpreter/Git/shell/env/realpath and
any invoked rustfmt paths, hashes and versions separately from repository input
hashes. Review the actual script/import, hook and subprocess command paths.
Include every named tool and relevant repository/data/fixture input used by the
reviewed cases; an interpreter hash alone is insufficient. Disable unreviewed
Python startup/user-site injection and record the actual flags/environment.

Exhaustive descendant-process and runtime/shared-library tracing is not required.
Existing Python module/maps observations remain supplemental and retain their
honest limits; never change an unperformed-tracing flag to true to pass review.
Bounded input completeness is not hermetic runtime completeness and does not
detect arbitrary host-library substitution. Record that residual limitation in
producer proof and independent review. This definition also governs references
to final infrastructure input/toolchain closure in the execution plan and older
preparation notes; those notes remain historical observations, not new blockers.

Pinned sources and named tools, real handler/entry-point observations, all
required success/refusal and state-preservation/recovery cases, independent
replay and exact owner activation remain mandatory. Product-native evidence and
the accepted pilot are unchanged. No new tracing subsystem or dependency is
required. A supplemental full drift/CI run has its own much wider build/input scope;
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

The separate `--inspect-review` mode uses closed `review-inspection-request`
bytes and refuses owner-response arguments. It requires matching valid live
COMPAT/RECHECK execution state in base and candidate, permitting only the exact
initial mapping addition, and preserves every other Frontier item. Prove
missing future rollout proof can pass only this non-authorizing inspection,
while strict promotion/activation refuses. Prove stale claims, lifecycle or
other-lane changes, unscoped history, malformed mapping, changed prior enrollment,
support/environment mismatch and protected-state changes refuse. Other enrolled
items still need their ordinary evidence; ordinary changed-input readiness still
escalates to proof. The result explicitly sets `evidence_validated:false` and
cannot be consumed as an activation request or owner receipt.

The prepared `--inspect-promotion` mode additionally requires a separately
selected response matching the exact `promotion-inspection-request` bytes.
It checks the PM042 initial-publication boundary and all enrolled delivery
evidence, including rollout independent replay, between repeated clean-state
inspections. Ordinary `--inspect` remains a preparation diagnostic; its success
cannot stand for this stronger check. Neither mode mutates main or local trust,
authenticates an owner, or completes controlled activation. Prove refusal for
unfinished predecessors, claim-bearing owner steps, changed external records,
out-of-scope full-history paths, missing/stale proof and a distinct authority
candidate. The successful full-candidate path also needs actual producer and
independent observations before this tooling can be declared ready.

The separate `--activate` mode requires exact `activation-request` bytes, an
external matching response and the PM042 writer-pause acknowledgement. It may
run only from its authenticated, prepared Git-common runtime. It takes the
cooperative activation lock, repeats full publication inspection, requires exact
defect dispositions, writes durable JSONL progress before mutations, fast-forwards
the exact candidate, installs the four Datum settings and then hooksPath, and
runs the installed hook and project-status check. Success does not complete the
roadmap owner step or accept the full reform. Failed/interrupted transactions
retain their journal and observed partial state; stale retries refuse, with no
automatic reset, repair, hook bypass or support deletion. The owner coordinates
all writers: a cooperative lock cannot stop an unrelated editor or Git client.
Refuse enabled worktree-specific configuration, and verify effective trust values
as well as the local configuration written. During an owned mutation or installed
verification child, handled SIGINT/SIGTERM records a pending stop and retains the
lock until that child finishes. Do not start another mutation after the signal.
Observe child PID, command, terminal result and output; the child inherits the
lock and runs in its own process group. The 120-second child deadline kills only
that owned group and reaps the child before reporting failure. SIGKILL cannot be
handled: retain the pre-mutation journal, and keep the inherited lock until the
surviving child exits. No unverified or interrupted run asserts activation.

Development tests now send actual SIGINT/SIGTERM during a live Git/post-merge
child and observe lock retention, no subsequent config write, child termination
and partial publication. A separate child-lifetime test kills its parent and
observes inherited-lock retention/release; it is not full Git-publication crash
recovery proof. Actual rollout producer/reviewer observations and complete
recovery-case coverage remain mandatory.
