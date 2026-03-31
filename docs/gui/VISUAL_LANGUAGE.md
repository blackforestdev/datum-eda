# Visual Language

> **Status**: Supporting frontend foundation document.
> This document defines Datum's visual direction for a serious Linux-native EDA
> application.

## Purpose

Set visual design expectations early enough that frontend work does not drift
into generic app styling or legacy EDA clutter.

## Tone

Datum should feel:
- precise
- calm
- technical
- modern without consumer-app affectation
- dense without looking cramped
- serious without looking stale

The product should not chase novelty styling at the expense of clarity.

## Visual Hierarchy

Visual hierarchy should be driven by:
- contrast
- grouping
- spacing
- edge treatment
- restrained accent color

The graphical viewport should carry the primary visual weight.
Panels, terminal, and AI lanes should support that focus without visually
competing for dominance.

## Typography

Typography should optimize for:
- technical readability at compact sizes
- stable rhythm in dense panels and lists
- clear separation between labels, values, identifiers, and status text

Direction:
- one primary UI type family
- one monospace family for IDs, coordinates, paths, and command/report content
- compact but readable size and line-height choices

Datum should avoid oversized generic desktop typography and avoid decorative
type choices that reduce scanability.

## Spacing And Grid

Datum should use:
- a compact baseline spacing system
- a small set of spacing tokens
- consistent panel rhythm
- enough whitespace for scanability without creating empty chrome

The target density should be closer to professional CAD/graphics tools than to
consumer productivity software.

## Color Philosophy

Color should communicate:
- authored baseline state
- proposed overlay state
- review focus and selection
- severity and status
- layer or domain identity where appropriate

Color should not be used primarily for decoration.

Datum should avoid:
- rainbow noise
- low-contrast trendy palettes
- over-saturated accents
- ambiguous reuse of the same color for unrelated semantic states

For `M7` v1, the following are locked:
- authored, proposed, and diagnostic states must use distinct visual semantics
- proposed state must not rely on color alone for differentiation

The following remain open until after the architecture spike:
- exact dark-first versus equal dual-theme launch commitment
- exact proposal accent palette
- final diagnostic palette

## Iconography

Icons should be:
- simple
- geometric
- sparse
- secondary to clear text where ambiguity exists

Datum should not depend on icon-only discoverability for important functions.

## Motion

Motion should be purposeful and restrained.

Appropriate uses:
- viewport continuity
- highlight transitions
- panel reveal/collapse
- explicit state confirmation

Datum should avoid ornamental or attention-seeking animation.

## Density And Readability Targets

Datum should support long expert sessions on Linux workstations and laptops.

Implications:
- compact controls
- readable small text
- no large dead regions of chrome
- no tiny, illegible metadata fields
- panel content that remains scannable under dense information load

The following remain open until after the architecture spike:
- finer-grained typography sizes and spacing tokens
- the full amount of review-list density and subsection collapsing behavior

## Linux Desktop Expectations

Datum should feel native and competent on Linux.

Expectations:
- correct Wayland/X11 behavior
- good font rendering
- strong HiDPI handling
- sane keyboard behavior
- professional window chrome and focus behavior

Datum should not depend on fragile theme behavior or platform tricks to look
acceptable.

## Lane Presentation

Canvas, terminal, and AI lanes should be visually related but distinct.

Guidelines:
- the canvas lane gets the strongest spatial focus
- the terminal lane should read as deterministic workflow/log space
- the AI lane should read as contextual assistance, not as a chat app pasted
  into the shell

All lanes should still feel like parts of one application, sharing typography,
spacing rhythm, and semantic color usage.
