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


### Owner QA revision — first edit remains undelivered in ordinary use

The owner reports that selecting an object then pressing M produces movement
delay exceeding five seconds followed by a crash. The appointed manager
withdraws development availability and reopens UED-C07 under the retained
first-edit correction grant. Earlier actual independent PASS and logs remain
valid at their recorded callback/recovery scopes; they do not establish success
for the failed ordinary input/frame sequence. A focused fixture must include
production selection, keyboard routing, pointer movement and intervening render
preparation/submission. Preserve current Project and original KiCad bytes;
reproduce on a copy and repair demonstrated causes. No successor work, prototype,
dependency, policy waiver or broader performance campaign is authorized.

### First-edit ordinary frame correction — retained UED-C07 scope

The copied owner-board fixture now drives the ordinary App Wayland/WGPU frame
loop, production pointer dispatch and normalized M shortcut. It requires an
actual moving Footprint preview, cancel with exact preimage preservation, a
commit click on its presented authored label, exact undo/redo, Save and reopened
revision. Production native deadlines remain authoritative; only scheduled
script inputs and a finite experiment bound add wakes. Pending writer work uses
production worker/queue wakes. Physical desktop input is excluded from this
fixture and physical/HUMAN qualification is not inferred.

Two demonstrated first-edit integration defects are corrected: M picks up the
single current selected Footprint at the canvas cursor through the existing
preview builder; presented authored-object hits no longer intercept Move commit
as selection. Locked/stale refusal, complete geometry, canonical guarded worker,
selection reconciliation, source identities and original bytes are preserved.
No new mutation, snapping, persistence, timeout or selection mechanism is added.

The owner failure log is preserved with its exact host, Wayland/Vulkan adapter
and recovery-stop message. Recovery exhaustion has not reproduced in the corrected
frame-interleaved trial. The original six-second fixture redraw gap was waiting
for another redraw, not proof of six seconds executing M or GPU work; that failed
harness was retained and its scheduler corrected. Ordinary recovery deadline
transitions now retain their acquisition-attempt/active-time state once, without
per-frame tracing or a changed protection limit. The precise originating wait
in the owner failure remains unproven and tracked as
`dat-first-edit-native-recovery-dhj9`. Development availability stays withdrawn;
UED-C07 remains in progress pending reserved bounded review and a defensible
ordinary failure disposition. Prior journal/callback PASS remains scoped; no
successor or wider acceptance is authorized. Detailed outcomes and raw evidence
are in `docs/reviews/workflow-delivery-rollout/user-edit/first-edit-runtime-qa.json`.

### Accumulated Move input correction — retained first-edit delivery

Actual reserved independent bounded PASS for `bf34a97475be0f328f5072bc909e49982a6ce3a4`
and packet blob `70d4a58c708dc56dcd14d002cfd15a94015e5eb3` is received and
preserved in `first-edit-runtime-corrected-review.json`. Its independent native
frame workflow and seven recovery tests passed. This receipt covers the pickup/
dispatch correction; it does not resolve the owner's original recovery stop.

The same fixture additionally demonstrates the movement delay: 192 accumulated
production pointer events spend 6,276 ms on dispatch before the next native
round. The cause introduced in `fcb7633c` is preview source invalidation and
complete retained copper reconstruction on every motion event. The retained-scope
repair stores the latest existing snapped Footprint intent and prepares the whole
preview once per admitted native frame. It preserves complete geometry, final
click destination, cancellation and the canonical mutation path. The same input
batch measured 16 ms after correction, followed by the complete frame/edit/history/
Save/reopen workflow. These are observations, not ratified latency budgets.
Cancelled queued motion must not reappear on the next frame; focused verification
and reserved review cover that boundary. The original recovery exhaustion remains
unattributed, with no timeout relaxation or new acceptance claim. UED-C07 and
ordinary development availability remain open/withdrawn respectively.

