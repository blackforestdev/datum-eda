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
  **split-view first-slice shell no-regression gate** `scripts/check_gui_visual_parity.py`
  vs the committed shell golden
  `crates/gui-render/testdata/golden/shell/datum-shell.golden.png` — captured from
  the datum-test board with a preset R1 component selection, a populated two-pane
  composition: Board pane A + inspector, Schematic pane B a labeled placeholder).
  That golden is a **Phase-2 split-view FIRST-SLICE INTERIM** target — it now
  captures the real two-pane split LAYOUT, but pane B is a **"Schematic (coming)"
  placeholder** (no schematic content / authoring / cross-probe), and it is **NOT
  owner-approved against `board-editor.html`**, whose Schematic pane shows real
  content with populated cross-probe (buildable only in later Phase-2 slices).
  **This directory adds no machine gate** — the shell golden gate only enforces
  that the current first-slice look does not silently regress (same-engine,
  build-vs-build, never wgpu-vs-HTML); it does not certify prototype parity.
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
| `board-editor.png` | The committed reference image — a browser screenshot of `docs/gui/prototypes/board-editor.html` (actual capture recorded in §2/§5). **This is a mockup you look at, not a value source you diff against.** |
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

**Currently committed:** `board-editor.png` is a hand-captured browser screenshot
at **1920 × 953** (the owner's browser viewport) — not the headless
`1680 × 1050 @2×` recipe above. That is acceptable: the reference is an eyeball
design target, **never pixel-diffed against the app** (cross-engine wgpu-vs-HTML
never matches — see §5), so its exact size need not equal the app golden's. Use
the command above only when you want a *reproducible* HiDPI re-capture.

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

> **Cross-link (only the FIRST-SLICE look is regression-gated; the full split
> composition is later Phase-2).** This reference-image loop is the **HUMAN
> owner-judgment layer** for the full board-editor composition. What is machine-
> gated today is only the **current split-view first-slice** shell look, via **G9**
> (`DATUM_GUI_CONFORMANCE_SPEC.md` §2.10 / §4): the same-engine gate
> `scripts/check_gui_visual_parity.py` diffs the running app against the committed
> split-view first-slice shell golden
> `crates/gui-render/testdata/golden/shell/datum-shell.golden.png` and FAILS on any
> regression (and if the golden is absent or a `*.PENDING` placeholder shadows it).
> That golden now captures the real two-pane split LAYOUT, but pane B is a labeled
> placeholder (no schematic content). That gate is build-vs-build (never
> wgpu-vs-HTML) and it is **NOT** prototype-parity certification. The **full
> board-editor.html composition** — the SPLIT Board+Schematic view with a
> populated inspector and real schematic content — cannot be captured until the
> later Phase-2 slices render the schematic pane; its owner-approved reference is
> gated by **G10** (`scripts/check_gui_reference_capture.py`), which is now
> **GREEN** — this directory's `board-editor.png` is captured. G10 stays live so
> the reference can never silently vanish or be shadowed by a `*.PENDING` note.
> This directory's job is that one-time cross-engine aesthetic judgment and the
> region-by-region eyeball review — not machine enforcement of the first-slice
> golden.

**Current status: CAPTURED (2026-07-09).** `board-editor.png` is committed — a
**browser screenshot of `docs/gui/prototypes/board-editor.html`** taken by the
owner at **1920 × 953** (his browser viewport). This differs from the §2 canonical
headless recipe (`1680 × 1050 @2×`), and that is fine: the committed reference is a
**human design target reviewed by eye**, never pixel-diffed against the app
(cross-engine wgpu-vs-HTML never matches), so its exact dimensions need not equal
the app golden's. G10 is GREEN; the `*.PENDING` placeholder is deleted.

The app does **not yet match this reference**: the full split Board+Schematic
composition with a populated inspector and real schematic content is a **Phase-2
build**. That gap is honestly scoped as future work, not marked complete.

_History:_ the automated headless capture SIGTRAP'd in the sandbox that stood up
this loop (Chromium could not complete its namespace/seccomp startup there), so a
`board-editor.png.PENDING` placeholder held the slot rather than a fabricated
image. The owner then captured the reference directly from a browser and deleted
the placeholder — the state this section now records.
