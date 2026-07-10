# Datum EDA

## What This Is
Datum is a professional, headless-first PCB design application for Linux,
with optional first-class AI collaboration. It imports existing designs,
inspects and transforms them, runs ERC/DRC, authors native schematics and
boards, generates manufacturing output, and routes boards — driven through
one inspectable design-mutation model that manual GUI tools, the CLI,
scripts, MCP, and AI agents all share.

Datum is **not** a KiCad importer, a board viewer, an AI-only design agent,
or a visualization shell around imported EDA files. Import/export are interop
infrastructure (migration, fixtures, compatibility, reverse engineering) —
they support the product; they do not define its identity.

## Ethos (controlling)
- **Manual-first.** Every core EDA workflow must be possible without AI. If a
  user cannot perform a workflow manually, it is not complete — even if an
  agent can perform or fake it. AI may assist, propose, explain, automate,
  and review; it is never required and never a hidden authority.
- **Optional first-class AI.** AI is a collaborator that operates through the
  same deterministic primitives a user does — stable IDs, queries, typed
  operations, proposals, checks, diffs, provenance — never private edit powers.
- **One canonical `DesignModel`.** Schematic, PCB, library, rules,
  manufacturing, and analysis are projections over one model assembled by an
  engine-owned resolver — not separate authorities. Files are persistence
  partitions; the resolved model is the authority.
- **Product North Star.** Datum is a professional native EDA system whose
  foundation is governed library plus schematic authority. The normal product
  flow is library -> schematic -> PCB -> manufacturing; import/export are
  compatibility paths, not the center of the roadmap.
- **One mutation path.** Every committed design change — manual, CLI, MCP,
  agent, import-repair, check-fix — reduces to typed operations through one
  `commit()` + journal, with provenance, diff, and undo. No private writers.
- **Lean.** Keep Datum clean and efficient: a capability should be a
  parameter of a small verb set, not a tool per object-class-times-format. A
  redundant tool is a defect.

> Controlling product doctrine lives in `docs/DATUM_PRODUCT_MECHANICS.md`, the
> ratified decision records in `docs/decisions/` (`PRODUCT_MECHANICS_000..022`),
> and the per-domain tool contracts in `docs/contracts/`. Read those before
> inferring product intent from code or a milestone.

## Specification Governance (controlling)
Core principle: **no orphaned specs — in coverage OR in sequence.** Every active
specification, contract, or decision record must be (a) tracked and classified,
and (b) placed in the ordered roadmap so its position, dependencies, and next
action are derivable. A doc that exists but is not tracked is a governance
defect; a doc that is classified but not woven into the roadmap frontier is a
**roadmap orphan** — equally a defect, and the specific failure this project has
repeatedly paid course-correction time for. Writing a spec is not finished until
the next agent can find where it sits and what comes next.

When you CREATE or REFINE any spec / contract / decision record, in the SAME
change you MUST:
1. Add or update its status row in `specs/PROGRESS.md` (current vs target).
2. Classify it in `specs/spec_governance_manifest.json`
   (`class`: `governed` / `doctrine` / `pending` / `historical`), enforced by
   `check_spec_governance.py` (coverage + classification).
3. Register any inventory shapes it defines in
   `specs/spec_parity_manifest.json` (enforced by `check_spec_parity.py`).
4. If it ratifies mechanism, it must be a numbered decision record in
   `docs/decisions/`, not a loose doc or an `OPEN_QUESTION_RESOLUTIONS` entry.
5. Weave it into the **Active Frontier** at the top of `specs/PROGRESS.md`: give
   it an ordered position with its dependency (what must land first), what it
   unblocks, and its state. A spec describing future work that does not appear in
   the Active Frontier is not done being written.

