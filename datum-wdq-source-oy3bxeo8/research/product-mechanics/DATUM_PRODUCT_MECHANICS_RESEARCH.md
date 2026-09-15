# Datum Product Mechanics Research

Status: research synthesis for owner review.
Date: 2026-06-14

Companion docs:
- `docs/DATUM_PRODUCT_MECHANICS.md`
- `docs/DATUM_PRODUCT_MECHANICS_REVIEW_AGENDA.md`
- `docs/NORTH_STAR_PROJECT_AUDIT.md`

Intended next decision record:
- `docs/decisions/PRODUCT_MECHANICS_001_CANONICAL_EDIT_MODEL.md`

## Purpose

This research pass defines what Datum needs to decide before more roadmap or
milestone work resumes.

The goal is not to copy Altium, OrCAD, KiCad, Horizon, or any AI research
system. The goal is to identify the product mechanics and primitives Datum
must own so it can become a professional EDA application with optional
first-class AI collaboration.

## Executive Synthesis

Datum's missing layer is not another import feature, renderer, or agent chat
surface. The missing layer is a coherent product mechanics model:

- one canonical edit path for every state change
- first-class schematic, PCB, library, rule, check, proposal, transaction, and
  artifact primitives
- manual editor workflows that do not require AI
- optional AI/tooling workflows that use the same primitives as manual tools
- standards and rules that drive checks, proposals, and sign-off behavior
- project/workspace state that is visible to GUI, CLI, terminal-launched
  agents, MCP tools, scripts, and importers

The most important conclusion is that **Canonical Edit Model must be decided
first**. Without it, every other surface risks becoming a separate mutation
system: GUI tools, CLI commands, assistant actions, import repair, DRC
correction proposals, and AI agent edits.

## Research Inputs

Local Datum inputs:
- `docs/DATUM_PRODUCT_MECHANICS.md`
- `docs/DATUM_PRODUCT_MECHANICS_REVIEW_AGENDA.md`
- `docs/NORTH_STAR_PROJECT_AUDIT.md`
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
- EDA/AI research:
  - PCBSchemaGen: LLM schematic design using constraint-guided synthesis and
    verification against datasheet-derived knowledge and pin/topology
    constraints.
  - SchGen: semantic code representation for editable PCB schematics,
    emphasizing editing primitives and pin-name-based wiring over raw
    geometry-heavy formats.
  - SmartonAI: natural-language interaction over complex EDA tooling using
    task decomposition and tool/plugin execution.

## What Professional EDA Mechanics Imply

### 1. Professional EDA is an integrated project system

Professional tools are not just schematic editors or board editors. They bind
schematic capture, PCB layout, libraries, component data, constraints,
simulation/analysis, manufacturing output, collaboration, and change management
inside a project model.

Implication for Datum:
- `Project` must be more than a folder of files.
- Project state must include documents, libraries, rules, checks, artifacts,
  workspace state, provenance, and open transaction/proposal state.
- GUI, CLI, MCP, terminal-launched agents, scripts, and imports must see the
  same project truth.

### 2. Schematic and PCB must be related but not collapsed

The schematic is normally the authored electrical design. The PCB is normally
the physical implementation. Some workflows start elsewhere, but Datum still
needs authority rules so board edits do not silently redefine electrical
intent.

Implication for Datum:
- `Schematic` and `PCB` are both first-class primitives.
- `Net` is derived from authored electrical/physical objects, not manually
  edited as a disconnected artifact.
- ECO/annotation must be an explicit comparison/proposal surface.
- Board-side changes that imply schematic changes must produce explicit
  decisions.

### 3. Rules and constraints are product mechanics, not post-processing

Professional PCB work depends on constraints: net class, layer, clearance,
width, impedance, differential pair, manufacturing, assembly, and standards
rules. Rules must influence tools, live feedback, DRC/ERC, proposals, and
manufacturing sign-off.

Implication for Datum:
- `Constraint` and `Rule` need clear separation.
- Rules must be scoped, inspectable, deterministic, and explainable.
- Checks must produce machine-readable findings with stable object references.
- DRC/ERC must be useful to a manual user and to an agent.
- Standards-aware checks belong in the same checking architecture, not in a
  separate research artifact.

