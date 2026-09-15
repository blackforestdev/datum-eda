# Workflow delivery pilot operational handoff

Recorded: 2026-09-06 UTC. Tracking: dat-workflow-gate-pilot-b3s.
Status: G01–G03 complete; G04 in progress under a bounded nonoverlapping claim;
activation remains off.

## G04 shared accessibility handoff authorized

Recorded 2026-09-06 19:42 UTC at clean HEAD
`3f377a7a61a3691e9ede564d8fe4ea3b83ce8ba5`. Preferences completed GP-CM03 and
released its claim; GP-CM04 remains an owner decision before GP-CM05. The owner
reported the other session paused, then answered `the working tree is clean.
please proceed.` to the explicit request for temporary WDQ-G04 ownership of the
held shared accessibility files, preservation of Terminal/Preferences behavior,
and the other session leaving those files untouched until handback.

This releases only the previously held native accessibility group for additive
menu publication, plus its focused modules/tests and frame-refresh connection.
It supersedes the historical hold statements below. The live WDQ lease records
the exact scope. No GP-CM05 approval, Preferences task transfer, prototype edit,
dependency addition or G06 activation is implied. At handback, record commits,
validation, remaining defects and whether any shared files are still dirty;
Preferences may resume only under its own owner authorization and file handoff.

### G04 production observation and capture tooling checkpoint

The next bounded implementation adds `DATUM_ACTION_EVIDENCE=1` observations to
the real menu pointer, menu keyboard and editor-shortcut call sites. The stream
exports the actual production registry for observed contexts, records invocation
only at dispatch, and records focus, selection, layout, pane cameras, scene
bounds, layer filters and Console feedback. It writes diagnostic stderr only;
there is no injected action, new project writer, readiness override or acceptance
flag. Disabled diagnostics do not build snapshots. A broken diagnostic pipe
cannot panic the application.

`scripts/workflow_delivery_pilot_capture.py` runs explicit physical inputs on a
private Weston/Xwayland display and accessibility bus, using installed system
tools. It pins the CLI executable, copies the archived native fixture, records
binary hashes and raw observations, sends normal WM_DELETE_WINDOW close requests,
and enumerates changed/removed and newly created project paths. It neither
installs dependencies nor produces an acceptance verdict. The sibling scenario
module records the inspected scale-one input coordinates; the X11 helper follows
the installed Xlib ABI. No prototype changes or Preferences approval are included.

Proof before collecting final scenario evidence: GUI app tests 305 passed with
eight existing tests ignored; strict all-target app Clippy passed; workflow
delivery/tooling tests 86 passed. The isolated harness smoke opened View,
collected actual native menu and Terminal accessibility nodes, captured the
native window and closed with exit zero. Archived files were unchanged; runtime
sidecars were enumerated. The smoke was on development bytes and is not a final
PILOT-S01–S05 result. G04 remains in progress pending identified scenario evidence.

### G04 additive menu accessibility implementation checkpoint

The bounded shared-file change projects currently open production menu rows
through the existing Linux accessibility service. Stable menu identities,
labels, focus, enabled/sensitive state and unavailable descriptions derive from
the actual menu inventory and contextual action admission. Dismissed menu paths
fail closed. Menu publication retains independently cached Terminal snapshots,
Preferences/New Project nodes and Console announcements; no settings or design
writer was added. Menu accessibility is read-only, not a new Action invocation
surface or a whole-application keyboard-only access claim.

Read-only implementation review identified one blocking event-index defect:
coalescing menu dismissal with Preferences publication used the new root sibling
offset for the removed menu. The worker now retains previous and next offsets;
removals use the previous index and additions use the next. A focused regression
covers both directions. This review is not WDQ-G05 independent native replay.

Verification at this checkpoint:

- Guarded offline/locked GUI app, protocol and render library/binary tests:
  571 passed (303 app, 114 protocol, 154 render), eight existing app tests ignored.
