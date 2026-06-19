# Datum AI / CLI / MCP Tool Surface

Status: draft implementation-spec 2026-06-19; derived from ratified
PRODUCT_MECHANICS 000-012.

## Driven by

- `docs/audits/scope-integration/DATUM_PRODUCT_MECHANICS_IMPLEMENTATION_READINESS_AUDIT.md`
  (ratified vocabulary and cross-doc invariants)
- `docs/decisions/PRODUCT_MECHANICS_000_UNIFIED_DESIGN_MODEL.md`
- `docs/decisions/PRODUCT_MECHANICS_000D_STORAGE_AND_VERSIONING_MODEL.md`
- `docs/decisions/PRODUCT_MECHANICS_000F_LIVE_PRODUCTION_PROOF_MODEL.md`
- `docs/decisions/PRODUCT_MECHANICS_001_CANONICAL_EDIT_MODEL.md`
- `docs/decisions/PRODUCT_MECHANICS_004_AI_TOOLING_CONTRACT.md`
  (contract classes; this doc is its implementation-spec realization)

## Purpose & Scope

This is the SHARED SUBSTRATE for every Datum authoring and inspection
surface. It defines, exactly ONCE, the seven shared operations that all
agent, CLI, and UI surfaces use, and the four cross-cutting contracts
(query, proposal/apply, artifact, session) that the five per-domain
tool-contract docs (schematic, PCB, library, rules, manufacturing) build
on top of.

The governing rule of this document: there is **exactly one operation
vocabulary, exactly one `commit()` and journal, exactly one proposal
lifecycle, and exactly one artifact-projection oracle**. CLI and MCP are
isomorphic over these same contract classes with stable JSON schemas;
human-readable summaries are presentation layers only.

A per-domain doc REFERENCES the contracts named here by name and adds
ONLY its domain-specific typed `Operation` variants and `aiQueryContext`.
It never redefines query, selection, run-check, propose, apply, export,
or undo/redo. A per-domain doc that re-mints a session, a revision, a
commit path, a delete verb, or an export tool is a defect.

This doc is implementation-spec, not philosophy. It names the existing
engine surfaces to build on and the gaps to close, and it cites file
paths so an implementer can locate both.

### Substrate prerequisite (hard, blocks all five domains)

The current engine does NOT yet have the substrate this contract
assumes. There is no general `Operation`/`OperationBatch` enum, no single
generic `commit()` with fsync journal, no `DatumToolSession` /
`DatumContextEnvelope`, no `project_id` / `model_revision`, no
`ObjectId` / `object_revision` provenance, no `ComponentInstance`, and no
Import Map `import_key`. Imported-board mutations journal via per-method
`TransactionRecord` (`crates/engine/src/api/write_ops/`), but native
schematic, library, and manufacturing authoring are private JSON writes
(`write_canonical_json`, 115 call sites across the CLI today).

Therefore this document DOCUMENTS THE TARGET. The substrate build
(operation enum + commit + journal + session + identity) is its own
foundational implementation slice and MUST land before any per-domain
authoring tool. The five domain docs reference this substrate as a
precondition; they do not re-describe it.

## Reference-Tool Survey (with the lean rationale)

The shape below is extracted from how production EDA tools expose agent /
batch / scriptable surfaces. The point of the survey is to separate
load-bearing structure from ceremony, and to justify why the lean set is
seven shared operations rather than a per-domain catalog.

- **Altium Designer** (primary reference). Scriptable surface is the
  Altium API plus the OutJob (Output Job) document. The load-bearing
  insight is OutJob: artifact generation is configured fan-out by one
  job document (which Gerbers, which drill, which assembly drawing,
  which BOM), not a separate command per output. Datum copies this:
  artifact SCOPE is a `--include` parameter and panel is a `--context`
  on one `generate` verb. Altium's rule system is also a single query
  language over the unified model, not per-domain rule dialects — Datum's
  `rules.query` / one `check run` mirrors this. Ceremony to omit: Altium's
  proliferation of per-object-class dialog actions; Datum collapses these
  to typed operations on stable IDs.

