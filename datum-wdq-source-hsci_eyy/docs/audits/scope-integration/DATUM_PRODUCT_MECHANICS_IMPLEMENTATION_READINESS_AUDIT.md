# Datum Product Mechanics Implementation-Readiness Audit

Status: coordination audit for parallel decision-doc workers.
Date: 2026-06-19

Owned file:
- `docs/audits/scope-integration/DATUM_PRODUCT_MECHANICS_IMPLEMENTATION_READINESS_AUDIT.md`

Do not treat this file as a replacement for the decision records. It is a
cross-doc audit checklist for making `PRODUCT_MECHANICS_002` through
`PRODUCT_MECHANICS_012` implementation-ready against the ratified mechanism
spine in `000`, `000B`, `000C`, `000D`, `000E`, `000F`, and `001`.

## Ratified Mechanism Vocabulary

These terms must be used consistently by every target decision doc.

- `DesignModel`: the single in-memory project authority assembled by the
  engine-owned `ProjectResolver` from source shards. It is never serialized as
  a monolith and never read directly as an on-disk authority.
- Source shards: deterministic persistence partitions. They are not schematic,
  PCB, library, rules, manufacturing, workspace, or artifact authorities.
- `ProjectResolver`: the only assembler of the resolved model. It validates
  shard versions, object references, relationships, library revisions, rule
  scopes, output contexts, proposal/transaction references, cache staleness,
  and artifact metadata. A split or incoherent project opens in recovery or
  diagnostic mode, not as accepted truth.
- `ObjectId = Uuid`: persisted surrogate identity. Native objects persist the
  v4 UUID generated at creation. Imported deterministic v5 UUIDs are demoted
  to import seeds persisted in an Import Map shard keyed by `import_key`, not
  `source_hash`.
- `ComponentInstance`: per-instance stable surrogate `Uuid` referenced by both
  `PlacedSymbol` and `PlacedPackage`. It is the canonical electrical-to-physical
  join and must replace reference-designator-string joins.
- `NetId` and `NetAnchor`: net identity is stable and bound to derived
  connectivity groups through resolver-refreshable anchors. Rename is a display
  change; merge survivor is lowest `NetId`.
- `Relationship`: authored binding with its own stable surrogate id and
  `RelationshipKind` in `{ImplementedBy, BoardOnly, SchematicOnly,
  ReverseEngineered, Pending, Mismatch}`.
- Derived relationship status: resolver-computed values
  `{Implemented, PendingImplementation, UnresolvedMismatch}`. They are
  recomputed on load and never persisted as source authority.
- Authored relationship intent: user/tool decision values
  `{LayoutDeviation, BoardOnlyObject, SchematicOnlyObject, ReverseEngineered,
  accepted-deviation}`. The resolver may report contradictions but may not
  silently overwrite them.
- Variant overlay: sparse authored overlay keyed by stable object identity.
  It stores only `Fitted`, `Unfitted`, option-link selections, and sparse
  per-variant relationship overrides. `NotApplicableForVariant` is derived
  during resolution and must not be stored.
- `Operation`, `OperationBatch`, `Proposal`, `Transaction`: every mutation from
  GUI, CLI, MCP, AI, importer, checker, router, terminal-launched tool, or
  script must reduce to typed operations. Automation and high-risk edits create
  proposals by default; accepted proposals and eligible direct edits become
  transactions.
- Single `commit()`: the only state mutation path. It applies in memory, stages
  shard bytes and fsyncs, appends a journal `TransactionRecord` and fsyncs
  that commit point, then atomically renames staged shards and fsyncs
  directories. No private file writer is acceptable except as a named,
  time-boxed migration defect.
- Journal: required source state from the first cut. Durable undo/redo are
  journal cursors; undo is a compensating transaction. Transactions are the
  sole producer of `object_revision` and `model_revision`.
- `model_revision`: sha256 over canonical sorted object revisions plus the
  accepted-transaction tip. Derived connectivity, zone fill, manufacturing
  projections, variant views, checks, and artifacts key staleness from revision
  state rather than ad-hoc dirty flags.
