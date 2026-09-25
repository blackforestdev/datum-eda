# Shared GUI performance implementation and adoption contract

Status: PM045 ratified specification; owner authorized amended S0–S5 execution
on2026-09-20. Runtime acceptance remains separate. Parent:
`GUI_PERFORMANCE_RECOVERY_PLAN.md`; engineering: `GUI_SHARED_ENGINEERING_CONTRACT.md`;
budgets and methods: `GUI_PERFORMANCE_ACCEPTANCE_MATRIX.md`.

<!-- EVIDENCE:GUI-PERFORMANCE-SPEC:GPS-C04-ADOPTION -->

## Traceability and proof semantics

`docs/reviews/gui-performance/implementation-map.json` is the individual mapping.
Its requirements have stable IDs, source clauses, consumers, shared owner,
production entry points, implementation slice, positive scenario, deliberately
bad control, expected artifact and acceptance rule. Its inverse indexes are
derived from those rows, not independently maintained assertions. The 67 parent
clauses and 25 historical cases remain individually addressable. Historical test
references are supplemental evidence, never a claim that the proposed integrated
production proof already exists. Each future test is explicitly marked planned.

All parameterized rows run through each listed consumer's real event adapter to
the shared production owner. Calling a helper with a renamed dialog title does
not qualify Project Preferences. Record the complete adapter-to-frame call path,
host/consumer IDs and before/after state alongside counters. Deterministic backend
fault injection substitutes only the backend result, not the scheduler, input
handler or resource owner being tested. Native trials remain separate.

Evidence under `docs/reviews/gui-performance/implementation/<slice>/<requirement>/`
must contain candidate and baseline commit/source/binary hashes, fixture and
environment manifest, exact command, raw counter/event/state data, static native
captures, statistical result and positive/negative result. This is a future path
contract, not fabricated files. Temporal evidence is additional when applicable;
static captures cannot pass a deferred temporal requirement. File presence and
inventory validation alone do not establish semantic coverage or acceptance.

For every negative control use the historical first parent recorded in
`c01-historical-hunk-review.json` if it runs the same recipe and fails for the
target defect. Otherwise apply one isolated defect injection in a disposable
source copy: record patch/hash and prove unchanged non-target behavior. Never
modify the active checkout or original fixture to manufacture a failing result.
Build serially through the guarded Cargo runner using disk-backed artifacts.
A build failure is not defect sensitivity. Positive passes and the specific
negative assertion must both be observed; report missing proof as missing.

## Proposed bounded slices

Each slice first pins the baseline and touched paths, records relevant matrix
rows and proof before code, then runs those rows against the candidate. Absolute
budgets and exact zero-work/correctness predicates both apply. Use STAT-01 for
relative claims and retain failed attempts. An unexplained regression blocks
advancement. Complete cross-consumer endurance is reserved for S5. Rollback means
reverting the isolated slice, preserving document/settings/terminal state and
earlier proven behavior; it is not a destructive reset of shared work.

