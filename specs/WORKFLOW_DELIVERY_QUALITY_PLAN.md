# Workflow delivery quality: staged adoption plan

Status: governed planning contract; proposed enforcement, not active policy
Frontier key: WORKFLOW-DELIVERY-QUALITY
Owner evidence: research/process-quality/WORKFLOW_DELIVERY_OWNER_BRIEF.md

## Authority and implementation boundary

This plan responds to the owner's 2026-09-05 request to improve delivery quality
without disrupting concurrent Preferences/Units corrections. Decision 025 and
docs/PROJECT_STATE_POLICY.md remain the sole task-selection and claim authority.
CLAUDE.md's manual-first, shared tooling, one-mutation-path, evidence review,
dependency and attribution rules remain controlling. Decision 022 remains the
source-health boundary. Claude retains every visual-truth HTML file.

The plan schedules research, specification and a concrete adoption decision.
It does not ratify a new data schema, implement a gate, change acceptance of an
existing feature, introduce dependencies, or authorize agent parallelism.
No percentages or legacy completion states are revised without an audit.
Implementation requires an owner-ratified numbered decision and synchronized
Frontier authorization. Existing checks are not weakened to accommodate rollout.

## Concurrent-work protocol

Preparing and registering this plan can coexist with the existing GP-CM01 claim.
The owner's subsequent `please proceed` authorizes the separate WDQ-C01 audit
lane: read-only runtime/source diagnostics and owned audit evidence. Record its
own scoped Frontier/beads lease; GP-CM01 remains canonical. This is owner-directed
audit work, not concurrency inferred from dependency readiness. Gate activation
still requires the coordinated adoption checkpoint below.
Do not edit its runtime, governing evidence, prototype lane, lease, acceptance
criteria or selected step. Shared governance transactions must preserve all
other entries, check fresh state, regenerate the projection, and stage only owned
changes. If another session changes the same hunk or staged file, coordinate a
handoff rather than overwrite or sweep it into a commit.

The initial proposed checkpoint was the GP-CM01 handoff. That is historical,
not a current resume instruction. The concrete adoption packet now requires a
fresh owner-selected bounded handoff in WDQ-G02, verified against current task
and claim state; no Preferences step is assumed still active. Activation requires an
explicit selector/authorization transaction with the current task owner. The
parent Preferences issue need not be falsely closed to arrange a bounded pause.
Keep its remaining steps and evidence intact and record how work resumes.

No blanket halt, push, branch, rebase, reset or rewrite is part of this work.
The 155-commit local lead is a separate checkpoint/publication concern; audit
local committed history, dirty ownership and proof status before any separately
authorized publication. A large local lead is not a reason to discard work.

## Ordered planning contract

The Frontier completion contract selects the actual step. The sections below
define requirements and deliverables, not independent concurrency permission.

<!-- REQ:WORKFLOW-DELIVERY-QUALITY:WDQ-C01 -->
### Establish an evidence-backed capability baseline

Audit representative manual paths through project creation/reopen, library part
resolution, schematic placement/connectivity, schematic-to-board propagation,
exact board editing and manufacturing output. Exercise available production
paths; record unavailable paths without constructing a test-only substitute.
For each path record fixture provenance, build commit and local modifications,
entry conditions, actual input, expected/observed result, persistence/recovery,
evidence locations, limitations and owning Frontier/bead. Do not manufacture a
full-board fixture or start implementing unavailable authoring to finish an audit.

Distinguish usable, partial, unavailable and unverified observations. Separate
code presence, engine proof, GUI integration, workflow verification and owner
acceptance. Record inherited claims as historical until examined. Reproduce a
bounded failure before opening a corrective issue, or mark an unverified concern
as such. Search beads first and link existing issues instead of duplicating them.

Deliverable: a small reproducible baseline and a gap-to-owner mapping, with no
unsupported product-completion claims. Audit scope expands only when an observed
cross-cutting failure requires it; this is not an open-ended whole-repo rewrite.

