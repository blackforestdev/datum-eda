# M7 Airwire Rendering Guidance

> **Status**: Active design guidance for the unrouted-connectivity lane in
> the opening `M7` board-review viewport.
> **Derived from**: `research/airwire-rendering/AIRWIRE_RENDERING_RESEARCH.md`
> (industry survey of Altium, KiCad, Allegro/OrCAD, PADS, Eagle/Fusion,
> Horizon EDA, DipTrace, EasyEDA, Quadcept, TARGET 3001).
> **Anchors**: extends the unrouted lane defined in
> `M7_RENDER_SEMANTIC_CONTRACT.md`. Engine surface today: `Airwire` struct
> in `crates/engine/src/board/board_info.rs:39` and
> `QuerySurface::get_unrouted` in `crates/engine/src/api/query_surface.rs:24`.

## Purpose

Lock the airwire (ratsnest) rendering contract for `M7` so engine, renderer,
and interaction work converge on industry-consistent behaviour without
re-litigating each decision per surface.

This is not a style memo. It is a behavioural contract: every item below is
either (a) a settled industry convention Datum must honour to be readable
to incoming users, or (b) an explicit Datum-specific differentiator.

## Hard Requirements

These are non-negotiable. The survey shows zero production tools deviate
from them, and forum traffic confirms users notice immediately when they
are wrong.

