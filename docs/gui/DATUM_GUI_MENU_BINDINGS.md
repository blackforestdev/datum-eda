# Datum GUI Menu Bindings

> **Status**: Governed active planning reference.
> **Authority**: Companion to
> `docs/decisions/PRODUCT_MECHANICS_019_GUI_PRODUCT_MODEL.md` and
> `docs/gui/DATUM_GUI_PRODUCT_SPEC.md`.
> **Scope**: Binds every GUI menu command to the real Datum mechanism behind it
> (verb-registry verb id, native-write builder, and/or CLI command), classifies
> each as live / engine-ready-GUI-blocked / not-built, and records the write-path
> plumbing that must exist before authoring menu items can be wired correctly.

## Purpose

A menu bar is only correct if every item behind it resolves to a real mechanism.
This document is the evidence layer for the product spec's "Shell Contract →
Minimum top menu": it prevents the menu from advertising capabilities that do not
exist, and prevents authoring items from being implemented as the terminal
CLI-string hack that decision 019 forbids. It doubles as the source for a future
machine-checkable `menu_model` manifest (each entry references a real verb id, so
a drift gate can verify the built menu against the registry).

Derived from a four-source internal inventory (2026-07-05): the verb registry
(`crates/verb-registry/`), the native-write facade
(`crates/engine/src/api/native_write/`), the CLI (`crates/cli/`), and the daemon
+ shared tool surface (`crates/engine-daemon/`,
`docs/contracts/AI_CLI_MCP_TOOL_SURFACE.md`).

## Capability inventory (what exists)

- **Verb registry**: 338 public verbs across 17 families — the single-source
  catalog (decision 017). Source of truth: `crates/verb-registry/src/lib.rs`
  (`verbs()`), projected to `mcp-server/datum_tool_catalog.json`.
- **Native-write facade**: typed `build_*` builders for nearly every authoring
  action (schematic, board, library, manufacturing, project, genesis), each
  producing a guarded `PreparedWrite` committed through the one
  `commit_prepared()` journaled path.
- **CLI**: ~250 `project` subcommands + families covering the full EDA surface.

Conclusion: **capability is not the blocker.** The GUI's ability to *reach* these
operations is.

## Status legend

- **LIVE-READ** — verb exists and the GUI can already reach it today via
  `run_cli_json` (read-only data loading).
- **ENGINE-READY / GUI-BLOCKED** — builder + verb + CLI exist and journal
  correctly, but the GUI can only reach it via the forbidden terminal CLI-string
  handoff. Needs the write-path plumbing (see "Prerequisite plumbing").
- **NOT-BUILT** — no backing mechanism yet; the menu item is a design decision or
  future work. Menu item present but disabled per product spec.

## Menu → mechanism bindings

### File
| Menu item | Backing mechanism | Status |
|---|---|---|
| New Project | `bootstrap_native_project` (genesis) / CLI `project new` | ENGINE-READY / GUI-BLOCKED |
| Open Project | none — project root is an implicit arg to every command; `project inspect` is nearest | NOT-BUILT (needs GUI open + session model) |
| Import | `build_kicad_board_import` / `build_kicad_schematic_import` / `build_kicad_footprint_import` / `build_eagle_library_import`; CLI `project import-kicad-*` / `import-eagle-library` | ENGINE-READY / GUI-BLOCKED |
| Save / Save As | none — native projects auto-persist per journaled mutation; no native save/save-as verb | NOT-BUILT (needs product decision — see "File-menu semantics") |
| Export | `datum.artifact.generate` / `export_manufacturing_set`; CLI `project export-*` (gerber/drill/bom/pnp/manufacturing-set) | ENGINE-READY / GUI-BLOCKED (generation), LIVE-READ (inspect) |
| Close | none (stateless CLI) | NOT-BUILT (needs GUI session model) |

### Edit
| Menu item | Backing mechanism | Status |
|---|---|---|
| Undo / Redo | `datum.journal.undo` / `redo` as typed GUI journal actions; no terminal CLI-string handoff | ENGINE-READY / GUI-BLOCKED |
| Cut/Copy/Paste/Delete | per-object delete builders exist (`build_delete_*`); no clipboard model | PARTIAL: delete ENGINE-READY; clipboard NOT-BUILT |
| Preferences | none | NOT-BUILT |

