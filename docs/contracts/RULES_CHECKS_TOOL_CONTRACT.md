# Rules & Checks Tool Contract

Status: draft implementation-spec 2026-06-19; derived from ratified
PRODUCT_MECHANICS 000-012

## Driven by

- `docs/decisions/PRODUCT_MECHANICS_000..012` (ratified product-mechanics
  decision layer)
- `docs/audits/scope-integration/DATUM_PRODUCT_MECHANICS_IMPLEMENTATION_READINESS_AUDIT.md`
  (authoritative ratified vocabulary and invariants)
- `docs/contracts/AI_CLI_MCP_TOOL_SURFACE.md` (the seven shared
  operations — referenced, never restated here)
- Existing engine surfaces:
  - Rule model: `crates/engine/src/rules/ast.rs`,
    `crates/engine/src/rules/validate.rs`, `crates/engine/src/rules/eval.rs`
  - DRC: `crates/engine/src/drc/mod.rs`
  - ERC: `crates/engine/src/erc/mod.rs`
  - Unified report: `crates/engine/src/api/query_surface.rs:179`,
    `crates/engine/src/api/check_summary.rs`
  - Waivers: `crates/engine/src/schematic/mod.rs:375` (`CheckWaiver`,
    `WaiverTarget`)
  - Rule write op: `crates/engine/src/api/write_ops/assign_package_rule.rs:272`,
    `crates/engine/src/api/mod.rs:131` (`TransactionRecord::SetDesignRule`)
  - Daemon dispatch: `crates/engine-daemon/src/dispatch.rs:259/439/447/451/455/467`
  - MCP catalog: `mcp-server/tools_catalog_data.py`

## Purpose & Scope

This contract specifies the concrete authoring/tool surface for the
**rules** domain: how a user, the CLI, and an AI agent author a design
rule, run checks, read findings, and dispose of or repair a finding.
It binds those actions to the ratified mechanism spine — typed
`Operation`/`OperationBatch` through one `commit()`; `Proposal` for
high-risk/AI-originated mutations; the engine-owned `ProjectResolver`
assembling one `DesignModel` at a known `model_revision`; and derived
state (`CheckRun`/`CheckFinding`) that is revision/hash-keyed and never
authority.

In scope: the rule object, check execution and findings, waiver/
deviation disposition, and finding-driven repair. Out of scope: the
mechanics of `commit()`, the proposal lifecycle, the query namespace,
the journal, session/context — all owned by the **Shared Surface**
(see cross-reference below). This doc adds only the rules-domain typed
operations and the rules-domain `aiQueryContext`.

The rule object is one Altium-style typed row: `rule_type` + scope +
parameters + priority + enabled + severity. Severity, net-class scope,
and standards basis are **parameters of the rule**, not separate tools.
Domain (ERC vs DRC) and check profile are **parameters of run_check**,
not separate tools. Edit-time vs full-batch is a **mode flag**, not a
tool.

CLI naming note: `datum-eda` is the canonical CLI executable. `eda` and bare
`datum` command examples are legacy/noncanonical; `datum.*` remains reserved
for MCP tool family names only. Rules/checks commands group as
`datum-eda rules set ...`, `datum-eda check run|show ...`,
`datum-eda proposal ...`, and proposed `datum-eda check waive ...`.

## Reference-Tool Survey (with the lean rationale)

**Altium Designer (primary reference).** One PCB Rules & Constraints
Editor with a single rule object: typed kind + query-language scope +
severity + priority, with binary rule resolution per object-pair.
Online DRC vs batch DRC is a mode, not a separate command. A Violations
panel gives per-violation drill-down; ERC is rule-driven via the
connection matrix.
*Load-bearing:* ONE rule object (typed kind + scope + severity +
priority); online-vs-batch as a mode; per-violation drill-down with
explanation in the panel.
*Ceremony to omit:* dozens of overlapping rule sub-UIs and the
heavyweight ad-hoc query builder.

