# Datum EDA GUI — AI Surfaces Specification (Area 4: The Wedge)

Status: draft GUI area specification, 2026-06-22, benchmarked to
commercial EDA. Controlling for the AI-native canvas, the agents/terminal
docks, the assistant surface, and visible provenance. Conforms to the
master `specs/GUI_SPEC.md`; inherits its bar, thesis, four architecture
constraints, and five-part buildability standard. This file does not edit
code, `specs/PROGRESS.md`, `specs/SPEC_PARITY.md`, `crates/`, or
`mcp-server`.

Driven by:
- `specs/GUI_SPEC.md` (master; this is the §5 area 4 detail, expanded to
  cover the agents/terminal docks and assistant surface that the AI
  canvas depends on)
- `docs/decisions/PRODUCT_MECHANICS_004_AI_TOOLING_CONTRACT.md` (proposal
  mechanics, contract classes, provenance, private-mutation ban)
- `docs/decisions/PRODUCT_MECHANICS_005_EMBEDDED_TERMINAL.md` (real PTY,
  context injection, the `AGENTS` dock as a terminal-agent launcher)
- `docs/decisions/PRODUCT_MECHANICS_006_ASSISTANT_SURFACE.md` (bounded
  read/propose/apply UI, card structure, apply-controls-never-in-text)
- `docs/contracts/AI_CLI_MCP_TOOL_SURFACE.md` (proposal lifecycle,
  diff/preview, `ComponentInstance` join, provenance query families)
- `crates/engine/src/substrate/proposal.rs` (the `Proposal`,
  `ProposalStatus`, `ProposalApplyValidation`, `OperationBatch` types
  this surface visualizes — read-only ground truth)
- `crates/gui-protocol/src/lib.rs` (`BoardReviewSceneV1`,
  `ProposalOverlayPrimitive`, `ReviewPrimitive`, identity triple)
- `crates/gui-app/src/{assistant_bridge,terminal_agent_launcher,terminal_*}.rs`
  (existing assistant bridge and `AGENTS`/terminal dock substrate)

---

## 1. Purpose And Why This Is The Marquee Surface

This is the differentiator with no commercial precedent. Altium, Allegro,
Xpedition, and CR-8000 have no concept of an agent's proposed edit
rendered as an in-canvas ghost you approve or reject visually. Their AI
features (where present at all) are sidebar chat or batch automation that
mutates the design and leaves you to diff after the fact. Datum's
architecture makes the opposite possible by construction: every candidate
mutation is already a typed `OperationBatch` owned by a `Proposal`
(Decision 004; `crates/engine/src/substrate/proposal.rs`), prepared
against a known `model_revision`, with `affected_objects` stored and a
classified `TransactionDiff` (`created`/`modified`/`deleted`) recoverable
by replaying the batch onto a model clone — exactly what
`create_draft_proposal_from_batch` does internally to derive
`affected_objects` (§4.2). The GUI's job is to RENDER that already-typed proposal as ghosted
geometry on the same canvas as committed state, let a supervisor inspect
the diff in place, and accept/reject through the canonical
`proposal apply` path — never a private write.

We do not match a $10k/seat tool here. We do something none of them can,
because none of them has a single resolved model and a single journaled
`commit()` underneath every actor.

This spec owns four coupled surfaces:
1. **AI-native canvas** — proposals as ghosts/diffs on board and
   schematic, reviewed and accepted/rejected visually.
2. **Agents dock** — the `AGENTS` terminal-agent launcher (Decision 005);
   a launcher into the real PTY, never a second design authority.
3. **Terminal dock** — the real PTY surface (Decision 005), specified
   here only at its AI-relevant boundary: it produces proposals via the
   CLI, it does not parse text into edits.
4. **Assistant surface** — the optional bounded card-based
   read/propose/apply panel (Decision 006).

Supervision-reflection precedes interactivity here as everywhere
(Decision 013): the read-only proposal-review surface (render a committed
or draft proposal as ghosts and let a human audit it) ships before any
in-canvas authoring that originates a proposal.

---

## 2. The Bar (commercial benchmark + match-vs-exceed)

### 2.1 Named commercial benchmark

There is no direct commercial analog to the AI-native canvas, so we
benchmark against the closest professional review/diff/preview UX in each
tool and then state what we do that they cannot:

- **Altium Designer** — the benchmark for the REVIEW interaction quality
  we must MATCH: the ECO (Engineering Change Order) preview dialog and the
  Storyboard/compare-and-apply flow; the dynamic route/ActiveRoute
  preview that shows proposed copper before commit; the Properties-panel
  affected-object emphasis. Altium shows a proposed change before you
  commit it. We MATCH that fluency (ghost rendered before commit, clear
  affected-object emphasis) and EXCEED it: Altium's ECO preview is a modal
  list dialog, not a manipulable in-canvas ghost reviewed against live
  geometry, and it is not produced by an agent.
- **Cadence Allegro / OrCAD** — the benchmark for batch-change preview
  and the "show what will change" discipline of Constraint Manager and the
  shape-update preview. We MATCH the show-before-apply discipline and
  EXCEED it by making the previewed change a typed `OperationBatch` an
  agent authored, with rationale and risks attached.
