# Datum Rendering Book

> **Status**: Governed visual-identity spec under decision 019, extending decision
> 015 (Design Book tokens) from UI chrome into **manufacturable content geometry** —
> schematic symbols, PCB footprints, silkscreen type, and icons. This is the
> "what it looks like" authority; its foundation — the two fidelity/beauty Laws and
> the DFM Geometry Solver — lives in
> `docs/gui/DATUM_RENDER_FIDELITY_AND_DFM_GEOMETRY.md`.
> **Controlling visual truth**: `docs/gui/prototypes/rendering-study.html`
> (owner-approved, design pass 3). A built surface must match its composition,
> palette, and geometry treatment.
> **Purpose**: pin the house style at concrete values so every development agent
> (and generator) produces **accurate and consistent** library symbols and
> footprints, not a per-agent reinvention.

Legend: **[LOCKED]** owner-approved this pass · **[OPEN]** awaiting owner lock.

## 0. Canonical palette — the board-editor token set  **[LOCKED]**

Content geometry uses the shipped `board-editor.html` tokens (do not reinvent):

| Role | Token | Hex |
|------|-------|-----|
| Canvas / substrate | `--canvas` | `#0B0C0E` |
| Copper — top / bottom | `--cu-f` / `--cu-b` | `#C83A34` / `#4D7FC4` |
| Pad (land) | `--pad` | `#C9974A` |
| Via | `--via` | `#C77B3C` |
| Silkscreen | `--silk` | `#E8E6DC` |
| Board edge | `--edge` | `#CBB24A` |
| Component body | (body / border) | `#151922` / `#3A414D` |
| Courtyard | (hairline) | `#6E7681` |
| Dimension / origin | (annotation) | `#7FB0C8` |
| Schematic wire | `--wire` | `#4FA75A` |
| Symbol stroke | `--sym` | `#AEB4BB` |
| Selection accent | `--acc` | `#CE5A92` |

Semantic (error/warn/ok/info) stays separate from the accent; the accent is
selection/active only.

## 1. Schematic canvas — dark works, vellum prints  **[LOCKED]**

- **Dark is the working default** (`#0E1013` paper, line grid `#141821`): modern,
  low-fatigue over long sessions, in the board-editor idiom.
- **Vellum is a print/documentation toggle** (`#E7E1D2` warm drafting film, ink
  `#2C2820`): the user flips to it for printing schematics and rendering
  **monochrome documentation to disk**, where warm paper translates better than
  glaring white. Vellum is fatiguing for long editing — hence it is the toggle,
  not the default.
- **One chrome.** The application chrome (tool rails, panels, marking menus, status
  bar) stays dark in both modes; only the document *canvas* changes ground.

## 2. Schematic symbols  **[LOCKED except symbol standard]**

- **Filled bodies** (`#161A24` on dark / `#F1ECDE` on vellum) with a small optical
  corner radius (~3px view) — solid objects with figure-ground, not hollow 1980s
  line-art.
- **Pins.** Every component pin renders as a **short stub** from the body edge
  ending in a small **terminal dot** at the electrical connection point; the pin
  name sits inside the body (number optional). The terminal dot is the pin marker
  and is visually distinct from a **junction dot** — pin terminals use the
  symbol-stroke colour, junctions are filled in the wire colour — so "a pin" and
  "wires meet here" never read the same. Wires connect *to* the terminal dot and
  **never cross the symbol body**.
- **Stroke hierarchy** (deliberate, not uniform): bus > wire > symbol body > pin
  stub. Wires `--wire` at ~1.6; always orthogonal with crisp corners.
- **Junction dots** filled in the wire color; **net-label pills** are *filled*
  rounded-rects (`#151922` fill, `#3A414D` border on dark) — the board-editor pill,
  not a hollow tag.
- **Ref/value typography** in the project typeface (§5). Designators tabular.
- **Selection highlights the whole symbol.** Body stroke, pins, terminal dots, and
  all symbol text render in the accent (`--sel`) under one soft glow — a
  half-highlighted symbol (body accented, pins left in the default colour) is a
  defect. Attached nets keep their normal wire colour (they are not part of the
  selected symbol). Selection is **screen-only presentation state** — never
  geometry, never journaled, never in any export (Law 1 fence).
- **Symbol standard — LOCKED: IEC 60617 rectangular.** The resistor — and the IEC
  rectangular convention generally — is the Datum house standard: cleaner,
  grid-aligned, and international. ANSI/IEEE 315 zigzag is **not** the canonical
  symbol (it may return later as an optional regional display, but the default and
  the authored/generated symbol is IEC).

## 3. PCB footprints  **[LOCKED]**

- **Rounded-rectangle pads at a 25% rounding ratio** (`radius = 0.25 × min(w,h)`,
  capped). The radius is **real manufactured geometry**, not cosmetic: it raises
  copper peel strength, etches clean, and solders without bridging. A sharp-cornered
  pad is the fiction — the etch rounds corners regardless. Gold copper `--pad` on
  the dark substrate.
