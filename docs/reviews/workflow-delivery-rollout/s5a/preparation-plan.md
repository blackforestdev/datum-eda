# S5A selection execution and native-proof packet

Status: S5A-C01 reconciled planning packet; S5A-C02 owner disposition pending.
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
The old installed-gate obstacle is historical: PM044 retired WDQ blocking
enforcement on 2026-09-17. No enrollment or local-trust promotion is a current
prerequisite. The present owner assignment authorizes planning only and excludes
fixture construction, candidates and native experiments in this turn. Historical
bounded preparation approval is preserved, not treated as full S5A execution.

The original proposal alone did not complete S5A-C01. The reconciled packet
below now accounts for complete domain scope, actual existing fixture inputs,
production integration, numerical provenance and review arrangements. Future
fixture/handler outputs remain unbuilt; D1–D5 require explicit C02 disposition.
The six historical readiness questions are mapped, not represented as proof.

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

Historical WDQ checker observation (not a current gate under PM044):
ordinary readiness did not require completed native Proof in that checker. It does require actual references and resolved decisions. Activation
and verification require correlated native results, and changes to enrolled
input roots can force activation checks even while a step label says pending.
That installed WDQ requirement is retired by PM044. Ordinary claim, source
ownership, governance, independent proof and C02 authorization remain required;
no bypass, enrollment repair or fake early proof is proposed.

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
The former installed preparation/landing obstacle is retired. Measurement
provenance and reviewer provision remain for the explicit C02 disposition below.
No native readiness, implementation or acceptance is asserted.

## Reconciled S5A-C01 execution packet — 2026-09-30

<!-- EVIDENCE:UVT-S5A-BUILD:S5A-C01-PACKET -->

Planning baseline: clean `1fd5193b7a5bbba65b351515fc241db2ee8896f6`.
Fresh next/details selected S5A-C01, planning, unassigned before this session's
synchronized Frontier/beads claim. This packet completes contract reconciliation
for presentation at C02, not fixture construction, native proof, implementation,
review reservation or acceptance. The seven fixture families above remain the
construction specification. Case aliases below are not persisted UUIDs.

Authority reviewed: complete `gui-selection`, `prototype-selection`,
`foundation-consumer-completion` and `workflow-delivery-cohort-preparation`
sources/consumers; PM023/026, UVT including the fourteen resolved choices,
Rendering Book, conformance §8, reference README and selection-study PNG;
foundation manual-workflow contract; PM044 retirement, PM051, PM052, resize repair
and S4 closure. Selection Study DOM/CSS/SVG was inspected, not reconstructed from
its screenshot. The historic research/guidance's pending-ratification language
is superseded by PM026 and UVT §2.2.21–22, not a new unresolved visual choice.
The other two cohort contracts are unchanged and acquire no S5A authority.

### Requirement, existing implementation and single owner

Paths below are relative to `crates/`; proposed normal modules are explicitly
marked new. Editors configure classes, ownership collapse, layer eligibility and
quantity context. They do not own competing reducers, identity resolvers or
aggregate computations. `gui-protocol` owns transport/data types and pure output
projection; `gui-viewport` owns interaction/lifetime mechanisms. Engine resolution
supplies authoritative IDs, bindings and connectivity. Protocol never depends on
viewport; renderer owns GPU emission, app owns native-event wiring.

