# S5A fixture and dispatch preparation proposal

Status: bounded preparation approved; installed preparation/landing path unresolved.
Frontier: UVT-S5A-BUILD / S5A-C01; `dat-uvt-s5a-build-1wv`.
Owning route: `workflow-delivery-cohort-preparation`.
Basis: `runtime-readiness-audit.json` at `7ab093c2` and
`readiness-input-assessment.json` at `933a5782`; these are observations, not
native proof. The current delivery contract supplies the inherited behavior.

## Purpose and limits

Produce actual, reviewable inputs for S5A readiness. This is not permission to
enable selection capabilities, amend PM026, build an editor, change Preferences,
modify a Claude prototype, add dependencies or install/promote delivery trust.
The owner approved this bounded preparation at commit `6ece73ed`; the exact
response and approved document hash are retained in
`preparation-owner-20260910.json`. Do not request this approval again.
The live Frontier remains planning until the installed gate's preparation path
is reconciled: it currently requires enrollment/readiness before execution and
has no promoted S5A source scope. Approval is recorded, not silently converted
into readiness, full product permission or permission to bypass local trust.

Do not mark S5A-C01 complete because this proposal exists. Its complete domain
matrix, actual fixture identities, production bindings, applicable budgets and
review arrangements still have to be reconciled. None of the six unresolved
delivery questions receives a disposition through this document.

## Existing inputs and their permitted interpretation

- The pilot archive has 11 packages, 31 pads and 32 tracks, but no schematic
  sheets/instances/definitions and no vias/zones/texts. Keep its acceptance
  historical; do not substitute it for the complete S5A corpus.
- `crates/test-harness/testdata/library/native_authored_baseline_v1` contains
  eight local pool objects in thirteen JSON files, including explicit Part and
  PinPadMap identities. It has an empty board and empty main sheet. Reuse is a
  candidate, subject to resolver, binding and visual-construction review, not
  a claim that it is already a placed selection fixture.
- The production action registry currently exports `view.fit`, `view.layers`,
  `window.documents` and `help.about`. None is a selection action. Existing
  pointer/session/Inspector functions are integration sites, not invented
  selection registry identities or proof of complete behavior.

## Proposed native fixture families

All names below are proposed case aliases, not fabricated persisted UUIDs.
Construction must record actual UUIDs, source/model revisions and expected sets
in a deterministic manifest. Use existing Datum-owned fixture/native-authoring
facilities; do not introduce a private production writer or third-party code.
Keep baseline and each revision variant distinct and hash their inputs.

| Family | Required population and exact oracle | Consumer groups |
| --- | --- | --- |
| `class-qualification` | Populate every non-deferred PCB and schematic class in UVT §2.2.16. Include pad/pin-majority parents, anchor-only fallbacks, straight/curved paths, oriented text and multi-island filled geometry. For an eight-anchor case, four inside must fail and five must qualify; straight paths require both endpoints and curves two of three anchors. Text has separate below/exactly/above-50% area cases; filled areas require every island. | S01, S03 |
| `electrical-scope` | Disconnected physical runs sharing one Net identity, conductive origins allowed by OPEN-1, and scalar/bus distinctions. Include bus section, bus run and semantic bus as different kinds; resolved net membership and merely-related parent bodies must have distinct expected sets. Cross-sheet occurrences share only their ratified identity. | S01, S02, S03, S04 |
| `compound-output` | Single objects; homogeneous and mixed-class compounds; explicit optional focus; common, mixed and unavailable typed fields; hidden/locked blockers. Include exact lengths 5080000 and 5080001 nm that can share a rounded label, plus exactly 256 and 257 members for bounded context serialization. Expected canonical values, summaries and full membership are separate from display strings. | S02, S03, S04 |
| `revision-lifetime` | Persist separately identified before/after revisions for identity-preserving edits, deletion, recreation with a new id, undo restoring an old id, loss of a Run origin, changed Net/Bus membership, and producing-artifact expiration. Expected selection follows drop/report/no-substitution/no-resurrection; fixture setup transactions are not selection effects. | S02, S05 |
| `non-authored-channels` | Actual proposal-action, evidence-surface Review and diagnostic/finding identities with their producing artifacts and lifetimes. Explicit acquisition only; region/Ctrl+A exclude them. Include overlay collisions without flattening proposal/evidence/severity identity. A synthetic colored rectangle is not equivalent evidence. | S01, S02, S03, S04, S06 |
| `dense-and-cancel` | The ratified 100k population, exact membership oracle, small-query pruning and a maximal-channel-collision variant. Exercise the 65,536 detailed-selection-primitive boundary and union-mask fallback, cancellation during evaluation and equal final regions reached by different pan paths. Record authored-object count separately from rendered primitive count. | S01, S05, S06 |
| `scope-and-reopen` | Two separately identified Projects with preconfigured quantity contexts, duplicate/mixed panes, hidden selected members and per-type Inspector scopes. Source/journal baselines establish that focus, visibility, inspection and cancel do not author changes. Project replacement clears scoped selection; reopen never treats a workspace snapshot as design authority. | S02, S03, S04, S05, S06 |

