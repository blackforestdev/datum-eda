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
| **Automation & AI** | A single verb registry generates the complete public tool surface (registry-generated `datum.*` tools across 17 families) for the CLI and the MCP server; proposals carry the full create / review / preview / validate / apply lifecycle so agents propose and humans commit. |
| **Desktop application** | wgpu-based shell with menu bar, tiled board and schematic panes, shared camera/grid/hit-test backbone, engine-resolved scenes, artifact preview, inspector and review panels, a drift-gated design-token system, an embedded Datum-owned terminal with session tabs and agent launch context, and a visual-regression harness. Bounded native library browsing and manual Footprint/Padstack authoring support preview, commit, undo/redo, save and reopen. |
| **Interoperability** | Deterministic import of KiCad boards, schematics, projects, and footprints and of Eagle libraries into the native model, with query and checks over the converted design. |

**Development availability.** The library GUI delivery covers browsing all eight
native kinds and the supported Footprint/Padstack authoring workflow. Its
[ordinary application record](docs/reviews/workflow-delivery-rollout/library-gui-authoring-delivery.json)
and current Frontier record preserve the delivered scope and separate full
qualification. Other editor capabilities, selection qualification and terminal
work remain subject to their own completion contracts. Consult
[`specs/PROGRESS.md`](specs/PROGRESS.md) for current availability and open work.

## Getting started

Requires the Rust toolchain pinned in [`rust-toolchain.toml`](rust-toolchain.toml)
and a C linker; the desktop application additionally
requires a Vulkan-capable environment (software Vulkan such as lavapipe is
sufficient).

```bash
python3 scripts/run_cargo_guarded.py --workload interactive -- cargo build
bash scripts/run_drift_gates.sh --focused # structural repository checks, no builds
python3 scripts/check_alignment.py --run-health # prebuild companions, then workspace tests
bash scripts/run_drift_gates.sh --full # full qualification and repository gates (CI default)
```

Local development output has a separate lifecycle. After releasing ownership,
preview `python3 scripts/prune_development_output.py --build-only`; apply with
`--apply --released` to clear build output while preserving native projects,
runtime binaries, companion dependency records and all `.datum` task output.
At task completion, the same command without `--build-only` also retires released
temporary proof output while preserving native projects and rollback metadata.
Never prune while Cargo or the application is active. With the repository Git
hooks enabled, every commit retains only the five newest local tracker backups.

### Desktop application

```bash
# Open a native project
python3 scripts/run_cargo_guarded.py --workload interactive -- cargo run -p datum-gui-app --bin datum-gui -- --project-root "$HOME/Documents/datum-projects/demo"

# Open an existing KiCad board as a Datum workspace
python3 scripts/run_cargo_guarded.py --workload interactive -- cargo run -p datum-gui-app --bin datum-gui -- --board path/to/board.kicad_pcb
```

### Command line

```bash
mkdir -p "$HOME/Documents/datum-projects"
python3 scripts/run_cargo_guarded.py --workload interactive -- cargo run -p datum-eda-cli -- project new "$HOME/Documents/datum-projects/demo" --name "Demo"
python3 scripts/run_cargo_guarded.py --workload interactive -- cargo run -p datum-eda-cli -- project query "$HOME/Documents/datum-projects/demo" summary

python3 scripts/run_cargo_guarded.py --workload interactive -- cargo run -p datum-eda-cli -- import library.lbr
python3 scripts/run_cargo_guarded.py --workload interactive -- cargo run -p datum-eda-cli -- pool search "100nF 0402" --library library.lbr

python3 scripts/run_cargo_guarded.py --workload interactive -- cargo run -p datum-eda-cli -- erc design.kicad_sch
python3 scripts/run_cargo_guarded.py --workload interactive -- cargo run -p datum-eda-cli -- drc design.kicad_pcb
```

Command families: `project`, `check`, `proposal`, `journal`, `artifact`,
`plan`, `modify`, `pool`, `query`, `erc`, `drc`, `import`, `context`.
Exit codes: `0` pass, `1` violations found, `2` execution error.

### MCP server

```bash
python3 scripts/run_cargo_guarded.py --workload interactive -- cargo run -p eda-engine-daemon -- --socket /tmp/datum-eda-engine.sock
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
through the `gui-protocol` scene contract, through the shared session/native-edit client and supervision adapters. Every surface
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
| `crates/terminal-core` | `datum-terminal-core` | Datum-owned terminal emulation core; qualification tracked separately |
| `crates/test-harness` | `eda-test-harness` | Performance, quality, and proof-gate harnesses |

GUI application owners live under `crates/gui-app/src/`: `application/` for
startup and orchestration, `windows/` for owned dialogs and project opening,
`terminal/` for terminal integration, `editor/` for tool/selection actions,
`viewport/` for camera and pane interaction, and `rendering/` for frame/GPU
coordination. Associated tests stay beside their owners. `main.rs` registers the
modules and starts the application; existing Rust module identities remain stable.

## Documentation

Start with the [documentation map](docs/README.md) to find requirement owners,
implementation and test entry points, status, reviews and historical context.

User designs belong in a user-chosen project directory. New generated native
fixtures use an external, owner-labelled workspace rather than the checkout:

```bash
DATUM_FIXTURE_OWNER="authorized-task-label" python3 scripts/run_cargo_guarded.py --workload proof -- cargo run -p eda-test-harness --bin s5a_fixture -- electrical-scope --managed
```

The default base is `$XDG_STATE_HOME/datum-eda/fixtures`, or
`$HOME/.local/state/datum-eda/fixtures`. `DATUM_FIXTURE_WORKSPACE_ROOT` can select
another absolute external base. One owner/family reserves one fresh root;
repeated generation refuses existing data/reservations. The `.owner.json`
sidecar records provenance, never a grant, lease or acceptance. Failed output
is retained for its owner. After owner release, audit references and preserve
native projects/journals outside the checkout before retiring their original
locations; never sweep user designs or referenced proof inputs. Existing explicit
root arguments remain available for source-pinned replay. Cargo build output
continues to use the separate guarded, disk-backed target policy.

The [documentation map](docs/README.md) names the existing product, domain,
IR/interface, evidence and historical owners, with source and test entry points.
Its subject-spec links delegate to the existing
[formal ownership map](specs/PROGRAM_SPEC.md#ownership-and-precedence).
Current product requirements and scheduled work retain their existing authority;
old launch and milestone records remain historical. For temporary captures and
preserved native inputs, use the [review index](docs/reviews/README.md).

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
