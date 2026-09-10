# Infrastructure delivery contract: local evidence review

Task: WORKFLOW-DELIVERY-IMPLEMENTATION / WDQ-I02.
Issue: `dat-wdq-rollout-implementation-ffy`.
Owning route: `workflow-delivery-infrastructure`.

This is a review of Datum's existing implementation and approved rollout scope,
not external product research, mechanism ratification or a proof result.

The approved R03 packet requires the infrastructure lane to consume the same
readiness and separate independent-review discipline it proposes for product
lanes. The explicit mapping proposal names WDQ-READY, WDQ-I03 and WDQ-REVIEW;
I04 remains owner-controlled activation and I06 remains full rollout acceptance.

The actual consuming entry points are `main` in
`scripts/check_workflow_delivery.py` (staged and exact-candidate CLI modes),
and `selector_failures` in `scripts/workflow_delivery_selector.py` (called
by the real project-status selector). They read trust, snapshots and delivery
records and produce refusals; they do not implement EDA editing. Function-qualified
identities in this contract describe those Python entry points, not new Datum
tool-registry verbs or a claim that a production telemetry emitter exists.

The prepared candidate at `a5f3d219f8a4f43e3509cc1d89d38033313226ab`
adds coverage, source-scope, specification-clause and distinct-review checks
at these entry points. Its 223 passing workflow tests are producer-side
regression evidence, not independently replayed infrastructure delivery or
product evidence. Fixture factories can construct hostile repositories, but
their fabricated proof/event helpers cannot supply the actual delivery record.

Complete technical consumers of this source are the infrastructure specification,
its machine contract and the proposed PM042 amendment. Their shared reviewed
digest binds the real normative mechanism. Operational progress, proposed
checkpoint mapping, proof, independent review and owner receipts remain outside
this authority route. Otherwise appending the review result would change its
own authority hash. PM042 is also a consumer of the rollout-planning route;
changing it requires reconciling both routes. This separation does not permit
an implementation or progress note to override either normative contract.

The input list is an explicit candidate-source inventory, including production
entry-point imports and existing workflow test recipes. It is not a claim that
all candidate modules are installed in main. Before readiness, reconcile it
against the assembled candidate, actual fixture recipe, environment and all
producer dependencies. Excluding arbitrary caches is necessary; excluding a
real runtime input to preserve an old hash is forbidden.

This bounded contract establishes read-only validator behavior only. Real
owner-hook and controlled-promotion success, refusal and interruption proof
remain additional I03/REVIEW obligations. Three native product cohorts and the
Preferences lane retain their own authority and owners. No infrastructure test
answers missing CAD units, numeric entry, selection, snapping, connectivity or
native persistence specifications.

## WDQ-READY — mixed-environment evidence finding

Direct inspection of the CLI and selector in assembled candidate `beddc2ef`
shows one requested environment object passed to every enrollment. The existing
validator requires exact object equality with each recorded proof environment,
including reproduction commands and toolchain. The accepted pilot records
Rust/X11/XTest; the observed infrastructure process uses `/usr/bin/python3`,
Python 3.13.5, and pipes with neither stdin nor stdout a TTY. A read-only check
confirmed unchanged pilot environment success and WDQ-ENVIRONMENT refusal when
the requested toolchain was replaced with this Python version. This is a
diagnostic, not a second workflow proof. Bead: `dat-wdq-environment-scope-kgq`.

The proposed PM042 environment section separates explicit owner-selected
per-enrollment requests from the proof being judged and provides an honest
infrastructure-only headless representation. Legacy pilot bytes and exact
equality remain controlling for the existing installation. A shared request
cannot become a blanket equivalence waiver, and absent display data cannot be
filled with invented positive window dimensions. The proposal needs candidate
implementation and real mixed-enrollment negative/replay proof before activation.
No environment selection instance, new observed proof or owner approval is
created by this review. Full fixture and runtime input closure remain separate
readiness obligations; a binary hash alone does not describe Python dependencies.

## WDQ-READY — fixture and capture review

The inspected coverage-entrypoint suite calls the real main function under
redirected stdout and the selector helper directly. Its Fixture snapshot excludes
Git metadata; accepted_fixture deliberately fabricates native evidence and owner
receipts as validator INPUT. These are valid refusal-test techniques, not the
real subprocess, complete protected-state or independent delivery observations
required by INFRA-S01..S06. The infrastructure procedure now makes that boundary
explicit and requires both a complete real-roadmap case and isolated hostile
inputs, retained restoration parameters, actual invocations and fresh replay.

