# Revision Recovery Implementation Inventory

Status: RVR-A03 recovery inventory plus RVR-A04 roadmap re-entry packet;
no successor selected

Date: 2026-08-31

## Purpose and controlling boundary

This inventory applies Product Mechanics 038 to the Revision implementation
that existed at `7734d87`. It is a containment map, not a continuation of the
withdrawn Product Revision Engine program. It classifies production-reachable
Revision code as **retain**, **rework**, **quarantine**, or **remove from
production presentation** and selects one small corrective implementation
slice.

The classifications mean:

- **Retain**: a technical or generic primitive may remain in place, but this
  inventory does not authorize new product behavior around it.
- **Rework**: the behavior addresses a real need but its current product
  contract is wrong or unresolved. It cannot be accepted as-is.
- **Quarantine**: no production-acceptance claim, public promise, successor
  execution, or downstream dependency may rely on it until a separately
  bounded owner review reauthorizes it.
- **Remove from production presentation**: the behavior directly violates
  PM-038 and must be disconnected from the running product. Test fixtures may
  retain unmistakably isolated example data.

No classification below ratifies the old specification, adopts a Project
revision policy, or decides where a genuinely controlled Project browses its
records. That browsing location remains an owner decision.

## Production reachability summary

The rejected result is not one isolated renderer. Four independent paths make
Revision behavior part of the current product:

| Entry path | Current behavior | Disposition |
| --- | --- | --- |
| GUI workspace construction and rendering | Every workspace owns Revision UI state; the Project rail always renders five fictional Revision rows; row actions open hard-coded panes. | Remove from production presentation, then rework contextual discovery from real Design subjects. |
| Native Design commit | Every journaled mutation resolves Revision authority and stages/promotes a `.datum/revision/v1` integrity generation, even when no formal policy is configured. | Quarantine the mandatory coupling; retain the technical integrity concept pending separation from product authority. |
| Project resolution | Opening a Project inspects/recovers the Revision store and may add Revision diagnostics to ordinary Project resolution. | Quarantine as a mandatory open-path dependency; rework into optional technical recovery with honest unmanaged semantics. |
| Public API, CLI, daemon, and MCP | Two public verbs advertise a broad operation/query/refusal catalog and a generic query endpoint over quarantined authority families. | Quarantine from public product claims; separately decide whether a narrow read-only diagnostic survives. |

The implementation contains roughly 12,857 lines in the explicitly named
Revision engine and GUI modules alone. A wholesale revert would mix reusable
integrity and pane primitives with rejected product behavior. Recovery must be
sequenced by the boundaries below.

## GUI inventory

### Remove from production presentation in RVR-I01

