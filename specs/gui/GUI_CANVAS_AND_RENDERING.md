# Datum EDA GUI Area Specification: Canvas & Rendering

Status: draft GUI area specification, 2026-06-22, benchmarked to commercial
EDA. Controlling for the canvas-rendering domain. Conforms to and is governed
by `specs/GUI_SPEC.md` (the master). Where this area spec and the master
conflict, the master governs; where this area spec adds domain detail, it is
authoritative.

Driven by:
- `specs/GUI_SPEC.md` (master: the bar, the thesis, the four architecture
  constraints, the five-part buildability standard)
- `docs/decisions/PRODUCT_MECHANICS_012_APPLICATION_QUALITY_BAR.md`
  (`QG-PERFORMANCE-LATENCY`, `QG-ZONEFILL-HONESTY`)
- `docs/decisions/PRODUCT_MECHANICS_000B_VIEW_COMPOSITION_AND_LIVE_PRODUCTION.md`
- `docs/decisions/PRODUCT_MECHANICS_013_GUI_SUPERVISION_AND_PARITY.md`
- `crates/gui-protocol` `BoardReviewSceneV1` scene contract
- `crates/gui-render` renderer, `dim_policy.rs`, and the visual-regression
  harness (`visual_runner.rs`, `visual_manifest.rs`, `visual_diff.rs`)
- `docs/gui/M7_RENDER_LAYER_DISCIPLINE_MEMO.md` (render-stack + material-first
  discipline, M7-REN-006 enforcement slice)
- `docs/gui/M7_RENDER_SEMANTIC_CONTRACT.md` (four semantic lanes)
- `docs/gui/M7_AIRWIRE_RENDERING_GUIDANCE.md` (ratsnest contract)
- `docs/gui/DATUM_GUI_VISUAL_REGRESSION_HARNESS.md`

---

## 1. Scope

This area owns the picture: how the resolved `DesignModel`, delivered through
the `gui-protocol` scene contract, becomes pixels on a GPU-composited canvas at
the commercial bar. It owns:

- GPU pan/zoom at 60 fps on large boards
- accurate layer compositing, transparency, and the declared render-stack order
- single-layer (active-layer-on-top) and layer-set (full-stack) display modes
- net highlight / dim and per-net ratsnest (airwire) rendering
- high-DPI (fractional-scale) correctness
- theming, including COLOR-BLIND-SAFE layer palettes
- numeric PERFORMANCE BUDGETS that concretize Decision 012
- a forward-compatible 3D seam (the 2D canvas does not block a later 3D view)

This area does NOT own (governed by the sibling area specs in `specs/gui/`).
Cross-references below name the files that EXIST in this suite; the suite as a
whole still carries some legacy filenames that do not resolve (OQ9), so this
spec deliberately points only at real siblings:
- selection/hover/drag consumer-state model, snap/grid, per-tool interaction
  state machines, the commit boundary, and cross-probe SELECTION IDENTITY
  (select-same-net over the `DesignModel` identity triple)
  -> `GUI_INTERACTION_GRAMMAR.md`
- the cross-probe MODEL itself (N-way identity resolution + per-pane emphasis
  broadcast across schematic/PCB/3D/BOM/CAM/findings) -> `GUI_CROSS_PROBE.md`
  (this spec owns only the highlight/dim render TREATMENT cross-probe drives,
  §4.4)
- the dockable/floating panel system and panel taxonomy (Properties/Constraint/
  findings docks) -> `GUI_INFORMATION_ARCHITECTURE.md`
- agent ghost/diff REVIEW semantics (the accept/reject Review machine)
  -> `GUI_AI_SURFACES.md`
  (this spec owns only the GPU PRIMITIVES ghosts render through, §4.6)
- online-DRC / live-feedback / live-CAM projection semantics + source<->CAM
  cross-highlight logic -> `GUI_LIVE_FEEDBACK_AND_RULES.md`
  (this spec owns the projection-overlay PRIMITIVES and zone-fill-honesty
  rendering, §4.5)
- the FIRST read-only supervision surface this canvas serves
  -> `GUI_SUPERVISION_REFLECTION.md` (Decision 013, GUI area 1)

This area is a renderer: it produces no design authority, runs no topology
algorithms (airwire MST, zone fill, connectivity are engine-owned), and
mutates nothing. Per master §4.1 it draws ONLY the resolved model via the
scene contract. The only "mutations" in scope are camera and display-mode
changes, which are CONSUMER STATE (master §4.2) — never operations, never
journaled.

---

## 2. The Bar (commercial benchmark)

Benchmarked against the commercial canvas, with KiCad's GAL as the explicit
FLOOR we exceed (master §2).

| Capability | Commercial reference (the thing we match or beat) |
|---|---|
| GPU canvas / compositing | Altium DirectX-accelerated 2D canvas; the smoothness floor every "good enough" call is measured against |
| Layer transparency + active-layer emphasis | Altium single-layer mode + "transparent layers"; Allegro display color/visibility (Color192) |
| Render-stack / draw-order discipline | Altium layer-stack draw priority; Allegro display priority of etch/planes/keepouts |
| Net highlight / dim | Altium net highlight + "dim other objects" mask level; Allegro highlight/dehighlight |
| Ratsnest / airwires | Altium connection lines; Allegro ratsnest with per-net show/hide; PADS unroute display |
| Color-blind-safe palettes | Altium ships color-blind-aware system color schemes; we make this a first-class palette family, not an afterthought |
| High-DPI / fractional scaling | modern pro-app expectation (Altium on 4K/200%); KiCad's GAL is the floor |
| Theming | Altium board-color schemes + system colors; Allegro color/visibility dialog presets |

Where we MATCH: 60 fps pan/zoom, accurate transparency/compositing, per-net
ratsnest control, high-DPI crispness, color-blind palettes, dark theme. These
are table stakes (master §3.1) — correctness and fluency attention, not
novelty.