- **Siemens Xpedition** — the benchmark for interactive plow/shove
  preview (the proposed copper rendered live as you sketch). We MATCH the
  visual-preview-of-proposed-geometry quality bar; we EXCEED it because
  the proposed geometry can come from an offline agent run and be reviewed
  asynchronously as a durable `Proposal` shard.
- **Zuken CR-8000** — design-reuse-block placement preview. We MATCH the
  block-preview quality; we EXCEED by making the block an inspectable
  proposal with provenance and a diff.

KiCad is the FLOOR: KiCad has no proposal/ghost/diff concept at all. We
exceed it the moment we render a single ghost.

### 2.2 Where we MATCH

- review interaction fluency: a proposed change is visible BEFORE commit,
  with snap-quality rendering equal to committed geometry (matches Altium
  ActiveRoute preview / ECO preview).
- affected-object emphasis: the objects a change touches are visually
  distinguished (matches Altium Properties affected-object highlight).
- batch discipline: nothing is applied silently; a clear apply/reject
  gate (matches Allegro shape-update / ECO apply confirmation).

### 2.3 Where we EXCEED (the wedge)

- **Agent proposals as in-canvas ghosts.** No commercial tool renders an
  agent's edit as a ghost on the live canvas you accept/reject in place.
- **Single-identity diff.** The diff is the engine's own
  `TransactionDiff`, produced by replaying the `OperationBatch` onto a
  model clone (`proposal.rs` `create_draft_proposal_from_batch` does
  precisely this — `preview.commit(batch.clone())` then reads
  `transaction.diff.{created,modified,deleted}`). The GUI renders that
  classified diff verbatim, not a GUI-recomputed netlist/geometry
  approximation. Beats every commercial tool's heuristic compare because
  the "diff" is the actual commit the actual engine would perform.
- **Accept == one journaled transaction.** Accept routes through
  `apply_accepted_proposal` -> one `TransactionRecord`, durably
  undoable across reopen. Reject leaves zero dirty shard. No commercial
  tool's AI feature is replayable/undoable-across-reopen by construction.
- **Visible provenance on every ghost.** Actor, tool, session, acceptance
  path, model revision, affected objects — rendered, not buried in a log
  (Decision 004 provenance rule).

---

## 3. Architecture Constraints Restated For This Area (non-negotiable)

Inherited from master §4 and the decision docs; a surface in this area
may not relax them.

1. **Render-only.** The AI canvas renders ghosts ONLY from a `Proposal`'s
   engine-computed diff/preview, surfaced through the scene contract. The
   GUI never computes design geometry for a ghost; it renders the engine's
   replay result. (Master §4.1; Decision 004 reads-through-resolved-model.)
2. **Accept/reject are the only mutations, and they go through
   `commit()`.** Accepting a ghost calls `proposal review (accept)` then
   `proposal apply`, which calls `apply_accepted_proposal` ->
   `commit_journaled` -> one `TransactionRecord`. Rejecting calls
   `proposal review (reject)`, which is itself a journaled metadata
   transaction (see `review_proposal_status`), NOT a private file write.
   Hover/select/expand-of-a-ghost is consumer state, never journaled.
   (Master §4.2; Decision 004 commit gateway; `proposal.rs`.)
3. **Supervision-first.** The read-only "render a proposal as a reviewable
   ghost and audit its diff/provenance/checks" surface ships before any
   surface that ORIGINATES a proposal from in-canvas authoring.
   (Master §4.3; Decision 013.)
4. **Visual goldens are acceptance.** Every surface here ships a golden
   that renders a real `Proposal` from the `datum-test` fixture plus an
   interaction test exercising its state machine. (Master §4.4.)
5. **The terminal is a real PTY, never an edit bridge.** The `AGENTS`
   dock focuses the PTY and prints shell guidance (existing
   `terminal_agent_launcher.rs`); it does not enter a separate authoring
   authority. Terminal-launched agents produce proposals via the CLI;
   their text is never parsed into edits. (Decision 005; `QG-PTY-REAL`.)
6. **The assistant has no private editing API.** It reads via
   `DatumQueryTool`, checks via `DatumCheckTool`, drafts via
   `DatumProposalTool`, and applies ONLY via `DatumProposalTool.apply` ->
   `DatumCommitTool`. Apply controls are product UI actions, never hidden
   in generated text. (Decision 006.)

---

## 4. Scene-Contract Schema Extensions (§6.1)

The contract already carries `ProposalOverlayPrimitive` and
`ReviewPrimitive` (`crates/gui-protocol/src/lib.rs`), built for the M7
route-proposal review spike. Those are RETAINED. This area generalizes
them from "route proposal review" to "any `Proposal` rendered as a
ghost/diff", as a new versioned companion scene the existing renderer
composes over `BoardReviewSceneV1`. All extensions obey the master §6.1
determinism rules: versioned, `nm` units, explicit draw order, the §2.5
identity triple on every renderable, byte-stable ordering, identity-stable
on unchanged persisted state, and NO new field introduces design
authority into the GUI (every field is a render of engine-computed state).

### 4.1 `ProposalReviewSceneV1` (new companion scene)

A read-only render of one `Proposal`'s engine-computed diff, composed over
the committed `BoardReviewSceneV1` (or the schematic equivalent). It is
NOT authority; it is a projection of `Proposal` + the engine's replay
diff. Ordering is deterministic: sorted by `change_index` (the stable
order the engine emits the diff in).