| Path | Current responsibility | Required outcome |
| --- | --- | --- |
| `crates/gui-render/src/side_panels.rs` | Unconditionally calls `revision_workspace::render_navigator` in the Project rail. | Stop rendering any Revision group or row in the ordinary Project Navigator. |
| `crates/gui-render/src/revision_workspace/navigator.rs` | Hard-codes `Changes 1 draft`, a baseline, `RLS-0007`, one controlled document, and `Evidence 12 records`, then emits selection and context-menu hit regions. | No production call path may render these examples or expose their hit regions. Keep examples only in an explicitly isolated fixture if still needed for tests. |
| `crates/gui-render/src/revision_workspace.rs` | Hard-codes Change, impact, release, evidence, configuration, reproduction, and witness identities and contents into pane rendering. | No running-project path may render these records. Preserve fixture rendering only if it cannot be reached by normal application state. |
| `crates/gui-render/src/side_panels/inspector_dispatch.rs` | Gives Revision evidence and Navigator selection priority in the Inspector. | Disconnect fictional Revision selection/inspection from ordinary workspaces. |
| `crates/gui-render/src/menu_chrome.rs` | Always offers the Revision Navigator context-menu renderer when state requests it. | Remove the obsolete Navigator-menu production route. Do not invent the replacement Design-subject menu in this slice. |
| `crates/gui-app/src/runtime_revision_workspace.rs` | Owns Revision row selection, double-click recognition, context menu actions, opening/closing panes, issuance-arm UI state, and tests for that contract. | Remove Navigator-driven production interaction and its fictional pane activation. Retain only generic pane-close mechanics if shared; do not claim contextual Revision workflow exists. |
| `crates/gui-app/src/main.rs` | Stores the last Revision row click and routes Revision hit targets. | Remove obsolete per-row interaction state and production hit dispatch where no real-data Revision surface exists. |
| `crates/gui-app/src/runtime_primary_pointer.rs` | Special-cases Revision context-menu hit handling. | Remove the obsolete Navigator case without disturbing generic pointer latency or menu behavior. |
| `crates/gui-app/src/app_bootstrap.rs` | Test/visual bootstrap can open hard-coded Revision surfaces. | Fence behind explicit fixture-only construction; never select it for ordinary startup. |
| `crates/gui-protocol/src/revision_workspace.rs` | Defines the five “permanent Revision Navigator” entries, fictional surface mapping, context actions, and workspace state. | Remove permanent-Navigator semantics from production state. Fixture-only types may remain temporarily if clearly named and unreachable. |
| `crates/gui-protocol/src/workspace_layout.rs` | Every `WorkspaceUiState` embeds Revision state and `PaneContent` accepts Revision panes. | Rework so ordinary workspace state does not imply Revision presence. Pane content may remain only for an explicit real-data or fixture construction path. |
| `crates/gui-render/src/render/scene.rs`, `render/pane_chrome.rs`, and `render/types.rs` | Render Revision pane content/chrome and define Revision-specific hit targets. | Disconnect fictional production construction. Preserve generic clipping and close behavior. |
| `crates/gui-render/src/visual_runner.rs` | Provides explicit Revision golden scenarios. | Keep only as named visual fixtures; labels and invocation must make clear that they are examples, not Project data. |

The related Revision GUI tests and goldens are not product authority. Tests in
`crates/gui-render/src/revision_pane_tests.rs`,
`crates/gui-render/tests/visual_goldens.rs`, GUI protocol layout tests,
`crates/gui-app/src/terminal_regression_boundary_tests.rs`, and the
`runtime_revision_workspace.rs` unit tests must be rewritten or retired when
they assert the withdrawn permanent-Navigator contract. Generic clipping,
minimum pane size, scrolling, single selection, keyboard activation, and
terminal isolation tests remain valid when decoupled from Revision.

### Retain as generic GUI mechanics

- `crates/gui-protocol/src/workspace_layout/revision_tiling.rs` contains useful
  split-tree, minimum-width, close, and sibling-pane behavior. Rename or
  generalize Revision-specific helpers when touched; do not remove generic pane
  safety merely because its first consumer was rejected.
- Existing scissor, hit-test, scrolling, focus, and pane-chrome mechanisms in
  GUI render/app modules remain normal shell infrastructure. RVR-I01 must prove
  Board and Schematic panes remain readable and unobstructed.

### Rework after owner input

The superseding visual contract shows contextual verbs on a real Design
subject. The current implementation attaches those verbs to fictional Revision
Navigator rows. Moving them is not mechanical: availability depends on a real
selected subject, a real baseline or adopted policy, and honest unavailable
states. RVR-I01 therefore removes the false doorway but does not implement a
new one. Controlled-project record browsing and full contextual behavior require
a separate small owner decision and authorization.

## Engine inventory

### Retain: technical substrate, without product-authority claims

