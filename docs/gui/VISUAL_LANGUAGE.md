# Datum Design Book

> **Status**: Controlling visual + interaction design specification.
> **Supersedes**: the prior "Visual Language" manifesto (this file is its
> resculpted, decision-closed successor — it does not run in parallel).
> **Ratified by**: `docs/decisions/PRODUCT_MECHANICS_015_UI_DESIGN_SYSTEM.md`.
> **Builds on**: the layout substrate in `PRODUCT_MECHANICS_014_UI_LAYOUT_SYSTEM`
> and `docs/contracts/UI_LAYOUT_SYSTEM_CONTRACT.md`.
> **Token values below are the initial ratified set** — derived within the
> locked aesthetic directions, contrast-checked, and adjustable when rendered as
> artboards/goldens. Hex/size literals here are the source of truth that Rust
> constants mirror; they are not to be re-invented in code.

## 0. What this book is (and is not)

This is the **design** — what Datum looks like, reads as, and how it behaves.
It is distinct from, and sits on top of, the **layout substrate** (Taffy
geometry, HiDPI, invariants). Layout decides *where* boxes go; this book decides
*what they look like and why*. A geometrically perfect, no-overlap, HiDPI-correct
UI is still undefined until this book tells it what colors, type, density,
states, and affordances to use.

Two token namespaces, governed differently:
- **Chrome tokens** — the application UI (panels, text, controls, accents).
  Design-owned; not user-themeable in v1.
- **Content tokens** — the board/schematic canvas (copper, silk, layers,
  markers). **User-themeable**, exactly as KiCad/Altium do. Never hardcode
  content color into chrome and never reuse a content token for chrome.

## 1. Principles

1. **Precise instrument, not consumer app.** Calm, technical, dense without
   cramped. Nothing decorative competes with the work.
2. **Color is reserved for meaning.** Chrome is near-monochrome; saturated color
   signals state (selection, status) or board content — never decoration.
3. **The canvas is the protagonist.** Chrome recedes; the board/schematic
   carries the visual weight and the only rich color in the room.
4. **Manual-first, keyboard-first.** Every affordance has a manual, keyboard, and
   command-palette path. Icons are accelerators, never the only route.
5. **Optional AI, shown as collaboration alongside.** Proposed/AI content is a
   de-emphasized overlay on the authored baseline, accepted/rejected in flow —
   never a chat app bolted into the shell, never gating the manual path.
6. **Encode state with more than color** (colorblind-safe, WCAG): pair hue with
   pattern, shape, weight, or dimming.
7. **Tokens, not literals.** No raw hex or magic type sizes in GUI code; every
   visual value resolves to a token in this book.

## 2. Foundations

### 2.1 Theme & elevation
Dark-first. **Elevation is encoded by lightening the surface** as it rises
(shadows are imperceptible on dark); higher surface = lighter. Contrast is
re-verified at every elevation level, not assumed from one.

### 2.2 Color — chrome tokens
Cool neutral gray (blue channel slightly above red). Surface ramp, deepest →
highest:

| Token | Hex | Use |
|---|---|---|
| `color.canvas` | `#0B0C0E` | board/schematic content stage (deepest; a distinct "stage", darker than chrome) |
| `color.bg.base` | `#121318` | application shell background |
| `color.surface.01` | `#181B21` | panels |
| `color.surface.02` | `#1F232A` | cards / raised rows on a panel |
| `color.surface.03` | `#272C35` | overlays, menus, popovers, command palette |
| `color.border.subtle` | `#2E343E` | hairline dividers, card borders |
| `color.border.strong` | `#3A414D` | active/raised borders, input outlines |

Text:

| Token | Hex | Contrast on `surface.01` | Use |
|---|---|---|---|
| `color.text.primary` | `#E4E7EB` | ~13:1 | values, primary labels |
| `color.text.secondary` | `#B2B8C3` | ~8.7:1 | labels, secondary text |
| `color.text.muted` | `#717885` | ~3.6:1 | metadata, captions (large/non-body only) |
| `color.text.onAccent` | `#141619` | — | text on a magenta fill |

Accent — **deep magenta** (the single chrome accent; reserved for
selection/focus/active, and the one chrome color allowed onto the canvas as the
selection signal):

