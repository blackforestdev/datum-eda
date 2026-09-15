# Datum Product Mechanics Documentation Goals

Status: working documentation plan.
Date: 2026-06-18

Companion docs:
- `docs/DATUM_PRODUCT_MECHANICS.md`
- `docs/audits/scope-integration/DATUM_PRODUCT_MECHANICS_RESEARCH_SYNTHESIS.md`
- `docs/audits/scope-integration/DATUM_PRODUCT_MECHANICS_REVIEW_AGENDA.md`
- `docs/audits/scope-integration/NORTH_STAR_PROJECT_AUDIT.md`

## Purpose

Define the next documentation stages needed to validate Datum's product
mechanics before new roadmap or feature implementation work resumes.

The goal is to keep research, decisions, feasibility checks, and eventual
implementation scope connected. Research may remain in ignored working folders,
but the research product must be reflected in tracked development
documentation.

## Current Premise

Datum is exploring:
- one canonical `DesignModel`
- domain projections instead of segregated schematic/PCB/library authorities
- composable tabs, panes, PiP, tiled views, and saved workbench profiles
- live manufacturing projections for Gerber, NC drill, soldermask, paste,
  panelization, BOM/PnP, and assembly output
- one canonical operation/proposal/transaction model

These are hypotheses until feasibility proof gates are reviewed.

## Stage Goals

### Stage 0: Unified Authority Hypothesis

Tracked docs:
- `docs/decisions/PRODUCT_MECHANICS_000_UNIFIED_DESIGN_MODEL.md`
- `docs/decisions/PRODUCT_MECHANICS_000B_VIEW_COMPOSITION_AND_LIVE_PRODUCTION.md`
- `docs/decisions/PRODUCT_MECHANICS_000C_UNIFIED_MODEL_FEASIBILITY.md`

Goal:
- Capture the unified model, view-composition, and live-production ideas as
  reviewable hypotheses, not final doctrine.

Acceptance:
- Docs state that the unified model is pending feasibility review.
- Docs describe conventional alternatives and failure modes.
- Docs identify proof gates before implementation depends on the model.

### Stage 1: Storage And Versioning

Target doc:
- `docs/decisions/PRODUCT_MECHANICS_000D_STORAGE_AND_VERSIONING_MODEL.md`

Goal:
- Define how Datum can maintain one `DesignModel` authority while persisting
  deterministic storage shards suitable for git diffs, review, merge, and
  artifact traceability.

Acceptance:
- Defines storage shard classes.
- Defines model/object/transaction/artifact/library/variant revision concepts.
- Maps common edits to expected touched shards.
- Defines generated artifact policy.
- Explains how shards avoid becoming separate authorities.
- Lists proof gates and owner questions.

### Stage 2: Multi-Sheet And Multi-Board Model

Target doc:
- `docs/decisions/PRODUCT_MECHANICS_000E_MULTI_SHEET_MULTI_BOARD_MODEL.md`

Goal:
- Define how multiple schematic sheets, multiple boards, board-only flows,
  schematic-only objects, modules, daughtercards, and alternate physical
  implementations fit inside one design model.

Acceptance:
- Distinguishes sheet from electrical design.
- Distinguishes board from physical design.
- Defines relationship states between electrical and physical domains.
- Covers one electrical design to many boards, many electrical scopes to one
  board, and reverse-engineered board-first flows.

### Stage 3: Live Production And Panelization Proof Model

Target doc:
- `docs/decisions/PRODUCT_MECHANICS_000F_LIVE_PRODUCTION_PROOF_MODEL.md`

Goal:
- Define the first concrete proof path for live Gerber/NC drill/mask/paste
  projections and panelization as manufacturing-domain state.

Acceptance:
- Defines the minimum live production projection subset.
- Defines panel objects and panel rules needed for the first proof.
- Defines board versus panel output artifact identity.
- Defines validation expectations for projection/export equivalence.

### Stage 4: Canonical Edit Model Reconciliation

Target doc:
- update `docs/decisions/PRODUCT_MECHANICS_001_CANONICAL_EDIT_MODEL.md`

Goal:
- Reconcile operation/proposal/transaction semantics with the feasibility
  decisions from stages 1-3.

Acceptance:
- Operations target `DesignModel` objects and relationships.
- Proposals cover cross-domain, standards, import-repair, and manufacturing
  changes.
- Transactions include revision/provenance requirements from the storage model.

### Stage 5: Complete Product-Mechanics Decision Set

Target docs:
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

Goal:
- Convert every open review-agenda topic into a decision record so Datum has a
  complete product-mechanics documentation baseline before roadmap rewrite or
  implementation planning resumes.

Acceptance:
- Each review-agenda topic has a corresponding decision record.
- Each record states product intent, user-visible behavior, manual workflow
  requirements, optional AI/tooling behavior, affected primitives, standards
  impact, non-goals, first proof slice, and owner questions.
- Decision records remain draft-for-review unless explicitly accepted by the
  project owner.
- The research synthesis and review agenda point to the completed decision set.
- No implementation milestone treats these drafts as approved architecture
  until owner review is complete.

## Multi-Agent Workstreams

The current completion pass is split into disjoint documentation workstreams:

- Multi-sheet and multi-board model:
  `PRODUCT_MECHANICS_000E_MULTI_SHEET_MULTI_BOARD_MODEL.md`