| Requirement and authority | Existing baseline / demonstrated gap | Required owner and affected consumers |
| --- | --- | --- |
| Nine subjects, UVT 2.2.20 / PM026 | `gui-protocol/src/lib.rs::SelectionTarget` has None/AuthoredObject(String)/ReviewAction/CheckFinding; no class, Compound or derived scope | New `gui-protocol/src/selection_subject.rs` owns typed identity vocabulary; `lib.rs` session/state consume it. Board, schematic, Inspector, Console, terminal/AI all migrate together. |
| Acquisition/focus, 2.2.1–2/10 | `ReviewWorkspaceState::select_authored_object` scans board arrays; `runtime_primary_pointer.rs` routes same-click pane focus; `runtime_camera_pane.rs::resolve_schematic_primary_click` traces then returns false | New `gui-viewport/src/selection.rs` owns one reducer and gesture grammar; `profile.rs` configures surfaces. `gui-app/src/runtime_primary_pointer.rs`, `app_native_events.rs`, `keyboard_focus.rs`, `workspace_keyboard.rs` adapt native events. |
| Region qualification/evaluation, 2.2.4/17 | `gui-viewport/src/hit.rs::SpatialHitIndex` is an existing AABB point-query owner; no region module exported in `lib.rs` | Extend that index with resumable region enumeration; new `gui-viewport/src/region.rs` owns exact qualification. Renderer supplies authored anchors/oriented rectangles/islands, not expanded hit silhouettes. |
| Revision/artifact/pane lifecycle, 2.2.18 | Singleton session changes exist; complete project-scoped typed re-resolution/drop reports absent | New `gui-viewport/src/selection_lifetime.rs` consumes resolved revision/artifact observations; app source refresh and pane lifecycle call it. Protocol stores subject/revision and reported losses, never journal history. |
| Run/Net/Bus membership and eight mapping classes, 2.2.20 | Schematic hit metadata exists but conflates port with Label; no typed derived selection | New `gui-protocol/src/selection_resolution.rs` translates engine-resolved membership into read-only identity/projection metadata. Existing `schematic_scene_import/{mod,symbols,buses,labels,drawings}.rs` and native scene construction preserve authored and parent IDs. Never infer connectivity from ID prefixes, names or pixel coincidence. |
| Common/Mixed/Unavailable/scopes, 2.2.19 | `side_panels/render_inspector.rs` composes singleton views; envelope is `{kind,id}` | New `gui-protocol/src/selection_projection.rs` owns one full typed output; `context_envelope.rs` serializes it. `gui-render/src/side_panels/{render_inspector,inspector_dispatch,inspector_chrome}.rs`, app Console and `terminal_session_context.rs` consume it. Existing engine Units service supplies quantity semantics; no new parser. |
| Immediate selection/related projection, 4.4 / RB 2 | `render/retained.rs` branches on selection for track/pad/via material; `retained_scene_history.rs::retained_selection_cache_key` retains object-specific keys; `overlay.rs` has partial text/graphic cues; `selection_relation.rs` is singleton alias logic | New normal `gui-render/src/render/selection_overlay.rs` owns shared cue emission/LOD. Retained geometry becomes selection-independent; `scene_projection.rs`, `overlay.rs`, `scene_visibility.rs`, history/source-dependency logic consume typed projection. Preserve shared RenderSession/submission/damage/resource owners. |
| Read-only boundary, 2.2.14 | Existing workspace shortcuts can set Move/Delete and canvas can hand authoring to Terminal | App selection entry must not reach those authoring dispatches. Inspector and selection menu seams stay visible disabled with reason; no transform handles. Existing separate authoring lane gains no authority. |
| Programmatic accessibility, focus and non-color | Keyboard focus router and native terminal ownership exist; complete selection value/count/focus accessibility proof absent | App publishes shared projection semantics through its accessible UI boundary; renderer supplies shape/pattern/cue states. Missing native accessibility exposure is a required integration gap, not screenshot-equivalent proof. |

PM022 applies before touching `gui-protocol/src/lib.rs`, `gui-app/src/main.rs`
or renderer include-root debt: extract cohesive session-selection/state ownership
into normal modules, reduce touched legacy owners and ratchet exact ceilings.
Do not add `include!` continuations. New paths in this table are bounded proposed
ownership, not claimed source or permission to edit them before C02. Engine
read-only metadata gaps must use existing resolver/connectivity authority; a
missing engine identity is escalated rather than fabricated in scene code.

### Complete class and granularity coverage

