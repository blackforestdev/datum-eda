# M4 Detailed Progress Notes

This file holds detailed M4 implementation notes referenced by
`specs/PROGRESS.md`.

## Native format

Current native scaffold and read/query/check slices are implemented across
project scaffold creation, deterministic schema layout, and broad native query
surfaces.
Current truth boundary: `project.json` pool references are now resolved during
native board mutation flow, but only the truthful supported package-linked
subset is materialized into persisted board state. Current support is limited
to package silkscreen non-text primitives; package text and package-linked
mechanical persistence remain open.
The native inspect surface now also reports each declared pool reference with
its priority, resolved path, and current existence state so the native pool
contract is auditable without mutating project state. The native summary query
now reports the same resolved pool-reference detail for automation-facing read
parity.

Evidence anchors:
- CLI/native surface: `crates/cli/src/command_project.rs`
- Query dispatch: `crates/cli/src/command_exec.rs`
- Native rationale: `docs/NATIVE_FORMAT.md`

## Schematic operations

Implemented operation families include symbols, symbol fields, text, drawings,
labels, wires, junctions, hierarchical ports, buses/bus entries, and
no-connect markers with matching query coverage.

Evidence anchors:
- Command schemas: `crates/cli/src/cli_args.rs`
- Command handlers: `crates/cli/src/command_exec.rs`
- Tests: `crates/cli/src/main_tests_project_*.rs`

## Board operations

Implemented operation families include component placement/motion/locking and
part/package reassignment, pads with net assignment, tracks/vias/zones,
texts/keepouts/dimensions, outline/stackup, nets, and net classes. Component
placement and package reassignment now also resolve packages from native
project pool roots and persist the supported silkscreen non-text subset into
`board/board.json`.

Evidence anchors:
- CLI mutations: `crates/cli/src/command_project.rs`
- Engine board model: `crates/engine/src/board/mod.rs`
- Tests: `crates/cli/src/main_tests_project_board_*.rs`

## Schematic query parity

Native read/query currently covers key schematic topology and checking surfaces
through CLI, with full cross-surface parity still open.

Evidence anchors:
- Queries: `project query <dir> ...`
- Progress authority: `specs/PROGRESS.md`

## Forward annotation

Audit/proposal/review/apply flows are in place, including proposal artifact
export/inspect/filter/compare and review import/replace flows, with partial
explicit-input constraints still open.

Evidence anchors:
- CLI forward-annotation handlers/tests
- Proposal artifact pathways in `crates/cli/src/command_project.rs`

## Gerber export

Narrow RS-274X outline and manufacturing-layer slices are in place, including
single-layer copper, soldermask, silkscreen, paste, and mechanical output.
The current silkscreen subset covers authored board text plus explicit
component text, lines, arcs, circles, closed polygons, and open polylines via
deterministic stroke-only emission. Package-linked silkscreen support is now
truthfully limited to the non-text pool package subset materialized into those
persisted component silkscreen maps during placement/package reassignment. The
current mechanical subset covers board
keepout polygons, fixed-width board-dimension span lines, authored board
text, plus explicit component-local mechanical text, lines, closed polygons,
open polylines, circles, and arcs persisted in native board state. Full
layer-set and richer geometry parity remain open.
Truth boundary for next expansion: do not claim package-linked text or
package-linked mechanical persistence until the source package model carries
explicit truthful fields for those subsets and the resolved data is persisted
into native board state.

Evidence anchors:
- Engine export: `crates/engine/src/export/mod.rs`
- CLI tests: `main_tests_project_gerber_*.rs`

## Drill export

Deterministic drill CSV and narrow Excellon export/inspect/validate/compare
slices are in place. Broader fabrication semantics remain open.

Evidence anchors:
- Engine export: `crates/engine/src/export/mod.rs`
- CLI tests: `main_tests_project_excellon_*.rs`

## BOM export

Deterministic board-component BOM CSV export is implemented; richer purchasing
metadata remains open.

## PnP export

Deterministic board-component PnP CSV export and semantic compare slice are
implemented; richer manufacturing metadata remains open.

## Gerber comparison

Plan-vs-directory Gerber artifact comparison is implemented, a file-level
inspection surface is available for the currently supported RS-274X subset,
and semantic comparison covers the currently emitted subset including outline,
copper, soldermask, silkscreen, paste, and mechanical slices. The current
mechanical comparison subset covers keepout regions plus explicit persisted
component mechanical line strokes, closed polygons, open polyline strokes,
circle strokes, and chordized arc strokes. Full geometry/viewer-level
comparison remains open.
