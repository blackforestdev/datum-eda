# Broader delivery enforcement — bounded implementation proposal

Status: WDQ-R02 proposal for WDQ-R03 owner review. Not installed or ratified.
Issue: `dat-wdq-broader-rollout-tqh`. Route: `workflow-delivery-rollout`.
Baseline inspection: `327b71d1`; refresh exact candidate/base before promotion.

## 1. Outcome, authority and implementation boundary

Implement a coverage refusal around the existing PM041 delivery validator,
separate independent replay from owner acceptance, and prepare explicit
production enrollment plus measured adoption. Passing the pilot, creating three
task names, or writing this plan does not complete the reform.

The requested authorization covers workflow validators, their tests, governed
schema/decision reconciliation, delivery-contract preparation and owner-run
promotion tooling. It does not cover another session's product implementation,
Claude HTML, numerical performance budgets, dependency selection or product
acceptance. PM025 remains the only selector and claim authority. Existing
product prerequisites remain in force. A change to the adopted PM041 mechanism
must be drafted and presented as a numbered-decision amendment for exact owner
ratification before promotion, not silently treated as already-ratified prose.

Implementation sequence: (I1) coverage and transition tests; (I2) review-phase
separation and exact contract/enrollment candidates; (I3) controlled promotion
package and independent gate review; (I4) owner activation; (I5) three real
adoption slices with per-lane proof/review/acceptance; (I6) owner rollout review.
Register these as a successor completion plan after R03 disposition. I3 and I5
need a distinct reviewer/session; no reviewer has been recruited by this plan.
Missing independent review stays incomplete. I4/I6 are explicit owner boundaries.

## 2. Coverage must be checked, not opted into by candidate code

Propose policy schema 2, retaining schema-1 pilot compatibility until exact
owner promotion. Add a closed `coverage` object to the existing policy:
`baseline_ref` (full commit ID), `rows`, `source_scopes`, `production_roots`
and `new_item_rule` (literal `classification_required`). Each row has exactly `frontier_key`, `issue_id`,
`category`, `boundary_ref` and `external_handoff_ref`. Categories are
`historical`, `specification`, `product`, `infrastructure`, `deferred` and
`external_lane`. References use existing `{path, marker}` semantics;
`external_handoff_ref` is null except for a documented external-lane boundary.
The external boundary records the exact selected completion step and permitted
transition/path scope; it is not a session-name exemption.

At promotion every Frontier key has exactly one row. Identity, classification
and exceptions are pinned in the trusted policy, not changed by an ordinary
candidate. Missing/duplicate keys, removed classifications, a new unclassified
item or a candidate changing category refuse with `WDQ-COVERAGE`. Proposed new
work can be prepared without activation; its classification lands only through
the reviewed governance/promotion transaction. The policy never stores a second
status, assignee or next-step selection.

Classification alone does not catch a source edit hidden behind pending labels.
`production_roots` is an owner-reviewed set of code/build/GUI resource paths,
initially the conservative build closure in section 4. Every changed path under
those roots must match at least one trusted `source_scopes` row. Each scope has
exactly `frontier_key`, `step_ids`, `paths` and `boundary_ref`; paths are normalized
files/directories and step IDs are existing authorized implementation steps.
The named task must have a current synchronized PM025 claim and authorization
for that step. A product scope requires its promoted delivery contract;
infrastructure requires its bounded consuming-workflow contract; external-lane
work requires its exact coordinated boundary. Uncovered changes refuse; an
ordinary candidate cannot add its own scope, contract or exception. Scope
records are permissions tied to existing state, not a second selector/lease.
All affected already-enrolled input closures still require proof, even when
another permitted scope owns the edit. Source scopes for future unready work
are not installed as permission to enable it. New implementation outside a
current scope first needs the bounded owner-reviewed promotion transaction.
This verifies authorized paths and evidence, not the honesty of an agent's
claimed semantic intent; independent review remains necessary.