| Exact paths | Retained concept | Boundary |
| --- | --- | --- |
| `revision/canonical.rs`, technical portions of `revision/model.rs` and `revision/store.rs` | Canonical bytes/digests, append-only transaction generations, exact shard postimages, crash-safe staging, integrity heads, and diagnostics. | The name/location and mandatory use are not ratified. Retention does not mean every unmanaged commit must write a Revision authority store. |
| Technical portions of `revision/backup.rs`, `revision/resolver.rs`, `revision/authority_store.rs`, and `revision/authority_validation.rs` | Exact-byte verification, recovery, last-complete generation, and read-only diagnosis. | Must be separated from broad product authority and from mandatory Project-open behavior before wider acceptance. |
| Typed identity/envelope foundations in `revision/authority.rs` | Non-interchangeable IDs, versioned envelopes, opaque preservation, structural validation. | Record-family breadth remains quarantined; these types do not establish a visible workflow. |
| `revision/transaction.rs` transaction-tip and deterministic query/mutation primitives where independently useful | Deterministic local state transition concepts. | Existing Revision mutation/query inventories are not public authorization. |

### Rework: minimum optional Revision domain

| Exact paths | Why rework is required |
| --- | --- |
| `revision/policy.rs`, `revision/scheme.rs`, `revision/reservation.rs`, and the relevant portions of `revision/authority.rs` | A deliberate Project adoption model, human revision identity, and explicit baseline concepts remain legitimate, but the current policy breadth and factory profile were designed for the withdrawn program. Unmanaged must be absence, not a stored or presented formal workflow. |
| `revision/design_commit.rs` and `revision/change_transaction.rs` | The one-mutation-path placement is structurally correct, but formal Change collection/authorization must not be on the ordinary Design path until a real adopted policy and owner-approved workflow require it. |
| `revision/reproduction.rs` and technical portions of `revision/release.rs` | Exact inputs, outputs, producer/invocation/environment identity, and byte-identical reproduction are valuable. Release lifecycle, controlled-document, title-block, and transmittal semantics remain quarantined. |
| `revision/impact.rs` and `revision/impact_analysis.rs` | Comparing a real subject to a real baseline is a valid future action. The current graph families, evaluations, and public workflow exceed the recovered baseline and have no authorized UI. |

### Quarantine: unvalidated authority and enterprise breadth

The following exact implementation modules remain in source but cannot support
a production-acceptance claim or successor work:

- authority workflow: `revision/approval.rs`, `revision/role.rs`,
  `revision/effectivity.rs`, `revision/departure.rs`;
- lifecycle workflow: `revision/change.rs`,
  `revision/change_transaction.rs`, `revision/release.rs`,
  `revision/release_projection.rs`, `revision/release_transaction.rs`;
- analysis/evidence workflow: `revision/impact.rs`,
  `revision/impact_analysis.rs`, `revision/reproduction.rs`;
- exchange and enterprise boundary: `revision/exchange.rs` and its adapter,
  mapping, signing/verification, or transport-facing concepts;
- broad state machinery: product-authority portions of `revision/authority.rs`,
  `revision/authority_query.rs`, `revision/authority_store.rs`,
  `revision/authority_validation.rs`, `revision/resolver.rs`,
  `revision/transaction.rs`, and the public re-exports in `revision/mod.rs`;
- Project adoption and Preferences seam: `revision/policy.rs` and
  `revision/seed.rs`, including the deferred keys
  `datum.revision.profile_seed`,
  `datum.revision.build_presentation_seed`, and
  `datum.revision.prototype_transition_seed`.

Unit and substrate tests in `revision/*_tests.rs`,
`crates/engine/src/substrate/tests/revision_*.rs`, and
`crates/engine-daemon/src/tests/main_tests_revision.rs` preserve implementation
evidence only. Passing tests do not lift quarantine or prove a product need.

### Quarantine: mandatory ordinary-Project integration

