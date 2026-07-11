# Datum Universal Viewport & Editor-Interaction Toolkit Spec

Status: governed spec (active)

Governed by **decision `PRODUCT_MECHANICS_023_UNIVERSAL_VIEWPORT_TOOLING`** (the
ratified law). This document is the *how*: it may strengthen but not weaken 023.
It **extends** — and does not restate — the existing surface it plugs into:
`docs/gui/DATUM_GUI_DESIGN_SPEC.md` (status-bar readout), `DATUM_RENDERING_BOOK.md`
(stroke hierarchy, grid colour, filled-outline text, selection-is-screen-only),
`DATUM_GUI_PARAMETRIC_TOOLING.md` (tri-modal verbs, align `reference: grid`),
`DATUM_GUI_CONTEXT_MENU_CONTENT.md` + `research/gui-context-menus/CONTEXT_MENU_RESEARCH.md`
(the menu content/form), `docs/gui/menu_model.json` (the data-driven, gated menus),
and decisions 014/020/021/022.

Each normative claim below carries an honest **check disposition** per the
`DATUM_GUI_CONFORMANCE_SPEC.md` discipline: **ENFORCED** (an existing gate/test/
golden already locks it), **TO-ENFORCE** (a named gate to add with the implementing
slice), or **HUMAN** (reference-image / eyeball review).

---

## 1. Architecture

### 1.1 The `EditorViewport` keystone

Every drawing surface is an `EditorViewport` (never a bare "viewport" —
decision-021 workspace pane and decision-020 paper-space viewport are distinct).
Screen↔world projection and hit-testing are resolved for the **focused** surface
in that surface's own camera/space. Today both funnel through two board-only
chokepoints (`world_point_at_screen` `scene.rs:174`, `hit_test_authored_world`
`scene.rs:428`) with the schematic emitting no hit regions (`scene.rs:389`); the
keystone generalizes both to the focused `EditorViewport`. The per-pane warm
camera (P2.2d, `runtime_camera_pane.rs:530`) is the precedent.

*Disposition: TO-ENFORCE — a routing test asserting each surface maps screen→world
and hit-tests in its own space; board frame stays byte-identical (visual-parity).*

### 1.2 Render-approach law (UVT-003)

Authored, fab-bearing geometry renders from the retained world (nm) buffer.
Grid, selection, hover, cursor, snap feedback, and menu render as an **immediate
screen-space overlay** driven by the live camera — on every surface. Baking
presentation chrome into a surface's world buffer is prohibited (that is the
defect that made the schematic grid scale with zoom). Preserves `render == CAM`.

*Disposition: ENFORCED for the board (existing immediate overlay); TO-ENFORCE for
the schematic (a test that the schematic grid/selection are not in the retained
buffer) + HUMAN (zoom the schematic, grid weight constant).*

### 1.3 `ViewportProfile` (the per-surface config)

A surface is one `ViewportProfile`, bundling small config structs — never new
mechanism:

```
ViewportProfile {
    grid:      GridConfig,        // pitch table, mode (square/rect), origin, colours
    camera:    CameraConfig,      // warm-camera slot, bounds source
    snap:      SnapConfig,        // registered SnapTarget kinds, running-snap defaults
    stroke:    StrokeConfig,      // primitive -> weight-class map (see §4)
    hover:     HoverConfig,       // hoverable classes
    select:    SelectionConfig,   // selectable classes, selection visuals
    tools:     ToolSet,           // per-surface tool enum + keymap
    menu:      MenuKeyNamespace,  // "pcb.*" | "schematic.*" (already in menu_model.json)
    readout:   ReadoutConfig,     // units, precision, polar
    layers:    LayerSet,          // the surface's layer/visibility set
}
```

*Disposition: TO-ENFORCE — a gate asserting no surface constructs grid/camera/snap/
hit/menu mechanism outside `gui-viewport`.*

### 1.4 Crate boundary (UVT-002)

The shared mechanism lives in a new consumer-side crate **`gui-viewport`**
(depends on `gui-protocol` i64-nm types; the engine, daemon, protocol, and
persisted formats never depend on it — a compile-time fence, decision-014
precedent). Input-event wiring stays in `gui-app`; render batching stays in
`gui-render`.

*Disposition: TO-ENFORCE — a dependency-direction check (engine/daemon/protocol
have no edge to `gui-viewport`).*

---

## 2. The engine set (mechanism vs config)

One shared mechanism each; the per-surface variation is only the `…Config`.