Required checks by category:

- Historical: landed at the exact coverage baseline; retain existing proof
  obligations, including the pilot. Do not claim retroactive native acceptance.
  Reopening or adding execution requires reclassification, not the old exemption.
- Specification: complete structured plan, governed requirements, reviewed
  source/consumer route and explicit owner decisions for unresolved mechanisms.
  Before completing authorship require artifact references, route freshness and
  the complete clause/disposition matrix. Refuse execution authorization or an
  execution step under this classification; a successor needs product or
  infrastructure classification. Mere document existence cannot ratify intent.
- Product: require the complete delivery declaration before readiness completion
  or execution authorization. Require owner-promoted enrollment before enabling
  source changes or activation; refusal must name the missing contract/consumer
  and responsible lane. Planning can retain honest unresolved questions.
- Infrastructure: require a named production consuming workflow and bounded
  evidence under PM041; never label the consumer product accepted. The rollout
  successor itself receives this classification, not a process-work exemption.
- Deferred: keep existing no-execution state; reactivation requires reviewed
  classification and authorization. Do not treat a dormant placeholder as proof.
- External lane: preserve the exact owner-reviewed migration boundary without
  changing its live task. Out-of-boundary transitions require the responsible
  session's coordinated handoff and owner promotion. Do not insert enforcement
  into GP-CM05V mid-edit. Full rollout closure requires the hold resolved or an
  explicit owner-approved, bounded continuing exception with stated risk.

Proposed classification of every non-landed baseline item:

| Category | Exact Frontier keys |
| --- | --- |
| Specification | GUI-SURFACE-SPECS; DISTRIBUTED-COLLAB-SPEC; PROJECT-PREFERENCES-SPEC; ADOPTED-DRAFTING-STANDARD-SPEC; PROJECT-WRITER-OWNERSHIP; NATIVE-CONNECTIVITY-CONTRACT; NATIVE-WORKFLOW-BUDGETS |
| Product | UVT-S5A-BUILD; GUI-P2-CROSSPROBE; GUI-P2-INSPECTOR; GUI-MARKING-MENU; GUI-WRITE-PATH; NATIVE-AUTHORING; PROJECT-PREFERENCES-BUILD; NATIVE-PROJECT-DOORWAY |
| Specification (planning item only) | WORKFLOW-DELIVERY-ROLLOUT |
| Deferred | PRODUCT-REVISION-ENGINE; MCAD-INTEROP-PLACEHOLDER |
| External lane | GLOBAL-PREFERENCES-COMPLETION, pending explicit coordination of GP-CM05V with its existing session |

Every remaining baseline item is historical, with the pilot still enrolled.
This is a complete proposed classification, not automatic permission to amend
each lane's contracts. The MCAD placeholder retains its current planned/none
state; `deferred` here means no delivery authorization, not a lifecycle rewrite.

## 3. Exact transition and independent-review deltas

Keep existing contract schema 1 behavior, consumer registry, eight foundation
answers, nine proof dimensions, native correlation, input/environment hashing
and exact acceptance. Introduce an explicitly versioned delivery mapping for
new enrollments: `{schema_version: 2, contract_path, checkpoints}` with five
distinct ordered checkpoints `ready`, `activate`, `verify`, `review`, `accept`.
Legacy mappings remain byte-compatible. `review` maps to an execution step;
product `accept` remains owner_decision. Infrastructure acceptance remains null,
but its separate reviewer checkpoint is required for rollout infrastructure.

For the three adoption candidates, map:

| Lane | ready | activate | verify | review | accept |
| --- | --- | --- | --- | --- | --- |
| UVT-S5A-BUILD | S5A-C01 | S5A-C03 | S5A-C04 | S5A-C05 | S5A-C06 |
| GUI-WRITE-PATH | GWP-C01 | GWP-C03 | GWP-C04 | GWP-C05 | GWP-C06 |
| NATIVE-AUTHORING | NA-C01 | NA-C03 | NA-C04 | NA-C05 | NA-C06 |

