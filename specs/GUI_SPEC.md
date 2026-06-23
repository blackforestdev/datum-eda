# Datum EDA GUI Specification (Master)

Status: draft GUI specification, 2026-06-22, benchmarked to commercial
EDA. Controlling. This document supersedes the "M7 review spike"
framing of `specs/M7_FRONTEND_SPEC.md` (see "Retiring the M7 review
spike" below). It does not redefine engine-first/GUI-last sequencing
(`PLAN.md`, 2026-03-24); it states the GUI requirements the standard
build workflow constructs the GUI from.

Driven by:
- `docs/decisions/PRODUCT_MECHANICS_000B_VIEW_COMPOSITION_AND_LIVE_PRODUCTION.md`
- `docs/decisions/PRODUCT_MECHANICS_002_MANUAL_EDITOR_BASELINE.md`
- `docs/decisions/PRODUCT_MECHANICS_012_APPLICATION_QUALITY_BAR.md`
- `docs/decisions/PRODUCT_MECHANICS_013_GUI_SUPERVISION_AND_PARITY.md`
- `docs/DATUM_PRODUCT_MECHANICS.md`
- `specs/M7_FRONTEND_SPEC.md` (the read-only review opening, now folded in)
- `specs/SCHEMATIC_EDITOR_SPEC.md`
- `crates/gui-protocol` `BoardReviewSceneV1` scene contract
- `crates/gui-render` renderer + visual-regression harness
- `docs/audits/CODE_VS_SPEC_CONFORMANCE_AUDIT.md` (GUI Substrate section)

---

## 1. Purpose

This is the controlling GUI requirement set the standard build workflow
builds from. It states what the Datum GUI must be, the bar it is held
to, the architecture it is not allowed to violate, and the executable
shape (scene schemas, interaction state machines, panel specs, proof
slices, visual goldens) every GUI surface ships with.

It is a master index. The seven area specifications under `specs/gui/`
carry the per-domain detail (six domain areas plus the dedicated
cross-probe coordination spec, master differentiator 2). This document
owns the cross-cutting contract: the bar, the thesis, the architecture
constraints, the buildability standard, and the integration/sequencing
the workflow consumes.

This document does not implement anything and does not edit code,
`specs/PROGRESS.md`, or `specs/SPEC_PARITY.md`. The GUI rows in those
two files are added later, by the workflow, when the codebase author is
clear of the GUI crates (see "Integration & Sequencing").

---

## 2. The Bar

Datum's GUI is benchmarked against the best COMMERCIAL EDA tools, not
against open source.

Benchmark set (the tools every "good enough?" call is measured against):
- Altium Designer — primary UX reference. Properties panel, heads-up
  display (HUD) cursor readout, online DRC, ActiveRoute, PCB/SCH
  cross-probe, panel/dockable workspace model.
- Cadence Allegro / OrCAD — Constraint Manager, spreadsheet-class
  constraint authoring, dynamic shape (zone) editing, ratsnest control.
- Siemens Xpedition — sketch routing, plow/shove interaction, batch +
  interactive DRC, multi-board/system design surfaces.
- Zuken CR-8000 — concurrent multi-board, 3D-aware co-design, design
  reuse blocks.

KiCad is a FLOOR we exceed, never a target. KiCad is cited only when
explaining what we must at minimum equal before we are allowed to claim
we have shipped a surface. Stop benchmarking open source. Every surface
spec must cite the specific commercial UX pattern it matches or beats —
naming the tool and the feature (e.g. "matches the Altium Properties
panel context-by-selection behavior", "beats Allegro Constraint Manager
by making each constraint row a typed op through `commit()`").

A surface is not "good enough" because it works. It is good enough when
it would not embarrass us next to a $10k/seat tool on the same task.

---

## 3. The Thesis

Match the table stakes. Win on the wedge our architecture uniquely
enables.

### 3.1 Table stakes (must match, not innovate)

These exist in every commercial tool and must reach the commercial bar.
They get correctness and fluency attention, not novelty:
- canvas quality: precise hit testing, smooth pan/zoom, layer rendering,
  HUD cursor readout (matches Altium HUD)
- interaction fluency: visible pre-edit preview, snap feedback, command
  cancellation, no modal dead ends (`QG-DIRECT-EDITING-FEEL`)
- panels: a context-sensitive Properties panel (matches Altium), a
  constraint surface (matches Allegro Constraint Manager), a findings
  navigator, a project/structure panel
- cross-probe: schematic <-> PCB cross-select (matches Altium)
- live DRC: online checking with navigable findings (matches Altium
  online DRC)

### 3.2 The three differentiators (win here; most design attention)

No commercial tool has these as we will build them. They get the
deepest design attention, the most explicit state machines, and the
strongest golden coverage.

1. AI-native canvas. Agent proposals render as reviewable GHOSTS / DIFFS
   directly on the board and schematic — accept/reject visually, in
   place, with the diff highlighted against committed geometry. No
   commercial tool renders an agent's proposed edit as an in-canvas
   ghost you approve. This is the marquee surface
   (`specs/gui/GUI_AI_SURFACES.md`).
2. Unified-model cross-probe. One resolved `DesignModel` makes
   cross-select clean, instant, drift-free, and N-WAY across every
   projection (schematic <-> PCB <-> 3D <-> BOM <-> manufacturing <->
   check findings) — selection is one identity resolved across
   projections, not a name/refdes/net-name join across two file
   authorities. We beat Altium's cross-probe by construction, not by
   tuning. The cross-probe MODEL (the N-way identity-resolution and
   coordination surface) is owned by its dedicated marquee spec
   `specs/gui/GUI_CROSS_PROBE.md`; the underlying MECHANISM stays in the
   area files it coordinates — selection identity in
   `specs/gui/GUI_INTERACTION_GRAMMAR.md`, emphasis rendering in
   `specs/gui/GUI_CANVAS_AND_RENDERING.md`, and multi-pane composition in
   `specs/gui/GUI_INFORMATION_ARCHITECTURE.md`.
3. Deterministic engine-backed canvas. Every visual mutation is a typed
   `Operation` through the single journaled `commit()`. Therefore every
   visual action is replayable, undoable across close/reopen (journal
   cursors, not in-memory stacks), and scriptable through the identical
   path as CLI/MCP. The canvas is a deterministic instrument, not a
   stateful editor with a save button.

---

## 4. Architecture Constraints (non-negotiable)

These come from the ratified doctrine (000B / 002 / 012 / 013 and
`docs/DATUM_PRODUCT_MECHANICS.md`). A surface spec may not relax them.

### 4.1 Render entirely from the resolved DesignModel

The GUI renders ONLY from the resolved `DesignModel` via the
`gui-protocol` scene contract. It owns no design authority. It never
reads source shards directly and never infers geometry or semantics
that materially alter design meaning (M7_FRONTEND_SPEC §2.3). All
surfaces read the `ProjectResolver`-produced model at a known
`model_revision` (Decision 002). A split or incoherent project opens in
resolver recovery / diagnostic mode, never as accepted truth
(`QG-RESOLVER-RECOVERY`).

### 4.2 Mutate only through the single commit() / journal

Every mutation is a typed `Operation` (or a `Proposal` that later calls
the same `commit()`) through the single journaled commit path
(Decision 002). No canvas tool, inspector field, constraint row,
terminal lane, or AI lane writes a shard directly. Selection, hover,
drag, route-in-progress, and camera are CONSUMER STATE — never
operations, never journaled (000B; project principle: interactive
behaviors produce operations, they are not operations). The terminal
lane is a real PTY and is not an edit API; design changes launched from
it reduce to typed ops on `commit()` (`QG-PTY-REAL-TERMINAL`).

### 4.3 Supervision-reflection is the FIRST GUI deliverable

Per Decision 013, the GUI's first job is to VISUALLY DISPLAY committed
engine/native state so a human can audit it before any interactive
editing exists: native authored geometry, operation/transaction
results, check findings, manufacturing projections, agent/session
activity. This is read-only — the supervisor's instrument panel.
Interactive authoring is a later, named phase, not held behind a vague
"M8 later". Supervision-reflection parity precedes interactive-authoring
parity in every domain.

### 4.4 Visual goldens are the acceptance mechanism

Every surface ships with a VISUAL GOLDEN plus an interaction test in the
`gui-render` screenshot-regression harness (bless + exact/tolerance
diff; `crates/gui-render/src/visual_runner.rs`,
`visual_manifest.rs`, `visual_diff.rs`). A surface is not accepted on
prose or unit tests; it is accepted when its golden renders real
committed state and its interaction test exercises the state machine.
The dormant `check_gui_parity` fuse (Decision 013) arms when the
interactive editor phase begins; until then each surface carries its own
golden-backed definition-of-done.

---

## 5. The Seven Area Specifications

The per-domain detail lives in `specs/gui/`. There are seven area files:
six domain areas (items 1–6 below) plus the dedicated cross-probe
coordination spec (item 7, master differentiator 2). Each is controlling
for its domain and conforms to this master. This §5 is the CANONICAL
area map — the filenames below are authoritative and every area file's
cross-references resolve to exactly these seven files. One line each:

1. `specs/gui/GUI_SUPERVISION_REFLECTION.md` — area 1, the read-only
   instrument panel: render committed `DesignModel`, checks, journal,
   sessions, proposal existence; the first deliverable in every domain
   (Decision 013).
2. `specs/gui/GUI_INTERACTION_GRAMMAR.md` — area 2 (grammar half): the
   command-first modeless model, keyboard reachability, selection model +
   filter, snap, the shared ghost-preview contract, undo/redo against the
   journal, command palette, and the cross-probe SELECTION IDENTITY over
   the single `DesignModel` (matches Altium HUD / editing feel; partly
   delivers DIFFERENTIATOR 2; `QG-DIRECT-EDITING-FEEL`).
3. `specs/gui/GUI_CANVAS_AND_RENDERING.md` — area 2 (canvas half): canvas
   quality, layer compositing, HUD rendering, net highlight/dim, the
   render target for ghosts, and the in-canvas zone-fill honesty + CAM
   projection rendering (matches Altium DirectX canvas / single-layer
   mode; `QG-ZONEFILL-HONESTY`).
4. `specs/gui/GUI_INFORMATION_ARCHITECTURE.md` — area 3 (panels half):
   context-sensitive Inspector (Properties), dockable workspace, design
   tree, findings navigator, profiles; context rules and
   validate-before-commit (matches Altium Properties panel + Workspaces).
5. `specs/gui/GUI_LIVE_FEEDBACK_AND_RULES.md` — area 3 (rules half):
   online-DRC live feedback during edits, findings overlay/navigator, and
   the Constraint/Rules editor with each row a typed op through `commit()`
   (matches Altium online DRC + Rules & Constraints / Allegro Constraint
   Manager).
6. `specs/gui/GUI_AI_SURFACES.md` — area 4, DIFFERENTIATOR 1: agent
   proposals as in-canvas ghosts/diffs, visual accept/reject, the ghost
   scene-contract extension, the review/assistant/terminal state machines.

7. `specs/gui/GUI_CROSS_PROBE.md` — area 7, DIFFERENTIATOR 2: the
   MARQUEE unified-model cross-probe. It is a first-class area spec, not a
   folded concern. It owns the cross-probe MODEL (the N-way
   identity-resolution and per-pane emphasis-broadcast coordination over
   one resolved `DesignModel`); it does NOT duplicate the mechanism, which
   stays in its homes: the selection IDENTITY and same-net resolution over
   one `DesignModel` are owned by `GUI_INTERACTION_GRAMMAR.md`; the
   emphasis highlight/dim rendering (incl. the PCB-canvas net-highlight
   half) is in `GUI_CANVAS_AND_RENDERING.md`; view composition
   (tabs/PiP/split/pinned per 000B) is owned by
   `GUI_INFORMATION_ARCHITECTURE.md`. The cross-probe spec is the
   coordination layer over those primitives — it is the SEVENTH area spec
   (matches/beats Altium Cross Select Mode + cross-probe/jump by
   construction, not by tuning).

One responsibility the earlier draft listed as its own area file is NOT a
separate file; it is FOLDED INTO the six domain areas above (items 1–6)
so no orphaned spec is referenced (cross-probe, by contrast, is PROMOTED
to its own item-7 area spec, not folded):
- Live production in-canvas: zone-fill honesty and CAM projection
  rendering are in `GUI_CANVAS_AND_RENDERING.md`; the projection-check
  feedback is in `GUI_LIVE_FEEDBACK_AND_RULES.md`.

The deterministic-engine-backed differentiator (3.2 #3) is not a
separate area; it is the commit/journal constraint (4.2) that every area
inherits.

---

## 6. The Buildability Standard

A surface spec is not adjectives. To be accepted into the workflow, each
area spec MUST produce all five of the following, per surface:

### 6.1 Scene-contract schema extensions

Concrete typed extensions to the `gui-protocol` scene contract, in the
shape of the existing `BoardReviewSceneV1` (`crates/gui-protocol/src/lib.rs`):
named fields, units (`nm` for geometry), explicit draw order, and the
§2.5 identity triple (`object_id`, `object_kind`, `source_object_uuid`)
on every renderable. New companion primitives (e.g. ghost/diff
primitives for the AI canvas, CAM projection primitives) follow the same
determinism rules: versioned, byte-stable ordering and identity-stable
on unchanged persisted state. No new field may introduce design
authority into the GUI.

### 6.2 Per-tool interaction state machines

States and transitions, not prose. Each tool (select, move, route, draw,
zone, place, propose-review, cross-probe) is specified as an explicit
state machine: enumerated states, the input events that transition
between them, the consumer-state mutated on each transition, the single
transition that constructs an `OperationBatch` and calls `commit()`, and
the cancel transition that leaves zero committed mutation and zero dirty
shard. Selection/hover/drag/route-in-progress live entirely inside these
state machines as consumer state; only the commit transition crosses
into the journal.

### 6.3 Panel / component specs with context rules

Each panel is specified with: its content per selection class (the
context rule — e.g. Properties shows pad fields when a pad is selected,
net-class fields when a net is selected, matching Altium's
selection-driven Properties panel), validate-before-commit behavior with
the error reported at the edited field (`QG-DIRECT-EDITING-FEEL`), and
the commit path each editable field takes. Read-only supervision panels
state the committed-state source they reflect.

### 6.4 Explicit proof slices

A narrow, demonstrable path per area, in the voice of the Decision 002 /
012 first-proof-slice sections: the smallest end-to-end demonstration
that the surface renders real committed state (supervision) and — once
interactive — commits through the journaled path with durable undo. Each
proof slice names the fixture (default: the `datum-test` regression
fixture) and the gates it must pass.

### 6.5 Visual-golden acceptance

Each surface names its golden(s) and interaction test in the
`gui-render` harness, the fixture/scene that drives it, and the
acceptance condition (exact or tolerance diff). A surface without a
golden that renders real committed state is not accepted.

---

## 7. Integration & Sequencing

### 7.1 Order

Supervision-reflection first, then per-domain interactivity (Decision
013). Within each of the six domain areas, the read-only reflection
surface ships before any interactive tool for that domain; cross-probe
(area 7) likewise ships read-only first into the supervision panel (see
`GUI_CROSS_PROBE.md`) before any interactive jump/push verb. The
cross-area order:

1. Supervision-reflection (`GUI_SUPERVISION_REFLECTION.md`) — the
   instrument panel for committed `DesignModel`, checks, journal,
   sessions. Unlocks human audit.
2. Interaction grammar + canvas + panels + rules
   (`GUI_INTERACTION_GRAMMAR.md`, `GUI_CANVAS_AND_RENDERING.md`,
   `GUI_INFORMATION_ARCHITECTURE.md`, `GUI_LIVE_FEEDBACK_AND_RULES.md`) —
   table-stakes editing feel, context panels, and online DRC, each gated
   by `QG-DIRECT-EDITING-FEEL`.
3. The AI-native ghost/diff canvas (`GUI_AI_SURFACES.md`) — DIFFERENTIATOR
   1, given the deepest design attention. The unified-model cross-probe
   (DIFFERENTIATOR 2) is owned by its dedicated coordinating spec
   `GUI_CROSS_PROBE.md`, coordinating the selection/emphasis/composition
   mechanism the area-2/area-3 specs own.
4. Live production in-canvas — built on the engine's existing live-CAM
   projection/equivalence path (000B); rendered by
   `GUI_CANVAS_AND_RENDERING.md`, fed by `GUI_LIVE_FEEDBACK_AND_RULES.md`.

### 7.2 How the standard workflow consumes each area

Each area spec yields a set of WORKFLOW TASKS, one per surface, where a
task's definition-of-done is the area's proof slice plus its visual
golden(s). The workflow:
- reads the area spec's proof slices and goldens as the task list
- treats a surface as done only when its golden renders real committed
  state and its interaction test passes
- never accepts a surface on prose; the buildability standard (§6) is
  the contract

### 7.3 PROGRESS / SPEC_PARITY rows (added later, not now)

GUI rows belong in `specs/PROGRESS.md` and `specs/SPEC_PARITY.md` once
the GUI work is scheduled and the codebase author (Codex) is clear of
the GUI crates. This master and the area specs do NOT edit those files.
When the workflow schedules a GUI area, it adds: a PROGRESS row per
surface keyed to its proof slice, and a SPEC_PARITY inventory row for
the new scene-contract extensions and goldens. Until then, the area
proof slices and goldens are the authority on GUI status.

---

## 8. Retiring the "M7 Review Spike" Framing

`specs/M7_FRONTEND_SPEC.md` framed the GUI as a single bounded
read-only route-proposal review spike that "must not become a general
GUI milestone" (M7_FRONTEND_SPEC §Purpose, §1.3, §3). That framing is
RETIRED. It was correct as an opening probe and its technical output —
the `BoardReviewSceneV1` scene contract, the visual-regression harness,
the three-column shell, the identity triple — is retained and is the
substrate this spec builds on.

What changes:
- The GUI is no longer scoped as one read-only review surface. It is the
  full GUI per Decisions 002 / 012 / 013, benchmarked to commercial EDA.
- Supervision-reflection (Decision 013) replaces "read-only review
  spike" as the first deliverable: it is read-only by design, but it
  covers all committed-state surfaces, not just route-proposal review.
- M7_FRONTEND_SPEC §1.7 / §1.8 "apply/commit from canvas/terminal: Not
  supported" is superseded: the sanctioned journaled-CLI terminal-prefill
  conduit and the Outputs lane already exceed it (PRODUCT_MECHANICS_001 /
  005; conformance audit GUI Substrate section). Interactive commit is a
  named later phase here, not a forbidden one.
- The "must not become a general GUI milestone" guardrail is lifted; the
  general GUI is now the explicit goal, sequenced per §7.

M7_FRONTEND_SPEC remains valid as the historical record of the opening
slice and as the source of the still-correct scene-contract and
identity rules. Where it conflicts with this master on scope, this
master governs.

---

## 9. Non-Goals

This master does not require:
- copying any commercial tool's exact interface or matching every
  feature on day one (Decision 012 non-goals); we match the BAR, not the
  pixels
- interactive GUI editing during the substrate phase (Decision 013)
- arming `check_gui_parity` before the interactive editor phase begins
- a 3D viewer, STEP/IDF/ODB++/IPC-2581 export surfaces, or panelization
  UI as part of the first GUI areas (deferred per CLAUDE.md / M8)
- live real-time recomputation of every projection on every keystroke
  (Decision 002); stale-status is an acceptable interim state
- multi-monitor, floating-window, and full workbench-profile persistence
  before the core surfaces are credible (002 / 012)
- GUI surfaces for internal plumbing that is not user-facing
  (Decision 013 parity scope)
- replacing external CAM viewers entirely (000B non-goals)
- editing `specs/PROGRESS.md`, `specs/SPEC_PARITY.md`, `crates/`, or
  `mcp-server` from this spec authoring track

---

## 10. Open Owner Questions

1. Dark-first vs. equal dual-theme launch commitment for the canvas and
   panels (left open by M7_FRONTEND_SPEC §1.8) — does the commercial bar
   require a polished light theme at first GUI release, or is dark-first
   acceptable?
2. Which committed-state surfaces are MANDATORY for the first
   supervision-reflection deliverable, and which may be owner-waived to a
   later increment (Decision 013 OQ3)?
3. What is the minimum visual-golden coverage per surface that counts as
   a MEANINGFUL build-back, so a surface cannot pass on a shallow stub
   (Decision 013 OQ4)?
4. The exact arming trigger for the `check_gui_parity` fuse — first
   commit of the interactive editor phase, a milestone marker, or an
   explicit owner switch (Decision 013 OQ1)?
5. Which high-risk GUI edits must be PROPOSALS rather than direct commits
   — relationship-state changes, destructive deletes, imported-geometry
   repair, standards deviations, batch edits (Decision 002 OQ6)?
6. For the AI-native canvas (differentiator 1): does an accepted ghost
   commit immediately, or always land in an undo-staged review batch the
   supervisor can revert as one transaction?
7. View-composition scope for the first cross-probe surface — split,
   PiP, floating, pinned sidecar, or saved workbench first (000B OQ2)?
8. First numeric latency budgets for pointer preview, commit,
   selection/inspector update, project open, and projection refresh on
   the `datum-test` fixture (Decision 012 OQ6) — these gate
   `QG-PERFORMANCE-LATENCY` for every interactive surface.
