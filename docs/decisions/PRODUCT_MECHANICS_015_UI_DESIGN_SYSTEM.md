# Product Mechanics 015: UI Design System (Design Book)

> **Status**: Ratified.
> **Scope**: Datum's visual + interaction design language — color, typography,
> spacing/density, state encoding, iconography, component visuals, interaction
> grammar.
> **Complements**: `PRODUCT_MECHANICS_014_UI_LAYOUT_SYSTEM` (Taffy geometry,
> HiDPI). 014 decides *where* boxes go; 015 decides *what they look like and why*.
> **Design Book**: `docs/gui/VISUAL_LANGUAGE.md` (controlling spec + token tables).

## Decision

Datum adopts a single, research-backed **design system** (the Design Book) as the
authority for every visual and interaction decision in the GUI. All GUI surfaces
resolve to its tokens and component specs; no surface defines raw visual literals.

The system separates two token namespaces, governed differently:
- **Chrome tokens** — application UI; design-owned.
- **Content tokens** — board/schematic canvas; **user-themeable** (KiCad/Altium
  model). Chrome and content tokens are never interchanged.

## Rationale

Before this decision, Datum had **no design authority**. The prior "Visual
Language" was a manifesto that deferred every concrete decision; the actual look
was ~12 hand-picked color constants and scattered font-size literals in
`gui-render`, with no rationale, no contrast basis, no type scale, and no
separation of content from chrome. Building editor surfaces on that means
improvising hundreds of micro-decisions that then calcify — the same failure mode
that produced the panel-overlap defects 014 addresses. A design system is
infrastructure; the differentiator is EDA authoring, and that authoring needs a
defined, consistent, accessible surface to live in.

## Locked foundation decisions

| Foundation | Decision |
|---|---|
| Theme | Dark-first; elevation by **lightening** surfaces (not shadows) |
| Base hue | Cool neutral gray |
| Accent | Deep magenta `#CE5A92` (single chrome accent; = canvas selection) |
| UI font | IBM Plex Sans (owner choice; Condensed-vs-standard variant OPEN) |
| Mono | OPEN — owner decision pending (IBM Plex Mono implied; not bundled) |
| Spacing | Carbon 2/4/8, 13-step (2–160px) |
| Density | Dual `comfortable` / `compact` on a 4px base |
| State encoding | Beyond color: pattern channel (fill/outline/stripe) + dimmed ghost-text for AI proposals + marker shapes for diagnostics |
| Token split | Separate **chrome** (design-owned) vs **content** (user-themeable) sets |
| Iconography | Tabler (MIT) line icons, ~2px, 16px/24px hit, rendered as glyphs through the text pipeline, magenta active state |
| Contrast gate | WCAG 2.x 4.5:1 floor **+** APCA (Lc 75/60) bar |

Concrete token values are in the Design Book §2 and are the **initial ratified
set** — derived within the locked directions, contrast-checked, adjustable when
rendered as artboards.

## Token authority & pipeline

- The Design Book token tables are the **canonical source** (tracked contract
  data). Rust constants **mirror** them; code never defines a raw visual literal.
- No runtime token dependency until the schema stabilizes (per 014).
- **Artboards = goldens:** reference renders of each component/surface are checked
  into the visual-regression harness across populated states and scale factors
  {1.0, 1.25, 1.5, 2.0}. The book's pictures and the test goldens are one artifact.

## Governance

New or reshaped GUI chrome must either consume Design Book tokens + components, or
document a bounded exception with a migration path. Specifically:
- raw hex / magic type sizes in GUI code are a gate failure;
- every chrome text/surface pairing passes the dual contrast gate;
- state is never encoded by color alone;
- content color never leaks into chrome tokens (and vice versa).

## Non-goals

This decision does **not**: adopt a full GUI framework or component kit; replace
the layout substrate (014 owns geometry); finalize the net/diff override
precedence policy (deferred until net coloring lands); or author the custom EDA
glyph set (named in the book §4 as future design work).

## Open items (owner calls)

- Final base-ramp lightness steps / elevation-level count.
- Tabler vs Lucide as the shipped icon set; final stroke weight.
- Font decision (owner): UI-sans variant (Condensed vs standard) and the data
  mono; then type-ramp validation at real render sizes.
- Net/diff override precedence policy.

## Asset-state notes (reconciled with the repo)

- **Fonts** have an asset structure (`crates/engine/src/text/registry.rs` loads
  Inter, IBM Plex Sans Condensed, JetBrains Mono, DejaVu/dev into the glyph
  pipeline). The UI-sans variant and the data mono are OPEN owner decisions
  (§2.4); any non-bundled font (standard Plex Sans, IBM Plex Mono, …) is a
  font-asset addition requiring explicit owner approval.
- **Icons have no asset structure yet.** The first custom EDA glyph SVGs exist
  (`crates/engine/assets/icons/eda/`, per Design Book §4), but there is no icon
  registry, loader, render path, or vendored base chrome set. Building that
  structure (registry + SVG→icon-font/MSDF pipeline, mirroring the font registry)
  is prerequisite work before icon-bearing GUI surfaces — see Design Book §4.

## Tracked gaps

This decision ratifies a sound but **largely generic** foundation. The
Datum-specific layers it does **not** yet specify — provenance/mutation visual
language, proposal/review grammar, real EDA-surface IA, multi-surface
parity/supervision, identity — plus the **blocking headless-render / CI-display
dependency** (the GUI binary needs a compositor even for offscreen goldens; no
virtual display is provisioned) are tracked in **Design Book §9**. These gate the
book becoming *Datum's* rather than *a template*, and gate the §7 golden pipeline.