Their C02 owner implementation authorization remains an intervening dependency,
not replaced by readiness or enrollment. Do not complete C01 with unresolved
mandatory behavior, or C03 from an unavailable-only scenario when the approved
outcome requires an enabled consumer. Changed reviewed build inputs continue to
force activation proof even if step labels remain pending. Selecting/starting
execution cannot bypass ready checks; completing review must invoke independent
replay checks before the owner acceptance phase.

Split `workflow_delivery_review.validate_review` into shared replay validation
and final owner-receipt validation. Replay validation binds proof packet,
implementation-session exclusion, same fixture and exact executable identity,
new event identities, environment, coverage and all findings. It may report
owner disposition pending; it must not demand or synthesize ACCEPT while review
is being prepared. Acceptance additionally retains every current trusted
defect-disposition and receipt requirement, including resolved blocking defects
bound to exact replay and explicit nonblocking deferral. A pending disposition
never permits acceptance. Preserve the current public acceptance wrapper for
legacy callers and negative tests.

Likely implementation seams: `workflow_delivery_trust.py` policy compatibility;
new small `workflow_delivery_coverage.py`; `workflow_delivery_selector.py` and
`check_workflow_delivery.py` integration; `workflow_delivery_checkpoints.py`
versioned transitions; `workflow_delivery_review.py` phase split; preparation
modules and owning tests. All are proposed, not edits authorized by R02. Add
new small modules rather than breaching source ceilings. Production mutation
paths and prototypes are outside this list.

## 4. Contract and root proposals — no fabricated production readiness

Proposed contract destinations are `specs/workflow_delivery/s5a.contract.json`,
`gui-write-path.contract.json` and `native-authoring.contract.json`. Their
governed authority includes the owning complete product routes, foundation and
consumer-completion routes, and the ratified delivery decision. Exact route
members and clause markers are resolved in each product C01; an invented marker,
null production handler for required enabled behavior, unresolved mechanism or
missing fixture prevents readiness. This lane prepares schemas and mappings;
the claimed product owner supplies/reviews actual handler identity and scope.

Start root review from the existing conservative build closure: `.cargo`,
`Cargo.toml`, `Cargo.lock`, `crates`, `docs/gui/icon_set.json`,
`docs/gui/menu_model.csv`, `docs/gui/menu_model.json`,
`scripts/cargo_resource_policy.json`, `scripts/run_cargo_guarded.py`, plus each
contract's exact native fixture inputs. This is not a claim that every such
path affects every slice. Use production build receipts and dependency evidence
to propose a narrower closure only if complete; the owner must promote it.
Do not exclude another session's relevant source merely to avoid a stale-proof
failure. Broad overlapping roots may require coordinated replay after Preferences
changes. Preserve the pilot's exact existing roots until its own reviewed change.

Every proposed product contract requires the following scenario groups, expanded
to the complete domain coverage matrix rather than a single passing example:

- S5A-S01…S06: native pointer/key/marquee selection; stable identity and stale
  subject refusal; pane/focus/filter scope; complete compound Common/Mixed/
  Unavailable inspection; no authored/journal writes through cancel/reopen;
  accessibility/non-color and fixture-budget evidence for board and schematic.
- GWP-S01…S06: native P0 typed rename and projection; W1/W2/W3 family/registry/
  schema coverage; direct versus proposal/dry-run; invalid/cancel/stale/second
  writer refusal; compensating undo/redo and reopen; crash/durability outcomes.
- NA-S01…S08: ordinary native fixture/doorway; library-backed symbol/pin identity;
  connection formation and attached movement; electrical-to-physical CI/PinPadMap
  join; exact placement/movement and numeric-entry states; shared snap/selection
  and whole-batch refusal; journal/proposal/undo/reopen/recovery; complete
  schematic/PCB tool/check/output and accessibility/performance coverage.