<!-- REQ:WORKFLOW-DELIVERY-QUALITY:WDQ-C02 -->
### Resolve foundational requirements through bounded research

For affected workflows, map the owners and consumers of units/precision and
coordinate transforms; numeric entry; selection/identity; grid/snap; undo and
cancel; persistence/crash recovery and writer ownership; library binding and
connectivity; and application/project/document settings scope. Add other
foundations only with a concrete workflow consequence.

Inspect each owning evidence route and every source/consumer before reviewing
or changing its specifications. Record source authority, contradictory statements,
missing decisions and implementation divergence separately. Runtime behavior
does not amend a ratified requirement. A gap requires a question, affected user
scenario, risk, existing owner, research output, and completion boundary.

Use primary sources for external research; record versions, provenance and
applicability. Distinguish standards requirements from conventions and design
choices. Do not fetch or introduce code dependencies as research. Route visual
changes to Claude with exact file/region, required outcome, preserved decisions
and expected native/reference proof.

Fold findings into existing scheduled owners where appropriate. Otherwise add
bounded planning beads and Frontier entries with dependencies, authority and
required owner decisions in one governance change. The outcome is a dependency
map feeding deliverable workflows, not a second roadmap or an indefinite
"enterprise readiness" epic. Research exits when each scoped question has a
supported recommendation or an explicit unresolved owner decision.

<!-- REQ:WORKFLOW-DELIVERY-QUALITY:WDQ-C03 -->
### Specify readiness, evidence and refusal behavior

Prepare a numbered decision proposal with a small machine-readable contract and
behavioral tests. Prefer extending the existing completion/evidence machinery;
justify a separate format only if it avoids duplicate authority. Fix exact
schema and integration details during this step, not by implication here.

The proposal must resolve these boundaries:

| Boundary | Required evidence | Proposed automatic refusal |
| --- | --- | --- |
| Start implementation | Named manual scenario, scope/exclusions, reviewed authority, affected consumers, resolved behavior, expected failure/cancel/save/reopen behavior | Missing or contradictory prerequisites; unresolved required owner decisions; claim or step mismatch |
| Activate a control | Production consumer, effective scope, timing, persistence and failure behavior | Visible/enabled entry has no connected consumer or required behavior proof |
| Claim workflow verification | Native input-to-engine-to-visible/persisted-result proof on identified source/fixture/environment | Engine-only proof substituted for a manual path; absent or failed required scenarios |
| Claim product acceptance | Independent review disposition, all required evidence, unresolved-defect disposition, owner acceptance of the reviewed revision | Missing/rejected/stale review, incomplete mandatory scenario, unsupported completion transition |
| Amend evidence or policy | Traceable authority change and affected-consumer review | Changed relevant source with stale acceptance; weakened checks or refreshed goldens without authorized review |

Scenario expectations must cover normal and invalid input, cancellation, scope,
precision where applicable, undo or an explicit justified non-applicability,
save/reopen, accessibility and failure recovery proportional to the feature.
Non-applicability needs a reason and review; an empty list is not proof.
Pure infrastructure work must name its consuming workflow and report its bounded
completion without pretending that the workflow is delivered.

Define evidence freshness using relevant inputs, not every unrelated repository
commit. Specify the code/build, fixture, contract/prototype and environment
identities, evidence integrity, and what changes invalidate which proof. Prevent
editing a manifest status or digest from manufacturing approval. Show the limits:
machines validate structure, references and observations; human review judges
interaction quality. A timestamp or a string naming a reviewer is not independent
verification. Specify reviewer identity and separation without AI credit in git.

Specify negative tests for absent/malformed contracts, missing and stale artifacts,
changed consumer mappings, duplicate scenario IDs, unrelated evidence reuse,
unresolved decisions, false non-applicability, self-approval, rejected review,
unsupported completion, and gate/policy bypass. Gates diagnose and fail; they
never mark work accepted, rewrite specs, select work or refresh authority.

Deliverable: a reviewable mechanism proposal, candidate data shape, examples of
valid/refused transitions, negative-test matrix and exact integration points in
the existing selector, proof battery and commit/CI paths. No blocking activation
occurs during planning and no external service or dependency is presumed.

