# Datum Product Mechanics

Status: controlling product doctrine (ratified).
Last reconciled: 2026-07-02

Companion review agenda:
- `docs/audits/scope-integration/DATUM_PRODUCT_MECHANICS_REVIEW_AGENDA.md` breaks unresolved mechanics
  questions into one-by-one review tracks.

Companion research synthesis:
- `docs/audits/scope-integration/DATUM_PRODUCT_MECHANICS_RESEARCH_SYNTHESIS.md` promotes the
  product-mechanics research into tracked conclusions that drive decision
  records.

Companion documentation goals:
- `docs/audits/scope-integration/DATUM_PRODUCT_MECHANICS_DOCUMENTATION_GOALS.md` defines the staged
  documentation work needed before new roadmap or feature implementation work
  resumes.

Controlling North Star:
- `docs/decisions/PRODUCT_MECHANICS_016_PRODUCT_NORTH_STAR.md` ratifies the
  product identity and priority order: governed native library -> schematic
  capture -> schematic-to-PCB implementation -> PCB layout -> manufacturing ->
  interop.

## Purpose

This document defines how Datum should function as an application before
roadmap or milestone work continues.

It is not an implementation spec and not a new milestone. It is the missing
product-mechanics layer between the product north star and the codebase.

## Product Definition

Datum is a professional EDA application with optional first-class AI
collaboration.

Datum is not:
- a KiCad importer
- a board viewer
- an AI-only design agent
- a visualization shell around imported EDA files

Import/export are interoperability infrastructure. They help users migrate
projects, inspect existing work, and exchange artifacts with other tools. They
must not define the product identity.

The sole active development focus is native authoring: drawing a schematic,
laying out a PCB, and generating CAM output — natively, with full AI
augmentation. KiCad import is frozen; the M7 spike already imports a board with
enough fidelity to recognize all design aspects, which is sufficient for now.
See "Interop Boundary And Import Posture" for the full posture.

The next product-driving foundation is native governed library data and
schematic capture. PCB layout and imported-board review are downstream or
supporting surfaces until the library/schematic foundation is viable.

## Manual-First Rule

Datum must be fully usable without AI.

Every core EDA workflow must be possible through normal user interaction:
schematic capture, PCB layout, library work, rules/checks, manufacturing
outputs, project navigation, and editing.

AI may assist, accelerate, explain, propose, automate, or review, but AI must
not be required for the user to complete a design.

Required rule:

> If a user cannot perform a core workflow manually, the workflow is not
> complete, even if an AI agent can perform or fake it.

## Optional AI Collaboration

AI is a first-class collaborator, not a hidden authority.

AI actions must map to the same deterministic primitives available to the user.
An AI must not have private editing powers that cannot be inspected, replayed,
or manually reproduced.

The primary AI integration path should be a real terminal plus Datum's
machine-readable project state, CLI, MCP, operation/proposal model, checks, and
artifacts. A user should be able to launch a code agent or EDA assistant from
inside Datum by typing a normal command such as `codex`, `claude`, `aider`, or
another installed tool into the embedded terminal.

"AI-enabled" in Datum means the application is structured so external agents
and optional built-in assistants can inspect and operate the design safely. It
does not mean a required chat tab, hidden model backend, or agent-only workflow.

The terminal is necessary, but it is not sufficient. Datum's AI integration
should feel intentional because the application exposes EDA-native tools,
context, transactions, proposals, checks, and artifacts for agents to use. A
terminal running a generic code agent beside an unrelated EDA app is not the
target. The target is a professional EDA application whose internal mechanics
are deliberately designed so manual users, scripts, external agents, and any
optional built-in assistant all operate through the same inspectable design
primitives.

AI tooling should therefore include:
- project-aware context discovery
- stable object IDs and selection context
- structured schematic and PCB queries
- deterministic edit/proposal APIs
- transaction diffs and apply/reject semantics
- ERC/DRC/check execution with machine-readable findings
- artifact generation and validation surfaces
- provenance for user, tool, import, script, and AI-originated changes