| ID | Dependency and scope | Required evidence / rollback boundary |
|---|---|---|
| S0 | Separate owner execution grant; pin current source/fixture/environment, select the first bounded S1 change from attributable work or a demonstrated correctness defect, and validate only the measurements used for that selection and its proof | Record production entry points, matched baseline, verified input/final state, applicable counters and their conformance/overhead, relevant static/input regressions, and every outstanding ADM/GPU/ACC obligation with its owning S1–S4 component. No global admission, budget or performance pass follows. Keep measurement repair separable from behavior changes. |
| S1 | S0 attribution; shared NativeFrameCoordinator, host damage generations, queue/configure coordination and recovery in main and all three auxiliary hosts | Establish implementation readiness from the current production call-site inventory for all four native hosts, retired competing default paths, shared-owner correctness regressions, actual per-host native adapter evidence, and implemented surface-generation/submission/attachment-lifetime accounting. Preserve bounded affected-change positive/negative evidence and all failed attempts; an unexplained regression still blocks advancement. Record each unfinished SH/LF/affected-HP predicate in its existing map row. Outstanding prescribed per-row negative-control replays, complete native final-state/static/input/focus coverage, backend/scale coverage and full resource/method qualification become explicit S5 exit prerequisites. They remain unqualified, not satisfied by S1 readiness. Rollback preserves coherent shared adoption. Resize qualification stays separately nonblocking. |
| S2 | S0 measurement; use S1 shared interfaces when available without selecting work out of order. Retained world/encoding/screen uploads, text and cache ownership | Establish bounded shared-implementation readiness from the default production consumers of retained world, encoding, screen/uniform upload and shaped-text ownership; retired competing paths; affected warm/replacement/eviction and painter/AA/text correctness proofs; and explicit resource limits. Keep each unfinished implementation and qualification predicate in its existing map row. Complete accounting/caps, exact changed-range transfers and complete shaping/layout dependency implementation are S4 exit prerequisites; full native/replacement/device-reset/negative and budget qualification is required at S5. Partial limits are not complete byte caps. Unexplained regressions block readiness. Preserve separately reversible geometry/text/upload patches. |
| S3 | S2 retained resource interfaces. Shared dialog/control profiles and scroll/clip/hit contracts in both Preferences, New Project and main panels | Establish bounded shared-implementation readiness from the current default production consumers of shared dialog/control/scroll/clip/hit owners, retirement of the New Project general backdrop and competing machinery, affected correctness and defect-sensitive tests, bounded native adapter evidence, and warm reuse/eviction behavior at explicitly recorded limits. Preserve legitimate discrete Layers/terminal semantics and existing preference/project authority. Record every unfinished S3 qualification predicate individually in its existing adoption-map row as required at S5, including complete per-consumer native input/focus/final-state/static appearance, backend/scale, prescribed negative controls, pass/forbidden-work and cache qualification. None becomes passed through readiness. Missing default-path adoption or an unexplained implementation regression still blocks S3 exit. Complete S2-owned resource dependencies retain their S4 implementation and S5 qualification deadlines. No other unfinished implementation is deferred by this amendment. |
| S4 | S1–S3. Terminal adapters, mixed activity/fairness, all-host closure and fault/recovery adoption | PTY byte/state preservation, hidden-render suppression, two-generation glyph bounds, allocation/recovery accounting and serial shared-device multiwindow proof. Concurrent multi-device crash remains separate. Revert rendering adapters without replacing TerminalCore/PTY state. Complete every unfinished S2 accounting/cap, exact changed-range transfer and shaping/layout dependency implementation carried in the adoption map, with affected positive/negative and counter-conformance proof. Private dependency internals remain a technical boundary to resolve within owner-authorized scope, not a waiver or permission to modify third-party code. |
| S5 | Every adopted implementation slice and required admission/method conformance. Final coverage, native owner UX, distinct-reviewer replay and endurance | Run all in-scope requirements on pinned candidate, both daily native backends and supported scale rows; 60-minute/200-cycle/20-recovery contract. Publish excluded temporal/T2/schematic rows as unqualified, never full performance acceptance. Full resize closure additionally requires all temporal/resource rows; no local or initial-scope result closes it. Close every in-scope SH/LF/affected-HP predicate carried from S1, including prescribed negative controls and native final-state/static/input/focus and complete resource/method proof. S1 readiness does not waive any such result. Close every in-scope qualification predicate carried from S2, including exact warm/replacement/address-reuse/eviction/device-reset negatives, painter/AA/text parity, complete byte caps and ACC-01/02 conformance across the required consumers/configurations. Unfinished implementation at S4 blocks S5 entry; moving a predicate does not satisfy it. Close every outstanding qualification predicate carried from S3, under its unchanged acceptance rule and actual applicable consumer scope. Consolidate native input/focus/final-state/static appearance, backend/scale, negative-control, pass/forbidden-work and cache proof with the existing pinned-candidate qualification. S3 readiness is not full adoption or performance acceptance. Missing proof and unresolved failures remain explicit; qualification is not satisfied merely by moving its deadline. |

S1 owns scheduler/surface generations, redraw/configure/acquire/submit/present
counts and changed surface-resource lifetime accounting. S2 owns retained
geometry, draw/upload, shaped-text/cache and allocation-capacity accounting.
S3 owns dialog/control geometry, text/pass counts and control-cache accounting.
S4 owns terminal/glyph uploads, hidden work, fairness and remaining all-host
recovery/teardown accounting. Each bounded patch records production consumers
migrated and competing paths retired; partially migrated hosts remain explicit.
Counters needed to prove that patch's invariants are implemented and validated
with the patch, before its result is accepted. Complete cross-component metrics
are not prerequisites to an unrelated component migration. Existing validated
observations and matched before/after state accompany every slice; unexplained
regressions still block advancement. A numerical claim requires the unchanged
MET/GPU/ACC/STAT methods, overhead and complete admission/accounting for that
claim. S5 requires all outstanding in-scope method, admission and accounting
proof plus all adoption, independent replay and endurance requirements.

