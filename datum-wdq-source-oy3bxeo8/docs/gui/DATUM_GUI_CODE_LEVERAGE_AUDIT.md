# Datum GUI Code Leverage Audit

> **Status**: Governed active planning audit.
> **Authority**: Companion to
> `docs/decisions/PRODUCT_MECHANICS_019_GUI_PRODUCT_MODEL.md` and
> `docs/gui/DATUM_GUI_PRODUCT_SPEC.md`.
> **Scope**: Classifies existing GUI code and docs as keep, adapt, replace,
> delete/deprecate, or missing for the recovered GUI product model.

## Purpose

Identify what current GUI work can be reused and what must be changed before
Datum can become a human-driven EDA application. This audit is not an
implementation plan; it is the bridge from current M7 review-shell code to the
product model in decision 019.

## Summary

The current GUI is a narrow M7 route-review/supervision shell. It has useful
rendering, layout, terminal, context, and visual-test infrastructure, but the
product model is incomplete:

- no real application menu;
- no document/view model;
- no library/schematic GUI workflow;
- board fidelity still reads as proxy/review geometry;
- authoring tools synthesize CLI strings and route them through the terminal;
- AGENTS remains visible as a dock concept even though agent work should be
  terminal-session based;
- supervision telemetry is visible in the product surface without a stable EDA
  placement.

## Keep

These assets are structurally useful and should remain part of the recovery
unless implementation proves otherwise.

| Area | Keep Rationale |
|------|----------------|
| `crates/gui-render` retained `wgpu` renderer | Custom renderer is appropriate for a serious EDA viewport. |
| `crates/gui-render/src/visual_capture.rs`, visual runner/diff/manifest | Required foundation for screenshot-golden acceptance. |
| Taffy-backed layout helpers and layout invariant tests | Align with decision 014; must be expanded, not removed. |
| Design tokens and design-book checks | Align with decision 015; visual system should feed the recovered shell. |
| Hit-region and picking infrastructure | Useful for selection/inspector/editor tools. |
| `gui-protocol` versioned view-model boundary | Correct separation between engine data and renderer/app state. |
| Native resolver-backed board scene loading | Correct direction: GUI should consume resolver/materialized engine truth. |
| `TerminalSessionRegistry` PTY/session infrastructure | Valuable, but needs product cleanup and possibly terminal-core hardening. |
| Context envelope/session metadata plumbing | Useful for terminal-launched tools/agents and reproducible workflows. |

## Adapt

These pieces have useful material but need product reshaping.

| Area | Adaptation Needed |
|------|-------------------|
| `ReviewWorkspaceState` | Broaden from route-review state into document/view workspace state, or wrap it inside a larger app model. |
| `BoardReviewSceneV1` rendering | Evolve toward PCB-native board scene/rendering with real footprint/pad/layer vocabulary. |
| Project/Filters/Inspector/Review panels | Rework into product panels: project browser, appearance/layers, inspector/properties, checks/issues, outputs. |
| Outputs dock | Keep production/artifact/check content; replace implementation-telemetry posture with EDA status/output language. |
| `GuiSupervisionSnapshot` | Adapt selected counts/status into status bar or diagnostics, or delete journal-tip/read-only telemetry from primary UI. |
| Terminal command handoff structures | Useful for explicit "run this command" affordances, but not as the implementation path for GUI editor tools. |
| Board text quick-edit terminal helpers | Replace with direct GUI property edits through engine operations; keep as temporary compatibility only if clearly labeled. |
| Current tool-state gesture preview | Keep the concept of active tool/gesture/preview; change finish path to engine operation/proposal calls. |
| Imported KiCad scene import helpers | Useful as fixture/migration/render-fidelity inputs, not active product authority. |

## Replace

These areas need a different architecture or product model.

| Area | Replacement Target |
|------|--------------------|
| Top-level shell without menus | Replace with normal desktop application shell: menus, toolbar, document switcher, status bar. |
| Single fixed M7 workspace | Replace with document/view workspace model for project, library, schematic, PCB, checks, manufacturing, terminal. |
| GUI authoring via terminal-injected CLI strings | Replace with GUI action model -> engine native write/proposal builder -> `OperationBatch` -> commit/proposal. |
| Six-button board tool menu | Replace with editor-specific tool palettes and command palette entries. |
| Debug-like board status strings | Replace with status bar and inspector/properties model. |
| M7 route-review-centered information architecture | Replace with product IA driven by library/schematic/PCB/manufacturing documents. |

## Delete Or Deprecate

These should not survive as first-class product surfaces unless explicitly
re-ratified.

| Area | Reason |
|------|--------|
| Separate `AGENTS` dock tab | Agents should be terminal sessions/launchers by default; separate assistant surface needs a future decision. |
| Primary-workspace journal-tip/read-only telemetry | Useful diagnostics, but not a product-center EDA surface. |
| Any GUI private writer path | Violates canonical edit model. |
| Any "terminal as editor backend" path | Makes the GUI a macro recorder rather than an editor. |
| Historical M7 imported-board fidelity plan as roadmap authority | Useful diagnosis only; import remains fixture/migration infrastructure. |

## Missing

The following are not present at the required product level.

| Missing Surface | Required Before Claiming Product Usability |
|-----------------|--------------------------------------------|
| Application menu | File/Edit/View/Place/Route/Project/Checks/Manufacturing/Window/Help. |
| Document model | Project/library/schematic/PCB/check/manufacturing/terminal documents and views. |
| Project open/import/save/export GUI commands | Visible shell commands, not CLI-only workflows. |
| Library browser/preview | Native pool objects inspectable without UUID/CLI knowledge. |
| Native schematic renderer | Sheet/symbol/wire/label/port/bus/junction/no-connect rendering from engine state. |
| Native board fidelity baseline | Footprints, pads, copper, vias, zones, silk/text, layers, selection, and fixture screenshots. |
| GUI operation dispatcher | Direct engine/proposal calls from GUI actions. |
| Property inspector editing | Contextual property edits through operations. |
| Command palette | Discoverable action/search layer distinct from terminal. |
| Real status bar | Coordinates, active tool, selection, model/check/output state. |
| Human-review visual gate | Accepted fixture screenshots and owner sign-off per GUI milestone. |

## Current Phase 1 Leverage

For the first recovery slice, reuse:

- offscreen screenshot infrastructure;
- `wgpu` renderer and retained scene model;
- layout solver and design tokens;
- native resolver-backed scene loading;
- hit testing and selection basics;
- terminal PTY infrastructure as a dock/shell surface.

Do not reuse as product patterns:

- M7-only panel taxonomy;
- terminal-injected authoring;
- AGENTS dock;
- supervision telemetry as primary product UI.

## Phase 1 Technical Questions

These must be answered before coding Phase 1:

1. `datum-test` is the Phase 1 human-reviewed visual target; confirm whether a
   repo-vendored native fixture must be derived from it before implementation.
2. Is Phase 1 based on imported KiCad fixture rendering, native board fixture
   rendering, or both?
3. What exact footprint/pad/layer subset must be rendered to pass owner review?
4. Which shell toolkit level is required first: menu bar only, menu + toolbar,
   or menu + toolbar + document tabs?
5. Which current supervision fields belong in a status bar versus diagnostics?
6. What is the first direct GUI operation proof after render fidelity: select +
   inspect only, or select + property edit?

## Governance

This audit is governed and classified in `specs/spec_governance_manifest.json`.
It should be updated when GUI implementation changes what is reusable or when a
formerly missing surface lands.
