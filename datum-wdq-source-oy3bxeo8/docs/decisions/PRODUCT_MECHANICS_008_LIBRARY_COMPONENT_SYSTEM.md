# Product Mechanics Decision 008: Library Component System

Status: draft for owner review; implementation mechanisms woven 2026-06-18.
Date: 2026-06-18

Driven by:
- `docs/DATUM_PRODUCT_MECHANICS.md`
- `docs/audits/scope-integration/DATUM_PRODUCT_MECHANICS_RESEARCH_SYNTHESIS.md`
- `docs/audits/scope-integration/DATUM_PRODUCT_MECHANICS_REVIEW_AGENDA.md`
- `specs/STANDARDS_COMPLIANCE_SPEC.md`
- `docs/IPC_FOOTPRINT_SYSTEM.md`
- `docs/STANDARDS_COMPLIANCE_INTEGRATION_GUIDANCE.md`
- `docs/STANDARDS_AUDIT_BATCH_1_GUIDANCE.md`
- `research/ipc-compliance/IPC_COMPLIANCE_RESEARCH.md`

## Decision Scope

Define Datum's native library and component model.

This decision covers reusable symbols, footprints, parts, packages, padstacks,
pin/pad mapping, attached models, metadata, lifecycle data, standards basis,
and provenance. It does not define the full library editor UI or final storage
layout.

## Product Intent

Datum's library system should be native product data, not a thin wrapper around
imported KiCad, Eagle, vendor, or spreadsheet records.

The library should let users answer professional design questions:
- what electrical function does this part expose
- what schematic symbol represents it
- what physical package and footprint implement it
- what manufacturer/orderable identity is bound
- what package dimensions and tolerances drove the footprint
- what pad, mask, paste, courtyard, and drill policies apply
- what simulation, SI, thermal, or 3D models are attached
- where the data came from and whether it was reviewed

Imported libraries may seed Datum libraries, but they must be converted into
Datum primitives with explicit provenance and unknown-basis markers where the
source data does not carry enough standards or derivation detail. Imported
library data routes through the Import Map keyed by `import_key`, which
preserves original source provenance; `source_hash` is trace data, not the
identity key.

## Decision

Datum shall use a first-class library/component system built from explicit
`Part`, `Package`, `Symbol`, `Footprint`, `Padstack`, `PinPadMap`,
`ModelAttachment`, standards-basis, metadata, and provenance primitives.

The system must support both reusable pool/library content and project-local
library content. A placed component must bind to a resolved library identity
while retaining pinned revision, snapshot/provenance information, and any
project-local override records required for reproducible builds, audit, BOM
generation, standards-aware checking, and import review.

Library objects are `DomainObject`s in the unified `DesignModel`, not private
library-file records. Each library object has a persisted `ObjectId = Uuid`,
`object_revision`, semantic version where useful, provenance, review state, and
standards basis where applicable. Pool paths, names, manufacturer part numbers,
symbol names, footprint names, and package codes are searchable labels; they
are not identity. Moving a record between pools or renaming it preserves object
identity.

Datum must preserve the separation between:
- logical/electrical representation
- schematic graphic representation
- physical package identity
- footprint/land-pattern geometry
- manufacturer/orderable part metadata
- attached behavioural, thermal, and mechanical models
- project-specific placement and overrides

Bindings between these records are explicit references by stable ID and pinned
revision. A project may choose to follow a library update only through an
operation/proposal that updates the binding and records the before/after
library revisions. There is no silent "latest library wins" behavior.

## User-Visible Behavior

Users should be able to browse, create, edit, duplicate, import, review, and
approve library content through a dedicated library projection.

Expected user-visible behavior:
- a part detail view shows symbol, footprint, package, pin/pad map, metadata,
  models, lifecycle, standards basis, provenance, and check state
- symbol and footprint editors expose native Datum objects rather than raw
  imported format text
- footprint views expose copper, drill, mask, paste, courtyard, fab, silkscreen,
  and mechanical/process geometry
- IPC/process basis and deviation state are visible near the geometry they
  govern
- part assignment in schematic or PCB layout shows whether the selected part is
  approved, deprecated, unknown, imported, or project-local
- a BOM view can trace placed components back to part, manufacturer, orderable,
  qualification, lifecycle, and compliance metadata