- **KiCad**. `kicad-cli` exposes `pcb export gerbers|drill|pos`,
  `sch export netlist`, `pcb drc`, `sch erc`, and `jobset run`. The
  jobset (recent) is KiCad converging on the same fan-out-by-config model
  as OutJob — direct precedent for `datum-eda artifact generate --include`.
  KiCad keeps ERC and DRC as separate verbs; Datum treats domain as a
  PARAMETER of one `check run` because the run/finding/waiver machinery is
  identical. KiCad's per-format export subcommands are exactly the kind of
  surface bloat Datum omits behind `--include`.

- **OrCAD / Cadence (Allegro)**. Skill scripting and the constraint
  manager. Load-bearing: a single constraint authority queried uniformly;
  artifact generation driven by a manufacturing setup, not ad-hoc
  commands. Ceremony: Skill's enormous flat function namespace — Datum
  rejects flat namespaces in favor of families under one verb.

- **Horizon EDA**. Pool architecture and a clean entity hierarchy with
  an action/tool model where interactive tools emit edits. Load-bearing
  and directly aligned: tools produce edits, edits are the unit of
  change; selection is interactive state, not a persisted edit. Datum
  adopts this directly — selection is consumer state, never an Operation,
  never journaled.

Lean rationale distilled: every reference tool that scaled well did so by
making capability a PARAMETER of a small verb set (OutJob include-set,
KiCad jobset, Allegro constraint manager) and every tool that aged badly
did so by minting a verb per object-class-times-format. Datum picks the
former on purpose. A redundant tool is a defect, not a convenience.

## Tool Inventory

The shared surface is SEVEN tools. Each maps 1:1 onto a Decision 004
contract class. Each is answered against the eight questions. Domains add
variants under these tools; domains do not add peer tools.

CLI and MCP are isomorphic: `datum-eda <group> <verb>` (CLI) ==
`datum.<group>.<verb>` (MCP). Every CLI verb has `--json` returning the
exact MCP schema.

CLI naming policy: `datum-eda` is the canonical CLI executable and the only
command form used in this contract and all five domain contracts; the command
shape is always `datum-eda <group> <verb> ...`. This matches the repo/product
policy (product name `Datum EDA`, machine identifier `datum-eda`). The CLI
binary is `datum-eda` (crate `datum-eda-cli`), renamed from the former `eda`
binary / `eda-cli` crate; the bare `eda` name is historical/legacy only,
retained (if at all) solely as a temporary compatibility alias until removed,
and must not appear in new docs or specs. Bare `datum` CLI examples are also
legacy/noncanonical. The generic `datum` prefix is explicitly not used — it
is too broad and conflicts with the machine-identifier policy; `datum.*`
remains reserved for MCP tool family names only.

### 1. Session / Context — `DatumToolSession` + `DatumContextEnvelope`

1. **Manual UI action**: opening a project, focusing an editor pane, or
   launching a terminal/agent from the workspace establishes the session
   and seeds the context envelope (selection, projection, cursor).
2. **Operation emitted**: NONE. Session/context is read-only capability
   state. It is never an Operation and never journaled.
3. **CLI command**: `datum-eda context get`, `datum-eda context refresh`.
4. **MCP/AI tool**: `datum.context.get`.
5. **AI query/context needed**: this IS the context bootstrap. Returns
   `session_id`, `actor_type` (`User|Cli|Mcp|Script|Importer|Checker|
   Router|Assistant|ExternalAgent`), `project_id` / `project_root`,
   storage + schema version, `model_revision` + accepted-transaction tip,
   `projection_id` / selection / `cursor_context`, visible finding +
   artifact IDs, `capabilities` (`read|check|artifact|propose|
   apply-approved|direct-commit-by-policy`), `provenance_seed`, expiry +
   refresh command.