Actual reserved follow-on independent PASS is received for source
`24b5971903615bf6ed441196a3e7622fe75f5c3c`, packet blob
`2152af64405eb000ddd5d6fa66d045f4ce9ed750`, with no mandatory findings.
`first-edit-motion-review.json` records independent production-tracing-OFF
Wayland/WGPU App replay: the accumulated 192-event batch took 19 ms; complete
preview, actual-identity queued cancellation, click commit, exact history, Save
and reopen passed. Six source and three ordinary binary pins were verified;
owner Project preimages remained unchanged. Unchanged writer/recovery evidence
was reused. The demonstrated dispatch repairs are delivered and reviewed.

The original recovery-stop host episode remains unattributed. Its launch-time
physical event sequence and originating queue/acquisition receipts are missing;
the current owner-copy fixture preserves post-QA source rather than reconstructing
unrecorded launch state. No further cause-changing edit is justified by available
evidence. Retain `dat-first-edit-native-recovery-dhj9` open and UED-C07 in progress;
ordinary availability, physical/HUMAN and wider acceptance remain withheld.
The rebuilt ordinary launch command is unchanged. No owner approval relay or
renewed scope grant is needed for continued diagnosis when concrete failure
evidence becomes available.

### Move preview presentation correction — owner repeat QA

Latest owner QA used the exact delivered GUI binary without inherited fault
injection. A journaled resistor move existed while the window showed the earlier
revision. Its captured log lacks the previous recovery error; that original
exhaustion remains separately open. Earlier fixture PASS proved prepared geometry
and canonical history, but did not assert a translated Footprint in an actually
presented frame. That visual inference is withdrawn.

The strengthened existing real-board native fixture reproduces preview upload
starvation: repeated motion replaces the complete source on every admitted
upload-continuation turn. The retained-scope correction keeps that source stable
until presentation, then schedules the latest queued snapped destination. Cancel
and canonical commit supersede it immediately. The same ordinary App Wayland/WGPU
trial now presents movement, cancellation, committed revision and Undo/Redo
positions, then saves/reopens exact bytes (one PASS, 75.77 seconds). Owner files
remain unchanged. Reserved correction review is pending; UED-C07, original recovery
attribution, physical-input/HUMAN and wider qualification remain open. No
availability acceptance, new mechanism, timeout or successor is inferred.

The owner-appointed first-edit manager received actual reserved independent PASS
for source `5155b41191988afa59973800ddd8c607774d3444`, packet blob
`f400b9f2106d5ba275d6810a02f069a4b448ea7e`, with no mandatory findings. The
tracing-off independent native replay passed presented motion/cancel/history and
exact Save/reopen (74.03 seconds); receipt is `first-edit-presentation-review.json`.
This dispositions the demonstrated presentation correction only. Original recovery
attribution and availability remain open; no UED-C07 acceptance or successor.

### Reviewed first-edit delivery restored — 2026-10-09

This outcome supersedes the current availability status of the earlier withdrawal;
its failure evidence and scoped historical receipts remain intact. UED-C07 is
complete on source `a365ae5ee4813c0280ccad1e770c45c7c9802d92`, packet blob
`4428877b00f4fbac4132c59ccd0d9815de104c24`. The actual reserved independent
`first-edit-journal-read-review.json` PASS and
`first-edit-completion-reconciliation.json` support bounded completion, with the
producer native workflow and prior independent replay reused at their scopes.
No new test run or independent execution is attributed to the latest source review.

The scoped ordinary-failure disposition is supported by reviewed selected-target
and pending-write guards, frame-coalesced preview, canonical writer repairs,
complete native Wayland/WGPU consecutive A/B move/cancel/commit, exact history,
Save/reopen and subsequent interaction without recovery exhaustion. The owner
now reports: “the edits for move object are much better now. we need to move on
to more of the development.” This is owner-observed movement improvement, not
complete physical-input/HUMAN qualification. The originating historical recovery
stop remains unproven and `dat-first-edit-native-recovery-dhj9` stays open.
Approximately one-second confirmation remains tracked in
`dat-first-edit-confirmation-latency-z9vn`; no instantaneous-response claim or
invented numeric acceptance budget is made. The stale structural ownership check
`dat-private-writer-journal-gate-0u2x` remains disclosed/unpassed; independent
reconciliation identified no demonstrated first-edit safety dependency. This
disposition does not waive separately required broader writer gates.

