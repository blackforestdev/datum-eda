# Datum EDA GUI — Live Feedback & Rules Specification

Status: draft GUI area specification, 2026-06-22, benchmarked to
commercial EDA. Controlling for its domain. Conforms to and is governed
by `specs/GUI_SPEC.md` (the master). This is area spec 3.b of the six —
the live-feedback / online-DRC / rules-editor surface that the canvas
(`specs/gui/GUI_CANVAS_AND_RENDERING.md`) and panels
(`specs/gui/GUI_INFORMATION_ARCHITECTURE.md`) areas consume.

Driven by:
- `specs/GUI_SPEC.md` (master: the bar, thesis, four architecture
  constraints, five-part buildability standard)
- `docs/decisions/PRODUCT_MECHANICS_009_RULES_CONSTRAINTS_CHECKS.md`
- `docs/decisions/PRODUCT_MECHANICS_002_MANUAL_EDITOR_BASELINE.md`
- `docs/decisions/PRODUCT_MECHANICS_012_APPLICATION_QUALITY_BAR.md`
- `docs/decisions/PRODUCT_MECHANICS_013_GUI_SUPERVISION_AND_PARITY.md`
- `docs/contracts/RULES_CHECKS_TOOL_CONTRACT.md`
- `specs/CHECKING_ARCHITECTURE_SPEC.md`
- Engine check model: `crates/engine/src/drc/mod.rs`
  (`DrcViolation`/`DrcLocation`/`DrcSummary`/`fingerprint`),
  `crates/engine/src/erc/mod.rs`, `crates/engine/src/api/check_summary.rs`
- Scene contract: `crates/gui-protocol/src/lib.rs`
  (`BoardReviewSceneV1`, `ProposalOverlayPrimitive`, `ReviewPrimitive`),
  `crates/gui-protocol/src/check_runs.rs`
  (`CheckRunReviewState`, `CheckFindingSummary`)
- Visual harness: `crates/gui-render/src/visual_runner.rs`,
  `visual_manifest.rs`, `visual_diff.rs`, `visual_capture.rs`

---

## 1. Purpose & Scope

This spec defines four cooperating surfaces:

1. **Findings overlay** — committed `CheckRun`/`CheckFinding` rendered as
   on-canvas violation markers, navigable from a Findings panel
   (supervision-reflection; ships first per Decision 013).
2. **Live-feedback layer** — clearance/violation feedback shown DURING an
   interactive edit (route, move, draw, zone), computed from the
   in-progress consumer-state geometry against the resolved rule set,
   BEFORE any `commit()`.
3. **Constraint-driven / correct-by-construction editing** — the
   feedback layer feeding the canvas tools so an interaction can be
   guided, snapped, or refused before it produces a violating
   `Operation`.
4. **Rules editor** — the Altium-style single-rule-object editor UX that
   authors the `Rule` rows the checks run against, each row a typed op
   through `commit()`.

It also reserves, but does not specify, the seam for interactive
push-shove and length tuning (§9 Non-Goals, §11 Reserved Seam).

In scope: the on-canvas presentation of findings; the live-feedback
state contract and the predicate that drives it; the rules-editor panel;
how each ties to `CheckRun`/`CheckFinding` and the journaled `commit()`.

