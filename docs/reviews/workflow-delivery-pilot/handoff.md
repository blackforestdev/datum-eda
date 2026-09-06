# WDQ-G01 operational handoff candidate

Recorded: 2026-09-06 UTC. Tracking: dat-workflow-gate-pilot-b3s.
Status: G01 planning handoff complete; G02 execution decision pending.

This is an operational record under the ratified PM041 gate contract and
WORKFLOW_DELIVERY_ADOPTION_PACKET, not a new product specification. It is outside
the pilot authority route: future session/receipt updates must not create a
proof/authority hash cycle. Candidate behavior is in
`specs/workflow_delivery/pilot.contract.json`, committed in `b64a79ba`.

## Current ownership observed

At `5c6ce3d9b0247ea1a36ad0cf7fe70a80c131fb81`, the worktree was clean. The
successful named selector reported WDQ-G01 in progress, planning authorization,
with live session `codex-wdq-g01-planning-20260906`. A clean worktree is not a
release of another session's claim.

GLOBAL-PREFERENCES-COMPLETION retained its canonical execution claim under
`codex-gp-cm03-execution-20260905`. Its declared scope is the GP-CM03 product
service, trusted-human/proposal authority, CLI/MCP/daemon parity, writer model
and eight-key Project Units genesis. Nothing here pauses, completes or transfers
that work. Refresh both claims and dirty/staged paths at the actual G02 window.

## Proposed implementation file handoff, not a lease