These groups reserve identities, not evidence or a waiver of full domain scope.
Proof/review artifacts live under `docs/reviews/workflow-delivery-rollout/`
in lane-specific subdirectories. No accepted pilot receipt is copied or refreshed
to create a new slice. A cohort can remain planning/enrolled-but-not-ready until
its genuine specification and implementation prerequisites are met.

## 5. Negative verification and proof of installation

Use isolated, owned temporary fixture repositories for destructive mutations;
never corrupt the shared tracker or disable installed hooks. Run the real
staged CLI, selector and owner hook paths, not only mocked helper functions.
Expected tests (codes proposed for new coverage; existing codes retained):

| Mutation or observation | Required result |
| --- | --- |
| New/unclassified task, deleted classification, product relabeled infrastructure | WDQ-COVERAGE or trusted-policy refusal; no transition |
| Product completes readiness without delivery declaration, missing foundation answer or unresolved mandatory decision | WDQ-COVERAGE/CONTRACT/AUTHORITY; no execution authorization |
| Enabled required consumer has no real handler/registry correlation, or all scenarios assert unavailability | WDQ-CONSUMER/RESULT; cannot count required delivery |
| Candidate changes enrolled source with ready/activation labels left pending | Activation proof required; absent/stale proof refused |
| Production source changes without a trusted scope/current authorized claim, or candidate adds its own scope | WDQ-COVERAGE/POLICY; cannot hide a new feature under an unenrolled pending task |
| Review step marked complete with no review, self-review or copied events | WDQ-REVIEW; cannot advance through review by writing a status |
| Authority, fixture, build, environment or correlated output differs | Existing WDQ-STALE/RESULT/REVIEW refusal; old evidence remains historical |
| Producer/replay defect has no finding, or final disposition is missing | WDQ-DEFECT; final acceptance blocked |
| Candidate invents receipt, changes roots/scenarios, weakens gate or selects moving refs | Existing WDQ-RECEIPT/POLICY/TRUST refusal |
| Unrelated document change with identical relevant inputs | Existing valid proof remains usable; no forced product rebuild |
| Reopened historical work, external-lane boundary exceeded or new execution under specification classification | Coverage refusal naming the responsible lane; no automatic task reassignment |
| Obsolete promotion base, dirty index, missing runner, no requested mode | Visible diagnostic and nonzero failure for activation; no silent partial installation |

Replay the full existing Python WDQ and PM025 suites, all new negative cases,
source/evidence/spec/dependency/resource/parity gates and unchanged pilot proof
under the proposed runner. Run Rust only when necessary for native proof,
through the guarded runner with disk-backed target ownership. Independent gate
review must reproduce staged refusal and success, not merely read the test log.
After owner installation, verify the *live* main checkout, local trust settings,
actual pre-commit invocation and selector. An isolated candidate test does not
prove the main checkout is protected.

## 6. Promotion, storage and recovery

`docs/reviews/workflow-delivery-rollout/proposal.json` is the non-operative
baseline classification/mapping/enrollment-intent inventory for this review.
It deliberately has no candidate commit, executed proof or owner receipt.
It is not `workflow_delivery_policy.json` and must never be read as live trust.
Only Frontier promotion, not ordinary beads intake, needs coverage classification;
batch reviewed new-item classifications to avoid an owner prompt for each bug.

Preparation produces an explicit candidate commit, exact clean base commit,
reviewed policy/contract/gate hashes, evidence environment path/digest and
independent-review packet. Owner invocation pins full object IDs, never HEAD or
a moving branch. Capture actual values at preparation time; this document's
baseline is not an executable activation pin. Recheck HEAD, index/worktree and
existing trust before publication; fail visibly on mismatch. No hook bypass,
force-push, history rewrite or candidate self-promotion.

