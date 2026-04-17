# IPC Footprint System

> **Status**: Product and architecture direction for IPC-aware footprint
> generation, validation, deviation tracking, and imported-board audit in
> Datum.
>
> Research basis:
> [IPC_COMPLIANCE_RESEARCH.md](/home/bfadmin/Documents/datum-eda/research/ipc-compliance/IPC_COMPLIANCE_RESEARCH.md)

## Purpose

Define how Datum should handle IPC-driven footprint work so the product does
more than ship an "IPC wizard" checkbox.

Datum should support:
- IPC-aware footprint generation
- IPC-aware validation of native and imported footprints
- explicit deviation tracking
- DRC/ERC-style manufacturability diagnostics for pad, mask, paste, and
  clearance semantics

Datum should **not** claim IPC support merely because it can generate a shape
that resembles a common land pattern.

## Product Position

The industry gap identified in the research is consistent:
- many tools advertise IPC support
- most really mean "IPC-derived footprint generation"
- few preserve the source assumptions
- few validate post-edit drift rigorously
- few treat import audit against IPC as a first-class diagnostic surface

Datum should differentiate by shipping an **IPC footprint system**, not just an
IPC wizard.

That system has four parts:
1. source model
2. generator
3. validator
4. deviation-aware diagnostics

## Core Product Requirements

### 1. IPC-aware footprint generation

Datum should be able to generate footprints from an explicit IPC source model
for supported package families.

Minimum inputs:
- package family
- source dimensions and tolerances
- density level
- lead/body assumptions
- courtyard assumptions
- mask and paste policy

Minimum outputs:
- copper pad geometry
- drill/padstack geometry where applicable
- mask apertures
- paste apertures
- naming derived from the chosen IPC rule set

### 2. IPC-aware validation

Datum should validate a footprint against the chosen IPC rule set, not just
render it.

Validation must be able to answer:
- is this footprint compliant with the declared IPC basis
- if not, what is different
- is the difference intentional and accepted
- does the deviation affect manufacturability, assembly, or naming

### 3. Explicit deviation model

Datum must support footprints that intentionally deviate from the calculated IPC
result.

Examples:
- thermal-pad tuning
- hand-solder variants
- stencil reductions for specific assembly lines
- vendor-recommended exceptions
- DFM-corrected footprints such as the audited DOA2526 board

The product model should distinguish:
- `compliant`
- `compliant_with_documented_deviation`
- `non_compliant`

### 4. Import audit as a first-class diagnostic

This is the important extension you asked for.

Datum should not treat IPC only as a library-authoring concern. It should also
treat IPC as an import-validation concern.

If an imported board or footprint violates the declared or inferred IPC
observable for:
- mask expansion
- paste reduction
- annular ring
- courtyard
- clearance
- density-level geometry

Datum should be able to flag that like a DRC/ERC-style result.

That means imported-board review can answer:
- "what did the source tool define"
- "what would IPC expect"
- "is the difference intentional, unknown, or likely wrong"

## Architecture

## 1. IPC source model

Datum needs a first-class IPC source record in the canonical library model.

Suggested record shape:

```text
IpcFootprintBasis
  standard_family        // IPC-7351B, IPC-7352A, IPC-7251, IPC-7525, etc.
  package_family         // 0603 chip, SOIC, QFN, SOT-23, TO-220, ...
  density_level          // Most / Nominal / Least
  source_dimensions      // body, lead, span, tolerances
  source_j_values        // toe / heel / side where applicable
  courtyard_rule
  mask_rule
  paste_rule
  derivation_version
  naming_basis
```

This basis should attach to the canonical package/footprint object, not live as
GUI-only metadata.

## 2. Generator

The generator should derive footprint geometry from the `IpcFootprintBasis`
record and package-family calculators.

It should produce:
- padstack geometry
- courtyard
- silks/fab placement constraints where the rule set defines them
- process apertures for mask/paste
- a generated IPC-compliant name when applicable

The generator should be deterministic and regeneratable.

## 3. Validator

The validator compares a footprint's actual geometry to:
- its declared `IpcFootprintBasis`, or
- an inferred basis when no declaration exists and the package family is
  recognized

Validator outputs should be structured diagnostics, not prose.

Suggested result shape:

```text
IpcValidationReport
  status                 // compliant | compliant_with_deviation | non_compliant | unknown
  basis_used
  findings[]

IpcFinding
  severity               // info | warning | error
  category               // copper | drill | mask | paste | courtyard | naming | clearance
  observable
  expected_value
  actual_value
  tolerance
  explanation
  standards_ref
  deviation_tag
```

## 4. Deviation registry

If a user intentionally edits away from the generated IPC result, Datum should
not erase that fact.

It should store:
- which observable differs
- why
- whether the deviation is approved
- whether it is local/project-specific or library-wide

That prevents the common industry failure mode:
- footprint generated from IPC once
- edited manually later
- still described forever as "IPC compliant"

## 5. Import audit path

Imported footprints and imported board pads should go through a separate audit
path that uses the same validator engine.

This is not footprint regeneration.
It is standards comparison.

Import audit should:
- preserve source geometry exactly
- compare source geometry to IPC expectations where a supported family/basis can
  be identified
- emit diagnostics without mutating the imported data

This matters because real imported boards often contain:
- vendor defaults
- stale library geometry
- tool-derived paste/mask assumptions
- assembly-line-specific edits

Datum should show the difference, not overwrite it silently.

## Diagnostics And UX

IPC findings should be visible through the same general diagnostic surfaces as
DRC/ERC, but with their own category.

Suggested diagnostic classes:
- `IPC_COPPER_GEOMETRY`
- `IPC_MASK_EXPANSION`
- `IPC_PASTE_REDUCTION`
- `IPC_COURTYARD`
- `IPC_CLEARANCE_POLICY`
- `IPC_NAMING_DRIFT`
- `IPC_UNKNOWN_BASIS`

Suggested severity policy:
- `error`: declared compliant basis is materially violated
- `warning`: likely manufacturability issue or undeclared drift
- `info`: non-blocking deviation or unvalidated imported geometry

Examples:
- imported pad paste aperture equals copper pad, but IPC-7525 / declared stencil
  rule expects reduction
- imported solder mask aperture is smaller than required expansion
- footprint name implies nominal density, but geometry matches neither nominal
  nor least

## Observables Datum Must Treat As First-Class

The system should validate against observable geometry, not marketing labels.

Minimum required observables:
- copper pad size and shape
- hole/drill and annular ring
- mask aperture size
- paste aperture size
- paste aperture ratio / reduction policy
- courtyard envelope
- density-level naming
- package-family rule identity

This is where the IPC research matters most:
- IPC-7351B / IPC-7352 for land-pattern semantics
- IPC-7251 for through-hole families
- IPC-7525 for stencil/paste semantics
- IPC-2221 and related rules for clearance observables

## Relationship To M7

For `M7`, the immediate requirement is not a full wizard UI.

The immediate requirement is:
1. import and render standards-bound observables correctly
2. stop silently inheriting host-tool defaults
3. make standards audit possible as part of review

The recent mask/paste work is the first concrete example:
- Datum must honor imported pad-level process margins
- not rederive apertures from copper pads plus global defaults
- and not call the result "close enough"

## Rollout Phases

### Phase 1: Standards-aware observables

- preserve IPC-relevant geometry in import/native models
- render it correctly
- add standards observables to delivery gates

### Phase 2: Validator engine

- footprint/package family recognition
- expected-vs-actual comparison
- structured findings
- imported-board audit path

### Phase 3: Native authoring support

- explicit `IpcFootprintBasis`
- generated footprints
- deviation tagging
- naming assistance

### Phase 4: Wizard UX

- IPC-aware guided footprint creation
- batch generation
- regenerate / compare / accept deviation workflow

## Default Product Rule

Datum should adopt this rule across library and import work:

> If a footprint or imported pad carries manufacturability-relevant geometry,
> Datum must preserve it, explain it, and where possible validate it against the
> governing IPC observable. It must not silently replace that meaning with a
> host-EDA default.

## References

- [IPC Standards for PCB Layout and Library Compliance — Industry Survey](/home/bfadmin/Documents/datum-eda/research/ipc-compliance/IPC_COMPLIANCE_RESEARCH.md)
- [Library Architecture](/home/bfadmin/Documents/datum-eda/docs/LIBRARY_ARCHITECTURE.md)
- [M7 Delivery Gates](/home/bfadmin/Documents/datum-eda/docs/gui/M7_DELIVERY_GATES.md)