Where we EXCEED (the wedge, master §3.2):
- The canvas renders entirely from one resolved `DesignModel` via a versioned,
  byte-stable scene contract. The same scene that drives the live GPU canvas
  drives offscreen golden capture, so what a reviewer sees is provably the same
  geometry the regression harness asserts. Commercial canvases are not
  golden-pinned to their own committed model this way.
- The canvas is a render target for the AI-native ghost/diff differentiator
  (§4.6): agent proposals draw as in-canvas GPU primitives distinct from
  authored copper. No commercial tool renders an agent edit as a reviewable
  in-canvas ghost.
- Display mode (single-layer/layer-set), net highlight, theme, and camera are
  all pure consumer-state projections of one model; none touch authority, so
  they are perfectly reproducible and golden-testable.

A render surface is not "good enough" because it draws the board. It is good
enough when, on the `datum-test` fixture at 4K/200%, it would not embarrass us
next to Altium on the same board.

---

## 3. Architecture Constraints inherited (non-negotiable)

From the master (§4), applied to rendering:

1. **Render-from-model only.** The renderer consumes `BoardReviewSceneV1`
   (and its successors). It never reads source shards, never infers geometry
   that alters design meaning (`M7_RENDER_SEMANTIC_CONTRACT.md` invariance
   rule), and never runs design algorithms. Airwire MST, zone fill, and
   connectivity arrive pre-computed in the scene from engine truth.
2. **Camera/display-mode/highlight are consumer state.** They mutate a
   `CanvasViewState` (§5) held by the consumer. They are never `Operation`s,
   never journaled, never advance `model_revision`. A theme switch or a
   single-layer toggle must be invisible to the journal (master §4.2; 000B
   workspace-vs-source separation).
3. **Supervision-reflection first.** The first canvas deliverable is the
   READ-ONLY reflection of committed state (Decision 013): accurately render
   what the engine committed, so a human can audit before any interactive tool
   exists. Every primitive below ships in the read-only canvas before any tool
   in `GUI_INTERACTION_GRAMMAR.md` is built.
4. **Visual goldens are acceptance.** Every render surface in §4 ships a
   golden + a render/interaction test in the `gui-render` harness
   (`visual_manifest.rs` fixture format v1: `viewport`, `golden` with
   `diff_policy` ∈ {exact, tolerance, ssim}, per-pixel + total-pct + ssim
   thresholds + optional mask). Prose does not accept a surface (§8).

Material-first render discipline (`M7_RENDER_LAYER_DISCIPLINE_MEMO.md`,
M7-REN-006) is inherited as a hard rule: **layer/material semantics are
primary, primitive-class treatment is secondary.** Copper families build
`LayerAppearance` through `from_copper_material`; the bounded-exception set
(through-hole pad pass, via family, board outline/Edge overlay, selection/hover
emphasis, unknown-layer fallback) is closed — growing it requires a memo note.

---

## 4. Render surfaces and scene-contract schema extensions

Each subsection is a render surface with its scene-contract dependency, its
match-vs-exceed call, and any schema extension. Extensions follow the existing
`BoardReviewSceneV1` shape (`crates/gui-protocol/src/lib.rs:35`): named fields,
`nm` units for geometry, the §2.5 identity triple (`object_id`, `object_kind`,
`source_object_uuid`) on every renderable, explicit `render_order`, versioned
and byte-stable on unchanged persisted state. **No new field introduces design
authority into the GUI.** All extensions are additive (`#[serde(default)]`) and
bump the consuming reader, not the on-disk authority.

### 4.1 GPU canvas + camera (60 fps pan/zoom on large boards)

The canvas is a wgpu-composited 2D view. The scene's geometry is in board `nm`;
the renderer applies a single board->NDC transform from `CanvasViewState`
(§5). Pan/zoom mutate ONLY the transform (consumer state); they do NOT
re-tessellate static geometry.

Requirements:
- Static authored geometry (tracks, pads, vias, zones, silk, outline) is
  tessellated once per scene revision into retained GPU buffers keyed by
  `(layer_id, primitive_family)`. Pan/zoom rebind the transform uniform only.
