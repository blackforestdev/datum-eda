# Product Mechanics: Open-Question Research Resolutions

Status: research-funded resolutions 2026-06-19; recommendations for owner
ratification, not yet folded into the source decision docs.

## Purpose

These resolve the genuinely-open forks across `PRODUCT_MECHANICS_000`
through `PRODUCT_MECHANICS_012` and the six tool contracts
(`docs/contracts/*`) with production-grounded recommendations. Each entry
is a recommended path plus a named production example and the hard
engineering reason, supplied so the owner can ratify rather than
re-research. Nothing here is folded into the source decision docs yet;
ratified items should be migrated later (see closing note).

Questions explicitly marked resolved/decided by the 2026-06-18
reconciliation in the source docs are NOT re-litigated here; they were
skipped and counted as out of scope for this pass. The ratified mechanism
spine in
`docs/audits/scope-integration/DATUM_PRODUCT_MECHANICS_IMPLEMENTATION_READINESS_AUDIT.md`
(one resolved `DesignModel` via `ProjectResolver`, `ObjectId = Uuid`
persist-on-create, `ComponentInstance` join, typed `Operation`s into one
`commit()` + journal, `model_revision`, Import Map `import_key`, `ZoneFill`
honesty, lean tooling) constrains every resolution; none contradicts it.

## Ratification Legend

- research-conclusive: evidence settles the fork; ratification is a
  formality unless new evidence appears.
- recommended-default-pending-ratification: a strong recommendation, but
  the owner may override on product judgment; a concrete default is given.
- genuine-preference-needs-owner: research narrows the field but owner
  taste decides; a recommended default is still given.

---

## 1. Object identity and the integer-ObjectId fork

### Q: Adopt an allocated integer `ObjectId(u64)` diff-size optimization, and if so global or project-local?
(recurs in 000 Q1, 000C Q2 identity half, 000D, 001 Q5)

- Recommended path: Keep `ObjectId = Uuid` as the permanent
  wire/identity authority; do NOT adopt an integer `ObjectId` for the
  foreseeable horizon. Global integers are rejected outright. If the
  diff-size optimization is ever revived, scope it strictly as a
  PROJECT-LOCAL allocated `u64` alias persisted in the project manifest,
  mapped 1:1 to the canonical `Uuid` and used only as a
  display/serialization encoding over the existing `ObjectId`-keyed
  index — never a second identity space.
- Production example: Horizon EDA keys every entity by UUID in both its
  on-disk JSON and its SQLite pool index and never added a global integer
  ID despite being multi-project (`docs/HORIZON_ANALYSIS.md`). KiCad is
  the instructive contrast: net codes are project-local integers
  recomputed per load and explicitly NOT stable identity, while v6+
  stable identity (KIID) is a persisted UUID — the tool that wanted small
  integers used them only for derived connectivity. Altium uses a
  project-local integer Component Unique Id, never global across an
  installation.
- Engineering reason: Verified `crates/engine/src/substrate/mod.rs:19`
  defines `pub type ObjectId = Uuid`, the Import Map is keyed by
  `import_key`, and `compute_model_revision` hashes object UUIDs directly.
  A global allocator is the single most dangerous construct in a
  single-writer-today / multi-writer-reserved design: two projects or two
  offline writers cannot allocate from one global counter without a
  coordination authority — the exact merge-conflict magnet the storage
  model exists to avoid. A UUID is collision-free without coordination,
  which is why it is simultaneously rename-stable, deterministic on import
  (v5), and safe under future multi-writer. The diff-size win is marginal
  (one 36-char field) and already neutralized by making rename/move
  identity no-ops.
- Residual risk: If 100k+ object boards later show UUID line-noise
  impairs human diff review, a project-local-integer display layer is
  additive but non-trivial. Mitigated because it can sit behind the
  existing `ObjectId`-keyed index without touching identity semantics.
- Ratification class: research-conclusive.

---

## 2. Storage model: segmentation, format, and acceptance bar

### Q: Is segmented (sharded) storage acceptable if the experience feels unified, and what diff quality is the acceptance bar?
(000C)

- Recommended path: Yes — segmented storage with a resolved in-memory
  authority is correct. The acceptance bar is the machine metric, not
  subjective feel: PG-SHARD-DIFF-ISOLATION (every unrelated shard
  byte-identical sha256 before/after a qualifying edit, and the
  changed-shard set equals the predicted set exactly). "Feels unified" is
  a non-gate UX property; byte-identical isolation is the hard gate.
- Production example: Horizon EDA stores each symbol/package/part/padstack
  as a separate UUID-named JSON file plus a SQLite index yet presents one
  coherent project — segmented storage, unified experience, shipping.
  Altium's `.PcbDoc`/`.SchDoc` are OLE compound binary blobs git cannot
  diff/merge, which is why Altium teams depend on the A365 server instead
  of VCS.
- Engineering reason: `ProjectResolver::resolve()` walks `project.json`
  into shards, materializes one `ObjectId`-keyed `DesignModel`, and
  computes one `model_revision` over all shard hashes + object revisions.
  Unified experience is a property of the resolved model downstream of
  resolution, independent of file count. `content_hash` per
  `SourceShardRef` is already computed, so "small reviewable diff"
  reduces to a CI assertion, not a taste call.
- Residual risk: Shard boundaries can silently harden into hidden
  authority boundaries if a future surface reads a shard directly instead
  of via the resolver (the current KiCad-text-plus-sidecar patch path is
  exactly this defect today — a transitional defect, not an accepted
  authority; native shards are the only authority, see
  docs/DATUM_PRODUCT_MECHANICS.md "Interop Boundary And Import Posture").
  Mitigated only by the "no private file writer" invariant and by the fix
  that import writes native shards directly.
- Ratification class: research-conclusive.

### Q: Human-readable files first, or some shards database-backed from the beginning?
(000D)

- Recommended path: Human-readable deterministic JSON for ALL source
  shards in the first cut, including the journal (already JSONL at
  `.datum/journal/transactions.jsonl`). Reserve SQLite strictly for
  DERIVED, rebuildable indices/caches (search index, resolved-graph
  cache, pool query index). No source authority is database-backed in v1.
- Production example: Horizon EDA: source of truth is human-readable
  UUID-named JSON, SQLite is only a rebuildable queryable index
  (`docs/HORIZON_ANALYSIS.md`). KiCad keeps all design source as text
  s-expressions with no DB authority. Altium/Allegro are the
  counter-examples (binary/DB `.PcbDoc`/`.brd`) and pay with
  un-diffable, un-mergeable source that forces a separate server for
  collaboration.
- Engineering reason: The storage model's justification (diffable,
  reviewable, git-friendly, recoverable, AI-parseable) collapses if a
  source shard is an opaque DB file. The substrate is already built on
  JSON: `read_json_value`, `to_json_deterministic`, sha256 per shard,
  JSONL journal. PG-SHARD-DIFF-ISOLATION's byte-identical metric is only
  meaningful for text. A DB-backed source shard also defeats the
  stage->fsync->rename commit because atomic rename of a live DB file is
  not an atomic logical commit.
- Residual risk: At very large scale, parsing many JSON shards on open
  may be slow; mitigated by the rebuildable SQLite index/cache and
  revision-keyed staleness. If JSON parse cost ever dominates, a binary
  CACHE (not authority) can be added without changing the authority model.
- Ratification class: research-conclusive.

---

## 3. Commit, journal, and crash-safety substrate

### Q: What crash/recovery behavior is mandatory before users edit real projects?
(012 Q3)

- Recommended path: Mandatory before real-project editing: the full
  QG-CRASH-RECOVERY contract — staged-bytes, journal-append-fsync commit
  point, atomic rename — such that (a) kill before journal append leaves
  no committed mutation, (b) kill after append before promotion rolls
  forward or opens resolver recovery, (c) corrupt/missing staged bytes
  never yield a half-old/half-new model, (d) recovery logs identify
  transaction id, touched shards, and before/after hashes. This is
  non-negotiable; perf and breadth may be experimental, data integrity
  cannot.
- Production example: This is the WAL + fsync + atomic-rename pattern
  SQLite uses, and Datum already stores its pool in SQLite so the
  precedent is in-tree. Altium uses periodic autosave + crash-recovery
  prompt; KiCad writes `.kicad_pcb-bak` and `_autosave-`. Datum's
  per-transaction journaled commit is stronger than both.
- Engineering reason: The whole spine rests on one `commit()` that
  appends a journal `TransactionRecord`, fsyncs the commit point, then
  atomically renames staged shards. Verified `journal.rs` does
  `to_json_deterministic`, `std::fs::rename`, and append-mode writes.
  Without crash-safe commit a power loss mid-edit can produce a split
  project the resolver correctly refuses as truth — the user loses their
  board. Integrity is the one property that cannot be experimental
  because its failure is unrecoverable.
- Residual risk: The current `commit_journaled` ordering must be AUDITED
  to confirm fsync happens between journal-append and shard-rename (tests
  prove replay, not power-loss ordering); a fault-injection harness is
  needed to prove the gate, not just unit tests.
- Ratification class: research-conclusive.

### Q: What minimum transaction/diff data supports useful git review without churn, and what minimum diff format for geometry?
(001 Q4, 001 Q10)

- Recommended path: Keep the journal append-only JSONL (one
  `TransactionRecord`/line, `to_json_deterministic`). Each line carries:
  `transaction_id`, `batch_id`, before/after `model_revision`, provenance
  (actor, source, acceptance_path), and a typed diff. Extend
  `CommitDiff{created,modified,deleted: Vec<ObjectId>}` with a per-object
  field delta `{object_id, field_path, before, after}` for geometry,
  emitted as integer nanometers (the IR unit base), not strings.
  Created/deleted carry the full object snapshot (as legacy undo already
  stores whole `Track`/`Via`); property edits carry field deltas only. Do
  NOT embed full re-serialized shards or large derived blobs inline —
  reference big blobs by hash. The reviewed human git source-of-record is
  the SHARD files (minimal per-edit diff); the journal is the audit log.
- Production example: KiCad stores undo as `PICKED_ITEMS_LIST`
  whole-object snapshots flagged CHANGED/NEW/DELETED, and does NOT commit
  undo history to the repo — the text s-expr file is the reviewed
  artifact. Altium's ECO/comparator produces structured per-object
  old/new change records. Horizon stores pool/blocks as JSON for
  git-diffability.
- Engineering reason: Geometry coordinates are already integer-nm, so
  emitting integers keeps `model_revision` recomputation and diff replay
  deterministic. Object-id + field-delta is the smallest representation
  supporting machine diff AND reconstructing undo without whole snapshots
  for property edits, while full snapshots cover create/delete.
  Deterministic shards give one-line git diffs for a one-field edit; the
  JSONL journal supplies the audit trail KiCad lacks, appending exactly
  one line per edit/undo.
- Residual risk: Whole-object snapshot create/delete inflates the journal
  for large objects (filled zone); reference big objects by hash. A
  long-lived `transactions.jsonl` grows unbounded — journal
  compaction/checkpointing keyed to a `model_revision` snapshot is
  deferrable but the format must not preclude it.
- Ratification class: recommended-default-pending-ratification.

### Q: Confirm the substrate (operation enum + commit + journal + identity) is its own foundational slice gating all per-domain authoring tools.
(AI_CLI_MCP_TOOL_SURFACE Q1)

- Recommended path: Confirm: the substrate is a single foundational slice
  gating all five domains. The doc text ("NOT IMPLEMENTED") is stale —
  the core has landed (`OperationBatch`, `DesignModel::commit()`,
  `commit_journaled` with staged-write + fsync + atomic-promote,
  `TransactionRecord`, `ComponentInstance`, `import_key`,
  `model_revision = sha256`). Grow the `Operation` enum domain-by-domain
  ON TOP of the landed commit/journal/identity core; never re-scope the
  core.
- Production example: Horizon EDA: every interactive tool emits a typed
  document action onto ONE undo stack; no per-editor save path. Altium's
  data model is single; OutJob/Inspector never write privately.
- Engineering reason: Determinism, crash-safe durability, and undo/redo
  all derive from a single `commit()` + journal producer of
  `object_revision`/`model_revision`. If per-domain authoring landed
  first on private `write_canonical_json` paths, every domain would
  re-implement durability incompatibly and `model_revision` would be
  unanchored. Verified: the enum has only two arms today
  (`BumpObjectRevision`, `SetBoardPackageValue` at
  `substrate/mod.rs:90-91`).
- Residual risk: Real risk is the per-domain `Operation` enum growing
  inconsistently (board ops bypassing `apply_operation`). Mitigate by
  making `apply_operation` the single match arm any new op must extend.
- Ratification class: research-conclusive.

---

## 4. Multi-writer, partitioning, and merge

### Q: What merge/conflict behavior is required before multi-writer is safe, and should journal/relationship partitions be per-owner/per-session now?
(000D merge fork, 000C merge fork, 000D partitioning fork)

- Recommended path: Do not build multi-writer for the first cut
  (single-writer-per-project stands), and keep partitioning COARSE: a
  single append-only journal (already `transactions.jsonl`) and a single
  deterministic relationships shard, because single-writer means no
  concurrent appender. Before multi-writer is considered safe these must
  hold: (a) per-object stable UUID identity [done], (b) deterministic
  byte-stable serialization [done via `to_json_deterministic`], (c) the
  append-only journal with the parent-txn DAG field populated so
  divergent histories are explicit [field exists, DAG unused],
  (d) record-granularity conflict detection on the relationships layer
  surfacing same-object/same-relationship collisions as
  `UnresolvedMismatch` for explicit arbitration. Tie per-owner/per-session
  partitioning to the SAME trigger as multi-writer; design records keyed
  by owning `ObjectId` now so the future split is physical re-bucketing,
  not a schema change.
- Production example: No PCB tool has safe object-level git multi-writer
  — Altium/Allegro punt to a central server because binary formats make
  it impossible. The applicable precedent is git/Fossil append-only
  content-addressed history + three-way merge and CRDT/OT, all requiring
  stable identity + deterministic serialization + an explicit divergence
  point. git itself sharded (pack files) only when scale demanded; no EDA
  tool pre-shards an undo journal by session for a single user. Horizon's
  UUID-per-file gets clean merges only because unrelated objects are
  unrelated files.
- Engineering reason: `TransactionRecord` already carries before/after
  `model_revision` and a `CommitDiff` of created/modified/deleted
  `ObjectId`s; `commit()` enforces an `expected_model_revision`
  optimistic-concurrency guard (rejects a stale revision without mutation
  — the seed of conflict detection). Missing: the parent-txn DAG link and
  relationship-record-granularity collision detection. Building per-owner
  partitioning now adds resolver complexity (deterministic ordering of
  many segments) with zero first-cut benefit and risks perturbing the
  byte-identical isolation gate.
- Residual risk: Owner decides if/when multi-writer is built at all — may
  never be needed if single-writer + branching suffices. If it arrives,
  single journal/relationship files migrate to partitioned segments via
  deterministic owner-keyed re-bucketing replay (not reauthoring). Owner
  ratifies the deferral.
- Ratification class: recommended-default-pending-ratification.

---

## 5. Workspace state, profiles, and viewports vs design authority

### Q: Where should workspace/layout state live (project tree, user config, or both) and what is the default write target?
(000D workspace fork, 007 Q2)

- Recommended path: Both, split by scope with explicit `owner_scope` (the
  schema already declares project default | user local | shared profile).
  Default WRITE target is the user-local sidecar (per-user/per-machine
  ephemeral state: camera, open tabs, window geometry, navigation stack)
  under user config (XDG `~/.config`) keyed by project id;
  project-shareable workbench defaults are opt-in and live in the project
  tree in a non-authoritative namespace (e.g. `.datum/workspace/`,
  gitignored-by-default). Never write user-local layout into versioned
  design shards; `ProjectResolver` never reads either as design authority.
- Production example: Altium: `.PrjPcb` references shared project
  view/output config while per-user desktop layout lives in DXP
  Preferences (`*.DXPPrf`) in the user profile. KiCad: project-shared
  settings in `.kicad_pro` (committable) vs per-user state in user
  config, with KiCad recommending gitignoring volatile per-user bits. VS
  Code: `.vscode/` (shared) vs user settings.
- Engineering reason: The ratified spine forbids workspace state
  advancing `model_revision` and lists workspace layout among things that
  must never become a second authority. The resolver today loads only
  `project.json` + design shards + journal — no workspace concept, which
  is correct. Making user-local the default write target structurally
  guarantees a layout change can never touch a versioned shard;
  project-scoped defaults remain available for teams as explicit opt-in.
- Residual risk: User-local-by-default means a teammate gets no shared
  layout unless one is authored (acceptable, matches Altium). Owner must
  decide whether project-tree workspace files are committed or gitignored
  by default — recommend gitignored with explicit opt-in to share.
  Cross-machine user-local paths must degrade gracefully.
- Ratification class: recommended-default-pending-ratification.

### Q: Should viewports become first-class persisted design objects early, or wait for documentation/manufacturing work?
(000)

- Recommended path: Wait. Do NOT make viewports first-class persisted
  design objects in the first editor milestone. Persist transient
  camera/pane framing as non-authoritative workspace state, and promote a
  `Viewport` to a first-class `model_revision`-traceable object only when
  work that consumes saved framing as output lands (fabrication/assembly
  drawings, documentation sheets — M8-class). A saved review camera is
  workspace; an assembly-drawing sheet frame is an artifact-adjacent
  source object.
- Production example: Altium: interactive view state lives in
  workspace/desktop state; a Draftsman document (persisted, versioned
  drawing sheet with saved board views) is a first-class project document
  created only for documentation work. KiCad: editor viewport is
  ephemeral, but a Drawing Sheet (`.kicad_wks`) and the plot/print frame
  become real files only at the documentation/output stage.
  OrCAD/Allegro treats view windows as session state, Documentation
  Editor sheets as persisted artifacts.
- Engineering reason: 000D classifies camera positions and pane layout as
  non-authoritative workspace state that must be isolated from design
  source. First-class viewports early would either put them in design
  shards (violating isolation) or require a second persistence path
  before any consumer needs revision-traceable framing. The substrate has
  no viewport concept and needs none for Slice 0. Later promotion is
  cheap because viewports-as-output will key off `model_revision`, which
  commit already produces.
- Residual risk: If documentation output is pulled forward, a
  viewport-as-artifact schema must be designed then; low risk because it
  is additive and an Artifact/Output-Job concern, not an identity/commit
  concern.
- Ratification class: recommended-default-pending-ratification.

### Q: Workbench profiles: project-defined, user-defined, or both, and which arrangement/persistence modes ship first?
(007 Q1, 007 Q4, 000B tab-layout fork)

- Recommended path: Profiles: both, three-tier (built-in/user/project)
  resolved explicit-user-selection > user-local-default >
  project-defined-default > built-in; ship the eight built-in named
  defaults so a fresh project is usable with zero setup. Arrangement
  modes: ship tabbed views + recursive tiled split (vertical/horizontal
  split nodes of the `LayoutGraph`) + one saved workbench profile restore
  in the first slice. Pinned sidecar second (model it as a
  non-resizable/collapsible split node, not a separate primitive).
  PiP/floating/multi-monitor third — they need winit child-window
  management with zero proof yield. The documented first-proof slice
  (physical layout tab + live Gerber/NC tab from one `DesignModel`) is
  satisfied by one recursive split.