- `ZoneFill`: derived state separate from `Zone.polygon`. The polygon is the
  authored boundary. `ZoneFill{Filled|Unfilled|Stale|Unsupported, islands}`
  carries production copper geometry with provenance. Only `Filled` contributes
  copper to projection/export. Native unfilled zones emit no copper plus a hard
  finding.
- Production projections: live manufacturing views are T1 memoized projections
  over a T0 cold full-regeneration core. T0 is both exporter and equivalence
  oracle. T2 sub-region incrementality is deferred behind T0.
- Artifact: generated snapshot tied to model revision, board/panel context,
  variant, manufacturing plan, output job, generator version, validation state,
  and content hash/path. Generated files are not design authority.
- PTY terminal: a real terminal session allocates a real PTY and runs the
  user's shell. Terminal text is not an edit API; terminal-originated design
  changes still use proposals/transactions.

## Worker Workstreams

Use these workstreams to split the target-doc pass without creating divergent
mechanism language.

- Workstream A, manual and authority docs: `002` and `003`. Align manual editor
  readiness and schematic/PCB authority to the ratified relationship split,
  `ComponentInstance`, stable identity, and one `commit()` path.
- Workstream B, AI/tool/terminal/workspace docs: `004`, `005`, `006`, and
  `007`. Keep AI optional, terminal PTY-real, workspace non-authoritative, and
  all mutation surfaces on the operation/proposal/transaction path.
- Workstream C, library/rules/standards/import docs: `008`, `009`, `010`, and
  `011`. Align standards basis, check findings, waivers/deviations, import
  provenance, and library repair with stable object identity, Import Map
  `import_key`, proposals, and journaled transactions.
- Workstream D, quality bar doc: `012`. Convert implementation readiness into
  product gates: crash-safe commit, resolver recovery, deterministic checks,
  projection honesty, artifact traceability, and no hidden mutation paths.
- Workstream E, final integration. After workers edit decision docs, run this
  audit against all target docs and reject any doc that reintroduces alternate
  authority, stored derived state, source-hash identity, private writes, fake
  terminal semantics, or vague standards/compliance claims.

## Cross-Doc Invariants

Every target decision doc must preserve these invariants.

- Files are persistence partitions. A doc may discuss sheets, boards, library
  pools, output jobs, workspace layouts, terminals, or artifact files, but none
  of those can become a second source authority.
- `ProjectResolver` is mandatory mechanism, not background plumbing. Any doc
  that introduces project open, import, workspace restore, check execution,
  artifact generation, variant switching, or projection serving must identify
  the resolved `DesignModel` and model/revision context it operates on.
- Relationship language must separate `RelationshipKind`, derived status,
  authored intent, and variant population. `Implemented`, `PendingImplementation`,
  `UnresolvedMismatch`, and `NotApplicableForVariant` are not authored source
  states. `LayoutDeviation`, `BoardOnlyObject`, `SchematicOnlyObject`,
  `ReverseEngineered`, and accepted deviations are authored intent.
- Cross-domain joins must use `ComponentInstance`, not reference designators,
  names, paths, positions, sheet IDs, board IDs, or imported source object paths.
- Variant state is sparse overlay state. Switching active variant must compose
  an overlay and produce zero base writes. Only authored `Fitted`/`Unfitted`
  and sparse relationship overrides are stored; `NotApplicableForVariant` is
  derived.
- Every mutation path must converge on one `commit()` primitive and journal.
  GUI, CLI, MCP, assistant, terminal-launched tools, import repair, standards
  correction, check-generated repairs, library updates, output-job edits, and
  panel edits do not get private write paths.
- Proposals are not committed design state. They own candidate operation
  batches, rationale, checks, preview/diff, assumptions, risks, provenance, and
  apply/reject/defer state. Acceptance is what creates a transaction.
- Direct manual commits are allowed only for local, visible, undoable edits
  without hidden cross-domain, standards, import-repair, destructive, or batch
  implications. High-risk edits require proposals or explicit confirmation.
