# PCB Text Rendering — Research and Implementation Strategy

> **Driver**: `M7-IMP-014` Imported KiCad Text Normalization brief
> (`docs/gui/M7_IMP_014_IMPORTED_TEXT_NORMALIZATION_BRIEF.md`).
> **Scope**: How Datum EDA should generate imported PCB text geometry without
> a KiCad runtime dependency, in a way that also serves later native-project
> authored text and schematic text without a re-do.
> **Tone**: Companion to `research/airwire-rendering/`,
> `research/copper-rendering/`, and `research/ipc-compliance/`. Same
> structure, source-citation discipline, and depth.
> **Attribution**: Per `CLAUDE.md`, no `Co-Authored-By` tags, no "generated
> by" notes, no AI attribution markers anywhere in this document.

## Executive Summary

- **The reference implementation is settled.** Every credible open-source PCB
  tool ships a stroke-font implementation derived from Hershey vector data
  (KiCad: Newstroke; Horizon EDA: OpenCV-style Hershey table; LibrePCB:
  FontoBene `newstroke.bene`). The only relevant divergence is whether the
  format permits arc segments (FontoBene yes, Hershey/Newstroke no) and how
  Unicode is extended (Newstroke ships full CJK, Horizon ships a Latin core
  with hand-curated extensions). Datum should implement a Newstroke-equivalent
  stroke font generator and not invent a new format.
- **Recommended architecture (one line)**: a single Datum-owned `pcb-text`
  module exposing `glyph_strokes(codepoint, style) → Vec<Polyline>` and
  `layout_text_block(text, attrs) → Vec<PositionedGlyph>`, fed by a vendored
  CC0 Newstroke (or Hershey-rederived) glyph table, producing stroke geometry
  by default and lazily tessellating to polygons only at export / DRC time.
- **License-risk verdict on Newstroke vendoring**: **safe**. Newstroke was
  relicensed by its author (Vladimir Uryvaev) to **CC0** by direct grant to
  KiCad, and KiCad's bundled copy carries the GPL-2 wrapper *as a project
  policy choice*, not as a constraint imposed by the underlying glyph data.
  The CC0 grant is independently confirmed by the KiCad developer mailing
  list and by the authors of the existing `kicad_newstroke_font` Rust port.
  Datum can vendor the Newstroke glyph table under CC0 without acquiring any
  GPL surface area, **provided** the import is done from the upstream CC0
  source files and not by transcription from KiCad's GPL-headered C++
  release. The clean path is the upstream Vladimir/CC0 source (or
  re-encoding from the public-domain Hershey JHF data); the dirty path is
  copy-translating KiCad's `newstroke_font.cpp`. Pick the clean path.
- **Output-format verdict**: **strokes are the canonical output; polygons are
  derived on demand**. Stroke-font glyphs are natively sets of polylines;
  Datum's renderer is GPU-instanced anyway and prefers thick line segments;
  Gerber `G36/G37` polygon-fill is only one of three valid Gerber
  representations for text, all of which are reachable from a stroke
  representation. A polygon-only design loses information (the centerline
  and stroke-width separation) that DRC and silkscreen-minimum-width checks
  need. Generate strokes, tessellate to polygons in one place per export
  consumer.
- **`render_cache` is not a parity oracle in the sense the brief assumes —
  but it is still useful, with caveats.** KiCad emits `render_cache` polygon
  glyphs **only for non-default (TrueType / `OUTLINE_FONT`) text**; for the
  default Newstroke stroke font there is no `render_cache` to compare
  against. KiCad issue #17666 confirms that even when the cache is present,
  KiCad itself ignores it on load (re-renders from the live font). This
  flips the brief's framing: cache-present boards are exactly the ones
  Datum *cannot* match by stroke-font fidelity alone, because the source
  of truth is a TrueType file Datum doesn't have. The pragmatic
  interpretation: parity-test Newstroke output against KiCad-rendered
  PNG/SVG snapshots (not against `render_cache`); use `render_cache`
  only as a polygon-shape oracle when the source font is TrueType, with
  the understanding that exact match requires shipping the same TrueType.
- **`DOA2526` vs `datum-test` quality gap is explained by font choice, not
  by Datum's code.** If `DOA2526` looks credible only because the source
  used a TrueType font and KiCad cached the polygons, the fix is not to
  invent new fallback geometry — it is to (a) implement Newstroke
  faithfully so all *default-font* boards look right, and (b) explicitly
  drop or substitute non-default fonts in the imported scene with a
  documented warning, rather than silently degrading them.
- **Keep-upright is the easiest correctness bug to ship.** KiCad's rule is:
  normalize the absolute text rotation, and if it lies in `(90°, 270°)`
  (i.e. the right side of the page would be reading upside-down), add 180°
  and swap H/V justification. Datum's existing `kicad_keep_text_upright_degrees`
  in `crates/gui-protocol/src/lib.rs:2700` matches this rule for the angle
  but does *not* swap justification, which will visibly mis-place rotated
  reference designators on bottom-side parts.
- **The `BoardText` struct is under-specified.** The current canonical IR
  carries `text / position / rotation / layer / height_nm / stroke_width_nm`
  (`crates/engine/src/board/board_types.rs:88`). To round-trip imported
  KiCad text faithfully and to drive a single generator, the struct must
  also carry `h_justify`, `v_justify`, `mirrored`, `keep_upright`,
  `italic`, `bold`, `multiline_allowed`, and `line_spacing_ratio`. These
  are all data already available in `gr_text` / `fp_text` s-expressions
  and are ignored today.
- **Biggest implementation risk**: **Unicode coverage drift between Newstroke
  and Datum's silkscreen export's hand-rolled ASCII font**. Today
  `crates/engine/src/export/silkscreen.rs:57` returns
  `ExportError::UnsupportedSilkscreenTextCharacter` for anything outside
  `[0-9A-Z .-_/+]`. A Newstroke-equivalent ships ~65,000 glyphs including
  full CJK; the silkscreen exporter must converge onto the same generator
  the GUI uses, otherwise import-and-export round-trips will silently
  truncate text and the dual-path representation lives on under a different
  name.
- **Effort estimate for Phase 1 (vendored font + basic LTR rendering)**:
  3–5 engineering days. The bulk is data prep (downloading and re-encoding
  the upstream CC0 Newstroke source into a Rust `static [&str; N]` table
  with a build-time validation step), wiring the existing
  `glyph_strokes()` call sites to the new module, and fixture-backed tests
  on `datum-test`. Phases 2–5 (layout, mirror/keep-upright, italic/bold,
  stroke→polygon, render-cache parity) are 1–2 weeks. Phases 6–7
  (TrueType for `OUTLINE_FONT` imports, native authored reuse) are
  separately scoped.

## Problem Definition

The `M7-IMP-014` brief sets the following constraints (quoted verbatim):

> Datum must generate final text geometry from text semantics, not from
> optional cached source geometry.

> KiCad `render_cache` may be used as a debug oracle, comparison surface,
> or test reference, but not as final rendered truth.

> cache-present and cache-absent boards must land on the same
> Datum-owned text geometry generation path.

> The accepted implementation may output:
> - polygon geometry
> - stroke/polyline geometry
> But it must be one Datum-owned geometry generator, not a
> representation split.

> If Datum needs KiCad-compatible stroke-font data or glyph rules, those
> must be vendored into the repo and used internally.

The acceptance criteria (also quoted):

> - imported KiCad text no longer uses `render_cache` as final render truth
> - cache-present and cache-absent fixtures go through the same Datum-owned
>   text generation path
> - no KiCad runtime dependency is introduced
> - `datum-test` and `DOA2526` no longer differ in visible text quality
>   solely because one fixture had cache polygons and the other did not
> - fixture-backed tests prove representation-invariant imported text
>   behavior

### Current Datum-side state of play

- `crates/engine/src/board/board_types.rs:88` — `BoardText { uuid, text,
  position, rotation, layer, height_nm, stroke_width_nm }`. No justify,
  mirror, keep-upright, italic, bold, multiline.
- `crates/engine/src/export/silkscreen.rs:13` — `render_silkscreen_text_strokes`
  uses an inline ASCII-only stroke table (`glyph_strokes()` at line 57)
  on a 3×5 cell grid. Returns
  `ExportError::UnsupportedSilkscreenTextCharacter` for anything else,
  including spaces (which it handles) but not lowercase (which is
  uppercased).
- `crates/engine/src/import/kicad/skeleton.rs:612` — `gr_text` import path
  reads only `text / position / layer / rotation`; hard-codes
  `height_nm: 1_000_000` (1.0 mm) and `stroke_width_nm: 100_000` (0.1 mm).
  Footprint text justify is parsed in `gui-protocol` but not threaded
  back to the engine.
