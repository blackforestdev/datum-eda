# Datum EDA — Code vs Spec Conformance Audit

Status: Refresh (read-only audit; no code/spec/doc changed by this pass)
Date: 2026-06-22 (refresh of the 2026-06-20 audit)
Snapshot commit: `22aeebe` (UNCHANGED since the prior audit)
Worktree volume: `git status --short | wc -l` = 453 entries
(~268 modified + ~181 untracked). HEAD has NOT advanced; essentially ALL
of the progress documented here is UNCOMMITTED worktree state. The entire
`crates/engine/src/substrate/` module (49 files) is untracked `??`. A fresh
checkout of HEAD would see almost none of this. All file:line citations
below describe worktree state and are labeled committed-vs-untracked where
load-bearing.
Concurrency note: Codex is still actively authoring `crates/`; this is a
moving-target snapshot. One drift-gate state and several line numbers drifted
during the audit window. Re-verify any specific file:line before acting.

## Purpose

This audit re-measures the Datum EDA production codebase against its
two-layer specification — formal specs in `specs/` plus design docs in
`docs/` — AND the ratified product-mechanics direction
(`docs/DATUM_PRODUCT_MECHANICS.md` "Interop Boundary And Import Posture",
`docs/decisions/PRODUCT_MECHANICS_000..012`, `docs/contracts/`, and
`docs/audits/scope-integration/`). This pass is a DELTA against the
2026-06-20 audit: for each prior finding it sets resolved / partial / open /
regressed / obsolete with current evidence, adds newly-landed capability and
new divergences, and re-rates each subsystem.

The ratified "north": native-first AI-augmented EDA (schematic → PCB → CAM);
the native model + native format are the ONLY authority; KiCad import is
FROZEN and sufficient; the substrate (typed Operation + single
commit()/journal + ProjectResolver + ObjectId=Uuid/ComponentInstance +
model_revision + Import Map import_key) is the foundation everything must
converge on; private write paths are the known transitional defect.

## Overall Verdict

PARTIAL, with a clearly positive trajectory. Since the 2026-06-20 audit the
substrate continued to win: it remains the strongest, most-converged
subsystem, and two previously-blocking subsystems flipped to ON-TRACK (GUI
write path DIVERGENT→ON-TRACK; native authoring PARTIAL→ON-TRACK).
Manufacturing made the single largest leap (PARTIAL→ON-TRACK, 4 of 5 prior
findings resolved). The Library/pool domain moved DIVERGENT→PARTIAL by
routing all native library mutation through the typed Operation +
commit/journal substrate. The MCP/CLI surface moved PARTIAL→ON-TRACK by
landing the entire ratified `datum.*` surface (~240 names + normalized
envelope) that the prior audit rated 0% exposed.

CRITICAL CAVEAT: essentially all of this progress is UNCOMMITTED worktree
state at HEAD `22aeebe`. Two of the three prior P0s are resolved-in-worktree
(GUI board_text; daemon-bypass resolved for the NEW native path only). The
third prior P0 — the schematic global-label merge keyed by root-count not
name — is byte-for-byte STILL OPEN and untouched.

The dominant theme is no longer "is the substrate real" (it is) but "docs
deny the substrate that ships" and "two write surfaces / two check paths
coexist." No subsystem shows drift toward import-as-ongoing-authority; import
is correctly frozen, and effort flows into the native substrate.

## Delta Since The 2026-06-20 Audit

### Improved

1. Substrate — `object_revision` is now shard-persisted and reload-asserted
   (prior minor divergence RESOLVED); proposal preview==commit parity
   machinery landed in production code (`predict_journaled_transaction_id` +
   post-commit equality assertion); `ComponentInstance` moved from scaffolded
   to genuinely populated via an electrical↔physical join; substrate roughly
   doubled (~14k→~20.7k lines) with new variant/relationship/zone_fill/
   check_run modules.
2. GUI — the BLOCKING private board-text writer is fully retired
   (`board_text_mutations.rs` + `board_text_field_values.rs` deleted; replaced
   by a journaled CLI-command-prefill conduit; a purpose-built drift gate
   fences re-entry; pattern ratified verbatim in PRODUCT_MECHANICS_001).
3. Native authoring — forward-annotation apply migrated off
   `write_canonical_json` onto the substrate proposal gateway; two new
   substrate-native Operation families (pool-library, schematic-symbol
   Create/Set/Delete) with apply+inverse; `operation.rs` grew to a 118-variant
   serde-tagged enum.
4. Manufacturing — BOM/PnP now keyed by ComponentInstance UUID (not reference
   string); `--include`/`--output-job`/`--job`/`--variant` fan-out;
   `Operation::SetOutputJob` with undo/redo; `ArtifactMetadata` persisted for
   bom/pnp/drill via a single resolver read path; ZoneFill honesty states +
   Filled-island copper projection at the CLI layer.
5. Library/pool — pool objects are now typed DomainObjects resolved across 8
   subdirs; full journaled library mutation surface; Part/Package enriched to
   ENGINE_SPEC §1.2; content-addressed `pool/models/<role>/<sha256>` storage.
6. MCP — the `datum.*` canonical AI surface (~240 names, 11 classes), the
   ok/schema/context/result envelope, and a substrate-backed native write path
   all landed (prior: 0% exposed).
7. Import — the first substrate-backed converter slice (KiCad footprint
   import) writes native DomainObject shards through commit/journal with
   import-map identity, idempotent re-import, and undo.

### Regressed / Newly-Widened

- CANONICAL_IR §4 (the normative invariant root) still describes a
  dyn-Operation trait the code never had — now FURTHER from reality (enacted
  is a 118-variant enum).