**KiCad 7/8.** Board Setup Custom Rules DSL (`rule`/`condition`/
`constraint`), net classes, DRC + ERC dialogs producing a violations
list with exclusions (the waiver equivalent, stored in `.kicad_pcb`),
and a per-rule-code severity map. `kicad-cli pcb drc` / `sch erc` emit
JSON reports — the headless precedent for **one report verb**.
*Load-bearing:* small textual rule object; exclusion-as-waiver;
severity-on-rule; one headless report verb.
*Ceremony to omit:* per-violation manual exclusion churn with no
grouping.

**OrCAD/Cadence Allegro Constraint Manager.** Spreadsheet of reusable
constraint sets bound to net classes/regions, online/batch DRC modes,
waivers as DRC markers.
*Load-bearing:* constraint-set reuse + class/region binding (Datum's
`CheckProfile` analog).
*Ceremony to omit:* the deep electrical-constraint-set matrix
hierarchy — overkill for a first slice.

**Horizon EDA.** Ordered rule list with enable/priority + `RuleMatch`
(net / net-class / pattern); `rules_check` produces results; discrete
`RuleId` kinds (clearance / track-width / etc.).
*Load-bearing:* confirms a lean **ordered rule object with typed
match** beats a query-language engine for v1 — almost exactly Datum's
existing `rules/ast.rs` shape.

**Lean rationale.** Every reference tool reduces this domain to four
verbs: author a rule, run checks and read findings, suppress/accept a
finding, fix a finding. The differences are surface ceremony (query
builders, sub-UI proliferation, manual exclusion churn). Datum already
has the lean rule object Horizon validates and the merged ERC+DRC
summary Allegro/Altium expose; the contract therefore extends what
exists rather than adding parallel engines.

## Tool Inventory

Four load-bearing tools. Each answers the 8 questions as labeled
fields. Shared mechanics (`commit()`, proposal lifecycle, query
namespace, session) are referenced, not restated.

---

### Tool 1 — `set_rule`

1. **Manual UI action.** In the Rules editor, create or edit one
   constraint row: pick `rule_type`, write the scope expression, set
   parameters, priority, enabled, and severity. One Altium-style rule
   object. Severity and net-class scope are fields **on this row**, not
   separate editors.
2. **Operation it emits.** `TransactionRecord::SetDesignRule
   {previous, current}` (exists — `api/mod.rs:131`; impl
   `assign_package_rule.rs:272`); journaled and undoable. 009 widens it
   to carry `severity_default` + `basis_refs`/`source_constraint_refs`
   and to write to a **rules source shard** rather than mutating
   `board.rules` in place. This is a **direct-commit** operation (local,
   visible, undoable — see Shared Surface proposal/apply contract).
3. **CLI command.** `datum-eda rules set --rule-type clearance_copper
   --scope <expr> --param min=100000 --priority 1 [--name ...]
   [--severity error|warning] [--basis <ref>]`. Today this is reachable
   only via project/modify surfaces + daemon `set_design_rule`;
   the legacy project/modify surface already sets the default all-scope
   clearance. **Gap:** no dedicated `rules set` verb yet.
4. **MCP/AI tool.** `set_design_rule` (live —
   `tools_catalog_data.py`, `dispatch.rs:259`): `rule_type` / `scope` /
   `parameters` / `priority` / `name`. **Needs:** `severity` +
   `basis_refs` added.
5. **AI query/context needed.** MCP query family `datum.query.rules.query`
   for the
   current rule list keyed at `model_revision`; `NetId` / `NetClassId` /
   `ComponentInstance` ids (never names) to author scope; the standards
   basis registry to attach `basis_refs`. (Folds under the shared
   `DatumQueryTool` namespace; reuses existing `get_design_rules`.)
6. **Validating checks.** `rules/validate.rs::validate_rule` (positive
   minima; `min <= preferred <= max`); the `ProjectResolver` must
   validate that `RuleScope` resolves to live object ids at the
   `model_revision`.
