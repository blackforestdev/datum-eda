# Product Mechanics 059: Delegated feature delivery

Status: owner-ratified operational amendment, 2026-10-08.

## Owner direction and problem

The owner explicitly requested removal from routine development approval relays
and correction of the governing documentation. Research and ratified contracts
must drive implementation; repeated planning, proof packets and administrative
stops must not replace delivered designer workflows. The supporting evidence is
`research/process-quality/FEATURE_DELIVERY_WORKFLOW_CORRECTION.md`. This decision
amends PM025's decision routing, not its roadmap, claim or validation authority.

## Delivery unit and rhythm

### Feature delivery enforcement

The owner's one-hour workflow correction is execution-authorized as
FEATURE-DELIVERY-ENFORCEMENT. Target roughly 80% of development effort toward
implementing usable behavior; this is a prioritization target, not a measured
commit percentage or permission to skip safety. Commit labels cannot prove effort.

A selected feature must carry `feature_delivery` in the structured Frontier:
`ordinary_workflow`, `safety_requirements`, `qualification_followups`,
`prerequisite_reasons`, `development_ready` and `ordinary_evidence`. The selector
rejects missing/empty workflow or safety criteria, unexplained hard prerequisites,
and planning/decision substeps without a concrete `step:<ID>` blocker reason.
Changing scope or selecting a successor still requires its existing authority.

`development_ready` may become true only with repository-local ordinary-application
evidence identifying source commit, launch command and passing results for every
declared workflow and safety requirement. An offline fixture, document or engine
test alone cannot satisfy this claim. This is a structural evidence check, not an
automatic independent PASS, HUMAN assessment or product acceptance.

Within the current grant, continue implementation, focused verification and
repairs until the usable unit is delivered or a specific authority, safety,
review/transport or ownership blocker prevents further authorized work. A commit,
document, lease renewal or successful test is not by itself a stopping condition.
State the blocker and what exact action resolves it; keep executable authorized
work moving. Deferred qualification remains explicitly open and cannot be quietly
turned into a new hard prerequisite. Necessary fixtures supplement ordinary use;
every new verification batch must identify the decision it enables.

<!-- REQ:FEATURE-DELIVERY-ENFORCEMENT:FDE-C01 -->
Implement and verify ordinary-workflow, prerequisite and development-evidence
checks in the existing selector; reconcile startup guidance and migrate the
current first-edit task without changing its claim, writer grant or review gates.
The owner authorizes this bounded tooling/governance implementation, not unrelated
product mechanisms or a writer review result.

A unit is a complete designer-visible behavior with a finite acceptance boundary.
Within an authorized contract, the default cycle is **build, test, fix, document,
review and deliver**. Check the contract and existing implementation first; do not
start a new planning cycle for routine engineering choices. Implement the ordinary
application path, not just a type, fixture or isolated engine API. Essential
research precedes unresolved mechanisms; this rhythm is not permission to code
first and invent product semantics later.

Use focused verification to answer an identified correctness or delivery question;
reuse adequate historical evidence. Document implemented behavior, limitations
and necessary governance with the coherent implementation unit where practical.
Separate records or commits remain justified for actual authorization changes,
independent reviews, ownership transfer or required synchronization. There is no
feature-commit percentage, time quota or permission to relabel supporting work.

Implementers continue through routine repairs and adequate verification under the
retained grant. Do not stop just because a commit, proof receipt, lease renewal or
planning summary is complete. Capture unrelated defects and continue unless they
demonstrably prevent delivery. A verification gap blocks its applicable claim,
not unrelated authorized implementation. Do not invent extra implementation when
it is exhausted: arrange review and resolve the finite remaining obligations.

## Management role and delegated authority

The owner appoints a management session; no implementer appoints itself or gains
authority merely by using that title. Record the named session and delegated task
scope in the existing handoff or task evidence. This decision is the standing
owner delegation; task-specific restrictions and explicit nondelegable decisions
still control. Until a manager is appointed, this policy does not fabricate one.

Within ratified scope, that manager may coordinate assignments and synchronized
claim transfers, arrange independent reviewers, receive actual review receipts,
record their outcomes, direct corrections, and approve routine continuation. It
may disposition intermediate delivery and objective acceptance against existing
criteria only when complete required evidence and independent review are present.
It may record explicit successor selection only when the successor's existing
authorization and prerequisites permit it; completion never authorizes new scope.
It cannot overwrite another session's claim or work without a valid handoff.