- CLAUDE.md and `docs/contracts/AI_CLI_MCP_TOOL_SURFACE.md` still DENY the
  substrate ("does NOT exist yet", "no Operation/OperationBatch enum, no
  commit()") — more wrong than before because the substrate grew.
  AI_CLI_MCP_TOOL_SURFACE.md is internally contradictory (denies the substrate
  at :45-52, admits partial implementation at :538-542).

### Newly Built

- The entire `datum.*` MCP surface; the Outputs/production GUI lane (~2k LOC);
  the standards CheckProfile lens; the substrate footprint-import converter.

### The Three Prior P0s — Current Status

1. Schematic global-label cross-sheet merge keyed by root COUNT not name —
   STILL OPEN, byte-for-byte unchanged at
   `crates/engine/src/connectivity/mod.rs:283`
   (`let merge_key = format!("global:{}", roots.len());`). The file moved out
   of `schematic/connectivity/` but the bug is verbatim and NOT in `git
   status` (committed at 22aeebe, untouched by Codex's batch). The by-name key
   exists at `mod.rs:161,185` but line 282 iterates `.values()` and discards
   it. The sole test uses one name across sheets and cannot catch a same-count
   collision. This is the single highest-priority correctness fix in the
   codebase and remains a data-integrity blocker feeding ERC/DRC/export.

2. GUI `board_text_mutations.rs` private write path — RESOLVED (in untracked
   worktree). `board_text_mutations.rs` and `board_text_field_values.rs` are
   DELETED (git status `D`); replaced by
   `crates/gui-app/src/board_text_terminal_commands.rs` which only generates
   `datum-eda project edit-board-text` CLI strings (no trailing newline = no
   auto-execute). The CLI target routes through `Operation::SetBoardText` →
   `commit_board_layout_operation` (`command_project_board_layout.rs:268-271,
   :361-384`): `ProjectResolver::resolve` + `commit_journaled` + revision
   guard + `CommitProvenance{Cli}`. A new drift gate
   (`scripts/check_schematic_private_writers.py`) fences the retired helper
   names and asserts the files stay deleted; pattern ratified verbatim in
   PRODUCT_MECHANICS_001. The only surviving raw writers in the GUI crates are
   demo-fixture/GUI-local-state, not board mutation.

3. MCP/daemon mutations bypass commit() — PARTIAL. RESOLVED for the NEW native
   write surface: `datum.pcb.*`/`datum.schematic.*`/`datum.library.*` aliases
   bridge to substrate-backed CLI verbs that route through `ProjectResolver` +
   `commit_journaled` (e.g. `command_project_board_component_mutations.rs:41-59`).
   STILL OPEN for the legacy daemon-socket path:
   `crates/engine-daemon/src/dispatch.rs` imports zero substrate (grep empty)
   and its write arms still call `engine.delete_track/move_component/set_value/
   set_net_class/set_design_rule` (lines 36,57,100,254,270) with no
   journaling. These bypassing legacy tools remain in the MCP catalog
   (`tools_catalog_data.py`) and socket-bound, so AI can still reach a
   non-journaled write path. The AI surface now has BOTH a compliant native
   path AND a coexisting non-compliant legacy path — a concrete
   Private-Mutation-Ban exposure (decision-004:55-59).

## How To Read This

- Each subsystem is classified on-track / partial / divergent, with
  conformance NOW vs PRIOR.
- Per prior finding: resolved / partial / open / regressed / obsolete, with
  current file:line evidence. "Resolved" requires positive evidence the fix is
  present; "still open" requires evidence it persists.
- committed (survives a HEAD checkout) is kept distinct from untracked
  worktree (`??`) progress.
- Per-subsystem findings were adversarially verified; where a finder's own
  evidence was refuted by its verifier, the refuted item was dropped.

---

## Subsystem: Canonical IR & Native Format

Conformance now: PARTIAL (prior: PARTIAL)

Headline: IR primitives correct and unchanged; the pool/library domain is now
engine-typed and write-gated (major convergence), but board native types
remain CLI-owned untyped `serde_json::Value` and CANONICAL_IR §4 is now
further from the enacted 118-variant Operation enum.

Prior-findings resolution:
- CANONICAL_IR §4 dyn-Operation trait + `OpDiff` + `Transaction{timestamp:
  Instant}` — OPEN, WIDENED. `docs/CANONICAL_IR.md:123-176` unchanged (no git
  flag). Enacted Operation is now a 118-variant serde-tagged enum in dedicated
  `crates/engine/src/substrate/operation.rs:7` (verified exactly 118 variants);
  `CommitDiff{created/modified/deleted: Vec<ObjectId>}` at `substrate/mod.rs:
  293-296`. The `Instant` field still contradicts §7 determinism.
- Native-format types CLI-owned — PARTIAL. Board side still CLI-owned + untyped:
  `crates/cli/src/command_project_native_types.rs` `NativeBoardRoot` carries
  `BTreeMap<String, serde_json::Value>` for packages/pads/tracks/vias/zones/
  nets/net_classes (:29,:63-79). The component-decoration layer IS typed
  (Vec<NativeComponentPad>, Vec<ModelRef>, etc.). RESOLVED for the pool domain:
  a write-time schema gate `validate_pool_library_object`
  (`substrate/pool_journal_ops.rs:315-392`) enforces schema_version==1,
  UUID/path/kind parity, and canonical typed deserialization on journaled
  Create/Set ops (two-layer: parity + per-kind shape). The engine now owns and
  gates the on-disk schema for the pool domain, NOT the board domain.
- Forward-annotation + project-create write_canonical_json bypass — PARTIAL.
  Forward-annotation RESOLVED for design-state: routed through
  `command_project_forward_annotation_substrate.rs` (ProjectResolver::resolve +
  Operation + Proposal + apply_accepted_proposal, :19,:38,:55,:123,:128). Only
  a `.datum/forward_annotation_review/review.json` decision-sidecar still uses
  write_canonical_json. Project-create STILL OPEN but minor:
  `command_project_roots.rs:239-242` seeds the four skeleton shards directly
  (genesis bootstrap, classified as an allowed boundary by the drift gate).
- Two parallel design models — OPEN. `api::Design` hand-rolled undo/redo stacks
  (`api/mod.rs:58-82`) vs `substrate::DesignModel` (`mod.rs:352`). Not unified.
- ProjectResolver does not gate unknown future schema_version — OPEN, now an
  explicit conflict: `project_resolver.rs:62-263` reads `schema_version` (:36)
  and propagates it (:263) with no rejection; NATIVE_FORMAT_SPEC §8 (:643) now
  states "an unknown future version is a load error."
- §6.1 created/modified timestamps conflict with §7 — OPEN (spec-stale; manifest
  struct omits them).
- §8 migration functions / historical goldens — OPEN (only v1; acceptably
  spec-ahead).
- §6.10 pool/models content-addressed tree — OPEN; note model attachment is now
  partially exercised through typed `AttachPoolPartModel`/`DetachPoolPartModel`
  ops (`operation.rs:239-251`), but the content-addressed tree itself is
  deferred.
- REFUTED prior: route-proposal apply rewrites board.json off-model — OBSOLETE
  (refutation holds; production apply goes through the substrate).

What's implemented now:
- IR primitives unchanged and correct: deterministic BTreeMap key-sort
  serialization (`ir/serialization.rs:12-35`); integer-nm units (mil_to_nm=
  25_400, inch_to_nm=25_400_000 — exact IPC/imperial defs, externally verified;
  `ir/units.rs:4-26`); i32 tenths angles with rem_euclid; UUIDv4 native /
  UUIDv5 deterministic-import identity (`ir/ids.rs:6-24`).
- NEW engine-owned typed pool-library authoring through commit/journal with a
  write-time schema gate (`substrate/pool_journal_ops.rs`).
- NATIVE_FORMAT_SPEC grew ~+180 lines documenting the typed pool authoring +
  write-gate surface — spec and code co-landed.

Still open / divergent:
- CANONICAL_IR §4 normative doc still describes the nonexistent dyn-Operation
  model; divergence widened.
- Board native types CLI-owned and untyped for copper/connectivity fields;
  engine cannot gate the board schema at load.
- No ProjectResolver schema_version load gate (now §8 conflict).
- Two design models persist.

Course guidance: Re-rate holds at PARTIAL; trajectory positive. (1) Highest
spec value — rewrite CANONICAL_IR §4 to the enacted serde-tagged-enum +
commit/journal model. (2) Extend the engine-owned write-time schema gate from
the pool domain to the BOARD domain. (3) Add the schema_version load gate to
ProjectResolver or downgrade §8. IR primitives need no work.

## Subsystem: Substrate — Resolver, Identity, Commit, Journal

Conformance now: ON-TRACK (prior: ON-TRACK)

Headline: Strongest, most-converged subsystem. `object_revision`
shard-persistence RESOLVED and proposal preview==commit parity landed in
production code; remaining gaps are the unbuilt PG-* proof-gate harness and the
absent `acceptance_path` provenance field — plumbing/spec-wording, not core
defects.

Prior-findings resolution:
- PG-* proof-gate harness wired/machine-enforced — OPEN. `scripts/
  run_migration_proof_gates.sh` absent; `run_drift_gates.sh:4-13` invokes zero
  PG gates; grep PG-IDENTITY/PG-COMMIT/PG-PROPOSAL = empty. Specs still assert
  wiring (000D:863-877, 001:496-497, 000B:474). Claim remains unbacked.
- No proposal preview==commit parity assertion — PARTIAL (machinery LANDED).
  `proposal.rs:373-385` `predict_journaled_transaction_id` stages writes,
  clones the model, runs commit() on the same batch; `:219-237`
  `apply_accepted_proposal` hard-asserts the committed transaction_id equals the
  prediction; `create_draft_proposal_from_batch` previews affected_objects via
  a cloned commit() (`:154-160`). But no PG-PROPOSAL-PARITY gate and no
  standalone parity test (`tests/proposal.rs` exercises it only transitively).
- No acceptance_path provenance field — OPEN. `CommitProvenance` is
  actor/source/reason only (`mod.rs:275-280`); grep `acceptance_path` empty.
  Approximated by `CommitSource`(`mod.rs:283-291`) + `ProposalSource` +
  `Proposal.applied_transaction_id`.
- object_revision journal-replay-derived not shard-persisted — RESOLVED.
  `object_revision_for()` reads it from shard JSON (`mod.rs:586-592`);
  `PersistedComponentInstance` serde field (`component_instance.rs:24`);
  reload-asserted (`tests/component_instance.rs:190,207`).
- GUI board_text bypass — OPEN from the substrate's view, but resolved in the
  GUI subsystem (board_text_mutations.rs deleted; see GUI section). The only
  raw writer remaining in gui-protocol is demo-fixture seeding.

What's implemented now:
- Single commit primitive intact in ratified order: `commit_journaled_with_links`
  (`mod.rs:425-474`): revision guard → inverse_operations_for_batch → stage
  shard bytes → clone+commit() (recompute revision) → append_transaction_journal
  → promote_staged_shard_writes → write_journal_cursor → swap. `commit`
  (`mod.rs:381-423`).
- `model_revision` = sha256 over project_id + non-evidence shard pairs + sorted
  object id/revision/source_shard (`compute_model_revision mod.rs:654-683`);
  evidence shards (ArtifactMetadata/ArtifactRun/OutputJobRun/CheckRun/ZoneFill/
  ImportMap/ProposalMetadata) excluded.
- ComponentInstance now genuinely POPULATED via electrical↔physical join
  (`component_instance.rs:61-160`), persisted shards, dedicated journal ops.
- Substrate roughly doubled (~14k→~20.7k lines) across ~50 modules: variant.rs,
  relationship.rs, zone_fill.rs, check_run/artifact_metadata/artifact_run, the
  per-domain operation_application_* splits.
- ImportMapEntry keyed by import_key, populated read-only at resolve
  (`mod.rs:545-556`).

Still open: PG-* harness; acceptance_path; standalone proposal-parity test/gate.
Convergence-debt (other subsystems' findings): daemon imports zero substrate;
GUI surfaces not all routed onto commit().

Course guidance: Stay the course. (1) Build `run_migration_proof_gates.sh` and
wire it (code now backs PG-IDENTITY/PG-COMMIT/PG-PROPOSAL-PARITY) OR downgrade
the "all gates wired" spec claims. (2) Add an acceptance_path field or ratify
the CommitSource/ProposalSource/applied_transaction_id mechanism as its
definition. (3) Add a standalone preview==commit parity test.

## Subsystem: Schematic Connectivity & ERC

Conformance now: PARTIAL (prior: PARTIAL)

Headline: Essentially flat. The P0 global-label merge keyed by root-count
(`connectivity/mod.rs:283` `global:{roots.len()}`) is byte-for-byte STILL OPEN
and untouched; `passive_only_net` still falsely `[x]`; narrow forward progress
only on shared checking types.

Prior-findings resolution:
- (P0) Global-label merge keyed by root COUNT not name — OPEN, verbatim.
  `connectivity/mod.rs:283` reads `let merge_key = format!("global:{}",
  roots.len());`. Groups ARE keyed by name in `global_label_groups_by_name`
  (`:161,:185`) but line 282 iterates `.values()` and discards the name. File
  NOT in git status (committed at 22aeebe). The sole test uses one name 'VCC'
  across sheets and cannot catch a same-count collision.
- passive_only_net `[x]` but absent — OPEN. `grep passive_only` over
  `crates/engine/src` returns only spec/PROGRESS hits (PROGRESS.md:467, :1525
  both `[x]`); no such rule in `erc/mod.rs`; absent from the required-codes
  corpus (`api/tests/goldens.rs:84-90`).
- NetSemanticClass enum — OPEN. `infer_semantic_class` returns
  `Option<String>` with only "ground"/"power"/None (`connectivity/mod.rs:
  410-426`); no enum.
- ErcReport/ErcViolation/ErcRuleKind/SchematicLocation — PARTIAL. Shipped shape
  is `ErcFinding` (`erc/mod.rs:22-37`); object-anchoring exists
  (`objects: Vec<ErcObjectRef>`, `object_uuids: Vec<Uuid>`), but no
  sheet/coordinate SchematicLocation; `code` is `&'static str`, no enum.
- Conflicting-labels-on-same-segment not detected — OPEN. `preferred_name`
  (`:716-732`) silently selects one; no conflict check.
- Bus container/bus_entry data model ignored — OPEN.
- ERC rule-code names diverge from ERC_SPEC §4; waiver keys on code string —
  OPEN.
- PinElectricalType narrows TriState/OC/OE away — OPEN
  (`schematic/mod.rs:105-112`); §5 compatibility matrix unrealizable.
- Native net/anon identity via import_uuid(namespace_kicad()) — OPEN
  (`connectivity/mod.rs:327-346`); NetPinRef join on `component:String`.

New since last audit:
- `WaiverTarget::Fingerprint(String)` arm now EXISTS in the shared enum
  (`schematic/mod.rs:393`), but is a no-op stub in ERC (`waiver_matches` returns
  false for Fingerprint at `erc/mod.rs:491`) — scaffolding, not behavior.
- New untracked `CheckDeviation` + `DeviationApprovalStatus::Accepted`
  (`schematic/check_disposition.rs:6-19`), unconsumed by the ERC apply path.
- `erc/mod.rs`/`schematic/mod.rs` are git-modified; `connectivity/mod.rs` is
  NOT — Codex touched the shared types but left the P0 untouched.

Course guidance: PARTIAL holds; delta flat. (1) P0 — fix the merge key at
`connectivity/mod.rs:282-283` to key on net NAME (iterate the by-name map's
entries, use `format!("global:{name}")`); add a two-distinct-equal-count
regression (VCC on 3 sheets + GND on 3 sheets must stay two nets). (2) Fix the
false `passive_only_net` `[x]` or implement it. (3) Surface the ERC
location-anchoring decision (SchematicLocation vs object-only) to the owner as
a product question. Do NOT widen KiCad bus/hierarchy import fidelity (frozen).

## Subsystem: Board, DRC & Routing Kernel

Conformance now: PARTIAL (prior: PARTIAL)

Headline: Both prior-missing pad aperture codes and the `WaiverTarget` 4-arm
enum RESOLVED; but the committed engine DRC still counts authored zones as
routed copper without consulting `ZoneFillState` (fabrication-trust false-pass)
while the honest ZoneFill-aware typed check lives only in the untracked CLI
layer — two check paths that disagree on the same board.

Prior-findings resolution:
- Unified typed CheckRun/CheckFinding model absent — PARTIAL. Committed engine
  STILL legacy: `drc/mod.rs:26-51` (DrcReport/DrcViolation),
  `api/query_surface.rs:235-247` run_drc → DrcReport, `dispatch.rs:465-476`
  returns the legacy report. Substrate `CheckFinding/CheckRun` grew typed-ish
  fields (fingerprint/domain/rule_id/status/primary_target/related_targets/
  evidence) at `substrate/check_run.rs` but severity/status/fingerprint are
  String and targets/evidence are `serde_json::Value`. The lossless
  legacy→CheckRun bridge exists only in the untracked CLI layer.
- CheckFingerprint type — PARTIAL. No `CheckFingerprint` type exists (grep=0).
  Mechanism landed as a String contract `datum-eda:check-finding-fingerprint:v1`
  (`command_project_check_run_view.rs:630-646`, sha256 over
  domain+rule_id+primary_target+evidence) — omits rule revision and import_key
  that §6 requires.
- Two of six pad process-aperture codes absent — RESOLVED. Both emitted by the
  committed engine: `drc/checks/mod.rs:380` (inherited_from_copper) and `:488`
  (inconsistent_with_peer_footprint). All six §11 codes present.
- Committed DRC not resolver-pinned — OPEN. run_drc operates on
  `require_board()` in-memory Board; no model_revision pinning.
- ZoneFill honesty hole — OPEN (the sharp subsystem P0). `drc/checks/mod.rs:19,
  69` count `net.zones==0` as the copper test without consulting fill state;
  `net_graph.rs:101-194` derives zone anchors from authored polygons with zero
  `ZoneFillState` reference. The honest path exists only in the untracked CLI
  layer (`artifact_checks.rs:79-101` emits zone_fill_unfilled/stale/unsupported;
  `zone_fill_projection.rs:18` blocks non-Filled). The two paths disagree;
  daemon/MCP receive the dishonest shape.
- WaiverTarget::Fingerprint arm missing — RESOLVED at the type level
  (`schematic/mod.rs:389-393`, 4 arms), live in the CLI bridge, but DEAD in the
  engine DRC (`drc/mod.rs:147` returns false; DrcViolation has no fingerprint).
- DRC rule set hard-coded — OPEN (`dispatch.rs:465-473`; no CheckProfile param).

What's implemented now: all 7 DRC rules with deterministic content-derived IDs;
all 6 §11 pad aperture codes; deterministic fingerprint contract; honest
ZoneFill reporting at the CLI layer; substrate CheckRun/CheckFinding persisted
shards; routing kernel: 50 deterministic `route_path_candidate*` strategies,
read-only/native-first; route-proposal APPLY through `substrate/proposal.rs`
(`apply_accepted_proposal :196`, ProposalSource/CommitSource provenance,
model_revision pin + stale-revision guard).

Course guidance: Net delta positive (2 resolved, 2 partial, 3 open). (1) P0 —
push CheckRun assembly + ZoneFillState consultation DOWN into the engine: fix
`drc/checks/mod.rs:19,69` to consult fill state, so run_drc (and daemon/MCP
consumers) return the honest typed model. (2) Give engine DrcViolation a
fingerprint so `drc/mod.rs:147` stops being dead. (3) Run DRC through
ProjectResolver and thread profile_id. Keep the routing kernel on course; add
the still-missing determinism contract section.

## Subsystem: Native Authoring & Operation Model

Conformance now: ON-TRACK (prior: PARTIAL)

Headline: Moved up. The GUI board-text private writer is retired and
forward-annotation apply routed through the substrate proposal gateway;
pool-library and schematic-symbol op families landed with apply/inverse;
remaining off-model paths (Engine::save s-expr text-patch, daemon dispatch,
project-create bootstrap) are correctly deferred.

Prior-findings resolution:
- (P0) GUI board-text off-model private writer — RESOLVED.
  `board_text_mutations.rs` + `board_text_field_values.rs` DELETED (git `D`).
  `crates/gui-app/src/board_text_terminal_commands.rs:114-118` generates
  `datum-eda project edit-board-text ...` strings only; main.rs:2652-2710 wire
  every canvas action through the conduit. CLI commits via
  `Operation::SetBoardText` → `commit_board_layout_operation`
  (`command_project_board_layout.rs:268-271, :361-384`).
- Drift gate fences gui-protocol private writes — RESOLVED. `scripts/
  check_schematic_private_writers.py` forbids 15 retired helper names in
  gui-app/main.rs (FORBIDDEN_GUI_BOARD_TEXT_PATTERNS :208) and asserts the
  retired files stay deleted (:153-156, :444-457). NOTE: the gate's
  board-text/retired-file guards are present and pass; the overall gate has at
  times tripped on an unrelated generated-export count in
  `command_project_inventory.rs`. Treat the gate's GUI-specific guards as
  green; do not assert the whole gate is unconditionally green.
- Imported-KiCad parallel non-journaled write model — OPEN (correctly deferred).
  `Engine::save` text-patches s-expressions (`api/save_kicad.rs:39` std::fs::
  write); callers `command_modify` and `engine-daemon/dispatch.rs:23`.
- Project-create write_canonical_json — OPEN, minor (classified bootstrap
  boundary; `command_project_roots.rs:239-242`).
- Forward-annotation apply bypass — RESOLVED (see Canonical IR section;
  `command_project_forward_annotation_substrate.rs`).
- No MoveSymbol/RotateSymbol/MirrorSymbol; no library/symbol op families —
  PARTIAL. Pool-library (Create/Set/Delete/Attach/Detach) and schematic-symbol
  (Create/Set/Delete) op families landed with apply+inverse, CLI-wired through
  commit_journaled. STILL MISSING dedicated Move/Rotate/Mirror ops (grep=0);
  symbol transforms ride on whole-symbol `SetSchematicSymbol` replacement.
- REFUTED prior CANONICAL_IR §4 scope — carried: the §4 staleness originates in
  the normative doc; ENGINE_DESIGN/AUTHORING_TOOLS inherit.

What's implemented now: 118-variant serde-tagged Operation enum; single
substrate commit gateway for native board mutations; GUI board-text via
journaled CLI conduit; forward-annotation on the proposal gateway; schematic-
symbol + pool-library op families; drift gate enforcing the convergence.

Course guidance: Stay the course (PARTIAL→ON-TRACK). Remaining off-model paths
are bounded and correctly NOT advanced; the daemon mutation dispatch is the
next convergence target (owned by the MCP/daemon subsystem). Decide whether to
add dedicated Move/Rotate/Mirror symbol ops vs keep Set-replacement. Refresh
PROGRESS rows 50-51 (library/symbol op families landed) and CANONICAL_IR §4.

## Subsystem: Manufacturing / CAM Export

Conformance now: ON-TRACK (prior: PARTIAL)

Headline: Largest single-domain leap. ComponentInstance-keyed BOM/PnP,
`--include` fan-out, `SetOutputJob` with undo/redo, and ArtifactMetadata for
bom/pnp/drill all RESOLVED; Gerber RS-274X externally verified correct. Open:
real zone-fill solver, drill fab-completeness, hide redundant public export
verbs.

Prior-findings resolution:
- BOM/PnP keyed by reference:String — RESOLVED. `NativeBomRow::identity_key()`
  returns `component_instance_uuid.to_string()` with package_uuid fallback
  (`command_project_inventory.rs:38-43`); identical PnP impl (`:106-110`);
  compare diffs via expected_by_identity/actual_by_identity. `reference` is now
  a drift field, not the join key.
- Single export verb with --include fan-out unrealized — RESOLVED.
  `parse_manufacturing_include()` (`command_project_manufacturing_scope.rs:
  302-327`) parses gerber-set/manufacturing-set/bom/pnp/drill/all;
  `scope.include: Vec<ArtifactKind>`; export gates each artifact on projection
  membership.
- No metadata persisted for bom/pnp/drill; split read path — RESOLVED.
  `manufacturing_set_artifact_metadata` (`scope.rs:198-234`) aggregates per-file
  sha256 for bom/pnp/drill_csv/excellon/gerber into one ArtifactMetadata via a
  single `ProjectResolver::resolve()` read path.
- No --include/--job/--variant fan-out on export — RESOLVED
  (`cli_args_manufacturing.rs:171-179`; variant/prefix/include inherited from
  the stored OutputJob).
- REFUTED prior `Operation::UpdateOutputJob` — RESOLVED. The update primitive
  landed as `Operation::SetOutputJob` (`operation.rs:307-318`); applied at
  `operation_application.rs:602`; inverse undo/redo at `production_journal_ops.rs:
  109-338`; CLI update-output-job routes through it.
- No real zone-fill solver — PARTIAL. `ZoneFillState` now has
  Filled/Unfilled/Stale/Unsupported; a fill-zones check persists Filled for
  safe-simple zones; copper projection renders only Filled islands. But
  `derive_zone_fills` default is Unfilled (`zone_fill.rs:96`); no clearance/
  antipad/thermal/obstacle/multi-island solver. Imported real pours still
  project empty copper unless a check upgrades them.
- No plated/non-plated NC separation, no G85; legacy drill.csv — OPEN.
  `excellon.rs:6-46` single combined file; both drill.csv and drill.drl emitted
  (`command_project_manufacturing.rs:332-337`).
- Excellon METRIC,TZ with explicit-decimal coords, no FMAT,2 — OPEN
  (functionally tolerated; externally confirmed formally imprecise).
- Per-format/per-layer export+validate+compare subcommands remain public — OPEN
  (11+ export-family variants in `cli_args_project_commands.rs:155-226`).
- (cross-domain) MCP/daemon manufacturing mutations bypass commit() — RESOLVED
  for manufacturing: OutputJob mutation is on-substrate; MCP exposes
  proposal-based output-job edits; export itself is a non-commit projection;
  daemon has no manufacturing entries.

What's implemented now: RS-274X Gerber (%FSLAX46Y46*% / %MOMM*% / %LPD*% with
raw-nm coords = 1 nm LSB, externally re-derived correct; C/R apertures, G36/G37
regions, D03 flashes); Excellon .drl; full OutputJob create/set/delete with
inverse; ComponentInstance-keyed BOM/PnP; unified --include fan-out;
ArtifactMetadata aggregation; variant fitted-state filtering; ZoneFill honesty
substrate; MCP proposal-based output-job authoring. Worktree builds clean.

Still open: real zone-fill solver; drill fab-completeness (plated/non-plated,
G85, drop legacy drill.csv, fix Excellon TZ/FMAT,2); hide per-format public
verbs.

Course guidance: PARTIAL→ON-TRACK. Priority: (1) real zone-fill solver — the
one functional gap letting the projection oracle pass on incomplete copper;
(2) drill fab-completeness; (3) hide per-format/per-layer public export verbs
behind --include (cosmetic). Retire the prior P1 manufacturing-identity row.

## Subsystem: Library & Pool System

Conformance now: PARTIAL (prior: DIVERGENT)

Headline: Two prior BLOCKING divergences RESOLVED (library objects now typed
DomainObjects through commit/journal); but decision-008 native types
(Footprint/PinPadMap/LibraryBinding/StandardsBasis) exist only as JSON subdirs
not Rust types, the IPC footprint system is absent, Padstack is still
single-aperture, and ~1300 lines of new code extended the M0 Horizon model
while decision-008 remains unratified draft.

Prior-findings resolution:
- Library objects NOT DomainObjects; pool a side HashMap — RESOLVED.
  `read_pool_ref_shards` (`project_resolver.rs:292-396`) walks 8 subdirs
  (units/symbols/entities/parts/packages/footprints/padstacks/pin_pad_maps),
  types each shard domain="pool", stamps `object.kind=subdir`.
- No proposal/journal path for library mutation — RESOLVED. Typed Create/Set/
  Delete/Attach/Detach ops (`operation.rs:226-256`) with inverses
  (`pool_journal_ops.rs:211-262`); CLI commits via OperationBatch +
  commit_journaled (`command_project_library.rs:1678-1681`).
- decision-008 native primitives absent — PARTIAL. PinPadMap/Footprint exist as
  resolver subdir kinds + CLI mutation kinds (JSON-shaped only); NO Rust types
  (grep struct Footprint/PinPadMap/LibraryBinding/StandardsBasis/ApprovalState/
  ProvenanceSet = 0). standards_basis/approval_state/object-revision-on-library
  absent.
- ENGINE_SPEC §1.2/§1.1a missing Part/Package/Model fields — RESOLVED.
  `pool/mod.rs` grew 194→589 lines: Part carries manufacturer_jep106 (:143),
  packaging_options, behavioural_models, thermal, supply_chain_offers; Package
  body_height_nm; ModelAttachment/ModelRole/ThermalSpec/SupplyOffer defined.
- Entire IPC footprint system — OPEN (grep=0). Only the pre-existing board-level
  ProcessAperture DRC rule exists.
- Padstack single-aperture + scalar drill — OPEN (`pool/mod.rs:87-102`).
- pool/models content-addressed storage, FTS, unit-aware search — PARTIAL.
  Content-addressed storage LANDED (`command_project_library.rs:1003-1046`,
  `pool/models/<role>/<sha256>`, deterministic v5 UUID, gc + validate). FTS and
  unit-aware parametric search still absent; PoolIndex still parts/part_tags/
  part_parametric only; search_parametric exact-string equality.
- Two incompatible library models; ratify before more code — OPEN. decision-008
  still "draft for owner review" with 6 open Owner Questions; no supersede
  banner in POOL/LIBRARY_ARCHITECTURE; code extended the M0 Horizon model.
- No LibraryBinding / ComponentInstance join — PARTIAL. ComponentInstance joins
  schematic component → part → board package (`component_instance.rs:55-128`,
  keyed on (reference, part)); but not the decision-008 LibraryBinding/
  ProvenanceSet object.

What's implemented now: typed pool DomainObjects across 8 subdirs; full
journaled library mutation surface; CLI library authoring/edit suite through
commit_journaled; Part/Package enriched to §1.2; behavioral-model attachment
with content-addressed storage; KiCad footprint importer keyed through the
import map; ComponentInstance join; MCP/daemon library surface.

Still open: decision-008 native Rust types; IPC footprint system; per-layer
Padstack; standards_basis/approval_state; FTS/unit-aware search; the
unreconciled Horizon-vs-008 contradiction.

Course guidance: DIVERGENT→PARTIAL. (1) P1 — Owner ratifies/reconciles
decision-008 vs the M0 Horizon model BEFORE further library type work (the
contradiction is now MORE load-bearing given ~1300 lines of new investment).
(2) Once ratified, promote footprints/pin_pad_maps to typed DomainObjects and
introduce LibraryBinding/StandardsBasis/approval_state. (3) Decide IPC scope or
banner spec-ahead. (4) Add a pool/library parity inventory.

## Subsystem: MCP / CLI / Daemon Surface

Conformance now: ON-TRACK (prior: PARTIAL)

Headline: The ratified `datum.*` AI surface (~240 names + ok/schema/context/
result envelope + substrate-backed native write path) landed end-to-end (prior:
0% exposed); the remaining defect is convergence-debt — bypassing legacy daemon
mutation tools left in the catalog alongside the compliant aliases, plus an
import-centric daemon that imports zero substrate.

Prior-findings resolution:
- 7-class datum.* surface 0% exposed; envelope empty — RESOLVED. ~240 distinct
  `datum.<class>.<verb>` names across 11 classes (context/query/check/proposal/
  journal/artifact/component_instance/pcb/schematic/library/artifact_metadata_
  list). Envelope built: `stdio_tool_host.py` `_datum_target_envelope`
  {ok,schema:{name,version},context,result} (:88-101); `_datum_error_envelope`
  ok:false (:104-108). `datum.context.get` end-to-end through substrate
  (`server_runtime.py:281` → CLI `context get` → `command_context.rs:8` using
  ProjectResolver).
- MCP mutations bypass commit(); daemon imports zero substrate — PARTIAL.
  RESOLVED for the native surface (datum.pcb/schematic/library aliases bridge to
  substrate-backed CLI verbs through ProjectResolver + commit_journaled,
  `command_project_board_component_mutations.rs:41-59`). STILL OPEN for the
  legacy daemon-socket path: `dispatch.rs` write arms call legacy `engine.*`
  mutators (36,57,100,254,270) with zero substrate import; those tools remain
  catalogued (`tools_catalog_data.py:42,60,87,301,339`) and socket-bound.
- open_project = engine.import() keeps daemon import-centric — PARTIAL. Daemon
  still import-centric (`main.rs:240-241`); no context/proposal/commit arm. But
  native context.get is exposed via the CLI bridge.
- Naming eda→datum-eda — RESOLVED (`server_runtime.py:32,40`, DATUM_* primary,
  EDA_* fallback).
- explain_violation by (domain,index) not fingerprint — OPEN (`dispatch.rs:
  477-479`).
- Parity counts 87/188/53 — OBSOLETE. Gate GREEN at grown counts: SPEC_PARITY.md
  mcp_runtime_methods=182, cli_project_commands=270, daemon_dispatch_methods=54.
- AI_CLI_MCP_TOOL_SURFACE.md "115 sites / no substrate" — REGRESSED. Header
  (:45-52) still denies the substrate (no Operation/OperationBatch, no commit()
  journal, no DatumToolSession, no model_revision/ComponentInstance/import_key,
  115 sites) — all now false and backing the live datum.* surface; the same
  doc's body (:538-542) admits partial implementation (internal contradiction).

New since last audit: the entire datum.* surface (240 names); the
ok/schema/context/result + ok:false envelope; the substrate-backed native AI
write path; MCP_API_SPEC.md rewritten with envelope + class table. New
divergence: dual write surface — bypassing legacy daemon tools were NOT retired
when the compliant aliases were added.

Course guidance: PARTIAL→ON-TRACK. (1) P1 — retire (or formally defer-with-gate)
the bypassing legacy daemon mutation tools from the MCP catalog so AI cannot
reach a non-journaled write path; add a gate ensuring no NEW bypassing daemon
tool is catalogued. (2) Point the daemon's own dispatch at commit_journaled or
document it as a read/legacy island. (3) Fix the stale AI_CLI_MCP_TOOL_SURFACE.md
header and reconcile the MCP_API_SPEC class table with the per-domain mutation
groups. Do NOT add more flat legacy daemon tools.

## Subsystem: GUI Substrate

Conformance now: ON-TRACK (prior: DIVERGENT)

Headline: Flipped from worst to among the cleanest. The prior BLOCKING private
board-text write path is fully retired (files deleted, journaled CLI conduit,
green GUI-specific drift guard, ratified canon); residual items are a spec-stale
M7_FRONTEND_SPEC read-only mandate and a minor `text_uuid` vs
`source_object_uuid` naming divergence.

Prior-findings resolution:
- (P0/BLOCKING) GUI write-capable via board_text_mutations.rs — RESOLVED.
  Both `board_text_mutations.rs` and `board_text_field_values.rs` deleted (git
  `D`). All canvas board-text actions (main.rs:2752-2820) and the terminal lane
  funnel into `begin_selected_board_text_command_edit` (main.rs:2731-2750) which
  records a journaled handoff then prefills the PTY (no auto-execute newline).
  CLI edit-board-text commits via Operation::SetBoardText.
- The named offender retired and ratified — RESOLVED. PRODUCT_MECHANICS_001
  "Retired bypasses stay retired" documents the exact module by name.
- Drift gate fences gui-protocol — RESOLVED (GUI-specific guards present and
  passing; see Native Authoring note on the unrelated generated-export count).
- Self-contradicting read-only posture — RESOLVED (no lane mutates the board
  document directly; all mutation lands via commit() in the CLI).
- gui-protocol/gui-render depend on eda-engine for read; gui-app does not — OPEN
  (now architecturally intentional: gui-app reaches the substrate only by
  emitting CLI command strings, per PRODUCT_MECHANICS_005).

What's implemented now: BoardReviewSceneV1 typed serde scene contract with the
§2.5 identity triple across primitives; visual regression harness (bless +
exact/tolerance diff, 4 checked-in goldens); NEW Outputs/production lane (~2k
LOC) using the same terminal-command-handoff conduit with zero private writes;
append-only terminal hand-off audit-event stream. All three GUI crates compile
clean.

Still open / divergent:
- BoardTextPrimitive uses `text_uuid` (`gui-protocol/lib.rs:109`) instead of the
  `source_object_uuid` every sibling primitive uses (§2.5).
- M7_FRONTEND_SPEC §1.7/§1.8 still list apply/commit from canvas/terminal as
  Not-supported; shipping behavior (sanctioned by PRODUCT_MECHANICS_001/005)
  exceeds this — the two spec layers disagree.
- Surviving demo/state raw writers (gui-protocol/lib.rs demo fixtures,
  main.rs:3056 .datum/assistant.json, terminal_session.rs:577-591) are
  GUI-local-state, not board mutation, but are unfenced — a latent re-entry
  point.

Course guidance: Stay the course (DIVERGENT→ON-TRACK). (1) Amend M7_FRONTEND_SPEC
§1.7/§1.8 to sanction the terminal-prefill journaled-CLI edit conduit and the
Outputs lane, cross-referencing PRODUCT_MECHANICS_001/005. (2) Rename
BoardTextPrimitive.text_uuid to source_object_uuid or document the exception.
(3) Optional: fence the surviving demo/state writers behind a fixture-only
allowlist. Keep renderer/contract/harness as-is.

## Subsystem: Standards & Compliance

Conformance now: ON-TRACK (prior: ON-TRACK)

Headline: Improved without regressing. CheckFinding gained domain/rule_id/
fingerprint, a standards CheckProfile lens, and track/via repair families; but
StandardsBasis keying/provenance is still entirely absent (grep=0), the
standards profile is rule-code-keyed not basis-keyed, and import_key is missing
from CheckFinding fingerprints.

Prior-findings resolution:
- Standards check rule-keyed not StandardsBasis-keyed — OPEN. StandardsBasis
  grep=0 across the tree; the `standards` CheckProfile keys off rule codes
  (`command_project_check_run_view.rs:99,105`); no declared/inferred/imported/
  unknown basis_kind per decision-010.
- CheckFinding lacks domain/import_key — PARTIAL. `domain` now exists
  (`check_run.rs:23`, with rule_id/status/primary_target/explanation/fingerprint)
  populated via `finding_domain`. BUT standards findings carry domain="drc"
  (not "Standards" per §5.2); and `import_key` is STILL ABSENT from CheckFinding
  and from the fingerprint material (omits rule-revision and import_key).
- ENGINE_SPEC §1.1a-1.3 batch-1 types lack stub banners — OPEN (ENGINE_SPEC was
  edited but §1.1a/§1.2/§1.3 still carry no inline banner; honesty lives only in
  PROGRESS).
- Process-aperture DRC + proposal-first repair + clean claim discipline —
  RESOLVED and EXPANDED. Repair now covers pads + tracks + vias (SetBoardPad/
  SetBoardTrack/SetBoardVia) via `create_draft_proposal_from_batch`
  (`command_project_standards_repairs.rs:72-301`), never mutating directly.
  certified/fabrication-approved/regulatory grep=0 in non-test code.
- Batch-1 schema types (StandardsRegistry/etc.) — OPEN (grep=0; correctly
  Planned, not pulled forward).
- SPEC_PARITY row for the shipped standards slice — OPEN (grep=0).

New since last audit: CheckFinding domain/rule_id/fingerprint; a `standards`
CheckProfile lens; standards repair expanded to track-width and via hole/annular;
canonical `datum-eda check repair-standards` / `datum.check.repair_standards`
taxonomy with fingerprint-scoped waive/accept-deviation committed through
OperationBatch+journal.

Course guidance: Stay the course (ON-TRACK, improved). (1) Introduce
StandardsBasis as the keying mechanism per decision-010:181-215. (2) Finish §5.2
— route standards-owned findings to domain="Standards" and fold import_key into
CheckFinding + its fingerprint. (3) Add inline ENGINE_SPEC batch-1 stub banners.
(4) Add the SPEC_PARITY row. Do NOT pull forward deferred batch-1 schema/MCP
stubs.

## Subsystem: Import / Interop (Frozen Lens)

Conformance now: PARTIAL (prior: PARTIAL)

Headline: Trajectory positive. The headline prior finding (import bypasses the
substrate entirely) is PARTIALLY RESOLVED — KiCad footprint import is now a
journaled, undoable, identity-stable native-shard converter; boards/schematics
still persist via legacy Engine::save text-patch (correctly deferred); the
.ids.json sidecar path is now dead AND superseded by the substrate ImportMap.

Prior-findings resolution:
- Import bypasses the substrate entirely — PARTIAL. RESOLVED for footprints
  (NEW, untracked): `import/kicad/footprint.rs:8` uses substrate
  ImportKey/ImportMapEntry/allocate_import_identity (:36 allocates stable
  identity); CLI `command_project_imports.rs:37-122` resolves via ProjectResolver,
  builds native DomainObjects (CreatePoolPackage/CreatePoolPadstack/
  CreateImportMapShard/AddProjectPoolRef), commits through commit_journaled with
  CommitProvenance, then re-resolves to verify. Tests assert journal records,
  identity reuse on re-import, and undo emptying the import_map. STILL OPEN for
  boards/schematics: Engine::save text-patches s-expressions
  (`api/save_kicad.rs:13-40`); import_board_document/import_schematic_document
  produce legacy Board/Schematic with no substrate.
- .ids.json identity sidecar path dead — OPEN. IdSidecar/merge/restore
  (`ids_sidecar.rs:20-129`) referenced only in itself + tests; production uses
  only the hashing helper. Now ALSO superseded by the substrate ImportMap.
- decision-011 Core Primitives absent — PARTIAL. ImportProvenance/ImportSession/
  LossinessRecord/MigrationProposal/ImportAuditFinding/InteropArtifact grep=0.
  EXCEPTION: RelationshipKind + ReverseEngineered/BoardOnly have begun landing
  (relationship.rs/mod.rs/journal ops), though NOT yet stamped by the importer.
- ImportMapEntry lacks status field — OPEN (`mod.rs:260-266`: import_key/
  object_id/source_shard_id/source_hash; no Active|MissingInSource|Replaced|
  Split|Merged). import_key (not source_hash) IS the durable reuse key — the
  core decision-011 requirement is satisfied.
- Eagle design import unimplemented — OPEN (correct per frozen posture; only
  Eagle .lbr pool synthesis exists).

New since last audit: substrate-backed KiCad footprint import — the first import
path writing native DomainObject shards through commit()/journal with import-map
identity, durability, idempotent re-import, and undo. The prior "import/ has
zero substrate references" assertion is now FALSE.

Course guidance: PARTIAL, trajectory better. (1) Treat the footprint
substrate-import path as the proof template; do NOT add KiCad/Eagle breadth.
(2) The boards/schematics text-patch path remains the explicitly-deferred
transitional defect — re-route to the same `_with_import_map` + commit_journaled
pattern only when the native-authoring frontier permits. (3) Retire the dead
.ids.json IdSidecar path (now doubly obsolete). (4) Record the landed slice in
PROGRESS so a fresh agent does not re-derive the prior "zero substrate" claim.

## Subsystem: Doc/Code Parity (Meta)

Conformance now: PARTIAL (prior: PARTIAL)

Headline: Gate machinery is healthy and maintained in lockstep with ~2x
inventory growth, but every prose parity finding is still open and several are
now MORE wrong: CLAUDE.md and AI_CLI_MCP_TOOL_SURFACE.md still deny a substrate
that ships; passive_only_net still falsely `[x]`; the green parity gate depends
entirely on the uncommitted SPEC_PARITY.md refresh.

Prior-findings resolution:
- Parity gate green; counts 87/188/53 — PARTIAL. Gate exits 0 in the worktree
  (7 inventories). Live counts grew: mcp=182, cli=270, daemon=54, engine_api=65.
  CRITICAL: green status depends ENTIRELY on the uncommitted SPEC_PARITY.md
  refresh — restoring HEAD:specs/SPEC_PARITY.md (75/182/53/64) and re-running
  fails all four of those rows against live code (verified by restore-and-rerun).
- check_progress_coverage / check_schematic_private_writers live and wired —
  RESOLVED. Both wired into run_drift_gates.sh (worktree-modified). Note: BOTH
  the private-writers gate script and its invocation are worktree-only; HEAD's
  run_drift_gates.sh does NOT reference the private-writers gate, so there is NO
  committed-invokes-missing-file gap (a prior-finding draft claim was refuted).
- PROGRESS substrate-readiness table strongest artifact — RESOLVED. Expanded to
  ~15 honest `[~]` rows with file:line-grade evidence (PROGRESS.md:43-59). The
  one doc artifact NOT understating progress.
- PROGRESS headline re-pointed at native-first — RESOLVED.
- CLAUDE.md denies ProjectResolver/commit/journal/ComponentInstance — OPEN, MORE
  wrong. CLAUDE.md:50 ("does NOT exist yet"), :83-84 ("No ProjectResolver,
  single commit()/journal, ComponentInstance, model_revision, or Import Map
  exist yet"), and a fourth site :254-257 ("## Not Yet Implemented") all deny a
  substrate present at project_resolver.rs:62, mod.rs:381/425/268/161,
  operation.rs:7.
- CLAUDE.md:80 ~115 private-write sites — OPEN. Actual write_canonical_json call
  sites = 14 (~8x overstated). Same 115 figure persists in
  AI_CLI_MCP_TOOL_SURFACE.md:52.
- PROGRESS prose MCP=81/CLI=182 vs gated — PARTIAL. The specific count phrases no
  longer match in MCP/CLI context; daemon prose is now consistent (PROGRESS.md:
  1610 "54 methods" == gated 54).
- Delta report dated 2026-05-25 stale 75/182 — OPEN. DOC_CODE_PARITY_DELTA_
  REPORT.md is CLEAN/unmodified at HEAD (a prior "git-modified" claim was
  refuted), but still dated 2026-05-25 with stale 75/182 counts.
- AI_CLI_MCP_TOOL_SURFACE.md "no substrate" partially stale — OPEN, REGRESSED.
  Header (:45-46,:52) still asserts no Operation/OperationBatch enum, no
  commit(), 115 sites — all false; the same doc admits partial implementation at
  :538-542 (internally contradictory).
- ComponentInstance struct location correction — RESOLVED (now at
  substrate/mod.rs:161; component_instance.rs holds the join/persistence helper).
- (cross-ref) passive_only_net `[x]` but absent — OPEN. Still `[x]` at
  PROGRESS.md:467,1525; 0 grep hits in erc/. The meta gate checks row coverage,
  not truth of `[x]` marks.

REFUTED prior-draft claims (dropped from the narrative): the "private-writers
gate is RED" regression (the gate exits 0; command_project_inventory.rs has 2
generated writes == expected, the "found 3" was transient during the audit
window); the "committed-invokes-missing-gate" meta-gap (HEAD's run_drift_gates.sh
does not invoke the gate at all).

Course guidance: PARTIAL holds. Gate machinery healthy; every prose finding
still open. Priority for read-only doc fixes: (a) correct CLAUDE.md
substrate-denial (incl. the :254-257 site) and ~115→~14 — highest orientation
value; (b) fix passive_only_net `[x]` (cheap, unguarded correctness); (c)
re-banner the delta report and de-contradict the AI_CLI_MCP contract. NOTE for
whoever commits: committing code WITHOUT the SPEC_PARITY.md refresh would land a
red parity gate.

---

## Cross-Cutting Findings

1. UNCOMMITTED WORKTREE is the dominant meta-fact. HEAD is still `22aeebe`; the
   entire `substrate/` module (49 files), all `datum.*` MCP code, the footprint
   converter, and most new checking/library/manufacturing capability are
   untracked `??` or modified `M`. A fresh checkout would see almost none of it.
   All file:line evidence describes worktree state.

2. DOCS DENY THE SUBSTRATE THAT SHIPS — the inverse of import-drift. CLAUDE.md
   (:50,:80,:83-84,:254-257) and AI_CLI_MCP_TOOL_SURFACE.md (:45-52) still
   assert ProjectResolver/commit()/journal/OperationBatch/ComponentInstance/
   model_revision do NOT exist. They demonstrably do, and grew. This is the top
   orientation hazard: a fresh agent could re-implement landed native-first work.

3. TWO PARALLEL MODELS / TWO WRITE SURFACES / TWO CHECK PATHS recur:
   api::Design hand-rolled undo vs substrate::DesignModel; compliant datum.*
   aliases vs bypassing legacy daemon tools; honest ZoneFill-aware CLI check vs
   dishonest committed-engine DRC; typed pool native types vs untyped board
   native types. Convergence is real but UNEVEN across domains.

4. CANONICAL_IR §4 (the normative invariant root) still describes a
   dyn-Operation trait + OpDiff with old-value payloads + Transaction{timestamp:
   Instant} that contradicts the determinism invariant and never matched code.
   Enacted reality (118-variant serde-tagged enum + commit/journal) has diverged
   further. ENGINE_DESIGN/AUTHORING_TOOLS inherit from this stale root.

5. FALSE/STALE STATUS MARKS persist and are unguarded: passive_only_net is `[x]`
   at PROGRESS.md:467/1525 but absent from code (0 grep hits); the parity gate
   checks row coverage, not truth of `[x]` marks.

6. FINGERPRINT mechanism landed as a String contract
   (`datum-eda:check-finding-fingerprint:v1`) but NO `CheckFingerprint` type
   exists, and it omits rule-revision and Import Map import_key that
   CHECKING_ARCHITECTURE_SPEC / STANDARDS_COMPLIANCE_SPEC §5.2 require.

7. SPEC-AHEAD-OF-CODE remains acceptable and correctly NOT pulled forward:
   decision-008 library types, IPC footprint system, decision-011 import
   primitives, Standards Audit Batch 1 schema — all spec-only by ratified
   posture; the risk is they read as shipped (no inline stub banners).

8. Import is correctly and uniformly frozen — no drift toward import-as-authority
   anywhere. The footprint substrate-import slice is a one-time converter into
   native journaled shards with import_key as read-only provenance. Stay-the-
   course datapoint.

## Course Corrections

| Pri | Area | Action |
|-----|------|--------|
| P0 | Schematic connectivity (prior P0) | Fix the global-label merge key at `connectivity/mod.rs:282-283` to key on net NAME (iterate the by-name map's entries, `format!("global:{name}")`); add a two-distinct-equal-count regression (VCC on 3 sheets + GND on 3 sheets must remain two nets). Only prior P0 byte-for-byte still open and unguarded. |
| P0 | Board DRC — ZoneFill honesty at the engine layer | Push CheckRun assembly + ZoneFillState consultation DOWN into the engine (`drc/checks/mod.rs:19,69`) so run_drc / daemon / MCP return the honest typed model instead of the legacy ZoneFill-dishonest DrcReport. |
| P0 | Doc orientation — CLAUDE.md + AI_CLI_MCP_TOOL_SURFACE.md | Delete the substrate-nonexistence claims (CLAUDE.md:50,80,83-84,254-257; AI_CLI_MCP_TOOL_SURFACE.md:45-52); replace with landed-slice statements pointing at the PROGRESS substrate-readiness table; correct ~115→~14 sites; restate the real gaps as exposure (legacy daemon bypass) + the genuinely-absent DatumToolSession/DatumContextEnvelope. |
| P1 | MCP/daemon legacy bypass (prior P0 residual) | Retire (or formally defer-with-gate) the bypassing legacy daemon mutation tools (move_component/set_value/set_net_class/set_design_rule/delete_track) from the MCP catalog; point the daemon dispatch at commit_journaled or document it as a read/legacy island; add a gate ensuring no NEW bypassing daemon tool is catalogued. |
| P1 | Library/pool governance (decision-008) | Owner ratifies/reconciles decision-008 vs the M0 Horizon model BEFORE further library type work; then promote footprints/pin_pad_maps to typed DomainObjects and introduce LibraryBinding/StandardsBasis/approval_state. |
| P1 | Normative spec root — CANONICAL_IR §4 | Rewrite §4 to the enacted serde-tagged Operation enum (118 variants) + OperationBatch + commit/commit_journaled + CommitDiff{Vec<ObjectId>} + journaled TransactionRecord; drop the dyn-Operation trait, old-value OpDiff, and Transaction{timestamp:Instant}. |

## Stay The Course

- Substrate commit/journal gateway — single commit() with capture-inverse →
  stage → recompute-revision → append-journal → promote → write-cursor order;
  model_revision correctly excludes evidence shards; ImportMap read-only at
  resolve. Strongest subsystem; keep extending breadth onto it.
- IR primitives — deterministic BTreeMap key-sort serialization, integer-nm
  units (externally verified), i32 tenths angles, UUIDv4 native / UUIDv5
  deterministic-import identity. Correct and unchanged.
- GUI renderer, BoardReviewSceneV1 scene contract, and visual regression
  harness (bless + exact/tolerance diff, 4 checked-in goldens) — intact.
- Routing kernel — 50 deterministic route_path_candidate strategies, read-only/
  native-first; route-proposal apply through ProjectResolver +
  apply_accepted_proposal with provenance and model_revision pinning. Add the
  still-missing determinism contract section.
- Gerber RS-274X export — %FSLAX46Y46*% / %MOMM*% / %LPD*% with raw-nanometer
  coords (1 nm LSB), G36/G37 regions, D03 flashes, C/R apertures — externally
  verified correct.
- Frozen import posture — KiCad import remains a one-time converter with UUIDv5
  identity and read-only import_key; the new footprint substrate-import slice is
  the correct converter template. Do NOT add KiCad/Eagle breadth.
- Proposal-first discipline — standards repair, route apply, and
  forward-annotation all build Draft proposals via create_draft_proposal_from_
  batch and never mutate directly; claim discipline clean.
- PROGRESS substrate-readiness table (PROGRESS.md:43-59) — the one doc artifact
  NOT understating progress; keep it as the live source of truth and point prose
  at it.

## Spec Adjustments Needed

| Spec | Change | Pri |
|------|--------|-----|
| `CLAUDE.md` (:50, :80, :83-84, :254-257) | Remove the substrate-nonexistence claims; state ProjectResolver/commit_journaled/OperationBatch/ComponentInstance/Operation enum exist (point to the PROGRESS substrate-readiness table); correct ~115→~14 write_canonical_json sites. | P0 |
| `docs/CANONICAL_IR.md` §4 (:123-176) | Rewrite to the enacted serde-tagged Operation enum (118 variants) + OperationBatch + commit/commit_journaled + CommitDiff{Vec<ObjectId>} + journaled TransactionRecord; drop the dyn-Operation trait, the old-value-bearing OpDiff, and the Transaction{timestamp:Instant} that contradicts determinism. | P1 |
| `docs/contracts/AI_CLI_MCP_TOOL_SURFACE.md` (:45-52, :52) | Delete the "no general Operation/OperationBatch enum, no single generic commit()" denial (the same doc admits partial implementation at :538-542); restate the real gap as legacy daemon bypass + missing DatumToolSession/DatumContextEnvelope; fix 115→~14. | P1 |
| `docs/decisions/PRODUCT_MECHANICS_008` + `docs/POOL_ARCHITECTURE.md` + `docs/LIBRARY_ARCHITECTURE.md` | Owner ratifies/amends decision-008 (still "draft for owner review" with 6 open Owner Questions) and adds a supersede/relationship banner stating whether the shipped Horizon Part/Package/Padstack model is the implemented foundation extended toward 008, or whether 008 sections are the migration target with a concrete first-proof-slice tracker. | P1 |
| `specs/PROGRESS.md` (:467, :1525) | Change passive_only_net from `[x]` to `[ ]` (or `[~]`) — the rule is absent in code; OR implement it and add to the required-codes corpus so the false status cannot recur. | P1 |
| `specs/NATIVE_FORMAT_SPEC.md` §8 (:643) and §6.1 (:185-186) | Either move the schema_version load gate into ProjectResolver to satisfy the now-explicit "unknown future version is a load error", or annotate that the gate lives only in `project validate`; remove created/modified from the §6.1 example; note board-domain native types are transitional/CLI-owned pending the engine write-gate the pool domain received. | P2 |
| `specs/M7_FRONTEND_SPEC.md` §1.7/§1.8/§2.5 | Amend §1.7/§1.8 to sanction the terminal-prefill journaled-CLI edit conduit and the Outputs lane as the canonical mutation path (cross-ref PRODUCT_MECHANICS_001/005); reconcile §2.5 by renaming BoardTextPrimitive.text_uuid to source_object_uuid or documenting it as a sanctioned exception. | P2 |
| `specs/ENGINE_SPEC.md` §1.1a-1.3 | Add inline "Batch 1 stub — spec-only, implementation deferred" banners at the head of each batch-1 type block (ModelAttachment/ImpedanceSpec/behavioural_models/supply_chain_offers). | P2 |
| `specs/SPEC_PARITY.md` + `spec_parity_manifest.json` | Add parity rows inventorying the shipped standards-aware DRC slice and the new pool/library Operation/CLI/MCP surface; note the green gate currently depends on committing the uncommitted SPEC_PARITY.md refresh alongside the code. | P2 |
| `docs/decisions/PRODUCT_MECHANICS_000/000B/000C/000D/001` (PG-* harness) | Build `scripts/run_migration_proof_gates.sh` and wire it into `run_drift_gates.sh` (the code now backs PG-IDENTITY/PG-COMMIT/PG-PROPOSAL-PARITY), OR downgrade the "all gates wired/machine-enforced" language to "planned"; reconcile to a single harness name. Add an acceptance_path field to CommitProvenance or ratify the CommitSource/ProposalSource/applied_transaction_id mechanism as its definition. | P2 |
| `docs/audits/scope-integration/DOC_CODE_PARITY_DELTA_REPORT.md` | Re-date and refresh stale counts (75/182 → live 182/270, point at `check_spec_parity.py --print`) OR banner as a historical 2026-05-25 snapshot superseded by SPEC_PARITY.md. (File is clean/unmodified at HEAD, not git-modified.) | P2 |

## Risk Areas

- Uncommitted-worktree fragility: essentially all progress is untracked/modified
  at HEAD `22aeebe`. A loss, bad rebase, or selective commit could drop the
  substrate (49 files), the datum.* surface, or the footprint converter.
  Committing code WITHOUT the SPEC_PARITY.md refresh would land a red parity
  gate (HEAD baseline 75/182/53/64 fails all 4 rows against live code).
- Concurrent Codex authorship: this audit is a moving-target snapshot. The drift
  gate transiently flipped during the audit window; line numbers drift between
  runs. Re-verify any specific file:line before acting.
- Fabrication-trust hazard: the committed-engine DRC false-passes on unfilled/
  stale copper pours (`drc/checks/mod.rs:19,69`) while daemon/MCP consumers
  receive that dishonest shape. No real zone-fill solver exists; the
  projection==export oracle can pass on copper omitting real imported fills.
- Data-integrity hazard: the global-label root-count merge
  (`connectivity/mod.rs:283`) silently fuses distinct global nets with equal
  sheet counts — feeds wrong connectivity into ERC, DRC, and export. Unguarded
  by any test.
- Governance risk: ~1300 lines of new library/pool code shipped on the
  unratified Horizon model while decision-008 (the stated native truth) is draft
  — deepening investment in a model the ratified direction intends to supersede.
- Orientation hazard compounding: stale docs denying the substrate could cause a
  fresh agent (human or AI) to re-implement landed native-first work or
  mis-scope; the docs are now more wrong than at the prior audit.
- Convergence-debt accumulation: the daemon still imports zero substrate and
  grew by legacy verbs, not substrate convergence — new bypass sites can still
  appear faster than old ones retire.

## Limitations

- Read-only audit at uncommitted worktree state: HEAD is `22aeebe`; ~268
  modified + ~181 untracked files. The entire substrate/ module and most
  headline capability are untracked `??` or modified `M`. All file:line evidence
  describes worktree state, not committed code, and is labeled
  committed-vs-untracked where load-bearing. None of this is verified to survive
  into a commit.
- Concurrent Codex authorship makes this a point-in-time snapshot; some line
  numbers drifted between verification runs and at least one gate state changed
  during the audit window. Per-subsystem findings were adversarially verified
  (each domain has an upheld/refuted/corrections verdict block); a small number
  of cosmetic citation errors were corrected by the verifier.
- No build/test execution beyond targeted `cargo check`/`cargo build` (gui
  crates and cli compile clean) and gate runs; conformance is assessed by code
  reading + grep + git status, not by running the full test suite or exercising
  designs end-to-end.
- External-standard checks were limited to arithmetic (nm units), Gerber RS-274X
  structure, and Excellon header conventions where load-bearing; no exhaustive
  format-conformance testing against real fabricators.
- Where a finding's own evidence was refuted by its verifier (the
  "private-writers gate RED" and "committed-invokes-missing-gate" meta claims,
  and a fabricated `route_proposal_substrate.rs` engine path), those refutations
  were honored and the refuted items dropped from the P0/risk narrative.