| Path | Current coupling | Required future decision |
| --- | --- | --- |
| `crates/engine/src/substrate/commit.rs` | Every journaled commit calls `prepare_revision_design_commit`, finalizes a possible authority snapshot, and stages/promotes an integrity generation through `RevisionAuthorityStore`. | Separate universally useful technical history from optional formal Revision authority. Prove unmanaged library/schematic/PCB mutations remain prompt-free and do not manufacture formal records. Do not create a second mutation path. |
| `crates/engine/src/substrate/project_resolver.rs` | Every Project open inspects/recovers `.datum/revision/v1`; Revision integrity state can add normal resolve diagnostics. | Decide whether this is generic journal integrity, an optional adopted capability, or both with separate storage/diagnostics. Unmanaged Project opening must not imply formal Revision state. |
| `crates/engine/src/substrate/undo_redo.rs`, `proposal.rs`, replay/journal code, and operation application modules | Carry transaction links, revision inputs, or integrity assumptions through ordinary mutation plumbing. | Retain one-path atomicity, but quarantine formal Revision semantics until the commit boundary is redesigned. |

RVR-I01 does not change these engine paths. They are identified now so removal
of visible fiction cannot be misrepresented as complete Revision recovery.

## Public surface inventory

| Exact paths | Current exposure | Disposition |
| --- | --- | --- |
| `crates/engine/src/revision/public_service.rs` | Advertises complete operations, queries, refusals, proposal twins, and record families. Its generic query validates a query name but returns the same snapshot-shaped payload for all accepted names, with only historical sequence filtering. | Quarantine. It overstates implemented product semantics and uses pre-recovery authority wording. |
| `crates/engine/src/api/native_write/revision.rs` and its export from the native API | Re-exports the broad catalog/query service as a public read facade. | Quarantine public acceptance; later choose a narrow honest diagnostic or remove it. |
| `crates/verb-registry/src/verbs_revision.rs` | Marks `datum.revision.catalog` and `datum.revision.query` Public. | Quarantine from the public verb inventory in a separately authorized slice; generated catalogs must follow registry truth. |
| `crates/engine-daemon/src/dispatch.rs` | Dispatches `revision.catalog` and `revision.query`. | Quarantine with the registry/API surface. |
| `crates/cli/src/args/revision.rs`, `crates/cli/src/commands/revision.rs`, and their root/dispatch modules | Exposes catalog and generic query commands. | Quarantine with the engine service; do not keep a CLI promise that the product no longer makes. |
| `mcp-server/datum_tool_catalog.json` and MCP fence tests | Publish generated Revision tools to agents. | Regenerate only after an authorized registry change; never hand-edit the generated catalog to hide drift. |

The public-surface quarantine is not part of RVR-I01 because it changes external
contracts and requires its own compatibility decision. Until then, documentation
and roadmap claims must label these verbs quarantined and must not direct users
or agents to rely on them.

## Non-Revision matches explicitly excluded

A broad text search also finds “revision” used as ordinary model-version,
artifact-revision, journal-revision, or standards vocabulary in board,
schematic, DRC, output, pool, terminal, and CLI modules. Those references are
not Product Revision Engine surfaces merely because they use the English word
or a `ModelRevision` fence. RVR-I01 must not sweep them up. The inventory is
bounded by typed Product Revision imports, `datum.revision.*` registration,
`.datum/revision/v1`, `RevisionWorkspace*`, `RevisionNav*`, and
`PaneContent::Revision` reachability.

## RVR-I01 authorized correction

RVR-I01 is limited to removing fictional Revision presentation from ordinary
running workspaces:

1. Disconnect the always-rendered Revision Navigator rows and their hit regions.
2. Remove ordinary runtime creation/activation of hard-coded Revision panes and
   Inspector content.
3. Ensure the application default state contains no selected Revision item,
   open Revision pane, badge, placeholder, teaching strip, or fictional record.
4. Retain explicit fixture-only render scenarios only when their entry point and
   test naming make the example boundary unmistakable.
5. Preserve generic pane clipping, minimum width, close behavior, scrolling,
   Board/Schematic layout, terminal isolation, and all non-Revision navigation.

RVR-I01 explicitly does **not**:

- implement the contextual replacement doorway;
- decide controlled-project record browsing;
- adopt or expose Project revision policy;
- edit any Claude-owned `docs/gui/prototypes/*.html` file;
- alter journal/Project-open integration;
- change the public CLI/daemon/MCP contract;
- delete the quarantined engine wholesale; or
- authorize any later Revision, Publish, Preferences, enterprise, adapter, or
  dependency work.

