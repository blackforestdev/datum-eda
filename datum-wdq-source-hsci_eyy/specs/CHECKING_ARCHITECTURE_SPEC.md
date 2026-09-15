# Checking Architecture Specification

## 1. Purpose

Defines Datum's shared checking substrate for electrical, physical,
standards-aware, and manufacturing checks.

ERC and DRC remain useful user-facing concepts, but they are no longer separate
result contracts. All checking entry points produce a `CheckRun` containing
revision-keyed `CheckFinding` records.

---

## 2. Canonical Product Surface

The canonical CLI/MCP surface is:

- `datum-eda check run`
- `datum-eda check show`

MCP tools should expose equivalent verbs using the same input and output
contracts. Legacy `run_erc` and `run_drc` remain compatibility aliases under
`datum-eda check run`; they select an ERC-only or DRC-only `CheckProfile` and
must return payloads that can be losslessly mapped into `CheckRun`.

`datum-eda check show <check_run_id>` returns the persisted `CheckRun`, its
`CheckFinding` records, explanations, evidence, waiver/deviation state, and
proposal references. Repair generation is proposal-owned: a check may attach a
candidate repair reference, but creation, validation, and application of the
repair use `datum-eda proposal ...`.

`get_violations` is a compatibility view over `CheckFinding` records.
`explain_violation` is a compatibility alias for `datum-eda check show` scoped
to one finding and must accept or infer the checking domain so results are
interpreted against the correct graph, geometry, revision, and location type.

---

## 3. Inputs And Resolution

Every check run must resolve project state through `ProjectResolver` rather
than ad hoc file reads.

Minimum run inputs:

```rust
pub struct CheckRunRequest {
    pub project_ref: ProjectRef,
    pub model_revision: ModelRevision,
    pub profile: CheckProfileRef,
    pub domains: Vec<CheckDomain>,
    pub rule_filter: Option<RuleFilter>,
    pub include_waived: bool,
    pub include_proposals: bool,
}
```

- `project_ref` is resolved by `ProjectResolver`
- `model_revision` pins the schematic, board, library, rule, and imported
  artifact state being checked
- runs against moving project state must first resolve an explicit
  `model_revision`
- findings, fingerprints, waivers, deviations, and repair proposals are keyed
  to that revision
- imported objects must carry import metadata, including `import_key` when an
  Import Map was used

---

## 4. Domains

```rust
pub enum CheckDomain {
    Erc,
    Drc,
    Standards,
    Manufacturing,
    Relationships,
}
```

- `Erc` consumes schematic connectivity and electrical semantics
- `Drc` consumes board geometry, board connectivity, and physical design rules
- `Standards` consumes declared standards bases, revision metadata, library
  assertions, and standards-owned observables
- `Manufacturing` consumes board geometry, stackup, process geometry, fab rules,
  assembly rules, and manufacturing artifact metadata
- `Relationships` consumes cross-model provenance and intent links between
  schematic, board, library, imports, generated artifacts, and external models

Relationship checks are first-class check-domain members, not hidden ERC/DRC
side effects. Examples include:

- schematic intent not realized on board
- board object exists with no schematic origin
- constraint propagation mismatch
- footprint/package/library assertion drift
- imported object identity mapped through an Import Map `import_key`

Profiles decide whether relationship findings participate in pass/fail for a
given workflow.

---

## 5. Check Profiles

```rust
pub struct CheckProfile {
    pub id: String,
    pub label: String,
    pub domains: Vec<CheckDomain>,
    pub rules: Vec<CheckRuleSelector>,
    pub severity_overrides: Vec<SeverityOverride>,
    pub pass_fail_policy: PassFailPolicy,
    pub proposal_policy: ProposalPolicy,
}
```

Required built-in profiles:

- `erc`: electrical schematic checks only
- `drc`: board physical checks only
- `standards`: standards-aware metadata and observable checks
- `manufacturing`: fabrication and assembly readiness checks
- `release`: ERC, DRC, standards, manufacturing, and relationship checks using
  release pass/fail policy

Profiles are authored configuration. They must serialize deterministically and
must not silently change historical run interpretation.

---

## 6. CheckRun And CheckFinding