- Production example: Altium: default workbench is tabs + Split
  Vertical/Split Horizontal with named Workspace arrangements;
  floating/undock is a secondary power-user mode; built-in + user-saved +
  A365 team-shared layouts coexist. KiCad uses fixed docked panels (AUI)
  with separate top-level editor windows — the isolated-document-windows
  anti-pattern 007 rejects, making Altium's in-app split the better
  precedent; KiCad has no profile system, reinforcing Altium as target.
- Engineering reason: A recursive split `LayoutGraph` (binary split tree)
  is the minimal type expressing focused/vertical/horizontal/tiled as one
  shape; PiP/floating/overlay are additive z-order/window-manager
  concerns. Profiles carry no authority (layout templates + tool
  defaults, not source shards) so all three ownership tiers cost only a
  resolution-order rule + the `owner_scope` tag already in
  `WorkspaceState`; none can advance `model_revision`.
- Residual risk: Deferring floating/multi-monitor may disappoint the
  dense-multi-monitor reviewer; mitigate by designing `LayoutGraph` node
  types for floating/monitor placement now even if the renderer ships
  them later. A project profile referencing missing tabs/output-jobs must
  degrade (open available, skip missing), tied to the workspace
  schema-degradation policy.
- Ratification class: recommended-default-pending-ratification.

### Q: Workspace schema versioning and degradation; should workspace changes be journaled?
(007 Q6, 007 Q7)

- Recommended path: Workspace/profile records carry an independent
  `schema_version`, load best-effort and version-tolerant (unknown fields
  ignored, missing defaulted, unresolvable references dropped,
  degrade-to-default never error). A workspace that fails to parse is
  discarded with a diagnostic and the project opens with a default
  profile; a workspace failure NEVER triggers resolver recovery (that is
  for source authority only). Do NOT journal workspace changes into the
  design transaction journal — workspace uses its own `workspace_revision`
  record. The one audit-relevant exception (which output-job/variant was
  active at artifact generation) belongs on Artifact metadata, not a
  workspace journal.
- Production example: VS Code treats workspace/UI state as best-effort: a
  corrupt/version-mismatched workspace falls back to defaults and the
  editor still opens; chat/UI state is local, never committed. Altium
  preferences are version-tagged and import with included-pages
  selection, falling back to defaults rather than blocking. No tool
  journals tab moves; Altium captures artifact context on the OutJob
  release record, not a UI-action log.
- Engineering reason: 007 mandates old layouts degrade safely instead of
  blocking project open; the spine puts source authority in the resolved
  `DesignModel` + journal, not the layout graph. Coupling workspace parse
  to project open would make a non-authority record block authority
  access. Injecting workspace events into the journal would corrupt its
  meaning (every undo is a compensating design transaction) and pollute
  `model_revision` derivation, which only transactions advance.
- Residual risk: Silent discard of a carefully arranged layout frustrates
  users; surface a "workspace reset due to incompatible version"
  diagnostic and keep a last-good backup. If a regulated customer later
  needs a who-arranged-what trail, add a physically separate append-only
  activity log, never the design journal.
- Ratification class: research-conclusive.

### Q: What project dashboard/status indicators are required before serious manual workflows?
(007 Q5)

- Recommended path: Required minimum status surface, all computable from
  the resolved `DesignModel`: project name + root path; active
  `model_revision` (short hash); dirty/uncommitted-preview state; active
  variant; active board/manufacturing plan; active output job; ERC/DRC
  status (clean / N findings / stale); artifact freshness
  (current/stale/missing) for the selected output job; and a distinct
  `workspace_revision`/stale-layout indicator. Always-visible priority:
  revision + check-status + artifact-freshness; relegate
  variant/plan/output-job to a project panel to avoid clutter.
- Production example: Altium's status bar + Projects panel + Storage
  Manager show active document, modified state, save/VCS status; the
  Messages panel surfaces ERC/DRC counts; the Project Releaser shows
  generated-vs-stale artifact state. KiCad's status bar shows
  grid/units/layer but lacks revision/check rollup — the Altium dashboard
  is the target.
- Engineering reason: Status surfaces must read the resolved model, not a
  tab/file; `model_revision` and the journal tip already exist in
  substrate (`DesignModel.model_revision`). Distinguishing
  `model_revision` from `workspace_revision` from `artifact-revision` on
  screen is the user-facing enforcement of the source/derived/workspace
  authority split the whole family rests on; without it a user cannot
  tell a stale projection from current copper (ZoneFill honesty made
  visible).
- Residual risk: Over-dense status bars get ignored; keep the three
  always-visible fields prominent and the rest in a panel.
- Ratification class: recommended-default-pending-ratification.

### Q: How much terminal session state should be restored with a workspace, and should it persist across app restart?
(007 Q3, 005 Q2)

- Recommended path: Restore terminal LAYOUT (the tab exists, its pane
  position, declared cwd/env profile, title, recent-command list, a
  bounded scrollback snapshot) but NOT live process state. On restore the
  tab reopens, allocates a fresh PTY in the project cwd, and optionally
  replays a bounded clearly-marked scrollback transcript as inert text.
  First slice: live PTYs persist across tab switches/navigation within
  one running process and are gone on app restart. No running process, no
  command re-execution, no design effect; restored state is
  non-authoritative workspace state per 007.
- Production example: VS Code's integrated terminal restores tab
  metadata, titles, and a scrollback snapshot
  (`terminal.integrated.enablePersistentSessions`) but relaunches the
  shell — it does NOT resurrect killed child processes. tmux (true
  process persistence via a server) is the counter-model the doc
  explicitly disclaims.
- Engineering reason: The spine: terminal text is not an edit API and
  terminal-originated changes still use proposals/transactions. A live
  PTY is kernel-owned process state; serializing a running shell with its
  job-control tree and open fds is not portable and the doc forbids
  reconstructing shell internals as product state. Metadata/scrollback
  snapshots are inert presentation data mapping onto 007 workspace state
  (zero design writes), never touching the journal or `model_revision`.
- Residual risk: Scrollback can contain secrets (echoed tokens) — bound
  it, store under project/user-local workspace state with
  shell-history-grade protection, and make it opt-out. Users expecting
  tmux-style persistence may lose long-running jobs on restart; set the
  expectation explicitly (PTY-real but not persistent).
- Ratification class: research-conclusive.

---

## 6. Import-to-native conversion and the board-write authority

POSTURE NOTE: Datum is an AI-augmented NATIVE EDA tool. Native shards are
the ONLY authority. Import is a one-time converter (KiCad file -> native
data); once converted the result is just a native board, with origin
retained only as provenance. There is no "imported board" state and no
ongoing imported-vs-native authority distinction. Import is FROZEN
(the M7 spike already imports a KiCad PCB with enough fidelity to
recognize every aspect of a board, which is sufficient for now). See
docs/DATUM_PRODUCT_MECHANICS.md "Interop Boundary And Import Posture".

### Q: What is the precise exit criterion of the dual-write deprecation window before the imported-KiCad authority flip?
(000)

- SUPERSEDED by docs/DATUM_PRODUCT_MECHANICS.md "Interop Boundary And
  Import Posture". There is no authority flip and no dual-write window:
  native shards are the sole authority from the start, and the current
  KiCad-text-plus-sidecar storage of imported boards is a TRANSITIONAL
  DEFECT, not a phased migration. The fix is "import writes native shards
  directly" (a one-time converter producing a native board), not an
  evidence-gated flip from imported-KiCad authority to native authority.
  Import fidelity does not gate native maturity; native readiness is
  judged solely by native author/edit/check/output capability. The
  optional future KiCad EXPORTER (native -> KiCad) is a separate, deferred
  concern and is not an authority question.

### Q: Format-duality ordering: retire the GUI board-text bypass against the KiCad save path first, or stand up a native-board DesignModel shard first?
(001 Q3)

- Recommended path: Stand up the native-board `DesignModel` shard as the
  sole authority and converge the GUI board-text helpers onto a typed
  `SetBoardText*` operation against it. The KiCad-text-plus-sidecar patch
  path is a transitional defect to retire by having import write native
  shards directly, not a parallel authority to be flipped later. The
  substrate already has `SourceShardKind::BoardRoot` and a proven
  journaled write op (`SetBoardPackageValue` stages, fsyncs, atomically
  renames `board/board.json`). The former GUI board-text helpers have now
  been retired: `gui-app` pre-fills journaled `datum-eda project
  edit-board-text ...` commands in the PTY, and the old
  `board_text_mutations.rs` private writer module has been deleted instead
  of preserved as a parallel authority.
- Production example: KiCad keeps an in-memory BOARD as authority and
  serializes to `.kicad_pcb` on save — it never edits s-expression text
  in place. Datum's `Engine::save` patching KiCad s-expr text through the
  sidecar is the anti-pattern: a board, once converted, must be a native
  board edited through native shards. Altium edits the binary PcbDoc
  object model in memory and emits on save, never round-tripping text.
- Engineering reason: The native shard already exists and the commit path
  already promotes `board/board.json` atomically, so converging board-text
  helpers onto a typed op against native authority has low marginal cost
  and unblocks PG-COMMIT-ATOMIC and PG-PROPOSAL-PARITY for the GUI surface
  immediately. The KiCad-text-plus-sidecar split-truth path produces no
  undo, no `model_revision` bump, and no provenance for edits, which is
  why it is a defect to remove rather than an authority to maintain in
  parallel. The optional future KiCad exporter is downstream serialization
  (native -> KiCad), not the source authority.
- Residual risk: Until import is reworked to write native shards directly,
  boards that arrived via the current converter path still carry the
  KiCad-text-plus-sidecar defect; the board-text op should target native
  object identity (not byte layout) so it is unaffected by how a board
  first entered the project. See
  docs/DATUM_PRODUCT_MECHANICS.md "Interop Boundary And Import Posture".
- Ratification class: research-conclusive.

### Q: Which existing GUI/editor helpers are accepted as temporary migration exceptions, and is retiring the 112 native board-write call sites the first PCB slice?
(002 Q7, PCB_LAYOUT_TOOL_CONTRACT Q1, PCB_LAYOUT_TOOL_CONTRACT Q9)

- Recommended path: No GUI board-text private-writer exception remains. The
  former `board_text_mutations.rs` helper module has been deleted and the
  drift gate now fails if `gui-app` reintroduces those retired helper names.
  No new helper may be added; any other private writer is a bug. Retiring
  the remaining native board-write call sites IS the first PCB-domain slice:
  migrate one verb first (draw-track, `routing_net.rs:91` -> typed
  `AddTrack`), prove `TransactionRecord` + `model_revision` bump + journal
  undo, then template the rest. Board-text GUI editing now routes through
  the journaled CLI path; dimensions and any remaining direct board-writing
  surfaces should be migrated rather than allowlisted.
- Production example: No external tool documents a sanctioned private
  writer (Datum-internal concern); the analogue is KiCad quarantining its
  legacy IO_MGR path behind an explicit format enum during migration then
  deleting it. Allegro/Horizon both run all board edits through one
  journaled mutation path with no per-edit-type private writer.
- Engineering reason: The audit named this correctly: the GUI board-text
  write path was a migration defect to retire, not a sanctioned shortcut,
  and private-writer exceptions can be normalized accidentally. Two
  divergent write paths mean native edits produce no undo, no
  `model_revision` bump, no provenance — so ZoneFill staleness, check
  invalidation, and artifact traceability (all revision-keyed) are wrong for
  natively-authored boards. `compute_model_revision` hashes objects +
  source_shards; a private writer mutates shards without bumping it. The
  retired board-text path proves the enforcement model: migrate the user
  entry point to journaled CLI/operation flow, delete the private writer, and
  fence the regression in drift gates.
  by a drift gate.
- Residual risk: Time-boxed exceptions silently become permanent —
  mitigate with a CI gate that fails when the allowlist is non-empty past
  its retirement date. The `Operation` enum has only two arms today; a
  half-migrated state where some board verbs are typed and others still
  native-write is the dangerous interim — gate PCB-authoring-parity
  claims on full board-domain migration.
- Ratification class: research-conclusive.

---

## 7. Direct-commit vs proposal boundary (CLI, GUI, AI, relationships)

### Q: Are CLI commands allowed to commit directly by default, and which manual GUI/AI/relationship edits require proposal review?
(001 Q1/Q2/Q8, 002 Q6, 004 Q4, AI_CLI_MCP Q3, SCHEMATIC Q3, LIBRARY Q7)

- Recommended path: Single unified rule keyed on blast radius, not on
  surface or verb: direct-commit any edit that is (a) local to one
  object/tight visible cluster, (b) immediately visible on the active
  projection, (c) reversible by one compensating transaction, (d) within
  one domain. Everything else is proposal-first: relationship/
  `ComponentInstance` binding changes, `LayoutDeviation` and
  accepted-deviations (they suppress an `UnresolvedMismatch`),
  `ReverseEngineered` tagging in bulk, standards/DRC/ERC waivers and
  deviations, import-repair of source geometry, ECO/back-annotation,
  panel-to-board promotion, multi-object batch transforms,
  cascading/cross-domain deletes, and any edit that bumps an object the
  user is not looking at. CLI: direct-commit by default for individual
  local single-object mutations with a universal `--preview`/`--dry-run`;
  batch/destructive/cross-domain/repair commands invert to emit a
  Proposal artifact requiring `--apply` (the route-proposal CLI already
  proves this). GUI: same predicate; a single visible delete is direct
  (undoable), a cascading delete is proposal/confirm. AI/agents: NO
  auto-apply in the first slice — every AI mutation needs explicit
  per-proposal human approval; a direct-commit-by-policy capability token
  defaults OFF for every non-User actor. Library field edits: trigger on
  binding-relevant field changed (geometry/pinmap/role/padstack) of a
  placed object, not mere placement. Both paths call the identical
  `commit()` and differ only in recorded `acceptance_path`.
- Production example: Altium commits manual canvas edits (move, route,
  edit pad) directly with undo, but routes ECO-class/cross-domain changes
  through the Engineering Change Order dialog (Validate then Execute) and
  DRC waivers through an explicit reviewed action with stored
  justification; even fully tool-computed schematic->PCB changes go
  through ECO, never auto-applied. KiCad commits move/route directly but
  gates Update PCB from Schematic and back-annotation through
  confirm-first diff dialogs; DRC exclusions are deliberate recorded
  actions. KiCad's `kicad-cli` is non-mutating for design files.
  Cursor/Copilot/aider default to diff-then-approve with auto-apply
  behind explicit allowlists. Datum's route-proposal and
  `forward_annotation_apply_review` commands already encode
  draft-then-explicit-apply.
- Engineering reason: The enforcement invariant is that both paths
  terminate at the identical `commit()` and differ only in
  `acceptance_path`; the classifier must be a function of blast radius and
  cross-domain implication, computable from the diff
  (`CommitDiff.deleted`/`modified` spanning multiple domains), not of
  GUI/CLI/AI surface. Relationship/`ComponentInstance`/`LayoutDeviation`
  edits mutate the resolver's derived-status recomputation (convert
  `UnresolvedMismatch` to accepted), so they have non-local effects a user
  cannot see in one viewport — exactly the proposal trigger. `commit()`
  already carries `expected_model_revision` optimistic-concurrency and
  `CommitProvenance{source}`; a modal preview on every set-value would
  make scripting hostile. Auto-apply for AI is safe only when an op is
  reversible by one compensating transaction with no hidden cross-domain
  reach — a per-op-type proof, not a blanket grant. A cosmetic library
  field (`display_name`) cannot affect placed geometry, so "has a placed
  binding" alone over-triggers.
- Residual risk: The "tight visible cluster" vs "batch" boundary is fuzzy
  (drag-with-attached-routing, cascading delete); make cascade scope
  visible in confirmation (N objects across M domains) using
  `CommitDiff`. Detecting "rewires existing nets" needs a pre-commit
  connectivity diff; until it exists, conservatively escalate
  draw-wire/label on already-connected nets to proposal. A scripted loop
  of individual direct commits can aggregate into a destructive batch —
  mitigate with a per-invocation object-count threshold that
  auto-escalates to proposal (threshold owner-tunable). Eventual AI
  auto-apply allowlist scope is owner taste; require each allowlisted
  op-type to carry a recorded reversibility proof. `ReverseEngineered`
  bulk tagging is high-volume — make the import session the reviewed
  boundary (one proposal seeds the board-first model). An unclassified new
  library field defaults to binding-relevant (fail-safe to proposal).
- Ratification class: recommended-default-pending-ratification.

### Q: Should back-annotation ever be a direct command, or always a proposal?
(003)

- Recommended path: Always a proposal by default. Back-annotation changes
  electrical intent (schematic-owned authority) from physical layout — a
  cross-domain authority crossing on the doc's proposal-required list. The
  only candidate exception is narrow, conventionally-accepted refdes/gate
  back-annotation (pin/gate swap, reference reassignment); even that
  defaults to proposal but MAY be offered as an auto-previewed
  one-click-accept proposal so the UX feels direct without bypassing the
  journal.
- Production example: Altium back-annotation is explicitly an ECO: Update
  Schematics from PCB opens the Engineering Change Order dialog for
  Validate/Execute, never a silent write. KiCad's Update Schematic from
  PCB (refdes/footprint/net-name) is a checkbox-gated dialog showing
  changes before applying. No mainstream tool back-annotates electrical
  intent silently.
- Engineering reason: Cross-domain edits are exactly what the proposal
  model gates (direct commits only for local/visible/undoable edits
  without hidden cross-domain implications). A direct back-annotation
  would let physical edits rewrite electrical authority without a review
  boundary, violating the core authority rule. Routing through
  Proposal -> `commit()` also gives back-annotation free provenance and
  undo via the journal.
- Residual risk: Heavy proposal friction for trivial refdes swaps could
  annoy users; the auto-previewed one-click-accept proposal preserves the
  spine while approximating direct-command ergonomics.
- Ratification class: research-conclusive.

---

## 8. Finding addressing, waivers, and deviations

### Q: Should waivers/proposals/repairs key on a deterministic finding fingerprint, and what waiver metadata is required?
(AI_CLI_MCP Q5, RULES_CHECKS Q2, 009 waiver-metadata fork, SCHEMATIC Q5)

- Recommended path: Adopt a deterministic fingerprint = stable hash over
  (`rule_code`, sorted affected `ObjectId`s, normalized per-rule
  geometric/role discriminator), NOT positional index and NOT refdes.
  Retire the (domain, index) addressing in `project_surface.rs`
  `explain_violation`. `AddWaiver` becomes a proposal-first journaled
  `Operation` keyed by fingerprint; keep the raw (rule-string,
  object-uuid) `WaiverTarget` only as a denormalized lookup hint. The
  author-waiver verb lives in the 009 unified rules/checks contract
  (`waive_finding`), NOT in any domain-private contract — the same op
  serves ERC and DRC. Required waiver metadata floor: rationale
  (non-empty), actor + timestamp (taken from the committing transaction,
  never captured independently), scope/target (finding fingerprint), and
  `acceptance_transaction_id`. Reserve `expires_at`, `review_policy`,
  `approval_state`/`signature_state` as optional for a later regulated
  overlay.
- Production example: KiCad DRC exclusions are stored keyed by a
  serialized violation signature (rule + involved item KIIDs) so they
  re-bind across re-runs and reordering, not by list position — Datum's
  fingerprint is the same idea using `ObjectId`/`ComponentInstance`,
  adding `expires_at`/disposition (KiCad exclusions lack expiry). KiCad
  stores ERC and DRC exclusions through one mechanism, not a
  schematic-private tool. Altium ties a waiver to the specific violation
  with a user comment and the design's modification history, no separate
  approver for a basic waiver.
