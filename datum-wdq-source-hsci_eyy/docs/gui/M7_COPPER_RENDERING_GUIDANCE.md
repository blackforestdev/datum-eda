# M7 Copper Rendering Guidance

> **Status**: Active design guidance for the copper-layer surfaces in the
> opening `M7` board-review viewport — covers rendering, interactive
> creation (routing), and interactive editing (drag, cleanup, rip-up).
> **Derived from**: `research/copper-rendering/COPPER_RENDERING_RESEARCH.md`
> (industry survey of Altium, KiCad, Allegro/OrCAD, PADS, Eagle/Fusion,
> Horizon EDA, DipTrace, EasyEDA, Quadcept, Pulsonix).
> **Anchors**: extends the authored-copper lane defined in
> `M7_RENDER_SEMANTIC_CONTRACT.md`. Companion to
> `M7_AIRWIRE_RENDERING_GUIDANCE.md` — the unrouted overlay sits *above*
> the copper this document defines. Engine surface today: `Track`, `Via`,
> `Zone`, `NetClass` in `crates/engine/src/board/board_types.rs:19`,
> `PlacedPad` in `crates/engine/src/board/pad.rs:32`.

## Purpose

Lock the copper-layer contract for `M7` so engine, renderer, routing
tools, and editing tools converge on industry-consistent behaviour
without re-litigating each surface.

This is not a style memo. It is a behavioural contract: every item below
is either (a) a settled industry convention Datum must honour to be
readable to incoming users, or (b) an explicit Datum-specific
differentiator. Hotkey assignments are part of the contract — copper
authoring is hotkey-driven and users come with deep muscle memory.

## Hard Requirements

Non-negotiable. Survey shows zero production tools deviate; forum
traffic confirms users notice immediately when they are wrong.

1. **Tracks render as round-cap stroked capsules.** No flat caps. Round
   caps match what the photoplotter produces and what the etched copper
   actually looks like. Anti-aliased.
2. **Vias render as filled annulus + drill marker.** Outer-diameter
   filled ring with the drill hole drawn through. Same on every layer
   the via passes through.
3. **Pads render as filled polygons** sized to the canonical pad-shape
   set: Circle, Rectangle, Roundrect, Oval, Trapezoidal, Chamfered,
   Custom. Every professional tool ships this set; anything missing
   reads as a toy.
4. **Zones render as solid filled polygon with knockouts**, with
   thermal-relief spokes drawn as actual computed geometry. Hatched
   fill is an opt-in mode, never the default. Outline-only is the
   intermediate (unfilled) state.
5. **Positive zones, never negative planes.** Match KiCad / Altium /
   Eagle / Horizon. The data-model and fab-output (IPC-2581 / ODB++)
   reasons align; legacy negative-plane representation is not in v1
   and probably never.
6. **Industry-standard default colour scheme**: F.Cu = red,
   B.Cu = blue, inner signal layers = yellow / cyan / magenta /
   violet rotation. Edge.Cuts = yellow. A user coming from KiCad,
   Altium, Eagle, or any consumer tool expects this and reaches for
   the layer panel with that mental model.
7. **Active layer indicated three ways**: bold layer name in the panel,
   a coloured indicator at the panel row, and a continuously-updated
   layer name in the status bar. The crosshair cursor colour during
   routing matches the active layer colour. Single-channel indication
   (panel only) is the symptomatic complaint from every tool that has
   ever shipped only one channel.
8. **Connectivity authority.** The connectivity graph driving DRC,
   ratsnest, and zone-fill pad-inclusion is the same graph. A pad
   that "should" connect to a zone but doesn't (layer mismatch,
   priority shadow, rule failure) carries a per-pad reason payload.
   One source of truth.

## Algorithm Contract (Engine)

The engine owns copper geometry, copper connectivity, and zone
fill computation. The GUI consumes stable lists and fragments;
it does not run booleans, MST, or shove logic.

### Tracks

- Stored as straight segments with `from`, `to`, `width`, `layer`,
  `net`. Add `arc_center: Option<Point>` so curved tracks ship
  native rather than as a bezier-of-segments approximation. Required
  by KiCad-pattern arc tracks; cheap to add now, painful later.