The previously owner-appointed manager approves bounded objective development
availability of this exact candidate under UED-C08. The refreshed
`first-edit-delivery.json` pins the matching ordinary GUI/CLI/daemon, current
read-only snapshot of the owner's editable Project, unchanged original KiCad
inputs, actual review receipts and focused workflow/safety logs. Select an existing
Footprint, press M, preview, Escape to cancel or left-click the destination to
commit; undo/redo, Save and reopen are delivered. Pending commit cannot pick up
another object or start another write. Geometry edits invalidate old occupied
copper; no refill, CAM waiver or stale-fill acceptance is supplied.

UED-C07 completion and explicit claim release are synchronized. Objective C08
acceptance is recorded; its formal closure publication alone remains pending an
explicit owner canonical-successor selection. The current selector requires one
actionable canonical item and rejects an all-complete canonical task, while this
grant authorizes no automatic successor. Consequently the existing C08 boundary
remains selected and unclaimed; no runtime work or additional proof is invented
to keep this implementation open. The smallest owner decision is to select and
authorize bounded readiness for the already-directed native library-backed symbol
placement/wire-connection delivery, or designate another existing Frontier task.
This does not grant successor execution. All full S5A/GWP/NA/C04–C06, physical
Wayland/HUMAN, CAM/T04, wider platform and full writer obligations remain pending.

### Owner-selected native schematic delivery readiness — 2026-10-09

The owner directs “remove the blockers and begin the next logical development
work” in response to the proposed bounded readiness for native library-backed
symbol placement and wire connection. This selects that readiness successor and
resolves UED-C08 publication. First-edit objective manager acceptance remains
pinned to `a365ae5e` and actual reserved receipts. USER-EDIT-DELIVERY lands with
its claim released; all broader qualification and historical defects remain open.

USER-SCHEMATIC-DELIVERY initially has bounded readiness authority. The existing
schematic/GUI authoring contracts control; full NATIVE-AUTHORING remains open.
No representation, dependency, import expansion or new product mechanism is granted.

<!-- REQ:USER-SCHEMATIC-DELIVERY:NSD-C01 -->
Reconcile actual symbol/library, wire, typed GUI, canonical writer/history and
Project setup owners; produce one concrete execution scope in this existing
delivery record. No general coverage matrix or fixture campaign.

<!-- REQ:USER-SCHEMATIC-DELIVERY:NSD-C02 -->
<!-- OWNER:USER-SCHEMATIC-DELIVERY:NSD-C02:SCHEMATIC-EXECUTION -->
Authorize the source-backed bounded implementation scope: existing native library
symbol selection, complete preview/cancel/placement and connected wire gesture,
canonical history/Save/reopen, necessary existing native sheet/pool setup, ordinary
Wayland/WGPU integration and reserved independent review. Reuse ratified contracts
and engine primitives; no private writer, new dependency, import expansion or
broad authoring acceptance.

<!-- REQ:USER-SCHEMATIC-DELIVERY:NSD-C03 -->
Implement authorized native schematic user behavior through shared tooling and
canonical asynchronous edits; preserve revision/cancellation and electrical identity.

<!-- REQ:USER-SCHEMATIC-DELIVERY:NSD-C04 -->
Verify focused cancel/commit/connected membership/history/Save/reopen and failure
safety in ordinary Wayland/WGPU; resolve actual defects and obtain reserved
bounded candidate review. Reuse adequate existing evidence.

<!-- REQ:USER-SCHEMATIC-DELIVERY:NSD-C05 -->
<!-- OWNER:USER-SCHEMATIC-DELIVERY:NSD-C05:SCHEMATIC-DEVELOPMENT-ACCEPTANCE -->
Disposition objective bounded development availability with actual proof and
independent review; no self-review, full qualification or automatic successor.

<!-- EVIDENCE:USER-SCHEMATIC-DELIVERY:NSD-C01-READINESS -->

NSD-C01 is complete against source `a4806669af33d813b7ad1a74c2a199585495f152`. The smallest missing behavior is
manual native-library symbol placement followed by connected schematic wiring
in the ordinary application, not a missing engine substrate. Source owners:

- `gui-protocol/src/native_edit_client.rs`: typed actions currently Move, Undo,
  Redo and Save only. Add schematic intent adapters to the same worker transport.
- `gui-app/src/runtime_native_edit.rs`: reuse asynchronous guarded execution,
  pending-write exclusion and atomic refreshed scene/selection publication.
- `engine/src/api/native_write/registry.rs`: currently aggregates project, board
  and waiver verbs; schematic adapters are missing from this route. Register
  adapters over existing builders, preserving fresh revision/lease validation.
- `engine/src/api/native_write/schematic_symbols.rs`: existing placement batch
  can atomically create the symbol and resolved ComponentInstance binding.
- `cli/src/commands/schematic/symbol_library_materialization.rs` and
  `symbol_mutations.rs`: existing complete pool-pin materialization and qualified
  entity/gate/part resolution must be reused/extracted into the engine owner;
  GUI must not invent a second binding algorithm or accept compatibility-only
  identifiers as equivalent to native library placement.
- `engine/src/api/native_write/schematic_connectivity.rs`: existing wire and
  Junction builders are reusable. Segment-only CLI behavior lacks auto-junction
  inference; implement the contract's wire gesture and same-batch T-junctions
  rather than silently substituting a disconnected line tool.
- `engine/src/api/native_write/schematic_sheets.rs`: existing canonical sheet
  creation and shard/history handling are available; no private setup writer.

Read-only inspection of the delivered DOA2526-first-edit Project finds
`project.json` pools empty and `schematic/schematic.json` sheets empty. Its board
and owner edits remain intact. Therefore library/sheet setup is a real first-use
prerequisite. It does not justify changing the frozen KiCad schematic importer
or hand-editing Project JSON. Reuse the existing native setup and library
authoring/attachment primitives on a separately Datum-owned schematic delivery
Project; preserve the current first-edit Project and original KiCad files.

FIRST-SYMBOL-WIRE-01 recommended execution scope:

1. Expose the existing qualified native library reads and symbol binding through
   the engine; extend existing typed writer adapters, avoiding new persistence,
   transport or source-model mechanisms. Establish a native sheet and an owned
   resolvable library using existing canonical setup capabilities. Library
   content/provenance must already be authorized; no new external dependency or
   unapproved imported library.
2. Ordinary schematic UI: choose native symbol, enter reference/value, preview,
   cancel without writes and click to place with complete pin identities and
   ComponentInstance binding. Use existing ratified library/tool surfaces; any
   protected prototype edit remains in Claude's lane.
3. Place Wire: snap to exact existing pins/grid, preview a connected wire gesture,
   finish/cancel under the existing contract, compose all segments and required
   T-junction markers as one canonical batch. Derived connectivity must show
   the intended pins on the same net. No tolerance or partial membership.
4. Reuse asynchronous interaction and first-edit history/Save/reopen. Pending
   edits cannot start another write; stale revision/cancel must not publish a
   false applied scene. Undo/redo restores complete objects and connectivity.
5. Focused ordinary Wayland/WGPU native proof on the delivered Project, isolated
   guard/cancel/history failure controls reusing existing fixtures, actual reserved
   independent candidate review, then pinned development delivery. Do not force
   X11 or claim physical HUMAN proof.

No existing first-edit defect or full S5A/NA proof is added as an implementation
prerequisite. This slice does not close full NATIVE-AUTHORING, GUI-WRITE-PATH,
S5A/C04–C06, CAM/T04, physical HUMAN, hierarchy/library-editor, other schematic
verbs, physical placement or routing obligations. Those remain explicitly open;
there is no broad acceptance or mandatory-scope waiver. No new tests or fixture
construction were needed to answer this readiness question.

Exact remaining decision: authorize FIRST-SYMBOL-WIRE-01 execution under the
existing schematic authoring and PM059/PM060 contracts, including necessary
canonical sheet/library setup and bounded GUI/engine adapters, focused native
verification and reserved review. NSD-C02 preserves the distinction between the
owner's selected readiness successor and the concrete production execution grant.

