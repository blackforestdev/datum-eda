# README.md Truthfulness Audit — 2026-08-18

Purpose: claim-by-claim comparison of the top-level `README.md` (last
substantive revision 2026-07-05, tracker alignment 2026-08-13) against the
codebase at HEAD, as the basis for the README rewrite landed alongside this
note. Verdicts: TRUE / PARTIAL / FALSE / OMITTED.

## Identity, direction, doctrine

| README claim | Verdict | Evidence |
|---|---|---|
| Headless-first, manual-first, optional first-class AI, one `DesignModel`, one `commit()` + journal | TRUE | `crates/engine/src/substrate/{operation,commit,journal}.rs`; `docs/DATUM_PRODUCT_MECHANICS.md` |
| Product flow library → schematic → PCB → manufacturing; KiCad is on-ramp not boundary | TRUE | decision 016; CLAUDE.md "Frozen: KiCad import" |
| "remaining write surfaces are converging onto it" | STALE | CLAUDE.md: write-surface convergence is COMPLETE; the CLI has zero op authoring |
| Roadmap = `specs/active_frontier.json` via `project_status.py next` | TRUE | `scripts/project_status.py`, decision 025 |
| `PLAN.md` as roadmap pointer | PARTIAL | `PLAN.md:1-20` self-declares non-authoritative / stale on conflict |
| `specs/PROGRAM_SPEC.md` defines scope terms | TRUE (file is legacy) | terms live at `:70-78`; header marks the file legacy M0–M4 master |

## "What it can do today"

| Claim | Verdict | Evidence |
|---|---|---|
| ERC 7 rules | UNDERSTATED | 10 finding codes in `crates/engine/src/erc/mod.rs` (`specs/ERC_SPEC.md:96-125` agrees); "7" tracks the stale `PROGRAM_SPEC.md:195` table |
| DRC 7 rules | TRUE | `crates/engine/src/drc/mod.rs:26-49`; `RuleType::HoleSize` declared but unimplemented (`rules/ast.rs:36-46`) |
| MCP daemon-dispatched + CLI-bridged | TRUE | 338 public `datum.*` verbs / 17 prefixes (`mcp-server/datum_tool_catalog.json`); 307 CLI-bridged, 31 daemon; CLAUDE.md says 337 (off by one) |
| Routing "60+ path-candidate strategies" | MISLEADING | 70 `route_path_candidate_*.rs` files incl. `_explain`/`_selection` helpers; ~27 distinct strategies, 26 solver entrypoints, no registry |
| Manufacturing: Gerber, Excellon, BOM, PnP | TRUE | `crates/engine/src/export/`; Excellon = via drills only; no ODB++/IPC-2581/STEP even as stubs (CLAUDE.md "spec stubs landed" refers to docs, not code) |
| Native projects: `project new`, query/check, forward-annotation review/apply | TRUE | forward annotation is engine + CLI only, no MCP verb |
| Native library: Unit/Symbol/Gate/Entity/Part/Footprint | TRUE | `crates/engine/src/pool/*`; also `PinPadMap`, `LibraryBinding` |
| "per-user KiCad/Horizon symbol import that normalizes into the native model" | FALSE | no `.kicad_sym` handling (`import/mod.rs:41-52`), no Horizon importer anywhere; only `.kicad_mod` footprints and Eagle `.lbr` |
| Eagle `.lbr` import | TRUE | `crates/engine/src/import/eagle/mod.rs`; Eagle `.brd`/`.sch` are explicit unimplemented stubs |
| KiCad `.kicad_pcb`/`.kicad_sch` import, query, check | TRUE | `crates/engine/src/import/kicad/` |
| KiCad write-back with round-trip fidelity | PARTIAL | `api/save_kicad.rs`: fenced imported-session text patch, byte-identical unmodified, one modify slice (`delete_track`) — not general export |
| GUI: read-only board review + supervision surface + visual harness | TRUE but severely understated | see GUI section |
| "Taffy layout and token design system are landing" | STALE | both landed and drift-gated (`gui-render/src/render/layout.rs`, `design_tokens.rs`, `check_gui_design_tokens.py`) |
| Interactive editor "in-progress, not end-to-end" | TRUE, imprecise | GUI-WRITE-PATH is `blocked`; authoring actions prefill CLI strings into the embedded terminal (`runtime_board_text_edit.rs:111-123`), which decision 019 disallows as an editor path |

## Omitted entirely

- **Embedded native terminal** — Datum-owned Linux PTY transport (`gui-app/src/terminal_transport/`), first-party VT screen model (`terminal_screen/`, ~3.7k LOC), session tabs, agent context env (`DATUM_PROJECT_ROOT`, `DATUM_CLI`, `DATUM_SESSION_ID`), no third-party terminal crate. Governed by decisions 027–031; the current in-progress Frontier item.
- **Substrate depth** — `Operation` enum has 133 variants (Board 40, Schematic 39, Pool 9, …); native-write facade has 19 family modules (CLAUDE.md says 11).
- **Native zone fill** — bounded solver exists (`substrate/zone_fill*.rs`, `datum.check.fill_zones`); CLAUDE.md "imported fills only" is stale.
- **Proposals** — 48 `datum.proposal.*` verbs with full lifecycle.
- **Shared viewport backbone crate** `crates/gui-viewport` (decision 023), pane tiling (decision 021), data-driven menu bar (149 entries), schematic scene view, artifact preview, IBM Plex embedding.
- **Verb registry** crate `datum-verb-registry` as the single source for CLI/MCP/menu surfaces.

## Build / run / architecture

| Claim | Verdict | Evidence |
|---|---|---|
| `cargo run -p eda-engine-daemon -- --socket …` | TRUE | flag mandatory, hand-parsed, no default (`engine-daemon/src/main.rs:210-233`) |
| CLI examples (8) | TRUE | `query <file> <what>` only via `external_subcommand` legacy shim (`args/root.rs:229-231`) |
| Exit codes 0/1/2 | TRUE | `cli/src/main.rs:49-61` |
| `scripts/run_drift_gates.sh` | TRUE | also runs proof gates and `mcp-server/server.py --self-test` |
| Vulkan-capable env for GUI | TRUE | `--features visual` needed for offscreen capture; system deps (fontconfig etc.) undocumented |
| Architecture: Python scripting via PyO3 | FALSE | no `pyo3` anywhere in the workspace |
| Architecture: GUI → Engine | MISLEADING | `gui-protocol` reaches the engine by spawning the `datum-eda` CLI (`lib.rs:4422-4467`, falls back to `cargo run`); no daemon client, no journaled write path |
| Env `DATUM_ENGINE_SOCKET`, legacy `EDA_ENGINE_SOCKET` | TRUE | `mcp-server/server_runtime.py:30` |
| LICENSE proprietary, Common Tuning LLC | TRUE | `LICENSE`, `Cargo.toml` `LicenseRef-Proprietary` |

## Stale statements found in CLAUDE.md (not edited — owner call)

- "337/337 public tools" → 338.
- "native-write facade … 11 families" → 19 family modules.
- "Native copper-pour zone fill (imported fills only today)" → bounded native
  solver landed; general pour reports `Unsupported`.
- Daemon `main.rs:7-10` header comment says "no socket transport yet" — stale.
