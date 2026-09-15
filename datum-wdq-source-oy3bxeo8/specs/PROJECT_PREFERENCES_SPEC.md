# Datum Project Preferences Specification

> **Status:** Pending specification under Product Mechanics 039. UNIT-I02R now
> defines a buildable Units-only vertical slice, subject to corrected Claude-owned
> visual truth and UNIT-I02V owner approval. Every other category still requires
> PPS-C01 through PPS-C04 before implementation.

## 1. Product boundary

Project Preferences is Datum's writable settings surface for the currently
open Project. It is reached through
`Edit > Preferences > Project Preferences…`, which is disabled without an open
Project. That is a terminal menu command which opens the window directly;
**Units** is selected in the window's category rail, not exposed as another
submenu. Available and disabled states are separate menu examples and never
appear as duplicate commands in one menu. It is not a Navigator node and opening
it performs no mutation.

Global Preferences remains a separate command and authority. A read-only Global
row may link to the corresponding Project setting, but the two stores and
mutation paths never merge.

## 2. Authority and mutation law

- Every row names its Project scope and resolves from real Project authority.
- Every write reduces to a typed operation through the canonical journaled
  Project mutation path with provenance, diff, undo, and refusal semantics.
- No UI-local file, machine preference contribution, or private writer may
  change Project policy.
- Global seeds copy once at Project creation with a receipt and never
  live-follow later Global changes.
- `ProjectDisplayUnits`, presented as **Project Working Units**, is the
  Project-owned source for editor display, measurement readouts, and contextual
  bare numeric input. Its Units slice writes through the same journaled path and
  never rescales canonical geometry.
- Publish/document units remain separately owned by `AdoptedDraftingStandard`
  and Publish/document authority; neither silently follows Project Working
  Units.
- Invalid, stale, conflicted, or unauthorized drafts cannot replace the last
  valid effective Project state.
- Opening, searching, inspecting, canceling, or closing the window makes no
  Project change.

## 3. Visibility law

Only categories with ratified Project-owned payloads and real query/mutation
support may be visible. Empty categories, planned rows, mock values, and
fictional records are forbidden in the running product.

Revision is not part of the initial Project Preferences implementation. A
future owner decision must ratify its Project-owned policy rows and protected
visual truth after the base Project Preferences surface is stable. Merely
opening Project Preferences never adopts or advertises Revision control.

## 4. Required specification work

<!-- REQ:PROJECT-PREFERENCES-SPEC:PPS-C01 -->
### PPS-C01 — exact Project settings inventory

Inventory every Project-owned setting, governing authority, current persistence
shard, read query, typed mutation, refusal, undo behavior, seed relationship,
and explicit exclusion. Machine presentation, session state, operation input,
restartable UI state, and deferred Revision behavior must remain outside.
Product Mechanics 040 requires the inventory to include Project Working Units
and to keep Publish/document units visibly separate.

For the Units-only first slice, the exact inventory is the eight-field
`UnitsProfile` defined by `GP_SHARED_UNITS_ENGINE_REQUIREMENT.md`: measurement
system; Board unit and precision; Drill unit and precision; Schematic unit and
precision; and decimal-degree precision. Each field reads and writes
`ProjectDisplayUnits` through one typed aggregate mutation. The durable Project
seed or migration receipt is read-only provenance. No document, Publish,
Revision, machine, session, or operation-owned value is included.

<!-- REQ:PROJECT-PREFERENCES-SPEC:PPS-C02 -->
### PPS-C02 — protected visual reconciliation

The Claude-owned lane must reconcile the actual menu and windows for:

- `Global Preferences…` and `Project Preferences…` under Edit > Preferences;
- Project disabled with no open Project;
- unmistakable Global versus Project scope;
- read-only Global-to-Project doorway;
- no settings presence in the Navigator;
- only real, ratified Project categories;
- ordinary, narrow, keyboard-only, reduced-motion, non-color, and screen-reader
  states; and
- Revision absent from the initial Project surface.

The Units target must show only one **Units** category, scope
`Project · <project name>`, all eight values, per-quantity Unit/Precision pairing,
Automatic precision resolution, cross-system overrides, seed/migration
provenance, ordinary/changed/search/refusal/unreadable/narrow/non-color states,
and the lossless edit contract. It must not draw future categories merely to
reserve space.