The currently installed owner hook was inspected read-only. In addition to the
WDQ runner it invokes file-lane and rustfmt gates; the latter reads the exemption
manifest and staged source. It also uses Git, bash and realpath. Those dependencies
are not established by the earlier Python-only import audit. Its current
outside-worktree runner restriction is a baseline observation, not compatible
repository-local support implementation. I03 must verify the reviewed replacement
and complete its input closure; this review does not modify or bypass that hook.

No fixture archive, capture instrumentation, producer event or independent replay
was created during this planning review. The procedure is the implementation
contract for those outputs, not evidence that they exist. Full subcase inventory
and source/toolchain closure remain necessary before readiness can be asserted.

The subsequent case review expands the six normative scenario groups into 62
stable case rows. Each listed variant and entry surface needs its own observed
record; the count is not a number of passing tests. The matrix distinguishes
real candidate history from staged/worktree views and preserves early hook
diagnostics. Two consistency tests check identity, scenario/surface coverage and
snapshot applicability only. They do not certify semantic completeness or an
unexecuted refusal. The full fixture capture and final input closure remain due.

## WDQ-READY — repository read inputs versus write authority

Inspection of the actual hook and rustfmt loader identified seven concrete
repository dependencies outside the original 69-file list: the file-lane and
rustfmt checkers, the exemption manifest, and its four existing Rust paths.
The contract now binds those 76 inputs conservatively. The Rust paths are checked
for existence and can be examined as staged formatting inputs; including their
bytes avoids an understated proof boundary. No Rust source was changed.

At that readiness snapshot the permission proposal stayed at 69 paths. A test previously
required equality between read inputs and write scope; it now checks the proper
subset relationship, and a negative scope test prevents these seven dependencies
from becoming workflow write permissions. Separate input coverage tests derive
the current exemption references rather than hiding omissions behind a fixed count.

Read-only review of drift/CI wiring also distinguishes supplementary full-build
verification from bounded CLI/selector/hook observations. External tools and
actual loaded Python/shared-library inputs require separate observed toolchain
accounting. Future capture/fixture modules and environment files must be added
when implemented before proof; this 76-file baseline does not certify those
not-yet-existing files, completed readiness or full installation.

## Construction/readiness sequencing correction

The added readiness checkpoint exposed a workflow-plan loop: it demanded final
capture/runtime inputs whose construction was scheduled only afterward in I03.
The original R03 authorization already includes those workflow validators,
fixture/capture support and promotion tools. WDQ-TOOLS now names that bounded
preparatory implementation before final readiness. I03 retains actual producer
verification after readiness; distinct replay and I04 activation remain later.
The prior readiness review work is retained, but readiness stays pending rather
than being marked complete from procedure prose or an observer inventory.

This is an internal sequence correction, not a product-lane takeover, scope
reduction, independent-review waiver or owner ratification. Preparation remains
in the repository-local isolated candidate until exact owner promotion. The
delivery mapping is unchanged; TOOLS is not a substitute mapped checkpoint.

## WDQ-TOOLS — owner-hook consumer reconciliation

The normative case matrix already requires H, but the machine validator consumer
declared only staged_cli and candidate_cli. Review of the prepared hook at
`d56e113deea8b39600509285e392cdce1a3ddda4` confirms that its authenticated
bootstrap reaches the existing check_workflow_delivery.main handler. The machine
contract now declares owner_hook on that consumer; no new dispatch verb or
product behavior is inferred from runtime code.

Normal hook observations must bind the real validator call. Early shell trust
and prerequisite refusals retain their actual diagnostic without inventing a
downstream call or JSON result. The optional authenticated observer does not
replace enforcement, complete subprocess/toolchain closure or authorize promotion.
The existing 62-row matrix, exact input reconciliation, independent replay and
owner activation remain required. The 76-file inventory remains a preparation
baseline, not the final capture-source closure. No installed hook is changed by
this reconciliation, and no readiness or delivery result is asserted.

## WDQ-TOOLS — implemented Python input reconciliation

Inspection at candidate `e96b0985c6fef52d133a3b6f6f037942f5b07e99`
found 66 implemented workflow Python modules/tests missing from the 76-path
input baseline. They cover capture, environment selection, authenticated startup,
support storage, snapshot history and promotion diagnostics. The machine input
list now includes them and the three existing rollout contract/mapping/proposal
tests: 145 explicit paths. A recursive local Python import check guards this
bounded inventory against recurrence; static imports cannot establish dynamic
loads, shell children, external runtime libraries or complete observed closure.

