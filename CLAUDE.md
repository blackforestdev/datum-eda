# Datum EDA — controlling agent instructions

Datum is a professional, manual-first, headless-first Linux EDA application with
optional AI collaboration. Native library → schematic → PCB → manufacturing is
the product flow; import/export supports it. Datum is not an importer, viewer or
AI-only shell. Use `Datum EDA` in prose and `datum-eda` for machine identifiers.
The CLI executable is `datum-eda`; do not introduce legacy placeholder names.

## Portable Datum workflows

<!-- DATUM-WORKFLOW-CATALOG:datum://workflows -->

`datum://workflows` is the canonical portable workflow inventory. Repository
instructions must not redefine its capabilities, gates or mutation semantics.

## Ethos (controlling)
- **Manual-first.** Every core EDA workflow must be possible without AI. If a
  user cannot perform a workflow manually, it is not complete — even if an
  agent can perform or fake it. AI may assist, propose, explain, automate,
  and review; it is never required and never a hidden authority.
- **Optional first-class AI.** AI is a collaborator that operates through the
  same deterministic primitives a user does — stable IDs, queries, typed
  operations, proposals, checks, diffs, provenance — never private edit powers.
- **One canonical `DesignModel`.** Schematic, PCB, library, rules,
  manufacturing, and analysis are projections over one model assembled by an
  engine-owned resolver — not separate authorities. Files are persistence
  partitions; the resolved model is the authority.
- **Product North Star.** Datum is a professional native EDA system whose
  foundation is governed library plus schematic authority. The normal product
  flow is library -> schematic -> PCB -> manufacturing; import/export are
  compatibility paths, not the center of the roadmap.
- **One mutation path.** Every committed design change — manual, CLI, MCP,
  agent, import-repair, check-fix — reduces to typed operations through one
  `commit()` + journal, with provenance, diff, and undo. No private writers.
- **Lean.** Keep Datum clean and efficient: a capability should be a
  parameter of a small verb set, not a tool per object-class-times-format. A
  redundant tool is a defect.
