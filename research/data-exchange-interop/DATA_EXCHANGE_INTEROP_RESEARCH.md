# Data Exchange & Interop — Industry Survey & Datum EDA Implementation Strategy

> Phase 2 deep-dive on Domain 1 of the 8-domain standards audit.
> Continues from `research/standards-audit/STANDARDS_AUDIT.md § 1`
> ("Per-Domain Audit → 1. Data exchange & interop").
> Cross-references `research/ipc-compliance/IPC_COMPLIANCE_RESEARCH.md`
> for IPC-2581 (DPMX), Gerber X3, IPC-D-356, and IPC-1752; this report
> does not re-research those standards, only adds the data-exchange-
> specific facets the IPC research did not cover (consumer-side
> support survey, comparative trade-offs, Datum implementation order).
>
> Companion to `research/airwire-rendering/AIRWIRE_RENDERING_RESEARCH.md`
> and `research/copper-rendering/COPPER_RENDERING_RESEARCH.md` for
> tone, structure, depth, and source-citation style.

> **Pending Exclusions Policy (verbatim, ratified 2026-04-17):**
>
> > The audit's "Recommended low-priority / skip" list is an
> > **advisory exclusion** for Phase 2 work. Phase 2 agents MUST NOT
> > re-investigate these standards. Final ratification of skips into
> > binding scope documents happens in a single consolidated pass
> > after Domain 8 lands, when full cross-domain context is available.
>
> For Domain 1 the advisory exclusion list contains: **Specctra
> DSN/SES** (legacy routing exchange), **Hyperlynx HYP** (single-vendor
> SI-extraction format), **Altium / OrCAD / PADS commercial native
> binary formats** (out-of-v1 per `specs/PROGRAM_SPEC.md` and
> `docs/COMMERCIAL_INTEROP_STRATEGY.md`), and **Windchill /
> Teamcenter / Aras / Arena** (PLM systems, belong to Domain 7).
> These were not deep-dived. They are surfaced under "Pending
> Exclusions (re-affirmed)" with cross-cutting-value notes.

## Executive Summary

- **ECAD↔MCAD exchange is the largest blind spot in Datum's spec
  surface and the single highest-leverage interop investment.**
  Every serious user above the hobbyist tier has an enclosure team,
  and the enclosure team uses SolidWorks, Fusion 360, Inventor,
  Creo, NX, or FreeCAD — none of which read native KiCad/Eagle
  formats. STEP AP242 (the modern target), STEP AP203 / AP214
  (legacy but ubiquitous), IDF 3.0/4.0 (the compatibility floor),
  IDX (incremental ECO exchange, ProSTEP iViP), and EDMD (the
  ProSTEP iViP modern XML schema) are all blind spots in the
  current spec. Datum's `ModelRef { path, transform }` slot
  (`specs/ENGINE_SPEC.md:74-77`) and `Package.models_3d` field
  (`specs/ENGINE_SPEC.md:138`) reserve room for the data, but no
  format is named, no transform encoding is specified, and KiCad
  3D-model imports are explicitly deferred
  (`specs/IMPORT_SPEC.md:135`). The first concrete step is to bind
  `ModelRef.path` to STEP files and define the transform schema.
- **STEP AP242 is the modern industry target for ECAD↔MCAD; STEP
  AP203 is the credible compatibility floor.** AP203 (ISO 10303-203)
  is "configuration-controlled 3D" and is the broadest-supported
  format on the MCAD side — every consumer tool reads it. AP214
  (ISO 10303-214) is the automotive variant; effectively a superset
  of AP203 with manufacturing/assembly process metadata. AP242
  (ISO 10303-242, edition 2 published 2020, edition 3 in
  preparation) merges AP203 + AP214 and adds PMI (product and
  manufacturing information), semantic GD&T, composites, kinematics,
  tessellated geometry, and external-element references. For Datum's
  purposes, **AP242 export of board+placed-components is the right
  long-term target**, with AP203 fallback for older MCAD
  installations.
- **IDF 3.0 is dead but undead; IDF 4.0 was a modernisation attempt
  that nobody adopted.** IDF (Intermediate Data Format) 3.0 (1996,
  Mentor) is the legacy ECAD↔MCAD board-and-component-outline
  exchange — board outline, component placements, simple component
  bodies (rectangular prisms with height). Every MCAD tool reads
  IDF 3.0. IDF 4.0 (2003) added arbitrary 3D body geometry,
  stack-up, and mounting holes; effectively no ECAD or MCAD tool
  shipped IDF 4.0 in production. The successors are STEP AP242 +
  IDX (for incremental ECO) and EDMD (for ProSTEP iViP-aligned XML).
  Datum should ship **IDF 3.0 export** as the "works with anything"
  fallback and not waste effort on IDF 4.0.
- **IDX (ProSTEP iViP) and EDMD (ProSTEP iViP) are the modern
  ECAD↔MCAD ECO-loop standards.** IDX = "Incremental Design
  Exchange", a binary-Tarball+XML format for ECO round-trips
  (mechanical engineer moves a connector, ECAD engineer accepts /
  rejects, design state advances). EDMD = "Electrical Design Model
  Data", an XML schema for the same exchange in a more open form.
  Both are published by the ProSTEP iViP CAx Implementor Forum.
  The XML form has free reference parsers; the Tarball form
  ("EDA-MDA Implementer Forum Recommended Practice") is the version
  Altium / SolidWorks / Creo actually exchange. **IDX is the
  practical target if Datum ever wants Altium-class ECO interop;
  EDMD is the open-spec inheritor.** The audit recommendation to
  pair "STEP AP242 + IDX" (high-priority) versus "IDF 3.0 + STEP
  AP203" (compatibility floor) holds: **ship both pairs**, IDF 3.0
  + AP203 first because the ecosystem is wider, then layer AP242
  + IDX/EDMD on top.
- **ODB++ ownership has changed but the spec is still openly
  published; the IP situation is more nuanced than "Mentor
  proprietary".** ODB++ originated at Valor (Israeli company,
  acquired by Mentor Graphics 2010, Mentor in turn acquired by
  Siemens 2017). The current owner is **Siemens Digital Industries
  Software**. The format spec is published openly under the
  ODB++ Solutions Alliance; the v8.1 specification is downloadable
  from `odb-sa.com` after a (free) form submission. It is **NOT
  royalty-bearing** — anyone can implement an ODB++ reader/writer
  without paying Siemens. The license restriction is on the *brand*
  ("ODB++") and the Siemens-published validation suite. Several
  open-source projects implement ODB++ readers
  (`pcb-tools-odb-plus`, `gerber-rs` partial support, KiCad's own
  ODB++ exporter shipped 2024). For Datum, ODB++ export is feasible
  without partnering with Siemens; the **format is folder-of-files
  with an ASCII text manifest, much simpler to write than IPC-2581's
  XML schema**.
- **The fab-house format preference matrix in 2026: Gerber X2 is
  still the universal floor; ODB++ is the professional preference;
  IPC-2581 is gaining ground but not yet universal.** Survey of
  major prototype fabs (JLCPCB, PCBWay, OSH Park, Sierra Circuits,
  Advanced Circuits, Eurocircuits, Multi-CB) and assembly houses
  (JLCPCB Assembly, PCBWay Assembly, MacroFab, Screaming Circuits)
  finds: **all** accept Gerber + Excellon + CSV pick-and-place; all
  professional fabs (Sierra, Advanced Circuits, Eurocircuits,
  Multi-CB) prefer ODB++ where available; IPC-2581 is accepted by
  Sierra, Advanced Circuits, Eurocircuits, Multi-CB but not by the
  Asian volume fabs (JLCPCB, PCBWay) as of mid-2026. The fab and
  assembly preference splits: **fab houses prefer ODB++/IPC-2581
  (richer netlist + impedance metadata); assembly houses live on
  CSV pick-and-place + BOM.** Datum's Gerber X2 + Excellon + CSV
  PnP coverage (`docs/INTEROP_SCOPE.md:73-77`) hits the universal
  floor; ODB++ and IPC-2581 are the next two professional-grade
  upgrades.
- **Vendor-neutrality is a real differentiator that Datum should
  market.** STEP is ISO. Gerber is open and royalty-free (Ucamco
  published). IPC-2581 is open consortium spec. EDMD is ProSTEP
  open. IDF 3.0 is documented in a Mentor-published spec that
  anyone can implement. ODB++ is Siemens-controlled but the spec
  is openly published. Specctra DSN/SES is open-spec but legacy.
  IDX is ProSTEP-published. **The only formats with closed-source
  encumbrance are the commercial native binaries (Altium .PcbDoc
  / .SchDoc, OrCAD .brd, PADS .pcb) and DWG (Open Design Alliance
  for non-Autodesk implementations).** This is a genuine open-stack
  argument Datum can make.
- **EDIF 2 0 0 is dead in practice; AltiumSchDoc / SchLib remain
  out of v1 scope per PROGRAM_SPEC.** EDIF (Electronic Design
  Interchange Format) 2 0 0, IEEE 1366 / IEC 61691-3, was the
  '90s netlist exchange standard and is still nominally supported
  by Cadence Allegro, Mentor PADS, and OrCAD as a dump format.
  No modern tool (KiCad, Altium since AD15, Eagle since v6) has
  exercised EDIF in years. The migration value is near zero;
  modern netlist exchange happens through IPC-2581, ODB++, or
  vendor-specific text netlists (Allegro Telesis, OrCAD .net,
  PADS .asc). EDIF is **not** a recommended Datum target.
- **Library exchange is dominated by SnapEDA / UltraLibrarian /
  Component Search Engine, not by any open standard.** The "library
  JSON exchange" question has no winner. KiCad's `.kicad_sym` /
  `.kicad_mod` is becoming a de-facto format because vendors
  (SnapEDA, UltraLibrarian) export to it alongside Altium /
  OrCAD / Eagle. There is no IPC equivalent of "library exchange
  format". Datum's pool is the right architecture; the practical
  ingestion path is **import KiCad / Altium library packages from
  SnapEDA → Datum pool**.
- **Datum has eight concrete recommended spec edits that fall out
  of this research.** They are listed at the end and bracketed by
  effort estimates so the project owner can sequence them. The
  highest-leverage single edit is to extend
  `docs/INTEROP_SCOPE.md`'s "Future M5+" list to enumerate
  IDF 3.0 / STEP AP203 / STEP AP242 / IDX / EDMD / ODB++ / IPC-2581
  / IPC-D-356 separately rather than the current four-item lump.
- **Datum's transaction + sidecar model is uniquely well-suited to
  IDX-style incremental ECO exchange.** IDX assumes an
  ECAD-baseline state and applies a delta; Datum's
  Operation/OpDiff/Transaction primitives
  (`specs/ENGINE_SPEC.md:684-731`) and `.ids/` UUID sidecar
  (`specs/IMPORT_SPEC.md:60-107`) are the right substrate for that
  flow without any new architecture. This is a Datum
  differentiator — KiCad's `.kicad_pcb` does not natively express
  per-transaction deltas; Altium's exchange goes through the IDX
  exporter, not the design database. **Datum could theoretically
  emit live IDX deltas as transactions commit**, which no other
  tool surveyed does today.

## Standards Catalog

### ECAD↔MCAD Exchange

#### STEP AP242 (ISO 10303-242)

**Full title.** *Industrial automation systems and integration —
Product data representation and exchange — Part 242: Application
protocol: Managed model-based 3D engineering.*

**Issuing body.** ISO TC184/SC4 (with active US/EU industry
consortia: ASD-STAN, NIST, ProSTEP iViP, AFNeT).

**Revision.** First edition 2014; **second edition (current
authoritative) 2020**; third edition in preparation as of 2026
(no published date). The 2020 edition added composites, semantic
PMI tessellation, kinematics improvements, and cleaner external-
element-reference (XER) handling.

