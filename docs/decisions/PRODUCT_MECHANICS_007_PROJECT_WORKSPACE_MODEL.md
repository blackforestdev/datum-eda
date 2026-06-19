# Product Mechanics Decision 007: Project Workspace Model

Status: draft hypothesis + how/mechanism woven 2026-06-18; aligned with
000-001 source/workspace authority split.
Date: 2026-06-18

Driven by:
- `docs/DATUM_PRODUCT_MECHANICS.md`
- `docs/audits/scope-integration/DATUM_PRODUCT_MECHANICS_REVIEW_AGENDA.md`
- `docs/audits/scope-integration/DATUM_PRODUCT_MECHANICS_DOCUMENTATION_GOALS.md`
- `docs/decisions/PRODUCT_MECHANICS_000_UNIFIED_DESIGN_MODEL.md`
- `docs/decisions/PRODUCT_MECHANICS_000B_VIEW_COMPOSITION_AND_LIVE_PRODUCTION.md`
- `docs/decisions/PRODUCT_MECHANICS_001_CANONICAL_EDIT_MODEL.md`

## Decision Scope

Define the user-facing project and workspace model around Datum's canonical
`DesignModel`.

This decision covers project identity, workspace state, tabs, panes,
picture-in-picture, tiled views, floating views, pinned sidecars, saved
workbench profiles, and the distinction between source authority and UI
composition.

## Product Intent

Datum should feel like a professional workbench, not a collection of isolated
document windows.

The user should be able to organize schematic, PCB, library, rules, checks,
terminal, and manufacturing views around the task at hand while still editing
one canonical project model.

Required product rule:

> Workspace composition is a user interface concern. It must never become the
> source authority model.

## Decision

A Datum `Project` is the user-visible design container. It owns project
metadata, settings, storage layout, design-model identity, artifact identity,
and pointers to workspace state.

A `Workspace` is the current interactive arrangement of project views and tools.
It may contain many tabs, panes, PiP views, tiled layouts, floating views,
pinned sidecars, terminals, inspectors, and saved workbench profiles.

Tabs and panes are live sessions over the project model. Closing, moving,
tiling, or floating a tab must not delete or re-author schematic, PCB, library,
rules, manufacturing, or artifact truth.

The persistence boundary is concrete:
- Source authority state lives in deterministic project shards resolved into
  the canonical `DesignModel`, plus the required transaction journal,
  proposals, artifact metadata, rules, relationships, manufacturing plans, and
  output jobs.
- Workspace state records UI composition and may be stored for convenience,
  but it is not source authority and does not advance `model_revision`.
- Session state records volatile runtime details such as active tool, hover
  object, unsent terminal scrollback, preview geometry, transient selections,
  and in-progress drags. It may be restored opportunistically, but loss of
  session state must never lose committed design data.
- Workbench profiles are reusable layout defaults. Applying one rearranges
  views and tool defaults; it does not mutate design, rules, artifacts, or
  manufacturing source state.

Any project/workspace command that changes design source state must still emit
typed operations and commit through Decision 001. A workspace restore may open
a PCB tab, but it may not replay PCB edits. A terminal tab may expose project
context, but terminal-launched tools do not gain a private file-write path.

## User-Visible Behavior

Users should be able to:
- create, open, save, close, duplicate, and archive projects
- see project name, root path, model revision, active variant, active board or
  manufacturing plan, and check status
- open electrical, physical, symbol, footprint, library, rules, checks,
  manufacturing, output job, artifact preview, terminal, log, and notes tabs
- split tabs vertically or horizontally
- tile tabs into a grid
- pin a related view as a sidecar
- open a PiP view linked to active selection
- float a tab or pane onto another monitor
- save and restore workbench profiles for common tasks
- preserve navigation history across projections and related objects
- restore recent workspace state without treating it as source design data

The workspace should support both focused single-view work and dense
multi-monitor professional review.

## Manual Workflow Requirements

Project and workspace features must support manual EDA work directly:
- a user can start a blank project without AI
- a user can navigate from schematic to PCB to library to manufacturing views
  without command-line knowledge
- a user can arrange views for manual routing, checking, library editing, or
  manufacturing review
- a user can keep live production projections visible while editing source
  layout
