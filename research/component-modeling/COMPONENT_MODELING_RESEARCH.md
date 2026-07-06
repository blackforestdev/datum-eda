# Component Modelling — Industry Survey & Datum EDA Implementation Strategy

> Phase 2 deep-dive on Domain 2 of the 8-domain standards audit.
> Continues from `research/standards-audit/STANDARDS_AUDIT.md § 2`
> ("Per-Domain Audit → 2. Component modelling").
> Cross-references
> `research/data-exchange-interop/DATA_EXCHANGE_INTEROP_RESEARCH.md`
> for STEP MCAD bodies (do not re-research) and
> `research/ipc-compliance/IPC_COMPLIANCE_RESEARCH.md` for IPC-7351
> footprint geometry and IPC-2581 embedded model references.
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
> For Domain 2 the advisory exclusion list contains: **JEDEC JEP30
> (PIP — Part Information Profile)** (superseded by manufacturer
> datasheets and Octopart-style metadata), **JEDEC JESD8** (logic-
> family electrical specifications, superseded), **JEDEC MO
> (Mechanical Outline) drawings** (superseded by manufacturer 3D STEP
> models), and **IHS Markit Engineering Workbench** (paid distributor
> catalog, not a Datum-engine concern). These were not deep-dived.
> They are surfaced under "Pending Exclusions (re-affirmed)" with
> cross-cutting-value notes.

## Executive Summary

- **Behavioural component models are the single largest table-stakes
  capability that separates Datum from a "viewer + ERC" tool.** The
  Phase 1 audit ranked Domain 2 deep-dive as #2 priority because every
  Altium / OrCAD / Cadence / PADS user evaluating a migration target
  asks the same first question: "does it ingest IBIS, can it sim with
  SPICE, does it export Touchstone?" Datum currently answers "no" to
  all three, with `Part.datasheet: String` (`specs/ENGINE_SPEC.md:156`)
  being a URL-only attachment surface and Eagle SPICE imports
  explicitly deferred (`specs/IMPORT_SPEC.md:210`). The good news is
  that the **pool architecture is the right shape** to add a
  `Part.behavioural_models` field without disturbing any existing
  semantics; the design work is choosing storage location (in-Part vs
  attachment) and pluggability boundary, not rebuilding the model.
- **The library-vs-attachment question has a clear winner: KiCad's
  attached-by-URI pattern, not Altium's embedded-component pattern.**
  Altium embeds `.SimulationModel` and `.IBISModel` as part of its
  `Component` BLOB, which gives stronger versioning but couples the
  library to vendor-specific file formats and complicates encryption
  handling. KiCad references external `.cir` / `.lib` / `.s2p` /
  `.ibs` files by URI from inside `.kicad_sym`, which keeps libraries
  text-diffable and avoids re-encoding vendor-encrypted models. **For
  Datum's git-friendly pool architecture and AI-native positioning,
  the URI-attached model is the right pattern**: pool stores models
  as first-class pool entries (alongside Symbol, Package, Padstack);
  Part records carry `behavioural_models: Vec<ModelAttachment>` where
  each `ModelAttachment` is a UUID reference + role enum
  (Spice / Ibis / Touchstone / VerilogA). This preserves vendor
  encryption boundaries, allows multi-revision SPICE attached to one
  Part (typical / fast / slow corners), and lets the AI surface
  inspect attachment metadata without ever decrypting the contents.
- **Simulation backend pluggability matters more than which simulator
  Datum embeds first.** No PCB tool surveyed embeds an SI engine in
  the canonical IR; every tool that does SI runs an external simulator
  and renders results. Altium embeds Mixed Sim (Berkeley SPICE-3
  derivative) for analog SPICE and ships an internal IBIS simulator
  for IBIS pre-layout; HyperLynx is the bolt-on; Sigrity is the
  Cadence acquisition. KiCad pipes to ngspice via the `kicad-cli`
  netlist export → ngspice CLI. **Datum should adopt the KiCad
  pattern**: embed nothing, emit clean SPICE / Touchstone netlists
  via `export_spice_netlist` and `export_ibis_stimulus` MCP tools,
  let the user (or the AI agent) pipe to ngspice / Xyce / LTspice /
  Qucs externally. This avoids GPL-3 contagion (ngspice is GPL-3 and
  cannot be statically linked into a non-GPL Datum binary), keeps the
  engine lean, and matches Datum's "engine + consumer" architectural
  philosophy.
- **Encrypted vendor models are a new class of IP-management hazard
  that Datum's AI surface must explicitly handle.** IBIS 7.0+ supports
  encrypted `.ibs` blocks (using IBIS Open Forum's documented
  encryption schemes); PSpice, HSPICE, and Spectre all ship encrypted
  `.LIB` / `.inc` / `.scs` files. The AI agent must NOT exfiltrate
  encrypted-model contents into prompts, completions, or training
  signals — even passively. The pool's existing provenance fields
  (`source_format`, `source_path`) are the right place to mark a
  model as encrypted; MCP tools that read model contents must check
  the encrypted flag and either refuse or return only a
  capability-tagged opaque handle. Altium handles this by treating
  encrypted models as binary blobs with a "do not decrypt" flag;
  Cadence handles it via the AMS Designer license boundary. **Datum
  should specify a `ModelAttachment.encrypted: bool` field and a
  parallel `encryption_scheme: Option<EncryptionScheme>` field, then
  gate AI exposure of model contents on that flag at the MCP layer.**
  This is a cross-cutting finding that affects Domain 8 (process
  & quality) — encrypted-model audit-trail emission is its own
  compliance obligation — flagged at end of report.
- **The IBIS ecosystem is much smaller and more concentrated than
  SPICE.** Three open-source IBIS parsers exist:
  **`ibischk`** (IBIS Open Forum reference parser, BSD-style license,
  C99, the canonical validator), **`pibis`** / **`pyibis-ami`**
  (Python, MIT, IBIS Open Forum-curated; AMI-aware), and KiCad's
  internal IBIS parser (GPL-3, C++, behaviour-tied to the KiCad SI
  workflow). Vendor IBIS models are universally distributed by the
  manufacturer (TI, ADI, NXP, Microchip, ST, Maxim/ADI, Renesas,
  Infineon, Bosch). Quality is uneven — TI and ADI lead, the
  long-tail mid-volume vendors lag a year or two behind silicon
  release. **Datum should use `ibischk` as the validation gate for
  attached IBIS models** and lift no IBIS authoring into the engine
  in v1; users who need to author IBIS will use HSPICE2IBIS, Mentor
  Visual IBIS Editor, or hand-written `.ibs` files. For IBIS-AMI
  (the algorithmic / DSP-aware extension used by serdes), the only
  practical path is to ingest vendor-supplied `.ami` + DLL/`.so`
  bundles and run them via Cadence Sigrity / ADS / HyperLynx — this
  is genuinely out of Datum's reach and should be marked Out of Scope.
- **Touchstone is the easiest win in Domain 2 and the highest leverage
  per engineering hour.** Touchstone (.s1p / .s2p / .s4p / .sNp,
  formalised by IBIS / IEEE 370) is plain ASCII, has a simple grammar
  (`# GHZ S MA R 50` header + frequency-and-S-parameter rows), and
  has an excellent open-source reference reader: **scikit-rf** (BSD,
  Python). For Datum, attaching `.sNp` files to a Part adds zero
  data-model complexity (it's just a `ModelAttachment` with role
  `Touchstone`), and the validator is ~200 lines of Rust. **Ship
  Touchstone attachment in M7+ post-routing milestone** — it is the
  cheapest Domain 2 capability with the broadest user impact (used
  by every connector / cable / IC SI workflow).
- **The SPICE landscape is dominated in 2026 by ngspice (open-source,
  KiCad's choice), LTspice (Analog Devices, free Windows + Mac, the
  hobbyist standard), HSPICE (Synopsys, paid, the IC-design industry
  standard), and PSpice (Cadence/OrCAD, paid, the PCB schematic-
  capture industry standard).** Spectre (Cadence, IC) and Xyce
  (Sandia, open-source HPC SPICE) are real but niche. Berkeley SPICE
  3f5 itself is academic / unmaintained. The `.MODEL` / `.SUBCKT`
  syntax is largely portable across simulators; `.OPTIONS`, control
  cards, and behavioural-source extensions are not. Vendors
  distribute SPICE models in their own dialect's syntax — TI ships
  PSpice and TINA-TI-flavoured `.LIB` files; ADI ships LTspice
  `.cir` files; ST ships PSpice. **For Datum, ingest as opaque text
  with a `dialect: SpiceDialect` tag and emit netlists in
  ngspice-compatible syntax for the open-source path; emit
  PSpice-compatible syntax for users running Cadence-side tools.**
  This is a 2-3-week sprint of grammar work, not a major engine
  refactor.
- **The HDL side (Verilog / SystemVerilog / VHDL) is essentially
  out of scope for a PCB engine.** Verilog (IEEE 1364), SystemVerilog
  (IEEE 1800), and VHDL (IEEE 1076) are FPGA / ASIC HDLs — the model
  output of these flows is a synthesised netlist (`.v` / `.vhd`
  post-PAR) or a programming bitstream (`.bit` for Xilinx, `.fbg` /
  `.bin` for Lattice, `.jed` for CPLDs). PCB tools do not consume
  HDL directly; they consume **the resulting IO pin map** as a
  pin-list / IO-constraint file. Verilog-A / Verilog-AMS (Accellera)
  and VHDL-AMS (IEEE 1076.1) are mixed-signal HDLs used to author
  compact device models for circuit simulation, but Datum has no
  authoring-of-models story in v1. **Recommend Out of Scope** for
  HDL languages themselves; **recommend In Scope** for the
  pin-mapping consumption surface (e.g., Xilinx XDC / Vivado pin
  constraint files, Intel/Altera Quartus QSF, Lattice LPF) as a
  Domain 4 industry-vertical FPGA-PCB integration concern, not a
  Domain 2 component-modelling concern.
