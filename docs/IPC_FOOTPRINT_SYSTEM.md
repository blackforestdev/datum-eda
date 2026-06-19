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
- emit DRC/check diagnostics without mutating the imported data
- offer explicit correction proposals when the delta is mechanically
  actionable and the governing basis is known or user-selected

This matters because real imported boards often contain:
- vendor defaults
- stale library geometry
- tool-derived paste/mask assumptions
- assembly-line-specific edits

Datum should show the difference and provide a controlled fix path, not
overwrite it silently.

Example:
- a pad's solder-mask aperture is inherited from copper instead of carrying
  the required positive mask expansion
- a pad's paste aperture is inherited from copper instead of carrying the
  required stencil reduction
- neighboring footprints of the same class use the expected mask/paste offsets,
  but one footprint or package instance does not

In that case Datum's DRC/checking surface should preserve the imported KiCad
geometry, report the mask/paste observable delta, and offer a proposal such as:

```text
propose_process_aperture_correction
  target: footprint/package/pad set
  basis: declared IPC/stencil rule or user-selected project rule
  mask_delta: +5 mil
  paste_delta: -5 mil
  affected_pads: [...]
  findings_resolved: [...]
```

The DRC finding is the required detection mechanism. The proposal is the
follow-on repair path. The proposal must be reviewable and explicitly accepted
before Datum mutates the design. The accepted change should record provenance
that it corrected an imported host-tool default rather than original native
Datum-authored geometry.

## Diagnostics And UX

IPC/process-geometry findings should be visible through Datum's DRC/checking
surface, with their own category where useful. A board with pad/mask/paste
geometry that violates the declared or selected process basis should not pass
DRC cleanly merely because the source file was accepted by the host EDA tool.

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

Suggested DRC finding codes:
- `pad_mask_expansion_missing`
- `pad_mask_expansion_below_rule`
- `pad_paste_reduction_missing`
- `pad_paste_reduction_below_rule`
- `pad_process_aperture_inherited_from_copper`
- `pad_process_aperture_inconsistent_with_peer_footprint`

Examples:
- imported pad paste aperture equals copper pad, but IPC-7525 / declared stencil
  rule expects reduction
- imported solder mask aperture is smaller than required expansion
- imported pad mask/paste apertures are implicitly inherited from copper while
  comparable pads in the same design carry explicit process offsets
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
3. make standards audit visible through DRC/check findings
4. prepare explicit correction proposals for actionable mask/paste/default
   inheritance findings, without silently applying them

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
- structured DRC/check findings
- imported-board audit path
- proposed process-aperture corrections for recognized pad/mask/paste deltas

### Phase 3: Native authoring support

- explicit `IpcFootprintBasis`
- generated footprints
- deviation tagging
- naming assistance
- correction-acceptance flow that records whether a mask/paste/stencil change
  is library-wide, package-specific, or project-local

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
