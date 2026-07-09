# Board Editor — Reference-Image Human-Review Loop

> **Status**: active / governed.
> **Ratified under**: decision 019 (GUI product-model recovery) on the systems of
> decisions 014 (layout) + 015 (design system).
> **Role**: this directory is the **HUMAN layer** of
> `docs/gui/DATUM_GUI_CONFORMANCE_SPEC.md`. That spec gives every prototype claim
> exactly one honest check disposition — **ENFORCED** (a real gate/test/golden
> verifies it today), **TO-ENFORCE** (machine-checkable; a named gate lands with
> the build slice), or **HUMAN** (not machine-verifiable — cross-engine fidelity,
> aesthetic / IA / owner-eye judgment). The HUMAN rows say "reviewed against the
> reference image + the committed chrome golden." **This directory is where the
> reference image lives and how that review is run.** It does not redefine the
> target: `docs/gui/prototypes/board-editor.html` + `docs/gui/VISUAL_LANGUAGE.md`
> remain authoritative.

## 0. The hard rule (binding, stated once)

**A Rust/wgpu render will NEVER pixel-match an HTML/CSS render.** Nothing in this
loop pixel-diffs the build against `board-editor.html`. There are two layers and
they are checked by different machinery:

- **Machine layer** — token values, layout metrics, component structure, and
  rendered goldens **diffed build-vs-build** (same renderer) against committed
  golden PNGs. Lives in the gates named in the conformance spec
  (`scripts/check_gui_design_tokens.py`, `crates/gui-render/src/*` layout/contract
  tests, `crates/gui-render/src/visual_runner.rs` + the board goldens, and the
  **composed-shell visual-parity gate** `scripts/check_gui_visual_parity.py` vs
  the owner-approved shell golden
  `crates/gui-render/testdata/golden/shell/datum-shell.golden.png` — captured from
  the datum-test board with a preset R1 component selection, a populated
  single-pane composition per `board-editor.html`). **This
  directory adds no machine gate** — but the shell golden gate does enforce that
  the once-approved composed look does not silently regress (it is same-engine,
  build-vs-build, never wgpu-vs-HTML).
- **Human layer (this directory)** — the committed reference image
  `board-editor.png` (a faithful render of the prototype) reviewed **by eye**,
  region by region, against the build's committed chrome goldens at
  `crates/gui-render/testdata/golden/board/datum-test.scale-*.golden.png`. The
  question the reviewer answers is *"does the build read as the same product as
  the prototype?"* — composition, hierarchy, density, color feel, aesthetic — **not
  "are the pixels identical?"** (they never will be).

There is no "reference-vs-HTML gate" and this doc does not propose one.

## 1. What is in this directory

| File | What it is |
|---|---|
| `README.md` | This loop. |
| `board-editor.png` | The committed reference image — a render of `docs/gui/prototypes/board-editor.html` at the capture spec in §2. **This is a mockup you look at, not a value source you diff against.** |
| `board-editor.png.PENDING` | Placeholder note present **only while the real image has not been captured** (see §5). Delete it in the same commit that lands a real `board-editor.png`. |

When a real `board-editor.png` exists, `board-editor.png.PENDING` must not.

## 2. Capturing / refreshing the reference image (manual command)

The prototype is a body fragment (it opens with `<style>` and lays out with
`height:100vh`), so it must be captured in a real browser viewport. Capture is a
**deliberate manual step**, re-run only when `board-editor.html` changes.

**Canonical capture command** (Chromium / Chrome headless; run from repo root):

```bash
chromium --headless --no-sandbox --hide-scrollbars \
  --force-device-scale-factor=2 --window-size=1680,1050 \
  --virtual-time-budget=4000 \
  --screenshot="docs/gui/reference/board-editor.png" \
  "file://$(pwd)/docs/gui/prototypes/board-editor.html"
```

