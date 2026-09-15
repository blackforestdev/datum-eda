# Datum EDA — Standards & Compliance Audit (Phase 1 Inventory)

> Phase 1 audit landscape inventory. Maps the 8 standards/compliance
> domains identified on 2026-04-17 against Datum EDA's current spec,
> classifying each standard as Covered / Partial / Blind Spot / Out of
> Scope / Already-researched. Phase 2 deep-dive research is triaged
> from this document; do not perform deep dives until the project owner
> reviews and prioritises.

## Method

This audit was performed by reading Datum's current controlling specs
(`specs/PROGRAM_SPEC.md`, `specs/INTEGRATED_PROGRAM_SPEC.md`,
`specs/ENGINE_SPEC.md`, `specs/NATIVE_FORMAT_SPEC.md`,
`specs/IMPORT_SPEC.md`, `specs/MCP_API_SPEC.md`,
`specs/M7_FRONTEND_SPEC.md`, `specs/SCHEMATIC_EDITOR_SPEC.md`,
`specs/SCHEMATIC_CONNECTIVITY_SPEC.md`,
`specs/CHECKING_ARCHITECTURE_SPEC.md`, `specs/ERC_SPEC.md`),
the supporting design rationale under `docs/`
(`CANONICAL_IR.md`, `POOL_ARCHITECTURE.md`, `LIBRARY_ARCHITECTURE.md`,
`MCP_DESIGN.md`, `CHECKING_ARCHITECTURE.md`, `AUTHORING_TOOLS.md`,
`ENGINE_DESIGN.md`, `NATIVE_FORMAT.md`, `INTEROP_SCOPE.md`,
`COMMERCIAL_INTEROP_STRATEGY.md`), and `CLAUDE.md`. The pre-existing
IPC survey (`research/ipc-compliance/IPC_COMPLIANCE_RESEARCH.md`) was
cross-referenced where IPC overlaps the eight domains, but its
contents were not re-investigated. For each standard, coverage was
determined by direct citation of spec/doc lines that explicitly
address the standard's scope (Covered), citations that address part
of its scope but leave other parts unaddressed (Partial), absence of
any addressing language (Blind Spot), or explicit exclusion language
(Out of Scope). Deep-dive analysis of any one standard was
intentionally avoided; this is an inventory, not a substantive review.

## Executive Summary

- Datum's spec corpus is dominated by engine-internal contracts
  (canonical IR, determinism, operations, ERC/DRC) and KiCad/Eagle
  interop. Industry compliance, signal-integrity standards, and
  professional lifecycle/PLM integration are essentially unaddressed.
- Blind-spot count by domain: (1) Data exchange & interop = 13;
  (2) Component modelling = 12; (3) Schematic & drawing conventions
  = 8 (most are Partial through KiCad-import inheritance);
  (4) Industry-vertical compliance = 13 (one Out of Scope is
  defensible v1 framing); (5) Materials & environmental = 13;
  (6) EMC & signal integrity = 13; (7) PLM & lifecycle integration
  = 12; (8) Process & quality = 12. Total: ~96 blind spots out of
  ~140 inventoried standards.
- Top 3 highest-priority Phase 2 deep-dives:
  1. **STEP AP242 / IDF / IDX / EDMD** (Domain 1) — ECAD↔MCAD
     exchange is now the dominant migration pain point above
     KiCad/Eagle compatibility; absence is a hard adoption blocker
     for any user with an enclosure team.
  2. **IBIS / Touchstone / SPICE subset** (Domain 2) — pure-MCP/AI
     differentiation falls apart without behavioural component
     models; Altium-class users will not migrate without IBIS.
  3. **IPC-2581C / ODB++ / Gerber X3 / IPC-D-356** (Domain 1, partly
     covered by IPC research) — modern fabrication interchange is
     no-longer-optional table stakes; cheap to wire to existing
     export framework.
- Top 3 lowest-priority "skip for now":
  1. **DO-254 / DO-160 / MIL-PRF-31032** (Domain 4 aerospace-defence)
     — process-grade certification, not engine work; refer users to
     external processes rather than encoding.
  2. **CMMI / ISO 9001 process-maturity assessments** (Domain 8) —
     organisational, not tool-level; Datum's audit-log story is
     enough as a substrate.
  3. **JEDEC JEP30 PIP** (Domain 2) — superseded in practice by
     manufacturer-specific datasheets and Octopart-style metadata;
     low ROI unless a major distributor demands it.
- Major surprise findings:
  - **No mention anywhere in Datum's specs of IEEE 315 / IEC 60617
    schematic graphic-symbol standards.** The Schematic Editor spec
    treats symbols as opaque graphics carrying pin semantics; the
    visual-language standard a professional user expects is wholly
    delegated to imported library content. This is a real gap for
    the native-symbol-authoring workflow.
  - **No regulatory-export controls** (ITAR/EAR markings, controlled
    data fields). Defensible v1 omission, but it should be made
    explicit because professional users in regulated industries need
    to confirm Datum will not be a leak vector.
  - **21 CFR Part 11 electronic-signature / audit-trail compliance**
    is implicitly enabled by Datum's deterministic transaction model
    but not explicitly claimed; medical-device users would need an
    explicit statement.
  - **Materials & environmental compliance (RoHS / REACH / IPC-1752A)
    has zero data-model touchpoints in Datum.** Even Part metadata
    has no compliance-attestation field. IPC research already
    flagged this; the audit confirms no other spec compensates.
  - **Variants are persisted in canonical IR (`Variant` struct in
    `specs/ENGINE_SPEC.md` line 440-444) but variant editing is
    deferred indefinitely.** This is the right bones for AS9102 /
    AS9100 First-Article-Inspection traceability later, but no spec
    mentions FAI or AS9102 as the consumer.

## Datum Spec Surface Reviewed

- `CLAUDE.md` — project framing, milestone status M0-M5 complete, M6
  active, attribution policy.