- a user can run and inspect checks from normal project UI surfaces
- a user can recover from accidental tab closure without losing design data
- a user can tell which project, variant, revision, and output job they are
  editing

The project UI must make source state, derived views, generated artifacts, and
workspace layout visibly distinct.

## Optional AI And Tooling Behavior

AI and tooling may use workspace context, but they must not depend on it as
the only source of truth.

Agents and tools may:
- inspect active project, selection, open tabs, visible projection, and current
  workbench profile
- launch from an embedded terminal with project cwd and environment context
- open proposal, diff, check, or artifact preview tabs
- suggest a workbench profile for a task
- create bounded proposals against the project model
- reference view context when explaining findings

Agents and tools may not:
- treat a closed tab as deleted design data
- mutate the project by rearranging workspace views
- assume hidden tabs are stale source authorities
- bypass transactions because a terminal is open in the project
- require an assistant pane for normal project navigation or editing

## Core Primitives

### Project

The user-facing container for a Datum design.

Owns:
- project identity and metadata
- root path and storage layout
- active `DesignModel` identity
- project settings
- variants and manufacturing plans
- library bindings
- artifact registry
- check/run history
- workspace references and policy

Source authority fields are persisted in the project manifest and referenced
source shards. The manifest may identify default workspace/profile references,
but it must not duplicate tab layout, camera, or transient view state in a way
that churns design diffs.

### ProjectManifest

The stable project entry point, resolved and assembled into the canonical
`DesignModel` by the engine-owned `ProjectResolver`. The manifest is the
resolver's entry point; there is no separate loader. On open the
`ProjectResolver` validates shard versions, object references, and object/model
revisions, and a split or incoherent project opens in recovery or diagnostic
mode, not as accepted truth. Workspace restore never substitutes for or masks
resolver recovery.

Owns:
- project ID, name, schema version, and storage layout version
- shard index for source authority partitions
- active or default variant, board, manufacturing plan, and output job
  references
- artifact registry references
- default workspace/profile references
- compatibility and recovery metadata

The manifest points to authority shards and optional workspace records. It is
not a serialized `DesignModel`, and it is not a saved desktop snapshot.

### SourceAuthorityState

The persisted source state that participates in `ProjectResolver` output and
`model_revision`.

Includes:
- electrical, physical, library, relationship, rules, manufacturing-plan,
  output-job, proposal, transaction, and artifact-metadata shards
- stable object identities and object revisions
- transaction journal records and recovery metadata

Changing source authority state requires an operation/proposal/transaction.
Workspace commands must not write these shards directly.

### Workspace

The interactive state of a project session.

Owns:
- open tabs
- pane arrangement
- floating windows
- PiP views
- pinned sidecars
- inspector/tool visibility
- active selection and navigation stack
- active workbench profile
- transient terminal sessions where appropriate

Workspace state may be persisted for convenience, but it is not design
authority. Persisted workspace state has its own `workspace_revision` or
timestamp and is keyed by user/profile/project context rather than by
`model_revision`. It may reference source objects by stable object ID for
selection restore, but missing or deleted objects degrade gracefully instead of
resurrecting source state.

### WorkspaceState

The persisted UI composition for a user or project.

Schema concepts:
- `workspace_id`
- `project_id`
- `owner_scope`: project default, user local, or shared profile
- `layout_graph_id`
- open `ViewTab` records
- active `WorkbenchProfile`
- inspector/tool visibility
- selected object references by stable ID
- navigation history references by stable ID and projection context
- last known source `model_revision` for staleness display only

The last known `model_revision` is diagnostic context, not authority. Restoring
a workspace whose recorded revision is stale marks the workspace context stale
and resolves tabs against the current model; it does not roll the project back.

### ViewTab

A live visual or tool session over the design model, a projection, a generated
artifact, or a project tool.

### Pane

A container for one or more tabs.

### LayoutGraph

The current arrangement of panes, tabs, floating windows, overlays, PiP views,
and pinned sidecars.

Schema concepts:
- nodes: pane, split, tile, floating window, overlay, PiP, sidecar
- edges: containment, focus, pin-to-selection, projection-link, monitor/window
  placement
- tab references by `view_tab_id`
- camera/view-state references scoped to projection and object IDs

The layout graph is a UI graph, not a domain graph. It may be saved in
workspace state or a workbench profile, but it must not own design objects.

