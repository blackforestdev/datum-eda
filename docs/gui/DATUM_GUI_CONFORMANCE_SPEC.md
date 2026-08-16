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

### 0.1 The composed-shell visual-parity gate (fixes the paperwork defect)

Historically the *composed shell look* was every per-row **HUMAN** disposition
in §2 and nothing else — reviewed against a reference image that existed only as
`docs/gui/reference/board-editor.png.PENDING`, with `check_gui_conformance.py`
merely *reporting* HUMAN rows and never failing on them. That gate enforced
paperwork, not the visual outcome.

That paperwork defect is closed by a **real, same-engine, FAILING gate over a
PHASE-2 SPLIT-VIEW FIRST-SLICE INTERIM target**: `scripts/check_gui_visual_parity.py`
captures the running app at a canonical command (`--board <datum-test path>
--select R1 --visual-test --window-size 1680x1050 --exit-after-screenshot` — the
datum-test board with a preset R1 component selection, a populated two-pane
composition: Board pane A + inspector, Schematic pane B a labeled placeholder) and
diffs it against a committed **shell golden**
`crates/gui-render/testdata/golden/shell/datum-shell.golden.png` with a small
tolerance (build-vs-build, one renderer — **never** a wgpu-vs-HTML pixel-diff).
The gate then **fails** on any regression from that frozen look, and is wired into
`check_gui_conformance.py` (hence `run_drift_gates.sh`). Re-capture is an explicit
owner action: `check_gui_visual_parity.py --bless`.

**Honest scope — do not overstate what this golden is.** This shell golden is a
Phase-2 **split-view first-slice interim** target: it now captures the real
two-pane split LAYOUT (Board pane A | Schematic pane B) with per-pane headers and
focus, but **pane B is a "Schematic (coming)" placeholder** (no schematic content,
authoring, or cross-probe) and it is **NOT owner-approved against
`board-editor.html`**. The prototype's Schematic pane shows **real schematic
content with a populated inspector and cross-probe**; that full composition
**cannot** be captured until the later Phase-2 slices (real schematic scene in
pane B, focus-switch, cross-probe) are built (there is no config shortcut —
`DATUM_GUI_PHASE_2_SPEC.md` P2.1). What this gate buys today is strictly *no silent
regression of the current first-slice shell look* — it does **not** certify
prototype parity. The one-time owner cross-engine approval of the full
board-editor composition is tracked separately by the reference-capture loop
(`docs/gui/reference/README.md` + `scripts/check_gui_reference_capture.py`), which
is **EXPECTED RED** until the full split composition is buildable and an
owner-approved `board-editor.png` reference exists.

**Reading the §2 HUMAN rows now:** each still names the *aesthetic/IA judgment*
(the one-time owner call that a region matches the prototype). The composed
first-slice result is regression-gated by §0.1 so the current look does not
silently drift; the *full split-view* composed judgment against the prototype
(with real schematic content) is later Phase-2 work and is not yet enforced or
approved.

## 1. Disposition legend (the discipline of this pilot)

Three dispositions, and only three. **Every row names a link** — a disposition
without a concrete reference is not a valid row.

| Disposition | Meaning | Link requirement |
|---|---|---|
| **ENFORCED** | A real existing gate / test / golden verifies this claim **today**. | Must name the file + gate/test function that verifies it now. |
| **TO-ENFORCE** | Machine-checkable, but the gate does **not** exist yet. | Must name the exact test/gate **file + the assertion to add**. **RULE:** never mark a TO-ENFORCE green, and never land it **red against un-built structure** — it lands *with the build slice it verifies*. |
| **HUMAN** | Not machine-verifiable (cross-engine pixel fidelity, aesthetic / IA / owner-eye / token-value judgment). | Must name the **reference image** + the committed **golden** it is reviewed against. |

No row proposes pixel-diffing wgpu against HTML. No row is uncomputable-yet-marked-green.

**Golden-backed-HUMAN rule (closes the paperwork hole).** A **HUMAN** disposition
for a *whole-surface* visual outcome (the composed look of the shell, or of any
whole surface) may **NOT** be the sole guardian of that outcome. It MUST be backed
by a **committed golden** that a machine gate regression-checks
build-vs-build (§2.10 / G9). For the current shell that golden is a **split-view
first-slice interim** target frozen against regression — **not** an owner-approval
of prototype parity (the full cross-engine judgment against `board-editor.html`,
with real schematic content in pane B, is later Phase-2 work, tracked by the
reference-capture loop, currently RED). The
ENFORCED half is "no regression from the committed golden." A `*.PENDING` placeholder can
**never** satisfy this — an absent or PENDING-shadowed golden FAILS the gate. This
is the exact defect being corrected: previously the composed shell look was every
per-row HUMAN disposition and nothing else, reviewed against an image that existed
only as `docs/gui/reference/board-editor.png.PENDING`, with the aggregate merely
*reporting* HUMAN rows and never failing. (Per-region HUMAN rows for a *sub-region's*
feel remain valid on their own; this rule binds only whole-surface outcomes.)

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
title **width tuning** (item 3) — enforced by the condensed advance metric test.