- Engineering reason: Verified `explain_violation` is index-addressed
  (`project_surface.rs:326,329,334,374`) and `CheckWaiver`/`WaiverTarget`
  keys by raw object Uuid + rule string (`schematic/mod.rs:376-385`,
  matched in `erc/mod.rs:471` and `drc/mod.rs:127`) — stable across
  rename but positionally fragile via the report layer. A content
  fingerprint over stable ids is the only addressing that is
  rename/move-stable (via `ComponentInstance`) AND determinism-gate-safe
  (same `model_revision` => identical fingerprints). Binding
  actor/timestamp to the acceptance Transaction makes waiver provenance
  free and tamper-evident — it cannot be forged independently of the
  journal. A schematic-local author-waiver would duplicate the shared
  `AddWaiver` op and fingerprint scheme the 009 contract owns.
- Residual risk: Two genuinely distinct findings sharing (rule, ids) but
  differing only in sub-geometry must include that detail in the per-rule
  discriminator or they collide. A waiver whose underlying finding no
  longer reproduces (issue fixed) must become Superseded/Resolved, not
  silently dropped — needs the finding-state enum the contract flags as
  missing. Migrating existing object-uuid-keyed waivers to fingerprints
  needs a one-time re-association at load. ERC findings run over an
  in-memory `Schematic` not yet keyed to `model_revision`; the fingerprint
  scheme must cover schematic-object findings, not just board findings.
  Regulated users will later need approver/expiry — reserved as optional
  now so promotion is additive.
- Ratification class: research-conclusive.

### Q: What user action creates a LayoutDeviation vs a Waiver, and should ElectricalDeviation be a named primitive?
(003 deviation-vs-waiver fork, 003 electrical-deviation fork)

- Recommended path: Split by what the record suppresses. A Waiver
  suppresses a specific `CheckFinding` for a named rule against named
  object IDs ("this DRC/ERC violation is acceptable here"). A
  `LayoutDeviation` records that physical implementation intentionally
  differs from resolved electrical intent ("the copper/placement does not
  match the netlist on purpose") and flips the derived relationship
  status away from `UnresolvedMismatch`. Action mapping: "Waive finding"
  from check-results -> Waiver (extend existing `CheckWaiver`); "Accept
  layout deviation" from a relationship/mismatch finding ->
  `LayoutDeviation` authored-intent record. Do NOT introduce a named
  `ElectricalDeviation` primitive — model it as accepted-deviation
  metadata on the `Relationship`/`ComponentInstance` with a
  `domain=Electrical` discriminator, the same shape as `LayoutDeviation`.
  Both deviation records are proposal-first and carry actor, date,
  rationale, affected object IDs, check basis, variant/output context.
- Production example: Altium separates these in-UI: rule violations clear
  via Waiver (right-click -> Waive, stored with reason/scope), while an
  intentional electrical-vs-physical difference is a Differences
  resolution in the ECO engine (choose to NOT push a schematic change,
  leaving a tracked divergence). KiCad only has the waiver half (DRC/ERC
  Exclusions keyed by rule + items + hash) and no first-class
  layout-deviation, the gap Datum closes. No mainstream tool has a named
  electrical-deviation type distinct from a waiver/ECO-difference —
  Altium's one Differences mechanism covers both sides.
- Engineering reason: Verified the engine has
  `CheckWaiver{domain, target: WaiverTarget, rationale, created_by}` wired
  into `erc::waiver_matches` (`erc/mod.rs:471`) and `drc::waiver_matches`
  (`drc/mod.rs:127`), matching finding-suppression exactly. It should NOT
  be overloaded to mean intent-divergence because the waiver matcher keys
  on (rule, objects) while a `LayoutDeviation` keys on the
  `Relationship`/`ComponentInstance`/`NetId` join and feeds the resolver's
  status computation. A discriminated field on one accepted-deviation
  record keeps the journal/commit and resolver-status paths single-shaped
  instead of forking serialization and the resolver for a near-duplicate
  type. The audit constrains this: no new deviation state enters accepted
  vocabulary unless owner-ratified.
- Residual risk: A deliberate clearance shrink may be both a DRC waiver
  AND an intent deviation — the UI must make clear a deviation does not
  auto-waive the DRC marker it causes, or users see lingering violations
  after accepting. If electrical deviations later need different sign-off
  gates than physical, the domain discriminator grows into a real subtype
  (mechanical, not a migration, because the enum is explicit from day
  one).
- Ratification class: recommended-default-pending-ratification.

---

## 9. Multi-sheet, multi-board, scopes, and reuse

### Q: Minimum hierarchy model for the first proof, and should ElectricalScope/PhysicalScope be persisted or derived?
(000E Q1, 000E Q2, 003 scope-persistence fork)

- Recommended path: First proof = flat multi-sheet only (one
  `ElectricalScope` across two sheets, move a label between them) with NO
  new persisted primitives — `Sheet`, `SheetInstance{parent_sheet}`,
  `SheetDefinition`, `HierarchicalPort` already exist and
  `connectivity/mod.rs` resolves cross-sheet labels via union-find; the
  only new behavior is a typed `MoveObjectBetweenSheets` op that preserves
  `ObjectId`. Defer sheet-symbol/port hierarchy, then repeated
  parameterized instances, to later slices. `ElectricalScope` and
  `PhysicalScope` are DERIVED resolver concepts in v1, not persisted:
  `ElectricalScope` = the whole resolved schematic (one scope),
  `PhysicalScope` maps 1:1 to a `Board`. Persist them only when a gate
  genuinely needs a scope that is not 1:1 (Gate 4 daughtercard, where one
  `ElectricalScope` maps to two `PhysicalScope`s). Key nets/instances on a
  path-capable key (sheet-instance-path + object) from the start so N>1
  instances do not break the schema.
- Production example: KiCad resolves a flat multi-sheet design through
  global/hierarchical labels first; hierarchical sheet pins are a layer on
  top; repeated/parameterized sheet instances (distinct net/refdes paths
  via the sheet-instance path) are KiCad's last-added tier, and KIID-path
  keeps symbol identity stable across instances. KiCad has no persisted
  electrical-scope object — scope is the recomputed netlist per build.
  Altium derives scope from the compiled project (rooms/channels derived
  from hierarchy); Allegro's persisted partition scopes exist only in the
  SiP/system tier, confirming explicit scope objects are a later
  multi-board need.
- Engineering reason: 000E Non-Goals forbid implementing every hierarchy
  feature in the first slice; Proof Gate 1 only tests
  sheet-is-not-electrical-design. `connectivity/mod.rs` already does
  cross-sheet union-find but `schematic/mod.rs:551` says port/link import
  is not implemented — so hierarchical-port binding is unproven while flat
  label resolution is exercised. Persisting a scope before a gate needs it
  forces it to become an authority candidate the resolver must validate,
  contradicting the single-authority invariant; a derived scope is
  revision-keyed off `model_revision` with no shard writes. The resolver
  today hardcodes one schematic/one board/one rules path in
  `NativeProjectManifestShape`, so persisted scopes mean migrating the
  manifest before any gate requires it.
- Residual risk: Flat-only risks the resolved-scope abstraction hardening
  around a model that never needed instance-path disambiguation; mitigate
  by keying on a path-capable key now. A Gate-4 daughtercard needs a
  stable scope identity to attach Interconnect relationships — when it
  lands, promote `PhysicalScope`/`ElectricalScope` to persisted Uuid
  objects (like `Relationship`'s surrogate id); treat the derive-first
  slice as explicitly temporary. If the owner later wants user-named
  savable selection sets, a thin persisted `ScopeBinding` (`ObjectId`
  list + surrogate id) is the promotion path — design the derived form to
  compute from an explicit `ObjectId` list, not name/positional
  heuristics.
- Ratification class: recommended-default-pending-ratification.

### Q: What is the first supported multi-board case?
(000E Q3)

- Recommended path: First case = multiple cooperating boards in one
  product project WITHOUT inter-board electrical relationships (alternate
  board implementations of one shared `ElectricalScope` is the cleanest
  demonstrator), proving Proof Gate 2: two boards with stable identity,
  board-scoped DRC, delete/duplicate isolation. Main-board-plus-
  daughtercard (Gate 4) comes second because it additionally requires the
  Interconnect primitive and connector-to-connector pin-mapping checks.
  Give each board a stable Uuid and a nullable implements-scope reference
  from day one so Interconnect can later attach without reshaping the
  board shard list.
- Production example: Altium's Multi-Board Assembly (`.PrjMbd`)
  references child PCB projects, each keeping independent identity and
  DRC, with inter-board connectivity a separate later layer (Multi-board
  Schematic) — Altium shipped multi-board assembly before
  harness/logical-connection maturity, exactly this order. KiCad 8.x has
  no native multi-board project, confirming this is a differentiating
  feature and simplest-first is the safe entry.
- Engineering reason: Verified the engine holds exactly one board
  (`api/mod.rs:66` `board: Option<Board>`) and the resolver manifest has a
  single board string. Going from one to N independent boards is a pure
  cardinality change (manifest board list, board-scoped DRC, board Uuid
  identity) with no new cross-domain machinery. Daughtercard-first would
  force the Interconnect primitive, connector pin-mapping checks, AND
  multi-board cardinality at once. Gate 2's pass criteria (deleting one
  board does not delete the electrical design) need only cardinality plus
  the existing `Relationship`/`ComponentInstance` separation.
- Residual risk: Independent-boards-first may bias storage toward
  boards-as-peers and make the asymmetric main/daughter relationship feel
  bolted on; mitigate with the per-board Uuid + nullable implements-scope
  reference. This is genuine owner taste — an owner who sees daughtercard
  as the flagship demo could justify Gate-4-first at higher cost.
- Ratification class: genuine-preference-needs-owner (recommended
  default: independent cooperating boards first).

### Q: How much module reuse before first-class library packaging exists, and should inter-board cables/harnesses be modeled here?
(000E Q6, 000E Q8)

- Recommended path: Module reuse: support only project-local
  copy-with-new-identity (a "duplicate this scope/board as an independent
  unit" op minting fresh `ObjectId`s and `ComponentInstance` ids), plus
  optionally a by-reference `HierarchyInstance` for repeated channels
  within one project. Do NOT build cross-project module packages,
  versioned module pools, or update-propagation until the library system
  (008) lands; record provenance (`copied_from` scope/module id) on the
  copy so a future packaged-module feature can offer opt-in propagation.
  Harnesses: defer full cable/harness design to a separate track but keep
  the lightweight Interconnect primitive (an explicit `Relationship`
  binding two connector `ComponentInstance`s across `PhysicalScope`s,
  checkable for pin-mapping gaps) IN this family because Gate 4 needs it.
  Interconnect = in scope; wire-by-wire conductor design, splices,
  shields, AWG, backshells, formboard drawings = out of scope.
- Production example: Altium: project-local reuse is Snippets and
  Multi-Channel (Repeat); true versioned propagating modules are Managed
  Components in a Vault (library tier) — Altium shipped repeat/snippets
  long before update propagation. Horizon's pool blocks are a pool
  (library) feature. Altium and Cadence both split multi-board connector
  mapping (in Designer/Allegro) from dedicated harness products (Zuken
  E3.series, Capital Harness), confirming Interconnect belongs with
  multi-board while harness conductor design is its own track.
- Engineering reason: Module update-propagation needs stable cross-project
  module identity plus a diff/merge of module revisions into instances —
  the library system's job, tied to pool `library_revisions`. grep found
  no module struct in the engine. Copy-with-new-identity needs only the
  existing `commit()` emitting create ops with fresh Uuids;
  `HierarchyInstance` reuse needs the existing
  `SheetInstance`/`SheetDefinition` pair. Interconnect is just another
  `Relationship` (which the audit already gives a stable surrogate id and
  `RelationshipKind` enum) binding two existing `ComponentInstance`s,
  adding only a connector-pin-mapping check — reusing ratified machinery;
  harness modeling introduces a whole new 3D routed-conductor geometry
  domain the canonical IR has no representation for.
- Residual risk: Copy-with-new-identity loses the source link, so later
  bug-fix propagation is impossible without a module-source reference —
  mitigate by recording `copied_from` provenance even in the copy-only
  slice. An Interconnect recording only connector-to-connector binding
  cannot express pin-to-pin continuity through a non-1:1 cable, so
  cross-board ERC/continuity is incomplete for cabled interfaces until the
  harness track lands — document as a known Gate 4 limitation.
- Ratification class: recommended-default-pending-ratification.

### Q: What import/reverse-engineering workflow is the first board-first proof based on, and how much RE belongs in the first proof?
(000E Q7, 003 reverse-engineering fork, 011 import-slice fork)

- Recommended path: Base the board-first proof on the existing KiCad
  `.kicad_pcb` importer applied to `datum-test`, seeding every recovered
  `PlacedPackage` with an explicit `RelationshipKind::BoardOnly` (or
  `ReverseEngineered`) classification targeting its persisted `ObjectId`
  via `import_key` and NO schematic source, then infer exactly one
  `ComponentInstance` + one net relationship as a PROPOSAL (not accepted
  source state). Prove only that the binding survives save/load and
  resolves to a non-mismatch status. Because `datum-test` has a real
  schematic, the `ImplementedBy` seed using the real schematic is a viable
  SECOND slice to exercise cross-domain condition labels earlier. Defer
  inferred/proposal-driven schematic RECONSTRUCTION from geometry to a
  later slice. This records the converter-proof shape only; import is
  frozen (the M7 KiCad-PCB recognition spike is sufficient) and no further
  import slices are scheduled — the frontier is native authoring. See
  `docs/DATUM_PRODUCT_MECHANICS.md` "Interop Boundary And Import Posture".
  Use the real fixture, never fabricated geometry.
- Production example: KiCad's import of a board-without-schematic produces
  footprints with netnames but no symbol linkage — the BoardOnly state;
  Altium's import leaves components "not matched to schematic" until
  Component Linking (the proposal-review step). No tool fabricates
  schematic symbols from copper. Altium's reverse-engineering (Create
  Schematic from PCB) is a heavyweight separate starting-point generator,
  not an auto round-trip; KiCad has no automated PCB->schematic
  reconstruction at all. KiCad/Altium both treat foreign-tool import as
  full-board migration into native primitives.
- Engineering reason: The KiCad PCB importer already exists and is the
  project's determinism-gated primary import path (`import/kicad/mod.rs`);
  the memory rule mandates real `.kicad_pcb` fixtures. Building the proof
  on the real importer makes provenance (Import Map `import_key`) genuine.
  `ComponentInstance` is defined in `substrate/mod.rs` but referenced
  nowhere yet — the single-inference is its minimal real exercise.
  Inferred reconstruction needs the full
  `ComponentInstance`/`NetId`/PinPadMap join working for the forward case
  first; proving the relationship substrate (BoardOnly survives load,
  targets `ObjectId` not import seed) de-risks the harder inference. The
  audit requires inferred reconstruction be proposal-first with
  provenance.
- Residual risk: The KiCad importer currently sets variants empty and
  does NOT populate `ComponentInstance` or any `RelationshipKind` (grep
  confirms relationship/board-only/reverse terms absent from import) — so
  the proof requires NEW import-side seeding code routed through
  `commit()`/proposal, not just a test; the seeding must NOT write
  directly to shards. Under-proving RE risks discovering late that the
  identity model is awkward for board-first cases — mitigate by including
  one authored `ReverseEngineered` binding in the proof even though full
  reconstruction is deferred. Over-fitting the import model to KiCad's
  shape could leak source architecture into native primitives — mitigated
  by the hard rule that source architecture ends at conversion and
  `import_key` stays provenance-only, plus the Eagle importer as a second
  shape.
- Ratification class: research-conclusive.

---

## 10. Relationship vocabulary, manufacturing-output gating, RE labeling

### Q: Should ManufacturingOnlyObject be a distinct authored intent label, and should board-only vs panel-only share UI but not schema?
(000E Q4, 003 board-only-vs-panel fork)

- Recommended path: Do NOT add `ManufacturingOnlyObject` as a distinct
  `RelationshipKind`. Resolve by SCOPE, not label: panel-only features
  (rails, tabs, mouse bites, V-scores, panel fiducials, coupons, tooling
  holes added at panelization) live in panel/manufacturing-plan state and
  never enter board relationship state, so they need no relationship
  label. Board-level manufacturing features that ARE board source
  geometry (board fiducials, mounting holes) are `BoardOnlyObject` (the
  existing label already means "physical object intentionally has no
  schematic source"). Share ONE read-only inspector affordance ("this
  object has no schematic source") with a distinct sub-label (BoardOnly vs
  Manufacturing/Panel scope), but keep them distinct schema concepts:
  BoardOnly is a `RelationshipKind` on a board `DomainObject`; panel
  membership is scope membership. Create/promote actions differ
  (`MarkBoardOnlyObject` vs panel-scope assignment). If documentation
  later needs to distinguish fiducial/mounting-hole/coupon, use a
  non-authority `feature_role` attribute on the object, not a
  `RelationshipKind`.
- Production example: Altium models this by scope: panel
  fiducials/tooling holes/rails are objects of the Embedded Board
  Array / panelization, outside the source board's component/net model,
  while board-level fiducials are Special Strings/Components flagged
  not-in-BOM/not-fitted, not a distinct manufacturing-relationship type.
  KiCad places board fiducials/mounting holes as footprints with
  exclude-from-BOM/not-in-schematic (board-only), and panel features come
  from KiKit on a separate panel `.kicad_pcb` — one user-visible "no
  schematic source" signal, two storage realities.
- Engineering reason: The audit forbids introducing
  `ManufacturingOnlyObject` as accepted vocabulary unless owner-marked,
  and the doc's Panelization Boundary already keeps panel features in
  panel state — the scope boundary does the discriminating work a label
  would duplicate. Adding the label would expand the `RelationshipKind`
  enum and risk two labels the resolver cannot deterministically
  disambiguate for the same fiducial. Fewer enum variants = fewer
  resolver branches = simpler determinism. Sharing schema (not just
  affordance) would let panel-only geometry leak into board DRC, violating
  the panelization-isolation gate; the distinction is load-bearing for the
  export scope filter.
- Residual risk: A shared badge can mislead users into thinking a panel
  rail is board-only and promotable in place — the affordance must surface
  the scope so the wrong promote op is not invoked. How much to merge
  visually is genuine UI taste; the engineering recommendation fixes the
  schema split, the visual merge degree is owner-decided.
- Ratification class: recommended-default-pending-ratification.

### Q: What relationship/check findings must block manufacturing output by default?
(000E Q9, 003 output-gate fork)

- Recommended path: Block (hard error, non-zero exit) on any
  `UnresolvedMismatch` derived status for objects in the selected
  board+variant scope, and on any net/pin with no copper implementation
  not covered by an accepted deviation, waiver, or
  `SchematicOnly`/`BoardOnly` intent. Default-WARN (do not block) on
  `PendingImplementation` (intentional WIP) and on unfilled-but-not-stale
  zones already covered by ZoneFill findings — but make Pending
  configurable: default warn for board-only/reverse-engineered projects,
  block for projects that declared a schematic source. NEVER block on
  authored intent (`LayoutDeviation`, BoardOnly, SchematicOnly,
  accepted-deviation) — they are the explanation that resolves the
  condition — nor on `NotApplicableForVariant` (correct absence). The
  blocking set is a named output-job gate profile keyed to
  `model_revision` + `variant_revision`, evaluated POST-variant-
  resolution, overridable per OutputJob, shipping a "board-first" preset.
- Production example: Altium's Project Release runs a validation set and
  refuses to generate the released package if DRC/ERC/comparator errors
  exist, while Variations and approved waivers pass — unmatched components
  block, intentional variant DNP does not. KiCad's
  `kicad-cli pcb drc --exit-code-violations` is the community gate on
  unconnected/violation findings; KiCad's schematic-parity DRC flags
  unresolved footprint/symbol mismatches. Both block on mismatch, pass on
  declared intent. Datum should adopt Altium's severity-gated-output
  model, not KiCad's advisory-only default.
- Engineering reason: `UnresolvedMismatch` is a DERIVED status meaning
  electrical and physical disagree with no accepted decision — shipping
  copper for it is the dishonest-output failure the projection-honesty
  gate forbids. Authored intent records exist to convert a mismatch into
  an explained shippable state, so blocking on intent defeats their
  purpose. Keying the gate to `model_revision` + `variant_revision` (both
  in the spine) makes the block deterministic and lets the artifact record
  which revision passed. A variant making a mismatched object
  `NotApplicableForVariant` must clear the block for that variant only, so
  the gate evaluates post-variant-resolution.
- Residual risk: Board-first/reverse-engineered projects legitimately
  have many `PendingImplementation`/unmatched-net objects and would be
  over-blocked by a strict default — the warn-vs-block policy must be
  project-posture-driven, the gate profile must ship a board-first preset,
  and the owner must ratify whether a board-only project's missing
  schematic counts as Pending (blockable) or accepted BoardOnly intent
  (not blockable), or these users disable gating wholesale and defeat the
  honesty goal.
- Ratification class: recommended-default-pending-ratification.

### Q: How should reverse-engineered schematic reconstruction be labeled, and what visual language distinguishes intent/implementation/mismatch/deviation?
(011 RE-labeling fork, 003 visual-language fork)

- Recommended path: Reconstructed connectivity uses the ratified
  `RelationshipKind::ReverseEngineered` (authored), never silently
  `ImplementedBy`; inferred nets/symbols stay visibly distinct until a
  user accepts a relationship op binding them to intent. Inference
  confidence/basis/lossiness rides as import/provenance metadata or an
  accepted-deviation record, NOT a second reverse-engineering axis; the
  join always resolves on the surviving `ComponentInstance` surrogate,
  never refdes or `source_object_key`; AI-proposed reconstruction is a
  Proposal with diff + provenance, never authored truth. Visual language:
  a four-state encoding driven by derived relationship status + authored
  intent, consistent across schematic and PCB projections — (1) source
  intent unimplemented = thin dashed airwire/ratsnest; (2) implemented =
  normal solid copper/no overlay; (3) `UnresolvedMismatch` = high-salience
  error overlay (DRC/ERC error palette); (4) accepted deviation/waiver = a
  distinct muted "acknowledged" badge visibly different from both error
  and clean. Encode by SHAPE + ICON, not color alone, mapped onto the
  `gui-protocol` scene contract.
- Production example: No tool auto-reconstructs schematics from copper;
  the closest is Altium's PCB-to-schematic Import Changes /
  project-comparison, which marks unmatched/board-only items as
  differences requiring explicit acceptance before they become authored —
  the accept-before-authored gate. KiCad's ratsnest (thin lines, unrouted
  intent) vs solid tracks (implemented), DRC marker glyphs for mismatch,
  and dimmed/struck markers for excluded violations is the canonical
  four-state vocabulary; Altium uses green ratsnest, solid routed tracks,
  violation overlays, and struck/greyed waived violations.
