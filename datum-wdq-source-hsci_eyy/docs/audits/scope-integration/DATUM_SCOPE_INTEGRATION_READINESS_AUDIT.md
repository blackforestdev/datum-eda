# Datum Scope Integration Readiness Audit

Status: final conductor synthesis
Date: 2026-06-19
Verdict: READY_FOR_SPEC_INTEGRATION

## Scope Audited

Audited source set:
- `docs/DATUM_PRODUCT_MECHANICS.md`
- `docs/audits/scope-integration/DATUM_PRODUCT_MECHANICS_DOCUMENTATION_GOALS.md`
- `docs/audits/scope-integration/DATUM_PRODUCT_MECHANICS_IMPLEMENTATION_READINESS_AUDIT.md`
- `docs/audits/scope-integration/DATUM_PRODUCT_MECHANICS_RESEARCH_SYNTHESIS.md`
- `docs/audits/scope-integration/DATUM_PRODUCT_MECHANICS_REVIEW_AGENDA.md`
- `docs/decisions/PRODUCT_MECHANICS_*.md`
- `docs/contracts/*.md`
- `CLAUDE.md`
- `specs/ENGINE_SPEC.md`
- `specs/MCP_API_SPEC.md`
- `specs/CHECKING_ARCHITECTURE_SPEC.md`
- `specs/STANDARDS_COMPLIANCE_SPEC.md`
- `specs/PROGRESS.md`

Worker result status: no separately readable worker artifacts were present in
`.agents` at audit time. This report therefore synthesizes the in-repo worker
outputs already materialized in the product-mechanics, decision, and contract
documents, especially
`docs/audits/scope-integration/DATUM_PRODUCT_MECHANICS_IMPLEMENTATION_READINESS_AUDIT.md`.

## Verdict

READY_FOR_SPEC_INTEGRATION.

The product-mechanics decision family is coherent enough to become the scope
source for reshaping the existing specs. The decision and contract docs now
preserve the critical spine: one `DesignModel` assembled by `ProjectResolver`,
stable identity, `ComponentInstance`, explicit relationship axes, sparse
variants, Import Map `import_key`, one journaled `commit()`, PTY-real terminal
semantics, optional assistant semantics, evidence-limited standards posture,
and ZoneFill honesty.

Spec integration can begin, but it should not be a broad mechanical rewrite.
The existing specs still describe the old implementation surface in many
places. Integration must therefore preserve current-implementation truth while
adding the new target substrate and clearly separating compatibility behavior
from target product-mechanics behavior.

## Resolved Blockers

1. Canonical CLI spelling has been normalized in the product-mechanics and
   contract docs: CLI examples use `datum-eda ...`; `datum.*` is reserved for
   explicit MCP/tool-family names.
2. `docs/DATUM_PRODUCT_MECHANICS.md` now presents the ratified relationship
   split instead of a flat mixed state list.
3. Variant overlay wording now describes sparse authored overlays keyed by
   stable object identity with `model_revision` / `variant_revision`
   invalidation. `source_hash` is evidence only, not overlay identity.
4. `ReverseEngineered` is now treated as a `RelationshipKind`, not a second
   authored-intent/deviation axis.
5. Standards claim discipline remains intact: Datum may store, validate,
   explain, and export evidence, but must not claim to certify a product,
   process, part, project, or organization.

## Integration Risks To Preserve

1. Current specs do not yet have the product-mechanics authority spine. In
   particular, `specs/ENGINE_SPEC.md` remains type/IR oriented and does not
   define `ProjectResolver`, source shards as persistence partitions,
   `ComponentInstance`, `RelationshipKind`, derived relationship status,
   sparse variant overlays, Import Map `import_key`, single `commit()`,
   journaled recovery, or `ZoneFill` as first-class controlling spec concepts.
2. `specs/MCP_API_SPEC.md` still defines the current MCP contract around
   `open_project` / `save` / `close_project`, current method lists, and
   per-method mutation APIs. That conflicts with the target shared surface in
   `docs/contracts/AI_CLI_MCP_TOOL_SURFACE.md`: session/context, query, check,
   proposal, commit, artifact, and journal tool classes over a resolver-backed
   model. Integration needs an explicit "current implementation compatibility"
   versus "target product-mechanics contract" split before edits land.
3. Tool contracts are granular enough to drive spec integration, but the spec
   pass must still decide which nested command forms are canonical versus
   compatibility aliases.

## Non-Blocking Risks

- ZoneFill honesty is correctly described in decisions and contracts, but the
  contracts also state the subsystem is not implemented. Specs must avoid
  implying current production-copper correctness until `ZoneFill{Filled}` and
  hard findings for unfilled/stale/unsupported zones exist.