<!-- REQ:WORKFLOW-DELIVERY-QUALITY:WDQ-C04 -->
### Prepare a bounded pilot and migration transaction

Choose a pilot from the audited work at an authorized handoff. Preserve accepted
history; do not mass-relabel existing features or grandfather them as usable.
Record a legacy baseline and explicit scope for newly enforced transitions.
Demonstrate that missing consumer behavior and stale acceptance would be refused
while unrelated edits and legitimately bounded infrastructure work remain valid.

Prepare implementation tasks with exact file ownership, prerequisites, acceptance
tests, source budgets and rollout/repair procedure. Use dependency-free Python
where sufficient; any new third-party code still requires decision 029 approval.
Run non-compiling gates independently and Rust proof through the guarded runner.
Keep prototype changes in their owning lane. Do not weaken a failing existing
gate or launder another session's evidence to make the pilot pass.

Define closure proof: a working native pilot, demonstrated negative refusals,
independent review, unchanged roadmap-selection semantics, source-health and
governance checks, and explicit owner acceptance. Measure repeated corrections,
reopened acceptance and completed manual scenarios to assess improvement; do not
use document count, code volume or raw closed-bead count as product maturity.

Deliverable: exact adoption scope, checkpoint, numbered decision draft, a bounded
implementation completion contract and a resume disposition for interrupted work.
Only existing authorized gates remain blocking until the adoption is ratified.

<!-- REQ:WORKFLOW-DELIVERY-QUALITY:WDQ-C05 -->
<!-- OWNER:WORKFLOW-DELIVERY-QUALITY:WDQ-C05:WDQ-C05 -->
### Owner disposition of the concrete adoption packet

Review the completed baseline, research dispositions, proposed mechanism, pilot,
proof requirements and exact activation checkpoint. Record approval, requested
corrections or deferral durably. Broad agreement with process improvement is not
evidence that an unbuilt pilot passed or that the schema is already ratified.

Concrete packet: `specs/WORKFLOW_DELIVERY_ADOPTION_PACKET.md`, with the full
baseline, F1–F8 research map, proposed PM041, exact gate contract and N01–N20 /
P01–P07 test oracle linked there. The implementation successor is bounded to
honest action availability and gate proof; it is not native EDA authoring.
Approve packet preparation/ratification only. WORKFLOW-DELIVERY-GATE-PILOT has
its own execution and activation owner boundaries, and FOUNDATION-WORKFLOW-SPEC
retains unresolved CAD questions and consuming-owner handoffs.

After approval, ratify the mechanism in the next available numbered decision and
register every affected source/consumer and inventory in the same governance
transaction. Schedule and authorize its bounded implementation separately;
complete this planning bead only with its landing evidence. Preserve the selected
task until an explicit handoff changes it. Planning closure never automatically
selects or authorizes its successor.

## Present delivery status

<!-- EVIDENCE:WORKFLOW-DELIVERY-QUALITY:WDQ-C04-COMPLETE -->

WDQ-C04 is complete as the concrete adoption packet committed in `6ba8ea37`.
`specs/WORKFLOW_DELIVERY_ADOPTION_PACKET.md` contains the bounded About/Fit
consumer-readiness pilot, exact G01–G06 contract, file ownership and budgets,
legacy boundary, independent proof, owner-controlled activation and fresh
existing-work handoff/resume transaction. Proposed PM041 and the exact gate
contract were committed with it. The runtime pilot and new N/P suite have not
been implemented or executed. WDQ-C05 now awaits owner disposition of this
specific packet; the planning claim is released and no successor is authorized.

<!-- EVIDENCE:WORKFLOW-DELIVERY-QUALITY:WDQ-C03-COMPLETE -->