1. **Per-net Minimum Spanning Tree over a Delaunay triangulation.** Not
   nearest-neighbour. Not pin-order. Not Steiner. The MST-over-Delaunay
   pipeline is what every production tool ships and is what the Horizon
   reference (`research/horizon-source/src/board/airwires.cpp`,
   acknowledged-derivative of KiCad's `RN_NET`) uses.
2. **Connectivity authority.** The connectivity graph that drives the
   airwire MST must be the same graph that ERC/DRC and routing read.
   Phantom airwires after a successful route are the most-cited bug class
   in KiCad history (`#1811010` and duplicates). One source of truth.
3. **Per-net visibility.** Hiding GND/VCC ratsnest is the universally
   most-requested workflow across every tool surveyed. Ship per-net
   visibility on day one, with a dedicated quick-hide affordance for
   power nets.
4. **Per-net colour.** Every professional tool (Altium, KiCad 6+,
   Allegro/OrCAD, PADS, Horizon) ships per-net or per-net-class colour
   override. Consumer tools without it are routinely cited as inferior.
5. **Live recompute on connectivity change**, with a drag throttle. No
   manual `B`-to-recompute model. KiCad explicitly documents a timer
   that suspends recomputation during high-pin-count drags; copy that.
6. **Endpoint anchoring snaps to zone-fragment containment**, not pad
   centres, when the net is connected via a fill. This is what makes
   "the airwire vanishes when the pour covers the pad" feel correct.
   Without it, users report ghost airwires over filled planes.
7. **Outline / wireframe only.** No filled airwires. No glow, no shadow,
   no fade gradient by default. Horizon makes this explicit
   (`LayerDisplay::Mode::OUTLINE`); every other tool follows the same
   convention implicitly.

## Algorithm Contract (Engine)

The engine owns airwire computation. The GUI does not run geometry
algorithms over net topology — it consumes a stable list of airwire
segments per net.

- **MST algorithm**: Kruskal with union-find / disjoint-set. Edge
  candidates come from Delaunay triangulation of the net's terminal
  points (Euclidean weight) plus zero-weight forced edges from
  already-routed track segments and zone-fragment containment. Kept
  edges with positive weight become emitted airwires.
- **Per-net incremental rebuild**. Recomputing every net on every edit
  does not scale. The engine must accept a `dirty_nets` set and rebuild
  only those nets. Mirror Horizon's `(fast, nets_only)` parameterisation:
  `fast` skips plane-fragment edge collection during interactive drag.
- **Endpoint snapping to zone fragments**. When a terminal sits inside a
  filled zone fragment, snap the airwire endpoint to the nearest point
  on the fragment edge for *display*, not the pad centre. This is the
  KiCad `OptimizeRNEdges` behaviour and is required for visual
  correctness over poured planes.
- **Layer-aware terminal grouping**. Terminals on layers that cannot
  electrically reach each other (no via stack covers both) emit a
  zero-length placeholder airwire so the user sees the unconnectability,
  rather than the connection being silently absent. Horizon's
  `update_airwire` step 2 documents this pattern.
- **Determinism**. Airwire output must be deterministic across runs for
  a given board state. This is a Datum-wide invariant; airwires
  inherit it. Stable sort emitted segments by `(net_uuid, from_uuid,
  to_uuid)` so screenshot goldens and JSON exports are diffable.
- **No Steiner points in the airwire output.** Steiner is a routing
  topology concept and would inject vertices the user has no semantic
  for. Steiner stays inside the global router, never the ratsnest.

## Renderer Contract (GUI)

The renderer consumes engine airwire output and is responsible only for
visual presentation, never for topology decisions.

- **Single thin antialiased line per MST edge.** Target ~1.0–1.5 px
  on-screen, resolution-independent (not in board units). Same
  instanced-line pipeline as thin track outlines; airwires are an extra
  flag/colour bit in the per-instance buffer (Horizon's
  `triangle-ubo.glsl` pattern). A tiny filled endpoint anchor is allowed
  to visually couple the line to the pad terminus, but it must remain
  subordinate to the line and must never read as a via/drill glyph.
- **Dedicated `unrouted` virtual lane in the layer stack.** Drawn above
  authored copper, below proposed-review overlays and diagnostic
  emphasis. Position 5 in the stack defined by
  `M7_RENDER_SEMANTIC_CONTRACT.md § Stack Rule`.
- **LOD culling.** Suppress the entire airwire layer below a minimum
  on-screen scale threshold so a zoomed-out view of a busy board stays
  usable. Match KiCad's `RATSNEST_VIEW_ITEM::lodScaleForThreshold`.
- **Colour resolution order**: per-net override → per-net-class override
  → global airwire palette colour. Default to the global colour; a
  checkerboard swatch in any UI surface indicates "no override". Opening
  `M7` may satisfy the per-net tier with deterministic per-net colours
  derived from stable net UUIDs until an explicit user override surface lands.
- **`color_mode` enum** controls where per-net colour applies:
  `None` (ignore all per-net colours), `Ratsnest` (apply only to
  airwires), `All` (apply to airwires + tracks + vias + zones for that
  net). Default `Ratsnest`. Matches KiCad semantics exactly so users
  carry over muscle memory.
- **Curved/arc mode** as a global on/off preference. Default off.
  When on, render each MST edge as a quadratic arc with deflection
  proportional to edge length and a per-net hash-based sign so
  neighbouring arcs do not stack. Useful for BGAs, distracting
  elsewhere; ship it as opt-in.
- **No dashed lines by default.** Dashed is reserved for *meaning*:
  user-defined topology overrides (Altium pattern) or layer-spanning
  gradient mode if Datum chooses to ship it. Never for plain airwires.
  The default `M7` airwire must be a thin solid line specifically to avoid
  the "sticks and bubblegum" appearance that comes from short dashed spans.

## Filtering & Visibility Contract

A Net Browser panel is the canonical surface for managing airwire
visibility. Modelled directly on Horizon's `AirwireFilterWindow`.

- **Searchable, sortable, net-class-filterable list of nets.** One row
  per net; columns: visibility checkbox, airwire count, colour swatch
  (with context menu for set/clear), net name, net class.
- **Batch operations**: check all / uncheck all / show only selected /
  reset filter. Power-net quick-hide is a top-level button
  (`Hide GND/VCC` or similar) — not buried in a context menu.
- **Per-net visibility persists in project metadata.** Stable net IDs
  make this trivial; user expectation (per Allegro/Altium forum
  threads) is that hiding a net survives schematic edits.
- **`N`-key drag-mode cycle** during component move:
  `Hidden / Pad-to-Pad / Breaks`. Match Altium semantics and keystroke
  exactly. The chosen mode shows in the HUD. Users coming from any
  professional tool will recognise it.
- **"Show selected only" toggle** as a preference. When on, selecting a
  pad / net / component shows only the relevant airwires and dims the
  rest. Default on. Match KiCad's `Always show selected ratsnest`.
- **Layer-aware filter** with two modes: `All layers` (draw airwires
  whose endpoints sit on hidden layers) vs `Visible layers` (suppress
  them). Both camps exist in user populations; ship both. Default
  `Visible layers`.
- **Viewport-end culling** as an optional perf mode (Allegro's
  `End in View Only`): only draw airwires whose at least one endpoint
  is inside the current viewport. Off by default; gate behind a perf
  preference for very large boards.

## Interaction Contract

- **Hover** an airwire → tooltip with net name, remaining unrouted
  length, layer-pair of endpoints. No permanent labels.
- **Click** an airwire → highlight the whole net (highlight + dim
  everything else). Same hotkey as net-name highlight elsewhere; do
  not invent a separate "airwire highlight" mode.
- **Right-click** an airwire → context menu: `Hide this net's
  ratsnest`, `Set net colour…`, `Route this connection`,
  `Why is this airwire here?`.
- **Click-to-route**: clicking an airwire while a router tool is
  active should start routing that connection. KiCad and Altium both
  ship this; users expect it.
- **Airwires are read-only.** Users do not drag airwires to influence
  topology. Topology overrides flow through a separate editor surface
  (Altium's From-To Editor, PADS' Pin Pair window). Datum follows the
  same separation; the ratsnest layer is presentation, not editing.
- **Length annotation** appears only when (a) a single net is
  selected, or (b) an airwire is hovered. Never permanent, never per
  segment in dense views. Avoids clutter that all surveyed tools
  explicitly avoid.

## Datum Differentiators

Things Datum should do that the surveyed tools do not, given the
existing engine substrate. Each is justified against a specific user
pain point in the research.

1. **"Why is this airwire here?" hover diagnostic.** On shift-hover
   (or context-menu invocation), surface a plain-text explanation from
   the connectivity engine: "This pad on `F.Cu` is not connected to
   that pad on `B.Cu`; no via stack reaches both layers" or "Zone fill
   on `In1.Cu` excludes this pad because of thermal-relief setting".
   Targets the recurring DipTrace/KiCad confusion ("why is there an
   airwire to my pour?") that no tool currently explains. Datum's
   connectivity graph already carries the data; this is a presentation
   feature.
2. **Stale-airwire visible marker.** If the engine cannot recompute in
   time (high-pin-count drag, big paste), airwires render with an
   explicit "stale" treatment (dimmed, dashed, or with a small clock
   marker) until the next recompute lands. Removes the entire class of
   "is this airwire real or am I seeing a stale frame?" bugs that
   plague KiCad and Altium.
3. **Net-colour stability across schematic edits.** Per-net colour
   overrides bind to stable net UUIDs, not net names. A net renamed
   from `NET1` to `CLK_OUT` keeps its colour. Surfaced in the research
   as a recurring Allegro/Altium frustration; trivial for Datum given
   the canonical IR's stable IDs.
4. **Print / export with ratsnest.** Datum's headless-first design
   makes ratsnest export a query-surface affair — emit airwires into
   PDF / SVG / PNG board snapshots from CLI. Recurring forum ask
   across every tool; cheap for Datum to ship.
5. **AI-accessible airwire surface.** Per Datum's AI-first contract,
   the airwire list and its diagnostics must be MCP-queryable. An
   agent should be able to ask "which nets still have unrouted
   connections, sorted by remaining length" and "explain why net N
   shows an airwire" without screen-scraping the GUI.

## Out of Scope (for `M7`)

These are tempting given Datum's substrate but excluded from the
opening slice. Each is documented so the choice does not get
re-litigated under perceived gap pressure.

- **Steiner-point airwire display.** Steiner stays inside routing
  topology. No production tool uses Steiner for the ratsnest overlay
  and there is no user demand for it.
- **Live geodesic ratsnest** (airwires routed around obstacles).
  Tempting given Datum's deterministic-routing substrate, but the
  per-frame compute cost is high and no production tool ships it.
  Revisit as an opt-in `preview-routed-airwire` mode in `M8` once the
  baseline GUI is solid.
- **User-draggable airwires.** Topology editing flows through a
  dedicated editor surface, not by dragging the airwire itself. Out of
  scope until a dedicated From-To / Pin-Pair editor lands (post-`M7`).
- **Curved ratsnest as default.** Ship the toggle, default off.

## Engine Surface Gaps

The current `Airwire` struct
(`crates/engine/src/board/board_info.rs:39`) carries
`net`, `net_name`, `from`/`to` `NetPinRef`s, positions, and
`distance_nm`. That is sufficient for a first-cut renderer but the
contract above requires the following additions before `M7` ratsnest
work begins. Each is captured here so the engine work can be sequenced
ahead of GUI integration.

1. **Per-net incremental query.** `QuerySurface::get_unrouted` returns
   the full set today. Add a `get_unrouted_for_nets(&[Uuid])` and a
   `dirty_nets_since(version)` cursor so the GUI can rebuild only what
   changed.
2. **Endpoint layer hint.** Each `Airwire` should carry the layer of
   its `from` and `to` terminals so the renderer can implement the
   `Visible layers` filter mode and the layer-aware tooltip.
3. **Endpoint kind.** Distinguish pad-anchored endpoints from
   zone-fragment-snapped endpoints (and the zero-length unconnectable
   placeholder case). The renderer needs this to draw the
   stale/unconnectable markers from the differentiator section above.
4. **MST-edge identity.** Each airwire needs a stable identity within
   its net so per-airwire interaction (hover, click, "route this
   connection") survives a recompute that does not change topology.
5. **Diagnostic payload.** A `why_unrouted: ConnectivityReason`
   attachment, populated from the existing connectivity engine, that
   feeds the "Why is this airwire here?" surface. Optional / lazy if
   compute cost is non-trivial.

These gaps are not blocking the engine's current `M6` work; they
become required before the `M7` GUI ratsnest surface can land.

## Acceptance Questions

Every airwire-touching change in the opening `M7` slice must answer
these. They are the operational form of the contract above.

1. Does the airwire render show only and exactly the connections the
   engine considers unrouted, with no phantom segments after a
   successful route?
2. Can the user hide GND and VCC airwires in one obvious action and
   have that survive across sessions?
3. Does selecting a component during placement immediately update the
   visible airwires for that component's nets, without a frame stall
   on high-pin-count parts?
4. Does turning off a layer correctly suppress (or correctly retain,
   per the layer filter mode) airwires anchored on that layer?
5. When a pour fills over a pad, does the airwire to that pad
   disappear without a manual recompute?
6. Can a user coming from KiCad or Altium find per-net colour and the
   `N`-key drag cycle without consulting documentation?
7. Does the "Why is this airwire here?" diagnostic produce a
   plain-text answer for any airwire on the canonical regression
   board?

If a change cannot be justified in those terms, it is design polish
or scope drift and belongs in a follow-up brief, not the opening
slice.
