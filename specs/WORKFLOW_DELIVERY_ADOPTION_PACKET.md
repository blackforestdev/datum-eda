# Workflow delivery quality: owner adoption packet

Status: WDQ-C04 packet approved at WDQ-C05; implementation/activation not authorized
Tracking: `dat-workflow-delivery-quality-xgj`

Owner response: `WORKFLOW-DELIVERY-QUALITY: approve adoption packet`, recorded
2026-09-06 UTC. PM041 records the exact reviewed revision and pre-ratification
hashes. The proposal below is retained as the approved scope; future-tense
implementation and acceptance requirements are not claims of completed work.

## What is being proposed

Approve the proposed PM041 delivery boundaries and exact gate contract, a
six-step bounded implementation/pilot, and the already scheduled foundation
research follow-on. This is **not** an assertion that a gate or pilot has run.
Approval permits a numbered ratification transaction, not automatic selection,
execution, CI activation or acceptance of Preferences or native EDA authoring.

The review packet consists of:

1. C01: `research/process-quality/WORKFLOW_DELIVERY_CAPABILITY_BASELINE.md` and
   its committed `evidence/wdq-c01-current/` archive, hashes, captures and logs.
2. C02: `research/process-quality/WORKFLOW_DELIVERY_FOUNDATION_RESEARCH.md`,
   F1–F8 owner/output map and scheduled FOUNDATION-WORKFLOW-SPEC contract.
3. C03: proposed `docs/decisions/PRODUCT_MECHANICS_041_WORKFLOW_DELIVERY_QUALITY.md`,
   `specs/WORKFLOW_DELIVERY_GATE_CONTRACT.md` and the refusal-case report.
4. C04: this bounded pilot, implementation contract and activation/resume plan.

## Expected improvement, with limits

Before coding, a feature author must turn intent into concrete user actions,
values, consumers and failure behavior, resolving required questions first.
Before enabling a control, its real production handler must work in the stated
context. Before claiming a workflow, native proof must connect actual input to
visible and persisted results. Before acceptance, another reviewer must replay
the required path and the owner must accept the exact reviewed packet.

The owner remains the product authority, but should review a bounded result and
its unresolved choices rather than discover basic integration gaps by repeatedly
re-auditing an entire implementation. The mechanism cannot guarantee design
quality, authenticate a person from JSON or certify enterprise compliance.

## Pilot scope: honest action availability, not a new EDA feature

Use the existing `dat-gui-local-readiness-uev` defect as the negative seed and
working View/Fit as the positive seed. The implementation owner first confirms
current action keys and all actual entry surfaces from the production registry.
Seed observations are from runtime source `6fa5aa3`, committed in `19d0758`;
rebuild and replay final pilot bytes. Do not accept the pilot from old captures.

Deliver a shared production consumer registry used by enablement and dispatch,
initially for the pilot action family. Help/About, View/Layers and
Window/Documents must not advertise an executable action where the actual
consumer is missing. Honest unavailable state with a reason is sufficient;
implementing these future panels is outside the pilot. View/Fit remains usable.
Review keyboard, pointer and context transitions against the same registry.
Do not turn all unknown actions on, or disable supported controls to make parity
trivial. Other action families remain explicitly outside this initial enrollment.

Native scenario proof must show:

| Scenario | Expected visible and model result |
| --- | --- |
| PILOT-S01 supported Fit | Pointer and actual supported keyboard entry reach the same production handler; deliberately zoomed board returns to fitted view; source/journal unchanged |
| PILOT-S02 missing consumer | All exposed pilot missing-consumer entries are unavailable with truthful explanation; pointer/keyboard cannot produce false success or mutate source |
| PILOT-S03 context/focus change | Re-evaluate eligibility on actual supported document/focus transitions; dismiss menu with Escape; no stale enabled action or trapped focus |
| PILOT-S04 failure and recovery | Unsupported key is refused without crash or private write; clean close/reopen preserves fixture state; no pending-edit recovery claim for these read-only actions |
| PILOT-S05 accessibility | Native focus order, keyboard operability and exposed disabled/enabled state agree with the rendered controls on the tested platform; screenshot alone insufficient |