Out of scope (owned elsewhere): the per-tool route/move/draw/zone state
machines themselves (`GUI_CANVAS_AND_RENDERING.md` — this spec adds
only the live-feedback OUTPUT each tool consumes); the engine rule
expression language and individual rule semantics (Decision 009 §"This
decision covers product mechanics, not the rule expression language");
the AI repair-proposal ghost surface (`GUI_AI_SURFACES.md` —
this spec hands findings to it, the ghost rendering lives there);
manufacturing-projection (CAM) RENDERING in-canvas, owned by
`GUI_CANVAS_AND_RENDERING.md` (the live-production responsibility is folded
there per master §5; this spec owns only the projection-check FINDINGS).

---

## 2. The Bar (commercial benchmark)

Per the master, KiCad is a FLOOR we exceed, never a target.

| Datum surface | Commercial pattern matched | Tool | Where we EXCEED |
|---|---|---|---|
| Findings overlay + navigator | Violations panel with per-finding drill-down, marker-on-canvas, navigate-to-violation | **Altium** Violations panel; **Cadence Allegro** DRC markers | Findings are revision-keyed `CheckFinding`s with deterministic fingerprints over one `DesignModel` — markers stay identity-stable across rename/move/re-import (Decision 009 fingerprint rule); Altium markers are positional and churn on edit |
| Live-feedback during routing | Online DRC: real-time clearance violation as you route; ratsnest/clearance "fence" | **Altium** online DRC + heads-up clearance; **Siemens Xpedition** sketch-routing live clearance/plow feedback | Live feedback is computed from in-progress CONSUMER STATE against the SAME engine predicate the batch `CheckRun` uses — no second "fast DRC" code path that disagrees with batch DRC (the classic Altium online-vs-batch mismatch); see §6.4 single-predicate rule |
| Correct-by-construction | Clearance-aware push/refuse; gloss/hug | **Xpedition** sketch routing; **Allegro** dynamic shape edit | Refusal is honest about WHY (the live predicate carries the would-be `CheckFinding` fingerprint), and the refused edit is never silently mutated — Datum never lands a half-shoved track as committed truth without a typed op |
| Rules editor | One PCB Rules & Constraints Editor: typed kind + scope + severity + priority, online-vs-batch as a MODE not a tool | **Altium** Rules & Constraints Editor; **Allegro** Constraint Manager (constraint-set reuse, class/region binding) | Every constraint row is a typed `SetDesignRule` op through the single journaled `commit()` — the rules table is undoable/redoable across reopen and scriptable identically from CLI/MCP, which no commercial rules editor offers (their rule table is a settings dialog, not a journaled edit) |
| Constraint surface | Spreadsheet-class constraint authoring, class/region scope | **Allegro** Constraint Manager | Constraint rows resolve `RuleScope` to stable `ObjectId`s at a known `model_revision`; scope edits are validated-before-commit against live objects, not free text that silently dangles |

A surface here is "good enough" only when, on the `datum-test` fixture,
it would not embarrass us next to Altium's online DRC + Rules editor or
Allegro's Constraint Manager on the same task.

---

## 3. The Thesis Applied To This Area

Table stakes to MATCH: violation markers, navigable findings, online DRC
feedback while routing, a credible rules editor.

The wedge we WIN on in this area (the architecture, not tuning):

- **Single-predicate live feedback (differentiator 3).** Live feedback
  and batch `CheckRun` are the SAME engine check predicate over the same
  `DesignModel`, differing only in input: batch runs over committed
  geometry, live runs over committed geometry + the tool's in-progress
  consumer-state geometry. There is no parallel "fast DRC" that can
  disagree. This is the deterministic-engine-backed wedge made concrete:
  what the live overlay warns about is exactly what the next `commit()`
  followed by `CheckRun` will report (modulo the documented edit-time
  `CheckProfile`).
- **Findings as durable, fingerprinted evidence (differentiator 3).**
  Markers reflect persisted `CheckRun` records (`.datum/check_runs`),
  not ephemeral dialog output, so the overlay survives reopen and a
  marker maps to a stable fingerprint a waiver/repair can address.
- **Repair handoff to the AI canvas (differentiator 1).** A finding
  marker's "propose fix" routes to `propose_repair`
  (`GUI_AI_SURFACES.md`), where the candidate `OperationBatch`
  renders as a reviewable ghost. The rules/findings surface is the
  feeder; the ghost is the marquee. No commercial tool turns a DRC
  marker into a visually-reviewable agent diff.

These get the deepest state-machine and golden attention (§5, §7, §8).

---

## 4. Architecture Constraints Inherited (non-negotiable)

From `GUI_SPEC.md` §4. Restated as they bind THIS area:

- **4.1 Render from the resolved DesignModel.** Findings markers render
  from the committed `CheckRun`/`CheckFinding` projection at a known
  `model_revision`; the rules editor renders the resolved `Rule` list.
  The GUI never re-derives findings client-side and never re-runs DRC in
  the renderer.
- **4.2 Mutate only through commit().** Authoring/editing a rule row is a
  typed `SetDesignRule` op through `commit()` (Decision 009; tool
  contract Tool 1). Waiving a finding is an `AddWaiver` op through
  `commit()` (proposal-first; tool contract Tool 3). Accepting a repair
  is a journaled transaction (Tool 4). **Live feedback is NOT a
  mutation:** the in-progress geometry it evaluates is CONSUMER STATE
  (route-in-progress, drag preview) and is never journaled. Running a
  `CheckRun` is NOT a mutation either (Decision 009 step 8 — check
  execution never mutates design truth; the persisted `CheckRun` is
  derived evidence recorded through the commit path, not a design op).
- **4.3 Supervision-reflection first.** The findings overlay + navigator
  (read-only display of committed `CheckRun`s) is the FIRST deliverable
  of this area and ships before any live-feedback-during-edit, which
  ships before correct-by-construction refusal, which ships before the
  reserved push-shove seam. See §10 sequencing.
- **4.4 Visual goldens are acceptance.** Every surface below names a
  golden in the `gui-render` harness (§8). No surface is accepted on
  prose.

---

## 5. Scene-Contract Schema Extensions

Concrete typed extensions to `gui-protocol`, in the shape of the
existing `BoardReviewSceneV1` primitives (`crates/gui-protocol/src/lib.rs`).
All lengths in `nm`. All renderables carry the §2.5 identity triple
(`object_id`, `object_kind`, `source_object_uuid`) where they map to a
persisted object; transient live-feedback primitives carry an identity
that is stable for the duration of the in-progress edit only and are
explicitly NON-persisted (they are derived from consumer state).

### 5.1 `FindingMarkerPrimitive` (persisted-evidence overlay)

Renders one committed `CheckFinding` on the canvas. Sourced from the
`CheckRunReviewState.findings` (`check_runs.rs::CheckFindingSummary`).

```rust
/// One committed CheckFinding rendered as a canvas marker. Derived from a
/// persisted CheckRun; identity-stable across reopen via `fingerprint`.
pub struct FindingMarkerPrimitive {
    /// = CheckFinding.fingerprint. Stable across rename/move/re-import
    /// (Decision 009). This is the canvas identity for the marker.
    pub object_id: String,
    /// Always "finding_marker".
    pub object_kind: String,
    /// The CheckRun this marker belongs to (model_revision-keyed).
    pub check_run_id: String,
    pub finding_fingerprint: String,
    /// "error" | "warning" | "info" — drives marker color/weight.
    pub severity: String,
    /// e.g. "clearance_copper", "process_geometry.paste", domain code.
    pub category: String,
    pub rule_id: String,
    /// Finding lifecycle state. Projected from `CheckFindingSummary.status`
    /// (the engine field is `status`, NOT `state`); the GUI normalizes it to
    /// this closed enum for marker styling:
    /// "unresolved" | "waived" | "deviated" | "resolved" | "stale" | "superseded".
    /// Derived from `status` plus the presence of `waiver_refs`/`deviation_refs`.
    pub finding_state: String,
    /// Engine DrcLocation projected to scene units. Sourced by projecting
    /// `CheckFindingSummary.primary_target` (a `Value` carrying the
    /// `{objects, location:{x_nm,y_nm,layer}}` shape produced by the engine
    /// `DrcLocation`). None for findings whose `primary_target` has no single
    /// point (e.g. a whole-net ERC finding -> see OQ7 + use related markers).
    pub location_nm: Option<PointNm>,
    pub layer_id: Option<String>,
    /// Geometry highlight: the affected object outlines / clearance gap
    /// segment, so the marker is not just a dot. Ordered, byte-stable.
    pub highlight_path: Vec<PointNm>,
    pub width_nm: Option<i64>,
    /// Object ids the finding touches, drives cross-highlight + selection.
    /// Projected from `CheckFindingSummary.primary_target.objects` UNIONED
    /// with each `related_targets[].objects` (the engine has no flat
    /// `affected_object_ids` field; the GUI computes this union read-side).
    pub affected_object_ids: Vec<String>,
    /// True iff `CheckFindingSummary.proposal_refs` is non-empty (a
    /// RepairProposal references this finding). Drives the "propose fix"
    /// affordance -> GUI_AI_SURFACES.md. Read-side derivation only.
    pub has_repair_proposal: bool,
}
```

Draw order: ABOVE copper/silk, BELOW the live-feedback layer (§5.2) and
BELOW AI ghosts. Determinism (two distinct orderings, both
load-bearing):

- **Canonical iteration / byte-stable golden order** is the engine's own
  `DrcReport` order, `(code, message, objects, id)` ascending
  (`drc/mod.rs::sort_by`, verified: `a.code.cmp` then `message` then
  `objects` then `id`). `FindingMarkerPrimitive`s are emitted into the
  scene in exactly this order so a golden over an unchanged committed
  `CheckRun` is byte-identical. The GUI does NOT re-sort by severity for
  the scene vector; severity-first grouping is a NAVIGATOR-only
  presentation transform (§8.1), applied over a stable copy, never
  written back into scene draw order.
- **Within-z paint tiebreak** for overlapping markers is `severity`
  (error over warning over info) so a high-severity marker is never
  occluded by a co-located low-severity one; this affects pixels only,
  not the scene vector order, so it does not perturb the byte-stable
  golden (the golden asserts the ordered vector, the AA tiebreak is a
  pure function of it).

### 5.2 `LiveFeedbackPrimitive` (transient, non-persisted)

The online-DRC overlay shown WHILE a tool is mid-edit. Derived from the
live-feedback predicate (§6) over the tool's in-progress consumer state.
It is NEVER serialized into a persisted scene and NEVER part of a golden
that asserts committed truth (its goldens are interaction-test goldens,
§8).

```rust
/// Transient online-DRC feedback for an in-progress interactive edit.
/// NOT persisted, NOT journaled. Identity scoped to the live edit only.
pub struct LiveFeedbackPrimitive {
    /// Stable for the duration of the in-progress edit only.
    pub live_id: String,
    /// "clearance_gap" | "violation_outline" | "snap_guide" |
    /// "refusal_marker" | "clearance_fence".
    pub primitive_kind: String,
    /// Predicted finding class if this geometry were committed, mirroring
    /// a CheckFinding category. "" for pure snap guides.
    pub predicted_category: String,
    /// "error" | "warning" | "ok" — "ok" allows positive (green) snap
    /// confirmation, matching Xpedition sketch-route legal feedback.
    pub predicted_severity: String,
    /// The deterministic fingerprint the committed finding WOULD carry,
    /// when computable. Lets the same marker survive the commit boundary
    /// as a FindingMarkerPrimitive (continuity, no flicker).
    pub predicted_fingerprint: Option<String>,
    pub layer_id: Option<String>,
    pub path: Vec<PointNm>,
    pub width_nm: Option<i64>,
    /// Numeric readout for the HUD (e.g. "0.12 mm < 0.15 mm required").
    pub observable_label: Option<String>,
    pub expected_value_label: Option<String>,
    pub actual_value_label: Option<String>,
}
```

Draw order: TOP of the canvas (above findings markers and below only the
active tool cursor/HUD), because it is the live edit the user is
manipulating. Determinism note: because it is transient and consumer-
state-derived, it is exempt from the persisted-scene byte-stability rule;
its goldens are captured against a FIXED synthetic interaction frame
(§8), not against project save state.

### 5.3 `RuleRowState` (rules-editor panel, read side)

Read-side projection of the resolved `Rule` list for the rules editor
panel. Mirrors the engine `Rule` schema (Decision 009) and the tool
contract Tool 1 fields. The WRITE side is a `SetDesignRule` op, not a
scene field.

```rust
pub struct RuleRowState {
    /// = Rule.id (ObjectId). Stable identity of the configured rule row.
    pub object_id: String,
    pub object_kind: String, // "design_rule"
    pub rule_code: String,   // RuleType, e.g. "clearance_copper"
    pub rule_version: String,
    pub name: Option<String>,
    /// Human-readable scope expression; resolved-id count shown beside it.
    pub scope_expr: String,
    pub resolved_object_count: usize,
    pub priority: i32,
    pub enabled: bool,
    /// "error" | "warning" | "info" — severity_default (Decision 009).
    pub severity_default: String,
    /// Typed params (min/preferred/max etc.), unit-tagged for display.
    pub parameters: Vec<RuleParamRow>,
    pub basis_refs: Vec<String>,
    /// Validation state from rules/validate.rs surfaced at the row, so the
    /// panel can show validate-before-commit errors inline (§7.2).
    pub validation: RuleRowValidation, // Ok | { field, message }
    /// Live finding count attributable to this rule at the current
    /// model_revision (drives the "N findings" badge, like Altium's
    /// per-rule violation count). Read-only, from the last CheckRun.
    pub attributed_finding_count: usize,
}
```

`RuleRowState` and `RuleParamRow` follow the determinism rules: ordered
by `(priority, rule_code, object_id)`; identity-stable on unchanged
persisted rule shard. No `RuleRowState` field introduces design authority
into the GUI — every value originates in the resolved model; edits leave
via `SetDesignRule`.

---

## 6. The Live-Feedback Predicate (the load-bearing mechanism)

This is the heart of the area and where we beat Altium online DRC by
construction.

### 6.1 Single-predicate rule (non-negotiable)

There is ONE check predicate. Batch `CheckRun` calls it over committed
geometry; the live-feedback layer calls it over committed geometry plus
the tool's in-progress consumer-state geometry. The GUI does not own a
second DRC implementation. Concretely, the engine exposes a check entry
that accepts a "scratch geometry overlay" (the in-progress track/zone/
move) and runs the SAME `drc`/`erc` checks the persisted `CheckRun`
runs, returning `DrcViolation`-shaped results the GUI projects to
`LiveFeedbackPrimitive`s.

Rationale: the cardinal failure mode of Altium-class online DRC is that
the fast incremental checker and the batch checker disagree, so users
learn to distrust the live overlay. Datum forbids that by using one
predicate. This is the proof slice's central assertion (§7.3).

### 6.2 What live feedback evaluates

For the in-progress edit, the predicate evaluates only the rule
categories that are (a) cheap enough for interactive latency and (b)
local to the edited geometry, selected by an **edit-time `CheckProfile`**
(Decision 009 / tool contract: edit-vs-batch is a MODE flag, not a
separate tool). The first edit-time profile MUST include copper
clearance and track-width; SHOULD include via-hole/annular and silk
clearance; MAY defer process-geometry/standards to batch.

The edit-time profile is the documented, owner-selectable difference
between live and batch results. A category in batch-only is shown to the
user as such (the live HUD states "edit-time checks only; run full DRC
for process/standards"), so a user is never misled that a clean live
overlay means a clean board.

### 6.3 Latency and staleness honesty

Live feedback is best-effort and bounded by `QG-PERFORMANCE-LATENCY`
(Decision 012; numeric budget is master OQ8). When the predicate cannot
keep up with pointer motion, the overlay shows a STALE state (dimmed,
"checking…") rather than a wrong-but-confident clean result. This mirrors
the master Non-Goal "live real-time recomputation of every projection on
every keystroke … stale-status is an acceptable interim state" and
`QG-ZONEFILL-HONESTY`: a zone whose fill is `Unfilled`/`Stale`/
`Unsupported` must not be live-checked as if its boundary were solid
copper (Decision 009 zone-honesty rule).

### 6.4 Continuity across the commit boundary

When `predicted_fingerprint` on a `LiveFeedbackPrimitive` equals the
`finding_fingerprint` the post-commit `CheckRun` produces, the GUI
renders no flicker: the transient live marker is replaced in place by the
persisted `FindingMarkerPrimitive` of the same fingerprint. This is the
visible payoff of the single-predicate rule and is golden-asserted (§8).

**Fingerprint-equality precondition (load-bearing, verified against
`drc/fingerprint.rs`).** The engine fingerprint material
(`datum-eda:drc-violation-fingerprint:v1`) hashes `rule_id` (= violation
`code`), `primary_target.{objects,location}`, and `evidence.{id,
rule_type, severity, message, objects, location}`. Two of those inputs
are NOT free for a scratch-overlay edit:
- `objects` are object Uuids. The live scratch geometry has no committed
  Uuid until commit. For `predicted_fingerprint == finding_fingerprint`,
  the route op MUST pre-allocate the SAME Uuid it will commit (the live
  predicate runs over the scratch geometry stamped with the op's
  pre-allocated id), so the object set is identical pre- and post-commit.
- `evidence.id` is the `DrcViolation.id` Uuid. Equality therefore
  requires the violation `id` to be a deterministic function of the
  violation content for the SAME geometry, not a fresh random Uuid per
  run. This is an ENGINE determinism precondition (OQ4): if violation
  `id`s are non-deterministic, fingerprint continuity degrades to
  equality on `(rule_id, primary_target)` only, and the spec's
  no-flicker handoff falls back to matching on those fields (still
  identity-stable across the boundary, but the golden in §8 asserts the
  weaker key). The strong gate (§12.3) is conditional on the engine
  guaranteeing deterministic violation `id`s for identical geometry.

---

## 7. Interaction State Machines

States and transitions, not prose (master §6.2). Selection/hover/drag/
route-in-progress are CONSUMER STATE inside these machines; only the
named commit transition crosses into the journal.

### 7.1 Live-feedback during an interactive route (consumer-state machine)

This machine is the live-feedback OUTPUT attached to the route tool owned
by `GUI_CANVAS_AND_RENDERING.md`. It produces zero journaled mutation;
it only maintains `LiveFeedbackPrimitive`s and feeds the route tool's
correct-by-construction policy.

States: `Idle`, `Routing_Clean`, `Routing_Warned`, `Routing_Violating`,
`Refused`, `Checking`.

| From | Event | To | Consumer-state effect | Commit? |
|---|---|---|---|---|
| `Idle` | route tool begins segment | `Checking` | start scratch geometry from cursor; request live predicate | no |
| `Checking` | predicate returns `ok` | `Routing_Clean` | set `predicted_severity="ok"`; emit green snap guide | no |
| `Checking` | predicate returns `warning` | `Routing_Warned` | emit `clearance_gap` + `observable_label` | no |
| `Checking` | predicate returns `error` | `Routing_Violating` | emit `violation_outline`; HUD red readout | no |
| `Checking` | predicate exceeds latency budget | `Routing_Clean`/last with STALE flag | dim overlay, "checking…"; never assert clean | no |
| `Routing_Clean`/`Warned`/`Violating` | pointer moves | `Checking` | update scratch geometry; re-request predicate | no |
| `Routing_Violating` | commit attempt AND policy=refuse-errors | `Refused` | emit `refusal_marker` with predicted fingerprint; reject segment; geometry reverts to last legal vertex | no |
| `Routing_Warned`/`Clean` | commit segment | `Idle` | build `OperationBatch` from scratch geometry; **call commit()**; transient markers handed to §6.4 continuity | **YES** |
| `Routing_Violating` | commit attempt AND policy=allow-with-violation | `Idle` | build `OperationBatch`; **call commit()**; the violation becomes a real `CheckFinding` on next run (Altium "allow violation" parity, but honest) | **YES** |
| any | Esc / cancel | `Idle` | discard all scratch geometry + live markers; **zero committed mutation, zero dirty shard** | no |

Cancel guarantee (master §6.2): the cancel transition leaves zero
committed mutation and zero dirty shard. The whole machine except the two
`commit segment` transitions is consumer state only.

The correct-by-construction policy (`refuse-errors` vs
`allow-with-violation` vs `warn-only`) is a tool setting, NOT an
operation — it governs which transition fires, never what is journaled.
Refusal never silently mutates geometry (the wedge over Xpedition: a
refused shove is visibly refused with the would-be finding named, not a
half-applied plow).

### 7.2 Rules-editor row edit (commits through the journal)

States: `Viewing`, `EditingRow`, `Validating`, `ValidationError`,
`Committing`.

| From | Event | To | Effect | Commit? |
|---|---|---|---|---|
| `Viewing` | select/begin edit of a `RuleRowState` field | `EditingRow` | local field buffer (consumer state); show live `resolved_object_count` as scope is typed | no |
| `EditingRow` | field change | `Validating` | run `rules/validate.rs` predicate + `RuleScope` resolution against `model_revision` | no |
| `Validating` | valid | `EditingRow` | clear error; enable commit affordance | no |
| `Validating` | invalid (e.g. min>max, scope resolves to 0 objects) | `ValidationError` | show error AT THE EDITED FIELD (`QG-DIRECT-EDITING-FEEL`); commit disabled | no |
| `EditingRow` (valid) | confirm | `Committing` | build `SetDesignRule {previous,current}` op; **call commit()** | **YES** |
| `Committing` | journaled | `Viewing` | row reflects new resolved state at new `model_revision`; `attributed_finding_count` marked STALE until next `CheckRun` | (done) |
| `EditingRow`/`ValidationError` | Esc / cancel | `Viewing` | discard field buffer; **zero op, zero dirty shard** | no |

Validate-before-commit is mandatory (master §6.3): a row can only reach
`commit()` from a valid state; the error is reported at the field, not in
a modal after the fact. This matches Altium's in-editor rule validation
and beats it by making the committed change journaled/undoable.

### 7.3 Finding disposition (waive) — proposal-first

States: `FindingSelected`, `WaiveDraft`, `WaiveValidating`,
`WaiveCommitting`.

| From | Event | To | Effect | Commit? |
|---|---|---|---|---|
| `FindingSelected` | choose Waive | `WaiveDraft` | open rationale + scope + disposition (suppress / accepted-deviation) buffer | no |
| `WaiveDraft` | confirm | `WaiveValidating` | assert the waiver matches a real `finding_fingerprint`; AI-originated waivers route as proposal (Decision 009: AI may propose, never silently waive) | no |
| `WaiveValidating` | valid | `WaiveCommitting` | build `AddWaiver` op; **call commit()** | **YES** |
| `WaiveCommitting` | journaled | `FindingSelected` | finding moves to `waived` (NOT removed); marker re-renders dimmed + "waived" badge; summary `waived` count increments | (done) |
| any | cancel | `FindingSelected` | discard; zero op | no |

A waived finding stays visible and auditable (Decision 009; tool
contract Tool 3). Removing a waiver is journal undo of the `AddWaiver`
transaction, never a separate verb.

"Propose fix" on a finding does NOT live here — it hands the finding
fingerprint(s) to `propose_repair`, whose ghost review machine is in
`GUI_AI_SURFACES.md`.

---

## 8. Panel Specs With Context Rules

### 8.1 Findings panel (navigator)

- **Content per selection class** (the context rule): with nothing
  selected → the full `CheckRunReviewState.findings` presented grouped by
  `(severity, domain)` (the navigator-only presentation transform of
  §5.1; `severity` and `domain` are real `CheckFindingSummary` fields)
  with summary counts (errors/warnings/waived) from
  `check_run_review_state` (`check_runs.rs`; engine `DrcSummary` carries
  `errors`/`warnings`/`waived`, no separate `infos` count — info-severity
  findings are counted in the by-severity group, not a top-line tile).
  With a board object selected → only findings whose computed
  `affected_object_ids` (the §5.1 union of `primary_target.objects` +
  `related_targets[].objects`) include that object's id (matches Altium
  Properties/Violations context behavior). With a rule row selected →
  findings whose `rule_id` matches the row.
- **Drill-down** mirrors the Altium Violations panel from fields that
  ACTUALLY EXIST on `CheckFindingSummary` (`check_runs.rs`), no separate
  explain fetch (tool contract: `explain_finding` is cut):
  - `severity`, `domain`, `code`, `rule_id`, `message` — header line.
  - `explanation` — the human "why". (Real field.)
  - `suggested_next_action: Option<String>` — the recommended fix verb.
    (Real field.)
  - `primary_target` / `related_targets` / `evidence` (`Value`s) — the
    expected-vs-actual numbers are carried INSIDE these structured
    payloads, not as flat `expected_value`/`actual_value` columns (those
    field names do not exist on the engine type). The navigator renders
    the expected/actual readout by projecting the typed `evidence`
    payload for the finding's `domain`; the same projection feeds the
    HUD `expected_value_label`/`actual_value_label` (§5.2), so live and
    committed readouts share one formatter.
- **Navigate-to-finding**: selecting a row pans/zooms the canvas to the
  `FindingMarkerPrimitive` and cross-highlights its `affected_object_ids`
  (cross-probe to schematic via the unified model — the selection IDENTITY
  is owned by `GUI_INTERACTION_GRAMMAR.md`; this panel raises the
  selection).
- **Read-only.** This panel reflects committed `CheckRun` evidence; it
  authors nothing. Disposition actions (waive/propose fix) launch the
  §7.3 / AI machines.
- **Source reflected**: persisted `CheckRun` (`.datum/check_runs`) at the
  current `model_revision`; shows STALE if `model_revision` advanced past
  the run's revision (re-run prompt), never silently shows wrong-revision
  findings.

### 8.2 Rules editor panel

- **Content**: the resolved `RuleRowState` list, one Altium-style row per
  configured rule (typed kind + scope + params + priority + enabled +
  severity), sortable by priority (Allegro Constraint Manager
  spreadsheet feel). Each row carries its `attributed_finding_count`
  badge.
- **Context rule**: when a board/net/net-class object is selected
  elsewhere, the panel can filter to rules whose `RuleScope` resolves to
  that object (matches Altium "rules that apply to this object").
- **Commit path per field**: every editable field routes through the §7.2
  machine → `SetDesignRule` op. No field writes a shard directly (master
  §4.2). Standards basis, severity, and net-class scope are FIELDS ON THE
  ROW, not separate panels (tool contract Minimal-Set).
- **Validate-before-commit**: inline at the field (§7.2).

### 8.3 HUD live readout (during edit)

Matches the Altium heads-up display: while routing/dragging, the cursor
HUD shows the active `LiveFeedbackPrimitive`'s `observable_label`,
`expected_value_label`, `actual_value_label` (e.g. "clearance 0.12 mm <
0.15 mm required"), color-keyed by `predicted_severity`. This is the live
arm of the canvas HUD owned by `GUI_CANVAS_AND_RENDERING.md`; this
spec supplies its content.

---

## 9. Non-Goals

This area does not:
- define the engine rule expression language or individual rule
  semantics (Decision 009 scope; combinator/regex/`IsDiffpair` scopes and
  impedance/length-match/diffpair `rule_type`s are engine gaps per tool
  contract Not-Yet-Supported, not GUI scope).
- implement interactive **push-shove / plow** routing — only RESERVE its
  seam (§11). Refusal and warn are in scope; auto-shoving committed
  geometry is not.
- implement interactive **length / delay tuning** (meanders, tuned
  diff-pairs) — reserve seam only (§11); requires length/diffpair rule
  types the engine does not yet have.
- own the AI repair-proposal ghost rendering (`GUI_AI_SURFACES.md`).
- own manufacturing/process-geometry projection (CAM) RENDERING in-canvas
  (`GUI_CANVAS_AND_RENDERING.md`, where the live-production responsibility
  is folded per master §5); process-geometry findings still appear as
  `FindingMarkerPrimitive`s from a batch run owned by this spec.
- run a second/independent DRC implementation in the GUI (§6.1 forbids
  it).
- claim a clean live overlay or clean `CheckRun` certifies the design
  (Decision 009: a check report means Datum evaluated modeled data
  against selected rules — no certification claim).
- require real-time recomputation on every keystroke; stale-status is
  acceptable (master Non-Goal; §6.3).
- edit `specs/PROGRESS.md`, `specs/SPEC_PARITY.md`, `crates/`, or
  `mcp-server`.

---

## 10. Sequencing Within The Area (supervision-first, Decision 013)

1. **Findings overlay + navigator** (read-only `CheckRun` reflection) —
   the supervision-reflection deliverable; ships first. Golden:
   `golden/rules/findings-overlay`.
2. **Rules-editor panel (read side)** — display resolved `RuleRowState`
   list; still read-only. Golden: `golden/rules/rules-table`.
3. **Rules-editor write** — §7.2 `SetDesignRule` through `commit()` with
   validate-before-commit; durable undo. Interaction test.
4. **Live-feedback during route** — §7.1 consumer-state machine + the
   single-predicate scratch-overlay check. Interaction test +
   commit-continuity golden.
5. **Finding disposition (waive)** — §7.3 `AddWaiver` proposal-first.
6. **Correct-by-construction refusal policy** — `refuse-errors` mode.
7. **(Reserved, not built)** push-shove + length tuning seam (§11).

Each step is a workflow task whose definition-of-done is its proof slice
(§ relevant) plus its visual golden(s).

---

## 11. Reserved Seam: Push-Shove & Length Tuning

Reserved, not specified. The architecture must not foreclose these:

- **Push-shove**: when added, a shove is a PROPOSAL or a journaled
  `OperationBatch` that moves the shoved tracks/vias — never a hidden
  mid-edit mutation. The live-feedback machine (§7.1) already separates
  "in-progress consumer-state geometry" from the commit transition; a
  shove extends the scratch geometry to include the displaced
  neighbors and commits them in the SAME `OperationBatch` as the routed
  segment. The `LiveFeedbackPrimitive` `clearance_fence`/`snap_guide`
  kinds are the hooks for shove preview. (Benchmark: Xpedition sketch
  routing / Allegro shove.)
- **Length / delay tuning**: requires engine `length`/`diffpair`
  `rule_type`s (engine gap). When present, the live HUD readout
  (`observable_label`) generalizes to a length/skew target and the
  meander is in-progress consumer-state geometry committed as a typed op.
  No new GUI authority — same commit spine.

Both are explicitly OUT of the first builds (§9) and gated on engine rule
support.

---

## 12. Proof Slices

Fixture default: the `datum-test` regression fixture
(`~/Documents/kicad_projects/Datum-eda/datum-test/`,
`datum-test.kicad_pcb` + `datum-test.kicad_sch`).

### 12.1 Supervision proof — findings overlay (ships first)

1. Run `datum-eda check run datum-test.kicad_pcb` → a persisted
   `CheckRun` with at least one DRC finding (clearance or process-
   aperture) and the existing summary counts.
2. The GUI loads the `CheckRunReviewState` and renders one
   `FindingMarkerPrimitive` per finding at its `DrcLocation`, with the
   affected-object highlight path.
3. Navigator lists findings with inline `explanation`,
   `suggested_next_action`, and the expected/actual readout projected from
   the typed `evidence`/`primary_target` payload (§8.1 — these are the
   real `CheckFindingSummary` fields, not flat `expected_value`/
   `actual_value` columns); a click navigates the canvas to the marker
   and cross-highlights the computed `affected_object_ids`.
4. Gate: marker identity is the finding `fingerprint`; re-running the
   check at the same `model_revision` yields identical fingerprints
   (Decision 009 determinism), so the overlay is byte-stable
   (golden `golden/rules/findings-overlay`).

### 12.2 Rules-editor write proof

1. In the rules panel, edit the clearance rule's `min` param; the row
   validates (`min<=preferred<=max`, `rules/validate.rs`) and shows the
   live `resolved_object_count`.
2. Confirm → `SetDesignRule` op through `commit()`; the row reflects the
   new value at the new `model_revision`.
3. Re-run check → finding count changes accordingly; the edit is undone
   by journal undo (`QG-DURABLE-UNDO`) and survives reopen.
4. Gate: identical `SetDesignRule` transaction whether driven from the
   GUI, CLI (`datum-eda rules set …`), or MCP (`set_design_rule`) — same
   journal entry (differentiator 3).

### 12.3 Live-feedback single-predicate proof (the wedge)

1. Begin an interactive route segment that runs within sub-clearance of
   an existing track; the live overlay shows `Routing_Violating` with the
   red `violation_outline` + HUD readout, and a `predicted_fingerprint`.
2. Commit the segment with `allow-with-violation`.
3. Run a full `CheckRun`; assert the produced `CheckFinding.fingerprint`
   EQUALS the `predicted_fingerprint` shown live (the single-predicate
   gate, §6.1 / §6.4) — i.e. the live overlay did not lie. This requires
   the route op to have pre-allocated the committed track Uuid before the
   live predicate ran, and the engine to produce a deterministic
   violation `id` for identical geometry (§6.4 precondition / OQ4). If
   the engine cannot yet guarantee deterministic violation `id`s, the
   asserted key weakens to `(rule_id, primary_target.objects,
   primary_target.location)` and the slice is marked engine-blocked on
   OQ4 rather than passing on the full fingerprint.
4. Re-route to a legal position with `refuse-errors`; assert the segment
   is refused with a `refusal_marker`, geometry reverts to the last legal
   vertex, and ZERO op was committed and ZERO shard dirtied (cancel/
   refusal guarantee).

### 12.4 Waive proof

1. Waive one finding via §7.3 with rationale; `AddWaiver` through
   `commit()`.
2. Re-run check; the finding is `waived` (visible, dimmed), not removed;
   `waived` count increments; undo restores `unresolved`.

---

## 13. Visual-Golden Acceptance

Every surface ships a golden + interaction test in the `gui-render`
harness (`visual_runner.rs`/`visual_manifest.rs`/`visual_diff.rs`/
`visual_capture.rs`; bless + diff). A surface is accepted only when its
golden renders REAL committed state (or, for live feedback, a fixed
synthetic interaction frame) and its interaction test exercises the
state machine.

| Golden | Drives | Scene/fixture | Accept |
|---|---|---|---|
| `golden/rules/findings-overlay` | §5.1 `FindingMarkerPrimitive`s over the board | `datum-test` + a persisted `CheckRun` with clearance + process-aperture findings | exact diff (committed `CheckRun` is deterministic) |
| `golden/rules/findings-overlay-waived` | waived finding dimmed + badge, summary counts | `datum-test` + `CheckRun` + one `AddWaiver` | exact diff |
| `golden/rules/rules-table` | §5.3 `RuleRowState` list, priority-sorted, finding-count badges | `datum-test` resolved rules | exact diff |
| `golden/rules/rules-table-validation-error` | inline validate-before-commit error at the edited field | `datum-test`, scripted invalid edit (min>max) | exact diff |
| `golden/rules/live-feedback-violating` (interaction) | §5.2 `LiveFeedbackPrimitive` violation outline + HUD readout mid-route | fixed synthetic in-progress route frame over `datum-test` | tolerance diff (anti-aliased overlay), interaction test asserts state `Routing_Violating` |
| `golden/rules/live-feedback-commit-continuity` (interaction) | §6.4 no-flicker handoff: live `predicted_fingerprint` → persisted `FindingMarkerPrimitive` same fingerprint | before/after-commit frame pair on `datum-test` | interaction test asserts continuity-key equality (full `fingerprint` if engine guarantees deterministic violation `id` per OQ4, else the `(rule_id, primary_target.objects, primary_target.location)` fallback key) + exact diff on the after-frame |
| `golden/rules/live-feedback-refusal` (interaction) | refusal marker + geometry revert under `refuse-errors`; zero committed op | scripted refused segment | interaction test asserts zero journal entry + tolerance diff |

The dormant `check_gui_parity` fuse (Decision 013) arms when the
interactive-editor phase begins; until then these per-surface goldens are
this area's definition-of-done.

---

## 14. Open Questions

1. **Edit-time `CheckProfile` contents.** Exactly which rule categories
   are in the first edit-time (live) profile vs batch-only? Tool contract
   OQ1/OQ5 and Decision 009 leave the first profiles open. Proposed
   minimum: clearance + track-width live; process-geometry/standards
   batch-only. Owner to ratify the live set and the batch-only banner
   wording.
2. **Live-feedback latency budget.** The numeric `QG-PERFORMANCE-LATENCY`
   budget for the live predicate per pointer move on `datum-test` (master
   OQ8). Above what latency does the overlay flip to STALE rather than
   block the pointer?
3. **Correct-by-construction default policy.** Is the default route
   policy `warn-only`, `refuse-errors`, or owner-configurable per rule
   severity? Altium defaults to allow-with-violation + marker; Xpedition
   leans refuse/shove. Datum default unset.
4. **Scratch-overlay check entry point + deterministic violation id.**
   §6.1 requires an engine check entry that runs the same predicate over
   committed geometry + a scratch overlay. Two sub-questions, both
   engine-owned, both gating the §6.4 strong fingerprint-continuity gate:
   (a) does the engine expose this incrementally (only edited geometry's
   neighborhood) for latency, and is the incremental result provably
   identical to the full-board batch result for the same geometry? (b)
   is `DrcViolation.id` a deterministic function of violation content for
   identical geometry (so `evidence.id` in the fingerprint material is
   stable pre/post commit), or is it a fresh Uuid per run? If (b) is
   non-deterministic, full-fingerprint continuity is impossible and the
   GUI must fall back to the `(rule_id, primary_target)` continuity key
   (§6.4). Owner/engine decision, not GUI.
5. **Deviation as its own disposition.** Tool contract OQ3 ratification
   fork: for the first slice waive carries `disposition=accepted-
   deviation`; does a standalone `Deviation` primitive (with its own
   marker state and approval state machine) graduate now or stay merged?
6. **ERC live feedback.** Is live feedback PCB-only at first (clearance/
   width), or does schematic editing get live ERC feedback in the same
   release? ERC findings render as markers either way; the live-during-
   edit arm for schematic is the open part.
7. **Whole-net / non-point findings on canvas.** ERC findings and some
   connectivity findings have no single `DrcLocation`. Render strategy:
   highlight all `affected_object_ids`, anchor the marker at the net's
   centroid, or list-only in the navigator with no canvas marker?
8. **Stale-finding presentation.** When `model_revision` advances past a
   `CheckRun`, do existing markers dim-as-stale in place (continuity) or
   clear until re-run (honesty)? §8.1 leans dim-as-stale-with-re-run-
   prompt; owner to confirm against the no-false-confidence rule.