A step spelled `owner_decision` may be handled by the appointed manager only
where the decision is within this delegation: review provisioning, receipt of
an actual independent review, unchanged-scope execution continuation, or objective
acceptance under already ratified criteria. The manager's durable disposition must
identify its session, task/step, exact candidate/evidence, this delegation and the
applicable contract. It is an owner-delegated disposition, never a claim that the
human replied. Record it and reconcile authorization/step state in the same
governance transaction before implementation resumes. Do not add a competing
roadmap or bypass failed selector validation. Existing selector response formats
and `owner_decision` states remain; routing alone is amended. An implementer at
such a boundary returns the packet to the manager rather than the busy owner.

## Owner-reserved decisions

Escalate new or conflicting product semantics, new source/identity mechanisms,
changes to ratified scope, requirement reductions, acceptance waivers, dependency
or licensing adoption under PM029, destructive actions, and decisions explicitly
reserved to the owner. Unsupported numerical budgets and fabrication tolerances
remain prohibited; neither a manager nor owner approval supplies missing evidence.
New UX/product choices and required HUMAN assessments remain owner-reserved unless
the owner explicitly designates an authorized assessor or separately delegates
that exact decision. Objective acceptance is not a substitute for HUMAN review.

First consult existing research, historical behavior and owning specifications.
If they settle the question, implement it; engineering judgment alone is not an
escalation reason. If they conflict or do not settle it, send one consolidated
decision packet with recommendation, evidence, consequences and exact authority
requested. Do not imply that routine continuation ratifies a new mechanism.

## Independent review and communication

The implementer cannot independently accept its own candidate. Management may
coordinate review but cannot fabricate availability, findings or PASS. A manager
that implemented or materially designed the candidate cannot be its independent
reviewer. Corrections inside the reviewed execution grant do not require renewed
owner authorization; scope-changing remedies do. Preserve mandatory proof, source
health, resource discipline, canonical mutation and Claude's prototype file lane.

Use available authorized session channels for manager/implementer/reviewer
handoffs. The owner authorizes those development-team handoffs; this is not
permission for unrelated external messaging. If session transport is unavailable,
record that concrete integration limitation rather than silently using the owner
as the default relay. No new transport dependency or service is authorized.

Report delivered user behavior, verification outcome and genuine blockers. Reuse
existing receipts and trackers; no new approval framework, proof ledger or separate
manager authorization is needed for each ordinary unit. The owner is contacted
only for the reserved decisions above or a specifically requested status report.

## Owner-directed usable development builds — 2026-10-08

The owner directs delivery of an application they can use to design, with their
ordinary use supplying primary defect feedback. They specifically report being
unable to edit an imported board. A passing artificial fixture is supporting
evidence, not delivery of that workflow.

The first development target is an ordinary GUI edit of an existing Footprint in
a Datum-owned project derived from the owner's board: select, preview a move,
cancel without writing, commit through the canonical mutation path, undo/redo,
save and reopen. Preserve the original KiCad inputs; this does not authorize
writing back to KiCad or expanding the frozen converter. Follow with native
library-backed symbol placement and wire connection, then physical placement and
manual routing. These are incremental deliveries inside the existing authoring
contracts, not replacements for their complete acceptance scope.

Deliver each usable slice after focused checks for canonical writes, persistence,
undo/recovery and obvious interaction failures plus its bounded independent review.
Give the owner a pinned ordinary launch command and a short description of what
they can do. Owner use is voluntary product feedback, not a required manual test
script and not a substitute for data-integrity checks. Reproduce reported defects
on the ordinary path, repair them, and retain focused regressions. Artificial
fixtures remain justified for isolated boundary/negative cases; do not expand
them into a campaign without naming the delivery decision they enable.

Distinguish development availability from full qualification. Missing platform,
visual, accessibility, endurance or comprehensive matrices remain explicitly open;
they block their associated acceptance claims, not an unrelated authoring slice
unless a concrete safety/correctness dependency is demonstrated. No missing proof
is passed, waived or silently deleted. A development build is not a qualified
release. Existing product, source-health, dependency and mutation rules remain.