- **Pad-1 marker**: silk dot + printed pad number — unambiguous, not shouting.
- **Courtyard** as a dashed hairline `#6E7681`; **component body** as the
  board-editor dark body (`#151922` / `#3A414D` border); **origin crosshair** in the
  annotation color.
- **Silkscreen–to–copper clearance**: the silk body outline is **clipped/inset**
  so it holds the IPC minimum silk-to-copper clearance (~0.15–0.20 mm, fab-dependent)
  — silk must never overlap or touch exposed copper. The **assembly** outline may
  abut/cross pads (documentation layer); the **silkscreen** outline may not. This
  clip is engine-derived geometry (Law 1: what renders is what the fab receives).
- **Dimension overlays** in mechanical-drawing style: thin extension lines, small
  arrowheads, tabular values, in `#7FB0C8` — IPC land-pattern math made visible and
  beautiful.

## 4. DFM-derived geometry defaults  **[LOCKED]**

The look is produced by the DFM Geometry Solver (engine spec in the fidelity note),
default-on per Law 2. Rendered geometry is byte-identical to CAM (Law 1).

- **Acute trace bend (<90°)** → a **small inner fillet on both inside edges**, sized
  only to compensate etch loss (open the acid trap) — **not** a chamfer, not a full
  corner round. The radius is **algorithmically derived** (`r ≈ f(etch_undercut,
  included_angle)`), kept as small as the etch-relief requires — never prominent.
- **Right-angle bend** → miter; for high-speed nets the Douville–James optimum
  microstrip miter `M% = 52 + 65·e^(−1.35·W/h)`.
- **Trace-to-pad/via junction** → **teardrop** fillet, sized from pad/via/drill
  diameter and trace width.

## 5. Typography — IBM Plex, program-wide  **[LOCKED]**

- **Two IBM Plex faces, no others.** `IBMPlexSansCondensed` is the primary face —
  application chrome, headings, on-canvas schematic/footprint text (pin names,
  reference designators, values, net labels), and silkscreen. `IBMPlexMono` is
  reserved for **aligned numeric data** — coordinates, hex, dimensions, counts —
  where digit columns must line up; it is **not** used for labels, names, or
  designators (those are Sans). No system fonts anywhere; a mixed font (even on one
  glyph like `·` / `—` / `≥`) is a defect. Faux-bold is **off**
  (`font-synthesis: none`) — weights are real, never synthesized.
- **Weights bundled.** Sans Condensed `Regular / Medium / SemiBold` and Mono
  `Regular / Medium` ship in `crates/engine/assets/fonts/` (SIL-OFL; license at
  `fonts/OFL.txt`). Hierarchy is carried by real weight + size + color.
- **Silk is the manufactured case.** IBM Plex rendered as **filled outline geometry**
  — not stroke centerlines (the "you can always tell Eagle" tell) — and **designed
  for silk**: it respects the silk minimum feature/line width (~0.15 mm / 6 mil,
  fab-dependent) so it prints clean. The on-screen silk *is* the geometry the fab
  receives (Law 1); the same face carries into fab/assembly **documentation** (title
  blocks, fab notes, assembly drawings).
- **Engine wiring — DONE (2026-07-08).** The engine text registry
  (`crates/engine/src/text/registry.rs`) vendors IBM Plex Sans Condensed
  Regular/Medium/SemiBold + IBM Plex Mono Regular/Medium, and **every text intent
  resolves to IBM Plex** (Sans for silk/annotation/UI/docs, SemiBold for branding);
  all faces recorded in `FONT_PROVENANCE.md`. The GUI's board silk and the CAM
  exporter share one resolution (Law 1). Manufacturing silk/mechanical gerber
  goldens were refreshed to the IBM Plex letterforms.

- **Type colour (Grauwert) — even tonal weight.** Large/heavy type is a dark mass that
  optically overpowers small text; soften it toward the ground so the composition reads
  with **even tonal colour** (Emil Ruder, *Typographie*, 1967, "shades of grey"; codified
  as "typographic colour" by Bringhurst). Datum's quantified rule on a light ground: body
  (≤16 px) and smaller stays **full ink**; above 16 px, step the fill toward the ground by
  ~8% per size-octave plus ~2% per weight-step, floored at **62% ink**, mixed in OKLab —
  so a 54 px SemiBold hero renders at ~79% ink (a warm grey). **Mono data is exempt**
  (always full ink). Grey value is rotation-invariant, so it balances identically in any
  orientation. Formula + constants + lookup:
  `research/gui-typography/TYPE_COLOUR_GRAUWERT_RESEARCH.md`.

## 6. Text sizing, clearspace & placement  **[LOCKED]**

> Owner-approved via `docs/gui/prototypes/text-placement-study.html`. Research:
> `research/gui-typography/TEXT_PLACEMENT_CLEARSPACE_RESEARCH.md`. **Honesty note:** the
> silk height/stroke numbers are fab-DFM + KiCad-library practice, not literal IPC
> clauses (IPC-2221C / IPC-A-610J mandate only "legible and durable").

**Silk text sizing — scales to the part, no forced minimum.**
- The **only hard limit is manufacturability:** a DRC *error* fires solely when the
  thinnest rendered glyph stem falls below the fab profile's `min_feature_width`
  (~0.15 mm standard / 0.10 mm HD). Text far below 0.8 mm is allowed — an 0402's `R1` is
  0.5 mm, an 0201's is 0.3 mm; a minimum height taller than the part is wrong.