- Stable sort emitted tracks by `(layer, net_uuid, uuid)` for
  deterministic screenshot goldens and JSON exports.
- Track lock is a per-track flag, not a session-only state. Locked
  tracks survive push-and-shove and refuse drag-with-rules.

### Vias

- Stored with `position`, `drill`, `diameter`, `from_layer`,
  `to_layer`, `net`.
- Add a `via_type: ViaType` enum: `Through`, `Blind`, `Buried`,
  `Microvia`. The `(from_layer, to_layer)` pair encodes the span;
  `via_type` distinguishes microvia (laser-drilled, single-layer
  span, smaller geometry rules) from regular blind/buried.
- Add `tented_top: bool` and `tented_bottom: bool`. Tenting affects
  the mask layer's render, not the copper layer's render — but the
  flag must travel with the via.
- Intermediate-layer copper rings: only `from_layer` and `to_layer`
  carry the via pad; the drilled barrel passes through intermediate
  layers without a copper ring unless the design opts into stacked
  microvias. Document this invariant explicitly so the renderer
  knows what to draw.

### Pads

- Carry an explicit padstack reference (`padstack_uuid: Option<Uuid>`)
  even if the in-board geometry is per-layer-derived. Required for
  clean ODB++ / IPC-2581 export; cheap to add now, expensive after
  fab-output adapters depend on the current model.
- Pad shape is a canonical enum: `Circle`, `Rectangle`, `Roundrect`,
  `Oval`, `Trapezoidal`, `Chamfered`, `Custom(Polygon)`.
  `Custom(Polygon)` is the universal escape hatch for odd footprints.
- Through-hole pads emit the pad shape on every copper layer the hole
  passes through; SMD pads emit only on the layer they live on.

### Zones

- Stored as `outline_polygon`, `layer`, `priority`, `net`,
  `thermal_relief`, `thermal_gap`, `thermal_spoke_width`,
  `fill_style: { Solid, Hatched }`.
- Fill computation produces a `Vec<Fragment>` where each fragment is
  `(outline_path, holes[], pad_inclusion: HashMap<PadUuid, ConnectionKind>)`.
  Match Horizon's `Plane::Fragment` model. Orphan fragments
  (no pad/via contact) are tracked independently so the GUI can
  surface them as a diagnostic.
- Per-zone incremental refill. A `dirty_zones` set on the engine API;
  refill rebuilds only those zones. Mirror the airwire
  `(fast, only_set)` parameterisation: `fast` skips thermal-spoke
  fitness for interactive drag.
- Stable per-fragment IDs derived deterministically from
  `(zone_uuid, fragment_centroid, fragment_area)`. The renderer
  holds per-fragment GPU buffers and re-uploads only fragments
  whose ID changed. Same determinism invariant as airwires.
- Fragment ordering inside a zone sorted by
  `(area_descending, centroid)` for deterministic golden output.

### Connectivity & diagnostics

- Every zone fragment carries a per-pad inclusion map: for each
  same-net pad in the zone's bounding region, did the fragment
  connect to it (solid / thermal-spoke / orphan), and if not,
  why not (`ConnectivityReason`). Feeds the
  "Why is this pad disconnected from this zone?" diagnostic.
- All copper output (track list, via list, zone fragment list) must
  be deterministic across runs for a given board state.

## Renderer Contract (GUI)

The renderer consumes engine copper output and is responsible only
for visual presentation, never for geometry computation.

### Tracks

- **Round-cap antialiased capsules**, GPU-instanced. One pipeline
  pass per layer with per-instance `(from, to, width, color)`.
- **Mitred 45° corners** by default. Rounded-corner mode is a global
  preference, off by default.