**Scope.** STEP AP242 is the modern unified application protocol
for 3D mechanical design exchange. Merges the two prior
mainstream 3D exchange protocols (AP203 "configuration-controlled
design" and AP214 "automotive mechanical design") and adds PMI
(geometric and dimensional tolerances rendered semantically, not
just as graphical annotations), composites, kinematics, and
external-element references. AP242 deliberately covers the
"managed-model" lifecycle picture: not just geometry, but
configuration version, classification, and approval-state
metadata.

**Format.** Two physical encodings:
- **STEP physical file (`.step` / `.stp`)** — ASCII text, ISO
  10303-21. The format that exists in the wild.
- **STEP-XML (ISO 10303-28)** — XML encoding of the same
  information model. Less common; some MCAD tools emit it on
  request.

A binary edition (STEP 10303-2X family) was discussed for years;
not in production use as of 2026.

**License / IP.** Open ISO standard. Specification access requires
ISO purchase (CHF 198 for ISO 10303-242:2020 ed. 2). The
information model itself is freely implementable. Reference
implementations (Open Cascade, STEP Tools Inc., NIST STEP-File
Analyzer) exist.

**Reference implementations.**
- **Open Cascade Technology (OCCT)** — LGPL 2.1, the dominant
  open-source STEP reader/writer. Used by FreeCAD, KiCad's
  `kicad-cli pcb export step`, and Horizon EDA. Heavy C++ library
  (~1.5M LOC). Provides full AP203 / AP214 / AP242 read/write
  with caveats around AP242 PMI semantic export.
- **PythonOCC** — LGPL Python bindings to OCCT.
- **STEPcode** — BSD-licensed C++ library, much smaller, AP203
  focused; AP242 support is partial. Originally NIST.
- **STEP Tools Inc.** — commercial reference parser, used by NIST
  for STEP-File Analyzer.
- **occt-rs** — early-stage Rust bindings to OCCT, MIT-licensed.
  Limited surface coverage.

**EDA tool support matrix (write):**
- **Altium Designer** — STEP AP203 / AP214 export (no AP242 yet
  as of AD24). MCAD Co-Designer integration works through AP203.
- **Cadence Allegro / OrCAD X** — STEP AP203 export. AP242
  through the Cadence Mechanical Co-Designer add-on.
- **Siemens Xpedition / PADS** — STEP AP242 export native
  (Siemens has the strongest AP242 implementation; expected, given
  ProSTEP iViP membership).
- **KiCad 7+** — STEP export via OCCT through `kicad-cli pcb
  export step`. AP203 + AP214 + AP242 (whichever OCCT version
  shipped).
- **Eagle / Fusion Electronics** — STEP export (AP203/214) since
  Fusion 360 integration; quality is limited by Fusion's MCAD
  representation.
- **Horizon EDA** — STEP export via OCCT, AP203/214 via
  `horizon-pool-prj-export-step`.
- **LibrePCB** — no STEP export as of 1.0.
- **DipTrace** — STEP AP203 export.
- **EasyEDA** — STEP AP203 export through the EasyEDA Pro
  workflow (consumer EasyEDA does not).

**MCAD tool support matrix (read):**
- **SolidWorks** — full AP203 / AP214 / AP242 read; PMI semantic
  read since SW 2018.
- **Inventor** — full AP203 / AP214 / AP242 read.
- **Fusion 360** — AP203 / AP214 read, AP242 read.
- **Creo Parametric** — full AP203 / AP214 / AP242 read with
  semantic PMI.
- **NX (Siemens)** — full AP203 / AP214 / AP242 with native
  ProSTEP iViP fidelity.
- **FreeCAD 0.21+** — AP203 / AP214 read via OCCT; AP242 read is
  partial (geometry yes, PMI not).
- **Onshape** — AP203 / AP214 / AP242 read.
- **CATIA V5/V6** — full AP203 / AP214 / AP242 read.

**Datum current coverage.** Blind Spot. `ModelRef.path` reserves
the slot (`specs/ENGINE_SPEC.md:74-77`); KiCad 3D imports are
deferred (`specs/IMPORT_SPEC.md:135`). No STEP code in the engine
or CLI as of M7-opening.

**Implementation cost (Datum).**
- **Canonical IR**: extend `ModelRef` to make the `transform`
  field a typed struct (translation in nm, rotation in 0.1°,
  scale 1×1×1 with optional anisotropic scale flag). Add
  `model_format: ModelFormat { Step | Wrl | Iges | Obj | Gltf }`
  alongside `path`. Add `Package.body_outline_3d:
  Option<Vec<Polygon3D>>` for the simplified-extrusion fallback
  used by IDF.
- **Pool**: pool needs to store STEP files referenced by `path`.
  Today `Package` has `models_3d: Vec<ModelRef>`; the pool index
  must learn to follow these references and validate file
  existence. Add a `models/` subdirectory convention to the pool
  layout (`docs/POOL_ARCHITECTURE.md` extension).
- **Transaction model**: no change — STEP export is read-only,
  no new operations needed.
- **MCP API additions**: `export_step { board_uuid,
  output_path, format: "AP203"|"AP214"|"AP242", include_components
  }` and `validate_step { path }` with the same error-schema
  surface as `validate_kicad`.
- **Minimum viable**: AP203 export of board outline + extruded
  copper layer (no per-component STEP merge). Effort: 2-3 weeks
  with OCCT FFI through `occt-rs` or a thin C++ shim.
- **Full implementation**: AP242 export with per-component STEP
  body merge, board-and-stackup as a multi-body assembly,
  semantic-PMI courtyard / restricted-zone export. Effort:
  3-4 months including OCCT FFI hardening.
- **Partner / library dependencies**: **Open Cascade Technology
  (LGPL 2.1)** is the only practical option in 2026. Pure-Rust
  STEP writers exist (`stepedit`, `step-rs`) but are early-stage
  and AP203-only. LGPL is compatible with Datum's likely
  licensing posture (TBD per `docs/LICENSING.md`); dynamic linking
  satisfies LGPL 2.1.

**Strategic recommendation.** **Implement post-M7.** STEP AP203
export first (works with every MCAD tool), AP242 layered on top
when courtyard / PMI semantics need to round-trip. Ship behind
an `--enable-step` build flag during M8 incubation so the OCCT
dependency is opt-in.

**Risks and edge cases.**
- OCCT is heavy (~80MB compiled). Linking statically blows up
  binary size; dynamic linking adds a runtime dependency that
  Linux distros handle fine but Windows packagers complain about.
- AP242 PMI semantic export is genuinely hard; the courtyard
  polygon → AP242 PMI tolerance-zone mapping is research-grade
  work, not implementation work. Defer until users ask.
- Component body merge requires a coordinate-system convention.
  Industry default: Z = up from the top surface of the board,
  origin at the board (0,0). KiCad and Altium agree; Eagle uses
  origin = component centre; Horizon agrees with KiCad.
  **Datum should adopt the KiCad/Altium convention**.

#### STEP AP203 (ISO 10303-203) / STEP AP214 (ISO 10303-214)

**Full title (AP203).** *Application protocol: Configuration
controlled 3D designs of mechanical parts and assemblies.*

**Full title (AP214).** *Application protocol: Core data for
automotive mechanical design processes.*

**Issuing body.** ISO TC184/SC4.

**Revision.** AP203 second edition 2011 (legacy maintenance only;
deprecated in favour of AP242). AP214 third edition 2010 (legacy
maintenance only). Both are still active ISO standards but
flagged "do not use for new applications" in ISO documentation —
AP242 supersedes both.

**Scope.** AP203 = pure 3D geometry exchange + minimal
configuration-management metadata. AP214 = AP203 superset with
automotive-process metadata (kinematics, tolerances, surface
finish). For ECAD↔MCAD use, the practical difference is
negligible: both export board geometry + component bodies
adequately.

**Adoption status 2026.** **Mainstream (legacy compat).** Every
MCAD tool reads them; almost every ECAD tool exports them. AP242
is taking over slowly; AP203 will remain the floor for years
because it is "the format that always works".

**License / IP.** Open ISO. Same situation as AP242.

**Reference implementations.** OCCT, STEPcode (AP203 native),
STEP Tools.

**Datum current coverage.** Blind Spot — same situation as AP242.

**Strategic recommendation.** **AP203 export is the right first
delivery** because every MCAD installation reads it. AP214 is
not worth a separate code path; AP242 is the next step up. The
practical ladder is **AP203 → AP242**, skipping AP214.

#### IDF 3.0 / IDF 4.0 (Mentor Intermediate Data Format)

**Full title.** *IDF — Intermediate Data Format for ECAD/MCAD
data exchange.*

**Issuing body.** Originally Mentor Graphics (1992); now de-facto
public-domain since the spec has not been actively maintained
since IDF 4.0 in 2003.

**Revision.** **IDF 2.0 (1996)**, **IDF 3.0 (1996, the version
everyone supports)**, **IDF 4.0 (2003, never gained traction)**.
There is no IDF 5.

**Scope.** Plain-text format. Two files per design: `.emn`
(board outline + component placement + holes) and `.emp`
(component library — outline polygon + height for each component).
Component bodies are constrained to extruded prisms (a footprint
polygon + Z-min + Z-max). No STEP fidelity, no curved surfaces.
IDF 4.0 added arbitrary-mesh component bodies, board cutouts,
keep-in / keep-out areas, and stack-up data; it is essentially
unsupported by EDA tools, so the spec sits unused.

**Adoption status 2026.** **Legacy ubiquitous.** IDF 3.0 reads
on **every** MCAD tool — SolidWorks (CircuitWorks add-on),
Inventor, Creo, NX, FreeCAD (via `kicad-StepUp` plugin), Fusion
360. It is the universal "Plan B when STEP doesn't work" format.
Effort to implement is trivial (text format, ~150 lines of
parser code).

**License / IP.** Free; spec was published openly by Mentor
without restriction. Public-domain in practice.

**Reference implementations.**
- **kicad-StepUp** — FreeCAD plugin that reads/writes IDF 3.0;
  open source, GPLv3.
- **idf2vrml** — historical converter, public domain.
- KiCad's own IDF 3.0 export module (`pcbnew/exporters/`),
  GPL3.

**EDA tool support matrix (write):**
- **Altium** — IDF 3.0 export native.
- **Allegro / OrCAD** — IDF 3.0 export native.
- **Xpedition / PADS** — IDF 3.0 + IDF 4.0 export (Siemens kept
  IDF 4.0 alive).
- **KiCad** — IDF 3.0 export native (legacy from 4.x; still
  shipped).
- **Eagle / Fusion** — IDF 3.0 export.
- **Horizon EDA** — no IDF export as of 2.4.
- **LibrePCB** — no IDF export.
- **DipTrace** — IDF 2.0 / 3.0 export.
- **EasyEDA** — no IDF export.

**MCAD tool support matrix (read):**
- **SolidWorks** — IDF 3.0 read via CircuitWorks (paid add-on,
  shipped with SW Premium).
- **Inventor** — IDF 3.0 read native.
- **Creo** — IDF 3.0 read via Creo ECAD.
- **NX** — IDF 3.0 + IDF 4.0 read native.
- **Fusion 360** — IDF 3.0 read.
- **FreeCAD** — IDF 3.0 read via kicad-StepUp plugin.
- **CATIA** — IDF 3.0 read via CATIA Electrical 3D Design.
- **Onshape** — no native IDF 3.0 read (workaround:
  third-party converter).

**Datum current coverage.** Blind Spot.

**Implementation cost (Datum).**
- **Canonical IR**: needs `Package.body_outline_2d` (already
  present as `courtyard: Polygon`) + `Package.body_height_nm:
  Option<i64>` + `Package.body_height_mounted_nm: Option<i64>`
  for tall components. Today there is no body-height field
  anywhere.
- **Pool**: extend pool query surface to fetch body height.
- **Transaction model**: no change.
- **MCP API additions**: `export_idf { board_uuid, output_dir,
  version: "3.0" }`.
- **Minimum viable** = full implementation. IDF 3.0 has no
  optional features worth deferring. Effort: ~1 week including
  parser/writer + golden tests against KiCad-generated IDF 3.0.
- **Partner / library dependencies**: none. Pure-Rust
  implementation, no external library.

**Strategic recommendation.** **Implement post-M7, before
STEP AP242, before any other ECAD↔MCAD format.** IDF 3.0 is the
universal floor and the cheapest format Datum can ship that
materially helps real users. Ship as part of the same
"manufacturing export" group that already covers Gerber + BOM +
PnP (`docs/INTEROP_SCOPE.md:73-77`).

**Risks and edge cases.**
- IDF 3.0 component bodies are extruded prisms only. Tall ICs
  with heatsinks, electrolytic capacitors, large connectors all
  collapse to "rectangular prism height H". This is a known
  industry limitation, not a Datum problem.
- Component rotation is encoded in 0.01° but most MCAD tools
  round to 0.1°. Datum's existing 0.1° angle resolution
  (`docs/CANONICAL_IR.md`) is fine.

#### IDX (ProSTEP iViP Incremental Design Exchange)

**Full title.** *ECAD-MDA Implementer Forum Recommended Practice
— Incremental Design Exchange (IDX).*

**Issuing body.** ProSTEP iViP CAx-IF (Computer-Aided x
Implementer Forum) jointly with PDES Inc. (US industry
consortium).

**Revision.** Recommended Practice **v1.4 (2017, current)**.
v1.3 (2014) is the version most installed tools support.

**Scope.** Incremental ECO exchange between ECAD and MCAD. Not a
full design dump like STEP — instead, a **delta** describing
what changed since the last accepted state. Each IDX file is a
*proposal* (component moves, adds, removals, mounting-hole
changes, board-outline changes, keepout changes) that the other
side accepts or rejects per item. The accepted set advances both
sides' baseline.

**Format.** Binary container (ZIP) with XML payloads. Two file
types: `.idx` (proposal) and `.idx-resp` (response with
accept/reject per change item). The XML schema is the EDMD
schema (see below) wrapped in transaction / change-item metadata.

**Adoption status 2026.** **Mainstream professional, niche
overall.** Used heavily in automotive (Bosch, Continental,
Daimler), aerospace (Boeing, Airbus), and large industrial
(Siemens, ABB) ECAD↔MCAD workflows. Outside those verticals,
adoption is thin. The actual exchange volume is tiny compared to
STEP, but the workflow value is high (no full re-export per ECO).

**License / IP.** Open spec. Published by ProSTEP iViP at
`prostep.org`; reference XSD freely available. No royalty.

**Reference implementations.**
- **ProSTEP iViP CAx-IF reference test data** — public, used for
  conformance testing.
- **Mentor Xpedition / Cadence Allegro / Altium / Zuken** — all
  ship native IDX writers/readers as part of their MCAD
  Co-Designer products.
- **No high-quality open-source IDX library exists.** The
  `pyEDMD` project covers the XML schema (which IDX wraps); IDX
  transactional layer is proprietary in practice.

**EDA tool support matrix.**
- **Altium** — full IDX read/write via MCAD Co-Designer.
- **Cadence Allegro** — full IDX support.
- **Siemens Xpedition** — full IDX support (as expected, given
  ProSTEP role).
- **Mentor PADS** — IDX support.
- **Zuken CR-8000 / CADSTAR** — IDX support.
- **KiCad** — no IDX support as of 9.x. Several feature requests
  exist; community pushback because the workflow is large-org.
- **Eagle / Fusion** — no IDX support.
- **Horizon / LibrePCB / DipTrace / EasyEDA** — no IDX support.

**MCAD tool support matrix.**
- **SolidWorks** — full IDX via CircuitWorks Enterprise.
- **Creo** — full IDX via Creo ECAD.
- **NX** — full IDX (Siemens flagship integration).
- **CATIA V6** — full IDX.
- **Inventor** — IDX via add-on (less mature).
- **Fusion 360** — no IDX.
- **FreeCAD** — no IDX.

**Datum current coverage.** Blind Spot.

**Implementation cost (Datum).**
- **Canonical IR**: no new types needed; the change-item set maps
  to existing operations (add/remove/move package, change
  outline, etc.).
- **Pool**: no change.
- **Transaction model**: **this is where Datum has a
  differentiator.** The Operation/OpDiff/Transaction primitives
  (`specs/ENGINE_SPEC.md:684-731`) and `.ids/` UUID sidecar
  (`specs/IMPORT_SPEC.md:60-107`) are the substrate IDX needs.
  Datum can map "transaction since last IDX baseline" → "IDX
  proposal" mechanically.
- **MCP API additions**: `export_idx { board_uuid, baseline_id,
  output_path }`, `import_idx_response { path }`,
  `apply_idx_proposal { path, accept_uuids, reject_uuids }`.
- **Minimum viable**: IDX *write* only (no response handling).
  Effort: ~3 weeks including XML schema bind-up.
- **Full implementation**: write + read + bidirectional response.
  Effort: ~6-8 weeks.
- **Partner / library dependencies**: ProSTEP iViP XSD is free to
  download; no commercial library required.

**Strategic recommendation.** **Implement on-demand.** IDX is a
high-value, low-volume capability. Ship if and only if a
target-market user (automotive, aerospace, large industrial)
explicitly needs it — at which point the engineering payoff is
high because Datum's transaction model is the right shape and
no other open-source ECAD has it. This is one of the few places
where Datum could plausibly be **better than KiCad** at
something professional users care about.

**Risks and edge cases.**
- The IDX response model assumes a stable identity for every
  change item. Datum's UUIDs satisfy this trivially; KiCad does
  not (KiCad regenerates UUIDs on some edits, breaking IDX
  round-trip in practice — a known KiCad gripe in the IDX
  community).
- Baseline-state tracking requires Datum to retain a snapshot of
  the last-IDX-exchanged state. Project-level metadata addition.

#### EDMD (ProSTEP iViP Electrical Design Model Data)

**Full title.** *ECAD/MDA Implementer Forum Recommended Practice
— EDMD Schema.*

**Issuing body.** ProSTEP iViP CAx-IF.

**Revision.** **Schema v3.0 (2017, current)**; v2.0 still widely
deployed.

**Scope.** XML schema for full ECAD design model exchange (board
outline, layers, components, nets, vias, copper). Unlike IDX
(which is a delta), EDMD is a **full state** exchange. Aligned
with ISO 10303-210 (PCB application protocol) but published as a
ProSTEP open spec rather than an ISO standard.

**Format.** XML files conforming to published XSDs.

**Adoption status 2026.** **Niche but growing.** EDMD is the
"open IDX" — same XML payloads, but standalone full-state files
without IDX's transactional wrapper. Useful when one side does
not need the ECO loop. Adoption is mostly inside ProSTEP iViP
member organisations.

**License / IP.** Open. Schemas freely downloadable from
`prostep.org`.

**Reference implementations.**
- **pyEDMD** — Python library, MIT, basic schema bind-up.
- ProSTEP iViP test corpus — public conformance data.

**EDA tool support matrix.** Same as IDX (the tools that
implement IDX implement EDMD as a byproduct). KiCad / Eagle /
LibrePCB / DipTrace / EasyEDA have no EDMD support.

**Datum current coverage.** Blind Spot.

**Implementation cost.** Implementing EDMD is implementing the
core IDX schema without the transactional wrapper — strictly
cheaper than IDX. Effort: ~2 weeks.

**Strategic recommendation.** **Implement alongside IDX, not
ahead of it.** EDMD without IDX is mostly useless; IDX without
EDMD does not exist (IDX wraps EDMD). The right packaging is
"EDMD schema as a vendored module, IDX as the transactional
wrapper on top". Skip if IDX is skipped.

#### DXF / DWG (Autodesk Drawing Interchange)

**Full title.** DXF — *Drawing Interchange and File Formats*; DWG
— Autodesk Drawing.

**Issuing body.** Autodesk (proprietary; widely reverse-engineered).

**Revision.** DXF and DWG release annually with AutoCAD; current
is **AC1032 (AutoCAD 2018+)** for DWG and the matching DXF
revision. DWG file format compatibility is announced per AutoCAD
release.

**Scope.** 2D mechanical drawing exchange. PCB-relevant uses:
- **Board outline import** from a mechanical CAD drawing (the
  enclosure team hands off "this is the board cutout").
- **Drill drawing export** as DXF for fab houses that prefer it
  over Excellon.
- **Mechanical-layer drawing export** for assembly drawings.

**Format.** DXF is ASCII (older, 80-column-style records). DWG
is binary, version-locked, much smaller files.

**Adoption status 2026.** **Universal compatibility floor for
mechanical drawing exchange.** Every MCAD tool reads DXF; most
read DWG. PCB-tool import of mechanical board outlines is the
single most common use case.

**License / IP.**
- **DXF**: Autodesk publishes the DXF Reference openly. Free to
  implement.
- **DWG**: Autodesk does not publish the spec. Two practical
  paths:
  - **Open Design Alliance (ODA) Teigha / Drawings SDK** —
    commercial library, ~$15,000/year corporate licensing.
    Industry standard for non-Autodesk DWG support.
  - **LibreDWG** (GNU/GPLv3) — open-source DWG library; quality
    is uneven, AutoCAD 2010+ files often fail.
  - **DWGdirect / dwg-rs** — early-stage Rust attempts.

**Reference implementations.**
- **dxf-rs** (MIT) — Rust DXF reader/writer; mature.
- **LibreDWG** (GPLv3) — DWG reader/writer.
- **ODA Teigha** (commercial) — the professional choice.
- **dxflib** (libreoffice / KiCad) — KiCad uses an internal
  DXF parser based on dxflib (GPL).

**EDA tool support matrix.**
- **All major EDA tools** read DXF for board-outline import.
- **All major EDA tools** export DXF for mechanical layers.
- **DWG support is rarer**: Altium reads DWG via Autodesk
  RealDWG (paid OEM). KiCad reads DXF only.

**Datum current coverage.** Blind Spot. No DXF read or write
mentioned in any spec. Board outline is currently authored
through the operation API or imported from KiCad/Eagle source
files (which already contain the outline geometry).

**Implementation cost (Datum).**
- **DXF read** for board-outline import: ~3 days using `dxf-rs`.
- **DXF write** for mechanical-layer export: ~2 days using
  `dxf-rs`.
- **DWG read/write**: skip in v1; refer users to "convert to
  DXF in any DWG-aware tool first". If demanded later, ODA
  Teigha is the only sane option.

**Strategic recommendation.** **Implement DXF read/write
post-M7 as a small, cheap utility.** Skip DWG — the licensing
is hostile and DXF covers 95% of real workflows.

#### JT (Siemens Jupiter Tessellation)

**Full title.** *JT — Lightweight 3D format.* ISO 14306.

**Issuing body.** Siemens (originally), now **ISO 14306:2017**.

**Revision.** ISO 14306:2017 second edition; JT 10.5 (2018) is
the current Siemens-published implementation guide.

**Scope.** Lightweight 3D format, designed for visualization and
collaboration of large MCAD assemblies. Tessellated geometry
plus optional B-rep (boundary representation) plus PMI plus
metadata. The standard "everyone in the supply chain can open
it" format for large-assembly review.

**Adoption status 2026.** **Mainstream in automotive/aerospace
MCAD, niche in ECAD.** Major automotive OEMs require JT in
tier-1 supplier deliverables. ECAD tools generally do not write
JT directly; the path is ECAD → STEP → MCAD → JT.

**License / IP.** ISO standard. Open. Reference implementations
exist (Siemens JT Open Toolkit, free download with registration).

**Datum current coverage.** Blind Spot.

**Strategic recommendation.** **Out of scope.** JT is one path
removed from where Datum operates (ECAD → STEP → MCAD → JT).
Implement only if a specific automotive customer requires
direct JT export.

#### glTF / GLB (Khronos Group)

**Full title.** *glTF — GL Transmission Format.* Khronos Group.

**Issuing body.** Khronos Group (the OpenGL/Vulkan/WebGL body).

**Revision.** **glTF 2.0 (2017, current)**.

**Scope.** Web-friendly 3D format. Binary (`.glb`) or text
(`.gltf` + accompanying `.bin` + textures) encoding of mesh +
material + animation data. Optimised for runtime rendering
(GPU buffers ready to upload) rather than CAD round-trip.

**Adoption status 2026.** **Mainstream for web 3D viewers, niche
for ECAD↔MCAD.** Used by browser-based PCB viewers (e.g., the
KiCad PCB Viewer plugin, JLCPCB online viewer). Not a CAD
exchange format.

**License / IP.** Royalty-free Khronos spec.

**Reference implementations.** `gltf-rs` (MIT, mature),
`cgmath`-friendly tessellation libraries.

**Datum current coverage.** Blind Spot.

**Strategic recommendation.** **Implement only when M7 GUI
needs it for 3D preview.** Not relevant before M7 ships a 3D
viewer (currently deferred per `CLAUDE.md` "Not Yet Implemented"
section). When it is needed, the path is `STEP/IDF → tessellation
→ glTF` which fits cleanly on top of an OCCT-based STEP
export.

#### OBJ / STL (note only)

**OBJ.** Wavefront `.obj` format. Public-domain text format for
mesh exchange. Mature ecosystem (`obj-rs`, MIT). Hobbyist /
3D-print compatibility. Not relevant for ECAD↔MCAD professional
exchange. **Skip in v1.**

**STL.** Stereolithography format. Triangle-mesh-only, no
material, no colour. Universal for 3D printing. Trivial to
implement. **Implement only if/when Datum offers a 3D-print
mode for board mockups; not in v1.**

### Fabrication / Assembly Output

#### ODB++ (Open Database)

**Full title.** *ODB++ Solutions Alliance Format Specification.*

**Issuing body.** Originally **Valor** (acquired by Mentor
Graphics 2010, then Mentor by Siemens 2017). Current owner:
**Siemens Digital Industries Software**, published under the
**ODB++ Solutions Alliance** umbrella (`odb-sa.com`).

**Revision.** **ODB++ v8.1 (2020, current)**. Active revision
schedule; v8.2 in preparation as of 2026.

**Scope.** Comprehensive PCB fabrication and assembly database.
Folder-of-files structure with an ASCII text manifest. Carries:
- Layer artwork (copper, mask, silk, paste, drill, route) in
  positive-polygon form
- Stack-up with material properties, dielectric Dk/Df,
  copper weight
- Board outline and cutouts
- Component package data with placement, rotation, layer
- Net list with component-pin connectivity
- DFM annotations from the fab back to the designer
- Pad-stack data per padstack ID
- Drill data (plated, non-plated, blind, buried)
- Test points
- Assembly notes, fabrication notes
- Variant data (multi-assembly support)

**ODB++ Inside vs ODB++ Outside.** Two distribution variants:
- **ODB++ Inside** — embedded inside an EDA tool's native
  database; not a file format, an in-memory model.
- **ODB++ Outside** (sometimes "ODB++ Outbound") — the file/
  folder package that is what people mean when they say "ODB++".

**Adoption status 2026.** **Mainstream professional.** The
preferred format at every Tier-1 NA/EU prototype fab (Sierra,
Advanced Circuits, Eurocircuits, Multi-CB) and many Asian
volume fabs. **Slightly broader real-world fab adoption than
IPC-2581** despite IPC-2581 having the consortium-friendly
positioning.

**License / IP.**
- **Spec is openly published** at `odb-sa.com`. Anyone can
  download v8.1 (form submission, no payment).
- **Format is royalty-free** to read and write.
- **The ODB++ trademark is Siemens-controlled**; "branded" use
  ("ODB++-certified") requires going through Siemens's
  qualification.
- **No patent encumbrance** is claimed in the spec; in practice
  no one has been sued over an open-source ODB++ writer.

**Reference implementations.**
- **`pcb-tools-odb-plus`** (Python, MIT) — reader, partial.
- **KiCad ODB++ exporter** (GPL3, shipped 2024 in KiCad 8.x;
  more complete in 9.x) — production-quality writer, the best
  open-source reference.
- **Horizon EDA** — no ODB++ support.
- **`odbplusplus-rs`** — early-stage Rust attempt; not
  production-ready.
- **Siemens ODB++ Validator** — free download from `odb-sa.com`,
  validates output against schema.

**EDA tool support matrix.**
- **Altium** — ODB++ Outbound export native (Output Job File).
- **Cadence Allegro / OrCAD** — ODB++ export native.
- **Siemens Xpedition / PADS** — ODB++ native (as expected,
  Siemens owns it).
- **KiCad 8.x+** — ODB++ export native.
- **Eagle / Fusion** — ODB++ export through Fusion 360
  Manufacturing add-on.
- **DipTrace** — ODB++ export native (in 4.x+).
- **EasyEDA** — ODB++ export in EasyEDA Pro.
- **LibrePCB** — no ODB++ support.
- **Horizon EDA** — no ODB++ support.

**Datum current coverage.** Partial. `docs/INTEROP_SCOPE.md:89`
lists "ODB++" under "Future M5+" without specifying a revision.
No code; no schema work.

**Implementation cost (Datum).**
- **Canonical IR**: needs `Stackup.layer.material` + `dk` + `df`
  + `copper_weight` + `roughness` (currently deferred per
  `specs/ENGINE_SPEC.md:255`, "M8+: dk, df, copper_weight,
  roughness"). Without these, ODB++ output is partial.
- **Pool**: padstack ID → ODB++ symbol mapping. Datum's
  `Padstack` model is already strong here; mostly a serialization
  exercise.
- **Transaction model**: no change.
- **MCP API additions**: `export_odbpp { board_uuid,
  output_path, version: "8.1" }` and `validate_odbpp { path }`.
- **Minimum viable** (no impedance): board layers + components
  + outline + drill + netlist. Effort: ~4-5 weeks including
  golden tests against Siemens Validator.
- **Full implementation** (with stackup materials, impedance,
  variants): ~8-10 weeks.
- **Partner / library dependencies**: none required. The format
  is plain text. Cross-validation against Siemens Validator
  during development is recommended.

**Strategic recommendation.** **Implement post-M7, paired with
or after IPC-2581.** Choose ODB++ over IPC-2581 if the target
user base skews professional NA/EU fab (Sierra, Advanced
Circuits, Eurocircuits); choose IPC-2581 if the target skews
consortium-aligned (Tier-1 commercial EDA users). **Realistically
ship both** — they share most of the IR-extraction code; the
incremental cost of the second after the first is roughly half.

**Risks and edge cases.**
- ODB++ stackup material fields will require Datum's deferred
  `dk` / `df` / `copper_weight` / `roughness` to land first
  (`specs/ENGINE_SPEC.md:255-256`). Either ship ODB++ with
  empty-string materials and a known-loss notice, or block on
  the Stackup extension.
- ODB++ component placement uses package-name strings; Datum's
  UUID-keyed packages need a stable name-derivation rule.
  KiCad uses `<library>:<footprint_name>`; Altium uses
  `<library>!<footprint>`. Pick one (KiCad's convention is
  more readable).
- The CAD section vs the FAB section of an ODB++ archive carry
  redundant data; writers choose how much to populate. Minimum
  viable = CAD section only.

#### IPC-2581 (DPMX) — cross-reference IPC research

**See `research/ipc-compliance/IPC_COMPLIANCE_RESEARCH.md`
§IPC-2581 (lines 625-690)** for full coverage including:
revision history through Rev C (November 2020), tool support
matrix as of mid-2026, schema availability, comparison to
ODB++ and Gerber X3.

**This deep-dive adds:**
- **Consortium adoption rates 2026.** IPC-2581 Consortium
  members in early 2026: 30+ EDA vendors / fab houses, including
  all three Tier-1 commercial (Cadence, Altium, Siemens), KiCad
  (since 2022), several major fab houses (Sierra Circuits,
  Advanced Circuits, Eurocircuits, NCAB Group). Notable
  abstainers: Mentor PADS (despite Siemens parent), Autodesk
  Fusion Electronics, EasyEDA / LCSC.
- **Viewer ecosystem.** Free public viewers: **Sierra Circuits
  IPC-2581 Viewer** (web), **Multi-CB IPC-2581 Viewer** (web),
  **CIMS Tech Inc. Lavenir Viewer** (Windows desktop, free
  download), **Cadence's Allegro Free Viewer** (with IPC-2581
  import). The viewer ecosystem is healthier than ODB++'s
  (ODB++ requires the Siemens Valor viewer or Sierra's hosted
  viewer).
- **Datum recommendation.** Implement Rev C (not Rev B) when
  the time comes — Rev C carries controlled-impedance schema
  (`<ImpedancesProperties>`) that Datum will eventually want
  for the deferred SI rule scaffold (`specs/ENGINE_SPEC.md:586-594`).
  Schema is XSD-based and free from the consortium.

#### Gerber X3 — cross-reference IPC research

**See `research/ipc-compliance/IPC_COMPLIANCE_RESEARCH.md`
§IPC-2581 (lines 671-678)** for the IPC-2581-vs-Gerber-X3
comparison. Datum's existing Gerber RS-274X / X2 export
coverage is in `docs/INTEROP_SCOPE.md:73-77`.

**This deep-dive adds: aperture-attribute (X3) consumer
support survey.**

Gerber X3 (2018) extends X2 with assembly-side attributes
(`%TF.AperFunction,...*%`, `%TF.ComponentRotation,...*%`,
`%TF.ComponentValue,...*%`) so a Gerber + an X3-aware viewer
can produce assembly drawings without a separate IPC-2581 /
ODB++ file. The key X3 attributes:
- `%TF.AperFunction,SMDPad,...*%` — pad function classification
- `%TF.ComponentRotation,...*%` — rotation
- `%TF.ComponentValue,...*%` — component value
- `%TF.ComponentMfr,...*%` / `%TF.ComponentMPN,...*%`
- `%TF.Pin,...*%` — pin number

**Consumer support 2026:**
- **Ucamco UcamX** — full X3 (Ucamco is the Gerber publisher).
- **Frontline (part of Mentor/Siemens)** — full X3.
- **CAM350 (Downstream)** — partial X3.
- **GC-Prevue (Graphicode)** — full X3 read.
- **gerbv (open source)** — X2 only as of 2.10; X3 attribute
  pass-through but no rendering of assembly metadata.
- **KiCad's gerbview** — partial X3 (reads attributes but does
  not render assembly views).
