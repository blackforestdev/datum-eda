# Product Mechanics Decision 012: Application Quality Bar

Status: draft hypothesis + how/mechanism woven 2026-06-18; quality gates tied
to 000-001 mechanisms.
Date: 2026-06-18

Driven by:
- `docs/DATUM_PRODUCT_MECHANICS.md`
- `docs/audits/scope-integration/DATUM_PRODUCT_MECHANICS_REVIEW_AGENDA.md`
- `docs/audits/scope-integration/DATUM_PRODUCT_MECHANICS_DOCUMENTATION_GOALS.md`
- `docs/decisions/PRODUCT_MECHANICS_000_UNIFIED_DESIGN_MODEL.md`
- `docs/decisions/PRODUCT_MECHANICS_000B_VIEW_COMPOSITION_AND_LIVE_PRODUCTION.md`
- `docs/decisions/PRODUCT_MECHANICS_001_CANONICAL_EDIT_MODEL.md`
- `docs/decisions/PRODUCT_MECHANICS_002_MANUAL_EDITOR_BASELINE.md`
- `docs/decisions/PRODUCT_MECHANICS_007_PROJECT_WORKSPACE_MODEL.md`

## Decision Scope

Define the concrete application quality bar Datum must satisfy to be credible
as a professional EDA application.

This decision focuses on user-visible product quality: reliability,
performance, editing feel, project safety, view composition, terminal/tooling
integration, checks, manufacturing confidence, and supportability.

## Product Intent

Datum should target an Altium-class professional application quality bar, even
if early feature breadth is smaller.

Altium-class does not mean copying Altium's UI or matching every feature on
day one. It means Datum must feel like serious engineering software: stable,
fast, predictable, inspectable, recoverable, and safe enough for real board
work.

Required product rule:

> A narrow workflow implemented to professional quality is better than a broad
> workflow that feels like a demo, viewer, or AI wrapper.

## Decision

Datum shall treat application quality as a product mechanic, not polish added
after features exist.

Every baseline workflow must meet concrete quality criteria before it is
presented as product-ready:
- manual editing must be direct, precise, and undoable
- project state must be durable and recoverable
- checks must be deterministic and navigable
- generated artifacts must be tied to model revisions and settings
- workspace composition must be stable across tabs, panes, PiP, tile,
  floating windows, and workbench profiles
- optional AI/tooling must operate through the same inspectable primitives as
  manual users

Quality is accepted by gates, not by demos. A workflow is not product-ready
until its source mutations go through the operation/proposal/transaction model,
its committed state survives forced failure through the journaled `commit()`
path, its undo/redo state is durable, its generated artifacts are traceable to
exact revisions and settings, and its interaction latency stays within the
budget for the target fixture.

## User-Visible Behavior

Users should see:
- fast project open, save, close, and recovery behavior
- no unexplained data loss after crashes or failed operations
- consistent selection, snapping, dragging, routing, and property editing
- responsive canvas interaction on realistic projects
- predictable undo/redo across schematic, PCB, library, rules, and
  manufacturing edits
- clear modified/dirty/check/artifact status
- clear distinction between source model, live projections, and generated
  artifacts
- actionable ERC, DRC, and manufacturing findings
- stable tab/pane/workbench restoration
- terminal and external tooling that behave like real project tools, not fake
  demos

The application should make risky state visible before the user ships a board.

## Manual Workflow Requirements

Manual quality requirements:
- common actions must be discoverable through menus, command palette, toolbar,
  shortcut, and context menu where appropriate
- canvas tools must provide visible pre-edit preview, snap feedback, and
  constraint feedback
- property edits must validate before commit and explain invalid values
- direct edits must commit through transactions and be undoable
- high-risk edits must become proposals or explicit confirmation flows
- selection and object inspection must remain coherent across projections
- check findings must navigate to exact affected objects through stable object
  IDs and `ComponentInstance`, not reference designators, names, or paths
- manufacturing previews must cross-highlight with source objects where
  applicable, resolving the electrical-to-physical join through
  `ComponentInstance` and stable object IDs
- keyboard-driven workflows must be practical for frequent commands
- workspace layouts must not corrupt or hide source state

Early releases may have fewer tools, but the tools that exist must be reliable
enough for manual board work.