All rows inherit exact stable-ID oracles, selection-independent hit geometry,
visibility/filter rules, full resolving-pane projection and read-only outputs.
The complete UVT 2.2.16 matrix remains authority; this table accounts for every
row without replacing its predicates.

| Class / workspace | Native acquisition and qualification to prove | Present availability and unit |
| --- | --- | --- |
| Board footprint | Owned pad/graphic/text hit collapses to parent; pad strict majority, padless placement anchor; component Edge.Cuts cannot capture board-outline clicks | Board singleton/hits exist; complete ownership metadata/reducer in U1/U2 |
| Track section/run | Section endpoints both inside; curve >=2/3 authored anchors; click section, double Run, triple Net | Section live; derived scopes missing, U1/U2 |
| Via | Center anchor; full conductive ladder; material/drill retained | Primitive live; scopes missing, U1/U2/U4 |
| Zone | Complete authored filled area/all islands, direct or Select-menu; conductive ladder; outline/fill/thermals one identity | Primitive exists; topology/qualification/projection review required, U1/U2/U4 |
| Board text | >50% rotated layout rectangle, exactly 50% fails; glyph-shaped cue | Live singleton; current box cue superseded, U2/U4 |
| Filled board graphic | Whole authored area enclosed; direct interior/menu acquisition | Interior/typed filled identity gap, U1/U2 |
| Board line/arc/outline | Authored endpoint/midpoint rule, independent board authority | Stroke hit exists; exact topology metadata U1/U2 |
| PCB Global Net | One semantic ID, current complete electrical membership incl. ratsnest; parent bodies only related | Subject absent; U1/U4 |
| Airwire | Never independent click/region/Ctrl+A subject; only Net projection | Derived rendering exists; U1/U4 negative case |
| Board dimension | Ratified exclusion, not selected or represented as a fabricated graphic | Deferred `dat-dimension-selection-reentry-kxk`; no S5A implementation |
| Schematic symbol | Pin/owned graphic/text collapses to symbol; pin strict majority, pinless anchor | Typed hits, dispatch false; U1/U2 |
| Wire section/run | Same path rules, independent sections; continuous run distinct from disconnected same-net occurrence | Typed hit; scopes absent, U1/U2 |
| Schematic Global Net | Same semantic ID/revision as board; full wires/labels/ports/pin terminals/junctions, bodies related | Subject absent, U1/U4 |
| Bus section/run/Bus | Three kinds; entries never originate or enlarge region test; spine/name/entries together; scalar members baseline; no board Bus cue | Spine typed, entry/scope gaps, U1/U2/U4 |
| Label/port | Distinct typed classes; oriented-layout majority; no ladder origin; explicit Select Net | Port currently Label-typed; U1/U2 |
| Junction/no-connect | Connection anchor; dot or complete X; junction is Net member, no-connect object-only | Typed hits, selection unwired; U1/U2/U4 |
| Schematic text/drawing | Text layout predicate/glyph cue; strokes path rule, fills complete area; independent drawings | Text/drawing hit-kind gaps, U1/U2/U4 |
| Hierarchical sheet body | Inert selection; double-click reserved for descend; navigation not implemented by S5A | Ratified exclusion `dat-sheet-interaction-reentry-9ee` |
| Footprint Editor pad/owned text/graphics | Pad anchor incl. number; all other owned text independent; path/fill rules; definitions merely related to instances | Native editor unavailable. Typed profile/oracle tests possible; native rows unverified, C02 disposition D2 |
| Symbol Editor pin/owned text/graphics | Pin connection anchor incl. stub/terminal/name/number; other text independent; body fill independent | Native editor unavailable; same D2, never claim placed-pin proof substitutes |
| Proposal | Explicit whole action from overlay/lane, not primitive; region/Ctrl+A exclude | Route-action path exists; distinct Proposal type/lifetime U1/U2 |
| Review | Same action ID with Review kind, lane/evidence surface; evidence child never independent | Evidence rendering exists; kind/acquisition U1/U2 |
| Diagnostic | Fingerprint from marker/checks/lane; target merely related; matching new fingerprint survives | Type/Inspector exists; marker pointer gap U1/U2/U4 |