- Strengthened the publication regression to retain nonempty Preferences nodes,
  then reran all accessibility tests inside a private `dbus-run-session` with
  desktop display variables unset and `--include-ignored`: 30 passed, zero
  ignored, including real accessibility-bus registration. No desktop GUI opened.
- Guarded offline/locked Clippy for the three GUI packages, all targets with
  `-D warnings`: passed.
- Workflow delivery validator tests: 82 passed. Selector/claim tests: 50 passed.
  Source-health regression tests: 13 passed.
- Source health: 1785 files passed; evidence traceability: 20 routes/114 artifacts
  passed; dependency authority, specification governance (203 specs), project
  state (50 Frontier items), generated Frontier and whitespace checks passed.

The initial Cargo resource preflight refused insufficient `/tmp` reserve. With
explicit owner authorization, inactive `/tmp/datum-ai-disc-target` compiler
artifacts were moved intact to
`/home/bfadmin/Documents/datum-build-archive-NZ8vkV/datum-ai-disc-target`, freeing
4.2 GiB; nothing was deleted and no resource policy was bypassed.

This is an implementation checkpoint only. Native PILOT-S01–S05, production
registry/dispatch evidence export, identified binary/input receipts and complete
`proof.json` remain outstanding. G04 stays in progress, G05/G06 are not advanced,
and enforcement stays off. The bounded shared accessibility lease remains with
WDQ pending native verification and explicit handback; committing this checkpoint
does not approve GP-CM05 or release overlapping files to concurrent edits.

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

<!-- EVIDENCE:WORKFLOW-DELIVERY-GATE-PILOT:WDQ-G02-AUTHORIZED -->
## Bounded implementation authorization

The owner replied exactly `WORKFLOW-DELIVERY-GATE-PILOT: authorize bounded implementation`
in this conversation on 2026-09-06 UTC, after G01 closure commit 9e04a702 and
presentation of the G02 decision packet. This completes G02 and authorizes the
reviewed G03–G05 build/proof scope with G03 as the sole selected execution step.
G06 remains the separate blocking activation/acceptance decision.

The current implementation window begins from 46764e7cb042cd65e4c0eafcfb6aec9e3901015d,
with a clean worktree observed before claiming. Preferences retains its own
canonical claim and continues on nonoverlapping files; there is no pause to resume.
Shared accessibility files remain held exactly as approved above. Any overlapping
change requires an explicit bounded handoff before editing; authorization does
not erase that condition or allow S05 to be waived.

Implementation/original-proof session: codex-wdq-g01-planning-20260906.
Independent replay lane: /root/wdq_independent_review, review only, as mapped above.
The execution lease starts with focused dependency-free JSON/path/hash input
handling in scripts/workflow_delivery_io.py and scripts/test_workflow_delivery_io.py,
then the already reviewed schema/evidence/trust modules and selector integration.
Neither schema migration nor validator completion is asserted by this authorization.

### G03 first implementation unit (not completion evidence)

`scripts/workflow_delivery_io.py` now provides read-only strict UTF-8 JSON parsing,
canonical JSON and raw-byte hashing, normalized repository paths, and contained
regular-file reads. Duplicate keys, nonfinite numbers, lone surrogates, path
escapes, missing files and broken/cyclic symlinks are refused. The companion
test module passes 13 hermetic tests; these are input primitive tests, not a
claim that N01–N20/P01–P07 have passed.

Verification for this unit: 45 project-status regression tests and 13 source-health
governance regression tests pass; project-status (50 items), evidence traceability
(20 routes / 114 artifacts), spec governance and progress coverage checks pass.
Source-health checks pass. No Rust build or native GUI proof was run for this
dependency-free Python unit.

G03 remains in progress. Closed-shape contract validation, evidence and freshness
validation, independent-review/owner-receipt validation, trusted revision and Git
index handling, report-only CLI, selector integration and the full refusal/positive
matrix remain outstanding. No hook, policy enrollment, Frontier schema, GUI
behavior or acceptance authority changed in this unit.