| Token | Hex | Use |
|---|---|---|
| `color.accent` | `#CE5A92` | selection, focus ring, active tool/control (AA small, ~4.7:1 on base) |
| `color.accent.hover` | `#D86EA0` | hover on accent surfaces |
| `color.accent.pressed` | `#B84A80` | pressed/active-down |
| `color.accent.tint` | `#2A1D25` | selected-row background tint |

Status (chrome context — distinct from content DRC markers, see §2.3):

| Token | Hex |
|---|---|
| `color.status.error` | `#E5534B` |
| `color.status.warn` | `#E0A23A` |
| `color.status.success` | `#4FA75A` |
| `color.status.info` | `#5B8BD0` |

### 2.3 Color — content tokens (canvas; user-themeable)
A **separate** token set, shipped as read-only defaults plus a user-editable
theme layer (KiCad/Altium model). Defaults honor the warm-front/cool-back copper
convention. `selection` deliberately equals `color.accent` so selection reads
consistently across chrome and canvas.

| Token | Hex | Notes |
|---|---|---|
| `content.copper.front` | `#C83A34` | F.Cu (warm/red) |
| `content.copper.back` | `#4D7FC4` | B.Cu (cool/blue) |
| `content.copper.in1` | `#4FA75A` | inner 1 |
| `content.copper.in2` | `#C2A13A` | inner 2 |
| `content.silk.top` | `#E8E6DC` | |
| `content.silk.bottom` | `#969BA1` | |
| `content.mask` | `#2FA38C` | renders translucent |
| `content.paste` | `#8C9299` | |
| `content.edge` | `#CBB24A` | edge cuts |
| `content.pad` | `#C9974A` | |
| `content.via` | `#C77B3C` | |
| `content.ratsnest` | `#AEB4BB` | per-net themeable override layer |
| `content.drc.error` | `#E5534B` | = `color.status.error`; **+ marker shape** (never hue alone) |
| `content.drc.warn` | `#E0A23A` | = `color.status.warn`; + marker shape |
| `content.exclusion` | `#6B7280` | |
| `content.selection` | `#CE5A92` | = `color.accent` (cross-surface selection) |

Net/diff coloring is a **separable override layer** over base copper, with a
defined zoom precedence policy (Altium model: base-pattern-scales /
layer-dominates / override-dominates) — to be specified when net coloring lands.

### 2.4 Typography
**UI sans = IBM Plex Sans (owner choice). Data mono = IBM Plex Mono.**
- **UI sans:** IBM Plex Sans. The repo currently bundles only the **Condensed**
  cut (`ibm_plex_sans_condensed`); standard non-condensed Plex Sans is *not*
  bundled and adding it is a font-asset addition requiring owner approval.
- **Data mono:** IBM Plex Mono, bundled and used for coordinates, IDs, terminal
  text, paths, and command-like data.
- Any font used must be OFL/permissive for embedding in the wgpu/glyphon pipeline.

Type ramp (UI sits in the 10–14px dense range):

| Token | Family | Size | Weight | Line | Use |
|---|---|---|---|---|---|
| `type.display` | IBM Plex Sans | 16 | 600 | 22 | rare titles |
| `type.header` | IBM Plex Sans | 12 | 600 | 16 | section headers (uppercase, +0.04em tracking) |
| `type.body` | IBM Plex Sans | 13 | 400 | 18 | labels, body |
| `type.strong` | IBM Plex Sans | 13 | 500 | 18 | emphasized values |
| `type.data` | IBM Plex Mono | 12 | 400 | 16 | IDs / coordinates / paths / commands |
| `type.caption` | IBM Plex Sans | 11 | 400 | 14 | secondary metadata |
| `type.micro` | IBM Plex Sans | 10 | 500 | 12 | status pills, micro-labels |

### 2.5 Spacing & density
Carbon 2/4/8 scale (the layout substrate already consumes this):

| Token | px | | Token | px |
|---|---|---|---|---|
| `sp.01` | 2 | | `sp.08` | 40 |
| `sp.02` | 4 | | `sp.09` | 48 |
| `sp.03` | 8 | | `sp.10` | 64 |
| `sp.04` | 12 | | `sp.11` | 80 |
| `sp.05` | 16 | | `sp.12` | 96 |
| `sp.06` | 24 | | `sp.13` | 160 |
| `sp.07` | 32 | | | |

