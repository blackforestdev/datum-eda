# Datum Product Mechanics Review Agenda

Status: working review agenda.
Date: 2026-06-14

## Purpose

This document breaks the unresolved Datum product-mechanics questions into
separate review tracks that should be worked through one by one with the
project owner before new roadmap or milestone work resumes.

The companion foundation is `docs/DATUM_PRODUCT_MECHANICS.md`.

The companion audit is `docs/audits/scope-integration/NORTH_STAR_PROJECT_AUDIT.md`.

The companion research synthesis is
`docs/audits/scope-integration/DATUM_PRODUCT_MECHANICS_RESEARCH_SYNTHESIS.md`.

The companion documentation goals are
`docs/audits/scope-integration/DATUM_PRODUCT_MECHANICS_DOCUMENTATION_GOALS.md`.

## Review Method

Each topic should produce a short decision record before implementation work
continues in that area.

For each topic, record:
- product intent
- user-visible behavior
- manual workflow requirements
- optional AI/tooling behavior
- core primitives affected
- implementation surfaces affected
- standards/compliance impact, if any
- explicit non-goals
- first proof slice

Do not convert these topics directly into milestones until the decisions are
reviewed together.

## Topics To Work Through

### 0. Unified Design Model

Question:
- Should Datum use one canonical `DesignModel` with electrical, physical,
  library, rules, manufacturing, and analysis projections instead of
  segregated schematic/PCB/library authorities?

Why it matters:
- Conventional EDA tools often reflect historical separation between
  schematic capture and PCB layout programs.
- Datum should support familiar workflows without inheriting file/workspace
  segregation as product doctrine.
- The canonical edit model needs to know what authority operations mutate.

Decision outputs:
- design model authority
- projection versus workspace semantics
- related-context and viewport roles
- electrical/physical relationship states
- split-view non-requirements
- first proof slice

### 1. Canonical Edit Model

Question:
- What is the single mutation path for manual tools, GUI, CLI, agents, MCP,
  scripts, and imports?

Why it matters:
- Datum currently has split mutation authority across engine write operations,
  CLI project-file helpers, GUI text helpers, and assistant stubs.
- AI-assisted editing requires one inspectable operation/proposal/transaction
  model.

Decision outputs:
- canonical operation/transaction ownership
- proposal versus committed transaction semantics
- undo/redo requirements
- diff/preview requirements
- provenance requirements

### 1B. View Composition And Live Production

Question:
- How should Datum compose tabs, panes, PiP, tiled views, live CAM projections,
  and manufacturing-stage panelization over one `DesignModel`?

Why it matters:
- Users should not be forced into fixed schematic/PCB split-screen workflows.
- Gerber, NC drill, paste, soldermask, BOM/PnP, panel, and assembly outputs
  should be visible during design, not only after export.
- Panelization belongs to manufacturing production intent, not source board
  geometry.

Decision outputs:
- tab and pane composition model
- PiP/tiled/floating/pinned sidecar expectations
- live manufacturing projection model
- Gerber/NC drill/paste/soldermask preview requirements
- panelization as manufacturing projection
- artifact snapshot/versioning requirements

### 1C. Unified Model Feasibility

Question:
- Is unified authority with projections and segmented storage actually viable,
  or is conventional segmented schematic/PCB/library authority simpler and
  safer?

Why it matters:
- Innovation is only useful if it improves the product without creating
  unacceptable persistence, versioning, collaboration, performance, or
  migration risk.
- The unified model should remain a hypothesis until key logistics are proven.

Decision outputs:
- architecture comparison
- feasibility risks
- required proof gates
- storage/versioning posture
- migration posture
- criteria for rejecting or narrowing the unified model

### 1D. Storage And Versioning Model

Question:
- How can Datum maintain one `DesignModel` authority while persisting project
  state as deterministic storage shards that work with git diffs, merges,
  artifacts, and revisions?

Why it matters:
- The unified model is only practical if storage shards remain persistence
  partitions rather than separate schematic, PCB, library, or manufacturing
  authorities.
- Versioning, object identity, generated artifacts, and transaction provenance
  must be clear before the canonical edit model is locked.

Decision outputs:
- storage approach comparison
- proposed shard classes
- stable object identity rules
- model/object/transaction/artifact/library/variant revision semantics
- common edit shard-impact expectations
- generated artifact policy
- loader/resolver authority rules
- git diff and merge proof gates

### 1E. Multi-Sheet And Multi-Board Model

