# Datum Product Mechanics Research Synthesis

Status: tracked research synthesis driving product-mechanics decisions.
Date: 2026-06-14

Source research:
- `research/product-mechanics/DATUM_PRODUCT_MECHANICS_RESEARCH.md`

Companion docs:
- `docs/DATUM_PRODUCT_MECHANICS.md`
- `docs/audits/scope-integration/DATUM_PRODUCT_MECHANICS_REVIEW_AGENDA.md`
- `docs/audits/scope-integration/NORTH_STAR_PROJECT_AUDIT.md`

## Purpose

This document promotes the product-mechanics research into tracked project
documentation. It is the bridge between research and decision records.

The ignored `/research` file may contain working notes, but this document is
the versioned synthesis that should drive Datum's product documentation,
decision records, and future roadmap correction.

## Core Finding

Datum's missing layer is a coherent product mechanics model:

- one canonical `DesignModel` rather than segregated schematic/PCB/library
  authorities
- domain projections for electrical, physical, library, rules, manufacturing,
  and analysis work
- composable tabs, panes, PiP, tiled views, and saved workbench profiles as UI
  arrangements over the design model
- live manufacturing projections so Gerber, NC drill, paste, soldermask,
  panel, BOM/PnP, and assembly outputs are visible during design
- one canonical edit path for every design-state mutation
- first-class schematic, PCB, library, rule, check, proposal, transaction, and
  artifact primitives
- manual editor workflows that do not require AI
- optional AI/tooling workflows that use the same primitives as manual tools
- standards and rules that drive checks, proposals, and sign-off behavior
- project/workspace state visible to GUI, CLI, terminal-launched agents, MCP
  tools, scripts, and importers

The first decision must be the Unified Design Model, followed immediately by
the Canonical Edit Model. Without the unified authority decision, the edit
model can accidentally encode conventional EDA segregation. Without the edit
model, every surface can become its own mutation system: GUI tools, CLI
commands, assistant actions, import repair, DRC correction proposals, and AI
agent edits.

## Research Inputs

Local Datum inputs:
- `docs/DATUM_PRODUCT_MECHANICS.md`
- `docs/audits/scope-integration/DATUM_PRODUCT_MECHANICS_REVIEW_AGENDA.md`
- `docs/audits/scope-integration/NORTH_STAR_PROJECT_AUDIT.md`
- `docs/AUTHORING_TOOLS.md`
- `docs/LIBRARY_ARCHITECTURE.md`
- `docs/ALTIUM_LESSONS.md`
- `docs/COMPETITIVE_ANALYSIS.md`
- `specs/CHECKING_ARCHITECTURE_SPEC.md`
- `specs/STANDARDS_COMPLIANCE_SPEC.md`

External reference points:
- Altium/Altium Designer public descriptions: unified electronics development,
  schematic capture, PCB layout, sourcing, versioning, manufacturing hand-off,
  component data, 3D/MCAD, and collaboration.
- OrCAD/Cadence public descriptions: Capture, PSpice, PCB Designer, CIS,
  schematic-to-PCB integration, automation, and DRC.
- EDA/AI research: PCBSchemaGen, SchGen, and SmartonAI.

## Product Mechanics Implications

### Professional EDA Is An Integrated Project System

Datum's `Project` must be more than a folder of files. It must bind documents,
libraries, rules, checks, artifacts, workspace state, provenance, and
transaction/proposal state. GUI, CLI, MCP, terminal-launched agents, scripts,
and imports must see the same project truth.

### Schematic And PCB Are Projections, Not Separate Authorities

The schematic is normally the authored electrical-intent projection. The PCB
is normally the physical-implementation projection. Datum should support other
workflows, including reverse engineering and imported-board recovery, but
board-side edits must not silently redefine electrical intent.

Electrical and physical data should live in one `DesignModel` with explicit
relationship states such as implemented, pending implementation, accepted
layout deviation, unresolved mismatch, reverse engineered, board-only object,
and schematic-only object.

### Rules And Constraints Are Product Mechanics

Rules must influence editing tools, live feedback, DRC/ERC, proposals, and
manufacturing sign-off. Standards-aware checks belong in the same checking
architecture as other DRC/ERC findings, not in isolated research artifacts.

### Libraries Are Foundational Product Data

Datum needs explicit library primitives: `Unit`, `Symbol`, `Entity`, `Part`,
`Package`, `Footprint`, `Padstack`, `PinPadMap`, and `ModelAttachment`.
Imported libraries can seed Datum, but cannot remain Datum's native library
architecture.

### AI Needs Semantic Primitives

AI should operate over stable object IDs, typed operations, constraints,
topology, checks, proposals, and diffs. It should not depend on screenshots,
GUI scraping, or private mutation paths.

### A Real Terminal Is Necessary But Not Sufficient

Datum's embedded terminal must be a real PTY-backed shell, but the shell is not
the mutation authority. Terminal-launched agents should use Datum CLI/MCP/tools
to inspect project state and create proposals or operations.

### Manufacturing Outputs Are Live Projections

Gerber, NC drill, paste, soldermask, panel, BOM/PnP, assembly, and output-job
views should be live production projections over the `DesignModel`, not
after-the-fact export artifacts. Exported files are snapshots of those
projections at a model revision.