**Density modes** (4px base): `comfortable` (default) and `compact` (data-dense
views). Compact reduces internal paddings and inter-row margins by one step.
Rule (Carbon): *sections* may be dense, the *page* must not be crowded — protect
whitespace at the page level.

### 2.6 Radius & borders
`radius.sm` 4px (inputs, tool buttons) · `radius.md` 6px (buttons, cards) ·
`radius.lg` 8px (panels, overlays). Borders are 1px hairlines
(`color.border.subtle` / `.strong`).

### 2.7 Contrast & accessibility
**Gate = both.** WCAG 2.x 4.5:1 (normal text) is the compliance **floor**; APCA
(Lc 75+ body, Lc 60+ larger/non-body UI text) is the dense-small-text **quality
bar**. Every chrome text-on-surface pairing is checked at every elevation. The
design-token CI check enforces this (see §7).

## 3. State & semantic encoding
State is never color-alone (colorblind-safe, WCAG non-color-reliance):

| State | Encoding |
|---|---|
| **Authored baseline** | full opacity, solid |
| **Proposed / AI preview** | dimmed "ghost" rendering (reduced opacity) + a distinct fill/outline; accepted with a single key (Tab), dismissed with Esc / continue working. Inline, contextual, non-gating. |
| **Selection / focus** | `color.accent` outline + `accent.tint` fill (chrome) / accent outline (canvas) — plus a focus ring; cross-surface consistent |
| **Severity / diagnostic** | status hue **+ marker shape** (DRC error/warn carry a glyph, never hue alone) |
| **Layer / domain identity** | content hue **+ pattern channel** (solid-fill / outline-only / striped — Horizon model) for colorblind-safe layer distinction |

## 4. Iconography
- **Style:** geometric **line/stroke** icons, ~2px stroke on a 24px keyline grid;
  line reads "precise instrument" (filled/duotone reads consumer).
- **Base set:** **Tabler Icons (MIT)** — broad coverage for a many-action tool, no
  attribution obligation. (Lucide/ISC is the curated alternative; avoid Codicons —
  CC BY requires attribution.)
- **Rendering:** icons ship as **monochrome glyphs through the existing text
  pipeline** (icon-font or MSDF atlas). They tint with chrome text tokens and
  **inherit the HiDPI/scale-factor handling for free** — one pipeline, crisp at
  fractional scale.
- **Sizing (on the 2/4/8 scale):** 16px glyph + `sp.02` pad = 24px hit target
  (compact); 20–24px for primary actions. Active tool/control uses
  `color.accent`.
- **Policy:** tooltips mandatory on icon-only controls; **never icon-only for
  critical or ambiguous actions**; the command palette + keyboard are the primary
  discoverability path.
- **Custom EDA glyphs** (generic sets lack these; same grid + stroke so they are
  indistinguishable). **Already authored** in `crates/engine/assets/icons/eda/`
  (24px grid, `currentColor`, round caps): route/track, via, zone/pour, pad,
  net/ratsnest, layer-stack, DRC marker, teardrop, keepout. **Still to author:**
  diff-pair, length-tune, and others as features land.
- **Asset structure (to build — the current gap).** Today only *fonts* have an
  asset structure (`crates/engine/src/text/registry.rs` → glyph pipeline); icons
  are loose SVGs with no loader/registry/render path, and no base chrome set is
  vendored. Establish, mirroring the font registry:
  1. vendor the base chrome set (Tabler MIT subset) beside the Datum EDA SVGs;
  2. an icon registry (`icon_id → asset path / codepoint`) parallel to the font
     registry;
  3. a build step compiling the SVGs → an icon-font (codepoints) or MSDF atlas;
  4. render through the existing glyph pipeline so icons inherit token tint +
     HiDPI. Until that lands, the SVGs are the editable asset of record (per the
     `assets/icons/eda/README.md`).

## 5. Component specifications
Each component resolves entirely to tokens above. States listed are the contract.