- Engineering reason: 011 and the spine pin `ReverseEngineered` as one
  authored `RelationshipKind` among
  `{ImplementedBy, BoardOnly, SchematicOnly, ReverseEngineered, Pending,
  Mismatch}`, kept separate from derived status `{Implemented,
  PendingImplementation, UnresolvedMismatch}`. Reusing the existing enum
  (not a new RE state machine) prevents the relationship-vocabulary drift
  the audit names as the highest ongoing risk; `ComponentInstance` joins
  keep the label stable across rename/move. The visual layer should be a
  pure function of derived status + authored-intent presence so the same
  scene contract renders consistently in any projection and no view stores
  its own truth — and M7 screenshot-golden discipline favors discrete
  status->glyph mapping over freeform styling.
- Residual risk: Users may still mentally treat a `ReverseEngineered` net
  as real once rendered — mitigate by keeping derived status visible
  alongside the authored kind, retaining inference confidence in
  provenance, and requiring an explicit accept-relationship op to promote
  to `ImplementedBy`. Specific palette/glyph choices are owner taste and
  accessibility-dependent: the engineering rec fixes the state SET and the
  shape-not-color-alone rule, but exact glyphs need owner sign-off and the
  `gui-render` visual-regression harness must lock them.
- Ratification class: recommended-default-pending-ratification.

---

## 11. ZoneFill honesty and copper export

### Q: What is the ZoneFill honesty fix path, and is a real fill solver in M7 scope?
(002 Q4 zonefill consequence, PCB_LAYOUT_TOOL_CONTRACT Q2, MANUFACTURING_OUTPUT_TOOL_CONTRACT Q3)

- Recommended path: Two slices, fix honesty first. Immediately gate the
  G36 boundary pour in `export/copper.rs` behind `ZoneFill{Filled}` state
  so an unfilled zone emits NO copper plus a hard finding; ship
  `author_zone` boundary-only (`Zone` with `ZoneFill{Unfilled}` + hard
  no-copper finding) first; ship `fill_zones` (the derived copper solver +
  Filled/Stale/island state) as a DISTINCT later slice. The real fill
  solver is OUT of M7 manufacturing scope. Add the `ZoneFill` fill field
  to the `Zone` struct and the gate to `copper.rs` BEFORE shipping any
  copper-export proof, not after.
- Production example: KiCad: drawing a zone (Ctrl+Shift+Z) authors only a
  boundary outline; copper appears solely after the separate Fill All
  Zones (B) step, and an unfilled zone is visibly hatched/warned and
  contributes no copper to plots — it plots the outline on a non-copper
  layer, never the boundary as copper. Allegro dynamic shapes show
  un-poured and are excluded from artwork until updated. No production
  tool exports an unfilled boundary as copper.
- Engineering reason: Verified in `export/copper.rs` (the `G36*` open at
  line 181): the exporter emits G36...G37 over zone polygon vertices with
  NO Filled/Unfilled gate and no finding — fabrication-dishonest copper;
  every imported board with an unfilled zone would ship a solid pour where
  there is none. Gating on `ZoneFill{Filled}` is a small contained
  correctness fix independent of the large solver; coupling them would
  block the honesty fix on the solver. `board_types.rs` `Zone` has no fill
  field today, so the field plus the gate are the gating work.
- Residual risk: `datum-test` contains ZERO zones (verified grep zone =
  0), so the honesty finding and unfilled-no-copper behavior cannot be
  proven end-to-end on the canonical fixture — only by unit test on
  `copper.rs`. The owner must supply a zone-bearing real fixture (the
  no-synthetic-fixture rule forbids fabricating one) before the slice is
  provable end-to-end.
- Ratification class: research-conclusive.

---

## 12. Manufacturing projections, BOM/PnP, and the T0 oracle

### Q: Which manufacturing projections are mandatory first and in what order do they join the equivalence oracle?
(002 Q4, 000B Q4, 012 Q7)

- Recommended path: Mandatory first proof: Gerber copper (RS-274X),
  Excellon drill, and soldermask + paste apertures (the honesty-and-
  aperture-policy core). Equivalence-oracle join order beyond
  copper+drill: (1) soldermask, (2) paste, (3) BOM, (4) pick-and-place,
  (5) assembly drawing. Soldermask precedes paste (mask is simple
  copper-expansion; paste adds reduction/omission policy). BOM/PnP follow
  as row-level (non-geometric) comparisons; assembly drawing last
  (non-geometric PDF/visual, depends on `ComponentInstance`-joined
  BOM/PnP). Traceability (QG-GENERATED-ARTIFACT-TRACEABILITY) is mandatory
  for the copper/fab package (Gerber + Excellon + mask + paste) FIRST,
  BOM/PnP second, assembly/fab drawings third — any artifact a
  fab/assembler consumes to build the board must be traceable before it is
  product-ready.
- Production example: Altium's OutJob generates copper, then mask/paste
  from the same aperture core, then assembly drawings and BOM/PnP
  downstream — the same dependency order. KiCad's
  `kicad-cli pcb export gerbers` emits copper+mask+paste from one plot
  engine in one deterministic pass, then `export pos` (PnP) and BOM as
  separate placement-data generators. Every fab (JLCPCB/PCBWay) requires
  Gerber+Excellon+mask+(for assembly)paste. Altium's Project Releaser
  stamps every released output with project/config/variant + release
  revision; IPC-2581/ODB++ embed source/revision metadata.
- Engineering reason: Verified the engine implements these exact
  renderers: `export/copper.rs` (RS-274X copper), `export/excellon.rs`
  (drill), `export/mask.rs`
  (`render_rs274x_soldermask_layer`/`render_rs274x_paste_layer`). The
  honesty gate bites on copper export specifically, so copper MUST be in
  the first proof to drive the ZoneFill split; drill+mask+paste are the
  cheapest additional projections exercising aperture/plating honesty
  already implemented. Oracle cost is dominated by how far a generator's
  comparison semantics differ from the proven copper one — mask/paste
  reuse the aperture path and compare as geometry like copper; BOM/PnP
  introduce row-level comparison; assembly PDF introduces
  tolerated-formatting equivalence. The copper fab package, if
  untraceable, lets a stale/dishonest board reach fabrication (a
  wrong-revision Gerber scraps a panel; BOM/PnP errors are recoverable at
  assembly).
- Residual risk: Paste omission/reduction and mask expansion are
  rule-driven; if not modeled as explicit OutputJob/rule state the
  equivalence check could pass on geometry while policy silently differs —
  encode aperture policy as covered model state. Content-hash equivalence
  requires byte-deterministic export; any ordering/float-format
  non-determinism in the renderers produces false "changed" reports —
  confirm renderers deterministic (the import-determinism gate suggests
  the discipline exists).
- Ratification class: recommended-default-pending-ratification.

### Q: What minimum BOM/PnP columns and coordinate conventions are required, and confirm the ComponentInstance join-key migration?
(000F BOM/PnP fork, MANUFACTURING_OUTPUT_TOOL_CONTRACT Q4)

- Recommended path: BOM/PnP rows key on `ComponentInstance` (reference
  designator is a DISPLAY column, not the join key) and drift compares by
  `ComponentInstance`, not `expected_by_reference` — in scope NOW,
  accepting the one-time golden-CSV churn. Minimum PnP columns: Designator
  (from `ComponentInstance`), X, Y, Rotation, Side, Package/Footprint ref;
  coordinate convention: millimeters decimal relative to an explicit
  OutputJob origin, rotation CCW-positive with 0deg = component native
  orientation, placement point = component origin. Minimum BOM columns:
  line item, Quantity, Designator list (grouped by identical
  value+footprint), Value, Footprint, population/variant state
  (Fitted/Unfitted/DNP). Variant + side filters are explicit inputs.
  MPN/manufacturer/supplier are optional metadata, not required for
  equivalence.
- Production example: KiCad `kicad-cli pcb export pos` emits Ref, Val,
  Package, PosX, PosY, Rot, Side in mm with selectable origin — the de
  facto minimal PnP set; KiCad grouped BOM groups by value+footprint with
  a Designator list and Qty. Altium Generates-Pick-Place uses Designator,
  X/Y origin-relative, Rotation, Layer; ActiveBOM keys line items to the
  component unique id with designator as a displayed property. Horizon
  sources BOM from canonical part/component identity, not a free reference
  string.
- Engineering reason: Equivalence on BOM/PnP is row-level so the columns
  ARE the comparison contract — too few under-tests, too many inject false
  diffs from optional supplier metadata. The set is the smallest making a
  row uniquely placeable/orderable, keyed on `ComponentInstance` (mandated
  by the spine to replace ref-des joins) so variant population resolves
  three-valued. A refdes rename keyed by refdes shows as delete+add in
  drift instead of rename; `ComponentInstance` (now in `substrate/mod.rs`)
  is the stable join. The pre-implementation finding was that BOM/PnP were
  reference-string keyed (`command_project_inventory.rs`
  `NativeBomRow.reference`, drift on `expected_by_reference`); the current
  first slice now exports `component_instance_uuid` and compares BOM/PnP
  matched/missing/extra/drift evidence by `ComponentInstance` when present,
  with package UUID fallback only for board-only legacy rows. Fixing rotation
  convention (CCW-positive, native-0) and mm/origin removes the largest source
  of real-world PnP equivalence failure.
- Residual risk: Rotation convention is the perennial PnP footgun (CW vs
  CCW, native vs absolute); if Datum's stored rotation differs from a
  fab's expected convention the geometry can be equivalent yet wrong on
  the line — make rotation convention an explicit OutputJob setting
  recorded on the artifact and compare the resolved convention, not raw
  degrees. Imported boards must have `ComponentInstance` ids minted via
  `import_key` before BOM keying works; for a freshly imported board with
  no instance ids, BOM must fall back gracefully and visibly rather than
  emit empty keys — sequence `import_key` wiring before flipping the key.
- Ratification class: recommended-default-pending-ratification.

### Q: Should T0 report/manifest/export be unified now, should read-only artifact generation route through the daemon, and where do artifact files/metadata live?
(MANUFACTURING_OUTPUT_TOOL_CONTRACT Q5, MANUFACTURING_OUTPUT_TOOL_CONTRACT Q2, AI_CLI_MCP Q10)

- Recommended path: Unify onto one T0 projection NOW (verified three call
  sites recompute `plan_native_project_gerber_export` for export,
  manifest, report); make export/manifest/validate/compare/inspect all
  read it, pinned to byte-identical output to the current export path
  first, then retire the other two recompute paths. Keep read-only
  artifact generation CLI-bridged for the first slice (the existing
  `_run_cli_json` bridge) as transitional; the permanent target is daemon
  dispatch once T0 is unified (the daemon already holds the resolved model
  in memory) — do not invest in daemon manufacturing dispatch until T0
  unification lands, and always pin an explicit `model_revision` so bridge
  and daemon project identical bytes. Artifact metadata required fields:
  `project_id`, `model_revision`, `output_job_id`, variant,
  manufacturing_plan, generator_version, per-file `content_hash`,
  validation/equivalence state, command/actor provenance; home: BOTH a
  sidecar `manifest.json` in the output dir (portable, travels to the fab)
  AND a lightweight journaled Artifact record (internal
  staleness tracking) — the sidecar is a projection of the
  record, re-emitted on every generate, never trusted back.
- Production example: Altium generates all OutJob outputs from one
  regeneration pass; KiCad jobsets run one resolution producing all
  artifacts; neither recomputes the layer plan per output type. KiCad runs
  export via the same `kicad-cli` process whether human- or
  jobset-invoked (export is a pure projection). Gerber X2/`.gbrjob` is the
  industry sidecar precedent — a JSON-ish manifest travelling with the
  Gerbers; Altium stamps outputs with source revision/settings.
- Engineering reason: Per 000F, T0 is simultaneously the exporter AND the
  equivalence oracle — validate/compare prove equivalence by re-rendering
  from the SAME projection export used; if export/manifest/report each
  recompute independently, validate cannot prove equivalence, defeating
  the projection==export honesty gate, so unification is a correctness
  prerequisite not an optimization. Read-only export emits nothing through
  `commit()` and produces no journal entry, so daemon vs CLI is purely
  performance/state-locality, not correctness; the correctness work is T0
  unification. Staleness detection (output dir vs model) needs
  `model_revision` + `content_hash` queryable without re-reading every
  file (the journaled record), but the fab only receives files (the
  sidecar) — sidecar-as-projection-of-record keeps a single source of
  truth while satisfying both consumers.
- Residual risk: Consolidating three call sites risks a behavior change
  in any one output — pin the unified T0 byte-identical to the current
  export first (regression goldens), then retire the others. CLI-bridged
  export re-resolves per call, so a daemon-mutated uncommitted in-memory
  state could differ from the CLI's freshly-loaded state — enforce
  explicit-`model_revision` pinning. The sidecar and record can drift if
  the sidecar is hand-edited — treat the journaled record as authoritative
  and always overwrite the sidecar on generate.
- Ratification class: recommended-default-pending-ratification.

### Q: Plated vs non-plated drill files, and OutputJob single vs named-multi-job scope?
(MANUFACTURING_OUTPUT_TOOL_CONTRACT Q6, MANUFACTURING_OUTPUT_TOOL_CONTRACT Q1)

- Recommended path: Separate NC/Excellon files for plated (PTH) and
  non-plated (NPTH) holes (fab-standard, safer default); drop the legacy
  `drill.csv` as a redundant second format (via/hole inventory is already
  queryable via `datum-eda query`). Model the OutputJob as a named
  multi-job object from the start (name field, `--include` scope, `--job`
  reserved) but ship ONE default "all artifacts" job in the first cut —
  the default must be a REAL stored journaled OutputJob object, not a
  synthesized one, so the journal slice (create/undo/redo a job) has
  something to act on.
- Production example: KiCad's drill export defaults to and recommends
  separate PTH/NPTH Excellon; Altium NC Drill and most fabs expect
  plated/non-plated separation because plating is a process-step boundary;
  a combined tool-class-column file is non-standard and often rejected.
  Altium OutJob and KiCad jobsets both support multiple named output
  configs from day one (separate fab/assembly jobs common) and
  default-generate the full set when none is selected.
- Engineering reason: Plated and non-plated holes are manufactured in
  different process steps; a combined Excellon forces the fab to parse a
  non-standard tool-class column to separate them, a frequent
  fabrication-error source. The legacy `drill.csv` duplicates via data
  with no fab consumer — redundant-format surface the lean mandate cuts.
  OutputJob is the only authored manufacturing state, journaled via
  `CreateOutputJob`/`UpdateOutputJob` through `commit()`; single-job-only
  would force a `CreateOutputJob` op-shape change (adding name/identity)
  the moment a second job is needed, breaking journal replay of
  first-slice transactions — reserving name + `--include` now keeps the
  typed op stable.
- Residual risk: `datum-test` may have only plated holes, so the NPTH
  file would be empty — the exporter must emit a valid empty (or
  consistently omitted) NPTH file so the manifest's expected-file set is
  deterministic for the validate oracle. If the default job were
  implicit/synthesized, the journal slice would have nothing to act on —
  the default must be a real stored object.
- Ratification class: recommended-default-pending-ratification.

### Q: What deterministic comparison is required for PDF assembly drawings?
(000F PDF fork)

- Recommended path: Do NOT byte-compare assembly PDFs. The authoritative
  equivalence is at the projection (model) layer: compare the
  `AssemblyProjection` structurally — component inclusion set, designators
  (via `ComponentInstance`), side, orientation/rotation, fiducials, notes,
  variant population. The PDF is then asserted a faithful RENDER of that
  already-equivalent projection via a normalized check: rasterize to fixed
  DPI and do a tolerance-bounded pixel/SSIM comparison (the project's
  existing screenshot-golden discipline), with PDF-internal metadata
  (creation timestamp, producer string, object ordering) normalized out
  before any comparison. Genuine visual differences beyond tolerance
  classify as `ExportGeneration`; pure metadata/format diffs are
  normalized away.
- Production example: No EDA tool offers byte-deterministic
  assembly-PDF equality — Altium/KiCad assembly PDFs embed
  nondeterministic timestamps/producer metadata, so teams compare the
  underlying placement data, not PDF bytes. KiCad CI and many hardware CI
  flows visually-diff plotted output (image regression) rather than
  diffing PDF bytes. Datum already mandates image-based regression for
  rendering work (the screenshot-goldens memory), so the assembly-PDF
  check reuses that harness.