## RVR-I01 proof contract

Completion requires all of the following:

- static production-reachability checks show no unconditional
  `render_navigator` call and no ordinary startup path constructing
  `PaneContent::Revision` from fixture identities;
- renderer/app regression tests prove an unmanaged default workspace contains
  none of `Changes`, `Baselines`, `Releases`, `Controlled Documents`,
  `Evidence`, `CHG-0031`, `BL-2026-08-24-01`, `RC-0009`, or `RLS-0007` as
  Revision UI output or hits;
- existing generic pane clipping/minimum-width/close and layer scrolling proofs
  remain green;
- a guarded focused Rust test/check set for the touched GUI crates passes;
- a running unmanaged real Project shows Design and Publish navigation only,
  readable Board and Schematic panes, no Revision surface, and the complete
  layer-list scroll range;
- evidence traceability, spec governance/parity, progress coverage, dependency
  authority, and project-state checks pass.

After RVR-I01, the next recovery proof is ordinary real-project authoring under
PM-038. Public API containment and engine decoupling remain named future
recovery decisions, not silently implied work.

## Owner review consequence

This inventory makes one immediate product correction comprehensible: remove
the false Revision product from the normal workspace while preserving the
ordinary EDA application. It deliberately does not ask the owner to approve a
replacement information architecture, authority model, or enterprise workflow.
Those questions return one at a time only after the running unmanaged workspace
is clean.

<!-- REQ:REVISION-PRODUCT-RECOVERY:RVR-A03 -->
<!-- EVIDENCE:REVISION-RECOVERY:RVR-A03-INVENTORY -->

## RVR-A04 roadmap re-entry packet

### Owner direction now controlling the re-entry audit

The owner approved the recovered unmanaged workspace and then clarified the
intended settings architecture and build order:

- the application menu bar is the entry point for system/user and Project
  configuration;
- Global Preferences owns machine/user preference behavior and may provide
  eligible new-Project defaults, but it cannot adopt or mutate policy in an
  existing Project;
- Project Preferences owns deliberate configuration of the current Project,
  including whether and how that Project adopts Revision behavior;
- configuration categories never appear in the Project Navigator;
- an internal Revision service may exist headlessly, but no product-facing
  Revision configuration or workflow is uncovered before its proper settings
  surfaces are implemented and stable; and
- operational results, if later authorized, are distinct from configuration
  and may use contextual closable work surfaces rather than permanent
  navigation groups.

This yields the intended dependency direction:

`preference authority -> preference GUI -> Project Revision configuration -> contextual Revision operations`

The exact menu composition remains to be reconciled. The product spec currently
places `Preferences` under Edit and `project settings` under Project, while the
owner described one menu-bar settings doorway capable of reaching both Global
and Project scopes. This packet records the conflict; it does not choose the
final menu hierarchy.

### Current implementation and authority facts

| Surface or subsystem | Current governed/runtime state | Re-entry consequence |
| --- | --- | --- |
| Global Preferences authority | Product Mechanics 037 and the 54-descriptor catalog are ratified, with Revision descriptors explicitly deferred by Product Mechanics 038. | Reusable typed authority exists on paper, but its Revision clauses are not implementation authority. |
| Global Preferences runtime | `GLOBAL-PREFERENCES-ENGINE` is blocked by the unfinished shared Units engine. Its current plan implements resolver, storage, Project seeding, and engine/CLI/MCP parity before the wgpu Preferences window at GP-I09. | It cannot be selected for execution now, and its late-GUI sequence has not yet been reconciled with the owner's Preferences-first product dependency. |
| Global Preferences application entry | `DATUM_GUI_MENU_BINDINGS.md` marks `Preferences` as `NOT-BUILT`. Runtime code contains only the isolated Console duration preference file, not a general Preferences engine or window. | The normal settings doorway does not exist in the running application. |
| Project Preferences | The two Claude-owned Project Preferences studies are clay and explicitly say the surface is not specified. No runtime Project Preferences window exists. | There is no ratified or buildable writable Project-settings surface into which Revision configuration can safely integrate. |
| Revision engine | Substantial engine and public-surface code exists, but Product Mechanics 038 defers the execution item and quarantines broad authority, commit/open coupling, public catalog/query behavior, policy gates, and enterprise adapters. | Passing code tests or retained internals cannot expose a product workflow. The default fictional GUI has been removed and stays removed. |
| Revision settings | The four `datum.revision.*` descriptors are deferred; the current Global Preferences render shows them as searchable retired/deferred vocabulary with no control or default. | Neither a global visibility switch nor a new-Project Revision seed may be implemented from the current catalog. |