This is what makes AI support native to Datum: not a chat pane, but a complete
tooling substrate built into the EDA application.

AI behavior should follow this model:

1. Inspect current project state.
2. Ask for clarification when intent is ambiguous.
3. Propose bounded edits or analysis steps.
4. Explain the proposal.
5. Run relevant checks when applicable.
6. Present a preview/diff.
7. Apply only after explicit user approval unless the user has configured a
   safe automation policy.

The user must remain able to inspect and reject AI work at every meaningful
boundary.

## Workflow-Neutral Foundation

Datum must not hard-code one universal design workflow.

Different designs and different engineers use different processes:
- simple boards may go directly from schematic to PCB
- analog circuits may involve math, hand calculation, schematic iteration, and
  optional simulation
- digital designs may emphasize architecture, buses, parts, constraints, and
  signal integrity
- power designs may emphasize thermal/current calculations, magnetics, safety
  margins, and layout constraints
- RF designs may move between schematic and layout earlier because parasitics
  matter
- reverse engineering may begin from an imported board and reconstruct a
  schematic later
- library work may begin from symbols, footprints, parts, or manufacturer data

Datum should provide primitives that compose across these workflows rather than
forcing a single sequence.

## Core Primitives

Datum should be built around durable EDA primitives:

- `Project`: the design container, file layout, metadata, settings, and
  workspace state.
- `DesignModel`: the canonical project truth containing electrical intent,
  physical implementation, library bindings, rules, checks, proposals,
  transactions, artifacts, and provenance.
- `Projection`: a task-oriented lens over the design model, such as
  electrical/schematic, physical PCB layout, symbol, footprint, library,
  rules/checks, manufacturing, BOM, or analysis.
- `EditorContext`: the active tools, snapping, selection behavior, overlays,
  inspector sections, and commands for editing one projection.
- `RelatedContext`: cross-domain context shown through inspector, overlays,
  navigation, or query surfaces without forcing a permanent split-screen
  workflow.
- `Viewport`: a saved visual framing of a projection for review,
  documentation, assembly drawings, fabrication drawings, comparison, or
  output pages.
- `ViewTab`: a live visual or tool session over the design model.
- `Pane`: a UI container that holds one or more tabs.
- `LayoutGraph`: the current focused, split, tiled, floating, PiP, pinned, or
  overlay arrangement of panes and tabs.
- `WorkbenchProfile`: a saved arrangement of tabs, panes, toolbars,
  inspectors, and overlays for a task.
- `ManufacturingProjection`: a live production view such as Gerber, NC drill,
  soldermask, paste, BOM/PnP, panel, assembly drawing, or output-job preview.
- `PanelProjection`: a manufacturing-stage arrangement of board instances,
  rails, tabs, mouse bites, V-scores, fiducials, tooling holes, coupons, and
  panel notes.
- `NavigationStack`: user movement through projections and related objects
  without losing place.
- `ElectricalIntent`: authored electrical design data inside the design model.
- `PhysicalImplementation`: physical layout and manufacturing geometry inside
  the design model.
- `LibraryPart`: reusable component identity and metadata.
- `Symbol`: schematic representation of a component or logical unit.
- `Footprint`: physical land pattern and mechanical/process geometry.
- `Net`: electrical connectivity derived from authored design state.
- `Constraint`: design intent or requirement that influences checks and tools.
- `Rule`: executable or declarative condition used by ERC, DRC, or other
  validators.
- `Check`: deterministic validation result with severity, location, and
  explanation.
- `Analysis`: optional calculation, simulation, measurement, or derived report.
- `Proposal`: uncommitted suggested change from a tool, AI, or user workflow.
- `Transaction`: committed design change with undo/redo, provenance, and diff.
- `Artifact`: generated output such as Gerber, drill, BOM, PnP, reports, or
  interchange files.