| Engine | Shared mechanism | Per-surface config |
|---|---|---|
| **CoordinateHit** (keystone) | per-pane screen↔world + hit-test over world hit-regions; point-in-poly/polyline predicates | hit-region set, bounds |
| **GridEngine** | adaptive-LOD screen-space axis rects via live camera; CROSS/DOT/LINES as one `mark_size` knob; origin (§5) | pitch table, mode, colours |
| **CameraEngine** | existing `CameraState`+`Projection`; zoom-to-cursor, fit=zoom-to-selection, drag-pan; one routing path | warm-camera slot, bounds source |
| **StrokeWeightModel** | the three weight classes + projection/floor math (§4) | primitive→class map |
| **HoverEngine + cursor** | per-surface hover from CoordinateHit; crosshair; snapped-cursor glyph | hoverable classes |
| **Selection + Marquee** | per-surface single + rubber-band selection | selectable classes, visuals |
| **ToolModeEngine** | per-focused-editor active tool/mode; toolbar hit regions; per-editor keymap routing | tool set + keymap |
| **ContextMenuEngine** | per-surface content from hit+selection+profile; multi-select intersection; overflow + nested wheels; **verb execution on leaf** (§6) | menu-key namespace |
| **CoordinateReadout** | cursor X/Y→units, dx/dy vs settable origin; focused-editor status fields (§7) | units, precision, polar |
| **SnapEngine** | 2-tier ordered-scan resolver; SnapTarget registry; SnapFilter (§3) | Target kinds, defaults |
| **LayerVisibility** | per-surface layer toggle → world-range filter | layer set |

---

## 3. Snap & Quantize

Snap and quantize are two faces of one idea: mapping a free coordinate onto a
disciplined one. **Snap** is an interactive gesture — it shapes the cursor point
that becomes an argument to a typed Operation, and is itself never journaled
(UVT-006). **Quantize** is the committed batch form — a journaled Operation that
rounds already-placed geometry onto the grid. They share the same grid model and
connectivity rules; they differ only in whether the result is a live cursor or a
diff.

### 3.1 Snap-priority model (v1) — Horizon two-tier `grid → object override`

1. **Tier 1 — grid.** Round `(cursor − grid_origin)` to `grid_spacing`, per axis.
   A fine-grid modifier divides spacing by `div = 10` while held.
2. **Tier 2 — nearest object Target (overrides tier 1).** Find the nearest
   registered `SnapTarget` within `snap_radius` **screen pixels**; if one exists
   it replaces the grid point (object-snap beats grid within radius — Horizon;
   Altium "object hotspot snapping overrides a snap grid"). The cursor recolours
   on engage.

The resolver is a single ordered scan over ranked Target providers, so deferred
tiers (electrical grid, snap guides) slot in as new providers, not a rewrite:

```
resolve_snap(cursor, providers) =
    candidate ← grid_point(cursor)                 // rank 0
    for provider in providers sorted by rank desc: // higher rank overrides
        t ← provider.nearest(cursor, provider.radius_px)
        if t.is_some(): candidate ← t; break
    candidate
```

**v1 params (consumer-side, user-configurable):** `snap_radius` = **10 px**;
fine-grid `div` = **10**; grid-snap suppressor + object-snap suppressor both exist
as momentary modifiers (bindings are keymap, not spec-frozen). All gesture state,
never journaled.

**SnapFilter:** exclude the object(s) under the gesture; gate to visible layers
(+ current-layer-only toggle); a per-`SnapTargetKind` type mask (v1 all-on) — the
seam where AutoCAD-style per-type running-snaps land later.

*Disposition: TO-ENFORCE — unit tests for the ordered-scan override, screen-px
radius invariance across zoom, and SnapFilter exclusion of the moved object.*

### 3.2 Object-snap Target registry (per surface)

`SnapTarget` is a projection of resolved model truth (never an authority):

```
SnapTarget { point: Point<i64 nm>, object: ObjectId, kind: SnapTargetKind,
             surface: SurfaceRef, vertex: Option<u32> }
```

`point` is exact i64 nm; screen-px only governs eligibility radius, never the
stored value. `object` is the UUID a resulting Operation references.

- **Board (`SnapTargetKind::Pcb`):** `PadCenter`, `ViaCenter`, `TrackEndpoint`,
  `TrackVertex`, `Junction`, `PadOnGrid`.
- **Schematic (`SnapTargetKind::Sch`):** `PinEndpoint`, `WireEndpoint`,
  `WireVertex`, `Junction`, `BusConnection`, `LabelAnchor`, `NoConnect`.

