# Manufacturing Output Tool Contract

Status: draft implementation-spec 2026-06-19; derived from ratified
PRODUCT_MECHANICS 000-012.

## Driven by

- `docs/audits/scope-integration/DATUM_PRODUCT_MECHANICS_IMPLEMENTATION_READINESS_AUDIT.md`
  (ratified mechanism vocabulary and cross-doc invariants)
- `docs/decisions/PRODUCT_MECHANICS_000B_VIEW_COMPOSITION_AND_LIVE_PRODUCTION.md`
  (T0/T1 production projections; T0 is exporter and equivalence oracle)
- `docs/decisions/PRODUCT_MECHANICS_000D_STORAGE_AND_VERSIONING_MODEL.md`
  (source shards as partitions; `model_revision`)
- `docs/decisions/PRODUCT_MECHANICS_001_CANONICAL_EDIT_MODEL.md`
  (single `commit()` + journaled `TransactionRecord`)
- `docs/decisions/PRODUCT_MECHANICS_004_AI_TOOLING_CONTRACT.md`
  (shared CLI/MCP/query/check/artifact/proposal/commit contract classes)
- `docs/decisions/PRODUCT_MECHANICS_010_INDUSTRY_STANDARDS_COMPLIANCE.md`
  (evidence-based posture; no certification claims)
- `docs/decisions/PRODUCT_MECHANICS_012_APPLICATION_QUALITY_BAR.md`
  (projection honesty, ZoneFill honesty, artifact traceability, no
  private writers)

## Purpose & Scope

This contract defines the concrete authoring/tool surface for the
**manufacturing output** domain: the deterministic fabrication and
assembly artifacts Datum projects from the resolved `DesignModel` —
Gerber (RS-274X) layer sets, NC/Excellon drill, BOM, pick-and-place,
and (deferred) panel outputs.

The domain has exactly one mutation primitive and one projection
primitive:

- The **OutputJob** is the only authored manufacturing state. It is a
  source shard, created/edited/deleted through the single `commit()`
  path and journaled. It is configuration, not an artifact.
- The **manufacturing-set projection** is the only generation surface.
  Artifacts are derived state keyed to `model_revision`, never source
  authority, and never produced through `commit()`. Which artifacts a
  run emits is an `--include` *parameter*, not a separate tool. Panel
  is a `--panel`/`--context` parameter on the same generate verb, not a
  parallel pipeline.

In scope: OutputJob authoring; manufacturing-set generation and its
validate/compare/manifest/inspect oracle verbs; artifact traceability
metadata; ZoneFill-honest copper projection. Out of scope (referenced,
not redefined): the seven shared operations (session/context, query,
check, propose, commit, artifact, journal) defined once in
`docs/contracts/AI_CLI_MCP_TOOL_SURFACE.md`; schematic/PCB/library/rules
authoring;
standards-report content (PRODUCT_MECHANICS_010).

## Reference-Tool Survey (with the lean rationale)

**Altium Designer (primary reference).** The OutJob is the single
artifact-orchestration object: output generators (Gerber, ODB++,
IPC-2581, NC Drill, BOM, PnP, assembly, 3D) are bound to output
containers once and regenerated deterministically. Load-bearing: the
OutJob abstraction and the single "Generate Outputs" action over the
job. Ceremony: per-generator modal dialogs and the separate CAMtastic
editor. The lean lesson is decisive — Altium has **one** generate
action over a saved config, **not N** per-format export commands.
Fan-out-by-config, never fan-out-by-tool.

**KiCad 8.x.** `kicad-cli pcb export gerbers/drill/pos` plus the newer
"Output Jobs" jobset: one job file, one runnable command, deterministic
outputs. Load-bearing: the headless jobset. Ceremony: the per-format
GUI dialogs duplicate the CLI. The jobset is direct proof that a single
export verb over a saved config beats a wall of per-format verbs.

**OrCAD / Cadence Allegro.** Artwork/film-control as a *named* output
set; NC Drill with legend; IPC-2581/ODB++; and the Panel Editor as a
*distinct* artifact that never edits the source board. Load-bearing:
named output set + panel-isolation. Ceremony: heavy per-film parameter
dialogs.

