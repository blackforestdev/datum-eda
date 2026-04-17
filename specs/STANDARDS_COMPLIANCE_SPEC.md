# Standards Compliance Specification

## 1. Purpose

This specification is the controlling home for Datum's standards and compliance
coverage.

It defines:

- the allowed disposition states for researched standards
- the standards registry Datum must keep explicit
- target-state contracts for footprint/library standards
- target-state contracts for schematic/capture standards
- project-level compliance metadata obligations
- audit-trail and traceability requirements for standards-facing work

It does not claim that every standard listed here is implemented today.
Implementation status remains controlled by `specs/PROGRESS.md`.

## 2. Ownership And Precedence

This file controls standards and compliance disposition across the product.

When standards-facing questions overlap other specs, responsibilities are:

- `specs/STANDARDS_COMPLIANCE_SPEC.md`: required disposition, minimum metadata,
  and cross-domain obligations
- `specs/ENGINE_SPEC.md`: canonical in-memory types and invariants
- `specs/NATIVE_FORMAT_SPEC.md`: on-disk persistence
- `specs/IMPORT_SPEC.md`: import and export fidelity boundaries
- `specs/ERC_SPEC.md` and `specs/CHECKING_ARCHITECTURE_SPEC.md`: checking-rule
  and diagnostic/reporting semantics
- `specs/PROGRAM_SPEC.md`: milestone gates and non-goals

## 3. Disposition Model

Every researched standard family materially relevant to Datum must have one
explicit disposition:

- `Implemented`: normative support exists in controlling specs and is expected
  to be backed by implementation evidence.
- `Planned`: Datum intends first-class support; target-state contract is
  defined, but implementation is incomplete.
- `Reference-only`: Datum uses the standard for vocabulary, labels, metadata,
  or explanatory alignment, but does not claim algorithmic validation or file
  compatibility against it.
- `Deferred with prerequisite`: support is intentionally postponed until a
  prerequisite capability exists, and that prerequisite is explicitly recorded.
- `Out of scope`: Datum intentionally does not target the standard for current
  product scope.

Silence is prohibited once a standard has been researched in-repo.

## 4. Registry Baseline

Datum must keep an explicit registry for the eight audited domains:

1. Data exchange and interop
2. Component modelling
3. Schematic and drawing conventions
4. Industry-vertical compliance
5. Materials and environmental
6. EMC and signal integrity
7. PLM and lifecycle integration
8. Process and quality

The domain registry for current research is:

### 4.1 Data Exchange And Interop

- KiCad native import/export: `Implemented` within the currently specified
  roadmap slice.
- Eagle native import: `Implemented` within the currently specified roadmap
  slice.
- Gerber RS-274X, Gerber X2, Excellon: `Implemented` or already owned in
  current roadmap language.
- IPC-2581 Rev C, Gerber X3, IPC-D-356A, ODB++ v8.1: `Planned` (contract
  surface defined; the post-Standards-Audit-Batch-1 export-tool stubs in
  `MCP_API_SPEC.md` and the `IMPORT_SPEC.md` § 5 IPC-2581 Import target
  define the contract; per-format implementation pending).
- STEP AP203 / AP242, IDF 3.0: `Planned` (contract surface defined;
  prerequisites delivered by Standards-Audit-Batch-1 — typed `Transform3D`,
  `ModelFormat` enum, expanded `ModelRef`, and `Package.body_height_nm` /
  `body_height_mounted_nm` in `ENGINE_SPEC.md` § 1.1a–1.2; stackup material
  fields in `ENGINE_SPEC.md` § 1.3; export-tool stubs in `MCP_API_SPEC.md`).
- IDX / EDMD (incremental ECAD↔MCAD): `Deferred with prerequisite`.
  Prerequisite: a versioned change-set export surface plus the schema-stable
  `ModelProvenance` shape introduced in `ENGINE_SPEC.md` § 1.1a; both must
  be exposed through `MCP_API_SPEC.md` before IDX/EDMD can commit.