- **Gerber Tools (npm)** — X2 + partial X3.

**Practical takeaway for Datum.** Datum's Gerber X2 export is
already in scope; **upgrading to X3 attributes is a marginal
addition (~2-3 days of work)** and should ride along whenever
the X2 exporter is touched. Most consuming tools either ignore
unknown X3 attributes or use them; emitting them is upside-only.

#### Excellon 2 / NC drill

**Full title.** Excellon Format 2 (often "Excellon 2", sometimes
just "Excellon").

**Issuing body.** Originally Excellon Automation (drill-machine
manufacturer); now de-facto open / public-domain. Successor
specifications maintained by IPC as **IPC-NC-349** (CNC machine
control) and Ucamco as **NC drill in Gerber** (XNC).

**Revision.** Excellon Format 2 (1979); IPC-NC-349 latest
revision is from 1996 and not actively maintained. **XNC**
(2014, Ucamco-published as "NC drill in Gerber syntax") is the
modern Gerber-aligned equivalent.

**Scope.** ASCII fixed-format drill and route file. Two file
types per board:
- `<board>.drl` for plated drilled holes
- `<board>-NPTH.drl` for non-plated through-hole

Coordinates in inches or mm with explicit precision flags.
Tools defined in a header section, drill operations referenced
by tool number.