- `ImportProvenance`: source-format metadata retained when data enters Datum
  from another EDA tool.

These primitives should be available to both manual tools and AI-assisted
tools.

## Unified Design Model

Datum should not assume that schematic and PCB must be segregated into
separate authorities or application silos.

For normal design creation, electrical intent usually informs physical
implementation. For reverse engineering, imported-board recovery, or
layout-driven domains, physical implementation may exist before complete
electrical intent. Datum must represent both without pretending they are the
same thing.

Required rule:

> Datum has one canonical `DesignModel`. Schematic, PCB, library, rules,
> manufacturing, and analysis surfaces are projections over that model, not
> separate authorities.

The schematic projection presents electrical intent. The PCB projection
presents physical implementation. Symbol, footprint, library, rules,
manufacturing, BOM, and analysis projections present other task-oriented views
of the same model.

The application may present projections as full-screen contexts, tabs, split
views, panels, or saved viewports. The UI arrangement is not the authority
model.

Datum should avoid forcing users into permanent split-screen management as the
primary way to understand relationships. Related electrical, physical, library,
rule, check, standards, and manufacturing context should be available through
inspectors, overlays, navigation, and optional comparison viewports.

## Electrical And Physical Relationship

Required rule:

> Physical edits must not silently redefine electrical intent. Electrical edits
> must not silently destroy physical implementation. Relationships and
> deviations must be explicit.

Relationship vocabulary is split across authored bindings, derived resolver
status, authored intent/deviation records, and derived variant population:

- Authored `RelationshipKind` bindings record the intended cross-domain link,
  such as component-to-package, pin-to-pad, net-to-copper, scope-to-board,
  interconnect, or panel/fixture usage.
- Derived resolver status is computed from the active model and variant:
  `Implemented`, `PendingImplementation`, or `UnresolvedMismatch`. These are
  not authored facts.
- Authored intent/deviation records explain intentional divergence:
  `LayoutDeviation`, `BoardOnlyObject`, `SchematicOnlyObject`, and accepted
  deviation rationale.
- Board-first or imported physical-first work uses `RelationshipKind::
  ReverseEngineered`; extra source evidence belongs in import/provenance
  metadata, not in a second `ReverseEngineered` intent axis.
- Derived variant population adds `NotApplicableForVariant` when the active
  variant excludes an option link, scope, or object from consideration. It is a
  resolved population value, not a stored relationship fact.

## Transactions And Proposals

All meaningful edits should pass through an explicit mutation model.

A transaction is a committed change. A proposal is an uncommitted candidate
change.

Transactions should provide:
- stable identity
- provenance: user, AI, import, tool, script, or automation policy
- changed objects
- deterministic diff
- validation state where relevant
- undo/redo support
- replay or audit support where practical

Proposals should provide:
- what will change
- why it is being proposed
- what checks were run
- risks or unresolved assumptions
- preview/diff
- apply/reject/defer actions

Manual tools and AI tools should both use this same model.

## Editor Competency Baseline

Datum is not a complete EDA application until users can manually perform the
basic editor workflows.

Minimum schematic editor competency:
- create/open project
- create sheets
- place/move/rotate/delete symbols
- draw/delete wires
- place labels, ports, junctions, no-connects, and power symbols
- edit symbol fields
- run ERC
- inspect and resolve connectivity/check findings

Minimum PCB editor competency:
- create/open PCB from native project state
- place/move/rotate/delete components
- draw/delete tracks
- place/delete vias
- create/edit/delete zones
- define/edit outline and stackup
- edit object properties in an inspector
- run DRC
- inspect and resolve physical/check findings

Minimum library competency:
- create/edit symbols
- create/edit footprints
- bind symbols, footprints, parts, and package metadata
- preserve source/provenance for imported library data

Minimum project competency:
- save/load native project state
- undo/redo edits
- run checks
- export manufacturing artifacts
- navigate design objects and project files