That input-only increment deliberately left source permission unchanged. Reading/hashing these
inputs does not authorize publishing them under a stale scope or an owner-decision
step. The separate publication-delta obligation remains open. Existing scenario,
foundation, consumer, independent-review and owner-activation requirements are
unchanged; PM042 is not ratified by this input reconciliation. Older snapshots
and failed captures retain their old identities rather than being refreshed.

## WDQ-TOOLS — separate source-permission reconciliation

Review of the complete parent-edge history from main `7a77e40a850172b50e2251132d171d55a8adfb37`
to candidate `701c3b82363973857c47d16f46a8baa0a597c008` identifies implemented
workflow files omitted from the old scope. The reviewed proposal now retains
the original 69 permissions and adds 68 final-diff script paths, the historical
rollout mapping test change and the coverage test: 139 exact files, not roots.
The coverage test is also an explicit read input, bringing that inventory to 150.
The original seven hook/formatting read-only dependencies remain outside scope.
Environment data and unrelated product/prototype files receive no permission.

The candidate policy and proposal must agree. A fixed-history regression checks
all production paths from every parent edge, including the mapping-test change
absent from the net diff. This records preparation consistency, not owner
ratification, current execution authorization or permission for future history.
The I04 owner-decision/current-execution-claim mismatch remains a separate
controlled-promotion obligation; no claim exemption or live trust is introduced.

### Initial owner-publication inspection boundary

Inspection of the actual source gate confirms that pending owner decisions
cannot hold ordinary execution permission. The proposed PM042 reconciliation
therefore separates read-only publication inspection from current transactions:
both base and candidate await claim-free I04, every prior tooling/readiness/
producer/reviewer checkpoint is complete, other Frontier items are unchanged,
and the complete history fits the one exact workflow scope. Shared coverage
state validation retains ordinary source-claim enforcement in its original
caller. Publication inspection additionally demands actual rollout review
evidence rather than relying on completed labels or a path-list comparison.

The explicit stronger inspection mode still cannot activate anything. A separately
selected response digest establishes exact-byte consistency, not owner identity.
Successful complete-packet observation, actual coordinated publication, recovery
and installed verification remain due; development boundary fixtures do not
satisfy these requirements or close dat-wdq-promotion-delta-mb9.

The subsequent shell/data review at `e16f3f0fea546a13922d72313f2ffc779fc5c392`
adds four concrete inputs beyond that Python inventory: the authenticated owner
hook shell source, explicit selection document, and both environment Blobs named
by that document. The resulting 149-path inventory preserves accepted pilot bytes
and keeps environment evidence outside its own normative authority route.
An input regression follows these exact selected data references and checks their
hashes; it does not select environments from proof or widen source permissions.
Observed executable versions remain distinct from actual hook/child invocation
and shared-library closure. Readiness and producer verification remain incomplete.

## WDQ-TOOLS — bounded activation construction

The prepared runtime now has a separate owner-invoked activation command. Its
implementation and synthetic integration test are two explicit additions to
both read inputs (152) and proposed permissions (141), not a broad root grant.
It authenticates startup before adjacent implementation imports, requires the
exact response and writer-pause acknowledgement, serializes cooperating
activators, and reruns full promotion and defect-disposition validation before
publication. Existing final-acceptance defect checks are shared without changing
ordinary review's ability to report findings awaiting owner disposition.

The command journals each boundary, fast-forwards, installs hooks last and runs
actual installed gates. Synthetic repositories exercise success, unchanged-state
refusals, a live competing lock and interruption after an actual config write.
No such command has been run on Datum main. Those observations are development
regressions, not actual I03/reviewer evidence, owner ratification, installed
rollout or adoption. Full runtime closure and real interruption/recovery review
remain outstanding; retained partial state is never silently repaired.

### Observed live-child interruption handling

The activation child runner now defers handled SIGINT/SIGTERM, retains the lock,
records actual child identities/results and stops before any subsequent mutation.
Real synthetic Git/post-merge children demonstrate both signal cases: the child
is confirmed live when signalled, the lock is still held, and releasing that
child leads to a terminal result and honest partial publication with old trust.
A bounded-timeout test kills and reaps only the owned process group. A separate
parent-SIGKILL test observes inherited lock retention until its child exits.

These tests do not claim actual Datum activation or the complete crash/recovery
matrix. They replace the earlier unproven handled-signal construction gap, not
the required final runtime closure, exact full producer run, distinct replay,
owner decision or three native adoption outcomes. No new permission/input path,
dependency, prototype or other product lane is changed.

## Owner-approved bounded verification — 2026-09-08