**Horizon EDA.** Gerber export with a *saveable* export-settings blob
persisted in the project (a lightweight output job), and BOM sourced
from the canonical part, not a free string. Load-bearing: export
settings as journaled project data + part-keyed BOM — directly
validates Datum's OutputJob-as-authored-state and the
`ComponentInstance`-keyed BOM invariant.

**Lean rationale.** Every reference tool that scaled converged on one
saved config object and one generate action. The redundant pattern is
a per-format/per-layer command wall. Datum therefore exposes three
surfaces, not nine; the production engineering value is in three real
adds (OutputJob authored state, artifact traceability, ZoneFill
honesty), none of which is a new tool.

## Tool Inventory

### Tool 1 — `output-job` (the single authored manufacturing-plan object)

The only mutation surface in the domain. Everything else is projection.

- **(1) Manual UI action:** User opens the Output Job editor, names a
  job, selects board + variant context, ticks the artifact scopes to
  include (gerber-set, drill, bom, pnp, assembly), sets prefix / units
  / format, and saves. This is the one place outputs are configured;
  regeneration is a single later action. Mirrors Altium OutJob, KiCad
  jobset, and Horizon saved export-settings.
- **(2) Operation it emits:** A typed `OperationBatch` of
  `{CreateOutputJob | UpdateOutputJob | DeleteOutputJob}` through the
  single `commit()` path, producing one journaled `TransactionRecord`.
  `OutputJob` is authored source state persisted as a manufacturing-plan
  shard; generated artifacts remain derived. OutputJob create/edit/delete
  is **direct-commit class** (local, visible, undoable, no hidden
  cross-domain/destructive implication) per the shared
  proposal/apply contract.
- **(3) CLI command:** `datum-eda project output-job {create|edit|delete|`
  `list|show} <root> --name <job> --include gerber-set,drill,bom,pnp`
  `--variant <v> --prefix <p>`. One multi-verb noun, not separate tools
  per verb. **GAP — does not exist today.** Configuration is currently
  an ad-hoc, prefix-only flag hardcoded in
  `crates/cli/src/command_project_manufacturing.rs`.
- **(4) MCP / AI tool:** `project_output_job_upsert` /
  `project_output_job_list`, CLI-bridged via
  `mcp-server/server_runtime.py:_run_cli_json`. `upsert` collapses
  create + edit. **GAP — not yet implemented.**
- **(5) AI query / context needed:** Read-only via the shared
  `datum-eda query` namespace — current `model_revision`, available
  variants, stackup layer kinds (from the gerber plan), existing output
  jobs, and which artifact scopes the resolved board supports. No new
  per-domain read tool.
- **(6) Validating checks:** Reuse `ProjectResolver` shard-version and
  reference validation; reject jobs referencing a nonexistent variant
  or layer. `UpdateOutputJob` is a journaled transaction, so undo/redo
  works through the shared journal.
- **(7) Proof slice:** On the imported `datum-test` native project,
  create an output job selecting gerber-set + drill + bom + pnp, commit,
  undo, redo; assert a journal `TransactionRecord` exists for each and
  that the base board shards are byte-unchanged across the cycle.
- **(8) Explicitly not-supported-yet:** The object does not exist. There
  is no persisted, journaled, variant-aware job today — only a `--prefix`
  flag. No `OutputJob` symbol appears anywhere in the engine (grep
  confirmed empty).

### Tool 2 — `manufacturing-set` projection (one export verb + oracle)

One export verb whose included artifacts are an `--include` parameter,
plus its validate/compare/manifest/inspect oracle verbs reading one T0
projection. This subsumes the former separate gerber-set / drill / bom /
pnp export tools — they are now `--include` scopes.

- **(1) Manual UI action:** User picks an output job (or the default
  "all"), picks an output directory, and clicks Generate; Datum writes
  the gerber set, NC/Excellon drill, BOM, and PnP, then shows the
  artifact list with traceability metadata. The same surface answers
  "what files *should* exist" (manifest), "are they present" (inspect),
  and "do they match the projection" (validate/compare).
