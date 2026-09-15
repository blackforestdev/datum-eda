# GUI Visual Regression — Architecture Research

> Driver: Datum needs a visual regression system that protects board
> rendering, text fidelity, layer/UI behavior, and full app layout —
> deterministic enough for CI, debuggable when failing, and not
> dependent on fragile window-manager behavior.
>
> User preliminary guidance (architectural floor — not subject to
> re-evaluation): two-layer architecture; offscreen/renderer goldens
> as the primary defence; full-GUI screenshots as a thinner second
> layer; the existing text fixtures at
> `crates/engine/testdata/golden/text/native/` are the first golden
> suite, not a one-off.
>
> Quoted verbatim:
>
> > - Do not recommend a quick full-window-only screenshot harness as the primary system.
> > - Prefer renderer/offscreen goldens for board-canvas fidelity.
> > - Add full GUI screenshots as a second layer for app-shell/layout regressions.
> > - Treat the text fixtures already at `crates/engine/testdata/golden/text/native/` as the first visual regression suite, not as a one-off tool.
>
> Honours `feedback_screenshot_goldens` (rendering work needs image-
> based regression), `feedback_no_copyleft_integration` (no GPL-class
> linkage), `feedback_no_synthetic_kicad` (real fixtures only),
> `feedback_research_only_mode` (research-only deliverable; no spec
> edits applied).

---

## Executive Summary

- **Recommendation: build two layers, in this order, with strict
  separation of duties.** Layer A (the primary defence) is an
  offscreen wgpu render-to-texture harness that captures the
  `gui-render` board scene before any window or compositor is
  involved. Layer B (the upper, thinner layer) is a full-window
  screenshot harness that boots `datum-gui` against a software
  Vulkan ICD (lavapipe) inside a pinned container. Layer A protects
  every pixel of the canvas; Layer B protects shell and layout. The
  two are not interchangeable and must not be conflated in the
  fixture taxonomy.

- **Stack lock-in already favours this split.** `crates/gui-render`
  depends on `wgpu 28` and `glyphon 0.10` (which pulls
  `cosmic-text 0.15` + `swash 0.2.7`); `crates/gui-app` adds
  `winit 0.30` for the windowed driver. Critically, `gui-render`
  has no `winit` dependency at all — the renderer is already a
  library with no window. That is the cleanest possible substrate
  for offscreen capture, and it means Layer A can be implemented
  without any architectural change to the renderer.

- **Selected Rust stack (concrete, all permissive licences):**
  - Capture (Layer A): `wgpu` `Texture::copy_to_buffer` →
    `image 0.25` PNG encode (Apache-2.0/MIT)
  - Capture (Layer B): X11/Xvfb + `xwd` or `scrot` invoked via
    `std::process::Command`; or wayland + `grim` under a Weston
    headless backend; both cited as Apache-2.0 / MIT-ish where
    relevant (see § Selected Rust/Tooling Stack)
  - Comparator: `image-compare 0.4` (MIT) for SSIM + per-pixel,
    fronted by a thin `datum-visual-diff` helper module so the
    underlying crate is swappable; `dssim-core 3.x` (AGPL —
    explicitly **rejected** below) is *not* the recommendation
  - Diff visualisation: in-process red-XOR overlay built on
    `image`; ImageMagick `compare` (Apache 2.0) as an optional
    operator-grade tool, invoked as a subprocess only
  - Test orchestration: thin custom harness over `cargo test` with
    a `visual` feature flag; `insta 1.x` (Apache-2.0) as the
    snapshot store for the *manifest* part of fixtures; *not* as
    the binary image differ (insta's image support is too coarse
    for masked perceptual comparison)
  - Determinism levers: pinned Mesa (≥ 24.0, llvmpipe + lavapipe),
    pinned FreeType (already vendored via `cosmic-text`/`swash`
    transitively), vendored fonts under
    `crates/engine/testdata/fonts/` (no system fontdb)

- **Top three risks, in priority order:**
  1. **Cross-architecture float drift in glyph rasterization.**
     Phase 2 of the text-engine research already named this as the
     dominant determinism risk. `swash` (used by `glyphon`)
     internally flattens TrueType/CFF outlines using floating-point
     subdivision. Pixel-exact agreement across x86_64 and aarch64
     is not promised by any layer of that stack. Mitigation:
     pin CI to a single architecture for golden authority
     (x86_64-linux), treat aarch64 as a secondary checked-against-
     tolerance lane, never as authority.
  2. **GPU/driver pixel non-determinism even with software raster.**
     Mesa lavapipe is *much* more reproducible than vendor drivers
     but it is not a bit-stable rasterizer across versions; minor
     point releases have changed pixel coverage at edges. Pin the
     Mesa version inside the CI container and gate Mesa updates as
     a deliberate "rebless" event.
  3. **Golden-rot from rubber-stamped reblesses.** The single most
     common failure mode of visual regression in the wild (Chromium
     `pixeltests`, Firefox `reftest`, Skia gold, KiCad QA) is that
     reviewers wave through new goldens because the diff "looks
     fine." Mitigation built into the workflow below: every
     rebless commit must carry a `VISUAL-REBLESS:` footer block
     listing each fixture and the human-readable reason; CI gates
     bulk reblesses behind a hard fixture-count threshold.

- **Phase 1 first slice (the immediate work):** stand up Layer A
  end to end on a single, narrow lane — board-text rendering — by
  taking the four existing `crates/engine/testdata/golden/text/
  native/text-*-repro/` projects, adding a `*.fixture.toml`
  manifest beside each, wiring an offscreen wgpu render path that
  consumes a board scene and writes a PNG, plumbing
  `image-compare` exact-pixel + SSIM, and gating the whole thing
  behind a `--features visual` opt-in so default `cargo test`
  remains fast. Estimated effort: **5–8 engineering days** for a
  single engineer who already knows the `gui-render` crate, with
  the bulk of the time spent on the offscreen wgpu surface
  scaffolding (≈ 2 days), the manifest format and loader (≈ 1
  day), the diff harness and failure-artifact emitter (≈ 1.5 days),
  and the CI container recipe (≈ 1.5 days).

- **Important non-recommendation:** do not adopt `egui_kittest`,
  `iced_test`, or any other framework-specific snapshot harness.
  Datum's GUI is a custom wgpu + winit application — there is no
  immediate-mode UI framework hiding underneath. The right harness
  is a small, focused, in-tree module Datum owns.

- **Important license filter applied:** every comparator and
  capture crate considered was checked against
  `feedback_no_copyleft_integration`. The widely-cited `dssim`
  crate is AGPL and is **excluded** from the linked stack. SSIM is
  available under MIT via `image-compare`, which is the
  recommended path. ImageMagick (Apache 2.0) is permitted because
  it is invoked only as a subprocess.

---

## Architecture Recommendation

### Two layers, with sharply different responsibilities

The architectural recommendation is **two layers, not one**, and the
two layers must remain separable in code, in CI configuration, in
fixture taxonomy, and in failure-triage workflow. They catch
different categories of regression and they have different
determinism profiles. Conflating them — for example, by trying to
run the offscreen path through the windowed app for "consistency" —
is the failure mode that has historically collapsed visual
regression in other projects into either-too-flaky-or-useless.

#### Layer A — Render-scene goldens (primary defence)

**What it captures:** the output of `crates/gui-render` against a
constructed scene, rendered offscreen into a wgpu texture, copied
into a CPU buffer, encoded as PNG. No `winit`, no window, no
compositor, no display server, no font server.

**What it protects, concretely:**

- Authored copper geometry (track widths, corner geometry, pad
  shapes, via inner/outer ring rendering)
- Layer ordering and visibility (front/back stacking per the M7
  render-stack rule)
- The four lane semantics from
  `docs/gui/M7_RENDER_SEMANTIC_CONTRACT.md` — authored, unrouted,
  proposed, diagnostic — and their relative visual weight
- Text fidelity for the engineering text path (Newstroke /
  stroke-font output) and, once present, the design text path
  (outline-font output through `glyphon`/`swash`)
- Airwire / ratsnest line styling and endpoint anchors
- Zone fill rendering, including hatched and solid pours
- Mask / paste / silkscreen layer-family colouring
- Selection, hover, and review-state diagnostic emphasis without
  lane-identity drift
- The "imported-board semantic invariance rule" from the M7
  contract: same scene with `render_cache` present and absent
  should produce equivalent pixels

**What it does NOT protect:**

- Window chrome
- Panel docking layout
- Toolbar contents and their interactive state
- Inspector panel data binding
- Theme application across the shell
- Cursor rendering
- Keyboard focus rings
- Platform-native menu rendering

**Determinism profile (the core argument for putting it first):**

Layer A is not subject to:
- Window-manager differences (no window manager)
- Compositor differences (no compositor)
- Display-server differences (no display server)
- Vsync timing (no presentation; we copy from a render target
  directly)
- Cursor rendering (no cursor)
- DPI scaling from the OS (we set the surface size programmatically)
- Frame pacing or async readback timing (we wait on the queue
  directly via `Queue::on_submitted_work_done` and
  `Buffer::map_async`)

It *is* still subject to:
- GPU driver / software-raster pixel coverage at AA edges
- Glyph rasterizer floating-point drift (the Phase 2 risk)
- wgpu version changes (so wgpu becomes a treat-as-pinned
  dependency; updates are reblessing events)

The first three "not subject to" items are doing all the work
here. Headless-window screenshot harnesses fail in production
overwhelmingly because of those three, not because of the
renderer itself. By moving Layer A *below* the windowing system,
we delete the entire failure category in one step.

**Implementation locus:** a new `crates/gui-render/tests/visual/`
module behind a `visual` cargo feature, plus a small
`crates/gui-render/src/visual_capture.rs` that exposes an
`OffscreenRenderer` builder accepting `(width, height,
ColorTargetState, MultisampleState, sample_seed)` and yielding a
`fn render_scene(scene: &BoardReviewScene) -> Vec<u8>` (RGBA8
unorm sRGB, row-padded per wgpu's `COPY_BYTES_PER_ROW_ALIGNMENT`
of 256 bytes, with a strip-padding helper).

#### Layer B — Full-GUI goldens (upper, thinner layer)

**What it captures:** the entire `datum-gui` window — chrome,
panels, toolbars, viewport, status — rendered to a real surface
inside a headless display environment (Xvfb on X11, or Weston
with the headless backend on Wayland), screenshot via an external
tool (`xwd` + `convert`, or `grim`), compared against a stored
golden.

