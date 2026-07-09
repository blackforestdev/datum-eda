# Board Editor Conformance Spec (prototype → build, with check dispositions)

> **Status**: active / governed.
> **Ratified under**: decision 019 (GUI product-model recovery) on the systems of
> decisions 014 (layout) + 015 (design system).
> **This doc does NOT redefine the target.** The controlling visual prototype
> `docs/gui/prototypes/board-editor.html` and the Design Book
> `docs/gui/VISUAL_LANGUAGE.md` remain authoritative for *what the board editor
> is*. This spec makes each conformance claim **executable and honestly
> dispositioned** so the Phase-1 build can be driven to match the prototype and
> drift is caught. It is a governance extension of `DATUM_GUI_PHASE_1_SPEC.md`,
> not a rival design.
> **Contract of this document:** *every claim below is actionable and carries
> exactly one honest check disposition* (ENFORCED / TO-ENFORCE / HUMAN), each with
> a named link. No claim is left as unverifiable prose.

## 0. Authority & reference chain (read this first)

The reference chain has a **direction**, and the two ends are checked by different
machinery:

- **Chrome & layout (regions, geometry, composition):** the reference *image* is
  `docs/gui/prototypes/board-editor.html`. It is a mockup you look at, not a value
  source you diff against.
- **Tokens / type / spacing / radius (values):** the machine-authoritative source
  is `docs/gui/VISUAL_LANGUAGE.md` §2, mirrored into
  `crates/gui-render/src/design_tokens.rs`. **The prototype's `:root` font-stacks
  (`--sans: ui-sans-serif …`, `--mono: ui-monospace …`) and its hand-tuned pixel
  values (`height:33px`, `font-size:13.5px`, `--row:25px`, …) are placeholder
  mockup values, NOT conformance targets.** Where the prototype px and the Design
  Book token disagree, the token wins and the prototype is the doc to fix (or the
  delta is logged in §5 as an owner reconciliation).

**Hard rule, stated once and binding everywhere in this doc:** a Rust/wgpu render
will **never** pixel-match an HTML/CSS render. **No check anywhere in this spec
pixel-diffs the build against `board-editor.html`.** The two layers are:

- **Machine layer** — token values, layout metrics, component structure, and
  rendered *goldens diffed against committed golden PNGs* (build-vs-build, same
  renderer). This is where ENFORCED / TO-ENFORCE live.
- **Human layer** — the committed reference image (`board-editor.html`) reviewed
  by the owner against the committed **chrome goldens**
  (`crates/gui-render/testdata/golden/board/datum-test.scale-*.golden.png`) for
  cross-engine fidelity and aesthetic/IA judgment. This is where HUMAN lives.

## 1. Disposition legend (the discipline of this pilot)

Three dispositions, and only three. **Every row names a link** — a disposition
without a concrete reference is not a valid row.

| Disposition | Meaning | Link requirement |
|---|---|---|
| **ENFORCED** | A real existing gate / test / golden verifies this claim **today**. | Must name the file + gate/test function that verifies it now. |
| **TO-ENFORCE** | Machine-checkable, but the gate does **not** exist yet. | Must name the exact test/gate **file + the assertion to add**. **RULE:** never mark a TO-ENFORCE green, and never land it **red against un-built structure** — it lands *with the build slice it verifies*. |
| **HUMAN** | Not machine-verifiable (cross-engine pixel fidelity, aesthetic / IA / owner-eye / token-value judgment). | Must name the **reference image** + the committed **golden** it is reviewed against. |

No row proposes pixel-diffing wgpu against HTML. No row is uncomputable-yet-marked-green.

## 2. Per-region conformance tables

Nine audited regions, all covered: **menu-bar · tool-rail · project-tree ·
layers-panel · board-pane · inspector · dock · status-bar · typography-tokens.**
Row shape: **claim | prototype value | build state | phase scope | disposition |
check ref.** Rows are transcribed from the region audits — a faithful
transcription, not a re-derivation.