Readiness source SHA256 pins:
- `crates/gui-protocol/src/native_edit_client.rs`: `5e08ef53a9865bad7b303c9890a375106eb192dcb5321f049bf44238b33ac59e`
- `crates/gui-app/src/runtime_native_edit.rs`: `e19478714651e47bf3298c312b6595e894b4759ff8dfceb8245ef7bbb90dc27f`
- `crates/engine/src/api/native_write/registry.rs`: `c8f91d50b6d96b19907c739daa43902a012d32ac2889a41af0d138d736312083`
- `crates/engine/src/api/native_write/schematic_symbols.rs`: `b8b9a4487e43764465f7fb1e7ade6b99c0e15a65ec7bbbfa79da31d2034ef9bd`
- `crates/engine/src/api/native_write/schematic_connectivity.rs`: `d23266bcc241ec2f62013ae0d005839d294a35d70846b9431b86f8adc0f75821`
- `crates/engine/src/api/native_write/schematic_sheets.rs`: `35207d364ad3db11bb53deb6d69a635c3bf8df1cfc7ab898544c6f53c94b4a37`
- `crates/cli/src/commands/schematic/symbol_library_materialization.rs`: `a2e114efd497a28396100f737bc1148d08963ec4fb354fa8313b405a2e8b0f1b`
- `crates/cli/src/commands/schematic/symbol_mutations.rs`: `8e4b83521563eb1fd37af3e536c10ec418c3e45766d74a114fd79bc1010615da`
- `docs/contracts/SCHEMATIC_AUTHORING_TOOL_CONTRACT.md`: `7259766f1f8c0cd377347612120c0df044ccb522a7166bdbe91346b462ad66bc`

<!-- EVIDENCE:USER-SCHEMATIC-DELIVERY:NSD-C02-EXECUTION-GRANT -->

Owner explicitly replies `USER-SCHEMATIC-DELIVERY: approve FIRST-SYMBOL-WIRE-01`
after the concrete scope in `e17cbc52`. This grants NSD-C03/04 execution of that
exact native sheet/library setup, qualified symbol placement and connected wire
behavior, canonical asynchronous GUI integration, history/Save/reopen, focused
ordinary Wayland/WGPU verification and reserved independent review. It grants no
new dependency, representation, private writer, import expansion, tolerance or
broader acceptance. Existing manager appointment is retained at its bounds;
independent review and full qualification are not passed by approval.

<!-- EVIDENCE:USER-SCHEMATIC-DELIVERY:NSD-C03-DELIVERY -->

FIRST-SYMBOL-WIRE-01 runtime delivery is implemented under the explicit owner
grant. Engine unit `9cda8d99` shares historical qualified library binding and
materialization with the CLI; new adapters translate complete pins exactly and
compose all wire segments plus required T junctions. The ordinary GUI exposes
Place > Symbol with qualified choices, reference/value, preview/cancel/place;
Place > Wire or W acquires exact source pins or the shared user grid. Finish Wire
or repeat-click finishes; Escape finishes accepted wire vertices, while Cancel
discards the gesture. Pending writes exclude new edits. Existing worker, revision
and transaction-tip qualification, canonical history and Save/reopen are reused.
Native library graphics replace the compatibility IEC body for qualified native
symbols using the existing text/scene pipeline.

Actual reserved audit authority assessment in thread
`01a0e343-3d9c-70f1-899e-5bd239513383` confirms existing Manual GUI provenance
(PM059/authoring contract) and PM053 first births from strictly empty authored
topology are authorized integrations. This is an authority assessment, not an
implementation PASS. Tool/Assistant proposal controls remain; missing identities
in nonempty unadopted source never authorize automatic adoption. First-birth
operations reuse the existing final-partition planner and canonical journal.

Finite production App callback replay on native Wayland/WGPU Vulkan, Intel HD
Graphics P630/Mesa 25.0.7, placed U1/U2 with complete five-pin/part bindings,
verified preview/cancel byte preservation, acquired off-grid exact terminals,
committed three connected wire segments, undid/redid exact sheet bytes, Saved and
reopened with U1 OUT and U2 IN- in one authoritative physical Run. The ordinary
executable separately reopened/rendered that persisted result. Original KiCad
board/schematic and existing first-edit board/project SHA256 pins are unchanged.
Prepared native Project: `/home/bfadmin/Documents/datum_projects/DOA2526-first-schematic`;
canonical setup uses the already approved first-party native authored baseline
library. Verification mutated private copies only.

