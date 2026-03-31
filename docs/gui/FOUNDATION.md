# Frontend Foundation

> **Status**: Non-normative product and architecture foundation.
> This document is the governing overview for Datum's frontend direction.
> Supporting details live in the other `docs/gui/*.md` documents. Formal
> implementation contracts remain in `specs/`.

## Purpose

Define what Datum's GUI is trying to become before `M7` implementation work
hardens shell, interaction, or rendering decisions.

This document exists to prevent three failure modes:
- treating the GUI as a generic desktop shell around a drawing widget
- allowing a narrow first viewer to define the long-term product
- letting terminal or AI surfaces become parallel design authorities

This foundation follows:
- [REFERENCE_STUDY.md](/home/bfadmin/Documents/datum-eda/docs/gui/REFERENCE_STUDY.md)
- [FRONTEND_DECISION_CRITERIA.md](/home/bfadmin/Documents/datum-eda/docs/gui/FRONTEND_DECISION_CRITERIA.md)

## What Datum's GUI Is Trying To Be

Datum's GUI is a **professional native graphical client for an engine-first EDA
system**.

It should become:
- a serious long-session environment for schematic, PCB, and later 3D review
  and editing
- a custom graphical application with first-class viewport behavior
- a high-density, high-clarity expert interface rather than a low-density
  generic desktop app
- one product with a coherent interaction grammar across canvas, command, and
  AI-assisted workflows
- a consumer of authoritative engine state, deterministic reports, and
  explicit operations

## What Datum's GUI Is Not Trying To Be

Datum's GUI is not:
- a web-style interface translated into a native package
- a dialog-driven legacy CAD clone
- a simplified hobbyist environment optimized around casual use first
- a semantic peer to the engine
- a shell where terminal output or AI chat can invent or persist unofficial
  design state
- a product where the canvas is secondary to menus, forms, or chrome

## First-Class Workspace Lanes

Datum reserves architectural space for three first-class lanes inside one
application:

1. **Graphical canvas/review lane**
   - board, schematic, and later 3D viewports
   - spatial inspection, highlighting, review, and future direct editing

2. **Integrated command/terminal lane**
   - direct CLI-style workflows
   - logs, deterministic command output, artifact inspection, and scripted
     actions

3. **Integrated AI assistant lane**
   - design assistance
   - explanation of engine reports and review evidence
   - guided action discovery and review support

All three lanes are first-class user experiences. Only the engine and its
explicit machine surfaces remain design authority.

## Core Product Principles

1. **Engine authority is non-negotiable.**
   The engine/daemon remain the source of truth for design semantics,
   operations, review payloads, and persisted state.

2. **The canvas is for spatial reasoning.**
   The viewport is the center of the graphical lane. Panels, terminal, and AI
   support it; they do not replace it.

3. **Selection is the root of interaction.**
   Every lane should be able to consume current review context, but only
   through explicit stable identities and deterministic contracts.

4. **Review and editing are different postures.**
   Datum should distinguish inspection/review from mutation in both behavior
   and visual language.

5. **Professional density with readable hierarchy.**
   Datum should be information-dense enough for expert work without inheriting
   legacy EDA clutter.

6. **One interaction grammar across lanes.**
   Canvas, command, and AI lanes may differ in form, but they must agree on
   object identity, action authority, and review context.

7. **2D and 3D are coordinated views, not separate products.**
   Future 3D work must reuse object identity, selection, and visibility
   concepts established in 2D review.

8. **AI and terminal surfaces are action brokers, not hidden editors.**
   They may inspect, explain, and request explicit engine actions, but they
   must not become private mutation channels outside the engine contract model.

9. **The opening `M7` slice must stay narrow without dead-ending the shell.**
   Early GUI decisions must support long-term product coherence even when the
   first implementation slice remains read-only.

## Locked `M7` Decisions

The following `M7` opening decisions are now treated as locked direction:
- viewport-centered three-column shell with a bottom dock strip
- fixed `M7` panel taxonomy: `Project`, `Filters`, `Inspector`, `Review`
- route review starts from the first proposal action
- single-selection model with a separate active review target
- explicit authored/proposed/diagnostic visual-state separation
- integrated bottom-docked terminal lane with read-only/supporting role
- integrated bottom-docked AI assistant lane with explanation/review-support
  role and no mutation authority

These decisions should be implemented unless the architect explicitly reopens
them.

## Governing Relationships

This document sets the top-level direction for:
- [WORKSPACE_MODEL.md](/home/bfadmin/Documents/datum-eda/docs/gui/WORKSPACE_MODEL.md)
- [INTERACTION_MODEL.md](/home/bfadmin/Documents/datum-eda/docs/gui/INTERACTION_MODEL.md)
- [CANVAS_REVIEW_MODEL.md](/home/bfadmin/Documents/datum-eda/docs/gui/CANVAS_REVIEW_MODEL.md)
- [VISUAL_LANGUAGE.md](/home/bfadmin/Documents/datum-eda/docs/gui/VISUAL_LANGUAGE.md)
- [TECHNICAL_PRINCIPLES.md](/home/bfadmin/Documents/datum-eda/docs/gui/TECHNICAL_PRINCIPLES.md)

Future `M7` specs should follow this document rather than redefining frontend
purpose from scratch.