If no keyboard entry exists for Fit, specify and review its access route before
claiming S01; do not fabricate a test-only shortcut. Undo, numeric precision and
library edits are N/A **only because this pilot changes no design state**. Source
and journal comparison prove that limit. Units/settings authority and actual
authoring remain with their existing owners. Linux/X11 native proof accepts only
that environment; other platform/backend claims require their own evidence.

## Proposed implementation completion contract

Frontier: WORKFLOW-DELIVERY-GATE-PILOT
Issue: `dat-workflow-gate-pilot-b3s`
WDQ adoption prerequisite approved; planning only until WDQ-G02 explicit authorization.
This successor is scheduled, not selected. Its gate code cannot certify itself
during bootstrap; existing gates and independent owner-run proof govern it.

<!-- REQ:WORKFLOW-DELIVERY-GATE-PILOT:WDQ-G01 -->
### Freeze the bounded implementation and handoff

Confirm numbered ratification, reviewed schema/refusal matrix, fresh current
task owners and exact pilot dispatch keys. Produce the pilot JSON contract with
real paths, handlers, input closure and scenario dimensions. Record owned files,
implementation/reviewer sessions and trusted-runner configuration. Review the
complete routes of every proposed GUI/menu/governance consumer edit first.
If reviewed authority contradicts the proposed unavailable-state treatment,
return that exact choice to the owner rather than changing the prototype.

<!-- REQ:WORKFLOW-DELIVERY-GATE-PILOT:WDQ-G02 -->
<!-- OWNER:WORKFLOW-DELIVERY-GATE-PILOT:WDQ-G02:WDQ-G02 -->
### Authorize the exact implementation window

Owner chooses a fresh handoff after the currently active task's bounded commit.
Record whether its owner continues on nonoverlapping files or pauses, exact
remaining step/claim disposition, pilot enrollment and the resume condition.
Approve the exact G01 contract and no dependency additions. Never reuse the
historical GP-CM01 checkpoint as though it were still current. Broad WDQ packet
approval is not this execution authorization.

<!-- REQ:WORKFLOW-DELIVERY-GATE-PILOT:WDQ-G03 -->
### Implement delivery validation in report-only mode

Implement schema, hashing, safe path resolution, checkpoint integration and
trusted receipt validation exactly as proposed PM041. Write and run N01–N20 and
P01–P07 as hermetic tests, including staged/worktree divergence and trusted-runner
bypass attempts. Extend existing completion machinery; no second roadmap.
Gate defaults remain report-only while bootstrap is unaccepted. Keep unknown
schema rejection and all existing PM025 output/claim/owner tests passing.

<!-- REQ:WORKFLOW-DELIVERY-GATE-PILOT:WDQ-G04 -->
### Implement and prove the native consumer pilot

Make the bounded production readiness correction through a single registry used
by actual dispatch and enablement. Prove PILOT-S01–S05 on fresh identified bytes
with a sanitized real-project-derived fixture. Run current relevant engine/UI
tests and report-only delivery checks, preserving source health and all existing
governance gates. No full Preferences or authoring implementation is included.

<!-- REQ:WORKFLOW-DELIVERY-GATE-PILOT:WDQ-G05 -->
### Obtain independent replay and activation evidence

Handoff proof to a distinct reviewer session/person not involved in implementation
or original proof; record separation, replay commands/input/artifacts and actual
findings. It must rerun mandatory native scenarios and inspect each N/A and
relevant-input closure. Fix and replay blocking defects in the owning lane.
Demonstrate relevant edits invalidate acceptance, unrelated edits remain valid,
infrastructure can finish narrowly, and selection/claims are unchanged. Prepare
the exact receipt digests and activation candidate; do not self-approve review.

<!-- REQ:WORKFLOW-DELIVERY-GATE-PILOT:WDQ-G06 -->
<!-- OWNER:WORKFLOW-DELIVERY-GATE-PILOT:WDQ-G06:WDQ-G06 -->
### Accept the pilot and activate only its enrollment

Owner reviews final native proof, independent disposition, full refusal results,
trusted-runner setup and remaining defects. Record explicit acceptance and
activation of the exact pilot enrollment and source revision. Promote the
receipt/policy authority through the owner-controlled boundary, enable scoped
blocking checks, prove them with the trusted runner and record the former task's
resume disposition. Only then may this successor close with its landing commit.
No automatically enrolled future features or product-acceptance claims beyond
this read-only availability pilot. Expansion requires another explicit decision.

## File ownership and size budgets

