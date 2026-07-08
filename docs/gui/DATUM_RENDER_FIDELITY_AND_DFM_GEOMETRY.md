# Datum — Render Fidelity, DFM Geometry Solver, and the Fab Fingerprint

> **Status**: Governed design thesis + future-engine capture under decision 019,
> feeding the (forthcoming) Datum Rendering Book. The two **Laws** below are
> ratified invariants the product is held to; the **DFM Geometry Solver** is a
> named future engine (execution requires authorization) placed on the roadmap as
> an Active-Frontier step-5 dependency. The **Fingerprint** section is the visual
> identity thesis the Rendering Book will make concrete once the symbol/footprint
> prototype settles the look.
> **Authority**: subordinate to decision 019 and the product-mechanics doctrine;
> extends decision 015 (Design Book tokens) from UI chrome into manufacturable
> content geometry.
> **Why this exists**: a CAD tool's on-screen geometry must translate to CAM
> output with zero drift, and Datum's product differentiator is that a
> professionally beautiful, DFM-optimal board is the *default* engine output — not
> something the user hand-forces. This doc pins both so no development agent
> re-derives them at course-correction cost.

## Law 1 — Single geometry source of truth (CAM fidelity)

The derived-geometry engine emits **one** canonical manufacturable geometry per
object. The screen renderer and the CAM exporter (Gerber / Excellon drill / paste
/ assembly) **both consume that same geometry**. There is no separate "pretty
screen path." The rounded pad, the teardrop, the mitered corner a user *sees* is,
geometrically, exactly what the fab receives.

- **This is a gated invariant, not a nicety.** A render/CAM fidelity gate derives
  the on-screen manufacturable layer and the CAM surface from the same source and
  asserts geometric equivalence within a stated tolerance. "The visual represents
  the CAM output accurately" is enforced in code, not hoped for.
- **The one carve-out — the presentation fence.** Presentation-only overlays —
  selection glow, hover, cross-probe accent, grid, ratsnest, anti-aliasing — are
  **not geometry**, are **never** in CAM, and must never be confusable with a
  manufacturable mark. The fidelity gate checks the manufacturable layer only.
  Everything that can be fabricated is byte-for-byte what was on screen.

## Law 2 — Engine drives visual → beauty by default

The clean board is the **default output** of the engine, not a state the user
grinds toward. Rounded pads, teardropped junctions, DFM-optimal miters, proper
thermal reliefs, and consistent transitions are **default-on**, rule-driven per
net-class, and overridable — but never something a user must fight *for*.

Competitive diagnosis this inverts:
- **KiCad** can produce a gorgeous board, but beauty is **opt-in / high-effort**.
  Datum makes beauty the resting state.
- **Eagle** produces sterile, cold boards recognizable by a stroke silkscreen font
  and mechanical traces. Datum's engine-driven defaults + real typography
  eliminate that tell by construction.

## The DFM Geometry Solver (future engine)

One engine, three faces, one principle: **author the topology; derive the
manufacturable geometry from rules + electrical context.** The user draws a
centerline, a width, a junction; the engine computes the corner radius, the miter,
the teardrop. This maps onto the canonical-IR **authored-vs-derived** split:
authored = centerline + width + pad nominal + topology; derived = every rounded
transition, recomputed when inputs change.

**Three faces:**
1. **Pad corner rounding** — sharp corners are stress risers (reduced copper
   peel/lift under thermal + rework), etch imperfectly (undercut/dog-bone), and
   solder worse (bridging, tombstoning). A radius fixes all three; the etch rounds
   corners anyway, so the sharp pad is the *fiction*. Rounded-rectangle is the
   default pad.
2. **Trace corner treatment** — acute angles (<90°) are acid traps → prohibited /
   auto-chamfered (hard rule); right-angle bends cause current crowding and
   impedance discontinuity → radiused or mitered per net-class.
3. **Teardrops** — filleted trace-to-pad/via junctions reinforce against
   annular-ring breakout and drill wander and improve current flow; sized from
   pad/via/drill diameter and trace width.