| Claim | Prototype value | Build state | Phase | Disposition | Check ref |
|---|---|---|---|---|---|
| Menu bar is the top strip, full width, single row | `.menubar` `height:33px` `flex:0 0 33px` | `ShellLayout.top_menu_bar`, `height = SP_07 + SP_01 = 34px` | P1 | ENFORCED | `layout_invariant_tests.rs::shell_and_hit_regions_hold_layout_invariants_across_scale_matrix` (PG-UI-LAYOUT-INVARIANTS) |
| Bar fill = shell background | `background:var(--bg)` `#121318` | fill `design_tokens::chrome::BG_BASE` | P1 | ENFORCED | `scripts/check_gui_design_tokens.py` (value); region *usage* → §4 gap |
| Bottom hairline divider | `border-bottom:1px var(--bd-sub)` | `BORDER_SUBTLE` | P1 | ENFORCED | `check_gui_design_tokens.py` (value) |
| Titles: File Edit View Place Route Project Checks Manufacturing Window Help | 10 literal `.menu` spans | rendered **from `menu_model.json`** in `menu_chrome.rs::render_menu_bar` | P1 | ENFORCED | `menu_chrome.rs::menu_model_is_available_to_renderer` |
| Adjacent titles never overlap (incl. "Manufacturing") | flow layout, `padding:3px 9px` | `menu_title_width` via `estimated_text_run_width_px` | P1 | ENFORCED | `menu_chrome.rs` menu-layout test (no-overlap side) |
| **Title width not over-spaced** (item 3) | Plex Condensed is narrower than 0.78 advance | `TextFace::Ui` uses condensed advance; retired 0.78 path guarded | P1 | ENFORCED | `menu_chrome.rs::conformance_menu_title_width_uses_condensed_measured_advance` |
| Wordmark `Datum·EDA`, accent middot | `.brand` 600/14px, `b{color:var(--acc)}` | brand run, `ACCENT` middot | P1 | HUMAN | ref `board-editor.html` vs golden `datum-test.scale-1_00.golden.png` |
| `Route` menu context-muted | `.menu.ctx{color:var(--tx3)}` | ctx-muting policy open | P1 | HUMAN (owner) | §5(d) reconciliation |
| Right-side rev pill `stm32-sensor-node · rev a3f19c` | `.pill` mono 12px `--tx3` | pill run, `TEXT_MUTED` mono | P1 | HUMAN | ref image vs golden |

---

### 2.2 Tool-rail
*Phase scope: P1 chrome region exists in the pane header.* **Headline:** the
region-existence conflict is resolved: the prototype has **no** standalone left
tool rail; tools live per-pane in `.pane-tools`, and the build now follows that.

