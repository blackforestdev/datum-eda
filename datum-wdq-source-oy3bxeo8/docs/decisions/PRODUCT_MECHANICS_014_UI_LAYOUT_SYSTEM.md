# Product Mechanics 014: UI Layout System

> **Status**: Ratified interim adoption.
> **Scope**: Datum GUI shell, panels, docks, and future chrome layout.
> **Decision date**: 2026-06-29.

## Decision

Datum adopts `taffy` as the interim layout solver behind the existing retained
`wgpu` renderer.

The adoption is intentionally narrow:
- `taffy` solves frontend box geometry only.
- The existing retained renderer continues to own drawing, text, hit regions,
  and canvas/world rendering.
- Engine, daemon, protocol, and persisted project formats must not depend on
  `taffy`.
- Layout is expressed in logical pixels. Conversion to physical pixels remains
  a renderer/window boundary concern.

## Rationale

The current GUI failures came from hand-positioned panel content and fixed card
heights. That style cannot scale to a professional EDA application because
populated states, docked states, fractional scale factors, and future schematic
or production panes will continuously create new overlap and clipping defects.

Datum's differentiator is EDA authoring, review, standards compliance,
production output, and AI/terminal-assisted workflows. Generic flex/grid box
solving is infrastructure. `taffy` is renderer-agnostic Rust infrastructure
that can feed solved rectangles into Datum's existing renderer without changing
the engine model.

## Adoption Depth Target

This decision is not satisfied by merely linking `taffy` or solving the outer
application shell.

The target state is:
- Taffy solves shell geometry.
- Taffy solves intra-panel card/content geometry.
- Dynamic card heights derive from content, not fixed constants.
- Magic-number `x + N` / `y + N` offsets are retired from ordinary panel
  layout.
- Manual offsets are allowed only for bounded drawing details inside a solved
  rectangle, or as documented exceptions with tests.

## Initial Implementation Boundary

The first integrated slice kept the public `ShellLayout` API stable and routed
top-level shell rectangle solving through `taffy` inside `datum-gui-render`.
The follow-on slice begins moving panel internals to Taffy with the `Project`
card content stack, derived `Filters` placement, and the right-side
`Inspector`/`Review` card stack. The bottom dock tab strip and content envelope
now also use a solver-backed layout helper. The Inspector's optional
review/evidence/status detail rows are solver-backed to prevent populated-state
status rows from colliding with the Review card. The Outputs lane now has a
solver-backed header/primary-command/body envelope. Its upper sections
(`FocusedArtifact`, `Checks`, `Actions`) and lower production sections
(`Panels`, `Plans`, `Jobs`) are allocated by solved body-section rectangles
before their rows draw. Inspector authored-object property rows and Outputs
dock rows now derive row spacing and hit-region height from rendered text
metrics instead of fixed row-height literals.

Accepted initial seam:
- workspace dependency: `taffy`
- consuming crate: `crates/gui-render`
- first solved object: top-level shell grid
- first regression gate: shell grid contract test
- first scale-factor seam: `datum-gui-app` feeds physical surface size and
  `winit` scale factor into `ShellLayout::for_surface`, which solves logical
  shell geometry before returning renderer-facing physical rectangles

This is not yet a complete UI system. It is the adoption path that lets Datum
migrate panel internals away from magic-number offsets incrementally. Current
known remaining work includes replacing interim token constants with generated
or measured token data where appropriate, and expanding multi-scale PNG goldens
beyond the first checked HiDPI fixture.

## Non-Goals

This decision does not:
- adopt a full GUI framework
- replace `wgpu`
- solve docking, focus, widget state, keyboard traversal, or theme design
- make `taffy` part of engine or protocol contracts
- require all existing panels to migrate in one change

## Open Forks

1. Docking manager.
   Recommendation: keep dock/splitter persistence Datum-owned above `taffy`.
   `taffy` should solve pane rectangles after the docking layer declares the
   current pane tree.

2. Token toolchain.
   Recommendation: author design tokens as tracked JSON/Markdown contract data,
   then compile or mirror them into Rust constants. Do not introduce a runtime
   token dependency until the token schema stabilizes.

3. Scale-factor policy.
   Recommendation: layout in logical pixels, accept fractional scale factors,
   and multiply at the renderer boundary. Add visual and invariant tests for
   scale factors `1.0`, `1.25`, `1.5`, and `2.0` before claiming HiDPI closure.
   Current status: shell/render-entry scale handling is implemented, and the
   visual harness can render/check scale-aware fixtures. The multi-scale smoke
   test exists, and `text-density-repro` has checked scale-suffixed PNG
   goldens for `1.0`, `1.25`, `1.5`, and `2.0`.

## Governance

New GUI work that adds or reshapes shell chrome, panels, docks, inspector
sections, property grids, or status strips must either:
- use the solver-backed layout contract, or
- document a bounded exception with a migration path.

Regression gates must include populated states, not only empty fixture states.
At minimum, layout tests must assert:
- no card-to-card overlap
- dynamic content stays inside the owning card
- panel hit regions stay inside their owning panel
- shell rectangles remain valid at supported window sizes
