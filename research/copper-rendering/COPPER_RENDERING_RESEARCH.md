# Copper Layers — Rendering, Creation & Editing — Industry Survey

> Scope: how professional and consumer PCB tools render the copper-layer
> object family (tracks, vias, pads, zones/pours), what interactive tools
> they expose for *creating* that geometry (routing, placement, pour
> drawing), and what *editing* tools they expose for mutating it
> (drag/slide, ripup, gloss/cleanup, length tuning). Intended to inform
> Datum EDA's M7 GUI design.
>
> Companion to `research/airwire-rendering/AIRWIRE_RENDERING_RESEARCH.md`,
> which covered only the unrouted overlay. This document covers the
> authored copper that the airwire overlay exists on top of.

## Executive Summary

- **Copper is the load-bearing visual surface.** Whatever else a PCB tool
  draws — silkscreen, mask, courtyards, ratsnest — copper is what the
  reviewer is actually looking at. Every surveyed tool treats copper as
  the dominant lane and tunes the rest down.
- **The rendering primitives are settled.** Tracks are stroked round-cap
  capsules. Vias are concentric circles (annulus + drill). Pads are
  filled polygons (rect / roundrect / oval / circle / custom). Zones are
  filled-polygon-with-knockouts (or hatched-line patterns). Nobody
  deviates from this set; the differences are colour, opacity, and
  layer-ordering policy.
- **Track stroke style is universally round-capped.** Flat-end tracks
  exist only in old Eagle and as an explicit footgun in KiCad — every
  modern tool draws tracks as line segments with hemispherical end caps,
  matching what the photoplotter actually produces. Mitred vs rounded
  *corners* are a separate choice and most tools default to mitred
  (chamfered) at 45°.
- **Layer-colour convention is informal but real.** Red = front copper,
  blue = back copper is the default in Altium, KiCad, OrCAD/Allegro,
  Eagle/Fusion, and most consumer tools. Inner layers vary widely
  (KiCad uses yellow/cyan/magenta/violet; Altium uses cyan/grey/yellow;
  Allegro uses theme-specific). There is no IPC standard, but a
  designer who switches tools expects red-on-top, blue-on-bottom
  immediately.
- **Per-net colour is mainstream; per-layer colour with high-contrast
  dim is the working mode.** Every professional tool (Altium, KiCad,
  Allegro/OrCAD, PADS) ships per-net colour override and a "single
  layer mode" that dims or hides everything not on the active layer.
  KiCad calls it High Contrast (Ctrl+H, three modes: normal/dimmed/hidden).
  Altium calls it Single Layer Mode (Shift+S, three modes:
  full/grayscale-others/hide-others). Both ship a configurable dim
  factor.
- **Push-and-shove is the modern routing standard.** KiCad's PNS (a
  CERN-funded extraction of an academic-style topology-aware shove
  router, originally architected by Tom Włostowski), Altium's Situs +
  ActiveRoute, OrCAD X Presto's Assisted (Hug/Shove) mode, PADS'
  push-and-shove, and Pulsonix push-aside all converge on the same UX:
  cycle modes with a single hotkey (Shift+R in Altium, E in KiCad's
  PNS — modes are "Highlight Collisions / Shove / Walk Around"),
  insert vias on layer change with V, and drop the trace with click /
  Enter. Eagle ships push-and-shove only in Fusion Electronics,
  inherited from the post-2017 rewrite.
- **Drag has two modes everywhere.** "Drag with router rules" (KiCad D,
  Altium Ctrl+drag, Allegro Slide) honours push/shove and
  rule-following; "Drag free angle" (KiCad G, Altium Move/Drag of an
  endpoint) breaks 45° and ignores rules. Datum users coming from any
  of these tools will reach for a two-mode drag affordance.
- **Zone fills are recomputed on demand, not live.** KiCad fills with B
  (unfills with Ctrl+B), Eagle/Fusion fills with `RATSNEST` /
  recompute-polygon, EasyEDA's hotkey is Shift+B, Altium runs
  Tools » Polygon Pours » Repour. Allegro's *dynamic* shapes are the
  exception — they refill live during track edits (the difference vs
  *static* shapes is one of the strongest Allegro wins, and the rest
  of the industry is moving toward live fills).
- **Negative vs positive plane representation is a real architectural
  fork.** Allegro and PADS historically use negative planes (etch
  artwork = absence of copper, anti-pad/thermal padstack entries
  control voids). KiCad, Altium, Horizon, Eagle/Fusion, EasyEDA,
  DipTrace use positive zones (copper polygon with knockouts
  computed at fill time). The data-model implication is large; the
  visual difference is mostly invisible to the user once filled.
  Datum should pick one and stick with it (the recommendation below
  is positive zones).
- **Length tuning is a first-class tool everywhere.** KiCad has hotkeys
  7 (single trace), 8 (diff pair), 9 (diff pair skew), with a
  serpentine that previews live and snaps to grid. Altium has
  Interactive Length Tuning + Interactive Diff Pair Length Tuning
  (Ctrl+drag accordion controls). OrCAD X Presto has Delay Tune with
  configurable corner styles (Trombone, Sawtooth). PADS draws
  serpentines and colour-codes them yellow / green / red as length
  approaches target.

## Per-Tool Analysis

### Altium Designer

Altium is the visual reference point most professional designers compare
against; its rendering is OpenGL-accelerated and its interaction model
(modeless, hotkey-driven) is the model most other tools have copied.

#### Rendering