- DXF (mechanical interop, board outline + mechanical layer write):
  `Planned` (contract surface defined; `import_dxf_outline` MCP stub
  reserves the import direction).
- DWG: `Out of scope` (license-hostile; DXF covers the mechanical-interop
  use case per `INTEROP_SCOPE.md`).
- Specctra DSN/SES, Hyperlynx HYP: `Out of scope` (legacy / vendor
  single-tool extraction; no Datum use case justifies the format-parser
  cost).
- Commercial native formats such as Altium `.PcbDoc` / `.SchDoc`,
  OrCAD/Allegro binaries, and PADS binaries: `Out of scope` for current
  delivery milestones except where the program explicitly treats them as
  research-only under `R1`. The practical migration path is **IPC-2581
  Rev C import** (see `IMPORT_SPEC.md` § 5), which avoids
  reverse-engineering vendor binary formats entirely.

### 4.2 Component Modelling

- Datum part parametrics, lifecycle, and pin-direction semantics:
  `Implemented` as internal contracts.
- IBIS 7.x, Touchstone 1.x/2.x, SPICE (Berkeley3 / ngspice / LTspice /
  PSpice / HSpice / Xyce / Spectre dialects) attachment surfaces:
  `Planned` (contract surface defined; `ModelAttachment`, `ModelRole`,
  `SpiceDialect`, `EncryptionScheme`, `ModelFormatMetadata`, and
  `ModelProvenance` types in `ENGINE_SPEC.md` § 1.1a; `Part.behavioural_models`
  field; `AttachModel` / `DetachModel` operations; pool-models directory
  contract in `POOL_ARCHITECTURE.md` and `NATIVE_FORMAT_SPEC.md` § 6.x;
  per-format MCP attach/validate/extract tool stubs in `MCP_API_SPEC.md`
  Component Modelling Tools section. Per-format parser implementation
  pending.).
- IBIS-AMI, IBIS-ISS attachment (no execution): `Planned` (covered by the
  same `ModelAttachment` contract; execution explicitly out of scope).
- Compact thermal models (JESD15-3 two-resistor, JESD15-4 DELPHI / ECXML):
  `Planned` (`Part.thermal: Option<ThermalSpec>` in `ENGINE_SPEC.md` § 1.2
  defines the two-resistor surface; full DELPHI multi-node attachment is
  on-demand only).
- Encrypted vendor models (IBIS BIRD-176, PSpice Encrypt-It, HSpice
  AvantHash, LTspice obfuscation, Spectre encryption): `Planned` (handling
  policy defined as `MCP_API_SPEC.md` § Encrypted Content Handling Policy:
  metadata always allowed, content extraction gated, pass-through
  preserved, every gate-check audit-logged; Datum never decrypts).
- Behavioural-model **execution** (running ngspice / Xyce / vendor SPICE
  simulators): `Deferred with prerequisite`. Prerequisite: subprocess
  invocation harness plus a solver/program milestone. Per Datum
  policy, simulators are integrated as subprocesses only (never linked)
  to keep Datum's distributable license unconstrained by GPL-class
  dependencies.
- HDL languages (Verilog 1364 / SystemVerilog 1800 / VHDL 1076), MAST
  (Saber proprietary), JEDEC programmable-logic `.jed`, EDIF for HDL
  exchange, JEDEC JEP30 PIP, JEDEC JESD8, JEDEC MO drawings, IHS Markit
  Engineering Workbench: `Out of scope`. (HDL languages and MAST do not
  belong to PCB tooling; JEDEC JEP30/JESD8/MO and IHS Markit are
  superseded in practice by manufacturer-specific datasheets and
  Octopart-class metadata.)
- VHDL-AMS, Verilog-A, Verilog-AMS: `Out of scope` for execution; the
  attachment surface is technically present via the generic
  `ModelAttachment` shape but no Datum-side parser is planned in v1.

### 4.3 Schematic And Drawing Conventions

