# Standards Compliance Specification

## 1. Purpose

This specification is the controlling home for Datum's standards and compliance
substrate.

It defines:

- the allowed support-disposition states for researched standards
- the standards registry Datum must keep explicit
- target-state contracts for footprint/library standards
- target-state contracts for schematic/capture standards
- project-level standards and compliance metadata obligations
- audit-trail and traceability requirements for standards-facing work

It does not claim that every standard listed here is implemented today, and it
does not claim that Datum certifies a design, part, organization, process, or
manufacturing output. Implementation evidence remains controlled by
`specs/PROGRESS.md`.

## 2. Ownership And Precedence

This file controls standards and compliance-substrate disposition across the
product.

When standards-facing questions overlap other specs, responsibilities are:

- `specs/STANDARDS_COMPLIANCE_SPEC.md`: required disposition, minimum metadata,
  and cross-domain obligations
- `specs/ENGINE_SPEC.md`: canonical in-memory types and invariants
- `specs/NATIVE_FORMAT_SPEC.md`: on-disk persistence
- `specs/IMPORT_SPEC.md`: import and export fidelity boundaries
- `specs/CHECKING_ARCHITECTURE_SPEC.md`: `CheckRun`, `CheckFinding`, profile,
  waiver, deviation, diagnostic, and reporting semantics
- `specs/ERC_SPEC.md`: ERC-specific rule semantics that feed the shared
  checking substrate
- `specs/PROGRAM_SPEC.md`: milestone gates and non-goals

## 3. Support Disposition Model

Every researched standard family materially relevant to Datum must have one
explicit disposition:

- `Normative support specified`: controlling specs define Datum's intended
  data, checking, report, or export contract for the standard.
- `Implementation evidence present`: code, tests, fixtures, or generated
  artifacts demonstrate that the specified contract is implemented for the
  stated product slice.
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

Disposition is not certification language. A Datum status may say that Datum
has specified or implemented support for a standard-facing substrate, but it
must not imply third-party certification, regulatory approval, auditor
acceptance, fabrication acceptance, or organizational conformance.

When an existing registry row uses `Implemented`, it means both normative
support is specified and implementation evidence is expected to exist in the
current product slice. If evidence is missing or incomplete, the row must be
split or clarified rather than using `Implemented` as an aspirational label.

### 3.1 Normative Support Versus Evidence

Normative support is the contract Datum promises to model or check. Evidence is
proof that the current implementation satisfies that contract.

Minimum evidence references may include:

- engine or import/export tests
- fixture projects
- generated reports
- MCP/CLI round-trip tests
- artifact metadata emitted by a tool
- issue/PR references in `specs/PROGRESS.md`

Research notes and registry text alone are not implementation evidence.

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

- IEEE 315 / IEC 60617 / JIS C 0617 symbol-style policy: `Planned`
  (contract surface identified by Domain 3 research; native symbols must carry
  explicit style-profile assertions rather than silently inheriting library
  aesthetics).
- IEEE 200 / ASME Y14.44 / IEC 81346 reference-designator policy: `Planned`
  (warn-only validation against selectable designator profiles; authored
  prefixes remain user-controlled).
- ISO 7200 title-block field policy: `Planned` (structured title-block fields
  plus sheet-template architecture; ISO 7200 controls field meaning, not visual
  frame layout).
- ANSI Y14.1 sheet sizes: `Reference-only` until native sheet-size enforcement
  is specified.
- IPC-T-50 and IEC 60050 vocabulary baseline: `Planned` for user-visible
  terminology.
- Hierarchy and net-label semantics already owned by schematic specs:
  `Implemented`.
- EDIF as a schematic-style interchange target, DIN 40700 / DIN 6771 /
  DIN 40719, IEEE 100, ISO 3098, and ANSI/ASME Y14.5: `Out of scope` for the
  current schematic standards surface.

### 4.4 Industry-Vertical Compliance

- IPC-A-600 / IPC-A-610 class metadata hooks: `Planned`.
- 21 CFR Part 11 overlay: `Deferred with prerequisite`.
  Prerequisite: exported audit-log surface with user identity, timestamps,
  rationale, and optional signature state.
