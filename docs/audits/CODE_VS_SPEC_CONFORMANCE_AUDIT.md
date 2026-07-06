# Datum EDA — Code vs Spec Conformance Audit

Status: Refresh #2 (read-only audit; no code/spec/doc changed by this pass)
Date: 2026-06-22 (refresh of the 2026-06-22 refresh #1)
Snapshot commit: `465cc33` (committed + pushed; tree clean; gates green)
Tree state: `git status --short | wc -l` = 0 (CLEAN). Everything is now
committed across `720eb55` (worktree commit) / `41f157e` (Phase 2) /
`465cc33` (Phase 3, HEAD). All 10 drift gates pass; the parity gate is
green at committed state; 318 tests green. Every file:line citation below
describes COMMITTED code at 465cc33.

## Purpose

This audit re-measures the Datum EDA production codebase against its
two-layer specification — formal specs in `specs/` plus design docs in
`docs/` — AND the ratified product-mechanics direction
(`docs/DATUM_PRODUCT_MECHANICS.md` "Interop Boundary And Import Posture",
`docs/decisions/PRODUCT_MECHANICS_000..012`, `docs/contracts/`, and
`docs/audits/scope-integration/`). This pass is a DELTA against the
2026-06-22 refresh #1: for each prior finding it sets resolved / partial /
open / regressed / obsolete with current evidence, adds newly-landed
capability and new divergences, and re-rates each subsystem.

> **OBSOLETE (2026-06-23):** The Decision-013 "GUI supervision-reflection"
> track — the decision record, `specs/GUI_SPEC.md` + the `specs/gui/*` area
> spec set, and the supervision-reflection code — was a misdirection and has
> been reverted/removed. Every finding, gap, and priority in this snapshot
> that references supervision-reflection, Decision-013, or the GUI spec set is
> VOID. The GUI direction will be redefined from the owner's intent
> (interactive authoring the user can actually use).

The ratified "north": native-first AI-augmented EDA (schematic → PCB →
CAM); the native model + native format are the ONLY authority; KiCad
import is FROZEN and sufficient (lack of import breadth is NOT a defect);
the substrate (typed Operation + single commit()/journal + ProjectResolver
+ ObjectId=Uuid/ComponentInstance + model_revision + Import Map import_key)
is the foundation everything must converge on; private write paths are the
known transitional defect.

## Overall Verdict

ON-TRACK (up from PARTIAL). At committed HEAD `465cc33` the working tree is
clean, all 10 drift gates pass, and 318 tests are green. The prior audit's
single most load-bearing meta-defect — that nearly all substrate /
convergence progress was UNCOMMITTED worktree state at `22aeebe`, and the
green parity gate depended on an uncommitted SPEC_PARITY refresh — is fully
discharged. The native-first convergence thesis is now DURABLE, not
worktree-fragile.

All four mandated closures are confirmed with positive committed evidence
where they touch each subsystem:

1. global-label cross-sheet merge now keys on label NAME
   (`connectivity/mod.rs:283` `format!("global:{name}")`) with the
   prescribed VCC×3 + GND×3 same-count regression.
2. zone-fill DRC fab-trust hole closed — the single `Engine::run_drc` entry
   routes through `run_with_zone_fills_and_waivers`
   (`api/query_surface.rs:242`) projecting only `ZoneFillState::Filled`
   zones as copper, with a conservative empty fill-map for imported boards
   that under-trusts but never masks.
3. daemon non-journaled AI-write bypass closed at the PUBLIC surface — 0
   public tools dispatch to `NON_JOURNALED_DAEMON_WRITE_METHODS`
   (`tools_catalog_data.py:995`); bypassing `datum.board.*` aliases are
   hidden-but-dispatchable; journaled `datum.pcb.*` is the only public
   board-write surface, machine-enforced by `test_protocol_catalog.py:227`.
4. doc-orientation cleanup landed — CLAUDE.md / CANONICAL_IR §4 /
   AI_CLI_MCP_TOOL_SURFACE.md now affirm rather than deny the substrate;
   PROGRESS `passive_only_net` `[x]`→`[~]`; NATIVE_FORMAT §6.1/§8 and
   ERC_SPEC shipped-vs-target reconciled.

Three subsystems re-rated UP this cycle (canonical-ir PARTIAL→on-track;
board-drc-routing PARTIAL→on-track; doc-code-parity PARTIAL→on-track).
Substrate / native-authoring / manufacturing / mcp-cli-daemon /
gui-substrate / standards hold on-track; schematic-erc and library-pool
hold PARTIAL with positive trajectory; import-interop holds PARTIAL (frozen
lens). Residual defects are bounded and correctly classified as governance
/ plumbing / spec-hygiene rather than correctness — chiefly the unretired
non-journaled daemon dispatch island (off the public path and now
cross-language fenced), SPEC_PARITY inventory gaps for shipped slices, and
larger typed-domain convergence frontiers.

## Delta Since The 2026-06-22 Refresh #1

IMPROVED, broadly and durably.

### The dominant flip

The prior audit's central caveat — almost all progress uncommitted at
`22aeebe`, a fresh checkout would see almost none of it, the green parity
gate dependent on an uncommitted SPEC_PARITY refresh — is FULLY DISCHARGED.
HEAD = `465cc33`, clean tree, parity gate + all 10 drift gates green at
committed state. The risk profile shifted from "a fresh agent may
re-implement landed native-first work" (high) to "a fresh agent may
mis-judge commit status from stale doc framing" (low).

### Closed P0s (all four mandated)

- global-label net-merge P0 — FIXED & committed (41f157e). Cross-sheet
  global-label merge keys on label name, not union-root count
  (`connectivity/mod.rs:283`); VCC×3 + GND×3 same-count regression landed.
- zone-fill DRC fab-trust hole — VERIFIED CLOSED & committed (465cc33).
  Fill-aware projection pushed DOWN into the engine; the single run_drc
  path routes all consumers through it; only `ZoneFillState::Filled` zones
  project as copper; conservative empty fill-map under-trusts, never masks.
- daemon non-journaled AI-write bypass — CLOSED at the public surface &
  committed (465cc33), then tightened again in the active substrate pass. 0
  public journal-bypassing write tools; bypassing `datum.board.*` and
  `datum.replacement.apply_*` aliases hidden compatibility-only; public catalog
  295; journaled `datum.pcb.*` the only public board-write surface, with the
  hidden daemon write fence cross-language-gated.
- doc-orientation / specs — CLEANED & committed. CLAUDE.md /
  CANONICAL_IR / AI_CLI_MCP no longer deny the substrate; PROGRESS
  `passive_only_net` `[x]`→`[~]`; NATIVE_FORMAT §6.1/§8; ERC_SPEC
  shipped-vs-target reconciled.

### What else improved / is new

- CANONICAL_IR §4 fully rewritten from the never-existent
  dyn-Operation/OpDiff/Transaction{Instant} model to the enacted
  serde-tagged 121-variant Operation enum + OperationBatch + CommitDiff +
  commit/commit_journaled; the §7 determinism contradiction explicitly
  removed.
- New `schema_version` load gate (`source_shard.rs:117-130` +
  `project_resolver.rs:118-121`) hard-rejects unknown future versions,
  closing the prior §8 conflict; test-backed
  (`source_shard_metadata.rs:231`).
- The substrate-backed KiCad footprint importer crossed from
  untracked-worktree to committed+green.
- Content-addressed `pool/models/{role}/{sha256}` storage landed.
- The standards-aware CheckFinding gained `domain="standards"`,
  `standards_basis`, `rule_revision`, and `import_key` folded into the
  fingerprint.
- The full `specs/gui/` area-spec set + Decision-013 GUI
  supervision-reflection mandate landed.

### Regressed / newly-widened

- REGRESSED: none structural.
- RESOLVED since the orientation cleanup: the rewritten docs no longer frame
  the substrate as uncommitted, no longer list both fixed P0s as open, and the
  missed library substrate-denial sites have been corrected.
- NEW spec-ahead gap: Decision-013 GUI supervision-reflection is fully
  specified but has ZERO implementation (0 Supervision* types, 0
  supervision goldens, 0 net-new SessionCommands) — a clean spec-vs-code
  divergence that did not exist at the prior audit.

## How To Read This

- Each subsystem is classified on-track / partial / divergent, with
  conformance NOW vs PRIOR.
- Per prior finding: resolved / partial / open / regressed / obsolete, with
  current file:line evidence. "Resolved" requires positive evidence the fix
  is present; "still open" requires evidence it persists.
- Everything is committed at 465cc33 (clean tree); committed-vs-untracked
  no longer applies.
- Per-subsystem findings were adversarially verified; where a finder's own
  evidence was refuted by its verifier, the refuted item was dropped.

---

## Subsystem: Canonical IR & Native Format

Conformance now: ON-TRACK (prior: PARTIAL)

Headline: CANONICAL_IR §4 rewritten to the enacted serde-tagged 121-variant
Operation enum + OperationBatch + CommitDiff + commit/commit_journaled
(Instant/§7 contradiction removed); a new schema_version load gate
hard-rejects unknown future versions; §6.1/§8 reconciled. Open: board
native content is still CLI-owned `serde_json::Value` (engine gates the POOL
schema, not the BOARD schema, at load); `api::Design` vs
`substrate::DesignModel` not unified; project-create skeleton bootstrap
(allowed genesis boundary).

Prior-findings resolution:
- CANONICAL_IR §4 dyn-Operation trait + OpDiff + Transaction{Instant} the
  code never had — RESOLVED. §4 rewritten: `docs/CANONICAL_IR.md:126-144`
  defines `Operation` as a serde-tagged enum ("not a dyn trait", variant
  home `substrate/operation.rs`) + OperationBatch{batch_id,operations,
  provenance}; :155-159 CommitDiff{created/modified/deleted: Vec<ObjectId>}
  (matches `substrate/mod.rs:304-307`); :169-180 apply_operation /
  inverse_operations_for_batch + commit()/commit_journaled. All three §4
  example variants exist verbatim in code; enum = 121 serde-tagged variants
  (`operation.rs:6-7`, grown from 118). §7 contradiction closed: §4 states
  no wall-clock/Instant field is stored (:179-180); `grep Instant` over
  `substrate/` = 0.
- Native-format types CLI-owned and untyped (board side) — PARTIAL,
  unchanged. `command_project_native_types.rs:29,63-73` `NativeBoardRoot`
  still BTreeMap<String, serde_json::Value> for packages/pads/tracks/vias/
  zones/nets/net_classes; component-decoration layer typed. Pool-domain
  write-time gate intact (`pool_journal_ops.rs:315-394`). Engine owns/gates
  the POOL schema at load, NOT the BOARD schema.
- ProjectResolver does not gate unknown future schema_version — RESOLVED. A
  load gate now exists end-to-end: `source_shard.rs:117-127`
  validate_source_shard_schema_version (SUPPORTED=1; version>1 returns an
  EngineError) called at parse time (:32,:90); `project_resolver.rs:118-121`
  detects via is_unsupported_schema_version_error (:299-303) and RETURNS the
  error (hard load failure). NATIVE_FORMAT_SPEC §8 documents the mechanism.
  Test-backed: `source_shard_metadata.rs:231`
  resolver_rejects_future_native_source_shard_schema_version passes.
- §6.1 created/modified manifest timestamps conflict with §7 — RESOLVED.
  NATIVE_FORMAT_SPEC §6.1 (:199-206) now states the manifest carries no
  wall-clock timestamps per the §7 determinism invariant.
- Two parallel design models — OPEN. `api/mod.rs:58-65` `Design` still owns
  undo_stack/redo_stack Vec<TransactionRecord>, distinct from
  `substrate::DesignModel`. Not unified.
- §6.10 pool/models content-addressed tree — RESOLVED.
  `command_project_library.rs:1003` sha256 over model bytes; :1011 stores at
  `{pool}/models/{role}/{sha256}.{ext}` with provenance enforcement; typed
  Attach/DetachPoolPartModel ops (operation.rs:243,249).
- Project-create write_canonical_json bootstrap — OPEN, minor.
  `command_project_roots.rs:239-242` seeds four skeleton shards directly
  (genesis bootstrap; allowed boundary).
- REFUTED prior: route-proposal apply rewrites board.json off-model —
  OBSOLETE (production apply goes through the substrate).

What's implemented now:
- IR primitives unchanged and correct: deterministic recursive BTreeMap
  key-sort serialization (`ir/serialization.rs:15-21`); exact integer-nm
  units mil_to_nm=25_400 / inch_to_nm=25_400_000 (`ir/units.rs:14-25`);
  UUIDv4 native / UUIDv5 deterministic-import identity (`ir/ids.rs:6-23`).
- Operation enum at 121 serde-tagged variants; CommitDiff matches §4
  verbatim; schema_version load gate wired into all shard reads;
  content-addressed pool/models tree; forward-annotation apply through the
  substrate proposal gateway (`...forward_annotation_substrate.rs:127`).

Still open / divergent:
- Board native types remain CLI-owned untyped; the engine cannot gate the
  BOARD schema at load the way it now gates the POOL schema (the one
  substantive convergence gap in this subsystem).
- Two design models persist; api::Design vs substrate::DesignModel not
  unified.
- Project-create skeleton bootstrap (allowed genesis boundary).

Course guidance: Re-rate UP, PARTIAL→ON-TRACK. The prior headline
divergence (§4 describing a nonexistent dyn-Operation/OpDiff/Instant model)
is genuinely resolved variant-for-variant; two further prior-OPEN findings
also closed (schema_version load gate; §6.1 reconciliation). Remaining work
is bounded: (1) extend the engine-owned write-time schema gate from the POOL
domain to the BOARD domain; (2) unify api::Design vs substrate::DesignModel
undo/redo (low urgency); (3) leave project-create as the allowed genesis
boundary. IR primitives need no work.

## Subsystem: Substrate — Resolver, Identity, Commit, Journal

Conformance now: ON-TRACK (prior: ON-TRACK)

Headline: Strongest, most-converged subsystem; moved further. Single commit
primitive factored into `substrate/commit.rs` with the full ratified
ordering; new schema_version load gate wired into all shard reads
(test-backed); object_revision shard-persisted; proposal preview==commit
machinery with a hard post-commit assert. Open P2 governance: the
compatibility-only daemon dispatch arms are not yet routed through
commit_journaled.

Prior-findings resolution:
- PG-* proof-gate harness wired/machine-enforced — RESOLVED for current
  aggregate proof coverage. `scripts/run_migration_proof_gates.sh` now runs all
  ten named PG gates and is invoked by `scripts/run_drift_gates.sh`. The gates
  are backed by focused engine/CLI regressions for import identity, resolver
  recovery/dirty-state, journal hardening and durable undo/redo, shard-diff
  isolation, proposal preview/apply parity, Gerber/Excellon production
  projections, panel-projection isolation, variant population, artifact
  traceability, and harness self-wiring.
- No proposal preview==commit parity assertion — RESOLVED for direct code
  coverage and PG harness wiring.
  `preview_proposal_diff_journaled` stages proposal writes, updates preview
  source hashes, and cleans staging without promoting shards;
  `predict_journaled_transaction_id` uses the same staged source-hash path; and
  `apply_accepted_proposal` hard-asserts committed transaction_id == predicted.
  `proposal_predicted_transaction_id_matches_preview_and_apply_transaction`
  directly asserts preview after-revision, deterministic UUIDv5 prediction, and
  accepted apply transaction UUID all match.
- acceptance_path provenance field — RESOLVED BY RATIFICATION.
  `CommitProvenance` remains actor/source/reason only by design. Direct commit
  class is `CommitProvenance.source`; proposal-mediated acceptance is proven by
  an applied `Proposal` whose `applied_transaction_id` equals the durable
  `TransactionRecord.transaction_id`, with `Proposal.source` preserving the
  proposer origin and CLI/MCP `approval_path` remaining a surface contract.
- object_revision journal-replay-derived not shard-persisted — RESOLVED.
  object_revision_for() reads from shard JSON (`mod.rs:503-509`);
  PersistedComponentInstance serde field (`component_instance.rs:24`);
  round-tripped at hydrate (:41) and create (:338); revision-equality
  compared on reload (:460).
- GUI board_text bypass — OBSOLETE for substrate (resolved in GUI
  subsystem; substrate exposes only the journaled write path).
- Convergence-debt: daemon imports zero substrate — PARTIAL.
  `engine-daemon/src/dispatch.rs` still operates on the legacy Engine API
  and imports zero substrate, but the PUBLIC AI board-write surface no
  longer routes through these arms (datum.pcb.* journaled-only; bypassing
  datum.board.* aliases hidden). Off the public path; not yet retired.

What's implemented now:
- Single commit primitive in ratified order, factored into `commit.rs`:
  commit_journaled_with_links (`:70-122`) = journal-cursor guard → require
  expected_model_revision → revision-match guard →
  inverse_operations_for_batch → stage shard writes → clone+commit()
  (recompute model_revision) → set kind/undo_of/redo_of/inverse →
  append_transaction_journal → promote_staged_shard_writes →
  write_journal_cursor → swap. Base commit() at `:17-60`.
- New validate_non_empty_operation_batch guard in both commit paths
  (`commit.rs:18,78,125-133`).
- Deterministic transaction identity = UUIDv5(project_id,
  "datum-eda:transaction:{batch_id}:{model_revision}") (`commit.rs:36-43`).
- model_revision = sha256 over project_id + non-evidence shard pairs +
  sorted object id/revision/source_shard; evidence shards excluded.
- ComponentInstance genuinely populated via electrical↔physical join;
  ImportMapEntry keyed by import_key, read-only at resolve.

Convergence-debt: compatibility-only daemon dispatch arms not routed through
commit_journaled.

Course guidance: Stay the course; strongest, most-converged subsystem and
moved further. (1) acceptance_path is resolved by ratifying the
CommitSource/ProposalSource/applied_transaction_id mechanism. (4) Eventually
route the compatibility-only daemon arms through commit_journaled to retire the
dual write path entirely.

## Subsystem: Schematic Connectivity & ERC

Conformance now: PARTIAL (prior: PARTIAL) — positive trajectory

Headline: The P0 global-label merge is RESOLVED — keyed on label name
(`connectivity/mod.rs:283`) with the exact prescribed VCC×3 + GND×3
regression (11/11 connectivity + 16/16 ERC green); passive_only_net doc
honesty restored `[x]`→`[~]`; ERC_SPEC shipped-vs-target banners landed.
Held at PARTIAL by unchanged target-only type gaps now honestly bannered as
target.

Prior-findings resolution:
- (P0) Global-label merge keyed by root COUNT not name — RESOLVED &
  committed (41f157e). `connectivity/mod.rs:282-284` iterates `for (name,
  roots) in &global_label_groups_by_name` with `merge_key =
  format!("global:{name}")`; by-name map populated :184-188; consumed-root
  de-dup :295-310. Prescribed regression landed:
  `mod_tests_netinfo_basics.rs:284`
  distinct_global_labels_with_equal_root_counts_stay_separate (VCC×3 + GND×3
  stay separate). 11/11 connectivity + 16/16 ERC green.
- passive_only_net `[x]` but absent — RESOLVED (doc honesty).
  PROGRESS.md:506 and :1564 now `[~]` with accurate evidence; grep
  `passive_only` over crates/engine/src = 0; absent from the goldens
  required-codes corpus (`api/tests/goldens.rs`). Implementation remains
  target work; the divergence is closed.
- NetSemanticClass enum — OPEN. `infer_semantic_class` returns
  Option<String> (ground/power/None) (`connectivity/mod.rs:410-426`); no
  enum.
- ErcReport/SchematicLocation/typed code enum — PARTIAL. `erc/mod.rs:22-34`
  ErcFinding still `code: &'static str`, object-anchoring, no
  SchematicLocation. Now spec-reconciled: ERC_SPEC §6 (:132-157) declares
  the location-bearing form as target and documents the shipped ErcFinding
  shape. Silent divergence → declared target gap; type gap persists.
- Conflicting-labels-on-same-segment not detected — OPEN. `preferred_name`
  (`:716`) silently selects one; grep `conflict` = 0.
- Bus container/bus_entry data model ignored — OPEN; correctly frozen per
  ratified north.
- ERC rule-code taxonomy / waiver keys on code string — PARTIAL. Waiver
  keyed on code string (`erc/mod.rs:471`); ERC_SPEC §3.1/§4/§5/§6 now names
  the shipped `LibraryPinElectricalType:v1` taxonomy and current string-code
  result shape while retaining target result-shape gaps.
- PinElectricalType target-vs-shipped divergence — RESOLVED.
  `schematic::PinElectricalType` is now an alias to the pool-owned
  `LibraryPinElectricalType` ten-variant taxonomy; ERC canonicalizes all ten
  names and classifies OpenCollector/OpenEmitter/TriState/NoConnect directly.
  Remaining ERC work is evidence/result-shape depth, not enum ownership.
- WaiverTarget::Fingerprint a no-op stub in ERC — OPEN. `schematic/mod.rs`
  4-arm enum; `erc/mod.rs:491` `Fingerprint(_) => false`. Dead scaffolding.
- CheckDeviation + DeviationApprovalStatus::Accepted unconsumed by ERC —
  OPEN. `check_disposition.rs`; grep CheckDeviation in erc/ = 0.
- Native net/anon identity via import_uuid(namespace_kicad()); NetPinRef
  join on component:String — OPEN (`connectivity/mod.rs:327-339`).

New since last audit:
- The P0 fix and its prescribed same-count regression landed and committed.
- PROGRESS passive_only_net `[x]`→`[~]` with 0-grep evidence.
- ERC_SPEC gained shipped-vs-target banners across pin taxonomy / codes /
  matrix / result shape, reconciling several prior silent divergences.

Course guidance: PARTIAL holds with a clearly positive trajectory. The
dominant prior finding (the P0 global-label merge) is RESOLVED with the
exact prescribed name-keyed fix and same-count regression; the two prior
doc defects (passive_only_net; ERC shipped-vs-target) are both closed. What
keeps the subsystem at PARTIAL is the remaining set of target-only gaps
(no NetSemanticClass enum, no typed ERC code enum / SchematicLocation,
dead Fingerprint waiver, no conflicting-label detection) — now honestly
bannered as target. Next, none urgent: (1)
implement passive_only_net OR keep `[~]` as a tracked deferral; (2) the
ErcFinding typed-code / SchematicLocation promotion is a product question —
surface to the owner; (3) wire or remove the dead Fingerprint waiver arm;
(4) add a SPEC_PARITY/inventory row so the shipped-vs-target ERC slice is
machine-checked. Do NOT widen KiCad bus/hierarchy import (frozen).

## Subsystem: Board, DRC & Routing Kernel

Conformance now: ON-TRACK (prior: PARTIAL)

Headline: The sharp prior subsystem P0 (zone-fill fab-trust honesty hole) is
CLOSED — fill-aware projection moved into the engine; the single run_drc
entry routes all consumers through it projecting only Filled zones as copper,
with a bidirectional regression. WaiverTarget::Fingerprint is now live in
engine DRC; all 6 pad aperture codes present; the routing kernel grew to 170
read-only candidate strategies. Open items are convergence-debt, not
correctness.

Prior-findings resolution:
- ZoneFill honesty hole (the sharp subsystem P0) — RESOLVED & committed
  (465cc33). Fill-aware projection moved DOWN into the engine
  (`drc/zone_fill_projection.rs:20` run_with_zone_fills_and_waivers clones
  the board and replaces authored zones with only the copper from
  `substrate/zone_fill.rs:468-501` — a zone renders as copper ONLY if it has
  a ZoneFill entry AND state==`ZoneFillState::Filled`, else blocked).
  `Engine::run_drc` (`api/query_surface.rs:242-247`) — the SINGLE engine
  entry — routes all consumers through this path with a conservative empty
  fill-map (`&BTreeMap::new()`); daemon `dispatch.rs:467` and CLI inherit
  it. Native CLI feeds the REAL resolved model.zone_fills
  (`command_project_board_diagnostics.rs:58-66`). Bidirectional regression
  `mod_tests_zone_fill_projection.rs:75-103` asserts under-trust (empty map
  ⇒ connectivity_unconnected_pin) and honest-trust (Filled ⇒ cleared).
- Two of six pad process-aperture codes absent — RESOLVED.
  `drc/checks/mod.rs:389,497` both emitted; all six §11 codes present.
- WaiverTarget::Fingerprint dead in the engine DRC — RESOLVED. DrcViolation
  carries Option<String> fingerprint (`drc/mod.rs:39-40`);
  attach_drc_violation_fingerprints runs in run_with_waivers (:107);
  waiver_matches matches Fingerprint against drc_violation_fingerprint
  (:155-157). Live, not just the CLI bridge.
- Unified typed CheckRun/CheckFinding model absent in the engine API —
  PARTIAL. Engine `run_drc` / `run_erc_prechecks` still return raw
  DrcReport/ErcFinding (`drc/mod.rs:53-58`; `query_surface.rs:235`), but
  daemon dispatch `run_erc` and `run_drc` now wrap those raw results in live
  non-persisted `check_run_v1` envelopes with normalized findings and raw
  compatibility data under `raw_report.erc` / `raw_report.drc`. Substrate
  CheckFinding is committed and gained standards_basis/rule_revision/import_key
  (`check_run.rs:27,29,31`), and the native CLI bridge includes read-only
  `project query <root> erc` / `drc` as `check_run_v1` profile views.
- CheckFingerprint type absent — PARTIAL. No Rust newtype yet, but the lower
  engine DRC fingerprint now uses the versioned
  `datum-eda:drc-violation-fingerprint:v2` material and folds
  `standards_basis`, `rule_revision`, and `import_key` into the hash when a
  DRC producer supplies them. Standards-backed DRC producers now stamp
  `rule_revision="v1"` plus `datum.process_aperture_and_geometry.current`
  before fingerprinting, and focused DRC waiver coverage proves revision
  changes alter finding identity.
- Committed DRC not resolver-pinned — PARTIAL. `run_drc` operates on
  require_board() in-memory, no model_revision pin; the native
  `project query <root> drc` path resolves through ProjectResolver and emits a
  read-only `check_run_v1` profile view. The daemon-facing compatibility entry
  now returns a typed live envelope but remains unpinned because it is
  in-memory/import-session based. CHECKING_ARCHITECTURE_SPEC:42 is satisfied
  for the native CLI substrate path; imported in-memory daemon sessions remain
  compatibility-only for resolver pinning.
- DRC rule set hard-coded; no CheckProfile param — OPEN.
  `dispatch.rs:465-472` takes an explicit RuleType list; not
  profile-parameterized.
- explain_violation by (domain,index) not fingerprint — RESOLVED for daemon
  dispatch. `explain_violation` now accepts `fingerprint` and resolves it
  against the current live CheckRun finding set, while `index` remains accepted
  for legacy positional callers.
- Routing kernel — RESOLVED & grew. 71 route_path_candidate* source files;
  170 distinct candidate fn definitions; pure read-only generators; apply
  through the substrate proposal gateway. The most-built part of the
  subsystem.

What's implemented now: zone-fill-aware DRC as the single engine path with a
conservative empty fill-map for imported boards (under-trusts, never masks);
all 7 DRC rules with deterministic content-derived IDs; all 6 §11 pad
aperture codes; engine-side Fingerprint waivers; 170 read-only routing
candidate strategies; route-proposal apply through ProjectResolver +
apply_accepted_proposal with provenance + model_revision pin.

Still open (convergence-debt): engine API entry not resolver-pinned /
profile-parameterized; daemon dispatch wraps raw engine results in typed live
`check_run_v1`; daemon explain_violation is fingerprint-aware with positional
fallback; real general zone-fill solver absent
(derive_zone_fills defaults Unfilled, so imported real pours project empty
copper).

Course guidance: PARTIAL→ON-TRACK. The fab-trust hole is genuinely closed
with a bidirectional regression; the conservative empty-fill-map can only
over-report, never mask — ratify it as the normative imported-board DRC
posture. Remaining work is convergence-debt: (1) keep native project checks on
ProjectResolver + model_revision pin + CheckProfileRef while treating
in-memory/import-session daemon checks as live `check_run_v1` compatibility;
(2) the lower DRC fingerprint now includes rule revision/import-key slots when
available; (3) the general copper-pour solver remains the one
functional gap that lets the native projection==export oracle pass on real
pours. Keep the routing kernel on course.

## Subsystem: Native Authoring & Operation Model

Conformance now: ON-TRACK (prior: ON-TRACK)

Headline: Both operation-model P0s RESOLVED and now committed: the GUI
board-text private writer is deleted (replacement emits journaled CLI
strings) and the public AI write-path bypass is closed and test-enforced;
forward-annotation apply is on the proposal gateway. The native write path
has genuinely converged on commit_journaled with CommitSource::Cli.

Prior-findings resolution:
- (P0) GUI board-text off-model private writer — RESOLVED & committed.
  `board_text_mutations.rs` + `board_text_field_values.rs` absent from disk
  AND git; replaced by `gui-app/src/board_text_terminal_commands.rs:114`
  which only emits `datum-eda project edit-board-text` CLI strings (no
  auto-execute). The CLI target commits via Operation::SetBoardText →
  commit_board_layout_operation (`command_project_board_layout.rs:361-384`:
  ProjectResolver::resolve → expected_model_revision guard →
  commit_journaled, CommitSource::Cli).
- (cross-domain) MCP/daemon AI mutations bypass commit() — RESOLVED at the
  public surface & committed. `NON_JOURNALED_DAEMON_WRITE_METHODS`
  (`tools_catalog_data.py:995-1001`) drives a public/hidden alias partition;
  0 public tools dispatch to a non-journaled write; bypassing datum.board.*
  aliases hidden compatibility-only; enforced by
  `test_protocol_catalog.py:227`. Residual (owned by MCP/daemon subsystem):
  the legacy daemon dispatch arms physically persist, non-journaled, off the
  public path.
- Forward-annotation apply bypass — RESOLVED.
  `command_project_forward_annotation_substrate.rs:19,38,55,85` builds an
  OperationBatch; :126 commit_proposal_metadata_journaled; :127
  apply_accepted_proposal (which itself commits via commit_journaled).
- No MoveSymbol/RotateSymbol/MirrorSymbol; no library/symbol op families —
  PARTIAL (corrected). Pool-library Create/Set/Delete + Attach/Detach
  (operation.rs:230-256) and schematic-symbol Create/Set/Delete
  (operation.rs:551-561) op families landed with apply + inverse. NOTE:
  dedicated CLI transform verbs DO exist and are journaled
  (`project move-symbol/rotate-symbol/mirror-symbol`,
  cli_args_project_commands.rs:384-388 →
  command_project_schematic_symbol_mutations.rs:112-166); they lower to
  Operation::SetSchematicSymbol whole-symbol replacement. Only dedicated
  Operation ENUM VARIANTS named Move/Rotate/MirrorSymbol are absent.
- Imported-KiCad parallel non-journaled write model (Engine::save text-patch
  s-expressions) — OPEN, correctly deferred. `api/save_kicad.rs:13-39`
  std::fs::write, gated to ImportKind::KiCadBoard only; frozen-import
  posture, not a native-authoring defect.
- Project-create write_canonical_json bootstrap — OPEN, minor (genesis
  boundary; `command_project_roots.rs:239-242`).
- REFUTED prior: route-proposal apply rewrites board.json off-model —
  OBSOLETE (the board.json write at command_project_route_proposal.rs:898 is
  a fixture seeder; real apply goes through the substrate).

What's implemented now: 121-variant serde-tagged Operation enum; single
substrate commit gateway for native board mutation; GUI board-text via a
journaled CLI-string conduit (zero private writers); forward-annotation on
the proposal gateway; schematic-symbol + pool-library op families with
apply+inverse; public AI write surface journaled-only and machine-enforced;
drift gate `check_schematic_private_writers.py` wired (run_drift_gates.sh:8).

Still open: dedicated Move/Rotate/MirrorSymbol op-enum variants (verbs exist;
product decision whether to add variants vs ratify Set-replacement);
Engine::save imported-KiCad text-patch (frozen, deferred); project-create
genesis bootstrap (allowed); legacy daemon dispatch arms (de-publicized,
owned by MCP/daemon).

Course guidance: Stay the course (ON-TRACK), now committed rather than
worktree-only. The two operation-model P0s are resolved with positive
committed + test-enforced evidence. Remaining items are bounded and correctly
deferred: (1) owner decides dedicated transform op-enum variants vs ratify
Set-replacement; (2) Engine::save text-patch stays deferred under frozen
import — do NOT advance import work; (3) the legacy daemon dispatch arms,
now de-publicized, should route through commit_journaled or be documented as
a read/legacy island. Highest-value spec work was rewriting CANONICAL_IR §4
to match the enacted enum+commit/journal model — done.

## Subsystem: Manufacturing / CAM Export

Conformance now: ON-TRACK (prior: ON-TRACK)

Headline: BOM/PnP keyed by component_instance_uuid; unified
manufacturing-set export with --include/--job/--variant scoping +
ArtifactMetadata traceability; OutputJob is the only journaled mutation (the
daemon has zero manufacturing dispatch — read-only-projection architecture);
the zone-fill solver is materially richer (v2/v3 bounded rectangular
cutouts). Open: general copper-pour solver, drill fab-completeness,
ArtifactMetadata gate-9 identity fields, collapsing per-format public verbs.

Prior-findings resolution:
- BOM/PnP keyed by reference:String — RESOLVED. Logic relocated to
  `command_project/command_project_inventory.rs`; NativeBomRow::identity_key()
  returns component_instance_uuid with package_uuid fallback (:39-43);
  identical PnP impl (:106-114); CSV header leads with component_instance_uuid
  (csv.rs:11,39); parser accepts new + legacy headers. reference is a
  diff_fields() drift field, not the join key.
- Single export verb with --include fan-out — RESOLVED. parse_manufacturing_
  include / ArtifactKind projection; ManufacturingSetScope.include + variant +
  output_job_id; export gates each artifact on projection membership
  (`command_project_manufacturing.rs:336-339`); --output-job/--job
  inheritance.
- No metadata persisted for bom/pnp/drill; split read path — RESOLVED.
  manufacturing_set_artifact_metadata (`scope.rs:198-235`) aggregates
  per-file sha256 into one ArtifactMetadata via a single ProjectResolver read;
  struct carries model_revision/output_job/variant/generator_version/per-file
  sha256/validation_state (`substrate/artifact.rs:146-160`).
- Operation::SetOutputJob update primitive — RESOLVED. Create/Set/Delete
  OutputJob ops + inverses (`production_journal_ops.rs:109-128`); participate
  in commit()+journal with model_revision keying.
- No real zone-fill solver — PARTIAL, advanced. zone_fill_copper_projection_
  zones projects only Filled islands; solver now handles bounded rectangular
  obstacle cutouts, keepouts, non-orthogonal track bounds, same-net solid fill,
  and thermal-relief pass-through when no same-net pad/via anchors need spoke
  geometry. derive_zone_fills default still Unfilled; no general clearance
  subtraction/antipads/thermal-anchor generation/arbitrary multi-island.
- No plated/non-plated NC separation, no G85; legacy drill.csv — OPEN. Both
  {prefix}-drill.csv and {prefix}-drill.drl emitted
  (`command_project_manufacturing.rs:341-362`); render_excellon_drill
  (`excellon.rs:6-47`) is one combined file, no split, no G85.
- Excellon METRIC,TZ + explicit-decimal coords, no FMAT,2 — OPEN
  (`excellon.rs:15`). TZ zero-suppression is moot under explicit decimals;
  formally imprecise but fab-tolerated.
- Per-format/per-layer export+validate+compare subcommands remain public —
  OPEN. 29 export-family variants still public
  (`cli_args_project_commands.rs:154-206`).
- (cross-domain) MCP/daemon manufacturing mutations bypass commit() —
  RESOLVED. dispatch.rs has zero manufacturing/gerber/drill/output-job
  entries; OutputJob is the only journaled mutation; export is a non-commit
  derived projection.

What's implemented now: RS-274X Gerber copper export (externally verified
correct); ZoneFill-honest copper projection (only Filled islands render as
copper); Excellon .drl NC drill; ComponentInstance-keyed BOM/PnP with variant
fitted-state filtering; OutputJob create/set/delete via commit()+journal;
ArtifactMetadata traceability aggregation; unified manufacturing-set export
with --include/--output-job/--job/--variant/--prefix scoping and
validate/compare/manifest/inspect oracle verbs.

