# Product Mechanics 036: Schematic Drawing Themes

Status: ratified doctrine

## Context

The locked Rendering Book defined a dark schematic working canvas and a warm
paper alternative, but treated the latter as a session-only documentation
toggle. The Preferences prototype surfaced the resulting authority conflict:
the drawing-space appearance could not be both a deliberate user choice and
forgotten at every launch.

The owner reviewed Candidate B in
`docs/gui/prototypes/canvas-background-decision.html` and selected it on
2026-08-27, reframed as selection among complete governed drawing themes rather
than editing a background color.

## Decision

Schematic drawing-space appearance is a machine-scoped Presentation preference
named **Schematic drawing theme**. It persists across sessions and is edited in
the Preferences pane.

A drawing theme is one indivisible, contrast-verified system covering the
canvas ground, grid, drawing ink, symbol bodies and strokes, wires, references,
selection treatment, and other schematic foreground roles. Users select a
governed theme as a whole; they cannot edit its individual palette members.

The initial themes are:

- **Dark** — the locked factory default; and
- **Light** — the locked warm-paper theme with ground `#E7E1D2`, deliberately
  not white.

“Vellum” is retired as the user-facing and doctrine name for the Light theme.
Existing locked palette values remain unchanged. Application chrome stays dark
under both themes.

The authoring theme is screen-only Presentation state. It is not authored
design data, a Publish-space surface, a standards-control fact, a journaled
operation, or an input to plotted/exported appearance. Publish templates and
render intent continue to govern print and document output independently.

## Scope boundary

This decision applies to schematic drawing themes. The board retains its
separate **Layer color scheme** preference and existing palette behavior. A
complete Light board theme is future governed work and is not implied by this
decision.

## Required Global Preferences contract

The Global Preferences specification must define the stable descriptor,
persistence, provenance, reset-to-Dark behavior, migration from any legacy
session toggle, and accessible theme preview/selection treatment. The setting
belongs to machine Presentation scope and is outside Project and organization
drafting-standard authority.

## Non-decisions

This decision does not authorize implementation, add a dependency, change
board colors, alter Publish/print behavior, allow free palette editing, resume
GP-C03 Q5-Q10 generally, or resolve a future Light board theme.

## Owner evidence

<!-- EVIDENCE:SCHEMATIC-DRAWING-THEMES:OWNER-APPROVED -->

On 2026-08-27 the owner selected Candidate B with the explicit whole-theme,
machine-Presentation, schematic-only, dark-default, Light-naming, dark-chrome,
and print-independence boundaries recorded above.

## Dependency and licensing impact

None. This decision grants no dependency or license exception under Product
Mechanics 029.