| Claim | Prototype value | Build state | Phase | Disposition | Check ref |
|---|---|---|---|---|---|
| Standalone left tool rail is absent | **absent** — tools are per-pane (`.pane-tools` inside `.pane-hd`) | `ShellLayout` has no `tool_rail` column | P1 | ENFORCED | `layout_invariant_tests.rs::shell_and_hit_regions_hold_layout_invariants_across_scale_matrix` |
| Per-pane tool cluster (Select/Move/Route/Via/Zone) | `.pane-tools` `.ptool` 25px, active = `--acc` outline + `--acc-tint` | board-pane header renders read-only S/M/R/V/Z tools | P1 | ENFORCED | `render_contract_tests.rs::conformance_pane_header_tools_and_binding_chips_render` |

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
| `pane-tag` / `follows pane` binding chips | `.pane-tag` acc-bordered; `Layers follows pane A` | Layers panel renders `FOLLOWS PANE A` binding chip | P1 | ENFORCED | `render_contract_tests.rs::conformance_pane_header_tools_and_binding_chips_render` |

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
| **Which swatch binds which content token** (usage) | swatch `style="background:var(--cu-f)"` etc. | layer→token binding in render path | P1 | ENFORCED | `render_contract_tests.rs::conformance_region_token_bindings_follow_design_book` |
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
| Selection = slight whole-owned-object luminance lift + accent internal glow, with material/semantic hue retained (cross-probed U1) | `stroke:var(--acc)` + `#glow` filter is the visual target, interpreted through Rendering Book §2.1 | immediate object-shaped selection overlay (retained-world recolor is migration debt) | S5 | ENFORCED (accent value) / TO-ENFORCE (overlay retention) / HUMAN (look) | value: `check_gui_design_tokens.py`; retention: ledger §8.1 V1–V5; look: `selection-study.png` panel 1 (§8.2) |
| Active/inactive resolving panes show identical full selection; only pane frame/focus header/tool availability grants GUI mutation authority | schematic/board cross-projection uses the same glow while `.pane.focus` frames pane A | shared selection projected through each pane camera; focus chrome remains pane-only | S5 | TO-ENFORCE / HUMAN | ledger §8.1 I2; `selection-study.png` panel 5 (§8.2) |
| Global Net is one semantic subject whose complete visible electrical projection glows across schematic/PCB; connected parent bodies remain related | selected `SPI2_SCK` wire/label and cross-probed PCB copper | typed global-net projection over visible wire/label/port/pin-terminal/track/via/pad/zone/airwire roles | S5 | TO-ENFORCE / HUMAN | ledger §8.1 I4 + L5 + O4; `selection-study.png` panel 6 (§8.2) |
| Related objects keep exact authored appearance; explicit relationship view may dim only unrelated context | related parent remains baseline; no second accent vocabulary | retire via-coloured `AUTHOR_RELATED`; governed legible unrelated-dim channel + Inspector relationship explanation | S5 | TO-ENFORCE / HUMAN | ledger §8.1 I3; `selection-study.png` panel 6 (§8.2) |
| Optional compound focus member has no stronger/different persistent canvas visual | n/a (compound deferred in prototype) | equal member selection overlays; focus only in Inspector/session; command-owned temporary reference marker | S5 | TO-ENFORCE / HUMAN | ledger §8.1 L6 + O3; `selection-study.png` panel 2 (§8.2) |
| Locked selection = slight neutral greying + normal selection + no handles + governed anchor padlock/cursor | not yet in prototype | typed lock channel; padlock blocked on icon_set declaration/contact-sheet entry/HUMAN review; dense compounds use Inspector count | S5 | TO-ENFORCE / HUMAN | ledger §8.1 R1/R3 + B3; icon gate CLOSED (declared+sheet+approved); `selection-study.png` panel 3 (§8.2) |
| Authored/proposal/selection/diagnostic collision preserves every channel; semantic marker topmost | proposal ghost and warn glyph visual vocabulary | ordered orthogonal overlays; selected proposal/diagnostic retains proposal/severity core plus selection cue | S5 | TO-ENFORCE / HUMAN | ledger §8.1 D2; `selection-study.png` panels 4+9 (§8.2) |
| Bus scope expands section → connected run → hierarchical bus; owned spine/name/entries glow, scalar member nets do not | selected bus spine/name visual | typed Bus selection projection distinct from scalar Global Net | S5 | TO-ENFORCE / HUMAN | ledger §8.1 I4 + A6; `selection-study.png` panel 6 (§8.2) |
| Text uses glyph selection without persistent bbox; junction/via/no-connect/pin preserve semantic silhouette/core with crisp tiny-object fallback | Rendering Study whole-symbol text/terminal treatment | typed glyph/path/ring overlays; text handles only in edit tool; Symbol Editor child-pin scope | S5 | TO-ENFORCE / HUMAN | ledger §8.1 A6 + V4/V5; `selection-study.png` panels 7+9 (§8.2) |
| Dense/global/panelized selection: sub-2px cue, 65,536 detail cap, then exact whole-pane union-mask fallback; never partial/bbox | n/a | deterministic visible-overlay LOD + Inspector aggregate-LOD state | S5 | TO-ENFORCE / HUMAN | ledger §8.1 D1; `selection-study.png` panel 8 (§8.2) |
| **Golden captures the board fit-to-bounds** (item 1) | mockup shows board filling the pane | `camera_for_manifest` centers scene bounds and uses fit zoom | P1 | ENFORCED | `visual_runner.rs::conformance_golden_camera_frames_board_fit_to_bounds` |
| **Overlay hint stays within the viewport** (item 2) | n/a (mockup has no overflowing hint) | hint is clipped to `layout.viewport` | P1 | ENFORCED | `layout_invariant_tests.rs::shell_and_hit_regions_hold_layout_invariants_across_scale_matrix` |
| Board-field border color | grid `#171A20`, edge `--edge` | board-field border uses `design_tokens::content::EDGE`; retired raw literal linted | P1 | ENFORCED | `scripts/check_gui_design_tokens.py` ad-hoc chrome literal guard |

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
**collapsed-height conflict** and **journal CLI-string handoffs** are reconciled.