- **Tracks**: rendered as filled round-cap capsules at the configured
  width, AA-smoothed. Corner style is a per-segment property (line +
  arc). Default corner mode is 45° mitre; corner radius and 45°/90°/Any
  Angle/Curved are toggled with **Shift+Spacebar** during routing.
  ([Altium Routing](https://www.altium.com/documentation/altium-designer/pcb/routing/interactive))
- **Vias**: rendered as filled annulus (outer diameter — drill diameter)
  plus a hole circle drawn through. Tented vias suppress the solder-mask
  aperture; Altium displays this by *not* drawing the mask cutout, so a
  tented via reads as a solid copper circle without the mask "halo".
  Blind/buried vias get a span indicator in the via dialog and can be
  optionally annotated in the canvas via the Pad and Via display
  options.
  ([Altium Pads & Vias](https://www.altium.com/documentation/altium-designer/pcb/pads-vias))
- **Pads**: shapes are Round, Rectangular, Rounded Rectangular,
  Octagonal, and Custom (any closed contour, derived from Region
  objects). Each pad can have a different shape per layer (pad stack).
  The default top-layer SMD pad is red (the layer colour); through-hole
  pads render concentrically (annular ring + hole) on every signal layer
  they pass through.
  ([Altium Custom Pad Stack](https://www.altium.com/documentation/altium-designer/pcb/custom-pad-stack),
  [Altium Custom Pad Shape](https://www.altium.com/documentation/knowledge-base/altium-designer/custom-pad-shape))
- **Zones / Pours**: filled polygons with knockouts. Polygon Manager
  panel manages priority/order. Hatch fill option exists (Solid, Hatched
  with horizontal/vertical/45°/none). Thermal relief connections render
  as the actual spoke geometry (configurable spoke count, width, angle).
  Pour repouring is on demand: **Tools » Polygon Pours » Repour
  Selected** / **Repour All**. There is also a "Polygon Pour Cutout"
  primitive that creates voids inside other pours.
- **Layer colour / single-layer mode**: **Shift+S** cycles full ↔
  enabled single-layer modes (configurable in Preferences → Board
  Insight Display: Hide Other Layers / Gray Scale Other Layers /
  Monochrome Other Layers).
  ([Altium Single Layer Mode](https://www.altium.com/documentation/altium-designer/pcb/your-view-of-the-board))
- **Per-net colour**: **PCB Panel → Nets → right-click → Change Net
  Color**. View » Show Net Color Override (**F5**) toggles whether the
  override is applied. Net colour is bound by net name; carries through
  PCB ↔ Schematic when the schematic is colour-tagged.
  ([Altium Net Highlight](https://www.altium.com/documentation/altium-designer/sch-pcb/using-net-highlight-color),
  [Altium Net Color Override](https://www.altium.com/documentation/altium-designer/sch-dlg-netcoloroverridenet-color-override-ad))
- **Highlighting**: hover a track/pad/via shows a translucent yellow
  highlight; click selects; **Shift+click** adds. Cross-probing
  schematic ↔ PCB applies the same highlight.
- **Negative vs positive planes**: positive (Polygon Pour) is the
  default and standard model. Internal Plane Layers (Split/Mixed) are
  the negative-plane sibling and used for pure power planes; they show
  as a negative-image artwork in the canvas (everything is copper
  except the anti-pads).
- **Layer ordering in Layers & Colors panel**: top → mid → bottom
  (stack-up order, signal-then-plane within each tier).

#### Creation

- **Routing**: **Place » Interactive Routing** (default hotkey **P R**
  in Altium's hotkey-string scheme; configurable to a single key).
  Modes cycled with **Shift+R**: Ignore Obstacles / Walkaround / Push
  / HugNPush / Stop at First Obstacle.
  ([Altium Push and Shove](https://resources.altium.com/p/push-and-shove-router-how-it-works-and-why-you-need-it),
  [Altium Modifying Routing](https://www.altium.com/documentation/altium-designer/modifying-the-routing-ad))
- **Corner style**: **Shift+Spacebar** cycles 45° / 45° with arcs / 90°
  / 90° with arcs / Any Angle / Rounded.
- **Trace bend direction**: **Spacebar** toggles bend orientation.
- **Via insertion**: **\*** (numeric keypad), **2**, or auto on layer
  change (configurable). Layer change while routing is **+** or **-**;
  the active via is dropped at the click location.
- **Differential pair routing**: **Place » Interactive Differential
  Pair Routing** (**P I**). Pair pickup is automatic from defined diff
  pairs. Gap maintained from rule; can be overridden mid-route.
  ([Altium Diff Pair Routing](https://www.altium.com/documentation/altium-designer/pcb/high-speed-design/interactively-routing-differential-pairs))
- **Active Route**: Altium's auto-completion for a single connection or
  pin-pair. Select a connection, hit **Active Route** (toolbar or
  shortcut), and it picks the best legal path with current rules.
  Optionally glosses on completion.
- **Zone creation**: **Place » Polygon Pour** (**P G**). Outline
  drawing then dialog (priority, fill style, connect style, hatch).
  Repour on demand.
- **Length tuning**: **Place » Interactive Length Tuning** (**T R 1**)
  for single trace; **Interactive Diff Pair Length Tuning** for pairs.
  Live serpentine accordion preview, parameters tunable with hotkeys
  (`,`/`.` for amplitude, `<`/`>` for pitch, **Tab** for dialog).

#### Editing

- **Drag**: **E M E** (Move Edit) drags a segment with rules
  (push/shove); **Ctrl+drag** also drags. Endpoint drag breaks the
  segment in two. Drag mode honours the same Shift+R conflict mode as
  routing.
- **Move**: **M** moves a component; tracks follow if **Connected
  Tracks** preference is on (**Ctrl+drag** explicitly).
- **Rip-up**: **Tools » Un-Route » All / Net / Component / Connection**.
  Hotkey strings (e.g. **U N** for Un-route Net).
- **Glossing / Cleanup**: **Tools » Polygon Pours » Repour** for zones;
  **Tools » Up-Convert** legacy; the routing system runs Glossing
  inline (cycled with **Ctrl+Shift+G**: Off / Weak / Strong) and on
  demand via **Tools » Gloss Selected / Retrace Selected**. Retrace
  re-applies width/clearance from the current rules.
  ([Altium Glossing](https://www.altium.com/documentation/altium-designer/glossing-retracing-existing-routes-pcb))
- **Cleanup runs automatically** after route/drag operations even with
  glossing off (overlapping segment elimination only).

### KiCad (8.x / 9.x / nightly)

Most-documented implementation; full source available.

#### Rendering

- **Tracks**: rendered as round-cap capsules. KiCad has no flat-end
  option (a long-running forum request that was rejected on the grounds
  that the photoplot is round-cap so the screen should match).
  ([KiCad track end style discussion](https://forum.kicad.info/t/changing-track-end-style-settings-in-pcbnew-square-end-traces-supported/2409))
  Corner style is per-segment; default is mitred 45°. Curved (arc)
  tracks supported in 7+; Tracks → Properties has an Arc option.
  Drawn via `pcb_painter.cpp`'s `KIGFX::PCB_PAINTER::draw(const
  PCB_TRACK*)`.
- **Vias**: filled annulus + hole circle. Through, Blind/Buried, and
  Microvia types each have a distinct visual treatment in the appearance
  panel (separate visibility + colour for each). Net name + via span
  optionally drawn inside the via at high zoom (`Show via net name` and
  `Show via span` preferences).
- **Pads**: Circle, Rectangular, Rounded Rectangular (Roundrect), Oval,
  Trapezoidal, Chamfered Rectangle, and Custom Shape. Through-hole pads
  show concentric annulus + drill on every copper layer; SMD pads
  render only on their assigned layer. Slotted holes (via Drill is
  Slot) render as elongated drill apertures.
  ([KiCad PCB Editor 9.0](https://docs.kicad.org/9.0/en/pcbnew/pcbnew.html))
- **Zones / Pours**: filled polygons (`SHAPE_POLY_SET` after fill).
  Solid or Hatched (Orientation, Hatch Width, Hatch Spacing). Thermal
  relief renders as actual spokes (configurable Gap, Spoke Width,
  Minimum Spokes). Filled when user invokes **B** (Edit » Fill All
  Zones); unfilled with **Ctrl+B**. Zone outlines are always shown,
  fill body shown only when zone is filled.
  ([KiCad zone fill](https://docs.kicad.org/doxygen/classZONE__FILLER.html))
- **Layer colour / High Contrast**: Default theme has F.Cu red, B.Cu
  green, In1.Cu yellow, In2.Cu cyan, In3.Cu magenta, In4.Cu violet (the
  intuition is rainbow: the deeper into the stack, the further across
  the rainbow). High Contrast mode is **Ctrl+H** to cycle through
  Normal / Dim / Hidden for inactive layers.
  Dim factor is configurable in Preferences → PCB Editor → Display
  Options → "High contrast mode dimming factor".
  ([KiCad COLOR_SETTINGS doxygen](https://docs.kicad.org/doxygen/classCOLOR__SETTINGS.html),
  [KiCad themes repo](https://github.com/pointhi/kicad-color-schemes))
- **Object opacity sliders**: Tracks / Vias / Pads / Zones each have an
  independent opacity slider in the Appearance panel. Default zones is
  translucent (~70%) so objects under fills stay visible.
- **Per-net colour**: **Nets tab of Appearance panel**, double-click
  swatch to assign. "Net color mode" pulldown: None / Ratsnest / All.
  When **All** is selected, the net's colour applies to its tracks,
  vias, pads, *and* zones simultaneously.
- **Active layer indication**: small triangle / arrow next to the
  active layer's colour swatch in the Layers tab. Active layer name is
  bolded. **PgUp** picks F.Cu, **PgDn** picks B.Cu, **+/-** cycles
  enabled signal layers.
- **Negative vs positive planes**: positive only (KiCad has no negative
  plane primitive). Power planes are filled zones at high priority.
- **Layer ordering**: top → bottom in the Layers tab matches
  stack-up order. F.Cu is always topmost in the panel.

#### Creation

- **Routing**: **Route » Single Track** (**X**); **Route » Differential
  Pair** (**6**).
  Modes (PNS): Highlight Collisions / Shove / Walk Around. Cycled with
  the dropdown in the routing options dialog (**Tools » Interactive
  Router Settings**) or the toolbar. There is no single hotkey to
  cycle modes mid-route as of 9.x, despite a long-standing wishlist
  bug ([gitlab #9414](https://gitlab.com/kicad/code/kicad/-/issues/9414)).
- **Track posture / corner**: **/** toggles diagonal vs straight initial
  segment.
- **Via insertion during routing**: **V** drops a through via and
  switches to the next active layer; **Shift+V** inserts a microvia;
  **Alt+Shift+V** inserts a blind/buried via; **<** / **>** cycle via
  size from the via list.
- **Zone creation**: **Place » Add Filled Zone** (**Ctrl+Shift+Z**) →
  draw outline → properties dialog (net, layer, fill type, thermal
  relief). Fill on demand with **B**.
- **Pad placement (footprint editor)**: pads are added in the Footprint
  Editor only; the PCB editor does not allow free pad placement on the
  board (you must add a footprint first). This is a long-standing
  philosophical choice that some users from Altium / Allegro background
  find frustrating.
- **Length tuning**: **7** (Single Trace), **8** (Differential Pair),
  **9** (Differential Pair Skew). Live serpentine preview; parameters
  on right-toolbar pop-out.
  ([KiCad diff pair tuning](https://techexplorations.com/guides/kicad/high-speed-pcb-design/kicad-9-differential-pair-length-tuning-guide/))
- **Snap behaviour**: object snap to pad centre, track endpoint, via
  centre, intersection, midpoint. Configurable per-snap in
  Preferences → PCB Editor → Editing Options.

#### Editing

- **Drag with rules**: **D** (Drag 45 mode) — uses the active PNS mode
  (Shove/Walkaround). **G** (Drag Free Angle) — splits the segment and
  drags as Highlight Collisions only.
  ([KiCad drag hotkeys](https://groups.io/g/kicad-users/topic/how_can_i_drag_traces_in/81002705))
- **Move**: **M** (rigid, no track-following); use Drag for connected
  glide.
- **Rip-up**: **Edit » Selection Filter** + **Delete** for selected;
  **Tools » Cleanup Tracks and Vias** for bulk cleanup.
- **Cleanup Tracks and Vias** dialog options: Merge Co-linear Segments,
  Delete Tracks Unconnected at One End, Delete Tracks in Pads, Delete
  Redundant Vias, Delete Tracks Inside Zones (with rule violation),
  Refill Zones After Cleanup.
  ([KiCad cleanup dialog](https://docs.kicad.org/doxygen/classDIALOG__CLEANUP__TRACKS__AND__VIAS.html))
- **Net selection**: backtick (**`**) highlights net under cursor with
  bright + dim-the-rest. **Ctrl+`** toggles highlighting on/off without
  re-picking. **Edit » Select All Tracks in Net** explicit menu item.
  ([KiCad net highlight](https://docs.kicad.org/9.0/en/pcbnew/pcbnew.html))
- **Component move with track-following**: enabled via Preferences →
  PCB Editor → Editing Options → "Allow free pads" + "Drag selected
  components with connected tracks". Off by default — a recurring
  surprise for new users.
- **Group edit**: Edit » Selection Filter restricts what mouse-drag
  selects (tracks-only, pads-only, zones-only, etc.). **Edit » Find →
  Find by Net** opens cross-probing.

### Cadence Allegro / OrCAD X Presto

Allegro is the long-standing high-end Cadence layout tool; OrCAD X
Presto (released ~2024) is the rebrand of OrCAD's interactive editor on
the Allegro engine, with a re-skinned UI. Most of the rendering and
algorithm are shared.

#### Rendering

- **Tracks** (called "clines" — connection lines): round-cap stroked
  capsules. Display Color settings have separate entries per layer per
  object class (Etch / Pin / Via / Drill / Boundary / RatT / etc).
  ([Allegro Visibility/Colour](https://becomepcbpro.com/ColourSelection.html))
- **Vias**: padstack-based (every via is a padstack instance). Through
  vias are filled annulus + drill marker; blind/buried vias annotated
  with span if **Display » Show Etch by Subclass** is enabled.
- **Pads**: full padstack model — Regular / Anti / Thermal definitions
  per copper layer, plus Mask / Paste / Film aperture. Custom shapes
  supported. NPTH (non-plated through hole) is a separate hole type
  with no copper aperture.
  ([Allegro Lesson 3 Padstacks](http://education.ema-eda.com/iTrain/PCBEditor163/lesson_3.html))
- **Zones (Shapes)**: positive (Dynamic Shape) or negative (anti-etch on
  a negative film layer). Dynamic shapes refill *live* during editing —
  Allegro's headline feature; voids appear automatically as you route
  through. Static shapes require **Shape » Compose Shape** /
  **Update To Smooth**. Hatched fill option on dynamic shapes.
  ([Allegro Dynamic Shapes](https://www.parallel-systems.co.uk/wp-content/uploads/2020/02/Shape_Settings.pdf))
- **Negative planes**: distinct workflow — Anti Etch on a negative
  artwork layer. Anti-pad / thermal-pad padstack entries control voids
  automatically. Split planes via **Shape » Split Plane**.
  ([Allegro split plane](https://resources.pcb.cadence.com/blog/2021-split-plane-routing-in-cadence-s-allegro-pcb-editor))
- **Visibility Pane** (OrCAD X Presto): **View » Panels » Visibility**.
  Three sections (Layers / Nets / Display), each row has an "etch"
  toggle, "rats" toggle, and colour swatch. Per-net colour assignment
  through right-click on Nets row.
  ([OrCAD X Visibility](https://resources.pcb.cadence.com/blog/2024-navigating-the-visibility-pane-in-orcad-x-presto-pcb-editor))
- **Active-layer indication**: subclass column highlighted in the
  Visibility Pane; active layer name shown in the status bar.
- **Layer ordering in panel**: stack-up order, top-to-bottom.

#### Creation

- **Routing**: **Add Connect** (Allegro) / **Add Connect** (Presto). In
  Presto the floating toolbar's *Mode* dropdown chooses Standard /
  Assisted (Hug or Shove) / Etch Edit.
  ([OrCAD X Routing](https://resources.pcb.cadence.com/layout-and-routing/orcad-x-routing-modes),
  [Quickly Route OrCAD X](https://www.ema-eda.com/how-to-page/how-to-quickly-route-connections-in-orcad-x-presto/))
- **Push-and-shove**: Shove mode is "Assisted → Shove". Hug mode
  walks-around copper features as close as legally possible.
- **Via insertion**: **F1** or right-click → Add Via while in Add
  Connect; via type per the active class. Layer cross-section displayed
  during routing.
- **Differential pair routing**: **Route » Diff Pair**; pair pickup
  from constraint manager.
- **Length / delay tuning**: **Route » Delay Tune**. Configurable corner
  styles include Trombone, Sawtooth, Rounded. Phase tuning for diff
  pairs.
  ([OrCAD X Delay Tune](https://www.ema-eda.com/how-to-page/how-to-add-delay-tuning-in-orcad-x-presto/))
- **Spread Between Voids** (Presto): user picks two reference objects
  defining a routing channel; tool spreads enclosed clines evenly.
  Allegro PCB Editor (non-Presto) lacks this.
- **Zone (Shape) creation**: **Shape » Polygon** / **Rectangle** /
  **Circle**, then assign net + parameters. Dynamic shapes refill on
  edit. Static needs explicit **Shape » Compose Shape**.

#### Editing

- **Slide**: drags a cline/via with rule honouring; corners adjusted to
  preserve 45/90°. Right-click → Slide on a selected segment.
- **Custom Smooth / Glossing**: **Custom Smooth** (Allegro) glosses an
  individual net interactively; reduces corner count, shortens path.
  ([Allegro glossing](http://education.ema-eda.com/iTrain/PCBEditor163/lesson_10.html))
- **Rip-up**: right-click → **Delete**, or **Edit » Delete** on
  selection. **Unroute** is per-net via Constraint Manager or
  right-click on a net.
- **Group edit**: Find filter (Visibility Pane → Find tab) restricts
  selection to specified object classes. Selection mask + Net Filter +
  Layer Filter combine.
- **Net selection**: hover highlights with a temporary halo; click
  selects all clines on a net.

### Mentor / Siemens Xpedition (PADS)

PADS Pro is the smaller-form Xpedition derivative. PADS Logic + Layout
+ Router is the classic three-tool suite; Xpedition Layout is the
high-end version sharing the same engine.

#### Rendering

- **Tracks**: round-cap stroked capsules. Display Colors dialog
  controls Connections / Routes / Pads / Vias / Drill / Drafting per
  layer.
- **Vias**: padstack instances; Pin Pair editor exposes via stack per
  layer span.
- **Pads**: Round, Rectangle, Square, Oval, Annular, Polygon (custom)
  per pad-stack layer.
- **Zones (Plane Areas / Copper Pours)**: PADS distinguishes
  **Plane Areas** (negative artwork generated at fab time, anti-pad
  control via padstack), **Copper Pours** (positive polygon flooded
  on demand), and **Copper** (manually drawn solid copper objects). The
  three-way split is unique to PADS and a long-standing source of new-
  user confusion.
  ([PADS Layout tutorial](http://www.theky22.com/downloads/pads%20layout%20tutorial%20-%20sgi%20-%20pcb%20design.pdf))
- **Per-net colour**: Display Colors dialog → Connections column → set
  the connection-line colour to "no colour" to suppress; per-track
  colour via right-click → Properties → Colour.
- **Active layer indication**: layer dropdown in the toolbar shows
  active layer; modeless command **L<n>** picks layer n.
- **Layer ordering**: stack-up order, signal then plane within each
  pair.

#### Creation

- **Routing**: **Add Route** (modeless command **AR**). Push-and-shove
  is the default; per-trace control via Options dialog. Diff pair
  routing via **Add Differential Pair Route** (**ADR**).
  ([PADS push-shove](https://blogs.sw.siemens.com/pads/2019/04/05/reduce-design-time-using-routing-automation-part-2/))
- **Via insertion**: **Spacebar** drops a via at cursor (pad layer
  switches automatically). **F4** is the layer switch hotkey.
- **Length tuning**: **Tune Routes** function with Trace Shove on/off,
  Via Shove on/off, Via Jump, Pad Jump as separate toggles. The trace
  colours yellow (close) / green (target) / red (over) as length
  changes.
  ([PADS tuning visual](https://blogs.sw.siemens.com/electronic-systems-design/2014/10/27/pcb-routing-solutions-simplifying-the-tuning-process-part-2-manual-tuning/))
- **Pin Pair**: a defined two-pin route with topology protection. Once
  protected, push-and-shove won't re-MST it.
- **Zone creation**: **Setup » Plane Areas** then draw outline; for
  Copper Pour use **Drafting Toolbar » Copper Pour**. Pours flood on
  demand via right-click → Flood.

#### Editing

- **Drag (Slide / Move)**: **Move Sequence** modeless command (**MS**)
  honours rules; **Move** plain (**M**) is rigid.
- **Glossing**: **Tune** function in Tune Routes acts as both length
  adjust and gloss; shortens paths and reduces corner count.
- **Rip-up**: **Unroute** modeless command per selection; full board
  via **Unroute All**.
- **Net selection**: pin-pair-aware; selecting a track expands to the
  enclosing pin pair, not the whole net.

### Eagle / Fusion 360 Electronics

Eagle (Autodesk-acquired in 2017) is being deprecated in favour of
Fusion 360 Electronics (the integrated successor). The rendering model
is largely unchanged from late Eagle; the routing got a real
push-and-shove in the Fusion era.

#### Rendering

- **Tracks**: round-cap stroked capsules. Default: 45° mitre.
- **Vias**: filled annulus + drill marker. Tented vias identified via
  the via dialog (Stop = mask aperture, on/off).
- **Pads**: Square, Round, Octagon, Long, Offset (Eagle's own slightly
  unusual shape names). Smashable text overlay.
- **Polygons (zones)**: positive only. Polygon outline drawn on a
  copper layer, then **RATSNEST** (or **B** in Fusion) recomputes the
  fill. Hatched fill via Polygon Properties → Pour = Hatch.
  Re-poured on **RATSNEST** call; not live.
  ([Eagle routing](https://www.autodesk.com/products/fusion-360/blog/routing-autorouting-pcb-layout-basics-2/))
- **Layer colour**: per-layer; default Top = red (layer 1), Bottom = blue
  (layer 16), Pads = grey (layer 17), Vias = grey (layer 18). Eagle's
  use of *layer 19 Unrouted* for airwires is unique.
- **Layer ordering**: numeric (Eagle assigns each layer an int; the
  panel sorts by number). Standard signal layers are 1..16, plane
  layers are positive integers in the inner range.

#### Creation

- **Routing**: **ROUTE** command (modeless), or click the route tool.
  Walk Around / Push Violators modes selected from the toolbar
  Bend Style dropdown plus the Violators toggle. Trace bend cycle:
  **Spacebar** rotates corner direction.
  ([Fusion routing](https://www.autodesk.com/products/fusion-360/blog/interactive-routing-with-fusion-360-electronics/),
  [Walkaround / Push Violators](https://www.autodesk.com/support/technical/article/caas/sfdcarticles/sfdcarticles/Routing-with-Walkarounds-violators-and-Push-Violators-is-not-working-on-PCB-in-Fusion-Electronics.html))
- **Via insertion**: layer change drops a via automatically; explicit
  **VIA** command (modeless) places one without routing.
- **Polygon creation**: **POLYGON** command, draw outline, set
  properties; **RATSNEST** recomputes fills.
- **Diff pair**: pair definition by name suffix `_P` / `_N` (or
  configurable). **ROUTE** picks both up if naming matches.
- **Length tuning**: **MEANDER** command — adds serpentine to bring a
  trace up to length. Visualises the target as a coloured indicator on
  the meander.

#### Editing

- **Drag**: **MOVE** command on a track segment moves it; corners
  adjust. No separate rigid/flex mode.
- **Rip-up**: **RIPUP** command. **RIPUP ;** (with semicolon) ripups
  everything. **RIPUP <signal>** ripups one net.
  ([Eagle RIPUP](https://www.autodesk.com/products/fusion-360/blog/routing-autorouting-pcb-layout-basics-2/))
- **Cleanup**: no equivalent; manual ripup-and-reroute is the workflow.
- **Net selection**: **SHOW <signal>** highlights the net.

### Horizon EDA

Studied directly from `research/horizon-source/`. Useful because Horizon
is the closest open-source design point for what Datum EDA is building
(positive zones, plane refresh on demand, KiCad-derived PNS, Cairo/OpenGL
rendering).

Source pointers (`research/horizon-source/src/`):

- `board/track.hpp/cpp` — `Track` carries a layer, width (with
  `width_from_rules` flag), `from`/`to` `Connection` (junction or
  pad), and an optional arc center. Width is rule-derived by default.
- `board/via.hpp/cpp` + `via_definition.hpp/cpp` — `Via` is a
  `Padstack` instance with a `LayerRange span`, `Source` enum
  (LOCAL / RULES / DEFINITION) controlling whether the padstack is
  one-off or rule-driven, and a `parameter_set`.
- `board/plane.hpp/cpp` + `plane_update.cpp` — `Plane` extends
  `PolygonUsage` and holds a `polygon`, `priority`, and
  `PlaneSettings` (style, fill_style SOLID/HATCH, hatch params,
  thermal_settings with ConnectStyle SOLID/THERMAL/FROM_PLANE,
  spoke params, n_spokes, angle, min_width, ROUND/SQUARE/MITER
  edge style). Each plane stores `std::deque<Fragment>` of computed
  fill geometry (Clipper paths, with first path = outline and
  remaining = holes).
- `canvas/render.cpp` — central `Canvas::render` overloads for each
  primitive (`Track`, `Via`, `Plane`, `Padstack`, `BoardHole`,
  `Polygon`). Track render uses `ColorP::FROM_LAYER` so the colour
  comes from the active layer scheme; bus tracks (no net) use
  `ColorP::BUS`. Via render delegates to `render(const Padstack&,
  bool interactive)`, which in turn renders polygons, shapes, and
  holes — i.e. the via barrel uses the same primitive pipeline as
  pads.
- `canvas/canvas_pads.cpp` — `CanvasPads::img_polygon` filters to copper
  layers and accumulates pad geometry into a per-pad map for use by
  the plane voider (anti-pad computation).
- `router/pns_horizon_iface.cpp/hpp` — Horizon's adapter into KiCad's
  PNS. Confirms that Horizon ships KiCad's push-and-shove engine
  verbatim, with a Horizon-specific shim mapping Horizon's data model
  into PNS's track/via/joint primitives.

#### Rendering

- **Tracks**: round-cap stroked. `Canvas::render(const Track&)` uses
  `draw_line(from, to, color, layer, true, width)` — `true` is the
  round-cap flag. Arc tracks use `draw_arc`. A small lock glyph
  (`draw_lock`) is drawn at the segment midpoint when `track.locked`
  is true; never drawn as part of the copper itself.
- **Vias**: rendered via the padstack pipeline so blind/buried/micro/
  through differ only in the padstack contents. Net name and span
  optionally drawn inside the via at high zoom (`show_text_in_vias`,
  `show_via_span` modes ALL / BLIND_BURIED).
- **Pads**: padstack-driven (TOP / BOTTOM / THROUGH / MECHANICAL).
  Mechanical pads only draw in interactive mode (don't show in
  fabrication output). Pad name overlay drawn on a virtual overlay
  layer above copper, with LOD culling tied to pad size.
- **Planes (zones)**: outline + computed `Fragment` geometry (Clipper
  paths). Hatched fill rendered as a parallel-line pattern. Orphan
  fragments (disconnected pieces) tracked separately and optionally
  flagged.
- **Per-net colour**: `ColorP::FROM_LAYER` is the default; a per-net
  colour override is applied via the same `color2` shader channel
  that airwires use (`triangle-ubo.glsl` line `color == 13 /*airwire*/`
  pattern, generalised).
- **Layer display**: `LayerDisplay` per-layer (Mode = OUTLINE / FILL /
  FILL_ONLY) controls how each layer renders. Inactive layers can be
  set to OUTLINE for the same single-layer effect.
- **Active-layer indication**: status bar plus the work-layer name
  highlighted in the layers panel.
- **Layer ordering**: stack-up order in the canonical
  `BoardLayers` enum (TOP_COPPER = 0, INNER layers = -1..-N,
  BOTTOM_COPPER = -100). Panel shows top-to-bottom.

#### Creation

- **Routing**: **Route track** tool (key sequence in the spacebar
  menu). PNS-driven so push-and-shove is on by default.
  ([Horizon docs](https://docs.horizon-eda.org/en/latest/feature-overview.html))
- **Via insertion**: **V** during routing (matches KiCad). Layer-pair
  rule defines which layer the router switches to on V. Standalone
  via via "Place via" tool.
- **Plane creation**: draw a polygon on a copper layer, then **Add
  plane** tool to assign a net + plane settings. Plane refresh: live
  during edit, or explicit "Update planes" / "Update all planes"
  tools.
- **Zone hatched fill**: PlaneSettings.fill_style = HATCH with
  configurable hatch_border_width, hatch_line_width,
  hatch_line_spacing.
- **Length tuning**: integrated via PNS — Horizon ships KiCad's
  meander/diff-pair tuning code unmodified.

#### Editing

- **Drag**: PNS-aware drag (D in KiCad → equivalent in Horizon).
- **Net tie**: explicit `BoardNetTie` primitive for joining two nets
  at a controlled location (a clean rendering example of "this is
  copper but it's a controlled connection between two nets" — Horizon
  draws net ties with the `ColorP::NET_TIE` palette colour and overlay
  text).
- **Plane refresh**: `update_planes(fast, planes_only)` parameterisation
  for incremental refill, mirroring the airwire `update_airwires(fast,
  nets_only)` pattern.

### Consumer tools

#### DipTrace

- **Tracks** with round caps, configurable via Route Manager. Smart
  Routing mode is push-and-shove style.
  ([DipTrace whats new](https://diptrace.com/diptrace-software/whats-new/))
- **Pads** support Round, Rect, Polygon, Oval, Lines (custom). DXF
  import for arbitrary pad shapes.
- **Copper Pour**: hatched or solid; via-stitching tool
  (Stitch Vias to Pour). **Verification → Check Net Connectivity**
  is the canonical "what's still unrouted" tool. Real-time pour update
  configurable.
- **Routing**: Manual / Interactive / Auto (DipTrace ships two
  autorouters — Shape Router and Grid Router). Push-and-shove called
  Smart Routing.
- **Layer colours**: customisable, default red top / blue bottom.

#### EasyEDA (Std + Pro)

- **Tracks**: round-capped, rectangular fills supported via Solid Region.
- **Pads**: Round, Rectangular, Oval, Polygon (Std), full custom
  Shaped Pad (Pro).
  ([EasyEDA Pro shaped pad](https://prodocs.easyeda.com/en/pcb/place-shaped-pad/))
- **Copper Area** (positive zone): solid or hatched (mesh). **Shift+M**
  hides fill (outline only). **Shift+B** rebuilds fill — explicit, no
  live update. EasyEDA users complain about this loud and often.
  ([EasyEDA Copper Area](https://docs.easyeda.com/en/PCB/Copper-Pour/))
- **Solid Region** is a separate primitive (manual copper with no
  voiding); useful for connecting close pads quickly.
- **Routing**: walkaround mode by default; Pro adds limited
  push-and-shove. No explicit per-mode hotkey.
- **Layer ordering**: top → bottom, with the Layer Manager panel
  matching stack-up.

#### Quadcept

- **Tracks**: round-capped; configurable corner styles.
- **Routing**: Semi-Auto Routing (single-click optimal-path finder),
  Push Routing, Parallel Routing.
  ([Quadcept routing](https://www.quadcept.com/en/manual/pcb/post-115))
- **Hotkeys**: **Z** toggles routing start direction; **P** toggles
  routing-at-pad-angle; **I** rebuilds plane fills.
- **Plane fill**: explicit rebuild via **I**; not live.
- **Diff pair**: dedicated diff-pair route tool; gap from rule.
- **Layer colours**: Quadcept ships a "Same-Layer / Cross-Layer" hint
  on rats: solid line for same-layer connections, dotted line for
  cross-layer (a small UX touch other tools don't have).
  ([Quadcept rats](https://www.quadcept.com/en/manual/pcb/post-97))

#### Pulsonix

- **Tracks**: Push-Aside routing with multi-trace mode (route N parallel
  traces simultaneously). Differential pair routing with online length
  display; dynamic serpentine routing during length tuning.
  ([Pulsonix high-speed](https://pulsonix.com/high-speed-design))
- **Vias**: Micro-via composite assemblies definable in the Technology
  dialog; per-class drill rules.
- **Pads**: standard shape set + custom; per-pad overrides.
- **Copper pour**: positive; rebuild on demand.
- **Selection**: clicking a track segment selects the *whole track
  path*, including all collinear segments and through-vias on the same
  net — convenient for whole-route operations, occasionally
  surprising for segment-only edits.
- **Highlighting**: pad-name colour controlled separately from pad
  colour; both selectable in the Colours dialog.
- **Layer ordering**: stack-up order; Layer Class assignment groups
  layers by purpose (Routing / Plane / Documentation / etc) for
  panel filtering.

## Cross-Cutting Patterns

### Visual conventions

#### Layer colour conventions

There is **no IPC standard** for EDA-tool layer colour. There is a
strong informal convention that survives across tools because it's the
default in the dominant tools.

- **Top copper (F.Cu / Layer 1 / Top Layer)**: **red**, in Altium,
  KiCad (default theme), Eagle/Fusion (layer 1), DipTrace,
  EasyEDA, Pulsonix, Quadcept. Allegro/OrCAD's default scheme is
  cyan for top etch but most users immediately swap to red.
- **Bottom copper (B.Cu / Layer 16 / Bottom Layer)**: **blue** in
  Altium, Eagle/Fusion, DipTrace, EasyEDA, Pulsonix; **green** in
  KiCad's default theme (the historical KiCad outlier), occasionally
  swapped to blue by users.
- **Inner signal layers**: KiCad uses yellow / cyan / magenta / violet
  rotation; Altium uses cyan / grey / yellow; Allegro is
  scheme-specific. No strong convention.
- **Plane layers**: usually a distinct hue family from signal layers
  in mature tools (green / olive / orange) so designers can spot
  power planes at a glance in the layer panel.
- **Solder mask layer (top/bottom)**: typically magenta/pink (top
  mask) and dark green (bottom mask) — chosen to be distinct from
  the copper underneath without obscuring it.
- **Silkscreen**: white on dark themes, black on light themes; named
  layer colour rather than a derived one.
- **Edge cuts / board outline**: yellow universally.
- **Holes / drill marks**: black on light themes, white on dark.

The most-cited reference for "what colour theme should I expect" is
the [pointhi/kicad-color-schemes repo](https://github.com/pointhi/kicad-color-schemes),
which collects 30+ community themes and reveals just how much
variation exists past the F.Cu = red, B.Cu = blue starting point.

#### Track / pad / via stroke conventions

- **Tracks** are **round-cap capsules** in every tool surveyed except
  raw Eagle (which had a flat-cap option in the dim past). Round caps
  match the photoplotter aperture and the actual etched copper.
  Anti-aliased on every modern tool.
- **Track corners** default to **45° mitre** (chamfered) in every tool.
  Modern tools (Altium, KiCad 7+, Allegro) optionally render corners
  as small fillet arcs ("Curved" or "Rounded" corner mode) but ship
  with mitre as the default.
- **Track widths** are stored in board units (nm in KiCad, mils in
  Allegro/PADS, mm in Altium 25+). Display units are a separate
  preference.
- **Pads** are filled polygons. SMD pads are layer-specific filled
  shapes; through-hole pads are concentric annulus + hole, drawn on
  every copper layer the hole passes through.
- **Pad shapes** universally include: Circle, Rectangle, Rounded
  Rectangle (sometimes called Roundrect), Oval (sometimes called
  Stadium / Long), and Custom (any polygon). Octagon is offered by
  Altium, DipTrace, Eagle/Fusion. Trapezoidal and Chamfered Rectangle
  are KiCad-specific. Custom is the universal escape hatch for odd
  footprints.
- **Vias** are **filled annulus (outer ⌀ minus drill ⌀) plus a
  hole-marker circle** in every tool. Tented vias suppress the
  solder-mask aperture but the via barrel itself is always drawn.
  Via-in-pad either renders the pad-on-top (Altium default) or
  ghost-shows the via through the pad (KiCad default).
- **NPTH (non-plated through holes)** render as a hole circle with no
  copper aperture, distinct from PTH (plated through holes) which
  always have an annulus. KiCad uses a smaller hole glyph and a
  red-X overlay for tooling holes; most other tools just show the
  bare hole.

#### Zone fill rendering

- **Solid fill** (single closed polygon with knockouts) is the
  universal default. Knockouts come from clearance to other-net pads,
  vias, tracks, and zones, plus thermal-relief spokes for same-net
  pads, plus the zone outline itself.
- **Hatched fill** is a global option in Altium, KiCad, Allegro,
  DipTrace, EasyEDA Pro. Useful for flex PCBs and impedance-controlled
  applications. KiCad ships orientation, hatch width, and hatch
  spacing as separate parameters.
- **Outline-only** (zone unfilled, only the outline visible) is the
  intermediate state during editing in every tool. KiCad's default
  is filled-on-save-only; Eagle/Fusion is filled-on-RATSNEST;
  Altium is filled-on-Repour; Allegro Dynamic is filled-live.
- **Thermal relief** renders the actual spoke geometry: a same-net pad
  inside a flooded zone gets clearance gap on all sides and N (default
  4) thin "spokes" of copper connecting it to the surrounding fill.
  Spoke count, width, gap, and angle are configurable. The visual
  result is the recognisable "dog-bone" or "wagon-wheel" pattern
  around through-hole pads.
- **Multi-island zones**: a zone whose fill computes into multiple
  disconnected pieces (because of obstacles) renders each piece
  separately. Allegro and Horizon explicitly track "orphan" fragments
  (pieces not connected to any pad/via) and either delete them, flag
  them as DRC violations, or surface them in a panel.
- **Keepout zones** render as crosshatch outline (no fill), in a
  distinct hue (Altium uses purple, KiCad uses purple/magenta).
  A keepout is data-model-distinct from a zone — it doesn't have a
  net, it has a keepout type (track / via / pad / footprint /
  combination).
- **Zone priority** controls which zone wins when two overlap on the
  same layer. Higher priority → drawn on top, knocks out lower. Used
  for split-power planes (e.g. a small +5V island inside a larger GND
  pour). Visualised as a per-zone numeric badge in some tools
  (Altium's Polygon Manager).

#### Active-layer indication

Universal patterns:

- **Coloured arrow / triangle pointing at the active layer's row** in
  the Layers panel. KiCad, Altium.
- **Bold layer name** in the Layers panel for the active layer. KiCad.
- **Status bar** continuously displays the active layer name. Every
  surveyed tool.
- **Layer dropdown / combo** in the toolbar shows active layer with a
  swatch. Altium, Allegro, PADS.
- **Cursor crosshair colour** matches the active layer colour during
  routing — a particularly readable affordance, used by KiCad,
  Altium, and Pulsonix.
- **Outline-on-other-layer-objects** when high-contrast mode is engaged.
  Altium's Single Layer / KiCad's High Contrast both convert
  inactive-layer copper to outline-only at user option.

#### Single-layer / high-contrast modes

| Tool      | Hotkey        | Modes                                     |
|-----------|---------------|-------------------------------------------|
| KiCad     | **Ctrl+H**    | Normal / Dim / Hidden                     |
| Altium    | **Shift+S**   | Full / Hide Other Layers / Gray Scale Other Layers / Monochrome (each enableable separately in Preferences) |
| Allegro   | **F4**-ish (configurable) | Per-layer visibility toggling via Visibility Pane (no single global mode) |
| OrCAD X   | Visibility Pane | Per-layer toggle; no global mode hotkey |
| PADS      | **L<n>**      | Layer-pick + per-class colour suppression |
| Eagle     | `DISPLAY` cmd | Per-layer visibility, no dim mode         |
| Horizon   | Layer panel   | Per-layer LayerDisplay (FILL/OUTLINE/FILL_ONLY) |
| EasyEDA   | Shift+M (zone outline) | Layer toggle in Layer Manager panel |
| Pulsonix  | Layer Class   | Layer-class-based group toggle            |

KiCad's three-mode High Contrast (with a configurable dim factor) is the
most cited model. Altium's Shift+S cycle is the most familiar to
professional users. Both let the user *configure which modes the
shortcut cycles through*, which removes the "I don't want hidden mode,
only dim mode" friction.

### Authoring conventions

#### Routing modes (push-shove vs walkaround vs ignore)

The three-mode taxonomy is universal:

- **Ignore obstacles / Highlight Collisions** — draw the trace anywhere,
  show DRC violations as you go. KiCad, Altium, Allegro.
- **Walkaround / Hug** — trace bends to avoid obstacles without
  displacing them. KiCad, Altium, Allegro Hug, Pulsonix, Horizon.
- **Push / Shove** — trace displaces other-net obstacles to make room
  while preserving their topology. KiCad PNS Shove, Altium, Allegro
  Shove, Horizon, PADS.

Some tools add a fourth:
- **HugNPush** (Altium) — hug if possible, shove if necessary.

The hotkey convention has not converged. Altium uses **Shift+R** to cycle
modes mid-route. KiCad has historically not had a cycle hotkey
(wishlist [#9414](https://gitlab.com/kicad/code/kicad/-/issues/9414));
mode is set in Tools » Interactive Router Settings. Allegro/Presto picks
mode from the floating widget.

Datum should pick a hotkey early and document it loud — every user
coming from another tool will reach for *some* cycle key.

#### Modal vs modeless routing

- **Modeless** is the modern default. Type a hotkey or click a tool
  button, and the cursor enters routing mode; everything else stays
  available (zoom, pan, layer change, query). Esc exits. KiCad,
  Altium, Allegro, PADS, Horizon, Pulsonix.
- **Modal** (full-screen route mode that disables other actions) is the
  Eagle/Fusion legacy. The newer Fusion routing improvements have made
  this less modal but the door is still distinct.

#### Via insertion conventions

- **`V`** is the dominant hotkey to drop a via and switch layers
  during routing. KiCad, Horizon, OrCAD X, Pulsonix.
- **Spacebar** is the PADS convention.
- **+/-** to cycle layers without a via, **PgUp/PgDn** to jump to
  F.Cu / B.Cu directly is the KiCad pattern.
- **Auto-via on layer change** (push **+** to switch layers and a via
  drops automatically): default-on in Altium, KiCad, Allegro.
- **Via type cycling**: `< / >` (KiCad, with via list cycling),
  numeric padstack pick (Allegro), **Tab** to open via dialog mid-route
  (Altium).
- **Microvia / blind / buried** picked separately (different hotkey or
  different tool button). KiCad: Shift+V microvia, Alt+Shift+V blind.

#### Length tuning conventions

| Tool      | Single | Diff Pair | Diff Pair Skew | Style options |
|-----------|--------|-----------|----------------|---------------|
| KiCad     | **7**  | **8**     | **9**          | Rounded, Mitre; live preview; right-toolbar parameters |
| Altium    | T R 1  | T R D     | (combined)     | Mitred / 90° / Curved; live accordion; Tab dialog |
| OrCAD X   | Delay Tune | Diff Pair Tune | Phase Tune | Trombone / Sawtooth / Rounded |
| PADS      | Tune Routes | Tune Routes (DP) | DP Skew  | Sawtooth; Trace Shove / Via Shove toggles |
| Eagle/Fusion | MEANDER cmd | (post-route) | -          | Mitred only |
| Pulsonix  | dynamic serpentine | Diff Pair | Skew       | Online length display |

Common UX features:
- **Live length readout** as you drag the serpentine — yellow / green /
  red colour banding (PADS) or numeric (Altium / KiCad).
- **Amplitude / pitch hotkeys** to grow/shrink the serpentine
  parametrically (`,` / `.` Altium; `1` / `2` KiCad).
- **Snap to design rule** for the serpentine pitch (so the serpentine
  doesn't auto-violate the diff-pair rule).

#### Differential pair conventions

- **Pair pickup is automatic from constraint manager** in
  Altium / Allegro / OrCAD X / PADS / Pulsonix. KiCad, Eagle, Horizon
  pick pairs from net-name suffix (`_P` / `_N`, `_+` / `_-`,
  configurable). Both work; constraint-manager is more flexible at the
  cost of more setup.
- **Gap maintenance** during routing is universal — the router holds
  the configured gap as both traces follow the cursor.
- **Length matching** is a separate post-route step, not part of the
  initial route.
- **Visual treatment**: the pair routes as two parallel tracks with
  the gap visible as a thin clear-net "channel". Some tools (Altium
  Live View) render a small "DP" badge near the cursor during routing.

#### Snap conventions

- **Grid snap** is the universal default; configurable grid size
  (with a "fine" / "coarse" / "user" pulldown in most tools).
- **Object snap** (snap to pad centre / track endpoint / via centre /
  intersection / midpoint) is configurable per-snap-type in
  Altium, KiCad, Allegro, Pulsonix.
- **Origin snap** (snap to component origin or board origin) in
  Altium, Allegro.
- **Modeless toggle**: most tools toggle grid snap with a hotkey
  (KiCad: configurable; Altium: G to cycle grid sizes; Pulsonix: Shift
  to override snap temporarily).
- **Crosshair shape during routing**: small crosshair at cursor +
  longer reticle through current trace direction is the universal
  pattern; "bullseye" or "long-cross" variants per user preference.

### Editing conventions

#### Drag modes (rigid / flexible / slide)

The two-mode pattern is universal:

| Tool   | Rigid (preserves angles) | Flexible / Slide (rules-aware) |
|--------|--------------------------|-------------------------------|
| KiCad  | **M** (Move)             | **D** (Drag, PNS-aware)       |
| KiCad  | -                         | **G** (Drag Free Angle)       |
| Altium | **M** (Move) for non-tracks | **Ctrl+drag** or **E M E** (Move Edit) |
| Altium | -                         | drag endpoint freely after click |
| Allegro| **Move**                  | **Slide**                     |
| PADS   | **M**                     | **Move Sequence (MS)**        |
| Eagle  | **MOVE**                  | (no separate slide)           |
| Horizon| Move tool                 | PNS-driven Drag tool          |

KiCad's **G** (Drag Free Angle) is unusual in that it explicitly
breaks 45° and ignores rules — a deliberate "let me touch this
exactly" affordance. Other tools achieve the same with a modifier key
(Shift to suppress snap).

"With pad glide" — moving a footprint and dragging connected tracks
along — is enabled per-tool:
- **Altium**: Preferences → PCB Editor → Interactive Routing → "Drag
  Connected Tracks" + Mode (Skew / Right Angle / 45° / Any Angle).
- **KiCad**: Preferences → PCB Editor → Editing Options → "Allow free
  pads" + footprint drag mode (Stretch / Move with rules / Move
  rigidly).
- **Allegro**: "Slide" mode with "Etch Edit" parameter.

#### Cleanup / gloss conventions

Two distinct flavours:

- **Geometric cleanup** — merge collinear, remove zero-length, remove
  dangling, remove unconnected, delete tracks-in-pads. Run on demand
  via dialog. KiCad: **Tools » Cleanup Tracks and Vias**. Allegro:
  Database Doctor. Eagle: manual.
- **Topological glossing** — shorten path, reduce corner count,
  re-apply rule width. Run automatically during routing or on demand
  per selection. Altium's Glossing (three-level Off/Weak/Strong with
  Ctrl+Shift+G). Allegro's Custom Smooth. Both are tightly coupled to
  the push-and-shove engine.

KiCad has cleanup but no separate glossing pass; you re-run the router
to gloss in practice.

#### Net selection conventions

- **Click track → segment selection**; **double-click** → whole-net
  selection in Altium, Allegro. KiCad uses Edit » Select All Tracks
  in Net or Shift+click to extend.
- **Hover net → highlight** (with or without click) is the modern UX.
  KiCad: backtick. Altium: hover with a selection-filter overlay.
- **Cross-probe schematic ↔ PCB**: clicking a net in either editor
  highlights it in the other. Universal in professional tools.
- **Selection by net class / net** via **Edit » Find** dialog in every
  tool. KiCad has Selection Filter; Altium has PCB Filter panel
  (very powerful query language: `IsTrack and OnLayer('Top') and
  InNet('GND')`).
- **Group by layer**: filter selection to one layer in Altium (PCB
  Filter), KiCad (Selection Filter), Allegro (Find filter).

### Performance strategies

- **GPU-instanced primitives** are the modern norm. KiCad uses GAL
  (Graphics Abstraction Layer) over OpenGL or Cairo; Horizon uses raw
  OpenGL with shared shader pipelines. Each track / via / pad becomes
  a per-instance vertex emit, with the shader resolving colour,
  width, and corner caps from per-instance data.
- **LOD culling**: small-on-screen primitives are skipped, drawn as
  outline-only, or batched into a single-pixel hint. KiCad uses
  `lodScaleForThreshold` per object class.
- **Layer-based batching**: all primitives on one layer rendered in a
  single pipeline run with the layer colour bound once. Horizon's
  `triangle-ubo.glsl` and KiCad's `pcb_painter.cpp` both do this.
- **Culling by viewport**: only primitives intersecting the visible
  region are drawn. Standard.
- **Zone fill caching**: filled zones store their computed
  `SHAPE_POLY_SET` (KiCad) or `Fragment` deque (Horizon) — recompute
  only happens when the zone outline or net or any contained obstacle
  changes. Allegro Dynamic shapes refill incrementally as edits land,
  affecting only the affected fragment.
- **Track-as-quad batching**: each track segment becomes 4 vertices
  (a screen-space quad with per-vertex round-cap parameters); 100k
  tracks = 400k vertices = trivial for a modern GPU.
- **Drag throttling**: heavy live-update operations (zone refill, zone
  net-tie recompute, plane refresh) suspend during interactive drag
  and resume on mouse-pause. Universal in modern tools.
- **Static vs Dynamic** zone modes (Allegro) are an explicit perf
  knob: dynamic refills live (cost) for visual correctness; static
  refills only on demand (saved cost) at the price of stale visuals.

## Industry Standards & Data Model Notes

The data model the engine exposes for copper has direct downstream
implications for **Gerber X3**, **ODB++**, and **IPC-2581** output —
the three formats the board fab will see. Datum's existing copper
model (`Track { from, to, width, layer }`, `Via { position, drill,
diameter, from_layer, to_layer }`, `Zone { polygon, layer, priority,
thermal_relief, thermal_gap, thermal_spoke_width }`,
`PlacedPad { ... }` per `crates/engine/src/board/board_types.rs`)
maps cleanly to all three at the conceptual level, but each format
imposes constraints worth noting.

- **Gerber X3** (the current format that the fab almost certainly
  uses): each copper layer becomes a separate Gerber file. Track
  segments emit as `D01` (interpolate) commands with a defined
  aperture (the track width). Vias emit as `D03` (flash) of an
  annulus aperture per layer. Pads emit as `D03` of the pad-shape
  aperture. Zones emit as `G36/G37` region fills.
  Gerber X2 added the `.AperFunction` attribute (e.g.
  `Conductor`, `ViaPad`, `SMDPad`) and `.N` net attribute; Gerber X3
  formalised these and added `.C` component attributes. The data
  model needs a per-primitive **role** (track/via/pad/zone) and a
  per-primitive **net** to emit X3 cleanly. Datum already has both.
  ([ODB++ vs IPC-2581 vs Gerber X3](https://resources.altium.com/p/pcb-production-file-format-wars))
- **ODB++**: a directory tree (or .tgz) holding per-layer artwork,
  drill data, BOM, netlist. Copper layers each have a `features`
  file that enumerates Lines, Arcs, Pads, Surfaces (the analogue of
  Gerber `D01/D03/G36`). ODB++ emits one drill file per drill span
  (through, blind/buried tiers), so the data model must distinguish
  via spans clearly.
- **IPC-2581** (DPMX): single XML file containing geometry,
  stack-up, BOM, netlist, fab notes. Copper appears under
  `LayerFeature` blocks with `Set` / `Feature` elements typed as
  `Line` / `Pad` / `Surface`. IPC-2581 explicitly carries the **net
  name** on every copper feature (Datum's per-primitive net field
  maps directly).
  ([IPC-2581 guide](https://www.nextpcb.com/blog/ipc-2581-guide))
- **Padstack representation** is the biggest difference between the
  model styles. Allegro / PADS / IPC-2581 / ODB++ all store a single
  *padstack* with per-layer apertures (regular / anti-pad / thermal)
  and per-layer mask / paste / silk. KiCad/Eagle store a *footprint
  pad* with derived per-layer geometry. Datum's `PlacedPad` should
  carry an explicit padstack reference to ease export to ODB++ /
  IPC-2581 even if the on-board representation is per-layer-derived;
  otherwise the export adapter has to invent padstacks at emit time.
- **Negative plane semantics** live in Gerber as a `LP` (level
  polarity) command (`%LPC*%` clear, `%LPD*%` dark). IPC-2581 and
  ODB++ both prefer **positive copper representation** with
  computed knockouts; the negative-plane model survives mostly
  for legacy Allegro/PADS workflows. Datum picking positive zones
  is the right call for output portability.
- **Net attribute propagation**: every copper primitive must carry
  its net attribute through to fab output for X3 / IPC-2581 net-aware
  fab and assembly. Datum's stable net IDs make this easy — emit the
  net name (or net ID) as the standard `.N` attribute on every
  emitted feature.
- **Drill file conventions**: separate Excellon files per drill span
  (PTH / NPTH / blind tiers). Datum's `Via { from_layer, to_layer }`
  span model maps to one drill file per distinct span pair.

## Academic & Open-Source Implementation Notes

- **KiCad PNS (Push and Shove router)** is the closest thing the open
  community has to a state-of-the-art interactive router. Originally
  developed by Tomasz Włostowski at CERN; the algorithm is a
  topology-aware shove that operates on a graph of "joints" between
  segments. Source under `pcbnew/router/`:
  - `pns_router.cpp` — top-level routing controller
  - `pns_shove.cpp` — shove algorithm (the core push-and-shove logic)
    ([pns_shove source](https://docs.kicad.org/doxygen/pns__shove_8cpp_source.html))
  - `pns_walkaround.cpp` — walk-around obstacle bypass
  - `pns_dragger.cpp` — drag-with-rules logic
  - `pns_topology.cpp` — joint graph and topology preservation
  - `pns_routing_settings.h` — `MODE` enum (RM_MarkObstacles /
    RM_Shove / RM_Walkaround) and `OPTIMIZATION_EFFORT`
  Forks exist (LibrePCB has discussed adopting it; Horizon ships it
  via `pns_horizon_iface.cpp`).
- **KiCad zone fill** lives in `pcbnew/zone_filler.cpp` (`ZONE_FILLER`
  class). Algorithm:
  1. Build the zone outline as a `SHAPE_POLY_SET`.
  2. Inflate / deflate by clearance, knockout other-net features
     (`SHAPE_POLY_SET::BooleanSubtract`).
  3. Add thermal-relief spokes for same-net pads.
  4. Fracture the resulting polygon into convex outputs
     (`SHAPE_POLY_SET::Fracture`).
  5. Optionally apply hatch pattern (`addHatchFillTypeOnZone`).
  ([KiCad zone filler](https://docs.kicad.org/doxygen/classZONE__FILLER.html))
- **Horizon plane fill**: similar pipeline in
  `board/plane_update.cpp` (Clipper-based booleans). The big
  Horizon-specific touch is the `Fragment` deque so each disconnected
  piece is tracked independently and orphans can be flagged.
- **FreeRouting** (Java open-source autorouter): operates on
  Specctra DSN format input, produces SES output. Three routing
  modes: 90° / 45° / Free Angle. Uses a topological autoroute
  (rip-up and reroute with cost metric) rather than a real
  push-and-shove. Quality is widely considered the best open-source
  autorouter for KiCad; there is also a CLI/API mode.
  ([FreeRouting GitHub](https://github.com/freerouting/freerouting))
- **Specctra DSN** is the open file format for routing exchange.
  Cadence's original 1990s SPECCTRA autorouter format; widely
  supported (KiCad export, Eagle export, Pulsonix, etc.). DSN
  describes structure (board outline, layers, via padstacks, nets,
  classes); SES describes the routing solution (wires + vias). DSN
  is the closest thing the industry has to a portable
  routing-input format.
- **Topology-aware routing literature**:
  - Cong, Madden, Naclerio, Pan (2001), *Performance-Driven
    Global Routing for Deep Sub-Micron VLSI Designs* — global
    routing topology that informs PNS-class router design.
  - Müller-Hannemann, Schulze (2007), *Hardness and Approximation
    of Octilinear Steiner Trees* — relevant to 45° topology
    optimisation in routing.
  - The KiCad PNS doxygen and source comments are the most
    accessible documentation of a working push-and-shove
    implementation; no published academic paper covers the KiCad
    PNS specifically, but the algorithm is rooted in
    Włostowski's CERN PCB design work circa 2013.
    ([KiCad PNS source mirror](https://github.com/rnestler/pns-router))
- **Padstack standardization**: IPC-7351 defines pad-shape land
  patterns; IPC-7351B extends with calculated thermal relief
  guidance.
- **Curved tracks**: implemented in KiCad 7+ via track-as-arc
  primitives; rendered with the same round-cap stroke pipeline.
  The arc primitive is `PCB_ARC` (extends `PCB_TRACK`).

## User Pain Points & Wishlist Items

Distilled from KiCad forums, EAGLE/Fusion forums, Altium community,
DipTrace forum, EasyEDA forum, Cadence Community, and (where searchable)
r/PrintedCircuitBoard and r/PCB.

1. **"My zone fills are stale" is a chronic source of confusion.**
   Users edit a track that voids a zone, save, send to fab, and only
   notice in CAM that the fill is wrong. KiCad addressed this with a
   "fill on save" preference; Altium ships a "Repour Modified Polygons
   on Reroute" option. Datum should default to fill-on-save *and*
   surface a stale-zone visual marker (matches the airwire stale
   marker recommendation).
2. **"Push-and-shove broke my topology."** Users routing critical
   nets (clocks, diff pairs) want an explicit "lock this trace"
   affordance so push-and-shove won't disturb it. Every tool has lock
   buttons but they're often hidden in dialogs; Pulsonix and PADS
   surface them prominently. Datum should make per-track lock a
   one-click affordance.
3. **Thermal relief vs solid connection invisible.** When a zone
   fills over a same-net pad, whether the connection is solid or
   thermal-relief is determined by the rule, but most users don't
   realise the distinction until DRC fails or the board doesn't
   solder properly. KiCad's thermal-spoke visual is the model; Datum
   should always render the actual computed spoke geometry, never a
   schematic stand-in.
4. **"Why is this pad not connected to my pour?"** A recurring KiCad
   forum thread. Causes: pad on wrong layer, zone priority shadowing,
   thermal-relief minimum-spoke rule failed, clearance violation
   blocked the connection. Datum's "Why is this airwire here?"
   diagnostic concept extends naturally to "Why is this pad
   disconnected from this zone?".
5. **Pour update lag on large boards.** Recomputing all zone fills on
   a 6-layer 10000-pin board takes seconds in KiCad / Altium / Eagle.
   Solutions: per-zone incremental refill (only the zone whose outline
   changed), drag-throttling (suspend refill during interactive
   moves), GPU-side zone clipping (research-grade). Allegro Dynamic
   shapes ship the perf already; the rest of the industry is
   catching up.
6. **Differential pair gap maintenance is fragile.** Push-and-shove
   sometimes breaks the pair gap; user has to ripup and redo. Altium's
   ActiveRoute is the highest-quality remediation. KiCad gets
   complaints regularly.
7. **Rip-up granularity.** Users want to ripup *the unrouted-end half
   of a partial route* without ripping the whole net. KiCad has it
   (Edit » Selection Filter + Delete on selected segments). Altium has
   "Un-route Connection". Most tools have it; the discoverability
   varies.
8. **Slide that respects the design rules.** Users want to drag a
   track *and have the router shove other tracks aside* — not just
   bend at the dragged endpoint. KiCad D + Shove mode does this;
   Altium Ctrl+drag does this. The discovery cost is high — users
   often don't realise the drag-with-rules mode exists.
9. **Pad shape limitations.** EasyEDA users complain that polygon
   pads can't be hatched. Eagle users complain that pads can't have
   per-layer shapes (i.e. a different shape on F.Cu vs B.Cu) — a
   limitation Altium and KiCad both addressed years ago. KiCad users
   want chamfered-rect pads on a per-corner basis (currently all-or-
   nothing); KiCad 9 partially addressed this.
10. **Net-class colour vs net colour conflict.** Users want net-class
    colour for a default and per-net override; tools that only offer
    one or the other (older KiCad, EasyEDA) get complaints. The
    KiCad approach (None / Ratsnest / All applied to net OR
    net-class, with override hierarchy) is the model.
11. **Inactive-layer dim factor.** KiCad's High Contrast factor
    defaults to ~50% which many users find too dim or too bright;
    common preference change. Altium ships per-layer monochrome as a
    workaround. Make the dim factor a slider, not a fixed value.
12. **Via-in-pad rendering.** When a via sits on top of a pad, which
    one wins visually? KiCad ghosts the via through the pad; Altium
    draws the pad on top. Both choices have advocates. Default
    "ghost via through pad" tends to be more readable for placement
    review; "via on top of pad" is more like the actual fabricated
    appearance. Make it a preference.
13. **Curved tracks for RF.** RF designers want a smooth-curve
    routing tool. KiCad 7 added arc-track primitives; the routing
    tool support is partial as of 9.x. Altium has had it for years
    via Any Angle + Curved corner mode.
14. **Length-tuning UX is too hidden.** Users coming from Altium to
    KiCad regularly ask "how do I tune length, where is the tool?"
    Better discoverability — a Routing → Tune Length submenu, an
    obvious right-toolbar button, an in-canvas "tune" widget on
    hover-over-selected-trace — would help.
15. **Net highlight conflicts with cross-probe.** Cross-probing a
    net from schematic shouldn't always *change* the active net
    highlight; sometimes the user wants to add to a multi-highlight
    set. Altium has multi-net highlighting; KiCad and most others
    are single-highlight only.
16. **"I can't see my copper through the zone."** Default zone
    opacity (KiCad's 70%) is sometimes too opaque, making routed
    tracks under a zone invisible. Per-class opacity sliders are the
    fix; KiCad ships them.

## Recommendations for Datum EDA

Concrete guidance for the M7 GUI copper implementation. References to
existing engine state come from
`crates/engine/src/board/board_types.rs` (Track, Via, Zone, NetClass)
and the M7 contracts in `docs/gui/M7_RENDER_SEMANTIC_CONTRACT.md`.

### Algorithm / Engine

1. **Positive zones, computed fills, per-fragment storage.** Match
   Horizon's `Plane::Fragment` model: each computed fill piece is a
   `(outline_path, holes[])` tuple; orphan fragments (no pad/via
   contact) are tracked independently. Negative planes are not in
   v1 and probably never; the data-model and output-format reasons
   align with positive zones.
2. **Per-zone incremental refill.** A `dirty_zones` set on the engine
   API; a refill operation rebuilds only those zones. Mirror the
   airwire `(fast, only_set)` parameterisation: `fast` skips
   thermal-spoke fitness for interactive drag.
3. **Stable per-fragment IDs.** Each computed fragment gets a
   deterministic ID derived from its outline + zone UUID; the
   renderer can hold per-fragment GPU buffers and only re-upload the
   ones whose ID changed across refill calls. Same determinism
   invariant as the airwire output.
4. **Track-as-arc primitive.** Add `Track::arc_center: Option<Point>`
   so curved tracks ship native, not as Bezier-of-segments. This
   matches KiCad / Altium / Eagle and avoids a future schema
   migration when curved RF routing becomes a Datum requirement.
5. **Padstack reference on `PlacedPad`.** Even if the in-board model
   stores per-layer pad geometry, carry an opt-in `padstack_uuid`
   that points at a canonical padstack so ODB++ / IPC-2581 export can
   emit a clean padstack table. Cheap to add now; expensive to add
   after fab-output adapters depend on the current model.
6. **Via span as `LayerRange`.** Datum's existing `Via { from_layer,
   to_layer }` is correct for through; for blind/buried/micro the
   pair must mean "drilled from layer X, terminating at layer Y" with
   the intermediate copper rings *implicit*. Document this invariant
   explicitly so the renderer knows which annulus rings to draw on
   intermediate layers (the answer: only on `from_layer` and
   `to_layer`; intermediate layers don't carry the via pad unless
   the design opts into stacked microvias).
7. **Net attribute everywhere.** `Track`, `Via`, `Zone`, `PlacedPad`
   already carry `net: Uuid`; preserve this through fab-export
   adapters as Gerber `.N` / IPC-2581 net names.
8. **Connectivity authority.** As with airwires, the
   connectivity graph that drives DRC, ratsnest, and zone-fill
   pad-inclusion must be the same graph. A pad inside a zone that
   "should" connect but doesn't (because of layer mismatch, priority
   shadowing, or rule failure) needs a per-pad reason payload that
   the renderer can surface as the "Why is this pad disconnected
   from this zone?" diagnostic.
9. **Determinism**: copper output (track list, via list, zone
   fragment list) must be deterministic across runs for a given
   board state. Sort by stable UUID. Fragment ordering inside a
   zone sorted by `(area_descending, centroid)` for stable
   screenshot goldens.

### Renderer

10. **Tracks as round-cap capsules**, GPU-instanced. One pipeline
    pass per layer, with per-instance `(from, to, width, color)`.
    Mitred 45° corners by default; rounded-corner mode as a global
    preference. Anti-aliased.
11. **Vias as concentric annulus + drill marker**. Same instanced
    pipeline (a via is a "stationary capsule with drill"). Tented
    vias suppress the mask aperture in the mask layer's pass — no
    special-case in the copper layer's pass.
12. **Pads as filled polygons**, with shape = (Circle, Rect,
    Roundrect, Oval, Trapezoidal, Chamfered, Custom). A canonical
    enum + a `Custom(Polygon)` escape. Through-hole pads emit one
    pad polygon per copper layer they pass through.
13. **Zones as `(outline_path, holes[])` per fragment**. Filled
    polygon with holes rendered through GPU stencil or polygon
    triangulation (earcut). Solid by default; hatched as
    parallel-line pattern if `PlaneSettings::fill_style == HATCH`.
14. **Thermal relief renders the actual computed spoke geometry** —
    not a schematic stand-in, not a stylised badge. The spokes are
    part of the zone fragment outline. Configurable spoke count,
    width, gap, and angle in the rule.
15. **Per-net colour with `color_mode: { None, Copper, All }`.**
    Match KiCad's `None / Ratsnest / All` model but with copper as
    the second option (Datum already ships airwire color via the
    airwire contract; copper is the symmetric copper-side toggle).
16. **Default layer colour scheme**: F.Cu = red, B.Cu = blue, inner
    layers = yellow / cyan / magenta / violet rotation. Match the
    industry informal standard so users coming from KiCad / Altium
    feel at home immediately. Ship a few alternative themes
    (high-contrast monochrome, dark + bright, photonics-friendly
    etc.) but default to red/blue.
17. **Layer ordering in panel**: stack-up order, top → bottom.
    Active layer marked with bold text *and* a coloured indicator
    arrow next to its swatch. Status bar shows active layer name
    continuously.
18. **High-contrast / single-layer mode**: default hotkey
    **Ctrl+H** (KiCad muscle memory) cycles **Normal → Dim →
    Hidden** for inactive copper layers. Dim factor is a slider in
    Preferences (default 50%). Process layers (mask, paste, silk)
    follow the same dim setting.
19. **Per-object opacity sliders** (Tracks / Vias / Pads / Zones)
    in the Appearance panel. Default zones to ~70% so underlying
    tracks remain visible; everything else 100%.
20. **LOD culling** for tracks and vias below a screen-pixel
    threshold (KiCad's `lodScaleForThreshold` pattern). Filled
    zones never LOD-cull (they're the dominant area; culling them
    breaks layer recognition).
21. **Crosshair colour matches active layer colour** during routing.
    Subtle but high-impact for layer awareness.
22. **Stale-zone marker**: zones whose computed fill is older than
    the most recent edit affecting their net (or polygon, or
    contained obstacles) render with a subtle "stale" treatment
    (slightly desaturated, optional outline overlay). Removes the
    "did the fill update?" anxiety class. Maps to the airwire
    stale-marker recommendation.
23. **Render stack for copper** (per `M7_RENDER_SEMANTIC_CONTRACT.md
    § Stack Rule`): authored copper goes in lane 1 (with back-side
    layers drawn first, then front). Tracks above zones within a
    layer (so airwires bridging tracks-on-zone read correctly).
    Vias above tracks within a layer. Pads above tracks within a
    layer (so the pad annulus is always visible).

### Creation tools

24. **Routing modes: cycle hotkey on day one.** Match Altium's
    **Shift+R**: Highlight Collisions / Walkaround / Push (Shove) /
    HugNPush. Even if the underlying engine doesn't ship all four
    initially, register the hotkey and HUD vocabulary so users
    coming from Altium reach for the right key immediately.
25. **Modeless routing as the default.** Click a route button, the
    cursor becomes a routing cursor, all other commands stay
    available. Esc exits.
26. **Via insertion: `V` for through, `Shift+V` for microvia,
    `Alt+Shift+V` for blind/buried.** Match KiCad. Auto-via on layer
    change as a preference (default on).
27. **Layer change: `+` / `-` cycles enabled signal layers,
    `PgUp` / `PgDn` jumps to F.Cu / B.Cu** (KiCad muscle memory).
28. **Trace bend / posture: `/` toggles diagonal-first vs
    straight-first** (KiCad muscle memory) and **`Spacebar`
    rotates corner direction** (Altium muscle memory). Both are
    cheap to implement and serve different mental models.
29. **Differential pair routing**: dedicated tool button +
    hotkey (`6` per KiCad). Pair pickup from net-name suffix
    (`_P/_N`) initially, plus opt-in via constraint manager once
    Datum has a constraint surface.
30. **Length tuning hotkeys**: `7` (single), `8` (diff pair),
    `9` (diff pair skew). Match KiCad. Live serpentine preview
    with parameters on a right-toolbar pop-out.
31. **Zone creation**: a single Add Filled Zone tool
    (**Ctrl+Shift+Z** per KiCad). Outline drawing → properties dialog
    (net, layer, fill type, thermal relief settings). Default to
    fill-on-save and surface a "Refill all zones" action (**B**
    hotkey).
32. **Snap behaviour**: grid snap default; object snap to pad
    centre / track endpoint / via centre / intersection /
    midpoint, configurable per-snap-type. Crosshair shape
    consistent across tools.

### Editing tools

33. **Two drag modes, named clearly.** **D** = Drag with rules
    (PNS-aware: shoves, walks-around per current routing mode).
    **G** = Drag Free Angle (breaks 45°, ignores rules, splits
    segment). Match KiCad. Document the distinction in the HUD
    when the tool is invoked.
34. **Move with track-following on by default.** Moving a footprint
    drags connected tracks (PNS-aware) so the design "stays
    connected". Right-click on the move tool offers Rigid Move as
    an alternative.
35. **Cleanup Tracks and Vias dialog** with options: Merge Collinear,
    Delete Dangling, Delete Tracks-in-Pads, Remove Redundant Vias,
    Refill Zones After Cleanup. Match KiCad's option set; users
    coming from KiCad will recognise it.
36. **Glossing pass** (post-route shorten + corner-reduction). Run
    inline during routing as a preference (Off / Weak / Strong, per
    Altium). Run on demand on selected routing via a Tools menu.
37. **Rip-up affordances**: per-segment (Delete on selection),
    per-net (right-click on net → Unroute), per-component
    (right-click on component → Unroute), per-area (rectangle select
    + Delete). All four idioms; users coming from any tool will
    reach for one of them.
38. **Net selection cascade**: click a track segment selects the
    segment; double-click extends to the whole-net path.
    Backtick (**`**) highlights the net under cursor (bright + dim
    everything else); **Ctrl+`** toggles highlight without
    re-picking. Match KiCad.
39. **Selection filter** to restrict mouse-drag selection by object
    class (tracks-only, pads-only, vias-only, zones-only, etc.).
    KiCad/Altium pattern; high productivity payoff for a small
    surface investment.

### Datum differentiators

40. **"Why is this pad disconnected from this zone?" diagnostic.**
    Same family as the airwire diagnostic. On hover-shift over a
    pad-near-a-zone-on-the-same-net but not connected, surface
    plain text from the connectivity engine: "This pad on F.Cu is
    separated from the GND zone on F.Cu by 0.3mm; clearance rule
    requires 0.2mm — should connect, but doesn't because zone
    priority 5 is shadowed by zone priority 10 on this region."
    Datum's deterministic-routing substrate already carries the
    necessary data.
41. **AI-accessible copper surfaces.** The copper primitive list, the
    zone fragment list, the per-zone fill diagnostics, and the
    route-mode HUD must all be MCP-queryable. An agent should be
    able to ask "list all tracks on net DDR_DQ0 sorted by length"
    and "explain why the +5V pour orphan fragment exists" without
    screen-scraping.
42. **Stable copper IDs across schematic-driven net rename.** A
    track tagged to net `NET1` retains its identity (and its
    per-net colour, lock state, length tuning) when the net is
    renamed to `DDR_CLK`. Trivial given the canonical IR's stable
    UUIDs; users moving from name-keyed tools (Altium, Allegro)
    will appreciate it.
43. **Print / export copper artwork with annotation.** PDF / SVG /
    PNG board snapshots from the CLI, with per-net colour, layer
    isolation, and a key/legend. Mirrors the airwire export
    recommendation; copper is the obvious next step.
44. **Lock affordance is a one-click toggle.** Per-track lock and
    per-zone lock surface as a context-menu top-level action. The
    lock icon renders adjacent to the locked object (Horizon
    pattern). Push-and-shove honours the lock automatically.
45. **Live zone refill with stale marker fallback.** Default to
    live refill on edit; if the refill cost exceeds a threshold
    (huge zone, big drag), fall back to async-refill + stale marker
    until the next quiet frame. Best of both worlds (Allegro
    Dynamic + KiCad explicit-refill).
46. **Per-fragment zone diagnostics.** For a multi-island fill,
    surface each fragment's pad/via inclusion in a panel. Orphans
    are flagged; same-net orphans get a "Is this intentional?"
    prompt the user can dismiss.

### Out of scope for M7

- **Negative plane primitive.** Positive zones only. Out of v1.
- **Real-time autorouter / batch autoroute.** Datum's M5 routing
  substrate exists but the M7 GUI integration is for *interactive*
  routing only. Auto-route comes later.
- **Curved tracks during interactive routing.** Track-as-arc data
  primitive should ship in M7 so the schema is correct;
  interactive curved-routing tool is M8.
- **Differential pair gap-tunable per segment.** Constant gap from
  rule for M7. Per-segment gap override is later.
- **High-density via-in-pad with per-via plating-fill spec.** Data
  model can carry the flag but the renderer needn't visualise it
  beyond "tented / not tented".
- **3D copper visualisation.** 2D only for M7.

## Sources

KiCad documentation & source:
- [KiCad 9.0 PCB Editor manual](https://docs.kicad.org/9.0/en/pcbnew/pcbnew.html) — pad shapes, zone fill, routing tools, layer panel, high-contrast mode
- [KiCad 8.0 PCB Editor manual](https://docs.kicad.org/8.0/en/pcbnew/pcbnew.html) — zone fill controls, track properties, hotkeys
- [KiCad COLOR_SETTINGS doxygen](https://docs.kicad.org/doxygen/classCOLOR__SETTINGS.html) — colour theme model, default scheme cascade
- [KiCad ZONE_FILLER doxygen](https://docs.kicad.org/doxygen/classZONE__FILLER.html) — zone fill algorithm and primitives used
- [KiCad TRACKS_CLEANER doxygen](https://docs.kicad.org/doxygen/classTRACKS__CLEANER.html) — cleanup options
- [KiCad DIALOG_CLEANUP_TRACKS_AND_VIAS](https://docs.kicad.org/doxygen/classDIALOG__CLEANUP__TRACKS__AND__VIAS.html) — cleanup dialog options
- [KiCad PCB_DISPLAY_OPTIONS doxygen](https://docs.kicad.org/doxygen/classPCB__DISPLAY__OPTIONS.html) — display option enums
- [KiCad pns_walkaround.h source](https://docs.kicad.org/doxygen/pns__walkaround_8h_source.html) — walkaround mode
- [KiCad pns_shove.cpp source](https://docs.kicad.org/doxygen/pns__shove_8cpp_source.html) — shove algorithm
- [KiCad PNS namespace](https://docs.kicad.org/doxygen/namespacePNS.html) — interactive router class hierarchy
- [KiCad pcbnew_layers source](https://github.com/KiCad/kicad-doc/blob/master/src/pcbnew/pcbnew_layers.adoc) — layer documentation
- [KiCad routing modes wishlist #9414](https://gitlab.com/kicad/code/kicad/-/issues/9414) — push/shove cycle hotkey discussion
- [KiCad layer colour theme forum](https://forum.kicad.info/t/pcb-layer-colour-theme-per-project/46804) — per-project colour discussion
- [KiCad track end style forum](https://forum.kicad.info/t/changing-track-end-style-settings-in-pcbnew-square-end-traces-supported/2409) — round-cap rationale
- [KiCad drag hotkeys discussion](https://groups.io/g/kicad-users/topic/how_can_i_drag_traces_in/81002705) — D vs G semantics
- [KiCad cleanup tracks forum](https://forum.kicad.info/t/pcb-cleanup-tracks-and-vias/19177) — usage patterns
- [KiCad-color-schemes repo](https://github.com/pointhi/kicad-color-schemes) — community colour themes survey
- [Tech Explorations KiCad diff pair tuning](https://techexplorations.com/guides/kicad/high-speed-pcb-design/kicad-9-differential-pair-length-tuning-guide/) — hotkeys 7/8/9
- [Sierra Circuits KiCad routing](https://www.protoexpress.com/blog/how-to-route-a-pcb-in-kicad/) — workflow overview
- [Tech Explorations KiCad interactive routing](https://techexplorations.com/blog/kicad/blog-kicad-interactive-routing/) — D/G hotkeys
- [KiCad bug 1835276 high contrast mode](https://bugs.launchpad.net/bugs/1835276) — Ctrl+H + dim factor

Altium documentation:
- [Altium Designer routing modes](https://www.altium.com/documentation/altium-designer/pcb/routing/interactive) — Shift+R, conflict resolution modes
- [Altium Custom Pad Stack](https://www.altium.com/documentation/altium-designer/pcb/custom-pad-stack) — pad shape options, padstack model
- [Altium Custom Pad Shape KB](https://www.altium.com/documentation/knowledge-base/altium-designer/custom-pad-shape) — custom pad creation
- [Altium Working with Pads & Vias](https://www.altium.com/documentation/altium-designer/pcb/pads-vias) — pad/via reference
- [Altium Mask Rule Types](https://www.altium.com/documentation/altium-designer/pcb/design-rule-types/mask) — via tenting controls
- [Altium Push and Shove blog](https://resources.altium.com/p/push-and-shove-router-how-it-works-and-why-you-need-it) — Shift+R cycle, mode set
- [Altium Diff Pair Routing](https://www.altium.com/documentation/altium-designer/pcb/high-speed-design/interactively-routing-differential-pairs) — pair pickup, length tuning
- [Altium Modifying Routing](https://www.altium.com/documentation/altium-designer/modifying-the-routing-ad) — drag with rules
- [Altium Glossing & Retracing](https://www.altium.com/documentation/altium-designer/glossing-retracing-existing-routes-pcb) — Ctrl+Shift+G, Off/Weak/Strong
- [Altium Single Layer Mode (Shift+S)](https://www.altium.com/documentation/altium-designer/pcb/your-view-of-the-board) — single-layer modes
- [Altium View Modes whitepaper](https://resources.altium.com/p/pcb-editor-view-modes) — display mode reference
- [Altium My Favorite Hotkeys](https://resources.altium.com/p/my-favorite-altium-designer-keyboard-shortcuts-and-viewing-features) — hotkey survey
- [Altium Net Highlight Color](https://www.altium.com/documentation/altium-designer/sch-pcb/using-net-highlight-color) — F5, per-net colour
- [Altium Net Color Override dialog](https://www.altium.com/documentation/altium-designer/sch-dlg-netcoloroverridenet-color-override-ad) — override mechanics
- [Altium Net Colors article](https://resources.altium.com/p/breaking-the-visual-barrier) — workflow, layer-colour interaction
- [Altium Thermal Relief Design Guide](https://resources.altium.com/p/thermal-relief-design) — thermal connect styles
- [Altium Tented vs Untented Vias](https://resources.altium.com/p/when-use-tented-vias-your-pcb-layout) — tenting visualisation
- [Altium Layer Stack Manager docs](https://www.altium.com/documentation/altium-designer/pcb-dlg-layerstackmanagerlayer-stack-manager-ad) — stack-up management
- [Altium Toggle Visibility](https://resources.altium.com/p/toggle-visibility-layers-and-objects) — layer panel
- [Altium View Configuration panel](https://resources.altium.com/p/view-configuration-panel) — per-class display

Cadence Allegro / OrCAD X:
- [OrCAD X Routing Modes](https://resources.pcb.cadence.com/layout-and-routing/orcad-x-routing-modes) — Hug / Shove / Standard
- [How to Route Boards in OrCAD X Presto](https://www.ema-eda.com/how-to-page/how-to-route-boards-in-orcad-x-presto/) — Add Connect, mode dropdown
- [How to Quickly Route in Presto](https://www.ema-eda.com/how-to-page/how-to-quickly-route-connections-in-orcad-x-presto/) — Spread Between Voids, Assisted modes
- [OrCAD X Visibility Pane](https://resources.pcb.cadence.com/blog/2024-navigating-the-visibility-pane-in-orcad-x-presto-pcb-editor) — per-layer/net colour
- [Allegro Routing & Glossing lesson](http://education.ema-eda.com/iTrain/PCBEditor163/lesson_10.html) — Custom Smooth gloss
- [Allegro PCB editor blog](https://resources.pcb.cadence.com/blog/2020-routing-with-allegro-pcb-editor) — routing workflow
- [Allegro shortcut keys cheat sheet](https://www.ema-eda.com/ema-resources/blog/allegro-keyboard-shortcut-cheat-sheet/) — hotkeys
- [Allegro shape settings PDF](https://www.parallel-systems.co.uk/wp-content/uploads/2020/02/Shape_Settings.pdf) — dynamic vs static shapes
- [Allegro split plane blog](https://resources.pcb.cadence.com/blog/2021-split-plane-routing-in-cadence-s-allegro-pcb-editor) — negative plane workflow
- [Allegro padstacks lesson](http://education.ema-eda.com/iTrain/PCBEditor163/lesson_3.html) — padstack model
- [Cadence PCB thermal relief](https://resources.pcb.cadence.com/blog/2021-pcb-thermal-relief-guidelines-for-effective-layouts) — thermal relief design
- [PCB PRO Allegro colour selection](https://becomepcbpro.com/ColourSelection.html) — colour scheme management
- [PCB PRO Allegro shapes](https://becomepcbpro.com/Shapes.html) — shape workflow
- [Cadence OrCAD X delay tune](https://www.ema-eda.com/how-to-page/how-to-add-delay-tuning-in-orcad-x-presto/) — Trombone, Sawtooth styles
- [Cadence diff pair length matching](https://resources.pcb.cadence.com/blog/2025-differential-pair-length-matching-guidelines) — pair routing guidelines
- [Cadence serpentine routing](https://resources.pcb.cadence.com/blog/2019-serpentine-routing-tips-to-snake-in-your-tuned-traces) — meander geometry
- [OrCAD X high-speed trace routing](https://resources.pcb.cadence.com/blog/2025-high-speed-trace-routing) — diff pair routing 2025
- [OrCAD X 2024 features](https://www.flowcad.de/datasheets/24-1-Whats-New.pdf) — 24.1 release notes

PADS / Xpedition (Mentor / Siemens):
- [PADS Layout tutorial PDF](http://www.theky22.com/downloads/pads%20layout%20tutorial%20-%20sgi%20-%20pcb%20design.pdf) — pad shapes, plane areas, copper pour
- [Siemens PADS routing automation](https://blogs.sw.siemens.com/pads/2019/04/05/reduce-design-time-using-routing-automation-part-2/) — push-and-shove, length tuning
- [Siemens manual length tuning](https://blogs.sw.siemens.com/electronic-systems-design/2014/10/27/pcb-routing-solutions-simplifying-the-tuning-process-part-2-manual-tuning/) — yellow/green/red feedback
- [Siemens diff pair rules guide](https://blogs.sw.siemens.com/xcelerator-academy/2023/08/02/how-to-master-the-game-of-setting-rules-differential-pairs/) — diff pair routing
- [Xpedition PCB layout PDF](https://sitecore.vargroup.com/-/media/Project/Var-Group-Tenant/Var4Industries/PDF/e_book/Cadlog/Siemens-SW-Xpedition-PCB-layout.pdf) — Xpedition feature overview

Eagle / Fusion 360 Electronics:
- [Eagle to Fusion migration guide](https://www.autodesk.com/products/fusion-360/blog/from-autodesk-eagle-to-fusion-360-10-things-to-know/) — feature continuity
- [Fusion Electronics walkaround/push violators KB](https://www.autodesk.com/support/technical/article/caas/sfdcarticles/sfdcarticles/Routing-with-Walkarounds-violators-and-Push-Violators-is-not-working-on-PCB-in-Fusion-Electronics.html) — push/walkaround modes
- [Fusion Electronics interactive routing blog](https://www.autodesk.com/products/fusion-360/blog/interactive-routing-with-fusion-360-electronics/) — routing toolkit
- [Eagle routing & autorouting basics](https://www.autodesk.com/products/fusion-360/blog/routing-autorouting-pcb-layout-basics-2/) — RIPUP / DRC / autorouter
- [Eagle Easily Applicable Manual v5](https://hades.mech.northwestern.edu/images/b/b4/Eagle_Manual.pdf) — historical reference
- [Configuring Eagle CAD layout](http://www2.ee.ic.ac.uk/t.clarke/EAGLE/EAGLE_configuration.htm) — layer numbering convention

Horizon EDA (sources read directly from `research/horizon-source/`):
- `src/board/track.hpp/cpp` — Track primitive with rule-derived width
- `src/board/via.hpp/cpp`, `via_definition.hpp/cpp` — via padstack model
- `src/board/plane.hpp/cpp`, `plane_update.cpp` — plane fragments, fill style, thermal settings
- `src/board/board_layers.hpp` — layer enum and stack-up convention
- `src/canvas/render.cpp` — `Canvas::render(const Track&)`, `(const Via&)`, `(const Padstack&)`, `(const BoardHole&)`, `(const Polygon&)`
- `src/canvas/canvas_pads.cpp` — pad geometry accumulation for plane voiding
- `src/router/pns_horizon_iface.cpp/hpp` — Horizon's adapter onto KiCad's PNS
- [Horizon EDA on GitHub](https://github.com/horizon-eda/horizon)
- [Horizon EDA documentation](https://docs.horizon-eda.org/en/latest/index.html)
- [Horizon EDA Board Editor docs](https://docs.horizon-eda.org/en/v2.3.0/imp-board.html)
- [Horizon EDA feature overview](https://docs.horizon-eda.org/en/latest/feature-overview.html)

Consumer tools:
- [DipTrace whats new](https://diptrace.com/diptrace-software/whats-new/) — version-by-version feature evolution
- [DipTrace copper pour forum](https://diptrace.com/forum/viewtopic.php?t=455) — workflow, thermal relief
- [DipTrace routing forum](https://diptrace.com/forum/viewtopic.php?t=11795) — Smart Routing
- [EasyEDA Copper Area docs](https://docs.easyeda.com/en/PCB/Copper-Pour/) — Shift+B refill, Shift+M outline
- [EasyEDA PCB Tools](https://docs.easyeda.com/en/PCB/PCB-Tools/) — drawing primitives
- [EasyEDA Pro Copper Region](https://prodocs.easyeda.com/en/pcb/place-copper-region/) — copper region properties
- [EasyEDA Pro Shaped Pad](https://prodocs.easyeda.com/en/pcb/place-shaped-pad/) — custom pad shape
- [EasyEDA hatched copper forum](https://easyeda.com/forum/topic/Hatched-Copper-Fill-4464424487f543c392333d236dd99f15) — hatched fill limitations
- [Quadcept routing docs](https://www.quadcept.com/en/manual/pcb/post-115) — semi-auto / push routing
- [Quadcept rats convention](https://www.quadcept.com/en/manual/pcb/post-97) — solid same-layer / dotted cross-layer
- [Quadcept route interpolation](https://www.quadcept.com/en/manual/pcb/post-45) — auto route completion
- [Quadcept differential pair](https://www.quadcept.com/en/manual/pcb/post-116) — diff pair tool
- [Pulsonix PCB design features](https://pulsonix.com/pcb-design) — push-aside, multi-trace
- [Pulsonix high-speed design](https://pulsonix.com/high-speed-design) — diff pair, length tuning
- [Pulsonix evaluation guide PDF](https://pulsonix.com/downloads/manuals/Pulsonix%20Evaluation%20Guide.pdf) — features overview

File-format / industry standards:
- [ODB++ vs Gerber X3 vs IPC-2581 comparison](https://resources.altium.com/p/pcb-production-file-format-wars) — format trade-offs
- [IPC-2581 guide](https://www.nextpcb.com/blog/ipc-2581-guide) — DPMX standard overview
- [IPC-2581 single-file PCB data](https://pcbsync.com/ipc-2581/) — data model
- [Sierra Circuits PCB file formats](https://www.protoexpress.com/kb/pcb-file-formats-overview/) — output format reference
- [Multi-CB IPC-2581 data](https://www.multi-circuit-boards.eu/en/support/pcb-data/ipc-2581.html) — fab perspective
- [Specctra DSN file format](https://pcbsync.com/dsn-specctra/) — routing exchange
- [Multi-CB via covering reference](https://www.multi-circuit-boards.eu/en/pcb-design-aid/surface/via-covering.html) — IPC-4761 via types

Algorithms & open source:
- [FreeRouting on GitHub](https://github.com/freerouting/freerouting) — autorouter, Specctra DSN/SES
- [FreeRouting documentation](https://freerouting.org/) — algorithm overview
- [KiCad PNS extracted to standalone](https://github.com/rnestler/pns-router) — PNS source mirror
- [KiCad PNS mailing list announcement](https://www.mail-archive.com/kicad-developers@lists.launchpad.net/msg08501.html) — original release context
- [Routing (EDA) Wikipedia](https://en.wikipedia.org/wiki/Routing_(electronic_design_automation)) — algorithm survey
- [TopoR Wikipedia](https://en.wikipedia.org/wiki/TopoR) — topological autorouter (Russian commercial tool with academic roots)
- [OrthoRoute KiCad GPU autorouter](https://github.com/bbenchoff/OrthoRoute) — emerging GPU-based KiCad autorouter

User pain points & wishlist:
- [KiCad bug 1414328 cleanup recursive](https://bugs.launchpad.net/kicad/+bug/1414328) — cleanup edge cases
- [KiCad bug 8326 cleanup unconnected in pads](https://gitlab.com/kicad/code/kicad/-/issues/8326) — cleanup limitations
- [KiCad bug 1787190 dangling tracks](https://bugs.launchpad.net/kicad/+bug/1787190) — cleanup behaviour
- [KiCad ground plane error forum](https://forum.kicad.info/t/ground-plane-error-thermal-relief-connection-to-zone-incomplete/41191) — thermal relief issue
- [KiCad zone fill forum](https://forum.kicad.info/t/filled-zone/56748) — fill behaviour discussion
- [KiCad cleanup forum](https://forum.kicad.info/t/cleanup-tracks-and-vias-tool/33570) — cleanup workflow
- [Altium vs KiCad page 2](https://forum.kicad.info/t/altium-vs-kicad/57136?page=2) — feature gap discussion
- [KiCad does have features I use in Altium](https://forum.kicad.info/t/does-kicad-have-the-features-i-use-in-altium/21467?page=3) — feature gap discussion
- [Sierra Circuits HDI routing challenges](https://www.protoexpress.com/blog/hdi-pcb-routing-challenges/) — routing pain points
- [eCADSTAR pain points article](https://www.ecadstar.com/en/resource/pain-points-in-schematic-and-pcb-design/) — generic pain points