Use an owned repository-local support store under `.git/datum-wdq/`: proposals
under `proposals/<packet-id>/`, immutable reviewed runner/hook bundles under
`trust/<full-authority-oid>/`, and activation logs under `logs/`. Resolve the real
Git common directory rather than assuming `.git` is a directory in a linked
worktree. Repository-tracked schemas/scripts/review evidence remain canonical;
the support store holds derived executable snapshots and logs, not the only
copy of the reform. No new sibling `~/Documents/datum-wdq*` directory is needed.

The current pilot preparation helper hard-codes one pilot and refuses output
inside the live repository. Replace that limitation only in the authorized,
reviewed general preparation implementation: permit the exact owned Git-common
support store, use an isolated candidate index/source checkout there, retain the
candidate through an object-retention ref and leave main unchanged until owner
fast-forward publication. Do not change old gates on live main and then bypass
their rejection to commit the replacement. No feature branch or PR is required.

Existing external hooks/runners remain installed until exact owner-authorized
replacement succeeds. The activation script reports each stage and final exit
code/log path, validates the reviewed candidate before fast-forward publication,
installs the exact owner-selected settings, then runs blocking verification.
Preserve a before-settings receipt; if post-publication verification fails,
stop with an explicit partial-activation diagnostic. Do not reset history or
restore an old runner that rejects the new main. The owner chooses a reviewed
forward repair or explicit rollback transaction. Never report success solely
because a merge succeeded. Interrupted activation must be diagnosable/re-runnable
only against its exact recorded phase and pins, not by skipping failed tests.

After live verification, inventory all old Documents bundles, Git worktree/ref
relationships and every hook/runner/config reference. Remove only positively
unreferenced owned scratch, reporting exact paths and recoverability. Keep the
active bundle and any rollback/evidence dependency; discuss retention of material
evidence before deletion. Cleanup is not permission to remove a live trust root.

## 7. Three actual adoption measurements and completion rule

Reserve S5A, GUI-WRITE-PATH and NATIVE-AUTHORING as separately enrolled product
cohorts, preserving their complete outcome and dependencies. At this baseline
all three have no live implementation claim; the latter two retain hard blockers.
There is no agreed product-owner start date or independent reviewer. This lane
must coordinate their claims, not invent availability or claim their code.
Track the distinction between an enrollment being installed and a feature being
ready/delivered. Owner approval of this proposal does not start those product tasks.

For each cohort record a committed adoption ledger with exact Frontier/issue,
contract identity, enrollment authority, producer/reviewer sessions and event
entries referencing candidate, requirement, evidence and bead IDs. Events are
append-only observations: readiness submission/refusal, specification gap,
owning-lane handoff, implementation rework, proof submission/refusal, independent
review, owner disposition and final acceptance. Corrections append a superseding
event; do not delete failed attempts. Event timestamps record observation, not
invented time spent working. Derive counts from evidence-linked events.

Start measurement at the first real readiness submission after enrollment;
finish only at exact owner acceptance of the approved cohort scope. Record
specification gaps discovered before versus after execution authorization,
corrective commits after failed proof/review, gate refusals by reason, handoffs,
owner correction rounds and unresolved defects. Do not manufacture a percentage
improvement without a comparable baseline. Report missing data explicitly.

Three task names, synthetic negative tests or implementer self-review do not
satisfy adoption. An accepted sub-slice cannot close a broader native-authoring
issue; if bounded slice measurement is desired, first obtain explicit owner
scope/contract mapping without discarding remaining domain obligations. Unavailable
cohorts stay incomplete or receive a separately reviewed replacement; no silent
substitution with easy documentation tasks.

Full rollout review requires installed coverage for the real roadmap, verified
refusals at production transition points, independently reviewed gate evidence,
resolved/bounded external-lane migration, three completed adoption measurements,
and scheduled remaining specification obligations with honest status. The
original reform goal remains active until this outcome is actually demonstrated.