- **(2) Operation it emits:** **None.** Export and its oracle verbs are
  derived projections and correctly do not call `commit()`. Export
  *records* an Artifact manifest (`model_revision` + output-job id +
  variant + generator version + per-file content hash). Existing code:
  `command_project_manufacturing.rs:export_native_project_`
  `manufacturing_set` composes `bom + pnp + drill.csv + drill.drl +`
  gerber-set via `command_project_gerber_plan.rs:export_native_`
  `project_gerber_set`. **REQUIRED refactor:** report/manifest/export
  must read **one** T0 projection (today each independently recomputes
  `plan_native_project_gerber_export`).
- **(3) CLI command:** `datum-eda project export-manufacturing-set <root>`
  `--output-dir <dir> [--include gerber-set,drill,bom,pnp]`
  `[--job <name>] [--variant <v>] [--prefix <p>]`. Export EXISTS;
  `--include` / `--job` / `--variant` are GAPS. Oracle verbs EXIST:
  `validate-manufacturing-set`, `compare-manufacturing-set`,
  `manifest-manufacturing-set`, `inspect-manufacturing-set`. The
  per-domain export-gerber-set / export-bom / export-pnp /
  export-excellon-drill (and twins) collapse into `--include` scopes of
  this verb family; keep their bodies internally, hide them from the
  public surface.
- **(4) MCP / AI tool:** `project_export_manufacturing_set` /
  `project_validate_manufacturing_set` /
  `project_compare_manufacturing_set` /
  `project_manifest_manufacturing_set` /
  `project_inspect_manufacturing_set` (CLI-bridged, EXIST). The
  per-domain `project_export_bom/pnp/gerber_set/excellon_drill` MCP
  tools are dropped from the public surface in favor of `--include`.
- **(5) AI query / context needed:** `model_revision`, output-job /
  variant selection, output dir, include-scope. The agent treats the
  returned manifest (kind, filename, content hash) as the equivalence
  record and does **not** re-derive files. Drill context reads via the
  shared query: pad/via drill sizes and plated/non-plated class. BOM/PnP
  context reads `ComponentInstance` + variant fitted state.
- **(6) Validating checks:** `validate-` / `compare-manufacturing-set`
  re-render from the projection and assert byte/semantic equality
  (`command_project_manufacturing.rs`). This **is** the projection ==
  export oracle — but unification to one T0 projection is required so
  manifest/report/export cannot diverge. **CRITICAL GAP carried in:**
  copper validation currently passes even on dishonest zone copper (see
  item 8).
- **(7) Proof slice:** Export the manufacturing set for `datum-test` to
  a tmp dir; run validate- and compare-manufacturing-set; assert
  `matched_count == expected_count` and `missing == mismatched ==`
  `extra == 0`. Drift slice: mutate one component value via `commit()`,
  then compare shows exactly one drift row keyed by `ComponentInstance`
  (not reference string). PnP slice: move one component via `commit()`,
  compare shows one position drift. ZoneFill slice: a board with an
  unfilled zone must emit **no** copper region and a hard finding.
- **(8) Explicitly not-supported-yet:**
  1. **Artifact metadata not emitted.** `ManufacturingArtifactView`
     today is `kind` + `output_path` only — no `model_revision`,
     output-job id, variant, generator-version, or content-hash.
  2. **Variant-scoped export and `--include` scoping not wired.**
  3. **BOM/PnP rows are reference-string keyed**
     (`command_project_inventory.rs` `NativeBomRow.reference` /
     `NativePnpRow.reference`, with drift keyed on
     `expected_by_reference`), violating the `ComponentInstance` join
     invariant.
  4. **ZoneFill DISHONESTY.** `crates/engine/src/export/copper.rs`
     lines ~176-202 pour the zone `polygon` boundary directly as an
     exportable G36 region with no `Filled`/`Unfilled`/`Stale` gate and
     no unfilled-zone finding — a correctness defect that must be fixed
     before any fabrication trust.
  5. **Drill gaps.** No separate plated/non-plated files, no route/slot
     (G85), no board-edge cutouts; the legacy `drill.csv` is a redundant
     second format alongside `.drl`.
  6. **Assembly/supply gaps.** No MPN/supply columns, no top/bottom PnP
     split, no fiducial flags (deferred per Standards Audit Batch 1).