S0 chooses the first bounded optimization from measured attributable work and
correctness defects; the table is dependency planning, not a claim that local
window invalidation has already won that comparison. No framework replacement,
new dependency, extra render thread, reduced AA or backend policy change follows
from this contract. The first execution gate must record the selected slice and
its prerequisites; later slice selection follows the canonical roadmap.

## Global adoption and retirement

All 14 ADP-01 consumers are enumerated in the map. Every consumer inherits shared
host scheduling, lifecycle generations, resource accounting and common clipping.
Main panes share one native surface; they do not receive individual swapchains.
Auxiliary hosts use the same coordinator/device owner and distinct host state.
The map names current adapter paths and specific competing paths to retire.

Completion requires a production call-site inventory showing every native
`request_redraw`, surface configure/acquire/present, queue submission, attachment
allocation and close callback routed through the shared ownership boundary.
Allow native compositor notification at the actual presentation point; it must
not become an alternate scheduler. Search and inspect call sites, exercise each
one through production tests, and record the exact remaining low-level calls
and why each is inside its owner. Merely adding an unused shared helper fails.

Preferences retain continuous offset and clipped hit regions. Terminal history
and Layers retain discrete rows; they reuse conversion/clip/capture primitives
where semantics agree. Revision, navigator and Inspector remain partial products:
qualify their existing behavior, without claiming absent authoring/navigation.
Resolved schematic performance remains unqualified until a real fixture is
admitted. Future panes must register host/profile, invalidation dependencies,
complete cache ownership/caps, input/focus/clip semantics and positive/negative
proof before admission. An exception needs a semantic reason and evidence;
private generic scheduling/rendering/scroll machinery is not an exception.

## Execution and acceptance boundaries

The owner authorized the amended existing S0–S5 contract on2026-09-20. Start
with bounded S0 readiness and selection under the amended GPI-S0 clause, preserving the recovered
candidate checkpoint c8bac0b6 as the implementation starting point. No deletion
or restart follows from incomplete compliance. Record bounded improvements,
diagnostic behavior, implementation gaps and any evidence-supported proposed
specification correction in the existing adoption map before consolidation.
GPS-C01R remains separately pinned and nonblocking; unresolved driver attribution,
resize budgets and deferred temporal qualification do not gate unrelated work.

<!-- EVIDENCE:GUI-PERFORMANCE-IMPLEMENTATION:OWNER-EXECUTION -->

Exact owner response: `GUI-PERFORMANCE-IMPLEMENTATION: authorize amended S0-S5 contract`.
This resolves GPI-GRANT. It authorizes the existing bounded slices, not production
acceptance, dependency adoption, backend policy or changed performance thresholds.
The preceding checkpoint/review instruction alone was not treated as this grant.

Final independent performance replay must use the same pinned candidate,
fixtures, environment and required methods, with a reviewer distinct from the
implementer actually rerunning the trials. A report-only review cannot pass it.
Initial acceptance, if later granted, prints all scope limitations beside the
results. Larger-project, resolved-schematic and temporal/display qualification
remain explicit future obligations; resize QA remains open until its complete
resource and temporal criteria pass. No full enterprise-readiness claim follows
from this limited scope.

## Prepared execution completion markers

The frozen `prepared-implementation-completion.json` records the reviewed draft.
Its live completion contract is now in `specs/active_frontier.json`, amended by
the owner's nonblocking resize disposition. Activation normalizes requirement
marker fields to step IDs, supplies the owner marker below, and preserves all
six existing downstream implementation dependencies. The frozen preparation
record remains history, not a competing live contract.

<!-- REQ:GUI-PERFORMANCE-IMPLEMENTATION:GPI-GRANT -->
<!-- OWNER:GUI-PERFORMANCE-IMPLEMENTATION:GPI-GRANT:GPI-GRANT -->

**GPI-GRANT.** Approve an exact bounded execution scope against the ratified specification, admitted fixture/method prerequisites and measured priority; no automatic renderer execution. Resize investigation is nonblocking under the owner amendment; its qualification remains separate.