Question:
- How do multiple schematic sheets, hierarchical sheets, multiple boards,
  modules, daughtercards, variants, and board-first flows fit inside one
  project model?

Why it matters:
- Datum should not assume one schematic file maps to one PCB file.
- Sheets and boards must be authoring or implementation scopes, not hidden
  authorities.
- The schematic-to-PCB relationship must support normal design flow while
  preserving explicit board-only, schematic-only, and reverse-engineered
  states.

Decision outputs:
- sheet versus electrical-design distinction
- board versus physical-design distinction
- electrical/physical scope model
- multi-board and module relationship rules
- variant relationship rules
- board-first and reverse-engineering policy
- proof gates for rejecting or narrowing the unified model

### 1F. Live Production Proof Model

Question:
- What minimum proof demonstrates that Gerber, NC drill, soldermask, paste,
  BOM/PnP, assembly, and panel outputs can behave as live production
  projections?

Why it matters:
- Manufacturing visibility should be part of design quality control, not a
  late export-only step.
- Panelization must not mutate source board geometry by default.
- Exported artifacts must be snapshots of known model, output-job, variant,
  and manufacturing-plan revisions.

Decision outputs:
- minimum live production projection subset
- board versus panel artifact identity
- panel object model
- projection/export equivalence expectations
- manufacturing-only geometry rules
- output artifact revision/provenance rules

### 2. Manual Editor Baseline

Question:
- What must a user be able to do manually before Datum can honestly be called a
  usable EDA editor across electrical, physical, library, rules, and
  manufacturing projections?

Why it matters:
- AI must be optional.
- CLI-only authoring is not the same as a professional GUI editor.

Decision outputs:
- minimum electrical projection capability
- minimum physical projection capability
- minimum library editor capability
- inspector/property-editing rules
- snapping/grid/tool-mode requirements

### 3. Electrical And Physical Relationship

Question:
- How do electrical-intent objects, physical-implementation objects, nets,
  ECO/relationship states, and physical-side edits relate inside one
  `DesignModel`?

Why it matters:
- Datum should not enforce one rigid workflow, but it does need authority rules.
- Physical edits must not silently redefine electrical intent.

Decision outputs:
- electrical-intent authority rules
- physical-implementation rules
- ECO/back-annotation policy
- reverse-engineering/import exception policy

### 4. AI Tooling Contract

Question:
- What exact tools and structured context should agents receive so AI feels
  intentionally integrated rather than duct-taped on?

Why it matters:
- The power of Datum is not just running an agent in a terminal.
- The power is EDA-native tooling: object IDs, queries, proposals, checks,
  diffs, artifacts, and provenance.

Decision outputs:
- initial agent tool list
- project/context discovery contract
- selection/context model
- proposal/apply contract
- safety and approval policy
- MCP/CLI/terminal relationship

### 5. Embedded Terminal

Question:
- What quality bar makes Datum's terminal a real terminal emulator rather than
  a fake command pane?

Why it matters:
- Users should be able to run normal Linux shell workflows and launch agents
  like `codex`, `claude`, or other tools inside Datum.

Decision outputs:
- PTY/session requirements
- tab/session model
- cwd/env/project-context behavior
- copy/paste/selection/scrollback requirements
- ANSI/VT behavior
- agent-launch expectations

### 6. Assistant Surface

Question:
- Should Datum have a built-in assistant panel, and if so, what is its role
  relative to the real terminal and external agents?

Why it matters:
- A built-in assistant tab should not become the primary AI architecture.
- It must not replace manual tools or the real terminal.

Decision outputs:
- whether assistant panel stays, changes, or is deferred
- allowed actions
- relationship to proposals and transactions
- relationship to terminal-launched agents

### 7. Project And Workspace Model

Question:
- What are Datum's first-class projections, editor contexts, workspaces, and
  viewports, and how do they share one design model?

Why it matters:
- Electrical, physical, library, rules/checks, manufacturing, terminal,
  assistant, and analysis views need a coherent application model without
  forcing permanent split-screen workflow management.

Decision outputs:
- workspace list
- projection list
- viewport role
- shared project state model
- document/view separation
- project navigation behavior
- persistence expectations

### 8. Library And Component System

Question:
- How do users create, edit, bind, validate, and reuse symbols, footprints,
  parts, packages, models, and metadata?

Why it matters:
- Professional EDA quality depends heavily on library quality.
- Standards compliance, supply-chain metadata, and AI tooling all depend on
  reliable component primitives.