The owner approved replacing exhaustive runtime tracing with pinned tool/source
identities, state-preservation checks, behavioral tests and independent review,
without weakening product-evidence requirements. The exact question and response
are retained outside proof authority in
`docs/reviews/workflow-delivery-rollout/verification-boundary-owner-20260908.json`.

The infrastructure specification and PM042 now define input completeness within
that bound. Historical references above to full descendant/shared-library closure
do not create an additional requirement beyond the revised normative boundary.
Existing interpreter module/maps and child observations are preserved as
supplemental evidence, with unobserved closure still honestly unclaimed. The
tradeoff is explicit: this is not hermetic runtime verification or detection of
arbitrary host-library substitution. Source/data/fixture inventory, every named
tool actually used, the case matrix, independent replay, product/native evidence,
and exact I04 owner activation remain required. This approval does not ratify
the complete PM042 activation packet, complete a checkpoint or approve a product.

## WDQ-COMPAT — preserved workspace and proof-input review

The actual final-publication inspection exposed a workspace mismatch: broad
ignored-input enumeration refused the unchanged owner GUI runner and ordinary
Python caches, and publication preflight also refused Beads runtime. The owner
authorized bounded repair, not cleanup, activation or new product authority.
Read-only inventory at main `b726cb6972148a6727971eed3e38066af543d4b3`
found 205 named Beads runtime files, 227 byte-verified current-source caches,
23 unclassified caches and the unchanged local GUI script. These observations
identify review inputs; they are not permission to exclude them.

The proposed PM042 workspace section separates exact owner-pinned local state,
derived runtime classification and actual proof closure. Source-only gate
startup prevents cache consumption by supported fresh gate imports; it cannot
establish irrelevance to independent product processes. Distinct inspection
identified two defects in the developing candidate: snapshot/live substitution
and proof exclusion based only on workspace eligibility. The corrected design
binds exemption bytes to captured state and retains local files, legacy payloads
and Beads data inside declared proof roots. Only verified-source cache output
with its source in the same proof closure can be omitted there.

Normal cache lifecycle is distinct from required owner files: genuinely absent
legacy caches or fully verified current-source regeneration do not imply source
mutation permission. Malformed replacements and false absence still refuse.
The implementation's regression tests support this design but do not replace
the newly required actual compatibility observations, final input/permission
reconciliation, distinct replay or exact I04 owner ratification. No owner cache,
local script, Beads runtime, prototype or Preferences record was changed to
manufacture compatibility. All native adoption obligations remain intact.

## WDQ-COMPAT — initial mapping prerequisite correction

The owner-approved sequencing repair also covers the initial mapping handoff.
At main `2793959cecd02a20037e87f8c3f63aa7e340a899`, the installed schema-1 policy
enrolls only the accepted pilot and the rollout Frontier has no delivery field.
Requiring that baseline to already contain the candidate's schema-2 mapping is
an invalid initial-publication prerequisite. The prepared correction permits
only genuine absence on that valid legacy boundary, preserves prior enrollment
records and legacy-baseline identity, and requires the exact renewed candidate
mapping. Present null/malformed mappings and removed or changed enrollments
remain refusals. Both views still require completed predecessors and claim-free
I04 for strict inspection/activation. This correction alone does not resolve
review-time inspection sequencing, complete proof renewal or activate anything.

## WDQ-COMPAT — break the review/publication evidence cycle

Inspection of the prepared call graph showed two circular prerequisites:
strict promotion demanded completed RECHECK and its independent-review packet,
while RECHECK and INFRA-S05 group 8 required that inspection as input. Merely
requesting ready instead of review was insufficient: trusted readiness escalates
changed inputs to proof. The bounded correction therefore separates source/history
and readiness-prerequisite inspection from final evidence inspection, not merely
the requested phase inside the activation path.

The separate review inspector preserves matching live COMPAT/RECHECK lifecycle
and claims, all other lanes, exact history/scope, contract authority, selected
environment, support integrity and protected-state equality. It does not demand
future rollout proof, but other enrollments and ordinary trusted readiness keep
their existing proof requirements. A distinct closed request and explicit false
evidence/activation/publication flags prevent its result from being an activation
receipt. Final strict inspection remains mandatory after review completion and
explicit reconciliation of the later evidence/governance delta. Independent
source review found no concrete blocker in this separation; synthetic tests are
development evidence only, not fresh producer or independent replay of the real
rollout. Follow-up review identified ambiguous producer/independent packet timing:
COMPAT records producer inspection first; RECHECK later repeats it independently
for its own replay/review. Neither requires the later I04 inspection as input.
This distinction preserves producer-before-review ordering and avoids replacing
the original cycle with a cross-phase one. No installed trust, product authority
or adoption obligation changes.