- **Shared by construction.** Any capability shared by more than one editor —
  grid, camera, snap, selection, undo, measure, the property inspector,
  geometry/constraint solving — is built ONCE as a shared backbone and
  *configured* per editor, never reimplemented per editor. An editor is a thin
  persona (the verbs it exposes, how its objects render, its one contextual
  inspector) over services it inherits: **the tools change, the scene doesn't.**
  Reimplementing a shared capability for a single editor is a defect — the same
  defect as a redundant tool (this is *Lean* applied to editor architecture). The
  field converges on this independently (FreeCAD workbenches, Fusion workspaces,
  Affinity personas, Bitwig's global transport), so it is a law, not a preference.
  Full treatment: `docs/DATUM_SHARED_TOOLING_TAXONOMY.md`; first ratification in
  `docs/decisions/PRODUCT_MECHANICS_023_UNIVERSAL_VIEWPORT_TOOLING.md`.

> Controlling product doctrine lives in `docs/DATUM_PRODUCT_MECHANICS.md`, the
> ratified decision records in `docs/decisions/` (`PRODUCT_MECHANICS_000..030`),
> and the per-domain tool contracts in `docs/contracts/`. Read those before
> inferring product intent from code or a milestone.

## Start with the task, not the documentation inventory

Read this file and AGENTS.md at startup. Then load only the relevant governed
contract, implementation owner and task evidence. Detailed action procedures live
in `docs/AGENT_OPERATIONAL_PROCEDURES.md`; its reading map identifies when to load
each section. Architecture/milestone context is in
`docs/PROJECT_ARCHITECTURE_REFERENCE.md` and is not startup or next-task authority.
Do not recursively read every path mentioned in a reference. This does not exempt
the required complete owning-route review for specification or visual work.

Authority order: this file; product mechanics and numbered decisions; domain tool
contracts; roadmap/status. Runtime incompleteness cannot override ratified rules
or visual prototypes. Research unresolved product questions before implementing
new mechanisms; ordinary choices inside an execution grant do not restart planning.

## Delivery and management

PM059 controls build → test → fix → document → independent review → delivery.
The unit is complete designer behavior in the ordinary application. Continue
routine corrections under the retained grant; commits, evidence packets and lease
renewals are not stopping points. Capture unrelated issues without expanding scope.

An owner-appointed manager coordinates actual independent review receipts,
unchanged-scope continuation and objective acceptance against existing criteria.
Record appointment and bounded disposition in existing task evidence. Implementers
do not self-appoint or self-review. New semantics, scope reductions, waivers,
dependencies, destructive actions and nondelegated HUMAN/UX decisions remain
owner-reserved. PM059 amends owner-boundary routing only; it does not bypass
selector validation, claims, missing evidence or successor authorization. No
management session is automatically appointed by these instructions.

## Roadmap and editing authority

Use fresh `python3 scripts/project_status.py next` or `details [key|issue]`
for ordinary task/completion queries and return stdout byte-for-byte; never infer
selection or substitute tracker readiness. Report selector failure and stop the
query. Stop implementation at an INPUT REQUIRED boundary; PM059 governs any
appointed manager's exact delegation. The complete presentation, response-packet,
claim and transition procedures belong to
[PROJECT_STATE_POLICY.md](docs/PROJECT_STATE_POLICY.md#standard-commands).
Synchronize the Frontier lease and beads before scheduled edits.

Read-only audits/research need no implementation claim or successful selector;
disclose unrelated state failures and continue the investigation. This exception
authorizes no production edits, dependency, acceptance or canonical-next claim.
Tracked planning/spec edits still need authorization and same-change governance.

## Specification and visual authority

Before reviewing, implementing or changing a specification/decision/GUI contract,
inspect `specs/evidence_traceability_manifest.json` and review all listed sources
and consumers for its owning route. Reconcile conflicts through the owning
authority. Source/consumer edits require deliberate digest reconciliation in the
same change. Every active specification needs classification and roadmap placement;
new mechanisms require a numbered decision. Follow the detailed same-change
checklist in the operational procedures; no orphan specifications.

Claude exclusively owns every `docs/gui/prototypes/*.html` file. Codex may inspect
and screenshot, never modify, format, rename, delete or regenerate them. Return
bounded changes to Claude. Never set/copy/forward the lane marker or refresh
digests/goldens to launder an out-of-lane change. For native translation inspect
prototype DOM/CSS as well as screenshots; verify matching relevant states.
Capture and inspect visual changes before presenting them.

## Verification must earn its cost

State the requirement/risk, evidence gap and decision enabled per cohesive batch.
Reuse adequate evidence; a new session or documentation edit is not a rerun reason.
Use the smallest meaningful checks; do not add implementation-mirroring tests.
Keep required acceptance, negative controls, independent review and commit gates.
Broaden only for affected changes, failures or unresolved concerns. Bound expensive
experiments and preserve failed attempts; no retries-until-green or parameter sweeps.
Keep raw logs once and reference them. Ordinary justified verification needs no
additional owner approval. Do not create a separate proof framework or ledger.

Baseline, aspiration and acceptance budget are different. No invented blocking
number: PM051 requires derivation, representative evidence, configuration, limits
and product rationale. Approval alone cannot supply missing evidence. Keep
unsupported figures undecided; review feasibility/value after repeated failure.
Read the numerical-provenance procedure before changing a threshold or measurement
campaign.

## Nonnegotiable implementation boundaries

- PM029: no adding/fetching/vendoring/linking third-party code without prior exact
  owner ratification and license obligations in a numbered decision. The terminal
  emulator/VT/PTY/session/fallback implementation is first-party only.
- PM022 source health: no new oversized modules or invented ceiling exceptions.
  Touching legacy debt requires real ownership extraction and a downward ratchet;
  forwarding/include splits are not decomposition. Load its policy before such edits.
- Every write uses the canonical engine mutation/journal route. Test against real
  designs; use governed fixtures, not invented product evidence.
- Rust proof/Clippy/release/smoke uses the guarded runner; no proof artifacts in
  `/tmp`, parallel proof builds or shared-cache cleanup while Cargo/rustc is active.
- Scoped formatting uses `scripts/format_rust.py`; never mutating workspace-wide
  formatting or `cargo fmt -- <paths>` in the shared worktree.
- Commit only owned work, explicitly staged, directly on main; substantive
  conventional subject/body and no AI attribution anywhere. Force-push/history
  rewrite/`--no-verify` needs explicit owner authority.
- WDQ blocking enforcement is retired (PM044). Use `scripts/git-hooks`; no ordinary
  enrollment, source-scope promotion or historical pilot replay. Retained tooling
  is manual history/diagnostics, not permission to reinstall it.

Terminal completion requires the applicable PM027–030/native terminal contracts,
not merely focus, PTY output or a cell grid. Imported KiCad conversion remains
terminally frozen under PM011 unless explicitly authorized work changes that scope.
Current capabilities and remaining work come from the selector and domain evidence,
not the historical architecture reference.