```rust
pub struct CheckRun {
    pub uuid: Uuid,
    pub project_ref: ProjectRef,
    pub model_revision: ModelRevision,
    pub profile_id: String,
    pub started_at: Timestamp,
    pub completed_at: Option<Timestamp>,
    pub status: CheckRunStatus,
    pub summary: CheckSummary,
    pub profile_basis: CheckRunProfileBasis,
    pub coverage: Vec<CheckRunCoverageEntry>,
    pub findings: Vec<CheckFinding>,
    pub artifact_metadata: Vec<CheckArtifactMetadata>,
}

pub struct CheckRunProfileBasis {
    pub profile_id: String,
    pub domains: Vec<CheckDomain>,
    pub description: String,
    pub standards_basis: Option<String>,
}

pub struct CheckRunCoverageEntry {
    pub domain: CheckDomain,
    pub rule_id: String,
    pub status: CheckRunCoverageStatus,
    pub target_scope: String,
    pub basis_id: Option<String>,
    pub rule_revision: Option<String>,
    pub standards_basis: Option<String>,
}

pub struct CheckFinding {
    pub uuid: Uuid,
    pub fingerprint: CheckFingerprint,
    pub domain: CheckDomain,
    pub rule_id: String,
    pub severity: CheckSeverity,
    pub status: CheckFindingStatus,
    pub primary_target: CheckTarget,
    pub related_targets: Vec<CheckTarget>,
    pub message: String,
    pub evidence: Vec<CheckEvidence>,
    pub proposal_refs: Vec<ProposalRef>,
    pub proposal_links: Vec<ProposalActionLink>,
    pub waiver_refs: Vec<Uuid>,
    pub deviation_refs: Vec<Uuid>,
}
```

`proposal_refs` remains the stable compatibility identity list. `proposal_links`
is the actionable view consumed by GUI, CLI, MCP, and agents: it carries proposal
status/source/rationale, a validation snapshot, and canonical proposal command
templates so repair review/apply UX routes through the proposal gateway instead
of private GUI mutation paths.

Finding order must be stable and deterministic.

Every persisted `CheckRun` must carry coverage metadata. The coverage list is
the contract that makes a clean or filtered result honest: `evaluated` means the
rule family participated in the run, `filtered_by_profile` means the selected
profile intentionally excluded it, `not_applicable` means the rule family had no
targets in the resolved model, and `not_implemented` means Datum knows about the
rule family but does not yet evaluate it. GUI and agent contexts must expose
this metadata rather than inferring scope from which findings happened to
appear.

`CheckFingerprint` must be stable across waiver-only and disposition-only edits.
It is derived from:

- domain
- rule id and rule revision
- normalized target identity
- normalized relevant observed values
- Import Map `import_key` where imported identity participates

The enclosing `CheckRun.model_revision` provides revision scope. Waivers and
deviations record the revision they were reviewed against separately, so adding
a waiver does not invalidate the fingerprint it targets. Fingerprints are used
for stable comparison across runs. They are not object UUIDs and must not be
treated as permanent identity when rule revision, normalized target identity, or
observed values change.

---

## 7. Severity, Status, And Summary

```rust
pub enum CheckSeverity {
    Error,
    Warning,
    Info,
}

pub enum CheckFindingStatus {
    Active,
    Waived,
    AcceptedDeviation,
    Resolved,
    Stale,
}

pub struct CheckSummary {
    pub errors: u32,
    pub warnings: u32,
    pub infos: u32,
    pub waived: u32,
    pub accepted_deviations: u32,
    pub proposals: u32,
}
```

Reports must expose:

- `passed`
- `summary`
- `findings`
- stable finding ordering for deterministic output
- waiver and deviation application state
- proposal references when repair is available

Waived and accepted-deviation findings remain visible. They suppress failure
state only according to the active profile's pass/fail policy.

---

## 8. Waivers And Deviations

```rust
pub struct CheckWaiver {
    pub uuid: Uuid,
    pub domain: CheckDomain,
    pub target: WaiverTarget,
    pub rationale: String,
    pub created_by: Option<String>,
    pub model_revision_scope: RevisionScope,
}

pub struct CheckDeviation {
    pub uuid: Uuid,
    pub domain: CheckDomain,
    pub rule_id: String,
    pub target: CheckTarget,
    pub expected: CheckObservedValue,
    pub actual: CheckObservedValue,
    pub rationale: String,
    pub approval_status: DeviationApprovalStatus,
    pub scope: DeviationScope,
    pub model_revision_scope: RevisionScope,
}

pub enum WaiverTarget {
    Object(Uuid),
    RuleObject {
        rule: String,
        object: Uuid,
    },
    RuleObjects {
        rule: String,
        objects: Vec<Uuid>,
    },
    Fingerprint(CheckFingerprint),
}
```

