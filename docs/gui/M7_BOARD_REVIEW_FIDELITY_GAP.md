# M7 Board Review Fidelity Gap

> **Status**: Historical -- M7 opening-slice fidelity-gap checkpoint, spike closed-for-scope; retained as historical evidence.
> This document does not change `M7` scope or semantics. It identifies the
> fidelity gap between the currently working `M7` route-review client and the
> level of board-review credibility required for a serious EDA product.

## Purpose

Record, in precise product and technical language, why the current `M7`
route-review screen still feels unlike a credible PCB review surface even
though the frontend architecture and read-only review workflow are now working.

This document exists to:
- define the actual gap
- separate renderer-only issues from scene-contract limitations
- give the next `M7` implementation passes a sharper target

Execution planning and roadmap placement for this gap now live in:
- `docs/gui/M7_IMPORTED_BOARD_FIDELITY_PLAN.md`

It is grounded in:
- `specs/M7_FRONTEND_SPEC.md`
- `docs/gui/CANVAS_REVIEW_MODEL.md`
- `docs/gui/VISUAL_LANGUAGE.md`
- `docs/gui/TECHNICAL_PRINCIPLES.md`
- `docs/gui/M7_ROUTE_REVIEW_SCREEN_REDESIGN.md`
- current human-reviewed screenshots of the live `M7` client

## Executive Summary

The current `M7` client is no longer blocked on architecture. The shell,
selection model, engine/frontend boundary, live loading path, and review
workflow all function.

The remaining problem is **board-review fidelity**.

The viewport currently behaves like a **minimal review-scene renderer** built
from simplified authored board primitives plus proposal overlays. It does not
yet behave like a **credible PCB-native review surface**.

That difference explains the main user reactions:
- footprints do not feel real
- the proposed route reads too much like connectivity linework
- the board does not yet encode the visual/material/layer conventions expected
  from a professional PCB viewer

The correction path therefore requires both:
- renderer-level improvement
- richer board review scene data

Renderer polish alone will not be enough.

## What The Current M7 Client Already Proves

The following are working and should not be reopened as product-direction
questions:
- locked `M7` shell and panel structure
- viewport-centered three-column review workspace
- stable picking and selection using contract IDs
- live engine-owned review payload loading
- read-only route-proposal review workflow
- bottom-docked terminal and assistant lanes as supporting surfaces
- real GPU/native frontend architecture viability

The fidelity gap is therefore not evidence that the architecture choice was
wrong. It is evidence that the current scene and render vocabulary are still
too primitive for the intended review quality.

## Correct Language For The Problem

The most useful terms for discussing the current deficiency are:
- **board review fidelity**
- **PCB-native object vocabulary**
- **proxy geometry vs footprint-native geometry**
- **routed-copper grammar**
- **ratsnest-like readability**
- **padstack / pad-shape fidelity**
- **layer appearance model**
- **board-field / material vocabulary**
- **review overlay authority**

These terms are more precise than saying the screen merely "looks wrong."

## User-Visible Deficiencies

### 1. Footprint Fidelity Is Too Low

The current viewport does not render real footprint geometry. It renders:
- component bounds
- simplified pads
- vias
- tracks
- zones

That is enough for a functional spike, but not enough for a credible PCB
review surface.

Practical effect:
- `J1`, `TP1`, and similar components read as proxy arrangements rather than
  known physical footprints
- component grouping and package identity are weak
- the board does not reward domain recognition from an experienced PCB user

### 2. Proposed Track Language Is Too Close To Connectivity Language

Even when the proposal is semantically a real `draw_track` action, it still
reads too much like a net connection or airwire because the visual grammar is
not yet sufficiently PCB-native.

What is missing visually:
- stronger segment/corner authority
- clearer width authority
- clearer entry/exit into the source and target pads
- route overlay that reads like proposed copper, not generic linework

### 3. Board Material / Layer Language Is Too Weak

Professional PCB tools present:
- a board field or substrate
- copper by layer
- pads and vias with distinct roles
- silkscreen / outline / mechanical context
- user-controlled appearance conventions

The current scene has only an early approximation of this. It lacks a proper
layer appearance model and therefore still feels generic.

### 4. Authored / Proposed / Diagnostic Separation Is Still Not Strong Enough

The conceptual split is present in the code and spec, but the viewport still
does not communicate it strongly enough in a board-native way.

Practical effect:
- proposed route authority is weaker than it should be
- evidence/support geometry risks reading like another route
- authored context still does not quite recede correctly under review focus

### 5. DIM UNRELATED Is Not Yet Trustworthy