7. **Proof slice.** `datum-test.kicad_pcb`: set a project clearance and
   a process-aperture (mask/paste) rule, re-run check, confirm findings
   appear/clear, and confirm the `SetDesignRule` transaction is
   undoable.
8. **Explicitly not-supported-yet.** Combinator/regex/`IsDiffpair`
   scopes parse but error on eval (`ast.rs:18-32`); no `severity_default`
   or `basis_refs`/`source_constraint_refs` on `Rule`; impedance/
   length-match/diffpair `rule_type`s absent (`ast.rs:45`); still mutates
   `board.rules` in place rather than a rules source shard.

---

### Tool 2 — `run_check`

1. **Manual UI action.** Run a check set and view the unified Findings
   panel: error/warning/info/waived counts, per-finding location,
   affected objects, expected-vs-actual, explanation, and suggested
   action. Edit-time vs full-batch is a **mode flag** on this verb; ERC
   and DRC are **domain filters**, not separate tools.
2. **Operation it emits.** NONE — check execution never mutates design
   truth (009 step 8). It SHOULD persist a `CheckRun`/`CheckFinding`
   record keyed by `model_revision` + variant + `input_object_revisions`
   with a deterministic fingerprint, recorded as generated **evidence**
   (a derived artifact, not a design op). Current native CheckRun evidence is
   persisted under `.datum/check_runs`. `explanation` +
   `suggested_next_action` live **on the finding record**, so no separate
   explain tool is needed.
3. **CLI command.** `datum-eda check run <path> [--fail-on warning|error]
   [--profile <id>] [--domain erc|drc|all] [--mode edit|batch]` (live —
   `query_check.rs::run_check`/`check_exit_code`) and
   `datum-eda check show <check_run_id>` for persisted reports.
   Legacy top-level DRC/ERC wrappers are noncanonical and kept ONLY as thin
   compatibility wrappers.
4. **MCP/AI tool.** `get_check_report` (live — `dispatch.rs:439`):
   unified Board/Schematic/Combined report carrying findings WITH
   `explanation` + `suggestion` fields. `run_erc` (`451`) and `run_drc`
   (`455`) collapse into this with a domain/profile param; keep as
   aliases only. (See Shared Surface `DatumCheckTool`.)
5. **AI query/context needed.** `model_revision` + `active_variant_ref`
   to key findings; `CheckProfile` selection (which rule set); standards
   basis in scope; the prior `CheckRun` id for delta comparison; finding
   fingerprints for downstream waive/deviation/repair.
6. **Validating checks.** Determinism gate: same `model_revision` +
   variant + rule versions => identical finding fingerprints. ZoneFill
   honesty: an `Unfilled`/`Stale`/`Unsupported` zone must not pass copper
   checks as if its boundary were copper (009). Fingerprint stability
   across rename/move via `ComponentInstance` + `ObjectId`.
7. **Proof slice.** `datum-test`: run check, get the unified report with
   summary counts and a paste/mask `process_geometry` finding carrying
   its explanation inline; assert exit code under `--fail-on`; re-run and
   confirm identical fingerprints.
8. **Explicitly not-supported-yet.** No persisted `CheckRun`/
   `CheckFinding` with the revision/variant/`input_object_revisions` key
   + deterministic fingerprint; not variant-aware (no overlay
   composition before run, 009 step 2); `run_drc` rule set hard-coded
   (`dispatch.rs:455`); only one process-geometry category (paste/mask);
   no declared/inferred/unknown-basis distinction. Current native live and
   persisted findings already include `explanation` and
   `suggested_next_action` fields.

---

### Tool 3 — `waive_finding`

1. **Manual UI action.** Right-click a finding > Waive; enter rationale
   and scope. Waived findings stay visible and auditable (KiCad
   exclusion / Allegro waiver analog). Removing a waiver is **undo** of
   this op, not a separate verb.