- Waivers and deviations are authored data
- Waivers suppress failure state; deviations record an accepted standards,
  manufacturing, or design-rule delta
- Neither waivers nor deviations delete findings
- Application must be explicit in reports
- `RuleObjects.objects` must be stored in deterministic UUID order
- matching is identity-based, not name-based
- imported object renames do not invalidate matching if UUID or Import Map
  `import_key` identity is stable
- revision scope must be explicit so stale waivers/deviations can be reported

M2 minimum:

- waivers may target exact object UUIDs, exact rule/object tuples, or exact
  fingerprints
- waived findings remain visible but do not fail the check
- waivers must serialize deterministically

Current implementation:

- `WaiverTarget::Fingerprint` is implemented in the shared waiver model
- `project query <root> check-run` applies fingerprint-scoped native waivers to
  normalized `CheckFinding` records, keeping findings visible with
  `status=waived` and `waiver_refs`
- `project waive-finding <root> --fingerprint <sha256:...> --rationale <text>`
  authors a fingerprint-scoped schematic waiver through `OperationBatch`,
  appends the transaction journal, updates the schematic root source shard, and
  supports journal undo/redo
- `project accept-deviation <root> --fingerprint <sha256:...> --rationale <text>`
  authors a fingerprint-scoped accepted deviation through `OperationBatch`,
  appends the transaction journal, updates the schematic root source shard, and
  supports journal undo/redo
- accepted deviations keep findings visible with `status=accepted_deviation`,
  populate `deviation_refs`, and suppress active failure counts without being
  counted as waivers

---

## 9. Proposal-First Repairs

Check execution is diagnostic. It must not mutate design, library, import, or
artifact state.

When a finding is mechanically actionable, the check system may attach a repair
proposal. Applying the proposal is a separate user-authorized transaction that
records provenance, source finding, model revision, and resulting model
revision.

Current implementation evidence:

- `project generate-standards-repair-proposals` creates draft
  `ProposalSource::Check` records from persisted `CheckFinding` fingerprints.
- Process-aperture repairs group pad mask/paste findings by pad and emit one
  `SetBoardPad` operation per affected pad.
- Dimension-rule repairs preserve netclass rules as the authority and emit
  direct geometry proposals: `track_width_below_min` produces `SetBoardTrack`,
  while `via_hole_out_of_range` and `via_annular_below_min` are grouped per via
  into one `SetBoardVia` proposal when both findings affect the same via.
- Clearance, silkscreen, connectivity, and route-topology findings remain
  diagnostic-only until their repair policies have explicit geometry/routing
  proposal mechanics.

This rule applies to all domains, including standards-aware footprint repair,
manufacturing apertures, ERC no-connect insertion, relationship repair, and
artifact regeneration.

---

## 10. ZoneFill Honesty

Zone-fill and derived-copper state must be reported honestly.

- Checks that require filled geometry must declare that dependency
- A run against stale, missing, or approximate zone fills must produce an
  explicit finding or run status rather than silently treating approximated
  geometry as authoritative
- Repair proposals may request zone refill/regeneration, but check execution
  must not perform hidden fills
- Findings affected by zone-fill state must include evidence showing whether
  filled, stale, or approximated geometry was used

---

## 11. Standards And Manufacturing Observables

DRC, standards, and manufacturing profiles must include standards/process
geometry findings when a board declares a process basis. Pad copper,
solder-mask, and paste/stencil apertures are separate manufacturing
observables; a pad must not pass cleanly merely because mask or paste inherited
copper geometry from an imported source.

Minimum pad process-aperture finding codes:

- `pad_mask_expansion_missing`
- `pad_mask_expansion_below_rule`
- `pad_paste_reduction_missing`
- `pad_paste_reduction_below_rule`
- `pad_process_aperture_inherited_from_copper`
- `pad_process_aperture_inconsistent_with_peer_footprint`

The finding is detection only. Any geometry repair must be represented as an
explicit proposal or transaction accepted by the user; import and checking must
not silently mutate source geometry toward an inferred IPC result.

Current proposal-backed repair coverage is narrower than the full standards
target: pad process-aperture findings can produce `SetBoardPad` proposals,
track-width findings can produce `SetBoardTrack` proposals, and via
hole/annular findings can produce `SetBoardVia` proposals. Ambiguous repairs
that require moving silkscreen, rerouting copper, relaxing rules, or choosing
between two clearance offenders remain explicit findings only.
