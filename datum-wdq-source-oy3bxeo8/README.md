# Datum EDA

Datum EDA is a professional electronics design system for Linux — schematic
capture, PCB layout, governed libraries, design-rule checking, and
manufacturing output — with AI collaboration engineered into its foundation.
A deterministic Rust engine owns a single design model and a single mutation
path; the desktop application, CLI, MCP server, and AI agents are peers over
that model. The result is a tool an engineer drives directly, in which
assistance is native rather than an add-on, and every workflow remains fully
manual.

Datum is an independent, native EDA system. Its foundation is governed native
libraries and schematic capture as the electrical source of truth; PCB
implementation and manufacturing output derive from that authority. The
product flow is **library → schematic → PCB → manufacturing**. Import and
export are provided for migration of existing designs and libraries, for
reference fixtures, and for reverse engineering; they are compatibility
infrastructure, not the product boundary.

## Principles

- **Human tool first.** Every core workflow is performed directly by an
  engineer; AI is never required and never a hidden authority.
- **AI in the core.** Assistants operate through the same stable IDs, queries,
  typed operations, proposals, checks, diffs, and provenance a user does — no
  private edit powers.
- **One engine, peer surfaces.** The engine is a library with no GUI or
  rendering dependency; the desktop application, CLI, MCP, and agents are
  equal consumers of one resolved design model.
- **One mutation path.** Every committed change is a typed operation through a
  single commit and journal, with provenance, diff, and undo.
- **Standards conformance by design.** Each researched standard family carries
  an explicit support disposition in a governed registry; checks run against
  named standard profiles; waivers and accepted deviations are first-class,
  provenance-bearing records. Datum states precisely what it checks and never
  implies third-party certification.

Controlling doctrine: [`docs/DATUM_PRODUCT_MECHANICS.md`](docs/DATUM_PRODUCT_MECHANICS.md),
the decision records in [`docs/decisions/`](docs/decisions/), and the domain
contracts in [`docs/contracts/`](docs/contracts/).

## Capabilities

| Domain | Available today |
|---|---|
| **Library** | Governed component pool (`Unit`, `Symbol`, `Gate`, `Entity`, `Part`, `Footprint`, `PinPadMap`); IPC-7351B land-pattern generation for two-terminal chip and SOIC families; Eagle library and KiCad footprint import. No third-party library content is bundled. |
| **Schematic** | Native sheets and hierarchy, symbol placement, wires, junctions, no-connects, net labels, ports, buses, text, and graphics, all as journaled operations; forward-annotation review and apply. |
| **PCB** | Components, pads, tracks, vias, zones, nets and net classes, outline, stackup, keepouts, dimensions, and text; bounded native zone fill; deterministic routing kernel with route proposals that can be exported, inspected, revalidated, and applied. |
| **Checks & standards** | ERC and DRC held at 0% false-positive/negative quality gates on real reference designs; check profiles (`erc`, `drc`, `standards`, `manufacturing`, `release`); fingerprint-scoped waivers and accepted deviations; standards-repair proposals generated from a check run. |
| **Manufacturing** | Gerber RS-274X (copper, mask, paste, silkscreen, outline, mechanical), Excellon drill, BOM, and pick-and-place, produced from journaled manufacturing plans and output jobs. |
| **Automation & AI** | A single verb registry generates the complete public tool surface (338 `datum.*` tools across 17 families) for the CLI and the MCP server; proposals carry the full create / review / preview / validate / apply lifecycle so agents propose and humans commit. |
| **Desktop application** | wgpu-based shell with menu bar, tiled board and schematic panes, shared camera/grid/hit-test backbone, engine-resolved scenes, artifact preview, inspector and review panels, a drift-gated design-token system, an embedded Datum-owned terminal with session tabs and agent launch context, and a visual-regression harness. |
| **Interoperability** | Deterministic import of KiCad boards, schematics, projects, and footprints and of Eagle libraries into the native model, with query and checks over the converted design. |

**In development.** The desktop application is a review and supervision
surface today; interactive authoring through a direct GUI→engine commit path
is a scheduled phase, and the embedded terminal is being completed into a
full daily-driver terminal over a Datum-owned core. Library depth (further
package families, native symbol import), general copper pour, and additional
exchange formats (ODB++, IPC-2581, STEP) are specified and not yet built.
Status is tracked in [`specs/PROGRESS.md`](specs/PROGRESS.md).

## Getting started

Requires Rust stable and a C linker; the desktop application additionally
requires a Vulkan-capable environment (software Vulkan such as lavapipe is
sufficient).

```bash
cargo build
cargo test
scripts/run_drift_gates.sh      # pre-merge gate: clippy, spec/parity/governance gates, proof gates
```

### Desktop application

```bash
# Open a native project
cargo run -p datum-gui-app --bin datum-gui -- --project-root ./demo

# Open an existing KiCad board as a Datum workspace
cargo run -p datum-gui-app --bin datum-gui -- --board path/to/board.kicad_pcb
```