## Optional AI And Tooling Behavior

AI and tooling quality requirements:
- agents inspect project state through stable APIs, CLI, MCP, files, or
  terminal context
- AI-originated changes are proposals by default
- proposal diffs are inspectable before apply
- accepted proposals become transactions with provenance
- rejected proposals leave no partial design mutation
- terminal sessions are real PTY-backed shells running the user's shell, and
  terminal-launched tools use the active project cwd and environment context
- tool failures are surfaced with actionable error messages
- automation cannot create hidden waivers, deviations, artifacts, or geometry

Gate `QG-PTY-REAL-TERMINAL`:
- a terminal session allocates a real PTY and runs the user's shell; it is not
  a fake demo surface
- terminal text is not an edit API; Datum never derives design mutations by
  parsing terminal output or input
- any design mutation launched from a terminal-resident tool reduces to typed
  operations and goes through proposals or journaled transactions on the single
  `commit()` path, exactly like every other mutation surface
- the pass/fail hook is that no terminal-originated design change reaches the
  resolved `DesignModel` except through `commit()` with journaled provenance

AI integration is considered high quality only when it increases confidence in
the design process without weakening manual control or auditability.

## Core Primitives

Quality requirements apply to:
- `Project`
- `Workspace`
- `DesignModel`
- `Projection`
- `EditorContext`
- `Operation`
- `Proposal`
- `Transaction`
- `Diff`
- `Provenance`
- `Rule`
- `Check`
- `ManufacturingProjection`
- `Artifact`
- `ViewTab`
- `Pane`
- `LayoutGraph`
- `WorkbenchProfile`
- `NavigationStack`

## Concrete Quality Bar

### Reliability And Safety

Datum must provide:
- crash-safe project persistence for committed transactions through the
  staged-bytes + journal-append commit point + atomic-rename mechanism
- atomic save or equivalent corruption-resistant write behavior for every
  source shard touched by an operation batch
- recovery path for interrupted saves, interrupted imports, interrupted
  proposal acceptance, and interrupted artifact metadata writes
- clear handling of invalid, missing, or incompatible project data
- a split, incoherent, or version-incompatible project opens in
  resolver-driven recovery or diagnostic mode, not as accepted truth
- no silent mutation outside the canonical edit model
- no hidden standards waivers or manufacturing setting changes

Every reliability gate operates over the resolved `DesignModel` assembled by
the engine-owned `ProjectResolver` from source shards, not over any single
shard, layout graph, or generated file. Crash recovery and workspace restore
roll the resolved model and its journal forward; they never treat an on-disk
shard as a standalone authority.

Gate `QG-CRASH-RECOVERY`:
- force termination before journal append leaves no committed source mutation
  after reopen
- force termination after journal append but before shard promotion rolls
  forward or opens resolver recovery mode with the journaled transaction
  identified against the resolved `DesignModel`
- corrupted or missing staged bytes do not produce a half-new, half-old
  `DesignModel`
- failed import/open conversion leaves either the previous committed project
  state or an explicit recovery project, never silent partial authority
- recovery logs identify transaction ID, touched shards, before/after hashes,
  and the recovery action taken

Gate `QG-RESOLVER-RECOVERY`:
- the `ProjectResolver` validates shard versions, object references,
  relationships, library revisions, rule scopes, output contexts,
  proposal/transaction references, cache staleness, and artifact metadata at
  open
- a project that fails coherence (split authority, missing or incompatible
  shards, dangling references) opens in recovery or diagnostic mode and is
  never presented as accepted truth
- the pass/fail hook is whether the resolver produces a single coherent
  resolved `DesignModel` with a defined `model_revision`, or an explicit
  recovery/diagnostic state identifying the failing shards and references
- recovery never silently fabricates authority; any repair is a proposal or
  journaled transaction through `commit()`

### Performance And Responsiveness

Datum should define performance budgets before broad implementation.

Initial expected bar:
- startup and project open feel interactive on small projects
- canvas pan/zoom remains smooth on representative small and medium boards
- selection and inspector update without visible stalls for ordinary objects
- checks may run asynchronously but must report progress and stale-state
  status
- live manufacturing projections may invalidate incrementally or show stale
  status rather than freezing editing

