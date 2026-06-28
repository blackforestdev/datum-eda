# Datum North-Star Project Audit

Status: review artifact, not a roadmap rewrite.
Date: 2026-06-13

Companion foundation:
- `docs/DATUM_PRODUCT_MECHANICS.md` defines the draft product mechanics this
  audit should be reviewed against.
- `docs/audits/scope-integration/DATUM_PRODUCT_MECHANICS_REVIEW_AGENDA.md` breaks the unresolved
  mechanics questions into one-by-one decision tracks.

## Purpose

This audit checks whether the current Datum codebase and roadmap are being
driven by the intended product north star:

> Datum is an AI-first professional EDA application, with native schematic and
> PCB authoring, rules/checking, deterministic design mutation, and human/AI
> review/apply workflows. Import from KiCad or other EDA tools is supporting
> infrastructure, not the product identity.

The goal is not to define a new milestone yet. The goal is to identify what
exists, what is misweighted, and what needs a course-correction discussion
before more implementation work continues.

## Executive Findings

1. The formal specs still contain the right product ambition.
   `PROGRAM_SPEC.md`, `SCHEMATIC_EDITOR_SPEC.md`, and `USER_WORKFLOWS.md`
   describe native schematic/board authoring, deterministic operations,
   schematic-to-board flow, and AI/CLI control surfaces.

2. Recent visible execution is dominated by imported-board GUI review.
   `M7` is explicitly scoped as a read-only route-proposal/imported-board
   review milestone. The implemented GUI follows that scope: it presents a
   board review surface, not an editor.

3. The codebase has more native authoring than the GUI suggests.
   The CLI includes broad native schematic and board mutation surfaces:
   symbols, wires, labels, buses, ports, board tracks, vias, zones, pads,
   components, text, keepouts, dimensions, stackup, nets, and net classes.

4. Native authoring is not productized where it matters.
   The authoring capability is mostly command/file based. It is not exposed as
   first-class GUI tooling, not available through the integrated terminal, and
   not available through the assistant/apply loop.

5. The central operation architecture is underbuilt relative to the specs.
   The `crates/engine/src/ops/` module is effectively empty, while specs depend
   on operations and transactions as the canonical mutation model. Some
   imported-board write operations have undo/redo in engine API write-op
   modules, but native project authoring is primarily implemented through CLI
   project-file mutation helpers.

6. The old embedded assistant bridge is retired as the primary AI architecture.
   GUI agent entry now routes through the PTY terminal and terminal-launched
   tools, with Datum context delivered through environment variables and
   discovery files instead of an assistant-only bridge.

7. The integrated terminal is now the first real AI/tooling substrate.
   It spawns a project-scoped shell through a PTY, streams terminal output,
   injects Datum context, and supports terminal-launched agents. Remaining work
   is terminal fidelity/session-history UX and broader command handoffs, not
   replacing a stub channel with a shell.

8. The `Terminal | Assistant` dock model is not grounded in agreed product
   mechanics.
   The intended terminal model is a real Linux terminal emulator where users
   can launch agents such as `codex`, `claude`, or other tools directly. A
   built-in assistant panel may exist later, but it should not be treated as
   the primary AI architecture.

9. A real terminal is necessary but not sufficient for Datum's AI strategy.
   The real power is EDA-native tooling: structured context, object IDs,
   queries, deterministic edit/proposal APIs, checks, diffs, artifacts, and
   provenance that make agents feel intentionally integrated rather than
   duct-taped onto the side of a generic EDA application.

10. The roadmap has allowed execution to orbit around interop/view fidelity.
   KiCad import fidelity and visual review are useful foundations, but they
   now consume the active frontier while native Datum authoring and schematic
   capture are not the visible product center.

## Product Identity Versus Execution

### Intended Product Identity

Evidence:
- `README.md` says Datum is an AI-native electronics design platform and not a
  wrapper around KiCad.
- `PROGRAM_SPEC.md` defines product identity as an AI-native EDA platform and
  warns not to infer product identity from implementation-slice limits.
- `SCHEMATIC_EDITOR_SPEC.md` defines a first-class schematic editor model and
  editing operation surface.
- `USER_WORKFLOWS.md` describes AI-assisted design creation from schematic to
  manufacturing outputs.

Assessment:
- The north-star intent exists in the repo.
- The issue is not that the repo lacks ambition on paper.
- The issue is that recent implementation focus and the visible GUI experience
  do not embody that ambition.

### Current Active Execution Identity

Evidence:
- `specs/progress/m7_opening.md` opens M7 as a visual review milestone, not a
  general-purpose GUI.
- `specs/M7_FRONTEND_SPEC.md` defines one read-only route-proposal review
  workspace.
- `docs/gui/M7_IMPORTED_BOARD_FIDELITY_PLAN.md` explicitly keeps M7 inside
  imported KiCad board review and excludes editing, apply flows, schematic
  review, and 3D work.
- `crates/gui-app/src/main.rs` describes the binary as a "Datum M7
  route-proposal review spike."
- The GUI blocks tool changes and mutating assistant actions in read-only M7
  review.