- AEC-Q component qualification metadata: `Planned` as Part metadata and
  library/BOM query surface.
- ISO 26262, IEC 61508, IEC 60601, ISO 13485, FDA Part 820, EU MDR, ATEX,
  CMMC, ISO 27001, and NIST 800-171: `Reference-only` or metadata-only at the
  core engine layer unless a later milestone explicitly promotes stronger
  workflow support.
- ITAR, EAR, and EU dual-use marking posture: `Planned` as project metadata
  plus a mandatory data-egress policy gate for MCP tools with external-network
  side effects, not as certification.
- AUTOSAR, DO-178C, ARP4754A, MIL-STD-1772, DO-254, DO-160, AS9100,
  IATF 16949, CMMI, MIL-PRF-31032, MIL-PRF-55110, and NASA workmanship
  standards: `Out of scope` as Datum product claims.

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

## 5. Standards Registry, Checks, And Reports

Datum's standards registry is authored product metadata. It is not a loose
documentation table and must be available to checking, import/export, report,
and artifact-generation flows.

Minimum registry record:

```rust
pub struct StandardsRegistryRecord {
    pub family: String,
    pub revision: Option<String>,
    pub domain: StandardsDomain,
    pub disposition: StandardsDisposition,
    pub normative_support_refs: Vec<SpecRef>,
    pub implementation_evidence_refs: Vec<EvidenceRef>,
    pub check_rule_refs: Vec<String>,
    pub report_section_refs: Vec<String>,
}
```

- `normative_support_refs` point to controlling specs
- `implementation_evidence_refs` point to tests, fixtures, generated artifacts,
  progress entries, or implementation references
- absence of evidence must not be hidden behind certification-like wording
- records must be deterministic and diff-friendly

### 5.1 Resolver And Revision Binding

All standards-aware checks and reports must resolve project state through
`ProjectResolver`.

Minimum inputs:

- `project_ref`
- `model_revision`
- selected standards registry revision
- selected `CheckProfile`
- project standards/compliance metadata
- Import Map context, including `import_key` for imported objects when present
- artifact metadata for generated or imported manufacturing/export files

Reports generated against moving project state must first resolve a concrete
`model_revision`. A standards report without an explicit model revision is a
draft view, not an auditable result.

### 5.2 CheckRun And CheckFinding Integration

Standards checks must emit the shared checking contract from
`specs/CHECKING_ARCHITECTURE_SPEC.md`.

- each standards check run is a `CheckRun`
- each diagnostic is a `CheckFinding`
- `CheckFinding.domain` is normally `Standards`, `Manufacturing`, or
  `Relationships`
- fingerprints include rule revision, normalized targets, observed values, and
  Import Map `import_key` where imported identity participates; `CheckRun` and
  waiver/deviation records carry the explicit revision scope
- standards-aware runs must include `CheckRun.profile_basis` and coverage
  entries that identify the selected standards/process basis and distinguish
  evaluated rule families from filtered, not-applicable, and not-yet-implemented
  families
- waiver and deviation application must be visible in the finding status
- repair must be proposal-first and must not mutate source geometry or metadata
  during checking

Legacy DRC views may include standards-owned findings, but the source finding
identity remains `CheckFinding`.

### 5.3 Artifact Metadata

Standards-facing imports, exports, reports, and manufacturing outputs must carry
artifact metadata sufficient to explain what was checked.

Minimum metadata:

- artifact type and format
- producing tool and Datum version when known
- source project reference
- `model_revision`
- standards registry revision
- selected `CheckProfile`
- import/export timestamp
- source import identity, including Import Map `import_key` when applicable
- waiver and deviation set revision or digest
- report/check run UUID when generated from a `CheckRun`

Artifact metadata may support compliance workflows, but it must not claim that
the artifact is certified.

### 5.4 Waivers And Deviations

Standards-facing waivers and deviations use the shared checking model.

- a waiver suppresses failure state for an identified finding target
- a deviation records an accepted delta from a selected standard, process, or
  rule basis
