# Project architecture and historical context

Status: historical context extracted from the former startup instructions on
2026-10-08. This is not current roadmap or implementation authority. Consult the
fresh selector and current domain contracts before using old status claims.
The product doctrine remains in CLAUDE.md and numbered decisions.

## Current Status
### WDQ blocking enforcement retired

The owner retired WDQ blocking enforcement on 2026-09-17 (PM044 retirement
disposition). Use `scripts/git-hooks`, not an immutable WDQ owner-hook bundle.
Do not require WDQ enrollment, source-scope promotion or historical-pilot replay
for ordinary development or roadmap queries. Retained WDQ tooling and records
are manual diagnostics/history only; do not reinstall them without new explicit
owner direction. PM025 ownership and roadmap checks, source health, dependency
authority, visual ownership, formatting and product acceptance requirements remain.

The project has been course-corrected from a milestone-driven roadmap to the
product-mechanics model above. Status sources of truth: the structured
`specs/active_frontier.json`, its generated `specs/PROGRESS.md` projection,
`specs/SPEC_PARITY.md` (machine-checked inventory shapes), and the
product-mechanics docs.

- **Active product area:** GUI product-model recovery toward a testable program
  (decision 019) — the recovered application shell, board-render fidelity, and
  the GUI→engine write path that lets menu/editor actions author typed operations
  directly (not terminal CLI strings). This is product context, not task
  ordering; use `python3 scripts/project_status.py next` for the current task.
  Its foundation, the product-mechanics
  **substrate** (typed `Operation` enum + single `commit()` + journal +
  `ProjectResolver` + stable `ObjectId`/`ComponentInstance` +
  `model_revision` + Import Map), is now the **universal native write
  authority**: the engine native-write facade
  (`crates/engine/src/api/native_write/`, 11 families) authors every native
  operation batch; the CLI is a thin args/dispatch/views surface with zero
  operation authoring; the daemon reaches the substrate through
  `native.write`/`native.describe`; genesis is engine-owned. See
  `specs/PROGRESS.md`.
- **Post-correction sequence (committed):** substrate (complete as the native
  write authority) → library →
  native authoring + GUI surface. The GUI build-out is a named, real phase of
  this sequence — **not** an implied "M8 later".
- **Terminal product boundary:** Product Mechanics 027–030,
  `docs/gui/DATUM_NATIVE_TERMINAL_SPEC.md`, and
  `docs/gui/DATUM_TERMINAL_AGENT_INTEROP_SPEC.md` control the embedded terminal
  and its agent interoperability plane. The clean-room standards baseline and
  exhaustive first-party package DAG are
  `docs/gui/DATUM_TERMINAL_CORE_RESEARCH.md` and
  `docs/gui/DATUM_TERMINAL_CORE_IMPLEMENTATION_PLAN.md`. The
  target is a fully fledged daily-driver terminal over a pinned mature
  `TerminalCore`, with exclusive PTY screen truth and verified support for
  normal shells, TUIs, Codex, Claude Code, Cursor-compatible CLI agents, and
  local agents. TF-01..TF-05 cover focus/input only; a focus fix, parser, PTY,
  cell grid, or output-only Datum Console must never be reported as completion of the
  terminal product. Delivery is T0 shell truth → T1 core/transport → T2 native
  renderer/input → T3 daily-driver UX/protocols → T4a–T4d discovery/MCP/
  context/workflow interoperability → T4e production proof. An inherited
  environment variable or suggested launch prompt is not agent discovery.
- **Frozen:** KiCad import. The M7 spike already imports a board with enough
  fidelity to recognize all design aspects; that is sufficient — no further
  import work until native authoring is real. Native is always the authority;
  import is a one-time converter and "imported vs native" is not a state. See
  `docs/DATUM_PRODUCT_MECHANICS.md` → "Interop Boundary And Import Posture".
- **Frozen:** M6 strategy reporting (landed; pending repeated evidence runs
  from the checked-in baseline gate).