The structured USER-EDIT-DELIVERY lane makes this direction discoverable by the
selector. Its immediate grant is a bounded readiness reconciliation, not permission
to implement an unratified writer protocol. Reuse the existing GUI-WRITE-PATH and
NATIVE-AUTHORING contracts; identify only actual dispatch, serialization and recovery
prerequisites. Consolidate genuine unresolved owner choices. Do not redesign
selection, polish the Inspector, launch another benchmark campaign or draft a new
general authoring specification as prerequisites to this first edit.

<!-- REQ:USER-EDIT-DELIVERY:UED-C01 -->
### UED-C01 — establish the first ordinary edit's real execution boundary

Inspect current GUI dispatch, canonical board move/undo/reopen builders and project
writer ownership against existing ratified contracts. Produce one source-backed
readiness disposition: reusable implemented owners, missing integration, genuine
authority gaps and the smallest executable footprint-move slice. Schedule actual
write-safety prerequisites and the runtime slice together; name exact granted versus
ungranted work. Do not assert that planning, project rename or a CLI-only edit
delivers board editing. Retain the follow-on symbol/wire and board-routing targets.
Before touching another session's work, obtain its explicit synchronized handoff.

## Current-work preservation

Apply prospectively to the retained S5A grant and all subsequent authorized work.
Preserve PM058 implementation, the existing claim and its exact scope, required
C04/C05/C06 evidence, open CAM/T04, unamended positives, performance provenance and
prototype ownership. This amendment neither closes S5A nor passes missing proof.
No runtime, selector schema, dependency or product geometry change is introduced.

<!-- EVIDENCE:USER-EDIT-DELIVERY:UED-C01-READINESS -->
### First-footprint edit readiness outcome — 2026-10-08

UED-C01 is complete at the planning boundary. The source-pinned handoff is
`docs/reviews/workflow-delivery-rollout/user-edit/first-footprint-edit-readiness.json`
(UED-FIRST-MOVE-01, inspected source f827ad1b). The S5A coherent unit8228b4b7
is committed; its claim is explicitly released, with no dirty runtime/prototype
work and no missing qualification passed. Its retained claim scope is not
transferred to this delivery.

Existing guarded package-position builders, owned-Pad translation, canonical
commit/inverse/replay and native compensating undo/redo are reusable. Ordinary
GUI authoring is disabled and its dormant move path sends terminal text. The
native daemon registry does not contain the move verb; typed GUI move/history
integration is missing. Preview must retain its acquired Footprint and cursor
anchor instead of replacing its target with successive hover hits. Reopen must
use a persistent Datum-owned Project, not repeat temporary --board import.

The real safety prerequisite is the existing Project writer decision: all writer
entry points need serialized fresh resolution/guard validation, persistence and
recovery. Current transaction lock acquisition follows in-memory revision checks;
that source observation is not a fresh reproduced race or a ratified protocol.
Recommend engine-owned serialization using the existing lease, because daemon-only
serialization cannot cover CLI and in-process writes. Preserve PWO-C02,
PWO-REVIEW and PWO-C03; no lease/recovery semantics are ratified here.

The packet schedules only writer safety, typed first-edit integration and ordinary
footprint delivery/review. Existing P0 and complete W1-W3/native-authoring obligations
remain; a first usable development build cannot close them. No new runtime edits,
fixtures, dependency, converter expansion, KiCad writeback, geometry tolerance,
prototype change, full qualification or execution authority are introduced.

<!-- REQ:USER-EDIT-DELIVERY:UED-C02 -->
<!-- OWNER:USER-EDIT-DELIVERY:UED-C02:FIRST-EDIT-GRANT -->
### UED-C02 — disposition the first-edit grant and its actual safety prerequisite

Present UED-FIRST-MOVE-01 together, pinned by source and packet commit/blob.
Request one bounded disposition: approve engine-owned writer-protocol preparation
and independent review under existing PWO gates; grant the first-footprint runtime
scope only after actual writer review and owner ratification. The intended slice
is typed move preview/cancel/commit, compensating undo/redo, Save and persistent
reopen on the owner-board-derived Project, with ordinary Wayland+WGPU verification,
focused failure controls and bounded independent implementation review. Preserve
original KiCad bytes and all broader W1-W3/S5A/NA/CAM obligations. Approval of this
readiness packet does not fabricate protocol review or ratification, and no
runtime work starts until its recorded prerequisites and execution grant permit it.

<!-- EVIDENCE:USER-EDIT-DELIVERY:UED-C02-OWNER-GRANT -->
### Appointed first-footprint delivery manager — 2026-10-08

