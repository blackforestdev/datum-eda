# Product Mechanics Decision 010: Industry Standards And Compliance

Status: draft for owner review; implementation mechanisms woven 2026-06-18.
Date: 2026-06-18

Driven by:
- `docs/DATUM_PRODUCT_MECHANICS.md`
- `docs/audits/scope-integration/DATUM_PRODUCT_MECHANICS_RESEARCH_SYNTHESIS.md`
- `docs/audits/scope-integration/DATUM_PRODUCT_MECHANICS_REVIEW_AGENDA.md`
- `specs/STANDARDS_COMPLIANCE_SPEC.md`
- `docs/STANDARDS_COMPLIANCE_INTEGRATION_GUIDANCE.md`
- `docs/STANDARDS_AUDIT_BATCH_1_GUIDANCE.md`
- `docs/audits/scope-integration/STANDARDS_DOMAINS_3_4_INTEGRATION_GUIDANCE.md`
- `research/standards-audit/STANDARDS_AUDIT.md`
- `research/ipc-compliance/IPC_COMPLIANCE_RESEARCH.md`

## Decision Scope

Define Datum's product role for industry standards and compliance.

This decision covers how standards influence product design, metadata, rules,
checks, import/export, auditability, and user-facing claims. It does not decide
that Datum implements every researched standard.

## Product Intent

Datum should treat standards as product mechanics, not marketing decoration.

Standards should shape:
- library metadata
- schematic conventions
- footprint and process geometry
- design rules
- check output
- import audits
- manufacturing projections
- project compliance metadata
- data-egress policy
- audit and sign-off surfaces

At the same time, Datum must be precise about its claims. Datum is compliance
substrate and standards-aware tooling. Datum is not the certifying authority
for a user's product, organization, manufacturing process, regulatory program,
or industry qualification.

## Decision

Datum shall maintain explicit standards and compliance posture throughout the
product.

Every standards-facing feature must identify whether the relevant standard is
implemented, planned, reference-only, deferred with prerequisite, or out of
scope according to the controlling standards-compliance spec.

Standards may drive product design, validation, metadata, warnings, reports,
and export behavior. Standards metadata and checks must never be worded as
third-party certification or regulatory compliance claims unless a future
owner-approved program defines such a claim with evidence and legal review.

The controlling source for dispositions is
`specs/STANDARDS_COMPLIANCE_SPEC.md`. This decision defines the product
mechanism: a resolved `StandardsRegistry`, standards-basis references attached
to affected objects, project compliance metadata, check findings, deviations,
and audit records inside the unified `DesignModel`. Registry records and
project posture are source state. Check results and reports are evidence about
a known `model_revision`, not certifications.

## User-Visible Behavior

Users should be able to inspect a project's standards and compliance posture.

Expected behavior:
- project settings expose selected standards/process bases where supported
- library and footprint views show declared basis, validation state,
  deviations, and unknown-basis warnings
- schematic sheets expose symbol-style, reference-designator, title-block, and
  sheet-template metadata where supported
- rules/checks projection shows which standards or process assumptions informed
  each check
- import review identifies source-format facts separately from Datum-inferred
  or user-selected standards basis
- reports use precise language such as "validated against selected rule basis"
  rather than "certified" or "compliant product"
- data-egress-sensitive tools show the active export-control or project-egress
  policy before using external services

## Manual Workflow Requirements

Datum must let users manage standards posture manually.

Manual workflows must include:
- select or record project standards/process basis
- mark standards as not applicable, unknown, reference-only, or deferred where
  appropriate
- choose symbol-style and designator profiles without losing authored strings
- enter title-block and project compliance metadata
- declare IPC class, intended environment, export-control posture, and
  material/compliance declaration posture as metadata where applicable
- inspect library/footprint standards basis and deviations
- run standards-aware checks and export reports
- document waivers, deviations, and review rationale

All standards, posture, registry, basis, waiver, and deviation mutations reduce
to typed `Operation`s that converge on the single `commit()`+journal; there is
no private standards write path. Standards correction and imported-basis
inference edits are proposal-first per the import/standards-correction
invariant, so an inferred basis or corrected posture is surfaced as a
`Proposal` for human acceptance before it becomes a `Transaction`.

## Optional AI And Tooling Behavior

AI and tools may assist standards work only as evidence-preserving helpers.

Allowed behavior:
- explain which selected standards basis applies to an object
- identify missing metadata or unknown-basis data
- propose standards-aware rules/checks
- summarize check results and unresolved assumptions
- suggest library metadata extraction or footprint validation steps
- prepare reports for human review

Required guardrails:
- AI may not claim certification
- AI may not infer regulated compliance status from incomplete metadata
- AI may not silently select project standards basis
- external lookups must respect data-egress policy
- generated standards conclusions must cite the modeled basis and uncertainty