- `specs/PROGRAM_SPEC.md` — milestone exit criteria M0-M4, R1
  commercial-interop research track, user stories, scope-integrity
  terms.
- `specs/INTEGRATED_PROGRAM_SPEC.md` — naming policy, source-of-truth
  precedence, M3/M4 acceptance tables.
- `specs/ENGINE_SPEC.md` — canonical types: Pin, Unit, Entity, Gate,
  Pad, Package, Part, Board, Schematic, Rule, RuleScope. Engine API
  surface (current + target M2+).
- `specs/NATIVE_FORMAT_SPEC.md` — native project layout (`project.json`,
  schematic/board files, rules, settings, `.ids/`), schema versioning,
  imported-project conversion rules.
- `specs/IMPORT_SPEC.md` — UUID v5 algorithm, sidecar `.ids.json`,
  KiCad/Eagle import feature matrices, round-trip guarantees,
  accepted lossiness.
- `specs/MCP_API_SPEC.md` — full implemented MCP/daemon method catalog
  including M5/M6 routing surfaces.
- `specs/M7_FRONTEND_SPEC.md` — opening read-only review workspace,
  `board_review_scene_v1` contract, fixed panel set, deferred
  features.
- `specs/SCHEMATIC_EDITOR_SPEC.md` — authored schematic object model,
  M4 operation set, hierarchy/instance semantics.
- `specs/SCHEMATIC_CONNECTIVITY_SPEC.md` — net resolution rules,
  labels/buses/hierarchy/no-connect, diagnostics.
- `specs/CHECKING_ARCHITECTURE_SPEC.md` — ERC/DRC domain split, shared
  result/severity/waiver models.
- `specs/ERC_SPEC.md` — pin-electrical types, M2 rule set, compatibility
  matrix.
- `docs/CANONICAL_IR.md` — UUID identity, units (nm/0.1°), authored vs
  derived data, transaction model, JSON serialization, rule expression
  tree.
- `docs/POOL_ARCHITECTURE.md` — Unit/Symbol/Entity/Package/Padstack/Part
  hierarchy, SQLite index, parametric tables, pool-population sources.
- `docs/LIBRARY_ARCHITECTURE.md` — Horizon-style canonical model
  product direction, IPC-aware footprint system pointer.
- `docs/MCP_DESIGN.md` — non-normative MCP rationale.
- `docs/CHECKING_ARCHITECTURE.md` — non-normative ERC/DRC separation
  rationale.
- `docs/AUTHORING_TOOLS.md` — non-normative tool-semantics rationale.
- `docs/ENGINE_DESIGN.md` — non-normative engine-architecture overview.
- `docs/NATIVE_FORMAT.md` — supplemental native-format design notes.
- `docs/INTEROP_SCOPE.md` — KiCad-priority-1, Eagle-priority-1,
  commercial deferred to M5+, future export list (ODB++, STEP, IPC-2581
  noted).
- `docs/COMMERCIAL_INTEROP_STRATEGY.md` — Altium/PADS/OrCAD migration
  staging strategy, library-first/full-design-later.
- `research/ipc-compliance/IPC_COMPLIANCE_RESEARCH.md` — already-delivered
  IPC survey covering land patterns, generic design, data exchange,
  acceptability standards, and tool-implementation comparison.

## Per-Domain Audit

### 1. Data exchange & interop

#### Standards inventory

| Standard | Title | Body | Type | Coverage | Evidence (file:line) | Priority |
|----------|-------|------|------|----------|----------------------|----------|
| STEP AP203 | Configuration Controlled 3D Designs of Mechanical Parts and Assemblies | ISO 10303-203 | output/interop | Blind Spot | — | high |
| STEP AP214 | Core Data for Automotive Mechanical Design Processes | ISO 10303-214 | output/interop | Blind Spot | — | medium |
| STEP AP242 | Managed Model-Based 3D Engineering | ISO 10303-242 | output/interop | Blind Spot | — | high |
| IDF 2.0 / 3.0 | Intermediate Data Format (ECAD↔MCAD board exchange) | Mentor (industry de-facto) | output/interop | Blind Spot | — | high |
| IDX | Incremental Design Exchange (ECAD↔MCAD ECO) | ProSTEP iViP | output/interop | Blind Spot | — | high |
| EDMD | Electrical Design Model Data (XML ECAD↔MCAD) | ProSTEP iViP | output/interop | Blind Spot | — | medium |
| DXF (AutoCAD R12+) | Drawing Interchange Format | Autodesk | output/interop | Blind Spot | — | medium |
| ODB++ | Open Database (PCB fab+assembly) | Mentor/Siemens (open spec) | output/interop | Partial | `docs/INTEROP_SCOPE.md:89` (listed under Future M5+) | high |
| IPC-2581C | DPMX (single-XML fab+assembly) | IPC | output/interop | Already-researched | `research/ipc-compliance/IPC_COMPLIANCE_RESEARCH.md` §IPC-2581 (lines 625-690); also `docs/INTEROP_SCOPE.md:91` | high |
| Gerber RS-274X | Extended Gerber image format | Ucamco | output/interop | Covered | `specs/PROGRAM_SPEC.md:230`; `docs/INTEROP_SCOPE.md:74` | (already in roadmap) |
| Gerber X2 | Extended attributes (net names, components) | Ucamco | output/interop | Covered | `docs/INTEROP_SCOPE.md:76` | medium |
| Gerber X3 | Assembly attributes superset of X2 | Ucamco | output/interop | Already-researched | `research/ipc-compliance/IPC_COMPLIANCE_RESEARCH.md` §IPC-2581 (cross-ref lines 671-678) | medium |
| Excellon (XNC) | Drill / route file format | Pre-IPC industry | output/interop | Covered | `specs/PROGRAM_SPEC.md:231`; `docs/INTEROP_SCOPE.md:77` | (already in roadmap) |
| IPC-D-356A | Bare-board electrical test netlist | IPC | output/interop | Already-researched | `research/ipc-compliance/IPC_COMPLIANCE_RESEARCH.md` §IPC-D-356 (lines 692-722) | medium |
| Specctra DSN/SES | Routing engine interchange (design / session) | Cadence Specctra | output/interop | Blind Spot | — | low |
| Hyperlynx HYP | SI/PI extraction interchange | Mentor/Siemens | output/interop | Blind Spot | — | skip |
| Altium native (.PcbDoc/.SchDoc) | OLE compound documents | Altium | output/interop | Out of Scope (R1 research only) | `docs/INTEROP_SCOPE.md:53` (Altium "Future — M5+, Not in v1 scope"); `docs/COMMERCIAL_INTEROP_STRATEGY.md:142-167` | high |
| KiCad native (.kicad_pcb/.kicad_sch) | S-expression source of truth | KiCad project | output/interop | Covered | `specs/IMPORT_SPEC.md:111-163` (KiCad import feature matrix); `specs/PROGRAM_SPEC.md:201` (KiCad write-back) | (already in roadmap) |
| Eagle native (.brd/.sch/.lbr) | XML, DTD-defined | Autodesk (legacy) | output/interop | Covered | `specs/IMPORT_SPEC.md:166-211` (Eagle import feature matrix) | (already in roadmap) |
| OrCAD/Allegro native (.brd/.dsn) | Allegro database | Cadence | output/interop | Out of Scope (R1 research only) | `docs/COMMERCIAL_INTEROP_STRATEGY.md:169-184` | medium |
| PADS native (.pcb/.sch) | PADS Logic/Layout binary | Siemens | output/interop | Out of Scope (R1 research only) | `docs/COMMERCIAL_INTEROP_STRATEGY.md:191-213` | medium |