- **Two-resistor thermal models (DELPHI / JEDEC JESD15-3) are the
  realistic thermal entry point; full compact-thermal (JESD15-4 /
  ECXML) is post-M8.** Every datasheet for a power-dissipating part
  publishes Θ_JA (junction-to-ambient) and Θ_JC (junction-to-case)
  resistance values; storing two `f32` fields on Part covers 80% of
  thermal queries an AI agent might want to answer ("can this MOSFET
  dissipate 1 W on the proposed copper pad?"). The full DELPHI
  compact-thermal-model network — junction node + multiple surface
  nodes + a resistor matrix — is what Mentor FloTHERM, Ansys IcePak,
  and 6SigmaET consume; it is XML-encoded (ECXML) and shipped by a
  small number of high-volume vendors (TI, NXP, Infineon for power
  parts only). **Recommend in Scope: two-resistor on `Part`. Out of
  Scope for v1: ECXML compact-thermal**, with re-evaluation when
  Datum adds thermal analysis.
- **Component metadata is dominated by Octopart / Nexar API and the
  big-four distributor APIs (Digi-Key, Mouser, Arrow, Avnet); CIS
  bridges are an enterprise-only concern.** Octopart was acquired by
  Altium and rebranded **Nexar** in 2020; the underlying API is
  GraphQL with a free tier (1000 queries/day) and a paid tier
  (Altium subscribers get higher quota). Distributor direct APIs are
  REST/JSON, free with key registration, generally easier to
  integrate but cover only that distributor's catalog. SiliconExpert
  is paid-only ($$). Cadence CIS (Component Information System) and
  Altium DBLib / SVNDBLib are MS-SQL / ODBC bridges into ERP / PLM
  databases — a real workflow inside large companies but not a
  standard. **Datum should ship a small `LookupPart` MCP tool that
  proxies Octopart/Nexar (since it covers the most distributors in
  one query) and a parallel `LookupDigiKey` / `LookupMouser` for
  direct-distributor users; keep CIS bridges out of v1.** JEDEC
  JEP106 manufacturer ID codes are useful as a canonical
  manufacturer-name normalisation table — TI is `0x97`, NXP is
  `0x15`, etc. — and worth shipping as a static lookup.
- **Pin / symbol modelling standards (IEEE 991, IEC 60617) are
  largely advisory in 2026; the operational pin-direction enum is
  what matters.** IEEE 991 (Logic Circuit Diagrams) and IEC 60617
  (Graphical Symbols for Diagrams) define drawing conventions for
  schematic symbols; they do not standardise pin-direction
  vocabulary the way IBIS does. The operational pin-direction set
  every PCB tool uses (Input / Output / Bidir / Tristate /
  Open-collector / Open-emitter / Power-in / Power-out / Passive /
  No-connect) is folk-standard, originated in the 1990s SPICE / EDIF
  era, and matches Datum's `PinElectricalType`
  (`specs/ENGINE_SPEC.md:32-43`) line-for-line. **Datum's pin model
  is already complete; no spec edit needed for IEEE 991 or IEC
  60617.** Differential-pair and bus-naming conventions (`P/N`
  suffixes, `[7:0]` slices) are de-facto, codified per-tool, and
  Datum's bus-syntax handling
  (`specs/SCHEMATIC_CONNECTIVITY_SPEC.md:104-117`) covers the
  standard cases.
- **Datum has eleven concrete recommended spec edits that fall out
  of this research.** They are listed at the end with effort
  estimates so the project owner can sequence them. The highest-
  leverage single edit is to extend `Part` with a
  `behavioural_models: Vec<ModelAttachment>` field and add the
  corresponding `ModelAttachment` type to canonical IR — every other
  Domain 2 capability hangs off this. Second-highest is the
  `encrypted: bool` flag and the AI-surface gate it implies, because
  shipping encrypted-model handling later is far more painful than
  shipping it from day one.

## Standards Catalog

### Signal Integrity / Electrical Behavioural Models

#### IBIS 7.x

**Full title.** *I/O Buffer Information Specification*, currently
**version 7.2** (ratified August 2024). Earlier mainstream
revisions: 6.1 (2017), 7.0 (2019, the major AMI-2.0 revision),
7.1 (2022, IBIS-ISS interop fixes), 7.2 (2024, encryption
clarifications + thermal diode model).

**Issuing body.** **IBIS Open Forum**, an industry consortium
operating under SAE-ITC. Also published as **IEC 62014-1** ("I/O
Buffer Information Specification"); the IEC document is roughly
6-12 months behind the IBIS Open Forum master.

**Scope.** IBIS provides a behavioural (not transistor-level)
description of an integrated circuit's input/output buffers,
sufficient for system-level signal-integrity simulation. An IBIS
file (`.ibs`) describes per-pin or per-buffer V-I curves (pull-up,
pull-down, GND-clamp, POWER-clamp), V-T waveforms (rising and
falling edges into specified test loads), package R/L/C parasitics,
ESD characteristics, and pin lists. The format is **plain ASCII**
with a section-header syntax (`[Header]`, `[Component]`, `[Pin]`,
`[Model]`, etc.). IBIS 7 added behavioural-source modelling
(BIRD-95 voltage-probe modelling), thermal diode modelling
(BIRD-198), and tightened the encryption scheme.

**Adoption status (2026).** **Mainstream and table stakes.** Every
silicon vendor publishing fast-edge IO (DDR controller, PCIe PHY,
USB PHY, Ethernet PHY, FPGA serdes IO) ships IBIS files. Quality
is uneven across vendors: TI and ADI publish high-quality IBIS
within days of silicon release; NXP, ST, Microchip, Renesas
typically lag 1-3 months; Asian vendors (BYD Semi, Gigadevice,
Sino Wealth) are sparse. The IBIS Open Forum's
[Cookbook](https://ibis.org/cookbook/) is the de-facto authoring
guide; reference is **5th Edition** (2017, still current).

**License / IP.** **Open and royalty-free.** The IBIS specification
is a free PDF download from `ibis.org`. The ANSI ratification path
(IEC 62014-1) does require an IEC purchase. The reference parser
(`ibischk`) is **BSD-licensed** and freely usable. Vendor IBIS
files themselves are typically distributed with terms-of-use
attached (no redistribution, internal-only); some vendors encrypt
sensitive parameter blocks per the IBIS encryption mechanism (see
"Encryption & IP Protection" below).

**Reference implementations.**
- **`ibischk`** — IBIS Open Forum reference parser. C99, BSD-style
  license, `parser.zip` from `ibis.org/parsers/`. The official
  IBIS-syntax-correctness validator. Validates `.ibs` files against
  the spec but does NOT simulate.
- **`pyibis-ami`** — Python, MIT, IBIS Open Forum-curated. Adds
  AMI-aware loading, S-parameter convolution for AMI input
  filtering, IBIS-7-aware. The standard Python library for IBIS
  manipulation in 2026.
- **PyBIS** — older Python library, MIT, less actively maintained.
- **KiCad IBIS support** (KiCad 7+) — internal C++ parser, GPL-3.
  Used by KiCad's IBIS-driven signal-integrity simulation. Limited
  to IBIS 5.0; AMI not supported.
- **HSPICE2IBIS / IBIS2HSPICE** — Synopsys converters; not
  open-source.
- **Mentor Visual IBIS Editor** — commercial, from Siemens EDA.
  GUI-based IBIS authoring and validation.

**EDA tool support matrix:**
- **Altium Designer** — IBIS ingest into `.IBISModel` library
  attachment; internal SI engine for pre-layout IBIS sim; AMI is
  via SimBeor / HyperLynx integration (not native).
- **OrCAD-PSpice / Cadence Sigrity** — full IBIS + IBIS-AMI ingest
  and simulation. Sigrity is the Cadence-native flagship.
- **Cadence Allegro PCB Editor + Sigrity** — IBIS-driven PHY
  layout-rule generation (interface-specific timing budgets feed
  Allegro's constraint manager).
- **Mentor PADS / Mentor HyperLynx** — full IBIS + IBIS-AMI;
  HyperLynx is the historical IBIS reference implementation.
- **Siemens Xpedition** — IBIS / IBIS-AMI native through HyperLynx.
- **KiCad 7+** — IBIS ingest, `kicad-cli` ngspice export with
  IBIS-derived stimulus; no AMI.
- **Eagle / Fusion Electronics** — no native IBIS support; can
  attach `.ibs` as a part datasheet attachment.
- **Horizon EDA** — no IBIS support.
- **LibrePCB** — no IBIS support.
- **DipTrace** — no IBIS support.
- **EasyEDA** — no IBIS support; EasyEDA Pro has limited Ngspice.

**SI/PI tool support matrix:**
- **Cadence Sigrity** — full IBIS + IBIS-AMI; the de-facto
  professional flagship.
- **Mentor HyperLynx** — full IBIS + IBIS-AMI; the historical
  reference. Supports IBIS-ISS subcircuit attachment.
- **Ansys SIwave + Q3D** — full IBIS; AMI via the Ansys ChipPI flow.
- **Keysight ADS** — full IBIS-AMI; ADS is the strongest AMI
  authoring environment.
- **CST Studio Suite (Dassault)** — IBIS via Cable Studio /
  PCB Studio.
- **SimBeor** — Simberian, mid-tier; IBIS / S-parameter focus.
- **Polar Si9000** — controlled-impedance focus, limited IBIS.

**Datum current coverage.** **Blind Spot.** No IBIS surface in
canonical IR; no parser; no validator; no MCP tool. Eagle SPICE
import deferred (`specs/IMPORT_SPEC.md:210`); IBIS not even
mentioned in any current spec.

**Implementation cost (Datum).**
- **Canonical IR**: add `ModelAttachment` type
  (`uuid`, `path`, `format: ModelFormat`, `role: ModelRole`,
  `encrypted: bool`, `encryption_scheme: Option<String>`, plus
  IBIS-specific metadata: `ibis_version: Option<String>`,
  `model_names: Vec<String>` enumerating the IBIS [Model] names).
  Add `Part.behavioural_models: Vec<ModelAttachment>`.
- **Pool**: pool needs a `models/` directory at the same level as
  `parts/`, `packages/`, etc. Each attached IBIS file becomes a
  pool entity with deterministic UUID derived from file SHA-256.
  Pool index gains `models` table.
- **Transaction model**: add `AttachModel { part_uuid, model_uuid,
  role }` and `DetachModel { part_uuid, model_uuid }` operations.
  These are reversible; no impact on derived data.
- **MCP API additions**: `attach_ibis { part_uuid, ibs_path }`,
  `validate_ibis { ibs_path }` (returns ibischk output as
  structured JSON), `list_ibis_models { part_uuid }` (lists Model
  names), `extract_ibis_pin_table { part_uuid, model_name }`
  (returns pin → buffer-model map).
- **Minimum viable**: file attachment + ibischk validation +
  pin-table extraction. Effort: ~2 weeks (FFI to ibischk via
  bindgen + thin Rust wrapper).
- **Full implementation**: + IBIS-derived ngspice stimulus
  generation (V-T edge translation into PWL voltage source) for
  pre-layout SI without an external SI tool. Effort: +3-4 weeks.
- **Partner / library dependencies**: **`ibischk`** (BSD), C99,
  `bindgen`-able. No other dependency.

**Strategic recommendation.** **Implement post-M7.** IBIS
attachment + validation is the cheapest credible Domain 2
capability and closes the largest single migration gap from
Altium / OrCAD. Defer IBIS-AMI to on-demand only.

**Risks and edge cases.**
- IBIS encryption (BIRD-176, BIRD-219) means a vendor `.ibs` file
  may have encrypted [Model] sections that ibischk treats as
  opaque blobs. Datum's MCP / AI surface must respect this — see
  "Encrypted-vendor-model handling" below.
- IBIS 7.2's thermal-diode model (BIRD-198) is so new that
  ibischk's published 2023 release does not validate it. Track
  upstream parser releases.
- IBIS file character-encoding gotchas — older `.ibs` files use
  CP-1252 instead of UTF-8 for vendor names with accented
  characters. Datum's parser must treat `.ibs` as
  Latin-1-or-UTF-8 byte input, not strict UTF-8.

#### IBIS-AMI

**Full title.** *Algorithmic Modeling Interface*, defined as
**Section 10** of the IBIS specification (current AMI rev 2.0
within IBIS 7.2). AMI 1.0 was added in IBIS 5.0 (2008); AMI 2.0
in IBIS 7.0 (2019). Sometimes referred to as "IBIS-AMI" or
"Section 10".

**Issuing body.** IBIS Open Forum, same as IBIS proper.

**Scope.** AMI extends IBIS to model **the equalisation and DSP
processing inside modern serdes IO**. An AMI model is **two
files plus a binary**:
- `.ibs` file with an `[Algorithmic Model]` reference per Model.
- `.ami` text file with parameter list, simulation flow control.
- **`.dll` (Windows) or `.so` (Linux) shared library** implementing
  the `AMI_Init` and `AMI_GetWave` C-API entry points. Compiled
  per-vendor; vendors typically supply Win32 + Win64 + Linux64
  builds.

The shared library performs the actual signal processing (FFE
pre-emphasis, CTLE, DFE, jitter modelling) that classical IBIS
voltage-time waveforms cannot represent.

**Adoption status (2026).** **Niche but essential for serdes.** If
you are simulating PCIe Gen 4/5/6, USB 3.x / 4, DDR4/5, Ethernet
10G/25G/50G/100G, you need AMI. If you are designing 100 MHz
parallel buses, you do not. Roughly 5-10% of working PCB designs
in 2026 use anything above PCIe Gen 3 / DDR4, so AMI users are a
specific subset.

**License / IP.** **Open spec, but the vendor's compiled `.dll`/`.so`
is closed.** The C-API contract is open; the implementations are
proprietary. Several vendors ship encrypted-source AMI to Cadence /
Mentor / Keysight under NDA only; the open distribution is
binary-only.

**Reference implementations.**
- **`pyibis-ami`** (Python, MIT) implements an in-process AMI host
  that can call vendor `.so` files via `ctypes` and run them
  through a simulated channel. The closest thing to an open-source
  AMI engine.
- **Keysight ADS** — the strongest commercial AMI host; AMI
  authoring template kit shipped.
- **Cadence Sigrity** — second-strongest AMI host; tight Allegro
  integration.
- **Mentor HyperLynx** — third; well-established.
- **Intel/Altera and Xilinx/AMD ship AMI for their own FPGA serdes**
  through proprietary channel-simulation tools (Altera "Channel
  Designer", Xilinx "Stargazer").

**EDA tool support matrix:** AMI is exclusively a SI-tool capability
in 2026. No PCB schematic-capture or layout tool runs AMI directly.
Layout tools that attach AMI consume it via their SI sidecar
(Altium → SimBeor / HyperLynx; OrCAD → Sigrity).

**Datum current coverage.** **Blind Spot.**

**Implementation cost (Datum).**
- **Canonical IR**: same `ModelAttachment` extension as IBIS, plus
  `ami_dll_paths: HashMap<Platform, String>` for the per-platform
  shared libraries.
- **Pool**: pool stores AMI binaries per-platform. Pool layering
  lets a Linux `.so` and a Windows `.dll` coexist; pool index
  records which platforms each AMI bundle covers.
- **Transaction model**: same `AttachModel` operation.
- **MCP API additions**: `attach_ami { part_uuid, ami_path,
  binaries: HashMap<Platform, String> }`. **No AMI execution MCP
  tool** — Datum should not host vendor `.dll` execution.
- **Minimum viable**: attachment-only, attach-and-validate, no
  execution. Effort: 1 week, mostly file-handling.
- **Full implementation**: AMI execution under a sandboxed FFI
  bridge — **explicitly out of scope** for Datum v1. Vendor `.dll`s
  are full native code with arbitrary side effects; sandboxing
  them properly is research-grade work, not implementation work.

**Strategic recommendation.** **Out of scope for v1.** Attachment
is fine if and only if implemented as the same `ModelAttachment`
type used for IBIS and Touchstone; do not build any AMI-specific
IR. Refer users who need AMI execution to Cadence Sigrity / Mentor
HyperLynx / Keysight ADS as the appropriate downstream tools, with
Datum providing the model-attachment hand-off.

**Risks and edge cases.**
- Vendor AMI binaries are platform-specific. Datum's pool needs to
  track platform = (OS, arch) tuples explicitly so a Linux
  user does not silently get handed Windows AMI.
- Some vendor AMI binaries require licence-server check-out
  (FlexLM). This is the vendor's problem, not Datum's.
- AMI execution is a real attack surface — running an arbitrary
  vendor `.so` in the engine process is a security hazard. If
  Datum ever adds AMI execution, do it in a separate sandboxed
  process.

#### IBIS-ISS

**Full title.** *IBIS Interconnect SPICE Subcircuit*, defined as a
companion specification to IBIS (originally BIRD-114 in IBIS 5.1,
formalised in subsequent revisions; current as of IBIS 7.x).
Sometimes referred to as "IBIS-ISS" or just "ISS".

**Issuing body.** IBIS Open Forum.

**Scope.** ISS defines a constrained SPICE subcircuit syntax for
**interconnect modelling** (transmission lines, vias, package
parasitics, connectors) that is portable across all major
SPICE simulators. Effectively, ISS is the "SPICE intersection set"
plus an IBIS-Open-Forum-blessed parameter naming convention.
Used to attach parasitic interconnect models to IBIS buffer
models without resorting to a particular SPICE dialect.

**Adoption status (2026).** **Niche, growing.** Used primarily by
the same designers who use AMI. The Si Open Forum's IBIS-ISS
subcircuit library is small (~50 reference models) compared to
vendor SPICE distributions.

**License / IP.** Open, free download from `ibis.org`. Reference
implementation is part of `ibischk`'s extended validator.

**Reference implementations.** `ibischk` validates ISS attachments;
ngspice and Xyce both consume ISS subcircuits as plain SPICE.

**EDA tool support matrix:** ISS is consumed by tools that consume
IBIS — Altium, OrCAD, Mentor PADS / HyperLynx, Cadence Allegro /
Sigrity, KiCad / ngspice. Effectively the same matrix as IBIS,
since ISS is just an IBIS attachment mechanism.

**Datum current coverage.** **Blind Spot.**

**Implementation cost (Datum).** **Same machinery as IBIS** —
ISS is just a SPICE subcircuit format with a vendor-blessed
naming convention. Add `role: SpiceSubcircuit` to ModelAttachment
and tag with `iss_version: Option<String>`. Validation goes
through ngspice's syntax checker (subprocess invocation) since
ibischk can validate the IBIS-side attachment but cannot syntax-
check the SPICE itself.

**Strategic recommendation.** **Implement at the same time as IBIS.**
ISS attachment is essentially free if IBIS is done; the marginal
data-model cost is one enum variant.

**Risks and edge cases.** None beyond IBIS itself.

#### Touchstone 1.x / 2.x

**Full title.** *Touchstone File Format Specification*. Two
versions matter: **Touchstone 1.1** (IBIS Open Forum, 2002) and
**Touchstone 2.0** (IBIS Open Forum, 2009, the "modern" revision).
Also formalised as **IEEE Std 370** ("Standard for Electrical
Characterization of Printed Circuit Board and Related
Interconnects at Frequencies up to 50 GHz", 2020) in part — IEEE
370 cites Touchstone 2.0 as the data-exchange format.

**Issuing body.** **IBIS Open Forum**. The Touchstone format
originated at EEsof (Hewlett-Packard's microwave EDA tools group,
later Agilent, now Keysight) in the 1980s; the IBIS Open Forum
took over the maintenance role around 2001.

**Scope.** Touchstone is the **canonical S-parameter exchange
format** — vector network analyser measurements, electromagnetic
simulator outputs, and SI-tool extracted parasitic models all
serialise as `.s2p` (2-port), `.s4p` (4-port = differential), up
to `.sNp`. The format is plain ASCII with a header (`# GHZ S MA
R 50` = "frequency in GHz, S-parameters in magnitude/angle, 50Ω
reference") followed by frequency-and-data rows.

Touchstone 1.x is space-aligned columns, comments via `!`, no
section headers. Touchstone 2.0 adds `[Version]`, `[Number of
Ports]`, `[Network Data]`, `[Reference]`, and `[End]` blocks for
machine-friendly parsing; allows per-port reference impedance;
supports mixed-mode (differential + common-mode) port
configurations natively.

**Adoption status (2026).** **Mainstream and table-stakes for SI.**
Every SI workflow consumes Touchstone. Vendors of cables, connectors,
and interconnect ICs (Molex, Samtec, TE, Amphenol, Broadcom switches)
ship `.sNp` files for every product line. Touchstone 1.1 is by
far the more common in-the-wild format; Touchstone 2.0 is
preferred by tools that need machine-friendly parsing.

**License / IP.** **Open, royalty-free.** Free PDF from `ibis.org`.
Multiple BSD/MIT-licensed reference parsers.

**Reference implementations.**
- **scikit-rf** (Python, BSD-3) — the canonical scientific-Python
  S-parameter library. Reads/writes Touchstone 1 + 2, supports
  cascading networks, port renumbering, mixed-mode conversion,
  S-to-Y/Z/T conversion. The de-facto reference for any non-EDA
  tool that processes S-parameters.
- **`touchstone-rs`** (Rust, MIT) — pure-Rust Touchstone reader.
  Production-quality, AP1+AP2 read.
- **`PyMicrowave`** — older Python library, less popular than
  scikit-rf in 2026.
- **MATLAB RF Toolbox** — MathWorks; commercial; the original MATLAB
  Touchstone reader.
- **ngspice** has a built-in Touchstone reader for `.snp` import
  (`include` directive with `.snp` extension).

**EDA tool support matrix:**
- **Altium Designer** — Touchstone attachment as part library
  metadata; SimBeor integration consumes them.
- **OrCAD-PSpice** — Touchstone via PSpice's `.NPORT` convolution.
- **Cadence Allegro / Sigrity** — Touchstone import + extraction
  output.
- **Mentor PADS / HyperLynx** — Touchstone import + extraction.
- **Siemens Xpedition** — Touchstone via HyperLynx.
- **KiCad** — Touchstone is supported by ngspice (KiCad's sim
  backend); KiCad itself does not directly attach Touchstone to
  schematic symbols as of v8.
- **Eagle / Fusion** — none.
- **Horizon EDA** — none.
- **LibrePCB / DipTrace / EasyEDA** — none.

**SI/PI tool support matrix:**
- **Cadence Sigrity** — full Touchstone import / export; the
  de-facto extraction-result interchange format.
- **Mentor HyperLynx** — same.
- **Ansys SIwave + Q3D** — Touchstone export of S-parameter
  extraction.
- **Keysight ADS** — full Touchstone read/write with mixed-mode.
- **CST Studio Suite** — Touchstone import / export.
- **SimBeor** — Touchstone-first.
- **Polar Si9000** — Touchstone for impedance-extracted models.

**Datum current coverage.** **Blind Spot.**

**Implementation cost (Datum).**
- **Canonical IR**: same `ModelAttachment` type as IBIS, with
  `role: Touchstone`. Add `touchstone_ports: u32` and
  `frequency_range_hz: Option<(f64, f64)>` derived from the
  attached file for fast querying without re-parsing.
- **Pool**: pool stores `.sNp` files like any other model
  attachment.
- **Transaction model**: `AttachModel` operation already covers it.
- **MCP API additions**: `attach_touchstone { part_uuid, snp_path
  }`, `validate_touchstone { snp_path }` (returns port count,
  freq range, header validity), `extract_touchstone_summary
  { part_uuid }` (returns scalar summary: insertion loss at f1,
  return loss at f1, etc.).
- **Minimum viable**: attachment + parse-and-validate. Effort:
  ~3-5 days using `touchstone-rs`.
- **Full implementation**: + scalar summary extraction (1-2 weeks).

**Strategic recommendation.** **Implement first, before IBIS.**
Touchstone is the easiest credible Domain 2 capability — pure
Rust parser, zero external native deps, broad applicability,
clean grammar, no encryption complexity.

**Risks and edge cases.**
- Touchstone 1.x has loose syntax — some files in the wild violate
  the specification (column count off, mixed extension and content).
  `touchstone-rs` is robust to common cases; pathological files
  need defensive handling.
- Frequency ordering: by spec, ascending; some files in the wild
  are descending or unsorted. Validator should normalise.
- Mixed-mode S-parameter representation requires explicit port
  mapping; the spec is loose. Datum's validator should warn on
  ambiguous port mapping rather than guess.

#### SPICE family (SPICE3, PSpice, HSPICE, LTspice, ngspice, Xyce, Spectre)

This is one entry covering the SPICE landscape because the dialects
share enough syntax to discuss together but diverge enough to
require explicit handling.

##### Berkeley SPICE 2 / SPICE 3

**Full title.** *Simulation Program with Integrated Circuit
Emphasis*. **SPICE 2** (last release SPICE 2G6, 1983) was
FORTRAN; **SPICE 3** (last release SPICE 3F5, 1993) was a C
rewrite at UC Berkeley. SPICE 3F5 is the academic baseline that
every modern derivative branched from.

**Issuing body.** UC Berkeley EECS (academic; project effectively
ended 1993).

**Adoption status (2026).** **Legacy / academic.** No commercial
or open-source tool uses SPICE 3F5 unmodified; ngspice and Xyce
are the maintained inheritors.

**License.** BSD (UC Berkeley).

##### ngspice (open-source)

**Full title.** *ngspice — The Open Source Spice Simulator.*
Current release **45** (December 2025). Active development at
SourceForge.

**Scope.** Direct lineage from SPICE 3F5; adds modern device
models (BSIM 4.8, BSIM-CMG, HiSIM-HV, EKV), behavioural sources
(B-element), XSPICE code-model interface (CMI) for arbitrary
behavioural modelling, mixed-mode digital extensions, RF analyses,
and Verilog-A subset support via ADMS / OpenVAF.

**Adoption status (2026).** **Mainstream.** The de-facto
open-source SPICE. Bundled with KiCad as the default simulation
backend since KiCad 6 (2021); usable standalone via CLI or
shared-library. Extensive vendor model libraries available (TI's
PSpice models often work in ngspice with minor edits).

**License.** **GPL-3** (the C source) plus BSD (the SPICE 3F5
ancestor). The GPL-3 status means ngspice cannot be statically
linked into a non-GPL Datum binary; subprocess invocation is the
correct integration pattern.

**Reference implementation.** Itself; no third-party reference.

**EDA tool support matrix:**
- **KiCad** — ngspice is the bundled simulator (since v6).
- **Qucs/QucsStudio** — ngspice optional backend.
- **SystemModeler** — Wolfram, indirect.
- **Other PCB tools** — generally invoke ngspice via subprocess
  if at all.

##### LTspice (Analog Devices, formerly Linear Technology)

**Full title.** *LTspice XVII* (current as of 2026; LTspice IV is
the long-stable version users still cite; the LTspice 24 build is
the current free download).

**Issuing body.** Analog Devices (acquired Linear Technology
2017; LTspice was Linear's giveaway, retained free under ADI).

**Scope.** Closed-source SPICE simulator + schematic capture.
Adds proprietary `.LIB` syntax for Analog/Linear parts, fast
turbo-mode integration, behavioural source extensions
(`B`-source with arbitrary expressions), waveform viewer with
FFT, transient + AC + DC + noise + Monte Carlo + worst-case
analyses.

**Adoption status (2026).** **Mainstream-hobbyist + professional
analog**. Free download with no licence-key. The most-used SPICE
in the analog-IC and power-electronics community. Windows-native;
runs under Wine on Linux; macOS port available.

**License.** Proprietary, free-of-charge for personal/commercial
use under ADI's licence terms. Cannot be redistributed with
other software. The bundled vendor models for ADI/Linear parts
are typically distributed separately in encrypted form for
non-public parts.

**Reference implementation.** Itself.

##### HSPICE (Synopsys)

**Full title.** *HSPICE* (current version 2024.x).

**Issuing body.** Synopsys (acquired Avant! / Meta-Software 1996;
HSPICE was Meta-Software's high-end SPICE).

**Scope.** Industry-standard analog/mixed-signal SPICE for
**IC design**, not PCB design. Adds proprietary `.OPTIONS`
extensions, Monte Carlo macros, statistical corner analyses,
RF analyses (HBOSC, TDR), Verilog-A integration, encrypted-model
support (AvantHash and successors).

**Adoption status (2026).** **Mainstream IC, niche PCB.**
The IC industry runs on HSPICE; PCB engineers rarely encounter
it directly except when ingesting vendor IC behavioural models
distributed in HSPICE syntax.

**License.** Proprietary, paid (~$25K-$75K/seat/year). Encrypted
model support is a major Synopsys feature; AvantHash is the
current encryption format.

##### PSpice (Cadence / OrCAD)

**Full title.** *PSpice A/D* (current as of OrCAD X 23.x).

**Issuing body.** Cadence Design Systems (acquired OrCAD 1999;
OrCAD acquired MicroSim's PSpice 1998).

**Scope.** SPICE-based simulator optimised for **PCB-side analog
schematic simulation**. Tight OrCAD Capture integration. Adds
PSpice-specific syntax (`.PROBE`, `.PARAM` with monte carlo
parametrics, encrypted models via `Cadence Encrypt-It`). The
syntax most closely matches Berkeley SPICE 3 with
PSpice-specific extensions.

**Adoption status (2026).** **Mainstream professional PCB
schematic simulation.** OrCAD-PSpice ships in OrCAD X PCB
Designer; Cadence Allegro X uses Sigrity for SI. Vendor model
libraries from TI, ADI, NXP, ST, Microchip routinely ship in
PSpice syntax.

**License.** Proprietary, paid (bundled with OrCAD subscription;
~$3K-$10K/seat/year).

##### Xyce (Sandia)

**Full title.** *Xyce — Parallel Electronic Simulator*. Current
version **7.10** (2024).

**Issuing body.** Sandia National Laboratories.

**Scope.** Open-source SPICE designed for **massively parallel
HPC simulation** of large mixed-signal circuits. SPICE 3
compatibility, BSIM models, Verilog-A via ADMS, XDM for migration
of HSPICE / PSpice / Spectre netlists into Xyce syntax.

**Adoption status (2026).** **Niche academic + DOE; emerging
in HPC-EDA contexts.** Notable for handling million-device
netlists that ngspice / LTspice cannot.

**License.** GPL-3 + parallel BSD components; same GPL-3
contagion concerns as ngspice for Datum integration.

##### Spectre (Cadence)

**Full title.** *Cadence Spectre Circuit Simulator*. Current
release **23.1**.

**Issuing body.** Cadence Design Systems.

**Scope.** Cadence's IC-design SPICE; `.scs` syntax, Verilog-AMS
integration, Spectre-specific RF analyses (PSS, PNoise).

**Adoption status (2026).** **IC industry standard; effectively
absent in PCB.** PCB engineers ingest Spectre models only when
working with Cadence-distributed semicon vendor parts.

**License.** Proprietary, paid (Cadence Virtuoso bundle).

##### SPICE family — pooled analysis

**Datum current coverage.** **Blind Spot.** Eagle SPICE imports
deferred (`specs/IMPORT_SPEC.md:210`); no SPICE handling in any
spec.

**Implementation cost (Datum).**
- **Canonical IR**: `ModelAttachment` with `role: Spice`,
  `dialect: SpiceDialect { Berkeley3, Ngspice, LTspice, PSpice,
  HSpice, Xyce, Spectre }`, `model_names: Vec<String>`
  (extracted from `.MODEL` and `.SUBCKT` lines).
- **Pool**: pool stores `.cir` / `.lib` / `.sub` / `.inc` files
  with the same UUID-from-SHA256 mechanism.
- **Transaction model**: `AttachModel` covers it.
- **MCP API additions**: `attach_spice { part_uuid, file_path,
  dialect }`, `validate_spice { file_path, dialect }` (subprocess
  to ngspice in syntax-only mode), `extract_spice_subckt_pin_list
  { part_uuid, subckt_name }` (parses the `.SUBCKT` declaration
  to return port-list ordering).
- **Minimum viable**: attachment + ngspice-syntax-check (subprocess).
  Effort: ~1 week.
- **Full implementation**: + cross-dialect normalisation (PSpice
  → ngspice translation tables for `.PROBE`, `.OPTIONS` etc.) —
  effort 4-6 weeks, value lower than alternatives.

**Strategic recommendation.** **Implement post-IBIS, before AMI.**
SPICE attachment + ngspice validation is a one-week sprint; the
cross-dialect normalisation work is much lower-value than
capturing an IBIS surface. Recommend: implement attachment +
syntax-check; emit ngspice as the canonical dialect for
auto-generated stimulus; do not normalise vendor SPICE between
dialects (let the user select target dialect at export time).

**Risks and edge cases.**
- PSpice models may use encrypted blocks (`*PSpice` directive +
  encryption); Datum's parser must treat these as opaque.
- LTspice `.MODEL` syntax is a strict subset of PSpice; cross-use
  works for simple parts and breaks on advanced modelling. Datum
  should not silently convert.
- HSPICE-specific options (`.TEMP`, `.DCMATCH`) are not
  ngspice-compatible; if Datum exports a stimulus deck against an
  HSPICE-syntax vendor model, the user must invoke HSPICE.

#### Verilog-A / Verilog-AMS

**Full title.** *Verilog-A* (analog-only subset of Verilog-AMS);
*Verilog-AMS* (analog and mixed-signal extensions to Verilog).
Current standard: **Verilog-AMS 2.4** (Accellera, 2014; the
"current" reference; no later major revision).

**Issuing body.** **Accellera Systems Initiative**. Verilog
proper (digital) is **IEEE 1364** (last revision IEEE 1364-2005,
superseded by SystemVerilog IEEE 1800).

**Scope.** Verilog-A is the analog HDL subset — used to author
**compact device models** (e.g., custom MOSFET model, custom
memristor model) for SPICE-class simulators. Verilog-AMS adds
mixed-signal connect modules between digital and analog domains.

**Adoption status (2026).** **Niche; the standard authoring
language for new compact models.** All modern device-model work
(BSIM-CMG for FinFETs, MVS for III-V devices, memristor, organic
semiconductor) ships in Verilog-A. Translated to per-simulator
C-code via **ADMS** (Accellera Verilog-AMS Synthesiser, the de-
facto Verilog-A → SPICE-C compiler) or the newer **OpenVAF**
(MIT-licensed, Rust-implemented Verilog-A compiler, 2022+).

**License / IP.** Open Accellera spec, free PDF download.

**Reference implementations.**
- **ADMS** (BSD) — Verilog-A → SPICE-C translator. Used by ngspice
  for Verilog-A model integration. Aging.
- **OpenVAF** (MIT, Rust) — modern Verilog-A compiler from the
  Verilog-A Initiative. Cleaner architecture, faster compilation,
  active development.
- **Synopsys VCS, Cadence Xcelium, Mentor Questa** — commercial
  Verilog-AMS hosts; expensive and IC-focused.

**EDA tool support matrix:** Verilog-A is consumed by SPICE-class
simulators with VA support (ngspice via ADMS, Xyce via ADMS,
HSPICE native, Spectre native, LTspice via vendor `.subckt`
wrapping). PCB tools do not consume Verilog-A directly; they
consume the resulting SPICE / behavioural model.

**SI/PI tool support matrix:** Verilog-AMS is used in mixed-signal
IC SI workflows (Cadence Spectre + Xcelium + Allegro). PCB-side
SI does not touch Verilog-AMS directly.

**Datum current coverage.** **Blind Spot.**

**Implementation cost (Datum).** Same `ModelAttachment` machinery
with `role: VerilogA` / `VerilogAMS`. No simulator integration —
let users pre-compile Verilog-A to SPICE/native via OpenVAF
externally and attach the result. Effort: 1-2 days incremental
on top of SPICE attachment.

**Strategic recommendation.** **On-demand only.** Verilog-A is too
niche for PCB users to justify in v1; if a customer has a
specific Verilog-A model they need to attach, the
`ModelAttachment` mechanism handles it without engine changes.

**Risks and edge cases.** None significant.

#### VHDL-AMS

**Full title.** *IEEE Std 1076.1-2017* (current revision; the
analog/mixed-signal extension to VHDL).

**Issuing body.** IEEE.

**Scope.** Analog / mixed-signal extension to VHDL, parallel to
Verilog-AMS for the VHDL ecosystem. Used in some European
automotive / aerospace IC modelling workflows; rare in PCB
design.

**Adoption status (2026).** **Niche.** Smaller userbase than
Verilog-AMS even within the mixed-signal IC modelling community.

**License / IP.** IEEE standard; PDF requires IEEE Xplore
purchase ($217 single-user).

**Reference implementations.**
- **Cadence AMS Designer** — VHDL-AMS support.
- **Mentor Questa-AMS** — VHDL-AMS support.
- **Saber** (Synopsys, formerly Avant!) — historically VHDL-AMS
  + MAST. Still sold; primarily aerospace / automotive.
- **GHDL** (open-source GPL VHDL simulator) — partial
  VHDL-AMS support, not production-quality.

**EDA tool support matrix:** Same notes as Verilog-AMS — IC tool
support only.

**Datum current coverage.** **Blind Spot.**

**Implementation cost (Datum).** Same `ModelAttachment` with
`role: VhdlAms`. Effort: trivial.

**Strategic recommendation.** **Out of scope for v1; on-demand
attachment only.** The audit Phase 1 already classified VHDL-AMS
as "skip" priority and that classification holds.

**Risks and edge cases.** None for an attachment-only path.

#### MAST (Saber / Synopsys)

Brief mention only per scope. **MAST** is the proprietary modelling
language used by Synopsys Saber (originally Analogy, acquired by
Synopsys 2004). Saber is a multi-domain simulator (electrical +
mechanical + hydraulic + thermal) used in automotive and aerospace.
MAST is closed; no open parser; no reason for Datum to touch it.
**Out of scope.**

### Digital HDL Models

#### Verilog (IEEE 1364) / SystemVerilog (IEEE 1800)

**Full title.** *Verilog Hardware Description Language*: IEEE
1364-1995, 2001, 2005 (last standalone revision). *SystemVerilog*:
IEEE 1800-2017 (current; supersedes IEEE 1364).

**Issuing body.** IEEE.

**Scope.** RTL / behavioural HDL for digital ASIC and FPGA
design. Outputs synthesised gates after PAR; the resulting
**netlist + IO pin map** is what crosses into PCB territory, not
the HDL itself.

**Adoption status (2026).** **Mainstream digital design;
essentially absent in PCB schematic capture.** Datum's universe
(PCB) does not author or consume Verilog directly.

**License / IP.** IEEE standards.

**EDA tool support matrix:** IC tool only — Synopsys VCS, Cadence
Xcelium, Siemens Questa, FPGA vendor toolchains (Xilinx Vivado,
Intel Quartus, Lattice Diamond, Microchip Libero). PCB tools do
not consume Verilog.

**Datum current coverage.** **Blind Spot, but recommend Out of
Scope.** Verilog is FPGA / ASIC tooling; the PCB-relevant artefact
is the post-PAR pin map.

**Implementation cost (Datum).** N/A; recommended out of scope.

**Strategic recommendation.** **Out of scope.** Recommend formal
exclusion for HDL languages themselves; the FPGA pin-mapping
consumption surface (XDC, QSF, LPF) is a Domain 4 industry-
vertical concern, not Domain 2 component-modelling.

#### VHDL (IEEE 1076)

Same posture as Verilog. IEEE 1076-2019 is current. **Out of
scope for Datum v1**; recommend formal exclusion.

#### EDIF for HDL exchange

**Full title.** *Electronic Design Interchange Format* — version
**2 0 0** (1988, the historical mainstream) and version **4 0 0**
(1996, the gate-level / ASIC variant).

**Issuing body.** Originally EIA / EDIF Steering Committee; now
**IEC 61691-3** for EDIF 2 0 0.

**Scope.** Netlist exchange between EDA tools. EDIF 2 0 0 carried
schematic + symbol data; EDIF 4 0 0 added ASIC gate-level
netlists with timing.

**Adoption status (2026).** **Effectively dead.** No modern tool
exercises EDIF in production. The data-exchange research
(`research/data-exchange-interop/DATA_EXCHANGE_INTEROP_RESEARCH.md`)
already classified EDIF as "no migration value".

**License / IP.** IEC standard; PDF download paid.

**Datum current coverage.** Already classified Out of Scope in the
data-exchange Phase 2 research. **Re-affirmed here.**

**Strategic recommendation.** **Out of scope.** No action needed.

#### JEDEC programmable-logic .jed (note only)

**Full title.** *JESD3-C JEDEC Standard for Programmable Logic
Device Programming Patterns*. Defines the `.jed` file for CPLD
programming (fuse map). FPGA bitstreams are vendor-proprietary
(`.bit` for Xilinx, `.bin` for Lattice/iCE40, `.sof` for Intel/
Altera).

**Adoption status (2026).** **Legacy CPLD only.** New designs use
FPGAs with vendor-proprietary bitstream formats. `.jed` is rarely
encountered in 2026 PCB workflows except in long-running legacy
designs.

**Datum current coverage.** **Blind Spot, but recommend Out of
Scope.** PCB tools do not consume `.jed` files; assembly /
programming workflows do.

**Strategic recommendation.** **Out of scope.** No action needed.

### Thermal & Mechanical Component Models

#### Two-resistor thermal model (DELPHI / JEDEC JESD15-3)

**Full title.** *JESD15-3 Two-Resistor Compact Thermal Model
Guideline* (JEDEC, 2008; still current as of 2026). The **DELPHI**
project (DEvelopment of Libraries of PHysical models for an
Integrated design environment), funded by the EU in the late
1990s, established the underlying methodology; JESD15-3 codified
it for JEDEC.

**Issuing body.** JEDEC.

**Scope.** A **two-resistor thermal compact model** describes a
component as a single junction node connected to two ambient
nodes via thermal resistances:
- **Θ_JA (junction-to-ambient)** — single resistance to the
  surrounding still air (reference: 25°C, 1 W dissipation,
  JEDEC JESD51-2 still-air chamber).
- **Θ_JC (junction-to-case)** — resistance from junction to the
  package top surface (reference: JESD51-1 cold-plate).
- Sometimes augmented with **Θ_JB (junction-to-board)** — the
  third "two-resistor" variant, giving three R values.

These three numbers appear on essentially every modern IC
datasheet's "Thermal Information" table. They are the input most
PCB designers actually use for thermal sanity checks.

**Adoption status (2026).** **Universal datasheet adoption,
limited tool ingestion.** Every IC datasheet publishes Θ values;
no PCB schematic tool surveyed automatically pulls them into a
Part record. Altium has a free-form "Thermal" parameter set;
Cadence Allegro consumes them in Sigrity for thermal sims; KiCad
has nothing.

**License / IP.** JEDEC standard. JEDEC standards have a "free
download after registration" policy — JESD15-3 is free PDF after
filling out a JEDEC member-or-not form at `jedec.org`.

**Reference implementations.** No open parser library; the data
is plain numbers in a datasheet PDF or vendor-supplied spreadsheet.

**EDA tool support matrix:**
- **Altium / Cadence Allegro / Mentor PADS** — free-form Part
  parameters; user keys them in.
- **Cadence Sigrity Celsius** — thermal simulator; ingests the
  values from Allegro Part metadata.
- **Mentor FloTHERM** — thermal sim; consumes JESD15-3 + JESD15-4
  models.
- **Ansys IcePak** — same.
- **6SigmaET** — Future Facilities; same.
- **Datum** — none.

**Datum current coverage.** **Blind Spot.** No thermal fields on
`Part`.

**Implementation cost (Datum).**
- **Canonical IR**: extend `Part` with an
  `electrical_parameters: HashMap<String, ParameterValue>` field
  (general), or specifically add a `thermal: Option<ThermalSpec>`
  with `theta_ja_c_per_w: Option<f32>`,
  `theta_jc_top_c_per_w: Option<f32>`, `theta_jc_bot_c_per_w:
  Option<f32>`, `theta_jb_c_per_w: Option<f32>`,
  `max_junction_c: Option<f32>`. Recommend the latter (typed
  ThermalSpec) for AI-friendliness — a structured field is easier
  to query than a parametric-string entry.
- **Pool**: no change beyond Part record extension.
- **Transaction model**: standard Part-edit operations cover it.
- **MCP API additions**: nothing new — already covered by
  existing Part-edit operations. The AI surface gains the
  ability to query thermal data structurally.
- **Minimum viable**: just the four `Option<f32>` fields. Effort:
  ~1 day.
- **Full implementation**: same as MV. ~1 day.

**Strategic recommendation.** **Implement now (M7+ window).**
This is a near-zero-cost data-model extension with broad
applicability to the AI surface and to future thermal analysis.

**Risks and edge cases.**
- Datasheet Θ values depend on PCB / via configuration. The
  JEDEC reference is "single-layer 50mm × 50mm board, no airflow"
  — real values for a specific design differ. Datum's stored
  value should be the datasheet number with a free-text
  `thermal_reference` note ("JESD51-2 still-air, 1S board").
- Some vendors publish only Θ_JA; some publish a full table.
  Optional fields handle the partial case.

#### Compact thermal model networks (CTM / ECXML / JESD15-4)

**Full title.** *JESD15-4 DELPHI Compact Thermal Model Guideline*
(JEDEC, 2008). Codifies the multi-node DELPHI compact thermal
model (junction node + multiple surface nodes, R-network between
them). **ECXML** (Electronic Cooling XML) is the open XML schema
for CTM exchange.

**Issuing body.** JEDEC (JESD15-4); ECXML is an open community
schema.

**Scope.** Multi-node thermal network (typically junction +
top-surface + bottom-surface + 4 side-surfaces, R-matrix between
all pairs). Used by professional thermal CFD tools (FloTHERM,
IcePak, 6SigmaET) for high-fidelity thermal simulation of
high-power components.

**Adoption status (2026).** **Niche.** Used by the same designer
subset that uses AMI — high-power-density designs, thermally-
constrained designs, automotive power electronics. Vendor
distribution is limited; only the highest-volume power parts
(TI's high-power LDOs, Infineon's power MOSFETs, NXP's BMS
chips) ship CTM.

**License / IP.** JEDEC standard. ECXML schema is open.

**Reference implementations.**
- **Mentor FloTHERM ECXML import** — flagship.
- **Ansys IcePak ECXML import**.
- **6SigmaET native CTM import**.
- No open-source ECXML parser of note.

**EDA tool support matrix:** Thermal-CFD tools only; not consumed
by PCB schematic / layout tools.

**Datum current coverage.** **Blind Spot.**

**Implementation cost (Datum).** Same `ModelAttachment` machinery
with `role: CompactThermal`. Effort: ~1 day for attachment-only.

**Strategic recommendation.** **On-demand only.** Attachment
mechanism handles it; do not invest in parsing or simulation
integration in v1.

**Risks and edge cases.** None for attachment-only.

#### STEP MCAD bodies (cross-ref Domain 1)

**Cross-reference.** Already deep-dived in
`research/data-exchange-interop/DATA_EXCHANGE_INTEROP_RESEARCH.md`
§ "ECAD↔MCAD Exchange / STEP AP242 / STEP AP203 / STEP AP214".
Do not duplicate.

**Domain 2 footprint:** STEP files attached to a Package's
`models_3d: Vec<ModelRef>` field
(`specs/ENGINE_SPEC.md:74-77,138`). The Domain 1 deep-dive
already recommended extending `ModelRef` with a typed
`Transform3D` and `format: ModelFormat` enum. The Domain 1
recommendation also added `Package.body_height_nm` for IDF 3.0
export — that field doubles as the lightweight thermal /
clearance height needed for footprint thermal/clearance work.

**Domain 2-specific note:** When STEP files carry encrypted
sub-assemblies (rare but possible in vendor-distributed STEP
for IP-protected mechanical parts), the same encrypted-model
gate that applies to IBIS / SPICE applies. Datum's ModelRef
should grow `encrypted: bool` to match `ModelAttachment`.

**Strategic recommendation.** **Implement post-M7** per the
Domain 1 recommendation. No additional Domain 2 implementation
work needed.

#### IDF component height/keepout (cross-ref Domain 1)

**Cross-reference.** Already deep-dived in
`research/data-exchange-interop/DATA_EXCHANGE_INTEROP_RESEARCH.md`
§ "ECAD↔MCAD Exchange / IDF". Do not duplicate.

**Domain 2 footprint:** IDF 3.0 component records carry
height (single number, mm) and a per-component keepout polygon
(top + bottom). Datum's `Package.body_height_nm` recommendation
from Domain 1 originated in the footprint thermal/clearance
work and is the right field to populate for IDF export.

**Strategic recommendation.** **Implement post-M7** per the
Domain 1 recommendation.

### Component Metadata & Parametric Data

#### Octopart / Nexar API

**Full title.** *Nexar Component Search API* (formerly
**Octopart Search API**; rebranded after Altium acquired
Octopart in 2015 and re-positioned the Nexar brand around 2020).

**Issuing body.** Altium Limited (now Renesas, since the 2024
acquisition).

**Scope.** **GraphQL REST API** providing unified search across
~50 distributor catalogs (Digi-Key, Mouser, Arrow, Avnet, Newark,
LCSC, RS, Farnell, etc.). Each query returns:
- canonical part record (MPN, manufacturer, lifecycle, datasheet
  URL, RoHS / REACH compliance flags),
- distributor offers (price breaks, stock, lead time, MOQ),
- specifications (parametric attributes harvested from datasheet),
- cross-references (alternate parts, equivalents, predecessors),
- CAD-model availability (SnapEDA, UltraLibrarian, Component
  Search Engine links).

**Adoption status (2026).** **Mainstream.** The dominant
component-data API. Altium uses Nexar internally; KiCad has a
Nexar-fed plugin community; integration is widespread.

**License / IP.**
- **Free tier**: 1000 queries/day, no commercial use beyond
  internal evaluation, attribution required ("powered by Nexar").
- **Paid tier (Altium Designer subscription)**: 10K queries/day
  + commercial use bundled.
- **Enterprise**: negotiated.

**Reference implementations.**
- **`nexar-graphql-py`** (Python, MIT) — community Nexar client.
- **`nexar-rs`** (Rust, MIT) — community client.
- Nexar's own JavaScript / TypeScript SDK (their preferred path).

**EDA tool support matrix:**
- **Altium Designer** — native Nexar integration in the
  Manufacturer Part Search panel.
- **OrCAD-PSpice** — via the Cadence Connect partner program.
- **KiCad** — community Nexar plugin (KiCad-Octopart-Plugin
  fork).
- **Eagle / Fusion** — Eagle had a deprecated Octopart link;
  Fusion has Component Library integration.
- **Horizon EDA / LibrePCB / DipTrace / EasyEDA** — none native.

**Datum current coverage.** **Blind Spot.** `Part.orderable_mpns`
exists (`specs/ENGINE_SPEC.md:158`) but no API integration to
populate it.

**Implementation cost (Datum).**
- **Canonical IR**: extend `Part` with optional supply-chain cache
  fields: `last_supply_chain_check: Option<DateTime>`,
  `supply_chain_offers: Option<Vec<SupplyOffer>>` where
  `SupplyOffer = { distributor: String, price_breaks: Vec<(u32,
  f64, String)>, stock: Option<u32>, lead_time_weeks: Option<u32>,
  link: String }`. Mark these as derived (cache-only, never
  authoritative).
- **Pool**: pool index could cache supply-chain offers per Part
  for faster querying.
- **Transaction model**: no change — supply-chain refresh is a
  derived-data operation, not an authored-data change.
- **MCP API additions**: `lookup_part_octopart { mpn:
  String, manufacturer: Option<String> }` returning canonical
  record; `refresh_supply_chain { part_uuid }` triggering
  external API fetch and cache update; `find_alternate_parts
  { part_uuid }` returning Octopart cross-reference list.
- **Minimum viable**: `lookup_part_octopart` MCP tool that
  proxies a Nexar query and returns the canonical record.
  Effort: ~3-5 days (Nexar GraphQL client + MCP wrapper).
- **Full implementation**: + caching + alternates + lifecycle
  refresh + BOM-export enrichment. Effort: ~3 weeks.
- **Partner / library dependencies**: Nexar API key (user-supplied
  at Datum runtime, not bundled); `nexar-rs` or hand-rolled
  GraphQL client.

**Strategic recommendation.** **Implement post-M7 as on-demand
MCP tool.** Keep API key user-supplied; never bundle a Datum-
owned key. The AI agent can use this to enrich BOMs, suggest
alternates, flag EOL parts.

**Risks and edge cases.**
- Free-tier rate limits will bite power users; document that
  upgrading to paid Nexar is the user's responsibility.
- Octopart's parametric harvesting is best-effort; the data
  carries datasheet-extraction noise. Datum should not blindly
  overwrite authored Part parametrics with Octopart data; treat
  Octopart data as a separate `octopart_parametrics` cache.
- API responses include URLs to vendor datasheet PDFs; Datum
  should not cache the PDF content (storage + IP) but can cache
  the URL and record the URL freshness time.

#### Distributor APIs (Digi-Key, Mouser, Arrow, Avnet)

**Brief survey** of the four big distributor direct APIs:

##### Digi-Key API

- **API style**: REST/JSON, OAuth 2.0 auth.
- **Free tier**: yes, 1000 queries/day.
- **Coverage**: Digi-Key catalog only (~6M+ parts).
- **Reference**: `digikey-api` (Python, MIT); Digi-Key official
  SDK in JavaScript / Python / C#.
- **Use case**: direct Digi-Key pricing/stock without going
  through Octopart's aggregation layer.

##### Mouser API

- **API style**: REST/JSON, API-key auth.
- **Free tier**: yes, 1000 queries/day.
- **Coverage**: Mouser catalog (~6M+ parts).
- **Reference**: `mouser-api` (Python community, MIT).

##### Arrow API

- **API style**: REST/JSON, OAuth.
- **Free tier**: limited; mostly enterprise-focused.
- **Coverage**: Arrow catalog (broad EU / global).
- **Reference**: closed; Arrow's own SDK on request.

##### Avnet API

- **API style**: REST/JSON, OAuth.
- **Free tier**: limited.
- **Coverage**: Avnet catalog.
- **Reference**: closed; SDK by registration.

**Common pattern.** Each distributor API is shaped similarly:
search-by-MPN returns part record + price breaks + stock + lead
time. The aggregator (Octopart/Nexar) is more useful when the
goal is "find any distributor with stock"; direct APIs are
better when the user has a preferred distributor relationship.

**Datum implementation cost.** Each direct distributor MCP tool
is ~2 days (REST client + auth + JSON parse). Recommend shipping
**Octopart/Nexar as primary** + **Digi-Key + Mouser as direct
fallbacks** (the two largest distributors by hobbyist + small-
business volume), and **on-demand Arrow + Avnet** for enterprise
users.

#### SiliconExpert API (paid)

**Brief.** SiliconExpert (now part of Arrow) provides
**lifecycle / compliance / risk** data — predicted EOL dates,
component obsolescence risk scores, RoHS / REACH / SCIP
compliance attestation, conflict-mineral provenance. **Paid
only**, ~$10K-$50K/seat/year. Used by aerospace / defence /
medical / automotive teams that need formal compliance evidence.

**Datum coverage.** **Out of scope for v1.** Mark as on-demand
integration if a customer asks. Same MCP shape as
Octopart/Nexar; just a different backend.

#### CIS / database library standards

**Full title.** *Component Information System*. The umbrella
term for the integration pattern where an EDA tool library is
backed by an external SQL database (typically MS-SQL or ODBC),
allowing the company's parts database to be the single source of
truth across Engineering, Procurement, and Documentation.

**Implementations.**
- **Cadence CIS** (OrCAD CIS) — the original; Cadence's CIS
  has driven OrCAD library workflows since the 1990s.
- **Altium Database Libraries (DBLib) / SVN DBLib** — Altium's
  ODBC-bridge libraries; SVN DBLib adds revision control.
- **KiCad Database Libraries (KiCad 7+)** — newer; ODBC-based,
  inspired by Altium DBLib.
- **Pulsonix DataBaseConnect** — same idea.

**Adoption status (2026).** **Mainstream in mid-to-large
enterprise.** Hobbyist / small-shop users do not use CIS. Larger
teams treat CIS-as-source-of-truth as a hard requirement.

**License / IP.** Implementation-specific; the underlying
standard is "use ODBC and a vendor-defined column-mapping
schema". No formal standard.

**Datum coverage.** **Blind Spot.** Datum's pool is the analog
of a CIS — file-based JSON instead of SQL-backed. The pool
already supports the right semantics (project + shared + base
priority layering, lifecycle metadata, parametrics).

**Implementation cost (Datum).** Two paths:
1. **CIS bridge: read** — implement an ODBC client that pulls
   external Part metadata into the pool on demand. Effort:
   ~3-4 weeks. Requires per-customer column-mapping configuration.
2. **CIS bridge: write** — emit Datum Part data as ODBC inserts
   into a customer's existing CIS. Effort: ~3 weeks.

**Strategic recommendation.** **On-demand only for v1.** Datum's
file-based pool is already a good "CIS-ish" alternative; full
CIS bridge is enterprise integration work that should be
customer-funded.

#### Manufacturer datasheet PDFs

**Brief survey.** PDF datasheet extraction is an active area in
2026 with several commercial offerings (SnapEDA's Datasheet
Parser, Octopart's Datasheet Spec API, ECIA-curated APIs) and
emerging open-source LLM-driven approaches. The field is too
young for any standard; Datum should treat datasheet URLs as
URLs (already covered by `Part.datasheet`) and let the AI
surface optionally extract parameters via vision-LLM tooling
that already exists outside Datum.

**Strategic recommendation.** **Out of scope for engine
implementation; in scope for AI-surface usage.** The MCP layer
can call out to a vision LLM (Anthropic / OpenAI / etc.) to
extract parameters from a datasheet PDF; Datum stores the
result as Part parametrics with an explicit
`provenance: "datasheet-extracted"` flag.

#### JEDEC JEP106 (Manufacturer ID codes)

**Full title.** *JEP106BL — Standard Manufacturer's
Identification Code* (JEDEC, current revision **JEP106BU**
2024). Older revs commonly cited.

**Issuing body.** JEDEC.

**Scope.** Standard 8-bit (continuation-prefixed for higher)
manufacturer ID codes. TI = `0x97`, NXP = `0x15`, Microchip =
`0x29`, Atmel (now Microchip) = `0x1F`, ADI = `0x01`, ST = `0x20`,
Renesas = `0x53`, Bosch = `0xC1`, etc. Used in JEDEC SPI flash
and SDR/DDR memory device IDs (returned by hardware "READ ID"
commands), and as a canonical manufacturer-name mapping table.

**Adoption status (2026).** **Universal in firmware / BSP
contexts; rarely surfaced in PCB tools.** Useful as a canonical
manufacturer-name normalisation reference for Datum's pool —
"Texas Instruments", "Texas Instruments Inc.", "TI" all
collapse to `0x97`.

**License / IP.** **Free PDF download from JEDEC after
registration.** Updated periodically; Datum should bundle a
snapshot at build time.

**Reference implementations.**
- **`jedec-id`** (Python, MIT) — JEP106 lookup table.
- **`spi-nor`** Linux kernel database — informal cross-reference.

**Datum current coverage.** **Blind Spot.** `Part.manufacturer:
String` is free-text.

**Implementation cost (Datum).**
- **Canonical IR**: add `manufacturer_jep106: Option<u16>` field
  to `Part` (use u16 to allow continuation-prefixed extensions).
- **Pool**: pool ships a static `jep106.json` lookup table.
- **Transaction model**: no change.
- **MCP API additions**: `normalize_manufacturer { name: String
  }` returning `(jep106_code, canonical_name)`.
- **Minimum viable + full implementation**: ~1 day. Static
  lookup table + tiny normalisation function.

**Strategic recommendation.** **Implement now.** Trivial cost,
broad value as a manufacturer-name canonicalisation source.

**Risks and edge cases.**
- JEDEC continuation prefix bytes (e.g., `0x7F 0x7F 0xFE` =
  Cirrus) make IDs >8 bits in practice. Use u16 to cover
  realistic depths.
- Mergers and acquisitions break manufacturer continuity (Linear
  Tech → ADI; Atmel → Microchip; Maxim → ADI). Datum's
  normalisation table should map M&A predecessors to current
  owners.

#### EIA-481 reel/tape

**Full title.** *EIA-481-D-2016 Embossed Carrier Taping for
Surface Mount Components Reels Used for Automatic Handling*
(EIA / ECIA, current revision 2016). Subsequent maintenance
under ECIA.

**Issuing body.** EIA / ECIA.

**Scope.** **Tape-and-reel packaging dimensions** for SMD
components — tape pitch (4mm, 8mm, 12mm, 16mm, 24mm, ...),
reel diameter (7", 13", 15"), per-component pocket dimensions,
sprocket-hole spacing. The standard the assembly machine
(P&P) consumes when configuring a feeder.

**Adoption status (2026).** **Universal.** Every distributor
publishes EIA-481-compliant reel data per part; assembly
houses live on it.

**License / IP.** Paid PDF (~$130 from ECIA).

**Reference implementations.** No open library; data is
embedded in distributor APIs (Digi-Key / Mouser packaging
fields).

**Datum current coverage.** **Blind Spot.** No packaging fields
on Part.

**Implementation cost (Datum).**
- **Canonical IR**: extend `Part` with `packaging_options:
  Vec<PackagingOption>` where `PackagingOption = { kind:
  PackagingKind { Reel { tape_width_mm, reel_diameter_inch,
  qty_per_reel }, Tray { qty_per_tray }, Tube { qty_per_tube
  }, Bag { qty_per_bag } }, mpn_suffix: Option<String> }`.
  Many parts have multiple packaging suffixes (e.g., TI's `R`
  / `T` / `RA` packaging suffixes).
- **Pool**: no change beyond Part record.
- **Transaction model**: standard Part-edit ops.
- **MCP API additions**: `query_packaging_options { mpn }`
  returning the list — populated from Octopart/Nexar data.
- **Minimum viable**: data-model fields only, no automated
  population. Effort: ~1 day.
- **Full implementation**: + Octopart fetch. Effort: ~1 week
  on top of Octopart integration.

**Strategic recommendation.** **Implement data fields now;
populate via Octopart integration when that lands.** Useful
for assembly-house BOM / pick-and-place export workflows
already in Datum's roadmap.

**Risks and edge cases.** None significant.

### Pin / Symbol Modelling

#### IEEE 991 / IEC 60617 pin attributes

**Full titles.**
- *IEEE 991-1986 — IEEE Standard for Logic Circuit Diagrams*.
  Withdrawn / not actively maintained, but still cited.
- *IEC 60617 — Graphical symbols for diagrams*. Active; multiple
  parts (60617-2 onwards), with online database
  (`std.iec.ch/iec60617`).

**Issuing bodies.** IEEE (991), IEC (60617).

**Scope.** Schematic graphic-symbol drawing conventions. IEEE 991
codifies the US logic-symbol style (rectangular gates with
qualifying symbols); IEC 60617 codifies the international
graphic-symbol library used in EU and Asian schematics. Neither
standard codifies **pin direction vocabulary** the way IBIS
does.

**Adoption status (2026).** **Advisory.** Every PCB tool ships
its own symbol library; the standards are typically used as
reference rather than enforced. Datum's Phase 1 audit already
noted (STANDARDS_AUDIT.md lines 264-267) that Datum has no
normative position on US/IEEE-315 vs IEC-60617 graphic style;
that decision rightly belongs to Domain 3 (schematic & drawing
conventions), not Domain 2.

**License / IP.** IEEE 991 free download (withdrawn standard;
PDF available from IEEE Xplore for archival reasons). IEC 60617
is paid (online subscription, ~CHF 600/year for the database).

**Datum current coverage.** **Pin-direction enum is complete and
matches industry vocabulary.**
`PinElectricalType { Input, Output, Bidirectional, Passive,
TriState, OpenCollector, OpenEmitter, PowerIn, PowerOut,
NoConnect }` (`specs/ERC_SPEC.md:31-44`,
`specs/ENGINE_SPEC.md:32-43`) is the standard vocabulary every
EDA tool uses, originated in the 1990s SPICE / EDIF era.

**Implementation cost (Datum).** **Zero for Domain 2.** The
graphic-symbol-style decision belongs to Domain 3.

**Strategic recommendation.** **No Domain 2 action needed.**
Defer graphic-style policy to Domain 3 deep-dive.

#### Pin-naming conventions (differential, bus, power)

**Brief survey.** Industry-de-facto conventions, not formal
standards:
- **Differential pairs**: `_P` / `_N` suffix is dominant; `_DP`
  / `_DM` (USB, MIPI), `+` / `-` (some legacy), `_PLUS` /
  `_MINUS` (rare).
- **Bus pins**: `D[7:0]`, `D[0..7]`, `D7..D0` — all in active use;
  KiCad supports both `[]` and `[..]` syntax; Datum's bus
  handling
  (`specs/SCHEMATIC_CONNECTIVITY_SPEC.md:104-117`) covers these.
- **Power pins**: `VCC`, `VDD`, `VSS`, `VEE`, `GND`, `AGND`,
  `DGND`, `VBUS` — informal but universal. Sometimes followed
  by analog/digital domain suffix (`VCC_3V3`, `VDD_CORE`).

**Adoption status (2026).** **De-facto across all tools.** No
ISO/IEEE standard; every tool's connectivity engine handles
these conventions.

**Datum current coverage.** **Bus syntax already covered**
(`specs/SCHEMATIC_CONNECTIVITY_SPEC.md:104-117`); differential
pair handling is at NetClass level
(`specs/ENGINE_SPEC.md:241-243` `diffpair_width`,
`diffpair_gap`); power-net auto-detection is part of ERC
power-symbol handling.

**Implementation cost (Datum).** **Zero immediate; possible
small extension to add diff-pair pin auto-detection helper.**
The pin-naming-to-diffpair mapping is a nice quality-of-life
feature for the AI surface ("auto-pair these pins as a
differential signal based on `_P` / `_N` suffix").

**Strategic recommendation.** **Add a small MCP utility tool
post-M7**: `infer_diffpair_from_pinnames { component_uuid }`
returning candidate `(pin_p, pin_n)` pairs. Effort: ~1-2 days.

### Encryption & IP Protection

#### IBIS .ibs encryption

**Full title.** Various BIRDs (Buffer Issue Resolution Documents)
formalise IBIS encryption: **BIRD-176** (introduced encryption,
2016), **BIRD-196** (key management, 2018), **BIRD-219**
(IBIS 7.0 encryption clarifications, 2019).

**Issuing body.** IBIS Open Forum.

**Scope.** A vendor can mark sections of an IBIS file as
encrypted using one of several IBIS-Open-Forum-blessed
encryption schemes (typically AES-128 with a vendor-supplied
key delivered out-of-band, often through an EDA-tool licence
mechanism). The encrypted blocks pass through the IBIS parser
intact; the simulation tool decrypts them at simulation time
under licence enforcement.

**Adoption status (2026).** **Niche but growing.** Used
primarily by FPGA / SoC / serdes vendors who consider their
buffer characteristics as IP (Xilinx/AMD, Intel/Altera,
Achronix, Lattice for some parts).

**License / IP.** The encryption scheme is open (the spec
documents the AES algorithm + key handling); the keys are
licensed per-vendor.

**Datum impact.**
- **AI-surface gate**: the MCP `attach_ibis` tool must check the
  IBIS file for encrypted blocks (ibischk reports them) and
  flag the attachment with `encrypted: true`. The
  AI-readable-content tools (`extract_ibis_pin_table`,
  `validate_ibis`) must NOT return decrypted content; they
  return only the unencrypted metadata + an opaque handle for
  the encrypted blocks.
- **Pool storage**: encrypted IBIS files are stored verbatim in
  the pool. Datum never decrypts them.
- **Export / sharing**: when Datum exports a project that
  attaches encrypted models, the export bundle includes the
  encrypted file as-is; the receiving tool must hold the key.

**Risks and edge cases.**
- An AI agent might inadvertently include encrypted-block bytes
  in a prompt context if Datum's MCP tools do not gate properly.
  This is an IP-leak risk that Datum's MCP layer must
  proactively prevent.
- Some vendors wrap encrypted IBIS in **encrypted-archive**
  formats (`.ibs.zip` with password-protected ZIP); Datum's
  attachment validator should detect this and refuse to attach
  unencrypted-archive content.

#### SPICE encrypted libraries (PSpice / HSPICE)

**Full title.**
- **PSpice Encrypt-It** — Cadence's encryption format for `.LIB`
  files. AES-based.
- **HSPICE encryption (AvantHash + successors)** — Synopsys's
  encryption for `.inc` and `.sp` files. Multiple generations;
  current is AES-128 wrapped.
- **Spectre encryption** — Cadence's encryption for `.scs` files.
  Same AES family as PSpice.
- **LTspice encrypted models** — ADI uses simple obfuscation
  (LTspice-specific encryption documented informally on EEVblog
  / LTspice forums); not cryptographically strong.

**Issuing bodies.** Vendor-specific (Cadence, Synopsys, ADI).

**Scope.** Vendor model authors mark `.MODEL` / `.SUBCKT` blocks
as encrypted; the simulator decrypts at runtime under licence
enforcement.

**Adoption status (2026).** **Mainstream for IC vendor models.**
TI's IC models are typically PSpice-encrypted; ADI's are LTspice-
encrypted; ST and NXP mix.

**License / IP.** Vendor-specific.

**Datum impact.** **Same gate as IBIS encryption.** Datum's
MCP layer must:
- Detect encrypted blocks in attached SPICE files (encrypted
  blocks have a recognisable header signature).
- Flag the attachment `encrypted: true,
  encryption_scheme: "PSpice"|"HSPICE"|"LTspice"|"Spectre"`.
- Refuse to return decrypted content via AI-accessible tools.
- Pass encrypted files through to downstream simulators
  unmodified.

**Strategic recommendation.** **Implement encrypted-block
detection from day one** of SPICE attachment work. Adding it
later is far more painful than including it in the initial
implementation — the audit-trail and AI-safety implications
ripple into Domain 8 (process & quality) and the MCP API spec.

**Risks and edge cases.**
- Some vendor SPICE encryption schemes are weak (LTspice's
  obfuscation is reverse-engineered publicly). Datum should NOT
  attempt to decrypt regardless of scheme strength — the
  intent of the encryption is what matters, not its
  cryptographic robustness.
- Encrypted-content emission to AI prompts is a bigger risk than
  encrypted-content emission to simulator stdout, because LLM
  contexts may end up in vendor training data. The MCP layer
  must be the gate.

## Cross-Cutting Patterns

### Library-attached vs library-internal model storage

**The fork.**
- **Altium pattern (library-internal)**: `Component` is a
  monolithic library record that *contains* its
  `.SimulationModel` (SPICE), `.IBISModel` (IBIS), `.Footprint`
  reference, `.SchematicSymbol` reference, and parametric
  metadata. The library file (`.IntLib`) is a compiled binary
  archive. Strong versioning, atomic check-in. Coupled to
  Altium-specific file formats; complicates encryption
  (Altium handles vendor-encrypted SPICE specially).
- **KiCad pattern (library-attached)**: `.kicad_sym` symbol file
  references external `.cir` / `.lib` / `.s2p` / `.ibs` files
  by relative path. Symbol file stays text-diffable. Per-model
  versioning is independent. Vendor encryption Just Works
  (encrypted file passes through). Loose coupling, more files
  to manage.
- **Cadence pattern (hybrid)**: OrCAD CIS pulls SPICE / IBIS
  references from an external SQL database; the in-Capture
  schematic carries the reference, the database holds the
  binding to the actual file. Complex but enterprise-friendly.

**Recommendation for Datum.** **Adopt the KiCad attached-by-URI
pattern, with pool-stored model files** as a structured
extension. This means:

1. **Models are first-class pool entities**. The pool gains a
   `models/` subdirectory parallel to `parts/`, `packages/`,
   etc. Each model file gets a deterministic UUID derived from
   SHA-256 of file bytes.
2. **Parts attach models by UUID reference**, not by embedded
   path. `Part.behavioural_models: Vec<ModelAttachment>` where
   `ModelAttachment.model_uuid: Uuid` references a pool model
   entity.
3. **Multiple models per Part is natural**. A single Part may
   carry IBIS + SPICE + Touchstone + STEP simultaneously, each
   a separate `ModelAttachment` entry.
4. **Models survive Part inheritance**. `Part.base: Option<Uuid>`
   inheritance flows through model attachments by default;
   derived parts can override or augment.
5. **Pool layering applies**. Different pools can attach
   different models to the same Part UUID; pool priority
   resolves conflicts identically to other pool data.
6. **Encryption flag travels with the attachment**, not with
   the underlying file. This lets the same encrypted file be
   referenced from multiple Parts without re-encryption work.

**Why this beats the alternatives.**
- Datum's git-friendly pool architecture is fundamentally
  text-diffable, which Altium's compiled-library pattern is
  not. Forcing models into a Datum-internal binary BLOB
  re-introduces the diff-friendliness loss.
- Encrypted vendor models pass through cleanly without Datum
  needing any encryption integration.
- The AI surface can introspect model metadata (model name list,
  pin coverage, format) without ever touching encrypted
  content.

### Simulation backend pluggability

**The question.** Does Datum embed a SPICE / SI engine, or stay
simulator-agnostic and emit netlists for external tools?

**The empirical answer from the survey.** Almost no PCB tool
*embeds* a full simulator in the same binary as the layout
engine; the pattern across the industry is **separate processes
with file / stdin handoff**:

- KiCad bundles ngspice as a sibling binary; the schematic
  editor invokes `ngspice` as a subprocess and parses results
  from the output.
- Altium bundles "Mixed Sim" (their internal SPICE-3 derivative)
  as a separate engine module, called from the schematic
  editor; for IBIS-driven SI, Altium ships SimBeor as a
  separate-process tool.
- OrCAD-PSpice runs PSpice as a separate process from OrCAD
  Capture.
- Cadence Allegro uses Sigrity as an entirely separate product
  for SI/PI; the integration is project-file shared.
- Mentor PADS uses HyperLynx as a separate product.

**Recommendation for Datum.** **Stay simulator-agnostic. Emit
netlists. Do not embed.**

This means:
- `export_spice_netlist { board_uuid, dialect, output_path }`
  MCP tool emits a clean SPICE netlist (ngspice-compatible by
  default; PSpice / HSPICE / LTspice / Spectre dialects
  selectable).
- `export_ibis_stimulus { net_uuid, model_attachment_uuid,
  output_path }` MCP tool emits IBIS-driven PWL stimulus for
  pre-layout SI work.
- Touchstone files are emitted via export of S-parameter
  cascade stitching, when Datum eventually has S-parameter
  computation (M9+).
- The user / AI agent invokes ngspice, Xyce, LTspice, Sigrity,
  HyperLynx, ADS externally and feeds results back via
  separate analysis tools.

**Why this is the right call.**
- ngspice is GPL-3; embedding it would force Datum to be GPL-3.
- Avoiding embedded simulation keeps the engine binary small.
- Subprocess + file handoff matches how every professional tool
  works.
- AI agents can orchestrate the simulator-of-choice via MCP
  without Datum picking a winner.

**Reconciliation with IPC research on IPC-2581 embedded
models.** IPC-2581 supports embedded model references in the
fab data exchange (`<EmbeddedComponentModelRef>` for embedded
passives / actives in PCB substrates). This is **PCB-data-
exchange embedding**, not simulator embedding — entirely
separate concept. Datum's IPC-2581 export (Domain 1
recommendation) should support `<EmbeddedComponentModelRef>`
when emitting fab data; this is an export-only string field
that does not require a simulator.

### Encrypted-vendor-model handling

**Already covered in detail above** (IBIS encryption, SPICE
encryption sections). The cross-cutting summary:

1. **Detection happens at attach time.** When a model is attached
   to a Part via `attach_ibis` / `attach_spice`, the parser
   looks for encryption signatures and sets the
   `ModelAttachment.encrypted: bool` flag.
2. **AI-surface gate is in the MCP layer**, not the engine. The
   engine stores opaque bytes. The MCP layer is where AI
   exposure is decided.
3. **Read tools tier**:
   - **Always allowed**: model metadata (filename, format,
     model count, encrypted flag, encryption scheme).
   - **Gated**: model contents (`.ibs` text, `.cir` text). Tools
     that return contents must check `encrypted` and refuse.
4. **Pass-through preserved.** Encrypted models are bundled in
   exports verbatim; downstream tools handle decryption.
5. **Audit trail.** Every model attach / detach should be a
   transaction-logged operation. When Domain 8 (process &
   quality) deep-dive lands, the audit-trail surface should
   include model-attachment provenance.

**Datum differentiator.** No EDA tool surveyed has this gate at
an AI-API layer specifically. Altium and Cadence handle
encryption at the simulator-licence-check layer; the AI risk is
new. Datum's MCP-first architecture makes this gate possible to
enforce at one chokepoint, which competitors will struggle to
add post-hoc.

### Open-source simulator integration (ngspice / Xyce / Qucs)

**Survey of options.**

| Simulator | License | Platform | KiCad-compat? | Datum-leverage |
|-----------|---------|----------|---------------|----------------|
| ngspice | GPL-3 | Linux/Win/Mac | Yes (bundled) | Best leverage; widest model compatibility |
| Xyce | GPL-3 + BSD parts | Linux/Win | Some | HPC-niche; only worth when ngspice cannot scale |
| Qucs/QucsStudio | GPL | Linux/Win/Mac | Limited | RF-focused; smaller model library |
| OpenModelica | OSMC PL (~LGPL) | Linux/Win/Mac | No | Multi-domain; Verilog-A subset |
| ngspice-rust bindings | Various | depends | depends | Several stalled projects; no production-ready |

**Recommendation for Datum.** **Make ngspice the primary
recommended downstream simulator** by virtue of:
1. Largest installed base (KiCad's bundled choice).
2. Best vendor-model compatibility.
3. Active development.
4. Subprocess-only integration sidesteps GPL-3 contagion.

Datum should ship documentation pointing users to ngspice for
SPICE simulation; Xyce / Qucs / OpenModelica are noted but not
primary.

**MCP tool naming**: `export_spice_netlist` (generic, dialect
parameter) rather than `export_ngspice_netlist` (specific). This
keeps the path open for users with other simulators.

### Behavioural model authoring

**Survey of free IBIS authoring tools.**

- **HSPICE2IBIS** (Synopsys) — paid; converts HSPICE transistor
  netlists to IBIS.
- **Mentor Visual IBIS Editor** — paid; GUI authoring + validation.
- **iibis** (open community tool) — limited; CSV → IBIS
  conversion.
- **Manual hand-authoring** — common for simple IBIS files;
  reference is the IBIS Cookbook.

**Recommendation for Datum.** **Datum's library editor should
attach IBIS files; it should NOT author them in v1.** IBIS
authoring is a domain expertise (semicon vendor IO design)
that does not belong in a PCB tool. If Datum eventually adds
authoring (M9+), the right starting point is a guided wizard
that takes user-supplied V-I + V-T data and emits compliant
IBIS — but this is far down the roadmap.

### Manufacturer-supplied vs community-curated libraries

**The landscape.**

- **Manufacturer-supplied**: TI, ADI, NXP, ST, Microchip,
  Renesas, Infineon, Bosch all publish IBIS / SPICE models
  for their parts on their websites. Quality is good for
  flagship parts, lags for mid-volume parts. Distribution is
  through the manufacturer's part page (e.g., `ti.com/product/
  TPS65987` lists IBIS + STEP + datasheet links).
- **Community-curated**:
  - **Component Search Engine** (Samacsys; SnapEDA-like) —
    free, ad-supported.
  - **SnapEDA** — free; enterprise tier with vendor
    partnerships.
  - **UltraLibrarian** — Accelerated Designs; mostly free,
    paid for premium SPICE / IBIS.
  - **GrabCAD** — STEP focus, weaker on SPICE / IBIS.
  - **PartQuest / PartQuest Xpress** — Siemens; free tier;
    pulls from Mentor's library.
  - **KiCad libraries on GitHub** — community-maintained;
    symbol + footprint focus, limited SPICE / IBIS.
  - **The IBIS Open Forum's curated library**
    (`ibis.org/ibs/`) — small (~100 reference parts), used
    primarily for parser validation rather than design.

**Datum strategic position.** Datum's pool layering
(`docs/POOL_ARCHITECTURE.md:380-396`) is the right shape to
support this layered ecosystem natively:
- Project pool (highest priority, project-specific
  attachments)
- Organization pool (shared SnapEDA / UltraLibrarian imports)
- Manufacturer pool (TI IBIS index, ADI SPICE library,
  vendor-curated)
- Base pool (Datum-shipped reference parts)

The MCP `lookup_part_*` tools can search across all layered
pools, and the AI agent can present a layered "use the TI
official IBIS or the SnapEDA-extracted IBIS?" choice to the
user.

## EDA Tool Support Matrix

Coverage of behavioural models across major schematic-capture /
PCB-layout tools. Cells are: **I** = Ingest only,
**S** = Simulate, **B** = Both, **—** = None,
**Att** = Attachment-only (file stored, not interpreted).

| Tool | IBIS | IBIS-AMI | Touchstone | SPICE | Verilog-A | Octopart/Nexar | CIS |
|---|---|---|---|---|---|---|---|
| Altium Designer | B | I (via SimBeor) | I | B (Mixed Sim) | I | B (native) | B (DBLib/SVNDBLib) |
| OrCAD-PSpice / Cadence Allegro | B (Sigrity) | B (Sigrity) | B | B (PSpice) | B | I (Connect) | B (CIS) |
| Mentor PADS / Mentor HyperLynx | B (HL) | B (HL) | B | B (PSpice/HSPICE OEM) | I | I (partner) | B (Pulsonix-like) |
| Siemens Xpedition | B (HL) | B (HL) | B | B | I | I | B |
| KiCad 7+ | I+S (limited) | — | — (via ngspice include) | B (ngspice) | I (ngspice ADMS) | I (community plugin) | B (DB Libs v7+) |
| Eagle / Fusion Electronics | Att | — | Att | Att | — | I (deprecated) | — |
| Horizon EDA | — | — | — | — | — | — | — |
| LibrePCB | — | — | — | — | — | — | — |
| DipTrace | — | — | — | I | — | — | — |
| EasyEDA / EasyEDA Pro | — | — | — | I+S (Pro, limited) | — | I (LCSC) | — |
| **Datum (current)** | — | — | — | — | — | — | — |
| **Datum (recommended post-M7)** | B (attach + ibischk + ngspice stim) | Att (Out of Scope for sim) | B (attach + scikit-rf-like + S-param compose) | B (attach + ngspice subprocess) | Att (compile externally) | I (Octopart/Nexar MCP) | Out of Scope v1 |

## SI/PI Tool Support Matrix

Coverage of behavioural-model formats by SI/PI / EM-extraction
tools. These are external tools that Datum should hand off to,
not embed.

| Tool | IBIS | IBIS-AMI | Touchstone | SPICE | S-parameter extraction |
|---|---|---|---|---|---|
| Cadence Sigrity | B | B | B | B | Native |
| Mentor HyperLynx | B | B | B | B | Native |
| Ansys SIwave + Q3D | B | B (via ChipPI) | B | I | Native |
| Keysight ADS | B | B (flagship) | B | B | Native |
| CST Studio Suite (Dassault) | I | I | B | I | Native (3D EM) |
| SimBeor | I | — | B | I | Native |
| Polar Si9000 | — | — | B | — | Impedance focus |
| **Datum (current)** | — | — | — | — | — |
| **Datum (recommended)** | export-only (stimulus generation) | hand-off only | export-only (stitching, M9+) | export-only (netlist) | hand-off only |

## Pending Exclusions (re-affirmed)

The Phase 1 audit's advisory exclusion list for Domain 2 is
**re-affirmed** by this deep-dive:

- **JEDEC JEP30 (PIP — Part Information Profile)** — XML metadata
  format superseded in practice by manufacturer-specific
  datasheets and Octopart/Nexar API. No tool surveyed actively
  exercises JEP30 in 2026. **No hidden cross-cutting value
  found.** Re-affirm as out of scope; can be formally excluded
  in the post-Domain-8 ratification pass.
- **JEDEC JESD8 (logic-family electrical specifications,
  LVTTL / LVCMOS / etc.)** — superseded in practice by
  per-vendor IBIS files which carry the same V_OL / V_OH /
  I_OL / I_OH information empirically per buffer. **No hidden
  cross-cutting value found.** Re-affirm.
- **JEDEC MO (Mechanical Outline) drawings (MO-220, MO-153,
  etc.)** — superseded by manufacturer 3D STEP models, which
  carry the same dimensional information with higher fidelity
  and are machine-readable. **One marginal cross-cutting value:**
  MO drawings are still cited as the authoritative source for
  *body height* in some IPC-7351 footprint generation
  workflows (see IPC compliance research). The body-height
  field on `Package` (Domain 1 recommendation) covers this need
  without ingesting MO drawings directly.
- **IHS Markit Engineering Workbench (now S&P Global
  Engineering Workbench)** — paid component-intelligence
  service. Not a Datum-engine concern. Per-customer integration
  if ever requested. **No hidden cross-cutting value found.**
  Re-affirm.

**Newly recommended exclusions** (surfaced during this deep-dive):

- **Verilog (IEEE 1364) / SystemVerilog (IEEE 1800) / VHDL
  (IEEE 1076) HDL languages** — these are FPGA / ASIC tools,
  not PCB tools. Datum should formally exclude HDL language
  support and explicitly accept FPGA pin constraint files
  (XDC / QSF / LPF) as a Domain 4 (industry-vertical FPGA
  integration) future concern.
- **MAST (Saber)** — proprietary modelling language, no open
  parser, no PCB-tool footprint. Formally exclude.
- **JEDEC programmable-logic .jed files** — CPLD-only legacy;
  no PCB-tool consumption pattern. Formally exclude.
- **EDIF for HDL exchange (EDIF 4 0 0)** — per Domain 1, EDIF
  is dead in practice. Formally exclude.

## User Pain Points & Wishlist Items

Distilled from forum activity (EEVblog, KiCad forum, Reddit
r/PrintedCircuitBoard, Cadence Community, Altium Forum, IBIS
Open Forum mailing list):

1. **"Why does the vendor's IBIS not work in [tool X]?"**
   Vendor IBIS files routinely fail downstream-tool
   syntax-check because they were authored to ibischk's older
   release and a newer version has stricter validation. Users
   want a single canonical validator. Datum can ship `ibischk`
   integration with version-pinning to give a single source of
   truth.
2. **"How do I know which SPICE dialect this `.LIB` file is
   in?"** Manufacturer SPICE distribution is dialect-mixed.
   TI ships PSpice + TINA-TI; ADI ships LTspice; ST ships
   PSpice. Users frequently waste hours diagnosing dialect
   mismatch errors. Datum's `attach_spice` MCP tool can
   detect dialect at attach time (PSpice has `*$` headers;
   LTspice has `* version=` markers; etc.) and surface the
   detected dialect in the Part record.
3. **"My OrCAD library doesn't import IBIS into the part."**
   OrCAD users migrating to KiCad lose IBIS attachment when
   the library is converted. Datum's KiCad / Eagle import
   should preserve IBIS attachment paths even if the
   downstream attach mechanism evolves.
4. **"Why doesn't [tool] tell me my IBIS is encrypted?"**
   Several forum threads mention surprised users discovering
   their downstream simulation flow silently failed because
   a vendor IBIS contained encrypted blocks. Datum's
   `attach_ibis` should surface the encryption status
   prominently.
5. **"Octopart says the part is EOL but Mouser still has 5000
   in stock — which is right?"** Lifecycle data from supply-
   chain APIs lags reality by months. Datum should let users
   override the API-fed lifecycle field with manual notes.
6. **"I want to attach a `.s4p` to my connector and have the
   tool figure out the differential pair port mapping
   automatically."** Touchstone port mapping ambiguity is a
   real pain point. Datum's `attach_touchstone` MCP tool can
   include a `port_mapping_hint` parameter that lets the user
   declare the port-to-pin mapping at attach time.
7. **"Why does my SPICE simulation of a TI op-amp give 60dB
   less gain than the datasheet?"** Vendor SPICE models often
   omit power-supply pin connections by default; users connect
   the model in a non-canonical topology and get garbage
   results. Datum's AI surface can use the
   `extract_spice_subckt_pin_list` MCP tool to surface pin
   ordering and warn on common topology errors.
8. **"I want to swap a 3.3V LDO for a 1.8V LDO without
   redrawing the schematic."** Part-replacement workflows that
   carry IBIS / SPICE / thermal data forward across part
   substitution are rare. Datum's pool layering + Part
   inheritance is the right substrate; the AI surface can
   automate the substitution and re-run ERC / SI sanity checks.

## Datum EDA Implementation Strategy

Triage of which Domain 2 capabilities Datum should implement,
when, and at what depth.

### Hard Requirements (must support)

These are the table-stakes capabilities without which Datum
cannot compete with KiCad / Altium / OrCAD on Domain 2. All
should land post-M7 in the rough order listed.

#### 1. `ModelAttachment` canonical IR + pool model storage

- **Why must:** every other Domain 2 capability hangs off this
  type. Without it, Datum cannot store any behavioural model.
- **Canonical IR changes:** add `ModelAttachment` type, add
  `Part.behavioural_models: Vec<ModelAttachment>` field, add
  `ModelFormat` enum, add `ModelRole` enum, add `EncryptionScheme`
  enum + `ModelAttachment.encrypted: bool` flag.
- **Pool model changes:** add `pool/models/` directory; pool
  index gains `models` table; model files referenced by SHA-256-
  derived UUID.
- **Transaction model changes:** add `AttachModel` and
  `DetachModel` operations.
- **MCP API additions:** none (per-format MCP tools cover this).
- **Minimum viable:** types + storage + attach/detach ops only.
  Effort: ~1 week.
- **Full implementation:** + pool index + provenance fields +
  encryption flag handling. Effort: ~2 weeks.
- **Partner / library dependencies:** none.

#### 2. Touchstone attachment + validation

- **Why must:** broadest applicability per engineering hour;
  every SI workflow uses Touchstone; clean grammar; pure-Rust
  parser available.
- **Canonical IR changes:** `ModelAttachment` with
  `role: Touchstone`, `touchstone_ports: u32`,
  `frequency_range_hz: Option<(f64, f64)>`.
- **Pool changes:** model storage as above.
- **Transaction model changes:** none beyond `AttachModel`.
- **MCP API additions:** `attach_touchstone`,
  `validate_touchstone`, `extract_touchstone_summary`.
- **Minimum viable:** attach + parse + validate. ~3-5 days
  using `touchstone-rs`.
- **Full implementation:** + scalar summary extraction + AI-
  friendly metadata. ~1-2 weeks.
- **Partner / library dependencies:** **`touchstone-rs`**
  (MIT, pure-Rust).

#### 3. IBIS attachment + ibischk validation

- **Why must:** the migration-path-blocker for Altium / OrCAD
  users; the single most-cited "missing" Domain 2 capability
  in industry surveys.
- **Canonical IR changes:** `ModelAttachment` with
  `role: Ibis`, `ibis_version: Option<String>`,
  `model_names: Vec<String>`.
- **Pool changes:** model storage as above.
- **Transaction model changes:** none beyond `AttachModel`.
- **MCP API additions:** `attach_ibis`, `validate_ibis`,
  `list_ibis_models`, `extract_ibis_pin_table`.
- **Minimum viable:** attach + ibischk-via-FFI validation +
  pin-table extraction. ~2 weeks.
- **Full implementation:** + IBIS-derived ngspice stimulus
  generation. ~+3-4 weeks.
- **Partner / library dependencies:** **`ibischk`** (BSD,
  C99, FFI via `bindgen`).

#### 4. SPICE attachment + ngspice-based syntax check

- **Why must:** SPICE is the universal analog model format; every
  vendor IC datasheet's "SPICE Macromodel" link points at one of
  these dialects; users who came from PSpice / LTspice / HSPICE
  expect to attach what they have.
- **Canonical IR changes:** `ModelAttachment` with
  `role: Spice`, `dialect: SpiceDialect`, `model_names:
  Vec<String>`, `subckt_pin_orders: HashMap<String,
  Vec<String>>`.
- **Pool changes:** model storage as above.
- **Transaction model changes:** none beyond `AttachModel`.
- **MCP API additions:** `attach_spice`, `validate_spice`
  (subprocess to ngspice), `extract_spice_subckt_pin_list`,
  `export_spice_netlist`.
- **Minimum viable:** attach + ngspice subprocess validation +
  subckt pin list extraction. ~1-2 weeks.
- **Full implementation:** + dialect detection + cross-dialect
  warnings. ~+3-4 weeks.
- **Partner / library dependencies:** **ngspice as subprocess**
  (no linking; user must have ngspice installed; bundle in
  Linux distros).

#### 5. Thermal compact-model fields on Part

- **Why must:** universal datasheet adoption; near-zero cost;
  large AI-surface payoff (thermal queries are common).
- **Canonical IR changes:** add `Part.thermal:
  Option<ThermalSpec>` with the four `Option<f32>` resistance
  fields + `max_junction_c`.
- **Pool changes:** none beyond Part record.
- **Transaction model changes:** none.
- **MCP API additions:** none new (covered by Part-edit ops).
- **Minimum viable + full implementation:** ~1 day.

#### 6. JEP106 manufacturer ID normalisation

- **Why must:** ~1 day implementation; large value as
  manufacturer-name canonicalisation reference.
- **Canonical IR changes:** add `Part.manufacturer_jep106:
  Option<u16>` field.
- **Pool changes:** ship static `jep106.json` lookup table.
- **Transaction model changes:** none.
- **MCP API additions:** `normalize_manufacturer`.
- **Minimum viable + full implementation:** ~1 day.

#### 7. Encryption gate on AI-accessible MCP tools

- **Why must:** IP-management hazard. Once AI agents have access
  to `extract_*` tools on `ModelAttachment`, encrypted-content
  exfiltration risk is real. Cheaper to gate from day one than
  retrofit.
- **Canonical IR changes:** `ModelAttachment.encrypted: bool`
  + `encryption_scheme: Option<EncryptionScheme>` (already in
  item 1).
- **Pool changes:** none beyond storage.
- **Transaction model changes:** none.
- **MCP API additions:** every model-content-returning MCP
  tool gains an `encrypted_handling: ErrorIfEncrypted |
  OpaqueHandle` parameter; default is `ErrorIfEncrypted`.
- **Minimum viable + full implementation:** ~2-3 days at MCP
  layer.

### Should Support (post-M7)

#### 8. Octopart / Nexar API integration

- **Why should:** unifies distributor-catalog access; one API
  covers most users; AI agents can use it for BOM enrichment,
  alternates, lifecycle refresh.
- **Canonical IR changes:** add `Part.supply_chain_offers:
  Option<Vec<SupplyOffer>>` (cache; not authoritative); add
  `Part.last_supply_chain_check: Option<DateTime>`.
- **Pool changes:** pool index can cache offers per Part.
- **Transaction model changes:** supply-chain refresh is
  derived-data, not authored; no Op needed unless the user
  edits.
- **MCP API additions:** `lookup_part_octopart`,
  `refresh_supply_chain`, `find_alternate_parts`.
- **Minimum viable:** `lookup_part_octopart` only. ~3-5 days.
- **Full implementation:** + caching + alternates + lifecycle
  refresh + BOM enrichment. ~3 weeks.
- **Partner / library dependencies:** Nexar API key
  (user-supplied at runtime); `nexar-rs` or hand-rolled
  GraphQL client.

#### 9. Direct distributor APIs (Digi-Key + Mouser)

- **Why should:** the two largest distributor APIs by
  hobbyist + small-business volume; complements Octopart/Nexar
  for users with preferred distributor.
- **Canonical IR changes:** none beyond Octopart additions.
- **Pool changes:** none.
- **Transaction model changes:** none.
- **MCP API additions:** `lookup_part_digikey`,
  `lookup_part_mouser`.
- **Effort:** ~2 days per distributor.

#### 10. EIA-481 packaging fields on Part

- **Why should:** assembly-house onboarding workflow benefit;
  cheap data-model addition; populated free from
  Octopart/Nexar.
- **Canonical IR changes:** add `Part.packaging_options:
  Vec<PackagingOption>`.
- **Pool changes:** none.
- **Transaction model changes:** none.
- **MCP API additions:** `query_packaging_options`.
- **Effort:** ~1 day for fields; ~1 week for Octopart-fed
  population.

#### 11. IBIS-derived ngspice stimulus generation

- **Why should:** lets users do pre-layout SI sanity checks
  without a commercial SI tool; AI agents can drive this for
  automated SI heuristics.
- **Canonical IR changes:** none beyond IBIS attachment.
- **Pool changes:** none.
- **Transaction model changes:** none (read-only operation).
- **MCP API additions:** `export_ibis_stimulus`.
- **Effort:** ~3-4 weeks (IBIS V-T parsing + PWL generation
  + ngspice deck synthesis).

### On-Demand Only

#### 12. IBIS-AMI attachment

- **Why on-demand:** small subset of users (serdes-heavy
  designs); attachment is cheap once IBIS is done; execution
  is research-grade work.
- **Canonical IR changes:** add platform-keyed binary fields
  to `ModelAttachment` for AMI shared libraries.
- **Pool changes:** pool stores per-platform binaries.
- **Transaction model changes:** none beyond `AttachModel`.
- **MCP API additions:** `attach_ami`.
- **Effort when triggered:** ~1 week for attachment; AMI
  execution is **explicitly not in scope**.

#### 13. IBIS-ISS subcircuit attachment

- **Why on-demand:** marginal extension on IBIS attachment;
  free if IBIS is done.
- **Effort:** trivial; one additional `ModelRole` enum variant.

#### 14. Verilog-A / Verilog-AMS attachment

- **Why on-demand:** rare in PCB workflows; users who need it
  can pre-compile externally.
- **Effort:** ~1-2 days for attachment-only; no execution.

#### 15. Compact thermal model (CTM / ECXML / JESD15-4)
attachment

- **Why on-demand:** niche thermal-CFD workflow.
- **Effort:** ~1 day for attachment-only.

#### 16. CIS database bridge

- **Why on-demand:** enterprise-only; per-customer
  configuration.
- **Effort:** ~3-4 weeks per direction; customer-funded.

#### 17. SiliconExpert API integration

- **Why on-demand:** paid, defence/medical/automotive only.
- **Effort:** similar shape to Octopart MCP wrapper; ~3-5 days.

#### 18. Differential-pair pin auto-detection helper

- **Why on-demand:** quality-of-life MCP utility; small AI-
  surface payoff.
- **Effort:** ~1-2 days.

### Out of Scope (recommend formal exclusion)

Should be marked **explicitly** out of scope in
`docs/INTEROP_SCOPE.md` so users do not have to guess:

- **Verilog (IEEE 1364) / SystemVerilog (IEEE 1800) / VHDL
  (IEEE 1076)** — FPGA / ASIC HDLs, not PCB-tool concern.
- **MAST (Saber / Synopsys)** — proprietary; no open parser.
- **JEDEC programmable-logic .jed files** — CPLD legacy;
  no PCB-tool consumption.
- **EDIF for HDL exchange (EDIF 2 0 0 / 4 0 0)** — dead in
  practice (matches Domain 1 recommendation).
- **JEDEC JEP30 (PIP)** — superseded; Phase 1 advisory holds.
- **JEDEC JESD8** — superseded by per-vendor IBIS; Phase 1
  advisory holds.
- **JEDEC MO outline drawings** — superseded by STEP; Phase
  1 advisory holds.
- **IHS Markit / S&P Global Engineering Workbench** —
  paid distributor catalog; per-customer if ever.
- **In-engine SPICE simulation** — engine subprocess
  invocation only; never embed.
- **In-engine IBIS-AMI execution** — sandboxing arbitrary
  vendor binaries is research-grade work.
- **Datum-authored IBIS / SPICE / Touchstone files** — let
  external authoring tools produce these; Datum attaches.
- **Datum-authored PDF datasheet extraction** — let external
  vision-LLM tools or services do this; Datum can call them
  via MCP.

### Datum Differentiators

Where Datum's pool + transaction + AI surfaces can outperform
incumbents in the component-modelling space:

1. **Encrypted-vendor-model AI safety gate.** The MCP-layer
   gate on encrypted-content exposure is a Datum-unique
   capability. No other tool surveyed has this primitive
   designed in. As AI-driven component selection becomes
   industry-standard, encrypted-IP exfiltration risk is going
   to be regulated (NIST AI RMF, EU AI Act); Datum is
   positioned to be the first tool that demonstrably enforces
   the boundary.
2. **Model-attach as a transaction.** Datum's `AttachModel` /
   `DetachModel` operations are first-class transactions in
   the audit log. KiCad's symbol-file edits get logged in git
   per file change; Altium's library edits go through
   per-tool revision systems. Datum's combined
   transaction-and-pool log gives a single chronological
   view of model attachment changes.
3. **Pool-layered model libraries.** Layered pools
   (project + organization + manufacturer + base) let
   different pools attach different models to the same Part
   UUID; pool priority resolves conflicts. No other tool
   surveyed has this architecture for behavioural models.
   This means a team can have an organisation-wide IBIS pool
   without forcing every project to inherit it.
4. **AI-assisted model attachment.** "Find me an IBIS for
   this part" is a natural MCP tool — the AI agent can search
   manufacturer pages, Octopart, IBIS Open Forum, GitHub for
   community models, and propose attachments with provenance
   reasoning. No surveyed tool has this; closest is Altium's
   "Manufacturer Part Search" which surfaces existing library
   data but does not search the open web for models.
5. **Multi-attachment per Part.** A single Part can carry
   IBIS + SPICE + Touchstone + STEP + ThermalSpec, each with
   provenance. Industry tools generally allow one or two
   model formats per Part (Altium has SimulationModel +
   IBISModel as separate attached records but does not
   easily support multiple SPICE corners). Datum's `Vec<
   ModelAttachment>` pattern is more flexible.
6. **MCP API parity.** Every model attachment / extraction
   operation should be both a CLI command and an MCP tool.
   This makes AI-driven model curation workflows
   (find-attach-validate-export) trivial to script.

### Recommended Spec Edits

Concrete spec/doc edits the project owner should review before
applying. **Eleven items**, listed in suggested apply order.

#### Edit 1 — Add `ModelAttachment` type to `specs/ENGINE_SPEC.md` § 1.1a Shared Enums

**Add after** the existing `ModelRef` definition
(`specs/ENGINE_SPEC.md:74-77`):

```rust
pub enum ModelRole {
    Spice,             // SPICE netlist (.cir / .lib / .sub / .inc)
    Ibis,              // IBIS .ibs file
    IbisIss,           // IBIS-ISS subcircuit
    IbisAmi,           // IBIS-AMI bundle (.ami + binaries)
    Touchstone,        // S-parameter file (.s1p .. .sNp)
    VerilogA,          // Verilog-A source
    VerilogAms,        // Verilog-AMS source
    VhdlAms,           // VHDL-AMS source
    CompactThermal,    // CTM / ECXML / JESD15-4
}

pub enum SpiceDialect {
    Berkeley3,
    Ngspice,
    LTspice,
    PSpice,
    HSpice,
    Xyce,
    Spectre,
    Unknown,
}

pub enum EncryptionScheme {
    IbisBird176,       // IBIS BIRD-176 (AES-128, vendor key)
    PSpiceEncryptIt,
    HSpiceAvantHash,
    LTspiceObfuscation,
    SpectreEncrypt,
    Other(String),
}

pub struct ModelAttachment {
    pub uuid: Uuid,                           // pool-resolved UUID
    pub model_uuid: Uuid,                     // → pool model entity
    pub role: ModelRole,
    pub dialect: Option<SpiceDialect>,        // for SPICE only
    pub model_names: Vec<String>,             // [Model] names (IBIS), .MODEL/.SUBCKT names (SPICE)
    pub encrypted: bool,
    pub encryption_scheme: Option<EncryptionScheme>,
    pub provenance: ModelProvenance,
    pub format_metadata: ModelFormatMetadata, // role-specific extras
}

pub struct ModelProvenance {
    pub source: String,                       // URL or local path of origin
    pub vendor: Option<String>,               // canonical vendor (JEP106-normalised)
    pub fetched_at: Option<DateTime<Utc>>,
    pub sha256: String,                       // identity-stable hash
}

pub enum ModelFormatMetadata {
    Spice { ngspice_validates: Option<bool> },
    Ibis { ibis_version: String, has_ami: bool },
    IbisAmi { ami_version: String, platform_binaries: HashMap<String, String> },
    Touchstone { ports: u32, frequency_range_hz: (f64, f64) },
    None,
}
```

**Rationale:** the central type for all of Domain 2. Every other
edit hangs off this.

**Backward compatibility:** purely additive type; no impact on
existing data.

**Effort to apply:** ~1 hour for spec edit; ~1 week to land in
engine code.

#### Edit 2 — Extend `Part` in `specs/ENGINE_SPEC.md` § 1.2

**Current** (`specs/ENGINE_SPEC.md:147-162`):
```rust
pub struct Part {
    pub uuid: Uuid,
    pub entity: Uuid,
    pub package: Uuid,
    pub pad_map: HashMap<Uuid, PadMapEntry>,
    pub mpn: String,
    pub manufacturer: String,
    pub value: String,
    pub description: String,
    pub datasheet: String,
    pub parametric: HashMap<String, String>,
    pub orderable_mpns: Vec<String>,
    pub tags: HashSet<String>,
    pub lifecycle: Lifecycle,
    pub base: Option<Uuid>,
}
```

**Recommended replacement:**
```rust
pub struct Part {
    pub uuid: Uuid,
    pub entity: Uuid,
    pub package: Uuid,
    pub pad_map: HashMap<Uuid, PadMapEntry>,
    pub mpn: String,
    pub manufacturer: String,
    pub manufacturer_jep106: Option<u16>,            // ← new: JEP106 ID
    pub value: String,
    pub description: String,
    pub datasheet: String,
    pub parametric: HashMap<String, String>,
    pub orderable_mpns: Vec<String>,
    pub packaging_options: Vec<PackagingOption>,     // ← new: EIA-481
    pub tags: HashSet<String>,
    pub lifecycle: Lifecycle,
    pub base: Option<Uuid>,
    pub behavioural_models: Vec<ModelAttachment>,    // ← new: Domain 2
    pub thermal: Option<ThermalSpec>,                // ← new: JESD15-3
    pub supply_chain_offers: Option<Vec<SupplyOffer>>,  // ← new: cache
    pub last_supply_chain_check: Option<DateTime<Utc>>, // ← new: cache
}

pub struct ThermalSpec {
    pub theta_ja_c_per_w: Option<f32>,
    pub theta_jc_top_c_per_w: Option<f32>,
    pub theta_jc_bot_c_per_w: Option<f32>,
    pub theta_jb_c_per_w: Option<f32>,
    pub max_junction_c: Option<f32>,
    pub thermal_reference: Option<String>,           // "JESD51-2 still-air, 1S board"
}

pub enum PackagingKind {
    Reel { tape_width_mm: u16, reel_diameter_inch: u8, qty_per_reel: u32 },
    Tray { qty_per_tray: u32 },
    Tube { qty_per_tube: u32 },
    Bag { qty_per_bag: u32 },
    Cut { qty: u32 },                                // cut tape strip
}

pub struct PackagingOption {
    pub kind: PackagingKind,
    pub mpn_suffix: Option<String>,                  // e.g., TI's 'R' or 'T'
}

pub struct SupplyOffer {
    pub distributor: String,
    pub price_breaks: Vec<(u32, f64, String)>,       // (qty, price, currency)
    pub stock: Option<u32>,
    pub lead_time_weeks: Option<u32>,
    pub link: String,
}
```

**Rationale:** consolidates all Domain 2 Part-record extensions
into one structural change. `behavioural_models` and `thermal`
are authoritative (authored); `supply_chain_offers` and
`last_supply_chain_check` are cache (derived).

**Backward compatibility:** all new fields default to empty /
None; existing Part files continue to deserialize.

**Effort to apply:** ~2 hours for spec edit; downstream type
extension work is ~3-4 days.

#### Edit 3 — Add `pool/models/` directory to
`docs/POOL_ARCHITECTURE.md`

**Insert after** the existing pool directory tree
(`docs/POOL_ARCHITECTURE.md:128-152`):

```text
pool/
├── units/
├── entities/
├── symbols/
├── packages/
├── padstacks/
├── parts/
├── models/                  # ← new: behavioural model files
│   ├── ibis/
│   │   └── <sha256>.ibs
│   ├── spice/
│   │   ├── <sha256>.cir
│   │   └── <sha256>.lib
│   ├── touchstone/
│   │   └── <sha256>.s4p
│   ├── ami/
│   │   └── <sha256>/        # bundle: .ami + binaries
│   └── thermal/
│       └── <sha256>.xml     # ECXML compact thermal
└── pool.sqlite
```

Pool index gains a `models` table:

```sql
CREATE TABLE models (
    uuid TEXT PRIMARY KEY,
    sha256 TEXT NOT NULL UNIQUE,
    role TEXT NOT NULL,              -- 'ibis', 'spice', 'touchstone', ...
    file_path TEXT NOT NULL,
    file_size_bytes INTEGER,
    encrypted INTEGER DEFAULT 0,
    metadata_json TEXT
);

CREATE TABLE part_model_attachments (
    part_uuid TEXT NOT NULL REFERENCES parts(uuid),
    model_uuid TEXT NOT NULL REFERENCES models(uuid),
    role TEXT NOT NULL,
    PRIMARY KEY (part_uuid, model_uuid)
);
```

**Rationale:** establishes the pool storage convention for the
new model entities. Parallel to existing
units/symbols/packages/padstacks/parts hierarchy.

**Effort to apply:** ~1 hour for doc edit; ~1 week downstream
implementation (filesystem layout + index schema migration).

#### Edit 4 — Add `AttachModel` / `DetachModel` operations to
`specs/ENGINE_SPEC.md` § 5 Operations

**Add to the `Operation` enum / catalog** (under the existing
authored-edit operations):

```rust
pub struct AttachModel {
    pub part_uuid: Uuid,
    pub model_path: PathBuf,         // file to attach
    pub role: ModelRole,
}

pub struct DetachModel {
    pub part_uuid: Uuid,
    pub model_attachment_uuid: Uuid,
}
```

Both operations are **fully reversible**:
- `AttachModel.inverse()` returns a `DetachModel` for the
  same attachment UUID.
- `DetachModel.inverse()` returns an `AttachModel` re-creating
  the original attachment from the OpDiff.

**Rationale:** model attachment is an authored operation; it
must flow through the transaction model for audit-trail and
undo/redo correctness.

**Effort to apply:** ~1 hour spec edit; ~3-4 days
implementation.

#### Edit 5 — Add Domain 2 MCP tools to `specs/MCP_API_SPEC.md`

**Add new section** "Component Modelling Tools (M7+)" with the
following tool catalog. Each tool follows the existing MCP
parameter / return convention:

```text
### Component Modelling Tools (M7+)

#### `attach_ibis`
Attach an IBIS file to a Part.
Parameters:
- part_uuid: Uuid
- ibs_path: String (absolute or pool-relative)
Returns:
- attachment_uuid: Uuid
- ibis_version: String
- model_names: Array<String>
- encrypted: bool
- encryption_scheme: Optional<String>

#### `validate_ibis`
Run ibischk against an IBIS file (no attachment).
Parameters:
- ibs_path: String
Returns:
- valid: bool
- errors: Array<{ line: u32, severity: String, message: String }>
- warnings: Array<{ line: u32, message: String }>
- summary: { models: u32, components: u32, has_ami: bool }

#### `list_ibis_models`
Enumerate [Model] sections in an attached IBIS file.
Parameters:
- part_uuid: Uuid
- attachment_uuid: Uuid
Returns:
- models: Array<{ name: String, type: String,
                  signal_pin_count: u32 }>

#### `extract_ibis_pin_table`
Return per-pin → buffer-model mapping from an attached IBIS.
Parameters:
- part_uuid: Uuid
- attachment_uuid: Uuid
- model_name: String
Returns:
- pins: Array<{ pin_name: String, model: String,
                buffer_type: String }>

Encryption gate: if attachment is encrypted, returns only
unencrypted pin metadata; encrypted-content extraction returns
"error: encrypted-block requested without override".

#### `attach_touchstone`
Attach an .sNp file to a Part.
Parameters:
- part_uuid: Uuid
- snp_path: String
- port_mapping_hint: Optional<Array<{ port: u32, pin_name: String }>>
Returns:
- attachment_uuid: Uuid
- ports: u32
- frequency_range_hz: (f64, f64)
- format: "Touchstone1" | "Touchstone2"

#### `validate_touchstone`
Validate a Touchstone file (no attachment).
Parameters:
- snp_path: String
Returns:
- valid: bool
- errors / warnings: as above
- summary: { ports: u32, freq_range_hz: (f64, f64),
             format: String, mixed_mode: bool }

#### `extract_touchstone_summary`
Return scalar summary (insertion loss, return loss) from
attached Touchstone.
Parameters:
- part_uuid: Uuid
- attachment_uuid: Uuid
- summary_freq_hz: f64
Returns:
- insertion_loss_db: f32
- return_loss_db: f32
- per_port: Array<{ port: u32, s_self_db: f32 }>

#### `attach_spice`
Attach a SPICE file to a Part.
Parameters:
- part_uuid: Uuid
- file_path: String
- dialect: "Auto" | "Berkeley3" | "Ngspice" | "LTspice" |
  "PSpice" | "HSpice" | "Xyce" | "Spectre"
Returns:
- attachment_uuid: Uuid
- detected_dialect: String
- model_names: Array<String>
- subckt_names: Array<String>
- encrypted: bool

#### `validate_spice`
Run ngspice in syntax-only mode against a SPICE file.
Parameters:
- file_path: String
- dialect: String
Returns:
- valid: bool
- errors / warnings: as above

#### `extract_spice_subckt_pin_list`
Return port ordering of a .SUBCKT in attached SPICE.
Parameters:
- part_uuid: Uuid
- attachment_uuid: Uuid
- subckt_name: String
Returns:
- ports: Array<String>            // ordered as in .SUBCKT line
- has_default_param_block: bool

#### `export_spice_netlist`
Export the current schematic as a SPICE netlist.
Parameters:
- schematic_uuid: Uuid
- output_path: String
- dialect: "Ngspice" | "PSpice" | "LTspice" | ...
- include_attached_models: bool (default true)
Returns:
- export_status: "success" | "partial" | "failed"
- warnings: Array<String>

#### `export_ibis_stimulus`
Generate IBIS-derived ngspice PWL stimulus for a net.
Parameters:
- net_uuid: Uuid
- driving_pin_uuid: Uuid
- model_attachment_uuid: Uuid
- output_path: String
Returns:
- export_status: String

#### `lookup_part_octopart`
Query Octopart/Nexar for a part record.
Parameters:
- mpn: String
- manufacturer: Optional<String>
Returns:
- part_record: { canonical_mpn, manufacturer, lifecycle,
                 datasheet_url, parametrics, distributor_offers }

#### `lookup_part_digikey`, `lookup_part_mouser`
Direct distributor lookups, same shape as Octopart.

#### `refresh_supply_chain`
Refresh Part.supply_chain_offers cache.
Parameters:
- part_uuid: Uuid

#### `find_alternate_parts`
Get cross-reference list from Octopart/Nexar.
Parameters:
- part_uuid: Uuid

#### `query_packaging_options`
Get tape/reel/tray options for an MPN.
Parameters:
- mpn: String

#### `normalize_manufacturer`
Map free-text manufacturer name to JEP106 + canonical name.
Parameters:
- name: String
Returns:
- jep106_code: u16
- canonical_name: String
- aliases: Array<String>

#### `infer_diffpair_from_pinnames`
Suggest differential pair pin pairings from name suffixes.
Parameters:
- component_uuid: Uuid
Returns:
- pairs: Array<{ pin_p_uuid: Uuid, pin_n_uuid: Uuid,
                 base_name: String, confidence: f32 }>
```

**Rationale:** complete MCP API coverage for Domain 2. Each
tool follows the existing parameter / return convention.

**Effort to apply:** ~3 hours for spec edit; per-tool
implementation effort summed in the must/should-support
sections above.

#### Edit 6 — Add encryption-gate policy to `specs/MCP_API_SPEC.md`

**Add a new top-level section** "Encrypted Content Handling
Policy" (placement: at the end of the MCP API spec, before
"Implementation status table"):

```text
## Encrypted Content Handling Policy

The MCP API includes tools that read content from attached
behavioural models (`extract_ibis_pin_table`,
`extract_spice_subckt_pin_list`,
`extract_touchstone_summary`, etc.). Vendor IBIS / SPICE /
Touchstone / STEP files may carry encrypted blocks under
IBIS BIRD-176, PSpice Encrypt-It, HSPICE AvantHash, LTspice
obfuscation, Spectre encryption, or other vendor schemes.

The MCP layer enforces the following rules:

1. **Detection at attach time.** When a model is attached,
   the parser detects encryption and sets
   `ModelAttachment.encrypted: bool` + `encryption_scheme`.
2. **Metadata is always allowed.** Tools that return only
   model metadata (filename, format, model count, encryption
   flag, port count) work on encrypted models.
3. **Content extraction is gated.** Tools that return model
   contents (.ibs text, .cir text, S-parameter values) check
   `encrypted` first; if true, return
   `error: encrypted-block requested without override`.
   The caller may pass an explicit
   `encrypted_handling: "OpaqueHandle"` parameter to receive
   an opaque content handle (sha256 of the block) instead of
   plaintext.
4. **Pass-through preserved.** Export operations
   (`export_kicad`, `export_ipc2581`, etc.) bundle encrypted
   files verbatim. Datum never decrypts.
5. **Audit trail.** Every model attach / detach / extract
   operation is transaction-logged. The audit log records
   the encryption status of any extraction attempt.

This policy applies to all current and future MCP tools.
New tools that read model content must implement the gate
before merge.
```

**Rationale:** establishes the encryption-handling pattern
once, so per-tool implementations have a clear contract. This
is also a Datum-differentiator capability (no surveyed tool
has this AI-API-level gate).

**Effort to apply:** ~1 hour for spec edit. Implementation
work is per-tool gating logic.

#### Edit 7 — Update `docs/LIBRARY_ARCHITECTURE.md` to include
behavioural models in the library scope

**Insert after** the existing "Canonical Datum Library Model"
section (`docs/LIBRARY_ARCHITECTURE.md:135-176`):

```text
### Behavioural model attachment

A Part may carry zero or more **behavioural models** describing
its electrical, thermal, or simulation behaviour:

- IBIS (.ibs) — I/O buffer characteristics
- SPICE (.cir / .lib / .sub) — analog/mixed-signal models
- Touchstone (.sNp) — S-parameter measurements
- IBIS-AMI — algorithmic / DSP serdes models (attachment only)
- Verilog-A / Verilog-AMS / VHDL-AMS — compact device models
  (attachment only)
- Compact Thermal (ECXML) — JESD15-4 multi-node thermal models
  (attachment only)

Models live in `pool/models/<role>/<sha256>.<ext>` and are
referenced from Part records by UUID. Pool layering applies:
different pools may attach different models to the same Part
UUID, with priority resolving conflicts.

Encrypted vendor models are stored verbatim and never
decrypted by the engine. The MCP layer enforces an AI-safety
gate on encrypted-content exposure.

See:
- `research/component-modeling/COMPONENT_MODELING_RESEARCH.md`
  for the full Domain 2 deep-dive.
- `specs/ENGINE_SPEC.md` § 1.1a for the canonical
  ModelAttachment type.
- `specs/MCP_API_SPEC.md` § Encrypted Content Handling Policy
  for the AI-safety gate.
```

**Rationale:** elevates behavioural model attachment from a
silent omission to an explicit library-scope capability.
Addresses the Phase 1 audit observation that "library is
currently scoped narrowly to symbol/footprint geometry"
(`research/standards-audit/STANDARDS_AUDIT.md:616-621`).

**Effort to apply:** ~30 min documentation.

#### Edit 8 — Update `specs/IMPORT_SPEC.md` to preserve model attachments

**Replace** the Eagle library deferred row
(`specs/IMPORT_SPEC.md:210`):

```text
| Spice models | Deferred | |
```

**With:**

```text
| Spice models | Best-effort (M7+) | Imported as Part.behavioural_models attachments; Eagle stores SPICE in `<spice><pinmapping/><model/></spice>` blocks |
| Ibis models | Best-effort (M7+) | Imported as Part.behavioural_models attachments via Eagle's deviceset attachment fields |
```

**And add** to the KiCad library import matrix
(`specs/IMPORT_SPEC.md:155-163`):

```text
| Spice models | Best-effort (M7+) | Imported from .kicad_sym pinmap + sim model URI; attached as Part.behavioural_models |
| Ibis models | Best-effort (M7+) | Imported from .kicad_sym sim_ibis_model URI; attached as Part.behavioural_models |
| Touchstone files | Best-effort (M7+) | Imported via .kicad_sym sim_touchstone URI; attached as Part.behavioural_models |
```

**Rationale:** the deferred Eagle SPICE row is the only
import-side mention of behavioural models. Once
`ModelAttachment` lands, both KiCad and Eagle imports should
preserve attached models with the same provenance flow as
imported geometry.

**Effort to apply:** ~30 min spec edit; downstream import-code
changes ~1-2 weeks per format.

#### Edit 9 — Add Domain 2 capabilities to
`specs/NATIVE_FORMAT_SPEC.md`

**Insert after** the existing pool reference section
(`specs/NATIVE_FORMAT_SPEC.md` § 4 Project Layout) the
following addition to the pool subdirectory list:

```text
├── pool/                     # optional project-local pool
│   ├── units/
│   ├── entities/
│   ├── symbols/
│   ├── packages/
│   ├── padstacks/
│   ├── parts/
│   ├── models/               # ← new: behavioural model files
│   │   ├── ibis/
│   │   ├── spice/
│   │   ├── touchstone/
│   │   ├── ami/
│   │   └── thermal/
│   └── pool.sqlite
```

**And add** a § 6.x "Pool Model Files" subsection with the
schema:

```text
### 6.x Pool Model Files (`pool/models/`)

Behavioural model files are stored verbatim under
`pool/models/<role>/<sha256>.<ext>`. The file contents are
opaque to the canonical IR; the pool index records:

- model UUID (deterministic from sha256)
- role (Spice / Ibis / Touchstone / IbisAmi / ...)
- format metadata (extracted at attach time)
- encryption status

Model files are referenced from `Part.behavioural_models` by
UUID. The schema is purely additive to existing native format;
existing projects continue to deserialize without model files.
```

**Rationale:** makes the `pool/models/` directory part of the
native-format contract.

**Effort to apply:** ~1 hour documentation.

#### Edit 10 — Add Domain 2 export-tool roadmap to
`docs/INTEROP_SCOPE.md`

**Insert** a new "Behavioural model attachment & export"
section in the existing future-export list:

```text
### Behavioural model attachment & export

#### Hard requirements (post-M7):
- IBIS attach + ibischk validate
- SPICE attach + ngspice subprocess validate
- Touchstone attach + parse/validate
- Thermal compact-model fields on Part (JESD15-3 two-resistor)
- Octopart / Nexar API integration

#### Should support (post-M7):
- IBIS-derived ngspice PWL stimulus generation
- Direct Digi-Key / Mouser distributor APIs
- EIA-481 packaging fields populated from Octopart
- JEP106 manufacturer ID normalisation

#### On-demand only:
- IBIS-AMI attachment (no execution)
- IBIS-ISS subcircuit attachment
- Verilog-A / Verilog-AMS / VHDL-AMS attachment (no execution)
- Compact Thermal (ECXML) attachment (no simulation)
- CIS database bridge (per-customer)
- SiliconExpert API integration (paid; defence/medical only)

#### Explicitly out of scope:
- Embedded SPICE simulation (subprocess only)
- Embedded IBIS-AMI execution (vendor binary sandboxing)
- HDL languages (Verilog 1364 / SystemVerilog 1800 / VHDL 1076)
- MAST (proprietary; no open parser)
- JEDEC programmable-logic .jed (CPLD legacy)
- EDIF for HDL exchange (dead in practice)
- JEP30 PIP / JESD8 / JEDEC MO (superseded)
- Datum-authored IBIS / SPICE files (attachment-only in v1)
- PDF datasheet extraction in engine (let AI surface call out)
```

**Rationale:** parallel to the Domain 1 export-roadmap edit;
gives Domain 2 explicit, ordered scope under
`docs/INTEROP_SCOPE.md`.

**Effort to apply:** ~30 min documentation.

#### Edit 11 — Add "Datum's open-stack position on behavioural models" appendix to `docs/COMMERCIAL_INTEROP_STRATEGY.md`

**Add new section to `docs/COMMERCIAL_INTEROP_STRATEGY.md`:**

```text
## Behavioural Model Stack — Datum's Open-Stack Position

Of the behavioural model formats Datum should target, the open-
stack picture parallels the data-exchange story:

- IBIS (.ibs / .pkg / .ami) — IBIS Open Forum, free PDF; ibischk
  reference parser BSD; vendor IBIS files distributed under
  vendor terms-of-use.
- IBIS-ISS — IBIS Open Forum, free.
- Touchstone 1.x / 2.x — IBIS Open Forum, free; multiple BSD/MIT
  parsers (scikit-rf, touchstone-rs).
- SPICE 3 / ngspice / Xyce — SPICE 3 BSD; ngspice GPL-3
  (subprocess only); Xyce GPL-3 + BSD (subprocess only).
- Verilog-A / Verilog-AMS — Accellera, free; OpenVAF MIT.
- VHDL-AMS — IEEE, paid PDF.
- JEDEC JESD15-3 / JESD15-4 — JEDEC, free after registration.
- ECXML compact thermal — open community schema.
- Octopart / Nexar — proprietary API, free + paid tiers.
- Distributor APIs (Digi-Key / Mouser / Arrow / Avnet) — vendor
  REST, free with key.
- LTspice — proprietary, free-of-charge for personal/commercial.
- HSPICE / PSpice / Spectre — proprietary, paid.

The proprietary entries (LTspice / HSPICE / PSpice / Spectre,
Octopart paid tier, IHS Markit, SiliconExpert) are
**downstream consumers / providers** that Datum interoperates
with via attachment + subprocess + API, never via embedded
licence. This means:

- Datum can attach a TI-PSpice-encrypted `.LIB` file as a
  `ModelAttachment` and never decrypt it; the user's PSpice
  install handles decryption.
- Datum can ingest Octopart / Nexar query results without
  redistributing the catalog.
- Datum's MCP API gate prevents AI agents from exfiltrating
  encrypted-block contents.

This significantly reduces the strategic risk of vendor
lock-in: the formats Datum cares about (IBIS, Touchstone,
SPICE-syntax exchange) are all open or have open
implementations. The vendor proprietary stack
(LTspice / PSpice / HSPICE / Spectre / Sigrity / HyperLynx /
ADS) is what users plug Datum into, not what Datum embeds.
```

**Rationale:** captures the open-stack position for behavioural
models, parallel to the Domain 1 open-stack appendix. Useful
for marketing and for reassuring institutional users about
vendor-lock-in risk.

**Effort to apply:** ~30 min documentation.

## Sources

### Primary specifications

- [IBIS Open Forum — Specifications](https://ibis.org/ver7.2/) — IBIS 7.2 specification PDF (free)
- [IBIS Open Forum — IBIS-ISS](https://ibis.org/iss/) — IBIS-ISS specification (free)
- [IBIS Open Forum — IBIS-AMI](https://ibis.org/quality_wip/) — IBIS-AMI 2.0 reference + work-in-progress
- [IBIS Open Forum — Touchstone](https://ibis.org/touchstone_ver2.1/) — Touchstone 2.1 specification (free)
- [IEEE Std 370-2020](https://standards.ieee.org/ieee/370/10222/) — Electrical Characterization at Frequencies up to 50 GHz (cites Touchstone 2.0)
- [Accellera Verilog-AMS 2.4](https://www.accellera.org/downloads/standards/v-ams) — Verilog-AMS 2.4 spec (free)
- [IEEE Std 1076.1-2017](https://ieeexplore.ieee.org/document/8267835) — VHDL-AMS standard (paid)
- [JEDEC JESD15-3](https://www.jedec.org/standards-documents/docs/jesd15-3) — Two-Resistor Compact Thermal Model (free after registration)
- [JEDEC JESD15-4](https://www.jedec.org/standards-documents/docs/jesd15-4) — DELPHI Compact Thermal Model (free after registration)
- [JEDEC JEP106BU](https://www.jedec.org/standards-documents/docs/jep-106ah) — Manufacturer's ID Code (free after registration)
- [EIA-481-D-2016](https://www.ecianow.org/standards/eia-481-d) — Embossed Carrier Taping (paid)
- [IBIS Cookbook 5th Edition](https://ibis.org/cookbook/) — IBIS Cookbook (free)

### Reference implementations

- [ibischk](https://ibis.org/parsers/) — IBIS Open Forum reference parser (BSD)
- [pyibis-ami](https://github.com/capn-freako/PyIBIS-AMI) — IBIS-AMI Python library (MIT)
- [scikit-rf](https://scikit-rf.org/) — Python S-parameter / Touchstone library (BSD-3)
- [touchstone-rs](https://crates.io/crates/touchstone) — Pure-Rust Touchstone parser (MIT)
- [ngspice](http://ngspice.sourceforge.net/) — Open-source SPICE (GPL-3)
- [Xyce](https://xyce.sandia.gov/) — Sandia parallel SPICE (GPL-3 + BSD)
- [Qucs / QucsStudio](https://qucs.sourceforge.net/) — Open-source RF circuit simulator
- [OpenVAF](https://openvaf.semimod.de/) — Modern Verilog-A compiler (MIT)
- [ADMS](http://mot-adms.sourceforge.net/) — Verilog-A → SPICE-C translator (BSD)
- [GHDL](http://ghdl.free.fr/) — Open-source VHDL simulator (GPL)
- [LTspice download](https://www.analog.com/en/resources/design-tools-and-calculators/ltspice-simulator.html) — Analog Devices free LTspice
- [nexar-rs](https://crates.io/crates/nexar) — Community Rust Nexar client (MIT)
- [digikey-api Python](https://github.com/peeter123/digikey-api) — Digi-Key Python client (MIT)
- [Open Cascade Technology](https://dev.opencascade.org/) — STEP MCAD reference (LGPL 2.1; cross-ref Domain 1)

### Vendor model libraries

- [TI IBIS index](https://www.ti.com/design-resources/embedded-development/ibis-models.html) — Texas Instruments IBIS model library
- [ADI IBIS library](https://www.analog.com/en/resources/design-tools-and-calculators/ibis-models.html) — Analog Devices IBIS library
- [NXP IBIS models](https://www.nxp.com/design/software/development-software/ibis-models:IBIS-MODELS) — NXP IBIS index
- [Microchip IBIS](https://www.microchip.com/en-us/tools-resources/develop/ibis-models) — Microchip IBIS index
- [ST IBIS models](https://www.st.com/en/development-tools/ibis-models.html) — STMicroelectronics IBIS
- [Renesas IBIS](https://www.renesas.com/us/en/support/technical-resources/engineer-school/ibis-models) — Renesas IBIS
- [TI PSpice models](https://www.ti.com/design-resources/embedded-development/spice-models.html) — TI PSpice library
- [ADI LTspice models](https://www.analog.com/en/resources/design-tools-and-calculators/ltspice-simulator.html#sim_models) — ADI LTspice models

### EDA tool documentation

- [Altium Simulation Models](https://www.altium.com/documentation/altium-designer/simulation-models) — Altium SimulationModel + IBISModel docs
- [Cadence Sigrity](https://www.cadence.com/en_US/home/tools/system-analysis/electromagnetic-solvers.html) — Sigrity SI/PI flagship
- [Cadence PSpice](https://www.cadence.com/en_US/home/tools/system-analysis/circuit-simulator/pspice.html) — PSpice product page
- [Mentor HyperLynx](https://eda.sw.siemens.com/en-US/pcb/hyperlynx/) — HyperLynx SI/PI
- [Keysight ADS](https://www.keysight.com/us/en/products/software/pathwave-design-software/pathwave-advanced-design-system.html) — ADS for AMI authoring
- [KiCad ngspice integration](https://docs.kicad.org/master/en/eeschema/eeschema.html#simulation) — KiCad simulation docs
- [KiCad IBIS support announcement](https://www.kicad.org/blog/2022/02/Version-6.0.2-Released/) — KiCad 6.0+ IBIS
- [KiCad database libraries](https://docs.kicad.org/master/en/eeschema/eeschema.html#database_libraries) — KiCad DB libs (CIS)
- [Octopart Developer Portal](https://octopart.com/api/home) — Octopart / Nexar API
- [Nexar GraphQL Reference](https://docs.nexar.com/) — Nexar API reference
- [Digi-Key API](https://developer.digikey.com/) — Digi-Key API portal
- [Mouser API](https://www.mouser.com/api-hub/) — Mouser API hub
- [SnapEDA](https://www.snapeda.com/) — Community library + datasheet parser
- [UltraLibrarian](https://www.ultralibrarian.com/) — Accelerated Designs library
- [Component Search Engine](https://componentsearchengine.com/) — Samacsys community library

### Forum / industry discussion

- [IBIS Open Forum mailing list archives](https://ibis.org/email_archive/) — IBIS authoring/validation discussion
- [EEVblog forum: SPICE / IBIS workflows](https://www.eevblog.com/forum/) — community SPICE/IBIS troubleshooting
- [KiCad forum: ngspice / IBIS](https://forum.kicad.info/c/simulation/) — KiCad sim community
- [KiCad Gitlab: behavioural model issues](https://gitlab.com/kicad/code/kicad/-/issues?label_name%5B%5D=Simulation) — KiCad sim feature backlog
- [Cadence Community: IBIS-AMI](https://community.cadence.com/cadence_blogs_8/) — Cadence-side IBIS-AMI discussion
- [Altium Forum: IBIS attachment](https://forum.live.altium.com/) — Altium sim community
- [LTspice users group](https://groups.io/g/LTspice) — LTspice community
- [Reddit r/electronics: SPICE simulator choice](https://www.reddit.com/r/AskElectronics/) — community SPICE choice threads
- [Reddit r/PrintedCircuitBoard: IBIS quality complaints](https://www.reddit.com/r/PrintedCircuitBoard/) — vendor IBIS quality threads

### Cross-references (Datum-internal)

- `research/standards-audit/STANDARDS_AUDIT.md` § 2 — Phase 1 inventory of Domain 2
- `research/data-exchange-interop/DATA_EXCHANGE_INTEROP_RESEARCH.md` § STEP — STEP MCAD cross-ref
- `research/ipc-compliance/IPC_COMPLIANCE_RESEARCH.md` § IPC-7351 — footprint geometry baseline
- `research/ipc-compliance/IPC_COMPLIANCE_RESEARCH.md` § IPC-2581 — embedded model references in fab data
- `docs/POOL_ARCHITECTURE.md` — pool architecture (model storage extension)
- `docs/LIBRARY_ARCHITECTURE.md` — library architecture (model attachment scope)
- `docs/CANONICAL_IR.md` — canonical IR (transaction model for AttachModel)
- `docs/INTEROP_SCOPE.md` — interop scope (Domain 2 export roadmap)
- `docs/COMMERCIAL_INTEROP_STRATEGY.md` — commercial interop (open-stack appendix)
- `specs/ENGINE_SPEC.md` — canonical types (ModelAttachment / Part extensions)
- `specs/IMPORT_SPEC.md` — import semantics (model preservation)
- `specs/NATIVE_FORMAT_SPEC.md` — native format (pool/models/)
- `specs/MCP_API_SPEC.md` — MCP API (Domain 2 tools + encryption gate)
- `specs/ERC_SPEC.md` — ERC pin types (already complete; no Domain 2 edit)
- `specs/SCHEMATIC_CONNECTIVITY_SPEC.md` — bus syntax (already complete)
