# Product Mechanics Decision 009: Rules, Constraints, And Checks

Status: draft for owner review; implementation mechanisms woven 2026-06-18.
Date: 2026-06-18

Driven by:
- `docs/DATUM_PRODUCT_MECHANICS.md`
- `docs/audits/scope-integration/DATUM_PRODUCT_MECHANICS_RESEARCH_SYNTHESIS.md`
- `docs/audits/scope-integration/DATUM_PRODUCT_MECHANICS_REVIEW_AGENDA.md`
- `docs/IPC_FOOTPRINT_SYSTEM.md`
- `specs/STANDARDS_COMPLIANCE_SPEC.md`
- `docs/STANDARDS_COMPLIANCE_INTEGRATION_GUIDANCE.md`
- `research/ipc-compliance/IPC_COMPLIANCE_RESEARCH.md`

## Decision Scope

Define Datum's product model for constraints, rules, deterministic checks,
findings, waivers, deviations, and standards-aware DRC/ERC behavior.

This decision covers the product mechanics, not the full rule expression
language or every individual ERC/DRC rule.

## Product Intent

Rules and checks are core product mechanics, not late-stage export filters.

Datum should use rules and constraints to guide design authoring, explain
manufacturability risk, support standards-aware review, inform AI/tooling
proposals, and create auditable sign-off evidence.

Users should be able to understand:
- what intent was authored
- what rule or standard basis applies
- what check found a problem
- what object is affected
- whether the result is waived, accepted as a deviation, or unresolved
- what proposal could fix it

## Decision

Datum shall model rules, constraints, and checks as first-class objects in the
canonical `DesignModel`.

Constraints capture authored design intent. Rules express executable or
declarative validation policy. Checks run deterministically against the design
model and produce structured findings. Findings may be resolved by edits,
waivers, documented deviations, or changes to the governing rule basis.

The rules/check subsystem is a projection over the same `DesignModel` authority
defined in 000 and uses the edit model from 001. Rule edits, waiver creation,
deviation creation, and accepted fixes are `OperationBatch`es committed through
`commit()` and recorded in the journal. Check execution itself does not mutate
design truth unless the user accepts a proposal produced from a finding.

Standards-aware DRC must include process-geometry checks for pad, solder-mask,
solder-paste/stencil, courtyard, drill, annular-ring, and clearance policy
where Datum has enough basis to evaluate them.

## User-Visible Behavior

Users should see rules and check state continuously, not only during final
export.

Expected behavior:
- rules/checks projection lists project rules, library rules, standards basis,
  check runs, findings, waivers, and deviations
- inspectors show relevant rules and findings for the selected object
- canvas overlays highlight check locations and affected geometry
- live feedback warns before obvious violations where inexpensive
- full check runs provide deterministic reports with severity, category,
  location, object IDs, expected values, actual values, and explanation
- users can create, edit, suppress, waive, or document deviations manually
- waived and deviated findings remain visible and auditable
- check output can be exported as a report or machine-readable artifact

## Manual Workflow Requirements

Datum must support rule and check workflows without AI.

Manual workflows must include:
- create and edit project constraints such as net class, clearance, width,
  differential-pair, length, impedance, layer, via, keepout, and manufacturing
  constraints
- select or declare standards/process basis for supported checks
- run ERC, DRC, library checks, import audits, standards checks, and
  manufacturing-output checks
- inspect each finding from a report, list, canvas overlay, or selected object
- change design geometry or metadata to resolve findings
- create a waiver with rationale, scope, actor, and timestamp
- create a documented deviation when the design intentionally differs from a
  standards or process basis
- re-run checks and compare result deltas

## Optional AI And Tooling Behavior

AI and automation may accelerate checking and repair, but they must not become
hidden authorities.

Allowed behavior:
- run relevant check sets
- explain findings in design context
- group related findings by root cause
- propose bounded repair operations
- propose rule changes when the authored constraint appears inconsistent
- generate reports for review

Required guardrails:
- tool-generated repairs are proposals by default
- AI may not silently waive findings or create deviations
- AI may not lower severity or mutate rules without explicit user approval
- external tools must preserve stable object IDs, check IDs, and provenance
- proposed fixes must list affected findings and checks expected to pass after
  application