- library changes that affect placed design objects appear as reviewable
  proposals or ECO-style updates, not silent global mutation

## Manual Workflow Requirements

Datum must be usable for library work without AI.

Manual workflows must include:
- create and edit symbols with pins, units, graphics, fields, and style-profile
  assertions
- create and edit footprints with padstacks, pads, mask/paste/courtyard policy,
  silkscreen/fab/mechanical geometry, and origins
- define packages with dimensions, tolerances, body height, mounting data, and
  model references
- define parts with manufacturer, MPN, lifecycle, parametrics, datasheet,
  qualification, supply-chain, compliance, and model metadata
- map logical pins to physical pads through explicit `PinPadMap` records
- attach 3D, IBIS, Touchstone, SPICE, AMI, and thermal model references as
  metadata even when execution or validation is deferred
- mark data as approved, deprecated, imported, unknown basis, or requiring
  review
- record documented deviations when footprint or metadata choices intentionally
  differ from a selected basis

## Optional AI And Tooling Behavior

AI and external tools may assist library work, but they must operate through
the same primitives and proposal model as manual users.

Allowed behavior:
- suggest symbol or footprint creation from datasheet-derived dimensions
- extract candidate metadata from datasheets, vendor files, or imported
  libraries
- propose IPC footprint generation or process-aperture corrections
- compare a package/footprint against a declared standards basis
- identify missing pin/pad mappings, model attachments, or lifecycle metadata
- propose alternate parts or supply-chain enrichments through gated tools

Required guardrails:
- AI-generated or extracted library data must be marked with provenance and
  review state
- attached encrypted vendor models may be preserved as pass-through metadata,
  but Datum must not decrypt them
- external lookup tools must respect project data-egress policy
- AI may not silently approve library content or rewrite placed design
  bindings without a proposal and explicit acceptance

## Core Primitives

### LibraryPool

A reusable collection of Datum-native library records.

Library pools may be bundled, user, organization, vendor-imported, or
project-local. Pool layering must make override and provenance behavior visible.
Pool membership is not identity; it is a source/availability layer resolved by
the `ProjectResolver` into library objects in the `DesignModel`.

### Part

Reusable component identity suitable for selection, BOM, lifecycle, and design
binding.

Implementation schema concept:

```text
Part
  id: ObjectId
  object_revision: u64
  display_name
  manufacturer_part_numbers[]
  orderable_part_numbers[]
  lifecycle_status
  approval_state
  parametrics{}
  datasheet_refs[]
  qualification_metadata{}
  compliance_metadata{}
  behavioural_models: ModelAttachmentRef[]
  package_options: PackageRef[]
  default_symbol: SymbolRef?
  default_footprint: FootprintRef?
  default_pin_pad_map: PinPadMapRef?
  provenance: ProvenanceSet
```

A `Part` describes the component users select and buy. It must not absorb the
schematic symbol, physical body, or land pattern as anonymous subfields. Those
remain separately addressable objects so checks, ECOs, package swaps, and
import repair can target the right authority.

### Symbol

Schematic representation of electrical intent.

Implementation schema concept:

```text
Symbol
  id: ObjectId
  object_revision: u64
  units[]
  pins[]
  graphics[]
  fields[]
  style_profile_assertions[]
  default_refdes_prefix?
  standards_basis[]
  provenance: ProvenanceSet
  check_state
```

Symbols must carry pin identity, units, graphics, fields, style-profile
assertions, source/provenance, and check state. Symbol style profiles support
planned IEEE 315, IEC 60617, JIS C 0617, imported-custom, and mixed assertions
without claiming visual certification. Pin IDs are stable within the symbol and
are the logical side of `PinPadMap`.

### Package

Physical component body and lead identity.

Implementation schema concept:

```text
Package
  id: ObjectId
  object_revision: u64
  package_family
  package_code?
  mounting_type
  body_dimensions
  body_height_nm?
  body_height_mounted_nm?
  lead_or_terminal_geometry
  tolerances
  source_dimensions
  standards_basis[]
  model_attachments: ModelAttachmentRef[]
  provenance: ProvenanceSet
```