**Mentor extensions.** Mentor (now Siemens) added several
extensions for CAM ergonomics: tool-change comments, decimal
coordinates, embedded zero-suppression flags. Mentor extensions
are widely accepted by fab CAM systems but not part of the
original Excellon spec.

**Adoption status 2026.** **Universal.** Every fab accepts
Excellon. XNC is gaining ground but not yet universal.

**License / IP.** Public-domain in practice.

**Reference implementations.** `excellon` Python library;
KiCad's drill writer; Ucamco's XNC reference parser.

**Datum current coverage.** Covered (M4 manufacturing export,
`docs/INTEROP_SCOPE.md:75`, `specs/PROGRAM_SPEC.md:231`).

**Strategic recommendation.** **Confirm XNC support is on the
roadmap, not just legacy Excellon.** The trend is XNC because
it shares Gerber's coordinate system and avoids per-format
unit/precision footguns. If Datum's current Excellon writer
emits classic Excellon Format 2 only, plan an XNC upgrade in
the same milestone as Gerber X3.

#### Pick-and-Place files

**Formats in the wild:**
- **Generic CSV** — universal lowest common denominator.
  Columns vary: `Reference, Value, Package, X, Y, Rotation,
  Layer` is the typical baseline. Most assembly houses accept
  this directly.
- **Mentor PNP** (`.pnp`) — Mentor/Siemens placement format.
  Tab-delimited with stack-up-aware coordinate origin.
- **Altium PNP** (`.csv` from Altium's "Generates pick and place
  files" output) — Altium's CSV with extra columns for variant /
  fitted state.
- **Cadence Allegro placement file** — text with Cadence-specific
  syntax.
- **JLCPCB CPL** (`.csv`) — JLCPCB's specific CSV column order
  (Designator, Mid X, Mid Y, Layer, Rotation). Differs from
  the generic CSV in column naming.
- **PCBWay PnP** — accepts JLCPCB CPL or generic CSV.
- **Macrofab PnP** — proprietary CSV variant.
- **IPC-7351 placement format** — never gained adoption; not
  used in practice.

**Adoption status 2026.** **Universal: every assembly house
accepts CSV.** The vendor-specific formats are convenience
accelerators when the column order matches the machine programmer's
expectations.

**License / IP.** All freely implementable; CSV needs no license.

**Datum current coverage.** Covered for generic CSV
(`docs/INTEROP_SCOPE.md:77`).

**Strategic recommendation.** **Add a per-vendor PnP profile
system in M8+.** A small registry of column-order templates
(JLCPCB, PCBWay, MacroFab, generic) emitted under user choice.
The IR data is identical; only the column order and naming
differ. Effort: ~2 days per profile.

#### Strategic comparison: ODB++ vs IPC-2581 vs Gerber X3

| Dimension | Gerber X3 | ODB++ v8.1 | IPC-2581 Rev C |
|---|---|---|---|
| Year of current revision | 2018 | 2020 | 2020 |
| Owner | Ucamco (open) | Siemens (open spec) | IPC consortium (open) |
| File structure | Multi-file (one per layer) | Folder-of-files + ASCII manifest | Single XML file |
| Netlist | No | Yes | Yes |
| Component placement | Yes (X3 attributes) | Yes | Yes |
| BOM | No | Yes | Yes |
| Stack-up materials (Dk/Df) | No | Yes | Yes |
| Controlled impedance per net | No | Yes | Yes (Rev C) |
| Differential pair identification | No | Yes | Yes (Rev C) |
| Drill data | Separate Excellon file | Embedded | Embedded |
| Bidirectional DFM | No | One-way + DFM annotations | Yes (Rev C, designer ↔ fab) |
| Royalty status | Royalty-free | Royalty-free | Royalty-free |
| Spec accessibility | Free PDF (Ucamco) | Free w/ form (Siemens) | Free PDF (IPC) |
| Open-source writer | Yes (multiple) | Yes (KiCad 8.x+) | Yes (KiCad 8.x+, Cadence) |
| Tier-1 fab acceptance | Universal | Strong (Sierra, Advanced, Eurocircuits) | Strong (consortium fabs) |
| Asian volume fab acceptance | Universal | Common | Less common |
| Assembly house acceptance | Strong | Strong | Strong |
| MCAD round-trip | No | No (different domain) | No |
| Single-file deliverable | No | Folder | **Yes** |
| Implementation effort for Datum | Marginal upgrade from X2 (~3 days) | Moderate (~4-5 weeks min) | Moderate (~5-6 weeks min) |
| Strategic position | Compatibility floor | Professional default | Future-proof default |

**Recommended Datum order:** Gerber X3 attribute upgrade first
(it is a marginal extension of existing Gerber X2 work),
ODB++ second (slightly easier write surface, more
real-world fab adoption), IPC-2581 Rev C third (needed for
controlled-impedance metadata Datum will want anyway when the
deferred SI rule scaffold lands per `specs/ENGINE_SPEC.md:586-594`).

### Schematic / Netlist Exchange

#### EDIF 2 0 0 (Electronic Design Interchange Format)

**Full title.** *Electronic Design Interchange Format Version 2 0 0.*

**Issuing body.** Originally Daisy / Mentor / Texas Instruments /
National Semiconductor consortium (1985); later **IEEE 1366** /
**IEC 61691-3**.

**Revision.** **EDIF 2 0 0 (1988)** is the version everyone
implements. EDIF 3 0 0 (1993) and EDIF 4 0 0 (1996) attempted
extensions for full PCB exchange and printed-circuit-board
manufacturing data; neither gained tool adoption.

**Scope.** Lisp-like S-expression schematic and netlist exchange.
Hierarchical; supports cell libraries, instances, ports, nets.
Was the standard inter-tool exchange in the late '80s and '90s.

**Adoption status 2026.** **Effectively dead.** Cadence Allegro,
Mentor PADS, OrCAD all retain EDIF as a dump format for legacy
flows. KiCad, Altium (since AD15), Eagle (since v6) have not
exercised EDIF in years. The format was superseded by IPC-2581
(for full design) and vendor-specific text netlists (for netlist
sharing).

**License / IP.** Open IEEE / IEC standard.

**Reference implementations.** `pyedif` (academic), various
ancient C libraries.

**Datum current coverage.** Blind Spot.

**Strategic recommendation.** **Out of scope.** Migration value
is near zero; designers who would use EDIF have moved on. Do
not implement.

#### Vendor netlists (PADS .asc, OrCAD .net, Allegro Telesis)

**PADS netlist (.asc).** PADS Logic exports a text netlist
listing nets with their connected pins. Format is documented in
the PADS Logic user guide. Used as the bridge between PADS Logic
(schematic) and PADS Layout (board).

**OrCAD .net.** OrCAD Capture exports text netlists in several
flavours: Allegro PCB, PADS, Layout-style. The "Allegro" flavour
is a multi-file dump; the "PADS" flavour matches the PADS
.asc format.

**Allegro Telesis.** Cadence Allegro's internal netlist format.
Multi-file (`pst*.dat` files). The format that Allegro PCB
consumes from Capture.

**Adoption status 2026.** **Mainstream within their respective
ecosystems.** Allegro Telesis is the only one a non-Cadence tool
would care about, and only for Cadence-Capture migration.

**License / IP.** All openly documented; freely implementable.

**Datum current coverage.** Blind Spot.

**Strategic recommendation.** **Implement on-demand as part of
the Commercial Interop Strategy
(`docs/COMMERCIAL_INTEROP_STRATEGY.md`).** PADS .asc → Datum
netlist is the cheapest commercial migration win because the
format is text and well documented. Allegro Telesis is much
heavier (multi-file binary-ish text). OrCAD .net is essentially
PADS .asc.

#### KiCad .kicad_sch / .kicad_sym — already supported

**Coverage.** Full per `specs/IMPORT_SPEC.md:111-163`. KiCad 7 /
8 / 9 board, schematic, and library import all in scope. No
update needed from this research; the format work is well
described.

**Cross-reference note for this report.** KiCad is becoming the
de-facto open library exchange format because library vendors
(SnapEDA, UltraLibrarian) export to it alongside the commercial
formats. Datum's pool already imports KiCad libraries
(`docs/INTEROP_SCOPE.md:101-105`), which means Datum is already
on the supply side of the largest open library ecosystem
without extra work.

#### Eagle .sch / .lbr — already supported

**Coverage.** Full per `specs/IMPORT_SPEC.md:166-211`. No update
needed. Eagle's XML + DTD-based format remains supported through
9.6.2; later Fusion 360 Electronics changes are out of scope.

#### AltiumSchDoc / SchLib / PADS / OrCAD native — out of scope

**Per `docs/INTEROP_SCOPE.md:52-68` and
`docs/COMMERCIAL_INTEROP_STRATEGY.md:142-213`, these are
explicitly out of v1 scope. This research does not revisit that
decision; the Pending Exclusions Policy explicitly defers
ratification of this exclusion to the post-Domain-8 consolidated
pass.**

**Cross-cutting note.** A user landing in Datum from Altium
will most realistically arrive via:
1. Altium Designer's "Export" → IPC-2581 Rev C (Altium ships
   this).