*Disposition: TO-ENFORCE — a test that each surface's registry is non-empty and
Target points are exact nm; HUMAN — snapping visibly engages pins/pads.*

### 3.3 Quantize-to-grid — anchor-rounding, connectivity-preserved, no new verb

- **Anchor, not bbox, not per-vertex.** Quantize rounds each selected object's
  **placement anchor/origin** to the nearest grid node and translates it rigidly.
  Bbox-rounding shifts the anchor by a non-grid delta (misleading); per-vertex
  rounding deforms polylines and tears connected geometry (deferred as an opt-in
  `granularity`).
- **Connectivity survives because Datum nets are UUID/net-addressed, not
  coordinate-coincident.** Rounding a component's anchor drags its pins; attached
  wire endpoints ride along via the same connectivity-preserving re-solve a normal
  move performs; the wire endpoint is not independently quantized. Quantize never
  severs a connection to satisfy the grid.
- **It is an argument value, not a new verb or Operation.** Quantize =
  `datum.pcb.align_components` (and its schematic mirror) with **`reference: grid`**
  — already listed in `DATUM_GUI_PARAMETRIC_TOOLING.md`; the verb today
  (`verbs_pcb.rs:258`) exposes only `mode`. Align's `axis` param doubles as the
  quantize-axis selector (`horizontal` = round X only, `vertical` = round Y only,
  omitted = both).

The journaled Operation is the **same guarded position batch align already emits**;
`reference: grid` resolves to concrete `new_point`s at the verb/facade edge, taking
the grid as explicit already-quantized nm args so the op is replayable without live
UI state:

```
align_components { path, components: [uuid,…], op: align,
    reference: grid,                       // NEW enum value on the existing param
    axis: horizontal | vertical | <both>,  // reused as the quantize-axis selector
    grid_origin: Point<i64 nm>, grid_spacing: Vector<i64 nm> }  // recorded in provenance
```

Locked objects are skipped by the existing batch guard.

*Disposition: TO-ENFORCE — verb-registry parity (the new `reference: grid` value +
menu verb) and a test that quantize preserves net connectivity; owner sign-off on
semantics before the slice lands (UVT-006).*

### 3.4 Deferred behind the interface (additive, no rework)

Altium electrical grid (a Target provider at its own rank + radius); snap guides
(a higher-rank provider); dual-axis snap distance (`snap_radius` → `{x,y}`);
AutoCAD per-type running-snap toggles (the SnapFilter type mask); per-vertex/bbox
quantize `granularity`. Each was checked against §3.1–§3.3 to confirm it is a later
addition, not a redesign.

---

## 4. Weight-Class Policy

> Resolves the "strokes thicken on zoom-in" defect: schematic grid, wires, and
> symbol strokes are baked in world-nm against a fixed reference projection and
> then re-scaled by the live camera. A **second** latent defect: `world_stroke_nm`
> (`geometry.rs:263`) floors `.max(1.0)` in **nanometres** — a no-op (1 nm is
> invisible) — so the intended min-width clamp never fires. This section assigns
> every primitive a weight class and fixes the widths, floors, and the single LOD
> threshold, cross-checked against `render==CAM` Law 1 and grid readability.

### 4.1 The three classes

- **A — `ScreenConstant(px)`** — fixed device-pixel weight, resolved every frame
  against the live camera, never emitted into a retained world buffer. Chrome only.
- **B — `WorldWidthWithMinClamp(nm, min_px)`** — true per-object world width;
  scales with zoom (physically correct) but its **projected** width is floored at
  `min_px` device px so a thin object never vanishes zoomed out.
- **C — `AuthoredConstantNm(nm, min_px)`** — a house/importer nominal nm literal;
  renders identically to B. B vs C is a **provenance** distinction (user-owned vs
  document-default width), not a render-behaviour one.

**Invariant:** class A is the *only* zoom-invariant class. Everything representing
real document/fab geometry is B/C and must scale. The grid thickening bug is a
class-A primitive mis-implemented as world geometry; the wire thickening is a B/C
primitive frozen against the reference (not live) projection.

### 4.2 Primitive → class table

`1 mil = 25 400 nm`. Widths: device px for A (exact), nm for B/C (nominal) + `min_px`.