**Roadmap wayfinding (the bullseye rule).** The single canonical answer to
"what is the next logical development step / specification?" is the **Active
Frontier** at the top of `specs/PROGRESS.md` — an ordered, dependency-aware list
whose every entry links to its governing spec/decision. A freshly spawned agent
must be able to read that section *alone* and return the correct next step.
Before proposing or starting work, read the Active Frontier; before finishing any
spec or course-correction, update it. Roadmap intent lives in ONE place — the
Frontier links down to detail; other docs (including this file's "Current Status"
and PROGRESS's lower sections) must not restate the next-step ordering as a rival
copy that drifts. If your work will not place cleanly into the Frontier, the
sequencing is wrong — fix that before writing more prose.

Behavioral invariants are enforced by the PG-* proof gates
(`run_migration_proof_gates.sh`) and the write-fence gates
(`check_schematic_private_writers.py`, `check_daemon_write_parity.py`,
`check_resolver_raw_loads.py`, `check_spec_parity.py`). There is NO per-doc
enforcement ledger and no doc-string pinning gates: gates lock behavior in
code, not prose in docs.

Authority ordering (lower layers must not contradict higher ones; on conflict
the higher layer wins and the lower doc is the one to fix):
1. `CLAUDE.md` — operational instructions
2. `docs/DATUM_PRODUCT_MECHANICS.md` + `docs/decisions/` — product doctrine
3. `docs/contracts/` — domain tool contracts
4. `specs/PROGRESS.md` — status truth

## Current Status
The project has been course-corrected from a milestone-driven roadmap to the
product-mechanics model above. Status sources of truth: `specs/PROGRESS.md`,
`specs/SPEC_PARITY.md` (machine-checked inventory shapes), and the
product-mechanics docs.

- **Active focus:** GUI product-model recovery toward a testable program
  (decision 019) — the recovered application shell, board-render fidelity, and
  the GUI→engine write path that lets menu/editor actions author typed operations
  directly (not terminal CLI strings). **The authoritative, ordered next steps
  are the Active Frontier at the top of `specs/PROGRESS.md`; read that, don't
  re-derive the roadmap from this prose.** Its foundation, the product-mechanics
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

## Working Posture (for agents)
- **Source health is blocking governance.** Before touching an oversized source
  module, read decision 022 and `docs/SOURCE_HEALTH_POLICY.md`. New source debt
  cannot be registered into existence; legacy ceilings are exact and
  downward-only; touching legacy debt requires real ownership extraction and a
  same-change ratchet. `include!`/forwarding/continuation splits do not count.
- **Planning vs execution.** Default to high-level planning and specification
  (decision records, tool contracts, sequencing). Feature execution —
  building engines, sourcing or authoring fixtures, running proof slices — is
  a separate phase: name it as execution and get explicit authorization
  before doing it. "Proof slice" / "proof gate" sections in the docs are
  specifications of what execution must later demonstrate, not to-dos.
- **Do not infer product identity** from the active milestone or the nearest
  import/render/regression task. Use the product-mechanics doctrine.
- No editing path may bypass the canonical operation/commit/journal model.
- A viewer or review surface is not an editor. **Engine-first/GUI-last** is
  deliberate only for the GUI *editor*: building the GUI editor (interactive
  authoring) is a real, named phase of the work, not an implied "M8 later".
- Prefer work that strengthens native EDA primitives and manual workflows.
- Do not expand import fidelity unless it supports a clear product need.
- **Screenshot to see your work — don't design blind.** To capture an HTML
  prototype or artifact (e.g. `docs/gui/prototypes/*.html`) to PNG, run
  `scripts/shot_html.sh <file> [out.png] [WxH]` (headless Firefox, throwaway
  isolated profile). Chromium SIGTRAPs in the sandbox — its multi-process sandbox
  needs namespace/seccomp syscalls the outer sandbox blocks, even with
  `--no-sandbox` — but Firefox/Gecko starts fine. The wgpu app screenshots via its
  own `--visual-test --screenshot-out --exit-after-screenshot` flags (see
  `scripts/check_gui_visual_parity.py`). Always capture and eyeball the render
  before showing the owner; you can Read the PNG to inspect it.

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

## Attribution Policy
Claude Code is engaged on this project as a paid service in a contractor
capacity. All code, architecture, and design work is authored by the project
owner. Claude Code is used strictly for research assistance and process
automation.

**No attribution of any kind is permitted in any commit, PR, comment, or
document in this repository.** This includes `Co-Authored-By` tags,
`Generated by` notes, or any other mechanism by which a commercial AI service
claims credit for work it did not author. Applying such attribution without
explicit instruction is a violation of standard professional contracting
ethics and is explicitly forbidden on this project.

## Project Principles
- Project name policy: use `Datum EDA` for product naming and `datum-eda` for
  machine identifiers (tool names, sockets, test generators, config keys). The
  canonical CLI executable is `datum-eda` (crate `datum-eda-cli`); the legacy
  `eda` binary name is historical only. Do not introduce any previous
  placeholder project name in code, docs, fixtures, tests, or configs.
- Datum must be fully usable without AI (manual-first).
- The engine compiles and runs without any GUI dependency.
- One canonical `DesignModel`; projections are not separate authorities.
- Every committed design change flows through one operation/commit model; no
  surface gets a private write path.
- Interactive behaviors (drag, route-in-progress, hover, selection) are
  consumer-specific — they produce operations, they are not operations
  themselves and are never journaled.
- Import fidelity is a quality gate, not the product center.
- Usefulness before authoring: inspect and transform before create.
- Canonical IR defines invariants, not just object shapes.
- Test against real designs from day one; never fabricate fixtures.

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
│   ├── decisions/          # PRODUCT_MECHANICS_000..022 — ratified mechanism
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
│   ├── PROGRESS.md         # Authoritative status tracker (current vs target)
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
- Full GUI editor (substrate + read-only review + PTY terminal lane exist;
  interactive editing not exposed end-to-end). Product-real assistant.
- Native copper-pour zone fill (imported fills only today).
- 3D viewer, panelization, STEP/IDF/ODB++/IPC-2581 export — spec stubs landed
  in Standards Audit Batch 1, implementation deferred.
- Supply-chain lookups (Octopart/Digi-Key/Mouser) and behavioral-model attach
  (IBIS/SPICE/Touchstone) — spec stubs landed, implementation deferred.
- Controlled-impedance solver — `ImpedanceSpec` stub landed, solver deferred.
