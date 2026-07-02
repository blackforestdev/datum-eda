# M7 Delivery Gates

> **Status**: Historical -- opening `M7` delivery rules, spike closed-for-scope; retained as historical evidence.
> This note defines when a slice is allowed to count as "done enough" to move
> on without hiding dependency debt behind anti-scope-creep language.

## Purpose

Stop the team from advancing `M7` with underbuilt, low-resolution slices that
exist in code but are not externally testable, understandable, or supported by
the interaction/render substrate they depend on.

This note does **not** broaden `M7` into a larger milestone.
It changes the completion rule for work that is already inside scope.

## Root Problem

The current failure mode is not ordinary scope creep.
It is dependency-order failure:

- feature slices are being advanced because they match the written spec at a
  low resolution
- but the minimum substrate needed to trigger, observe, and understand those
  features is not always in place
- so downstream development and product testing become ambiguous
- and later features expose the missing pillars again as "collateral bugs"

That is not acceptable delivery quality for opening `M7`.

## Hard Rule

A slice may not be treated as complete enough to proceed unless it is:

- in scope
- externally triggerable in the current shell
- visually or behaviorally observable without insider knowledge
- backed by the minimum infrastructure needed for the claimed behavior
- testable without relying on undefined ownership, focus, hit-testing, or
  render-state assumptions

If those conditions are not true, the missing infrastructure work is **not**
"scope creep."
It is prerequisite completion for the slice already being claimed.

## Minimum Gates

Every user-facing `M7` slice must clear all of the following before the team
moves on.

### 1. Triggerability Gate

The tester must be able to intentionally trigger the feature in the current
runtime path.

Examples:
- a toggle actually changes something visible when its preconditions are met
- a selectable object can actually be selected from the geometry the user sees
- a review-mode behavior is not hidden behind an unavailable review-action path

### 2. Observability Gate

The tester must be able to tell what changed and what state the feature is in
without needing private implementation knowledge.

Examples:
- "selected", "related", and "unrelated" have distinguishable visible states
- unrouted, authored, proposed, and diagnostic lanes are visually separable
- a dimming feature does not look inert or random

### 3. Infrastructure Sufficiency Gate

The supporting substrate for the claimed behavior must exist and behave
coherently enough for the feature to be evaluated.

For `M7`, this especially includes:
- selection ownership
- hit testing
- focus / relatedness model
- visibility / layer semantics
- render-state consistency

If a feature depends on one of those pillars and that pillar is not stable
enough to support it, the slice is not ready to count as complete.

### 4. Readability Gate

A feature may emphasize one thing only if it does not make the rest of the
board unreadable.

Examples:
- dimming may de-emphasize unrelated context, but may not destroy board
  legibility
- selection may narrow attention, but may not make ownership ambiguous
- ratsnest may stand out, but may not read as copper, drills, or random
  overlays

### 5. Dependency Declaration Gate

If a slice requires supporting infrastructure beyond its local file edits, that
dependency must be stated explicitly in the working docs before the team claims
success.

Examples:
- "feature is blocked on component-cluster selection consistency"
- "feature requires visible selectable geometry to match hit ownership"
- "feature requires review focus to exist in the shell"

### 6. Standards Observable Gate

If a slice touches a standards-bound PCB domain, the expected observable must
be stated explicitly and checked against the governing research/source note
before the team claims success.

Examples:
- solder mask aperture is larger than copper when the imported source defines a
  positive mask expansion
- paste aperture is smaller than copper when the imported source defines stencil
  reduction
- land-pattern geometry does not silently collapse to an EDA-tool default when
  the source design carries explicit manufacturability intent

For current `M7` work, the default reference for these cases is:
- [IPC_COMPLIANCE_RESEARCH.md](/home/bfadmin/Documents/datum-eda/research/ipc-compliance/IPC_COMPLIANCE_RESEARCH.md)

This gate exists because "the host EDA package derives it this way by default"
is not an acceptable reason for Datum to ignore explicit manufacturability data
or industry-standard observables.

## What Does Not Count As Progress

The following do **not** count as acceptable completion:

- code exists but the tester cannot intentionally exercise it
- feature only works in a hidden or unavailable runtime path
- visible state change is so weak the tester cannot tell what happened
- behavior depends on unresolved selection / ownership ambiguity
- renderer polish is used to hide missing import or scene truth
- standards-bound geometry is accepted merely because it matches a host EDA
  package default, without checking whether the imported source and governing
  standard/research say otherwise
- a slice is called "done enough" only because expanding the prerequisite work
  would be labeled scope creep

## Required Team Behavior

When a tester reports "I cannot tell what is happening" or "the substrate does
not support this feature," the default team response must **not** be:

- "move on per spec"
- "that would be scope creep"
- "the feature exists at a low resolution"

The correct response is:

- identify the missing pillar
- decide whether that pillar is required for the claimed slice
- if yes, complete that prerequisite work before advancing

## M7 Pillars

The current opening `M7` pillars that repeatedly affect downstream work are:

- selection engine
- hit-testing ownership
- focus / relatedness model
- layer / visibility semantics
- render-state consistency

When one of these pillars is unstable, downstream feature work should be
treated as blocked by prerequisite infrastructure, not "done enough."

## Usage

Use this note when:
- deciding whether to advance the next `M7` slice
- closing a checklist item
- pushing back on "implemented per spec" claims that are not externally
  testable
- deciding whether dependency repair is required work or actual scope creep