- Generated or derived state must be revision/hash-keyed. Connectivity graph,
  relationship status, zone fill, manufacturing projections, variant views,
  check results, and artifacts must not become alternate authorities.
- Zone fill must be honest. A zone boundary is not production copper. Native
  unfilled zones must render as unfilled/stale/unsupported and export no copper
  with a hard finding unless a real fill is available.
- Panelization is manufacturing state. Rails, tabs, mouse bites, V-scores,
  panel fiducials, tooling holes, coupons, labels, and panel notes must not
  mutate source board geometry unless an explicit board-level operation is
  accepted.
- Terminal sessions are real PTY-backed shells. Assistant surfaces are optional
  Datum-aware proposal/explanation surfaces. Neither may gain hidden edit powers
  or become required for manual EDA workflows.
- Standards and compliance wording must remain evidence-based. Datum can store
  basis, run checks, report findings, record waivers/deviations, and export
  reports; it must not imply product, process, regulatory, or third-party
  certification.
- Import preserves source provenance and unknown-basis markers. Repair,
  normalization, inferred schematic reconstruction, and standards correction
  are proposals unless covered by a narrow explicit automation policy.

## Target-Doc Audit Notes

### `002` Manual Editor Baseline

- Post-worker status: implementation-ready draft.
- Manual editor tools are now framed as operation-producing GUI surfaces that
  call the single `commit()` path or create proposals where required.
- Proof gates cover GUI bypass retirement, durable undo, journaled provenance,
  manual relationship proof, manufacturing previews, and quality thresholds.
- Remaining owner questions are narrow scope choices: first proof board,
  mandatory physical tools, project-local library timing, manufacturing
  projection minimums, and which legacy GUI helpers are temporary migration
  defects.

### `003` Schematic And PCB Authority

- Post-worker status: implementation-ready draft after reconciliation.
- The doc now separates authored `RelationshipKind`, authored intent/deviation
  records, derived resolver status, and derived variant population/applicability.
- `NotApplicableForVariant` is derived-only and must not be persisted as
  relationship source state.
- Manufacturing-only production objects are scoped outside schematic/PCB
  relationship state unless explicitly promoted to board-level geometry.
- `ElectricalDeviation` remains only as an owner-review fork question; first
  schema behavior should use accepted deviation metadata unless the owner
  ratifies a new primitive.
- Back-annotation, board-first reconstruction, deviations, import repair, and
  bulk ECO remain proposal-first unless explicitly narrowed later.

### `004` AI Tooling Contract

- Post-worker status: implementation-ready draft.
- The doc now defines shared CLI/MCP/query/check/artifact/proposal/commit
  contract classes and requires accepted mutations to call `commit()` and
  journal provenance.
- AI context exposes stable Datum model concepts rather than screenshots or
  hidden GUI automation.
- Remaining owner questions are first-slice tool family, context-discovery
  mechanism, stable-ID scope, automation policy, and artifact/history policy.

### `005` Embedded Terminal

- Post-worker status: implementation-ready draft.
- Terminal semantics are PTY-real: shell, cwd/env/session behavior, and normal
  Linux tooling are separate from design mutation authority.
- Terminal-launched tools discover context through Datum env/CLI/MCP, but
  design mutation still goes through proposals or journaled transactions.
- Remaining owner questions are terminal library/platform abstraction,
  persistence scope, socket/CLI discovery, command-history recording, and
  copy/paste/selection minimums.

### `006` Assistant Surface

- Post-worker status: implementation-ready draft.
- Assistant behavior is bounded to read, explain, draft proposals, and apply
  only through approved proposal/transaction mechanics.
- The assistant has no private API, no journal-writing access, and no hidden GUI
  command path.
- Remaining owner questions are defer-vs-ship timing, read-only/proposal/apply
  scope, initial safe contexts, persistence, UI distinction from terminal, and
  handoff rules.

### `007` Project Workspace Model

- Post-worker status: implementation-ready draft.
- Workspace state is separated from source authority and source shards; tabs,
  panes, profiles, context selectors, and terminal sessions do not become design
  authorities.
