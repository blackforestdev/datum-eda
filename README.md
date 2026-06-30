# Datum EDA

Datum EDA is a professional, headless-first electronics design system for Linux:
a deterministic Rust engine core with **manual-first** workflows and **optional
first-class AI** collaboration, all driven through one inspectable
design-mutation model that the GUI, CLI, scripts, MCP, and AI agents share.

Datum is an independent, native EDA system — not a wrapper around another tool.
Its product center is **native governed libraries**, **schematic capture** as the
normal electrical source of truth, PCB implementation from that authority, and
manufacturing-ready output from one resolved native model. The normal product
flow is **library → schematic → PCB → manufacturing**.

KiCad-first support is an interop/migration on-ramp for fast, verifiable
delivery — it is **not the long-term product boundary**. Import and export are
compatibility infrastructure (migration, fixtures, reverse engineering); they
support the product, they do not define it.

---

## Why

Most EDA systems were built GUI-first and automation was layered in later.
Datum inverts that model:
- **Engine-first deterministic core** (Rust library crate, no GUI/rendering dep).
- **Manual-first.** Every core workflow is possible without AI.
- **Optional first-class AI / CLI / MCP.** They are collaborators operating
  through the same typed operations a user does — stable IDs, queries, proposals,
  checks, diffs, provenance — never private edit authorities.
- **One canonical `DesignModel`.** Schematic, PCB, library, rules, manufacturing
  are projections over one model. Every committed change — manual, CLI, MCP, AI,
  import-repair — flows through one `commit()` + journal with provenance, diff,
  and undo.
- **GUI as a downstream consumer**, not the system of record.

---

## Direction and scope

Product flow and roadmap sequencing follow one native authority:

- **Substrate** (landed): typed `Operation` + single `commit()` + journal +
  `ProjectResolver` + stable `ObjectId`/`ComponentInstance` + `model_revision` +
  Import Map.
- **Native library** → **schematic capture** → **constraint/ERC** → **PCB
  layout** → **manufacturing/CAM**.
- **Interop** (KiCad/Eagle import/export) runs alongside as a compatibility path
  throughout — never the maturity metric.

Controlling product doctrine: `docs/DATUM_PRODUCT_MECHANICS.md`, the decision
records in `docs/decisions/` (including `PRODUCT_MECHANICS_016_PRODUCT_NORTH_STAR`),
and the per-domain tool contracts in `docs/contracts/`. Status sources of truth:
[`specs/PROGRESS.md`](specs/PROGRESS.md) and `specs/SPEC_PARITY.md`.

Canonical scope terminology is defined in
[`specs/PROGRAM_SPEC.md`](specs/PROGRAM_SPEC.md) (`Product identity`,
`Implementation slice`, `Execution strategy`, `Non-goals`).

---

## What it can do today

Foundation (landed, committed):
- The canonical mutation substrate above, backing a growing share of native
  authoring; remaining write surfaces are converging onto it.
- ERC (7 rules) + DRC (7 rules) at 0% FP/FN quality gates.
- MCP runtime tools (daemon-dispatched + CLI-bridged) and a CLI with proper exit
  codes.

Native authoring:
- Native projects: `project new`, native query/check, forward-annotation
  review/apply, and manufacturing export (Gerber, Excellon drill, BOM,
  pick-and-place).
- Deterministic routing kernel (60+ path-candidate strategies) and route-proposal
  export/apply/inspect/revalidate.

Native library (in progress):
- Governed pool with `Unit`/`Symbol`/`Gate`/`Entity`/`Part` and a real
  `Footprint` land-pattern type; per-user KiCad/Horizon symbol import that
  normalizes into the native model (no third-party library content is bundled).

Interop / compatibility:
- Deterministic KiCad `.kicad_pcb` / `.kicad_sch` import, query, and check.
- Eagle `.lbr` library import into the pool.
- KiCad write-back with round-trip fidelity; imported-board modify slices with
  undo/redo.

GUI (engine consumer):
- Read-only board review surface + visual-regression harness. A Taffy-based
  layout system and a token-based design system are landing. The interactive
  editor is a named, in-progress phase — not yet end-to-end.

See [`docs/USER_WORKFLOWS.md`](docs/USER_WORKFLOWS.md) for usage examples,
[`specs/MCP_API_SPEC.md`](specs/MCP_API_SPEC.md) for the MCP tool catalog, and
[`specs/PROGRESS.md`](specs/PROGRESS.md) for status and limitations.