## Core Primitives

### Constraint

Authored design intent or requirement.

Examples:
- min clearance
- track width
- differential-pair coupling
- length match
- impedance target
- voltage/current requirement
- keepout
- mask expansion
- paste reduction
- fabrication capability
- assembly or stencil process choice

Constraints may be project-wide, rule-scope-specific, net-class-specific,
object-specific, library-specific, or manufacturing-plan-specific.

Implementation schema concept:

```text
Constraint
  id: ObjectId
  object_revision: u64
  constraint_kind
  scope: RuleScope
  value
  units?
  basis_refs[]
  priority
  provenance
```

### Rule

Executable or declarative validation policy derived from constraints,
standards basis, project settings, library data, or manufacturing process
settings.

Rules must have stable identity, scope, severity defaults, source/provenance,
and enough explanation for a user to understand why the rule exists.

Implementation schema concept:

```text
Rule
  id: ObjectId
  object_revision: u64
  rule_code
  rule_version
  scope: RuleScope
  inputs[]
  severity_default
  check_engine
  standards_basis_refs[]
  source_constraint_refs[]
  explanation
  provenance
```

`rule_code` and `rule_version` identify executable behavior. `id` identifies
the configured rule object in this project. Updating a rule version or changing
scope is auditable because the changed object revision becomes part of the
next `CheckRun` input key.

### RuleScope

Typed selector for where a rule applies.

Scopes may target project, board, sheet, net, net class, component, footprint,
padstack, layer, region, library object, manufacturing projection, imported
object set, or explicit object IDs.

Rule scopes are resolved to object IDs before execution. A scope expression may
be human-readable, but the emitted finding must record the actual resolved
objects so rename, move, or sheet/board reorganization does not change finding
identity accidentally.

### CheckRun

A deterministic execution of one or more rule sets against a model revision.

Required fields:
- check-run ID
- model revision
- rule set and versions
- standards/process basis used
- actor/tool provenance
- timestamp
- summary counts
- findings

Implementation schema concept:

```text
CheckRun
  id: ObjectId
  model_revision
  active_variant_ref?
  variant_revision?
  input_object_revisions{}
  check_profile_id?
  rule_refs[]
  rule_engine_versions{}
  standards_basis_refs[]
  manufacturing_plan_ref?
  output_job_ref?
  actor_provenance
  timestamp
  summary_counts
  finding_refs[]
```

A `CheckRun` is reproducible evidence for the model revision, active-variant
reference, and rule versions it names. The active variant matters because
findings are derived state and variant-sensitive: composing the active sparse
variant overlay changes the fitted/unfitted population, and an unfitted
`ComponentInstance` changes its nets, courtyard, and connectivity findings.
The same `model_revision` can therefore yield different findings per active
variant, so `active_variant_ref`/`variant_revision` are part of the input key
and the variant population is derived input keyed by `model_revision`.
Re-running after any relevant object revision or active-variant change creates
a new run; old findings remain historical evidence and may be marked stale by
comparison, but they are not rewritten.

### CheckFinding

Structured diagnostic produced by a check run.

Required fields:
- stable finding identity or deterministic fingerprint
- severity
- category
- affected object IDs
- projection/location
- observable
- expected value
- actual value
- tolerance where applicable
- explanation
- standards/process reference where applicable
- suggested next action where known
- waiver/deviation/resolution state

Implementation schema concept:

```text
CheckFinding
  id: ObjectId?
  fingerprint
  check_run_id
  rule_ref
  severity
  category
  affected_object_ids[]
  related_object_ids[]
  projection_location?
  observable
  expected_value?
  actual_value?
  tolerance?
  basis_refs[]
  explanation
  source_provenance_refs[]
  repair_proposal_templates[]
  state: Unresolved | Resolved | Waived | Deviated | Stale | Superseded
  waiver_ref?
  deviation_ref?
  resolved_by_transaction_id?
```