## Embedded Terminal

Datum's terminal must be a real embedded terminal, not a fake command lane.

It should feel like a native terminal application such as Ghostty, Konsole, or
similar professional Linux terminals.

Required behavior:
- spawn the user's real shell, such as `$SHELL`
- support normal shell commands: `cd`, `ls`, `cp`, `git`, `cargo`, `python`,
  and arbitrary user tools
- preserve working directory per terminal session
- support environment inheritance plus Datum-specific environment variables
- support ANSI/VT escape handling, colors, cursor movement, scrollback,
  selection, copy, paste, and resize behavior
- support long-running commands
- support launching external code agents or AI tools from inside Datum
- avoid artificial restriction to Datum-only commands

Product rule:

> Datum's terminal is a native shell surface. It is not an AI action proxy and
> not a limited command palette.

The terminal tab model should follow terminal-emulator conventions:
- terminal tabs are shell sessions
- each tab owns a PTY-backed process/session
- a user can run agents, shells, scripts, compilers, CLI tools, or Datum tools
  inside those terminal sessions
- the terminal does not need a special "agent tab" to run agents

The terminal should start in a useful project context when launched from a
Datum project, with Datum-specific environment variables available where
appropriate. For example, a terminal-launched agent should be able to discover
the active project root, run Datum CLI commands, talk to Datum MCP surfaces if
configured, inspect files, and use normal Linux tooling without leaving Datum.

The assistant may help a user compose or understand terminal commands, but the
terminal itself remains a normal user-controlled shell.

## Assistant Surface

The assistant surface, if present, is separate from the terminal and secondary
to the real terminal/CLI/tooling model.

The assistant should understand the current project context, selected objects,
active checks, open files, and recent transactions. It should be able to
propose actions, explain state, and help operate Datum. It should not replace
manual tools.

The assistant must never be the only path to a core capability.

AI-generated edits should be inspectable as proposals or transactions using
the same primitives as manual edits.

The assistant surface should not be modeled as a terminal tab unless it behaves
as a terminal. A `Terminal | Assistant` UI can be acceptable only if the two
surfaces are clearly different:
- Terminal: real PTY shell sessions.
- Assistant: optional Datum-aware conversation/review/proposal surface.

If this distinction is not clear, the assistant surface should be deferred
rather than weakening the terminal model.

## Projections, Viewports, And Workspaces

Datum should support distinct projections over one design model:

- Project projection: files, design tree, libraries, settings, outputs.
- Electrical/schematic projection: circuit capture and electrical intent.
- Physical PCB projection: physical layout and manufacturing geometry.
- Symbol projection: reusable schematic representation.
- Footprint projection: reusable land pattern and process geometry.
- Library projection: parts, packages, pin-pad maps, models, metadata.
- Rules/checks projection: constraints, ERC, DRC, waivers, deviations,
  reports.
- Analysis projection: optional calculations, measurements, simulation, and
  design notes.
- Manufacturing projection: BOM, PnP, Gerber, drill, assembly, and validation.

Datum may present these projections through workspaces, tabs, panels, saved
viewports, or output pages. Workspaces are UI arrangements, not separate data
authorities.

Terminal and assistant surfaces are application workspaces that operate with
project context. They are not design projections and must use Datum's
operation/proposal/transaction model for design mutations.

## View Composition And Live Production

Datum should treat tabs as composable view/tool containers over the design
model.

Any projection or tool session may be opened as a tab and arranged as focused,
split vertical, split horizontal, tiled, floating, picture-in-picture, pinned
sidecar, overlay, or saved workbench layout.

Tabs are not data authorities. Closing a tab closes a view or tool session, not
the design data.

Datum should treat fabrication and assembly outputs as live production
projections:
- Gerber layer projection
- NC drill projection
- soldermask projection
- paste projection
- fabrication drawing projection
- assembly drawing projection
- BOM projection
- pick-and-place projection
- panel projection
- output job projection

