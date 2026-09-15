# EDA Text Sizing, Clearspace & Placement — Research

> Study for the Datum Rendering Book text-placement chapter (agent-assisted,
> 2026-07-08). Extends `docs/gui/DATUM_RENDERING_BOOK.md` §3 (silk-to-copper
> clearance) and §5 (IBM Plex typography); cross-references
> `research/ipc-compliance/` and `research/schematic-drawing-conventions/`
> (does not re-survey them). **Honesty flag:** the concrete silk height/stroke
> numbers below are fab-house DFM + KiCad-library practice, **not** literal IPC
> clauses — IPC-2221C / IPC-A-610J mandate only that marking be *legible and
> durable*. Cite accordingly in the governed chapter.

## 1. Silkscreen text sizing (fabrication)
- **Legibility guideline vs. manufacturability floor (owner correction, 2026-07-08):**
  DFM guides quote ~**0.8 mm (32 mil)** height / **0.15 mm (6 mil)** line as a
  *recommended-legibility* figure — **not** a hard manufacturability limit. The real hard
  floor is the fab's **minimum imageable feature width** (~0.10 mm HD / 0.15 mm standard,
  a fab-profile parameter); a fab prints text far below 0.8 mm, it just may not be crisply
  legible. **Forcing a 0.8 mm minimum is wrong** — it exceeds the body of an 0402
  (1.0 × 0.5 mm), 0201 (0.6 × 0.3 mm), or 01005, and dense/small design routinely uses
  sub-0.8 mm silk or omits per-part RefDes for the assembly drawing. So Datum's only
  *hard* text limit is manufacturability (thinnest glyph stem ≥ fab min feature); 0.8–1.0
  mm is a *soft, advisory* legibility guideline the designer may knowingly go under.
- **Reliable-legibility default (populated board):** height ≥ **1.0 mm (40 mil)**,
  stroke ≥ **0.2 mm**; 1.27–1.52 mm for inspection/field-service text. RefDes practice
  (KiCad KLC F5.2): height ≈ 1.0 mm, thickness ≈ 15% of height.
- **Aspect ratio:** silk **5:1–6:1** height:stroke — deliberately thicker than ISO 3098
  technical lettering (Lettering B = h/10, A = h/14), because a 10:1 stem is too thin to
  print cleanly on mask.