The owner approves UED-FIRST-MOVE-01 at commit
`e7fd1b28dd2c8ecb79eab6a7f30a5ed594f6c8b4`, packet blob
`874ae14396bd25ed282df3cd95cb5f2aed93461d`, and explicitly appoints this
implementation session (`codex-user-edit-delivery-20261008`) as delivery manager
for the first-footprint edit under PM059. This is an owner appointment, not
self-appointment. Record actual reviewer receipts, routine corrections,
unchanged-scope continuation and objective development-build acceptance without
owner approval relays. Use the existing reserved independent audit session;
the implementing manager cannot review its own specification or implementation.

The owner additionally delegates **PWO-C03 ratification for this bounded
engine-owned writer protocol**, conditional on actual independent specification
PASS and disposition of every mandatory finding. This explicit delegation is
an exception to PM059's otherwise owner-reserved writer-policy ratification,
limited to UED-FIRST-MOVE-01. Preserve canonical mutation, serialization, fresh
guard validation, durability and recovery. The execution grant covers writer
preparation/review/ratification, the necessary safety correction, typed dispatch,
first-footprint preview/cancel/commit, undo/redo, Save and persistent reopen,
focused ordinary Wayland+WGPU verification and bounded independent review.
Writer representation implementation still waits for that actual reviewed
ratification. No new dependency, scope reduction, tolerance, broader product
rule, full qualification or successor feature phase is delegated.

<!-- REQ:USER-EDIT-DELIVERY:UED-C03 -->
Prepare the smallest complete bounded engine-owned writer protocol and actual
consumer/failure reconciliation using the existing PWO clauses and source pins.

<!-- REQ:USER-EDIT-DELIVERY:UED-C04 -->
Return the pinned writer proposal to the reserved independent audit session;
record actual specification PASS or mandatory corrections. No substitute review.

<!-- REQ:USER-EDIT-DELIVERY:UED-C05 -->
<!-- OWNER:USER-EDIT-DELIVERY:UED-C05:BOUNDED-WRITER-RATIFICATION -->
The appointed manager may ratify the exact bounded writer protocol only after
actual independent specification PASS and resolution of mandatory findings under
the owner's additional PWO-C03 delegation. Retain broader PWO obligations.

<!-- REQ:USER-EDIT-DELIVERY:UED-C06 -->
Implement the ratified minimal writer safety and first-footprint ordinary edit
through existing canonical builders/history and typed GUI/native dispatch.

<!-- REQ:USER-EDIT-DELIVERY:UED-C07 -->
Prove first-footprint cancel/commit/undo/redo/Save/reopen and focused failure
safety on the owner-board-derived Project; repair actual failures and return the
exact candidate to the reserved independent reviewer. Use ordinary Wayland+WGPU;
missing platform/HUMAN qualification remains explicit, not forced-X11 success.

<!-- REQ:USER-EDIT-DELIVERY:UED-C08 -->
<!-- OWNER:USER-EDIT-DELIVERY:UED-C08:DEVELOPMENT-BUILD-ACCEPTANCE -->
The appointed manager may record objective first-edit development-build acceptance
only with the required focused safety and actual bounded independent review.
Deliver pinned source/binaries/native Project and ordinary launch command. Keep
full GWP/NA/S5A/CAM and nondelegated HUMAN acceptance separate; no successor grant.

<!-- EVIDENCE:FEATURE-DELIVERY-ENFORCEMENT:FDE-C01 -->

FDE-C01 implemented in `27e989084464f0a78a9701d87588b4d28ff0d206`. Sixty focused feature-policy/selector tests
passed, including missing contracts, unexplained prerequisites, fixture-only
availability, missing ordinary results/source pins and evidence path escape.
Project-state, generated roadmap, specification governance/parity, evidence
traceability, progress coverage and source health passed. The first-edit
manager claim, execution conditions and actual writer review are unchanged.
This completes the owner-authorized enforcement correction; it does not claim
measured effort allocation, board editing or broader product acceptance.

<!-- EVIDENCE:USER-EDIT-DELIVERY:UED-C06-FIRST-EDIT -->

