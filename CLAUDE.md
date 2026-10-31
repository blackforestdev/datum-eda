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

## Work-effort allocation (controlling; owner-ratified 2026-10-31)

Datum is built in Rust. **At least 80% of work effort — what sessions, agents
and turns are actually spent on — goes to writing, fixing and reviewing
actual Rust product code**: the engine, GUI, editors, services and verbs a
designer uses in the running application. Everything else combined — Python
harnesses, qualification corpora, evidence JSON, governance records, trackers,
packets, handoffs, coordination — is overhead, capped at the remaining
fraction. Overhead that outgrows the cap is a defect to cut, not work to
schedule; a governance or qualification artifact never counts as product
progress or fix success. Progress is reported as designer-visible behavior
delivered in the running application; git-history composition (lines/touches
by extension) serves as a coarse diagnostic of the balance in delivery
summaries, never as the definition of effort — padding, splitting or
relabeling work to move a ratio is itself a violation. A
qualification campaign serves a Rust deliverable and runs once, consolidated,
at a stable candidate; its records batch with delivery. When choosing the next
unit of work at equal priority, the Rust product unit always wins. The
standing project manager enforces this allocation continuously; sustained
violation is a delivery failure regardless of how green the process looks.

## Start with the task, not the documentation inventory

Read this file and AGENTS.md at startup. Then load only the relevant governed
contract, implementation owner and task evidence. Detailed action procedures live
in `docs/AGENT_OPERATIONAL_PROCEDURES.md`; its reading map identifies when to load
each section. Architecture/milestone context is in
`docs/PROJECT_ARCHITECTURE_REFERENCE.md` and is not startup or next-task authority.
Do not recursively read every path mentioned in a reference. For specification or visual work, inspect the complete owning route; reuse its
verified unchanged baseline and review the affected delta under the procedures.

Authority order: this file; product mechanics and numbered decisions; domain tool
contracts; roadmap/status. Runtime incompleteness cannot override ratified rules
or visual prototypes. Research unresolved product questions before implementing
new mechanisms; ordinary choices inside an execution grant do not restart planning.

## Delivery and management

PM059 controls build → test → fix → document → independent review → delivery.
The unit is complete designer behavior in the ordinary application. Continue
routine corrections under the retained grant; commits, evidence packets and lease
renewals are not stopping points. Capture unrelated issues without expanding scope.
Select product execution by default; justify planning detours by the blocked
workflow and finite exit condition. Consolidate routine evidence and state updates
with delivery under PM059; no separate receipt set for each ordinary substep.

A verified standing project-manager appointment covers its declared decisions
across current and future tasks until revoked. Route covered product choices,
visual acceptance, phase acceptance and successor decisions to that manager;
do not repeatedly ask the owner to renew authority already granted. Unclassified
decisions require manager classification before disposition only when the grant
includes that capability; otherwise they remain owner decisions. Classification
is not automatic approval.
Explicit owner_only decisions and authority outside the appointment remain owner
boundaries. Record decisions in existing task evidence and permissions in existing
policy/configuration; create no standalone permission or per-session waiver document.
Implementers do not self-appoint or self-review. PM059 preserves selector validation,
claims, actual proof, independent review and explicit successor selection. A
literal human-performed observation still requires that observation; permission
cannot fabricate it. No manager is appointed automatically by this policy.

Owner-authorized cleanup, refocus and reorganization remain separate from product
development. Product-delivery defaults apply within product execution grants;
a bounded maintenance assignment completes its own accepted artifacts and does
not authorize product implementation or successor work. Cleanup findings and
readiness records supply no product grant.

## Roadmap and editing authority

Routine development follows PM059's **one-task workflow**: the verified manager
records the bounded grant, implementer, file scope **and the definition of done —
one or two sentences of designer-visible behavior, fixed before work starts and
never extensible from inside the work** — in one existing task record; the
implementer fixes the Rust behavior, runs affected checks, obtains one actual
independent review and lands it. Work whose done-statement cannot be written
that way does not start. Milestone-sized work gets its done-statement
owner-approved first, as one sentence, before any machinery may exist under it;
acceptance criteria authored and graded by the same process that executes them
anchor nothing (owner-ratified 2026-10-31, from the GP-CM05 overrun). A tracker claim alone grants nothing. Routine
work needs no separate Frontier item/lease, completion graph, availability packet
or qualification campaign. Record concise results with delivery. Explicit execution
holds, conflicting ownership and retained owner-only decisions still stop their
affected scope. Milestone acceptance does not withdraw other recorded grants.

