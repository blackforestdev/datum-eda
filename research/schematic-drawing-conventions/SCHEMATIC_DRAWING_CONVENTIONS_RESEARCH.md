# Schematic & Drawing Conventions — Industry Survey & Datum EDA Implementation Strategy

> Phase 2 deep-dive on Domain 3 of the 8-domain standards audit.
> Continues from `research/standards-audit/STANDARDS_AUDIT.md § 3`
> ("Per-Domain Audit → 3. Schematic & drawing conventions").
> Cross-references `research/ipc-compliance/IPC_COMPLIANCE_RESEARCH.md`
> for the IPC-T-50 vocabulary baseline (do not re-survey) and
> `research/data-exchange-interop/DATA_EXCHANGE_INTEROP_RESEARCH.md`
> for schematic interop formats (EDIF, KiCad, Eagle, Altium SchDoc).
> Companion to `research/component-modeling/COMPONENT_MODELING_RESEARCH.md`
> for pin / part / library overlap.
>
> Reads against the post-Standards-Audit-Batch-1 spec baseline merged
> 2026-04-18 (PR #1). Quotes the Pending Exclusions Policy:
>
> > **Pending Exclusions Policy (verbatim, ratified 2026-04-17):**
> >
> > The audit's "Recommended low-priority / skip" list is an
> > **advisory exclusion** for Phase 2 work. Phase 2 agents MUST NOT
> > re-investigate these standards. Final ratification of skips into
> > binding scope documents happens in a single consolidated pass
> > after Domain 8 lands, when full cross-domain context is
> > available.
>
> For Domain 3 specifically, **none of the Phase 1 advisory-exclusion
> list items are Domain-3 standards.** The "skip" list is dominated by
> regulatory-vertical, PLM-platform, and SI-extraction standards that
> belong to Domains 4, 6, and 7. Domain 3 has no items off-limits.
> The two cross-references that **must not** be re-deep-dived because
> they are owned elsewhere are:
>
> - **IPC-T-50 vocabulary baseline** — already covered in
>   `research/ipc-compliance/IPC_COMPLIANCE_RESEARCH.md` § IPC-T-50
>   (lines 726-742). Cited and integrated; not re-surveyed.
> - **Schematic interop formats (EDIF, AltiumSchDoc, KiCad/Eagle
>   native)** — covered in `research/data-exchange-interop/`. Cited as
>   the carrier for symbol-style hint provenance; not re-surveyed.

## Executive Summary

- **The IEEE 315 vs IEC 60617 fork is real, regional, and load-bearing
  for native authoring.** US-trained engineers, MIL-STD-15 (military)
  shops, and most North-American industrial shops draw resistors as
  zigzags, logic gates as distinctive D-and-bubble shapes, and ground
  as a triangle of decreasing horizontal lines (IEEE 315 / ANSI Y32.2,
  with logic-gate sub-detail in IEEE Std 91-1984 / 91a-1991). European,
  Japanese (JIS C 0617 ≈ IEC), and most international-vertical shops
  draw resistors as rectangles, logic gates as rectangles with
  `&` / `1` / `≥1` qualifying labels, and ground as a horizontal bar
  (IEC 60617). Datum cannot pick one and ignore the other; per
  `STANDARDS_COMPLIANCE_SPEC.md` § 6.1 the answer is **both profiles
  plus an explicit `imported/custom` profile**, with profile state
  tracked at the Symbol record level (not the Project level — see § 6
  below for the rationale).

- **No Domain-3 standard is freely downloadable in full text.** IEEE
  Std 91/91a, IEEE Std 200, IEEE Std 991, ANSI Y32.2 / IEEE 315, ASME
  Y14.44, ISO 7200, ISO 5457, ISO 128, ISO 3098, IEC 60617, IEC 81346,
  IEC 81714 are all paywalled (IEEE Xplore, ASME store, ISO Webstore,
  IEC Webstore). The **only freely-readable normative source for
  electrotechnical vocabulary** is IEC 60050 Electropedia
  (`std.iec.ch/iev/`), which is searchable HTML, free, and authoritative.
  This is a real cost issue: implementing Domain-3 standards either
  requires per-standard purchase ($100-$400 each) or working from
  textbook digests + competitor implementations + community
  reverse-engineering. Purchase budget should be planned.

- **The IEC 60617 graphic-symbol database moved from a sold PDF to an
  online subscription database in 2012, and is the most expensive
  symbol-style standard to access.** IEC 60617 was historically
  published as a 12-part PDF (IEC 60617-2 through -13 — there is no
  -1, the introduction is in -DB). Since 2012 IEC sells it as a paid
  online database (`std.iec.ch/iec60617/`) at roughly CHF 1500/year
  per single-user subscription. The IEEE 315 standard has been stable
  since 1986 (no further amendments) and is sold as a one-time PDF
  at roughly USD 318. **For Datum's purposes, the practical move is
  to render symbols from KiCad's bundled libraries — KiCad ships both
  IEEE-style (`Device.kicad_sym`) and IEC-style symbols and they are
  CC-BY-SA-4.0 licensed (see § 9 for the license issue this raises
  for Datum).**

- **KiCad's symbol library is CC-BY-SA-4.0, not GPL-3, but the
  copyleft side-effect is identical from Datum's perspective and the
  user's `feedback_no_copyleft_integration` rule applies.** Datum can
  reference KiCad's libraries for *aesthetic inspiration* (matching
  the visual style users already know) without licensing exposure,
  but it cannot **redistribute** KiCad's `.kicad_sym` files under a
  Datum-controlled license, and any rendered output that incorporates
  KiCad-symbol pixels carries the Share-Alike obligation. Datum's
  built-in symbol library must be authored from scratch (or sourced
  under a permissive license). This is the same pattern the IPC
  research established for IPC-7351 footprints: Datum re-derives the
  geometry from the published spec rather than copying tool output.

