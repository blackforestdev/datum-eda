# Library Architecture

> **Status**: Governing architecture direction for Datum library development.
> This document sets product and system direction for library handling.
> Detailed pool storage mechanics remain in [POOL_ARCHITECTURE.md](/home/bfadmin/Documents/datum-eda/docs/POOL_ARCHITECTURE.md).

## Purpose

Define how Datum should handle schematic symbols, parts, packages, footprints,
padstacks, and imported library sources so the product can evolve from import-
driven interoperability into a credible native EDA system.

This document exists because `M7` exposed a real product problem: GUI review
quality is limited by library fidelity. If Datum does not have a coherent
library subsystem, every downstream surface becomes partially ad hoc:

- schematic authoring
- package/footprint rendering
- part replacement
- board review fidelity
- AI-assisted explanation and search
- import provenance and migration

This direction now also includes an explicit IPC-aware footprint system for
generation, validation, deviation tracking, and imported-board audit:
- [IPC_FOOTPRINT_SYSTEM.md](/home/bfadmin/Documents/datum-eda/docs/IPC_FOOTPRINT_SYSTEM.md)

## Research-Driven Conclusions

### 1. Horizon's library architecture is useful prior art, not the target ceiling

[HORIZON_ANALYSIS.md](/home/bfadmin/Documents/datum-eda/docs/HORIZON_ANALYSIS.md)
shows why explicit library identity and separation matter:

- `Unit`: electrical identity
- `Symbol`: schematic representation
- `Entity`: logical multi-gate component
- `Package`: physical package/terminal family in Horizon's terminology
- `Part`: purchasable component binding entity to package

This is stronger than KiCad's flat symbol/footprint pairing because it gives
the system an explicit place to represent:

- pin-to-pad binding
- multi-gate devices
- package variants
- part variants
- manufacturer/MPN/lifecycle metadata

Datum should adopt the explicit-identity lesson, not Horizon's exact physical
model. Horizon fuses package body and PCB land pattern into `Package`; Datum's
ratified product mechanics split them. `Package` is the physical component
body/terminal family. `Footprint` is the PCB land pattern, courtyard,
fabrication/assembly graphics, pad geometry, and mask/paste process aperture
policy. This split is mandatory because Datum is targeting commercial-grade
library governance, standards validation, and schematic-to-PCB authoring, not
just a cleaner KiCad/Horizon clone.

### 2. KiCad is a valuable source library, not the architecture to copy

KiCad libraries are useful because they are:

- widely available
- text-based
- easy to seed from
- familiar to users

But KiCad does not provide a first-class "Part" abstraction. Symbol-to-
footprint binding happens at design time, not as a strong library object. That
is useful for interchange, but not sufficient as Datum's native library model.

Datum should import from KiCad. Datum should not inherit KiCad's flat library
architecture as the product model.

### 3. Eagle libraries are strategically useful because they encode explicit binding

Eagle's `deviceset -> device -> connect` chain maps well onto Datum's desired
canonical model. The repo already recognizes this in
[POOL_ARCHITECTURE.md](/home/bfadmin/Documents/datum-eda/docs/POOL_ARCHITECTURE.md),
[EAGLE_BLUEPRINT.md](/home/bfadmin/Documents/datum-eda/docs/EAGLE_BLUEPRINT.md),
and [INTEROP_SCOPE.md](/home/bfadmin/Documents/datum-eda/docs/INTEROP_SCOPE.md).

That means Eagle libraries are not just migration baggage; they are a useful
source corpus for validating Datum's canonical library semantics.

### 4. Datum needs both import and native authoring eventually

Imported libraries can seed the system, but a serious EDA product needs a
native library subsystem:

- stable UUID-based identity
- canonical normalized library data
- deterministic serialization
- search/index/query
- explicit provenance
- future editing/curation workflows

Imported libraries should feed the subsystem. They should not remain the
primary long-term source of truth.

### 5. Datum should validate IPC observables, not just generate IPC-shaped footprints

The industry research for Datum shows a recurring failure mode: tools advertise
IPC support through generators and naming, but do not rigorously validate
post-edit or imported geometry against IPC-relevant observables.

Datum should treat IPC as:
- a generation basis
- a validation basis
- an import-audit basis

That means Datum should eventually be able to:
- generate footprints from an explicit IPC basis
- validate native footprints against that basis
- detect deviations
- flag imported-board/process geometry that does not match the expected IPC
  observable