| Claim | Prototype value | Build state | Phase | Disposition | Check ref |
|---|---|---|---|---|---|
| Dock is a strip above the status bar, full width | `.dock` `flex:0 0 32px` | `ShellLayout.bottom_strip` | P1 | ENFORCED | `layout_invariant_tests.rs` |
| **Collapsed dock height** | `32px` | `ShellLayout` collapsed/default dock height = `SP_07` / 32px | P1 | ENFORCED | `lib.rs::tests::shell_layout_reserves_bottom_dock_and_viewport` |
| Tabs: bash(on) · claude(rundot) · codex · + | `.dtab`, `.rundot` = `--ok` | `bottom_dock.rs::render_bottom_tabs` | P1 | ENFORCED (value) / HUMAN (look) | `check_gui_design_tokens.py` (STATUS_SUCCESS rundot); look vs golden |
| Active tab: surface-01 + top accent rule | `.dtab.on{background:var(--s01);box-shadow:inset 0 1px 0 var(--acc)}` | active-tab material | P1 | ENFORCED (value) | `check_gui_design_tokens.py` |
| Terminal is a real shell GUI never writes to (decision 005) | mockup only | PTY lane, `render_terminal_lane` | P1 | ENFORCED | `terminal_dock_contract_tests.rs` |
| Journal command handoffs | n/a | journal commands are not projected into terminal handoffs | P1 | ENFORCED | `terminal_dock_contract_tests.rs`; `terminal_command_catalog.rs` filters `datum.journal.*` |

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
| **DRC cell uses chrome STATUS_WARN** | `.warnc{color:var(--warn)}` = `#E0A23A` (chrome warn) | status DRC cell → `STATUS_WARN`; content DRC warn aliases to the same muted token | P1 | ENFORCED | `check_gui_design_tokens.py` (STATUS_WARN / DRC_WARN value parity) |
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
| Data mono = IBM Plex Mono | `--mono` placeholder stack | `TextFace::Mono` → "IBM Plex Mono" (bundled + used) | P1 | ENFORCED | `VISUAL_LANGUAGE.md` §2.4 + `check_gui_design_tokens.py` type-family parity |
| Native terminal cell exception = JetBrains Mono Regular | prototype terminal stack is non-authoritative | `TextFace::Terminal` → bundled "JetBrains Mono" for complete terminal cell runs; chrome remains `TextFace::Mono` | T1 | ENFORCED | Decision 031; deterministic box-drawing/Powerline shaping and terminal-run render contracts |
| Type ramp values (display/header/body/…/micro) | prototype px are placeholders | `design_tokens::typography` mirrors §2.4 | P1 | ENFORCED | `check_gui_design_tokens.py` typography-ramp parity |
| Spacing (Carbon 2/4/8) + radius mirror the book | `--row` etc. placeholders | `spacing::*`, `radius::*` | P1 | ENFORCED | `check_gui_design_tokens.py` spacing/radius parity |
| **Menu advance factor** (item 3) | Plex Condensed narrower than 0.78 | condensed UI advance guarded against retired 0.78 path | P1 | ENFORCED | `menu_chrome.rs::conformance_menu_title_width_uses_condensed_measured_advance` |
| **Weight tiers reachable** (type.strong / type.micro = Medium 500) | book §2.4 assigns Medium 500 | `TextFace::UiMedium` selects `Weight::MEDIUM` | P1 | ENFORCED | `lib.rs::tests::conformance_medium_type_tiers_resolve_to_medium_weight` |
| **DRC marker colors** (item 4) — **already specced/approved in the prototype** | prototype renders ALL checks/DRC in the muted `--err #E5534B` / `--warn #E0A23A` (CHECKS panel count, `SPI2_SCK ⚠`, status `.warnc`) | `content.drc.*` aliases to muted `STATUS_ERROR/STATUS_WARN` | P1 | ENFORCED | `check_gui_design_tokens.py`; `render_contract_tests.rs::conformance_region_token_bindings_follow_design_book` |
| **Back-side silk (`SILK_BOTTOM`)** | prototype specs a single `--silk #E8E6DC` (front only); back silk not covered by the prototype | `SILK_BOTTOM #969BA1` (dimmer) ≠ `SILK_TOP` | P1 | RECORDED-GAP | prototype does not address back silk; the dimmer default is physically sensible (far side reads recessed) — keep it, recorded so any future change is deliberate |

---

### 2.10 Split-view first-slice shell no-regression (interim)
*Phase scope: P1/P2-first-slice (the composed look of the current split-view
first-slice shell — two panes, pane B a placeholder — not the full split-view
prototype with real schematic content, and not a single region).*
**Headline:** the current split-view first-slice shell look is frozen into a
committed **shell golden** and regression-gated same-engine, so it does not
silently drift. The golden now captures the real two-pane split LAYOUT, but pane B
is a labeled placeholder. This is **not** prototype-parity certification: the
prototype `board-editor.html` is a **split Board+Schematic composition with a
populated inspector and real schematic content**, which cannot be captured until
the later Phase-2 slices (`DATUM_GUI_PHASE_2_SPEC.md` P2.1+) render the schematic
pane. The whole-split-shell composed judgment against the prototype is tracked by
the reference-capture loop (`scripts/check_gui_reference_capture.py`), **EXPECTED
RED** until the full split composition is buildable.