Exported files are artifacts: versioned snapshots of a production projection at
a specific model revision, board or panel, variant, and output job.

Panelization belongs to the manufacturing projection, not the source board
layout. A panel should reference board instances with transforms and add
production-only features such as rails, breakaway tabs, mouse bites, V-score
lines, fiducials, tooling holes, coupons, route paths, labels, and panel notes.

Required rule:

> Panelization must not mutate source board geometry unless the user
> explicitly requests a board-level change.

## Interop Boundary And Import Posture

Datum EDA is an AI-augmented native EDA design tool. Its job is to draw a
schematic, lay out a PCB, and generate CAM output — natively, with full AI
augmentation. It is not a KiCad import viewer. The old development spec drifted
into treating KiCad import as the primary goal; that drift turned Datum into an
import viewer and is the root cause this product-mechanics correction exists to
undo.

Import is frozen. The M7 spike already imports a KiCad PCB with enough fidelity
to recognize all aspects of a board design; that is sufficient. No further
import work happens until native authoring (schematic -> PCB -> CAM) is real and
capable.

Native is the only authority. The native model and native file format are
always the source of truth. Import is a one-time converter: a KiCad file becomes
native data, and once conversion completes the result is just a native board.
There is no "imported board" state — origin is retained only as provenance
metadata. Export (native -> KiCad) is a separate, optional, deferred converter.

Import fidelity does not gate native maturity. Native readiness is judged solely
by native capability: whether a user and an AI can author, edit, check, and
output a schematic and board. The KiCad importer's quality is never a gate or
milestone for the native interface or the native file format.

Import still must not fabricate. Conversion preserves provenance and must not
silently heal, infer, or invent data the source did not contain; genuine gaps
(such as missing electrical intent on a recovered board) are marked with
explicit relationship states such as `ReverseEngineered` or `BoardOnly`, not
filled in silently.

Resource directive. All development focus goes to native schematic + PCB + CAM
authoring with AI augmentation. Anything framed around import-as-a-primary-goal,
"imported vs native" authority, dual-write, or authority-flip is drift from the
old spec and is removed. This supersedes the earlier "authority flip after M7 +
dual-write window" framing in the 000-series decisions.

## Code-Agent Guidance

Any code agent working on Datum should apply these rules before choosing work:

1. Do not infer product identity from the current active milestone.
2. Prefer work that strengthens native EDA primitives and manual workflows.
3. Treat AI as optional augmentation, not as a replacement for manual editor
   capability.
4. Do not expand import fidelity unless it supports a clear product need.
5. Do not create a new editing path that bypasses the canonical transaction
   model.
6. Do not treat a viewer or review surface as equivalent to an editor.
7. Do not implement terminal features as fake command text unless they are on
   the path to a real PTY-backed terminal.
8. Do not treat a built-in assistant tab as the primary AI architecture. The
   primary architecture is Datum primitives plus real terminal/CLI/MCP access
   that lets users run whichever agents or tools they choose.

## Open Review Questions

These questions should be resolved with the project owner before roadmap
rewrites or major new implementation:

1. What exact manual workflows must exist before Datum can be called a usable
   EDA editor?
2. Which primitives should become canonical operation/transaction types first?
3. How should existing CLI native-authoring commands migrate into the canonical
   operation model?
4. What is the first GUI editor slice that proves Datum is not just a review
   surface?
5. Should a built-in assistant surface exist before the real terminal and
   CLI/MCP agent workflow are product-real?
6. What terminal implementation quality is required before the terminal can be
   considered product-real?
7. Which current M7/import-fidelity work should be frozen, completed, or
   demoted after this mechanics model is accepted?

## Immediate Implication

No new roadmap milestone should be written until this product-mechanics model
is reviewed and corrected.

After review, the existing specs and roadmap should be audited against this
document so that future implementation is driven by Datum's intended product
mechanics rather than by the nearest concrete import, render, or regression
task.
