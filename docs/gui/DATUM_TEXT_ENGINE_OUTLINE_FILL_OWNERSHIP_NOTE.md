# Datum Text Engine Outline Fill Ownership Note

> **Status**: Investigation note
> **Primary evidence**:
> `tmp/text-intent-repro/board/board.json`
> **Primary research input**:
> `research/pcb-text-rendering/DATUM_TEXT_ENGINE_PHASE_2_RESEARCH.md`
> **Companion brief**:
> `docs/gui/DATUM_TEXT_ENGINE_PHASE_2_BRIEF.md`
> **Companion plan**:
> `docs/gui/DATUM_TEXT_ENGINE_PHASE_2_IMPLEMENTATION_PLAN.md`
> **Attainment note**:
> `docs/gui/DATUM_TEXT_ENGINE_OUTLINE_FILL_ATTAINMENT_NOTE.md`

## Purpose

Record the current outline-font rendering anomaly, assign ownership of the
problem to the correct subsystem, and define the engine-side contract that
must be implemented before more fixes are attempted.

This note exists specifically to prevent symptom-driven letter patches.

## Observed Failure Pattern

On the native repro board:
- line 1 (`MFG DEFAULT`) is unaffected
- line 2 (`ANNOTATION DEFAULT`) is affected
- line 3 (`BRANDING DEFAULT`) is affected
- line 4 (`DOCUMENTATION DEFAULT`) is unaffected
- line 5 (`MONO OVERRIDE v1.2.3`) is unaffected

Within the affected lines, only some glyphs misrendered:
- `A`
- `D`
- `R`
- later analysis also implicated `Q`

This immediately ruled out:
- KiCad import
- source `render_cache`
- generic text-engine orientation bugs

The bug class is specific to Datum's outline-font interpretation.

## Confirmed Family Split

The repro fixture binds:
- `Manufacturing` -> `newstroke`
- `Annotation` -> `inter`
- `Branding` -> `inter_display`
- `Documentation` -> `ibm_plex_sans_condensed`
- explicit mono override -> `jetbrains_mono`

Current product fact:
- `inter_display` still resolves to the same `InterVariable.ttf` asset as
  `inter`

Therefore the affected lines are both using the same underlying font asset,
while the unaffected outline-font lines are using different font families.

## Confirmed Topology Difference

Raw contour inspection of vendored fonts showed:

For `Inter`:
- `A` has two contours with the same winding sign
- `D` has two contours with the same winding sign
- `R` has two contours with the same winding sign

For `IBM Plex Sans Condensed` and `JetBrains Mono`:
- inner contours for these glyphs flip winding sign relative to the outer
  contour in the classic outer-plus-counter pattern

This means Datum is not dealing with a generic "inner contour means hole"
world. At least one supported font family uses additive nested contours in a
way Datum's current interpretation does not normalize correctly.

## Current Datum Contract

Today Datum applies two simplifying assumptions:

1. In the engine:
   `crates/engine/src/text/outline.rs`
   `group_contours_into_polygons(...)`
   classifies contours by containment depth:
   - even depth => outer
   - odd depth => hole

2. In the renderer:
   `crates/gui-render/src/lib.rs`
   `push_*_polygon_fill_scanline_contours(...)`
   fills by even-odd span pairing

These assumptions happen to work for some families and fail for others.

## Research Contract

Per research:
- OpenType's standard rule is **non-zero winding**
- even-odd is a different fill rule with different semantics

Therefore Datum currently has a contract mismatch:
- font data expects non-zero winding interpretation
- Datum currently normalizes/consumes it as containment parity plus even-odd

## Ownership Decision

The anomaly must be owned by the **engine**, not the renderer.

Reasons:
- renderer-side font interpretation would spread font semantics too late in
  the pipeline
- preview/export/future consumers would all need to re-implement the same
  logic
- Datum's product rule is that renderers consume Datum-owned geometry, not
  source-tool or font-format semantics directly

So the correct ownership boundary is:
- glyph backend provides raw font contours
- engine resolves fill semantics
- engine emits canonical Datum-owned filled geometry
- renderer fills already-resolved geometry

This ownership choice also aligns with the product doctrine:
- manufacturing text is not allowed to be lower quality by default
- any backend choice or geometry resolution must preserve high visual fidelity
- fabrication constraints are handled by validation/policy, not renderer-side
  or backend-side uglification

## What Must Not Happen

Not acceptable:
- per-letter fixes
- per-family exceptions
- renderer-only fill-rule hacks
- continued reliance on containment-depth parity as final outline truth

These are symptom patches and make later root-cause work harder.

## Required Engine Outcome

The engine must produce canonical filled outline geometry such that:
- supported font-family topology differences are normalized away before render
- the renderer does not need font-specific fill logic
- the same geometry contract can be reused by preview, export, and later DRC

That means Phase 2 outline work should converge on:
- raw contour intake from the font backend
- engine-side non-zero-winding-aware fill resolution
- canonical filled polygons/islands/holes or equivalent unambiguous geometry

## Immediate Next Investigation Rule

Before any new fix is applied:
- evaluate affected glyphs under Datum's current parity/even-odd model
- evaluate the same glyphs under the intended non-zero-winding model
- define the exact canonical engine geometry contract needed to bridge the two

Only after that comparison is written down should implementation resume.