6. **Validating checks**: the envelope is a SNAPSHOT. A tool MUST
   `context refresh` before propose/apply if its envelope
   `model_revision` is stale. A session grants capability to Datum tools
   ONLY — never raw write to shards, caches, or journal files.
7. **Proof slice**: a terminal-launched agent calls `datum-eda context get`,
   reads `model_revision` and `project_id`, and carries them into a later
   query and proposal. See Proof Slice below.
8. **Not-supported-yet**: no `DatumToolSession` / `DatumContextEnvelope`,
   no `project_id` / `model_revision` substrate, no actor/provenance
   tagging exists today. The engine has `TransactionRecord` (`crates/
   engine/src/api/write_ops/`) but no actor field, no session, and no
   context endpoint. Defined ONCE here; no domain mints its own session.

### 2. Query — `DatumQueryTool` (read-only, never mutates)

1. **Manual UI action**: inspecting an object, hovering a net, opening a
   properties panel, describing the current selection.
2. **Operation emitted**: NONE. Queries never mutate. Selection is
   consumer state, exposed read-only via `selection.describe`; there is
   no select/deselect/filter mutation tool in any domain.
3. **CLI command**: `datum-eda query <family> [--domain ...] [--at
   <model_revision>]`.
4. **MCP/AI tool**: `datum.query.*`.
5. **AI query/context needed**: this IS the structured-context source.
   Families (Decision 004): `project.get_context`, `object.get`,
   `object.neighbors`, `selection.describe`, `schematic.query`,
   `pcb.query`, `library.query`, `rules.query`, `relationships.query`,
   `checks.query`, `manufacturing.query_projection`, `artifacts.query`,
   `transactions.query`, `provenance.query`. ALL per-domain
   `*.query` / `get_*` / `search_*` reads fold here as a
   `--family` / `--domain` parameter, not new tools.
6. **Validating checks**: every record returns stable `ObjectId` +
   `object_revision` + projection provenance. Cross-domain joins are
   keyed on `ComponentInstance` ONLY (binds `PlacedSymbol` <->
   `PlacedPackage`); an agent must never reconstruct the join by string
   matching refdes / name / path / sheet-id / board-id. Imported identity
   resolves via Import Map `import_key` (never `source_hash`);
   `provenance.query` surfaces `import_key`-keyed source provenance plus
   unknown-basis markers. Queries are pinned to a `model_revision` (or
   `projection_revision`) so context, findings, and proposals all cite
   the same revision.
7. **Proof slice**: `datum-eda query object.get --id <uuid>` returns the
   object with its revision and `ComponentInstance` link; a follow-up
   `object.neighbors` returns the bound package without string matching.
8. **Not-supported-yet**: EXISTING reads —
   `search_pool` / `get_part` / `get_package` / `get_symbols`
   (`crates/engine/src/api/project_surface.rs`, `crates/engine-daemon/
   src/dispatch.rs`), `get_board_summary` / `get_components` /
   `get_unrouted` / `get_net_info`, `get_design_rules` /
   `get_check_report` (`dispatch.rs:447/439`). GAP: not unified under one
   namespace, not `model_revision`-keyed, no `ComponentInstance` join, no
   `import_key` provenance, no `transactions` / `provenance` query family.

### 3. Run-Check — `DatumCheckTool` (read-only derived, never mutates source)

1. **Manual UI action**: running ERC/DRC, opening the violations panel,
   drilling into one finding.
2. **Operation emitted**: NONE to source. A persisted `CheckRun` is
   committed via `commit()` as DERIVED EVIDENCE (not a design op);
   ephemeral previews are session state.
3. **CLI command**: `datum-eda check run --domain <erc|drc|standards|
   process|manufacturing|all> --profile <id> --mode <edit|batch>`,
   `datum-eda check show <check_run_id>`.