### 4. Libraries are foundational product data

Professional EDA quality depends on library quality: symbols, footprints,
padstacks, pin-to-pad maps, part identity, manufacturer metadata, lifecycle,
models, and standards basis. Imported libraries can seed Datum, but cannot
remain the architecture.

Implication for Datum:
- `LibraryPart`, `Symbol`, `Footprint`, `Package`, `Padstack`, and
  `ModelAttachment` must be explicit primitives.
- Datum should preserve source provenance for imported library data.
- Native library editing must eventually be a first-class manual workflow.
- Agents need library query, validation, and proposal tools rather than raw
  file editing.

### 5. AI needs semantic primitives, not screenshots

Recent AI/schematic research points in the same direction: LLMs work better
when they operate over semantic editing primitives, constraints, pin names,
topology, and verification loops rather than raw geometry-heavy formats.

Implication for Datum:
- The AI integration substrate should expose intent-level operations:
  place symbol, connect pin, assign part, set rule, propose ECO, route net,
  inspect DRC, generate output job.
- Agents should receive stable object IDs, structured context, and explicit
  proposal/apply APIs.
- AI should not mutate files behind Datum's back.
- A terminal-launched agent should be powerful because Datum exposes tools and
  project state, not because the agent scrapes the GUI.

### 6. A real terminal is necessary, not sufficient

The embedded terminal should behave like a normal Linux terminal. But the
terminal alone does not make Datum AI-native. The terminal lets users run
agents and tools; Datum's primitives let those agents act safely and
intelligently.

Implication for Datum:
- `TerminalSession` is an application primitive, but not an EDA mutation
  primitive.
- Terminal-launched agents must use Datum CLI/MCP/project tools for EDA edits.
- Terminal commands may create proposals or invoke operations, but the shell is
  not the authority model.

## Proposed Datum Primitive Set

This is a research proposal, not a final spec.

### Project and workspace primitives

- `Project`: root container, metadata, file layout, settings, active standards
  basis, workspace state, and artifact registry.
- `Workspace`: named UI/work context such as schematic, PCB, library,
  rules/checks, manufacturing, terminal, or assistant.
- `Document`: schematic sheet, PCB document, library document, rule file, or
  generated report.
- `SelectionContext`: current selected objects, active workspace, cursor/tool
  context, and visible check/proposal context.

### Design-domain primitives

- `Schematic`: authored electrical design surface.
- `Sheet`: schematic document or reusable hierarchical sheet.
- `Symbol`: schematic representation of a logical unit.
- `SchematicObject`: wire, junction, label, bus, bus entry, port, no-connect,
  power symbol, drawing primitive, field, note.
- `PCB`: physical implementation surface.
- `BoardObject`: component placement, pad, track, via, zone, keepout,
  dimension, board text, outline, stackup layer, mechanical/process geometry.
- `Net`: derived connectivity identity with domain-specific projections.
- `ConnectivityGraph`: derived graph for schematic and board connectivity.
- `ECO`: explicit comparison between source-of-intent and implementation.

### Library primitives

- `Unit`: electrical gate/unit definition and pin semantics.
- `Entity`: logical component definition and gate structure.
- `Part`: selectable/purchasable component identity with manufacturer and
  lifecycle metadata.
- `Package`: physical package/footprint definition.
- `Footprint`: land pattern and mechanical/process geometry.
- `Padstack`: copper/drill/mask/paste geometry and manufacturing semantics.
- `ModelAttachment`: SPICE, IBIS, Touchstone, thermal, 3D, or other model
  reference with provenance.
- `PinPadMap`: explicit mapping between logical pins and physical pads.

### Rule/check primitives

- `Constraint`: authored design intent or requirement.
- `Rule`: executable/scoped condition used by tools and checks.
- `RuleScope`: expression determining where a rule applies.
- `CheckRun`: deterministic execution record for ERC, DRC, standards checks,
  import audit, manufacturing validation, or output validation.
- `CheckFinding`: stable finding with code, severity, location, objects,
  explanation, waiver/deviation state, and optional proposal references.