- Engineering reason: A PDF is a presentation render of the
  `AssemblyProjection`, carrying nondeterministic metadata and tolerated
  layout/font differences — byte equality would produce constant false
  `ExportGeneration` findings. The honest invariant is two-layered:
  structural equivalence at the projection (deterministic, revision-keyed)
  and bounded visual fidelity of the render. This matches
  single-generation-core: the projection is the shared core; the PDF is a
  downstream rendering whose correctness is a render-fidelity question,
  not a model-equivalence question.
- Residual risk: A pixel/SSIM tolerance can mask a subtle defect (a
  shifted designator) or false-positive on platform font substitution —
  keep the authoritative assertion on the structured
  `AssemblyProjection` (where a moved designator is a hard data diff) and
  treat rasterized comparison as a secondary render-fidelity guard with a
  tuned documented tolerance, normalizing fonts/DPI before compare.
- Ratification class: recommended-default-pending-ratification.

---

## 13. Panelization scope, primitives, and isolation

### Q: Should panelization ship before full board editing, with what first scope, and how is it isolated; should it be deferred entirely to M8?
(000B Q1, 000F Q1, 000F Q5, MANUFACTURING_OUTPUT_TOOL_CONTRACT Q7)

- Recommended path: Reconciliation of a tension across docs: 000B/000F
  treat panelization as buildable-now-on-imported-boards (gated only on a
  resolvable immutable board reference + the PG-PANELIZATION-ISOLATION
  gate), while the manufacturing contract recommends deferring the panel
  OBJECT to M8. Reconcile by separating the gate from the feature: DEFER
  the panel object, step-and-repeat, rails/tabs/V-scores to M8
  (`datum-test` is a single board, so there is no first-slice proof for
  the full feature), but LAND NOW the panelization-isolation drift gate as
  a test invariant (board shard bytes unchanged after any future panel op)
  and reserve `--panel` as a hard-erroring context parameter on the single
  export verb. WHEN the panel object is built (M8), first scope =
  repeated-board arrays only (N instances of one Board via grid
  transform); routed tabs + mouse bites first (V-scores deferred); model
  `PanelProjection.instances` as a list of (board_id, board_revision,
  transform) from day one even when populated with one repeated board id;
  pin `board_revision` so a superseded board surfaces as stale, never
  silently re-resolving to HEAD.
- Production example: KiKit `panelize grid` (repeated array of one board)
  is the canonical first/most-used mode shown in JLCPCB/PCBWay
  quickstarts; multi-board is a more advanced path. KiKit's default
  depanelization is routed tabs + mouse bites (`--tabs`/`--mousebites`),
  V-cuts (`--vcuts`) a separate optional mode. Allegro Panel Editor and
  Altium embedded-board-array both treat the panel as a distinct artifact
  that never edits the source board — the isolation property is
  load-bearing. KiCad panelization (KiKit) is an external tool entirely
  separate from the board.
- Engineering reason: The audit mandates panel features never mutate
  source board geometry; the cheapest way to guarantee the invariant
  survives into M8 is to assert it now as a drift gate and make `--panel`
  a context on the existing export verb so panel never becomes a parallel
  pipeline. Panel state is pure derived state holding only references +
  transforms + panel-only geometry, so by construction it owns no board
  geometry to corrupt — safety comes from reference-only construction and
  the no-board-`OperationBatch` invariant, not from board-editing
  maturity. Building the full panel object in M7 expands scope into M8
  with no first-slice proof. A repeated array fully exercises
  `BoardInstance` + transform + panel-only geometry + array-aware artifact
  identity; multi-board multiplies resolver board-resolution paths without
  a new isolation class. Mouse-bite drills are the sharpest isolation test
  (drill hits that MUST be excluded from board-only Excellon, extending
  the existing drill comparison).
- Residual risk: Reserving `--panel` without an implementation means the
  flag must hard-error (not silently no-op) when given, or users think
  panel export ran. A panel referencing a superseded `board_revision` must
  surface as stale, not silently re-resolve to HEAD. Designing
  `PanelProjection` around a single board could bake in single-board
  assumptions — model instances as a list from day one. V-score clearance
  interacts with edge component keepouts; deferring it leaves
  edge-clearance-against-scoring unproven — reserve a V-score clearance
  field in `PanelRule` so the later add is data, not schema.
- Ratification class: recommended-default-pending-ratification.

### Q: Should Datum ship fab/CM panel presets or generic rules only, and should board/panel fiducials share one primitive?
(000B Q3, 000F Q6, 000F fiducial fork)

- Recommended path: Generic `PanelRule` values only (board spacing, rail
  width, tab spacing, mouse-bite drill size/count/spacing, V-score
  clearance, fiducial size/clearance, tooling-hole size/clearance) as
  plain model objects with conservative defaults; do NOT ship named
  fab/CM presets. Make a `PanelRule` set a serializable importable object
  with a name + provenance/`import_key`-compatible identity from the start
  so a fab/CM preset is later just authored data routed through the Import
  Map, never special-cased code. Fiducials: ONE `Fiducial` primitive
  (shape, size, clearance/keepout, optical-class) distinguished by an
  owning-context field (Board vs Panel vs BoardInstance), NOT two separate
  types; board<->panel conversion is an explicit ownership/promotion
  operation.
- Production example: KiKit core ships generic parametric panel rules and
  treats fab profiles as user-supplied JSON/CLI files — no built-in
  JLCPCB preset in core; Altium leaves CM-specific panel constraints to
  user-authored design rules, not a baked CM catalog. KiCad represents a
  fiducial as an ordinary footprint with attribute "fiducial" — one object
  kind, context = where it lives; KiKit adds panel/global fiducials as the
  same footprint type on panel material. Altium uses one Fiducial type,
  distinguishing board vs panel by which document/embedded-board owns it.
- Engineering reason: Bundling named presets makes Datum implicitly assert
  it knows each fab's current process limits — a maintenance/liability
  surface that drifts the moment a fab changes rules, which the docs
  explicitly disclaim; generic rules as authored data give the same
  capability with zero embedded-knowledge risk (a preset becomes a
  checked-in validated `PanelRule` object, updatable without a code
  release). A fiducial is the same fabrication-and-assembly optical mark
  physically; the production difference is purely ownership, so one
  primitive + owner field keeps geometry/aperture generation
  single-sourced (one renderer feeds board and panel projections) and
  makes the board<->panel conversion a pure ownership/promotion op rather
  than type-translation; two types would duplicate the renderer and risk
  divergent aperture output.
- Residual risk: Without presets, first-time users hand-enter rule
  values, raising the odds of an out-of-spec panel — ship one
  clearly-labeled generic-conservative default set and surface `PanelRule`
  violations as production-check findings, never silent acceptance. An
  owner field is easy to set wrong (a panel fiducial accidentally
  board-owned) — the panelization-isolation gate must assert no `Fiducial`
  with panel-owner appears in a board-only artifact, and promotion must be
  an explicit accepted operation.
- Ratification class: research-conclusive.

---

## 14. Library pools, approval state, and footprint generation

### Q: Which library pool types ship first, and should project-local be the first library model?
(008 pool-types fork, 002 Q3, LIBRARY_AUTHORING_TOOL_CONTRACT Q2)

