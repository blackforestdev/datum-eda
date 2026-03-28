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
to package silkscreen non-text primitives, package `courtyard` on mechanical
layer `41`, resolved package pads in `component_pads`, and package
`models_3d`; the `component_silkscreen_texts` map remains schema-only and
empty by design in this slice. Package text and broader package-linked
mechanical persistence remain open. Package pad aperture geometry is now
persisted only when the resolved source padstack explicitly defines it;
the current copper/soldermask/paste Gerber slices consume only that
explicit-aperture persisted subset, and broader package-linked pad
manufacturing remains open. Package pads may now also persist optional
source-backed `drill_nm` from resolved padstacks. The current Excellon drill
and drill-hole-class reporting slices consume that drill-bearing persisted
subset as through holes spanning the outer copper pair, while CSV drill export
and broader package-linked drill semantics remain open.
The native inspect surface now also reports each declared pool reference with
its priority, resolved path, and current existence state so the native pool
contract is auditable without mutating project state. The native summary query
now reports the same resolved pool-reference detail plus aggregate
board-level persisted component silkscreen counts plus persisted
component-mechanical counts plus persisted component package-pad and
`models_3d` counts, plus how many components currently carry each persisted
subset, for automation-facing read parity, and the native board-components
query now reports per-component presence flags plus the currently
materialized silkscreen subset counts, persisted component-mechanical counts,
persisted component package-pad counts, and persisted component `models_3d`
counts. A direct `board-component-models-3d` query now exposes the persisted
`component_models_3d` refs for one component instead of only counts. A direct
`board-component-pads` query now exposes the persisted `component_pads`
subset for one component instead of only counts. A direct
`board-component-silkscreen` query now exposes the persisted package-linked
silkscreen subset for one component instead of only counts. A direct
`board-component-mechanical` query now exposes the persisted package-linked
mechanical subset for one component instead of only counts.

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

Implemented operation families include component placement/motion/locking,
part/package/layer/reference/value reassignment, pads with net assignment, tracks/vias/zones,
texts/keepouts/dimensions, outline/stackup, nets, and net classes. Component
placement and package reassignment now also resolve packages from native
project pool roots and persist the supported silkscreen non-text subset plus
package `courtyard` on mechanical layer `41`, resolved package pads, and
package `models_3d` into `board/board.json`. Resolved package pads now carry
circle/rect aperture geometry only when the loaded source padstack defines
explicit aperture fields; otherwise the persisted pad remains aperture-less.
They may also carry optional source-backed `drill_nm` when the resolved
padstack defines it.
The native query surface now includes a direct per-component readback for
persisted `models_3d` refs.
Current copper/soldermask/paste Gerber flows consume only the
explicit-aperture persisted package-pad subset.
The current Excellon drill and drill-hole-class slices also consume
drill-bearing persisted package pads as through holes on the outer copper
pair.
Fresh native project scaffolds now seed minimal default layers for top
copper, top soldermask, top silkscreen, top paste, and mechanical `41`, and
existing native projects can be retrofitted to that same canonical top-side
default stackup without overwriting conflicting occupied default layer IDs.

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
layer-set and richer geometry parity remain open. The deterministic Gerber
artifact plan is now executable as a batch export for the currently supported
stackup-backed subset via `export-gerber-set`, and that same subset can be
batch-validated in one directory via `validate-gerber-set` and batch-compared
semantically in one directory via `compare-gerber-set`. A read-only
`report-manufacturing` slice now summarizes the same persisted-state
manufacturing surface without writing artifacts, covering BOM/PnP component
counts, via-only CSV drill rows, Excellon/hole-class drill counts, and the
planned Gerber artifact filenames. `export-manufacturing-set` now writes that
same currently supported subset into one directory by composing the existing
BOM, PnP, via-only CSV drill, Excellon drill, and Gerber-set exporters.
`validate-manufacturing-set` now validates that same directory-level subset
using semantic BOM/PnP checks, deterministic CSV drill comparison, strict
Excellon validation, and per-artifact Gerber validation.
`compare-manufacturing-set` now semantically compares that same directory-level
subset using semantic BOM/PnP, semantic Excellon and Gerber comparison, plus
deterministic CSV drill comparison.
`manifest-manufacturing-set` now reports the deterministic expected
directory-level artifact set for that same persisted-state subset, including
artifact kind, comparison contract, and filename in stable order, without
writing files.
Copper, soldermask, and paste now also consume the persisted package-pad
subset from `component_pads` when, and only when, those pads carry explicit
circle/rect aperture geometry resolved from source padstacks.
Truth boundary for next expansion: do not claim package-linked text or
broader package-linked mechanical persistence until the source package model carries
explicit truthful fields for those subsets and the resolved data is persisted
into native board state.

Evidence anchors:
- Engine export: `crates/engine/src/export/mod.rs`
- CLI tests: `main_tests_project_gerber_*.rs`

## Drill export

Deterministic drill CSV export/inspect/validate/compare and narrow Excellon
export/inspect/validate/compare slices are in place. Excellon and
drill-hole-class reporting now include
drill-bearing package `component_pads` as through holes spanning the outer
copper pair; CSV drill export remains via-only. Broader fabrication semantics
remain open.

Evidence anchors:
- Engine export: `crates/engine/src/export/mod.rs`
- CLI tests: `main_tests_project_excellon_*.rs`

## BOM export

Deterministic board-component BOM CSV export, inspection, and semantic compare
are implemented; richer purchasing metadata remains open.

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
