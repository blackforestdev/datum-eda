# Datum EDA GUI Area Specification: Unified-Model Cross-Probe (DIFFERENTIATOR 2)

Status: draft GUI area specification, 2026-06-22, benchmarked to commercial
EDA. Controlling for the cross-probe COORDINATION domain. Conforms to and
inherits from `specs/GUI_SPEC.md` (the master): its bar, thesis, the four
architecture constraints, and the five-part buildability standard apply here
unchanged and are not restated except where this area adds detail.

This is the dedicated specification for master differentiator 2
(`specs/GUI_SPEC.md` §3.2 #2). The earlier draft FOLDED cross-probe into the
area-2 and area-3 specs (master §5 "folded"); this spec PROMOTES it to its
own coordinating surface because it is a marquee/signature feature and
deserves one coherent home. The MECHANISM pieces still live in their owners
(selection identity in `GUI_INTERACTION_GRAMMAR.md`, emphasis rendering in
`GUI_CANVAS_AND_RENDERING.md`, multi-pane composition in
`GUI_INFORMATION_ARCHITECTURE.md`); this spec owns the cross-probe MODEL —
the N-way identity-resolution surface that coordinates those primitives into
the differentiator — and references them by their real filenames.

Driven by:
- `specs/GUI_SPEC.md` (master: bar/thesis/constraints/buildability;
  differentiator 2)
- `specs/gui/GUI_INTERACTION_GRAMMAR.md` (selection identity, `SelectionState`,
  `EmphasisMode`, the `SelectedRef` identity triple, select-same-net)
- `specs/gui/GUI_CANVAS_AND_RENDERING.md` (emphasis highlight/dim rendering,
  `CanvasViewState.highlighted_nets`, the PCB-canvas half of cross-probe)
- `specs/gui/GUI_INFORMATION_ARCHITECTURE.md` (`LayoutGraph`, panes, PiP,
  `PinnedContext`, `NavigationStack`, multi-pane composition)
- `specs/gui/GUI_AI_SURFACES.md` (proposal review `S2b_GhostSelected`
  cross-probes affected committed objects)
- `specs/gui/GUI_SUPERVISION_REFLECTION.md` (the read-only instrument panel
  cross-probe ships into first; §48 "single resolved model for cross-probe")
- `specs/gui/GUI_LIVE_FEEDBACK_AND_RULES.md` (finding -> source-object +
  CAM-projection cross-probe; online-DRC navigation)
- `docs/decisions/PRODUCT_MECHANICS_000B_VIEW_COMPOSITION_AND_LIVE_PRODUCTION.md`
  (`ViewTab`/`Pane`/`LayoutGraph`/`PinnedContext`/PiP; source<->CAM
  cross-highlight; cross-selection between source object and generated
  manufacturing geometry)
- `docs/decisions/PRODUCT_MECHANICS_004_AI_TOOLING_CONTRACT.md` (AI tooling
  reads/writes the SAME resolved `DesignModel`; agents cross-probe too)
- `docs/decisions/PRODUCT_MECHANICS_013_GUI_SUPERVISION_AND_PARITY.md`
  (supervision-reflection first; cross-probe ships read-only first)
- the resolved `DesignModel` identity substrate (READ-ONLY):
  `crates/engine/src/substrate/component_instance.rs` (`ComponentInstance`,
  `ComponentInstanceId`, `placed_symbol_refs`/`placed_package_refs`),
  `crates/engine/src/substrate/relationship.rs` +
  `crates/engine/src/substrate/mod.rs` (`RelationshipKind`),
  `crates/engine/src/board/board_types.rs` (`Net.uuid`)
- `crates/gui-protocol/src/lib.rs` (`BoardReviewSceneV1`, the §2.5 identity
  triple `object_id`/`object_kind`/`source_object_uuid` on every renderable)

This area is master area #2/#3's cross-cutting CONCERN, lifted into its own
file. Where `GUI_INTERACTION_GRAMMAR.md` owns the selection GESTURE and the
`SelectionState` it produces, and `GUI_CANVAS_AND_RENDERING.md` owns the
pixel-level highlight/dim, THIS spec owns the COORDINATION layer that turns a
single selection in one projection into emphasized related objects in every
OTHER projection — the cross-probe model itself.

---

## 1. Purpose

State the controlling model for Datum's signature cross-probe: selecting an
object in any projection of the design instantly emphasizes the SAME design
identity in every other open projection — schematic, PCB, 3D, BOM,
manufacturing (CAM), and check-findings — with no name/refdes/net-name string
join and no second database to keep in sync.

Cross-probe is the feature a professional uses dozens of times an hour: "where
is this part on the board?", "what net is this pad on, and where else does it
go?", "this DRC violation — show me the schematic pin it came from". Every
commercial tool has a version of it; every commercial version is a SYNC across
two authorities joined by string matching. Datum's is selection over ONE
resolved model by stable identity. This spec makes that difference executable.

Cross-probe is **engine-neutral consumer state** (master §4.2): nothing here
is journaled, nothing is an `Operation`, nothing mutates a shard. It is
selection and emphasis — the same consumer-state vocabulary the grammar and
canvas already define — COORDINATED across panes. It renders entirely from the
resolved `DesignModel` via the scene contract.

---

## 2. The Wedge (this IS the story)

### 2.1 What every commercial tool actually does

In Altium, Allegro/OrCAD, and Xpedition, the schematic and the PCB are two
SEPARATE databases. Cross-probe ("Cross Select Mode", "cross-probe/jump")
works by:
- the schematic editor and the PCB editor each hold their own object store;
- when you select `R12` in the schematic, the tool sends a message
  ("cross-probe") to the PCB editor saying, in effect, *find the object whose
  reference designator string is `R12`*;
- the PCB editor does a NAME LOOKUP (refdes for components, net-name string for
  nets) in ITS database and highlights whatever matches.

This is fragile by construction:
- **It can drift.** If the two databases disagree — a refdes renamed on one
  side, a net renamed, a forward/back-annotation not yet run, an ECO half
  applied — cross-probe silently highlights the wrong object or nothing. The
  join key is a string, and strings drift.
- **It is schematic<->PCB only.** The 3D view, the BOM, the Gerber/CAM
  projection, and the DRC report are not in the join. You cannot click a BOM
  line and light up the footprint AND the schematic symbol AND the CAM pad in
  one gesture, because there is no shared identity spanning them — only
  pairwise string bridges where anyone bothered to build one.
- **It is a message, not a fact.** Cross-probe is an inter-process or
  inter-editor notification. It is as correct as the last sync, no more.

KiCad is the floor: KiCad has schematic<->PCB cross-probe by refdes/net-name,
same string-join construction. We exceed it, we do not target it.

### 2.2 What Datum does instead

Datum has ONE resolved `DesignModel`. The schematic symbol, the PCB footprint,
and the BOM line for `R12` are not three rows in three databases joined by the
string `"R12"` — they are PROJECTIONS of one `ComponentInstance` with one
stable `ComponentInstanceId`, resolved by the engine from the same source
shards (`component_instance.rs`: a `ComponentInstance` holds both
`placed_symbol_refs` and `placed_package_refs`, joining schematic symbol and
board package under one identity). A net is one `Net` with one `Net.uuid`
spanning every track, via, pad, zone, and ratsnest on every layer and the
schematic wire that declares it. Every renderable in the scene already carries
the §2.5 identity triple (`object_id` / `object_kind` / `source_object_uuid`).

Therefore cross-probe in Datum is not a sync — it is a QUERY over relationships
that already exist in the resolved model:

> Select identity *I* in projection *A*. Emphasize every renderable in every
> open projection whose identity resolves to *I* through the `DesignModel`'s
> relationships.

This is the differentiator, stated plainly. Datum's cross-probe is:
- **EXACT** — the join is identity, not a string. `R12` renamed to `R12A` on
  one side is impossible to "drift" against, because there is no second side;
  there is one `ComponentInstanceId` and the displayed refdes is a projection
  of it.
- **INSTANT** — no inter-editor message round-trip; the related-object set is
  a resolution over an in-memory model at a known `model_revision`.
- **DRIFT-FREE** — there is no second authority to fall out of sync. A split
  or incoherent project does not silently mis-probe; it opens in resolver
  recovery (`QG-RESOLVER-RECOVERY`, master §4.1), so cross-probe is correct
  or visibly degraded, never quietly wrong.
- **N-WAY** — not schematic<->PCB only. Because the identity spans every
  projection, ONE selection emphasizes the symbol, the footprint, the 3D body,
  the BOM line, the CAM pad/aperture, AND the related check findings, all at
  once. The number of projections is bounded only by how many are open.
- **AGENTS AND PROPOSALS CROSS-PROBE TOO** — an agent operates on the same
  `DesignModel` identities (Decision 004), so an agent citing
  `ComponentInstanceId` *I* drives the identical cross-probe a human click
  does, and selecting a proposal ghost cross-probes the committed objects it
  affects (`GUI_AI_SURFACES.md` `S2b_GhostSelected`).

We beat Altium's cross-probe BY CONSTRUCTION, not by tuning. There is no
faster or more correct string join to write; we removed the string join.

### 2.3 Where the architecture makes this free

Cross-probe is almost entirely a COORDINATION of primitives that already
exist:
- the SELECTION identity (`SelectionState`, `SelectedRef`, the identity triple)
  is owned by `GUI_INTERACTION_GRAMMAR.md` §5;
- the EMPHASIS vocabulary (`EmphasisMode::{None, DimOthers, HighlightOnly}`)
  is owned by `GUI_INTERACTION_GRAMMAR.md` §5 and rendered by
  `GUI_CANVAS_AND_RENDERING.md` §4.4;
- the MULTI-PANE arrangement (`LayoutGraph`, panes, PiP, `PinnedContext`,
  `NavigationStack`) is owned by `GUI_INFORMATION_ARCHITECTURE.md` §5/§6.

This spec adds the thin coordinating layer ON TOP: the identity-resolution
path that maps one selection to related objects across projections, the
cross-probe interaction state machine, and the per-pane emphasis broadcast.
The wedge is that this coordination is cheap and total precisely because the
identity is unified — there is no per-projection adapter, no string bridge,
one resolver.

---

## 3. Scope

### 3.1 In scope (the cross-probe coordination model)

- **Component cross-probe.** Select a `ComponentInstance` (or its symbol /
  footprint / 3D body / BOM line / assembly-drawing mark projection); emphasize
  all of its projections everywhere open.
- **Net cross-probe.** Select a net (or any pad/track/via/zone/ratsnest on it,
  or its schematic wire); emphasize the whole net across schematic and PCB,
  resolved over `Net.uuid` (not net-name string), reusing the canvas
  net-highlight treatment.
- **Pin/pad cross-probe.** Select a schematic pin; emphasize the corresponding
  board pad (and vice versa) — the finest-grained probe, resolved through the
  `ComponentInstance` pin<->pad mapping.
- **Check-finding cross-probe.** Select a `CheckFinding` (DRC/ERC/process);
  emphasize the committed source object(s) it cites AND, for process/CAM
  findings, the manufacturing projection it cites (000B: "DRC/process findings
  link to both source object and manufacturing projection"). This is the
  finding-navigator -> canvas jump.
- **N-way across projections.** schematic <-> PCB <-> 3D <-> BOM <->
  manufacturing/CAM <-> check-findings, all driven by one selection.
- **The interaction model:** hover-probe vs click-probe; cross-select-mode
  toggle (sticky vs momentary); push/jump-to-counterpart (focus + frame the
  counterpart in another pane); bidirectional (any projection can originate);
  multi-pane (per 000B) and multi-window behavior.
- **AI/proposal cross-probe:** an agent reference or a selected proposal ghost
  drives the same coordination.

### 3.2 Out of scope (owned elsewhere; coordinated here)

- The selection GESTURE, `SelectionState`, `EmphasisMode`, select-same-net, and
  the pick/box/lasso machine — `GUI_INTERACTION_GRAMMAR.md`. This spec consumes
  the selection it produces.
- The pixel-level highlight/dim rendering and `CanvasViewState.highlighted_nets`
  — `GUI_CANVAS_AND_RENDERING.md`. This spec decides WHAT to emphasize; that
  spec decides how it draws.
- The pane/PiP/split/floating docking model and `NavigationStack` —
  `GUI_INFORMATION_ARCHITECTURE.md`. This spec decides WHICH pane gets a
  jump/frame; that spec owns the layout graph it lands in.
- The proposal review state machine and accept/reject —
  `GUI_AI_SURFACES.md`. This spec defines only how a ghost selection
  cross-probes committed objects.
- The CAM projection geometry, zone-fill honesty, and online-DRC content —
  `GUI_CANVAS_AND_RENDERING.md` / `GUI_LIVE_FEEDBACK_AND_RULES.md`. This spec
  defines the source<->CAM cross-probe COORDINATION over them.

---

## 4. Commercial Benchmark + Match-vs-Exceed

### 4.1 The named patterns we benchmark

| Datum cross-probe element | Commercial pattern matched | Tool |
|---|---|---|
| Sticky cross-select mode (selection in one editor persistently mirrors into the other) | Altium **Cross Select Mode** (Tools > Cross Select Mode); selection in SCH highlights in PCB and back | Altium Designer |
| Jump/push to counterpart (focus + center the matching object in the other editor) | Altium **cross-probe / jump** (right-click > Cross Probe; `Ctrl+double-click` jump) | Altium Designer |
| Schematic<->layout cross-probe of components and nets | Cadence **OrCAD Capture <-> PCB Editor cross-probe** (Capture probe -> Allegro highlight) | Allegro / OrCAD |
| Net highlight that spans both views | Allegro **highlight/dehighlight** net across schematic + board | Allegro |
| Cross-probing during interactive work (probe while routing) | Siemens **Xpedition cross-probing** between schematic and layout | Xpedition |
| Findings -> object navigation (click a DRC error, jump to it) | Altium **Messages/Violations** double-click-to-locate; Xpedition DRC results navigation | Altium / Xpedition |
| Source <-> CAM cross-highlight | Altium **CAMtastic** / Gerber-preview locate-to-source (limited) | Altium |

KiCad is the FLOOR: KiCad has schematic<->PCB cross-probe by refdes/net-name
string match. Datum must at minimum equal that schematic<->PCB probe before
the surface is claimed shipped; we do not benchmark to it.

### 4.2 Where we MATCH (table stakes — reach the bar, do not innovate)

Sticky cross-select mode, jump-to-counterpart, schematic<->PCB component and
net probe, net highlight spanning both views, probe-while-routing, and
findings-to-object navigation are TABLE STAKES (master §3.1). We match the
commercial bar on fluency and correctness; a cross-probe element is not "done"
because it functions — it is done when, on the same SCH->PCB jump, it would
not embarrass us next to Altium's Cross Select Mode.

### 4.3 Where we EXCEED (the architecture wedge — most design attention)

1. **Identity join, not string join (EXACT, DRIFT-FREE).** Commercial
   cross-probe matches on refdes/net-name strings across two authorities and
   can drift. Datum resolves over one `DesignModel` by
   `ComponentInstanceId` / `Net.uuid` / the identity triple. There is no second
   authority to drift against. `it_crossprobe_no_string_join` (§12) is the
   falsifiable gate: a fixture whose displayed refdes/net-name differs from a
   naive string-join expectation still cross-probes the correct object, because
   the join is identity. If a string rename could break it, the wedge is not
   real — so the test is a gate, not a nicety.
2. **N-way, not schematic<->PCB only.** Because the identity spans every
   projection, one selection emphasizes symbol + footprint + 3D + BOM line +
   CAM pad + findings simultaneously. No commercial tool cross-probes a single
   click across six projection kinds, because none has a single spanning
   identity. This is the headline EXCEED.
3. **Findings and CAM are first-class probe endpoints.** A DRC/process finding
   cross-probes to BOTH its source object and its manufacturing projection
   (000B). A commercial tool's CAM viewer is typically a separate application
   with at best a partial locate-to-source; Datum's CAM is a projection of the
   same model, so source<->CAM cross-probe is the same identity resolution,
   not a bridge.
4. **Agents and proposals cross-probe through the identical path.** An agent
   (Decision 004) cites `DesignModel` identities, so an agent's "I changed
   `R12`" drives the same emphasis a human click does, and a selected proposal
   ghost cross-probes the committed objects it touches
   (`GUI_AI_SURFACES.md` `S2b_GhostSelected`). Cross-probe is therefore a
   shared human/agent/proposal vocabulary, not a human-editor-only feature.
5. **Deterministic and golden-pinned.** The related-object set is a pure
   function of (selection, `model_revision`); the same scene drives the live
   view and the golden harness, so a cross-probe golden is reproducible.
   Commercial cross-probe is a runtime message, not a pinned, replayable fact.

---

## 5. The Cross-Probe Doctrine (normative rules)

These hold across every projection and probe kind. A consuming spec may not
relax them.

- **X1 — Consumer state, never a commit.** Cross-probe selection and emphasis
  are consumer state (master §4.2; X-grammar G4). No probe transition reaches
  `commit()`, appends to the journal, advances `model_revision`, or dirties a
  shard. The cross-probe state machine (§9) has NO `[[COMMIT]]` transition by
  construction.
- **X2 — Identity, not string.** The related-object set resolves over the
  `DesignModel` identity (`ComponentInstanceId` / `Net.uuid` / identity triple
  / relationships), NEVER over a refdes or net-name string compare. Displayed
  refdes/net-name is a projection for the human, never the join key (§2.2).
- **X3 — Reuse the shared emphasis vocabulary.** Cross-probe emphasizes using
  `EmphasisMode` (`GUI_INTERACTION_GRAMMAR.md` §5) and the canvas
  highlight/dim treatment (`GUI_CANVAS_AND_RENDERING.md` §4.4). It introduces
  NO new emphasis system; default cross-probe emphasis is `DimOthers` (the
  Altium net-mask read), per grammar §5.3.
- **X4 — Render from the model only.** The related-object set is computed from
  the resolved `DesignModel` via the scene contract; cross-probe never reads
  source shards and never infers a relationship the model does not assert
  (master §4.1).
- **X5 — Degrade honestly.** If a projection is not open, cross-probe queues
  the emphasis so it applies when that projection opens (or, on push/jump,
  opens it per `LayoutGraph`); if the model is in resolver recovery, the probe
  set is marked degraded, never silently wrong (X2 + `QG-RESOLVER-RECOVERY`).
- **X6 — Bidirectional and symmetric.** Any projection can ORIGINATE a probe
  and any can RECEIVE one. The resolution from symbol->footprint and
  footprint->symbol is the same identity lookup run in either direction; there
  is no privileged "master" projection.
- **X7 — Supervision-first.** Cross-probe ships READ-ONLY first (Decision 013):
  in the supervision build it emphasizes and navigates committed state with no
  edit affordance. It is, by nature, a read-only selection feature, so it is a
  natural first deliverable (master §4.3).

---

## 6. Identity-Resolution Path (the heart of the spec)

This is the query X2 names: given one `SelectedRef`, produce the related
renderables in every projection. It runs ENTIRELY over the resolved
`DesignModel` and the scene-contract identity triple — no string join.

### 6.1 The resolution function (shape, not implementation)

```text
// Pure function over the resolved model. No commit, no shard read (X1/X4).
// Returns the cross-probe set: the related identities grouped by projection,
// plus the canonical design identity the selection resolved to.
fn resolve_cross_probe(
    origin: &SelectedRef,          // the originating selection (identity triple)
    model: &DesignModel,           // resolved at a known model_revision
    open_projections: &[ProjectionKind],
) -> CrossProbeSet
```

The steps, by origin `object_kind`:

1. **Canonicalize the origin to a DESIGN identity.** Map the originating
   renderable's identity triple to the design object it projects:
   - any pad/track/via/zone/ratsnest/schematic-wire whose `net_uuid` is set ->
     the `Net` identity (`Net.uuid`, `board_types.rs`);
   - any symbol/footprint/3D-body/BOM-line/CAM-component-mark ->
     the `ComponentInstance` identity (`ComponentInstanceId`,
     `component_instance.rs`), via the instance's `placed_symbol_refs` /
     `placed_package_refs`;
   - a schematic pin / board pad -> the (`ComponentInstanceId`, pin-name) pair
     for finest-grained pin<->pad probe;
   - a `CheckFinding` -> the set of source object identities it cites (and the
     CAM projection identity for process findings).

2. **Expand the design identity to ALL its projections.** For the canonical
   identity, gather every renderable across `open_projections` whose identity
   resolves to it:
   - a `Net.uuid` expands to every pad/track/via/zone/ratsnest carrying that
     `net_uuid` (the same grouping `GUI_CANVAS_AND_RENDERING.md` §4.4 does for
     net highlight) plus the schematic wires/labels of that net;
   - a `ComponentInstanceId` expands to its symbol (schematic), its footprint
     (PCB copper + courtyard + silk), its 3D body, its BOM row, and its CAM
     component mark, by following `placed_symbol_refs` / `placed_package_refs`
     and the component projection in each open `ViewTab`;
   - a (component, pin) pair expands to the schematic pin endpoint and the
     board pad of that pin;
   - a finding expands to its cited source objects + CAM projection geometry.

3. **Resolve relationship state for honesty (X5).** Use `RelationshipKind`
   (`relationship.rs` / `mod.rs`: `ImplementedBy`, `BoardOnly`,
   `SchematicOnly`, `ReverseEngineered`, `Pending`, `Mismatch`) to mark the
   probe set's completeness: a `SchematicOnly` component has no board
   projection to emphasize and the cross-probe set says so (it does not
   silently highlight nothing); a `Mismatch` relationship marks the probe
   degraded rather than guessing. This is the structural reason Datum cannot
   "drift" — the model already CARRIES the relationship state a string join
   throws away.

4. **Group by projection and hand to emphasis.** Return the related identities
   bucketed per `ProjectionKind`; each open pane applies `EmphasisMode` to its
   bucket (§7). Net buckets reuse `CanvasViewState.highlighted_nets`; component
   and finding buckets emphasize by `object_id` membership (the canvas already
   keys highlight/dim by membership, grammar §5, canvas §4.4).

### 6.2 Why this cannot drift (the wedge as a property, not a claim)

There is exactly ONE canonicalization step (6.1.1) and it maps to a stable
design identity, not a string. Every projection in 6.1.2 is reached by
following that identity through the model's own references
(`placed_symbol_refs`, `placed_package_refs`, `net_uuid`, relationships). No
step compares two display strings. Therefore renaming a refdes or a net cannot
mis-probe: the rename is a projection of the same identity, and the identity is
the join. `it_crossprobe_no_string_join` (§12) asserts exactly this against a
fixture engineered to break any string-join implementation.

---

## 7. The Emphasis Model (reusing the shared vocabulary)

Cross-probe does NOT define a new emphasis system (X3). It drives the existing
`EmphasisMode` (`GUI_INTERACTION_GRAMMAR.md` §5) and the canvas highlight/dim
treatment (`GUI_CANVAS_AND_RENDERING.md` §4.4) per pane:

- **Default emphasis is `DimOthers`** — the cross-probe set renders at full
  emphasis, everything else dims, matching Altium's net-mask read (grammar
  §5.3). `HighlightOnly` is the alternative for a plain click that should not
  mask the surrounding context (OQ, ties to grammar OQ4).
- **Per-pane emphasis is independent but coordinated.** Each open pane applies
  the emphasis to ITS bucket of the cross-probe set; the PCB pane dims to its
  net members, the schematic pane dims to that net's wires, the 3D pane
  emphasizes the body — one selection, per-projection-appropriate emphasis,
  one coordinating set.
- **Net cross-probe reuses net highlight exactly.** A net probe sets
  `CanvasViewState.highlighted_nets = { net_uuid }` on every PCB pane and the
  schematic equivalent; there is no parallel "cross-probe net color" — it is
  the existing net-highlight lane (canvas §4.4), which is itself the
  PCB-canvas half of this differentiator.
- **Hover-probe vs click-probe have distinct treatments.** Hover emphasis is
  lighter/transient (a "preview" of the probe), click emphasis is the full
  `DimOthers` set; the distinction is a render treatment keyed by the
  cross-probe state (§9), not a new primitive.
- **Finding probe emphasis** marks the cited source object(s) with the
  finding's severity accent (owned by `GUI_LIVE_FEEDBACK_AND_RULES.md`) on top
  of the cross-probe set, so the violating object is unmistakable in the jump
  target pane.

---

## 8. Scene-Contract Extension (`CrossProbeSet`)

Cross-probe needs ONE typed, versioned consumer-state struct the panes, the
goldens, and the AI/proposal path all resolve against. It is CONSUMER STATE
(X1): it lives in the `gui-app`/`gui-render` consumer layer alongside
`SelectionState` and `CanvasViewState`, NOT in the engine `DesignModel`, and it
is never serialized into a shard. It carries no design authority — it is a
RESOLVED VIEW of identities that already exist in the model.

```text
pub const CROSS_PROBE_SET_VERSION: &str = "datum.cross_probe_set.v1";

/// The resolved cross-probe set for the current selection. Consumer state:
/// recomputed when selection or model_revision changes; never journaled.
/// Byte-stable ordering (master §6.1): buckets sorted by ProjectionKind, refs
/// sorted by object_id, so a cross-probe golden is reproducible.
pub struct CrossProbeSet {
    /// The canonical design identity the origin resolved to (§6.1.1). Exactly
    /// one of these is set; it is the JOIN KEY, and it is an identity, never a
    /// string (X2).
    pub canonical: CrossProbeIdentity,
    /// The originating selection, by identity triple (which pane/click started
    /// the probe). Cross-probe is bidirectional (X6): origin is informational.
    pub origin: SelectedRef,
    /// Related renderables grouped by projection. Each open pane emphasizes
    /// its matching bucket via EmphasisMode (§7). Empty bucket for a
    /// projection means "no projection of this identity exists there"
    /// (e.g. a SchematicOnly component has no Pcb bucket) — surfaced honestly
    /// (X5), not as a silent miss.
    pub buckets: Vec<CrossProbeBucket>,
    /// How the probe was raised; drives hover-vs-click emphasis treatment (§7)
    /// and the state machine (§9).
    pub mode: CrossProbeMode,
    /// Relationship completeness for honest degrade (X5): derived from
    /// RelationshipKind. `Complete` = every expected projection resolved;
    /// `Partial` = a known one-sided relationship (BoardOnly/SchematicOnly);
    /// `Degraded` = Mismatch/recovery, probe set may be incomplete.
    pub completeness: CrossProbeCompleteness,
}

pub enum CrossProbeIdentity {
    /// Net identity: Net.uuid (board_types.rs). Reuses net highlight (§7).
    Net { net_uuid: String },
    /// Component identity: ComponentInstanceId (component_instance.rs).
    Component { component_instance_id: String },
    /// Finest-grained: a pin/pad of a component.
    Pin { component_instance_id: String, pin_name: String },
    /// A check finding's cited identities (DRC/ERC/process).
    Finding { finding_id: String },
}

pub struct CrossProbeBucket {
    pub projection: ProjectionKind,
    /// Related renderables in this projection, by identity triple, that the
    /// pane should emphasize. Sorted by object_id (determinism).
    pub refs: Vec<SelectedRef>,   // SelectedRef from GUI_INTERACTION_GRAMMAR §5
}

pub enum ProjectionKind {
    Schematic,
    Pcb,
    ThreeD,        // 3D body view (M8 seam; reserved-but-typed, canvas §4.9)
    Bom,
    Manufacturing, // CAM: Gerber/NC-drill/paste/mask projections (000B)
    Findings,      // the check/DRC/ERC findings navigator
}

pub enum CrossProbeMode {
    /// Transient preview emphasis from hover (lighter treatment, §7).
    HoverProbe,
    /// Full DimOthers emphasis from a click (§7).
    ClickProbe,
    /// Sticky cross-select-mode: every selection mirrors until toggled off
    /// (Altium Cross Select Mode parity).
    CrossSelectMode,
}

pub enum CrossProbeCompleteness { Complete, Partial, Degraded }
```

Notes (master §6.1 determinism + no-authority):
- `CrossProbeSet` adds NO per-primitive field to the scene; every renderable
  already carries the identity triple. The set is a RESOLVED OVERLAY keyed by
  `object_id` membership, exactly like net highlight (canvas §4.4) — no
  parallel `net_id`/`component_id` field is introduced or permitted.
- `canonical` is the single join key and it is always an identity, never a
  display string — the type makes X2 a compile-time property, not a convention.
- `ThreeD` and `Manufacturing` buckets are typed now even though 3D is M8 and
  CAM panes are a later increment, so the N-way contract reserves the seam
  without forcing those panes to exist first (canvas §4.9 / 000B).

Consumer-state boundary (hard rule, mirrors canvas §5): a function that
produces or mutates a `CrossProbeSet` MUST NOT have `&mut DesignModel`, a
journal handle, or `commit()` in scope. `it_crossprobe_no_journal` (§12)
asserts a probe leaves `model_revision` unchanged and the journal empty.

---

## 9. Cross-Probe Interaction State Machine

States and transitions, not prose (master §6.2). Notation: `STATE
--event[guard]/effect--> STATE`. There is NO `[[COMMIT]]` transition: every
transition mutates consumer state only (X1). Cancel/clear is marked
`[[CLEAR]]` and leaves the base scene with zero emphasis and zero journal
effect.

### 9.1 The two orthogonal axes

The coordinator is a product of two ORTHOGONAL pieces of state, and conflating
them is the usual way a cross-probe UI rots:

- a **probe-phase state** — `ProbeIdle` / `ProbeHovering` / `ProbeActive` —
  describing whether a set is currently emphasized and how it was raised;
- a **sticky flag** `cross_select_mode: bool` — Altium Cross Select Mode parity
  — describing whether a plain grammar `select` AUTO-raises a click-probe. It is
  a persistent toggle, NOT a phase; it is orthogonal to the phase machine and
  modifies the GUARD on the `select` transition rather than being a state of its
  own. This is why `CrossSelectMode` in §8 is the `mode` STAMPED on a set raised
  while the flag is set, not a fourth phase.

`CrossProbeSet.mode` is therefore derived, not free: `HoverProbe` in
`ProbeHovering`; `ClickProbe` for a click while `cross_select_mode == false`;
`CrossSelectMode` for a click (or grammar select) while `cross_select_mode ==
true`. The phase machine and the flag never disagree because the flag is read,
not stored twice.

### 9.2 Cross-probe coordinator (sits over the grammar's select machine)

```
ProbeIdle (no cross-probe emphasis; base scene)
  --hover(obj)[hover_probe_enabled]/
       set = resolve_cross_probe(obj, model, open); set.mode = HoverProbe;
       broadcast light emphasis to each pane's bucket--> ProbeHovering
  --select(obj)  // from grammar Select machine (grammar §10.2)
       /set = resolve_cross_probe(obj, model, open);
        set.mode = if cross_select_mode { CrossSelectMode } else { ClickProbe };
        broadcast DimOthers to each pane's bucket--> ProbeActive

ProbeHovering
  --unhover/clear hover emphasis--> ProbeIdle
  --hover(obj2)/recompute hover set for obj2; rebroadcast light--> ProbeHovering
  --select(obj)/promote to a durable set (mode per flag, as above)--> ProbeActive
  --clear | Esc | move-off-canvas [[CLEAR]]/clear set, panes to base--> ProbeIdle

ProbeActive (a durable click/sticky set is emphasized across panes)
  --hover(other)/overlay HoverProbe preview without losing the durable set--> ProbeActive
  --select(obj2)/recompute set for obj2 (mode per flag); rebroadcast--> ProbeActive
  --jump(pane P)[set has non-empty bucket for P]/focus P, frame its refs
       (camera move, IA SM-2); push NavigationStack entry (IA §6)--> ProbeActive
  --push(counterpart)[set has a counterpart projection identity]/open or raise
       that projection per LayoutGraph (PiP/split/tab, IA SM-4), frame it--> ProbeActive
  --clear | Esc | click(empty) [[CLEAR]]/clear set, all panes to base--> ProbeIdle
  --model_revision_changed[origin identity still present]/recompute set
       against new revision (X4); rebroadcast--> ProbeActive
  --model_revision_changed[origin identity vanished]/mark set Degraded (X5),
       hold last emphasis with a degraded marker, do not silently clear--> ProbeActive

// The sticky flag is orthogonal (§9.1): toggling it changes only the GUARD on
// the select transitions above; it raises/clears no emphasis on its own.
(any phase)
  --toggle(cross_select_mode)/cross_select_mode = !cross_select_mode;
       current set persists; no broadcast--> (same phase)
```

The `[[CLEAR]]` transitions are the only terminations and each leaves the base
scene with zero emphasis and, by X1, zero journal effect; there is no
`[[COMMIT]]` transition anywhere in this machine — that absence is the
state-machine-level proof of doctrine X1.

### 9.3 Probe origination is bidirectional (X6)

The originating `select`/`hover` event arrives from WHICHEVER pane has focus —
the schematic select machine, the PCB select machine, the BOM list, the
findings navigator (a finding click is a `select` with `object_kind=finding`),
or an agent/proposal (§11). The coordinator is symmetric: it does not care
which projection originated the event; it canonicalizes (§6.1.1) and broadcasts
to all (§6.1.4). There is no master projection.

### 9.4 Hover-probe vs click-probe (the two-tier model)

- **Hover-probe** (`HoverProbe`) is a transient PREVIEW: a light emphasis that
  follows the cursor and clears on un-hover, so a designer can "scrub" a row of
  pads and watch the schematic light up without committing a selection. It is
  off by default for dense canvases (perf + noise) and on for list/finding
  panes; the default is OQ.
- **Click-probe** (`ClickProbe`) is the durable `DimOthers` set that persists
  until cleared or replaced. This is the primary cross-probe.
- **Cross-select-mode** (`CrossSelectMode`) makes every grammar selection
  auto-raise a click-probe broadcast — the sticky Altium parity. Toggling it
  off leaves the last set in place but stops mirroring new selections.

### 9.5 Jump vs push (the navigation verbs)

- **Jump (to-probe)** focuses an ALREADY-OPEN pane that has a bucket for the
  set and frames its refs (camera move + `NavigationStack` push, IA §5/§6) —
  "show me where this is in the PCB pane I already have open".
- **Push (to-counterpart)** opens or raises the counterpart projection if it is
  not already a focused pane — per `LayoutGraph` it may open as a PiP, a split,
  or focus an existing tab (IA owns the layout decision; OQ which is default).
  This is the Altium jump-to-counterpart parity, generalized N-way.
- Both are consumer-state camera/layout actions (IA SM-2/SM-4); neither
  mutates the model.

---

## 10. Multi-Pane and Multi-Window Behavior

Cross-probe is where multi-pane composition (000B / IA §5) earns its keep: the
value of two panes side by side is that one selection lights up both.

- **Multi-pane (same window).** Every open pane in the `LayoutGraph` that has a
  bucket in the `CrossProbeSet` applies emphasis to that bucket simultaneously.
  A schematic split beside a PCB pane: click a footprint, the symbol emphasizes
  in the other split — no message, one set, two consumers (000B "concurrent
  multi-view"; IA §5.2 split/tile).
- **PiP and PinnedContext.** A `PinnedContext` PiP (000B: "schematic PiP pinned
  while routing physical layout") is a first-class cross-probe receiver:
  routing a net in the main PCB pane emphasizes that net's wires in the pinned
  schematic PiP. The pin link is the IA `pin-to-selection` edge (IA §5.2); the
  cross-probe set drives what the pinned pane emphasizes.
- **Multi-window.** Floating windows (IA §5.2 `Floating`, single-monitor first)
  receive the same broadcast: the `CrossProbeSet` is consumer state shared
  across the application's panes regardless of which OS window hosts them. There
  is no per-window database, so cross-window probe is the same broadcast — the
  unified-model wedge again removes the hard part. Multi-MONITOR window
  placement is a master Non-Goal for the first cut; cross-probe ACROSS floating
  windows is in scope, multi-monitor persistence is not.
- **Closed projections.** If the set has a bucket for a projection that is not
  open, the emphasis is queued (X5): opening that projection later applies it,
  or a `push` (§9.5) opens it on demand.

---

## 11. AI / Proposal Cross-Probe

Cross-probe is not a human-editor-only feature; agents and proposals use the
identical identity path (Decision 004: AI tooling operates on the same resolved
`DesignModel`). This is a genuine EXCEED (§4.3 #4).

- **Agent-originated probe.** When an agent references a design identity
  (`ComponentInstanceId` / `Net.uuid` / object_id) in a message, the Assistant
  surface (`GUI_AI_SURFACES.md`) can raise the SAME cross-probe a human click
  raises: the reference is canonicalized (§6.1.1) and broadcast (§6.1.4). A
  designer reading "I rerouted `GND` and it now clears `U3`" can click the
  agent's reference and watch `GND` and `U3` emphasize across panes — the
  agent and the human share the cross-probe vocabulary.
- **Proposal-ghost probe.** Selecting a proposal ghost
  (`GUI_AI_SURFACES.md` `S2b_GhostSelected`) cross-probes the COMMITTED objects
  the proposal affects: the ghost's `proposal_action_id` resolves to the
  committed identities it adds/removes/modifies, and those emphasize across
  panes so the supervisor sees, in every projection, what the proposal touches
  before accepting. This is the cross-probe side of the marquee AI-native
  ghost/diff surface (master differentiator 1); the two differentiators
  COMPOSE — the ghost shows the proposed geometry, cross-probe shows its blast
  radius across projections.
- **Same consumer-state rule.** An agent/proposal probe is still consumer state
  (X1): it emphasizes and navigates, it never commits. Accepting a proposal is
  a separate, journaled action owned by `GUI_AI_SURFACES.md`; cross-probe only
  illuminates it.

---

## 12. Proof Slices

Each names the `datum-test` regression fixture (default; the canonical M7
fixture with a schematic, 11 footprints, 32 routed segments) and the gates.
Supervision-reflection precedes interaction (master §4.3): read-only
cross-probe ships before any editing leans on it.

- **PS-XPROBE-1 (component cross-probe, supervision, read-only).** Open
  `datum-test` with a schematic pane and a PCB pane. Click a footprint in the
  PCB pane; assert the SAME `ComponentInstance`'s symbol emphasizes in the
  schematic pane (`DimOthers`), resolved over `ComponentInstanceId` (the
  instance's `placed_symbol_refs`/`placed_package_refs`), not a refdes string.
  Gates: renders real committed state; no `commit()`; the journal is untouched
  (X1); the join is identity (X2, see `it_crossprobe_no_string_join`).
- **PS-XPROBE-2 (net cross-probe, N-way over `Net.uuid`).** Click a pad on a
  net in the PCB pane; assert every track/via/zone/ratsnest on that
  `Net.uuid` emphasizes (reusing `highlighted_nets`, canvas §4.4) AND the
  net's wires emphasize in the schematic pane. Gate: one selection, two
  projections, one `Net.uuid`; consumer state only.
- **PS-XPROBE-3 (finding cross-probe / jump).** Select a DRC finding in the
  Violations navigator; assert it cross-probes (jumps + frames) the cited
  source object in the PCB pane and, where the finding is a process/CAM
  finding, the manufacturing projection too (000B). Gate: finding -> object +
  CAM navigation; `NavigationStack` push (IA); zero `Operation`.
- **PS-XPROBE-4 (cross-select-mode sticky + push-to-counterpart).** Toggle
  cross-select mode on; select three different objects in turn in the schematic
  pane; assert each auto-broadcasts a click-probe to the PCB pane (Altium Cross
  Select Mode parity). Then `push` a component whose PCB pane is closed; assert
  the counterpart projection opens/raises per `LayoutGraph` and frames it.
  Gate: sticky mirroring; push opens via IA layout, not a new authority path;
  consumer state only.
- **PS-XPROBE-5 (proposal-ghost cross-probe — the wedges compose).** Load a
  fixture proposal; select a ghost (`S2b_GhostSelected`); assert the COMMITTED
  objects the proposal affects emphasize across schematic + PCB panes (the
  proposal's blast radius), with the ghost itself still rendered by the AI
  canvas. Gate: agent/proposal cross-probe uses the identity path (§11); no
  commit; differentiator 1 + differentiator 2 compose.

---

## 13. Visual-Golden Acceptance

Every cross-probe surface ships a golden + interaction test in the `gui-render`
harness (master §4.4 / §6.5). Acceptance = the golden renders real committed
state from `datum-test` and the interaction test exercises the named
state-machine transitions. Cross-probe goldens that span two panes capture the
multi-pane `LayoutGraph` arrangement so the emphasis-in-both-panes is the
asserted picture.

| Golden | Drives | Asserts | Diff |
|---|---|---|---|
| `xprobe_component_sch_pcb.png` | PS-XPROBE-1 | footprint clicked in PCB pane; same-instance symbol emphasized (`DimOthers`) in schematic pane | tolerance |
| `xprobe_net_nway.png` | PS-XPROBE-2 | one net highlighted across PCB copper + schematic wires over one `Net.uuid` | tolerance |
| `xprobe_finding_jump.png` | PS-XPROBE-3 | DRC finding selected; cited source object framed + emphasized with severity accent | tolerance |
| `xprobe_cross_select_mode.png` | PS-XPROBE-4 | sticky-mode selection mirrored into the counterpart pane | tolerance |
| `xprobe_proposal_blast_radius.png` | PS-XPROBE-5 | ghost selected; affected committed objects emphasized across panes | tolerance |

Interaction tests (non-image, drive the state machine + the wedge gates):
- `it_crossprobe_no_string_join` — the FALSIFIABLE wedge gate (§4.3 #1 / §6.2):
  on a fixture whose displayed refdes/net-name is engineered to defeat a naive
  string join, the probe still resolves the correct object because the join is
  `ComponentInstanceId`/`Net.uuid`. If a rename can break the probe, this test
  fails and the wedge is not real.
- `it_crossprobe_no_journal` — every probe transition (hover/click/jump/push/
  sticky) leaves `model_revision` unchanged and the journal empty (X1).
- `it_crossprobe_nway_buckets` — one selection of a component present in
  schematic + PCB (and, when those panes exist, BOM) produces non-empty buckets
  for each open projection, all resolving to one `ComponentInstanceId` (§6.1).
- `it_crossprobe_bidirectional` — symbol->footprint and footprint->symbol
  resolve to the identical `CrossProbeSet.canonical` (X6).
- `it_crossprobe_honest_degrade` — a `SchematicOnly` / `Mismatch` relationship
  yields `completeness != Complete` and an explicitly empty PCB bucket, never
  a silent miss (X5).
- `it_crossprobe_proposal_identity` — a proposal-ghost selection cross-probes
  the committed identities the proposal's `proposal_action_id` resolves to
  (§11), via the same path a human click uses.

A surface is not accepted on prose; it is accepted when its golden renders the
two-pane emphasis from real `datum-test` state and its interaction test
exercises the named transitions and wedge gate.

---

## 14. Non-Goals

- Owning the selection GESTURE, `SelectionState`, `EmphasisMode`, or
  select-same-net — `GUI_INTERACTION_GRAMMAR.md`. This spec coordinates them.
- Owning pixel-level highlight/dim rendering or `CanvasViewState.highlighted_nets`
  — `GUI_CANVAS_AND_RENDERING.md`. This spec decides WHAT to emphasize.
- Owning the pane/PiP/split/floating docking model, `NavigationStack`, or
  `WorkbenchProfile` — `GUI_INFORMATION_ARCHITECTURE.md`. This spec decides
  which pane a jump/push targets; IA owns the layout it lands in.
- Owning the proposal review/accept state machine — `GUI_AI_SURFACES.md`.
  This spec defines only how a ghost selection cross-probes committed objects.
- Owning the CAM projection geometry, zone-fill honesty, or online-DRC content
  — `GUI_CANVAS_AND_RENDERING.md` / `GUI_LIVE_FEEDBACK_AND_RULES.md`.
- Any cross-probe that MUTATES — cross-probe is read-only selection/emphasis by
  doctrine (X1); a probe never commits, never journals, never edits.
- A string/refdes/net-name join fallback — explicitly forbidden (X2); the
  identity path is the only path, and removing the string join IS the wedge.
- Multi-MONITOR window placement persistence in the first cut (master §9;
  cross-probe ACROSS floating windows IS in scope, multi-monitor persistence
  is not).
- 3D and full CAM-pane cross-probe as a first-cut REQUIREMENT — the `ThreeD`
  and `Manufacturing` buckets are typed and reserved (§8), but those panes are
  later increments (3D = M8); schematic<->PCB<->findings is the first-cut body.
- Editing `specs/PROGRESS.md`, `specs/SPEC_PARITY.md`, `crates/`, or
  `mcp-server` from this authoring track.

---

## 15. Open Owner Questions

1. **Hover-probe default per pane.** Is hover-probe on by default for
   list/finding panes and off for dense canvases (the §9.4 proposal), or off
   everywhere until explicitly enabled? Affects perf and noise; ties to grammar
   OQ4 (default emphasis per gesture).
2. **Default emphasis for cross-probe.** Is `DimOthers` the right default for a
   click-probe (the Altium net-mask read), or should a plain cross-probe use
   `HighlightOnly` and reserve `DimOthers` for net/same-net probes? Must agree
   with `GUI_INTERACTION_GRAMMAR.md` OQ4 and §5.5 (selection persistence across
   projection switch).
3. **Push-to-counterpart default layout.** When `push` opens a closed
   counterpart projection, does it default to a PiP, a split, or focusing an
   existing tab? This is an IA layout decision (000B OQ "which tab layout modes
   first"; IA OQ1) that cross-probe consumes — confirm the default.
4. **Selection persistence across projection switch.** When the user
   cross-probes SCH->PCB, does the selection set carry by identity into the
   target pane's `SelectionState` automatically, and does emphasis mode carry
   with it? Shared with grammar OQ5; this spec needs the answer to decide
   whether a jump leaves the target pane SELECTED or only EMPHASIZED.
5. **Cross-select-mode default state.** Does cross-select mode default ON
   (every selection always mirrors, the Altium-on power-user default) or OFF
   (explicit, less surprising)? And is it a global toggle or per-pane?
6. **N-way scope for the first cross-probe surface.** The first cut is
   schematic<->PCB<->findings. Are BOM and CAM cross-probe required day one, or
   fast-follow once the BOM/Output and live-CAM panes are credible
   (000B live-production sequencing)? 3D is M8 regardless.
7. **Finding cross-probe to CAM.** For a process/CAM finding, do we require the
   source-object AND the manufacturing-projection emphasis in the first cut
   (000B "findings link to both"), or is source-object-only acceptable until
   the live-CAM panes land?
8. **Cross-window probe scope.** Cross-probe across FLOATING windows is in
   scope (§10); is it required in the first cut, or is single-window multi-pane
   sufficient first, with floating-window probe arriving with the floating
   docking increment (IA OQ1)?