### Discrepancy dispositions and protected behavior

`dat-selection-envelope-run-flr` is **already closed**, specification-only in
`a30cbcfe`. UVT 2.2.19 now maps None/Object/Compound/Run/Global Net/Bus/Proposal/
Review/Diagnostic to `none/authored_object/compound/run/global_net/bus/proposal/
review/check_finding`. Preserve Run typed origin and revision above 256 members;
Proposal and Review remain different even with equal action IDs. The current
`review_action` runtime tag is the superseded predecessor, not a tenth subject.
U1/U3 must migrate the actual producers/consumers together and reject unknown
kinds without guessing. Do not reopen the closed specification bug or claim its
closure proves runtime round-trip.

PM026 parent/child and merely-related mappings stand. Same-identity equality
requires subject ID and revision, not equal reference text or library binding.
A Run re-derives wholly through its surviving origin; a deleted origin dissolves
it, whereas an enumerated compound drops only missing members and clears lost
focus without promotion. No create/delete/undo substitution or resurrection.
No new Group, universal lock, transform, field patch, private selection undo,
writer or persistent selection authority is proposed.

Observed retained-selection dependency is established by source branches and
cache keys, not a newly measured timing defect. Cause: singleton presentation
is an input to retained authored construction. Remedy: remove that presentation
dependency and re-compose selection in one post-world path; recoloring two editor
buffers would violate PM023/026. Existing hover/camera mechanisms need no
replacement. No demonstrated new resize defect exists: no resize repair proposed.

Preserve PM049 shared session ownership and PM052 default workspace retained
resize reuse from `a89f94b2`, closure `1fd5193b`: safe exact-allocation fallback,
scale/output reset, temporary resampling only during resize, exact settled output,
explicit diagnostic overrides and existing compatibility option. Selection must
not change allocation policy, render quality, painter order or submission lifetime.
S4 closure remains bounded implementation/correctness closure, with Q01–Q07 and
performance S5 qualification separate; functionality C04 is not that campaign.

### Native proof matrix and observation boundary

Use normal native Project board/schematic panes, not an imported-board-only
shortcut. Every case binds actual persisted IDs/revisions, expected subject/focus,
complete membership and typed values, pane IDs/cameras and input sequence before
entry. Observe production reducer/output transitions correlated with native input,
not a test-only alternate dispatch. Existing registry exports only four pilot
keys; `pending.S5A-*` labels in the archived contract are not real dispatch IDs.
Actual handler references are frozen from the implemented typed entry before C04.

All cases record source-shard hashes, model revision, journal tip/count and
operation-dispatch observation before/after; expected selection-only delta is
zero. Capture whole membership or an independently computed exact-set oracle,
never only the 256-ID context list. Fixture setup revisions and their journal
transactions are explicitly separate. Screenshots supplement identity/value,
accessible-state and non-mutation evidence. Reopen resolves authoritative source.