- `Waiver`: explicit suppression or acceptance of a finding.
- `Deviation`: standards/process exception with rationale and provenance.

### Change primitives

- `Operation`: smallest typed edit command Datum understands.
- `OperationBatch`: ordered set of operations that should be applied together.
- `Proposal`: uncommitted suggested operation batch with rationale, diff,
  assumptions, checks, risks, and apply/reject/defer actions.
- `Transaction`: committed operation batch with stable identity, provenance,
  diff, affected objects, check state, undo/redo data, and audit record.
- `Diff`: machine-readable before/after object/property/geometry delta.
- `Provenance`: user/tool/import/script/AI origin, source references, time,
  acceptance path, and review state.

### Tooling and artifact primitives

- `ToolMode`: active manual editor mode such as select, place symbol, wire,
  route track, place via, edit zone, measure, or inspect.
- `Command`: command-palette, CLI, MCP, script, or assistant-invoked action
  that resolves to queries, proposals, or operations.
- `TerminalSession`: PTY-backed shell session with cwd/env/project context.
- `AgentSession`: optional external or built-in agent context with tool access
  and approval policy.
- `Artifact`: generated output such as Gerber, drill, BOM, PnP, PDF, STEP,
  IPC-2581, ODB++, report, or analysis result.
- `OutputJob`: reusable manufacturing/report generation recipe.

## Canonical Edit Model Research Conclusion

The first decision record should define this rule:

> Every design-state mutation in Datum must resolve to typed operations and
> commit through transactions. Suggestions, automation, DRC fixes, ECOs,
> imports, and AI actions must enter as proposals unless they are explicit
> user-authored direct edits.

### Operation

An operation is the smallest edit Datum understands semantically. It should be
typed, validated, serializable where practical, and replayable where practical.

Examples:
- `PlaceSymbol`
- `MoveSymbol`
- `DrawWire`
- `PlaceComponent`
- `RouteTrack`
- `PlaceVia`
- `SetRule`
- `AssignPart`
- `SetPadProcessAperture`
- `ApplyEcoChange`

Operations do not silently perform unrelated edits. Compound behaviors emit
operation batches.

### Proposal

A proposal is an uncommitted candidate change.

Sources:
- AI agent
- DRC/ERC correction suggestion
- ECO comparator
- import repair/audit workflow
- router/planner
- command preview
- batch script in review mode

Required fields:
- operation batch
- rationale
- expected result
- affected objects
- diff/preview
- checks run
- unresolved assumptions
- risks
- apply/reject/defer state
- provenance

### Transaction

A transaction is a committed change.

Sources:
- manual GUI direct edit
- accepted proposal
- CLI command in commit mode
- MCP operation with approval
- import/open conversion step
- script with configured trust policy

Required fields:
- stable transaction ID
- ordered operations
- changed objects
- before/after diff or equivalent undo data
- provenance
- timestamp
- check state if relevant
- undo/redo data

## Mapping To Review Agenda

### 1. Canonical Edit Model

Research recommendation:
- Decide this first.
- Make operation/proposal/transaction the only mutation authority.
- Treat GUI, CLI, MCP, scripts, importers, DRC repairs, ECOs, and agents as
  producers of operations or proposals, not separate mutation systems.

Decision questions:
- Are manual GUI edits allowed to commit immediately, or must every edit also
  be represented as a proposal first?
- Which operations are direct-edit safe?
- Which operations always require proposal review?
- What is the minimum transaction record for undo/redo and audit?
- What object identity scheme is required before transactions are trustworthy?

First proof slice:
- One schematic operation and one PCB operation use the same operation,
  transaction, diff, undo/redo, and provenance path from GUI or CLI.

### 2. Manual Editor Baseline

Research recommendation:
- Define minimum professional manual capability before more AI feature work.
- Schematic and PCB editors must have tool modes, snapping, inspector editing,
  selection, properties, undo/redo, and visible checks.

Decision questions:
- What is the smallest visible slice that proves Datum is an editor?
- Should schematic editor proof come before PCB editor proof?
- What manual board operation should be first: draw track, move component,
  place via, edit property, or board outline?

