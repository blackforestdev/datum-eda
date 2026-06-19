# M7 Render Layer Discipline Memo

> **Status**: Active team memo.
> This note records the render-architecture issue exposed during imported-board
> fidelity work and sets the rule for follow-on renderer changes.

## Problem

The current renderer is not doing the worst possible thing, but it is still a
**hybrid**:
- partly layer-driven
- partly object-class driven
- partly special-case driven

That hybrid shape is now load-bearing enough that it needs to be named
explicitly, otherwise the team will keep treating layer-fidelity failures as
isolated styling bugs instead of as a renderer-contract issue.

## What Is Good

The codebase already has real layer-oriented infrastructure:
- scene primitives carry `layer_id` keys in `board_review_scene_v1`
- the renderer resolves a `LayerAppearance` from the scene layer list
- copper layers are rendered bottom-to-top by layer family
- normal authored copper objects are generally gated by `layer_visible(...)`
- live component silk/text now also resolve color through the scene layer table
- live selected/moving pads now follow explicit copper-layer membership too

This is **not** a system where each imported item gets an ad hoc color baked in
at import time.

## What Is Still Wrong

The renderer is still not disciplined enough to say:
"for authored board geometry, layer/material grouping is primary and object
class is secondary."

Concrete evidence in the current code:

1. `LayerAppearance` is still object-class shaped.
   File:
   - [crates/gui-render/src/lib.rs](/home/bfadmin/Documents/datum-eda/crates/gui-render/src/lib.rs:361)
   Current fields include:
   - `authored_track`
   - `pad_copper`
   - `zone_fill`
   This is better than item-by-item styling, but it still encodes separate
   material slots per primitive family inside a layer. The dead
   `pad_highlight` slot has now been removed; the remaining shape is tighter
   but still not fully material-first.

2. The main geometry pass is still partly hand-authored by primitive class.
   File:
   - [crates/gui-render/src/lib.rs](/home/bfadmin/Documents/datum-eda/crates/gui-render/src/lib.rs:1575)
   Current ordering is effectively:
   - zones
   - tracks
   - copper pads
   - components
   - post-copper stages sorted through the shared render-stack policy
   - vias
   - outline
   This is better than the earlier bespoke pass order because mask, paste,
   silkscreen, mechanical graphics, and board graphics now flow through the
   same declared post-copper stage walk. It is still not a full generalized
   layer/material pipeline because copper, vias, component bodies, and the
   final outline overlay remain explicit families.

3. Through-hole pads are still a special-case render path.
   File:
   - [crates/gui-render/src/lib.rs](/home/bfadmin/Documents/datum-eda/crates/gui-render/src/lib.rs:1773)
   Even after the recent visibility fixes, through-hole pads are rendered in a
   dedicated post-layer pass using separate visibility logic and a multi-layer
   display rule. That may be a justified exception, but it is still an
   exception.

4. Vias now inherit copper material directly, but still remain a distinct
   geometry family.
   File:
   - [crates/gui-render/src/lib.rs](/home/bfadmin/Documents/datum-eda/crates/gui-render/src/lib.rs:1860)
   The old dedicated `via_outer` / `via_inner` appearance slots have been
   removed; the live via path now uses the visible copper layer's material
   color directly. This is an improvement, but vias are still drawn through a
   dedicated primitive path rather than a unified copper geometry pipeline.
5. Outline and `board_graphics` remain coordinated special cases.
   Files:
   - [crates/gui-render/src/lib.rs](/home/bfadmin/Documents/datum-eda/crates/gui-render/src/lib.rs:1935)
   - [crates/gui-render/src/lib.rs](/home/bfadmin/Documents/datum-eda/crates/gui-render/src/lib.rs:1954)
   This is probably the right product choice for now, but it means the
   renderer still contains multiple authored-geometry families with different
   draw roles.

## Product Rule

For authored board geometry, the renderer must follow this rule:

**Layer/material semantics are primary. Primitive-class treatment is secondary.**

Operationally, that means:
- layer ownership decides visibility
- layer family decides the default material vocabulary
- primitive class may refine stroke/fill behavior, but should not invent a
  separate semantic color system
- exceptions such as vias, through-hole pads, and board-boundary views must be
  explicit and justified in product terms

## What The Team Must Stop Doing

- Do not argue from what the code "intends" to do when the live GUI violates
  layer semantics.
- Do not add more one-off render branches for imported objects without writing
  down why they cannot fit the layer/material model.
- Do not treat per-object-class coloring as equivalent to a layer-driven
  renderer just because the colors happen to be similar.

## Immediate Engineering Guidance

Use this review question for every authored-board render change:

> "Is this object inheriting its visibility and base appearance from its owning
> layer/material model, or are we sneaking in another object-class-specific
> rendering rule?"

If the answer is the latter, the change needs one of two things:
- redesign into the normal layer/material path
- or an explicit bounded-exception note in the ticket/brief

## 2026-04-16 Render Stack Rule

As a direct follow-on from imported-board fidelity work, the renderer now needs
an explicit **render-stack policy** rather than more pass-local ordering fixes.

Rule:
- sort by **layer type group** first
- sort by **front/back side** second
- use scene `render_order` only as a stable tie-breaker inside a stage

Current intended authored-board stage order:
- bottom copper
- inner copper
- top copper
- bottom mask
- top mask
- bottom paste
- top paste
- bottom silkscreen
- top silkscreen
- mechanical / fab / courtyard
- edge / board-boundary overlay

This means:
- `F.Paste` renders above `B.Paste`
- `F.Mask` renders above `B.Mask`
- `F.Paste` renders above `F.Mask`
- front silkscreen renders above front mask/paste
- post-copper authored geometry now follows one shared stage walk in code
  rather than separate local loops for process, silk, mechanical, and edge

This is still not a full material-pipeline rewrite, but it is now a declared
renderer contract and must be treated as such in code review.

## Expected Follow-On

This memo does **not** require an immediate renderer rewrite.

It does require:
- a tracked renderer-contract ticket for layer/material discipline
- future fidelity tickets to reference that rule
- code review to reject new ad hoc render-property assignments for imported
  geometry

The right near-term outcome is a stricter hybrid renderer with fewer
exceptions, not a rushed abstraction rewrite.

## 2026-06-09 Enforcement Slice (M7-REN-006 closure)

The discipline this memo demanded is now enforced in code:

- **One ordering encoding.** `RenderStage` declaration order is the draw
  order; `render_stage_priority` is the discriminant. The previously
  divergent derived `Ord` (which encoded paste-before-mask, contradicting
  the 2026-04-16 rule) was corrected, and the three duplicated hand-authored
  `post_copper_stages` arrays were consolidated into one shared
  `POST_COPPER_STAGES` walk.
- **Material-first construction.** Known copper families build
  `LayerAppearance` through `from_copper_material`, making the rule that
  tracks, pads, and zones inherit one base material color structural rather
  than coincidental. The unknown-layer fallback keeps deliberately divergent
  colors as a bounded exception so unresolved layer identity stays visible.
- **Bounded exception set documented in code.** The
  `push_retained_scene_geometry` contract header enumerates the explicit
  exceptions: through-hole pad pass, via geometry family, board
  outline/Edge overlay, selection/hover emphasis, unknown-layer fallback.
  Growing that list requires a memo note.
- **Contract regression tests.** `render_stack_policy_follows_declared_contract`,
  `render_stage_declaration_order_is_the_only_priority_encoding`, and
  `copper_layer_appearance_is_material_first` lock the stage ladder, the
  single-encoding rule, and material inheritance.

What this slice deliberately did **not** do, per this memo's own boundary:
unify vias and through-hole pads into a generalized copper pipeline. Those
remain explicit, documented exceptions; revisit only with a product-justified
follow-on ticket.