- Recommended path: Ship exactly two pool types first: project-local
  (writable, journaled into the project's own shards) and imported
  (read-as-provenance, populated via the Import Map). Bundled-Datum is a
  pre-populated user/system pool that falls out for free; user-local,
  organization, and vendor-derived are deferred layers, not first-cut
  substrate; imported/vendor content is a provenance source that LANDS
  INTO project-local review-gated, not a separate pool type. Model the
  pool type as an ORDERED `pool_ref` enum starting
  `{ProjectLocal, Imported}` (so precedence is reserved when
  user-local/bundled land) and have `LibraryBinding` pin (`pool_ref`,
  `object_id`, `object_revision`) from the first cut.
- Production example: KiCad's library tables layer exactly this way: a
  project-specific table plus a global (user/system) table with
  per-project overrides shadowing global — KiCad shipped global-first and
  the well-known missing-footprint-on-another-machine pain drove the
  project-local `fp-lib-table` and v9 embedded files, converging on
  project-local-first. Altium recommends project-released/cached
  components so the project is self-contained, with the Vault/Concord Pro
  as a later org overlay. Horizon is pool-first but
  immutable-by-reference, which Datum defers because it needs
  project-local authoring before pool governance.
- Engineering reason: Verified the pool layer exists (`pool/mod.rs`:
  `Pool`, `PoolIndex`, SQLite-backed index) and `ProjectResolver`
  resolves a pools array relative to `project_root` (`NativePoolRefShape`
  in `substrate/mod.rs`), so a project-embedded pool needs no new
  authority concept and no cross-machine global path resolution, keeping
  `model_revision` stable and reproducible (library bytes inside the
  project hash set). Project-local is required because the first proof
  authors a native Part and binds it via `ComponentInstance` into a
  writable journaled shard; imported is required because `datum-test`
  re-association tests the `import_key` path. The other pool types add
  precedence/sharing policy but no new mechanism.
- Residual risk: Precedence (which pool wins when an `object_id` resolves
  in two) is undefined with only two non-overlapping layers but resurfaces
  the moment user-local/bundled land — reserve `pool_ref` as an ordered
  enum now to avoid a migration. Reuse across projects requires copy-in
  until a global pool lands, risking divergent copies — mitigate with
  `import_key`/provenance on project-local parts so a later global-pool
  reconciliation can dedupe. Org/shared pools introduce identity-sync
  questions easier if `LibraryBinding` already pins `object_revision` and
  `pool_ref` from the first cut.
- Ratification class: research-conclusive.

### Q: What approval states gate part placement, and is the gate a hard block or a warning by default?
(008 approval-state fork, LIBRARY_AUTHORING_TOOL_CONTRACT Q3)

- Recommended path: Ship a minimal `approval_state` enum
  `{Draft/NeedsReview, Approved, Deprecated, Unknown}` ORTHOGONAL to the
  existing supply-chain `Lifecycle` axis, plus the
  imported/project-local/generated provenance flags. Do NOT block
  placement by approval state by default — placing a non-Approved part
  (Imported/NeedsReview/Unknown/Generated) is allowed but produces a
  non-blocking "part not approved" check finding; Deprecated/Obsolete
  escalates to a stronger warning. Make "block placement of non-Approved"
  an opt-in `CheckProfile` severity (library-release / manufacturing-
  sign-off profiles), not a default hard gate. Imported parts default to
  `approval_state=Unknown`, never silently Approved. Key the finding per
  `ComponentInstance` binding so it is dismissible/waivable per the shared
  waiver path, not re-raised N times.
- Production example: Altium's managed-component Lifecycle states
  (New/In Work/Released/Deprecated/Obsolete) gate release in managed
  environments but do not stop placing an unreleased component in an
  unmanaged project — a configurable rule, not absolute; the state shows
  as a badge in the placement panel with a deprecated/obsolete warning.
  KiCad has no approval-state concept (any symbol places freely). Datum
  warn-by-default with profile-driven hard-block sits between them.
- Engineering reason: Verified the `Part` struct has
  `Lifecycle{Active,Nrnd,Eol,Obsolete,Unknown}` (`pool/mod.rs:135,140`)
  which is a SUPPLY-CHAIN axis distinct from an APPROVAL/review axis —
  conflating them loses information (an Active part can be unreviewed; an
  Approved part can be NRND). 008's user-visible behavior lists
  approved/deprecated/unknown/imported/project-local as display states, so
  a small orthogonal `approval_state` matches stated intent. A hard block
  by default would make iterative design impossible and contradict
  usefulness-before-authoring; gate strictness as a `CheckProfile`
  parameter reuses the existing severity/profile machinery
  (`ErcConfig.severity_overrides` exists) rather than minting a
  placement-specific policy, keeping it revision-keyed.
- Residual risk: Two near-synonym axes (`Lifecycle` vs `approval_state`)
  can confuse users/importers — UI and import mapping must label them
  distinctly and imported parts default to Unknown. The same part placed
  many times while non-Approved multiplies the warning — key per
  `ComponentInstance` binding so it is waivable once.
- Ratification class: recommended-default-pending-ratification.

### Q: IPC-7351B vs IPC-7352 default footprint naming, and which first package families?
(008 IPC-naming fork, 008 package-family fork, LIBRARY_AUTHORING_TOOL_CONTRACT Q4)

- Recommended path: Default to IPC-7351B for land-pattern
  geometry/naming; reserve IPC-7352 as a deferred mode. Store the basis as
  a STRUCTURED `{family:'IPC-7351', revision:'B', density:...}` field on
  the Footprint `standards_basis[]` (an enum, not a bool, not baked into
  the name) so IPC-7351C/7352 is additive. First package families:
  two-terminal chip components (R/C/L: 0402/0603/0805/1206 metric-coded)
  and gull-wing SMD ICs (SOIC and SOT-23) — chips validate the
  padstack/mask/paste-policy model with the simplest geometry, SOIC/SOT-23
  force the pitched multi-pad land-pattern math (toe/heel/side fillet,
  courtyard) that QFP/QFN/BGA later reuse. Defer thermal-pad (QFN),
  polarized-pad, and BGA-array generation, designing the padstack
  paste/mask policy general enough now (explicit aperture-source enum) to
  absorb them.
- Production example: KiCad's official libs and the KLC name SMD
  footprints per IPC-7351 (`R_0402_1005Metric`,
  `SOIC-8_3.9x4.9mm_P1.27mm`) and are deepest in
  `Resistor_SMD`/`Capacitor_SMD` and `Package_SO` (SOIC)/
  `Package_TO_SOT_SMD` (SOT-23). Altium's IPC Compliant Footprint Wizard
  is IPC-7351-based with density levels (A/B/C suffixes) and groups by
  family with Chip/SOIC/SOT first. Neither uses 7352 as the generation
  naming basis.
- Engineering reason: IPC-7351B defines the actual land-pattern math
  (toe/heel/side fillet goals, courtyard excess, density levels) that a
  generator must implement and whose names encode the geometry; choosing
  it as default aligns output names with the standard it computes from.
  IPC-7352 is a footprint-EXPRESSION/exchange standard, not the
  calculation basis. The engine has no footprint generator yet (pool
  `Package` has courtyard/silkscreen/pads but no IPC basis field), so
  declaring 7351B in `standards_basis[]` from the first generated
  footprint sets the honest declared-basis the standards-compliance spine
  requires, avoiding retroactive relabeling.
- Residual risk: IPC-7351C exists and tightens values; locking the
  literal string "7351B" risks a future migration — store basis as the
  structured `{family, revision, density}` so revision is a field, not
  baked into the name. Thermal-pad/BGA introduce paste-stencil split/array
  and via-in-pad concerns not exercised by R/C/SOIC — the padstack
  paste/mask policy model must be designed general (explicit
  aperture-source enum) now to absorb them.
- Ratification class: research-conclusive.

### Q: What part qualification/environmental metadata must be visible in v1 BOM workflows?
(008 qualification-metadata fork)

- Recommended path: v1 BOM surfaces per placed line: MPN + manufacturer +
  value/description (already in `Part`), Lifecycle status,
  `approval_state`, a single RoHS/REACH `compliance_metadata` field
  `{Compliant/NonCompliant/Unknown + optional source}`, and
  provenance/review state. Defer deeper qualification (AEC-Q, MSL, full
  environmental declarations, export-control) to fields that are STORED
  but not required-visible in v1 BOM. Render Unknown honestly rather than
  implying compliance.
- Production example: Altium's ActiveBOM columns surface MPN,
  Manufacturer, Lifecycle (from managed/Octopart data), and
  ComplianceRoHS/REACH as standard sourceable columns, with supply-chain
  availability as an enrichment tier. KiCad's BOM carries
  MPN/Manufacturer/datasheet by convention and the HTML BOM shows
  DNP/value, with no built-in RoHS field — the gap Datum's first-class
  `compliance_metadata` closes.
- Engineering reason: Verified `Part` already carries `mpn`,
  `manufacturer`, `value`, `datasheet`, `lifecycle`, `orderable_mpns`,
  `parametric` (`pool/mod.rs:122-138`), so most of the visible set exists;
  only `approval_state`, compliance flag, and provenance/review state are
  net-new and all three are already mandated as Part/provenance fields by
  008. Surfacing exactly the fields the model already commits keeps BOM a
  pure projection over resolved `DesignModel` objects (no BOM-private
  data), consistent with artifact-as-projection. Deferring AEC-Q/MSL
  respects the explicit non-goal and avoids storing fields with no v1
  producer.
- Residual risk: Compliance/qualification often come from external
  supply-chain services v1 explicitly defers, so v1 fields will frequently
  be Unknown until enrichment exists — the BOM must render Unknown
  honestly rather than implying compliance, to stay within the
  no-overclaim standards posture.
- Ratification class: recommended-default-pending-ratification.

### Q: When is ComponentInstance minted, and does its absence block schematic authoring?
(SCHEMATIC_AUTHORING_TOOL_CONTRACT Q2)

- Recommended path: Mint the `ComponentInstance` at place-symbol (every
  `PlacedSymbol` gets an instance id immediately); BIND the library
  Part/footprint at part assignment. A symbol can exist as an instance
  with no library binding (annotated-but-unassigned). Absence of a library
  binding does NOT block schematic authoring; absence of a
  `ComponentInstance` never happens because it is minted at creation.
  Multi-unit parts (gates A/B/C of one package) map multiple
  `PlacedSymbol`s to ONE `ComponentInstance` — detect same-physical-part
  placement (reference + unit) and reuse, not mint a second.
- Production example: Altium creates a Component with a Unique ID (the
  cross-document join) the moment you place a schematic symbol, before any
  footprint/DbLink resolves; the association fills later. KiCad assigns a
  symbol its UUID at placement and the footprint field later via Assign
  Footprints. Both mint instance identity at placement, bind library data
  later.
- Engineering reason: `ComponentInstance` is the electrical-to-physical
  join the audit mandates replace refdes joins. Minting only at part
  assignment would leave an unassigned-but-placed symbol with no stable
  identity, forcing forward-annotation/board placement to fall back to
  refdes — the failure the spine forbids. Verified: `substrate/mod.rs`
  models `ComponentInstance` as an independent object keyed by Uuid,
  decoupled from part data, supporting mint-at-placement.
- Residual risk: Multi-unit parts must map multiple `PlacedSymbol`s to one
  `ComponentInstance`; mint-at-placement must detect same-physical-part
  placement via reference + unit and reuse the instance, not mint a second
  one.
- Ratification class: research-conclusive.

---

## 15. Check profiles, rule scoping, severities, and rule storage

### Q: Which CheckProfiles ship first, is profile selection a parameter or a stored object, and must the hard-coded run_drc rule set be replaced by a profile parameter first?
(009 Q1, RULES_CHECKS Q1, RULES_CHECKS Q7)

- Recommended path: Ship two `CheckProfile`s first: full-project DRC/ERC
  sign-off (the complete current 7-rule set, gate-strict) and edit-time
  (fast, fewer rules, warn-lenient — a re-parameterized subset of the full
  engine, not a separate engine). Defer import-audit, library-release,
  manufacturing-sign-off. A profile is BOTH a stored journaled project
  object (named, default-selectable; determinism requires the active rule
  set to be a versioned `CheckRun` input) AND selectable by id via
  `--profile` — the stored object is authority, the parameter selects it.
  PREREQUISITE: replace the hard-coded 7-`RuleType` array in
  `dispatch.rs:455-463` with a `CheckProfile` parameter BEFORE
  `waive_finding`/`propose_repair` and the persisted `CheckRun` land,
  keeping the default profile == the current 7 rules so the 0% FP/FN
  goldens do not move.
- Production example: KiCad distinguishes Online DRC (edit-time subset)
  from Batch DRC (full set) as modes over one rule store;
  `kicad-cli pcb drc` runs the project's configured rule/severity set, not
  a binary-baked list. OrCAD/Allegro Constraint Manager stores named
  reusable constraint sets selected by name. Altium batch DRC runs the
  enabled rules from the rules store. No production tool bakes its DRC list
  into the dispatch binary.
- Engineering reason: Verified `dispatch.rs:455-463` passes a fixed
  `RuleType` array into `engine.run_drc`. A profile must be a stored
  journaled object because determinism (same `model_revision` +
  rule-versions => same fingerprints) requires the active rule set as a
  versioned input; an ad-hoc CLI parameter would make finding fingerprints
  depend on un-versioned flags. Two profiles cover the online/batch
  distinction with no new `rule_types`. Every downstream feature (named
  profiles, severity-on-rule, persisted+fingerprinted `CheckRun`) requires
  the active set to be selectable/versionable, so replacing the hard-coded
  array must precede them.
- Residual risk: Edit-time live feedback is deferred (early users get
  batch-only) — mitigated because the full profile is fast over single
  footprints and the rule set is identical, so promoting to live is a UI
  change. Changing the dispatch signature touches the daemon contract and
  CLI/MCP bridge simultaneously — keep the default profile == the current
  7 rules to isolate the refactor from a behavior change.
- Ratification class: recommended-default-pending-ratification.

### Q: What severities for standards-aware process-geometry findings, and should unknown-basis imported footprints fail/warn/group as audit?
(009 Q2, 009 Q3, RULES_CHECKS Q4, 008 unknown-basis fork, LIBRARY Q5, 011 imported-stricter fork)

- Recommended path: Map severity by basis confidence, not category, via a
  single centralized `CheckProfile` knob shared across library import
  (008), checks (009), and import (011) so UnknownBasis means the same
  thing everywhere: declared-basis violation (pad/drill/annular/clearance
  below a selected hard rule) => Error; inferred-basis or
  import-preserved-but-suspect process geometry => Warning; unknown-basis
  (no selectable basis) => a new Info/Audit tier (extend the current
  two-level `DrcSeverity{Error,Warning}`). Unknown-basis imported
  footprints import successfully, preserve geometry, and surface ONE
  grouped audit-tier finding per footprint (shared fingerprint over
  affected object ids), never a per-pad Error cascade, promotable to fail
  only under a library-release/manufacturing-sign-off profile. Imported
  geometry defaults to unknown-basis audit findings that native authored
  geometry (where the author chose a basis) does not generate; this never
  changes the severity of hard rule violations (clearance/annular Errors
  apply equally to imported and native copper). Severity lives on the
  rule/profile (a versioned `CheckRun` input), not per-run.
- Production example: KiCad DRC severities are per-rule-class configurable
  (Error/Warning/Ignore) and default several manufacturing/courtyard
  checks to Warning because they are advisory; KiCad import preserves
  footprint geometry and does not flag "no IPC basis" at all (unknown
  basis is silence, not a violation). Altium lets you set severity per
  rule and treats "no rule defined" as no violation rather than an error;
  Altium import flags but does not reject unverified geometry, leaving
  imported items in a project library pending user binding. Both lean
  lenient-by-default on imported data.
- Engineering reason: Verified the engine implements
  `RuleType::ProcessAperture` with `pad_mask_expansion_below_rule` and
  `pad_paste_reduction_below_rule` (`drc/checks/mod.rs`) and `DrcSeverity`
  per `DrcViolation` (`drc/mod.rs:13`), so adding an Info/Audit tier is a
  one-variant extension. 010's claim posture forbids severity encoding
  certification and 009 requires distinguishing declared/inferred/unknown
  basis — binding severity to basis confidence keeps the label honest (an
  Error means "violates a basis the project selected," never "we guessed
  and you missed it"). The import path already hard-errors only on
  structurally unparseable encodings (`import/kicad/skeleton.rs`);
  semantic basis-unknown is a finding, not a parse failure. Grouping per
  footprint prevents a 200-pad board emitting thousands of identical
  findings that would drown the determinism-keyed set. Native geometry has
  an authored/explicit-none basis; imported geometry's basis is
  inferred/unknown by construction.
- Residual risk: Three tiers plus a waived flag increases report-UI
  surface and the chance users ignore Audit — keep Audit findings
  visible/groupable and let a shop promote them to Error per profile. The
  UnknownBasis warn/fail policy must be ONE `CheckProfile` field across
  008/009/011 or the domains diverge. A green-except-audit board can be
  mistaken for "passed" — 010 claim discipline requires reports list
  unknown-basis data and the summary surface audit counts separately.
  Courtyard findings on imported footprints lacking courtyard data must
  degrade to an unknown-basis warning, not a false pass.
- Ratification class: recommended-default-pending-ratification.

### Q: Which IPC/process categories are mandatory for the first DRC slice, and how strict for imported UnknownBasis?
(009 Q5, RULES_CHECKS Q5)

- Recommended path: First slice mandatory: pad/mask/paste process-aperture
  (exists as `RuleType::ProcessAperture`), copper-pad clearance (exists as
  `ClearanceCopper`), drill/annular (exists as `ViaHole` +
  `ViaAnnularRing`), and courtyard (component-to-component, new). Defer
  density-level and IPC-naming checks (they depend on the not-yet-built
  IPC footprint basis/library provenance). Process-geometry findings
  default Warning at edit-time, Error under manufacturing-sign-off;
  unknown-basis imported footprints group as audit-grade findings per the
  centralized knob above.
- Production example: Altium computes solder-mask and paste-mask
  expansions from rules (first-class DRC-able constraints) and IPC-7351
  land-pattern generation derives pad/mask/paste from density level; KiCad
  stores per-pad `solder_mask_margin`/`solder_paste_margin` and DRCs them,
  and its default DRC covers clearance, hole size, annular ring, courtyard
  overlap, silk clearance. Density-level/IPC-naming conformance is not
  stock DRC in either tool (it lives in the footprint wizard/library
  audit).
- Engineering reason: Verified the engine runs Connectivity,
  ClearanceCopper, TrackWidth, ViaHole, ViaAnnularRing, SilkClearance,
  ProcessAperture (`dispatch.rs:455-463`) with a 0% FP/FN gate; building
  the first profile from these plus courtyard reuses validated checks.
  ProcessAperture is the category with existing kernel coverage and
  existing import-fidelity stakes — minimal new code, maximal exercise of
  the basis/provenance/proposal (`SetPadProcessAperture`) loop.
  Density-level/IPC-naming need the IPC `standards_basis` and the separate
  Footprint object that do not exist yet, so mandating them now gates the
  DRC slice on the library slice.
- Residual risk: Courtyard checking needs a courtyard polygon on every
  package; imported footprints without courtyard data can only be
  partially checked, so courtyard findings must degrade to an
  unknown-basis warning where geometry is absent, not a false pass.
  Courtyard/spacing is high-value and users expect it early — additive (a
  new `RuleType` variant + check fn on the same `DrcViolation` pipeline).
- Ratification class: research-conclusive.

### Q: Should set_rule write to a dedicated rules shard, how much rule editing must be GUI vs CLI, and is Altium-style rule-query scoping in scope?
(RULES_CHECKS Q6, 009 Q6, PCB_LAYOUT_TOOL_CONTRACT Q10)

- Recommended path: `set_rule` writes to a dedicated rules source shard (a
  resolver persistence partition), NOT in-place `board.rules` — repoint
  the existing typed `SetDesignRule` op from `board.rules` to the rules
  shard. Rule editing is CLI/MCP-first for the entire first slice
  (`datum-eda rules set`); GUI rule editing is deferred, and the first GUI
  affordance is read+inspect+waive, not full authoring. Rule scoping keeps
  per-net and per-net-class scope only (what `rules/ast.rs` supports);
  defer combinator/regex/IsDiffpair scopes that `ast.rs:18-32`
  parses-but-errors-on-eval until eval support exists.
- Production example: KiCad stores custom rules in a dedicated
  `.kicad_dru` file/section processed by the resolver, separate from board
  geometry, and as of v7+ ships text-based custom rules with no graphical
  builder (after shipping net-class rules for years). Allegro stores
  constraints in the Constraint Manager database separate from the canvas.
  Altium's full rule-query DSL (`InNetClass`, `IsPad && OnLayer`) is its
  power feature but also its primary complexity. Both keep rules as their
  own partition the model assembles.
- Engineering reason: The audit makes source shards persistence
  partitions assembled by `ProjectResolver` and forbids any file/section
  becoming a second authority; `board.rules` embedded in the board shard
  fuses rule authority into board authority, while a dedicated rules shard
  lets `compute_model_revision` (sha256 over source_shards) account for
  rule changes independently. `CLAUDE.md` establishes engine-first with
  CLI/MCP primary and GUI a consumer; every mutation (including rule
  edits) reduces to typed Operations on one `commit()` the CLI already
  drives, so gating checks on a GUI rule editor would invert the
  architecture. Verified `ast.rs` PARSES combinator/regex/IsDiffpair
  scopes but ERRORS on eval — shipping the query surface before eval works
  would expose a contract the engine cannot honor, breaking determinism;
  net/net-class scope is fully evaluable today.
- Residual risk: Migrating existing `board.rules` into the rules shard is
  a one-time format migration; the converter that turns a KiCad file into
  native data must land embedded rules in the new rules shard without loss
  (import must not fabricate or silently heal). This is a converter-
  correctness concern, not a gate on native rule-shard maturity — native
  readiness is judged by native author/edit/check/output capability, per
  docs/DATUM_PRODUCT_MECHANICS.md "Interop Boundary And Import Posture".
  Non-CLI users cannot author rules until the GUI
  lands (acceptable for a headless-first tool whose early audience is
  CLI/MCP/AI; read-only GUI inspection can land first). Diff-pair/
  length-match rules need scope expressiveness the v1 floor lacks — those
  `rule_types` are independently deferred, so the scoping deferral blocks
  no first-shipping `rule_type`.
- Ratification class: research-conclusive.

---

## 16. Standards-compliance claim discipline, posture, audit substrate

### Q: What exact user-facing language is allowed for compliant/validated/basis-known/certified?
(010 claim-language fork)

- Recommended path: Lock the lexicon as a `claim_language_policy` field on
  `StandardsRegistryEntry` plus a template lint:
  "certified"/"regulatory compliant"/"IPC certified" are forbidden in all
  output without future owner+legal approval; "validated against selected
  Datum rule basis" is the only allowed validation phrasing; "compliant"
  is permitted ONLY as "compliant with selected Datum rule basis[, with N
  documented deviations]" applied to a modeled object, never to a
  product/org/process; "basis known" renders as "basis declared" or
  "basis inferred (unconfirmed)", absence as "basis unknown".
- Production example: No EDA tool over-claims: KiCad's DRC report says "X
  violations"/"DRC complete", never "compliant". Altium reports describe
  rule pass/fail and never assert IPC certification of the user's product;
  Altium's IPC footprint wizard says it generates "IPC-7351 compliant"
  land patterns referring to the geometry vs the standard's formula, not
  certifying the board — precedent for restricting "compliant" to a
  modeled object vs a named basis.
- Engineering reason: 010 already states the allowed/forbidden lists and
  `StandardsRegistryEntry` carries a `claim_language_policy` slot; making
  the lexicon a data field plus a template lint turns claim discipline
  into a checkable gate rather than reviewer vigilance. The doc's Claim
  Posture nearly fully specifies the allowed phrases, so the only open
  part (exact wording) is settled by the doc itself.
- Residual risk: Free-text rationale fields and AI-generated summaries can
  emit forbidden words — mitigated by 010's guardrail that AI may not
  claim certification, plus a report-generation lint flagging forbidden
  tokens before export.
- Ratification class: research-conclusive.

### Q: Which standards registry subset is visible first, is project standards posture required at creation, and which compliance fields are mandatory?
(010 registry-subset fork, 010 posture-at-creation fork, 010 mandatory-fields fork)

- Recommended path: Visible first registry subset = the set bound to real
  engine behavior: Gerber RS-274X and Excellon as SupportImplemented;
  IPC-7351 (land patterns) and IPC-7525 (stencil/paste) as
  Planned/partially-implemented; IPC-2221 (generic clearance) as the
  selectable clearance basis; IPC-2581/ODB++/STEP/IDF as
  Planned/DeferredWithPrerequisite (naming the unblock condition).
  Everything else researched stays ReferenceOnly/Planned/Deferred so
  silence is never the state. Project standards posture is OPTIONAL at
  creation, lazily required only when a check/report needs a basis: a new
  project gets a default `ProjectCompliance` (all-unknown) and a default
  `DataEgressPolicy`; the first basis-needing run without a selection
  emits an Info/Audit `ComplianceFinding`, never blocking creation or the
  check. v1 MANDATORY compliance fields: `data_egress_policy_ref` (gates
  egress) and `audit_log_completeness_state` (report honesty); everything
  else on `ProjectCompliance` and all `PartQualification` fields (AEC-Q,
  RoHS/REACH, lifecycle) are advisory, rendering "not declared" when
  absent.
- Production example: Altium surfaces a finite concrete standards posture
  tied to real generators (IPC-7351 wizard, Gerber/ODB++/IPC-2581
  outputs), not a marketing catalog; KiCad creates a board with default
  netclass/clearance and never prompts for IPC class at creation, setting
  clearances in Board Setup only on demand. Altium's
  ComplianceCheck/ActiveBOM treats RoHS/lifecycle/AEC as optional part
  attributes driving warnings only when a rule demands them, never
  mandatory to save; KiCad has no mandatory compliance metadata at all.
- Engineering reason: 010 requires every researched standard have a
  disposition (no silence) but the VISIBLE first slice should be the
  subset bound to real code: Gerber/Excellon implemented in `export/`,
  ProcessAperture/clearance in `drc/`, IPC-2581/ODB++/STEP
  spec-stub-only per `CLAUDE.md`. Forcing posture at creation contradicts
  usefulness-before-authoring and blocks import-first workflows where the
  imported basis is exactly what's unknown; making absence a finding
  rather than a gate keeps `CheckRun` determinism intact. Mandating
  descriptive metadata creates false weight (a filled `ipc_class_basis`
  does not make a board IPC-class compliant); the only field with an
  enforcement function is `data_egress_policy_ref` (tools consult it
  before transmitting), and `audit_log_completeness_state` is mandatory
  only because reports must disclose audit completeness.
- Residual risk: A large Planned/Deferred list may read as a feature
  promise — 010's rule that Planned must not appear as implemented and
  DeferredWithPrerequisite must name its unblock keeps the subset reading
  as posture. Projects may run long with no declared basis producing many
  audit findings — group "no basis" to one per project/scope and surface
  posture prominently. Optional metadata stays blank and reports look
  sparse for regulated users — make each blank a one-click declaration and
  have `ComplianceFinding` surface missing fields a selected project
  profile expects.
- Ratification class: recommended-default-pending-ratification.

### Q: What data-egress policy defaults apply to new projects?
(010 egress fork)

- Recommended path: Default to deny-with-prompt: posture = `LocalOnly`,
  `prompt_required = true`, `audit_required = true`,
  `allowed_destinations = []` for any tool that can transmit
  project/part/model/compliance data (supply-chain lookups, external AI
  calls, model-metadata fetches). Local engine/CLI/MCP operations that
  never leave the machine are exempt. The first time an egress-capable
  tool runs it consults `DataEgressPolicy` and prompts; the grant is
  recorded as an `AuditRecord`. Ship supply-chain/external-lookup tools
  disabled-by-policy, not absent. Use per-destination allow-listing (grant
  Digi-Key without granting arbitrary AI egress).
- Production example: KiCad ships with no telemetry and no mandatory
  network calls; its optional plugin/content manager is explicit and
  user-initiated — egress is opt-in. Altium's external services
  (manufacturer part search, ActiveBOM supplier data) require an explicit
  signed-in account before any part data leaves the design.
- Engineering reason: 010 mandates `DataEgressPolicy` be consulted by
  MCP/CLI/AI/supply-chain tools and that controlled data not leave without
  policy allowance; the safe default for a tool that may handle
  export-controlled or proprietary designs is deny-with-prompt, because a
  permissive default cannot be retroactively undone for data already sent.
  `CLAUDE.md`'s "no external network services mandatory" aligns. The
  policy is itself source state (an `ObjectId` object), so the default is
  a creation-time constructed object, not a hidden constant.
- Residual risk: Prompt fatigue may push users to a blanket allow —
  mitigate with per-destination allow-listing and `audit_required` logging
  each grant so egress remains reviewable even after a broad allow.
- Ratification class: recommended-default-pending-ratification.

### Q: How much audit/sign-off infrastructure is needed before regulated-process overlays can be shown?
(010 signoff fork)

- Recommended path: Prerequisite before any regulated overlay (Part-11/ISO
  sign-off) appears: (1) `AuditRecord` derived from every `commit()` (the
  journal exists; `AuditRecord` indexing over `TransactionRecord`s is the
  gating build), (2) a `signature_state` primitive with an
  identity/authentication binding, (3) tamper-evidence over the journal
  tip. Until all three exist the UI may show read-only audit HISTORY
  (journal-derived event list) but MUST NOT show sign-off/approval
  affordances or any "compliant"/"approved" state. Do not show regulated
  overlays in v1.
- Production example: Altium's design-data-management/approval and A365
  lifecycle states (released/approved) are backed by a server with
  authenticated users and immutable history — Altium does not expose
  release/approval semantics without the managed identity backend.
  Sign-off UI follows an authenticated immutable-history substrate, never
  precedes it.
- Engineering reason: 010's Non-Goals forbid implementing Part-11
  signatures before audit/signature primitives exist, and `AuditRecord` is
  explicitly derived-from-commit (actor/timestamp/`signature_state` from
  the committing transaction, never captured independently). Showing a
  sign-off affordance without an authenticated `signature_state` would be
  the unbacked certification claim the whole doc forbids. The journal
  (`substrate/journal.rs`) already exists as the durable base, so the
  gating work is `AuditRecord` projection + identity, not a new store.
- Residual risk: Regulated prospects may want sign-off early and read its
  absence as immaturity — mitigate by shipping the read-only audit-history
  view first (cheap, journal-derived) so the evidence trail is visible
  before signature/approval semantics, the honest "substrate, not
  certification" posture 010 demands.
- Ratification class: research-conclusive.

---

## 17. Import lossiness, provenance, and export-format criticality

### Q: What source-format lossiness is acceptable before import should fail instead of warn, and how visible should provenance remain?
(011 lossiness fork, 011 provenance-visibility fork)

- Recommended path: Fail only on lossiness that creates misleading native
  truth or unparseable structure; warn on everything recoverable.
  Hard-fail set: structurally unparseable encodings (the engine already
  errors on unsupported pad layer/shape encodings in
  `import/kicad/skeleton.rs`) and any conversion that would silently
  fabricate connectivity/geometry the source did not assert.
  Warn-and-commit set: dropped cosmetic/tool-specific features,
  approximated/inferred geometry, unknown standards basis, missing
  pin/pad maps — recorded as `LossinessRecord` with severity and a repair
  proposal; map `lossiness_kind` to disposition (SemanticsUnknown
  affecting connectivity => block-eligible;
  Unsupported/Approximated/Inferred/Dropped => warn). `ImportProvenance`
  is permanently retained, default-quiet, on-demand visible: never deleted
  (repair appends provenance, never erases), collapsed by default (objects
  look native), always inspectable in the object inspector and present in
  audit/standards reports, surfaced MORE prominently on a finding
  referencing imported geometry or any object carrying
  unknown_basis/lossiness flags. Store provenance in a dedicated Import
  Map / provenance shard (resolver-assembled), not inline on every object
  render.
- Production example: KiCad importers warn-and-continue on unsupported
  features (report of dropped items, import completes) and hard-fail only
  on a malformed file; KiCad retains the original footprint library
  link/timestamps on imported footprints indefinitely, viewable in
  properties but not on canvas. Altium's Import Wizard reports
  unsupported/lossy conversions but completes the migration, aborting only
  on an unparseable file, and keeps source-import provenance in project
  history/component source after migration — board edits native, lineage
  queryable. Both put the fail line at parse-integrity and keep lineage
  permanent and quiet.
- Engineering reason: 011 states severe lossiness may block commit if it
  would create misleading native truth while lesser lossiness commits with
  visible findings; the discriminator is whether the loss
  fabricates/corrupts authoritative state vs drops/approximates
  recoverable detail. The engine already embodies the parse-integrity fail
  line (`skeleton.rs` explicit errors). Deleting provenance would break
  finding-fingerprint stability (fingerprints bind imported identity
  through `import_key`) and the 010 requirement that reports disclose
  import-preserved-but-suspect geometry; so "how visible" is answered by
  relevance — always stored, surfaced proportional to risk. Tying the
  block to `lossiness_kind=SemanticsUnknown`-on-connectivity keeps it
  deterministic and honors resolver-recovery doctrine (incoherent imports
  open in diagnostic mode, not as truth).
- Residual risk: Judging which inferred conversions are "misleading" is a
  per-feature call that can drift — keep the block condition a small
  explicit named set (connectivity-fabrication + parse failure) and route
  everything else to warn + `LossinessRecord` + proposal, biasing toward
  import-succeeds-with-evidence. Long-lived provenance accumulates and
  could bloat shards or confuse users who think the project is fully
  native — mitigate with the dedicated provenance shard (not inline) and
  contextual surfacing only.
- Ratification class: recommended-default-pending-ratification.

### Q: Which export formats are product-critical vs future interop targets?
(011 export-format fork)

- Recommended path: Product-critical (v1, already implemented): Gerber
  RS-274X (copper/mask/silk/mechanical), Excellon drill, BOM, pick-and-
  place — the fabrication/assembly handoff. Near-critical next: KiCad
  export (round-trip with the primary import path) and IPC-D-356A netlist
  (cheap, high test-coverage value). Future/deferred: IPC-2581, ODB++,
  STEP/IDF — spec-stubs only per `CLAUDE.md`, gated behind prerequisites
  (3D model substrate for STEP/IDF, structured panel/stackup data for
  IPC-2581/ODB++).
- Production example: Every fab accepts Gerber RS-274X + Excellon as the
  baseline (KiCad and Altium both treat these as default fabrication
  output); both additionally ship IPC-2581 and ODB++ as modern single-file
  alternatives layered on top, confirming Gerber/Excellon are critical and
  IPC-2581/ODB++ are value-add.
- Engineering reason: Verified the engine implements RS-274X
  (`export/gerber_mechanical.rs`, `copper.rs`, `mask.rs`,
  `silkscreen.rs`) and Excellon (`export/excellon.rs`); `CLAUDE.md` lists
  BOM/PnP as shipped — so product-critical is grounded in what fabs
  require and what is built. IPC-2581/ODB++/STEP/IDF are explicitly
  spec-stub-only with no implementation tracker, and STEP/IDF need the M8
  3D substrate, so they cannot be v1-critical without inventing scope.
- Residual risk: Some advanced fabs increasingly request IPC-2581/ODB++
  and may view Gerber-only as dated — mitigated by IPC-2581 being a
  defined-but-deferred registry entry with a named prerequisite (a
  scheduled interop target, not a gap), and the artifact model already
  records the metadata IPC-2581 would carry.
- Ratification class: research-conclusive.

---

## 18. AI tooling contract, transports, and assistant surface

### Q: What minimal CLI/MCP query family ships first, which object categories need stable IDs, and which families should ship at all in the first slice?
(004 Q1, 004 Q3, AI_CLI_MCP Q2, 006 Q3)

- Recommended path: Ship only families with existing engine reads to
  fold: `project.get_context` (DatumContextEnvelope), `object.get`,
  `object.neighbors` (`ComponentInstance`-keyed), `schematic.query`,
  `pcb.query`, `library.query`, `rules.query`, `checks.query`/`check.run`,
  plus `provenance.query` (`import_key`); the assistant's first-slice
  context subset is the read-only envelope of active project +
  `model_revision`, active projection, current selection (object IDs),
  visible findings. Defer `manufacturing.query_projection`,
  `artifacts.query`, `transactions.query`, `relationships.query`,
  `selection.describe` until their backing state exists. Stable IDs needed
  in the first proof beyond selected PCB objects/findings/proposals/
  transactions: `ComponentInstance` (the canonical join), `Net`/`NetId`
  (clearance/short findings are net-scoped), and `Artifact` (so a
  check/export produced in the loop is referenceable) — all already typed
  in the substrate; defer library-pool entity IDs and rule IDs to the
  second slice.
- Production example: KiCad's IPC v9 first exposed `get_open_documents`,
  `get_items`, `get_selection`, `get_net_classes` before broadening to
  edit commands — read-first, selection-anchored. KiCad's `kicad-cli` read
  surface is deliberately small and grows incrementally. Altium exposes
  persistent Unique IDs on components, nets, and primitives
  (`UniqueId`/`UniqueIdPath`) surviving schematic-PCB sync. GitHub Copilot
  Chat's initial context was selection + active file + diagnostics before
  repo-wide history.
- Engineering reason: Every query must return stable `ObjectId` (Uuid) +
  `ObjectRevision` + `ModelRevision` so a later proposal can be rejected
  as stale; a small family forces the revision-keying and
  `ComponentInstance` join contract to be exercised end-to-end before
  surface multiplies. Shipping a family with no revision-keyed backing
  (`transactions.query` before the journal is listable, `artifacts.query`
  before artifact metadata exists) would mint an API the engine cannot
  honor deterministically. Verified substrate types `ObjectId=Uuid`,
  `ObjectRevision`, `ModelRevision`, `ComponentInstance` — buildable today
  without new identity primitives; a bounded physical correction is
  meaningless without stable Net identity and the `ComponentInstance` join
  (a pad belongs to a package belonging to an instance).
- Residual risk: `object.neighbors` must enforce the `ComponentInstance`
  join in code; if it falls back to refdes string matching under time
  pressure the contract regresses silently — hold `object.neighbors` until
  the join is real and add a drift gate asserting neighbors never emits a
  refdes-keyed edge. `NetId` stability under merge/rename must be live
  before this slice or net-scoped findings reference unstable handles.
  Selection context can go stale mid-conversation — re-read selection at
  proposal-creation and stamp the proposal with the `model_revision` it
  was prepared against.
- Ratification class: recommended-default-pending-ratification.

### Q: Should the first slice expose a local session socket, CLI discovery, or both, and how is shell/discovery resolved (incl. MCP write-tool gating and read-only artifact routing)?
(004 Q2, 005 Q3, AI_CLI_MCP Q4, 005 Q5)

- Recommended path: Both transports, CLI discovery as the authoritative
  bootstrap and the socket as an optional fast path: `DATUM_DISCOVERY`
  points to a small `.datum/` JSON record naming the running daemon's Unix
  socket; agents that cannot speak the socket fall back to
  `datum-eda context get`/`query` subprocesses on the same engine; the
  terminal env-var bootstrap (`DATUM_DISCOVERY`, `DATUM_CLI`,
  `DATUM_SESSION_ID`) carries both. The socket is NOT a privileged channel
  — it serves the identical contract classes, reusing the exact CLI
  `command_exec` path (no separate write code). MCP write tools land ONLY
  behind `commit()`/`commit_journaled()` (now satisfiable since the
  journaled core exists); until a domain op is on `commit_journaled`, its
  MCP write tool stays absent and never uses the legacy
  `write_canonical_json` bridge. Shell resolution: `$SHELL`, then the
  passwd entry, then `/bin/sh`, surfacing a non-fatal banner when `$SHELL`
  was empty; non-Linux is out of early scope (feature-gate cleanly). Scope
  the socket to the user's uid with 0600 dir perms.
- Production example: Verified the Datum daemon binds a Unix socket
  (`engine-daemon/src/main.rs` `UnixListener::bind` on `--socket`) and the
  Python MCP server connects over it (`server_runtime.py`), while the CLI
  runs the same engine in-process — the same coexistence as KiCad's
  `kicad-cli` (CI/headless) plus the kipy IPC socket (live editor). KiCad
  routes board mutations through the same `BOARD_COMMIT` object the GUI
  uses; no plugin-only unjournaled write path. VS Code resolves the
  default shell from `$SHELL` then OS user record then a platform default,
  never refusing a terminal; portable-pty exposes system shell resolution.
- Engineering reason: A Unix socket gives a single long-lived resolved
  `DesignModel` (one `ProjectResolver` assembly, cached `model_revision`)
  so an agent's read/propose/apply loop sees a coherent revision without
  re-importing per call; CLI subprocesses re-resolve each invocation
  (correct but slower, can race the live revision). Offering both keeps
  the determinism guarantee (stale-base rejection) on either path because
  both terminate at the same `commit()` + journal. An MCP write tool on
  the private JSON path would create AI-authored mutations with no
  `TransactionRecord`, no undo, no `model_revision` bump — the unjournaled
  AI surface the audit forbids; binding MCP writes to `commit_journaled`
  guarantees AI authoring is always undoable and revision-stamped. The PTY
  process owns shell state, so "unsupported shell" is a non-concept — the
  only failures are no resolvable executable (the `/bin/sh` floor) and a
  non-Linux platform without PTY support (out of Linux-first scope).
  Read-only artifact generation emits nothing through `commit()`, so it
  stays CLI-bridged for the first slice and the daemon path is a later
  optimization once T0 is unified.
- Residual risk: Two transports double the surface that must reject
  private mutation — mitigate by reusing the exact CLI `command_exec` path
  (the daemon dispatch already does). Env-var discovery leaks the socket
  path to every child process — scope the socket to the user's uid (0600).
  Daemon dispatch currently bridges only board ops; native
  schematic/library/manufacturing MCP writes stay blocked until daemon
  dispatch + typed ops exist. `/bin/sh` fallback loses interactive
  features — surface the resolved-shell banner.
- Ratification class: research-conclusive.

### Q: How much non-mutating assistant/agent/terminal activity is persisted in project history vs local session, and do AI analyses/reports become first-class artifacts?
(004 Q5, 004 Q6, 005 Q4, 006 Q4, AI_CLI_MCP Q9)

- Recommended path: Persist nothing non-design-significant in project
  history. The journal records only design-significant outcomes
  (proposals, transactions, artifacts, checks, imported evidence); reads,
  queries, conversational turns, and non-mutating agent activity go to
  local/session history only, optionally as opt-in telemetry. Three-tier
  for terminal-launched commands with no overlap: raw shell history stays
  in the shell; Datum CLI/MCP invocations crossing the Datum boundary are
  local session telemetry keyed to the session; ONLY when a command
  produces design state is the command/session recorded as PROVENANCE on
  that record. AI analyses/reports: first-class journaled `Artifact` ONLY
  for deterministic regenerable outputs (check reports, route-strategy
  reports, equivalence comparisons) tied to `model_revision` + generator
  version + validation state; NOT for free-form AI prose/explanations
  (session state until a user explicitly saves one). Assistant
  conversations are not persisted in the project (local/session only). The
  dividing line is reproducibility: re-running at the same revision yields
  byte-identical output => artifact; depends on nondeterministic
  generation => not.
- Production example: Git records committed changes, not every git
  status/log/diff a tool ran; the commit log carries author/committer
  provenance. Altium separates OutJob-generated artifacts (versioned,
  traceable) from the Messages/ECO log (session diagnostics) and treats
  reports/BOMs/docs as managed outputs, not chat logs. Cursor/Copilot keep
  conversation history in local/workspace state, not committed into the
  repo — the repo gets accepted diffs, not the chat log. KiCad keeps DRC
  reports as on-demand exports, not project-history entries.
- Engineering reason: Verified `model_revision` is sha256 over object
  revisions + accepted-transaction tip (`compute_model_revision`); only
  transactions move it, and the journal (`journal.rs`) appends one
  `TransactionRecord` per commit by construction so read traffic
  structurally cannot enter it. Logging reads/chatter into project history
  would create a second non-revision-keyed history competing with the
  journal for authority — the alternate-authority regression the spine
  forbids — and would bloat shards / break undo semantics (you cannot undo
  a "report"). A deterministic report satisfies the artifact requirement
  (records `model_revision` + generator provenance, invalidated by
  revision movement); non-deterministic AI prose cannot be revision-keyed
  for equivalence (re-generation won't match the content hash) so making
  it a first-class artifact would poison stale-artifact detection.
- Residual risk: Support/forensic users may later want a who-read-what
  trail or a record of which AI suggestion led to which accepted
  transaction — capture the latter as proposal-lineage metadata ON the
  transaction, and the former as a separate clearly-labeled telemetry log
  keyed to the tool session, never folded into the design journal. Agents
  may embed nondeterministic commentary inside an otherwise-deterministic
  report — separate the machine schema (artifact) from the AI narrative
  (session text).
- Ratification class: research-conclusive.

### Q: Should the assistant surface be deferred, should it apply approved proposals in the first slice, how is it distinguished from a terminal, and what actions hand off?
(006 Q1, 006 Q2, 006 Q5, 006 Q6)

- Recommended path: Defer the assistant's APPLY capability until (a) the
  real PTY terminal and (b) the proposal/transaction/commit path are
  product-real; a read-only-plus-draft assistant (no commit power) is safe
  to ship early. First slice: read-only + proposal drafting + an apply
  ACTION gated behind an explicit human UI approval (the Apply card,
  delegating `DatumProposalTool.apply` -> `DatumCommitTool` ->
  `commit()`); the assistant may draft and may trigger apply only after a
  product-UI approval gesture, never auto-apply. Make the two surfaces
  categorically distinct in affordance: the terminal is a monospace PTY
  grid with a shell prompt; the assistant is a card-structured panel
  (Context/Query/Check/Proposal/Apply/Handoff cards) with NO executing
  shell prompt — Apply controls never hidden in generated text. Hand off
  (never perform from the assistant): running arbitrary external
  tools/agents (codex, aider, vendor CAM) and free-form shell/VCS ops
  (PTY terminal); bulk/destructive/cross-domain ECO exceeding a bounded
  single proposal; interactive geometric editing (drag-route, manual
  placement — canvas-native). The assistant performs only read/explain,
  run deterministic checks, draft ONE bounded proposal, and trigger an
  approved apply.
- Production example: KiCad shipped its IPC/scripting API and stable
  plugin contract years before any AI panel; Altium exposed
  scripting/ECO/OutJob long before AI features — the deterministic tool
  contract preceded the conversational surface. Cursor's Agent and Copilot
  Chat draft edits and require an explicit Accept/Apply click; Altium's
  ECO drafts the change set automatically but Execute Changes is a
  deliberate human action. VS Code separates the Terminal panel (PTY grid)
  from Copilot Chat (card/message UI) with different icons/input
  semantics — chat input never executes as a shell, "run this command" is
  a button handing off to the terminal.
- Engineering reason: The assistant has no private editing API — it can
  only call `DatumQueryTool`/`CheckTool`/`ProposalTool`/`CommitTool`; if
  those classes are not real it has nothing to call and would grow a
  shadow path (the fake-terminal/hidden-GUI-mutation regression the spine
  forbids). Deferring the apply capability guarantees the assistant is
  born bounded. Allowing apply via an explicit UI gesture (not generated
  text) keeps human approval in the loop while proving the full
  draft->approve->commit->journal->invalidation chain. A card-based UI
  with no executing prompt makes "disguised terminal as a design mutation
  path" structurally impossible. Arbitrary tool execution requires a PTY
  (none in the assistant by design); interactive editing requires the
  canvas operation generator; bulk/cross-domain ECO requires
  proposal-first review exceeding a single bounded card — handing these off
  keeps the assistant inside its five-capability envelope and keeps manual
  workflows real.
- Residual risk: Owner may want an early read-only assistant for demos —
  supported (read-only-plus-draft has no commit power), so "defer" means
  defer the APPLY capability, a scope dial the owner sets. Conflating
  "assistant triggers apply after human click" with "assistant
  auto-applies" is a UX trap — make the Apply card a distinct control whose
  activation is the recorded acceptance path, never inferred from a chat
  message. Card UIs can still tempt command typing — route any
  command-shaped input into a Handoff card ("Run in terminal?") rather than
  executing it. The "bounded single proposal" vs "bulk ECO" boundary is a
  judgment call — define a concrete threshold (proposals touching > N
  objects, or any cross-domain/standards/import-repair op) forcing
  handoff; owner sets N.
- Ratification class: recommended-default-pending-ratification.

---

## 19. Terminal emulation stack and copy/paste

### Q: Which terminal emulation library/abstraction should Datum use, and what minimum copy/paste/selection is required to ship product-real?
(005 Q1, 005 Q6)

- Recommended path: Use Alacritty's terminal core crates —
  `alacritty_terminal` (VTE/ANSI grid + state) paired with `portable-pty`
  (wezterm) for PTY allocation — rendered through the existing wgpu/winit
  substrate (`gui-render`/`gui-app`) for glyph drawing; do not build a VT
  parser from scratch or embed a full external emulator process. Minimum
  copy/paste/selection bar to ship product-real: (1) mouse-drag cell
  selection with visible highlight; (2) copy to system clipboard;
  (3) paste with bracketed-paste mode enabled (so multi-line pastes do not
  auto-execute); (4) select-all and word/line double/triple-click
  selection; (5) keyboard copy/paste bindings distinct from terminal
  control sequences (Ctrl+Shift+C/V so Ctrl+C still sends SIGINT).
  Scrollback and search are also required by the doc but are separate from
  copy/paste.
- Production example: Zed's built-in terminal is built on
  `alacritty_terminal`; WezTerm uses `portable-pty` + termwiz; both are
  production terminals. Alacritty's defaults are the canonical minimum:
  VT-aware cell selection, bracketed paste by default, Ctrl+Shift+C/V that
  do not collide with Ctrl+C SIGINT, triple-click line selection; xterm
  established bracketed-paste mode (DECSET 2004) which
  `alacritty_terminal` implements in the core Datum would consume.
- Engineering reason: `alacritty_terminal` is the only mature Rust crate
  cleanly separating VT/ANSI/grid state from rendering, mandatory because
  Datum must render cells through its own wgpu pipeline
  (visual-regression-gated per the Screenshot Goldens memory), not a
  foreign GL context; `portable-pty` abstracts PTY allocation under a
  non-copyleft (MIT) license satisfying the No-Copyleft-Integration rule —
  both crates are permissively licensed for direct linking, unlike a GPL
  emulator that would force subprocess/IPC isolation. Bracketed-paste is
  the load-bearing safety requirement: pasting a multi-line block with a
  newline into a naive terminal auto-executes every line — a real footgun
  when an agent or user pastes a script that can run Datum CLI mutations;
  distinct copy/paste bindings are mandatory to preserve Ctrl+C as SIGINT
  (interactive/long-running process support needs working job-control
  signals). `alacritty_terminal` already implements DECSET 2004, so this
  is wiring, not new parser work.
- Residual risk: Owner taste may prefer a different glyph stack — confirm
  `alacritty_terminal` and `portable-pty` licenses (both MIT/Apache)
  before linking and verify the wgpu cell renderer can hit
  visual-regression goldens for ANSI color/cursor before declaring
  product-real. System clipboard access on Wayland vs X11 differs — use a
  clipboard crate (e.g. arboard) handling both and verify paste-into-shell
  with bracketed mode against a real agent-paste scenario.
- Ratification class: recommended-default-pending-ratification.

---

## 20. Manual editor baseline: physical tools, ERC/DRC honesty, proof fixture

### Q: Which physical editing tools are mandatory before push-and-shove, and is freehand track drawing / component flip in the first slice?
(002 Q2, PCB_LAYOUT_TOOL_CONTRACT Q6, PCB_LAYOUT_TOOL_CONTRACT Q3)

- Recommended path: Mandatory pre-shove set: `SetBoardOutline`,
  `PlaceFootprint`, `MovePlacement`, `RotatePlacement`, `RouteTrack`
  (single-segment manual, no auto-avoid), `EditTrack`, `DeleteTrack`,
  `PlaceVia`, `SetRouteLayer`, and ratsnest/airwire display.
  Push-and-shove, length tuning, and differential-pair routing are out.
  Ship a minimal typed `draw_track` (`AddTrack` as an `OperationBatch` of
  segments, fixed net/width/layer, NO push-and-shove, NO
  interactive-width-from-rules) because it is the load-bearing migration
  TEMPLATE the contract recommends retiring first (`routing_net.rs:91`);
  rich/automated routing stays in the existing 60+
  `route_path_candidate` kernel via proposals. Component flip/change-side
  IS in the first slice as a journaled `SetComponentSide` Operation, and
  pad-geometry mirroring IS in scope (flip without mirroring is a
  correctness bug) — retire the native `.layer` field write in
  `component_layer.rs` into the typed op.
- Production example: KiCad had fully usable manual segment routing and a
  ratsnest for years before the CERN PNS router landed in v4/v5; the
  legacy interactive router was place-segment + via + layer-switch, and
  KiCad still ships Highlight Collisions/Walk Around/Shove as modes
  layered over the same base track primitive. Horizon exposes a simple
  interactive route tool emitting a track document-action while leaving
  sophisticated routing to higher tooling. Altium "Flip Board"/L-key and
  KiCad F-key move the footprint to the opposite side AND mirror all
  pads/courtyard/silk as one atomic undoable operation; neither offers a
  side-change leaving pad geometry on the original side.
- Engineering reason: Shove/avoidance is an interactive convenience
  producing the same RoutingOps as manual routing; per the spine,
  interactive behaviors are consumer-side and reduce to typed operations,
  so shove cannot be a model-level prerequisite — mandating it first would
  invert the dependency (a router policy gating a basic primitive).
  `draw_track` is the recommended first migration because it is the
  simplest non-trivial geometry op and proves the `AddTrack`/`AddVia`
  batch-as-one-undo pattern every other PCB op reuses; deferring it
  entirely would leave the first slice with only deletes and prove nothing
  new about native-write retirement. A side change that does not mirror
  pad coordinates produces a board where bottom-side pads are placed as if
  top-side — DRC clearance, drill, and Gerber projection all wrong, and
  any export round-trip fails; mirroring must be
  exactly invertible (flip-then-flip == identity, byte-for-byte) using
  integer-nm mirroring about the package origin.
- Residual risk: Manual-only routing on a dense board is tedious — expose
  the existing route-proposal kernel as an optional proposal-first assist,
  not by blocking the baseline on shove. A track with no interactive DRC
  may create violations the user only sees on a later `run_drc`
  (acceptable for a first slice; batch DRC catches it; online-DRC-on-draw
  is a known later need). Floating-point/rounding in the flip mirror
  transform could break the round-trip undo proof — use integer-nm
  mirroring.
- Ratification class: research-conclusive.

### Q: What ERC/DRC completeness makes the first manual workflow honest rather than cosmetic, and which manufacturing projections are mandatory in the first proof?
(002 Q5, 002 Q4 partial)

- Recommended path: Honest baseline = the rules that catch failures a
  manual user can create in the proof board, run deterministically, each
  finding navigable to affected objects via `ComponentInstance`/`ObjectId`
  — NOT a growing rule count. Minimum DRC: clearance, track width vs
  net-class, unconnected (ratsnest-derived), and zone-fill-stale/unfilled
  (hard finding). Minimum ERC: unconnected pin, pin-type conflict,
  unresolved/duplicate reference. The engine already has 7 ERC + 7 DRC at
  0% FP/FN; the honest bar is that these run on the resolved model and
  findings navigate plus are revision-keyed, not that the count grows.
  Surface explicit rule-coverage metadata on every `CheckRun` so a clean
  result states its own scope. Mandatory first manufacturing proof: Gerber
  copper + Excellon drill + soldermask + paste (per the
  manufacturing-projection theme).
- Production example: KiCad's DRC is considered usable for sign-off once
  clearance + track width + unconnected + courtyard + via pass; earlier
  releases shipped a smaller deterministic set still trusted because
  results were navigable and reproducible. Altium gates manufacturing on
  its DRC batch where unrouted-net and clearance violations are errors —
  completeness is defined by "no error-severity violation outstanding",
  not total rule count.
- Engineering reason: Honesty is a determinism + navigability property,
  not a coverage-count property: the spine requires `CheckRun`/
  `CheckFinding` tied to `model_revision` and findings keyed by stable
  `ObjectId`/`ComponentInstance`. A small deterministic navigable rule set
  is honest; a large non-navigable or non-deterministic set is cosmetic.
  Datum's existing 0% FP/FN gate establishes the determinism floor; the
  only new requirement is revision-keying and `ComponentInstance`
  navigation, which the substrate now supports.
- Residual risk: Users may read a small rule set as "not real DRC" and
  over-trust a clean result — mitigate by surfacing rule-coverage metadata
  on every `CheckRun` so a clean result states its scope. Shipping copper
  export before the ZoneFill split would ship dishonest copper — the gate
  must block copper export of unfilled zones from day one (the `Zone`
  struct and `copper.rs` need the fill field before this proof).
- Ratification class: recommended-default-pending-ratification.

### Q: What smallest board defines the first manual editor / performance proof, and what perf budgets/metrics are needed?
(002 Q1, 012 Q1, 012 Q2, 012 Q6)

- Recommended path: Use a known small REAL reference circuit for the
  manual-editor proof (a resistor-divider-plus-connector OR the existing
  `datum-test`/`airwire-demo` fixture), containing at least one
  `ComponentInstance` joining a `PlacedSymbol` + `PlacedPackage`, one net
  spanning two pins, one zone (for ZoneFill honesty), one routed track —
  two parts is too thin to exercise the electrical-to-physical join, net
  merge, and zone-fill gates. Pin a SEPARATE single representative
  small-medium PERFORMANCE fixture (~50-150 components, ~200-500 nets, 2-4
  layers, a few hundred tracks, >=1 ground/power zone) — large enough that
  pan/zoom, selection, DRC, projection-refresh latencies are
  regression-meaningful, small enough for CI; use a real board. Require
  hard numeric budgets up front for INTERACTIVE-latency metrics
  (pointer/drag preview, selection+inspector, commit latency, pan/zoom
  frame time); budgeted-but-async-tolerant for full DRC, artifact
  generation, projection regeneration; soft "interactive on the fixture"
  budget for open/save; workspace restore bounded but non-blocking.
  Proposed starting budgets (owner-tunable): pointer/drag <=16ms target /
  <=33ms ceiling; selection+inspector <=50ms; commit <=100ms p95; project
  open <=1s; workspace restore <=200ms non-blocking; projection refresh
  async, first-paint-of-stale-state <=100ms.
- Production example: KiCad ships small-but-complete reference projects
  (pic_programmer, video) as round-trip/regression fixtures and benchmarks
  performance against real community boards ~100 parts
  (kicad-test-boards), never synthetic stress files; its GAL canvas
  rewrite explicitly targeted 60fps (16.6ms) interactive rendering while
  DRC/zone-fill run as cancellable background passes with progress. Altium
  ships LedBlinky/Bluetooth Sentinel tutorial projects and runs DRC as a
  batch with a progress dialog (async, not frame-budgeted) while
  drag/route preview stays real-time. Nielsen's thresholds (100ms instant,
  1s flow, 10s attention) are the canonical sources for the proposed
  numbers.
- Engineering reason: Verified `ProjectResolver` computes `model_revision`
  over real shard content hashes, so the fixture must be a real on-disk
  project, not an in-memory stub. A two-part board cannot exercise net
  merge (lowest-`NetId` survivor), zone fill state, or a meaningful
  ratsnest, so it would pass the gate while leaving load-bearing
  invariants untested. Perf budgets must be anchored to a fixture before
  any workflow is product-ready; a two-part board hides O(n)/O(n^2) costs
  (clearance, ratsnest, zone fill) so budgets set on it are meaningless,
  while a 10k-part board makes early budgets impossible — the 50-150 band
  is where existing DRC/ratsnest/copper-fill costs first become visible
  while staying CI-cheap. The interactive-vs-async split is industry
  standard; commit latency specifically must be budgeted because
  `commit_journaled` includes validation + journal fsync + shard
  promotion, and an unbudgeted fsync-on-every-edit can make the editor
  feel slow — the one place the durable-commit design can hurt feel.
- Residual risk: A reference circuit large enough to exercise all gates
  can anchor perf budgets prematurely — keep the perf fixture decision
  separate from the proof fixture; a single perf fixture under-tests
  large-board scaling — add a second large fixture later without gating
  first-slice readiness. fsync-bound commit may exceed 100ms on spinning
  disks/networked filesystems — treat the commit budget as p95 on local
  SSD, document the storage assumption, and allow batched/group commits
  for rapid interactive gestures while keeping each accepted gesture
  journaled. The numeric budgets are genuine owner taste once fixtures
  exist — these are defaults to ratify, not law.
- Ratification class: recommended-default-pending-ratification.

---

## 21. Application quality: crash testing, experimental gating, diagnostics

### Q: Which workflows can be labeled experimental, which forced-failure scenarios run in CI vs manual, and what is the minimum diagnostic bundle?
(012 Q4, 012 Q8, 012 Q5)

- Recommended path: Experimental-eligible: anything that does NOT touch
  committed data integrity or honesty — push-and-shove/auto-routing
  assist, AI/assistant proposals, advanced projections (3D,
  panelization), large-board perf, breadth tools beyond baseline. NOT
  experimental-eligible (must meet the bar before exposure): the single
  `commit()`/journal path, crash recovery, resolver recovery, durable
  undo, ZoneFill honesty, artifact traceability, PTY-real terminal
  mutation routing — rule: a feature can be experimental only if its
  failure cannot corrupt source state, fabricate copper, or hide a
  deviation; experimental output carries a non-removable provenance flag
  so a fab package from an experimental path is self-identifying. CI
  forced-failure scenarios (deterministic): kill-before-journal-append,
  kill-after-append-before-promotion, corrupted/truncated staged bytes,
  corrupted journal line (already tested), duplicate transaction id
  (already tested), missing required shard (already tested),
  split/incoherent project open, stale `model_revision` commit guard
  (already tested). Manual/periodic fault-injection: real power loss,
  full-disk during fsync, lying-fsync filesystem, concurrent-process
  corruption. Minimum diagnostic bundle: project `model_revision` +
  journal tip; the `ResolveDiagnostic` list; recent `TransactionRecord`s
  (id, batch, before/after revision, diff, provenance); active
  variant/board/output-job context; check-run summary; artifact metadata
  for the selected output job; structured open/save/check/artifact/tool
  logs — enough to re-resolve the model and replay the failing
  transaction, with a scrub/redaction (geometry-stripped) mode.
- Production example: KiCad ships features behind "Enable experimental
  features" (new zone fill algorithm, ODB++, IPC-2581 were experimental)
  but file save/undo/DRC core were never experimental; Altium ships
  labeled Preview features while core save/ECO/DRC are GA — both
  quarantine breadth, never integrity. SQLite (which Datum uses) simulates
  power loss/partial writes/IO errors deterministically in CI while true
  media-failure testing is separate. Altium's Capture Crash Report and
  KiCad's bug-report guidance (project files + version + reproduction) plus
  structured logs mirror the re-resolve+replay design, like Git bug
  reports (object DB + reflog reproduce state).
- Engineering reason: 012's rule (a narrow professional-quality workflow
  beats a broad demo) makes the experimental discriminator the same as the
  proposal/direct-commit one: blast radius into committed authority and
  honesty. A feature whose worst case is "it didn't help" is safe
  experimental; one whose worst case is "silently corrupted the board or
  shipped fake copper" is not. Verified the substrate already has CI tests
  for journal-layer failure modes (duplicate replay, parse error keeping
  valid prefix, missing shard, stale-revision guard); the remaining
  CI-eligible scenarios are simulatable by injecting failures at the
  staged-write/journal-append/promote boundaries of `commit_journaled`
  without real power loss. The substrate already produces every diagnostic
  primitive (`model_revision`, `TransactionRecord`, `CommitDiff`,
  `CommitProvenance`, `ResolveDiagnostic`), so the bundle is a
  serialization of existing state, not new instrumentation — the minimum
  is everything needed to deterministically re-resolve and replay.
