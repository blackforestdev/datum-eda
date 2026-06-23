# Datum EDA GUI Area Spec: Supervision-Reflection

Status: draft GUI area specification, 2026-06-22, benchmarked to commercial
EDA. Controlling for its domain. Conforms to `specs/GUI_SPEC.md` (master) and
inherits its bar, thesis, architecture constraints, and buildability standard.
This is GUI area 1 of six; it is the FIRST GUI deliverable per Decision 013.

Driven by:
- `docs/decisions/PRODUCT_MECHANICS_013_GUI_SUPERVISION_AND_PARITY.md`
  (the two-parity-levels ruling; supervision-reflection is required near-term)
- `docs/decisions/PRODUCT_MECHANICS_000B_VIEW_COMPOSITION_AND_LIVE_PRODUCTION.md`
- `docs/decisions/PRODUCT_MECHANICS_002_MANUAL_EDITOR_BASELINE.md`
- `docs/decisions/PRODUCT_MECHANICS_012_APPLICATION_QUALITY_BAR.md`
- `crates/gui-protocol` `BoardReviewSceneV1` scene contract
  (`crates/gui-protocol/src/lib.rs`, `check_runs.rs`)
- `crates/engine/src/substrate` — the resolved `DesignModel`
  (`substrate/mod.rs:358`), `ProjectResolver` (`project_resolver.rs`),
  `TransactionRecord` / `JournalCursor` / `CommitProvenance`,
  `DerivedRelationshipStatus`, `ResolveDiagnostic`
- `docs/audits/CODE_VS_SPEC_CONFORMANCE_AUDIT.md` (GUI Substrate section)

---

## 1. Purpose

Supervision-reflection is the supervisor's instrument panel: a READ-ONLY GUI
that visually displays committed engine and native state so a human can audit
what the headless engine, the CLI, and AI agents have done — before any
interactive editing exists. It is not an editor and has no edit affordances.

Decision 013 is explicit: a large headless push advanced engine capability
while the GUI stayed visually unchanged, so the supervisor could only audit
through text, files, and machine interfaces. Reflecting committed state is
NOT the same work as interactive authoring, and it must not be deferred
alongside it. This spec defines, concretely and buildably, exactly what is
rendered and how each rendered thing maps to a field of the resolved
`DesignModel`.

The thesis applies here unusually directly. Supervision-reflection is the
ground floor of all three differentiators:
- it renders the **deterministic engine-backed** model state every visual
  action will later mutate — same `DesignModel`, same identity, same revision;
- it is the surface on which the **AI-native canvas** (area 4) will later
  paint ghosts, so it must already render the committed geometry those ghosts
  diff against, and must already distinguish AI-authored commits from human
  ones in the journal;
- it renders the **unified-model** in one place (objects, relationships,
  variants, checks, production) so cross-probe (area 5) has a single resolved
  identity to select across.

This area is held to the commercial bar exactly like the rest: it is not "good
enough" because it renders something; it is good enough when a supervisor would
not reach for Altium's panels, Allegro's Constraint Manager, or a text dump to
answer "what is the current state and who changed it."

---

## 2. Commercial Benchmark + Match-vs-Exceed

### 2.1 The named patterns we benchmark

| Reflection surface | Commercial pattern benchmarked | Tool |
|---|---|---|
| Committed-geometry canvas | PCB editor canvas + layer manager (LAYERS panel, color/visibility) | Altium Designer |
| Object/structure browser | PCB panel / Navigator object lists; net + component browsers | Altium |
| Properties (read-only inspect) | Properties panel populated by selection, read-only when nothing is editable | Altium |
| Findings navigator | online-DRC Messages panel / rule-violation navigator, click-to-locate | Altium online DRC |
| Constraint/relationship reflection | Constraint Manager spreadsheet view of resolved constraint + status | Cadence Allegro |
| Manufacturing projection beside layout | CAM/Gerber preview, output job state | Altium Output Job / external CAM |
| Variant reflection | assembly-variant fitted/unfitted state coloring | Altium variant view |
| Activity / who-changed-what | Comments/ECO history; design-state audit | Altium / Allegro ECO |

### 2.2 Where we MATCH (table stakes — correctness + fluency, not novelty)

- A precise, smooth committed-geometry canvas with a layer manager and HUD
  cursor readout (matches the Altium PCB canvas + LAYERS panel + HUD). KiCad's
  board canvas is the FLOOR; we must at minimum equal its rendering fidelity
  on the `datum-test` fixture before this surface is accepted, and we are
  measured against Altium.
- A selection-driven inspector that shows the resolved fields of the selected
  object, read-only in this phase (matches the Altium Properties panel's
  context-by-selection behavior, minus editing).
- A findings navigator with click-to-locate cross-highlight onto the canvas
  (matches the Altium online-DRC Messages panel).

### 2.3 Where we EXCEED (the wedge)

- **One resolved model, not a join.** Every reflection surface reads the SAME
  `DesignModel` at one `model_revision`. Altium's panels read across the
  schematic and PCB documents and reconcile by refdes/net name; our object
  browser, inspector, findings, constraints, variants, and production
  projections are all projections of one resolved identity space
  (`object_id` / `source_object_uuid`). The supervisor never asks "are these
  two panels showing the same revision?" — they cannot disagree by
  construction.