`google-chrome` / `google-chrome-stable` accept the identical flags. `--headless`
and `--headless=new` both work on current builds; if one SIGTRAPs in a restricted
environment, try the other, and see §5.

Fallback capture (WebKit, if no Chromium is available):

```bash
wkhtmltoimage --width 1680 --quality 100 \
  docs/gui/prototypes/board-editor.html docs/gui/reference/board-editor.png
```

**Capture parameters are fixed so the reference is stable and comparable:**

| Parameter | Value | Why |
|---|---|---|
| Window size | `1680 × 1050` | Desktop review size that gives every region enough room to read (menu bar, both side columns, split board pane, inspector, dock, status bar) without horizontal scroll. |
| Device scale | `2` (HiDPI) | The board goldens are captured HiDPI; a HiDPI reference keeps type weight / hairline density comparable to the eye. |
| Scrollbars | hidden | The prototype sets `overflow:hidden`; a stray scrollbar would shift the right column. |
| Virtual time budget | `4000ms` | Lets the inline `<script>` (collapsible sections, focus dots) settle before the shot. |

If you change any capture parameter, record it here in the same commit — the
reference image is only meaningful next to the parameters that produced it.

## 3. The region-by-region review protocol

Put `board-editor.png` and a fresh build golden side by side and walk the **nine
regions of the conformance spec** in order. The spec's per-region tables
(`DATUM_GUI_CONFORMANCE_SPEC.md` §2.1–§2.9) are the checklist; this loop is how you
walk the **HUMAN** rows of each.

Build golden to compare against (pick the scale nearest your display density;
`1_00` is the primary review target):

```
crates/gui-render/testdata/golden/board/datum-test.scale-1_00.golden.png
                                          …scale-1_25.golden.png
                                          …scale-1_50.golden.png
                                          …scale-2_00.golden.png
```

For each region, the reviewer asks the HUMAN-layer questions the spec assigns
(NOT pixel identity):

1. **Menu bar** (§2.1) — wordmark `Datum·EDA` with accent middot; rev pill reads
   right; titles balanced, not crowded or loose. (Width tuning itself is a
   **TO-ENFORCE** machine row — polish item 3 — not judged here.)
2. **Tool rail** (§2.2) — **open region-existence conflict** (§5a of the spec:
   prototype has no standalone rail). Do not "review" a region whose existence is
   unresolved; note the owner call, move on.
3. **Project tree** (§2.3) — row density and indent ladder *feel* right; selected
   row reads as accent-tinted with a left accent rule; fixture content legible.
4. **Layers panel** (§2.4) — swatch colors read as the right layer families
   (F.Cu warm red, B.Cu blue, silk near-white, edge gold, ratsnest grey);
   off/hidden layer visibly dimmed.
5. **Board pane** (§2.5) — **the protagonist.** Board reads as the same board:
   copper/silk/edge stack legible, selection glow reads. Two live defects are
   judged here at the HUMAN layer even though each also has a machine row:
   - **Polish item 1 (golden fit-to-board framing):** does the golden frame the
     board *fit-to-bounds and centered*, or is it tiny in a corner? A corner-tiny
     golden is **unreviewable** — flag it; the machine fix is
     `visual_runner.rs::golden_camera_frames_board_fit_to_bounds` (conformance §2.5,
     TO-ENFORCE), not a change to this image.
   - **Polish item 2 (overlay-text overflow):** does the canvas hint row
     (`F FIT / REVIEW NAV / …`) stay inside the board pane, or bleed into the
     inspector? Machine fix extends `layout_invariant_tests.rs` (conformance §2.5).
6. **Inspector** (§2.6) — cross-probe banner, title block (`ref` + kind +
   SELECTED chip), section headers, key/value grid with mono coordinates all read
   as the prototype's inspector.
7. **Dock** (§2.7) — tab strip reads right; run-dot on the active agent tab.
   (Collapsed height 44-vs-32 and journal CLI-string handoffs are **owner
   reconciliations**, spec §5b/§5c — recorded, not eyeballed to a verdict.)