Assessment:
- The active GUI is intentionally not an EDA editor.
- The current visible product reads as a KiCad/imported-board review viewer.
- That is consistent with M7 scope, but it is misaligned with the desired
  product emphasis if M7 continues to dominate planning.

## Capability Inventory

### Native Schematic Model

Implemented foundations:
- `Schematic`, `Sheet`, `PlacedSymbol`, `SchematicWire`, `Junction`,
  `NetLabel`, `Bus`, `BusEntry`, `HierarchicalPort`, `NoConnectMarker`,
  `SchematicText`, and drawing primitives exist in
  `crates/engine/src/schematic/mod.rs`.
- Query helpers and tests exist for schematic summaries and object lists.
- ERC support exists over schematic data.

Gaps:
- The model exists, but there is no GUI schematic editor.
- Some spec-required operations remain deferred in progress notes, including
  power-symbol placement, sheet-instance lifecycle operations, and full
  deterministic annotation completion.
- The schematic authoring path is mainly CLI/native-file based, not interactive
  or assistant-driven.

### Native Board Model

Implemented foundations:
- `PlacedPackage`, `Track`, `Via`, `Zone`, `Net`, `NetClass`, `Keepout`,
  `Dimension`, and `BoardText` exist in `crates/engine/src/board/board_types.rs`.
- DRC, connectivity, unrouted computation, routing-substrate work, route
  proposal machinery, manufacturing exports, and native project persistence
  exist in varying degrees.

Gaps:
- The GUI does not expose normal PCB layout tools: draw track, place via, move
  component, edit zone, place component, edit board outline, etc.
- The CLI can mutate native board files, but there is no professional canvas
  authoring workflow.
- Interactive routing, snapping, constraints UX, object property editing, and
  undo/redo UX are not productized in the app.

### Native CLI Authoring

Implemented foundations:
- `ProjectCommands` contains broad native schematic and board authoring
  commands.
- Schematic command families include symbol placement/motion/rotation/mirror,
  fields, labels, wires, junctions, ports, buses, no-connects, text, and
  drawing primitives.
- Board command families include components, pads, tracks, vias, zones, text,
  keepouts, dimensions, outline, stackup, nets, net classes, and routing apply.
- CLI tests prove at least focused round trips for symbol, wire, board track,
  board text, and many related surfaces.

Gaps:
- This is a command/file mutation layer, not a unified product interaction
  model.
- It is not the same as a professional EDA editor.
- There is no single visible "Datum-native authoring" workflow in the GUI.

### Engine Operation Model

Implemented foundations:
- Imported-board engine write operations exist for selected board mutations:
  delete track/via/component, move/rotate component, set value/reference,
  set package, set net class, set design rule, component replacement, and
  undo/redo records.

Gaps:
- `crates/engine/src/ops/mod.rs` and `crates/engine/src/ops/impls/mod.rs` are
  effectively placeholders.
- Native project authoring commands bypass a central operation/transaction
  engine and write project JSON through CLI helpers.
- This weakens AI-first architecture because AI tools need one authoritative
  mutation path with validation, diffing, undo/redo, preview, apply/reject, and
  provenance.

### GUI

Implemented foundations:
- WGPU/winit GUI shell exists.
- `BoardReviewSceneV1`, render pipeline, hit testing, filters, selection,
  inspector, review panel, visual regression fixtures, and board-text editing
  affordances exist.
- The board review view can render real imported boards and text fixtures.

Gaps:
- The GUI is not a board editor.
- There is no schematic editor.
- There is no professional tool mode stack comparable to KiCad, much less
  Altium: placement tools, routing tools, draw tools, selection filters,
  property editing, live DRC, undo/redo, and command palette are not present
  as real authoring workflows.
- Existing board-text editing is a narrow exception and is not enough to
  establish an EDA editor.

### Assistant And Terminal

Implemented foundations:
- The old embedded assistant bridge (`scripts/datum_assistant_bridge.py`) has
  been retired; GUI agent entry now routes through terminal-launched tools.
- Terminal-launched shells receive Datum project/session/model context through
  environment variables and `.datum/gui-terminal-context.json`.
- The terminal discovery/context envelope advertises canonical Datum CLI
  commands for checks, proposals, artifacts, journal, resolver queries, and
  agent launch templates.

Gaps:
- The retired assistant bridge should not re-enter as the primary AI path.
- Some GUI actions still need terminal-command handoffs instead of private
  writers or assistant-only commands.
- PTY screen fidelity, attach/detach persistence, and richer session-history UX
  remain incomplete.
- Any remaining `Terminal | Assistant` split should be treated as transitional
  UI, not the agreed product mechanics.
- The intended primary AI path is a real terminal where users can launch code
  agents and EDA assistants directly against Datum project files, CLI tools,
  and MCP surfaces.
- The assistant is therefore not yet a working optional AI collaboration
  surface.

### Interop And Imported-Board Fidelity

Implemented foundations:
- KiCad import is substantial and under active hardening.
- M7 imported-board rendering has meaningful regression evidence.
- Visual fixtures and render-contract tests pass locally.