- both must be scoped to a revision range or explicit `model_revision`
- both must preserve the original finding and evidence
- stale waivers/deviations must be reportable when fingerprints, rule
  revisions, imported identities, or model revisions no longer match

Deviation records must be structured enough to support report generation and
audit review; free-form prose alone is insufficient.

### 5.5 Report Language

Standards reports must use substrate language:

- allowed: "checked against selected IPC-7351B profile"
- allowed: "no active findings for this CheckProfile at model revision X"
- allowed: "accepted deviation recorded for pad paste aperture"
- prohibited: "IPC certified"
- prohibited: "regulatory compliant"
- prohibited: "fabrication approved"
- prohibited: "organization certified"

If a report needs a project-facing status, it must be phrased as a Datum check
state, not as third-party certification.

## 6. Footprint And Library Contracts

### 6.1 Required IPC Basis Ownership

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

### 6.2 Standards Check Status And Deviation Tracking

Datum must distinguish standards-check state without using certification or
third-party conformance language:

- no active findings for the selected Datum rule profile at the checked
  `model_revision`
- no failing findings after accepted deviations are applied
- active finding exists
- unknown or undeclared basis

Documented deviation must be first-class data and must map to
`CheckDeviation`. The target-state deviation record must capture:

- observable affected
- expected versus actual value
- rationale
- approval status
- scope: library-wide, package-specific, or project-local
- standards family/revision and rule id
- `model_revision` or revision scope
- related `CheckFinding` fingerprint

### 6.3 Import Audit Requirements

Imported footprints and imported board geometry must preserve source data
exactly. Datum may not silently "heal" geometry to an inferred IPC result.

When an Import Map is used, the imported identity must be retained as
`import_key` and included in standards-aware finding fingerprints and report
evidence.

When a supported package family can be recognized, Datum must be able to report
the delta between:

- imported geometry
- inferred or declared standards basis

through structured `CheckFinding` diagnostics.

When the delta is mechanically actionable, Datum should also be able to create
an explicit correction proposal rather than stopping at display-only reporting.
For example, if an imported pad inherits solder-mask or paste aperture geometry
from copper where the declared or selected process rule expects a positive
mask expansion or stencil reduction, Datum should:

- preserve the imported geometry as source truth
- report the mask/paste observable delta
- propose a bounded correction against the selected standards/process basis
- require explicit user acceptance before mutating the design
- record provenance that the accepted edit corrected imported host-tool default
  behavior

This is not silent healing. It is standards-aware proposal/apply behavior.

The detection mechanism belongs in Datum's checking system. A board must not
pass the selected DRC, standards, or manufacturing `CheckProfile` cleanly when
pad/process aperture geometry violates the declared or selected
standards/process basis. At minimum, the standards-aware checking surface must
be able to represent findings for:

- missing solder-mask expansion
- solder-mask expansion below rule
- missing paste/stencil reduction
- paste/stencil reduction below rule
- process apertures inherited directly from copper where a rule expects an
  explicit offset
- inconsistent mask/paste policy among peer pads or peer footprints where a
  common basis is declared

### 6.4 Minimum Standards-Owned Observables

The standards-aware footprint system must explicitly own these observables:

- copper pad geometry
- drill and annular ring
- solder-mask aperture policy
- paste-aperture policy
- courtyard policy
- thermal-pad policy for BTC/QFN/DFN-class parts
- naming policy and density suffix semantics
- via-type vocabulary where relevant

### 6.5 Current Standard Families

- IPC-7351B / IPC-7352 / IPC-7251 land-pattern basis: `Planned`
- IPC-7525 stencil/paste rules: `Planned`
- IPC-7093 BTC/thermal-pad rules: `Planned`
- IPC-2221 clearance-policy basis: `Planned`
- IPC-4761 via taxonomy: `Planned`
- IPC-A-600 / IPC-A-610 class selection: `Reference-only` until checking and
  output contracts consume the field

## 7. Schematic And Capture Contracts

### 7.1 Symbol-Style Policy