UED-C06 implements the bounded first Footprint edit: typed native Move/history,
complete source-qualified preview and cancellation, asynchronous canonical
commit and qualified scene publication, durability-confirming Save and native
Project reopen. Engine writer prerequisite is `3efebe9d`; retained owner grant
and actual reserved specification PASS/ratification are recorded above. The
implementation and focused raw outcomes are pinned by source blobs in
`docs/reviews/workflow-delivery-rollout/user-edit/first-edit-implementation.json`.
Production-callback proof on the real owner-board-derived native Wayland/WGPU
host passed preview/cancel, complete 81-Pad translation, exact undo/redo, Save
and reopen. Locked/overflow/stale refusal, required P0 transport and guarded
uncertain-receipt recovery passed at their recorded scopes. Subsequent source
corrections and the exact binary cohort remain subject to UED-C07 verification.

The canonical next step is UED-C07: finish focused ordinary integration proof
and obtain actual reserved bounded candidate review. UED-C08 development
acceptance remains pending. Programmatic callbacks are not physical Wayland
input/HUMAN qualification. Preserve full GWP/NA/S5A/C04-C06, CAM/T04,
nondelegated HUMAN and wider platform obligations; no successor scope granted.
Original KiCad inputs are byte-identical; the editable copy uses
`--project-root`. A geometry commit invalidates current-fill correspondence;
this unit does not display stale occupied copper as current or waive CAM.


<!-- EVIDENCE:USER-EDIT-DELIVERY:UED-C07-FIRST-EDIT-PASS -->

UED-C07 completed on source `a07ce3e57e29179072adbc9e55534008984312dc`,
implementation packet blob `1370cb179ec481b6be5101f8cccdc47e1a26e7cc`.
Actual reserved independent corrected implementation PASS is preserved unchanged
in `docs/reviews/workflow-delivery-rollout/user-edit/first-edit-corrected-review.json`.
UED-IR-01–03 are RESOLVED: authoritative selection retention, snapped invocation
reference translation, and accepted unterminated journal-record preservation.
Independent replay on an audit-owned copy passed the existing real-board native
Wayland/WGPU production-callback workflow and both journal regressions. Complete
81-Pad membership, cancel without writes, off-grid Move, exact undo/redo,
selection retention through Save/refusal, and Save/reopen passed. The ordinary
no-test-flags Wayland/Vulkan visible reopen and inspected capture are retained
at their producer scopes. Physical compositor input/HUMAN qualification is not
claimed. Raw outcomes, source/Project/binary pins and failure attempts are retained;
original KiCad inputs and delivered Project pins were independently checked.

<!-- EVIDENCE:USER-EDIT-DELIVERY:UED-C08-DEVELOPMENT-ACCEPTANCE -->

The owner-appointed `codex-user-edit-delivery-20261008` delivery manager records:
`USER-EDIT-DELIVERY: manager approve DEVELOPMENT-BUILD-ACCEPTANCE —
a07ce3e57e29179072adbc9e55534008984312dc`, conditioned on the actual reserved
PASS above and the objective evidence in
`docs/reviews/workflow-delivery-rollout/user-edit/first-edit-delivery.json`.
This is bounded first-footprint-edit development availability under the recorded
PM059 appointment, not self-review or full product acceptance. Every declared
workflow and safety requirement has actual GUI/canonical-path evidence. The
matching GUI, CLI and daemon are pinned; the ordinary launch opens the persistent
Datum-owned `DOA2526-first-edit` Project. M/Edit Move, click Footprint or its Pad,
preview, Escape cancel, destination click commit, undo/redo, Save and reopen are
available. The native copy retains producer demonstration moves; original KiCad
board and schematic bytes are unchanged.

UED-C07 and the bounded product delivery are complete. The manager approves
objective development availability; UED-C08 synchronized closure publication is
pending one explicit owner canonical-successor choice. PM025 currently rejects
zero canonical tasks, while this grant authorizes no automatic successor. This
is a roadmap-transition boundary, not missing implementation, review or first-edit
safety proof. The implementation claim is explicitly released. No successor task
is selected or authorized by this completion. Physical Wayland input/HUMAN, full S5A/GWP/NA/C04–C06, wider platform,
CAM/T04 and broader writer qualification remain pending at their existing gates.
A committed geometry change invalidates old copper fill correspondence; stale
occupied copper is omitted, with no refill or manufacturing waiver. Missing
schematic/native editor obligations are unchanged. No dependency, prototype,
persistence representation, timeout, tolerance or scope reduction is introduced.