- **Closed for scope:** M0–M5.
- Legacy milestone rows in `specs/PROGRESS.md` are truthful historical
  evidence, not the next implementation priority.

### What has landed (historical evidence, M0–M6)
Real, shipped capability — read alongside the substrate gap below:
- Canonical IR, pool foundation, deterministic KiCad import
- Query engine over imported and native designs
- ERC (7 rules) + DRC (7 rules) at 0% FP/FN quality gates
- MCP runtime tools (daemon-dispatched + CLI-bridged, locked via
  `specs/SPEC_PARITY.md`); CLI with proper exit codes
- Imported-board write operations with undo/redo (engine API write-ops)
- KiCad write-back with sidecar persistence and round-trip fidelity
- Manufacturing export: Gerber, Excellon drill, BOM, pick-and-place
- Deterministic routing kernel (60+ path-candidate strategies) and
  route-proposal artifact export/apply/inspect/revalidate
- GUI substrate (`gui-protocol`/`gui-render`/`gui-app`): read-only board
  review surface, engine supervision/status snapshot, Outputs-lane
  supervision rendering, and visual regression harness

### Known gap between status and ethos (do not overstate)
Write-surface convergence is COMPLETE — do not resurrect the old
"convergence-debt" framing. The honest remainder:
- **Legacy converter session (terminal, not debt):** the imported-KiCad
  session keeps 4 legacy mutator methods + `Engine::save` with a private
  in-memory `ImportedSessionUndoRecord` memo (never journaled); terminally
  frozen, dies with the one-time converter (decision 011).
- **Tier A substrate close-out:** `component_instance` bind/set/delete have
  governed proposal twins (`datum.proposal.bind_component_instance`,
  `.set_component_instance`, `.delete_component_instance`), so assistant
  provenance can create proposals instead of direct writes. Genesis t=0 is
  ratified by decision 018: no mutation-journal record; future visible genesis
  evidence must be a non-mutation sidecar.
- **Verb registry migration complete for public `datum.*` tools (decision 017):**
  all 17 public prefixes are registry-generated (337/337 public tools);
  hidden compatibility tools remain fenced outside the public generated surface.
- **Library:** decision-008 `Footprint`/`PinPadMap` are now engine Rust types
  and `LibraryBinding` is now the component-instance join type for placed
  part/symbol/package/footprint/pin-pad-map/model bindings. IPC generation now
  covers two-terminal chips and SOIC as engine-owned vertical slices with CLI,
  proposal, MCP, and LibraryGraph validation coverage; broader package-family
  coverage plus check-run/deviation/export/import-audit consumption remains
  future work.
- **GUI:** a review/supervision surface, not an editor; native board scenes and
  the Outputs lane reflect resolver/materialized engine truth, but interactive
  authoring (the user selecting a tool and drawing/placing/editing through a
  direct GUI commit path) is not yet built.

The per-domain tool contracts in `docs/contracts/` specify the target each
domain builds toward on the landed substrate.


## Architecture: Engine-First

```
                    ┌─────────────┐
                    │  MCP Server  │  ← AI agents
                    └──────┬──────┘
                           │ JSON-RPC / Unix socket
    ┌──────────┐    ┌──────┴──────┐    ┌──────────┐
    │   GUI    │    │   Engine    │    │   CLI    │
    │          │───→│   (Rust)    │←───│ datum-eda│
    └──────────┘    └─────────────┘    └──────────┘
                           │
                    ┌──────┴──────┐
                    │   Python    │  ← scripting (PyO3)
                    └─────────────┘
```

The engine is a Rust library with no GUI or rendering dependencies. The
target authority is one resolved `DesignModel` assembled by an engine-owned
resolver; every surface (GUI, CLI, MCP, scripting) operates on that model
through the same operation/commit model. The GUI consumes the engine via the
`gui-protocol` scene contract; it is a consumer, not a prerequisite.

## Architectural Lineage
Informed by study of three existing tools (not forked from any):
- **Eagle 9.6.2** — Command-line philosophy, part model, file format DTD
  (docs/EAGLE_BLUEPRINT.md)
