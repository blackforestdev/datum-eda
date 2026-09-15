# Airwire / Ratsnest Rendering — Industry Survey

> Scope: how professional and consumer PCB tools render the unrouted-connection
> overlay (variously called airwires, ratsnest, ratlines, connection lines,
> guide lines). Intended to inform Datum EDA's M7 GUI design.

## Executive Summary

- **Convention is settled, not exciting.** Every surveyed tool draws a thin
  straight line per remaining connection, computed as a Minimum Spanning Tree
  (MST) over net terminals — usually built on a Delaunay triangulation. KiCad,
  Horizon EDA and Eagle all use this exact pattern; Altium, Allegro and PADS
  expose the same visual effect. Steiner-tree airwires are not used in
  production tools (Steiner trees are for *routing*, not *display*).
- **Per-net colour and per-net visibility are table stakes.** All
  professional tools (Altium, KiCad 6+, Allegro/OrCAD, PADS, Horizon) let the
  user assign a colour per net or per net-class and toggle visibility per net
  or per net-class. Consumer tools (Eagle, EasyEDA, DipTrace, Fusion) have
  the visibility toggle but weaker colour control.
- **Hiding GND/VCC is the universal pain point.** Every forum thread for
  every tool eventually arrives at "how do I hide ratsnest lines for ground
  and power so I can see the signal nets?" — Eagle's `RATSNEST ! GND VCC`
  command became a de-facto idiom that other tools later imitated through
  per-net checkboxes.
- **Update behaviour is the differentiator.** Two failure modes: the static
  "must press B to recompute" model (older Eagle, older Allegro) and the
  always-live "recomputed every drag tick" model (KiCad, Altium, Horizon).
  Live recompute combined with a *throttle/timer* during high-pin-count drags
  is the modern pattern; KiCad explicitly notes a timer that suspends
  recomputation during fast drag because BGA-class moves can't complete
  inside one frame.
- **"Local ratsnest" is a feature, not a corner case.** Showing only the
  unrouted connections of the currently-selected component during placement
  is the most-requested workflow across every forum surveyed. KiCad regressed
  this feature in 5.x and got months of angry threads about it; Altium spells
  it `Pad To Pad` mode, cycled with the `N` key during drag.
- **Curved (arc) ratsnest lines** are an accepted enhancement for BGAs and
  dense pin arrays — KiCad shipped them in 6.0. They are an option, never
  the default.

## Per-Tool Analysis

### Altium Designer

Altium calls the airwire a **Connection** (a `PCB_Connection` object in its
model). The unrouted overlay is the "ratsnest" colloquially.

- **Visual style:** Thin solid lines between pads, drawn point-to-point.
  When the user defines a custom From-To, *that* connection draws as a
  **dashed** line to distinguish manual topology from system-generated
  connections.
- **Color scheme:** Per-net colour (right-click a net → **Change Net Color**)
  or fall back to the global "Connection Lines" system colour. A separate
  multi-layer mode exists: enabling **Use Layer Colors for Connection
  Drawing** (View Configuration panel → View Options → Additional Options)
  draws each connection as a dashed gradient between the colour of its start
  layer and the colour of its end layer — only when the connection actually
  spans layers; intra-layer connections keep their net colour.
- **Topology:** Per-net MST decomposed into individual From-To pin-pair
  connections; the user can override topology in the **From-To Editor** in
  the PCB panel.
- **Endpoint anchoring:** Pad anchor (centre of electrical hot-spot).
- **Update behaviour:** Live during edit; during a component drag with many
  connected pads, all connection lines are auto-hidden once the count exceeds
  `PCB.ComponentDrag.ConnectionLimit` (Preferences → System → General →
  Advanced).
- **Filtering / hide:** Three modes cycled with **`N`** during component
  drag — **Hidden**, **Pad to Pad**, **Breaks** (Route to Route, i.e.
  partial-routing-aware). The chosen mode is announced in the HUD.
  Per-net hide via **Edit Net** dialog → "Hide Connections" checkbox, or
  `View » Connections » Hide Net`.
- **Single-layer mode:** "All Connections in Single Layer Mode" option
  (View Configuration → View Options) controls whether connections appear
  while you're viewing only one layer (Shift+S).
- **Length / annotation:** Length information is in the PCB panel (Nets
  view, From-To Editor) rather than drawn on the airwire itself.
- **Partial-routing:** Connections re-anchor to the routed segment endpoint
  in **Breaks** mode; **Pad to Pad** mode keeps the anchor at the original
  pad regardless of partial routing.

### KiCad (8.x / 9.x / nightly)

The most-documented implementation; algorithm is in the public source.

