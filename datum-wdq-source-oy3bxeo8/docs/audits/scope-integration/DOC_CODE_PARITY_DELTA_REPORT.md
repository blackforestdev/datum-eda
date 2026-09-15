# Doc/Code Parity Delta Report

> **HISTORICAL SNAPSHOT (2026-05-25) — SUPERSEDED.** This report is a
> point-in-time audit captured on 2026-05-25. Any counts or inventories it
> records (including the `mcp_runtime_methods` count in §0A) are frozen at
> that snapshot and are NOT current. The authoritative, machine-checked
> inventory shapes now live in `specs/SPEC_PARITY.md`
> (gated by `scripts/check_spec_parity.py`; run `--print` for live names);
> consult that file, not this report, for current numbers. Retained for
> historical/remediation-tracking context only.
>
> **Status**: Audit plus remediation record. Initially produced 2026-05-25;
> updated 2026-05-25 after adding the first machine-checkable spec/code parity
> gate.
>
> **Purpose**: Surface every drift class precisely enough that doc-correction
> work can be authorized in scoped batches, and so that automated parity
> gates can be specified against concrete failure modes that already exist.
>
> **Scope**: This report identifies what to rewrite, where, and what gate would
> have caught the drift. The first gate now exists in
> `scripts/check_spec_parity.py` and is wired through
> `scripts/run_drift_gates.sh`.

---

## 0A. Remediation Landed 2026-05-25

The repo now has a compact, manifest-driven parity mechanism:

- `specs/spec_parity_manifest.json` defines code-derived inventories and the
  owning spec for each inventory.
- `specs/SPEC_PARITY.md` records checked counts plus SHA-256 digests for those
  inventories.
- `scripts/check_spec_parity.py` verifies the inventories and supports
  `--update` and `--print`.
- `scripts/run_drift_gates.sh` now runs `check_spec_parity.py` after progress
  coverage and before alignment/file-size gates.

Current enforced inventories:

| Inventory | Count | Owner |
|-----------|-------|-------|
| `mcp_runtime_methods` | 75 | `specs/MCP_API_SPEC.md` |
| `cli_project_commands` | 182 | `specs/PROGRAM_SPEC.md` |
| `engine_text_modules` | 11 | `docs/gui/DATUM_TEXT_ENGINE_PHASE_2_IMPLEMENTATION_PLAN.md` |
| `m7_text_visual_fixtures` | 4 | `docs/gui/DATUM_TEXT_ENGINE_FIDELITY_FIXTURES.md` |

This does not make the specs complete. It creates an enforceable failure mode:
if any tracked implemented surface changes shape without a matching spec
inventory refresh, the drift gate fails.

## 0. Headline Findings