- **The reference-designator question has a clean answer: ASME Y14.44
  (the 2008 successor to IEEE 200-1975) is the modern controlling
  US standard; IEC 81346-2 is the international parallel; both are
  back-compatible with the everyday R/C/L/U/Q/D/J prefix tradition
  that every EDA tool already uses.** Datum's existing
  `Entity.prefix: String` (`specs/ENGINE_SPEC.md` § 1.2) is correctly
  scoped — the spec edit is to add a normative default-prefix table
  (drawn from ASME Y14.44 Appendix A) and a validator that flags
  drift from the table without forbidding override. The prefix table
  itself is small (~50 entries) and freely reproducible from
  publicly-cited summaries (the prefix list is not the copyrighted
  part of the standard — only the standard's specific text is).

- **ISO 7200 is short, English-language, and the only ISO Domain-3
  standard worth implementing first.** ISO 7200:2004 (with technical
  corrigendum 2007) is a 14-page document specifying ~20 mandatory and
  optional title-block fields (Title, Identification number, Document
  type, Issuing organisation, Revision indicator, Date of issue,
  Sheet number, Number of sheets, Status, Technical reference, etc.).
  Datum's existing `SheetFrame { title, revision, company,
  page_number }` (`specs/ENGINE_SPEC.md` § 1.4) covers four of those;
  six more (`document_number`, `sheet_count`, `date_of_issue`,
  `status`, `approver`, `reviewer`) need to land. The visual layout
  is **not** specified by ISO 7200 — only the data fields. Datum
  needs a separate template engine for the visual.

- **The bus-syntax question has a five-tool fork that needs harmonising.**
  KiCad uses `D[7..0]`, Altium uses `D[0:7]`, Cadence Capture / OrCAD
  uses `D<7..0>`, Mentor PADS uses `D[7:0]` *and* `D<7..0>` (both
  accepted), and Eagle uses `D[7..0]` (KiCad-compatible by lineage).
  Datum's spec already commits to `NAME[n]` and `NAME[a..b]` per
  `specs/SCHEMATIC_CONNECTIVITY_SPEC.md` § 4.5 (KiCad-compatible).
  The recommendation is **adopt KiCad syntax as canonical Datum
  authored form**, accept the other four during import (with explicit
  per-source-format normalisation), and emit the canonical form on
  export. This is a connectivity-spec text edit, not an architectural
  change.

- **The title-block template question is the largest under-specified
  area in the schematic editor spec.** Every surveyed tool (Altium,
  OrCAD, KiCad, Eagle, Horizon) ships some template format —
  Altium's `.SchDot`, KiCad's `.kicad_wks` (worksheet), OrCAD's
  `.dot` and `.opj` template fields — and each is incompatible with
  the others. Datum's spec has zero template surface today: there is
  a `SheetFrame` data record but no template that says "draw the
  ISO 7200 fields at these positions on a B-size sheet". The minimum
  viable model is a separate `SheetTemplate` pool entry referenced
  from `Sheet`, with a small set of bundled defaults (ANSI A/B/C/D,
  ISO A4/A3/A2/A1) and a permissive author-your-own surface.

- **Vocabulary alignment cost is non-trivial but bounded.** A grep of
  the Datum CLI / MCP / spec surface for IPC-T-50 deviations
  (annular ring, courtyard, padstack, land, fillet, solder mask,
  silkscreen, etc.) finds the spec is largely T-50-aligned already,
  but the CLI command surface uses informal terms in places (e.g.
  `--soldermask` for mask, `--silk` for silkscreen) that should be
  audited against T-50's preferred forms. This is a one-pass
  vocabulary audit, not a research-extension question — flagged for
  follow-up rather than re-surveyed here.

- **Datum's existing hierarchy and net-label work is genuinely
  industry-equivalent.** The `Schematic` / `Sheet` /
  `SheetDefinition` / `SheetInstance` / `HierarchicalPort` model in
  `specs/SCHEMATIC_EDITOR_SPEC.md` § 2.1-2.4, paired with the
  `Local` / `Global` / `Hierarchical` / `Power` `LabelKind` enum
  (§ 2.4), maps cleanly onto Altium's hierarchical schematic, KiCad's
  hierarchical sheets, and OrCAD's hierarchical blocks. The spec's
  classification of these as `Implemented` in
  `STANDARDS_COMPLIANCE_SPEC.md` § 4.3 is correct.

- **Regulated industries mandate specific symbol-style profiles.**
  US military programs typically mandate MIL-STD-15-style symbols
  (which trace lineage to ANSI Y32.2 / IEEE 315). European industrial
  programs typically mandate IEC 60617. Japanese consumer electronics
  defaults to JIS C 0617 (which is essentially IEC 60617 with a few
  Japanese-specific symbols added). This means the symbol-style
  profile is not a pure aesthetic preference — it can be a
  contract-mandated deliverable. The data model needs to support
  both per-symbol and per-project profile assertion, with audit-log
  visibility per `STANDARDS_COMPLIANCE_SPEC.md` § 8 — flagged for
  Domain 4 as a cross-cutting requirement.

- **Title-block field overlap with PLM document control and ISO 9001
  audit-trail is large enough to flag for Domains 7 and 8.** ISO 7200's
  "approver", "reviewer", "release date", and "revision" fields are
  exactly the fields ISO 9001 / 21 CFR Part 11 / AS9100 expect to see
  on a release-controlled drawing. The title-block template engine is
  also the natural surface for PLM-driven field substitution
  (Windchill, Teamcenter, Aras can push `released_by`,
  `released_at_revision`, `eco_number` into the block). These
  cross-domain implications are flagged in § "Cross-Domain Insights"
  rather than relitigated.

## Standards Catalog

### Symbol-Style Standards

#### IEEE 315 (and IEEE 91 / 91a — distinctive logic shapes)

**Full title:** IEEE Std 315-1975 — *Graphic Symbols for Electrical and
Electronics Diagrams (Including Reference Designation Letters)*. Also
issued as ANSI Y32.2-1975 and CSA Z99-1975.

**Revision history:** Original 1975. Supplement 315A-1986 added
microelectronic symbols. **No revision since 1986** — the standard is
stable. There is no announced revision plan as of 2026.

**Issuing body:** Institute of Electrical and Electronics Engineers
(IEEE) and American National Standards Institute (ANSI). Maintained
within the IEEE Standards Association.

**Adoption status (2026):** Mainstream in North America (US, Canada);
common in Mexico and parts of Latin America. Required by reference in
many MIL-STD documents (MIL-STD-15A is the historical military symbol
standard, largely superseded by IEEE 315 + IEEE 91). Effectively
universal in US-trained EDA workflows.

**License / IP:** **Paywalled.** Available from IEEE Xplore at
approximately USD 318 (single-user PDF). The full normative text of
both IEEE 315-1975 and 315A-1986 is not freely available. Several
secondary sources (textbooks, Wikipedia, university course material)
reproduce the symbol shapes; the **shapes themselves are not
copyrightable** (functional graphic conventions), but the **specific
drawings as published** are copyrighted as part of the standard.

**Logic-symbol detail (IEEE 91 / 91a):** IEEE Std 91-1984 *Graphic
Symbols for Logic Functions* and supplement IEEE Std 91a-1991
specify logic-gate symbology in two parallel forms:

- **Distinctive-shape form** — the AND symbol (D-shape), OR symbol
  (curved-back), inverter (triangle + bubble), buffer (triangle), XOR
  (OR + curve). The form most North-American engineers recognise.
- **Rectangular-shape form** — rectangles labelled with `&` (AND),
  `≥1` (OR), `=1` (XOR), `1` (buffer), with active-low indicated by
  bubble or polarity-triangle prefix on the pin. Aligns with IEC 60617-12.

IEEE 91 explicitly permits both forms; IEEE 315 references IEEE 91 for
logic. Both standards are paywalled at IEEE Xplore (~USD 318 each).

**Key symbol families covered:**
- Resistors (zigzag, US-style)
- Capacitors (parallel-plate, with curved bottom plate denoting
  polarity for electrolytics)
- Inductors (curlicue / loop sequence; air-core vs ferrite by line
  detail)
- Transistors (BJT with emitter arrow, FET with gate-channel notch)
- Diodes (triangle-bar; LED with arrows; zener with notched bar; Schottky
  with squared-S notch)
- Logic gates (distinctive-shape per IEEE 91; rectangular permitted)
- Power and ground (triangle ground, slash chassis, bar earth)
- Connectors (varying — circle, ellipse, rectangle conventions)

**Visual conflicts with IEC 60617:**
- Resistors: zigzag (IEEE) vs filled rectangle (IEC).
- Inductors: looped curls (IEEE) vs row of bumps (IEC).
- Logic gates: distinctive shape (IEEE) vs rectangular with qualifying
  symbol (IEC, also permitted by IEEE 91).
- Ground: triangle of decreasing lines (IEEE signal ground) vs
  horizontal-bar with vertical lines (IEC chassis).

**Datum coverage:** `Planned` per `STANDARDS_COMPLIANCE_SPEC.md` § 6.1.
The `Symbol` pool record has primitives (`Vec<Primitive>`, see
`ENGINE_SPEC.md` § 1.1a) that are style-agnostic — Datum draws what's
in the primitives, regardless of style. The required surface is a
`SymbolStyleProfile` enum on the Symbol record, not a re-render of
geometry. Confirmed.

**Datum implementation cost (IEEE 315 profile):**
- Data-model: low (one enum field on `Symbol`)
- Library content: medium-high (Datum cannot redistribute KiCad's IEEE
  symbols; either license a third-party permissive library, author
  in-house, or render minimal primitives only)
- Validator: low (the validator just checks the enum is set)

**Strategic recommendation:** Implement **profile classification now**
(low cost), **defer authoring of a comprehensive in-house IEEE-style
symbol library to post-M7** (high cost, low immediate user value
because users import KiCad/Eagle libraries that already carry their
own style).

**Risks/edge-cases:**
- Multi-style projects: a project that mixes IEEE-style power symbols
  with IEC-style logic gates (common in international design houses
  with US and European designers on the same team) — the profile must
  be per-Symbol, not per-Project.
- Mid-project style switches: regulatory-mandated style switches
  (e.g., a project transitions from internal review to MIL-STD review)
  must be representable as a transformation, not a re-author.

#### IEC 60617 (12-part international graphic symbols)

**Full title:** IEC 60617 — *Graphical symbols for diagrams*.

**Structure:** Historically published as 12 parts (Part 2 through Part
13; there is no Part 1). Since 2012 the IEC has migrated the
publication to an online database (`std.iec.ch/iec60617`). The 12-part
structure is preserved as topic chapters within the database.

| Part | Topic |
|------|-------|
| 60617-2 | Symbol elements, qualifying symbols and other symbols having general application |
| 60617-3 | Conductors and connecting devices |
| 60617-4 | Basic passive components |
| 60617-5 | Semiconductors and electron tubes |
| 60617-6 | Production and conversion of electrical energy |
| 60617-7 | Switchgear, controlgear and protective devices |
| 60617-8 | Measuring instruments, lamps and signalling devices |
| 60617-9 | Telecommunications: switching and peripheral equipment |
| 60617-10 | Telecommunications: transmission |
| 60617-11 | Architectural and topographical installation plans and diagrams |
| 60617-12 | Binary logic elements |
| 60617-13 | Analogue elements |

Part 12 (binary logic elements) is the IEC peer to IEEE Std 91 and
specifies rectangular logic-gate symbology with qualifying labels (`&`,
`≥1`, `=1`, etc.) and pin-polarity indicators (bubble or triangle).

**Issuing body:** International Electrotechnical Commission (IEC),
Geneva.

**Revision status:** Continuous revision via the database. Last
print-edition revisions of individual parts ranged from 1996 (Part 11)
to 2002 (Part 12). The IEC Database adds new symbols on a rolling
basis (most recently power-electronics symbols for SiC/GaN devices,
2022-2024).

**Adoption status (2026):** Mainstream in Europe, the UK, India,
Australia, Japan (JIS C 0617 is essentially IEC 60617 + Japanese
extensions). Required for export drawings to most European industrial
customers. Common in international consulting work.

**License / IP:** **Paywalled, subscription model.** The IEC 60617
database is sold per-seat, per-year. CHF ~1500/year for a single-user
license; site licenses negotiated. **Re-distribution of symbol PNG/SVG
exports is restricted by the database license.** This is the most
expensive Domain-3 standard to access and the most legally constrained
for redistribution.

**Practical workaround:** The symbol *shapes* (not the IEC's specific
drawings) are functional and not copyrightable. Datum can author
IEC-compliant symbols from textbook references or competitor
inspection (KiCad's IEC libraries; gEDA's gschem libraries). The
risk is misinterpretation; the safest reference for any contested
symbol is to consult the IEC database via a paid subscription before
shipping.

**Datum coverage:** `Planned` per `STANDARDS_COMPLIANCE_SPEC.md` § 6.1.
Same data-model surface as IEEE 315 — one enum entry per Symbol.

**Datum implementation cost (IEC 60617 profile):**
- Data-model: zero additional cost (same enum field as IEEE 315)
- Library content: medium-high (cannot redistribute IEC content
  directly; cannot redistribute KiCad's IEC library either due to
  CC-BY-SA-4.0; must source permissively or author in-house)
- Validator: low

**Strategic recommendation:** Same as IEEE 315 — implement profile
classification now, defer comprehensive in-house IEC library to
post-M7. **Budget for a one-seat IEC 60617 database subscription**
(approximately CHF 1500/year, or one-time CHF ~500 if a single
person can do all symbol work in a focused window) **before any in-
house IEC-symbol authoring begins.**

#### DIN 40700/40717/40900 (German national, largely IEC-aligned)

**Full title:** DIN 40700 — *Sinnbilder für Schaltungsunterlagen*
(Symbols for circuit documentation), and family standards DIN 40717,
DIN 40900 covering specific component classes.

**Status (2026):** **Largely superseded by IEC 60617.** DIN withdrew
the original 40700/40717/40900 series in favour of DIN EN 60617 (the
DIN-published German edition of IEC 60617) starting in 1997. Modern
German engineering offices use DIN EN 60617. Older drawings (pre-
2000) sometimes still use DIN 40700 conventions and may need to be
recognised on import.

**Issuing body:** Deutsches Institut für Normung (DIN). Now a German
National Adoption (DNA) of IEC content.

**License / IP:** Paywalled at Beuth Verlag (DIN's publisher);
roughly EUR 100-300 per part.

**Datum coverage:** `Out of scope` recommendation. Treat DIN 40700 as
"legacy IEC 60617" — if Datum supports IEC 60617 it covers the
modern German use case. **No separate Datum profile needed.** Imported
drawings using DIN 40700-specific symbols would map to the IEC 60617
profile with a note about source vintage.

**Strategic recommendation:** Skip as a distinct profile. Mention in
the disposition table only.

#### ANSI Y32.16 / IEEE Std 91 logic-symbol detail

**Full title:** ANSI Y32.16 — *Reference designation letters for use
on electrical/electronics diagrams* (later subsumed into IEEE 200,
then ASME Y14.44; see Reference-Designator section below). The
Y32-series prefix originally covered both symbol style (Y32.2 = IEEE
315) and reference designation (Y32.16 = IEEE 200).

**Logic-symbol form distinction:** IEEE Std 91/91a (cross-referenced
above under IEEE 315) is the controlling logic-gate standard. The
choice between distinctive-shape and rectangular-shape logic forms is
**permitted within either IEEE 91 or IEC 60617-12**, so a Datum
profile that says "IEEE 315" might still author rectangular logic
gates if the user prefers; the symbol-style enum needs to be a
profile name, not a per-shape rule.

**Datum coverage:** Reference-only; the enum profile is the surface.
The fine-grained "distinctive vs rectangular" choice is a property of
each individual logic Symbol, not the project profile. Recommend
adding a `LogicSymbolForm` enum (`DistinctiveShape | Rectangular |
Mixed`) to the `Symbol` record for logic gates only, defaulting to
the project's profile setting.

### Reference-Designator Standards

#### IEEE 200 (historical)

**Full title:** IEEE Std 200-1975 — *Reference Designations for
Electrical and Electronic Parts and Equipment*. Also issued as ANSI
Y32.16-1975.

**Status (2026):** **Withdrawn / superseded by ASME Y14.44.** IEEE
withdrew Std 200 in 1996. ASME Y14.44 replaced it as the normative
US standard. Old drawings (pre-1996) cite IEEE 200; modern designs
should cite ASME Y14.44.

**License / IP:** Standard is no longer for sale through IEEE Xplore.
Historical PDFs available through some technical-library archives.

**Datum coverage:** Reference-only. Datum should treat "IEEE 200" as
an alias for the same underlying prefix table that ASME Y14.44 carries
forward. The MCP / CLI surface should accept both designations as
equivalent (a courtesy to users with legacy MIL specs that cite IEEE
200).

#### ASME Y14.44 (current US standard)

**Full title:** ASME Y14.44-2008 — *Reference Designations for
Electrical and Electronic Parts and Equipment*. Reaffirmed 2014;
current revision is ASME Y14.44-2024 (issued late 2024).

**Status (2026):** **Mainstream US standard.** Cited by reference in
most US MIL-STD documents that touch electronics drawings. Default
for any modern US-market product.

**Issuing body:** American Society of Mechanical Engineers (ASME).

**License / IP:** Paywalled at the ASME store, approximately USD 96
for a PDF. The **prefix table itself (the list of letters and their
meanings)** is not subject to copyright protection because it is a
list of facts; many secondary sources (textbooks, internet
references) reproduce it freely. The standard's specific text is
copyrighted; the table content is public.

**Prefix table (representative subset, ASME Y14.44-2008 Appendix A,
publicly cited):**

| Prefix | Class |
|--------|-------|
| `A`  | Assembly, separable subassembly |
| `AT` | Attenuator, isolator, terminator |
| `B`  | Motor, vibrator |
| `BT` | Battery |
| `C`  | Capacitor |
| `CB` | Circuit breaker |
| `CP` | Coupler (directional, hybrid) |
| `D`  | Diode (including LED, Zener, Schottky), thyristor (was `CR` historically) |
| `DC` | Directional coupler (alt to `CP`) |
| `DL` | Delay line |
| `DS` | Display, indicator (lamp, LED indicator) |
| `E`  | Miscellaneous electrical part |
| `F`  | Fuse |
| `FB` | Ferrite bead |
| `FL` | Filter |
| `G`  | Generator, oscillator (rotating; for non-rotating osc see `Y`) |
| `H`  | Hardware (mounting, heatsink — when assigned a designator) |
| `HY` | Circulator |
| `J`  | Connector, jack (fixed) |
| `JP` | Jumper, link |
| `K`  | Relay |
| `L`  | Inductor, coil |
| `LS` | Loudspeaker |
| `M`  | Meter |
| `MK` | Microphone |
| `MP` | Mechanical part (when designated) |
| `N`  | Numeric indicator (rare) |
| `P`  | Connector, plug (movable) |
| `PS` | Power supply |
| `Q`  | Transistor (BJT, FET, IGBT) |
| `R`  | Resistor |
| `RT` | Thermistor |
| `RV` | Varistor |
| `S`  | Switch |
| `T`  | Transformer |
| `TC` | Thermocouple |
| `TP` | Test point |
| `U`  | Inseparable assembly, integrated circuit |
| `V`  | Vacuum tube, gas-filled tube |
| `VR` | Voltage regulator (discrete) |
| `W`  | Cable, wire harness |
| `X`  | Socket (for `Q`/`U`/`V`) — `XQ`, `XU`, `XV` |
| `Y`  | Crystal, oscillator (non-rotating) |
| `Z`  | Network (passive) |

(Pre-Y14.44 historical: `CR` was diode, replaced by `D` in Y14.44.
Some shops still use `CR`; the validator should accept both with a
warning if the project profile selects modern Y14.44.)

**Datum coverage:** `Planned` per `STANDARDS_COMPLIANCE_SPEC.md` §
6.2. The `Entity.prefix: String` field
(`specs/ENGINE_SPEC.md` § 1.2) is correctly scoped — the spec edit
is to add a normative default-prefix table (the table above can ship
as a JSON resource bundled with Datum) plus a project-level setting
that selects which prefix profile (Y14.44-2024, Y14.44-2008, IEEE
200-legacy, IEC 81346, custom).

**Datum implementation cost:**
- Data-model: low (one enum on Project / one optional field on
  `Symbol` for prefix-override audit trail)
- Validator: low-medium (validate `Entity.prefix` against the active
  profile's table; emit a warning, not an error, on drift)
- Library content: low (the table ships as a JSON resource; no
  per-symbol authoring effort)

**Strategic recommendation:** **Implement now.** Cheapest of all
Domain-3 must-haves; high user-confidence value because designators
are visible on every schematic and BOM.

#### IEC 81346 (multi-part international system)

**Full title:** IEC 81346 — *Industrial systems, installations and
equipment and industrial products — Structuring principles and
reference designations*. Multi-part (currently parts -1, -2, -10, -12
in active revision).

**Key parts:**
- IEC 81346-1:2009 — Basic rules
- IEC 81346-2:2019 — Classification of objects and codes for classes
  (the part that defines designator prefixes — single letters with
  optional category subdivision)
- IEC 81346-10:2022 — Power supply systems

**Issuing body:** IEC, jointly with ISO (the standard is ISO/IEC
81346 in some publications).

**Adoption status (2026):** Mainstream in European industrial
process and power-systems work; less common in board-level
electronics. Process-engineering and power-distribution drawings in
Europe routinely use IEC 81346. Datum's primary user base
(board-level) will encounter it occasionally for industrial-product
work but it is not a daily concern.

**License / IP:** Paywalled at IEC Webstore, approximately CHF
~300 per part.

**Designator scheme:** IEC 81346-2 uses a single uppercase letter
with optional sub-letter; the letter codes overlap substantially with
ASME Y14.44 (R=resistor, C=capacitor, L=inductor, etc.) but with
some differences (E for "Electrical equipment, miscellaneous" rather
than "Engineering hardware"; X for "Transmission/transducer" rather
than "Socket").

**Datum coverage:** `Planned` (alongside ASME Y14.44) per
`STANDARDS_COMPLIANCE_SPEC.md` § 6.2. The same prefix-profile
mechanism handles IEC 81346 — Datum ships a second JSON prefix table
and selects between them by project profile.

**Strategic recommendation:** **Implement table only.** No per-symbol
work. Selection is a project-level profile choice.

#### DIN 40719

**Full title:** DIN 40719-2 — *Code letters for designation of
elements*.

**Status (2026):** Superseded in Germany by DIN EN 81346-2 (the
German national adoption of IEC 81346-2). DIN 40719-2 is withdrawn.

**Datum coverage:** Out of scope as a distinct profile. Mention only
for legacy-import recognition.

#### Common designator prefix tables — alignment with Datum

Datum's pool already uses single-letter prefix conventions
(`R/C/L/U/Q/D/J/K`) by default. The recommended ASME Y14.44 default
table above aligns with Datum's existing usage and with KiCad / Eagle
/ Altium / OrCAD library defaults; **no per-symbol re-author is
required**, only the addition of a normative default table and a
validator that flags drift.

### Drawing / Sheet Standards

#### ISO 7200 (title-block data fields)

**Full title:** ISO 7200:2004 — *Technical product documentation —
Data fields in title blocks and document headers*. Technical
corrigendum 2007 (clarification, no field changes).

**Status (2026):** Mainstream international standard for title-block
data content (NOT visual layout). Cited by reference in most ISO
drawing standards (ISO 5457, ISO 128) and by ASME Y14.1 by analogy.

**Issuing body:** ISO TC 10 (Technical product documentation).

**License / IP:** Paywalled at ISO Webstore, approximately CHF ~110
per PDF.

**Mandatory fields (ISO 7200 § 4.1):**
- Title (free text)
- Identification number (document number / part number / drawing number)
- Document type (drawing, list, report, specification…)
- Issuing organisation (legal name + organisation symbol/logo)
- Revision indicator (alphanumeric or numeric — both permitted)
- Date of issue (release date)
- Sheet number / Total number of sheets (ISO 5457 expression: `n/N`)

**Optional fields (ISO 7200 § 4.2):**
- Status (e.g. "Preliminary", "Released", "Obsolete")
- Technical reference (originator engineer)
- Approver (release authority signature placeholder)
- Reviewer (peer review signature placeholder)
- Title supplement / continuation
- Language code
- Page format (A0/A1/A2/A3/A4 reference)
- Document classification (ITAR / EAR / IP-confidential markings —
  cross-domain to Domain 4)
- Copyright statement
- Project number / Contract number
- Customer identification

**Datum coverage:** `Planned` per `STANDARDS_COMPLIANCE_SPEC.md` § 6.3.
Existing `SheetFrame` (`ENGINE_SPEC.md` § 1.4) carries `title`,
`revision`, `company`, `page_number`. **Six mandatory or commonly-
required fields are missing**: `document_number`, `document_type`,
`sheet_count`, `date_of_issue`, `status`, `approver` /
`reviewer` placeholders.

**Datum implementation cost:**
- Data-model: low (one struct extension; six new optional fields)
- Persistence: trivial (additive JSON schema bump)
- Title-block template engine: medium-high (NEW SUBSYSTEM — see § 6
  cross-cutting topics)
- Validator: low (presence-of-mandatory-field check)

**Strategic recommendation:** **Extend `SheetFrame` now**; ship the
template engine post-M7. Document-number / sheet-count / date-of-issue
are minimum-viable fields users need on day one; the visual template
can be deferred until GUI work resumes.

#### ANSI Y14.1 / ANSI Y14.1M (sheet sizes US)

**Full title:** ANSI/ASME Y14.1-2020 — *Drawing Sheet Size and
Format* (decimal-inch). Companion ANSI/ASME Y14.1M-2020 (metric).

**Status (2026):** Mainstream US drawing standard; defines the A
through F (and J) sheet-size series:

| Designator | Decimal inch | Approximate metric |
|------------|--------------|--------------------|
| A | 8.5 × 11 | 216 × 279 mm |
| B | 11 × 17 | 279 × 432 mm |
| C | 17 × 22 | 432 × 559 mm |
| D | 22 × 34 | 559 × 864 mm |
| E | 34 × 44 | 864 × 1118 mm |
| F | 28 × 40 | 711 × 1016 mm |
| J | Roll size, 36 wide | 914 × ?? mm (continuous) |

**Issuing body:** ASME (the Y14 series is ASME-controlled despite the
ANSI prefix history).

**License / IP:** Paywalled at ASME store, approximately USD ~90.

**ANSI Y14.1M (metric) sheet sizes:**

| Designator | Metric |
|------------|--------|
| A0 | 841 × 1189 mm |
| A1 | 594 × 841 mm |
| A2 | 420 × 594 mm |
| A3 | 297 × 420 mm |
| A4 | 210 × 297 mm |

(Same ISO 216 / ISO 5457 series, adopted into US use under the M
suffix.)

**Datum coverage:** `Reference-only` per
`STANDARDS_COMPLIANCE_SPEC.md` § 4.3 ("until native sheet-size
enforcement is specified"). Acceptable v1 stance.

**Datum implementation cost:**
- Data-model: low (one `SheetSize` enum with the values above + a
  `Custom { width, height }` variant)
- Validator: low (presence check; mismatch warning vs the project
  default)
- Template engine binding: medium (templates must declare their
  intended sheet size)

**Strategic recommendation:** **Add the `SheetSize` enum now** (as
part of the `SheetTemplate` work — see § 6 below); skip
enforcement until GUI sheet-frame rendering work begins.

#### ISO 5457 (sheet sizes ISO)

**Full title:** ISO 5457:1999 — *Technical product documentation —
Sizes and layout of drawing sheets*.

**Status (2026):** International standard for drawing sheet sizes.
Defines the ISO 216 A-series (A0–A6) plus extended sizes (A0×2,
A0×3, etc.). Mainstream in Europe, Asia, Australia, Africa, Latin
America.

**Sheet sizes (subset relevant to PCB schematics):**

| Designator | Trimmed mm | Drawing area mm |
|------------|------------|-----------------|
| A0 | 841 × 1189 | 821 × 1159 |
| A1 | 594 × 841 | 574 × 811 |
| A2 | 420 × 594 | 400 × 564 |
| A3 | 297 × 420 | 277 × 390 |
| A4 | 210 × 297 | 180 × 277 |

(Drawing area = trimmed area minus the standard 10 mm border + 20 mm
filing margin on the binding edge.)

**Issuing body:** ISO TC 10.

**License / IP:** Paywalled at ISO Webstore, approximately CHF ~110.

**Datum coverage:** Same as ANSI Y14.1 — fold into `SheetSize` enum
with both ISO and ANSI variants. `Reference-only` until sheet-frame
rendering work resumes.

#### ISO 128 (presentation principles)

**Full title:** ISO 128 (multi-part) — *Technical product
documentation — General principles of presentation*. Parts include
ISO 128-1 (terms / definitions / arrangement of views), ISO 128-2
(line types, weights), ISO 128-15 (sectional views).

**Status (2026):** Mainstream international drawing standard; cited
by reference in nearly every ISO drawing standard. Less directly
applicable to schematics (it is a mechanical-drawing standard) but
the line-type and lettering principles touch schematic title blocks
and revision tables.

**Issuing body:** ISO TC 10.

**License / IP:** Paywalled at ISO Webstore, approximately CHF ~110
per part.

**Datum coverage:** `Reference-only`. Recommend mentioning ISO 128
line-type compliance as an aspirational target for the schematic
renderer (M7+) but not requiring it in v1. The schematic line types
Datum currently uses (continuous for wires, dashed for hierarchical
borders) align with ISO 128 conventions by default.

**Strategic recommendation:** Skip explicit implementation. Mention
in cross-cutting sheet-template guidance.

#### ANSI/ASME Y14.5 (GD&T — note only)

**Full title:** ASME Y14.5-2018 (and prior 2009, 1994 revisions) —
*Dimensioning and Tolerancing*. **The Geometric Dimensioning and
Tolerancing standard.**

**Status (2026):** Mainstream US mechanical-drawing standard.
Largely irrelevant to schematics; cited here only because the
title-block conventions defined in Y14.5 (revision blocks, drawing
notes, tolerance blocks) sometimes appear on PCB fabrication
drawings.

**Datum coverage:** Out of scope for the schematic editor. **Will
matter** for PCB fabrication-drawing output (post-M7) — flagged for
Domain 1 / Domain 6 cross-reference (the impedance / mechanical
fabrication drawings produced from board data may need GD&T
notation). Not in Domain 3 scope.

#### DIN 6771

**Full title:** DIN 6771 — *Schriftfeld für Zeichnungen, Pläne und
Listen* (Title block for drawings, plans and lists).

**Status (2026):** **Withdrawn**, replaced by DIN EN ISO 7200 (the
DIN edition of ISO 7200). Older German engineering drawings cite
DIN 6771; modern ones cite DIN EN ISO 7200.

**Datum coverage:** Out of scope as a distinct profile. Treat as
"legacy ISO 7200" for import-recognition purposes only.

#### ISO 3098 (lettering)

**Full title:** ISO 3098 (multi-part) — *Technical product
documentation — Lettering*.

**Status (2026):** International standard for technical-drawing
lettering. Specifies font shapes (Type A, Type B), heights (1.8, 2.5,
3.5, 5, 7, 10, 14, 20 mm), and stroke weights. Largely a mechanical-
drawing concern; schematics typically use proportional fonts that do
not conform to ISO 3098.

**Datum coverage:** Out of scope for v1 schematic. The schematic
text in Datum (`SchematicText` per `ENGINE_SPEC.md` § 1.4) does not
specify font; rendering is consumer-determined. ISO 3098 conformance
would be a M7+ rendering option.

**Strategic recommendation:** Skip. Add as a renderer option much
later if a regulated user demands it.

### Schematic-Specific Conventions

#### Hierarchical sheets and ports

**Industry survey:**

- **Altium Designer**: hierarchical schematics use *sheet symbols* on
  a parent sheet, each containing port pins that reference a child
  schematic file. Multi-instance is supported via "Repeat" syntax in
  port labels. Hierarchical port direction (Input / Output /
  Bidirectional / Passive) is enforced.

- **KiCad (8.x)**: hierarchical sheets are placed as rectangular
  blocks on the parent; each block has hierarchical pins; child
  sheets carry hierarchical labels with matching names that connect
  to those pins. Multi-instance (KiCad 7.0+) supports the same
  child sheet placed multiple times with separate annotation paths.

- **Cadence Capture**: hierarchical blocks (`hier blocks`); each
  hier block references a sub-schematic; ports on the sub-schematic
  expose connection pins on the parent block. Multi-instance via
  "PCB Footprint" mapping per instance.

- **OrCAD X Presto** (modern Capture replacement): similar to
  Capture; "Hierarchical Block" and "Hierarchical Port" objects.

- **PADS Logic**: hierarchical blocks via the "Off-Page Connector"
  primitive (PADS treats hierarchy as a special case of off-page
  connection).

- **Eagle (legacy and Fusion Electronics)**: **Eagle has no
  hierarchical schematic support in the legacy version** — all
  Eagle schematics are flat, multi-sheet only. Fusion Electronics
  (post-2017 rewrite) added basic hierarchy.

- **Horizon EDA**: hierarchical blocks ("Block" type), ports,
  multi-instance. Aligned with Datum's model.

- **LibrePCB**: hierarchical blocks added in v1.0 (2024).

- **DipTrace, EasyEDA**: hierarchical sheets supported but limited;
  EasyEDA Pro is the only EasyEDA tier with full hierarchy.

**Datum's model** (`SCHEMATIC_EDITOR_SPEC.md` § 2.1-2.4):
- `SheetDefinition` (the reusable definition; one per logical
  hierarchy node)
- `SheetInstance` (the placement; multiple instances per definition)
- `HierarchicalPort` (port on a child sheet; appears as a pin on the
  parent's `SheetInstance` block)
- Resolution rules in `SCHEMATIC_CONNECTIVITY_SPEC.md` § 4.4

**Datum coverage:** `Implemented` per `STANDARDS_COMPLIANCE_SPEC.md`
§ 4.3. Aligned with industry standard. Confirmed correct.

**Notes / risks:**
- The `HierarchicalPort.direction: PortDirection` enum
  (`ENGINE_SPEC.md` § 1.1a, values `Input | Output | Bidirectional |
  Passive`) does not match every tool's set (Altium adds `Power`,
  KiCad adds `Tri-state`). Datum's set is the smallest consistent
  set; richer direction labels can be authored as `SymbolField`
  metadata if needed.
- Multi-instance child-sheet net-resolution is correctly handled per
  `SCHEMATIC_CONNECTIVITY_SPEC.md` § 4.4 ("each instance gets
  independent net resolution"). This is the same semantic as KiCad
  7.0+ and Altium hierarchical schematics.

#### Multi-sheet project conventions

**Sheet naming:** Industry-wide convention is
"<filename>_<short-purpose>" (e.g., `top.sch`, `power.sch`,
`mcu.sch`). Page numbering: `n/N` format (sheet 3 of 7) is the
ISO 7200 / ASME Y14.1 convention.

**Datum's model:** `Sheet.name: String` (free-form);
`SheetFrame.page_number: Option<String>` (free-form). The
`page_number` field is permissive, allowing `3/7`, `Page 3`, or any
custom format. **Recommendation: add `Sheet.sheet_index: Option<u32>`
and `Sheet.sheet_count: Option<u32>` to support deterministic
`n/N` rendering**, while preserving `page_number` for free-form
override.

**Per-tool comparison:**

| Tool | Sheet naming | Page numbering |
|------|--------------|----------------|
| Altium | Free-form, recommended `.SchDoc` per logical sheet | `n` of `N` (auto-computed) |
| KiCad | `.kicad_sch` per sheet, hierarchy via filename or in-file | `1/N`, `2/N`, … (auto-numbered along hierarchy traversal) |
| OrCAD Capture | `.dsn` project, sheets within the project file | `n/N` (auto) |
| PADS Logic | `.sch` project file with sheet pages | Per-sheet user-set |
| Eagle (legacy) | Multi-sheet `.sch` (one file, multiple sheets) | Per-sheet user-set |

**Recommendation:** Datum's existing per-sheet UUID-keyed JSON file
layout (`schematic/sheets/<uuid>.json` per
`NATIVE_FORMAT_SPEC.md` § 4) aligns with KiCad's per-file
convention. Adopt KiCad-style auto-numbered traversal as the
deterministic-numbering algorithm; preserve user override.

#### Bus syntax (per-tool comparison)

**Survey:**

| Tool | Range form | List form | Notes |
|------|------------|-----------|-------|
| KiCad | `D[7..0]`, `D[0..7]` | `{SDA, SCL}` | Both directions accepted |
| Altium | `D[0:7]` | `{NETA,NETB}` (Altium "Net Identifier" expansion) | Colon-separated range, lowest-first by convention |
| Cadence Capture | `D<7..0>` | `BUS = {SDA, SCL}` | Angle-bracket range |
| OrCAD X Presto | `D<7..0>` (inherits Cadence) | (same) | (same) |
| PADS Logic | `D[7:0]` and `D<7..0>` | `BUS = {…}` | Both bracket forms accepted |
| Eagle | `D[7..0]` | `BUS = {…}` (sometimes `D@5` for tap) | KiCad-compatible by lineage |
| Horizon EDA | `D[0..7]` | `{NET_A, NET_B}` | KiCad-compatible |
| LibrePCB | `D[7..0]` | (same) | KiCad-compatible |
| EasyEDA | `D[0:7]` | (limited) | Altium-compatible |

**Datum's current commitment** (`SCHEMATIC_CONNECTIVITY_SPEC.md`
§ 4.5): scalar member `NAME[n]`, bus range `NAME[a..b]`. KiCad
syntax. This is correct.

**Recommendation:** Adopt **KiCad-compatible `D[a..b]` as canonical
authored form**. On import:
- KiCad / Eagle / Horizon / LibrePCB → already KiCad-compatible
- Altium → normalise `D[0:7]` to `D[0..7]` at import time
- Cadence / OrCAD → normalise `D<7..0>` to `D[7..0]` at import time
- PADS → accept both forms; normalise to `D[a..b]`

The normalisation should preserve the original syntax as a
`SymbolField` or per-sheet metadata attribute so the user can see
and re-emit the source form on round-trip export if desired.

**Spec edit:** Extend `SCHEMATIC_CONNECTIVITY_SPEC.md` § 4.5 with
the canonical-form declaration and the normalisation matrix above.
Cross-reference Domain 1 (data-exchange research) for the round-trip
fidelity requirements.

#### Net-label semantics (local / global / hierarchical)

**Datum's model:** `LabelKind` enum
(`ENGINE_SPEC.md` § 1.4: `Local | Global | Hierarchical | Power`).
Aligns with industry practice:

- `Local`: scoped to one sheet (KiCad "label", Altium "Net Label",
  OrCAD "Wire Label")
- `Global`: design-wide (KiCad "Global Label", Altium "Power Port"
  *or* "Global Net Identifier", OrCAD "Off-Page Connector")
- `Hierarchical`: parent/child (KiCad "Hierarchical Label", Altium
  "Sheet Entry", OrCAD "Hierarchical Port")
- `Power`: special-cased global with implied semantic class (KiCad
  "Power Port", Altium "Power Port", OrCAD "Power Symbol")

**Industry alignment:** Datum's enum maps cleanly. The semantics
in `SCHEMATIC_CONNECTIVITY_SPEC.md` § 4.2 / 4.3 ("Local labels
connect segments within the same sheet scope; Global labels connect
segments across all sheets; Hierarchical labels connect through
sheet ports only; Power symbols inject a named global net") match
all major tools.

**Note on Altium's "Off-Page Connector":** Altium has an additional
construct (the off-page connector, `OffPageConnector`) which is
visually distinct from a net label and connects only across sheets at
the same hierarchy level. KiCad treats this as a global-scope label
with no visual distinction. Datum's `LabelKind::Global` covers the
electrical semantic; the **visual rendering as off-page-connector
vs label** is a renderer choice, not a connectivity semantic. Flag
for M7 consideration: Datum's renderer may want an
`OffPageConnector` visual treatment for `Global` labels that are the
sole label on a wire crossing a sheet boundary.

**Datum coverage:** `Implemented` per `STANDARDS_COMPLIANCE_SPEC.md`
§ 4.3. Confirmed.

#### Power symbol conventions

**Visual conventions per profile:**

- **IEEE 315 (US):**
  - Earth ground: triangle of three decreasing horizontal lines
    (formal designation: "earth")
  - Signal ground: inverted triangle (single line at apex)
  - Chassis ground: angled-hatching ground or "rake" pattern
  - V+ / V-: arrow up / arrow down with label
  - Bus power (e.g., +3V3, +5V): "Y" or "T" shape with bar at top

- **IEC 60617 (international):**
  - Earth: horizontal line above three vertical lines (formal "earth")
  - Frame/chassis: horizontal line above slashes
  - Equipotential: circle with line through
  - V+ / V-: same arrow convention as IEEE

- **Common (both profiles):**
  - VCC, VDD, +3V3, +5V, +12V, -12V, GND, GNDA, GNDD: textual
    labels rather than fully iconographic forms; tool-rendered as a
    line + label

**Datum's model:** `LabelKind::Power`
(`ENGINE_SPEC.md` § 1.4) carries the semantic ("inject a named global
net"); the *graphic shape* is a `Symbol` from the pool with
appropriate primitives. Datum's pool can carry both IEEE-style and
IEC-style power symbols; the user picks at placement time.

**Recommendation:**
- Pool ships a small bundled set of universal-textual power symbols
  (`+3V3`, `+5V`, `GND`, `VCC`, `VDD`) with style-neutral
  rendering (just the label and a connection line)
- IEEE / IEC iconographic power symbols (the triangle, the bar, the
  arrow) ship as separate Symbol entries in the pool, classified by
  the same `SymbolStyleProfile` enum
- The validator flags placement-style mismatch only if the project
  explicitly opts in to style enforcement (warning, not error)

#### Net-tie / no-connect markers

**No-connect (NC) marker conventions:**

- **IEEE 315 / IEC 60617:** no specific symbol prescribed; the
  industry-de-facto symbol is a small "X" cross at the unconnected
  pin. Sometimes a small filled triangle.

- **Per-tool implementation:**
  - KiCad: explicit `NoConnect` object, drawn as a blue X
  - Altium: "No ERC" object, drawn as a coloured X
  - OrCAD: "No Connect" symbol, X glyph
  - Eagle: "Don't connect" pin attribute, no separate object
  - Horizon: explicit no-connect object; cross glyph

**Datum's model:** `NoConnectMarker { symbol, pin, position }`
(`ENGINE_SPEC.md` § 1.4) is correctly an explicit object targeting
a specific pin (matches KiCad, Altium, OrCAD, Horizon). ERC
suppression semantic is in `ERC_SPEC.md` § 4 rule 4
`noconnect_connected`. Confirmed.

**Net-tie conventions:**

A "net tie" is an explicit zero-impedance bridge between two named
nets — used for star-grounding, signal-ground / chassis-ground
bridges, single-point-grounding designs, etc. Industry-de-facto
visual: two pads on a single component-shaped object, named with
both net names; sometimes a small "T" or arc connecting them.

**Per-tool implementation:**
- KiCad: a `Net Tie` library footprint family ships with KiCad;
  symbols are special "net-tie" parts.
- Altium: `NetTie` part type; explicit electrical bridge.
- OrCAD: explicit `NETSHORT` / `NETLOCK` constraint.
- Horizon: net-tie part type.

**Datum's coverage:** **No spec mention of net-ties anywhere.**
This is a real gap if Datum targets professional power-electronics
work. However, the workaround (place a 0-ohm resistor between two
named nets) is industry-acceptable. Recommendation: defer net-tie
as a first-class object until M8+; document the 0-ohm-resistor
pattern as the v1 workaround.

#### Off-page / off-sheet references

**Conventions:**

- **KiCad**: hierarchical labels carry off-page reference visually
  via the label arrow shape; global labels carry off-page reference
  by name only.
- **Altium**: explicit `OffSheetConnector` primitive (separate from
  port symbols).
- **OrCAD**: off-page connectors (`OFFPAGELEFT-L`, `OFFPAGELEFT-R`)
  are explicit symbols with arrow/flag glyphs.

**Datum's coverage:** `LabelKind::Global` covers the electrical
semantic; the off-page-connector visual is a renderer concern (see
"Net-label semantics" note above). Recommendation: flag this for
M7 renderer work — Datum may want an `OffPageConnector` visual
treatment for global labels that cross sheet boundaries, drawn as a
flag/arrow shape distinct from the in-sheet label rendering.

### Symbol-Library Standards

#### IEC 81714 (design of graphical symbols)

**Full title:** IEC 81714 (multi-part) — *Design of graphical
symbols for use in the technical documentation of products*. Parts
include 81714-1 (Basic rules), 81714-2 (Symbol elements), 81714-3
(Reference numbers).

**Status (2026):** Authoritative international standard for
**how to author** new graphical symbols (line weights, grid spacing,
proportions, terminal placement, pin spacing). Used primarily by
symbol-library authors; less commonly invoked at the EDA-tool level.

**Issuing body:** IEC.

**License / IP:** Paywalled at IEC Webstore.

**Relevance to Datum:** Important for any future Datum native-symbol
authoring tool. Currently low priority because Datum's M4 work
treats symbols as imported, not authored. Becomes relevant when
in-house symbol authoring tools land.

**Datum coverage:** `Reference-only`. Recommendation: cite as the
authoring-style basis when symbol-authoring tools are designed
(post-M7). Add to `STANDARDS_COMPLIANCE_SPEC.md` § 6.1 as a
prerequisite for any "Datum-authored standards-compliant symbol"
claim.

**Strategic recommendation:** Defer. Reference-only with a tracked
prerequisite (post-M7 symbol-authoring tool).

#### ANSI/IEEE Std 991 (pin attributes — cross-ref Domain 2)

**Full title:** IEEE Std 991-1986 — *Logic Circuit Diagrams*.

**Status (2026):** Reaffirmed periodically. Specifies the
**pin-attribute conventions** for logic-circuit diagrams — pin
labels (active-low naming with overbar, e.g., `R\` for active-low
reset), pin grouping, signal direction marks. Aligns with IEEE 91
on logic-symbol form; specifies how the *names* on pins are written.

**Issuing body:** IEEE.

**License / IP:** Paywalled at IEEE Xplore.

**Relevance to Datum:** Cross-references Domain 2 (component
modelling — pin direction codes already covered) and the schematic
editor (active-low pin display). The active-low convention
(overbar in print, backslash or `n_` prefix in text formats) is
relevant to Datum's `Pin.alternates: Vec<AlternateName>` field
(`ENGINE_SPEC.md` § 1.2) and the visual rendering of pin polarity
(bubble per IEC 60617-12 vs polarity-triangle per IEEE 91).

**Datum coverage:** Pin-direction codes are `Implemented`
(`ENGINE_SPEC.md` § 1.1a `PinDirection` enum); active-low naming is
**author-controlled** via the pin name string (no normative
overbar/`n_` policy in Datum). Active-low **visual indication** (the
bubble or triangle drawn on the symbol) is a renderer concern.

**Recommendation:** Add a normative pin-name policy to
`STANDARDS_COMPLIANCE_SPEC.md` § 6.4: "Active-low signals SHOULD
be named with `n_` prefix (e.g., `n_RESET`) or `_N` suffix (e.g.,
`RESET_N`) per common digital-design convention; `~RESET` and
`/RESET` are acceptable alternates for imported designs". This is
text-vocabulary work; the renderer-side bubble/triangle work is
M7+.

### Vocabulary

#### IPC-T-50 (cross-ref IPC research)

**Reference:** `research/ipc-compliance/IPC_COMPLIANCE_RESEARCH.md`
§ IPC-T-50 (lines 726-742). Not re-surveyed here.

**Summary:** IPC-T-50 Rev N (November 2021) is the master IPC
glossary; ~1500 terms covering footprints, pads, vias, copper
features, soldering, fab-process vocabulary. Datum already adopted
it as the controlling vocabulary for footprint / DRC surfaces. Per
`STANDARDS_COMPLIANCE_SPEC.md` § 6.4, Datum extends the same
adoption to user-visible standards-facing terminology generally.

**Datum's vocabulary alignment status:** The CLI / MCP / engine spec
are largely T-50-aligned for footprint terminology (annular ring,
courtyard, padstack, land, fillet, solder mask, silkscreen). The
schematic editor terminology has not been audited against T-50;
this is recommended as a one-pass audit task — the result is a small
number of CLI flag rename / MCP method rename suggestions, not a
research extension.

**Datum coverage:** `Planned` for user-visible terminology per
`STANDARDS_COMPLIANCE_SPEC.md` § 4.3.

#### IEEE 100

**Full title:** IEEE Std 100, *The Authoritative Dictionary of IEEE
Standards Terms*. Last printed edition: 7th edition, 2000.

**Status (2026):** Largely superseded by online resources; IEEE has
not maintained the print edition in 25 years. Many of its
definitions have migrated to individual IEEE standards' "Definitions"
sections. Cited in older standards as the authoritative IEEE
vocabulary source.

**License / IP:** Out of print; secondary editions available.

**Relevance to Datum:** Historical citation only. Datum's vocabulary
baseline is IPC-T-50 (footprint / fab) plus IEC 60050 Electropedia
(electrotechnical generally) — see below. IEEE 100 is not needed.

**Datum coverage:** Out of scope.

#### IEC 60050 (Electropedia)

**Full title:** IEC 60050 — *International Electrotechnical
Vocabulary (IEV)*. Multi-part (over 80 parts), continuously updated.
Free online at `std.iec.ch/iev/` as the **IEC Electropedia** website.

**Status (2026):** **Mainstream international electrotechnical
vocabulary; freely searchable.** Each term has a unique IEV
identifier (e.g., 151-13-04 for "voltage"); definitions in 14
languages.

**Issuing body:** IEC TC 1 (Terminology).

**License / IP:** **Free online access**, browseable; PDF download
of individual parts is paywalled but the web search interface is
free and authoritative.

**Relevance to Datum:** Authoritative international vocabulary
source. Where IPC-T-50 covers PCB-fabrication terminology,
Electropedia covers everything else (circuit theory, components,
electromagnetic phenomena, etc.). For schematic-editor and ERC
terminology that touches general electrical concepts (driver,
sink, drain, gate, source, polarity, etc.), Electropedia is the
controlling reference.

**Datum coverage:** `Reference-only`. Recommendation: cite IEC
60050 alongside IPC-T-50 in `STANDARDS_COMPLIANCE_SPEC.md` § 6.4
as the dual vocabulary baseline (IPC-T-50 for fabrication terms,
IEC 60050 for general electrotechnical terms).

**Strategic recommendation:** Adopt as second baseline. Cost:
near-zero (web reference, no spec PDF needed). High value: free,
authoritative, multi-language.

## Cross-Cutting Patterns

### Symbol-style profile model (IEEE vs IEC vs imported/custom)

**The architectural question:** at what scope does a symbol-style
profile attach? Per-symbol? Per-library? Per-project?

**Survey:**

- **Altium**: profile is implicit per-Symbol (the Symbol object
  carries its drawn primitives; the renderer draws what's there).
  Project-level "preferred style" is a setting that affects only
  newly-created symbols.
- **KiCad**: same — Symbol carries its primitives; user selects
  IEEE-style or IEC-style by choosing which library to draw from
  (KiCad ships separate libraries: `74xx.kicad_sym` for distinctive-
  shape, `74xx-IEC.kicad_sym` for rectangular).
- **OrCAD Capture**: same — library-driven; no project-level style
  enforcement.
- **Horizon EDA**: pool-driven; symbols are pool entries; the
  renderer draws what's there.

**Recommendation for Datum:** **Per-Symbol profile**, with a
project-level "preferred profile" hint that affects new-symbol
creation and triggers a validator warning when a placed Symbol's
profile differs from the project preference. **Not** a per-Project
hard rule (would break international design teams with mixed-style
imports). **Not** per-Library hard rule (would over-constrain
re-use of single-style libraries in mixed-style projects).

**Spec changes required:**

```rust
// Add to ENGINE_SPEC.md § 1.2 Pool Types:
pub enum SymbolStyleProfile {
    Ieee315,                    // ANSI/IEEE 315 + IEEE 91 distinctive logic
    Iec60617,                   // IEC 60617 (12-part, with IEC 60617-12 logic)
    JisC0617,                   // Japanese national IEC variant
    ImportedCustom,             // imported, profile not asserted
    Mixed,                      // contains elements of multiple profiles
}