| Primitive | Class | Width / nominal | `min_px` | Notes |
|---|---|---|---|---|
| Grid minor line | A | 1.0 px | — | hairline; differentiate by tone, not weight |
| Grid major line | A | 1.0 px | — | heavier tone, same stroke |
| Grid axis / origin | A | 1.5 px | — | accent, still zoom-invariant |
| Grid dot / cross | A | 1.0 px | — | KiCad parity |
| Selection highlight | A | 2.0 px halo | — | existing overlay emphasis floor |
| Hover pre-highlight | A | 1.5 px | — | lighter than selection |
| Cursor crosshair | A | 1.0 px | — | |
| Snapped-cursor glyph | A | 1.5 px | — | at the snapped point |
| Marquee rectangle | A | 1.0 px dashed | — | |
| Copper trace | B | per-object | 1.0 px | scaling is correct (Law 1) |
| Pad / via | B | filled area | — | LOD-hide sub-pixel, no clamp |
| Board silk line | C | 150 000 nm | 1.0 px | |
| Board/silk text | B | filled-outline glyph | — | §5 RENDERING_BOOK; LOD-hide < ~6 px cap |
| Edge.Cuts / outline | C | 100 000 nm | 1.0 px | |
| Schematic wire | C | 152 400 nm (6 mil) | 1.0 px | KiCad default |
| Schematic bus | C | 304 800 nm (12 mil) | 1.5 px | top of hierarchy (§2) |
| Bus-entry | C | 152 400 nm | 1.0 px | |
| Symbol body outline | C | 127 000 nm (5 mil) | 1.0 px | below wire |
| Pin line / stub | C | 101 600 nm (4 mil) | 1.0 px | thinnest |
| Pin terminal dot | C | 300 000 nm dia | 3.0 px | symbol-stroke colour |
| Junction dot | C | 400 000 nm dia | 3.0 px | wire colour, larger |
| RefDes/Value/label text | B | filled-outline, 1.27 mm | — | LOD-hide < ~6 px cap |
| Pin-name / pin-number text | B | filled-outline (Plex Mono for numbers) | — | |
| Net-label pill | C | border 127 000 nm + fill | 1.0 px | |
| No-connect marker | C | 152 400 nm (X) | 1.0 px | |
| Power-symbol glyph | C | 127 000 nm | 1.0 px | |

**Text is never a class-A stroke** — per RENDERING_BOOK §5 all on-canvas/silk text
is **filled-outline geometry** (class B fill), governed by LOD hide-below-cap, not
a min-px clamp (which would smear a sub-pixel glyph).

### 4.3 `min_px` floor reconciliation (three roles, not one constant)

- **Grid = 1.0 px exact, class A** — an exact width, not a floor.
- **Real geometry (copper/silk/wire/pin/outline) = 1.0 px floor, class B/C** —
  applied **in device px after live projection**: `width_px = (nominal_nm ×
  live_scale).max(min_px)`. This fixes the nm-floor no-op in `world_stroke_nm`.
- **Attention overlays (selection/proposal/hover) = 2.0 px floor, class A** —
  deliberately heavier for emphasis; not the geometry floor, never applied to copper.
- Junction/terminal **dots = 3.0 px floor** (a sub-3-px disc reads as a stray pixel).

*Disposition: TO-ENFORCE — a unit test that class-B width floors in device px (not
nm) against the live projection; HUMAN — zoom test, grid + selection weight constant.*

### 4.4 LOD threshold (unified, one rule for both panes)

Replace the two ad-hoc `px_per_mm` cutoffs (`pads_and_layers.rs:1212`) with one
threshold on **on-screen grid spacing** `S_px = pitch_nm × 1e-6 × px_per_mm`:

- **Coarsen knee — `S_px < 20`:** advance one tier (drop minor, promote major, ×2
  pitch). Horizon's coarsen point.
- **Re-fine knee — `S_px > 80`:** step one tier finer. The 4× gap is deliberate
  hysteresis to kill tier-flicker.
- **Hide-grid floor — major `S_px < 10`:** draw no grid (KiCad `m_gridMinSpacing`).

The existing `≥8` Normal cutoff already *is* the 20-px knee (2.5 mm × 8 = 20);
retune Fine to `px_per_mm ≥ 16` (1.25 mm × 16 = 20) so both boundaries share the
one rule and the schematic pane inherits it from its own pitches.

*Disposition: TO-ENFORCE — a test that both panes hit the same 20-px knee.*

---

## 5. Grid engine

Screen-space axis rects (class A, §4) drawn per-frame against the live camera;
CROSS/DOT/LINES as one continuous `mark_size` knob (Horizon); origin marker; LOD
per §4.4. Config: pitch table (board metric 2.5/5/10 mm; schematic imperial
1.27/2.54 mm), mode SQUARE/RECTANGULAR, colours (`#141821` line grid from
RENDERING_BOOK). The board grid already renders this way; the fix is to route the
schematic grid through the same engine and stop baking it into the world buffer
(`scene.rs:357`).