WDQ-C03 is complete as a mechanism proposal: proposed PM041 and
`specs/WORKFLOW_DELIVERY_GATE_CONTRACT.md` define exact closed data shapes,
checkpoint integration with the existing selector, relevant-input freshness,
production consumer checks, independent replay and owner-controlled receipts.
`research/process-quality/WORKFLOW_DELIVERY_REFUSAL_CASES.md` specifies N01–N20
and P01–P07 with explicit automated-versus-human limits. These are future test
oracles, not executable tests claimed to have passed. No schema or gate is active.

<!-- EVIDENCE:WORKFLOW-DELIVERY-QUALITY:WDQ-C02-COMPLETE -->

WDQ-C02 is complete as bounded process research and specification scheduling.
`research/process-quality/WORKFLOW_DELIVERY_FOUNDATION_RESEARCH.md` distinguishes
verified local observations, external conventions and unresolved CAD questions.
F1–F8 map affected consumers to existing owners; none ratifies units, selection,
storage or library semantics in this lane. FOUNDATION-WORKFLOW-SPEC is the
noncanonical planning follow-on below; existing implementation claims and
dependencies are preserved. The startup intake waits for this contract, while
the enabled-but-unhandled action defect is linked to the proposed gate pilot.

<!-- EVIDENCE:WORKFLOW-DELIVERY-QUALITY:WDQ-C01-COMPLETE -->

WDQ-C01 is complete as a bounded audit, documented in
`research/process-quality/WORKFLOW_DELIVERY_CAPABILITY_BASELINE.md` with a fresh
6fa5aa3 build, matching source/import-map provenance, an archived native fixture,
isolated actual input, explicit unavailable paths and preservation/reopen limits.
The associated captures and input logs are committed under
`research/process-quality/evidence/wdq-c01-current/`. This accepts no product
workflow. Selection, zoom, Fit, dismissal and argument-based reopen are observed;
native authoring and manufacturing completion remain unavailable/unverified.

The owner has authorized completion of this entire planning item. Foundation
domain research, mechanism ratification, gate implementation and native pilot
acceptance remain outstanding. Proposed data shapes are now explicit; no runtime
inventory or executable gate is introduced. Parity registration and enforcement
belong to the separately authorized implementation transaction.

## Bounded foundational specification follow-on

Frontier key: FOUNDATION-WORKFLOW-SPEC; issue:
`dat-manual-foundation-contracts-fsw`. Blocked on reviewed WDQ adoption, planning
only. This is an explicit future contract, not alternate next work or permission
to rewrite another owner's route. Scope is an ordinary native project doorway,
one exact board-edit/reopen path and its library/connectivity handoff; broad
enterprise capability and new dependencies are excluded.

<!-- REQ:FOUNDATION-WORKFLOW-SPEC:WDQ-F01 -->
### Review foundation authority and consuming owners

Read every source/consumer of the routes identified for F1–F8 in the foundation
research. Map each applicable clause to ratified, proposed, contradictory or
unanswered authority. Reconcile the PM012 draft-header/doctrine-class mismatch
through its owning governance lane, preserving explicit ratifications. Confirm
current owners and output a clause-level decision list, not a general audit.

<!-- REQ:FOUNDATION-WORKFLOW-SPEC:WDQ-F02 -->
### Specify one connected manual-workflow contract

Resolve F1–F8 for the bounded scenario with concrete values, boundary/invalid
cases, direct/proposal and persistence timelines and consumer bindings. Reuse
current authority; conduct primary research for genuine unanswered questions.
Return necessary visual changes to Claude with exact file/region, preserved
decisions and expected proof. No product or prototype implementation here.

<!-- REQ:FOUNDATION-WORKFLOW-SPEC:WDQ-F03 -->
<!-- OWNER:FOUNDATION-WORKFLOW-SPEC:WDQ-F03:WDQ-F03 -->
### Review foundational contract and dependent-owner handoff

Present answers or explicit unresolved decisions for F1–F8, complete route
reconciliation, consuming-owner handoff, test expectations and proposed exact
changes to existing GUI-SURFACE-SPECS, GUI-WRITE-PATH and NATIVE-AUTHORING
prerequisites. Owner approval is required before amending those dependencies
or authorizing implementation. Incomplete required behavior remains blocked.