Still open: general copper-pour solver (the one functional gap — imported
real pours project empty copper under the Unfilled default); drill
fab-completeness (plated/non-plated split, G85 slots, drop legacy drill.csv,
fix Excellon FMAT,2/coordinate-format); ArtifactMetadata missing 000F gate-9
identity fields (generated timestamp, source plan/board revisions,
revision-level output-job/variant); 29 per-format public CLI verbs not
collapsed behind --include.

Course guidance: Hold ON-TRACK. The two required closures touching this
subsystem hold: zone-fill DRC fab-trust verified closed (empty fill-map
under-trust; only Filled islands project copper); daemon has zero
manufacturing dispatch (OutputJob is the only journaled mutation). Priority
against decision 000F + the manufacturing contract: (1) general copper-pour
solver; (2) drill fab-completeness + Excellon header; (3) extend
ArtifactMetadata to the 000F gate-9 identity set; (4) cosmetic — hide the 29
per-format public verbs behind --include. Panel/assembly correctly deferred
(M8); do NOT flag import-path gaps (frozen).

## Subsystem: Library & Pool System

Conformance now: PARTIAL (prior: PARTIAL) — now committed, positive
trajectory

Headline: Both prior BLOCKING divergences RESOLVED and committed (typed pool
DomainObjects across 8 resolver subdirs; full journaled
Create/Set/Delete/Attach/Detach pool mutation with a write-time schema gate;
content-addressed pool/models storage; daemon exposes pool reads only). Held
at PARTIAL by single-aperture Padstack, exact-equality parametric search,
absent IPC footprint generation, absent SPEC_PARITY pool row, and
decision-008 still draft-for-owner-review with no supersede banner vs the
Horizon model — the gating blocker.