| Claim | Reference | Build state | Phase | Disposition | Check ref |
|---|---|---|---|---|---|
| No regression of the current split-view first-slice shell look (interim; NOT prototype-parity) | committed split-view first-slice shell golden | app split-view first slice (pane B placeholder) | P1 | ENFORCED | `scripts/check_gui_visual_parity.py` vs `crates/gui-render/testdata/golden/shell/datum-shell.golden.png` |
| Full board-editor.html composition parity (split Board+Schematic, populated inspector, real schematic content) | prototype `board-editor.html` | first slice built (pane B placeholder) | P2 | HUMAN (gated on later Phase-2) | reference-capture loop `scripts/check_gui_reference_capture.py` + `docs/gui/reference/README.md` (RED until captured) |

**Canonical capture (fixed; the golden was captured at exactly these parameters and
the gate re-runs them):**

```bash
cargo run -q -p datum-gui-app --bin datum-gui --features visual -- \
  --board /home/bfadmin/Documents/kicad_projects/Datum-eda/datum-test/datum-test.kicad_pcb \
  --select R1 --visual-test \
  --screenshot-out <tmp>/datum-shell.capture.png \
  --window-size 1680x1050 --exit-after-screenshot
```

Fixed inputs: launch bin `datum-gui`, the datum-test board + preset R1 selection
(a populated two-pane composition — Board pane A + inspector, Schematic pane B a
labeled placeholder — NOT the full board-editor.html composition with real
schematic content), `--visual-test` deterministic offscreen path, window
`1680x1050`. **Semantic guards** (`guard_intended_fixture`): the capture fixture
must be a real board whose layer stack includes `F.Cu` / `B.Cu` / `F.SilkS` /
`Edge.Cuts` and must NOT be the synthetic `Datum GUI Known Good` demo scene, so a
wrong scene cannot be blessed. The split LAYOUT now renders (two pane headers),
but the guard is input-side (pre-capture) and the **U1 / STM32 schematic-content
guards remain deferred** — pane B is still a placeholder. **Tolerances
(same-engine, so near-identical — verified at 0 differing px on the reference
workstation):** dimensions must match exactly (hard fail otherwise); then (a) the
fraction of pixels differing by more than **8/255** on any channel must be **< 0.5%**,
AND (b) the **mean per-channel difference** must be **< 1.5/255**. Any real chrome
regression (a moved panel, a re-introduced uppercase run, a resurrected ON/OFF
badge, the floating-card border) blows past this and FAILS. Capture-infra failure
(no GPU/offscreen path) FAILS LOUD — the gate never silently passes. Re-approval is
an explicit owner action: `scripts/check_gui_visual_parity.py --bless`. This is the
sanctioned **build-vs-build** golden of §0's Machine layer, extended from the board
sub-region to the whole shell — **not** the forbidden wgpu-vs-HTML cross-engine diff.

---

## 3. The four polish items — where they land (index)

Folded in as first-class rows above, not a sidebar:

1. **Golden fit-to-board framing** → §2.5 board-pane, **ENFORCED**
   (`conformance_golden_camera_frames_board_fit_to_bounds` in `visual_runner.rs`; not a pixel-diff).
2. **Overlay-text overflow** → §2.5 board-pane, **ENFORCED** (`layout_invariant_tests.rs`;
   fixed via `draw_text_clipped`).
3. **Menu-width tuning** → §2.1 menu-bar + §2.9 typography, **ENFORCED**
   (condensed advance + two-sided menu metric test in `menu_chrome.rs`).
4. **DRC marker colors** → §2.9 typography, **ENFORCED** — build fix, not an owner
   decision: DRC is **already specced/approved in the prototype** as the muted
   `--err/--warn`; the build's separate hot `DRC_*` tokens diverge and must be
   reconciled to those values. Back-side silk is a **recorded prototype gap** (dimmer
   default kept). (Status-bar already uses chrome `STATUS_WARN` correctly.)

## 4. Machine-layer gap register (checks added)

A build/test agent's to-do list, distinct from the prose. Each lands **with the
slice it verifies**, never red against un-built structure.

- **G1 — Typography / spacing / radius parity arm.** ENFORCED by
  `scripts/check_gui_design_tokens.py`.
- **G2 — Prototype-parity arm.** ENFORCED by `scripts/check_gui_design_tokens.py`
  parsing `board-editor.html` `:root` for the approved subset. Still not a pixel
  check — value parity only.
- **G3 — Region token-binding contract tests.** ENFORCED by
  `render_contract_tests.rs::conformance_region_token_bindings_follow_design_book`.
- **G4 — Ad-hoc color lint.** ENFORCED by `scripts/check_gui_design_tokens.py`
  against the retired chrome literals, including the old board-field border.
- **G5 — Weight-tier resolution test.** ENFORCED by
  `lib.rs::tests::conformance_medium_type_tiers_resolve_to_medium_weight`.
- **G6 — Golden fit-to-board framing.** ENFORCED by
  `visual_runner.rs::conformance_golden_camera_frames_board_fit_to_bounds`.