2. **Operation it emits.** MUST become an `AddWaiver` Operation through
   `commit()` + journal (009: waiver creation is an `OperationBatch`).
   This is a **proposal-first** class action (AI may propose, never
   silently waive). TODAY there is NO journaled waiver op — `CheckWaiver`
   lives in `schematic.waivers` and is applied only at run time
   (`drc/mod.rs:119` `apply_waivers`). **This is the single biggest
   contract gap in the domain.** Record-as-intent (deviation) is a
   **disposition value** carried by this same op family, not a separate
   tool for the first slice (see Minimal-Set Recommendation).
3. **CLI command.** No waiver verb yet (gap). Proposed taxonomy:
   `datum-eda check waive --finding <fingerprint> --rationale ...
   [--disposition suppress|accepted-deviation] [--expires <date>]`.
4. **MCP/AI tool.** NONE yet (gap). Per 009 AI may PROPOSE a waiver but
   may not silently waive — so this is a **proposal-producing** surface
   (routes through the shared `DatumProposalTool`), not a direct AI
   write.
5. **AI query/context needed.** Finding **fingerprint(s)** to cover (NOT
   `(domain, index)`); rationale; actor; scope; the `model_revision` the
   waiver is approved against; disposition (suppress vs
   accepted-deviation).
6. **Validating checks.** The waiver must match a real finding
   fingerprint; it must not flip error->ok silently (the summary keeps a
   waived count — `check_summary.rs` already tracks `waived`); waived
   findings stay visible in the report.
7. **Proof slice.** `datum-test`: waive one DRC finding with rationale
   via a committed op; re-run check; confirm it moves to **waived** (not
   removed) and is journaled/undoable.
8. **Explicitly not-supported-yet.** No waiver-creation Operation/
   `TransactionRecord`; `CheckWaiver` (`schematic/mod.rs:375`) has only
   `uuid`/`domain`/`target`/`rationale`/`created_by` — no `expires_at`/
   `review_policy`/`acceptance_transaction_id`/`disposition`; target is
   keyed by raw object `Uuid` + rule string (`WaiverTarget`), not by a
   deterministic finding fingerprint; finding state is only `waived:
   bool` (no `Deviated`/`Resolved`/`Stale`/`Superseded` enum).

---

### Tool 4 — `propose_repair`

1. **Manual UI action.** From one or more findings, preview a suggested
   fix (e.g. set pad process aperture, apply footprint process policy,
   assign pin-pad-map) showing the candidate operation batch, the
   expected-resolved findings, the expected-remaining findings, and the
   risks before applying. Mirrors the existing route-proposal preview
   UX.
2. **Operation it emits.** Produces a `RepairProposal` (a candidate
   `OperationBatch`); ACCEPT -> a normal journaled transaction via
   `commit()`. Check-generated fixes are **proposals by default** (009).
   Underlying ops REUSE existing typed ops (e.g. `set_design_rule` /
   pin-pad-map assign) — `propose_repair` is a grouping/proposal
   generator, **not a new mutation op**.
3. **CLI command.** No dedicated proposal generator yet (gap); routes
   through the generic proposal surface —
   `crates/cli/src/command_project_route_proposal.rs` is the established
   precedent to mirror (`datum-eda proposal create|show|validate|apply`).
4. **MCP/AI tool.** NONE yet (gap). This is the primary AI value-add:
   group related findings and propose bounded repairs; must list
   `expected_resolved_findings` + `expected_remaining` (009). Routes
   through the shared `DatumProposalTool`.
5. **AI query/context needed.** The source `CheckRun` id + finding
   fingerprints; affected `ObjectId`s via `ComponentInstance`; the rule
   basis; the `model_revision` the proposal is built against.
6. **Validating checks.** After applying, a re-run must clear exactly
   `expected_resolved_findings` and produce no unexpected new errors
   (proposal-parity gate); repairs target `ComponentInstance`/`ObjectId`,
   never refdes/names (009).