```
ProposalReviewSceneV1 {
  kind: "proposal_review_scene"            // discriminant
  version: u32                             // = 1
  scene_id: String                         // stable per proposal render
  proposal_id: String                      // = Proposal.proposal_id
  project_uuid: String
  prepared_against_revision: String        // = Proposal.prepared_against
  current_model_revision: String           // for stale detection in-canvas
  base_scene_ref: String                   // scene_id of the committed BoardReviewSceneV1 it overlays
  source: String                           // ProposalSource: manual|cli|tool|assistant|check|import
  status: String                           // ProposalStatus: draft|accepted|deferred|rejected|applied
  stale: bool                              // prepared_against != current_model_revision
  rationale: String                        // Proposal.rationale (rendered in the review card, not the canvas)
  provenance: ProposalProvenanceBlock      // see 4.4
  diff_summary: ProposalDiffSummary        // counts + check status (see 4.3)
  ghost_primitives: Vec<GhostPrimitive>    // the in-canvas ghosts (see 4.2)
  affected_emphasis: Vec<AffectedObjectEmphasis> // emphasis on committed objects the change touches
  unresolved_assumptions: Vec<String>      // Decision 004: assumptions surfaced, never hidden
  risks: Vec<String>                       // Decision 004: risks surfaced
}
```

### 4.2 `GhostPrimitive` (the in-canvas ghost)

One renderable ghost for each created/modified/deleted geometric object
the engine's diff reports. It carries the §2.5 identity triple plus the
diff role. `change_kind` comes from the engine diff, never inferred by the
GUI.

Ground-truth derivation (this is the load-bearing dependency, stated
precisely so the scene builder is buildable). The per-kind classification
is `TransactionDiff { created, modified, deleted }`, produced by replaying
the proposal's `OperationBatch` on a model clone:
`create_draft_proposal_from_batch` (`proposal.rs`) already does exactly
this — `let mut preview = model.clone(); let preview_report =
preview.commit(batch.clone())?;` — and reads
`preview_report.transaction.diff.{created,modified,deleted}`. BUT it then
FLATTENS those three lists into the single sorted-deduped
`Proposal.affected_objects` and stores ONLY that; the per-kind split and
any ordering are NOT persisted on the `Proposal` shard. Therefore the
scene builder MUST obtain `change_kind`/`change_index` by re-running the
same preview commit to recover the live `TransactionDiff` (or by an engine
capability that exposes the classified diff for a stored proposal — see
OQ11). The GUI never reconstructs the classification from geometry; it
renders the engine's `TransactionDiff` verbatim. `change_index` is the
deterministic position within the engine's `created` ++ `modified` ++
`deleted` concatenation (stable because `commit` is deterministic).

```
GhostPrimitive {
  // identity triple (master §2.5) — points at the PROPOSED object identity
  object_id: String                        // engine-assigned object id of the proposed/affected object
  object_kind: String                      // "track" | "via" | "pad" | "zone" | "component" | "wire" | "symbol" | "label" | ...
  source_object_uuid: String               // stable source uuid; for created objects = the deterministic id the batch assigns

  ghost_id: String                         // stable per (proposal_id, change_index)
  change_index: u64                        // engine diff order; primary sort key
  change_kind: String                      // "created" | "modified" | "deleted"
  proposal_action_id: String               // ties ghost to a per-action accept/reject control when granular review is enabled

  layer_id: Option<String>                 // for layer-scoped board geometry
  render_role: String                      // "ghost_add" | "ghost_remove" | "ghost_modify_before" | "ghost_modify_after"
  geometry_kind: String                    // "polyline" | "polygon" | "circle" | "rect" | "text" — mirrors the committed primitive families
  path: Vec<PointNm>                       // nm; same coordinate space as committed scene
  width_nm: Option<i64>
  drill_nm: Option<i64>                    // for via/pad ghosts
}
```

Render-role semantics (the visual diff language; the renderer maps these
to appearance, the contract only carries the role):
- `ghost_add` — proposed-new geometry, drawn as a translucent additive
  ghost over the committed layer.
- `ghost_remove` — geometry the proposal deletes, drawn as a
  struck/hatched removal ghost over its still-committed position.
- `ghost_modify_before` / `ghost_modify_after` — a modified object's prior
  and proposed states, paired by `object_id`, so the supervisor sees the
  before/after delta in place (matches Altium ECO before/after, exceeds it
  by being on-canvas and agent-authored).

### 4.3 `ProposalDiffSummary` and check linkage

```
ProposalDiffSummary {
  created_count: u64                        // = TransactionDiff.created.len() from the preview replay
  modified_count: u64                       // = TransactionDiff.modified.len()
  deleted_count: u64                        // = TransactionDiff.deleted.len()
  affected_object_count: u64               // = Proposal.affected_objects.len() (the flattened stored field)
  checks_run: Vec<String>                  // = Proposal.checks_run (CheckRun uuids, rendered as strings)
  finding_fingerprints: Vec<String>        // = Proposal.finding_fingerprints — the findings this proposal addresses
  check_status: String                     // "not_run" | "pass" | "fail" | "stale"
}
```