First proof slice:
- Manual schematic wire/symbol edit and manual PCB track edit committing
  through canonical transactions.

### 3. Schematic And PCB Authority

Research recommendation:
- Schematic is normally electrical source of truth.
- PCB is physical implementation.
- ECO mediates changes between them.
- Reverse-engineering/import flows are exceptions with explicit project mode.

Decision questions:
- Which board-side edits imply schematic ECO proposals?
- Are net renames allowed from PCB side?
- How does Datum represent imported boards with no schematic authority?

First proof slice:
- Add a schematic component, generate an ECO proposal to place a PCB component,
  accept it, and record the transaction.

### 4. AI Tooling Contract

Research recommendation:
- AI tools should operate over the same typed operations and proposals as
  manual workflows.
- AI should get structured context, not raw screenshots as primary data.

Decision questions:
- What are the first agent tools?
- What can an agent do without approval?
- How are agent proposals displayed and applied?
- How does an agent discover project state from a terminal session?

First proof slice:
- Terminal-launched agent can query selected object context, propose a bounded
  edit, run checks, and present a proposal without direct mutation.

### 5. Embedded Terminal

Research recommendation:
- The terminal must be a PTY-backed shell, separate from assistant UI.
- Terminal sessions should inherit project context and expose Datum-specific
  environment variables.

Decision questions:
- What environment variables should Datum inject?
- How do terminal sessions attach to project/workspace context?
- Should terminal sessions be stored in workspace state?

First proof slice:
- Open terminal, spawn `$SHELL`, run `pwd`, `ls`, `datum-cli` command, and
  launch an external agent command from project root.

### 6. Assistant Surface

Research recommendation:
- The built-in assistant is optional and secondary.
- It should be a proposal/review/conversation surface, not a fake terminal and
  not the primary AI architecture.

Decision questions:
- Keep, redesign, or defer the assistant panel?
- Is it allowed to create proposals?
- Is it allowed to apply transactions?

First proof slice:
- Assistant explains current selection and creates a proposal, but apply goes
  through the same review/transaction path as all other changes.

### 7. Project And Workspace Model

Research recommendation:
- Datum should define project/workspace/document separation before adding more
  GUI panels.

Decision questions:
- What workspaces are first-class?
- What is document state versus view state?
- How is active selection shared?

First proof slice:
- Project opens with schematic, PCB, rules/checks, terminal, and artifact
  workspace entries sharing one selection/context model.

### 8. Library And Component System

Research recommendation:
- Adopt the stronger Horizon-like separation: Unit, Symbol, Entity, Package,
  Part, Padstack, PinPadMap.
- Do not inherit KiCad's flat symbol/footprint binding as Datum's native
  model.

Decision questions:
- Which library primitives are required for first native schematic-to-PCB
  proof?
- How much part metadata is required before library authoring is useful?
- How are imported library objects normalized and traced?

First proof slice:
- Create native part with symbol, footprint, padstack, and pin-pad map; place
  it in schematic; generate PCB placement through ECO.

### 9. Rules, Constraints, And Checks

Research recommendation:
- Rules must drive editing tools and checks, not just batch reports.
- Standards-aware process geometry checks belong in DRC.

Decision questions:
- What is the first rule scope language subset?
- Are constraints separate from executable rules?
- How do waivers differ from standards deviations?

First proof slice:
- Rule-scoped DRC finding creates a correction proposal and records acceptance
  as a transaction.

### 10. Industry Standards And Compliance

Research recommendation:
- Datum should claim standards substrate and validation, not certification,
  unless a future explicit certification workflow exists.
- Standards support must be implementation-status tracked.

Decision questions:
- What standards are active gates for v1 primitives?
- What status labels are allowed?
- What standards metadata belongs in project state?

First proof slice:
- Project declares IPC/process basis; DRC flags pad mask/paste violations and
  offers explicit proposals without silent mutation.

### 11. Import And Interop Role

Research recommendation:
- Import is migration, fixtures, compatibility, reverse engineering, and audit.
- Import should not be the active product center unless it unlocks native
  authoring, validation, or standards migration.