- **Width is in board units** (the engine's `i64` nm). The renderer
  converts to screen px per zoom; do not store widths in screen
  units.
- **Per-net colour with `color_mode: { None, Copper, All }`.**
  Symmetric to the airwire `color_mode` enum
  (`None / Ratsnest / All`). Default `None` for copper (let layer
  colour dominate by default; per-net colour in copper is a
  power-user mode).
- **Layer colour wins by default** when no per-net override is set.
  Per-net override applies when `color_mode != None`.
- **LOD culling** for tracks below a screen-pixel threshold (KiCad
  `lodScaleForThreshold` pattern). Still draw the layer-batch outline
  so the layer remains visible when zoomed out.

### Vias

- **Concentric annulus + drill marker**. Same instanced pipeline as
  tracks (a via is "stationary capsule with drill"). The annulus is
  filled at the layer colour; the drill marker is a contrasting hue
  (black on light themes, white on dark).
- **Tented vias**: drill marker is suppressed in the mask layer's
  pass when the via is tented for that side. The copper layer's
  pass is unaffected.
- **Via-in-pad**: the via draws *under* the pad on the same layer
  by default. Pad on top so users see the fabricated appearance.
  A preference flips this to "ghost via through pad" for placement
  review (KiCad default behaviour).
- **Via type indicator**: blind, buried, and microvia render with a
  subtle distinguishing mark (a thin extra ring, a tick, or a
  per-type drill-marker glyph). Through vias are unmarked baseline.

### Pads

- **Filled polygon at the pad shape**. Layer colour by default.
- **Through-hole pads** emit one pad polygon per layer the hole
  passes through. The drill marker draws on top of the pad, same
  rule as vias.
- **Pad outline mode**: an Appearance preference renders pads as
  outlines only (high-contrast mode for placement review). Off by
  default.
- **Solder-mask aperture overlay** (when the mask layer is visible)
  draws as a translucent overlay slightly larger than the pad,
  per the mask-expansion rule. Renders on the mask layer's pass,
  not the copper layer's.

### Zones

- **Filled polygon with holes** per fragment. GPU triangulation
  (earcut) or stencil fill. Solid by default; hatched when
  `fill_style == Hatched`.
- **Thermal relief renders the actual computed spoke geometry** —
  not a schematic stand-in, not a stylised badge. Spokes are part
  of the fragment outline returned by the engine.
- **Default zone opacity ~70%** so underlying tracks remain visible.
  Configurable per-class via Appearance opacity sliders.
- **Multi-island zones**: each fragment renders as an independent
  fill, not as a single combined polygon. Orphan fragments
  (no pad/via contact) render with a distinguishing visual
  treatment — see the "Stale & orphan markers" subsection below.
- **Keepout zones** render as crosshatch outline only, no fill, in a
  distinct hue (purple / magenta — match Altium / KiCad). A keepout
  is data-model distinct from a copper zone and renders distinct.

### Layer system

- **Default colour scheme**: F.Cu = red, B.Cu = blue, inner signal
  layers = yellow / cyan / magenta / violet rotation, Edge.Cuts =
  yellow, mask = magenta (top) / dark green (bottom), silk = white
  (dark theme) / black (light theme), drill marker = contrasting
  hue. Ship two or three alternative themes
  (high-contrast monochrome, photonics-friendly) as built-in
  presets but default to red/blue.
- **Layer ordering in the panel**: stack-up order, top → bottom.
  Active layer marked with bold name *and* a coloured indicator
  arrow next to its swatch.
- **Active layer cursor**: crosshair colour matches the active
  layer's copper colour during routing. Subtle but high-impact
  for layer awareness.
- **High-contrast mode**: hotkey **Ctrl+H** cycles
  `Normal → Dim → Hidden` for inactive copper layers. Dim factor
  is a slider in Preferences (default 50 %). Process layers
  (mask, paste, silk) follow the same dim setting.
- **Per-object opacity sliders** in the Appearance panel:
  Tracks, Vias, Pads, Zones each have an independent slider.
  Default zones to 70 %, everything else 100 %.

### Stack ordering (within a layer)

Per `M7_RENDER_SEMANTIC_CONTRACT.md § Stack Rule`. Within an
authored-copper layer, the draw order is:

1. Zone fragments (lowest, so tracks show on top of pours)
2. Tracks
3. Pads (so the pad annulus is always visible above tracks
   that abut it)
4. Vias (so the via annulus is always visible above tracks)

Across layers: back-side layers draw before front-side, with
the active layer's full opacity preserved per the high-contrast
mode setting.

### Stale & orphan markers

Datum-specific visual lane that no surveyed tool ships, but that
multiple forum threads beg for:

- **Stale-zone marker**: zones whose computed fill is older than
  the most recent edit affecting their net / outline / contained
  obstacles render with a subtle stale treatment (slightly
  desaturated + an outline overlay). Removes the "did the fill
  update?" anxiety class. Symmetric to the airwire stale marker.
- **Orphan-fragment marker**: an unconnected fill island on the
  same net as the zone (reachable by no pad / via on the net)
  renders with a distinguishing outline (dashed magenta is the
  Allegro convention; pick something equivalent and document it).

## Interaction Contract

### Routing tools

- **Modeless routing.** Click a route button or hotkey, the cursor
  enters routing mode, all other commands stay available, Esc
  exits. Eagle-style modal routing is not the convention any more
  and Datum should not adopt it.
- **Routing mode cycle: `Shift+R`** cycles
  `Highlight Collisions → Walkaround → Push (Shove) → HugNPush`.
  The chosen mode shows in the HUD. Match Altium semantics and
  keystroke. Even if the underlying engine ships fewer modes
  initially, register the hotkey and HUD vocabulary so users
  reach for the right key.
- **Via insertion**: `V` drops a through via during routing,
  `Shift+V` drops a microvia, `Alt+Shift+V` opens a blind/buried
  picker. Match KiCad muscle memory.
- **Layer change during routing**: `+` / `-` cycles enabled signal
  layers, `PgUp` / `PgDn` jumps to F.Cu / B.Cu directly.
  Auto-via on layer change is a preference, default on.
- **Trace posture**: `/` toggles diagonal-first vs straight-first
  (KiCad muscle memory). `Spacebar` rotates corner direction
  (Altium muscle memory). Both are cheap and serve different
  mental models.
- **Differential pair routing**: `6` enters DP mode. Pair pickup
  from net-name suffix (`_P`/`_N`, `_+`/`_-`) initially, plus
  opt-in via constraint manager once Datum has a constraint
  surface.
- **Length tuning**: `7` (single trace), `8` (diff pair),
  `9` (diff pair skew). Match KiCad. Live serpentine preview;
  parameters on a right-toolbar pop-out. Length readout
  colour-bands as length approaches target (PADS pattern):
  yellow under, green at, red over.
- **Track width** comes from the active net class by default;
  override via the Tab dialog mid-route or via the
  width-pulldown in the toolbar.

### Editing tools

Two-mode drag is universal across professional tools and is
non-negotiable here.

- **`D` = Drag with rules.** PNS-aware: shoves or walks-around
  per the current routing mode. Honours track lock. Match KiCad.
- **`G` = Drag Free Angle.** Breaks 45°, ignores rules, splits
  segment if needed. The deliberate "let me touch this exactly"
  affordance.
- **`M` = Rigid move.** Preserves track angles; for non-routed
  objects (footprints, zones, text). Footprint move drags
  connected tracks (PNS-aware) by default; right-click on the
  move tool offers Rigid Move as an alternative.
- **Rip-up affordances**: per-segment (`Delete` on selection),
  per-net (right-click on net → Unroute), per-component
  (right-click on component → Unroute), per-area (rectangle
  select + `Delete`). Ship all four idioms.
- **Cleanup Tracks and Vias** dialog with options: Merge
  Collinear, Delete Dangling, Delete Tracks-in-Pads, Remove
  Redundant Vias, Refill Zones After Cleanup. Match KiCad's
  option set wholesale.
- **Glossing pass** (post-route shorten + corner-reduction). Run
  inline during routing as a preference (Off / Weak / Strong,
  per Altium). Run on demand on selection via Tools menu.

### Zone management

- **Zone creation**: an Add Filled Zone tool. Outline drawing →
  properties dialog (net, layer, fill type, thermal relief
  settings). Hotkey to be assigned — note that
  `Ctrl+Shift+Z` clashes with the universal Redo binding;
  pick a different chord or a single-letter (the airwire doc
  pattern of `N` for ratsnest cycle suggests a similar
  single-letter approach for zone tools).
- **Refill all zones**: hotkey `B`. Match KiCad muscle memory.
- **Default to fill-on-save** so the saved board never has stale
  fills. Combine with the stale-zone marker so the user sees
  unsaved-stale state in the viewport.
- **Per-zone lock** is a one-click context-menu top-level
  action. Locked zones refuse refill on the cascade.

### Selection & highlight

- **Click track segment** → segment selection. **Double-click**
  → whole-net selection. Match Altium / Allegro.
- **Backtick (`` ` ``)** highlights the net under the cursor
  (bright + dim everything else). **`Ctrl+`** toggles highlight
  without re-picking the net. Match KiCad.
- **Multi-net highlight**: `Shift+`` adds a net to the
  highlight set rather than replacing. Datum should ship this
  on day one (Altium ships it; KiCad does not, and forum
  traffic confirms users want it).
- **Selection filter** restricts mouse-drag selection by object
  class (Tracks / Pads / Vias / Zones / Footprints). Surface
  it in the left toolbar. Match KiCad / Altium.
- **Cross-probe schematic ↔ PCB**: clicking a net in either
  editor highlights it in the other. Universal in professional
  tools; required for parity.

### Snap

- Grid snap default. Configurable grid size with fine / coarse /
  user pulldown.
- Object snap to pad centre / track endpoint / via centre /
  intersection / midpoint. Per-snap-type configurable.
- `Shift` temporarily overrides snap during a drag (Pulsonix
  pattern). `G` cycles grid sizes (Altium pattern).

## Datum Differentiators

Things Datum should do that surveyed tools do not, given the
existing engine substrate. Each is justified against a specific
user pain point in the research.

1. **"Why is this pad disconnected from this zone?" diagnostic.**
   Same family as the airwire diagnostic. On shift-hover over a
   same-net pad-near-a-zone that is not connected, surface
   plain text from the connectivity engine: "This pad on F.Cu
   is separated from the GND zone on F.Cu by 0.3 mm; clearance
   rule requires 0.2 mm — should connect, but doesn't because
   zone priority 5 is shadowed by zone priority 10 on this
   region." Datum's connectivity carries this data already.
2. **Stale & orphan visible markers.** Both are rendering-only
   features (no engine cost) that remove an entire class of
   "did the fill update?" and "wait, why is there a fill island
   with no pad?" forum threads.
3. **Per-fragment zone diagnostics surfaced in a side panel.**
   For a multi-island fill, list each fragment's pad/via
   inclusion. Same-net orphans get an "Is this intentional?"
   prompt the user can dismiss.
4. **Stable copper IDs across schematic-driven net rename.** A
   track tagged to net `NET1` retains identity (and its per-net
   colour, lock state, length-tuning config) when the net is
   renamed to `DDR_CLK`. Trivial given the canonical IR's
   stable UUIDs; designers moving from name-keyed tools
   (Altium, Allegro) will appreciate it.
5. **AI-accessible copper surfaces.** The track list, via list,
   pad list, zone fragment list, fragment diagnostics, and
   route-mode HUD are all MCP-queryable. An agent should be
   able to ask "list all tracks on net `DDR_DQ0` sorted by
   length" or "explain why the +5V pour orphan fragment
   exists" without screen-scraping.
6. **One-click lock as a top-level affordance.** Per-track and
   per-zone lock surface as a context-menu top-level action
   with an adjacent inline lock icon (Horizon pattern).
   Push-and-shove honours the lock automatically. Removes the
   "PNS broke my critical net" pain point.
7. **Live zone refill with stale-marker fallback.** Default to
   live refill on edit; if the refill cost exceeds a threshold
   (huge zone, big drag), fall back to async refill + stale
   marker until the next quiet frame. Best of both Allegro
   Dynamic and KiCad explicit-refill.
8. **Print / export copper artwork with annotation.** PDF / SVG
   / PNG board snapshots from the CLI, with per-net colour,
   layer isolation, and a key/legend. Mirrors the airwire
   export recommendation.

## Out of Scope (for `M7`)

These are excluded from the opening slice and documented so the
choice does not get re-litigated under perceived gap pressure.

- **Negative plane primitive.** Positive zones only.
- **Real-time autorouter / batch autoroute.** Datum's `M5`
  routing substrate exists, but the `M7` GUI integration is
  for *interactive* routing only.
- **Curved tracks during interactive routing.** The
  track-as-arc data primitive ships in `M7` so the schema is
  correct; the interactive curved-routing tool waits for `M8`.
- **Per-segment differential-pair gap override.** Constant gap
  from rule for `M7`.
- **Stacked microvias with per-via plating-fill spec.** Data
  model carries the flag but the renderer does not visualise
  it beyond "tented / not tented".
- **3D copper visualisation.** 2D only for `M7`.

## Engine Surface Gaps

The current engine state suffices for a first-cut copper renderer
but the contract above requires the following additions before
`M7` copper work begins. Each is captured here so engine work
can be sequenced ahead of GUI integration.

1. **`Track::arc_center: Option<Point>`** so curved tracks ship
   native rather than as decomposed segments.
   (`crates/engine/src/board/board_types.rs:19`)
2. **`Track::locked: bool`** so per-track lock survives sessions
   and shove operations honour it.
3. **`Via::via_type: ViaType`** enum
   (`Through / Blind / Buried / Microvia`) so the renderer can
   distinguish via flavours and DRC can apply microvia-specific
   rules. (`crates/engine/src/board/board_types.rs:29`)
4. **`Via::tented_top: bool`, `Via::tented_bottom: bool`** so
   mask aperture rendering and fab output have authoritative
   data.
5. **`Via::locked: bool`** matching the track lock.
6. **`Zone::fill_style: FillStyle`** enum (`Solid / Hatched`)
   plus the hatch parameters when hatched.
   (`crates/engine/src/board/board_types.rs:40`)
7. **`Zone::locked: bool`** for the per-zone lock affordance.
8. **`ZoneFill { zone: Uuid, fragments: Vec<Fragment> }`**
   artifact (or a derived field on `Zone`) where each
   `Fragment { outline_path, holes[], pad_inclusion: HashMap<Uuid, ConnectionKind>, is_orphan: bool }`.
   The renderer consumes fragments; the engine owns
   computation.
9. **`PlacedPad::padstack_uuid: Option<Uuid>`** for clean ODB++
   / IPC-2581 export. (`crates/engine/src/board/pad.rs:32`)
10. **Per-net incremental query**:
    `QuerySurface::get_copper_for_nets(&[Uuid])` and
    `dirty_copper_since(version)` so the GUI rebuilds only
    what changed.
11. **`ConnectivityReason` payload** on the per-pad inclusion
    map inside each `Fragment`, populated from the existing
    connectivity engine. Feeds the
    "Why is this pad disconnected from this zone?" surface.
12. **High-contrast / dim render hints** on the engine's layer
    descriptor — let the engine tell the renderer which layers
    are inactive so the dim factor applies consistently across
    all object types in a single draw pass.

These gaps are not blocking the engine's current `M6` work; they
become required before the `M7` copper surface can land.

## Acceptance Questions

Every copper-touching change in the opening `M7` slice must
answer these. They are the operational form of the contract
above.

1. Can a designer coming from KiCad or Altium identify F.Cu and
   B.Cu by colour alone, in under one second?
2. Does the active layer indicate three-channel (panel bold +
   coloured indicator + status bar) on every screen?
3. Does Ctrl+H cycle high-contrast modes the way a KiCad user
   expects, with a configurable dim factor?
4. Does Shift+R cycle routing modes the way an Altium user
   expects, with the chosen mode visible in the HUD?
5. Do the canonical hotkeys (`V` / `D` / `G` / `M` / `B` / `7`
   / `8` / `9` / `` ` ``) work on day one with the
   industry-standard semantics?
6. Does a zone fill update visually in lockstep with edits that
   affect it, or does the stale marker appear?
7. Does an orphan fill island render distinctly from a connected
   fragment, with a side-panel entry the user can act on?
8. Does the "Why is this pad disconnected from this zone?"
   diagnostic produce a plain-text answer for any same-net
   pad-near-zone case on the canonical regression board?
9. Does deleting a track, a via, or a zone produce deterministic
   downstream changes (refill, ratsnest update) that survive a
   reload?
10. Does push-and-shove refuse to disturb a locked track, every
    time, without a setting?

If a change cannot be justified in those terms, it is design
polish or scope drift and belongs in a follow-up brief, not the
opening slice.