Packages carry family, dimensions, tolerances, body height, mounted height,
lead/pin geometry assumptions, mounting type, package provenance, and model
references. The package is the source of physical body truth; the footprint is
the board land-pattern/process realization.

### Footprint

Physical land pattern and process geometry.

Implementation schema concept:

```text
Footprint
  id: ObjectId
  object_revision: u64
  package_ref: PackageRef?
  pads[]
  padstack_refs[]
  copper_geometry
  courtyard_geometry
  fab_geometry
  silkscreen_geometry
  mechanical_geometry
  origin
  placement_constraints
  process_aperture_policy
  standards_basis[]
  deviation_refs[]
  provenance: ProvenanceSet
```

Footprints carry pads, padstacks, copper geometry, drill geometry,
solder-mask policy, paste/stencil policy, courtyard policy, fab/silkscreen
geometry, origin, placement constraints, standards basis, deviations, and
provenance. A footprint may reference generated `Zone` boundaries, but the
filled copper result is still derived `ZoneFill` state with provenance and
staleness, never footprint authority.

### Padstack

Reusable pad/drill/process-aperture definition.

Implementation schema concept:

```text
Padstack
  id: ObjectId
  object_revision: u64
  pad_kind
  copper_shapes_by_layer
  drill: DrillSpec?
  annular_ring_policy?
  mask_aperture_policy
  paste_aperture_policy
  via_or_component_terminal_policy
  standards_basis[]
  provenance: ProvenanceSet
```

Padstacks must support layer-specific copper, drill, annular-ring, mask, and
paste geometry so standards-aware DRC can reason about process apertures rather
than only copper. Mask and paste are not assumed to equal copper. Their policy
must explicitly state whether an aperture is generated from copper expansion,
generated from stencil reduction, suppressed, split, manually authored, or
import-preserved. Unknown or inherited source-tool behavior is represented as
`UnknownBasis` or `ImportPreserved`, not silently normalized.

### PinPadMap

Explicit relationship between logical pins and physical pads.

The map is required for ERC/DRC correlation, forward/back annotation, import
review, BOM validation, simulation export, and package replacement.

Implementation schema concept:

```text
PinPadMap
  id: ObjectId
  object_revision: u64
  symbol_ref: SymbolRef
  package_ref: PackageRef
  footprint_ref: FootprintRef
  entries[]
    logical_pin_id
    package_terminal_id?
    footprint_pad_id
    electrical_role?
    variant_condition?
  provenance: ProvenanceSet
```

The map is the canonical bridge from logical pins to physical pads, and it is
the library-side binding that a placed `ComponentInstance` uses to join its
symbol and package/footprint projections across domains; cross-domain
correlation keys on the `ComponentInstance` and stable object IDs, never on
reference designators or names. It is not derived from matching names at check
time. Name matching may create an import or repair proposal, but the accepted
relationship is a stable object committed through `commit()`.

`variant_condition` references the sparse variant overlay's authored
Fitted/Unfitted and option-link selections (see Decision 003); it never persists
derived `NotApplicableForVariant` or any other derived population state.

### ModelAttachment

Reference to behavioural, SI, thermal, or mechanical model data.

Model attachments should support format, role, dialect, encrypted-content
policy, provenance, transform where applicable, hash identity, and validation
state. Attachment is not the same as simulator execution.

Implementation schema concept:

```text
ModelAttachment
  id: ObjectId
  object_revision: u64
  target_object_id
  model_role
  model_format
  dialect_or_revision?
  content_ref
  content_hash
  encrypted_content_policy
  transform_3d?
  validation_state
  format_metadata{}
  provenance: ProvenanceSet
```

Encrypted models may be retained as pass-through content with metadata and
hashes. Datum must not decrypt vendor-encrypted content. Execution is deferred
unless a later solver/subprocess milestone promotes it.

### LibraryProvenance

Trace of source and review history for library data.

Implementation schema concept:

```text
LibraryProvenance
  source_type: Native | Imported | Generated | Vendor | User | Script | AI | ExternalService
  source_ref?
  source_hash?
  source_object_key?
  import_session_id?
  derivation_basis[]
  actor?
  timestamp?
  review_state
  lossiness_flags[]
  unknown_basis_flags[]
  acceptance_transaction_id?
```