- **1.0 mm is a default; ~0.8 mm is a soft legibility advisory that is OFF by default** —
  never a gate. Datum does not nag every DRC run the way KiCad does.
- **Weight assist, not enlargement:** at small sizes the renderer may pick IBM Plex Sans
  Condensed **Medium** to keep the stem imageable; it never makes text taller. Where no
  printable weight can image, advise "RefDes → assembly drawing," don't enlarge.
- Small parts (0402/0201/01005) commonly carry **no per-part silk RefDes** — the
  designator lives on the assembly drawing.

**Schematic text sizing.**
- Default **1.27 mm (50 mil)** for RefDes, value, pin name, and net label (matches the
  50 mil grid users know); pin numbers smaller, in IBM Plex Mono (they are data); pin
  minimum 0.5 mm. Grid-aligned but **finely nudgeable off-grid**. Title-block / fab-note
  text uses the ISO 3098 ladder (2.5 / 3.5 / 5 mm).

**Clearspace — a minimum distance, expressed as a ratio of cap height.**
- Keep-clear = **0.5 × cap height (H)** minimum (1.0×H preferred) between text and any
  neighbour (other silk, text, courtyard, symbol body). Because it is a ratio of H it
  scales with the text automatically. It is **presentation/legibility**, distinct from
  the hard silk-to-copper clearance (§3); effective keep-out = `max(silk_to_copper,
  clearspace)`.

**Placement & repositioning — a movable label with a tether.**
- RefDes/value/label text is an **anchor + offset field owned by its component/net**:
  nine auto-position presets plus free manual drag; a drag switches the field to Manual.
  Nudged off its default, a **tether line** shows what it belongs to (as in every EDA
  tool).
- **Moving text is a presentation act.** It authors a journaled `MoveText` /
  `SetTextAnchor` typed op (undo, provenance, and — unlike screen-only selection — it is
  in the model and in CAM per Law 1), but it is **structurally incapable of changing the
  netlist/connectivity** — it is a label, not a connection.

**Legibility checks (DRC).**
- `text_over_pad` — silk text intersects exposed copper → **error** (ties to §3).
- `text_overlap` — a neighbour is inside the clearspace → **warning** (ignorable).
- Manufacturability — thinnest stem < fab `min_feature_width` → **error** (the only hard
  size limit).

## 7. Icons  **[LOCKED]**

24px viewBox · 1.7 stroke · no fill · round caps &amp; joins · 2px optical grid.
Every glyph is one declared entry in `docs/gui/icon_set.json` (gated by
`check_menu_model.py`); the gate forbids an undeclared icon. Phase 1 renders
not-yet-authored glyphs via a declared fallback so the shell never blocks on art
(`DATUM_GUI_PHASE_1_SPEC.md` D6, fallback-first).

## 8. Title block & sheet frame  **[content LOCKED · visual in progress]**

> Content foundation owner-approved 2026-07-08. Research:
> `research/documentation-system/TITLE_BLOCK_AND_DOC_CONTROL_RESEARCH.md`. Principle:
> the standards (ISO 7200) require the data to **exist and be controlled**, not to be
> crammed on the sheet face. Minimalism = separating the **on-face set** from the
> **model-captured record**. Visual language is being developed on the prototype
> `docs/gui/prototypes/title-block-study.html`.

**On-face set — minimal, always shown:** Title · Drawing number · Revision (current
only) · Date of issue · Sheet n/N · Project name · Company (logo + name) · Status ·
Drawn by · Document type (small/implicit). This is the whole face.

**Compact on the face only when applicable:** Checked/Approved as a single compact line
(initials + date), never a signature matrix; Classification as one small mark if the
doc is classified.

**Captured in the doc-control model, NOT on the face:** full revision history table (→
register + optional revision sheet; the face shows current rev only); full approval
chain / signatures; client/customer (off by default); contract/PO/project number (in
the model, or folded into the drawing number); company address/contact (logo + name
only on the face).

**Per-document-type additions (DFM-aware — only relevant fields ever appear):**
schematic = the on-face set, nothing more (no scale); fabrication drawing adds scale ·
projection · units · tolerance · CAGE · material/finish; assembly drawing adds scale ·
part number · BOM reference.

The ISO 7200 core (title, drawing number, document type, issuing organization,
revision, date of issue, sheet n/N) is non-negotiable; everything else on the face is
professional-standard, kept minimal by intent.

## 9. Still to specify (next passes, not blockers)

Sheet borders + title block design; full layer-stackup palette; broader symbol and
footprint families; thermal-relief and copper-pour rendering; assembly/fab-doc
templates. Each extends this book downstream of an owner-approved prototype pass.

## Open decisions (owner to lock)

- Rounding-ratio validation against measured peel strength; teardrop auto-apply
  thresholds; dimension-line typography; exact vellum warmth.

(Fork B — symbol standard — is now **locked to IEC**, see §2. Font engine wiring is
**done**, see §5.)