Preserved failed trials exposed swallowed selection-press placement, incorrect
Tool provenance for manual binding, and unavailable identities in nonempty
legacy source. Corrections respect the explicit old-source adoption boundary.
Replay-only missing script wake/off-canvas input trials are preserved separately;
they do not establish production scheduler failures. Engine contact/history and
first-birth/legacy-source checks, daemon guarded writer/provenance checks and
historical CLI binding proof supplement ordinary application evidence.

NSD-C03 is complete; NSD-C04 selects the actual reserved candidate review and
remaining focused safety checks. No implementation PASS, bounded development
acceptance, physical compositor/HUMAN proof, full authoring/S5A/C04-C06,
performance budget, CAM/T04 or automatic successor is inferred.

<!-- EVIDENCE:USER-SCHEMATIC-DELIVERY:NSD-C04-INDEPENDENT-PASS -->

Reserved independent audit session `01a0e343-3d9c-70f1-899e-5bd239513383`
returns actual bounded implementation **PASS** for source
`94c95ae66270b703fdbe1d17e9e2811f1932133e`, engine unit `9cda8d99`, reviewed
delivery packet blob `f00055c5a85b5a0242b5198d8293e7e049dd7af1`. No mandatory
source findings. Unmodified actual receipt is archived at
`docs/reviews/workflow-delivery-rollout/native-schematic/independent-review.json`,
SHA256 `740f7766f46a0d8914fbb5c0981b2e3fa11e423b20e4a6a2e207bce47d1b2aff`. Independent guarded production App
Wayland/WGPU replay on a private copy passed the declared symbol/cancel/place,
exact connected three-segment wire, complete physical Run membership, undo/redo,
Save/reopen workflow (one case, 7.53 seconds). Reviewer inspected ordinary
executable reopen capture and verified cohort/seed/original preimages. Producer
engine four cases (including Manual/Tool/Assistant negatives), daemon ten and CLI
eight are explicitly reused at their scopes, not independently rerun.

NSD-C04 is complete for the declared bounded integration/proof/review. Required
drift invocation did not pass: initial /tmp reserve refusal is preserved; exact
session-owned temporary Projects were moved to disk with evidence retained.
Subsequent Clippy executed and failed 13 lints in five unchanged baseline files,
captured as `dat-preexisting-engine-clippy-k2kn`. The remaining battery was not
reached. Existing GUI-supervision parity drift remains
`dat-first-edit-parity-drift-dlkt`. No gate PASS or waiver is claimed. Source
health and its 13 regressions, workspace/staged formatting, file lane, menu model,
project state, spec governance and traceability passed separately. Physical
compositor/HUMAN, full authoring/GWP/S5A/C04-C06, performance, CAM/T04 and all
other qualification remain open.

The reserved audit also confirms the actual delegation boundary: the owner
appointment at UED-C02 covers the first-footprint edit, excludes successor phase
delegation, and the schematic grant retains that appointment at its bounds.
NSD-C05's recommended manager response does not appoint a manager. Therefore
implementation and review need no renewed approval, but objective schematic
development acceptance must be dispositioned directly by the owner or by an
explicit extension of the named manager's scope to FIRST-SYMBOL-WIRE-01/NSD-C05.
No authority is inferred from the reviewer's assessment.

The implementation claim is explicitly released with C03/C04 complete and C05
selected at its owner boundary. The exact ordinary launch candidate is:
`/home/bfadmin/Documents/datum-eda/target/debug/datum-gui --project-root /home/bfadmin/Documents/datum_projects/DOA2526-first-schematic --focus-pane schematic`.
Use Place > Symbol, enter reference/value, Place and click the sheet; Escape
cancels preview. Place > Wire or W connects exact pins/grid vertices; Finish Wire
completes the batch. Ctrl+Z, Ctrl+Shift+Z/Ctrl+Y and Ctrl+S use canonical history
and Save. Original KiCad and owner board edits are preserved. No successor is
selected or authorized by this delivered candidate.