#### Cross-domain overlaps
- IPC-2581 / Gerber X3 / IPC-D-356A appear here and in IPC research;
  do not re-investigate.
- IPC-1752A (materials declaration) intersects Domain 5; IPC research
  noted it (`research/ipc-compliance/IPC_COMPLIANCE_RESEARCH.md` §1752,
  lines 744-764).
- STEP / IDF / IDX overlap with the future 3D viewer (CLAUDE.md "Not
  Yet Implemented" line 226). Domain 4 aerospace concerns also touch
  STEP AP242.

#### Notes
- Datum's `Package.models_3d: Vec<ModelRef>`
  (`specs/ENGINE_SPEC.md:138`) and `ModelRef { path, transform }`
  (`specs/ENGINE_SPEC.md:74-77`) reserve a 3D-model slot but defer the
  actual format and transform exactly.
- KiCad import explicitly defers 3D model assignments
  (`specs/IMPORT_SPEC.md:135`, "Deferred — No 3D in v1").
- The "Future M5+" export list in `docs/INTEROP_SCOPE.md:89-92`
  enumerates ODB++, STEP, PDF, IPC-2581 but does not classify by
  AP203/214/242 or note IDF/IDX/EDMD at all.

### 2. Component modelling

#### Standards inventory

| Standard | Title | Body | Type | Coverage | Evidence (file:line) | Priority |
|----------|-------|------|------|----------|----------------------|----------|
| IBIS 7.x | I/O Buffer Information Specification | IBIS Open Forum | library/data | Blind Spot | — | high |
| IBIS-AMI | Algorithmic Modeling Interface | IBIS Open Forum | library/data | Blind Spot | — | medium |
| Touchstone .sNp v1.1 / v2.0 | S-parameter file format | IEEE Std 370 / IBIS | library/data | Blind Spot | — | high |
| SPICE3 | Berkeley SPICE netlist syntax | UC Berkeley | library/data | Blind Spot | — | medium |
| PSpice / OrCAD subset | Cadence SPICE dialect | Cadence | library/data | Blind Spot | — | low |
| HSPICE subset | Synopsys SPICE dialect | Synopsys | library/data | Blind Spot | — | low |
| LTspice / ngspice subset | Open-source SPICE | open community | library/data | Blind Spot | — | medium |
| JEDEC JEP30 (PIP) | Part Information Profile (XML metadata) | JEDEC | library/data | Blind Spot | — | low |
| VHDL-AMS / Verilog-A | Mixed-signal HDL behavioural | IEEE 1076.1 / Accellera | library/data | Blind Spot | — | skip |
| ECAD STEP package model | Per-Package 3D body STEP file | Vendor | library/data | Partial | `specs/ENGINE_SPEC.md:74-77,138` (`ModelRef.path` reserved); `specs/IMPORT_SPEC.md:135,159` (3D model imports deferred) | medium |
| JEDEC JESD8 family | Logic-family interface (LVTTL, LVCMOS…) | JEDEC | library/data | Blind Spot | — | low |
| JEDEC MO outline drawings | Mechanical-outline drawings (MO-220 etc.) | JEDEC | library/data | Blind Spot (referenced indirectly via IPC-7351 land patterns) | — | low |
| Datum Part metadata: parametrics, lifecycle, MPN | Pool Part record | Datum-internal | library/data | Covered | `specs/ENGINE_SPEC.md:147-162` (`Part.parametric`, `lifecycle`, `mpn`, `orderable_mpns`); `docs/POOL_ARCHITECTURE.md:104-119` | (already in roadmap) |
| Datum Pin electrical-type model | ERC pin semantics | Datum-internal | library/data | Covered | `specs/ENGINE_SPEC.md:32-43`; `specs/ERC_SPEC.md:31-44` | (already in roadmap) |

#### Cross-domain overlaps
- ECAD STEP package model overlaps Domain 1 (STEP AP203/AP242).
- IBIS feeds Domain 6 EMC/SI workflows (DDR, USB, PCIe pre-layout).

#### Notes
- The `Part.datasheet: String` field (`specs/ENGINE_SPEC.md:156`) is a
  URL only — no behavioural-model attachment surface.
- Eagle import explicitly defers SPICE models
  (`specs/IMPORT_SPEC.md:210`, "Spice models | Deferred").
- Datum's pin-direction enum (Input, Output, Bidirectional, Passive,
  PowerIn, PowerOut, OpenCollector, OpenEmitter, TriState, NoConnect
  in `specs/ENGINE_SPEC.md:32-43`) matches IBIS-style buffer
  classification at the connectivity layer but carries no IBIS
  electrical parameters.