- The doc now describes persistence concepts for workbench/session/layout state
  and status indicators without making them mutation paths.
- Remaining owner questions are first workspace modes, project-local vs
  user-local persistence, session restore policy, profile ownership,
  dashboard/status indicators, audit journaling for workspace actions, and
  schema migration.

### `008` Library Component System

- Post-worker status: implementation-ready draft.
- Library primitives are now mechanism-level model objects with stable IDs,
  provenance, `ComponentInstance` binding, `PinPadMap`, padstack/process
  metadata, and journaled change propagation.
- Imported library data routes through provenance/import mapping rather than
  becoming trusted native data without review.
- Remaining owner questions are first pool types, approval states, IPC naming
  basis, first package families, unknown-basis strictness, and BOM compliance
  metadata.

### `009` Rules, Constraints, And Checks

- Post-worker status: implementation-ready draft.
- Rules/checks now resolve through `ProjectResolver`, produce versioned
  `CheckRun`/`CheckFinding` records, and generate proposal-first repairs.
- Check findings tie to revisions, affected object IDs, rule basis, waiver or
  deviation records, and invalidation through journaled transactions.
- Remaining owner questions are first profiles, severities, unknown-basis
  behavior, waiver/deviation approval metadata, mandatory IPC/process
  categories, and GUI vs CLI/MCP rule editing.

### `010` Industry Standards And Compliance

- Post-worker status: implementation-ready draft.
- Standards support is framed as modeled basis, registry metadata, checks,
  reports, waivers/deviations, and audit evidence, not certification.
- Standards reports/artifacts carry model revision, basis, output settings,
  generator/tool version, uncertainty, and validation state.
- Remaining owner questions are allowed wording, first registry subset, project
  posture defaults, data-egress defaults, mandatory metadata, and regulated
  audit/sign-off threshold.

### `011` Import And Interop Role

- Post-worker status: implementation-ready draft.
- Import now distinguishes import session, source object provenance, Import Map
  `import_key`, native conversion, lossiness, unknown-basis markers, and repair
  proposals.
- Imported board-first data aligns with `RelationshipKind`, authored intent,
  derived status, and proposal-first schematic reconstruction.
- Export remains artifact generation from Datum projections at known revisions,
  not an alternate source authority.
- Remaining owner questions are first import slice, provenance visibility
  lifetime, lossiness fail/warn threshold, imported-geometry strictness,
  critical export formats, and inferred schematic labeling.

### `012` Application Quality Bar

- Post-worker status: implementation-ready draft.
- Quality gates now include crash-safe `commit()`/journal/recovery, durable
  undo, resolver recovery, projection honesty, ZoneFill honesty, artifact
  traceability, no private writers, and PTY-real terminal behavior.
- Stale derived state is acceptable only when visibly marked and recoverable by
  revision-keyed recomputation.
- Remaining owner questions are first performance fixture, numeric budgets,
  mandatory crash/recovery behavior, experimental labeling, diagnostic bundle
  scope, latency budgets, artifact traceability format, and CI failure cases.

## Acceptance Checklist

Use this as the final pass after workers edit `002` through `012`.

- Each target doc says or implies one resolved `DesignModel` authority assembled
  by `ProjectResolver`; no doc assigns authority to a sheet, board, tab,
  library file, output file, workspace layout, terminal session, assistant
  session, imported file, or generated artifact.
- Every target doc that mentions edits names the operation/proposal/transaction
  path and does not permit private file writes.
- Every target doc that mentions persistence, undo, recovery, revisions,
  checks, projections, or artifacts aligns with the single journaled `commit()`
  model.
- Every target doc that mentions relationship state separates
  `RelationshipKind`, derived status, authored intent, and variant population.
- No target doc stores `Implemented`, `PendingImplementation`,
  `UnresolvedMismatch`, or `NotApplicableForVariant` as authored source state.
- No target doc introduces `ElectricalDeviation`, `ManufacturingOnlyObject`, or
  any other new relationship state as accepted vocabulary unless it is clearly
  marked as an owner-review fork and not first-slice implementation doctrine.