4. **MCP/AI tool**: `datum.check.*`.
5. **AI query/context needed**: model/projection revision, scope (object
   IDs / output-job IDs), `CheckProfile` id, variant. `run_erc` and
   `run_drc` COLLAPSE into `check run` with a domain/profile parameter
   (kept only as thin compatibility aliases).
6. **Validating checks**: produces `CheckRun{check_run_id, model_revision
   / projection_revision, checker_version, pass|fail|error|stale}`. Each
   `CheckFinding` carries a stable finding id + affected `ObjectId`s +
   rule basis + inline explanation + `suggested_next_action` — so there
   is NO standalone explain tool. Findings are revision/hash-keyed derived
   state, never authority. Determinism gate: identical `model_revision` +
   variant + rule-versions => identical finding FINGERPRINTS. ZoneFill
   honesty: an unfilled / stale / unsupported zone boundary must NOT pass
   copper checks as if it were copper.
7. **Proof slice**: `datum-eda check run --domain drc --profile default`
   yields a `CheckRun`; a finding's fingerprint is stable across a
   no-op re-run on the same revision.
8. **Not-supported-yet**: EXISTING —
   `get_check_report` / `run_erc` / `run_drc` / `explain_violation`
   (`dispatch.rs:439/451/455/467`), `CheckSummary` merges ERC+DRC
   (`crates/engine/src/.../check_summary.rs`). GAP: `CheckRun` /
   `CheckFinding` not persisted nor revision/variant/
   `input_object_revisions`-keyed; not variant-aware; the `run_drc` rule
   set is hard-coded (`dispatch.rs:455-463`) instead of a `CheckProfile`
   parameter; explanation + `suggested_next_action` are not yet fields.
   `explain_violation` is addressed by `(domain, index)` today
   (`project_surface.rs` `explain_violation`), which breaks across runs;
   it must be re-addressed by stable finding FINGERPRINT.

### 4. Propose — `DatumProposalTool`

1. **Manual UI action**: invoking any high-risk / cross-domain / batch /
   destructive action (bulk reannotate, board-bound-symbol delete, ECO
   propagation, import repair) presents a proposal for review.
2. **Operation emitted**: a typed `OperationBatch` OWNED by the proposal;
   nothing is committed until `apply`.
3. **CLI command**: `datum-eda proposal create|show|validate|reject|defer|
   apply`.
4. **MCP/AI tool**: `datum.proposal.*`. AI may PROPOSE but never silently
   apply.
5. **AI query/context needed**: the proposal carries `OperationBatch` +
   rationale + affected `ObjectId`s + expected result + prepared-against
   `model_revision` + assumptions/risks/checks + `DatumToolSession`
   provenance.
6. **Validating checks**: returns `proposal_id` + machine-readable
   diff/preview + validation result + required acceptance policy +
   stale/conflict status vs current `model_revision`. Domain repair
   generators (`propose_repair`, IPC footprint synthesis,
   `UpdateLibraryBinding`, bulk delete, bulk reannotate, board-bound-
   symbol delete, panel-to-board promotion) are proposal PRODUCERS that
   REUSE existing typed ops — not new mutation primitives.
7. **Proof slice**: a tool proposes a bounded physical correction tied to
   one DRC finding; `proposal show` returns the diff; `proposal apply`
   yields one journaled transaction.
8. **Not-supported-yet**: EXISTING — only routing-specific proposal
   machinery (`crates/cli/src/command_project_route_proposal.rs`,
   `command_project_forward_annotation_proposal.rs`; review/apply/
   revalidate/explain in the MCP catalog). GAP: no general `Proposal`
   primitive, no general `OperationBatch` enum — routing proposals are the
   TEMPLATE to generalize.

### 5. Apply / Commit — `DatumCommitTool` (THE ONLY mutation gateway)

1. **Manual UI action**: any local visible undoable edit (place,
   transform, draw, label, marker, field edit, single property,
   align-distribute batch, output-job edit) commits directly; everything
   else commits via an accepted proposal.