The concrete product and architecture direction for that system is in:
- [IPC_FOOTPRINT_SYSTEM.md](/home/bfadmin/Documents/datum-eda/docs/IPC_FOOTPRINT_SYSTEM.md)

## Product Goals

Datum's library subsystem should:

- provide a canonical internal model for symbols, parts, packages, footprints,
  and padstacks
- preserve source provenance from imported libraries
- support project-local, shared, and shipped library roots
- allow the GUI to render familiar library-native geometry rather than proxy
  stand-ins
- support deterministic search and replacement flows
- support future AI-assisted lookup, explanation, and migration guidance

Datum's library subsystem is **not** trying to be:

- a thin wrapper over KiCad's symbol/footprint pairing
- an opaque cache of imported source files
- a GUI-only asset folder
- a mutable side system that bypasses the engine's canonical IR

## Canonical Datum Library Model

Datum should continue with the canonical model already established in the repo,
with this document making it explicit as product direction:

```text
Unit
  -> Symbol
Entity
  -> Part
Package
  -> Footprint
       -> Padstack
PinPadMap
  -> Entity pins
  -> Package terminals
  -> Footprint pads
LibraryBinding
  -> ComponentInstance
  -> pinned Part/Symbol/Package/Footprint/PinPadMap revisions
```

### Unit
- electrical identity of one logical gate or one simple component electrical form
- owns pins and pin semantics

### Symbol
- schematic visual representation of a unit
- owns symbol graphics, pin stubs, text anchors, display metadata

### Entity
- logical component definition
- maps one or more gates to units and symbols
- owns prefix and gate structure

### Package
- physical component package definition
- owns body dimensions, tolerances, terminal family, mounting style, body
  height, package-level metadata, and package-level model references
- does not own PCB land-pattern geometry in the target Datum model

### Footprint
- PCB land-pattern definition for one package/process basis
- owns pads referencing padstacks, courtyard, fabrication/silkscreen/mechanical
  layers, assembly cues, origin, process-aperture policy, standards basis, and
  footprint-level model references
- is the object generated by IPC/package-family tooling and validated against
  pad/mask/paste/courtyard observables

### Padstack
- pad geometry and drill/mask/paste semantics
- should become first-class so drilled/via/pad rendering is not reduced to
  simplistic shape tags

### Part
- purchasable or selectable component
- binds entity to package/footprint choices
- owns default symbol/package/footprint choices and parametric/lifecycle data
- does not own the authoritative pin-to-pad table in the target model
- owns MPN/manufacturer/value/lifecycle/parametric metadata

### PinPadMap
- first-class mapping object from logical pins to package terminals and
  footprint pads
- supports variants, pin-swap/group metadata, and reviewable migration from
  imported or legacy `Part.pad_map` shapes

### Behavioural model attachment

A `Part` may carry zero or more **behavioural models** describing its
electrical, thermal, or simulation behaviour. This elevates library
scope from symbol/footprint/parametric metadata only into
behavioural-model territory.

Supported attachment kinds (per `specs/ENGINE_SPEC.md` § 1.1a
`ModelRole`):

- **IBIS** (`.ibs`) — I/O buffer characteristics; the IBIS-AMI variant
  attaches as a bundle (`.ami` + per-platform binary cosim libraries)
- **SPICE** (`.cir` / `.lib` / `.sub` / `.inc`) — analog and
  mixed-signal models across Berkeley3 / ngspice / LTspice / PSpice /
  HSpice / Xyce / Spectre dialects
- **Touchstone** (`.s1p` … `.sNp`) — S-parameter measurements
- **IBIS-ISS** — IBIS interconnect SPICE subcircuit
- **Verilog-A / Verilog-AMS / VHDL-AMS** — compact device models
  (attachment only; no Datum-side simulator)
- **Compact Thermal** (ECXML, JESD15-4) — multi-node thermal networks

Storage and identity:

- model files live in `pool/models/<role>/<sha256>.<ext>` per
  `docs/POOL_ARCHITECTURE.md` § 2 and `specs/NATIVE_FORMAT_SPEC.md`
  § 6.10
- model UUIDs are deterministic from the file SHA-256, so two pools
  carrying the same vendor IBIS file resolve to the same model
  identity without coordination
