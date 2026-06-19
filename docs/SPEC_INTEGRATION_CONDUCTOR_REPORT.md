# Spec Integration Conductor Report

Status: final conductor audit after controlling-spec integration
Date: 2026-06-19
Owned file: `docs/SPEC_INTEGRATION_CONDUCTOR_REPORT.md`

## Integration Verdict

Verdict: READY_FOR_SUBSTRATE_IMPLEMENTATION.

The product-mechanics, decision, contract, and controlling specification
documents now preserve the target mechanism spine: resolver-owned
`DesignModel`, stable object identity, `ComponentInstance`, split relationship
semantics, sparse variants, Import Map `import_key`, one journaled `commit()`,
PTY-real terminal behavior, proposal-first high-risk automation, ZoneFill
honesty, `CheckRun` / `CheckFinding`, artifact/output-job metadata, and
evidence-limited standards wording.

The controlling specs now have an explicit current-vs-target split and are
ready to drive the first implementation slices. They are not a claim that the
substrate already exists in code. Implementation must start with the engine
substrate and commit/journal slices before higher-level GUI, AI, checking,
artifact, or import-repair behavior is treated as product-complete.

## Files Audited

Primary source-of-truth audits:
- `docs/audits/scope-integration/DATUM_SCOPE_INTEGRATION_READINESS_AUDIT.md`
- `docs/audits/scope-integration/DATUM_PRODUCT_MECHANICS_IMPLEMENTATION_READINESS_AUDIT.md`
- `docs/contracts/AI_CLI_MCP_TOOL_SURFACE.md`

Contract docs audited:
- `docs/contracts/LIBRARY_AUTHORING_TOOL_CONTRACT.md`
- `docs/contracts/MANUFACTURING_OUTPUT_TOOL_CONTRACT.md`
- `docs/contracts/PCB_LAYOUT_TOOL_CONTRACT.md`
- `docs/contracts/RULES_CHECKS_TOOL_CONTRACT.md`
- `docs/contracts/SCHEMATIC_AUTHORING_TOOL_CONTRACT.md`

Decision docs audited:
- `docs/decisions/PRODUCT_MECHANICS_000_UNIFIED_DESIGN_MODEL.md`
- `docs/decisions/PRODUCT_MECHANICS_000B_VIEW_COMPOSITION_AND_LIVE_PRODUCTION.md`
- `docs/decisions/PRODUCT_MECHANICS_000C_UNIFIED_MODEL_FEASIBILITY.md`
- `docs/decisions/PRODUCT_MECHANICS_000D_STORAGE_AND_VERSIONING_MODEL.md`
- `docs/decisions/PRODUCT_MECHANICS_000E_MULTI_SHEET_MULTI_BOARD_MODEL.md`
- `docs/decisions/PRODUCT_MECHANICS_000F_LIVE_PRODUCTION_PROOF_MODEL.md`
- `docs/decisions/PRODUCT_MECHANICS_001_CANONICAL_EDIT_MODEL.md`
- `docs/decisions/PRODUCT_MECHANICS_002_MANUAL_EDITOR_BASELINE.md`
- `docs/decisions/PRODUCT_MECHANICS_003_SCHEMATIC_PCB_AUTHORITY.md`
- `docs/decisions/PRODUCT_MECHANICS_004_AI_TOOLING_CONTRACT.md`
- `docs/decisions/PRODUCT_MECHANICS_005_EMBEDDED_TERMINAL.md`
- `docs/decisions/PRODUCT_MECHANICS_006_ASSISTANT_SURFACE.md`
- `docs/decisions/PRODUCT_MECHANICS_007_PROJECT_WORKSPACE_MODEL.md`
- `docs/decisions/PRODUCT_MECHANICS_008_LIBRARY_COMPONENT_SYSTEM.md`
- `docs/decisions/PRODUCT_MECHANICS_009_RULES_CONSTRAINTS_CHECKS.md`
- `docs/decisions/PRODUCT_MECHANICS_010_INDUSTRY_STANDARDS_COMPLIANCE.md`
- `docs/decisions/PRODUCT_MECHANICS_011_IMPORT_INTEROP_ROLE.md`
- `docs/decisions/PRODUCT_MECHANICS_012_APPLICATION_QUALITY_BAR.md`

Controlling specs sampled for integration readiness:
- `specs/ENGINE_SPEC.md`
- `specs/MCP_API_SPEC.md`
- `specs/CHECKING_ARCHITECTURE_SPEC.md`
- `specs/STANDARDS_COMPLIANCE_SPEC.md`
- `specs/PROGRESS.md`

## Implementation-Ready Assessment

Worker decision docs: implementation-ready as target doctrine. Remaining
questions are mostly slice ordering, owner policy choices, proof fixtures, and
budget thresholds rather than unresolved architecture.

Shared contract docs: implementation-ready as target contracts with explicit
substrate prerequisites. They correctly call out missing current substrate
instead of pretending it already exists.

Controlling specs: ready for substrate implementation planning. `ENGINE_SPEC.md`
now front-loads the resolver, source-shard, identity/revision,
`ComponentInstance`, relationship, variant, Import Map, `OperationBatch`,
`commit()`/journal, ZoneFill, artifact, output-job, and checking substrate.
`MCP_API_SPEC.md` now separates target `datum-eda` / `datum.*` tool classes
from current compatibility methods. `CHECKING_ARCHITECTURE_SPEC.md` and
`STANDARDS_COMPLIANCE_SPEC.md` now define revision-keyed runs/findings,
profiles, waivers/deviations, standards evidence, proposal-first repair, and
pad/process-aperture observability. `PROGRESS.md` now separates historical
milestone evidence from active substrate readiness.