Use fresh `python3 scripts/project_status.py next` or `details [key|issue]`
for roadmap milestone/completion queries and return stdout byte-for-byte; never
infer milestone selection or substitute tracker readiness. Report selector failure
and stop that query. An INPUT REQUIRED boundary stops the identified milestone
decision, not separately authorized routine work. PM059 governs the manager's
exact delegation. The complete presentation, response-packet,
claim and transition procedures belong to
[PROJECT_STATE_POLICY.md](docs/PROJECT_STATE_POLICY.md#standard-commands).
Synchronize the Frontier lease and beads before milestone execution edits.

Read-only audits/research need no implementation claim or successful selector;
disclose unrelated state failures and continue the investigation. This exception
authorizes no production edits, dependency, acceptance or canonical-next claim.
Tracked planning/spec edits still need authorization and same-change governance.

## Specification and visual authority

Before reviewing, implementing or changing a specification/decision/GUI contract,
inspect `specs/evidence_traceability_manifest.json` and review all listed sources
and consumers for its owning route. Reconcile conflicts through the owning
authority. Source/consumer edits require deliberate digest reconciliation in the
same change. Review changed artifacts and affected claims against the verified
unchanged baseline; full rereading is required when that baseline is unavailable
or a change alters the route-wide meaning. A digest is no substitute for review. Every active specification needs classification and roadmap placement;
new mechanisms require a numbered decision. Follow the detailed same-change
checklist in the operational procedures; no orphan specifications.

Claude exclusively owns every `docs/gui/prototypes/*.html` file. Codex may inspect
and screenshot, never modify, format, rename, delete or regenerate them. Return
bounded changes to Claude. Never set/copy/forward the lane marker or refresh
digests/goldens to launder an out-of-lane change. For native translation inspect
prototype DOM/CSS as well as screenshots; verify matching relevant states.
Capture and inspect visual changes before presenting them.

## Verification must earn its cost

Routine development uses focused checks for the changed behavior and its affected
integration/safety boundaries. Exhaustive end-to-end, full-workspace, cross-platform
and statistical qualification runs only for an explicitly designated major milestone
or release candidate. The project manager schedules that milestone; a routine edit,
new commit, lease renewal or evidence-file change does not create one. A line-color
change calls for the affected rendering checks, not an automatic full campaign.

Batch known corrections and use focused regression checks while iterating. Changes
invalidate only affected qualification claims; retained proof keeps its original
candidate and provenance. Establish ordinary designer workflow behavior before
broad qualification. The next milestone needs complete evidence applicable to its
candidate, not automatic wholesale recapture after every edit. Historical proof
alone cannot establish a new candidate's release readiness.
Keep required staged commit checks, relevant safety checks and independent review.
Focused passes establish their stated scope only; they never become a production PASS.

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

### Acceptance bars (controlling; owner-ratified 2026-10-31, from the GP-CM05 overrun)
- An acceptance bar is fixed OUTSIDE the work it grades and FROZEN when work
  starts. New findings become new task records; they never extend a live bar.
  A bar authored and graded by the same process that executes it anchors nothing.
- Before any campaign begins, every requirement gets a producibility check
  against the real environment and engine surfaces. An unproducible requirement
  is a defect IN THE BAR — cut it or route one sentence to the owner; never
  defer it, carry it as OWED debt, or build machinery toward it.
- No acceptance scheme may void all evidence whenever the product improves.
  A defect fix invalidates only its affected rows. A bar that punishes fixing
  the product is defective per se.
- Two consecutive whole-evidence invalidations, or any discovered impossibility,
  triggers a mandatory PM bar review whose allowed outcomes are: shrink the bar,
  or one sentence to the owner. More machinery is the forbidden response.
  "This bar cannot be met as written" is a required escalation, never a failure.

## Nonnegotiable implementation boundaries

- PM029: no adding/fetching/vendoring/linking third-party code without prior exact
  owner ratification and license obligations in a numbered decision. The terminal
  emulator/VT/PTY/session/fallback implementation is first-party only.
- PM022 source health: no new oversized modules or invented ceiling exceptions.
  Touching legacy debt requires real ownership extraction and a downward ratchet;
  forwarding/include splits are not decomposition. Load its policy before such edits.
- Never launch datum-gui, GUI drives, captures or screenshots on the owner's
  live display or active session (owner-ratified 2026-10-31, after three weeks
  of test popups on the owner's desktop). GUI verification runs only in
  isolated owned compositor envelopes (headless/offscreen); a check that
  cannot run isolated is deferred, not run live.
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