### Command line

```bash
cargo run -p datum-eda-cli -- project new ./demo --name "Demo"
cargo run -p datum-eda-cli -- project query ./demo summary

cargo run -p datum-eda-cli -- import library.lbr
cargo run -p datum-eda-cli -- pool search "100nF 0402" --library library.lbr

cargo run -p datum-eda-cli -- erc design.kicad_sch
cargo run -p datum-eda-cli -- drc design.kicad_pcb
```

Command families: `project`, `check`, `proposal`, `journal`, `artifact`,
`plan`, `modify`, `pool`, `query`, `erc`, `drc`, `import`, `context`.
Exit codes: `0` pass, `1` violations found, `2` execution error.

### MCP server

```bash
cargo run -p eda-engine-daemon -- --socket /tmp/datum-eda-engine.sock
python3 mcp-server/server.py
```

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

The tool catalog (`mcp-server/datum_tool_catalog.json`) is generated from the
verb registry and drift-gated; [`specs/MCP_API_SPEC.md`](specs/MCP_API_SPEC.md)
is the contract.

## Architecture

```
                 ┌──────────────┐
                 │  MCP server  │  ← AI agents (stdio)
                 └──┬────────┬──┘
     JSON-RPC/socket│        │ CLI-bridged verbs
                 ┌──┴─────┐  │
                 │ Daemon │  │
                 └──┬─────┘  │
    ┌───────────┐   │   ┌────┴─────┐
    │  Desktop  │───┼──→│   CLI    │  datum-eda
    │   (wgpu)  │   │   └────┬─────┘
    └───────────┘   │        │
                 ┌──┴────────┴──┐
                 │    Engine    │  eda-engine (Rust library)
                 └──────────────┘
```

The engine is a Rust library with no GUI or rendering dependency. The daemon
exposes it over Unix-socket JSON-RPC; the MCP server is a thin translation
layer over the daemon and CLI; the desktop application consumes engine state
through the `gui-protocol` scene contract, currently via the CLI. Every surface
operates on one resolved design model through the same operation/commit model.

| Crate | Package | Role |
|---|---|---|
| `crates/engine` | `eda-engine` | Engine library: substrate, resolver, checks, routing, import/export |
| `crates/cli` | `datum-eda-cli` → `datum-eda` | Command-line surface |
| `crates/engine-daemon` | `eda-engine-daemon` | JSON-RPC daemon |
| `crates/verb-registry` | `datum-verb-registry` | Single-source verb registry and catalog generator |
| `crates/gui-protocol` | `datum-gui-protocol` | Scene contract, workspace and menu model |
| `crates/gui-viewport` | `datum-gui-viewport` | Shared camera, grid, hit-test, and interaction backbone |
| `crates/gui-render` | `datum-gui-render` | wgpu renderer, design tokens, visual-regression harness |
| `crates/gui-app` | `datum-gui-app` → `datum-gui` | Desktop application shell and embedded terminal |
| `crates/terminal-core` | `datum-terminal-core` | Datum-owned terminal emulation core (in development) |
| `crates/test-harness` | `eda-test-harness` | Performance, quality, and proof-gate harnesses |

## Documentation

| Document | Contents |
|---|---|
| [`docs/DATUM_PRODUCT_MECHANICS.md`](docs/DATUM_PRODUCT_MECHANICS.md) | Controlling product doctrine |
| [`docs/decisions/`](docs/decisions/) | Ratified decision records |
| [`docs/contracts/`](docs/contracts/) | Domain tool contracts: schematic, PCB, library, rules, manufacturing, AI/CLI/MCP surface, UI layout |
| [`specs/STANDARDS_COMPLIANCE_SPEC.md`](specs/STANDARDS_COMPLIANCE_SPEC.md) | Standards registry, support dispositions, waiver and deviation obligations |
| [`docs/CANONICAL_IR.md`](docs/CANONICAL_IR.md) · [`docs/ENGINE_DESIGN.md`](docs/ENGINE_DESIGN.md) | Data model, invariants, operation model |
| [`docs/DATUM_SHARED_TOOLING_TAXONOMY.md`](docs/DATUM_SHARED_TOOLING_TAXONOMY.md) | Shared editor-tooling backbone |
| [`docs/gui/`](docs/gui/) | Desktop application product model, conformance, visual language, terminal specifications |
| [`specs/PROGRESS.md`](specs/PROGRESS.md) · [`specs/SPEC_PARITY.md`](specs/SPEC_PARITY.md) | Implementation status and machine-checked inventory |

## Roadmap

The scheduled roadmap is the structured manifest
[`specs/active_frontier.json`](specs/active_frontier.json), projected as the
Active Frontier at the top of [`specs/PROGRESS.md`](specs/PROGRESS.md) and
tracked in the in-repo `br` issue tracker. The canonical current task is:

```bash
python3 scripts/project_status.py next
```

## License

Copyright (c) 2026 Common Tuning LLC. All rights reserved. See
[`LICENSE`](LICENSE).