**Representative parametric relationships the engine encodes** (defaults to be
tuned with the owner and bench data, not hardcoded here):
- Pad: `radius = rounding_ratio × min(w, h)`, capped at `max_radius`; default
  ratio ~25%, raised for high peel-strength / high-reliability.
- Trace, acute bend: **small inner fillet on both inside edges**, sized only to
  compensate etch loss and open the acid trap (not a chamfer, not a full round).
  The fillet radius is **algorithmically derived**, not aesthetic — a small
  `r ≈ f(etch_undercut, included_angle)`, where `etch_undercut` scales with copper
  weight × etch factor and the relief grows as the included angle sharpens.
- Trace, right-angle high-speed: Douville–James optimum microstrip miter,
  `M% = 52 + 65·e^(−1.35·W/h)` (W/h ≥ 0.25) — width-over-substrate-height driven.
- Trace, high-current: radius sized to cap inner-corner current density.
- Teardrop: length/width from `f(pad_or_via_dia, drill, trace_width)`.

**Why it's Datum-shaped (not a bolt-on):** every other tool applies teardrops and
rounding as a post-process. In Datum this is a **rule-driven derived-geometry
solver** — same architecture as DRC (a rules table, per-net-class, inspectable,
overridable) — applied through the **one commit()/journal path** (undoable, with
provenance), and expressed as **a parameter of a small verb set, not a
tool-per-shape** (Lean ethos). It sits on existing threads:
- **Routing kernel** (60+ path-candidate strategies) — already landed.
- **`ImpedanceSpec`** controlled-impedance solver — stub landed, solver deferred;
  supplies the net-class electrical context (target impedance, W/h) for miters.
- **Rendering Book** — supplies defaults and style; renders the derived geometry
  true (Law 1).

## The fingerprint — a recognizable Datum board at the fab

The goal: a fab house recognizes a Datum board by its *hand*. Two axes:
1. **A real silkscreen typeface rendered as filled outline geometry**, not
   centerline stroke text — *designed for silk*, respecting the silkscreen minimum
   feature/line width (~0.15 mm / 6 mil, fab-dependent) so it prints clean rather
   than breaking up. This is the direct antidote to the Eagle stroke-font tell,
   and the same typeface carries the fab/assembly **documentation** (title blocks,
   fab notes, assembly drawings — a later, related workstream).
2. **DFM-optimal copper as a signature** — uniform teardrops, consistent
   miters/radii, proper thermal reliefs, a harmonized layer palette and consistent
   courtyard/assembly conventions. The copper reads as *designed*, the way a
   typographer's spacing is recognizable.

Both are structural consequences of Laws 1–2 plus a typeface earned within the
manufacturing envelope — not luck, and not user effort.

## Roadmap placement (un-orphaned)

- **Active Frontier step 5 (native authoring depth)** owns the DFM Geometry Solver
  as a named dependency; it depends on the routing kernel (done), the
  `ImpedanceSpec` solver (deferred), and the Rendering Book defaults.
- **This doc is the foundation the Datum Rendering Book builds on.** The Rendering
  Book (symbol style, footprint style, icon contact sheet + drawing rules) is the
  next spec, produced *downstream of the owner-approved prototype* so the doc
  captures taste rather than guessing ahead of it.

## Decisions settled on the prototype (design pass 3)

- **Schematic ground — LOCKED:** dark is the working default; vellum is a
  print/documentation toggle. See `DATUM_RENDERING_BOOK.md` §1.
- **Silkscreen typeface — LOCKED:** IBM Plex (`IBMPlexSansCondensed`, the shipped
  asset), filled outline, silk-min-feature-safe. See Rendering Book §5.
- **Acute-bend treatment — LOCKED:** small inner fillets for etch-loss
  compensation (superseding the earlier "chamfer" phrasing).

## Open decisions (to settle with the owner + prototype/bench)

- Default rounding ratio and max-radius cap (locked ~25%; still validate against
  measured peel strength).
- Miter vs radius selection policy per net-class, and the impedance-miter formula's
  operating envelope.
- Teardrop sizing ratios and when they auto-apply vs. require opt-in.
- **Symbol standard (IEC vs ANSI) — Fork B**, still open; carried in the Rendering
  Book.