Gate `QG-PERFORMANCE-LATENCY`:
- numeric budgets are recorded per representative fixture before a workflow is
  called product-ready
- direct-manipulation preview latency is measured separately from committed
  transaction latency
- commit latency is measured for the operation families in Decision 002 and
  includes validation, journal append, shard promotion, revision updates, and
  projection invalidation
- projection regeneration reports stale/current state and must not block
  canvas interaction while running asynchronously
- regressions are caught by repeatable fixtures rather than by subjective UI
  impressions

Initial numeric budgets remain owner-defined once fixtures exist. Until then,
performance work may be marked experimental, not product-ready.

### Editing Feel

Professional editing requires:
- precise hit testing
- predictable snapping
- visible tool state
- low-latency drag previews
- command cancellation
- selection filters
- copy/paste semantics
- keyboard shortcuts
- inspector/property editing
- consistent undo/redo
- no modal dead ends for routine work

Gate `QG-DIRECT-EDITING-FEEL`:
- placement, movement, routing, selection, and property edits provide visible
  preview before commit
- local manual actions that qualify for direct commit do not require proposal
  approval or modal review
- pointer/key cancellation leaves no committed mutation and no dirty source
  shard
- inspector edits validate before commit and report invalid values at the
  edited field
- committed edits are immediately reflected in source projection, related
  projection status, transaction history, and undo availability
- hit testing, snapping, and selection filters are deterministic enough for
  regression fixtures

Direct editing feel is compatible with the canonical edit model: preview state
is session state, and the final accepted gesture becomes an operation batch
committed through the same journaled path as every other mutation.

### Durable Undo And Redo

Undo/redo quality is part of project safety, not a volatile UI convenience.

Gate `QG-DURABLE-UNDO`:
- every direct manual edit in the proof slice creates inverse data or an
  equivalent compensating operation in the transaction journal
- undo and redo survive close/reopen because they are journal cursors, not
  in-memory stacks
- undo is itself recorded as a compensating transaction so history remains
  append-only and auditable
- undo/redo across electrical, physical, rules, library, manufacturing, and
  workspace-adjacent source edits use the same mechanism
- generated projections and check state become stale/current according to
  revisions after undo/redo, not according to ad-hoc dirty flags

### Project And Workspace Quality

Project/workspace quality requires:
- reliable recent-project and project-root handling
- clear dirty state and save state
- clear model revision and artifact revision indicators
- stable tab restore
- stable split/tile/floating/PiP behavior
- saved workbench profiles that do not alter source authority
- multi-monitor behavior that does not lose windows or project context

Workspace persistence is quality-gated separately from source persistence.
Saving or restoring tabs, panes, PiP views, floating windows, sidecars,
camera positions, and workbench profiles must not advance `model_revision` or
rewrite source shards. A workspace restore can fail partially and still open
the project because source authority lives in the resolved `DesignModel` and
journal, not in the layout graph.

### Checks And Manufacturing Confidence

Datum must make sign-off risk visible:
- ERC/DRC findings include severity, affected objects, rule/check ID, and
  explanation
- manufacturing previews distinguish source objects from generated production
  geometry
- a zone boundary polygon is never treated as exportable copper; the
  `Zone.polygon` is the authored boundary, and only a real `ZoneFill{Filled}`
  contributes copper to projection or export
- artifacts record source model revision, settings, tool version, and check
  state
- output jobs expose completeness and stale/generated status
- waivers and deviations are explicit, reviewable model objects

Gate `QG-ZONEFILL-HONESTY`:
- an authored `Zone.polygon` boundary is never exportable copper; it carries
  no production geometry by itself
- only `ZoneFill{Filled}` contributes copper to projections and exports, with
  provenance carried by the derived fill state
- `ZoneFill{Unfilled|Stale|Unsupported}` renders as such, exports no copper,
  and emits a hard finding rather than silently shipping a boundary as fill
- fill freshness keys from `model_revision`, so a stale fill is reported stale
  against the resolved model rather than treated as current copper

Gate `QG-GENERATED-ARTIFACT-TRACEABILITY`:
- every generated artifact records source `model_revision`, source board or
  panel ID/revision, variant revision where applicable, output-job revision,
  manufacturing-plan revision where applicable, generator version, timestamp,
  validation/check state, and content hash or generated path
