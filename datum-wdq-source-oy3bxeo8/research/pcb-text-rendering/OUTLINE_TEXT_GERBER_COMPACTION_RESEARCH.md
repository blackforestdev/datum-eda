# Compact High-Fidelity Outline Text → Gerber — Research

> Continuation of `research/pcb-text-rendering/PCB_TEXT_RENDERING_RESEARCH.md` and
> the Phase 2/3 text-engine research. Closes the **arc/biarc deferral** those docs
> explicitly left open ("skip FontoBene for M7; revisit in M8").
> **Doctrine (controlling, do not violate):** high-fidelity text across all intents;
> fabrication concerns are handled by **validation/policy or the export path**,
> **never** by coarsening the renderer's outline flattening (`crates/engine/src/
> text/outline.rs:270`). This report obeys that doctrine — the renderer is untouched;
> only the Gerber export encoding changes.

## 1. Root cause of the gerber blow-up (it is NOT the flatten tolerance)

The IBM Plex silk-wiring surfaced a ~190 KB gerber for four characters. Investigation
(agent-assisted, 2026-07-08) found the cause is the **export encoding**, not the
(correctly fine) flatten tolerance:

- `outline.rs` flattens each glyph's Béziers to a fine polyline (de Casteljau midpoint
  subdivision, tolerance ~0.001·height — doctrine-correct, keep it).
- `resolve_contour_set_non_zero_scanline` (`outline.rs:309`) then decomposes the filled
  glyph into **one 4-point trapezoid per horizontal Y-band**, and
  `geometry.rs::polygon_ring_to_strokes` **re-strokes the perimeter of every trapezoid**.
- The Gerber writer (`export/silkscreen.rs`, `export/export_surface.rs`,
  `export/formatting.rs`) emits those as `D02`/`D01` draws with **circular apertures
  only — no `G36/G37` regions, no `G75/G02/G03` arcs**, and raw 7–9-digit-nanometer
  absolute coordinates.

So a curved glyph becomes *dozens of Y-bands × 4 strokes each* in the most verbose
possible encoding. This is a **pathological interim** — worse than the repo's own
research plan, which already prescribes `G36/G37` filled polygons derived at export via
`i_overlay` boolean union, but left the arc step deferred.

## 2. The solved approaches (with citations)

### A. Gerber native region + arc primitives — the target encoding
The **Ucamco *Gerber Layer Format Specification*** (rev. 2021-11 / current 2024.05) is
authoritative: **§4.10 Region Statement (`G36…G37`)** ("Valid Contours") and **§4.7.2
Circular plotting (`G75` multi-quadrant, `G02` CW / `G03` CCW)**.
- A region is a filled area bounded by a **closed, non-self-intersecting contour** of
  connected `D01` segments; multiple contours per `G36…G37` block.
- **Arcs are legal inside regions:** emit `G75*`, then `G03`/`G02` with `I`/`J` center
  offset. A curved glyph edge collapses from hundreds of `D01` lines to one arc `D01`.
- **Holes/counters** (in 'O','A','e','B','R'): nested contours, or — more fab-robust —
  outer region in dark polarity `%LPD*%` + hole as clear polarity `%LPC*%`.
- **Production proof:** KiCad's `GERBER_PLOTTER` already emits pads/zones as
  `G36`→region and has `plotArc` emitting `G75` arcs inside regions ("plot round
  rectangle pads using a region with arcs"). Arc-in-region is shipping and fab-accepted.
- Spec: <https://www.ucamco.com/en/gerber>. KiCad: <https://docs.kicad.org/doxygen/classGERBER__PLOTTER.html>.

### B. Curve → circular-arc (biarc) approximation — the compaction algorithm
To emit arcs, convert each quadratic/cubic Bézier to a short chain of circular arcs via
**biarc fitting** (two arcs sharing a common tangent at the join).
- Canonical: **W. Bolton, "Biarc curves," *Computer-Aided Design* 7(2):89–92, 1975**
  (the seminal reference — the "Kurtenbach & Buxton" of curve→arc). Meek & Walton
  arc-spline papers (1990s–2000s); Riškus & Liutkus. Implementable write-up:
  dlacko (2016) <https://dlacko.org/blog/2016/10/19/approximating-bezier-curves-by-biarcs/>.
- Algorithm: split at inflections (convex pieces) → biarc join at the incenter →
  centers by endpoint-tangent perpendiculars → recursively subdivide at max-deviation
  until within tolerance. A glyph edge typically needs a handful of arcs.
- Result: near-exact (tolerance-bounded, arbitrarily tight at negligible byte cost) at
  a tiny primitive count — the literal "compact-yet-high-fidelity" answer.

### C. GPU / resolution-independent vector text — the on-SCREEN path (optional upgrade)
For the renderer (not export), render glyphs directly from Béziers, no fixed flattening:
- **Loop & Blinn, "Resolution Independent Curve Rendering using Programmable Graphics
  Hardware," SIGGRAPH 2005 / ACM TOG 24(3)** — seminal. <https://dl.acm.org/doi/10.1145/1073204.1073303>.
- **Eric Lengyel, "Slug,"** JCGT 2017 — GPU rendering from glyph outlines, atlas-free,
  correct AA under magnification and minification; **patent dedicated to the public
  domain March 2026**, reference shaders MIT. <https://sluglibrary.com/>.
- **Green (Valve), SDF text, SIGGRAPH 2007** (softens corners); **Chlumský MSDF**
  recovers sharp corners (`msdfgen`). Datum's current `lyon` CPU mesh cache is a valid
  middle ground; **Slug** is the drop-in upgrade for true zoom-independence.

### D. Curvature-adaptive flattening — the fab-fallback
If arcs are declined for a conservative fab profile, flatten *adaptively* (dense only at
high curvature): **Hain et al., *Computers & Graphics* 29(5), 2005** (~67% of the
vertices of recursive subdivision); **Raph Levien, "Flattening quadratic Béziers"
(2019)**. Even keeping flattening, **replacing scanline-trapezoids with one `G36/G37`
polygon per contour is already a ~100–300× win**.

## 3. Recommendation for Datum (doctrine-faithful; the concrete governed dev task)

1. **Renderer: unchanged** (`mesh.rs` lyon glyph-mesh cache stays; tolerance stays fine
   — the `outline.rs:270` doctrine is correct). Optional later upgrade: **Slug** backend
   for true resolution independence.
2. **Export: stop scanline-trapezoid-stroking.** Emit **one `G36…G37` region per glyph
   contour** (outer `%LPD*%`, holes `%LPC*%`). This alone kills the blow-up.
3. **Export: emit arcs, not line chains, for curved edges.** Feed the export a **biarc
   fit of the glyph's original `ttf_parser` quad/cubic segments** (not the pre-flattened
   polyline); emit `G75*` + `G03/G02` for arcs and `G01` for straight runs inside the
   region. Glyph → a few primitives.
4. **Fab compatibility by policy, not coarsening.** Export/validation profile: default =
   arc regions; conservative fallback = adaptive-flattened line-segment regions
   (Hain/Levien) for CAM tools with weak `G75`-in-region support.
5. **Cheap secondary wins:** use the format statement's zero-omission / shorter coords in
   `formatting.rs`; offer Newstroke stroke font as an opt-in Manufacturing default
   (stroked polylines are natively the smallest gerber — KiCad's own default), reserving
   outline+arc-region for branding/marketing silk.

**Bottom line:** the renderer is already right; the gerber blow-up is entirely the
export path (scanline-trapezoid-stroking, no regions, no arcs). Doctrine-faithful fix =
full-fidelity renderer untouched; export emits `G36/G37` regions whose curved edges are
`G75`+`G03/G02` arcs from biarc-fitting the original glyph Béziers, with an
adaptive-segment fallback gated by fab policy. This closes the arc/biarc deferral the
repo's own Phase-2/3 research left open.

## 4. Canonical references
- Ucamco, *The Gerber Layer Format Specification* — §4.10 regions, §4.7.2 arcs. <https://www.ucamco.com/en/gerber>
- W. Bolton, "Biarc curves," *CAD* 7(2), 1975 (seminal). dlacko practical write-up (2016).
- Loop & Blinn, SIGGRAPH 2005, ACM TOG 24(3). <https://dl.acm.org/doi/10.1145/1073204.1073303>
- E. Lengyel, "Slug," JCGT 2017 (public domain 2026). <https://sluglibrary.com/>
- Green (Valve) SDF, SIGGRAPH 2007; Chlumský MSDF (`msdfgen`).
- Hain et al., *Computers & Graphics* 29(5), 2005; Levien, flattening quad Béziers (2019).
- KiCad `GERBER_PLOTTER` (region + arc reference impl). <https://docs.kicad.org/doxygen/classGERBER__PLOTTER.html>

**Key repo files:** `crates/engine/src/text/outline.rs` (scanline trapezoid `:309`,
doctrine tolerance `:270`), `geometry.rs::polygon_ring_to_strokes:161`, `mesh.rs`
(good renderer path), `export/{silkscreen,export_surface,formatting}.rs` (writer — no
`G36/G37`, no `G75`), `text/registry.rs` (Manufacturing→outline default).