<!-- EVIDENCE:USER-SCHEMATIC-DELIVERY:NSD-C05-OWNER-ACCEPTANCE -->

The owner replies `USER-SCHEMATIC-DELIVERY: manager approve` to the pending
NSD-C05 packet for the only reviewed candidate, source
`94c95ae66270b703fdbe1d17e9e2811f1932133e`, packet blob
`f00055c5a85b5a0242b5198d8293e7e049dd7af1`, evidence commit `c5db9d4d`.
This explicitly approves bounded objective development availability. The recording
session `codex-user-edit-delivery-20261008` handles this owner-directed disposition;
no broader standing appointment or successor authority is inferred from the short
reply. Exact disposition and launch are preserved in
`docs/reviews/workflow-delivery-rollout/native-schematic/development-acceptance.json`.
Actual independent PASS and finite ordinary Wayland/WGPU proof are reused; current
GUI/CLI/daemon and four preserved source hashes were checked and still match.
No implementation change, rebuild or repeated proof is needed for this approval.

NSD-C05 is complete and `development_ready` is true for FIRST-SYMBOL-WIRE-01.
Designers can choose a qualified native symbol, enter reference/value, preview,
cancel or place it, connect exact pins with a wire gesture, undo/redo and
Save/reopen in the prepared native Project. The ordinary launch remains the
command pinned in C04. Original KiCad inputs and existing owner board edits are
preserved. Physical compositor/HUMAN, full authoring/GWP/S5A/C04-C06, CAM/T04,
undecided budgets and the disclosed Clippy/parity defects remain unpassed and
unwaived. Development availability is not complete qualification.

<!-- REQ:USER-SCHEMATIC-DELIVERY:NSD-C06 -->
<!-- OWNER:USER-SCHEMATIC-DELIVERY:NSD-C06:SCHEMATIC-SUCCESSOR-SELECTION -->
Record an explicit owner successor selection and publish synchronized closure of
this accepted delivery. This administrative boundary expresses the existing
no-automatic-successor contract and the selector's one-actionable-item invariant;
it adds no feature implementation or qualification prerequisite. C01-C05 remain
complete, with no live implementation claim. Do not reopen acceptance or repeat
proof while waiting. A successor requires its own existing authorization and
prerequisites; this owner acceptance supplies none.


<!-- EVIDENCE:USER-SCHEMATIC-DELIVERY:NSD-C06-DELEGATED-SELECTION -->
### Owner-appointed project management and successor disposition — 2026-10-09

The owner appoints this session to manage Datum agents and explicitly delegates
these successor decisions while away. Verbatim appointment and delegation are
recorded in the existing `native-schematic/development-acceptance.json` task
evidence, durable receipt commit `f7a4a465`. Named manager session
`codex-project-manager-20261009` uses available `/root` session transport.
This is an explicit owner appointment and delegated disposition under PM059,
not a fabricated human selection or an extension inferred from earlier acceptance.

For NSD-C06 the appointed manager records:
`USER-SCHEMATIC-DELIVERY: select successor — UVT-S5A-BUILD / S5A-C04, retained execution authorization only`.
FIRST-SYMBOL-WIRE-01 C01-C06 close at accepted source
`94c95ae66270b703fdbe1d17e9e2811f1932133e` and reviewed packet
`f00055c5a85b5a0242b5198d8293e7e049dd7af1`. Existing objective availability and
actual reserved product review are reused; no new product proof or acceptance.

The selected successor has existing execution authority, complete implementation
and an explicitly released prior claim. It requires a fresh synchronized
Frontier/beads claim before scheduled work. Selection does not resolve its reported
shared-desktop focus/concurrent-input proof blocker or pass remaining native
Wayland/HUMAN, N01-N17, C05/C06 or CAM/T04 obligations. No product semantics,
dependency, scope reduction, waiver, destructive action, claim takeover or new
successor execution grant is introduced. PM059 owner-reserved decisions retain
their bounds. Independent governance review of the actual disposition receipt
passed without findings; synchronized publication receives its own scoped review.