*Disposition: ENFORCED (board golden byte-identical); HUMAN + TO-ENFORCE for the
schematic weight-constant-on-zoom check.*

---

## 6. Context menu (local/marking menu) build contract

The content/form is already designed — build to it, do not re-author:
`DATUM_GUI_CONTEXT_MENU_CONTENT.md` (per-object content, both surfaces),
`CONTEXT_MENU_RESEARCH.md` (HCI form: cardinal-4, ≤8/level, depth ≤2, "More…"
overflow, mark-ahead), and the CI-validated data model `menu_model.json`
(`pcb.*` AND `schematic.*` menus, gated by `check_menu_model.py`).

The runtime (`ContextMenuEngine`) must add what is missing today:
1. **Per-surface content** resolved from the focused `EditorViewport`'s hit-test +
   selection + `MenuKeyNamespace` — replacing the board-only `pcb.*` key function
   (`main.rs:3431`) and the board-coordinate gate that currently makes a schematic
   right-click fall through to a pan (`main.rs:415`).
2. **Multi-select = intersection** of per-type menus (`workspace().selection`,
   ignored today).
3. **Verb execution on leaf-select** — replacing `MarkingMenuItem => dismiss`
   (`main.rs:2982`). Each leaf fires its bound typed verb (tri-modal). View/read
   verbs fire immediately; **authoring verbs ride the GUI→engine write path**
   (Frontier write-path step); until then an authoring leaf is disabled/queued, not
   silently inert.
4. **Overflow list + nested `▸` sub-wheels** (unbuilt; `marking_menu.rs:114` draws a
   static "MORE…").

*Disposition: ENFORCED (`check_menu_model.py` locks the data + slot invariants);
TO-ENFORCE (a test that a schematic right-click opens a schematic menu and a leaf
fires its verb); HUMAN (radial matches `context-menu-marking-menu.html`).*

---

## 7. Coordinate readout & status-bar field ownership

Add the currently-absent readout: cursor X/Y → display units, dx/dy vs a settable
origin, per the status bar already specified in `DATUM_GUI_DESIGN_SPEC.md`
("cursor X/Y (mm) · grid"). Status fields (Tool/Sel/Layers) are owned by the
**focused** editor, not the global board state (`scene.rs:617` already routes the
document label per focus — extend to the other fields). v1 = X/Y + dx/dy + grid +
units; deferred = Z/dist/polar/Space-to-zero (KiCad full readout).

*Disposition: TO-ENFORCE — a test that readout tracks the focused pane's cursor in
that pane's units.*

---

## 8. Surface profiles (v1)

- **Board** — Pcb Target kinds; metric grid; board tool set; `pcb.*` menu; full
  board layer set. (Repointed onto the shared engines only once schematic-proven —
  schematic-first rollout.)
- **Schematic** — Sch Target kinds; imperial grid; schematic tool set (new — extend
  the tool enum, `geometry.rs:1154`); `schematic.*` menu; net-role layer set.
- **Footprint / Symbol (future)** — authored as profiles when those editors land;
  no new mechanism.

---

## 9. Research traceability

| Open question | Resolved in | Implementing slice |
|---|---|---|
| Weight-class per primitive | §4.2 | S0 (model) + S1 (grid) + per-primitive as surfaces repoint |
| min-px floor unit bug | §4.3 | S0 / S1 |
| Adaptive-LOD threshold | §4.4 | S1 |
| Snap-priority model | §3.1 | S10 |
| Object-snap Target registry | §3.2 | S10 |
| Quantize-to-grid semantics + op shape | §3.3 | S11 |
| Context-menu (already designed) | §6 | S7 |
| Coordinate-readout scope | §7 | S8 |
| Deferred richness (named/polar/electrical/…) | §3.4, §8 | post-v1, same interfaces |

---

## 10. Slice map

Delivery is staged per the campaign plan; the ordered, dependency-aware position
lives in the **Active Frontier** (`specs/PROGRESS.md`), not restated here. Spine:
S0 crate + StrokeWeightModel → S1 GridEngine (fixes the bug) → S2 CameraEngine →
**S3 CoordinateHit keystone** → {S4 hover, S5 selection+marquee, S6 tool-mode,
S7 context-menu, S8 readout, S9 layer-vis} → S10 SnapEngine → S11 quantize verb.
Each slice keeps the board visual-parity golden green (or a deliberate re-bless)
and honours source-health burn-down (decision 022).
