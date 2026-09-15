# Datum Text Engine Outline Geometry Contract Note

> **Status**: Design note
> **Companion ownership note**:
> `docs/gui/DATUM_TEXT_ENGINE_OUTLINE_FILL_OWNERSHIP_NOTE.md`
> **Companion attainment note**:
> `docs/gui/DATUM_TEXT_ENGINE_OUTLINE_FILL_ATTAINMENT_NOTE.md`

## Purpose

Define the engine-side geometry contract that must replace the current
parity-based `outer + holes` assumption for outline-font text.

This note is the bridge from investigation to implementation.

## Problem With The Current Contract

Current outline geometry is modeled as:

- one `outer` ring
- zero or more `holes`

That contract is represented today by:
- [TextPolygon](/home/bfadmin/Documents/datum-eda/crates/engine/src/text/geometry.rs:73)

That structure assumes:
- one dominant exterior
- holes are always inferable as subtractive children

The `Inter` evidence disproves that assumption.

It is not sufficient for:
- additive nested contours
- multiple filled islands
- islands inside voids
- any font family whose intended fill semantics cannot be reduced to simple
  containment parity

## Design Goal

The engine must emit canonical Datum-owned filled geometry such that:
- font-family-specific contour topology is normalized away
- the renderer does not interpret font semantics
- export/preview consumers can share the same resolved geometry

## Recommended Replacement Contract

Datum should introduce a two-stage outline geometry contract.

### Stage A: Raw fill-rule-aware contour set

This is an engine-internal representation, not a renderer target.

Recommended shape:

```rust
pub enum TextFillRule {
    NonZero,
}

pub struct TextContourRing {
    pub points: Vec<Point>,
    pub signed_area_nm2: i128,
}

pub struct TextContourSet {
    pub fill_rule: TextFillRule,
    pub rings: Vec<TextContourRing>,
}
```

Purpose:
- preserve flattened contour topology
- preserve winding information explicitly
- avoid premature reduction to `outer + holes`

This is what the outline backend should produce immediately after
deterministic flattening and transform application.

### Stage B: Canonical resolved filled geometry

This is the renderer/export-facing representation.

Recommended shape:

```rust
pub struct TextFilledRegion {
    pub outer: Vec<Point>,
    pub holes: Vec<Vec<Point>>,
}

pub struct TextResolvedFill {
    pub regions: Vec<TextFilledRegion>,
}
```

Then:

```rust
pub enum TextGeometryPrimitive {
    Stroke(TextStroke),
    ResolvedFill(TextResolvedFill),
}
```

Key difference from the current model:
- one glyph may produce multiple filled regions
- each region may have holes
- region membership is resolved by the engine from the contour set using the
  correct fill rule

This avoids encoding font semantics in the renderer.

## Ownership Split

### Glyph backend owns

- extracting font outlines
- flattening deterministically
- returning raw contour sets with winding preserved

### Engine fill resolver owns

- applying `NonZero` fill semantics
- reducing contour sets into canonical filled regions
- producing final Datum-owned resolved fill geometry

### Renderer owns

- drawing resolved fill geometry only
- no contour interpretation
- no font-family-specific logic

## Why This Contract Is Better

It fixes the current weakness directly:

- `TextPolygon { outer, holes }` is too early a commitment
- `TextContourSet` preserves the truth
- `TextResolvedFill` gives downstream consumers an unambiguous result

So the engine stops guessing too early and the renderer stops guessing at all.

## Required Engine Pipeline

For outline text, the pipeline should become:

1. parse font outline
2. flatten deterministically
3. emit `TextContourSet { fill_rule: NonZero, rings }`
4. resolve contour set into `TextResolvedFill`
5. pass resolved geometry to renderer/export

The current direct path:

flattened contours -> containment parity -> `outer + holes`

must be removed as final truth.

## Migration Plan

### Step 1

Introduce `TextContourSet` and `TextResolvedFill` in the engine text geometry
module without deleting `TextPolygon` yet.

### Step 2

Change the outline backend to emit raw contour sets internally rather than
immediately reducing them to `outer + holes`.

### Step 3

Add an engine resolver:
- input: `TextContourSet`
- output: `TextResolvedFill`

This resolver is where non-zero winding semantics become Datum-owned geometry.

### Step 4

Retarget renderer consumption from the old single-polygon fill model to
resolved fill regions.

### Step 5

Only after parity is proven:
- deprecate/remove the old outline `TextPolygon` path for outline fonts

## Required Tests

Implementation is not complete until these exist:

1. **Topology preservation test**
   - flattened `Inter` / `IBM` / `JetBrains` contours retain winding and ring
     count correctly in `TextContourSet`

2. **Resolved fill equivalence test**
   - engine-resolved filled area matches the intended non-zero-winding area
     across the regression corpus

3. **Native repro test**
   - `text-intent-repro` renders without outline anomalies after the new
     contract lands

4. **No renderer font semantics test**
   - renderer tests only consume resolved fill regions, not raw font contours

## Attainment Condition

The contract is attained when:
- outline backend no longer reduces directly to `outer + holes`
- engine has an explicit non-zero-winding contour resolver
- renderer consumes resolved fill regions only
- the `Inter` anomaly disappears as a consequence of the contract change, not a
  glyph exception