- **Datum-specific consequence (from §5's filled-outline doctrine):** stroke width is not
  a knob — it is the glyph's **thinnest stem** at a given cap height. IBM Plex Sans
  Condensed Regular stem ≈ 10–13% of cap height, so to keep the stem ≥ 0.15 mm a
  Regular glyph needs cap height ≈ **1.2–1.5 mm**. Therefore **auto-promote silk to IBM
  Plex Sans Condensed *Medium* below ~1.2 mm cap height** rather than shrink Regular past
  its printable stem. (Measure the exact stem ratio from the shipped face.)

## 2. Silk-to-copper / silk-to-pad clearance (cross-reference only)
Confirms/cites existing §3: silk holds ≥ ~**0.15 mm (6 mil)** clear of exposed copper /
mask openings / pads (some DFM cite 5 mil / 0.127 mm floor); silk ink on solderable
surface causes solder rejection / smearing. No new value — reaffirm §3, add the citation
trail (IPC-A-610J marking + fab DFM). This is a **manufacturing** rule, distinct from the
**legibility** clearspace of §3 below.

## 3. Clearspace / keep-clear (brand-systems concept applied to EDA text)
Brand clearspace = a minimum empty margin around a mark, expressed as a **ratio of cap
height** (typ. 1×–1.5× for a logo; 0.5× for dense technical layouts), never an absolute
value. Applied to EDA text it is a **legibility/presentation** zone (keep text off other
silk, text, courtyard lines, symbol bodies) — distinct from §2's manufacturing copper
clearance. No EDA standard formalizes it; it is a differentiator Datum can import.
- Recommend: `text.clearspace.min = 0.5 × cap_height`, `.preferred = 1.0 × cap_height`;
  ratio-based so it **scales with size**; effective keep-out from a neighbor =
  `max(silk_to_copper §2, clearspace)`. Soft/warning (presentation, not manufacturing).
- Sources: brand-guideline practice (Johns Hopkins Medicine, Univ. at Buffalo, Akrivi).

## 4. Schematic text conventions
The schematic grid is the sizing anchor. KiCad (community reference; Datum compatibility
target) fixes: working grid **50 mil (1.27 mm)**; **all text fields (pin name/number,
RefDes, value, net label) default to 50 mil height** (KLC S3.2); pin name/number may drop
to 20 mil for cramped symbols. RefDes + value are separate movable symbol-anchored fields
(auto-placeable + manual override). Net labels attach to a wire, user-movable, kept
horizontal / 90° (never upside-down). ISO 3098's nominal ladder (1.8/2.5/3.5/5 mm) applies
to **title-block/fab-note** text, not on-canvas symbol text.
- Recommend: `sch.text.height.default = 1.27 mm (50 mil)` (RefDes/value/label);
  `sch.text.height.pin.min = 0.5 mm`; grid 1.27 mm, text finely nudgeable off-grid;
  title-block/fab-note text on the ISO 3098 ladder.
- Sources: KLC S3.2, KiCad Schematic Editor docs, ISO 3098-1:2015.

## 5. Movable / repositionable text — uniform across pro tools
Text is an **anchored, offset field owned by the component/net, electrically inert**:
- **Altium:** RefDes/comment with **Autoposition** (nine anchor presets that follow
  rotation/move) + **Manual** mode; dragging text auto-switches to Manual. Overlay/silk
  data, never netlist.
- **KiCad:** each symbol field has position/alignment relative to the symbol anchor;
  **Autoplace Fields** (disableable per field); moving a field never touches the netlist.
- **OrCAD/Allegro:** **AutoSilk** clips text off pins/vias and out from under parts;
  **Label Tune** auto-arranges; lock toggle freezes location; silkscreen clearance is a
  **Constraint-Manager fabrication rule**.
- **Silk-overlap DRC exists in the pros** (Allegro silkscreen-clearance, Altium
  silk-to-mask/copper) — most open tools lack it → a Datum differentiator.
- Sources: Altium Component Text Position / Autoposition; KiCad field-autoplace; EMA/
  Cadence AutoSilk + Label Tune + Allegro Constraint Manager docs.

## 6. The invariant (governing)
Every surveyed tool derives the netlist from **electrical connectivity only** — never
from RefDes/value/label text position. Moving text is a **documentation/presentation act**.

## Recommended Rendering Book rules (for the governed chapter)
- **A. Silk sizing (no forced height floor):** hard limit is **manufacturability only** —
  DRC *error* iff the thinnest rendered glyph stem < the fab profile's `min_feature_width`
  (~0.15 mm standard / 0.10 mm HD). `height.default 1.00 mm` (auto-placed RefDes where
  space allows); `height.legibility_advisory ≈ 0.8 mm` — soft warning, **off by default**
  for experts, never blocking. **Weight assist, not enlargement:** at small sizes the
  renderer may pick IBM Plex Sans Condensed **Medium** to keep the stem imageable on the
  fab profile, but never forces the text *taller*; where even the thinnest printable
  weight can't image, advise "RefDes → assembly drawing", don't enlarge. height:stroke
  ≈ 5:1–6:1 is a target, not a gate. Small parts (0402/0201/01005) commonly carry no
  per-part silk RefDes — the designator lives on the assembly drawing.
- **B. Schematic sizing:** `default 1.27 mm (50 mil)`, `pin.min 0.5 mm`, grid 1.27 mm,
  nudgeable off-grid; title-block/fab-note on ISO 3098 ladder (2.5/3.5/5 mm).
- **C. Clearspace:** `min 0.5 × cap_height`, `preferred 1.0 × cap_height`, ratio-based;
  keep-out = `max(silk_to_copper, clearspace)`; soft/warning.
- **D. Placement model:** text = `TextAnchor { owner, anchor_preset, offset, rotation,
  justification }` on component/net; Altium-style nine presets + free manual nudge; drag
  ⇒ Manual; moves author a journaled `MoveText`/`SetTextAnchor` typed op.
- **E. Legibility DRC:** `text_over_pad` (error, §2), `text_overlap` (warning, §3
  clearspace), `text_illegible` (error, cap < 0.80 mm or min stem < 0.15 mm).
- **F. Governing invariant — Text-Position Inertness:** moving RefDes/value/label text is
  journaled presentation geometry (undo/provenance; in-model and in-CAM per Law 1) but is
  **structurally incapable of mutating the netlist/connectivity/electrical fields** — safe
  by construction, not discipline. Pairs with §2's "selection is screen-only" fence:
  selection is screen-only and never journaled; text position is real geometry and always
  journaled — but neither can alter electrical meaning.

## Sources
Silk sizing: Magellan Circuits; PCBCart; Aivon; JLCPCB; PCBWay. Standards: IPC-2221C
(NextPCB); IPC-A-610 (PCBSync, Wevolver). Schematic: KLC S3.2, F5.2; KiCad docs; ISO
3098-1. Movable text: Altium Autoposition; KiCad autoplace; EMA/Cadence AutoSilk/Label
Tune. Clearspace: Johns Hopkins Medicine; Univ. at Buffalo; Akrivi.