- pool layering applies: a project-local pool may attach a different
  model to the same `Part` UUID than a shared/shipped pool; standard
  pool-priority resolution decides which wins

Authoring discipline:

- model attach/detach is an authored operation (`AttachModel` /
  `DetachModel` per `specs/ENGINE_SPEC.md` § 3) so the audit trail
  and undo/redo stack remain correct
- encrypted vendor models (IBIS BIRD-176, PSpice / HSpice / LTspice /
  Spectre encryption) are stored verbatim and **never decrypted by
  the engine**; the MCP layer enforces an AI-safety gate on
  encrypted-content exposure (per
  `specs/MCP_API_SPEC.md` § Encrypted Content Handling Policy)
- behavioural-model authoring (creating IBIS or SPICE files inside
  Datum) is explicitly out of scope for v1; Datum is an attachment
  consumer, not an authoring tool, for behavioural models

Simulator integration:

- Datum integrates external simulators (ngspice and similar) as
  **subprocesses only** to keep Datum's distributable license
  unconstrained by GPL-class linkage; this is a project-wide
  invariant
- `validate_spice` and `export_ibis_stimulus` MCP tools invoke
  ngspice via subprocess, never as a linked library

Provenance:

- every attached model carries a `ModelProvenance` record (source
  URL or local path, vendor identity normalised to JEP106 where
  possible, fetched-at timestamp, content SHA-256) per
  `specs/ENGINE_SPEC.md` § 1.1a
- the same provenance shape is shared with `ModelRef` (3D geometry)
  so library tooling has one provenance contract for all
  attachments

## Architectural Principles

### 1. Library truth belongs to the engine, not the GUI

The GUI may inspect and render library-backed data, but it must not become a
parallel footprint or symbol manager. Canonical library objects remain engine
truth.

### 2. Imported source libraries are inputs, not runtime truth

Datum should ingest source libraries from KiCad, Eagle, Horizon-compatible
exports, and future formats into canonical pool/library objects. Runtime
behavior should depend on canonical Datum objects, not on reparsing foreign
libraries on every use.

### 3. Provenance must be preserved

Every imported canonical object should retain:

- source format
- source path or source library identity
- source object name
- importer version
- fidelity notes where needed

This matters for trust, migration, debugging, and future reimport behavior.

### 4. The library subsystem must support both reuse and overrides

Datum should support:

- shipped base libraries
- shared team/org libraries
- project-local overrides or additions

Search order and shadowing rules should be explicit and deterministic.

### 5. GUI rendering should consume library-native geometry when available

If a package has real silkscreen, courtyard, fab, text, padstack, and drill
semantics, the review surface should render those. Coarse proxy bounds are
acceptable only as fallback, not as the main long-term rendering path.

## Library Sources And Roles

### Horizon-style model

Role:
- architecture reference
- strongest model for native Datum library semantics

Adopt:
- separation of unit/entity/part/package
- library index/search
- pool-style local storage

Do not copy directly:
- Horizon implementation details
- GTK/OpenGL-era editor assumptions

### Eagle libraries

Role:
- high-value import source
- good validation corpus for explicit device binding

Use for:
- importer validation
- canonical model testing
- pin-to-pad mapping preservation

### KiCad libraries

Role:
- practical seed libraries
- immediate familiar geometry source for GUI evaluation

Use for:
- symbol and package ingestion
- initial shipped reference corpus
- GUI evaluation boards and known-good references

Constraint:
- KiCad import into Datum is inherently lossy at the "Part" layer unless a
  design-level pairing or explicit binding policy exists

### External research corpus

Paths already useful on this machine:

- `~/sandbox/eagle-analysis/eagle-9.6.2/lib/`
- system KiCad libraries under `/usr/share/kicad/footprints/`
- future symbol libraries under KiCad `.kicad_sym`
- Horizon research corpus where legally and technically usable

These should be treated as seed and validation inputs, not as Datum's native
library format.

## Storage Strategy

Datum should keep the current pool direction:

- JSON files as human-readable source of truth
- SQLite index as derived search/query layer
- deterministic UUID identity
- deterministic serialization

This already aligns with:

- [POOL_ARCHITECTURE.md](/home/bfadmin/Documents/datum-eda/docs/POOL_ARCHITECTURE.md)
- [CANONICAL_IR.md](/home/bfadmin/Documents/datum-eda/docs/CANONICAL_IR.md)
- [NATIVE_FORMAT_SPEC.md](/home/bfadmin/Documents/datum-eda/specs/NATIVE_FORMAT_SPEC.md)