## Core Primitives

### StandardsRegistry

Project-visible registry of researched or product-relevant standard families
and their disposition.

Dispositions:
- `SupportImplemented`
- `Planned`
- `ReferenceOnly`
- `DeferredWithPrerequisite`
- `OutOfScope`

The standards-disposition enum is a distinct namespace from the reserved
derived relationship status `{Implemented, PendingImplementation,
UnresolvedMismatch}`. Relationship status is resolver-computed, recomputed on
load, and never persisted as source authority; standards disposition is
project-visible source state recorded on a registry entry. The two must not
share an enum type, which is why this token is `SupportImplemented` rather than
the reserved `Implemented`.

The registry prevents silent standards gaps after research has been delivered.

Implementation schema concept:

```text
StandardsRegistryEntry
  id: ObjectId
  standard_family
  revision?
  domain
  disposition: SupportImplemented | Planned | ReferenceOnly | DeferredWithPrerequisite | OutOfScope
  support_scope
  prerequisite?
  controlling_spec_refs[]
  claim_language_policy
  updated_by_transaction_id
  provenance
```

The registry covers the eight audited domains from the standards-compliance
spec: data exchange and interop, component modelling, schematic/drawing
conventions, industry-vertical compliance, materials/environmental, EMC/SI,
PLM/lifecycle, and process/quality. A researched standard may not exist only
in prose. If Datum knows about it, a registry entry states its disposition.

`ReferenceOnly` entries may inform vocabulary, labels, metadata, or help text,
but they do not produce pass/fail checks. `Planned` entries may be visible as
roadmap/posture but must not appear as implemented behavior. `Deferred` entries
must name their prerequisite so future work has a clear unblock condition.

### StandardsBasis

The selected standard family, revision, profile, or process basis used by a
library object, rule, check, project, manufacturing projection, or artifact.

Examples:
- IPC land-pattern basis
- IPC stencil/paste basis
- IPC clearance basis
- symbol-style profile
- designator profile
- title-block field basis
- IPC class metadata
- export format revision

Implementation schema concept:

```text
StandardsBasis
  id: ObjectId
  registry_entry_ref
  revision_or_profile?
  selected_by
  selection_scope
  basis_kind
  declared | inferred | user_selected | imported | unknown
  evidence_refs[]
  uncertainty?
  provenance
```

Basis records are referenced from `Part`, `Package`, `Symbol`, `Footprint`,
`Padstack`, `Rule`, `CheckRun`, `ManufacturingPlan`, `OutputJob`, and exported
artifact metadata. If the basis is inferred during import or audit, that
uncertainty remains visible until a user accepts or replaces it.

### ProjectCompliance

Project-level metadata describing intended compliance posture.

Initial metadata categories:
- IPC class and revision basis
- intended operating environment
- industry vertical tags where useful
- export-control/data-egress posture
- material/environmental declaration posture
- audit-log completeness posture
- future approval/sign-off overlay status

This is metadata and workflow substrate, not certification.

Implementation schema concept:

```text
ProjectCompliance
  id: ObjectId
  ipc_class_basis?
  intended_operating_environment?
  industry_vertical_tags[]
  export_control_posture?
  data_egress_policy_ref
  material_declaration_posture?
  audit_log_completeness_state
  signoff_overlay_state
  compliance_claims_allowed[]
  provenance
```

`ProjectCompliance` is selected and edited through normal operations. It does
not make the project compliant with a regulation. It gives checks, reports,
exports, and data-egress gates the metadata they need to behave consistently.

### PartQualification

Library part metadata for qualification, lifecycle, environmental,
supply-chain, and compliance declarations.

Examples:
- AEC-Q metadata
- lifecycle status
- RoHS/REACH/IPC-1752A declaration posture
- manufacturer qualification references
- source and review provenance

These fields live on `Part` and are exposed to BOM/check/report projections.
They are evidence and metadata. Unless a later spec promotes a reporting
engine, Datum does not generate legal RoHS, REACH, IPC-1752A, automotive,
medical, aerospace, or defense compliance declarations.

### DataEgressPolicy

Policy gate for tools that may send project, part, model, or compliance data
outside the local project.

The policy must apply to MCP, CLI, assistant, AI, and supply-chain/model lookup
tools.

Implementation schema concept:

```text
DataEgressPolicy
  id: ObjectId
  posture
  allowed_destinations[]
  blocked_data_classes[]
  prompt_required: bool
  audit_required: bool
  export_control_basis?
  provenance
```

Any external lookup, AI call, MCP action, or supply-chain/model metadata tool
that can transmit project data must consult this policy before execution and
record an audit event when required.

### ComplianceFinding

Standards or compliance metadata issue emitted through the same check system as
other findings.

Examples:
- unknown footprint basis
- missing material declaration posture
- export-control marking missing for a controlled project
- title-block metadata incomplete
- AEC-Q metadata missing for a selected project profile

