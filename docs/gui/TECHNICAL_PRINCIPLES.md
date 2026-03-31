# Technical Principles

> **Status**: Supporting frontend foundation document.
> This document defines ownership boundaries, state philosophy, rendering
> expectations, and explicit anti-goals for Datum's frontend.

## Purpose

Keep frontend implementation choices aligned with the engine-first product
model and prevent the GUI from accumulating unofficial semantics.

## Engine / Daemon / Frontend Boundary

The engine and daemon own:
- design truth
- deterministic query and review results
- operation validation and execution
- persisted project state
- semantic meaning of reports, proposals, diagnostics, and artifacts

The frontend owns:
- workspace and panel layout state
- camera and viewport state
- ephemeral hover/focus/selection state
- derived view-models and render scenes
- presentation logic for canvas, terminal, and AI lanes

The frontend must not own:
- alternate design truth
- hidden semantic state that can drift from engine outputs
- private geometry inference that changes review meaning
- unofficial write paths that bypass engine actions

## State And View-Model Philosophy

Frontend state should be layered:
- transport/contracts from engine or daemon
- derived view-models
- derived render scene
- ephemeral UI state

Terminal and AI lanes should consume the same authoritative context graph:
- current project identity
- current document/view identity
- current selected object identities
- current review target identities
- current deterministic engine outputs

They may cache or summarize for responsiveness, but they must not become
parallel authorities.

## Rendering Principles

Rendering is product infrastructure, not a disposable shell detail.

Principles:
- render layers should map to semantic layers
- picking should use stable identities
- authored, proposed, and diagnostic states must remain visually separable
- scene derivation should be explicit and testable
- future 2D and 3D views should share enough identity/state infrastructure to
  coexist cleanly

## Determinism And Testability

The frontend should be built as a deterministic consumer wherever practical.

Expectations:
- versioned frontend-consumed contracts
- fixture-backed view-model and scene derivation tests
- stable ordering and identity where contracts promise it
- snapshot or image-based render checks where appropriate
- lane-specific tests for context synchronization and action visibility

The integrated terminal lane should preserve deterministic command and log
visibility. The AI lane should expose which authoritative context and engine
results it is using when that matters to the user.

## Integrated Terminal Lane: Technical Role

The terminal lane is a product surface for deterministic workflows.

Technical expectations:
- it invokes documented engine/CLI/daemon-backed actions only
- it surfaces stdout/stderr/logs/results inside the application shell
- it can consume current explicit selection or review context when supported
- its actions must remain inspectable by the rest of the application

Anti-goal:
- no undocumented backchannel mutations or hidden shell-only semantics

For `M7` v1, this lane is locked to a bottom-docked read-only/supporting role.

## Integrated AI Assistant Lane: Technical Role

The AI lane is a contextual assistance surface, not a design database.

Technical expectations:
- it consumes explicit project, selection, review, and report context
- it explains deterministic engine outputs rather than replacing them
- any mutating action it proposes must resolve to explicit engine-backed
  requests visible to the rest of the application
- its conversational state must not become authoritative project state

Anti-goal:
- no hidden side effects or private model-only truth

For `M7` v1, this lane is locked to a bottom-docked explanation/review-support
role with no mutation authority.

## Explicit Anti-Goals

Datum should explicitly avoid:
- GUI-owned semantic models that compete with engine truth
- modal-dialog dependence for routine inspection and review
- generic desktop-app architecture that sidelines the viewport
- panel proliferation without a stable taxonomy
- terminal or AI lanes acting as shadow authoring systems
- architecture choices that optimize only for a narrow first viewer
- visual or interaction systems that would make future 3D coordination awkward

## Foundation Implication For `M7`

Any `M7` opening spec should prove:
- the GUI is a disciplined consumer of authoritative engine contracts
- canvas, terminal, and AI lanes can coexist without competing truth models
- a narrow review slice can establish the right shell and state boundaries for
  later growth

`M7` should not attempt to solve the whole frontend, but it must respect these
technical boundaries from the start.
