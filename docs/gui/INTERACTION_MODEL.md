# Interaction Model

> **Status**: Supporting frontend foundation document.
> This document defines the interaction grammar Datum should preserve across
> canvas, terminal, and AI-assisted workflows.

## Purpose

Describe how users should think, point, inspect, command, and ask for help
inside Datum without creating competing interaction systems.

## Selection Model

Selection is Datum's primary interaction primitive.

Selection should:
- use stable object identity from authoritative engine-consumed contracts
- remain meaningful across canvas, panels, terminal references, and AI context
- support both spatial inspection and later action dispatch

Core selection categories:
- graphical objects such as components, pads, tracks, vias, zones, and later
  schematic objects
- review objects such as proposal actions, evidence items, and diagnostics
- document-level scopes such as board, sheet, or later 3D assembly context

Selection states should distinguish:
- `hovered`
- `selected`
- `active review target`
- `related but not selected`

Terminal and AI lanes may consume the current selection context, but they must
do so through explicit stable identifiers rather than hidden heuristics.

For `M7` v1, single selection with a separate active review target is now
locked.

## Hover And Focus Rules

Hover is transient and lightweight.
- it previews possible interaction
- it may highlight geometry or related evidence
- it must never silently change authoritative context

Focus determines where keyboard input goes.
- canvas focus enables viewport and tool commands
- terminal focus enables command entry and log navigation
- AI lane focus enables conversation input and response navigation

Changing focus must not implicitly change design selection unless the user
explicitly triggers a selection action.

For `M7` v1:
- opening or focusing the terminal lane must not clear or replace selection
- opening or focusing the AI lane must not clear or replace selection
- the active review target may persist while the user inspects related authored
  geometry

## Inspection Model

Inspection should happen primarily through:
- persistent inspector/properties panels
- review/evidence panels
- contextual terminal output
- contextual AI explanation

Routine inspection should not depend on modal dialogs.

Inspection rules:
- one selected object or review target must always be inspectable
- panel and canvas inspection should cross-highlight one another
- terminal and AI explanations should reference the same underlying identities
  and contracts

## Command And Search Model

Datum should expose a first-class command/search surface that complements the
canvas instead of bypassing it.

Expected command/search functions:
- command palette
- object search and jump
- action search
- project/document navigation
- later net/component/rule search

The integrated terminal lane is distinct from the command palette:
- command palette is a UI-native action discovery and dispatch surface
- terminal lane is direct deterministic CLI-style workflow execution and log
  visibility

Both should operate on the same underlying engine actions and identities.

## Keyboard And Mouse Principles

Datum should optimize for expert long-session use.

Keyboard expectations:
- stable shortcuts for viewport control, search, review navigation, and lane
  switching
- explicit focus changes between canvas, terminal, and AI lanes
- action dispatch that remains legible rather than hidden behind obscure modes

Pointer expectations:
- low-latency pan and zoom
- precise pointing and picking
- explicit click-to-select behavior
- predictable modifier semantics

Neither input path should feel secondary. Datum should support fluent
mouse-driven inspection and fluent keyboard-driven workflow.

## CAD Interaction Expectations

Datum should behave like a serious CAD/EDA environment:
- the viewport is first-class
- pointer-centered navigation is smooth and reliable
- selection and highlighting are semantically meaningful
- repeated expert actions should not require dialog interruption

Later edit tools may be modal, but Datum should avoid unnecessary hidden
submodes. Current mode and click meaning must remain clear.

## Role Of The Terminal Lane

The terminal lane is part of the interaction grammar, not a bolt-on console.

Its role is:
- direct deterministic workflows
- transparent logs
- reproducible command output
- workflow surfaces that map cleanly to engine and CLI contracts

It is not:
- a private mutation path
- a place where undocumented state exists
- an excuse to skip proper UI interaction design for core review flows

For `M7` v1, the terminal lane is locked as a bottom-docked read-only/supporting
lane.

Terminal commands should be able to reference:
- current project
- explicit selected identities when applicable
- explicit artifact paths or request arguments

## Role Of The AI Assistant Lane

The AI lane is a guided interaction surface for:
- explanation
- review support
- workflow discovery
- contextual assistance

It should be able to consume:
- current project context
- current selection or review target
- current deterministic engine reports and review payloads

It should not:
- fabricate design truth
- issue hidden side effects
- hold private state that disagrees with authoritative engine state

For `M7` v1, the AI lane is locked as a bottom-docked explanation/review-
support lane with no mutation authority.

Any action proposed by the AI lane must resolve to explicit engine-backed
requests that the rest of the application can understand.

## Unified Interaction Grammar

Datum's interaction grammar should make one thing consistent across all lanes:
- object identity
- review context
- action authority
- result visibility

The user should never need to wonder whether the canvas, terminal, and AI lane
are talking about different versions of the design.