---

## Build

Requires Rust stable and a C linker. The GUI/visual features additionally need a
Vulkan-capable environment (software Vulkan/lavapipe is sufficient for headless
rendering).

```bash
cargo build
cargo test
```

Pre-merge drift gate (required for feature handoff):

```bash
scripts/run_drift_gates.sh
```

Run the engine daemon (required for MCP):

```bash
cargo run -p eda-engine-daemon -- --socket /tmp/datum-eda-engine.sock
```

---

## CLI quick start

```bash
# Import an Eagle library into the pool
cargo run -p datum-eda-cli -- import library.lbr

# Check imported KiCad designs (compatibility path)
cargo run -p datum-eda-cli -- erc design.kicad_sch
cargo run -p datum-eda-cli -- drc design.kicad_pcb

# Query a design
cargo run -p datum-eda-cli -- query design.kicad_pcb summary
cargo run -p datum-eda-cli -- query design.kicad_pcb nets

# Search the component pool
cargo run -p datum-eda-cli -- pool search "100nF 0402" --library library.lbr

# Native project
cargo run -p datum-eda-cli -- project new ./demo --name "Demo"
cargo run -p datum-eda-cli -- project query ./demo summary
```

Exit codes: `0` = pass, `1` = violations found, `2` = execution error. The CLI,
MCP, and GUI share one operation vocabulary; see `specs/MCP_API_SPEC.md` for the
current canonical surface.

---

## MCP server

The MCP server exposes the engine surface to AI agents over stdio.

```bash
# Start the engine daemon
cargo run -p eda-engine-daemon -- --socket /tmp/datum-eda-engine.sock

# Start the MCP server
python3 mcp-server/server.py
```

Example client registration:

```json
{
  "mcpServers": {
    "datum-eda": {
      "command": "python3",
      "args": ["/path/to/datum-eda/mcp-server/server.py"],
      "env": { "DATUM_ENGINE_SOCKET": "/tmp/datum-eda-engine.sock" }
    }
  }
}
```

`EDA_ENGINE_SOCKET` remains a legacy fallback for existing local configs.

---

## Architecture

```
                    ┌─────────────┐
                    │  MCP Server  │  ← AI agents (Python, stdio)
                    └──────┬──────┘
                           │ JSON-RPC / Unix socket
    ┌──────────┐    ┌──────┴──────┐    ┌──────────┐
    │   GUI    │───→│   Engine    │←───│   CLI    │
    │ (wgpu)   │    │   (Rust)    │    │ datum-eda│
    └──────────┘    └─────────────┘    └──────────┘
                           │
                    ┌──────┴──────┐
                    │   Python    │  ← scripting (PyO3)
                    └─────────────┘
```

The engine is a Rust library crate with no GUI/rendering dependency; the daemon
exposes it over Unix-socket JSON-RPC, the MCP server is a thin Python translation
layer, and the GUI consumes the engine through the `gui-protocol` scene contract.
Every surface operates on one resolved `DesignModel` through the same
operation/commit model.

Key documentation:

| Document | Contents |
|----------|----------|
| [`docs/DATUM_PRODUCT_MECHANICS.md`](docs/DATUM_PRODUCT_MECHANICS.md) | Controlling product-mechanics doctrine |
| [`docs/CANONICAL_IR.md`](docs/CANONICAL_IR.md) | Core data model and invariants |
| [`docs/ENGINE_DESIGN.md`](docs/ENGINE_DESIGN.md) | Operation model and API surface |
| [`docs/decisions/`](docs/decisions/) | Ratified product-mechanics decision records |
| [`docs/contracts/`](docs/contracts/) | Per-domain tool contracts |
| [`specs/PROGRESS.md`](specs/PROGRESS.md) | Implementation status per spec requirement |

---

## Roadmap

See [`PLAN.md`](PLAN.md) and the product-mechanics doctrine above. The program is
governed by the product-mechanics model and the ratified North Star, not by a
milestone roadmap; legacy milestone rows in `specs/PROGRESS.md` are historical
evidence, not the next implementation priority.

---

## License

Copyright (c) 2026 Common Tuning LLC. All rights reserved.

See [`LICENSE`](LICENSE).
