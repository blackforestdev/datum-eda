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

Recommended adoption checkpoint: the GP-CM01 handoff, before authorizing further
expansion. Alternative: the Global Preferences production-acceptance handoff.
These are proposals, not rival next-task instructions. Activation requires an
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

After approval, ratify the mechanism in the next available numbered decision and
register every affected source/consumer and inventory in the same governance
transaction. Schedule and authorize its bounded implementation separately;
complete this planning bead only with its landing evidence. Preserve the selected
task until an explicit handoff changes it. Planning closure never automatically
selects or authorizes its successor.

## Present delivery status

This package supplies the owner brief, planning requirements and synchronized
roadmap/tracker placement. WDQ-C01 has a partial observation record in
`research/process-quality/WORKFLOW_DELIVERY_CAPABILITY_BASELINE.md`; it is not
completion evidence. Current-build and native-interaction verification remain.
The complete capability audit, external research, mechanism
ratification, gate implementation and native pilot are outstanding. No new
runtime inventory or executable gate schema is introduced; parity registration
belongs to the mechanism transaction once that inventory is specified.
