# Datum EDA

Datum EDA is an AI-native electronics design platform.

Datum is being built as a new class of EDA system: deterministic engine core,
machine-native interfaces, and automation-first workflows from day one.

What Datum EDA is:
- A standalone core architecture for analysis, checking, modification, and
  eventually native design authoring.
- AI/CLI-first control surfaces with a shared deterministic execution model.
- A foundation intended to scale across multiple EDA ecosystems over time.

What Datum EDA is not:
- A CLI facade for another tool.
- A compatibility layer whose long-term value is format translation.

KiCad-first support is the current execution beachhead for rapid validation on
real projects. It is not the product boundary.

---

## Why

Most EDA systems were built GUI-first. Automation and AI were added later.

Datum EDA inverts that model:
- Engine-first deterministic core.
- AI/CLI interfaces as first-class surfaces.
- GUI as a downstream consumer, not the system of record.

The objective is not to replicate existing UX with scripts attached. The
objective is to define a modern EDA architecture where AI-native workflows are
fundamental.

---

## Direction and Scope

Datum EDA is not scoped to KiCad + Eagle as a permanent ceiling.

The current roadmap intentionally sequences delivery:
- First: deterministic core IR, import/query/check correctness, and reliable
  AI/CLI automation.
- Next: safe write operations and native authoring.
- Then: broader ecosystem interoperability, including commercial-tool migration
  paths.

This sequencing is about execution risk and engineering quality, not limited
ambition. Datum is being built as its own system.

Canonical scope terminology is defined in
[`specs/PROGRAM_SPEC.md`](specs/PROGRAM_SPEC.md) (`Product identity`,
`Implementation slice`, `Execution strategy`, `Non-goals`).

---

## What it can do today

- **Import/query/check** KiCad `.kicad_pcb` and `.kicad_sch` designs
- **Import** Eagle `.lbr` component libraries (pool ingestion)
- **Modify (current M3 slice)** imported KiCad boards:
  `move_component`, `delete_track`, `delete_via`, `set_design_rule`,
  `undo`/`redo`, `save`
- **Query** any design: components, nets, stackup, hierarchy, airwires,
  connectivity diagnostics
- **ERC** — electrical rule checking with structured violation reports,
  configurable severity, and waiver support
- **DRC** — physical rule checking: clearance, track width, via geometry,
  silk clearance, unrouted nets
- **MCP server** — all of the above accessible to AI agents through any
  MCP-compatible client via Unix socket JSON-RPC
- **CLI** — `eda import`, `query`, `erc`, `drc`, `check`, `modify`,
  `pool search` with JSON output and CI-friendly exit codes

Current limitations (intentional, milestone-scoped):
- Eagle `.brd` / `.sch` design import is not implemented yet.
- KiCad import/write-back currently targets a defined subset while fidelity and
  corpus confidence continue to expand.

Current state should be read as implementation maturity, not product identity.

See [`docs/USER_WORKFLOWS.md`](docs/USER_WORKFLOWS.md) for end-to-end usage
examples and [`specs/MCP_API_SPEC.md`](specs/MCP_API_SPEC.md) for the full
MCP tool catalog.

---

## Build

Requires Rust stable (1.80+) and a C linker.

```bash
cargo build
cargo test
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

# Current M3 board modify slice
cargo run -p eda-cli -- modify design.kicad_pcb --move-component "<uuid>:25:15:90" --save out.kicad_pcb
```

Exit codes: `0` = pass, `1` = violations found, `2` = execution error.

---

## MCP server

The MCP server exposes the full engine surface to AI agents over stdio.

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

The engine is a Rust library crate with no GUI or rendering dependencies.
The daemon exposes it over a Unix socket for multi-client access. The MCP
server is a thin Python translation layer on top of that socket.

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

Directionally, Datum EDA targets a full AI-native EDA stack:
- independent core architecture
- broad format interoperability
- native design-authoring capability
- advanced automation and strategy layers on top of deterministic primitives

Format coverage starts where execution is fastest and safest, then expands.
Long-term scope includes both open-tool and commercial-tool migration paths.

Formal milestone contracts live in [`specs/PROGRAM_SPEC.md`](specs/PROGRAM_SPEC.md).

---

## License

Copyright (c) 2026 Common Tuning LLC. All rights reserved.

See [`LICENSE`](LICENSE).