<!-- REQ:GUI-PERFORMANCE-IMPLEMENTATION:GPI-S0 -->

**GPI-S0.** Pin the matched baseline and first bounded S1 production migration; verify its input/final-state and measurement prerequisites, document attributable selection, and assign remaining ADM/GPU/ACC proof to the components below. Reconcile canonical slice order before any different selection. Completion establishes bounded implementation readiness only, not full measurement closure.

<!-- REQ:GUI-PERFORMANCE-IMPLEMENTATION:GPI-S1 -->

**GPI-S1.** Adopt shared surface scheduling/lifecycle in every default native host and establish bounded implementation readiness under the S1 table. Preserve unfinished SH/LF and affected HP qualification individually for S5; do not declare global acceptance. RS and resize-budget qualification remain pinned and nonblocking.

<!-- REQ:GUI-PERFORMANCE-IMPLEMENTATION:GPI-S2 -->

**GPI-S2.** Adopt retained geometry/text/encoding/upload ownership and establish bounded shared-implementation readiness under the S2 table. Preserve unfinished implementation explicitly for S4 and complete qualification for S5; neither readiness nor partial cache proof establishes resource-budget acceptance.

<!-- REQ:GUI-PERFORMANCE-IMPLEMENTATION:GPI-S3 -->

**GPI-S3.** Adopt shared dialog/control/scroll/clip/hit owners in default production consumers, retire New Project backdrop and competing paths, and establish bounded implementation readiness under the S3 table. Preserve unfinished qualification individually for S5; do not declare global acceptance.

<!-- REQ:GUI-PERFORMANCE-IMPLEMENTATION:GPI-S4 -->

**GPI-S4.** Finish terminal fairness/hidden work and all-host recovery/resource adoption; preserve separate concurrent multi-device failure. Complete the unfinished S2 implementation predicates carried by the S2/S4 tables before advancing to S5.

For private font selection/loading/cache and raster construction only, the S4 implementation prerequisite is the shared MEM-02/ACC-02 call guard, correct simultaneous local/process accounting, and default production refusal/release/retry integration with affected positive/negative proof. A hard pre-call bound on these opaque internal allocations is no longer required. Every other unfinished S2/S4 implementation predicate remains due before S5. This change supplies no completion evidence by itself.

<!-- REQ:GUI-PERFORMANCE-IMPLEMENTATION:GPI-S5 -->

**GPI-S5.** Run complete in-scope adoption/native UX/endurance and independent performance replay; publish all excluded scope without claiming resize closure.

Qualification must demonstrate that private text construction actually stays within the unchanged numerical memory limits for every admitted configuration and required workload, including cold load, cache churn, simultaneous hosts and recovery. Any observed overrun, missing instantaneous/peak accounting or required content that cannot fit fails the tier; the guard detecting it does not satisfy qualification. S4 guard implementation is neither a memory-budget pass nor protection against every possible opaque allocation.

## Owner amendment: resize investigation is nonblocking

The subsequent owner instruction in
`docs/reviews/gui-performance/resize-deferral-owner-direction.json` pins
`dat-gui-vertical-resize-cpu-toj` as unresolved investigation and removes resize
resolution, kernel/driver attribution and completed temporal calibration as
development gates for S0–S5. This amendment supersedes earlier E10 pre-change
calibration requirements and GPS-C01R blocking sequence. It does not mark resize
budgets, flicker or displayed-frame criteria passed or change their numerical
limits. The Linux-kernel/Intel-driver/hardware explanation is an unconfirmed
owner hypothesis, not an established root cause.

Shared application lifecycle and reuse fixes may proceed under the separate
execution grant, with bounded relevant production regressions, available
before/after resource observations and final-state/extent/hit/focus/PTY/static
correctness checks. Missing temporal proof is reported as unqualified. It cannot
be used to claim resize closure, but it does not stop other development.
All non-resize performance/correctness, adoption, independent replay and
endurance requirements remain. S5 can conclude only its explicitly limited
non-resize qualification; full resize resource/temporal closure stays on the
pinned issue. No new kernel investigation or renderer execution is authorized
by this amendment.

## Owner amendment: component-scoped measurement delivery

<!-- EVIDENCE:GUI-PERFORMANCE-IMPLEMENTATION:SEQUENCING-APPROVED -->