Prior-findings resolution:
- Library objects NOT DomainObjects; pool a side HashMap — RESOLVED &
  committed. read_pool_ref_shards walks 8 subdirs
  (units/symbols/entities/parts/packages/footprints/padstacks/pin_pad_maps)
  at `project_resolver.rs:307-325`, types each shard domain=pool and stamps
  object.kind=subdir (:396-409).
- No proposal/journal path for library mutation — RESOLVED & committed.
  Typed Create/Set/Delete/Attach/Detach ops in
  `substrate/pool_journal_ops.rs` (apply :17-114, materialize :129-151,
  inverse :175-262); CLI commits via OperationBatch + commit_journaled
  through ProjectResolver (`command_project_library.rs`). Daemon has NO
  library-write bypass (read-only search_pool/get_part/get_package; legacy
  assign_part/set_package flagged NON_JOURNALED + hidden).
- decision-008 native primitives absent — OPEN. grep
  struct/enum Footprint/PinPadMap/LibraryBinding/StandardsBasis/ApprovalState/
  ProvenanceSet across crates = 0. Footprint/PinPadMap exist only as resolver
  subdir kinds + CLI JSON mutation kinds, gated as untyped serde_json::Value
  by validate_pool_library_object. FOCUS answered YES: decision-008 native
  types remain JSON-only.