### Tool 3 — `panel` projection (deferred future peer; panelization isolation)

Listed so it is not mistaken for a current tool. Recommend deferring to
M8 unless the owner pulls it forward.

- **(1) Manual UI action:** User defines a panel (step-and-repeat board
  array + rails, tabs/mouse-bites, V-scores, panel fiducials, tooling
  holes, coupons, labels) and generates panel-level outputs. Panel
  features are authored panel state; panel outputs reuse the
  manufacturing-set projection under a panel context.
- **(2) Operation it emits:** Panel-feature edits are authored panel
  state via `commit()`ed `OperationBatch{CreatePanel | AddRail | AddTab`
  `| ...}`; they MUST NOT mutate source board geometry. Panel outputs
  are derived projections like the board set — they reuse the
  manufacturing-set projection verb, not a parallel export pipeline.
- **(3) CLI command:** NOT YET (defer to M8 unless owner pulls forward):
  `datum-eda project panel {create|add-rail|add-tab|...}` and
  `export-manufacturing-set --panel <name>` — panel as a context flag on
  the same export verb, not a new export-panel-set tool.
- **(4) MCP / AI tool:** NOT YET: `project_panel_*` (the mutations) plus
  reuse `project_export_manufacturing_set --panel`.
- **(5) AI query / context needed:** source board outline + size, panel
  constraints, `model_revision`; the agent must never write panel
  features back into the board shard.
- **(6) Validating checks:** Panelization-isolation drift gate — board
  shard bytes unchanged after panel ops; panel outputs validate via the
  same per-layer oracle scaled by step-and-repeat.
- **(7) Proof slice:** (future) Build a 2x2 panel of `datum-test`,
  export panel gerber/drill via `export-manufacturing-set --panel`,
  assert the source `datum-test` board shards are byte-identical
  pre/post.
- **(8) Explicitly not-supported-yet:** Entirely unimplemented — no
  panel object, no step-and-repeat, no rails/tabs/V-scores/coupons
  anywhere in the engine or CLI. Recommend deferring to M8.

## Minimal-Set Recommendation

**Three surfaces total, down from nine.**

1. `output-job` upsert/list — the **only** mutation, on `commit()` +
   journal.
2. `manufacturing-set` projection — **one** export verb whose included
   artifacts are an `--include` parameter (gerber-set, drill, bom, pnp),
   plus its validate/compare/manifest/inspect oracle verbs reading **one
   T0 projection**.
3. `panel` — a deferred future peer that reuses (2) under a `--panel`
   context, not a parallel pipeline.

This is the consistent expression of the matrix's own lean intent: the
exploratory matrix argued fan-out-by-scope yet still listed gerber-set /
drill / bom / pnp / manufacturing-set / manifest / inspect as five-to-
seven surviving tools — that contradiction is the defect removed here.
Real-world proof: Altium OutJob and KiCad jobset both expose one
generate action over a saved config, not N per-format export commands.
The three real engineering adds remain unchanged and are not new tools:
OutputJob authored state, Artifact traceability metadata, and
ZoneFill-honest copper.

## Omitted / Redundant Tools

- **export-gerber-set / export-bom / export-pnp / export-excellon-drill
  as distinct top-level export tools** (and their validate/compare/
  inspect twins). `export-manufacturing-set` already composes all of
  them (`command_project_manufacturing.rs:export_native_project_`
  `manufacturing_set`). A user wanting only gerbers passes
  `--include gerber-set`, not a second command — exactly KiCad jobset /
  Altium OutJob. Collapsing removes ~16 redundant public surfaces (4
  export + their validate/compare/inspect). Keep bodies internally; hide
  from the public surface.