- Every cross-domain component/symbol/package/footprint relation uses
  `ComponentInstance` and stable object IDs, not reference designators.
- Every variant discussion uses sparse overlays, zero base writes on active
  variant switch, stored `Fitted`/`Unfitted`, and derived
  `NotApplicableForVariant`.
- Every import discussion uses Import Map `import_key` and preserves original
  source provenance/unknown-basis markers.
- Every manufacturing projection discussion uses T0/T1 generation, revision
  keys, artifact traceability, and ZoneFill honesty.
- Every panelization discussion keeps panel features in manufacturing/panel
  state and keeps board-only artifacts free of panel-only geometry.
- Every standards/check/import repair path is proposal-first unless a narrow,
  explicit, provenance-recording automation policy is stated.
- Every terminal mention preserves PTY-real shell semantics and denies hidden
  design mutation by parsing terminal text.
- Every assistant/AI mention keeps AI optional, bounded by structured Datum
  APIs, proposals, checks, diffs, provenance, and explicit approval.
- Every compliance claim avoids certification wording and ties reports to
  modeled standards basis, uncertainty, check state, and revisions.

## Known Risks

- Relationship vocabulary drift remains the highest ongoing risk, but the
  immediate `003` conflict has been reconciled. Future edits must keep
  `RelationshipKind`, derived status, authored intent, and variant population
  separate.
- Source-shard language can regress into hidden separate authorities if target
  docs talk about schematic files, board files, library files, workspace files,
  or generated outputs without restating resolver authority.
- Private writer exceptions can be normalized accidentally. The known GUI
  board-text write path is a migration defect to retire, not a sanctioned
  shortcut.
- Import identity can regress if future edits treat `source_hash` as the
  identity reload key. The ratified mechanism is Import Map keyed by
  `import_key`.
- Variant overlays can become full copies if docs do not repeat sparse overlay,
  zero base writes, and derived `NotApplicableForVariant`.
- Zone fill can become dishonest if live CAM docs or manual editor docs imply a
  boundary polygon is exportable copper.
- Standards docs can overclaim if "compliant" or "certified" language appears
  without the explicit modeled-basis and evidence limits.
- Terminal and assistant docs can become product shortcuts if they are allowed
  to compensate for incomplete manual editor workflows.
- Artifact metadata can be treated as optional. It is required for authority
  flip, live projection equivalence, stale artifact detection, and support.

## Final Integration Criteria

The 002-012 decision family is implementation-ready only when the following
criteria are all true.

1. The target docs read as one mechanism: resolved `DesignModel` authority,
   source shards as partitions, `ProjectResolver`, stable identity,
   `ComponentInstance`, explicit relationships, sparse variants, live
   projections, artifacts, proposals, transactions, and journaled `commit()`.
2. There is no conflict between target-doc vocabulary and the ratified spine.
   Any additional concept is either mapped to ratified vocabulary or explicitly
   labeled as an unresolved owner question outside first-slice doctrine.
3. Every first proof slice can be implemented using typed operations,
   proposals where required, a single commit path, durable undo/redo,
   model/object revisions, and machine-readable provenance.
4. Every derived surface has an invalidation story keyed by model/object,
   variant, output-job, manufacturing-plan, generator, or artifact revisions.
5. Every high-risk workflow has a user-review boundary: relationship changes,
   ECO/back-annotation, import repair, standards/process correction, waiver,
   deviation, panel-to-board promotion, library update propagation, and
   AI/tool-originated edits.
6. The docs give implementers enough pass/fail hooks for drift gates:
   identity substrate, resolver recovery, commit atomicity and durable undo,
   shard diff isolation, proposal parity, live-CAM equivalence, panelization
   isolation, variant resolution, artifact traceability, and harness wiring.
7. Manual workflows remain real. AI, assistant, terminal, import, and external
   tooling can accelerate or inspect work, but they cannot be required to make
   core schematic, PCB, library, rules/checks, or manufacturing workflows
   appear complete.
