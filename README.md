# Datum EDA

A headless-first PCB design engine for Linux. AI agents and CLI scripts are
the primary interfaces; the GUI is a later consumer.

Datum EDA is built to be useful before it is complete: a headless AI/CLI-first
engine with a KiCad-first implementation path today, bounded Eagle migration
support, and an explicit roadmap to broaden interoperability across additional
industry-standard EDA ecosystems over time.

---

## Why

Every major EDA tool on Linux was designed for humans clicking a GUI. The
command-line and scripting interfaces are afterthoughts. AI integration is
nonexistent or bolted on.

Datum EDA inverts that: the engine is a pure Rust library with no GUI
dependency. The GUI is a future consumer, not a prerequisite. This makes the
engine the best possible host for AI-driven design analysis and automation.

---

## Direction and Scope

Datum EDA is not scoped to KiCad + Eagle as a permanent ceiling.

The current roadmap intentionally sequences delivery:
- First: deterministic core IR, import/query/check correctness, and reliable
  AI/CLI automation.
- Next: safe write operations and native authoring.
- Then: broader ecosystem interoperability, including commercial-tool migration
  paths.

This sequencing is about execution risk, not product ambition.

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
- **MCP server** — all of the above accessible to AI agents (Claude Code,
  any MCP-compatible client) via Unix socket JSON-RPC
- **CLI** — `eda import`, `query`, `erc`, `drc`, `check`, `modify`,
  `pool search` with JSON output and CI-friendly exit codes

Current limitations (intentional, milestone-scoped):
- Eagle `.brd` / `.sch` design import is not implemented yet.
- KiCad import/write-back currently targets a defined subset while fidelity and
  corpus confidence continue to expand.

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

Register in Claude Code `~/.claude/settings.json`:

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
| [`specs/PROGRAM_SPEC.md`](specs/PROGRAM_SPEC.md) | Controlling feature specification |
| [`specs/MCP_API_SPEC.md`](specs/MCP_API_SPEC.md) | Full MCP tool catalog |
| [`specs/PROGRESS.md`](specs/PROGRESS.md) | Implementation status per spec requirement |

---

## Roadmap

See [`PLAN.md`](PLAN.md).

Directionally, the long-term interoperability intent is explicitly broader than
KiCad + Eagle. The current roadmap prioritizes getting the core engine correct,
deterministic, and automatable first, then expanding format coverage with a
commercial interop track (Altium, PADS, OrCAD/Allegro) once core milestones are
stable.

Formal milestone contracts live in [`specs/PROGRAM_SPEC.md`](specs/PROGRAM_SPEC.md).

---

## License

Copyright (c) 2026 Common Tuning LLC. All rights reserved.

See [`LICENSE`](LICENSE).