Datum may support multiple symbol-style profiles, but native symbol authoring
may not remain standards-agnostic forever.

The controlling target-state model must include a symbol-style policy with at
least:

- IEEE 315 profile
- IEC 60617 profile
- JIS C 0617 profile
- imported/custom profile
- mixed profile

If the authored symbol does not match a recognized profile, it must be marked
as custom rather than implied standard-conforming.

The style assertion belongs on the symbol/library record. Project-level
metadata may express a preferred or mandated profile, but it may not erase the
per-symbol profile, because imported and international projects can legitimately
contain mixed styles.

### 7.2 Reference Designators

Datum must define a deterministic reference-designator policy that can map to
recognized standards practice.

The target-state designator policy must support:

- default prefix tables aligned to IEEE 200 / ASME Y14.44 expectations
- international profile support aligned to IEC 81346 practice
- explicit custom-prefix override
- validation that distinguishes policy drift from user-intent override

Validation is warn-only by default. Datum may flag a designator that drifts from
the selected profile, but it must not refuse authored prefixes solely because
they are profile-specific or team-specific.

### 7.3 Title Blocks And Sheet Metadata

Native schematic ownership must reserve structured fields for title-block and
sheet metadata that align with ISO 7200-class expectations.

Minimum target-state fields:

- title
- document number or project identifier
- document type
- revision
- company or organization
- date of issue or release marker
- document status
- approver and reviewer placeholders
- sheet index and sheet count
- technical reference
- project number
- customer
- classification or export-control marking hook

Datum must model title-block data separately from title-block visual layout.
The target architecture is a sheet-template object that places fields onto a
declared sheet size. ISO 7200 drives field semantics; Datum's template system
drives visual placement.

### 7.4 Vocabulary Baseline

User-visible standards-facing terminology should align to IPC-T-50 for PCB
manufacturing/library vocabulary and IEC 60050 for electrotechnical vocabulary.
Mixed or ad hoc terminology across docs, UI, MCP tools, CLI surfaces, and
exports is not acceptable once standards-aware authoring is introduced.

## 8. Project-Level Standards And Compliance Metadata

Datum must provide a project-level home for standards and compliance posture.

The target-state metadata set must support:

- IPC class selection and standards revision basis
- intended operating environment profile
- export-control marking posture
- materials/compliance declaration posture
- compliance notes and references
- approval/audit-overlay settings when enabled
- industry vertical declaration
- mandated symbol-style and designator-profile declarations
- data-egress policy for AI/MCP/network side effects

These fields are required even when full validation is deferred, because
professional workflows need an explicit place to state intent.

### 8.1 Substrate Versus Certification

Datum is a compliance substrate, not a certifying authority.

Datum may store, validate, export, and explain compliance-relevant metadata.
Datum must not claim that a project, part, organization, or manufacturing
process is certified merely because Datum carries fields related to that
standard.

Required project-compliance posture includes:

- `industry_vertical`
- `ipc_class`
- `intended_environment`
- export-control posture for ITAR / EAR / EU dual-use work
- intended safety-integrity declaration where applicable
- mandated symbol profile and designator profile where applicable
- `data_egress_policy`
- `audit_overlay`
- compliance notes and marking fields

Required part-qualification posture includes:

- AEC-Q grade and evidence fields where declared
- temperature grade
- radiation-tolerance declaration where declared
- MIL-spec qualification pass-through where declared
- per-part export-control attributes such as ECCN / USML / EU dual-use entry

Any MCP tool with external-network or external-AI side effects must consult the
project data-egress policy before execution once the policy surface exists.
Policy decisions must be auditable once the Domain 8 audit surface exists.

## 9. Audit Trail And Review Contracts

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

## 10. Cross-Spec Update Rule

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

## 11. Research Integration Rule

Research-to-spec flow is mandatory:

1. research artifact lands under `research/`
2. guidance document lands under `docs/`
3. controlling disposition lands in `specs/STANDARDS_COMPLIANCE_SPEC.md`
4. affected domain specs are updated if the disposition changes a contract

Research findings may not be treated as integrated until step 3 is complete.