Decision questions:
- What imported data becomes native truth?
- What remains source provenance?
- What import repairs are proposals versus automatic conversions?

First proof slice:
- Import board, preserve source geometry, emit standards/process findings,
  create optional repair proposals, and record accepted fixes as native
  transactions.

### 12. Application Quality Bar

Research recommendation:
- "Altium-class" should mean specific interaction qualities: direct
  properties, keyboard-first workflows, predictable tool modes, strong
  inspection, responsiveness, stable undo/redo, and clean recovery.

Decision questions:
- What responsiveness targets matter first?
- What inspector behavior is mandatory?
- What keyboard/command palette behavior is required?

First proof slice:
- Selection and property editing for a small object set feels direct,
  undoable, and visible through transaction history.

## Proposed Decision Record Sequence

1. `PRODUCT_MECHANICS_001_CANONICAL_EDIT_MODEL.md`
2. `PRODUCT_MECHANICS_002_MANUAL_EDITOR_BASELINE.md`
3. `PRODUCT_MECHANICS_003_SCHEMATIC_PCB_AUTHORITY.md`
4. `PRODUCT_MECHANICS_004_AI_TOOLING_CONTRACT.md`
5. `PRODUCT_MECHANICS_005_EMBEDDED_TERMINAL.md`
6. `PRODUCT_MECHANICS_006_ASSISTANT_SURFACE.md`
7. `PRODUCT_MECHANICS_007_PROJECT_WORKSPACE_MODEL.md`
8. `PRODUCT_MECHANICS_008_LIBRARY_COMPONENT_SYSTEM.md`
9. `PRODUCT_MECHANICS_009_RULES_CONSTRAINTS_CHECKS.md`
10. `PRODUCT_MECHANICS_010_INDUSTRY_STANDARDS_COMPLIANCE.md`
11. `PRODUCT_MECHANICS_011_IMPORT_INTEROP_ROLE.md`
12. `PRODUCT_MECHANICS_012_APPLICATION_QUALITY_BAR.md`

## Draft Decision For Item 1

This is the proposed starting point for review:

Datum shall use a canonical edit model where:
- all design mutations are typed operations
- direct manual edits commit as transactions
- automation, AI, ECO, import repair, DRC corrections, and routing/planning
  produce proposals by default
- accepted proposals commit as transactions
- every transaction carries provenance, changed objects, and undo/redo data
- every proposal carries rationale, diff/preview, affected objects, check
  state, and apply/reject/defer status
- no GUI, CLI, MCP, assistant, terminal, script, or importer may bypass this
  model for design-state mutations

Open owner questions:
- Are CLI commands allowed to commit directly by default, or should they have
  preview/proposal mode as the default?
- Which manual GUI edits require review instead of direct commit?
- Should imports be represented as one transaction, many transactions, or a
  source-load event plus native-conversion proposals?
- What minimum diff format is acceptable for geometry changes?
- Should transaction history be persisted in the project from the first slice,
  or initially be session-only?

## References

External:
- Altium Designer overview: https://www.altium.com/altium-designer/overview
- Altium public background: https://en.wikipedia.org/wiki/Altium_Designer
- OrCAD public background: https://en.wikipedia.org/wiki/OrCAD
- PCBSchemaGen: https://arxiv.org/abs/2602.00510
- SchGen: https://arxiv.org/abs/2605.30345
- SmartonAI / New Interaction Paradigm for Complex EDA Software Leveraging
  GPT: https://arxiv.org/abs/2307.14740

Local:
- `docs/DATUM_PRODUCT_MECHANICS.md`
- `docs/DATUM_PRODUCT_MECHANICS_REVIEW_AGENDA.md`
- `docs/NORTH_STAR_PROJECT_AUDIT.md`
- `docs/ALTIUM_LESSONS.md`
- `docs/AUTHORING_TOOLS.md`
- `docs/LIBRARY_ARCHITECTURE.md`
- `docs/IPC_FOOTPRINT_SYSTEM.md`
- `specs/CHECKING_ARCHITECTURE_SPEC.md`
- `specs/STANDARDS_COMPLIANCE_SPEC.md`