- **Provenance-honest activity.** Because every mutation is a journaled
  `TransactionRecord` carrying `CommitProvenance { actor, source, reason }`
  with `source ∈ {manual, cli, tool, assistant}`, the activity panel shows a
  truthful, replayable history of WHO (human / CLI / AI agent / tool) changed
  WHAT and WHY — keyed to the exact `before/after_model_revision`. No
  commercial tool exposes an agent-vs-human commit ledger because no
  commercial tool routes every edit through one journaled `commit()`. This is
  the supervision unlock Decision 013 asks for, and it is the seed of the
  AI-native review surface.
- **Revision-honest staleness.** Derived projections (checks, zone fills, CAM)
  carry the `model_revision` they were computed against. When that differs from
  the live `DesignModel.model_revision`, the surface renders a STALE badge
  rather than silently showing old data (000B live-production honesty; Decision
  002 allows stale-status as an acceptable interim state). Commercial tools
  show stale CAM without flagging it; we make staleness a first-class visual.
- **Recovery instead of false truth.** A split or incoherent project opens in
  resolver-recovery mode showing `DesignModel.diagnostics`, never as accepted
  geometry (`QG-RESOLVER-RECOVERY`). The supervisor audits the failure rather
  than auditing a lie.

---

## 3. What Is Rendered, and Its Engine Mapping

This is the contract: every reflection surface maps to a named field of the
resolved `DesignModel` (`crates/engine/src/substrate/mod.rs:358`). The GUI
renders ONLY from the scene contract derived from these; it reads no source
shard directly (master §4.1).