### WorkbenchProfile

A named arrangement of tabs, panes, toolbars, inspectors, overlays, and task
defaults.

Expected initial profiles:
- schematic capture
- PCB layout
- library authoring
- checks and rules
- manufacturing review
- terminal/automation
- focused single-view
- multi-monitor review

Workbench profiles may be project-defined or user-defined. A profile stores
layout templates and tool defaults, not source shards. Applying a profile may
open an output-job tab by reference; editing the output job itself remains a
normal source transaction.

### SessionState

Volatile runtime state for one running application instance.

May include:
- active tool and tool mode
- hover state
- drag preview geometry
- pending text-entry buffers
- transient terminal process state
- temporary check/projection progress
- unsaved workspace layout edits before persistence

Session state is recoverable convenience only. If Datum crashes, committed
transactions and persisted workspace records are recovered through their own
mechanisms; in-progress previews or partial UI gestures are discarded unless
they had already committed.

### NavigationStack

Cross-projection movement history, including jumps from objects to related
objects, findings, rules, artifacts, and manufacturing projections.

Navigation entries reference stable object IDs, projection kind, board/sheet/
variant/output-job context, and optional view framing. They never duplicate the
target object.

## View Composition Requirements

Datum shall support view composition as defined by the live-production
decision:
- focused main view
- vertical split
- horizontal split
- tiled/grid layout
- floating window
- picture-in-picture
- pinned sidecar
- overlay
- saved workbench layout

Expected examples:
- schematic focused with PCB PiP for selected component
- PCB layout focused with schematic sidecar for selected net
- PCB layout tiled beside live Gerber and NC drill projections
- rules/checks sidecar pinned during routing
- 3D preview floating during placement
- terminal tab beside check output and proposal diff
- manufacturing workbench with output job, Gerber, drill, BOM/PnP, and panel
  tabs tiled

## Standards And Compliance Impact

Project and workspace state must make compliance-relevant context visible:
- active standards/process basis
- active ruleset and rule revision
- current check status
- open findings and waiver/deviation state
- output job completeness
- artifact revision and validation status
- manufacturing plan and panel context

Workbench profiles may expose compliance review layouts, but compliance state
must live in the `DesignModel`, rules/checks domain, manufacturing plan, and
artifact metadata, not in a screen layout.

## Explicit Non-Goals

This decision does not require:
- cloud collaboration
- enterprise project vaults
- real-time multi-user editing
- final on-disk storage format
- final workspace serialization format
- every tab type in the first release
- making saved workspace layouts mandatory
- treating tabs as files
- replacing the canonical edit model with document-window semantics

## First Proof Slice

The first proof slice should demonstrate:
- one project containing one canonical design model
- electrical and physical tabs over the same model
- live manufacturing projection tab over the same model
- split or tiled layout with cross-selection between source and projection
- one PiP or pinned sidecar showing related context
- one saved workbench profile restored across sessions
- source-authority persistence through deterministic shards and the journal,
  with `model_revision` advancing only after source transactions
- any source mutation exercised during the demo flows through the single
  `commit()` path with a journaled `TransactionRecord`, an `object_revision`
  bump, and durable undo, so the slice's source side is explicitly hookable
- workspace persistence through a separate `WorkspaceState`/`LayoutGraph`
  record whose save/restore does not advance `model_revision`
- session-state loss tolerance: after a forced restart, committed design data
  and persisted workspace layout recover, while transient drag/hover/tool
  previews are discarded safely
- clear UI distinction between source model revision, workspace revision,
  generated artifact revision, and stale projection state

## Open Owner Questions

1. Which workspace modes are mandatory first: split, tiled, PiP, floating,
   pinned sidecar, or saved profiles?
2. Should persisted `WorkspaceState` default to project-local shared records,
   user-local sidecar records, or both with explicit owner scope?
3. How much of terminal session state should be restored with a workspace?
4. Should workbench profiles be project-defined, user-defined, or both?
5. What project dashboard/status indicators are required before serious manual
   workflows begin?
6. Which workspace changes should be journaled for audit, if any, even though
   they do not mutate `DesignModel` source state?
7. What is the minimum schema versioning policy for workspace/profile records
   so old layouts degrade safely instead of blocking project open?