- **G7 — Overlay-within-viewport.** ENFORCED by `layout_invariant_tests.rs`.
- **G8 — Two-sided menu-layout metric.** ENFORCED by
  `menu_chrome.rs::conformance_menu_title_width_uses_condensed_measured_advance`.
- **G9 — Split-view first-slice shell no-regression gate (interim).** ENFORCED by
  `scripts/check_gui_visual_parity.py` (committed split-view first-slice shell
  golden `crates/gui-render/testdata/golden/shell/datum-shell.golden.png`; captures
  the running app at the §2.10 canonical command and fails on any regression from
  the frozen first-slice look, fails if the golden is absent or a `*.PENDING`
  placeholder shadows it, applies the `guard_intended_fixture` semantic guards, and
  fails loud on capture-infra failure — never silently passes). Wired into
  `check_gui_conformance.py` (hence `run_drift_gates.sh`). This gate is **interim
  first-slice no-regression, NOT prototype-parity** — the golden captures the real
  two-pane split LAYOUT but pane B is a placeholder; the full board-editor.html
  composition (real schematic content, cross-probe) is later Phase-2 and its owner
  approval is tracked by G10.
- **G10 — Reference-capture gate (EXPECTED RED until the full split is buildable).**
  ENFORCED by `scripts/check_gui_reference_capture.py` (wired into
  `run_drift_gates.sh`): FAILS while `docs/gui/reference/board-editor.png` is
  missing or a `*.PENDING` placeholder exists. This red is the **honest signal**
  that no owner-approved reference of the full board-editor.html split composition
  has been captured yet; it is not a bug and must not be silenced with a fabricated
  image. It resolves when the owner captures the reference (per
  `docs/gui/reference/README.md` §2) — which becomes meaningful once the later
  Phase-2 slices render real schematic content in pane B.

## 5. Reconciliation register (resolved in this conformance pass)

This spec **records** these; it does not resolve them. Structural checks that depend
on them are non-authoritative until the owner decides.

- **(a) Tool-rail region existence.** Resolved to prototype parity: tools live
  per-pane; `ShellLayout` has no standalone `tool_rail`.
- **(b) Dock collapsed height.** Resolved to 32px / `SP_07`; layout and
  app resize clamp both use the prototype target.
- **(c) Journal CLI-string handoffs.** Resolved: journal commands are not rendered
  or projected as terminal handoffs. Future execution belongs to the GUI write
  path.
- **(d) Route menu ctx-muting + font choice.** Data mono is ratified as IBM Plex
  Mono. Route ctx-muting remains HUMAN/owner-eye because it is visual policy, not
  a machine parity gap.

## 6. Coverage validation

- All **9** regions present (§2.1–§2.9), plus the split-view first-slice shell
  no-regression row (§2.10, ENFORCED via G9) that regression-gates the current
  *split-view first-slice* composed look (two panes, pane B a placeholder), and the
  full-split-composition row (§2.10, HUMAN, gated on later Phase-2 via G10 —
  currently RED, no owner-approved reference captured).
- Every line item carries **exactly one** of ENFORCED / TO-ENFORCE / HUMAN (rows
  marked "ENFORCED (value) / HUMAN (look)" split a *value* claim from a distinct
  *appearance* claim — each half carries one disposition).
- **No row pixel-diffs wgpu against `board-editor.html`.**
- Every **TO-ENFORCE** names a concrete file + assertion (§4 G1–G9; §2 rows).
- Every **HUMAN** names the reference image (`board-editor.html`) + the committed
  golden (`crates/gui-render/testdata/golden/board/datum-test.scale-*.golden.png`).
- The **current split-view first-slice shell look** (the aggregate of the §2 HUMAN
  rows on the first-slice build) is regression-gated by §0.1 — the same-engine
  shell visual-parity gate (`scripts/check_gui_visual_parity.py` vs
  `crates/gui-render/testdata/golden/shell/datum-shell.golden.png`), wired into
  `check_gui_conformance.py`. The owner captures the interim golden; the gate
  fails on drift. This is first-slice no-regression, **not** prototype parity —
  the golden captures the real two-pane split LAYOUT but pane B is a placeholder;
  the **full board-editor.html composition** (real schematic content, cross-probe)
  is later Phase-2 and its owner-approved reference is tracked by G10
  (`scripts/check_gui_reference_capture.py`), correctly RED until captured. Visual
  parity of the current shell is no longer report-only paperwork; parity of the
  full prototype is honestly marked as not-yet-built.

## 7. Pilot status — the discipline generalizes (future doc-by-doc pass)

This document is the **pilot application** of a governance discipline, not a
one-off. The discipline is:

> **Every claim in a GUI spec must carry exactly one honest check disposition —
> ENFORCED (a named existing gate/test/golden), TO-ENFORCE (a named machine check
> to add with the slice it verifies, never faked green or landed red against
> un-built structure), or HUMAN (reviewed against a committed reference image /
> golden; never a pixel-diff of wgpu against HTML).** No claim is left as
> unverifiable prose.

