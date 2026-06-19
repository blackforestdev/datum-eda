# Datum Text Engine Outline Fill Attainment Note

> **Status**: Investigation note
> **Companion ownership note**:
> `docs/gui/DATUM_TEXT_ENGINE_OUTLINE_FILL_OWNERSHIP_NOTE.md`
> **Companion contract note**:
> `docs/gui/DATUM_TEXT_ENGINE_OUTLINE_GEOMETRY_CONTRACT_NOTE.md`
> **Primary research input**:
> `research/pcb-text-rendering/DATUM_TEXT_ENGINE_PHASE_2_RESEARCH.md`

## Purpose

Describe what Datum must attain to resolve the current outline-font anomaly
correctly at the engine layer, without falling back into symptom-driven glyph
patching.

This note is intentionally about requirements and invariants, not a code fix.

## Current Failure Model

The current failure is now bounded:

- affected lines use `InterVariable.ttf`
- unaffected outline lines use different families
- affected glyphs (`A`, `D`, `R`, later `Q`) show topology that does not fit
  Datum's current parity-based interpretation

Current Datum behavior:
- engine reduces contours to `outer + holes` by containment depth
- renderer fills those rings using even-odd span pairing

Current research-backed contract:
- OpenType outlines expect **non-zero winding** semantics

So the bug is not "bad letters." It is:

**Datum's current canonical outline geometry contract is too weak to preserve
font fill semantics across supported font families.**

## Key Evidence

Raw contour inspection showed:

For `Inter`:
- `A`, `D`, and `R` contain same-winding interior/additive contour structure

For `IBM Plex Sans Condensed` and `JetBrains Mono`:
- analogous glyphs follow the classic opposite-winding outer/counter pattern

That means Datum cannot keep assuming:
- nested contour => hole
- parity grouping is sufficient
- `TextPolygon { outer, holes }` is enough to represent all supported outline
  families before semantic resolution

## What Datum Must Attain

### 1. Engine-owned fill-rule resolution

The engine must become the owner of font fill interpretation.

Required result:
- the renderer no longer needs to infer font semantics
- source font topology differences are normalized before render

### 2. A canonical geometry contract stronger than `outer + holes`

Current contract:
- [TextPolygon](/home/bfadmin/Documents/datum-eda/crates/engine/src/text/geometry.rs:73)
  stores only:
  - one `outer`
  - zero or more `holes`

That is not strong enough for the observed font topologies.

Datum must attain one of these engine-side contracts:

1. **Fill-rule-aware contour object**
   - preserves flattened rings
   - carries explicit fill rule (`NonZero`)
   - later resolves to final geometry in the engine

2. **Already-resolved canonical filled geometry**
   - multiple filled islands allowed
   - holes allowed
   - no ambiguity left for the renderer

Preferred direction:
- canonical filled geometry resolved in the engine

### 3. A family-agnostic invariant

The repair must be proven against an invariant that holds across families.

Required invariant:
- Datum's interpreted filled area for a glyph must match the font-intended
  filled area under the correct fill rule

This must be checked across a corpus that includes:
- `Inter`
- `IBM Plex Sans Condensed`
- `JetBrains Mono`

And glyphs covering:
- additive interiors
- classic counters
- islands inside holes

Minimum regression set:
- `A`
- `D`
- `R`
- `Q`
- `O`

### 4. Separation between flattening and fill semantics

Datum must stop conflating:
- flattening contours
- interpreting contours

These are separate phases.

Required pipeline:
1. extract raw glyph outline
2. flatten deterministically
3. preserve contour orientation/topology
4. resolve fill semantics in engine
5. emit canonical Datum geometry

### 5. Explicit non-zero-winding support

Datum must be able to evaluate glyph fill using non-zero winding semantics.

That support may be realized by:
- an internal contour interpreter
- engine-side boolean preparation
- tessellation/input prep using the Phase 2 library stack

But the system must no longer depend on parity/depth heuristics as final truth.

## What Must Be Investigated Before Implementation

Before code changes resume, Datum needs a written answer for each:

1. What exact canonical engine geometry type will replace or extend
   `TextPolygon { outer, holes }`?
   See:
   `docs/gui/DATUM_TEXT_ENGINE_OUTLINE_GEOMETRY_CONTRACT_NOTE.md`

2. Where will non-zero-winding resolution happen?
   - before tessellation
   - during boolean prep
   - during canonical geometry construction

3. What test proves the repair at the class level?

4. What export/preview consumers need the new geometry contract?

5. Does the renderer need any change at all once the engine emits resolved
   filled geometry?

## Success Criteria

We have attained the goal when all of the following are true:

- `Inter` no longer misrenders on the native text-intent repro
- `IBM Plex Sans Condensed` and `JetBrains Mono` still render correctly
- no per-glyph or per-family exceptions are needed
- renderer does not contain font-family-specific fill logic
- engine tests prove class-level correctness against the chosen fill invariant
- the fix can be explained as one contract change, not a collection of
  localized repairs
