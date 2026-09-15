# M7 Render Semantic Contract

> **Status**: Active semantic contract for the opening `M7` renderer.
> This note defines how the viewport should read to a PCB reviewer before
> aesthetic polish. It is a bounded product-semantic document, not a style
> exploration memo.

## Purpose

Lock the minimum visual vocabulary for the opening `M7` board-review viewport
so renderer changes can be judged against product meaning instead of taste.

This contract covers the four semantic lanes already present in the opening
slice:
- authored board geometry
- unrouted connectivity
- proposed review geometry
- diagnostic / review emphasis

## Core Rule

The viewport must read like a PCB review tool before it reads like a generic
vector canvas.

That means:
- authored copper must read as fabricated board truth
- unrouted connectivity must read as ratsnest / intent, not copper
- proposed geometry must read as candidate copper, not as authored copper or
  unrouted linework
- diagnostic emphasis must modify the active lane without replacing its basic
  meaning

Imported-board semantic invariance rule:
- optional source representations such as KiCad `render_cache` may exist, but
  they may not define final imported-board truth
- position, orientation, layer ownership, visibility behavior, and review-state
  meaning must remain equivalent whether imported text/graphics arrive from
  cached KiCad polygons or Datum fallback synthesis
- imported-board architecture should converge toward Datum-owned geometry
  generation from text/graphic semantics
- for imported board text specifically, follow
  `research/pcb-text-rendering/PCB_TEXT_RENDERING_RESEARCH.md`: Phase 1 uses a
  Datum-owned Newstroke-equivalent generator, treats strokes as canonical, and
  does not use `render_cache` as the general parity oracle
- any board-to-board difference caused only by representation presence or
  absence is a regression, not an acceptable fixture quirk

## Lane Vocabulary

### Authored

Authored geometry is the persisted board truth.

It includes:
- copper tracks, pads, zones, vias
- process layers such as mask and paste
- silkscreen and bounded board/footprint display companions
- `Edge.Cuts` authored graphics

It must read:
- stable
- fabrication-oriented
- layer-owned

It must not read:
- animated
- inferred
- speculative

### Unrouted

Unrouted geometry is remaining connectivity intent from engine truth.

It includes:
- ratsnest spans
- endpoint markers showing where a remaining connection terminates

It must read:
- clearly non-copper
- subordinate to authored copper
- available for visibility toggling as its own lane

It must not read:
- like routed copper
- like a proposal overlay
- like random selection noise

Minimum grammar:
- line-based, not filled copper geometry
- thin solid linework by default, not dashed
- only a subtle endpoint anchor is allowed by default, and it must never read
  as a via, drill, or copper feature
- distinguishable under dim/unrelated review states

### Proposed

Proposed geometry is candidate copper for review.

It must read:
- copper-like
- intentional
- reviewable as a possible authored outcome

It must not read:
- like unrouted ratsnest
- like authored copper already on the board
- like generic path editing handles

### Diagnostic

Diagnostic emphasis is a modifier, not a separate fabricated object family.

It may:
- brighten
- focus
- dim unrelated context
- add bounded emphasis markers

It must not:
- erase lane identity
- cause authored copper to read as unrouted
- cause proposed geometry to read as authored truth

## Stack Rule

The renderer stack must follow declared product semantics, not opportunistic
draw order.

Current rule:
1. authored copper families
2. authored process layers
3. authored silks / mechanical context
4. authored `Edge.Cuts` graphics
5. unrouted lane
6. proposed review overlays
7. diagnostic emphasis / review-specific accents
8. bounded board-frame outline exception

Within a layer family:
- layer type comes before front/back side choice
- back-side draws first
- front-side draws later
- stable tie-breakers follow after semantic ordering

## Acceptance Questions

Every renderer change in the opening `M7` slice should be answerable with these
questions:

1. Can a PCB reviewer immediately distinguish authored copper from unrouted
   connectivity on the canonical half-routed board?
2. Can a PCB reviewer tell proposed geometry from both authored copper and
   unrouted connectivity?
3. Do diagnostic focus and dim states preserve the meaning of the underlying
   lane?
4. Does turning a layer off actually remove geometry owned by that layer?
5. Does the view still read correctly without side-by-side explanation?
6. Would the same authored intent still read the same way if the KiCad source
   omitted `render_cache` and forced Datum to synthesize the geometry?

If a change cannot be justified in those terms, it is outside the semantic
contract and should be treated as design polish or scope creep.