The sequencing failure is therefore concrete: Revision implementation reached a
visible product surface while the general Preferences entry was still
`NOT-BUILT` and Project Preferences was still unratified clay. Recovery fixed
the visible result, but the dependency must be corrected before either product
area resumes.

### Queued Frontier state

This is a status inventory, not a ranking or selection.

| Frontier item | Current state | What selection would and would not mean |
| --- | --- | --- |
| `GUI-SURFACE-SPECS` | planned; planning-authorized | May specify the manual-first schematic and library surfaces. It does not repair Preferences or authorize Revision. |
| `UVT-S5A-BUILD` | specified; planning-authorized | Has no current completion contract or execution authorization. It cannot be treated as a ready implementation start. |
| `GUI-MARKING-MENU` | specified; planning-authorized | Covers an inert marking-menu shell only. It does not create the application settings doorway. |
| `DISTRIBUTED-COLLAB-SPEC` | planned; planning-authorized | Is independent future collaboration specification work, not settings recovery. |
| `ADOPTED-DRAFTING-STANDARD-SPEC` | planned; planning-authorized | May specify Project documentation authority, but does not supply Project Preferences itself. |
| `SHARED-UNITS-ENGINE` | specified; owner-decision boundary | Is the existing hard prerequisite to `GLOBAL-PREFERENCES-ENGINE`; selecting it asks the owner to review UNIT-I00 and does not build Preferences UI. |
| `GLOBAL-PREFERENCES-ENGINE` | blocked; owner-decision authorization recorded | Cannot start until shared Units closes. Its current completion order still places the wgpu Preferences surface at GP-I09. |
| GUI P2 cross-probe, full Inspector, GUI write path, and native authoring | blocked | Their blockers remain intact; recovery creates no shortcut around them. |
| `MCAD-INTEROP-PLACEHOLDER` | planned; no authorization | Is a placeholder, not actionable work. |
| `PRODUCT-REVISION-ENGINE` | deferred; no authorization | Remains quarantined and cannot be selected through recovery approval. |

### Decision-ready re-entry boundary

No existing executable Frontier item represents “build and stabilize the
Global and Project Preferences settings doorway before exposing Revision.” The
Global Preferences engine is blocked and orders its GUI late; Project
Preferences has no governed specification or implementation item; Revision is
deferred. Selecting any of those as though the desired sequence already existed
would repeat the same planning error.

At RVR-O02 the owner can safely do one of two things:

1. select an already-governed unrelated Frontier key from the table, accepting
   the exact limited meaning stated there; or
2. direct a bounded audit to reconcile the Preferences-first execution order,
   specify the Global/Project settings doorway, classify Global versus Project
   Revision configuration, and return with a small ratification packet before
   any implementation.

The second response is the route that corresponds to the owner's newly stated
goal, but this packet does not select it. The required response remains the
closed RVR-O02 form:

`REVISION-RECOVERY-NEXT: <Frontier key>`

or

`REVISION-RECOVERY-NEXT: audit — <specific uncertainty>`

<!-- EVIDENCE:REVISION-RECOVERY:RVR-A04-ROADMAP-REENTRY -->

## Dependency and licensing impact

None. This inventory adds no dependency and grants no exception to Product
Mechanics 029.
