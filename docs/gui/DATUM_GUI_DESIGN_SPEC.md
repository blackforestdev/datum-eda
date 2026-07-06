# Datum GUI Design Spec — The "How"

> **Status**: Governed active design spec.
> **Authority**: Realizes the *visual/interaction execution* ("how") of the GUI
> that `docs/gui/DATUM_GUI_PRODUCT_SPEC.md` (the "what") and decision 019 leave
> open, on the systems ratified by decisions **014** (layout geometry) and **015**
> (Design Book tokens). Design authored by the project owner; this doc records the
> ratified design decisions and points at the controlling visual prototype.
> **Reference prototype**: `docs/gui/prototypes/board-editor.html` (design pass 3 — split PCB|Schematic view).
> **Scope**: The concrete visual composition, density, and interaction craft of
> the Datum desktop GUI — starting with the board editor.

## Why this exists

The product spec, menu bindings, and even the Design Book specify *what* the GUI
contains and *why* — a menu tree, a component checklist, a token palette. None of
them is a **design**: the composition, visual hierarchy, density rhythm, and
interaction choreography that decide whether the interface is clean, attractive,
and intuitive. That craft is the "how," and in UI it is often more decisive than
the what/why. This spec captures it, developed together with a live HTML
prototype (the vehicle) so the two directly convey the intended interface.

## Design thesis (the pinned direction)

Datum's GUI is a **professional instrument in the pro-audio idiom** — the visual
language of Bitwig / Ableton (and plugins in that lineage: Vital, iZotope RX,
FabFilter). This is not an aesthetic accident: pro-audio apps render everything
custom (no OS-native widgets) — the exact architecture Datum chose with wgpu — so
they are Datum's real peer class. The governing rules:

1. **Flat, dark, restrained chrome. Zero decoration.**
2. **Color is meaning, never decoration.** Chrome is the Design Book's monochrome
   ramp; saturated color appears *only* on canvas content (copper/nets/layers)
   and as the single `#CE5A92` magenta selection accent. (Ableton's gray chrome +
   colored clips; Bitwig's semantic modulation color.)
3. **The canvas is the protagonist**, and controls act *on* it (Altium's Properties
   Panel + direct manipulation — the anti-dialog-per-object thesis).
4. **Dense, not cramped** — ruthless hierarchy and uniform rhythm, generous where
   it counts.

## Reference prototype

`docs/gui/prototypes/board-editor.html` is the **controlling visual reference**
for the board editor. It is built entirely from the Design Book chrome/content
tokens. This document narrates the decisions it embodies; when they disagree, the
prototype wins for visuals and this doc wins for rationale/rules — reconcile in
the same change.

## Locked decisions — board editor v1

- **Shell composition** (left→right, top→bottom): menu bar · tool rail (icon
  accelerators) · left column (Project tree over Layers) · **central board canvas
  (protagonist)** · right column (Inspector) · bottom dock (Terminal/Assistant
  tabs) · status bar. Approximate widths: rail ~46px, left ~228px, right ~300px.
- **Color-application law**: chrome uses only `bg/surface.01–03/border/text`
  tokens; the only chrome color allowed onto the canvas is `--acc` (#CE5A92) as
  selection. Copper/nets/pads/vias/ratsnest use the content tokens.
- **Selection → Inspector binding**: single selection is the primary model;
  selecting an object (magenta outline + glow on canvas) drives the Inspector and
  cross-highlights panel ↔ canvas.
- **Inspector = Properties Panel**: context-sensitive to the selection;
  inline-editable rows (no dialogs); collapsible grouped sections (Identity /
  Placement / Nets / Checks); key-value rows with mono tabular values for
  coordinates.
- **Layers panel**: Ableton colored-track-row idiom — swatch + name + visibility,
  active layer accented.
- **Type & density**: dense but legible — ~25px rows, chrome text ~13.5–14.5px,
  uppercase section labels ~11.5px with letter-spacing, `tabular-nums` on all
  coordinates/IDs. **Production UI font = IBM Plex Sans** (the prototype uses a
  system stand-in; the sandbox can't embed the binary).
- **Icon scale/style**: line/stroke glyphs — tool-rail ~20px, tree/panel ~15px;
  every tool carries a single-key accelerator; tooltips mandatory on icon-only
  controls; never icon-only for critical actions.
- **Status bar**: cursor X/Y (mm) · active tool · selection · grid · active layer
  · DRC count · model revision.

## Workspace & Mode Model

Datum is a **single unified viewport**, not a set of separate editor windows. The
user opens a document from the Project pane and the viewport enters that
document's **mode**; the mode carries its own toolset and menus.

- **Documents / modes**: schematic, PCB/board, footprint editor, symbol editor
  (library-object modes), plus read views (rules/check report, manufacturing).
  Selecting `project → Schematic` / `Board` / a library `Footprint` / `Symbol`
  switches the mode.
- **Mode-specific tools (the SolidWorks pattern)**: each mode swaps the tool rail
  and the active-editor-gated menus (schematic: wire / symbol / label / bus /
  junction; PCB: route / via / zone / place; footprint & symbol: their drawing
  tools) — exactly the gating `DATUM_GUI_MENU_BINDINGS.md` already assumes.
- **Tiling ("tmux for EDA")**: the viewport splits into panes; each pane is a
  **(document, view) pair**. This one abstraction covers both "2D + 3D of the same
  board" and "schematic here, PCB there, footprint in a PIP." Panes tile or float
  (picture-in-picture).
- **Context follows focus**: the focused pane owns the tool rail and menus, and
  the Inspector / Layers / Filters panels bind to the focused pane's document and
  selection.
- **Cross-probe over one model**: selection is engine-level, so selecting an
  object in one pane highlights its counterparts in every other pane showing
  related objects (schematic symbol ↔ board footprint ↔ net). This falls out of
  tiled panes over one shared `DesignModel` — Altium's cross-probe / Horizon's
  message bus, for free.
- **Open sub-decisions**: whether tools live in a per-pane header strip (as pass 3
  shows) vs. a global left rail that retargets on focus; PIP vs. tile-only.

Reference: `docs/gui/prototypes/board-editor.html` (pass 3) shows a PCB|Schematic
split with per-pane mode tools, context-follows-focus, and U1 cross-probed across
both panes.

## Open design decisions (resolve before broader build-out)

1. **Bottom dock vs. inline overlay for AI/Terminal.** The prototype draws the
   docked Terminal/Assistant model (FOUNDATION/M7); the Design Book argues AI is
   an *inline ghost overlay*, "never a chat app bolted into the shell." Pick one.
2. **Fonts**: embed IBM Plex Sans; choose the data-mono face (Design Book leaves
   it open).
3. **Context menus (right-click) — currently UNSPECIFIED and unresearched.** A
   core EDA interaction surface (canvas context menu, per-object-type actions,
   panel-row menus) is absent from the entire corpus. Must be researched and
   designed before the editor is buildable.
4. **Datum visual identity**: reference frames, origins, fiducials, measurement
   styling — no Datum-specific identity yet (generic dark theme). Owner call.

## Working method

Spec ⇄ prototype, co-developed with tight back-and-forth until the pair directly
conveys the interface. The prototype is the visual source of truth; this spec is
the ratified rationale + rules. Each approved pass updates both in the same
change. Next surfaces after the board editor: schematic editor, library browser.

## Governance

Tracked in `specs/spec_governance_manifest.json` (docs/gui enforced glob) and in
the `specs/PROGRESS.md` Active Frontier (step 1's first concrete deliverable).
The prototype under `docs/gui/prototypes/` is referenced here so it is not an
orphan. Update this spec + the prototype together; never let one drift.