Decision outputs:
- symbol/footprint/part/package ownership
- binding model
- library editor baseline
- provenance rules
- metadata and model attachment policy

### 9. Rules, Constraints, And Checks

Question:
- How are constraints authored, scoped, checked, explained, and used by manual
  tools and agents?

Why it matters:
- Rules are where professional EDA tools become reliable.
- AI tooling should propose changes against rules, not freehand geometry.

Decision outputs:
- constraint primitive definitions
- rule scope model
- ERC/DRC/check report requirements
- standards-aware DRC findings for manufacturability observables such as
  pad/mask/paste aperture policy
- waiver and deviation model
- live versus batch checking behavior

### 10. Industry Standards And Compliance

Question:
- What does "fully industry-standard compliant" mean for Datum, and how should
  that be represented without making false certification claims?

Why it matters:
- Standards compliance was an early Datum requirement.
- Existing standards docs are substantial, but they must be connected to the
  product-mechanics model and future roadmap.

Existing project documentation:
- `specs/STANDARDS_COMPLIANCE_SPEC.md`
- `docs/STANDARDS_COMPLIANCE_INTEGRATION_GUIDANCE.md`
- `docs/IPC_FOOTPRINT_SYSTEM.md`
- `docs/STANDARDS_AUDIT_BATCH_1_GUIDANCE.md`
- `docs/audits/scope-integration/STANDARDS_DOMAINS_3_4_INTEGRATION_GUIDANCE.md`
- `research/standards-audit/STANDARDS_AUDIT.md`
- `research/ipc-compliance/IPC_COMPLIANCE_RESEARCH.md`

Decision outputs:
- standards-compliance product promise
- distinction between compliance substrate and certifying authority
- standards registry ownership
- IPC footprint/library requirements
- DRC/check requirements for IPC/process observables, including mask expansion
  and paste/stencil reduction
- schematic symbol/drawing convention requirements
- project compliance metadata requirements
- audit trail and sign-off requirements
- import/export standards boundaries
- milestone gates for standards-facing changes

Important framing:
- Datum can store, validate, explain, and export standards-relevant data.
- Datum should not claim to certify a product or organization against a
  standard unless a future explicit compliance workflow supports that claim.
- Standards support must be represented as implemented, planned,
  reference-only, deferred with prerequisite, or out of scope.

### 11. Import And Interop Role

Question:
- What is import for, and when is it allowed to drive development?

Why it matters:
- Import fidelity has become the active product center.
- Import should support migration, fixtures, compatibility, and reverse
  engineering, not define Datum's product identity.

Decision outputs:
- import role statement
- supported migration paths
- source-provenance rules
- standards-aware import audit policy
- boundaries for new importer work

### 12. Application Quality Bar

Question:
- What does Altium-class ambition mean in concrete application terms?

Why it matters:
- "Professional quality" must become measurable enough to guide agents and
  implementation.

Decision outputs:
- interaction quality expectations
- keyboard and command behavior
- inspector/property editing quality
- responsiveness and stability targets
- error handling and recovery expectations
- visual density and workspace polish expectations

## Review Order

Recommended order:

0. Unified Design Model
1. Canonical Edit Model
1B. View Composition And Live Production
1C. Unified Model Feasibility
1D. Storage And Versioning Model
1E. Multi-Sheet And Multi-Board Model
1F. Live Production Proof Model
2. Manual Editor Baseline
3. Electrical And Physical Relationship
4. AI Tooling Contract
5. Embedded Terminal
6. Assistant Surface
7. Project And Workspace Model
8. Library And Component System
9. Rules, Constraints, And Checks
10. Industry Standards And Compliance
11. Import And Interop Role
12. Application Quality Bar

This order starts with model authority, then mutation authority. The unified
design model defines what operations mutate; the canonical edit model defines
how mutations are created, reviewed, committed, audited, and exposed across
manual, AI-assisted, GUI, CLI, terminal, and standards-facing workflows. The
view-composition/live-production decision then defines how projections are
arranged and how manufacturing outputs become live quality-control surfaces.
The feasibility decision prevents those hypotheses from becoming final
architecture before logistics and failure modes are tested.

## Immediate Rule

Until these topics are reviewed, new implementation should be limited to:
- preserving current working state
- fixing regressions
- documenting actual capability
- building review artifacts

New feature work should not proceed from the existing M7/import-review center
by default.