The board editor was chosen as the pilot because it is the first buildable GUI
surface (Active-Frontier step 2) and already has a controlling prototype, a token
mirror, and a golden harness — so all three dispositions are exercisable today.
The intended trajectory is a **doc-by-doc pass** that applies this same discipline
to the other GUI specs as each surface reaches buildable definition — e.g. the
marking-menu shell (`DATUM_GUI_CONTEXT_MENU_CONTENT.md`, Active-Frontier step 3),
the command console (step 4), and the schematic/library surfaces (step 6) — each
turning its own prototype/claims into an actionable, honestly-dispositioned
conformance rail. This spec is the **template and precedent** for that pass; it does
not itself perform it. Sequencing for the rollout lives in the Active Frontier
(`specs/PROGRESS.md`), which threads each surface's conformance rail behind the
build slice it governs.

## 8. S5 disposition ledger (S5-C10)

<!-- EVIDENCE:UVT-S5-SPEC:S5-C10-DISPOSITION -->

Every S5 claim closes here with exactly one honest disposition. Machine
claims are **TO-ENFORCE**: each names its exact future test home and
assertion, landing with the S5A build slice; a test-home relocation requires
updating this ledger in the same change. HUMAN claims are **CLOSED for the
specification phase**: each cites the committed reference
`docs/gui/reference/selection-study.png` (owner-approved 2026-08-14, review
record `DATUM_UNIVERSAL_VIEWPORT_TOOLING_SPEC.md` §2.2.21) by panel number,
with re-review at S5A landing against the build's own goldens (never
cross-engine pixel-diff). The `selection-lock-anchor` icon gate is closed:
declared in `icon_set.json`, on the contact sheet, design owner-approved
(asset `to_author`, lands with S5A).

### 8.1 Machine assertions (TO-ENFORCE, land with S5A)