- Line/track width is screen-aware: a track renders at its true `width_nm` at
  any zoom, with a sub-pixel floor so a hairline track stays visible when
  zoomed far out (matches Altium's minimum-object-size visibility).
- Level-of-detail: below a per-family screen-size threshold, glyph/text meshes
  and tiny silk fall back to a bounded box or are culled (Altium's
  text-rendering-by-zoom behavior). LOD is a pure function of zoom; it never
  changes which objects exist.
- Frustum culling: only primitives intersecting the viewport rect are issued.

Schema: NO new authority field. The camera lives entirely in consumer state
(§5). The scene already carries `bounds: SceneBounds` for fit-to-board. Add an
OPTIONAL renderer hint, advisory only:

```text
// BoardReviewSceneV1, additive, advisory hint (never authority)
detail_hints: SceneDetailHints   // #[serde(default)]

struct SceneDetailHints {
    // Engine's count of primitives by family, so the consumer can pre-size
    // GPU buffers and choose an LOD strategy before tessellation. Advisory:
    // wrong values degrade perf, never correctness.
    track_count: u32,
    pad_count: u32,
    via_count: u32,
    zone_count: u32,
    text_count: u32,
}
```

Match/exceed: MATCH Altium's DirectX pan/zoom smoothness and minimum-object
visibility. EXCEED nothing here — this is table stakes; the win is that the
same retained scene drives the golden harness.

### 4.2 Layer compositing, transparency, and the render stack

The renderer composites layers in the declared stack order, with per-layer
opacity for transparent-stack viewing.

Declared authored-board stage order. This is NOT a fresh proposal: it is the
existing `RenderStage` enum in `crates/gui-render/src/lib.rs`, whose declaration
order is the only priority encoding (locked by
`M7_RENDER_LAYER_DISCIPLINE_MEMO.md` 2026-04-16 + M7-REN-006). The spec pins
that order; it does not invent one:

```text
RenderStage (declared order = draw order, bottom to top):
  BottomCopper -> InnerCopper -> TopCopper
  -> BottomMask -> TopMask
  -> BottomPaste -> TopPaste
  -> BottomSilk  -> TopSilk
  -> Mechanical          // mechanical, fab, and courtyard ROLES map here
  -> Edge                // board-boundary overlay, drawn last
  -> Other               // catch-all for unrecognized stages (stays visible)
```

Fab and courtyard are layer ROLES (§4.2 `layer_role`), not separate stages —
they composite within `Mechanical`. `Other` is the bounded catch-all so an
unrecognized layer still draws (paired with the `"unknown"` divergent fallback
color) rather than vanishing. The post-copper walk is the existing
`POST_COPPER_STAGES` constant; adding a stage requires a memo note, not a spec
edit.

Within a stage: layer type first, back side before front side, scene
`render_order` (`SceneLayer.render_order`) only as a stable tie-breaker.

Requirements:
- Per-layer visibility comes from `SceneLayer.visible_by_default` as the seed,
  then from `CanvasViewState.layer_visibility` overrides (consumer state).
  Turning a layer off REMOVES geometry owned by that layer
  (`M7_RENDER_SEMANTIC_CONTRACT.md` acceptance Q4).
- Per-layer opacity (consumer state, §5) drives alpha compositing. Default
  copper opacity < 1.0 so inner-layer copper reads through top copper, matching
  Altium transparent-layer viewing. Opacity is a render parameter only.
- Color comes from the material-first `LayerAppearance` resolved from the scene
  layer table; primitive class refines stroke/fill, never invents a color
  system.

Schema extension on `SceneLayer` (additive, advisory color/role metadata so the
renderer/theme can resolve a default material without baking color at import):

```text
struct SceneLayer {
    layer_id: String,
    name: String,
    kind: String,            // existing free-form kind
    render_order: u32,
    visible_by_default: bool,
    // ADDITIVE:
    layer_role: String,      // #[serde(default = "default_unknown")] render role:
                             //   "copper.top" | "copper.inner" | "copper.bottom"
                             //   | "mask.top" | "mask.bottom" | "paste.top"
                             //   | "paste.bottom" | "silk.top" | "silk.bottom"
                             //   | "mechanical" | "fab" | "courtyard"  (-> Mechanical stage)
                             //   | "edge"                              (-> Edge stage)
                             //   | "unknown"                           (-> Other stage)
    stack_index: i32,        // #[serde(default)] physical stack position for
                             //   inner-copper ordering; ties broken by render_order
}
```

`layer_role` lets the THEME map a semantic role to a palette entry instead of
matching layer names by string. `kind` remains for back-compat; `layer_role`
is the resolution key when present. `"unknown"` keeps the deliberately
divergent unknown-layer fallback color (a bounded exception) so unresolved
layer identity stays visible.

Match/exceed: MATCH Altium single-layer + transparent-layer compositing and
Allegro display-priority. The match obligation is that the stack order and
transparency read identically to a board reviewer.

### 4.3 Single-layer and layer-set display modes

Two display modes, both pure projections of consumer state:

- **Layer-set mode** (default): all visible layers composited per §4.2.
- **Single-layer mode**: the active layer is drawn at full opacity and on top;
  all other layers are dimmed to a low alpha (the "monochrome / gray others"
  effect), so the active layer is unambiguous. Matches Altium's Single Layer
  Mode (Shift+S cycle) and Allegro's active-subclass emphasis.

Requirements:
- Dimming reuses the existing `dim_policy.rs` family (`dim_authored_color`,
  `dim_process_color`, `dim_structural_color`, `dim_context_color`) — these are
  the inactive-layer treatment. No new dim system.
- The active layer is a single `layer_id` in `CanvasViewState`. Switching it is
  consumer state; it never mutates the model.
- Mode + active layer are golden-testable: each mode has its own golden on the
  same scene.

Schema: NO new authority field. Active layer + mode live in `CanvasViewState`.

Match/exceed: MATCH Altium Single Layer Mode. EXCEED via determinism — the
mode is a pure function of (scene, active_layer), so the golden is exact, not
"looks right".

### 4.4 Net highlight / dim and ratsnest (airwires)

Net highlight, net dim, and the unrouted lane.

Net highlight/dim:
- Highlighting a net (or net class) draws all primitives whose net matches at
  full emphasis and dims everything else via `dim_policy.rs`. Matches Altium
  net highlight + "dim" mask level and Allegro highlight/dehighlight.
- The highlighted net set is CONSUMER STATE (`CanvasViewState.highlighted_nets`),
  driven by selection or by cross-probe. The cross-probe SELECTION IDENTITY and
  same-net resolution live in `GUI_INTERACTION_GRAMMAR.md` (the interaction
  machine that resolves select-same-net over the `DesignModel` identity triple);
  this spec owns only the VISUAL highlight/dim treatment that identity drives.
- Net identity on a primitive ALREADY comes from the scene: as of
  `crates/gui-protocol/src/lib.rs`, `PadPrimitive`, `TrackPrimitive`,
  `ViaPrimitive`, and `ZonePrimitive` each carry `net_uuid: Option<String>`
  (`None` = no net, e.g. a mechanical pad), and `UnroutedPrimitive` carries
  `net_uuid: String`. The renderer GROUPS highlight/dim over `net_uuid`. NO new
  per-primitive field is needed or permitted; adding a parallel `net_id` would
  duplicate authority. `highlighted_nets` (§5) is therefore a `Set<net_uuid>`.

The scene already carries `net_display: Vec<NetDisplayEntry>`, and that entry
already carries per-net ratsnest color (`airwire_color_rgb: [f32; 3]`). The
ONLY additive need is per-net ratsnest visibility and a power-net flag for the
quick-hide workflow:

```text
struct NetDisplayEntry {
    net_uuid: String,                 // existing (the highlight/ratsnest key)
    net_name: String,                 // existing
    airwire_color_rgb: [f32; 3],      // existing per-net ratsnest color
    // ADDITIVE:
    ratsnest_visible: bool,           // #[serde(default = "default_true")]
    is_power: bool,                   // #[serde(default)] enables quick power-hide
}
```

`airwire_color_rgb` is the per-net color (already present); the additive
`ratsnest_visible` is the data-driven per-net show/hide seed, overridable by
`CanvasViewState.ratsnest_hidden_nets` (consumer state). No `[u8;3]` color
override is introduced — the scene's `[f32; 3]` is authoritative and a consumer
recolor, if ever wanted, lives in `CanvasViewState`, never the scene.

Ratsnest (airwires) — render contract from `M7_AIRWIRE_RENDERING_GUIDANCE.md`
(engine owns the MST-over-Delaunay computation; renderer consumes
`unrouted_primitives`):
- Outline/wireframe only. Thin solid linework by default, NOT dashed. No fill,
  no glow, no shadow, no fade gradient (semantic-contract unrouted lane;
  airwire hard requirement 7).
- Per-net visibility and per-net color, data-driven from `NetDisplayEntry`
  above. A dedicated quick-hide for power nets (`is_power`). This is the
  most-requested ratsnest workflow across every tool surveyed (airwire hard
  requirement 3-4).
- Endpoint anchors: only a subtle anchor marker; it must NEVER read as a via,
  drill, or copper feature (semantic-contract unrouted grammar).
- The unrouted lane must remain distinguishable under dim/single-layer/highlight
  states (it has its own lane, not a copper layer).
- Live recompute is the ENGINE's job with a drag throttle; the renderer just
  re-draws the new `unrouted_primitives` list it is handed. No `B`-to-recompute
  in the renderer.

Match/exceed: MATCH Altium/Allegro per-net ratsnest control and net highlight.
EXCEED via the unified model — net highlight crosses schematic and PCB over one
`net_uuid` identity resolved from a single `DesignModel` (the highlight
treatment here is the PCB-canvas render half of the cross-probe differentiator;
the cross-probe MODEL that coordinates it across panes is owned by
`GUI_CROSS_PROBE.md`, and the selection identity by
`GUI_INTERACTION_GRAMMAR.md`). A name-join tool
cannot guarantee the two views highlight the same electrical net; resolving over
`net_uuid` does, by construction.

### 4.5 Zone-fill honesty in the canvas

Per `QG-ZONEFILL-HONESTY` (Decision 012) and 000B, the canvas must never let an
authored zone BOUNDARY read as fabricated copper.

Requirements:
- A `ZonePrimitive` authored boundary renders as an OUTLINE / boundary, not as
  a filled copper region, unless it carries a real filled state.
- Only a `ZoneFill{Filled}` derived fill contributes the copper FILL render.
  `Unfilled | Stale | Unsupported` render as a distinct boundary-only / hatched
  treatment and must read as "not copper yet", never as solid pour.
- Filled-zone copper is a derived shade of the owning layer's base material
  (`dim_policy.rs::ZONE_FILL_FIELD_MIX`, M7-REN-004) so pad/track boundaries
  against the pour and teardrop flanks stay readable — they must not merge into
  one undifferentiated mass.

Schema extension on `ZonePrimitive`:

```text
struct ZonePrimitive {
    // existing (crates/gui-protocol/src/lib.rs):
    object_id: String,
    object_kind: String,
    source_object_uuid: String,
    zone_uuid: String,
    net_uuid: Option<String>,     // already present (highlight key, §4.4)
    layer_id: String,
    polygon: Vec<PointNm>,        // already present: the AUTHORED boundary
    // ADDITIVE:
    fill_state: String,           // #[serde(default = "default_unfilled")]
                                  //   "filled" | "unfilled" | "stale" | "unsupported"
    fill_geometry: Vec<Vec<PointNm>>,  // #[serde(default)] engine-derived filled
                                  //   polygon(s) with islands; empty unless
                                  //   fill_state == "filled". Renderer NEVER
                                  //   computes fill.
}
```

The authored boundary is the existing `polygon`; the additive `fill_geometry`
is the engine-derived pour (a `Vec<Vec<PointNm>>` so islands/thermals are
distinct polygons, matching how `OutlinePolyline`/`holes` already model
multi-ring geometry). `net_uuid` already exists — no net field is added.

The renderer draws `fill_geometry` as copper ONLY when `fill_state == "filled"`;
the authored `polygon` boundary always draws as outline. Live-CAM cross-highlight
and the production projection side are owned by `GUI_LIVE_FEEDBACK_AND_RULES.md`;
this spec owns the canvas honesty rendering. Filled-zone copper uses
`dim_policy.rs::ZONE_FILL_FIELD_MIX` via the `zone_fill`/`zone_outline` slots of
`LayerAppearance::from_copper_material` (M7-REN-004).

Staleness contract: the scene is byte-stable per persisted state and carries
`source_revision` (`BoardReviewSceneV1.source_revision`). The engine sets
`fill_state = "stale"` when the cached pour predates the current model; the
renderer then draws the stale treatment (boundary + hatch, never solid copper)
rather than a misleading last-good pour. The renderer does not compute staleness
— it renders the engine's verdict.

Match/exceed: MATCH Allegro dynamic-shape filled/unfilled display. EXCEED via
the gate — the renderer cannot draw a boundary as copper by construction
(`polygon` is outline-only; copper requires `fill_state == "filled"` +
`fill_geometry`), and a stale fill renders stale from engine truth, which is
stronger than a tool that silently shows the last computed pour.

### 4.6 Proposal / ghost / diff primitives (the GPU side of differentiator 1)

This spec owns the GPU PRIMITIVES the AI-native canvas
(`GUI_AI_SURFACES.md`) renders through; that spec owns the review
state machine and accept/reject semantics.

The scene already carries `proposal_overlay_primitives:
Vec<ProposalOverlayPrimitive>` and `review_primitives: Vec<ReviewPrimitive>`.
Proposed geometry must read as CANDIDATE copper — copper-like, intentional, but
distinct from authored copper and from unrouted linework
(`M7_RENDER_SEMANTIC_CONTRACT.md` proposed lane). The renderer treats proposals
as a derived refinement of the owning layer's material (the `proposal` slot in
`LayerAppearance::from_copper_material`), not a separate color system.