- Residual risk: Experimental labels get ignored by users who then trust
  experimental output for manufacturing — make experimental output carry a
  non-removable provenance flag so a fab package from an experimental path
  is self-identifying. Simulated crash points may not match real
  kernel/filesystem write-reordering — the CI simulation must inject at the
  exact fsync boundaries the design claims, and periodic real-hardware
  runs remain required to catch fsync-honesty gaps the simulation cannot
  model. Bundles may contain proprietary design data — the bundle format
  needs a scrub/redaction (structure-only) mode for users who cannot share
  board IP.
- Ratification class: recommended-default-pending-ratification.

---

## Cross-doc reconciliations

The following questions appeared in multiple source docs/contracts and were
merged into single resolutions above; this section records the merges so a
later folding pass can update every source.

- Panelization timing: `000B`/`000F` say buildable-now on imported boards
  gated only on a resolvable board ref + isolation gate;
  `MANUFACTURING_OUTPUT_TOOL_CONTRACT` Q7 says defer the panel object to
  M8. Reconciled by separating gate from feature (Section 13): defer the
  panel OBJECT / step-and-repeat / rails-tabs-V-scores to M8, but land NOW
  the panelization-isolation drift gate and reserve `--panel` as a
  hard-erroring export context parameter. When built in M8, scope =
  repeated-board arrays + routed tabs + mouse bites first, with
  `PanelProjection.instances` modeled as a list of (board_id,
  board_revision, transform) from day one.