Codex must provide a bounded handoff and must not edit prototype HTML.

<!-- REQ:PROJECT-PREFERENCES-SPEC:PPS-C03 -->
### PPS-C03 — buildable interaction and conformance contract

The Units-only slice uses immediate per-control commits, not staged Apply/OK/
Cancel. Each selection validates a complete `UnitsProfile` and submits one
canonical journaled Project mutation; on success it updates the read model and
offers ordinary Project Undo, while refusal leaves the last valid state intact.
Reset restores that row to the value recorded in this Project's immutable seed
or migration receipt through the same journaled, undoable mutation. It never
reads today's Global default, never changes the receipt, and is unavailable with
a typed reason if the receipt/value cannot be read.
An external generation change triggers re-read and explicit stale-edit refusal,
never last-writer-wins. Window close discards search, open choices, transient
notices, and focus but not committed Project values. Reopen selects Units with
an empty search and current Project values. Escape closes the innermost choice,
then clears a nonempty focused search, then closes the native window. Focus
returns to the invoking menu item with the previous editor as fallback.

Controls announce name, role, stored and resolved value, Project scope,
immediate-save behavior, changed/default provenance, seed or migration receipt,
and any cross-system/rounded/refusal state. Global defaults are read-only
provenance only and are never a fallback for an existing Project. Real-Project
fixtures must prove undo, stale refusal, reopen state, zero geometry mutation,
and deterministic migration of a pre-feature Project from a versioned Datum
factory profile—not from the current machine's Global settings.

<!-- REQ:PROJECT-PREFERENCES-SPEC:PPS-C04 -->
<!-- OWNER:PROJECT-PREFERENCES-SPEC:PPS-C04:PPS-C04 -->
### PPS-C04 — owner ratification and build placement

Return the complete inventory, protected renders, specification, and proposed
bounded implementation sequence to the owner. UNIT-I02V may ratify only the
Units slice and place UNIT-I03A/UNIT-I03B; it does not ratify or execute any
other Project Preferences category.

## 5. Initial build boundary

<!-- REQ:PROJECT-PREFERENCES-BUILD:PP-I00 -->
<!-- OWNER:PROJECT-PREFERENCES-BUILD:PP-I00:PP-I00 -->
PP-I00 ordinarily authorizes one real Project Preferences vertical slice after
PPS-C04 approval. The Project Working Units slice is instead co-owned by the
shared Units program and may execute only through UNIT-I03B after UNIT-I02V.

<!-- REQ:PROJECT-PREFERENCES-BUILD:PP-I01 -->
PP-I01 implements the menu route, scope-safe read model, and one ratified real
Project setting through the canonical journaled mutation path. UNIT-I03B must
reuse this architecture and evidence rather than create a second settings path.

<!-- REQ:PROJECT-PREFERENCES-BUILD:PP-I02 -->
<!-- OWNER:PROJECT-PREFERENCES-BUILD:PP-I02:PP-I02 -->
PP-I02 separately reviews the vertical slice before any complete-category work.

<!-- REQ:PROJECT-PREFERENCES-BUILD:PP-I03 -->
PP-I03 completes and production-accepts only the categories ratified by
PPS-C04. Revision configuration remains excluded.

## 6. Protected-lane handoff

Exact files and outcomes for Claude:

- `preferences-window.html`, application-menu/Project-policy doorway region:
  show the approved two-command Preferences family and preserve Global scope;
- `project-preferences-category-study.html`, status/category region: retain all
  non-Units categories as clay, but add the bounded UNIT-I02R Units-only target
  defined above without implying that code existence ratifies any other row;
- `project-preferences-revision-gate.html`, whole artifact: preserve it as
  Revision-specific clay, state that Revision is absent from the initial
  Project Preferences build, and preserve the unmanaged/no-advertising law;
- the Units-only Project Preferences target: use only the eight real
  Project-owned Working Units values and the Global window's established chrome
  without merging authority.

Proof expected: wide/narrow renders, disabled-without-Project menu state,
keyboard focus order and restoration, non-color scope cues, screen-reader names,
no Navigator setting, and an explicit clay/approved status for every artifact.

## 7. Dependency and licensing impact

None. This pending specification adds no dependency and grants no Product
Mechanics 029 exception.

<!-- EVIDENCE:PROJECT-PREFERENCES:PENDING-DOORWAY-SPEC -->