2. Datum imports IPC-2581 Rev C.

This is the **practical** Altium migration path that does not
require reverse-engineering `.PcbDoc` / `.SchDoc`. **If Datum
ships an IPC-2581 importer, the Altium migration story works
without ever touching native Altium files.** This is the
strongest argument for prioritising IPC-2581 *import* (not just
export) when ODB++/IPC-2581 work is sequenced.

### Pool / Library Exchange

#### Library JSON conventions

**There is no IPC standard for ECAD library exchange.** The
"library exchange format" question has no winner in 2026. The
practical exchange formats are:
- **KiCad `.kicad_sym` + `.kicad_mod`** — S-expression text;
  becoming de-facto because vendors ship to it.
- **Altium `.IntLib` integrated library** — binary; opens only
  in Altium.
- **Eagle `.lbr`** — XML; well-documented; legacy.
- **OrCAD `.olb` symbol library + `.dra` footprint drawing** —
  binary-ish.
- **PADS `.lib` library** — binary.
- **EDIF 2 0 0 cell libraries** — defined but unused.
- **IPC-2581 library section** — defined; little adoption as a
  *library* exchange (people use IPC-2581 for full designs).

**Adoption status 2026.** **No winner.** KiCad's text formats
have the most ecosystem momentum because of the SnapEDA / UL /
CSE supply side; Altium IntLibs dominate inside Altium shops.

**Datum current coverage.** Pool architecture
(`docs/POOL_ARCHITECTURE.md`) is Datum's library model. KiCad and
Eagle library import are covered (`docs/INTEROP_SCOPE.md:96-105`).

**Strategic recommendation.** **Datum's pool JSON could be
positioned as a library exchange format, but only after
M7 / M8 hardening.** The pool's UUID-keyed entity / package /
part hierarchy is more expressive than KiCad's flat
symbol+footprint pair. If Datum publishes a stable pool JSON
schema with a versioned sidecar, third-party library tools
could target it; this is a long-tail differentiator, not a v1
priority.

#### Component vendor library formats

**SnapEDA.** Library service; exports per-vendor formats
including KiCad, Altium, Eagle, OrCAD, Allegro, PADS, DipTrace,
EasyEDA. Free tier (search, single-user download); paid tier
for bulk / API access. Symbol + footprint + 3D model + datasheet
link per part. Roughly 1.5M+ parts in 2026.

**UltraLibrarian.** Mentor Graphics library service (Siemens
acquisition). Supports the same EDA tool list. Slightly stronger
on STEP 3D models. ~10M+ parts.

**Component Search Engine (CSE).** Same business model; per-tool
download. ~12M+ parts.

**Octopart.** Not a library service — a parametric search
engine. Provides pricing, availability, datasheet links;
sometimes links to SnapEDA / UL / CSE for the actual library
download.

**Datum current coverage.** None of these are part of any
current spec.

**Strategic recommendation.** **Implement KiCad-format ingestion
as the primary library-vendor ingestion path.** All three vendor
services export to KiCad format; if Datum's KiCad library import
is solid, vendor-library ingestion is automatic. **Do not target
SnapEDA / UL / CSE proprietary formats directly** — the KiCad
intermediate is the right ingestion path. This converts a
"ship N integrations" problem into a "harden one importer"
problem.

### Project Archive / Portability

#### IPC-2581 archive mode

**Coverage.** IPC-2581's design is already a single-file project
archive — that's the value proposition. Cross-reference to
`research/ipc-compliance/IPC_COMPLIANCE_RESEARCH.md` §IPC-2581.

**Distinction.** ODB++ "archive" is a folder with a manifest;
IPC-2581 "archive" is a single XML file. For long-term archival
purposes (regulated industries, AS9100 first-article inspection
records), IPC-2581's single-file delivery is more
filesystem-friendly than ODB++.

#### ODB++ project archive

ODB++ is folder-of-files. Standard practice is to ZIP the
folder for archival. The Siemens-published "ODB++ Wrapper"
convention puts the folder inside a `.tgz`. Either is fine; no
special archive mode beyond ZIP/tar.

#### Altium PCB Project Archive (.PCBPRJ / .PrjPcb)

Out of scope per Pending Exclusions.

#### Git-based ECAD versioning conventions

**Industry conventions for ECAD git repositories:**
- **`.gitignore` patterns**: exclude derived data (zone fills,
  DRC reports, build outputs). Datum's "derived not persisted as
  authored truth" rule (`specs/NATIVE_FORMAT_SPEC.md:132-146`)
  aligns with this. 
- **Diffability**: text formats only. Datum's deterministic
  JSON serialization (`docs/CANONICAL_IR.md` §Serialization)
  satisfies this.
- **Mergeability**: object-keyed JSON maps with stable UUIDs
  merge cleanly (each object is its own JSON sub-tree). Datum's
  per-sheet file split (`specs/NATIVE_FORMAT_SPEC.md:249-273`)
  helps merge granularity.
- **LFS for binaries**: 3D models (STEP files), datasheets, and
  fabrication output should go to git-lfs to avoid bloating the
  repo. Datum's `models/` pool subdirectory should be
  `.gitignore`-aware.
- **Submodule for shared pool**: shared pools (Datum's
  `docs/POOL_ARCHITECTURE.md:380-396` priority-layered pool model)
  are commonly published as git submodules.

**Datum current coverage.** Implicit. The format design supports
git; no explicit conventions are documented.

**Strategic recommendation.** **Add a "Datum Project Repository
Conventions" appendix to `docs/POOL_ARCHITECTURE.md` or
`specs/NATIVE_FORMAT_SPEC.md`** covering: recommended
`.gitignore`, `.gitattributes` (binary-pattern declarations),
LFS pointers for `models/`, submodule conventions for shared
pools. Effort: ~1 day. Big quality-of-life win for
serious users.

### Emerging / Note Only

#### OpenSCAD-style programmatic 3D models

**OpenSCAD** is a script-driven CSG modeller used heavily for
3D-printable object design. **JSCAD** (formerly OpenJSCAD) is a
JavaScript variant. Both have small but vocal communities for
parametric component-body authoring.

**Adoption in ECAD.** Negligible. KiCad has experimental
plugins; no professional ECAD tool consumes OpenSCAD directly.

**Strategic recommendation.** **Note only.** Datum is unlikely
to grow OpenSCAD-native authoring. The path is OpenSCAD →
STL/STEP → Datum's `ModelRef.path`.

#### HDF5 / Parquet for high-speed simulation result interchange

**HDF5.** Hierarchical data format used by scientific computing.
Some commercial SI tools (Cadence Sigrity, Ansys SIwave, Mentor
HyperLynx) export simulation results to HDF5.

**Parquet.** Columnar format from the Apache Arrow ecosystem.
Used by data-science tooling.

**Adoption in ECAD.** Niche. Mostly for SI/PI engineers
exporting simulation results to Python / Pandas / Jupyter
analysis pipelines.

**Strategic recommendation.** **Note only.** If Datum ever
implements an SI/PI analysis layer (Domain 6 work, deferred
beyond M8), Parquet / HDF5 result emission would be on the
roadmap as a Python-ecosystem compatibility play. No action
in v1.

## Cross-Cutting Patterns

### Lossy vs lossless exchange

| Format | Lossy on import | Lossy on export | Notes |
|---|---|---|---|
| Gerber X2 / X3 | Yes — no netlist | Mild — drops paste/silk linkage to BOM | Image format |
| Excellon / XNC | Yes — no plating context | None | Drill only |
| ODB++ | None for fab data; lossy for ECAD intent | None | Stack-up materials needed |
| IPC-2581 Rev C | None for fab + assembly | None | The "lossless" exchange story |
| IDF 3.0 | Yes — components → extruded prisms | Yes — same | Body geometry collapse is universal |
| IDF 4.0 | Mild | Mild | Unused in practice |
| STEP AP203 | None for geometry | None for geometry | No PMI |
| STEP AP242 | None | None | The only "MCAD-lossless" path |
| IDX | None — delta is exact | None | Delta-only |
| EDMD | None | None | Full state |
| KiCad write-back | Mild — formatting/ordering | Mild — formatting/ordering | Datum's existing target, see `specs/IMPORT_SPEC.md:215-227` |
| EDIF 2 0 0 | Heavy | Heavy | Schematic only; no graphical/visual fidelity |
| DXF | Mild — line/arc only | Mild | 2D mechanical |
| PnP CSV | None for placement | None | Format-trivially-exact |

**Datum's canonical IR + sidecar persistence model puts it
ahead of the pack on round-trip fidelity.** UUID-keyed objects
(`docs/CANONICAL_IR.md` §Identity) survive write-modify-write
cycles without identity drift. The `.ids/` sidecar
(`specs/NATIVE_FORMAT_SPEC.md:390-398`) preserves source-format
identity bridges through native conversion. **The KiCad ecosystem
loses identity on rename; Datum does not.** This is a marketable
differentiator for IDX-style ECO loops.

### Vendor-neutrality and license situation

| Format | Vendor | Open spec | Royalty | Open-source impl |
|---|---|---|---|---|
| Gerber X2/X3 | Ucamco | Yes (free PDF) | None | Yes |
| Excellon / XNC | Public domain | Yes | None | Yes |
| ODB++ v8.1 | Siemens | Yes (free w/ form) | None on use | Yes (KiCad) |
| IPC-2581 Rev C | IPC consortium | Yes (free PDF) | None | Yes (KiCad, Cadence) |
| IDF 3.0 | Mentor (de-facto PD) | Yes | None | Yes (KiCad, kicad-StepUp) |
| IDX | ProSTEP iViP | Yes (free) | None | No (commercial only) |
| EDMD | ProSTEP iViP | Yes (free) | None | Yes (pyEDMD) |
| STEP AP203/214/242 | ISO | Pay for ISO; model is free | None | Yes (OCCT — LGPL) |
| JT | ISO 14306 | Yes | None | Partial (Siemens free toolkit) |
| glTF 2.0 | Khronos | Yes | None | Yes (gltf-rs) |
| DXF | Autodesk | Yes (free) | None | Yes (dxf-rs) |
| DWG | Autodesk | **No** | **ODA license required** | LibreDWG (incomplete) |
| Specctra DSN/SES | Cadence | Yes (free) | None | Yes (legacy) |
| Hyperlynx HYP | Siemens | No | Restricted | No |
| EDIF 2 0 0 | IEEE/IEC | Yes | None | Yes (academic) |
| Altium .PcbDoc | Altium | **No** | Closed | No |
| OrCAD .brd | Cadence | **No** | Closed | No |
| PADS .pcb | Siemens | **No** | Closed | No |
| KiCad .kicad_pcb | KiCad project | Yes (S-expr in source) | None | Yes (Datum) |
| Eagle .lbr | Autodesk (legacy) | Yes (DTD CC BY-ND 3.0) | None | Yes (Datum) |

**Datum's strongest interop posture is the open-spec set.** Of
the formats Datum should target, only DWG has license-hostile
issues; Datum can avoid DWG without practical loss (DXF covers
real workflows). **Datum can credibly market "no closed-source
format dependency" if it skips DWG and stays on the open-spec
list.**

### Streaming / incremental exchange

**Three formats support partial / incremental updates:**
- **IDX** — explicitly designed for it. Datum's transaction
  model maps naturally.
- **IPC-2581 Rev C bidirectional DFM** — fab → designer
  annotations as a partial overlay on the original IPC-2581.
  Less mature than IDX.
- **EDMD with version metadata** — the schema supports it;
  tool support is uneven.

**ODB++ has incremental support (`incr_step` block) added in
v8.0** but it is rarely used; most fab flows treat ODB++ as
full-state delivery.

**Datum's transaction model interaction.** Datum's
Operation/OpDiff/Transaction primitives
(`specs/ENGINE_SPEC.md:684-731`) are the right substrate for any
incremental exchange. The mapping is:
- A `Transaction` ↔ an IDX proposal change-item set
- An `OpDiff` ↔ an individual change-item (move, add, delete)
- A `Variant` ↔ an IDX configuration option

**This is a Datum-native workflow opportunity.** No surveyed
open-source ECAD has this primitive cleanly available; KiCad's
`undo_stack` is internal and not exportable as a delta. Datum
could ship incremental-exchange ahead of every other open tool
if/when an actual user requests it.

### Reference implementation availability

**The single highest-leverage open-source dependency is Open
Cascade Technology (OCCT)**, LGPL 2.1, for STEP read/write.
Without OCCT, Datum either implements STEP from scratch (months
to years of work for AP242) or uses a much weaker pure-Rust
library (`step-rs`, AP203 only). The LGPL constraint is
satisfied by dynamic linking; this is the model FreeCAD,
KiCad, and Horizon use.

**Other notable open-source dependencies Datum could lean on:**
- **`dxf-rs`** (MIT) — DXF read/write.
- **`gltf-rs`** (MIT) — glTF read/write (M7+ if 3D viewer ships).
- **`zip`** crate (MIT) — IDX / EDMD container format.
- **`quick-xml`** (MIT) — XML for IPC-2581 / EDMD.
- **KiCad's ODB++ exporter** (GPL3) — *reference, not
  dependency*; license incompatibility prevents directly
  vendoring, but the source is available to study.

**Datum should NOT vendor:**
- **LibreDWG** (GPLv3) — license hostility for likely Datum
  posture.
- **Siemens Valor / ODB++ Validator** — not source-available.
- **Cadence Allegro Free Viewer** — not source-available.

## Industry / Fab Preference Matrix (2026)

### Fab house format preferences

Survey based on fab houses' published "what to send us" pages
and forum-tested user reports as of mid-2026.

