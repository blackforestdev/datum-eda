# UI Layout System Contract

> **Status**: Interim contract for Taffy-backed GUI layout.
> **Applies to**: `datum-gui-render`, `datum-gui-app`, GUI specs, and visual
> regression gates.

## Contract

Datum GUI layout is a frontend-owned, solver-backed system.

The current adopted solver is `taffy`. It computes logical-pixel rectangles for
application chrome and panel content. The retained `wgpu` renderer consumes
those rectangles and remains responsible for drawing.

Shell-only Taffy use is an adoption seam, not completion. The contract target
is solver-backed intra-panel layout for normal cards, rows, tool groups,
inspector sections, review sections, and dock content.

## Ownership

Allowed dependency direction:
- `datum-gui-render` may depend on `taffy`.
- `datum-gui-app` may consume solved layout through `ShellLayout` and future
  frontend layout structs.
- `datum-gui-protocol`, `eda-engine`, and `eda-engine-daemon` must not depend on
  `taffy`.

The engine owns semantic state. The GUI layout system owns presentation
geometry only.

## Coordinate Policy

All layout solver inputs and outputs are logical pixels.

Renderer boundary rules:
- window scale factor is not baked into solver-owned logical geometry
- physical-pixel conversion happens only where `winit`/`wgpu` surfaces,
  scissor rectangles, render targets, or glyph atlas rasterization require it
- fractional scale factors must be treated as real values, not silently rounded
  in layout code

Current implementation status:
- `datum-gui-app` captures the `winit` window scale factor and routes physical
  surface dimensions through `ShellLayout::for_surface`.
- `ShellLayout::for_surface` converts the physical surface to logical pixels,
  solves the shell in logical coordinates, then scales the renderer-facing
  rectangles back to physical pixels at the GUI boundary.
- `PreparedScene::from_workspace_for_surface` and
  `RetainedScene::from_workspace_for_surface` are the render-path entry points
  for scale-aware scene construction.
- Text run sizes are scaled at the renderer boundary so glyph rasterization and
  solved shell geometry consume the same scale factor.
- This closes the first shell-layout/render-entry scale-factor contract. It
  does not yet close every text-measurement or panel-row-height concern.

## Layout Primitive Set

Initial allowed layout primitives:
- fixed-size shell regions
- flexible viewport region
- vertical stacks
- horizontal rows
- grid rows/columns
- gaps and padding from design tokens
- min/max size constraints

Deferred primitives:
- arbitrary user-authored dock trees
- persisted split-pane graphs
- floating tool palettes
- modal layering
- virtualized property tables

## Design Tokens

Panel layout must migrate away from raw magic numbers.

Token categories:
- spacing
- card padding
- panel gutter
- row height
- typography size
- color role
- layer depth
- hit target size

Until a generated token pipeline exists, Rust constants may mirror this
taxonomy, but new constants should be named semantically rather than by local
implementation detail.

Current implementation status:
- `datum-gui-render` now carries semantic interim constants for card margins,
  padding, row heights, stack gaps, and project-panel rows.
- The Project panel uses those constants as Taffy row inputs instead of local
  one-off row-height literals.
- Text measurement is not yet the row-height source of truth; these constants
  are the migration step between raw offsets and a generated/measured token
  pipeline.

## Invariants

Every solver-backed layout must be testable without opening a real window.

Required invariants:
- child rectangles stay within parent rectangles unless explicitly marked as an
  overlay
- sibling cards do not overlap
- sidebars, viewport, and dock stay inside the window
- interactive hit regions stay inside their owning visible region
- populated states are tested, including last-command status, selected review
  action, selected evidence, and active dock content

## Visual Gates

Visual regression coverage must include:
- empty/default state
- populated inspector/review state
- active authoring tool state
- terminal dock open state
- scale factors `1.0`, `1.25`, `1.5`, and `2.0`

Passing the empty fixture at scale `1.0` is not sufficient.

Current implementation status:
- the Layer A fixture manifest supports `viewport.ui_scale_factors`
- non-`1.0` fixture artifacts use scale-suffixed filenames
- `VisualFixtureRun::render_actual_at_scale` routes offscreen rendering through
  the scale-aware retained/prepared scene constructors
- the ignored `board_text_multi_scale_visual_smoke_renders_nonblank` test
  renders scale factors `1.0`, `1.25`, `1.5`, and `2.0`
- `text-density-repro.fixture.toml` opts into scale factors `1.0`, `1.25`,
  `1.5`, and `2.0` with checked-in scale-suffixed PNG goldens
- broader multi-scale expansion across every visual fixture remains follow-on
  work, not a prerequisite for the first usable HiDPI regression gate

## Migration Rule

Existing hand-positioned code may remain temporarily, but new GUI chrome should
not add fresh independent `x + N` / `y + N` layouts without either:
- using the Taffy-backed layout seam, or
- adding a regression test that proves the manual geometry cannot overlap in
  populated states.

Completion gate:
- all persistent shell cards use solver-backed parent/child rectangles
- populated-state invariant tests cover each persistent card family
- remaining manual offsets are documented as drawing-local, not layout-owning

Implemented slices:
- shell grid solved by Taffy
- Project card content stack solved by Taffy
- Filters card placement flows below the solved Project card
- Filters card toggle rows, layer rows, and summary rows solved by Taffy
- Inspector/Review card stack solved by Taffy
- Inspector optional review/evidence/status detail rows solved by Taffy with
  row height derived from rendered key/value text metrics
- Inspector authored-object property rows and their hit regions derive vertical
  spacing from rendered key/value text metrics instead of fixed `18px` row
  assumptions
- bottom dock tab strip and content envelope solved by Taffy
- Outputs dock lane header, primary command row, and body envelope solved by
  Taffy
- Outputs dock upper body section envelopes (`FocusedArtifact`, `Checks`,
  `Actions`) solved by Taffy and clipped to the available body rect
- Outputs dock lower production section envelopes (`Panels`, `Plans`, `Jobs`)
  solved by Taffy and clipped to the available body rect
- Outputs dock row heights and row hit regions derive from rendered output text
  metrics instead of fixed 14px assumptions
- populated inspector status regression asserts REVIEW starts below INSPECTOR
- shell scale-factor regression asserts logical solve then physical scaling
- visual fixture runner supports scale-aware render/check/bless paths