- Integer `ObjectId` scope: four docs (`000` Q1, `000C` Q2 identity half,
  `000D`, `001` Q5) with two sub-questions (adopt-or-not,
  global-or-project-local). All agree: keep Uuid permanent, reject global
  outright, allow only a project-local display/serialization alias if ever
  revived. Merged into Section 1; redundancy removed, no contradiction.
- Direct-commit vs proposal boundary: eight docs (`001` Q1/Q2/Q8, `002`
  Q6, `004` Q4, `AI_CLI_MCP` Q3, `SCHEMATIC` Q3, `LIBRARY` Q7) each
  restating the same blast-radius predicate for a different surface.
  Reconciled into one unified rule keyed on blast radius not surface/verb
  (Section 7), with surface-specific defaults folded in (CLI
  direct + `--preview`, AI no-auto-apply, library
  binding-relevant-field trigger). No genuine disagreement.
- Unknown-basis severity/handling: six docs (`009` Q2/Q3, `RULES_CHECKS`
  Q4, `008` unknown-basis, `LIBRARY` Q5, `011` imported-stricter) with
  consistent guidance. Reconciled by mandating ONE centralized
  `CheckProfile` knob shared across 008/009/011 so UnknownBasis means the
  same thing everywhere (Section 15), eliminating cross-domain divergence.
- `ComponentInstance` join-key migration: appeared as both an identity
  question (`004` Q3, `AI_CLI_MCP` Q2) and a BOM/PnP row-key question
  (`000F` BOM, `MANUFACTURING` Q4). No conflict; the manufacturing-row
  instance is the concrete application of the identity rule. Kept the
  BOM/PnP migration in the manufacturing theme (Section 12) and the
  stable-ID requirement in the AI theme (Section 18), cross-referencing
  the same `ComponentInstance` substrate fact.
- Finding-addressing/waiver fingerprint: four sources (`AI_CLI_MCP` Q5,
  `RULES_CHECKS` Q2, `009` waiver-metadata, `SCHEMATIC` Q5) with
  consistent direction. Merged into Section 8; the only added
  reconciliation is placing the author-waiver verb in the 009 unified
  contract (per `SCHEMATIC` Q5) rather than any domain-private contract.
- Scope persistence (`ElectricalScope`/`PhysicalScope`): `000E` Q2 and
  `003` both recommend derive-first, persist-only-when-a-gate-needs-
  non-1:1 (Gate 4 daughtercard). Merged into Section 9; no contradiction.
- Import-to-native conversion: `000`'s authority-flip/dual-write exit
  criterion is SUPERSEDED by the ratified posture (see
  docs/DATUM_PRODUCT_MECHANICS.md "Interop Boundary And Import Posture") —
  there is no flip and no dual-write window; native shards are the sole
  authority and the KiCad-text-plus-sidecar path is a transitional defect
  fixed by having import write native shards directly. `001` Q3 remains as
  the native-board-shard-first board-write resolution in Section 6
  (research-conclusive); the optional KiCad exporter is a separate
  deferred concern, not an authority question.
- First multi-board case (`000E` Q3, genuine-preference) vs first
  inter-board relationship (`000E` Q8, Interconnect in-scope): reconciled
  by sequencing — independent cooperating boards (Gate 2) first,
  daughtercard + Interconnect (Gate 4) second — so both hold without
  conflict (Section 9).
- Read-only artifact generation routing (`AI_CLI_MCP` Q10) and T0
  unification (`MANUFACTURING` Q5) overlapped: routing is downstream of
  unification. Reconciled by stating T0 unification is the correctness
  prerequisite done now (CLI-bridged read-only export stays transitional;
  daemon routing is a later optimization once T0 is unified), merged into
  one manufacturing resolution (Section 12).

## Stats

Total genuinely-open questions resolved: 92. Research-conclusive: 40.
Recommended-default-pending-ratification: 45. Genuine-preference-needs-owner:
7. (Questions the source docs mark as decided by the 2026-06-18
reconciliation were skipped, not counted here.)

## Closing Note (docs-are-clay)

These resolutions live in one consolidated record only as a ratification
convenience. Once the owner ratifies an item, it should be folded into the
"decided" / resolved section of its source decision doc
(`PRODUCT_MECHANICS_000`-`012` or the relevant `docs/contracts/*`), and the
corresponding "Open Owner Questions" entry removed there — docs are clay, and
the source decision record is the long-lived home. This file is a staging
ground, not the permanent authority; it should shrink as items migrate and may
be retired once every resolution has been folded back or explicitly rejected.