- `crates/gui-protocol/src/lib.rs:113` — `ComponentTextPrimitive` carries
  a `cached_polygons: Vec<Vec<PointNm>>` field. `lib.rs:3168` decides
  per-text whether to emit a polygon-glyph branch (when cached) or fall
  back to `component_text_stroke_graphics()` (Datum's stroke generator)
  when not cached. **This is precisely the dual-path the brief targets
  for removal.**
- `crates/gui-protocol/src/lib.rs:2700` — `kicad_keep_text_upright_degrees`
  flips angles in `(90°, 270°)` by 180°, but does not swap H/V justify
  or mirror flags as KiCad does.
- `crates/gui-render/src/lib.rs:4163` — renderer reads
  `text.cached_polygons` and draws them as filled polygons when present;
  otherwise falls back to the Datum stroke geometry. **The renderer is
  the consumer that has to stop seeing two representation classes.**

The product-semantic invariant from `M7_RENDER_SEMANTIC_CONTRACT.md`:

> any board-to-board difference caused only by representation presence or
> absence is a regression, not an acceptable fixture quirk

is what the brief's acceptance criteria operationalize.

## KiCad's Text Rendering Pipeline (Reference)

KiCad is the de-facto reference because the brief targets KiCad-imported
boards and because KiCad is the only open-source PCB tool whose stroke-font
implementation has been published, peer-reviewed, and used in commercial
production for 15+ years.

### Newstroke Font

**What it is**: a derived stroke font originally created by Vladimir
Uryvaev (2010), seeded from the Hershey font family. KiCad 5.x adopted
Newstroke as the default and only stroke font for both schematic and
board text; KiCad 6.x extended its Unicode coverage to include CJK
(via Adobe's Source Han Sans glyph polygons re-encoded as stroke
approximations); KiCad 7.x added `OUTLINE_FONT` for TrueType but kept
Newstroke as the default.
([KiCad newstroke_font.cpp Source File](https://docs.kicad.org/doxygen/newstroke__font_8cpp_source.html),
[Newstroke 001.000 Fonts Free Download](https://www.onlinewebfonts.com/download/e2394cb7d30696bdf001b6028851e91a))

**Where the data lives**: `common/font/newstroke_font.cpp` in the KiCad
repo, declared in `common/font/newstroke_font.h` as
`extern KICOMMON_API const char* const newstroke_font[];` and
`extern KICOMMON_API const int newstroke_font_bufsize;`. Each entry is
a single C string encoding one glyph in the original Hershey ASCII
encoding (described below).
([newstroke_font.h Source File](https://docs.kicad.org/doxygen/newstroke__font_8h_source.html))

**Glyph count and file size**: `newstroke_font_bufsize` is in the
~65,000 range; the array contains ~65,000+ glyphs covering Latin,
extended Latin, Greek, Cyrillic, IPA, math, arrows, geometric shapes,
Hangul, Hiragana, Katakana, and CJK Unified Ideographs. The
`newstroke_font.cpp` source file is roughly 65,700 lines long (one
glyph per line plus boilerplate). At runtime memory cost is roughly
2–4 MB depending on whether the data is held as `const char*` (pointer
table + flat string heap) or copied into per-glyph `std::vector`.

**Data structure (the Hershey ASCII encoding)**: each glyph string
follows the original 1967 Hershey convention. The first two characters
encode the glyph's **left and right pen-position bounds** (which set
the advance width) using the offset `c - 'R'`, where `'R'` is the zero
point. The remaining characters are pairs of `(x, y)` strokes, again
using `c - 'R'` so the printable ASCII range covers `±31` units. A
literal `' R'` (space followed by `R`) means "pen up — start a new
sub-stroke". The standard glyph cell is 21 units tall, with the
baseline historically offset by `-8`.

For example, the Newstroke glyph for `'I'` is the string `"H\\MWMV "`,
which decodes to: left bound `H - R = -10`, right bound `\\ - R = +10`
(advance width = 20), single sub-stroke from `(M-R, M-R) = (-5, -5)`
to `(W-R, V-R) = (+5, +4)`. (The above is illustrative, not the
literal Newstroke entry — Newstroke entries are typically several
sub-strokes per glyph for serif and stem variations.)

Horizon EDA's vendored `hershey_glyphs[]` array in
`/research/horizon-source/src/canvas/hershey_fonts.cpp:56` shows the
identical encoding convention (`s[0] - 'R'` for left bound,
`s[1] - 'R'` for right bound, then 2-char `(x, y)` pairs with `' R'`
as pen-up).

**Could it be vendored as-is into a Rust crate?** Yes — and it has
been. The `kicad_newstroke_font` crate (kicad-rs/kicad_newstroke_font
on GitHub) ships the raw KiCad v6 Newstroke data as a Rust crate with a
layered `LICENSE.txt` covering GPL-2 (KiCad wrapper), MIT (CJK
extensions), and OFL (Source Han Sans). For Datum's purposes this
existing crate is **not directly usable** because the GPL-2 wrapper
license forces Datum into GPL-2 (per the project's
`feedback_no_copyleft_integration` policy: GPL-class deps are
subprocess-only). The clean path is to vendor Newstroke from its
**upstream CC0 source** rather than from the KiCad C++ release that
re-licensed it under GPL-2 by inclusion.
([kicad-rs/kicad_newstroke_font](https://github.com/kicad-rs/kicad_newstroke_font),
[crates.io: hershey](https://crates.io/crates/hershey))

### STROKE_FONT class and algorithm

KiCad's `STROKE_FONT` class lives in `libs/kimath/src/font/stroke_font.cpp`
and `libs/kimath/include/font/stroke_font.h`. It inherits from `KIFONT::FONT`
(`libs/kimath/include/font/font.h`).

**Class shape** (`stroke_font.h`):
- `m_glyphs: std::vector<std::shared_ptr<STROKE_GLYPH>>` — per-glyph stroke
  list, decoded once at load.
- `m_glyphBoundingBoxes: std::vector<BOX2D>` — pre-computed glyph bounds.
- `m_maxGlyphWidth: double` — used for tab stops.
- Public: `STROKE_FONT()`, `IsStroke() → true`, static
  `LoadFont(wxString)`, `GetInterline(double aGlyphHeight)`,
  `GetTextAsGlyphs(BOX2I*, std::vector<std::unique_ptr<GLYPH>>*, ...)`,
  `GetGlyphCount()`, `GetGlyph(unsigned)`, `GetGlyphBoundingBox(unsigned)`.

**Key constants** (mostly in `font.h` and `stroke_font.cpp`):
- `STROKE_FONT_SCALE = 1.0 / 21.0` — scale factor from raw Hershey unit
  (21 units per em) to normalized glyph height of 1.0.
- `FONT_OFFSET = -8` — Y baseline shift (units), legacy compatibility
  with the original KiCad 4.x stroke renderer.
- `ITALIC_TILT = 1.0 / 8` — italic shear factor; for italic glyphs each
  stroke X coordinate is biased by `dy * ITALIC_TILT`. (Equivalent to
  ~7.125° shear, not the typographic 12–15°.)
- `INTER_CHAR = 0.2` — extra space between glyphs as a fraction of em
  height. Adds to the per-glyph advance width.
- `LEGACY_FACTOR = 0.9583` — `GetInterline` returns
  `aFontMetrics.GetInterline(aGlyphHeight) * LEGACY_FACTOR` to keep
  visual line height matched against KiCad 4.x corpora.
- Tab expansion: aligned to 4-character boundaries.
- Superscript / subscript: scaled to `0.8 ×` base size; superscript
  baseline shift `+0.35 × em`; subscript shift `-0.15 × em`.
- Bold rendering: stroke pen width is doubled (no glyph reshape — same
  centerlines, thicker pen).

**`LoadNewStrokeFont()` algorithm**:
1. For each entry in the `newstroke_font[]` array:
2. Read the first two characters as the glyph's `(left_bound, right_bound)`
   in raw units (`c - 'R'`).
3. Walk subsequent characters in pairs `(x_char, y_char)`. The pair
   `(' ', 'R')` is interpreted as **pen-up** and starts a new sub-stroke
   (polyline). All other pairs append `(x - 'R', y - 'R')` as a vertex
   to the current sub-stroke.
4. Apply `FONT_OFFSET` to each Y to move the baseline.
5. Apply `STROKE_FONT_SCALE` to convert to em-normalized coordinates.
6. Compute per-glyph bounding box.
7. Store under thread-safe mutex (font is shared across viewports).

**`GetTextAsGlyphs()` algorithm**:
1. Walk codepoints of the input text.
2. Lookup glyph index = `(int)c - ' '` (this is the convention from
   the original Hershey table; codepoints below `' '` are control
   characters handled separately, and high codepoints index directly
   into the Unicode-extended portion of the Newstroke array).
3. Apply tab/space/newline rules.
4. For each glyph: clone the glyph's stroke list, apply
   `(scale, italic_shear, position, rotation)` transform, append to the
   output list.
5. Advance cursor by `(glyph_bbox.width + INTER_CHAR) × em_size`.
6. Track running bounding box across all glyphs for layout reuse.

### Multi-line layout

KiCad supports `\n` in text strings when `m_Multiline = true` (which is
the default per `text_attributes.cpp:24-39`). The layout walks lines:

1. Split text on `\n`.
2. For each line, compute its width via `GetTextAsGlyphs()` (no draw)
   to enable per-line justification.
3. Apply per-line vertical offset:
   `y_offset = line_index × GetInterline(em_size) × m_LineSpacing`.
   Default `m_LineSpacing = 1.0`. `GetInterline` returns
   `~0.96 × em_size` per the `LEGACY_FACTOR`.
4. Apply per-line horizontal alignment based on `m_Halign` (left =
   anchor at line start; center = anchor at line midpoint; right =
   anchor at line end).
5. Apply global vertical alignment to the *full multi-line block*
   based on `m_Valign` and total block height.

The mirror flag inverts the line-stacking direction (line 0 at bottom
instead of top) — see Horizon's equivalent `lineskip *= -1` in
`util/text_renderer.cpp:38`.

### Justification, anchor, and mirror semantics

KiCad's enums (`eda_text.h`):

```
enum GR_TEXT_H_ALIGN_T { GR_TEXT_H_ALIGN_LEFT, GR_TEXT_H_ALIGN_CENTER,
                         GR_TEXT_H_ALIGN_RIGHT };
enum GR_TEXT_V_ALIGN_T { GR_TEXT_V_ALIGN_TOP, GR_TEXT_V_ALIGN_CENTER,
                         GR_TEXT_V_ALIGN_BOTTOM };
```

That's nine combinations. The defaults in `TEXT_ATTRIBUTES` are
`(CENTER, CENTER)`. The `m_Halign × m_Valign` pair is interpreted with
respect to the **anchor point** stored in the text's `m_Pos`:

- `H_ALIGN_LEFT`: anchor is the leftmost point of the (rendered, italic-
  sheared, mirrored) bounding box.
- `H_ALIGN_CENTER`: anchor is the bbox horizontal midpoint.
- `H_ALIGN_RIGHT`: anchor is the rightmost point.
- `V_ALIGN_TOP`: anchor is at the bbox's *top* — i.e. above the visible
  glyphs (this is the reading-line-top, not the cap-line).
- `V_ALIGN_CENTER`: anchor is the vertical midpoint.
- `V_ALIGN_BOTTOM`: anchor is the baseline.

**Mirror interaction**: in `eda_text.cpp:GetTextBox`, when
`m_Mirrored = true`, the bbox X-extent calculation flips:

```
case GR_TEXT_H_ALIGN_LEFT:
    if( IsMirrored() )
        bbox.SetX( bbox.GetX() - ( bbox.GetWidth() - italicOffset ) );
```

i.e. mirror swaps the visual meaning of LEFT and RIGHT alignment about
the anchor. (KiCad does **not** keep `m_Halign` and `m_Mirrored` as
independent — there's a "reflective" renormalization the importer needs
to be aware of.)

### Mirror semantics

For board text (`PCB_TEXT`), the mirror flag is set when the text is on
a back-copper or back-silkscreen layer, or when the user explicitly
ticks "Mirror". Implementation: each glyph's local-X is negated
(reflection about the anchor's vertical axis) **after** italic shear
and **before** rotation. Y is not touched; the resulting glyphs read
reversed when viewed from the *front* of the board, which is the
correct convention for back-side artwork (the photoplotter draws the
board as if looking at the bottom from below).

### Keep-upright behavior

The flagship surprising-but-important rule. `PCB_TEXT::KeepUpright()`
in `pcb_text.cpp` (lines 369–387 per the Doxygen source), as confirmed
by the KiCad source listing:

```
EDA_ANGLE newAngle = GetTextAngle();
newAngle.Normalize();
bool needsFlipped = newAngle >= ANGLE_180;
```

When `needsFlipped`, KiCad **(a) inverts horizontal and vertical
justification values, (b) adds 180° to the angle, (c) re-normalizes**,
forcing the angle into `[-90°, +90°]`.

The `GetDrawRotation()` method (lines 204–223) summarizes this in the
comment `"Keep angle between ]-90..90] deg. Otherwise the text is not
easy to read"`.

`KeepUpright` is gated on whether the text has a parent footprint
(line 763 of `pcb_text.cpp`): only footprint text exposes the
`KeepUpright` property. Free `gr_text` is left at the user's
authored angle.

**Datum's existing implementation has a partial bug**:
`crates/gui-protocol/src/lib.rs:2700` correctly normalizes and flips
the angle to `(-90°, +90°]` but **does not invert H/V justification**.
For a footprint placed at 180° on the back side with a left-justified
reference, this will visibly mis-anchor the text by the full text-block
width — invisible on `R1` (1-char block), severely visible on
`U17_FPGA_BANK_3` (20-char block). The fix has to land alongside the
generator extension, not inside the existing helper.

### `render_cache` format

KiCad's parser expects (`pcb_io_kicad_sexpr_parser.cpp:parseRenderCache`):

```
(render_cache <text> <angle>
  (polygon
    (pts
      (xy <x> <y>)
      ...
    )
  )
  ...
)
```

- `<text>` echoes the source text string (so the cache can be
  re-validated against drift).
- `<angle>` is a degrees double.
- Each `(polygon ...)` is one **filled outline glyph** — typically one
  polygon per visible glyph component (a `B` produces three polygons:
  the outer outline plus two inner counters, with even-odd fill).
- Points are in **board coordinates** (i.e. the cache is post-transform,
  not glyph-local).

**When KiCad emits `render_cache`**: per `eda_text.cpp:GetRenderCache`,
the cache is built **only when `aFont->IsOutline()` is true** — i.e. for
TrueType / `OUTLINE_FONT` text. Stroke-font (Newstroke) text never
generates a `render_cache`. (KiCad issue #17666 confirms this and
notes that KiCad doesn't actually use the cache on load — it
re-renders from the live font.)
([gr_text render_cache doesn't really do anything (#17666)](https://gitlab.com/kicad/code/kicad/-/issues/17666))

**Implication for `M7-IMP-014`**:
- The `datum-test` fixture's text is presumably *all default Newstroke* →
  no `render_cache` in the source file → Datum's importer has no choice
  but to synthesize from semantics. ✓
- The `DOA2526` fixture's text presumably uses a *non-default font* →
  `render_cache` polygons present → Datum's current importer takes the
  cached-polygon branch and produces high-fidelity geometry "for free".
- The brief's framing — "one path serves both" — therefore means **drop
  the cached-polygon branch**. The `DOA2526` text will then render
  through Newstroke (or through a TrueType fallback if Datum implements
  one in Phase 6), and the visible quality difference between fixtures
  collapses because both are now Datum-synthesized.
- Whether `DOA2526`'s Datum-synthesized text *exactly matches* its
  KiCad-cached polygons depends on whether Datum can resolve the same
  TrueType file. Phase 1–5 should accept that imported TrueType text
  renders in the Datum default font (Newstroke) with a logged downgrade
  warning; Phase 6 introduces optional TrueType resolution.

### TrueType / OUTLINE_FONT path (KiCad 7+)

KiCad's `KIFONT::OUTLINE_FONT` wraps FreeType. Its `GetTextAsGlyphs`
calls FreeType to load the font, walks each character to obtain
outline contours (Bézier curves), tessellates the curves to
polygons (KiCad uses a fixed quadratic-Bézier subdivision depth), and
returns `OUTLINE_GLYPH` objects. The renderer fills these polygons.

For Datum:
- **Out of scope for `M7-IMP-014`** per the brief ("imported KiCad PCB
  text only" with the default font). Phase 6 can add TrueType via
  the `ttf-parser` Rust crate (Apache-2.0 / MIT) and `lyon` for
  Bézier-to-polygon tessellation.
- **In-scope risk**: `DOA2526`'s `render_cache` polygons exist exactly
  because they *aren't* Newstroke. Phase 1–5's Newstroke output for
  `DOA2526` will not exactly match the cached polygons — that's
  expected and acceptable per the brief, but should be documented in
  the test fixture so future contributors don't try to chase the
  parity.

### Stroke thickness model

KiCad's text has two independent geometric parameters:

- `m_Size` (`VECTOR2I`) — width and height of one em cell, in
  internal nm. Default `DEFAULT_SIZE_TEXT = 50 mils = 1.27 mm`.
- `m_StrokeWidth` (`int`) — pen width in nm. When `m_StrokeWidth = 0`
  KiCad applies an automatic ratio derived from text height:
  the **default thickness:height ratio is ~15%** (`m_StrokeWidth =
  GetPenSizeForBold/Normal(textHeight)` returns
  `textHeight × DEFAULT_PEN_RATIO` where `DEFAULT_PEN_RATIO ≈ 0.152`).
  Bold doubles this.

The Newstroke glyph centerlines are scale-invariant; the visible glyph
*body* width grows linearly with `m_StrokeWidth`. A 1.0 mm-tall
character with a 0.15 mm stroke pen is the canonical KiCad default; a
0.3 mm pen on the same glyph reads as a heavy bold; a 0.05 mm pen reads
as laser-marker thin.

**Datum's existing default** in `crates/engine/src/import/kicad/skeleton.rs:635`
is `height_nm: 1_000_000` (1.0 mm) with `stroke_width_nm: 100_000`
(0.1 mm) → 10% ratio. That's slightly thinner than KiCad's 15%
default, which is why imported text in Datum may currently look
"spindlier" than the same board in KiCad. Adopting the 15% default
when `stroke_width_nm` is absent would be a low-risk parity fix.

### `gr_text` / `fp_text` / `Reference` / `Value` semantics

| Token            | Owner       | Position frame      | Visibility default      | Mirror behavior            |
|------------------|-------------|---------------------|-------------------------|----------------------------|
| `gr_text`        | Board       | Board absolute      | Always shown            | Per author                 |
| `fp_text user`   | Footprint   | Footprint local     | Always shown            | Inherits footprint side    |
| `fp_text reference` | Footprint | Footprint local     | Shown unless `hide`     | Inherits + keep-upright    |
| `fp_text value`  | Footprint   | Footprint local     | Often `hide`            | Inherits + keep-upright    |
| `property "Reference"` | Footprint (KiCad 7+) | Footprint local | Shown unless `(hide yes)` | Inherits + keep-upright    |
| `property "Value"` | Footprint (KiCad 7+) | Footprint local  | Often `(hide yes)`      | Inherits + keep-upright    |

KiCad 7 transitioned `fp_text reference` and `fp_text value` to the
`property` syntax. Both syntaxes appear in real-world boards depending
on save format version.

**What the geometry generator needs to know about each**:
- For `gr_text`: rotation is absolute, mirror is author-set.
- For `fp_text` / `property`: rotation = `footprint_rotation +
  local_rotation` then **keep-upright** if the property is set; mirror =
  XOR of footprint side and local mirror flag.
- For `Reference` / `Value`: same as `fp_text`, plus respect
  `(hide yes)` (don't generate any geometry).

Datum's import path already handles much of this in
`crates/gui-protocol/src/lib.rs` around lines 3140–3260, but the
mirror-on-bottom-side logic is only implicit (it assumes the renderer
will see "this layer is `B.SilkS`" and flip), and the keep-upright
justification swap is missing.

### Newstroke Unicode coverage and gaps

Newstroke's array indexes by codepoint with the offset `c - ' '` for the
ASCII portion (so glyph[33] is `'!'`, glyph[65] is `'A'`, etc.) and
extends to a flat Unicode codepoint mapping for higher ranges. From the
KiCad source the array contains roughly **65,500+ glyph entries**,
covering:

- U+0020 – U+007E ASCII printable.
- U+00A0 – U+00FF Latin-1 Supplement (full).
- U+0100 – U+017F Latin Extended-A (full).
- U+0180 – U+024F Latin Extended-B (good but not exhaustive).
- U+0370 – U+03FF Greek (full, with stroke approximations).
- U+0400 – U+04FF Cyrillic (full).
- U+1F00 – U+1FFF Greek Extended (partial).
- U+2000 – U+27BF general punctuation, math, arrows (good coverage).
- U+3000 – U+30FF CJK Symbols, Hiragana, Katakana (full).
- U+3400 – U+4DBF CJK Unified Ideographs Extension A (partial).
- U+4E00 – U+9FFF CJK Unified Ideographs (substantial; not all glyphs).
- U+AC00 – U+D7AF Hangul Syllables (partial).
- Selected emoji (e.g. U+1F384 Christmas Tree per Horizon's table).

**Gaps**: Arabic, Hebrew, Devanagari, Thai, Tibetan, Ethiopic — none.
Bidirectional / RTL not modeled. Combining marks not supported (so
diacritics that exist as precomposed codepoints work; combining ones
don't).

**Fallback strategy** (Horizon's, see
`/research/horizon-source/src/util/text_data.cpp:298`): unknown
codepoints render as glyph index `870` (a "palm tree" glyph used as
the missing-glyph indicator). KiCad uses an empty glyph for
out-of-range codepoints, which silently swallows the character. Datum
should pick one — recommendation: emit a Tofu-like outlined box with
a dotted center to make missing glyphs visible (matches typographic
convention) **and** emit a one-time warning per unique missing
codepoint to the engine diagnostic stream.

## Industry Comparison

### Altium Designer

Altium's PCB string objects support **both** stroke and TrueType:

- **Stroke font** is the historic default ("Default" font in the
  Properties panel). It's a simple proprietary vector font derived
  from a Hershey-style stroke set — single-stroke polylines with no
  arc support, very similar in appearance to Newstroke.
- **TrueType / OpenType** fonts are picked from
  `\Windows\Fonts\` and rendered as filled outlines. Altium warns the
  user that TT fonts slow down vector output (Gerber) generation
  because every glyph contour has to be tessellated to polygons.
- For Gerber export, Altium converts text to "simple polygons (a
  series of straight lines and arcs)". For stroke fonts that's a
  centerline polyline rendered with the stroke pen as round-cap
  flashes (`G75 D03` aperture flash with a circular aperture stepped
  along the stroke); for TT it's `G36/G37` polygon fill.
- Per-string mirror is independently authored (not implied by layer
  side).
- Justification is the same 9-combination (LEFT/CENTER/RIGHT ×
  TOP/CENTER/BOTTOM) matrix.
- Altium does **not** ship "keep-upright" — the user manages text
  rotation manually per ref-des.

The Altium recommendation in their own docs is "use Default stroke font
on copper layers and silkscreen for Gerber-friendliness; use TrueType
only for marketing-style logo text" — which validates Datum's choice
to default to stroke.
([Altium Text Documentation](https://www.altium.com/documentation/cstu/text-0),
[Altium PCB String Properties](https://www.altium.com/documentation/altium-designer/pcb-string-properties),
[Altium TrueType Fonts Preferences](https://www.altium.com/documentation/altium-designer/pcb-editor-true-type-fonts-preferences))

### Eagle / Fusion 360 Electronics

Eagle uses its own simple vector "plotter-stroke" font (all-line
segments, no arcs, no italic, no bold). It is **not** Hershey-derived;
it was hand-authored by Cadsoft in the 1990s. The font's appearance is
notoriously basic — the Eagle community has built ULPs (User
Language Programs) like `hershey-text.ulp` to substitute Hershey
glyphs for finer-looking silkscreen.
([hershey-text-eagle](https://github.com/nallison/hershey-text-eagle))

Eagle's CAM Processor "always draws texts with Vector font" regardless
of the screen-display font, so the `.brd` file's authored vector text
is what the photoplotter actually receives. Justification is supported
in the same 9-combination shape. Mirror is implicit when text sits on a
back-side layer (`bPlace` etc.).

The Eagle font is *not* a candidate for Datum vendoring — the source
data isn't published as stroke tables, and the visual quality is
worse than Newstroke. Use it only as a "this is what users coming
from Eagle expect" reference point.
([Eagle Help: TEXT](https://web.mit.edu/xavid/arch/i386_rhel4/help/93.htm))

### Horizon EDA (with source pointers from `/research/horizon-source/`)

Horizon ships a vendored Hershey font derived from OpenCV's
`cvImgProc` font tables (BSD-3-Clause, not GPL). Source pointers:

- `/research/horizon-source/src/canvas/hershey_fonts.cpp` — the raw
  glyph table. ~870 glyph entries, mapped via
  `hershey_glyphs[INDEX]` where each entry is an ASCII string in the
  Hershey encoding (`s[0] - 'R'` for left bound, `s[1] - 'R'` for
  right bound, then 2-char `(x, y)` pairs with `' R'` as pen-up).
  Smaller than Newstroke (no CJK), but covers Latin, Greek, math,
  and selected symbols.
- `/research/horizon-source/src/util/text_data.hpp:9` — `Font` enum
  with 12 styles: `SMALL`, `SMALL_ITALIC`, `SIMPLEX`, `COMPLEX_SMALL`,
  `COMPLEX_SMALL_ITALIC`, `DUPLEX`, `COMPLEX`, `COMPLEX_ITALIC`,
  `TRIPLEX`, `TRIPLEX_ITALIC`, `SCRIPT_SIMPLEX`, `SCRIPT_COMPLEX`.
  Each maps to a per-codepoint index table in `text_data.cpp`.
- `/research/horizon-source/src/util/text_data.cpp:21–115` — twelve
  font index tables (`font_hershey_simplex`, etc.) each ~95 entries
  long, mapping printable ASCII to Hershey glyph indices.
- `/research/horizon-source/src/util/text_data.cpp:161–300` —
  `codepoint_to_hershey()` with hand-curated extensions for German
  umlauts, Greek mu/Omega, Ohm sign, multiplication sign, no-break
  space, plus-or-minus, degree, middle dot, Christmas tree (yes,
  really), and a "palm tree" missing-glyph indicator.
- `/research/horizon-source/src/util/text_data.cpp:304–365` —
  `TextData::TextData()` constructor that walks the input string
  glyph by glyph, decodes the Hershey ASCII per the convention above,
  and emits the line buffer. Also implements **overbar** support via
  `~text~` syntax (lines 313–326): tilde toggles overbar mode, the
  emitted `(overbar_start, x0)` segment is drawn at `bar_y = 24`
  (above the cap line).
- `/research/horizon-source/src/canvas/text_renderer.cpp` and
  `/research/horizon-source/src/util/text_renderer.cpp` — the layout
  engine. `text_renderer.cpp:14` shows the keep-upright detection
  (Horizon's term: `backwards`):
  `bool backwards = (angle > 16384) && (angle <= 49152) &&
   !opts.allow_upside_down;` (using Horizon's 16-bit angle units
  where 65536 = 360°, so `16384 < angle ≤ 49152` is `90° < angle ≤
  270°` — exactly KiCad's threshold).
- Multi-line layout: `lineskip = size * 1.35 + opts.width;` (line 36).
  Note Horizon's `1.35` interline factor versus KiCad's `0.96` —
  this is because Horizon measures from baseline-to-baseline and
  KiCad's `LEGACY_FACTOR` is applied to a glyph-cell-height interline.
  Both end up visually similar.

Horizon's design is a **drop-in template** for Datum because it's BSD-3,
~870 glyphs, complete with codepoint mapping, overbar handling, multi-
line layout, and keep-upright. The downside is no CJK and limited
extended-Unicode coverage compared to Newstroke.

### LibrePCB

LibrePCB diverged: instead of Hershey/Newstroke they invented
**FontoBene**, a stroke-font format that supports **circular arc
segments** in addition to straight lines. Quoting their blog:

> Hershey and NewStroke fonts have many edges because they only
> consist of straight line segments. … FontoBene improves upon
> predecessors by supporting circular arc segments, allowing
> smoother glyph rendering than segment-only alternatives.

LibrePCB's `StrokeFont` class (`librepcb::StrokeFont`) loads `.bene`
files asynchronously, exposes `stroke()` / `strokeLine()` /
`strokeLines()` / `strokeGlyph()` / `getLetterSpacing()` /
`getLineSpacing()`. **Notably it does not expose italic, mirror, or
bold transformations** — those are layered by the caller on top of
the stroke output.

Their bundled font is `newstroke.bene`, which is the Newstroke data
re-encoded in FontoBene format. So even LibrePCB ends up using
Newstroke; they just wrap it in a more general-purpose container.

For Datum: the FontoBene format is interesting but introducing it
adds complexity (arc-aware stroke rendering, arc-aware DRC) that
isn't needed for KiCad-imported board fidelity. **Skip FontoBene for
M7-IMP-014; revisit in M8 if Datum's native authoring needs
arc-based glyphs for marketing-grade silkscreen.**
([LibrePCB Meets FontoBene](https://librepcb.org/blog/2018-04-21_librepcb_meets_fontobene/),
[LibrePCB StrokeFont Class Reference](https://developers.librepcb.org/d1/d45/classlibrepcb_1_1_stroke_font.html),
[LibrePCB/fontobene-fonts](https://github.com/LibrePCB/fontobene-fonts),
[Decide how to implement fonts in footprints and boards · #165](https://github.com/LibrePCB/LibrePCB/issues/165))

### Consumer tools (DipTrace, EasyEDA)

**DipTrace** offers two font modes: "Vector" (default, simple stroke
font similar to Eagle's) and "TrueType" (system fonts). DipTrace
specifically warns: *"Silkscreen text exported as very fine vector
polygons (especially TrueType fonts) massively increases file size,
so converting text to stroke fonts is recommended before export."*
General recommendation: vector font except for logo text or non-Latin
Unicode.
([DipTrace TrueType Fonts](https://diptrace.com/forum/viewtopic.php?t=14308))

**EasyEDA** allows arbitrary system fonts but warns that they must be
locally installed, and specifically calls out a **0.8 mm minimum text
height with 0.15 mm stroke width** as the practical fab limit. Most
EasyEDA designs use the default vector font for silkscreen.
([EasyEDA Place Text](https://prodocs.easyeda.com/en/schematic/place-text/),
[EasyEDA PCB Settings](https://prodocs.easyeda.com/en/pcb/settings-pcb/index.html))

The consensus across consumer tools matches the professional ones:
**stroke-font default, TrueType opt-in for branding-grade text only**.

### Gerber-side text representation

Per Ucamco's Gerber X3 spec, text on a Gerber file can be expressed
three ways:

1. **Aperture-stroked polylines** (`G01 D02 ... D01 ...`) — each
   stroke is a draw with a circular aperture of the chosen stroke
   width. This is the "vector font" path. Smallest file size,
   round caps automatically, photoplotter-friendly. Datum's
   silkscreen exporter currently does this.
2. **Aperture-flashed dot trails** (`G01 D02 ... D03 ...`) — same
   idea but using flash instead of draw. Used by very old plotters.
   Larger files.
3. **`G36/G37` filled polygons** — outline mode followed by
   `G36 ... G37` to declare a filled polygon. This is what TrueType
   fonts have to use because they have curve outlines. Largest files
   (especially for serif fonts), but exact-shape fidelity. KiCad's
   `render_cache` polygons are essentially pre-tessellated `G36/G37`
   payloads.

Datum's recommendation: emit `(1)` for stroke-font text by default
(small, round-cap, fab-friendly); offer `(3)` as an opt-in when the
text is from a TrueType source. **This is exactly the trade-off the
"strokes are canonical, polygons are derived" architecture supports.**

## Output Format Decision (polygon vs polyline)

### Trade-offs

**Stroke / polyline output**
- *Pros*: natural fit for Newstroke (the data is strokes); small data
  size (a typical glyph is 5–15 strokes vs 5–30 polygon vertices);
  trivially renderable as instanced thick line segments on the GPU;
  preserves the distinction between centerline and stroke width
  (DRC needs this — silkscreen-to-edge clearance is measured to the
  stroke edge, not the centerline); natural for Gerber aperture-stroke
  output (smallest fab files); natural for laser-marker output.
- *Cons*: harder to use as a fill region (logo text on copper, decorative
  silkscreen); requires per-pixel stroke-width math during DRC clearance
  checks; doesn't trivially support solid-fill effects.

**Polygon output**
- *Pros*: natural fit for outline fonts; renderable as filled polygons
  via standard tessellation; natural for `G36/G37` Gerber export and
  ODB++ export; collision-detection via polygon intersection is
  uniform with copper polygons.
- *Cons*: larger data (each stroke becomes a 4-vertex capsule rectangle
  or, after union, a closed glyph outline with ~20 vertices); loses
  centerline information; requires earcut/CDT triangulation for GPU;
  doesn't support stroke-width-as-DRC-input cleanly.

**Hybrid: strokes as authored, polygons as derived**
- Brief explicitly forbids a "representation split". But the brief
  also says "may output polygon geometry **or** stroke/polyline
  geometry" — singular generator, single representation per code path.
- The hybrid that is *allowed*: one Datum-owned generator that
  produces strokes, and a *separate* lazy converter that turns
  strokes into polygons for export consumers that need them. This is
  not a representation split because the stroke output is the
  canonical form and the polygon output is a deterministic derivation
  with no fixture-conditional branching.

### What the renderer wants

Datum's M7 GUI uses `wgpu`. wgpu is happiest with **triangles**
either way. For strokes, the standard pattern is to instance a unit
quad per segment and let the vertex shader extrude it to the stroke
width — this is what `lyon`'s tessellator does for stroke paths and
what Bevy's `bevy_prototype_lyon` ships. The stroke geometry path is
**lower-cost on the GPU** than tessellated polygons because:

- Each glyph stroke is one quad (4 verts, 2 triangles) — total per
  glyph ~8–30 verts.
- A tessellated `G` glyph polygon is ~25 verts before triangulation,
  ~40 triangles after — about 6× the GPU work.
- Stroke-width changes (e.g. zoom-aware fattening at low zoom) are a
  uniform tweak, not a re-tessellation.

For the DRC consumer: clearance math against strokes is `point-to-
segment + half_stroke_width`, against polygons is `polygon-polygon
intersection`. The first is faster and less prone to numerical
edge cases. **Stroke is preferable for DRC.**

For the silkscreen Gerber consumer: aperture-stroked output (Gerber
representation #1 above) **directly consumes strokes** — no
conversion needed.

For the brand-text-on-copper / IPC-2581 export consumer: polygon-
fill output is mandatory; stroke→polygon conversion happens once at
export time.

### Recommendation with rationale

**Adopt strokes as the canonical generator output.** Provide a
`strokes_to_filled_polygons()` helper that consumers needing polygon
fill (Gerber `G36/G37`, ODB++, IPC-2581, screenshot-golden-test
rasterization) can call. The helper:

1. Inflates each stroke segment to a round-cap capsule polygon
   (4 vertices + 2 arcs approximated to N segments).
2. Optionally unions all capsules in a glyph using
   `geo_clipper` / `i_overlay` / `lyon`'s Boolean-op support.
3. Returns a `Vec<Polygon>` per glyph.

This satisfies the brief: there is **one generator** (the stroke
generator). Polygon output is a **deterministic derivation**, not a
parallel code path. The cache-present and cache-absent fixtures both
go through the stroke generator and reach the renderer as identical
geometry.

## Vendoring Strategy

### Source options

**Option A: Vendor `newstroke_font.cpp` directly from KiCad's release.**
- Pros: largest Unicode coverage available (~65,000+ glyphs incl. CJK).
  Exact pixel-parity with KiCad rendering for default-font text.
- Cons: KiCad's bundled file has a GPL-2 license header on the C++
  *source code* even though the underlying glyph data is CC0. Lifting
  the file directly imports the GPL-2 header. **Datum cannot accept
  this** per the project's `feedback_no_copyleft_integration` memory
  ("any GPL-class dependency = subprocess only or excluded entirely").

**Option B: Re-encode Newstroke from its upstream CC0 source.**
- The original Newstroke author (Vladimir Uryvaev) granted CC0 to KiCad
  for the glyph data; the CC0 grant is documented on KiCad's
  developers mailing list and on the author's own font website. The
  glyph table itself is *data*, not code, and CC0 places it firmly in
  the public-domain-equivalent bucket.
- Implementation: download the upstream CC0 release of Newstroke
  (typically distributed as a `.jhf` file or as the same `const
  char*[]` form KiCad uses, but with CC0 dedication), re-encode into
  a Rust `static [&'static str; N]` table, and ship under Datum's
  own license with a `NEWSTROKE_DATA_PROVENANCE.md` note documenting
  the CC0 origin.
- Pros: clean license footprint. Same Unicode coverage as Option A.
- Cons: requires sourcing the upstream CC0 file (not just lifting from
  KiCad's release). The author's website needs to be located and
  preserved (snapshot to `research/pcb-text-rendering/sources/`).

**Option C: Re-author from public-domain Hershey JHF data + KiCad's
spacing/metrics tables.**
- The original Hershey font data (from Dr. A. V. Hershey, 1967, NTIS
  format, then James Hurt's `.jhf` re-encoding) is in the **U.S.
  public domain** (federally-funded research, pre-2002). The
  Free Software Directory lists it as public-domain and free to use.
- Implementation: import a public-domain `.jhf` Hershey set
  (`hershey-fonts/hershey.txt` from kamalmostafa/hershey-fonts is
  one canonical source — note the *library* there is GPL-2+ but the
  *data files* are public-domain), apply the codepoint→glyph mapping
  Horizon uses (BSD-3 from OpenCV), and ship under Datum's license.
- Pros: maximum license clarity (PD + BSD-3). Smallest data size
  (~870 glyphs covering Latin, Greek, math, symbols).
- Cons: no CJK. Limited extended-Latin coverage. Visual style is
  Hershey-classic, not Newstroke (Newstroke has been hand-tuned by
  KiCad users for 15 years to read better at PCB silkscreen sizes).
  Imported KiCad boards using non-Latin text will render as missing
  glyphs.

**Option D: Use the existing `kicad_newstroke_font` Rust crate.**
- License: the crate ships KiCad's GPL-2 wrapper on the data file,
  even though the data itself is CC0. **Datum cannot accept** for
  the same reason as Option A.
- Use as: a *reference* during Datum's CC0 re-encoding of the upstream
  Newstroke data, to validate that the encoded byte stream matches.

### License analysis (GPL risk, Hershey provenance)

| Source                                  | License (data)          | License (wrapper code) | Datum-acceptable?    |
|----------------------------------------|-------------------------|------------------------|----------------------|
| KiCad `newstroke_font.cpp`             | CC0 (per author grant)  | GPL-2-or-later         | **No** (wrapper)     |
| `kicad_newstroke_font` Rust crate      | CC0                     | GPL-2 (license file)   | **No** (wrapper)     |
| Upstream Vladimir Newstroke release    | CC0                     | (data only)            | **Yes**              |
| Hershey original `.jhf` (Hurt re-enc.) | U.S. public domain      | (data only)            | **Yes**              |
| Horizon `hershey_fonts.cpp` table      | BSD-3-Clause (OpenCV)   | BSD-3-Clause           | **Yes**              |
| LibrePCB `newstroke.bene`              | CC0 (re-encoding)       | (data file)            | **Yes** (but format) |
| FontoBene format spec                  | (format spec)           | LGPL-3                 | **Subprocess only**  |
| `ttf-parser` crate (for Phase 6)       | n/a                     | Apache-2.0 OR MIT      | **Yes**              |
| `lyon` crate (tessellation)            | n/a                     | Apache-2.0 OR MIT      | **Yes**              |
| `hershey` crate (msrd0)                | n/a                     | Apache-2.0 OR LGPL-3   | **Subprocess only** if LGPL chosen; **Yes** if Apache (it's an OR license — Datum can pick Apache) |

**Critical clarification on the CC0 grant**: per the KiCad developer
mailing list and confirmed in the `kicad_newstroke_font` Rust crate's
documentation, the Newstroke font's author granted CC0 to KiCad
*specifically*. KiCad chose to relicense their re-distribution under
GPL-2 (consistent with the rest of KiCad). The original CC0 grant is
not extinguished by KiCad's choice — Datum can re-import the data
from a CC0 source as long as it doesn't incorporate KiCad's GPL-2
wrapper. Practically: download Newstroke from
`https://stdfonts.com/newstroke/` or equivalent author-controlled
CC0 distribution, not from KiCad's GitLab.
([Re: Release - licenses and legal issues, kicad-developers mailing list](https://lists.launchpad.net/kicad-developers/msg21342.html))

**If the upstream CC0 source can't be located**, fall back to Option C
(public-domain Hershey + Horizon's BSD-3 mapping) — Datum will lose
~64,000 glyphs of CJK coverage but gain absolute license clarity.
The brief's scope is "imported KiCad PCB text" and the typical KiCad
silkscreen is Latin-only (R1, U1, +3.3V, GND), so the practical loss
is small. Document the limitation in `OUT_OF_SCOPE.md`.

### Format options

**Source-embedded `static [&'static str; N]` array**
- Compile-time known. Fast. Cache-friendly.
- ~2 MB binary impact for full Newstroke; ~80 KB for Hershey-only.
- Recommended.

**SVG paths compiled at build time (build.rs codegen)**
- Allows the source data to live as `.jhf` or FontoBene `.bene` and
  be regenerated. Adds a build-time tool.
- Useful if Datum wants to support multiple stroke fonts in the future
  (M8 native authoring with logo fonts).
- Defer to M8.

**Custom binary asset loaded at runtime**
- Smallest binary, but adds a runtime asset-discovery step and a
  failure mode ("Newstroke data not found").
- The brief's "self-contained importer" rule weighs against this —
  Datum should not have a "works only if asset file present" mode
  any more than it should have a "works only if KiCad installed"
  mode.
- Reject.

### Recommendation

**Vendor Newstroke from its upstream CC0 source (Option B), encoded as a
Rust `static [&'static str; N]` array in a new module
`crates/engine/src/text/newstroke_data.rs`. Document provenance in
`crates/engine/src/text/NEWSTROKE_PROVENANCE.md`.** If the upstream CC0
release proves unobtainable in good time, fall back to Option C
(Hershey + Horizon mapping) and document the CJK gap as a known
limitation tied to a follow-on ticket.

## Rust Ecosystem Evaluation

### TrueType crates (mostly wrong shape)

- **`ab_glyph`** (Apache-2.0/MIT, actively maintained): TrueType glyph
  rasterization. Wrong shape for stroke fonts (no centerline output).
  Useful in Phase 6 for `OUTLINE_FONT` import via outline extraction.
- **`fontdue`** (Apache-2.0/MIT, actively maintained): TT raster.
  Same wrong shape. Same Phase 6 utility.
- **`rusttype`** (Apache-2.0/MIT, archived): superseded by `ab_glyph`.
  Skip.
- **`swash`** (Apache-2.0/MIT): full text shaping, OpenType features,
  Bidi. Massive overkill for PCB text. Mention only.
- **`ttf-parser`** (Apache-2.0/MIT, actively maintained): low-level TT
  outline extraction returning Bézier curves. **This is the right
  shape for Phase 6 OUTLINE_FONT support** — give the Béziers to
  `lyon` for tessellation and you have polygon glyphs that match
  KiCad's `render_cache` output.

### Tessellation crates (lyon and friends)

- **`lyon`** (Apache-2.0/MIT, actively maintained): vector path
  tessellation. Has both stroke→triangle and fill→triangle paths.
  Datum should use `lyon` for the stroke-to-quad GPU path AND for
  Bézier-to-polygon tessellation when Phase 6 lands. **Recommend
  adopting `lyon` as the canonical tessellation crate across copper
  rendering, text rendering, and zone fill rendering** — the
  `copper-rendering` research already noted this.
- **`earcut`**, **`earcutr`**, **`spade`** (CDT): general-purpose
  polygon triangulation. `lyon` already includes equivalent
  functionality; no need to add a separate dependency.
- **`geo_clipper`** / **`i_overlay`** (MIT): polygon Boolean ops
  (union, intersection, difference). Useful for unioning per-glyph
  capsule polygons into one closed glyph polygon for `G36/G37`
  Gerber export. Pick one — `i_overlay` is faster, `geo_clipper` is
  more battle-tested.

### Hershey-derived crates (if any)

- **`hershey`** (msrd0, Apache-2.0 OR LGPL-3, v0.1.2 March 2025):
  parser for Hershey font format. ~470 downloads/month. Datum can
  pick the Apache-2.0 side of the dual license to avoid LGPL surface.
  However, it only *parses* — it doesn't ship the glyph data, and
  doesn't handle the layout/justify/keep-upright concerns Datum needs.
  Useful as a parser library; Datum still needs its own layout engine.
- **`kicad_newstroke_font`** (kicad-rs, layered license incl. GPL-2):
  ships KiCad v6 Newstroke raw data. Unusable due to GPL-2 wrapper.
  Use as a *byte-stream reference* to validate Datum's own CC0
  re-encoding.
- **No ideal "Hershey or Newstroke + layout + justify" crate exists** in
  the Rust ecosystem as of April 2026.

### Curve math crates

- **`kurbo`** (Apache-2.0/MIT, used by Druid/Xilem): Bézier curve math,
  affine transforms, path arithmetic. Useful for italic shear (a 2D
  affine transform) and for stroke-to-polygon conversion. Lightweight
  and widely used.
- **`bezier-rs`** (MIT, Graphite-vector-editor): higher-level Bézier
  manipulation. Heavier than needed.
- **Recommend `kurbo`** for the affine transform / stroke offset /
  Bézier sub-path math. It's already a transitive dep of `lyon` so
  there's no new license footprint.

### Embedded-graphics

- **`embedded-graphics`** (Apache-2.0/MIT): has `mono_font` for tiny
  bitmap fonts on microcontroller displays. Wrong shape — bitmap
  rasters, not vector strokes. Skip.

### Layout crates

- **`taffy`**, **`yoga`**, **`cosmic-text`**: Flexbox-style layout, not
  PCB-text geometric layout. Wrong concern. Skip.

### Recommendation: custom implementation with existing crates as helpers

**Build a custom `pcb-text` module** that:
- Owns the Newstroke (or Hershey) glyph data as a vendored Rust
  `static` array.
- Implements its own glyph lookup, italic shear, line layout,
  justification, and keep-upright rules (these are PCB-specific and no
  general-purpose crate matches the geometry).
- Uses **`kurbo`** for affine transforms and curve math.
- Uses **`lyon`** at the consumer boundary for stroke→GPU triangulation
  (renderer) and stroke→polygon tessellation (export).
- Optionally uses **`hershey`** crate (Apache-2.0 side) as a one-time
  parser if Datum chooses to keep glyph data in `.jhf` form rather than
  pre-encoded `static`. Probably not needed if pre-encoded.
- Phase 6: adds **`ttf-parser`** for OUTLINE_FONT support.

## Architecture Proposal for Datum

### Module placement and crate structure

Create a new module **inside the existing `engine` crate** rather than a
separate `pcb-text` crate. Rationale:

- The text generator has zero dependencies that the engine doesn't
  already need (`lyon`, `kurbo` are already candidates for copper
  rendering per `research/copper-rendering/`).
- Crate-boundary churn would be high relative to the value: text
  geometry is consumed by the engine's silkscreen export, by the
  GUI scene materializer in `gui-protocol`, and (eventually) by DRC
  clearance checks. A new crate would force three new dep declarations.
- Per `feedback_decomp_philosophy`, "decomp is governance-triggered and
  organic, not scheduled" — start in-engine and split only if file-size
  or testing reasons emerge later.

Proposed file layout:

```
crates/engine/src/text/
├── mod.rs                       — re-exports, public API surface
├── newstroke_data.rs            — vendored CC0 Newstroke glyph table
├── glyph.rs                     — Glyph type, decode, bounding box
├── style.rs                     — TextStyle, Italic/Bold/Mirror flags
├── attributes.rs                — TextAttributes (justify, keep-upright,
│                                 line-spacing, multiline)
├── layout.rs                    — block layout: line wrap, justify,
│                                 multi-line, keep-upright
├── stroke_generator.rs          — the canonical generator:
│                                 (text, attrs) → Vec<Polyline>
├── polygon_derivation.rs        — strokes → filled polygon outlines for
│                                 Gerber G36/G37 / ODB++ / IPC-2581
├── tests/
│   ├── parity_render_cache.rs   — render_cache as oracle (Phase 5)
│   ├── parity_screenshot.rs     — image-based golden tests (Phase 5)
│   ├── focused_justify.rs       — 9 H×V cases
│   ├── focused_rotation.rs      — 16 angle cases × keep-upright
│   ├── focused_mirror.rs        — front/back × rotation
│   ├── focused_multiline.rs     — \n + per-line justification
│   ├── focused_italic_bold.rs   — style variants
│   └── focused_unicode.rs       — Latin/extended/missing-glyph fallback
└── NEWSTROKE_PROVENANCE.md      — license-history note for the data
```

The new module is an **imported-board normalization dependency** per
the brief's "Minimum Code Surface" requirement, not an ad-hoc renderer
helper. Document this explicitly in `mod.rs`.

### Data-model additions

Extend `BoardText` (`crates/engine/src/board/board_types.rs:88`) to
carry the missing semantic fields. Proposal:

```rust
pub struct BoardText {
    pub uuid: Uuid,
    pub text: String,
    pub position: Point,
    pub rotation: i32,            // tenths of degree (per ENGINE_SPEC.md §1)
    pub layer: LayerId,
    pub height_nm: i64,
    pub stroke_width_nm: i64,     // 0 = auto (15% of height per KiCad)
    pub h_justify: TextHJustify,  // Left, Center, Right
    pub v_justify: TextVJustify,  // Top, Center, Bottom
    pub mirrored: bool,
    pub keep_upright: bool,       // only meaningful for footprint text
    pub italic: bool,
    pub bold: bool,
    pub multiline_allowed: bool,  // default true (matches KiCad)
    pub line_spacing_ratio: u32,  // ppm; 1_000_000 = 1.0× (default)
}
```

All new fields get `#[serde(default = "...")]` so existing native
project files round-trip without touching the schema. The defaults
match KiCad's `TEXT_ATTRIBUTES` defaults (`(CENTER, CENTER)`, no
italic, no bold, not mirrored, not keep-upright, multiline allowed,
1.0× spacing).

Introduce supporting types in `crates/engine/src/text/`:

```rust
#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
pub enum TextHJustify { Left, Center, Right }

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
pub enum TextVJustify { Top, Center, Bottom }

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct TextStyle {
    pub italic: bool,
    pub bold: bool,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct TextAttributes {
    pub h_justify: TextHJustify,
    pub v_justify: TextVJustify,
    pub mirrored: bool,
    pub keep_upright: bool,
    pub style: TextStyle,
    pub multiline_allowed: bool,
    pub line_spacing_ratio_ppm: u32,
}

#[derive(Debug, Clone)]
pub struct Glyph {
    pub strokes: Vec<Vec<Point>>,    // each inner Vec is a polyline
    pub left_bound: i32,             // raw Newstroke unit
    pub right_bound: i32,            // raw Newstroke unit
}

pub struct TextLayout {
    pub strokes: Vec<Stroke>,
    pub bounding_box: Rect,
}

pub struct Stroke {
    pub vertices: Vec<Point>,        // polyline, ≥ 2 points
    pub width_nm: i64,
}
```

### Generator API surface

Public API (`crates/engine/src/text/mod.rs`):

```rust
/// Canonical text geometry generator.
///
/// Consumes text content + semantic attributes, returns a deterministic
/// stroke geometry layout. This is the ONE generator path; cache-present
/// and cache-absent imported boards both flow through here.
pub fn generate_text_layout(
    text: &str,
    position: Point,
    rotation_deg_tenths: i32,
    height_nm: i64,
    stroke_width_nm: i64,
    attributes: TextAttributes,
) -> TextLayout;

/// Convenience wrapper for `BoardText`.
pub fn generate_board_text_layout(text: &BoardText) -> TextLayout;

/// Convert stroke output to filled polygons for Gerber G36/G37 export,
/// ODB++ export, IPC-2581 export, and DRC clearance polygons.
///
/// Lazy derivation: same generator output, deterministic transform.
pub fn strokes_to_filled_polygons(
    layout: &TextLayout,
    options: PolygonDerivationOptions,
) -> Vec<Polygon>;

pub struct PolygonDerivationOptions {
    pub round_cap_segments: u32,    // default 8
    pub union_glyph_capsules: bool, // default true (one polygon per glyph)
}

/// Lookup a single glyph by Unicode codepoint, with fallback handling.
pub fn lookup_glyph(c: char, style: TextStyle) -> &'static Glyph;
```

### Stroke→polygon path

Implementation in `polygon_derivation.rs`:

1. For each `Stroke` in the layout, walk consecutive vertex pairs.
2. For each segment, compute the perpendicular offset by
   `width_nm / 2` to produce a 4-vertex rectangle.
3. At each segment endpoint, append a half-circle arc approximated to
   `round_cap_segments` line segments (shared between adjacent
   segments to avoid double-counting at joins).
4. For each glyph, optionally union all the resulting capsule polygons
   into one closed outline using `i_overlay`'s union op. This gives
   the exact same polygon shape that KiCad's outline tessellator
   produces for the same glyph (validated against `render_cache`
   polygons in the parity tests).
5. Emit `Polygon { vertices, closed: true }` per glyph.

The conversion is **stateless** (depends only on `layout` + `options`)
so it can be cached per-glyph if profiling shows hot paths. M7-IMP-014
should not optimize prematurely.

### Italic / bold variant handling

- **Italic**: shear glyph X by `dy × ITALIC_TILT` where
  `ITALIC_TILT = 1.0 / 8` (KiCad value). Applied at glyph-decode time,
  not at draw time, so the bounding box reflects the sheared shape.
- **Bold**: double the `stroke_width_nm` for the entire layout. Do
  **not** synthesize a separate bold glyph table (KiCad doesn't, and
  the visual result is the same — fatter pen on the same centerline).

This matches KiCad exactly and avoids the complexity of a parallel
bold glyph set.

### Imported text vs future native text reuse

The same generator must serve:

- **Phase 1–5 (M7-IMP-014)**: imported KiCad `gr_text`, `fp_text`,
  `Reference`/`Value` properties.
- **M7+ later**: native Datum-authored board text, where the user
  picks justification, mirror, keep-upright in the GUI.
- **M7+ later**: schematic text (component labels, sheet titles,
  free-floating notes).
- **M8 later**: TrueType `OUTLINE_FONT` import via Phase 6, layered
  on the same `TextLayout` output type.

The only thing the generator should not do is hard-code anything
KiCad-specific. The keep-upright threshold of `(90°, 270°)` is
KiCad's choice; the `ITALIC_TILT = 1/8` is KiCad's choice; the
default `LineSpacing = 1.0` is KiCad's choice. All three are
appropriate defaults for a PCB tool generally and are what users
coming from KiCad will expect, so adopt them as Datum's defaults
without ceremony.

## Test Strategy

### KiCad render_cache as parity oracle

Per the brief, `render_cache` is allowed for "regression comparison,
fixture-backed parity measurement, debugging, proving that
Datum-generated geometry converges toward KiCad-authored output".

Concrete test pattern (`tests/parity_render_cache.rs`):

```
For each fp_text / gr_text in the fixture .kicad_pcb that has
render_cache present:
  1. Parse the cache polygons (already done in
     gui-protocol/src/lib.rs:3722).
  2. Generate Datum's stroke layout from the text semantics.
  3. Convert Datum's strokes to filled polygons via
     strokes_to_filled_polygons(...).
  4. Compute Hausdorff distance (or symmetric-difference area)
     between Datum's polygons and the cached polygons.
  5. Assert distance < threshold_nm.
```

**Caveats**:
- Threshold can't be zero — Newstroke ≠ TrueType, so the cache
  (which is *only present* for TrueType-using text) will *not*
  exactly match a Newstroke rendering. Use a generous threshold
  like 0.3 mm Hausdorff for the M7-IMP-014 acceptance bar; the
  test is checking "Datum's text occupies roughly the right region
  of the board" not "Datum's text is pixel-identical to the
  TrueType source".
- For text where the source is **default Newstroke** (no
  `render_cache` present), this test doesn't apply — use the
  screenshot-golden path below.

A **stronger** parity oracle: render KiCad's PCB to PNG via
`kicad-cli` **at fixture-prep time only**, store the PNG in the
fixture directory, and use it as a screenshot golden. This satisfies
"no KiCad runtime dependency" because the dependency is at
test-fixture-prep time on a developer machine, not at Datum runtime.

### Screenshot goldens (per project policy)

Per `feedback_screenshot_goldens` memory: "rendering work requires
image-based regression, not just unit tests". Implement
`tests/parity_screenshot.rs` to:

1. Render Datum's `BoardReviewSceneV1` for `datum-test` and `DOA2526`
   to PNG via the existing `gui-render` headless wgpu path.
2. Compare against committed golden PNGs using a perceptual diff
   (e.g. `dssim`) with a tight threshold (≤ 0.5% per-pixel diff).
3. On failure, write the new PNG and a diff PNG to a `*-actual.png`
   sibling for human review.

Goldens go in `tests/corpus/<fixture>/golden/text-rendering-*.png`.

### Per-feature focused tests

Build matrix-style focused tests that don't require a full board
fixture:

- **Justification**: 9 cases (LEFT/CENTER/RIGHT × TOP/CENTER/BOTTOM).
  Generate a single 5-character string at each justification, assert
  bbox anchor matches expected.
- **Rotation × keep-upright**: 16 angles (every 22.5°) × `{keep_upright
  on, off}`. Assert that with keep-upright on, rendered angle is in
  `[-90°, 90°]` AND H/V justify is correctly swapped when flipped.
- **Mirror × rotation**: 4 combinations (mirror on/off × 0°/90°).
  Assert that mirror inverts X about anchor regardless of rotation.
- **Multi-line**: 3-line string with `\n` and per-line justification
  set to LEFT, CENTER, RIGHT. Assert each line anchors as expected.
- **Italic**: assert that italic shear adds `dy × 1/8` to each X.
- **Bold**: assert that bold doubles emitted stroke width.
- **Narrow stroke** (laser-marker fonts): 0.05 mm pen on 0.4 mm height.
  Assert geometry remains valid (no degenerate strokes).
- **Wide stroke** (silkscreen): 0.3 mm pen on 1.0 mm height. Assert
  glyph bounds expand correctly to include stroke half-width.
- **Unicode**: `'A'` (basic), `'Ä'` (Latin-1), `'Ω'` (Greek),
  `'µ'` (compatibility), `'\u{4e2d}'` (CJK if Newstroke vendored,
  fallback otherwise), `'\u{E000}'` (private-use, missing). Assert
  fallback emits the missing-glyph indicator and logs once.

### Fixture obligations (`datum-test` + `DOA2526`)

Per the brief and the `M7_IMPORTED_BOARD_FIDELITY_FIXTURES.md`
authority:

- **`datum-test`** (cache-absent): all text materializes through the
  Datum stroke generator. No `cached_polygons` branch should fire
  during scene materialization — assert
  `ComponentTextPrimitive.cached_polygons` is empty for every
  imported text in the resulting scene.
- **`DOA2526`** (cache-present): same assertion. The imported scene
  must NOT carry cached polygons through to the renderer. The
  `kicad_render_cache_world_polygons` call site at
  `gui-protocol/src/lib.rs:3168` should either be removed or moved
  to a debug-only path that doesn't populate the scene contract.
- **Acceptance**: visible text quality (per screenshot golden)
  must NOT differ between the two fixtures by more than the
  perceptual-diff threshold. If `DOA2526`'s text was using a
  TrueType source font and Datum can't match it exactly with
  Newstroke, that's an acceptable failure mode **as long as
  `datum-test` and `DOA2526` are equally non-pixel-perfect** —
  the brief is about parity between fixtures, not parity with
  KiCad.

If a `DOA2526` fixture text has visibly worse rendering after the
generator change than before, ship a recorded warning — the right
follow-on is Phase 6 (TrueType OUTLINE_FONT support), not a
re-introduction of the cached-polygon shortcut.

## Recommended Implementation Phases

### Phase 1 — vendored stroke font + basic glyph rendering

**Estimated effort: 3–5 engineering days.**

- Source upstream CC0 Newstroke release (or fall back to Hershey +
  Horizon mapping). Snapshot to `research/pcb-text-rendering/sources/`.
- Re-encode into `crates/engine/src/text/newstroke_data.rs` as a Rust
  `static [&'static str; N]` table.
- Write `NEWSTROKE_PROVENANCE.md` documenting the license chain.
- Implement `glyph.rs::Glyph::decode_hershey(s)` — the 2-character
  bound + pen-up/pen-down stroke decoding.
- Implement `lookup_glyph(c, style)` with codepoint→array index +
  Latin-1/Greek/Cyrillic/CJK extension table mirroring Horizon's
  `codepoint_to_hershey`.
- Implement `glyph_strokes(c, style) → &'static [Stroke]` returning
  raw glyph centerlines.
- Wire `crates/engine/src/export/silkscreen.rs::glyph_strokes` to
  call the new module instead of its inline ASCII table. Verify
  silkscreen export tests still pass (the existing tests use only
  ASCII so they'll continue to pass — but now Datum can also export
  silkscreen text containing `'µ'`, `'°'`, `'±'`, `'Ω'`).
- Acceptance: ASCII text renders with Newstroke glyph shapes;
  unsupported codepoints emit the missing-glyph indicator instead
  of `ExportError`.

### Phase 2 — layout engine (justification, anchor, multi-line)

**Estimated effort: 3–4 days.**

- Implement `attributes.rs::TextAttributes`.
- Extend `BoardText` schema with the new fields, `serde(default)` for
  back-compat. Update import-side parsers
  (`crates/engine/src/import/kicad/skeleton.rs:612` for `gr_text`,
  `crates/gui-protocol/src/lib.rs:3140+` for `fp_text` /
  `Reference`/`Value`) to populate them from the s-expression.
- Implement `layout.rs::layout_text_block(text, attrs)`:
  - Split on `\n`.
  - Per-line: compute width via cumulative glyph advance.
  - Apply per-line H justify (left/center/right anchor selection).
  - Apply global V justify across the multi-line block.
  - Honor `line_spacing_ratio_ppm`.
- Implement `INTER_CHAR = 0.2 × em` glyph spacing.
- Acceptance: 9 H×V justify focused tests pass; multi-line test passes.

### Phase 3 — mirror, keep-upright, italic, bold

**Estimated effort: 2–3 days.**

- Implement mirror as X-axis reflection about anchor (post layout,
  pre rotation).
- Implement keep-upright correctly: normalize angle, detect
  `(90°, 270°)`, flip 180° AND swap H/V justify (fixing the existing
  Datum bug at `gui-protocol/src/lib.rs:2700`).
- Implement italic shear `dy × 1/8` at decode time.
- Implement bold as `width × 2` at draw time.
- Wire all into `generate_text_layout()`.
- Acceptance: mirror/rotation/keep-upright focused tests pass; italic
  and bold focused tests pass.

### Phase 4 — stroke→polygon for export and DRC

**Estimated effort: 2–3 days.**

- Implement `polygon_derivation.rs::strokes_to_filled_polygons()`:
  - Per-segment capsule generation (4-vertex rectangle + 2 round-cap
    arcs).
  - Optional `i_overlay` union per glyph.
- Wire into the silkscreen Gerber `G36/G37` export path (currently
  Datum's silkscreen export is aperture-stroke only, which is fine —
  expose polygon mode as opt-in for Phase 8).
- Wire into the DRC clearance check: silkscreen-to-edge-cut clearance
  uses the polygon representation so the silkscreen-min-width check
  measures to the stroke edge, not centerline.
- Acceptance: tested per-glyph polygon outline matches a hand-verified
  reference for `'I'`, `'O'`, `'A'`, `'B'` (ascender/descender/holes).

### Phase 5 — render_cache parity tests as regression gate

**Estimated effort: 2–3 days.**

- Implement `tests/parity_render_cache.rs` per the test-strategy
  section above.
- Implement `tests/parity_screenshot.rs` using the existing
  `gui-render` headless path.
- Capture initial golden PNGs from `datum-test` and `DOA2526`
  (whichever generator output the team accepts as the new baseline).
- Add CI gate: any text-related PR must produce identical screenshot
  output on both fixtures unless the golden is intentionally
  re-baselined with sign-off.
- Remove the `cached_polygons` branch from `gui-protocol` per the
  brief's "removal of `render_cache` as final imported-text render
  truth" deliverable.
- Acceptance: `datum-test` and `DOA2526` screenshot diffs are within
  threshold; both go through the same generator path.

### Phase 6 (deferred) — TrueType for OUTLINE_FONT imports

**Out of scope for M7-IMP-014.** Document as a follow-on ticket.

When tackled:
- Add `ttf-parser` and `lyon` deps (lyon may already be present).
- Detect `OUTLINE_FONT` text in the importer (KiCad emits a
  `(font (face "FontName") ...)` block).
- Resolve the font file via a Datum-managed font cache (default:
  bundled set of permissive-license fonts; user-extensible).
- Use `ttf-parser` to extract glyph Bézier outlines.
- Use `lyon` to tessellate to polygons.
- Return polygons through the same `TextLayout` type that strokes
  return (just with `width_nm = 0` strokes that are pure outlines).
- Renderer treats the result as fill geometry instead of stroke.

This is a M8 candidate at the earliest.

### Phase 7 (deferred) — native authored text reuses the same generator

**Out of scope for M7-IMP-014.** Document as a follow-on for M7's
native-authoring slice.

When tackled:
- Native authoring tools (place text, edit text) emit operations that
  produce / mutate `BoardText` entries with full attribute support.
- Schematic text uses the same generator with a different
  `LayerId` ownership model.
- No new generator code needed — only new consumer surfaces.

## Out of Scope (re-affirmed)

The brief lists the following as out-of-scope for `M7-IMP-014`:

- **native-project authored text generation** — defer to a M7 native-
  authoring follow-on. Same generator will be reused.
- **schematic text** — defer to M7 schematic-renderer scope.
- **generic UI font rendering** — separate concern; the GUI framework
  uses TrueType via wgpu's font path.
- **using KiCad as a runtime dependency** — explicitly forbidden by the
  brief and by `feedback_no_copyleft_integration`.
- **widening opening M7 into a full text-engine product initiative
  outside the imported-board slice** — scope discipline.

### Recommended follow-on tickets

- **`M7-IMP-014a` (or M7+1)** — extend the new generator to native-
  authored board text. Mostly UI-side: tools produce `BoardText` with
  full attributes, generator already supports them.
- **`M7-IMP-014b`** — extend to schematic text. Generator-side change
  is small (same API); schematic uses different layer semantics.
- **`M7-IMP-014c`** — TrueType / `OUTLINE_FONT` import (Phase 6).
- **`M7-IMP-014d`** — Newstroke CJK coverage decision: ship CJK glyphs
  (~64,000 glyphs adds ~2 MB to binary), or document as a deliberate
  exclusion. Probably revisit at M8 when international users emerge.
- **`M7-IMP-014e`** — overbar (`~text~`) rendering per Horizon's
  convention (see `text_data.cpp:313`). Common in schematic net labels
  for active-low signals (`~CS`, `~RESET`). Easy to add in the
  layout engine but adds parsing surface.
- **`M7-IMP-014f`** — superscript / subscript via `^{...}` and
  `_{...}` markup. KiCad supports these for board text.

## Datum-Specific Risks

**Risk: license drift when the upstream Newstroke source moves.**
- *Mitigation*: snapshot the upstream CC0 release into
  `research/pcb-text-rendering/sources/` and treat that as the
  long-term source of truth. Document SHA-256 of the snapshot in
  `NEWSTROKE_PROVENANCE.md`. Re-validate annually.

**Risk: Newstroke CJK coverage is slightly different from Source Han Sans
(KiCad's CJK source) so KiCad-imported CJK text doesn't pixel-match.**
- *Mitigation*: out-of-scope for M7-IMP-014. Document as a
  known-acceptable difference.

**Risk: stroke-to-polygon union via `i_overlay` produces self-intersecting
artifacts on degenerate glyphs (e.g. an italic `'@'` with crossing
strokes).**
- *Mitigation*: Phase 4 includes a per-glyph polygon validity assertion;
  fall back to non-unioned capsule output for glyphs that fail
  validity. Log once per glyph.

**Risk: keep-upright justification swap is non-obvious; under-tested
implementations look right at 0° and break at 180°.**
- *Mitigation*: Phase 3's keep-upright focused test exercises every
  rotation × justification combination. CI gate.

**Risk: replacing the `cached_polygons` rendering path causes a visible
regression in `DOA2526` and the team is tempted to revert.**
- *Mitigation*: Phase 5's screenshot golden tests must be approved by a
  human review of the new baseline images (not a blind acceptance).
  The brief explicitly accepts this regression: *"the board should not
  become visually credible or non-credible based only on optional
  source representation."*

**Risk: the silkscreen exporter's existing inline ASCII font
(`crates/engine/src/export/silkscreen.rs:57`) and the new generator
diverge over time.**
- *Mitigation*: delete the inline ASCII font in Phase 1. The exporter
  must call into `crates/engine/src/text/` as the single source of
  glyph data. A grep gate in CI rejects re-introduction of inline
  glyph tables.

**Risk: no Datum maintainer is fluent in the Hershey ASCII encoding
and a future contributor mis-encodes a glyph.**
- *Mitigation*: include a build-time validator in `text/build.rs` that
  parses every glyph string and asserts each pair decodes to a valid
  `(x, y)` point. Round-trip test: vendored data → decoded glyphs →
  re-encoded ASCII → identical to vendored data.

**Risk: `lyon` and `kurbo` adoption is decided here but not in the
copper-rendering work, leading to two divergent tessellation choices.**
- *Mitigation*: cross-reference with `research/copper-rendering/`
  recommendations before adding deps. The two efforts should converge
  on the same crate set.

**Risk: the brief says "imported KiCad PCB text only" but `BoardText`
is a shared canonical IR struct used by native authoring too. Adding
fields to it implies a schema change that ripples beyond the brief's
scope.**
- *Mitigation*: the new fields all have `serde(default)` so existing
  native files round-trip unchanged. Native authoring tooling continues
  to populate only the original fields until separately upgraded.
  Document the schema change in the M7 status notes.

## Sources

- Datum project context:
  - `/home/bfadmin/Documents/datum-eda/CLAUDE.md`
  - `/home/bfadmin/Documents/datum-eda/docs/gui/M7_IMP_014_IMPORTED_TEXT_NORMALIZATION_BRIEF.md`
  - `/home/bfadmin/Documents/datum-eda/docs/gui/M7_RENDER_SEMANTIC_CONTRACT.md`
  - `/home/bfadmin/Documents/datum-eda/docs/CANONICAL_IR.md`
  - `/home/bfadmin/Documents/datum-eda/specs/ENGINE_SPEC.md` (§ 1.3 BoardText)
  - `/home/bfadmin/Documents/datum-eda/crates/engine/src/board/board_types.rs:88`
  - `/home/bfadmin/Documents/datum-eda/crates/engine/src/export/silkscreen.rs`
  - `/home/bfadmin/Documents/datum-eda/crates/engine/src/import/kicad/skeleton.rs:612`
  - `/home/bfadmin/Documents/datum-eda/crates/gui-protocol/src/lib.rs:113`
    (ComponentTextPrimitive)
  - `/home/bfadmin/Documents/datum-eda/crates/gui-protocol/src/lib.rs:2700`
    (kicad_keep_text_upright_degrees)
  - `/home/bfadmin/Documents/datum-eda/crates/gui-protocol/src/lib.rs:3168`
    (cached_polygons branch in scene materialization)
  - `/home/bfadmin/Documents/datum-eda/crates/gui-protocol/src/lib.rs:3722`
    (kicad_render_cache_world_polygons)
  - `/home/bfadmin/Documents/datum-eda/crates/gui-render/src/lib.rs:4163`
    (renderer reads cached_polygons)
- KiCad source (online):
  - [stroke_font.h Source File](https://docs.kicad.org/doxygen/stroke__font_8h_source.html)
  - [stroke_font.cpp Source File](https://docs.kicad.org/doxygen/stroke__font_8cpp_source.html)
  - [font.h Source File](https://docs.kicad.org/doxygen/font_8h_source.html)
  - [eda_text.h Source File](https://docs.kicad.org/doxygen/eda__text_8h_source.html)
  - [eda_text.cpp Source File](https://docs.kicad.org/doxygen/eda__text_8cpp_source.html)
  - [pcb_text.cpp Source File](https://docs.kicad.org/doxygen/pcb__text_8cpp_source.html)
    (KeepUpright at lines 369–387, GetDrawRotation 204–223)
  - [text_attributes.h Source File](https://docs.kicad.org/doxygen/text__attributes_8h_source.html)
  - [text_attributes.cpp Source File](https://docs.kicad.org/doxygen/text__attributes_8cpp_source.html)
    (constructor defaults at lines 24–39)
  - [newstroke_font.h Source File](https://docs.kicad.org/doxygen/newstroke__font_8h_source.html)
  - [newstroke_font.cpp Source File](https://docs.kicad.org/doxygen/newstroke__font_8cpp_source.html)
  - [pcb_io_kicad_sexpr_parser.cpp Source File](https://docs.kicad.org/doxygen/pcb__io__kicad__sexpr__parser_8cpp_source.html)
    (parseRenderCache lines 862–909)
- KiCad documentation and issues:
  - [KiCad Licenses](https://www.kicad.org/about/licenses/)
  - [KiCad mailing list: Newstroke CC0 relicense](https://lists.launchpad.net/kicad-developers/msg21342.html)
  - [KiCad Stroke font discussion](https://lists.launchpad.net/kicad-developers/msg04107.html)
  - [Outline font support MR !613](https://gitlab.com/kicad/code/kicad/-/merge_requests/613)
  - [gr_text render_cache doesn't really do anything (#17666)](https://gitlab.com/kicad/code/kicad/-/issues/17666)
  - [S-Expression Format intro](https://dev-docs.kicad.org/en/file-formats/sexpr-intro/)
  - [Board File Format](https://dev-docs.kicad.org/en/file-formats/sexpr-pcb/index.html)
- Newstroke font (upstream):
  - [Newstroke 001.000 Fonts (mirror)](https://www.onlinewebfonts.com/download/e2394cb7d30696bdf001b6028851e91a)
  - [kicad-rs/kicad_newstroke_font (Rust port, GPL-2 wrapper)](https://github.com/kicad-rs/kicad_newstroke_font)
- Hershey font ecosystem:
  - [hershey crate (msrd0, Apache-2.0/LGPL-3)](https://lib.rs/crates/hershey)
  - [kamalmostafa/hershey-fonts (data + GPL-2 lib)](https://github.com/kamalmostafa/hershey-fonts)
  - [TheV360/hershey_fonts (Rust toy port)](https://github.com/TheV360/hershey_fonts)
  - [Hershey-fonts Free Software Directory](https://directory.fsf.org/wiki/Hershey-fonts)
- Horizon EDA (in-tree at `/research/horizon-source/`):
  - `src/canvas/hershey_fonts.cpp` — OpenCV BSD-3 Hershey table
  - `src/util/text_data.hpp:9` — Font enum
  - `src/util/text_data.cpp:21–115` — 12 font index tables
  - `src/util/text_data.cpp:161–300` — codepoint_to_hershey extensions
  - `src/util/text_data.cpp:304–365` — Hershey ASCII decoder + overbar
  - `src/util/text_renderer.cpp:14` — keep-upright (`backwards`) detection
  - `src/util/text_renderer.cpp:36` — line-skip multi-line layout
  - `src/canvas/text_renderer.cpp` — canvas-side draw routine
- LibrePCB:
  - [LibrePCB Decide how to implement fonts in footprints and boards #165](https://github.com/LibrePCB/LibrePCB/issues/165)
  - [LibrePCB Meets FontoBene blog post](https://librepcb.org/blog/2018-04-21_librepcb_meets_fontobene/)
  - [LibrePCB StrokeFont class reference](https://developers.librepcb.org/d1/d45/classlibrepcb_1_1_stroke_font.html)
  - [LibrePCB/fontobene-fonts repo](https://github.com/LibrePCB/fontobene-fonts)
  - [LibrePCB/librepcb-fonts repo](https://github.com/LibrePCB/librepcb-fonts)
- Eagle:
  - [hershey-text-eagle ULP](https://github.com/nallison/hershey-text-eagle)
  - [Eagle Help: TEXT command](https://web.mit.edu/xavid/arch/i386_rhel4/help/93.htm)
  - [Adafruit Pinguin for EAGLE silkscreen](https://learn.adafruit.com/adafruit-pinguin-for-eagle-cad)
  - [Avon Technical Solutions Hershey Text for EAGLE](http://avon-tech-solutions.co.nz/hershey.html)
- Altium:
  - [Altium CircuitStudio Text](https://www.altium.com/documentation/cstu/text-0)
  - [Altium PCB String](https://www.altium.com/documentation/altium-designer/pcb-string)
  - [Altium PCB String Properties](https://www.altium.com/documentation/altium-designer/pcb-string-properties)
  - [Altium PCB Editor TrueType Fonts Preferences](https://www.altium.com/documentation/altium-designer/pcb-editor-true-type-fonts-preferences)
  - [Altium TrueType Font Question (element14)](https://community.element14.com/products/manufacturers/altium/f/forum/31606/what-font-works-best-and-where-is-the-default-font-selected)
  - [How to export to PDF with smooth fonts (EEVblog)](https://www.eevblog.com/forum/altium/how-to-export-to-pdf-with-smooth-fonts/)
- DipTrace and EasyEDA:
  - [DipTrace TrueType Fonts forum thread](https://diptrace.com/forum/viewtopic.php?t=14308)
  - [DipTrace silk font question](https://diptrace.com/forum/viewtopic.php?t=11829)
  - [DipTrace Rather large Silkscreen Gerbers](https://diptrace.com/forum/viewtopic.php?p=38005)
  - [EasyEDA Place Text](https://prodocs.easyeda.com/en/schematic/place-text/)
  - [EasyEDA PCB Settings](https://prodocs.easyeda.com/en/pcb/settings-pcb/index.html)
- Rust ecosystem:
  - [crates.io: hershey](https://crates.io/crates/hershey)
  - [crates.io: ttf-parser](https://crates.io/crates/ttf-parser)
  - [crates.io: lyon](https://crates.io/crates/lyon)
  - [crates.io: kurbo](https://crates.io/crates/kurbo)
  - [crates.io: ab_glyph](https://crates.io/crates/ab_glyph)
  - [crates.io: fontdue](https://crates.io/crates/fontdue)
  - [crates.io: i_overlay](https://crates.io/crates/i_overlay)
- Companion Datum research:
  - `/home/bfadmin/Documents/datum-eda/research/airwire-rendering/AIRWIRE_RENDERING_RESEARCH.md`
  - `/home/bfadmin/Documents/datum-eda/research/copper-rendering/COPPER_RENDERING_RESEARCH.md`
  - `/home/bfadmin/Documents/datum-eda/research/ipc-compliance/IPC_COMPLIANCE_RESEARCH.md`