pub struct Symbol {
    pub uuid: Uuid,
    pub unit: Uuid,
    pub primitives: Vec<Primitive>,
    pub style_profile: SymbolStyleProfile,    // NEW: explicit style assertion
    pub style_provenance: Option<String>,     // NEW: source library name when imported
    // ... existing fields preserved ...
}

// Project-level preferred profile:
// Add to NATIVE_FORMAT_SPEC.md § 6.1 project.json:
{
  ...,
  "schematic_style": {
    "preferred_profile": "ieee_315",   // or "iec_60617" / "mixed"
    "enforce_uniform": false           // when true, flags off-profile symbols as warnings
  }
}
```

**Imported-symbol profile assignment on import:**
- KiCad libraries from `Device.kicad_sym` (the IEEE-default library)
  → `Ieee315`
- KiCad libraries from `Device-IEC.kicad_sym` or other IEC-suffixed
  libraries → `Iec60617`
- KiCad symbols from third-party libraries with no profile hint →
  `ImportedCustom`
- Altium SchDoc / Eagle .lbr — no normative style hint in source
  format; default to `ImportedCustom` and let the user re-classify

### Reference-designator policy enforcement

**The architectural question:** when does Datum enforce the
ASME Y14.44 / IEC 81346 prefix table? At authoring time (refuse a
non-conforming prefix)? At validation time (warn but accept)? Or
both?

**Survey:**

- **Altium**: warn-only (the "Designator Manager" flags
  non-conformant prefixes but does not refuse).
- **KiCad**: no enforcement; designator prefix is whatever the
  library's `R/C/L/U/Q/D` field says. Annotation respects the
  library prefix.
- **OrCAD**: warn-only.

**Recommendation for Datum:** **Warn-only at validation time;
honour the user's authored prefix at authoring time.** This is the
industry-consistent posture. The validator runs as part of ERC or as
a separate `validate_designators` MCP tool, emits a `Warning` for
prefix-table drift, and emits a `Note` for legitimate custom
prefixes (e.g., `JP1` for jumper, `TP1` for test point — both in
the table) where the validator might otherwise mis-classify.

**Where the prefix table lives:**

- **Bundled with Datum as a JSON resource** (recommended): ships
  with the engine, versioned with the engine release; user can
  override at project level.
- **In the pool**: alternative; allows team-specific prefix tables.
  More flexible but harder to migrate.

**Recommended answer:** Bundle the default ASME Y14.44 and IEC
81346-2 tables; accept project-level override as an additional JSON
file under `settings/`. Enable the override via
`STANDARDS_COMPLIANCE_SPEC.md` project metadata
(`designator_profile: { table: "asme_y14_44_2024" | "iec_81346_2_2019"
| "custom" }`).

**Spec edit:**
- `ENGINE_SPEC.md` § 1.2 — no change to `Entity.prefix: String`
  (already correct; free-form by design)
- `STANDARDS_COMPLIANCE_SPEC.md` § 6.2 — add a normative
  designator-profile selection mechanism
- New file: `designator_profile_asme_y14_44.json` and
  `designator_profile_iec_81346_2.json` ship with the engine

### Sheet-size enforcement and template system

**Architectural question:** should Datum enforce sheet-size choices,
validate, or just preserve?

**Survey:**

- **Altium**: defines a list of standard sizes (A, B, C, D, E,
  A4, A3, A2, A1, A0, plus custom); each schematic sheet has a
  size attribute; the title block is a separate template that
  references the sheet size.
- **KiCad**: similar — `paper_size` field per `.kicad_sch` file;
  values include `A`, `B`, `C`, `D`, `E`, `A4`–`A0`, `USLetter`,
  `USLegal`, `User { width, height }`.
- **OrCAD Capture**: per-page size; "Sheet Setup" dialog; standard
  + custom sizes.
- **Eagle**: per-sheet size attribute; standard + custom.

**Recommendation for Datum:** **Validate, do not enforce.** Datum's
authoring spec should accept any sheet size; the renderer warns if
the placed content exceeds the declared sheet bounds; the title-
block template engine selects the appropriate template based on
declared sheet size.

**Template system architecture:**

ISO 7200 prescribes the *data fields* in the title block; **it does
not prescribe the visual layout**. Every tool ships its own template
format. Datum needs a small template engine.

**Survey of template formats:**

- **Altium**: `.SchDot` files; template includes a graphic frame
  + parametric placeholders that resolve from `SheetFrame` fields.
- **KiCad**: `.kicad_wks` (worksheet) files; XML-like format with
  graphic primitives + `${TITLE}`, `${REVISION}`, `${COMPANY}`
  variable substitutions.
- **OrCAD**: `.dot` template files; field-based.
- **Cadence Allegro / Capture**: custom template format with field
  substitution.

**Recommendation for Datum:** Adopt a **KiCad-style minimal template
format**:

```rust
// Add to ENGINE_SPEC.md § 1.2 Pool Types (or new SheetTemplate type):
pub struct SheetTemplate {
    pub uuid: Uuid,
    pub name: String,
    pub sheet_size: SheetSize,
    pub frame_primitives: Vec<Primitive>,    // border, dividing lines
    pub field_placements: Vec<FieldPlacement>,
}

