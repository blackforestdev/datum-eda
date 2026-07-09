# Title-Block, Sheet-Frame & Documentation-Control System — Research

> Multi-agent research synthesis (2026-07-08): 5 parallel research dimensions
> (standards · contemporary design · template systems · field data-binding ·
> doc-control) + a conductor pass, ~330K tokens. Research-only. Feeds a governed
> Rendering Book chapter (§8 "Sheet borders + title block") + future spec after the
> owner-approved HTML prototype. Owner is a stickler for Pentagram-grade contemporary
> title blocks; reference aesthetic = Autodesk Revit / audreynoakes.com "contemporary
> title blocks"; firms must be able to lay out/build/import custom blocks + their own
> doc-control assets; beautiful documentation BY DEFAULT + full customization; field
> "formulas" drive it.

## Thesis — three separated layers on the substrate

A title block / sheet frame is **one subsystem, three layers**:
1. **Data / semantic** — ISO 7200 fields + doc-control state (hard-required content).
2. **Binding / formula** — every field is a *resolved expression* over the model +
   journal, never free text (where the owner's formulas plug in).
3. **Visual / layout** — anchored graphic primitives + field slots (the free design
   space where "Pentagram-grade" lives; ISO 7200 does **not** specify layout).

All ride Datum's substrate: layout authoring + field-literal entry are typed
`Operation`s through `commit()`+journal; **field *resolution* is a projection (like
hover/selection), never journaled** → render stays deterministic. **Differentiator no
surveyed tool has:** the revision block + status field are *projections of the commit
journal* → the block is always live + provenance-backed, and render==CAM fidelity (Law
1) extends to it — the released fab-drawing block is a CAM deliverable, byte-identical
on screen / PDF / plot.

**Single most important structural fix** (from KiCad `.kicad_wks`): field positions are
**corner-anchored offsets, not absolute points** — so one template survives A4→A3 and
portrait↔landscape. (The repo's first-cut `FieldPlacement { position: Point }` must
become anchor+offset.)

## Standards baseline (constrains content/semantics, never styling)

- **Sheet sizes nm-exact** (ISO 5457 / ASME Y14.1): ISO A0 841×1189 … A4 210×297; ANSI
  A–F; US Letter/Legal/Tabloid; Custom. Posture: **validate, don't enforce** (warn on
  overflow). Repo `SheetSize` enum is the correct set.
- **7 ISO 7200 mandatory fields**: Title, Identification/doc number, Document type,
  Issuing org, Revision indicator, Date of issue, Sheet n/N. (Repo `SheetFrame` has 4:
  title/revision/company/page_number — add the rest + `document_type`, `status`,
  `approver`, `reviewer`, `project_number`, `customer`, `classification`; `scale` +
  `projection` gated to fab/assembly `DocumentType`.)
- **Revision sequence** (ASME Y14.35): `A,B,C…` uppercase omitting `I,O,Q,S,X,Z`, ≤2
  chars then AA…, first issue `-` or `A`.
- **Border/frame** (ISO 5457): 20 mm binding margin, 10 mm others; frame 0.7 mm; grid
  0.35 mm (deliberate weight hierarchy). Title block bottom-right; folded-visibility
  ≤170–180 mm from corner (origin of the ~180 mm block width).
- **Zone grid** (ISO 5457): 50 mm fields, even counts, letters (omit I/O) × numerals.
- **Profile-parameterise ISO vs ASME, don't fork**: `DrawingStandardProfile
  { Iso5457 | AsmeY14 | Custom }` — ISO zone origin top-left/centre-symmetric; **ASME
  origin = corner adjacent to title block (lower-right), numerals right→left, letters
  bottom→top.** One table-driven code path (hard-coding silently corrupts zone refs).
- **No interoperable template format exists** — every tool's is proprietary
  (`.kicad_wks`/`.SchDot`/`.dot`). The template format is Datum's to own.

## Contemporary design pattern language (the "designer-made by default" layer)

Bind to the *locked* Rendering Book tokens (do not add new ones): vellum `#E7E1D2` /
ink `#2C2820` for docs, dark `#0E1013` while editing; IBM Plex Sans Condensed
(Regular/Medium/SemiBold) + Mono (data). Diagnosis: legacy-CAD = dense equal-weight
boxed-cell matrix; contemporary/editorial = whitespace-separated groups, two-weight
hairlines, tiny tracked uppercase micro-label + larger value, **sheet number as hero**,
colour = status only.

Twelve patterns: **P1** right-edge vertical title strip (40–55 mm) / P1b bottom strip ·
**P2** sheet number hero (SemiBold ~4–6× body, bottom-right) · **P3** micro-label (Mono
6 pt uppercase tracked `#717885`) / value (Sans Cond Medium 10 pt ink) stack · **P4**
two-weight hairlines (frame `border.strong` 0.35 mm / dividers `border.subtle` 0.13 mm,
**no boxed cells**) · **P5** modular micro-grid, sparsely filled · **P6** generous margin
quantum · **P7** monochrome optically-placed logo (SVG/PNG, never stretched) · **P8**
colour = status only (accent `#CE5A92` reserved) · **P9** status band/watermark bound to
`status` (empty when Released) · **P10** revision *ledger* not cells (`REV|DATE|DESC|BY`,
Mono tabular, auto-appended from journal) · **P11** borderless/float variant · **P12**
every value a `${key}` binding, never a literal.