- ENGINE_SPEC §1.2/§1.1a Part/Package/Model fields — RESOLVED.
  `pool/mod.rs` (589 lines): Part carries manufacturer_jep106 /
  packaging_options / behavioural_models / thermal / supply_chain_offers;
  Package body_height_nm; ModelRole/ModelAttachment/ThermalSpec/SupplyOffer.
- Entire IPC footprint system — OPEN (grep ipc-7351/footprint-generation = 0
  functional; only the board-level ProcessAperture DRC rule).
- Padstack single-aperture + scalar drill — OPEN. `pool/mod.rs:88-94`
  Padstack { aperture: Option<PadstackAperture>, drill_nm: Option<i64> }.
- pool/models content-addressed storage, FTS, unit-aware search — PARTIAL.
  Content-addressed storage RESOLVED & committed
  (`command_project_library.rs` sha256, pool/models/<role>/<sha256> + gc).
  FTS/unit-aware parametric search absent; PoolIndex only
  part_tags/part_parametric (`pool/mod.rs:442-448`); search_parametric is
  exact-string equality (:551-576).
- Two incompatible library models; ratify decision-008 vs M0 Horizon — OPEN.
  decision-008 still "draft for owner review" with 6 open Owner Questions; no
  supersede banner in LIBRARY_ARCHITECTURE / POOL_ARCHITECTURE; code extended
  the M0 Horizon model.
- No LibraryBinding / ComponentInstance join — PARTIAL. ComponentInstance
  join exists & committed, keyed on (reference, part)
  (`component_instance.rs:57-58`); NOT the decision-008
  LibraryBinding/ProvenanceSet object (grep=0).

