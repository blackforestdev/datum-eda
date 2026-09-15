# M7-INT-001 Interaction Stability Brief

> **Ticket**: `M7-INT-001`
> **Track**: Opening `M7` interaction substrate
> **Status**: First slice closed 2026-06-09 — proof obligations landed in
> `crates/gui-render/tests/selection_ownership.rs` (exclusive selection
> ownership, switch-clears-prior, hover-preview-only) plus existing
> protocol-level review-target persistence tests.

## Purpose

Define the first bounded implementation slice for the `M7-INT-001` interaction
stability track.

This brief exists to turn the current broad blocker:
- selection ownership
- hit-testing ownership
- focus / relatedness behavior
- dimming trustworthiness

into one concrete engineering slice that can land cleanly and unblock the next
renderer/readability work.

## Chosen First Slice

The first `M7-INT-001` slice is:

- **authored-object selection ownership and relatedness stability**

This means:
- clicking or programmatically selecting one authored object must produce one
  stable, explainable ownership result
- the selected object, related authored context, and unrelated context must be
  distinguishable enough for dimming/readability work to build on top
- active review target and authored selection must coexist without silently
  replacing one another

This slice is intentionally narrower than "fix all interaction behavior".

## Problem

The current codebase has the right state vocabulary, but the execution frontier
is still too ambiguous:

- `ReviewWorkspaceState` already separates `selection` from
  `active_review_target_id`
- the renderer already computes `selected`, `related`, and `dimmed` states
- the app already has both UI-rect hit testing and world-space authored hit
  testing

But the remaining blocker is trust:

- a tester must be able to tell which authored object actually owns the current
  selection
- nearby components/pads must not read as co-selected by accident
- the currently selected authored object must remain inspectable while the
  active review target persists
- dimming must not collapse "related" and "unrelated" into the same weak visual
  result

Without that, `M7-REN-003`, `M7-REN-004`, and broader readability work are hard
to evaluate honestly.

## Current Evidence

Relevant code surfaces:

- [crates/gui-protocol/src/lib.rs](/home/bfadmin/Documents/datum-eda/crates/gui-protocol/src/lib.rs:1832)
  `ReviewWorkspaceState` owns `selection`, `active_review_target_id`,
  `hovered_object_id`, and `dim_unrelated`
- [crates/gui-app/src/main.rs](/home/bfadmin/Documents/datum-eda/crates/gui-app/src/main.rs:1769)
  click dispatch and authored-object selection path
- [crates/gui-app/src/main.rs](/home/bfadmin/Documents/datum-eda/crates/gui-app/src/main.rs:2009)
  hover ownership update path
- [crates/gui-render/src/lib.rs](/home/bfadmin/Documents/datum-eda/crates/gui-render/src/lib.rs:1589)
  dimming activation and hover/selection helpers
- [crates/gui-render/src/lib.rs](/home/bfadmin/Documents/datum-eda/crates/gui-render/src/lib.rs:1638)
  selection-relatedness helpers
- [crates/gui-render/src/lib.rs](/home/bfadmin/Documents/datum-eda/crates/gui-render/src/lib.rs:6880)
  existing renderer tests for component-selection ownership, including ignored
  tests that show this area has already been a real problem

Supporting product rules:

- [docs/gui/M7_DELIVERY_GATES.md](/home/bfadmin/Documents/datum-eda/docs/gui/M7_DELIVERY_GATES.md)
- [docs/gui/INTERACTION_MODEL.md](/home/bfadmin/Documents/datum-eda/docs/gui/INTERACTION_MODEL.md)
- [docs/gui/M7_BOARD_REVIEW_FIDELITY_GAP.md](/home/bfadmin/Documents/datum-eda/docs/gui/M7_BOARD_REVIEW_FIDELITY_GAP.md)

## Scope

This slice covers:

- authored-object selection ownership for components and their immediate
  rendered companions
- authored-object hover ownership where it affects the same visual state model
- stable coexistence of:
  - one explicit authored selection
  - one separate active review target
- selected / related / unrelated state separation strong enough for downstream
  dimming/readability work
- focused regression coverage proving that selection does not leak across nearby
  components

This slice does **not** cover:

- multi-select
- keyboard shortcut expansion
- terminal/assistant focus changes beyond preserving selection
- generalized review-list UX redesign
- full render-material discipline (`M7-REN-006`)
- proposal-overlay styling (`M7-REN-003`)
- final dimming/readability tuning (`M7-REN-004`)

## Required Behavior

After this slice lands:

1. Selecting one component must not visually select neighboring components.
2. The selected component's owned geometry must be the only geometry that reads
   as selected.
3. Geometry related to the active review target may remain related, but must
   not collapse into "selected" unless it is the explicit authored selection.
4. The active review target must persist while the user inspects authored
   geometry.
5. Hover must remain preview-only and must not replace selection or review
   target.
6. `DIM UNRELATED` must act on a stable ownership model, even if the exact
   final dimming strength is tuned later.

## Minimum Code Surface To Audit

The implementation pass must inspect at least:

- `crates/gui-protocol/src/lib.rs`
  - `ReviewWorkspaceState::select_review_action`
  - `ReviewWorkspaceState::select_authored_object`
  - `ReviewWorkspaceState::clear_selection`
- `crates/gui-app/src/main.rs`
  - `handle_primary_click`
  - `select_hit_target`
  - `update_hover`
- `crates/gui-render/src/lib.rs`
  - hit-region registration for authored objects
  - selection/relatedness helpers
  - component/pad/text rendering branches that currently derive selected or
    related visual state
  - any ignored tests around component selection ownership

## Test Direction

The first proof should be built around component-selection ownership, because
that is the narrowest stable slice already supported by repo fixtures and test
shape.

Minimum proof obligations:

- selecting `Q3` does not emit selected geometry inside `C1` pad bounds
- selecting `Q2` does not emit selected geometry inside `Q1` pad bounds
- switching authored selection from one component to another clears the prior
  selected geometry
- selecting an authored component does not clear the active review target
- hover remains preview-only and does not overwrite explicit selection

Where older tests are ignored because they depended on one exact color token,
replace them with lane-aware assertions instead of restoring brittle
color-exact checks.

## Acceptance Criteria

`M7-INT-001` first slice is complete only when all of the following are true:

- one authored-object selection has clear ownership on the canonical fixture
- nearby components no longer read as co-selected under the tested cases
- active review target persists while authored selection changes
- hover does not silently replace explicit selection
- regression tests cover the ownership cases above
- the resulting state is strong enough that `M7-REN-003` / `M7-REN-004` can be
  judged on readability rather than on selection ambiguity

## Deliverable Summary

Expected patch shape:

- small protocol/app/render interaction fixes where needed
- focused renderer/hit ownership cleanup
- replacement or un-ignore strategy for existing ownership tests
- no broad UX redesign and no scope expansion into unrelated GUI work