pub enum SheetSize {
    AnsiA, AnsiB, AnsiC, AnsiD, AnsiE, AnsiF,    // ANSI Y14.1
    IsoA0, IsoA1, IsoA2, IsoA3, IsoA4,             // ISO 5457
    UsLetter, UsLegal, UsTabloid,                  // North American common
    Custom { width_nm: i64, height_nm: i64 },
}

pub struct FieldPlacement {
    pub field_key: String,        // "title", "revision", "document_number", "approver", ...
    pub position: Point,
    pub rotation: i32,
    pub font_size: i32,
    pub max_width: Option<i64>,   // for text wrapping
}
```

`Sheet` gains an optional `template: Option<Uuid>` referencing a
`SheetTemplate` pool entry. Resolution at render time substitutes
`SheetFrame` field values into the template's `FieldPlacement`
slots.

**Bundled templates:** Datum ships a small set:
- ANSI A landscape with minimal title block
- ANSI B landscape with full ISO 7200 fields
- ISO A4 portrait with minimal title block
- ISO A3 landscape with full ISO 7200 fields

User can author additional templates as pool entries.

### Title-block template system

**Cross-reference:** The template system above (§ Sheet-size
enforcement) is the title-block template system. The visual layout
is template-defined; the data fields are owned by `SheetFrame` per
ISO 7200.

**ISO 7200 mandatory field coverage (current vs proposed):**

| ISO 7200 field | Current `SheetFrame` | Proposed |
|----------------|---------------------|----------|
| Title | `title: String` | (kept) |
| Identification number | — | `document_number: Option<String>` (NEW) |
| Document type | — | `document_type: Option<DocumentType>` (NEW) |
| Issuing organisation | `company: Option<String>` | (kept; renamed conceptually) |
| Revision indicator | `revision: Option<String>` | (kept) |
| Date of issue | — | `date_of_issue: Option<DateTime<Utc>>` (NEW) |
| Sheet number / total | `page_number: Option<String>` | (kept; supplemented by `sheet_index` / `sheet_count`) |
| Status | — | `status: Option<DocumentStatus>` (NEW) |
| Approver | — | `approver: Option<String>` (NEW) |
| Reviewer | — | `reviewer: Option<String>` (NEW) |
| Technical reference | — | `technical_reference: Option<String>` (NEW) |
| Project number | — | `project_number: Option<String>` (NEW) |
| Customer identification | — | `customer: Option<String>` (NEW) |
| Document classification | — | (cross-domain to Domain 4 — `classification: Option<ClassificationMarking>`) |

**Spec edit:**

```rust
// Modify SheetFrame in ENGINE_SPEC.md § 1.4 and SCHEMATIC_EDITOR_SPEC.md § 2.2:
pub struct SheetFrame {
    pub uuid: Uuid,
    pub title: String,
    pub revision: Option<String>,
    pub company: Option<String>,
    pub page_number: Option<String>,
    // NEW ISO-7200 fields:
    pub document_number: Option<String>,
    pub document_type: Option<DocumentType>,
    pub date_of_issue: Option<DateTime<Utc>>,
    pub sheet_index: Option<u32>,
    pub sheet_count: Option<u32>,
    pub status: Option<DocumentStatus>,
    pub approver: Option<String>,
    pub reviewer: Option<String>,
    pub technical_reference: Option<String>,
    pub project_number: Option<String>,
    pub customer: Option<String>,
    pub classification: Option<String>,    // ITAR/EAR markings, cross-domain
}