| Responsibility | Paths and boundary |
| --- | --- |
| Delivery validator | New scripts/workflow_delivery_contract.py, scripts/check_workflow_delivery.py and focused scripts/test_workflow_delivery_*.py, as already specified by the adoption packet; no code written in G01 |
| Selector integration | scripts/project_status.py and scripts/project_task_details.py; preserve selection, claims and byte-for-byte output; focused new tests instead of growing test_project_status.py |
| Staged/standing checks | scripts/git-hooks/pre-commit, scripts/run_drift_gates.sh, .github/workflows/alignment.yml; preserve every existing check and use the trusted revision for final enforcement |
| Actual action admission | crates/gui-protocol/src/gui_menu_model.rs; crates/gui-app/src/runtime_menu_actions.rs, runtime_view_actions.rs, workspace_keyboard.rs; crates/gui-render/src/menu_chrome.rs. One production readiness source must serve dispatch, paint and accessibility |
| Camera eligibility integration | crates/gui-app/src/runtime_camera_pane.rs; existing focused-camera resolver, not a separate menu-only context heuristic. Source-health review/extraction is required before touching oversized code |
| Shared native accessibility handoff | crates/gui-app/src/terminal_accessibility_bridge.rs and terminal_accessibility_platform/{worker.rs,atspi.rs,events.rs,atspi/properties.rs,atspi/introspection.rs,atspi_tests.rs}; exact final diff scope requires the current consumer owner's agreement and complete owning-route review before edits |
| Visual truth | No prototype change proposed. All docs/gui/prototypes/*.html remain exclusively in Claude's lane; no marker transfer or golden/digest bypass |

The current planning lease authorizes none of those implementation edits.
Module additions, registration changes and source-health extractions must be
named in the final implementation lease, not assumed from this starting map.
The inactive FitBoard arm in main.rs does not justify editing that monolith or
restoring its removed button. Preferences runtime, engine, CLI and MCP semantics
remain with their owners; no blanket unknown-action disablement is permitted.

## Hook authority review completed

Read the complete visual-truth-file-lane-governance route: its enforcement
packet, AGENTS.md, pre-commit hook, file-lane checker and tests. Reviewed route
digest: 83178dae95c68fbe991b71b56ade930fb0265f3173ebb19681f318acc1d53b5c.
No route member was edited and no authority digest was refreshed.

The configured local hooksPath is scripts/git-hooks. The hook runs staged lane
validation before exec'ing staged rustfmt. A future third check must preserve
both checks and their failure behavior; it cannot be appended after an exec and
claimed reachable. It must inspect index bytes, not another session's unstaged
work. Existing test coverage includes nonmutation and lane-before-format order;
this planning review did not execute or alter the protected-lane marker tests.

## Trusted-runner configuration handoff

Inspection found no WDQ invocation or authority/base input in the checked-in
hook, drift runner or alignment workflow. No externally configured owner trust
was verified. The existing source-health CI base is not WDQ approval authority.

Use the PM041 fallback: bootstrap remains report-only under existing blocking
gates, with independent owner-run verification until an owner-controlled runner
is established. No CI enforcement or tamper-proof approval claim is justified.
At G06 the owner selects/promotes the exact trusted authority commit externally;
the trusted runner executes that revision's validator against the candidate and
receives the comparison base independently. Never derive authority from candidate
HEAD, repository policy fields or an implementing session's chosen receipt.
No trust values, Git settings, CI variables or activation flags are installed here.

## Session separation at the initial handoff

- Planning session: codex-wdq-g01-planning-20260906.
- Proposed implementation/original-proof producer: this session only if explicitly
  named in the final G02 handoff and synchronized execution lease. No execution
  has occurred and a future session change must be recorded before implementation.
- Independent replay session/person: **not assigned**. The owner was asked to
  identify it. It must not participate in implementation or original proof, must
  replay every mandatory native scenario, and must inspect input closure and N/A
  dispositions. A fabricated reviewer ID is not a handoff.
- Current Preferences owner handoff: **not confirmed**. At G02 record its bounded
  commit, exact continue-on-nonoverlapping-files or pause disposition, retained
  step/claim and resume condition. Historical GP-CM01 is not that checkpoint.
- Trusted owner-selected authority/base values: intentionally not set during
  bootstrap; only the owner-controlled boundary can promote them at activation.

The unassigned/unconfirmed entries above describe the initial ed1ef716 handoff.
The owner disposition below supersedes those two missing coordination choices.

<!-- EVIDENCE:WORKFLOW-DELIVERY-GATE-PILOT:WDQ-G01-HANDOFF -->
## Owner-reviewed coordination and completed planning handoff

The owner first replied `I reviewed the handoff markdown please proceed.`
This session then asked: `One coordination choice remains before I can close
WDQ-G01: may I use a fresh, independent review-only session and let the Preferences
session continue on nonoverlapping files, with shared accessibility files held
until an explicit handoff?` The owner replied exactly `yeah, peoceed`.
Recorded 2026-09-06 UTC from this conversation. This approves the reviewed
handoff and that coordination arrangement, not unperformed proof or G02 execution.

- Independent lane actually created: `/root/wdq_independent_review`, fresh context,
  review only. Receipt-facing session label: `wdq-independent-review-20260906`;
  this label maps to that actual lane, not a fictional reviewer. It acknowledged
  document intake only, no implementation/original proof, no file/tracker edits
  and no owner approval. No blocker to reservation was reported. G05 is not started.
- Intended implementation/original-proof producer remains
  `codex-wdq-g01-planning-20260906` (this session), subject to G02 execution
  authorization and its synchronized lease. Any replacement of either session
  requires a recorded handoff before participation; separation must be preserved.
- Preferences continues under its unchanged claim on nonoverlapping files.
  At this transaction its daemon dispatch.rs, main.rs and preferences_state.rs
  changes were already staged by the other lane; none belongs to this commit.
  No blanket pause, completion claim or canonical-task transfer occurs.
- The complete shared accessibility path group in the table remains held,
  excluded from an executable lease until an explicit bounded handoff with its
  consumer owner and complete owning-route review. A future newly overlapping
  path also stops at that boundary. This is the approved reservation, not a claim
  that the other session has already transferred those files.
- Initial implementation window proposed for G02: report-only G03 validator and
  selector integration on the listed nonoverlapping script/governance paths.
  G04 native work must observe the held-file condition; no native acceptance can
  skip S05 because accessibility work is held. Preferences needs no resume action
  while it continues; if an actual overlap later requires a pause, record the
  bounded commit, retained step/claim and explicit resume condition first.
- Trusted-runner disposition is the report-only/owner-run fallback above. No
  candidate-selected trust values or CI enforcement are installed. G06 must
  establish the external owner-controlled trust boundary before activation.

G01 completes its planning deliverables: ratified contract and real input paths,
four-consumer/five-scenario candidate, reviewed hook authority, named independent
lane, file reservations and trusted-runner fallback. It does not assert native
readiness, completed implementation or acceptance. The sole selected successor
is G02 for the exact bounded implementation authorization, including review of
the candidate's keyboard-access scope limit. G06 remains the separate activation
decision. The parent issue remains open.