Legend for *phase scope*: **P1** = Phase-1 in-scope (read-only board review);
**DEF** = deferred (authoring / other surfaces, per `DATUM_GUI_PHASE_1_SPEC.md`).

---

### 2.1 Menu-bar
*Phase scope: P1 (chrome present; items rendered from manifest, mutating items
disabled).* **Headline:** structure + token fill are enforced; the one real gap is
title **width tuning** (item 3) — the 0.78 advance over-spaces IBM Plex Condensed.

| Claim | Prototype value | Build state | Phase | Disposition | Check ref |
|---|---|---|---|---|---|
| Menu bar is the top strip, full width, single row | `.menubar` `height:33px` `flex:0 0 33px` | `ShellLayout.top_menu_bar`, `height = SP_07 + SP_01 = 34px` | P1 | ENFORCED | `layout_invariant_tests.rs::shell_and_hit_regions_hold_layout_invariants_across_scale_matrix` (PG-UI-LAYOUT-INVARIANTS) |
| Bar fill = shell background | `background:var(--bg)` `#121318` | fill `design_tokens::chrome::BG_BASE` | P1 | ENFORCED | `scripts/check_gui_design_tokens.py` (value); region *usage* → §4 gap |
| Bottom hairline divider | `border-bottom:1px var(--bd-sub)` | `BORDER_SUBTLE` | P1 | ENFORCED | `check_gui_design_tokens.py` (value) |
| Titles: File Edit View Place Route Project Checks Manufacturing Window Help | 10 literal `.menu` spans | rendered **from `menu_model.json`** in `menu_chrome.rs::render_menu_bar` | P1 | ENFORCED | `menu_chrome.rs::menu_model_is_available_to_renderer` |
| Adjacent titles never overlap (incl. "Manufacturing") | flow layout, `padding:3px 9px` | `menu_title_width` via `estimated_text_run_width_px` | P1 | ENFORCED | `menu_chrome.rs` menu-layout test (no-overlap side) |
| **Title width not over-spaced** (item 3) | Plex Condensed is narrower than 0.78 advance | `TextFace::Ui => 0.78` flat advance, `lib.rs:6180` | P1 | **TO-ENFORCE** | replace flat `0.78` at `lib.rs:6178` with a Plex-Condensed measured advance; add a **two-sided** metric test in `menu_chrome.rs` (box width ≤ measured glyph run + prototype padding within tol, AND no overlap) |
| Wordmark `Datum·EDA`, accent middot | `.brand` 600/14px, `b{color:var(--acc)}` | brand run, `ACCENT` middot | P1 | HUMAN | ref `board-editor.html` vs golden `datum-test.scale-1_00.golden.png` |
| `Route` menu context-muted | `.menu.ctx{color:var(--tx3)}` | ctx-muting policy open | P1 | HUMAN (owner) | §5(d) reconciliation |
| Right-side rev pill `stm32-sensor-node · rev a3f19c` | `.pill` mono 12px `--tx3` | pill run, `TEXT_MUTED` mono | P1 | HUMAN | ref image vs golden |

---

### 2.2 Tool-rail
*Phase scope: P1 chrome region exists in the shell; its very existence conflicts
with the prototype (see §5a).* **Headline:** **open region-existence conflict** —
the prototype has **no** standalone left tool rail (tools live per-pane in
`.pane-tools`), but the build's `ShellLayout` reserves a `tool_rail` column and
`DATUM_GUI_PHASE_1_SPEC.md` L40/L54 name a "left tool rail". Structural checks here
are **not authoritative until §5a is resolved.**

| Claim | Prototype value | Build state | Phase | Disposition | Check ref |
|---|---|---|---|---|---|
| A standalone left tool rail exists | **absent** — tools are per-pane (`.pane-tools` inside `.pane-hd`) | `ShellLayout.tool_rail`, `width = SP_09 = 48px` | P1 | HUMAN (owner) | §5(a) — region-existence conflict; blocks authoritative structural checks |
| Rail (if kept) does not overlap columns / stays in window | n/a (mockup has none) | Taffy-solved rail rect | P1 | ENFORCED | `layout_invariant_tests.rs` (non-overlap / within-window over scale matrix) |
| Per-pane tool cluster (Select/Move/Route/Via/Zone) | `.pane-tools` `.ptool` 25px, active = `--acc` outline + `--acc-tint` | pane-level tool cluster not built (read-only) | DEF | TO-ENFORCE | on the authoring slice, add a `render_contract_tests.rs` structural test for the pane tool cluster; **do not land red now** |