<!-- EVIDENCE:WORKFLOW-DELIVERY-GATE-PILOT:WDQ-G03-VALIDATED -->
## Report-only delivery implementation and hermetic validation

Recorded 2026-09-06 UTC. This supersedes the first-unit outstanding-work list
above. The validator and integration are implemented; no production registry,
native PILOT-S01–S05 run, G05 replay or G06 activation is asserted here.

The dependency-free implementation separates closed contract/evidence shapes,
authority resolution, Git candidate views, input/build freshness, typed evidence
correlation, independent replay/owner receipts and Frontier integration. New
modules remain below the 350-line target; existing selector modules remain below
700 lines, with no legacy source debt or dependency addition.

Working-tree, staged-index and explicit-commit views are separate. Required
artifacts must be present in the candidate Git view; untracked production inputs
are included in worktree input manifests. Index reads never consult unstaged
files. Full PM025 validation/projection is reused in an isolated temporary
metadata view for enforcement; candidate files, index, configuration and authority
records are not written. Temporary metadata is removed on exit.

### Implemented evidence transport

These are the validator/runner serialization interfaces implementing PM041, not
additional product decisions or evidence that the native producer exists:

- A hashed result artifact with `kind: datum.workflow-delivery.artifacts/v1`
  binds every sibling artifact to explicit events/captures/state/registry roles.
  Event roles—not filenames or identical screenshots—control replay independence.
- Event records bind scenario, producer, input method/actions, binary and current
  authority hashes to production dispatch observations and visible/state results.
  Registry records bind tested binary, actual keys/handlers, entry surfaces and
  contextual eligibility. G04 must emit these from production, not maintain an
  independent contract-shaped registry or substitute CLI edits for native input.
- The requested environment is supplied explicitly by `--environment-path` and
  compared exactly with the proof environment: OS, backend, toolchain, scale,
  input method, window size and reproduction commands. No equivalence is inferred.
- Owner receipts are extracted only from the uniquely marked governed section,
  containing the exact ACCEPT line plus `Source:` and `Date:`. Trusted defect
  dispositions explicitly name `RESOLVED <issue>` or `DEFER <issue>`; a blocking
  resolution also binds `REPLAY <replay-blob-sha256>` and requires passing replay.
- Enforcement requires full owner-selected authority/base commit IDs and the
  trusted revision's gate bytes. Candidate policy/gate/contract weakening,
  disappearing enrollment and regressed completed obligations refuse. Changes
  to reviewed build inputs cannot evade proof by leaving activation pending.

Local hook order remains lane gate, staged rustfmt, then staged **report-only**
delivery diagnostics. Existing blocking behavior is preserved. Drift runs add the
hermetic suite and report-only diagnostics; CI adds no owner trust value or
enforcement switch. G06 must establish the external trusted runner/promotion.
The optional owner-controlled clone configuration keys consumed by ordinary
selector checks are `datum.workflowDeliveryAuthorityRef`,
`datum.workflowDeliveryBaseRef` and `datum.workflowDeliveryEnvironmentPath`;
none has been installed by this lane.

### Verification and limits

`python3 -m unittest discover -s scripts -p 'test_workflow_delivery_*.py'`
passes 82 tests. The N/P matrix is exercised with tiny hermetic records; native
production-path demonstration remains G04, and independent native replay remains
G05. Tests do not certify that prose N/A dispositions or screenshots are truthful.