- Native board and schematic authoring still have documented private-write or
  non-atomic paths. The docs correctly call these migration defects, but spec
  language must not accidentally legitimize them as supported alternatives.
- Standards wording is mostly disciplined. The standards spec correctly says
  Datum is a compliance substrate, not a certifying authority. The main risk is
  the word `Implemented` in standards disposition tables being misread as
  external compliance rather than "normative support exists in Datum specs and
  should have implementation evidence."
- Import identity is generally corrected to Import Map `import_key`; residual
  `source_hash` mentions appear acceptable when framed as evidence or a defect
  being replaced. Integration should still audit every promoted spec sentence
  that mentions import identity.
- PTY terminal and assistant boundary language is strong in decisions 005 and
  006. The risk is downstream UI/spec integration compressing them into one
  "AI command lane" and losing the real-terminal requirement.

## Required Guardrails During Spec Integration

1. Add a spec-integration preface or migration section to `specs/ENGINE_SPEC.md`
   before importing product mechanics. It must define `ProjectResolver`,
   source-shard authority boundaries, `ComponentInstance`, relationships,
   variants, Import Map `import_key`, `commit()`, journal, `model_revision`,
   and `ZoneFill`.
2. Add a target/current split to `specs/MCP_API_SPEC.md` so the current
   method-list contract remains truthful while the target CLI/MCP classes from
   `AI_CLI_MCP_TOOL_SURFACE.md` become the integration destination.
3. Produce one canonical command taxonomy for shared tools and domain tools,
   including compatibility aliases and not-supported-yet markers, before
   updating `PROGRESS.md` or parity manifests.
4. Preserve standards claim discipline during integration: no wording may imply
   Datum certifies a project, process, product, part, or organization.

## Suggested Spec Integration Order

1. `specs/ENGINE_SPEC.md`: integrate the authority, identity, resolver,
   relationship, variant, commit/journal, revision, Import Map, and ZoneFill
   substrate first. This prevents downstream specs from inventing local
   authority models.
2. `specs/MCP_API_SPEC.md`: split current implementation from target
   product-mechanics surface, then map shared CLI/MCP contract classes and
   domain tools onto that surface.
3. `specs/CHECKING_ARCHITECTURE_SPEC.md`: promote `CheckRun`,
   `CheckFinding`, standards/process basis, waiver/deviation, proposal-first
   repair, and revision-keyed invalidation after the engine substrate exists.
4. `specs/STANDARDS_COMPLIANCE_SPEC.md`: align standards disposition and
   evidence language with the new resolver/check/artifact model, retaining the
   substrate-not-certification boundary.
5. `specs/PROGRESS.md`: update only after the controlling specs have been
   changed, keeping current implementation status separate from target
   contract status.

## Evidence Notes

- `CLAUDE.md` states the naming rule: `Datum EDA` for product naming and
  `datum-eda` for machine identifiers.
- `docs/audits/scope-integration/DATUM_PRODUCT_MECHANICS_IMPLEMENTATION_READINESS_AUDIT.md` records the
  ratified mechanism vocabulary and acceptance checklist covering
  `ProjectResolver`, Import Map `import_key`, single `commit()`, journal,
  `RelationshipKind`, derived status, sparse variants, ZoneFill honesty,
  PTY terminal behavior, assistant limits, and standards claim discipline.
- `docs/decisions/PRODUCT_MECHANICS_003_SCHEMATIC_PCB_AUTHORITY.md` separates
  `RelationshipKind`, derived status, authored intent, and derived
  `NotApplicableForVariant`.
- `docs/decisions/PRODUCT_MECHANICS_005_EMBEDDED_TERMINAL.md` and
  `docs/decisions/PRODUCT_MECHANICS_006_ASSISTANT_SURFACE.md` preserve the
  PTY terminal / assistant boundary and deny hidden design mutation paths.
- `docs/contracts/AI_CLI_MCP_TOOL_SURFACE.md` defines the shared tool classes
  and identifies missing target substrate: generic `commit()` /
  `OperationBatch`, durable fsync journal, import provenance query, and
  ZoneFill honesty.
- `docs/contracts/PCB_LAYOUT_TOOL_CONTRACT.md`,
  `docs/contracts/SCHEMATIC_AUTHORING_TOOL_CONTRACT.md`, and
  `docs/contracts/LIBRARY_AUTHORING_TOOL_CONTRACT.md` are granular enough to
  guide implementation slices, but they also document current private-write or
  missing-substrate gaps that must remain marked as defects.
- `specs/STANDARDS_COMPLIANCE_SPEC.md` already contains the correct
  certification boundary: Datum may store, validate, export, and explain
  compliance-relevant metadata, but must not claim certification merely because
  fields exist.
- `specs/MCP_API_SPEC.md` remains truthful about the current MCP method list,
  but that list is not the same as the target product-mechanics tool surface.