- Live production and panelization proof:
  `PRODUCT_MECHANICS_000F_LIVE_PRODUCTION_PROOF_MODEL.md`
- Manual editor, workspace model, and application quality:
  `PRODUCT_MECHANICS_002_MANUAL_EDITOR_BASELINE.md`,
  `PRODUCT_MECHANICS_007_PROJECT_WORKSPACE_MODEL.md`, and
  `PRODUCT_MECHANICS_012_APPLICATION_QUALITY_BAR.md`
- AI tooling, terminal, and assistant surfaces:
  `PRODUCT_MECHANICS_004_AI_TOOLING_CONTRACT.md`,
  `PRODUCT_MECHANICS_005_EMBEDDED_TERMINAL.md`, and
  `PRODUCT_MECHANICS_006_ASSISTANT_SURFACE.md`
- Library, rules/checks, standards, and import role:
  `PRODUCT_MECHANICS_008_LIBRARY_COMPONENT_SYSTEM.md`,
  `PRODUCT_MECHANICS_009_RULES_CONSTRAINTS_CHECKS.md`,
  `PRODUCT_MECHANICS_010_INDUSTRY_STANDARDS_COMPLIANCE.md`, and
  `PRODUCT_MECHANICS_011_IMPORT_INTEROP_ROLE.md`

The main thread owns integration:
- update `docs/audits/scope-integration/DATUM_PRODUCT_MECHANICS_RESEARCH_SYNTHESIS.md`
- update `docs/audits/scope-integration/DATUM_PRODUCT_MECHANICS_REVIEW_AGENDA.md`
- draft or reconcile `PRODUCT_MECHANICS_003_SCHEMATIC_PCB_AUTHORITY.md`
- reconcile `PRODUCT_MECHANICS_001_CANONICAL_EDIT_MODEL.md` after stages
  000E and 000F exist

## Implementation-Readiness Pass

The next pass closes the gap between product-mechanics decisions and
implementation-driving documentation.

Goal:
- Convert the decision set from "what/why" into enough "how" for an
  implementation architecture plan to be written without guessing.

Four logical steps:
- Reconcile `PRODUCT_MECHANICS_003_SCHEMATIC_PCB_AUTHORITY.md` with the
  ratified relationship, variant, resolver, and commit vocabulary from
  decisions 000 through 001.
- Mechanize `PRODUCT_MECHANICS_002_MANUAL_EDITOR_BASELINE.md`,
  `PRODUCT_MECHANICS_007_PROJECT_WORKSPACE_MODEL.md`, and
  `PRODUCT_MECHANICS_012_APPLICATION_QUALITY_BAR.md` with operation families,
  workspace persistence, crash recovery, durable undo, and quality gates.
- Mechanize `PRODUCT_MECHANICS_004_AI_TOOLING_CONTRACT.md`,
  `PRODUCT_MECHANICS_005_EMBEDDED_TERMINAL.md`, and
  `PRODUCT_MECHANICS_006_ASSISTANT_SURFACE.md` with PTY/session behavior,
  Datum context injection, CLI/MCP contracts, proposal application, and
  private-mutation boundaries.
- Mechanize `PRODUCT_MECHANICS_008_LIBRARY_COMPONENT_SYSTEM.md`,
  `PRODUCT_MECHANICS_009_RULES_CONSTRAINTS_CHECKS.md`,
  `PRODUCT_MECHANICS_010_INDUSTRY_STANDARDS_COMPLIANCE.md`, and
  `PRODUCT_MECHANICS_011_IMPORT_INTEROP_ROLE.md` with schema concepts,
  check execution, standards metadata, import provenance, repair operations,
  and artifact/proposal mechanics.

Coordination model:
- A conductor owns
  `docs/audits/scope-integration/DATUM_PRODUCT_MECHANICS_IMPLEMENTATION_READINESS_AUDIT.md`.
- Workers own only their assigned decision records.
- The main thread integrates only after the conductor audit and worker outputs
  converge.

Acceptance:
- Later decision records use the same vocabulary as 000 through 001:
  `ProjectResolver`, stable IDs, `ComponentInstance`, derived net identity,
  `RelationshipKind`, derived relationship status, sparse variant overlays,
  three-valued population, `ZoneFill`, single `commit()`, transaction journal,
  proposals, artifacts, and provenance.
- No document describes direct file mutation, GUI bypasses, or assistant/agent
  private powers as acceptable implementation paths.
- Each mechanism doc identifies first proof slices that can map to concrete
  engineering tasks.
- Remaining open questions are narrow owner decisions, not missing
  architecture.

## Agent Delegation Policy

Documentation agents may draft stages, but their output must be reviewed into
tracked docs.

Agent outputs should:
- use existing product-mechanics docs as source context
- write concrete decision or feasibility docs
- avoid creating disconnected research artifacts
- avoid claiming hypotheses are final decisions
- list files changed and unresolved questions

## Stop Conditions

Do not proceed to implementation scope if:
- storage/versioning remains undefined
- multiple sheets/boards remain ambiguous
- panelization mutates board source geometry by default
- artifacts can be mistaken for source authority
- operations can bypass transaction/provenance requirements
- research conclusions exist only in ignored research notes