**Two shipped skeletons** (A3-landscape 420×297, scale linearly): **Skeleton A "Strip"**
— ~52 mm right column, identity→project→drawing→**sheet hero**→rev, ~4 internal
hairlines (the Pentagram default). **Skeleton B "Ledger"** — ~32 mm bottom band,
identity|title|control|**sheet hero**, 3 hairlines (won't scare a hardware engineer).

## Template + editor architecture

Fuse **KiCad geometry** (corner-anchor + `repeat/incr` stamping) + **Revit asset model**
(self-contained importable family + firm shared-parameter schema) + **AutoCAD formulas**
(FIELD/DIESEL) — routed through the substrate none of them have.

```rust
struct DrawingSheet {              // supersedes first-cut SheetTemplate
    uuid, name, sheet_size: SheetSize,
    profile: DrawingStandardProfile,   // Iso5457 | AsmeY14 | Custom
    margins, items: Vec<SheetItem>,    // primitives AND fields, ordered
    embedded_assets: Vec<AssetBlob>,   // logos, embedded for portability
    field_schema: FieldSchema,         // firm-extensible vocabulary (Revit shared params)
}
enum Anchor { TL,TC,TR, ML,C,MR, BL,BC,BR }   // 9-point (KiCad has 4)
struct Placement { anchor: Anchor, offset: Vec2Nm, rotation_mdeg }
enum SheetItem { Line, Rect, Polyline, Text, Image, Field(SheetField) }
```

- Anchoring is the load-bearing fix; two-point items anchor each endpoint (rules stretch
  with the page). Primitives carry KiCad `repeat + incr` (parametric zone rulers) + page
  scope `{AllPages, FirstOnly, ContinuationOnly}`. `Image` = **SVG preferred** (crisp in
  PDF/CAM) + PNG fallback. **Governed pool object**, not a loose file → provenance/diff/
  share/import for free.
- **Editor = a "Sheet Template" edit mode of the Datum canvas**, not a standalone app
  (reject KiCad `pl_editor` / Altium Draftsman fork). Direct manipulation; chosen anchor
  shows a **live tether** to its corner; inspector for anchor/offset/style/formula;
  field picker inserting/validating `${…}`; **live preview from a sample DesignModel** so
  the designer sees resolved values, not `${TITLE}` placeholders. Every edit = a typed op
  (`PlaceSheetItem`, `SetItemAnchor`, `SetFieldFormula`, `EmbedAsset`) — journaled with
  undo/provenance, **not locked behind reopen** (beats Altium).
- **Beautiful-by-default + BYO coexist**: ship a Datum house set (Skeleton A+B × A4/A3/A2
  + ANSI A/B/C, portrait+landscape, pre-wired bindings) as pool `DrawingSheet` entries;
  firms fork/swap (`SetSheetTemplate`), import their own (SVG logo embedded), and a
  **`.kicad_wks` importer** (same corner-anchor model) gives an instant starter library.
- **One template renders on schematic sheets AND fab/assembly drawings** — kills the
  OrCAD/Altium sch-vs-fab fork.

## Field + formula (data-binding) model — the seam for the owner's formulas

Replace flat `field_key: String` with a resolved binding:

```rust
struct SheetField { placement, style, binding: FieldBinding }
enum FieldBinding { Literal(Value), Ref(FieldPath), Computed(Expr) }
enum FieldPath { Project(k), Document(k), Sheet(k), Firm(k), System(k),
                 Revision(k), BindTime(k) }        // namespaced sources
enum Expr { Lit, Get(FieldPath), Concat(..), Format{value,pattern},
            SheetOf{scope}, DrawingNumber{scheme}, If{cond,then,els} }
```

- Binding classes: **A**uthored literal (title/checker), **C**omputed (project name /
  filename / n/N / scale-from-view), **H**ybrid firm-default→override (company/logo/
  classification).
- **Three computation rules the evidence forces:** (1) **n/N** — N is an *engine-owned
  aggregate* over the ordered sheet set (Revit "Total Sheets drifts" = anti-pattern),
  scope explicit (schematic-set vs board vs register); (2) **revision** = last qualifying
  row of the revision table (computed projection), authored fallback if no table; (3)
  **drawing number** = tokenized composite over a firm format
  (`[project]-[disc]-[type]-[serial]-[rev]`), serial = engine-owned monotonic register
  counter via journaled `allocate_drawing_number`.
- **Live-vs-frozen (protects Law 1):** computed fields are pure functions of journaled
  facts; wall-clock values are **captured at the release/issue op and stored, never
  re-sampled**; a truly-live clock field (if offered) is flagged non-deterministic and
  CAM-excluded.
- Resolution order (override chain): `sheet → document → project → firm-default →
  system/computed`; unbound → **visible marker**, never silent blank; each resolved field
  returns `(value, source_layer, binding_kind)`.
- **Typed-op boundary:** ops author only literals + bindings (`set_document_field`,
  `set_firm_default`, `define_field_binding`, `allocate_drawing_number`);
  Computed/Ref resolution is a **projection, never journaled**.
- On-canvas syntax: adopt KiCad `${…}` verbatim (familiar; near-direct `.kicad_wks`
  import); richer `Concat/Format/If/DrawingNumber` authored in the editor. **Owner's
  formulas plug in at the `Expr` tier** via `field_schema` (declare custom firm fields →
  join `Project/Document` namespaces → reference in `Computed(Expr)`), evaluated by the
  engine resolver, never a private read path.

## Documentation-control integration (firms impose their own control)

Model the **scheme**, not one convention; all journaled, role-gated ops:
- **`DocumentControlProfile`** (pool object, firm-authored, project-applied) — bundles
  `RevisionScheme`, numbering-formula template, release-state roles + signatory list,
  sheet-size series, imported title-block templates + assets. **The owner's "firm plugs
  in its own control" primitive.**
- **`RevisionScheme`**: `AlphaY14_35 | Numeric | Iso19650{status,rev} | CustomSequence`
  (ISO 19650 couples status+rev, both shown).
- **`ControlledDocument` + `DrawingRegister`** (ISO 9001 §7.5 / ASME Y14.34) —
  authoritative current-rev/state, number uniqueness. **`SheetSet`** — ordering + n/N.
- **Document `ReleaseState`** — lift the repo's existing six-state PLM machine
  (Draft→InReview→Approved→Released→Deprecated→Obsolete) from library objects to
  documents; transitions journaled+role-gated; **drives the `status` field as a read-only
  projection** (can't type a false status).
- **`RevisionHistory`** rows sourced from the journal / repo `EngineeringChangeOrder`
  (each commit batch can seed a row — one-mutation-path as a visible feature).
- **`Transmittal`** — issue/release record; the fab/assembly drawing-package export **is
  a transmittal** (IPC-D-325A required blocks; IPC-D-326 data package); ties to repo
  `DocumentRef.uri` + `data_egress_policy`.
- Framing: **Datum is the substrate a PLM wraps, not a PLM** — model abstract hooks, not
  vendor connectors. Journal + release-state + register + transmittal collectively = an
  ISO 9001 §7.5-conformant control surface.

## Fit to Datum
Rides the substrate cleanly (authoring/application/field-entry/state = typed ops;
resolution = projection). Binds to the locked Rendering Book tokens (labels→Mono muted
uppercase, values→Sans Cond Medium, sheet-hero→SemiBold, frame→`border.strong`,
dividers→`border.subtle`, status→`status.*`, accent reserved). render==CAM holds because
fields resolve to real IBM Plex filled-outline geometry; the live-vs-frozen rule protects
it. Standard-grounded (must be correct): sizes, 7 fields, rev skip-set, zone mechanics +
per-profile origin, line hierarchy, folded ≤180 mm, n/N, IPC-D-325 blocks, ISO 9001 §7.5.
Design-choice (Datum owns): all visual layout, Skeleton A/B, 9-point anchor, SVG logos,
`${}`+`Expr` grammar, `DrawingSheet` as pool asset, edit-mode editor, journal-sourced
revision rows, `DocumentControlProfile`.

## Next steps
- **Prototype (HTML, owner-approval gate before spec):** render Skeleton A "Strip" +
  Skeleton B "Ledger" at A3-landscape on vellum/ink in IBM Plex, exercising P1–P11 with
  realistic resolved values; an A4-portrait + ANSI-B variant to prove the anchor model;
  a borderless + a `status=PRELIMINARY` state.
- **Spec (after approval, each with governance ritual):** `DrawingSheet`/`Anchor`/
  `SheetItem`; the field-binding `Expr` grammar (numbered decision record — new
  mechanism); extended `SheetFrame`; doc-control objects; `.kicad_wks` importer; resculpt
  Rendering Book §8.

## Open questions for the owner
1. **Formula seam depth (key):** closed `Expr` enum (deterministic, gate-friendly —
   recommended) vs a small embeddable expression language (more power, needs a
   determinism/sandbox story to protect Law 1)?
2. **Default primary skeleton:** ship both — which is out-of-box: Strip (Pentagram) or
   Ledger (engineering-familiar)?
3. **Default revision scheme:** ASME Y14.35 alpha vs numeric vs ISO 19650 status+rev?
4. **Register scope:** project-scoped or firm-wide register/counter (where
   `allocate_drawing_number` lives)?
5. **Live-clock field:** offer a flagged/CAM-excluded live-date field, or forbid it
   (bind-time-frozen only)?
6. **Zone grid on schematics** by default, or only on fab/assembly outputs?