The UI exposes dimming as active, but the viewport does not yet deliver a
strong enough distinction between:
- directly relevant authored geometry
- indirectly related geometry
- unrelated board context

That weakens confidence in the review model.

## Root Cause Split

The current fidelity gap comes from two different sources.

### A. Renderer-Layer Deficiencies

These can be improved without changing engine meaning:
- proposal overlay styling and endpoint treatment
- actual dimming of unrelated authored geometry
- stronger copper/pad/via/outline treatment
- better layer-aware display roles
- improved panel typography and hierarchy
- better grid / board-field composition

These are real problems and should be fixed.

### B. Scene-Contract Deficiencies

These cannot be solved by styling alone because the frontend is not yet being
given enough board-native information.

Likely missing or too weak for a serious PCB review surface:
- real footprint display primitives beyond coarse component bounds
- better pad/padstack fidelity
- silkscreen/courtyard/fab/assembly geometry where relevant
- richer layer-class information
- more authoritative authored display primitives for board-native categories
- stronger proposal companion geometry if later route actions become more
  complex than the current simple case

This is the more important strategic issue.

## What This Means

The current `M7` client should be described as:
- **a working read-only route-review client with minimal board-context
  rendering**

It should not yet be described as:
- **a credible PCB-native board-review surface**

That does not mean `M7` is failing. It means the opening slice has reached the
point where fidelity quality now depends on whether Datum wants to keep `M7`
minimal or let it grow into a genuinely recognizable PCB review environment.

## Correction Strategy

The right correction path is staged.

### Stage 1: Finish The Immediate Renderer Gaps

These are the next bounded passes that remain inside the existing opening
contract:
- make proposed `draw_track` geometry read as reviewed routed copper
- make `DIM UNRELATED` actually dim unrelated authored geometry
- tighten panel label/value hierarchy further
- refine board grid and spatial reference only as a supporting aid

These improve the current `M7` screen materially and should continue.

### Stage 2: Expand `board_review_scene_v1` For PCB-Native Fidelity

To address the root cause, the board scene contract should grow toward a more
credible review surface. Likely additions:
- richer footprint display primitives
- better package/footprint grouping information
- more detailed pad geometry or padstack categories
- optional silkscreen / courtyard / fab review primitives
- layer-role metadata for appearance control
- clearer authored copper display classes

This does not change routing semantics. It improves the frontend-consumed board
review representation.

### Stage 3: Add A Real Layer Appearance System

Once the scene contract is richer, the GUI should support a true board
appearance model:
- board field / substrate role
- copper role by layer
- pad/via role
- outline / marking / mechanical roles
- theme-driven but user-assignable conventions later

This is the correct way to approach the color complaint. The problem is not
just "wrong colors." It is the lack of a real appearance model.

## Recommended Immediate Next Work Order

### 1. Proposal Path Fidelity Pass

Stay within current `M7` scope and make the proposal read unmistakably as
proposed routed copper.

Target:
- less connectivity-line readability
- stronger segment authority
- stronger endpoint coupling
- clearer reviewed-route identity

### 2. Real Authored Dimming Pass

Make the existing `DIM UNRELATED` control materially affect:
- components
- pads
- tracks
- vias
- zones

This should be strong enough to improve review focus without ghosting the board
out of existence.

### 3. `board_review_scene_v1` Expansion Checkpoint

Before continued visual tuning, explicitly decide whether `M7` should now
upgrade from minimal board-context rendering toward a more credible PCB-native
surface.

If yes, define a bounded contract-expansion slice rather than continuing ad hoc
renderer polish.

## Acceptance Criteria For Closing This Gap

The `M7` route-review workspace becomes credible enough for the opening when:
- the proposed route clearly reads as reviewed route geometry, not generic
  connectivity linework
- related vs unrelated authored board context is visually trustworthy
- components and pads read as intentional PCB object classes rather than just
  boxes
- the board field reads like a board review surface rather than a graphics
  sandbox
- an experienced PCB user can look at the screen and understand object class,
  route status, and review target quickly

The gap remains open if:
- the route still reads like an airwire
- footprints still read only as abstract proxies
- layer/material roles are still visually arbitrary
- the board would not be recognizable as “real enough” to someone familiar
  with the imported design

## Guidance For Future Implementation Prompts

When requesting more `M7` work, separate requests into one of these categories:

- **Renderer pass**
  - improves how existing scene data is drawn
- **Scene-contract pass**
  - expands what board review data the engine/daemon exports
- **Appearance-model pass**
  - defines layer and material display roles

Avoid mixing all three at once unless the work is explicitly intended as a
checkpoint milestone.