| # | Reflection surface | DesignModel source | Scene-contract carrier |
|---|---|---|---|
| R1 | Committed board geometry (pads, tracks, vias, zones, outline, component bodies, board/footprint graphics + text) | `objects`, `component_instances` | `BoardReviewSceneV1` (existing primitives) |
| R2 | Native authored geometry (authored-from-scratch boards) | same `objects`/`component_instances`, native shards | same scene; no special-casing imported vs native |
| R3 | Zone-fill projection (filled copper vs. outline only) | `zone_fills` | `ZonePrimitive` + fill state (see §4.2) |
| R4 | Unrouted ratsnest | derived net graph | `unrouted_primitives`, `net_display` (existing) |
| R5 | Object / structure browser | `objects`, `component_instances`, nets | `SupervisionObjectIndexV1` (new, §4.1) |
| R6 | Read-only inspector (selected object's resolved fields) | the selected `DomainObject` / `ComponentInstance` | scene primitive fields + `SupervisionObjectIndexV1` |
| R7 | Relationship / constraint reflection (SCH↔PCB implement status) | `relationships`, `relationship_statuses` (`DerivedRelationshipStatus`) | `SupervisionRelationshipReflectionV1` (new, §4.3) |
| R8 | Variant reflection (fitted/unfitted per active variant) | `variants`, `variant_populations` (`DerivedVariantPopulation`) | `SupervisionVariantReflectionV1` (new, §4.4) |
| R9 | Check findings overlay + navigator | `check_runs` (→ `CheckRunReviewState`, `CheckFindingSummary`) | existing `check_runs.rs` + overlay (§4.5) |
| R10 | Manufacturing projection (output jobs, artifacts, plans, panels) | `output_jobs`, `output_job_runs`, `artifact_runs`, `manufacturing_plans`, `panel_projections`, `artifact_metadata` | existing `ProductionStatus` (`lib.rs:621`) |
| R11 | Proposals (pending agent/check proposals, read-only) | `proposals` | `SupervisionProposalIndexV1` (new, §4.6); FULL review UX is area 4 |
| R12 | Journal / activity ledger (who changed what, when, why) | `journal` (`Vec<TransactionRecord>`), `journal_cursor` | `SupervisionJournalReflectionV1` (new, §4.7) |
| R13 | Resolver diagnostics / recovery state | `diagnostics` (`Vec<ResolveDiagnostic>`) | `SupervisionResolverStatusV1` (new, §4.8) |
| R14 | Model revision + freshness banner | `model_revision`, per-projection computed revision | `SupervisionModelStampV1` (new, §4.9) |

Non-renderable internal plumbing (e.g. `source_shards` raw bytes,
`import_map` internals) is OUT of scope: Decision 013 limits parity to
user-facing capability. `import_map`/`source_shards` surface only as a count
and a coherence indicator inside R13, never as a browsable artifact here.

---

## 4. Scene-Contract Schema Extensions

All new types follow the existing `BoardReviewSceneV1` determinism rules
(`crates/gui-protocol/src/lib.rs`): geometry in `nm` (`PointNm`/`RectNm`),
the §2.5 identity triple (`object_id`, `object_kind`, `source_object_uuid`) on
every renderable, byte-stable ordering, identity-stable on unchanged persisted
state, versioned, and NO new field that introduces design authority into the
GUI. Every new field is a read-only projection of a `DesignModel` field named
in §3. These are companion contracts attached to the scene, not edits to the
existing primitives (so area 1 does not perturb the four checked-in goldens).

> IDENTITY-FIELD SEAM (buildability constraint, not a code change). The
> existing `gui-protocol` primitives are NOT yet uniform on the third tuple
> field: several renderables carry `source_object_uuid`
> (`lib.rs:99/216/285/303/317/353/365/379`) while the board/footprint TEXT
> primitives carry `text_uuid` (`lib.rs:112/137`). The conformance audit GUI
> section already flags `text_uuid → source_object_uuid` as the canonical name.
> The new §4 companion types MUST use `source_object_uuid` (the canonical
> spelling) so the supervision index is internally uniform. Where a companion
> references a text primitive, the renderer maps `text_uuid` ↔
> `source_object_uuid` at projection time. This spec does NOT rename the code
> field (that is a crate change owned by Codex, §9); it pins the contract field
> name and records the bridge so cross-highlight on text objects is buildable
> on day one rather than discovered as a mismatch in the golden run.

> Naming note (RESOLVED 2026-06-22): the master `specs/GUI_SPEC.md` §5 now
> indexes this file at its authored path `GUI_SUPERVISION_REFLECTION.md` (an
> earlier master draft used `GUI_SUPERVISION_REFLECTION_SPEC.md`). The
> canonical area-1 path is `specs/gui/GUI_SUPERVISION_REFLECTION.md`.

### 4.1 `SupervisionObjectIndexV1` (R5/R6)

A flat, sorted index over the resolved model for the browser + inspector. It
carries no geometry (the canvas already has that via R1); it is the audit list.

```
struct SupervisionObjectIndexV1 {
    contract: String,            // "supervision_object_index_v1"
    version: u32,                // 1
    model_revision: String,      // DesignModel.model_revision
    entries: Vec<SupervisionObjectEntry>,   // sorted by (kind, object_id)
}

struct SupervisionObjectEntry {
    object_id: String,           // identity triple
    object_kind: String,         // "component" | "pad" | "track" | "via"
                                 //  | "zone" | "net" | "board_graphic" | ...
    source_object_uuid: String,
    domain: String,              // "schematic" | "board" | "shared"
    display_label: String,       // e.g. refdes "R12", net "GND", pad "R12.1"
    summary_fields: Vec<SupervisionField>,  // resolved read-only fields
    relationship_ref: Option<String>,  // -> SupervisionRelationshipEntry.id
    variant_ref: Option<String>,        // -> SupervisionVariantEntry.id
    finding_refs: Vec<String>,          // -> CheckFindingSummary.finding_id
}

struct SupervisionField {
    key: String,                 // "net", "layer", "width_nm", "value", ...
    value: String,               // pre-formatted display value
    unit: Option<String>,        // "nm", "deg", ... when numeric
}
```

The inspector (R6) renders `summary_fields` for the selected entry. It is
READ-ONLY this phase: no field is editable, no commit path exists from here.
That distinction is the whole point of Decision 013's level-1 parity.

### 4.2 Zone-fill projection state (R3)

Extend the rendered zone projection (NOT the persisted `ZonePrimitive`) with a
fill-state companion so the canvas can render filled copper honestly vs.
outline-only, with staleness:

```
struct SupervisionZoneFillStateV1 {
    contract: String,            // "supervision_zone_fill_state_v1"
    version: u32,                // 1
    zone_object_id: String,      // -> ZonePrimitive.object_id
    fill_state: String,          // "filled" | "outline_only" | "stale"
    filled_polygons: Vec<Vec<PointNm>>,  // empty unless "filled"
    computed_model_revision: Option<String>, // revision the fill was solved at
}
```

`fill_state == "stale"` when `computed_model_revision != model_revision`. The
canvas renders stale fills hatched/dimmed with a STALE badge
(`QG-ZONEFILL-HONESTY`, 000B). This is a reflection of `DesignModel.zone_fills`
only; the GUI never solves a fill.

### 4.3 `SupervisionRelationshipReflectionV1` (R7)

Reflects `relationships` + `relationship_statuses`. This is the
Constraint-Manager-class surface: a sorted table of resolved relationship
state, beating Allegro by reading one resolved status enum rather than a
two-document reconciliation.

```
struct SupervisionRelationshipReflectionV1 {
    contract: String,            // "supervision_relationship_reflection_v1"
    version: u32,                // 1
    model_revision: String,
    entries: Vec<SupervisionRelationshipEntry>,  // sorted by object_id
}

struct SupervisionRelationshipEntry {
    id: String,                  // relationship object_id
    schematic_object_id: Option<String>,
    board_object_id: Option<String>,
    status: String,              // "implemented" | "pending_implementation"
                                 //  | "unresolved_mismatch"
                                 // (DerivedRelationshipStatus, snake_case)
    detail: String,              // human-readable status explanation
}
```

`unresolved_mismatch` rows render in the alert color and are click-to-locate
on both projections (cross-highlight seam shared with area 5).

### 4.4 `SupervisionVariantReflectionV1` (R8)

```
struct SupervisionVariantReflectionV1 {
    contract: String,            // "supervision_variant_reflection_v1"
    version: u32,                // 1
    model_revision: String,
    active_variant: Option<String>,   // selected variant object_id, if any
    available_variants: Vec<String>,  // selectable (consumer state) variants
    entries: Vec<SupervisionVariantEntry>,
}

struct SupervisionVariantEntry {
    component_object_id: String,
    population: String,          // "applicable" | "not_applicable_for_variant"
                                 // (DerivedVariantPopulation, snake_case)
    fitted: bool,                // resolved fitted/unfitted
}
```

Selecting `active_variant` is CONSUMER STATE (it changes which projection of
the resolved model is shown; it is never journaled and never a commit). The
canvas renders unfitted components dimmed/ghosted (matches Altium variant
view). The active variant choice is view state, mirroring the layer-visibility
toggles already in `WorkspaceFilterState`.

### 4.5 Findings overlay (R9)

Reuse the existing `CheckRunReviewState` / `CheckFindingSummary`
(`check_runs.rs`). Add a thin canvas-overlay companion mapping a finding to a
locatable highlight on the committed geometry:

```
struct SupervisionFindingOverlayV1 {
    contract: String,            // "supervision_finding_overlay_v1"
    version: u32,                // 1
    finding_id: String,          // -> CheckFindingSummary.finding_id
    severity: String,            // "error" | "warning" | "info"
    target_object_ids: Vec<String>,  // objects to highlight on locate
    marker_points: Vec<PointNm>,     // pin marker(s) in board space
}
```

Findings carry the `model_revision` of their `CheckRun`; if it differs from
the live model the navigator shows a STALE badge (§4.9).

### 4.6 `SupervisionProposalIndexV1` (R11)

A READ-ONLY list of pending `proposals` so the supervisor can see that
proposals exist and their headline status. The full visual accept/reject ghost
UX is area 4 (AI-native canvas); area 1 only reflects existence + status.

```
struct SupervisionProposalIndexV1 {
    contract: String,            // "supervision_proposal_index_v1"
    version: u32,                // 1
    model_revision: String,
    entries: Vec<SupervisionProposalEntry>,
}

struct SupervisionProposalEntry {
    proposal_id: String,
    source: String,              // EXHAUSTIVE over ProposalSource
                                 //   (proposal.rs:29): "manual" | "cli" | "tool"
                                 //   | "assistant" | "check" | "import"
    status: String,              // EXHAUSTIVE over ProposalStatus
                                 //   (proposal.rs:19): "draft" | "accepted"
                                 //   | "deferred" | "rejected" | "applied"
    summary: String,
    target_object_ids: Vec<String>,
}
```

Both enums are reflected verbatim from the engine (snake_case serde). The
status accent rule is total over `ProposalStatus`: `draft`/`accepted`/`deferred`
render as actionable-later (muted), `applied` as historical (neutral),
`rejected` as struck-through. `source == "assistant"` shares the AI accent with
the journal ledger so the same agent reads identically across both panels (the
unified-identity wedge — one actor, one accent, every surface).

### 4.7 `SupervisionJournalReflectionV1` (R12)

The provenance-honest activity ledger — the differentiating supervision
surface. A reflection of `journal: Vec<TransactionRecord>` and
`journal_cursor`.

```
struct SupervisionJournalReflectionV1 {
    contract: String,            // "supervision_journal_reflection_v1"
    version: u32,                // 1
    model_revision: String,
    applied_transaction_count: usize,  // JournalCursor
    entries: Vec<SupervisionJournalEntry>,   // newest-last, journal order
}

struct SupervisionJournalEntry {
    transaction_id: String,      // TransactionRecord.transaction_id (Uuid)
    batch_id: String,            // TransactionRecord.batch_id (Uuid)
    transaction_kind: String,    // "normal" | "undo" | "redo"
                                 //   (TransactionKind, snake_case — exhaustive
                                 //    over the real enum; not free-form)
    undo_of: Option<String>,     // TransactionRecord.undo_of (Uuid), if Undo
    redo_of: Option<String>,     // TransactionRecord.redo_of (Uuid), if Redo
    actor: String,               // CommitProvenance.actor
    source: String,              // "manual" | "cli" | "test" | "tool"
                                 //   | "assistant" — EXHAUSTIVE over CommitSource
                                 //   (substrate/mod.rs:288). `test` is real and
                                 //   MUST render; goldens are committed with
                                 //   CommitSource::Test, so a panel that drops
                                 //   `test` would fail its own golden.
    reason: String,              // CommitProvenance.reason
    before_model_revision: String,
    after_model_revision: String,
    created_object_ids: Vec<String>,   // CommitDiff.created
    modified_object_ids: Vec<String>,  // CommitDiff.modified
    deleted_object_ids: Vec<String>,   // CommitDiff.deleted
    applied: bool,               // index < applied_transaction_count
}
```

Source → ledger-class mapping (exhaustive; the panel's accent rule is a total
function over `CommitSource`, so a new engine variant forces a spec amendment
rather than silently falling into a default bucket):

| `CommitSource` | Ledger class | Visual |
|---|---|---|
| `manual` | human | neutral glyph, no accent |
| `cli` | human-scripted | terminal glyph, no accent |
| `test` | synthetic | muted/secondary glyph (fixtures only) |
| `tool` | machine-agent | agent accent (cool) |
| `assistant` | AI-agent | AI accent (warm) + agent glyph |

`source == "assistant"` (and `"tool"`) rows are visually distinguished (AI
glyph + accent) from `"manual"`/`"cli"` rows — the human-vs-agent ledger.
`transaction_kind ∈ {undo, redo}` rows render with an undo/redo chevron and
link to their `undo_of`/`redo_of` origin transaction (replay-honest history —
Altium/Allegro ECO history shows applied state but not a typed undo lineage).
Selecting a journal entry cross-highlights its `created/modified/deleted`
objects on the canvas (consumer state). This entry's
`created/modified/deleted` set is precisely what area 4 will replay as a ghost
diff, so the contract is shared.

### 4.8 `SupervisionResolverStatusV1` (R13)

```
struct SupervisionResolverStatusV1 {
    contract: String,            // "supervision_resolver_status_v1"
    version: u32,                // 1
    mode: String,                // "resolved" | "recovery"
    model_revision: Option<String>,  // None in recovery
    shard_count: usize,
    coherent: bool,
    diagnostics: Vec<SupervisionDiagnosticEntry>,
}

struct SupervisionDiagnosticEntry {
    code: String,                // ResolveDiagnostic.code
    message: String,             // ResolveDiagnostic.message
    path: Option<String>,        // ResolveDiagnostic.path (PathBuf -> display)
    severity: String,            // "error" | "warning" — GUI CLASSIFICATION,
                                 //   not an engine field: ResolveDiagnostic has
                                 //   no severity today (substrate/mod.rs:344).
                                 //   Classified by a named code-prefix table
                                 //   (below) so recovery promotion is golden-
                                 //   stable; see OQ9.
}
```

Severity classification is a total, golden-stable mapping from
`ResolveDiagnostic.code` to `{error, warning}`. Until the engine carries
severity natively (OQ9), the GUI owns a checked-in classification table keyed by
diagnostic code prefix; an unknown code defaults to `error` (fail-safe: an
unclassified diagnostic forces recovery rather than silently downgrading to a
warning the supervisor might ignore). The table is part of the
`supervision_resolver_recovery` golden's fixture so any new diagnostic code that
lands unclassified is caught by the golden.

Derivation (none of these are new engine fields — all are projections of the
already-resolved model): `mode = if diagnostics has any severity=="error" then
"recovery" else "resolved"` (the GUI classifies; the engine does not carry a
mode); `shard_count = source_shards.len()`; `coherent = mode == "resolved"`;
`model_revision = Some(live)` in resolved mode, `None` in recovery;
`diagnostics` reflects `DesignModel.diagnostics` (`ResolveDiagnostic`, whose
`path` is a `PathBuf` rendered as a display string, never a browsable shard —
§9 plumbing exclusion). The error/warning split is a GUI classification of
`ResolveDiagnostic` (the engine type has no severity field today); the
classification rule is named here so the recovery promotion is deterministic
and golden-stable, and OQ flags whether severity should instead be promoted
into the engine type (a Codex-owned code change, not done here).

`mode == "recovery"` forces the whole window into recovery layout: the canvas
is suppressed (no false geometry), the diagnostics list is primary
(`QG-RESOLVER-RECOVERY`). This is the "audit the failure, not a lie" guarantee.

### 4.9 `SupervisionModelStampV1` (R14)

The freshness banner. One line that makes the audited revision unambiguous and
every derived projection's staleness explicit.

```
struct SupervisionModelStampV1 {
    contract: String,            // "supervision_model_stamp_v1"
    version: u32,                // 1
    model_revision: String,      // live DesignModel.model_revision
    projection_freshness: Vec<SupervisionProjectionFreshness>,
}

struct SupervisionProjectionFreshness {
    projection: String,          // "zone_fill" | "checks" | "cam" | "ratsnest"
    computed_model_revision: Option<String>,
    fresh: bool,                 // computed == live
}
```

---

## 5. Interaction State Machine (read-only audit navigation)

Supervision-reflection has NO commit transition by definition. Its single
state machine is the audit-navigation machine: every input mutates only
CONSUMER STATE (selection, hover, camera, active-view filters), and the only
journal interaction is READING `journal` to render R12. Selection/hover/camera
are consumer state, never operations, never journaled (master §4.2; 000B).

This extends the existing `LiveDesignSession` / `SessionCommand` /
`SessionEvent` model (`lib.rs:675`). To keep the machine BUILDABLE (not
aspirational), every abstract event below binds to EITHER an existing
`SessionCommand` variant or a NAMED net-new one that this spec proposes for the
area-1 increment. The existing surface today is exactly:
`SelectReviewAction`, `SelectAuthoredObject`, `ClearSelection`,
`SelectPreviousReviewAction`, `SelectNextReviewAction`, `ToggleShowAuthored`,
`ToggleShowProposed`, `ToggleShowUnrouted`, `ToggleDimUnrelated`,
`ToggleLayerVisibility`, `FocusProductionArtifact[File]`, and the
artifact-preview zoom/pan/reset/toggle commands (`lib.rs:675–694`); emitted
events are exactly `SelectionChanged(SelectionTarget)`, `SceneChanged`,
`FrameChanged` (`lib.rs:697–701`).

Event → `SessionCommand` binding (E = exists today; N = net-new for area 1):

| Abstract event | Concrete `SessionCommand` | Status |
|---|---|---|
| `SelectObject(id)` | `SelectAuthoredObject(id)` | E |
| `ClearSelection` | `ClearSelection` | E |
| `ToggleLayerVisibility(l)` | `ToggleLayerVisibility(l)` | E |
| `ToggleShow*` / `ToggleDimUnrelated` | same | E |
| `FocusArtifact` | `FocusProductionArtifact[File]` | E |
| `PointerMove(p)` / `PointerExit` | `HoverObject(Option<id>)` | N |
| `SelectFinding(fid)` | `SelectFinding(fid)` | N |
| `SelectJournalEntry(txid)` | `SelectJournalEntry(txid)` | N |
| `SelectRelationship(id)` | `SelectRelationship(id)` | N |
| `SetActiveVariant(v)` | `SetActiveVariant(Option<v>)` | N |
| `RefreshModel` | `RefreshModel` | N |

The net-new commands extend `SessionCommand` (and `SelectionTarget` gains
`Finding`/`JournalEntry`/`Relationship` arms) WITHOUT new events: each still
emits only the existing `SelectionChanged`/`FrameChanged`/`SceneChanged`. This
keeps the event surface — and therefore the daemon dispatch contract — stable
while the read-only navigation surface grows. None of the N commands carry an
`OperationBatch`; that is enforced by the §5 invariant below and PS-SR-2.

States:
- `Idle` — model rendered, nothing selected.
- `Hovering(object_id)` — pointer over a renderable; HUD shows its id/kind +
  key fields (Altium HUD parity).
- `Selected(target)` — an object/finding/journal-entry/relationship/proposal
  is selected; inspector + cross-highlight populated.
- `Recovery` — `resolver_status.mode == "recovery"`; canvas suppressed,
  diagnostics primary; only diagnostic navigation is allowed.
- `Stale(view)` — a projection's freshness is false; the view renders with a
  STALE badge but remains navigable.

Transitions (event → guard → consumer-state mutation → emitted `SessionEvent`):

Columns: From → Event (concrete `SessionCommand`) → Guard → consumer-state
mutation → resulting state → emitted `SessionEvent`. `hovered_object_id` and
`selection` live in `WorkspaceFilterState` (`lib.rs:613`); no row writes any
`DesignModel` field.

| From | Event (`SessionCommand`) | Guard | Consumer-state mutation | To | Emits |
|---|---|---|---|---|---|
| Idle/Selected | `HoverObject(Some id)` | hit-test resolves id; mode≠recovery | set `hovered_object_id = id` | Hovering | `FrameChanged` |
| Hovering | `HoverObject(None)` | — | clear `hovered_object_id` | Idle/Selected (prior) | `FrameChanged` |
| any (≠Recovery) | `SelectAuthoredObject(id)` | id ∈ object index | `selection = AuthoredObject(id)`; populate inspector; compute cross-highlight set | Selected | `SelectionChanged`+`FrameChanged` |
| any (≠Recovery) | `SelectFinding(fid)` | fid ∈ findings | `selection = Finding(fid)`; locate marker; highlight `target_object_ids` | Selected | `SelectionChanged`+`FrameChanged` |
| any (≠Recovery) | `SelectJournalEntry(txid)` | txid ∈ journal | `selection = JournalEntry(txid)`; highlight created/modified/deleted set | Selected | `SelectionChanged`+`FrameChanged` |
| any (≠Recovery) | `SelectRelationship(id)` | id ∈ relationships | `selection = Relationship(id)`; cross-highlight both projections | Selected | `SelectionChanged`+`FrameChanged` |
| any | `ClearSelection` | — | `selection = None`; clear highlight | Idle (or Recovery if mode=recovery) | `SelectionChanged`+`FrameChanged` |
| any (≠Recovery) | `ToggleLayerVisibility(l)` / `ToggleShow*` / `ToggleDimUnrelated` | — | update `WorkspaceFilterState` | unchanged | `FrameChanged` or `SceneChanged` per `layer_visibility_change_is_frame_only` |
| any (≠Recovery) | `SetActiveVariant(Some v)` | v ∈ available_variants | set active-variant view state; recompute fitted dimming | unchanged | `SceneChanged` |
| any | `RefreshModel` | — | re-resolve; recompute freshness stamps | Recovery if diagnostics else prior | `SceneChanged` |
| any | (internal) resolve produced diagnostics | — | enter recovery; suppress canvas; promote diagnostics | Recovery | `SceneChanged` |

`Stale(view)` is not a distinct selection state but a per-projection rendering
modifier derived from `SupervisionModelStampV1.projection_freshness` (§4.9): a
view is rendered with a STALE badge whenever its `fresh == false`, orthogonally
to Idle/Hovering/Selected. Recovery is the one state that disables the entire
left column of transitions (object/finding/journal/relationship/variant), per
`QG-RESOLVER-RECOVERY`; only diagnostic navigation and `RefreshModel` survive.

INVARIANT (gated, two-layer):
- TYPE-LEVEL: the area-1 `SessionCommand` arms (existing + the §5 net-new ones)
  carry NO `OperationBatch`, `Operation`, or commit-bearing payload. A
  supervision-only handler therefore has no value from which to construct a
  commit; the absence of a write path is structural, not merely a runtime
  discipline. Any future editing command is a SEPARATE arm introduced in
  areas 2/3, so it is reviewable as a diff to this enum.
- RUNTIME (gated by PS-SR-2): the full interaction test drives EVERY area-1
  command — including the net-new `HoverObject`/`SelectFinding`/
  `SelectJournalEntry`/`SelectRelationship`/`SetActiveVariant`/`RefreshModel` —
  and asserts `DesignModel.journal.len()` and `journal_cursor` are byte-identical
  before/after and every source-shard mtime is unchanged. The test fails if any
  transition touches `commit()` or mutates a shard.

Editing arrives only in areas 2/3 and is a NAMED later phase (Decision 013),
never reached from this machine.

---

## 6. Panel Specs (read-only; context rules)

Each supervision panel states the committed-state source it reflects (master
§6.3). All are read-only; "context rule" here means what the panel shows per
selection class, not an edit path.

### 6.1 Layer Manager panel
- Source: `BoardReviewSceneV1.layers` + `WorkspaceFilterState.layer_visibility`.
- Content: layer rows (id, name, kind, render order), per-layer visibility
  toggle (consumer state), color swatch. Matches Altium LAYERS panel.
- Context rule: none (always full layer list). Selecting an object flashes its
  layer row.

### 6.2 Object Browser panel
- Source: `SupervisionObjectIndexV1`.
- Content: filterable, sorted tree by kind → object. Counts per kind in the
  header (matches Altium PCB panel object counts).
- Context rule: clicking an entry → `SelectObject`; the canvas selection and
  the browser highlight stay in lockstep (one identity).

### 6.3 Inspector panel (read-only)
- Source: selected entry's `summary_fields` (R6).
- Content per selection class (the Altium Properties-panel context behavior,
  read-only):
  - pad → net, layer(s), shape, size, drill, clearances;
  - track → net, layer, width, length;
  - via → net, drill, diameter, layer span;
  - component → refdes, value, placement layer, position, rotation, variant
    fitted-state, relationship status;
  - net → name, pad/track/via counts, unrouted count;
  - zone → net, layer, fill state (R3);
  - finding → severity, code, message, explanation, targets;
  - nothing selected → empty state ("select an object to inspect").
- Read-only banner: every field is shown disabled with a note that interactive
  editing is the GUI editor phase (Decision 013). No commit path exists.

### 6.4 Relationship / Constraint panel
- Source: `SupervisionRelationshipReflectionV1`.
- Content: spreadsheet-class table (sch object, board object, status, detail).
  `unresolved_mismatch` rows alert-colored. Matches Allegro Constraint Manager
  layout; exceeds it by reading one resolved status enum.
- Context rule: selecting a row cross-highlights both projections.

### 6.5 Findings Navigator panel
- Source: `CheckRunReviewState.findings` + `SupervisionFindingOverlayV1`.
- Content: grouped by severity then domain/rule; profile basis + coverage
  header (`CheckRunProfileBasisSummary`, `CheckRunCoverageSummary`); STALE
  badge when the run's `model_revision` ≠ live. Matches Altium online-DRC
  Messages panel.
- Context rule: click finding → locate marker + highlight targets on canvas.

### 6.6 Activity / Journal panel
- Source: `SupervisionJournalReflectionV1` + `JournalCursor`.
- Content: reverse-chronological ledger; each row shows actor, source badge
  (human/CLI/AI/tool), reason, revision delta, and created/modified/deleted
  counts. AI/tool rows accented. Applied vs. beyond-cursor rows distinguished.
- Context rule: select row → cross-highlight its diff objects on canvas.

### 6.7 Manufacturing / Outputs panel
- Source: `ProductionStatus` (existing; `lib.rs:621`).
- Content: output jobs, artifact runs, manufacturing plans, panel projections,
  focused artifact preview (existing artifact-preview viewport). Per-projection
  freshness stamp (R14). Matches Altium Output Job state + CAM preview.
- Context rule: focus an artifact → preview viewport (existing
  `FocusProductionArtifact*` commands).

### 6.8 Model / Resolver status banner
- Source: `SupervisionModelStampV1` + `SupervisionResolverStatusV1`.
- Content: live `model_revision`, shard count/coherence, projection-freshness
  chips. In recovery mode this banner becomes the alert header and the
  diagnostics list is promoted to primary; canvas suppressed.

---

## 7. Proof Slices

Default fixture: the `datum-test` regression fixture
(`~/Documents/kicad_projects/Datum-eda/datum-test/`), per master §6.4 and the
M7 regression-fixture memory. Each slice is the smallest end-to-end
demonstration that the surface renders REAL committed state.

### PS-SR-1 — The supervision audit loop (the Decision 013 first step)
Author a change via CLI (a typed op through `commit()` on the `datum-test`
fixture), then open the GUI and confirm the committed change is visible:
geometry on the canvas (R1/R2), the new `TransactionRecord` in the Activity
panel (R12) with correct actor/source/reason, and the bumped `model_revision`
in the banner (R14). Gates: golden `supervision_audit_loop` renders the
post-commit state; interaction test confirms the journal row's
`after_model_revision` matches the banner.

### PS-SR-2 — Read-only invariant (no write path)
Drive the full §5 command set against the resolved `datum-test` model —
`HoverObject(Some/None)`, `SelectAuthoredObject`, `SelectFinding`,
`SelectJournalEntry`, `SelectRelationship`, every `Toggle*`, `SetActiveVariant`,
`ClearSelection`, `RefreshModel` — and assert across the entire session:
`DesignModel.journal.len()` and `journal_cursor.applied_transaction_count` are
unchanged, `model_revision` is unchanged (no re-solve mutated authored state),
and every source-shard mtime is unchanged. Gate: the test fails if any
transition touches `commit()`, constructs an `OperationBatch`, or writes a
shard. This is the runtime half of the §5 invariant.

### PS-SR-3 — Findings cross-highlight
With a fixture variant that produces ≥1 DRC finding, render the Findings
Navigator (R9), select a finding, and confirm the marker + target highlight
land on the correct committed geometry. Gate: golden
`supervision_findings_locate`.

### PS-SR-4 — Provenance honesty (human vs. agent ledger)
Build a fixture journal carrying at least one commit per `CommitSource` arm
(`Manual`, `Cli`, `Test`, `Tool`, `Assistant`); confirm the Activity panel
renders each with its §4.7 accent class — `assistant` (warm AI accent) and
`tool` (cool agent accent) visually distinct from `manual`/`cli` (human, no
accent) and `test` (muted) — each with the correct
`before/after_model_revision` delta. Assert the accent function is total over
`CommitSource` (a new engine arm with no accent rule fails the test, not
silently buckets). Gate: golden `supervision_activity_provenance`. This is the
proof of the wedge no commercial tool can show: a per-commit human-vs-AI-vs-CLI
ledger keyed to one journaled `commit()`.

### PS-SR-5 — Staleness honesty
Compute a zone fill at revision N, then commit a change advancing to N+1
WITHOUT re-filling; confirm the zone renders hatched/dimmed with a STALE badge
and the banner freshness chip for `zone_fill` reads stale (R3/R14). Gate:
golden `supervision_zonefill_stale`.

### PS-SR-6 — Resolver recovery
Open an intentionally incoherent/split project copy; confirm recovery mode:
canvas suppressed, diagnostics list primary, no geometry rendered as truth
(`QG-RESOLVER-RECOVERY`). Gate: golden `supervision_resolver_recovery`.

---

## 8. Visual-Golden Acceptance

Per master §6.5 and §4.4, each surface ships a golden + interaction test in
the `gui-render` harness (`visual_runner.rs`, `visual_manifest.rs`,
`visual_diff.rs`; bless + exact/tolerance diff). A surface is accepted only
when its golden renders REAL committed `datum-test` state and its interaction
test exercises the §5 machine. Minimum golden set for area 1:

| Golden | Surface(s) | Drives | Diff |
|---|---|---|---|
| `supervision_overview` | full window: canvas + all panels populated | `datum-test` resolved model | tolerance |
| `supervision_audit_loop` | canvas + Activity + banner post-CLI-commit | PS-SR-1 | tolerance |
| `supervision_inspect_pad` / `_component` / `_net` | inspector context rule | object selection | exact |
| `supervision_findings_locate` | Findings Navigator + canvas marker | PS-SR-3 | tolerance |
| `supervision_activity_provenance` | Activity panel: `manual` vs `cli` vs `test` vs `tool` vs `assistant` rows, each with its §4.7 accent class | PS-SR-4 (all five `CommitSource` arms present) | exact |
| `supervision_relationship_mismatch` | Relationship panel `unresolved_mismatch` | relationship fixture | exact |
| `supervision_variant_dimming` | canvas with unfitted components dimmed | active-variant view state | tolerance |
| `supervision_zonefill_stale` | stale zone hatch + freshness chip | PS-SR-5 | tolerance |
| `supervision_resolver_recovery` | recovery layout, canvas suppressed | PS-SR-6 | exact |

A shallow stub does not satisfy the rule (Decision 013 calibration): each
golden must render real resolved state, not a placeholder. The exact minimum
golden coverage per surface is Open Owner Question 3 (mirrors master OQ3 /
Decision 013 OQ4). The dormant `check_gui_parity` fuse does not arm for this
area (it arms at the interactive editor phase); area 1's definition-of-done is
the golden set above plus the §7 proof slices.

---

## 9. Non-Goals

- No interactive editing, no commit path, no edit affordances on any surface
  (Decision 013 level-1 parity; this is the entire boundary of area 1).
- No proposal accept/reject UX — area 1 reflects proposal existence/status
  only; the visual ghost/diff accept/reject is area 4 (AI-native canvas).
- No live real-time re-solve on every keystroke; stale-status is an acceptable
  interim state (Decision 002), made HONEST here, not eliminated.
- No browsable raw `source_shards` / `import_map` internals — internal
  plumbing is excluded from parity scope (Decision 013); only counts/coherence
  surface in R13.
- No new design authority in the GUI: every §4 type is a read-only projection
  of a named `DesignModel` field (master §4.1).
- No 3D viewer, STEP/IDF/ODB++/IPC-2581 export surfaces, or panelization UI
  (deferred per CLAUDE.md / M8); panel projections are reflected as summaries
  (R10), not authored.
- No edits to `specs/PROGRESS.md`, `specs/SPEC_PARITY.md`, `crates/`, or
  `mcp-server` from this spec authoring track. The `text_uuid` →
  `source_object_uuid` naming divergence flagged in the conformance audit GUI
  section is noted but NOT resolved here (it is a code change).

---

## 10. Open Owner Questions

1. Which of R1–R14 are MANDATORY for the first supervision-reflection
   deliverable, and which may be owner-waived to a later increment (Decision
   013 OQ3 / master OQ2)? Proposed mandatory floor: R1, R5, R6, R9, R12, R13,
   R14 (geometry + browse + inspect + findings + activity + diagnostics +
   stamp) — the minimum that makes the headless build supervisable.
2. Does supervision-reflection get its own lighter goldens-backed gate now, or
   is the per-surface definition-of-done sufficient until `check_gui_parity`
   arms at the editor phase (Decision 013 OQ2)?
3. What is the minimum golden coverage per surface that counts as a MEANINGFUL
   build-back, so a surface cannot pass on a shallow stub (Decision 013 OQ4 /
   master OQ3)?
4. View composition for the first deliverable — fixed three-column shell (the
   existing M7 shell) vs. dockable panels vs. split/PiP (000B OQ2). The
   inherited shell is the cheapest start; is dockable required at first
   release for the commercial bar?
5. Provenance display granularity: is `CommitSource` (manual/cli/tool/
   assistant) enough to distinguish human vs. AI in the Activity panel, or
   does the supervisor need the specific tool/agent identity surfaced from
   `CommitProvenance.actor`/`reason`?
6. Staleness policy: should opening a project with stale derived projections
   auto-trigger a re-solve, or always reflect-as-stale and leave the re-solve
   to an explicit CLI/agent action (Decision 002 stale-status interim)?
7. Dark-first vs. equal dual-theme for the supervision surfaces at first
   release (master OQ1).
8. First numeric latency budgets for project open, selection/inspector update,
   and projection-freshness recompute on `datum-test`
   (`QG-PERFORMANCE-LATENCY`; master OQ8). Proposed first targets to ratify:
   project open ≤ 500 ms, selection→inspector update ≤ 16 ms (one frame),
   freshness recompute on `RefreshModel` ≤ 100 ms, all on `datum-test` — a
   commercial-feeling instrument panel, not a batch report.
9. Should `ResolveDiagnostic` (and `CheckFindingSummary` resolver-class entries)
   carry a native `severity` field in the engine, retiring the GUI-side
   code-prefix classification table in §4.8? This is a Codex-owned engine
   change; until then the GUI owns the fail-safe (unknown ⇒ error ⇒ recovery)
   classification table. The owner should decide whether to promote severity
   into the canonical model so every consumer (CLI, MCP, GUI) classifies
   identically rather than each re-deriving it.