| Surface | Spec/charter claim | Actual code | Drift |
|---------|---------------------|-------------|-------|
| Workspace crates | 4 (`engine`, `cli`, `engine-daemon`, `test-harness`) — CLAUDE.md and `PROGRESS.md` Infrastructure table | **7** (adds `gui-protocol`, `gui-render`, `gui-app`) | Three crates undocumented |
| MCP tool count | "26+" (CLAUDE.md), "26/26 implemented" (PROGRESS M2 MCP table) | **75 runtime tools tracked in `SPEC_PARITY.md`** | Headline docs stale, but the runtime inventory is now gated |
| MCP daemon dispatch | PROGRESS Infrastructure row says "53 methods in `dispatch.rs`" | **53 daemon methods + 22 CLI-bridged runtime methods = 75 MCP runtime methods** | The old daemon-only parity model was wrong; see §3 |
| CLI top-level commands | M2 PROGRAM_SPEC + PROGRESS M2 table list 8 commands | **9 top-level** (`Import`, `Query`, `Drc`, `Erc`, `Check`, `Pool`, `Project`, `Plan`, `Modify`) | OK at top level; PROGRESS still labels this as "M2 slice" — see §4 |
| CLI `project` subcommand variants | PROGRESS scattered references mention ~30 surfaces under M4/M5/M6/M7 | **182 variants** in `ProjectCommands` enum | Major — no enumerated parity table exists for the post-M4 CLI surface |
| Engine API `pub fn` (all of `crates/engine/src/api/`) | ENGINE_SPEC §5.1 enumerates ~40 methods | **64 unique** `pub fn` names | +24 methods not in ENGINE_SPEC §5.1 table |
| Engine subsystems documented | text-engine subsystem **not mentioned anywhere** in `specs/` | `crates/engine/src/text/` has 11 files (backend, determinism, geometry, layout, mesh, newstroke_data, outline, registry, semantic, stroke, mod) | Whole subsystem absent from specs |
| M7 framing | "narrow read-only route-proposal review layer" per `m7_opening.md` and PROGRESS M7 section | `gui-protocol` (9.2k LOC), `gui-render` (10.3k LOC), `gui-app` (3.6k LOC), full wgpu renderer, hit-testing, **board-text-editing field enums**, full visual regression harness | Charter understates implemented surface by an order of magnitude |
| Milestone framing | CLAUDE.md "In progress (M6)" | Active work is M7 + Standards Audit; M6 is "frozen pending evidence" per its own charter | CLAUDE.md is one milestone behind |
| Standards Audit Batch 1 | Spec stubs landed in commit `db98eff` (`AttachModel`, `DetachModel`, IBIS/SPICE/Touchstone attach, IDF/STEP/ODB++/IPC-2581 exports, IPC pool tables, supply-chain lookups) | **No implementation status row in PROGRESS.md** | New spec surface has no tracker |
| Uncommitted in-flight work | Not described anywhere (PROGRESS doesn't claim it) | +9,308 / −1,555 across 79 files: new `command_project_board_layout.rs`, new `main_tests_project_board_text.rs`, KiCad import depth, silkscreen export expansion, gui-protocol +3.5k, gui-render +2.8k, gui-app +1.5k | Pre-merge — expected, but flags that "PROGRESS updated when code changes" claim already broke for ~30 prior commits |

---

## 1. Crate Inventory Drift

**Code reality** (`Cargo.toml`):
- `crates/engine` — 252 files, 62k LOC
- `crates/cli` — 353 files, 90k LOC
- `crates/engine-daemon` — 16 files, 4.4k LOC
- `crates/test-harness` — 24 files, 7.6k LOC
- `crates/gui-protocol` — 1 file, 9.2k LOC
- `crates/gui-render` — 7 files, 10.3k LOC
- `crates/gui-app` — 2 files, 3.6k LOC

**Documented as**:
- `CLAUDE.md` lines 151–157 (ASCII tree): `engine`, `cli`, `engine-daemon`, `test-harness` — four
- `CLAUDE.md` line 19: "26+ MCP tools, CLI with proper exit codes" — pre-M5 figure
- `PROGRESS.md` line 1222: "Rust workspace (4 crates)" — wrong
- `PROGRAM_SPEC.md` line ~ (workspace section): same 4-crate claim

**Drift class**: structural — orientation reader cannot find GUI work that is the largest active development front by line count and the bulk of the uncommitted diff.

---

## 2. Engine API Surface Drift

**Code reality**: 64 unique `pub fn` declarations under `crates/engine/src/api/` across submodules:
- `api/mod.rs` — core entry points (`new`, `has_open_project`, `can_undo`, `can_redo`, …)
- `api/project_surface.rs` + `api/project_surface/` — native project surface
- `api/query_surface.rs` — board/schematic query API
- `api/write_ops/` — `basic_mutations.rs`, `component_replacements.rs`, `assign_package_rule.rs`, `undo_redo.rs`
- `api/save_kicad/` + `api/save_kicad.rs` — KiCad write-back
- `api/persistence_helpers.rs`, `api/ops_helpers.rs`, `api/check_summary.rs`, `api/api_types.rs`

**Documented as**:
- `ENGINE_SPEC.md` §5.1 "Current Implemented Engine API (2026-03-25)" — a hand-curated table of ~40 methods. Cuts off at M3 mutation set.
- §5.2 "Target M2+ Engine API" — aspirational, mixes target and current

**Specific omissions** (sampling): every native-authoring write (place_board_*, edit_board_*, delete_board_*, set_board_*, place_*, draw_wire, …), every routing surface (route_proposal, route_apply*, export_route_*, inspect_route_proposal_artifact, revalidate_*, route_strategy_*), every forward-annotation surface, every manufacturing exporter, every gerber compare/validate/inspect, board text/keepout/dimension authoring.

**Drift class**: spec under-claims engine capability — an AI agent reading ENGINE_SPEC.md would assume those surfaces don't exist.

---

## 3. MCP Tool Catalog vs Runtime Dispatch Drift

**Runtime chain**:
- Python `mcp-server/tools_catalog_data.py` declares the MCP tool catalog.
- Rust `engine-daemon/src/dispatch.rs` owns long-lived daemon JSON-RPC methods.
- Python `mcp-server/server_runtime.py` also owns CLI-bridged runtime methods
  for project-root operations that are implemented through the native CLI rather
  than the daemon session object.

**Code reality**:
- `tools_catalog_data.py` declares **75 tools**.
- `engine-daemon/src/dispatch.rs` has **53** daemon match arms.
- `server_runtime.py` has **22** CLI-bridged runtime methods for the route,
  strategy, and project validation tools listed below.
- The enforced MCP runtime surface is now `daemon_methods ∪ cli_bridge_methods`,
  and `scripts/check_spec_parity.py` plus `scripts/check_progress_coverage.py`
  require that runtime surface to match the catalog/spec inventory.

```
apply_route_proposal_artifact          inspect_route_strategy_batch_result
capture_route_strategy_curated_baseline revalidate_route_proposal_artifact
compare_route_strategy_batch_result    review_route_proposal
export_route_path_proposal             route_apply
export_route_proposal                  route_apply_selected
gate_route_strategy_batch_result       route_proposal
inspect_route_proposal_artifact        route_proposal_explain
route_strategy_batch_evaluate          route_strategy_compare
route_strategy_delta                   route_strategy_report
summarize_route_strategy_batch_results validate_project
validate_route_strategy_batch_result   write_route_strategy_curated_fixture_suite
```

**Correction**: the original audit treated every catalog tool as daemon-backed.
That was too narrow. These 22 methods are intentionally CLI-bridged through the
Python runtime rather than Rust daemon match arms. The parity gate now models
that explicitly.

**Documented as**:
- `MCP_API_SPEC.md` documents all 75 current runtime methods.
- `specs/SPEC_PARITY.md` now records the 75-method runtime inventory digest.
- `PROGRESS.md` and CLAUDE.md still contain stale headline counts and should be
  corrected in a documentation cleanup batch.

**Drift class**: stale orientation prose, not runtime capability mismatch.

**Current status**: mitigated by automated parity checks. Remaining cleanup is
to update human-facing prose and add deeper behavior-level live tests for the
CLI-bridged MCP methods if they become high-risk user entry points.

---

## 4. CLI Command Surface Drift

**Code reality** (`cli_args_commands.rs`, `cli_args_project_commands.rs`):
- 9 top-level commands
- **182 `project` subcommand variants** covering:
  - Native project lifecycle (`new`, `inspect`, `validate`, `query`)
  - Manufacturing: BOM/PnP/drill export+validate+compare+inspect (each format), Gerber outline/copper/soldermask/silkscreen/paste/mechanical export+validate+compare (per layer), manufacturing set report/export/inspect/validate/compare/manifest, Excellon variant, Gerber export plan
  - Forward annotation: artifact export/inspect/validate/compare/filter/plan/apply/import-review/replace-review, action defer/reject/clear, audit query/export, action apply, reviewed-apply
  - Routing: route-proposal (select/explain/export/apply-selected), route-apply (per candidate), route-path-proposal export, route proposal artifact (inspect/revalidate/apply), route-strategy report/compare/delta/batch-evaluate/inspect/validate/compare/gate/summarize, curated fixture suite writer, curated baseline capture, review-route-proposal
  - Schematic authoring: 30+ subcommands (place/move/rotate/mirror/delete symbol, place/rename/delete label, draw/delete wire, place/delete junction, place/edit/delete port, create/edit bus, place/delete bus entry, place/delete noconnect, set-symbol-*/clear-symbol-*, set-pin-override, add/edit/delete symbol field, place/edit/delete text, place/edit/delete drawing primitives)
  - Board authoring: place/edit/delete board text, board keepout, board outline, stackup, board net, draw/delete track, place/delete via, place/delete zone, set/clear pad net, edit/place/delete pad, place/move/rotate/delete board component, lock/unlock component, set component reference/value/part/package/layer, place/edit/delete net class, place/edit/delete dimension

**Documented as**:
- `PROGRAM_SPEC.md` M2 CLI table: 8 commands (`import`, `query --nets`, `query --components`, `query --summary`, `erc`, `drc`, `pool search`, `check`)
- `PROGRESS.md` M2 CLI table: same 8 + exit codes row
- M4/M5/M6/M7 charter shards reference subsets but no consolidated table exists
- No spec inventories the schematic-authoring or board-authoring subcommand sets

**Drift class**: enumeration drift — the spec lists ~5% of the command surface. There is no single document a contributor can read to know what CLI commands exist.

---

## 5. Undocumented or Under-Documented Subsystems

### 5.1 Text Engine

**Code reality**: `crates/engine/src/text/` — 11 files implementing a Newstroke-equivalent stroke font with mesh tessellation:
- `backend.rs`, `determinism.rs`, `geometry.rs`, `layout.rs`, `mesh.rs`, `newstroke_data.rs`, `outline.rs`, `registry.rs`, `semantic.rs`, `stroke.rs`, `mod.rs`

Plus consumer-side docs (`docs/gui/DATUM_TEXT_ENGINE_FIDELITY_FIXTURES.md`, `DATUM_TEXT_ENGINE_PHASE_2_BRIEF.md`, `DATUM_TEXT_ENGINE_PHASE_2_IMPLEMENTATION_PLAN.md`, `DATUM_TEXT_ENGINE_OUTLINE_FILL_*` notes, `DATUM_TEXT_ENGINE_OUTLINE_GEOMETRY_CONTRACT_NOTE.md`) and research (`research/pcb-text-rendering/`).

**Documented as**: nothing in `specs/`. `ENGINE_SPEC.md` does not mention a text engine.

**Drift class**: missing subsystem spec — a substantial deterministic-by-design engine subsystem with multi-phase research backing has no entry in the controlling specs.

### 5.2 GUI Substrate

**Code reality**: ~23k LOC across three crates, defining:
- `gui-protocol`: `BoardReviewSceneV1`, primitives (Pad/Track/Via/Zone/ProposalOverlay/Review/ComponentGraphic/ComponentText/BoardGraphic/BoardText/GlyphMesh/Outline/Unrouted), hit-testing (`HitTarget`, `HitRegion`), shell layout, camera state, glyph mesh assets, **board-text-editing field enums** (`BoardTextBooleanField`, `BoardTextAlignmentField`, `BoardTextLineSpacingStep`, `BoardTextHeightStep`, `BoardTextRotationStep`, `BoardTextCycleField`), workspace backing, selection target
- `gui-render`: wgpu `Renderer`, `PreparedScene`/`RetainedScene`, `OffscreenRenderer`, visual regression harness (`visual_capture`, `visual_diff`, `visual_manifest`, `visual_runner`), `datum_visual_fixture` binary
- `gui-app`: winit application

**Documented as**:
- `m7_opening.md` describes a "narrow read-only route-proposal review layer"
- `M7_FRONTEND_SPEC.md` defines `board_review_scene_v1` and a three-column shell
- `M7_IMPORTED_BOARD_FIDELITY_*` plan/checklist/issues/artifacts/fixtures notes (5 docs)
- `M7_RENDER_SEMANTIC_CONTRACT.md`, `M7_RENDER_LAYER_DISCIPLINE_MEMO.md`, `M7_ROUTE_REVIEW_SCREEN_REDESIGN.md`, `M7_DECISION_PROPOSALS.md`, etc.

**Drift class**: framing — the M7 charter is honest about the *intended* opening scope but the code already implements editing primitives (board text field cycle enums are an editing affordance, not a review affordance). The spec needs to recognize editing has begun, even if read-only-review is still the user-facing entry point.

### 5.3 Visual Regression Harness

**Code reality**: `crates/gui-render/src/visual_*.rs` plus `bin/datum_visual_fixture.rs` — full offscreen render + image diff + manifest + report pipeline consumed by GUI tests.

**Documented as**: working notes in `docs/gui/DATUM_GUI_VISUAL_REGRESSION_HARNESS.md` and `docs/gui/M7_IMPORTED_BOARD_FIDELITY_FIXTURES.md`. Not surfaced in `PROGRESS.md` or `TEST_STRATEGY.md`.

**Drift class**: testing infrastructure not surfaced in test-strategy authority.

### 5.4 Forward-Annotation Lifecycle

**Code reality**: 9 CLI subcommands (audit query/export, action apply, reviewed-apply, artifact export/inspect/validate/compare/filter/plan/apply, import/replace-review, defer/reject/clear). 9 dedicated `command_project_forward_annotation_*.rs` files in CLI.

**Documented as**: `PROGRESS.md` M4 table has one row "Forward annotation" marked `[x]` pointing to `m4_details.md`. The full artifact lifecycle (defer/reject/import/replace/filter/plan/select) is not enumerated.

### 5.5 Manufacturing / Inventory / Pool-Materialization

**Code reality**: per-layer Gerber export+validate+compare+inspect, Excellon drill variant, manufacturing set, inventory rendering, pool materialization, default stackup, gerber plan compare.

**Documented as**: M4 table has 4 rows (Gerber, Drill, BOM, PnP). The full surface (per-layer parity, compare/validate/inspect, manifest, manufacturing set, plan compare) is not enumerated.

---

## 6. CLAUDE.md Specific Drift Items

Line-anchored items in the project's primary orientation doc:

| Line | Claim | Reality |
|------|-------|---------|
| 12 | "M6 (layout strategy + AI layer) is in active development" | M6 is "frozen pending evidence" per its own charter; M7 is opened and is the active milestone |
| 15 | "**Completed capabilities (M0–M5):**" header | OK |
| 19 | "26+ MCP tools" | Catalog has 75; ~53 are daemon-backed |
| 24–25 | "Deterministic routing kernel: 60+ path candidate strategies" | Verifiable but not enumerated; the candidate family CLI files total 104 — claim may be conservative |
| 32–37 | "**In progress (M6):**" block — strategy reporting, batch eval, MCP parity | All landed and the M6 charter is now "frozen" |
| 39 | "**Future:** M7 (GUI), M8 (professional features)" | M7 is current, not future |
| 41 | "## Architecture: Engine-First" ASCII diagram shows 4 boxes (MCP server, GUI future, Engine, CLI) | OK as abstraction but elides Python scripting (mentioned in body) and the now-present GUI crates |
| 151–157 | `crates/` tree lists 4 crates | Actually 7 (`gui-protocol`, `gui-render`, `gui-app` missing) |
| 167 | "├── mcp-server/             # MCP server (Python, talks to engine via IPC)" | OK |
| 182–185 | "## Not Yet Implemented: GUI editor (M7); 3D viewer or STEP export (M8); Panelization, supply chain, impedance solver (M8)" | GUI editor partially implemented; STEP/ODB++/IPC-2581 export stubs were just added in Standards Batch 1 (spec only); controlled-impedance spec stub added in Standards Batch 1 |

**Recommendation**: CLAUDE.md is the highest-leverage doc for a fresh session (mine or human). It should be the **first** corrected, before deeper specs, because it sets the orientation frame everything else reads through.

---

## 7. Standards Audit Batch 1 — No Implementation Status Tracker

Commit `db98eff` (2026-04-17) landed 16 spec edits across `ENGINE_SPEC.md`, `POOL_ARCHITECTURE.md`, `NATIVE_FORMAT_SPEC.md`, `MCP_API_SPEC.md`. The edits define:
- Spec types: `Transform3D`, `ModelFormat`, `ModelRef`, `ModelProvenance`, `ModelAttachment`, `ModelRole`, `SpiceDialect`, `EncryptionScheme`, `ModelFormatMetadata`, `ImpedanceSpec`, `ThermalSpec`, `JEP106ManufacturerId`, EIA-481 packaging metadata
- Spec ops: `AttachModel`, `DetachModel`
- Pool structure: `pool/models/{ibis,spice,touchstone,ami,thermal}/` content-addressed by SHA-256, SQL index tables
- MCP tool stubs: `export_step`, `export_idf`, `export_odbpp`, `export_ipc2581`, `import_dxf_outline`, plus IBIS/SPICE/Touchstone attach/validate/extract, `export_spice_netlist`, `export_ibis_stimulus`, Octopart/Digi-Key/Mouser lookups, `find_alternate_parts`, `infer_diffpair_from_pinnames`
- Policy: encrypted-content handling

**No implementation status row exists** for any of these in `PROGRESS.md`. The next implementation pass on any of them will not have a status anchor.

**Drift class**: tracker-gap for newly-introduced spec stubs.

---

## 8. Working-Tree State (Pre-Merge)

`git diff --stat HEAD` shows **+9,308 / −1,555 across 79 files** not yet committed. Largest deltas:
- `gui-protocol/src/lib.rs` +3,486
- `gui-render/src/lib.rs` +2,808
- `gui-app/src/main.rs` +1,461
- `engine/src/export/silkscreen.rs` +204
- `cli/src/command_project_board_layout.rs` (new file, +189)
- `engine/src/import/kicad/parser_helpers.rs` +128
- `cli/src/main_tests_project_board_text.rs` (new file, +133)
- `specs/STANDARDS_COMPLIANCE_SPEC.md` +107
- `engine/src/import/kicad/mod.rs` +97
- ~30 routing test files: small uniform deltas (looks like a test-harness signature refactor)

This is in-flight, not drift per se. It's noted here because the "PROGRESS updated when code changes" claim at `PROGRESS.md` line 4 means the doc-correction process should kick in **at merge**, not as a post-hoc heroic catch-up. The current PR-less single-author flow needs an explicit pre-merge gate.

---

## 9. Gate Proposals — What Would Have Caught Each Drift

Each row maps a drift class above to an automated check. Existing scripts in `scripts/check_*.py` are noted where they already partially address the gap.

| # | Drift class | Existing partial gate | Proposed gate |
|---|-------------|------------------------|---------------|
| G1 | Crate inventory drift (§1) | none | Parse `Cargo.toml` workspace members; assert every crate appears in `CLAUDE.md` ASCII tree and `PROGRESS.md` Infrastructure row |
| G2 | Engine API drift (§2) | none | Parse `pub fn` names from `crates/engine/src/api/**/*.rs`; assert every name appears in `ENGINE_SPEC.md` §5.1 (or in an explicit deferred-list) |
| G3 | MCP catalog ⊆ daemon dispatch (§3) — **highest severity** | `check_progress_coverage.py` enforces catalog ↔ server_runtime; does not enforce catalog ↔ daemon | Add `parse_daemon_methods() ⊇ parse_tool_catalog_methods()` assertion; fail CI when an MCP tool has no daemon match arm |
| G4 | CLI subcommand enumeration (§4) | none | Parse `ProjectCommands` enum variants from `cli_args_project_commands.rs`; auto-generate a parity table in `PROGRESS.md` (or assert each variant appears in a `Spec` table) |
| G5 | Undocumented subsystems (§5) | none | Maintain an explicit "subsystem registry" in `ENGINE_SPEC.md` and assert every `crates/engine/src/<dir>/` with `mod.rs` is registered |
| G6 | CLAUDE.md milestone framing (§6) | none | Date-anchor every "in progress" / "completed" claim in CLAUDE.md to a milestone state; assert against `PROGRESS.md` headline |
| G7 | Standards Audit tracker gap (§7) | none | When a commit touches `specs/*.md` introducing new spec types or MCP tool sections, require a corresponding row in `PROGRESS.md` under a "Spec stubs awaiting implementation" section |
| G8 | Pre-merge doc-update gate (§8) | none | Pre-commit/pre-push hook: if `crates/cli/src/cli_args_*.rs`, `crates/engine/src/api/**/*.rs`, `crates/engine-daemon/src/dispatch.rs`, or `mcp-server/tools_catalog_data.py` changed and `PROGRESS.md` did not, fail with a clear "update PROGRESS.md or add deferred-doc row" message |

**Implementation order suggested**: G3 first (highest severity — capability/transport gap), then G1+G6 (orientation surface), then G2+G4 (enumeration surfaces), then G7+G8 (process), then G5 (subsystem registry — requires a one-time registration pass).

All gates are realistic extensions of `scripts/check_progress_coverage.py` (already 100+ lines of similar parsing) wired into `.github/workflows/alignment.yml` (already the CI integration point).

---

## 10. Doc-Correction Batches (Suggested Authorization Scopes)

So you can authorize doc edits in scoped chunks rather than a single sweep:

| Batch | Scope | Files touched | Risk |
|-------|-------|---------------|------|
| B1 | CLAUDE.md orientation: crate count, milestone framing (M7 active not M6), MCP tool count, "Not Yet Implemented" list | `CLAUDE.md` only | Low — orientation only, no spec contract changes |
| B2 | PROGRESS.md headline status: crate count row, MCP tool count, current-milestone marker, add "M7 (active)" and "Standards Audit Batch 1 (spec landed, impl deferred)" sections | `PROGRESS.md` only | Low |
| B3 | PROGRESS.md surface tables: replace M2-only CLI table with current 9-command top-level + collapsible "project subcommand inventory" pointing to an auto-generated table | `PROGRESS.md`, possibly new `specs/CLI_SURFACE_INVENTORY.md` | Medium — enumerative; benefits from G4 first |
| B4 | ENGINE_SPEC §5.1 refresh: parse `pub fn` and align table | `ENGINE_SPEC.md` | Medium — table maintenance |
| B5 | New subsystem entries: text-engine subsystem section in ENGINE_SPEC; GUI substrate section acknowledging editing primitives | `ENGINE_SPEC.md`, possibly new `specs/TEXT_ENGINE_SPEC.md` and `specs/GUI_SUBSTRATE_SPEC.md` | Higher — introduces new spec contracts; needs your design input |
| B6 | M7 charter reframe: acknowledge editing primitives exist in `gui-protocol`; either expand "opening slice" definition or split into "M7 review" + "M7 editing affordances landing" | `specs/progress/m7_opening.md`, `PROGRESS.md` M7 section | Higher — milestone redefinition |
| B7 | Standards Audit Batch 1 tracker: add implementation-status section in `PROGRESS.md` for each spec stub | `PROGRESS.md` | Low |
| B8 | MCP_API_SPEC.md: mark the 22 catalog tools missing daemon dispatch with explicit "implementation status: catalog-only, daemon dispatch pending" until G3 is added and the daemon catches up | `specs/MCP_API_SPEC.md` | **Recommended before any further MCP tool addition** — prevents capability/transport gap from widening |

---

## 11. Open Questions For The User

1. **Where should this report live long-term?** Currently written to `docs/audits/scope-integration/DOC_CODE_PARITY_DELTA_REPORT.md` to sit alongside `STANDARDS_AUDIT_BATCH_1_GUIDANCE.md`. Move if you prefer.
2. **G3 verification**: should I actually attempt to call one of the 22 missing tools through the daemon to confirm "method not found", before stating it as definitive in the spec correction? Currently asserted on dispatch.rs match-arm analysis only.
3. **Authorization batching**: which of B1–B8 do you want first? B1+B2 are the safest opener — pure orientation fixes with no contract changes.
4. **Gate sequencing**: do you want G3 (catalog ⊆ daemon) before or after the doc corrections? Building the gate first means the corrections land green; deferring the gate means we know corrections are accurate but new drift can still creep in until the gate exists.
5. **Charter philosophy**: the opening charters in `specs/progress/m*_opening.md` claim to be frozen historical snapshots of opening scope. Do you want them treated as immutable history (no edits) with all drift moved to `PROGRESS.md`, or should the charters be reframed to current state?
