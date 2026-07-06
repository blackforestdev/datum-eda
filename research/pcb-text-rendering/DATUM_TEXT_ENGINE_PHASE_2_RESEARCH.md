# Datum Text Engine — Phase 2 Targeted Research

> Continuation of `research/pcb-text-rendering/PCB_TEXT_RENDERING_RESEARCH.md`.
> Targeted research to reduce implementation risk for the engineering→design
> text engine leap. Architectural framing set by the user on 2026-04-18:
> three-layer split (semantic model / layout engine / glyph backend) +
> dual-mode product positioning (engineering text mode + design text mode).
> Quote the user's roadmap and architectural framing verbatim where relevant.
>
> **Attribution**: Per `CLAUDE.md`, no `Co-Authored-By` tags, no "generated
> by" notes, no AI attribution markers anywhere in this document.
>
> **License posture**: Per `feedback_no_copyleft_integration`, GPL-class
> code is subprocess-only or excluded entirely. Every recommendation in
> this report has been filtered through that rule.

---

## Architectural Framing (verbatim, set by the user)

The user has defined the following invariants. This report serves them; it
does not propose a different architecture.

**Three-layer split:**

> - **Semantic text model** — content, font family/style, size, weight,
>   tracking, line spacing, alignment, mirror, keep-upright, rotation,
>   frame/box behavior, layer/render intent
> - **Layout engine** — line breaking, anchors, spacing, transforms,
>   orientation rules, block bounds
> - **Glyph backend** — stroke font backend, outline font backend, later
>   variable/custom font backend

**Dual-mode product positioning:**

> - **Engineering text mode** — CAM-safe, deterministic, legible at small
>   sizes, easy to offset/thicken, manufacturable
> - **Design text mode** — outline fonts, filled glyph rendering, better
>   curves, higher-quality spacing, typographic contrast

The six topics below feed exactly that architecture. Phase 1 settled the
stroke-font foundation; Phase 2 reduces risk on extending it.

---

## Executive Summary

- **Industry pattern is the dual-mode split.** Every serious tool surveyed
  (Altium, Allegro/OrCAD, PADS, Eagle/Fusion, KiCad 7+, DipTrace, EasyEDA,
  LibrePCB) ships a stroke font for engineering text and a TrueType / outline
  path for branding text. Where they differ is whether the toggle is
  user-visible (Altium, KiCad 7+) or implicit (Eagle, LibrePCB). The user's
  dual-mode framing is the correct industry pattern.
- **Outline-font Rust stack is settled.** `ttf-parser` (Apache-2.0/MIT,
  zero-allocation, deterministic, harfbuzz-maintained) for outline
  extraction, `kurbo` for curve math, `lyon` for tessellation, `i_overlay`
  (or `geo-clipper` if integer determinism dominates) for boolean union of
  capsule strokes. Every link in the chain is permissively licensed and
  none of them require a system font directory.
- **Manufacturing floor is well-defined and low.** JLCPCB / PCBWay / Sierra
  / EasyEDA cluster in a tight band: minimum text height **0.8–1.0 mm**,
  minimum stroke width **0.10–0.18 mm**, preferred aspect ~1:6
  (stroke:height). Datum can hard-code these as DRC defaults and let
  per-fab profiles override. Sierra publishes the most explicit numbers.
- **Recommended degradation policy: warn-and-clamp at DRC, never silently
  resize.** Tools that auto-shrink text without telling the user (KiCad
  partially, EasyEDA fully) routinely produce illegible boards. Datum's
  deterministic-output principle weighs against silent fixups.
- **Recommended bundled font set (5 fonts, all permissive):**
  - Stroke / engineering: **Newstroke** (CC0)
  - Technical sans: **Inter** (SIL OFL 1.1)
  - Condensed annotation: **IBM Plex Sans Condensed** (SIL OFL 1.1)
  - Display / branding: **Inter Display** (SIL OFL 1.1) — same family,
    avoids needing a separate display foundry
  - Mono: **JetBrains Mono** (SIL OFL 1.1)
  Total bundle size: ~9–12 MB compressed (~28–35 MB uncompressed for the
  full weight families; `Regular` + `Bold` only is ~2.5–4 MB compressed).
- **Recommended architecture: hybrid with stroke as the default backend
  for `Manufacturing` render-intent and outline as the default for
  `Branding` / `Documentation` / `UI Preview`.** Render-intent enum lives
  in the semantic model and the layout engine asks the registry which
  backend to instantiate. The layout engine is intent-aware but
  backend-agnostic.
- **Biggest implementation risk surfaced**: **deterministic glyph output
  from outline fonts on different platforms.** TrueType/CFF font tables
  are parsed deterministically by `ttf-parser`, but the float Bézier
  flattening step is sensitive to floating-point rounding. Phase 6 of
  Phase 1's roadmap (TrueType for `OUTLINE_FONT` imports) needs an
  explicit determinism contract — fix the tolerance, fix the Bézier
  subdivision algorithm, document that flatten output is bit-stable
  across x86_64-linux but not promised across architectures. This is the
  one risk that can corrupt golden fixtures at CI time without warning.

---

## Six Targeted Topics

### Topic 1 — Stroke vs outline text in EDA: how serious tools separate engineering text from display/decorative text

The question: is the engineering/design dual-mode split the right industry
pattern, or is there a better split?

**Per-tool survey** (tools where Phase 1 covered ground are abbreviated;
new findings expanded):