Schema extension: a typed ghost/diff role on the proposal primitive so the
renderer can draw add/remove/modify diffs distinctly (the visual grammar of
"this op proposes to ADD this track / REMOVE this track / MOVE this pad"). The
primitive ALREADY carries a per-proposal grouping key (`proposal_action_id`) and
a `render_role`; the only additive need is the diff role:

```text
struct ProposalOverlayPrimitive {
    // existing (crates/gui-protocol/src/lib.rs):
    overlay_id: String,
    primitive_kind: String,
    proposal_action_id: String,   // groups primitives of one proposed action
    layer_id: Option<String>,
    render_role: String,
    width_nm: Option<i64>,
    path: Vec<PointNm>,
    // ADDITIVE:
    diff_role: String,            // #[serde(default = "default_add")]
                                  //   "add" | "remove" | "modify_before"
                                  //   | "modify_after"
}
```

Grouping reuses the existing `proposal_action_id` — no new `proposal_id` is
introduced. `modify_before`/`modify_after` primitives of one edit share a
`proposal_action_id`, and `CanvasViewState.visible_proposal_ids` (§5) toggles
visibility keyed on `proposal_action_id`.

Render grammar (the visual contract the golden pins):
- `add`: candidate-copper material with a proposal accent (distinct from
  authored copper).
