# Commercial Interop Strategy

> **Status**: Non-normative roadmap and strategy document.
> This document does not define current implementation contracts.
> Current controlling import contracts remain in `specs/IMPORT_SPEC.md`
> and `docs/INTEROP_SCOPE.md`.
> Scope terminology follows `specs/PROGRAM_SPEC.md` §Scope Integrity Terms.
> `docs/R1_G0_FOUNDATION.md` is the controlling foundation for lineage,
> migration-pain taxonomy, and fidelity-boundary language used here.

## Purpose
Define how Altium, PADS, and OrCAD/Allegro enter the roadmap without
destabilizing the `M0-M2` foundation. These formats matter strategically,
but they should not be allowed to distort the early engine architecture.

The main objective is not "open any proprietary file on day one." It is:

- build a migration path for commercial-tool users stranded on Windows
- preserve the engine-first, Linux-first value proposition
- avoid baking vendor-specific assumptions into the canonical IR
- stage reverse-engineering and fidelity work after the foundation is proven

Consumption rule:
- this document inherits the `R1-G0` categories `exact`, `approximated`,
  `preserved-as-metadata`, and `unsupported`
- this document must not make broader migration claims than the lineage map,
  pain taxonomy, and fidelity policy support

---

## 1. Why These Formats Matter

The long-term replacement problem is not KiCad. It is the shrinking set
of professional EDA tools that run anywhere except Windows.

The strategic migration targets are:

- **Altium Designer**
- **OrCAD / Allegro**
- **PADS**

These are the tools many serious Linux users are trapped behind today.
If this project is going to become a broadly adopted AI-native EDA platform,
it needs a credible path for bringing those users and their libraries forward.

---

## 2. Why They Are Not Priority-1 Import Targets

Commercial interop is delayed not because it is unimportant, but because
it has the wrong dependency order.

These formats generally imply:

- proprietary or partially documented storage
- binary containers or database-backed formats
- more complex library/component semantics
- vendor-specific rule, variant, and output concepts
- higher legal/reverse-engineering care
- stronger expectations of migration fidelity once support is claimed

If implemented too early, they would force premature decisions about:

- lossiness policy
- unsupported-object preservation
- library normalization rules
- rule-system mapping
- native persistence requirements
- external tool/runtime dependencies

The correct sequence is:

1. Stabilize canonical IR
2. Prove import fidelity on open/documented formats
3. Prove checking/query/value on imported projects
4. Only then add commercial migration paths

---

## 3. Strategic Principles

### 3.1 Interop is a migration program, not a parser collection

Commercial import should be treated as a structured migration effort:

- ingest
- normalize
- diagnose lossiness
- preserve intent where possible
- generate an explicit migration report

The goal is not merely to "parse a file." It is to make the imported
design usable and auditable inside this engine.

### 3.2 Library extraction comes before full design parity

The first practical value for commercial interop is often at the library
and component-model level:

- symbol extraction
- footprint/package extraction
- component parameter extraction
- part-to-footprint binding recovery

That is much cheaper than full board/schematic import and directly helps
the pool become useful to migrating users.

### 3.3 Use vendor-neutral intermediates when they reduce risk

Where a vendor format is extremely painful or legally sensitive, the
project should allow a staged flow:

- vendor export
- intermediate representation
- engine import

Examples:

- CSV/XML exports from database-backed tools
- IPC-2581 or other interchange outputs when available
- scripted extraction via external converters

This is acceptable if the workflow is explicit about what fidelity is
lost and what metadata is preserved.

### 3.4 Do not let vendor concepts leak into the canonical IR

The canonical IR should model enduring design concepts, not branded tool
features. If an Altium or OrCAD concept appears in import, the importer
must decide one of:

- cleanly map it to existing canonical concepts
- preserve it as structured import metadata
- declare it unsupported with explicit loss reporting

It should not force ad hoc canonical fields just because one vendor has
one special representation.

---

## 4. Tool-Specific Strategy

### 4.1 Altium

**Why it matters**
- likely the most strategically important migration target
- strong professional user base
- common benchmark for workflow expectations

**Challenges**
- `.PcbDoc` / `.SchDoc` complexity
- format accessibility and library quality variance
- many embedded workflow concepts beyond simple geometry/connectivity
- strong risk of hidden semantics unless provenance and unsupported constructs
  are reported explicitly