### View
Fit / Zoom / Pan / Layer Visibility / Panels / Reset Workspace — **consumer-side
GUI state**, not engine operations (per CLAUDE.md interactive-behavior rule). No
verb binding; owned by the GUI shell + decisions 014/015. Status: GUI-LOCAL.

### Place (schematic, active-editor-gated)
| Item | Verb / builder | Status |
|---|---|---|
| Symbol | `datum.schematic.place_symbol` / `build_place_schematic_symbol` | ENGINE-READY / GUI-BLOCKED |
| Wire | `datum.schematic.draw_wire` / `build_create_schematic_wire` | ENGINE-READY / GUI-BLOCKED |
| Label | `datum.schematic.place_label` / `build_create_schematic_label` | ENGINE-READY / GUI-BLOCKED |
| Junction / No-connect / Port / Bus / Bus-entry | `datum.schematic.place_*` / connectivity builders | ENGINE-READY / GUI-BLOCKED |
| Text / Drawing | `datum.schematic.place_text` / `place_drawing_*` | ENGINE-READY / GUI-BLOCKED |

### Place / Route (PCB, active-editor-gated)
| Item | Verb / builder | Status |
|---|---|---|
| Component (place) | `datum.pcb.place_component` / `build_place_board_package` | ENGINE-READY / GUI-BLOCKED |
| Move / Rotate / Flip / Lock | `datum.pcb.move_component` etc. / `BoardPackageEdit::{Position,Rotation,Side,Locked}` | ENGINE-READY / GUI-BLOCKED |
| Align / Distribute | `datum.pcb.align_components` / `build_align_board_packages` | ENGINE-READY / GUI-BLOCKED |
| Track / Via / Zone | `datum.pcb.draw_track` / `place_via` / `place_zone` + set/delete | ENGINE-READY / GUI-BLOCKED |
| Zone fill | `datum.check.fill_zones` / `build_set_zone_fills` | ENGINE-READY / GUI-BLOCKED |
| Net / Net class / Pad / Dimension / Keepout / Board text | `datum.pcb.place_net` / `place_net_class` / `place_pad` / `place_dimension` / `place_keepout` / `place_text` | ENGINE-READY / GUI-BLOCKED |
| Route apply | `datum.route.apply` / `apply_selected` | ENGINE-READY / GUI-BLOCKED (facade migration pending — see gaps) |

### Project
| Item | Verb / builder | Status |
|---|---|---|
| Validate | `datum.project.validate` / CLI `project validate` | LIVE-READ |
| Resolve / Debug | `datum.query.source_shards` / CLI `project query resolve-debug` | LIVE-READ |
| Rename | `build_set_project_name` / CLI `project set-project-name` | ENGINE-READY / GUI-BLOCKED |
| Design rules | `build_*_project_rule(s)` / CLI `project set-project-rules` etc. | ENGINE-READY / GUI-BLOCKED |
| General settings | none beyond rules/name | NOT-BUILT |

### Checks
| Item | Verb | Status |
|---|---|---|
| Run ERC / DRC / profiles | `datum.check.run` / `run_profile` (profiles: erc, drc, standards, manufacturing, release, native-combined) | ENGINE-READY / GUI-BLOCKED (write-evidence); results LIVE-READ |
| Profiles list | `datum.check.profiles` | LIVE-READ |
| Findings | `datum.check.list` / `show`; `datum.query.check`/`board-check` | LIVE-READ |
| Explain finding | `datum.check.explain_violation` (daemon RPC) | LIVE-READ |
| Waive | `datum.check.waive` | ENGINE-READY / GUI-BLOCKED (one of the 7 already daemon-wired) |
| Accept deviation | `datum.check.accept_deviation` | ENGINE-READY / GUI-BLOCKED (daemon-wired) |
| Standards repair | `datum.check.repair_standards` (proposal) | ENGINE-READY / GUI-BLOCKED |

