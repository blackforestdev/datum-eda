# Datum Selection And Compound Editing Guidance

Status: governed research-derived guidance; S5 design integration in progress

Research basis:
`research/gui-compound-selection/GUI_COMPOUND_SELECTION_RESEARCH.md`.
Behavioral authority remains
`docs/gui/DATUM_UNIVERSAL_VIEWPORT_TOOLING_SPEC.md` §2.2 and the numbered
selection-identity decision to be ratified at final S5 specification review.
This guidance does not independently authorize implementation or rival Active
Frontier sequencing.

## Purpose

Translate the compound-selection research into a durable classification that
the S5 specification and later domain-tool contracts can consume. DNP is one
illustrative batch attribute among many; it is not a privileged organizing
surface.

## Controlling guidance

1. A temporary multi-selection is a compound Inspector subject, not an authored
   group. Reserve `Group <name>` for an explicitly created persistent object.
2. The Inspector keeps canvas membership intact while exposing `All N` and
   explicit per-type scopes. Scope changes are view/target declarations, not
   hidden reselection.
3. Common values render normally, divergent values render `Mixed`, and
   unavailable properties explain incompatibility. Compatibility is typed
   semantic identity plus value domain/units/verb, never display-label equality.
4. Every batch edit states its exact affected set, preflights the complete
   declared scope, and commits all-or-nothing through one typed operation batch
   and one undo step. Locked, stale, invalid, constrained, or incompatible
   members are never silently skipped.
5. Derived properties—bounds, coverage, effective rules, connectivity,
   population, checks, provenance—remain inspectable but are not written back as
   aggregate member values.
6. Connectivity-, hierarchy-, rule-, library-, variant-, manufacturing-, or
   generated-geometry consequences route to dedicated domain tools rather than
   a generic property field.

## Delivery classification

### S5A — selection and compound inspection

- Selection acquisition, membership, optional focus, lifecycle, pane ownership,
  cross-workspace projection, and active-workspace GUI authority.
- Ephemeral compound Inspector subject with stable identity inventory, counts,
  combined bounds/reference, workspace/layer/net coverage, hidden/locked state,
  `All N` and per-type scopes, and Common/`Mixed`/Unavailable presentation.
- Read-only derived attributes and exact blocker reporting.

### S5B — selection authority substrate

- Persistent authored Group model and typed create/set/delete operations.
- Universal lock vocabulary or an explicit per-object capability matrix.
- Field-level typed batch patch/operation contracts with revision guards.
- Atomic transforms already ratified for compatible targets: translation,
  rotation, and schematic-symbol horizontal/vertical mirror.
- Exact preflight, one commit, one undo step, and no partial success.

### Later dedicated domain tools

- PCB track/via bulk editing, zone manager/refill, padstack, text/graphics,
  layers/stackup, rules/netclasses, component side transfer, annotation, and
  alignment/distribution.
- Schematic symbol fields, connectivity-aware wire/bus/label/port tools,
  library pin/symbol/part authoring, and hierarchy operations.
- Variant/assembly population, DNP and exclusions, alternate parts, production
  plans/outputs, copy/delete closure, and persistent-group membership tooling.

These are sequencing boundaries, not product-scope rejection. The specification
must retain typed extension seams for later surfaces without claiming absent
model/operation authority has landed.

## Required reconciliations before S5 execution

- Replace the scalar/stringly selection substrate with the ratified typed set,
  optional focus, projection, and compound-subject model.
- Reconcile immediate-overlay selection law with retained-selection styling and
  tests.
- Reconcile the parametric-tooling locked-member skip rule with S5 atomic
  refusal; the final controlling decision must select one behavior.
- Define Group identity, membership, nesting/overlap, revision, lock, broken-
  member diagnostics, persistence, query, journal, undo/redo, CLI/MCP parity,
  and performance budgets before persistent Group is claimed.
- Define the lock capability matrix before a universal group lock/unlock field
  is exposed.
- Do not expose narrow Inspector fields over whole-object Set operations without
  fresh-state preservation and guarded atomic batch semantics.
- Keep DNP, BOM/board/simulation exclusions, and variant fitted state distinct.

## Traceability posture

The research report contains the external-source evidence, Datum substrate
audit, object-family attribute inventory, unsafe patterns, and open research
items. The UVT specification contains owner-ratified S5 behavior. The Active
Frontier in `specs/PROGRESS.md` alone controls sequence and implementation state.