| Case aliases / inherited group | Inputs and positive/negative oracle | Proof purpose / ledger |
| --- | --- | --- |
| N01/S01 | Both surfaces, every non-deferred class above: click replace, Shift add/idempotent, Ctrl remove/no-op, empty click preserve, same-click focus+select; below/at 4-device-px threshold, Shift+Ctrl is no third operation | Establish reducer/ownership and enabled native entry, B1–3/I1 |
| N02/S01 | Rightward rectangle/leftward lasso, locked shape; additive/subtractive; 1px clipped dashed boundary, reduced motion stops animation; Space before press pans, Space after activation cannot steal | Establish gesture grammar without authored drag/move, A1/A4/B1 |
| N03/S01/S06 | SOIC eight anchors 4 fail/5 pass; symbol fourteen 7 fail/8 pass; pinless/padless; straight 1/2 fail, 2/2 pass, curve 2/3; rotated text below/exactly/above half; all-island versus missing-island fill; even-odd self-intersecting lasso, zero area | Independent per-class oracle, A3/A6/V4 |
| N04/S01/S06 | Edge/corner auto-pan in 24px band, stop/diagonal; equal final world regions by distinct pan/zoom paths; small AABB in 100k design; release under pressure evaluates across frames, no partial commit | Prove exactness and pruning, A1–5; no camera/preview-dependent membership |
| N05/S01/S03 | Hidden/class filters reject new canvas/menu/electrical acquisition; dim visible and locked remain eligible; hide/show committed member preserves identity; Ctrl+A includes hidden/filtered authored geometry, parent collapse, excludes all non-authored/airwires | Prove eligibility and honest global counts, A7/O4 |
| N06/S01/S03 | Deterministic overlapping candidates, Select menu labels/order, preview and Select All; dismiss outside/Escape preserves prior; every seam disabled with reason | Manual ambiguity resolution, B1–3; full menu build remains separate |
| N07/S02/S03 | Revision survivor updates; deleted member drop/report, delete+recreate new ID not selected; undo old ID not resurrected; loss of focus leaves none; stale restored/context IDs drop/report | Lifetime identity, L1–4/L6/L8; source setup deltas distinct |
| N08/S02/S03 | Close pane, swap content/sheet, replace Project, missing scene; committed set survives pane loss, gesture cancels, Project replacement clears; duplicate panes have independent cameras, full equal cues only where resolvable | Scope/lifecycle/partial projection, L7/L9/I2/I3 |
| N09/S01/S02 | Conductive origin section/via/zone ladder; pads/labels explicit Select Net; disconnected same-net run and scalar/bus distinctions; revise membership and remove origin/semantic ID | Deterministic authoritative derived sets, L5/I4; no label/pixel connectivity |
| N10/S02/S04 | Proposal/Review equal action ID but distinct kinds; explicit Diagnostic fingerprint; commit/discard/check invalidation dissolves/report, equal new fingerprint survives; region/Ctrl+A exclude; no mixed non-authored compound | Artifact lifecycle and nine-kind parity, A7/I1/O5 |
| N11/S04 | Singleton then homogeneous/mixed compounds; All N/per-type switch preserves membership/focus; Common vs Mixed exact canonical quantities 5080000/5080001nm despite equal rounded display; absent/incompatible field Unavailable with reason | Full typed output oracle and scope, O1–4/O6–7 |
| N12/S04/S05 | 256 and 257 enumerated AND derived members; origin/semantic identity/focus retained, list omitted only above cap; Inspector/Console/terminal-AI round-trip agree on revision/kind/totals/reasons | Bounded transport without membership loss, O5/I1 |
| N13/S05 | Attempt numeric entry/Move/rotate/mirror/lock/Group from pointer, keyboard, menu, Console and context; plain drag inert; cancel evaluating/gesture via Escape/focus/capture/window loss/content/pane close; then idle Escape clears | Zero Operations/shard/journal delta, B1–3/O7; no implicit repair |
| N14/S03/S05 | Two preconfigured Project quantity contexts, source reopen and stale consumer restore; view/scope changes do not rescale geometry or write either Project/Global; no selection history entries | Foundation eight-concern isolation, L8/O6; no new Preferences UI |
| N15/S06 | Whole-owned symbol/footprint and glyph/point silhouettes, hidden no cue, locked grey+approved padlock, selection beats hover, optional focus no extra persistent cue; all eight same/related mappings, channel collisions | V1–5/I2–3, native goldens + HUMAN panels 1–7/9; verify static bytes/uploads/CAM, not appearance alone |
| N16/S06 | 100k authored objects with separate primitive count, sub-2px cues, exactly 65,536 versus overflow; whole-pane exact union, no omitted members, maximal proposal/finding collision; repeated warm select/deselect/cancel/reopen | D1–2, bounded capacity/work and preserved channels; HUMAN panels 8/9 |
| N17/S01–06 | Keyboard-only Ctrl+A/Escape/menu and Inspector scope/inventory; terminal/text owns keys; focused versus pointer pane; accessible names, selected/focus/count/Mixed/Unavailable/hidden/locked/dropped states; grayscale/CVD/high contrast/reduced motion | Programmatic and non-color parity; unavailable native accessibility route blocks this row |