| Refusal/positive cases | Focused test ownership |
| --- | --- |
| N01–N04; P01 | Input, contract, authority and CLI tests |
| N05–N08; P02–P04 | Typed production-consumer/correlation protocol tests; synthetic, not native runs |
| N09–N12, N19; P05 | Artifact/input/build freshness, index/worktree and authority-change tests |
| N13–N15; P06 | Distinct replay, same fixture/binary, copied-event refusal, exact trusted receipt and defect disposition tests |
| N16–N18; P07 | Full Frontier/claim/selection, retained enrollment/completion, candidate tampering and explicit trust tests |
| N20 | Explicit requested-environment mismatch tests; no implicit equivalence |

Additionally: 45 existing project-status tests, five claim-state tests and 13
source-health governance tests pass. Evidence traceability (20 routes / 114
artifacts), spec governance, progress coverage, project status (50 items), Cargo
resource policy and whitespace checks pass. Source-health passed at 1,763 files
after the other lane corrected its transient oversized Preferences module.
Schema-5/6 legacy next/details output is tested byte-for-byte unchanged.

The full drift battery was attempted: delivery tests and Cargo-resource checks
passed, then it stopped at Preferences-owned rustfmt failures in
`crates/engine/src/preferences/mod.rs` and `project_genesis.rs`. A separate parity
check still reports the concurrent engine API inventory at 203 versus its
documented 201. This is **not** a green full-repository or native-GUI verdict.
Those files and their parity authority remain with the Preferences owner; this
lane has not formatted them, changed their inventory or absorbed their failure.

The reserved independent review lane performed read-only code inspection. Its
findings about completion regression, disappearing enrollment, fixture/binary
identity, defect disposition and full Frontier validation were corrected with
regressions. Its bounded final recheck found no remaining blocker in those fixes;
it did not run tests, produce original proof or perform G05 replay/acceptance.

### G03 landing and synchronized continuation

Implementation landed in `9cb42473`. The completion transaction records its
committed validation evidence, migrates the live Frontier to supported schema 6
without adding any delivery enrollment, releases the finished G03 lease and
selects G04 pending. Existing task history, canonical Preferences selection and
other claims are unchanged. The pilot issue remains open under the already
recorded G02 build/proof authorization; no new approval is inferred.

Before claiming G04, refresh the production file map and declare its exact scope.
Editing any held shared accessibility file still requires the explicit bounded
consumer-owner handoff; a nonoverlapping scope does not release that hold.
The separate reviewer remains reserved for G05 native replay. G06 still requires
owner acceptance and external trusted-runner promotion; report-only operation
must not be described as activated enforcement.

## G04 production admission: first implementation unit

Recorded 2026-09-06 UTC. The owner replied `pkease proceed` after confirming
that the other session was still developing Preferences. The refreshed selector
selected G04 with execution authorization and no claim. The synchronized lease
now names the menu/protocol/runtime/render files and the shared viewport camera
resolver; the implementation/original-proof session remains unchanged. No
Preferences claim, selected step or authorization was transferred.

The production registry owns exactly the four reviewed pilot keys, typed Fit
handler identity, actual entry-surface identities and stable unavailable reasons.
Menu paint and invocation consult the same admission function; editor F reaches
the same dispatch instead of directly bypassing it. Missing consumers stay
unavailable, and missing registry keys never acquire handlers. Other GUI-local
families retain their existing dispatch ownership, including Preferences, pane
navigation and individual layer controls.

Shared viewport camera-scene resolution lives in `workspace_interaction/camera.rs`,
not in the menu model. Readiness and actual camera routing consume that resolver;
hidden, stale or unresolved panes cannot silently target another Board. Menu
activation/refusal restores editor focus before a supported action establishes
any new overlay focus. Existing Preferences submenu navigation tests remain in
place. No prototype, golden, menu inventory, dependency or source-health policy
changed. All touched source modules remain within normal budgets.

The reserved independent reviewer inspected this bounded diff read-only and
reported no concrete blocking defect. It ran no tests or native input and did
not perform G05 replay or acceptance.

### Remaining work and held ownership