- live projections and exported artifacts share the same T0 generation oracle
  or record a classified equivalence divergence
- artifact metadata is source state committed through transactions when it is
  part of the project record; generated files remain snapshots, not authority
- comparing a live projection to the last artifact reports current, stale,
  changed settings, changed model revision, changed generator version, or
  unsupported rather than a generic mismatch
- deleting or moving an artifact file does not delete design source state, and
  stale/missing artifact status is visible before output sign-off

### Supportability

Professional quality also requires diagnostic support:
- structured logs for project open/save, checks, artifact generation, and
  tool/agent activity
- reproducible error reports with project revision and command context where
  possible
- debug queries or test fixtures that can inspect transactions, diffs,
  provenance, and check state
- deterministic behavior suitable for regression tests

Diagnostics are part of each quality gate. A failed gate must leave enough
structured evidence to identify the project revision, transaction/proposal,
operation family, projection/artifact key, and workspace/session context that
failed.

## Standards And Compliance Impact

Application quality directly affects compliance credibility.

Datum must not allow compliance-relevant state to be hidden in UI-only
settings, stale generated files, undocumented agent edits, or untracked
terminal scripts.

Minimum compliance-support behavior:
- standards/process basis is visible at project level
- rule/check state is inspectable and versioned with the design model
- findings, waivers, and deviations are explicit and navigable
- manufacturing artifacts include source revision and generation settings
- audit/provenance exists for manual, import, script, tool, and AI-originated
  changes

## Explicit Non-Goals

This decision does not require:
- matching every Altium feature
- copying Altium's interface
- guaranteeing formal regulatory certification
- solving enterprise PLM, vault, or lifecycle workflows immediately
- perfect performance on very large designs before small/medium workflows are
  credible
- AI features as a substitute for application quality
- treating prototype UI surfaces as production-ready

## First Proof Slice

The first proof slice should demonstrate a narrow but professional path:
- create or open a project
- manually edit electrical and physical objects
- edit at least one rule
- run checks with navigable findings
- preview at least one manufacturing projection
- generate an artifact snapshot tied to model revision and settings
- close and reopen the project with source state, transactions, and workspace
  profile intact
- recover cleanly from a forced failure in save, check, or artifact generation
- show transaction/provenance/debug state for manual and tool-created changes

Gate coverage required for the proof:
- `QG-CRASH-RECOVERY` passes for one source edit and one artifact metadata
  write
- `QG-RESOLVER-RECOVERY` passes for one coherent open and one deliberately
  split/incoherent project that opens in recovery/diagnostic mode
- `QG-ZONEFILL-HONESTY` passes for one filled zone exporting copper and one
  unfilled/stale zone exporting no copper plus a hard finding
- `QG-PTY-REAL-TERMINAL` passes for one terminal-launched tool whose design
  mutation reaches the model only through a proposal/transaction `commit()`
- `QG-DURABLE-UNDO` passes across close/reopen for at least one electrical edit
  and one physical edit
- `QG-DIRECT-EDITING-FEEL` passes for one placement/move gesture, one routing
  gesture, and one inspector/property edit
- `QG-GENERATED-ARTIFACT-TRACEABILITY` passes for one generated production
  artifact
- `QG-PERFORMANCE-LATENCY` has fixture-backed budgets and measured results for
  open, pan/zoom, selection, commit, and projection refresh, even if the owner
  later changes the exact thresholds

## Open Owner Questions

1. What exact board size and complexity should define the first performance
   fixture?
2. Which quality metrics need hard numeric budgets before implementation:
   startup, open, save, pan/zoom, selection, DRC, artifact generation, or
   workspace restore?
3. What crash/recovery behavior is mandatory before users are allowed to edit
   real projects?
4. Which workflows can be labeled experimental without violating the product
   quality bar?
5. What is the minimum diagnostic bundle needed for support and regression
   testing?
6. What are the first numeric latency budgets for pointer preview, commit,
   selection/inspector update, project open, workspace restore, and projection
   refresh?
7. Which artifact formats must satisfy traceability before manufacturing
   output is considered product-ready?
8. Which forced-failure scenarios must run in CI versus manual fault-injection
   tests?