7. **Proof slice.** `datum-test`: from a paste-aperture finding, propose
   `SetPadProcessAperture`, preview the diff, accept, re-run, confirm the
   finding clears and the transaction is journaled/undoable.
8. **Explicitly not-supported-yet.** No `RepairProposal` type; no
   proposal-from-finding generator; no proposal-parity harness for
   checks; the only existing proposal machinery is for routing
   (`command_project_route_proposal.rs`), not checks.

## Minimal-Set Recommendation

**FOUR load-bearing tools.** The domain reduces to: AUTHOR a rule
(`set_rule`), RUN checks and read findings (`run_check`), SUPPRESS/ACCEPT
a finding (`waive_finding`), and FIX a finding (`propose_repair`).

- `set_rule` and `run_check` exist in the engine and need **extension**:
  `severity` + `basis_refs` on the rule; a persisted variant/revision-
  keyed `CheckRun` with stable-fingerprint addressing and inline
  explanation.
- `waive_finding` and `propose_repair` are **real gaps** and are the two
  journaled/proposal mutation surfaces 009 requires.

Everything else is a **parameter, not a tool**:

- Edit-time vs batch is a **mode flag** on `run_check`.
- Severity is a **parameter** of `set_rule` (or a `CheckProfile`
  override — `ErcConfig.severity_overrides` already exists).
- Net-class authoring is **scope + param** on `set_rule`; net-class
  membership belongs to connectivity/board.
- Standards-aware DRC is an added `rule_type` + `basis_refs` + a
  standards `CheckProfile`, **not a parallel compliance tool**.
- Profiles are a **named selection** passed to `run_check`.

Two cuts beyond the proposer's own omissions are made below; they are
the load-bearing lean decisions for this domain.

## Omitted / Redundant Tools

- **`explain_finding` (as a standalone domain tool / new CLI verb).**
  Cut. Explanation, basis reference, expected/actual, and
  `suggested_next_action` are **fields on the `CheckFinding`** (009
  schema) that `run_check`/`get_check_report` already return; a separate
  tool re-fetches data it already has. The interactive MCP
  `explain_violation` may survive as a thin drill-down alias, but it is
  not a canonical domain tool, and its current `(domain, index)`
  addressing (`project_surface.rs:326`) is a **defect** — addressing must
  be by stable finding fingerprint. Altium/KiCad surface explanation in
  the violations panel/report, not a separate command, so no new CLI
  verb is warranted.

- **`record_deviation` (as a separate tool in the first slice).**
  Merged into `waive_finding` as a `disposition` value (`suppress` |
  `accepted-deviation`). 009 and the 003 `ElectricalDeviation` note
  explicitly defer the standalone `Deviation` primitive to an
  owner-review FORK and direct first-slice behavior to
  accepted-deviation metadata on existing records. A deviation differs
  from a waiver only by disposition + `basis_ref` and would otherwise
  duplicate the same proposal->commit->journal path with the same
  fingerprint addressing. Graduates to its own tool ONLY after the owner
  ratifies the primitive and its approval state machine.

- **`run_erc` / `run_drc` as separate top-level tools.** Redundant.
  Both exist (`dispatch.rs:451/455`) but are domain-filtered,
  non-variant-aware slices of `run_check`/`get_check_report`; the
  resolved `DesignModel` is one authority and `check_summary.rs` already
  merges ERC+DRC. Collapse into `run_check` with a domain/profile
  parameter; keep only as thin compatibility aliases. `run_drc`'s
  hard-coded 7-`RuleType` array (`dispatch.rs:455-463`) must be replaced
  by an explicit `CheckProfile` param.

- **`set_severity` / severity-override tool.** Redundant. Severity is a
  property of a rule (`severity_default`, 009) or a `CheckProfile`
  override (`ErcConfig.severity_overrides` already exists). A parameter
  of `set_rule` + profile config, not a tool. Altium/KiCad edit severity
  inside the rule.