| Lane | Exact starting files / budget and constraint |
| --- | --- |
| Gate implementation | New `scripts/workflow_delivery_contract.py`, `scripts/check_workflow_delivery.py`, focused `scripts/test_workflow_delivery_*.py`; target under 350 physical lines each, absolute PM022 ceilings remain controlling. Split by schema, evidence and trust responsibility before growth, not arbitrary file-count targets |
| Existing selector | `scripts/project_status.py`, `scripts/project_task_details.py`; thin calls only. `scripts/test_project_status.py` was 699 lines at planning time: add new focused tests rather than extend it past its 700-line ceiling |
| Hook/CI | `scripts/git-hooks/pre-commit`, `scripts/run_drift_gates.sh`, `.github/workflows/alignment.yml`; coordinate owning gate lanes and review complete routes. Never replace existing checks with WDQ-only proof |
| Native pilot | `crates/gui-protocol/src/gui_menu_model.rs`, `crates/gui-app/src/runtime_menu_actions.rs`, `crates/gui-app/src/runtime_view_actions.rs`; identify keyboard consumers during G01. New registry module only if needed to maintain a single source and PM022 budgets; do not add dependencies |
| Governed inputs | Pilot JSON/policy, governed decision/contract, relevant menu inventory and evidence-route records; owning-lane review before mutation. Register any new parity inventory in the existing governance/parity machinery |
| Visual truth | **Claude only**. If needed, bounded handoff for `docs/gui/prototypes/board-editor.html`, `.menubar` Help/View/Window region: represent only reviewed availability, preserve shell/units/selection authority, and provide exact native/reference proof. No Codex HTML annotation or golden refresh |

No compiler proof target in `/tmp`; Cargo work uses `run_cargo_guarded.py`, serial
proof builds and exact-file Rust formatting. Screenshots/fixtures use isolated
headless displays. Never open a topmost temporary window on the owner's desktop.

## Migration, repair and concurrent-work transaction

1. Preserve current history as a legacy baseline; do not label it accepted by the
   new mechanism. Existing defect records and rejection history stay intact.
2. At G02, read fresh selector/claims and Git status; agree file ownership at the
   current owner's commit boundary. Leave its selected task unchanged unless
   the owner explicitly chooses a canonical handoff. Parallel permission, if
   chosen, is recorded in Frontier and does not allow overlapping files.
3. Build/report-only-test the pilot, then independently review it. Before G06,
   existing gates are blocking and proposed WDQ output is diagnostic only.
4. Owner accepts exact packet and promotes trusted authority; register only the
   pilot enrollment, schema migration, hook/CI configuration and authorized
   claim/resume state in one reviewed transaction. No later task autoselection.
5. A new gate failure stays with its owning lane. Diagnose with report-only output
   without disabling enforce mode. An implementation defect is repaired under
   the same reviewed contract; a policy defect returns for an explicit owner
   amendment. Never use `--no-verify`, delete enrollment or refresh unrelated
   authority as a repair. Owner-authorized rollback restores the last reviewed
   policy/runner and records affected claims; no silent downgrade to accepted.

Closure proof: all N/P tests pass with non-mutating refusals; all PILOT scenarios
pass on identified native inputs; independent review approves; owner acceptance
and enrollment are exact and trusted; schema/selector regressions, source health,
spec governance, evidence traceability, progress coverage, file-lane, staged
formatting and relevant GUI/menu tests pass. The issue closes only against those
results. Full EDA workflow and external publication remain excluded.

## Check whether this is helping

For the pilot and next three **separately enrolled** slices record: required
manual scenarios passed/total; defects found before versus after owner review;
owner-requested behavior corrections after “ready”; and reopenings due to missing
consumer behavior or stale acceptance. Classify newly requested scope separately
from failure to meet agreed intent. Existing C01 is a qualitative baseline, not
a fabricated historical correction count. After those slices the owner decides
whether to expand, simplify or revise the gate. Documents/lines/commits/closed
beads are not maturity measures.

## WDQ-C05 decision requested

Approve this packet to proceed to numbered ratification and the separately
authorized G01–G06 implementation sequence, or name corrections/defer. Approval
does not close unanswered F1–F8 domain questions, turn on blocking gates or accept
a native pilot that has not yet been built. The live WDQ selector presents the
canonical response format and decision boundary.