**What it protects, concretely:**

- Panel layout regressions when window is resized
- Theme application across the shell (light/dark, palette swap)
- Toolbar button layout and icon presence
- Inspector panel content rendering at known states
- Bottom-docked terminal lane and AI lane layout
- Modal, popover, and tooltip rendering position
- Z-order between panels and viewport
- Top menu bar item presence and ordering

**What it does NOT protect (because Layer A already does):**

- Anything inside the board canvas pixel-exact
- Stroke width or corner geometry of authored copper
- Glyph-level text fidelity inside the canvas

**Determinism profile (the argument for keeping it second):**

Layer B is subject to *all* of Layer A's risks plus:
- Headless display server quirks (Xvfb font fallback, missing
  fontconfig matches, default cursor theme)
- Window decorations from the chosen WM-or-no-WM environment (the
  recommendation is to use Xvfb without any window manager at
  all, so windows render undecorated)
- Async UI ready-time (the app may render before all panels have
  laid out; the harness must actively wait for a "ready" signal)
- Compositor double-buffering and damage tracking

These risks are real but they are also tractable when Layer B's
fixture set is kept *small and architectural* (≈ 8–20 fixtures
total, lifetime), rather than trying to cover the full canvas
space. Layer A handles the canvas; Layer B handles the shell.

**Implementation locus:** a new `crates/gui-app/tests/shell/`
module behind the same `visual` feature flag, plus a small
external test driver script that brings up Xvfb, launches
`datum-gui` with a `--shell-screenshot=<name>` flag (a debug-only
capture mode that opens against a fixture, waits for a quiescence
signal, screenshots the window, and exits 0).

### Why offscreen-primary, not full-GUI-primary

The single decisive argument: when the canvas regresses, the
canvas is what the user cares about. A full-GUI golden tells you
*the screenshot changed* but it does not tell you *the canvas
changed*; you need a per-pixel inspection of a 1920×1080 image to
locate the change, and you cannot tell from the diff alone whether
the change came from the renderer, the panel layout, the theme,
or a font-fallback drift in the inspector. A renderer-level
golden against a fixed scene tells you, with a single pass-fail,
that the renderer changed — and the change can be diffed against
the lane vocabulary directly. Layer B is genuinely useful, but it
is the wrong *primary* line of defence because its diffs
under-specify causation.

Two further reasons reinforce the order:
- Layer A is **cheap to run** (no display server, no window, no
  compositor) — it can run on every PR without budget pressure.
  Layer B requires a container with Xvfb and is slower; it should
  run on every PR but with a smaller fixture count.
- Layer A is **cheap to debug** when it fails: the fixture is a
  scene-construction function, the failure is a PNG triplet
  (actual / expected / diff), and the cause is in the renderer.
  Layer B failures require eyes on the full window and judgement
  about what region matters.

### What this rules out

The recommendation explicitly rules out three alternative
architectures the team should not be tempted into:

1. **Full-GUI-only.** Rejected per the user's architectural floor.
   It is also the highest-flake architecture in the wild
   (Chromium gave it up for canvas-internal regressions years
   ago; KiCad's QA explicitly avoids it for board rendering).

2. **Offscreen-only.** Rejected because shell layout regressions
   are real, particularly during `M7` opening when panel
   structure is still in flux. A theme change that breaks panel
   borders should be caught.

3. **Single-layer "smart" diffing that tries to crop the canvas
   region out of a full screenshot.** Tempting because it sounds
   like one harness. Rejected because it inherits Layer B's
   determinism risks for what is logically a Layer A concern, and
   because the crop coordinates drift whenever the shell changes
   — a shell change quietly invalidates every canvas golden.

---

## Risk Table

| # | Risk | Severity | Where it bites | Mitigation |
|---|------|----------|----------------|------------|
| 1 | Cross-architecture floating-point drift in glyph outline flattening (`swash` Bézier subdivision) | **High** — already named as the top risk in `DATUM_TEXT_ENGINE_PHASE_2_RESEARCH.md` Topic 6 | Any text-bearing golden; especially design-mode (outline-font) text | Pin a single architecture as golden authority (x86_64-linux). Run aarch64 as tolerance-checked secondary. Keep stroke-font (Newstroke) on the integer-arithmetic path where feasible — the strokes ARE the canonical form. Any move to outline-font goldens carries an explicit determinism contract before fixtures are blessed. |
| 2 | wgpu version drift changes pixel coverage at edges | **High** | Every Layer A fixture | Treat `wgpu` like a vendored numerical kernel. Pin to an exact patch version in `Cargo.toml` (currently `wgpu = "28"` workspace-wide; consider `= "28.0.0"` for the duration of golden authority). Mesa-and-wgpu coupled updates become reblessing events, never silent. |
| 3 | Mesa lavapipe / llvmpipe version drift changes AA edges | **High** | Every Layer A and Layer B fixture | Pin Mesa version inside the CI container (Dockerfile uses Debian/Ubuntu repo at a specific snapshot date; the recipe in § CI Strategy uses Debian 13 `bookworm` snapshot for predictable Mesa pinning). Document the rebless-on-upgrade workflow. |
| 4 | Vendor-GPU CI flake when developers run goldens locally on Nvidia/AMD/Intel hardware | **Medium** | Local dev experience; not authority | Default the offscreen test path to `WGPU_BACKEND=vulkan` + force Mesa via `LIBGL_ALWAYS_SOFTWARE=1` + `VK_ICD_FILENAMES=/usr/share/vulkan/icd.d/lvp_icd.x86_64.json` *in the test harness itself*. Tests refuse to bless unless the software ICD is active. |
| 5 | Subpixel font AA on / off mismatch between dev and CI | **Medium** | Any text-bearing golden | Force greyscale AA in `cosmic-text` swash backend; assert this at test setup time; document. cosmic-text exposes `SwashCache::with_render_target` configuration; verify the antialiasing mode is greyscale, not subpixel. |
| 6 | Async render not complete before capture | **Medium** | Layer A and Layer B both | Layer A: explicit `Queue::on_submitted_work_done` + `Buffer::map_async` await before reading. Layer B: explicit application-side "ready" signal published via stdout (e.g. `READY_FOR_SCREENSHOT` line); the harness reads stdout and only screenshots on receipt. |
| 7 | Xvfb font fallback when a needed font is missing | **Medium-High** | Layer B | Layer B's container vendors the same fontconfig + the same fonts as the application. Datum's bundled font set (Newstroke + Inter + IBM Plex Sans Condensed + Inter Display + JetBrains Mono per Phase 2 text research) lives at `crates/engine/testdata/fonts/`. Fontconfig in the container points at that directory exclusively. |
| 8 | Window-manager decoration drift in headless display | **Medium** | Layer B | Run Xvfb without any WM. The application opens an undecorated window. Verified by setting `WM_HINTS` from winit accordingly; if winit refuses, the harness sets `XLIB_SKIP_ARGB_VISUALS=1` and relies on Xvfb's default no-WM behavior. |
| 9 | Cursor rendering in screenshot | **Medium** | Layer B | Either disable cursor via Xvfb arguments (`-nocursor`) or move cursor to a known off-window position before capture; recommend `-nocursor`. |
| 10 | DPI scaling drift | **Low-Medium** | Layer B | Force HiDPI scale = 1.0 explicitly; set `GDK_SCALE=1`, `QT_AUTO_SCREEN_SCALE_FACTOR=0`, and pass winit's monitor scale-factor override if available. |
| 11 | Animations not yet quiesced | **Medium** | Layer B | The application has, per `M7` v1, very few animations; the recommendation is to *forbid* animations in the shell for Layer B fixtures (the visual harness sets a `DATUM_NO_ANIMATIONS=1` env var the app respects and clamps any "should animate" path to instant-final-state). |
| 12 | Clock-dependent rendering (timestamps in status bar, "last updated" labels) | **Medium** | Layer B; possibly Layer A overlays | Frozen-clock fixture flag; the app reads `DATUM_FIXED_CLOCK=2026-01-01T00:00:00Z` and uses that as the wall-clock for any UI-visible time. |
| 13 | Random IDs / random colours / hash-derived colours | **Medium** | Both layers | Forbid randomness in any visible-pixel code path. Where pseudo-random is structurally needed (e.g. layer auto-colour assignment), seed with a fixed seed exposed via the fixture manifest. |
| 14 | Golden-rot via rubber-stamped reblesses | **High** | Workflow | Mandatory `VISUAL-REBLESS:` block in commit message listing fixture-by-fixture human-readable reasons; CI hard-fails the commit if more than N fixtures change without a `VISUAL-REBLESS-BULK:` companion footer that names a bounded reason. |
| 15 | Golden bloat (binary churn in git) | **Medium-Long-term** | Repo size | PNGs are stored under `crates/.../testdata/golden/`. PNGs are *not* re-encoded on rebless unless content changed; bit-for-bit identical PNGs do not produce a git diff. Consider Git LFS only if any single golden exceeds 1 MB (most should be < 50 KB at 1024×768 RGBA). |
| 16 | Diff visualisation hard to read for sub-pixel changes | **Medium** | Triage | Default diff visualisation is red-XOR with brightness amplification (multiply absolute pixel difference by 8 before clamping). Operator tool: ImageMagick `compare -highlight-color red -lowlight-color white -compose src` for a high-contrast view on the side. |
| 17 | Mask images go stale silently when fixture intent changes | **Medium** | Long-term | Masks are first-class fixture artifacts and a mask change is a rebless event; the harness logs "mask was applied to N pixels" so reviewers can see when a mask is hiding real coverage. |
| 18 | Cross-platform fixture authority confusion | **Medium** | Multi-developer workflow | One repository, one authority architecture (x86_64-linux), one container. mac/Windows tests run for early warning but never write goldens. |
| 19 | False-pass when scene is fully blank | **Low-Medium** | Layer A | Harness sanity-asserts at least P% of pixels differ from background (default 1%); fixture failing this gate cannot be blessed without a manifest opt-out (`expect_blank = true`). |
| 20 | False-pass when fixture was never re-rendered (cached old PNG) | **Low** | Both | Build-time hash of fixture inputs (manifest + scene fn + renderer crate version) embedded into PNG `tEXt` chunk; harness compares hashes and fails-fast if no re-render occurred. |

---

## Selected Rust/Tooling Stack

This section names exact crates, exact versions where pinning
matters, and the rationale for each pick. Every dependency listed
here is permissively licensed. The license-filter line is repeated
where the choice is non-obvious.

### Image capture

#### Layer A — offscreen wgpu

The capture path is built on wgpu's documented texture-to-buffer
copy primitive. No new crate is needed.

- **`wgpu`** — already a workspace dep at version `28`. Use
  `wgpu::TextureUsages::COPY_SRC | RENDER_ATTACHMENT` on the
  target texture, render the scene, then
  `encoder.copy_texture_to_buffer` into a buffer with usage
  `MAP_READ | COPY_DST`. After `queue.submit()`, call
  `buffer.slice(..).map_async(MapMode::Read, ...)` and
  poll the device with `device.poll(Maintain::Wait)`.
  License: Apache-2.0 / MIT (dual). Permissive.

- **`bytemuck`** — already a workspace dep. Used to reinterpret
  the mapped buffer as `&[u8]` for PNG encoding. License:
  Zlib / Apache-2.0 / MIT. Permissive.

- **`image`** at `0.25.10` (already in `Cargo.lock` transitively).
  Use `image::ImageBuffer::<Rgba<u8>, _>::from_raw(width, height,
  data)` then `.save("path.png")` with the PNG codec which is
  default-enabled. License: Apache-2.0 / MIT. Permissive.

- **Row-padding helper.** wgpu requires the bytes-per-row of the
  copy destination to be a multiple of `COPY_BYTES_PER_ROW_ALIGNMENT
  = 256`. For a 1024-wide RGBA8 image this is naturally aligned
  (4096 % 256 = 0); for an 800-wide image it is not (3200 % 256
  = 128). The helper strips the padding row-by-row before
  passing to `image::ImageBuffer::from_raw`. This is a known
  10-line idiom; document it in `gui-render/src/visual_capture.rs`
  rather than pulling a new crate.

The whole Layer A capture path is, in total, perhaps 80-150 lines
of Rust that Datum owns.

#### Layer B — full-window screenshot

Three viable paths. Recommend the first.

1. **Xvfb + scrot (recommended).** The CI container runs Xvfb
   on display :99, the test driver launches `datum-gui` with
   `DISPLAY=:99` and a debug screenshot mode flag, the app
   prints `READY_FOR_SCREENSHOT <window-id>` when its render
   loop has quiesced, and the driver shells out to `scrot
   --no-cursor -i <window-id> /tmp/shot.png`. Both tools are
   MIT-class licensed. Xvfb is `xorg-server` (MIT). `scrot` is
   `BSD-2-Clause`-equivalent (the X11/MIT-derived "X11 license"
   variant). No copyleft contamination.

2. **Wayland + grim under Weston headless.** Modern alternative.
   `weston` runs with `--backend=headless-backend.so`. `grim`
   captures. Both are MIT. The pain is launching applications
   that target X11 by default into a Wayland session reliably
   — winit handles both, so this is feasible, but the X11 path
   has more decade-of-CI-experience to draw on and is the
   recommendation for the first iteration.

3. **VNC server + xdpyinfo / x11vnc + xwd.** Older, more
   moving parts, no advantage. Not recommended.

The recommendation is path 1 for the first implementation.

### Image comparison

The choice is more constrained than it first appears because of
the license filter.

#### Considered and recommended

- **`image-compare`** at `0.4.x` (currently `0.4.6` on crates.io).
  License: **MIT**. Provides `ssimulacra2`-inspired SSIM and
  per-pixel comparators. Maintained, simple API, no system deps,
  pure Rust. **Recommended primary comparator.**

- **`imageproc`** at `0.25.x`. License: MIT. Provides primitives
  (per-pixel diff, RMS, geometric transforms). Useful as a
  secondary toolbox for the diff-visualisation step. Permissive.

- **Custom exact-pixel comparator** for Layer A's strict path —
  ten lines of `impl PartialEq for Rgba<u8>`-style iteration
  with a tolerance band parameter. Datum owns it; no crate
  needed. The recommendation is to use this path (with tolerance
  = 0) for Layer A by default and reach for `image-compare`'s
  SSIM only when a particular fixture justifies it.

#### Considered and rejected

- **`dssim` / `dssim-core`.** License: **AGPL-3.0**. This is the
  most-cited SSIM implementation in the Rust ecosystem and has
  been used by Cloudinary internally for image quality. It is
  **excluded** from the Datum stack per
  `feedback_no_copyleft_integration` because linking AGPL into
  Datum's test binaries would constitute combined-work AGPL
  for the test crates, which compromises future test-tool
  reuse. `dssim` could be invoked subprocess-only in principle,
  but `image-compare` already provides equivalent SSIM under
  MIT and is the cleaner choice.

- **`butteraugli` Rust port.** License: variable across forks;
  the canonical C++ butteraugli is Apache-2.0 but the most
  active Rust binding (`butteraugli`) is unmaintained. Not
  recommended.

- **`pixelmatch-rs`.** License: MIT. Port of the JavaScript
  pixelmatch library. Functional, but the diff output is
  stylistically tied to web tooling and `image-compare` covers
  the same territory more cleanly.

- **`insta`** image support. License: Apache-2.0. `insta` has a
  `_binary_snapshot!` macro family which can store binary
  snapshots, but its diff UI is not designed for image
  comparison and the operator workflow for accepting changes
  (`cargo insta review`) is text-oriented. **Recommend `insta`
  for the manifest+metadata snapshots only**, not for the PNG
  binaries themselves.

#### Considered as subprocess-only operator tools

- **ImageMagick `compare`.** License: **Apache-2.0** (since
  ImageMagick 7 the licence is the ImageMagick licence which is
  Apache-2.0 compatible). Battle-tested. Useful for ad-hoc
  developer-machine inspection of failed goldens. Datum invokes
  it as a subprocess only — no library linkage — so the
  permissive licence is preserved end-to-end. Recommend
  documenting `compare -metric AE -fuzz 0.5%` and `compare
  -highlight-color red` invocations in the developer-facing
  doc but **not** building them into the harness's required
  path.

### Diff visualisation

For the in-process pass that emits the diff PNG on failure:

1. **Red-XOR overlay.** For each pixel where actual ≠ expected,
   set the diff image to `Rgba([255, 0, 0, 255])`. For unchanged
   pixels, set to a desaturated copy of the expected image
   (`y = 0.299*r + 0.587*g + 0.114*b`, then
   `Rgba([y/2, y/2, y/2, 255])`) so the diff is read against
   the original geometry context. ~30 lines of Rust on top of
   `image`.

2. **Brightness-amplified red overlay.** For sub-pixel
   differences, multiply absolute pixel difference by 8 and
   clamp; gives a red intensity proportional to severity.
   Useful for AA-edge inspection.

3. **ImageMagick `compare -compose src -highlight-color red`**
   as the operator-grade alternative. Subprocess-only.

Recommend bundling (1) and (2) as the harness-default outputs;
document (3) for human inspection.

### Test orchestration

- **`cargo test` + a `visual` feature flag.** No new framework.
  Tests live under `crates/<crate>/tests/visual/` and only
  compile when `--features visual` is passed. This means
  `cargo test` (no features) remains fast; the visual suite is
  opt-in for developers and gated by a CI matrix entry.

- **`insta`** at `1.x` (Apache-2.0). Used for the *manifest
  snapshot* portion of fixtures — the JSON serialisation of
  the scene under test. This catches scene-construction
  regressions independent of pixel rendering, and provides
  `cargo insta review` for the metadata diff. The PNG goldens
  are *not* stored in `.snap` files; they sit beside the
  manifest as `<fixture>.golden.png`.

- **`goldenfile`** at `1.x` (MIT). Considered as an alternative
  to a custom golden-file framework. Decision: do not adopt.
  `goldenfile` has a clean API for text goldens but not for
  binary; rolling Datum's own ~40-line orchestrator over
  `image-compare` is cleaner than fighting `goldenfile`'s text
  assumptions.

- **`pollster`** — already a workspace dep. Used to await wgpu
  buffer-map futures inside synchronous `#[test]` functions
  without pulling tokio.

### Determinism levers (not a "crate" but a stack item)

These are settings the harness must apply at process start, not
choices the user makes:

- `WGPU_BACKEND=vulkan` (force vulkan; do not allow OpenGL
  fallback because OpenGL backends differ across platforms more
  than vulkan does)
- `LIBGL_ALWAYS_SOFTWARE=1` (force software GL even if a hardware
  GL is present, in case wgpu falls back)
- `VK_ICD_FILENAMES=/usr/share/vulkan/icd.d/lvp_icd.x86_64.json`
  (force lavapipe as the only vulkan ICD visible to wgpu)
- `MESA_VK_VERSION_OVERRIDE=1.3` (lock the lavapipe API surface)
- `FONTCONFIG_PATH=$DATUM_REPO/crates/engine/testdata/fonts/
  fontconfig` (vendored fontconfig pointing only at vendored
  fonts)
- `LANG=C.UTF-8`, `LC_ALL=C.UTF-8` (predictable text shaping
  fallback paths)
- `DATUM_NO_ANIMATIONS=1` (the app skips animation transitions)
- `DATUM_FIXED_CLOCK=2026-01-01T00:00:00Z` (frozen clock for
  UI-visible time strings)
- `DATUM_FORCE_GREYSCALE_AA=1` (forces cosmic-text swash to
  greyscale antialiasing rather than subpixel; verify at fixture
  setup that this took effect by inspecting the `swash`
  rasterizer config)

The harness sets all of these unconditionally; it does not
inherit them from the developer's shell.

### License posture summary

| Crate / tool | Version | Licence | Allowed | Notes |
|--------------|---------|---------|---------|-------|
| `wgpu` | `28.0.0` | Apache-2.0 / MIT | yes | Already in tree |
| `image` | `0.25.x` | Apache-2.0 / MIT | yes | Already in tree |
| `bytemuck` | workspace | Zlib / MIT / Apache-2.0 | yes | Already in tree |
| `pollster` | workspace | Apache-2.0 / MIT | yes | Already in tree |
| `image-compare` | `0.4.x` | MIT | yes | New dep; small, focused |
| `imageproc` | `0.25.x` | MIT | yes | Optional new dep |
| `insta` | `1.x` | Apache-2.0 | yes | New dep; metadata snapshots only |
| `dssim-core` | — | **AGPL-3.0** | **no** | Excluded |
| `butteraugli` | — | inactive | no | Inactive Rust binding |
| `goldenfile` | `1.x` | MIT | not adopted | Does not match binary use case |
| ImageMagick `compare` | system | Apache-2.0 | yes (subprocess only) | Operator tool |
| Xvfb | system | MIT (X11) | yes (subprocess only) | CI container only |
| scrot | system | X11/MIT-equivalent | yes (subprocess only) | CI container only |
| Mesa lavapipe | system | MIT | yes (subprocess / ICD only) | CI container only |
| FreeType | system / vendored | FreeType / GPLv2 dual | yes under FreeType licence | Used via `cosmic-text` / `swash`; FreeType's own dual-licence allows the FreeType-licence path which is permissive |

The FreeType row is worth one extra sentence: FreeType is dual
licensed under the FreeType Project License and GPLv2. Linking
projects pick the FreeType licence to avoid GPLv2; this is the
universal practice and is honoured by `cosmic-text`/`swash` already
in tree. Datum's no-copyleft posture is preserved.

---

## Fixture Manifest Approach

### The principle: nothing implicit

A fixture is not a file. A fixture is a *complete, externally-
captured description of a render scene state*, such that any
human or machine, given only the manifest, the renderer at the
declared version, the vendored fonts, and the input project, can
produce the same PNG bit-for-bit (Layer A) or perceptually-
identical-to-tolerance (Layer B). Anything implicit — a default
viewport, an inherited theme, a system font — is a determinism
hole and the manifest format must close it.

### Manifest format

TOML, one file per fixture, named `<fixture-name>.fixture.toml`,
beside the golden PNG. TOML is chosen over JSON because it
allows comments, which fixtures use heavily (one-line "why this
fixture exists"); over YAML because YAML's whitespace + type
inference rules introduce determinism risk; over RON because
the rest of the Datum tree uses TOML for non-design metadata.

Skeleton:

```toml
# crates/gui-render/testdata/golden/board/text-density-repro.fixture.toml
#
# Fixture purpose:
#   Verifies that high-density board text remains crisp at
#   1:1 viewport with the engineering-text (Newstroke) backend.
#
# Lane: A (offscreen renderer)
# Suite: board-text
# Created: 2026-04-16
# Last reblessed: 2026-04-16
# Last rebless reason: initial creation

[fixture]
name = "text-density-repro"
lane = "A"
suite = "board-text"
fixture_format_version = 1

[input]
# Real project file. Per feedback_no_synthetic_kicad and
# feedback_ask_for_fixtures: this is a real project, not a
# fabricated scene. Path is repo-relative.
project_path = "crates/engine/testdata/golden/text/native/text-density-repro/"
project_kind = "datum-native"

[scene]
# Which lanes from the M7 semantic contract to render.
# Empty list means all lanes per scene.
lanes = ["authored", "unrouted", "proposed", "diagnostic"]
# Layer visibility filter; empty list means all layers visible.
layers_visible = []
# Selection set; empty list means no selection.
selected_object_ids = []
# Review state; "none" | "diagnostic-focus" | "review-dim".
review_state = "none"

[viewport]
width_px = 1024
height_px = 768
# Camera in board millimetres. center_mm + zoom_mm_per_px is
# explicit so the manifest fully determines the projection.
center_mm = [50.0, 50.0]
zoom_mm_per_px = 0.05  # 0.05 mm/px = 1024 px = 51.2 mm wide

[theme]
# Theme is a named profile that resolves to a stable colour set.
# Datum vendors theme profiles; "default-dark" and
# "default-light" are the only two for M7 v1.
profile = "default-dark"

[fonts]
# Font set the renderer is allowed to use. Names resolve via
# the vendored fontconfig path.
text_engineering = "Newstroke Regular"
text_design = "Inter Regular"
mono = "JetBrains Mono Regular"

[render]
# wgpu config. Pinned per fixture so an upgrade is a deliberate
# rebless event.
backend = "vulkan"
backend_min_version = "1.3"
multisample_count = 4
color_format = "rgba8-unorm-srgb"
force_software_raster = true

[determinism]
greyscale_aa = true
no_animations = true
fixed_clock = "2026-01-01T00:00:00Z"
random_seed = 0

[golden]
filename = "text-density-repro.golden.png"
diff_policy = "exact"        # "exact" | "tolerance" | "ssim" | "masked"
diff_tolerance_per_pixel = 0 # used only by "tolerance"
diff_tolerance_total_px_pct = 0.0  # used only by "tolerance"
ssim_threshold = 0.0         # used only by "ssim"; 1.0 = identical
mask_filename = ""            # used only by "masked"

[blank_check]
# Sanity gate: at least this fraction of pixels must differ from
# the corner-pixel-defined background colour, or the fixture is
# treated as accidentally-blank and fails to bless.
expect_non_blank_pct = 1.0

[provenance]
# Hashes embedded into the golden PNG's tEXt chunk so the
# harness can detect "we never re-rendered, we just compared
# against a stale cache".
manifest_sha256 = "<filled by harness on bless>"
renderer_crate_version = "<filled by harness on bless>"
wgpu_crate_version = "<filled by harness on bless>"
mesa_version_string = "<filled by harness on bless>"
font_set_sha256 = "<filled by harness on bless>"
```

Layer B reuses the same `[fixture]`, `[input]`,
`[determinism]`, `[golden]`, `[provenance]` sections; replaces
`[scene]` / `[viewport]` / `[render]` with `[shell]`
(`window_size_px`, `panels_visible`, `panel_layout`,
`theme_profile`, `active_view`, `opened_documents`); replaces
`[fonts]` with shell font defaults; adds `[ready_signal]`
(`stdout_line = "READY_FOR_SCREENSHOT"`, `timeout_seconds = 30`)
because shell screenshots wait on async readiness; defaults
`diff_policy = "ssim"` with `ssim_threshold = 0.985` and
typically references a `mask_filename`.

### Naming convention

```
<fixture-name>.fixture.toml      — manifest, version-controlled
<fixture-name>.golden.png        — golden image, version-controlled
<fixture-name>.mask.png          — optional mask, version-controlled
<fixture-name>.actual.png        — produced on test failure;
                                   gitignored at the directory level
<fixture-name>.diff.png          — produced on test failure;
                                   gitignored
```

The `.actual.png` and `.diff.png` artifacts are written next to
the golden so a developer running `cargo test --features visual`
sees them in the same directory and can `eog` / `ristretto` /
`xdg-open` them immediately, rather than navigating to a separate
artifacts directory. The directory-level `.gitignore` excludes
`*.actual.png` and `*.diff.png` from version control.

### Directory layout

The user has flagged the existing text fixtures as the first
golden suite. Layer A goldens live under
`crates/gui-render/testdata/golden/board/` (one
`.fixture.toml` + one `.golden.png` per fixture, optional
`.mask.png`). Layer B goldens live under
`crates/gui-app/testdata/golden/shell/`. Vendored fonts and a
fontconfig override directory live under
`crates/engine/testdata/fonts/`.

The text fixture *projects* stay where they are (under
`crates/engine/testdata/golden/text/native/<fixture>/`). The
text fixture *manifests and goldens* live in the
`gui-render` crate because that is where the renderer's tests
live; the manifest's `[input].project_path` field links the
two. This separation is deliberate: the project files are
engine fixtures (and have other uses for engine tests); the
rendered goldens are renderer fixtures. Co-locating them would
conflate crate ownership.

### Cross-reference to existing text fixtures

The four existing text fixture projects already on disk are:

- `crates/engine/testdata/golden/text/native/text-density-repro/`
- `crates/engine/testdata/golden/text/native/text-fidelity-repro/`
- `crates/engine/testdata/golden/text/native/text-intent-repro/`
- `crates/engine/testdata/golden/text/native/text-transform-repro/`

Each contains the standard Datum-native quartet:
`project.json`, `board/board.json`, `schematic/schematic.json`,
`rules/rules.json`. They are real Datum-native projects (not
fabricated KiCad s-expressions), they exist for text engine
work, and they are the natural seed corpus for Layer A's first
suite.

The recommendation is to:
1. Adopt these four projects as Layer A's seed corpus exactly
   as they are (no edits to the project files).
2. Add four `.fixture.toml` manifests under
   `crates/gui-render/testdata/golden/board/` that point at
   them.
3. Bless four `.golden.png` images by running the harness once
   on the authoritative container and committing the output.

These four fixtures alone catch text-density, layer-text-AA,
intent-driven font selection, and transform-applied text — the
intersection of "text rendering" (high Phase 2 risk) and "lane
semantics" (M7 contract surface). They are the highest-leverage
opening fixture set Datum could pick.

---

## Image-Diff Policy

### Policy by layer

| Layer | Default policy | Tolerance defaults | When to escalate |
|-------|----------------|--------------------|------------------|
| A (offscreen) | **`exact`** — bit-for-bit pixel match | `diff_tolerance_per_pixel = 0`, `diff_tolerance_total_px_pct = 0.0` | Escalate to `tolerance` (1 LSB per channel, 0.01% of total pixels) only for fixtures empirically demonstrated to drift on the authoritative container. Document the reason in the manifest. Escalate to `ssim` (≥ 0.999) only for fixtures that exercise outline-font glyph rasterization while Phase 2 of the text engine is still landing. |
| B (full GUI) | **`ssim`** — SSIM ≥ 0.985 | Mask required for any region with a known dynamic element (timestamps, scroll indicators, loading spinners) | Escalate to `masked` + `exact` for fixtures that test a single specific shell element (e.g. a tooltip's appearance) by masking everything else. |

### Why `exact` for Layer A by default

Layer A controls every variable that produces pixel drift: the
GPU is software, the driver is pinned, the wgpu version is
pinned, the font is vendored, the AA mode is forced greyscale,
the scene is constructed deterministically. The only remaining
drift source is wgpu's internal implementation (which is part of
the determinism contract, not a noise source — a wgpu drift is
either a wgpu bug worth catching or a deliberate upgrade
worth reblessing for). Exact-pixel match is therefore the right
default; anything looser is a noise budget the team has not
earned yet.

### Why `ssim ≥ 0.985` for Layer B by default

Layer B inherits all of Layer A's determinism risks plus the
shell-side risks. SSIM in the 0.985–0.995 band tolerates 1-pixel
shifts in panel borders, mild AA differences at icon edges, and
font hinting drift in the inspector panel — none of which are
bugs Layer B is meant to catch. Layer B is meant to catch
*architectural* shell regressions: panel disappeared, layout
collapsed, theme palette swapped, modal renders behind viewport.
SSIM at that threshold catches all of those without firing on
"the inspector text shifted half a pixel."

### Mask convention

A mask PNG is the same dimensions as the golden. Pixel value:
- White (`#FFFFFF`) → compare this pixel
- Black (`#000000`) → ignore this pixel
- Any other value → harness rejects the mask at fixture-load time

The black/white-only rule prevents accidental "soft mask" usage
that hides too much. If a soft mask is genuinely needed (e.g.
for an animated region that drifts in extent), the fixture
should be redesigned around `DATUM_NO_ANIMATIONS=1` rather than
masked.

The harness logs `mask: applied to N pixels (P%)` on every run
so reviewers can see when a mask is hiding a meaningful fraction
of the image.

### Artifact-output requirements (non-negotiable)

Every failed visual test produces, beside the golden:

1. `<fixture>.actual.png` — the image the renderer / app
   produced this run
2. `<fixture>.diff.png` — the diff visualisation: red where
   pixels differ, desaturated original where they match
3. `<fixture>.report.txt` — a one-screen text summary:
   - Diff policy in effect
   - Number and percentage of differing pixels
   - SSIM score (if SSIM was computed)
   - Per-channel max delta
   - Mask coverage (if mask present)
   - Provenance hash mismatch (if any)
   - The `git rev-parse HEAD` and `git status --porcelain`
     output at run time

The `.report.txt` is what gets pasted into a commit message when
a developer reblesses; it is the human-readable record of "what
changed and by how much."

In CI, all three artifacts are uploaded under a `visual-failures/`
directory in the workflow's artifact store with retention long
enough that triage can happen out-of-band (recommend 30 days).

---

## CI Strategy

### CI architecture, in one paragraph

Every PR runs both visual layers under a single, pinned
container that vendors Mesa, FreeType, fontconfig, Xvfb,
scrot, ImageMagick, and the Datum-vendored fonts. Layer A
runs first (faster, cheaper, more discriminating); if it
passes, Layer B runs second. Both layers gate the merge. On
any failure, the harness uploads `actual` + `diff` + `report`
artifacts. Reblessing is a deliberate act on a developer's
machine using `cargo run -p datum-visual-bless --
--fixture <name>` against the same container; the rebless
commit must carry a `VISUAL-REBLESS:` footer block listing
each fixture and the reason. CI hard-fails any commit that
changes more than N goldens without a corresponding
`VISUAL-REBLESS-BULK:` footer naming the bounded reason
(e.g. "wgpu upgrade 28.0.0 → 28.1.0"; "Mesa upgrade 24.0.5 →
24.1.0"; "deliberate theme palette change").

### Container recipe

A Dockerfile under `crates/gui-render/testdata/visual-ci/Dockerfile`:

```dockerfile
# crates/gui-render/testdata/visual-ci/Dockerfile
#
# Datum visual-regression CI base image.
#
# Pinning notes:
#   - Debian bookworm at the snapshot date below pins Mesa to a
#     specific version. We update this snapshot deliberately as
#     a "rebless wave" event, never silently.
#   - We do NOT pull from `latest` for any apt package.
#
# To rebuild after a bless wave:
#   docker build -t datum-visual-ci:<date> .

FROM debian:bookworm-20250407-slim

# Pin apt to a specific snapshot for reproducible package versions.
ARG SNAPSHOT_DATE=20250407T000000Z
RUN echo "deb http://snapshot.debian.org/archive/debian/${SNAPSHOT_DATE}/ bookworm main" \
        > /etc/apt/sources.list && \
    echo "Acquire::Check-Valid-Until \"false\";" \
        > /etc/apt/apt.conf.d/no-check-valid-until

RUN apt-get update && apt-get install --no-install-recommends -y \
        # Mesa: software vulkan (lavapipe) and software GL (llvmpipe)
        mesa-vulkan-drivers \
        libgl1-mesa-dri \
        libegl1-mesa \
        # Fontconfig: we will OVERRIDE the system config via env var,
        # but the library must be present.
        libfontconfig1 \
        # FreeType: vendored via cosmic-text but we need the system
        # library for the runtime linker.
        libfreetype6 \
        # X11 headless display
        xvfb \
        x11-utils \
        # Screenshot tool
        scrot \
        # Operator-grade image diff tool
        imagemagick \
        # Build essentials for the Rust crates that need C glue
        build-essential \
        pkg-config \
        # Vulkan loader
        libvulkan1 \
        vulkan-tools \
        # Misc
        ca-certificates \
        curl \
        git \
    && rm -rf /var/lib/apt/lists/*

# Install Rust at the workspace's pinned toolchain.
ARG RUST_TOOLCHAIN=1.83.0
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
        | sh -s -- -y --default-toolchain ${RUST_TOOLCHAIN} \
                       --profile minimal \
                       --component rustfmt clippy
ENV PATH=/root/.cargo/bin:$PATH

# Force software vulkan and explicit ICD selection.
ENV LIBGL_ALWAYS_SOFTWARE=1 \
    WGPU_BACKEND=vulkan \
    VK_ICD_FILENAMES=/usr/share/vulkan/icd.d/lvp_icd.x86_64.json \
    MESA_VK_VERSION_OVERRIDE=1.3 \
    LANG=C.UTF-8 \
    LC_ALL=C.UTF-8

# Verify the lavapipe ICD is present and exposed.
RUN test -f /usr/share/vulkan/icd.d/lvp_icd.x86_64.json \
        || (echo "lavapipe ICD missing; aborting" && exit 1) && \
    vulkaninfo --summary | grep -q "llvmpipe" \
        || (echo "lavapipe not visible to vulkan loader; aborting" && exit 1)

# Capture the Mesa version into the image so visual-test runs can
# embed it as provenance into the golden PNG tEXt chunk.
RUN apt-cache policy mesa-vulkan-drivers \
        | grep "Installed:" > /etc/datum-mesa-version

WORKDIR /workspace
```

This Dockerfile is opinionated about:
- snapshot-pinned apt for reproducibility
- Mesa lavapipe forced as the only vulkan ICD
- FreeType comes from system but cosmic-text uses vendored
  font files exclusively (via fontconfig override at test time)
- Rust toolchain pinned to the workspace value

### The Layer A CI step

```yaml
# Conceptual; actual workflow file lives under .github/ when the
# user wires this up. Research-only mode does NOT add this file.

name: Visual regression — Layer A (offscreen)

on:
  pull_request:
    branches: [ main ]
  workflow_dispatch:

jobs:
  layer-a:
    runs-on: ubuntu-22.04
    container:
      image: datum-visual-ci:2025-04-07
    steps:
      - uses: actions/checkout@v4
      - name: Run offscreen visual tests
        env:
          # Belt and braces; the container ENV already sets these,
          # but we set them again at the step level for any future
          # reader inspecting the workflow alone.
          WGPU_BACKEND: vulkan
          LIBGL_ALWAYS_SOFTWARE: "1"
          VK_ICD_FILENAMES: /usr/share/vulkan/icd.d/lvp_icd.x86_64.json
          FONTCONFIG_PATH: /workspace/crates/engine/testdata/fonts/fontconfig
          DATUM_NO_ANIMATIONS: "1"
          DATUM_FIXED_CLOCK: "2026-01-01T00:00:00Z"
          DATUM_FORCE_GREYSCALE_AA: "1"
        run: |
          cargo test --features visual \
                     -p datum-gui-render \
                     -- --test-threads 1 visual::
      - name: Upload failure artifacts
        if: failure()
        uses: actions/upload-artifact@v4
        with:
          name: layer-a-failures
          path: |
            crates/gui-render/testdata/golden/**/*.actual.png
            crates/gui-render/testdata/golden/**/*.diff.png
            crates/gui-render/testdata/golden/**/*.report.txt
          retention-days: 30
```

### The Layer B CI step

```yaml
  layer-b:
    runs-on: ubuntu-22.04
    needs: layer-a
    container:
      image: datum-visual-ci:2025-04-07
      options: --shm-size=2g
    steps:
      - uses: actions/checkout@v4
      - name: Build datum-gui
        run: cargo build --release -p datum-gui-app
      - name: Start Xvfb
        run: |
          Xvfb :99 -screen 0 1920x1080x24 -nocursor &
          sleep 1  # Xvfb startup; one-time, well-bounded
          export DISPLAY=:99
          xdpyinfo > /dev/null || (echo "Xvfb failed" && exit 1)
      - name: Run shell visual tests
        env:
          DISPLAY: ":99"
          # Same determinism env as Layer A.
          WGPU_BACKEND: vulkan
          LIBGL_ALWAYS_SOFTWARE: "1"
          VK_ICD_FILENAMES: /usr/share/vulkan/icd.d/lvp_icd.x86_64.json
          FONTCONFIG_PATH: /workspace/crates/engine/testdata/fonts/fontconfig
          DATUM_NO_ANIMATIONS: "1"
          DATUM_FIXED_CLOCK: "2026-01-01T00:00:00Z"
          DATUM_FORCE_GREYSCALE_AA: "1"
          GDK_SCALE: "1"
          QT_AUTO_SCREEN_SCALE_FACTOR: "0"
        run: |
          cargo test --features visual \
                     -p datum-gui-app \
                     -- --test-threads 1 visual::shell::
      - name: Upload failure artifacts
        if: failure()
        uses: actions/upload-artifact@v4
        with:
          name: layer-b-failures
          path: |
            crates/gui-app/testdata/golden/**/*.actual.png
            crates/gui-app/testdata/golden/**/*.diff.png
            crates/gui-app/testdata/golden/**/*.report.txt
          retention-days: 30
```

A note on the `sleep 1` for Xvfb startup: this is the one place
the harness uses an unconditional sleep; it is bounded
(1 second), it is one-time per CI run, it cannot be replaced by a
`xdpyinfo` poll because Xvfb may not yet be advertising the
display when the first poll runs, and the actual `xdpyinfo` check
on the next line is the real readiness gate. Readers should not
generalize the sleep.

### Performance budget

- Layer A: 4 fixtures × ~ 200 ms render + ~ 100 ms compare =
  < 2 seconds wall-clock for the seed corpus. Plan for growth
  to 50–100 fixtures over the next year; budget < 30 seconds
  total. This fits within ordinary CI time.
- Layer B: 8–20 fixtures × (~ 5 second app boot + ~ 1 second
  screenshot + ~ 200 ms compare) = roughly 50 seconds to
  2 minutes per layer-B run. Acceptable but explicitly the
  reason Layer B is kept small.

### Cross-platform CI

Linux is the authority. The recommendation is:

- **x86_64-linux**: full visual suite, both layers, **golden
  authority** (this is the only architecture allowed to bless
  goldens)
- **aarch64-linux**: full Layer A only, run with
  `expect_drift = true` flag in the manifest header that
  switches the comparator to SSIM ≥ 0.998 (early warning
  without rebless authority)
- **macOS / Windows**: not in scope for `M7`. Visual regression
  on those platforms is a future-only concern; the architecture
  above does not preclude it but does not ship it.

---

## Golden Update Workflow

### Local-developer workflow

A developer running `cargo test --features visual` sees a failure
and wants to investigate:

```
$ cargo test --features visual -p datum-gui-render -- visual::

running 4 tests
test visual::board_text::text_density_repro ... FAILED
test visual::board_text::text_fidelity_repro ... ok
test visual::board_text::text_intent_repro ... ok
test visual::board_text::text_transform_repro ... ok

failures:

---- visual::board_text::text_density_repro stdout ----
golden mismatch:
  fixture:        text-density-repro
  policy:         exact
  differing px:   142 of 786432  (0.0181%)
  max channel Δ:  3
  artifacts:
    expected: crates/gui-render/testdata/golden/board/text-density-repro.golden.png
    actual:   crates/gui-render/testdata/golden/board/text-density-repro.actual.png
    diff:     crates/gui-render/testdata/golden/board/text-density-repro.diff.png
    report:   crates/gui-render/testdata/golden/board/text-density-repro.report.txt
```

The developer opens the three PNGs side-by-side, decides whether
the change is intentional, and either:

- (a) Fixes the bug (no rebless), or
- (b) Reblesses if the change is intentional:

```
$ cargo run -p datum-visual-bless -- \
        --fixture text-density-repro \
        --reason "wider Newstroke stroke width to match Phase 1 spec change"

Reblessing text-density-repro:
  Re-running fixture inside the visual-CI container...
  [docker run ... --workdir /workspace -v $PWD:/workspace ...]
  Re-rendered. Computing provenance hashes...
  Embedding provenance into golden PNG tEXt chunk...
  Golden updated:
    crates/gui-render/testdata/golden/board/text-density-repro.golden.png
  Manifest updated:
    crates/gui-render/testdata/golden/board/text-density-repro.fixture.toml
        manifest_sha256:        a3f...
        renderer_crate_version: 0.7.2
        wgpu_crate_version:     28.0.0
        mesa_version_string:    23.2.1-1
        font_set_sha256:        9e1...

Stage these changes:
    git add crates/gui-render/testdata/golden/board/text-density-repro.golden.png
    git add crates/gui-render/testdata/golden/board/text-density-repro.fixture.toml

Add this footer to your commit message:

    VISUAL-REBLESS:
        text-density-repro: wider Newstroke stroke width to match
                            Phase 1 spec change
```

The bless tool runs the fixture *inside the same CI container*
the user has built locally (`docker run` against the
`datum-visual-ci:<date>` image), so the resulting golden is the
authoritative one — not the developer's local-machine output,
which would carry their machine's GPU drivers.

### Why a custom bless tool, not "just delete and re-run"

The naive "delete the golden, re-run, commit the new one" workflow
has three failure modes that bite in practice:

1. The new golden carries the developer's local GPU/driver
   state, not the CI container's. CI then fails on the rebless
   commit and the developer ping-pongs.
2. No human-readable reason is recorded; six months later
   nobody remembers why the golden changed.
3. Bulk reblesses go un-explained; goldens drift collectively
   without any single person justifying the drift.

The bless tool fixes all three: container-driven re-render,
mandatory `--reason`, structured commit-message footer.

### CI-driven bless workflow (no PRs on this project)

Per `feedback_no_pull_requests` (project rule referenced in the
brief), Datum does not use pull requests. The CI rebless story
adapts:

1. A developer runs the rebless locally (the workflow above).
2. They commit with the `VISUAL-REBLESS:` footer.
3. They push directly to `main` once the `VISUAL-REBLESS:` block
   has been reviewed by the user out-of-band (Datum's standard
   review pattern per project conventions).
4. CI runs the new goldens against itself; they should pass on
   the first run because the bless tool used the container.

A `workflow_dispatch` action exists for the bulk-rebless case
(e.g. wgpu version bump): the action runs the bless tool against
every fixture, commits the result with a `VISUAL-REBLESS-BULK:`
footer naming the systemic reason, and the user merges the
commit manually after reviewing the diff PNGs in the workflow
artifacts.

### The "no rubber-stamp" gate

Two enforcement mechanisms:

1. **Hard fixture-count threshold.** Any commit that touches
   more than 10 `.golden.png` files in a single commit must
   carry a `VISUAL-REBLESS-BULK:` footer. CI inspects the
   commit message and fails the commit otherwise. This
   prevents the "I reblessed everything because they were all
   failing and I'm in a hurry" failure mode.

2. **Diff PNG attached to commit.** The `VISUAL-REBLESS:`
   footer schema requires a `diff_artifact_url` field for any
   fixture being reblessed (CI artifact link to the
   `<fixture>.diff.png` from the failing run). This forces the
   reviewer to look at the diff before the rebless commit can
   be created.

Both mechanisms are enforced by a small `scripts/check_visual_rebless.py`
that runs as part of the existing alignment / drift-gate CI
pass. (Implementation owned by the user; this is research.)

---

## Phase 1 Implementation Plan

The objective of Phase 1 is to land Layer A end-to-end on a
single narrow lane (board text) with the four existing
fixtures, with no new shell-screenshot work. Layer B does not
land in Phase 1.

### Steps

#### Step 1.1 — Add the `visual` cargo feature flag (≈ 0.25 day)

In `crates/gui-render/Cargo.toml`:

```toml
[features]
visual = ["dep:image-compare"]

[dev-dependencies]
image = { workspace = true }
image-compare = { version = "0.4", optional = true }
```

In `crates/gui-app/Cargo.toml`, add the same feature flag for
future Layer B work:

```toml
[features]
visual = []
```

The flag gates compilation of the `tests/visual/` directory
(via `#![cfg(feature = "visual")]` at the test module top).

Default `cargo test` does not pull `image-compare` and does not
compile the visual test directory, preserving the existing
fast test path.

#### Step 1.2 — Implement `OffscreenRenderer` in `gui-render` (≈ 2 days)

Add `crates/gui-render/src/visual_capture.rs` exposing
`OffscreenRenderer::new(width, height)`, `render_scene(scene,
camera)`, and `read_pixels() -> image::RgbaImage`. Pattern mirrors
the upstream wgpu `examples/capture` example.

Critical details:
- Texture descriptor: `format: Rgba8UnormSrgb`,
  `usage: RENDER_ATTACHMENT | COPY_SRC`,
  `sample_count: 4` (matching the application's MSAA setting)
- Resolve to a non-MSAA texture before copying (separate
  `RENDER_ATTACHMENT | COPY_SRC` texture for the resolve target)
- Output buffer: size = padded_bytes_per_row × height,
  `usage: MAP_READ | COPY_DST`,
  `padded_bytes_per_row = ((width * 4 + 255) / 256) * 256`
- After copy: `device.poll(Maintain::Wait)`,
  `buffer.slice(..).map_async(MapMode::Read, ...)`,
  `pollster::block_on(receiver.recv())`
- Strip row padding on read, build
  `image::RgbaImage::from_raw`

#### Step 1.3 — Implement the manifest loader (≈ 0.5 day)

Add `crates/gui-render/src/visual_manifest.rs` with a
`FixtureManifest` struct mirroring the TOML schema (sections:
`fixture` / `input` / `scene` / `viewport` / `theme` / `fonts` /
`render` / `determinism` / `golden` / `blank_check` /
`provenance`), deserialized via the workspace `toml` crate, plus
a `validate()` method that rejects unknown values, missing
provenance hashes on bless-time, and inconsistent diff-policy
parameter combinations.

#### Step 1.4 — Implement the comparator harness (≈ 1 day)

Add `crates/gui-render/src/visual_diff.rs` exposing
`DiffPolicy { Exact, Tolerance { per_pixel, total_pct }, Ssim
{ threshold }, Masked { mask_path, inner } }`, a `compare(
expected, actual, policy)` returning `DiffResult { passed,
differing_pixels, differing_pct, max_channel_delta, ssim }`,
plus `write_diff_image` (red-XOR overlay on desaturated original)
and `write_report` (the textual `report.txt` format documented
in § Image-Diff Policy). `Exact` and `Tolerance` are pure-Rust
loops over `image::Rgba<u8>` iterators; `Ssim` calls
`image-compare`.

#### Step 1.5 — Wire the four fixture manifests (≈ 0.5 day)

Author four `.fixture.toml` files under
`crates/gui-render/testdata/golden/board/`, one per existing
text project. Use the manifest schema from § Fixture Manifest
Approach.

Each manifest's `[input]` section points at the existing
`crates/engine/testdata/golden/text/native/<name>/` project.

#### Step 1.6 — Bless the four goldens (≈ 0.25 day, plus container build time)

Build the `datum-visual-ci` Docker image once. Run the bless
tool (Step 1.7) against each fixture. Commit the four
`.golden.png` files alongside the four manifests.

#### Step 1.7 — Implement the bless CLI (≈ 1 day)

Add `crates/visual-bless/` (a new tiny binary crate) invoked as
`datum-visual-bless --fixture <name> --reason <quoted-string>`.
The binary: detects whether it is running inside the visual-CI
container and, if not, re-invokes itself via `docker run`
against the pinned image; loads the manifest; renders the
fixture through `OffscreenRenderer`; embeds provenance hashes
(manifest sha256, renderer crate version, wgpu version, Mesa
version, font-set sha256) into the golden PNG's `tEXt` chunk
via the `image` crate's `PngEncoder` + ancillary-chunk write;
updates manifest provenance fields; prints the `VISUAL-REBLESS:`
commit-message footer to stdout for the developer to paste.

#### Step 1.8 — Plumb the test harness (≈ 0.5 day)

Add `crates/gui-render/tests/visual/board_text.rs` with
four `#[test]` functions (`text_density_repro`,
`text_fidelity_repro`, `text_intent_repro`,
`text_transform_repro`) that each call a shared
`run_fixture(name)` helper. The helper loads the manifest,
applies determinism env vars, constructs an `OffscreenRenderer`
at the manifest's viewport size, builds the scene from the
manifest's input project, renders, reads pixels, opens the
expected PNG, calls `compare` with the manifest's policy, and
on failure writes `<fixture>.actual.png`,
`<fixture>.diff.png`, and `<fixture>.report.txt` next to the
golden, then panics with the diff path. All four tests are
gated by `#![cfg(feature = "visual")]` at the module top.

#### Step 1.9 — Document the failure-artifact path (≈ 0.5 day)

Add `crates/gui-render/testdata/golden/board/README.md`
explaining:
- Purpose of the directory
- Manifest schema reference
- Where to find diff artifacts on failure
- How to run the bless tool
- The `VISUAL-REBLESS:` footer convention

#### Step 1.10 — Add the CI workflow (NOT in research-only scope)

The CI workflow YAML files are owned by the user. The
recommendation provides the conceptual workflow above (§ CI
Strategy). Research-only mode does **not** add `.github/workflows/`
files.

### Phase 1 effort estimate

| Step | Effort |
|------|--------|
| 1.1 cargo feature | 0.25 day |
| 1.2 OffscreenRenderer | 2.0 days |
| 1.3 manifest loader | 0.5 day |
| 1.4 comparator harness | 1.0 day |
| 1.5 four manifests | 0.5 day |
| 1.6 bless four goldens | 0.25 day (+ container build) |
| 1.7 bless CLI | 1.0 day |
| 1.8 test harness | 0.5 day |
| 1.9 docs | 0.5 day |
| **Total** | **~ 6.5 engineering days** |

Add 1.5 days of buffer for the inevitable wgpu surface-config
tuning and the first painful golden-blessing cycle (where you
discover the AA setting in the application differs from what the
manifest declared, etc.). **Realistic Phase 1 estimate: 5–8
engineering days.**

---

## Phases 2–4 Roadmap

### Phase 2 — Board-rendering goldens beyond text (≈ 8–14 days)

Expand Layer A's fixture set from text-only to the full lane
vocabulary of the M7 semantic contract.

Fixture additions:
- Authored copper at three densities (sparse / medium / dense)
- Unrouted ratsnest with endpoint anchors at three lengths
- Proposed geometry overlays in light and dark themes
- Diagnostic emphasis (focus + dim) at one DRC violation
- Layer toggle: front-only, back-only, F-Cu only
- Selection state (one component selected)
- Imported-board: same project rendered from a project where
  KiCad `render_cache` is present and from one where it is
  absent (the M7 contract's "imported-board semantic invariance
  rule"); fixture asserts the two PNGs are identical
- Edge.Cuts overlay on a non-rectangular board outline
- Vias: through-hole pad and via stack rendering
- Mask / paste / silkscreen colour rendering

Estimated 12–20 fixtures, 1–2 hours each (manifest authoring,
bless, eyes-on review). The renderer infrastructure already
exists from Phase 1; the work is primarily fixture authoring.

Effort: **8–14 days** depending on how many real corpus boards
are needed. Per `feedback_no_synthetic_kicad`, every fixture
input is a real project; if a needed scene state is not
representable from existing corpus boards, the work pauses to
ask the user for a real fixture file.

### Phase 3 — Full-GUI goldens (Layer B) (≈ 10–15 days)

This phase brings up Layer B end to end.

Subphases:
- 3a — Container additions for Xvfb + scrot (already in the
  Phase 1 Dockerfile; re-validate)
- 3b — `datum-gui` debug screenshot mode: a `--shell-screenshot
  <fixture-name>` flag that opens the fixture, prints
  `READY_FOR_SCREENSHOT`, screenshots the window after a
  short quiescence delay, exits 0
- 3c — `crates/gui-app/src/visual_capture.rs` for the
  application-side screenshot path (calls scrot via
  `std::process::Command` after stdout-publishing readiness)
- 3d — Layer B manifest schema (already drafted in § Fixture
  Manifest Approach)
- 3e — Initial Layer B fixture set (≈ 8 fixtures): empty board,
  populated board, light theme, dark theme, terminal lane
  expanded, AI lane expanded, inspector panel populated,
  modal-dialog appearance
- 3f — Mask authoring for any fixture that includes a known
  dynamic region

Estimated 8 fixtures × 1.5 hours each (Layer B fixtures take
longer due to async/quiescence debugging) plus the
application-side debug mode. **Effort: 10–15 days.**

### Phase 4 — Interaction goldens (long-term, ≈ 3–6 weeks)

This phase introduces scripted interaction sequences.

Subphases:
- 4a — A small interaction-script DSL (TOML or YAML) that
  describes a sequence of user inputs (pan, zoom, click,
  hover, key chord)
- 4b — winit user-input injection for Layer B (winit supports
  programmatic event injection in test/headless mode)
- 4c — A multi-step fixture format extension where the manifest
  declares a sequence of `[step.0]`, `[step.1]`, ... blocks
  each with a captured golden
- 4d — A small fixture set covering: pan/zoom interaction,
  selection click, hover state, route in progress (when M7
  routing-in-canvas lands)

Phase 4 is **not Phase 1's problem**. It is enumerated here so
the architecture is forward-compatible — the manifest schema
above already has room for a `[steps]` array; the harness
already supports per-step PNG comparison; the only missing
piece is the input-injection driver. Effort estimate is
deliberately wide because input injection in headless winit has
known sharp edges. Recommend re-scoping Phase 4 only after
Phase 3 is in production for at least 4 weeks.

### Total roadmap effort

| Phase | Effort |
|-------|--------|
| Phase 1 — text-only Layer A seed | 5–8 days |
| Phase 2 — full lane Layer A | 8–14 days |
| Phase 3 — Layer B shell goldens | 10–15 days |
| Phase 4 — interaction goldens | 15–30 days |
| **Total** | **~ 38–67 engineering days** (≈ 8–14 weeks) |

With reasonable assumption of one engineer half-time on the
work, the full visual regression substrate lands in **3–6
calendar months** alongside other M7 work.

---

## Open Questions and Risks

These are the items the research could not resolve definitively,
listed honestly so they don't surprise the implementer.

1. **wgpu 28 offscreen-render bit-determinism across container
   restarts.** The wgpu repository's CI uses lavapipe for
   software-vulkan testing and the renderer-comparison tests
   pass bit-for-bit, but Datum's specific render passes
   (custom shaders for stroked paths, glyph quads from
   `glyphon`/`cosmic-text`) have not been tested for bit-
   stability across container restarts. **Recommend an
   empirical test in Phase 1**: render the same scene 100
   times in a row, hash each output PNG, assert all hashes
   identical. If this fails, the comparator default for that
   fixture moves from `exact` to `tolerance` with a documented
   reason.

2. **`cosmic-text` 0.15's antialiasing-mode default and how to
   override it.** `cosmic-text` exposes `SwashCache` for glyph
   rasterization but the documented public API for forcing
   greyscale-only AA may have changed across recent versions.
   **Recommend an empirical test in Phase 1**: render the same
   text fixture once, change the AA mode setting, render
   again, diff. If the output differs, the lever exists. If
   not, document this as a known determinism risk and fall
   back to `tolerance` with `per_pixel = 1` for text
   fixtures.

3. **`glyphon` 0.10's atlas packing determinism.** The glyph
   atlas is built incrementally as glyphs are requested. Two
   identical scenes that request glyphs in different orders
   may produce different atlas layouts and therefore different
   texture-coordinate floats, which could perturb pixel
   coverage at glyph-quad edges. **Recommend an empirical test
   in Phase 1** using the four text fixtures: render in lexical
   order vs reverse-lexical order; bit-compare. If different,
   pre-warm the atlas with a deterministic glyph order at
   fixture setup.

4. **Lavapipe vs llvmpipe pixel-stability deltas.** Mesa's
   Vulkan software ICD (lavapipe) and OpenGL software driver
   (llvmpipe) use different code paths internally. wgpu on
   `WGPU_BACKEND=vulkan` uses lavapipe; if anything ever falls
   back to OpenGL, llvmpipe pixel output will differ. The CI
   container forces vulkan and refuses to run otherwise, but
   developer machines without lavapipe installed may
   accidentally render through llvmpipe. **Recommend the
   harness asserts at fixture setup time that the active
   adapter is "llvmpipe"** (lavapipe identifies itself this way
   in `wgpu::AdapterInfo`) and refuses to compare otherwise.

5. **HiDPI on developer machines.** Developers on HiDPI
   displays may see goldens render at a different DPI even with
   `GDK_SCALE=1` etc. set, because winit's monitor scale
   factor is queried at window creation. For Layer A this
   doesn't matter (no winit). For Layer B it does. **Recommend
   the Layer B harness force `WaylandSettings::set_scale_factor`
   / `X11Window::set_scale_factor` to 1.0 explicitly via
   winit's API**, falling back to a runtime panic if the API
   is not available.

6. **Imported-board `render_cache` invariance fixture.** The
   M7 contract requires that an imported board renders
   identically whether KiCad's `render_cache` polygons are
   present or synthesized. This implies a Phase 2 fixture
   that contains *two* imported boards (one with, one
   without) and asserts identical pixels. **This depends on
   the Phase 1 text engine landing the Newstroke-equivalent
   generator described in `PCB_TEXT_RENDERING_RESEARCH.md`**;
   until that lands, this fixture is unbuildable. Track as a
   Phase 2 dependency.

7. **`insta` for manifest snapshots — friction worth?** The
   recommendation uses `insta` for the JSON serialization of
   the scene-under-test, separate from the PNG. This means
   `cargo insta review` becomes part of the workflow for
   metadata changes. If the team finds the dual-tool workflow
   confusing, the fallback is to drop `insta` and write the
   scene serialization to a `<fixture>.scene.json` golden
   file that's compared like any other golden. Either is
   workable; the choice is operational, not architectural.

8. **PNG storage growth.** A 1024×768 RGBA PNG of a typical
   board scene is ≈ 30–80 KB. At 50 fixtures that's ≈ 4 MB; at
   200 fixtures, ≈ 16 MB. This is well within "fine for git";
   no LFS is needed. But if interaction-step goldens land in
   Phase 4 and a single fixture has 12 steps, the multiplier
   bites. **Recommend a periodic audit (every 6 months) of
   total `testdata/golden/**/*.png` size**; if it exceeds
   ~ 100 MB, revisit LFS.

9. **Reblessing wave coordination.** The Mesa version pin in
   the CI container will eventually need to update (security,
   feature, or bug fix). When it does, every Layer A and
   Layer B fixture potentially needs reblessing. **Recommend a
   dedicated `mesa-rebless` ceremony**: bump the container,
   run all visual tests, expect bulk failures, run the bless
   tool against everything, file one commit with
   `VISUAL-REBLESS-BULK: Mesa upgrade <old> -> <new>` and the
   complete list of affected fixtures. The user reviews the
   bulk diff PNGs in artifacts and approves out-of-band.

10. **Operator awareness when goldens are masked heavily.**
    Masking is necessary but easy to overuse. If a fixture's
    mask covers 60% of the image, the fixture is testing very
    little. **Recommend the harness emits a warning at run
    time** for any fixture where the mask covers more than 25%
    of the image, and a hard refusal-to-bless for masks
    covering more than 50%. The thresholds are tunable.

---

## Industry Reference Points

Brief, focused references. Each contributes a specific lesson the
recommendation absorbs.

- **KiCad QA** — `qa/` tree golden-image tests for plot output
  (Gerber/PDF) plus targeted GAL render-output comparisons; no
  comprehensive visual suite. KiCad's `pcbnew_print_test` is the
  "constructed scene to PNG" pattern Layer A mirrors at higher
  fidelity.
- **Blender** — image-based regression for Cycles/Eevee with
  tolerance per-pixel diffs and a maintained blessed set per
  engine. Lesson: GPU/driver pinning plus tolerance bands works
  at scale, but the blessed-set maintenance cost is real. Datum's
  `VISUAL-REBLESS:` footer is the lightweight equivalent of
  Blender's blessing ceremony.
- **Chromium pixel tests** — the most-evolved industry instance;
  currently runs a two-tier approach (GPU per-pixel internal
  correctness + full-window shell screenshots) that closely
  parallels this research's Layer A / Layer B split. The hard-
  won 15-year lesson is that the two layers must remain
  operationally distinct.
- **`egui_kittest`** — not applicable to Datum (no egui in tree)
  but the architecture is informative: it runs egui headlessly
  against a `Painter` into `image::RgbaImage`, then snapshots.
  Convergent evidence that the wgpu-style Layer A pattern is
  industry-correct.
- **Skia Gold** — service-backed image-diff with a triage web UI;
  heavyweight and out of scope. Lessons that translate: (a)
  artifact-on-failure is non-negotiable; (b) triage UX is the
  biggest operator-effectiveness lever; (c) exact-pixel plus a
  small tolerance band suffices for software-rasterized output.
- **Inkscape `--export-png`** — architecturally identical to
  Layer A: no window, no display server, just a renderer-against-
  input-file producing PNG. Operates with a 0.5% pixel tolerance
  + ≤ 2 LSB per-channel delta as the standing fallback band; useful
  as Datum's fall-back tolerance if exact-pixel ever proves
  unworkable per fixture.

---

## Sources

URLs, crate-page citations, and project-specific references
consulted during this research. Each carries a one-line note
on what it provided.

### Datum-internal documents

- `/home/bfadmin/Documents/datum-eda/CLAUDE.md` — project
  overview, attribution policy (no `Co-Authored-By`, no
  "Generated by", no AI markers).
- `/home/bfadmin/Documents/datum-eda/docs/gui/M7_RENDER_SEMANTIC_CONTRACT.md`
  — lane vocabulary (authored / unrouted / proposed /
  diagnostic) and the imported-board semantic invariance rule.
  This research's Layer A fixture coverage maps directly to
  these lanes.
- `/home/bfadmin/Documents/datum-eda/docs/gui/M7_RENDER_LAYER_DISCIPLINE_MEMO.md`
  — current renderer hybrid-state and the render-stack policy.
  Influences the Layer A fixture set (post-copper stage walk
  ordering must be visually verified).
- `/home/bfadmin/Documents/datum-eda/docs/gui/TECHNICAL_PRINCIPLES.md`
  — engine/daemon/frontend boundary; the "snapshot or image-
  based render checks where appropriate" principle this
  research formalises.
- `/home/bfadmin/Documents/datum-eda/research/pcb-text-rendering/PCB_TEXT_RENDERING_RESEARCH.md`
  — Phase 1 stroke-font foundation; the Newstroke-equivalent
  generator that Layer A's text fixtures depend on.
- `/home/bfadmin/Documents/datum-eda/research/pcb-text-rendering/DATUM_TEXT_ENGINE_PHASE_2_RESEARCH.md`
  — Phase 2 outline-font extension; *the* source for the
  cross-architecture float-determinism risk that drives this
  research's recommendation to make x86_64-linux the sole
  golden authority.
- `/home/bfadmin/Documents/datum-eda/crates/engine/testdata/golden/text/native/`
  — the four existing text fixture projects that Phase 1 of
  this research adopts as the seed corpus for Layer A.
- `/home/bfadmin/Documents/datum-eda/crates/gui-render/Cargo.toml`,
  `/home/bfadmin/Documents/datum-eda/crates/gui-app/Cargo.toml`,
  `/home/bfadmin/Documents/datum-eda/Cargo.lock` — the
  authoritative reference for what Datum's GUI stack actually
  is (wgpu 28, winit 0.30, glyphon 0.10, cosmic-text 0.15,
  swash 0.2.7, image 0.25.10).

### Rust crate documentation

- wgpu — https://docs.rs/wgpu/28/ — `Texture::copy_to_buffer`
  documentation, the offscreen capture pattern.
- wgpu examples — https://github.com/gfx-rs/wgpu/tree/trunk/examples
  — the `capture` example is the pattern Phase 1 mirrors.
- image — https://docs.rs/image/0.25/ — PNG codec, `RgbaImage`,
  `tEXt` chunk support for provenance hashing.
- image-compare — https://docs.rs/image-compare/0.4/ — SSIM
  and per-pixel comparators under MIT.
- glyphon — https://github.com/grovesNL/glyphon — wgpu-native
  text rendering atop cosmic-text.
- cosmic-text — https://docs.rs/cosmic-text/0.15/ — text
  shaping/layout with swash backend; the `SwashCache` API for
  glyph rasterization.
- swash — https://docs.rs/swash/0.2/ — TrueType/OpenType
  rasterization; the floating-point Bézier subdivision risk
  source.
- insta — https://insta.rs/ — snapshot testing framework
  (Apache-2.0); recommended for manifest snapshots only.
- bytemuck — https://docs.rs/bytemuck/ — safe `&[u8]`
  reinterpretation of mapped wgpu buffers.
- pollster — https://docs.rs/pollster/ — synchronous future
  blocking inside `#[test]` functions.

### License-filter checks (per `feedback_no_copyleft_integration`)

- dssim — https://github.com/kornelski/dssim — AGPL-3.0,
  **excluded** from the recommendation.
- ImageMagick — https://imagemagick.org/script/license.php —
  Apache-2.0; permitted only as subprocess invocation.
- FreeType — https://freetype.org/license.html — dual
  FreeType/GPLv2; cosmic-text+swash use the FreeType-licence
  path, which is permissive.
- Mesa — https://docs.mesa3d.org/license.html — MIT-class for
  the relevant components (lavapipe, llvmpipe, loader);
  permitted as system component / subprocess.
- Xvfb — part of `xorg-server`, MIT-equivalent; permitted as
  subprocess.
- scrot — `BSD-2-Clause`-style "X11 license" derivative;
  permitted as subprocess.

### External infrastructure references

- KiCad qa — https://gitlab.com/kicad/code/kicad/-/tree/master/qa
  — `pcbnew_print_test` and the gerber-output golden tests;
  the architectural pattern Datum's Layer A mirrors.
- Blender CI — https://wiki.blender.org/wiki/Tools/Tests/
  Render — render-engine pixel diffing with tolerance bands
  and Mesa pinning; the maintenance-cost reference point for
  large fixture sets.
- Chromium pixel tests — https://chromium.googlesource.com/
  chromium/src/+/main/docs/testing/gpu_testing.md — the
  industry reference for two-tier visual regression; the
  source of "the two layers must remain operationally
  distinct" lesson.
- Skia Gold — https://skia.org/docs/dev/testing/skiagold/ —
  service-backed image-diff and triage; the reference point
  for what a heavyweight golden infrastructure looks like
  (and why Datum should explicitly not try to build one).
- egui_kittest — https://docs.rs/egui_kittest/ — egui's
  official visual snapshot framework; informative
  architectural parallel even though Datum does not use egui.
- Inkscape regression tests — https://gitlab.com/inkscape/
  inkscape/-/tree/master/testfiles — `--export-png` →
  diff golden pattern; the source for fall-back tolerance
  defaults if exact-pixel ever proves unworkable.
- Mesa lavapipe — https://docs.mesa3d.org/drivers/llvmpipe.html
  — software vulkan ICD; the Datum CI container's required
  vulkan provider.
- wgpu lavapipe-CI usage — https://github.com/gfx-rs/wgpu/
  blob/trunk/.github/workflows/ci.yml — confirms upstream wgpu
  itself uses lavapipe for CI, validating the choice.

### Notes on what this research did NOT consult (per skip list)

- Web visual regression tools (Percy, Chromatic, BackstopJS,
  Loki) — explicitly excluded; Datum is native Rust, no web
  surface.
- Long historical surveys of Chromium pixel test evolution —
  one paragraph above is the entire Chromium contribution to
  this research.
- General visual-regression theory beyond what fits the
  recommendation — not consulted.
- Tools without a Rust integration story — not consulted.

---

## Appendix — Layer A vs Layer B at-a-glance

| Aspect                    | Layer A (offscreen)                  | Layer B (full GUI)                    |
|---------------------------|--------------------------------------|----------------------------------------|
| What it tests             | Renderer output for a given scene    | Full app shell + window contents       |
| Crate under test          | `datum-gui-render`                   | `datum-gui-app`                        |
| Display server required   | No                                   | Yes (Xvfb)                             |
| GPU required              | No (wgpu via lavapipe)               | No (wgpu via lavapipe)                 |
| Async readback            | Buffer map after queue submit        | App stdout-publishes ready signal      |
| Default diff policy       | Exact pixel                          | SSIM ≥ 0.985                           |
| Default fixture size      | 1024×768 RGBA8                       | 1920×1080 RGBA8                        |
| Wall-clock per fixture    | ≈ 200 ms                             | ≈ 5–7 seconds (boot + screenshot)      |
| Total fixture count goal  | 50–200 (steady state)                | 8–30 (steady state)                    |
| Catches lane regressions  | Yes (M7 contract lanes)              | No (canvas is one pixel block)         |
| Catches shell regressions | No                                   | Yes (panels, theme, layout, modals)    |
| Recommended priority      | **Primary defence** — every PR       | **Upper layer** — every PR, smaller    |

---

*End of research artifact. Research-only mode: no spec edits,
no commits, no branches, no PRs. Integration owned by the
user.*