- **Net-class CRUD as a checks-domain tool.** Redundant. Net class is a
  scope target (`TransactionRecord::SetNetClass` exists); constraints
  attach via `set_rule scope=NetClass(..)`. A checks-domain net-class
  tool duplicates the rule-scoping path; net-class membership belongs to
  connectivity/board, referenced by rule scope.

- **`list_findings` / `get_findings`.** Redundant. Findings are the
  payload of `run_check`/`get_check_report`; a separate list endpoint is
  a redundant read. Filtering (severity/category/object) is a query
  parameter, not a tool (folds under `DatumQueryTool` `checks.query`).

- **`standards_check` / `compliance_check` as its own tool.** Redundant.
  009 states standards-aware DRC is a **normal check path**, not a
  separate engine — `run_check` with standards-basis rules + a
  standards-focused `CheckProfile`. A parallel compliance tool risks the
  certification overclaim the audit forbids and duplicates the check
  path.

- **`clear_waiver` / `delete_deviation`.** Redundant. Removal is **undo**
  of the `AddWaiver`/`CreateDeviation` transaction (journal cursor) or a
  compensating op, not a distinct tool. The single `commit()` + journal
  model already gives durable undo; a bespoke delete verb is a private
  path the audit forbids.

## Shared Surface

This domain consumes the seven shared operations defined in
`docs/contracts/AI_CLI_MCP_TOOL_SURFACE.md`. They are referenced by name
and **not restated** here:

- **`DatumToolSession` + `DatumContextEnvelope`** — every `set_rule` /
  `run_check` / `waive_finding` / `propose_repair` invocation runs inside
  a session and refreshes context before propose/apply if its envelope
  `model_revision` is stale.
- **`DatumQueryTool`** — `rules.query` and `checks.query` families
  supply the `aiQueryContext` for all four tools (rule list, finding
  fingerprints, prior `CheckRun` for delta). Reuses existing
  `get_design_rules`/`get_check_report`. Selection is consumer state,
  never an op.
- **`DatumCheckTool`** — `run_check` IS this surface for the rules
  domain; `run_erc`/`run_drc` are its compatibility aliases. CheckRun/
  CheckFinding revision/fingerprint-keyed; determinism and ZoneFill-
  honesty gates apply.
- **`DatumProposalTool`** — `waive_finding` (`AddWaiver`) and
  `propose_repair` (`RepairProposal`) are proposal PRODUCERS that reuse
  existing typed ops; they do not define their own apply path.
- **`DatumCommitTool`** — the SOLE mutation gateway. `SetDesignRule`,
  `AddWaiver`, and accepted repair batches all flow through the one
  `commit()`. No rules-domain private write path.
- **`DatumArtifactTool`** — not used by this domain (checks are derived
  evidence, not artifacts), except that a persisted `CheckRun` is
  recorded as derived evidence via `commit()`.
- **`DatumJournalTool`** — undo/redo of a `SetDesignRule`/`AddWaiver`/
  repair transaction is a compensating batch through the same `commit()`.
  This domain ships NO `clear_waiver`/`delete_deviation` verb.

## Proof Slice & Fixture

Fixture: `~/Documents/kicad_projects/Datum-eda/datum-test/`
(`datum-test.kicad_pcb` + `datum-test.kicad_sch` present). The board
drives DRC/process-geometry (pad/mask/paste, clearance, via, track-width
— the existing 7 checks); the schematic drives ERC.

First-slice proof (per 009):

1. Import `datum-test.kicad_pcb`.
2. `set_rule`: set a pad/mask/paste process basis (with `severity` +
   `basis_ref`).
3. `run_check`: emit a structured mask/paste-aperture finding carrying
   its `explanation` + `suggested_next_action` INLINE (no separate
   explain call). Assert the exit code under `--fail-on`. Re-run and
   assert **identical finding fingerprints** at the same `model_revision`
   (determinism gate).