2. **Operation emitted**: a typed `OperationBatch` through the single
   `commit()`.
3. **CLI command**: not a standalone user verb — every authoring CLI
   command and `proposal apply` routes through `commit()`. CLI/MCP/script
   batches commit directly only under explicit
   `direct-commit-by-policy`.
4. **MCP/AI tool**: invoked by `datum.proposal.apply`; MCP write tools
   land ONLY behind `commit()` so AI authoring is never unjournaled.
5. **AI query/context needed**: the approved proposal or the direct-commit
   `OperationBatch`, the prepared-against `model_revision`, and the
   session provenance.
6. **Validating checks**: `commit()` is the sole producer of
   `object_revision` + `model_revision` and the sole journal writer
   (Decision 001: apply-in-memory -> stage shard bytes + fsync -> append
   one `TransactionRecord` + fsync commit point -> atomic rename + dir
   fsync). REJECTS: stale base revisions (unless explicitly rebased +
   revalidated), unknown/mismatched object revisions, writes to derived
   caches as if source, proposal-policy bypass for AI/assistant/checker/
   import-repair, missing provenance/acceptance path. There is exactly ONE
   `commit()`; any per-domain save/write tool is a FORBIDDEN private path.
7. **Proof slice**: a direct-commit place-component and a proposal-applied
   correction both produce exactly one `TransactionRecord` with full
   provenance; both are undoable.
8. **Not-supported-yet**: EXISTING — imported-board ops journal correctly
   via `TransactionRecord` + undo/redo (`crates/engine/src/api/write_ops/`
   `basic_mutations.rs`, `assign_package_rule.rs`, `undo_redo/`). GAP: no
   single generic `commit()` / `OperationBatch`; ~115 native-write call
   sites (`write_canonical_json`) across the PCB CLI files plus the
   native-schematic CLI functions and all library/manufacturing authoring
   bypass `commit()` via `load_native_project -> write_canonical_json`
   (the private-writer migration defect the audit forbids); no fsync /
   journal-commit-point even on the journaled board path.

### 6. Artifact — `DatumArtifactTool` (derived projection, NEVER commit())

1. **Manual UI action**: generating manufacturing output (Gerber set,
   drill, BOM, PnP, assembly, panel, reports), or comparing an export to
   a prior one.
2. **Operation emitted**: NONE to source. Artifacts are derived
   projection snapshots, never committed as design state.
3. **CLI command**: `datum-eda artifact generate --include <gerber-set,drill,
   bom,pnp,...> [--panel <ctx>]`, `datum-eda artifact compare`,
   `datum-eda artifact show`.
4. **MCP/AI tool**: `datum.artifact.*`.
5. **AI query/context needed**: `project_id`, `model_revision`,
   output-job, variant, manufacturing-plan. Artifact SCOPE/format is a
   PARAMETER (`--include`) and panel is a CONTEXT (`--panel`) on the SAME
   `generate` verb — fan-out-by-config, never fan-out-by-tool (Altium
   OutJob / KiCad jobset precedent). `validate` / `compare` / `manifest` /
   `inspect` are oracle VERBS reading ONE T0 projection (T0 = exporter AND
   equivalence oracle).
6. **Validating checks**: every artifact records `project_id` +
   `model_revision` + output-job + variant + manufacturing-plan +
   projection-revision + generator-version + command/tool provenance +
   validation/equivalence state + per-file content-hash. Artifacts are
   never source authority. BOM/PnP rows keyed by `ComponentInstance`,
   never reference string. ZoneFill honesty: only `ZoneFill{Filled}`
   contributes copper; an unfilled native zone emits NO copper plus a hard
   finding.
7. **Proof slice**: `datum-eda artifact generate --include gerber-set,drill`
   records metadata; `compare` against a prior run reports equivalence.