Concrete implementation note: Datum's current persisted `CheckFinding` schema
realizes `affected_object_ids[]` as a typed `primary_target: CheckTarget` plus
`related_targets: CheckTarget[]`. `primary_target` is the stable affected object
or scope that owns the finding; `related_targets` carries additional affected
objects when the finding spans multiple objects. This preserves the
affected-object invariant while allowing non-object targets such as artifacts,
zone-fill evidence, imported objects, or net scopes to remain typed instead of
being forced into a UUID-only array.

`fingerprint` is deterministic from rule code/version, normalized affected
object IDs, observable, basis, and relevant geometry/value signature. It lets
Datum correlate findings across runs even when each run stores a fresh finding
object. Cross-domain findings that span schematic and board — pin-pad-map
checks, ERC, and symbol-vs-footprint consistency — join their electrical and
physical objects through the `ComponentInstance` surrogate and stable
`ObjectId`s, never reference designators, names, or positions, so
`affected_object_ids` and the fingerprint stay stable across rename, move, and
reorganization. If the issue refers to imported geometry, the finding records
both the preserved source provenance and the Datum rule basis used for
comparison; imported-object identity is bound through the Import Map
`import_key` (never `source_hash`) so finding fingerprints and
`affected_object_ids` remain stable across re-import and reload.

### Waiver

User-approved suppression of a finding or rule application.

Waivers must record scope, rationale, actor, timestamp, expiration or review
policy where applicable, and the exact finding/rule context they cover.

Implementation schema concept:

```text
Waiver
  id: ObjectId
  object_revision: u64
  scope: RuleScope | FindingFingerprint
  covered_rule_refs[]
  covered_finding_fingerprints[]
  rationale
  actor
  timestamp
  expires_at?
  review_policy?
  acceptance_transaction_id
  provenance
```

A waiver never changes design intent. It changes reporting state for the
covered finding/rule context and remains visible in reports.

### Deviation

Documented intentional difference from a selected constraint, standard,
library basis, or process policy.

Deviations differ from waivers: a deviation records accepted design intent; a
waiver suppresses a finding without necessarily changing the intent.

Implementation schema concept:

```text
Deviation
  id: ObjectId
  object_revision: u64
  target_object_ids[]
  deviated_basis_refs[]
  observable
  expected_value?
  accepted_value
  rationale
  approval_state
  actor
  timestamp
  scope
  acceptance_transaction_id
  provenance
```

Creating a deviation is a proposal-required action under 001 because it accepts
intent that differs from a selected rule, standard, library basis, or process
policy. A deviation may resolve a finding as `Deviated`; it does not make the
underlying geometry "compliant".

### CheckProfile

Named set of rule/check selections and severities.

Examples:
- quick edit feedback
- full project sign-off
- import audit
- library release
- manufacturing-output validation
- standards-focused review

### RepairProposal

Reviewable fix produced from one or more findings.

Implementation schema concept:

```text
RepairProposal
  proposal_id
  source_check_run_id
  source_finding_fingerprints[]
  operation_batch
  rationale
  expected_resolved_findings[]
  expected_new_or_remaining_findings[]
  risks[]
  provenance
```

Examples include `SetPadProcessAperture`, `ApplyFootprintProcessPolicy`,
`SetRule`, `AssignPinPadMap`, `CreateWaiver`, and `CreateDeviation`. Repairs
that span schematic and board — including `AssignPinPadMap` and other
pin-pad-map, ERC, or symbol/footprint corrections — target the affected
objects through their `ComponentInstance` join and stable `ObjectId`s, never
reference designators or names. Check-generated fixes are proposals by
default. Accepting one creates a normal journaled transaction; rejecting or
deferring it preserves the finding and the proposal disposition.

## Check Execution Model

Check execution follows one deterministic path:

1. Resolve the project through `ProjectResolver` into one `DesignModel`.
2. Compose the active sparse variant overlay into the resolved model before
   any rule execution, producing zero base writes. The derived
   fitted/unfitted population and `NotApplicableForVariant` status are inputs
   to checking, keyed by `model_revision`; without this step the reproducible-
   evidence claim is false, because the same revision yields different findings
   per active variant.