| Fab | Gerber X2 | Gerber X3 | ODB++ | IPC-2581 Rev C | Notes |
|---|---|---|---|---|---|
| **JLCPCB (CN)** | Required | Accepted | Accepted (since 2024) | Not officially | Volume Asian; CSV BOM + CSV CPL preferred for assembly |
| **PCBWay (CN)** | Required | Accepted | Accepted | Accepted (rare in practice) | Asian volume; growing IPC-2581 acceptance |
| **OSH Park (US)** | Required | Accepted | No | No | Hobbyist; Gerber-only |
| **Sierra Circuits (US)** | Accepted | Preferred | **Preferred** | Preferred | Tier-1 NA; full-package preference |
| **Advanced Circuits / 4PCB (US)** | Accepted | Accepted | **Preferred** | Accepted | Tier-1 NA; ODB++ first |
| **Eurocircuits (EU)** | Accepted | Accepted | **Preferred** | Accepted | Tier-1 EU; full-package preference |
| **Multi-CB / Multi Circuit Boards (EU)** | Accepted | Accepted | Accepted | **Preferred** | Tier-1 EU; IPC-2581 leadership |
| **NCAB Group (Global)** | Accepted | Accepted | Preferred | Preferred | Multi-country distributor |
| **Würth Elektronik (EU)** | Accepted | Accepted | Preferred | Accepted | EU professional |

**Pattern:** Asian volume fabs prioritise the universal floor
(Gerber X2 + Excellon + CSV); US/EU professional fabs prefer the
rich packages (ODB++, IPC-2581). Datum's M4 manufacturing export
hits the universal floor; the ODB++ + IPC-2581 upgrades are the
Tier-1 professional unlock.

### Assembly house format preferences

| Assembly House | BOM Format | PnP Format | Notes |
|---|---|---|---|
| **JLCPCB Assembly** | Specific CSV (Comment, Designator, Footprint, LCSC) | JLCPCB CPL | Their LCSC parts catalogue; specific column order |
| **PCBWay Assembly** | Generic CSV | Generic CSV | Flexible |
| **MacroFab (US)** | XYRS file (proprietary CSV) | XYRS | Clear documentation |
| **Screaming Circuits (US)** | Generic CSV | Generic CSV | Flexible |
| **Worthington Assembly (US)** | Generic CSV | Mentor PNP or generic CSV | Tier-1 US |
| **Compass Components (US)** | Generic CSV | Generic CSV | |
| **CircuitHub (cloud)** | Generic CSV (parametric search) | Generic CSV | Cloud-flow |

**Pattern:** **Generic CSV is the universal answer.** Per-vendor
column-order presets are convenience-wins; the underlying data
is always reference / value / X / Y / rotation / layer.

### MCAD tool support matrix (read direction; per ECAD↔MCAD format)

| MCAD Tool | STEP AP203 | STEP AP242 | IDF 3.0 | IDX | DXF | DWG |
|---|---|---|---|---|---|---|
| **SolidWorks** | Yes | Yes (PMI) | CircuitWorks (paid) | CircuitWorks Enterprise | Yes | Yes (paid OEM) |
| **Inventor** | Yes | Yes | Yes | Add-on | Yes | Yes (Autodesk native) |
| **Fusion 360** | Yes | Yes | Yes | No | Yes | Yes (Autodesk native) |
| **Creo Parametric** | Yes | Yes (PMI) | Yes (Creo ECAD) | Yes (Creo ECAD) | Yes | Yes |
| **NX (Siemens)** | Yes | Yes (PMI) | Yes (incl. 4.0) | Yes (flagship) | Yes | Yes |
| **CATIA V5/V6** | Yes | Yes (PMI) | Yes | Yes (V6) | Yes | Yes |
| **FreeCAD 0.21+** | Yes (OCCT) | Partial (geometry yes, PMI no) | Plugin (kicad-StepUp) | No | Yes | LibreDWG (uneven) |
| **Onshape** | Yes | Yes | No | No | Yes | Yes (paid) |

## EDA Tool Support Matrix

Read/write status per format. **R = reads, W = writes, B =
both, — = neither.**

| Format | Altium | OrCAD/Allegro | PADS/Xpedition | KiCad 9 | Eagle/Fusion | Horizon EDA | LibrePCB | DipTrace | EasyEDA | **Datum (current)** |
|---|---|---|---|---|---|---|---|---|---|---|
| Gerber X2 | B | B | B | B | B | B | B | B | B | **W (M4)** |
| Gerber X3 | B | B | B | R | B | — | — | B | B | **— (planned upgrade)** |
| Excellon 2 | B | B | B | B | B | B | B | B | B | **W (M4)** |
| XNC | B | B | B | B | B | — | — | — | B | **— (planned)** |
| Pick-and-Place CSV | B | B | B | B | B | B | B | B | B | **W (M4)** |
| ODB++ v8.1 | W | B | B | W | W (Fusion) | — | — | W | W | **— (planned post-M7)** |
| IPC-2581 Rev C | B | B | W | W | — | — | — | — | — | **— (planned post-M7)** |
| IPC-D-356A | W | W | W | W | W | — | — | W | W | **W (M4)** |
| STEP AP203 | W | W | W | W | W | W | — | W | W | **— (planned post-M7)** |
| STEP AP242 | — | — | W | Partial | — | Partial | — | — | — | **— (planned M8+)** |
| IDF 3.0 | W | W | W | W | W | — | — | W | — | **— (planned post-M7)** |
| IDF 4.0 | — | — | W | — | — | — | — | — | — | **— (skip)** |
| IDX | B | B | B | — | — | — | — | — | — | **— (on-demand)** |
| EDMD | B | B | B | — | — | — | — | — | — | **— (with IDX or skip)** |
| DXF (board outline import) | R | R | R | R | R | — | — | R | R | **— (planned)** |
| DXF (mechanical layer write) | W | W | W | W | W | — | — | W | W | **— (planned)** |
| DWG | R (paid) | R | R | — | — | — | — | — | — | **— (skip)** |
| JT | — | — | W (NX-bridge) | — | — | — | — | — | — | **— (skip)** |
| glTF 2.0 | — | — | — | Plugin | — | — | — | — | — | **— (M7 GUI dependent)** |
| EDIF 2 0 0 | W (legacy) | W (legacy) | W (legacy) | — | — | — | — | — | — | **— (skip)** |
| Specctra DSN/SES | B (legacy) | B (legacy) | B (legacy) | — (PNS replaced it) | — | — | — | — | — | **— (skip)** |
| Hyperlynx HYP | W | W | W | — | — | — | — | — | — | **— (skip)** |
| KiCad native | W (read) | — | — | B | — | R | — | R | R | **B (M3 import + M4 write-back)** |
| Eagle native | W (read) | — | — | R | B | R | — | R | R | **R (M3 import)** |
| Altium native | B | R (limited) | R (limited) | R (incomplete) | — | — | — | R (incomplete) | — | **— (R1 research only per `docs/COMMERCIAL_INTEROP_STRATEGY.md`)** |
| OrCAD native | R (limited) | B | R (limited) | — | — | — | — | — | — | **— (R1 research only)** |
| PADS native | R (limited) | R (limited) | B | — | — | — | — | — | — | **— (R1 research only)** |

**Key observation: Datum currently meets the universal Gerber/
Excellon/PnP floor; the credibility gap is ODB++ + IPC-2581 +
STEP/IDF.**

## Pending Exclusions (re-affirmed)

The audit's advisory exclusion list for Domain 1 is preserved.
Each item is re-affirmed here with cross-cutting-value notes.

- **Specctra DSN/SES** — legacy. **Cross-cutting note:** Specctra
  DSN remains the input format for the FreeRouting open-source
  autorouter, which some KiCad users invoke as a workflow. If
  Datum ever wants FreeRouting integration, Specctra DSN export
  would be required. **Recommendation:** Re-evaluate **only if**
  FreeRouting integration becomes a goal. Datum's M5 routing
  kernel makes this unlikely.
- **Hyperlynx HYP** — single-vendor SI-extraction format. **No
  cross-cutting value found.** Strict skip.
- **Altium / OrCAD / PADS commercial native binary formats** —
  out of v1 per `docs/COMMERCIAL_INTEROP_STRATEGY.md` and
  `specs/PROGRAM_SPEC.md`. **Cross-cutting note:** As discussed
  in §EDA Tool Support Matrix, the practical Altium → Datum
  migration path is **IPC-2581 Rev C** rather than direct
  `.PcbDoc` parsing. This significantly reduces the strategic
  cost of formally excluding Altium native parsing in v1; the
  IPC-2581 path is open via Altium's existing exporter.
- **Windchill / Teamcenter / Aras / Arena** — PLM (Domain 7).
  **Cross-cutting note:** None for Domain 1; correctly belongs
  to Domain 7.

**No items in the exclusion list turned out to have hidden
cross-cutting value sufficient to re-queue them for Phase 2
deep-dive.** Specctra has a thin connection through a possible
future FreeRouting integration but Datum's own routing kernel
makes that unlikely. The exclusions hold.

## User Pain Points & Wishlist Items

Distilled from KiCad forum, EEVblog forum, Altium Forum,
Hackaday, Reddit r/PrintedCircuitBoard, Cadence Community.

- **"I exported STEP from KiCad and SolidWorks won't open it."**
  Recurring; usually a unit-mismatch (KiCad emits mm, SolidWorks
  CircuitWorks expects mm but often mis-detects), or a malformed
  AP203 header. Lesson for Datum: **always emit the unit
  declaration in the STEP header explicitly**, never rely on
  defaults.
- **"Why does my IDF file lose component heights?"** Common with
  KiCad. The 3D-model-to-IDF-height inference is brittle when
  models are not present or are placeholders. Lesson: **Datum
  should require an explicit `body_height_nm` on Package, with
  a UI that prompts when missing**.