8. **Not-supported-yet**: EXISTING (CLI-bridged to MCP via
   `mcp-server/server_runtime.py` `_run_cli_json`) —
   export/validate/compare/manifest/inspect manufacturing-set
   (`crates/cli/src/command_project_manufacturing.rs`), gerber plan
   (`command_project_gerber_plan.rs`). GAP: no artifact metadata (kind +
   path only), reference-string-keyed BOM/PnP, ZoneFill dishonesty
   (`crates/engine/src/export/.../copper.rs` pours the boundary as G36
   copper), triple-recompute (report/manifest/export) instead of one T0,
   no daemon dispatch (CLI-bridged only), ~30 redundant per-layer /
   per-format manufacturing subcommands instead of `--include` scopes.

### 7. Journal + Undo/Redo — `DatumJournalTool` (global, project-wide)

1. **Manual UI action**: viewing history, undo, redo.
2. **Operation emitted**: undo/redo are COMPENSATING `OperationBatch`es
   committed through the same `commit()` — NOT a private path and NOT
   per-domain. `list` / `show` are read-only.
3. **CLI command**: `datum-eda journal list|show|undo|redo`.
4. **MCP/AI tool**: `datum.journal.*`.
5. **AI query/context needed**: transaction history, proposal lineage,
   durable undo/redo cursors, provenance (actor/tool/session/
   acceptance-path/`model_revision`/affected-objects).
6. **Validating checks**: read-only except undo/redo. Defined ONCE; no
   domain ships its own undo/redo or clear-waiver/delete verb (removal =
   undo of the originating `AddWaiver`/`Create` transaction).
7. **Proof slice**: undo of a committed transaction produces a
   compensating transaction visible in `journal list`, and re-applies on
   redo.
8. **Not-supported-yet**: EXISTING — engine `undo()` / `redo()`
   (`crates/engine/src/api/write_ops/undo_redo/{undo.rs,redo.rs}`),
   daemon-bridged (`dispatch.rs:272/276`). GAP: undo/redo NOT exposed in
   the CLI (`crates/cli/src/main_project.rs`); no journal list/show/
   provenance query; journal not durable-from-first-cut on native paths
   (no fsync commit point).

## Minimal-Set Recommendation

The load-bearing shared surface is exactly these seven tools and no more:

- `datum-eda context get|refresh` — session/context bootstrap
- `datum-eda query <family>` — all reads, one namespace
- `datum-eda check run|show` — all checks, domain/profile as parameters
- `datum-eda proposal create|show|validate|reject|defer|apply` — all
  high-risk/cross-domain/batch mutations
- `datum-eda commit()` (gateway, not a standalone user verb) — the sole
  mutation path for direct edits and applied proposals
- `datum-eda artifact generate|compare|show` — all derived outputs,
  scope/panel as parameters
- `datum-eda journal list|show|undo|redo` — history + reversal, global

These seven map 1:1 onto the Decision 004 contract classes
(`DatumToolSession`, `DatumContextEnvelope`, `DatumQueryTool`,
`DatumCheckTool`, `DatumProposalTool`, `DatumCommitTool`,
`DatumArtifactTool`, `DatumJournalTool`) and use only ratified 000-012
vocabulary. Per-domain docs add typed `Operation` variants under these
seven; they add no peer tools.

## Omitted / Redundant Tools (with rationale)

Each entry is a capability folded into a parameter of an existing tool,
not a tool. A redundant tool is a defect.

- **`run_erc`, `run_drc`, per-domain check verbs** -> folded into
  `check run` with `--domain` + `--profile` + `--mode`. The run /
  finding / fingerprint / waiver machinery is identical across domains;
  splitting it triples the surface for no capability. `run_erc` / `run_drc`
  survive only as thin compatibility aliases. (KiCad keeps them separate;
  Datum does not, because the finding model is shared.)

- **`select` / `deselect` / selection-filter mutation tools** -> NONE.
  Selection is consumer-side interactive state (Horizon precedent),
  exposed read-only via `query selection.describe`. It is never an
  Operation and never journaled.