### 3. Schematic & drawing conventions

#### Standards inventory

| Standard | Title | Body | Type | Coverage | Evidence (file:line) | Priority |
|----------|-------|------|------|----------|----------------------|----------|
| IEEE 315-1975 / 315A-1986 | Graphic Symbols for Electrical and Electronics Diagrams | IEEE/ANSI Y32.2 | library/data | Blind Spot | — | medium |
| IEC 60617 | Graphical symbols for diagrams | IEC | library/data | Blind Spot | — | medium |
| IEEE 200-1975 / ANSI Y32.16 | Reference designation for electrical and electronics parts | IEEE / ANSI | library/data | Partial | `specs/ENGINE_SPEC.md:118` (`Entity.prefix`); `specs/SCHEMATIC_EDITOR_SPEC.md:269-275` (Annotation determinism) — prefix is free-form, no IEEE 200 constraint or table | medium |
| ASME Y14.44 / IEEE 200-1975 successor | Modern reference designation practice | IEEE / ASME | library/data | Blind Spot | — | medium |
| ISO 7200 | Title blocks and headers for technical product documentation | ISO | layout | Partial | `specs/ENGINE_SPEC.md:309-315` (`SheetFrame { title, revision, company, page_number }`) — fields exist but no ISO 7200 mandatory-field validation | low |
| ANSI Y14.1 | Drawing sheet sizes (A/B/C…) | ASME | layout | Blind Spot | — | low |
| ANSI Y14.5 (GD&T) | Geometric Dimensioning and Tolerancing | ASME | output/interop | Blind Spot | — | skip |
| IEC 81714 | Design of graphical symbols for use in technical documentation of products | IEC | library/data | Blind Spot | — | low |
| IPC-T-50 | Terms and Definitions | IPC | library/data | Already-researched | `research/ipc-compliance/IPC_COMPLIANCE_RESEARCH.md` §IPC-T-50 (lines 726-742) | medium |
| Hierarchical-sheet semantics (Altium/Cadence convention) | Industry-de-facto multi-sheet hierarchy | Altium / Cadence (informal) | layout | Covered | `specs/SCHEMATIC_CONNECTIVITY_SPEC.md:96-103` (hierarchy resolution); `specs/SCHEMATIC_EDITOR_SPEC.md:33-71` (Sheet/SheetDefinition/SheetInstance) | (already in roadmap) |
| Bus/bus-entry convention | KiCad/Eagle bus-syntax compatibility | KiCad / Eagle (informal) | layout | Partial | `specs/SCHEMATIC_CONNECTIVITY_SPEC.md:104-117` (bus rules; current scope = `NAME[n]` and `NAME[a..b]`) | low |
| Net-naming convention (global/local/hierarchical) | Industry net-scoping | Industry-wide | layout | Covered | `specs/SCHEMATIC_CONNECTIVITY_SPEC.md:84-95` (label scoping rules) | (already in roadmap) |

#### Cross-domain overlaps
- IPC-T-50 vocabulary applies wherever Datum surfaces user-facing
  text (Domains 3, 4, 8); IPC research already recommended it as
  a vocabulary baseline.
- Sheet-frame fields touch both Domain 3 (drawing convention) and
  Domain 8 (process traceability — revision/company are audit-trail
  data).

#### Notes
- Datum has no normative position on whether native symbols ship in
  IEEE 315 (US/ANSI) or IEC 60617 (European) graphic style. Symbol
  graphics are opaque `Vec<Primitive>` (`specs/ENGINE_SPEC.md:79-87`),
  so the rendering style is delegated to whatever the imported library
  ships.
- Reference-designator generation determinism is required
  (`specs/SCHEMATIC_EDITOR_SPEC.md:269-275`) but the prefix list is
  not constrained against IEEE 200 / ASME Y14.44 (e.g., no rule that
  diodes use D, transistors Q, integrated circuits U).

### 4. Industry-vertical compliance

#### Standards inventory

| Standard | Title | Body | Type | Coverage | Evidence (file:line) | Priority |
|----------|-------|------|------|----------|----------------------|----------|
| AEC-Q100 / Q101 / Q200 | Automotive component qualification (IC / discrete / passive) | Automotive Electronics Council | library/data | Blind Spot | — | medium |
| ISO 26262 | Road vehicles — Functional safety | ISO | process | Blind Spot | — | medium |
| IATF 16949 | Automotive QMS | IATF | process | Blind Spot | — | low |
| DO-254 | Design Assurance Guidance for Airborne Electronic Hardware | RTCA | process | Blind Spot | — | skip |
| AS9100 | Aerospace QMS | SAE/IAQG | process | Blind Spot | — | skip |
| MIL-PRF-31032 | Printed wiring boards, general specification | DoD | process | Blind Spot | — | skip |
| MIL-PRF-55110 | Printed wiring boards, rigid | DoD | process | Blind Spot | — | skip |
| ITAR (22 CFR 120-130) | Arms Export Control / controlled-data markings | US State Dept | process | Blind Spot | — | medium |
| EAR (15 CFR 730-774) | Export Administration Regulations | US Commerce | process | Blind Spot | — | medium |
| NASA-STD-8739.x | Workmanship standards (soldering, harness, etc.) | NASA | process | Blind Spot | — | skip |
| IEC 60601 | Medical electrical equipment | IEC | process | Blind Spot | — | low |
| ISO 13485 | Medical-device QMS | ISO | process | Blind Spot | — | medium |
| FDA 21 CFR Part 820 | Quality System Regulation (medical devices) | US FDA | process | Blind Spot | — | medium |
| FDA 21 CFR Part 11 | Electronic Records / Electronic Signatures | US FDA | process | Partial | `docs/CANONICAL_IR.md:122-188` (deterministic transactions, undo); `specs/ENGINE_SPEC.md:684-731` (transaction model) — substrate exists, no electronic-signature/audit-claim layer | medium |
| IEC 61508 | Functional safety of E/E/PE safety-related systems | IEC | process | Blind Spot | — | low |
| IPC-A-600 / IPC-A-610 (Class 1/2/3) | PCB / assembly acceptability classes | IPC | process/library | Already-researched | `research/ipc-compliance/IPC_COMPLIANCE_RESEARCH.md` §IPC-A-600/610 (lines 566-622) | medium |