The important clarification is product intent:

- the pool is not only an import artifact cache
- the pool is Datum's library subsystem foundation

## Import Architecture

### Phase 1: Source ingestion

Importers should ingest foreign library objects into canonical Datum objects:

- Eagle `.lbr` -> unit/entity/package/part
- KiCad `.kicad_sym` -> unit/symbol/entity
- KiCad `.kicad_mod` -> package/padstack
- design import can create parts where foreign libraries lack explicit part semantics

### Phase 2: Canonicalization

Import must produce:

- deterministic UUIDs
- normalized geometry in nanometers
- explicit layer roles
- explicit padstack semantics
- explicit binding objects where source data permits them

### Phase 3: Indexing and query

The library subsystem should support:

- keyword search
- parametric search
- tag search
- package compatibility search
- part replacement candidates
- symbol/package direct lookup by UUID

### Phase 4: Provenance-aware reimport

Eventually Datum should support source-aware reimport/update behavior, but not
by mutating live runtime truth from foreign formats. Reimport should flow
through canonical objects with explicit review.

## Immediate Practical Direction

### What Datum should do now

1. Continue using the canonical pool model as the native library architecture.
2. Treat Horizon as the primary architecture reference.
3. Use KiCad and Eagle libraries as the immediate seed corpus.
4. Improve GUI fidelity by consuming real imported library geometry instead of
   synthetic placeholders.
5. Formalize padstack and footprint-text support as first-class imported data.

### What Datum should not do now

- invent a new flat symbol+footprint-only library model
- delay GUI progress until a perfect library editor exists
- keep M7 dependent on synthetic footprint proxies once real sources are
  available
- let the GUI become the place where library truth is silently reconstructed

## Development Priorities

### Near-term

1. **Library-backed reference boards for GUI work**
- known-good demo boards should use imported recognizable library geometry
- this is required for meaningful human review

2. **Padstack fidelity**
- drilled through-hole pads
- richer SMD pad semantics
- layer-aware mask/paste/copper roles

3. **Footprint text and graphics fidelity**
- reference/value text
- silkscreen/fab/courtyard separation
- proper use of package-native graphics

4. **KiCad library import improvement**
- better direct ingestion of `.kicad_mod`
- clearer mapping from KiCad artifacts into Datum packages/padstacks

### Medium-term

1. **Native library browsing/search UX**
- search parts, packages, symbols, padstacks
- inspect provenance and compatibility

2. **Project/library reference management**
- project-local library roots
- external library roots
- shipped base library

3. **Part binding workflows**
- assign part
- change package
- explicit compatibility explanation

### Longer-term

1. **Native library authoring**
- create/edit symbols
- create/edit packages
- create/edit parts

2. **Curated shipped Datum library**
- project-starter quality base corpus
- deterministic IDs and provenance

## Relationship To Existing Docs

This document does not replace existing pool documentation. It clarifies
product intent and priorities.

- [POOL_ARCHITECTURE.md](/home/bfadmin/Documents/datum-eda/docs/POOL_ARCHITECTURE.md)
  remains the detailed storage/query rationale.
- [HORIZON_ANALYSIS.md](/home/bfadmin/Documents/datum-eda/docs/HORIZON_ANALYSIS.md)
  provides the strongest reference for the native model.
- [INTEROP_SCOPE.md](/home/bfadmin/Documents/datum-eda/docs/INTEROP_SCOPE.md)
  remains the source-format staging guide.

## Acceptance Criteria For Calling The Library Direction Viable

Datum's library direction is on track when:

- GUI review surfaces can render real imported package geometry without proxy
  stand-ins for common parts
- package, symbol, and part identity are canonicalized into stable Datum IDs
- Eagle and KiCad libraries can both feed the same canonical pool model
- part replacement and compatibility queries operate on native library objects,
  not source-format-specific hacks
- the project can explain, for any placed component, which entity, package,
  part, and source provenance it came from

## Immediate Recommendation

Use the current `M7` pressure as the forcing function:

- keep improving the GUI
- but stop treating library fidelity as a rendering-only problem
- continue moving demo/reference boards onto imported real library geometry
- then use that same path to harden the broader Datum library subsystem

This is the shortest path from "GUI review looks synthetic" to "Datum has a
real library architecture."