- **Panel** (`surface.01`, `radius.lg`, `border.subtle`): header = `type.header`
  in `text.secondary`, `sp.05` inset, hairline divider below.
- **Card / raised row** (`surface.02`): used for grouped content inside a panel.
- **Section header**: `type.header`, uppercase, `text.secondary`, `sp.04` above /
  `sp.02` below.
- **Button**: `radius.md`, `sp.03` x-pad. *Default* `surface.02` + `text.primary`;
  *hover* `surface.03`; *primary* `color.accent` fill + `text.onAccent`;
  *pressed* `accent.pressed`; *disabled* `surface.02` + `text.muted`.
- **Toggle / boolean row**: label `type.body` `text.secondary`; ON = `color.accent`
  text/indicator, OFF = `text.muted`; full-row hit target.
- **Key–value row** (inspector/property grid): key `type.body` `text.secondary`,
  value `type.strong` or `type.data` (mono for IDs/coords) `text.primary`.
- **List row**: `type.body`; selected = `accent.tint` bg + `color.accent` left
  rule; hover = `surface.02`.
- **Tab**: `type.body`; active = `text.primary` + `color.accent` underline;
  inactive = `text.muted`.
- **Status strip**: `type.caption`/`type.data`, `text.muted`, status hues for
  state pills.
- **Tool button**: 24px hit / 16px glyph; active = `accent` outline + `accent.tint`.
- **Menu / popover / command palette**: `surface.03`, `radius.md`, hairline
  border; selected item `accent.tint`.

## 6. Interaction & motion grammar
- **Selection model:** click selects; the inspector is a single context-sensitive
  panel that reflects the current selection (Altium model) — not per-type dialogs.
- **Tools:** modal tool activation (Select / Route / Via / Zone / …), keyboard
  accelerators, and a **command palette** as the primary discoverability surface.
- **Feedback:** hover and focus states on every interactive element; the focus
  ring is always visible for keyboard users.
- **AI overlay:** proposals render as dimmed ghost content (see §3), accepted/
  rejected inline by keyboard; the assistant is contextual, never modal, never
  required.
- **Motion:** purposeful and restrained — viewport continuity, highlight
  transitions, panel reveal/collapse, explicit state confirmation. No ornamental
  or attention-seeking animation.

## 7. Token → code & artboard → golden pipeline
- **Token authority:** the tables in this book are the canonical source (tracked
  contract data). Rust constants **mirror** these tokens; code never defines a raw
  visual literal. (No runtime token dependency until the schema stabilizes, per
  `PRODUCT_MECHANICS_014`.)
- **Contrast CI:** a gate checks every chrome text/surface token pair against the
  §2.7 dual gate (WCAG floor + APCA bar) and fails on regressions.
- **Artboards = goldens:** because Datum owns its renderer, reference artboards of
  each component/surface are produced by rendering them and are checked into the
  visual-regression harness as goldens, across populated states and scale factors
  {1.0, 1.25, 1.5, 2.0}. The design book's pictures and the test goldens are the
  same artifacts.
- **Governance:** new GUI chrome must consume tokens + components from this book or
  document a bounded exception; raw hex / magic type sizes are a gate failure.

## 8. Open decisions (owner calls remaining)
- Final base-ramp lightness steps and number of elevation levels (values in §2.2
  are the initial set — confirm against rendered artboards).
- Tabler vs Lucide as the shipped base icon set; final stroke weight (1.5 vs 2px).
- **Font decision (owner):** the UI-sans variant (Condensed, bundled, vs standard
  Plex Sans, a font addition). IBM Plex Mono is ratified as the data mono. Adding
  any non-bundled font requires owner approval. Then validate the ramp at real
  render sizes.
- Net/diff override precedence policy (deferred until net coloring lands).

## 9. Tracked gaps — what this book does NOT yet specify

This book currently defines a sound but **largely generic** dark-pro-tool
foundation (tokens, type, spacing, state encoding). Strip the magenta and the
copper colors and it could be reskinned for any dense desktop app. The layers
below — derived from **Datum's own product mechanics**, not external design
systems — are required before the book is *Datum's* and not *a template*. They are
the next authoring work and are intentionally unspecified rather than guessed.

