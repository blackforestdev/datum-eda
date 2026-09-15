# Canvas Review Model

> **Status**: Supporting frontend foundation document.
> This document defines the principles for graphical review surfaces, starting
> with board review and extending toward future schematic and 3D coexistence.

## Purpose

Define how Datum should present authored design state, proposal state, and
review evidence inside graphical viewports.

## Board Review Canvas Principles

The board review canvas should:
- present authoritative authored board state as the baseline
- make proposed and diagnostic state visibly distinct from authored state
- support precise selection, hover, and highlighting
- preserve spatial context while the user inspects evidence
- keep pan/zoom performance and clarity high enough for long sessions

The board review canvas is not merely a picture of the board. It is a spatial
review surface tied to stable object identity and deterministic engine output.

## Authored / Proposed / Diagnostic Model

Datum should preserve a clear visual and conceptual split between:

### Authored State

The persisted board state currently accepted by the engine.

This is the spatial baseline for review.

### Proposed State

Machine-produced or otherwise reviewable candidate geometry or actions that
have not been committed into authored state.

This must appear as an overlay or explicitly differentiated layer, never as
ambiguous board geometry.

### Diagnostic / Evidence State

Explanatory markers, evidence rows, supporting highlights, or diagnostic
structures that help the user understand a current review target.

This state supports interpretation. It is not itself design truth.

## Overlay And Highlight Model

Overlays should be first-class render layers, not ad hoc annotations.

Expected overlay categories:
- selection emphasis
- hover emphasis
- review target emphasis
- proposal geometry
- evidence markers
- filter/mask state

Highlighting should support:
- direct emphasis of the current target
- dimming of unrelated geometry
- relationship highlighting between authored and proposed objects
- synchronized emphasis between panel rows and canvas geometry

Datum should avoid using a single generic highlight treatment for every state.
Authored, proposed, selected, and evidence-linked states must remain distinct.

For `M7` v1, authored/proposed/diagnostic separation is locked and must remain
legible in dense PCB scenes.

## Evidence And Explain Visualization

Evidence should be shown in ways that attach clearly to spatial context.

Rules:
- evidence must link back to explicit geometry or review objects when possible
- the user must be able to move between evidence panels and canvas focus
- explanation text should not be the only way to understand a review target
- the canvas should make the focal object or path obvious before the user reads
  long textual explanation

The AI lane may explain evidence, but the graphical review lane must remain
self-sufficient enough that the user can still inspect what matters visually.

## Schematic Review Direction

Schematic review should eventually follow the same principles as board review:
- object identity first
- baseline authored state clearly presented
- overlays for proposals or diagnostics
- explicit review focus
- synchronized inspection across panels and other lanes

This means Datum should not design board review with assumptions that would be
awkward for future schematic review.

## 2D / 3D Coexistence Direction

Future 3D review should be treated as a coordinated view of the same design
state, not as a separate product.

Foundational expectations:
- shared object identity between 2D and 3D
- shared selection and visibility concepts
- shared distinction between authored, proposed, and diagnostic states
- synchronized review target where appropriate

Datum should avoid a path where 3D becomes an isolated viewer with unrelated
interaction and state rules.

## Terminal And AI Relationship To Canvas Review

The canvas remains the primary spatial review lane.

The terminal lane supports it by:
- exposing deterministic review workflows and logs
- surfacing exact command/report output

The AI lane supports it by:
- explaining selected review targets
- summarizing current evidence
- helping the user navigate available deterministic actions

Neither lane replaces the need for a clear graphical review surface.

For `M7` v1, both lanes are supporting only:
- the terminal lane remains read-only/supporting
- the AI lane remains explanation/review-support only