What's implemented now: typed pool DomainObjects across 8 subdirs; full
journaled library mutation surface with write-time schema gate; Part/Package
enriched to §1.2; behavioral-model attachment with content-addressed storage;
ComponentInstance (reference,part) join; public datum.library.* AI write
surface through CLI commit_journaled; daemon exposes only pool reads.

Still open: decision-008 native Rust types (JSON-only); IPC footprint
generation; multi-layer Padstack; standards_basis/approval_state;
FTS/unit-aware search; SPEC_PARITY pool/library row; the unreconciled
Horizon-vs-008 contradiction.

Course guidance: PARTIAL holds; the DIVERGENT→PARTIAL flip is now committed
and durable. (1) P1 — Owner ratifies/reconciles decision-008 vs the Horizon
model and answers the 6 Owner Questions BEFORE more library type work; ~1300
lines of pool/library code now ship against an unreconciled draft — the
gating blocker. (2) Once ratified, promote footprints/pin_pad_maps to typed
DomainObjects and introduce LibraryBinding/StandardsBasis/approval_state. (3)
Decide IPC-footprint scope or banner spec-ahead; multi-layer Padstack. (4)
FTS/unit-aware parametric search and a SPEC_PARITY pool/library inventory row.

## Subsystem: MCP / CLI / Daemon Surface

Conformance now: ON-TRACK (prior: ON-TRACK)

Headline: 295 public datum.* tools across 17 classes with the
ok/schema/context/result envelope; 0 public tools dispatch to the 19-method
NON_JOURNALED frozenset; datum.board.* and datum.replacement.apply_* aliases
hidden-but-dispatchable;
journaled datum.pcb.* the only public board-write surface, machine-gated by
`test_protocol_catalog.py:227`. SPEC_PARITY green (mcp=186/cli=271/
daemon=54). The residual defect is convergence-debt: the daemon dispatch
source island persists, but cross-language drift is now fenced.

Prior-findings resolution:
- 7-class datum.* surface 0% exposed; envelope empty — RESOLVED & committed.
  Public tools/list = 298 (verified by executing the catalog); 17 distinct
  datum.* classes. Envelope built in `stdio_tool_host.py` _datum_target_
  envelope / _datum_error_envelope (ok:false). datum.context.get bridges to
  CLI `context get` (ProjectResolver-backed).
- (P0) MCP mutations bypass commit(); dual write surface — RESOLVED at the
  PUBLIC surface & committed. The 15 non-journaled write methods are
  enumerated in `NON_JOURNALED_DAEMON_WRITE_METHODS`
  (`tools_catalog_data.py:995-1001`); their flat tools AND datum.board.*
  aliases partition into COMPATIBILITY_TOOL_SPECS (dispatchable, hidden);
  public TOOLS = DATUM_TOOL_SPECS + _PUBLIC_CANONICAL_ALIAS_SPECS
  (:1014-1024). Executed catalog: 0 public tools dispatching to a
  non-journaled write; 0 public datum.board.* or datum.replacement.apply_*;
  19 hidden daemon-write aliases reachable
  via TOOL_BY_NAME. Enforced by test_protocol_catalog.py:227
  test_no_public_write_tool_bypasses_the_journaled_commit_path (11/11 catalog
  tests OK). Journaled datum.pcb.* is the only public board-write surface.
- daemon dispatch.rs imports zero substrate; write arms call legacy engine.*
  with no journaling — PARTIAL. grep substrate/commit_journaled/ProjectResolver
  in `engine-daemon/src/dispatch.rs` = empty; legacy write arms at :36,:42,
  :49,:56-57,:99-100,:119-120,:129-130,:253-254,:269-270. The P0 closure
  happened in the Python catalog (hidden), NOT by rewriting dispatch. The
  non-journaled island exists and is reachable via hidden compat aliases /
  direct socket, but `scripts/check_daemon_write_parity.py` now parses the Rust
  dispatch source and diffs it against `NON_JOURNALED_DAEMON_WRITE_METHODS`,
  and the full drift gate invokes it.
- open_project = engine.import() keeps daemon import-centric — PARTIAL.
  `main.rs:245-246` open_project = engine.import(); no context/proposal/
  commit arm on the socket path; native commit surface lives only in the CLI
  bridge.
- explain_violation by (domain,index) not fingerprint — RESOLVED for daemon
  dispatch. `fingerprint` is accepted and preferred; `index` remains
  compatibility-only.
- Naming eda→datum-eda — RESOLVED (`server_runtime.py:40`,
  `stdio_tool_host.py:31` serverInfo name 'datum-eda').
- Parity counts — OBSOLETE. Gate GREEN at grown committed counts:
  mcp_runtime_methods=186, cli_project_commands=271, daemon_dispatch_
  methods=54 (`SPEC_PARITY.md:24-29`); check_spec_parity.py passes (7
  inventories).