Gaps:
- Interop fidelity has become the active product center.
- KiCad has become the oracle for many recent development slices.
- This is useful for migration and fixture realism, but it should not define
  Datum's product identity or the next planning frontier.

## Drift Classes

### Drift 1: Visible Product Drift

The user-visible app is not an EDA editor. It is a review surface. This creates
a mismatch between the stated product and what a user can actually do.

Severity: high.

Reason:
- A professional EDA package must let users create and edit schematics and
  boards.
- Datum currently cannot draw a board line through the GUI even though the CLI
  can write a track to native JSON.

### Drift 2: Architecture Drift

Mutation authority is split:
- Some imported-board mutations live in engine API write-op modules with
  undo/redo.
- Native project authoring lives primarily in CLI project-file helpers.
- GUI board-text edits mutate JSON through protocol-side helpers.
- Assistant actions are blocked or selection-only.

Severity: high.

Reason:
- AI-first authoring requires a single mutation contract.
- Without that contract, every new surface risks becoming its own editing
  system.

### Drift 3: Milestone Weight Drift

M4 native authoring is marked closed for scoped CLI/native-file work, while
the visible active milestone is M7 imported-board review. This makes the repo
look as though native authoring is "done enough" even though the product
experience has not crossed the editor threshold.

Severity: high.

Reason:
- Closing scoped authoring at the CLI layer can hide the fact that product
  authoring remains incomplete.

### Drift 4: AI-First Surface Drift

AI support exists as context and bridge scaffolding, but not as a functioning
design-edit loop or a real terminal-launched agent workflow.

Severity: high.

Reason:
- Datum's AI story should be built on user-controlled terminal/CLI/MCP access,
  inspectable primitives, and optional assistant collaboration.
- Datum also needs EDA-native AI tooling so an agent can operate through
  structured design context, proposals, checks, diffs, and artifacts rather
  than scraping a GUI or acting like an external bolt-on.
- The current app blocks the exact mutation actions that would demonstrate a
  safe assistant workflow.
- The current terminal does not let a user launch normal tools or agents from
  inside Datum.

### Drift 5: Documentation Framing Drift

Docs are internally inconsistent:
- Some specs preserve the north star.
- M7 docs correctly say read-only review.
- Other docs/progress reports mark scoped native authoring as closed.
- A contributor can easily follow the active M7 trail and conclude that the
  product is an imported-board visualizer.

Severity: medium-high.

Reason:
- Code agents optimize for concrete active docs. If active docs emphasize
  import fidelity, agents will keep improving import fidelity.

## What Should Be Protected

The following work is valuable and should not be discarded:
- Native schematic and board data models.
- Native CLI authoring commands and tests.
- Deterministic project format and canonical JSON persistence.
- DRC/ERC/checking architecture.
- Route proposal and review/apply artifact machinery.
- GUI rendering substrate and scene contract.
- Visual regression harness.
- KiCad import as a migration and fixture source.

The course correction is not "throw away M7." It is to stop letting M7 define
the product identity.

## What Should Be Paused Or Demoted

Until the north-star review is complete:
- Do not open new KiCad fidelity tickets unless they unblock a specific native
  authoring, migration, or validation decision.
- Do not expand M7 visual polish as the default next work.
- Do not treat imported-board review screenshots as product-completeness
  evidence.
- Do not write a new M8 milestone spec yet.
- Do not let docs mark "authoring done" when only CLI/file mutation exists.

## Course-Correction Questions For Review

1. What is the minimum visible proof that Datum is an EDA editor, not a viewer?
2. Should native authoring be considered incomplete until it exists in the GUI,
   regardless of CLI coverage?
3. Should all native board/schematic mutations move behind one engine operation
   API before GUI authoring expands?
4. What should the embedded assistant be allowed to do in the first safe
   authoring slice?
5. Should the built-in assistant surface be deferred until the terminal is a
   real PTY-backed shell?
6. Is KiCad import now "fixture and migration infrastructure" rather than an
   active product frontier?
7. Which existing code should be elevated into the product path first: CLI
   native board commands, CLI schematic commands, route proposal apply, or
   terminal-launched agent command handoffs?
8. What definition of "AI-assisted EDA" should every future code agent load before
   selecting work?

## Recommended Review Agenda

The next human review should be a product/architecture review, not a task
planning session.

Suggested order:

1. Confirm or revise the north-star statement.
2. Decide whether active M7 work should be frozen after current regression
   cleanup.
3. Decide whether "native authoring closed" needs to be reframed as "CLI
   foundation closed; product authoring not closed."
4. Decide whether the central operation model must be repaired before more GUI
   editing is added.
5. Decide the first visible editor proof, but do not spec a full milestone yet.
6. Only after those decisions, write corrected roadmap/governance docs.

## Preliminary Conclusion

Datum has not lost the ingredients for an AI-first professional EDA tool. The
repo contains substantial native authoring, checking, routing, persistence, and
GUI infrastructure.

The project has lost execution alignment. The visible product and active
milestone energy are pointed at imported-board review, while the actual
north-star requires native schematic/PCB authoring and AI-mediated design
mutation to become the center.

The next step is a deliberate course-correction review with the project owner,
using this audit as the starting point.