Definition-editor obligations in the UVT class matrix remain visible. The packet
must identify which cases can exercise shared typed/profile services and which
require unavailable native editor surfaces. Such cases stay unverified; neither
the read-only board/schematic delivery boundary nor the unavailable editor is a
silent waiver. Board dimensions and hierarchical sheets retain their ratified
deferrals and existing re-entry beads, not completed capability claims.

## Dispatch and observation plan

Reconcile each route against the production code before assigning a registry key:

| Entry or consequence | Inspected integration site | Preparation output |
| --- | --- | --- |
| Native pointer acquisition | `Runtime::handle_primary_click`, `Runtime::select_hit_target_inner` | Exact Board/Schematic entry routing and actual typed handler identity; distinguish a hit result from dispatched selection. |
| Schematic acquisition | `Runtime::resolve_schematic_primary_click` | Preserve the observation that it currently traces and returns false. Name the authorized implementation seam without claiming it is enabled. |
| Native keyboard/cancel | `keyboard_focus` and `workspace_keyboard` | Real shortcut/focus ownership and cancellation dispatch mapping; no generic `native-keyboard` label standing for every input. |
| Typed selection state | `SelectionTarget`, `SessionCommand`, `ReviewWorkspaceState` selection methods | Current-to-required nine-kind mapping, revision lifetime and parent/child identity boundaries; no fictional registry export. |
| Inspector/Console/context | Inspector dispatcher, selection Console echoes, `DatumSelectionContext::from_selection` | One planned typed-projection owner and all real consumer entry surfaces; capture exact identity/value/count observations, not screenshot-only agreement. |
| Projection invalidation | `retained_selection_cache_key`, `Runtime::apply_session_result` | Trace selected-state consequences and later measure retained-buffer/CAM invariance; cache-key inspection alone is not GPU proof. |

Any authorized registration must be derived from the same typed production
dispatch used by the application, not a parallel evidence-only handler table.
Do not map required compound or schematic behavior to camera-fit just because
that handler already exports correctly. A disabled/unimplemented branch remains
honestly unavailable; it cannot satisfy required normal invocation.

Ordinary readiness does not require completed native Proof in the inspected
checker. It does require actual references and resolved decisions. Activation
and verification require correlated native results, and changes to enrolled
input roots can force activation checks even while a step label says pending.
Before any preparatory source change, the owner-approved transaction must resolve
its allowed preparation/landing path; no hook bypass or fake early proof.

## Proposed file ownership and authorization boundary

Fixture destination: `crates/test-harness/testdata/selection/s5a_v1/` (new,
proposed, not created). Prefer an existing Datum fixture builder; if none can
produce the required native data, propose one small first-party builder and its
exact path before expanding the executable claim. Do not generate Rust output
under `/tmp` or create another Documents checkout/archive directory.

Potential production integration is limited to the inspected selection entry,
protocol, viewport and Inspector paths above and the existing action registry.
These are **not claimed source files** by this planning proposal. Final exact
path ownership must be reconciled before execution; changes to oversized modules
require real extraction and ceiling ratchets under PM022, not continuation files.
Claude retains all prototype ownership. Preferences, writer ownership, native
authoring, cross-probe and other product lanes remain independently controlled.

The bounded preparation disposition must say whether fixture generation and
registration/observation integration are authorized, which exact paths and
preparation/landing mechanism are permitted, and who performs the independent
readiness review. It must not implicitly authorize complete S5A execution,
native acceptance, a new dependency, a numerical performance budget or promotion.

## Verification and exit conditions

1. Validate every generated Project through the existing native resolver and
   product validation surface. Record expected-invalid variants separately.
   Inspect exact pool bindings and persisted identities, not labels alone.
2. Freeze deterministic fixture manifests with full class/population membership,
   canonical quantities, revision transitions, source hashes and rebuild recipe.
   Regeneration must reproduce the declared semantic identities and oracles.
3. Verify actual entry/dispatch/handler export correlation and honest availability
   without enabling an unapproved capability. Missing paths stay unresolved.
4. Reconcile fixture/input closure, all foundation/dimension rows and complete
   product routes in the existing delivery contract. Preserve failed attempts,
   deferred editor cases and the independent review boundary.
5. Resolve the measurement contract: hardware/backend/toolchain, viewport/device
   scale, sample count, cold/warm conditions, timing endpoints and owner-ratified
   limits. This proposal invents no latency target or feasibility result.
6. Present actual preparation outputs and independent readiness review for the
   S5A execution packet. A successful fixture validator is neither native GUI
   proof nor completion of S5A-C01's entire contract or WDQ adoption.

Fixture construction and dispatch-evidence preparation have owner approval.
The installed preparation/landing path, measurement limits and reviewer provision
remain unresolved. No readiness, enrollment or acceptance is asserted by this plan.