- **Horizon EDA** — Pool architecture, entity hierarchy, data model principles
  (docs/HORIZON_ANALYSIS.md)
- **Altium Designer** — Professional UX, rule query language, routing workflow
  (docs/ALTIUM_LESSONS.md)


## Key Technical Decisions

| Layer | Choice | Rationale |
|-------|--------|-----------|
| Engine | Rust | Memory safety, no GC, modern toolchain, cargo ecosystem |
| Authority | One resolved `DesignModel` over segmented storage shards | Unified authority, projections not silos; git-reviewable persistence (see docs/decisions/PRODUCT_MECHANICS_000D) |
| Data model | Canonical IR (docs/CANONICAL_IR.md) | Authored/derived separation, stable surrogate IDs, invariant-first |
| Mutation | Typed operations → one `commit()` + journal | Single inspectable path for manual/CLI/MCP/AI; provenance, diff, undo |
| Pool | SQLite + JSON files | Queryable index, human-readable source |
| Design files | JSON shards (imported formats as first-class) | Diffable, AI-parseable |
| MCP server | Python | MCP SDK, AI library ecosystem |
| CLI | Rust (`datum-eda`, same engine crate) | Batch ops, CI/CD, scripting |
| Scripting | Python via PyO3 | Universal, AI ecosystem |
| Layout engine | Custom constraint-formalized placement + routing with AI policy layer | No PNS dependency; classical algorithms in a formal constraint pipeline. See docs/LAYOUT_ENGINE.md |
| GUI | wgpu + winit + custom (`gui-protocol`/`gui-render`/`gui-app`) | Engine consumer; review surface today, editor is target |
| Tooling | Lean shared tool surface (docs/contracts/) | One operation vocabulary across UI/CLI/MCP; redundant tools are defects |