8. **Status bar** (§2.8) — mono segments, hairline separators, accent focus
   segment, DRC cell. Note: the DRC cell **correctly** uses the muted chrome
   `STATUS_WARN`, which pre-answers polish item 4 for this region.
9. **Typography tokens** (§2.9) — overall type color/weight/rhythm reads as one
   family with the prototype. The type-ramp / spacing / radius *values* are a
   **TO-ENFORCE** parity arm (spec §4 G1), not an eyeball judgment.

**Polish item 4 (DRC / silk-bottom saturation)** is an **owner-eye token-value**
call, reviewed here but decided by the owner: `DRC_ERROR #FF4D4D` / `DRC_WARN
#FFB02E` are more saturated than the muted chrome `STATUS_*`, and `SILK_BOTTOM
#969BA1` differs from `SILK_TOP #E8E6DC`. **Do not change these values in this
loop.** They stay mirror-gated by `check_gui_design_tokens.py` so any future change
is deliberate; the reference render is where the owner judges whether they want
them softened.

## 4. Which claims are machine-checked vs human-reviewed (the split)

The conformance spec is the authority; this is the one-line orientation:

- **ENFORCED / TO-ENFORCE (machine layer) — NOT reviewed here.** Token values
  (`scripts/check_gui_design_tokens.py`), layout metrics and non-overlap
  (`crates/gui-render/src/*` layout-invariant tests), render-stack and region
  structure (`render_contract_tests.rs`), and build-vs-build goldens
  (`visual_runner.rs` + the board golden PNGs). If it can be asserted, it belongs
  to a gate, not to this loop.
- **HUMAN (this loop).** Cross-engine visual fidelity, composition / information
  architecture, density and color *feel*, aesthetic judgment, and owner-eye
  token-value calls (polish item 4). Reviewed by eye, reference image vs the
  committed chrome golden.

If a row you are "reviewing" here could instead be a value/structure assertion,
it is misfiled — push it back to the machine layer per conformance spec §4
(G1–G8). The HUMAN layer is the residue that genuinely cannot be computed, not a
dumping ground for un-built gates.

## 5. Capture status (honest, updated per capture)

> **Cross-link (whole-shell no-regression is now ENFORCED elsewhere).** This
> reference-image loop is the **HUMAN owner-judgment layer only** — it is no longer
> the sole line of defense for the composed shell look. Whole-shell composition
> parity is now regression-gated by **G9**
> (`DATUM_GUI_CONFORMANCE_SPEC.md` §2.10 / §4): the same-engine shell visual-parity
> gate `scripts/check_gui_visual_parity.py` diffs the running app against the
> owner-approved shell golden
> `crates/gui-render/testdata/golden/shell/datum-shell.golden.png` and FAILS on any
> regression (and if the golden is absent or a `*.PENDING` placeholder shadows it).
> That gate is build-vs-build (never wgpu-vs-HTML). This directory's job is the
> one-time cross-engine aesthetic judgment that blesses the golden and the
> region-by-region eyeball review — not machine enforcement.

**Current status: DEFERRED TO MANUAL CAPTURE — no real reference image committed
yet.**

The automated capture was attempted in the environment that stood up this loop.
Chromium 150 is installed with all shared libraries present, but it **SIGTRAPs and
core-dumps immediately** — even on a trivial `data:text/html` page, even with
`--no-sandbox --disable-gpu --single-process` — because the sandbox this ran in
blocks the namespace/seccomp calls Chromium needs at startup. This is an
environment blocker, not a missing dependency and not a problem with the prototype.

Rather than commit a fabricated or blank image, `board-editor.png.PENDING` is
committed as a placeholder. **To resolve:** on a workstation with a working
headless browser, run the §2 command, verify `board-editor.png` renders the full
board-editor shell, then commit it and delete `board-editor.png.PENDING` in the
same change. Update this section to `CAPTURED` with the date and the browser
build used.