4. Exercise the two missing gates:
   - a committed/journaled `waive_finding` (use
     `disposition=accepted-deviation` on the same path to prove the
     merged deviation case), confirming the finding moves to **waived**
     (not removed) and is undoable;
   - a `propose_repair` (`SetPadProcessAperture`) that clears the
     finding on re-run and is undoable (proposal-parity gate: exactly
     the expected-resolved set clears, no new errors).

## Not-Yet-Supported

- No persisted `CheckRun`/`CheckFinding` keyed by `model_revision` +
  variant + `input_object_revisions` with a deterministic fingerprint —
  reports are ephemeral today.
- Not variant-aware: no overlay composition before a run (009 step 2).
- `run_drc`'s rule set is hard-coded in the daemon
  (`dispatch.rs:455-463`) instead of taking a `CheckProfile`.
- Only one process-geometry category exists (paste/mask —
  `process_aperture`, `drc/mod.rs:83`); 009 lists copper-pad,
  drill/annular, mask, paste, courtyard, density-level, IPC naming.
- No `severity_default` / `basis_refs` / `source_constraint_refs` on
  `Rule`; no declared/inferred/unknown-basis distinction on findings.
- Combinator/regex/`IsDiffpair` scopes parse but error on eval
  (`ast.rs:18-32`); impedance/length-match/diffpair `rule_type`s absent
  (`ast.rs:45`).
- No waiver-creation Operation; `CheckWaiver` lacks `expires_at`/
  `review_policy`/`acceptance_transaction_id`/`disposition` and is keyed
  by raw object `Uuid` + rule string, not by finding fingerprint.
- Finding state is only `waived: bool` (no `Deviated`/`Resolved`/
  `Stale`/`Superseded` enum).
- No `RepairProposal` type, no proposal-from-finding generator, no
  proposal-parity harness for checks.
- `set_rule` still mutates `board.rules` in place
  (`assign_package_rule.rs:272`) rather than writing a rules source
  shard.

## Open Owner Questions

1. Which `CheckProfile`s ship first (edit-time / full-DRC / import-audit
   / library-release / manufacturing-sign-off), and is profile selection
   a CLI/MCP parameter or a stored project object? (009 leaves this
   open.)
2. Waiver creation must become a journaled `AddWaiver` Operation (today
   it is not). Is the target keyed by deterministic finding fingerprint
   (preferred for stability) or by the current raw object-uuid + rule-
   string `WaiverTarget` (`schematic/mod.rs:375`)? Extending
   `CheckWaiver` with `expires_at`/`review_policy`/
   `acceptance_transaction_id`/`disposition` is required.
3. **Ratification fork:** for the first slice, deviation is merged into
   `waive_finding` as `disposition=accepted-deviation` (per the 003
   `ElectricalDeviation` owner-review fork). Does the owner ratify the
   standalone `Deviation` primitive + approval state machine + a
   `Deviated` finding state now (graduating `record_deviation` to its own
   tool), or keep accepted-deviation metadata on the waiver record until
   later?
4. Default severities for standards-aware process-geometry findings, and
   whether unknown-basis imported footprints fail / warn / group as
   audit findings (009 owner question).
5. How many process-geometry categories are mandatory for the first DRC
   slice — today only paste/mask (`process_aperture`, `drc/mod.rs:83`)
   exists; 009 lists copper-pad, drill/annular, mask, paste, courtyard,
   density-level, IPC naming. Which are gates vs deferred?
6. Should `set_rule` write to a dedicated rules source shard (009/000
   model) instead of the current in-place `board.rules` mutation
   (`assign_package_rule.rs:272`), and how much rule editing must be GUI
   vs CLI/MCP-only for the first slice?
7. Should `run_drc`'s hard-coded 7-`RuleType` set (`dispatch.rs:455-463`)
   be replaced by an explicit `CheckProfile` parameter BEFORE any further
   check tools land, to avoid baking a fixed rule list into the daemon
   contract?
