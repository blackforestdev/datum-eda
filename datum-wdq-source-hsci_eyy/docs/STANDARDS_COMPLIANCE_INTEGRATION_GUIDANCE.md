# Standards Compliance Integration Guidance

> **Status**: Active guidance bridging the standards audit and IPC research into
> controlling spec changes.
>
> Research basis:
> [STANDARDS_AUDIT.md](/home/bfadmin/Documents/datum-eda/research/standards-audit/STANDARDS_AUDIT.md)
> and
> [IPC_COMPLIANCE_RESEARCH.md](/home/bfadmin/Documents/datum-eda/research/ipc-compliance/IPC_COMPLIANCE_RESEARCH.md).

## Purpose

Datum's current gap is not only "missing support for some standards". The real
failure mode is that research findings are landing without a controlling place
in `specs/` that says whether each finding is:

- implemented
- required for a later milestone
- reference-only vocabulary
- intentionally out of scope

This document defines the integration pattern the controlling spec must follow.

## Integration Rules

### 1. Every researched standard gets an explicit status

No researched standard may remain implicit after triage. Each standard or
standard family must be represented in controlling specs with one of these
states:

- `Implemented`
- `Planned`
- `Reference-only`
- `Deferred with prerequisite`
- `Out of scope`

Silence is not acceptable because silence is what created the current coverage
gap.

### 2. Footprint standards need first-class data, not prose promises

The IPC research is clear: Datum cannot claim serious footprint compliance if
pad geometry is stored without the source assumptions used to derive it.

The spec stack therefore needs first-class ownership for:

- IPC basis on package/padstack/library records
- density level
- source dimensions and tolerances
- mask and paste policy
- deviation tracking
- import-audit status when the source basis is unknown

`docs/IPC_FOOTPRINT_SYSTEM.md` already defines the product direction. The
controlling spec should promote the same model into a normative contract.

### 3. Schematic standards need native-authoring policy

The standards audit found a real blind spot around native symbol authoring and
schematic drawing convention. Imported symbols can inherit IEEE 315 / IEC 60617
style from source libraries, but native authoring cannot stay style-agnostic.

The controlling spec therefore needs normative policy for:

- allowed symbol-style profiles
- reference-designator policy
- title-block metadata
- sheet-size conventions
- vocabulary baseline for user-visible terms

### 4. Compliance claims need project metadata and auditability

Regulatory and lifecycle standards do not all belong in the geometry engine,
but they still require explicit project-level metadata and traceability.

The spec should require project-level placeholders and status fields for:

- IPC class and revision basis
- intended operating environment
- export-control marking posture
- materials/compliance declaration posture
- audit-log completeness
- sign-off and approval overlays when later milestones add them

### 5. Deferment must be bounded by prerequisites

Many standards are correctly deferred today, but the deferment is not bounded.
The controlling spec must tie deferrals to explicit prerequisites such as:

- stackup material properties for impedance work
- native/project metadata for materials declarations
- mechanical model ownership for STEP / IDF / IDX
- exported audit trail for regulated-process overlays

## Required Controlling-Spec Outcomes

The spec patch should establish:

1. A controlling standards-compliance spec.
2. A standards registry covering all eight audit domains.
3. Explicit target-state contracts for footprint, schematic, compliance
   metadata, audit trail, and export surfaces.
4. Milestone-gate language that prevents future research from landing without a
   controlling-spec disposition.

## Coverage Baseline

The new controlling spec should treat the following as the minimum standards
surface Datum must own explicitly.

### Encode directly in Datum contracts

- IPC-7351B / IPC-7352 / IPC-7251 footprint basis and naming policy
- IPC-7525 paste/stencil policy observables
- IPC-7093 BTC / thermal-pad policy observables
- IPC-2221 clearance-policy basis
- IPC-4761 via-type vocabulary
- IEEE 315 / IEC 60617 symbol-style policy
- IEEE 200 / ASME Y14.44 reference-designator policy
- ISO 7200 title-block field policy
- IPC-T-50 vocabulary baseline

### Represent as project metadata plus deferred capability hooks

- IPC-A-600 / IPC-A-610 class selection
- 21 CFR Part 11 / ISO 9001 / ISO 13485 audit-overlay prerequisites
- RoHS / REACH / IPC-1752A / IEC 62474 declaration posture
- ITAR / EAR marking posture
- IBIS / Touchstone / SPICE attachment hooks
- STEP / IDF / IDX / EDMD exchange prerequisites
- IPC-2581C / ODB++ / IPC-D-356 export intent

### Keep explicit but out of scope for now

- DO-254
- AS9100 process certification
- MIL-PRF-31032 / MIL-PRF-55110
- CMMI / ISO 12207 organisation-process assessments

## Change-Control Rule

Future research integration should follow one path only:

`research/*` -> `docs/*GUIDANCE*.md` -> `specs/*`

If a standards finding cannot yet be promoted into a full engine or native-file
contract, it still must be captured in the controlling standards spec as
`Planned`, `Deferred with prerequisite`, or `Out of scope`.