- AI_CLI_MCP_TOOL_SURFACE.md denies the substrate — RESOLVED. Rewritten to
  AFFIRM the substrate (:44-48 "has substantially LANDED ... all exist in
  crates/engine/src/substrate/"); honestly records the daemon non-journaled
  residue, the daemon write-fence gate, and the remaining ~13 classified
  `write_canonical_json` sites.

What's implemented now: 295 public datum.* tools (17 classes); the
ok/schema/context/result + ok:false envelope; decision-004 Private Mutation
Ban enforced at the public-surface layer (0 public tools dispatch to a
non-journaled write), regression-gated; the hidden compatibility fence is
cross-language-gated against Rust daemon dispatch; journaled datum.pcb.* is the
only public board-write surface; datum.context/query/check/proposal/journal/
library/component_instance families CLI-bridged through the substrate;
SPEC_PARITY green.

Still open: `dispatch.rs` imports zero substrate and its 15 write arms call
legacy non-journaled engine.* mutators (closed at the catalog layer, not the
Rust source); daemon remains import-centric. `explain_violation` is no longer
positional-only at the daemon/MCP boundary because fingerprint dispatch is
accepted with index fallback.

Course guidance: Hold ON-TRACK; the subsystem improved. 0 PUBLIC
journal-bypassing write tools at committed HEAD, machine-gated. The pointed
question is answered conservatively: dispatch.rs is HIDDEN, not routed
through commit — the closure happened entirely in the Python catalog
partition, not in the Rust daemon. The cross-language drift gate now closes the
one residual P0-adjacent fencing risk. Next, in order: (1) route the 15
dispatch.rs write arms through commit_journaled OR document the daemon as a
read/legacy island; (2) tidy the minor AI_CLI_MCP staleness. Do NOT add new
flat legacy daemon write tools or re-promote a datum.board.* write alias.

## Subsystem: GUI Substrate

Conformance now: ON-TRACK (prior: ON-TRACK)

Headline: The prior P0 private off-model board-text writer is resolved AND
committed; the drift gate is green; the M7_FRONTEND_SPEC 1.7/1.8
read-only-mandate conflict is superseded by master GUI_SPEC §8. The center of
gravity shifted to a clean spec-ahead gap: Decision-013 supervision-reflection
is the named FIRST GUI deliverable, fully specified but entirely unbuilt.

Prior-findings resolution:
- (P0/BLOCKING) GUI write-capable private off-model writer — RESOLVED &
  committed. board_text_mutations.rs / board_text_field_values.rs absent from
  disk AND git; replaced by `board_text_terminal_commands.rs:114` which only
  generates `datum-eda project edit-board-text` CLI strings (no auto-execute).
  CLI target commits via Operation::SetBoardText →
  commit_board_layout_operation (ProjectResolver::resolve + commit_journaled +
  revision guard + CommitProvenance{Cli}).
- Drift gate fences gui-protocol private writes — RESOLVED & committed.
  `scripts/check_schematic_private_writers.py` runs EXIT 0; retired-file
  assertions + FORBIDDEN_GUI_BOARD_TEXT_PATTERNS fence re-entry; the prior
  unrelated generated-export trip no longer fires.
- M7_FRONTEND_SPEC 1.7/1.8 read-only mandate — RESOLVED at the controlling
  level. master GUI_SPEC.md §8 ("Retiring the M7 Review Spike Framing")
  supersedes §1.7/§1.8 "apply/commit ... Not supported" and governs on scope
  conflict. RESIDUAL (downgraded): M7_FRONTEND_SPEC header still "Active
  opening specification" and 1.7 still carries stale "Not supported" text
  with no inline supersede banner.
- BoardTextPrimitive uses text_uuid not source_object_uuid — OPEN.
  `gui-protocol/src/lib.rs:114,139` declare `text_uuid` while siblings use
  source_object_uuid. The new supervision spec pins the contract field to
  source_object_uuid and bridges at projection time but explicitly defers the
  code rename.
- Surviving demo/state raw writers — OPEN. `gui-protocol/src/lib.rs:1994`
  write_json_file (demo fixture); `gui-app/src/main.rs:3250`
  save_assistant_config (.datum/assistant.json); terminal context/session/
  activity snapshots. All non-board-mutation GUI-local-state; unfenced;
  latent, low-risk.
- gui-app does not depend on eda-engine — OBSOLETE (ratified-intentional;
  gui-app reaches the substrate only by emitting CLI command strings per
  PRODUCT_MECHANICS_005).

What's implemented now: BoardReviewSceneV1 typed serde scene contract with
the identity triple; journaled CLI-command-prefill board-text edit conduit;
committed private-writer drift gate; visual-regression harness (bless +
exact/tolerance diff, 4 checked-in text goldens); Outputs/production GUI lane
using the same conduit with zero private board writes; append-only terminal
hand-off audit-event stream; all three GUI crates compile clean.

Still open / divergent:
- BoardTextPrimitive.text_uuid vs the source_object_uuid sibling convention
  (supervision spec defers the rename).
- M7_FRONTEND_SPEC header / 1.7 stale "Not supported" text — superseded only
  by reference in master GUI_SPEC §8.
- Unfenced GUI-local-state raw writers (latent re-entry).
- NEW spec-ahead gap: the entire GUI_SUPERVISION_REFLECTION (Decision-013,
  the named FIRST GUI deliverable) is UNBUILT — 0 Supervision* types across
  all GUI crates, 0 of 9 supervision goldens, 0 companion contracts, 0 of the
  section-5 net-new read-only SessionCommand arms (HoverObject/SelectFinding/
  SelectJournalEntry/SelectRelationship/SetActiveVariant/RefreshModel).

New since last audit: the full `specs/gui/` area-spec set landed (GUI_SPEC
master + 7 area specs + PRODUCT_MECHANICS_013); GUI_SPEC §8 supersedes
M7_FRONTEND_SPEC 1.7/1.8 at the controlling level; the durability flip (all
prior GUI progress now committed at 465cc33); the clean Decision-013
spec-ahead gap.

Course guidance: Stay the course; GUI substrate holds ON-TRACK and is now
durable (committed). The write path is compliant; the center of gravity is
the clean spec-ahead gap. Priority order: (1) begin the
GUI_SUPERVISION_REFLECTION mandatory floor — start with the pure-projection
journal/activity ledger (R12) and resolver-status/recovery (R13), depending
only on already-resolved DesignModel fields; (2) land the section-5 net-new
read-only SessionCommand arms carrying NO OperationBatch, plus the PS-SR-2
read-only invariant test; (3) rename text_uuid → source_object_uuid (or
ratify the exception) before the supervision index lands; (4) add the inline
supersede banner to M7_FRONTEND_SPEC. Build NO edit/commit path into the
supervision surfaces — area 1 is read-only by definition (Decision-013
level-1 parity).

## Subsystem: Standards & Compliance

Conformance now: ON-TRACK (prior: ON-TRACK)

Headline: Two of three substantive prior findings advanced. Standards
findings now carry `domain="standards"` and import_key (RESOLVED); named
standards-basis keying is wired end-to-end with rule_revision +
standards_basis + import_key folded into the fingerprint (basis-keying
OPEN→PARTIAL→PARTIAL+TYPED-V1); proposal-first repair with clean claim
discipline; zone-fill honesty as a second standards basis; SPEC_PARITY now has
a `standards_check_surface` row for the shipped standards slice. Open:
broader StandardsRegistry schema and ENGINE_SPEC batch-1 inline banners.

Prior-findings resolution:
- Standards check rule-keyed not StandardsBasis-keyed — PARTIAL (advanced).
  Named-basis keying wired: CheckFinding.standards_basis (`check_run.rs:27`);
  per-code basis assignment (`command_project_check_finding_identity.rs:23-39`:
  process-aperture → datum.process_aperture_and_geometry.current; zone-fill →
  datum.zone_fill_honesty.current); CheckRunProfileBasis.standards_basis +
  profile-aware coverage rows now mark shipped DRC `clearance_copper` and
  `silk_clearance_copper` as evaluated/filtered rather than stale
  not-implemented placeholders (`command_project_check_run_view.rs:218-273`);
  regression (`main_tests_project_check_profiles.rs:149-220`). Generated
  CheckRun evidence now also carries typed v1 `standards_basis_detail` on
  profile basis, coverage rows, and findings with basis_id, registry_entry_ref,
  basis_kind, disposition, selection scope, and provenance while preserving the
  flat basis-id string for compatibility. The current process-aperture and
  ZoneFill honesty details resolve through an engine-owned v1 check
  standards-basis registry seam instead of CLI-local fallback constructors; the
  broader project-authored StandardsRegistry schema and full decision-010
  registry object model remain deferred.
- CheckFinding lacks domain/import_key — RESOLVED. finding_domain() routes all
  standards-profile codes to domain="standards"
  (`command_project_check_run_view.rs:715-718`); import_key is a CheckFinding
  field (`check_run.rs:31`), populated from payloads
  (`command_project_check_finding_identity.rs:45-51`), folded into the
  fingerprint alongside standards_basis + rule_revision (:96-117). Note: spec
  §5.2 prose says domain is normally `Standards` (capitalized); code uses the
  lowercase token "standards" consistent with the erc/drc/manufacturing
  convention — a casing convention, not a contract gap.
- ENGINE_SPEC §1.1a-1.3 batch-1 types lack inline stub banners — OPEN. grep
  stub/spec-only/implementation-deferred/batch-1 in ENGINE_SPEC = 0.
  Spec-hygiene item, not a code gap.
- Process-aperture DRC + proposal-first repair + clean claim discipline —
  RESOLVED. generate_native_project_standards_repair_proposals() builds
  drafts via create_draft_proposal_from_batch
  (`command_project_standards_repairs.rs:8,319`); only mutation path is the
  journaled OperationBatch+proposal. grep certified/fabrication-approved/
  regulatory-approval in non-test code = 0. Repair surface canonical
  (datum.check.repair_standards).
- Batch-1 schema types (StandardsRegistry/etc.) — OPEN (grep=0; correctly
  Planned, not pulled forward).
- SPEC_PARITY row for the shipped standards slice — RESOLVED.
  `standards_check_surface` freezes CheckFinding standards fields, standards
  finding codes, standards repair/check CLI/MCP markers, zone-fill query
  exposure, and `SetCheckRun`.

What's implemented now: CheckFinding carrying domain/rule_id/standards_basis/
rule_revision/import_key + sha256 fingerprint; standards-aware fingerprint
including all of those fields; per-code standards-basis keying; standards
CheckProfile lens distinguishing evaluated vs not-yet-implemented families;
all standards-profile findings routed to domain="standards"; proposal-first
standards repair across pads/tracks/vias; canonical datum.check.repair_standards
MCP tool, CLI-bridged; zone-fill honesty as a standards basis (only Filled
zones project as copper — conservative, never masks); regression coverage.

Still open: broader StandardsRegistry typed schema and full decision-010
registry integration (correctly deferred); ENGINE_SPEC inline stub banners;
import_key emission idle on native findings (slot exists, exercise is
import-side, correctly idle).

Course guidance: Stay the course (ON-TRACK, improved). Two of three
substantive prior findings advanced (domain/import_key RESOLVED; basis-keying
OPEN→PARTIAL with typed v1 evidence and an engine-owned v1 basis-registry seam now present on CheckRun payloads). The zone-fill DRC fab-trust closure is verified at the
standards seam (only Filled zones project as copper; unfilled/missing
surfaced under datum.zone_fill_honesty.current — under-trusts, never masks).
Next, in priority order: (1) add the SPEC_PARITY inventory row for the shipped
slice; (2) add ENGINE_SPEC inline stub banners; (3) when broader standards
work warrants, connect the typed CheckRun basis detail to a project-authored
StandardsRegistry rather than only the current engine-owned check-basis registry
seam. Do NOT pull forward deferred batch-1 schema/MCP stubs.

## Subsystem: Import / Interop (Frozen Lens)

Conformance now: PARTIAL (prior: PARTIAL) — positive trajectory

Headline: The footprint substrate-import slice crossed from
untracked-worktree to COMMITTED+GREEN (footprint.rs substrate-backed; CLI
commits DomainObjects via commit_journaled then re-resolves to verify); the
prior "import/ has zero substrate references" assertion is now provably
false; PROGRESS.md:50-51 document the landed converter. Frozen posture
honored — no breadth added.

Prior-findings resolution:
- Import bypasses the substrate entirely — PARTIAL. RESOLVED for footprints &
  now COMMITTED. `import/kicad/footprint.rs:8` uses substrate
  ImportKey/ImportMapEntry/allocate_import_identity (:36 allocates stable
  identity). CLI `command_project_imports.rs:36-123` resolves via
  ProjectResolver, builds DomainObjects (AddProjectPoolRef :63,
  CreatePoolPadstack :74, CreatePoolPackage :80, CreateImportMapShard :85),
  commits via commit_journaled with CommitProvenance{Cli} (:103-115), then
  re-resolves to verify resolver-visibility (:118-123). grep -rl substrate
  over import/ now returns footprint.rs + its test. STILL OPEN for
  boards/schematics: `api/save_kicad.rs:13-39` text-patches s-expressions;
  import_board_document / import_schematic_document carry zero substrate;
  correctly deferred per frozen posture.
- .ids.json identity sidecar path dead — OPEN. IdSidecar struct + merge/
  restore (`ids_sidecar.rs:20-129`) referenced only in itself + tests;
  production uses only compute_source_hash_*; doubly obsolete under the
  substrate ImportMap.
- decision-011 Core Primitives absent — PARTIAL. ImportProvenance/
  ImportSession/LossinessRecord/MigrationProposal/ImportAuditFinding/
  InteropArtifact grep=0. EXCEPTION: RelationshipKind::BoardOnly /
  ::ReverseEngineered exist (`relationship.rs:150,152`) but the importer does
  not stamp them. Correctly not pulled forward.
- ImportMapEntry lacks status field — OPEN. `substrate/mod.rs:264-276` carries
  import_key/object_id/source_shard_id/source_tool/source_path/
  source_object_ref/source_hash; no status field. decision-011 core
  requirement satisfied: import_key (not source_hash) is the durable reuse key
  (footprint reuse keys on import_key via allocate_import_identity).
- Eagle design import unimplemented — OPEN (correct per frozen posture; only
  Eagle .lbr pool synthesis exists).

What's implemented now: substrate-backed KiCad footprint converter committed
at 465cc33; import_key as the durable read-only provenance/reuse key;
reused_existing_identity gating idempotent re-import; committed test coverage
(identity reuse, drilled-padstack fidelity, journal records for all four op
kinds, undo emptying the import_map); ImportMap as the live identity surface
(journaled .datum/import_map/*.json shards excluded from model_revision).

Still open: ImportMapEntry status field; decision-011 Core Primitives;
importer stamps no provenance/relationship records; dead IdSidecar machinery;
boards/schematics still text-patch s-expressions (correctly deferred).

Course guidance: PARTIAL holds with positive trajectory; the converter slice
moved from untracked-worktree to COMMITTED+GREEN, discharging the prior
critical caveat for this subsystem. Stay the frozen posture: do NOT add
KiCad/Eagle breadth and do NOT penalize the absent board/schematic
substrate-import. Next, only when the native-authoring frontier permits: (1)
when import is next exercised, attach ImportProvenance + stamp RelationshipKind;
(2) add the ImportMapEntry status field per decision-011; (3) retire the dead
IdSidecar struct; (4) re-route board/schematic import onto the
_with_import_map + commit_journaled pattern the footprint converter proves.
Treat the footprint path as the proof template, not a prompt for breadth.

## Subsystem: Doc/Code Parity (Meta)

Conformance now: ON-TRACK (prior: PARTIAL)

Headline: All four structural prior defects resolved: tree clean + committed
(uncommitted-at-22aeebe hazard gone); all 10 drift gates + parity gate green
at committed state; CLAUDE.md / CANONICAL_IR §4 / AI_CLI_MCP rewritten to
affirm the substrate; passive_only_net honestly `[~]`; parity-delta report
carries a SUPERSEDED banner. The later orientation-cleanup staleness is now
closed: CLAUDE.md and AI_CLI_MCP no longer frame the substrate as
uncommitted-at-22aeebe, fixed P0s are not listed as open, and the remaining
private-writer count is normalized to ~13.

Prior-findings resolution:
- Green parity status depends on the uncommitted SPEC_PARITY refresh; ~all
  progress uncommitted at 22aeebe — RESOLVED. Tree clean (`git status
  --short` = 0); committed across 720eb55/41f157e/465cc33.
  check_spec_parity.py exits 0 (9 inventories) with grown counts including
  standards_check_surface and pool_library_surface.
  The "depends on uncommitted refresh" hazard is gone.
- CLAUDE.md denies ProjectResolver/commit/journal/ComponentInstance —
  RESOLVED. CLAUDE.md:48-56 affirms the substrate "has substantially landed";
  no residual substrate-denial in CLAUDE.md.
- AI_CLI_MCP_TOOL_SURFACE.md denies the substrate / internally contradictory
  — RESOLVED. Rewritten to AFFIRM (:44-48,:312), no longer carries the
  uncommitted-at-22aeebe framing, and normalizes the remaining classified
  `write_canonical_json` debt to ~13.
- passive_only_net falsely `[x]` — RESOLVED. PROGRESS.md:506 and :1564 flipped
  `[x]`→`[~]` with honest 0-grep evidence; grep passive_only over
  crates/engine/src = 0.
- CANONICAL_IR §4 dyn-Operation/OpDiff/Instant root — RESOLVED. §4 rewritten
  to the enacted serde-tagged enum + commit/commit_journaled (no-Instant
  explicit).
- DOC_CODE_PARITY_DELTA_REPORT.md stale 75/182 — RESOLVED. Carries
  "HISTORICAL SNAPSHOT (2026-05-25) — SUPERSEDED" banner (:3).
- check_progress_coverage + check_schematic_private_writers wired — RESOLVED.
  Both committed and wired into run_drift_gates.sh; `bash run_drift_gates.sh`
  runs all 10 gates GREEN (318 tests OK, exit 0).
- PROGRESS substrate-readiness table — RESOLVED. Still the live source of
  truth, native-first headline.
- CLAUDE.md / AI_CLI_MCP private-write site counts — RESOLVED. Both now report
  the remaining classified `write_canonical_json` debt as ~13 rather than the
  stale ~115 figure.
- ComponentInstance struct location — RESOLVED (substrate present, committed).

New since last audit:
- Orientation-cleanup staleness is closed. CLAUDE.md and AI_CLI_MCP now frame
  the substrate as committed/current, fixed P0s are not listed as open,
  SPEC_PARITY is described as an inventory gate rather than an uncommitted
  hazard, and AI_CLI_MCP no longer carries the stale ~115 private-writer count.
- The missed library substrate-denial sites are closed.
  LIBRARY_AUTHORING_TOOL_CONTRACT now states that the ComponentInstance +
  commit/journal substrate exists and that the remaining work is the
  library-specific binding operation family above it.
- POSITIVE beyond the brief: CANONICAL_IR §4 was fully rewritten, not merely
  the orientation lines — a larger doc-parity gain than enumerated.

Still open: SPEC_PARITY lacks dedicated rows for the shipped standards-aware
DRC slice and the pool/library Operation/CLI/MCP surface.

Course guidance: PARTIAL→ON-TRACK. All four structural prior defects are
resolved at committed HEAD. The orientation cleanup is now resolved as well.
The prior PG-* harness doc-vs-gate gap is closed by
`scripts/run_migration_proof_gates.sh` and its full-drift wiring. Remaining
meta work is machine-visibility, not false orientation: add SPEC_PARITY rows
for shipped standards-aware DRC and pool/library surfaces.

---

## Cross-Cutting Findings

1. DURABILITY FLIP (the dominant positive theme). The prior audit's central
   caveat — almost all convergence work uncommitted at 22aeebe, a fresh
   checkout would see almost none of it, the green parity gate dependent on an
   uncommitted SPEC_PARITY refresh — is fully discharged at 465cc33. Clean
   tree; all 10 drift gates + parity gate green at committed state. The
   native-first convergence thesis is now durable, not worktree-fragile.

2. SINGLE COMMIT/JOURNAL GATEWAY IS REAL FOR THE PUBLIC SURFACE. Native
   authoring (board layout, board-text, schematic-symbol, forward-annotation,
   pool/library mutation, footprint import) all route through
   ProjectResolver::resolve → commit_journaled with
   CommitProvenance{CommitSource::Cli}; the public MCP write surface is
   journaled datum.pcb.* only. The operation model has genuinely converged.

3. RESIDUAL TRANSITIONAL DEFECT — the non-journaled daemon dispatch island.
   `engine-daemon/src/dispatch.rs` imports zero substrate and its 15 legacy
   write arms remain non-journaled. The P0 closure happened in the Python
   catalog (hidden from public tools/list), NOT by rewriting the Rust source.
   The hidden-set and the daemon source are now protected by
   `scripts/check_daemon_write_parity.py`, which parses Rust dispatch arms and
   diffs them against `NON_JOURNALED_DAEMON_WRITE_METHODS`.

4. SPEC-AHEAD HONESTY, IMPROVING BUT UNEVEN. CANONICAL_IR §4, NATIVE_FORMAT
   §6.1/§8, ERC_SPEC, CLAUDE.md, AI_CLI_MCP, and the library contract now carry
   honest shipped-vs-target framing for the substrate. Remaining spec-ahead
   concern is narrower: decision-008/010/011 carry normative-shaped types with
   no inline not-implemented banner. The prior PG-* proof-gate wiring claim is
   now backed by `scripts/run_migration_proof_gates.sh` and full-drift
   invocation.

5. TYPED-DOMAIN-OBJECT CONVERGENCE GAP — POOL gated, BOARD not. The engine now
   owns and gates the POOL schema at load (validate_pool_library_object) but
   BOARD native content remains CLI-owned serde_json::Value with no engine
   load-time gate. The same uneven typing recurs in decision-008/010/011
   native primitives (Footprint/PinPadMap/LibraryBinding/StandardsBasis/
   ImportProvenance) which exist only as JSON, not Rust types — correctly
   deferred under owner-review/frozen posture but a consistent convergence
   frontier.

6. LEGACY vs TYPED CHECK MODEL DUALITY. RESOLVED for daemon dispatch:
   `run_erc` and `run_drc` now return live non-persisted `check_run_v1`
   envelopes with normalized CheckFinding-shaped rows and raw compatibility
   payloads under `raw_report`. The lower in-memory engine API still returns
   raw DrcReport/DrcViolation and ErcFinding (code: &'static str, no
   SchematicLocation), so full engine API type convergence remains a smaller
   follow-on.

7. MACHINE-VISIBILITY GAPS in SPEC_PARITY. RESOLVED for the shipped
   standards-aware DRC slice and pool/library Operation/CLI/MCP surface:
   `standards_check_surface` and `pool_library_surface` now freeze those
   marker sets. ERC shipped-vs-target machine visibility remains a smaller
   follow-on if a dedicated inventory is still desired.

8. FROZEN-IMPORT POSTURE HONORED throughout. No import breadth added; the
   footprint converter is the proof-template, not a prompt for breadth;
   board/schematic substrate-import correctly deferred. No drift toward
   import-as-authority.

## Course Corrections

| Pri | Area | Action |
|-----|------|--------|
| P1 | MCP/daemon — dispatch island | RESOLVED for drift fencing. `scripts/check_daemon_write_parity.py` parses non-journaled `api/write_ops` mutators, detects daemon dispatch arms that call them, diffs the result against `NON_JOURNALED_DAEMON_WRITE_METHODS`, and is invoked by `scripts/run_drift_gates.sh`. Residual convergence-debt remains: the legacy daemon arms still exist and are hidden/compatibility-only rather than routed through `commit_journaled`. |
| P1 | Substrate / Doc-parity — PG-* proof-gate harness | RESOLVED. `scripts/run_migration_proof_gates.sh` now runs all ten named PG gates and is wired into `scripts/run_drift_gates.sh`; direct execution passed on 2026-06-26. |
| P1 | Doc/Code Parity (Meta) — stale orientation framing | RESOLVED. CLAUDE.md, AI_CLI_MCP, and LIBRARY_AUTHORING now affirm the committed substrate, fixed P0s, ~13 remaining classified JSON-write sites, daemon write-fence gate, and library-specific binding gap without stale uncommitted framing. |
| P1 | Library & Pool — decision-008 ratification | Owner ratifies/reconciles decision-008 vs the M0 Horizon model and answers its 6 open Owner Questions BEFORE further library type work; flip Status to ratified or add a supersede banner. ~1300 lines of pool/library code now ship against an unreconciled draft — the gating blocker for promoting footprints/pin_pad_maps to typed DomainObjects. |
| P1 | Board, DRC & Routing — copper-pour solver | Build the general copper-pour solver (clearance subtraction, antipads, thermals, arbitrary multi-island) so derive_zone_fills can move beyond the Unfilled default; the one functional gap that lets the native projection==export oracle pass on real pours. Until then the conservative under-trust posture is correct and should be ratified as normative imported-board DRC behavior. |
| P1 | GUI substrate — Decision-013 supervision-reflection | VOID/OBSOLETE. The audit banner marks Decision-013, the GUI spec set, and supervision-reflection priorities as reverted misdirection. Do not implement this row; redefine GUI work from the owner's interactive-authoring intent. |
| P2 | Substrate / Board DRC — engine onto typed model | Native `project query <root> drc` now uses ProjectResolver + model_revision pin + DRC CheckRun profile and returns `check_run_v1`; daemon `run_drc` now returns a live non-persisted `check_run_v1` envelope with raw DrcReport under `raw_report.drc`, daemon `explain_violation` now accepts finding fingerprints, and the lower DRC fingerprint folds standards basis, rule revision, and import-key slots into its versioned hash material. Remaining work: move the lower engine API itself onto resolver-pinned typed CheckRun profiles rather than raw compatibility reports. |
| P2 | Spec parity machine-visibility | PARTIAL RESOLVED. `standards_check_surface` and `pool_library_surface` rows now guard the shipped standards-aware DRC and pool/library Operation/CLI/MCP surfaces. ERC shipped-vs-target remains optional follow-on machine visibility. |
| P2 | Manufacturing / CAM — drill + ArtifactMetadata | Add plated/non-plated NC separation + G85 route/slot, drop the legacy drill.csv second format, fix the Excellon header (FMAT,2 / coordinate-format); extend ArtifactMetadata to the 000F gate-9 identity set. Collapse the 29 per-format public CLI verbs behind --include. |
| P2 | Native authoring / Substrate — design-model unification & lineage | Unify api::Design hand-rolled undo/redo with substrate::DesignModel; acceptance_path lineage is ratified as the existing CommitSource + ProposalSource + applied_transaction_id triple; standalone preview==commit parity coverage now calls the journal prediction path directly. |

## Stay The Course

- Substrate commit/journal gateway — single commit() in the ratified
  capture-inverse → stage → recompute-revision → append-journal → promote →
  write-cursor order (now factored into substrate/commit.rs); model_revision
  excludes evidence shards; ImportMap read-only at resolve; schema_version
  load gate rejects unknown future versions. Strongest subsystem; keep
  extending breadth onto it.
- IR primitives — deterministic BTreeMap key-sort serialization, exact
  integer-nm units (externally verified), UUIDv4 native / UUIDv5
  deterministic-import identity. Correct and unchanged.
- GUI renderer, BoardReviewSceneV1 scene contract, and visual regression
  harness (bless + exact/tolerance diff, 4 checked-in goldens) — intact.
- Routing kernel — 170 deterministic route_path_candidate strategies,
  read-only/native-first; route-proposal apply through ProjectResolver +
  apply_accepted_proposal with provenance + model_revision pinning.
- Gerber RS-274X export — %FSLAX46Y46*% / %MOMM*% / %LPD*% with raw-nm coords
  (1 nm LSB), G36/G37 regions, D03 flashes, C/R apertures — externally
  verified correct.
- Zone-fill-aware DRC as the single engine path — only Filled zones project
  as copper; the conservative empty fill-map for imported boards under-trusts
  but never masks a violation. Ratify as normative imported-board DRC posture.
- Frozen import posture — the footprint substrate-import slice is the correct
  converter template (one-time, UUIDv5 identity, read-only import_key). Do NOT
  add KiCad/Eagle breadth.
- Proposal-first discipline — standards repair, route apply, and
  forward-annotation all build Draft proposals via
  create_draft_proposal_from_batch and never mutate directly; claim discipline
  clean.
- Public AI write surface journaled-only — datum.pcb.* is the only public
  board-write surface; 0 public tools dispatch to a non-journaled write;
  machine-enforced.
- PROGRESS substrate-readiness table — the one doc artifact NOT understating
  progress; keep it as the live source of truth and point prose at it.

## Spec Adjustments Needed

| Spec | Change | Pri |
|------|--------|-----|
| `PRODUCT_MECHANICS_000/000D/001` (PG-* proof-gate wiring) | RESOLVED. `scripts/run_migration_proof_gates.sh` exists, runs all ten named PG gates, and is invoked from `scripts/run_drift_gates.sh`; direct execution passed on 2026-06-26. | P1 |
| `docs/contracts/AI_CLI_MCP_TOOL_SURFACE.md` (:45, :349, :493) | RESOLVED. The stale uncommitted-at-22aeebe framing is absent, the private-writer count is normalized to ~13, and the current text records that the public daemon write-bypass is closed by the daemon write-parity gate. | P1 |
| `CLAUDE.md` (:53, :89-92, :99-101) | RESOLVED. CLAUDE.md frames the substrate as committed/current, removes the misleading SPEC_PARITY commit warning, and lists both former P0s as fixed. | P1 |
| `docs/contracts/LIBRARY_AUTHORING_TOOL_CONTRACT.md` (:74, :161) | RESOLVED. The contract now affirms that the ComponentInstance + commit/journal substrate exists and scopes remaining work to the library-specific binding operation family. | P1 |
| `docs/decisions/PRODUCT_MECHANICS_008` + `LIBRARY_ARCHITECTURE.md` / `POOL_ARCHITECTURE.md` | Decision-008 still "draft for owner review" with 6 unresolved Owner Questions while ~1300 lines of pool/library code ship against the Horizon model; the two models are unreconciled. Owner ratifies/supersedes decision-008 and adds a relationship banner to the Horizon-model docs identifying the on-disk authority. | P1 |
| `specs/SPEC_PARITY.md` + `spec_parity_manifest.json` | PARTIAL RESOLVED. `standards_check_surface` now guards CheckFinding standards fields, standards finding codes, standards repair/check CLI/MCP markers, zone-fill query exposure, and `SetCheckRun`; `pool_library_surface` now guards pool/library Operation variants, ProjectCommands variants, and public `datum.library.*` tools. ERC shipped-vs-target can still get a dedicated row later if needed. | P2 |
| `CHECKING_ARCHITECTURE_SPEC.md` (ProjectResolver pinning :42 + CheckRunRequest profile contract) | PARTIAL RESOLVED for the native CLI substrate path and daemon dispatch: `project query <root> erc` / `drc` route through ProjectResolver + model_revision + CheckRun profiles, while in-memory daemon `run_erc` / `run_drc` now return live non-persisted `check_run_v1` envelopes with raw compatibility payloads. Also ratify the conservative empty-fill-map / Filled-only-projects-copper posture as normative imported-board DRC behavior (under-trust over false-pass) with the zone-fill-solver gap as the parked follow-up. | P2 |
| `docs/CANONICAL_IR.md` §4 (note) / §8 + `specs/NATIVE_FORMAT_SPEC.md` (board schema) | §4 rewrite is done and correct; add a note that board-domain schema gating is deferred to a future slice (the pool-domain write-time gate shipped first) so the spec does not imply uniform engine ownership of the board schema today. §8 still overstates a SQLite/FTS pool index with parametric tables vs the implemented exact-string-equality index — banner as spec-ahead or trim (defer to Library/Pool owner). | P2 |
| `specs/ENGINE_SPEC.md` §1.1a-1.3 (batch-1 type blocks) | Spec-only batch-1 shared enum/reference-type blocks carry no inline deferral banner (honesty lives only in PROGRESS). Add an inline "Batch 1 stub — spec-only, implementation deferred" banner at the head of each block. | P2 |
| `specs/M7_FRONTEND_SPEC.md` (header + 1.7/1.8) + decision-010/011 banners | Add an inline supersede banner to M7_FRONTEND_SPEC ("superseded by GUI_SPEC §8; scene-contract/identity rules still authoritative") and annotate 1.7 "Not supported: apply/commit" to point at the sanctioned conduit. Add inline spec-ahead/not-implemented banners to decision-011 Core Primitives and the ImportMapEntry status field, and a v1-staging note in STANDARDS_COMPLIANCE_SPEC §5.2 that a flat basis-id String is an acceptable v1 staging of the typed StandardsBasis (domain tokens lowercase: Standards == "standards"). | P2 |

## Risk Areas

- Cross-language hidden-set drift: closed by `scripts/check_daemon_write_parity.py`
  and full-drift wiring. Direct-socket access to the legacy non-journaled write
  arms still remains as compatibility convergence-debt.
- Stale orientation docs causing mis-judgment of commit status: closed for the
  identified CLAUDE.md, AI_CLI_MCP, and library-contract sites. Remaining meta
  risk is missing SPEC_PARITY inventory rows for shipped slices, not false
  uncommitted-status framing.
- Imported-board DRC under-trust surfacing spurious findings: derive_zone_fills
  defaults to Unfilled and the engine uses an empty fill-map, so imported
  boards with real copper pours will report spurious connectivity/clearance
  findings until a general zone-fill solver or imported-fill ingestion exists.
  Correct-by-default (never masks) but a UX/accuracy gap that should be
  ratified as normative.
- Decision-008 unratified while code accretes: ~1300 lines of pool/library
  code ship against the M0 Horizon model while decision-008 remains
  draft-for-owner-review with 6 open questions and no supersede banner.
- Spec-ahead types treated as shipped: decision-010 StandardsBasis,
  decision-011 Core Primitives, and ENGINE_SPEC batch-1 blocks carry
  normative-shaped definitions with no inline not-implemented banner (grep=0
  in code).
- Legacy vs typed check-model duality: RESOLVED for daemon dispatch.
  `run_erc` / `run_drc` return live `check_run_v1` envelopes while preserving
  raw DrcReport/ErcFinding payloads under `raw_report`; only the lower
  in-memory engine API remains raw for compatibility tests and imported-session
  internals.
- Decision-013 supervision-reflection spec-vs-code divergence: the named FIRST
  GUI deliverable is fully specified with zero implementation; reading the
  spec as shipped would mis-scope the GUI frontier.

## Limitations

- Read-only audit at committed HEAD `465cc33` (clean tree, `git status
  --short` = 0); no code/spec/doc was modified by this pass. This is a
  conductor synthesis of twelve independently-produced and adversarially-
  verified per-subsystem findings, each carrying its own upheld/refuted/
  corrections verdict at HEAD 465cc33.
- Load-bearing facts confirmed directly this pass: HEAD = 465cc33; working
  tree clean; the four mandated closures at code level
  (connectivity/mod.rs:283 name-keyed merge; api/query_surface.rs:242
  run_with_zone_fills_and_waivers + empty fill-map;
  tools_catalog_data.py:995 NON_JOURNALED_DAEMON_WRITE_METHODS).
- The full 318-test suite and all 10 drift gates were NOT re-run in this
  synthesis pass; test-greenness is reported on the strength of the
  per-subsystem verifications (which ran targeted suites — e.g. connectivity
  11/11, ERC 16/16, catalog 11/11) plus the refresh brief.
- Minor citation drift exists in some inherited findings (e.g. LIBRARY_
  AUTHORING denial site :161 not :269; the "298 public catalog" count is
  predicate-enforced, not a hardcoded test assertion; the daemon "11 classes"
  figure is actually 17) — noted in the source verdicts; none alters any
  conformance rating.
- External-standard correctness (e.g. Excellon FMAT,2 / coordinate-format
  formality, Gerber RS-274X structure, nm-unit arithmetic) was assessed from
  the inherited findings, not exhaustively re-derived this pass.