Every S01–S06 dimension remains required: normal/invalid/cancel/scope/precision/
undo-redo/save-reopen/accessibility/failure-recovery are exercised by the cases
above and the existing nine-dimension rows, not erased by a happy-path screenshot.
Undo-redo means external authorized fixture history plus zero selection history;
no S5A edit engine is built to test it. Definition-editor native rows remain
unavailable; board/schematic child collapse and pure profile tests do not close
them. Full hierarchical-sheet selection and dimensions retain only their already
ratified exclusions. Missing proposal/diagnostic marker or semantic connectivity
metadata is implementation work inside the selected scope, not accepted absence.

### Numerical provenance and missing proof

| Figure / requirement | Provenance, applicability and current authority |
| --- | --- |
| 65,536 detailed primitives; sub-2px cues; 100k fixture | UVT 2.2.13, OPEN-14, RB 2.8, final owner approval and PM026. Binding deterministic emission/fallback/corpus requirements for every pane and cue channel. Not observed latency or evidence that 100k is currently feasible. Required tests D1/D2/A2/A3 remain absent. |
| 256 context IDs | UVT 2.2.19, `a30cbcfe`; binding serialization bound, not selection membership or performance acceptance. |
| 4096 point-query candidates | Existing `hit.rs` constant and S3 reference in UVT 2.2.17. Point-query work bound only; no provenance authorizes copying it as a region completion bound. Region chunk constant/preview tuning must be explicit and tested for exactness/cancel; no latency inferred. |
| 4px activation/24px edge/1px marquee/2px crisp cue | Ratified interaction/visual geometry, not timing budgets. Apply in physical device pixels across live camera/scale, retaining PM052 transient resize exception. |
| Fixture-specific S5A latency | Foundation plan asks for controlling budgets; preparation/S06 defer actual numbers. No derived S5A numerical latency criterion or populated selection timing baseline found in these sources. Missing provenance D1, not silently satisfied or substituted with performance S5 GPU figures. |

PM051 withdraws GPU 4/8ms blocking gates only. Adjacent CPU/duty/resource
thresholds keep their authority; no cap is changed by this plan. No observed
S5A timing baseline or aspirational numeric target is claimed. PM052 resize
percentages and the accepted limited pointer assessment are scoped observations,
not selection responsiveness baselines. Review affected local resource admission
with existing owners; do not open DRM qualification or target chasing.

At C04, proposed bounded functionality timing is one cold and one warm replay
per board/schematic dense case on one pinned reference configuration, plus one
repeat to reveal variability, with exact N/method/input/output and per-trial
results; setup failure or correctness loss stops the affected batch for
assessment. This is a proposed collection boundary subject to C02, not a budget
or campaign grant. Do not invent percentile confidence from that small sample.
Record input-to-committed-subject, evaluation frame work, overlay preparation and
observed presentation separately where observable; unavailable display timing
stays unmeasured. Instrumented/uninstrumented observer effects must be identified;
if no adequate comparison exists, label timing diagnostic and non-qualifying.
No optimization rerun follows an exceedance without demonstrated cause and scope.

Reuse: S0–S4 camera/hit/hover/focus tests as substrate evidence; selection-study
approved nine-panel reference and padlock contact-sheet design; closed envelope
spec repair; S4 closure and existing painter/resource/session proofs; PM052
13 focused resize tests, release/Clippy and no-option launch/captures recorded in
resize-repair.md. None needs rerunning merely for this session. Retained selected
color tests are migration-debt evidence; they must be superseded by V1/V2 byte-
stability proof, not blessed as the target. New S5A evidence covers changed
selection paths only. Source/fixture/binary changes invalidate affected evidence,
not every historical workload.