- **Standalone `explain_violation` tool** -> explanation +
  `suggested_next_action` are FIELDS on the `CheckFinding` returned by
  `check run/show`. `explain_violation` survives only as an interactive
  drill-down alias, addressed by stable finding FINGERPRINT (never
  `(domain, index)`, which breaks across runs —
  `project_surface.rs` today).

- **Per-domain delete verbs** (schematic `delete-*`, PCB
  delete-track/via/zone/keepout, library delete-object) -> ONE
  `DeleteObjects{ids:[ObjectId]}` typed op routed through `commit()`
  (object type recoverable from the id). Board-bound / bulk deletes
  escalate to a Proposal. No domain-specific delete tool.

- **Per-domain undo/redo and clear-waiver / delete-deviation /
  clear-binding** -> ONE global `journal undo|redo` (compensating
  `OperationBatch` through `commit()`). Removal = undo of the originating
  transaction. No per-domain reversal verb.

- **Per-format / per-layer export** (schematic netlist, PCB
  gerber-set/drill, manufacturing bom/pnp/excellon — ~30 manufacturing
  and ~18 per-layer gerber subcommands today) -> ONE `artifact
  generate|compare|show` with `--include` scope and `--panel` context;
  validate/compare/manifest/inspect are oracle verbs over one T0
  projection. Per-format bodies are kept internal, hidden from the public
  surface (Altium OutJob / KiCad jobset precedent).

- **Per-domain proposal/apply** (route-proposal,
  forward-annotation-proposal, library binding-update, `propose_repair`,
  IPC footprint synthesis) -> ONE `proposal create|...|apply`. Domain
  repair generators are proposal PRODUCERS reusing existing typed ops, not
  new apply paths.

- **Per-domain save/write/commit primitive** -> NONE. Every per-domain
  authoring tool EMITS a typed `OperationBatch`; it is not its own commit
  path. Any per-domain save/write tool is a FORBIDDEN private path. (This
  retires the 115 `write_canonical_json` call sites.)

- **Per-domain context/session bootstrap** -> ONE `DatumToolSession` +
  `DatumContextEnvelope` via `context get|refresh`. Domains consume the
  envelope; none mints its own session or revision.

- **`record_deviation` as its own tool (first slice)** -> ships as a
  `disposition` value on `waive_finding` (per the 003 `ElectricalDeviation`
  owner-review fork). Graduates to its own primitive only after the owner
  ratifies a standalone Deviation primitive + approval state machine.

## Shared Surface

This document IS the shared surface. The five per-domain tool-contract
docs (schematic, PCB, library, rules, manufacturing) must cross-reference
`docs/contracts/AI_CLI_MCP_TOOL_SURFACE.md` by name for all seven shared
operations and the four cross-cutting contracts (query, proposal/apply,
artifact, session). A domain doc adds ONLY:

- its domain-specific typed `Operation` variants (the `OperationBatch`
  members it emits through the shared `commit()`), and
- its `aiQueryContext` (which existing query families and finding kinds
  an authoring tool in that domain reads before proposing).

A domain doc must NOT restate or redefine query, selection, run-check,
propose, apply, export, undo, or redo. If a domain needs a behavior that
looks like a new shared tool, that is an Open Owner Question against THIS
doc, not a domain-local invention.

## Proof Slice & Fixture

Fixture: the canonical M7 regression project at
`~/Documents/kicad_projects/Datum-eda/datum-test/` (real KiCad design,
not fabricated). Imported identity exercises the Import Map `import_key`
path; the board exercises components, nets, zones, and DRC.

End-to-end proof slice (one scenario exercising all seven tools):

1. `datum-eda context get` -> envelope with `project_id`, `model_revision`,
   selection.
2. `datum-eda query object.get --id <component>` then `object.neighbors` ->
   the bound package via `ComponentInstance`, no string matching.
3. `datum-eda check run --domain drc --profile default` -> a `CheckRun` with a
   finding carrying a stable fingerprint + `suggested_next_action`.