`created_count/modified_count/deleted_count` are the lengths of the same
`TransactionDiff` lists the ghosts are built from (§4.2 derivation), so the
summary and the rendered ghosts are guaranteed consistent —
`created_count + modified_count + deleted_count` equals the number of
`GhostPrimitive`s after `ghost_modify_before/after` pairing (a modified
object yields two ghost render rows but one `modified_count`). The scene
builder asserts this equality; a mismatch is a contract violation, not a
render. `check_status` is the engine's classification (Decision 004 check
records are revision-keyed and go stale on `model_revision` movement); the
GUI never recomputes it. NOTE: `Proposal` carries `checks_run` and
`finding_fingerprints` but does NOT carry a stored `check_status` enum
today — deriving `check_status` from the keyed `CheckRun` records against
the current revision is an engine read the scene builder depends on (OQ4).

### 4.4 `ProposalProvenanceBlock` (visible provenance — Decision 004)

Every ghost render and review card carries provenance. This is the field
set Decision 004 mandates be inspectable for any tool-originated change.

```
ProposalProvenanceBlock {
  actor: String                            // user/tool identity
  actor_type: String                       // User|Cli|Mcp|Script|Importer|Checker|Router|Assistant|ExternalAgent
  tool: String                             // command / MCP method / assistant action / executable+argv summary
  session_id: String                       // DatumToolSession.session_id
  acceptance_path: Option<String>          // set once accepted: "gui_review" | "cli_apply" | "policy_auto" | ...
  model_revision: String                   // = Proposal.prepared_against
  applied_transaction_id: Option<String>   // = Proposal.applied_transaction_id (set once applied)
}
```

Derivation and a deliberate gap (stated so this is buildable, not
aspirational). Today the `Proposal` struct stores only `source`
(`ProposalSource: manual|cli|tool|assistant|check|import`),
`prepared_against`, and `applied_transaction_id`. The richer provenance
fields above — `actor`/`actor_type`/`tool`/`session_id`/`acceptance_path`
— are NOT all present on the `Proposal` shard at this writing. The
mappable-today subset is: `model_revision` <- `prepared_against`,
`applied_transaction_id` <- `applied_transaction_id`, and a coarse
`actor_type`/`tool` derivable from `source`. The full block is resolved by
joining the proposal to its journal lineage: `acceptance_path`,
`session_id`, and the precise `actor`/`tool` come from the
`TransactionRecord`(s) the `proposal review`/`proposal apply` path writes
(the `provenance.query`/`transactions.query` families in
`AI_CLI_MCP_TOOL_SURFACE.md`). Where a field is not yet sourced, the GUI
renders it as an explicit `unknown` marker (the Decision 004
unknown-basis convention) — it NEVER fabricates provenance. Closing the
gap (which provenance fields the engine must persist vs. reconstruct from
the journal) is OQ10/OQ12.

### 4.5 `AffectedObjectEmphasis`

Emphasis applied to COMMITTED objects (in the base scene) that the change
touches but does not itself ghost — e.g. a net whose ratsnest changes, a
component whose pad a routed track now lands on. Render-only; points at
committed identity. Derivation: the emphasis set is
`Proposal.affected_objects` (the stored flattened list) restricted to
objects that exist in the COMMITTED base scene and are not themselves a
ghost — i.e. `affected_objects` minus the `TransactionDiff.created` ids
(which appear as `ghost_add`) and minus `TransactionDiff.deleted` ids
(which appear as `ghost_remove`). Modified objects appear BOTH as a
ghost (before/after) and may carry `touched` emphasis on their committed
identity. The scene builder computes this set difference from the same
`TransactionDiff` used for ghosts, so emphasis and ghosts never disagree.

```
AffectedObjectEmphasis {
  object_id: String                        // committed object id in base_scene_ref
  object_kind: String
  source_object_uuid: String
  emphasis_role: String                    // "touched" | "neighbor" | "constraint_scope"
}
```

### 4.6 Non-authority invariant

`ProposalReviewSceneV1` and all its primitives are derived entirely from a
`Proposal` shard plus the engine's replay diff. The GUI persists nothing
from it. There is no field by which the GUI could author or alter a
proposal — proposal mutation happens only through the
`proposal create|review|apply` engine path. This satisfies master §6.1
"no new field may introduce design authority into the GUI."

---

## 5. Interaction State Machines (§6.2)

States and transitions, not prose. Selection/hover/expand are consumer
state. Exactly one transition per machine crosses into the journal
(accept/reject = a journaled metadata or applied transaction).

### 5.1 Proposal Review machine (the AI-native canvas core)

Read-only audit first (supervision), then the accept/reject gate. This is
the machine the marquee surface ships.