<!-- REQ:USER-SCHEMATIC-SYMBOL-MOVE:SM-C01 -->
### Manager-directed native Symbol Move delivery — 2026-10-09

The appointed manager records the owner's renewed instruction to remove them
from the development loop in `native-schematic/symbol-move-authority.json`,
under `docs/reviews/workflow-delivery-rollout/`. In direct response to the exact
reviewed proposal, the owner delegates the pending feature decision to management.
The manager authorizes and selects USER-SCHEMATIC-SYMBOL-MOVE / SM-C01 (dat-native-symbol-move-ydza);
this is a named delegated disposition, not a fabricated literal human approval.

Implement one unconnected native Symbol Move in a unique resolved occurrence:
ordinary selection and M/shared Move, pointer preview, Escape cancel or click
commit, canonical undo/redo and Save/reopen. The exact proposal
`symbol-move-proposal.json` has SHA-256 `5e4049a3629b8767955e9723938d3dd5facc91f0e8169496778bb6b1f3fe3dd5`;
`symbol-move-proposal-review.json` records actual independent proposal-readiness
PASS only, with no implementation acceptance. Translate origin and all absolute
pins by one checked delta; preserve identities, binding and metadata. Require
complete source-qualified unique occurrence and absence of external contacts in
both preimage and destination. Unknown basis, repeated/ambiguous occurrence,
positioned fields/overrides, stale revision and existing/new contact refuse before
writes. Use shared gestures and the engine-owned canonical writer/history route.

Release the clean S5A claim by its implementer's explicit handoff, leave C04
uncompleted/resumable and C05/C06 open, and synchronize the new developer lease
with tracker/export and generated Frontier. Connected/repeated/positioned Symbol
transformations remain open follow-ups; no broader authoring, HUMAN, Wayland,
CAM, numerical, dependency, source-health or visual-lane requirement is waived.
Continue implementation through focused ordinary proof, fixes and actual
independent candidate review, then manager objective disposition. No further
coverage-only S5A campaign is a prerequisite for this slice.


<!-- EVIDENCE:USER-SCHEMATIC-SYMBOL-MOVE:SM-C01-DELIVERY -->
### Bounded Move delivery and library-foundation priority — 2026-10-09

Appointed manager `codex-project-manager-20261009` records objective bounded Move
availability at `2ca54eaacee03158e9cc6a84778bbd98d8a3f6ba` after actual independent source/test review and ordinary
X11 keyboard/pointer preview/cancel/commit, undo/redo, Save and fresh GUI reopen.
Exact sources, binaries, journal, preserved inputs, focused engine guards and
retained failed attempts are recorded in `native-schematic/symbol-move-delivery.json`
and `symbol-move-delivery-review.json` under the existing review route. SM-C01
closes and its implementation claim releases; full qualification remains separate.

The input is the existing synthetic first-party `native_authored_baseline_v1`
test fixture introduced in `2049f22c`, not an approved production library or
manufacturer-qualified part. Git authorship and `reviewed_by: datum` are not actual
content review or owner approval receipts. This disposition approves Move behavior
only; library source/rights, content approval and production availability remain
unresolved. The owner's explicit foundation concern is recorded verbatim in
`symbol-move-authority.json` and intake `dat-native-library-source-authority-8xm7`.

Under existing delegated successor selection, the manager explicitly selects
`GUI-SURFACE-SPECS / SURFACE-S01` with retained planning authorization and a fresh
synchronized claim. Reconcile the library-first product foundation and complete
current schematic/library requirements inventory; do not invent a library adoption,
Approved object, distribution decision or implementation grant. S5A stays resumable
with C04/C05/C06, physical/HUMAN, CAM and broader qualification unpassed. No automatic
successor, scope waiver, dependency, protected prototype or new product mechanism.

The owner subsequently directs: `we should discuss library source.` Retain the
selected task's existing planning grant and claim for coordination/read-only
inventory. Hold scheduled specification drafting, library adoption/content creation,
implementation and approval-state changes until that discussion provides direction.
No component/source approval is inferred from this discussion request.