### Manufacturing
| Item | Verb | Status |
|---|---|---|
| Output jobs | `datum.output_job.create*/update/delete/run`; `datum.proposal.*_output_job` | ENGINE-READY / GUI-BLOCKED |
| Artifacts (list/show/files/preview/compare) | `datum.artifact.list/show/files/preview/compare` | LIVE-READ |
| Generate | `datum.artifact.generate` / `export_manufacturing_set` | ENGINE-READY / GUI-BLOCKED |
| Validate | `datum.artifact.validate` / `validate_manufacturing_set` | LIVE-READ |
| Plans / Panels | `datum.manufacturing.*` (proposals) | ENGINE-READY / GUI-BLOCKED |

### Window / Help
Documents / terminal sessions / workspace layout / about / diagnostics —
GUI-LOCAL shell concerns; terminal sessions owned by decision 005. Status:
GUI-LOCAL (except `datum.context.*` reads, LIVE-READ).

## Prerequisite plumbing (blocks correct authoring menus)

Every ENGINE-READY / GUI-BLOCKED row above is blocked by the same missing write
path. Before authoring menu items can be wired per decision 019 (gesture →
`OperationBatch` → `commit()`/proposal, never a CLI string), four things must
exist:

1. **`NativeWrite` dispatch variant in the verb registry.** Today
   `crates/verb-registry/src/lib.rs` `Dispatch` has only `Cli { method, argv }`
   and `DaemonRpc { method }`; 239 verbs resolve to CLI strings and no verb
   targets `native.write`. Add a journaled-commit dispatch variant (verb →
   daemon `native.write` with typed params).
2. **Register the remaining native-write families.**
   `crates/engine/src/api/native_write/registry.rs` wires only `project::VERBS`
   + `waivers::VERBS` (7 verbs). The board/schematic/library/routing/etc.
   builders exist but have no `VERBS` table, so `native.write` cannot reach them.
3. **Daemon verb/schema enumeration.** `native.describe` returns only the
   stale-guard anchor + project identity — it enumerates no verbs or param
   schemas. Extend it (or add `native.list_verbs`) so the GUI can discover
   available operations and their parameters.
4. **A daemon client in `gui-app`.** `gui-app` does not link the engine and has
   no Unix-socket/JSON-RPC client; its only bridges are `run_cli_json` (reads)
   and the terminal CLI-string handoff (writes). Give it a daemon client (or a
   direct in-process engine authoring binding) so menu actions author
   `OperationBatch` → `commit_prepared`/proposal directly.

Reads (LIVE-READ rows) do not need items 1–3; they already work via
`run_cli_json`. This is why Phase 1 (shell + board render fidelity + read-only
inspection) is buildable now, while authoring menus wait on the plumbing.

## File-menu semantics (design decision, not a binding)

Native projects auto-persist per journaled mutation, so there is no native
Save / Save-As / Open / Close. The product spec must decide GUI semantics:
- **Save** — likely a no-op indicator ("all changes journaled") or an explicit
  checkpoint/commit-message affordance, not a write gate.
- **Save As** — copy/branch the project tree (no verb today).
- **Open** — needs a GUI project-session model (which root is active).
- **Close** — GUI session teardown.
These belong in the product spec's document/session model, resolved against
decision 007 (Project Workspace Model).

## Known authoring gaps (surface as disabled or delete+recreate)

- **Schematic wire / junction / bus-entry / no-connect** have create + delete
  builders but **no `set`/edit** — moving/editing one is delete+recreate until an
  edit builder lands.
- **Route apply** is not yet migrated onto the native-write facade
  (`board_routing.rs` notes it as the declared follow-on).
- **Forward-annotation proposals** cover only `SetComponentValue` and
  `RemoveComponent` — no "add component to board" action.
- **Board multi-select transforms** beyond align/distribute require looping the
  single-object edit builder or a proposal batch.

## Governance

Classified in `specs/spec_governance_manifest.json` (tracked_docs + entries).
Update this document when the plumbing lands (rows move to LIVE) or when a
formerly NOT-BUILT surface gets a mechanism. When the `menu_model` manifest is
introduced, this table is its human-readable source.