On 2026-09-20 the owner approved the presented sequencing delta and directed
completion of the five implementation slices:

> your next /goal is to complete the next 5 steps as you have presented them. this is your approval of this sequescing amendment. please proceed.

This approves the bounded S0 readiness and S1–S4 component proof allocation above,
followed by S5 comprehensive in-scope qualification. It supersedes the earlier
all-host S0 accounting prerequisite, including that reading of GP-045-05,
E08 and ADM-01. It does not complete S0 or any implementation slice. The existing
execution grant remains in force. All numerical criteria, affected production
proof, independent replay, exclusions and owner-only dependency authority remain.
Frozen specification approval packets and earlier receipts remain historical.

<!-- EVIDENCE:GUI-PERFORMANCE-IMPLEMENTATION:S0-BOUNDED-READY -->

Bounded S0 readiness is recorded in
`docs/reviews/gui-performance/implementation/S0/readiness/receipt.json` and the
existing adoption map. The fresh offline release build reproduces the prior
binary exactly; four Xwayland production host configurations pass native extent
input/final-state checks. A fresh Wayland smoke attempt did not start its resize
sequence and remains failed/unqualified. Selection is the demonstrated redraw
ownership/damage-retirement gap, not a numerical performance claim. S1 begins
with shared redraw tokens/generations; full ADM/GPU/ACC, S1 adoption and S5
qualification remain outstanding under the approved component allocation.

## Owner amendment: S1 implementation exit

<!-- EVIDENCE:GUI-PERFORMANCE-IMPLEMENTATION:S1-EXIT-APPROVED -->

On 2026-09-21 the owner approved the exact four-edit proposal reviewed in commit
`1e4d2e2f`:

> Approve the proposed S1 exit amendment

The S1 table and GPI-S1 now define bounded shared-implementation readiness;
unfinished prescribed negative-control, native final-state/static/input/focus,
backend/scale and full resource/method qualification explicitly remain S5 exit
prerequisites under the amended S5 table. The Frontier mirrors this wording.
Each partial map row and original evidence receipt remains intact. No unfinished
requirement becomes passed through this amendment.

Approval changes when that qualification must be complete. It does not change
numerical thresholds, default-path adoption, PM029 dependency authority, product
semantics, independent replay, endurance, the resize exclusion or final acceptance.
An evidence-based readiness review must precede S1 completion and S2 selection;
this owner disposition does not itself complete S1.

<!-- EVIDENCE:GUI-PERFORMANCE-IMPLEMENTATION:S1-BOUNDED-READY -->

Bounded S1 readiness is recorded in
`docs/reviews/gui-performance/implementation/S1/readiness/result.json` and the
existing adoption map. Current default call-site inventory, all four native
host adapters, retired competing paths, shared-owner correctness and negative
proof, and implemented generation/submission/attachment-lifetime accounting
satisfy the approved implementation exit. The 30 initial-scope S1 rows retain
their partial test statuses and now name GPI-S5 as their qualification exit.
Four pinned resize rows and one deferred temporal HP row retain their exclusions.
No complete row qualification or performance acceptance follows. S2 may begin
through the synchronized Frontier transition; S3–S5 remain pending.

## Owner amendment: S2 implementation exit

<!-- EVIDENCE:GUI-PERFORMANCE-IMPLEMENTATION:S2-EXIT-APPROVED -->

On 2026-09-21 the owner approved the exact six-clause proposal in commit
`74ba0b78`:

> Approve the proposed S2 amendment

The S2 table and GPI-S2 now require bounded shared-implementation readiness.
Unfinished accounting/cap, exact changed-range transfer and shaping/layout
implementation remains required at S4; complete qualification remains required
at S5 under the amended tables and GPI-S4. The Frontier mirrors these actions.
The adoption map records the outstanding predicates individually; their original
acceptance rules, statuses and receipts remain unchanged.

An evidence-based S2 readiness reconciliation must precede S2 completion and S3
selection. This approval itself completes neither. The approval transaction kept S2 selected and in progress pending that
reconciliation, subsequently recorded below.

The amendment changes sequencing, not numerical budgets, dependency authority,
product behavior, independent replay, endurance or the nonblocking resize scope.
It does not supply a technical solution for private resource accounting. The
installed public wgpu allocator report can observe atlas backing allocations on
the tested Vulkan backend, but snapshots do not establish stable allocation IDs,
upload events, CPU owned capacities or complete ACC-01/02 qualification.