| ID | Assertion (governing §) | Test home |
|---|---|---|
| A1 | Region membership deterministic per (revision, final region) — UVT §2.2.17 | `crates/gui-viewport/src/region.rs::region_membership_deterministic` |
| A2 | Region enumeration AABB-pruned, never O(total) — §2.2.17 | `crates/gui-viewport/src/region.rs::region_query_prunes_by_aabb` |
| A3 | Rect+lasso membership equals brute-force oracle (100k fixture) — §2.2.17 | `crates/gui-viewport/src/region.rs::region_membership_matches_oracle` |
| A4 | Auto-pan gesture ≡ directly-specified equal region — §2.2.17 | `crates/gui-viewport/src/region.rs::region_autopan_membership_equivalence` |
| A5 | Commit atomic under budget pressure; cancel preserves prior — §2.2.17 | `crates/gui-viewport/src/region.rs::region_commit_atomic_under_budget` |
| A6 | Per-class qualification cases incl. exactly-50% rejection — §2.2.4/§2.2.16 | `crates/gui-viewport/src/region.rs::region_class_qualification_cases` |
| A7 | Regions never acquire non-authored subjects — OPEN-12 | `crates/gui-viewport/src/region.rs::region_excludes_non_authored` |
| L1 | Re-resolution on every model_revision — §2.2.18 | `crates/gui-viewport/src/selection_lifetime.rs::lifetime_reresolution_on_revision` |
| L2 | No phantom members post-revision — §2.2.18 | `crates/gui-viewport/src/selection_lifetime.rs::lifetime_no_phantom_members` |
| L3 | Identity-preserving edits keep selection — §2.2.18 | `crates/gui-viewport/src/selection_lifetime.rs::lifetime_edit_preserves_selection` |
| L4 | No resurrection on undo; no substitution on recreate — §2.2.18 | `crates/gui-viewport/src/selection_lifetime.rs::lifetime_no_resurrection` |
| L5 | Semantic subjects re-derive wholly; dissolve-to-empty — §2.2.18 | `crates/gui-viewport/src/selection_lifetime.rs::lifetime_semantic_subject_rederivation` |
| L6 | Focus clears with no auto-promotion — §2.2.18/§2.2.10 | `crates/gui-viewport/src/selection_lifetime.rs::lifetime_focus_clears_no_promotion` |
| L7 | Pane close preserves; content swap cancels gestures only; project swap drops — §2.2.18 | `crates/gui-viewport/src/selection_lifetime.rs::lifetime_pane_document_lifecycle` |
| L8 | Stale identities drop-and-report, never fault or fuzzy-match — §2.2.18 | `crates/gui-viewport/src/selection_lifetime.rs::lifetime_stale_identity_drop` |
| L9 | Per-pane projection = per-pane resolution; Inspector totals global — §2.2.18 | `crates/gui-viewport/src/selection_lifetime.rs::lifetime_partial_pane_projection` |
| O1 | One projection structure feeds Inspector/console/envelope — §2.2.19 | `crates/gui-protocol/src/selection_projection.rs::outputs_single_projection_source` |
| O2 | Common/Mixed/Unavailable vs oracle — §2.2.19 | `crates/gui-protocol/src/selection_projection.rs::outputs_field_state_semantics` |
| O3 | Scopes are view filters; never mutate membership — §2.2.19 | `crates/gui-protocol/src/selection_projection.rs::outputs_scope_view_filter` |
| O4 | Blocker counts carry exact member/reason disclosure — §2.2.19 | `crates/gui-protocol/src/selection_projection.rs::outputs_blocker_reporting` |
| O5 | Envelope stable order, 256-id cap + truncated marker, round-trip — §2.2.19 | `crates/gui-protocol/src/context_envelope.rs::outputs_envelope_bounded_deterministic` |
| O6 | Outputs complete with no status bar — §2.2.19 | `crates/gui-protocol/src/selection_projection.rs::outputs_no_status_bar_dependency` |
| O7 | S5A outputs expose no mutation entry — §2.2.19 | `crates/gui-protocol/src/selection_projection.rs::outputs_readonly_boundary` |
| B1 | No code path from any selection surface to an Operation — §2.2.14 | `crates/gui-app/src/main_tests/boundary.rs::boundary_no_mutation_entry` |
| B2 | S5B seams render disabled-with-reason — §2.2.14 | `crates/gui-app/src/main_tests/boundary.rs::boundary_seams_disabled_with_reason` |
| B3 | No transform-handle geometry in S5A — §2.2.14 | `crates/gui-render/src/render/overlay.rs::boundary_no_handles_in_s5a` |
| R1 | Any of five blocker kinds ⇒ whole refusal, zero journal — §2.2.14 atomic law | `crates/engine/src/api/native_write/batch_guard.rs::refusal_whole_operation_no_journal` |
| R2 | Refusal report matches §2.2.19 blocker vocabulary — atomic law | `crates/engine/src/api/native_write/batch_guard.rs::refusal_report_matches_outputs` |
| R3 | Refusal never implicitly repairs selection/locks — atomic law | `crates/engine/src/api/native_write/batch_guard.rs::refusal_no_implicit_repair` |
| R4 | GUI/CLI/MCP refusal parity — atomic law | `crates/engine/src/api/native_write/batch_guard.rs::refusal_surface_parity` |
| V1 | Zero retained-resolve on any selection state change — UVT §4.4 | `crates/gui-render/src/render/overlay.rs::overlay_zero_retained_resolve_on_selection` |
| V2 | Static buffers static across select/deselect — §4.4 | `crates/gui-render/src/render/overlay.rs::overlay_static_buffers_across_selection` |
| V3 | CAM/export byte-identical under selection — §4.4 / Law 1 | `crates/gui-render/src/render/overlay.rs::overlay_cam_byte_identity` |
| V4 | Hit/region results invariant under selection — §4.4 | `crates/gui-render/src/render/overlay.rs::overlay_hit_geometry_invariant` |
| V5 | Cue 2.0 physical px across zoom sweep — §4.4 | `crates/gui-render/src/render/overlay.rs::overlay_cue_zoom_invariant` |
| I1 | Subject vocabulary one-to-one across enum/envelope/spec — §2.2.20 | `crates/gui-protocol/src/lib.rs::identity_vocabulary_roundtrip` |
| I2 | Same-identity projects full strength in all resolving panes — §2.2.20 | `crates/gui-render/src/render/scene.rs::identity_same_projection_equality` |
| I3 | Merely-related never receives selection treatment — §2.2.20 table | `crates/gui-render/src/render/scene.rs::identity_related_baseline_only` |
| I4 | Run/Global-Net/Bus derivation deterministic per (identity, revision) — §2.2.20 | `crates/gui-protocol/src/lib.rs::identity_derived_membership` |
| I5 | P2.3 cross-probe consumes the typed vocabulary (structural) — §2.2.20 | `crates/gui-app/src/main_tests/boundary.rs::identity_p23_consumes_vocabulary` |
| D1 | Dense cap/union-mask determinism, warm capacity, full membership (100k fixture) — §2.2.13/RB §2.8 | `crates/gui-render/src/render/visual_runner.rs::dense_cap_union_mask_determinism` |
| D2 | Maximal-collision dense variant preserves channel identity — OPEN-14 | `crates/gui-render/src/render/visual_runner.rs::dense_maximal_collision_channels` |

### 8.2 HUMAN claims (closed for the specification phase)

Reference: `docs/gui/reference/selection-study.png`; review record UVT §2.2.21.
Panels: 1 construction · 2 compound · 3 locked+padlock · 4 maximal collision ·
5 pane parity · 6 Global Net/Bus · 7 text/points · 8 dense fallback ·
9 accessibility/zoom. Each S5 board-pane row in §2.5 names its panel; S5A
landing re-reviews each panel against the build's own goldens.