---

### 2.3 Project-tree
*Phase scope: P1 (read-only navigation).* **Headline:** panel + selected-row token
structure enforced; tree content is fixture-driven; row density (`--row:25px`) is a
placeholder px not a token target.

| Claim | Prototype value | Build state | Phase | Disposition | Check ref |
|---|---|---|---|---|---|
| Tree lives in the left column, above Layers | `.col.left` `flex:0 0 224px` | `ShellLayout.left_sidebar`, `left_width = 228px` | P1 | ENFORCED | `layout_invariant_tests.rs` (columns don't overlap; cards within sidebar) |
| Panel header: uppercase, secondary text, count on right | `.panel-hd` 28px, 11.5px/600, `--tx2`; `.count` mono `--tx3` | header from `type.header` + `TEXT_SECONDARY`, count `TEXT_MUTED` mono | P1 | ENFORCED | `check_gui_design_tokens.py` (values); *usage* → §4 gap |
| Selected row: accent tint + left accent rule | `.tree-row.sel{background:var(--acc-tint);box-shadow:inset 2px 0 0 var(--acc)}` | `REVIEW_ROW_ACTIVE_BG = ACCENT_TINT` + accent rule | P1 | ENFORCED | `check_gui_design_tokens.py` (ACCENT/ACCENT_TINT values) |
| Row height / indent ladder (l2 26 / l3 44) | `--row:25px`, `l2:26px`, `l3:44px` | row height from spacing tokens (px is placeholder, not a target) | P1 | HUMAN | ref image vs golden (density judged, not px-diffed) |
| Content: project → Schematic[pane B] / Board[pane A]·sel → Components 24 / Nets 41; Library → Footprints 18 / Symbols 22 | literal tree markup | tree from resolved fixture model | P1 | HUMAN | ref image vs `datum-test.scale-*.golden.png` |
| `pane-tag` / `follows pane` binding chips | `.pane-tag` acc-bordered; `Layers follows pane A` | split-pane / pane-binding not built | DEF | TO-ENFORCE | on the split-view slice, structural test in `render_contract_tests.rs`; not red now |

---

### 2.4 Layers-panel
*Phase scope: P1 (read-only; visibility toggles as view state).* **Headline:**
swatch→content-token mapping is value-gated but **not usage-gated** (which swatch
binds which layer token is untested — §4).

| Claim | Prototype value | Build state | Phase | Disposition | Check ref |
|---|---|---|---|---|---|
| Layers panel sits below the tree in the left column | `.panel` `flex:0 0 auto`, header "Layers" | left-column lower panel | P1 | ENFORCED | `layout_invariant_tests.rs` |
| Active layer row: accent tint + left accent rule | `.layer-row.active` same as tree sel | active-row accent tint | P1 | ENFORCED | `check_gui_design_tokens.py` (ACCENT_TINT) |
| Swatches use **content** tokens (F.Cu, B.Cu, silk, edge, ratsnest) | `--cu-f #C83A34`, `--cu-b #4D7FC4`, `--silk #E8E6DC`, `--edge #CBB24A`, `--rat #AEB4BB` | `content::COPPER_FRONT/BACK`, `SILK_TOP`, `EDGE`, `RATSNEST` | P1 | ENFORCED (value) | `check_gui_design_tokens.py` REQUIRED_CONTENT_TOKENS |
| **Which swatch binds which content token** (usage) | swatch `style="background:var(--cu-f)"` etc. | layer→token binding in render path | P1 | **TO-ENFORCE** | region-token-binding test in `render_contract_tests.rs` (swatch → family token); values are gated, usage is not (§4) |
| Off/hidden layer dimming (Ratsnest) | `.layer-row.off{opacity:.4}` + `--tx3` name | visibility dim state | P1 | HUMAN | ref image vs golden |

---

### 2.5 Board-pane
*Phase scope: P1 (the protagonist — board rendered from resolved model, read-only).*
**Headline:** render fidelity is golden-backed, but **two concrete defects** live
here: golden capture framing (item 1) and overlay-text overflow (item 2).

| Claim | Prototype value | Build state | Phase | Disposition | Check ref |
|---|---|---|---|---|---|
| Canvas is the deepest stage, distinct from chrome | `.pane{background:var(--canvas)}` `#0B0C0E` | `CANVAS` / board-field surface | P1 | ENFORCED | `check_gui_design_tokens.py` (CANVAS value) |
| Layer render stack order (back→front, group ladder) | copper under silk under edge (visual) | `scene_layer_stack_priority` ladder | P1 | ENFORCED | `render_contract_tests.rs::render_stack_policy_follows_declared_contract` |
| Copper built from tokens, not raw literals | `--cu-f/--cu-b` widths, glow | material-first copper from `design_tokens::content` | P1 | ENFORCED | `check_gui_design_tokens.py` copper-consumption guard (`from_copper_material([…])` fence) |
| Selection = accent outline + glow (cross-probed U1) | `stroke:var(--acc)` + `#glow` filter | accent selection material | P1 | ENFORCED (value) / HUMAN (look) | value: `check_gui_design_tokens.py`; look: ref image vs golden |
| **Golden captures the board fit-to-bounds** (item 1) | mockup shows board filling the pane | `camera_for_manifest` yields ~32% zoom, board tiny lower-right → goldens unreviewable | P1 | **TO-ENFORCE** | new `visual_runner.rs::golden_camera_frames_board_fit_to_bounds`: for the `datum-test` manifest assert camera **center == scene bounds center** AND effective zoom == `fit_scale` within tol (projected board bbox ≥ ~70% of `board_field`, centered). **Not a pixel-diff.** |
| **Overlay hint stays within the viewport** (item 2) | n/a (mockup has no overflowing hint) | hint `"F FIT  [ ] REVIEW NAV  …"` drawn via `draw_text` (unclipped) at `lib.rs:1193`, overflows into inspector | P1 | **TO-ENFORCE** | extend `layout_invariant_tests.rs`: assert every overlay `TextRun` box (`x + estimated_text_run_width_px`) ≤ `layout.viewport` right edge across the scale matrix; fix routes the hint through `draw_text_clipped` / `truncate_text` |
| Board-field border color | grid `#171A20`, edge `--edge` | **raw `[0.46,0.49,0.53]` literal** at `lib.rs:1275` | P1 | **TO-ENFORCE** | ad-hoc color lint (§4): fence raw `[f32;3]` chrome literals in render paths; this literal sits outside the design-tokens seam the module docstring declares canonical |

---

### 2.6 Inspector
*Phase scope: P1 (read-only reflection of current selection).* **Headline:**
right-column structure + accent/field tokens enforced; content is selection-driven;
editability is deferred.

| Claim | Prototype value | Build state | Phase | Disposition | Check ref |
|---|---|---|---|---|---|
| Inspector is the right column, follows focused pane | `.col.right` `flex:0 0 296px`, `follows pane A` | `ShellLayout.right_sidebar`, `right_width = 300px` | P1 | ENFORCED | `layout_invariant_tests.rs` (right-panel cards within sidebar, no overlap) |
| Title block: ref + kind + SELECTED chip | `.insp-title` ref 16.5/600, kind mono `--tx3`, chip acc-bordered | inspector title render | P1 | ENFORCED (value) | `check_gui_design_tokens.py` (ACCENT / text tokens) |
| Cross-probe banner | `.xprobe` acc-tint bg + acc border | cross-probe reflection | P1 | HUMAN | ref image vs golden |
| Section headers collapsible (Identity/Placement/Checks) | `.sect-hd` uppercase 11/600 `--tx3` | section render | P1 | ENFORCED (value) | `check_gui_design_tokens.py` |
| Key/value grid, mono for coords | `.kv` `100px 1fr`; `.field.mono` `62.400 mm` | kv rows, mono data face | P1 | ENFORCED (type value) | §4 typography-parity gap (ramp not gated) |
| Field focus ring = accent | `.field.focus{border-color:var(--acc)}` | focus material `ACCENT` | DEF (edit) | ENFORCED (value) | `check_gui_design_tokens.py` |
| Checks list with warn count (2) | `--warn` glyph rows | check-finding reflection | P1 | ENFORCED (value) | `inspector_check_finding.rs` render; `check_gui_design_tokens.py` (STATUS_WARN) |

---

### 2.7 Dock
*Phase scope: P1 read-only terminal lane; journal echo deferred.* **Headline:**
**collapsed-height conflict** (44 build vs 32 prototype, §5b) and **journal
CLI-string handoffs** vs decision 019 (§5c).

| Claim | Prototype value | Build state | Phase | Disposition | Check ref |
|---|---|---|---|---|---|
| Dock is a strip above the status bar, full width | `.dock` `flex:0 0 32px` | `ShellLayout.bottom_strip` | P1 | ENFORCED | `layout_invariant_tests.rs` |
| **Collapsed dock height** | `32px` | **44px** (`clamp(44.0, …)`, `lib.rs:163`) | P1 | ENFORCED-to-44 → **reconcile** | `lib.rs:7285 shell_layout_reserves_bottom_dock_and_viewport` asserts `bottom_strip.height == 44.0`; §5(b): accept 44 as a Datum token or set 32, then update the test |
| Tabs: bash(on) · claude(rundot) · codex · + | `.dtab`, `.rundot` = `--ok` | `bottom_dock.rs::render_bottom_tabs` | P1 | ENFORCED (value) / HUMAN (look) | `check_gui_design_tokens.py` (STATUS_SUCCESS rundot); look vs golden |
| Active tab: surface-01 + top accent rule | `.dtab.on{background:var(--s01);box-shadow:inset 0 1px 0 var(--acc)}` | active-tab material | P1 | ENFORCED (value) | `check_gui_design_tokens.py` |
| Terminal is a real shell GUI never writes to (decision 005) | mockup only | PTY lane, `render_terminal_lane` | P1 | ENFORCED | `terminal_dock_contract_tests.rs` |
| Journal command handoffs | n/a | `render_terminal_journal_commands` emits CLI strings | DEF | HUMAN (owner) | §5(c): reconcile vs decision 019 "do not synthesize CLI strings" |

---

### 2.8 Status-bar
*Phase scope: P1 (reflects engine/selection truth).* **Headline:** the DRC cell
**correctly** uses chrome `STATUS_WARN`, not content `DRC_WARN` — this pre-answers
item 4 for this region.

| Claim | Prototype value | Build state | Phase | Disposition | Check ref |
|---|---|---|---|---|---|
| Status bar is the bottom-most strip, full width | `.status` `flex:0 0 26px`, `--s01` bg | `ShellLayout.status_bar`, `height = SP_06 + SP_01 = 26px`, `SURFACE_01` | P1 | ENFORCED | `layout_invariant_tests.rs`; `check_gui_design_tokens.py` (SURFACE_01) |
| Mono segments, hairline separators | `.seg` mono 12px, `border-right:var(--bd-sub)` | mono segs, `BORDER_SUBTLE` | P1 | ENFORCED (value) | `check_gui_design_tokens.py` |
| Focus/selection segments (accent) | `.accc{color:var(--acc)}` | accent segs | P1 | ENFORCED (value) | `check_gui_design_tokens.py` (ACCENT) |
| **DRC cell uses chrome STATUS_WARN, not content DRC_WARN** | `.warnc{color:var(--warn)}` = `#E0A23A` (chrome warn) | status DRC cell → `STATUS_WARN` | P1 | ENFORCED | `check_gui_design_tokens.py` (STATUS_WARN value); **pre-answers item 4 here** — content `DRC_WARN #FFB02E` is *not* used for chrome |
| Right rev + version caption | `rev a3f19c` / "Datum EDA — split view…" | rev + caption run | P1 | HUMAN | ref image vs golden |

---

### 2.9 Typography-tokens
*Phase scope: P1 (all rendered chrome text).* **Headline:** color tokens are gated;
**the type ramp, spacing, and radius modules are hand-mirrored with NO gate**
(§4). Two live defects: menu advance (item 3) and an **unreachable Medium weight
tier**.

| Claim | Prototype value | Build state | Phase | Disposition | Check ref |
|---|---|---|---|---|---|
| UI sans = IBM Plex Sans (Condensed cut bundled) | prototype `--sans` is a **placeholder** system stack (NOT a target) | `TextFace::Ui` → "IBM Plex Sans Condensed" (`text_attrs`, `lib.rs:6226`) | P1 | ENFORCED | Design Book §2.4 is authoritative; font load `load_datum_fonts`; prototype stack explicitly not conformance-relevant (§0) |
| Data mono is OPEN (owner) | `--mono` placeholder stack | `TextFace::Mono` → "IBM Plex Mono" (bundled + used) | P1 | HUMAN (owner) | §5(d): mono marked OPEN in Design Book §2.4/§8; owner decision |
| Type ramp values (display/header/body/…/micro) | prototype px are placeholders | `design_tokens::typography` mirrors §2.4 | P1 | **TO-ENFORCE** | `check_gui_design_tokens.py` parses **only colors**; add a typography-ramp parity arm (§4) — ramp is hand-mirrored, ungated |
| Spacing (Carbon 2/4/8) + radius mirror the book | `--row` etc. placeholders | `spacing::*`, `radius::*` | P1 | **TO-ENFORCE** | add spacing/radius parity arms to `check_gui_design_tokens.py` (§4) |
| **Menu advance factor** (item 3) | Plex Condensed narrower than 0.78 | flat `0.78` Ui advance, `lib.rs:6180` | P1 | **TO-ENFORCE** | see §2.1 item-3 row (measured advance + two-sided menu test) |
| **Weight tiers reachable** (type.strong / type.micro = Medium 500) | book §2.4 assigns Medium 500 | `IBMPlexSansCondensed-Medium.ttf` **loaded but `text_attrs` never selects `Weight::MEDIUM`** → tiers unreachable | P1 | **TO-ENFORCE** | add a tier→weight resolution test asserting strong/micro resolve to `Weight::MEDIUM`; fix `text_attrs` to select it |
| **DRC_ERROR / DRC_WARN / SILK_BOTTOM saturation** (item 4) | `--err/--warn` chrome; content silk single value | `DRC_ERROR #FF4D4D`, `DRC_WARN #FFB02E`, `SILK_BOTTOM #969BA1` ≠ `SILK_TOP #E8E6DC` | P1 | HUMAN (owner-eye) — **do NOT change values** | values stay ENFORCED-as-mirrored by `check_gui_design_tokens.py` so any future change is deliberate; saturation/dimness judgment is owner-eye vs the reference render (§5d) |

---

## 3. The four polish items — where they land (index)

Folded in as first-class rows above, not a sidebar:

1. **Golden fit-to-board framing** → §2.5 board-pane, **TO-ENFORCE**
   (`golden_camera_frames_board_fit_to_bounds` in `visual_runner.rs`; not a pixel-diff).
2. **Overlay-text overflow** → §2.5 board-pane, **TO-ENFORCE** (extend
   `layout_invariant_tests.rs`; fix via `draw_text_clipped`/`truncate_text`).
3. **Menu-width tuning** → §2.1 menu-bar + §2.9 typography, **TO-ENFORCE** (replace
   `0.78` at `lib.rs:6178`; two-sided menu metric test in `menu_chrome.rs`).
4. **DRC / silk-bottom token softening** → §2.9 typography + §2.8 status-bar,
   **HUMAN / owner-eye** (values unchanged, stay mirror-gated; status-bar already
   uses chrome `STATUS_WARN`, pre-answering item 4 there).

## 4. Machine-layer gap register (checks this spec asks to ADD)

A build/test agent's to-do list, distinct from the prose. Each lands **with the
slice it verifies**, never red against un-built structure.

- **G1 — Typography / spacing / radius parity arm.** `check_gui_design_tokens.py`
  parses **only** color tokens today; the `typography` (§2.4), `spacing` (§2.5) and
  `radius` (§2.6) modules of `design_tokens.rs` are hand-mirrored with **no gate**.
  Add non-color parity arms covering those tables ↔ `design_tokens.rs`.
- **G2 — Prototype-parity arm.** Parse `board-editor.html` `:root` and assert its
  declared vars equal the Design Book / `design_tokens.rs` values (**subset** — the
  prototype legitimately omits `drc.*`, `silk.bottom`, `paste`, `exclusion`).
  Nothing gates prototype drift today. (Still not a pixel check — value parity only.)
- **G3 — Region token-binding contract tests** in `render_contract_tests.rs`:
  assert **which token a region actually uses** — menu-bar / status-bar / dock fill
  `BG_BASE` vs `SURFACE_01/02`; focus/selection value = accent; layer swatch → its
  content-family token. Token **values** are gated; token **usage** is not.
- **G4 — Ad-hoc color lint.** Fence raw `[f32;3]` chrome literals in render paths
  (e.g. board-field border `[0.46,0.49,0.53]` at `lib.rs:1275`) that sit outside the
  `design_tokens` seam the module docstring declares canonical.
- **G5 — Weight-tier resolution test.** `type.strong` / `type.micro` (Medium 500)
  are unreachable: `IBMPlexSansCondensed-Medium.ttf` is loaded but `text_attrs`
  never selects `Weight::MEDIUM`. Add a tier→weight test and wire the selection.
- **G6 — Golden fit-to-board framing** (item 1): `visual_runner.rs`
  `golden_camera_frames_board_fit_to_bounds`.
- **G7 — Overlay-within-viewport** (item 2): extend `layout_invariant_tests.rs`.
- **G8 — Two-sided menu-layout metric** (item 3): `menu_chrome.rs`, over a measured
  Plex-Condensed advance.

## 5. Open-reconciliation register (owner decisions recorded, not resolved)

This spec **records** these; it does not resolve them. Structural checks that depend
on them are non-authoritative until the owner decides.

- **(a) Tool-rail region existence.** Prototype puts tools **per-pane**
  (`.pane-tools`); `DATUM_GUI_PHASE_1_SPEC.md` L40/L54 name a **"left tool rail"**
  and `ShellLayout` reserves one (`tool_rail`, 48px). The conflict must be resolved
  before §2.2 structural checks can be authoritative.
- **(b) Dock collapsed height 44 vs 32.** Currently ENFORCED to 44 by
  `shell_layout_reserves_bottom_dock_and_viewport` (`lib.rs:7285`). Reconcile —
  accept 44 as a Datum token or set 32 — then update that test.
- **(c) Journal CLI-string handoffs.** The dock's `render_terminal_journal_commands`
  emits CLI strings; reconcile against decision 019 ("do not synthesize CLI
  strings").
- **(d) Route menu ctx-muting + font choice.** The `Route` menu context-muting
  policy, and the data-mono choice (build ships IBM Plex Mono while the Design Book
  §2.4/§8 marks mono **OPEN**) plus the Condensed-sans UI cut — owner calls.

## 6. Coverage validation

- All **9** regions present (§2.1–§2.9).
- Every line item carries **exactly one** of ENFORCED / TO-ENFORCE / HUMAN (rows
  marked "ENFORCED (value) / HUMAN (look)" split a *value* claim from a distinct
  *appearance* claim — each half carries one disposition).
- **No row pixel-diffs wgpu against `board-editor.html`.**
- Every **TO-ENFORCE** names a concrete file + assertion (§4 G1–G8; §2 rows).
- Every **HUMAN** names the reference image (`board-editor.html`) + the committed
  golden (`crates/gui-render/testdata/golden/board/datum-test.scale-*.golden.png`).