#### Cross-domain overlaps
- IPC-A-600/610 Class 1/2/3 selectability also drives Domain 5
  (materials/finish) and the entire DRC default-rule story; IPC
  research recommended `ipc_class` project-level metadata.
- 21 CFR Part 11 substrate (transaction log, undo determinism)
  overlaps Domain 8 (process & quality / audit trail).
- ITAR/EAR markings are also a PLM concern (Domain 7) once vault /
  check-out is in scope.

#### Notes
- Datum's variant data model (`Variant` in `specs/ENGINE_SPEC.md:440-444`,
  fitted-component selection per project-level logical component) is
  the hook on which AS9102 First-Article-Inspection traceability would
  hang. No spec mentions FAI/AS9102 today.
- Project-level "intended environment" (RH, temperature, altitude)
  was flagged by IPC research as a missing field; same field would
  serve ISO 26262 / IEC 61508 derate calculations.

### 5. Materials & environmental

#### Standards inventory

| Standard | Title | Body | Type | Coverage | Evidence (file:line) | Priority |
|----------|-------|------|------|----------|----------------------|----------|
| RoHS (Directive 2011/65/EU + 2015/863 amendment) | Restriction of Hazardous Substances | EU | output/data | Blind Spot | — | medium |
| REACH (EC 1907/2006) | Registration, Evaluation, Authorisation and Restriction of Chemicals | EU | output/data | Blind Spot | — | medium |
| WEEE (Directive 2012/19/EU) | Waste Electrical and Electronic Equipment | EU | output/data | Blind Spot | — | low |
| Conflict Minerals (Dodd-Frank §1502) | 3TG conflict-mineral reporting | US SEC | output/data | Blind Spot | — | low |
| EU Conflict Minerals (Reg 2017/821) | EU 3TG due diligence | EU | output/data | Blind Spot | — | low |
| IPC-1752A | Materials Declaration Management (XML) | IPC | output/data | Already-researched | `research/ipc-compliance/IPC_COMPLIANCE_RESEARCH.md` §IPC-1752 (lines 744-764) | medium |
| JEDEC JS709C | Halogen-free electronics | JEDEC | library/data | Blind Spot | — | low |
| IEC 62474 | Material Declaration for Products of and for the Electrotechnical Industry | IEC | output/data | Blind Spot | — | low |
| ELV (Directive 2000/53/EC) | End-of-Life Vehicles | EU | output/data | Blind Spot | — | low |
| China RoHS (SJ/T 11364) | China Hazardous Substances marking | MIIT (China) | output/data | Blind Spot | — | low |
| California Prop 65 | Safe Drinking Water and Toxic Enforcement Act | California | output/data | Blind Spot | — | skip |
| Packaging & Packaging Waste Directive | EU 94/62/EC + amendments | EU | output/data | Blind Spot | — | skip |
| SCIP database (ECHA / WFD) | Substances of Concern In articles, as such or in complex objects (Products) | ECHA | output/data | Blind Spot | — | low |
| RoHS exemption tracking | Exemption validity windows | EU | output/data | Blind Spot | — | skip |

#### Cross-domain overlaps
- IPC-1752A is the canonical XML carrier for RoHS / REACH / SCIP
  declarations; one Phase 2 deep-dive can cover the materials picture
  through the IPC-1752A lens.
- `Part.lifecycle` (Active/NRND/EOL/Obsolete) at
  `specs/ENGINE_SPEC.md:160` is the closest existing supply-chain
  field; no analogous compliance-status field exists.

#### Notes
- The Part record has no RoHS/REACH/SCIP/MSL fields. A future
  materials-declaration deep-dive should propose the minimal set
  (compliance status, last-attestation date, exemption codes) before
  any single regulation is encoded.
- BOM export is already in scope (`specs/PROGRAM_SPEC.md:231-232`,
  CSV/JSON), but no spec mentions enriching BOM rows with
  compliance markers.

### 6. EMC & signal integrity

#### Standards inventory

| Standard | Title | Body | Type | Coverage | Evidence (file:line) | Priority |
|----------|-------|------|------|----------|----------------------|----------|
| FCC Part 15 (Subparts B & C) | Radio Frequency Devices (US emissions) | FCC | layout/process | Blind Spot | — | low |
| CISPR 22 / 32 | EMC for ITE / multimedia equipment (international Class A/B) | IEC/CISPR | layout/process | Blind Spot | — | low |
| EN 55032 / 55035 | EU EMC for multimedia equipment (emission/immunity) | CENELEC | layout/process | Blind Spot | — | low |
| CISPR 25 | Vehicles, boats, internal combustion engines — radio disturbance | IEC/CISPR | layout | Blind Spot | — | low |
| DO-160 | Environmental Conditions and Test Procedures for Airborne Equipment | RTCA | process | Blind Spot | — | skip |
| IEEE 802.3 (Ethernet PHY layout checklists) | Ethernet PHY design rules pre-packaged in some EDA | IEEE | layout | Blind Spot | — | medium |
| USB 2.0 / 3.x layout requirements | USB-IF compliance design rules | USB-IF | layout | Blind Spot | — | medium |
| PCI Express base spec (trace length / impedance / via stub) | PCIe layout rules | PCI-SIG | layout | Blind Spot | — | medium |
| JEDEC JESD79-3/4/5 (DDR3/4/5 PHY rules) | DDR routing/layout rules | JEDEC | layout | Blind Spot | — | medium |
| MIPI D-PHY / C-PHY layout | MIPI Alliance display/camera PHY rules | MIPI Alliance | layout | Blind Spot | — | low |
| HDMI / DisplayPort layout guidelines | A/V interface routing | HDMI Forum / VESA | layout | Blind Spot | — | low |
| IPC-2141 (controlled impedance) | Design Guide for High-Speed Controlled Impedance Circuit Boards | IPC | layout | Already-researched (referenced) | `research/ipc-compliance/IPC_COMPLIANCE_RESEARCH.md` §IPC-2581 cross-ref (line 682) | medium |
| Datum length-match / diff-pair / impedance rule scaffold | RuleType deferred to M5+ | Datum-internal | layout | Partial | `specs/ENGINE_SPEC.md:586-594` (RuleType enum comment "M5+: Impedance, LengthMatch, DiffpairGap, DiffpairSkew"); `specs/ENGINE_SPEC.md:241-243` (NetClass `diffpair_width`, `diffpair_gap`) | high (foundational) |