- `remove`: the authored geometry it would delete, drawn with a removal accent
  (e.g. desaturated + struck), so the supervisor SEES what disappears.
- `modify_before` / `modify_after`: before drawn as removal accent, after drawn
  as add accent, paired by shared `proposal_action_id`.

Match/exceed: EXCEED — no commercial tool has this. This is the marquee surface
(master §3.2 #1). The render-side obligation is that an agent's proposed edit is
visually unmistakable from committed board truth and that accept/reject (owned
by the AI spec) has a clean primitive set to toggle.

### 4.7 High-DPI

- The renderer is DPI-aware: it queries the surface scale factor and renders at
  physical pixel resolution; line widths, glyph meshes, and the sub-pixel
  hairline floor are computed in physical pixels.
- Fractional scale factors (1.25, 1.5, 2.0) produce crisp output with no
  half-pixel blur on the dominant geometry (Altium-on-4K expectation; KiCad GAL
  is the floor).
- Golden capture records the scale factor; a golden is valid only at its
  recorded `(width_px, height_px)` (the harness already pins viewport size in
  `visual_manifest.rs`). A high-DPI golden is a separate fixture at 2x viewport.

Schema: NO scene change. DPI is a render/surface property, not model state.

### 4.8 Theming + color-blind-safe palettes

Theme is a named palette that maps `layer_role` (§4.2) and semantic lanes
(authored / unrouted / proposed / diagnostic) to colors. Theme is pure consumer
state; it never touches the model and is golden-testable per theme.

Required palette families at first GUI release subject to OQ1:
- **Dark** (default; commercial dark-canvas norm).
- **Light** (Altium ships polished light schemes; gated by master OQ1).
- **Color-blind-safe** (deuteranopia/protanopia/tritanopia-aware): a palette
  family where the copper-stack, mask, paste, silk, ratsnest, and proposal
  lanes remain DISTINGUISHABLE under simulated CVD. Altium ships color-blind
  system schemes; we make this a first-class palette family, not an
  accessibility afterthought.

Requirements:
- A palette assigns: per-`layer_role` material base, board field/background,
  unrouted lane color, proposal add/remove accents, diagnostic emphasis,
  selection/hover emphasis. It does NOT assign per-object colors (material-first
  discipline).
- Per-net ratsnest color (`NetDisplayEntry.airwire_color_rgb`, already in the
  scene) layers ON TOP of the palette for the ratsnest lane only; the palette
  supplies the DEFAULT lane color when an entry leaves it at the palette value.
  No per-object palette override exists (material-first discipline).
- The color-blind palette is validated by a CVD-simulation check in the harness:
  render the layer-stack swatch fixture, run a deuteranope simulation, assert a
  minimum perceptual distance between adjacent stack lanes (acceptance §8).

Schema: NO scene change. Palettes are renderer/consumer config keyed by
`layer_role` and lane.

Match/exceed: MATCH Altium color schemes incl. color-blind. EXCEED via the
golden — the color-blind palette ships with a CVD-simulation golden that PROVES
lanes stay distinguishable, which is a stronger guarantee than shipping a
scheme and trusting it.

### 4.9 3D seam (forward-compatible, not built here)

3D is M8 (CLAUDE.md / master §9 non-goals). This area only guarantees the 2D
canvas does not architecturally block it:
- The scene contract carries per-layer `stack_index` (§4.2) and `nm` geometry —
  enough for a later extruded-stack 3D view to consume the SAME scene.
- `CanvasViewState` (§5) holds a `view_kind` enum with `Plan2D` only today;
  `Layer3D` is reserved. A 3D camera is a different `CanvasViewState` variant,
  not a new authority path.

NON-GOAL here: any 3D rendering, STEP/IDF import, board thickness solver. Listed
only so the 2D contract reserves the seam.

---

## 5. Consumer state: `CanvasViewState`

All non-authority canvas state is one consumer-owned struct (master §4.2: never
an operation, never journaled, never advances `model_revision`; 000B
workspace-vs-source separation). It is the input to every render pass alongside
the scene.

```text
struct CanvasViewState {
    view_kind: ViewKind,            // Plan2D today; Layer3D reserved
    // camera (pan/zoom). These are the SAME projection the golden harness
    // pins via VisualViewport (center_mm + zoom_mm_per_px) in
    // crates/gui-render/src/visual_manifest.rs, so a live view and its golden
    // are the identical board->NDC transform — the determinism wedge (§2).
    center_mm: [f64; 2],            // board-space center (matches VisualViewport)
    zoom_mm_per_px: f64,            // board-mm per logical pixel (matches VisualViewport)
    dpi_scale: f64,                 // surface scale factor (physical-px multiplier)
    // display mode
    display_mode: DisplayMode,      // LayerSet | SingleLayer
    active_layer_id: Option<String>,
    layer_visibility: Map<String, bool>,   // override of visible_by_default
    layer_opacity: Map<String, f64>,       // per-layer alpha for transparency
    // net / highlight (keys are scene net_uuids, §4.4)
    highlighted_nets: Set<String>,  // net_uuids drawn emphasized; others dimmed
    ratsnest_hidden_nets: Set<String>,     // net_uuids whose ratsnest is hidden
    // theme
    theme_id: String,               // "dark" | "light" | "cvd_safe" | custom
    // proposals: which proposed actions render as ghosts; keyed by the scene's
    // ProposalOverlayPrimitive.proposal_action_id (§4.6), not a new id.
    visible_proposal_ids: Set<String>,
}

enum ViewKind { Plan2D, Layer3D /* reserved, not implemented */ }
enum DisplayMode { LayerSet, SingleLayer }
```

Hard rule: a function that mutates `CanvasViewState` MUST NOT be able to reach
`commit()` or write a shard. This is the consumer-state boundary, enforced by
type, not convention: `CanvasViewState` lives in the `gui-app`/`gui-render`
consumer layer with no `&mut DesignModel`, no journal handle, and no `commit()`
in scope. An interaction test (§8) asserts that camera/mode/theme/highlight/
proposal-toggle changes leave `model_revision` unchanged and the journal empty.

---

## 6. Performance budgets (concretizing Decision 012 `QG-PERFORMANCE-LATENCY`)

Decision 012 left numeric budgets owner-defined until fixtures exist
(`PRODUCT_MECHANICS_012` §Performance, OQ6). This area proposes the FIRST
fixture-backed canvas budgets, on the `datum-test` regression fixture (the
canonical M7 fixture), pending owner ratification (OQ5 below). Budgets are
measured by repeatable harness benchmarks, not subjective impression
(`QG-PERFORMANCE-LATENCY`).

Fixture tiers (the "large board" the master asks for is the LARGE tier):

| Tier | Definition (target representative board) |
|---|---|
| Small | `datum-test` baseline (few hundred primitives) |
| Medium | ~5k copper primitives, 4 layers |
| Large | ~50k copper primitives, 8+ layers (the 60-fps obligation tier) |

Proposed budgets (PENDING owner ratification, OQ5):

| Metric | Small | Medium | Large | Measured how |
|---|---|---|---|---|
| Pan/zoom frame time | <= 8 ms | <= 12 ms | <= 16.6 ms (60 fps) | rebind-transform-only frame loop, p99 over a scripted pan/zoom sweep |
| Initial scene tessellation | <= 30 ms | <= 150 ms | <= 600 ms | one-time on scene load (off the interaction path) |
| Display-mode toggle (single<->set) | <= 16 ms | <= 16 ms | <= 33 ms | recomposite, no re-tessellation |
| Net highlight apply | <= 16 ms | <= 16 ms | <= 33 ms | dim-pass re-color, no re-tessellation |
| Theme switch | <= 16 ms | <= 16 ms | <= 33 ms | palette re-resolve + recomposite |
| Offscreen golden capture | (not interactive) | — | — | bounded so CI stays fast |

Measurement harness: the visual-regression harness (`visual_runner.rs`) proves
CORRECTNESS, not frame time; perf is a SEPARATE bench target in
`crates/gui-render` (a `criterion`-style or fixed-iteration frame-loop bench,
e.g. `benches/canvas_panzoom.rs`) that loads the tier fixture, builds retained
buffers once, then drives a scripted pan/zoom path rebinding the transform
uniform per frame and records p99 frame time. The bench fixture and the golden
fixture share the same `BoardReviewSceneV1`, so perf and picture are measured on
identical geometry. A frame-time number is only meaningful against a named
reference GPU; the budgets below assume a stated baseline adapter (owner to fix
the reference machine at ratification, OQ5) and the bench records the adapter
string so a regression is attributable to code, not hardware drift.

Rules:
- The 60-fps Large-tier pan/zoom obligation is the headline number the master's
  "GPU 60fps pan/zoom on large boards" requires. It is met by rebind-transform-
  only frames over retained buffers (§4.1) — re-tessellation per frame fails the
  budget by construction.
- Preview latency (camera) is measured SEPARATELY from any future commit latency
  (Decision 012 separation rule). This area has no commit path, so it owns only
  preview-class budgets.
- Regressions are caught by the benchmark fixture in CI, not by eyeballing.
- These numbers are a PROPOSAL; the owner ratifies the thresholds (OQ5). Until
  ratified, the canvas perf work is marked experimental per Decision 012, but
  the FIXTURES and measurements exist so ratification is a number change, not
  new engineering.

---

## 7. Proof slices

In the voice of Decision 012's first-proof-slice. Each names the fixture
(default `datum-test`) and the gates.

### PS-CANVAS-1: Supervision-reflection render (FIRST, read-only)
The read-only canvas renders the committed `datum-test` board faithfully:
authored copper stack, mask/paste/silk, vias, zones (boundary + filled fill per
§4.5), outline, component bodies, board/footprint text, and the unrouted lane.
Layer-set mode, default dark theme. Demonstrates master §4.3 supervision-first.
- Gates: `QG-ZONEFILL-HONESTY` (filled zone draws copper; unfilled/stale draws
  boundary-only + reads not-copper). Golden `canvas_reflection_datum_test`.

### PS-CANVAS-2: Layer compositing + single-layer mode
Same fixture, two goldens: layer-set (transparent stack, inner copper reads
through top) and single-layer (active = top copper, others dimmed). Toggling
the mode and active layer mutates only `CanvasViewState`.
- Gates: interaction test asserts mode/active-layer changes leave
  `model_revision` unchanged and the journal empty (master §4.2).
  Goldens `canvas_layerset_datum_test`, `canvas_singlelayer_top_datum_test`.

### PS-CANVAS-3: Net highlight + ratsnest
Highlight one signal net and one power net; show/hide the power-net ratsnest via
the quick-hide. Highlighted net at full emphasis, rest dimmed; ratsnest is thin
solid wireframe with subtle anchors, per-net color.
- Gates: ratsnest reads as non-copper under highlight + dim (semantic contract
  acceptance Q1-Q3); highlight is consumer state (journal empty). Goldens
  `canvas_net_highlight_datum_test`, `canvas_ratsnest_power_hidden_datum_test`.

### PS-CANVAS-4: AI ghost/diff primitives
Render a fixture proposal carrying `add` + `remove` + `modify` diff roles. The
add reads as candidate copper, the remove shows the struck authored geometry,
modify pairs before/after. Toggling `visible_proposal_ids` shows/hides ghosts.
- Gates: proposed geometry visually distinct from authored copper AND from
  unrouted (semantic-contract proposed lane); ghost toggle is consumer state.
  Golden `canvas_proposal_ghost_diff_datum_test`. (Review/accept semantics:
  `GUI_AI_SURFACES.md`.)

### PS-CANVAS-5: High-DPI + color-blind palette
Render `datum-test` at 2x viewport (high-DPI golden) and under the `cvd_safe`
palette. CVD golden passes the deuteranope-simulation distinctness check.
- Gates: high-DPI crispness (no half-pixel blur on dominant geometry); CVD lane
  distinctness (§8 CVD check). Goldens `canvas_hidpi_2x_datum_test`,
  `canvas_cvd_safe_datum_test`.

### PS-CANVAS-6: Large-board 60 fps budget
The Large-tier fixture renders a scripted pan/zoom sweep within the §6 budget.
- Gates: `QG-PERFORMANCE-LATENCY` p99 pan/zoom frame <= 16.6 ms on Large tier
  (PENDING owner ratification, OQ5). Benchmark `canvas_panzoom_large.bench`.

---

## 8. Visual-golden acceptance

Each surface ships a golden + render/interaction test in the `gui-render`
harness. The harness `FixtureManifest` (`visual_manifest.rs`, fixture format
version 1) pins per fixture, exactly these fields:
`name`, `lane`, `suite`, `fixture_format_version`, `project_path`,
`project_kind`, `viewport {width_px, height_px, center_mm, zoom_mm_per_px}`,
`golden {filename, diff_policy ∈ {exact, tolerance, ssim},
diff_tolerance_per_pixel, diff_tolerance_total_px_pct, ssim_threshold,
mask_filename}`, and `blank_check {expect_non_blank_pct}`. Canvas goldens reuse
the SAME `viewport.center_mm` / `viewport.zoom_mm_per_px` projection as the live
`CanvasViewState` (§5), so the asserted image is provably the geometry the live
view shows. `visual_diff.rs` implements `exact` (zero differing pixels) and
`tolerance` (per-pixel channel delta + total-pct ceiling); `blank_check` rejects
an all-blank or all-solid frame so a stub cannot pass as a render.

Acceptance matrix:

| Surface | Golden | Policy | Interaction / extra assertion |
|---|---|---|---|
| Reflection (PS-1) | `canvas_reflection_datum_test` | tolerance (tight) | zone fill_state honesty: filled draws copper, unfilled/stale draws boundary-only |
| Layer-set (PS-2) | `canvas_layerset_datum_test` | tolerance | — |
| Single-layer (PS-2) | `canvas_singlelayer_top_datum_test` | tolerance | mode/active-layer toggle leaves journal empty + `model_revision` unchanged |
| Net highlight (PS-3) | `canvas_net_highlight_datum_test` | tolerance | highlight set is consumer state |
| Ratsnest power-hide (PS-3) | `canvas_ratsnest_power_hidden_datum_test` | tolerance | hidden power net absent; signal ratsnest present, wireframe-only |
| Ghost/diff (PS-4) | `canvas_proposal_ghost_diff_datum_test` | tolerance | proposal toggle is consumer state; add/remove/modify visually distinct |
| High-DPI (PS-5) | `canvas_hidpi_2x_datum_test` | tolerance | captured at 2x viewport |
| CVD palette (PS-5) | `canvas_cvd_safe_datum_test` | tolerance | CVD-simulation distinctness check (below) |
| Large-board perf (PS-6) | (benchmark, not image) | — | `QG-PERFORMANCE-LATENCY` frame-time p99 |

Tolerance rationale: GPU rasterization differs sub-pixel across drivers, so the
canvas goldens use `tolerance` (small per-pixel channel delta, small total-pct
ceiling) rather than `exact`. `exact` is reserved for deterministic non-canvas
surfaces. Each golden's exact tolerance is set when blessed and locked
thereafter; loosening a tolerance to pass is a review-blocking regression.

CVD-simulation distinctness check (the color-blind acceptance, §4.8): render the
layer-stack swatch fixture, apply a deuteranope CVD transform to the captured
image, and assert a minimum perceptual color distance (CIEDE2000) between every
pair of adjacent stack-lane swatches. A palette that collapses two lanes under
CVD fails the gate. This is an EXCEED guarantee: the palette is proven, not
asserted.

A surface that renders a stub, or whose golden does not render REAL committed
`datum-test` state, is not accepted (master §4.4 + OQ3 minimum-coverage).

---

## 9. Non-Goals

- 3D rendering, board-thickness solver, STEP/IDF/ODB++ import (M8; §4.9 reserves
  only the seam).
- Running ANY design algorithm in the renderer — airwire MST, zone fill,
  connectivity, DRC are engine-owned; the renderer draws their results.
- Interactive editing tools (select/move/route/draw) — those are
  `GUI_INTERACTION_GRAMMAR.md`; this area renders the canvas they act on.
- The agent review state machine and accept/reject — `GUI_AI_SURFACES.md`;
  this area owns only the ghost/diff PRIMITIVES.
- Live-CAM projection semantics and source<->CAM cross-highlight logic —
  `GUI_LIVE_FEEDBACK_AND_RULES.md`; this area owns the canvas zone-fill-honesty and
  projection-overlay primitives.
- Per-object hand-authored colors (material-first discipline forbids it).
- Real-time re-tessellation per frame (fails the perf budget by construction).
- Multi-monitor / floating-window canvas persistence (master §9; 012).
- Animation, glow, fade, or shadow on any lane (semantic contract forbids it).
- Light/custom theme polish parity at first release if owner accepts dark-first
  (OQ1).

---

## 10. Open Questions

1. **Color-blind palette as a launch requirement vs. fast-follow** (relates to
   master OQ1). Altium ships CVD schemes; do we gate the first GUI release on a
   blessed `cvd_safe` palette + its CVD golden, or land dark-first and
   fast-follow CVD?
2. **Default copper transparency value.** What default per-layer copper alpha
   reads best for inner-layer-through-top viewing without muddying a dense
   board? Owner taste call against the Altium transparent-stack reference.
3. **Single-layer dim treatment.** Gray-out (monochrome others) vs. dim-toward-
   field (reuse `dim_policy.rs`). The code already has `dim_policy`; confirm it
   is the single-layer treatment or whether single-layer warrants a stronger
   monochrome mode like Altium's.
4. **Ratsnest density management on large boards.** At the Large tier a full
   ratsnest can be visually overwhelming. Do we ship a "ratsnest within
   selection / within window" mode (Allegro-style) in this area, or defer to a
   later increment? Pure render filter, but a UX scope call.
5. **Ratification of the §6 numeric budgets + reference GPU** (Decision 012
   OQ6). The proposed Small/Medium/Large frame-time and tessellation budgets need
   owner sign-off, the Large-tier representative board needs to be named/provided
   (Decision 012 OQ1, OQ6 below), AND the budgets need a named REFERENCE adapter
   — a 16.6 ms frame is meaningless without the GPU it is measured on. Should the
   reference be a developer iGPU, a CI software adapter (e.g. lavapipe, which
   would force much looser numbers), or a fixed discrete baseline? Until all
   three are fixed, canvas perf is experimental per 012.
6. **Large-tier fixture provenance.** The `datum-test` fixture is Small. The
   Medium/Large perf fixtures are not in the repo; per project policy
   (ask-for-fixtures) the owner should provide a real ~50k-primitive board
   rather than us fabricating one. Which real board defines Large?
7. **LOD thresholds.** The exact screen-size thresholds for text/silk LOD
   fallback are tuning values; should they be theme/owner-configurable or fixed
   constants pinned by golden?
8. **`detail_hints` necessity.** The §4.1 `SceneDetailHints` is an advisory
   perf hint. If the consumer can size buffers from the primitive vectors
   directly without measurable cost, this extension may be dropped — confirm
   whether the engine should populate it at all.
9. **Suite-wide spec filename drift (RESOLVED 2026-06-22).** Earlier drafts of
   the GUI area specs referenced each other under invented `*_SPEC.md` names
   (`GUI_CANVAS_INTERACTION_SPEC.md`, `GUI_PANELS_INSPECTOR_SPEC.md`,
   `GUI_AI_NATIVE_CANVAS_SPEC.md`, `GUI_CROSS_PROBE_SPEC.md`,
   `GUI_LIVE_PRODUCTION_SPEC.md`) that never existed as files. The canonical
   six AREA files are `GUI_SUPERVISION_REFLECTION.md`,
   `GUI_CANVAS_AND_RENDERING.md`, `GUI_INTERACTION_GRAMMAR.md`,
   `GUI_LIVE_FEEDBACK_AND_RULES.md`, `GUI_INFORMATION_ARCHITECTURE.md`, and
   `GUI_AI_SURFACES.md`. Note the invented `GUI_CROSS_PROBE_SPEC.md` is NOT
   the same as the later-authored dedicated cross-probe spec
   `GUI_CROSS_PROBE.md` (no `_SPEC` suffix), which is a real file owning the
   cross-probe coordination MODEL alongside the six area files; see master
   `specs/GUI_SPEC.md` §3.2/§5. The master §5 ratifies the area map (including
   where the live-production responsibility is folded and where cross-probe is
   promoted), and the conductor reconciliation pass repointed every
   cross-reference in the suite to these files. This OQ is closed and retained
   only as the record of the reconciliation.
