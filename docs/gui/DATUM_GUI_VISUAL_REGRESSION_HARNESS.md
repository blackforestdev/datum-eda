# Datum GUI Visual Regression Harness

> **Status**: Active implementation brief. Layer A manifest parsing, image
> diff utilities, and offscreen capture are implemented behind the
> `datum-gui-render/visual` feature.
> **Research source**:
> [GUI_VISUAL_REGRESSION_RESEARCH.md](/home/bfadmin/Documents/datum-eda/research/gui-visual-regression/GUI_VISUAL_REGRESSION_RESEARCH.md)

Datum needs visual regression coverage that protects board rendering, text
fidelity, layer/UI behavior, and full application layout without becoming
fragile window-manager screenshot theatre.

This brief adopts the research recommendation: **two layers, with strict
separation of duties**.

## Doctrine

- Visual regressions are product regressions.
- Text fidelity, board rendering, and UI layout require image-based proof.
- Renderer/canvas fidelity must not depend on a real desktop window.
- Full-GUI screenshots are valuable, but they are not the primary defence for
  board-canvas pixels.
- Golden updates are engineering decisions and must be explainable.
- Local visual wins are not valid if they bypass the responsible rendering or
  semantic contract.

## Architecture

### Layer A: Render-Scene Goldens

Layer A is the primary defence.

It renders `gui-render` output offscreen into a `wgpu` texture, copies the
texture to CPU memory, writes a PNG, and compares it to a golden.

Layer A must not use:

- `winit`
- a desktop window
- a compositor
- a display server
- manual screenshots

Layer A protects:

- board text fidelity
- copper rendering
- silkscreen / mask / paste / edge-cuts rendering
- layer visibility and ordering
- airwire rendering
- proposed/diagnostic/review overlays
- selection and hover visual semantics once fixture support exists

Layer A does not protect:

- shell panels
- menu bars
- inspector layout
- terminal / assistant panels
- platform window behavior

### Layer B: Full-GUI Goldens

Layer B is a thinner upper layer.

It launches the real `datum-gui` shell inside a pinned headless display
environment and captures the full application window.

Layer B is in M7 scope, but it must be triggered deliberately. It is not a
replacement for Layer A and must not be used to debug board-canvas fidelity that
Layer A already owns.

Layer B protects:

- sidebars
- inspector layout
- toolbar/menu layout
- status and terminal panels
- modal/popover layering
- shell theme regressions
- full-window composition

Layer B must not become the main canvas-fidelity test system. Canvas pixels are
owned by Layer A.

Layer B owns product-shell proof:

- application launch composition
- dock/sidebar/inspector layout
- toolbar and status-strip layout
- visual interaction state that only exists in the shell
- full-window regressions caused by real `winit`/window integration

Layer B must not own:

- board-text glyph fidelity
- copper/mask/paste/edge-cuts fidelity
- renderer geometry correctness
- imported-board semantic correctness
- source-CAD parity judgments

Those remain Layer A / engine / protocol responsibilities.

## Layer B Tripwires

Build Layer B when one of these conditions is true:

- M7 work changes shell layout, sidebars, inspector, toolbar/menu, dock panels,
  terminal/assistant panels, modal layering, or status-strip composition.
- Taffy-backed layout contracts or design-token-derived dimensions change.
- A bug can be reproduced in the real application window but not through Layer A
  offscreen rendering.
- A UI regression depends on `winit`, surface sizing, swapchain/window size,
  DPI/scale factor, input focus, dock state, or full-window composition.
- A reviewer needs image proof that the whole product shell still composes
  correctly after a GUI infrastructure change.
- Layer A passes but the user-visible application window is wrong.

Do not build Layer B for:

- isolated board-canvas pixel regressions
- text glyph quality regressions
- engine-owned semantic import bugs
- renderer-owned geometry bugs already reproducible in Layer A

## Layer B First Slice

When a tripwire fires, the first Layer B implementation must stay minimal.

Required first slice:

- one full-window fixture: implemented as the shell launch fixture in
  `crates/gui-app/tests/visual_shell.rs`
- one pinned window size: implemented through `--window-size 1280x768`
- one deterministic native project root: starts with
  `crates/engine/testdata/golden/text/native/text-fidelity-repro`
- one screenshot output path: implemented through `--screenshot-out <path>`
- one ignored/manual test: implemented as
  `layer_b_shell_screenshot_first_slice`
- the same artifact retention policy as Layer A: the ignored smoke test writes
  to `/tmp` and removes its successful output

Target app contract:

```sh
datum-gui \
  --project-root <fixture> \
  --visual-test \
  --window-size 1280x768 \
  --screenshot-out <path> \
  --exit-after-screenshot
```

Target test command:

```sh
cargo test -p datum-gui-app --features visual --test visual_shell -- --ignored --nocapture
```

Layer B should start with `ssim` or masked comparison only after the dynamic
regions are identified. Exact full-window comparison should not be assumed
until the headless environment is pinned and proven stable.

Current first-slice implementation captures the full Datum shell from the real
`datum-gui` runtime after a `winit` window has been created at the requested
size. The screenshot is rendered through the production `Renderer` using the
runtime's app state and surface format, then read back to PNG with explicit
BGRA-to-RGBA conversion when required by the platform surface format.

This is not yet an authoritative golden comparison. It is the Layer B smoke
tripwire needed before inspector/sidebar work. Golden comparison, masks, and
CI authority remain blocked until the environment contract below is pinned.

## Layer B Environment Contract

Layer B cannot become authoritative until the execution environment is pinned.

The visual shell lane must specify:

- display backend (`xvfb` first is acceptable; pinned Wayland/headless can
  replace it later)
- Mesa/lavapipe version
- Vulkan/OpenGL backend choice
- font set and fontconfig behavior
- window size and scale factor
- animation/clock disabling behavior
- screenshot timing/readiness signal

Until that exists, Layer B tests must remain ignored/manual.

## Phase 1 Scope

Phase 1 implements **Layer A only** for board text.

Seed fixtures:

- `text-intent-repro`
- `text-fidelity-repro`
- `text-transform-repro`
- `text-density-repro`

Stable fixture projects live under:

```text
crates/engine/testdata/golden/text/native/
```

Layer A manifests and rendered PNG goldens should live under:

```text
crates/gui-render/testdata/golden/board/
```

The engine projects and renderer goldens intentionally live in different
crates. The projects are engine fixtures; the rendered images are renderer
fixtures.

## Phase 1 Implementation Plan

### Step 1: Visual Feature Gate

Add a `visual` feature to `crates/gui-render`.

Default tests must remain fast and must not compile the visual harness unless
explicitly requested.

Implemented shape:

```toml
[features]
visual = ["dep:image", "dep:pollster"]

[dependencies]
image = { version = "0.25.10", optional = true }
pollster = { workspace = true, optional = true }
```

### Step 2: Offscreen Renderer

Add an offscreen capture module in `gui-render`.

Target API:

```rust
OffscreenRenderer::new(width_px, height_px)
OffscreenRenderer::render_workspace(state, camera) -> image::RgbaImage
```

Required properties:

- render to `Rgba8UnormSrgb`
- support MSAA resolve when the app renderer uses MSAA
- copy texture to a padded readback buffer
- strip `wgpu` row padding deterministically
- wait for GPU work before reading pixels

Implemented module:

```text
crates/gui-render/src/visual_capture.rs
```

The capture path uses the production `Renderer`, `PreparedScene`, and
`RetainedScene`. It owns only the `wgpu` device setup and texture readback. This
is the critical ownership boundary: Layer A must prove the real renderer, not a
second visual-test renderer.

### Step 3: Fixture Manifest Loader

Add a TOML fixture manifest loader.

Each fixture manifest must define:

- input project path
- fixture lane and suite
- viewport size
- camera center and zoom
- layer visibility
- selection state
- review state
- theme
- render backend policy
- determinism settings
- golden filename and diff policy
- blank-scene sanity threshold
- provenance fields

Nothing important may be implicit.

Implemented module:

```text
crates/gui-render/src/visual_manifest.rs
```

The current loader is intentionally fixed-schema and self-contained. It avoids a
general TOML dependency until the manifest grammar outgrows the small Phase 1
shape.

### Step 4: Image Diff Harness

Add a visual diff module.

Required policies:

- `exact`
- `tolerance`
- later `ssim`
- later `masked`

Every failure must emit:

- `<fixture>.actual.png`
- `<fixture>.diff.png`
- `<fixture>.report.txt`

The report must include:

- diff policy
- differing pixel count and percentage
- max channel delta
- SSIM when computed
- output paths

Implemented module:

```text
crates/gui-render/src/visual_diff.rs
```

### Step 5: Text Fixture Manifests

Add four Layer A manifests under:

```text
crates/gui-render/testdata/golden/board/
```

The manifests point at:

```text
crates/engine/testdata/golden/text/native/<fixture-name>/
```

Initial fixture suite:

- `text-intent-repro.fixture.toml`
- `text-fidelity-repro.fixture.toml`
- `text-transform-repro.fixture.toml`
- `text-density-repro.fixture.toml`

Implemented manifest directory:

```text
crates/gui-render/testdata/golden/board/
```

### Step 6: Bless Workflow

Add a bless path that renders a fixture and writes/updates its golden PNG.

Implemented command:

```sh
cargo run -p datum-gui-render --features visual --bin datum-visual-fixture -- bless crates/gui-render/testdata/golden/board/text-density-repro.fixture.toml
```

Implemented behavior:

- accepts an explicit fixture manifest path
- writes the golden deterministically through the Layer A offscreen renderer
- runs the blank-scene sanity check before writing
- removes stale `.actual.png`, `.diff.png`, and `.report.txt` artifacts after a
  successful bless
- supports `bless-all` for controlled batch generation

Still required at review/commit time:

- provide the human-readable reason in the commit message
- include the expected `VISUAL-REBLESS:` footer text

Longer-term behavior:

- run inside the pinned visual-CI container
- embed provenance hashes in PNG metadata
- update manifest provenance fields

### Step 7: Visual Tests

Add `gui-render` visual tests behind `--features visual`.

Each test:

- loads one fixture manifest
- builds the board scene from the fixture project
- renders offscreen
- compares against the golden
- writes failure artifacts beside the golden

Default `cargo test` must not run these.

Implemented library runner:

```text
crates/gui-render/src/visual_runner.rs
```

Implemented command:

```sh
cargo run -p datum-gui-render --features visual --bin datum-visual-fixture -- check crates/gui-render/testdata/golden/board/text-density-repro.fixture.toml
```

Batch commands:

```sh
cargo run -p datum-gui-render --features visual --bin datum-visual-fixture -- bless-all
cargo run -p datum-gui-render --features visual --bin datum-visual-fixture -- check-all
cargo run -p datum-gui-render --features visual --bin datum-visual-fixture -- clean-all
```

## Artifact Retention

Layer A generated artifacts are bounded by policy.

- `.fixture.toml` and `.golden.png` are source artifacts and must never be
  removed by cleanup.
- `.actual.png`, `.diff.png`, and `.report.txt` are generated artifacts.
- Before a fixture check starts, stale generated artifacts for that fixture are
  removed.
- After a fixture check passes, generated artifacts for that fixture are
  removed.
- After a fixture check fails, generated artifacts are retained for debugging.
- After a fixture bless succeeds, stale generated artifacts for that fixture are
  removed.
- `clean <fixture.toml>` and `clean-all` remove only generated artifacts.

This policy prevents silent disk growth while preserving failure evidence.

## Diff Policy

Layer A default:

- start with `exact`
- escalate only with evidence
- any tolerance must be documented in the manifest

Layer B default:

- `ssim`
- masks allowed only for known dynamic regions
- masks are reviewed artifacts

## Determinism Requirements

Layer A must control:

- surface size
- viewport/camera
- layer state
- theme
- fonts
- random seed
- clock
- animation state
- renderer backend configuration

Layer A offscreen capture must prefer the fallback/software `wgpu` adapter.
Hardware/vendor adapters are allowed only as availability fallback for local
smoke execution. They are not golden authority because they can produce stable
but different antialias/color quantization and clipping-boundary pixels.

CI golden authority should be scoped to:

```text
x86_64-linux-gnu
```

The visual-CI container must pin:

- Mesa / lavapipe
- Vulkan ICD
- `wgpu` version through Cargo
- vendored fonts

GPU/vendor-driver output is not golden authority.

## Rebless Policy

A rebless is not a housekeeping task. It is a design/engineering change.

Every rebless commit must include a footer block:

```text
VISUAL-REBLESS:
- fixture: text-density-repro
  reason: manufacturing default now resolves to Inter outline text
```

Bulk reblesses require a stronger explanation and should be gated separately.

## Non-Goals For Phase 1

Phase 1 does not include:

- full GUI/window screenshots
- CI workflow authoring
- interaction replay
- shell screenshot masks
- imported-board visual suites beyond the text fixtures
- arbitrary screenshot SaaS integration

## Acceptance Criteria

Phase 1 is complete when:

- Layer A can render the four native text fixtures offscreen: complete.
- Each fixture has a manifest: complete.
- Each fixture has a golden PNG: complete.
- Visual tests run only with `--features visual`: complete.
- Failures emit actual/diff/report artifacts: complete for checked fixtures.
- Default test runs remain unaffected: complete.
- The text-engine fidelity fixtures are protected by image comparison, not only
  manual screenshots: complete through the ignored Layer A integration test.

## Verification Commands

Default renderer build:

```sh
cargo check -p datum-gui-render
```

Visual feature build:

```sh
cargo check -p datum-gui-render --features visual
```

Non-GPU visual unit tests:

```sh
cargo test -p datum-gui-render --features visual visual_manifest -- --nocapture
cargo test -p datum-gui-render --features visual visual_diff -- --nocapture
cargo test -p datum-gui-render --features visual visual_capture::tests::align_to_preserves_aligned_rows -- --nocapture
cargo test -p datum-gui-render --features visual visual_runner -- --nocapture
```

Explicit local offscreen smoke test:

```sh
cargo test -p datum-gui-render --features visual visual_capture::tests::offscreen_capture_renders_fixture_workspace -- --ignored --nocapture
```

Explicit Layer A golden test:

```sh
cargo test -p datum-gui-render --features visual --test visual_goldens -- --ignored --nocapture
```

The same test target includes the explicit multi-scale HiDPI smoke:

```sh
cargo test -p datum-gui-render --features visual --test visual_goldens board_text_multi_scale_visual_smoke_renders_nonblank -- --ignored --nocapture
```

Layer A manifests may opt into scaled goldens with:

```toml
[viewport]
ui_scale_factors = [1.0, 1.25, 1.5, 2.0]
```

When a fixture declares more than one scale factor, non-legacy artifacts and
goldens are written with scale suffixes, for example:

```text
text-density-repro.scale-1_25.golden.png
text-density-repro.scale-1_25.actual.png
text-density-repro.scale-1_25.diff.png
text-density-repro.scale-1_25.report.txt
```

Explicit Layer B shell smoke test:

```sh
cargo test -p datum-gui-app --features visual --test visual_shell -- --ignored --nocapture
```

Generate goldens:

```sh
cargo run -p datum-gui-render --features visual --bin datum-visual-fixture -- bless-all
```

Check goldens:

```sh
cargo run -p datum-gui-render --features visual --bin datum-visual-fixture -- check-all
```

Clean generated visual artifacts:

```sh
cargo run -p datum-gui-render --features visual --bin datum-visual-fixture -- clean-all
```