3. Select a `CheckProfile` or explicit rule set.
4. Resolve all `RuleScope`s to stable object IDs and capture input object
   revisions, active-variant reference, standards basis, manufacturing plan,
   output job, and engine versions.
5. Execute rule engines against authored source state and eligible derived
   projections. Derived inputs such as connectivity, `ZoneFill`, and live
   manufacturing projections must carry current/stale/unsupported state keyed
   by `model_revision`.
6. Emit structured `CheckFinding`s with deterministic fingerprints.
7. Correlate findings against existing waivers and deviations without hiding
   them.
8. Optionally emit `RepairProposal`s. No geometry, metadata, waiver, or
   deviation is committed by the check run itself.

Zone checks must be honest about fill state. A zone with `ZoneFill.state =
Unfilled`, `Stale`, or `Unsupported` may not pass copper/manufacturing checks
as if its boundary were filled copper. The check result must report missing,
stale, or unsupported derived fill rather than silently using a solid boundary
or source-tool assumption.

## Standards-Aware DRC

Datum must support standards-aware DRC as a normal check path.

Minimum process-geometry categories:
- copper pad geometry
- drill and annular ring
- solder-mask expansion or clearance
- solder-paste/stencil aperture reduction or split policy
- courtyard and component spacing
- package density-level observables
- IPC/process naming basis where applicable
- generic clearance basis such as IPC-2221 where project rules select it

Checks must distinguish:
- declared-basis validation
- inferred-basis validation
- unknown-basis audit
- imported-source geometry preserved but suspect
- compliant with documented deviation
- non-compliant

Example finding:

```text
category: process_geometry.paste
observable: paste_aperture_delta
expected_value: copper pad minus selected stencil reduction
actual_value: copper pad copied directly to paste aperture
basis: declared IPC/stencil policy or user-selected project process
state: unresolved
```

Pad/mask/paste policy is mandatory in this category. A check compares authored
or import-preserved copper, mask, and paste apertures against the selected
padstack, footprint, IPC/stencil, or manufacturing-plan basis. If the source
geometry is preserved from import, the finding says so, identifies the imported
object through the Import Map `import_key` (never `source_hash`) so the finding
stays stable across reload, and any correction is a proposal that records
repair provenance separately from import provenance.

## Standards And Compliance Impact

Rules and checks are how standards influence product behavior.

Datum must not claim that a clean check report certifies a project, footprint,
library, product, process, or regulatory posture. A check report means Datum
evaluated modeled data against selected rules and recorded the result.

Standards-facing checks must:
- preserve standards family and revision basis where known
- report unknown basis explicitly
- keep deviations and waivers auditable
- avoid silently converting imported geometry to Datum-preferred geometry
- avoid hard-coding certification claims into severity labels
- make project metadata, library metadata, and manufacturing-process settings
  visible in check output

## Non-Goals

This decision does not:
- define every ERC or DRC rule
- require real-time checking for every rule
- require solver-grade SI/PI or thermal simulation
- certify IPC, FCC, CISPR, ISO, FDA, medical, automotive, aerospace, or defense
  compliance
- allow imported projects to pass cleanly merely because the source tool
  accepted them
- allow AI to waive or fix findings invisibly

## First Proof Slice

The first proof slice should connect rules, standards basis, and visible
findings:

1. Add a project/library rule basis for pad, mask, paste, and courtyard policy.
2. Run a DRC check over one footprint or imported board segment.
3. Emit structured findings for a mask or paste aperture mismatch.
4. Show findings in a report and on the affected pad geometry.
5. Offer a reviewable correction proposal that changes the process aperture and
   records provenance.
6. Support a manual documented deviation instead of the correction.

## Owner Questions

- Which check profiles should exist first: edit-time, full DRC, import audit,
  library release, or manufacturing sign-off?
- What severities should standards-aware process-geometry findings use by
  default?
- Should unknown-basis imported footprints fail checks, warn, or be grouped as
  audit findings?
- What waiver/deviation approval metadata is required for early product use?
- Which IPC/process categories are mandatory for the first DRC proof slice?
- How much rule editing belongs in a GUI before CLI/MCP-only rule authoring is
  acceptable?