- **"My fab can't read my IPC-2581."** Usually Rev mismatch (the
  fab's CAM tool is on Rev B, designer emitted Rev C). Or
  schema-validation failures. Lesson: **emit Rev B + Rev C
  side-by-side, validate with the consortium's published XSD,
  document which Rev was tested at which fab**.
- **"Can my MCAD see my zone fills?"** No surveyed format
  cleanly carries zone fills — STEP captures the *outline*,
  IDF captures only the body, only ODB++ and IPC-2581 carry the
  filled-polygon copper at all (and MCAD doesn't read those).
  Lesson: **out of scope for ECAD↔MCAD; teach users that copper
  visualisation is a board-side task**.
- **"How do I round-trip an Altium board?"** Altium → IPC-2581
  → KiCad / Datum is the practical path. Forum threads are full
  of users discovering that direct binary-file approaches do not
  work; the IPC-2581 path does. Lesson: **Datum's IPC-2581
  *importer* is the unstated migration story for Altium**.
- **"My PnP file confuses the assembly machine."** Usually
  origin-convention or rotation-convention mismatch (degrees vs
  tenths-of-degrees, CW vs CCW, top-vs-bottom mirror). Lesson:
  **per-vendor PnP profile system as recommended above**.
- **"DXF board outline came in flipped."** Coordinate-system
  mismatch (Y-up vs Y-down) on DXF import is the most common
  forum complaint. Lesson: **provide a Y-flip toggle on DXF
  import**.
- **"I want IDX support in KiCad."** Standing forum request;
  KiCad maintainers cite low priority because the user base is
  large-org and KiCad's user base skews hobbyist/small-org.
  **Datum could ship IDX before KiCad does** — a rare
  competitive opportunity.

## Datum EDA Implementation Strategy

### Hard Requirements (must support)

These are required for Datum to be credible as a professional
PCB tool. None are optional in the long run.

#### 1. STEP AP203 export

- **Why must:** every MCAD tool reads it; without it Datum is
  unusable in any organisation with an enclosure team.
- **Canonical IR changes:** add typed `Transform3D { translation:
  Point3D, rotation: Euler3D }` to `ModelRef`
  (`specs/ENGINE_SPEC.md:74-77`); add `Package.body_height_nm:
  Option<i64>` and `Package.body_height_mounted_nm: Option<i64>`.
- **Pool model changes:** add `models/` subdirectory convention
  to `docs/POOL_ARCHITECTURE.md`; pool index must validate
  `ModelRef.path` resolution.
- **Transaction model changes:** none (read-only export).
- **MCP API additions:** `export_step { board_uuid,
  output_path, format: "AP203" | "AP214" | "AP242",
  include_components: bool }`; `validate_step { path }`.
- **Minimum viable:** AP203 export of board outline + extruded
  copper layers (no component STEP merge). Effort: 2-3 weeks
  with OCCT FFI through `occt-rs` or thin C++ shim.
- **Full implementation:** AP203 + AP214 with per-component
  STEP body merge. Effort: 6-8 weeks total.
- **Partner / library dependencies:** **Open Cascade Technology
  (LGPL 2.1)**, dynamically linked.
- **Effort estimate (full):** 6-8 weeks engineering + 2 weeks
  golden testing.

#### 2. IDF 3.0 export

- **Why must:** the universal "Plan B when STEP doesn't work"
  format. Trivial to implement, broad reach.
- **Canonical IR changes:** uses `body_height_nm` from STEP
  work; otherwise no new types.
- **Pool model changes:** none.
- **Transaction model changes:** none.
- **MCP API additions:** `export_idf { board_uuid, output_dir,
  version: "3.0" }`.
- **Minimum viable** = full implementation. Effort: ~1 week.
- **Partner / library dependencies:** none (pure Rust).

#### 3. ODB++ v8.1 export

- **Why must:** preferred format at every Tier-1 NA/EU fab.
  Without it, Datum is "Asian-volume-fab only".
- **Canonical IR changes:** **requires Stackup material fields
  (`dk`, `df`, `copper_weight`, `roughness`) currently deferred
  per `specs/ENGINE_SPEC.md:255-256`**. Either ship ODB++ with
  empty-string materials and a documented partial-fidelity flag,
  or do the Stackup extension first.
- **Pool model changes:** none.
- **Transaction model changes:** none.
- **MCP API additions:** `export_odbpp { board_uuid,
  output_path, version: "8.1" }`; `validate_odbpp { path,
  validator: "siemens" | "internal" }`.
- **Minimum viable** (no impedance, no materials): ~4-5 weeks +
  Siemens Validator round-trip testing.
- **Full implementation:** ~8-10 weeks.
- **Partner / library dependencies:** none required.

#### 4. IPC-2581 Rev C export (cross-reference IPC research)

- **Why must:** modern fab consortium standard; unblocks
  controlled-impedance metadata for the deferred SI rule scaffold.
- **Canonical IR changes:** same Stackup extension as ODB++
  (Dk/Df/copper_weight); plus `Net.controlled_impedance:
  Option<ImpedanceSpec>` field.
- **Pool model changes:** none directly; but the netlist→XML
  binding work needs stable net-name strings (already covered).
- **Transaction model changes:** none.
- **MCP API additions:** `export_ipc2581 { board_uuid,
  output_path, revision: "B" | "C" }`; `validate_ipc2581 {
  path, schema_version: "B" | "C" }`.
- **Minimum viable:** Rev B without impedance schema. ~5 weeks.
- **Full implementation:** Rev C with controlled-impedance
  schema. ~8 weeks.
- **Partner / library dependencies:** XSD freely from
  `ipc-2581.com`.

#### 5. Gerber X3 attribute upgrade

- **Why must:** marginal upgrade from existing X2; gives Datum
  free assembly-attribute compatibility.
- **Canonical IR changes:** none (X3 attributes are derivable
  from existing PlacedPackage / Net / Pad data).
- **Pool model changes:** none.
- **Transaction model changes:** none.
- **MCP API additions:** add `gerber_x3: bool` flag to
  `export_gerber { ... }` (or upgrade default to X3 unconditionally).
- **Effort:** ~3 days within existing Gerber export.

#### 6. XNC drill output (alongside or replacing classic Excellon)

- **Why must:** XNC is the modern Gerber-aligned drill format;
  Ucamco recommends it; eliminates per-vendor unit/precision
  footguns.
- **Effort:** ~3 days. Mostly a coordinate-system
  re-serialization.

### Should Support (post-M7)

These materially expand Datum's reach but are one rung below
"required for professional credibility".

#### 7. STEP AP242 export (upgrade from AP203)

- **Why should:** modern unified MCAD exchange standard;
  required for advanced PMI / configuration management.
- **Canonical IR changes:** AP242 PMI export needs courtyard /
  keepout zones marked semantically. The IR has the geometry
  (`Package.courtyard`, `Keepout`); the semantic-PMI mapping is
  the work.
- **Effort:** +4-6 weeks on top of AP203 baseline.
- **Library dependencies:** OCCT (already in for AP203 baseline).

#### 8. DXF read/write

- **Why should:** mechanical board outline import is the most
  common manual workflow step today.
- **Canonical IR changes:** none.
- **Effort:** ~1 week using `dxf-rs`.
- **MCP API additions:** `import_dxf_outline { path,
  unit: "mm" | "inch", flip_y: bool }`; `export_dxf_layer {
  layer_id, output_path, unit: "mm" | "inch" }`.

#### 9. Per-vendor PnP profile system

- **Why should:** assembly-house onboarding friction is real;
  per-vendor presets (JLCPCB CPL, MacroFab XYRS, generic) are
  cheap quality-of-life.
- **Effort:** ~2 days per profile; suggest landing 4 profiles
  (generic, JLCPCB, PCBWay, MacroFab).

#### 10. Project repository conventions appendix

- **Why should:** Datum's deterministic JSON serialization is a
  git-native superpower; documenting the conventions multiplies
  the benefit.
- **Effort:** ~1 day documentation.

### On-Demand Only

These are high-value when needed but should not be built
speculatively.

#### 11. IDX (ProSTEP iViP Incremental Design Exchange)

- **Why on-demand:** high-leverage in automotive / aerospace /
  industrial verticals; near-zero value elsewhere.
- **Datum differentiator:** the transaction model maps cleanly.
  No other open-source ECAD has this primitive ready.
- **Effort when triggered:** ~6-8 weeks for full bidirectional.

#### 12. EDMD (paired with IDX or skip)

- Standalone EDMD is mostly useless without IDX. Implement only
  alongside IDX.

#### 13. Vendor netlist exports (PADS .asc, OrCAD .net)

- **Why on-demand:** part of the larger commercial-interop
  staging in `docs/COMMERCIAL_INTEROP_STRATEGY.md`. Cheap when a
  specific migration target needs it.

### Out of Scope (recommend formal exclusion)

These should be marked **explicitly** out of scope in
`docs/INTEROP_SCOPE.md` so users do not have to guess.

- **DWG (Autodesk binary)** — license-hostile for non-Autodesk
  implementations; ODA Teigha is paid (~$15K/yr corporate);
  LibreDWG is GPLv3 and incomplete. **DXF covers the same use
  case.**
- **JT (ISO 14306 Siemens lightweight 3D)** — one path removed
  from Datum's domain (ECAD → STEP → MCAD → JT). Not a direct
  ECAD↔MCAD format.
- **EDIF 2 0 0** — effectively dead in practice; no migration
  value.
- **IDF 4.0** — never gained tool adoption; IDF 3.0 covers the
  use case.
- **OBJ / STL** — hobbyist; not relevant for ECAD↔MCAD
  professional exchange.
- **Specctra DSN/SES** — legacy routing exchange; only relevant
  if FreeRouting integration becomes a goal (Datum's own routing
  kernel makes this unlikely).
- **Hyperlynx HYP** — single-vendor SI extraction; no open
  exchange value.
- **OpenSCAD / JSCAD** — programmatic 3D authoring; tangential.
- **HDF5 / Parquet** — SI/PI simulation result interchange; not
  relevant unless Datum implements its own SI/PI layer (M8+).

### Datum Differentiators

Where Datum's pool + transaction + AI surfaces can do better
than incumbents in the data-exchange space:

1. **UUID-stable identity through round-trips.** KiCad
   regenerates UUIDs on some edits; Altium uses internal
   integer IDs. Datum's UUID v5 import sidecar
   (`specs/IMPORT_SPEC.md:60-107`) preserves identity through
   write-modify-write cycles, which is exactly what IDX needs.
2. **Transaction-as-IDX-proposal mapping.** Datum's
   Operation/OpDiff/Transaction model can naturally emit IDX
   change items as transactions commit. No surveyed open-source
   ECAD has this primitive ready.
3. **Pool layering for shared library distribution.**
   `docs/POOL_ARCHITECTURE.md`'s priority-ordered pools are the
   right shape for component vendor library distribution
   (SnapEDA pool, UltraLibrarian pool, organisation-internal
   pool, project-local pool — layered with conflict resolution).
   No other tool exposes this concept directly.
4. **AI-assisted format selection.** "Tell me which export
   format my fab actually needs" is a natural MCP tool — the AI
   layer can read the fab's order page, match against Datum's
   format support, and recommend the right deliverable bundle.
   Cadence / Altium have no equivalent.
5. **Deterministic JSON for git native ECAD.** Datum's byte-
   stable serialization (`docs/CANONICAL_IR.md` §Serialization;
   `specs/NATIVE_FORMAT_SPEC.md:402-419`) makes diff-friendly
   git workflows real. KiCad's S-expression files are
   technically text but reorder objects on save; Altium /
   OrCAD / PADS are binary.
6. **MCP API parity for every export format.** Every export
   should be an MCP tool, not just a CLI command. AI agents can
   then automate "validate the design, export the right
   bundle, attach to a fab quote request" workflows. This is the
   AI-native positioning made concrete.

### Recommended Spec Edits

Concrete spec/doc edits the project owner should review before
applying. **Eight items**, listed in suggested apply order.

#### Edit 1 — Expand and re-organise `docs/INTEROP_SCOPE.md`'s "Future M5+" export list

**Current** (`docs/INTEROP_SCOPE.md:88-92`):
```text
### Future (M5+)
- ODB++
- STEP (3D)
- PDF (documentation)
- IPC-2581
```

**Recommended replacement** (and rename section to "Future
export targets — research-staged"):
```text
### Future export targets (research-staged)

Categorised by the Phase 2 Domain 1 deep-dive
(`research/data-exchange-interop/`).

#### Hard requirements (post-M7):
- IDF 3.0 (board+component-outline ECAD↔MCAD floor;
  pure-Rust)
- STEP AP203 (3D ECAD↔MCAD; OCCT LGPL2.1)
- ODB++ v8.1 (Tier-1 fab preferred; pure-Rust)
- IPC-2581 Rev C (consortium DPMX; XSD-driven; pure-Rust)
- Gerber X3 (marginal upgrade from existing X2)
- XNC (modern Gerber-aligned drill format)

#### Should support (post-M7):
- STEP AP242 (PMI semantic export upgrade on AP203)
- DXF (board outline import + mechanical layer write)
- Per-vendor PnP profile system

#### On-demand only:
- IDX / EDMD (ECAD↔MCAD ECO loop; high value in
  automotive/aerospace)
- Vendor netlists (PADS .asc, OrCAD .net)

#### Explicitly out of scope:
- DWG (license-hostile; DXF covers the use case)
- JT (ISO 14306 Siemens lightweight 3D)
- EDIF 2 0 0 (effectively dead)
- IDF 4.0 (never adopted)
- Specctra DSN/SES, Hyperlynx HYP
- OBJ / STL / OpenSCAD / JSCAD (hobbyist 3D)
- HDF5 / Parquet (SI/PI; if Datum ever ships an SI layer)
```

**Rationale:** the current four-item lump treats ODB++ /
STEP / PDF / IPC-2581 as a single block; they have very
different effort, partner, and license pictures. Surfacing the
out-of-scope list also helps target-market users plan.

**Effort to apply:** 30 min documentation work.

#### Edit 2 — Extend `Package` and `ModelRef` in `specs/ENGINE_SPEC.md`

**Current** (`specs/ENGINE_SPEC.md:74-77,132-140`):
```rust
pub struct ModelRef {
    pub path: String,
    pub transform: Option<serde_json::Value>,  // exact 3D transform deferred
}

pub struct Package {
    pub uuid: Uuid,
    pub name: String,
    pub pads: HashMap<Uuid, Pad>,
    pub courtyard: Polygon,
    pub silkscreen: Vec<Primitive>,
    pub models_3d: Vec<ModelRef>,
    pub tags: HashSet<String>,
}
```

**Recommended replacement:**
```rust
pub enum ModelFormat {
    Step,           // .step / .stp
    Wrl,            // VRML — KiCad legacy
    Iges,           // .igs / .iges — legacy
    Obj,            // Wavefront — hobbyist
    Gltf,           // glTF 2.0 — web/M7 GUI
}

pub struct Transform3D {
    pub translation_nm: Point3D,
    pub rotation_tenths_deg: Euler3D,
    pub scale: f32,        // 1.0 default
}

pub struct ModelRef {
    pub path: String,
    pub format: ModelFormat,
    pub transform: Transform3D,
}

pub struct Package {
    pub uuid: Uuid,
    pub name: String,
    pub pads: HashMap<Uuid, Pad>,
    pub courtyard: Polygon,
    pub silkscreen: Vec<Primitive>,
    pub models_3d: Vec<ModelRef>,
    pub body_height_nm: Option<i64>,           // for IDF 3.0 export
    pub body_height_mounted_nm: Option<i64>,   // tall-component height
    pub tags: HashSet<String>,
}
```

**Rationale:** unblocks both STEP and IDF 3.0 export. Adds
required body-height fields that no current import format
provides automatically.

**Backward compatibility:** existing imports populate
`body_height_nm = None`; downstream tools that need it (IDF
exporter) report `Lossy: missing body height` and emit a
default 0nm with a warning.

**Effort to apply:** 2-3 hours engineering for the type
extension; downstream import code (KiCad / Eagle) needs to
populate the new fields where source data is available
(KiCad 8+ has body height in `.kicad_mod`; Eagle has it in
`<package3d>` blocks).

#### Edit 3 — Extend `StackupLayer` in `specs/ENGINE_SPEC.md`

**Current** (`specs/ENGINE_SPEC.md:250-256`):
```rust
pub struct StackupLayer {
    pub id: LayerId,
    pub name: String,
    pub layer_type: StackupLayerType,  // Copper, Dielectric
    pub thickness_nm: i64,
    // M8+: dk, df, copper_weight, roughness
}
```

**Recommended replacement (move M8+ comment fields into
real schema, gate at MCP/CLI level if not authored):**
```rust
pub struct StackupLayer {
    pub id: LayerId,
    pub name: String,
    pub layer_type: StackupLayerType,
    pub thickness_nm: i64,
    pub dielectric_constant: Option<f32>,    // Dk, dimensionless
    pub loss_tangent: Option<f32>,           // Df, dimensionless
    pub copper_weight_oz: Option<f32>,       // 0.5, 1.0, 2.0 typical
    pub roughness_um: Option<f32>,           // Rrms surface roughness
    pub material_name: Option<String>,       // FR-4, Rogers 4350B, etc.
}
```

**Rationale:** ODB++ and IPC-2581 Rev C both require these for
non-trivial output. Bringing the M8+ comment fields forward
into actual schema with `Option<>` wrapping unblocks export
without forcing every authored project to populate them.

**Effort to apply:** 1-2 hours type extension; KiCad import
already has fallback handling, can populate from
`stackup-material` block in `.kicad_pro` v8+.

#### Edit 4 — Extend `Net` for controlled-impedance metadata

**Add to `specs/ENGINE_SPEC.md` Board Types section** (after
the existing `Net` definition):
```rust
pub struct ImpedanceSpec {
    pub target_ohms: f32,           // 50, 90 (USB), 100 (LVDS), etc.
    pub tolerance_pct: f32,         // ±10 typical
    pub controlled_dielectric: Option<Uuid>,  // → StackupLayer
}

// Field added to Net:
pub struct Net {
    pub uuid: Uuid,
    pub name: String,
    pub net_class: Option<Uuid>,
    pub controlled_impedance: Option<ImpedanceSpec>,  // ← new field
}
```

**Rationale:** IPC-2581 Rev C's `<ImpedancesProperties>`
schema and ODB++'s impedance-control attributes both need
per-net target impedance + tolerance. This is also the field
the deferred M5+ `Impedance` rule type
(`specs/ENGINE_SPEC.md:586-594`) will consume.

**Effort to apply:** 1-2 hours type extension; no immediate
import-side population needed (KiCad does not carry per-net
impedance until 9.x; Datum can author it directly).

#### Edit 5 — Add MCP export tool surface

**Add to `specs/MCP_API_SPEC.md` "M2 Tools (v1)" → Export
section** (or create a new "M7+ Export Tools" section):

```text
#### `export_step`
Export board + components to STEP file for MCAD round-trip.

Parameters:
- board_uuid: Uuid
- output_path: String (absolute or project-relative)
- format: "AP203" | "AP214" | "AP242" (default AP203)
- include_components: bool (default true)
- coordinate_origin: "board_corner" | "board_center"

Returns:
- export_status: "success" | "partial" | "failed"
- warnings: array of partial-fidelity messages

#### `export_idf`
Export board to IDF 3.0 for MCAD compatibility floor.

Parameters:
- board_uuid: Uuid
- output_dir: String (writes board.emn + board.emp)
- version: "3.0"

#### `export_odbpp`
Export board to ODB++ v8.1 archive.

Parameters:
- board_uuid: Uuid
- output_path: String (folder; tarball wrapper optional)
- version: "8.1"
- include_assembly: bool (default true)

#### `export_ipc2581`
Export board to IPC-2581 archive.

Parameters:
- board_uuid: Uuid
- output_path: String (single XML file)
- revision: "B" | "C" (default C)
- include_impedance: bool (default true if revision = "C")

#### `import_dxf_outline`
Import board outline polygon from DXF.

Parameters:
- path: String
- unit: "mm" | "inch"
- flip_y: bool (workaround for Y-down DXF sources)

Returns:
- outline: Polygon (in canonical nm)
```

**Rationale:** AI-native positioning requires every export
format to be an MCP tool, not a CLI-only command. This is
also the natural place for the recommended "AI assistant
recommends format for fab" workflow.

**Effort to apply:** schema definition + handler stubs ~1 day;
real implementations land per the export-format work.

#### Edit 6 — Document `.gitignore` / `.gitattributes` conventions

**Add appendix to `specs/NATIVE_FORMAT_SPEC.md`** (new
§12 "Repository Conventions") or to
`docs/POOL_ARCHITECTURE.md`:

```text
## 12. Repository Conventions (Recommended)

Datum projects are designed for git-native workflows. The
following conventions are recommended for project repositories:

### .gitignore

# Derived data — recomputed by engine
*.fill.cache
*.routability.cache
**/board/*.derived/

# Local build artifacts
build/
dist/
manufacturing-output/

### .gitattributes

# Treat native JSON as text for diffs
*.json text

# Treat 3D models as binary; store via git-lfs
*.step binary filter=lfs diff=lfs merge=lfs -text
*.stp binary filter=lfs diff=lfs merge=lfs -text
*.wrl binary filter=lfs diff=lfs merge=lfs -text
*.iges binary filter=lfs diff=lfs merge=lfs -text

# Treat fabrication output as binary; do not store in repo
*.gbr binary
*.drl binary
*.odb.tgz binary

### Pool sharing

Shared pools should be published as git submodules referenced
from `project.json`'s `pools` array. The submodule provides
versioning and pin-to-revision semantics.
```

**Rationale:** Datum's deterministic serialization is a
git-native superpower; documenting the conventions multiplies
the benefit. The addition is non-normative but important
for adoption.

**Effort to apply:** ~1 hour documentation.

#### Edit 7 — Augment `specs/IMPORT_SPEC.md` with IPC-2581 import target

**Add to `specs/IMPORT_SPEC.md`** after the Eagle Import
Feature Matrix (after line 211):

```text
## 5. IPC-2581 Import (Future — Post-M7)

Target: IPC-2581 Rev B and Rev C archives.

### Why imported as a first-class format

IPC-2581 import is the **practical Altium / OrCAD / PADS
migration path** that does not require reverse-engineering
commercial native binary formats. Every Tier-1 commercial EDA
tool can export IPC-2581 Rev C. Datum's IPC-2581 importer
serves as the receiving end of that migration without taking on
binary-format reverse-engineering risk.

### Feature Matrix (target)

| Feature | Required | Notes |
|---|---|---|
| Board outline | Required | |
| Stackup (with materials) | Required | Maps to extended `StackupLayer` fields |
| Components | Required | Placement, rotation, layer |
| Tracks (per-layer) | Required | |
| Vias | Required | Including blind/buried |
| Zones (filled polygons) | Required | |
| Net list | Required | With component-pin connectivity |
| Net classes | Required | |
| Controlled impedance | Required | Maps to `Net.controlled_impedance` |
| Diff pairs | Required | Maps to NetClass `diffpair_width`/`diffpair_gap` |
| BOM | Best-effort | Imported as Part metadata enrichment |
| Pad-stack data | Required | Maps to Datum's Padstack pool entries |
| Drill data | Required | |
| Fabrication / assembly notes | Best-effort | Stored as project-level metadata |
| Embedded components | Deferred | M8+ |
| Cavities | Deferred | M8+ |
| DFM annotations | Deferred | One-way fab → designer |
```

**Rationale:** documents that IPC-2581 import is the practical
Altium/OrCAD/PADS migration path, parallel to (and in some
ways superior to) direct binary parsing.

**Effort to apply:** ~30 min documentation.

#### Edit 8 — Add "Datum's Open-Stack Position" appendix to `docs/COMMERCIAL_INTEROP_STRATEGY.md`

**Add new section to
`docs/COMMERCIAL_INTEROP_STRATEGY.md` § 10 "Datum's Open-
Stack Position":**

```text
## 10. Datum's Open-Stack Position

Of the data-exchange formats Datum should target, **only DWG
has license-hostile issues**. Skipping DWG (DXF covers the use
case) leaves Datum on a fully open-spec stack:

- Gerber X2/X3 (Ucamco, free PDF)
- Excellon / XNC (public domain)
- ODB++ v8.1 (Siemens, free spec)
- IPC-2581 Rev C (IPC consortium, free PDF)
- IDF 3.0 (Mentor de-facto public domain)
- STEP AP203 / AP242 (ISO; OCCT LGPL2.1)
- IDX / EDMD (ProSTEP iViP, free)
- DXF (Autodesk, free spec)

**Datum can credibly market "no closed-source format
dependency" against Altium / Cadence / Mentor.** This is a
genuine differentiator at the foundation level.

The practical commercial-tool migration path leverages this:
**Altium → IPC-2581 Rev C → Datum** uses Altium's own export
without touching native `.PcbDoc` binary parsing. Same applies
to OrCAD and PADS.

This significantly reduces the strategic urgency to ever
implement direct commercial native binary parsers — the
intermediate-format path covers most user-migration scenarios
without the engineering / licensing risk.
```

**Rationale:** captures the "open-stack" marketing position as
a documented strategic stance, and explicitly grounds the
decision to **not** pursue direct binary parsing in the fact
that an open intermediate-format path exists.

**Effort to apply:** ~30 min documentation.

## Sources

### Primary specifications

- [STEP AP242 ISO 10303-242:2020](https://www.iso.org/standard/66654.html) — current ISO standard (paid)
- [STEP AP203 ISO 10303-203:2011](https://www.iso.org/standard/44308.html) — superseded but still active
- [ProSTEP iViP CAx Implementor Forum](https://www.prostep.org/en/cax-if/) — IDX, EDMD, ECAD/MDA recommended practices
- [ODB++ Solutions Alliance](https://www.odb-sa.com/) — ODB++ v8.1 spec download portal
- [IPC-2581 Consortium](https://www.ipc-2581.com/) — IPC-2581 Rev C spec, sample data, viewer pointers
- [Ucamco Gerber Format](https://www.ucamco.com/en/gerber) — Gerber X2/X3 reference
- [Ucamco XNC reference](https://www.ucamco.com/files/downloads/file_en/456/excellon-format-eqv-xnc_en.pdf) — NC drill in Gerber syntax
- [Khronos glTF 2.0 spec](https://registry.khronos.org/glTF/specs/2.0/glTF-2.0.html) — current spec
- [Autodesk DXF Reference](https://help.autodesk.com/view/OARX/2026/ENU/?guid=GUID-235B22E0-A567-4CF6-92D3-38A2306D73F3) — DXF format reference

### Reference implementations

- [Open Cascade Technology (OCCT)](https://dev.opencascade.org/) — LGPL 2.1 STEP/IGES library
- [STEPcode](https://github.com/stepcode/stepcode) — BSD-licensed C++ STEP library
- [kicad-StepUp](https://github.com/easyw/kicadStepUpMod) — FreeCAD plugin, IDF and STEP bridge
- [dxf-rs](https://crates.io/crates/dxf) — pure-Rust DXF
- [gltf-rs](https://crates.io/crates/gltf) — pure-Rust glTF 2.0
- [pyEDMD](https://github.com/proSTEP-iViP/pyEDMD) — Python EDMD schema bind-up
- [pcb-tools-odb-plus](https://github.com/curtacircuitos/pcb-tools-extension) — Python ODB++ partial reader
- [LibreDWG](https://www.gnu.org/software/libredwg/) — GPLv3 DWG; quality uneven

### Industry / fab references

- [JLCPCB capabilities](https://jlcpcb.com/capabilities/Capabilities) — what JLCPCB accepts
- [PCBWay PCB capabilities](https://www.pcbway.com/capabilities.html) — PCBWay accepted formats
- [Sierra Circuits IPC-2581 viewer + workflow](https://www.protoexpress.com/tools/ipc-2581-viewer/) — IPC-2581 viewer
- [Eurocircuits ODB++ acceptance](https://www.eurocircuits.com/blog/odb-the-pcb-data-format-of-choice/) — Tier-1 EU fab perspective
- [Multi-CB IPC-2581 page](https://www.multi-circuit-boards.eu/en/support/pcb-data/ipc-2581.html) — Tier-1 EU fab IPC-2581 stance
- [Advanced Circuits PCB design files](https://www.4pcb.com/free-pcb-file-check.html) — accepted formats survey
- [MacroFab XYRS file format](https://help.macrofab.com/knowledge/the-xyrs-file) — proprietary CSV variant
- [Screaming Circuits PnP requirements](https://blog.screamingcircuits.com/2019/01/the-cpl-or-pnp-file.html) — PnP format guidance

### Tool documentation

- [Altium MCAD Co-Designer](https://www.altium.com/documentation/altium-designer/co-designer-multi-board-mcad) — Altium IDX/IDF/STEP exchange
- [Cadence Mechanical Co-Designer](https://www.cadence.com/en_US/home/tools/system-analysis/mcad-collaboration/mechanical-co-designer.html) — Allegro IDX flow
- [Siemens NX MCAD ECAD Collaboration](https://blogs.sw.siemens.com/ee-systems/) — NX IDX/EDMD flagship
- [KiCad IPC-2581 export announcement](https://www.kicad.org/blog/2023/06/Version-7.0.6-Released/) — KiCad 7.0.6 added IPC-2581
- [KiCad ODB++ export](https://www.kicad.org/blog/2024/02/KiCad-8.0.0-Release/) — KiCad 8.0 added ODB++
- [KiCad STEP export](https://docs.kicad.org/master/en/cli/cli.html#kicad-cli-pcb-export-step) — kicad-cli STEP export reference
- [SolidWorks CircuitWorks](https://help.solidworks.com/2024/english/SolidWorks/circuitworks/c_intro_circuitworks.htm) — IDF / STEP / EDMD intake
- [Fusion 360 ECAD-MCAD](https://www.autodesk.com/products/fusion-360/blog/ecad-mcad-collaboration/) — Fusion 360 STEP/IDF flow
- [FreeCAD KicadStepUp WB](https://wiki.freecad.org/KicadStepUp_Workbench) — FreeCAD ECAD↔MCAD workbench

### Forum / industry discussion

- [EEVblog forum: ODB++ vs IPC-2581 vs Gerber](https://www.eevblog.com/forum/manufacture/) — ongoing fab-format discussion threads
- [KiCad forum: STEP export issues](https://forum.kicad.info/t/topics-tagged/step) — STEP export common-pitfalls threads
- [KiCad forum: IDX support requests](https://gitlab.com/kicad/code/kicad/-/issues?label_name%5B%5D=Feature) — IDX feature requests
- [Altium Forum: IDX vs IDF](https://forum.live.altium.com/) — IDX/IDF discussion
- [Reddit r/PrintedCircuitBoard fab-format threads](https://www.reddit.com/r/PrintedCircuitBoard/) — community fab-format complaints
- [Cadence Community IPC-2581 vs ODB++](https://community.cadence.com/cadence_blogs_8/) — Cadence-side discussion
- [Hackaday Datum/KiCad ODB++ coverage](https://hackaday.com/) — KiCad 8 ODB++ release coverage

### Cross-references (Datum-internal)

- `research/standards-audit/STANDARDS_AUDIT.md` § 1 — Phase 1 inventory of Domain 1
- `research/ipc-compliance/IPC_COMPLIANCE_RESEARCH.md` §IPC-2581 (lines 625-690) — IPC-2581 deep treatment
- `research/ipc-compliance/IPC_COMPLIANCE_RESEARCH.md` §IPC-D-356 (lines 692-722) — IPC-D-356 deep treatment
- `research/ipc-compliance/IPC_COMPLIANCE_RESEARCH.md` §IPC-1752 (lines 744-764) — materials declaration overlap
- `docs/INTEROP_SCOPE.md` — current Datum interop scope
- `docs/COMMERCIAL_INTEROP_STRATEGY.md` — Altium/OrCAD/PADS staged strategy
- `specs/IMPORT_SPEC.md` — current import contracts
- `specs/NATIVE_FORMAT_SPEC.md` — native format
- `specs/ENGINE_SPEC.md` — canonical types referenced for IR-extension recommendations
- `specs/MCP_API_SPEC.md` — MCP API surface for new export tools