### Finite delivery units, review and C02 dispositions

These are serial units within C03, not additional canonical completion steps:

1. **U1 typed authority and metadata.** Extract selection state/session ownership;
   add nine kinds, engine-resolved read-only membership/parent/qualification data,
   revision/artifact lifetime. Dependency direction and resolver IDs first.
2. **U2 shared acquisition.** One reducer, region index/evaluation/cancel, click
   ladder, filters and native board/schematic input. Read-only Select-menu leaves
   integrate with existing menu owner; no wholesale S7/menu or cross-probe build.
3. **U3 one inspection projection.** Typed aggregates/values/scopes/blockers,
   full inventory and capped context; migrate Inspector/Console/terminal-AI and
   accessible semantics together. Never parse display strings back into data.
4. **U4 shared immediate projection.** Remove retained selected styling/key
   dependencies, add silhouette lift/glow/cue and one dense LOD, lock asset and
   channel composition. Preserve PM049/052 and unaffected authored render output.
5. **U5 exact fixture binding and native proof readiness.** Construct the seven
   declared families through Datum-owned native facilities; a small proposed
   `crates/test-harness/src/bin/s5a_fixture.rs` may orchestrate existing native
   write APIs if needed (no complete shared builder was found in harness lib).
   Freeze manifest/oracles under `crates/test-harness/testdata/selection/s5a_v1/`,
   candidate/source/binary/environment/input bindings and actual production
   observation references before C04. No fixture authored in this planning turn.

Fixture base inspected: native_authored_baseline_v1 Project UUID
`10000000-0000-4000-8000-000000000001`, Board `...0003`, Sheet `...0004`;
all board/sheet object maps are empty. Part `90000000-0000-4000-8000-000000000001`
and PinPadMap `a0000000-0000-4000-8000-000000000001` are actual pool inputs,
not placed subjects. Preserve original fixture; construction creates separate
revision variants and records real IDs, bindings, source hashes and rebuild
recipe. Missing native arc/fill/semantic-bus authority cannot be solved by a
colored synthetic rectangle or ad-hoc production writer. Stop that case and
return its exact authority gap. Input closure includes actual source/modules,
Cargo lock/toolchain, bundled fonts/icons, source fixtures/producer recipe,
reference hashes, launch/backend/scale/viewport and instrument identity.

Verification batches earn specific decisions: U1 lifecycle/vocabulary tests
allow U2 to consume stable authority; U2 independent region oracle/pruning/cancel
allows native acquisition; U3 output/quantity/cap tests establish consumer parity;
U4 byte/CAM/cue/collision/LOD tests establish lawful presentation; C04 native
N01–N17 establishes real invocation and non-mutation; C05 independent replay
establishes evidence reproducibility; C06 owner accepts only exact reviewed
candidate. Use the conformance §8 named assertions/test homes, reconciling any
relocation in that owning lane when implemented. No speculative harness platform
or general test ledger is proposed. Required affected governance/contract gates
run per commit; compiling proof uses run_cargo_guarded.py, serially.

**D1 — missing latency provenance.** Owner must disposition whether execution
proceeds with deterministic work/correctness/resource criteria and the bounded
missing-only timing observations above while latency stays undecided. This is
an explicit specification disposition, not a silently waived controlling budget.
Any new binding time threshold needs PM051 derivation/evidence/rationale and
owner ratification before enforcement. No benchmark collection now.

**D2 — unavailable definition-editor native rows.** Exact board/schematic scope
is executable without building new editors; shared definition profile predicates
remain testable, but native Footprint/Symbol Editor proof cannot currently run.
Owner must record treatment of these outstanding rows before claiming complete
acceptance. A scope deferral changing PM026 obligations requires its numbered
amendment; this packet grants no automatic waiver or new editor execution.