`ComplianceFinding` is a `CheckFinding` category, not a separate diagnostic
system. It uses the same finding fingerprint, waiver, deviation, report, and
proposal mechanics as ERC/DRC findings.

### AuditRecord

Structured evidence of standards-facing decisions, changes, waivers,
deviations, exports, and approvals.

Audit records must support later process overlays, but early Datum should not
claim Part 11, ISO 9001, ISO 13485, or similar compliance until prerequisites
exist.

Implementation schema concept:

```text
AuditRecord
  id: ObjectId
  model_revision
  transaction_id?
  event_kind
  actor
  timestamp
  affected_object_ids[]
  basis_refs[]
  rationale?
  evidence_refs[]
  signature_state?
  provenance
```

The project journal is the durable transaction substrate. `AuditRecord`s are
derived from committed `TransactionRecord`s and are emitted only as part of
`commit()`; no path may write an `AuditRecord` while bypassing the single
`commit()`+journal. The `actor`, `timestamp`, and `signature_state` fields are
taken from the committing transaction's provenance, not captured independently.
`AuditRecord`s index standards-facing events for reports and future regulated
overlays; they do not replace the journal and they do not create compliance
claims by themselves.

## Claim Posture

Datum must use precise claim language:

- Allowed: "basis declared", "basis unknown", "validated against selected Datum
  rule basis", "finding waived", "documented deviation accepted", "metadata
  present", "artifact generated from model revision X".
- Not allowed without future owner/legal approval: "certified", "regulatory
  compliant", "IPC certified", "FDA compliant", "ISO compliant",
  "automotive-qualified product", or equivalent claims about the user's product
  or organization.
- `compliant` may only describe a modeled object relative to a selected Datum
  rule/standards basis and must preserve deviation state. It must not imply
  third-party certification.
- `ReferenceOnly`, `Planned`, `DeferredWithPrerequisite`, and `OutOfScope`
  registry entries must be visible in reports when relevant so users can see
  what Datum did not evaluate.

Reports must include model revision, relevant object revisions, standards
basis, rule/check versions, unresolved assumptions, waivers, deviations, and
unknown-basis data. Exported artifacts inherit this metadata from their output
job and live production projection.

## Standards And Compliance Impact

This decision makes standards a design input across the product.

Required impact:
- standards registry stays explicit once a standard has been researched
- library primitives store standards basis and provenance
- rules/checks can report standards-aware findings
- imported data can be audited against declared or inferred basis without being
  silently rewritten
- manufacturing artifacts record the model revision, rule basis, and output
  settings used to generate them
- project metadata supports regulated or vertical workflows as substrate

Pad, mask, and paste aperture policy is a standards-facing data surface, not a
renderer detail. IPC/stencil/check support must record whether apertures are
generated from a declared basis, manually authored, import-preserved, unknown,
or intentionally deviated. Zone-related manufacturing claims must also respect
`ZoneFill` state: unfilled, stale, unsupported, or import-preserved fills are
reported as such rather than implied to satisfy copper/manufacturing checks.

Required claim discipline:
- "supports metadata for" is not "complies with"
- "validated against selected Datum rule basis" is not "certified"
- "reference-only" standards must not drive pass/fail checks
- "planned" support must not appear as implemented behavior
- regulatory and process standards remain metadata/deferred unless the
  controlling spec promotes them

## Non-Goals

This decision does not:
- claim IPC, FDA, FCC, CISPR, ISO, automotive, medical, aerospace, defense, or
  environmental certification
- implement every researched standard
- require Datum to redistribute paywalled standards text or third-party
  libraries
- turn project metadata into legal compliance advice
- implement 21 CFR Part 11 signatures before audit/signature primitives exist
- implement supplier, PLM, or regulatory reporting integrations by default
- make external network services mandatory

## First Proof Slice

The first proof slice should make standards posture visible and checkable:

1. Add a project standards/compliance posture view with a small registry subset.
2. Record IPC footprint/process basis for one footprint and unknown-basis state
   for one imported footprint.
3. Run standards-aware checks that report basis, finding, uncertainty, and
   deviation state.
4. Export a report that avoids certification language and includes model
   revision, standards basis, and unresolved assumptions.
5. Enforce a data-egress policy prompt or gate for one external metadata lookup
   tool.

## Owner Questions

- What exact user-facing language is allowed for "compliant",
  "validated", "basis known", and "certified"?
- Which standards registry subset should be visible in the first product slice?
- Should project standards posture be required at project creation or optional
  until a check/report needs it?
- What data-egress policy defaults should apply to new projects?
- Which compliance metadata fields are mandatory versus advisory for v1?
- How much audit/sign-off infrastructure is needed before regulated-process
  overlays can be shown in the UI?