### 9.1 Provenance & mutation as visual language
Datum's core is "every change is a typed operation through `commit()` + journal,
with provenance, diff, and undo." Unspecified: how an object's **provenance** reads
(authored manually / CLI / MCP / agent / import-repair), how the **journal/history**
and undo/redo are represented and navigated, and how a **diff** (authored vs
proposed, or before/after a commit) renders. This is the heart of the product and
the book is currently silent on it.

### 9.2 Proposal & review grammar
Beyond the generic "ghost text" of §3: Datum has a real proposal substrate (route,
repair, AI proposals as reviewable artifacts — inspect / apply / revalidate).
Unspecified: the authored-baseline-vs-proposed-overlay visual system, the
accept/reject/inspect affordances, and proposal provenance/policy display.

### 9.3 EDA surface information architecture
The §5 components are generic (panel / card / inspector / list). Unspecified: the
IA of the **real surfaces** — schematic editor, PCB editor (layers / nets / DRC),
routing review, manufacturing/CAM output, library/pool, BOM — each as a designed
surface, not a generic panel.

### 9.4 Multi-surface parity & supervision
Datum's ethos is one operation vocabulary across GUI / CLI / MCP / terminal.
Unspecified: how the GUI shows the same operations as the terminal/CLI (command
parity; the terminal lane as a first-class surface), and how the GUI **visually
reflects engine truth** for supervision (`model_revision`, source-shard / dirty
state, journal cursor).

### 9.5 Identity
"Datum" denotes a reference point / plane / origin (metrology, GD&T). The book uses
a generic dark theme and does **not** develop a Datum-specific identity (reference
frames, origins, fiducials, measurement). Owner aesthetic call — to be explored,
not chosen unilaterally.

### 9.6 Headless rendering / CI display (infrastructure dependency — blocking §7)
The artboard→golden pipeline (§7) and the HiDPI multi-scale gate depend on
rendering the GUI **headlessly in CI/containers**. This does not work today: the
binary constructs a winit `EventLoop` at startup (before any `--visual-test`
branch), which fails when no compositor is present — *"neither WAYLAND_DISPLAY nor
WAYLAND_SOCKET nor DISPLAY is set."* So even the offscreen screenshot path cannot
run without a display server. The visual-regression harness doc assumes a "pinned
headless display (xvfb)" that is **not provisioned** (no Xvfb / Wayland-headless
installed); software Vulkan (lavapipe `lvp_icd.json`) **is** present, so no GPU is
required — only a display. Fix avenues:
- **(a) Provision a virtual display** — Xvfb (X11) or a headless Wayland
  compositor (weston-headless / cage / sway `WLR_BACKENDS=headless`) + lavapipe,
  and run the harness under it. Smallest change; matches the harness doc's
  assumption.
- **(b) Decouple the visual/offscreen render from winit** — a true headless wgpu
  path (offscreen adapter + texture, no `EventLoop`/window) so goldens need **no**
  display at all. Only the genuinely windowed smokes (interaction / resize /
  kwin-lifecycle) then still need a virtual display.
**Status (landed):** avenue (b) is implemented — `datum-gui --visual-test
--exit-after-screenshot` now branches *before* `EventLoop::new()` and renders via
the offscreen `OffscreenRenderer` (no event loop, window, or compositor). Verified
rendering a real frame with `DISPLAY`/`WAYLAND_DISPLAY` unset, at scale 1.0 and
1.5, on software Vulkan (lavapipe). Requires building with `--features visual`
(which now also enables `datum-gui-render/visual`). **Remaining:** wire this
headless command into the visual-regression harness/CI as the multi-scale golden
gate across {1.0, 1.25, 1.5, 2.0}; the genuinely windowed smokes
(interaction/resize/kwin-lifecycle) still need avenue (a), a virtual display.

## 10. References (research basis)
Dark elevation & contrast: Atlassian, Material 3, APCA/Myndex. Tokens & density:
W3C DTCG, IBM Carbon, AWS Cloudscape. EDA content color: KiCad, Altium, Horizon
EDA. Typography: IBM Plex, JetBrains Mono. State/AI overlay: VS Code. Iconography:
Tabler, Lucide, VS Code Codicons, MSDF rendering. Full citations in the design-axis
and iconography research records.