- **Visual style:** Default is **straight** thin lines. KiCad 6.0 added an
  **arc/curved** mode (toggle on the left toolbar — "Switches between
  straight and curved ratsnest lines"). Thickness is configurable in
  **Preferences → PCB Editor → Editing Options**.
- **Color scheme:** **Nets tab of the Appearance panel** holds per-net and
  per-net-class colour assignments (double-click colour swatch). The
  "net color mode" pulldown picks where the colour applies:
  - `None` — colours ignored
  - `Ratsnest` — only the ratsnest gets net colours (default)
  - `All` — all copper items in the net (pads, tracks, vias, zones) take
    the net colour
- **Topology:** Per-net MST built with Kruskal's algorithm
  (`RN_NET::compute()` → `RN_NET::kruskalMST()` per the Doxygen reference).
  Edges fed to Kruskal come from a Delaunay triangulation of the net's
  terminals (the standard Delaunay-MST trick: only triangulation edges are
  needed because the EMST is a subgraph of the Delaunay graph).
  `RN_NET::OptimizeRNEdges()` adjusts endpoints when zones are involved
  (snaps to the nearest point on a fill, not to the nominal pad centre);
  `RN_NET::NearestBicoloredPair()` uses a sweep-line approach for
  cross-net nearest-pair queries.
- **Endpoint anchoring:** Pad centre by default, but optimised onto the
  nearest point of a copper fill / zone fragment when the net is connected
  via a pour. This is what makes "the ratsnest line vanishes when the
  zone fills" work.
- **Update behaviour:** Live, recomputed when any connectivity event fires
  (`Compile_Ratsnest()` → `CONNECTIVITY_DATA::RecalculateRatsnest()` →
  `BOARD::OnRatsnestChanged()`). During fast drag of high-pin-count
  components a timer suspends recomputation until movement pauses to keep
  the drag responsive.
- **Layer / z-order:** Drawn on the dedicated `LAYER_RATSNEST` layer through
  `RATSNEST_VIEW_ITEM` (a `KIGFX::VIEW_ITEM` wrapping a
  `CONNECTIVITY_DATA` shared pointer); LOD-controlled
  (`lodScaleForThreshold`) so it fades out when zoomed too far.
- **Filtering / hide:** Per-net visibility checkbox in Nets tab; ratsnest
  draw mode pulldown — **All layers** (draw to items on hidden layers too)
  vs **Visible layers** (suppress airwires whose endpoint is on a hidden
  layer). Global on/off is a left-toolbar button.
- **Local ratsnest:** Separate left-toolbar button "Display local ratsnest"
  shows the airwires for only the selected footprint(s). Preference
  **"Always show selected ratsnest"** (Preferences → PCB Editor → Editing
  Options) auto-enables local ratsnest for any selection. The history of
  this feature is contentious — see "User Pain Points" below.
- **Highlight dimming:** When net highlighting is active (default hotkey
  backtick), the highlighted net stays bright and *everything else dims*;
  ratsnest follows the same rule.
- **Partial-routing:** Ratsnest is rebuilt against the current connectivity
  graph including routed track segments and zone fills, so an airwire
  always represents what's actually still missing — it stretches from the
  routed-segment endpoint, not the original pad.

Source pointers (KiCad GitLab, `pcbnew/`):
- `pcbnew/ratsnest/ratsnest_data.h/.cpp` — `RN_NET`, MST + Delaunay edge
  generation
- `pcbnew/ratsnest/ratsnest_view_item.h/.cpp` — `RATSNEST_VIEW_ITEM::ViewDraw`
- `pcbnew/ratsnest/ratsnest.cpp` — `Compile_Ratsnest` orchestration
- `pcbnew/connectivity/connectivity_algo.cpp` — BFS clustering (the input
  to ratsnest, not the ratsnest itself)
- `pcbnew/pcb_painter.cpp` — actual GAL drawing (line primitive, colour
  resolution)

### Cadence Allegro / OrCAD X Presto

Allegro's terminology is **"rats"** or **"rats nest"**.

- **Visual style:** Thin straight lines; per-net colours overridable in the
  Visibility / Color setup.
- **Color scheme:** Right-click a net (or a subsection — Groups, Diff Pairs,
  XNets, Nets) in the **Visibility Pane** to override the net's layer
  colour. Net-colour-on/off global toggle.
- **Topology:** Per-net MST.
- **Update behaviour:** Live during placement; explicit recompute commands
  exist (`Display » Show Rats`).
- **Filtering / hide:** **Display » Show Rats** menu with options:
  - `All` — show all
  - `Net` — show one selected net
  - `Component` — show rats touching one component
  - **`End in View Only`** (added in Allegro 16.6) — only draw rats whose
    *endpoint* is inside the current viewport, automatically suppressing
    "pass-through" airwires that just clutter the screen during pan/zoom.
    This is a feature no other surveyed tool has.
- **Visibility Pane (OrCAD X Presto):** Three-section pane (Layers / Nets /
  Display); the Nets section toggles ratsnest per-subsection or per-net,
  with a "rats" toggle button next to each entry. **`View » Panels »
  Visibility`** opens it.
- **Endpoint anchoring:** Pad anchor.
- **Partial-routing:** Recomputes against current connectivity.

### Mentor / Siemens Xpedition (PADS)

PADS calls them **Connections** in the data model and **ratsnest / rat lines**
colloquially. PADS' specialty is the **Pin Pair** abstraction.

- **Visual style:** Thin straight lines; per-net colour controlled in the
  Display Colors dialog (the "Connections" entry).
- **Topology:** A **Pin Pair** is a user-orderable two-pin segment of a net.
  PADS lets the user enforce a particular topology by defining/protecting
  pin pairs, and the airwire display follows that topology rather than a
  pure MST. This matters for length-matched busses and daisy-chain
  topologies.
- **Visibility:** Modeless command **`Z U`** toggles unrouted-connection
  display globally; per-net handled via the colour dialog (set Connections
  colour to "no colour" for that net to suppress it).
- **Topology protection:** Once the user is happy with the rats topology
  during placement, "Protect" locks it so subsequent edits don't re-MST.
- **Update behaviour:** Live during placement; recompute on net change.
- **Length:** PADS' Router Spreadsheet shows pin-pair routed length and
  remaining unrouted length per pair — alongside the rats display, not on
  it.

### Eagle (Autodesk)

Eagle established the term **airwire** that the rest of the industry borrowed.

- **Visual style:** Thin straight lines, drawn on a dedicated layer (the
  "Unrouted" layer, traditionally layer 19), so visibility and colour are
  controlled like any other Eagle layer.
- **Color scheme:** Layer colour for the Unrouted layer, plus per-signal
  hide via the `RATSNEST` command.
- **Topology:** Per-net MST.
- **Update behaviour:** **`RATSNEST`** (or `B`) recomputes; not always live
  in older Eagle. Modern Eagle / Fusion Electronics recomputes
  automatically (and Fusion goes further — see below).
- **Filtering / hide:** The `RATSNEST` command is the canonical interface:
  - `RATSNEST` — recompute and show all
  - `RATSNEST ! GND VCC` — hide airwires for GND and VCC (the "!" =
    suppress)
  - `RATSNEST GND` — re-show GND
  - `RATSNEST *` — un-hide everything
  - Wildcards allowed (`RATSNEST ! *GND*` hides AGND, DGND, etc.)
- **Selection:** In Fusion Electronics' Selection Filter, **Airwire** is
  off by default — Ctrl+click to enable.

### Horizon EDA

Studied directly from `research/horizon-source/`. Useful because Horizon is
the closest open-source design point for what Datum EDA is building.

Source pointers (`research/horizon-source/src/`):
- `board/airwire.hpp/.cpp` — `Airwire` is a `(from, to)` pair of
  `Track::Connection` endpoints with a `LayerRange`.
- `board/airwires.cpp` — `Board::update_airwire(net)` is the algorithm:
  1. Collect all junction and pad terminals on the net into a point set.
  2. Add zero-length airwires for terminals that share position but
     conflict on layer (cannot connect).
  3. Add edges from existing tracks (only if track layer matches both
     endpoint layers).
  4. (If `!fast`) add edges from filled plane fragments — terminals
     contained in the same fragment are considered connected.
  5. Build a Delaunay triangulation via `delaunator/` (or a sorted-linear
     fallback when points are collinear) for the remaining edge
     candidates.
  6. Combine board edges (weight `-1`, forced) + Delaunay edges (weight =
     Euclidean distance) + zero-length placeholders (weight `1e9`).
  7. Run **Kruskal MST** (`kruskalMST` with a `disjoint_set`) — kept
     edges with weight > 0 become airwires.
  8. Comment in the source explicitly says "adapted from kicad's
     ratsnest_data.cpp".
- `imp/imp_board.cpp` — `ImpBoard::update_airwire_annotation()` is the
  renderer. Draws each airwire as a single line with `ColorP::AIRWIRE`
  (palette colour 13) using a "canvas annotation" abstraction; each line
  carries an optional `color2` per-net override sourced from the
  `AirwireFilterWindow`. Filters: hidden nets, work-layer-only mode,
  layer visibility.
- `imp/airwire_filter_window.hpp/.cpp` + `airwire_filter.ui` — the
  per-net visibility/colour UI: a sortable, searchable, netclass-filterable
  TreeView with a "Show airwires" toggle column, an "Airwires count"
  column, and a colour swatch column with a context menu
  (Check / Uncheck / Toggle / Set color / Clear color). Selection in the
  list cross-highlights selected nets in the canvas.
- `canvas/shaders/triangle-ubo.glsl` — comment `color == 13 /*airwire*/`
  shows airwires share the same instanced-triangle pipeline as everything
  else; per-net `color2` override is shader-resolved.
- `imp/imp_board.cpp` line 783: `airwire_annotation->set_display(LayerDisplay(true, LayerDisplay::Mode::OUTLINE))`
  — airwires render in **outline mode** (always wireframe, never filled).

Behaviour:
- **Visual style:** Single straight line per MST edge.
- **Colour:** Default `AIRWIRE` palette colour; per-net override colour
  applied via shader `color2` channel.
- **Update behaviour:** `update_all_airwires()` rebuilds everything;
  `update_airwires(fast, nets_only)` rebuilds only changed nets, with
  `fast=true` skipping the (expensive) plane-fragment edge collection.
  Connected to `core_board.signal_rebuilt()` for live updates.
- **Filtering:** Per-net checkbox + `set_only(set<UUID>)` for "show only
  these nets" + `Reset airwire filter` action that re-shows all.
- **Highlight:** When highlights are non-empty during a tool, only
  highlighted nets render; otherwise all visible nets render.
- **JSON-persisted state:** `serialize() / load_from_json()` preserves
  per-net visibility and per-net colour across sessions.

### Consumer tools (DipTrace, EasyEDA, Fusion)

- **DipTrace:** "Ratlines" in **Design Manager → Objects tab** with a
  global toggle, plus per-net rendering. **Verification → Check Net
  Connectivity** is the formal "what is unrouted?" path. Note that polygon
  pours don't count as routed in the connectivity report — a recurring
  source of user confusion.
- **EasyEDA (Std + Pro):** Per-net checkbox in the **Design Manager**'s
  Nets folder hides individual ratlines; clicking an "incomplete net"
  highlights it. Pro has more granular `Route → Unroute` controls but the
  display model is the same.
- **Fusion 360 Electronics (Eagle successor):** Calls them airwires
  exactly like Eagle. Auto-recomputed on every PCB change (no more manual
  `B` press). Inherits Eagle's `RATSNEST ! net` CLI, plus an
  inspector-panel "Hidden" checkbox per net. **Push Violators** and
  **Walkaround Violators** modes trigger an implicit `RATSNEST` after each
  edit. **Airwire is off by default in the Selection Filter** — a
  surprising default that gets reported as a bug regularly.

(TARGET 3001! also surfaces in searches and uses identical conventions —
straight ratlines, per-signal hide.)

## Cross-Cutting Patterns

### Visual style conventions

- **Solid thin straight line** is the universal default. ~1 px on screen,
  resolution-independent (not in board units). Anti-aliased.
- **Dashed** lines are reserved for *meaning*: Altium uses dashed for
  user-defined From-Tos and for layer-spanning gradient connections; no
  tool uses dashed as the default.
- **Curved/arc** is an opt-in enhancement, only KiCad ships it in
  production. Forum sentiment is split: helpful for BGAs, distracting
  elsewhere — keep as a toggle.
- **Outline / wireframe-only** is universal: no tool fills airwires, ever.
  Horizon makes this explicit (`LayerDisplay::Mode::OUTLINE`).

### Color & per-net identity

- **Per-net colour** is mandatory. Per-net-class colour is preferred (saves
  config). KiCad and Altium are state of the art here.
- A "**color mode**" enum is the right abstraction (KiCad's
  `None / Ratsnest / All`): controls whether a net's colour applies only
  to its airwires, only to its copper, or both. Makes it possible to use
  net colours as a *placement aid* without polluting the routed appearance.
- **Layer-gradient mode** (Altium's "Use Layer Colors for Connection
  Drawing") is unique to Altium and genuinely useful for multi-layer
  routing planning — the dashed gradient tells you at a glance which layer
  pair the connection will need.
- A **checkerboard "no colour"** swatch (KiCad's UI convention) is the
  right way to indicate "this net has no override".

### Performance strategies

- **MST-of-Delaunay** is the dominant algorithm. Delaunay reduces the edge
  count from O(n²) to O(n) so Kruskal runs fast. Both KiCad and Horizon
  use this exact pipeline.
- **Per-net incremental rebuild** is universal. Touching one net should
  not recompute every other net's MST. Horizon's `update_airwires(fast,
  nets_only)` is a clean example.
- **Drag throttle / timer** during interactive drag (KiCad explicitly
  documented). Recomputing on every mouse-move event for a 200-pin BGA
  drag will stutter; suspend recompute, fall back to translating the last
  computed airwires by the drag delta until the user pauses.
- **LOD culling** (KiCad's `RATSNEST_VIEW_ITEM::lodScaleForThreshold`):
  hide ratsnest entirely below some screen-scale threshold so a zoomed-out
  view of a busy board stays usable.
- **GPU-instanced thin lines** is the natural rendering primitive — one
  shader pipeline shared with track outlines, the airwire is just a flag
  in the per-instance buffer (Horizon's `triangle-ubo.glsl` does exactly
  this).
- **Viewport-end culling** (Allegro's "End in View Only") is a clever
  middle ground for huge boards: skip drawing any airwire whose endpoint
  is off-screen.

### Interaction patterns

- **Hover/click → highlight net** is universal. Backtick (`` ` ``) is the
  KiCad hotkey, Shift-click is common elsewhere.
- **Highlight + dim everything else** is the standard visual treatment.
- **`N` key cycles drag display modes** (Altium): Hidden / Pad-to-Pad /
  Breaks. Worth copying — gives the user a fast modal control.
- **Per-net filter window** (Horizon's `AirwireFilterWindow`, KiCad's
  Nets tab, Allegro's Visibility Pane) is the standard "manage many nets
  at once" UI: searchable list, per-row visibility toggle, per-row colour
  swatch, batch operations (all on / all off / show only selected).
- **No tool lets the user *drag* an airwire to influence routing
  topology** — except indirectly via Altium's From-To Editor and PADS'
  Pin Pair protection. The airwire itself is read-only; topology is
  edited through a separate editor.
- **Length on the airwire** is rare — most tools surface remaining
  unrouted length in a side panel, not on the line itself. Worth
  considering as a Datum differentiator if it can be done without clutter
  (e.g. label only on hover, or only when single-net mode is active).
- **Click-to-trace** (clicking an airwire to start routing that
  connection) is supported in KiCad ("Route Selected") and Altium.

### Partial-routing handling

- **Modern tools rebuild the connectivity graph including routed
  segments**, so an airwire always anchors at the *current* end of the
  partial route, not the original pad. KiCad, Altium ("Breaks" mode),
  Allegro and Horizon all do this. PADS does it; Eagle does it after
  `RATSNEST` recompute.
- Altium uniquely keeps a **"Pad to Pad"** mode that ignores routed
  segments — useful for visualising original net intent during
  refactoring.
- **Zone fills count as routed** in KiCad and Horizon (terminals inside a
  filled fragment are considered connected, so airwires disappear).
  DipTrace explicitly **does not** count pours, and gets bug reports
  about it.

## Academic & Open-Source Implementation Notes

- **Steiner trees are not used for airwire display.** The Steiner Minimum
  Tree (SMT) is up to 3/2× shorter than an MST, but it requires inserting
  Steiner points (extra vertices that aren't pads) — these would have no
  meaning to the user as visual aids and would cost NP-hard compute time.
  Steiner trees belong in *routing* (global router topology), not in the
  ratsnest overlay. No surveyed tool uses Steiner for display.
- **MST-of-Delaunay is the right answer** for an EMST (Euclidean Minimum
  Spanning Tree) in 2D. The Delaunay graph is a superset of the EMST, so
  running Kruskal over Delaunay edges is both correct and O(n log n).
  Both KiCad's `RN_NET` and Horizon's `Board::update_airwire` use this.
- **"Geodesic ratsnests"** (airwires routed around obstacles) appear in
  research papers but no production tool ships them. The argument
  against is performance — recomputing geodesic paths every drag tick
  doesn't scale — and the argument for is "the airwire actually
  represents the route the autorouter will pick." This is a possible
  Datum differentiator given Datum's deterministic-routing substrate;
  see Recommendations.
- **Connectivity graph as foundation:** All tools separate the
  connectivity-cluster computation (BFS over electrical contacts: shared
  pad/track endpoints, via stacks, zone-fragment containment) from the
  airwire MST computation. KiCad makes this explicit:
  `connectivity_algo.cpp` does BFS clustering; ratsnest only runs over
  the resulting clusters that are still disconnected.

## User Pain Points & Wishlist Items

Distilled from KiCad forums, Eagle community, Altium docs, EasyEDA forum,
DipTrace forum, Cadence Community, Autodesk forums.

1. **"Hide GND/VCC ratsnest" is the #1 request, everywhere.** Every tool
   has acquired some form of per-net hide because of this single workflow.
   Eagle's `RATSNEST ! GND VCC` is the most-cited workaround across all
   forums. Datum should ship per-net visibility from day one and make it
   *easy* (one click in the net browser, a context-menu "hide ratsnest
   for this net" on any pad).
2. **"Local ratsnest while moving a component" must work, every time.**
   KiCad regressed this in 5.x and got months of angry threads ("makes
   the tool useless"). Altium's `N`-key cycle is the gold standard.
   Datum should have it as a default-on behaviour, with the `N`-key
   cycle as a familiar shortcut.
3. **Ratsnest still visible after track is drawn.** Bug
   `KiCad #1811010` and many duplicates. The user-visible symptom is "I
   connected this, why is the line still there?" — almost always caused
   by a connectivity algorithm not recognising a zone or via stack as
   completing the connection. Datum's connectivity must be authoritative
   and must recompute in lockstep with edits, *or* the airwire must be
   visibly marked as stale.
4. **Ratsnest on hidden layers.** Users want airwires for nets whose
   endpoints are on hidden layers to either disappear (so I can focus on
   what I'm working on) or stay visible (so I can see what I'm about to
   need). KiCad's "All layers" / "Visible layers" toggle handles both
   camps. Ship both.
5. **Curved ratsnest for BGAs, straight everywhere else.** The KiCad
   curved-line debate suggests this should be a per-display preference,
   not a per-net property, and definitely not a default. Some users want
   it; others find it distracting.
6. **Print/export with airwires visible.** Recurring ask: "I want to
   print my unrouted board to share with a colleague who'll review
   placement." All tools eventually added this; Datum should have a
   print/export mode that includes the ratsnest layer.
7. **"Show ratsnest of selected item" toggle.** A frequently-requested
   refinement of "local ratsnest" — when I select a pad or a net, show
   only that net's airwires, dim everything else. KiCad has it as a
   preference; should be default-on for Datum.
8. **Net colour assignment that survives net renumbering.** A subtle
   request from Allegro and Altium users: persistent per-net colour
   should follow the net through schematic edits, not get reset whenever
   the net is reassigned. Datum's stable IDs make this trivial — surface
   it.
9. **"Why is there an airwire to a fill?"** Recurring DipTrace and KiCad
   confusion when the user thinks a copper pour completes a connection
   but it doesn't (no thermal connection, wrong layer, plane fragment
   doesn't actually contain the pad). Datum can do better here: surface
   *why* a connection is still needed when hovering an airwire ("not
   connected because: pad on layer 3, zone on layer 1").
10. **Performance with 100k+ pin designs.** Multiple Allegro, Altium and
    KiCad threads about ratsnest update lag on enterprise boards. The
    solution is the standard cocktail: per-net incremental rebuild, drag
    throttle, LOD-cull at low zoom, viewport-end cull (Allegro's "End
    in View Only" pattern).

## Recommendations for Datum EDA

Concrete guidance for the M7 GUI airwire implementation. References to
existing engine state (`Board::airwires`, etc.) come from the brief
codebase scan in `crates/engine/src/board/`.

**Algorithm (engine side, mostly already present per spec):**
1. Use **MST-of-Delaunay** as the algorithm. It's the consensus answer
   and Horizon's `airwires.cpp` is a clean reference implementation
   (acknowledged-in-comment Kicad-derived). Implement Kruskal with a
   union-find / disjoint-set; feed it Delaunay edges (Euclidean weight)
   plus zero-cost "forced" edges from already-routed tracks/zones.
2. **Run per-net, incrementally.** Provide a `dirty_nets` set on the
   engine API; the GUI subscribes and rebuilds only those nets. Mirror
   Horizon's `(fast, nets_only)` parameterisation: `fast` skips
   plane-fragment edge collection during interactive drag.
3. **Snap endpoints to zone fragments**, not pad centres, when the net
   has a fill. This is what makes "the line vanishes when the pour
   covers the pad" feel correct.
4. **Make connectivity authoritative.** Datum's deterministic-routing
   substrate already has a connectivity graph; the airwire MST must run
   over the same data so there's never a "phantom airwire after I
   routed it" bug.

**Renderer (GUI side, M7):**
5. **Single thin antialiased line** per MST edge. ~1.0–1.5 px on-screen.
   Use the same instanced-line pipeline used for thin track outlines —
   airwires are just an extra colour/flag bit per instance (Horizon's
   pattern in `triangle-ubo.glsl`).
6. **Outline-only**, never filled. No glow, no shadow, no fade
   gradient by default. Sober defaults; users have strong opinions and
   most of them prefer austere airwires.
7. **Dedicated "ratsnest" virtual layer** at the top of the z-order
   above copper but below selection/hover overlays. LOD-culled below a
   minimum on-screen scale.
8. **Per-net + per-net-class colour**, with a `color_mode` enum:
   `None / Ratsnest / All`. Default to `Ratsnest` (matches KiCad). Show
   a checkerboard swatch for "no override" (KiCad UI idiom).
9. **Optional curved/arc mode** as a global preference. Implementation:
   render each MST edge as a quadratic arc with deflection proportional
   to edge length and a per-net hash-based sign so neighbouring arcs
   don't overlap. Default off.

**Filtering & visibility:**
10. **Per-net visibility checkbox** in a Net Browser panel
    (sortable/searchable list — copy Horizon's `AirwireFilterWindow`
    UX wholesale). Persist across sessions in project metadata. Add
    an explicit GND/VCC quick-hide affordance ("Hide power nets") to
    reduce the most-common friction.
11. **`N`-key drag-mode cycle** during component move:
    `Hidden / Pad-to-Pad / Breaks`. Match Altium semantics and
    keystroke; users coming from any other tool will recognise it.
12. **Show-selected-only toggle** as a preference: when on, selecting a
    pad/net/component shows only the relevant airwires and dims the
    rest. Default on. Match KiCad's "Always show selected ratsnest"
    semantics.
13. **All layers / Visible layers** mode for layer-aware filtering.
14. **Viewport-end culling** as a low-priority optimisation
    (Allegro's pattern) — only worth it for boards with thousands of
    nets; gate behind a perf preference.

**Interaction & semantics:**
15. **Hover an airwire → tooltip with net name + remaining length +
    layers**. Click → highlight the whole net. Right-click → context
    menu (`Hide this net's ratsnest`, `Set net colour`, `Route this
    connection`).
16. **"Why is this airwire here?" diagnostic on shift-hover.** A real
    Datum differentiator: explain in plain text what the engine thinks
    is missing ("This pad on F.Cu is not connected to that pad on B.Cu;
    no via stack reaches both"). Leverages Datum's existing
    connectivity diagnostics.
17. **Length annotation on long-running airwires only** (e.g. when net
    is single-selected, or when the airwire spans more than some
    distance). Avoid permanent labels — they clutter.
18. **Print / export with ratsnest** as an explicit option. Frequently
    requested across tools.

**Things to skip (for now):**
- Steiner-point ratsnest display. Keep Steiner inside routing topology,
  not display.
- Live geodesic ratsnest (airwires routed around obstacles). Tempting
  given Datum's deterministic router substrate, but the perf cost is
  high and no production tool ships it. Could be an opt-in
  "preview-routed airwire" mode in M8 once the GUI is solid.
- User-draggable airwires that influence topology. Altium and PADS
  handle this through dedicated From-To / Pin-Pair editors, not by
  manipulating the airwire directly. Datum should follow the same
  separation of concerns.
- Curved ratsnest as default. Ship the toggle, default off.

## Sources

KiCad documentation & source:
- [KiCad 8.0 PCB Editor manual](https://docs.kicad.org/8.0/en/pcbnew/pcbnew.html) — Nets tab, ratsnest options, per-net colours, curved/straight toggle, "All layers" / "Visible layers" modes
- [KiCad 9.0 PCB Editor manual](https://docs.kicad.org/9.0/en/pcbnew/pcbnew.html) — current behaviour
- [RN_NET class doxygen](https://docs.kicad.org/doxygen/classRN__NET.html) — Kruskal MST, `OptimizeRNEdges`, `NearestBicoloredPair`
- [RATSNEST_VIEW_ITEM doxygen](https://docs.kicad.org/doxygen/classRATSNEST__VIEW__ITEM.html) — view-item integration, LOD
- [ratsnest_view_item.cpp doxygen](https://docs.kicad.org/doxygen/ratsnest__view__item_8cpp.html) — includes pcb_painter, GAL
- [ratsnest.cpp doxygen](https://docs.kicad.org/doxygen/ratsnest_8cpp.html) — `Compile_Ratsnest` orchestration
- [KiCad GitLab: pcbnew/ratsnest tree](https://gitlab.com/kicad/code/kicad/-/tree/master/pcbnew/ratsnest) — algorithm + view-item sources
- [Bug 1766597: Curved ratsnest history](https://bugs.launchpad.net/bugs/1766597) — why arcs were added; BGA motivation
- [Forum: Local ratsnest discussion](https://forum.kicad.info/t/local-ratsnest-do-you-use-it/4901) — usage patterns
- [Forum: Selectively show ratsnest](https://forum.kicad.info/t/selectively-show-ratsnest/12591) — per-net selective hide patterns
- [Forum: Ratsnest thickness](https://forum.kicad.info/t/ratsnest-thickness/47554) — preference path
- [Forum: Ratsnest Coloring](https://forum.kicad.info/t/ratsnest-coloring/17375) — per-net colour history

Altium documentation:
- [Altium: PCB Connection object](https://www.altium.com/documentation/altium-designer/pcb-obj-connectionconnection-ad) — connection model, dashed user-defined lines, From-To, gradient layer mode
- [Altium KB: Control appearance of connection lines](https://www.altium.com/documentation/knowledge-base/altium-designer/control-appearance-of-lines-connecting-same-net-objects) — `N`-key cycle, View Configuration paths
- [Altium: Routing the PCB](https://www.altium.com/documentation/altium-designer/routing-the-pcb) — From-To Editor, connection topology
- [Altium: Net Colors article](https://resources.altium.com/p/breaking-the-visual-barrier) — per-net colour workflow
- [Altium: Using Net Highlight Color](https://www.altium.com/documentation/altium-designer/using-net-highlight-color-schematics-pcb) — highlight + dim semantics

Cadence Allegro / OrCAD:
- [Cadence blog: Allegro 16.6 Ratsnest Display Option](https://community.cadence.com/cadence_blogs_8/b/pcb/posts/what-s-good-about-allegro-pcb-editor-new-ratsnest-display-option-check-out-16-6) — `End in View Only` mode
- [Cadence blog: OrCAD X Visibility Pane](https://resources.pcb.cadence.com/blog/2024-navigating-the-visibility-pane-in-orcad-x-presto-pcb-editor) — Visibility Pane, per-net colour overrides
- [Cadence Community: Change rats colour](https://community.cadence.com/cadence_technology_forums/pcb-design/f/pcb-design/11502/how-to-change-the-rats-net-connection-wire-color-in-allegro-pcb-editor) — colour override discussion
- [Cadence Community: Hiding rats per net](https://community.cadence.com/cadence_technology_forums/pcb-design/f/pcb-design/22135/is-there-a-way-to-make-the-rat-lines-of-certain-nets-disappear) — per-net hide patterns
- [EMA-EDA: Lesson 9 Advanced Placement](http://education.ema-eda.com/iTrain/PCBEditor163/lesson_9.html) — Show Rats menu paths

Eagle / Fusion Electronics:
- [EAGLE Help: RATSNEST command](https://web.mit.edu/xavid/arch/i386_rhel4/help/75.htm) — canonical reference for `!` syntax, wildcards
- [Autodesk KB: Hide GND/VCC airwires in Eagle](https://www.autodesk.com/support/technical/article/caas/sfdcarticles/sfdcarticles/How-to-not-show-airwires-to-GND-or-VCC-in-EAGLE.html) — exact syntax
- [Autodesk: Cannot select airwire in Fusion Electronics](https://www.autodesk.com/support/technical/article/caas/sfdcarticles/sfdcarticles/Cannot-select-or-highlight-airwire-in-Fusion-Electronics-PCB-layout.html) — Selection Filter default
- [Autodesk Forum: Hide nets in Fusion ratsnest](https://forums.autodesk.com/t5/fusion-electronics-forum/hide-certain-nets-in-ratsnest-airwires-unrouted-layer/td-p/13824106) — per-net hide
- [Autodesk: Where is Ratsnest in Fusion?](https://forums.autodesk.com/t5/eagle-forum/where-is-the-ratsnest-function-in-fusion/td-p/10206914) — auto-recompute behaviour

PADS / Xpedition:
- [Mentor PADS Layout Tutorial PDF](http://www.theky22.com/downloads/pads%20layout%20tutorial%20-%20sgi%20-%20pcb%20design.pdf) — Connections, Pin Pairs, Display Colors
- [PADS Layout User Guide](https://www.freecalypso.org/pub/CAD/PADS/pdfdocs/padslayout_ref.pdf) — modeless commands, ratsnest behaviour
- [Pin Pair group discussion](https://groups.google.com/g/pads-user-group/c/sAiddfZfX6c) — pin-pair ratsnest topology

Consumer tools:
- [DipTrace forum: How to find unrouted nets](https://diptrace.com/forum/viewtopic.php?t=10803) — Design Manager Objects tab
- [DipTrace forum: Hide ratsnest temporarily](https://diptrace.com/forum/viewtopic.php?t=2007) — global toggle
- [EasyEDA Pro: Unroute](https://prodocs.easyeda.com/en/pcb/route-unroute/) — per-net hide
- [EasyEDA: Hide GND ratlines](https://easyeda.com/forum/topic/Hide-Ground-GND-ratlines-during-layout-e53156a975d74e86a8f75cf493bf3d31) — workflow
- [Quadcept: About Rats](https://www.quadcept.com/en/manual/pcb/post-97) — solid same-layer / dotted cross-layer convention
- [TARGET 3001: Airwires/Ratsnest wiki](https://server.ibfriedrich.com/wiki/ibfwikien/index.php?title=Airwires_/_Ratsnest) — terminology

Algorithms & academic:
- [Steiner Tree problem (Wikipedia)](https://en.wikipedia.org/wiki/Steiner_tree_problem) — why Steiner is for routing not display
- [Robins & Zelikovsky: Minimum Steiner Tree Construction](https://www.cs.virginia.edu/~robins/papers/Steiner_chapter.pdf) — VLSI/PCB context for Steiner
- [Randomized HyperSteiner (arxiv 2510.09328)](https://arxiv.org/abs/2510.09328) — recent Delaunay-based Steiner heuristic; relevant for future routing topology, not airwire display

Horizon EDA (sources read directly from `research/horizon-source/`):
- `src/board/airwire.hpp`, `src/board/airwire.cpp`, `src/board/airwires.cpp`
- `src/imp/imp_board.cpp`, `src/imp/imp_board.hpp`
- `src/imp/airwire_filter_window.hpp/.cpp`, `src/imp/airwire_filter.ui`
- `src/canvas/shaders/triangle-ubo.glsl` (per-instance colour override)
- [Horizon EDA on GitHub](https://github.com/horizon-eda/horizon)

Forum threads cited for user pain points:
- [KiCad: Hide GND ratsnest](https://forum.kicad.info/t/hide-gnd-ratsnest/19279)
- [KiCad: Turn off GND airwires](https://forum.kicad.info/t/turn-off-gnd-airwires/3791)
- [KiCad: How can I hide GND ratsnest](https://forum.kicad.info/t/how-can-i-hide-gnd-ratsnest-or-assign-parts-of-ratsnest-to-different-layers/16637)
- [KiCad bug 1811010: ratsnest still shown after routing](https://bugs.launchpad.net/kicad/+bug/1811010)
- [KiCad bug 593962: ratsnest colouring/visibility](https://bugs.launchpad.net/bugs/593962)
- [KiCad bug 1740156: ratsnest display options wonky](https://bugs.launchpad.net/kicad/+bug/1740156)
- [KiCad bug 1821183: Show local ratsnest when footprint is moving](https://bugs.launchpad.net/kicad/+bug/1821183)
- [KiCad bug 1826635: Toggle "Always show selected ratsnest"](https://bugs.launchpad.net/kicad/+bug/1826635)
- [Element14: Hide GND airwires in Eagle](https://community.element14.com/products/eagle/f/eagle-user-support-english/45776/how-do-i-hide-the-gnd-air-wires)
- [GroupDIY: Hide airwires Eagle tip](https://groupdiy.com/threads/tip-for-eagle-hide-airwires-for-specific-nets.58334/)
- [EDABoard: Altium ratsnest for ground not visible](https://www.edaboard.com/threads/altium-10-ratsnest-for-ground-net-is-not-visiable.264602/)