pub enum DocumentType {
    Schematic,
    BoardLayout,
    Assembly,
    BomList,
    FabricationDrawing,
    StackupDrawing,
    Other(String),
}

pub enum DocumentStatus {
    Preliminary,
    InReview,
    Approved,
    Released,
    Obsolete,
    Custom(String),
}
```

### Vocabulary alignment (IPC-T-50 baseline)

**Already covered in IPC research.** Cross-reference:
`research/ipc-compliance/IPC_COMPLIANCE_RESEARCH.md` § IPC-T-50.

**Additions for Domain 3:**

- IEC 60050 Electropedia adopted as second vocabulary baseline
  (free, authoritative, international, multi-language).
- Schematic-specific vocabulary audit: pass over CLI / MCP / spec
  text for terms like "wire" (T-50: "conductor"), "label"
  (T-50: acceptable), "junction" (T-50: "node" preferred), "bus"
  (industry-de-facto, T-50: "signal bus"). Recommend keeping
  industry-de-facto where T-50 is awkward; explicitly cite which
  term is normative per spec.

### Bus syntax harmonisation

**Cross-reference:** see § Schematic-Specific Conventions → Bus
syntax above. Recommendation: KiCad-compatible `D[a..b]` as
canonical; per-source-format normalisation on import.

### Cross-EDA symbol exchange (cross-ref Domain 1)

**The state of the art (2026):** there is **no widely-adopted
neutral exchange format for schematic symbols**. EDIF (Electronic
Design Interchange Format, IEC 61690) is the closest historical
candidate but is dead in practice. The de-facto exchange paths are:

- **KiCad-native** (`.kicad_sym`) — text-based, widely-readable;
  cross-referenced in Domain 1
- **Altium SchLib** — binary, vendor-controlled; some open-source
  parsers exist (`pyaltium`)
- **Eagle .lbr** — XML, DTD-defined; well-understood
- **SnapEDA / UltraLibrarian / Component Search Engine** — commercial
  translation services that take a manufacturer datasheet and emit
  per-tool symbol+footprint+3D-model bundles in each tool's native
  format. Effectively the practical standard for cross-tool symbol
  acquisition.

**Datum's posture:** Per Domain 1 Strategy, **import from
KiCad/Eagle is the practical exchange floor**. Datum should:
- Import KiCad and Eagle symbols (already in scope)
- Allow the user to import SnapEDA / UltraLibrarian / CSE bundles
  (which are KiCad-format among others)
- Not attempt to write a proprietary Datum-symbol export format
  (no consumer for it)

**Cross-reference:** Domain 1 deep-dive § Cross-EDA Symbol Exchange
(in `data-exchange-interop`).

## EDA Tool Support Matrix

| Standard / Convention | Altium | OrCAD/Capture | OrCAD X Presto | Cadence Allegro | PADS Logic | KiCad 8 | Eagle 9 / Fusion | Horizon | LibrePCB | DipTrace | EasyEDA Pro | Datum (current spec) |
|----------------------|--------|---------------|----------------|-----------------|------------|---------|------------------|---------|----------|----------|-------------|---------------------|
| **IEEE 315 symbols** | Library default (US installs) | Library | Library | Library | Library | Both libs ship | Library | Library | Library | Library | Library | Profile field `Planned` |
| **IEC 60617 symbols** | Library available | Library | Library | Library | Library | Both libs ship | Library available | Library | Library | Library | Library | Profile field `Planned` |
| **JIS C 0617 symbols** | Library (JP add-on) | (limited) | (limited) | (limited) | (limited) | (limited) | (limited) | (limited) | (limited) | (limited) | (limited) | Profile field `Planned` |
| **ASME Y14.44 prefixes** | Default convention; no enforcement | Default; no enforcement | Default; no enforcement | Default; no enforcement | Default; no enforcement | Default; no enforcement | Default; no enforcement | Default; no enforcement | Default; no enforcement | Default; no enforcement | Default; no enforcement | Validator `Planned` |
| **IEC 81346 prefixes** | Settings option | (limited) | (limited) | (limited) | (limited) | (limited; user libs) | (limited) | (limited) | (limited) | (limited) | (limited) | Validator `Planned` |
| **ISO 7200 title-block fields** | Template-driven; full coverage | Template-driven; full coverage | Template-driven; full coverage | Template-driven; full coverage | Template-driven | Template-driven (`.kicad_wks`); partial coverage | Template-driven | Template-driven | Template-driven | Template-driven; partial | Limited | 4/15 fields covered today; `Planned` extension |
| **Sheet sizes (ANSI/ISO)** | Full | Full | Full | Full | Full | Full | Full | Full | Full | Full | Partial | None today; `Reference-only` |
| **Hierarchical sheets** | Full + multi-instance | Full + hier blocks | Full + hier blocks | Full + hier blocks | Off-page connectors | Full + multi-instance (7.0+) | Limited (Fusion only) | Full | Full (1.0+) | Limited | Full (Pro) | Full; `Implemented` |
| **Bus syntax (canonical)** | `D[0:7]` | `D<7..0>` | `D<7..0>` | `D<7..0>` | `D[7:0]` / `D<7..0>` | `D[7..0]` | `D[7..0]` | `D[0..7]` | `D[7..0]` | `D[0:7]` | `D[0:7]` | `D[a..b]`; KiCad-compatible; `Implemented` |
| **Net labels (local/global/hier)** | Full | Full | Full | Full | Full | Full | Full | Full | Full | Full | Full | Full; `Implemented` |
| **Power symbols (IEEE/IEC)** | Full both | Full both | Full both | Full both | Full both | Full both | Full both | Full both | Full both | Full both | Full both | Symbol-driven; `Implemented` |
| **No-connect markers** | Full | Full | Full | Full | Full | Full | Pin attribute | Full | Full | Full | Full | Full; `Implemented` |
| **Net-tie objects** | Full (NetTie part) | Full (NETSHORT) | Full | Full | Full | Net-tie footprint family | (workaround: 0Ω) | Full | (limited) | (limited) | (limited) | None today (workaround: 0Ω resistor) |
| **Off-page connector visual** | Explicit primitive | Explicit primitive | Explicit primitive | Explicit primitive | Explicit primitive | Hierarchical-label arrow | (workaround: global label) | Hierarchical port | Limited | Limited | Limited | Renderer-deferred (M7+) |

## Pending Exclusions (re-affirmed)

For Domain 3, **none** of the Phase 1 advisory-exclusion list items
apply. The skip list (DO-254 / DO-160 / MIL-PRF / NASA-STD / AS9100 /
IATF / CMMI / JEDEC JEP30 / IHS Markit / Specctra / Hyperlynx /
Windchill / Teamcenter / Aras / Arena / Prop 65 / EU Packaging) is
dominated by regulatory-vertical and PLM concerns belonging to
Domains 4, 6, 7, 8 — not Domain 3.

The two cross-references that **must not** be re-deep-dived because
they are owned elsewhere are:

- **IPC-T-50 vocabulary baseline** — Domain 3 cites it as the
  controlling vocabulary baseline for fabrication terminology;
  detail lives in IPC research.
- **Schematic interop formats (EDIF / KiCad / Eagle / Altium SchDoc
  / OrCAD .dsn)** — Domain 1 covers their import/export; Domain 3
  cites them for symbol-style provenance hints only.

**New Domain-3 exclusions recommended for the consolidated post-
Domain-8 ratification pass:**

- **EDIF (IEC 61690 / Electronic Design Interchange Format)** — dead
  in practice as a schematic exchange medium; KiCad / Eagle / Altium
  /OrCAD all provide better paths. Recommend formal `Out of scope`
  classification.
- **DIN 40700 / 40717 / 40900** — superseded by DIN EN 60617;
  recommend formal `Out of scope` classification (handled via the
  IEC 60617 profile).
- **DIN 6771 / DIN 40719** — superseded by DIN EN ISO 7200 / DIN EN
  IEC 81346; recommend formal `Out of scope` classification.
- **IEEE 100** — out-of-print, superseded by per-standard
  definitions; recommend formal `Out of scope` classification.
- **ISO 3098 (lettering)** — mechanical-drawing focus; Datum
  schematic renderer uses standard fonts; recommend `Reference-only`
  with no implementation.
- **ANSI/ASME Y14.5 (GD&T)** — mechanical, not schematic; recommend
  `Out of scope` for the schematic editor (may be in scope for PCB
  fabrication-drawing output, post-M7).

None of these Domain-3 skip recommendations have hidden cross-cutting
value worth flagging.

## User Pain Points & Wishlist Items

Distilled from KiCad community forums, EEVblog, Altium forum,
r/PrintedCircuitBoard, r/KiCad, and the EDA industry survey blogs
(EDN, AspenCore, EE Times):

1. **"Why does my European reviewer reject my schematic? It works in
   the US."** — Style mismatch. Engineer trained on IEEE 315
   (zigzag resistors, distinctive logic gates) submits to a German
   client who expects IEC 60617 (rectangular resistors, qualifying-
   label logic gates). The fix today is to redraw symbols in the
   target style; no tool offers a one-click style switch. **Datum
   opportunity:** if the symbol carries a `style_profile` and the
   pool has the symbol authored in both profiles, a one-click
   per-symbol style swap is feasible. Authentic differentiator vs
   incumbents.

2. **"Designator drift: my schematic has `CR3` and the BOM has
   `D3`."** — Reference-designator inconsistency between schematic-
   imported library (older convention `CR` for diode) and BOM
   downstream tooling (modern `D` per ASME Y14.44). Engineers
   manually re-annotate. **Datum opportunity:** the
   designator-profile validator catches this at import.

3. **"My title block looks different on every sheet."** — Mixed-
   template schematics where different sheets reference different
   templates by accident. **Datum opportunity:** project-level
   template uniformity check (warns when sheets reference different
   templates than the project default).

4. **"Why doesn't ERC catch my floating power symbol?"** — Power
   symbol placed on a sheet but never connected to a wire / pin.
   Some tools catch this; others don't. **Datum coverage:** ERC
   rule `power_without_source` catches power-net-without-driver but
   not the unconnected-power-symbol case. Add a connectivity-
   diagnostic for unconnected power symbols.

5. **"Bus syntax breaks when I import from another tool."** —
   Recurring forum complaint. Altium → KiCad imports lose `D[0:7]`
   syntax; Cadence → KiCad loses `D<7..0>` syntax. **Datum
   opportunity:** explicit normalisation matrix and round-trip
   preservation per `SCHEMATIC_CONNECTIVITY_SPEC.md` extension.

6. **"I want my reviewer-name and reviewed-date in the title block
   but my tool only has 'rev' and 'date'."** — ISO 7200 has been
   asking for these fields for 22 years; Datum can land them
   correctly on first try.

7. **"How do I author an IEC 60617 schematic when my company doesn't
   subscribe to the IEC database?"** — Real cost issue for small
   teams; the answer is to use KiCad's IEC libraries (CC-BY-SA-4.0)
   which is workable for KiCad output but does not help for Datum
   redistribution. **Datum opportunity:** ship a permissive minimal
   IEC-style symbol library (resistors / capacitors / inductors /
   diodes / common transistors / common logic gates) authored
   in-house under MIT or Apache-2.0.

8. **"My net labels conflict on import."** — Imported designs with
   the same net name on different scopes (one local, one global)
   trigger label conflicts. Datum's `SCHEMATIC_CONNECTIVITY_SPEC.md`
   § 4.2 already handles this as a connectivity diagnostic.
   Confirmed correct; no change needed.

## Datum EDA Implementation Strategy

### Hard Requirements (must support)

These land as part of the Domain 3 spec edits in the next batch.

#### HR-1. `SymbolStyleProfile` enum on `Symbol` records

**Standard:** IEEE 315 / IEC 60617 / JIS C 0617 (cross-coverage).

**Canonical IR changes:** Add `SymbolStyleProfile` enum to
`ENGINE_SPEC.md` § 1.1a (Shared Enums); add `style_profile:
SymbolStyleProfile` and `style_provenance: Option<String>` fields
to `Symbol` (§ 1.2 Pool Types).

**Schematic-spec changes:** `SCHEMATIC_EDITOR_SPEC.md` notes the
profile at placement time (placement does not coerce style; user
authoring authority preserved).

**Library / pool model changes:** Pool index gets a `symbol_style`
column on the `symbols` table; query interface allows filtering by
profile.

**Transaction model changes:** `SetSymbolStyleProfile` operation
(authored, undoable) — for re-classifying imported symbols.

**MCP API additions:**
- `query_symbol_style(symbol_uuid)` — returns profile
- `set_symbol_style(symbol_uuid, profile, provenance)` — sets profile
- `validate_project_style_uniformity(project_uuid)` — emits warnings
  for off-profile placements

**Minimum viable:** enum + field + query (no validator). Authored
classification only.

**Full implementation:** validator + project-level enforcement +
import-time profile inference + per-symbol style-swap operation
(when both-profile authoring lands, post-M7).

**Partner / library dependencies:** None for the data-model surface.
The symbol-library content (the actual IEEE-style and IEC-style
graphics) is a separate workstream — see HR-7.

**Effort estimate:** 2-3 days for the data-model + MCP changes;
1-2 days per profile for the validator polish.

#### HR-2. ASME Y14.44 / IEC 81346 default-prefix tables

**Standard:** ASME Y14.44-2024 / IEC 81346-2:2019.

**Canonical IR changes:** None to `Entity.prefix` (already
correctly free-form). Add `DesignatorProfile` enum to
`ENGINE_SPEC.md` § 1.1a (e.g. `AsmeY14_44_2024 | Iec81346_2_2019 |
IeeeStd200_1975 | Custom`).

**Schematic-spec changes:** None.

**Library / pool model changes:** Bundled JSON tables ship with the
engine: `designator_profile_asme_y14_44.json` and
`designator_profile_iec_81346_2.json`.

**Transaction model changes:** `SetProjectDesignatorProfile` operation
(authored project-level setting).

**MCP API additions:**
- `validate_designators(project_uuid, profile?)` — runs the prefix
  validator; returns drift warnings
- `get_designator_profile(project_uuid)` — returns active profile
- `set_designator_profile(project_uuid, profile)` — sets profile

**Minimum viable:** ASME Y14.44 table only; warn-only validator.

**Full implementation:** both tables; both profiles; per-Entity
override (for legacy library reuse).

**Partner dependencies:** None. The prefix table is freely
reproducible.

**Effort estimate:** 1-2 days.

#### HR-3. ISO 7200 title-block field extension to `SheetFrame`

**Standard:** ISO 7200:2004.

**Canonical IR changes:** Extend `SheetFrame` in
`ENGINE_SPEC.md` § 1.4 with the eleven new optional fields listed
in § "Title-block template system" above. Add `DocumentType` and
`DocumentStatus` enums to § 1.1a.

**Schematic-spec changes:** `SCHEMATIC_EDITOR_SPEC.md` § 2.2
(SheetFrame definition) updated to match. Add `SetSheetFrameField`
operation for editing individual fields without re-authoring the
whole frame.

**Library / pool model changes:** None.

**Transaction model changes:** `SetSheetFrameField(sheet_uuid,
field_key, value)` operation. Granular for AI-assisted field-fill
workflows (the AI can fill the document number from a project
manifest without touching other fields).

**MCP API additions:**
- `set_sheet_frame_field(sheet_uuid, key, value)` — write
- `get_sheet_frame(sheet_uuid)` — read all fields
- `validate_iso7200_compliance(project_uuid)` — checks that all
  mandatory fields are populated on every sheet

**Minimum viable:** Add the field surface. No validator.

**Full implementation:** Add the validator. Add bundled templates.

**Partner dependencies:** None. ISO 7200 lists the fields; the
spec is paywalled but the field list is freely citable.

**Effort estimate:** 1 day for the data-model extension; 1-2 days
for MCP surface.

#### HR-4. KiCad-compatible bus-syntax canonical form + import normalisation matrix

**Standard:** Industry-de-facto (no formal standard). Datum picks
KiCad as canonical.

**Canonical IR changes:** None to `Bus.name: String` (free-form
preserves authored intent).

**Schematic-spec changes:** Extend
`SCHEMATIC_CONNECTIVITY_SPEC.md` § 4.5 with the canonical-form
declaration and the per-source-format normalisation matrix.

**Library / pool model changes:** None.

**Transaction model changes:** None.

**MCP API additions:**
- `normalize_bus_syntax(bus_name, source_format)` — converts
  per-tool syntax to canonical form

**Minimum viable:** Documentation only. The current spec already
supports the canonical form.

**Full implementation:** Importer code paths emit canonical form;
exporter rewrites to source-target form on round-trip if requested.

**Partner dependencies:** None.

**Effort estimate:** 0.5 days documentation + 1-2 days per importer
to implement normalisation.

#### HR-5. `SheetSize` enum with ANSI + ISO + custom variants

**Standard:** ANSI Y14.1-2020, ANSI Y14.1M-2020, ISO 5457:1999.

**Canonical IR changes:** Add `SheetSize` enum to `ENGINE_SPEC.md`
§ 1.1a. Add `Sheet.size: SheetSize` field (or
`Sheet.template: Option<Uuid>` referencing a `SheetTemplate` that
declares its own size — see HR-6).

**Schematic-spec changes:** `SCHEMATIC_EDITOR_SPEC.md` § 2.2
updated; add `SetSheetSize` operation.

**Library / pool model changes:** None for HR-5; templates live in
HR-6.

**Transaction model changes:** `SetSheetSize` operation.

**MCP API additions:**
- `set_sheet_size(sheet_uuid, size)`
- `get_sheet_size(sheet_uuid)`

**Minimum viable:** Enum only.

**Full implementation:** Enum + sheet bounds validator (warns if
authored content exceeds declared sheet bounds; no hard enforcement).

**Partner dependencies:** None.

**Effort estimate:** 1 day.

### Should Support (post-M7)

#### SS-1. `SheetTemplate` pool entry for ISO 7200-style title blocks

**Standard:** ISO 7200 (data fields) + visual layout per template.

**Canonical IR changes:** Add `SheetTemplate` and `FieldPlacement`
types to `ENGINE_SPEC.md` § 1.2 Pool Types. Add `template:
Option<Uuid>` field to `Sheet`.

**Schematic-spec changes:** `SCHEMATIC_EDITOR_SPEC.md` § 2.2
updated; add `SetSheetTemplate` operation.

**Library / pool model changes:** New pool entry kind: `templates/`
directory; SQL index gets a `templates` table; FTS index supports
template name search.

**Transaction model changes:** `SetSheetTemplate(sheet_uuid,
template_uuid)`. Template-edit operations are pool-edit operations
(`CreateTemplate`, `EditTemplateField`, `DeleteTemplate`).

**MCP API additions:**
- `list_templates(pool_uuid)` — enumerate
- `get_template(template_uuid)` — read
- `create_template(...)` — author
- `set_sheet_template(sheet_uuid, template_uuid)` — bind
- `render_sheet_with_template(sheet_uuid, template_uuid)` — render
  preview (M7+ renderer)

**Minimum viable:** Type definitions + bundled defaults (ANSI A
landscape, ISO A4 portrait); no template editor.

**Full implementation:** GUI template editor (M7+).

**Partner dependencies:** None.

**Effort estimate:** 3-5 days for data-model + bundled defaults; 5-10
days for GUI editor (post-M7).

#### SS-2. IEC 60050 Electropedia as second vocabulary baseline

**Standard:** IEC 60050.

**Canonical IR changes:** None.

**Spec changes:** `STANDARDS_COMPLIANCE_SPEC.md` § 6.4 updated to
cite both IPC-T-50 (fabrication) and IEC 60050 (general
electrotechnical) as dual baselines.

**MCP API additions:** None directly; vocabulary alignment is a
text-pass exercise.

**Effort estimate:** 0.5 days spec edit + 1-2 days vocabulary audit
of CLI / MCP / spec text.

#### SS-3. `LogicSymbolForm` distinctive-vs-rectangular per-Symbol enum

**Standard:** IEEE Std 91 / IEC 60617-12.

**Canonical IR changes:** Add `LogicSymbolForm` enum
(`DistinctiveShape | Rectangular | Mixed`) to `ENGINE_SPEC.md` §
1.1a. Optional field on `Symbol` for logic gates only.

**Effort estimate:** 1 day.

#### SS-4. Off-page-connector visual treatment for `LabelKind::Global`

**Standard:** Industry-de-facto.

**Canonical IR changes:** None (electrical semantic already correct).

**Renderer changes (M7+):** Renderer detects when a `Global` label
is the sole label on a wire that touches a sheet boundary; renders
it with a flag/arrow off-page connector glyph instead of a plain
label.

**Effort estimate:** 2-3 days renderer work (post-M7).

### On-Demand Only

#### OD-1. JIS C 0617 profile

**Standard:** JIS C 0617 (Japanese national IEC variant).

**Implement when:** A Japanese-market user requests it. The data-model
surface is already in place via the `SymbolStyleProfile` enum
(HR-1); only the symbol-library content differs.

**Effort estimate:** 5-10 days symbol authoring per requested family.

#### OD-2. Net-tie object as first-class

**Standard:** Industry-de-facto.

**Implement when:** Power-electronics or analog-precision users
request explicit net-tie support. Until then, the 0Ω-resistor
workaround is acceptable.

**Effort estimate:** 3-5 days.

#### OD-3. Sheet-bounds enforcement (auto-warn when content exceeds
sheet)

**Standard:** ANSI Y14.1 / ISO 5457.

**Implement when:** Sheet-frame rendering work begins (M7+).

**Effort estimate:** 1-2 days.

### Out of Scope (recommend formal exclusion)

The following Domain-3 standards are recommended for formal
`Out of scope` classification in the consolidated post-Domain-8
ratification pass. Each is documented in the Pending Exclusions
section above with rationale.

- EDIF (IEC 61690)
- DIN 40700 / 40717 / 40900 / 6771 / 40719 (all superseded)
- IEEE 100 (out-of-print)
- ISO 3098 (mechanical lettering)
- ANSI/ASME Y14.5 GD&T (mechanical, not schematic)

### Datum Differentiators

Where Datum's pool + transaction + AI surfaces can do better than
incumbents:

1. **Live profile-switching.** No incumbent offers one-click
   IEEE↔IEC profile swap on a placed Symbol. Datum's pool can
   carry both profile variants of common symbols; the swap is a
   transaction, undoable, deterministic. Authentic differentiator
   for international design teams.

2. **AI-explained style violations.** When a project-level uniformity
   validator flags an off-profile symbol, the MCP-surfaced AI agent
   can explain in natural language: "Symbol R5 is drawn as a
   rectangular resistor (IEC 60617 style); the project profile is
   IEEE 315 (zigzag). Fix: replace with the IEEE-profile resistor
   from the bundled library." This is more useful than a one-line
   error message.

3. **MCP-queryable title-block fields.** Title-block content is
   first-class data, not opaque template prose. AI agents can
   inspect, validate, and fill the fields programmatically. Useful
   for ISO 9001 / 21 CFR Part 11 audit-trail integration (Domain 8
   cross-cut).

4. **Deterministic per-symbol style provenance.** Every Symbol
   carries `style_provenance: Option<String>` recording where the
   style classification came from (KiCad library name, user
   override, AI inference). Audit-traceable.

5. **Designator-profile drift detection.** No incumbent tool runs a
   prefix-table audit at validation time. Datum's
   `validate_designators` MCP tool produces a structured report:
   per-Entity prefix, expected category, drift status, audit-loggable
   override rationale. Useful for ISO 9001 / AS9100 design-review
   sign-off (Domain 8 cross-cut).

6. **Bundled permissively-licensed in-house symbol library.** No
   competing tool ships a permissively-licensed (MIT/Apache-2.0)
   IEC-style symbol library that downstream consumers can
   redistribute without copyleft inheritance. KiCad's libraries are
   CC-BY-SA-4.0; Eagle's are vendor-controlled; Altium's are paid.
   Datum can ship a small permissive baseline and let the user
   layer KiCad libraries on top for personal use.

### Recommended Spec Edits

Concrete file:line edits for the user to review. Pattern follows
Standards Audit Batch 1.

| # | Source | Target file | Substance |
|---|--------|-------------|-----------|
| **Pass 0 — STANDARDS_COMPLIANCE_SPEC.md disposition refresh** ||||
| D3-0a | this report | `specs/STANDARDS_COMPLIANCE_SPEC.md` § 4.3 | Domain 3 dispositions refreshed: IEEE 315 / IEC 60617 / ASME Y14.44 / IEC 81346 / ISO 7200 dispositions promoted to "Planned (contract surface defined, implementation pending)" with citations to new ENGINE_SPEC types; IEC 60050 added as second vocabulary baseline; explicit `Out of scope` classifications added for EDIF, DIN 40700/6771/40719, IEEE 100, ISO 3098, ANSI/ASME Y14.5 |
| D3-0b | this report | `specs/STANDARDS_COMPLIANCE_SPEC.md` § 6.1 | Symbol-style policy updated to enumerate IEEE 315 / IEC 60617 / JIS C 0617 / ImportedCustom / Mixed profile values explicitly |
| D3-0c | this report | `specs/STANDARDS_COMPLIANCE_SPEC.md` § 6.2 | Reference-designator policy updated with the bundled-table mechanism (ASME Y14.44 + IEC 81346-2 ship as JSON resources) and the warn-only validator semantic |
| D3-0d | this report | `specs/STANDARDS_COMPLIANCE_SPEC.md` § 6.3 | Title-block contract updated to enumerate the eleven additional ISO 7200 fields and the `SheetTemplate` engine architecture |
| D3-0e | this report | `specs/STANDARDS_COMPLIANCE_SPEC.md` § 6.4 | Vocabulary baseline expanded to dual baseline (IPC-T-50 + IEC 60050 Electropedia) |
| **Pass 1 — `specs/ENGINE_SPEC.md` schema bedrock** ||||
| D3-1 | this report | `specs/ENGINE_SPEC.md` § 1.1a | New enums: `SymbolStyleProfile`, `LogicSymbolForm`, `DesignatorProfile`, `DocumentType`, `DocumentStatus`, `SheetSize` |
| D3-2 | this report | `specs/ENGINE_SPEC.md` § 1.2 | Extend `Symbol` with `style_profile: SymbolStyleProfile`, `style_provenance: Option<String>`, `logic_form: Option<LogicSymbolForm>` |
| D3-3 | this report | `specs/ENGINE_SPEC.md` § 1.2 | New pool type: `SheetTemplate { uuid, name, sheet_size, frame_primitives, field_placements }` and `FieldPlacement { field_key, position, rotation, font_size, max_width }` |
| D3-4 | this report | `specs/ENGINE_SPEC.md` § 1.4 | Extend `SheetFrame` with eleven new optional fields (`document_number`, `document_type`, `date_of_issue`, `sheet_index`, `sheet_count`, `status`, `approver`, `reviewer`, `technical_reference`, `project_number`, `customer`, `classification`) |
| D3-5 | this report | `specs/ENGINE_SPEC.md` § 1.4 | Add `Sheet.size: SheetSize` and `Sheet.template: Option<Uuid>` (referencing `SheetTemplate` pool entry) |
| D3-6 | this report | `specs/ENGINE_SPEC.md` § 3 (Operations) | New operations: `SetSymbolStyleProfile`, `SetSheetFrameField`, `SetSheetSize`, `SetSheetTemplate`, `SetProjectDesignatorProfile` — each with `inverse()` for undo |
| **Pass 2 — pool & native persistence** ||||
| D3-7 | this report | `docs/POOL_ARCHITECTURE.md` § 2 | New pool kind `templates/` directory; new SQL index table `templates`; `symbols` table gains `style_profile` column |
| D3-8 | this report | `specs/NATIVE_FORMAT_SPEC.md` § 4 | `pool/templates/` added to project layout; `settings/designator_profile.json` added (optional) |
| D3-9 | this report | `specs/NATIVE_FORMAT_SPEC.md` § 6.1 | `project.json` gains `schematic_style: { preferred_profile, enforce_uniform }` and `designator_profile: { table }` |
| D3-10 | this report | `specs/NATIVE_FORMAT_SPEC.md` new § 6.x | "Sheet Template Files" schema documenting the pool/templates/<uuid>.json layout |
| **Pass 3 — `specs/MCP_API_SPEC.md` (stubs and headings only)** ||||
| D3-11 | this report | `specs/MCP_API_SPEC.md` new "Schematic Standards Tools" | Section header + per-tool stubs for `query_symbol_style`, `set_symbol_style`, `validate_project_style_uniformity`, `validate_designators`, `get_designator_profile`, `set_designator_profile`, `set_sheet_frame_field`, `get_sheet_frame`, `validate_iso7200_compliance`, `set_sheet_size`, `set_sheet_template`, `list_templates`, `create_template`, `normalize_bus_syntax` |
| **Pass 4 — `specs/SCHEMATIC_EDITOR_SPEC.md`** ||||
| D3-12 | this report | `specs/SCHEMATIC_EDITOR_SPEC.md` § 2.2 | `SheetFrame` updated to match ENGINE_SPEC.md changes (D3-4); `Sheet` updated for `size` / `template` fields (D3-5) |
| D3-13 | this report | `specs/SCHEMATIC_EDITOR_SPEC.md` § 2.3 | `PlacedSymbol` documentation note: profile is inherited from `Symbol`, not overridden at placement |
| D3-14 | this report | `specs/SCHEMATIC_EDITOR_SPEC.md` § 4.1 | Operation list extended with the five new operations (D3-6) |
| D3-15 | this report | `specs/SCHEMATIC_EDITOR_SPEC.md` § 7 | New ERC-relevant editor behavior: unconnected-power-symbol diagnostic |
| **Pass 5 — `specs/SCHEMATIC_CONNECTIVITY_SPEC.md`** ||||
| D3-16 | this report | `specs/SCHEMATIC_CONNECTIVITY_SPEC.md` § 4.5 | Bus syntax extended with canonical-form declaration (`D[a..b]`) and per-source-format normalisation matrix (Altium / Cadence / PADS / Eagle / KiCad) |
| **Pass 6 — `specs/IMPORT_SPEC.md`** ||||
| D3-17 | this report | `specs/IMPORT_SPEC.md` § 3 (KiCad) | Import-time `SymbolStyleProfile` inference from KiCad library names (Device → IEEE 315, Device-IEC → IEC 60617, third-party → ImportedCustom) |
| D3-18 | this report | `specs/IMPORT_SPEC.md` § 4 (Eagle) | Import-time profile inference defaults to ImportedCustom (Eagle .lbr does not carry style hints) |
| **Pass 7 — architecture & guidance docs** ||||
| D3-19 | this report | `docs/STANDARDS_AUDIT_BATCH_2_GUIDANCE.md` (NEW) | Batch-2 bridging doc following the Batch-1 pattern (must-land vs deferred, apply order, ModelProvenance-style overlap resolutions if any, Pass 0 disposition refresh, Cross-Spec Update Rule compliance) |
| D3-20 | this report | `docs/LIBRARY_ARCHITECTURE.md` after § "Canonical Datum Library Model" | New "Symbol-style profiles and bundled symbol-library policy" subsection; addresses CC-BY-SA-4.0 KiCad library license issue and the in-house permissive-baseline plan |
| D3-21 | this report | `docs/POOL_ARCHITECTURE.md` § 2 | Update for `templates/` pool kind; cross-reference D3-7 |
| D3-22 | this report | `docs/INTEROP_SCOPE.md` | Add "Schematic & Drawing Conventions (research-staged)" section: bus-syntax normalisation matrix; ISO 7200 field coverage; symbol-style profile import inference |

**Total recommended spec edits:** **22** (5 disposition refreshes,
6 schema bedrock, 4 pool/native persistence, 1 MCP, 4 schematic
editor, 1 connectivity, 2 import, 4 architecture docs).

This count is intentionally larger than Batch 1 (16 edits) because
Domain 3 spans more spec files: `ENGINE_SPEC` schema bedrock,
`SCHEMATIC_EDITOR_SPEC` and `SCHEMATIC_CONNECTIVITY_SPEC` updates
(both untouched by Batch 1), plus the title-block template engine
(a new pool entity kind). The user can split into multiple PRs if
the batch is judged too large for a single review pass; suggested
split is "Pass 0 + Pass 1 + Pass 2 + Pass 4 + Pass 5" as Batch
2.0 (the schema and editor work) and "Pass 3 + Pass 6 + Pass 7" as
Batch 2.1 (the API and guidance docs).

## Cross-Domain Insights to Thread Forward

### To Domain 4 (industry-vertical compliance)

- **Regulated-industry style mandates:** US military programs
  typically mandate IEEE 315 / MIL-STD-15 (the IEEE 315 lineage);
  European industrial programs typically mandate IEC 60617;
  Japanese industrial programs typically mandate JIS C 0617. The
  `SymbolStyleProfile` enum needs to carry these as first-class
  values; project metadata in
  `STANDARDS_COMPLIANCE_SPEC.md` needs to expose
  `mandated_symbol_profile` for regulated workflows.

- **MIL-STD-15 / MIL-STD-806B reference:** For US military programs,
  MIL-STD-15A (1958, withdrawn 1986) was the original symbol
  standard; MIL-STD-806B (1966, also withdrawn) added logic
  symbology. Both are subsumed into IEEE 315 + IEEE 91 by reference;
  newer MIL-PRF documents simply cite IEEE 315/91. **Datum's
  IEEE 315 profile is a sufficient implementation for MIL-STD-15
  compliance** — flag this for Domain 4.

- **ITAR / EAR title-block markings:** ISO 7200's optional
  `classification` field (`ClassificationMarking` in the proposed
  `SheetFrame` extension) is the first-class home for ITAR / EAR
  / "Confidential" / "ECCN xxxxx" markings. Domain 4 should
  consume this.

- **ASME Y14.44 designator profile is the US-mandated standard for
  DoD electronic equipment.** Domain 4 should mandate the ASME
  designator profile when the project's industry vertical is set to
  "US-defence".

### To Domain 7 (PLM & lifecycle integration)

- **Title-block fields ARE PLM document-control fields.** Eight of
  the eleven proposed `SheetFrame` extensions
  (`document_number`, `document_type`, `revision`, `date_of_issue`,
  `status`, `approver`, `reviewer`, `project_number`,
  `customer`) map 1:1 onto Windchill / Teamcenter / Aras /
  Arena document-control attributes. The `SheetTemplate` engine is
  the natural surface for PLM-driven field substitution: a PLM
  integration writes `released_by`, `released_at_revision`,
  `eco_number` into the SheetFrame; the template renders them.
  Domain 7 should plan its PLM data model as a superset of the ISO
  7200 fields.

- **Symbol-style profile is also a PLM library-control concern.**
  PLM-managed symbol libraries (Cadence CIS, PartQuest) carry
  per-symbol metadata that includes style classification; Datum's
  pool can mirror this with the `style_profile` field. Domain 7
  should specify how PLM-source style classifications feed the
  pool.

- **Designator profile is a project-level PLM attribute.** Per-
  project ASME Y14.44 vs IEC 81346-2 selection is part of project
  setup metadata; PLM systems often carry this as a project
  attribute. Domain 7 should plan for it.

### To Domain 8 (process & quality)

- **ISO 9001 / 21 CFR Part 11 / AS9100 / ISO 13485 audit-trail all
  expect designer / reviewer / approver title-block sign-offs.**
  The proposed `SheetFrame.approver` / `SheetFrame.reviewer` /
  `SheetFrame.status` fields are exactly the fields these QMS
  regimes expect. The `DocumentStatus` enum (`Preliminary | InReview
  | Approved | Released | Obsolete`) is the audit-trail-visible
  release-state. Domain 8 should specify the transaction-log entries
  that fire when these fields change (who, when, optional signature).

- **Designator-drift validator results are auditable.** Per-design
  validation reports including designator-drift findings should be
  loggable as part of the design-review record. Domain 8's
  audit-trail-export specification should include validator-result
  serialisation.

- **Symbol-style profile changes should be transaction-logged.** A
  mid-project profile switch (rare but contractually significant) is
  a recordable event; the transaction model should make this
  visible to audit-log queries. Domain 8 should specify.

- **Vocabulary alignment (IPC-T-50 + IEC 60050) is part of process-
  quality controlled-language compliance.** ISO 9001 controlled-
  source requirements and aerospace-grade documentation expect
  consistent terminology. Datum's vocabulary-alignment work is a
  cross-cut to Domain 8's controlled-language story.

## Sources

### Primary specifications

- [IEEE Std 315-1975 / 315A-1986](https://standards.ieee.org/ieee/315/1015/) — *Graphic Symbols for Electrical and Electronics Diagrams*. IEEE Xplore (~USD 318); paywalled.
- [IEEE Std 91-1984 / 91a-1991](https://standards.ieee.org/ieee/91/1318/) — *Graphic Symbols for Logic Functions*. IEEE Xplore; paywalled.
- [IEEE Std 991-1986](https://standards.ieee.org/ieee/991/2003/) — *Logic Circuit Diagrams*. IEEE Xplore; paywalled.
- [IEEE Std 200-1975 (withdrawn)](https://standards.ieee.org/standard/200-1975.html) — *Reference Designations for Electrical and Electronic Parts and Equipment*. Withdrawn 1996; archive only.
- [ASME Y14.44-2024](https://www.asme.org/codes-standards/find-codes-standards/y14-44-reference-designations-electrical-electronics-parts-equipment) — *Reference Designations for Electrical and Electronic Parts and Equipment*. ASME store (~USD 96); paywalled.
- [IEC 60617 Database](https://std.iec.ch/iec60617/) — *Graphical symbols for diagrams*. Online subscription database; CHF ~1500/year.
- [IEC 81346-1:2009](https://webstore.iec.ch/publication/4754) — *Industrial systems… Structuring principles and reference designations — Part 1: Basic rules*. IEC Webstore; paywalled.
- [IEC 81346-2:2019](https://webstore.iec.ch/publication/27326) — *…Part 2: Classification of objects and codes for classes*. IEC Webstore; paywalled.
- [IEC 81714 (multi-part)](https://webstore.iec.ch/publication/29054) — *Design of graphical symbols for use in technical documentation of products*. IEC Webstore; paywalled.
- [IEC 60050 Electropedia](https://std.iec.ch/iev/) — *International Electrotechnical Vocabulary*. **Free** online search; PDF download paywalled.
- [ISO 7200:2004](https://www.iso.org/standard/30669.html) — *Technical product documentation — Data fields in title blocks and document headers*. ISO Webstore (~CHF 110); paywalled.
- [ISO 5457:1999](https://www.iso.org/standard/12364.html) — *Technical product documentation — Sizes and layout of drawing sheets*. ISO Webstore; paywalled.
- [ISO 128 (multi-part)](https://www.iso.org/standard/65296.html) — *Technical product documentation — General principles of presentation*. ISO Webstore; paywalled.
- [ISO 3098 (multi-part)](https://www.iso.org/standard/35266.html) — *Technical product documentation — Lettering*. ISO Webstore; paywalled.
- [ANSI/ASME Y14.1-2020](https://www.asme.org/codes-standards/find-codes-standards/y14-1-decimal-inch-drawing-sheet-size-format) — *Drawing Sheet Size and Format (decimal inch)*. ASME store (~USD 90); paywalled.
- [ANSI/ASME Y14.1M-2020](https://www.asme.org/codes-standards/find-codes-standards/y14-1m-metric-drawing-sheet-size-format) — *…(metric)*. ASME store; paywalled.
- [ANSI/ASME Y14.5-2018](https://www.asme.org/codes-standards/find-codes-standards/y14-5-dimensioning-and-tolerancing) — *Dimensioning and Tolerancing*. ASME store; paywalled. (Note-only for schematic; relevant for fab drawings.)
- [JIS C 0617](https://webdesk.jsa.or.jp/) — Japanese national IEC variant. JSA Webstore; paywalled.
- [DIN EN 60617](https://www.beuth.de/) — German national adoption of IEC 60617. Beuth Verlag; paywalled.

### Reference implementations / open-source libraries

- [KiCad symbol libraries](https://gitlab.com/kicad/libraries/kicad-symbols) — KiCad-bundled symbol library (CC-BY-SA-4.0). **Inspiration only — copyleft prevents Datum redistribution under permissive license.**
- [gEDA gschem symbol library](http://www.geda-project.org/) — gEDA symbol set (GPL-2). Reference only; same copyleft caveat.
- [pcb-rnd / librnd](http://www.repo.hu/projects/pcb-rnd/) — pcb-rnd schematic foundation; minimal symbol set.
- [LibrePCB symbol catalog](https://librepcb.org/library/) — LGPL-3 library; copyleft caveat.
- [Horizon EDA pool](https://github.com/horizon-eda/horizon-pool) — Horizon-bundled symbol library (GPL-3). Inspiration only.

### EDA tool documentation

- [Altium symbol-library docs](https://www.altium.com/documentation/altium-designer/sch-pnl-libraries) — Altium SchLib + library workflow.
- [Altium hierarchical schematic docs](https://www.altium.com/documentation/altium-designer/sch-obj-sheetsymbol) — sheet symbols and ports.
- [Altium title-block templates](https://www.altium.com/documentation/altium-designer/sheet-symbol-template) — `.SchDot` template format.
- [KiCad EeSchema documentation — symbols](https://docs.kicad.org/master/en/eeschema/eeschema.html#symbols) — KiCad symbol model.
- [KiCad EeSchema documentation — hierarchical sheets](https://docs.kicad.org/master/en/eeschema/eeschema.html#hierarchical_sheets) — KiCad hierarchy.
- [KiCad EeSchema worksheet (`.kicad_wks`)](https://docs.kicad.org/master/en/eeschema/eeschema.html#page_layout_editor) — KiCad title-block worksheet format.
- [Cadence Capture documentation — hierarchical blocks](https://www.cadence.com/en_US/home/training/all-courses/86089.html) — OrCAD/Capture hier-block training.
- [OrCAD X Presto sheet management](https://www.orcad.com/products/orcad-capture/overview) — OrCAD modern Capture.
- [Mentor PADS Logic documentation](https://eda.sw.siemens.com/en-US/pcb/pads/) — PADS Logic schematic.
- [Eagle library documentation](https://help.autodesk.com/view/EAGLE/ENU/) — Eagle .lbr format reference.
- [Horizon EDA library documentation](https://docs.horizon-eda.org/en/latest/) — Horizon library architecture and pool docs.
- [LibrePCB library documentation](https://docs.librepcb.org/) — LibrePCB symbol/library model.

### Forum / industry discussion

- [KiCad forum: IEEE vs IEC symbol style](https://forum.kicad.info/) — community discussion on symbol-style preference (search "IEC symbols" / "IEEE symbols").
- [EEVblog forum: IEC 60617 access](https://www.eevblog.com/forum/) — community discussion on IEC database subscription cost (search "IEC 60617 subscription").
- [r/PrintedCircuitBoard: symbol style](https://www.reddit.com/r/PrintedCircuitBoard/) — recurring threads on US/EU symbol style mismatch.
- [r/KiCad: bus syntax](https://www.reddit.com/r/KiCad/) — bus-syntax compatibility complaints.
- [r/AskElectronics: ASME Y14.44 designators](https://www.reddit.com/r/AskElectronics/) — community questions on `D` vs `CR` for diodes.
- [Altium forum: title-block templates](https://forum.live.altium.com/) — `.SchDot` workflow discussion.
- [IPC EDGE](https://www.ipcedge.org/) — IPC-T-50 vocabulary training material.
- [IEEE-SA standard pages](https://standards.ieee.org/) — IEEE 315 / 91 / 200 / 991 standards pages.
- [ISO Online Browsing Platform](https://www.iso.org/obp/ui/) — ISO 7200 / 5457 / 128 abstracts (free; full text paywalled).

### Cross-references (Datum-internal)

- `research/standards-audit/STANDARDS_AUDIT.md` § 3 — Phase 1 inventory of Domain 3.
- `research/ipc-compliance/IPC_COMPLIANCE_RESEARCH.md` § IPC-T-50 — vocabulary baseline (cross-ref, not re-surveyed).
- `research/data-exchange-interop/DATA_EXCHANGE_INTEROP_RESEARCH.md` — schematic interop formats (KiCad, Eagle, Altium, OrCAD; cross-ref, not re-surveyed).
- `research/component-modeling/COMPONENT_MODELING_RESEARCH.md` § Pin attributes — pin-direction codes and active-low naming (cross-ref).
- `docs/CANONICAL_IR.md` — canonical IR (transaction model for new operations).
- `docs/POOL_ARCHITECTURE.md` — pool architecture (templates/ pool kind extension).
- `docs/LIBRARY_ARCHITECTURE.md` — library architecture (symbol-style profile + bundled-library policy).
- `docs/STANDARDS_AUDIT_BATCH_1_GUIDANCE.md` — Batch-1 integration pattern (model for Batch-2 guidance doc).
- `docs/STANDARDS_COMPLIANCE_INTEGRATION_GUIDANCE.md` — meta-rules for research → spec integration.
- `specs/STANDARDS_COMPLIANCE_SPEC.md` — controlling standards spec (Pass 0 disposition refresh target).
- `specs/ENGINE_SPEC.md` — canonical types (Pass 1 schema bedrock target).
- `specs/SCHEMATIC_EDITOR_SPEC.md` — authored schematic object model (Pass 4 update target).
- `specs/SCHEMATIC_CONNECTIVITY_SPEC.md` — connectivity resolution (Pass 5 update target).
- `specs/NATIVE_FORMAT_SPEC.md` — on-disk persistence (Pass 2 update target).
- `specs/IMPORT_SPEC.md` — import semantics (Pass 6 update target).
- `specs/MCP_API_SPEC.md` — MCP API (Pass 3 update target).
- `specs/ERC_SPEC.md` — ERC pin types (no Domain 3 edit needed; cross-ref only).