## S2 bounded implementation readiness

<!-- EVIDENCE:GUI-PERFORMANCE-IMPLEMENTATION:S2-BOUNDED-READY -->

The current production inventory, retired paths, explicit partial limits and
bounded positive/negative/native proof are reconciled in
`docs/reviews/gui-performance/implementation/S2/readiness/result.json` and the
existing adoption map. All four default native hosts consume the shared retained
resource and text owners. The amended S2 readiness exit is satisfied; GPI-S3 is
selected for shared dialog/control/scroll/clip/hit migration, including retirement
of New Project's remaining general backdrop.

The 50 individually carried predicates remain unqualified. Complete S2-owned
accounting/cap, exact changed-range and shaping/layout implementation is required
at S4; full qualification remains required at S5. No full byte-budget, native
backend/scale, independent replay, endurance, schematic admission or resize
acceptance follows from this bounded readiness disposition.

## Owner amendment: S3 shared-resource dependency clarification

<!-- EVIDENCE:GUI-PERFORMANCE-IMPLEMENTATION:S3-RESOURCE-APPROVED -->

The owner approved the exact pending clarification in commit `1bc02aaf`:

> Approve the exact S3 clarification

The S3 table's required-evidence cell now applies that exact replacement. Only
complete S2-owned resource accounting/cap dependencies of HP14-01, HP14-02 and
HP16-03 move to S4 implementation and S5 qualification. The existing adoption
map records their individual MEM associations, retaining original acceptance
rules and partial statuses. All other S3 functional/native/negative proof remains
required before S4; a separate evidence-based exit review must precede selection.

This approval does not complete S3, qualify resource bounds, solve private
resource accounting, authorize dependencies, change numerical budgets or alter
the nonblocking resize exclusion. S3 remains selected and in progress.

The subsequently approved S3 bounded implementation exit supersedes only the requirement that all remaining S3 functional/native/negative qualification precede S4. Those obligations now precede S5 completion; their original acceptance rules remain unchanged. The earlier S4 resource implementation deadline remains binding. A separate evidence-based readiness review still precedes S3 completion and S4 selection; approval alone completes neither. Unexplained implementation regressions remain blocking. The historical approval and failed evidence are preserved.

## Owner amendment: S3 bounded implementation exit

<!-- EVIDENCE:GUI-PERFORMANCE-IMPLEMENTATION:S3-EXIT-APPROVED -->

The owner replied `proceed` to the explicit request to approve the exact
four-clause amendment in commit `ccf42618`. All four edits are applied: the
S3 table and GPI-S3 define bounded implementation readiness, the S5 table
requires every carried qualification predicate, and the earlier resource-only
clarification is explicitly superseded for qualification timing. Its S4
resource implementation deadline remains unchanged.

The existing adoption map retains all 20 S3 rows and their original acceptance
rules, partial statuses, failed evidence and remaining actions, assigning
unfinished qualification to S5. No missing production adoption or unexplained
implementation regression is deferred. Approval alone does not complete S3;
an evidence-based readiness review must precede S4 selection.

No dependency, prototype, numerical budget, independent replay, endurance or
nonblocking resize boundary changes. Reuse bounded evidence for unchanged
code/scenarios; check affected regressions after relevant changes.

## S3 bounded implementation readiness

<!-- EVIDENCE:GUI-PERFORMANCE-IMPLEMENTATION:S3-BOUNDED-READY -->

The separate determination in the existing adoption map's
`s3_bounded_exit_review` establishes the approved S3 implementation exit.
Default dialog/control/scroll/clip/hit consumers, New Project backdrop and
competing-path retirement, affected positive/negative correctness tests,
bounded native adapter evidence and bounded warm reuse/eviction are recorded.
The reviewed source inventory is unchanged from the proposal; the convergence
guard passes. No missing default adoption or unexplained implementation
regression was identified. Known strict native-redraw, Console golden and
historical input/capture failures retain their explicit scoped dispositions;
no failed receipt is erased or relabeled passed.

GPI-S3 is complete for bounded implementation readiness and GPI-S4 is selected.
All 20 S3 qualification rows retain their original partial statuses and are
required at S5. All 50 carried S2 implementation predicates remain due at S4.
No full HP-row, resource-budget, global adoption, product or resize acceptance
follows. The overall implementation issue remains open.