Panelization belongs to the manufacturing projection, not the source board
layout. A panel references board instances and adds production-only features
such as rails, tabs, mouse bites, V-scores, fiducials, tooling holes, coupons,
and panel notes.

## Proposed Primitive Set

Project/workspace primitives:
- `Project`
- `DesignModel`
- `Workspace`
- `Document`
- `Projection`
- `EditorContext`
- `RelatedContext`
- `Viewport`
- `ViewTab`
- `Pane`
- `LayoutGraph`
- `WorkbenchProfile`
- `PinnedContext`
- `NavigationStack`
- `SelectionContext`

Design-domain primitives:
- `Schematic`
- `Sheet`
- `Symbol`
- `SchematicObject`
- `PCB`
- `BoardObject`
- `Net`
- `ConnectivityGraph`
- `ECO`

Library primitives:
- `Unit`
- `Entity`
- `Part`
- `Package`
- `Footprint`
- `Padstack`
- `ModelAttachment`
- `PinPadMap`

Rule/check primitives:
- `Constraint`
- `Rule`
- `RuleScope`
- `CheckRun`
- `CheckFinding`
- `Waiver`
- `Deviation`

Change primitives:
- `Operation`
- `OperationBatch`
- `Proposal`
- `Transaction`
- `Diff`
- `Provenance`

Tooling/artifact primitives:
- `ToolMode`
- `Command`
- `TerminalSession`
- `AgentSession`
- `Artifact`
- `OutputJob`
- `ManufacturingProjection`
- `PanelProjection`
- `ManufacturingPlan`

## Decision Sequence

Research drives the following decision records:

0. `docs/decisions/PRODUCT_MECHANICS_000_UNIFIED_DESIGN_MODEL.md`
0B. `docs/decisions/PRODUCT_MECHANICS_000B_VIEW_COMPOSITION_AND_LIVE_PRODUCTION.md`
0C. `docs/decisions/PRODUCT_MECHANICS_000C_UNIFIED_MODEL_FEASIBILITY.md`
0D. `docs/decisions/PRODUCT_MECHANICS_000D_STORAGE_AND_VERSIONING_MODEL.md`
0E. `docs/decisions/PRODUCT_MECHANICS_000E_MULTI_SHEET_MULTI_BOARD_MODEL.md`
0F. `docs/decisions/PRODUCT_MECHANICS_000F_LIVE_PRODUCTION_PROOF_MODEL.md`
1. `docs/decisions/PRODUCT_MECHANICS_001_CANONICAL_EDIT_MODEL.md`
2. `docs/decisions/PRODUCT_MECHANICS_002_MANUAL_EDITOR_BASELINE.md`
3. `docs/decisions/PRODUCT_MECHANICS_003_SCHEMATIC_PCB_AUTHORITY.md`
4. `docs/decisions/PRODUCT_MECHANICS_004_AI_TOOLING_CONTRACT.md`
5. `docs/decisions/PRODUCT_MECHANICS_005_EMBEDDED_TERMINAL.md`
6. `docs/decisions/PRODUCT_MECHANICS_006_ASSISTANT_SURFACE.md`
7. `docs/decisions/PRODUCT_MECHANICS_007_PROJECT_WORKSPACE_MODEL.md`
8. `docs/decisions/PRODUCT_MECHANICS_008_LIBRARY_COMPONENT_SYSTEM.md`
9. `docs/decisions/PRODUCT_MECHANICS_009_RULES_CONSTRAINTS_CHECKS.md`
10. `docs/decisions/PRODUCT_MECHANICS_010_INDUSTRY_STANDARDS_COMPLIANCE.md`
11. `docs/decisions/PRODUCT_MECHANICS_011_IMPORT_INTEROP_ROLE.md`
12. `docs/decisions/PRODUCT_MECHANICS_012_APPLICATION_QUALITY_BAR.md`

## Immediate Documentation Action

Item #0, Unified Design Model, should be documented first because it defines
the authority that all operations mutate.

Item #0B, View Composition And Live Production, should be documented alongside
Item #0 because it defines how projections appear to the user and how
manufacturing outputs become live quality-control surfaces instead of late
export artifacts.

Item #0C, Unified Model Feasibility, should be reviewed before the unified
model is treated as final architecture. It compares conventional segmented
authority, monolithic unified storage, and unified authority with segmented
storage.

Item #0D, Storage And Versioning Model, should define how one `DesignModel`
authority can be persisted as deterministic storage shards without turning
those shards into separate schematic, PCB, library, or manufacturing
authorities.

Item #0E, Multi-Sheet And Multi-Board Model, should define how multiple sheets,
hierarchy, boards, modules, daughtercards, variants, and board-first flows fit
inside one project model without turning sheets or boards into separate
authorities.

Item #0F, Live Production Proof Model, should define how Gerber, NC drill,
soldermask, paste, BOM/PnP, assembly, and panel projections can be proven as
live manufacturing views whose exported files are revisioned snapshots.

Item #1, Canonical Edit Model, should follow immediately because it controls:

- manual GUI editing
- CLI commands
- MCP tools
- terminal-launched agents
- assistant proposals
- imports and import repair
- ECO flows
- DRC/ERC correction proposals
- undo/redo
- provenance and auditability

No new feature roadmap should be written until Item #1 through Item #4 are at
least reviewed in draft form.