## Remaining Implementation Decisions

These are not scope-integration blockers, but they should be resolved during
the first implementation slice:

- Final Rust shapes for placeholder target types such as `DomainObject`,
  `SourceShardRef`, `RelationshipOverride`, provenance/status enums, and
  operation variants.
- Direct-commit policy boundaries for manual GUI edits versus proposal-required
  assistant, checker, import-repair, destructive, batch, and cross-domain
  edits.
- Artifact metadata storage home: journaled record, sidecar manifest, or both.
- Whether accepted electrical/physical divergence starts as relationship
  metadata plus check waivers/deviations or grows a standalone deviation
  primitive immediately.
- Alias deprecation timing after target `datum.*` methods exist.
- Migration of legacy standards registry rows that still say `Implemented`
  into explicit normative-support and evidence records where evidence is weak.
- Native private writers and non-generic mutation paths remain migration
  defects until they are routed through `OperationBatch -> commit()`.

## Required Next Implementation Slices

1. Engine substrate slice: introduce the resolver/source-shard authority model,
   stable object identity, `ComponentInstance`, Import Map `import_key`,
   revision computation, and typed `OperationBatch` vocabulary.
2. Commit/journal slice: replace private native write paths with the single
   durable `commit()` path, including transaction provenance, fsync commit
   point, atomic shard promotion, durable undo/redo cursors, and recovery mode.
3. Relationship/variant slice: implement `RelationshipKind`, derived
   relationship status, authored intent/deviation records, sparse variant
   overlays, and derived `NotApplicableForVariant`.
4. Tool-surface slice: expose `datum-eda` CLI and `datum.*` MCP families over
   shared context/query/check/proposal/apply/artifact/journal schemas, with
   current method aliases clearly marked as compatibility.
5. Checking/standards slice: implement revision-keyed `CheckRun` and
   `CheckFinding`, stable finding fingerprints, standards/process basis,
   waiver/deviation evidence, and proposal-first repairs.
6. ZoneFill/projection slice: separate authored `Zone.polygon` from derived
   `ZoneFill`; only `ZoneFill{Filled}` may contribute copper to projections or
   export, while unfilled/stale/unsupported zones emit hard findings.
7. Progress/parity slice: update `PROGRESS.md`, `SPEC_PARITY.md`, and drift
   gates only after the controlling specs and code slices are reconciled.

## Consistency Checks

`ProjectResolver`: PASS. The target docs and `ENGINE_SPEC.md` make
`ProjectResolver` the only assembler of the resolved `DesignModel`.

`ComponentInstance`: PASS as target spec, not yet code-complete. The docs
consistently forbid refdes/name/path joins and use `ComponentInstance` as the
electrical-to-physical join.

`commit()` and journal: PASS as target spec, not yet code-complete. All target
mutation surfaces route through one journaled mutation path. Current private
writers and per-method mutation paths remain documented migration defects.

`RelationshipKind` split: PASS. The target relationship model separates
authored `RelationshipKind`, derived status, authored intent/deviation, and
variant applicability. `Implemented`, `PendingImplementation`,
`UnresolvedMismatch`, and `NotApplicableForVariant` must not be persisted as
authored source states.

`ZoneFill`: PASS as target spec, not yet code-complete. Docs correctly separate
`Zone.polygon` from derived `ZoneFill` and require unfilled/stale/unsupported
zones to export no copper with a hard finding.

`datum-eda` CLI naming: PASS. Contracts and decision docs use `datum-eda` for
CLI examples and reserve `datum.*` for MCP tool families. Bare `eda` and bare
`datum` must remain legacy/noncanonical only.

Current-vs-target MCP split: PASS. `MCP_API_SPEC.md` now maps current daemon
methods into the product-mechanics target classes while preserving current
method truth as compatibility.

Standards claim discipline: PASS with wording risk. The standards docs frame
Datum as a compliance/evidence substrate, not a certifying authority. Maintain
that discipline when using words like `Implemented`; it must mean Datum support
and evidence, not external certification of a product, process, organization,
or part.

`PROGRESS` current-vs-target split: PASS. `PROGRESS.md` now keeps historical
milestone evidence separate from active substrate readiness and does not mark
target substrate rows complete without implementation anchors.

## Reusable Audit Checklist

Use this checklist if more worker edits land before spec integration starts:

- No doc introduces a second source authority besides the resolved
  `DesignModel` assembled by `ProjectResolver`.
- No doc treats a sheet, board, pool file, workspace, terminal, assistant,
  generated artifact, imported file, or cache as design authority.
- Every mutation surface routes through typed operations and the single
  `commit()`/journal path.
- Every high-risk, cross-domain, import-repair, standards-correction, batch, or
  AI-originated edit is proposal-first unless an explicit direct-commit policy
  says otherwise.
- Relationship language keeps `RelationshipKind`, derived status, authored
  intent/deviation, and variant applicability separate.
- Cross-domain joins use stable IDs and `ComponentInstance`, never refdes,
  names, paths, positions, sheet IDs, board IDs, or imported source paths.
- Import identity uses Import Map `import_key`; `source_hash` may only be
  evidence, provenance, or a described legacy defect.
- Variant overlays are sparse, keyed by stable object identity, and do not
  write base state when switching active variant.
- Zone fill language never equates an authored zone boundary with production
  copper.
- CLI examples use `datum-eda`; MCP tool names use `datum.*`.
- Standards wording never claims Datum certifies products, processes, parts,
  projects, organizations, or regulatory compliance.
- `PROGRESS.md` marks only landed implementation as current and separately
  tracks target contracts.