States:
- `S0_NoProposal` — no proposal selected; base scene only.
- `S1_LoadingDiff` — a proposal selected; engine replay/diff requested.
- `S2_ReviewingDiff` — ghosts rendered over base; supervisor auditing.
- `S2a_GhostHovered` — a ghost is hovered (consumer state; shows the
  ghost's provenance/change tooltip).
- `S2b_GhostSelected` — a ghost is selected (consumer state; cross-probes
  its affected committed objects, drives the review card focus).
- `S3_StaleBlocked` — `prepared_against != current_model_revision`; accept
  disabled, revalidate/discard offered (mirrors
  `ProposalApplyValidation.blockers` `stale_model_revision`).
- `S4_AcceptPending` — accept requested; calls `review(accept)` then
  `apply`; awaiting `CommitReport`.
- `S5_RejectPending` — reject requested; calls `review(reject)`.
- `S6_Applied` — `CommitReport` returned; ghosts retire, base scene
  refreshes to the new `model_revision`, provenance shows
  `applied_transaction_id`.
- `S7_Rejected` — proposal status `rejected`; ghosts cleared; zero dirty
  shard.
- `S8_Error` — engine returned a blocker/error; surfaced verbatim.

Transitions (event -> next; consumer-state mutated; commit boundary):
- `S0 --select_proposal--> S1` (consumer: `selected_proposal_id`)
- `S1 --diff_ready--> S2` (consumer: cache `ProposalReviewSceneV1`) |
  `S1 --diff_error--> S8`
- `S1 --diff_ready & stale--> S3`
- `S2 --hover_ghost--> S2a` / `S2a --unhover--> S2` (consumer:
  `hovered_ghost_id`)
- `S2 --click_ghost--> S2b` (consumer: `selected_ghost_id`;
  cross-probe emphasis on affected committed objects)
- `S2b --click_empty--> S2`
- `S2 --accept--> S4` ONLY if `check_status != fail` per policy gate (OQ);
  **COMMIT BOUNDARY**: `proposal review(accept)` (journaled metadata tx)
  then `proposal apply` -> `apply_accepted_proposal` -> one
  `TransactionRecord`.
- `S2 --reject--> S5`; **COMMIT BOUNDARY**: `proposal review(reject)`
  (journaled metadata tx; leaves zero design-geometry dirty shard).
- `S2 --defer--> S0`; **COMMIT BOUNDARY**: `proposal review(defer)`
  (journaled metadata tx; proposal persists as `deferred`).
- `S3 --revalidate--> S1` (re-prepare against current revision; OQ:
  auto-rebase vs. discard) | `S3 --discard--> S5`
- `S4 --commit_ok--> S6` | `S4 --commit_blocked--> S8` (surface
  `ProposalApplyValidation.blockers` verbatim)
- `S5 --review_ok--> S7`
- `S6 --dismiss--> S0` / `S7 --dismiss--> S0`
- `S8 --acknowledge--> S2` (return to review; no mutation occurred)

Cancel invariant: from `S2`/`S2a`/`S2b`, leaving the surface without
accept/reject leaves the proposal `draft` and zero dirty shard. Only the
accept/reject/defer/revalidate-as-accept transitions cross the journal.

Granular (per-action) accept (OPTIONAL, gated by OQ): if a proposal's
`OperationBatch` is partitionable, a ghost's `proposal_action_id` lets a
supervisor accept a SUBSET. This MUST be modeled as the engine producing a
NEW narrowed `OperationBatch`/`Proposal` (re-prepared against current
revision), never as the GUI editing the batch. Until the engine supports
partition, this is whole-proposal accept only (see OQ7).

### 5.2 Assistant Card machine (Decision 006)

The assistant surface is card-based, not a chat blob. Apply controls are
product UI actions, never in generated text (Decision 006).

States:
- `A0_Idle` — assistant panel open, context card shown.
- `A1_Reading` — running a `DatumQueryTool` query (read-only).
- `A2_Checking` — running a `DatumCheckTool` check.
- `A3_DraftingProposal` — composing an `OperationBatch` via
  `DatumProposalTool.create` (status `draft`).
- `A4_ProposalDrafted` — proposal card shown: rationale, affected
  objects, diff summary, assumptions, risks, checks, stale/conflict state,
  actions. Hands the proposal to the §5.1 Review machine for in-canvas
  ghosting.
- `A5_AwaitingApproval` — apply card shown; explicit approval required.
- `A6_Applying` — `DatumProposalTool.apply` -> `DatumCommitTool`.
- `A7_Applied` — apply card shows transaction id, journal tip, invalidated
  derived state/artifacts, assistant provenance.
- `A8_Handoff` — handoff card: terminal/CLI/manual-editor action when the
  task belongs outside the assistant.
- `A9_Stale` — model revision moved under a draft; proposal marked stale;
  revalidate or discard.

Transitions:
- `A0 --ask--> A1 --result--> A0` (consumer: conversation/card state; no
  mutation)
- `A0 --run_check--> A2 --findings--> A0`
- `A0 --draft--> A3 --created--> A4`
- `A4 --review_in_canvas--> (hand to §5.1 S1)` (the draft renders as
  ghosts; the canvas Review machine owns accept/reject)
- `A4 --request_apply--> A5`
- `A5 --approve--> A6`; **COMMIT BOUNDARY**: `DatumProposalTool.apply` ->
  `DatumCommitTool` -> one `TransactionRecord`.
- `A6 --ok--> A7` | `A6 --blocked--> A4` (blockers shown on the proposal
  card)
- any `A*` `--revision_moved--> A9 --revalidate--> A3` /
  `A9 --discard--> A0`
- `A0/A4 --handoff--> A8`

Hard invariants (Decision 006): the assistant cannot reach `A6` except via
the explicit `A5 --approve--> A6` UI action; it never presents `A7` state
unless `commit()` actually returned; apply controls never appear inside
generated message text.

### 5.3 Agents-dock / Terminal machine (Decision 005 boundary)

The `AGENTS` dock is a launcher into the real PTY, not an authoring
authority. The existing `open_terminal_agent_launcher`
(`terminal_agent_launcher.rs`) already focuses the terminal and prints
context-aware shell guidance; this spec governs only its AI-surface
boundary.

States:
- `T0_TerminalActive` — PTY running; normal shell.
- `T1_AgentLauncherInvoked` — `AGENTS` dock activated; focuses PTY, prints
  the context-aware launch hint (codex/claude/aider + `$DATUM_DISCOVERY` +
  `context session-activity`). This is the existing
  `TERMINAL_AGENT_LAUNCHER_HINT` behavior.
- `T2_AgentRunningInPty` — user launched an agent in the shell; it is a
  normal child process.
- `T3_ProposalSurfaced` — a terminal-launched agent ran
  `datum-eda proposal create`; a new `Proposal` shard appears under
  `.datum/proposals/`; the GUI detects it and offers it to the §5.1 Review
  machine.

Transitions:
- `T0 --activate_agents_dock--> T1` (consumer: active dock = Terminal;
  writes per-session discovery context; NO design mutation)
- `T1 --user_runs_agent--> T2`
- `T2 --agent_creates_proposal--> T3` (the proposal is journaled metadata
  via the CLI's `proposal create`; the GUI only DETECTS and renders it)
- `T3 --open_in_canvas--> (hand to §5.1 S1)`

Hard invariant (Decision 005): no transition here mutates design state.
The terminal never parses shell text into edits. A design change from a
terminal agent exists only as a `Proposal` produced by the CLI, reviewed
through §5.1.

---

## 6. Panel / Component Specs With Context Rules (§6.3)

### 6.1 Proposal Review card (the canvas-side inspector)

Context rule by `ProposalReviewSceneV1.status` and selection:
- always: proposal id, source, status, `prepared_against` vs.
  `current_model_revision` with a stale badge, rationale, provenance block
  (4.4), diff summary (4.3), unresolved assumptions, risks.
- when a ghost is selected (`S2b`): the card focuses that change —
  before/after geometry summary, the affected committed objects (cross-probed),
  and the check finding(s) this change addresses (by fingerprint).
- when `stale` (`S3`): accept disabled, blockers from
  `ProposalApplyValidation` shown verbatim, revalidate/discard offered.

Validate-before-commit (master §6.3 / `QG-DIRECT-EDITING-FEEL`): the
accept action calls `validate_proposal_apply` first and renders
`ProposalApplyValidation` directly. If `!can_apply`, each
`ProposalApplyBlocker { code, message }` renders AT the accept control
(not as a toast), grouped by `code` and showing the engine's verbatim
`message` — the supervisor sees exactly why before anything is attempted.
The blocker codes are exactly those `validate_proposal_apply` emits today:
`missing_acceptance` (status is not `Accepted`), `stale_model_revision`
(`prepared_against != current_model_revision`), and
`missing_revision_guard` (the batch lacks the prepared-revision guard). The
card also surfaces the validation's structured booleans
(`prepared_against_current_model`, `batch_revision_guard_matches`) as the
machine-checkable basis for the human-readable blockers. The GUI defines
no blocker code of its own; new codes are an engine change.

Commit path per control: Accept -> `review(accept)` + `apply`; Reject ->
`review(reject)`; Defer -> `review(defer)`. No control writes a shard
directly.

### 6.2 Assistant cards (Decision 006 card set, restated as context rules)

The assistant UI is the explicit card set from Decision 006 §"Assistant
UI Mechanics", each with its context rule:
- Context card — active project/projection/selection/model-revision/visible
  findings; always present.
- Query result card — read-only output + source query + revision (shown
  per query).
- Check card — check run, findings, checked revision, stale state.
- Proposal card — rationale, affected objects, batch summary, diff,
  assumptions, risks, stale/conflict state, actions (shown in `A4`).
- Apply card — approval path, commit result, transaction id, journal tip,
  invalidated derived state/artifacts (shown in `A7`).
- Handoff card — terminal/CLI/manual action (shown in `A8`).

Context rule (relationship-state guard, Decision 006): the assistant may
draft proposals that alter only authored `RelationshipKind`/authored
intent; it MUST NOT draft changes to resolver-derived status
(`Implemented`, `PendingImplementation`, `UnresolvedMismatch`,
`NotApplicableForVariant`). The Proposal card surfaces this distinction so
a supervisor never approves a derived-state edit.

### 6.3 Provenance panel (visible provenance — Decision 004)

A read-only panel reflecting committed-state provenance, sourced from the
engine `provenance.query` / `transactions.query` families. For any
selected committed object or transaction it shows actor, actor_type, tool,
session, acceptance path, model revision, and (for imported objects) the
`import_key`-keyed source provenance with unknown-basis markers. This is a
supervision-reflection surface: it proves "AI work must not appear as
unexplained geometry" (Decision 004) by making every change's origin
inspectable after the fact.

Committed-state source (read-only panels must state it, master §6.3): the
provenance panel reflects the engine `TransactionRecord` journal and
`Proposal` lineage; it never reads shards directly.

---

## 7. Proof Slices (§6.4)

Fixture default: the `datum-test` regression fixture
(`~/Documents/kicad_projects/Datum-eda/datum-test/`). Each slice names its
gates.

### 7.1 PS-AI-1 — Render a proposal as a reviewable ghost (SUPERVISION, ships first)

Smallest end-to-end proof that the marquee surface renders REAL committed
state:
1. Take a checked-in `Proposal` shard on `datum-test` (a bounded physical
   correction addressing one DRC finding; the Decision 004 first-scenario
   shape), prepared against the fixture's `model_revision`.
2. The GUI builds `ProposalReviewSceneV1` from the proposal + the engine's
   replay diff (the same diff `create_draft_proposal_from_batch` computes).
3. Ghosts render over the committed `BoardReviewSceneV1`: `ghost_add` for
   the proposed track, `ghost_modify_before/after` for any rerouted
   segment, `affected_emphasis` on the touched net.
4. The review card shows rationale, provenance (actor/tool/session/
   revision), diff summary, and the addressed finding fingerprint.
5. Read-only: no accept/reject yet. Proves render-from-proposal with zero
   GUI design authority.
Gates: golden VG-AI-1; determinism (byte-stable scene on unchanged
proposal+model); identity triple present on every ghost; zero shard
written.

### 7.2 PS-AI-2 — Accept a ghost into one journaled transaction (INTERACTIVE)

1. From PS-AI-1's rendered proposal, the supervisor accepts.
2. `validate_proposal_apply` passes; `review(accept)` + `apply` run;
   `apply_accepted_proposal` returns a `CommitReport` with one
   `TransactionRecord`.
3. Ghosts retire; base scene refreshes to the new `model_revision`;
   provenance now shows `applied_transaction_id` and acceptance path
   `gui_review`.
4. Undo (`journal undo` -> compensating batch) reverts it; reopen the
   project and the transaction is still in the journal (durable across
   reopen).
Gates: golden VG-AI-2 (post-accept state); interaction test exercising
§5.1 `S2 -> S4 -> S6`; durable-undo gate; exactly-one-`TransactionRecord`
assertion; reject-path companion test proving zero dirty shard on reject.

### 7.3 PS-AI-3 — Stale proposal is blocked in-canvas

1. Move the model revision (commit an unrelated edit) so the proposal is
   stale.
2. The Review machine enters `S3_StaleBlocked`; accept is disabled; the
   `stale_model_revision` blocker from `ProposalApplyValidation` renders
   at the accept control.
Gates: golden VG-AI-3 (stale-blocked card); interaction test for the
`S1 -> S3` path; assertion that accept is unreachable while stale.

### 7.4 PS-AI-4 — Terminal-agent proposal round-trip (Decision 005)

1. From the `AGENTS` dock, launch the existing context-aware hint; in the
   real PTY run `datum-eda proposal create ...` (the Decision 005 first
   slice).
2. The GUI detects the new `.datum/proposals/*.json` shard (`T2 -> T3`)
   and offers it to the Review machine.
3. Accept it through §5.1; provenance records the terminal session and
   executable/argv.
Gates: golden VG-AI-4 (terminal-originated proposal in review); assertion
that no design mutation occurred from terminal TEXT (only via the CLI's
journaled `proposal create`/`apply`); PTY-real gate (`QG-PTY-REAL`).

### 7.5 PS-AI-5 — Assistant draft-to-apply card flow (Decision 006, optional/gated)

1. Open the assistant panel; ask a read-only question (context + query
   cards).
2. Draft one proposal (`A3 -> A4`); it renders as ghosts via §5.1.
3. Approve via the explicit Apply card action (`A5 -> A6 -> A7`); apply
   card shows transaction id, journal tip, invalidated artifacts,
   assistant provenance.
Gates: golden VG-AI-5; assertion that apply was reachable only via the
explicit UI action (never from message text); assistant provenance on the
transaction. Gated by OQ on whether the assistant ships read-only+draft
first or with apply (Decision 006 OQ2).

---

## 8. Visual-Golden Acceptance (§6.5)

Harness: `crates/gui-render` (`visual_runner.rs`, `visual_manifest.rs`,
`visual_diff.rs`); bless + diff. Each surface is accepted only when its
golden renders a REAL `Proposal` from `datum-test` and its interaction
test passes.

| Golden | Surface | Driving fixture/scene | Acceptance |
|--------|---------|------------------------|------------|
| VG-AI-1 | Proposal rendered as ghosts (read-only) | `datum-test` + checked-in `Proposal`; `ProposalReviewSceneV1` over base | exact diff |
| VG-AI-2 | Post-accept committed state | `datum-test` after `proposal apply` | exact diff |
| VG-AI-3 | Stale-blocked review card | `datum-test` + stale `Proposal` | exact diff |
| VG-AI-4 | Terminal-originated proposal in review | `datum-test` + CLI-created `Proposal` | exact diff |
| VG-AI-5 | Assistant proposal/apply cards | `datum-test` + assistant draft | tolerance (text antialiasing in cards) |
| VG-AI-6 | Ghost render roles legend (add/remove/modify-before/after) | synthetic minimal proposal exercising all four roles | exact diff |
| VG-AI-7 | Provenance panel for a committed AI transaction | `datum-test` post-accept | exact diff |

Interaction tests (one per state machine): Review machine
(`S2->S4->S6`, `S2->S5->S7`, `S1->S3`), Assistant Card machine
(`A3->A4->A5->A6->A7`, stale `A9`), Agents/Terminal machine
(`T1->T2->T3->open_in_canvas`). A surface without a golden that renders
real committed `Proposal` state is not accepted (master §4.4).

Minimum meaningful coverage (master OQ3 / Decision 013 OQ4): a ghost
golden must render all `change_kind` values present in its proposal
(VG-AI-6 exists specifically to prevent passing on an add-only stub).

---

## 9. Non-Goals

- A built-in model provider, required chat UI, prompt text, or assistant
  personality (Decision 004/006 non-goals). The assistant bridge
  (`assistant_bridge.rs`) is provider-pluggable substrate, not a
  product-mandated model.
- The assistant as the primary AI architecture, a fake terminal, or the
  only route to core functionality (Decision 006). The real PTY +
  CLI/MCP proposal path is primary; the assistant is optional and
  dismissible.
- Parsing terminal shell text into design edits (Decision 005 hard ban).
- GUI-side editing of an `OperationBatch` or `Proposal` (the GUI renders
  and accepts/rejects; the engine authors). Granular/partial accept, if
  built, is the engine producing a narrowed proposal, not GUI batch
  surgery (OQ7).
- Auto-applying agent proposals without an explicit acceptance path,
  except under an owner-configured direct-commit-by-policy (Decision 004);
  no such policy is defined here.
- Persisting assistant chat transcripts as design authority (Decision 006:
  only proposals/transactions/artifacts/checks are project history).
- Multi-agent orchestration UI, agent scheduling, or hosted/cloud agent
  infrastructure (Decision 004 non-goals).
- Editing `specs/PROGRESS.md`, `specs/SPEC_PARITY.md`, `crates/`, or
  `mcp-server` from this spec.

---

## 10. Open Questions

1. **Accept granularity.** Does the engine ever partition an
   `OperationBatch` so a supervisor can accept a SUBSET of ghosts, or is
   whole-proposal accept the permanent model? (§5.1 granular path; current
   `proposal.rs` is whole-batch.) If partition is wanted, it must be an
   engine capability producing a narrowed re-prepared proposal.
2. **Accept == immediate commit, or staged review batch?** (Master OQ6.)
   Does an accepted ghost commit immediately as one transaction, or land
   in an undo-staged review batch the supervisor reverts as one unit
   before it becomes "real"? Affects the `S6_Applied` definition.
3. **Stale handling: auto-rebase vs. discard.** On `S3`, do we offer
   engine-side rebase+revalidate (re-prepare the batch against the current
   revision and recompute the diff) or force discard-and-redraft? Auto-
   rebase risks silently changing what the supervisor reviewed.
4. **Check-fail gate on accept.** Should accept be hard-blocked when
   `check_status == fail`, soft-blocked with an explicit override
   (recorded in provenance), or policy-driven? (Decision 004: no silent
   waivers.)
5. **Schematic ghosting parity.** This spec specifies board ghosts
   concretely; the schematic `ProposalReviewSceneV1` equivalent (wire/
   symbol/label ghosts) needs the same treatment once the schematic scene
   contract is settled. Which ships first?
6. **Which AI edits MUST be proposals vs. allowed direct commit?** (Master
   OQ5 / Decision 004.) Relationship-state changes, destructive deletes,
   imported-geometry repair, standards deviations, batch edits — the
   acceptance-path policy for each is unset.
7. **Assistant ship shape.** Read-only+draft first, or draft+apply in the
   first slice? (Decision 006 OQ2.) PS-AI-5 is gated on this.
8. **Ghost appearance language.** The contract carries `render_role`; the
   exact visual encoding (color/opacity/hatch per add/remove/modify) is a
   renderer + theme decision tied to the master dark-first-vs-dual-theme
   question (master OQ1). Must be legible on both committed-copper and
   empty board.
9. **Terminal-proposal detection mechanism.** How does the GUI learn a
   terminal agent created a proposal — filesystem watch on
   `.datum/proposals/`, a session-event poll
   (`context session-events`), or a socket push? Affects `T2 -> T3`
   latency and the `QG-PERFORMANCE-LATENCY` budget (master OQ8).
10. **Provenance for external-agent proposals.** When `actor_type ==
    ExternalAgent` (a terminal-launched `codex`/`claude`/`aider`), how
    much identity is captured beyond executable/argv + session, and what
    is shown in the provenance panel without overclaiming the agent's
    authorship?
11. **Engine exposure of the classified diff for a STORED proposal.**
    `Proposal` persists only the flattened `affected_objects`; the
    per-kind `TransactionDiff { created, modified, deleted }` and its order
    exist only transiently inside `create_draft_proposal_from_batch`'s
    preview commit (§4.2). The scene builder needs that classified diff for
    a proposal it did not just create (e.g. a CLI/terminal-originated or
    deferred proposal loaded from a shard). Does the engine add a
    `preview_proposal_diff(proposal_id) -> TransactionDiff` read (replays
    the stored batch on a clone), or must the GUI re-run the preview commit
    itself? A typed engine read is strongly preferred so the GUI carries no
    classification logic; this is a prerequisite for VG-AI-1.
12. **Which provenance fields the engine persists vs. reconstructs.** The
    `ProposalProvenanceBlock` (§4.4) names
    `actor`/`tool`/`session_id`/`acceptance_path` that are not all on the
    `Proposal` shard today. Are these added to the proposal/transaction
    records, or always reconstructed via `provenance.query` from the
    journal at render time? This sets whether the provenance panel is a
    cheap field read or a journal join, and bounds `unknown`-marker scope.
```