#### Cross-domain overlaps
- IPC-2581C carries controlled-impedance metadata per net/layer
  (`research/ipc-compliance/IPC_COMPLIANCE_RESEARCH.md` lines
  646-651), so SI rules eventually round-trip through Domain 1.
- DDR/USB/PCIe length-match rules tie into Domain 2 (component
  modelling — IBIS provides the timing budget).
- CISPR 25 (automotive EMC) overlaps Domain 4 (automotive vertical).

#### Notes
- The deferred rule types in `RuleType`
  (`specs/ENGINE_SPEC.md:586-594`) are the only acknowledgement of
  SI-class checking; no spec mentions any pre-packaged interface
  routing template.
- Stackup material properties Dk/Df are explicitly deferred
  (`specs/ENGINE_SPEC.md:255-256`, "M8+: dk, df, copper_weight,
  roughness"), which gates impedance-solver work.

### 7. PLM & lifecycle integration

#### Standards inventory

| Standard | Title | Body | Type | Coverage | Evidence (file:line) | Priority |
|----------|-------|------|------|----------|----------------------|----------|
| Windchill | PLM platform integration (PTC) | PTC | process/interop | Blind Spot | — | skip |
| Teamcenter | PLM platform integration (Siemens) | Siemens | process/interop | Blind Spot | — | skip |
| Aras Innovator | PLM platform integration | Aras | process/interop | Blind Spot | — | skip |
| Arena PLM | Cloud PLM | PTC | process/interop | Blind Spot | — | skip |
| OpenBOM | Cloud BOM/PLM | OpenBOM | process/interop | Blind Spot | — | low |
| Component Information System (CIS) | Cadence component database integration | Cadence | process/interop | Blind Spot | — | low |
| PartQuest / PartQuest Xpress | Siemens parts library service | Siemens | library/interop | Blind Spot | — | low |
| Octopart Premium API | Component pricing/availability API | Altium/Octopart | library/interop | Blind Spot | — | medium |
| SiliconExpert | Component compliance/lifecycle service | SiliconExpert | library/interop | Blind Spot | — | low |
| IHS Markit Engineering Workbench | Component intelligence service | S&P Global | library/interop | Blind Spot | — | skip |
| AS9102 | First Article Inspection report | SAE/IAQG | process | Blind Spot | — | medium |
| Datum pool layering / shared pool concept | Pool priority / overlay | Datum-internal | library | Covered | `docs/POOL_ARCHITECTURE.md:380-396`; `specs/NATIVE_FORMAT_SPEC.md:178-199` (`project.json` `pools` array with priority) | (already in roadmap) |
| Datum lifecycle metadata | Active / NRND / EOL / Obsolete | Datum-internal | library | Covered | `specs/ENGINE_SPEC.md:50-56,160`; `docs/POOL_ARCHITECTURE.md:430-447` | (already in roadmap) |

#### Cross-domain overlaps
- Octopart / SiliconExpert both feed Domain 5 (materials/compliance
  status) and Domain 2 (component modelling — they often expose IBIS
  / SPICE / STEP).
- AS9102 First-Article-Inspection ties into Domain 4 aerospace and
  Domain 8 process/quality.

#### Notes
- Datum's pool already supports multi-pool layering with priority
  ordering (`docs/POOL_ARCHITECTURE.md:380-396`), which is the right
  shape for vault/check-out semantics. No spec describes vault
  protocol, lock, or check-out/check-in.
- No spec mentions any external BOM enrichment service. The MCP
  surface is a natural place to add a "lookup_part_supply_chain"
  tool, but it is not currently planned.

### 8. Process & quality

#### Standards inventory

| Standard | Title | Body | Type | Coverage | Evidence (file:line) | Priority |
|----------|-------|------|------|----------|----------------------|----------|
| ISO 9001 | QMS — general (audit log, change control) | ISO | process | Partial | `docs/CANONICAL_IR.md:122-188` (transaction/operation determinism); `specs/ENGINE_SPEC.md:684-731` (Operation/OpDiff/Transaction); `specs/ENGINE_SPEC.md:309-315` (`SheetFrame.revision`) — substrate is present, audit-export view absent | medium |
| AS9100 | Aerospace QMS | SAE/IAQG | process | Blind Spot | — | skip |
| IATF 16949 | Automotive QMS | IATF | process | Blind Spot | — | skip |
| ISO 13485 | Medical-device QMS | ISO | process | Blind Spot | — | medium |
| CMMI for Development | Capability Maturity Model Integration | ISACA / CMMI Institute | process | Blind Spot | — | skip |
| ISO/IEC 12207 | Software life cycle processes | ISO/IEC | process | Blind Spot | — | skip |
| 21 CFR Part 11 | Electronic records / electronic signatures | US FDA | process | Partial | `docs/CANONICAL_IR.md:122-188`; `specs/ENGINE_SPEC.md:709-731` (Transaction with id+description) — undo/redo deterministic, no signature surface | medium |
| ECO (Engineering Change Order) management | EDA-side ECO workflow | Industry-wide | process | Covered | `specs/PROGRAM_SPEC.md:228` (Forward annotation with per-change accept/reject); `specs/INTEGRATED_PROGRAM_SPEC.md:289-291,302-318` (M4 ECO contracts); `docs/AUTHORING_TOOLS.md:151-160` | (already in roadmap) |
| Design version control (git-based) | Diff-friendly project files | Industry-wide | process | Covered | `docs/CANONICAL_IR.md:191-211` (deterministic JSON serialization); `docs/POOL_ARCHITECTURE.md:158-167,389-396` (per-file pool, git-friendly) | (already in roadmap) |
| Library approval / release workflow | Vault check-in/out, supersede | Industry-wide | process | Blind Spot | — | medium |
| Design review sign-off workflow | Reviewer/approver electronic markup | Industry-wide | process | Blind Spot | — | low |
| Audit-trail completeness (ISO 19011 audit principles) | Auditor-grade activity log | ISO / industry-wide | process | Partial | Operation/Transaction model gives substrate; no exported audit-log surface — `specs/MCP_API_SPEC.md` exposes no log-query method | medium |

#### Cross-domain overlaps
- 21 CFR Part 11 also appears under Domain 4 (FDA) — same finding.
- ISO 9001 / AS9100 / ISO 13485 / IATF 16949 share the audit-trail
  primitive; one Phase 2 deep-dive on "audit-trail surface design"
  serves them all.
- ECO workflow is the foundation Datum already has; AS9102 FAI in
  Domain 4 would consume the ECO record stream.

#### Notes
- The transaction/operation model gives Datum a strong substrate for
  audit logging (`docs/CANONICAL_IR.md:122-188`), but no spec
  describes an exported, queryable change log.
- `Transaction { id, operations, description }`
  (`specs/ENGINE_SPEC.md:709-714`) carries description and id but no
  user identity, no timestamp, no signature, no rationale field. A
  21 CFR Part 11 / ISO 13485 audit overlay would need each of those.

## Phase 2 Triage Recommendations

### Pending Exclusions (advisory) — DO NOT DEEP-DIVE, DO NOT FORMALLY EXCLUDE

**Policy (ratified 2026-04-17):** the standards in
`### Recommended low-priority / skip` below are **advisory exclusions**
for Phase 2 deep-dive work. They MUST NOT be re-investigated by Phase 2
agents (do not waste cycles), but they are NOT yet committed to a
binding scope document (`docs/INTEROP_SCOPE.md`,
`specs/PROGRAM_SPEC.md`). Final ratification happens in a single
consolidated pass after Domain 8 (Process & quality) lands, when full
cross-domain context is available — at which point still-confirmed
skips are promoted to formal exclusions and any skip that turned out to
have hidden cross-cutting value gets returned to the deep-dive queue.

Phase 2 agent briefs MUST quote this policy and pass the skip list as a
boundary so domain agents know what is off-limits.

### Recommended high-priority deep-dives

1. **Domain 1 — STEP AP242 + IDF + IDX + EDMD (ECAD↔MCAD).**
   Rationale: every serious user has an enclosure team. Without a
   credible mechanical-exchange story Datum cannot be adopted by
   organisations that already use SolidWorks / Fusion / Inventor /
   Creo / NX. Cheap to scope (Datum already has a `ModelRef` slot
   and a 3D-model deferral); expensive to retrofit if data-model
   choices for stackup, courtyard, and component height bake in
   without IDF/IDX in mind. Scope of deep-dive: format families,
   commercial library options (e.g., Flow Design Solutions,
   ProSTEP iViP open source), per-format read/write delta versus
   IDF 3.0 status quo, recommendation on which pair (STEP AP242 +
   IDX vs IDF 3.0 + STEP AP203) to support first. Effort estimate:
   ~3-5 days research, ~10-page report.
2. **Domain 2 — IBIS 7.x + Touchstone + SPICE-subset.**
   Rationale: behavioural component models are the single largest
   table-stakes capability that separates Datum from a "viewer +
   ERC" tool. AI-native positioning is undermined if AI cannot
   reason about timing, jitter, or termination. Migration target
   users (Altium / OrCAD) will not adopt without it. Scope: IBIS
   parser ecosystem (open source: ibischk, pibis), Touchstone
   library options, ngspice-as-engine path versus pure-SPICE-text
   library, what Datum's `Part` record needs to grow. Effort:
   ~5-7 days, ~12-15 page report.
3. **Domain 1 — IPC-2581C + ODB++ + Gerber X3 + IPC-D-356A export
   strategy.** Rationale: modern fabrication interchange is now
   table stakes; KiCad 8+ ships IPC-2581C and Gerber X3 attributes;
   any tool without them looks dated. The IPC research already did
   most of the homework — Phase 2 here is a focused implementation-
   strategy doc on ordering, schema choice (B vs C), and which
   open-source reference exporter to mine. Effort: ~2-3 days,
   ~8 page report.
4. **Domain 6 — High-speed routing-rule scaffold (length-match,
   diff-pair, impedance, IPC-2141).** Rationale: the M5+ deferral
   in `RuleType` (`specs/ENGINE_SPEC.md:586-594`) is going to be
   the next contract family Datum needs once layout work resumes;
   getting the rule shape right against industry expectations is
   cheaper than retrofitting. Scope: USB / DDR / PCIe / Ethernet
   layout-rule taxonomies, what Altium / Cadence / KiCad expose,
   what NetClass needs to grow beyond `diffpair_width` /
   `diffpair_gap`. Effort: ~4-5 days, ~10 page report.

### Recommended medium-priority deep-dives

5. **Domain 5 — IPC-1752A as the umbrella for RoHS / REACH /
   SCIP.** Rationale: if Datum is going to land any compliance
   metadata in the Part record, doing it once through the IPC-1752A
   lens covers most regulations. Effort: ~3 days.
6. **Domain 7 — Library vault / check-out semantics + Octopart-
   class supply-chain integration.** Rationale: pool layering
   already exists, but the protocol and authoritative-source rules
   are blind spots. Effort: ~3-4 days.
7. **Domain 8 — Audit-trail surface design (ISO 9001 / 21 CFR Part
   11 / ISO 13485 substrate).** Rationale: covers four QMS regimes
   in one deep-dive by formalising the change-log export from the
   transaction model. Effort: ~2-3 days.
8. **Domain 3 — Schematic graphic-symbol style policy (IEEE 315 vs
   IEC 60617 vs JIS C 0617) + reference-designator policy (IEEE
   200 / ASME Y14.44).** Rationale: the native-symbol-authoring
   work will need a documented stance; cheaper to decide before
   M4 schematic editor work proves out. Effort: ~2 days.
9. **Domain 4 — IPC Class 1/2/3 project-level metadata (already
   recommended by IPC research) + intended-environment field +
   AEC-Q100 / ISO 26262 lightweight Part metadata (qualification
   level only).** Rationale: minimal data-model addition with
   leveraged downstream effect on DRC defaults and BOM filters.
   Effort: ~3 days.

### Recommended low-priority / skip

- **DO-254, DO-160, MIL-PRF-31032, MIL-PRF-55110, NASA-STD-8739,
  AS9100, IATF 16949, CMMI** — process-grade certification that
  the tool cannot enforce; the right answer is to position Datum
  as compatible substrate (deterministic logs, immutable history)
  and refer users to their own QMS for the certification work.
- **JEDEC JEP30 PIP, IHS Markit Engineering Workbench, JEDEC JESD8,
  JEDEC MO drawings** — superseded in practice by manufacturer-
  specific datasheets and Octopart-style metadata; deferred until
  a real distributor partnership emerges.
- **California Prop 65, EU Packaging Directive, RoHS-exemption
  tracking** — out of EDA tool scope in any reasonable v1.
- **HDMI / DisplayPort / MIPI layout templates** — fold into the
  high-speed-rule deep-dive if it materialises; otherwise skip.
- **Specctra DSN/SES, Hyperlynx HYP** — Specctra is legacy;
  Hyperlynx is one vendor's SI-extraction format, not an open
  exchange.
- **Windchill / Teamcenter / Aras / Arena PLM** — connector work,
  not Datum-engine work; if a customer demands it, build per-
  customer; do not build pre-emptively.

### Recommended scope cuts to bring forward

- `docs/INTEROP_SCOPE.md` should grow an explicit "Out of v1 / Out
  of v2 scope" section that names the regulatory-vertical
  certifications (DO-254, MIL-PRF, NASA-STD, AS9100, IATF 16949,
  Prop 65, EU Packaging) so target-market users do not have to
  guess. Today these are silent omissions.
- `specs/PROGRAM_SPEC.md` v1 Definition (lines 61-79) should be
  augmented with a one-line "v1 does not implement industry
  vertical QMS support" statement parallel to the existing
  GUI/routing exclusions.
- `docs/INTEROP_SCOPE.md`'s "Future M5+" export list (lines 89-92)
  should be re-named to something like "Future export targets
  (research-staged)" and split into IPC-2581 / ODB++ / STEP /
  IDF-IDX rather than lumped together; the four have very
  different effort, partner, and licensing pictures.
- `docs/LIBRARY_ARCHITECTURE.md` already points at IPC footprint
  work; a parallel pointer to a "compliance metadata roadmap"
  document (covering Domains 4, 5, 7) would help thread future
  Phase-2 deep-dives back into the library subsystem rather than
  scattering them.

## Cross-Cutting Observations

- **Datum's transaction model is consistently better-positioned
  than its other surfaces for compliance work.** ISO 9001 audit
  trail, 21 CFR Part 11 electronic records, AS9102 FAI capture
  all hang naturally on the existing Operation/OpDiff/Transaction
  primitives (`specs/ENGINE_SPEC.md:684-731`,
  `docs/CANONICAL_IR.md:122-188`). The data is being captured;
  the export surface and authentication-context fields are
  missing.
- **The Part record is the universal pinch-point for industry
  metadata.** Lifecycle and parametrics are present
  (`specs/ENGINE_SPEC.md:147-162`), but every domain in this audit
  (compliance status for Domain 5, qualification level for
  Domain 4, behavioural-model link for Domain 2, supply-chain
  intel for Domain 7) wants a small addition to Part. A
  consolidated Phase 2 "Part metadata extension" deep-dive could
  serve four domains at once if planned together.
- **Datum has no concept of "intended environment" anywhere.** The
  IPC research already noted this for `intended_environment` in
  project metadata; the audit confirms ISO 26262 derate, IEC 61508
  SIL classification, IPC-A-600 Class selection, automotive vs
  consumer Q100 grade, and EMC class A/B all want the same
  upstream field. Make it once.
- **"Library" is currently scoped narrowly to symbol/footprint
  geometry.** IBIS, SPICE, STEP package model, IPC-1752A material
  declaration, and AEC-Q qualification data are all "library"
  data in the broader industry sense but have no representation in
  the Datum pool today. `docs/LIBRARY_ARCHITECTURE.md` is the
  natural place to expand this conception.
- **Frontend (M7) does not appear in any compliance discussion.**
  The M7 spec is read-only review; compliance metadata visibility
  in the GUI is silently deferred. That is fine for v1, but Phase
  2 deep-dives should explicitly note GUI-deferral so the
  compliance work is not blocked on M7 timeline.
- **Datum's spec attribution rules (`CLAUDE.md` Attribution
  Policy) are themselves a process-quality artifact** that
  intersects ISO 9001 / 21 CFR Part 11 controlled-source
  requirements; worth a short note in the Domain 8 deep-dive.

## Sources

(Phase 1 was a Datum-spec reading exercise; no web research was
needed to confirm standard revisions because the IPC-revision-state
question was already settled by `research/ipc-compliance/`. No web
URLs are cited.)