4. `datum-eda proposal create` -> a bounded physical correction tied to that
   finding's fingerprint, with diff/preview and provenance.
5. `datum-eda context refresh` if the envelope revision is stale, then
   `datum-eda proposal apply` -> through `commit()`, one `TransactionRecord`.
6. `datum-eda artifact generate --include gerber-set,drill` -> metadata-
   stamped artifacts; ZoneFill honesty enforced (unfilled zone => no
   copper + hard finding).
7. `datum-eda journal list` shows the transaction with full provenance;
   `journal undo` produces a compensating transaction.

Determinism gate for the slice: identical `model_revision` + variant +
rule-versions => identical finding fingerprints and identical artifact
content-hashes.

## Not-Yet-Supported

- The entire substrate (general `Operation`/`OperationBatch`, single
  `commit()` + fsync journal, `DatumToolSession` /
  `DatumContextEnvelope`, `project_id` / `model_revision`, `ObjectId` /
  `object_revision`, `ComponentInstance`, Import Map `import_key`) is
  NOT IMPLEMENTED. It is the foundational slice that must land first.
- MCP currently has no write tools and no daemon dispatch for artifacts
  (CLI-bridged only via `server_runtime.py`). MCP write tools land only
  behind `commit()`.
- Findings are addressed by `(domain, index)` today; fingerprint
  addressing is not implemented.
- ZoneFill is dishonest today (boundary poured as copper); honest
  projection is not implemented.
- Direct-commit-by-policy automation is not implemented; default first
  slice is AI-proposes / human-accepts only.

## Open Owner Questions

1. **Substrate prerequisite (hard, blocks all five domains)**: confirm
   this doc DOCUMENTS THE TARGET and the substrate build (operation enum +
   commit + journal + session + identity) is its own foundational slice
   that must land before any per-domain authoring tool.
2. **First-slice query-family scope** (Decision 004 Q1): which of the 14
   query families ship first, and do they expose a local session socket,
   CLI discovery only, or both (Q2)?
3. **Direct-commit-by-policy**: which automation policies, if any, may
   apply proposals without per-proposal human approval (Q4)? Default
   recommendation: none in the first slice — AI proposes, human accepts.
4. **MCP write tools**: land now, or stay read-only until commit()/journal
   lands? Recommendation: MCP write tools land ONLY behind `commit()` so
   AI authoring is never an unjournaled surface.
5. **Finding addressing**: confirm waivers/proposals/repairs key on a
   DETERMINISTIC finding fingerprint (stable across rename/move via
   `ComponentInstance` + `ObjectId`), retiring the current `(domain,
   index)` addressing (`project_surface.rs`) which breaks across runs.
6. **`explain_violation`**: confirm it is demoted from a canonical tool to
   an interactive drill-down alias (explanation/`suggested_next_action`
   are fields on the finding), re-addressed by fingerprint not
   `(domain, index)`.
7. **`record_deviation`**: confirm deviation ships as a `disposition`
   value on `waive_finding` for the first slice (per the 003
   `ElectricalDeviation` owner-review fork) and graduates to its own tool
   only after the owner ratifies the standalone Deviation primitive +
   approval state machine.
8. **Artifact traceability format and home**: minimum required metadata
   fields (`model_revision` + output-job + variant + generator-version +
   per-file content-hash) and whether the manifest is a sidecar JSON in
   the output dir or a journaled artifact record (Decision 004 Q5; 012 Q).
9. **AI-created analyses/reports**: should they become first-class
   artifacts from the first slice, and how much non-mutating
   assistant/agent activity is persisted in project history vs local
   session history (Decision 004 Q5/Q6)?
10. **Read-only artifact generation routing**: should read-only
    manufacturing/artifact generation ever route through the daemon (it is
    CLI-bridged only today, no `dispatch.rs` entries) or is CLI-bridged
    projection the permanent architecture for read-only outputs?