- **manifest-manufacturing-set + inspect-manufacturing-set +
  report-manufacturing as distinct tools.** These are the same
  projection oracle as validate/compare (expected vs present:
  matched/missing/extra) over one projection. They are *verbs* of the
  manufacturing-set surface, not separate tools. `report-manufacturing`
  additionally recomputes the plan independently (it calls
  `plan_native_project_gerber_export` again, as do manifest and export —
  confirmed: three call sites in `command_project_manufacturing.rs`,
  `command_project_gerber_plan.rs`, `command_project_surface.rs`), so
  two-plus recompute paths risk projection divergence and violate
  projection == export equivalence. Fold report's counts into manifest
  as a header, all driven by one T0 projection.

- **18 per-layer single-file gerber export/validate/compare
  subcommands** (`ExportGerberCopperLayer` / `Soldermask` / `Paste` /
  `Silkscreen` / `Mechanical` / `Outline` + validate/compare twins,
  `cli_args_project_commands.rs`). These are the gerber-set pipeline
  scoped to one layer; they exist as golden-test fixtures, not
  user-facing tools, and the set verb already iterates them internally
  (`command_project_gerber_plan.rs:export_native_project_gerber_set`).
  Express as `export-manufacturing-set --include gerber-set --layer`
  `<id> --kind copper` if ever needed publicly. Keep internally for
  tests, omit from the public surface.

- **export-drill (legacy CSV drill:** `ExportDrill` / `ValidateDrill` /
  `CompareDrill` / `InspectDrill`, `command_exec_drill.rs`). Two drill
  formats for the same via data. Excellon (`.drl`) is the fab-accepted
  NC format; the CSV duplicates the via inventory already queryable via
  the shared `datum-eda query`. Demote CSV to a `--format csv` flag on the
  drill scope if ever needed; remove the 4 standalone CSV-drill tools.

- **compare-gerber-export-plan (plan-vs-plan diff).** Plan-to-plan
  comparison is a regression/test utility, not a manufacturing workflow.
  The user-facing equivalence question is artifact-vs-projection
  (validate/compare-manufacturing-set). Keep as an internal harness
  check, omit from the lean catalog.

- **export-panel-set as a separate export pipeline.** A second export
  verb for panel outputs would duplicate the board projection machinery
  and create a second projection == export oracle to keep in sync. Panel
  is a context (`--panel`) on the one manufacturing-set export verb;
  only the panel-feature mutations are new. Avoids a parallel pipeline
  before it is even built.

## Shared Surface

This contract adds only the domain-specific `OutputJob` operation
variants (`CreateOutputJob` / `UpdateOutputJob` / `DeleteOutputJob`) and
the manufacturing `aiQueryContext`. It does **not** redefine the seven
shared operations. See `docs/contracts/AI_CLI_MCP_TOOL_SURFACE.md` for
the single
definitions of:

- **Session/Context** (`DatumToolSession` + `DatumContextEnvelope`) —
  this domain consumes the envelope and refreshes before propose/apply;
  it mints no session of its own.
- **Query** (`DatumQueryTool`) — manufacturing reads fold in as the
  `manufacturing.query_projection` and `artifacts.query` families; this
  domain adds no new read tool.
- **Check** (`DatumCheckTool`) — the manufacturing/process check domain
  and the ZoneFill-honesty finding are profiles/findings of the shared
  check surface, not domain checks.
- **Propose** (`DatumProposalTool`) — used only if/when an OutputJob or
  panel edit is escalated; OutputJob edits are direct-commit class.
- **Commit** (`DatumCommitTool`) — the **only** mutation gateway;
  OutputJob create/edit/delete is a typed `OperationBatch` through it.
  Any per-domain manufacturing save/write path is a forbidden private
  writer.
- **Artifact** (`DatumArtifactTool`) — the manufacturing-set projection
  IS this surface scoped to fabrication/assembly outputs; `--include`
  scope, `--panel` context, the one T0 projection oracle, artifact
  traceability metadata, `ComponentInstance`-keyed BOM/PnP, and ZoneFill
  honesty are all defined there.
- **Journal + Undo/Redo** (`DatumJournalTool`) — OutputJob transactions
  participate in the global journal; this domain ships no private undo.

## Proof Slice & Fixture