**D3 — conformance refusal ledger boundary.** Intake
`dat-s5a-refusal-ledger-8fhn` (related to the build; no execution claim). Conformance §8 names engine
batch-guard R1–R4 as landing with S5A, whereas PM026/UVT 2.2.14 explicitly
reserve mutation/batch guard for S5B. Higher doctrine controls: implement no
mutation guard or operation path in S5A. S5A proves disabled seams, zero dispatch
and exact blocker disclosure; R1–R4 engine mutation proof remains future work.
Obtain explicit disposition and owning conformance-route reconciliation before
marking that ledger complete; no R assertion is labeled passed here.

**D4 — derived-subject modifier transition.** Intake
`dat-selection-derived-modifier-caa8` (related; owner specification question). The reviewed contract defines
add/remove authored members and distinguishes enumerated Compound from derived
Run/Net/Bus, but does not explicitly say whether modified acquisition while a
derived subject is active converts it into an enumerated compound or replaces/
refuses it. No new runtime reproduction is needed to establish this missing
transition rule. Owner must select that behavior through the selection route
(and amend PM026 if its identity law changes); do not implement implicit
conversion from displayed/capped member lists. Normal authored-member grammar
and explicit semantic acquisition remain ratified.

**D5 — independent reviewer and bounded execution.** No reviewer is reserved
or availability asserted. C02 must name an available reviewer/session independent
of the implementer, replay arrangement and scope; Claude owns any prototype
reconciliation. Reviewer validates production/native inputs, full ID/value oracles,
source/journal non-mutation, unavailable rows, numerical provenance and all
negative cases against the exact candidate. Findings receive fixed/not-reproduced/
expected/insufficient-evidence/no-change-needed dispositions with evidence;
mandatory unresolved failures block C06. No additional agent is spawned here.

Claude reconciliation list (not sent or edited by this session):
`docs/gui/prototypes/selection-study.html`, title tag/statusmark near h1:
reconcile old pending-review annotation with UVT 2.2.21 approval if the visual
owner chooses to refresh it. Preserve panels 1–9, material/cue/channel/focus laws;
proof is Claude-lane review and updated fixed-size reference only if changed.
This cosmetic historical label does not reopen approved construction or block
planning. Approved lock SVG candidate remains to_author in icon_set.json; U4
may author the code-native asset against that design, not redesign the prototype.
No other HTML change is required by this packet.

**Exact C02 presentation scope:** this committed packet, complete original
fixture-family table, requirements/owners and N01–N17, U1–U5 boundaries,
PM026/PM049/PM051/PM052 preservation, D1–D5, unavailable scenario accounting and
no enrollment/trust change. The older six draft readiness questions map here:
complete routes/classes → authority/class tables; dispatch → U2/U3/U5;
fixture closure → U5; foundation/dimensions → N01–N17; mechanisms/prerequisites
→ discrepancy dispositions/D1–D4; independent review → D5. Null historical
handler/fixture/proof placeholders remain honest until real outputs exist.
C01 completion records reconciliation and this decision packet only. C02 remains
pending owner_decision; C03–C06 remain pending. The build issue stays open,
and cross-probe is neither selected nor authorized.


### Planning verification record

Non-compiling checks passed: project-state (61 Frontier items), evidence
traceability (28 routes / 124 research-prototype artifacts), specification
governance (501 classified), inventory parity (20 inventories), progress coverage,
source health (2672 source files) and diff whitespace. These establish planning
integration, not native feature proof. Staged lane/format checks run at commit.

Verification mistake retained: invoking `check_gui_conformance.py` without
first inspecting its execution boundary started guarded Cargo `console_` tests.
The parent checker/guard/Cargo were terminated (exit143) during dependency
compilation when the out-of-scope build was noticed. Conformance is **not passed**;
no native experiment or completed candidate result is claimed. Partial shared
Cargo artifacts were left in place; no shared-target sweep or gate bypass.
No runtime source, fixture, dependency or prototype was changed by this attempt.
Further compiling/native conformance belongs to authorized execution only.