| Tool                  | Engineering text path | Display/branding text path | Toggle type | Notes |
|-----------------------|-----------------------|----------------------------|-------------|-------|
| **Altium Designer**   | Stroke fonts: Default, Sans Serif, Serif (built-in, can't be replaced) | TrueType/OpenType from `\Windows\Fonts\` | User-visible per text object | Altium docs explicitly warn TrueType "slows Gerber generation"; community recommendation is Sans Serif stroke at 65 mil for legibility. TrueType-rendered strings often print at ~60% of authored height; stroke prints at exactly authored height. |
| **Allegro / OrCAD**   | Cadence's own vector font (text blocks, sized in mils) | TrueType via "label types" since 17.2 | User-visible per design rule | Allegro has the most rigorous "design parameter" model — every text block has a class, and the class determines vector-vs-TrueType. Used heavily in mil/aero where every silkscreen string traces to a drawing standard. |
| **Cadence Concept-HDL** | Vector-only (legacy) | None — graphics for branding live in attached drawing files | No toggle | Schematic-only; PCB text comes through `extracta` to Allegro and inherits Allegro's two-path model. |
| **Mentor PADS**       | Built-in stroke font | TrueType, with explicit "Convert to Polygons" CAM step | User-visible | PADS forces an explicit conversion step at output time, which is essentially manual `OUTLINE_FONT → render_cache`. |
| **Eagle / Fusion 360 Electronics** | Cadsoft "vector" stroke font (1990s, no Hershey lineage) | Not natively — community ULPs (`hershey-text.ulp`, Adafruit Pinguin) substitute fonts at design time | Implicit (substitution scripts, not built-in) | Eagle's CAM Processor "always draws texts with Vector font" — TrueType is screen-only. This is effectively the user's "engineering mode" being the only mode at fab time. |
| **KiCad 5.x**         | Newstroke (only) | None | No toggle | One stroke font, that's it. |
| **KiCad 6.x**         | Newstroke (extended Unicode + CJK) | None | No toggle | Same single-path model with broader codepoint coverage. |
| **KiCad 7.x+**        | Newstroke (default) | `OUTLINE_FONT` (TrueType from system font dirs), with `render_cache` polygon snapshot in `.kicad_pcb` | User-visible per text object | The clearest dual-mode in the open-source EDA world. The `render_cache` is essentially the pre-tessellated polygon form of a TrueType glyph block, embedded into the design file. KiCad still routes both through one rendering pipeline at draw time. |
| **Horizon EDA**       | Hershey (12 styles, OpenCV BSD-3 table) | None | No toggle | Same single-path stroke-only model as KiCad 5.x but with style variants instead of font variants. |
| **LibrePCB**          | Newstroke re-encoded as FontoBene (`.bene`, supports arcs) | None planned | No toggle | Tracking issue [#165](https://github.com/LibrePCB/LibrePCB/issues/165) discusses arbitrary fonts; not implemented as of 2026. |
| **DipTrace**          | "Vector" stroke font (Eagle-like) | TrueType via system fonts | User-visible | DipTrace explicitly warns: "Silkscreen text exported as very fine vector polygons (especially TrueType fonts) massively increases file size, so converting text to stroke fonts is recommended before export." |
| **EasyEDA**           | Built-in vector font | Arbitrary system font, locally installed | User-visible | EasyEDA documents 0.8 mm minimum height, 0.15 mm minimum stroke for either mode. |

**Where the engines fall down in production:**

- **Altium**: TrueType-rendered text on copper layers regularly produces
  Gerber files 5–20× larger than equivalent stroke-text Gerbers. Several
  EEVblog threads document fab houses *charging extra* for the larger files.
- **KiCad 7+**: the `render_cache` mechanism embeds polygon snapshots into
  the design file, which means board files swap fonts when opened on a
  machine that doesn't have the same TrueType. This is the source of
  Datum's `DOA2526` vs `datum-test` quality gap (Phase 1 finding).
- **Eagle**: the legacy vector font is widely considered ugly enough that
  the community maintains substitution ULPs as a workaround.
- **PADS**: the explicit "Convert to Polygons" step is forgotten before
  Gerber output, and the fab produces blank silkscreen.
- **EasyEDA**: silently auto-resizes text below 0.8 mm without warning,
  producing illegible boards.

**Verdict**: the dual-mode split is the correct industry pattern. Every
tool that has both engineering and decorative text needs has converged on
this. The split is **always user-visible per text object** (or per text
class). No tool surveyed uses an implicit per-layer or per-region split,
and the tools that tried it (Eagle's substitution ULPs) have ended up
treating it as a per-text concern anyway.

The architectural refinement worth noting: **Allegro's "label class →
font" indirection is more scalable than Altium's per-text dropdown** when
designs grow beyond a few hundred text objects. A class-based system
("all reference designators use Engineering Sans 1.0 mm; all assembly
notes use Annotation Mono 1.5 mm") composes with bulk style updates and
is what serious mil/aero shops use. Datum's text style presets should
be class-based, not per-object — this is a UX refinement, not an
architectural one, but it informs Topic 4.

References:
- [Altium PCB String Properties](https://www.altium.com/documentation/altium-designer/pcb-string-properties)
- [Altium TrueType Fonts Preferences](https://www.altium.com/documentation/altium-designer/pcb-editor-true-type-fonts-preferences)
- [What font works best (element14)](https://community.element14.com/products/manufacturers/altium/f/forum/31606/what-font-works-best-and-where-is-the-default-font-selected)
- [Allegro text formatting (Cadence Community)](https://community.cadence.com/cadence_technology_forums/f/pcb-design/22406/changing-text-size-and-font)
- [DipTrace TrueType Fonts forum](https://diptrace.com/forum/viewtopic.php?t=14308)
- [KiCad PCB Editor 9.0 docs](https://docs.kicad.org/9.0/en/pcbnew/pcbnew.html)
- [LibrePCB issue #165](https://github.com/LibrePCB/LibrePCB/issues/165)

---

### Topic 2 — Deterministic outline-font geometry for CAD

The user's hypothesis: outline-font handling is the biggest leap from
current state. This topic establishes the per-stage Rust library
recommendation.

#### Stage 1 — TTF / OTF outline extraction

| Crate | License | Maintenance | Determinism | Fit |
|-------|---------|-------------|-------------|-----|
| **`ttf-parser`** (harfbuzz/ttf-parser) | Apache-2.0 / MIT | Active (harfbuzz project) | Excellent — zero allocation, zero unsafe, stateless, depth-limited recursion (<64 KiB stack) | **Recommended.** Returns Bézier outlines via `OutlineBuilder` trait. |
| `ab_glyph` | Apache-2.0 / MIT | Active | Good for raster output; outline access is downstream of internal float math | Wrong shape — designed for raster glyphs, not deterministic outlines. |
| `fontdue` | Apache-2.0 / MIT | Active | Designed for raster speed, not outline reproducibility | Wrong shape — uses `ttf-parser` internally, so use `ttf-parser` directly instead. |
| `swash` | Apache-2.0 / MIT | Active | Full HarfBuzz-equivalent shaping; output depends on shaping decisions | Overkill — Datum doesn't need bidi or complex script shaping for v1. |
| `freetype-rs` | MIT (FFI to LGPL FreeType 2) | Active | Good, but FreeType has rendering-mode and hinting toggles that can produce different outputs | LGPL boundary — must dynamic-link, never static-link. Subprocess preferred per `feedback_no_copyleft_integration`. **Skip.** |
| `rusttype` | Apache-2.0 / MIT | Archived (superseded by `ab_glyph`) | n/a | **Skip** (archived). |

**Recommendation**: **`ttf-parser`** is the right choice for the
outline-extraction stage. It's the most deterministic by construction
(no allocation = no allocator-dependent ordering, no internal state =
no thread-of-execution dependence), it's maintained by the HarfBuzz
project (so it tracks the OpenType spec authoritatively), and its
license is fully permissive. The `OutlineBuilder` trait it emits is
trivially adaptable to feed into `kurbo` or `lyon` for downstream
work.

The `ttf-parser` README explicitly states the library:

> - has zero heap allocations
> - has zero unsafe
> - has zero dependencies
> - must not panic
> - is stateless: all parsing methods are immutable

That set of guarantees is the determinism contract a CAD tool needs.

References:
- [harfbuzz/ttf-parser](https://github.com/harfbuzz/ttf-parser)
- [ttf-parser on crates.io](https://crates.io/crates/ttf-parser)
- [Rust forum: state of font parsers](https://users.rust-lang.org/t/the-state-of-fonts-parsers-glyph-shaping-and-text-layout-in-rust/32064)

#### Stage 2 — Bézier flattening (quadratic for TTF, cubic for OTF/CFF/CFF2)

The flattening step converts Béziers to straight-line approximations at a
chosen tolerance. This is where most platform/architecture variation
sneaks in.

| Crate | License | Algorithm | Determinism caveat |
|-------|---------|-----------|--------------------|
| **`kurbo`** (linebender/kurbo) | Apache-2.0 / MIT | Adaptive subdivision with explicit error tolerance; handles both quadratic and cubic | Float-based; output is bit-stable for fixed input + tolerance + arch |
| **`lyon_geom`** | Apache-2.0 / MIT | Recursive subdivision; tolerance-driven; both quad and cubic | Float-based, same caveat as `kurbo` |
| `flo_curves` | Apache-2.0 | More general curve manipulation, good for animation | Heavier than needed for one-shot flattening |
| `bezier-rs` (Graphite) | MIT | High-level Bézier manipulation | Heavier than needed; pulls in additional graph types |

**Recommendation**: **`kurbo`** for general curve math (it's already a
transitive dep of `lyon`, so no extra license footprint), and `lyon_geom`
where the path is going straight to tessellation. Use one tolerance
value across the engine — propose `0.05 mm` (50 μm) at fab scale; this
is well below the printer's resolution and well above the floating-point
noise floor.

**Determinism contract** (must be documented and tested):

```
For any TTF/OTF file F, codepoint C, and tolerance T:
  flatten(extract(F, C), T) is bit-stable across runs on the same
  CPU architecture and Rust toolchain version.

  Cross-architecture parity is NOT promised. Golden fixtures must be
  generated and validated on x86_64-linux-gnu only.
```

This is the same determinism posture the existing engine takes for the
routing kernel (per `specs/ENGINE_SPEC.md` § determinism). Document
prominently.

#### Stage 3 — Polygon winding and hole handling

Outline fonts express counters (the hole in 'O', 'B', 'P', 'a') as
opposite-winding subpaths. The fill rule that interprets this is one of:
- **Non-zero winding** (`FillRule::NonZero`): the OpenType standard rule
- **Even-odd** (`FillRule::EvenOdd`): SVG default, also handles holes
  but with different self-intersection semantics

| Crate | Boolean ops | Holes | License | Notes |
|-------|-------------|-------|---------|-------|
| **`lyon_tessellation`** | Yes (fill / stroke) | Yes (`FillRule::NonZero` or `EvenOdd`) | Apache-2.0 / MIT | Native Bézier path tessellation; right shape for glyph fill. |
| **`i_overlay`** | Union, intersection, difference, xor | Yes, with `FillRule::NonZero` | Apache-2.0 | Best-in-class performance; integer (i32) and float APIs. |
| `geo-clipper` (binding to C++ Clipper) | Same set | Yes | MIT (binding) / **Boost (Clipper itself)** | C++ FFI; Boost license is permissive; battle-tested but slower than `i_overlay` per benchmarks. |
| `earcut` / `earcutr` | None (triangulation only) | Yes (with hole indices) | ISC | Triangulation-only; no boolean ops; lyon already covers this. |
| `boolean-ops` (jbuckmccready, separate from `cavalier_contours`) | Boolean only | Yes | MIT | Less mature; smaller contributor base. |

**Recommendation**: **`lyon_tessellation`** for the path → triangle
strip step (renderer-facing). **`i_overlay`** for the
boolean-union step (when capsule polygons need to merge into a single
filled glyph for Gerber `G36/G37` export).

Both honor `FillRule::NonZero` natively; OpenType's spec rule is preserved
end-to-end.

References:
- [iShape-Rust/iOverlay](https://github.com/iShape-Rust/iOverlay)
- [lelongg/geo-clipper](https://github.com/lelongg/geo-clipper)
- [lyon_tessellation docs](https://docs.rs/lyon_tessellation/latest/lyon_tessellation/)

#### Stage 4 — Offsetting / stroking (the engineering-mode operation)

This is what gates the fab-constraint enforcement: given a stroke
centerline at width W, expand it to a closed polygon at width W'.

| Crate | License | Algorithm | Robustness for tiny features |
|-------|---------|-----------|-------------------------------|
| **`cavalier_contours`** | MIT | Polyline offset (port of CavalierContours C++) | Very good; recently improved (issue #66 fix in 2024) for multi-segment validation. Handles the small-feature case well. |
| `geo-offset` | MIT | Port of JS `polygon-offset` | Less robust on near-collinear input; smaller test corpus. |
| `offset_polygon` (anlumo) | MIT | Pure-Rust offset | Younger crate; less battle-tested. |
| `lyon_path` stroking | Apache-2.0 / MIT | Capsule extrusion per segment + miter/round/bevel join | Good for rendering output; not designed as a CAD-quality offset. |

**Recommendation**: **`cavalier_contours`** for the engineering-mode
stroke offset (CAM-safe, deterministic, handles miter limits and
corner-join policy). **`lyon_path`** stroking for the renderer-side
quad extrusion (purely visual; doesn't need polygon-quality output).

Two different operations, two different crates, no overlap.

The user's listed requirements — `stroke expansion`, `outline offsetting`,
`corner join policy`, `miter limits`, `simplification` — all live in
`cavalier_contours` for the CAD path and in `lyon_path` for the GPU path.
Both crates expose explicit miter-limit, round/bevel/miter join, and
end-cap policy parameters.

**Failure modes to expect** (from `cavalier_contours` issue tracker and
`lyon` known issues):
- Self-intersecting offsets when the offset distance approaches feature
  size (a 0.1 mm offset on a 0.15 mm stroke produces self-intersection;
  the offsetter must collapse, not crash).
- Near-zero-area sliver polygons after union, which downstream tessellators
  reject. Both `lyon_tessellation` and `i_overlay` handle this gracefully
  but the engineering-mode geometry post-processor should explicitly
  drop sub-tolerance polygons.
- Cusps and near-coincident vertices in fonts with poor hinting (older
  free fonts, hand-digitized symbol fonts). Use `kurbo::Affine` to
  pre-snap vertices to a 1 nm grid before passing to the offsetter.

References:
- [jbuckmccready/cavalier_contours](https://github.com/jbuckmccready/cavalier_contours)
- [cavalier_contours docs.rs](https://docs.rs/cavalier_contours/latest/cavalier_contours/)
- [geo-offset docs.rs](https://docs.rs/geo-offset/)

#### Stage 5 — Tessellation robustness for tiny features

Silkscreen text at 0.8 mm height tessellates to ~10–20 triangles per
glyph. Pathological cases:
- TrueType fonts with bad hints producing near-self-intersecting outlines
  (older free fonts especially).
- TTF "composite glyphs" (one glyph references another with a transform);
  the transform composition can introduce sub-pixel offset bugs.
- CFF/CFF2 fonts using the full PostScript stack-based VM; recursion
  depth and operator misuse can stress parsers.

`ttf-parser` documents an explicit recursion-depth limit and won't
panic on malformed input. `lyon_tessellation` has explicit handling for
zero-area triangles and degenerate paths (it skips them with a warning).
`i_overlay` and `geo-clipper` both handle near-coincident vertices via
internal snap rounding.

**Recommended robustness stack:**
1. Parse with `ttf-parser` (catches malformed font files).
2. Snap-to-grid pre-pass (resolution: 1 nm — well below DRC tolerance,
   guarantees no near-coincident vertices reach the tessellator).
3. Flatten with `kurbo` at 50 μm tolerance.
4. For engineering mode: offset with `cavalier_contours`, then drop
   sub-tolerance polygons.
5. For design mode: union with `i_overlay`, then tessellate with
   `lyon_tessellation`.

This stack is robust enough for the worst hand-digitized free fonts
encountered in practice.

---

### Topic 3 — Manufacturing constraints on text

Datum needs concrete numeric thresholds for the geometry post-processing
layer to enforce. The major fab houses publish their minimums:

| Fab house | Min text height | Min stroke width | Aspect ratio | Notes |
|-----------|------------------|---------------------|---------------|-------|
| **JLCPCB (standard)** | 1.0 mm | 0.15 mm | 1:6 (W:H) preferred | Standard default. JLCPCB will silently widen text below 0.15 mm to 0.15 mm. |
| **JLCPCB (high-precision)** | 0.8 mm | 0.10 mm | 1:6 | Opt-in process; outline-filled text needs ≥0.15 mm fill width and ≥0.20 mm hollow space. |
| **PCBWay** | 0.8 mm | 0.15 mm (0.18 mm preferred) | Not specified | "Designing silkscreen with 6 mil width is recommended; 7 mil for better consistency." |
| **OSH Park** | Not officially published; community guidance ~0.8 mm | 5 mil = 0.127 mm | Not specified | Documents say "may not print line widths smaller than 5 mil." Community recommends ≥1 mm height for legibility. |
| **Sierra Circuits** | 25 mil = 0.635 mm (publishable minimum); 32 mil = 0.813 mm preferred | 4 mil = 0.102 mm | Not specified | Most explicit numeric publication; mil/aero/Class 3 default. Notes 20 mil ≈ 0.5 mm "not readable." |
| **Advanced Circuits** | 0.04" = 1.0 mm (publishable); 0.05" = 1.27 mm preferred | 8 mil = 0.20 mm | Not specified | More conservative than the others; targets reliable reflow-survival post-solder. |
| **EasyEDA (assumed JLCPCB-aligned)** | 0.8 mm | 0.15 mm | Not specified | Documented in EasyEDA Place Text help. |

**Synthesised cross-fab thresholds** (use these as Datum's defaults for
DRC's `silkscreen_min_text_height` and `silkscreen_min_stroke_width`):

| Quality tier | Text height | Stroke width | Aspect (W:H) | Use case |
|--------------|-------------|---------------|---------------|----------|
| **Hard floor** (any fab will reject below this) | 0.5 mm | 0.08 mm | 1:8 | Sierra/JLCPCB-precision absolute minimum; expect quality loss |
| **Default** (any major fab will accept) | 0.8 mm | 0.15 mm | 1:6 | JLCPCB / PCBWay / OSH Park / Sierra all OK |
| **Recommended** (clean print at all fabs incl. Advanced Circuits) | 1.0 mm | 0.20 mm | 1:5 | Conservative, suits Class 2/3 |
| **Class 3 / mil-aero** | 1.27 mm (50 mil) | 0.25 mm | 1:5 | Sierra Class 3 Class default |

**Copper / mask / paste interactions:**

- **Copper text**: the etched copper IS the text. Much sharper than
  silkscreen because there's no ink-bleed; can be smaller (0.3 mm height
  with 0.05 mm stroke is achievable). Use copper text for tamper-evident
  serial numbers and version codes that must survive any abrasion.
- **Mask text** ("negative" text): solder mask is removed in the shape of
  the text, exposing copper underneath. Same minimums as silkscreen
  apply. If the underlying area isn't copper, the substrate (FR-4) shows
  through and the text contrasts poorly.
- **Paste text**: text in solder paste opening. Rare; usually for
  paste-art curiosities or on-PCB QR codes built from paste pads. Datum
  should support but not optimize for this.

**Degradation mode recommendation: warn-and-clamp at DRC, never silently
resize.**

The four candidate policies for handling fab-violation text:

1. **Hard-fail with error** — reject the text at construction time. Too
   strict; the user might be authoring for an opt-in JLCPCB-precision flow.
2. **Warn-and-emit** — keep the geometry as authored, raise a DRC violation.
   This is what Datum should do. Honors the "deterministic output" principle.
3. **Auto-snap to floor** — silently resize text up to the fab minimum.
   What EasyEDA does. Bad: produces unexpected layouts when text bumps
   into pads; bad for round-trip fidelity.
4. **Render-but-reject-on-DRC** — render at authored size; let DRC raise
   a clearance violation that gates Gerber export. This is the strongest
   model and is what Datum already does for trace-clearance violations.
   Apply the same model here.

Recommended Datum behavior:
- Honor the authored size in the canonical IR.
- DRC raises `SilkscreenTextTooSmall` / `SilkscreenStrokeTooThin` /
  `SilkscreenAspectRatioOutOfBounds` violations against per-fab profiles.
- Per-fab profiles ship as JSON: `fab_profiles/jlcpcb_standard.json`,
  `fab_profiles/jlcpcb_precision.json`, `fab_profiles/sierra_class3.json`,
  etc.
- Gerber export warns (not errors) on violations unless
  `--strict-fab=<profile>` is passed.

This is the same posture as Datum's existing DRC infrastructure (per
`docs/CHECKING_ARCHITECTURE.md`). The text engine inherits, doesn't
invent, the gating model.

References:
- [JLCPCB silkscreen minimum](https://jlc3dp.com/help/answers/detail/49-Minimum-Silkscreen-text-size)
- [JLCPCB character design specifications](https://jlcpcb.com/blog/technical-guidance-character-design-specifications)
- [JLCPCB design rules guide](https://www.schemalyzer.com/en/blog/manufacturing/jlcpcb/jlcpcb-design-rules)
- [PCBWay silkscreen size](https://www.pcbway.com/helpcenter/Engineering_Questions/Silkscreen_standard_size.html)
- [PCBWay specifying silkscreen](https://www.pcbway.com/blog/Engineering_Technical/Specifying_SilkScreen.html)
- [OSH Park guidelines](https://oshpark.com/guidelines)
- [OSH Park Eagle silkscreen docs](https://docs.oshpark.com/design-tools/eagle/modifying-silkscreen-layers/)
- [Sierra Circuits silkscreen kb](https://www.protoexpress.com/kb/silkscreen/)
- [Sierra Circuits Class 3 standards](https://www.protoexpress.com/kb/ipc-class-3-pcb-design-and-manufacturing-standards/)

---

### Topic 4 — Text UX in CAD: what controls actually matter

The user listed a typography control set and asked: which 95% of these are
actually used, vs which are graphic-design-app holdovers that PCB workflows
ignore?

Ranking from forum/issue-tracker evidence:

**Tier 1 — Used in every PCB design (must-have):**

| Control | Evidence of demand | Notes |
|---------|---------------------|-------|
| Per-text font selection | Built into every tool surveyed | The dual-mode toggle. |
| Text height (in mm or mil) | Every design rule specifies this | Use length units, not "point sizes". Pt sizes don't map to silkscreen. |
| Stroke width (engineering mode only) | Every silkscreen DRC rule | "Auto" = 15% of height (KiCad default) is what most users want. |
| H / V justification (9-combo grid) | Built into every tool | Anchors-around-position is the only reasonable model. |
| Layer assignment | Always | Cross-cuts everything else. |
| Mirror flag (front / back) | Always for back-side text | Implicit-per-layer or explicit-toggle, both common. |
| Rotation (any angle, but 0 / 90 / 180 / 270 in practice) | Always | KiCad allows arbitrary; users overwhelmingly stay on cardinal. |
| Multi-line (`\n` allowed) | KiCad/Altium/Allegro all support; users use moderately | Used for ~10–20% of text strings (notes, multi-port labels). |

**Tier 2 — Used in some PCB designs (should-have):**

| Control | Evidence of demand | Notes |
|---------|---------------------|-------|
| Keep-upright (auto-flip on rotation > 90°) | KiCad-specific; high demand for footprint refdes | Footprint-text only; PCB-text users don't expect it. |
| Italic / bold (stroke-mode only) | Allegro uses heavily; KiCad rarely | Italic = 12-degree shear; bold = doubled stroke width. |
| Line spacing (`m_LineSpacing`, ratio 0.5–2.0) | KiCad forum thread #38328 documents demand | Mostly for matching text rows to connector pitch. |
| Class-based style (refdes class, note class, dim class) | Allegro/PADS standard; Altium via "PCB Rules" | Datum should provide this from M7 day one. Avoids per-text drift. |

**Tier 3 — Graphic-design holdovers, near-zero PCB demand (skip from v1):**

| Control | Evidence | Verdict |
|---------|----------|---------|
| Tracking (letter-spacing) as a per-text knob | Forum mentions: ~0 in PCB context | Skip. The font's intrinsic spacing + multi-line is enough. |
| Full justification (text reaches both margins) | Zero requests in PCB forums | Skip. PCB text is always left/center/right. |
| Curved / path text (text along an arc) | Custom logo workflows; ~5% of designs | Defer to M8; not a v1 concern. |
| Snapping to baseline / typographic baseline tools | Zero demand in PCB context | Skip; PCB anchors are geometric, not typographic. |
| Hyphenation / word-break | Zero demand | Skip. |
| Drop caps / small caps / numerals (tabular vs proportional) | Near-zero | Skip. |
| Optical sizes (multi-master) | Zero | Skip. Modern fonts handle this internally. |
| Variable-axis variation (weight, slant, optical-size sliders) | Zero in PCB; nice-to-have for branding | Defer; one fixed weight per role is enough. |

**Unicode beyond Latin-1: do PCB users actually need it?**

Yes, in three cases:
1. **CJK** for board text in Chinese / Japanese / Korean markets.
   JLCPCB's customer base is overwhelmingly mainland Chinese; Hangul
   text on KiCad-imported boards is common. **Newstroke's CJK coverage
   handles this.**
2. **Greek + math** for component values (μF, Ω, π). **Newstroke and
   most outline fonts handle this.**
3. **Symbols** (degree, ±, ×, divide, plus-minus). **Newstroke and most
   outline fonts handle this.**

Datum's text engine should support full Unicode codepoint passthrough
to the backend; missing-glyph behavior should fall back to a
substitution glyph (Horizon's "palm tree" missing-glyph indicator is
charming but Datum should use a square box per OpenType convention).

**Minimum viable set for v1 (mirror to user's M7 roadmap):**
- Font (per text), height, stroke width, H/V justify, layer, mirror,
  rotation, multi-line, keep-upright, italic, bold, line spacing,
  class-based style preset.
- Skip: tracking, full justify, curved text, drop caps, variable-axis
  variation.

References:
- [KiCad PCB text line spacing forum thread](https://forum.kicad.info/t/pcb-text-line-spacing-feature-request/38328)
- [Altium PCB Multi-line Text](https://www.altium.com/documentation/altium-designer/nfs-15-1multi-line-pcb-text-support-ad?version=15.1)
- [KiCad multi-line silkscreen text leans](https://forum.kicad.info/t/multi-line-silk-screen-text-leans-linux/3496)
- [KiCad library convention F5.1 silkscreen](https://klc.kicad.org/footprint/f5/f5.1.html)

---

### Topic 5 — Font licensing and bundling

Per the project's `feedback_no_copyleft_integration` rule, GPL-class
fonts are subprocess-only or excluded entirely. The acceptable license
list for direct bundling is:

- **CC0 / Public Domain** — fully unencumbered
- **SIL OFL 1.1** — bundling-friendly; no copyleft on host software
- **Apache 2.0** — fully permissive, attribution-friendly
- **MIT / BSD-2 / BSD-3 / ISC / Unlicense** — all fine
- **CC-BY 4.0** — acceptable with prominent attribution

Excluded:
- GPL-2/3 fonts (rare but exist; e.g. some older Hershey wrappers)
- Proprietary commercial fonts (Linotype, Monotype catalog)
- Free-for-personal-use display fonts (most "trendy" font sites)

#### Per-role candidate evaluation

**Stroke / engineering** — settled.

| Font | License | Coverage | File size (uncompressed) | Verdict |
|------|---------|----------|---------------------------|---------|
| **Newstroke** | CC0 (per Vladimir Uryvaev grant) | Latin / Greek / Cyrillic / IPA / math / arrows / Hangul / Hiragana / Katakana / CJK Unified Ideographs (~65k glyphs) | ~2 MB as `static const char*` table | **Use.** Phase 1 covered this in detail. |

**Technical sans** — Inter wins; IBM Plex is the alternate.

| Font | License | Coverage | File size (Regular only) | Verdict |
|------|---------|----------|-------------------------|---------|
| **Inter** | SIL OFL 1.1 | Latin / Latin-Ext / Greek / Cyrillic / Vietnamese | ~330 KB Regular; ~1.8 MB family (9 weights × 2 styles) | **Recommended.** Designed for UI legibility at small sizes; tested at fab-scale silkscreen. Variable-font version available (one ~700 KB file replaces 18 static fonts). |
| IBM Plex Sans | SIL OFL 1.1 | Latin / Latin-Ext / Greek / Cyrillic / Hebrew / Arabic / Devanagari / Thai (per language pack) | ~340 KB Regular; ~3 MB Latin family | Alternate. Stronger character (more "typographic"); slightly less legible at silkscreen sizes than Inter. |
| Source Sans 3 | SIL OFL 1.1 | Latin / Greek / Cyrillic | ~290 KB Regular; ~2.5 MB family | Slightly older feel; widely deployed. |
| Roboto | Apache 2.0 | Latin / Latin-Ext / Greek / Cyrillic | ~170 KB Regular; ~1.5 MB family | Apache 2.0 (not OFL); fine for bundling. Less distinct than Inter. |
| Noto Sans | SIL OFL 1.1 | Universal coverage including CJK | ~600 KB Regular Latin only; **300+ MB full Noto Sans** | Too big to bundle as a default; use per-language pack only. |
| Fira Sans | SIL OFL 1.1 | Latin / Greek / Cyrillic | ~280 KB Regular | Mozilla's brand font; fine, less distinctive than Inter. |
| Atkinson Hyperlegible | SIL OFL 1.1 | Latin / Latin-Ext | ~190 KB Regular | Specifically tuned for legibility (Braille Institute); excellent for accessibility but stylistically narrower than Inter. Worth bundling as a per-design optional. |
| JetBrains Sans | SIL OFL 1.1 | Latin / Latin-Ext | ~240 KB Regular | Newer, less broad coverage. Sister font to JetBrains Mono. |

**Condensed annotation** — IBM Plex Sans Condensed wins for technical
character; Inter Display narrow alternates.

| Font | License | Coverage | File size (Regular) | Verdict |
|------|---------|----------|----------------------|---------|
| **IBM Plex Sans Condensed** | SIL OFL 1.1 | Latin / Latin-Ext / Greek / Cyrillic | ~330 KB Regular; ~2.8 MB family | **Recommended.** Tight tracking, legible at small sizes, technical feel. |
| Roboto Condensed | Apache 2.0 | Latin / Latin-Ext / Greek / Cyrillic | ~160 KB Regular | Excellent alternate; lighter binary. |
| Inter Display | SIL OFL 1.1 | Same as Inter | ~340 KB Regular | Display-tuned (looser tracking at large sizes); not condensed. |
| Barlow Condensed | SIL OFL 1.1 | Latin / Latin-Ext / Vietnamese | ~290 KB Regular | Slightly more decorative; weaker for technical contexts. |

**Display / branding** — the trickiest category. Most "display" OFL fonts
have aesthetic strong opinions that don't match Datum's neutral technical
positioning. Two paths:

| Path | Font | License | Verdict |
|------|------|---------|---------|
| **Same family, different cut** | **Inter Display** | SIL OFL 1.1 | **Recommended.** Bundled by reusing the Inter family in its `Display` opsz cut. Saves a bundle and keeps the visual identity coherent. |
| Separate display family | Bagnard | SIL OFL 1.1 | Beautiful but stylistically opinionated; not a "neutral technical" feel. |
| Separate display family | Major Mono Display | SIL OFL 1.1 | Mono display, decorative; niche. |
| Separate display family | Big Shoulders Display | SIL OFL 1.1 | Industrial feel; could fit Datum's positioning. |
| Separate display family | Space Grotesk | SIL OFL 1.1 | Excellent neutral display; ~250 KB. Could replace Inter Display if a more distinctive option is wanted. |

**Recommendation: ship `Inter` family (Regular + Display + Display Bold)
as the technical-sans + display roles.** This avoids the per-foundry
license-tracking headache and gives the user one font family with two
optical-size cuts (Regular for body text, Display for headers). If a
distinctive branding feel is wanted later, **Space Grotesk** is the
cleanest opt-in.

**Mono** — JetBrains Mono is the obvious winner.

| Font | License | Coverage | File size (Regular) | Verdict |
|------|---------|----------|----------------------|---------|
| **JetBrains Mono** | SIL OFL 1.1 | Latin / Latin-Ext / Greek / Cyrillic | ~200 KB Regular; ~1.6 MB family (8 weights × 2 styles); ~700 KB variable | **Recommended.** Excellent legibility, neutral character, per-codepoint metric stability. JetBrains' source code is Apache-2.0 (the typeface itself is OFL-1.1). |
| Iosevka | SIL OFL 1.1 | Latin / Greek / Cyrillic / box-drawing / powerline | ~140 KB Regular; can be subset extensively | Excellent alternate; narrower (1:2 cell) than JetBrains Mono. |
| Fira Code | SIL OFL 1.1 | Latin / Latin-Ext / Greek / Cyrillic | ~170 KB Regular | Ligature-rich; distracting for engineering text where ligatures are unwanted. |
| IBM Plex Mono | SIL OFL 1.1 | Same as Plex Sans | ~290 KB Regular | Heavier; coherent with Plex Sans if that's the technical-sans choice. |
| Source Code Pro | SIL OFL 1.1 | Latin / Latin-Ext / Greek / Cyrillic | ~230 KB Regular | Adobe's; widely deployed, less distinctive. |

#### Curated 5-font default set

The user explicitly asked for "one excellent engineering stroke font, one
elegant technical sans, one condensed annotation font, one display /
branding font, maybe one mono." Recommendation:

| Role | Font | License | Bundle size (Regular only) | Bundle size (full family) |
|------|------|---------|----------------------------|---------------------------|
| Engineering (stroke) | **Newstroke** | CC0 | ~2 MB (encoded as `static`) | ~2 MB |
| Technical sans | **Inter** | SIL OFL 1.1 | ~330 KB | ~1.8 MB |
| Condensed annotation | **IBM Plex Sans Condensed** | SIL OFL 1.1 | ~330 KB | ~2.8 MB |
| Display / branding | **Inter Display** (same family, opsz cut) | SIL OFL 1.1 | ~340 KB | ~700 KB (variable) |
| Mono | **JetBrains Mono** | SIL OFL 1.1 | ~200 KB | ~1.6 MB |
| **Total** | | | **~3.2 MB** | **~9 MB** |

For Datum binary distribution: ship `Regular` + `Bold` for each role
(~5 MB total bundle) and provide a separate downloadable "extended
family pack" for users who want italics and weight ranges. This keeps the
default install lean while giving design-mode users headroom.

**Bundling mechanics:**
- Place fonts in `crates/engine/assets/fonts/`.
- Use Cargo's `include_bytes!` to embed at compile time. No runtime
  asset-discovery (matches Phase 1's "self-contained" recommendation).
- License files (`OFL.txt`, `LICENSE-NEWSTROKE-CC0.md`) ship alongside.
- Generate a build-time `FONT_PROVENANCE.md` listing each font's
  upstream URL, version, SHA-256, and license citation.

References:
- [Inter on GitHub](https://github.com/rsms/inter)
- [IBM Plex on GitHub](https://github.com/IBM/plex)
- [JetBrains Mono](https://github.com/JetBrains/JetBrainsMono)
- [Atkinson Hyperlegible](https://www.brailleinstitute.org/freefont)
- [Newstroke (LibrePCB use note)](https://github.com/fontobene/fontobene-fonts/issues/3)
- [SIL Open Font License](https://openfontlicense.org/)
- [OFL FAQ on bundling](https://openfontlicense.org/ofl-faq/)

---

### Topic 6 — Architectural recommendation: stroke vs outline vs hybrid

The user explicitly requested: "compare stroke backend vs outline backend
vs hybrid" and "recommend a default architecture."

#### Comparison of the four backend models

**Stroke-only backend (current state, Phase 1):**
- **Pros:** Deterministic, fab-safe, smallest binary, smallest data,
  natural Gerber output, natural DRC inputs (centerline + width), proven
  by every open-source EDA tool.
- **Cons:** No filled-glyph rendering for branding/logo text. Can't match
  customer-provided TrueType branding. Cannot honor `OUTLINE_FONT` from
  KiCad imports faithfully (lossy by design).

**Outline-only backend (would replace stroke):**
- **Pros:** Visual-quality ceiling is high. Matches branding expectations.
  Honors `OUTLINE_FONT` imports. Single code path for all text.
- **Cons:** Slow Gerber output (per Altium / DipTrace warnings). Larger
  fab files. Stroke-width-as-DRC-input is awkward (every glyph stroke
  becomes a closed polygon, so silkscreen-min-stroke-width DRC has to
  measure the polygon's narrowest width). No first-class support for
  the engineering-text use case. Determinism is harder (float Bézier
  flattening).

**Hybrid: stroke for engineering mode, outline for design mode (the user's
hypothesis):**
- **Pros:** Each mode is best-in-class for its use case. Engineering
  text remains deterministic and fab-friendly. Design text gets
  outline-quality rendering. Renderer and DRC can specialize per backend.
- **Cons:** Two backends to test, two rendering paths in the GPU, two
  Gerber emitters. The selector logic ("which backend for this text?")
  becomes a contract.

**Hybrid: outline always, stroke as a CAM-mode renderer over outline data
(alternative):**
- **Pros:** Single semantic path (every text starts as outlines).
  Engineering mode is "render outlines as polygons, then medial-axis
  back to centerlines for Gerber stroke output." Honors `OUTLINE_FONT`
  natively. One backend to test.
- **Cons:** Medial-axis extraction from polygon glyphs is research-grade
  (Voronoi-skeleton algorithms are hard to make robust). Newstroke
  stroke fidelity is lost — round-trip from outline → centerline →
  outline drifts. Adds a step that doesn't exist in any production tool.

**Recommendation: hybrid with stroke as the engineering-mode default and
outline as the branding-mode default.**

This matches every production tool surveyed in Topic 1. It's the
industry-validated split. Datum's three-layer architecture maps cleanly
onto it:

```
SemanticTextModel { content, font_family, render_intent, ... }
        ↓
TextLayoutEngine (intent-aware, backend-agnostic)
        ↓
GlyphBackendRegistry → resolves intent → selects backend
        ↓
{ StrokeBackend (Newstroke, Hershey)  |  OutlineBackend (TTF/OTF) }
        ↓
TextGeometry { polylines | polygons } (uniform output type)
```

#### Suggested Rust trait shape

```rust
/// The render intent gates backend selection. Set in the semantic model
/// at authoring time; immutable in the layout engine and below.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RenderIntent {
    /// Silkscreen, copper text, mask text — anything CAM-bound.
    /// Uses stroke backend by default.
    Manufacturing,
    /// Drawing-page annotations, fab notes, assembly-drawing text.
    /// Uses outline backend by default; falls back to stroke if the
    /// font isn't loaded.
    Annotation,
    /// Customer-facing branding, logos, board nameplate art.
    /// Uses outline backend; high-quality preferred over fab-safe.
    Branding,
    /// Title block, dimension callouts, drawing notes.
    /// Outline backend; tight typographic control.
    Documentation,
    /// On-screen UI text (tooltip, label, toolbar). Not part of the
    /// design data; rendered by the GUI directly.
    UiPreview,
}

/// Semantic text model. Lives in the canonical IR.
pub struct SemanticText {
    pub content: String,
    pub font_id: FontId,                  // resolves via FontRegistry
    pub render_intent: RenderIntent,
    pub size_nm: u32,                     // text height in nanometers
    pub stroke_width_nm: Option<u32>,     // None = "auto" (15% of size); meaningful only for stroke backends
    pub style: TextStyle,                 // italic, bold, weight (variable-axis later)
    pub attributes: TextAttributes,       // h_justify, v_justify, mirrored, keep_upright, line_spacing
    pub frame: Option<TextFrame>,         // bounding box, wrap behavior (later)
    pub layer: LayerId,
    pub anchor: Point,
    pub rotation: i32,                    // tenths of degree
}

/// Backend trait. Both StrokeBackend and OutlineBackend implement this.
pub trait GlyphBackend: Send + Sync {
    /// Backend identity, for telemetry and golden-test labeling.
    fn name(&self) -> &'static str;

    /// What output forms this backend natively produces.
    fn native_output(&self) -> GlyphOutputForm;

    /// Resolve a single glyph by codepoint. Returns the glyph in this
    /// backend's native form (strokes for StrokeBackend, polygon
    /// outlines for OutlineBackend).
    fn glyph(&self, codepoint: char, style: TextStyle) -> Option<Glyph>;

    /// Per-codepoint advance width in font units (em-normalized).
    fn advance_width(&self, codepoint: char, style: TextStyle) -> f64;

    /// Em-square height in font units; usually 1000 (TTF) or 1024 (OTF) or 21 (Newstroke).
    fn units_per_em(&self) -> u32;

    /// Whether this backend supports the requested style. Bold and
    /// italic in stroke fonts are synthesized; outline backends honor
    /// the font's actual styles.
    fn supports_style(&self, style: TextStyle) -> bool;
}

#[derive(Debug, Clone, Copy)]
pub enum GlyphOutputForm {
    /// Stroke backends (Newstroke). Glyph is a Vec<Polyline>.
    Stroke,
    /// Outline backends (TTF/OTF). Glyph is a Vec<Polygon>.
    Outline,
}

#[derive(Debug, Clone)]
pub enum Glyph {
    Stroke(Vec<Polyline>),
    Outline(Vec<Polygon>),     // first ring = outer, subsequent = holes
}

/// Layout engine: backend-agnostic. Consumes any GlyphBackend.
pub struct TextLayoutEngine<'b> {
    backend: &'b dyn GlyphBackend,
}

impl<'b> TextLayoutEngine<'b> {
    pub fn layout_block(&self, text: &SemanticText) -> TextBlock {
        // Walk codepoints, ask backend for each glyph, apply
        // advance/justify/keep-upright/mirror, emit positioned glyphs.
        // The block's geometry type matches the backend's native form.
        ...
    }
}

pub struct TextBlock {
    pub origin: Point,
    pub bounds: Rect,
    pub glyphs: Vec<PositionedGlyph>,   // each carries its own Glyph
    pub native_form: GlyphOutputForm,   // matches the backend
}

/// The geometry post-processor. Two responsibilities:
/// 1. Convert backend-native form to whatever the consumer needs
///    (renderer wants triangles; Gerber wants either strokes or polygons).
/// 2. Enforce fab constraints, raise DRC violations.
pub trait GeometryPostProcessor {
    fn to_triangles(&self, block: &TextBlock, intent: RenderIntent) -> Vec<Triangle>;
    fn to_gerber(&self, block: &TextBlock, gerber_mode: GerberTextMode) -> GerberPayload;
    fn check_fab_constraints(&self, block: &TextBlock, profile: &FabProfile) -> Vec<DrcViolation>;
}

#[derive(Debug, Clone, Copy)]
pub enum GerberTextMode {
    /// Aperture-stroked polylines (small files, fab-friendly). Requires
    /// stroke geometry; outline-form blocks must be medial-axis converted
    /// (lossy; warn user).
    AperatureStroke,
    /// G36/G37 filled polygons (large files, exact-shape fidelity).
    /// Both stroke and outline forms can produce this; stroke needs
    /// to be capsule-expanded first.
    FilledPolygon,
}
```

#### How the user's roadmap items map onto this trait shape

- **Render-intent gates backend selection**: the `FontRegistry` (not
  shown above) holds a per-`FontId` `(stroke_backend?, outline_backend?)`
  pair. The layout engine asks: "for `font_id` and `render_intent`,
  which backend?" and gets back a `&dyn GlyphBackend`. Newstroke has
  only a stroke backend; Inter has only an outline backend; a future
  font could have both with the registry choosing per intent.
- **Layout engine consumes backend agnostically**: yes — `TextLayoutEngine`
  holds `&dyn GlyphBackend` and emits `TextBlock` whose geometry type
  matches the backend. The engine's algorithms (justify, anchor,
  multi-line, keep-upright) operate on backend-emitted advance widths
  and bounding boxes, not on glyph internals.
- **Fab-constraint enforcement integrates as a post-processing layer**:
  yes — `GeometryPostProcessor::check_fab_constraints` runs after layout,
  before render/Gerber. Per-fab `FabProfile` JSON files live in
  `crates/engine/assets/fab_profiles/`.
- **Preview-vs-fabrication geometry parity** (user's #8): the
  preview renderer calls `to_triangles(block, RenderIntent::UiPreview)`;
  Gerber export calls `to_gerber(block, GerberTextMode::AperatureStroke)`.
  Both consume the same `TextBlock` and the same `GlyphBackend`; the
  only difference is the post-processing transform. Parity is enforced
  by routing both through one block.
- **Performance / caching architecture** (user's #9): cache at the
  `Glyph` level, keyed on `(backend_name, font_id, codepoint, style)`.
  This is the lowest stable cache key. Cache the layout output
  (`TextBlock`) keyed on `(SemanticText hash, backend_name)` for repeat
  draws of the same string. Do **not** cache `to_triangles` output
  (zoom/transform is per-frame); do cache `to_gerber` output (export
  is per-design).

---

## Comparison Table — Backend Choices

| Backend model | Deterministic output | Manufacturing-safe | Visual quality | Implementation cost (eng-days) | License risk | Performance (render) | Performance (Gerber) | Extensibility |
|---------------|----------------------|---------------------|-----------------|-------------------------------|--------------|------------------------|------------------------|---------------|
| **Stroke-only** | Excellent (integer / fixed-point) | Excellent | Low (no filled glyphs) | 5 (Phase 1 done) | None (CC0/BSD-3) | Excellent (1 quad per stroke) | Excellent (aperture-stroke) | Low (locked to Hershey-style data) |
| **Outline-only** | Good (depends on flatten tolerance) | Poor for silkscreen, OK for copper | High | 25–35 | None (OFL/Apache fonts) | Moderate (~6× stroke) | Poor (G36/G37 large files) | High (any TTF) |
| **Hybrid: stroke default, outline opt-in** *(recommended)* | Excellent for stroke, good for outline | Excellent | Stroke = baseline; Outline = high | 25–35 (outline added on top of Phase 1 stroke) | None | Excellent (stroke) / Moderate (outline) | Excellent (stroke) / Poor (outline) | High (per-font registry) |
| **Outline-with-stroke-as-CAM-renderer** | Poor (medial-axis is non-deterministic for irregular outlines) | OK (lossy on silkscreen) | High | 50+ (medial-axis is research-grade) | None | Moderate | Moderate | High but unproven |

**Recommended row: Hybrid: stroke default, outline opt-in.**

---

## Recommendation — Default Architecture and Bundled Font Set

**One sentence:** ship a hybrid backend with `Newstroke` as the
default `Manufacturing`-intent stroke backend, `ttf-parser`-driven
outline backends for `Annotation` / `Branding` / `Documentation` intents,
and bundle `Newstroke + Inter (Regular, Display) + IBM Plex Sans
Condensed + JetBrains Mono` as the default 5-font set under permissive
licenses (CC0 + SIL OFL 1.1).

**Architecture summary:**
1. Three-layer split per the user's framing — semantic model in the
   canonical IR, layout engine in `crates/engine/src/text/layout.rs`,
   glyph backends in `crates/engine/src/text/backends/`.
2. `RenderIntent` enum lives in the semantic model and gates backend
   selection. The default mapping: `Manufacturing → Stroke`,
   {`Annotation`, `Branding`, `Documentation`} → Outline, `UiPreview →
   GUI's own font system, not Datum's text engine`.
3. Backends implement `GlyphBackend` trait. `StrokeBackend` for
   Newstroke (and a future Hershey backend if needed). `OutlineBackend<F:
   FontParser>` for TTF/OTF, defaulting to `ttf-parser` as the parser.
4. Layout engine is intent-aware (knows which backend type to ask for)
   but glyph-internals-agnostic (operates on advance widths and bounding
   boxes uniformly).
5. `GeometryPostProcessor` is the single point where backend-native
   output (strokes or polygons) is converted to consumer-required form
   (triangles, Gerber payloads, DRC inputs). All consumers go through it;
   no consumer reaches into glyph internals.
6. Per-fab DRC profiles ship as JSON at `crates/engine/assets/fab_profiles/`.
   Default profile: JLCPCB-standard (1.0 mm height / 0.15 mm stroke /
   1:6 aspect). User can override per-design.
7. Font assets bundle into the engine binary via `include_bytes!`. No
   runtime asset discovery, no system-font-directory scanning.

**Library stack (every entry permissive):**
- `ttf-parser` — outline extraction (Apache-2.0 / MIT)
- `kurbo` — curve math, affine transforms (Apache-2.0 / MIT)
- `lyon_geom` — Bézier flattening (Apache-2.0 / MIT)
- `lyon_tessellation` — fill / stroke tessellation (Apache-2.0 / MIT)
- `cavalier_contours` — polygon offset for engineering-mode stroke
  expansion (MIT)
- `i_overlay` — boolean union (Apache-2.0)
- (existing) Newstroke vendored CC0 data, Phase 1's existing module.

**Font bundle (5 fonts, ~9 MB uncompressed full families, ~5 MB lean
Regular+Bold subset):**
- `Newstroke` — CC0 — engineering / silkscreen / copper text
- `Inter Regular` — SIL OFL 1.1 — technical sans (annotations, fab notes,
  title blocks)
- `Inter Display` — SIL OFL 1.1 — branding / nameplate / large headers
- `IBM Plex Sans Condensed` — SIL OFL 1.1 — dense annotation, refdes
  on tight footprints, BOM/PnP overlays
- `JetBrains Mono` — SIL OFL 1.1 — code-like text (timestamps, version
  hashes, machine-readable strings)

---

## Implementation Sequencing — Mapped to the User's 10-Step Roadmap

The user has a 10-step roadmap (referenced as "user's #1 through #10"
in the brief). The phase 2 research reduces risk for each as follows:

| Step | Description (paraphrased) | Risk reduction from Phase 2 | Status |
|------|---------------------------|------------------------------|--------|
| 1 | Semantic text model | Topic 4 trims the typography control set; Topic 6 specifies the `SemanticText` struct shape | Ready to spec |
| 2 | Layout engine (line breaking, anchors, transforms) | Topic 6 specifies `TextLayoutEngine` trait; Phase 1 covered keep-upright / mirror / multi-line semantics | Ready to spec |
| 3 | Stroke backend (Newstroke) | Phase 1 settled this; Phase 2 confirms `cavalier_contours` for offset operations | Ready (Phase 1 deliverable) |
| 4 | Outline backend (TTF/OTF) | Topic 2 settled the library stack (`ttf-parser` + `kurbo` + `lyon`); Topic 6 specifies the trait | Ready to spec |
| 5 | Render-intent gating | Topic 6 specifies the `RenderIntent` enum and backend selection | Ready to spec |
| 6 | Fab-constraint enforcement | Topic 3 settled the numeric thresholds and degradation policy | Ready to spec |
| 7 | Per-fab profiles | Topic 3 catalogued the major fabs' published minimums | Ready to author profile JSONs |
| 8 | Preview-vs-fabrication parity | Topic 6 specifies the single-path post-processor architecture | Ready to spec |
| 9 | Performance / caching | Topic 6 specifies cache keys and layers | Spec'd; benchmarking deferred |
| 10 | Variable-axis / custom font backend | **Out of Phase 2 scope.** Use `swash` if needed; defer evaluation | Not yet researched |

Step 10 is the only one that still needs research. Variable-axis font
support is a graceful extension of `OutlineBackend` (the underlying
parser handles it via `gvar` table) but the UX and the cache-key
implications need more thought. Defer to Phase 3 if/when needed.

---

## Risks and Open Questions

**Risk 1 — Cross-architecture determinism for outline geometry.**
`ttf-parser` is bit-deterministic, `kurbo`/`lyon` Bézier flattening is
**not** bit-deterministic across architectures (different float behavior
on aarch64 vs x86_64, different SIMD-coverage paths). The Phase 1 report
already noted Datum's existing engine accepts x86_64-linux as the golden
fixture architecture; outline-text geometry must adopt the same posture.
**Recommended action:** add a determinism test that reads the same TTF,
flattens at the same tolerance, and compares the output bytes against
a checked-in fixture. Run only on x86_64-linux. Document the
single-arch promise in `docs/CANONICAL_IR.md`.

**Risk 2 — Newstroke CC0 source provenance.**
Phase 1 identified the upstream CC0 source as Vladimir Uryvaev's
`http://vovanium.ru/sledy/newstroke`. This domain may not always be
reachable. **Recommended action:** snapshot the CC0 release into
`research/pcb-text-rendering/sources/newstroke-CC0-snapshot/` (with
SHA-256s) before starting the import work. Phase 2 web search confirmed
the upstream URL is live as of April 2026; mirror it now while it's
available.

**Risk 3 — `OUTLINE_FONT` import fidelity for KiCad boards using
non-bundled TrueType fonts.**
If a KiCad board was authored with `Times New Roman` (Microsoft
proprietary), Datum cannot bundle the source font. Phase 1 documented
the correct behavior (substitute with closest-bundled, document the
substitution). Phase 2 doesn't surface new risk here, but flags it
as the most-likely user complaint after launch.

**Risk 4 — Variable-axis fonts in design mode.**
Inter Variable, JetBrains Mono Variable, and IBM Plex are all available
as variable fonts (one file per family instead of one file per
weight/style combination). This saves ~5 MB of bundle but adds
complexity to the `OutlineBackend` (the `style` parameter has to map to
variation-axis values). **Recommended action:** ship static cuts (Regular,
Bold) in v1; evaluate variable-font support in Phase 3 as a binary-size
optimization, not a v1 feature.

**Risk 5 — `RenderIntent::Branding` is the most likely scope-creep
vector.**
Branding text is where users will ask for variable-weight, optical-size
sliders, custom curves, gradient fills, drop shadows, embossing, "make
my logo look like Apple's." Datum should explicitly say: "branding text
gets the same outline backend that annotation text uses; per-glyph
typographic effects are not in scope." Document this in the spec to
forestall the conversation.

**Open question — class-based text styles vs per-text styles.**
Topic 4 noted that Allegro/PADS use class-based style ("all refdes
texts use Engineering Sans 1.0 mm") rather than per-text style. This is
a UX decision the user owns. Datum's authoring API can support both;
the question is which is the default authoring affordance. Recommended
default: class-based (matches mil/aero workflows; scales better than
per-text); allow per-text override.

**Open question — how aggressively to enforce fab constraints.**
The "warn-and-clamp at DRC" recommendation works for most users. But
some users will want "auto-resize text to fab minimum at export time"
because they're authoring at design-time and don't want to think about
fab. Datum could expose this as a Gerber-export option (`--text-policy={
strict-warn, auto-snap, allow-as-authored }`). Decide before locking
the spec.

---

## Sources

**Phase 1 cross-reference (do not duplicate):**
- `/home/bfadmin/Documents/datum-eda/research/pcb-text-rendering/PCB_TEXT_RENDERING_RESEARCH.md`
  — covers the full KiCad pipeline, Newstroke license analysis, Hershey
  provenance, and the Datum-side current state.

**Industry text-engine survey (Topic 1):**
- [Altium PCB String Properties](https://www.altium.com/documentation/altium-designer/pcb-string-properties)
  — Altium's text-object dropdown for stroke vs TrueType.
- [Altium TrueType Fonts Preferences](https://www.altium.com/documentation/altium-designer/pcb-editor-true-type-fonts-preferences)
  — Altium's docs on TrueType slowing Gerber generation.
- [What font works best (element14)](https://community.element14.com/products/manufacturers/altium/f/forum/31606/what-font-works-best-and-where-is-the-default-font-selected)
  — community recommendation of Sans Serif stroke at 65 mil; TrueType
  printing at ~60% of authored height.
- [Allegro text formatting (Cadence Community)](https://community.cadence.com/cadence_technology_forums/f/pcb-design/22406/changing-text-size-and-font)
  — Allegro's design-parameter model.
- [DipTrace TrueType Fonts forum](https://diptrace.com/forum/viewtopic.php?t=14308)
  — DipTrace's official "convert text to stroke" warning.
- [LibrePCB issue #165](https://github.com/LibrePCB/LibrePCB/issues/165)
  — LibrePCB's tracking issue for arbitrary-font support.

**Outline-font Rust ecosystem (Topic 2):**
- [harfbuzz/ttf-parser](https://github.com/harfbuzz/ttf-parser) — the
  canonical parser; documented zero-allocation, zero-unsafe, stateless
  determinism contract.
- [ttf-parser on crates.io](https://crates.io/crates/ttf-parser)
- [linebender/kurbo](https://github.com/linebender/kurbo) — curve math.
- [Rust forum: state of font parsers](https://users.rust-lang.org/t/the-state-of-fonts-parsers-glyph-shaping-and-text-layout-in-rust/32064)
  — community consensus on parser/renderer split.
- [iShape-Rust/iOverlay](https://github.com/iShape-Rust/iOverlay) —
  best-in-class boolean ops; supports `FillRule::NonZero` for fonts.
- [lelongg/geo-clipper](https://github.com/lelongg/geo-clipper) —
  C++ Clipper binding; mature alternative.
- [jbuckmccready/cavalier_contours](https://github.com/jbuckmccready/cavalier_contours)
  — polyline offset; recently improved robustness (issue #66 fix).
- [cavalier_contours docs.rs](https://docs.rs/cavalier_contours/latest/cavalier_contours/)
- [lyon_tessellation docs](https://docs.rs/lyon_tessellation/latest/lyon_tessellation/)
- [nical/lyon (GitHub)](https://github.com/nical/lyon)

**Fab manufacturing constraints (Topic 3):**
- [JLCPCB: Minimum Silkscreen Text Size](https://jlc3dp.com/help/answers/detail/49-Minimum-Silkscreen-text-size)
  — 1.0 mm / 0.15 mm standard; 0.8 mm / 0.10 mm precision.
- [JLCPCB Character Design Specifications](https://jlcpcb.com/blog/technical-guidance-character-design-specifications)
- [JLCPCB Design Rules guide (Schemalyzer)](https://www.schemalyzer.com/en/blog/manufacturing/jlcpcb/jlcpcb-design-rules)
- [PCBWay: Silkscreen standard size](https://www.pcbway.com/helpcenter/Engineering_Questions/Silkscreen_standard_size.html)
  — 0.8 mm / 0.15 mm (0.18 mm preferred).
- [PCBWay Specifying SilkScreen](https://www.pcbway.com/blog/Engineering_Technical/Specifying_SilkScreen.html)
- [OSH Park guidelines](https://oshpark.com/guidelines)
- [OSH Park Eagle silkscreen docs](https://docs.oshpark.com/design-tools/eagle/modifying-silkscreen-layers/)
  — 5 mil = 0.127 mm minimum line width.
- [Sierra Circuits silkscreen kb](https://www.protoexpress.com/kb/silkscreen/)
  — 25 mil text height, 4 mil stroke width minimums.
- [Sierra Circuits Class 3 standards](https://www.protoexpress.com/kb/ipc-class-3-pcb-design-and-manufacturing-standards/)
- [Sierra Circuits DFM rules](https://www.protoexpress.com/kb/dfm-rules/)
- [EasyEDA Place Text](https://prodocs.easyeda.com/en/schematic/place-text/)

**UX and authoring controls (Topic 4):**
- [KiCad PCB text line spacing forum thread](https://forum.kicad.info/t/pcb-text-line-spacing-feature-request/38328)
- [Altium PCB Multi-line Text](https://www.altium.com/documentation/altium-designer/nfs-15-1multi-line-pcb-text-support-ad?version=15.1)
- [KiCad multi-line silkscreen text leans](https://forum.kicad.info/t/multi-line-silk-screen-text-leans-linux/3496)
- [KiCad library convention F5.1 silkscreen](https://klc.kicad.org/footprint/f5/f5.1.html)

**Font licensing and bundling (Topic 5):**
- [SIL Open Font License](https://openfontlicense.org/) — license text.
- [OFL FAQ](https://openfontlicense.org/ofl-faq/) — bundling and
  embedding guidance.
- [Inter on GitHub (rsms/inter)](https://github.com/rsms/inter)
- [IBM Plex on GitHub (IBM/plex)](https://github.com/IBM/plex)
- [JetBrains Mono](https://github.com/JetBrains/JetBrainsMono) — OFL-1.1
  for the typeface, Apache-2.0 for the build tooling.
- [Atkinson Hyperlegible](https://www.brailleinstitute.org/freefont)
- [Fontobene NewStroke license note (issue #3)](https://github.com/fontobene/fontobene-fonts/issues/3)
  — independent confirmation of Newstroke CC0 status.

**Architectural decisions:**
- `/home/bfadmin/Documents/datum-eda/CLAUDE.md`
- `/home/bfadmin/Documents/datum-eda/docs/gui/M7_IMP_014_IMPORTED_TEXT_NORMALIZATION_BRIEF.md`
- `/home/bfadmin/Documents/datum-eda/docs/CANONICAL_IR.md`
- `/home/bfadmin/Documents/datum-eda/specs/ENGINE_SPEC.md`
- `~/.claude/projects/-home-bfadmin-Documents-datum-eda/memory/feedback_no_copyleft_integration.md`
  — the GPL-class hard-exclusion rule applied throughout this report.

