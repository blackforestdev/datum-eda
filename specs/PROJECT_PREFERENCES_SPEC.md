# Datum Project Preferences Specification

> **Status:** Pending specification under Product Mechanics 039. The doorway
> and authority laws below are ratified; category payloads, interaction details,
> and visual truth must complete PPS-C01 through PPS-C04 before implementation.

## 1. Product boundary

Project Preferences is Datum's writable settings surface for the currently
open Project. It is reached through
`Edit > Preferences > Project Preferences…`, which is disabled without an open
Project. It is not a Navigator node and opening it performs no mutation.

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
  bare numeric input. Its eventual Units row writes through the same journaled
  path and never rescales canonical geometry.
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

Codex must provide a bounded handoff and must not edit prototype HTML.

<!-- REQ:PROJECT-PREFERENCES-SPEC:PPS-C03 -->
### PPS-C03 — buildable interaction and conformance contract

Specify draft/apply/cancel behavior, cross-row validation, external-change
handling, stale generations, conflict/refusal presentation, focus restoration,
search, provenance, undo, accessibility, real-Project fixtures, and exact menu
bindings. No behavior may be copied from another application without owner
review.

<!-- REQ:PROJECT-PREFERENCES-SPEC:PPS-C04 -->
<!-- OWNER:PROJECT-PREFERENCES-SPEC:PPS-C04:PPS-C04 -->
### PPS-C04 — owner ratification and build placement

Return the complete inventory, protected renders, specification, and proposed
bounded implementation sequence to the owner. Approval may place but does not
execute the build.

## 5. Initial build boundary

<!-- REQ:PROJECT-PREFERENCES-BUILD:PP-I00 -->
<!-- OWNER:PROJECT-PREFERENCES-BUILD:PP-I00:PP-I00 -->
PP-I00 may authorize only one real Project Preferences vertical slice after
PPS-C04 approval.

<!-- REQ:PROJECT-PREFERENCES-BUILD:PP-I01 -->
PP-I01 implements the menu route, scope-safe read model, and one ratified real
Project setting through the canonical journaled mutation path.

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
- `project-preferences-category-study.html`, status/category region: retain clay
  until PPS-C01 proves each payload; remove any implication that code existence
  alone ratifies a category;
- `project-preferences-revision-gate.html`, whole artifact: preserve it as
  Revision-specific clay, state that Revision is absent from the initial
  Project Preferences build, and preserve the unmanaged/no-advertising law;
- any new Project Preferences primary window: draw only after PPS-C01, using
  real Project-owned rows and the Global window's established chrome without
  merging authority.

Proof expected: wide/narrow renders, disabled-without-Project menu state,
keyboard focus order and restoration, non-color scope cues, screen-reader names,
no Navigator setting, and an explicit clay/approved status for every artifact.

## 7. Dependency and licensing impact

None. This pending specification adds no dependency and grants no Product
Mechanics 029 exception.

<!-- EVIDENCE:PROJECT-PREFERENCES:PENDING-DOORWAY-SPEC -->