## Repository Layout
```
project/
├── CLAUDE.md               # This file
├── PLAN.md                 # Development roadmap
│
├── docs/
│   ├── DATUM_PRODUCT_MECHANICS.md  # CONTROLLING product-mechanics doctrine
│   ├── decisions/          # PRODUCT_MECHANICS_000..030 — ratified mechanism
│   │                       #   decision records (what + why + how)
│   ├── contracts/          # Per-domain tool-contract implementation specs:
│   │                       #   schematic/PCB/library/rules/manufacturing +
│   │                       #   shared AI_CLI_MCP_TOOL_SURFACE.md
│   ├── audits/scope-integration/  # Historical audit/evidence: north-star
│   │                       #   audit, readiness audits, research synthesis,
│   │                       #   review agenda, doc/code parity delta
│   ├── PRODUCTION_CLEANUP_MANIFEST.md  # Worktree classification for commits
│   ├── SPEC_INTEGRATION_CONDUCTOR_REPORT.md
│   ├── gui/                # GUI substrate, text engine, visual harness briefs
│   ├── CANONICAL_IR.md     # Core data model: invariants, IDs, units, txns
│   ├── ENGINE_DESIGN.md    # Operation model, API surface, rule engine
│   ├── CHECKING_ARCHITECTURE.md  # ERC/DRC separation, shared reporting
│   ├── ERC_SPEC.md / SCHEMATIC_CONNECTIVITY_SPEC.md
│   ├── AUTHORING_TOOLS.md  # Tool semantics: place, wire, move, route, delete
│   ├── POOL_ARCHITECTURE.md / LIBRARY_ARCHITECTURE.md / NATIVE_FORMAT.md
│   ├── LAYOUT_ENGINE.md / INTEROP_SCOPE.md / MCP_DESIGN.md
│   ├── IPC_FOOTPRINT_SYSTEM.md / STANDARDS_*_GUIDANCE.md
│   ├── EAGLE_BLUEPRINT.md / HORIZON_ANALYSIS.md / ALTIUM_LESSONS.md
│   ├── COMPETITIVE_ANALYSIS.md / COMMERCIAL_INTEROP_STRATEGY.md
│   ├── FFI_BOUNDARY.md / LICENSING.md / RISK_REGISTER.md / TEST_STRATEGY.md
│   ├── IMPLEMENTATION_PLAN.md / IMPLEMENTATION_GUARDRAILS.md
│   ├── DECOMPOSITION_PLAN.md / DECOMPOSITION_BACKLOG.md / STABILIZATION_PLAN.md
│   ├── RESEARCH_TRACEABILITY.md / USER_WORKFLOWS.md
│   └── (legacy: M0/M1 checklists, R1_G0_FOUNDATION.md, workflows/)
│
├── crates/
│   ├── engine/             # Core engine; src/api/native_write/ facade
│   ├── cli/                # CLI `datum-eda` (crate datum-eda-cli): args/ +
│   │                       #   commands/<family>/ + context/ + main_tests/
│   ├── engine-daemon/      # JSON-RPC daemon (native.write/native.describe)
│   ├── verb-registry/      # Single-source verb registry (decision 017)
│   ├── test-harness/       # Perf/quality harnesses and helpers
│   ├── gui-protocol/       # Scene contract + primitive types (no GUI deps)
│   ├── gui-render/         # wgpu renderer + visual regression harness
│   └── gui-app/            # winit shell for the GUI
├── mcp-server/             # MCP server (Python, talks to engine via IPC)
├── specs/                  # Controlling formal specifications
│   ├── active_frontier.json # Canonical scheduled roadmap + next-task state
│   ├── PROGRESS.md         # Generated Frontier + implementation evidence
│   ├── SPEC_PARITY.md + spec_parity_manifest.json  # Code-derived inventory
│   │                       #   digests (gated by check_spec_parity.py)
│   ├── PROGRAM_SPEC.md / INTEGRATED_PROGRAM_SPEC.md / ENGINE_SPEC.md
│   ├── NATIVE_FORMAT_SPEC.md / IMPORT_SPEC.md / MCP_API_SPEC.md
│   ├── SCHEMATIC_EDITOR_SPEC.md / M7_FRONTEND_SPEC.md
│   ├── CHECKING_ARCHITECTURE_SPEC.md / ERC_SPEC.md
│   ├── SCHEMATIC_CONNECTIVITY_SPEC.md / STANDARDS_COMPLIANCE_SPEC.md
│   └── progress/           # Per-milestone shards (historical)
├── scripts/                # Validation and governance gates
│   ├── check_spec_parity.py / check_alignment.py / check_spec_governance.py
│   ├── check_progress_coverage.py / check_schematic_private_writers.py
│   ├── check_daemon_write_parity.py / check_resolver_raw_loads.py
│   ├── check_import_query_determinism.py / check_native_project_fixtures.py
│   └── run_drift_gates.sh  # Drift gate runner (+ run_migration_proof_gates.sh)
├── .github/                # CI/CD (alignment.yml) and PR template
├── tests/corpus/           # Real designs for golden testing
└── research/               # Analysis artifacts (gitignored working notes)
```

## Not Yet Implemented
The authoring frontier (see `docs/contracts/` for target specs); the substrate
is landed and authoritative — what remains is depth on top of it:
- Richer semantic schematic/library editors, rule reference-resolution
  semantics, and broader proposal/check operation vocabulary.
- Full IPC footprint-system depth beyond the landed IPC-7351B two-terminal and
  SOIC slices: broader families, deviation/check-run/export consumption, and
  import-audit integration.
- Full GUI editor (substrate + read-only review exist; interactive editing is
  not exposed end-to-end). The current PTY lane is not yet the decision-027
  fully fledged terminal and must not be described as one. Product-real
  assistant.
- Native copper-pour zone fill (imported fills only today).
- 3D viewer, panelization, STEP/IDF/ODB++/IPC-2581 export — spec stubs landed
  in Standards Audit Batch 1, implementation deferred.
- Supply-chain lookups (Octopart/Digi-Key/Mouser) and behavioral-model attach
  (IBIS/SPICE/Touchstone) — spec stubs landed, implementation deferred.
- Controlled-impedance solver — `ImpedanceSpec` stub landed, solver deferred.