**Fixture:** `datum-test` at
`~/Documents/kicad_projects/Datum-eda/datum-test/` (`datum-test.kicad_pcb`
+ `.kicad_sch` + `.kicad_pro`) — the canonical M7 regression fixture.
Import to a native project root, then run the manufacturing proof
slices.

**Proof slices (in dependency order):**

1. *OutputJob journal slice* (Tool 1): create job, commit, undo, redo;
   assert a `TransactionRecord` per step and byte-unchanged base board
   shards.
2. *Set equivalence slice* (Tool 2): export to tmp dir; validate- and
   compare-manufacturing-set; assert `matched == expected` and
   `missing == mismatched == extra == 0`.
3. *BOM drift slice*: mutate one component value via `commit()`; compare
   shows exactly one drift row keyed by `ComponentInstance`.
4. *PnP drift slice*: move one component via `commit()`; compare shows
   one position drift.
5. *ZoneFill-honesty slice*: an unfilled zone must emit **no** copper
   region and a hard finding (exercises `copper.rs` ~176-202).

**Fixture blocker (confirmed):** `datum-test.kicad_pcb` contains **zero
zones** (verified — grep for `zone` returns 0 matches). The
ZoneFill-honesty slice therefore cannot be exercised against this
fixture. Per the no-synthetic-fixtures rule, this is an Open Owner
Question below — do not fabricate a zone; request a zone-bearing
variant.

## Not-Yet-Supported

- `OutputJob` authored state (object, shard, CLI, MCP) — does not exist.
- Artifact traceability metadata — `ManufacturingArtifactView` is
  `kind` + `output_path` only.
- `--include` scoping, `--job` selection, and `--variant`-scoped export.
- `ComponentInstance`-keyed BOM/PnP rows — currently reference-string
  keyed, drift keyed on the string.
- ZoneFill-honest copper — currently pours the boundary as G36 copper
  with no `Filled` gate and no finding.
- Drill: plated/non-plated separation, route/slot (G85), board-edge
  cutouts; legacy `drill.csv` is a redundant second format.
- Assembly extras: MPN/supply columns, top/bottom PnP split, fiducial
  flags (deferred per Standards Audit Batch 1).
- Panelization: no panel object, step-and-repeat, rails/tabs/V-scores/
  coupons (deferred to M8).
- Daemon dispatch: manufacturing/gerber/drill are CLI-bridged only
  (confirmed — no entries in `crates/engine-daemon/src/dispatch.rs`).

## Open Owner Questions

1. **OutputJob first-slice scope:** a single default job (all artifacts)
   for M7, or named multi-job ("fab" vs "assembly") from the start?
   Determines whether `--job` is needed in the first cut.
2. **Artifact traceability format:** minimum required metadata fields
   (`model_revision` + output-job-id + variant + generator-version +
   per-file content-hash) and where the manifest lives (sidecar JSON in
   the output dir vs a journaled artifact record)?
3. **ZoneFill honesty fix path:** authorize gating copper export on
   `ZoneFill{Filled}` and emitting a hard finding for native unfilled
   zones. Confirm whether a real fill solver is in M7 scope or whether
   an unfilled zone simply blocks export. Also confirm provision of a
   **zone-bearing** `datum-test` variant so the honesty slice can run
   (current fixture has zero zones).
4. **BOM/PnP join-key migration:** confirm switching row identity from
   reference-string to `ComponentInstance` is in-scope now (it changes
   golden CSVs and drift semantics).
5. **T0 unification:** unify report/manifest/export onto one T0
   projection now (removes the triple-recompute divergence risk), or
   accept the current triple-recompute until panelization forces it?
6. **Plated vs non-plated drill:** separate NC files (fab-standard) or a
   single combined Excellon with a tool-class column for the first
   slice?
7. **Panelization:** defer entirely to M8, or land a minimal panel
   object + isolation drift gate during M7 manufacturing work?
8. **Daemon routing:** should manufacturing export ever route through
   the daemon (CLI-bridged only today, no `dispatch.rs` entries), or is
   CLI-bridged projection the intended permanent architecture for
   read-only outputs?