- IEEE 315 / IEC 60617 symbol-style policy: `Planned`.
- IEEE 200 / ASME Y14.44 reference-designator policy: `Planned`.
- ISO 7200 title-block field policy: `Planned`.
- ANSI Y14.1 sheet sizes: `Reference-only` until native sheet-size enforcement
  is specified.
- IPC-T-50 vocabulary baseline: `Planned` for user-visible terminology.
- Hierarchy and net-label semantics already owned by schematic specs:
  `Implemented`.

### 4.4 Industry-Vertical Compliance

- IPC-A-600 / IPC-A-610 class metadata hooks: `Planned`.
- 21 CFR Part 11 overlay: `Deferred with prerequisite`.
  Prerequisite: exported audit-log surface with user identity, timestamps,
  rationale, and optional signature state.
- AEC-Q, ISO 26262, ISO 13485, FDA Part 820, IEC 61508: `Reference-only` at
  the core engine layer unless and until a milestone explicitly promotes them.
- ITAR and EAR marking posture: `Planned` as project metadata and export guard
  policy, not as certification.
- DO-254, AS9100, MIL-PRF-31032, MIL-PRF-55110, NASA workmanship standards:
  `Out of scope`.

### 4.5 Materials And Environmental

- RoHS, REACH, IPC-1752A, IEC 62474 posture and metadata fields: `Planned`.
- WEEE, conflict minerals, JS709C, ELV, China RoHS, SCIP: `Deferred with
  prerequisite`.
  Prerequisite: stable compliance-declaration schema on `Part` and project/BOM
  export surfaces.
- Regulation-specific reporting engines: `Out of scope` for current milestones.

### 4.6 EMC And Signal Integrity

- Length-match, differential-pair, and impedance rule foundations: `Planned`.
- Interface-specific rule families such as USB, PCIe, DDR, and Ethernet:
  `Deferred with prerequisite`.
  Prerequisite: the foundational rule types above plus stackup material
  properties.
- Stackup material properties needed for impedance work: `Planned`.
- Product-certification standards such as FCC/CISPR/EN emissions compliance:
  `Reference-only` at core-engine level.

### 4.7 PLM And Lifecycle Integration

- Multi-pool layering and part lifecycle status: `Implemented`.
- Supply-chain and lifecycle-service attachment points: `Planned`.
- AS9102 first-article traceability hooks: `Deferred with prerequisite`.
  Prerequisite: explicit audit/export surface and variant/sign-off flow.
- Vendor-specific PLM integrations such as Windchill, Teamcenter, Aras, Arena:
  `Out of scope` unless promoted by a later milestone.

### 4.8 Process And Quality

- Deterministic transaction substrate and diff-friendly persistence:
  `Implemented`.
- Audit-log query/export surface: `Planned`.
- Library approval workflow and design-review sign-off overlay:
  `Planned`.
- ISO 9001 / ISO 13485 / 21 CFR Part 11 conformance claims:
  `Deferred with prerequisite`.
  Prerequisite: exported audit-trail completeness plus user/signature metadata.
- CMMI, ISO/IEC 12207, organisation-process assessments: `Out of scope`.

## 5. Footprint And Library Contracts

### 5.1 Required IPC Basis Ownership

Datum must treat IPC-derived footprint intent as first-class authored data for
native library content.

The target-state package/padstack ownership must include:

- standards family and revision basis
- package family identity
- density level
- source dimensions and tolerances
- source toe/heel/side or equivalent derivation values where applicable
- courtyard policy
- solder-mask policy
- paste/stencil policy
- naming basis
- derivation version or provenance

### 5.2 Compliance Status And Deviation Tracking

Datum must distinguish:

- compliant
- compliant with documented deviation
- non-compliant
- unknown basis

Documented deviation must be first-class data, not free-form prose only. The
target-state deviation record must capture:

- observable affected
- expected versus actual value
- rationale
- approval status
- scope: library-wide, package-specific, or project-local

### 5.3 Import Audit Requirements

Imported footprints and imported board geometry must preserve source data
exactly. Datum may not silently "heal" geometry to an inferred IPC result.

When a supported package family can be recognized, Datum must be able to report
the delta between:

- imported geometry
- inferred or declared standards basis

through structured diagnostics.

### 5.4 Minimum Standards-Owned Observables

The standards-aware footprint system must explicitly own these observables:

- copper pad geometry
- drill and annular ring
- solder-mask aperture policy
- paste-aperture policy
- courtyard policy
- thermal-pad policy for BTC/QFN/DFN-class parts
- naming policy and density suffix semantics
- via-type vocabulary where relevant

### 5.5 Current Standard Families

- IPC-7351B / IPC-7352 / IPC-7251 land-pattern basis: `Planned`
- IPC-7525 stencil/paste rules: `Planned`
- IPC-7093 BTC/thermal-pad rules: `Planned`
- IPC-2221 clearance-policy basis: `Planned`
- IPC-4761 via taxonomy: `Planned`
- IPC-A-600 / IPC-A-610 class selection: `Reference-only` until checking and
  output contracts consume the field

## 6. Schematic And Capture Contracts

### 6.1 Symbol-Style Policy

Datum may support multiple symbol-style profiles, but native symbol authoring
may not remain standards-agnostic forever.

The controlling target-state model must include a symbol-style policy with at
least:

- IEEE 315 profile
- IEC 60617 profile
- imported/custom profile

If the authored symbol does not conform to a recognized profile, it must be
marked as custom rather than implied compliant.

### 6.2 Reference Designators

Datum must define a deterministic reference-designator policy that can map to
recognized standards practice.

The target-state designator policy must support:

- default prefix tables aligned to IEEE 200 / ASME Y14.44 expectations
- explicit custom-prefix override
- validation that distinguishes policy drift from user-intent override

### 6.3 Title Blocks And Sheet Metadata

Native schematic ownership must reserve structured fields for title-block and
sheet metadata that align with ISO 7200-class expectations.

Minimum target-state fields:

- title
- document number or project identifier
- revision
- company or organization
- approver/reviewer placeholders
- page numbering
- date or release marker

### 6.4 Vocabulary Baseline

User-visible standards-facing terminology should align to IPC-T-50 or another
explicitly chosen baseline. Mixed or ad hoc terminology across docs, UI, and
exports is not acceptable once standards-aware authoring is introduced.

## 7. Project-Level Compliance Metadata

Datum must provide a project-level home for standards and compliance posture.

The target-state metadata set must support:

- IPC class selection and standards revision basis
- intended operating environment profile
- export-control marking posture
- materials/compliance declaration posture
- compliance notes and references
- approval/audit-overlay settings when enabled

These fields are required even when full validation is deferred, because
professional workflows need an explicit place to state intent.

## 8. Audit Trail And Review Contracts

Standards-facing and regulated-workflow claims require auditability beyond the
current transaction substrate.

The target-state audit surface must support:

- queryable transaction history
- timestamp ownership
- acting user identity
- rationale/description
- approval or sign-off overlays
- optional electronic-signature state where enabled

Until those fields exist, Datum may not claim compliance with standards that
depend on auditor-grade records or signatures.

## 9. Cross-Spec Update Rule

Any change that promotes a standards disposition from `Planned`,
`Reference-only`, or `Deferred with prerequisite` to `Implemented` must update
all affected specs in the same change, including as applicable:

- `specs/ENGINE_SPEC.md`
- `specs/NATIVE_FORMAT_SPEC.md`
- `specs/IMPORT_SPEC.md`
- `specs/CHECKING_ARCHITECTURE_SPEC.md`
- `specs/ERC_SPEC.md`
- `specs/MCP_API_SPEC.md`
- `specs/PROGRAM_SPEC.md`
- `specs/PROGRESS.md`

## 10. Research Integration Rule

Research-to-spec flow is mandatory:

1. research artifact lands under `research/`
2. guidance document lands under `docs/`
3. controlling disposition lands in `specs/STANDARDS_COMPLIANCE_SPEC.md`
4. affected domain specs are updated if the disposition changes a contract

Research findings may not be treated as integrated until step 3 is complete.