**Recommended first step**
- library/component extraction first
- then read-only design import via an external extraction path or a
  vetted parsing library

**Initial scope**
- symbols
- footprints
- component parameters
- netlist and placement
- board geometry and routed copper later

### 4.2 OrCAD / Allegro

**Why it matters**
- entrenched in many serious hardware organizations
- often paired with enterprise libraries and manufacturing flows

**Challenges**
- format complexity
- split product lineage and different data domains
- enterprise-specific database/process assumptions
- high probability of metadata-preservation requirements before clean canonical
  mapping is possible

**Recommended first step**
- import from exported interchange artifacts before native project files
- focus on schematic/netlist/library extraction first

**Initial scope**
- schematic connectivity intent
- library symbols/packages
- BOM/component metadata

### 4.3 PADS

**Why it matters**
- common in mid-market professional PCB workflows
- often closer to "practical migration candidate" than Allegro-scale flows

**Challenges**
- multiple historical format generations
- different ownership across product eras
- library and rules semantics vary by vintage
- target definition is ambiguous unless version/generation and ingestion path
  are named explicitly

**Recommended first step**
- identify the most common text/exportable format path
- prefer read-only import of board/schematic essentials over early
  write-back ambitions

**Initial scope**
- library extraction
- design netlist/placement import
- board geometry later

---

## 5. Proposed Milestone Entry

Commercial interop should not enter before the engine proves value on
KiCad/Eagle.

Recommended sequence:

- `M0-M2`: no commercial import implementation
- `M3-M4`: research, corpus gathering, library extraction experiments
- `M5+`: first supported commercial migration path

That does **not** mean waiting until `M5` to think about it. It means:

- research now
- architecture discipline now
- implementation later

---

## 6. Entry Criteria Before Implementation

Do not start Altium/PADS/OrCAD implementation until all of these hold:

1. Canonical IR is stable enough that import mappings are not churn-heavy
2. Native persistence exists for imported projects (`M4`)
3. Import reporting can express partial fidelity and unsupported features
4. Query and checking layers are mature enough to validate migrated data
5. A representative corpus exists for the target tool
6. Legal/licensing posture is documented for the chosen approach

All implementation proposals must also state:

7. Which source family/version is in scope
8. Which data is expected to be exact vs approximated vs preserved-as-metadata
9. Which unsupported-loss boundaries are accepted for the slice

---

## 7. Recommended Technical Approach

### Phase A: Research and corpus

- gather representative designs and libraries
- identify real-world versions in active use
- document accessible export paths
- classify formats as text, binary, database, or hybrid

### Phase B: Library extraction

- symbols
- packages
- component parameters
- part bindings
- variants if available

This is the lowest-risk, highest-leverage starting point.

### Phase C: Read-only design import

- schematic structure
- board placement
- netlist
- essential geometry

No write-back claims yet.

### Phase D: Migration-grade reporting

Every commercial import should emit a structured report covering:

- imported feature counts
- unsupported constructs
- approximated mappings
- preserved source metadata
- confidence/fidelity summary

### Phase E: Write-back or native conversion

Only after read-only import is trusted:

- native project persistence
- ECO-safe modifications
- export/migration outputs

---

## 8. Architecture Requirements Implied Today

Even before commercial import exists, the foundation should preserve these
properties:

- importers can attach structured source metadata
- import reports can describe partial fidelity
- canonical IR avoids vendor-specific fields
- pool can absorb heterogeneous library sources
- lossiness is explicit, not silent

This is the real reason to think about commercial interop now: not to
implement it early, but to avoid blocking it accidentally.

---

## 9. Recommendation

Near-term:

- keep KiCad + Eagle as the `M0-M2` supported import contracts
- keep Eagle library import as the `M0` proving ground
- add commercial interop only as research and architecture discipline

Medium-term:

- add a commercial interop research/corpus track after `M2`
- begin with library extraction, not full design parity

Long-term:

- prioritize **Altium first**
- evaluate **PADS second**
- treat **OrCAD/Allegro** as the heaviest enterprise migration program

That ordering balances strategic value against implementation risk.
