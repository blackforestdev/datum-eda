# Datum EDA

Datum EDA is an AI-native electronics design platform with a deterministic
engine core and machine-native control surfaces.

It is being built as an independent EDA system, not as a wrapper around
another tool.

KiCad-first support is the current execution path for fast, verifiable
delivery. It is not the long-term product boundary.

---

## Why

Most EDA systems were built GUI-first and automation was layered in later.
Datum inverts that model:
- Engine-first deterministic core.
- AI/CLI as first-class interfaces.
- GUI as a downstream consumer, not the system of record.

---

## Direction and Scope

Roadmap sequencing is intentional:
- foundation: deterministic IR and reliable import/query/check surfaces
- expansion: safe write operations and native authoring
- later: broader ecosystem interop, including commercial migration paths

Canonical scope terminology is defined in
[`specs/PROGRAM_SPEC.md`](specs/PROGRAM_SPEC.md) (`Product identity`,
`Implementation slice`, `Execution strategy`, `Non-goals`).

---

## What it can do today

- Import/query/check KiCad `.kicad_pcb` and `.kicad_sch` designs.
- Import Eagle `.lbr` libraries into the pool.
- Run ERC/DRC and consume structured check reports in CLI and MCP.
- Modify imported KiCad board slices via deterministic operation flows.
- Create and edit native project slices (`project new`, native query/check,
  forward-annotation review/apply + artifact slices, and manufacturing exports
  (Gerber/drill/set report/export/validate/compare surfaces).
- Run deterministic M5 routing-kernel query slices from persisted native board
  state (routing substrate, preflight/corridor, and path-candidate explain
  surfaces).

See [`docs/USER_WORKFLOWS.md`](docs/USER_WORKFLOWS.md) for end-to-end usage
examples and [`specs/MCP_API_SPEC.md`](specs/MCP_API_SPEC.md) for the full
MCP tool catalog.

For milestone-scoped status and limitations, use
[`specs/PROGRESS.md`](specs/PROGRESS.md).

---

## Build

Requires Rust stable (1.80+) and a C linker.

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
# Import Eagle libraries into the pool
cargo run -p eda-cli -- import library.lbr

# Check a KiCad design
cargo run -p eda-cli -- erc design.kicad_sch
cargo run -p eda-cli -- drc design.kicad_pcb

# Query a design
cargo run -p eda-cli -- query design.kicad_pcb summary
cargo run -p eda-cli -- query design.kicad_pcb nets
cargo run -p eda-cli -- query design.kicad_pcb components

# Search the component pool
cargo run -p eda-cli -- pool search "100nF 0402" --library library.lbr

# Imported-board modify slice
cargo run -p eda-cli -- modify design.kicad_pcb --move-component "<uuid>:25:15:90" --save out.kicad_pcb

# Native project slice (M4 closed for scope; M5 in progress)
cargo run -p eda-cli -- project new ./demo --name "Demo"
cargo run -p eda-cli -- project query ./demo summary
cargo run -p eda-cli -- project query ./demo routing-substrate
```

Exit codes: `0` = pass, `1` = violations found, `2` = execution error.

---

## MCP server

The MCP server exposes the current engine surface to AI agents over stdio.

```bash
# Start the engine daemon
cargo run -p eda-engine-daemon -- --socket /tmp/datum-eda-engine.sock

# Start the MCP server
python3 mcp-server/server.py
```

Example MCP client registration:

```json
{
  "mcpServers": {
    "datum-eda": {
      "command": "python3",
      "args": ["/path/to/datum-eda/mcp-server/server.py"],
      "env": {
        "EDA_ENGINE_SOCKET": "/tmp/datum-eda-engine.sock"
      }
    }
  }
}
```

Client-specific config file locations vary by MCP host.

---

## Architecture

```
┌─────────────┐
│  MCP Server  │  ← AI agents (Python, stdio)
└──────┬──────┘
       │ JSON-RPC / Unix socket
┌──────┴──────┐    ┌──────────┐
│   Engine    │←───│   CLI    │
│   (Rust)    │    │  (Rust)  │
└─────────────┘    └──────────┘
```

The engine is a Rust library crate with no GUI/rendering dependency.
The daemon exposes it over Unix socket JSON-RPC.
The MCP server is a thin Python translation layer over that daemon.

Key documentation:

| Document | Contents |
|----------|----------|
| [`docs/CANONICAL_IR.md`](docs/CANONICAL_IR.md) | Core data model and invariants |
| [`docs/ENGINE_DESIGN.md`](docs/ENGINE_DESIGN.md) | Operation model and API surface |
| [`docs/RESEARCH_TRACEABILITY.md`](docs/RESEARCH_TRACEABILITY.md) | Research conclusions mapped to roadmap/spec sequencing |
| [`specs/PROGRAM_SPEC.md`](specs/PROGRAM_SPEC.md) | Controlling feature specification |
| [`specs/MCP_API_SPEC.md`](specs/MCP_API_SPEC.md) | Full MCP tool catalog |
| [`specs/PROGRESS.md`](specs/PROGRESS.md) | Implementation status per spec requirement |

---

## Roadmap

See [`PLAN.md`](PLAN.md).

Directionally, Datum targets a full AI-native EDA stack:
- independent deterministic core architecture
- broad format interoperability
- native design-authoring capability
- higher-level automation and strategy layers

Formal milestone contracts live in [`specs/PROGRAM_SPEC.md`](specs/PROGRAM_SPEC.md).
Current M5 entry boundary is defined in
[`specs/progress/m5_opening.md`](specs/progress/m5_opening.md).

---

## License

Copyright (c) 2026 Common Tuning LLC. All rights reserved.

See [`LICENSE`](LICENSE).