Provenance is attached to individual library objects and may also be grouped in
a `ProvenanceSet` when one imported/generated source produced several objects.
Accepted repairs add new provenance entries; they do not overwrite original
import provenance.

Imported library objects keep `ObjectId = Uuid` persist-on-create identity; any
imported deterministic source UUID is demoted to an import seed. Re-association
of an imported library object across a re-import is keyed by the Import Map
`import_key` recorded in the Import Map shard, never by `source_hash`. The
`source_hash`, `source_object_key`, and `import_session_id` fields are
provenance trace data that preserve original source provenance and unknown-basis
markers; they are never the reload or re-association identity key.

### LibraryBinding

Project use of a library object at a known revision.

Implementation schema concept:

```text
LibraryBinding
  id: ObjectId
  target_object_id
  pinned_object_revision
  pool_ref?
  local_override_refs[]
  binding_role
  provenance
```

Placed `ComponentInstance`s bind to `Part`, `Symbol`, `Package`, `Footprint`,
`PinPadMap`, and `ModelAttachment` records through `LibraryBinding`s. Updating
a binding, accepting a library ECO, or converting an imported anonymous
footprint into a reusable library footprint is an operation/proposal committed
through the journal. When a library change affects existing boards, panels,
checks, or artifacts, propagation is proposal-first and `commit()` / journal
routed; there is no silent global mutation of placed design objects.

## Standards And Compliance Impact

Standards must drive library design by shaping metadata, generation, checking,
deviation tracking, and user warnings.

Datum must not claim that a part, footprint, library, or project is certified
because it stores standards metadata or passes a Datum check. Datum may state
the declared basis, validation result, deviation state, and evidence available
inside the project.

Required standards-facing behavior:
- IPC footprint basis is first-class data on packages/footprints/padstacks
- supported density level, source dimensions, tolerances, J-values, naming
  basis, courtyard policy, mask policy, and paste policy are stored
- footprint compliance states distinguish compliant, compliant with documented
  deviation, non-compliant, and unknown basis
- schematic symbols carry style-profile assertions and provenance
- reference-designator and title-block standards influence warnings and
  metadata, not automatic user intent replacement
- part qualification, environmental declarations, and export-control metadata
  are metadata surfaces unless a later milestone defines stronger workflows

Pad/mask/paste aperture policy is part of this standards impact. A pad that
has copper geometry but no explicit mask/paste basis is checkable as
unknown-basis data. A pad that copies copper into mask or paste because an
imported source did so remains import-preserved until the user accepts a
proposal such as `SetPadProcessAperture` or `ApplyFootprintProcessPolicy`.

## Non-Goals

This decision does not:
- require copying KiCad symbol libraries into Datum
- make imported library formats Datum's native architecture
- implement every IPC package-family generator immediately
- claim third-party certification, IPC certification, regulatory compliance, or
  library correctness
- require behavioural-model execution
- require supply-chain service integration before data-egress policy exists
- force all projects to use approved central libraries only

## First Proof Slice

The first proof slice should implement a minimal native part flow:

1. Create one reusable `Part` with one `Symbol`, one `Package`, one
   `Footprint`, one `Padstack` family, and one `PinPadMap`.
2. Store IPC/process basis and provenance on the footprint/padstack.
3. Place the part as a `ComponentInstance` in a project and bind its schematic
   and PCB projections to the same library identity through that instance.
4. Seed one imported library object, re-associate it across a re-import via the
   Import Map `import_key` (not `source_hash`), and preserve its source
   provenance and unknown-basis markers.
5. Run a check that reports missing/unknown library basis and one pad/mask/paste
   policy mismatch.
6. Show a reviewable proposal for a standards-aware process-aperture correction
   without silently mutating the library; any library change that touches placed
   `ComponentInstance`s is proposal-first and committed through `commit()` /
   journal, never silently propagated to existing boards, panels, checks, or
   artifacts.

## Owner Questions

- Which library pool types should exist first: bundled Datum, user-local,
  organization, project-local, imported, or vendor-derived?
- What approval states are required before a part can be placed in a design?
- Should IPC-7351B or IPC-7352 naming be the default for generated footprints?
- Which first package families should the native generator/checker support?
- How strict should unknown-basis imported library data be in default checks?
- What part qualification and environmental metadata must be visible in v1 BOM
  workflows?