This unit is not G04 completion. Production registry export, native event/state
correlation, accessible menu publication, proof-grade GUI/CLI build receipts and all
PILOT-S01–S05 native evidence remain outstanding. Unit/render construction tests
do not replace those observations. No `proof.json`, `review.json`, policy
enrollment, trusted setting or owner acceptance is created here.

The owner was asked to obtain a bounded Preferences consumer-owner handoff for
the shared accessibility files already listed above, naming its current commit,
retained ownership and in-flight changes. Those files remain untouched and held;
this request is not a release. The other session may continue on nonoverlapping
files. No native window has been opened on the owner's desktop.

The pre-existing menu CSV round-trip failure was reproduced without writes:
`menu_model_csv.build_obj()` omits the live `edit.preferences` submenu, and its
column/entry schema cannot retain `requires_project`. Captured as
`dat-menu-csv-roundtrip-etp`; blindly following the checker's regeneration advice
would remove current Preferences semantics. The owning menu/Preferences lane
must reconcile that separately. Neither CSV/JSON nor generator was edited here.

### First-unit verification

Guarded, offline/locked tests for the application, protocol and renderer passed
558 library/binary tests: 294 application, 112 protocol and 152 renderer;
eight existing application tests were ignored, not passed. The final rerun used
`cargo test --offline --locked -q -p datum-gui-protocol -p datum-gui-render
-p datum-gui-app --lib --bins` through the guarded proof runner. The
focused runs also passed 11 menu protocol tests and 18 application/renderer menu
tests. Coverage includes real menu inventory mapping, exact pilot handler/reason
identities, unresolved/hidden/stale camera contexts, inspectable unavailable rows,
rendered enabled/disabled state and existing Preferences submenu navigation.
These are unit/construction tests, not native user-input proof.

Delivery tests passed 82; existing project-status tests passed 45, claim tests
five and source-health regressions 13. Source health, evidence traceability
(20 routes / 114 artifacts), spec governance (203 classified), progress coverage,
project-state/projection, menu-model validity and dependency/Cargo-resource policy
checks passed. No authority digest was refreshed.

Full drift was attempted: delivery and Cargo-resource checks plus workspace
rustfmt passed, then workspace Clippy stopped with 33 Preferences-owned engine
diagnostics (large error/enum payloads and repository open options). This is
ongoing other-lane work, not a completed Preferences verdict or a WDQ repair
authorization. The separate CSV round-trip failure above is also unresolved.
No full-repository green claim is made.

### Offscreen render inspection, not native proof

The guarded command `cargo build --offline --locked -p datum-gui-app
-p datum-eda-cli --features datum-gui-app/visual` passed. Diagnostic binary hashes:

- GUI: `ecec45fea1746c89b48c0849eb5d1154d9f5f91c52b609c0f94aafdb5223484d`.
- CLI: `32b79c2f0addf2cefd6d8eb1f979c50bfb2b469d8e6bd0df21568f3a014900be`.

The existing C01 archive was extracted under the isolated temporary directory
`/tmp/datum-wdq-g04-preview-ptukH1/project`. Captures used that Project, pinned
`EDA_CLI_BIN` to the just-built CLI, private XDG config/runtime directories,
unset DISPLAY/WAYLAND_DISPLAY, and `--visual-test --exit-after-screenshot
--window-size 1280x768`. Help used `--open-menu Help`; the unresolved schematic
capture used `--open-menu View --focus-pane schematic`. No native input was sent.

Both [Help preview](g04-help-preview.png) and
[unresolved schematic View preview](g04-schematic-preview.png) were inspected.
About and the ineligible Fit row are muted, focus outlines remain visible and
the existing menu/pane geometry is preserved. The displayed fixture revision
is not a build identity. `tar --compare` confirmed all archived source/journal
bytes unchanged after both captures; this is not a complete new-path or native
reopen audit. Concurrent Preferences inputs were not frozen into a full proof
manifest; these previews must never substitute for fresh PILOT-S01–S05 evidence.
